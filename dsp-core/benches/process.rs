//! Per-hop cost of `Tuner::process`. Must stay well under one hop (21 ms at 48 kHz).

use criterion::{Criterion, criterion_group, criterion_main};

fn process(c: &mut Criterion) {
    // TODO: build a `Tuner` with the default config and bench one `process` call
    // on a synthesized sine once `process` is implemented.
    c.bench_function("process", |b| b.iter(|| ()));
}

criterion_group!(benches, process);
criterion_main!(benches);
