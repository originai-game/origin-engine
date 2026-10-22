//! `og test [project] [--filter <path>] [--json] [--no-check]`: the project's type check
//! (crate::check), its bun tests (`*.test.ts`), then each playtest script
//! `tests/*.og.ts` through `og run --headless --commands` in its own process
//! (`packages/engine/src/node/playtest-run.ts`). Fails when anything fails,
//! and when there was nothing to run. A type error fails the run and the
//! playtests do not run (there is no bundle); the bun tests still do.
//! `--json` prints the `check` event first and a `test` event last, with
//! the check's `{ok, ms, errors}`.
//! `--filter` selects suite files by a case-sensitive substring of their
//! project-relative path (with `/` separators), before any tests execute.
//!
//! Without a renderer (the platform's engine and probe sandboxes,
//! `OG_RENDERER=none`) a playtest's screenshots are frames unchecked: each
//! playtest lists its own, the summary counts them, and they never change
//! the exit status.

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
    let mut check = true;
    let mut filter = None;
    let mut args = args.iter();
    while let Some(a) = args.next() {
        match *a {
            "--json" => json = true,
            "--no-check" => check = false,
            "--filter" => {
                let Some(value) = args.next().filter(|v| !v.is_empty() && !v.starts_with('-'))
                else {
                    eprintln!("og test: --filter needs a nonempty suite path substring");
                    return ExitCode::from(2);
                };
                filter = Some(*value);
            }
            a if a.starts_with('-') => {
                eprintln!(
                    "og test: unknown option `{a}`. usage: og test [project] [--filter <path>] [--json] [--no-check]"
                );
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
    if let Some(n) = project::pin_notice(&p, &package) {
        eprintln!("og test: {n}");
    }
    let mut unit = Vec::new();
    files(&p.root, ".test.ts", &mut unit);
    let mut playtests = Vec::new();
    files(&p.root.join("tests"), ".og.ts", &mut playtests);
    if let Some(filter) = filter {
        let matches = |path: &PathBuf| {
            path.strip_prefix(&p.root)
                .expect("discovered tests are inside the project")
                .to_string_lossy()
                .replace('\\', "/")
                .contains(filter)
        };
        unit.retain(matches);
        playtests.retain(matches);
        if unit.is_empty() && playtests.is_empty() {
            eprintln!("og test: no suite paths match --filter `{filter}`");
            return ExitCode::FAILURE;
        }
    }
    if unit.is_empty() && playtests.is_empty() {
        eprintln!(
            "og test: {} has no tests: add tests/*.og.ts playtests (see the template's tests/smoke.og.ts) or *.test.ts files",
            p.root.display()
        );
        return ExitCode::FAILURE;
    }
    let mut failed = 0;
    let mut unchecked = 0;
    // The type check first: its failure leaves no bundle for the playtests.
    let checked = if check {
        if p.has_assets() {
            // Cook first, as og run does, so the check sees what the run would.
            if let Err(e) = crate::cook::cook(&p.root, 0) {
                eprintln!("og test: {e}");
                return ExitCode::FAILURE;
            }
        }
        match crate::check::gate(&p, &package, json) {
            Ok(r) => {
                if !r.ok() {
                    eprintln!("og test: {}", crate::check::refusal(&r));
                    failed += 1;
                }
                Some(Ok(r))
            }
            Err(e) => {
                eprintln!("og test: {e}");
                failed += 1;
                Some(Err(e))
            }
        }
    } else {
        None
    };
    let check_failed = match &checked {
        Some(Ok(r)) => !r.ok(),
        Some(Err(_)) => true,
        None => false,
    };
    if !unit.is_empty() {
        println!("og test: bun test ({} file(s))", unit.len());
        match Command::new("bun")
            .arg("test")
            .args(unit.iter().map(|p| crate::check::typescript_path(p)))
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
    if !playtests.is_empty() && check_failed {
        println!(
            "og test: the {} playtest(s) did not run: the type check failed",
            playtests.len()
        );
    } else if !playtests.is_empty() {
        let bundle = match crate::run::prepare(&p, &package, false, json) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("og test: {e}");
                return ExitCode::FAILURE;
            }
        };
        // Each playtest's result line, for the frames it left unchecked.
        let results = std::env::temp_dir().join(format!("og-test-{}", std::process::id()));
        for (i, t) in playtests.iter().enumerate() {
            let result = results.join(format!("{i}.json"));
            let mut cmd = project::bun(&package, "src/node/playtest-run.ts");
            cmd.arg("--file")
                .arg(crate::check::typescript_path(t))
                .arg("--entry")
                .arg(&bundle);
            cmd.arg("--result").arg(&result);
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
                    let _ = std::fs::remove_dir_all(&results);
                    return ExitCode::FAILURE;
                }
            }
            unchecked += unchecked_frames(&result);
        }
        let _ = std::fs::remove_dir_all(&results);
    }
    let total = usize::from(!unit.is_empty()) + playtests.len() + usize::from(check_failed);
    if json {
        println!(
            "{}",
            serde_json::json!({
                "event": "test", "ok": failed == 0, "suites": total, "failed": failed,
                "unchecked": unchecked,
                "check": match &checked {
                    Some(Ok(r)) => r.brief(),
                    // A checker that failed.
                    Some(Err(e)) => serde_json::json!({ "ok": false, "error": e }),
                    None => serde_json::Value::Null,
                },
            })
        );
    }
    let frames = if unchecked > 0 {
        format!(
            "; {unchecked} frame(s) unchecked: no renderer here (frames are verified on the author's machine, in the browser-check sandbox or on an executor with a GPU)"
        )
    } else {
        String::new()
    };
    if failed == 0 {
        println!("og test: {total} suite(s) passed{frames}");
        ExitCode::SUCCESS
    } else {
        println!("og test: {failed} of {total} suite(s) failed{frames}");
        ExitCode::FAILURE
    }
}

/// How many frames a playtest's result line (`playtest-run.ts --result`)
/// reports unchecked; 0 when it wrote none.
fn unchecked_frames(result: &Path) -> usize {
    std::fs::read(result)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .and_then(|v| v["unchecked"].as_array().map(Vec::len))
        .unwrap_or(0)
}
