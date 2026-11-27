//! `og build [project] [--publish] [--out DIR] [--assets URL] [--content URL] [--modules cdn|local] [--json] [--no-check]`: cook,
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
//! and prints the asset part of `og style check-project` as warnings (it never
//! refuses a build for them).
//! project.ogt's `publish` block is checked here (`publish.rs`) and handed
//! over as JSON; `packages/engine/src/node/web-build.ts` writes the files
//! and prints the size table. Layout and contract: docs/web-export.md.

use crate::project::{self, Project};
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "usage: og build [project] [--publish] [--out DIR] [--assets URL] [--content URL] [--modules cdn|local] [--json] [--no-check]";

fn build_trace(event: &str) {
    if let Some(path) = std::env::var_os("OG_BUILD_TRACE")
        && let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
    {
        let at = std::time::UNIX_EPOCH
            .elapsed()
            .unwrap_or_default()
            .as_millis();
        let row = serde_json::json!({"event": event, "pid": std::process::id(), "at": at});
        let _ = file.write_all(format!("{row}\n").as_bytes());
    }
}

pub fn run(args: &[&str]) -> ExitCode {
    build_trace("cli-start");
    let mut project = ".".to_string();
    let mut publish = false;
    let mut json = false;
    let mut check = true;
    let mut out = None;
    let mut assets = None;
    let mut content = None;
    let mut modules = None;
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
            // The asset base, over project.ogt's `publish { assets }`: where
            // the platform serves this build's cooked/ (docs/web-export.md
            // §Asset base).
            // The content base, over project.ogt's `publish { content }`:
            // where og.load.data reads catalogs (guide loading §14).
            flag @ ("--assets" | "--content") => {
                let Some(a) = args.get(i + 1) else {
                    eprintln!("og build: {flag} takes a URL. {USAGE}");
                    return ExitCode::from(2);
                };
                match crate::publish::asset_base(a) {
                    Ok(a) if flag == "--assets" => assets = Some(a),
                    Ok(a) => content = Some(a),
                    Err(why) => {
                        eprintln!("og build: {flag} {why}");
                        return ExitCode::from(2);
                    }
                }
                i += 1;
            }
            // Where a publish build's split engine modules come from
            // (docs/web-export.md §Modules): the CDN (the default) or a copy
            // in the output, for an offline or self-hosted build.
            "--modules" => match args.get(i + 1) {
                Some(&m @ ("cdn" | "local")) => {
                    modules = Some(m);
                    i += 1;
                }
                _ => {
                    eprintln!("og build: --modules takes cdn or local. {USAGE}");
                    return ExitCode::from(2);
                }
            },
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
                "og build: {} is not a project (no og-project.json); og build makes a play page from a project",
                p.root.display()
            );
            return ExitCode::FAILURE;
        }
        Err(e) => {
            eprintln!("og build: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut settings = match crate::publish::settings(&p.root) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("og build: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Some(a) = assets {
        settings["publish"]["assets"] = serde_json::json!(a);
    }
    if let Some(c) = content {
        settings["publish"]["content"] = serde_json::json!(c);
    }
    if let Some(m) = modules {
        settings["publish"]["modules"] = serde_json::json!(m);
    }
    let Some(package) = project::engine_package(Some(&p.root)) else {
        eprintln!("og build: {}", project::NO_ENGINE);
        return ExitCode::FAILURE;
    };
    if let Some(n) = project::pin_notice(&p, &package) {
        eprintln!("og build: {n}");
    }
    if p.has_assets() {
        build_trace("cook-start");
        match crate::cook::cook_for(&p.root, 0, true, publish) {
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
        build_trace("cook-complete");
    }
    if check {
        build_trace("check-start");
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
    build_trace("export-spawn");
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
