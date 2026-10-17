# Mobile Tuner

```
dsp-core/   Rust core
android/    Android app
ios/        iOS app
scripts/    build scripts
docs/       spec and math
```

## Requirements

- rustup
- cargo-ndk (`cargo install cargo-ndk`)
- Android Studio with NDK
- Xcode

## Start

```sh
scripts/prepare.sh
```

Android: open `android/` in Android Studio, run `app`.

iOS: open `ios/MobileTuner.xcodeproj` in Xcode, run `MobileTuner`. After changing `dsp-core/`, run `scripts/prepare.sh ios` again.

## Core

```sh
cd dsp-core
cargo test
cargo clippy --all-targets -- -D warnings
```
