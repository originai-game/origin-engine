//! `og doctor`: which tools are present, at which versions, against the
//! repository's pins, with one line naming the fix for each problem.

use std::path::Path;
use std::process::{Command, ExitCode};

struct Tool {
    name: &'static str,
    /// What the tool is for, shown when it is missing.
    role: &'static str,
    install: fn(Option<&str>) -> String,
    pin: Pin,
}

#[derive(Clone, Copy)]
enum Pin {
    /// The channel in rust-toolchain.toml.
    RustToolchain,
    /// The `wasm-bindgen` crate version in Cargo.lock: the CLI must match it.
    WasmBindgenLock,
    None,
}

const TOOLS: &[Tool] = &[
    Tool {
        name: "rustc",
        role: "builds the engine",
        install: |pin| match pin {
            Some(v) => {
                format!("install rustup (https://rustup.rs), then `rustup toolchain install {v}`")
            }
            None => "install rustup from https://rustup.rs".into(),
        },
        pin: Pin::RustToolchain,
    },
    Tool {
        name: "cargo",
        role: "builds the engine",
        install: |pin| match pin {
            Some(v) => {
                format!("install rustup (https://rustup.rs), then `rustup toolchain install {v}`")
            }
            None => "install rustup from https://rustup.rs".into(),
        },
        pin: Pin::RustToolchain,
    },
    Tool {
        name: "wasm-bindgen",
        role: "generates the web host's glue",
        install: |pin| match pin {
            Some(v) => format!("`cargo install wasm-bindgen-cli --version {v} --locked`"),
            None => "`cargo install wasm-bindgen-cli --locked`".into(),
        },
        pin: Pin::WasmBindgenLock,
    },
    Tool {
        name: "bun",
        role: "bundles and type-checks game code",
        install: |_| "`curl -fsSL https://bun.sh/install | bash`".into(),
        pin: Pin::None,
    },
    Tool {
        name: "node",
        role: "loads the native addon outside bun",
        install: |_| "install Node.js 22 or newer from https://nodejs.org".into(),
        pin: Pin::None,
    },
];

pub fn run() -> ExitCode {
    let cwd = std::env::current_dir().unwrap_or_else(|_| ".".into());
    let root = crate::find_root(&cwd);
    let toolchain = root.as_deref().and_then(rust_channel);
    let bindgen = root.as_deref().and_then(wasm_bindgen_lock);

    println!("og doctor (og {})", crate::VERSION);
    match &root {
        Some(r) => {
            println!("repository  {}", r.display());
            println!(
                "pinned      rust {} (rust-toolchain.toml), wasm-bindgen {} (Cargo.lock)",
                toolchain.as_deref().unwrap_or("not pinned"),
                bindgen.as_deref().unwrap_or("not locked")
            );
        }
        None => println!(
            "repository  none above {}; versions are not compared with pins",
            cwd.display()
        ),
    }

    let mut problems = 0;
    for tool in TOOLS {
        let pin = match tool.pin {
            Pin::RustToolchain => toolchain.as_deref(),
            Pin::WasmBindgenLock => bindgen.as_deref(),
            Pin::None => None,
        };
        // rustup resolves rust-toolchain.toml from the working directory.
        let found = version_of(tool.name, root.as_deref().unwrap_or(&cwd));
        match (found, pin) {
            (None, _) => {
                problems += 1;
                println!("missing  {:<13} {}", tool.name, tool.role);
                println!("         fix: {}", (tool.install)(pin));
            }
            (Some(v), Some(p)) if v != p => {
                problems += 1;
                println!("differs  {:<13} {v}, pinned {p}", tool.name);
                println!("         fix: {}", (tool.install)(Some(p)));
            }
            (Some(v), _) => println!("ok       {:<13} {v}", tool.name),
        }
    }
    if problems == 0 {
        println!("all tools present.");
        ExitCode::SUCCESS
    } else {
        println!("{problems} problem(s); each line above names its fix.");
        ExitCode::FAILURE
    }
}

/// The first dotted version number in `<tool> --version`.
fn version_of(tool: &str, dir: &Path) -> Option<String> {
    let out = Command::new(tool)
        .arg("--version")
        .current_dir(dir)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    text.split_whitespace()
        .map(|w| w.trim_start_matches('v'))
        .find(|w| {
            w.split('.').count() >= 2
                && w.split('.')
                    .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
        })
        .map(str::to_string)
}

/// `channel = "1.97.0"` in rust-toolchain.toml.
fn rust_channel(root: &Path) -> Option<String> {
    let text = std::fs::read_to_string(root.join("rust-toolchain.toml")).ok()?;
    text.lines().find_map(|l| {
        let (key, value) = l.split_once('=')?;
        (key.trim() == "channel").then(|| value.trim().trim_matches('"').to_string())
    })
}

/// The locked version of the `wasm-bindgen` crate.
fn wasm_bindgen_lock(root: &Path) -> Option<String> {
    let text = std::fs::read_to_string(root.join("Cargo.lock")).ok()?;
    let mut lines = text.lines();
    while let Some(l) = lines.next() {
        if l.trim() == "name = \"wasm-bindgen\"" {
            let v = lines.next()?.trim().strip_prefix("version = ")?;
            return Some(v.trim_matches('"').to_string());
        }
    }
    None
}
