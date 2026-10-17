//! `og run [project|entry] [--headless] [--commands "t:action[:args];..."] [--seed S]
//! [--port P] [--ticks N] [--out DIR] [--size WxH] [--setup action[:arg]]... [--json]`
//!
//! Cooks the project, bundles its entry for the native host and runs it with
//! `packages/engine/src/node/run.ts` under bun on the napi addon: the probe is
//! mounted, `--commands` are scheduled through og-probe, outputs
//! (`screenshot`, `dump_*`) are written under `--out` (default
//! `<project>/og-out`) at the probe's `Output::file_name()`, and `--port`
//! serves the probe at `ws://127.0.0.1:<port>/probe` (`og probe`). Native runs
//! are always headless: `--headless` is accepted, and without it the run is
//! paced by the wall clock instead of as fast as it can go.

use crate::project::{self, Project};
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "usage: og run [project|entry] [--headless] [--commands \"t:action[:args];...\"] [--seed S] [--port P] [--ticks N] [--out DIR] [--size WxH] [--setup action[:arg]]... [--json]";

/// What `og run` was asked.
pub struct RunArgs {
    pub project: String,
    pub headless: bool,
    pub forward: Vec<String>,
    pub out: Option<String>,
    pub port: Option<String>,
}

pub fn parse(args: &[&str]) -> Result<RunArgs, String> {
    let mut r = RunArgs {
        project: ".".into(),
        headless: false,
        forward: Vec::new(),
        out: None,
        port: None,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--headless" => r.headless = true,
            "--json" | "--no-render" => r.forward.push(args[i].into()),
            flag @ ("--ticks" | "--seed" | "--setup" | "--commands" | "--size" | "--out"
            | "--port") => {
                let Some(v) = args.get(i + 1) else {
                    return Err(format!("{flag} needs a value. {USAGE}"));
                };
                match flag {
                    "--out" => r.out = Some(v.to_string()),
                    "--port" => r.port = Some(v.to_string()),
                    _ => {
                        r.forward.push(flag.into());
                        r.forward.push(v.to_string());
                    }
                }
                i += 1;
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown option `{other}`. {USAGE}"))
            }
            other => r.project = other.to_string(),
        }
        i += 1;
    }
    Ok(r)
}

/// Cook (when the project has assets) and bundle; the bundle's path.
pub fn prepare(p: &Project, package: &Path) -> Result<std::path::PathBuf, String> {
    if p.has_assets() {
        let report = crate::cook::cook(&p.root, 0)?;
        let cooked = report
            .assets
            .iter()
            .flat_map(|a| a.outputs.iter())
            .filter(|o| o.cooked)
            .count();
        if cooked > 0 {
            eprintln!("{}", crate::cook::summary(&report));
        }
    }
    project::bundle_native(p, package)
}

pub fn run(args: &[&str]) -> ExitCode {
    let a = match parse(args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("og run: {e}");
            return ExitCode::from(2);
        }
    };
    let p = match Project::open(Path::new(&a.project)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("og run: {e}");
            return ExitCode::FAILURE;
        }
    };
    let Some(package) = project::engine_package(Some(&p.root)) else {
        eprintln!("og run: {}", project::NO_ENGINE);
        return ExitCode::FAILURE;
    };
    let bundle = match prepare(&p, &package) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("og run: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut cmd = project::bun(&package, "src/node/run.ts");
    cmd.arg("--entry").arg(&bundle).args(&a.forward);
    if p.manifest {
        cmd.arg("--root").arg(&p.root);
    }
    cmd.arg("--out").arg(
        a.out
            .clone()
            .unwrap_or_else(|| p.root.join("og-out").display().to_string()),
    );
    if let Some(port) = &a.port {
        cmd.arg("--port").arg(port);
    }
    if !a.headless {
        cmd.arg("--realtime");
    }
    match cmd.status() {
        Ok(s) => project::exit_of(s),
        Err(e) => {
            eprintln!("og run: {} ({e})", project::NO_BUN);
            ExitCode::FAILURE
        }
    }
}
