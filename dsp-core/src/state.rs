//! Mutable DSP state owned by the analysis thread.

use std::ops::Range;

use crate::TunerConfig;
use crate::fft::{fft, fill_hann, fill_twiddles};

/// Buffers that `process` and `learn_noise` read and write.
///
/// Allocated once by [`State::new`]; never resized afterwards.
pub struct State {
    pub window: Vec<f32>,
    pub twiddles: Vec<(f32, f32)>,
    /// Rolling input frame; each hop shifts in `hop_size` new samples. Never
    /// transformed in place, so it stays raw for YIN.
    pub frame: Vec<f32>,
    pub re: Vec<f32>,
    pub im: Vec<f32>,
    /// Per-bin noise power `|N[k]|^2` learned from silence, `fft_size / 2` entries.
    /// Zero until learned, which makes spectral subtraction a no-op.
    pub noise_profile: Vec<f32>,
    pub smoothed: Vec<f32>,
    /// Bins kept by the band-pass; every bin outside is zeroed. May be empty.
    pub band: Range<usize>,
}

impl State {
    pub fn new(config: &TunerConfig) -> Self {
        let fft_size = config.fft_size as usize;
        let bins = fft_size / 2;
        let mut window = vec![0.0; fft_size];
        fill_hann(&mut window);
        let mut twiddles = vec![(0.0, 0.0); bins];
        fill_twiddles(&mut twiddles);

        Self {
            window,
            twiddles,
            frame: vec![0.0; fft_size],
            re: vec![0.0; fft_size],
            im: vec![0.0; fft_size],
            noise_profile: vec![0.0; bins],
            smoothed: vec![0.0; bins],
            band: band(config, bins),
        }
    }
}

/// Bins whose centre frequency `k * sample_rate / fft_size` lies in
/// `min_hz..=max_hz`, clamped to the `bins` that are output.
// `Tuner::new` guarantees a finite band with `min_hz >= 0`, so the casts only
// truncate the already-rounded value; `as` saturates for bands past Nyquist.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn band(config: &TunerConfig, bins: usize) -> Range<usize> {
    let hz_to_bin = config.fft_size as f32 / config.sample_rate as f32;
    let low = (config.min_hz * hz_to_bin).ceil() as usize;
    let high = ((config.max_hz * hz_to_bin).floor() as usize).saturating_add(1);
    low.min(bins)..high.min(bins)
}

/// Hann-windowed FFT of `samples` into `re` and `im`: bin `k` has power
/// `re[k]^2 + im[k]^2`.
///
/// `samples` must yield exactly `fft_size` values: `zip` stops at the shorter
/// side, so fewer would leave the previous frame's tail in `re`.
pub fn transform(
    samples: impl IntoIterator<Item = f32>,
    window: &[f32],
    twiddles: &[(f32, f32)],
    re: &mut [f32],
    im: &mut [f32],
) {
    for ((r, x), w) in re.iter_mut().zip(samples).zip(window) {
        *r = x * w;
    }
    im.fill(0.0);
    fft(re, im, twiddles);
}
