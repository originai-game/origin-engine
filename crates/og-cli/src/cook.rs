//! `og cook [project] [--threads N] [--report [out.md]] [--json] [--watch
//! [--exit-after N]]`: assets/ into cooked/ through og-cook's
//! `cook_project`, incrementally by stamp. `og run`, `og build`, `og dev` and
//! `og test` cook first, so a run never reads stale blobs.
//!
//! `--report` prints the ten assets whose last cook took longest (each
//! asset's time is recorded in `cooked/.cook-times.ogt`, so an up-to-date
//! asset keeps the time of the cook that made it), and with a path ending in
//! `.md` writes the whole report there. `--watch` cooks, then cooks again
//! whenever a file under assets/ changes, printing each cooked asset's time
//! and the cook's against the 300 ms reload budget (`--exit-after N` stops
//! after N of them; `--json` prints one JSON line per cook).

use crate::project::Project;
use og_cook::{CookOptions, Report, cook_project};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::ExitCode;
use std::time::Duration;

const USAGE: &str = "usage: og cook [project] [--threads N] [--report [out.md]] [--json] [--watch [--exit-after N]]";

/// The engine's own components as the cook packs scene tables with them
/// (`Transform`, `Model`, the lights), by the names scenes use.
pub fn engine_schemas() -> Vec<og_core::Schema> {
    use og_core::reflect::{FieldSchema, FieldType};
    let ty = |f: &og_api::host::FieldDecl| -> Option<FieldType> {
        if f.capacity > 0 {
            // A text field (`str.48` in the IDL).
            return FieldType::string(f.capacity as usize).ok();
        }
        Some(match f.column {
            "f32" => FieldType::F32,
            "f64" => FieldType::F64,
            "u32" => FieldType::U32,
            "i32" => FieldType::I32,
            "Vector2" => FieldType::Vec2,
            "Vector3" | "Color" => FieldType::Vec3,
            "Quaternion" => FieldType::Quat,
            "Entity" | "Geometry" | "Material" | "Asset" => FieldType::U64,
            other => {
                return serde_json::from_value(serde_json::Value::String(
                    match other {
                        "bool" => "bool",
                        "Matrix4" => "mat4",
                        o => o,
                    }
                    .into(),
                ))
                .ok();
            }
        })
    };
    og_api::host::RECORDS
        .iter()
        .filter(|r| r.kind == "component")
        .filter_map(|r| {
            let fields = r
                .fields
                .iter()
                .map(|f| {
                    ty(f).map(|t| {
                        let mut fs = FieldSchema::new(f.name.to_string(), t);
                        fs.default = default_lanes(f, t.lanes());
                        fs
                    })
                })
                .collect::<Option<Vec<_>>>()?;
            Some(og_core::Schema {
                name: r.name.to_string(),
                version: 1,
                doc: String::new(),
                fields,
            })
        })
        .collect()
}

/// A field's IDL default (JSON) as the lanes the cook packs, so a scene
/// row that omits a field gets the component's default, not zero.
fn default_lanes(f: &og_api::host::FieldDecl, lanes: usize) -> Option<Vec<f64>> {
    use serde_json::Value;
    let v: Value = serde_json::from_str(f.default).ok()?;
    let out: Vec<f64> = match &v {
        Value::Number(n) => vec![n.as_f64()?],
        Value::Bool(b) => vec![f64::from(u8::from(*b))],
        Value::String(s) => vec![f.variants.iter().position(|x| x == s)? as f64],
        Value::Object(m) if m.contains_key("elements") => m["elements"]
            .as_array()?
            .iter()
            .map(Value::as_f64)
            .collect::<Option<_>>()?,
        Value::Object(m) => ["x", "y", "z", "w", "r", "g", "b"]
            .iter()
            .filter_map(|k| m.get(*k))
            .map(Value::as_f64)
            .collect::<Option<_>>()?,
        _ => return None,
    };
    (out.len() == lanes).then_some(out)
}

