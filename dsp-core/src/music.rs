//! Equal-tempered note math at A4 = 440 Hz (docs/MUSIC.md section 1).

/// Frequency in Hz of a MIDI note, with A4 (69) at 440 Hz.
///
/// `midi_note` must be in 0..=127, as validated by `Tuner::set_target`.
#[allow(clippy::cast_precision_loss)] // MIDI 0..=127 converts to f32 exactly.
#[must_use]
pub fn midi_to_hz(midi_note: i32) -> f32 {
    440.0 * 2.0_f32.powf((midi_note as f32 - 69.0) / 12.0)
}

/// MIDI note nearest to a finite, positive frequency in Hz.
///
/// Callers must gate silence and failed detections before calling this helper.
/// The result is not clamped to 0..=127; it describes the detected pitch.
#[allow(clippy::cast_possible_truncation)] // Round before converting to an integer.
#[must_use]
pub fn nearest_midi(pitch_hz: f32) -> i32 {
    (69.0 + 12.0 * (pitch_hz.log2() - 440.0_f32.log2())).round() as i32
}

/// Signed cents from `target_hz` to `pitch_hz`, both finite and positive.
///
/// Positive means sharp and negative means flat. The result is unbounded;
/// saturating the display color is the UI's responsibility.
#[must_use]
pub fn cents(pitch_hz: f32, target_hz: f32) -> f32 {
    1200.0 * (pitch_hz.log2() - target_hz.log2())
}

#[cfg(test)]
mod tests {
    use super::{cents, midi_to_hz, nearest_midi};

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
        // A4 + 49 cents, A4 + 51 cents, A4 - 49 cents, A4 - 51 cents.
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
}
