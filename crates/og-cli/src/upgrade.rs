//! `og upgrade [project] [--to <version>] [--dry-run] [--json]`: moves a
//! game to another engine version (docs/release.md §3).
//!
//! The installed engine package's `changelog.json` says what each version
//! between the pin (`project.ogt`'s `engine`) and the target (`--to`, else
//! the installed version) changed: `og upgrade` writes the target into
//! project.ogt's `engine` line and package.json's `@origingame/engine`
//! dependency, applies the mechanical rewrites (`codemod`) of the breaking
//! entries it crosses to the game's sources (`src/`, `assets/` text), and
//! lists every place the other breaking entries' identifiers (`find`)
//! appear, with the change and its fix, as `og migrate three` lists what it
//! cannot rewrite. A breaking entry without identifiers is listed under
//! `review`. It does not install the package (`bun install` does, with the
//! new dependency) and never downgrades.
//!
//! Exit status: 0 done (nothing left to edit by hand), 3 done with places
//! to edit by hand (`manual`), 1 failed (nothing written), 2 usage.

use crate::project::{self, Project};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: og upgrade [project] [--to <version>] [--dry-run] [--json]";

/// The exit status of an upgrade that left places to edit by hand.
const NEEDS_EDITS: u8 = 3;

pub fn run(args: &[&str]) -> ExitCode {
    let mut dir = ".".to_string();
    let mut to: Option<String> = None;
    let (mut json_out, mut dry) = (false, false);
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--json" => json_out = true,
            "--dry-run" => dry = true,
            "--to" => {
                let Some(v) = args.get(i + 1) else {
                    eprintln!("og upgrade: --to needs a version. {USAGE}");
                    return ExitCode::from(2);
                };
                to = Some(v.to_string());
                i += 1;
            }
            a if a.starts_with('-') => {
                eprintln!("og upgrade: unknown option `{a}`. {USAGE}");
                return ExitCode::from(2);
            }
            a => dir = a.to_string(),
        }
        i += 1;
    }
    let report = upgrade(Path::new(&dir), to.as_deref(), dry);
    let status = report["status"].as_str().unwrap_or("failed").to_string();
    if json_out {
        println!("{report}");
    } else {
        print_text(&report, dry);
    }
    match status.as_str() {
        "done" => ExitCode::SUCCESS,
        "manual" => ExitCode::from(NEEDS_EDITS),
        _ => ExitCode::FAILURE,
    }
}

fn failed(reason: String, extra: Value) -> Value {
    let mut v = json!({ "status": "failed", "reason": reason });
    if let (Some(o), Value::Object(e)) = (v.as_object_mut(), extra) {
        o.extend(e);
    }
    v
}

