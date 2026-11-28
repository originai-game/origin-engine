//! `og cook [project] [--threads N] [--report [out.md|out.json]] [--json] [--watch
//! [--exit-after N]]`: assets/ into cooked/ through og-cook's
//! `cook_project`, incrementally by stamp. `og run`, `og build`, `og dev` and
//! `og test` cook first, so a run never reads stale blobs. A long cook says
//! how far it is on stderr: each stage and output that took 5 s or more,
//! and every 10 s the outputs cooked and the one running longest.
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
//! after N of them; `--json` prints one JSON line per cook, after a
//! `watching` line naming the root and the built-in styles' directory).
//!
//! Colour textures cook as project.ogt's `cook { color_textures etc1s|uastc
//! }` says (etc1s without it); the web export's cook (`og build`, `og run
//! --device`) takes `publish { color_textures ... }` over it, so a game can
//! ship UASTC natively and a quarter of the bytes on the web
//! ([`texture_options`]). Switching between the two re-encodes only the
//! colour textures, and the shared texture cache keeps both encodings.

use crate::project::Project;
use og_cook::cook::{CodeGraph, TextureOptions};
use og_cook::{CookOptions, Report, cook_project};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::ExitCode;
use std::time::Duration;

const USAGE: &str = "usage: og cook [project] [--pack ID] [--threads N] [--report [out.md|out.json]] [--json] [--watch [--exit-after N]]";

/// Where `og cook --report` writes the diagnostics by default, under the
/// project (`og doctor --project` reads it there).
pub const COOK_REPORT: &str = "og-out/cook-report.json";

