# Music for the tuner

Companion to [SPEC.md](SPEC.md). Signal processing is in [FFT.md](FFT.md).
Notation: $f_t$ target frequency, $n$ MIDI note number.

## 1. Notes, frequencies, cents

An octave doubles the frequency. Equal temperament splits the octave into 12 equal ratios $r$,
so $r^{12} = 2$ and $r = 2^{1/12} \approx 1.0595$. With A4 = 440 Hz = MIDI note 69:

$$f(n) = 440 \cdot 2^{(n-69)/12}, \qquad n(f) = 69 + 12 \log_2 \frac{f}{440}.$$

The nearest note is $\mathrm{round}(n(f))$. Offset from a target is measured in cents, 1200 per octave:

$$c = 1200 \log_2 \frac{f}{f_t}.$$

One semitone is $1200 / 12 = 100$ cents, so after rounding to the nearest note $c \in [-50, 50]$.

| Note | MIDI | Hz |
|---|---|---|
| E2 | 40 | 82.41 |
| A2 | 45 | 110.00 |
| D3 | 50 | 146.83 |
| G3 | 55 | 196.00 |
| B3 | 59 | 246.94 |
| E4 | 64 | 329.63 |
| A4 | 69 | 440.00 |
| B4 | 71 | 493.88 |

The first six rows are standard guitar tuning, the preset in the nice-to-have list.

## 2. Colour

Green at $|c| < 5$, red at $|c| \ge 50$, linear interpolation between. Five cents is roughly the
limit of what a listener notices; fifty is halfway to the next note.

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

## 5. Chords

A chord is several notes sounded together. Intervals are counted in semitones from the root:

| Chord | Semitones | Example |
|---|---|---|
| major triad | $\{0, 4, 7\}$ | C E G |
| minor triad | $\{0, 3, 7\}$ | C E♭ G |

A major triad's fundamentals sit near the ratio $4 : 5 : 6$. The spectrum of a chord is the sum of
each note's comb, and the combs overlap: the 3rd harmonic of C (G5) lands on the pitch class of the
chord's fifth, the 2nd and 4th harmonics land back on C.

A **pitch class** is a note name with the octave ignored, 12 of them. Folding the spectrum into 12
pitch-class bins gives the chroma vector $C \in \mathbb{R}^{12}$ (computed in FFT.md section 6).
A chord template is a 0/1 vector $t$ over pitch classes; each root is a rotation of the table above. Score:

$$\mathrm{score}(C, t) = \frac{\langle C, t \rangle}{\|C\|\,\|t\|}.$$

Report the best root and kind, and show the score as a match percentage.

Why it works: harmonics 2, 4, 8 of any note fold onto the note's own pitch class, and harmonic 3 is a
fifth, which both triads contain. Why it is limited: harmonic 5 is a major third, which pushes minor
chords toward major. Weighting lower bins more reduces this but does not remove it. Individual strings
of a strummed chord cannot be tuned to the cent from a chroma vector; chord mode reports a match, not cents.