/// The whole upgrade as its report (written unless `dry`).
fn upgrade(dir: &Path, to: Option<&str>, dry: bool) -> Value {
    let p = match Project::open(dir) {
        Ok(p) if p.manifest => p,
        Ok(_) => return failed(format!("{}: no project.ogt", dir.display()), json!({})),
        Err(e) => return failed(e, json!({})),
    };
    let base = json!({ "project": p.root.display().to_string() });
    let Some(from) = p.engine.clone() else {
        return failed(
            "project.ogt has no `engine \"<version>\"` line: add the version the game is written against first".into(),
            base,
        );
    };
    let Some(package) = project::engine_package(Some(&p.root)) else {
        return failed(project::NO_ENGINE.into(), base);
    };
    let installed = project::package_version(&package).unwrap_or_default();
    let log: Value = match std::fs::read_to_string(package.join("changelog.json"))
        .map_err(|e| e.to_string())
        .and_then(|t| serde_json::from_str(&t).map_err(|e| e.to_string()))
    {
        Ok(v) => v,
        Err(e) => {
            return failed(
                format!(
                    "the engine package at {} has no readable changelog.json ({e}); upgrade @origingame/engine to 0.6.2 or later first",
                    package.display()
                ),
                base,
            );
        }
    };
    let to = to.map(str::to_string).unwrap_or_else(|| installed.clone());
    let (Some(f), Some(t)) = (project::semver(&from), project::semver(&to)) else {
        return failed(format!("not a version: {from} or {to}"), base);
    };
    let versions = log["versions"].as_array().cloned().unwrap_or_default();
    let known = |v: &str| versions.iter().any(|e| e["version"] == v);
    if !known(&to) {
        return failed(
            format!(
                "{to} is not in the installed engine's changelog (the installed engine is {installed}): install @origingame/engine@{to} and run its og upgrade"
            ),
            base,
        );
    }
    if t < f {
        return failed(
            format!(
                "{to} is older than the pinned {from}: og upgrade never downgrades (revert the upgrade's commit instead)"
            ),
            base,
        );
    }
    // The versions crossed, oldest first: after the pin, up to the target.
    let mut crossed: Vec<&Value> = versions
        .iter()
        .filter(|e| {
            e["version"]
                .as_str()
                .and_then(project::semver)
                .is_some_and(|v| v > f && v <= t)
        })
        .collect();
    crossed.reverse();
    let kind = if f == t {
        "none"
    } else if f.0 == t.0 && f.1 == t.1 {
        "patch"
    } else {
        "minor"
    };
    let simulation = if crossed.iter().any(|e| e["simulation"] != "unchanged") {
        "changed"
    } else {
        "unchanged"
    };
    let sources = game_sources(&p.root);
    let mut texts: Vec<(PathBuf, String, bool)> = sources
        .iter()
        .filter_map(|f| {
            std::fs::read_to_string(f)
                .ok()
                .map(|t| (f.clone(), t, false))
        })
        .collect();
    let rel = |f: &Path| {
        f.strip_prefix(&p.root)
            .unwrap_or(f)
            .to_string_lossy()
            .replace('\\', "/")
    };
    let (mut codemods, mut manual, mut review) = (Vec::new(), Vec::new(), Vec::new());
    for e in &crossed {
        let version = e["version"].as_str().unwrap_or("");
        for b in e["breaking"].as_array().into_iter().flatten() {
            let member = b["member"].as_str().unwrap_or("");
            if let (Some(from_s), Some(to_s)) =
                (b["codemod"]["from"].as_str(), b["codemod"]["to"].as_str())
            {
                for (file, text, touched) in &mut texts {
                    let n = text.matches(from_s).count();
                    if n > 0 {
                        *text = text.replace(from_s, to_s);
                        *touched = true;
                        codemods.push(json!({ "version": version, "member": member, "file": rel(file), "from": from_s, "to": to_s, "count": n }));
                    }
                }
                continue;
            }
            let finds: Vec<&str> = b["find"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect();
            if finds.is_empty() {
                review.push(json!({ "version": version, "member": member, "change": b["change"], "fix": b["fix"] }));
                continue;
            }
            for (file, text, _) in &texts {
                for (n, line) in text.lines().enumerate() {
                    if finds.iter().any(|w| line.contains(w)) {
                        manual.push(json!({
                            "version": version, "member": member, "change": b["change"], "fix": b["fix"],
                            "file": rel(file), "line": n + 1, "text": line.trim(),
                        }));
                    }
                }
            }
        }
    }
    // The pin and the dependency.
    let mut edited = Vec::new();
    let mut writes: Vec<(PathBuf, String)> = Vec::new();
    if kind != "none" {
        let manifest = p.root.join("project.ogt");
        match std::fs::read_to_string(&manifest) {
            Ok(text) => match set_engine_line(&text, &to) {
                Some(t) => writes.push((manifest, t)),
                None => {
                    return failed(
                        "project.ogt: the engine line could not be rewritten".into(),
                        base,
                    );
                }
            },
            Err(e) => return failed(format!("project.ogt: {e}"), base),
        }
        let pkg = p.root.join("package.json");
        if let Ok(text) = std::fs::read_to_string(&pkg)
            && let Some(t) = set_dependency(&text, &to)
        {
            writes.push((pkg, t));
        }
    }
    writes.extend(
        texts
            .into_iter()
            .filter(|(_, _, touched)| *touched)
            .map(|(f, t, _)| (f, t)),
    );
    for (file, text) in &writes {
        if !dry && let Err(e) = std::fs::write(file, text) {
            return failed(format!("{}: {e}", file.display()), base);
        }
        edited.push(rel(file));
    }
    let status = if manual.is_empty() { "done" } else { "manual" };
    json!({
        "status": status,
        "project": p.root.display().to_string(),
        "from": from,
        "to": to,
        "installed": installed,
        "kind": kind,
        "simulation": simulation,
        "versions": crossed.iter().map(|e| e["version"].clone()).collect::<Vec<_>>(),
        "dryRun": dry,
        "edited": edited,
        "codemods": codemods,
        "manual": manual,
        "review": review,
    })
}

/// The game's own text: `src/` and `assets/` (TypeScript, JavaScript and
/// `.ogt`), leaving out build output and dependencies.
fn game_sources(root: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        let mut entries: Vec<_> = rd.flatten().map(|e| e.path()).collect();
        entries.sort();
        for p in entries {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if p.is_dir() {
                if !matches!(name, "node_modules" | ".og" | "cooked" | "dist" | "og-out") {
                    walk(&p, out);
                }
            } else if [".ts", ".tsx", ".js", ".mjs", ".ogt"]
                .iter()
                .any(|x| name.ends_with(x))
            {
                out.push(p);
            }
        }
    }
    let mut out = Vec::new();
    walk(&root.join("src"), &mut out);
    walk(&root.join("assets"), &mut out);
    out
}

