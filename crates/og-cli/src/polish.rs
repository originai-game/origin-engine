//! `og polish [project] [--out DIR] [--frames N] [--json]`: the project's
//! playtests played headless and measured against og-taste and og-feel
//! (packages/engine/src/node/polish.ts), into DIR/report.md (default
//! `<project>/og-out/polish`) with up to N frames (default 8, half size),
//! the scores and a fix per finding. Refuses a project without playtests: polish measures what a
//! playtest plays. The cook's warnings (a locale's missing string keys)
//! join the report.

use crate::project::{self, Project};
use std::path::Path;
use std::process::{Command, ExitCode};

const USAGE: &str = "usage: og polish [project] [--out DIR] [--frames N] [--json]";

pub fn run(args: &[&str]) -> ExitCode {
    let mut project = ".";
    let mut out: Option<&str> = None;
    let mut frames = "8";
    let mut json = false;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--json" => json = true,
            "--out" | "--frames" if i + 1 < args.len() => {
                if args[i] == "--out" {
                    out = Some(args[i + 1]);
                } else {
                    frames = args[i + 1];
                }
                i += 1;
            }
            a if a.starts_with('-') => {
                eprintln!("og polish: unknown option `{a}`. {USAGE}");
                return ExitCode::from(2);
            }
            a => project = a,
        }
        i += 1;
    }
    if frames.parse::<u32>().is_err() {
        eprintln!("og polish: --frames takes a whole number. {USAGE}");
        return ExitCode::from(2);
    }
    let p = match Project::open(Path::new(project)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("og polish: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut playtests = Vec::new();
    crate::test::files(&p.root.join("tests"), ".og.ts", &mut playtests);
    if playtests.is_empty() {
        eprintln!(
            "og polish: {} has no playtests: polish measures what playtests play, so add tests/*.og.ts (see the template's tests/smoke.og.ts) and run og polish again",
            p.root.display()
        );
        return ExitCode::FAILURE;
    }
    let Some(package) = project::engine_package(Some(&p.root)) else {
        eprintln!("og polish: {}", project::NO_ENGINE);
        return ExitCode::FAILURE;
    };
    let mut warnings: Vec<String> = Vec::new();
    if p.has_assets() {
        match crate::cook::cook(&p.root, 0) {
            Ok(r) => warnings = r.warnings,
            Err(e) => {
                eprintln!("og polish: {e}");
                return ExitCode::FAILURE;
            }
        }
    }
    let bundle = match project::bundle_native(&p, &package) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("og polish: {e}");
            return ExitCode::FAILURE;
        }
    };
    let out = out.map_or_else(|| p.root.join("og-out").join("polish"), |o| {
        let o = Path::new(o);
        if o.is_absolute() {
            o.to_path_buf()
        } else {
            std::env::current_dir().unwrap_or_default().join(o)
        }
    });
    if let Err(e) = std::fs::create_dir_all(&out) {
        eprintln!("og polish: {}: {e}", out.display());
        return ExitCode::FAILURE;
    }
    let warnings_file = out.join(".cook-warnings.json");
    let text = serde_json::to_string(&warnings).unwrap_or_else(|_| "[]".into());
    if let Err(e) = std::fs::write(&warnings_file, text) {
        eprintln!("og polish: {}: {e}", warnings_file.display());
        return ExitCode::FAILURE;
    }
    let mut cmd: Command = project::bun(&package, "src/node/polish.ts");
    cmd.arg("--entry")
        .arg(&bundle)
        .arg("--root")
        .arg(&p.root)
        .arg("--out")
        .arg(&out)
        .arg("--warnings")
        .arg(&warnings_file)
        .arg("--frames")
        .arg(frames);
    if json {
        cmd.arg("--json");
    }
    cmd.args(&playtests);
    let status = cmd.status();
    let _ = std::fs::remove_file(&warnings_file);
    match status {
        Ok(s) if s.success() => ExitCode::SUCCESS,
        Ok(_) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("og polish: {} ({e})", project::NO_BUN);
            ExitCode::FAILURE
        }
    }
}
