use chrono::NaiveDate;
use serde::Deserialize;
use std::process::Command;

use crate::stats::{DataPoint, RegressionResult, linear_regression};

#[derive(Debug, Clone, Deserialize)]
pub struct Measurement {
    pub date: NaiveDate,
    pub weight_kg: f64,
    pub body_fat_pct: f64,
    pub skeletal_muscle_pct: f64,
    pub resting_metabolism_kcal: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SleepEntry {
    pub date: NaiveDate,
    pub total_sleep_minutes: i32,
    pub rem_minutes: i32,
    pub deep_minutes: i32,
    pub light_minutes: i32,
    pub awake_minutes: i32,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NutritionReport {
    period: NutritionPeriod,
    days: Vec<NutritionDay>,
}

#[derive(Debug, Deserialize)]
struct NutritionPeriod {
    pub since: NaiveDate,
    pub until: NaiveDate,
    pub days: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NutritionDay {
    pub date: NaiveDate,
    pub totals: NutritionTotals,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NutritionTotals {
    pub energy_kcal: f64,
    pub protein_g: f64,
    pub fat_g: f64,
    pub fiber_g: f64,
    pub sugars_g: f64,
}

#[derive(Debug, Clone)]
pub struct MetricSeries {
    pub name: String,
    pub unit: String,
    pub points: Vec<SeriesPoint>,
    pub regression: Option<RegressionResult>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SeriesPoint {
    pub date: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug)]
pub struct DashboardData {
    pub period_since: NaiveDate,
    pub period_until: NaiveDate,
    pub days: u32,
    pub measurements: Vec<Measurement>,
    pub sleep: Vec<SleepEntry>,
    pub nutrition: Vec<NutritionDay>,
    pub body_metrics: Vec<MetricSeries>,
}

impl Measurement {
    pub fn fat_mass_kg(&self) -> f64 {
        self.weight_kg * self.body_fat_pct / 100.0
    }

    pub fn muscle_mass_kg(&self) -> f64 {
        self.weight_kg * self.skeletal_muscle_pct / 100.0
    }
}

pub fn fetch_all(days: u32) -> Result<DashboardData, String> {
    let mut measurements = fetch_measurements(days)?;
    measurements.sort_by_key(|m| m.date);
    let sleep = fetch_sleep(days)?;
    let nutrition = fetch_nutrition(days)?;

    let period_since = measurements
        .iter()
        .map(|m| m.date)
        .chain(nutrition.iter().map(|n| n.date))
        .chain(sleep.iter().map(|s| s.date))
        .min()
        .or_else(|| nutrition.first().map(|n| n.date))
        .unwrap_or_else(|| chrono::Local::now().date_naive());

    let period_until = measurements
        .iter()
        .map(|m| m.date)
        .chain(nutrition.iter().map(|n| n.date))
        .chain(sleep.iter().map(|s| s.date))
        .max()
        .unwrap_or(period_since);

    let body_metrics = build_body_metrics(&measurements, period_since);

    Ok(DashboardData {
        period_since,
        period_until,
        days,
        measurements,
        sleep,
        nutrition,
        body_metrics,
    })
}

fn run_command(cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run {cmd}: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{cmd} failed: {stderr}"));
    }

    String::from_utf8(output.stdout).map_err(|e| format!("Invalid UTF-8 from {cmd}: {e}"))
}

fn fetch_measurements(days: u32) -> Result<Vec<Measurement>, String> {
    let json = run_command(
        "bodylog",
        &[
            "measurement",
            "list",
            "--days",
            &days.to_string(),
            "--json",
        ],
    )?;
    serde_json::from_str(&json).map_err(|e| format!("Failed to parse measurements: {e}"))
}

fn fetch_sleep(days: u32) -> Result<Vec<SleepEntry>, String> {
    let json = run_command(
        "bodylog",
        &["sleep", "list", "--days", &days.to_string(), "--json"],
    )?;
    serde_json::from_str(&json).map_err(|e| format!("Failed to parse sleep data: {e}"))
}

fn fetch_nutrition(days: u32) -> Result<Vec<NutritionDay>, String> {
    let json = run_command(
        "nutlog",
        &[
            "report",
            "nutrition",
            "list",
            "--days",
            &days.to_string(),
            "--json",
        ],
    )?;
    let report: NutritionReport =
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse nutrition: {e}"))?;
    Ok(report.days)
}

fn build_body_metrics(measurements: &[Measurement], origin: NaiveDate) -> Vec<MetricSeries> {
    let mut sorted: Vec<&Measurement> = measurements.iter().collect();
    sorted.sort_by_key(|m| m.date);

    vec![
        make_series("Weight", "kg", &sorted, origin, |m| m.weight_kg),
        make_series("Body Fat", "%", &sorted, origin, |m| m.body_fat_pct),
        make_series(
            "Muscle",
            "%",
            &sorted,
            origin,
            |m| m.skeletal_muscle_pct,
        ),
        make_series("Fat Mass", "kg", &sorted, origin, |m| m.fat_mass_kg()),
        make_series("Muscle Mass", "kg", &sorted, origin, |m| m.muscle_mass_kg()),
        make_series(
            "Resting Metabolism",
            "kcal",
            &sorted,
            origin,
            |m| m.resting_metabolism_kcal as f64,
        ),
    ]
}

fn make_series(
    name: &str,
    unit: &str,
    measurements: &[&Measurement],
    origin: NaiveDate,
    value_fn: impl Fn(&Measurement) -> f64,
) -> MetricSeries {
    let points: Vec<SeriesPoint> = measurements
        .iter()
        .map(|m| {
            let days = (m.date - origin).num_days() as f64;
            SeriesPoint {
                date: m.date.format("%Y-%m-%d").to_string(),
                x: days,
                y: value_fn(m),
            }
        })
        .collect();

    let regression_points: Vec<DataPoint> = points
        .iter()
        .map(|p| DataPoint {
            x: p.x,
            y: p.y,
            label: p.date.clone(),
        })
        .collect();

    MetricSeries {
        name: name.to_string(),
        unit: unit.to_string(),
        points,
        regression: linear_regression(&regression_points),
    }
}

pub fn muscle_mass_on_date(measurements: &[Measurement], date: NaiveDate) -> Option<f64> {
    measurements
        .iter()
        .find(|m| m.date == date)
        .map(|m| m.muscle_mass_kg())
        .or_else(|| {
            measurements
                .iter()
                .min_by_key(|m| (m.date - date).num_days().unsigned_abs())
                .map(|m| m.muscle_mass_kg())
        })
}