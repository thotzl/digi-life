//! Small descriptive statistics shared by the trainer stats and the diagnosis probes (TCK-201).

use serde::Serialize;

/// Linear-interpolated quantile of an ascending, non-empty slice: position `q · (n − 1)`.
pub fn quantile(sorted: &[f32], q: f32) -> f32 {
    let pos = q * (sorted.len() - 1) as f32;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo as f32)
}

/// Sorts ascending; NaN compares equal (keeps its place).
pub fn sort_ascending(values: &mut [f32]) {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
}

/// p10/p25/p50/p75/p90 (linear interpolation), mean, min, max.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Distribution {
    pub n: usize,
    pub mean: f32,
    pub min: f32,
    pub p10: f32,
    pub p25: f32,
    pub p50: f32,
    pub p75: f32,
    pub p90: f32,
    pub max: f32,
}

impl Distribution {
    /// `None` for an empty slice. The mean is summed in f64.
    pub fn of(values: &[f32]) -> Option<Self> {
        if values.is_empty() {
            return None;
        }
        let mut sorted = values.to_vec();
        sort_ascending(&mut sorted);
        Some(Self {
            n: sorted.len(),
            mean: sorted.iter().map(|&v| v as f64).sum::<f64>() as f32 / sorted.len() as f32,
            min: sorted[0],
            p10: quantile(&sorted, 0.1),
            p25: quantile(&sorted, 0.25),
            p50: quantile(&sorted, 0.5),
            p75: quantile(&sorted, 0.75),
            p90: quantile(&sorted, 0.9),
            max: sorted[sorted.len() - 1],
        })
    }
}

/// Pearson correlation; `None` for fewer than 2 pairs or zero variance.
pub fn pearson(pairs: &[(f32, f32)]) -> Option<f32> {
    if pairs.len() < 2 {
        return None;
    }
    let n = pairs.len() as f64;
    let (mx, my) = pairs.iter().fold((0.0, 0.0), |(sx, sy), &(x, y)| (sx + x as f64, sy + y as f64));
    let (mx, my) = (mx / n, my / n);
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for &(x, y) in pairs {
        let (dx, dy) = (x as f64 - mx, y as f64 - my);
        sxy += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }
    if sxx <= 0.0 || syy <= 0.0 { None } else { Some((sxy / (sxx * syy).sqrt()) as f32) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantile_interpolates() {
        let sorted = [0.0, 10.0, 20.0, 30.0, 40.0];
        assert_eq!(quantile(&sorted, 0.5), 20.0);
        assert_eq!(quantile(&sorted, 0.1), 4.0);
        assert_eq!(quantile(&[7.0], 0.9), 7.0);
    }

    #[test]
    fn test_pearson_and_distribution() {
        assert_eq!(pearson(&[(1.0, 2.0), (2.0, 4.0), (3.0, 6.0)]), Some(1.0));
        assert_eq!(pearson(&[(1.0, 1.0), (1.0, 2.0)]), None);
        let d = Distribution::of(&[0.0, 10.0, 20.0, 30.0, 40.0]).unwrap();
        assert_eq!((d.p50, d.p10, d.mean), (20.0, 4.0, 20.0));
        assert_eq!(Distribution::of(&[]), None);
    }
}
