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
    ColorAdjust, ColorCurve, ColorGrading, ColorLut, ColorSpace, Curve, Decal, DepthOfField,
    DirectionalLight, DofQuality, EmitterHandle, EmitterShape, Environment, Exposure, FilmGrain,
    Glow, GodRays, Instance, InstanceFx, JointPalette, LensFlare, Light, Light2d, LightKind,
    Lighting2d, Material, MeshData, Mips, MotionBlur, Occluder2d, Outline, ParticleAlign,
    ParticleBlend, ParticleCollision, ParticleEmitter, ParticleNoise, ParticleRender,
    ParticleSystem, ProceduralSky, RenderScene, Ribbon, RibbonPoint, RibbonUv, SheetMode,
    SimulationSpace, SkinData, SkinnedInstance, Sprite, SpriteFx, SpriteSheet, StandardMaterial,
    Text, TextureData, TextureHandle, Tonemap, Trail, TrailStyle, Vertex, Vignette, shapes,
};

mod models;
mod particles;
mod post;
mod showcase;
mod sprites;

pub use models::*;
pub use particles::*;
pub use post::*;
pub use showcase::*;
pub use sprites::*;

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

pub(crate) fn smooth(e0: f32, e1: f32, x: f32) -> f32 {
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
pub(crate) fn noise2(x: f32, y: f32, seed: u32) -> f32 {
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
pub(crate) fn tiling_fbm(u: f32, v: f32, period: i32, seed: u32) -> f32 {
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

// ---------------------------------------------------------------------------
// Decals.
// ---------------------------------------------------------------------------

/// A bullet-hole atlas: the left half the hole's colour (a dark pit in a
/// pale chipped rim), the right half its tangent-space normal map (the rim
/// raised, the pit sunk; glTF's convention, +y up the image), sRGB as an
/// artist's atlas usually is.
pub fn bullet_hole_atlas(scene: &mut RenderScene) -> TextureHandle {
    scene.add_texture(texture(256, ColorSpace::Srgb, |u, v| {
        let qu = (u * 2.0).fract();
        let (x, y) = (qu - 0.5, 0.5 - v);
        let d = (x * x + y * y).sqrt() * 2.0;
        // Height: a raised rim at d 0.45 around a pit, flat outside 0.9.
        let h = |d: f32| (-((d - 0.45) / 0.18).powi(2)).exp() * 0.6 - (1.0 - smooth(0.0, 0.35, d));
        if u < 0.5 {
            let pit = 1.0 - smooth(0.12, 0.3, d);
            let rim = (1.0 - smooth(0.0, 0.25, (d - 0.45).abs())) * 0.8;
            let a = (pit + rim).min(1.0) * (1.0 - smooth(0.7, 0.9, d));
            let c = 30.0 + 170.0 * rim * (1.0 - pit);
            [
                c as u8,
                (c * 0.95) as u8,
                (c * 0.9) as u8,
                (a * 255.0) as u8,
            ]
        } else {
            // The slope of h along the radius, as a normal.
            let e = 0.01;
            // Gentle at normalScale 1 (40° at the steepest, the pit's lip), so a
            // normalScale of 2 visibly deepens it.
            let g = (h(d + e) - h(d - e)) / (2.0 * e) * 0.15;
            let (dx, dy) = if d > 1e-4 {
                (x / (d * 0.5) * 0.5, y / (d * 0.5) * 0.5)
            } else {
                (0.0, 0.0)
            };
            let n = Vec3::new(-g * dx, -g * dy, 1.0).normalize();
            let enc = |c: f32| ((c * 0.5 + 0.5) * 255.0).round() as u8;
            [enc(n.x), enc(n.y), enc(n.z), 255]
        }
    }))
}

/// A wall and the ground before it peppered with 640 bullet holes (more
/// than the 32 decals a frame drew before R7), each with the atlas's normal
/// map, and two large ones that show the normal map's relief under the low
/// sun: their rims lit on the sun's side, shadowed on the other.
pub fn decals_many_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    terrain(&mut scene, 40.0);
    scene.decal_atlas = Some(bullet_hole_atlas(&mut scene));
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
    let hole = |p: Vec3, n: Vec3, size: f32, rot: f32| Decal {
        uv_rect: [0.0, 0.0, 0.5, 1.0],
        normal_rect: [0.5, 0.0, 1.0, 1.0],
        normal_scale: 1.0,
        roughness: 0.95,
        ..Decal::on_surface(p, n, Vec2::splat(size), 0.3, rot)
    };
    let mut decals = Vec::new();
    for k in 0..640u32 {
        let (i, j) = ((k % 32) as f32, (k / 32) as f32);
        let jitter = |s: u32| hash2(k as i32, s as i32, 77) - 0.5;
        let rot = hash2(k as i32, 9, 77) * std::f32::consts::TAU;
        let size = 0.12 + 0.1 * hash2(k as i32, 3, 77);
        if k < 400 {
            // The wall's face (z = -2.0), 32 × 12 holes.
            let p = Vec3::new(
                -3.8 + i * 0.245 + jitter(1) * 0.1,
                0.2 + j * 0.22 + jitter(2) * 0.1,
                -2.0,
            );
            decals.push(hole(p, Vec3::Z, size, rot));
        } else {
            // The ground before it, 32 × 8 more.
            let p = Vec3::new(
                -3.8 + i * 0.245 + jitter(1) * 0.1,
                0.0,
                -1.6 + (j - 12.5) * 0.3 + jitter(2) * 0.1,
            );
            decals.push(hole(p, Vec3::Y, size, rot));
        }
    }
    // Two large ones, under the low sun, where the relief reads.
    decals.push(hole(Vec3::new(-1.5, 0.0, 1.0), Vec3::Y, 1.6, 0.0));
    decals.push(Decal {
        normal_scale: 2.0,
        ..hole(Vec3::new(1.5, 0.0, 1.0), Vec3::Y, 1.6, 0.0)
    });
    scene.decals = decals;
    let sun = Vec3::new(-0.8, 0.45, 0.4);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        ..Default::default()
    };
    scene.environment.sky.sun_direction = sun;
    scene.camera = Camera {
        position: Vec3::new(0.0, 3.0, 5.6),
        target: Vec3::new(0.0, 0.6, -0.8),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene
}

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
