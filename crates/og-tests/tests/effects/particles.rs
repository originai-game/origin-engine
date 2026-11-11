//! Particle simulation, trails and beam frame tests.

use super::*;

// ---------------------------------------------------------------------------
// Particles.
// ---------------------------------------------------------------------------

/// The 600-tick determinism run for one simulation path: two identical
/// runs give byte-identical frames, and jumping straight to tick 600 (a
/// replay of the last lifetime) gives the same frame as stepping every tick.
fn particles_are_deterministic(path: ParticleSimulation) {
    let desc = RenderDesc {
        particles: path,
        ..small()
    };
    let stepped = |scene: &mut RenderScene, f: u32| fx::tick_all(scene, f as u64);
    // The two runs one after the other on this thread, never at once on
    // two: the Vulkan loader from 1.3.290 to 1.4.344 (Debian trixie's
    // 1.4.309, CI's lavapipe) frees an instance's unused drivers in its
    // first vkEnumeratePhysicalDevices under one lock while another
    // thread's device walks that list under another (debug names, device
    // function lookups), and the walk segfaults on the freed entry
    // (the former test sample fixed upstream in
    // 1.4.345). Each renderer is dropped before the next is made.
    let (a, sa, _) = run(&desc, &mut fx::particles_scene(), 601, stepped);
    let (b, _, _) = run(&desc, &mut fx::particles_scene(), 601, stepped);
    let (c, _, _) = run(&desc, &mut fx::particles_scene(), 1, |s, _| {
        fx::tick_all(s, 600)
    });
    save(&format!("particles-600-{path:?}"), &a);
    assert!(sa.particle_systems >= 6, "{} systems", sa.particle_systems);
    let lit = a
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[0] > 200)
        .count();
    assert!(lit > 200, "the particles drew only {lit} bright pixels");
    assert_eq!(a.pixels, b.pixels, "two 600-tick runs differ ({path:?})");
    assert_eq!(
        a.pixels, c.pixels,
        "a jump to tick 600 differs from stepping there ({path:?})"
    );
}

#[test]
#[ignore = "requires GPU"]
fn a_600_tick_gpu_particle_run_is_deterministic() {
    particles_are_deterministic(ParticleSimulation::Auto);
}

#[test]
#[ignore = "requires GPU"]
fn a_600_tick_cpu_particle_run_is_deterministic() {
    particles_are_deterministic(ParticleSimulation::Cpu);
}

/// Compute particles meet the scene's colliders where the emitter names
/// them (`collision ... colliders`), and always in sprite mode, with
/// nothing read back to the CPU: with no ground drawn (so no depth to
/// bounce off) a plane, a height field or a box under the fountain keeps
/// them above it, some on their way back up. Before, a sprite-mode
/// fountain passed through every collider and was counted as skipped,
/// and a billboard one met only what the depth prepass drew.
#[test]
#[ignore = "requires GPU"]
fn compute_particles_bounce_off_the_scene_colliders() {
    let run_with = |sprite: bool, colliders: Vec<og_render::SceneCollider>| {
        let mut scene = collision_scene(true);
        // Nothing drawn under the fountain: only the colliders are there.
        scene.instances.clear();
        let e = scene.effects.particles[0].emitter;
        let mut emitter = scene.emitter(e).expect("emitter").clone();
        emitter.collision = Some(ParticleCollision {
            against: og_render::CollisionTarget::Colliders,
            ..Default::default()
        });
        if sprite {
            emitter.render = ParticleRender::Sprite { layer: 0 };
            // A sprite-mode emitter meets the colliders whatever it names.
            emitter.collision = Some(ParticleCollision::default());
            scene.camera = Camera::orthographic_2d(Vec2::new(0.0, 0.5), 6.0);
        }
        scene.set_emitter(e, emitter);
        scene.world.colliders = colliders;
        let (_, stats, r) = run(&small(), &mut scene, 90, |s, f| fx::tick_all(s, f as u64));
        assert_eq!(stats.particles_cpu, 0, "simulated in compute");
        let live: Vec<_> = r
            .read_particles(1)
            .expect("system")
            .into_iter()
            .filter(|p| p.alive())
            .collect();
        assert!(live.len() > 50, "{} alive", live.len());
        let lowest = live.iter().map(|p| p.pos_age[1]).fold(f32::MAX, f32::min);
        let bounced = live
            .iter()
            .filter(|p| p.vel_life[1] > 0.0 && p.pos_age[1] < 1.0)
            .count();
        (lowest, bounced, stats.particle_collision_skipped)
    };
    let plane = og_render::SceneCollider::Plane {
        normal: Vec3::Y,
        offset: 0.0,
    };
    let field = og_render::SceneCollider::HeightField {
        data: og_render::TerrainData::flat(Vec2::splat(40.0), 9, 9),
        transform: Mat4::IDENTITY,
    };
    let block = og_render::SceneCollider::Box {
        transform: Mat4::from_translation(Vec3::new(0.0, -1.0, 0.0)),
        half_extents: Vec3::new(10.0, 1.0, 10.0),
    };
    for sprite in [false, true] {
        let (without, _, skipped) = run_with(sprite, Vec::new());
        assert!(
            without < -1.0,
            "sprite {sprite}: without colliders the lowest is {without}"
        );
        assert_eq!(skipped, 1, "sprite {sprite}: nothing to meet");
        for (name, c) in [("plane", &plane), ("height field", &field), ("box", &block)] {
            let (lowest, bounced, skipped) = run_with(sprite, vec![c.clone()]);
            println!("sprite {sprite}, {name}: lowest {lowest}, {bounced} bounced");
            assert_eq!(skipped, 0, "sprite {sprite}, {name}");
            assert!(
                lowest > -0.1,
                "sprite {sprite}, {name}: a particle sank to {lowest}"
            );
            assert!(bounced > 5, "sprite {sprite}, {name}: {bounced} bounced");
        }
    }
}

