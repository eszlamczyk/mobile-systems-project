#![allow(
    missing_docs,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use dsp_core::{Tuner, TunerConfig, TunerError};
use std::f64::consts::TAU;

fn config() -> TunerConfig {
    TunerConfig {
        sample_rate: 45_056,
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

#[test]
fn eight_harmonics_have_one_over_k_levels_on_the_live_db_grid() -> Result<(), TunerError> {
    let tuner = Tuner::new(config())?;
    let spectrum = tuner.target_spectrum()?;
    assert_eq!(spectrum.len(), tuner.spectrum_size()? as usize);
    for harmonic in 1..=8 {
        let bin = 80 * harmonic;
        let expected = -20.0 * (harmonic as f32).log10();
        assert!((spectrum[bin] - expected).abs() < 0.01);
        assert!(spectrum[bin] > spectrum[bin - 1]);
        assert!(spectrum[bin] > spectrum[bin + 1]);
        assert!((spectrum[bin - 1] - spectrum[bin] + 6.02).abs() < 0.01);
        assert!((spectrum[bin + 1] - spectrum[bin] + 6.02).abs() < 0.01);
    }
    assert!((spectrum[720] + 120.0).abs() < 0.001);
    Ok(())
}

#[test]
fn fractional_bin_lobes_match_a_direct_hann_dft() -> Result<(), TunerError> {
    for (sample_rate, fft_size, midi, hz) in [
        (48_000, 8192, 69, 440.0),
        (44_100, 4096, 45, 110.0),
        (16_000, 256, 40, 82.406_889_228_217_5),
    ] {
        let config = TunerConfig {
            sample_rate,
            fft_size,
            hop_size: fft_size,
            ..config()
        };
        let tuner = Tuner::new(config)?;
        tuner.set_target(midi)?;
        let spectrum = tuner.target_spectrum()?;
        let size = f64::from(fft_size);
        for (bin, &actual) in spectrum.iter().enumerate() {
            let frequency = bin as f64 * f64::from(sample_rate) / size;
            if !(50.0..=5000.0).contains(&frequency) {
                continue;
            }
            let mut magnitude = 0.0;
            for harmonic in 1..=8 {
                let centre = hz * f64::from(harmonic);
                let offset = (frequency - centre) / f64::from(sample_rate);
                if centre >= f64::from(sample_rate) / 2.0 || (offset * (size - 1.0)).abs() >= 2.0 {
                    continue;
                }
                let (mut re, mut im) = (0.0, 0.0);
                for sample in 0..fft_size {
                    let window = 0.5 * (1.0 - (TAU * f64::from(sample) / (size - 1.0)).cos());
                    let angle = TAU * offset * f64::from(sample);
                    re += window * angle.cos();
                    im += window * angle.sin();
                }
                magnitude += 2.0 * re.hypot(im) / size / f64::from(harmonic);
            }
            let expected = 20.0 * (magnitude + 1e-6).log10();
            assert!(
                (f64::from(actual) - expected).abs() < 0.02,
                "{sample_rate} Hz, N={fft_size}, MIDI {midi}, bin {bin}: {actual} vs {expected}"
            );
        }
    }
    Ok(())
}

#[test]
fn band_edges_and_empty_bands_use_the_configured_floor() -> Result<(), TunerError> {
    for (min_hz, max_hz) in [(440.0, 880.0), (441.0, 879.0), (30_000.0, 40_000.0)] {
        let tuner = Tuner::new(TunerConfig {
            min_hz,
            max_hz,
            epsilon: 1e-4,
            ..config()
        })?;
        for (bin, level) in tuner.target_spectrum()?.into_iter().enumerate() {
            let hz = bin as f32 * 5.5;
            assert!(level.is_finite());
            if hz < min_hz || hz > max_hz {
                assert!((level + 80.0).abs() < 0.001, "bin {bin}: {level}");
            } else if bin == 80 || bin == 160 {
                assert!(level > -7.0);
            }
        }
    }
    Ok(())
}

#[test]
fn target_changes_move_the_comb_and_invalid_targets_preserve_it() -> Result<(), TunerError> {
    let tuner = Tuner::new(config())?;
    let a4 = tuner.target_spectrum()?;
    tuner.set_target(81)?;
    let a5 = tuner.target_spectrum()?;
    assert!(a4[80] > -0.01 && a5[80] < -119.0);
    assert!(a5[160] > -0.01);
    assert!(tuner.set_target(128).is_err());
    assert_eq!(a5, tuner.target_spectrum()?);
    tuner.set_target(69)?;
    assert_eq!(a4, tuner.target_spectrum()?);
    Ok(())
}

#[test]
fn harmonics_above_nyquist_do_not_alias_into_the_ghost() -> Result<(), TunerError> {
    let tuner = Tuner::new(TunerConfig {
        sample_rate: 4096,
        fft_size: 4096,
        max_hz: 5000.0,
        ..config()
    })?;
    let spectrum = tuner.target_spectrum()?;
    assert!(spectrum[1760] > -12.1);
    assert!((spectrum[1896] + 120.0).abs() < 0.001);
    tuner.set_target(127)?;
    assert!(
        tuner
            .target_spectrum()?
            .iter()
            .all(|db| (*db + 120.0).abs() < 0.001)
    );
    Ok(())
}

#[test]
fn all_midi_notes_and_small_frames_produce_finite_spectra() -> Result<(), TunerError> {
    for fft_size in [2, 4, 256, 8192] {
        let tuner = Tuner::new(TunerConfig {
            fft_size,
            hop_size: fft_size,
            min_hz: 0.0,
            max_hz: 30_000.0,
            ..config()
        })?;
        for midi in 0..=127 {
            tuner.set_target(midi)?;
            let spectrum = tuner.target_spectrum()?;
            assert_eq!(spectrum.len(), fft_size as usize / 2);
            assert!(spectrum.iter().all(|db| db.is_finite()));
        }
    }
    Ok(())
}

#[test]
fn audio_and_noise_learning_do_not_change_the_target_spectrum() -> Result<(), TunerError> {
    let tuner = Tuner::new(config())?;
    let before = tuner.target_spectrum()?;
    let pcm: Vec<u8> = (0..8192)
        .flat_map(|sample| {
            (0.5 * (TAU * 110.0 * f64::from(sample) / 45_056.0).sin() as f32).to_ne_bytes()
        })
        .collect();
    tuner.learn_noise(&pcm)?;
    for hop in pcm.as_chunks::<{ 1024 * 4 }>().0 {
        tuner.process(hop)?;
    }
    assert_eq!(before, tuner.target_spectrum()?);
    Ok(())
}
