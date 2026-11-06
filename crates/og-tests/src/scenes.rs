//! The named gfx scenes (the reference set). Each one
//! is a fixed [`RenderDesc`] and [`RenderScene`], how many frames to render
//! before the capture (with an optional per-frame update), and the
//! tolerance its reference comparison uses.

use crate::compare::Tolerance;
use crate::fx_scenes as fx;
use glam::{Mat4, Quat, Vec2, Vec3};
use og_render::gltf::{GltfModel, load_glb};
use og_render::{
    Antialias, Background, Bloom, Camera, ColorSpace, DirectionalLight, Environment, Exposure,
    Glyph, Light, LightKind, Material, Mips, MsdfFont, ProceduralSky, Projection, RenderDesc,
    RenderScene, ShadowBounds, Sprite, Text, TextureData, Tonemap, shapes,
};
use std::path::PathBuf;

mod geometry;
mod models;
mod sprites;
mod text;
mod tilemap;

pub use geometry::*;
pub(crate) use models::*;
pub use sprites::*;
pub use text::*;
pub use tilemap::*;

/// Every scene the harness knows, in the order `og-gfx --all` renders them.
pub const SCENES: &[&str] = &[
    "gltf-reference",
    "spheres-20k",
    "cascades-200m",
    "point-spot-lights",
    "taa",
    "bloom",
    "exposure",
    "sprites-20k",
    "text-msdf",
    "sky-only",
    "showcase",
    "overlay",
    "particles",
    "particles-cpu",
    "particles-lit",
    "fx-2d",
    "trails-beams",
    "decals",
    "decals-many",
    "post-grade",
    "post-agx",
    "environment-map",
    "environment-blurry",
    "environment-blend",
    "environment-grounded",
    "environment-uncooked",
    "ao-gtao",
    "ao-ssao",
    "gi-room",
    "gi-room-taa",
    "gi-room-probes",
    "gi-shelter",
    "light-volume-room",
    "light-volume-mirror",
    "light-volume-glossy",
    "light-volume-rough",
    "ssr-wet-floor",
    "dof-low",
    "dof-high",
    "god-rays",
    "flare-noon",
    "flare-edge",
    "flare-lights",
    "motion-blur",
    "stylised",
    "skinning",
    "crowd-culling",
    "crowd-lods",
    "atmosphere-fog",
    "atmosphere-dusk",
    "atmosphere-night",
    "water-3d",
    "water-2d",
    "fluid-2d",
    "fluid-groups",
    "fluid-3d",
    "shader-hooks",
    "weather-rain",
    "weather-snow",
    "terrain-near",
    "terrain-mid",
    "terrain-far",
    "water-planar",
    "water-shadows",
    "water-underwater",
    "terrain-grass",
    "lights-256",
    "atmosphere-moon",
    "materials",
    "skinned-hooks",
    "materials-physical",
    "shadows-soft",
    "point-shadows-paraboloid",
    "shadows-local-8",
    "shadows-local-contact",
    "shadows-cascades-4",
    "fxaa",
    "smaa",
    "smaa-t2x",
    "smaa-t2x-moving",
    "taa-sharp",
    "taau",
    "taau-moving",
    "stylise-pixel",
    "stylise-palette",
    "stylise-print",
    "stylise-halftone",
    "stylise-paint",
    "stylise-bands",
    "stylise-retro",
    "stylise-material",
    "sprites-array",
    "stylise-ink-depth",
    "sprites-shadow",
    "sprites-billboard",
    "sprites-array-fx",
    "sprites-shadow-terrain",
    "stylise-surface",
    "stylise-object",
    "stylise-halftone-surface",
    "materials-array",
    "stylise-outline-voxels",
    "stylise-toon-physical",
    "sprites-untonemapped",
    "stylise-bands-tint",
    "tilemap-lit",
    "model-flash",
    "shadows-crypt",
    "shadow-flags",
    "ground-grid",
    "stylise-outline-ground",
    "geometry-procedural",
    "lines",
    "points",
    "wireframe",
    "debug-overlay",
    "views-monitor",
    "views-mirror",
    "views-split",
    "views-layers",
    "views-depth",
    "shader-full-dissolve",
    "post-crt",
    "rect-lights",
    "light-cookies",
    "volumetric-fog",
    "geometry-shapes",
    "water-mesh",
    "water-ripples",
    "water-fresnel",
    "water-glint",
    "shadow-box",
    "hemisphere-light",
    "ao-direct",
    "ao-screen-water",
    "post-sharpen",
    "bloom-unreal",
    "alpha-to-coverage",
    "hooks-eye-front",
    "hooks-planar",
    "hooks-screen-ao",
    "water-planar-two",
    "terrain-hidden",
    "terrain-scatter-unshadowed",
    "post-adjust",
    "post-neutral",
    "post-lens",
    "terrain-grass-wind",
    "terrain-holes",
    "terrain-layers",
    "terrain-tiling",
    "terrain-steep",
    "terrain-horizon",
    "terrain-displaced",
    "terrain-cliff",
    "terrain-cliff-blend",
    "impostors",
    "impostor-scatter",
    "wind-models",
    "grass-near",
    "grass-mid",
    "grass-far",
    "grass-backlit",
    "grass-wind",
    "post-blur",
    "ui-shapes",
    "ui-paints",
    "ui-effects",
    "ui-layers",
    "ui-text-effects",
    "ui-world",
    "ui-panel",
    "ui-glass-bright",
    "ui-glass-dark",
    "ui-glass-busy",
    "ui-glass-dialog",
    "ui-glass-fallback",
    "sky-physical-noon",
    "sky-physical-sunset",
    "sky-physical-twilight",
    "sky-physical-hazy",
    "clouds-fair",
    "clouds-overcast",
    "clouds-sunset",
    "clouds-gradient",
    "weather-wet",
    "weather-snow-cover",
    "weather-lightning",
    "weather-shelter-snow",
    "weather-shelter-rain",
    "weather-shelter-meshlet",
    "weather-street-wet",
    "weather-puddles",
    "weather-snow-buildup",
    "weather-dust-desert",
    "materials-cloth",
    "weather-cobbles-wet",
    "aurora-arcs",
    "aurora-breakup",
    "live-sky",
    "water-ocean",
    "water-ocean-storm",
    "water-ocean-sunset",
    "water-flow-map",
    "water-ocean-dusk-storm",
    "water-ocean-night",
    "water-shore",
    "water-shore-storm",
    "water-shallows",
    "water-wake",
    "water-river",
];

/// A per-frame change (frame index from 0): moving sprites, an orbiting camera.
pub type Update = fn(&mut RenderScene, u32);

pub struct GfxScene {
    pub desc: RenderDesc,
    pub scene: RenderScene,
    pub tolerance: Tolerance,
    /// Frames rendered before the capture (TAA needs its history).
    pub frames: u32,
    pub update: Option<Update>,
    /// The renderer's render scale (`Renderer::set_render_scale`; 1).
    pub render_scale: f32,
}

/// The directory of the committed test assets (`crates/og-tests/assets`),
/// or `OG_TEST_ASSETS` (the sandbox capture runs the binary elsewhere).
pub fn assets_dir() -> PathBuf {
    match std::env::var_os("OG_TEST_ASSETS") {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
    }
}

/// The bytes of the test asset `rel` (a path under [`assets_dir`]). In the
/// browser (og-tests-web) there is no file system: the assets the scenes
/// read are compiled in.
#[cfg(not(target_arch = "wasm32"))]
pub fn asset(rel: &str) -> Result<std::borrow::Cow<'static, [u8]>, String> {
    let path = assets_dir().join(rel);
    std::fs::read(&path)
        .map(Into::into)
        .map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(target_arch = "wasm32")]
