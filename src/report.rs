use serde::Serialize;
use std::fmt::Write as _;

use crate::data::{DashboardData, MetricSeries, muscle_mass_on_date};
use crate::stats::{median, trend_label};

#[derive(Serialize)]
struct ChartPayload {
    labels: Vec<String>,
    datasets: Vec<ChartDataset>,
}

#[derive(Serialize)]
struct ChartDataset {
    label: String,
    data: Vec<Option<f64>>,
    #[serde(rename = "borderColor")]
    border_color: String,
    #[serde(rename = "backgroundColor")]
    background_color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    fill: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tension: Option<f64>,
    #[serde(rename = "pointRadius", skip_serializing_if = "Option::is_none")]
    point_radius: Option<u32>,
    #[serde(rename = "borderDash", skip_serializing_if = "Option::is_none")]
    border_dash: Option<Vec<u32>>,
}

pub fn generate_html(data: &DashboardData) -> String {
    let mut html = String::new();

    let body_charts: String = data
        .body_metrics
        .iter()
        .enumerate()
        .map(|(i, m)| body_metric_section(i, m))
        .collect();

    let sleep_chart = sleep_chart_json(data);
    let calories_chart = nutrition_calories_chart(data);
    let protein_chart = nutrition_ratio_chart(data, "protein");
    let fat_chart = nutrition_ratio_chart(data, "fat");
    let fiber_chart = nutrition_simple_chart(data, "fiber_g", "Fiber (g)");
    let sugars_chart = nutrition_simple_chart(data, "sugars_g", "Sugars (g)");

    let overview_cards = overview_cards(data);
    let sleep_table = sleep_history_table(data);
    let generated = chrono::Local::now().format("%Y-%m-%d %H:%M");

    write!(
        html,
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Body Dashboard</title>
<script src="https://cdn.jsdelivr.net/npm/chart.js@4"></script>
<style>
:root {{
  --bg: #0f1419;
  --surface: #1a2332;
  --surface2: #243044;
  --text: #e8edf4;
  --muted: #8b9cb3;
  --accent: #4fc3f7;
  --accent2: #81c784;
  --warn: #ffb74d;
  --danger: #ef5350;
  --radius: 12px;
  --nav-h: 56px;
}}
* {{ box-sizing: border-box; margin: 0; padding: 0; }}
body {{
  font-family: system-ui, -apple-system, sans-serif;
  background: var(--bg);
  color: var(--text);
  line-height: 1.5;
  padding-bottom: calc(var(--nav-h) + 16px);
}}
header {{
  position: sticky;
  top: 0;
  z-index: 100;
  background: var(--surface);
  border-bottom: 1px solid var(--surface2);
  padding: 12px 16px;
}}
header h1 {{ font-size: 1.1rem; font-weight: 600; }}
header p {{ font-size: 0.75rem; color: var(--muted); margin-top: 2px; }}
main {{ padding: 16px; max-width: 720px; margin: 0 auto; }}
section {{ display: none; }}
section.active {{ display: block; }}
section h2 {{
  font-size: 1.25rem;
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 2px solid var(--accent);
}}
section h3 {{
  font-size: 1rem;
  margin: 20px 0 12px;
  color: var(--accent);
}}
.card-grid {{
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 12px;
  margin-bottom: 20px;
}}
.card-grid.overview-metrics {{
  grid-template-columns: repeat(2, 1fr);
}}
.card {{
  background: var(--surface);
  border-radius: var(--radius);
  padding: 14px;
  border: 1px solid var(--surface2);
}}
.card .label {{ font-size: 0.7rem; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; }}
.card .value {{ font-size: 1.4rem; font-weight: 700; margin: 4px 0; }}
.card .trend {{ font-size: 0.75rem; }}
.trend-up {{ color: var(--accent2); }}
.trend-down {{ color: var(--danger); }}
.trend-flat {{ color: var(--muted); }}
.chart-wrap {{
  background: var(--surface);
  border-radius: var(--radius);
  padding: 12px;
  margin-bottom: 16px;
  border: 1px solid var(--surface2);
}}
.chart-wrap canvas {{ max-height: 260px; }}
.metric-stats {{
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  margin-bottom: 12px;
  font-size: 0.8rem;
}}
.metric-stats div {{
  background: var(--surface2);
  border-radius: 8px;
  padding: 8px 10px;
}}
.metric-stats span {{ color: var(--muted); display: block; font-size: 0.7rem; }}
table {{
  width: 100%;
  border-collapse: collapse;
  font-size: 0.8rem;
  margin-bottom: 16px;
}}
th, td {{
  text-align: left;
  padding: 8px 6px;
  border-bottom: 1px solid var(--surface2);
}}
th {{ color: var(--muted); font-weight: 500; font-size: 0.7rem; text-transform: uppercase; }}
nav {{
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  height: var(--nav-h);
  background: var(--surface);
  border-top: 1px solid var(--surface2);
  display: flex;
  justify-content: space-around;
  align-items: center;
  z-index: 100;
}}
nav a {{
  color: var(--muted);
  text-decoration: none;
  font-size: 0.7rem;
  text-align: center;
  padding: 6px 8px;
  border-radius: 8px;
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}}
nav a .icon {{ font-size: 1.2rem; }}
nav a.active {{ color: var(--accent); background: var(--surface2); }}
.subsection {{ margin-bottom: 28px; }}
@media (min-width: 600px) {{
  main {{ padding: 24px; }}
  .card-grid:not(.overview-metrics) {{ grid-template-columns: repeat(3, 1fr); }}
}}
</style>
</head>
<body>
<header>
  <h1>Body Dashboard</h1>
  <p>{since} → {until} · {days} days · Generated {generated}</p>
</header>
<main>
  <section id="overview" class="active">
    <h2>Overview</h2>
    {overview_cards}
    <h3>Sleep Summary</h3>
    {sleep_summary}
    <h3>Nutrition Summary</h3>
    {nutrition_summary}
  </section>

  <section id="body">
    <h2>Body</h2>
    {body_charts}
  </section>

  <section id="sleep">
    <h2>Sleep</h2>
    <div class="chart-wrap"><canvas id="sleepChart"></canvas></div>
    {sleep_table}
  </section>

  <section id="nutrition">
    <h2>Nutrition</h2>
    <div class="subsection">
      <h3>Calories</h3>
      <div class="chart-wrap"><canvas id="caloriesChart"></canvas></div>
    </div>
    <div class="subsection">
      <h3>Protein per Muscle Mass</h3>
      <div class="chart-wrap"><canvas id="proteinChart"></canvas></div>
    </div>
    <div class="subsection">
      <h3>Fat per Muscle Mass</h3>
      <div class="chart-wrap"><canvas id="fatChart"></canvas></div>
    </div>
    <div class="subsection">
      <h3>Fiber</h3>
      <div class="chart-wrap"><canvas id="fiberChart"></canvas></div>
    </div>
    <div class="subsection">
      <h3>Sugars</h3>
      <div class="chart-wrap"><canvas id="sugarsChart"></canvas></div>
    </div>
  </section>
</main>

<nav>
  <a href="#overview" data-section="overview" class="active"><span class="icon">📊</span>Overview</a>
  <a href="#body" data-section="body"><span class="icon">⚖️</span>Body</a>
  <a href="#sleep" data-section="sleep"><span class="icon">😴</span>Sleep</a>
  <a href="#nutrition" data-section="nutrition"><span class="icon">🍽️</span>Nutrition</a>
</nav>

<script>
const chartDefaults = {{
  responsive: true,
  maintainAspectRatio: true,
  plugins: {{ legend: {{ labels: {{ color: '#8b9cb3', font: {{ size: 11 }} }} }} }},
  scales: {{
    x: {{ ticks: {{ color: '#8b9cb3', maxRotation: 45 }}, grid: {{ color: '#243044' }} }},
    y: {{ ticks: {{ color: '#8b9cb3' }}, grid: {{ color: '#243044' }} }}
  }}
}};

function makeRegressionChart(canvasId, payload) {{
  const ctx = document.getElementById(canvasId);
  if (!ctx) return;
  new Chart(ctx, {{
    type: 'line',
    data: payload,
    options: {{
      ...chartDefaults,
      plugins: {{
        ...chartDefaults.plugins,
        tooltip: {{ mode: 'index', intersect: false }}
      }},
      interaction: {{ mode: 'nearest', axis: 'x', intersect: false }}
    }}
  }});
}}

function makeBarChart(canvasId, payload) {{
  const ctx = document.getElementById(canvasId);
  if (!ctx) return;
  new Chart(ctx, {{
    type: 'bar',
    data: payload,
    options: chartDefaults
  }});
}}

{body_chart_inits}

const sleepData = {sleep_chart};
makeBarChart('sleepChart', sleepData);

const caloriesData = {calories_chart};
makeBarChart('caloriesChart', caloriesData);

const proteinData = {protein_chart};
makeBarChart('proteinChart', proteinData);

const fatData = {fat_chart};
makeBarChart('fatChart', fatData);

const fiberData = {fiber_chart};
makeBarChart('fiberChart', fiberData);

const sugarsData = {sugars_chart};
makeBarChart('sugarsChart', sugarsData);

function showSection(id) {{
  document.querySelectorAll('section').forEach(s => s.classList.remove('active'));
  document.querySelectorAll('nav a').forEach(a => a.classList.remove('active'));
  const section = document.getElementById(id);
  if (section) section.classList.add('active');
  const link = document.querySelector(`nav a[data-section="${{id}}"]`);
  if (link) link.classList.add('active');
}}

function handleNav() {{
  const hash = location.hash.slice(1) || 'overview';
  showSection(hash);
}}

window.addEventListener('hashchange', handleNav);
handleNav();
</script>
</body>
</html>"##,
        since = data.period_since,
        until = data.period_until,
        days = data.days,
        generated = generated,
        overview_cards = overview_cards,
        sleep_summary = sleep_summary_html(data),
        nutrition_summary = nutrition_summary_html(data),
        body_charts = body_charts,
        sleep_table = sleep_table,
        body_chart_inits = body_chart_inits(data),
        sleep_chart = sleep_chart,
        calories_chart = calories_chart,
        protein_chart = protein_chart,
        fat_chart = fat_chart,
        fiber_chart = fiber_chart,
        sugars_chart = sugars_chart,
    )
    .unwrap();

    html
}

