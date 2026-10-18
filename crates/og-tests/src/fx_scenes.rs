//! The effect scenes of the E1 round (`crates/og-render/README.md`
//! §Effects): particles (GPU and CPU paths), 2D lighting with glow and 2D
//! particles, trails and beams, decals, the grading stack, ambient
//! occlusion, depth of field, god rays, motion blur, the stylised look and
//! GPU skinning. Every texture is generated here, so the scenes need no
//! assets beyond the two glTF models.

use crate::scenes::{ground, load, texture};
use glam::{Mat4, Vec2, Vec3};
use og_render::{
    AmbientOcclusion, AoMethod, Background, Beam, Bloom, Burst, Camera, ChromaticAberration,
    ColorCurve, ColorGrading, ColorLut, ColorSpace, Curve, Decal, DepthOfField, DirectionalLight,
    DofQuality, EmitterHandle, EmitterShape, Environment, Exposure, FilmGrain, Glow, GodRays,
    JointPalette, Light, Light2d, LightKind, Lighting2d, Material, MeshData, Mips, MotionBlur,
    Occluder2d, Outline, ParticleAlign, ParticleBlend, ParticleCollision, ParticleEmitter,
    ParticleNoise, ParticleRender, ParticleSystem, ProceduralSky, RenderScene, Ribbon, RibbonPoint,
    RibbonUv, SheetMode, SimulationSpace, SkinData, SkinnedInstance, Sprite, SpriteFx, SpriteSheet,
    StandardMaterial, Text, TextureData, TextureHandle, Tonemap, Trail, TrailStyle, Vertex,
    Vignette, shapes,
};

/// Advances every clock of the scene to `tick`: the scene's, each particle
/// system's and each trail's.
pub fn tick_all(scene: &mut RenderScene, tick: u64) {
    scene.tick = tick;
    for p in &mut scene.particles {
        p.tick = tick;
    }
    for t in &mut scene.trails {
        t.tick = tick;
    }
}

fn smooth(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn hash2(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841) ^ seed;
    h = (h ^ (h >> 13)).wrapping_mul(0x85eb_ca6b);
    h ^= h >> 16;
    (h & 0xffff) as f32 / 65535.0
}

/// Smooth 2D value noise in 0..1.
fn noise2(x: f32, y: f32, seed: u32) -> f32 {
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - x.floor(), y - y.floor());
    let (ux, uy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let a = hash2(ix, iy, seed);
    let b = hash2(ix + 1, iy, seed);
    let c = hash2(ix, iy + 1, seed);
    let d = hash2(ix + 1, iy + 1, seed);
    let top = a + (b - a) * ux;
    let bottom = c + (d - c) * ux;
    top + (bottom - top) * uy
}

/// Value noise on a lattice that wraps every `period` cells (tiles).
fn noise2_periodic(x: f32, y: f32, period: i32, seed: u32) -> f32 {
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - x.floor(), y - y.floor());
    let (ux, uy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let w = |i: i32| i.rem_euclid(period);
    let a = hash2(w(ix), w(iy), seed);
    let b = hash2(w(ix + 1), w(iy), seed);
    let c = hash2(w(ix), w(iy + 1), seed);
    let d = hash2(w(ix + 1), w(iy + 1), seed);
    let top = a + (b - a) * ux;
    let bottom = c + (d - c) * ux;
    top + (bottom - top) * uy
}

/// Four octaves of tiling noise over the unit square, `period` cells first.
fn tiling_fbm(u: f32, v: f32, period: i32, seed: u32) -> f32 {
    let mut value = 0.0;
    let mut amp = 0.5;
    let mut p = period;
    for o in 0..4 {
        value += noise2_periodic(u * p as f32, v * p as f32, p, seed + o) * amp;
        p *= 2;
        amp *= 0.5;
    }
    value
}

fn fbm(x: f32, y: f32, seed: u32) -> f32 {
    let mut v = 0.0;
    let mut amp = 0.5;
    let mut f = 1.0;
    for o in 0..4 {
        v += noise2(x * f, y * f, seed + o) * amp;
        f *= 2.0;
        amp *= 0.5;
    }
    v
}

/// A ground of packed dirt and patchy grass (tiling every 4 m), instead
/// of the harness's checker: the effect scenes should read as a game.
pub fn terrain(scene: &mut RenderScene, size: f32) {
    let tex = scene.add_texture(texture(512, ColorSpace::Srgb, |u, v| {
        let grass = smooth(0.45, 0.62, tiling_fbm(u, v, 4, 3));
        let detail = tiling_fbm(u, v, 24, 9);
        let pebbles = smooth(0.7, 0.76, tiling_fbm(u, v, 48, 13));
        let dirt = [96.0, 76.0, 56.0];
        let green = [62.0, 84.0, 40.0];
        let mut c = [0.0f32; 3];
        for i in 0..3 {
            c[i] = (dirt[i] + (green[i] - dirt[i]) * grass) * (0.7 + 0.6 * detail);
            c[i] = c[i] * (1.0 - pebbles * 0.35) + 130.0 * pebbles * 0.35;
        }
        [c[0] as u8, c[1] as u8, c[2] as u8, 255]
    }));
    let material = scene.add_material(Material {
        base_color_texture: Some(tex),
        roughness: 0.92,
        ..Default::default()
    });
    let mesh = scene.add_mesh(shapes::plane(size, size / 4.0));
    scene.add_instance(mesh, material, Mat4::IDENTITY);
}

/// A soft, billowing smoke puff (white, alpha-shaped).
pub fn smoke_texture(scene: &mut RenderScene) -> TextureHandle {
    scene.add_texture(texture(64, ColorSpace::Srgb, |u, v| {
        let d = ((u - 0.5).powi(2) + (v - 0.5).powi(2)).sqrt() * 2.0;
        let n = fbm(u * 4.0, v * 4.0, 7);
        let a = (1.0 - smooth(0.35, 1.0, d + (n - 0.5) * 0.5)) * (0.55 + 0.45 * n);
        let shade = (200.0 + 55.0 * n) as u8;
        [shade, shade, shade, (a.clamp(0.0, 1.0) * 255.0) as u8]
    }))
}

/// A 4x4 sheet of flame frames: a teardrop whose flicker drifts per frame.
pub fn flame_sheet(scene: &mut RenderScene) -> TextureHandle {
    scene.add_texture(texture(256, ColorSpace::Srgb, |u, v| {
        let (fu, fv) = ((u * 4.0).fract(), (v * 4.0).fract());
        let frame = (u * 4.0) as u32 + 4 * (v * 4.0) as u32;
        let t = frame as f32 / 16.0;
        let x = (fu - 0.5) * 2.0;
        let y = 1.0 - fv;
        let width = (1.0 - y).powf(0.6) * 0.75 * (1.0 - y * 0.2);
        let wobble = (fbm(fu * 3.0, fv * 3.0 - t * 4.0, 11) - 0.5) * 0.5;
        let inside = 1.0 - smooth(width * 0.6, width + 0.05, (x + wobble * y).abs());
        let a = inside * smooth(0.0, 0.15, y) * (1.0 - smooth(0.75, 1.0, y));
        let heat = (1.0 - y) * inside;
        let r = 255.0;
        let g = 90.0 + 165.0 * heat.powf(1.5);
        let b = 30.0 + 200.0 * heat.powi(4);
        [r as u8, g as u8, b as u8, (a.clamp(0.0, 1.0) * 255.0) as u8]
    }))
}

/// A bright point with a soft halo.
pub fn spark_texture(scene: &mut RenderScene) -> TextureHandle {
    scene.add_texture(texture(32, ColorSpace::Srgb, |u, v| {
        let d = ((u - 0.5).powi(2) + (v - 0.5).powi(2)).sqrt() * 2.0;
        let a = (1.0 - d).clamp(0.0, 1.0).powi(2) + (1.0 - smooth(0.0, 0.25, d)) * 0.5;
        [255, 255, 255, (a.min(1.0) * 255.0) as u8]
    }))
}

/// Energy stripes along u (beams scroll it).
pub fn beam_texture(scene: &mut RenderScene) -> TextureHandle {
    let mut t = texture(128, ColorSpace::Srgb, |u, v| {
        let core = 1.0 - smooth(0.05, 0.45, (v - 0.5).abs());
        let pulse = 0.55 + 0.45 * (u * std::f32::consts::TAU * 3.0).sin().abs();
        let n = fbm(u * 8.0, v * 2.0, 3);
        let a = core * (0.5 + 0.5 * pulse) * (0.7 + 0.3 * n);
        [255, 255, 255, (a.clamp(0.0, 1.0) * 255.0) as u8]
    });
    t.mips = Mips::Generate;
    scene.add_texture(t)
}

