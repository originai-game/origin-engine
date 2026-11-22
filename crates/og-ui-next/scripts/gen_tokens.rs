//! `cargo run -p og-ui-next --example gen-tokens -- <tokens dir> <tokens.json>`:
//! writes the design tokens as JSON (`packages/design/tokens.json`). Use
//! `crates/og-ui-next/scripts/gen-tokens.sh`.

#[path = "design_tokens.rs"]
mod design_tokens;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [tokens, out] = &args[..] else {
        eprintln!("usage: gen-tokens <tokens dir> <tokens.json>");
        std::process::exit(2);
    };
    let written = design_tokens::generate(std::path::Path::new(tokens))
        .map_err(|e| format!("{tokens}: {e}"))
        .and_then(|json| std::fs::write(out, json).map_err(|e| format!("{out}: {e}")));
    if let Err(e) = written {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
