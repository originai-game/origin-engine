//! `og i18n extract [project] [--json] [--write] [--check]`: the og.i18n
//! keys a project's dialogues, quests and data tables show, checked against its string
//! tables per locale (og-cook's `strings_extract`).

use og_cook::cook::strings_extract::{extract, write_base};
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "usage: og i18n extract [project] [--json] [--write] [--check] | og i18n tables [project] [--builtins | --index] --json";

fn go(args: &[&str]) -> Result<bool, String> {
    if let ["tables", rest @ ..] = args {
        let (mut root, mut builtins, mut index_only, mut json) = (".", false, false, false);
        for arg in rest {
            match *arg {
                "--builtins" => builtins = true,
                "--index" => index_only = true,
                "--json" => json = true,
                p if !p.starts_with('-') => root = p,
                p => return Err(format!("unknown option {p}. {USAGE}")),
            }
        }
        if !json || (builtins && index_only) {
            return Err(USAGE.into());
        }
        let root = Path::new(root);
        if index_only {
            let project =
                og_cook::cook::source_project::Project::open(root).map_err(|e| e.to_string())?;
            let index = og_cook::cook::source_index::Index::open(root, &project)
                .map_err(|e| e.to_string())?;
            let tables: Vec<_> = index
                .entries
                .values()
                .filter(|entry| entry.factory == "strings")
                .map(|entry| serde_json::json!({"id":entry.id,"path":entry.file}))
                .collect();
            println!("{}", serde_json::json!({"tables":tables}));
            return Ok(true);
        }
        let (tables, index) = if builtins {
            let engine =
                crate::project::engine_package(Some(root)).ok_or(crate::project::NO_ENGINE)?;
            let ui = engine.join("src/ui");
            let mut tables = Vec::new();
            for path in og_cook::cook::source_project::files(&ui).map_err(|e| e.to_string())? {
                if !path.to_string_lossy().ends_with(".strings.ts") {
                    continue;
                }
                let file = path
                    .strip_prefix(&ui)
                    .map_err(|e| e.to_string())?
                    .to_string_lossy()
                    .replace('\\', "/");
                tables.push((
                    file.clone(),
                    og_cook::cook::strings_extract::read_builtin(&ui, &file)?,
                ));
            }
            (tables, None)
        } else {
            let project =
                og_cook::cook::source_project::Project::open(root).map_err(|e| e.to_string())?;
            let index = og_cook::cook::source_index::Index::open(root, &project)
                .map_err(|e| e.to_string())?;
            (
                og_cook::cook::strings_extract::read_tables(root, &project, &index)?,
                Some(index),
            )
        };
        let tables: Vec<_> = tables.into_iter().map(|(path,t)| {
            let id = index.as_ref().map(|i| i.files[&path].clone()).unwrap_or_else(|| format!("@ui/{path}"));
            let entries: std::collections::BTreeMap<_,_> = t.entries.into_iter().map(|(k,(text,_))| (k,text)).collect();
            serde_json::json!({"id":id,"path":path,"locale":t.locale,"base":t.base,"direction":t.direction.as_str(),"fallback":t.fallback,"fonts":t.fonts,"entries":entries})
        }).collect();
        println!("{}", serde_json::json!({"tables":tables}));
        return Ok(true);
    }
    let ["extract", rest @ ..] = args else {
        return Err(USAGE.to_string());
    };
    let (mut project, mut json, mut write, mut check) = (".", false, false, false);
    for a in rest {
        match *a {
            "--json" => json = true,
            "--write" => write = true,
            "--check" => check = true,
            a if !a.starts_with("--") => project = a,
            other => return Err(format!("unknown option `{other}`. {USAGE}")),
        }
    }
    let root = Path::new(project);
    let mut x = extract(root)?;
    if write {
        let (path, added) = write_base(root, &x)?;
        if !json {
            println!("og i18n: added {added} key(s) to {path}");
        }
        x = extract(root)?;
    }
    if json {
        let keys: serde_json::Map<String, serde_json::Value> = x
            .keys
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    serde_json::json!({ "args": v.args, "sources": v.sources }),
                )
            })
            .collect();
        let tables: Vec<serde_json::Value> = x
            .tables
            .iter()
            .map(|t| {
                serde_json::json!({
                    "locale": t.locale, "base": t.base, "path": t.path,
                    "missing": t.missing,
                    "unpassed": t.unpassed.iter().map(|(k, a)| serde_json::json!({ "key": k, "arg": a })).collect::<Vec<_>>(),
                })
            })
            .collect();
        let errors: Vec<serde_json::Value> = x
            .errors
            .iter()
            .map(|(p, e)| serde_json::json!({ "path": p, "error": e }))
            .collect();
        println!(
            "{}",
            serde_json::json!({
                "keys": keys, "stories": x.stories, "tables": tables,
                "errors": errors, "complete": x.complete(),
            })
        );
    } else {
        println!(
            "og i18n: {} key(s) in {} dialogue(s) and quest(s)",
            x.keys.len(),
            x.stories.len()
        );
        for (k, v) in &x.keys {
            let args = if v.args.is_empty() {
                String::new()
            } else {
                format!(" {{{}}}", v.args.join("} {"))
            };
            println!("  {k}{args}  ({})", v.sources.join(", "));
        }
        for (p, e) in &x.errors {
            println!("error: {p}: {e}");
        }
        if x.tables.is_empty() {
            println!("no string table (`og i18n extract --write` starts content/strings/en.ts)");
        }
        for t in &x.tables {
            let base = if t.base { " (base)" } else { "" };
            if t.missing.is_empty() && t.unpassed.is_empty() {
                println!("{}{base} {}: complete", t.locale, t.path);
                continue;
            }
            println!(
                "{}{base} {}: {} missing, {} argument(s) nothing passes",
                t.locale,
                t.path,
                t.missing.len(),
                t.unpassed.len()
            );
            for k in &t.missing {
                println!("  missing {k}");
            }
            for (k, a) in &t.unpassed {
                println!("  {k} takes {{{a}}}, which no line passes");
            }
        }
    }
    Ok(!check || x.complete())
}

pub fn run(args: &[&str]) -> ExitCode {
    match go(args) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(e) => {
            eprintln!("og i18n: {e}");
            ExitCode::from(2)
        }
    }
}