/// A 2x2 decal atlas: scorch mark, glowing rune circle, cracks, paint splat.
pub fn decal_atlas(scene: &mut RenderScene) -> TextureHandle {
    scene.add_texture(texture(512, ColorSpace::Srgb, |u, v| {
        let (qu, qv) = ((u * 2.0).fract(), (v * 2.0).fract());
        let q = (u >= 0.5) as u32 + 2 * (v >= 0.5) as u32;
        let (x, y) = (qu - 0.5, qv - 0.5);
        let d = (x * x + y * y).sqrt() * 2.0;
        let n = fbm(qu * 6.0, qv * 6.0, 21 + q);
        match q {
            // Scorch: dark, ragged edge.
            0 => {
                let a = 1.0 - smooth(0.45, 0.95, d + (n - 0.5) * 0.6);
                [18, 14, 12, (a * 235.0) as u8]
            }
            // Rune circle: rings and spokes.
            1 => {
                let ring = (1.0 - smooth(0.0, 0.04, (d - 0.8).abs()))
                    .max(1.0 - smooth(0.0, 0.03, (d - 0.55).abs()));
                let ang = y.atan2(x);
                let spokes = (1.0 - smooth(0.0, 0.08, (ang * 3.0).sin().abs()))
                    * smooth(0.55, 0.6, d)
                    * (1.0 - smooth(0.75, 0.8, d));
                let a = ring.max(spokes);
                [120, 220, 255, (a * 255.0) as u8]
            }
            // Cracks: thin dark lines from a noise field's ridges.
            2 => {
                let r = (fbm(qu * 3.0, qv * 3.0, 5) - 0.5).abs();
                let a = (1.0 - smooth(0.0, 0.035, r)) * (1.0 - smooth(0.6, 1.0, d));
                [25, 22, 20, (a * 240.0) as u8]
            }
            // Paint splat.
            _ => {
                let a = 1.0 - smooth(0.5, 0.65, d + (n - 0.5) * 0.7);
                [220, 40, 60, (a * 255.0) as u8]
            }
        }
    }))
}

/// The four quadrants of [`decal_atlas`].
pub const DECAL_SCORCH: [f32; 4] = [0.0, 0.0, 0.5, 0.5];
pub const DECAL_RUNE: [f32; 4] = [0.5, 0.0, 1.0, 0.5];
pub const DECAL_CRACK: [f32; 4] = [0.0, 0.5, 0.5, 1.0];
pub const DECAL_PAINT: [f32; 4] = [0.5, 0.5, 1.0, 1.0];

/// A campfire's emitters: flames (a sprite sheet), smoke (alpha, soft) and
/// embers (velocity-stretched, bouncing off the depth buffer).
pub struct Campfire {
    pub flames: EmitterHandle,
    pub smoke: EmitterHandle,
    pub embers: EmitterHandle,
}

pub fn campfire(scene: &mut RenderScene) -> Campfire {
    let flame = flame_sheet(scene);
    let smoke_tex = smoke_texture(scene);
    let spark = spark_texture(scene);
    let flames = scene.add_emitter(ParticleEmitter {
        max_particles: 160,
        shape: EmitterShape::Cone {
            angle_degrees: 12.0,
            radius: 0.28,
        },
        rate: 70.0,
        lifetime: [0.55, 0.9],
        speed: [0.6, 1.1],
        size: [0.55, 0.85],
        size_over_life: Curve::new(&[(0.0, 0.6), (0.3, 1.0), (1.0, 0.35)]),
        color_over_life: ColorCurve::new(&[
            (0.0, [1.0, 1.0, 1.0, 0.0]),
            (0.12, [1.0, 1.0, 1.0, 1.0]),
            (0.7, [1.0, 0.8, 0.6, 0.8]),
            (1.0, [0.6, 0.3, 0.2, 0.0]),
        ]),
        color: [1.5, 1.05, 0.8, 1.0],
        rotation_degrees: [-12.0, 12.0],
        gravity: Vec3::new(0.0, 1.2, 0.0),
        drag: 0.8,
        noise: ParticleNoise {
            strength: 1.2,
            frequency: 1.5,
            scroll: 0.8,
        },
        texture: Some(flame),
        sheet: SpriteSheet {
            columns: 4,
            rows: 4,
            frames: 16,
            mode: SheetMode::OverLife { cycles: 1.0 },
        },
        blend: ParticleBlend::Additive,
        soft_distance: 0.3,
        ..Default::default()
    });
    let smoke = scene.add_emitter(ParticleEmitter {
        max_particles: 120,
        shape: EmitterShape::Cone {
            angle_degrees: 10.0,
            radius: 0.2,
        },
        rate: 22.0,
        lifetime: [2.6, 3.6],
        speed: [0.6, 0.9],
        size: [0.5, 0.8],
        size_over_life: Curve::new(&[(0.0, 0.5), (1.0, 2.6)]),
        color_over_life: ColorCurve::new(&[
            (0.0, [0.06, 0.05, 0.05, 0.0]),
            (0.15, [0.07, 0.06, 0.06, 0.55]),
            (1.0, [0.12, 0.12, 0.13, 0.0]),
        ]),
        color: [1.0; 4],
        rotation_degrees: [0.0, 360.0],
        angular_velocity_degrees: [-30.0, 30.0],
        gravity: Vec3::new(0.25, 0.35, 0.0),
        drag: 0.4,
        noise: ParticleNoise {
            strength: 0.6,
            frequency: 0.6,
            scroll: 0.3,
        },
        texture: Some(smoke_tex),
        blend: ParticleBlend::Alpha,
        soft_distance: 0.6,
        ..Default::default()
    });
    let embers = scene.add_emitter(ParticleEmitter {
        max_particles: 200,
        shape: EmitterShape::Sphere {
            radius: 0.25,
            surface: false,
        },
        spread_degrees: 35.0,
        rate: 40.0,
        bursts: vec![Burst {
            time: 0.0,
            count: 30,
            cycles: 1,
            interval: 0.0,
        }],
        lifetime: [1.2, 2.2],
        speed: [1.8, 3.4],
        size: [0.035, 0.06],
        size_over_life: Curve::linear(1.0, 0.4),
        color_over_life: ColorCurve::new(&[
            (0.0, [1.0, 0.9, 0.6, 1.0]),
            (0.6, [1.0, 0.5, 0.2, 1.0]),
            (1.0, [0.8, 0.2, 0.1, 0.0]),
        ]),
        color: [8.0, 4.0, 1.5, 1.0],
        gravity: Vec3::new(0.0, -4.0, 0.0),
        drag: 0.3,
        noise: ParticleNoise {
            strength: 2.0,
            frequency: 1.2,
            scroll: 1.0,
        },
        texture: Some(spark),
        blend: ParticleBlend::Additive,
        align: ParticleAlign::Velocity { stretch: 0.04 },
        collision: Some(ParticleCollision {
            bounce: 0.35,
            friction: 0.3,
        }),
        ..Default::default()
    });
    Campfire {
        flames,
        smoke,
        embers,
    }
}

/// Places a campfire's systems at `at` (ids from `id`).
pub fn light_campfire(scene: &mut RenderScene, fire: &Campfire, at: Vec3, id: u64) {
    let t = Mat4::from_translation(at);
    for (k, e) in [fire.smoke, fire.flames, fire.embers]
        .into_iter()
        .enumerate()
    {
        let lift = if k == 0 { 0.4 } else { 0.05 };
        scene.particles.push(ParticleSystem::new(
            id + k as u64,
            e,
            Mat4::from_translation(Vec3::Y * lift) * t,
            0x5eed + id + k as u64,
            0,
        ));
    }
}

fn logs(scene: &mut RenderScene, at: Vec3) {
    let log = scene.add_mesh(shapes::cuboid(Vec3::new(0.9, 0.12, 0.12)));
    let wood = scene.add_material(Material {
        base_color: [0.16, 0.09, 0.05, 1.0],
        roughness: 0.9,
        emissive: [0.25, 0.06, 0.0],
        ..Default::default()
    });
    for i in 0..4 {
        let a = i as f32 * std::f32::consts::FRAC_PI_2 + 0.4;
        scene.add_instance(
            log,
            wood,
            Mat4::from_translation(at + Vec3::new(0.0, 0.08, 0.0))
                * Mat4::from_rotation_y(a)
                * Mat4::from_rotation_z(0.25),
        );
    }
    let stone = scene.add_mesh(shapes::sphere(10, 8));
    let rock = scene.add_material(Material {
        base_color: [0.35, 0.33, 0.31, 1.0],
        roughness: 0.85,
        ..Default::default()
    });
    for i in 0..9 {
        let a = i as f32 / 9.0 * std::f32::consts::TAU;
        scene.add_instance(
            stone,
            rock,
            Mat4::from_translation(at + Vec3::new(a.cos() * 0.62, 0.05, a.sin() * 0.62))
                * Mat4::from_scale(Vec3::new(0.26, 0.17, 0.22)),
        );
    }
}

