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

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
pub(crate) fn target_spectrum(config: &crate::TunerConfig, midi_note: i32) -> Vec<f32> {
    let bins = (config.fft_size / 2) as usize;
    let mut spectrum = vec![0.0; bins];
    let size = f64::from(config.fft_size);
    let band = crate::state::band(config, bins);
    let fundamental = f64::from(midi_to_hz(midi_note)) * size / f64::from(config.sample_rate);
    if config.fft_size > 2 {
        for harmonic in 1..=8 {
            let centre = fundamental * f64::from(harmonic);
            if centre >= size / 2.0 {
                break;
            }
            for bin in band.clone() {
                let offset = (bin as f64 - centre) / size;
                if (offset * (size - 1.0)).abs() < 2.0 {
                    let shift = 1.0 / (size - 1.0);
                    let lobe = dirichlet(offset, size)
                        + dirichlet(offset - shift, size).midpoint(dirichlet(offset + shift, size));
                    spectrum[bin] += (lobe.abs() / f64::from(harmonic)) as f32;
                }
            }
        }
    }
    for magnitude in &mut spectrum {
        *magnitude = 20.0 * (*magnitude + config.epsilon).log10();
    }
    spectrum
}

fn dirichlet(frequency: f64, size: f64) -> f64 {
    sinc(size * frequency) / sinc(frequency)
}

fn sinc(value: f64) -> f64 {
    if value.abs() < f64::EPSILON {
        1.0
    } else {
        let angle = std::f64::consts::PI * value;
        angle.sin() / angle
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn target_spectrum_does_not_wait_for_the_dsp_mutex() -> Result<(), crate::TunerError> {
        let tuner = crate::Tuner::new(crate::TunerConfig {
            sample_rate: 48_000,
            fft_size: 8192,
            hop_size: 1024,
            min_hz: 50.0,
            max_hz: 5000.0,
            alpha: 1.5,
            beta: 0.01,
            lambda: 0.7,
            epsilon: 1e-6,
        })?;
        let guard = tuner
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let target = std::sync::Arc::clone(&tuner);
        let (sender, receiver) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || sender.send(target.target_spectrum()).is_ok());
        let result = receiver.recv_timeout(std::time::Duration::from_secs(2));
        drop(guard);
        assert!(matches!(worker.join(), Ok(true)));
        assert!(matches!(result, Ok(Ok(_))));
        Ok(())
    }
}
