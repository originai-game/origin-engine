//! The atmosphere scenes of the E5 round (`crates/og-render/README.md`
//! §Atmosphere): fog with a procedural sky, the day-night cycle at dusk and
//! at night, planar 3D water with a rippling height field, a side-on 2D
//! water body, a 2D fluid drawn as metaballs, and material shader hooks
//! (a dissolve by noise and wind-swayed grass), and weather presets.

use crate::fx_scenes::terrain;
use glam::{Mat4, Vec2, Vec3};
use og_render::{
    Background, Bloom, Camera, DirectionalLight, Environment, Exposure, Fog, Material,
    MaterialShader, MeshData, Metaballs, ParamKind, ProceduralSky, RenderScene, ShaderParam,
    Sprite, StandardMaterial, Tonemap, Vertex, Water, Water2d, WaterHeights, day_night, shapes,
};
use std::sync::Arc;

/// Pillars and boulders receding into the distance, for fog to swallow.
fn colonnade(scene: &mut RenderScene) {
    terrain(scene, 400.0);
    let pillar = scene.add_mesh(shapes::cuboid(Vec3::new(1.0, 8.0, 1.0)));
    let rock = scene.add_mesh(shapes::sphere(20, 14));
    let stone = scene.add_material(Material {
        base_color: [0.72, 0.68, 0.62, 1.0],
        roughness: 0.85,
        ..Default::default()
    });
    let moss = scene.add_material(Material {
        base_color: [0.34, 0.42, 0.26, 1.0],
        roughness: 0.95,
        ..Default::default()
    });
    for i in 0..14 {
        let z = -6.0 - i as f32 * 9.0;
        for x in [-6.0f32, 6.0] {
            scene.add_instance(pillar, stone, Mat4::from_translation(Vec3::new(x, 4.0, z)));
        }
        let r = 1.2 + (i % 3) as f32 * 0.7;
        scene.add_instance(
            rock,
            moss,
            Mat4::from_translation(Vec3::new(if i % 2 == 0 { -14.0 } else { 15.0 }, r * 0.3, z - 3.0))
                * Mat4::from_scale(Vec3::splat(r * 2.0)),
        );
    }
}

fn colonnade_camera() -> Camera {
    Camera {
        position: Vec3::new(0.0, 2.2, 4.0),
        target: Vec3::new(0.0, 3.4, -40.0),
        exposure: Exposure::linear(0.8),
        ..Default::default()
    }
}

/// The colonnade at `hours` with the day-night look, drawn over the
/// procedural sky, with fog tinted to the horizon (`fog: false`: none).
pub fn day_night_scene(hours: f32, fog: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    colonnade(&mut scene);
    let look = day_night(hours, 25.0, -20.0);
    scene.light = DirectionalLight {
        direction_to_light: look.light_direction,
        color: look.light_color,
        intensity: look.light_intensity,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: look.sun_direction,
            sun_color: look.sun_color,
            look: Some(look.sky),
        },
        intensity: look.ambient,
        background: Background::Procedural {
            sun_direction: look.sun_direction,
            sun_color: look.sun_color,
        },
    };
    if fog {
        scene.fog = Some(Fog {
            color: look.fog_color,
            density: 0.012,
            start: 8.0,
            height_density: 0.05,
            height: 0.0,
            falloff: 0.8,
            sky: 0.35,
            ..Default::default()
        });
    }
    scene.post.bloom = Some(Bloom::default());
    scene.camera = colonnade_camera();
    scene
}

/// Distance and height fog over the colonnade in daylight.
pub fn fog_scene() -> RenderScene {
    day_night_scene(10.5, true)
}

/// Ripples from three drops on a `n` x `n` field over `size` metres.
pub fn ripples(n: u32, size: f32) -> WaterHeights {
    let drops = [(-2.5f32, 1.5f32, 0.0f32), (3.0, -1.0, 1.6), (0.5, 3.5, 3.1)];
    let mut h = Vec::with_capacity((n * n) as usize);
    for row in 0..n {
        for col in 0..n {
            let x = (col as f32 / (n - 1) as f32 - 0.5) * size;
            let z = (row as f32 / (n - 1) as f32 - 0.5) * size;
            let mut y = 0.0;
            for (dx, dz, phase) in drops {
                let d = ((x - dx).powi(2) + (z - dz).powi(2)).sqrt();
                y += 0.16 * (d * 2.4 - phase * 2.0).cos() * (-d * 0.3).exp();
            }
            h.push(y);
        }
    }
    WaterHeights {
        resolution: n,
        size,
        heights: Arc::from(h),
    }
}

