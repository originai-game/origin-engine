//! `cargo run -p og-ui --example gen-theme -- <tokens.json> <theme.rs>`:
//! writes the default style generated from the design language's tokens.
//! Use `crates/og-ui/scripts/gen-theme.sh`.

#[path = "theme_codegen.rs"]
mod theme_codegen;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [tokens, out] = &args[..] else {
        eprintln!("usage: gen-theme <tokens.json> <theme.rs>");
        std::process::exit(2);
    };
    let text = std::fs::read_to_string(tokens).unwrap_or_else(|e| {
        eprintln!("{tokens}: {e}");
        std::process::exit(1);
    });
    match theme_codegen::generate(&text) {
        Ok(rs) => {
            if let Err(e) = std::fs::write(out, rs) {
                eprintln!("{out}: {e}");
                std::process::exit(1);
            }
            println!("wrote {out} from {tokens}");
        }
        Err(e) => {
            eprintln!("{tokens}: {e}");
            std::process::exit(1);
        }
    }
}
