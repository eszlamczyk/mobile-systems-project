# Spectrum grid

Companion to [FFT.md](FFT.md). Implemented twice, keep them identical:
`android/.../audio/SpectrumGrid.kt` and `ios/MobileTuner/Audio/SpectrumGrid.swift`.

The core returns $N/2$ dB values, one per bin, linear in Hz (FFT.md section 3). The plot has a log-frequency
axis and a fixed number of points $P$ (`POINTS = 400`), so each plot frame (~30 per second) the spectrum
is reduced to $P$ values on the DSP thread before it reaches the UI.

Notation: $\Delta = f_s / N$ bin width, $r = f_{\max} / f_{\min}$, $i = 0..P-1$ the point index.

## 1. Grid

Point $i$ sits at

$$f_i = f_{\min}\, r^{\,i/(P-1)}, \qquad x_i = \frac{i}{P-1} = \frac{\log_2(f_i / f_{\min})}{\log_2 r},$$

so points are evenly spaced in octaves and $x$ is the normalised horizontal position (`xOf` / `x(ofHz:)`).
Its cell is $[f_{i-1/2}, f_{i+1/2}]$, using the same formula at half-integer $i$.

## 2. Band

Only bins inside the band carry signal; the core sets the rest to the $20\log_{10}\varepsilon$ floor.
The grid uses the same band as `dsp-core/src/state.rs`:

$$lo = \lceil f_{\min} / \Delta \rceil, \qquad hi = \min(\lfloor f_{\max} / \Delta \rfloor,\ N/2 - 1).$$

Every bin index the grid reads is clamped to $[lo, hi]$, so floor bins never leak into the plot.

## 3. Reduction

Precomputed once per $(f_s, N, f_{\min}, f_{\max})$:

$$first_i = \max(\lceil f_{i-1/2} / \Delta \rceil, lo), \qquad last_i = \min(\lfloor f_{i+1/2} / \Delta \rfloor, hi), \qquad pos_i = \mathrm{clamp}(f_i / \Delta, lo, hi).$$

Per frame, for each point:

- $first_i \le last_i$, the cell holds at least one bin centre: take the max over $first_i..last_i$.
  Max rather than mean so a narrow partial is not averaged away in the wide high-frequency cells.
- otherwise the cell is narrower than a bin: Catmull-Rom through bins $k-1, k, k+1, k+2$ with
  $k = \lfloor pos_i \rfloor$, $t = pos_i - k$:

$$y = p_1 + \tfrac{1}{2} t \big(p_2 - p_0 + t (2p_0 - 5p_1 + 4p_2 - p_3 + t (3(p_1 - p_2) + p_3 - p_0))\big).$$

Cells get narrower than a bin below $f \approx \Delta / \ln r^{1/(P-1)}$. With the defaults
($f_s = 48000$, $N = 8192$, 50 to 5000 Hz) that is about 508 Hz, the band is bins 9 to 853,
and 123 of the 400 points are interpolated.

## 4. Normalisation

$$v_i = \mathrm{clamp}\!\left(\frac{y_i - DB_{\min}}{DB_{\max} - DB_{\min}}, 0, 1\right), \qquad DB_{\min} = -100,\ DB_{\max} = 0.$$

## 5. Ghost

The target spectrum goes through the same `downsample`, recomputed only when the target note changes.
Until the core implements `target_spectrum`, `placeholderGhost` draws Gaussian-ish bumps in semitones at
each harmonic $n f_0$ below $f_{\max}$, starting at $-20$ dB and falling $20\log_{10} n$ per harmonic.
