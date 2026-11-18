//! `particles`: a fire at nightfall. Upright flames on an effect material
//! (a heat field through a ramp, pushed by a noise flow, eroded into
//! tongues), smoke, embers bouncing on the flags and sparks off the logs,
//! on a Meshy log pile in a ring of sooted stones; an orb on a plinth
//! beside it throws a stream of blue sparks into the fire and sparkles
//! over its surface (a mesh emitter).

use super::*;

/// Where the orb stands and the fire burns.
const ORB: Vec3 = Vec3::new(-1.7, 1.2, 0.5);

pub fn fire_scene() -> Result<RenderScene, String> {
    use crate::kit;
    let mut scene = RenderScene::new();
    let grass = kit::surface_tinted(&mut scene, "grass", [0.7, 0.85, 0.6]);
    let lawn = scene.add_mesh(kit::slab(Vec2::splat(-220.0), Vec2::splat(220.0), 0.0, 7.0));
    scene.add_instance(lawn, grass, Mat4::IDENTITY);
    kit::hills(&mut scene, None);
    let flags = kit::surface_tinted(&mut scene, "stone", [0.62, 0.6, 0.58]);
    let court = scene.add_mesh(kit::slab(Vec2::splat(-6.0), Vec2::splat(6.0), 0.01, 2.0));
    scene.add_instance(court, flags, Mat4::IDENTITY);
    // Late twilight: the sun 1° under the horizon behind the fire.
    crate::sky_scenes::physical_light_under(&mut scene, Vec3::new(-0.3, -0.01, -0.95));
    scene.lighting.light.cast_shadows = true;
    hearth(&mut scene)?;
    // The fire: a wide body of flames over a short dense core.
    let burn = flame_material(&mut scene);
    let flame =
        |scene: &mut RenderScene, rate: f32, size: [f32; 2], tall: [f32; 3], radius: f32| {
            scene.add_emitter(ParticleEmitter {
                max_particles: rate as u32 + 16,
                shape: EmitterShape::Cone {
                    angle_degrees: 10.0,
                    radius,
                },
                rate,
                lifetime: [0.7, 1.0],
                speed: [0.2, 0.45],
                size,
                size_over_life: Curve::new(&[(0.0, 0.7), (0.25, 1.0), (1.0, 0.55)]),
                height_over_life: Some(Curve::new(&[
                    (0.0, tall[0]),
                    (0.4, tall[1]),
                    (1.0, tall[2]),
                ])),
                color_over_life: ColorCurve::new(&[
                    (0.0, [1.0, 1.0, 1.0, 0.0]),
                    (0.12, [1.0, 1.0, 1.0, 1.0]),
                    (0.5, [1.0, 1.0, 1.0, 0.85]),
                    (0.8, [1.0, 1.0, 1.0, 0.0]),
                ]),
                rotation_degrees: [-8.0, 8.0],
                gravity: Vec3::new(0.0, 1.0, 0.0),
                drag: 0.8,
                noise: ParticleNoise {
                    strength: 0.5,
                    frequency: 1.6,
                    scroll: 0.9,
                },
                blend: ParticleBlend::Additive,
                align: ParticleAlign::Upright { axis: Vec3::Y },
                sheet: SpriteSheet {
                    columns: 4,
                    rows: 4,
                    frames: 16,
                    mode: SheetMode::OverLife { cycles: 1.0 },
                    blend: true,
                },
                effect: Some(burn),
                soft_distance: 0.15,
                ..Default::default()
            })
        };
    let fire = campfire(&mut scene);
    let flames = flame(&mut scene, 12.0, [1.15, 1.45], [1.25, 1.55, 1.65], 0.2);
    let core = flame(&mut scene, 4.0, [0.55, 0.7], [0.9, 1.0, 1.1], 0.06);
    // Its smoke unlit (a point light inside the plume would light it as a
    // singular point: lights have no source radius), dark against the sky.
    let mut smoke = scene.emitter(fire.smoke).expect("smoke").clone();
    smoke.color_over_life = ColorCurve::new(&[
        (0.0, [0.05, 0.045, 0.045, 0.0]),
        (0.15, [0.06, 0.055, 0.055, 0.8]),
        (1.0, [0.08, 0.08, 0.09, 0.0]),
    ]);
    smoke.rate = 30.0;
    let smoke = scene.add_emitter(smoke);
    light_campfire(
        &mut scene,
        &Campfire {
            flames,
            smoke,
            embers: fire.embers,
        },
        Vec3::ZERO,
        10,
    );
    // An upright quad stands centred on its particle: the flames' roots
    // (each frame's foot) sit half a quad below it, so the systems stand
    // that much higher, the roots in the logs.
    for p in &mut scene.effects.particles {
        if p.emitter == flames {
            p.transform = Mat4::from_translation(Vec3::Y * 0.8) * p.transform;
        }
    }
    scene.effects.particles.push(ParticleSystem::new(
        13,
        core,
        Mat4::from_translation(Vec3::Y * 0.36),
        0x5eed + 13,
        0,
    ));
    scene.lighting.lights.push(Light {
        position: Vec3::new(0.0, 0.95, 0.0),
        color: Vec3::new(1.0, 0.55, 0.22),
        intensity: 22.0,
        range: 14.0,
        cast_shadows: true,
        ..Default::default()
    });
    orb(&mut scene);
    scene.look.post.bloom = Some(Bloom {
        threshold: 1.8,
        intensity: 0.4,
        ..Bloom::default()
    });
    scene.frame.camera = Camera {
        position: Vec3::new(1.95, 0.6, 2.7),
        target: Vec3::new(-0.6, 0.85, 0.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: -1.9 },
        ..Default::default()
    };
    Ok(scene)
}