fn collision_scene(collide: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    let ground = scene.add_mesh(shapes::plane(20.0, 1.0));
    let m = scene.add_material(Material::default());
    scene.add_instance(ground, m, Mat4::IDENTITY);
    let e = scene.add_emitter(ParticleEmitter {
        max_particles: 256,
        shape: EmitterShape::Point,
        spread_degrees: 40.0,
        rate: 80.0,
        lifetime: [2.5, 2.5],
        speed: [1.0, 2.0],
        gravity: Vec3::new(0.0, -9.8, 0.0),
        collision: collide.then(ParticleCollision::default),
        ..Default::default()
    });
    scene.effects.particles.push(ParticleSystem::new(
        1,
        e,
        Mat4::from_translation(Vec3::new(0.0, 1.5, 0.0)),
        1,
        0,
    ));
    scene.camera = Camera {
        position: Vec3::new(0.0, 3.0, 6.0),
        target: Vec3::ZERO,
        ..Default::default()
    };
    scene
}

// ---------------------------------------------------------------------------
// Trails and beams.
// ---------------------------------------------------------------------------

/// A trail grows while its head moves, empties a lifetime after it stops,
/// and restarts when time runs backwards.
#[test]
#[ignore = "requires GPU"]
fn trails_remember_the_path_for_their_lifetime() {
    let mut scene = RenderScene::new();
    let style = scene.add_trail_style(TrailStyle {
        lifetime: 0.25,
        width: Curve::linear(0.4, 0.1),
        ..Default::default()
    });
    scene.effects.trails.push(Trail {
        id: 1,
        style,
        position: Vec3::ZERO,
        tick: 0,
    });
    scene.camera = Camera::orthographic_2d(Vec2::ZERO, 6.0);
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    let mut r = Renderer::new_blocking(&small()).expect("renderer");
    // The trail's lit pixels: how many, and the columns they span. The
    // history lives on the GPU, so what it draws is the evidence.
    let mut drawn = |scene: &RenderScene| {
        r.render(scene, &RenderTarget::Offscreen).expect("render");
        let img = r.read_offscreen().expect("capture");
        let lit: Vec<u32> = (0..img.width * img.height)
            .filter(|&i| luma(px(&img, i % img.width, i / img.width)) > 30.0)
            .map(|i| i % img.width)
            .collect();
        let span = lit
            .iter()
            .max()
            .zip(lit.iter().min())
            .map_or(0, |(a, b)| a - b);
        (lit.len(), span)
    };
    let mut moving = (0, 0);
    for t in 0..10u64 {
        scene.effects.trails[0].tick = t;
        scene.effects.trails[0].position = Vec3::new(t as f32 * 0.2 - 1.0, 0.0, 0.0);
        moving = drawn(&scene);
    }
    // Moving 0.2 a tick for nine ticks: 1.8 units of path behind the head,
    // 54 pixels at 30 a unit (inside the 15-tick lifetime).
    assert!(
        moving.0 > 150 && moving.1 >= 45,
        "{moving:?} lit pixels and span while moving"
    );
    // Standing still for longer than the lifetime: the trail collapses.
    scene.effects.trails[0].tick = 40;
    assert_eq!(drawn(&scene).0, 0, "a resting trail draws nothing");
    // Backwards in time: restarts with a single point.
    scene.effects.trails[0].tick = 5;
    scene.effects.trails[0].position = Vec3::new(2.0, 0.0, 0.0);
    assert_eq!(drawn(&scene).0, 0, "a trail sent back in time starts again");
}

