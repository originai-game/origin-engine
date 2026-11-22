//! `packages/design/tokens.json` is exactly what the generator writes from
//! `packages/design/tokens/*.css`: a hand edit, a stale file or a token
//! changed without regenerating fails here.

#[path = "../scripts/design_tokens.rs"]
mod design_tokens;

#[test]
fn tokens_json_is_the_generators_output() {
    let design = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/design");
    let made = design_tokens::generate(&design.join("tokens")).expect("the tokens read");
    let kept = std::fs::read_to_string(design.join("tokens.json")).expect("tokens.json");
    assert!(
        made == kept,
        "packages/design/tokens.json is stale: run crates/og-ui-next/scripts/gen-tokens.sh"
    );
}
