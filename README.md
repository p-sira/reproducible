# reproducible

**reproducible** is a Rust library for generating reproducible **accuracy** and **benchmark** reports.

<p>
    <a href="https://opensource.org/license/BSD-3-clause" style="text-decoration:none">
        <img src="https://img.shields.io/badge/License-BSD--3--Clause-brightgreen.svg" alt="License">
    </a>
    <a href="https://crates.io/crates/reproducible" style="text-decoration:none">
        <img src="https://img.shields.io/crates/v/reproducible" alt="Crate">
    </a>
    <a href="https://crates.io/crates/reproducible" style="text-decoration: none">
        <img src="https://img.shields.io/crates/d/reproducible" alt="Total Downloads">
    </a>
    <a href="https://docs.rs/reproducible" style="text-decoration:none">
        <img src="https://img.shields.io/badge/Docs-docs.rs-blue" alt="Documentation">
    </a>
</p>

It is designed to:

- work with any Rust crate (no project-specific assumptions),
- be quick to set up,
- stay customizable for advanced workflows.

## Installation

```bash
cargo add reproducible
```

## Quick Start

```rust
use reproducible::prelude::*;
use reproducible::metrics;

// 1. Define the functions we want to evaluate

let fn_add = |inputs: &[f64]| vec![inputs[0] + inputs[1] + 5.0 * f64::EPSILON];
let fn_sub = |inputs: &[f64]| vec![inputs[0] - inputs[1] - 10.0 * f64::EPSILON];

// 2. Compose the report

let report = Report::new()

    // Add columns for metrics and statistics
    .with_column(
        Column::<f64>::accuracy("Mean Relative Error (eps)")
            .with_metric(metrics::rel_err_eps)
    )
    .with_column(
        Column::<f64>::accuracy("Max Absolute Error")
            .with_metric(metrics::abs_err)
            .with_stat(ColumnStat::Max)
    )
    .with_column(Column::<f64>::perf("Latency"))
    
    // Add rows corresponding to each function you want to evaluate
    .with_row(Row::new("math/add", fn_add).with_test_cases(vec![
        TestCase { inputs: vec![1.0, 2.0], expected: vec![3.0] },
    ]))
    .with_row(Row::new("math/sub", fn_sub).with_test_cases(vec![
        TestCase { inputs: vec![1.0, 2.0], expected: vec![-1.0] },
    ]));

println!("{}", report.render_markdown());
```

## Features

- **Fluent API**: Compose reports with a declarative builder pattern.
- **Accuracy Metrics**: Evaluate functions against test cases with built-in or custom metrics.
- **Criterion Integration**: Automatically pull benchmark results from Criterion's output directory.
- **CSV Support**: Load test cases directly from CSV files.
- **Customizable Rendering**: Control precision, scientific notation, and table styles (Markdown, Modern, Sharp, etc.).

## Examples

See the `examples/` directory for more details:
- `basic.rs`: Core functionality and reporting.
- `csv_loading.rs`: Loading test cases from external files.
