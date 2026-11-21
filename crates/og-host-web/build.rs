//! A publish wasm is linked with `--emit-relocs`: its `linking` and
//! `reloc.*` sections say which data each function reaches, so the split
//! gives each on-demand module its own data (packages/engine/scripts/
//! wasm-split-data.ts). A link argument here, not in RUSTFLAGS, so only this
//! crate's link changes and the variants share every dependency's build.
fn main() {
    let wasm = std::env::var("CARGO_CFG_TARGET_ARCH").is_ok_and(|a| a == "wasm32");
    if wasm && std::env::var_os("CARGO_FEATURE_PUBLISH").is_some() {
        println!("cargo:rustc-link-arg-cdylib=--emit-relocs");
    }
}
