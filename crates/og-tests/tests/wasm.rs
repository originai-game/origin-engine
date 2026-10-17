//! The renderer must keep compiling for the browser (WebGPU and WebGL2).

use std::path::PathBuf;
use std::process::Command;

#[test]
fn og_render_checks_for_wasm32() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    // A separate target directory: the outer `cargo test` may hold the lock on its own.
    let target = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("wasm-check");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let out = Command::new(cargo)
        .current_dir(&root)
        .args([
            "check",
            "--target",
            "wasm32-unknown-unknown",
            "-p",
            "og-render",
            "--message-format=short",
        ])
        .env("CARGO_TARGET_DIR", &target)
        .output()
        .expect("run cargo");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "cargo check --target wasm32-unknown-unknown -p og-render failed:\n{stderr}"
    );
}
