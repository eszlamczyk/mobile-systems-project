# Music for the tuner

Companion to [SPEC.md](SPEC.md). Signal processing is in [FFT.md](FFT.md).
Notation: $f_t$ target frequency, $n$ MIDI note number.

## 1. Notes, frequencies, cents

An octave doubles the frequency. Equal temperament splits the octave into 12 equal ratios $r$,
so $r^{12} = 2$ and $r = 2^{1/12} \approx 1.0595$. With A4 = 440 Hz = MIDI note 69:

$$f(n) = 440 \cdot 2^{(n-69)/12}, \qquad n(f) = 69 + 12 \log_2 \frac{f}{440}.$$

The nearest note is $\mathrm{round}(n(f))$. Offset from a target is measured in cents, 1200 per octave:

$$c = 1200 \log_2 \frac{f}{f_t}.$$

One semitone is $1200 / 12 = 100$ cents, so the offset from the **nearest** note is in $[-50, 50]$.
The offset from a user-chosen target is unbounded (A4 played against a B4 target is $-200$); the colour
map in section 2 saturates at 50.

| Note | MIDI | Hz |
|---|---|---|
| D2 | 38 | 73.42 |
| E2 | 40 | 82.41 |
| A2 | 45 | 110.00 |
| D3 | 50 | 146.83 |
| G3 | 55 | 196.00 |
| B3 | 59 | 246.94 |
| E4 | 64 | 329.63 |
| A4 | 69 | 440.00 |
| B4 | 71 | 493.88 |

E2 to E4 are standard guitar tuning. D2 replaces E2 in Drop D and DADGAD. The other tunings are listed in SPEC.md (Tuning presets),
and any note not in this table follows from $f(n)$.

## 2. Colour

Green at $|c| < 2$, red at $|c| \ge 50$, linear interpolation between. One cent is too strict to hit
reliably by hand. Five cents lets through detuning that is audible as beating when two strings sound together.
Fifty is halfway to the next note.

## 3. Harmonics

A played note is not one frequency. A string fixed at both ends only supports standing waves whose
half-wavelength divides the string length, so the allowed frequencies are $k f_1$ for $k = 1, 2, 3, \dots$
The lowest, $f_1$, is the pitch we hear. The rest are the harmonics and their relative strengths are the timbre.

Consequences for the app:

- The live plot of a single note is a comb of peaks at $f_1, 2f_1, 3f_1, \dots$
- The loudest peak is often not $f_1$. Guitar low strings and many voices have a stronger 2nd or 3rd
  harmonic, which is why pitch is not read from the tallest FFT bin (see FFT.md, YIN).

## 4. The target ghost

The grey ghost is the comb a perfectly tuned note would show:

$$G(f) = \sum_{k=1}^{K} \frac{1}{k}\, H(f - k f_t), \qquad K = 8,$$

where $H$ is the shape of one FFT peak (the Hann main lobe, FFT.md section 2) so the ghost has the same
width as real peaks. The $1/k$ roll-off is a generic plucked-string profile and is cosmetic.
