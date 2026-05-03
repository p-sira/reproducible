//! Fluent report composition API.

use crate::columns::Column;
use crate::metrics::{ErrorMetric, MetricValue, rel_err};
use crate::render::{RenderConfig, TableStyle, render_dynamic_markdown};
use crate::rows::{Row, TestCase};
use delegate::delegate;
use getset::{Getters, Setters, WithSetters};
use std::path::PathBuf;

/// Fluent builder for composing and exporting reports.
#[derive(Getters, Setters, WithSetters)]
#[getset(get = "pub", set = "pub", set_with = "pub")]
pub struct Report<T = f64> {
    #[getset(skip)]
    default_metric: Box<ErrorMetric<T>>,
    render_config: RenderConfig,
    criterion_root: PathBuf,
    columns: Vec<Column<T>>,
    rows: Vec<Row<T>>,
    test_cases: Vec<TestCase<T>>,
}

impl Default for Report<f64> {
    fn default() -> Self {
        Self::new()
    }
}

impl Report<f64> {
    pub fn new() -> Self {
        Self {
            default_metric: Box::new(rel_err),
            render_config: RenderConfig::default(),
            criterion_root: PathBuf::from("target/criterion"),
            columns: Vec::new(),
            rows: Vec::new(),
            test_cases: Vec::new(),
        }
    }
}

impl<T> Report<T> {
    delegate! {
        to self.render_config {
            pub fn set_float_precision(&mut self, precision: usize);
            pub fn set_error_unit(&mut self, unit: String);
            pub fn set_table_style(&mut self, style: TableStyle);
        }
    }

    pub fn with_column(mut self, column: impl Into<Column<T>>) -> Self {
        self.columns.push(column.into());
        self
    }

    pub fn with_row(mut self, row: Row<T>) -> Self {
        self.rows.push(row);
        self
    }

    pub fn with_float_precision(mut self, precision: usize) -> Self {
        self.set_float_precision(precision);
        self
    }

    pub fn with_error_unit(mut self, unit: impl Into<String>) -> Self {
        self.set_error_unit(unit.into());
        self
    }

    pub fn with_style(mut self, style: TableStyle) -> Self {
        self.set_table_style(style);
        self
    }

    #[inline]
    pub fn with_metric<F>(mut self, metric: F) -> Self
    where
        F: Fn(&[T], &[T]) -> MetricValue + Send + Sync + 'static,
    {
        self.default_metric = Box::new(metric);
        self
    }

    pub fn default_metric_ref(&self) -> &ErrorMetric<T> {
        &*self.default_metric
    }

    /// Render the report using the current dynamic column configuration.
    pub fn render_markdown(&self) -> String {
        render_dynamic_markdown(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{columns::ColumnStat, rows::Row};

    #[test]
    fn test_basic_report() {
        let add = |i: &[f64]| vec![i[0] + i[1]];
        let report = Report::new()
            .with_columns(vec![Column::accuracy("Mean Accuracy").into()])
            .with_rows(vec![Row::new("add", add)])
            .with_test_cases(vec![TestCase {
                inputs: vec![1.0, 2.0],
                expected: vec![3.0],
            }]);

        let md = report.render_markdown();
        assert!(md.contains("add"));
        assert!(md.contains("Mean Accuracy"));
        assert!(md.contains("0.00")); // Zero error
    }

    #[test]
    fn test_metric_inheritance_and_stats() {
        let add = |i: &[f64]| vec![i[0] + i[1] + 0.1]; // Add some error
        let report = Report::new()
            .with_metric(|a, e| MetricValue::Numerical((a[0] - e[0]).abs())) // Absolute error
            .with_column(Column::<f64>::accuracy("Mean").with_stat(ColumnStat::Mean))
            .with_column(Column::accuracy("Max").with_stat(ColumnStat::Max))
            .with_rows(vec![Row::new("add", add)])
            .with_test_cases(vec![
                TestCase {
                    inputs: vec![1.0, 1.0],
                    expected: vec![2.0],
                },
                TestCase {
                    inputs: vec![2.0, 2.0],
                    expected: vec![4.0],
                },
            ]);

        let md = report.render_markdown();
        assert!(md.contains("0.10"));
    }

    #[test]
    fn test_row_macro() {
        let my_func = |i: &[f64]| vec![i[0]];
        let report = Report::new().with_row(Row::new("my_func", my_func));

        assert_eq!(report.rows()[0].name, "my_func");
    }

    #[test]
    fn test_table_style() {
        let report = Report::new().with_style(TableStyle::Sharp);

        assert_eq!(report.render_config().table_style, TableStyle::Sharp);
    }
}