/// Checked project.ts report budgets over the report defaults.
pub fn budgets(root: &Path) -> Result<og_cook::cook::stats::Budgets, String> {
    let project = og_cook::cook::source_project::Project::open(root).map_err(|e| e.to_string())?;
    Ok(og_cook::cook::stats::Budgets::from_json(
        &project.data.root().into_json()["budgets"],
    ))
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

/// Checked cook texture settings; publish overrides apply to web exports.
pub fn texture_options(root: &Path, web: bool) -> Result<TextureOptions, String> {
    use og_cook::cook::source_schemas::project::ColorEncoding;
    let project = og_cook::cook::source_project::Project::open(root).map_err(|e| e.to_string())?;
    let mut o = TextureOptions::default();
    let encoding = web
        .then(|| {
            project
                .input
                .publish
                .as_ref()
                .and_then(|p| p.color_textures.as_ref())
        })
        .flatten()
        .or_else(|| {
            project
                .input
                .cook
                .as_ref()
                .and_then(|c| c.color_textures.as_ref())
        });
    if let Some(encoding) = encoding {
        o.color_uastc = matches!(encoding, ColorEncoding::Uastc);
    }
    Ok(o)
}

/// Checked project.ts mesh settings, preserving the cooker's defaults.
pub fn mesh_options(root: &Path) -> Result<og_cook::cook::MeshOptions, String> {
    let project = og_cook::cook::source_project::Project::open(root).map_err(|e| e.to_string())?;
    let mut o = og_cook::cook::MeshOptions::default();
    if let Some(clusters) = project.input.cook.and_then(|c| c.clusters) {
        o.clusters = clusters;
    }
    Ok(o)
}

/// The cook's options for the project at `root`, its code reaching what
/// `code` says: the code's raycasts decide whether models carry a cooked
/// raycast BVH (`MeshOptions::raycast`, about a quarter of a model blob; a
/// model's `model { raycast true|false }` decides for it, and a raycast at
/// a model cooked without one builds it then).
fn options(root: &Path, threads: usize, web: bool, code: CodeGraph) -> Result<CookOptions, String> {
    let mut mesh = mesh_options(root)?;
    mesh.raycast = code.raycast;
    Ok(CookOptions {
        threads,
        progress: true,
        schemas: og_cook::cook::engine_schemas(),
        texture: texture_options(root, web)?,
        mesh,
        code,
        ..CookOptions::default()
    })
}

/// Cooks a project for a native run, or for the web export (`web`, which
/// takes project.ogt's `publish` overrides); `Err` names the failing assets.
pub fn cook(root: &Path, threads: usize, web: bool) -> Result<Report, String> {
    cook_for(root, threads, web, false)
}

/// A tooling preview cooks its own roots, never the game's entry or boot scene.
pub fn cook_roots(root: &Path, roots: Vec<String>) -> Result<Report, String> {
    let mut options = options(root, 0, false, CodeGraph::default())?;
    options.roots = Some(roots);
    checked_cook(root, &options)
}

/// [`cook`] for a publish (`og build --publish`): a prefab override that
/// no longer does anything is refused, naming it.
pub fn cook_for(root: &Path, threads: usize, web: bool, publish: bool) -> Result<Report, String> {
    let options = options(root, threads, web, code_graph(root)?)?;
    checked_cook(root, &CookOptions { publish, ..options })
}

/// What the code of the project at `root` reaches: the import graph of a
/// native bundle of it (the bundle og run makes, from its cache when the
/// code is as it was). An entry that does not exist reaches nothing.
pub fn code_graph(root: &Path) -> Result<CodeGraph, String> {
    let p = Project::open(root)?;
    if !p.entry.is_file() {
        return Ok(CodeGraph::default());
    }
    let package = crate::project::engine_package(Some(&p.root))
        .ok_or_else(|| format!("og cook: {}", crate::project::NO_ENGINE))?;
    let bundle =
        crate::project::bundle_native(&p, &package).map_err(|e| format!("og cook: {e}"))?;
    CodeGraph::read(&crate::project::graph_of(&bundle), &p.root)
}

/// [`cook_for`] with the code's graph given (`code_graph`, or the one the
/// bundle a run just made wrote).
pub fn cook_with(
    root: &Path,
    threads: usize,
    web: bool,
    publish: bool,
    code: CodeGraph,
) -> Result<Report, String> {
    let options = options(root, threads, web, code)?;
    let options = CookOptions {
        publish,
        entry_only: true,
        ..options
    };
    checked_cook(root, &options)
}

fn selected_cook(root: &Path, threads: usize, pack: Option<&str>) -> Result<Report, String> {
    let mut options = options(root, threads, false, code_graph(root)?)?;
    options.pack = pack.map(str::to_owned);
    checked_cook(root, &options)
}

fn checked_cook(root: &Path, options: &CookOptions) -> Result<Report, String> {
    crate::author_types::ensure(root)?;
    let report = cook_project(root, options).map_err(|e| format!("og cook: {e}"))?;
    if report.ok() {
        Ok(report)
    } else if let Some(missing) = missing_checkout_pins() {
        // The failures are this checkout's, not the game's: say the one fix.
        Err(format!(
            "og cook: {} asset(s) failed, and this engine checkout lacks {missing}; run bun scripts/assets.ts ensure templates",
            report.errors().len()
        ))
    } else {
        Err(format!(
            "og cook: {} asset(s) failed:\n  {}",
            report.errors().len(),
            report.errors().join("\n  ")
        ))
    }
}

/// In an engine checkout whose built-in styles or fonts come from
/// `templates/`, the first file `templates/og-assets.json` pins there that is
/// not installed, with how many are missing: those live in the asset store
/// until `bun scripts/assets.ts ensure templates`. `None` for an installed
/// engine (no manifest) or a complete checkout.
fn missing_checkout_pins() -> Option<String> {
    let dirs = [
        og_cook::cook::look::styles_dir(),
        og_cook::cook::look::fonts_dir(),
    ];
    let templates = dirs.into_iter().flatten().find_map(|d| {
        let t = d.parent()?;
        t.join("og-assets.json").is_file().then(|| t.to_path_buf())
    })?;
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(templates.join("og-assets.json")).ok()?).ok()?;
    let missing: Vec<&str> = manifest["files"]
        .as_array()?
        .iter()
        .filter_map(|f| f["path"].as_str())
        .filter(|p| !templates.join(p).exists())
        .collect();
    let first = missing.first()?;
    Some(format!(
        "{} pinned file(s) under templates/ ({first})",
        missing.len()
    ))
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
    if !r.stages.is_empty() {
        let stages: Vec<String> = r
            .stages
            .iter()
            .map(|s| format!("{} {:.1}", s.name, s.ms))
            .collect();
        out.push(format!("og cook: stages (ms): {}", stages.join(", ")));
    }
    out
}

