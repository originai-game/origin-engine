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
//!
//! `og doctor --project <dir> [--json]` checks a project instead: its cook
//! report against its budgets (`doctor_project.rs`).

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

/// Where the checks go: lines on stdout, or one JSON object (`--json`).
struct Out {
    json: bool,
    checks: Vec<serde_json::Value>,
    problems: usize,
}

impl Out {
    /// One check: `status` (ok, missing, differs, broken, optional, skipped,
    /// none), what it is about, the detail and, for a problem, its fix.
    fn check(&mut self, status: &str, name: &str, detail: &str, fix: Option<String>) {
        if matches!(status, "missing" | "differs" | "broken") {
            self.problems += 1;
        }
        if self.json {
            let mut c = serde_json::json!({ "status": status, "name": name, "detail": detail });
            if let Some(f) = &fix {
                c["fix"] = serde_json::json!(f);
            }
            self.checks.push(c);
        } else {
            println!("{status:<8} {name:<13} {detail}");
            if let Some(f) = fix {
                println!("         fix: {f}");
            }
        }
    }

    fn say(&self, line: &str) {
        if !self.json {
            println!("{line}");
        }
    }
}

pub fn run(args: &[&str]) -> ExitCode {
    // `og doctor --project <dir> [--json]`: the project's diagnostics.
    if args.contains(&"--project") {
        return crate::doctor_project::run(args);
    }
    let json = args.contains(&"--json");
    if let Some(a) = args.iter().find(|a| **a != "--json") {
        eprintln!(
            "og doctor: unexpected argument `{a}`; og doctor takes --json (or --project <dir> [--json])."
        );
        return ExitCode::from(2);
    }
    let mut out = Out {
        json,
        checks: Vec::new(),
        problems: 0,
    };
    let cwd = std::env::current_dir().unwrap_or_else(|_| ".".into());
    let root = crate::find_root(&cwd);
    let toolchain = root.as_deref().and_then(rust_channel);
    let bindgen = root.as_deref().and_then(wasm_bindgen_lock);
    let tsrs = root.as_deref().and_then(tsrs_version);

    out.say(&format!("og doctor (og {})", crate::VERSION));
    match &root {
        Some(r) => {
            out.say(&format!("repository  {}", r.display()));
            out.say(&format!(
                "pinned      rust {} (rust-toolchain.toml), wasm-bindgen {} (Cargo.lock), tsrs {} (TSRS_VERSION)",
                toolchain.as_deref().unwrap_or("not pinned"),
                bindgen.as_deref().unwrap_or("not locked"),
                tsrs.as_deref().unwrap_or("not pinned")
            ));
        }
        None => out.say(&format!(
            "repository  none above {}; versions are not compared with pins",
            cwd.display()
        )),
    }

    let bun = version_of("bun", root.as_deref().unwrap_or(&cwd)).is_some();
    for tool in TOOLS {
        if tool.repo_only && root.is_none() {
            out.check(
                "skipped",
                tool.name,
                &format!("{} (only inside the engine repository)", tool.role),
                None,
            );
            continue;
        }
        let pin = match tool.pin {
            Pin::RustToolchain => toolchain.as_deref(),
            Pin::WasmBindgenLock => bindgen.as_deref(),
            Pin::TsrsVersion => {
                tsrs_check(&mut out, tool, tsrs.as_deref(), &cwd);
                continue;
            }
            Pin::None => None,
        };
        // rustup resolves rust-toolchain.toml from the working directory.
        let found = version_of(tool.name, root.as_deref().unwrap_or(&cwd));
        match (found, pin) {
            (None, _) if tool.name == "node" && bun => {
                out.check(
                    "optional",
                    tool.name,
                    "not installed; bun loads the addon",
                    None,
                );
            }
            (None, _) => out.check("missing", tool.name, tool.role, Some((tool.install)(pin))),
            (Some(v), Some(p)) if v != p => out.check(
                "differs",
                tool.name,
                &format!("{v}, pinned {p}"),
                Some((tool.install)(Some(p))),
            ),
            (Some(v), _) => out.check("ok", tool.name, &v, None),
        }
    }
    // ffmpeg: og render's video (absent, og render writes frames only).
    match crate::render::ffmpeg() {
        Some((bin, v)) => out.check(
            "ok",
            "ffmpeg",
            &format!("{v} ({}; og render encodes video)", bin.display()),
            None,
        ),
        None => out.check(
            "optional",
            "ffmpeg",
            "not found (OG_FFMPEG or PATH): og render writes frames only",
            Some("install ffmpeg (apt install ffmpeg, brew install ffmpeg) or set OG_FFMPEG for video and footage (type video)".into()),
        ),
    }
    // ffprobe: the cook reads a `type video` source's length and size with it.
    let probe = og_cook::cook::video::ffprobe_bin();
    match version_of_dash(&probe) {
        Some(v) => out.check(
            "ok",
            "ffprobe",
            &format!("{v} ({}; og cook reads footage)", probe.display()),
            None,
        ),
        None => out.check(
            "optional",
            "ffprobe",
            "not found (OG_FFPROBE or PATH): og cook refuses `type video` assets",
            None,
        ),
    }
    let engine = engine_checks(&mut out, &cwd);
    if out.json {
        let report = serde_json::json!({
            "og": crate::VERSION,
            "repository": root.as_ref().map(|r| r.display().to_string()),
            "checks": out.checks,
            "engine": engine.version.as_ref().map(|v| serde_json::json!({ "version": v, "path": engine.path })),
            "pin": engine.pin,
            "renderer": engine.renderer,
            "problems": out.problems,
            "ok": out.problems == 0,
        });
        println!("{report}");
    } else if out.problems == 0 {
        println!("all tools present.");
    } else {
        println!(
            "{} problem(s); each line above names its fix.",
            out.problems
        );
    }
    if out.problems == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// The third word of `<tool> -version`'s first line (ffmpeg's tools).
fn version_of_dash(tool: &Path) -> Option<String> {
    let out = Command::new(tool).arg("-version").output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    out.status
        .success()
        .then(|| {
            text.lines()
                .next()?
                .split_whitespace()
                .nth(2)
                .map(str::to_string)
        })
        .flatten()
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
fn tsrs_check(out: &mut Out, tool: &Tool, pin: Option<&str>, cwd: &Path) {
    let package = crate::project::engine_package(Some(cwd));
    match crate::check::locate_tsrs(package.as_deref()) {
        Ok(Some(t)) => match pin {
            Some(p) if t.package_version != p => out.check(
                "differs",
                tool.name,
                &format!("{}, pinned {p} ({})", t.package_version, t.path.display()),
                Some((tool.install)(Some(p))),
            ),
            _ => out.check(
                "ok",
                tool.name,
                &format!(
                    "{} (TypeScript {}, from {}: {})",
                    t.package_version,
                    t.version,
                    t.source,
                    t.path.display()
                ),
                None,
            ),
        },
        Ok(None) if crate::check::platform().is_none() => out.check(
            "optional",
            tool.name,
            &format!(
                "no tsrs package for this platform; og check uses tsgo {} through bun x",
                crate::check::TSGO_VERSION
            ),
            None,
        ),
        Ok(None) => out.check("missing", tool.name, tool.role, Some((tool.install)(pin))),
        Err(e) => out.check("broken", tool.name, &e, Some((tool.install)(pin))),
    }
}

/// What the engine checks found, for `--json`.
#[derive(Default)]
struct EngineReport {
    version: Option<String>,
    path: Option<String>,
    /// `{ pinned, installed, compatible }` in a project with a pin.
    pin: serde_json::Value,
    renderer: serde_json::Value,
}

/// The engine package's checks (`packages/engine/src/node/doctor.ts`): the
/// addon loads, the wasm is built, and whether a 64x64 frame renders headless.
fn engine_checks(out: &mut Out, cwd: &Path) -> EngineReport {
    let mut found = EngineReport::default();
    let Some(package) = crate::project::engine_package(Some(cwd)) else {
        out.check(
            "missing",
            "engine",
            "the engine package (@origingame/engine)",
            Some(crate::project::NO_ENGINE.into()),
        );
        return found;
    };
    let output = crate::project::bun(&package, "src/node/doctor.ts").output();
    let report: serde_json::Value = match output {
        Err(_) => {
            out.check(
                "skipped",
                "engine",
                &format!("{} (needs bun)", package.display()),
                None,
            );
            return found;
        }
        Ok(o) => match serde_json::from_slice(o.stdout.trim_ascii()) {
            Ok(v) => v,
            Err(_) => {
                out.check(
                    "broken",
                    "engine",
                    &package.display().to_string(),
                    Some(format!(
                        "reinstall @origingame/engine; its doctor printed: {}",
                        String::from_utf8_lossy(&o.stderr).trim()
                    )),
                );
                return found;
            }
        },
    };
    let installed = report["version"].as_str().map(str::to_string);
    found.version = installed.clone();
    found.path = Some(package.display().to_string());
    out.check(
        "ok",
        "engine",
        &format!(
            "{} {}",
            installed.as_deref().unwrap_or("?"),
            package.display()
        ),
        None,
    );
    // In a project: its pin against the installed package (docs/release.md
    // §3): compatible when the installed version is in the pin's minor and
    // not older (a patch is compatible by the version policy).
    if let Ok(p) = crate::project::Project::open(cwd)
        && p.manifest
    {
        match (p.engine.as_deref(), installed.as_deref()) {
            (Some(pinned), Some(inst)) => {
                let compatible = crate::project::compatible(pinned, inst);
                found.pin = serde_json::json!({ "pinned": pinned, "installed": inst, "compatible": compatible });
                if pinned == inst {
                    out.check(
                        "ok",
                        "pin",
                        &format!("project.ogt pins engine \"{pinned}\""),
                        None,
                    );
                } else if compatible {
                    out.check(
                        "ok",
                        "pin",
                        &format!("project.ogt pins engine \"{pinned}\", installed {inst}: compatible (same minor, newer); `og upgrade` moves the pin"),
                        None,
                    );
                } else {
                    out.check(
                        "differs",
                        "pin",
                        &format!("project.ogt pins engine \"{pinned}\", installed {inst}: not compatible (another minor, or older)"),
                        Some(format!("`og upgrade` moves the game to {inst} (and lists what to edit); otherwise bun add @origingame/engine@{pinned}")),
                    );
                }
            }
            (Some(pinned), None) => {
                found.pin =
                    serde_json::json!({ "pinned": pinned, "installed": null, "compatible": false });
                out.check(
                    "ok",
                    "pin",
                    &format!("project.ogt pins engine \"{pinned}\""),
                    None,
                );
            }
            (None, _) => {
                found.pin = serde_json::json!({ "pinned": null, "installed": installed, "compatible": false });
                out.check(
                    "missing",
                    "pin",
                    "project.ogt has no `engine \"<version>\"` line",
                    Some(format!(
                        "add `engine \"{}\"` to project.ogt (the version this game is written against)",
                        installed.as_deref().unwrap_or("?")
                    )),
                );
            }
        }
    }
    let addon = &report["addon"];
    let fix_addon = "install the package for this platform (npm installs @origingame/engine-<platform>-<arch> through optionalDependencies), or in a checkout run `bun run build` in packages/engine";
    if addon["ok"].as_bool().unwrap_or(false) {
        out.check(
            "ok",
            "addon",
            &format!(
                "{} (views {})",
                addon["path"].as_str().unwrap_or(""),
                addon["views"].as_str().unwrap_or("?")
            ),
            None,
        );
    } else {
        out.check(
            "missing",
            "addon",
            addon["reason"].as_str().unwrap_or("does not load"),
            Some(fix_addon.into()),
        );
    }
    for (key, what) in [
        ("wasm", "wasm"),
        ("publishWasm", "wasm-publish"),
        ("publishWasm2d", "wasm-publish-2d"),
        ("lanesWasm", "wasm-lanes"),
        ("lanesWasm2d", "wasm-lanes-2d"),
    ] {
        let w = &report[key];
        let detail = match w["bytes"].as_u64() {
            Some(b) => format!("{} ({b} bytes)", w["path"].as_str().unwrap_or("")),
            None => format!("no {}", w["path"].as_str().unwrap_or("wasm")),
        };
        if w["ok"].as_bool().unwrap_or(false) {
            out.check("ok", what, &detail, None);
        } else {
            out.check(
                "missing",
                what,
                &detail,
                Some("reinstall @origingame/engine, or in a checkout run `bun run build` in packages/engine".into()),
            );
        }
    }
    // No renderer is a supported configuration (the platform's engine and
    // probe sandboxes carry none): reported, never counted as a problem.
    let r = &report["renderer"];
    found.renderer = r.clone();
    match r["ok"].as_bool() {
        Some(true) => out.check(
            "ok",
            "renderer",
            &format!(
                "{} (a 64x64 frame headless in {} ms)",
                r["tier"].as_str().unwrap_or("?"),
                r["ms"]
            ),
            None,
        ),
        _ => {
            out.check(
                "none",
                "renderer",
                r["reason"].as_str().unwrap_or("no adapter"),
                None,
            );
            out.say("         without one: og run, og test and the probe run the same; a screenshot is a frame unchecked and capture answers null; frames are verified where there is a GPU (Metal or Vulkan)");
        }
    }
    found
}
