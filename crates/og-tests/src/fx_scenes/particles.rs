//! Campfire particles, trails, ribbons and beams.

use super::*;

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
            ..Default::default()
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
        scene.effects.particles.push(ParticleSystem::new(
            id + k as u64,
            e,
            Mat4::from_translation(Vec3::Y * lift) * t,
            0x5eed + id + k as u64,
            0,
        ));
    }
}

pub(super) fn logs(scene: &mut RenderScene, at: Vec3) {
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
    scene.lighting.light = DirectionalLight {
        direction_to_light: moon,
        color: Vec3::new(0.5, 0.6, 1.0),
        intensity: 0.25,
        ..Default::default()
    };
    scene.lighting.environment = Environment {
        sky: ProceduralSky {
            sun_direction: moon,
            sun_color: Vec3::new(0.3, 0.35, 0.5),
            look: None,
        },
        intensity: 0.06,
        background: Background::Color(Vec3::new(0.008, 0.01, 0.022)),
        ..Environment::default()
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
    scene.effects.particles.push(ParticleSystem::new(
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
        .effects
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
    scene.effects.particles.push(ParticleSystem::new(
        40,
        burst,
        Mat4::from_translation(Vec3::new(0.9, 0.6, 1.6)) * Mat4::from_rotation_z(-0.6),
        5,
        0,
    ));
    scene.lighting.lights = vec![
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
    scene.look.post.bloom = Some(Bloom {
        threshold: 1.0,
        knee: 0.6,
        intensity: 0.7,
    });
    scene.frame.camera = Camera {
        position: Vec3::new(0.4, 2.6, 6.5),
        target: Vec3::new(0.0, 0.9, -0.5),
        exposure: Exposure::linear(1.4),
        ..Default::default()
    };
    scene
}

/// R7's particle looks under a low sun: lit smoke (left), lit mesh debris
/// tumbling out of a crater that bounces on the ground and kicks a dust
/// puff at each bounce (a collision sub-emitter, so the CPU path), and a
/// heat haze over a glowing brazier that bends the checker wall behind it.
pub fn particles_lit_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    terrain(&mut scene, 40.0);
    let smoke_tex = smoke_texture(&mut scene);
    // The checker wall the haze bends.
    let checker = scene.add_texture(shapes::checker(512, 16));
    let wall = scene.add_mesh(shapes::cuboid(Vec3::new(9.0, 3.5, 0.3)));
    let tiles = scene.add_material(Material {
        base_color_texture: Some(checker),
        roughness: 0.8,
        ..Default::default()
    });
    scene.add_instance(
        wall,
        tiles,
        Mat4::from_translation(Vec3::new(0.0, 1.75, -2.6)),
    );
    // Lit smoke: an alpha puff column the sun shades from the right.
    let smoke = scene.add_emitter(ParticleEmitter {
        max_particles: 200,
        shape: EmitterShape::Cone {
            angle_degrees: 10.0,
            radius: 0.3,
        },
        rate: 30.0,
        lifetime: [2.5, 3.2],
        speed: [0.5, 0.8],
        size: [0.7, 1.0],
        size_over_life: Curve::new(&[(0.0, 0.5), (1.0, 1.6)]),
        color_over_life: ColorCurve::new(&[
            (0.0, [1.0, 1.0, 1.0, 0.0]),
            (0.15, [1.0, 1.0, 1.0, 0.9]),
            (1.0, [1.0, 1.0, 1.0, 0.0]),
        ]),
        color: [0.85, 0.85, 0.85, 1.0],
        rotation_degrees: [0.0, 360.0],
        gravity: Vec3::new(0.0, 0.3, 0.0),
        drag: 0.4,
        texture: Some(smoke_tex),
        blend: ParticleBlend::Alpha,
        lit: true,
        ..Default::default()
    });
    scene.effects.particles.push(ParticleSystem::new(
        50,
        smoke,
        Mat4::from_translation(Vec3::new(-2.8, 0.0, -0.6)),
        11,
        0,
    ));
    // Debris: lit tetrahedra thrown up, bouncing; dust at each bounce.
    let dust = scene.add_emitter(ParticleEmitter {
        max_particles: 400,
        shape: EmitterShape::Point,
        spread_degrees: 80.0,
        rate: 0.0,
        lifetime: [0.5, 0.8],
        speed: [0.2, 0.5],
        size: [0.15, 0.25],
        color_over_life: ColorCurve::new(&[
            (0.0, [1.0, 1.0, 1.0, 0.7]),
            (1.0, [1.0, 1.0, 1.0, 0.0]),
        ]),
        // Linear: a dry earth tan (sRGB about 140, 120, 95).
        color: [0.26, 0.19, 0.12, 1.0],
        gravity: Vec3::new(0.0, 0.2, 0.0),
        texture: Some(smoke_tex),
        blend: ParticleBlend::Alpha,
        lit: true,
        ..Default::default()
    });
    let shard = scene.add_mesh(shapes::tetrahedron());
    let debris = scene.add_emitter(ParticleEmitter {
        max_particles: 120,
        shape: EmitterShape::Point,
        spread_degrees: 35.0,
        rate: 0.0,
        // Forty every second (the emitter loops its one-second cycle).
        bursts: vec![Burst::once(0.0, 40)],
        duration: 1.0,
        lifetime: [1.6, 2.0],
        speed: [3.5, 5.0],
        size: [0.12, 0.22],
        color_over_life: ColorCurve::new(&[(0.0, [1.0; 4]), (1.0, [1.0; 4])]),
        // Linear: a rock brown (sRGB about 120, 85, 60), as dark as the dirt.
        color: [0.19, 0.09, 0.045, 1.0],
        rotation_degrees: [0.0, 360.0],
        angular_velocity_degrees: [-540.0, 540.0],
        gravity: Vec3::new(0.0, -9.8, 0.0),
        collision: Some(ParticleCollision {
            bounce: 0.35,
            friction: 0.4,
            ..Default::default()
        }),
        render: ParticleRender::Mesh { mesh: shard },
        lit: true,
        sub_emitters: vec![og_render::SubEmitter {
            trigger: og_render::SubTrigger::Event(og_render::ParticleEventKind::Collision),
            emitter: dust,
            count: 2,
            inherit_velocity: 0.1,
        }],
        ..Default::default()
    });
    scene.effects.particles.push(ParticleSystem::new(
        51,
        debris,
        Mat4::from_translation(Vec3::new(0.0, 0.1, 0.4)),
        23,
        0,
    ));
    scene.world.colliders.push(og_render::SceneCollider::Plane {
        normal: Vec3::Y,
        offset: 0.0,
    });
    // A brazier and the haze over it.
    let brazier = scene.add_mesh(shapes::cuboid(Vec3::new(0.7, 0.5, 0.7)));
    let coals = scene.add_material(Material {
        base_color: [0.1, 0.08, 0.07, 1.0],
        emissive: [3.0, 0.9, 0.2],
        roughness: 0.9,
        ..Default::default()
    });
    scene.add_instance(
        brazier,
        coals,
        Mat4::from_translation(Vec3::new(2.8, 0.25, -1.2)),
    );
    let haze = scene.add_emitter(ParticleEmitter {
        max_particles: 120,
        shape: EmitterShape::Box {
            half_extents: Vec3::new(0.3, 0.05, 0.3),
        },
        rate: 30.0,
        lifetime: [1.4, 1.8],
        speed: [0.6, 0.9],
        size: [0.6, 0.9],
        size_over_life: Curve::new(&[(0.0, 0.6), (1.0, 1.4)]),
        color_over_life: ColorCurve::new(&[
            (0.0, [1.0, 1.0, 1.0, 0.0]),
            (0.25, [1.0, 1.0, 1.0, 1.0]),
            (1.0, [1.0, 1.0, 1.0, 0.0]),
        ]),
        rotation_degrees: [0.0, 360.0],
        gravity: Vec3::ZERO,
        // About 4 px at 720 rows: a waver, not a lens.
        distortion: 0.006,
        ..Default::default()
    });
    scene.effects.particles.push(ParticleSystem::new(
        52,
        haze,
        Mat4::from_translation(Vec3::new(2.8, 0.55, -1.2)),
        31,
        0,
    ));
    let sun = Vec3::new(0.85, 0.4, 0.3);
    scene.lighting.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        cast_shadows: true,
        ..Default::default()
    };
    scene.lighting.environment.sky.sun_direction = sun;
    // A low, clear sun over a dimmer sky, so the lit particles' sun side
    // reads against the sky's fill.
    scene.lighting.environment.intensity = 0.5;
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 2.4, 6.4),
        target: Vec3::new(0.0, 1.0, -0.8),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene
}

/// Ticks the scene's clocks to the frame (60 ticks per second).
pub fn tick_update(scene: &mut RenderScene, frame: u32) {
    tick_all(scene, frame as u64);
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
        scene.effects.trails.push(Trail {
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
    scene.effects.ribbons.push(Ribbon {
        points,
        texture: None,
        uv: RibbonUv::Stretch,
        blend: ParticleBlend::Additive,
        effect: None,
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
    scene.effects.beams.push(Beam {
        start: Vec3::new(2.2, 2.0, -2.5),
        end: Vec3::new(4.6, 2.0, -2.5),
        width: 0.22,
        color: [0.5, 1.4, 2.8, 1.0],
        texture: Some(beam_tex),
        tile: 1.2,
        scroll: 2.0,
        ..Default::default()
    });
    scene.effects.beams.push(Beam {
        start: Vec3::new(2.2, 0.9, -2.5),
        end: Vec3::new(4.6, 0.9, -2.5),
        width: 0.06,
        color: [6.0, 5.0, 9.0, 1.0],
        segments: 24,
        jitter: 0.18,
        seed: 9,
        ..Default::default()
    });
    scene.look.post.bloom = Some(Bloom::default());
    scene.frame.camera = Camera {
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
    let n = scene.effects.trails.len();
    for i in 0..n {
        let p = wisp_at(i, tick);
        scene.effects.trails[i].position = p;
        // The orbs are the last three instances before the ribbon's pylons.
        let k = 1 + i;
        scene.objects.instances[k].transform =
            Mat4::from_translation(p) * Mat4::from_scale(Vec3::splat(0.18));
    }
}
