//! Where the WGSL the renderer compiles is: og-render-data's build step
//! strips `shaders/` into its own output (its build.rs), and the `wgsl!`
//! macro (src/lib.rs) includes the files from there.

fn main() {
    let out = std::env::var("DEP_OG_RENDER_SHADERS_OUT").expect("og-render-data's build.rs");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=DEP_OG_RENDER_SHADERS_OUT");
    println!("cargo:rustc-env=OG_RENDER_SHADERS={out}");
}