fn night(scene: &mut RenderScene) {
    let moon = Vec3::new(-0.3, 0.6, -0.6);
    scene.light = DirectionalLight {
        direction_to_light: moon,
        color: Vec3::new(0.5, 0.6, 1.0),
        intensity: 0.25,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: moon,
            sun_color: Vec3::new(0.3, 0.35, 0.5),
            look: None,
        },
        intensity: 0.06,
        background: Background::Color(Vec3::new(0.008, 0.01, 0.022)),
    };
}

/// Night camp: a campfire (flames, smoke, embers bouncing on the ground), a
/// magic fountain (local space, noise), sparkles on a sphere's surface (a
/// mesh emitter) and a spark burst; bloom and exposure.
pub fn particles_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    terrain(&mut scene, 40.0);
    night(&mut scene);
    let fire = campfire(&mut scene);
    logs(&mut scene, Vec3::ZERO);
    light_campfire(&mut scene, &fire, Vec3::ZERO, 10);
    // A rune stone with a fountain of blue magic above it.
    let pillar = scene.add_mesh(shapes::cuboid(Vec3::new(0.5, 1.4, 0.5)));
    let stone = scene.add_material(Material {
        base_color: [0.4, 0.4, 0.45, 1.0],
        roughness: 0.7,
        ..Default::default()
    });
    scene.add_instance(
        pillar,
        stone,
        Mat4::from_translation(Vec3::new(-2.6, 0.7, -1.4)),
    );
    let spark = spark_texture(&mut scene);
    let magic = scene.add_emitter(ParticleEmitter {
        max_particles: 300,
        shape: EmitterShape::Cone {
            angle_degrees: 18.0,
            radius: 0.1,
        },
        rate: 120.0,
        lifetime: [1.0, 1.6],
        speed: [2.6, 3.2],
        size: [0.05, 0.09],
        color_over_life: ColorCurve::new(&[
            (0.0, [0.6, 0.9, 1.0, 1.0]),
            (0.7, [0.4, 0.5, 1.0, 1.0]),
            (1.0, [0.6, 0.2, 1.0, 0.0]),
        ]),
        color: [2.0, 3.5, 6.0, 1.0],
        gravity: Vec3::new(0.0, -4.5, 0.0),
        noise: ParticleNoise {
            strength: 1.5,
            frequency: 2.0,
            scroll: 0.5,
        },
        texture: Some(spark),
        space: SimulationSpace::Local,
        ..Default::default()
    });
    scene.particles.push(ParticleSystem::new(
        20,
        magic,
        Mat4::from_translation(Vec3::new(-2.6, 1.4, -1.4)),
        77,
        0,
    ));
    // Sparkles over an orb's surface.
    let orb_mesh = shapes::sphere(24, 16);
    let orb = scene.add_mesh(orb_mesh);
    let glass = scene.add_material(Material {
        base_color: [0.1, 0.05, 0.15, 1.0],
        metallic: 0.2,
        roughness: 0.15,
        emissive: [0.4, 0.1, 0.6],
        ..Default::default()
    });
    let orb_at =
        Mat4::from_translation(Vec3::new(2.4, 0.9, -1.0)) * Mat4::from_scale(Vec3::splat(1.2));
    scene.add_instance(orb, glass, orb_at);
    let sparkle = scene.add_emitter(ParticleEmitter {
        max_particles: 160,
        shape: EmitterShape::Mesh { mesh: orb },
        rate: 90.0,
        lifetime: [0.6, 1.2],
        speed: [0.15, 0.4],
        size: [0.03, 0.06],
        size_over_life: Curve::new(&[(0.0, 0.0), (0.2, 1.0), (1.0, 0.0)]),
        color: [6.0, 2.5, 8.0, 1.0],
        gravity: Vec3::ZERO,
        texture: Some(spark),
        ..Default::default()
    });
    scene
        .particles
        .push(ParticleSystem::new(30, sparkle, orb_at, 99, 0));
    // A spark burst every second against the pillar's base.
    let burst = scene.add_emitter(ParticleEmitter {
        max_particles: 120,
        shape: EmitterShape::Point,
        spread_degrees: 70.0,
        rate: 0.0,
        bursts: vec![Burst {
            time: 0.0,
            count: 60,
            cycles: 1,
            interval: 0.0,
        }],
        duration: 1.0,
        lifetime: [0.5, 0.9],
        speed: [3.0, 5.0],
        size: [0.03, 0.05],
        color_over_life: ColorCurve::new(&[
            (0.0, [1.0, 0.95, 0.7, 1.0]),
            (1.0, [1.0, 0.4, 0.1, 0.0]),
        ]),
        color: [10.0, 7.0, 3.0, 1.0],
        gravity: Vec3::new(0.0, -9.0, 0.0),
        texture: Some(spark),
        align: ParticleAlign::Velocity { stretch: 0.05 },
        collision: Some(ParticleCollision::default()),
        ..Default::default()
    });
    scene.particles.push(ParticleSystem::new(
        40,
        burst,
        Mat4::from_translation(Vec3::new(0.9, 0.6, 1.6)) * Mat4::from_rotation_z(-0.6),
        5,
        0,
    ));
    scene.lights = vec![
        Light {
            position: Vec3::new(0.0, 0.7, 0.0),
            color: Vec3::new(1.0, 0.55, 0.25),
            intensity: 9.0,
            range: 10.0,
            cast_shadows: true,
            ..Default::default()
        },
        Light {
            position: Vec3::new(-2.6, 2.2, -1.4),
            color: Vec3::new(0.3, 0.5, 1.0),
            intensity: 3.0,
            range: 6.0,
            ..Default::default()
        },
    ];
    scene.post.bloom = Some(Bloom {
        threshold: 1.0,
        knee: 0.6,
        intensity: 0.7,
    });
    scene.camera = Camera {
        position: Vec3::new(0.4, 2.6, 6.5),
        target: Vec3::new(0.0, 0.9, -0.5),
        exposure: Exposure::linear(1.4),
        ..Default::default()
    };
    scene
}

/// Ticks the scene's clocks to the frame (60 ticks per second).
pub fn tick_update(scene: &mut RenderScene, frame: u32) {
    tick_all(scene, frame as u64);
}

// ---------------------------------------------------------------------------
// 2D: lights and soft shadows from sprite occluders, glow, 2D particles,
// a sprite hit flash.
// ---------------------------------------------------------------------------

pub(crate) fn tile_texture(scene: &mut RenderScene) -> TextureHandle {
    scene.add_texture(texture(64, ColorSpace::Srgb, |u, v| {
        let edge = (u.min(1.0 - u).min(v).min(1.0 - v) * 16.0).min(1.0);
        let n = fbm(u * 5.0, v * 5.0, 2);
        let c = (70.0 + 50.0 * n) * (0.6 + 0.4 * edge);
        [c as u8, (c * 0.95) as u8, (c * 0.9) as u8, 255]
    }))
}

pub(crate) fn crate_texture(scene: &mut RenderScene) -> TextureHandle {
    scene.add_texture(texture(64, ColorSpace::Srgb, |u, v| {
        let border = u.min(1.0 - u).min(v).min(1.0 - v) < 0.1;
        let diag = ((u - v).abs() < 0.08) || ((u + v - 1.0).abs() < 0.08);
        let n = fbm(u * 3.0, v * 9.0, 4);
        let (r, g, b) = if border || diag {
            (120.0, 80.0, 40.0)
        } else {
            (170.0 + 40.0 * n, 120.0 + 30.0 * n, 60.0)
        };
        [r as u8, g as u8, b as u8, 255]
    }))
}

fn hero_texture(scene: &mut RenderScene) -> TextureHandle {
    scene.add_texture(texture(64, ColorSpace::Srgb, |u, v| {
        // A round-bodied character with eyes, v down.
        let body = ((u - 0.5).powi(2) / 0.16 + (v - 0.6).powi(2) / 0.13) < 1.0;
        let head = ((u - 0.5).powi(2) + (v - 0.28).powi(2)).sqrt() < 0.2;
        let eye = ((u - 0.43).powi(2) + (v - 0.26).powi(2)).sqrt() < 0.035
            || ((u - 0.57).powi(2) + (v - 0.26).powi(2)).sqrt() < 0.035;
        if eye {
            [20, 20, 30, 255]
        } else if head {
            [250, 210, 170, 255]
        } else if body {
            [60, 120, 230, 255]
        } else {
            [0, 0, 0, 0]
        }
    }))
}

fn coin_texture(scene: &mut RenderScene) -> TextureHandle {
    scene.add_texture(texture(32, ColorSpace::Srgb, |u, v| {
        let d = ((u - 0.5).powi(2) + (v - 0.5).powi(2)).sqrt() * 2.0;
        let a = 1.0 - smooth(0.85, 1.0, d);
        let shine = 1.0
            - smooth(
                0.0,
                0.5,
                ((u - 0.35).powi(2) + (v - 0.35).powi(2)).sqrt() * 3.0,
            );
        [
            255,
            (200.0 + 55.0 * shine) as u8,
            (60.0 + 150.0 * shine) as u8,
            (a * 255.0) as u8,
        ]
    }))
}