/// The orb on its plinth: sparkles over its surface, a stream of blue
/// sparks arcing from it down into the fire, arcs crackling round it, and
/// its blue light.
fn orb(scene: &mut RenderScene) {
    // A broken column's stump for its plinth, 1 m high.
    crate::kit::prop(scene, "column_broken").place(scene, ORB.with_y(0.0), 0.6, 1.0);
    let mesh = scene.add_mesh(shapes::sphere(32, 24));
    let glass = scene.add_material(Material {
        base_color: [0.1, 0.05, 0.15, 1.0],
        metallic: 0.2,
        roughness: 0.15,
        emissive: [0.4, 0.1, 0.6],
        ..Default::default()
    });
    let at = Mat4::from_translation(ORB) * Mat4::from_scale(Vec3::splat(0.2));
    scene.add_instance(mesh, glass, at);
    let spark = spark_texture(scene);
    let sparkle = scene.add_emitter(ParticleEmitter {
        max_particles: 160,
        shape: EmitterShape::Mesh { mesh },
        rate: 90.0,
        lifetime: [0.6, 1.2],
        speed: [0.15, 0.4],
        size: [0.02, 0.04],
        size_over_life: Curve::new(&[(0.0, 0.0), (0.2, 1.0), (1.0, 0.0)]),
        color: [6.0, 2.5, 8.0, 1.0],
        gravity: Vec3::ZERO,
        texture: Some(spark),
        ..Default::default()
    });
    scene
        .effects
        .particles
        .push(ParticleSystem::new(30, sparkle, at, 99, 0));
    let to_fire = Vec3::new(0.0, 0.45, 0.0) - ORB;
    let stream = scene.add_emitter(ParticleEmitter {
        max_particles: 400,
        shape: EmitterShape::Cone {
            angle_degrees: 5.0,
            radius: 0.03,
        },
        rate: 380.0,
        lifetime: [0.5, 0.62],
        speed: [2.6, 3.0],
        size: [0.035, 0.06],
        color_over_life: ColorCurve::new(&[
            (0.0, [0.8, 0.95, 1.0, 1.0]),
            (0.7, [0.35, 0.55, 1.0, 1.0]),
            (1.0, [0.4, 0.25, 1.0, 0.0]),
        ]),
        color: [1.5, 2.5, 5.0, 1.0],
        gravity: Vec3::new(0.0, -2.4, 0.0),
        noise: ParticleNoise {
            strength: 0.35,
            frequency: 3.0,
            scroll: 1.0,
        },
        texture: Some(spark),
        align: ParticleAlign::Velocity { stretch: 0.05 },
        ..Default::default()
    });
    let aim = (to_fire + Vec3::Y * 0.6).normalize();
    let from = ORB + to_fire.normalize() * 0.2;
    scene.effects.particles.push(ParticleSystem::new(
        31,
        stream,
        Mat4::from_translation(from) * Mat4::from_quat(glam::Quat::from_rotation_arc(Vec3::Y, aim)),
        0x5711,
        0,
    ));
    for (k, d) in [
        Vec3::new(0.2, 0.15, 0.05),
        Vec3::new(0.15, -0.12, 0.15),
        Vec3::new(0.25, 0.05, -0.12),
    ]
    .into_iter()
    .enumerate()
    {
        scene.effects.beams.push(Beam {
            start: from,
            end: from + d + to_fire.normalize() * 0.12,
            width: 0.018,
            color: [4.0, 6.0, 12.0, 1.0],
            segments: 10,
            jitter: 0.05,
            seed: 31 + k as u64,
            ..Default::default()
        });
    }
    scene.lighting.lights.push(Light {
        position: ORB,
        color: Vec3::new(0.45, 0.4, 1.0),
        intensity: 4.0,
        range: 4.0,
        ..Default::default()
    });
}

