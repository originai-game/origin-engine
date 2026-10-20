//! The named gfx scenes (the M3 reference set, `docs/phase-1.md`). Each one
//! is a fixed [`RenderDesc`] and [`RenderScene`], how many frames to render
//! before the capture (with an optional per-frame update), the backend tiers
//! it applies to, and the tolerance its reference comparison uses.

use crate::compare::Tolerance;
use crate::fx_scenes as fx;
use glam::{Mat4, Quat, Vec2, Vec3};
use og_render::gltf::{GltfModel, load_glb};
use og_render::{
    Antialias, Background, Bloom, Camera, ColorSpace, DirectionalLight, Environment, Exposure,
    Glyph, Light, LightKind, Material, Mips, MsdfFont, ProceduralSky, Projection, RenderDesc,
    RenderScene, ShadowBounds, Sprite, Text, TextureData, Tier, Tonemap, shapes,
};
use std::path::PathBuf;

/// Every scene the harness knows, in the order `og-gfx --all` renders them.
pub const SCENES: &[&str] = &[
    "gltf-reference",
    "spheres-20k",
    "cascades-200m",
    "point-spot-lights",
    "taa",
    "msaa",
    "bloom",
    "exposure",
    "sprites-20k",
    "text-msdf",
    "sky-only",
    "showcase",
    "overlay",
    "particles",
    "particles-cpu",
    "fx-2d",
    "trails-beams",
    "decals",
    "post-grade",
    "post-agx",
    "environment-map",
    "ao-gtao",
    "ao-ssao",
    "dof-low",
    "dof-high",
    "god-rays",
    "motion-blur",
    "stylised",
    "skinning",
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
];

/// Which backend tiers a scene has a reference for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Applies {
    Every,
    /// TAA is the full tier's antialiasing.
    FullTier,
    /// MSAA is the no-compute (WebGL2/GL) tier's antialiasing.
    NoComputeTier,
}

impl Applies {
    pub fn includes(self, tier: Tier) -> bool {
        match self {
            Applies::Every => true,
            Applies::FullTier => tier == Tier::Full,
            Applies::NoComputeTier => tier == Tier::NoCompute,
        }
    }
}

/// A per-frame change (frame index from 0): moving sprites, an orbiting camera.
pub type Update = fn(&mut RenderScene, u32);

