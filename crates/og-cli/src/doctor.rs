//! `og doctor`: what a game author's loop needs (bun, tsrs, the engine
//! package, its native addon, the web host's wasm), whether this machine has a
//! headless renderer (reported, not required: without one the playtests'
//! frames are unchecked), then, inside the engine repository, the build
//! toolchain against the repository's pins. Every problem has one line
//! naming its fix.
//!
//! Outside the engine repository (an executor: bun, the engine package and
//! its addon) the Rust and wasm-bindgen checks are skipped, and node is
//! optional when bun is present (bun loads the addon itself).

use std::path::Path;
use std::process::{Command, ExitCode};

struct Tool {
    name: &'static str,
    /// What the tool is for, shown when it is missing.
    role: &'static str,
    install: fn(Option<&str>) -> String,
    pin: Pin,
    /// Needed only to build the engine: checked inside the repository only.
    repo_only: bool,
}

#[derive(Clone, Copy)]
enum Pin {
    /// The channel in rust-toolchain.toml.
    RustToolchain,
    /// The `wasm-bindgen` crate version in Cargo.lock: the CLI must match it.
    WasmBindgenLock,
    /// `TSRS_VERSION` in scripts/install-tools.sh; tsrs is found by og
    /// check's own chain (crate::check::locate_tsrs), not only on PATH.
    TsrsVersion,
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
        repo_only: true,
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
        repo_only: true,
        pin: Pin::RustToolchain,
    },
    Tool {
        name: "wasm-bindgen",
        role: "generates the web host's glue",
        install: |pin| match pin {
            Some(v) => format!("`cargo install wasm-bindgen-cli --version {v} --locked`"),
            None => "`cargo install wasm-bindgen-cli --locked`".into(),
        },
        repo_only: true,
        pin: Pin::WasmBindgenLock,
    },
    Tool {
        name: "bun",
        role: "bundles and type-checks game code",
        install: |_| "`curl -fsSL https://bun.sh/install | bash`".into(),
        repo_only: false,
        pin: Pin::None,
    },
    Tool {
        name: "tsrs",
        role: "type-checks game code (og check, and before og run/test/build)",
        install: |pin| {
            match pin {
            Some(v) => format!("run scripts/install-tools.sh (tsrs {v}), or set OG_TSRS to the binary"),
            None => "`bun install` in the project (@origingame/engine brings @tsrs/<platform>), or set OG_TSRS to the binary".into(),
        }
        },
        repo_only: false,
        pin: Pin::TsrsVersion,
    },
    Tool {
        name: "node",
        role: "loads the native addon outside bun",
        install: |_| "install Node.js 22 or newer from https://nodejs.org".into(),
        repo_only: false,
        pin: Pin::None,
    },
];

