//! og-tests' gfx scenes ([`og_tests::scenes`]) and the ones that cook a
//! project, in one registry: [`build`] knows both, and `og-gfx --all`
//! renders [`SCENES`] and [`NATIVE_SCENES`].

use og_render::RenderScene;
use og_tests::compare::Tolerance;
pub use og_tests::scenes::*;

/// Scenes rendered natively only, beside [`SCENES`] (`og-gfx --all` takes
/// both): their content is cooked by og-cook at build time, which the
/// browser's wasm build of the scenes does not carry.
#[cfg(not(target_arch = "wasm32"))]
pub const NATIVE_SCENES: &[&str] = &[
    "vox-keyframes",
    "skeleton-2d",
    "lod-levels",
    "probe-interior",
    "ssr-wet-floor",
    "baked-cornell",
    "probe-two-rooms",
    "probe-outdoor",
    "baked-batches",
    "terrain-generated",
    "ui-vector",
];

/// Builds the scene called `name`: the scenes that cook here, every other
/// one through og-tests' [`og_tests::scenes::build`].
pub fn build(name: &str) -> Result<GfxScene, String> {
    let mut g = GfxScene {
        desc: base_desc(),
        scene: RenderScene::new(),
        tolerance: Tolerance::default(),
        frames: 1,
        update: None,
    };
    let scene: fn(&mut GfxScene, &str) -> Result<(), String> = match name {
        // Owed row 63: a .vox transform keyed across frames, cooked and
        // posed at frames 0 and 2 side by side.
        #[cfg(not(target_arch = "wasm32"))]
        "vox-keyframes" => |g, _| {
            g.scene = crate::vox_scenes::keyframes_scene()?;
            Ok(())
        },
        #[cfg(not(target_arch = "wasm32"))]
        "lod-levels" => |g, _| {
            g.scene = crate::depth_render_scenes::lod_scene()?.0;
            Ok(())
        },
        // Round 3: a terrain the cook generated (noise, erosion, a sea,
        // rivers, biomes), its water at the cooked sea level.
        #[cfg(not(target_arch = "wasm32"))]
        "terrain-generated" => |g, _| {
            g.scene = crate::terrain_scenes::generated_scene()?;
            Ok(())
        },
        #[cfg(not(target_arch = "wasm32"))]
        "ui-vector" => |g, _| {
            g.scene = crate::ui_scenes::vector()?;
            Ok(())
        },
        other => return og_tests::scenes::build(other),
    };
    scene(&mut g, name)?;
    Ok(g)
}
