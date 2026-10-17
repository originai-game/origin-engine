//! `og-cook <project> [--threads N] [--report out.md] [--json out.json] [--schemas schema.json]`
//!
//! Cooks `<project>/assets` into `<project>/cooked` and prints the report. A
//! development tool for this crate; `og cook` is the user-facing command.

use og_cook::{cook_project, CookOptions};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut root = None;
    let mut options = CookOptions::default();
    let (mut report, mut json) = (None, None);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--threads" => options.threads = args.next().and_then(|v| v.parse().ok()).unwrap_or(0),
            "--report" => report = args.next(),
            "--json" => json = args.next(),
            "--schemas" => {
                let path = args.next().unwrap_or_default();
                let text = match std::fs::read_to_string(&path) {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("{path}: {e}");
                        return ExitCode::from(2);
                    }
                };
                let doc: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
                for t in doc["types"].as_array().cloned().unwrap_or_default() {
                    match og_core::Schema::from_json(&t.to_string()) {
                        Ok(s) => options.schemas.push(s),
                        Err(e) => {
                            eprintln!("{path}: {e}");
                            return ExitCode::from(2);
                        }
                    }
                }
            }
            "-h" | "--help" => {
                println!("usage: og-cook <project> [--threads N] [--report out.md] [--json out.json] [--schemas schema.json]");
                return ExitCode::SUCCESS;
            }
            _ => root = Some(a),
        }
    }
    let Some(root) = root else {
        eprintln!("usage: og-cook <project> [--threads N] [--report out.md] [--json out.json]");
        return ExitCode::from(2);
    };
    match cook_project(std::path::Path::new(&root), &options) {
        Ok(r) => {
            let md = r.to_markdown();
            print!("{md}");
            if let Some(p) = report {
                let _ = std::fs::write(p, &md);
            }
            if let Some(p) = json {
                let _ = std::fs::write(p, r.to_json());
            }
            if r.ok() {
                ExitCode::SUCCESS
            } else {
                for e in r.errors() {
                    eprintln!("error: {e}");
                }
                ExitCode::FAILURE
            }
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
