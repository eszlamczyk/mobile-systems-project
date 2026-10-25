//! Config validation and setters of the public `Tuner` API.

use dsp_core::{Tuner, TunerConfig};

fn config() -> TunerConfig {
    TunerConfig {
        sample_rate: 48_000,
        fft_size: 8192,
        hop_size: 1024,
        min_hz: 50.0,
        max_hz: 5000.0,
    }
}

#[test]
fn defaults_are_accepted() {
    let tuner = Tuner::new(config());
    assert!(tuner.is_ok());
}

#[test]
fn spectrum_size_is_half_the_fft() {
    let config = config();
    let expected = config.fft_size / 2;
    let Ok(tuner) = Tuner::new(config) else {
        unreachable!("default config is valid");
    };
    assert!(matches!(tuner.spectrum_size(), Ok(size) if size == expected));
}

#[test]
fn zero_sample_rate_is_rejected() {
    let bad = TunerConfig {
        sample_rate: 0,
        ..config()
    };
    assert!(Tuner::new(bad).is_err());
}

#[test]
fn fft_size_must_be_a_power_of_two_of_at_least_two() {
    for fft_size in [0, 1, 1000, 8191] {
        let bad = TunerConfig {
            fft_size,
            hop_size: 1,
            ..config()
        };
        assert!(Tuner::new(bad).is_err(), "fft_size {fft_size}");
    }
}

#[test]
fn hop_size_must_be_in_one_to_fft_size() {
    for hop_size in [0, 8193, 16_384] {
        let bad = TunerConfig {
            hop_size,
            ..config()
        };
        assert!(Tuner::new(bad).is_err(), "hop_size {hop_size}");
    }
    for hop_size in [1, 8192] {
        let good = TunerConfig {
            hop_size,
            ..config()
        };
        assert!(Tuner::new(good).is_ok(), "hop_size {hop_size}");
    }
}

#[test]
fn band_must_be_finite_and_ordered() {
    let bad_bands = [
        (100.0, 100.0),
        (200.0, 100.0),
        (-1.0, 100.0),
        (f32::NAN, 100.0),
        (50.0, f32::NAN),
        (50.0, f32::INFINITY),
        (f32::INFINITY, 100.0),
    ];
    for (min_hz, max_hz) in bad_bands {
        let bad = TunerConfig {
            min_hz,
            max_hz,
            ..config()
        };
        assert!(Tuner::new(bad).is_err(), "band {min_hz}..{max_hz}");
    }
}

#[test]
fn set_target_accepts_only_midi_range() {
    let Ok(tuner) = Tuner::new(config()) else {
        unreachable!("default config is valid");
    };
    for note in [0, 69, 127] {
        assert!(tuner.set_target(note).is_ok(), "note {note}");
    }
    for note in [-1, 128, i32::MIN, i32::MAX] {
        assert!(tuner.set_target(note).is_err(), "note {note}");
    }
}
