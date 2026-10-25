//! Real-time instrument tuner core: pitch detection and spectrum rendering
//! exposed to Kotlin and Swift through `UniFFI`.

// Skeleton: every method is still `todo!()`. Remove once they are implemented.
#![allow(dead_code, unused_variables)]

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI32, Ordering};

pub mod fft;
mod state;

use state::State;

uniffi::setup_scaffolding!();

/// Analysis parameters, fixed for the lifetime of a [`Tuner`].
///
/// The defaults below are the single source of truth for both apps: the
/// generated Kotlin and Swift constructors take them as default arguments, so
/// the apps pass only `sample_rate` and read `hop_size` back for their buffers.
#[derive(uniffi::Record)]
pub struct TunerConfig {
    /// Device native sample rate in Hz (44100 or 48000). No resampling is done.
    pub sample_rate: u32,
    /// FFT length in samples. Must be a power of two. 8192 trades latency
    /// (171 ms frame at 48 kHz) for accuracy on the low strings (docs/FFT.md).
    #[uniffi(default = 8192)]
    pub fft_size: u32,
    /// Samples between consecutive frames.
    #[uniffi(default = 1024)]
    pub hop_size: u32,
    /// Lower edge of the analysis band-pass in Hz.
    #[uniffi(default = 50.0)]
    pub min_hz: f32,
    /// Upper edge of the analysis band-pass in Hz.
    #[uniffi(default = 5000.0)]
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
    /// `spectrum_size()` magnitudes as native-endian `f32` bytes.
    ///
    /// Returned instead of written into a caller buffer until `UniFFI` ships
    /// `&mut [u8]` arguments (see docs/SPEC.md).
    pub spectrum: Vec<u8>,
}

/// Error returned by every [`Tuner`] call.
///
/// Every exported method is fallible so that Swift sees it as `throws`: a Rust
/// panic then surfaces as a catchable error instead of a `try!` crash, and the
/// app drops the frame.
#[derive(Debug, uniffi::Error)]
#[uniffi(flat_error)]
pub enum TunerError {
    /// Input had the wrong length or shape.
    InvalidInput,
}

impl std::fmt::Display for TunerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput => f.write_str("invalid input"),
        }
    }
}

impl std::error::Error for TunerError {}

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
    /// No allocation happens after this call. The target starts at A4 (MIDI 69).
    ///
    /// # Errors
    ///
    /// [`TunerError::InvalidInput`] if `config` is unusable: `sample_rate` is 0,
    /// `fft_size` is not a power of two of at least 2, `hop_size` is 0 or larger
    /// than `fft_size`, or the band is not `0 <= min_hz < max_hz` with both finite.
    #[uniffi::constructor]
    #[allow(clippy::needless_pass_by_value)] // UniFFI passes records by value
    pub fn new(config: TunerConfig) -> Result<Arc<Self>, TunerError> {
        let sample_rate_ok = config.sample_rate != 0;
        let fft_ok = config.fft_size >= 2 && config.fft_size.is_power_of_two();
        let hop_ok = config.hop_size > 0 && config.hop_size <= config.fft_size;
        let band_ok =
            config.min_hz >= 0.0 && config.max_hz.is_finite() && config.min_hz < config.max_hz;

        if !sample_rate_ok || !fft_ok || !hop_ok || !band_ok {
            return Err(TunerError::InvalidInput);
        }

        let state = State::new(&config);
        Ok(Arc::new(Self {
            config,
            target_midi: AtomicI32::new(69),
            inner: Mutex::new(state),
        }))
    }

    /// Number of magnitude bins produced per frame, equal to `fft_size / 2`.
    ///
    /// # Errors
    ///
    /// Never in practice; fallible only so a panic reaches Swift as a throw.
    #[uniffi::method]
    pub fn spectrum_size(&self) -> Result<u32, TunerError> {
        Ok(self.config.fft_size / 2)
    }

    /// Sets the note the player is tuning to.
    ///
    /// Atomic store; safe to call from the UI thread at any time.
    ///
    /// # Errors
    ///
    /// [`TunerError::InvalidInput`] if `midi_note` is outside 0..=127.
    #[uniffi::method]
    pub fn set_target(&self, midi_note: i32) -> Result<(), TunerError> {
        if !(0..=127).contains(&midi_note) {
            return Err(TunerError::InvalidInput);
        }
        // Relaxed: the target is a single independent value, no other memory hangs off it.
        self.target_midi.store(midi_note, Ordering::Relaxed);
        Ok(())
    }

    /// Returns the ideal spectrum of the current target note, one magnitude per bin.
    ///
    /// Intended for the "ghost" overlay. Call from the UI thread when the target changes.
    ///
    /// # Errors
    ///
    /// Never in practice; fallible only so a panic reaches Swift as a throw.
    #[uniffi::method]
    pub fn target_spectrum(&self) -> Result<Vec<f32>, TunerError> {
        todo!()
    }

    /// Learns the noise profile from a recording of silence.
    ///
    /// `pcm` is native-endian `f32` samples as bytes; about one second is enough.
    ///
    /// # Errors
    ///
    /// [`TunerError::InvalidInput`] if `pcm` is not a whole number of `f32` samples.
    #[uniffi::method]
    pub fn learn_noise(&self, pcm: &[u8]) -> Result<(), TunerError> {
        todo!()
    }

    /// Analyses one hop of audio.
    ///
    /// `pcm` holds `hop_size` native-endian `f32` samples as bytes. The spectrum
    /// comes back in [`TunerResult::spectrum`].
    ///
    /// # Errors
    ///
    /// [`TunerError::InvalidInput`] if `pcm` is not exactly one hop.
    #[uniffi::method]
    pub fn process(&self, pcm: &[u8]) -> Result<TunerResult, TunerError> {
        todo!()
    }
}
