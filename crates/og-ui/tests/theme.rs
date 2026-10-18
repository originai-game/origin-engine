//! `src/theme.rs` is exactly what the generator writes from the vendored
//! platform tokens: a hand edit, a stale file or a token file changed without
//! regenerating all fail here. (`scripts/gen-theme.sh --check` additionally
//! compares the vendored copy with a platform checkout.)

#[path = "../scripts/theme_codegen.rs"]
mod theme_codegen;

use std::path::PathBuf;

#[test]
fn theme_rs_is_generated_from_the_vendored_tokens() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tokens = std::fs::read_to_string(root.join("tokens/tokens.json")).expect("tokens.json");
    let committed = std::fs::read_to_string(root.join("src/theme.rs")).expect("theme.rs");
    let generated = theme_codegen::generate(&tokens).expect("tokens generate");
    if committed != generated {
        let line = committed
            .lines()
            .zip(generated.lines())
            .position(|(a, b)| a != b)
            .unwrap_or(committed.lines().count().min(generated.lines().count()));
        panic!(
            "src/theme.rs differs from the generator's output at line {}:\n  committed: {:?}\n  generated: {:?}\nrun crates/og-ui/scripts/gen-theme.sh (never edit theme.rs by hand)",
            line + 1,
            committed.lines().nth(line),
            generated.lines().nth(line)
        );
    }
    // The theme the toolkit uses is the generated one.
    let dark = og_ui::Theme::dark();
    assert_eq!(dark.palette, og_ui::theme::DARK);
    assert_eq!(og_ui::Theme::light().palette, og_ui::theme::LIGHT);
    assert_eq!(dark.space, og_ui::theme::SPACE);
    assert_eq!(dark.type_ramp, og_ui::theme::TYPE_RAMP);
}
