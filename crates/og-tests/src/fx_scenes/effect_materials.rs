//! Effect materials (og-render `effect.rs`): one parameter set drawing
//! billboards, mesh particles, effect meshes, trails, ribbons and beams.
//! Every texture is generated here.

use super::*;
use og_render::geometry::primitives::{ring, torus};
use og_render::{
    EffectChannel, EffectMaterial, EffectMesh, EffectTexture, EffectUv, EffectValue, Erosion, Flow,
    Fresnel, MaterialShader, ParamKind, ShaderParam,
};

/// Tiling greyscale noise (erosion, flow, smoke shapes), linear.
fn noise_texture(scene: &mut RenderScene, period: i32, seed: u32) -> TextureHandle {
    let mut t = texture(128, ColorSpace::Linear, |u, v| {
        let n = tiling_fbm(u, v, period, seed);
        let g = ((n - 0.15) / 0.7).clamp(0.0, 1.0);
        let b = (g * 255.0) as u8;
        [b, b, b, 255]
    });
    t.mips = Mips::Generate;
    scene.add_texture(t)
}

/// A flow map: a swirl about the centre in red and green (0.5 still).
fn swirl_flow(scene: &mut RenderScene) -> TextureHandle {
    let mut t = texture(64, ColorSpace::Linear, |u, v| {
        let (x, y) = (u - 0.5, v - 0.5);
        let r = (x * x + y * y).sqrt().max(1e-3);
        let (fx, fy) = (-y / r, x / r);
        let s = (r * 2.0).min(1.0);
        [
            ((0.5 + 0.5 * fx * s) * 255.0) as u8,
            ((0.5 + 0.5 * fy * s) * 255.0) as u8,
            128,
            255,
        ]
    });
    t.mips = Mips::Generate;
    scene.add_texture(t)
}

/// A soft bar across v: a slash's or beam's cross section, in red.
fn bar_mask(scene: &mut RenderScene) -> TextureHandle {
    let mut t = texture(64, ColorSpace::Linear, |u, v| {
        let across = 1.0 - smooth(0.0, 0.5, (v - 0.5).abs());
        let along = smooth(0.0, 0.15, u) * (1.0 - smooth(0.6, 1.0, u));
        let b = (across * along * 255.0) as u8;
        [b, b, b, 255]
    });
    t.mips = Mips::Generate;
    scene.add_texture(t)
}

/// A soft ring across v (with polar coordinates, across the radius).
fn ring_mask(scene: &mut RenderScene) -> TextureHandle {
    let mut t = texture(64, ColorSpace::Linear, |_, v| {
        let a = smooth(0.15, 0.55, v) * (1.0 - smooth(0.8, 1.0, v));
        let b = (a * 255.0) as u8;
        [b, b, b, 255]
    });
    t.mips = Mips::Generate;
    scene.add_texture(t)
}

/// A round soft falloff from the centre, in red.
fn round_mask(scene: &mut RenderScene) -> TextureHandle {
    let mut t = texture(64, ColorSpace::Linear, |u, v| {
        let d = ((u - 0.5).powi(2) + (v - 0.5).powi(2)).sqrt() * 2.0;
        let a = (1.0 - smooth(0.2, 1.0, d)).powi(2);
        let b = (a * 255.0) as u8;
        [b, b, b, 255]
    });
    t.mips = Mips::Generate;
    scene.add_texture(t)
}

/// The hook a material shader can give an effect: bands of custom colour
/// along v, as many as the particle's custom data asks (`effect_custom1`).
fn banded_hook() -> MaterialShader {
    MaterialShader {
        name: "effect-bands".into(),
        code: "fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> {
  let bands = max(effect_custom1().x, 1.0);
  let b = step(0.5, fract(h.local.y * bands + effect_random()));
  return vec4<f32>(mix(c.rgb, p.tint.rgb * c.a, b * 0.8), c.a);
}
"
        .into(),
        params: vec![ShaderParam {
            name: "tint".into(),
            kind: ParamKind::Vec4,
            default: [4.0, 0.6, 2.5, 1.0],
        }],
        mode: Default::default(),
    }
}

