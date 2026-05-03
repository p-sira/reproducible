//! Statistical utilities for report aggregation.

use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};

/// Numerical summary statistics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct NumericalStats {
    pub mean: f64,
    pub median: f64,
    pub variance: f64,
    pub max: f64,
    pub p99: f64,
    pub p95: f64,
    pub n: usize,
}

/// Categorical statistics (frequency distribution).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CategoricalStats {
    pub counts: std::collections::HashMap<String, usize>,
    pub total: usize,
}

/// Summary statistics used by report entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Stats {
    Numerical(NumericalStats),
    Categorical(CategoricalStats),
}

impl Default for Stats {
    fn default() -> Self {
        Stats::Numerical(NumericalStats::default())
    }
}

impl Stats {
    /// Compute summary statistics from metric values.
    ///
    /// If all values are numerical, returns [`Stats::Numerical`].
    /// Otherwise, returns [`Stats::Categorical`].
    pub fn from_metric_values(values: &[crate::metrics::MetricValue]) -> Self {
        if values.is_empty() {
            return Self::Numerical(NumericalStats::default());
        }

        // Check if all are numerical
        let all_numerical = values
            .iter()
            .all(|v| matches!(v, crate::metrics::MetricValue::Numerical(_)));

        if all_numerical {
            let samples: Vec<f64> = values
                .iter()
                .map(|v| {
                    if let crate::metrics::MetricValue::Numerical(n) = v {
                        *n
                    } else {
                        f64::NAN
                    }
                })
                .collect();
            return Self::from_samples(&samples);
        }

        // Categorical
        let mut counts = std::collections::HashMap::new();
        for v in values {
            let label = match v {
                crate::metrics::MetricValue::Numerical(n) => format!("{n}"),
                crate::metrics::MetricValue::Categorical(s) => s.clone(),
            };
            *counts.entry(label).or_insert(0) += 1;
        }

        Self::Categorical(CategoricalStats {
            counts,
            total: values.len(),
        })
    }

    /// Compute numerical summary statistics from finite sample values.
    pub fn from_samples<T: ToPrimitive + Copy>(samples: &[T]) -> Self {
        let mut valids: Vec<f64> = samples
            .iter()
            .copied()
            .filter_map(|x| x.to_f64())
            .filter(|x| x.is_finite())
            .collect();

        if valids.is_empty() {
            return Self::Numerical(NumericalStats::default());
        }

        valids.sort_by(|a, b| a.partial_cmp(b).expect("finite values are comparable"));
        let n = valids.len();
        let mean = valids.iter().sum::<f64>() / n as f64;
        let median = if n % 2 == 0 {
            (valids[n / 2 - 1] + valids[n / 2]) / 2.0
        } else {
            valids[n / 2]
        };

        let p99_pos = (n - 1) as f64 * 0.99;
        let low = p99_pos.floor() as usize;
        let frac = p99_pos - low as f64;
        let p99 = if low + 1 < n {
            valids[low] * (1.0 - frac) + valids[low + 1] * frac
        } else {
            valids[n - 1]
        };

        // Calculate P95 error
        let p95_pos = (n - 1) as f64 * 0.95;
        let low_95 = p95_pos.floor() as usize;
        let frac_95 = p95_pos - low_95 as f64;
        let p95 = if low_95 + 1 < n {
            valids[low_95] * (1.0 - frac_95) + valids[low_95 + 1] * frac_95
        } else {
            valids[n - 1]
        };

        let variance = valids
            .iter()
            .map(|x| {
                let d = mean - *x;
                d * d
            })
            .sum::<f64>()
            / n as f64;

        let max = *valids.last().unwrap_or(&f64::NAN);

        Self::Numerical(NumericalStats {
            mean,
            median,
            variance,
            max,
            p99,
            p95,
            n,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_from_samples_works() {
        let s = Stats::from_samples(&[1.0, 1.5, 2.0, 3.0, 4.0, f64::NAN]);
        if let Stats::Numerical(ns) = s {
            assert_eq!(ns.n, 5);
            assert_eq!(ns.mean, 2.3);
            assert_eq!(ns.median, 2.0);
            assert!((ns.p99 - 3.96).abs() < 1e-10);
            assert!((ns.p95 - 3.8).abs() < 1e-10);
            assert_eq!(ns.max, 4.0);
        } else {
            panic!("Expected Numerical stats");
        }
    }

    #[test]
    fn test_categorical_stats() {
        use crate::metrics::MetricValue;
        let values = vec![
            MetricValue::Categorical("Pass".to_string()),
            MetricValue::Categorical("Pass".to_string()),
            MetricValue::Categorical("Fail".to_string()),
        ];
        let stats = Stats::from_metric_values(&values);
        if let Stats::Categorical(cs) = stats {
            assert_eq!(*cs.counts.get("Pass").unwrap(), 2);
            assert_eq!(*cs.counts.get("Fail").unwrap(), 1);
            assert_eq!(cs.total, 3);
        } else {
            panic!("Expected Categorical stats");
        }
    }
}
