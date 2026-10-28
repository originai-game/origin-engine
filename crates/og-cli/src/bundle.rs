//! `og build [project] [--publish] [--out DIR] [--json] [--no-check]`: cook,
//! type-check (crate::check; a type error builds nothing, and a publish
//! build is always checked: `--publish` refuses `--no-check`), then export
//! the game into `<project>/dist` (or `publish.out` of project.ogt): the
//! player shell (`index.html`, `play.js`), the engine Worker bundle
//! `game.js`, the web host (`origin-engine.js`, `origin-engine_bg.wasm`),
//! the Basis transcoder, `cooked/`, the icons, `og-build.json`, `_headers`
//! and, with the project's isolation, `origingame.runtime.json`.
//! `--publish` takes the `publish` build of the web host, whose probe keeps
//! only the player-facing members (hello, tick, state, pick, entity, input,
//! capture), minifies, and adds the PWA (`manifest.webmanifest`, `sw.js`),
//! and prints the asset part of `og style check` as warnings (it never
//! refuses a build for them).
//! project.ogt's `publish` block is checked here (`publish.rs`) and handed
//! over as JSON; `packages/engine/src/node/web-build.ts` writes the files
//! and prints the size table. Layout and contract: docs/web-export.md.

use crate::project::{self, Project};
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "usage: og build [project] [--publish] [--out DIR] [--json] [--no-check]";

pub fn run(args: &[&str]) -> ExitCode {
    let mut project = ".".to_string();
    let mut publish = false;
    let mut json = false;
    let mut check = true;
    let mut out = None;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--publish" => publish = true,
            "--json" => json = true,
            "--no-check" => check = false,
            "--out" => {
                let Some(o) = args.get(i + 1) else {
                    eprintln!("og build: --out takes a directory. {USAGE}");
                    return ExitCode::from(2);
                };
                out = Some(o.to_string());
                i += 1;
            }
            a if a.starts_with('-') => {
                eprintln!("og build: unknown option `{a}`. {USAGE}");
                return ExitCode::from(2);
            }
            a => project = a.to_string(),
        }
        i += 1;
    }
    if publish && !check {
        eprintln!(
            "og build: --publish builds what players get and is always type-checked; drop --no-check"
        );
        return ExitCode::from(2);
    }
    let p = match Project::open(Path::new(&project)) {
        Ok(p) if p.manifest => p,
        Ok(p) => {
            eprintln!(
                "og build: {} is not a project (no project.ogt); og build makes a play page from a project",
                p.root.display()
            );
            return ExitCode::FAILURE;
        }
        Err(e) => {
            eprintln!("og build: {e}");
            return ExitCode::FAILURE;
        }
    };
    let settings = match crate::publish::settings(&p.root.join("project.ogt")) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("og build: {e}");
            return ExitCode::FAILURE;
        }
    };
    if publish && let Err(e) = crate::publish::refuse_3d_world(&p.root, &settings) {
        eprintln!("og build --publish: {e}");
        return ExitCode::FAILURE;
    }
    let Some(package) = project::engine_package(Some(&p.root)) else {
        eprintln!("og build: {}", project::NO_ENGINE);
        return ExitCode::FAILURE;
    };
    if let Some(n) = project::pin_notice(&p, &package) {
        eprintln!("og build: {n}");
    }
    if p.has_assets() {
        match crate::cook::cook(&p.root, 0, true) {
            Ok(r) => {
                if !json {
                    println!("{}", crate::cook::summary(&r));
                }
                // The style against the look's contract (docs/art.md §2.5):
                // warnings, never a refusal.
                if publish {
                    for w in crate::style_check::publish_warnings(&p.root, &r) {
                        eprintln!("og build: warning: {w}");
                    }
                }
            }
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::FAILURE;
            }
        }
    }
    if check && let Err(e) = crate::check::require(&p, &package, json) {
        eprintln!("og build: {e}");
        return ExitCode::FAILURE;
    }
    let out = out.unwrap_or_else(|| p.root.join(&p.out).display().to_string());
    let mut cmd = web_build(&p, &package, &settings, Path::new(&out), publish);
    if json {
        cmd.arg("--json");
    }
    match cmd.status() {
        Ok(s) => project::exit_of(s),
        Err(e) => {
            eprintln!("og build: {} ({e})", project::NO_BUN);
            ExitCode::FAILURE
        }
    }
}

/// `web-build.ts` writing the export of `p` (cooked already) into `out`,
/// with project.ogt's checked `settings` (`og build`, `og run --device`).
pub fn web_build(
    p: &Project,
    package: &Path,
    settings: &serde_json::Value,
    out: &Path,
    publish: bool,
) -> std::process::Command {
    let mut cmd = project::bun(package, "src/node/web-build.ts");
    cmd.arg("--project")
        .arg(&p.root)
        .arg("--entry")
        .arg(&p.entry)
        .arg("--out")
        .arg(out)
        .arg("--settings")
        .arg(settings.to_string());
    if publish {
        cmd.arg("--publish");
    }
    cmd
}