/// Seven effect surfaces in a row under a night sky, each using one part
/// of the material: dissolving puffs (erosion over life, a ramp), a portal
/// (polar coordinates, a turning flow map), a shield (fresnel rim and
/// scrolling noise), a slash (a mask and erosion along an arc), a trail
/// (scrolling noise along it), a beam (a scrolling noise core) and puffs
/// banded by a material hook from their custom data.
pub fn effect_materials_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    terrain(&mut scene, 40.0);
    night_sky(&mut scene);
    let noise = noise_texture(&mut scene, 4, 3);
    let fine = noise_texture(&mut scene, 8, 11);
    let flow = swirl_flow(&mut scene);
    let bar = bar_mask(&mut scene);
    let rim = ring_mask(&mut scene);
    let round = round_mask(&mut scene);
    let noise_tex = |texture, tile: f32, scroll: Vec2| EffectTexture {
        texture: Some(texture),
        uv: EffectUv {
            tile: Vec2::splat(tile),
            scroll,
            ..Default::default()
        },
        channel: EffectChannel::Red,
    };
    let fire_ramp = ColorCurve::new(&[
        (0.0, [0.2, 0.01, 0.0, 0.0]),
        (0.35, [0.9, 0.18, 0.02, 0.7]),
        (0.7, [2.0, 0.8, 0.15, 1.0]),
        (1.0, [3.0, 2.2, 1.2, 1.0]),
    ]);
    // 1. Dissolving fire puffs: noise through a ramp, eroded over life.
    let puffs = scene.add_effect_material(EffectMaterial {
        main: noise_tex(noise, 0.6, Vec2::new(0.0, -0.4)),
        ramp: Some(fire_ramp.clone()),
        erosion: Some(Erosion {
            noise: noise_tex(fine, 0.8, Vec2::ZERO),
            amount: EffectValue::Curve(Curve::new(&[(0.0, 0.0), (0.3, 0.15), (1.0, 1.0)])),
            softness: 0.12,
            edge: 0.08,
            edge_color: [3.0, 1.2, 0.3, 1.0],
        }),
        mask: Some(EffectTexture {
            texture: Some(round),
            channel: EffectChannel::Red,
            ..Default::default()
        }),
        intensity: EffectValue::Constant(0.6),
        soft_depth: 0.3,
        ..Default::default()
    });
    let puff_emitter = scene.add_emitter(ParticleEmitter {
        max_particles: 64,
        shape: EmitterShape::Sphere {
            radius: 0.25,
            surface: false,
        },
        rate: 9.0,
        lifetime: [1.2, 1.6],
        speed: [0.3, 0.6],
        size: [0.9, 1.3],
        size_over_life: Curve::new(&[(0.0, 0.6), (1.0, 1.2)]),
        color_over_life: ColorCurve::constant([1.0; 4]),
        rotation_degrees: [0.0, 360.0],
        angular_velocity_degrees: [-40.0, 40.0],
        gravity: Vec3::new(0.0, 0.8, 0.0),
        drag: 0.6,
        effect: Some(puffs),
        ..Default::default()
    });
    scene.particles.push(ParticleSystem::new(
        1,
        puff_emitter,
        Mat4::from_translation(Vec3::new(-5.4, 0.6, 0.0)),
        5,
        0,
    ));
    // 2. A portal: a disc in polar coordinates, its noise streaming
    // round and in along a swirl flow, through a violet ramp.
    let portal = scene.add_effect_material(EffectMaterial {
        main: EffectTexture {
            texture: Some(noise),
            uv: EffectUv {
                tile: Vec2::new(3.0, 1.0),
                scroll: Vec2::new(0.15, -0.6),
                polar: true,
                ..Default::default()
            },
            channel: EffectChannel::Red,
        },
        ramp: Some(ColorCurve::new(&[
            (0.0, [0.0, 0.0, 0.0, 0.0]),
            (0.4, [0.3, 0.05, 0.9, 0.7]),
            (0.75, [0.9, 0.35, 2.4, 1.0]),
            (1.0, [2.2, 1.8, 3.5, 1.0]),
        ])),
        flow: Some(Flow {
            map: EffectTexture {
                texture: Some(flow),
                ..Default::default()
            },
            strength: 0.08,
            speed: 0.6,
        }),
        mask: Some(EffectTexture {
            texture: Some(rim),
            uv: EffectUv {
                polar: true,
                ..Default::default()
            },
            channel: EffectChannel::Red,
        }),
        double_sided: true,
        ..Default::default()
    });
    let disc = scene.add_mesh(ring(0.0, 1.0, 48, 8, 0.0, std::f32::consts::TAU));
    scene.effect_meshes.push(EffectMesh {
        id: 2,
        mesh: disc,
        material: portal,
        transform: Mat4::from_translation(Vec3::new(-3.2, 1.3, 0.0))
            * Mat4::from_scale(Vec3::splat(1.0)),
        life: 0.5,
        seed: 2,
        color: [1.0; 4],
        blend: ParticleBlend::Additive,
        layers: 1,
    });
    // 3. A shield dome: a fresnel rim over scrolling hex-ish noise.
    let shield = scene.add_effect_material(EffectMaterial {
        main: noise_tex(fine, 2.0, Vec2::new(0.05, 0.25)),
        fresnel: Some(Fresnel {
            power: 2.5,
            strength: 1.0,
            invert: false,
        }),
        color: [0.4, 1.6, 3.2, 1.0],
        intensity: EffectValue::Constant(1.6),
        double_sided: true,
        ..Default::default()
    });
    let dome = scene.add_mesh(shapes::sphere(32, 20));
    scene.effect_meshes.push(EffectMesh {
        id: 3,
        mesh: dome,
        material: shield,
        transform: Mat4::from_translation(Vec3::new(-0.9, 0.0, 0.0))
            * Mat4::from_scale(Vec3::splat(2.2)),
        life: 0.5,
        seed: 3,
        color: [1.0; 4],
        blend: ParticleBlend::Additive,
        layers: 1,
    });
    // 4. A slash: an open arc whose bar mask runs along it, its noise
    // scrolling with the swing and eroding the tail.
    let slash = scene.add_effect_material(EffectMaterial {
        main: noise_tex(noise, 1.0, Vec2::new(-1.5, 0.0)),
        ramp: Some(ColorCurve::new(&[
            (0.0, [0.0, 0.0, 0.0, 0.0]),
            (0.5, [0.5, 1.1, 2.0, 0.9]),
            (1.0, [2.0, 2.8, 3.5, 1.0]),
        ])),
        mask: Some(EffectTexture {
            texture: Some(bar),
            channel: EffectChannel::Red,
            ..Default::default()
        }),
        erosion: Some(Erosion {
            noise: noise_tex(fine, 1.5, Vec2::new(-0.8, 0.0)),
            amount: EffectValue::Constant(0.35),
            softness: 0.2,
            edge: 0.0,
            edge_color: [1.0; 4],
        }),
        double_sided: true,
        ..Default::default()
    });
    let arc = scene.add_mesh(torus(1.0, 0.12, 6, 48, std::f32::consts::PI * 1.2));
    scene.effect_meshes.push(EffectMesh {
        id: 4,
        mesh: arc,
        material: slash,
        transform: Mat4::from_translation(Vec3::new(1.4, 1.2, 0.0))
            * Mat4::from_rotation_z(-0.4)
            * Mat4::from_scale(Vec3::new(1.0, 1.0, 0.25)),
        life: 0.5,
        seed: 4,
        color: [1.0; 4],
        blend: ParticleBlend::Additive,
        layers: 1,
    });
    // 5. A trail through scrolling noise, fading to its tail by a curve.
    let streak = scene.add_effect_material(EffectMaterial {
        main: noise_tex(noise, 1.0, Vec2::new(-2.0, 0.0)),
        ramp: Some(fire_ramp),
        alpha: EffectValue::Curve(Curve::new(&[(0.0, 1.0), (1.0, 0.0)])),
        ..Default::default()
    });
    let style = scene.add_trail_style(TrailStyle {
        lifetime: 0.9,
        min_distance: 0.02,
        width: Curve::new(&[(0.0, 0.5), (1.0, 0.1)]),
        color: ColorCurve::constant([1.0; 4]),
        uv: RibbonUv::Tile { length: 1.5 },
        effect: Some(streak),
        ..Default::default()
    });
    scene.trails.push(Trail {
        id: 5,
        style,
        position: effect_trail_at(0),
        tick: 0,
    });
    // 6. A beam: an open tube of noise core between two points.
    let core = scene.add_effect_material(EffectMaterial {
        main: noise_tex(fine, 1.0, Vec2::new(-3.0, 0.0)),
        color: [0.5, 1.6, 0.4, 1.0],
        intensity: EffectValue::Constant(1.0),
        fresnel: Some(Fresnel {
            power: 1.5,
            strength: 1.0,
            invert: true,
        }),
        ..Default::default()
    });
    scene.beams.push(Beam {
        start: Vec3::new(4.6, 0.3, 0.0),
        end: Vec3::new(4.6, 2.8, 0.0),
        width: 0.5,
        tile: 1.0,
        segments: 8,
        effect: Some(core),
        ..Default::default()
    });
    // 7. Puffs banded by a material hook from their custom data (a random
    // band count per particle).
    let hook = scene.add_shader(banded_hook());
    let banded = scene.add_effect_material(EffectMaterial {
        color: [0.5, 0.6, 1.4, 1.0],
        custom: {
            let mut c: [[EffectValue; 4]; 2] =
                std::array::from_fn(|_| std::array::from_fn(|_| EffectValue::Constant(0.0)));
            c[0][0] = EffectValue::Random([2.0, 6.0]);
            c
        },
        shader: Some(hook),
        params: {
            let mut p = [[0.0; 4]; og_render::EFFECT_PARAMS];
            p[0] = [2.5, 0.4, 1.2, 1.0];
            p
        },
        ..Default::default()
    });
    let banded_emitter = scene.add_emitter(ParticleEmitter {
        max_particles: 40,
        shape: EmitterShape::Sphere {
            radius: 0.5,
            surface: false,
        },
        rate: 10.0,
        lifetime: [2.0, 2.5],
        speed: [0.1, 0.3],
        size: [0.5, 0.7],
        gravity: Vec3::ZERO,
        effect: Some(banded),
        ..Default::default()
    });
    scene.particles.push(ParticleSystem::new(
        7,
        banded_emitter,
        Mat4::from_translation(Vec3::new(6.6, 1.4, 0.0)),
        7,
        0,
    ));
    scene.post.bloom = Some(Bloom::default());
    scene.camera = Camera {
        position: Vec3::new(0.6, 2.4, 9.5),
        target: Vec3::new(0.6, 1.1, 0.0),
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene
}

fn night_sky(scene: &mut RenderScene) {
    let moon = Vec3::new(-0.3, 0.6, -0.6);
    scene.light = DirectionalLight {
        direction_to_light: moon,
        color: Vec3::new(0.5, 0.6, 1.0),
        intensity: 0.3,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: moon,
            sun_color: Vec3::new(0.3, 0.35, 0.5),
            look: None,
        },
        intensity: 0.08,
        background: Background::Color(Vec3::new(0.01, 0.012, 0.026)),
        ..Environment::default()
    };
}

/// Where the effect trail's head is at `tick`: a figure of eight.
fn effect_trail_at(tick: u64) -> Vec3 {
    let t = tick as f32 / 60.0 * 2.2;
    Vec3::new(3.0 + t.sin() * 0.7, 1.3 + (t * 2.0).sin() * 0.6, 0.3)
}

/// Steps the scene's clocks and moves the trail's head.
pub fn effect_materials_update(scene: &mut RenderScene, frame: u32) {
    tick_all(scene, frame as u64);
    scene.trails[0].position = effect_trail_at(frame as u64);
}
