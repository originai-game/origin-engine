//! `cargo run -p og-ui --example gen-theme -- <tokens.json> <theme.rs> <default.css>`:
//! writes the default style generated from the design language's tokens.
//! Use `crates/og-ui/scripts/gen-theme.sh`.

#[path = "theme_codegen.rs"]
mod theme_codegen;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [tokens, out, css] = &args[..] else {
        eprintln!("usage: gen-theme <tokens.json> <theme.rs> <default.css>");
        std::process::exit(2);
    };
    let text = std::fs::read_to_string(tokens).unwrap_or_else(|e| {
        eprintln!("{tokens}: {e}");
        std::process::exit(1);
    });
    match theme_codegen::generate(&text)
        .and_then(|rs| theme_codegen::generate_css(&text).map(|css| (rs, css)))
    {
        Ok((rs, sheet)) => {
            for (path, content) in [(out, rs), (css, sheet)] {
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
