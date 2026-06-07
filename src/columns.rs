//! Accuracy and performance columns.

use crate::metrics::{ErrorMetric, MetricValue};
use serde::{Deserialize, Serialize};

/// Statistical aggregator to use for a column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnStat {
    Mean,
    Median,
    Max,
    P99,
    P95,
    Variance,
}

/// A column definition for a report.
pub enum Column<T = f64> {
    Accuracy(AccuracyColumn<T>),
    /// A performance column that reads benchmark data from Criterion.
    Performance(PerformanceColumn),
    /// A custom column with a user-defined renderer.
    Custom(
        String,
        Box<dyn Fn(&crate::stats::Stats) -> String + Send + Sync>,
    ),
}

impl<T: std::fmt::Debug> std::fmt::Debug for Column<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Column::Accuracy(c) => f.debug_tuple("Accuracy").field(c).finish(),
            Column::Performance(c) => f.debug_tuple("Performance").field(c).finish(),
            Column::Custom(name, _) => f.debug_tuple("Custom").field(name).finish(),
        }
    }
}

impl<T> Column<T> {
    /// Create a new accuracy column.
    ///
    /// # Example
    ///
    /// ```
    /// # use reproducible::columns::Column;
    /// let col = Column::<f64>::accuracy("Mean L2");
    /// ```
    pub fn accuracy(name: impl Into<String>) -> AccuracyColumn<T> {
        AccuracyColumn {
            name: name.into(),
            metric: None,
            target_stat: ColumnStat::Mean,
        }
    }

    /// Create a new performance column.
    pub fn perf(name: impl Into<String>) -> PerformanceColumn {
        PerformanceColumn {
            name: name.into(),
            target_stat: ColumnStat::Median,
        }
    }
}

/// A column that evaluates error metrics on test cases.
pub struct AccuracyColumn<T = f64> {
    pub name: String,
    pub metric: Option<Box<ErrorMetric<T>>>,
    pub target_stat: ColumnStat,
}

impl<T> std::fmt::Debug for AccuracyColumn<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccuracyColumn")
            .field("name", &self.name)
            .field("target_stat", &self.target_stat)
            .finish()
    }
}

impl<T> AccuracyColumn<T> {
    /// Set a custom error metric for this column.
    ///
    /// # Example
    ///
    /// ```
    /// # use reproducible::columns::Column;
    /// # use reproducible::metrics::MetricValue;
    /// let col = Column::<f64>::accuracy("Abs Err")
    ///     .with_metric(|a, e| MetricValue::Numerical((a[0] - e[0]).abs()));
    /// ```
    pub fn with_metric<F>(mut self, metric: F) -> Self
    where
        F: Fn(&[T], &[T]) -> MetricValue + Send + Sync + 'static,
    {
        self.metric = Some(Box::new(metric));
        self
    }

    /// Set the statistical aggregator for this column.
    pub fn with_stat(mut self, stat: ColumnStat) -> Self {
        self.target_stat = stat;
        self
    }
}

impl<T> From<AccuracyColumn<T>> for Column<T> {
    fn from(c: AccuracyColumn<T>) -> Self {
        Column::Accuracy(c)
    }
}

/// A column that reads benchmark data from Criterion.
#[derive(Debug)]
pub struct PerformanceColumn {
    pub name: String,
    pub target_stat: ColumnStat,
}

impl PerformanceColumn {
    pub fn with_stat(mut self, stat: ColumnStat) -> Self {
        self.target_stat = stat;
        self
    }
}

impl From<PerformanceColumn> for Column {
    fn from(c: PerformanceColumn) -> Self {
        Column::Performance(c)
    }
}