/// A night dungeon room in 2D: a torch (2D fire particles that glow) and a
/// cold magic light cast soft shadows from crates, a hit-flashed hero, glowing
/// coins (order-independent glow, one of them under a crate), and an unlit UI
/// label on a layer above the lit range.
pub fn fx_2d_scene() -> Result<RenderScene, String> {
    let mut scene = RenderScene::new();
    let tile = tile_texture(&mut scene);
    let crate_tex = crate_texture(&mut scene);
    let hero = hero_texture(&mut scene);
    let coin = coin_texture(&mut scene);
    let flame = flame_sheet(&mut scene);
    for y in -7..7 {
        for x in -12..12 {
            scene.add_sprite(Sprite {
                texture: Some(tile),
                position: Vec3::new(x as f32 + 0.5, y as f32 + 0.5, 0.0),
                size: Vec2::ONE,
                layer: 0,
                ..Default::default()
            });
        }
    }
    let crates = [
        (Vec2::new(-3.0, 0.5), 1.2, 0.0),
        (Vec2::new(-1.6, -1.8), 1.0, 0.3),
        (Vec2::new(2.2, 1.6), 1.4, -0.2),
        (Vec2::new(4.0, -1.2), 0.9, 0.5),
        (Vec2::new(0.6, 2.8), 0.8, 0.1),
    ];
    for (p, s, r) in crates {
        let sprite = Sprite {
            texture: Some(crate_tex),
            position: p.extend(0.0),
            size: Vec2::splat(s),
            rotation: r,
            layer: 2,
            ..Default::default()
        };
        scene
            .occluders_2d
            .push(Occluder2d::from_sprite(&sprite, 0.02));
        scene.add_sprite(sprite);
    }
    // The hero, hit: a red flash.
    let hero_index = scene.sprites.len() as u32;
    scene.add_sprite(Sprite {
        texture: Some(hero),
        position: Vec3::new(0.8, -0.6, 0.0),
        size: Vec2::splat(1.3),
        layer: 3,
        ..Default::default()
    });
    scene.sprite_fx.push(SpriteFx {
        sprite: hero_index,
        flash: [1.0, 1.0, 1.0, 0.6],
        ..Default::default()
    });
    // Glowing coins (the last one under a crate: the glow still shows).
    for (i, p) in [
        Vec2::new(-4.6, -2.6),
        Vec2::new(-4.0, -2.9),
        Vec2::new(5.5, 2.4),
        Vec2::new(3.4, -3.2),
        Vec2::new(2.2, 1.6),
    ]
    .iter()
    .enumerate()
    {
        let index = scene.sprites.len() as u32;
        scene.add_sprite(Sprite {
            texture: Some(coin),
            position: p.extend(0.0),
            size: Vec2::splat(0.45),
            color: [1.4, 1.2, 0.6, 1.0],
            layer: 1,
            ..Default::default()
        });
        scene.sprite_fx.push(SpriteFx {
            sprite: index,
            glow: 1.5 + i as f32 * 0.2,
            ..Default::default()
        });
    }
    // The torch: 2D flames that glow, in the sprite batch.
    let torch = scene.add_emitter(ParticleEmitter {
        max_particles: 120,
        shape: EmitterShape::Cone {
            angle_degrees: 10.0,
            radius: 0.12,
        },
        rate: 60.0,
        lifetime: [0.5, 0.8],
        speed: [0.6, 1.0],
        size: [0.5, 0.7],
        size_over_life: Curve::new(&[(0.0, 0.7), (0.3, 1.0), (1.0, 0.3)]),
        color_over_life: ColorCurve::new(&[
            (0.0, [1.0, 1.0, 1.0, 0.0]),
            (0.15, [1.0, 1.0, 1.0, 1.0]),
            (1.0, [1.0, 0.5, 0.2, 0.0]),
        ]),
        color: [1.6, 1.3, 1.0, 1.0],
        gravity: Vec3::new(0.0, 1.0, 0.0),
        noise: ParticleNoise {
            strength: 1.0,
            frequency: 2.0,
            scroll: 1.0,
        },
        texture: Some(flame),
        sheet: SpriteSheet {
            columns: 4,
            rows: 4,
            frames: 16,
            mode: SheetMode::OverLife { cycles: 1.0 },
        },
        render: ParticleRender::Sprite { layer: 4 },
        glow: 0.8,
        ..Default::default()
    });
    scene.particles.push(ParticleSystem::new(
        1,
        torch,
        Mat4::from_translation(Vec3::new(-5.5, 2.2, 0.0)),
        3,
        0,
    ));
    // Magic motes (untextured: soft dots), additive.
    let motes = scene.add_emitter(ParticleEmitter {
        max_particles: 80,
        shape: EmitterShape::Sphere {
            radius: 0.6,
            surface: false,
        },
        rate: 30.0,
        lifetime: [1.5, 2.5],
        speed: [0.1, 0.3],
        size: [0.08, 0.16],
        size_over_life: Curve::new(&[(0.0, 0.0), (0.3, 1.0), (1.0, 0.0)]),
        color: [0.6, 0.9, 1.6, 1.0],
        gravity: Vec3::new(0.0, 0.2, 0.0),
        render: ParticleRender::Sprite { layer: 4 },
        glow: 1.0,
        ..Default::default()
    });
    scene.particles.push(ParticleSystem::new(
        2,
        motes,
        Mat4::from_translation(Vec3::new(3.6, -3.4, 0.0)),
        4,
        0,
    ));
    scene.lighting_2d = Some(Lighting2d {
        ambient: Vec3::new(0.06, 0.06, 0.1),
        min_layer: 0,
        max_layer: 50,
        resolution: 0.5,
    });
    scene.lights_2d = vec![
        Light2d {
            position: Vec2::new(-5.5, 2.4),
            color: Vec3::new(1.0, 0.65, 0.35),
            intensity: 2.2,
            radius: 9.0,
            falloff: 1.6,
            source_radius: 0.3,
            cast_shadows: true,
        },
        Light2d {
            position: Vec2::new(3.6, -3.2),
            color: Vec3::new(0.35, 0.6, 1.0),
            intensity: 1.6,
            radius: 6.5,
            falloff: 1.4,
            source_radius: 0.5,
            cast_shadows: true,
        },
        Light2d {
            position: Vec2::new(0.8, -0.2),
            color: Vec3::new(0.9, 0.9, 1.0),
            intensity: 0.5,
            radius: 2.2,
            falloff: 2.0,
            source_radius: 0.1,
            cast_shadows: false,
        },
    ];
    let font = crate::scenes::load_font(&mut scene)?;
    scene.texts.push(Text {
        position: Vec3::new(-9.6, 5.2, 0.0),
        size: 0.6,
        color: [1.0, 0.95, 0.8, 1.0],
        layer: 100,
        ..Text::new(font, "Dungeon 1   coins x3")
    });
    scene.post.glow = Some(Glow { intensity: 1.0 });
    scene.camera = Camera::orthographic_2d(Vec2::ZERO, 12.0);
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.post.tonemap = Tonemap::None;
    Ok(scene)
}

// ---------------------------------------------------------------------------
// Trails, ribbons, beams.
// ---------------------------------------------------------------------------

/// Where wisp `i` is at `tick`.
fn wisp_at(i: usize, tick: u64) -> Vec3 {
    let t = tick as f32 / 60.0;
    let a = t * (1.6 + i as f32 * 0.35) + i as f32 * 2.1;
    let r = 1.6 + i as f32 * 0.5;
    Vec3::new(
        a.cos() * r,
        1.1 + (t * 2.0 + i as f32).sin() * 0.5,
        a.sin() * r,
    )
}

