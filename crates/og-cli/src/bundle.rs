//! `og build [project] [--publish] [--out DIR] [--json]`: cook, then the play
//! page's files in `<project>/dist` (or `publish.out` of project.ogt): the
//! engine Worker bundle `game.js` (bun, the game with the runtime), the page
//! (`index.html`, `play.js`), the web host runtime (`origin-engine.js`,
//! `origin-engine_bg.wasm`), the Basis transcoder and `cooked/`. `--publish`
//! takes the `publish` build of the web host, whose probe keeps only the
//! player-facing members (hello, tick, state, pick, entity, input, capture),
//! and minifies the bundle. Prints the size table
//! (`packages/engine/src/node/web-build.ts` does the copying).

use crate::project::{self, Project};
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "usage: og build [project] [--publish] [--out DIR] [--json]";

pub fn run(args: &[&str]) -> ExitCode {
    let mut project = ".".to_string();
    let mut publish = false;
    let mut json = false;
    let mut out = None;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--publish" => publish = true,
            "--json" => json = true,
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
    let Some(package) = project::engine_package(Some(&p.root)) else {
        eprintln!("og build: {}", project::NO_ENGINE);
        return ExitCode::FAILURE;
    };
    if p.has_assets() {
        match crate::cook::cook(&p.root, 0) {
            Ok(r) => {
                if !json {
                    println!("{}", crate::cook::summary(&r));
                }
            }
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::FAILURE;
            }
        }
    }
    let out = out.unwrap_or_else(|| p.root.join(&p.out).display().to_string());
    let mut cmd = project::bun(&package, "src/node/web-build.ts");
    cmd.arg("--project")
        .arg(&p.root)
        .arg("--entry")
        .arg(&p.entry)
        .arg("--out")
        .arg(&out);
    if publish {
        cmd.arg("--publish");
    }
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
