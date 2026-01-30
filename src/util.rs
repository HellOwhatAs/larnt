pub fn radians(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}

pub fn degrees(radians: f64) -> f64 {
    radians * 180.0 / std::f64::consts::PI
}

pub fn median(items: &[f64]) -> f64 {
    let n = items.len();
    match n {
        0 => 0.0,
        _ if n % 2 == 1 => items[n / 2],
        _ => {
            let a = items[n / 2 - 1];
            let b = items[n / 2];
            (a + b) / 2.0
        }
    }
}

pub fn parse_floats(items: &[&str]) -> Vec<f64> {
    items.iter()
        .map(|s| s.parse::<f64>().unwrap_or(0.0))
        .collect()
}