/// The hearth: a pile of charred logs (a Meshy model, its ember cracks
/// glowing) inside a ring of sooted fieldstones (one Meshy stone, turned
/// and sized per place, its sooted side to the fire).
fn hearth(scene: &mut RenderScene) -> Result<(), String> {
    let logs = load("cast/camp_logs.glb")?.add_to(scene);
    // 1.66 m across and 0.85 m tall as modelled, about its centre.
    let s = 0.42;
    logs.instance(
        scene,
        Mat4::from_translation(Vec3::new(0.0, 0.3 * s, 0.0))
            * Mat4::from_rotation_y(0.4)
            * Mat4::from_scale(Vec3::splat(s)),
    );
    let stone = load("cast/camp_stone.glb")?.add_to(scene);
    for i in 0..12 {
        let h = |k: i32| hash2(i, k, 0x57);
        let a = (i as f32 + 0.3 * h(0)) / 12.0 * std::f32::consts::TAU;
        // 1.9 m across as modelled: 0.18 to 0.25 m here, half sunk.
        let size = (0.18 + 0.07 * h(1)) / 1.9;
        let r = 0.6 + 0.05 * h(4);
        stone.instance(
            scene,
            Mat4::from_translation(Vec3::new(a.cos() * r, 0.3 * size, a.sin() * r))
                * Mat4::from_rotation_y(std::f32::consts::PI - a + 0.5 * (h(5) - 0.5))
                * Mat4::from_rotation_x(0.3 * (h(6) - 0.5))
                * Mat4::from_scale(Vec3::new(size, size * (0.7 + 0.4 * h(2)), size)),
        );
    }
    Ok(())
}

