//! Musical note frequencies, rounding, and cents offsets.

use dsp_core::music::{cents, midi_to_hz, nearest_midi};

#[test]
fn known_notes_have_the_expected_frequencies() {
    for (midi, expected) in [
        (0, 8.175_799),
        (28, 41.203_445),
        (40, 82.406_89),
        (45, 110.0),
        (64, 329.627_56),
        (69, 440.0),
        (127, 12_543.854),
    ] {
        let actual = midi_to_hz(midi);
        assert!(
            (actual - expected).abs() / expected < 1e-6,
            "MIDI {midi}: expected {expected}, got {actual}"
        );
    }
}

#[test]
fn all_midi_notes_round_trip_and_increase_in_pitch() {
    let mut previous = 0.0;
    for midi in 0..=127 {
        let hz = midi_to_hz(midi);
        assert!(hz.is_finite() && hz > previous, "MIDI {midi}: {hz}");
        assert_eq!(nearest_midi(hz), midi);
        previous = hz;
    }
}

#[test]
fn nearest_note_rounds_on_both_sides_of_a_semitone_midpoint() {
    for (hz, expected) in [
        (452.631_47, 69),
        (453.154_66, 70),
        (427.721_2, 69),
        (427.227_42, 68),
    ] {
        assert_eq!(nearest_midi(hz), expected, "{hz} Hz");
    }
}

#[test]
fn cents_are_signed_and_not_clamped_to_the_nearest_note() {
    for (pitch, target, expected) in [
        (440.0, 440.0, 0.0),
        (441.0, 440.0, 3.930_158),
        (440.0, 441.0, -3.930_158),
        (880.0, 440.0, 1200.0),
        (220.0, 440.0, -1200.0),
        (440.0, 493.883_3, -200.0),
    ] {
        let actual = cents(pitch, target);
        assert!(
            (actual - expected).abs() < 0.001,
            "{pitch} Hz against {target} Hz: expected {expected}, got {actual}"
        );
    }
}

#[test]
fn finite_positive_extremes_do_not_overflow_or_underflow_a_ratio() {
    assert_eq!(nearest_midi(f32::from_bits(1)), -1824);
    assert_eq!(nearest_midi(f32::MAX), 1500);
    assert!((cents(f32::MAX, f32::MIN_POSITIVE) - 304_800.0).abs() < 0.1);
    assert!((cents(f32::from_bits(1), f32::MAX) + 332_400.0).abs() < 0.1);
}
