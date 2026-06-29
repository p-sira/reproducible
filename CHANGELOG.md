# 0.4

## 0.4.0

- Postprocess the value for a column, e.g. dividing by the number of elements, using `Column::postprocess`.
- Extract specific Criterion performance statistics, using `ColumnStat`.

# 0.3

## 0.3.1

- Support any criterion ID, instead of just restricting to one slash (directory/group).

## 0.3.0

- `Env`: a struct for extracting and storing environment context for reproducible tests. Also provides `to_string` method.
- Improve documentation.

# 0.2

## 0.2.0

### Breaking Changes

- `with_test_cases_from_csv` and `with_test_cases_from_separate_csvs` have been renamed to `with_csv_cases` and `with_split_csv_cases`, respectively.

### New Features

- Support batch functions to evaluate all test cases altogether. Use `Row::new_batch` instead of `Row::new`.

### Enhancements

- Add `with_*` methods to `Report` for `RenderConfig` fields.

# 0.1.0

- Initial release.