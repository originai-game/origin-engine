//! `og explore [project] [--seconds S] [--ticks N] [--memory MB] [--jobs N]
//! [--seed S] [--horizon T] [--out DIR] [--web auto|on|off] [--first] [--no-shrink] [--json]`
//!
//! A search of the game's states for invariant violations and for the
//! `reachable`/`sometimes`/`each` sites its playtests do not reach: the game
//! is prepared as `og run` prepares it (cook, type-check, bundle), then
//! `packages/engine/src/node/explore.ts` searches, shrinks each finding to a
//! minimal input trace, replays it from tick 0 in this process, in a fresh
//! `og run` and on the web host, and writes report.json, a repro and a
//! bun:test playtest per finding into --out (default <project>/og-out/explore).
//! The guide is `crates/og-api/guides/explore.md`. Exit 0 (whatever was
//! found: the report says it), 1 a failure, 2 a usage error.

use crate::project::{self, Project};
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "usage: og explore [project] [--seconds S] [--ticks N] [--memory MB] [--jobs N] [--seed S] [--horizon T] [--out DIR] [--web auto|on|off] [--first] [--no-shrink] [--json]";

pub fn run(args: &[&str]) -> ExitCode {
    let mut project = ".".to_string();
    let mut out: Option<std::path::PathBuf> = None;
    let mut forward: Vec<String> = Vec::new();
    let mut positional = 0;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            flag @ ("--json" | "--no-shrink" | "--first") => forward.push(flag.into()),
            flag @ ("--seconds" | "--ticks" | "--memory" | "--jobs" | "--seed" | "--horizon"
            | "--web" | "--out") => {
                let Some(v) = args.get(i + 1) else {
                    eprintln!("og explore: {flag} needs a value. {USAGE}");
                    return ExitCode::from(2);
                };
                if flag == "--out" {
                    match std::path::absolute(v) {
                        Ok(p) => out = Some(p),
                        Err(e) => {
                            eprintln!("og explore: --out {v}: {e}");
                            return ExitCode::from(2);
                        }
                    }
                } else {
                    forward.extend([flag.to_string(), v.to_string()]);
                }
                i += 1;
            }
            other if other.starts_with('-') => {
                eprintln!("og explore: unknown option `{other}`. {USAGE}");
                return ExitCode::from(2);
            }
            other => {
                positional += 1;
                if positional > 1 {
                    eprintln!("og explore: one project at most. {USAGE}");
                    return ExitCode::from(2);
                }
                project = other.to_string();
            }
        }
        i += 1;
    }
    let p = match Project::open(Path::new(&project)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("og explore: {e}");
            return ExitCode::FAILURE;
        }
    };
    let Some(package) = project::engine_package(Some(&p.root)) else {
        eprintln!("og explore: {}", project::NO_ENGINE);
        return ExitCode::FAILURE;
    };
    if let Some(n) = project::pin_notice(&p, &package) {
        eprintln!("og explore: {n}");
    }
    let json = forward.iter().any(|f| f == "--json");
    let bundle = match crate::run::prepare(&p, &package, true, json) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("og explore: {e}");
            return ExitCode::FAILURE;
        }
    };
    let out = out.unwrap_or_else(|| p.root.join("og-out").join("explore"));
    let mut cmd = project::bun(&package, "src/node/explore.ts");
    cmd.arg("--entry")
        .arg(&*bundle)
        .arg("--out")
        .arg(&out)
        .arg("--project")
        .arg(if p.manifest { &p.root } else { &p.entry })
        .args(&forward);
    if p.manifest {
        cmd.arg("--root").arg(&p.root);
    }
    // The repros replay in a fresh `og run` of this binary.
    if let Ok(exe) = std::env::current_exe() {
        cmd.env("OG_EXPLORE_OG", exe);
    }
    match cmd.status() {
        Ok(s) => match s.code() {
            Some(c) => ExitCode::from(c.clamp(0, 255) as u8),
            None => {
                eprintln!("og explore: the search process was killed ({s})");
                ExitCode::FAILURE
            }
        },
        Err(e) => {
            eprintln!("og explore: {} ({e})", project::NO_BUN);
            ExitCode::FAILURE
        }
    }
}