pub struct GfxScene {
    pub desc: RenderDesc,
    pub scene: RenderScene,
    pub tolerance: Tolerance,
    /// Frames rendered before the capture (TAA needs its history).
    pub frames: u32,
    pub update: Option<Update>,
    pub applies: Applies,
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

/// Builds the scene called `name`.
pub fn build(name: &str) -> Result<GfxScene, String> {
    let mut g = GfxScene {
        desc: base_desc(),
        scene: RenderScene::new(),
        tolerance: Tolerance::default(),
        frames: 1,
        update: None,
        applies: Applies::Every,
    };
    match name {
        // The spike's reference scene (two glTF models x 25 on a checker
        // ground), drawn with the sky pass, mipmapped textures and CSM.
        "gltf-reference" => g.scene = spike_gltf_scene()?,
        // 20,000 low-poly spheres submitted one by one: two batches in all.
        "spheres-20k" => g.scene = sphere_field(20_000),
        // The same field with every sphere bobbing each frame (the bench's
        // sphere field, tools/bench/spheres): the renderer's per-instance
        // cost when every transform changes. Perf only (no reference).
        "spheres-20k-moving" => {
            g.scene = sphere_field(20_000);
            g.update = Some(bob_spheres);
        }
        // A 400 m field with pillars from 3 m to 190 m: shadows in every cascade.
        "cascades-200m" => g.scene = cascades_200m(true),
        // Night: seven point and spot lights, the first two shadowed.
        "point-spot-lights" => g.scene = point_spot_lights(),
        // Thin geometry after 16 frames of TAA (static camera, converged).
        "taa" => {
            g.scene = thin_geometry();
            g.desc.antialias = Antialias::Tier;
            g.frames = 16;
            g.applies = Applies::FullTier;
        }
        // The same geometry with the GL tier's 4x MSAA.
        "msaa" => {
            g.scene = thin_geometry();
            g.desc.antialias = Antialias::Tier;
            g.applies = Applies::NoComputeTier;
        }
        // Emissive spheres over a dark ground with bloom.
        "bloom" => {
            g.scene = emissive_spheres();
            g.scene.post.bloom = Some(Bloom {
                threshold: 1.0,
                knee: 0.5,
                intensity: 0.8,
            });
        }
        // A dim scene brought to middle grey by metered exposure.
        "exposure" => g.scene = dim_scene(1.0),
        // 20,000 sprites, three textures (one an atlas), two layers.
        "sprites-20k" => {
            g.scene = sprite_field(20_000)?;
            g.update = Some(move_sprites);
        }
        // MSDF text at several sizes, rotated, Latin and CJK.
        "text-msdf" => g.scene = text_scene()?,
        // E4: a 256x256 tilemap with four layers (one animated, one at half
        // parallax) through og-tilemap's chunks, the camera panning 80x45
        // tiles of it (16 px tiles at 1280x720). Perf only (no reference).
        "tilemap-256" => {
            g.scene = tilemap_256();
            g.update = Some(pan_tilemap);
        }
        // The same map zoomed out until every chunk of every layer is in view
        // (the worst case: about 170,000 quads; for the record, not a budget).
        "tilemap-256-all" => {
            g.scene = tilemap_256();
            g.scene.camera = Camera::orthographic_2d(Vec2::new(128.0, 128.0), 260.0);
        }
        // Phase 3: 40,000 buildings in sixteen batches from the street
        // (walking down the avenue) and from the air. Perf only (no
        // reference; tests/evidence/P3/culling/).
        "city-street" => {
            g.scene = crate::culling_scenes::city_street();
            g.update = Some(crate::culling_scenes::walk);
        }
        "city-aerial" => g.scene = crate::culling_scenes::city_aerial(),
        // Only the sky, the sun in view.
        "sky-only" => g.scene = sky_only(),
        // Everything at once: models, CSM, local lights, bloom, auto exposure,
        // world-space text and sprites, the tier's antialiasing.
        "showcase" => {
            g.scene = showcase()?;
            crate::fx_scenes::showcase_effects(&mut g.scene);
            g.desc.antialias = Antialias::Tier;
            g.frames = 16;
            g.update = Some(crate::fx_scenes::showcase_update);
        }
        // The showcase under a screen-space HUD in the overlay: after TAA,
        // bloom, auto exposure and ACES, never tone mapped or blurred.
        "overlay" => {
            g.scene = showcase()?;
            crate::fx_scenes::showcase_effects(&mut g.scene);
            crate::fx_scenes::overlay_hud(&mut g.scene)?;
            g.desc.antialias = Antialias::Tier;
            g.frames = 16;
            g.update = Some(crate::fx_scenes::showcase_update);
        }
        // Night camp: GPU particles (CPU on the no-compute tier) after 120 ticks.
        "particles" => {
            g.scene = fx::particles_scene();
            g.frames = 120;
            g.update = Some(fx::tick_update);
        }
        // The same simulated on the CPU path on any tier (no collision).
        "particles-cpu" => {
            g.scene = fx::particles_scene();
            g.desc.particles = og_render::ParticleSimulation::Cpu;
            g.frames = 120;
            g.update = Some(fx::tick_update);
        }
        // 2D: lights with soft shadows from crates, glow, 2D particles, flash.
        "fx-2d" => {
            g.scene = fx::fx_2d_scene()?;
            g.frames = 60;
            g.update = Some(fx::tick_update);
        }
        "trails-beams" => {
            g.scene = fx::trails_scene();
            g.frames = 50;
            g.update = Some(fx::move_wisps);
        }
        "decals" => g.scene = fx::decals_scene(),
        // The spike's scene through a teal-orange LUT, vignette, fringing and grain.
        "post-grade" => {
            g.scene = spike_gltf_scene()?;
            fx::grade(&mut g.scene);
        }
        // The bloom scene's saturated emissive spheres through AgX instead
        // of ACES: the brightest cores go to white instead of skewing hue.
        "post-agx" => {
            g.scene = emissive_spheres();
            g.scene.post.bloom = Some(Bloom {
                threshold: 1.0,
                knee: 0.5,
                intensity: 0.8,
            });
            g.scene.post.tonemap = Tonemap::Agx;
        }
        // Spheres from mirror to rough, chrome and white, lit only by a
        // cooked HDR environment map (the synthetic studio), turned, as
        // the background too.
        "environment-map" => g.scene = crate::ibl_scenes::environment_scene()?,
        "ao-gtao" => g.scene = fx::ao_scene(og_render::AoMethod::Gtao),
        "ao-ssao" => g.scene = fx::ao_scene(og_render::AoMethod::Ssao),
        "dof-low" => g.scene = fx::dof_scene(og_render::DofQuality::Low)?,
        "dof-high" => g.scene = fx::dof_scene(og_render::DofQuality::High)?,
        "god-rays" => g.scene = fx::god_rays_scene(),
        "motion-blur" => {
            g.scene = fx::motion_blur_scene();
            g.frames = 4;
            g.update = Some(fx::move_blur);
        }
        "stylised" => g.scene = fx::stylised_scene()?,
        "skinning" => g.scene = fx::skinning_scene(),
        // E5: fog over the colonnade by day, the day-night look at dusk
        // and at night (stars, moonlight), planar water.
        "atmosphere-fog" => g.scene = crate::atmo_scenes::fog_scene(),
        "atmosphere-dusk" => g.scene = crate::atmo_scenes::day_night_scene(18.2, true),
        "atmosphere-night" => g.scene = crate::atmo_scenes::day_night_scene(23.0, false),
        "water-3d" => g.scene = crate::atmo_scenes::water_scene(),
        "water-2d" => g.scene = crate::atmo_scenes::water_2d_scene(),
        "fluid-2d" => g.scene = crate::atmo_scenes::fluid_2d_scene(),
        // Phase 2: ten fluids a frame, two groups (blended, layered).
        "fluid-groups" => g.scene = crate::atmo_scenes::fluid_groups_scene(),
        // Phase 2: 3D particle fluids as a screen-space surface (row 29).
        "fluid-3d" => g.scene = crate::p2_scenes::fluid_3d_scene(),
        // Material shader hooks: dissolve by noise, wind-swayed grass.
        "shader-hooks" => g.scene = crate::atmo_scenes::hooks_scene(45),
        // Phase 2: 256 point and spot lights over a night plaza (Forward+).
        "lights-256" => g.scene = crate::p2_scenes::many_lights(256),
        // Phase 2: the night colonnade with a waxing gibbous moon in view.
        "atmosphere-moon" => g.scene = crate::p2_scenes::moon_scene(0.35, 2.4),
        // Phase 2: every M6 material feature (metalness, roughness,
        // emissive, normal maps, clearcoat, alpha mask and blend,
        // double-sided, unlit, a non-uniformly scaled ellipsoid).
        "materials" => g.scene = crate::p2_scenes::materials_scene(),
        // Every MeshPhysicalMaterial lobe (clearcoat maps, sheen,
        // iridescence, anisotropy) and transmission in front of textured
        // geometry.
        "materials-physical" => g.scene = crate::physical_scenes::physical_scene(),
        // Phase 2: skinned meshes drawn with a material shader that ripples,
        // bands and lifts them, their shadows and prepass displaced too.
        "skinned-hooks" => g.scene = crate::p2_scenes::skinned_hooks_scene(),
        // Weather presets over the colonnade: particles, fog, dimmed light.
        "weather-rain" | "weather-snow" => {
            g.scene = crate::atmo_scenes::weather_scene(if name == "weather-rain" {
                og_render::WEATHER_RAIN
            } else {
                og_render::WEATHER_SNOW
            });
            g.frames = crate::atmo_scenes::WEATHER_FRAMES;
            g.update = Some(fx::tick_update);
        }
        // Phase 2: the 2 km island from three distances, and the lake's
        // planar reflection, shadows and underwater view.
        "terrain-near" => g.scene = crate::terrain_scenes::island_view("near"),
        "terrain-mid" => g.scene = crate::terrain_scenes::island_view("mid"),
        "terrain-far" => g.scene = crate::terrain_scenes::island_view("far"),
        "water-planar" => g.scene = crate::terrain_scenes::water_planar_scene(),
        "water-shadows" => g.scene = crate::terrain_scenes::water_shadows_scene(),
        "water-underwater" => g.scene = crate::terrain_scenes::water_underwater_scene(),
        "terrain-grass" => g.scene = crate::terrain_scenes::grass_scene(),
        other => {
            return Err(format!(
                "unknown gfx scene '{other}' (known: {})",
                SCENES.join(", ")
            ));
        }
    }
    Ok(g)
}

/// Rows top to bottom of an RGBA8 image made by `f(x, y) -> [r, g, b, a]`.
pub(crate) fn texture(
    size: u32,
    color_space: ColorSpace,
    f: impl Fn(f32, f32) -> [u8; 4],
) -> TextureData {
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let u = (x as f32 + 0.5) / size as f32;
            let v = (y as f32 + 0.5) / size as f32;
            rgba.extend_from_slice(&f(u, v));
        }
    }
    TextureData {
        width: size,
        height: size,
        rgba,
        color_space,
        mips: Mips::Generate,
    }
}