pub fn asset(rel: &str) -> Result<std::borrow::Cow<'static, [u8]>, String> {
    let bytes: &'static [u8] = match rel {
        "trickster.glb" => include_bytes!("../assets/trickster.glb"),
        "dropship.glb" => include_bytes!("../assets/dropship.glb"),
        "fonts/noto-sans-sc-msdf.png" => include_bytes!("../assets/fonts/noto-sans-sc-msdf.png"),
        "fonts/noto-sans-sc-msdf.json" => include_bytes!("../assets/fonts/noto-sans-sc-msdf.json"),
        "fonts/noto-sans-sc-msdf.csv" => include_bytes!("../assets/fonts/noto-sans-sc-msdf.csv"),
        "env/studio.hdr" => include_bytes!("../assets/env/studio.hdr"),
        _ => {
            return Err(format!(
                "{rel}: not compiled into the wasm harness (scenes::asset)"
            ));
        }
    };
    Ok(bytes.into())
}

fn asset_text(rel: &str) -> Result<String, String> {
    String::from_utf8(asset(rel)?.into_owned()).map_err(|e| format!("{rel}: {e}"))
}

/// The renderer description every scene starts from: 1280x720 on the
/// adapter `OG_GFX_BACKEND` names (`metal`, `vulkan`; default: the platform's).
pub fn base_desc() -> RenderDesc {
    let backend = match std::env::var("OG_GFX_BACKEND").as_deref() {
        Ok("metal") => og_render::Backend::Metal,
        Ok("vulkan") => og_render::Backend::Vulkan,
        _ => og_render::Backend::Auto,
    };
    RenderDesc {
        width: 1280,
        height: 720,
        backend,
        ..Default::default()
    }
}

fn crowd_perf(g: &mut GfxScene, count: u32) -> Result<(), String> {
    g.scene = fx::crowd_scene(count);
    g.update = Some(fx::walk_crowd);
    g.desc.antialias = Antialias::Tier;
    g.frames = 8;
    Ok(())
}