fn flame_material(scene: &mut RenderScene) -> og_render::EffectMaterialHandle {
    use og_render::{
        ColorCurve, EffectChannel, EffectMaterial, EffectTexture, EffectUv, EffectValue, Erosion,
        Flow, Mips,
    };
    // A 4 × 4 flipbook of a flame's heat over its life: three tongues
    // rising from a shared hot root, each curling as it climbs and
    // narrowing to a tip, with dark gaps between them; they lick higher
    // and part further as the frames run.
    let mut heat = texture(512, og_render::ColorSpace::Linear, |u, v| {
        // A sheet frame's v runs from its top (0) to the quad's foot (1).
        let (fu, up) = ((u * 4.0).fract(), 1.0 - (v * 4.0).fract());
        let frame = (u * 4.0) as u32 + 4 * (v * 4.0) as u32;
        let t = frame as f32 / 16.0;
        let mut h = 0.0f32;
        for k in 0..3 {
            let side = k as f32 - 1.0;
            let seed = k as f32 * 1.7;
            let lick = 0.5 + 0.5 * (seed + t * 5.0).sin();
            let tall = if k == 1 {
                0.85 + 0.12 * lick
            } else {
                0.5 + 0.3 * lick
            };
            let y = up / tall;
            if y >= 1.0 {
                continue;
            }
            let base = 0.5 + side * (0.17 + 0.05 * t);
            let curl = 0.08 * (up * 7.0 - t * 9.0 + seed * 2.3).sin() * up + 0.06 * side * up;
            let width = 0.12 * (1.0 - y).powf(1.0) + 0.004;
            let x = (fu - base - curl).abs() / width;
            let inside = (1.0 - x * x).max(0.0);
            h = h.max(inside.powf(1.3) * (1.0 - y).powf(1.4));
        }
        // The hot root joining them.
        let root =
            0.95 * (1.0 - smooth(0.0, 0.12, up)) * (1.0 - smooth(0.08, 0.34, (fu - 0.5).abs()));
        let h = h.max(root) * smooth(0.0, 0.05, up) * (1.0 - smooth(0.44, 0.5, (fu - 0.5).abs()));
        // The last frames die down: the flame cools to nothing, root and all.
        let h = h * (1.0 - smooth(0.55, 0.95, t));
        let b = (h.clamp(0.0, 1.0) * 255.0) as u8;
        [b, b, b, 255]
    });
    // No mip chain: below a frame's own size the levels would average
    // neighbouring frames (a hot root under a tongue's tip), and the
    // flames stand about a frame's size on screen.
    heat.mips = Mips::None;
    let heat = scene.add_texture(heat);
    let mut flow = texture(64, og_render::ColorSpace::Linear, |u, v| {
        let a = tiling_fbm(u, v, 4, 0x3a);
        let b = tiling_fbm(u, v, 4, 0x3b);
        [(a * 255.0) as u8, (b * 255.0) as u8, 128, 255]
    });
    flow.mips = Mips::Generate;
    let flow = scene.add_texture(flow);
    let noise = effect_materials::noise_texture(scene, 4, 0x3c);
    scene.add_effect_material(EffectMaterial {
        main: EffectTexture {
            texture: Some(heat),
            channel: EffectChannel::Red,
            ..Default::default()
        },
        ramp: Some(ColorCurve::new(&[
            (0.0, [0.0, 0.0, 0.0, 0.0]),
            (0.16, [0.0, 0.0, 0.0, 0.0]),
            (0.28, [0.9, 0.08, 0.01, 1.0]),
            (0.5, [1.8, 0.4, 0.05, 1.0]),
            (0.8, [2.6, 1.2, 0.25, 1.0]),
            (1.0, [3.4, 2.7, 1.6, 1.0]),
        ])),
        flow: Some(Flow {
            map: EffectTexture {
                texture: Some(flow),
                uv: EffectUv {
                    tile: Vec2::new(1.0, 0.5),
                    scroll: Vec2::new(0.0, 1.2),
                    ..Default::default()
                },
                ..Default::default()
            },
            // A frame of the 4 × 4 sheet is 0.25 across: a push past a few
            // hundredths would sample the next frame.
            strength: 0.05,
            speed: 0.7,
        }),
        erosion: Some(Erosion {
            noise: EffectTexture {
                texture: Some(noise),
                uv: EffectUv {
                    tile: Vec2::new(1.2, 0.8),
                    scroll: Vec2::new(0.0, 1.1),
                    ..Default::default()
                },
                channel: EffectChannel::Red,
            },
            amount: EffectValue::Curve(og_render::Curve::new(&[
                (0.0, 0.06),
                (0.5, 0.25),
                (1.0, 0.65),
            ])),
            softness: 0.1,
            edge: 0.0,
            edge_color: [0.0; 4],
        }),
        intensity: EffectValue::Constant(0.6),
        // Gone by three quarters of its life: what the erosion leaves of
        // an old flame is its hot root, which would float up as a shard.
        alpha: EffectValue::Curve(og_render::Curve::new(&[
            (0.0, 1.0),
            (0.4, 1.0),
            (0.72, 0.0),
        ])),
        soft_depth: 0.15,
        ..Default::default()
    })
}
