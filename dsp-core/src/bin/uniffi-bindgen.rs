//! Generates the Kotlin and Swift bindings from the compiled library, using the
//! same `uniffi` version the library links so the two never drift.
//!
//! Run with `cargo run --features bindgen --bin uniffi-bindgen -- generate --library <lib> --language kotlin`.

fn main() {
    uniffi::uniffi_bindgen_main();
}
