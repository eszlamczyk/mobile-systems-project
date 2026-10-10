# Math behind the tuner

Companion to [SPEC.md](SPEC.md). Each section states what we compute and why it is correct.
Notation: $f_s$ sample rate, $N$ frame length, $x[n]$ the PCM frame, $f_t$ the target frequency.

## 1. Notes, frequencies, cents

An octave doubles the frequency. Equal temperament splits the octave into 12 equal ratios $r$,
so $r^{12} = 2$ and $r = 2^{1/12} \approx 1.0595$. With A4 = 440 Hz = MIDI note 69:

$$f(n) = 440 \cdot 2^{(n-69)/12}, \qquad n(f) = 69 + 12 \log_2 \frac{f}{440}.$$

The nearest note is $\mathrm{round}(n(f))$. Offset from a target is measured in cents, 1200 per octave:

$$c = 1200 \log_2 \frac{f}{f_t}.$$

One semitone is $1200 / 12 = 100$ cents, so after rounding to the nearest note $c \in [-50, 50]$.
Colour: green at $|c| < 5$, red at $|c| \ge 50$, linear between.

| Note | MIDI | Hz |
|---|---|---|
| E2 | 40 | 82.41 |
| A4 | 69 | 440.00 |
| B4 | 71 | 493.88 |
| E4 | 64 | 329.63 |

## 2. DFT and the hand-written FFT

### Definition

$$X[k] = \sum_{n=0}^{N-1} x[n]\, e^{-2\pi i kn/N}, \qquad k = 0..N-1.$$

Bin $k$ corresponds to frequency $k f_s / N$. For real input $X[N-k] = \overline{X[k]}$,
so only $k \le N/2$ (the Nyquist frequency $f_s/2$) carries information. We output $N/2$ magnitudes.

Bin width $f_s / N$ is the frequency resolution of the plot:

| $f_s$ | $N$ | bin width | frame length |
|---|---|---|---|
| 48000 | 2048 | 23.4 Hz | 43 ms |
| 48000 | 4096 | 11.7 Hz | 85 ms |
| 48000 | 16384 | 2.9 Hz | 341 ms |

### Why Cooley-Tukey radix-2 is correct

Let $W_N = e^{-2\pi i/N}$. Split the sum into even and odd $n$:

$$X[k] = \underbrace{\sum_{m=0}^{N/2-1} x[2m] W_{N/2}^{mk}}_{E[k]} + W_N^k \underbrace{\sum_{m=0}^{N/2-1} x[2m+1] W_{N/2}^{mk}}_{O[k]},$$

using $W_N^{2mk} = W_{N/2}^{mk}$. $E$ and $O$ are DFTs of length $N/2$, hence periodic with period $N/2$.
Together with $W_N^{k+N/2} = -W_N^k$ this gives, for $k < N/2$:

$$X[k] = E[k] + W_N^k O[k], \qquad X[k+N/2] = E[k] - W_N^k O[k].$$

That is the butterfly. Recursing gives $T(N) = 2T(N/2) + O(N) = O(N \log N)$ instead of $O(N^2)$.

Implementation plan: iterative, in place. Permute input by bit-reversed index (that is the order the
recursion would leave the leaves in), then run $\log_2 N$ stages of butterflies with span $2, 4, \dots, N$.
Twiddles $W_N^k$ for $k < N/2$ are precomputed once in `Tuner::new`.

Correctness tests: compare against the naive $O(N^2)$ sum on random input, max abs error below $10^{-3}$
for $N = 4096$ in float; Parseval $\sum |x[n]|^2 = \frac{1}{N}\sum |X[k]|^2$; a pure sine of frequency
$k f_s / N$ must produce energy only in bins $k$ and $N-k$.

### Windowing

A finite frame is the signal multiplied by a rectangle. In frequency that is convolution with a sinc,
whose side lobes are only 13 dB down, so one strong tone smears across the whole plot (leakage).
We multiply by a Hann window first:

$$w[n] = \tfrac{1}{2}\left(1 - \cos \tfrac{2\pi n}{N-1}\right).$$

Side lobes drop to 31 dB and fall off at 18 dB per octave. The cost is a main lobe twice as wide,
which is fine because the pitch number does not come from the FFT (section 3).

### Magnitude and display

$|X[k]| = \sqrt{\mathrm{Re}^2 + \mathrm{Im}^2}$. The plot uses $20 \log_{10}(|X[k]| + \varepsilon)$ because
harmonics span several orders of magnitude.

### Peak refinement

If a peak is ever read from the FFT, interpolate with the log-magnitudes $\alpha, \beta, \gamma$ of bins $k-1, k, k+1$:

$$\delta = \frac{1}{2} \frac{\alpha - \gamma}{\alpha - 2\beta + \gamma}, \qquad f = (k + \delta)\frac{f_s}{N}.$$