/// project.ogt with its `engine "<v>"` line set to `to`.
fn set_engine_line(text: &str, to: &str) -> Option<String> {
    let mut hit = false;
    let lines: Vec<String> = text
        .split('\n')
        .map(|l| {
            let t = l.trim_start();
            if !hit
                && t.strip_prefix("engine")
                    .is_some_and(|r| r.starts_with(char::is_whitespace))
            {
                hit = true;
                let indent = &l[..l.len() - t.len()];
                format!("{indent}engine \"{to}\"")
            } else {
                l.to_string()
            }
        })
        .collect();
    hit.then(|| lines.join("\n"))
}

/// package.json with its `"@origingame/engine": "<v>"` set to `to`.
fn set_dependency(text: &str, to: &str) -> Option<String> {
    let key = "\"@origingame/engine\"";
    let at = text.find(key)? + key.len();
    let rest = &text[at..];
    let open = at + rest.find('"')?;
    let close = open + 1 + text[open + 1..].find('"')?;
    (text[at..open].trim() == ":")
        .then(|| format!("{}\"{to}\"{}", &text[..open], &text[close + 1..]))
}

fn print_text(r: &Value, dry: bool) {
    if r["status"] == "failed" {
        eprintln!("og upgrade: {}", r["reason"].as_str().unwrap_or("failed"));
        return;
    }
    let s = |v: &Value| v.as_str().unwrap_or("").to_string();
    if r["kind"] == "none" {
        println!("og upgrade: already on {}", s(&r["to"]));
        return;
    }
    println!(
        "og upgrade: {} -> {} ({}, simulation {}){}",
        s(&r["from"]),
        s(&r["to"]),
        s(&r["kind"]),
        s(&r["simulation"]),
        if dry {
            ", dry run: nothing written"
        } else {
            ""
        }
    );
    for f in r["edited"].as_array().into_iter().flatten() {
        println!("  edited   {}", s(f));
    }
    for c in r["codemods"].as_array().into_iter().flatten() {
        println!(
            "  rewrote  {}: {} -> {} ({}x, {})",
            s(&c["file"]),
            s(&c["from"]),
            s(&c["to"]),
            c["count"],
            s(&c["version"])
        );
    }
    for m in r["manual"].as_array().into_iter().flatten() {
        println!(
            "  edit     {}:{}: {} ({}): {} Fix: {}",
            s(&m["file"]),
            m["line"],
            s(&m["member"]),
            s(&m["version"]),
            s(&m["change"]),
            s(&m["fix"])
        );
    }
    for m in r["review"].as_array().into_iter().flatten() {
        println!(
            "  review   {} ({}): {} Fix: {}",
            s(&m["member"]),
            s(&m["version"]),
            s(&m["change"]),
            s(&m["fix"])
        );
    }
    if !dry {
        println!("  then: bun install, og check, og test");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pin_and_the_dependency_are_rewritten_in_place() {
        let ogt =
            "type project\nversion 1\n---\nname \"x\"\nengine \"0.5.0\"\nentry \"src/main.ts\"\n";
        assert_eq!(
            set_engine_line(ogt, "0.6.2").unwrap(),
            ogt.replace("0.5.0", "0.6.2")
        );
        assert!(set_engine_line("type project\n---\nengineer \"a\"\n", "1.0.0").is_none());
        let pkg = "{\n  \"dependencies\": {\n    \"@origingame/engine\": \"^0.5.0\",\n    \"x\": \"1\"\n  }\n}\n";
        assert_eq!(
            set_dependency(pkg, "0.6.2").unwrap(),
            pkg.replace("^0.5.0", "0.6.2")
        );
    }
}
