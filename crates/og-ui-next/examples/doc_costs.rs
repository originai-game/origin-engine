//! The per-document compile on First Light's HUD and the components it
//! uses: elements, conditional spans, the compile's time, and what it
//! skips, by kind. Run from the checkout's root.
//!
//!     cargo run --release -p og-ui-next --example doc_costs

use og_ui::UiDocument;
use og_ui_next::{Tokens, doc::DocStyle, doc::Skip};
use std::collections::BTreeMap;
use std::time::Instant;

fn read(p: &str) -> String {
    std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{p}: {e}"))
}

fn main() {
    let mut css = String::new();
    for f in [
        "seed", "color", "role", "derived", "contrast", "state", "space", "density", "type",
        "effect", "motion",
    ] {
        css += &read(&format!("packages/design/tokens/{f}.css"));
    }
    css += &read("crates/og-ui/default.css");
    let tokens = Tokens::parse(&css);
    let surfaces =
        og_ui_next::style::Sheet::compile(&read("crates/og-ui-next/default/surfaces.css"), &tokens);
    for s in &surfaces.skipped {
        println!("surfaces.css skips {s}");
    }
    // Each built-in pack's surfaces over its tokens.
    for pack in ["fantasy", "scifi", "cartoon"] {
        let mut t = tokens.clone();
        t.over(&read(&format!("templates/styles/{pack}/tokens.css")));
        let s = og_ui_next::style::Sheet::compile(
            &read(&format!("templates/styles/{pack}/surfaces.css")),
            &t,
        );
        println!("{pack} pack: {} tokens, {} skips", t.len(), s.skipped.len());
        for k in &s.skipped {
            println!("{pack}/surfaces.css skips {k}");
        }
    }
    // First Light's HUD, then every document of the directories rewritten
    // so far (`@ui/<dir>/<file>` is a document's id).
    let mut docs = vec![(
        "ui/hud.svelte".to_string(),
        "showcase/first-light/assets/ui/hud.svelte".to_string(),
    )];
    for dir in [
        "components/hud",
        "components/rpg",
        "components/shooter",
        "components/racing",
        "components/strategy",
        "components/rhythm",
        "components/arena",
        "components/base",
        "components/card",
        "components/idle",
        "components/meta",
        "components/panels",
        "components/puzzle",
        "components/world",
        "controls",
    ] {
        let root = format!("packages/engine/src/ui/{dir}");
        let mut files: Vec<String> = std::fs::read_dir(&root)
            .unwrap_or_else(|e| panic!("{root}: {e}"))
            .filter_map(|e| e.ok()?.file_name().into_string().ok())
            .filter(|n| n.ends_with(".svelte"))
            .collect();
        files.sort();
        for f in files {
            docs.push((format!("@ui/{dir}/{f}"), format!("{root}/{f}")));
        }
    }
    // The screens, Night Shift's HUD and the gallery's sheet pages.
    let mut roots = vec![(
        "@ui/screens".to_string(),
        "packages/engine/src/ui/screens".to_string(),
    )];
    roots.push((
        "night-shift/ui".to_string(),
        "showcase/night-shift/assets/ui".to_string(),
    ));
    let sheets = "packages/engine/sheets";
    let mut names: Vec<String> = std::fs::read_dir(sheets)
        .unwrap_or_else(|e| panic!("{sheets}: {e}"))
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .collect();
    names.sort();
    for n in names {
        roots.push((format!("{n}/ui"), format!("{sheets}/{n}/assets/ui")));
    }
    for (prefix, root) in roots {
        let Ok(dir) = std::fs::read_dir(&root) else {
            continue;
        };
        let mut files: Vec<String> = dir
            .filter_map(|e| e.ok()?.file_name().into_string().ok())
            .filter(|n| n.ends_with(".svelte"))
            .collect();
        files.sort();
        for f in files {
            docs.push((format!("{prefix}/{f}"), format!("{root}/{f}")));
        }
    }
    let mut skips: BTreeMap<Skip, usize> = BTreeMap::new();
    for (id, path) in &docs {
        // A document the parser refuses is reported and the rest go on.
        let doc = match UiDocument::parse(id, &read(path)) {
            Ok(doc) => doc,
            Err(e) => {
                println!("{id}: refused: {e:?}");
                continue;
            }
        };
        let t = Instant::now();
        let s = DocStyle::compile(&doc, &tokens, &surfaces);
        let us = t.elapsed().as_secs_f64() * 1e6;
        println!(
            "{id}: {} elements, {} spans ({} conditional), {} steps, compiled in {us:.0} us",
            s.nodes.len(),
            s.spans.len(),
            s.conditions.len(),
            s.steps.len()
        );
        for k in s.skipped {
            // What a sheet still has to change, by document; the missing
            // declarations arrive with later heads.
            if !matches!(k, Skip::Declaration(_) | Skip::Query) {
                println!("    skips {k:?}");
            }
            *skips.entry(k).or_default() += 1;
        }
    }
    println!("skipped:");
    for (k, n) in skips {
        println!("  {n:3} {k:?}");
    }
}
