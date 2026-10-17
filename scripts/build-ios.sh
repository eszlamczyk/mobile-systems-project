#!/usr/bin/env bash
# Builds dsp-core for iOS and packages it as an XCFramework. macOS only.
#
#   XCFramework -> ios/DspCore.xcframework   (static libs + DspCoreFFI headers)
#   Swift       -> ios/generated/uniffi/DspCore.swift
set -euo pipefail

if [[ "$(uname)" != "Darwin" ]]; then
    echo "build-ios.sh needs macOS and Xcode" >&2
    exit 1
fi

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
core="$root/dsp-core"
profile="${PROFILE:-release}"
targets=(aarch64-apple-ios aarch64-apple-ios-sim)

cd "$core"

for target in "${targets[@]}"; do
    cargo build --profile "$profile" --target "$target"
done

gen="$root/ios/generated/uniffi"
cargo run --quiet --features bindgen --bin uniffi-bindgen -- \
    generate \
    --library "$core/target/aarch64-apple-ios/$profile/libdsp_core.a" \
    --language swift \
    --no-format \
    --out-dir "$gen"

# xcodebuild wants the headers and a `module.modulemap` together in one directory.
headers="$core/target/uniffi-headers"
rm -rf "$headers"
mkdir -p "$headers"
cp "$gen/DspCoreFFI.h" "$headers/"
cp "$gen/DspCoreFFI.modulemap" "$headers/module.modulemap"

xcframework="$root/ios/DspCore.xcframework"
rm -rf "$xcframework"
args=()
for target in "${targets[@]}"; do
    args+=(-library "$core/target/$target/$profile/libdsp_core.a" -headers "$headers")
done
xcodebuild -create-xcframework "${args[@]}" -output "$xcframework"