/// Orbiting wisps with trails, a spiral ribbon, a scrolling laser between
/// two pylons and a jittering lightning arc.
pub fn trails_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    terrain(&mut scene, 40.0);
    night(&mut scene);
    let beam_tex = beam_texture(&mut scene);
    let orb = scene.add_mesh(shapes::sphere(16, 12));
    let colors = [[1.0, 0.4, 0.15], [0.2, 0.8, 1.0], [0.6, 1.0, 0.3]];
    for (i, c) in colors.iter().enumerate() {
        let style = scene.add_trail_style(TrailStyle {
            lifetime: 0.6,
            min_distance: 0.02,
            width: Curve::new(&[(0.0, 0.22), (1.0, 0.0)]),
            color: ColorCurve::new(&[
                (0.0, [c[0] * 4.0, c[1] * 4.0, c[2] * 4.0, 1.0]),
                (1.0, [c[0], c[1], c[2], 0.0]),
            ]),
            ..Default::default()
        });
        scene.trails.push(Trail {
            id: i as u64,
            style,
            position: wisp_at(i, 0),
            tick: 0,
        });
        let m = scene.add_material(Material {
            base_color: [0.0, 0.0, 0.0, 1.0],
            emissive: [c[0] * 6.0, c[1] * 6.0, c[2] * 6.0],
            ..Default::default()
        });
        scene.add_instance(orb, m, Mat4::from_scale(Vec3::splat(0.18)));
    }
    // A spiral ribbon, widening.
    let points = (0..60)
        .map(|k| {
            let f = k as f32 / 59.0;
            let a = f * std::f32::consts::TAU * 2.0;
            RibbonPoint {
                position: Vec3::new(-3.6 + a.cos() * 0.5, 0.2 + f * 2.8, -2.0 + a.sin() * 0.5),
                width: 0.05 + 0.2 * f,
                color: [1.5 - f, 0.6, 0.4 + f * 1.4, 0.9],
            }
        })
        .collect();
    scene.ribbons.push(Ribbon {
        points,
        texture: None,
        uv: RibbonUv::Stretch,
        blend: ParticleBlend::Additive,
    });
    // Pylons, a laser and a lightning arc.
    let pylon = scene.add_mesh(shapes::cuboid(Vec3::new(0.3, 2.4, 0.3)));
    let metal = scene.add_material(Material {
        base_color: [0.6, 0.62, 0.66, 1.0],
        metallic: 0.8,
        roughness: 0.35,
        ..Default::default()
    });
    for x in [2.2f32, 4.6] {
        scene.add_instance(
            pylon,
            metal,
            Mat4::from_translation(Vec3::new(x, 1.2, -2.5)),
        );
    }
    scene.beams.push(Beam {
        start: Vec3::new(2.2, 2.0, -2.5),
        end: Vec3::new(4.6, 2.0, -2.5),
        width: 0.22,
        color: [0.5, 1.4, 2.8, 1.0],
        texture: Some(beam_tex),
        tile: 1.2,
        scroll: 2.0,
        ..Default::default()
    });
    scene.beams.push(Beam {
        start: Vec3::new(2.2, 0.9, -2.5),
        end: Vec3::new(4.6, 0.9, -2.5),
        width: 0.06,
        color: [6.0, 5.0, 9.0, 1.0],
        segments: 24,
        jitter: 0.18,
        seed: 9,
        ..Default::default()
    });
    scene.post.bloom = Some(Bloom::default());
    scene.camera = Camera {
        position: Vec3::new(0.5, 3.0, 7.0),
        target: Vec3::new(0.3, 1.0, -0.8),
        exposure: Exposure::linear(1.6),
        ..Default::default()
    };
    scene
}

/// Moves the wisps (and their orbs) to the frame's tick.
pub fn move_wisps(scene: &mut RenderScene, frame: u32) {
    let tick = frame as u64;
    tick_all(scene, tick);
    let n = scene.trails.len();
    for i in 0..n {
        let p = wisp_at(i, tick);
        scene.trails[i].position = p;
        // The orbs are the last three instances before the ribbon's pylons.
        let k = 1 + i;
        scene.instances[k].transform =
            Mat4::from_translation(p) * Mat4::from_scale(Vec3::splat(0.18));
    }
}

// ---------------------------------------------------------------------------
// Decals.
// ---------------------------------------------------------------------------

/// A courtyard with scorch marks, a glowing rune circle, cracks up a wall
/// and a paint splat over the corner where wall meets ground.
pub fn decals_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    terrain(&mut scene, 40.0);
    let atlas = decal_atlas(&mut scene);
    scene.decal_atlas = Some(atlas);
    let wall = scene.add_mesh(shapes::cuboid(Vec3::new(8.0, 3.0, 0.4)));
    let plaster = scene.add_material(Material {
        base_color: [0.82, 0.78, 0.7, 1.0],
        roughness: 0.9,
        ..Default::default()
    });
    scene.add_instance(
        wall,
        plaster,
        Mat4::from_translation(Vec3::new(0.0, 1.5, -2.2)),
    );
    let barrel = scene.add_mesh(shapes::cuboid(Vec3::new(0.6, 0.9, 0.6)));
    scene.add_instance(
        barrel,
        plaster,
        Mat4::from_translation(Vec3::new(2.4, 0.45, 0.2)),
    );
    let up = Vec3::Y;
    scene.decals = vec![
        Decal {
            uv_rect: DECAL_SCORCH,
            ..Decal::on_surface(Vec3::new(-1.6, 0.0, 0.6), up, Vec2::splat(2.4), 0.6, 0.3)
        },
        Decal {
            uv_rect: DECAL_SCORCH,
            ..Decal::on_surface(Vec3::new(2.2, 0.0, 0.6), up, Vec2::splat(1.8), 0.6, 1.4)
        },
        Decal {
            uv_rect: DECAL_RUNE,
            color: [1.0, 1.0, 1.0, 1.0],
            emissive: Vec3::new(1.5, 3.5, 5.0),
            roughness: 0.2,
            ..Decal::on_surface(Vec3::new(0.2, 0.0, 1.6), up, Vec2::splat(2.0), 0.4, 0.0)
        },
        Decal {
            uv_rect: DECAL_CRACK,
            ..Decal::on_surface(
                Vec3::new(-2.0, 1.6, -2.0),
                Vec3::Z,
                Vec2::new(2.2, 2.6),
                0.6,
                0.2,
            )
        },
        // Projected diagonally into the corner: it covers wall and ground.
        Decal {
            uv_rect: DECAL_PAINT,
            angle_fade_degrees: 80.0,
            ..Decal::on_surface(
                Vec3::new(1.0, 0.05, -1.8),
                Vec3::new(0.0, 1.0, 1.0),
                Vec2::splat(1.6),
                1.4,
                0.5,
            )
        },
    ];
    let sun = Vec3::new(0.5, 0.75, 0.6);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        ..Default::default()
    };
    scene.environment.sky.sun_direction = sun;
    scene.post.bloom = Some(Bloom::default());
    scene.camera = Camera {
        position: Vec3::new(0.0, 4.2, 6.2),
        target: Vec3::new(0.0, 0.4, -0.6),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene
}

// ---------------------------------------------------------------------------
// The post stack: grading, vignette, chromatic aberration, grain, flash.
// ---------------------------------------------------------------------------

/// A teal-and-orange grade with lifted blacks: shadows towards teal,
/// highlights towards orange, a little more contrast.
pub fn teal_orange() -> ColorLut {
    ColorLut::from_fn(|c| {
        let l = c.dot(Vec3::new(0.2126, 0.7152, 0.0722));
        let shadow = Vec3::new(-0.02, 0.03, 0.06) * (1.0 - l);
        let high = Vec3::new(0.06, 0.02, -0.05) * l;
        let s = |x: f32| x * x * (3.0 - 2.0 * x);
        let graded = c + shadow + high;
        Vec3::new(s(graded.x), s(graded.y), s(graded.z)) * 0.92 + Vec3::splat(0.03)
    })
}

pub fn grade(scene: &mut RenderScene) {
    let lut = scene.add_lut(teal_orange());
    scene.post.color_grading = Some(ColorGrading {
        lut,
        intensity: 1.0,
    });
    scene.post.vignette = Some(Vignette::default());
    scene.post.chromatic_aberration = Some(ChromaticAberration { intensity: 2.5 });
    scene.post.film_grain = Some(FilmGrain::default());
}

// ---------------------------------------------------------------------------
// Ambient occlusion.
// ---------------------------------------------------------------------------

/// Stacked blocks, steps, a sphere on the ground and a wall corner under a
/// bright overcast sky: contact shadows only the ambient occlusion makes.
pub fn ao_scene(method: AoMethod) -> RenderScene {
    let mut scene = RenderScene::new();
    ground(&mut scene, 30.0, 1.0, 0.9);
    let cube = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    let sphere = scene.add_mesh(shapes::sphere(32, 24));
    let white = scene.add_material(Material {
        base_color: [0.8, 0.78, 0.74, 1.0],
        roughness: 0.85,
        ..Default::default()
    });
    let place = |s: &mut RenderScene, m, p: Vec3, size: Vec3| {
        s.add_instance(m, white, Mat4::from_translation(p) * Mat4::from_scale(size));
    };
    // A wall corner.
    place(
        &mut scene,
        cube,
        Vec3::new(0.0, 1.5, -3.0),
        Vec3::new(8.0, 3.0, 0.4),
    );
    place(
        &mut scene,
        cube,
        Vec3::new(-3.8, 1.5, -1.0),
        Vec3::new(0.4, 3.0, 4.0),
    );
    // Steps.
    for i in 0..4 {
        let h = 0.25 * (i + 1) as f32;
        place(
            &mut scene,
            cube,
            Vec3::new(2.2, h / 2.0, -2.4 + i as f32 * 0.0)
                + Vec3::new(0.0, 0.0, 0.5 * i as f32 - 0.5),
            Vec3::new(2.0, h, 0.5),
        );
    }
    // Blocks stacked and leaning.
    place(
        &mut scene,
        cube,
        Vec3::new(-1.6, 0.4, -0.6),
        Vec3::splat(0.8),
    );
    place(
        &mut scene,
        cube,
        Vec3::new(-1.5, 1.1, -0.6),
        Vec3::splat(0.6),
    );
    place(
        &mut scene,
        sphere,
        Vec3::new(0.3, 0.5, 0.2),
        Vec3::splat(1.0),
    );
    place(
        &mut scene,
        sphere,
        Vec3::new(-0.6, 0.25, 1.0),
        Vec3::splat(0.5),
    );
    let sun = Vec3::new(0.3, 0.9, 0.5);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 0.6,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            sun_color: Vec3::new(1.0, 0.98, 0.95),
            look: None,
        },
        intensity: 1.0,
        background: Background::Sky,
    };
    scene.post.ao = Some(AmbientOcclusion {
        radius: 0.7,
        intensity: 1.2,
        method,
    });
    scene.camera = Camera {
        position: Vec3::new(1.0, 2.6, 5.0),
        target: Vec3::new(-0.2, 0.6, -1.2),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene
}

