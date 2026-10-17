//! `og run --headless <project>`, as far as M2 goes: load the game bundle on
//! the native host and advance N fixed ticks, printing the world hash. The
//! scheduled command channel (`--commands`) and windowed runs arrive with
//! the probe (M5).
//!
//! `<project>` is a game entry file (`.ts`, `.js`), or a directory whose
//! `project.ogt` names one (`entry "src/game.ts"`; `game.ts` when absent).
//! The runner is `packages/engine/src/node/run.ts` of the engine package
//! (`OG_ENGINE_PACKAGE`, else `packages/engine` of the repository `og` runs
//! in), under bun.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "usage: og run --headless <project> [--ticks N] [--seed S] [--setup action[:arg...]]... [--json]";

pub fn run(args: &[&str]) -> ExitCode {
    let mut headless = false;
    let mut project = None;
    let mut forward: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--headless" => headless = true,
            "--json" => forward.push("--json".into()),
            flag @ ("--ticks" | "--seed" | "--setup") => {
                let Some(v) = args.get(i + 1) else {
                    eprintln!("og run: {flag} needs a value. {USAGE}");
                    return ExitCode::from(2);
                };
                forward.push(flag.into());
                forward.push(v.to_string());
                i += 1;
            }
            "--commands" => {
                eprintln!("og run: --commands is the probe's scheduled command channel and lands with M5; use --setup <action> to run a registered action before the first tick. Nothing was run.");
                return ExitCode::from(2);
            }
            other if other.starts_with('-') => {
                eprintln!("og run: unknown option `{other}`. {USAGE}");
                return ExitCode::from(2);
            }
            other => project = Some(other.to_string()),
        }
        i += 1;
    }
    if !headless {
        eprintln!(
            "og run: only --headless runs in M2 (a windowed run needs the M5 shell). {USAGE}"
        );
        return ExitCode::from(2);
    }
    let Some(project) = project else {
        eprintln!("og run: no project. {USAGE}");
        return ExitCode::from(2);
    };
    let entry = match entry_of(Path::new(&project)) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("og run: {e}");
            return ExitCode::FAILURE;
        }
    };
    let Some(package) = engine_package() else {
        eprintln!("og run: no engine package: set OG_ENGINE_PACKAGE to packages/engine of an origin-engine checkout, or run inside one");
        return ExitCode::FAILURE;
    };
    let runner = package.join("src/node/run.ts");
    let status = Command::new("bun")
        .arg(&runner)
        .arg("--entry")
        .arg(&entry)
        .args(&forward)
        .status();
    match status {
        Ok(s) if s.success() => ExitCode::SUCCESS,
        Ok(s) => ExitCode::from(s.code().map_or(1, |c| c.clamp(1, 255) as u8)),
        Err(e) => {
            eprintln!("og run: cannot start bun ({e}); install it: curl -fsSL https://bun.sh/install | bash");
            ExitCode::FAILURE
        }
    }
}

fn entry_of(project: &Path) -> Result<PathBuf, String> {
    if project.is_file() {
        return Ok(project.to_path_buf());
    }
    if !project.is_dir() {
        return Err(format!(
            "{} is neither a game entry nor a project directory",
            project.display()
        ));
    }
    let manifest = project.join("project.ogt");
    let mut entry = PathBuf::from("game.ts");
    if manifest.is_file() {
        let text = std::fs::read_to_string(&manifest)
            .map_err(|e| format!("cannot read {}: {e}", manifest.display()))?;
        let doc = og_core::ogt::parse(&text)
            .map_err(|e| format!("{}: {}", manifest.display(), e.reason()))?;
        if let Some(e) = doc.body.get("entry") {
            entry = PathBuf::from(
                e.string(0)
                    .map_err(|_| format!("{}: `entry` takes a quoted path", manifest.display()))?,
            );
        }
    }
    let path = project.join(&entry);
    if !path.is_file() {
        return Err(format!(
            "{}: the game entry {} does not exist",
            project.display(),
            entry.display()
        ));
    }
    Ok(path)
}

fn engine_package() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("OG_ENGINE_PACKAGE") {
        return Some(PathBuf::from(p));
    }
    let cwd = std::env::current_dir().ok()?;
    let root = crate::find_root(&cwd).or_else(|| {
        let exe = std::env::current_exe().ok()?;
        crate::find_root(&exe)
    })?;
    let pkg = root.join("packages/engine");
    pkg.join("src/node/run.ts").is_file().then_some(pkg)
}
