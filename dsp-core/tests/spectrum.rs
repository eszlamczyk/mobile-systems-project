//! The spectrum path through the public API: input checks, peak placement,
//! band-pass, noise subtraction and smoothing (docs/FFT.md sections 3 and 5).
//!
//! `spectrum` is in dBFS: a full-scale sine on a bin reads 0, a cut bin
//! reads `20 log10(epsilon)`.

// Index-to-float casts are exact at these sizes.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use std::f64::consts::TAU;

use dsp_core::{Tuner, TunerConfig, TunerError};

const SAMPLE_RATE: u32 = 48_000;
const FFT_SIZE: usize = 8192;
const HOP: usize = 1024;
const MIN_HZ: f32 = 50.0;
const MAX_HZ: f32 = 5000.0;
const EPSILON: f32 = 1e-6;

fn config() -> TunerConfig {
    TunerConfig {
        sample_rate: SAMPLE_RATE,
        fft_size: FFT_SIZE as u32,
        hop_size: HOP as u32,
        min_hz: MIN_HZ,
        max_hz: MAX_HZ,
        alpha: 1.5,
        beta: 0.01,
        lambda: 0.7,
        epsilon: EPSILON,
    }
}

fn bytes(samples: &[f32]) -> Vec<u8> {
    samples.iter().flat_map(|s| s.to_ne_bytes()).collect()
}

fn floats(bytes: &[u8]) -> Vec<f32> {
    let (chunks, _) = bytes.as_chunks::<4>();
    chunks.iter().map(|&c| f32::from_ne_bytes(c)).collect()
}

/// Sine at exactly the centre of `bin`.
fn sine(bin: usize, amplitude: f32, len: usize) -> Vec<f32> {
    (0..len)
        .map(|n| amplitude * (TAU * (bin * n) as f64 / FFT_SIZE as f64).sin() as f32)
        .collect()
}

/// Uniform noise in `[-amplitude, amplitude)` from a fixed-seed xorshift.
fn noise(amplitude: f32, len: usize, seed: u64) -> Vec<f32> {
    let mut state = seed;
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            amplitude * ((state >> 40) as f32 / (1u64 << 23) as f32 - 1.0)
        })
        .collect()
}

fn last_spectrum(tuner: &Tuner, signal: &[f32]) -> Result<Vec<f32>, TunerError> {
    let mut spectrum = Vec::new();
    for hop in signal.as_chunks::<HOP>().0 {
        spectrum = floats(&tuner.process(&bytes(hop))?.spectrum);
    }
    Ok(spectrum)
}

fn bin_of(hz: f32) -> f32 {
    hz * FFT_SIZE as f32 / SAMPLE_RATE as f32
}

#[test]
fn process_rejects_anything_but_one_hop() -> Result<(), TunerError> {
    let tuner = Tuner::new(config())?;
    for len in [0, HOP - 1, HOP + 1, 2 * HOP] {
        assert!(
            tuner.process(&bytes(&vec![0.0; len])).is_err(),
            "{len} samples"
        );
    }
    let mut ragged = bytes(&[0.0; HOP]);
    ragged.pop();
    assert!(tuner.process(&ragged).is_err(), "one byte short");
    Ok(())
}

#[test]
fn learn_noise_rejects_partial_samples() -> Result<(), TunerError> {
    let tuner = Tuner::new(config())?;
    assert!(tuner.learn_noise(&[0; 4 * HOP + 1]).is_err());
    Ok(())
}

#[test]
fn learn_noise_needs_at_least_one_frame() -> Result<(), TunerError> {
    let tuner = Tuner::new(config())?;
    assert!(tuner.learn_noise(&bytes(&vec![0.0; FFT_SIZE - 1])).is_err());
    assert!(tuner.learn_noise(&bytes(&vec![0.0; FFT_SIZE])).is_ok());
    assert!(
        tuner
            .learn_noise(&bytes(&noise(0.3, SAMPLE_RATE as usize, 3)))
            .is_ok()
    );
    Ok(())
}

#[test]
fn spectrum_has_one_f32_per_bin() -> Result<(), TunerError> {
    let tuner = Tuner::new(config())?;
    let result = tuner.process(&bytes(&[0.0; HOP]))?;
    assert_eq!(result.spectrum.len(), 4 * tuner.spectrum_size()? as usize);
    Ok(())
}