pub(crate) fn ground(scene: &mut RenderScene, size: f32, cell_m: f32, roughness: f32) {
    let checker = scene.add_texture(shapes::checker(512, 16));
    let material = scene.add_material(Material {
        base_color_texture: Some(checker),
        roughness,
        ..Default::default()
    });
    let mesh = scene.add_mesh(shapes::plane(size, size / (16.0 * cell_m)));
    scene.add_instance(mesh, material, Mat4::IDENTITY);
}

/// The 400 m field. `tall` adds the 8 m pillars and the 20 m wall; without
/// them only short casters remain (0.5 m blocks, 0.6 m kerbs), whose shadows
/// a bias too large for their size would lose.
pub fn cascades_200m(tall: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    ground(&mut scene, 420.0, 1.0, 0.9);
    let pillar = scene.add_mesh(shapes::cuboid(Vec3::new(1.0, 8.0, 1.0)));
    let block = scene.add_mesh(shapes::cuboid(Vec3::splat(0.5)));
    let wall = scene.add_mesh(shapes::cuboid(Vec3::new(30.0, 20.0, 2.0)));
    let stone = scene.add_material(Material {
        base_color: [0.75, 0.72, 0.68, 1.0],
        roughness: 0.8,
        ..Default::default()
    });
    let red = scene.add_material(Material {
        base_color: [0.8, 0.15, 0.1, 1.0],
        roughness: 0.6,
        ..Default::default()
    });
    // Kerbs running away from the camera: the sun throws their shadows
    // sideways onto the ground in view.
    let kerb = scene.add_mesh(shapes::cuboid(Vec3::new(1.0, 0.6, 30.0)));
    for d in [12.0f32, 32.0, 80.0, 150.0] {
        scene.add_instance(
            kerb,
            red,
            Mat4::from_translation(Vec3::new(1.5 + d * 0.05, 0.3, -d)),
        );
    }
    let pillars: &[f32] = if tall {
        &[
            3.0, 6.0, 10.0, 16.0, 24.0, 35.0, 50.0, 70.0, 95.0, 125.0, 160.0, 190.0,
        ]
    } else {
        &[]
    };
    for (i, d) in pillars.iter().enumerate() {
        let x = if i % 2 == 0 { -5.0 } else { 5.0 } * (1.0 + d / 60.0);
        scene.add_instance(pillar, stone, Mat4::from_translation(Vec3::new(x, 4.0, -d)));
    }
    // Small blocks near the camera: shadows of small objects in cascade 0.
    for i in 0..6 {
        let z = -3.0 - i as f32 * 0.9;
        scene.add_instance(
            block,
            red,
            Mat4::from_translation(Vec3::new(-1.2, 0.25, z)) * Mat4::from_rotation_y(i as f32),
        );
    }
    if tall {
        scene.add_instance(
            wall,
            stone,
            Mat4::from_translation(Vec3::new(-20.0, 10.0, -200.0)),
        );
    }
    scene.camera = Camera {
        position: Vec3::new(0.0, 5.0, 6.0),
        target: Vec3::new(0.0, 0.0, -45.0),
        far: 1000.0,
        exposure: Exposure::linear(0.8),
        ..Default::default()
    };
    let sun = Vec3::new(0.8, 0.45, 0.25);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 4.0,
        ..Default::default()
    };
    scene.environment.sky.sun_direction = sun;
    scene
}

fn point_spot_lights() -> RenderScene {
    let mut scene = RenderScene::new();
    ground(&mut scene, 30.0, 0.5, 0.7);
    let pillar = scene.add_mesh(shapes::cuboid(Vec3::new(0.6, 3.0, 0.6)));
    let sphere = scene.add_mesh(shapes::sphere(32, 24));
    let white = scene.add_material(Material {
        base_color: [0.85, 0.85, 0.85, 1.0],
        roughness: 0.5,
        ..Default::default()
    });
    let metal = scene.add_material(Material {
        base_color: [0.9, 0.8, 0.6, 1.0],
        metallic: 1.0,
        roughness: 0.3,
        ..Default::default()
    });
    for (x, z) in [
        (-3.0, -2.0),
        (2.5, -3.0),
        (0.0, 2.5),
        (-4.0, 3.0),
        (4.5, 1.5),
    ] {
        scene.add_instance(pillar, white, Mat4::from_translation(Vec3::new(x, 1.5, z)));
    }
    for (x, z) in [(-1.2, -0.5), (1.5, 0.3), (0.2, -2.0)] {
        scene.add_instance(sphere, metal, Mat4::from_translation(Vec3::new(x, 0.5, z)));
    }
    scene.lights = vec![
        // Shadowed: a spot from above-left, and a point light among the pillars.
        Light {
            kind: LightKind::Spot {
                direction: Vec3::new(0.5, -1.0, 0.2),
                angle_degrees: 35.0,
                penumbra: 0.3,
            },
            position: Vec3::new(-4.0, 6.0, -1.0),
            color: Vec3::new(1.0, 0.95, 0.85),
            intensity: 90.0,
            range: 20.0,
            cast_shadows: true,
        },
        Light {
            position: Vec3::new(0.3, 1.2, 0.0),
            color: Vec3::new(1.0, 0.7, 0.4),
            intensity: 12.0,
            range: 12.0,
            cast_shadows: true,
            ..Default::default()
        },
        Light {
            position: Vec3::new(-5.0, 0.8, -5.0),
            color: Vec3::new(1.0, 0.1, 0.1),
            intensity: 8.0,
            range: 7.0,
            ..Default::default()
        },
        Light {
            position: Vec3::new(5.0, 0.8, -5.0),
            color: Vec3::new(0.1, 1.0, 0.2),
            intensity: 8.0,
            range: 7.0,
            ..Default::default()
        },
        Light {
            position: Vec3::new(5.0, 0.8, 5.0),
            color: Vec3::new(0.2, 0.3, 1.0),
            intensity: 8.0,
            range: 7.0,
            ..Default::default()
        },
        Light {
            position: Vec3::new(-6.0, 0.6, 5.0),
            color: Vec3::new(1.0, 0.5, 1.0),
            intensity: 6.0,
            range: 6.0,
            ..Default::default()
        },
        Light {
            kind: LightKind::Spot {
                direction: Vec3::new(-1.0, -0.6, -0.2),
                angle_degrees: 20.0,
                penumbra: 0.6,
            },
            position: Vec3::new(8.0, 3.0, 2.0),
            color: Vec3::new(0.3, 1.0, 1.0),
            intensity: 60.0,
            range: 18.0,
            cast_shadows: false,
        },
    ];
    scene.light.intensity = 0.0;
    scene.light.cast_shadows = false;
    scene.environment = Environment {
        intensity: 0.04,
        background: Background::Color(Vec3::new(0.004, 0.006, 0.015)),
        ..Default::default()
    };
    scene.camera = Camera {
        position: Vec3::new(0.0, 7.0, 11.0),
        target: Vec3::new(0.0, 0.5, 0.0),
        exposure: Exposure::linear(1.2),
        ..Default::default()
    };
    scene
}

