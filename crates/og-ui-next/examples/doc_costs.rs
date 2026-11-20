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
    let docs = [
        ("ui/hud.svelte", "showcase/first-light/assets/ui/hud.svelte"),
        (
            "@ui/components/hud/ability-bar.svelte",
            "packages/engine/src/ui/components/hud/ability-bar.svelte",
        ),
        (
            "@ui/components/hud/ability-slot.svelte",
            "packages/engine/src/ui/components/hud/ability-slot.svelte",
        ),
        (
            "@ui/components/hud/quest-tracker.svelte",
            "packages/engine/src/ui/components/hud/quest-tracker.svelte",
        ),
    ];
    let mut skips: BTreeMap<Skip, usize> = BTreeMap::new();
    for (id, path) in docs {
        let doc = UiDocument::parse(id, &read(path)).unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let t = Instant::now();
        let s = DocStyle::compile(&doc, &tokens);
        let us = t.elapsed().as_secs_f64() * 1e6;
        println!(
            "{id}: {} elements, {} spans ({} conditional), {} steps, compiled in {us:.0} us",
            s.nodes.len(),
            s.spans.len(),
            s.conditions.len(),
            s.steps.len()
        );
        for k in s.skipped {
            *skips.entry(k).or_default() += 1;
        }
    }
    println!("skipped:");
    for (k, n) in skips {
        println!("  {n:3} {k:?}");
    }
}
