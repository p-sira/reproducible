use reproducible::parser::{CsvParserOptions, read_csv_cases};
use reproducible::prelude::*;

fn main() {
    // 1. Load test cases from a CSV file
    // The CSV should have columns like: x, y, expected
    // read_accuracy_cases_csv assumes the last column is the expected result.
    let csv_path = "examples/data/cases.csv";
    let test_cases = read_csv_cases::<f64>(csv_path, &CsvParserOptions::default())
        .expect("Failed to load CSV");

    println!("Loaded {} test cases from {}\n", test_cases.len(), csv_path);

    // 2. Define the function to evaluate
    let my_add = |inputs: &[f64]| vec![inputs[0] + inputs[1]];

    // 3. Compose and render the report
    let report = Report::new()
        .with_test_cases(test_cases)
        .with_column(Column::<f64>::accuracy("Accuracy"))
        .with_row(Row::new("My Addition", my_add));

    println!("## CSV-based Accuracy Report\n");
    println!("{}", report.render_markdown());
}
