# TODO

## dsp-core
- [ ] `State` struct (window, twiddles, frame, noise profile) in its own module
- [x] `src/bin/uniffi-bindgen.rs` calling `uniffi::uniffi_bindgen_main()` so generated bindings always match the runtime
- [ ] `[profile.release]` with `lto = "fat"`, `codegen-units = 1` (never `panic = "abort"`, UniFFI must catch panics)
- [ ] First test in `tests/` (sine in, pitch out) and `criterion` bench of `process` in `benches/`
- [ ] Switch `process` to `&mut [u8]` output once uniffi 0.33 ships (see docs/SPEC.md)

## Toolchain
- [ ] Replace asdf-rust with rustup (needed for Android/iOS targets)
- [ ] `cargo install cargo-ndk`, Android NDK via Android Studio

## CI
- [ ] GitHub Actions: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`