/// A pool under an afternoon sky: a sloping sand bed (depth colour from
/// shallow to deep), posts and a rock breaking the surface (foam), a
/// rippling height field, the sky and the posts reflected.
pub fn water_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let sand = scene.add_material(Material {
        base_color: [0.76, 0.68, 0.5, 1.0],
        roughness: 0.9,
        ..Default::default()
    });
    let grass = scene.add_material(Material {
        base_color: [0.3, 0.42, 0.2, 1.0],
        roughness: 0.95,
        ..Default::default()
    });
    let wood = scene.add_material(Material {
        base_color: [0.42, 0.28, 0.16, 1.0],
        roughness: 0.7,
        ..Default::default()
    });
    let red = scene.add_material(Material {
        base_color: [0.8, 0.16, 0.1, 1.0],
        roughness: 0.4,
        ..Default::default()
    });
    // The bed: a slab sloping from 0.5 m below the surface at the near
    // edge to 4 m at the far one; grass banks on both sides.
    let bed = scene.add_mesh(shapes::cuboid(Vec3::new(60.0, 0.4, 60.0)));
    scene.add_instance(
        bed,
        sand,
        Mat4::from_translation(Vec3::new(0.0, -2.8, -22.0)) * Mat4::from_rotation_x(-0.07),
    );
    let bank = scene.add_mesh(shapes::cuboid(Vec3::new(20.0, 2.0, 60.0)));
    for x in [-17.0f32, 17.0] {
        scene.add_instance(
            bank,
            grass,
            Mat4::from_translation(Vec3::new(x * 1.12, -1.6, -22.0)) * Mat4::from_rotation_z(x.signum() * 0.35),
        );
    }
    let post = scene.add_mesh(shapes::cuboid(Vec3::new(0.35, 4.0, 0.35)));
    for (x, z) in [(-3.0f32, -2.0f32), (-1.0, -5.0), (2.5, -3.0), (4.0, -8.0)] {
        scene.add_instance(post, wood, Mat4::from_translation(Vec3::new(x, 0.6, z)));
    }
    let ball = scene.add_mesh(shapes::sphere(24, 16));
    scene.add_instance(
        ball,
        red,
        Mat4::from_translation(Vec3::new(0.6, 0.15, -1.0)) * Mat4::from_scale(Vec3::splat(1.2)),
    );
    let sun = Vec3::new(-0.5, 0.45, -0.75);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        color: Vec3::new(1.0, 0.92, 0.8),
        intensity: 3.0,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            sun_color: Vec3::new(1.0, 0.9, 0.75),
            look: None,
        },
        intensity: 0.6,
        background: Background::Sky,
    };
    scene.water.push(Water {
        transform: Mat4::from_translation(Vec3::new(0.0, 0.0, -6.0)),
        size: Vec2::new(40.0, 90.0),
        segments: 96,
        heights: Some(ripples(64, 22.0)),
        ..Default::default()
    });
    scene.camera = Camera {
        position: Vec3::new(0.0, 2.6, 6.0),
        target: Vec3::new(0.0, 0.0, -6.0),
        exposure: Exposure::linear(0.85),
        ..Default::default()
    };
    scene
}