/// Thin poles, a sphere and a fine checker at a grazing angle: aliasing.
pub fn thin_geometry() -> RenderScene {
    let mut scene = RenderScene::new();
    ground(&mut scene, 60.0, 0.25, 0.8);
    let pole = scene.add_mesh(shapes::cuboid(Vec3::new(0.04, 4.0, 0.04)));
    let bar = scene.add_mesh(shapes::cuboid(Vec3::new(6.0, 0.05, 0.05)));
    let sphere = scene.add_mesh(shapes::sphere(32, 24));
    let dark = scene.add_material(Material {
        base_color: [0.05, 0.05, 0.06, 1.0],
        roughness: 0.4,
        ..Default::default()
    });
    for i in 0..12 {
        let x = -6.0 + i as f32 * 1.1;
        let z = -2.0 - i as f32 * 0.6;
        scene.add_instance(
            pole,
            dark,
            Mat4::from_translation(Vec3::new(x, 2.0, z)) * Mat4::from_rotation_z(0.15),
        );
    }
    for i in 0..5 {
        scene.add_instance(
            bar,
            dark,
            Mat4::from_translation(Vec3::new(0.0, 0.6 + i as f32 * 0.5, -1.0))
                * Mat4::from_rotation_z(0.08 * i as f32),
        );
    }
    let shiny = scene.add_material(Material {
        base_color: [0.9, 0.9, 0.95, 1.0],
        metallic: 1.0,
        roughness: 0.15,
        ..Default::default()
    });
    scene.add_instance(
        sphere,
        shiny,
        Mat4::from_translation(Vec3::new(2.0, 1.0, 1.0)) * Mat4::from_scale(Vec3::splat(2.0)),
    );
    scene.camera = Camera {
        position: Vec3::new(0.0, 1.6, 6.0),
        target: Vec3::new(0.0, 1.2, -4.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene
}

fn emissive_spheres() -> RenderScene {
    let mut scene = RenderScene::new();
    ground(&mut scene, 30.0, 0.5, 0.6);
    let sphere = scene.add_mesh(shapes::sphere(32, 24));
    for (i, c) in [
        [8.0, 2.0, 0.5],
        [0.5, 3.0, 10.0],
        [2.0, 9.0, 3.0],
        [12.0, 12.0, 12.0],
        [10.0, 0.5, 6.0],
    ]
    .iter()
    .enumerate()
    {
        let m = scene.add_material(Material {
            base_color: [0.0, 0.0, 0.0, 1.0],
            emissive: *c,
            ..Default::default()
        });
        let x = -4.0 + i as f32 * 2.0;
        scene.add_instance(
            sphere,
            m,
            Mat4::from_translation(Vec3::new(x, 0.6 + (i % 2) as f32 * 0.8, -(i as f32) * 0.7))
                * Mat4::from_scale(Vec3::splat(0.8 + 0.2 * (i % 3) as f32)),
        );
    }
    scene.light.intensity = 0.3;
    scene.environment = Environment {
        intensity: 0.08,
        background: Background::Color(Vec3::new(0.01, 0.012, 0.02)),
        ..Default::default()
    };
    scene.camera = Camera {
        position: Vec3::new(0.0, 3.0, 8.0),
        target: Vec3::new(0.0, 0.8, -1.0),
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene
}

/// A dusk scene lit `brightness` times: metered exposure brings it to middle grey.
pub fn dim_scene(brightness: f32) -> RenderScene {
    let mut scene = RenderScene::new();
    ground(&mut scene, 40.0, 1.0, 0.8);
    let sphere = scene.add_mesh(shapes::sphere(32, 24));
    let block = scene.add_mesh(shapes::cuboid(Vec3::new(1.5, 2.5, 1.5)));
    let warm = scene.add_material(Material {
        base_color: [0.9, 0.5, 0.25, 1.0],
        roughness: 0.4,
        ..Default::default()
    });
    let cool = scene.add_material(Material {
        base_color: [0.3, 0.5, 0.8, 1.0],
        roughness: 0.7,
        ..Default::default()
    });
    for i in 0..5 {
        let x = -6.0 + i as f32 * 3.0;
        scene.add_instance(sphere, warm, Mat4::from_translation(Vec3::new(x, 0.5, 0.0)));
        scene.add_instance(
            block,
            cool,
            Mat4::from_translation(Vec3::new(x + 1.5, 1.25, -4.0)),
        );
    }
    let sun = Vec3::new(-0.6, 0.25, 0.4);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 0.15 * brightness,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            sun_color: Vec3::new(1.0, 0.7, 0.45),
            look: None,
        },
        intensity: 0.03 * brightness,
        background: Background::Color(Vec3::new(0.01, 0.012, 0.02) * brightness),
    };
    scene.camera = Camera {
        position: Vec3::new(0.0, 3.0, 10.0),
        target: Vec3::new(0.0, 1.0, -1.0),
        exposure: Exposure::auto(),
        ..Default::default()
    };
    scene
}

/// Sprite textures: a soft disc, a 2x2 atlas of shapes, a ring.
pub(crate) fn sprite_textures(scene: &mut RenderScene) -> [og_render::TextureHandle; 3] {
    let disc = scene.add_texture(texture(64, ColorSpace::Srgb, |u, v| {
        let d = ((u - 0.5).powi(2) + (v - 0.5).powi(2)).sqrt() * 2.0;
        let a = ((1.0 - d) * 4.0).clamp(0.0, 1.0);
        [255, 255, 255, (a * 255.0) as u8]
    }));
    let atlas = scene.add_texture(texture(128, ColorSpace::Srgb, |u, v| {
        let (qu, qv) = ((u * 2.0).fract(), (v * 2.0).fract());
        let quadrant = (u >= 0.5) as u8 + 2 * (v >= 0.5) as u8;
        let inside = match quadrant {
            0 => (qu - 0.5).abs() < 0.4 && (qv - 0.5).abs() < 0.4,
            1 => (qu - 0.5).abs() + (qv - 0.5).abs() < 0.45,
            2 => qv > 0.1 && (qu - 0.5).abs() < (qv - 0.1) * 0.5,
            _ => ((qu - 0.5).powi(2) + (qv - 0.5).powi(2)).sqrt() < 0.42,
        };
        let colors = [
            [255, 200, 60],
            [80, 220, 255],
            [140, 255, 120],
            [255, 110, 200],
        ];
        let c = colors[quadrant as usize];
        [c[0], c[1], c[2], if inside { 255 } else { 0 }]
    }));
    let ring = scene.add_texture(texture(64, ColorSpace::Srgb, |u, v| {
        let d = ((u - 0.5).powi(2) + (v - 0.5).powi(2)).sqrt() * 2.0;
        let a = (1.0 - ((d - 0.75).abs() * 8.0)).clamp(0.0, 1.0);
        [255, 255, 255, (a * 255.0) as u8]
    }));
    [disc, atlas, ring]
}

/// Where sprite `i` of `n` sits at `frame`: a slowly turning sunflower spiral.
fn sprite_at(i: usize, n: usize, frame: u32) -> (Vec2, f32) {
    let golden = std::f32::consts::PI * (3.0 - 5f32.sqrt());
    let r = (i as f32 / n as f32).sqrt() * 9.5;
    let a = i as f32 * golden + frame as f32 * 0.01;
    (Vec2::new(r * a.cos() * 1.7, r * a.sin()), a)
}

fn sprite_field(n: usize) -> Result<RenderScene, String> {
    let mut scene = RenderScene::new();
    let [disc, atlas, ring] = sprite_textures(&mut scene);
    for i in 0..n {
        let (p, a) = sprite_at(i, n, 0);
        let kind = i % 5;
        let (texture, uv_rect, layer, size) = match kind {
            0 | 1 => (disc, [0.0, 0.0, 1.0, 1.0], 0, 0.22),
            2 => (ring, [0.0, 0.0, 1.0, 1.0], 1, 0.3),
            _ => {
                let q = (i / 5) % 4;
                let (qu, qv) = ((q % 2) as f32 * 0.5, (q / 2) as f32 * 0.5);
                (atlas, [qu, qv, qu + 0.5, qv + 0.5], 1, 0.26)
            }
        };
        let hue = i as f32 / n as f32;
        scene.add_sprite(Sprite {
            texture: Some(texture),
            position: p.extend(0.0),
            size: Vec2::splat(size),
            rotation: a,
            color: [
                0.4 + 0.6 * (hue * std::f32::consts::TAU).cos().abs(),
                0.4 + 0.6 * (hue * 9.0).sin().abs(),
                0.9,
                0.9,
            ],
            uv_rect,
            layer,
            ..Default::default()
        });
    }
    scene.camera = Camera::orthographic_2d(Vec2::ZERO, 20.0);
    scene.environment.background = Background::Color(Vec3::new(0.02, 0.02, 0.03));
    scene.post.tonemap = Tonemap::None;
    Ok(scene)
}

/// Moves every sprite along the spiral (`frame` 0 is the reference frame).
pub fn move_sprites(scene: &mut RenderScene, frame: u32) {
    let n = scene.sprites.len();
    for (i, s) in scene.sprites.iter_mut().enumerate() {
        let (p, a) = sprite_at(i, n, frame);
        s.position = p.extend(0.0);
        s.rotation = a;
    }
}

/// The committed MSDF test atlas (`assets/fonts/noto-sans-sc-msdf.*`, see
/// `crates/og-render/README.md` §Testing): the atlas PNG plus the glyph CSV
/// and metrics JSON `msdf-atlas-gen` wrote with `-yorigin top`.
pub fn load_font(scene: &mut RenderScene) -> Result<og_render::FontHandle, String> {
    let png = "fonts/noto-sans-sc-msdf.png";
    let img = image::load_from_memory(&asset(png)?)
        .map_err(|e| format!("{png}: {e}"))?
        .to_rgba8();
    let (width, height) = img.dimensions();
    let atlas = scene.add_texture(TextureData {
        width,
        height,
        rgba: img.into_raw(),
        color_space: ColorSpace::Linear,
        mips: Mips::None,
    });
    let json_path = "fonts/noto-sans-sc-msdf.json";
    let json = asset_text(json_path)?;
    let number = |key: &str| -> Result<f32, String> {
        let at = json
            .find(&format!("\"{key}\":"))
            .ok_or_else(|| format!("{json_path}: no {key}"))?;
        let rest = &json[at + key.len() + 3..];
        let end = rest.find([',', '}']).unwrap_or(rest.len());
        rest[..end]
            .trim()
            .parse()
            .map_err(|e| format!("{json_path}: {key}: {e}"))
    };
    let csv_path = "fonts/noto-sans-sc-msdf.csv";
    let csv = asset_text(csv_path)?;
    let mut glyphs = Vec::new();
    for line in csv.lines().filter(|l| !l.trim().is_empty()) {
        let f: Vec<f32> = line
            .split(',')
            .map(|v| v.trim().parse::<f32>())
            .collect::<Result<_, _>>()
            .map_err(|e| format!("{csv_path}: {line}: {e}"))?;
        if f.len() != 10 {
            return Err(format!("{csv_path}: {line}: 10 columns expected"));
        }
        let codepoint = char::from_u32(f[0] as u32)
            .ok_or_else(|| format!("{csv_path}: bad codepoint {}", f[0]))?;
        // msdf-atlas-gen with -yorigin top: plane bounds left, top, right,
        // bottom with y down; atlas bounds left, top, right, bottom in pixels.
        glyphs.push(Glyph {
            codepoint,
            advance: f[1],
            plane_bounds: [f[2], -f[5], f[4], -f[3]],
            atlas_bounds: [f[6], f[7], f[8], f[9]],
        });
    }
    Ok(scene.add_font(MsdfFont {
        atlas,
        distance_range: number("distanceRange")?,
        line_height: number("lineHeight")?,
        glyphs,
        kerning: Vec::new(),
    }))
}

/// The non-whitespace characters of the text scene, for glyph counts.
pub const TEXT_LINES: &[(&str, f32, f32, f32, [f32; 4])] = &[
    // (text, x, y, em size, colour)
    ("Origin Engine", -9.2, 3.2, 2.4, [1.0, 1.0, 1.0, 1.0]),
    ("原点引擎 文字渲染", -9.2, 0.4, 1.5, [1.0, 0.8, 0.3, 1.0]),
    (
        "MSDF text: sharp at any size. 0123456789 !?@#&\nThe quick brown fox jumps over the lazy dog.",
        -9.2,
        -1.6,
        0.55,
        [0.6, 0.85, 1.0, 1.0],
    ),
    (
        "tiny 12px-ish text stays legible",
        -9.2,
        -4.2,
        0.32,
        [0.8, 0.8, 0.8, 1.0],
    ),
];

fn text_scene() -> Result<RenderScene, String> {
    let mut scene = RenderScene::new();
    let font = load_font(&mut scene)?;
    for (text, x, y, size, color) in TEXT_LINES {
        scene.texts.push(Text {
            position: Vec3::new(*x, *y, 0.0),
            size: *size,
            color: *color,
            ..Text::new(font, *text)
        });
    }
    scene.texts.push(Text {
        position: Vec3::new(1.0, -4.5, 0.0),
        size: 0.9,
        rotation: 0.35,
        color: [1.0, 0.4, 0.5, 1.0],
        layer: 1,
        ..Text::new(font, "rotated")
    });
    // A huge glyph: the distance field is magnified ~10x.
    scene.texts.push(Text {
        position: Vec3::new(6.0, -3.0, 0.0),
        size: 4.5,
        color: [0.35, 0.9, 0.5, 1.0],
        ..Text::new(font, "&")
    });
    scene.camera = Camera::orthographic_2d(Vec2::ZERO, 10.5);
    scene.environment.background = Background::Color(Vec3::new(0.015, 0.02, 0.035));
    scene.post.tonemap = Tonemap::None;
    Ok(scene)
}

fn sky_only() -> RenderScene {
    let mut scene = RenderScene::new();
    let sun = Vec3::new(0.3, 0.12, -0.9);
    scene.environment.sky = ProceduralSky {
        sun_direction: sun,
        sun_color: Vec3::new(1.0, 0.85, 0.65),
        look: None,
    };
    scene.light.direction_to_light = sun;
    scene.camera = Camera {
        position: Vec3::new(0.0, 1.0, 0.0),
        target: Vec3::new(0.15, 1.15, -1.0),
        projection: Projection::Perspective {
            fov_y_degrees: 70.0,
        },
        exposure: Exposure::linear(0.8),
        ..Default::default()
    };
    scene
}

fn showcase() -> Result<RenderScene, String> {
    let mut scene = RenderScene::new();
    let models = [load("trickster.glb")?, load("dropship.glb")?];
    let parts: Vec<_> = models.iter().map(|m| (m.add_to(&mut scene), m)).collect();
    crate::fx_scenes::terrain(&mut scene, 60.0);
    let place = |scene: &mut RenderScene, k: usize, pos: Vec3, yaw: f32, height: f32| {
        let (p, m) = &parts[k];
        let scale = height / (m.max - m.min).max_element().max(1e-3);
        let t = Mat4::from_translation(pos + Vec3::Y * (-m.min.y * scale))
            * Mat4::from_quat(Quat::from_rotation_y(yaw))
            * Mat4::from_scale(Vec3::splat(scale));
        p.instance(scene, t);
    };
    for (i, x) in [-3.0f32, 0.0, 3.0].iter().enumerate() {
        place(
            &mut scene,
            0,
            Vec3::new(*x, 0.0, -1.0 - i as f32),
            0.4 - i as f32 * 0.4,
            2.0,
        );
    }
    place(&mut scene, 1, Vec3::new(-5.0, 0.0, -5.0), 1.2, 3.0);
    place(&mut scene, 1, Vec3::new(5.5, 0.0, -4.0), -0.6, 3.0);
    let orb = scene.add_mesh(shapes::sphere(32, 24));
    for (i, c) in [[10.0, 3.0, 1.0], [1.0, 4.0, 12.0], [3.0, 12.0, 4.0]]
        .iter()
        .enumerate()
    {
        let m = scene.add_material(Material {
            base_color: [0.0, 0.0, 0.0, 1.0],
            emissive: *c,
            ..Default::default()
        });
        scene.add_instance(
            orb,
            m,
            Mat4::from_translation(Vec3::new(-2.0 + i as f32 * 2.2, 2.6, 0.8))
                * Mat4::from_scale(Vec3::splat(0.35)),
        );
    }
    scene.lights = vec![
        Light {
            kind: LightKind::Spot {
                direction: Vec3::new(0.2, -1.0, -0.3),
                angle_degrees: 30.0,
                penumbra: 0.4,
            },
            position: Vec3::new(-1.0, 7.0, 2.0),
            color: Vec3::new(1.0, 0.9, 0.75),
            intensity: 120.0,
            range: 20.0,
            cast_shadows: true,
        },
        Light {
            position: Vec3::new(4.0, 1.5, -1.5),
            color: Vec3::new(0.3, 0.6, 1.0),
            intensity: 10.0,
            range: 9.0,
            cast_shadows: true,
            ..Default::default()
        },
        Light {
            position: Vec3::new(-4.5, 1.0, 0.5),
            color: Vec3::new(1.0, 0.3, 0.1),
            intensity: 6.0,
            range: 6.0,
            ..Default::default()
        },
    ];
    let sun = Vec3::new(-0.7, 0.3, -0.5);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        color: Vec3::new(1.0, 0.75, 0.5),
        intensity: 1.5,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            sun_color: Vec3::new(1.0, 0.7, 0.45),
            look: None,
        },
        intensity: 0.35,
        background: Background::Sky,
    };
    scene.post.bloom = Some(Bloom::default());
    scene.camera = Camera {
        position: Vec3::new(0.5, 3.2, 9.0),
        target: Vec3::new(0.0, 1.3, -1.5),
        exposure: Exposure::Auto {
            compensation: 0.3,
            min_ev: -4.0,
            max_ev: 4.0,
            adapt_speed_up: 0.0,
            adapt_speed_down: 0.0,
        },
        ..Default::default()
    };
    // A world-space label and a few sprites in the xy plane, facing the camera.
    let font = load_font(&mut scene)?;
    scene.texts.push(Text {
        position: Vec3::new(-3.1, 4.3, -2.0),
        size: 0.8,
        color: [3.0, 2.6, 1.6, 1.0],
        ..Text::new(font, "Origin Engine 原点引擎")
    });
    let [disc, _, ring] = sprite_textures(&mut scene);
    for i in 0..40 {
        let a = i as f32 * 0.9;
        scene.add_sprite(Sprite {
            texture: Some(if i % 3 == 0 { ring } else { disc }),
            position: Vec3::new(
                a.cos() * (2.0 + i as f32 * 0.12),
                3.0 + (a * 1.7).sin() * 0.8,
                -2.5,
            ),
            size: Vec2::splat(0.18 + (i % 4) as f32 * 0.05),
            color: [2.0, 1.4, 0.6, 0.8],
            ..Default::default()
        });
    }
    Ok(scene)
}

