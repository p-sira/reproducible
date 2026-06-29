//! Markdown table rendering for reports.

use crate::columns::{Column, ColumnStat};
use crate::report::Report;
use crate::stats::Stats;
use getset::{Setters, WithSetters};
use tabled::builder::Builder;
use tabled::settings::Style;

/// Presets for table styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableStyle {
    /// Standard GitHub-flavored markdown table.
    Markdown,
    /// Modern look with bold headers.
    Modern,
    /// Sharp corners with double lines.
    Sharp,
    /// Rounded corners.
    Rounded,
    /// No borders.
    Blank,
}

/// Optional rendering configuration.
#[derive(Debug, Clone, Setters, WithSetters)]
#[getset(set = "pub", set_with = "pub")]
pub struct RenderConfig {
    /// Number of decimal places to show.
    pub float_precision: usize,
    /// Cutoff for switching to scientific notation.
    pub scientific_cutoff: f64,
    /// Unit label for error columns (e.g., "ε" or "err").
    pub error_unit: String,
    /// Visual style of the markdown table.
    pub table_style: TableStyle,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            float_precision: 2,
            scientific_cutoff: 1e3,
            error_unit: "ε".to_owned(),
            table_style: TableStyle::Markdown,
        }
    }
}

fn fmt_float(x: f64, cfg: &RenderConfig) -> String {
    if x.is_nan() {
        "NAN".to_owned()
    } else if x != 0.0
        && (x.abs() >= cfg.scientific_cutoff || x.abs() < 1.0 / cfg.scientific_cutoff)
    {
        format!("{:.*e}", cfg.float_precision, x)
    } else {
        format!("{:.*}", cfg.float_precision, x)
    }
}

fn format_categorical(cs: &crate::stats::CategoricalStats) -> String {
    if cs.counts.is_empty() {
        return "N/A".to_string();
    }
    let mut entries: Vec<_> = cs.counts.iter().collect();
    entries.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));

    let (label, count) = entries[0];
    let percent = (*count as f64 / cs.total as f64) * 100.0;
    format!("{} ({:.0}%)", label, percent)
}

fn fmt_time_ns(x: f64) -> String {
    if x.is_nan() {
        "NAN".to_owned()
    } else if x < 1_000.0 {
        format!("{x:.1} ns")
    } else if x < 1_000_000.0 {
        format!("{:.1} us", x / 1_000.0)
    } else {
        format!("{:.1} ms", x / 1_000_000.0)
    }
}

fn get_stat_val(stats: &Stats, stat: ColumnStat) -> f64 {
    match stats {
        Stats::Numerical(ns) => match stat {
            ColumnStat::Mean => ns.mean,
            ColumnStat::Median => ns.median,
            ColumnStat::Min => ns.min,
            ColumnStat::Max => ns.max,
            ColumnStat::P99 => ns.p99,
            ColumnStat::P95 => ns.p95,
            ColumnStat::Variance => ns.variance,
        },
        Stats::Categorical(_) => f64::NAN,
    }
}

/// Dynamic markdown renderer using the declarative Report structure.
pub fn render_dynamic_markdown<T: Clone>(report: &Report<T>) -> String {
    let mut builder = Builder::default();
    let cfg = report.render_config();

    // Headers: "Function" + Column Names
    let mut headers = vec!["Function".to_string()];
    for col in report.columns() {
        let name = match col {
            Column::Accuracy(c) => c.name.clone(),
            Column::Performance(c) => c.name.clone(),
            Column::Custom(name, _) => name.clone(),
        };
        headers.push(name);
    }
    builder.push_record(headers);

    // Rows
    for row in report.rows() {
        let mut row_data = vec![row.name.clone()];

        // Materialize evaluation results for this row if it has a function
        let cached_results: Option<Vec<Vec<T>>> = if let Some(batch_func) = &row.batch_function {
            let cases = row.test_cases.as_ref().unwrap_or(report.test_cases());
            let inputs: Vec<Vec<T>> = cases.iter().map(|c| c.inputs.clone()).collect();
            Some(batch_func(&inputs))
        } else if let Some(func) = &row.function {
            let cases = row.test_cases.as_ref().unwrap_or(report.test_cases());
            Some(cases.iter().map(|c| func(&c.inputs)).collect())
        } else {
            None
        };

        for col in report.columns() {
            let val = match col {
                Column::Accuracy(ac) => {
                    if let Some(cached) = &cached_results {
                        if row.function.is_none() && row.batch_function.is_none() {
                            "N/A".to_string()
                        } else {
                            let cases = row.test_cases.as_ref().unwrap_or(report.test_cases());
                            let metric =
                                ac.metric.as_deref().unwrap_or(report.default_metric_ref());

                            let results: Vec<crate::metrics::MetricValue> = cached
                                .iter()
                                .zip(cases.iter())
                                .map(|(actual, c)| metric(actual, &c.expected))
                                .collect();

                            let stats = Stats::from_metric_values(&results);
                            match stats {
                                Stats::Numerical(ns) => {
                                    let mut val = get_stat_val(&Stats::Numerical(ns), ac.target_stat);
                                    if let Some(pp) = &ac.postprocess {
                                        val = pp(val);
                                    }
                                    fmt_float(val, cfg)
                                }
                                Stats::Categorical(cs) => format_categorical(&cs),
                            }
                        }
                    } else {
                        "N/A".to_string()
                    }
                }
                Column::Performance(pc) => {
                    let mut val = crate::benchmark::extract_criterion_stat_ns_with_id(
                        row.criterion_root(),
                        row.criterion_id(),
                        pc.target_stat,
                    )
                    .unwrap_or(f64::NAN);
                    if let Some(pp) = &pc.postprocess {
                        val = pp(val);
                    }
                    fmt_time_ns(val)
                }
                Column::Custom(_, f) => {
                    if let Some(cached) = &cached_results {
                        if row.function.is_none() && row.batch_function.is_none() {
                            "N/A".to_string()
                        } else {
                            let cases = row.test_cases.as_ref().unwrap_or(report.test_cases());
                            let results: Vec<crate::metrics::MetricValue> = cached
                                .iter()
                                .zip(cases.iter())
                                .map(|(actual, c)| {
                                    (report.default_metric_ref())(actual, &c.expected)
                                })
                                .collect();
                            let stats = Stats::from_metric_values(&results);
                            f(&stats)
                        }
                    } else {
                        "N/A".to_string()
                    }
                }
            };
            row_data.push(val);
        }
        builder.push_record(row_data);
    }

    match cfg.table_style {
        TableStyle::Markdown => builder.build().with(Style::markdown()).to_string(),
        TableStyle::Modern => builder.build().with(Style::modern()).to_string(),
        TableStyle::Sharp => builder.build().with(Style::sharp()).to_string(),
        TableStyle::Rounded => builder.build().with(Style::rounded()).to_string(),
        TableStyle::Blank => builder.build().with(Style::blank()).to_string(),
    }
}
