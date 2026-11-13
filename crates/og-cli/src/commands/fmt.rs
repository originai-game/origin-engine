//! `og fmt [path…] [--check] [--json]`: a project's `.ogt` files in the
//! canonical layout, comments kept (og-core's `ogt::format`): two spaces a
//! level, single spaces between tokens, canonical strings, one blank line
//! where there were several. The order of entries is data and stays.
//!
//! A path is a file, a directory (every `.ogt` under it) or a project
//! (its `project.ogt` and `assets/`); none is the current directory. An
//! installed pack (a directory with an `extension.json`) is its pack's
//! and is not walked; a file named outright is always formatted. An `.ogt`
//! whose body has lines of its own (a style, a UI document, a 2D effect) is
//! skipped. `--check` writes nothing and exits 1 when a file would change.

use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: og fmt [path…] [--check] [--json]";

/// Every `.ogt` under `dir`, in name order. Never through a directory
/// symlink, a dot directory or an installed pack.
fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let (path, name) = (e.path(), e.file_name().to_string_lossy().to_string());
        let ty = e.file_type()?;
        if name.starts_with('.') || ty.is_symlink() {
            continue;
        }
        if ty.is_dir() {
            if !path.join("extension.json").is_file() {
                walk(&path, out)?;
            }
        } else if name.ends_with(".ogt") {
            out.push(path);
        }
    }
    Ok(())
}

fn files(paths: &[&str]) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    for p in paths {
        let path = Path::new(p);
        let io = |e: std::io::Error| format!("{}: {e}", path.display());
        if path.is_file() {
            out.push(path.to_path_buf());
        } else if path.join("project.ogt").is_file() {
            out.push(path.join("project.ogt"));
            if path.join("assets").is_dir() {
                walk(&path.join("assets"), &mut out).map_err(io)?;
            }
        } else if path.is_dir() {
            walk(path, &mut out).map_err(io)?;
        } else {
            return Err(format!("{}: no such file or directory", path.display()));
        }
    }
    Ok(out)
}

/// `Ok(Some(text))` when the file's canonical layout differs from it,
/// `Ok(None)` when it is canonical already, or not `.ogt`'s lines to format.
fn formatted(text: &str) -> Result<Option<String>, String> {
    let header = og_core::ogt::Document::parse_header(text).map_err(|e| e.to_string())?;
    if og_cook::cook::own_lines(&header.ty) {
        return Ok(None);
    }
    let out = og_core::ogt::format(text).map_err(|e| e.to_string())?;
    Ok((out != text).then_some(out))
}

pub fn run(args: &[&str]) -> ExitCode {
    let (mut check, mut json_output, mut paths) = (false, false, Vec::new());
    for a in args {
        match *a {
            "--check" => check = true,
            "--json" => json_output = true,
            flag if flag.starts_with('-') => {
                eprintln!("og fmt: unknown option {flag}\n{USAGE}");
                return ExitCode::from(2);
            }
            path => paths.push(path),
        }
    }
    if paths.is_empty() {
        paths.push(".");
    }
    let files = match files(&paths) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("og fmt: {e}");
            return ExitCode::from(2);
        }
    };
    let (mut changed, mut errors) = (Vec::new(), Vec::new());
    for path in &files {
        let name = path.display().to_string();
        let result = std::fs::read(path)
            .map_err(|e| e.to_string())
            .and_then(|b| String::from_utf8(b).map_err(|_| "not UTF-8 text".to_string()))
            .and_then(|text| formatted(&text))
            .and_then(|out| match out {
                Some(out) if !check => std::fs::write(path, out)
                    .map(|()| true)
                    .map_err(|e| e.to_string()),
                out => Ok(out.is_some()),
            });
        match result {
            Ok(true) => changed.push(name),
            Ok(false) => {}
            Err(e) => errors.push((name, e)),
        }
    }
    if json_output {
        println!(
            "{}",
            json!({
                "files": files.len(),
                "check": check,
                "changed": changed,
                "errors": errors.iter().map(|(file, error)| json!({ "file": file, "error": error })).collect::<Vec<_>>(),
            })
        );
    } else {
        for (file, e) in &errors {
            eprintln!("{file}: {e}");
        }
        for file in &changed {
            println!("{file}");
        }
        eprintln!(
            "og fmt: {} file(s), {} {}{}",
            files.len(),
            changed.len(),
            if check { "not formatted" } else { "formatted" },
            if errors.is_empty() {
                String::new()
            } else {
                format!(", {} not read", errors.len())
            }
        );
    }
    if errors.is_empty() && (!check || changed.is_empty()) {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