/// A side-on level: a tiled wall behind, a 2D water body whose surface
/// carries a splash and travelling waves, a crate half under it (drawn
/// before the water, so it shows through), a post in front of it.
pub fn water_2d_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let tile = crate::fx_scenes::tile_texture(&mut scene);
    let crate_tex = crate::fx_scenes::crate_texture(&mut scene);
    for y in -7..7 {
        for x in -12..12 {
            scene.add_sprite(Sprite {
                texture: Some(tile),
                position: Vec3::new(x as f32 + 0.5, y as f32 + 0.5, 0.0),
                size: Vec2::ONE,
                color: [0.55, 0.6, 0.7, 1.0],
                layer: 0,
                ..Default::default()
            });
        }
    }
    scene.add_sprite(Sprite {
        texture: Some(crate_tex),
        position: Vec3::new(-2.5, -1.4, 0.0),
        size: Vec2::splat(1.4),
        rotation: 0.2,
        layer: 1,
        ..Default::default()
    });
    scene.add_sprite(Sprite {
        position: Vec3::new(4.0, -2.0, 0.0),
        size: Vec2::new(0.5, 6.0),
        color: [0.35, 0.24, 0.15, 1.0],
        layer: 3,
        ..Default::default()
    });
    let columns = 96;
    let surface = (0..columns)
        .map(|i| {
            let x = i as f32 / (columns - 1) as f32 * 20.0 - 10.0;
            let splash = 0.6 * (-(x - 1.0) * (x - 1.0) * 1.5).exp() - 0.25 * (-(x - 1.0) * (x - 1.0) * 0.3).exp();
            splash + 0.08 * (x * 1.7).sin() + 0.05 * (x * 3.1 + 1.0).sin()
        })
        .collect();
    scene.water_2d.push(Water2d {
        origin: Vec2::new(-10.0, -1.2),
        width: 20.0,
        depth: 6.0,
        surface,
        layer: 2,
        ..Default::default()
    });
    scene.camera = Camera::orthographic_2d(Vec2::ZERO, 12.0);
    scene.environment.background = Background::Color(Vec3::new(0.05, 0.06, 0.09));
    scene.post.tonemap = Tonemap::None;
    scene
}

/// Particles of a fluid at rest in a basin with a blob falling into it,
/// laid out on a jittered grid (the shape og-physics' og.Fluid2D settles
/// into), as metaballs; a second, orange fluid in a separate pool.
pub fn fluid_2d_particles(seed: u32) -> (Vec<Vec2>, Vec<Vec2>) {
    let jitter = |i: u32| {
        let mut h = i.wrapping_mul(0x9e37_79b9) ^ seed;
        h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
        ((h & 0xffff) as f32 / 65535.0 - 0.5) * 0.06
    };
    let mut water = Vec::new();
    let mut k = 0;
    let spacing = 0.16;
    for row in 0..18 {
        for col in 0..60 {
            let x = -8.0 + col as f32 * spacing + if row % 2 == 1 { spacing * 0.5 } else { 0.0 };
            let y = -4.6 + row as f32 * spacing * 0.87;
            water.push(Vec2::new(x + jitter(k), y + jitter(k + 1)));
            k += 2;
        }
    }
    // The falling blob and a few drops.
    for row in 0..8 {
        for col in 0..8 {
            let x = -3.0 + col as f32 * spacing;
            let y = 0.8 + row as f32 * spacing;
            if (Vec2::new(x, y) - Vec2::new(-2.45, 1.35)).length() < 0.62 {
                water.push(Vec2::new(x + jitter(k), y + jitter(k + 1)));
            }
            k += 2;
        }
    }
    for i in 0..6 {
        water.push(Vec2::new(-1.2 + i as f32 * 0.35, -1.0 + (i as f32 * 1.3).sin() * 0.4));
    }
    let mut lava = Vec::new();
    for row in 0..10 {
        for col in 0..22 {
            let x = 3.0 + col as f32 * spacing;
            let y = -4.6 + row as f32 * spacing * 0.87 + 0.25 * ((col as f32) * 0.3).sin();
            lava.push(Vec2::new(x + jitter(k), y + jitter(k + 1)));
            k += 2;
        }
    }
    (water, lava)
}

