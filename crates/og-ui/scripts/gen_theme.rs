//! `cargo run -p og-ui --example gen-theme -- <tokens dir> <theme.rs> <default.css> <tokens.json>`:
//! writes the default style generated from the design language's tokens,
//! and the tokens as JSON.
//! Use `crates/og-ui/scripts/gen-theme.sh`.

#[path = "theme_codegen.rs"]
mod theme_codegen;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [tokens, out, css, json] = &args[..] else {
        eprintln!("usage: gen-theme <tokens dir> <theme.rs> <default.css> <tokens.json>");
        std::process::exit(2);
    };
    match theme_codegen::load(std::path::Path::new(tokens)).and_then(|t| {
        Ok((
            theme_codegen::generate(&t)?,
            theme_codegen::generate_css(&t)?,
            theme_codegen::json(&t)?,
        ))
    }) {
        Ok((rs, sheet, tree)) => {
            for (path, content) in [(out, rs), (css, sheet), (json, tree)] {
                if let Err(e) = std::fs::write(path, content) {
                    eprintln!("{path}: {e}");
                    std::process::exit(1);
                }
                println!("wrote {path} from {tokens}");
            }
        }
        Err(e) => {
            eprintln!("{tokens}: {e}");
            std::process::exit(1);
        }
    }
}