/// Builds the scene called `name`.
pub fn build(name: &str) -> Result<GfxScene, String> {
    let mut g = GfxScene {
        desc: base_desc(),
        scene: RenderScene::new(),
        tolerance: Tolerance::default(),
        frames: 1,
        update: None,
        render_scale: 1.0,
    };
    // Each scene is built in a closure of its own, so its temporaries live
    // in that closure's frame. Unoptimised, one function's frame holds every
    // match arm's temporaries side by side (a RenderScene is 3.9 KB): this
    // match built inline took 804 KB of stack, and a glTF import on top
    // overflowed Windows' 1 MiB main thread in a debug og-gfx.
    let scene: fn(&mut GfxScene, &str) -> Result<(), String> = match name {
        // The spike's reference scene (two glTF models x 25 on a checker
        // ground), drawn with the sky pass, mipmapped textures and CSM.
        "gltf-reference" => |g, _| {
            g.scene = spike_gltf_scene()?;
            Ok(())
        },
        // 20,000 low-poly spheres submitted one by one: two batches in all.
        "spheres-20k" => |g, _| {
            g.scene = sphere_field(20_000);
            Ok(())
        },
        // The same field with every sphere bobbing each frame (the bench's
        // sphere field, tools/bench/spheres): the renderer's per-instance
        // cost when every transform changes. Perf only (no reference).
        "spheres-20k-moving" => |g, _| {
            g.scene = sphere_field(20_000);
            g.update = Some(bob_spheres);
            Ok(())
        },
        // Crowds at scale (fx_scenes/crowds.rs): 5,000, 20,000 and 50,000
        // walkers playing one baked walk, every one moving and stepping
        // each frame, under TAA and a shadowing sun. Perf only.
        // Crowds' levels and occlusion (fx_scenes/crowds.rs): an avenue
        // of about 2,100 people in twelve outfits from 4 m to 160 m, a
        // gate across it.
        "crowd-lods" => |g, _| {
            g.scene = fx::crowd_lods_scene();
            g.desc.antialias = Antialias::Tier;
            g.frames = 12;
            Ok(())
        },
        // The same at 2,000 walkers: GPU culling's crowd scene (culling.rs).
        "crowd-2k" => |g, _| crowd_perf(g, 2_000),
        "crowd-5k" => |g, _| crowd_perf(g, 5_000),
        "crowd-20k" => |g, _| crowd_perf(g, 20_000),
        "crowd-50k" => |g, _| crowd_perf(g, 50_000),
        // Visual bodies on the GPU (og-render's VisualBody): banners in a
        // gusting wind, a sheet draped over a ball, a jelly and a balloon,
        // after TICKS ticks one frame each.
        "visual-bodies" => |g, _| {
            g.scene = crate::visual_scenes::visual_bodies_scene(true);
            g.desc.antialias = Antialias::Taa;
            g.frames = crate::visual_scenes::TICKS;
            g.update = Some(crate::visual_scenes::next_tick);
            Ok(())
        },
        // A visual garment on the GPU: a skirt on a mannequin in a gusting
        // wind, held off the legs and hips, after GARMENT_TICKS ticks.
        "visual-garment" => |g, _| {
            g.scene = crate::visual_scenes::visual_garment_scene(true);
            g.desc.antialias = Antialias::Taa;
            g.frames = crate::visual_scenes::GARMENT_TICKS;
            g.update = Some(crate::visual_scenes::next_tick);
            Ok(())
        },
        // Compute particles meeting the scene's colliders: a grinder's
        // sparks ricocheting off a block, a plate, a ball and the floor.
        "visual-particles" => |g, _| {
            g.scene = crate::visual_scenes::visual_particles_scene();
            g.desc.antialias = Antialias::Taa;
            g.frames = crate::visual_scenes::SPARK_TICKS;
            g.update = Some(crate::visual_scenes::next_particle_tick);
            Ok(())
        },
        // A visual fluid on the GPU (MLS-MPM): a dam break in a basin
        // round a ball, with a spout pouring in, after FLUID_TICKS ticks.
        "visual-fluid" => |g, _| {
            g.scene = crate::visual_scenes::visual_fluid_scene(true);
            g.desc.antialias = Antialias::Taa;
            g.frames = crate::visual_scenes::FLUID_TICKS;
            g.update = Some(crate::visual_scenes::next_tick);
            Ok(())
        },
        // A 400 m field with pillars from 3 m to 190 m: shadows in every cascade.
        "cascades-200m" => |g, _| {
            g.scene = cascades_200m(true);
            Ok(())
        },
        // Night: seven point and spot lights, the first two shadowed.
        "point-spot-lights" => |g, _| {
            g.scene = point_spot_lights();
            Ok(())
        },
        // Thin geometry after 16 frames of TAA (static camera, converged).
        "taa" => |g, _| {
            g.scene = thin_geometry();
            g.desc.antialias = Antialias::Tier;
            g.frames = 16;
            Ok(())
        },
        // The same geometry with FXAA (what `Msaa4` draws without a
        // multisampled HDR target).
        "fxaa" => |g, _| {
            g.scene = thin_geometry();
            g.desc.antialias = Antialias::Fxaa;
            Ok(())
        },
        "smaa" => |g, _| {
            g.scene = thin_geometry();
            g.desc.antialias = Antialias::Smaa;
            Ok(())
        },
        "smaa-t2x" => |g, _| {
            g.scene = thin_geometry();
            g.desc.antialias = Antialias::SmaaT2x;
            g.frames = 2;
            Ok(())
        },
        "smaa-t2x-moving" => |g, _| {
            g.scene = thin_geometry();
            g.desc.antialias = Antialias::SmaaT2x;
            g.render_scale = 0.5;
            g.frames = 24;
            g.update = Some(crate::shadow_scenes::pan_and_roll_sphere);
            Ok(())
        },
        // The same geometry under TAA with the camera sliding sideways:
        // Catmull-Rom history and a 0.5 sharpening pass, after 16 frames.
        // The browser tier judges a moving camera's reprojection against
        // metal here.
        // Temporal upscaling: the thin geometry drawn at render scale 0.5
        // under TAA and resolved by TAA to the output over the jitter's 32
        // phases (twice over).
        "taau" => |g, _| {
            g.scene = thin_geometry();
            g.desc.antialias = Antialias::Taa;
            g.render_scale = 0.5;
            g.frames = 64;
            Ok(())
        },
        // TAAU with the camera sliding and the sphere crossing the view
        // with its previous transforms (the prepass's motion vectors).
        "taau-moving" => |g, _| {
            g.scene = thin_geometry();
            g.desc.antialias = Antialias::Taa;
            g.render_scale = 0.5;
            g.frames = 24;
            g.update = Some(crate::shadow_scenes::pan_and_roll_sphere);
            Ok(())
        },
        "taa-sharp" => |g, _| {
            g.scene = thin_geometry();
            g.desc.antialias = Antialias::Taa;
            g.desc.taa_history = og_render::TaaHistory::CatmullRom;
            g.desc.taa_sharpen = 0.5;
            g.frames = 16;
            g.update = Some(crate::shadow_scenes::pan_thin_geometry);
            Ok(())
        },
        // The sun's soft shadows (PCSS) with cascade blending: contact
        // shadows sharp, penumbrae widening with the caster's height.
        "shadows-soft" => |g, _| {
            g.scene = crate::shadow_scenes::soft_shadows_scene(true);
            Ok(())
        },
        // The same with hard shadows and no blending (evidence only, no reference).
        "shadows-soft-off" => |g, _| {
            g.scene = crate::shadow_scenes::soft_shadows_scene(false);
            Ok(())
        },
        // Cost measurements (evidence only, no references): point-spot-lights
        // and shadows-local-8 with each point shadow storage forced, and the
        // thin geometry without antialiasing.
        "point-shadows-cube" => |g, _| {
            g.scene = point_spot_lights();
            g.desc.point_shadows = og_render::PointShadows::Cube;
            Ok(())
        },
        "shadows-local-8-cube" | "shadows-local-8-paraboloid" => |g, name| {
            g.scene = crate::shadow_scenes::many_shadowed_lights(true);
            g.desc.point_shadows = if name.ends_with("cube") {
                og_render::PointShadows::Cube
            } else {
                og_render::PointShadows::Paraboloid
            };
            Ok(())
        },
        "aa-off" => |g, _| {
            g.scene = thin_geometry();
            Ok(())
        },
        // taa-sharp's moving camera with a bilinear history and no
        // sharpening (evidence only, no reference).
        "taa-motion" => |g, _| {
            g.scene = thin_geometry();
            g.desc.antialias = Antialias::Taa;
            g.frames = 16;
            g.update = Some(crate::shadow_scenes::pan_thin_geometry);
            Ok(())
        },
        // point-spot-lights with the point light's shadow in two paraboloid
        // hemispheres instead of six cube faces (`PointShadows::Paraboloid`).
        "point-shadows-paraboloid" => |g, _| {
            g.scene = point_spot_lights();
            g.desc.point_shadows = og_render::PointShadows::Paraboloid;
            Ok(())
        },
        // A roofed room on a sunlit field: the sun reaches the floor only
        // through the door, and a shadowed point light inside a glass box
        // that casts no shadow lights the altar and the pillar (its shadow
        // in cube faces falls behind a post).
        "shadows-crypt" => |g, _| {
            g.scene = crate::shadow_scenes::crypt_scene();
            Ok(())
        },
        // Per-instance shadow flags under a low sun: a model that casts as
        // added beside one that casts none, and behind a wall's shadow a
        // box and a model that receive beside a box and a model that do
        // not (lit as if unshadowed).
        "shadow-flags" => |g, _| {
            g.scene = crate::shadow_scenes::shadow_flags_scene()?.scene;
            Ok(())
        },
        // meridian-loop #15: a 1,600 m ground as og.render.planeGeometry
        // makes it, its grid tile repeated every 10 m by `uvRepeat` 160.
        "ground-grid" => |g, _| {
            g.scene = ground_grid(160.0);
            Ok(())
        },
        // meridian-loop #7: neon-vector's outline over a 1,600 m ground
        // from a chase camera: lined where it ends, never washed.
        "stylise-outline-ground" => |g, _| {
            g.scene = crate::stylise_scenes::outline_ground_scene(1.4, 120.0);
            Ok(())
        },
        // Eight shadowed lights (six point, two spot) and two more far
        // behind whose shadows are left out (the default quality's eight).
        // A box and a ball on a plain floor lit by one shadowed spot: the
        // box's shadow starts at its base (no lit line), its edges soft
        // and straight (the tent PCF).
        "shadows-local-contact" => |g, _| {
            g.scene = crate::shadow_scenes::local_contact_scene(false, true);
            Ok(())
        },
        // The 200 m street of pillars and kerbs under four cascades (the
        // high preset's count): the near pillars' shadows finer than three
        // cascades give over the same distance.
        "shadows-cascades-4" => |g, _| {
            g.scene = crate::scenes::cascades_200m(true);
            g.scene.quality.shadow_cascades = 4;
            Ok(())
        },
        "shadows-local-8" => |g, _| {
            g.scene = crate::shadow_scenes::many_shadowed_lights(true);
            Ok(())
        },
        // Emissive spheres over a dark ground with bloom.
        "bloom" => |g, _| {
            g.scene = emissive_spheres();
            g.scene.post.bloom = Some(Bloom {
                threshold: 1.0,
                knee: 0.5,
                intensity: 0.8,
            });
            Ok(())
        },
        // A dim scene brought to middle grey by metered exposure.
        "exposure" => |g, _| {
            g.scene = dim_scene(1.0);
            Ok(())
        },
        // 20,000 sprites, three textures (one an atlas), two layers.
        "sprites-20k" => |g, _| {
            g.scene = sprite_field(20_000)?;
            g.update = Some(move_sprites);
            Ok(())
        },
        // Sprites drawn from texture arrays: an eight-frame pixel-art
        // flipbook (nearest) in every frame, flipped and turned, and a
        // three-layer smooth array (linear, mipmapped) added, around one
        // plain textured sprite; one draw per array and blend.
        "sprites-array" => |g, _| {
            g.scene = sprite_array_scene();
            Ok(())
        },
        // Pixel figures standing in a 3D set, each with a drop shadow on
        // the ground along the sun (one without, one from a texture
        // array), beside a pillar and a crate casting their own.
        "sprites-shadow" => |g, _| {
            g.scene = sprites_shadow_scene();
            Ok(())
        },
        // Billboards (RenderScene::sprite_facing) in a 3D set seen from
        // above and to the side: a flat figure for comparison, upright
        // (cameraY) trees and figures casting sun-facing drop shadows, a
        // camera-facing turned sign, a 2D-lit figure, two translucent
        // camera-facing puffs sorted back to front though submitted near
        // first, one billboard behind the camera culled, and a
        // picture-in-picture view from the side that every billboard
        // faces too.
        "sprites-billboard" => |g, _| {
            g.scene = billboard_scene();
            Ok(())
        },
        // Drop shadows off the plane y = 0: figures on a hilly terrain
        // (each shadow on the plane of the slope under its feet), one on a
        // raised deck (its shadow at the deck's height), one beside a wall
        // whose cascaded shadow its drop shadow fades into, and one whose
        // shadow bends into a hollow narrower than itself (ShadowBend).
        "sprites-shadow-terrain" => |g, _| {
            g.scene = sprites_shadow_terrain_scene();
            Ok(())
        },
        // Texture arrays through every sprite path: glowing array sprites
        // (the glow pass), two 2D particle emitters drawing from arrays
        // (layers by age, and at random with the soft dot's round edge,
        // glowing), and overlay sprites from an array beside a plain one.
        "sprites-array-fx" => |g, _| {
            g.scene = sprite_array_fx_scene();
            g.frames = 60;
            g.update = Some(fx::tick_update);
            Ok(())
        },
        // MSDF text at several sizes, rotated, Latin and CJK.
        "text-msdf" => |g, _| {
            g.scene = text_scene()?;
            Ok(())
        },
        // E4: a 256x256 tilemap with four layers (one animated, one at half
        // parallax) through og-tilemap's chunks, the camera panning 80x45
        // tiles of it (16 px tiles at 1280x720). Perf only (no reference).
        "tilemap-256" => |g, _| {
            g.scene = tilemap_256();
            g.update = Some(pan_tilemap);
            Ok(())
        },
        // R11: 64 shadowed 2D lights over 128 occluders at the default 1024
        // shadow angles, at 4096, and with normal-mapped crates (the
        // light directions drawn too). Perf only (no reference).
        "lights-2d-64" => |g, _| {
            g.scene = lights_2d_scene(1024, false);
            g.update = Some(drift_lights_2d);
            Ok(())
        },
        "lights-2d-64-4096" => |g, _| {
            g.scene = lights_2d_scene(4096, false);
            g.update = Some(drift_lights_2d);
            Ok(())
        },
        "lights-2d-64-normal" => |g, _| {
            g.scene = lights_2d_scene(1024, true);
            g.update = Some(drift_lights_2d);
            Ok(())
        },
        // The same map zoomed out until every chunk of every layer is in view
        // (the worst case: about 170,000 quads; for the record, not a budget).
        "tilemap-256-all" => |g, _| {
            g.scene = tilemap_256();
            g.scene.camera = Camera::orthographic_2d(Vec2::new(128.0, 128.0), 260.0);
            Ok(())
        },
        // Phase 3: 40,000 buildings in sixteen batches from the street
        // (walking down the avenue) and from the air. Perf only (no
        // reference; tests/evidence/P3/culling/).
        "city-street" => |g, _| {
            g.scene = crate::culling_scenes::city_street();
            g.update = Some(crate::culling_scenes::walk);
            Ok(())
        },
        "city-aerial" => |g, _| {
            g.scene = crate::culling_scenes::city_aerial();
            Ok(())
        },
        // Only the sky, the sun in view.
        "sky-only" => |g, _| {
            g.scene = sky_only();
            Ok(())
        },
        // Everything at once: models, CSM, local lights, bloom, auto exposure,
        // world-space text and sprites, the default antialiasing (TAA).
        "showcase" => |g, _| {
            g.scene = showcase()?;
            crate::fx_scenes::showcase_effects(&mut g.scene);
            g.desc.antialias = Antialias::Tier;
            g.frames = 16;
            g.update = Some(crate::fx_scenes::showcase_update);
            Ok(())
        },
        // The showcase under a screen-space HUD in the overlay: after TAA,
        // bloom, auto exposure and ACES, never tone mapped or blurred.
        "overlay" => |g, _| {
            g.scene = showcase()?;
            crate::fx_scenes::showcase_effects(&mut g.scene);
            crate::fx_scenes::overlay_hud(&mut g.scene)?;
            g.desc.antialias = Antialias::Tier;
            g.frames = 16;
            g.update = Some(crate::fx_scenes::showcase_update);
            Ok(())
        },
        // Night camp: GPU particles after 120 ticks.
        "particles" => |g, _| {
            g.scene = fx::particles_scene();
            g.frames = 120;
            g.update = Some(fx::tick_update);
            Ok(())
        },
        // The same simulated on the CPU path (no collision).
        "particles-cpu" => |g, _| {
            g.scene = fx::particles_scene();
            g.desc.particles = og_render::ParticleSimulation::Cpu;
            g.frames = 120;
            g.update = Some(fx::tick_update);
            Ok(())
        },
        // R7: lit smoke, lit mesh debris with collision dust (sub-emitters,
        // so the CPU path) and a heat haze bending a checker wall.
        "particles-lit" => |g, _| {
            g.scene = fx::particles_lit_scene();
            g.frames = 90;
            g.update = Some(fx::tick_update);
            Ok(())
        },
        // 2D: lights with soft shadows from crates, glow, 2D particles, flash.
        "fx-2d" => |g, _| {
            g.scene = fx::fx_2d_scene()?;
            g.frames = 60;
            g.update = Some(fx::tick_update);
            Ok(())
        },
        "trails-beams" => |g, _| {
            g.scene = fx::trails_scene();
            g.frames = 50;
            g.update = Some(fx::move_wisps);
            Ok(())
        },
        "decals" => |g, _| {
            g.scene = fx::decals_scene();
            Ok(())
        },
        // 642 normal-mapped bullet holes on a wall and the ground (R7).
        "decals-many" => |g, _| {
            g.scene = fx::decals_many_scene();
            Ok(())
        },
        // The spike's scene through a teal-orange LUT, vignette, fringing and grain.
        "post-grade" => |g, _| {
            g.scene = spike_gltf_scene()?;
            fx::grade(&mut g.scene);
            Ok(())
        },
        // The bloom scene's saturated emissive spheres through AgX instead
        // of ACES: the brightest cores go to white instead of skewing hue.
        "post-agx" => |g, _| {
            g.scene = emissive_spheres();
            g.scene.post.bloom = Some(Bloom {
                threshold: 1.0,
                knee: 0.5,
                intensity: 0.8,
            });
            g.scene.post.tonemap = Tonemap::Agx;
            Ok(())
        },
        // Spheres from mirror to rough, chrome and white, lit only by a
        // cooked HDR environment map (the synthetic studio), turned, as
        // the background too.
        "environment-map" => |g, _| {
            g.scene = crate::ibl_scenes::environment_scene()?;
            Ok(())
        },
        // The environment-map scene with its background blurred half way
        // (backgroundBlurriness 0.5): the spheres exactly as there, the
        // studio behind them soft.
        "environment-blurry" => |g, _| {
            g.scene = crate::ibl_scenes::blurry_scene()?;
            Ok(())
        },
        // The environment-map scene half way through a blend from the
        // studio to a sunny outdoor map: light and background both mixed.
        "environment-blend" => |g, _| {
            g.scene = crate::ibl_scenes::blend_scene(0.5)?;
            Ok(())
        },
        // Spheres standing on the outdoor map's checked ground, projected
        // onto a grounded dome (three.js GroundedSkybox).
        "environment-grounded" => |g, _| {
            g.scene = crate::ibl_scenes::grounded_scene(true)?;
            Ok(())
        },
        // The environment-map scene lit by the outdoor map's image loaded
        // uncooked and prefiltered on the GPU (og.load.environmentData).
        "environment-uncooked" => |g, _| {
            g.scene = crate::ibl_scenes::uncooked_scene(false)?;
            Ok(())
        },
        "ao-gtao" => |g, _| {
            g.scene = fx::ao_scene(og_render::AoMethod::Gtao);
            Ok(())
        },
        "ao-ssao" => |g, _| {
            g.scene = fx::ao_scene(og_render::AoMethod::Ssao);
            Ok(())
        },
        // Screen-space GI (og-render ssgi.rs) without TAA: each frame reads
        // the last one's lit image, one more bounce a frame, so the capture
        // waits four.
        "gi-room" => |g, _| {
            g.scene = fx::gi_scene(true);
            g.frames = 4;
            Ok(())
        },
        // The same under TAA: the rays turn every frame and TAA's history
        // averages them (and is the image the GI reads).
        "gi-room-taa" => |g, _| {
            g.scene = fx::gi_scene(true);
            g.desc.antialias = Antialias::Taa;
            g.frames = 32;
            Ok(())
        },
        // The probe GI (gi_probes.rs): a bounce more each frame, the
        // probes' estimators settled by the capture.
        "gi-room-probes" => |g, _| {
            g.scene = fx::gi_probe_scene(true);
            g.frames = 32;
            Ok(())
        },
        "gi-shelter" => |g, _| {
            g.scene = fx::gi_shelter_scene(true, true);
            g.frames = 32;
            Ok(())
        },
        // A realtime light volume swept about five times (8 of its 75
        // probes a frame): the lamp's bounce in the shade, light bouncing
        // once more with each sweep.
        "light-volume-room" => |g, _| {
            g.scene = fx::light_volume_scene(true, Vec3::new(1.0, 0.9, 0.78), None);
            g.frames = 48;
            Ok(())
        },
        // The same with a chrome sphere in the middle, its reflection
        // relit by the probes.
        "light-volume-mirror" => |g, _| {
            g.scene = fx::light_volume_scene(true, Vec3::new(1.0, 0.9, 0.78), Some(0.02));
            g.frames = 48;
            // The clock runs, as in a game: the realtime probe captures
            // every 8 ticks, so it sees the light volume once it is lit
            // (at tick 0 alone it keeps its first, unlit capture).
            g.update = Some(fx::tick_update);
            Ok(())
        },
        // The same sphere rough (0.4 and 0.7): the probe's capture blurred
        // with its level, then blending into the probes' soft light, with
        // no step between them.
        "light-volume-glossy" => |g, _| {
            g.scene = fx::light_volume_scene(true, Vec3::new(1.0, 0.9, 0.78), Some(0.4));
            g.frames = 48;
            // The clock runs, as in a game: the realtime probe captures
            // every 8 ticks, so it sees the light volume once it is lit
            // (at tick 0 alone it keeps its first, unlit capture).
            g.update = Some(fx::tick_update);
            Ok(())
        },
        "light-volume-rough" => |g, _| {
            g.scene = fx::light_volume_scene(true, Vec3::new(1.0, 0.9, 0.78), Some(0.7));
            g.frames = 48;
            // The clock runs, as in a game: the realtime probe captures
            // every 8 ticks, so it sees the light volume once it is lit
            // (at tick 0 alone it keeps its first, unlit capture).
            g.update = Some(fx::tick_update);
            Ok(())
        },
        "dof-low" => |g, _| {
            g.scene = fx::dof_scene(og_render::DofQuality::Low)?;
            Ok(())
        },
        "dof-high" => |g, _| {
            g.scene = fx::dof_scene(og_render::DofQuality::High)?;
            Ok(())
        },
        "god-rays" => |g, _| {
            g.scene = fx::god_rays_scene();
            Ok(())
        },
        // Lens flare: a sun almost overhead seen past the roofs, a low sun
        // half behind a wall's edge, and lamps at night (one half behind a
        // pillar, a spot aimed at the camera).
        "flare-noon" => |g, _| {
            g.scene = fx::flare_noon_scene();
            Ok(())
        },
        "flare-edge" => |g, _| {
            g.scene = fx::flare_edge_scene(0.3);
            Ok(())
        },
        "flare-lights" => |g, _| {
            g.scene = fx::flare_lights_scene();
            Ok(())
        },
        "motion-blur" => |g, _| {
            g.scene = fx::motion_blur_scene();
            g.frames = 4;
            g.update = Some(fx::move_blur);
            Ok(())
        },
        "stylised" => |g, _| {
            g.scene = fx::stylised_scene()?;
            Ok(())
        },
        "crowd-culling" => |g, _| {
            g.scene = fx::crowd_culling_scene();
            g.frames = 3;
            Ok(())
        },
        "skinning" => |g, _| {
            g.scene = fx::skinning_scene();
            Ok(())
        },
        // kestrel-stair #11: per-instance flash and tint, a glTF model's
        // instances sharing its materials (`InstanceFx`).
        "model-flash" => |g, _| {
            g.scene = fx::model_flash_scene()?;
            Ok(())
        },
        // E5: fog over the colonnade by day, the day-night look at dusk
        // and at night (stars, moonlight), planar water.
        "atmosphere-fog" => |g, _| {
            g.scene = crate::atmo_scenes::fog_scene();
            Ok(())
        },
        "atmosphere-dusk" => |g, _| {
            g.scene = crate::atmo_scenes::day_night_scene(18.2, true);
            Ok(())
        },
        "atmosphere-night" => |g, _| {
            g.scene = crate::atmo_scenes::day_night_scene(23.0, false);
            Ok(())
        },
        "water-3d" => |g, _| {
            g.scene = crate::atmo_scenes::water_scene();
            Ok(())
        },
        "water-2d" => |g, _| {
            g.scene = crate::atmo_scenes::water_2d_scene();
            Ok(())
        },
        "fluid-2d" => |g, _| {
            g.scene = crate::atmo_scenes::fluid_2d_scene();
            Ok(())
        },
        // Phase 2: ten fluids a frame, two groups (blended, layered).
        "fluid-groups" => |g, _| {
            g.scene = crate::atmo_scenes::fluid_groups_scene();
            Ok(())
        },
        // Phase 2: 3D particle fluids as a screen-space surface (row 29).
        "fluid-3d" => |g, _| {
            g.scene = crate::p2_scenes::fluid_3d_scene();
            Ok(())
        },
        // Material shader hooks: dissolve by noise, wind-swayed grass.
        "shader-hooks" => |g, _| {
            g.scene = crate::atmo_scenes::hooks_scene(45);
            Ok(())
        },
        // Phase 2: 256 point and spot lights over a night plaza (Forward+).
        "lights-256" => |g, _| {
            g.scene = crate::p2_scenes::many_lights(256);
            Ok(())
        },
        // Phase 2: the night colonnade with a waxing gibbous moon in view.
        "atmosphere-moon" => |g, _| {
            g.scene = crate::p2_scenes::moon_scene(0.35, 2.4);
            Ok(())
        },
        // Phase 2: every M6 material feature (metalness, roughness,
        // emissive, normal maps, clearcoat, alpha mask and blend,
        // double-sided, unlit, a non-uniformly scaled ellipsoid).
        "materials" => |g, _| {
            g.scene = crate::p2_scenes::materials_scene();
            Ok(())
        },
        // Every MeshPhysicalMaterial lobe (clearcoat maps, sheen,
        // iridescence, anisotropy) and transmission in front of textured
        // geometry.
        "materials-physical" => |g, _| {
            g.scene = crate::physical_scenes::physical_scene();
            Ok(())
        },
        // The depth wave's geometry (docs/depth.md §1, geometry_scenes.rs):
        // the three.js primitives, a tube and a buffer-geometry height
        // field; lines of every kind; point clouds; wireframe materials;
        // debug draw over a bloomed frame.
        "geometry-procedural" => |g, _| {
            g.scene = crate::geometry_scenes::procedural_scene();
            Ok(())
        },
        "lines" => |g, _| {
            g.scene = crate::geometry_scenes::lines_scene();
            Ok(())
        },
        "points" => |g, _| {
            g.scene = crate::geometry_scenes::points_scene();
            Ok(())
        },
        "wireframe" => |g, _| {
            g.scene = crate::geometry_scenes::wireframe_scene();
            Ok(())
        },
        "debug-overlay" => |g, _| {
            g.scene = crate::geometry_scenes::debug_scene()?;
            Ok(())
        },
        // Round 3 (geom-1): og-geom's generators: an extruded, bevelled
        // gear with holes, a lathed vase, a swept horn, a CSG dice.
        "geometry-shapes" => |g, _| {
            g.scene = crate::geometry_scenes::shapes_scene();
            Ok(())
        },
        // Render targets and secondary cameras (docs/depth.md §8): a
        // monitor showing a second camera's view (with an object on a layer
        // only it sees), a mirror reflecting the room, split-screen.
        "views-monitor" => |g, _| {
            g.scene = crate::target_scenes::monitor_scene();
            Ok(())
        },
        "views-mirror" => |g, _| {
            g.scene = crate::target_scenes::mirror_scene();
            Ok(())
        },
        "views-split" => |g, _| {
            g.scene = crate::target_scenes::split_scene();
            Ok(())
        },
        // The monitor's camera drawing what the main pass draws over its
        // opaque scene: glass, a world sprite, particles and lines, each on
        // a layer the main camera leaves out.
        "views-layers" => |g, _| {
            use crate::target_scenes::{ViewKind, view_kinds_scene};
            g.scene = view_kinds_scene(&ViewKind::ALL, 3, 60);
            Ok(())
        },
        // A render target's depth texture shown full frame (from the view's
        // depth prepass).
        "views-depth" => |g, _| {
            g.scene = crate::target_scenes::depth_scene();
            Ok(())
        },
        // A full custom shader (`mode full`): a dissolve cut through a
        // sphere and a box, their shadows cut the same way.
        "shader-full-dissolve" => |g, _| {
            g.scene = crate::target_scenes::full_dissolve_scene(0.45);
            Ok(())
        },
        // A custom post pass (`type post`): the room through a CRT curve.
        "post-crt" => |g, _| {
            g.scene = crate::target_scenes::room();
            let crt = g.scene.add_post(og_render::example_crt());
            g.scene.custom_post.push(crt);
            Ok(())
        },
        // The depth wave's light (docs/depth.md §7, light_scenes.rs): rect
        // lights (LTC), light cookies, volumetric fog, and natively a
        // Cornell box baked by og-bake with a light volume.
        "rect-lights" => |g, _| {
            g.scene = crate::light_scenes::rect_lights_scene();
            Ok(())
        },
        "light-cookies" => |g, _| {
            g.scene = crate::light_scenes::light_cookies_scene();
            Ok(())
        },
        "volumetric-fog" => |g, _| {
            g.scene = crate::light_scenes::volumetric_fog_scene();
            Ok(())
        },
        #[cfg(not(target_arch = "wasm32"))]
        "baked-cornell" => |g, _| {
            g.scene = crate::light_scenes::baked_cornell_scene(true)?;
            Ok(())
        },
        #[cfg(not(target_arch = "wasm32"))]
        "baked-batches" => |g, _| {
            g.scene = crate::light_scenes::baked_batches_scene(true)?;
            Ok(())
        },
        // Materials sampling texture arrays: eight cubes of one tile
        // material (colour and normal arrays), each on its own layer or the
        // material's, one batch; a cube of layer 1 as plain 2D textures;
        // two blended panes of one array material; a sphere with only a
        // normal array.
        "materials-array" => |g, _| {
            g.scene = crate::physical_scenes::array_scene();
            Ok(())
        },
        // The stylisation primitives (stylise_scenes.rs), each over a scene
        // the references know.
        "stylise-pixel" => |g, _| {
            g.scene = crate::stylise_scenes::pixel_scene()?;
            Ok(())
        },
        "stylise-palette" => |g, _| {
            g.scene = crate::stylise_scenes::palette_scene()?;
            Ok(())
        },
        "stylise-print" => |g, _| {
            g.scene = crate::stylise_scenes::print_scene();
            Ok(())
        },
        "stylise-halftone" => |g, _| {
            g.scene = crate::stylise_scenes::halftone_scene()?;
            Ok(())
        },
        "stylise-paint" => |g, _| {
            g.scene = crate::stylise_scenes::paint_scene();
            Ok(())
        },
        "stylise-bands" => |g, _| {
            g.scene = crate::stylise_scenes::bands_scene()?;
            Ok(())
        },
        "stylise-retro" => |g, _| {
            g.scene = crate::stylise_scenes::retro_scene();
            Ok(())
        },
        "stylise-material" => |g, _| {
            g.scene = crate::stylise_scenes::material_scene();
            Ok(())
        },
        "stylise-ink-depth" => |g, _| {
            g.scene = crate::stylise_scenes::ink_depth_scene();
            Ok(())
        },
        "stylise-surface" => |g, _| {
            g.scene = crate::stylise_scenes::surface_scene();
            Ok(())
        },
        "stylise-object" => |g, _| {
            g.scene = crate::stylise_scenes::object_scene();
            Ok(())
        },
        "stylise-halftone-surface" => |g, _| {
            g.scene = crate::stylise_scenes::halftone_surface_scene()?;
            Ok(())
        },
        "stylise-outline-voxels" => |g, _| {
            g.scene = crate::stylise_scenes::outline_voxels_scene();
            Ok(())
        },
        // A 2D rig posed by og-anim on three slopes (IK feet, a deformed
        // mesh cape), drawn as sprite-batch triangles under 2D lighting.
        #[cfg(not(target_arch = "wasm32"))]
        "skeleton-2d" => |g, _| {
            g.scene = crate::skeleton_scenes::skeleton_scene(true)?;
            Ok(())
        },
        // Render-lod (depth S6; depth_render_scenes.rs): a cooked tree at
        // three distances, each at the level its size on screen picks and
        // tinted as debugView lod tints it; a chrome sphere in a closed
        // room reflecting its reflection probe; SSR on a wet floor.
        #[cfg(not(target_arch = "wasm32"))]
        "probe-interior" => |g, _| {
            g.scene = crate::depth_render_scenes::probe_scene(true);
            Ok(())
        },
        "ssr-wet-floor" => |g, _| {
            g.scene = crate::depth_render_scenes::ssr_scene(true);
            g.frames = crate::depth_render_scenes::SSR_FRAMES;
            Ok(())
        },
        // Reflection probes picked per instance (two rooms sharing a
        // sphere mesh) and capturing skinned meshes, terrain, grass and a
        // translucent pane outdoors (probe_scenes.rs).
        #[cfg(not(target_arch = "wasm32"))]
        "probe-two-rooms" => |g, _| {
            g.scene = crate::probe_scenes::two_rooms_scene(true);
            Ok(())
        },
        #[cfg(not(target_arch = "wasm32"))]
        "probe-outdoor" => |g, _| {
            g.scene = crate::probe_scenes::outdoor_scene(crate::probe_scenes::Outdoor {
                content: true,
                probe: Some(1.0),
            });
            Ok(())
        },
        "stylise-toon-physical" => |g, _| {
            g.scene = crate::stylise_scenes::toon_physical_scene();
            Ok(())
        },
        "stylise-bands-tint" => |g, _| {
            g.scene = crate::stylise_scenes::bands_tint_scene();
            Ok(())
        },
        // Untonemapped sprites under TAA (its history carries the alpha).
        // A dark cave of tile layers lit by two Light2d with occluders,
        // a window's sky layer opting out (hollowbell FINDINGS #9).
        "tilemap-lit" => |g, _| {
            g.scene = fx::tilemap_lit_scene();
            Ok(())
        },
        "sprites-untonemapped" => |g, _| {
            g.scene = crate::stylise_scenes::untonemapped_sprites_scene();
            g.desc.antialias = Antialias::Tier;
            g.frames = 16;
            Ok(())
        },
        // Phase 2: skinned meshes drawn with a material shader that ripples,
        // bands and lifts them, their shadows and prepass displaced too.
        "skinned-hooks" => |g, _| {
            g.scene = crate::p2_scenes::skinned_hooks_scene();
            Ok(())
        },
        // Weather presets over the colonnade: particles, fog, dimmed light.
        // R4: the physical sky at noon, sunset, twilight and in haze.
        "sky-physical-noon"
        | "sky-physical-sunset"
        | "sky-physical-twilight"
        | "sky-physical-hazy"
        | "clouds-fair"
        | "clouds-overcast"
        | "clouds-sunset"
        | "clouds-gradient"
        | "weather-wet"
        | "weather-snow-cover"
        | "weather-lightning"
        | "weather-shelter-snow"
        | "weather-shelter-rain"
        | "weather-shelter-meshlet"
        | "aurora-arcs"
        | "aurora-breakup"
        | "live-sky" => |g, name| {
            g.scene = crate::sky_scenes::sky_scene(name);
            Ok(())
        },
        // R5: the sea under the physical sky, and a stream's flow map.
        "water-ocean"
        | "water-ocean-storm"
        | "water-ocean-sunset"
        | "water-flow-map"
        | "water-ocean-dusk-storm"
        | "water-ocean-night"
        | "water-shore"
        | "water-shore-storm"
        | "water-shallows"
        | "water-river" => |g, name| {
            g.scene = crate::ocean_scenes::ocean_scene(name);
            Ok(())
        },
        "water-wake" => |g, name| {
            g.scene = crate::ocean_scenes::ocean_scene(name);
            g.frames = crate::ocean_scenes::WAKE_FRAMES;
            g.update = Some(crate::ocean_scenes::wake_update);
            Ok(())
        },
        // Wave B: weather on surfaces (a rain-wet street, puddles drying,
        // snow built up, a dusty desert, rain on cobbles) and velvet and
        // cotton.
        // The wet street and the puddles mirror through SSR, whose hits
        // read the frame before: they draw its 16.
        "weather-street-wet" | "weather-puddles" => |g, name| {
            g.scene = crate::weather_scenes::weather_scene(name);
            g.frames = crate::SSR_FRAMES;
            Ok(())
        },
        // Rain on T4's cobbles (a terrain).
        "weather-snow-buildup"
        | "weather-dust-desert"
        | "materials-cloth"
        | "weather-cobbles-wet" => |g, name| {
            g.scene = crate::weather_scenes::weather_scene(name);
            Ok(())
        },
        "weather-rain" | "weather-snow" => |g, name| {
            g.scene = crate::atmo_scenes::weather_scene(if name == "weather-rain" {
                og_render::WEATHER_RAIN
            } else {
                og_render::WEATHER_SNOW
            });
            g.frames = crate::atmo_scenes::WEATHER_FRAMES;
            g.update = Some(fx::tick_update);
            Ok(())
        },
        // Phase 2: the 2 km island from three distances, and the lake's
        // planar reflection, shadows and underwater view.
        "terrain-near" => |g, _| {
            g.scene = crate::terrain_scenes::island_view("near");
            Ok(())
        },
        "terrain-mid" => |g, _| {
            g.scene = crate::terrain_scenes::island_view("mid");
            Ok(())
        },
        "terrain-far" => |g, _| {
            g.scene = crate::terrain_scenes::island_view("far");
            Ok(())
        },
        "water-planar" => |g, _| {
            g.scene = crate::terrain_scenes::water_planar_scene();
            Ok(())
        },
        "water-shadows" => |g, _| {
            g.scene = crate::terrain_scenes::water_shadows_scene();
            Ok(())
        },
        "water-underwater" => |g, _| {
            g.scene = crate::terrain_scenes::water_underwater_scene();
            Ok(())
        },
        "terrain-grass" => |g, _| {
            g.scene = crate::terrain_scenes::grass_scene();
            Ok(())
        },
        // The rendering fields of the Grand View Garden fork merge
        // (fork_scenes.rs, tests/fork_fields.rs).
        "water-mesh" => |g, _| {
            g.scene = crate::fork_scenes::water_mesh_scene(true);
            Ok(())
        },
        "water-ripples" => |g, _| {
            g.scene = crate::fork_scenes::water_ripples_scene();
            Ok(())
        },
        "water-fresnel" => |g, _| {
            g.scene = crate::fork_scenes::water_fresnel_scene();
            Ok(())
        },
        "water-glint" => |g, _| {
            g.scene = crate::fork_scenes::water_glint_scene();
            Ok(())
        },
        "shadow-box" => |g, _| {
            g.scene = crate::fork_scenes::shadow_box_scene();
            Ok(())
        },
        "hemisphere-light" => |g, _| {
            g.scene = crate::fork_scenes::hemisphere_scene();
            Ok(())
        },
        "ao-direct" => |g, _| {
            g.scene = crate::fork_scenes::ao_direct_scene();
            Ok(())
        },
        "ao-screen-water" => |g, _| {
            g.scene = crate::fork_scenes::ao_screen_water_scene(true);
            Ok(())
        },
        "post-sharpen" => |g, _| {
            g.scene = crate::fork_scenes::sharpen_scene();
            Ok(())
        },
        "bloom-unreal" => |g, _| {
            g.scene = crate::fork_scenes::bloom_unreal(build("bloom")?.scene);
            Ok(())
        },
        "alpha-to-coverage" => |g, _| {
            g.scene = crate::fork_scenes::alpha_to_coverage_scene(true);
            g.desc.antialias = Antialias::Msaa4;
            Ok(())
        },
        "hooks-eye-front" => |g, _| {
            g.scene = crate::fork_scenes::hooks_eye_front_scene();
            Ok(())
        },
        // Material shaders read the planar reflection, and a second surface
        // gets one, only with the second slot (RenderOptions.planarReflections 2,
        // og_render::planar_slot).
        "hooks-planar" => |g, _| {
            g.scene = crate::fork_scenes::hooks_planar_scene(true);
            g.desc.planar_reflections = 2;
            Ok(())
        },
        "hooks-screen-ao" => |g, _| {
            g.scene = crate::fork_scenes::hooks_screen_ao_scene();
            Ok(())
        },
        "water-planar-two" => |g, _| {
            g.scene = crate::fork_scenes::water_planar_two_scene();
            g.desc.planar_reflections = 2;
            Ok(())
        },
        "terrain-hidden" => |g, _| {
            g.scene = crate::fork_scenes::terrain_flags_scene(false, true);
            Ok(())
        },
        "terrain-scatter-unshadowed" => |g, _| {
            g.scene = crate::fork_scenes::terrain_flags_scene(true, false);
            Ok(())
        },
        // The spike's scene through the teal-orange LUT at 0.6, then the
        // colour adjustments baked with it: warmer, more contrast and
        // vibrance, teal shadows and warm highlights.
        "post-adjust" => |g, _| {
            g.scene = spike_gltf_scene()?;
            fx::adjust(&mut g.scene);
            Ok(())
        },
        // The bloom scene's saturated emissive spheres through Khronos PBR
        // Neutral: the bodies keep their authored colour, which ACES darkens
        // and shifts, and the brightest cores roll to white.
        "post-neutral" => |g, _| {
            g.scene = emissive_spheres();
            g.scene.post.bloom = Some(Bloom {
                threshold: 1.0,
                knee: 0.5,
                intensity: 0.8,
            });
            g.scene.post.tonemap = Tonemap::Neutral;
            Ok(())
        },
        // The spike's scene through a centred barrel lens (0.35): the
        // checker floor's lines bow outwards, the frame fitted to its
        // corners.
        "post-lens" => |g, _| {
            g.scene = spike_gltf_scene()?;
            g.scene.post.lens_distortion = Some(og_render::LensDistortion {
                amount: 0.35,
                ..Default::default()
            });
            Ok(())
        },
        // The grass scene in a gusting wind at tick 90 (strength 0.4 across
        // the meadow, gust 0.7): the tufts lean downwind, further where a
        // gust passes.
        "terrain-grass-wind" => |g, _| {
            g.scene = crate::terrain_scenes::grass_scene();
            g.scene.wind = Some(og_render::Wind {
                strength: 0.4,
                direction: Vec3::new(1.0, 0.0, 0.4),
                speed: 3.0,
                gust: 0.7,
                ..og_render::Wind::default()
            });
            g.scene.tick = 90;
            Ok(())
        },
        // R6: a 16 × 12 m pit cut out of the island's meadow over an
        // orange cave floor, the sun nearly overhead: the floor and the
        // sunlight falling through the hole show through it, the meadow
        // round it unbroken.
        "terrain-holes" => |g, _| {
            g.scene = crate::terrain_scenes::holes_scene();
            Ok(())
        },
        // R6: the island's meadow from 70 m above with eight layers
        // (texture arrays): four discs on it painted with layers 4 to 7 on the
        // control map's second page (clay, heather, mud, gold), 28 m
        // across and well apart, each blended into the grass by the terrain's
        // height blend (a crisp rim) as the first four are.
        "terrain-layers" => |g, _| {
            g.scene = crate::terrain_scenes::layers_scene(true);
            Ok(())
        },
        // R7: a flat terrain of one high-contrast layer (dark blobs
        // repeating every 4 m) seen straight down, its repeat broken by
        // hex-tiling (antiTiling 1): no 4 m period shows.
        "terrain-tiling" => |g, _| {
            g.scene = crate::terrain_scenes::tiling_scene(1.0);
            Ok(())
        },
        // R7: an 85° wall 60 m high, a checker of 4 m cells over it
        // sampled past 45° by the two projections facing it (projectSlope
        // 45): square cells on the face, not vertical streaks.
        "terrain-steep" => |g, _| {
            g.scene = crate::terrain_scenes::steep_scene(45.0);
            Ok(())
        },
        // T5: a 300 m ridge with the sun 8° up behind it, its shade on the
        // plain in front far past the cascades, drawn from the horizon map.
        "terrain-horizon" => |g, _| {
            g.scene = crate::terrain_scenes::horizon_scene(true);
            Ok(())
        },
        // T4: a crest of dressed setts 10 cm proud of their joints (one layer's
        // albedo, normal map and displacement) seen from knee height 4 m
        // before it under a low side sun: the
        // skyline and the ground near it carry the setts' relief and
        // their shadows.
        "terrain-displaced" => |g, _| {
            g.scene =
                crate::terrain_scenes::displaced_scene(crate::terrain_scenes::DISPLACED_RELIEF);
            Ok(())
        },
        // T3: a 300 m cliff the height field cannot make, drawn by a block
        // of rock stamped over the step (og.TerrainStamp), its ground
        // carved under it: grass on its top, rock on its face, its foot
        // meeting the valley's grass with a hard edge (blend 0).
        "terrain-cliff" => |g, _| {
            g.scene = crate::terrain_scenes::cliff_scene(0.0);
            Ok(())
        },
        // T3: the same cliff with blend 1: the face's lowest metre takes the
        // ground's grass and normal, so it meets the valley with no seam.
        "terrain-cliff-blend" => |g, _| {
            g.scene = crate::terrain_scenes::cliff_scene(1.0);
            Ok(())
        },
        // R6: a ball red on its +x half and blue on its -x half, 2 m
        // across on a grey floor, drawn as its mesh (left) and as its
        // octahedral impostor (right): the two read alike, red towards the
        // camera's right, blue its left, each casting a round shadow.
        "impostors" => |g, _| {
            g.scene = crate::impostor_scenes::impostors_scene();
            Ok(())
        },
        // R6: the ball scattered over a 48 m green terrain, its copies
        // farther than 15 m drawn as its impostor (cross-dithered over the
        // last 1.5 m): the far balls look as the near ones do.
        "impostor-scatter" => |g, _| {
            g.scene = crate::impostor_scenes::impostor_scatter_scene(15.0);
            Ok(())
        },
        // R6: two 6 m columns on a grey floor, the red one (left) leaning
        // downwind (+x, to the right) under og.Wind's hook, its top about
        // 0.9 m over, the blue one (right) upright; the red column's
        // shadow leans with it.
        "wind-models" => |g, _| {
            g.scene = crate::wind_scenes::wind_scene(true, 90);
            Ok(())
        },
        // R6's grass: a default og.Grass over the island's meadow, casting
        // into the first cascade. Near, at eye height: curved blades in
        // clumps, dark at their roots, bright and some dry at their tips,
        // the boulders' and the blades' own shadows on them.
        "grass-near" => |g, _| {
            g.scene = crate::grass_scenes::grass_scene("near");
            Ok(())
        },
        // 12 m up across 100 m of meadow: thinner, wider blades past 30 m,
        // the ground between them grass-coloured, no ring where the
        // levels of detail change.
        "grass-mid" => |g, _| {
            g.scene = crate::grass_scenes::grass_scene("mid");
            Ok(())
        },
        // 40 m up towards the mountains: the meadow reads as grass to the
        // slopes with no seam where its blades end (150 m), only rock and
        // snow above.
        "grass-far" => |g, _| {
            g.scene = crate::grass_scenes::grass_scene("far");
            Ok(())
        },
        // At eye height into a low sun: the blades glow yellow-green where
        // it shines through them, their edges brightest.
        "grass-backlit" => |g, _| {
            g.scene = crate::grass_scenes::grass_scene("backlit");
            Ok(())
        },
        // The near view in a gusting wind (+x, tick 90): every blade bent
        // downwind, further in the gust patches.
        "grass-wind" => |g, _| {
            g.scene = crate::grass_scenes::grass_wind_scene(90);
            Ok(())
        },
        // The bloom scene through a zoom blur towards a point left of and
        // above the centre (strength 0.15, 16 taps): the spheres streak
        // along the lines to it, and bloom, after the blur, glows round the
        // streaks.
        "post-blur" => |g, _| {
            g.scene = emissive_spheres();
            g.scene.post.bloom = Some(Bloom {
                threshold: 1.0,
                knee: 0.5,
                intensity: 0.8,
            });
            g.scene.post.screen_blur = Some(og_render::ScreenBlur {
                kind: og_render::ScreenBlurKind::Zoom,
                strength: 0.15,
                center: Vec2::new(0.42, 0.4),
                ..Default::default()
            });
            Ok(())
        },
        // The UI pass (docs/ui.md U1, src/ui_scenes.rs): its shapes, paints,
        // effects, layers and materials, text effects, and a cooked vector.
        "ui-shapes" => |g, _| {
            g.scene = crate::ui_scenes::shapes();
            Ok(())
        },
        "ui-paints" => |g, _| {
            g.scene = crate::ui_scenes::paints();
            Ok(())
        },
        "ui-effects" => |g, _| {
            g.scene = crate::ui_scenes::effects();
            Ok(())
        },
        "ui-layers" => |g, _| {
            g.scene = crate::ui_scenes::layers();
            Ok(())
        },
        // Flat glass (docs/ui.md §5.3, src/ui_scenes/glass.rs): its weights,
        // hairline, shadows, scrim and fallbacks on three backdrops.
        "ui-glass-bright" | "ui-glass-dark" | "ui-glass-busy" | "ui-glass-dialog"
        | "ui-glass-fallback" => |g, name| {
            let which = crate::ui_scenes::GlassScene::ALL
                .into_iter()
                .find(|s| s.name() == name)
                .expect("a glass scene");
            g.scene = crate::ui_scenes::glass_scene(which, load_font, true)?.0;
            Ok(())
        },
        "ui-text-effects" => |g, _| {
            g.scene = crate::ui_scenes::text_effects(load_font)?;
            Ok(())
        },
        // World-space UI (docs/ui.md §14, src/ui_scenes.rs): mass elements
        // (numbers, nameplates, bars, icons) and occluders over a 3D scene.
        "ui-world" => |g, _| {
            g.scene = crate::ui_scenes::world(load_font, |s| sprite_textures(s)[1])?;
            Ok(())
        },
        // A world panel (docs/ui.md §14): a UI layer drawn into a render
        // target on a quad, read the frame after.
        "ui-panel" => |g, _| {
            g.scene = crate::ui_scenes::panel();
            g.frames = 2;
            Ok(())
        },
        other => {
            return Err(format!(
                "unknown gfx scene '{other}' (known: {})",
                SCENES.join(", ")
            ));
        }
    };
    scene(&mut g, name)?;
    Ok(g)
}
