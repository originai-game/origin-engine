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
        ["check", rest @ ..] => match root_option(rest) {
            Ok(root) => api_check(&root),
            Err(code) => code,
        },
        _ => {
            eprintln!(
                "og api: expected `og api gen` or `og api check`; run `og --help` for the options."
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
    let dir = root.join(og_idl::WIKI_DIR);
    let mut stale: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.is_file())
                .filter(|p| {
                    let rel = format!(
                        "{}/{}",
                        og_idl::WIKI_DIR,
                        p.file_name().unwrap_or_default().to_string_lossy()
                    );
                    !files.iter().any(|f| f.path == rel)
                })
                .collect()
        })
        .unwrap_or_default();
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

fn api_check(root: &Path) -> ExitCode {
    match og_idl::check(root) {
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
