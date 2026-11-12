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
    "meshlet-forest",
    "meshlet-occlusion",
    "meshlet-near",
    "meshlet-shadows",
    "meshlet-views",
    "meshlet-far",
    "meshlet-sway",
    "probe-interior",
    "baked-cornell",
    "probe-two-rooms",
    "probe-outdoor",
    "baked-batches",
    "terrain-generated",
    "plant-grown",
    "plant-scatter",
    "plant-scatter-switch",
    "ui-vector",
    "ui-rive",
    "gltf-uv-transform",
    "gltf-punctual-lights",
    "gltf-instancing",
    "gltf-variants-0",
    "gltf-variants-1",
];

/// Builds the scene called `name`: the scenes that cook here, every other
/// one through og-tests' [`og_tests::scenes::build`].
pub fn build(name: &str) -> Result<GfxScene, String> {
    let mut g = GfxScene {
        desc: base_desc(),
        scene: RenderScene::new(),
        tolerance: Tolerance::default(),
        cross_tolerance: Tolerance::CROSS_PLATFORM,
        frames: 1,
        update: None,
        render_scale: 1.0,
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
        // R1: the meshlet path (meshlet_scenes).
        #[cfg(not(target_arch = "wasm32"))]
        "meshlet-forest" => |g, _| {
            g.scene = crate::meshlet_scenes::forest_scene()?;
            Ok(())
        },
        #[cfg(not(target_arch = "wasm32"))]
        "meshlet-occlusion" => |g, _| {
            g.scene = crate::meshlet_scenes::occlusion_scene(true)?;
            // The second and third frames cull in two phases.
            g.frames = 3;
            Ok(())
        },
        #[cfg(not(target_arch = "wasm32"))]
        "meshlet-near" => |g, _| {
            g.scene = crate::meshlet_scenes::near_scene(true)?;
            Ok(())
        },
        #[cfg(not(target_arch = "wasm32"))]
        "meshlet-shadows" => |g, _| {
            g.scene = crate::meshlet_scenes::shadow_scene(true)?;
            Ok(())
        },
        #[cfg(not(target_arch = "wasm32"))]
        "meshlet-views" => |g, _| {
            g.scene = crate::meshlet_scenes::views_scene(true, 1.0)?;
            // The probe's capture, then the frame that shows it.
            g.frames = 2;
            Ok(())
        },
        #[cfg(not(target_arch = "wasm32"))]
        "meshlet-far" => |g, _| {
            g.scene = crate::meshlet_scenes::far_scene(true)?;
            Ok(())
        },
        #[cfg(not(target_arch = "wasm32"))]
        "meshlet-sway" => |g, _| {
            g.scene = crate::meshlet_scenes::sway_scene(1.0, true)?;
            Ok(())
        },
        // Round 3: a terrain the cook generated (noise, erosion, a sea,
        // rivers, biomes), its water at the cooked sea level.
        #[cfg(not(target_arch = "wasm32"))]
        "terrain-generated" => |g, _| {
            g.scene = crate::terrain_scenes::generated_scene()?;
            Ok(())
        },
        // Plants the cook grows from a `plant` block, and a wood of them
        // scattered whole (plant_scenes).
        #[cfg(not(target_arch = "wasm32"))]
        "plant-grown" => |g, _| {
            g.scene = crate::plant_scenes::grown_scene()?;
            Ok(())
        },
        #[cfg(not(target_arch = "wasm32"))]
        "plant-scatter" => |g, _| {
            g.scene = crate::plant_scenes::scatter_scene()?;
            Ok(())
        },
        #[cfg(not(target_arch = "wasm32"))]
        "plant-scatter-switch" => |g, _| {
            g.scene = crate::plant_scenes::switch_scene()?;
            // As a game draws a wood: leaves alias without it, cards do not.
            g.desc.antialias = og_render::Antialias::Taa;
            g.frames = 16;
            Ok(())
        },
        #[cfg(not(target_arch = "wasm32"))]
        "ui-rive" => |g, _| {
            g.scene = crate::ui_scenes::rive()?;
            Ok(())
        },
        #[cfg(not(target_arch = "wasm32"))]
        "ui-vector" => |g, _| {
            g.scene = crate::ui_scenes::vector()?;
            Ok(())
        },
        // G1-3: glTF extensions through the cook (gltf_scenes).
        #[cfg(not(target_arch = "wasm32"))]
        "gltf-uv-transform" => |g, _| {
            g.scene = crate::gltf_scenes::uv_transform_scene()?.0;
            Ok(())
        },
        #[cfg(not(target_arch = "wasm32"))]
        "gltf-punctual-lights" => |g, _| {
            g.scene = crate::gltf_scenes::lights_scene()?.0;
            Ok(())
        },
        #[cfg(not(target_arch = "wasm32"))]
        "gltf-instancing" => |g, _| {
            g.scene = crate::gltf_scenes::instancing_scene()?.0;
            Ok(())
        },
        #[cfg(not(target_arch = "wasm32"))]
        "gltf-variants-0" | "gltf-variants-1" => |g, name| {
            let variant = usize::from(name == "gltf-variants-1");
            g.scene = crate::gltf_scenes::variants_scene(variant)?.0;
            Ok(())
        },
        other => return og_tests::scenes::build(other),
    };
    scene(&mut g, name)?;
    // The meshlet path's scenes draw through it whatever their size: it
    // follows GPU culling, which `Culling::Auto` keeps for frames of
    // GPU_CULLING_MIN_INSTANCES instances and these have a few hundred.
    if name.starts_with("meshlet-") {
        g.desc.culling = og_render::Culling::Gpu;
    }
    Ok(g)
}