pub fn run(args: &[&str]) -> ExitCode {
    if let Some(a) = args.first() {
        eprintln!("og doctor: unexpected argument `{a}`; og doctor takes none.");
        return ExitCode::from(2);
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| ".".into());
    let root = crate::find_root(&cwd);
    let toolchain = root.as_deref().and_then(rust_channel);
    let bindgen = root.as_deref().and_then(wasm_bindgen_lock);
    let tsrs = root.as_deref().and_then(tsrs_version);

    println!("og doctor (og {})", crate::VERSION);
    match &root {
        Some(r) => {
            println!("repository  {}", r.display());
            println!(
                "pinned      rust {} (rust-toolchain.toml), wasm-bindgen {} (Cargo.lock), tsrs {} (TSRS_VERSION)",
                toolchain.as_deref().unwrap_or("not pinned"),
                bindgen.as_deref().unwrap_or("not locked"),
                tsrs.as_deref().unwrap_or("not pinned")
            );
        }
        None => println!(
            "repository  none above {}; versions are not compared with pins",
            cwd.display()
        ),
    }

    let mut problems = 0;
    let bun = version_of("bun", root.as_deref().unwrap_or(&cwd)).is_some();
    for tool in TOOLS {
        if tool.repo_only && root.is_none() {
            println!(
                "skipped  {:<13} {} (only inside the engine repository)",
                tool.name, tool.role
            );
            continue;
        }
        let pin = match tool.pin {
            Pin::RustToolchain => toolchain.as_deref(),
            Pin::WasmBindgenLock => bindgen.as_deref(),
            Pin::TsrsVersion => {
                problems += tsrs_check(tool, tsrs.as_deref(), &cwd);
                continue;
            }
            Pin::None => None,
        };
        // rustup resolves rust-toolchain.toml from the working directory.
        let found = version_of(tool.name, root.as_deref().unwrap_or(&cwd));
        match (found, pin) {
            (None, _) if tool.name == "node" && bun => {
                println!(
                    "optional {:<13} not installed; bun loads the addon",
                    tool.name
                );
            }
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
    problems += engine_checks(&cwd);
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

/// `TSRS_VERSION="${TSRS_VERSION:-<version>}"` in scripts/install-tools.sh.
fn tsrs_version(root: &Path) -> Option<String> {
    let text = std::fs::read_to_string(root.join("scripts/install-tools.sh")).ok()?;
    text.lines().find_map(|l| {
        let v = l.strip_prefix("TSRS_VERSION=\"${TSRS_VERSION:-")?;
        Some(v.strip_suffix("}\"")?.to_string())
    })
}

/// The tsrs line: found by og check's chain, its package version against
/// the pin inside the repository. A platform tsrs has no package for is
/// not a problem: og check takes tsgo there.
fn tsrs_check(tool: &Tool, pin: Option<&str>, cwd: &Path) -> usize {
    let package = crate::project::engine_package(Some(cwd));
    match crate::check::locate_tsrs(package.as_deref()) {
        Ok(Some(t)) => match pin {
            Some(p) if t.package_version != p => {
                println!(
                    "differs  {:<13} {}, pinned {p} ({})",
                    tool.name,
                    t.package_version,
                    t.path.display()
                );
                println!("         fix: {}", (tool.install)(Some(p)));
                1
            }
            _ => {
                println!(
                    "ok       {:<13} {} (TypeScript {}, from {}: {})",
                    tool.name,
                    t.package_version,
                    t.version,
                    t.source,
                    t.path.display()
                );
                0
            }
        },
        Ok(None) if crate::check::platform().is_none() => {
            println!(
                "optional {:<13} no tsrs package for this platform; og check uses tsgo {} through bun x",
                tool.name,
                crate::check::TSGO_VERSION
            );
            0
        }
        Ok(None) => {
            println!("missing  {:<13} {}", tool.name, tool.role);
            println!("         fix: {}", (tool.install)(pin));
            1
        }
        Err(e) => {
            println!("broken   {:<13} {e}", tool.name);
            println!("         fix: {}", (tool.install)(pin));
            1
        }
    }
}

/// The engine package's checks (`packages/engine/src/node/doctor.ts`): the
/// addon loads, the wasm is built, and whether a 64x64 frame renders headless.
fn engine_checks(cwd: &Path) -> usize {
    let Some(package) = crate::project::engine_package(Some(cwd)) else {
        println!(
            "missing  {:<13} the engine package (@origingame/engine)",
            "engine"
        );
        println!("         fix: {}", crate::project::NO_ENGINE);
        return 1;
    };
    let out = crate::project::bun(&package, "src/node/doctor.ts").output();
    let report: serde_json::Value = match out {
        Err(_) => {
            println!(
                "skipped  {:<13} {} (needs bun)",
                "engine",
                package.display()
            );
            return 0;
        }
        Ok(o) => match serde_json::from_slice(o.stdout.trim_ascii()) {
            Ok(v) => v,
            Err(_) => {
                println!("broken   {:<13} {}", "engine", package.display());
                println!(
                    "         fix: reinstall @origingame/engine; its doctor printed: {}",
                    String::from_utf8_lossy(&o.stderr).trim()
                );
                return 1;
            }
        },
    };
    println!(
        "ok       {:<13} {} {}",
        "engine",
        report["version"].as_str().unwrap_or("?"),
        package.display()
    );
    let mut problems = 0;
    let mut line = |name: &str, ok: bool, detail: String, fix: &str| {
        if ok {
            println!("ok       {name:<13} {detail}");
        } else {
            problems += 1;
            println!("missing  {name:<13} {detail}");
            println!("         fix: {fix}");
        }
    };
    let addon = &report["addon"];
    line(
        "addon",
        addon["ok"].as_bool().unwrap_or(false),
        match addon["ok"].as_bool() {
            Some(true) => format!(
                "{} (views {})",
                addon["path"].as_str().unwrap_or(""),
                addon["views"].as_str().unwrap_or("?")
            ),
            _ => addon["reason"]
                .as_str()
                .unwrap_or("does not load")
                .to_string(),
        },
        "install the package for this platform (npm installs @origingame/engine-<platform>-<arch> through optionalDependencies), or in a checkout run `bun run build` in packages/engine",
    );
    for (key, what) in [
        ("wasm", "wasm"),
        ("publishWasm", "wasm-publish"),
        ("publishWasm2d", "wasm-publish-2d"),
        ("lanesWasm", "wasm-lanes"),
        ("lanesWasm2d", "wasm-lanes-2d"),
    ] {
        let w = &report[key];
        line(
            what,
            w["ok"].as_bool().unwrap_or(false),
            match w["bytes"].as_u64() {
                Some(b) => format!("{} ({b} bytes)", w["path"].as_str().unwrap_or("")),
                None => format!("no {}", w["path"].as_str().unwrap_or("wasm")),
            },
            "reinstall @origingame/engine, or in a checkout run `bun run build` in packages/engine",
        );
    }
    // No renderer is a supported configuration (the platform's engine and
    // probe sandboxes carry none): reported, never counted as a problem.
    let r = &report["renderer"];
    match r["ok"].as_bool() {
        Some(true) => println!(
            "ok       {:<13} {} (a 64x64 frame headless in {} ms)",
            "renderer",
            r["tier"].as_str().unwrap_or("?"),
            r["ms"]
        ),
        _ => {
            println!(
                "none     {:<13} {}",
                "renderer",
                r["reason"].as_str().unwrap_or("no adapter")
            );
            println!(
                "         without one: og run, og test and the probe run the same; a screenshot is a frame unchecked and capture answers null; frames are verified where there is a GPU (Metal or Vulkan)"
            );
        }
    }
    problems
}
