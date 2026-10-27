//! `og replay record [project] --commands "<script>" | --commands-file <f>
//! [--ticks N] [--seed S] [--clock <UTC ms>] [--locale <tag>]
//! [--keyframe-every K] --out <trace.json> [--json]`
//!
//! `og replay compare <trace.json> [project] [--json]`
//!
//! Cross-version deterministic replay: `record` runs the game as `og run
//! --no-render --commands` does (cook, type-check, bundle; one tick per
//! frame) and writes a trace of the world's portable hash after every tick
//! (og-net's `Snapshot::portable_hash`), with the world's save every K
//! ticks (default 30) and at the last one; `compare` replays a trace's
//! script, seed, clock and locale with the installed engine (on the
//! trace's entry of the project, default the current directory) and
//! reports the first tick whose hash differs, with og-net's diff of the
//! first keyframe from there. `packages/engine/src/node/replay.ts` has the
//! trace's and the report's shapes. Exit 0 (compare: the same), 3 (compare:
//! diverged), 1 a failure, 2 a usage error.

use crate::project::{self, Project};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: og replay record [project] --commands \"t:action[:args];...\" | --commands-file <f> [--ticks N] [--seed S] [--clock <UTC ms>] [--locale <tag>] [--keyframe-every K] --out <trace.json> [--json]\n       og replay compare <trace.json> [project] [--json]";

/// What `og replay` was asked.
struct Args {
    project: String,
    /// `compare`'s trace (absolute).
    compare: Option<PathBuf>,
    /// `record`'s trace (absolute).
    out: Option<PathBuf>,
    forward: Vec<String>,
}

fn parse(args: &[&str]) -> Result<Args, String> {
    let (record, rest) = match args {
        ["record", rest @ ..] => (true, rest),
        ["compare", rest @ ..] => (false, rest),
        _ => return Err(USAGE.into()),
    };
    let mut a = Args {
        project: ".".into(),
        compare: None,
        out: None,
        forward: Vec::new(),
    };
    let mut commands = false;
    let mut positional = Vec::new();
    let mut i = 0;
    while i < rest.len() {
        match rest[i] {
            "--json" => a.forward.push("--json".into()),
            flag @ ("--commands" | "--commands-file" | "--ticks" | "--seed" | "--clock"
            | "--locale" | "--keyframe-every" | "--out")
                if record =>
            {
                let Some(v) = rest.get(i + 1) else {
                    return Err(format!("{flag} needs a value. {USAGE}"));
                };
                match flag {
                    "--out" => {
                        a.out = Some(std::path::absolute(v).map_err(|e| format!("--out {v}: {e}"))?)
                    }
                    "--commands-file" => {
                        let script = std::fs::read_to_string(v)
                            .map_err(|e| format!("--commands-file {v}: {e}"))?;
                        a.forward
                            .extend(["--commands".into(), script.trim().to_string()]);
                        commands = true;
                    }
                    _ => {
                        commands |= flag == "--commands";
                        a.forward.extend([flag.to_string(), v.to_string()]);
                    }
                }
                i += 1;
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown option `{other}`. {USAGE}"));
            }
            other => positional.push(other),
        }
        i += 1;
    }
    if record {
        match positional.as_slice() {
            [] => {}
            [p] => a.project = p.to_string(),
            _ => return Err(format!("one project at most. {USAGE}")),
        }
        if !commands {
            return Err(format!(
                "og replay record needs --commands or --commands-file. {USAGE}"
            ));
        }
        if a.out.is_none() {
            return Err(format!(
                "og replay record needs --out <trace.json>. {USAGE}"
            ));
        }
    } else {
        match positional.as_slice() {
            [t] => a.compare = Some(std::path::absolute(t).map_err(|e| format!("{t}: {e}"))?),
            [t, p] => {
                a.compare = Some(std::path::absolute(t).map_err(|e| format!("{t}: {e}"))?);
                a.project = p.to_string();
            }
            _ => return Err(format!("og replay compare takes a trace. {USAGE}")),
        }
    }
    Ok(a)
}

/// The trace's entry, relative to its project.
fn trace_entry(trace: &Path) -> Result<String, String> {
    let text = std::fs::read_to_string(trace).map_err(|e| format!("{}: {e}", trace.display()))?;
    let v: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", trace.display()))?;
    if v["kind"] != "og-replay" {
        return Err(format!("{} is not an og replay trace", trace.display()));
    }
    v["project"]["entry"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| format!("{} names no project entry", trace.display()))
}

pub fn run(args: &[&str]) -> ExitCode {
    let a = match parse(args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("og replay: {e}");
            return ExitCode::from(2);
        }
    };
    let mut p = match Project::open(Path::new(&a.project)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("og replay: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Some(trace) = &a.compare {
        match trace_entry(trace) {
            Ok(entry) => p.entry = p.root.join(entry),
            Err(e) => {
                eprintln!("og replay: {e}");
                return ExitCode::FAILURE;
            }
        }
    }
    let Some(package) = project::engine_package(Some(&p.root)) else {
        eprintln!("og replay: {}", project::NO_ENGINE);
        return ExitCode::FAILURE;
    };
    if let Some(n) = project::pin_notice(&p, &package) {
        eprintln!("og replay: {n}");
    }
    let bundle = match crate::run::prepare(&p, &package, true, false) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("og replay: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut cmd = project::bun(&package, "src/node/replay.ts");
    cmd.arg("--entry").arg(&bundle).args(&a.forward);
    if p.manifest {
        cmd.arg("--root").arg(&p.root);
    }
    match (&a.compare, &a.out) {
        (Some(trace), _) => {
            cmd.arg("--compare").arg(trace);
        }
        (None, Some(out)) => {
            let entry = p.entry.strip_prefix(&p.root).unwrap_or(&p.entry);
            cmd.arg("--record")
                .arg(out)
                .arg("--project-entry")
                .arg(entry.to_string_lossy().replace('\\', "/"));
        }
        (None, None) => unreachable!("parse requires --out for record"),
    }
    match cmd.status() {
        Ok(s) => match s.code() {
            Some(c) => ExitCode::from(c.clamp(0, 255) as u8),
            None => {
                eprintln!("og replay: the game process was killed ({s})");
                ExitCode::FAILURE
            }
        },
        Err(e) => {
            eprintln!("og replay: {} ({e})", project::NO_BUN);
            ExitCode::FAILURE
        }
    }
}
