#!/usr/bin/env bash
# Builds dsp-core for Android and generates the Kotlin bindings.
#
#   .so      -> android/app/src/main/jniLibs/<abi>/libdsp_core.so
#   Kotlin   -> android/app/build/generated/uniffi/
#
# Needs cargo-ndk (`cargo install cargo-ndk`) and ANDROID_NDK_HOME (or an NDK
# installed through Android Studio, which cargo-ndk finds on its own).
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
core="$root/dsp-core"
profile="${PROFILE:-release}"

cd "$core"

# arm64-v8a for phones, x86_64 for the emulator.
cargo ndk -t arm64-v8a -t x86_64 \
    -o "$root/android/app/src/main/jniLibs" \
    build --profile "$profile"

cargo run --quiet --features bindgen --bin uniffi-bindgen -- \
    generate \
    --library "$core/target/aarch64-linux-android/$profile/libdsp_core.so" \
    --language kotlin \
    --no-format \
    --out-dir "$root/android/app/build/generated/uniffi"