fn overview_cards(data: &DashboardData) -> String {
    let mut cards = String::new();

    if data.measurements.is_empty() {
        return r#"<div class="card-grid overview-metrics"></div>"#.to_string();
    }

    for metric in &data.body_metrics {
        if let Some(last) = metric.points.last() {
            let trend = metric
                .regression
                .as_ref()
                .map(|r| {
                    let cls = if r.slope_per_day.abs() < 0.001 {
                        "trend-flat"
                    } else if r.slope_per_day > 0.0 {
                        "trend-up"
                    } else {
                        "trend-down"
                    };
                    let label = trend_label(r.slope_per_day, &metric.unit);
                    format!(r#"<div class="trend {cls}">{label}</div>"#)
                })
                .unwrap_or_else(|| r#"<div class="trend trend-flat">—</div>"#.to_string());

            write!(
                cards,
                r#"<div class="card"><div class="label">{}</div><div class="value">{:.1} {}</div>{}</div>"#,
                metric.name, last.y, metric.unit, trend
            )
            .unwrap();
        }
    }

    format!(r#"<div class="card-grid overview-metrics">{cards}</div>"#)
}

fn sleep_summary_html(data: &DashboardData) -> String {
    if data.sleep.is_empty() {
        return r#"<p style="color:var(--muted);font-size:0.85rem">No sleep data</p>"#.to_string();
    }

    let sleep_hours: Vec<f64> = data
        .sleep
        .iter()
        .map(|s| s.total_sleep_minutes() as f64 / 60.0)
        .collect();
    let deep_minutes: Vec<f64> = data
        .sleep
        .iter()
        .map(|s| s.deep_minutes as f64)
        .collect();
    let rem_minutes: Vec<f64> = data.sleep.iter().map(|s| s.rem_minutes as f64).collect();

    let med_sleep_h = median(&sleep_hours).unwrap_or(0.0);
    let med_deep = median(&deep_minutes).unwrap_or(0.0);
    let med_rem = median(&rem_minutes).unwrap_or(0.0);

    format!(
        r#"<div class="card-grid">
  <div class="card"><div class="label">Median Sleep</div><div class="value">{:.1}h</div></div>
  <div class="card"><div class="label">Nights</div><div class="value">{}</div></div>
  <div class="card"><div class="label">Median Deep</div><div class="value">{:.0}m</div></div>
  <div class="card"><div class="label">Median REM</div><div class="value">{:.0}m</div></div>
</div>"#,
        med_sleep_h,
        data.sleep.len(),
        med_deep,
        med_rem,
    )
}

fn nutrition_summary_html(data: &DashboardData) -> String {
    if data.nutrition.is_empty() {
        return r#"<p style="color:var(--muted);font-size:0.85rem">No nutrition data</p>"#.to_string();
    }

    let kcal: Vec<f64> = data
        .nutrition
        .iter()
        .map(|d| d.totals.energy_kcal())
        .collect();
    let protein: Vec<f64> = data
        .nutrition
        .iter()
        .map(|d| d.totals.protein_g())
        .collect();
    let fiber: Vec<f64> = data.nutrition.iter().map(|d| d.totals.fiber_g()).collect();
    let sugars: Vec<f64> = data
        .nutrition
        .iter()
        .map(|d| d.totals.sugars_g())
        .collect();

    format!(
        r#"<div class="card-grid">
  <div class="card"><div class="label">Median Calories</div><div class="value">{:.0}</div></div>
  <div class="card"><div class="label">Median Protein</div><div class="value">{:.0}g</div></div>
  <div class="card"><div class="label">Median Fiber</div><div class="value">{:.1}g</div></div>
  <div class="card"><div class="label">Median Sugars</div><div class="value">{:.0}g</div></div>
</div>"#,
        median(&kcal).unwrap_or(0.0),
        median(&protein).unwrap_or(0.0),
        median(&fiber).unwrap_or(0.0),
        median(&sugars).unwrap_or(0.0),
    )
}

fn body_metric_section(index: usize, metric: &MetricSeries) -> String {
    let stats = if let Some(reg) = &metric.regression {
        format!(
            r#"<div class="metric-stats">
  <div><span>Rate / day</span>{:+.3} {}</div>
  <div><span>Rate / week</span>{:+.3} {}</div>
  <div><span>R²</span>{:.3}</div>
  <div><span>Latest</span>{:.2} {}</div>
</div>"#,
            reg.slope_per_day,
            metric.unit,
            reg.slope_per_week,
            metric.unit,
            reg.r_squared,
            metric.points.last().map(|p| p.y).unwrap_or(0.0),
            metric.unit,
        )
    } else {
        r#"<div class="metric-stats"><div><span>Trend</span>Insufficient data</div></div>"#
            .to_string()
    };

    format!(
        r#"<div class="subsection">
  <h3>{name}</h3>
  {stats}
  <div class="chart-wrap"><canvas id="bodyChart{index}"></canvas></div>
</div>"#,
        name = metric.name,
        stats = stats,
        index = index,
    )
}

fn body_chart_inits(data: &DashboardData) -> String {
    data.body_metrics
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let payload = regression_chart_json(m);
            format!("makeRegressionChart('bodyChart{i}', {payload});")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn regression_chart_json(metric: &MetricSeries) -> String {
    let labels: Vec<String> = metric.points.iter().map(|p| p.date.clone()).collect();

    let mut datasets = vec![ChartDataset {
        label: "Observed".to_string(),
        data: metric.points.iter().map(|p| Some(p.y)).collect(),
        border_color: "#4fc3f7".to_string(),
        background_color: "rgba(79,195,247,0.2)".to_string(),
        fill: None,
        tension: None,
        point_radius: Some(5),
        border_dash: None,
    }];

    if let Some(reg) = &metric.regression {
        let regression_inputs: Vec<crate::stats::DataPoint> = metric
            .points
            .iter()
            .map(|p| crate::stats::DataPoint {
                x: p.x,
                y: p.y,
                label: p.date.clone(),
            })
            .collect();

        let ci_at_points: Vec<(f64, f64, f64)> = metric
            .points
            .iter()
            .map(|p| {
                crate::stats::confidence_at(
                    &regression_inputs,
                    reg.slope,
                    reg.intercept,
                    p.x,
                    reg.ss_res,
                )
            })
            .collect();

        datasets.push(ChartDataset {
            label: "95% CI Upper".to_string(),
            data: ci_at_points.iter().map(|(_, _, u)| Some(*u)).collect(),
            border_color: "rgba(129,199,132,0.3)".to_string(),
            background_color: "rgba(129,199,132,0.15)".to_string(),
            fill: Some(serde_json::json!("+1")),
            tension: Some(0.1),
            point_radius: Some(0),
            border_dash: Some(vec![4, 4]),
        });

        datasets.push(ChartDataset {
            label: "95% CI Lower".to_string(),
            data: ci_at_points.iter().map(|(_, l, _)| Some(*l)).collect(),
            border_color: "rgba(129,199,132,0.3)".to_string(),
            background_color: "rgba(129,199,132,0.15)".to_string(),
            fill: Some(serde_json::json!(false)),
            tension: Some(0.1),
            point_radius: Some(0),
            border_dash: Some(vec![4, 4]),
        });

        datasets.push(ChartDataset {
            label: "Regression".to_string(),
            data: ci_at_points.iter().map(|(y, _, _)| Some(*y)).collect(),
            border_color: "#81c784".to_string(),
            background_color: "rgba(129,199,132,0)".to_string(),
            fill: None,
            tension: Some(0.1),
            point_radius: Some(0),
            border_dash: None,
        });
    }

    let payload = ChartPayload { labels, datasets };

    serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
}

fn sleep_chart_json(data: &DashboardData) -> String {
    let mut sorted = data.sleep.clone();
    sorted.sort_by_key(|s| s.date);

    let labels: Vec<String> = sorted
        .iter()
        .map(|s| s.date.format("%m/%d").to_string())
        .collect();

    let payload = ChartPayload {
        labels,
        datasets: vec![
            ChartDataset {
                label: "Total (h)".to_string(),
                data: sorted
                    .iter()
                    .map(|s| Some(s.total_sleep_minutes() as f64 / 60.0))
                    .collect(),
                border_color: "#4fc3f7".to_string(),
                background_color: "rgba(79,195,247,0.6)".to_string(),
                fill: None,
                tension: None,
                point_radius: None,
                border_dash: None,
            },
            ChartDataset {
                label: "Deep (h)".to_string(),
                data: sorted
                    .iter()
                    .map(|s| Some(s.deep_minutes as f64 / 60.0))
                    .collect(),
                border_color: "#81c784".to_string(),
                background_color: "rgba(129,199,132,0.6)".to_string(),
                fill: None,
                tension: None,
                point_radius: None,
                border_dash: None,
            },
            ChartDataset {
                label: "REM (h)".to_string(),
                data: sorted
                    .iter()
                    .map(|s| Some(s.rem_minutes as f64 / 60.0))
                    .collect(),
                border_color: "#ffb74d".to_string(),
                background_color: "rgba(255,183,77,0.6)".to_string(),
                fill: None,
                tension: None,
                point_radius: None,
                border_dash: None,
            },
        ],
    };

    serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
}

fn sleep_history_table(data: &DashboardData) -> String {
    let mut sorted = data.sleep.clone();
    sorted.sort_by_key(|s| s.date);
    sorted.reverse();

    let mut rows = String::new();
    for s in &sorted {
        let hours = s.total_sleep_minutes() as f64 / 60.0;
        let notes = s.notes.as_deref().unwrap_or("—");
        write!(
            rows,
            "<tr><td>{}</td><td>{:.1}h</td><td>{}m</td><td>{}m</td><td>{}</td></tr>",
            s.date, hours, s.deep_minutes, s.rem_minutes, notes
        )
        .unwrap();
    }

    format!(
        r#"<table>
<thead><tr><th>Date</th><th>Total</th><th>Deep</th><th>REM</th><th>Notes</th></tr></thead>
<tbody>{rows}</tbody>
</table>"#
    )
}

fn nutrition_calories_chart(data: &DashboardData) -> String {
    let mut sorted = data.nutrition.clone();
    sorted.sort_by_key(|d| d.date);

    let labels: Vec<String> = sorted
        .iter()
        .map(|d| d.date.format("%m/%d").to_string())
        .collect();

    let payload = ChartPayload {
        labels,
        datasets: vec![ChartDataset {
            label: "Calories".to_string(),
            data: sorted
                .iter()
                .map(|d| Some(d.totals.energy_kcal()))
                .collect(),
            border_color: "#4fc3f7".to_string(),
            background_color: "rgba(79,195,247,0.6)".to_string(),
            fill: None,
            tension: None,
            point_radius: None,
            border_dash: None,
        }],
    };

    serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
}

fn nutrition_ratio_chart(data: &DashboardData, kind: &str) -> String {
    let mut sorted = data.nutrition.clone();
    sorted.sort_by_key(|d| d.date);

    let (label, getter): (&str, fn(&crate::data::NutritionTotals) -> f64) = match kind {
        "protein" => ("Protein / Muscle Mass (g/kg)", |t| t.protein_g()),
        "fat" => ("Fat / Muscle Mass (g/kg)", |t| t.fat_g()),
        _ => ("Ratio", |_| 0.0),
    };

    let labels: Vec<String> = sorted
        .iter()
        .map(|d| d.date.format("%m/%d").to_string())
        .collect();

    let values: Vec<Option<f64>> = sorted
        .iter()
        .map(|d| {
            muscle_mass_on_date(&data.measurements, d.date).map(|mm| {
                if mm > 0.0 {
                    getter(&d.totals) / mm
                } else {
                    0.0
                }
            })
        })
        .collect();

    let payload = ChartPayload {
        labels,
        datasets: vec![ChartDataset {
            label: label.to_string(),
            data: values,
            border_color: if kind == "protein" {
                "#81c784".to_string()
            } else {
                "#ffb74d".to_string()
            },
            background_color: if kind == "protein" {
                "rgba(129,199,132,0.6)".to_string()
            } else {
                "rgba(255,183,77,0.6)".to_string()
            },
            fill: None,
            tension: None,
            point_radius: None,
            border_dash: None,
        }],
    };

    serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
}

fn nutrition_simple_chart(data: &DashboardData, field: &str, label: &str) -> String {
    let mut sorted = data.nutrition.clone();
    sorted.sort_by_key(|d| d.date);

    let labels: Vec<String> = sorted
        .iter()
        .map(|d| d.date.format("%m/%d").to_string())
        .collect();

    let values: Vec<Option<f64>> = sorted
        .iter()
        .map(|d| {
            Some(match field {
                "fiber_g" => d.totals.fiber_g(),
                "sugars_g" => d.totals.sugars_g(),
                _ => 0.0,
            })
        })
        .collect();

    let payload = ChartPayload {
        labels,
        datasets: vec![ChartDataset {
            label: label.to_string(),
            data: values,
            border_color: "#ce93d8".to_string(),
            background_color: "rgba(206,147,216,0.6)".to_string(),
            fill: None,
            tension: None,
            point_radius: None,
            border_dash: None,
        }],
    };

    serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
}