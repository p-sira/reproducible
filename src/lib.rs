//! `reproducible` helps Rust projects generate reproducible
//! accuracy and benchmark reports.
//!
//! ## Quick Start
//!
//! ```
//! use reproducible::prelude::*;
//! use reproducible::metrics;
//!
//! // 1. Define the functions we want to evaluate
//!
//! let fn_add = |inputs: &[f64]| vec![inputs[0] + inputs[1] + 5.0 * f64::EPSILON];
//! let fn_sub = |inputs: &[f64]| vec![inputs[0] - inputs[1] - 10.0 * f64::EPSILON];
//!
//! // 2. Compose the report
//!
//! let report = Report::new()
//!
//!     // Add columns for metrics and statistics
//!     .with_column(
//!         Column::<f64>::accuracy("Mean Relative Error (eps)")
//!             .with_metric(metrics::rel_err_eps)
//!     )
//!     .with_column(
//!         Column::<f64>::accuracy("Max Absolute Error")
//!             .with_metric(metrics::abs_err)
//!             .with_stat(ColumnStat::Max)
//!     )
//!     .with_column(Column::<f64>::perf("Latency"))
//!
//!     // Add rows corresponding to each function you want to evaluate
//!     .with_row(Row::new("math/add", fn_add).with_test_cases(vec![
//!         TestCase { inputs: vec![1.0, 2.0], expected: vec![3.0] },
//!     ]))
//!     .with_row(Row::new("math/sub", fn_sub).with_test_cases(vec![
//!         TestCase { inputs: vec![1.0, 2.0], expected: vec![-1.0] },
//!     ]));
//!
//! // 3. Render to Markdown and print the testing environment
//!
//! println!("{}", report.render_markdown());
//! # assert!(report.render_markdown() == "| Function | Mean Relative Error (eps) | Max Absolute Error | Latency |\n|----------|---------------------------|--------------------|---------|\n| math/add | 1.33                      | 8.88e-16           | 9.3 ns  |\n| math/sub | 10.00                     | 2.22e-15           | 9.7 ns  |");
//! println!("Tested on {}", current_env!());
//! ```
//!
//! Output:
//! ```text
//! | Function | Mean Relative Error (eps) | Max Absolute Error | Latency |
//! |----------|---------------------------|--------------------|---------|
//! | math/add | 1.33                      | 8.88e-16           | 9.3 ns  |
//! | math/sub | 10.00                     | 2.22e-15           | 9.7 ns  |
//!
//! Tested on AMD Ryzen 5 4600H with Radeon Graphics @2.4 GHz RAM 16 GB running x86_64-unknown-linux-gnu rustc 1.90.0 using reproducible v0.2.0
//! ```
//!

extern crate self as reproducible;

pub mod benchmark;
pub mod columns;
pub mod env;
pub mod metrics;
pub mod parser;
pub mod render;
pub mod report;
pub mod rows;
pub mod stats;

pub mod prelude {
    pub use crate::columns::{Column, ColumnStat};
    pub use crate::current_env;
    pub use crate::report::Report;
    pub use crate::rows::{Row, TestCase};
    pub use crate::stats::Stats;
}

#[cfg(feature = "macros")]
pub use reproducible_macros::{as_array, row, wrap_args};

#[cfg(feature = "macros")]
#[allow(unused_imports)]
mod macro_docs {
    /// Shorthand for creating a `Row` from a function, using its name as the row label.
    ///
    /// # Example
    ///
    /// ```
    /// use reproducible::row;
    /// let my_func = |i: &[f64]| vec![i[0]];
    /// let r = row!(my_func);
    /// assert_eq!(r.name, "my_func");
    /// ```
    pub use crate::row;

    /// Wrap a function that takes positional arguments into one that takes a slice.
    ///
    /// This is useful when your function has a fixed number of arguments
    /// but the `reproducible` API expects `Fn(&[T]) -> Vec<T>`.
    ///
    /// # Example
    ///
    /// ```
    /// use reproducible::wrap_args;
    /// fn add(a: f64, b: f64) -> f64 { a + b }
    /// let wrapped = wrap_args!(add, 2);
    /// assert_eq!(wrapped(&[1.0, 2.0]), vec![3.0]);
    /// ```
    pub use crate::wrap_args;

    /// Wrap a function that takes a fixed array into one that takes a slice.
    ///
    /// # Example
    ///
    /// ```
    /// use reproducible::as_array;
    /// fn sum_array(arr: [f64; 3]) -> f64 { arr.iter().sum() }
    /// let wrapped = as_array!(sum_array, 3);
    /// assert_eq!(wrapped(&[1.0, 2.0, 3.0]), vec![6.0]);
    /// ```
    pub use crate::as_array;
}

use serde::Serialize;

/// Serialize report rows to pretty JSON for machine-friendly artifacts.
pub fn to_pretty_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_json_is_valid() {
        use serde::Serialize;
        #[derive(Serialize)]
        struct DummyRow {
            name: String,
        }
        let json = to_pretty_json(&vec![DummyRow {
            name: "f".to_owned(),
        }])
        .expect("json");
        assert!(json.contains("\"name\": \"f\""));
    }
}