/// Cooks a project; `Err` names the failing assets.
pub fn cook(root: &Path, threads: usize) -> Result<Report, String> {
    let options = CookOptions {
        threads,
        schemas: engine_schemas(),
        ..CookOptions::default()
    };
    let report = cook_project(root, &options).map_err(|e| format!("og cook: {e}"))?;
    if report.ok() {
        Ok(report)
    } else {
        Err(format!(
            "og cook: {} asset(s) failed:\n  {}",
            report.errors().len(),
            report.errors().join("\n  ")
        ))
    }
}

/// One line: what was cooked this time; then one per warning (a locale's
/// missing string keys).
pub fn summary(r: &Report) -> String {
    let outputs: Vec<_> = r.assets.iter().flat_map(|a| a.outputs.iter()).collect();
    let cooked = outputs.iter().filter(|o| o.cooked).count();
    let bytes: u64 = outputs.iter().map(|o| o.bytes).sum();
    let mut s = format!(
        "og cook: {} assets, {} outputs ({cooked} cooked, {} up to date), {bytes} bytes in cooked/, {:.0} ms",
        r.assets.len(),
        outputs.len(),
        outputs.len() - cooked,
        r.elapsed_ms
    );
    for w in &r.warnings {
        s.push_str(&format!("\nog cook: warning: {w}"));
    }
    s
}

/// One line per asset cooked in this run, slowest first: its time (its
/// outputs' summed) and how many outputs it cooked.
pub fn cooked_lines(r: &Report) -> Vec<String> {
    r.cooked_assets()
        .iter()
        .map(|a| {
            let n = a.outputs.iter().filter(|o| o.cooked).count();
            format!(
                "og cook: {} cooked in {:.3} ms ({n} output{})",
                a.source,
                a.cook_ms,
                if n == 1 { "" } else { "s" }
            )
        })
        .collect()
}

/// The slowest ten as aligned text lines (`og cook --report`).
pub fn slowest_lines(r: &Report) -> Vec<String> {
    let rows = r.slowest(10);
    let width = rows.iter().map(|a| a.source.len()).max().unwrap_or(0);
    let mut out = vec![format!(
        "og cook: the slowest {} asset(s) by their last cook (cooked/{}):",
        rows.len(),
        og_cook::COOK_TIMES
    )];
    for (i, a) in rows.iter().enumerate() {
        out.push(format!(
            "  {:>2}. {:<width$}  {:>10.3} ms  {}{}",
            i + 1,
            a.source,
            a.cook_ms,
            a.kind,
            if a.outputs.iter().any(|o| o.cooked) {
                ", cooked now"
            } else {
                ""
            }
        ));
    }
    out
}

