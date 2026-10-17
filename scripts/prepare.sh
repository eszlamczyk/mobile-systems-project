#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
platform="${1:-all}"

case "$platform" in
    all | ios | android) ;;
    *)
        echo "usage: scripts/prepare.sh [ios|android]" >&2
        exit 2
        ;;
esac

export PATH="$HOME/.cargo/bin:$PATH"

require() {
    if ! command -v "$1" > /dev/null; then
        echo "missing $1: $2" >&2
        exit 1
    fi
}

require rustup "https://rustup.rs"

if [[ "$platform" != ios ]]; then
    require cargo-ndk "cargo install cargo-ndk"
    bash "$root/scripts/build-android.sh"
fi

if [[ "$platform" != android ]]; then
    if [[ "$(uname)" == Darwin ]]; then
        require xcodebuild "install Xcode"
        bash "$root/scripts/build-ios.sh"
    elif [[ "$platform" == ios ]]; then
        echo "iOS needs macOS" >&2
        exit 1
    fi
fi
