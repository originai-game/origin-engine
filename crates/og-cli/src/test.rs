//! `og test [project] [--json]`: the project's bun tests (`*.test.ts`), then
//! each playtest script `tests/*.og.ts` through `og run --headless
//! --commands` in its own process (`packages/engine/src/node/playtest-run.ts`).
//! Fails when anything fails, and when there was nothing to run.

use crate::project::{self, Project};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

pub(crate) fn files(dir: &Path, suffix: &str, out: &mut Vec<PathBuf>) {
    let Ok(read) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = read.flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if p.is_dir() {
            if !matches!(name, "node_modules" | ".og" | "og-out" | "dist" | "cooked")
                && !name.starts_with('.')
            {
                files(&p, suffix, out);
            }
        } else if name.ends_with(suffix) {
            out.push(p);
        }
    }
}

pub fn run(args: &[&str]) -> ExitCode {
    let mut project = ".";
    let mut json = false;
    for a in args {
        match *a {
            "--json" => json = true,
            a if a.starts_with('-') => {
                eprintln!("og test: unknown option `{a}`. usage: og test [project] [--json]");
                return ExitCode::from(2);
            }
            a => project = a,
        }
    }
    let p = match Project::open(Path::new(project)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("og test: {e}");
            return ExitCode::FAILURE;
        }
    };
    let Some(package) = project::engine_package(Some(&p.root)) else {
        eprintln!("og test: {}", project::NO_ENGINE);
        return ExitCode::FAILURE;
    };
    let mut unit = Vec::new();
    files(&p.root, ".test.ts", &mut unit);
    let mut playtests = Vec::new();
    files(&p.root.join("tests"), ".og.ts", &mut playtests);
    if unit.is_empty() && playtests.is_empty() {
        eprintln!(
            "og test: {} has no tests: add tests/*.og.ts playtests (see the template's tests/smoke.og.ts) or *.test.ts files",
            p.root.display()
        );
        return ExitCode::FAILURE;
    }
    let mut failed = 0;
    if !unit.is_empty() {
        println!("og test: bun test ({} file(s))", unit.len());
        match Command::new("bun")
            .arg("test")
            .args(&unit)
            .current_dir(&p.root)
            .status()
        {
            Ok(s) if s.success() => {}
            Ok(_) => failed += 1,
            Err(e) => {
                eprintln!("og test: {} ({e})", project::NO_BUN);
                return ExitCode::FAILURE;
            }
        }
    }
    if !playtests.is_empty() {
        let bundle = match crate::run::prepare(&p, &package) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("og test: {e}");
                return ExitCode::FAILURE;
            }
        };
        for t in &playtests {
            let mut cmd = project::bun(&package, "src/node/playtest-run.ts");
            cmd.arg("--file").arg(t).arg("--entry").arg(&bundle);
            if p.manifest {
                cmd.arg("--root").arg(&p.root);
            }
            if json {
                cmd.arg("--json");
            }
            match cmd.status() {
                Ok(s) if s.success() => {}
                Ok(_) => failed += 1,
                Err(e) => {
                    eprintln!("og test: {} ({e})", project::NO_BUN);
                    return ExitCode::FAILURE;
                }
            }
        }
    }
    let total = usize::from(!unit.is_empty()) + playtests.len();
    if failed == 0 {
        println!("og test: {total} suite(s) passed");
        ExitCode::SUCCESS
    } else {
        println!("og test: {failed} of {total} suite(s) failed");
        ExitCode::FAILURE
    }
}
