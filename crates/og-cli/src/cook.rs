//! `og cook [project] [--threads N] [--report [out.md|out.json]] [--json] [--watch
//! [--exit-after N]]`: assets/ into cooked/ through og-cook's
//! `cook_project`, incrementally by stamp. `og run`, `og build`, `og dev` and
//! `og test` cook first, so a run never reads stale blobs.
//!
//! `--report` prints the ten assets whose last cook took longest (each
//! asset's time is recorded in `cooked/.cook-times.ogt`, so an up-to-date
//! asset keeps the time of the cook that made it), with a path ending in
//! `.md` writes the whole report there, and always writes the diagnostics
//! as data: `cook-report.json` (og-cook's `cook::stats`: per asset its
//! triangles, vertices and LODs, textures with sizes and formats, bones,
//! clips, bytes cooked and the budget flags against project.ogt's `budgets
//! { }`, and the warnings: a model whose raycast BVH is past `bvhShare`
//! of its blob, also printed), at `og-out/cook-report.json` or the `.json`
//! path given, which `og doctor --project` reads. A generated terrain
//! (`type terrain` with `generate`) adds its height histogram, land
//! fraction above the sea and river length, and its height and control
//! maps as PNGs under the report's directory (`terrain-maps/`), for a
//! reviewer's eyes, never into assets/. `--watch` cooks, then cooks again
//! whenever a file under assets/ changes, printing each cooked asset's time
//! and the cook's against the 300 ms reload budget (`--exit-after N` stops
//! after N of them; `--json` prints one JSON line per cook).
//!
//! Colour textures cook as project.ogt's `cook { color_textures etc1s|uastc
//! }` says (etc1s without it); the web export's cook (`og build`, `og run
//! --device`) takes `publish { color_textures ... }` over it, so a game can
//! ship UASTC natively and a quarter of the bytes on the web
//! ([`texture_options`]). Switching between the two re-encodes only the
//! colour textures, and the shared texture cache keeps both encodings.

use crate::project::Project;
use og_cook::cook::TextureOptions;
use og_cook::{CookOptions, Report, cook_project};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::ExitCode;
use std::time::Duration;

const USAGE: &str = "usage: og cook [project] [--threads N] [--report [out.md|out.json]] [--json] [--watch [--exit-after N]]";

/// Where `og cook --report` writes the diagnostics by default, under the
/// project (`og doctor --project` reads it there).
pub const COOK_REPORT: &str = "og-out/cook-report.json";

/// The budgets of the project at `root` (project.ogt's `budgets { }` over
/// the defaults; the defaults without a manifest).
pub fn budgets(root: &Path) -> Result<og_cook::cook::stats::Budgets, String> {
    match std::fs::read_to_string(root.join("project.ogt")) {
        Ok(text) => {
            let doc =
                og_core::ogt::Document::parse(&text).map_err(|e| format!("project.ogt: {e}"))?;
            og_cook::cook::stats::Budgets::from_project(&doc.body)
        }
        Err(_) => Ok(og_cook::cook::stats::Budgets::default()),
    }
}

/// Writes `cook-report.json` for a finished cook at `path`; returns its
/// assets' warnings (a model's raycast BVH past `budgets.bvhShare`).
pub fn write_cook_report(root: &Path, report: &Report, path: &Path) -> Result<Vec<String>, String> {
    let budgets = budgets(root)?;
    let mut json = og_cook::cook::stats::cook_report_json(report, &root.join("cooked"), &budgets);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        og_cook::cook::stats::write_terrain_maps(&mut json, &root.join("cooked"), dir)?;
    }
    let text = serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?;
    std::fs::write(path, text + "\n").map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(json["assets"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|a| a["warnings"].as_array().into_iter().flatten())
        .filter_map(|w| w["message"].as_str().map(str::to_owned))
        .collect())
}

