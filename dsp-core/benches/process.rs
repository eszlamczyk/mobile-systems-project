//! Per-hop cost of `Tuner::process`. Must stay well under one hop (21 ms at 48 kHz).

// Index-to-float casts are exact at these sizes.
#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]

use std::f64::consts::TAU;
use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use dsp_core::{Tuner, TunerConfig};

/// The `#[uniffi(default)]` values, which Rust callers have to spell out.
fn config() -> TunerConfig {
    TunerConfig {
        sample_rate: 48_000,
        fft_size: 8192,
        hop_size: 1024,
        min_hz: 50.0,
        max_hz: 5000.0,
        alpha: 1.5,
        beta: 0.01,
        lambda: 0.7,
        epsilon: 1e-6,
    }
}

fn process(c: &mut Criterion) {
    let config = config();
    let hop = config.hop_size as usize;
    let sample_rate = f64::from(config.sample_rate);
    let Ok(tuner) = Tuner::new(config) else {
        unreachable!("default config is valid");
    };

    // One hop of A4 as the bytes the apps pass in. Learn a quiet noise profile
    // first so the subtraction runs against a real profile.
    let pcm: Vec<u8> = (0..hop)
        .map(|n| 0.5 * (TAU * 440.0 * n as f64 / sample_rate).sin() as f32)
        .flat_map(f32::to_ne_bytes)
        .collect();
    let silence: Vec<u8> = (0..48_000)
        .map(|n| 1e-3 * (n as f32 * 0.618).fract())
        .flat_map(f32::to_ne_bytes)
        .collect();
    if tuner.learn_noise(&silence).is_err() {
        unreachable!("one second is more than one frame");
    }

    c.bench_function("process", |b| b.iter(|| tuner.process(black_box(&pcm))));
}

criterion_group!(benches, process);
criterion_main!(benches);
