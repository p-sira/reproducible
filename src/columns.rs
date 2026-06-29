//! Accuracy and performance columns.

use crate::metrics::{ErrorMetric, MetricValue};
use serde::{Deserialize, Serialize};

/// Statistical aggregator to use for a column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnStat {
    Mean,
    Median,
    Min,
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
            postprocess: None,
        }
    }

    /// Create a new performance column.
    pub fn perf(name: impl Into<String>) -> PerformanceColumn {
        PerformanceColumn {
            name: name.into(),
            target_stat: ColumnStat::Median,
            postprocess: None,
        }
    }

    /// Add a postprocess function to apply to the numerical value in the column.
    ///
    /// # Example
    ///
    /// ```
    /// # use reproducible::columns::Column;
    /// let col = Column::<f64>::accuracy("Mean L2")
    ///     .postprocess(|val| val * 100.0); // Convert to percentage
    /// ```
    pub fn postprocess<F>(self, f: F) -> Self
    where
        F: Fn(f64) -> f64 + Send + Sync + 'static,
    {
        let f = std::sync::Arc::new(f);
        match self {
            Column::Accuracy(mut ac) => {
                ac.postprocess = Some(f.clone());
                Column::Accuracy(ac)
            }
            Column::Performance(mut pc) => {
                pc.postprocess = Some(f.clone());
                Column::Performance(pc)
            }
            Column::Custom(name, func) => Column::Custom(name, func),
        }
    }
}

/// A column that evaluates error metrics on test cases.
pub struct AccuracyColumn<T = f64> {
    pub name: String,
    pub metric: Option<Box<ErrorMetric<T>>>,
    pub target_stat: ColumnStat,
    pub postprocess: Option<std::sync::Arc<dyn Fn(f64) -> f64 + Send + Sync>>,
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

    /// Add a postprocess function to apply to the numerical value in the column.
    ///
    /// # Example
    ///
    /// ```
    /// # use reproducible::columns::Column;
    /// let col = Column::<f64>::accuracy("Mean L2")
    ///     .postprocess(|val| val * 100.0); // Convert to percentage
    /// ```
    pub fn postprocess<F>(mut self, f: F) -> Self
    where
        F: Fn(f64) -> f64 + Send + Sync + 'static,
    {
        self.postprocess = Some(std::sync::Arc::new(f));
        self
    }
}

impl<T> From<AccuracyColumn<T>> for Column<T> {
    fn from(c: AccuracyColumn<T>) -> Self {
        Column::Accuracy(c)
    }
}

/// A column that reads benchmark data from Criterion.
pub struct PerformanceColumn {
    pub name: String,
    pub target_stat: ColumnStat,
    pub postprocess: Option<std::sync::Arc<dyn Fn(f64) -> f64 + Send + Sync>>,
}

impl std::fmt::Debug for PerformanceColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PerformanceColumn")
            .field("name", &self.name)
            .field("target_stat", &self.target_stat)
            .finish()
    }
}

impl PerformanceColumn {
    pub fn with_stat(mut self, stat: ColumnStat) -> Self {
        self.target_stat = stat;
        self
    }

    /// Add a postprocess function to apply to the numerical value in the column.
    ///
    /// # Example
    ///
    /// ```
    /// # use reproducible::columns::{Column, PerformanceColumn};
    /// let col = Column::<f64>::perf("Latency")
    ///     .postprocess(|val| val / 1000.0);
    /// ```
    pub fn postprocess<F>(mut self, f: F) -> Self
    where
        F: Fn(f64) -> f64 + Send + Sync + 'static,
    {
        self.postprocess = Some(std::sync::Arc::new(f));
        self
    }
}

impl From<PerformanceColumn> for Column {
    fn from(c: PerformanceColumn) -> Self {
        Column::Performance(c)
    }
}
