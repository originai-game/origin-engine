//! `cargo run -p og-ui --example gen-theme -- <tokens.json> <theme.rs>`: writes
//! the generated theme. Use `crates/og-ui/scripts/gen-theme.sh`, which first
//! syncs the vendored token file from the platform.

#[path = "theme_codegen.rs"]
mod theme_codegen;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [tokens, out] = &args[..] else {
        eprintln!("usage: gen-theme <tokens.json> <theme.rs>");
        std::process::exit(2);
    };
    let source = match std::fs::read_to_string(tokens) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{tokens}: {e}");
            std::process::exit(1);
        }
    };
    match theme_codegen::generate(&source) {
        Ok(text) => {
            if let Err(e) = std::fs::write(out, text) {
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