#[test]
fn sine_peaks_on_its_bin() -> Result<(), TunerError> {
    const BIN: usize = 100; // 586 Hz
    let tuner = Tuner::new(config())?;
    let spectrum = last_spectrum(&tuner, &sine(BIN, 0.5, FFT_SIZE))?;
    let peak = spectrum
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(bin, _)| bin);
    assert_eq!(peak, Some(BIN));
    Ok(())
}

#[test]
fn bins_outside_the_band_are_cut() -> Result<(), TunerError> {
    let tuner = Tuner::new(config())?;
    let spectrum = last_spectrum(&tuner, &noise(0.3, FFT_SIZE, 7))?;
    let cut = spectrum[0];
    // One bin of slack at each edge, so rounding of the edge bins is up to the implementation.
    let (low, high) = (bin_of(MIN_HZ), bin_of(MAX_HZ));
    for (bin, &value) in spectrum.iter().enumerate() {
        let k = bin as f32;
        if k < low - 1.0 || k > high + 1.0 {
            assert!(
                (value - cut).abs() <= 1e-6 * cut.abs().max(1.0),
                "bin {bin} outside the band is {value}, bin 0 is {cut}"
            );
        } else if k > low + 1.0 && k < high - 1.0 {
            assert!(
                value > cut,
                "bin {bin} inside the band is {value}, cut is {cut}"
            );
        }
    }
    Ok(())
}

#[test]
fn learned_noise_is_subtracted() -> Result<(), TunerError> {
    let learned = Tuner::new(config())?;
    let plain = Tuner::new(config())?;
    learned.learn_noise(&bytes(&noise(0.3, SAMPLE_RATE as usize, 1)))?;

    let live = noise(0.3, FFT_SIZE, 2);
    let learned = last_spectrum(&learned, &live)?;
    let plain = last_spectrum(&plain, &live)?;

    let band = (bin_of(MIN_HZ) as usize + 2)..(bin_of(MAX_HZ) as usize - 1);
    let mean = |s: &[f32]| s[band.clone()].iter().sum::<f32>() / band.len() as f32;
    assert!(
        mean(&learned) < mean(&plain),
        "mean in-band level {} with the profile, {} without",
        mean(&learned),
        mean(&plain)
    );
    Ok(())
}

#[test]
fn smoothing_remembers_earlier_frames() -> Result<(), TunerError> {
    const OLD: usize = 100;
    const NEW: usize = 300;
    // One hop past a full frame of NEW: the frame holds no trace of OLD any more,
    // so only the smoothing state can keep bin OLD up.
    let new = sine(NEW, 0.5, FFT_SIZE + HOP);

    let switched = Tuner::new(config())?;
    last_spectrum(&switched, &sine(OLD, 0.5, FFT_SIZE))?;
    let switched = last_spectrum(&switched, &new)?;

    let fresh = Tuner::new(config())?;
    let fresh = last_spectrum(&fresh, &new)?;

    assert!(
        switched[OLD] > fresh[OLD],
        "bin {OLD}: {} after switching, {} fresh",
        switched[OLD],
        fresh[OLD]
    );
    Ok(())
}

#[test]
fn full_scale_sine_reads_zero_dbfs() -> Result<(), TunerError> {
    const BIN: usize = 100;
    let tuner = Tuner::new(config())?;
    // 40 hops: the frame fills after 8, then smoothing settles (0.7^32 ~ 1e-5).
    let spectrum = last_spectrum(&tuner, &sine(BIN, 1.0, 40 * HOP))?;
    assert!(
        spectrum[BIN].abs() < 0.1,
        "bin {BIN} is {} dBFS, want 0",
        spectrum[BIN]
    );
    Ok(())
}

#[test]
fn cut_bins_sit_at_the_epsilon_floor() -> Result<(), TunerError> {
    let tuner = Tuner::new(config())?;
    let spectrum = last_spectrum(&tuner, &noise(0.3, FFT_SIZE, 11))?;
    let floor = 20.0 * EPSILON.log10();
    assert!(
        (spectrum[0] - floor).abs() < 1e-3,
        "bin 0 is {} dB, want {floor}",
        spectrum[0]
    );
    Ok(())
}
