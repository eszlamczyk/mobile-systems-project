//! Radix-2 FFT and the tables it runs on (docs/FFT.md sections 1 and 2).
//!
//! Public so `tests/fft.rs` can check it against the definition; not exported
//! through `UniFFI`.

use std::f64::consts::PI;

/// Fills `window` with a symmetric Hann window (divides by `N - 1`, so both
/// ends are 0). `window.len()` is at least 2.
// Indices stay far below 2^52, and the f64 result is narrowed to f32 on purpose.
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
pub fn fill_hann(window: &mut [f32]) {
    let last = (window.len() - 1) as f64;
    for (n, w) in window.iter_mut().enumerate() {
        *w = (0.5 * (1.0 - (2.0 * PI * n as f64 / last).cos())) as f32;
    }
}

/// Fills `twiddles` with `(cos t, sin t)` for `t = 2 pi k / N`, `k < N / 2`,
/// where `N = 2 * twiddles.len()`. The butterfly applies the minus sign of
/// `W_N^k = e^(-2 pi i k / N)`.
// Indices stay far below 2^52, and the f64 result is narrowed to f32 on purpose.
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
pub fn fill_twiddles(twiddles: &mut [(f32, f32)]) {
    let big_n = twiddles.len() as f64 * 2.0;
    for (k, twiddle) in twiddles.iter_mut().enumerate() {
        let t = 2.0 * PI * k as f64 / big_n;
        *twiddle = (t.cos() as f32, t.sin() as f32);
    }
}

/// In-place forward DFT `X[k] = sum x[n] e^(-2 pi i k n / N)` of the complex
/// signal `re + i im`, unnormalised.
///
/// `re`, `im` have length `N`, a power of two, and `twiddles` comes from
/// [`fill_twiddles`] with length `N / 2`.
pub fn fft(re: &mut [f32], im: &mut [f32], twiddles: &[(f32, f32)]) {
    let n = re.len();
    if n < 2 {
        return;
    }

    let shift = usize::BITS - n.trailing_zeros();
    for i in 0..n {
        let j = i.reverse_bits() >> shift;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }

    // Stages of span 2, 4, ..., N. A block of span `len` needs W_len^k, which is
    // W_N^(k * N / len), so it reads the table with stride N / len.
    let mut len = 2;
    while len <= n {
        let half = len / 2;
        let stride = n / len;
        for (block_re, block_im) in re.chunks_exact_mut(len).zip(im.chunks_exact_mut(len)) {
            let (even_re, odd_re) = block_re.split_at_mut(half);
            let (even_im, odd_im) = block_im.split_at_mut(half);
            let pairs = even_re
                .iter_mut()
                .zip(even_im.iter_mut())
                .zip(odd_re.iter_mut().zip(odd_im.iter_mut()));
            for (((er, ei), (or, oi)), &(c, s)) in pairs.zip(twiddles.iter().step_by(stride)) {
                // W * odd with W = c - i s.
                let tr = c * *or + s * *oi;
                let ti = c * *oi - s * *or;
                *or = *er - tr;
                *oi = *ei - ti;
                *er += tr;
                *ei += ti;
            }
        }
        len *= 2;
    }
}
