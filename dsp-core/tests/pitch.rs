//! Pitch detection through the public API, using independent f64 synthesis.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use std::f64::consts::TAU;

use dsp_core::{Tuner, TunerConfig, TunerError, TunerResult};

fn config(sample_rate: u32) -> TunerConfig {
    TunerConfig {
        sample_rate,
        fft_size: 8192,
        hop_size: 8192,
        min_hz: 30.0,
        max_hz: 5000.0,
        alpha: 1.5,
        beta: 0.01,
        lambda: 0.7,
        epsilon: 1e-6,
    }
}

fn frequency(midi: i32) -> f64 {
    440.0 * 2.0_f64.powf(f64::from(midi - 69) / 12.0)
}

fn bytes(samples: impl Iterator<Item = f32>) -> Vec<u8> {
    samples.flat_map(f32::to_ne_bytes).collect()
}

fn sine_hop(start: usize, len: usize, rate: u32, hz: f64, second: bool) -> Vec<u8> {
    bytes((start..start + len).map(|n| {
        let phase = TAU * hz * n as f64 / f64::from(rate);
        if second {
            (0.2 * phase.sin() + 0.7 * (2.0 * phase).sin()) as f32
        } else {
            (0.5 * phase.sin()) as f32
        }
    }))
}

fn assert_pitch(result: &TunerResult, expected_hz: f64) {
    let error = 1200.0 * (f64::from(result.pitch_hz) / expected_hz).log2();
    assert!(
        error.abs() <= 1.0,
        "expected {expected_hz} Hz, got {} ({error} cents)",
        result.pitch_hz
    );
    assert!((0.9..=1.0).contains(&result.confidence));
}

#[test]
fn every_semitone_e2_to_e4_is_within_one_cent() -> Result<(), TunerError> {
    for rate in [44_100, 48_000] {
        let tuner = Tuner::new(config(rate))?;
        for midi in 40..=64 {
            let hz = frequency(midi);
            let result = tuner.process(&sine_hop(0, 8192, rate, hz, false))?;
            assert_pitch(&result, hz);
            assert_eq!(result.nearest_midi, midi);
        }
    }
    Ok(())
}

#[test]
fn louder_second_harmonic_keeps_the_fundamental() -> Result<(), TunerError> {
    let tuner = Tuner::new(config(48_000))?;
    for midi in [28, 35, 40, 45, 52, 64] {
        let hz = frequency(midi);
        let result = tuner.process(&sine_hop(0, 8192, 48_000, hz, true))?;
        assert_pitch(&result, hz);
        assert_eq!(result.nearest_midi, midi);
    }
    Ok(())
}

#[test]
fn default_hops_report_signed_detuning_against_the_selected_target() -> Result<(), TunerError> {
    let mut cfg = config(48_000);
    cfg.hop_size = 1024;
    let tuner = Tuner::new(cfg)?;
    for cents in [-50.0, -20.0, -5.0, 5.0, 20.0, 50.0] {
        tuner.set_target(40)?;
        let hz = frequency(40) * 2.0_f64.powf(cents / 1200.0);
        for start in (0..8192).step_by(1024) {
            let result = tuner.process(&sine_hop(start, 1024, 48_000, hz, false))?;
            if start == 7168 {
                assert_pitch(&result, hz);
                assert!((f64::from(result.cents) - cents).abs() <= 1.0);
            }
        }
    }
    // Cents are against the chosen target, rather than the nearest note.
    tuner.set_target(52)?;
    let result = tuner.process(&sine_hop(
        8192,
        1024,
        48_000,
        frequency(40) * 2.0_f64.powf(50.0 / 1200.0),
        false,
    ))?;
    assert!((result.cents + 1150.0).abs() <= 1.0);
    Ok(())
}