// ---------------------------------------------------------------------------
// Depth of field.
// ---------------------------------------------------------------------------

/// A row of tricksters receding into the dark with small bright lamps
/// behind (bokeh) and one in front; focus on the middle figure.
pub fn dof_scene(quality: DofQuality) -> Result<RenderScene, String> {
    let mut scene = RenderScene::new();
    ground(&mut scene, 60.0, 0.5, 0.7);
    let model = load("trickster.glb")?;
    let parts = model.add_to(&mut scene);
    let scale = 1.8 / (model.max - model.min).max_element().max(1e-3);
    for i in 0..6 {
        let z = 2.0 - i as f32 * 2.6;
        let x = -2.0 + i as f32 * 0.8;
        parts.instance(
            &mut scene,
            Mat4::from_translation(Vec3::new(x, -model.min.y * scale, z))
                * Mat4::from_rotation_y(0.3 - i as f32 * 0.15)
                * Mat4::from_scale(Vec3::splat(scale)),
        );
    }
    let lamp = scene.add_mesh(shapes::sphere(12, 8));
    for i in 0..18 {
        let c = [[8.0, 5.0, 2.0], [2.0, 5.0, 9.0], [7.0, 2.0, 6.0]][i % 3];
        let m = scene.add_material(Material {
            base_color: [0.0, 0.0, 0.0, 1.0],
            emissive: c,
            ..Default::default()
        });
        let x = -6.0 + (i as f32 * 0.71) % 12.0;
        let y = 0.6 + (i as f32 * 1.37) % 2.8;
        scene.add_instance(
            lamp,
            m,
            Mat4::from_translation(Vec3::new(x, y, -16.0 - (i % 4) as f32 * 2.0))
                * Mat4::from_scale(Vec3::splat(0.12)),
        );
    }
    let m = scene.add_material(Material {
        base_color: [0.0, 0.0, 0.0, 1.0],
        emissive: [9.0, 6.0, 2.0],
        ..Default::default()
    });
    scene.add_instance(
        lamp,
        m,
        Mat4::from_translation(Vec3::new(1.2, 1.4, 5.2)) * Mat4::from_scale(Vec3::splat(0.15)),
    );
    let sun = Vec3::new(-0.4, 0.5, 0.6);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        color: Vec3::new(1.0, 0.8, 0.6),
        intensity: 1.6,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            sun_color: Vec3::new(1.0, 0.7, 0.45),
            look: None,
        },
        intensity: 0.25,
        background: Background::Color(Vec3::new(0.01, 0.012, 0.02)),
    };
    scene.post.bloom = Some(Bloom::default());
    scene.post.depth_of_field = Some(DepthOfField {
        focus_distance: 10.2,
        focus_range: 1.2,
        max_blur: 0.014,
        falloff: 3.0,
        quality,
    });
    scene.camera = Camera {
        position: Vec3::new(0.0, 1.5, 7.0),
        target: Vec3::new(0.0, 1.1, -4.0),
        exposure: Exposure::linear(1.1),
        ..Default::default()
    };
    Ok(scene)
}

// ---------------------------------------------------------------------------
// God rays.
// ---------------------------------------------------------------------------

/// A colonnade against a low sun in view: shafts of light between the columns.
pub fn god_rays_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    terrain(&mut scene, 120.0);
    let column = scene.add_mesh(shapes::cuboid(Vec3::new(0.7, 7.0, 0.7)));
    let beam = scene.add_mesh(shapes::cuboid(Vec3::new(24.0, 0.8, 1.0)));
    let stone = scene.add_material(Material {
        base_color: [0.7, 0.66, 0.6, 1.0],
        roughness: 0.8,
        ..Default::default()
    });
    for i in 0..13 {
        let x = -10.8 + i as f32 * 1.8;
        scene.add_instance(
            column,
            stone,
            Mat4::from_translation(Vec3::new(x, 3.5, -12.0)),
        );
    }
    scene.add_instance(
        beam,
        stone,
        Mat4::from_translation(Vec3::new(0.0, 7.4, -12.0)),
    );
    let sun = Vec3::new(0.1, 0.16, -1.0);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        color: Vec3::new(1.0, 0.8, 0.55),
        intensity: 2.5,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            sun_color: Vec3::new(1.0, 0.75, 0.45),
            look: None,
        },
        intensity: 0.4,
        background: Background::Sky,
    };
    scene.post.god_rays = Some(GodRays {
        intensity: 0.7,
        ..Default::default()
    });
    scene.post.bloom = Some(Bloom::default());
    scene.camera = Camera {
        position: Vec3::new(0.0, 1.8, 6.0),
        target: Vec3::new(0.3, 3.2, -12.0),
        exposure: Exposure::linear(0.7),
        ..Default::default()
    };
    scene
}

// ---------------------------------------------------------------------------
// Motion blur.
// ---------------------------------------------------------------------------

fn spinner(frame: u32) -> Mat4 {
    Mat4::from_translation(Vec3::new(-1.6, 1.2, 0.0))
        * Mat4::from_rotation_z(frame as f32 * 0.35)
        * Mat4::from_scale(Vec3::new(2.4, 0.25, 0.25))
}

fn racer(frame: u32) -> Mat4 {
    Mat4::from_translation(Vec3::new(-3.0 + frame as f32 * 0.9, 0.5, 1.4))
}

/// A spinning bar, a ball racing across and the camera panning: per-object
/// and camera motion blur.
pub fn motion_blur_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    terrain(&mut scene, 60.0);
    let cube = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    let ball = scene.add_mesh(shapes::sphere(32, 24));
    let red = scene.add_material(Material {
        base_color: [0.85, 0.2, 0.15, 1.0],
        roughness: 0.4,
        ..Default::default()
    });
    let yellow = scene.add_material(Material {
        base_color: [0.95, 0.8, 0.2, 1.0],
        roughness: 0.3,
        ..Default::default()
    });
    scene.add_instance(cube, red, spinner(0));
    scene.add_instance(ball, yellow, racer(0));
    for i in 0..6 {
        scene.add_instance(
            cube,
            red,
            Mat4::from_translation(Vec3::new(-5.0 + i as f32 * 2.0, 0.5, -3.0)),
        );
    }
    scene.post.motion_blur = Some(MotionBlur::default());
    scene
}

