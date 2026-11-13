//! gfx reference frames of the scenes that cook (og-tests' tests/gfx.rs
//! has the others): each matches its committed reference in og-tests'
//! references/, plus behaviour read from the frames.

use og_tests_cooked::gfx;
use std::path::PathBuf;

#[path = "gfx/effect_beam.rs"]
mod effect_beam;
#[path = "gfx/gltf_extensions.rs"]
mod gltf_extensions;
#[path = "gfx/impostors.rs"]
mod impostors;
#[path = "gfx/meshlets.rs"]
mod meshlets;
#[path = "gfx/overlay_effects.rs"]
mod overlay_effects;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-gfx")
}

fn check(scene: &str) -> gfx::GfxReport {
    match gfx::check(scene, &out_dir()) {
        Ok(report) => {
            println!("{report}");
            report
        }
        Err(e) => panic!("{e}"),
    }
}

macro_rules! references {
    ($($name:ident => $scene:literal),+ $(,)?) => {
        $(#[test]
        #[ignore = "requires GPU"]
        fn $name() {
            check($scene);
        })+
    };
}

references! {
    gltf => "gltf-reference",
    physical_materials => "materials-physical",
    skinning => "skinning",
    crowds => "crowd-lods",
    geometry => "geometry-shapes",
    lights => "lights-256",
    shadows => "shadows-soft",
    ambient_occlusion => "ao-gtao",
    global_illumination => "gi-room",
    reflections => "ssr-wet-floor",
    bloom => "bloom-unreal",
    depth_of_field => "dof-high",
    temporal_upscale => "taau-moving",
    tonemap => "post-agx",
    print => "stylise-print",
    toon => "stylise-toon-physical",
    atmosphere => "atmosphere-moon",
    fog => "volumetric-fog",
    clouds => "clouds-sunset",
    weather => "weather-rain",
    ocean => "water-ocean",
    lake => "water-planar",
    lake_shadows => "water-shadows",
    underwater => "water-underwater",
    underwater_shafts => "water-underwater-shafts",
    fluids => "fluid-3d",
    terrain => "terrain-layers",
    grass => "terrain-grass",
    meshlets => "meshlet-forest",
    particles => "particles",
    trails => "trails-beams",
    effect_materials => "effect-materials",
    sprites => "sprites-array",
    text => "text-msdf",
    tilemap => "tilemap-lit",
    ui_shapes => "ui-shapes",
    ui_paths => "ui-paths",
    ui_ground => "ui-ground",
    ui_glass => "ui-glass-dialog",
    hooks => "hooks-planar",
    views => "views-split",
    garment => "visual-garment",
    environment => "environment-map",
    decals => "decals",
}

/// A frame at a limited tier, on a full device: each of
/// [`MINI_GAME_SCENES`](og_tests_cooked::scenes::MINI_GAME_SCENES) matches
/// its own reference, drawn by a renderer whose capabilities the tier
/// clamped (no MRT, asynchronous pipelines or timestamps, whatever the
/// adapter has), so a pass that ignores the clamp or a fallback that
/// stops drawing moves a frame here.
#[test]
#[ignore = "requires GPU"]
fn frames_at_the_mini_game_tier() {
    use og_render::{Caps, Tier};
    for scene in og_tests_cooked::scenes::MINI_GAME_SCENES {
        let report = check(scene);
        assert_eq!(report.info.tier, Tier::MiniGame, "{scene}");
        for lacking in [Caps::MRT, Caps::ASYNC_PIPELINES, Caps::TIMESTAMPS] {
            assert!(!report.info.caps.contains(lacking), "{scene}: {lacking:?}");
        }
    }
}
