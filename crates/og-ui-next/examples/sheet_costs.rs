//! What the style compiler takes of a real sheet: the design tokens and
//! First Light's HUD with the components it uses, compiled; the time to
//! read the tokens and compile, and each skipped selector or declaration.
//! Run from the checkout's root.
//!
//!     cargo run --release -p og-ui-next --example sheet_costs

use og_ui_next::{Tokens, style::Sheet};
use std::time::Instant;

fn read(p: &str) -> String {
    std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{p}: {e}"))
}

fn style_of(p: &str) -> String {
    let t = read(p);
    let a = t.find("<style>").map_or(t.len(), |i| i + 7);
    let b = t[a..].find("</style>").map_or(t.len(), |i| a + i);
    t[a..b].to_owned()
}

fn main() {
    let mut tokens_css = String::new();
    for f in [
        "seed", "color", "role", "derived", "contrast", "state", "space", "density", "type",
        "effect", "motion",
    ] {
        tokens_css += &read(&format!("packages/design/tokens/{f}.css"));
    }
    tokens_css += &read("crates/og-ui/default.css");
    let mut css = String::new();
    for c in ["hud/ability-bar", "hud/ability-slot", "hud/quest-tracker"] {
        css += &style_of(&format!("packages/engine/src/ui/components/{c}.svelte"));
    }
    css += &style_of("showcase/first-light/assets/ui/hud.svelte");
    let t = Instant::now();
    let tokens = Tokens::parse(&tokens_css);
    let tokens_ms = t.elapsed().as_secs_f64() * 1e3;
    let t = Instant::now();
    let sheet = Sheet::compile(&css, &tokens);
    let compile_ms = t.elapsed().as_secs_f64() * 1e3;
    println!(
        "tokens {} in {tokens_ms:.2} ms; sheet compiled in {compile_ms:.3} ms; {} skipped:",
        tokens.len(),
        sheet.skipped.len()
    );
    for s in &sheet.skipped {
        println!("  {s}");
    }
}
