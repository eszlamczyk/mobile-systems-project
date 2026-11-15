const THRESHOLD: f32 = 0.1;
const RMS_GATE: f32 = 0.001;
const MAX_BAND_ERROR_RATIO: f32 = 1.000_577_8;

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub(crate) fn detect(
    frame: &[f32],
    difference: &mut [f32],
    sample_rate: u32,
    min_hz: f32,
    max_hz: f32,
    rms: f32,
) -> (f32, f32) {
    if rms < RMS_GATE {
        return (0.0, 0.0);
    }
    let window = frame.len() / 2;
    let rate = sample_rate as f32;
    let first = ((rate / max_hz).floor() as usize).max(2);
    let last = ((rate / min_hz).ceil() as usize).min(window.saturating_sub(1));
    if first > last {
        return (0.0, 0.0);
    }

    difference[0] = 1.0;
    let mut running_sum = 0.0;
    for lag in 1..=last + 1 {
        let sum: f32 = frame[..window]
            .iter()
            .zip(&frame[lag..lag + window])
            .map(|(a, b)| (a - b) * (a - b))
            .sum();
        running_sum += sum;
        difference[lag] = if running_sum > 0.0 {
            sum * lag as f32 / running_sum
        } else {
            1.0
        };
    }

    let Some(mut lag) = (first..=last).find(|&lag| difference[lag] < THRESHOLD) else {
        return (0.0, 0.0);
    };
    while lag < last && difference[lag + 1] < difference[lag] {
        lag += 1;
    }
    let (left, middle, right) = (difference[lag - 1], difference[lag], difference[lag + 1]);
    if middle > left || middle > right {
        return (0.0, 0.0);
    }
    let curvature = left - 2.0 * middle + right;
    let offset = if curvature > 0.0 {
        (0.5 * (left - right) / curvature).clamp(-0.5, 0.5)
    } else {
        0.0
    };
    let frequency = rate / (lag as f32 + offset);
    if frequency < min_hz / MAX_BAND_ERROR_RATIO || frequency > max_hz * MAX_BAND_ERROR_RATIO {
        return (0.0, 0.0);
    }
    (
        frequency.clamp(min_hz, max_hz),
        (1.0 - middle).clamp(0.0, 1.0),
    )
}