/// The spike's time for its last timed frame (frame 59 of 60).
const SPIKE_TIME: f32 = 59.0 / 60.0;
const SUN: Vec3 = Vec3::new(0.45, 0.8, 0.35);
const SUN_COLOR: Vec3 = Vec3::new(1.0, 0.95, 0.85);

pub(crate) fn load(name: &str) -> Result<GltfModel, String> {
    load_glb(&asset(name)?).map_err(|e| format!("{name}: {e}"))
}

fn spike_gltf_scene() -> Result<RenderScene, String> {
    let models = [load("trickster.glb")?, load("dropship.glb")?];
    let mut scene = RenderScene::new();
    let mut placed = Vec::new();
    for model in &models {
        let parts = model.add_to(&mut scene);
        placed.push((parts, model.min, model.max));
    }
    let parts: Vec<_> = placed
        .iter()
        .map(|(p, min, max)| (p.parts.clone(), *min, *max))
        .collect();
    layout(&mut scene, &parts, 25, 3.0);
    Ok(scene)
}

/// Moves every instance of the sphere field up or down on its own phase.
fn bob_spheres(scene: &mut RenderScene, frame: u32) {
    let t = frame as f32 / 60.0;
    for (i, inst) in scene.instances.iter_mut().enumerate() {
        let speed = 1.0 + (i % 7) as f32 * 0.25;
        let phase = (i as f32 * 0.37) % std::f32::consts::TAU;
        let dy = (t * speed + phase).sin() - ((t - 1.0 / 60.0) * speed + phase).sin();
        inst.transform.w_axis.y += dy * 0.5;
    }
}