/// An emitter using every expressive module at once: a closed spline
/// shape, speed, spin and rate curves, two attractors (one with a radius,
/// one pushing), a curl field (or a grid field), inherited velocity, on an
/// emitter moving across the frame.
fn expressive_scene(grid: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    let mut points = [Vec3::ZERO; og_render::MAX_SPLINE_POINTS];
    for (i, p) in points.iter_mut().take(5).enumerate() {
        let a = i as f32 / 5.0 * std::f32::consts::TAU;
        *p = Vec3::new(a.cos() * 1.2, 0.3 * (i % 2) as f32, a.sin() * 0.8);
    }
    let field = if grid {
        let n = 4u32;
        og_render::VectorField::Grid {
            strength: 2.0,
            size: 3.0,
            cells: n,
            vectors: (0..n * n * n)
                .map(|i| Vec3::new((i % 3) as f32 - 1.0, 0.5, ((i / 7) % 3) as f32 - 1.0))
                .collect(),
        }
    } else {
        og_render::VectorField::Curl {
            strength: 4.0,
            frequency: 0.7,
            scroll: 0.3,
        }
    };
    let e = scene.add_emitter(ParticleEmitter {
        max_particles: 512,
        shape: EmitterShape::Spline {
            points,
            count: 5,
            closed: true,
        },
        rate: 120.0,
        rate_over_time: Some(Curve::new(&[(0.0, 0.2), (0.5, 1.5), (1.0, 0.5)])),
        duration: 2.0,
        lifetime: [1.0, 1.8],
        speed: [0.5, 1.5],
        angular_velocity_degrees: [90.0, 270.0],
        speed_over_life: Some(Curve::new(&[(0.0, 1.5), (0.7, 0.3), (1.0, 0.1)])),
        spin_over_life: Some(Curve::new(&[(0.0, 1.0), (1.0, 0.0)])),
        attractors: vec![
            og_render::Attractor {
                position: Vec3::new(0.0, 1.5, 0.0),
                strength: 6.0,
                radius: 3.0,
            },
            og_render::Attractor {
                position: Vec3::new(1.0, 0.0, 0.0),
                strength: -2.0,
                radius: 0.0,
            },
        ],
        field: Some(field),
        inherit_velocity: 0.8,
        gravity: Vec3::new(0.0, -0.5, 0.0),
        drag: 0.3,
        ..Default::default()
    });
    scene
        .effects
        .particles
        .push(ParticleSystem::new(1, e, Mat4::IDENTITY, 7, 0));
    scene.camera = Camera {
        position: Vec3::new(0.0, 2.0, 7.0),
        target: Vec3::new(0.0, 0.8, 0.0),
        ..Default::default()
    };
    scene
}

/// Compute and the CPU path agree on the expressive modules: after 90
/// ticks of a moving emitter, every slot alive on one is alive on the
/// other and each particle's position, velocity and age match to within
/// float noise (the CPU's og-core sines against the GPU's). An emitter
/// that dropped a module on one path (the attractors read in the wrong
/// space, the speed curve applied to the velocity, the grid read from the
/// wrong offset, the inherited velocity lost) differs by metres.
#[test]
#[ignore = "requires GPU"]
fn compute_and_cpu_agree_on_curves_attractors_fields_and_splines() {
    for grid in [false, true] {
        let slots = |path: ParticleSimulation| {
            let desc = RenderDesc {
                particles: path,
                ..small()
            };
            let mut scene = expressive_scene(grid);
            let (_, stats, r) = run(&desc, &mut scene, 90, |s, f| {
                // Across the frame at 1.5 m/s, the systems' clocks on.
                s.effects.particles[0].transform =
                    Mat4::from_translation(Vec3::new(-2.0 + f as f32 * 0.025, 0.5, 0.0));
                fx::tick_all(s, f as u64);
            });
            println!("{path:?}: {} simulated on the CPU", stats.particles_cpu);
            assert_eq!(stats.particles_cpu == 0, path == ParticleSimulation::Auto);
            r.read_particles(1).expect("system")
        };
        let gpu = slots(ParticleSimulation::Auto);
        let cpu = slots(ParticleSimulation::Cpu);
        assert_eq!(gpu.len(), cpu.len());
        let alive = cpu.iter().filter(|p| p.alive()).count();
        assert!(alive > 80, "grid {grid}: {alive} alive");
        let mut worst = 0.0f32;
        for (i, (g, c)) in gpu.iter().zip(&cpu).enumerate() {
            assert_eq!(g.alive(), c.alive(), "grid {grid}: slot {i}");
            if !c.alive() {
                continue;
            }
            let d = |a: [f32; 4], b: [f32; 4]| {
                Vec3::from_slice(&a[..3]).distance(Vec3::from_slice(&b[..3]))
            };
            worst = worst
                .max(d(g.pos_age, c.pos_age))
                .max(d(g.vel_life, c.vel_life) * 0.1);
            assert!(
                (g.pos_age[3] - c.pos_age[3]).abs() < 1e-4,
                "grid {grid}: slot {i} age"
            );
        }
        println!("grid {grid}: {alive} alive, worst difference {worst:.5} m");
        assert!(
            worst < 0.02,
            "grid {grid}: compute and CPU differ by {worst} m"
        );
    }
}
