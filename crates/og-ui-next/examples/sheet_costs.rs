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
    // What the cook writes, and loading it back: what a game pays at load.
    let (sheet_blob, token_blob) = (sheet.to_bytes(), tokens.to_bytes());
    let t = Instant::now();
    let loaded = Sheet::from_bytes(&sheet_blob).unwrap();
    let load_sheet_us = t.elapsed().as_secs_f64() * 1e6;
    let t = Instant::now();
    let loaded_tokens = Tokens::from_bytes(&token_blob).unwrap();
    let load_tokens_us = t.elapsed().as_secs_f64() * 1e6;
    assert_eq!(loaded_tokens.len(), tokens.len());
    assert_eq!(loaded.to_bytes(), sheet_blob);
    println!(
        "cooked: sheet {} B loads in {load_sheet_us:.1} us; tokens {} B load in {load_tokens_us:.1} us",
        sheet_blob.len(),
        token_blob.len()
    );
    println!(
        "tokens {} in {tokens_ms:.2} ms; sheet compiled in {compile_ms:.3} ms; {} skipped:",
        tokens.len(),
        sheet.skipped.len()
    );
    for s in &sheet.skipped {
        println!("  {s}");
    }
}