fn sphere_field(count: u32) -> RenderScene {
    let mut scene = RenderScene::new();
    let mesh = scene.add_mesh(shapes::sphere(16, 12));
    let material = scene.add_material(Material {
        base_color: [0.85, 0.35, 0.2, 1.0],
        metallic: 0.1,
        roughness: 0.45,
        ..Default::default()
    });
    layout(
        &mut scene,
        &[(vec![(mesh, material)], Vec3::splat(-0.5), Vec3::splat(0.5))],
        count,
        1.2,
    );
    scene
}

type Parts = (
    Vec<(og_render::MeshHandle, og_render::MaterialHandle)>,
    Vec3,
    Vec3,
);

/// The spike's `reference_scene`: `count` instances of each model on a grid,
/// a checker ground, the camera orbiting the bounds at `SPIKE_TIME`, and the
/// shadow box fitted to the same bounds. The arithmetic follows the spike
/// exactly (this reproduced `.spike-out/native.png` byte for byte, see
/// `tests/evidence/M0/NOTES.md`).
fn layout(scene: &mut RenderScene, models: &[Parts], count: u32, spacing: f32) {
    let n = models.len().max(1) as f32;
    let side_count = (count as f32).sqrt().ceil();
    let ground_size = (side_count * spacing + 6.0).max(12.0);
    let mut min = Vec3::new(-ground_size / 2.0, 0.0, -ground_size / 2.0);
    let mut max = Vec3::new(ground_size / 2.0, 0.0, ground_size / 2.0);
    for (k, (parts, model_min, model_max)) in models.iter().enumerate() {
        let extent = *model_max - *model_min;
        let scale = 2.0 / extent.max_element().max(1e-3);
        let offset = Vec3::new(0.0, 0.0, (k as f32 - (n - 1.0) / 2.0) * spacing * 0.5);
        let side = side_count as u32;
        let corners = [*model_min, *model_max];
        for i in 0..count {
            let x = (i % side) as f32 - (side as f32 - 1.0) / 2.0;
            let z = (i / side) as f32 - (side as f32 - 1.0) / 2.0;
            let rot = Quat::from_rotation_y((i as f32 * 0.7).sin() * 1.2);
            let grid =
                Mat4::from_translation(Vec3::new(x * spacing, -model_min.y * scale, z * spacing))
                    * Mat4::from_quat(rot)
                    * Mat4::from_scale(Vec3::splat(scale));
            let m = Mat4::from_translation(offset) * grid;
            for &(mesh, material) in parts {
                scene.add_instance(mesh, material, m);
            }
            for c in 0..8 {
                let corner = Vec3::new(
                    corners[c & 1].x,
                    corners[(c >> 1) & 1].y,
                    corners[(c >> 2) & 1].z,
                );
                let w = m.transform_point3(corner);
                min = min.min(w);
                max = max.max(w);
            }
        }
    }
    let checker = scene.add_texture(shapes::checker(512, 16));
    let ground_material = scene.add_material(Material {
        base_color: [1.0; 4],
        base_color_texture: Some(checker),
        metallic: 0.0,
        roughness: 0.85,
        ..Default::default()
    });
    let ground = scene.add_mesh(shapes::plane(ground_size, ground_size / 2.0));
    scene.add_instance(ground, ground_material, Mat4::IDENTITY);

    let center = (min + max) / 2.0;
    let radius = (max - min).length() / 2.0;
    let angle = 0.6 + SPIKE_TIME * 0.25;
    let dist = radius * 1.05;
    scene.camera = Camera {
        position: center + Vec3::new(angle.cos() * dist, radius * 0.5, angle.sin() * dist),
        target: center,
        up: Vec3::Y,
        projection: Projection::Perspective {
            fov_y_degrees: 50.0,
        },
        near: 0.1,
        far: radius * 8.0,
        exposure: Exposure::linear(0.8),
    };
    scene.light = DirectionalLight {
        direction_to_light: SUN,
        color: SUN_COLOR,
        intensity: 4.0,
        cast_shadows: true,
        // Cascades reach the far side of the instances' bounds.
        shadow_bounds: ShadowBounds::Auto,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: SUN,
            sun_color: SUN_COLOR,
            look: None,
        },
        intensity: 0.55,
        background: Background::Sky,
    };
}

