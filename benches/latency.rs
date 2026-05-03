use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn fn_add(inputs: &[f64]) -> Vec<f64> {
    vec![inputs[0] + inputs[1]]
}

fn fn_sub(inputs: &[f64]) -> Vec<f64> {
    vec![inputs[0] - inputs[1]]
}

fn benchmark_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("math");

    group.bench_function("add", |b| b.iter(|| fn_add(black_box(&[1.0, 2.0]))));

    group.bench_function("sub", |b| b.iter(|| fn_sub(black_box(&[1.0, 2.0]))));

    group.finish();
}

criterion_group!(benches, benchmark_latency);
criterion_main!(benches);
