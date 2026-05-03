use reproducible::prelude::*;

fn main() {
    // 1. Define the functions we want to evaluate
    let my_add = |inputs: &[f64]| vec![inputs[0] + inputs[1]];
    let my_sub = |inputs: &[f64]| vec![inputs[0] - inputs[1]];

    // 2. Define some test cases (inputs and expected outputs)
    let test_cases = vec![
        TestCase {
            inputs: vec![1.0, 2.0],
            expected: vec![3.0],
        },
        TestCase {
            inputs: vec![10.0, 5.0],
            expected: vec![15.0], // my_add is correct here
        },
        TestCase {
            inputs: vec![0.0, 0.0],
            expected: vec![1e-10], // my_add will have a small error
        },
    ];

    // 3. Compose the report
    let report = Report::new()
        .with_test_cases(test_cases)
        // Add an accuracy column (Mean error by default)
        .with_column(Column::<f64>::accuracy("Mean Accuracy"))
        // Add another accuracy column with a specific statistic
        .with_column(Column::<f64>::accuracy("Max Error").with_stat(ColumnStat::Max))
        // Add a performance column
        .with_column(Column::<f64>::perf("Latency").with_stat(ColumnStat::Median))
        // Add our rows
        .with_row(Row::new("My Add Fn", my_add).with_criterion_id("math/add"))
        .with_row(Row::new("My Sub Fn", my_sub).with_criterion_id("math/sub"));

    // 4. Render to Markdown
    println!("## Reproducible Accuracy & Performance Report\n");
    println!("{}", report.render_markdown());
}
