# FFT and signal processing

Companion to [SPEC.md](SPEC.md). Music theory is in [MUSIC.md](MUSIC.md).
Notation: $f_s$ sample rate, $N$ frame length, $x[n]$ the PCM frame, $f_{\min}, f_{\max}$ the band-pass edges.

## 1. DFT and the hand-written FFT (background)

$$X[k] = \sum_{n=0}^{N-1} x[n]\, e^{-2\pi i kn/N}, \qquad k = 0..N-1.$$

Bin $k$ is frequency $k f_s / N$. For real input $X[N-k] = \overline{X[k]}$, so only $k \le N/2$
(Nyquist, $f_s/2$) carries information and we output $N/2$ magnitudes. Bin width $f_s/N$ is the plot resolution:

| $f_s$ | $N$ | bin width | frame length |
|---|---|---|---|
| 48000 | 2048 | 23.4 Hz | 43 ms |
| 48000 | 4096 | 11.7 Hz | 85 ms |
| 48000 | 16384 | 2.9 Hz | 341 ms |

Radix-2 butterfly, with $W_N = e^{-2\pi i/N}$ and $E, O$ the length-$N/2$ DFTs of the even and odd samples:

$$X[k] = E[k] + W_N^k O[k], \qquad X[k+N/2] = E[k] - W_N^k O[k], \qquad k < N/2.$$

Holds because $W_N^{2mk} = W_{N/2}^{mk}$, $E$ and $O$ are $N/2$-periodic, and $W_N^{k+N/2} = -W_N^k$.
Gives $O(N \log N)$.

Implementation: iterative, in place, bit-reversal permutation first, then $\log_2 N$ stages.
Twiddles $W_N^k$ for $k < N/2$ and the window are precomputed once in `Tuner::new`.

Correctness tests in `core/tests/`:

- random input against the naive $O(N^2)$ sum, max abs error below $10^{-3}$ for $N = 4096$ in `f32`;
- Parseval, $\sum |x[n]|^2 = \frac{1}{N} \sum |X[k]|^2$;
- a sine at exactly $k f_s / N$ puts energy only in bins $k$ and $N - k$.

## 2. Windowing

A finite frame is the signal times a rectangle; in frequency that is convolution with a sinc whose side
lobes are only 13 dB down, so one strong tone smears over the plot. Multiply by Hann first:

$$w[n] = \tfrac{1}{2}\left(1 - \cos \tfrac{2\pi n}{N-1}\right).$$

Side lobes drop to 31 dB, falling 18 dB per octave. The main lobe doubles in width, which is fine
because the pitch number does not come from the FFT (section 4).

## 3. Magnitude, display, peak refinement

$|X[k]| = \sqrt{\mathrm{Re}^2 + \mathrm{Im}^2}$. Plot $20 \log_{10}(|X[k]| + \varepsilon)$, since harmonics span orders of magnitude.

If a peak is ever read from the FFT, refine it with the log-magnitudes $\alpha, \beta, \gamma$ of bins $k-1, k, k+1$:

$$\delta = \frac{1}{2} \frac{\alpha - \gamma}{\alpha - 2\beta + \gamma}, \qquad f = (k + \delta)\frac{f_s}{N}.$$

The log of the Hann main lobe is close to a parabola; $\delta$ is the vertex of the parabola through the three points.

## 4. Pitch detection with YIN

The tallest FFT bin is not the pitch, for two reasons: resolution (at E2 the next semitone is 4.9 Hz away,
under one bin at $N = 4096$) and harmonics louder than the fundamental (MUSIC.md section 3).

YIN works on the raw time-domain frame. Difference function over a window $W$ (we use $W = N/2$):

$$d(\tau) = \sum_{j=0}^{W-1} \left(x[j] - x[j+\tau]\right)^2.$$

Why it is correct: if $x$ is periodic with period $T$ then $x[j] = x[j+T]$ for every $j$, so $d(T) = 0$,
while $d(\tau) > 0$ for $\tau$ not a multiple of $T$. The pitch is the **smallest** $\tau$ with $d(\tau) \approx 0$.

$d(0) = 0$ trivially, so normalise by the running mean:

$$d'(\tau) = \begin{cases} 1 & \tau = 0 \\[4pt] d(\tau) \Big/ \left(\dfrac{1}{\tau} \sum_{j=1}^{\tau} d(j)\right) & \tau > 0 \end{cases}$$

Take the first $\tau$ where $d'(\tau)$ dips below a threshold (0.1 to 0.15) and is a local minimum.
Taking the first dip rather than the global minimum is what prefers the fundamental over its multiples.
Refine $\tau$ with the parabola from section 3 on $d'(\tau-1), d'(\tau), d'(\tau+1)$. Then

$$f = \frac{f_s}{\tau}, \qquad \mathrm{confidence} = 1 - d'(\tau).$$

Search range $\tau \in [f_s / f_{\max},\ f_s / f_{\min}]$, about 10 to 960 at 48 kHz.
Cost $O(W \cdot \tau_{\max}) \approx 2 \cdot 10^6$ multiply-adds per frame, under a millisecond on a phone.
If that ever matters, $d(\tau) = r(0) + r_\tau(0) - 2r(\tau)$ with $r$ the autocorrelation, which the FFT gives in $O(N \log N)$.

Gate: if the frame RMS is below a threshold, report `pitch_hz = 0` and keep the previous display.

## 5. Noise removal: spectral subtraction

Model the mic signal as $y = s + n$ with noise $n$ stationary and uncorrelated with the sound $s$.
Then in expectation $|Y[k]|^2 \approx |S[k]|^2 + |N[k]|^2$, so the noise can be subtracted per bin.

Learning: over $M$ frames of silence, $\hat{N}[k] = \frac{1}{M} \sum_m |Y_m[k]|$.

Cleaning each live frame:

$$|\hat{S}[k]| = \max\left(|Y[k]| - \alpha \hat{N}[k],\ \beta \hat{N}[k]\right),$$

with over-subtraction $\alpha \in [1, 2]$ and floor $\beta \approx 0.01$. The floor stops bins flickering
around zero ("musical noise"). Band-pass is free here: bins outside $[f_{\min}, f_{\max}]$ are set to zero.

Temporal smoothing for the plot: $\bar{S}_t = \lambda \bar{S}_{t-1} + (1 - \lambda)\hat{S}_t$, $\lambda \approx 0.7$.

Why not SVD: the signal is one-dimensional. Low-rank tricks need a spectrogram or Hankel matrix, cost
$O(N^3)$ per update, and assume structure a tuner does not need. Spectral subtraction is $O(N)$.

## 6. Chroma vector

Fold the cleaned spectrum into 12 pitch classes, summing all octaves. With $f_k = k f_s / N$:

$$C[p] = \sum_{k \,:\, p(k) = p} |\hat{S}[k]|^2, \qquad p(k) = \mathrm{round}\!\left(69 + 12 \log_2 \frac{f_k}{440}\right) \bmod 12,$$

over bins inside the band-pass, then normalise $C$ to unit length. Template matching and the musical
reasoning are in MUSIC.md section 5.
