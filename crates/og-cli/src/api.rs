//! `og api gen [--wiki]` and `og api check`: the IDL's generated files
//! (crates/og-api/README.md), including the wiki the knowledge package ships
//! (`packages/knowledge/wiki/`).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// `og api gen [--wiki] [--root <dir>]` or `og api check [--root <dir>]`.
pub fn run(args: &[&str]) -> ExitCode {
    match args {
        ["gen", rest @ ..] => {
            let wiki = rest.contains(&"--wiki");
            let rest: Vec<&str> = rest.iter().copied().filter(|a| *a != "--wiki").collect();
            match root_option(&rest) {
                Ok(root) => api_gen(&root, wiki),
                Err(code) => code,
            }
        }
        [cmd @ ("find" | "show"), rest @ ..] => crate::api_lookup::run(cmd, rest),
        ["check", rest @ ..] => {
            let list = rest.contains(&"--budget");
            let rest: Vec<&str> = rest.iter().copied().filter(|a| *a != "--budget").collect();
            match root_option(&rest) {
                Ok(root) => {
                    let drift = api_check(&root);
                    let budget = api_budget(&root, list);
                    if drift == ExitCode::SUCCESS { budget } else { drift }
                }
                Err(code) => code,
            }
        }
        _ => {
            eprintln!(
                "og api: expected `og api find`, `og api show`, `og api gen` or `og api check`; run `og --help` for the options."
            );
            ExitCode::from(2)
        }
    }
}

/// `--root <dir>`, or the nearest ancestor of the current directory that holds the IDL.
fn root_option(rest: &[&str]) -> Result<PathBuf, ExitCode> {
    match rest {
        [] => {
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            crate::find_root(&cwd).ok_or_else(|| {
                eprintln!(
                    "og api: no {} above {}; run inside the engine repository or pass --root <dir>.",
                    og_idl::IDL_PATH,
                    cwd.display()
                );
                ExitCode::from(2)
            })
        }
        ["--root", dir] => Ok(PathBuf::from(dir)),
        _ => {
            eprintln!(
                "og api: unexpected arguments `{}`; the options are --root <dir> (and --wiki for `og api gen`).",
                rest.join(" ")
            );
            Err(ExitCode::from(2))
        }
    }
}

/// Writes every generated file, or with `wiki` only the wiki pages, and
/// removes wiki pages the IDL no longer generates (a deleted module's page).
fn api_gen(root: &Path, wiki: bool) -> ExitCode {
    // The written guides (the component catalogue from og-ui's schema, the
    // effect packs from their manifests); the wiki reads them with the others.
    for (path, text) in written_guides(root) {
        let file = root.join(path);
        if std::fs::read_to_string(&file).ok().as_deref() != Some(text.as_str()) {
            if let Err(e) = std::fs::write(&file, &text) {
                eprintln!("og api gen: cannot write {}: {e}", file.display());
                return ExitCode::FAILURE;
            }
            println!("wrote {path}");
        }
    }
    let api = match og_idl::read(&root.join(og_idl::IDL_PATH)) {
        Ok(api) => api,
        Err(e) => {
            eprint!("og api gen: {e}");
            return ExitCode::FAILURE;
        }
    };
    let files = if wiki {
        og_idl::generate_wiki(&api)
    } else {
        og_idl::generate(&api)
    };
    let written = match og_idl::write(root, &files) {
        Ok(written) => written,
        Err(e) => {
            eprintln!("og api gen: cannot write under {}: {e}", root.display());
            return ExitCode::FAILURE;
        }
    };
    for path in &written {
        println!("wrote {path}");
    }
    // Unchanged files keep their modification time, so cargo rebuilds only
    // the crates whose generated Rust changed.
    println!(
        "{} of {} generated files unchanged",
        files.len() - written.len(),
        files.len()
    );
    // Pages the IDL no longer generates, at any depth (a deleted module's
    // page and its items' pages, a removed type's page).
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else {
                out.push(p);
            }
        }
    }
    let mut found = Vec::new();
    walk(&root.join(og_idl::WIKI_DIR), &mut found);
    let mut stale: Vec<PathBuf> = found
        .into_iter()
        .filter(|p| {
            let rel = p
                .strip_prefix(root)
                .unwrap_or(p)
                .to_string_lossy()
                .replace('\\', "/");
            !files.iter().any(|f| f.path == rel)
        })
        .collect();
    stale.sort();
    for p in stale {
        if let Err(e) = std::fs::remove_file(&p) {
            eprintln!(
                "og api gen: cannot remove the stale page {}: {e}",
                p.display()
            );
            return ExitCode::FAILURE;
        }
        println!("removed {}", p.display());
    }
    ExitCode::SUCCESS
}

/// The guides `og api gen` writes from other sources than the IDL, as (path, text).
fn written_guides(root: &Path) -> [(&'static str, String); 2] {
    [
        (
            og_ui::doc::catalogue::PATH,
            og_ui::doc::catalogue::markdown(),
        ),
        (
            crate::vfx_catalogue::PATH,
            crate::vfx_catalogue::markdown(root),
        ),
    ]
}

/// The description budget (`og_idl::budget`): a summary, or with `--budget`
/// every description over it; a failure when more are over than the backlog.
fn api_budget(root: &Path, list: bool) -> ExitCode {
    use og_idl::budget::{BACKLOG, FIELD, ITEM, over};
    let Ok(api) = og_idl::read(&root.join(og_idl::IDL_PATH)) else {
        return ExitCode::FAILURE;
    };
    let over = over(&api);
    let items = over.iter().filter(|o| o.budget == ITEM).count();
    if list {
        for o in &over {
            let (part, line) = api.locate(o.line);
            println!("{part}:{line}: {} has {} characters of {}", o.entry, o.chars, o.budget);
        }
    }
    println!(
        "og api check: {} description(s) over budget ({items} over {ITEM} characters for modules, functions and types; {} over {FIELD} for fields, parameters and the rest), backlog {BACKLOG}{}",
        over.len(),
        over.len() - items,
        if list || over.is_empty() { "" } else { "; `og api check --budget` lists them" }
    );
    if over.len() > BACKLOG {
        eprintln!(
            "og api check: more descriptions are over budget than the backlog of {BACKLOG}: move workflow, sizes, timings and cross-member caveats to a guide and point at it with `guide \"<guide> §N\"` (crates/og-idl/src/budget.rs states the rule)"
        );
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn api_check(root: &Path) -> ExitCode {
    let stale: Vec<og_idl::Drift> = written_guides(root)
        .into_iter()
        .filter(|(path, text)| {
            std::fs::read_to_string(root.join(path)).ok().as_deref() != Some(text.as_str())
        })
        .map(|(path, _)| og_idl::Drift::Differs {
            path: path.into(),
            line: 1,
        })
        .collect();
    match og_idl::check(root).map(|d| stale.into_iter().chain(d).collect::<Vec<_>>()) {
        Err(e) => {
            eprint!("og api check: {e}");
            ExitCode::FAILURE
        }
        Ok(drift) if drift.is_empty() => {
            println!(
                "og api check: the generated files match {}.",
                og_idl::IDL_PATH
            );
            ExitCode::SUCCESS
        }
        Ok(drift) => {
            eprintln!(
                "og api check: {} file(s) differ from what {} generates:",
                drift.len(),
                og_idl::IDL_PATH
            );
            for d in &drift {
                eprintln!("  {d}");
            }
            ExitCode::FAILURE
        }
    }
}
