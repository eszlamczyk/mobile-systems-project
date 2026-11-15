#![allow(missing_docs)]
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub fn midi_to_hz(midi_note: i32) -> f32 {
    440.0 * 2.0_f32.powf((midi_note as f32 - 69.0) / 12.0)
}

#[allow(clippy::cast_possible_truncation)]
#[must_use]
pub fn nearest_midi(pitch_hz: f32) -> i32 {
    (69.0 + 12.0 * (pitch_hz.log2() - 440.0_f32.log2())).round() as i32
}

#[must_use]
pub fn cents(pitch_hz: f32, target_hz: f32) -> f32 {
    1200.0 * (pitch_hz.log2() - target_hz.log2())
}
