//! The renderer must keep compiling for the browser (WebGPU and WebGL2).
//!
//! The gate's wasm-build step (scripts/check.sh, before the test step)
//! already compiles og-render for wasm32 into `$OG_WASM_OUT` (default
//! `target/og-wasm`). When that build succeeded, produced og-render's
//! artifact and is newer than every source it was built from, this test
//! takes it as the proof; otherwise it runs `cargo check` itself, so a
//! missing or stale gate build is never a pass.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

#[test]
fn og_render_checks_for_wasm32() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root");
    let gate = std::env::var_os("OG_WASM_OUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target/og-wasm"));
    match gate_build_of_og_render(&root, &gate) {
        Ok(proof) => {
            println!("og-render for wasm32: the gate's build ({proof})");
            return;
        }
        Err(why) => {
            println!("og-render for wasm32: not reusing the gate's build ({why}); cargo check")
        }
    }
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

/// `Ok` with what was found when `<gate>/cargo.json` (the wasm-build step's
/// cargo messages) reports a successful build with an og-render artifact
/// that still exists, written after the last change to og-render, its path
/// dependencies, the workspace manifest, the lock file and the toolchain.
fn gate_build_of_og_render(root: &Path, gate: &Path) -> Result<String, String> {
    let messages = gate.join("cargo.json");
    let text = fs::read_to_string(&messages).map_err(|e| format!("{}: {e}", messages.display()))?;
    if !text
        .lines()
        .any(|l| l.contains(r#""reason":"build-finished""#) && l.contains(r#""success":true"#))
    {
        return Err(format!(
            "{} reports no successful build",
            messages.display()
        ));
    }
    let manifest = root.join("crates/og-render/Cargo.toml");
    let wanted = format!(r#""manifest_path":"{}""#, manifest.display());
    let artifact = text
        .lines()
        .find(|l| l.contains(r#""reason":"compiler-artifact""#) && l.contains(&wanted))
        .ok_or_else(|| format!("{} has no og-render artifact", messages.display()))?;
    let files = string_array(artifact, "filenames");
    if files.is_empty() || !files.iter().all(|f| Path::new(f).is_file()) {
        return Err(format!("og-render's artifact {files:?} is gone"));
    }
    let built = modified(&messages)?;
    let mut sources = vec![
        root.join("Cargo.toml"),
        root.join("Cargo.lock"),
        root.join("rust-toolchain.toml"),
    ];
    for dir in path_closure(&root.join("crates/og-render")) {
        walk(&dir, &mut sources);
    }
    for s in &sources {
        if s.exists() && modified(s)? > built {
            return Err(format!("{} changed after it", s.display()));
        }
    }
    Ok(format!("{}, {} sources older", files[0], sources.len()))
}

/// The values of a JSON string array field on one line of cargo's messages.
fn string_array(line: &str, field: &str) -> Vec<String> {
    let key = format!(r#""{field}":["#);
    let Some(at) = line.find(&key) else {
        return Vec::new();
    };
    let rest = &line[at + key.len()..];
    let Some(end) = rest.find(']') else {
        return Vec::new();
    };
    rest[..end]
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// A crate directory and every crate it reaches through `path = "…"`
/// dependencies (normal, build and dev), each once.
fn path_closure(start: &Path) -> Vec<PathBuf> {
    let mut seen: Vec<PathBuf> = Vec::new();
    let mut todo = vec![start.to_path_buf()];
    while let Some(dir) = todo.pop() {
        let Ok(dir) = dir.canonicalize() else {
            continue;
        };
        if seen.contains(&dir) {
            continue;
        }
        if let Ok(toml) = fs::read_to_string(dir.join("Cargo.toml")) {
            for (i, _) in toml.match_indices("path = \"") {
                let rest = &toml[i + 8..];
                if let Some(end) = rest.find('"') {
                    todo.push(dir.join(&rest[..end]));
                }
            }
        }
        seen.push(dir);
    }
    seen
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        let name = e.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') || name == "target" {
            continue;
        }
        if p.is_dir() {
            walk(&p, out);
        } else {
            out.push(p);
        }
    }
}

fn modified(p: &Path) -> Result<SystemTime, String> {
    fs::metadata(p)
        .and_then(|m| m.modified())
        .map_err(|e| format!("{}: {e}", p.display()))
}