/// `--watch`: cooks again on every change under assets/, printing what
/// cooked and how long it took against the reload budget.
fn watch(root: &Path, threads: usize, json: bool, exit_after: Option<usize>) -> ExitCode {
    let assets = root.join("assets");
    let take = || {
        let mut m = BTreeMap::new();
        crate::dev::snapshot(&assets, &mut m);
        m
    };
    let mut seen = take();
    let budget = crate::dev::BUDGET_MS;
    if json {
        println!(
            "{}",
            serde_json::json!({ "event": "watching", "root": root.display().to_string(), "budgetMs": budget })
        );
    } else {
        println!(
            "og cook: watching {}; each change cooks what it reaches (budget {budget} ms)",
            assets.display()
        );
    }
    let mut cooks = 0usize;
    loop {
        std::thread::sleep(Duration::from_millis(25));
        let now = take();
        let changes = crate::dev::changed(&seen, &now);
        if changes.is_empty() {
            continue;
        }
        seen = now;
        cooks += 1;
        let changed: Vec<String> = changes
            .iter()
            .filter_map(|c| c.strip_prefix(&assets).ok())
            .map(|c| c.to_string_lossy().replace('\\', "/"))
            .collect();
        let options = CookOptions {
            threads,
            schemas: engine_schemas(),
            ..CookOptions::default()
        };
        match cook_project(root, &options) {
            Ok(r) if json => {
                let cooked: Vec<serde_json::Value> = r
                    .cooked_assets()
                    .iter()
                    .map(|a| {
                        serde_json::json!({
                            "source": a.source,
                            "ms": (a.cook_ms * 1000.0).round() / 1000.0,
                            "outputs": a.outputs.iter().filter(|o| o.cooked).count(),
                        })
                    })
                    .collect();
                println!(
                    "{}",
                    serde_json::json!({
                        "event": "cook", "n": cooks, "ok": r.ok(),
                        "changed": changed, "assets": cooked, "errors": r.errors(),
                        "ms": (r.elapsed_ms * 10.0).round() / 10.0,
                        "budgetMs": budget, "withinBudget": r.elapsed_ms <= budget,
                    })
                );
            }
            Ok(r) => {
                for line in cooked_lines(&r) {
                    println!("{line}");
                }
                for e in r.errors() {
                    eprintln!("og cook: {e}");
                }
                println!(
                    "og cook: {} in {:.0} ms (budget {budget} ms{})",
                    match r.cooked_assets().len() {
                        0 => "nothing to cook".to_string(),
                        1 => "1 asset cooked".to_string(),
                        n => format!("{n} assets cooked"),
                    },
                    r.elapsed_ms,
                    if r.elapsed_ms <= budget { "" } else { ", OVER" }
                );
            }
            Err(e) => eprintln!("og cook: {e}"),
        }
        if exit_after.is_some_and(|n| cooks >= n) {
            return ExitCode::SUCCESS;
        }
    }
}

pub fn run(args: &[&str]) -> ExitCode {
    let mut project = ".".to_string();
    let mut threads = 0;
    let mut report = None;
    let mut slowest = false;
    let mut json = false;
    let mut watching = false;
    let mut exit_after = None;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--threads" => {
                let Some(n) = args.get(i + 1).and_then(|v| v.parse().ok()) else {
                    eprintln!("og cook: --threads takes a number. {USAGE}");
                    return ExitCode::from(2);
                };
                threads = n;
                i += 1;
            }
            "--report" => {
                slowest = true;
                // The path is optional: a file ending in .md.
                if let Some(p) = args.get(i + 1).filter(|p| p.ends_with(".md")) {
                    report = Some(p.to_string());
                    i += 1;
                }
            }
            "--json" => json = true,
            "--watch" => watching = true,
            "--exit-after" => {
                let Some(n) = args.get(i + 1).and_then(|v| v.parse().ok()) else {
                    eprintln!("og cook: --exit-after takes a number. {USAGE}");
                    return ExitCode::from(2);
                };
                exit_after = Some(n);
                i += 1;
            }
            a if a.starts_with('-') => {
                eprintln!("og cook: unknown option `{a}`. {USAGE}");
                return ExitCode::from(2);
            }
            a => project = a.to_string(),
        }
        i += 1;
    }
    let p = match Project::open(Path::new(&project)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("og cook: {e}");
            return ExitCode::FAILURE;
        }
    };
    if !p.has_assets() {
        eprintln!("og cook: {} has no assets/ to cook", p.root.display());
        return ExitCode::FAILURE;
    }
    match cook(&p.root, threads) {
        Ok(r) => {
            if let Some(path) = report
                && let Err(e) = std::fs::write(&path, r.to_markdown())
            {
                eprintln!("og cook: cannot write {path}: {e}");
                return ExitCode::FAILURE;
            }
            if watching {
                if !json {
                    println!("{}", summary(&r));
                }
                return watch(&p.root, threads, json, exit_after);
            }
            if json {
                println!("{}", r.to_json());
            } else {
                println!("{}", summary(&r));
                if slowest {
                    for line in slowest_lines(&r) {
                        println!("{line}");
                    }
                }
            }
            ExitCode::SUCCESS
        }
        // A watch keeps going: the next change may fix the failing asset.
        Err(e) if watching => {
            eprintln!("{e}");
            watch(&p.root, threads, json, exit_after)
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
