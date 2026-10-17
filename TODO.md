# TODO

## dsp-core
- [x] `State` struct (window, twiddles, frame, noise profile) in its own module
- [x] `src/bin/uniffi-bindgen.rs` calling `uniffi::uniffi_bindgen_main()` so generated bindings always match the runtime
- [x] `[profile.release]` with `lto = "fat"`, `codegen-units = 1` (never `panic = "abort"`, UniFFI must catch panics)
- [ ] Fill in `tests/pitch.rs` (sine in, pitch out) and `benches/process.rs` once `process` exists; both are placeholders
- [x] `process` returns the spectrum in `TunerResult::spectrum` until `&mut [u8]` ships
- [x] Every exported method returns `Result<_, TunerError>` so Swift can catch panics (non-throwing calls go through `try!`)
- [ ] Recover `Mutex<State>` from poisoning (`lock().unwrap_or_else(PoisonError::into_inner)`), or one panic fails every later frame
- [ ] Drop `#![allow(dead_code, unused_variables)]` in `lib.rs` once the stubs are implemented
- [ ] Switch `process` to `&mut [u8]` output once uniffi 0.33 ships, drop `TunerResult::spectrum` (see docs/SPEC.md)

## Toolchain
- [x] Pin the Rust toolchain (`rust-toolchain.toml`, 1.99.0)
- [ ] Replace asdf-rust with rustup (needed for Android/iOS targets)
- [ ] `cargo install cargo-ndk`, Android NDK via Android Studio

## CI
- [ ] GitHub Actions: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`
