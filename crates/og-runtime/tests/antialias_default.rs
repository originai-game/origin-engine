//! The antialias default follows the project's profile: a 2D project's
//! `tier` (RenderOptions.antialias left out, or tier) resolves to 4x MSAA,
//! since TAA's jitter and history blend soften nearest-sampled pixel art; a
//! 3D project's stays tier (TAA); a game that names a method keeps it.
//!
//! Ways this could fail: the default stops reading the profile (a 2D game
//! back on TAA), the 2D default leaks into 3D, or the profile overrides a
//! method the game chose.

use og_render::{Antialias, RenderDesc};
use og_runtime::{og_engine_create, og_game_game, og_host_begin_bundle, render_desc};
use serde_json::{Value, json};

fn resolved(profile: &str, render: Value) -> Antialias {
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 1, "render": render }).to_string()));
    let v: Value = serde_json::from_str(&og_engine_create(
        game,
        json!({ "profile": profile }).to_string(),
    ))
    .unwrap();
    let e = v["value"].as_u64().unwrap_or_else(|| panic!("{v}")) as u32;
    render_desc(e, RenderDesc::default()).antialias
}

#[test]
fn a_2d_project_defaults_to_msaa4_and_an_explicit_method_wins() {
    assert_eq!(resolved("2d", json!({})), Antialias::Msaa4);
    assert_eq!(
        resolved("2d", json!({ "antialias": "tier" })),
        Antialias::Msaa4
    );
    assert_eq!(resolved("3d", json!({})), Antialias::Tier);
    assert_eq!(
        resolved("3d", json!({ "antialias": "tier" })),
        Antialias::Tier
    );
    assert_eq!(
        resolved("2d", json!({ "antialias": "taa" })),
        Antialias::Taa
    );
    assert_eq!(
        resolved("2d", json!({ "antialias": "off" })),
        Antialias::Off
    );
    assert_eq!(
        resolved("3d", json!({ "antialias": "msaa4" })),
        Antialias::Msaa4
    );
}
