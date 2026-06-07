//! Error metrics.

use serde::{Deserialize, Serialize};

/// The result of an error metric evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Numerical(f64),
    Categorical(String),
}

/// Error metric used for accuracy evaluation.
///
/// Takes `(actual, expected)` slices and returns a [`MetricValue`].
pub type ErrorMetric<T = f64> = dyn Fn(&[T], &[T]) -> MetricValue + Send + Sync;

/// Computes relative error between `actual` and `expected`.
///
/// For multi-dimensional data, this computes the L2 norm of the difference
/// normalized by the L2 norm of the expected value.
///
/// # Example
///
/// ```
/// use reproducible::metrics::{rel_err, MetricValue};
/// let val = rel_err(&[1.0 + 2.0 * f64::EPSILON], &[1.0]);
/// if let MetricValue::Numerical(err) = val {
///     assert!(err > 0.0);
/// }
/// ```
pub fn rel_err(actual: &[f64], expected: &[f64]) -> MetricValue {
    if actual.is_empty() || expected.is_empty() || actual.len() != expected.len() {
        return MetricValue::Numerical(f64::NAN);
    }

    if actual.len() == 1 {
        let a = actual[0];
        let e = expected[0];
        let abs_err = (a - e).abs();
        if abs_err < f64::EPSILON {
            return MetricValue::Numerical(0.0);
        }
        let denom = a.abs().max(e.abs());
        let err = if denom == 0.0 { 0.0 } else { abs_err / denom };
        return MetricValue::Numerical(err);
    }

    // L2 norm based relative error
    let mut sq_sum = 0.0;
    let mut expected_sq_sum = 0.0;
    for (a, e) in actual.iter().zip(expected.iter()) {
        let diff = a - e;
        sq_sum += diff * diff;
        expected_sq_sum += e * e;
    }

    let diff_norm = sq_sum.sqrt();
    let expected_norm = expected_sq_sum.sqrt();

    if diff_norm < f64::EPSILON {
        return MetricValue::Numerical(0.0);
    }

    let err = if expected_norm < 1e-12 {
        diff_norm
    } else {
        diff_norm / expected_norm
    };
    MetricValue::Numerical(err)
}

/// Computes epsilon-scaled error between `actual` and `expected`.
///
/// For multi-dimensional data, this computes the L2 norm of the difference
/// normalized by the L2 norm of the expected value. The result is scaled
/// by `f64::EPSILON`.
///
/// # Example
///
/// ```
/// use reproducible::metrics::{rel_err_eps, MetricValue};
/// let val = rel_err_eps(&[1.0000000000000002], &[1.0]);
/// if let MetricValue::Numerical(err) = val {
///     assert!(err > 0.0 && err < 2.1);
/// }
/// ```
pub fn rel_err_eps(actual: &[f64], expected: &[f64]) -> MetricValue {
    let err = rel_err(actual, expected);
    if let MetricValue::Numerical(err) = err {
        MetricValue::Numerical(err / f64::EPSILON)
    } else {
        err
    }
}

/// Computes absolute error between `actual` and `expected`.
pub fn abs_err(actual: &[f64], expected: &[f64]) -> MetricValue {
    if actual.is_empty() || expected.is_empty() || actual.len() != expected.len() {
        return MetricValue::Numerical(f64::NAN);
    }

    if actual.len() == 1 {
        let a = actual[0];
        let e = expected[0];
        let abs_err = (a - e).abs();
        if abs_err < f64::EPSILON {
            return MetricValue::Numerical(0.0);
        }
        return MetricValue::Numerical(abs_err);
    }

    // L2 norm based absolute error
    let mut sq_sum = 0.0;
    for (a, e) in actual.iter().zip(expected.iter()) {
        let diff = a - e;
        sq_sum += diff * diff;
    }

    let diff_norm = sq_sum.sqrt();
    if diff_norm < f64::EPSILON {
        return MetricValue::Numerical(0.0);
    }
    MetricValue::Numerical(diff_norm)
}
