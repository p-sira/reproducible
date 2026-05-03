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
    mean: CriterionMean,
}

#[derive(Debug, Deserialize)]
struct CriterionMean {
    point_estimate: f64,
}

/// Parse Criterion's `estimates.json` and return mean point estimate in ns.
pub fn extract_criterion_mean_ns(path: impl AsRef<Path>) -> Result<f64, String> {
    let content = std::fs::read_to_string(path.as_ref())
        .map_err(|e| format!("cannot read {}: {e}", path.as_ref().display()))?;
    let estimates: CriterionEstimates =
        serde_json::from_str(&content).map_err(|e| format!("cannot parse criterion JSON: {e}"))?;
    Ok(estimates.mean.point_estimate)
}

/// Build an `estimates.json` path from Criterion root and `group/function`.
///
/// Example with default root:
/// `target/criterion/math/elliprf/new/estimates.json`.
///
/// # Example
///
/// ```
/// use reproducible::benchmark::criterion_estimates_path;
/// let path = criterion_estimates_path("target/criterion", "math/ellip").unwrap();
/// assert!(path.to_str().unwrap().contains("math/ellip/new/estimates.json"));
/// ```
pub fn criterion_estimates_path(
    criterion_root: impl AsRef<Path>,
    group_function: &str,
) -> Result<PathBuf, String> {
    let mut parts = group_function.split('/');
    let group = parts
        .next()
        .ok_or_else(|| "group/function cannot be empty".to_owned())?;
    let function = parts
        .next()
        .ok_or_else(|| "group/function must contain exactly one '/'".to_owned())?;
    if parts.next().is_some() || group.is_empty() || function.is_empty() {
        return Err("group/function must be in the form '<group>/<function>'".to_owned());
    }
    Ok(criterion_root
        .as_ref()
        .join(group)
        .join(function)
        .join("new")
        .join("estimates.json"))
}

/// Extract mean ns from a `group/function` path in Criterion output.
pub fn extract_criterion_mean_ns_with_id(
    criterion_root: impl AsRef<Path>,
    criterion_id: &str,
) -> Result<f64, String> {
    let path = criterion_estimates_path(criterion_root, criterion_id)?;
    extract_criterion_mean_ns(path)
}
