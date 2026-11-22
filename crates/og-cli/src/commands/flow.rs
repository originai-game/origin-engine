//! `og flow`: a project's flows (og.flow) as facts, read from its sources.
//!
//! - `og flow list [project] [--json]`: each flow dialogue's nodes, their
//!   segments (footage, live or both; duration; window or hold), the
//!   footage and scenes they name with whether each exists and whether the
//!   footage is still a placeholder, and where each choice leads.
//! - `og flow routes [project] [--json]`: every route from a flow's start
//!   to an ending, as the choices, outcomes and cuts that take it, and the
//!   endings no route reaches.
//!
//! The project.ogt `flow` dialogue comes first. A route through a logic
//! node's condition is marked `condition`: it is taken only when the
//! story's variables send it that way.

use crate::project::Project;
use serde_json::Value;
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "usage: og flow list|routes [project] [--json]";

pub fn run(args: &[&str]) -> ExitCode {
    let (what, rest) = match args {
        [w @ ("list" | "routes"), rest @ ..] => (*w, rest),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    let json = rest.contains(&"--json");
    let dir = rest
        .iter()
        .find(|a| !a.starts_with('-'))
        .copied()
        .unwrap_or(".");
    let result = Project::open(Path::new(dir)).and_then(|p| match what {
        "list" => og_cook::cook::flow::list(&p.root),
        _ => og_cook::cook::flow::routes(&p.root),
    });
    let facts = match result {
        Ok(v) => v,
        Err(e) => {
            eprintln!("og flow {what}: {e}");
            return ExitCode::FAILURE;
        }
    };
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&facts).unwrap_or_default()
        );
    } else if what == "list" {
        print_list(&facts);
    } else {
        print_routes(&facts);
    }
    ExitCode::SUCCESS
}

fn s(v: &Value) -> String {
    match v {
        Value::String(t) => t.clone(),
        Value::Null => "-".into(),
        v => v.to_string(),
    }
}

fn print_list(facts: &Value) {
    let flows = facts["flows"].as_array().cloned().unwrap_or_default();
    if flows.is_empty() {
        println!("og flow list: no dialogue under assets/ has a flow node (an `og-flow` header)");
    }
    for f in flows {
        let declared = if f["declared"] == true {
            " (project.ogt flow)"
        } else {
            ""
        };
        println!("{} ({}){declared}", s(&f["dialogue"]), s(&f["file"]));
        for n in f["nodes"].as_array().into_iter().flatten() {
            let seg = &n["segment"];
            let w = &seg["window"];
            let timing = if seg["hold"].is_number() {
                format!("hold {}", s(&seg["hold"]))
            } else if w.is_object() {
                format!(
                    "window {}-{}, cut {}",
                    s(&w["open"]),
                    s(&w["lock"]),
                    s(&w["cut"])
                )
            } else {
                String::new()
            };
            let end = n["ending"]
                .as_str()
                .map(|e| format!(" ending {e}"))
                .unwrap_or_default();
            println!(
                "  {}: {} {} {}s {timing}{end}",
                s(&n["node"]),
                s(&seg["id"]),
                s(&seg["kind"]),
                s(&seg["duration"])
            );
            for c in n["choices"].as_array().into_iter().flatten() {
                let how = c["outcome"]
                    .as_str()
                    .map(|o| format!("#outcome:{o}"))
                    .unwrap_or_else(|| format!("choice {}", s(&c["index"])));
                println!(
                    "    {how} -> {}",
                    c["to"]
                        .as_array()
                        .map(|t| t.iter().map(s).collect::<Vec<_>>().join(", "))
                        .unwrap_or_default()
                );
            }
            if let Some(next) = n["next"].as_array().filter(|t| !t.is_empty()) {
                println!(
                    "    cut -> {}",
                    next.iter().map(s).collect::<Vec<_>>().join(", ")
                );
            }
            for p in n["problems"].as_array().into_iter().flatten() {
                println!("    ! {}", s(p));
            }
        }
    }
}

fn print_routes(facts: &Value) {
    for f in facts["flows"].as_array().into_iter().flatten() {
        let routes = f["routes"].as_array().cloned().unwrap_or_default();
        println!(
            "{} ({}): {} route(s){}",
            s(&f["dialogue"]),
            s(&f["file"]),
            routes.len(),
            if f["truncated"] == true {
                ", truncated"
            } else {
                ""
            }
        );
        for r in &routes {
            let steps: Vec<String> = r["steps"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|st| {
                    let take = if let Some(o) = st["advance"].as_str() {
                        format!("advance:{o}")
                    } else if st["choose"].is_number() {
                        format!("choose:{}", s(&st["choose"]))
                    } else {
                        "cut".into()
                    };
                    let cond = if st["condition"] == true { "?" } else { "" };
                    format!("{} {take}{cond}", s(&st["node"]))
                })
                .collect();
            println!(
                "  -> {} ({}): {}",
                s(&r["end"]),
                s(&r["ending"]),
                steps.join(" | ")
            );
        }
        if let Some(u) = f["unreached"].as_array().filter(|u| !u.is_empty()) {
            println!(
                "  ! no route reaches ending(s) {}",
                u.iter().map(s).collect::<Vec<_>>().join(", ")
            );
        }
    }
}