fn motion_camera(frame: u32) -> Camera {
    Camera {
        position: Vec3::new(frame as f32 * 0.25, 2.4, 6.0),
        target: Vec3::new(frame as f32 * 0.25, 0.8, -1.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    }
}

/// Moves the spinner, the racer and the camera; records the previous
/// transforms for per-object motion vectors.
pub fn move_blur(scene: &mut RenderScene, frame: u32) {
    let prev = frame.saturating_sub(1);
    scene.previous_transforms = scene.instances.iter().map(|i| i.transform).collect();
    // Instance 0 is the ground.
    scene.previous_transforms[1] = spinner(prev);
    scene.previous_transforms[2] = racer(prev);
    scene.instances[1].transform = spinner(frame);
    scene.instances[2].transform = racer(frame);
    scene.camera = motion_camera(frame);
    tick_all(scene, frame as u64);
}

// ---------------------------------------------------------------------------
// The stylised look: toon, outlines, rim light, dissolve, flash.
// ---------------------------------------------------------------------------

fn torus(major: f32, minor: f32, segments: u32, sides: u32) -> MeshData {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for i in 0..=segments {
        let u = i as f32 / segments as f32 * std::f32::consts::TAU;
        for j in 0..=sides {
            let v = j as f32 / sides as f32 * std::f32::consts::TAU;
            let n = Vec3::new(u.cos() * v.cos(), v.sin(), u.sin() * v.cos());
            let c = Vec3::new(u.cos(), 0.0, u.sin()) * major;
            vertices.push(Vertex {
                position: (c + n * minor).to_array(),
                normal: n.to_array(),
                uv: [i as f32 / segments as f32, j as f32 / sides as f32],
                color: [1.0; 4],
            });
        }
    }
    for i in 0..segments {
        for j in 0..sides {
            let a = i * (sides + 1) + j;
            let b = a + sides + 1;
            indices.extend_from_slice(&[a, a + 1, b, a + 1, b + 1, b]);
        }
    }
    MeshData { vertices, indices }
}

fn ramp_texture(scene: &mut RenderScene) -> TextureHandle {
    // Four hard bands, cool shadow to warm light.
    let rgba: Vec<u8> = (0..8)
        .flat_map(|i| {
            let v = [40u8, 40, 110, 110, 190, 190, 255, 255][i];
            [v, v, v, 255]
        })
        .collect();
    scene.add_texture(TextureData {
        width: 8,
        height: 1,
        rgba,
        color_space: ColorSpace::Linear,
        mips: Mips::None,
    })
}

/// A cel-shaded toy box: toon spheres and a torus with outlines and rim
/// light, a trickster dissolving with a glowing edge, one hit-flashed.
pub fn stylised_scene() -> Result<RenderScene, String> {
    let mut scene = RenderScene::new();
    let ramp = ramp_texture(&mut scene);
    let floor = scene.add_mesh(shapes::plane(40.0, 1.0));
    let floor_m = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.55, 0.75, 0.55),
        toon: 1,
        gradientMap: ramp.slot(),
        ..Default::default()
    });
    scene.add_instance(floor, floor_m, Mat4::IDENTITY);
    let sphere = scene.add_mesh(shapes::sphere(48, 32));
    let ring = scene.add_mesh(torus(0.7, 0.25, 64, 24));
    let toon = |color: Vec3, outline: f32| StandardMaterial {
        color,
        roughness: 0.4,
        toon: 1,
        gradientMap: ramp.slot(),
        rimColor: Vec3::new(1.0, 0.95, 0.9),
        rimIntensity: 0.6,
        rimPower: 3.5,
        outlineColor: Vec3::new(0.05, 0.04, 0.08),
        outlineWidth: outline,
        ..Default::default()
    };
    let red = scene.add_standard_material(toon(Vec3::new(0.9, 0.25, 0.2), 3.0));
    let blue = scene.add_standard_material(toon(Vec3::new(0.2, 0.45, 0.95), 2.0));
    let gold = scene.add_standard_material(StandardMaterial {
        metalness: 0.0,
        ..toon(Vec3::new(1.0, 0.75, 0.2), 4.0)
    });
    let flashed = scene.add_standard_material(StandardMaterial {
        flash: 0.65,
        flashColor: Vec3::new(1.0, 1.0, 1.0),
        ..toon(Vec3::new(0.3, 0.8, 0.4), 3.0)
    });
    scene.add_instance(
        sphere,
        red,
        Mat4::from_translation(Vec3::new(-2.0, 0.6, 0.0)) * Mat4::from_scale(Vec3::splat(1.2)),
    );
    scene.add_instance(
        sphere,
        blue,
        Mat4::from_translation(Vec3::new(2.1, 0.5, 0.6)),
    );
    scene.add_instance(
        sphere,
        flashed,
        Mat4::from_translation(Vec3::new(0.9, 0.35, 2.0)) * Mat4::from_scale(Vec3::splat(0.7)),
    );
    scene.add_instance(
        ring,
        gold,
        Mat4::from_translation(Vec3::new(0.0, 1.0, -1.2)) * Mat4::from_rotation_x(1.1),
    );
    // A trickster half dissolved.
    let model = load("trickster.glb")?;
    let parts = model.add_to(&mut scene);
    for &(_, mat) in &parts.parts {
        let mut m = *scene.standard_material(mat).expect("a material");
        m.dissolve = 0.45;
        m.dissolveScale = 6.0;
        m.dissolveEdgeColor = Vec3::new(1.0, 3.0, 6.0);
        m.dissolveEdgeWidth = 0.08;
        m.rimIntensity = 0.4;
        m.outlineWidth = 2.0;
        m.outlineColor = Vec3::new(0.05, 0.04, 0.08);
        scene.set_standard_material(mat, m);
    }
    let scale = 2.2 / (model.max - model.min).max_element().max(1e-3);
    parts.instance(
        &mut scene,
        Mat4::from_translation(Vec3::new(-0.4, -model.min.y * scale, 0.6))
            * Mat4::from_rotation_y(0.5)
            * Mat4::from_scale(Vec3::splat(scale)),
    );
    let sun = Vec3::new(0.5, 0.8, 0.4);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 2.6,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            sun_color: Vec3::ONE,
            look: None,
        },
        intensity: 0.3,
        background: Background::Color(Vec3::new(0.35, 0.55, 0.85)),
    };
    scene.post.outline = Some(Outline {
        width: 1.5,
        ..Default::default()
    });
    scene.post.bloom = Some(Bloom::default());
    scene.camera = Camera {
        position: Vec3::new(0.0, 3.2, 7.0),
        target: Vec3::new(0.0, 0.7, 0.0),
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    Ok(scene)
}

// ---------------------------------------------------------------------------
// GPU skinning.
// ---------------------------------------------------------------------------

/// Bones of the skinned cylinder.
pub const CYLINDER_BONES: usize = 4;
/// Its height (bones split it evenly along +y).
pub const CYLINDER_HEIGHT: f32 = 2.4;