/// A 256x256 map of 1-unit tiles in four layers, drawn as og-tilemap chunks.
pub fn tilemap_256() -> RenderScene {
    use og_tilemap::{Layer, MapState, TilemapAsset, Tileset, TilesetRef, chunk_quads};
    use std::sync::Arc;
    let mut scene = RenderScene::new();
    // A 128x128 atlas of 16 tiles of 32 px, each a distinct colour.
    let mut rgba = Vec::with_capacity(128 * 128 * 4);
    for y in 0..128u32 {
        for x in 0..128u32 {
            let t = (y / 32) * 4 + x / 32;
            let edge = x % 32 == 0 || y % 32 == 0;
            let c = [
                (t * 37 % 200 + 40) as u8,
                (t * 71 % 200 + 40) as u8,
                (t * 113 % 200 + 40) as u8,
            ];
            let k = if edge { 2 } else { 1 };
            rgba.extend_from_slice(&[c[0] / k, c[1] / k, c[2] / k, 255]);
        }
    }
    let texture = scene.add_texture(TextureData {
        width: 128,
        height: 128,
        rgba,
        color_space: ColorSpace::Srgb,
        mips: Mips::None,
    });
    scene.set_texture_filter(texture, og_render::TextureFilter::Nearest);
    let tileset = Tileset::parse(
        "type tileset\nversion 1\n---\ntexture \"t.png\"\ngrid 32 32 4 4\ntile 15 {\n  frames 15 14 13\n  fps 4\n}\n",
    )
    .expect("tileset");
    let n = 256u32;
    let mut layers = Vec::new();
    for l in 0..4u32 {
        let mut layer = Layer::new(format!("l{l}"), n, n, l as i32);
        for (i, c) in layer.cells.iter_mut().enumerate() {
            let (x, y) = (i as u32 % n, i as u32 / n);
            let h = (x.wrapping_mul(73_856_093)
                ^ y.wrapping_mul(19_349_663)
                ^ l.wrapping_mul(83_492_791))
                % 100;
            // Layer 0 is full ground, the others sparser decoration.
            let filled = l == 0 || h < [100, 40, 20, 10][l as usize];
            *c = if filled { 1 + (h % 16) } else { 0 };
        }
        layers.push(layer);
    }
    let asset = TilemapAsset {
        width: n,
        height: n,
        tile_width_px: 32,
        tile_height_px: 32,
        unit: 1.0,
        tilesets: vec![TilesetRef::Embedded(tileset.clone())],
        layers,
        objects: Vec::new(),
    };
    let state = MapState::new(&asset, &[Arc::new(tileset)]).expect("map");
    let (cw, ch) = state.chunks();
    for l in 0..4usize {
        let mut chunks = Vec::new();
        for cy in 0..ch {
            for cx in 0..cw {
                let q = chunk_quads(&state, l, cx, cy, 128, 128);
                let mut min = Vec2::splat(f32::MAX);
                let mut max = Vec2::splat(f32::MIN);
                for t in &q.quads {
                    min = min.min(t.min);
                    max = max.max(t.min + t.size);
                }
                chunks.push(og_render::TileChunk {
                    min,
                    max,
                    quads: q
                        .quads
                        .iter()
                        .map(|t| og_render::TileQuad {
                            min: t.min,
                            size: t.size,
                            uv: t.uv,
                        })
                        .collect(),
                    animated: q
                        .animated
                        .iter()
                        .map(|a| og_render::TileAnimation {
                            quad: a.quad,
                            uvs: a.frames.clone(),
                            durations: a.durations.clone(),
                        })
                        .collect(),
                });
            }
        }
        scene.tile_layers.push(og_render::TileLayer {
            texture,
            chunks: Arc::new(chunks),
            origin: Vec3::new(0.0, 256.0, 0.0),
            layer: l as i32 - 10,
            color: [1.0; 4],
            repeat: [None, None],
            time: 0.0,
        });
    }
    scene.camera = Camera::orthographic_2d(Vec2::new(64.0, 128.0), 45.0);
    scene.environment.background = Background::Color(Vec3::new(0.02, 0.02, 0.03));
    scene.post.tonemap = Tonemap::None;
    scene
}

/// Pans the tilemap camera diagonally; layer 3 scrolls at half the camera's
/// speed and the animated tiles follow the frame's time.
pub fn pan_tilemap(scene: &mut RenderScene, frame: u32) {
    let t = frame as f32;
    let c = Vec2::new(64.0 + (t * 0.37) % 120.0, 128.0 - (t * 0.21) % 60.0);
    scene.camera = Camera::orthographic_2d(c, 45.0);
    for (i, l) in scene.tile_layers.iter_mut().enumerate() {
        l.time = t / 60.0;
        l.origin = if i == 3 {
            Vec3::new(c.x * 0.5, 256.0 + c.y * 0.5, 0.0)
        } else {
            Vec3::new(0.0, 256.0, 0.0)
        };
    }
}
