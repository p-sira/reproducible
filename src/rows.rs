//! Rows for the report.

use std::path::{Path, PathBuf};

use crate::benchmark::default_criterion_root;
use crate::parser::CsvParserOptions;
use serde::{Deserialize, Serialize};

/// A function that a row evaluates.
pub type RowFunction<T> = Box<dyn Fn(&[T]) -> Vec<T> + Send + Sync>;

/// A batch function that a row evaluates over all test cases at once.
pub type BatchRowFunction<T> = Box<dyn Fn(&[Vec<T>]) -> Vec<Vec<T>> + Send + Sync>;

/// A single test row in a report.
pub struct Row<T = f64> {
    /// Identifying name for the row (e.g., function name).
    pub name: String,
    /// Optional function to evaluate for this row.
    pub function: Option<RowFunction<T>>,
    /// Optional batch function to evaluate for this row.
    pub batch_function: Option<BatchRowFunction<T>>,
    pub test_cases: Option<Vec<TestCase<T>>>,
    criterion_root: Option<PathBuf>,
    criterion_id: Option<String>,
    csv_parser_options: CsvParserOptions,
}

impl<T> Row<T> {
    /// Create a new row with a name and an evaluation function.
    ///
    /// # Example
    ///
    /// ```
    /// # use reproducible::rows::Row;
    /// let r = Row::<f64>::new("my_row", |i| vec![i[0] * 2.0]);
    /// ```
    pub fn new(
        name: impl Into<String>,
        func: impl Fn(&[T]) -> Vec<T> + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            function: Some(Box::new(func)),
            batch_function: None,
            test_cases: None,
            criterion_root: None,
            criterion_id: None,
            csv_parser_options: CsvParserOptions::default(),
        }
    }

    /// Create a new row with a name and a batch evaluation function.
    pub fn new_batch(
        name: impl Into<String>,
        func: impl Fn(&[Vec<T>]) -> Vec<Vec<T>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            function: None,
            batch_function: Some(Box::new(func)),
            test_cases: None,
            criterion_root: None,
            criterion_id: None,
            csv_parser_options: CsvParserOptions::default(),
        }
    }

    #[inline]
    pub fn with_function(mut self, func: impl Fn(&[T]) -> Vec<T> + Send + Sync + 'static) -> Self {
        self.function = Some(Box::new(func));
        self
    }

    #[inline]
    pub fn with_test_cases(mut self, test_cases: Vec<TestCase<T>>) -> Self {
        self.test_cases = Some(test_cases);
        self
    }

    #[inline]
    pub fn with_csv_cases(mut self, path: impl AsRef<Path>) -> std::io::Result<Self>
    where
        T: std::str::FromStr + Clone,
        <T as std::str::FromStr>::Err: std::fmt::Display,
    {
        self.test_cases = Some(crate::parser::read_csv_cases(
            path,
            &self.csv_parser_options,
        )?);
        Ok(self)
    }

    #[inline]
    pub fn with_split_csv_cases(
        mut self,
        inputs_path: impl AsRef<Path>,
        expected_path: impl AsRef<Path>,
    ) -> std::io::Result<Self>
    where
        T: std::str::FromStr + Clone,
        <T as std::str::FromStr>::Err: std::fmt::Display,
    {
        self.test_cases = Some(crate::parser::read_split_csv_cases(
            inputs_path,
            expected_path,
            &self.csv_parser_options,
        )?);
        Ok(self)
    }

    #[inline]
    pub fn with_csv_options(mut self, options: CsvParserOptions) -> Self {
        self.csv_parser_options = options;
        self
    }

    #[inline]
    pub fn criterion_root(&self) -> &Path {
        self.criterion_root
            .as_deref()
            .unwrap_or(default_criterion_root())
    }

    #[inline]
    pub fn criterion_id(&self) -> &str {
        self.criterion_id.as_deref().unwrap_or(&self.name)
    }

    #[inline]
    pub fn criterion_path(&self) -> PathBuf {
        self.criterion_root().join(self.criterion_id())
    }

    #[inline]
    pub fn set_criterion_root(&mut self, root: impl Into<PathBuf>) {
        self.criterion_root = Some(root.into());
    }

    #[inline]
    pub fn set_criterion_id(&mut self, id: impl Into<String>) {
        self.criterion_id = Some(id.into());
    }

    #[inline]
    pub fn with_criterion_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.set_criterion_root(root);
        self
    }

    #[inline]
    pub fn with_criterion_id(mut self, id: impl Into<String>) -> Self {
        self.set_criterion_id(id);
        self
    }
}

/// A single accuracy test case:
/// all inputs for a function call + expected output(s).
///
/// # Example
///
/// ```
/// # use reproducible::rows::TestCase;
/// let case = TestCase {
///     inputs: vec![1.0, 2.0],
///     expected: vec![3.0],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase<T = f64> {
    pub inputs: Vec<T>,
    pub expected: Vec<T>,
}
