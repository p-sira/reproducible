//! Benchmark ingestion helpers.
//!
//! This module reads Criterion artifacts and turns them into report rows.

use serde::Deserialize;
use std::path::{Path, PathBuf};

pub(crate) fn default_criterion_root() -> &'static Path {
    Path::new("target/criterion")
}

#[derive(Debug, Deserialize)]
struct CriterionEstimates {
    mean: CriterionPointEstimate,
    median: CriterionPointEstimate,
}

#[derive(Debug, Deserialize)]
struct CriterionPointEstimate {
    point_estimate: f64,
}

#[derive(Debug, Deserialize)]
struct CriterionSample {
    iters: Vec<f64>,
    times: Vec<f64>,
}

/// Parse Criterion's `estimates.json` and return mean point estimate in ns.
pub fn extract_criterion_mean_ns(path: impl AsRef<Path>) -> Result<f64, String> {
    let content = std::fs::read_to_string(path.as_ref())
        .map_err(|e| format!("cannot read {}: {e}", path.as_ref().display()))?;
    let estimates: CriterionEstimates =
        serde_json::from_str(&content).map_err(|e| format!("cannot parse criterion JSON: {e}"))?;
    Ok(estimates.mean.point_estimate)
}

/// Parse Criterion's `sample.json` and return summary stats in ns.
pub fn extract_criterion_stats_ns(path: impl AsRef<Path>) -> Result<crate::stats::Stats, String> {
    let content = std::fs::read_to_string(path.as_ref())
        .map_err(|e| format!("cannot read {}: {e}", path.as_ref().display()))?;
    let sample: CriterionSample =
        serde_json::from_str(&content).map_err(|e| format!("cannot parse criterion JSON: {e}"))?;

    let mut per_iter_times = Vec::with_capacity(sample.iters.len());
    for (time, iters) in sample.times.iter().zip(sample.iters.iter()) {
        if *iters > 0.0 {
            per_iter_times.push(time / iters);
        }
    }

    Ok(crate::stats::Stats::from_samples(&per_iter_times))
}

/// Build an `estimates.json` path from Criterion root.
///
/// Example with default root:
/// `target/criterion/math/elliprf/new/estimates.json`.
///
/// # Example
///
/// ```
/// use reproducible::benchmark::criterion_estimates_path;
/// let path = criterion_estimates_path("target/criterion", "math/ellip").unwrap();
/// let expected = std::path::Path::new("math").join("ellip").join("new").join("estimates.json");
/// assert!(path.ends_with(expected));
/// ```
pub fn criterion_estimates_path(
    criterion_root: impl AsRef<Path>,
    group_function: &str,
) -> Result<PathBuf, String> {
    if group_function.is_empty() {
        return Err("group/function cannot be empty".to_owned());
    }
    Ok(criterion_root
        .as_ref()
        .join(group_function)
        .join("new")
        .join("estimates.json"))
}

/// Build a `sample.json` path from Criterion root and `group/function`.
pub fn criterion_sample_path(
    criterion_root: impl AsRef<Path>,
    group_function: &str,
) -> Result<PathBuf, String> {
    if group_function.is_empty() {
        return Err("group/function cannot be empty".to_owned());
    }
    Ok(criterion_root
        .as_ref()
        .join(group_function)
        .join("new")
        .join("sample.json"))
}

/// Extract mean ns from a `group/function` path in Criterion output.
pub fn extract_criterion_mean_ns_with_id(
    criterion_root: impl AsRef<Path>,
    criterion_id: &str,
) -> Result<f64, String> {
    let path = criterion_estimates_path(criterion_root, criterion_id)?;
    extract_criterion_mean_ns(path)
}

/// Extract a specific statistic from the samples in Criterion output.
pub fn extract_criterion_stat_ns_with_id(
    criterion_root: impl AsRef<Path>,
    criterion_id: &str,
    stat: crate::columns::ColumnStat,
) -> Result<f64, String> {
    match stat {
        crate::columns::ColumnStat::Mean | crate::columns::ColumnStat::Median => {
            let path = criterion_estimates_path(&criterion_root, criterion_id)?;
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
            let estimates: CriterionEstimates = serde_json::from_str(&content)
                .map_err(|e| format!("cannot parse criterion JSON: {e}"))?;
            
            Ok(match stat {
                crate::columns::ColumnStat::Mean => estimates.mean.point_estimate,
                crate::columns::ColumnStat::Median => estimates.median.point_estimate,
                _ => unreachable!(),
            })
        }
        _ => {
            let path = criterion_sample_path(criterion_root, criterion_id)?;
            let stats = extract_criterion_stats_ns(path)?;
            if let crate::stats::Stats::Numerical(ns) = stats {
                Ok(match stat {
                    crate::columns::ColumnStat::Min => ns.min,
                    crate::columns::ColumnStat::Max => ns.max,
                    crate::columns::ColumnStat::P99 => ns.p99,
                    crate::columns::ColumnStat::P95 => ns.p95,
                    crate::columns::ColumnStat::Variance => ns.variance,
                    _ => unreachable!(),
                })
            } else {
                Err("Expected numerical stats".to_owned())
            }
        }
    }
}