#[test]
fn silence_and_sub_gate_input_clear_pitch_even_after_a_note() -> Result<(), TunerError> {
    let mut cfg = config(48_000);
    cfg.hop_size = 1024;
    let tuner = Tuner::new(cfg)?;
    for start in (0..8192).step_by(1024) {
        tuner.process(&sine_hop(start, 1024, 48_000, 110.0, false))?;
    }
    for amplitude in [0.0, 0.0001] {
        let result =
            tuner.process(&bytes((0..1024).map(|n| {
                amplitude * (TAU * 110.0 * f64::from(n) / 48_000.0).sin() as f32
            })))?;
        assert!(result.pitch_hz.abs() < f32::EPSILON);
        assert!(result.confidence.abs() < f32::EPSILON);
        assert!(result.cents.abs() < f32::EPSILON);
        assert_eq!(result.nearest_midi, 0);
    }
    Ok(())
}

#[test]
fn constant_input_and_unsearchable_bands_do_not_report_a_note() -> Result<(), TunerError> {
    for (size, min_hz, max_hz) in [
        (8192, 0.0, 5000.0),
        (2, 30.0, 5000.0),
        (8192, 30_000.0, 40_000.0),
    ] {
        let mut cfg = config(48_000);
        cfg.fft_size = size;
        cfg.hop_size = size;
        cfg.min_hz = min_hz;
        cfg.max_hz = max_hz;
        let tuner = Tuner::new(cfg)?;
        let result = tuner.process(&bytes((0..size).map(|_| 0.1)))?;
        assert!(result.pitch_hz.abs() < f32::EPSILON);
        assert!(result.confidence.abs() < f32::EPSILON);
    }
    Ok(())
}

#[test]
fn bass_e1_is_inside_the_analysis_range() -> Result<(), TunerError> {
    for rate in [44_100, 48_000] {
        let tuner = Tuner::new(config(rate))?;
        tuner.set_target(28)?;
        let result = tuner.process(&sine_hop(0, 8192, rate, frequency(28), false))?;
        assert_pitch(&result, frequency(28));
        assert_eq!(result.nearest_midi, 28);
        assert!(result.cents.abs() <= 1.0);
    }
    Ok(())
}

#[test]
fn nonperiodic_noise_and_pitch_below_the_band_do_not_report_a_note() -> Result<(), TunerError> {
    let mut cfg = config(48_000);
    cfg.min_hz = 100.0;
    cfg.max_hz = 200.0;
    let tuner = Tuner::new(cfg)?;
    let mut seed = 1_u32;
    let noise = bytes((0..8192).map(|_| {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        (f64::from(seed) / f64::from(u32::MAX) - 0.5) as f32
    }));
    for pcm in [noise, sine_hop(0, 8192, 48_000, frequency(40), false)] {
        let result = tuner.process(&pcm)?;
        assert!(result.pitch_hz.abs() < f32::EPSILON);
        assert!(result.confidence.abs() < f32::EPSILON);
    }
    Ok(())
}

#[test]
fn exact_band_edges_remain_detectable() -> Result<(), TunerError> {
    for hz in [82.406_89, 110.0, 329.627_56, 440.0] {
        for lower in [false, true] {
            let mut cfg = config(48_000);
            if lower {
                cfg.min_hz = hz;
            } else {
                cfg.max_hz = hz;
            }
            let band = cfg.min_hz..=cfg.max_hz;
            let tuner = Tuner::new(cfg)?;
            let result = tuner.process(&sine_hop(0, 8192, 48_000, f64::from(hz), false))?;
            assert_pitch(&result, f64::from(hz));
            assert!(band.contains(&result.pitch_hz));
            let detune = if lower { -5.0 } else { 5.0 };
            let outside = f64::from(hz) * 2.0_f64.powf(detune / 1200.0);
            let result = tuner.process(&sine_hop(0, 8192, 48_000, outside, false))?;
            assert!(
                result.pitch_hz.abs() < f32::EPSILON,
                "outside band: {outside} Hz"
            );
        }
    }
    Ok(())
}
