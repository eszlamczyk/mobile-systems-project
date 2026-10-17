//! Mutable DSP state owned by the analysis thread.

use crate::TunerConfig;

/// Buffers that `process` and `learn_noise` read and write.
///
/// Allocated once by [`State::new`]; never resized afterwards.
pub struct State {
    /// Analysis window, `fft_size` samples.
    pub window: Vec<f32>,
    /// FFT twiddle factors as `(cos, sin)` pairs, `fft_size / 2` entries.
    pub twiddles: Vec<(f32, f32)>,
    /// Rolling input frame, `fft_size` samples. Each hop shifts in `hop_size` new ones.
    pub frame: Vec<f32>,
    /// Per-bin noise floor learned from silence, `fft_size / 2` magnitudes.
    pub noise_profile: Vec<f32>,
}

impl State {
    /// Allocates every buffer for `config`, zero-filled.
    pub fn new(config: &TunerConfig) -> Self {
        let fft_size = config.fft_size as usize;
        let bins = fft_size / 2;
        Self {
            window: vec![0.0; fft_size],
            twiddles: vec![(0.0, 0.0); bins],
            frame: vec![0.0; fft_size],
            noise_profile: vec![0.0; bins],
        }
    }
}
