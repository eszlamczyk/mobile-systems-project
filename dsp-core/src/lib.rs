//! Real-time instrument tuner core: pitch detection and spectrum rendering
//! exposed to Kotlin and Swift through `UniFFI`.

// Skeleton: every method is still `todo!()`. Remove once they are implemented.
#![allow(dead_code, unused_variables)]

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicI32;

mod state;

use state::State;

uniffi::setup_scaffolding!();

/// Analysis parameters, fixed for the lifetime of a [`Tuner`].
#[derive(uniffi::Record)]
pub struct TunerConfig {
    /// Device native sample rate in Hz (44100 or 48000). No resampling is done.
    pub sample_rate: u32,
    /// FFT length in samples. Must be a power of two; default 4096.
    pub fft_size: u32,
    /// Samples between consecutive frames; default 1024.
    pub hop_size: u32,
    /// Lower edge of the analysis band-pass in Hz; default 50.
    pub min_hz: f32,
    /// Upper edge of the analysis band-pass in Hz; default 5000.
    pub max_hz: f32,
}

/// Result of analysing one hop of audio.
#[derive(uniffi::Record)]
pub struct TunerResult {
    /// Detected pitch in Hz, or 0 when the input is gated out.
    pub pitch_hz: f32,
    /// Offset from the target note in cents. Unbounded; the UI saturates at ±50.
    pub cents: f32,
    /// MIDI note closest to `pitch_hz`.
    pub nearest_midi: i32,
    /// YIN confidence in the range 0..=1.
    pub confidence: f32,
    /// RMS loudness of the input. Drives the noise gate.
    pub rms: f32,
}

/// Stateful tuner. Create once, feed audio from the DSP thread, read the
/// target from the UI thread.
///
/// `process` and `learn_noise` must be called from a single DSP thread.
/// `set_target` and `target_spectrum` are lock-free and safe from the UI thread.
#[derive(uniffi::Object)]
pub struct Tuner {
    config: TunerConfig,
    target_midi: AtomicI32,
    inner: Mutex<State>,
}

#[uniffi::export]
impl Tuner {
    /// Creates a tuner and allocates every buffer it will need.
    ///
    /// No allocation happens after this call.
    #[uniffi::constructor]
    #[allow(clippy::needless_pass_by_value)] // UniFFI passes records by value
    #[must_use]
    pub fn new(config: TunerConfig) -> Arc<Self> {
        todo!()
    }

    /// Number of magnitude bins produced per frame, equal to `fft_size / 2`.
    #[uniffi::method]
    #[must_use]
    pub fn spectrum_size(&self) -> u32 {
        todo!()
    }

    /// Sets the note the player is tuning to.
    ///
    /// Atomic store; safe to call from the UI thread at any time.
    #[uniffi::method]
    pub fn set_target(&self, midi_note: i32) {
        todo!()
    }

    /// Returns the ideal spectrum of the current target note, one magnitude per bin.
    ///
    /// Intended for the "ghost" overlay. Call from the UI thread when the target changes.
    #[uniffi::method]
    pub fn target_spectrum(&self) -> Vec<f32> {
        todo!()
    }

    /// Learns the noise profile from a recording of silence.
    ///
    /// `pcm` is native-endian `f32` samples as bytes; about one second is enough.
    #[uniffi::method]
    pub fn learn_noise(&self, pcm: &[u8]) {
        todo!()
    }

    /// Analyses one hop of audio.
    ///
    /// `pcm` holds `hop_size` native-endian `f32` samples as bytes.
    /// `spectrum` receives `spectrum_size()` magnitudes as native-endian `f32` bytes.
    #[uniffi::method]
    #[allow(clippy::needless_pass_by_value)] // placeholder signature, see TODO.md
    pub fn process(&self, pcm: &[u8], spectrum: Vec<u8>) -> TunerResult {
        todo!()
    }
}
