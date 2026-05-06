//! CSV parsing utilities for accuracy test datasets.

use crate::rows::TestCase;
use std::path::Path;

/// Config for parsing CSV rows into [`TestCase`].
#[derive(Debug, Clone)]
pub struct CsvParserOptions {
    /// Column delimiter, default is comma (`,`).
    pub delimiter: u8,
    /// Whether the CSV file has a header row.
    pub has_headers: bool,
}

impl Default for CsvParserOptions {
    fn default() -> Self {
        Self {
            delimiter: b',',
            has_headers: true,
        }
    }
}

macro_rules! cannot_read_csv {
    ($path:expr) => {
        |e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("cannot read CSV {}: {}", $path.as_ref().display(), e),
            )
        }
    };
}
macro_rules! cannot_parse_csv_row {
    ($row_idx:expr) => {
        |e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("cannot parse CSV row {}: {}", $row_idx + 1, e),
            )
        }
    };
}

macro_rules! cannot_parse_cell {
    ($row_idx:expr, $col_idx:expr) => {
        |e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "cannot parse row {}, column {} as f64: {e}",
                    $row_idx + 1,
                    $col_idx + 1
                ),
            )
        }
    };
}

fn parse_cell<T>(cell: &str, row_idx: usize, col_idx: usize) -> std::io::Result<T>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    cell.parse::<T>()
        .map_err(cannot_parse_cell!(row_idx, col_idx))
}

/// Parse CSV rows into vectors of `T`.
///
/// Each row becomes a `Vec<T>`.
pub fn read_csv_vectors<T>(
    path: impl AsRef<Path>,
    options: &CsvParserOptions,
) -> std::io::Result<Vec<Vec<T>>>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(options.delimiter)
        .has_headers(options.has_headers)
        .from_path(path.as_ref())
        .map_err(cannot_read_csv!(path))?;

    let mut out = Vec::new();
    for (row_idx, row) in reader.records().enumerate() {
        let record = row.map_err(cannot_parse_csv_row!(row_idx))?;
        let mut vals = Vec::with_capacity(record.len());
        for (i, v) in record.iter().enumerate() {
            vals.push(parse_cell(v, row_idx, i)?);
        }
        out.push(vals);
    }
    Ok(out)
}

/// Parse CSV rows of the form `(input1, input2, ..., expected)`.
///
/// The last column is always interpreted as the expected value.
///
/// # Example
///
/// ```no_run
/// use reproducible::parser::{read_csv_cases, CsvParserOptions};
/// let cases = read_csv_cases::<f64>("data.csv", &CsvParserOptions::default()).unwrap();
/// ```
pub fn read_csv_cases<T>(
    path: impl AsRef<Path>,
    options: &CsvParserOptions,
) -> std::io::Result<Vec<TestCase<T>>>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(options.delimiter)
        .has_headers(options.has_headers)
        .from_path(path.as_ref())
        .map_err(cannot_read_csv!(path))?;

    let mut out = Vec::new();
    for (row_idx, row) in reader.records().enumerate() {
        let record = row.map_err(cannot_parse_csv_row!(row_idx))?;
        if record.len() < 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "row {} has {} column(s); expected at least 2",
                    row_idx + 1,
                    record.len()
                ),
            ));
        }

        let expected_col = record.len() - 1;
        let mut inputs = Vec::with_capacity(expected_col);
        for i in 0..expected_col {
            let v = record.get(i).ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("missing value at row {}, column {}", row_idx + 1, i + 1),
                )
            })?;
            inputs.push(parse_cell(v, row_idx, i)?);
        }

        let expected_cell = record.get(expected_col).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "missing expected value at row {}, column {}",
                    row_idx + 1,
                    expected_col + 1
                ),
            )
        })?;
        let expected = parse_cell(expected_cell, row_idx, expected_col)?;

        out.push(TestCase {
            inputs,
            expected: vec![expected],
        });
    }

    Ok(out)
}

