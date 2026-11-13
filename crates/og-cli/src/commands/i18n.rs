//! `og i18n extract [project] [--json] [--write] [--check]`: the og.i18n
//! keys a project's dialogues and quests show, checked against its string
//! tables per locale (og-cook's `strings_extract`).

use og_cook::cook::strings_extract::{extract, write_base};
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "usage: og i18n extract [project] [--json] [--write] [--check]";

fn go(args: &[&str]) -> Result<bool, String> {
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
            println!("no string table (`og i18n extract --write` starts assets/strings/en.ogt)");
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
