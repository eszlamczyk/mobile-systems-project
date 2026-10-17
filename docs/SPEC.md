# Mobile systems project specification

This document is the specification for this project. It is **not** the final report we want alongside the working project.
All derivations ("why is this correct") live in [MUSIC.md](MUSIC.md) (notes, cents, harmonics) and [FFT.md](FFT.md) (FFT, windowing, YIN, noise removal); this file only states decisions.

## Overview

A tuner. The user picks a target note, plays a sound, and the app shows the live frequency spectrum,
a grey "ghost" of what the target should look like, and a red..green indicator of how close the played
pitch is to the target.

## Architecture

One monorepo, three parts:

```
dsp-core/      Rust crate: signal processing. Unit tested
android/   Kotlin UI + mic capture (AudioRecord). Calls core through UniFFI-generated Kotlin.
ios/       Swift UI + mic capture (AVAudioEngine). Calls core through UniFFI-generated Swift.
```

This is done so we don't have to write the same logic multiple times. Pure math goes in `dsp-core/`. Anything that touches the OS (mic, threads, drawing) stays in the apps.
The core has no threads, no I/O, and no allocations after `Tuner::new`. Buffers cross the UniFFI boundary borrowed, not copied (see Core API).

### Core language

Rust, chosen for DX, exposed through [UniFFI](https://mozilla.github.io/uniffi-rs/) proc-macros (no `.udl` file).
`cargo test` on the desktop, no undefined behaviour in the DSP, and generated Kotlin and Swift classes instead of hand-written JNI and bridging headers.

Toolchain on top of Android Studio and Xcode:

- `uniffi` 0.32+ for zero-copy `&[u8]` arguments. `&mut [u8]` is on `main` but unreleased as of 0.32.2; pin a git revision, or return the spectrum as `Vec<u8>` until it ships.
- `rustup` with targets `aarch64-linux-android`, `aarch64-apple-ios`, `aarch64-apple-ios-sim`
- `cargo-ndk` to build the Android `.so` against the NDK and drop it into `jniLibs/`.
- `uniffi-bindgen`, run as a binary in the crate (`cargo run --features bindgen --bin uniffi-bindgen`), to generate the Kotlin and Swift sources from the compiled library. Names come from `dsp-core/uniffi.toml`: Kotlin package `uniffi.dsp_core`, Swift module `DspCore`.
- `scripts/build-android.sh` and `scripts/build-ios.sh` wrap the whole build (see Glue); the iOS one packages the static libs with `xcodebuild -create-xcframework`.
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
// dsp-core/src/lib.rs   UniFFI proc-macros. Buffers are borrowed, records are copied (see traps).

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

- **Android** (`scripts/build-android.sh`): `cargo ndk -t arm64-v8a -t x86_64 -o android/app/src/main/jniLibs build --release`, then
  `uniffi-bindgen generate --library libdsp_core.so --language kotlin` into `android/app/build/generated/uniffi/`, wired into Gradle as a source dir. Kotlin gets `class Tuner` and uses it directly. The generated bindings load the library through JNA (`net.java.dev.jna:jna@aar` dependency).
- **iOS** (`scripts/build-ios.sh`, macOS only): `cargo build --release` for device and simulator targets, `uniffi-bindgen generate --library libdsp_core.a --language swift`, then fill the local Swift package `ios/DspCore/`: the static libs as `DspCoreFFI.xcframework`, the generated Swift as `Sources/DspCore/DspCore.swift`. The app (`ios/MobileTuner.xcodeproj`) depends on it and does `import DspCore`. Swift gets `class Tuner`.
- **Tests**: `dsp-core/tests/` with synthesized sines and recorded WAVs; FFT compared against a naive DFT. No need to Unit test the UI, possibly using Argent Flows. `cargo bench` for `process` to keep an eye on the per-hop cost.

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

> On top a selectable bar to pick the target: a single note (for example B) or a tuning preset.
> A big panel saying what you selected, colour coded red..green by how close you are to the target.
> A plot of the live frequency spectrum, cleaned up with spectral subtraction.
> The same plot shows a greyed out ghost of how the target should look.

### The layout

```
+---------------------------+
|                           |
|  [ Drop D            v ]  |  <- selector: single note or preset
|  [D] [A] [D] [G] [B] [E]  |  <- string buttons (preset only)
|                           |
|  +---------------------+  |
|  |                     |  |
|  |          D          |  |  <- big panel, colour coded
|  |                     |  |     red..green by how close
|  |      +3.2 cents     |  |     you are, with the offset
|  |     (73.6 Hz)       |  |     in cents; Hz only when
|  |                     |  |     the toggle is on
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

- **Cents** are always shown. The **frequency** is behind a toggle, off by default: most users only need "how far and which way",
  the raw Hz is for people who want it.
- Colour thresholds are in MUSIC.md section 2.

### Tuning presets

A **preset** is a named, ordered list of target notes, one per string. Picking a preset shows a row of
string buttons; tapping one sets the target exactly as picking a single note does. The DSP does not change,
the core only ever sees one target MIDI note through `set_target`.

The minimum is a **fixed list we provide**, focused on alternate tunings, not only standard:

| Group | Presets |
|---|---|
| Standard | E standard (E A D G B E) |
| Lowered | Half step down (E♭ A♭ D♭ G♭ B♭ E♭), full step down (D G C F A D) |
| Drop | Drop D (D A D G B E), Drop C (C G C F A D), Drop B (B F♯ B E G♯ C♯) |
| Open | Open G (D G D G B D), Open D (D A D F♯ A D), Open E (E B E G♯ B E), Open C (C G C G C E) |
| Modal | DADGAD (D A D G A D, a "suspended" tuning popular in Celtic and folk music) |
| Other instruments | Bass 4 string (E A D G), ukulele (G C E A), violin (G D A E) |

Exact octaves (MIDI numbers) per string are fixed in the enum, not here.

**Where the data lives**: in each app, not in the core. Presets are UI data, not math, and keeping them out of
`dsp-core/` keeps the core's job to "PCM in, result out".

- Kotlin and Swift each have a `Preset` enum with the same cases, names and MIDI notes.
- The UI works with a plain `Preset(name, notes)` value that the enum maps into, so user-made presets
  (Nice to have) can be added later without touching the UI.
- A CI check keeps the two enums in sync: a script extracts case name and MIDI notes from both files and fails on any diff.

## Nice to have

### Presets

- **Custom presets**: the user builds their own list of notes, saved locally per app. Same `Preset(name, notes)` value as the fixed list.
- **Auto string pick**: while a preset is active, highlight the string nearest to the played pitch, so the user does not have to tap a string first.
- **Progress**: mark which strings are already in tune, so a full retune shows progress.
- **Reference pitch**: A4 adjustable (for example 432 to 446 Hz) instead of fixed 440. This one does touch the core (`f(n)` in MUSIC.md section 1).

### Other

- Log-frequency axis on the plot so low notes are readable.