/// Parse accuracy cases from two separate CSV files: one for inputs, one for expected values.
///
/// Both files must have the same number of rows. Row `i` in the inputs file corresponds to
/// row `i` in the expected file.
///
/// # Example
///
/// ```no_run
/// use reproducible::parser::{read_split_csv_cases, CsvParserOptions};
/// let cases = read_split_csv_cases::<f64>("inputs.csv", "expected.csv", &CsvParserOptions::default()).unwrap();
/// ```
pub fn read_split_csv_cases<T>(
    inputs_path: impl AsRef<Path>,
    expected_path: impl AsRef<Path>,
    options: &CsvParserOptions,
) -> std::io::Result<Vec<TestCase<T>>>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let inputs_vectors = read_csv_vectors(inputs_path, options)?;
    let expected_vectors = read_csv_vectors(expected_path, options)?;

    if inputs_vectors.len() != expected_vectors.len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "row count mismatch: inputs has {} rows, expected has {} rows",
                inputs_vectors.len(),
                expected_vectors.len()
            ),
        ));
    }

    let mut out = Vec::with_capacity(inputs_vectors.len());
    for (inputs, expected) in inputs_vectors.into_iter().zip(expected_vectors) {
        out.push(TestCase { inputs, expected });
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn read_accuracy_csv_last_column_expected() {
        let tmp = tempdir().expect("tempdir");
        let csv_path = tmp.path().join("cases.csv");
        std::fs::write(&csv_path, "x,y,expected\n1.0,2.0,3.0\n2.0,5.0,7.0\n").expect("write");

        let cases = read_csv_cases::<f64>(&csv_path, &CsvParserOptions::default()).expect("parse");
        assert_eq!(cases.len(), 2);
        assert_eq!(cases[0].inputs, vec![1.0, 2.0]);
        assert_eq!(cases[0].expected, vec![3.0]);
    }

    #[test]
    fn read_csv_empty_file() {
        let tmp = tempdir().expect("tempdir");
        let csv_path = tmp.path().join("empty.csv");
        std::fs::write(&csv_path, "").expect("write");

        let cases = read_csv_cases::<f64>(&csv_path, &CsvParserOptions::default()).expect("parse");
        assert_eq!(cases.len(), 0);
    }

    #[test]
    fn read_csv_only_headers() {
        let tmp = tempdir().expect("tempdir");
        let csv_path = tmp.path().join("headers.csv");
        std::fs::write(&csv_path, "col1,col2,expected\n").expect("write");

        let cases = read_csv_cases::<f64>(&csv_path, &CsvParserOptions::default()).expect("parse");
        assert_eq!(cases.len(), 0);
    }

    #[test]
    fn read_csv_invalid_numeric() {
        let tmp = tempdir().expect("tempdir");
        let csv_path = tmp.path().join("invalid.csv");
        std::fs::write(&csv_path, "1.0,abc,3.0\n").expect("write");

        let res = read_csv_cases::<f64>(
            &csv_path,
            &CsvParserOptions {
                has_headers: false,
                ..Default::default()
            },
        );
        assert!(res.is_err());
        assert!(
            res.unwrap_err()
                .to_string()
                .contains("cannot parse row 1, column 2")
        );
    }

    #[test]
    fn read_csv_different_delimiter() {
        let tmp = tempdir().expect("tempdir");
        let csv_path = tmp.path().join("semi.csv");
        std::fs::write(&csv_path, "1.0;2.0;3.0\n").expect("write");

        let options = CsvParserOptions {
            delimiter: b';',
            has_headers: false,
        };
        let cases = read_csv_cases::<f64>(&csv_path, &options).expect("parse");
        assert_eq!(cases[0].inputs, vec![1.0, 2.0]);
        assert_eq!(cases[0].expected, vec![3.0]);
    }

    #[test]
    fn read_csv_pathological_numbers() {
        let tmp = tempdir().expect("tempdir");
        let csv_path = tmp.path().join("patho.csv");
        std::fs::write(&csv_path, "NaN,inf,-inf,1.0\n").expect("write");

        let cases = read_csv_cases::<f64>(
            &csv_path,
            &CsvParserOptions {
                has_headers: false,
                ..Default::default()
            },
        )
        .expect("parse");
        assert!(cases[0].inputs[0].is_nan());
        assert!(cases[0].inputs[1].is_infinite());
        assert!(cases[0].inputs[2].is_infinite());
        assert_eq!(cases[0].expected, vec![1.0]);
    }

    #[test]
    fn read_csv_generic_f32() {
        let tmp = tempdir().expect("tempdir");
        let csv_path = tmp.path().join("f32.csv");
        std::fs::write(&csv_path, "1.0,2.0\n").expect("write");

        let cases = read_csv_cases::<f32>(
            &csv_path,
            &CsvParserOptions {
                has_headers: false,
                ..Default::default()
            },
        )
        .expect("parse");
        assert_eq!(cases[0].inputs, vec![1.0f32]);
        assert_eq!(cases[0].expected, vec![2.0f32]);
    }

    #[test]
    fn test_read_separate_csvs() {
        let tmp = tempdir().expect("tempdir");
        let inputs_path = tmp.path().join("inputs.csv");
        let expected_path = tmp.path().join("expected.csv");
        std::fs::write(&inputs_path, "x,y\n1.0,2.0\n2.0,5.0\n").expect("write");
        std::fs::write(&expected_path, "expected\n3.0\n7.0\n").expect("write");

        let cases =
            read_split_csv_cases::<f64>(&inputs_path, &expected_path, &CsvParserOptions::default())
                .expect("parse");
        assert_eq!(cases.len(), 2);
        assert_eq!(cases[0].inputs, vec![1.0, 2.0]);
        assert_eq!(cases[0].expected, vec![3.0]);
    }

    #[test]
    fn test_read_separate_csvs_mismatch() {
        let tmp = tempdir().expect("tempdir");
        let inputs_path = tmp.path().join("inputs.csv");
        let expected_path = tmp.path().join("expected.csv");
        std::fs::write(&inputs_path, "x,y\n1.0,2.0\n2.0,5.0\n").expect("write");
        std::fs::write(&expected_path, "expected\n3.0\n").expect("write");

        let res =
            read_split_csv_cases::<f64>(&inputs_path, &expected_path, &CsvParserOptions::default());
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("row count mismatch"));
    }
}