pub fn fluid_2d_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let (water, lava) = fluid_2d_particles(7);
    let stone = [0.32, 0.3, 0.34, 1.0];
    for (p, size) in [
        (Vec2::new(-8.3, -2.0), Vec2::new(0.6, 6.0)),
        (Vec2::new(1.8, -2.0), Vec2::new(0.6, 6.0)),
        (Vec2::new(-3.25, -5.2), Vec2::new(10.7, 0.6)),
        (Vec2::new(6.9, -2.0), Vec2::new(0.6, 6.0)),
        (Vec2::new(4.6, -5.2), Vec2::new(5.2, 0.6)),
    ] {
        scene.add_sprite(Sprite {
            position: p.extend(0.0),
            size,
            color: stone,
            layer: 2,
            ..Default::default()
        });
    }
    scene.metaballs.push(Metaballs {
        particles: Arc::from(water),
        radius: 0.26,
        ..Default::default()
    });
    scene.metaballs.push(Metaballs {
        particles: Arc::from(lava),
        radius: 0.28,
        color: glam::Vec4::new(1.4, 0.45, 0.08, 1.0),
        rim: Vec3::new(2.2, 1.5, 0.4),
        layer: 1,
        ..Default::default()
    });
    scene.camera = Camera::orthographic_2d(Vec2::new(-0.6, -1.5), 9.0);
    scene.environment.background = Background::Color(Vec3::new(0.08, 0.08, 0.1));
    scene.post.tonemap = Tonemap::None;
    scene
}

/// The dissolve hooks (`assets/shaders/dissolve.wgsl`).
pub const DISSOLVE_WGSL: &str = include_str!("../assets/shaders/dissolve.wgsl");
/// The grass hooks (`assets/shaders/grass.wgsl`).
pub const GRASS_WGSL: &str = include_str!("../assets/shaders/grass.wgsl");

fn param(name: &str, kind: ParamKind, default: [f32; 4]) -> ShaderParam {
    ShaderParam {
        name: name.into(),
        kind,
        default,
    }
}

/// The dissolve shader: `amount` (0..1), noise `scale`, `edgeWidth`, `edgeColor`.
pub fn dissolve_shader() -> MaterialShader {
    MaterialShader {
        name: "dissolve".into(),
        code: DISSOLVE_WGSL.into(),
        params: vec![
            param("amount", ParamKind::Float, [0.0; 4]),
            param("scale", ParamKind::Float, [3.0, 0.0, 0.0, 0.0]),
            param("edgeWidth", ParamKind::Float, [0.08, 0.0, 0.0, 0.0]),
            param("edgeColor", ParamKind::Vec3, [4.0, 1.6, 0.4, 0.0]),
        ],
    }
}

/// The grass shader: `wind` (xz, metres at the tip), `speed`, `waveScale`,
/// blade `height`, `tip` colour.
pub fn grass_shader() -> MaterialShader {
    MaterialShader {
        name: "grass".into(),
        code: GRASS_WGSL.into(),
        params: vec![
            param("wind", ParamKind::Vec2, [0.22, 0.08, 0.0, 0.0]),
            param("speed", ParamKind::Float, [2.2, 0.0, 0.0, 0.0]),
            param("waveScale", ParamKind::Float, [0.9, 0.0, 0.0, 0.0]),
            param("height", ParamKind::Float, [0.7, 0.0, 0.0, 0.0]),
            param("tip", ParamKind::Vec3, [0.75, 0.85, 0.35, 0.0]),
        ],
    }
}

/// A grass blade: a tapering quad from the root (y 0) to `height`,
/// facing +z, split in three for the bend.
pub fn blade(height: f32, width: f32) -> MeshData {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let rows = 4;
    for r in 0..rows {
        let t = r as f32 / (rows - 1) as f32;
        let w = width * (1.0 - t * 0.85) * 0.5;
        for side in [-1.0f32, 1.0] {
            vertices.push(Vertex {
                position: [side * w, t * height, 0.0],
                normal: [0.0, 0.3, 1.0],
                uv: [side * 0.5 + 0.5, 1.0 - t],
                ..Default::default()
            });
        }
    }
    for r in 0..rows - 1 {
        let a = r * 2;
        indices.extend_from_slice(&[a, a + 1, a + 3, a, a + 3, a + 2]);
    }
    MeshData { vertices, indices }
}

/// A sphere and a cube dissolving by noise (amounts 0.35 and 0.6, the
/// glowing edge), a third sphere with the shader at amount 0, and a patch
/// of 900 grass blades bending in the wind, at `tick`.
pub fn hooks_scene(tick: u64) -> RenderScene {
    hooks_scene_parts(tick).0
}