/// The engine's own components as the cook packs scene tables with them
/// (`Transform`, `Model`, the lights), by the names scenes use.
pub fn engine_schemas() -> Vec<og_core::Schema> {
    use og_core::reflect::{FieldSchema, FieldType};
    let scalar = |f: &og_api::host::FieldDecl| -> Option<FieldType> {
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
    let ty = |f: &og_api::host::FieldDecl| -> Option<FieldType> {
        if f.capacity > 0 {
            // A text field (`str.48` in the IDL).
            return FieldType::string(f.capacity as usize).ok();
        }
        if f.list > 0 {
            // A list field (`List.u32.4` in the IDL): its element's scalar.
            return FieldType::list(scalar(f)?.scalar(), f.list as usize).ok();
        }
        scalar(f)
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
                        // A list starts empty: it declares no default.
                        fs.default = (!t.is_list())
                            .then(|| default_lanes(f, t.lanes()))
                            .flatten();
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

/// The texture options of the project at `root`: project.ogt's `cook {
/// color_textures uastc|etc1s }` (etc1s without it), which `publish {
/// color_textures ... }` overrides for the web export (`web`: `og build`,
/// `og run --device`). The defaults without a manifest.
pub fn texture_options(root: &Path, web: bool) -> Result<TextureOptions, String> {
    let mut o = TextureOptions::default();
    let Ok(text) = std::fs::read_to_string(root.join("project.ogt")) else {
        return Ok(o);
    };
    let doc = og_core::ogt::Document::parse(&text).map_err(|e| format!("project.ogt: {e}"))?;
    let blocks: &[&str] = if web { &["cook", "publish"] } else { &["cook"] };
    for name in blocks {
        let Some(block) = doc.body.get(name).and_then(|e| e.block.as_ref()) else {
            continue;
        };
        for e in &block.entries {
            match e.key.as_str() {
                "color_textures" => {
                    let v = e
                        .values
                        .first()
                        .and_then(|v| v.as_word().or_else(|| v.as_str()));
                    o.color_uastc = match v {
                        Some("uastc") => true,
                        Some("etc1s") => false,
                        _ => {
                            return Err(format!(
                                "project.ogt:{}: {name}.color_textures is `{}`; use etc1s (the default, a quarter of the bytes) or uastc (no 4x4 block colour loss)",
                                e.line,
                                v.unwrap_or("")
                            ));
                        }
                    };
                }
                // The publish block's other settings are the export's
                // (`publish.rs`).
                _ if *name == "publish" => {}
                other => {
                    return Err(format!(
                        "project.ogt:{}: cook has no `{other}`; it takes color_textures",
                        e.line
                    ));
                }
            }
        }
    }
    Ok(o)
}

/// Cooks a project for a native run, or for the web export (`web`, which
/// takes project.ogt's `publish` overrides); `Err` names the failing assets.
pub fn cook(root: &Path, threads: usize, web: bool) -> Result<Report, String> {
    let options = CookOptions {
        threads,
        schemas: engine_schemas(),
        texture: texture_options(root, web)?,
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

/// The art contract's lines (`og cook --report`): how many assets it
/// changed and how many opted out, then one line each (docs/art.md §2.3).
pub fn conform_lines(r: &Report) -> Vec<String> {
    if r.conform.is_empty() {
        return Vec::new();
    }
    let count =
        |a: og_cook::cook::ConformAction| r.conform.iter().filter(|e| e.action == a).count();
    let mut out = vec![format!(
        "og cook: the art contract changed {} asset(s), {} met it as they were, {} opted out:",
        count(og_cook::cook::ConformAction::Changed),
        count(og_cook::cook::ConformAction::Unchanged),
        count(og_cook::cook::ConformAction::Off)
    )];
    for e in &r.conform {
        out.push(format!("  {} ({}): {}", e.source, e.action.name(), e.what));
    }
    out
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
        let options = match texture_options(root, false) {
            Ok(texture) => CookOptions {
                threads,
                schemas: engine_schemas(),
                texture,
                ..CookOptions::default()
            },
            Err(e) => {
                eprintln!("og cook: {e}");
                continue;
            }
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
    let mut report_json = None;
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
                } else if let Some(p) = args.get(i + 1).filter(|p| p.ends_with(".json")) {
                    report_json = Some(std::path::PathBuf::from(p));
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
    match cook(&p.root, threads, false) {
        Ok(r) => {
            if let Some(path) = report
                && let Err(e) = std::fs::write(&path, r.to_markdown())
            {
                eprintln!("og cook: cannot write {path}: {e}");
                return ExitCode::FAILURE;
            }
            if slowest {
                let path = report_json.unwrap_or_else(|| p.root.join(COOK_REPORT));
                match write_cook_report(&p.root, &r, &path) {
                    Ok(warnings) => {
                        for w in warnings {
                            eprintln!("og cook: warning: {w}");
                        }
                    }
                    Err(e) => {
                        eprintln!("og cook: cook-report.json: {e}");
                        return ExitCode::FAILURE;
                    }
                }
                if !json {
                    println!("wrote {}", path.display());
                }
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
                    for line in conform_lines(&r) {
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

#[cfg(test)]
mod tests {
    use super::texture_options;

    fn project(text: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU32, Ordering};
        static N: AtomicU32 = AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!(
            "og-cook-options-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("project.ogt"), text).unwrap();
        dir
    }

    #[test]
    fn color_textures_reach_texture_options_with_the_publish_override() {
        let head = "type project\nversion 1\n---\nname \"t\"\n";
        let both = project(&format!(
            "{head}cook {{\n  color_textures uastc\n}}\npublish {{\n  probe player\n  color_textures etc1s\n}}\n"
        ));
        assert!(texture_options(&both, false).unwrap().color_uastc);
        assert!(!texture_options(&both, true).unwrap().color_uastc);
        // Without an override the web export takes the cook block's.
        let cook_only = project(&format!("{head}cook {{\n  color_textures uastc\n}}\n"));
        assert!(texture_options(&cook_only, true).unwrap().color_uastc);
        // The default is ETC1S, and a publish override alone moves only the export.
        let publish_only = project(&format!("{head}publish {{\n  color_textures uastc\n}}\n"));
        assert!(!texture_options(&publish_only, false).unwrap().color_uastc);
        assert!(texture_options(&publish_only, true).unwrap().color_uastc);
        let bad = project(&format!("{head}cook {{\n  color_textures astc\n}}\n"));
        let e = texture_options(&bad, false).unwrap_err();
        assert!(e.contains("project.ogt:6") && e.contains("etc1s"), "{e}");
        for d in [both, cook_only, publish_only, bad] {
            let _ = std::fs::remove_dir_all(d);
        }
    }
}