/// A capped cylinder along +y with `rings` rings, skinned to
/// [`CYLINDER_BONES`] bones: each vertex blends the two bones nearest its
/// height.
pub fn skinned_cylinder(radius: f32, rings: u32, sides: u32) -> (MeshData, SkinData) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut joints = Vec::new();
    let mut weights = Vec::new();
    let seg = CYLINDER_HEIGHT / CYLINDER_BONES as f32;
    let skin_at = |y: f32| {
        let f = (y / seg - 0.5).clamp(0.0, (CYLINDER_BONES - 1) as f32);
        let a = (f.floor() as u16).min(CYLINDER_BONES as u16 - 1);
        let b = (a + 1).min(CYLINDER_BONES as u16 - 1);
        let w = f - a as f32;
        ([a, b, 0, 0], [1.0 - w, w, 0.0, 0.0])
    };
    for r in 0..=rings {
        let y = r as f32 / rings as f32 * CYLINDER_HEIGHT;
        for s in 0..=sides {
            let a = s as f32 / sides as f32 * std::f32::consts::TAU;
            let n = Vec3::new(a.cos(), 0.0, a.sin());
            vertices.push(Vertex {
                position: [n.x * radius, y, n.z * radius],
                normal: n.to_array(),
                uv: [s as f32 / sides as f32, 1.0 - r as f32 / rings as f32],
                color: [1.0; 4],
            });
            let (j, w) = skin_at(y);
            joints.push(j);
            weights.push(w);
        }
    }
    for r in 0..rings {
        for s in 0..sides {
            let a = r * (sides + 1) + s;
            let b = a + sides + 1;
            indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    // Caps.
    for (y, up) in [(0.0f32, -1.0f32), (CYLINDER_HEIGHT, 1.0)] {
        let center = vertices.len() as u32;
        vertices.push(Vertex {
            position: [0.0, y, 0.0],
            normal: [0.0, up, 0.0],
            uv: [0.5, 0.5],
            color: [1.0; 4],
        });
        let (j, w) = skin_at(y);
        joints.push(j);
        weights.push(w);
        for s in 0..=sides {
            let a = s as f32 / sides as f32 * std::f32::consts::TAU;
            vertices.push(Vertex {
                position: [a.cos() * radius, y, a.sin() * radius],
                normal: [0.0, up, 0.0],
                uv: [0.5 + a.cos() * 0.5, 0.5 + a.sin() * 0.5],
                color: [1.0; 4],
            });
            joints.push(j);
            weights.push(w);
        }
        for s in 0..sides {
            let (p, q) = (center + 1 + s, center + 2 + s);
            if up > 0.0 {
                indices.extend_from_slice(&[center, q, p]);
            } else {
                indices.extend_from_slice(&[center, p, q]);
            }
        }
    }
    (
        vertices_mesh(vertices, indices),
        SkinData { joints, weights },
    )
}

fn vertices_mesh(vertices: Vec<Vertex>, indices: Vec<u32>) -> MeshData {
    MeshData { vertices, indices }
}

/// The palette that bends the cylinder: each bone turns by `bend` radians
/// about z (and `twist` about y) relative to its parent, at its base.
pub fn bend_palette(bend: f32, twist: f32) -> JointPalette {
    let seg = CYLINDER_HEIGHT / CYLINDER_BONES as f32;
    let mut matrices = Vec::with_capacity(CYLINDER_BONES);
    let mut global = Mat4::IDENTITY;
    for b in 0..CYLINDER_BONES {
        // Bone b's rest frame sits at height b * seg.
        let local = if b == 0 {
            Mat4::from_rotation_y(twist)
        } else {
            Mat4::from_translation(Vec3::Y * seg)
                * Mat4::from_rotation_z(bend)
                * Mat4::from_rotation_y(twist)
        };
        global *= local;
        let inverse_bind = Mat4::from_translation(-Vec3::Y * seg * b as f32);
        matrices.push(global * inverse_bind);
    }
    JointPalette { matrices }
}

/// Five skinned cylinders (tentacles) on one mesh and material, each posed
/// by its own palette, casting shadows.
pub fn skinning_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    terrain(&mut scene, 40.0);
    let (mesh, skin) = skinned_cylinder(0.22, 24, 24);
    let cylinder = scene.add_skinned_mesh(mesh, skin);
    let flesh = scene.add_material(Material {
        base_color: [0.75, 0.3, 0.55, 1.0],
        roughness: 0.45,
        ..Default::default()
    });
    for i in 0..5 {
        let bend = -0.45 + i as f32 * 0.22;
        scene
            .joint_palettes
            .push(bend_palette(bend, i as f32 * 0.2));
        scene.skinned_instances.push(SkinnedInstance::new(
            cylinder,
            flesh,
            Mat4::from_translation(Vec3::new(-3.0 + i as f32 * 1.5, 0.0, 0.0)),
            i,
        ));
    }
    scene.camera = Camera {
        position: Vec3::new(0.0, 2.6, 7.0),
        target: Vec3::new(0.0, 1.2, 0.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene
}

// ---------------------------------------------------------------------------
// The showcase's effects.
// ---------------------------------------------------------------------------

/// The showcase's starting tick (its particles and trails are under way).
pub const SHOWCASE_TICK: u64 = 150;

/// Adds the effects to the showcase scene: a campfire between the figures
/// (with its light), scorch decals under the ships, wisps with trails, an
/// energy tether between the ships, ambient occlusion and the grading
/// stack (the sun is out of frame: god rays have their own scene).
pub fn showcase_effects(scene: &mut RenderScene) {
    let fire = campfire(scene);
    logs(scene, Vec3::new(1.4, 0.0, 1.4));
    light_campfire(scene, &fire, Vec3::new(1.4, 0.0, 1.4), 100);
    scene.lights.push(Light {
        kind: LightKind::Point,
        position: Vec3::new(1.4, 0.6, 1.4),
        color: Vec3::new(1.0, 0.55, 0.25),
        intensity: 5.0,
        range: 7.0,
        cast_shadows: false,
    });
    let atlas = decal_atlas(scene);
    scene.decal_atlas = Some(atlas);
    for (p, s, r) in [
        (Vec3::new(-5.0, 0.0, -5.0), 4.0, 0.2),
        (Vec3::new(5.5, 0.0, -4.0), 3.6, 1.3),
        (Vec3::new(1.4, 0.0, 1.4), 2.2, 0.0),
    ] {
        scene.decals.push(Decal {
            uv_rect: DECAL_SCORCH,
            ..Decal::on_surface(p, Vec3::Y, Vec2::splat(s), 0.8, r)
        });
    }
    scene.decals.push(Decal {
        uv_rect: DECAL_RUNE,
        emissive: Vec3::new(0.8, 2.0, 3.0),
        ..Decal::on_surface(
            Vec3::new(-1.6, 0.0, 2.4),
            Vec3::Y,
            Vec2::splat(1.6),
            0.4,
            0.5,
        )
    });
    let beam_tex = beam_texture(scene);
    scene.beams.push(Beam {
        start: Vec3::new(-4.4, 0.9, -4.8),
        end: Vec3::new(5.0, 0.9, -3.8),
        width: 0.18,
        color: [0.8, 2.4, 5.0, 1.0],
        texture: Some(beam_tex),
        tile: 0.8,
        scroll: 1.5,
        ..Default::default()
    });
    for (i, c) in [[1.0, 0.45, 0.2], [0.3, 0.7, 1.0]].iter().enumerate() {
        let style = scene.add_trail_style(TrailStyle {
            lifetime: 0.7,
            min_distance: 0.02,
            width: Curve::new(&[(0.0, 0.12), (1.0, 0.0)]),
            color: ColorCurve::new(&[
                (0.0, [c[0] * 5.0, c[1] * 5.0, c[2] * 5.0, 1.0]),
                (1.0, [c[0], c[1], c[2], 0.0]),
            ]),
            ..Default::default()
        });
        scene.trails.push(Trail {
            id: 200 + i as u64,
            style,
            position: showcase_wisp(i, SHOWCASE_TICK),
            tick: SHOWCASE_TICK,
        });
    }
    scene.post.ao = Some(AmbientOcclusion::default());
    grade(scene);
    if let Some(g) = &mut scene.post.film_grain {
        g.intensity = 0.025;
    }
    scene.post.chromatic_aberration = Some(ChromaticAberration { intensity: 1.2 });
    // The title glows warm against the graded sky.
    for t in &mut scene.texts {
        t.color = [2.4, 0.9, 0.25, 1.0];
    }
    // The darker ground meters brighter: hold the sky back.
    scene.camera.exposure = Exposure::Auto {
        compensation: -1.1,
        min_ev: -4.0,
        max_ev: 4.0,
    };
    tick_all(scene, SHOWCASE_TICK);
}

fn showcase_wisp(i: usize, tick: u64) -> Vec3 {
    let t = tick as f32 / 60.0;
    let a = t * 2.2 + i as f32 * std::f32::consts::PI;
    Vec3::new(
        -0.2 + a.cos() * 1.3,
        2.6 + (t * 3.0 + i as f32).sin() * 0.3,
        0.8 + a.sin() * 1.3,
    )
}

/// The showcase's per-frame update: clocks and wisps.
pub fn showcase_update(scene: &mut RenderScene, frame: u32) {
    let tick = SHOWCASE_TICK + frame as u64;
    tick_all(scene, tick);
    for (i, t) in scene.trails.iter_mut().enumerate() {
        t.position = showcase_wisp(i, tick);
    }
}

/// A HUD in the screen-space overlay over a 3D scene (the `overlay` scene):
/// a panel with a title and a health bar, a rotated marker, a list scrolled
/// half out of its clip rectangle (rows and their text cut at the edge), and
/// a crosshair; 1280×720 CSS pixels at `scale` 0 (fit to the output).
pub fn overlay_hud(scene: &mut RenderScene) -> Result<(), String> {
    let font = crate::scenes::load_font(scene)?;
    let mut o = og_render::Overlay::new(1280.0, 720.0, 0.0);
    let solid = |x: f32, y: f32, w: f32, h: f32, color: [f32; 4], layer: i32| Sprite {
        texture: None,
        position: Vec3::new(x, y, 0.0),
        size: Vec2::new(w, h),
        anchor: Vec2::ZERO,
        color,
        layer,
        ..Default::default()
    };
    let text = |s: &str, x: f32, y: f32, size: f32, color: [f32; 4], layer: i32| Text {
        position: Vec3::new(x, y, 0.0),
        size,
        color,
        layer,
        ..Text::new(font, s)
    };
    // The panel, the title and the health bar (exact token-like colours).
    o.sprites.push(solid(
        24.0,
        24.0,
        320.0,
        120.0,
        [0.02, 0.02, 0.025, 0.85],
        0,
    ));
    o.texts.push(text(
        "Origin 原点",
        44.0,
        64.0,
        30.0,
        [1.0, 1.0, 1.0, 1.0],
        1,
    ));
    o.sprites
        .push(solid(44.0, 92.0, 280.0, 20.0, [0.1, 0.1, 0.1, 1.0], 1));
    o.sprites
        .push(solid(44.0, 92.0, 196.0, 20.0, [0.9, 0.18, 0.05, 1.0], 2));
    o.texts.push(text(
        "HP 70/100",
        48.0,
        132.0,
        16.0,
        [0.8, 0.8, 0.8, 1.0],
        1,
    ));
    // A marker turned 30° clockwise about its centre.
    o.sprites.push(Sprite {
        anchor: Vec2::splat(0.5),
        rotation: 30f32.to_radians(),
        ..solid(1180.0, 90.0, 60.0, 60.0, [1.0, 0.73, 0.0, 1.0], 0)
    });
    // A list in a 260×150 clip, scrolled 23 px: its first and last rows cut.
    let clip = o.add_clip([980.0, 300.0, 260.0, 150.0]);
    o.sprites.push(solid(
        980.0,
        300.0,
        260.0,
        150.0,
        [0.02, 0.02, 0.025, 0.85],
        0,
    ));
    for row in 0..5 {
        let y = 300.0 - 23.0 + row as f32 * 44.0;
        o.sprites.push(Sprite {
            clip: Some(clip),
            ..solid(988.0, y, 244.0, 38.0, [0.2, 0.22, 0.26, 1.0], 1)
        });
        o.texts.push(Text {
            clip: Some(clip),
            ..text(&format!("Item {row}"), 1000.0, y + 27.0, 20.0, [1.0; 4], 2)
        });
    }
    // A crosshair at the centre.
    o.sprites
        .push(solid(638.0, 348.0, 4.0, 24.0, [1.0, 1.0, 1.0, 0.9], 0));
    o.sprites
        .push(solid(628.0, 358.0, 24.0, 4.0, [1.0, 1.0, 1.0, 0.9], 0));
    scene.overlay = Some(o);
    Ok(())
}
