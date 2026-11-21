//! `cargo run -p og-ui-next --example gen-pack -- <out>`: writes the
//! default pack (`default/pack.json`) from the design tokens.

fn main() {
    let Some(out) = std::env::args().nth(1) else {
        eprintln!("usage: gen-pack <default/pack.json>");
        std::process::exit(2);
    };
    std::fs::write(&out, og_ui_next::pack::design::default_pack_json()).expect("write the default pack");
}
