//! Hand-written FFT against the definition (docs/FFT.md section 1) and the
//! Hann window (section 2).

// Index-to-float casts are exact at these sizes.
#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]

use std::f64::consts::TAU;

use dsp_core::fft::{fft, fill_hann, fill_twiddles};

const N: usize = 4096;

fn twiddles(n: usize) -> Vec<(f32, f32)> {
    let mut twiddles = vec![(0.0, 0.0); n / 2];
    fill_twiddles(&mut twiddles);
    twiddles
}

/// FFT of a real signal, as `(re, im)`.
fn transform(x: &[f32]) -> (Vec<f32>, Vec<f32>) {
    let mut re = x.to_vec();
    let mut im = vec![0.0; x.len()];
    fft(&mut re, &mut im, &twiddles(x.len()));
    (re, im)
}

/// Uniform samples in `[-1, 1)` from a fixed-seed xorshift, so failures reproduce.
fn random_signal(n: usize, seed: u64) -> Vec<f32> {
    let mut state = seed;
    (0..n)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 40) as f32 / (1u64 << 23) as f32 - 1.0
        })
        .collect()
}

/// `O(N^2)` DFT in `f64` with the twiddle index reduced mod `N`. Both matter:
/// an `f32` reference with the raw angle is itself off by about `2e-2`.
fn naive_dft(x: &[f32]) -> Vec<(f64, f64)> {
    let n = x.len();
    let table: Vec<(f64, f64)> = (0..n)
        .map(|j| {
            let angle = TAU * j as f64 / n as f64;
            (angle.cos(), -angle.sin())
        })
        .collect();
    (0..n)
        .map(|k| {
            x.iter().enumerate().fold((0.0, 0.0), |(re, im), (j, &v)| {
                let (c, s) = table[(k * j) % n];
                (re + f64::from(v) * c, im + f64::from(v) * s)
            })
        })
        .collect()
}

/// Largest absolute difference, real or imaginary part, between FFT and naive DFT.
fn max_error(x: &[f32]) -> f64 {
    let (re, im) = transform(x);
    naive_dft(x)
        .iter()
        .zip(re.iter().zip(&im))
        .map(|(&(want_re, want_im), (&re, &im))| {
            (f64::from(re) - want_re)
                .abs()
                .max((f64::from(im) - want_im).abs())
        })
        .fold(0.0, f64::max)
}

#[test]
fn impulse_gives_flat_spectrum() {
    let mut x = vec![0.0; 16];
    x[0] = 1.0;
    let (re, im) = transform(&x);
    for (k, (re, im)) in re.iter().zip(&im).enumerate() {
        assert!(
            (re - 1.0).abs() < 1e-6 && im.abs() < 1e-6,
            "bin {k}: {re} + {im}i"
        );
    }
}

#[test]
fn matches_naive_dft_for_every_small_size() {
    for log2 in 1..=10 {
        let n = 1 << log2;
        let error = max_error(&random_signal(n, 0x5eed + n as u64));
        assert!(error < 1e-3, "N {n}: max abs error {error:e}");
    }
}

#[test]
fn matches_naive_dft() {
    let error = max_error(&random_signal(N, 0xdead_beef));
    assert!(error < 1e-3, "max abs error {error:e}");
}

#[test]
fn parseval_holds() {
    let x = random_signal(N, 42);
    let (re, im) = transform(&x);
    let time: f64 = x.iter().map(|&v| f64::from(v).powi(2)).sum();
    let freq: f64 = re
        .iter()
        .zip(&im)
        .map(|(&re, &im)| f64::from(re).powi(2) + f64::from(im).powi(2))
        .sum::<f64>()
        / N as f64;
    let relative = (time - freq).abs() / time;
    assert!(
        relative < 1e-5,
        "time {time}, freq {freq}, relative {relative:e}"
    );
}

#[test]
fn sine_on_exact_bin_lands_only_in_that_bin_and_its_mirror() {
    let sample_rate = 48_000.0;
    let peak = N as f32 / 2.0;
    for k in [1, 37, N / 4, N / 2 - 1] {
        let freq = k as f64 * sample_rate / N as f64;
        let x: Vec<f32> = (0..N)
            .map(|n| (TAU * freq * n as f64 / sample_rate).sin() as f32)
            .collect();
        let (re, im) = transform(&x);
        for (bin, (re, im)) in re.iter().zip(&im).enumerate() {
            let magnitude = re.hypot(*im);
            if bin == k || bin == N - k {
                assert!(
                    (magnitude - peak).abs() < 1e-3 * peak,
                    "k {k}: bin {bin} is {magnitude}, want {peak}"
                );
            } else {
                assert!(
                    magnitude < 1e-5 * peak,
                    "k {k}: bin {bin} leaked {magnitude}"
                );
            }
        }
    }
}

#[test]
fn hann_window_matches_definition() {
    let mut window = vec![0.0; N];
    fill_hann(&mut window);
    for (n, &w) in window.iter().enumerate() {
        let want = 0.5 * (1.0 - (TAU * n as f64 / (N - 1) as f64).cos());
        assert!(
            (f64::from(w) - want).abs() < 1e-5,
            "n {n}: {w}, want {want}"
        );
    }
}