Why: the log of the Hann main lobe is close to a parabola, and $\delta$ is the vertex of the parabola through the three points.

## 3. Pitch detection with YIN

Two reasons the loudest FFT bin is not the pitch:

1. Resolution. At E2 the next semitone is 4.9 Hz away, below one bin at $N = 4096$.
2. Harmonics. Guitar low strings and many voices have a 2nd or 3rd harmonic louder than the fundamental, giving octave errors.

YIN works in the time domain on the raw frame. Difference function over a window $W$ (we use $W = N/2$):

$$d(\tau) = \sum_{j=0}^{W-1} \left(x[j] - x[j+\tau]\right)^2.$$

Why it is correct: if $x$ is periodic with period $T$ then $x[j] = x[j+T]$ for all $j$, so $d(T) = 0$,
and $d(\tau) > 0$ for $\tau$ that is not a multiple of $T$. The pitch is the **smallest** $\tau$ with $d(\tau) \approx 0$.

$d(0) = 0$ trivially, so normalise by the running mean:

$$d'(\tau) = \begin{cases} 1 & \tau = 0 \\ d(\tau) \Big/ \left(\frac{1}{\tau}\sum_{j=1}^{\tau} d(j)\right) & \tau > 0 \end{cases}$$

Then take the first $\tau$ where $d'(\tau)$ dips below a threshold (0.1 to 0.15) and is a local minimum.
Taking the first dip rather than the global minimum is what prefers the fundamental over its multiples.
Refine $\tau$ with the parabola from section 2 applied to $d'(\tau-1), d'(\tau), d'(\tau+1)$. Then

$$f = \frac{f_s}{\tau}, \qquad \text{confidence} = 1 - d'(\tau).$$

Search range: $\tau \in [f_s / \text{max\_hz},\ f_s / \text{min\_hz}]$, about 10 to 960 at 48 kHz.
Cost is $O(W \cdot \tau_{\max}) \approx 2 \cdot 10^6$ multiply-adds per frame, well under a millisecond on a phone.
If it ever matters, $d(\tau) = r(0) + r_\tau(0) - 2r(\tau)$ with $r$ the autocorrelation, which the FFT computes in $O(N \log N)$.

Gate: if the frame RMS is below a threshold, report `pitch_hz = 0` and keep the previous display.

## 4. Noise removal: spectral subtraction

Model the mic signal as $y = s + n$ with noise $n$ stationary and uncorrelated with the sound $s$.
Then in expectation $|Y[k]|^2 \approx |S[k]|^2 + |N[k]|^2$, so the noise can be subtracted per bin.

Learning: over $M$ frames of silence, $\hat{N}[k] = \frac{1}{M}\sum_m |Y_m[k]|$.

Cleaning every live frame:

$$|\hat{S}[k]| = \max\left(|Y[k]| - \alpha \hat{N}[k],\ \beta \hat{N}[k]\right),$$

with over-subtraction $\alpha \in [1, 2]$ and floor $\beta \approx 0.01$. The floor prevents bins from
flickering around zero ("musical noise"). The band-pass is free here: bins outside $[\text{min\_hz}, \text{max\_hz}]$ are set to zero.

Temporal smoothing for the plot: $\bar{S}_t = \lambda \bar{S}_{t-1} + (1-\lambda) \hat{S}_t$, $\lambda \approx 0.7$.

Why SVD is not used: the signal is one-dimensional. Low-rank tricks need a spectrogram or Hankel matrix,
cost $O(N^3)$ per update, and assume structure a tuner does not need. Spectral subtraction is $O(N)$.

## 5. The target ghost

A played note at $f_t$ shows harmonics at $k f_t$. The ghost is a comb:

$$G(f) = \sum_{k=1}^{K} \frac{1}{k}\, H(f - k f_t), \quad K = 8,$$

where $H$ is the Hann main-lobe shape from section 2, so the ghost has the same width as real peaks.
The $1/k$ decay is a generic plucked-string roll-off; it is cosmetic.

## 6. Chords: chroma and templates

Fold the cleaned spectrum into 12 pitch classes, summing all octaves:

$$C[p] = \sum_{k:\ \mathrm{round}(69 + 12\log_2 \frac{k f_s / N}{440}) \bmod 12 = p} |\hat{S}[k]|^2, \qquad p = 0..11,$$

then normalise $C$ to unit length. A chord template is a 0/1 vector over pitch classes
(C major = C, E, G = $\{0, 4, 7\}$; minor = $\{0, 3, 7\}$; each root is a rotation).
Score = cosine similarity $\langle C, t \rangle / (\|C\| \|t\|)$, report the best root and kind, show the score as match percentage.

Why it works: harmonics 2, 4, 8 of any note fold onto the same pitch class, and harmonic 3 is a fifth,
which is in both major and minor triads anyway. Why it is limited: harmonic 5 is a major third, which
pushes minor chords toward major; weighting lower bins more reduces this but does not remove it.
