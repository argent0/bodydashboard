#[derive(Debug, Clone)]
pub struct DataPoint {
    pub x: f64,
    pub y: f64,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct RegressionResult {
    pub slope: f64,
    pub intercept: f64,
    pub r_squared: f64,
    pub slope_per_week: f64,
    pub slope_per_day: f64,
    pub ss_res: f64,
    pub ci_points: Vec<CiPoint>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CiPoint {
    pub x: f64,
    pub y: f64,
    pub lower: f64,
    pub upper: f64,
    pub label: String,
}

pub fn linear_regression(points: &[DataPoint]) -> Option<RegressionResult> {
    let n = points.len();
    if n < 2 {
        return None;
    }

    let x_mean = points.iter().map(|p| p.x).sum::<f64>() / n as f64;
    let y_mean = points.iter().map(|p| p.y).sum::<f64>() / n as f64;

    let mut ss_xy = 0.0;
    let mut ss_xx = 0.0;
    let mut ss_yy = 0.0;

    for p in points {
        let dx = p.x - x_mean;
        let dy = p.y - y_mean;
        ss_xy += dx * dy;
        ss_xx += dx * dx;
        ss_yy += dy * dy;
    }

    if ss_xx.abs() < f64::EPSILON {
        return None;
    }

    let slope = ss_xy / ss_xx;
    let intercept = y_mean - slope * x_mean;

    let ss_res: f64 = points
        .iter()
        .map(|p| {
            let predicted = slope * p.x + intercept;
            let residual = p.y - predicted;
            residual * residual
        })
        .sum();

    let r_squared = if ss_yy.abs() < f64::EPSILON {
        1.0
    } else {
        1.0 - ss_res / ss_yy
    };

    let df = (n - 2) as f64;
    let mse = if df > 0.0 { ss_res / df } else { 0.0 };
    let se = mse.sqrt();
    let t_crit = t_critical_95(df);

    let x_min = points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
    let x_max = points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
    let x_range = x_max - x_min;
    let steps = 20.max(n * 2);

    let mut ci_points = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let frac = i as f64 / steps as f64;
        let x = if x_range < f64::EPSILON {
            x_mean
        } else {
            x_min + frac * x_range
        };
        let y = slope * x + intercept;
        let se_fit = if df > 0.0 {
            se * ((1.0 / n as f64) + (x - x_mean).powi(2) / ss_xx).sqrt()
        } else {
            0.0
        };
        let margin = t_crit * se_fit;

        ci_points.push(CiPoint {
            x,
            y,
            lower: y - margin,
            upper: y + margin,
            label: String::new(),
        });
    }

    Some(RegressionResult {
        slope,
        intercept,
        r_squared,
        slope_per_day: slope,
        slope_per_week: slope * 7.0,
        ss_res,
        ci_points,
    })
}

fn t_critical_95(df: f64) -> f64 {
    if df <= 0.0 {
        return 0.0;
    }
    if df >= 120.0 {
        return 1.96;
    }

    let table: &[(f64, f64)] = &[
        (1.0, 12.706),
        (2.0, 4.303),
        (3.0, 3.182),
        (4.0, 2.776),
        (5.0, 2.571),
        (6.0, 2.447),
        (7.0, 2.365),
        (8.0, 2.306),
        (9.0, 2.262),
        (10.0, 2.228),
        (15.0, 2.131),
        (20.0, 2.086),
        (25.0, 2.060),
        (30.0, 2.042),
        (40.0, 2.021),
        (60.0, 2.000),
    ];

    for &(d, t) in table {
        if df <= d {
            return t;
        }
    }
    1.96
}

pub fn confidence_at(
    points: &[DataPoint],
    slope: f64,
    intercept: f64,
    x: f64,
    residual_ss: f64,
) -> (f64, f64, f64) {
    let n = points.len();
    if n < 2 {
        let y = slope * x + intercept;
        return (y, y, y);
    }

    let x_mean = points.iter().map(|p| p.x).sum::<f64>() / n as f64;
    let ss_xx: f64 = points.iter().map(|p| (p.x - x_mean).powi(2)).sum();
    let df = (n - 2) as f64;
    let se = if df > 0.0 {
        (residual_ss / df).sqrt()
    } else {
        0.0
    };
    let t_crit = t_critical_95(df);
    let y = slope * x + intercept;

    if ss_xx.abs() < f64::EPSILON {
        return (y, y, y);
    }

    let se_fit = se * ((1.0 / n as f64) + (x - x_mean).powi(2) / ss_xx).sqrt();
    let margin = t_crit * se_fit;
    (y, y - margin, y + margin)
}

pub fn trend_label(slope_per_day: f64, unit: &str) -> String {
    if slope_per_day.abs() < 0.001 {
        format!("stable ({unit})")
    } else if slope_per_day > 0.0 {
        format!("↑ +{:.3} {unit}/day", slope_per_day)
    } else {
        format!("↓ {:.3} {unit}/day", slope_per_day)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regression_on_linear_data() {
        let points: Vec<DataPoint> = (0..5)
            .map(|i| DataPoint {
                x: i as f64,
                y: 2.0 * i as f64 + 1.0,
                label: String::new(),
            })
            .collect();
        let result = linear_regression(&points).unwrap();
        assert!((result.slope - 2.0).abs() < 0.01);
        assert!((result.intercept - 1.0).abs() < 0.01);
        assert!(result.r_squared > 0.99);
    }
}