/// `--watch`: cooks again on every change under assets/, and under the
/// engine's built-in styles, documents and (in a checkout) design tokens
/// and default CSS the cook reads, printing what cooked and how long it
/// took against the reload budget. One process: what an earlier cook
/// parsed and did not change is not parsed again.
fn watch(
    root: &Path,
    threads: usize,
    pack: Option<&str>,
    json: bool,
    exit_after: Option<usize>,
) -> ExitCode {
    let assets = root.join("assets");
    let mut watched = Vec::new();
    watched.extend(og_cook::cook::look::styles_dir().map(Path::to_path_buf));
    let take = || {
        let mut m = BTreeMap::new();
        for file in og_cook::cook::source_project::files(root).unwrap_or_default() {
            if let Ok(time) = std::fs::metadata(&file).and_then(|m| m.modified()) {
                m.insert(file, time);
            }
        }
        for w in &watched {
            if w.is_file() {
                if let Ok(t) = std::fs::metadata(w).and_then(|m| m.modified()) {
                    m.insert(w.clone(), t);
                }
            } else {
                crate::dev::snapshot(w, &mut m);
            }
        }
        m
    };
    let mut seen = take();
    let budget = crate::dev::BUDGET_MS;
    if json {
        println!(
            "{}",
            serde_json::json!({
                "event": "watching",
                "root": root.display().to_string(),
                "budgetMs": budget,
                // Where the built-in style packs (`ui "<pack>"`) are read from, null without them.
                "styles": og_cook::cook::look::styles_dir().map(|d| d.display().to_string()),
            })
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
        let walk = std::time::Instant::now();
        let now = take();
        let walk_ms = walk.elapsed().as_secs_f64() * 1000.0;
        let changes = crate::dev::changed(&seen, &now);
        if changes.is_empty() {
            continue;
        }
        // How long the newest edit waited to be seen (the poll and the walk).
        let detect_ms = changes
            .iter()
            .filter_map(|c| now.get(c))
            .max()
            .and_then(|t| std::time::SystemTime::now().duration_since(*t).ok())
            .map_or(0.0, |d| d.as_secs_f64() * 1000.0);
        let started = std::time::Instant::now();
        seen = now;
        cooks += 1;
        let changed: Vec<String> = changes
            .iter()
            .map(|c| c.strip_prefix(&assets).unwrap_or(c))
            .map(|c| c.to_string_lossy().replace('\\', "/"))
            .collect();
        // Refresh bootstrap, generated types and the runtime import graph on
        // every edit; keeping the first graph loses newly imported assets.
        match selected_cook(root, threads, pack) {
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
                        // The steps around the cook: the edit's age when seen,
                        // one walk of the watched trees, and seen to reported.
                        "detectMs": detect_ms.round(), "walkMs": walk_ms.round(),
                        "reportMs": (started.elapsed().as_secs_f64() * 1000.0).round(),
                        // The cook's own stages (index, plan, outputs, write…), ms each.
                        "stages": r.stages.iter().map(|s| serde_json::json!([s.name, (s.ms * 10.0).round() / 10.0])).collect::<Vec<_>>(),
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
            Err(e) if json => println!(
                "{}",
                serde_json::json!({
                    "event": "cook", "n": cooks, "ok": false,
                    "changed": changed, "errors": [e],
                })
            ),
            Err(e) => eprintln!("og cook: {e}"),
        }
        if exit_after.is_some_and(|n| cooks >= n) {
            return ExitCode::SUCCESS;
        }
    }
}

pub fn run(args: &[&str]) -> ExitCode {
    let mut project = ".".to_string();
    let mut pack = None;
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
            "--pack" => {
                let Some(id) = args.get(i + 1).filter(|id| !id.starts_with('-')) else {
                    eprintln!("og cook: --pack takes an ID. {USAGE}");
                    return ExitCode::from(2);
                };
                pack = Some(*id);
                i += 1;
            }
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
    match selected_cook(&p.root, threads, pack) {
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
                return watch(&p.root, threads, pack, json, exit_after);
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
            watch(&p.root, threads, pack, json, exit_after)
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