/// [`hooks_scene`] and the dissolving cube's material.
pub fn hooks_scene_parts(tick: u64) -> (RenderScene, og_render::MaterialHandle) {
    let mut scene = RenderScene::new();
    terrain(&mut scene, 60.0);
    let dissolve = scene.add_shader(dissolve_shader());
    let grass = scene.add_shader(grass_shader());
    let sphere = scene.add_mesh(shapes::sphere(48, 32));
    let cube = scene.add_mesh(shapes::cuboid(Vec3::splat(1.4)));
    let hooked = |scene: &mut RenderScene, color: Vec3, amount: f32| {
        let m = scene.add_standard_material(StandardMaterial {
            color,
            roughness: 0.45,
            alphaTest: 0.5,
            side: 2,
            ..Default::default()
        });
        scene.set_material_shader(m, Some(dissolve));
        scene.set_shader_param(m, "amount", &[amount]);
        m
    };
    let gold = hooked(&mut scene, Vec3::new(0.9, 0.62, 0.2), 0.35);
    let teal = hooked(&mut scene, Vec3::new(0.15, 0.55, 0.6), 0.6);
    let whole = hooked(&mut scene, Vec3::new(0.7, 0.2, 0.25), 0.0);
    scene.add_instance(
        sphere,
        gold,
        Mat4::from_translation(Vec3::new(-2.2, 1.2, -1.0)) * Mat4::from_scale(Vec3::splat(2.0)),
    );
    scene.add_instance(cube, teal, Mat4::from_translation(Vec3::new(0.4, 0.9, -2.5)) * Mat4::from_rotation_y(0.6));
    scene.add_instance(
        sphere,
        whole,
        Mat4::from_translation(Vec3::new(2.6, 0.9, -1.5)) * Mat4::from_scale(Vec3::splat(1.6)),
    );
    let blade_mesh = scene.add_mesh(blade(0.7, 0.07));
    let blade_mat = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.3, 0.55, 0.16),
        roughness: 0.8,
        side: 2,
        ..Default::default()
    });
    scene.set_material_shader(blade_mat, Some(grass));
    let mut k = 0u32;
    let mut rnd = || {
        k = k.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (k >> 8) as f32 / 16_777_216.0
    };
    for i in 0..900 {
        let x = -4.5 + (i % 30) as f32 * 0.3 + (rnd() - 0.5) * 0.25;
        let z = 0.4 + (i / 30) as f32 * 0.12 + (rnd() - 0.5) * 0.1;
        let s = 0.7 + rnd() * 0.6;
        scene.add_instance(
            blade_mesh,
            blade_mat,
            Mat4::from_translation(Vec3::new(x, 0.0, z))
                * Mat4::from_rotation_y((rnd() - 0.5) * 1.6)
                * Mat4::from_scale(Vec3::splat(s)),
        );
    }
    scene.tick = tick;
    scene.camera = Camera {
        position: Vec3::new(0.0, 2.2, 6.5),
        target: Vec3::new(0.0, 0.9, -1.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene.post.bloom = Some(Bloom::default());
    (scene, teal)
}

/// The colonnade in weather `kind` (an og-render `WEATHER_*` preset) at
/// intensity 1: its particles placed over the camera, its fog, its dimmed
/// light. Render [`WEATHER_FRAMES`] frames with [`crate::fx_scenes::tick_update`]
/// so the particles fill the view.
pub fn weather_scene(kind: u32) -> RenderScene {
    let mut scene = day_night_scene(14.0, false);
    let look = og_render::weather(
        kind,
        1.0,
        Vec3::new(2.0, 0.0, 0.6),
        18.0,
        Vec3::ONE,
        1.0,
        None,
    );
    scene.fog = look.fog;
    scene.light.intensity *= look.light;
    scene.environment.intensity *= 0.5 + 0.5 * look.light;
    if let Some(e) = look.emitter.clone() {
        let emitter = scene.add_emitter(e);
        scene.particles.push(og_render::ParticleSystem::new(
            1,
            emitter,
            Mat4::from_translation(look.origin(
                scene.camera.position,
                scene.camera.target - scene.camera.position,
            )),
            7,
            0,
        ));
    }
    scene
}

/// Frames a weather scene runs before its capture.
pub const WEATHER_FRAMES: u32 = 240;
