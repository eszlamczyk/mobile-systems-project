# Mobile systems project specification

This document is the specification for this project. It is **not** the final report we want alongside the working project.
All derivations ("why is this correct") live in [MUSIC.md](MUSIC.md) (notes, cents, harmonics, chords) and [FFT.md](FFT.md) (FFT, windowing, YIN, noise removal); this file only states decisions.

## Overview

A tuner. The user picks a target note, plays a sound, and the app shows the live frequency spectrum,
a grey "ghost" of what the target should look like, and a red..green indicator of how close the played
pitch is to the target.

## Architecture

One monorepo, three parts:

```
core/      Rust crate: signal processing. Unit tested
android/   Kotlin UI + mic capture (AudioRecord). Calls core through UniFFI-generated Kotlin.
ios/       Swift UI + mic capture (AVAudioEngine). Calls core through UniFFI-generated Swift.
```

This is done so we don't have to write the same logic multiple times. Pure math goes in `core/`. Anything that touches the OS (mic, threads, drawing) stays in the apps.
The core has no threads, no I/O, and no allocations after `Tuner::new`. Buffers cross the UniFFI boundary borrowed, not copied (see Core API).

### Core language

Rust, chosen for DX, exposed through [UniFFI](https://mozilla.github.io/uniffi-rs/) proc-macros (no `.udl` file).
`cargo test` on the desktop, no undefined behaviour in the DSP, and generated Kotlin and Swift classes instead of hand-written JNI and bridging headers.

Toolchain on top of Android Studio and Xcode:

- `uniffi` 0.32+ for zero-copy `&[u8]` arguments. `&mut [u8]` is on `main` but unreleased as of 0.32.2; pin a git revision, or return the spectrum as `Vec<u8>` until it ships.
- `rustup` with targets `aarch64-linux-android`, `aarch64-apple-ios`, `aarch64-apple-ios-sim`
- `cargo-ndk` to build the Android `.so` against the NDK and drop it into `jniLibs/`.
- `uniffi-bindgen`, run as a binary in workspace (`cargo run --bin uniffi-bindgen`), to generate the Kotlin and Swift sources from the compiled library.
- A script that wraps the iOS static libs into an XCFramework (`xcodebuild -create-xcframework`), or `cargo-swift` which does the same.
- Reference scaffolding: [uniffi-starter](https://github.com/ianthetechie/uniffi-starter) (Android library build, XCFramework script, Swift package layout).

Other consideration was Rust with a raw C ABI via `cbindgen` plus the `jni` crate (also zero-copy, but all the glue is hand written).

## Threads

Threads are already mapped out to not forget anything related to the threading (that would cause blocking behaviour on the main thread)

| Thread | Owned by | Does | Must not |
|---|---|---|---|
| **Audio capture** | OS (iOS tap callback) / us (Android `AudioRecord.read` loop) | copy PCM into a lock-free ring buffer | allocate, take a lock the UI can hold, log, touch UI |
| **DSP** | us | pull one hop from the ring buffer, call `Tuner.process`, publish result by atomic swap of a double buffer | touch UI objects |
| **UI / main** | OS | at most 30 times per second read the latest result, set colour, redraw plot | **block waiting on DSP** |

### Additional notes 
- On Android capture and DSP may be the same thread, because `read()` blocks on a thread we own.
- Noise learning is requested from UI by setting a flag; the DSP thread executes it.
- The `Tuner` object holds a `Mutex<State>` because UniFFI objects must be `Send + Sync`. Only the DSP thread locks it (`process`, `learn_noise`), so it is never contended. UI calls (`set_target`, `target_spectrum`) touch only atomics and immutable config.
- **Capture stops** when the app goes to background. Nothing runs while not visible.

## Core API

```rust
// core/src/lib.rs   UniFFI proc-macros. Buffers are borrowed, records are copied (see traps).

#[derive(uniffi::Record)]
pub struct TunerConfig {
    pub sample_rate: u32,   // device native rate (44100 or 48000), no resampling
    pub fft_size: u32,      // power of two, default 4096
    pub hop_size: u32,      // samples between frames, default 1024
    pub min_hz: f32,        // band-pass, default 50
    pub max_hz: f32,        // band-pass, default 5000
}

#[derive(uniffi::Record)]
pub struct TunerResult {
    pub pitch_hz: f32,      // 0 when nothing detected (gate)
    pub cents: f32,         // offset from target, unbounded (UI saturates colour at +-50)
    pub nearest_midi: i32,  // nearest note to pitch_hz
    pub confidence: f32,    // 0 .. 1 from YIN
    pub rms: f32,           // input loudness, drives the gate
}

#[derive(uniffi::Object)]
pub struct Tuner {
    config: TunerConfig,      // immutable after new
    target_midi: AtomicI32,   // written by UI, read by process
    inner: Mutex<State>,      // window, twiddles, frame, noise profile; DSP thread only
}

#[uniffi::export]
impl Tuner {
    #[uniffi::constructor]
    pub fn new(config: TunerConfig) -> Arc<Self>;       // all buffers allocated here
    pub fn spectrum_size(&self) -> u32;                 // fft_size / 2
    pub fn set_target(&self, midi_note: i32);           // atomic store, no lock
    pub fn target_spectrum(&self) -> Vec<f32>;          // grey ghost comb, UI thread, on target change
    pub fn learn_noise(&self, pcm: &[u8]);              // ~1 s of silence
    pub fn process(&self, pcm: &[u8], spectrum: &mut [u8]) -> TunerResult;
                                                        // pcm: one hop of f32 samples as bytes
                                                        // spectrum: fft_size / 2 magnitudes, written in place
}
```

Inside `process`: window, FFT, magnitude, spectral subtraction,
band-pass, smoothing into `spectrum`; YIN on the time-domain frame for `pitch_hz`; cents against target.

Buffers are native-endian f32 as bytes: a direct `ByteBuffer` on Kotlin, `Data` on Swift. The apps allocate them once and reuse them every hop.
`process` expects exactly one hop. The ring buffer guarantees that; anything else panics and the frame is dropped (see traps).

### Glue

- **Android**: `cargo ndk -t arm64-v8a -o android/app/src/main/jniLibs build --release`, then
  `uniffi-bindgen generate --library <libtuner.so> --language kotlin` into a generated source dir wired into Gradle. Kotlin gets `class Tuner` and uses it directly. The generated bindings load the library through JNA (`net.java.dev.jna:jna@aar` dependency).
- **iOS**: `cargo build --release` for device and simulator targets, `uniffi-bindgen ... --language swift`, bundle into `Tuner.xcframework`, add as a local Swift package. Swift gets `class Tuner`.
- **Tests**: `core/tests/` with synthesized sines and recorded WAVs; FFT compared against a naive DFT. No need to Unit test the UI, possibly using Argent Flows. `cargo bench` for `process` to keep an eye on the per-hop cost.

## Performance traps

- **Allocations on the audio or DSP thread** cause GC pauses (Kotlin) and ARC traffic (Swift). Preallocate in `Tuner::new`; the apps allocate their buffers once and reuse them.
- **Calling the core per sample** is the classic mistake. UniFFI serialises records and `Vec`s; a call costs microseconds through JNA. Once per hop (47 Hz) is fine, 48 kHz is not.
- **Copies across the boundary**: `&[u8]` / `&mut [u8]` are borrowed, so PCM and spectrum cross without a copy. A `Vec<f32>` would become a Kotlin `List<Float>`, boxing 2048 floats per hop.
- **Swift `Array` to pointer** copies unless done through `withUnsafeBufferPointer`; irrelevant through UniFFI, relevant if we add a raw path.
- **Recomputing twiddles or the window** every frame. Precompute once in `new`.
- **`f64` instead of `f32`** halves NEON throughput and doubles cache use.
- **Bounds checks in the FFT inner loop**: use `chunks_exact` and iterators so the compiler can elide them; measure before reaching for `get_unchecked`.
- **Mutex between audio and UI** stalls the audio callback. Ring buffer plus atomic swap only; the core's own mutex is taken only by the DSP thread (see Threads).
- **Panics in the hot path**: UniFFI turns a Rust panic into an exception, which drops the frame; the DSP loop catches it and continues. No `unwrap` on input-dependent values in `process`. Recover the `Mutex` from poisoning, or one panic kills every later call.
- **Logging in the audio path** allocates and blocks.
- **Resampling**: ask the device for its native rate instead of forcing 44100.
- **Plot drawing**: 2048 path segments at 60 fps is slow. Downsample to pixel width (~400 points), use one path, redraw when a new result arrives (max 30 Hz), not every screen frame.
- **Battery**: continuous mic plus DSP. Stop on background, skip DSP while gated.
- **Denormal floats** in decaying filters run up to 100 times slower. Enable flush-to-zero.
- **Debug builds lie**: a Rust debug build is 10 to 50 times slower than release. Build with `--release`, set `lto = "fat"` and `codegen-units = 1` in the release profile, arm64 only, and measure with Perfetto and Instruments.

Latency budget with defaults at 48 kHz: 85 ms frame, 21 ms hop, about 47 DSP updates per second, UI reads 30.

## Must

The UI should look like:

> On top a selectable bar to pick the target note (for example B).
> A big panel saying what you selected, colour coded red..green by how close you are to the target.
> A plot of the live frequency spectrum, cleaned up with spectral subtraction.
> The same plot shows a greyed out ghost of how the target should look.

### The layout

```
+---------------------------+
|                           |
|  [ Target note:   B  v ]  |  <- selector
|                           |
|  +---------------------+  |
|  |                     |  |
|  |          B          |  |  <- big panel, colour coded
|  |                     |  |     red..green by how close
|  |      +3.2 cents     |  |     you are, with the offset
|  |      494.8 Hz       |  |     shown in real numbers
|  |                     |  |
|  +---------------------+  |
|                           |
|  +---------------------+  |
|  |        .            |  |  <- live spectrum plot
|  |       / \   .--.    |  |     (grey ghost = target)
|  |      /   \./    \   |  |
|  |  ___/           \__ |  |
|  +---------------------+  |
|                           |
+---------------------------+
```

## Nice to have

### Chords

Viable, with limits. A chord is several notes at once (C major = C, E, G). The spectrum shows all
fundamentals plus their overlapping harmonics, so single-pitch detection cannot work. Instead:

- The core folds the spectrum into a 12-bin **chroma vector** (one bin per pitch class, all octaves summed)
  and scores it against templates for major and minor triads. See MUSIC.md section 5.
- **UI**: the selector gains a chord mode; the ghost draws three combs; the big panel shows the chord
  name and a match percentage instead of cents.
- **API**: `Tuner::chroma(&self) -> Vec<f32>` (12 values) and `Tuner::match_chord(&self, root: i32, kind: ChordKind) -> f32`.
- **Limits**: polyphonic instruments only (guitar, piano), clean signal, templates we add by hand.
  Individual strings of a strummed chord cannot be tuned to the cent this way.


This would be a separate Screen/panel displaying "What did you just play"

### Other

- Instrument presets (guitar standard tuning as six selectable targets).
- Log-frequency axis on the plot so low notes are readable.
