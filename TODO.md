# TODO

## dsp-core
- [x] `State` struct (window, twiddles, frame, noise profile) in its own module
- [x] `src/bin/uniffi-bindgen.rs` calling `uniffi::uniffi_bindgen_main()` so generated bindings always match the runtime
- [x] `[profile.release]` with `lto = "fat"`, `codegen-units = 1` (never `panic = "abort"`, UniFFI must catch panics)
- [ ] Fill in `tests/pitch.rs` (sine in, pitch out) and `benches/process.rs` once `process` exists; both are placeholders
- [ ] `process` takes `spectrum: Vec<u8>` by value, so writes never reach the app. Return the spectrum instead until `&mut [u8]` ships
- [ ] Drop `#![allow(dead_code, unused_variables)]` in `lib.rs` once the stubs are implemented
- [ ] Switch `process` to `&mut [u8]` output once uniffi 0.33 ships (see docs/SPEC.md)

## Toolchain
- [x] Pin the Rust toolchain (`rust-toolchain.toml`, 1.99.0)
- [ ] Replace asdf-rust with rustup (needed for Android/iOS targets)
- [ ] `cargo install cargo-ndk`, Android NDK via Android Studio

## CI
- [ ] GitHub Actions: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`
