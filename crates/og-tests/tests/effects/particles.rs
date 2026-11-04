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
    // (tests/evidence/debt3/particle-sigsegv/NOTES.md; fixed upstream in
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
fn a_600_tick_gpu_particle_run_is_deterministic() {
    particles_are_deterministic(ParticleSimulation::Auto);
}

#[test]
fn a_600_tick_cpu_particle_run_is_deterministic() {
    particles_are_deterministic(ParticleSimulation::Cpu);
}

/// A fountain without noise or collision.
fn plain_fountain(scene: &mut RenderScene) -> og_render::EmitterHandle {
    scene.add_emitter(ParticleEmitter {
        max_particles: 512,
        shape: EmitterShape::Cone {
            angle_degrees: 25.0,
            radius: 0.2,
        },
        rate: 60.0,
        bursts: vec![Burst::once(0.0, 40)],
        lifetime: [1.0, 1.0],
        speed: [2.0, 3.0],
        ..Default::default()
    })
}

/// The GPU and CPU paths spawn the same particles (same random values,
/// same schedule) and integrate them the same way within float error.
#[test]
fn gpu_and_cpu_paths_simulate_the_same_particles() {
    let mut states = Vec::new();
    for path in [ParticleSimulation::Auto, ParticleSimulation::Cpu] {
        let mut scene = RenderScene::new();
        let e = plain_fountain(&mut scene);
        scene
            .particles
            .push(ParticleSystem::new(7, e, Mat4::IDENTITY, 42, 0));
        let desc = RenderDesc {
            particles: path,
            ..small()
        };
        let (_, _, r) = run(&desc, &mut scene, 50, |s, f| fx::tick_all(s, f as u64));
        states.push(r.read_particles(7).expect("system 7"));
    }
    let (gpu, cpu) = (&states[0], &states[1]);
    let alive = |s: &[og_render::ParticleState]| s.iter().filter(|p| p.alive()).count();
    println!("alive: gpu {} cpu {}", alive(gpu), alive(cpu));
    assert_eq!(alive(gpu), alive(cpu));
    let worst = gpu
        .iter()
        .zip(cpu)
        .filter(|(g, _)| g.alive())
        .map(|(g, c)| Vec3::from_slice(&g.pos_age[..3]).distance(Vec3::from_slice(&c.pos_age[..3])))
        .fold(0.0f32, f32::max);
    println!("largest position difference: {worst}");
    assert!(worst < 1e-3, "GPU and CPU particles differ by {worst}");
}

/// Spawn counts follow the schedule: a 40-particle burst at start, then 60
/// per second, each living exactly a second.
#[test]
fn spawn_counts_follow_rate_bursts_and_lifetime() {
    let mut scene = RenderScene::new();
    let e = plain_fountain(&mut scene);
    // A second emitter whose rate does not divide the tick rate: after 30
    // ticks (0.5 s) it owes 22.5 particles, so the count must round down.
    let slow = scene.add_emitter(ParticleEmitter {
        max_particles: 512,
        rate: 45.0,
        lifetime: [1.0, 1.0],
        speed: [2.0, 3.0],
        ..Default::default()
    });
    scene
        .particles
        .push(ParticleSystem::new(1, e, Mat4::IDENTITY, 3, 0));
    scene
        .particles
        .push(ParticleSystem::new(2, slow, Mat4::IDENTITY, 4, 0));
    let desc = RenderDesc {
        particles: ParticleSimulation::Cpu,
        ..small()
    };
    let (_, stats, mut r) = run(&desc, &mut scene, 30, |s, f| fx::tick_all(s, f as u64));
    // Ticks 0..=29: the burst plus 30 continuous particles, and 22 (not 23)
    // of the slow emitter.
    assert_eq!(stats.particles_cpu, 40 + 30 + 22);
    assert_eq!(stats.particle_ticks, 2);
    // At tick 150 (a jump: the last lifetime is replayed) one second of
    // continuous particles lives (60 + 45), plus the burst of the cycle at
    // 2 s (the emitter loops every second, bursts included).
    fx::tick_all(&mut scene, 150);
    let stats = r.render(&scene, &RenderTarget::Offscreen).expect("render");
    assert!(
        (144..=146).contains(&stats.particles_cpu),
        "{} alive at tick 150",
        stats.particles_cpu
    );
    // Stopping emission: nothing alive a lifetime later.
    scene.particles[0].stop_tick = Some(160);
    scene.particles[1].stop_tick = Some(160);
    fx::tick_all(&mut scene, 230);
    let stats = r.render(&scene, &RenderTarget::Offscreen).expect("render");
    assert_eq!(stats.particles_cpu, 0);
}

/// Soft particles fade where they meet geometry: a big quad crossing a wall
/// is darker along the intersection with soft particles than without.
#[test]
fn soft_particles_fade_against_the_scene() {
    let build = |soft: f32| {
        let mut scene = RenderScene::new();
        let wall = scene.add_mesh(shapes::cuboid(Vec3::new(6.0, 6.0, 0.2)));
        let black = scene.add_material(Material {
            base_color: [0.0, 0.0, 0.0, 1.0],
            ..Default::default()
        });
        scene.add_instance(wall, black, Mat4::IDENTITY);
        let e = scene.add_emitter(ParticleEmitter {
            max_particles: 1,
            shape: EmitterShape::Point,
            rate: 0.0,
            bursts: vec![Burst::once(0.0, 1)],
            lifetime: [100.0, 100.0],
            speed: [0.0, 0.0],
            size: [3.0, 3.0],
            gravity: Vec3::ZERO,
            color_over_life: ColorCurve::constant([1.0; 4]),
            soft_distance: soft,
            ..Default::default()
        });
        scene.particles.push(ParticleSystem::new(
            1,
            e,
            Mat4::from_translation(Vec3::new(0.0, 0.0, 0.15)),
            1,
            0,
        ));
        scene.light.intensity = 0.0;
        scene.environment.intensity = 0.0;
        scene.environment.background = Background::Color(Vec3::ZERO);
        scene.post.tonemap = Tonemap::None;
        scene.camera = Camera {
            position: Vec3::new(0.0, 0.0, 5.0),
            target: Vec3::ZERO,
            ..Default::default()
        };
        scene
    };
    let (hard, _) = frame(&small(), &build(0.0));
    let (soft, _) = frame(&small(), &build(1.0));
    let c = (160, 90);
    println!(
        "centre luma: hard {} soft {}",
        luma(px(&hard, c.0, c.1)),
        luma(px(&soft, c.0, c.1))
    );
    assert!(luma(px(&soft, c.0, c.1)) < 0.5 * luma(px(&hard, c.0, c.1)));
}

/// Particles with collision stay above the ground (compute); without it
/// they fall through.
#[test]
fn particles_bounce_off_the_depth_buffer() {
    let lowest = |collide: bool| {
        let mut scene = collision_scene(collide);
        let (img, _, r) = run(&small(), &mut scene, 90, |s, f| fx::tick_all(s, f as u64));
        save(&format!("collision-{collide}"), &img);
        r.read_particles(1)
            .expect("system")
            .iter()
            .filter(|p| p.alive())
            .map(|p| p.pos_age[1])
            .fold(f32::MAX, f32::min)
    };
    let (with, without) = (lowest(true), lowest(false));
    println!("lowest particle: with collision {with}, without {without}");
    assert!(with > -0.3, "collision let a particle sink to {with}");
    assert!(without < -1.0, "without collision the lowest is {without}");
}

/// The CPU path (`ParticleSimulation::Cpu`) bounces
/// particles off the scene's colliders: a ground plane, a flat height
/// field or a box under the fountain each keep them above the ground,
/// some on their way back up; without colliders they fall through and the
/// system is counted as skipped.
#[test]
fn cpu_particles_bounce_off_the_scene_colliders() {
    let desc = RenderDesc {
        particles: ParticleSimulation::Cpu,
        ..small()
    };
    let run_with = |colliders: Vec<og_render::SceneCollider>| {
        let mut scene = collision_scene(true);
        scene.colliders = colliders;
        let (_, stats, r) = run(&desc, &mut scene, 90, |s, f| fx::tick_all(s, f as u64));
        let live: Vec<_> = r
            .read_particles(1)
            .expect("system")
            .into_iter()
            .filter(|p| p.alive())
            .collect();
        let lowest = live.iter().map(|p| p.pos_age[1]).fold(f32::MAX, f32::min);
        // Rising again below the emitter: bounced.
        let bounced = live
            .iter()
            .filter(|p| p.vel_life[1] > 0.0 && p.pos_age[1] < 1.0)
            .count();
        (lowest, bounced, stats.particle_collision_skipped)
    };
    let (without, _, skipped) = run_with(Vec::new());
    assert!(without < -1.0, "without colliders the lowest is {without}");
    assert_eq!(skipped, 1);
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
    for (name, c) in [("plane", plane), ("height field", field), ("box", block)] {
        let (lowest, bounced, skipped) = run_with(vec![c]);
        println!("{name}: lowest {lowest}, {bounced} bounced");
        assert_eq!(skipped, 0, "{name}");
        assert!(lowest > -0.1, "{name}: a particle sank to {lowest}");
        assert!(bounced > 5, "{name}: {bounced} bounced");
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
    scene.particles.push(ParticleSystem::new(
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

/// Sprite-mode emitters draw through the sprite batch on their layer:
/// simulated in compute where the adapter has it (their range one draw of
/// the batch), else on the CPU as sprites.
#[test]
fn two_d_emitters_draw_as_sprites() {
    let mut scene = RenderScene::new();
    let e = scene.add_emitter(ParticleEmitter {
        render: ParticleRender::Sprite { layer: 2 },
        rate: 100.0,
        lifetime: [1.0, 1.0],
        ..Default::default()
    });
    scene
        .particles
        .push(ParticleSystem::new(1, e, Mat4::IDENTITY, 1, 0));
    scene.camera = Camera::orthographic_2d(Vec2::ZERO, 6.0);
    scene.environment.background = Background::Color(Vec3::ZERO);
    let (img, stats, _) = run(&small(), &mut scene, 30, |s, f| fx::tick_all(s, f as u64));
    assert_eq!(stats.particles_cpu, 0, "simulated in compute");
    assert!(stats.sprite_draws >= 1);
    assert!(img.pixels.as_chunks::<4>().0.iter().any(|p| p[0] > 100));
}

// ---------------------------------------------------------------------------
// Trails and beams.
// ---------------------------------------------------------------------------

/// A trail grows while its head moves, empties a lifetime after it stops,
/// and restarts when time runs backwards.
#[test]
fn trails_remember_the_path_for_their_lifetime() {
    let mut scene = RenderScene::new();
    let style = scene.add_trail_style(TrailStyle {
        lifetime: 0.25,
        width: Curve::linear(0.4, 0.1),
        ..Default::default()
    });
    scene.trails.push(Trail {
        id: 1,
        style,
        position: Vec3::ZERO,
        tick: 0,
    });
    scene.camera = Camera::orthographic_2d(Vec2::ZERO, 6.0);
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.post.tonemap = Tonemap::None;
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
        scene.trails[0].tick = t;
        scene.trails[0].position = Vec3::new(t as f32 * 0.2 - 1.0, 0.0, 0.0);
        moving = drawn(&scene);
    }
    // Moving 0.2 a tick for nine ticks: 1.8 units of path behind the head,
    // 54 pixels at 30 a unit (inside the 15-tick lifetime).
    assert!(
        moving.0 > 150 && moving.1 >= 45,
        "{moving:?} lit pixels and span while moving"
    );
    // Standing still for longer than the lifetime: the trail collapses.
    scene.trails[0].tick = 40;
    assert_eq!(drawn(&scene).0, 0, "a resting trail draws nothing");
    // Backwards in time: restarts with a single point.
    scene.trails[0].tick = 5;
    scene.trails[0].position = Vec3::new(2.0, 0.0, 0.0);
    assert_eq!(drawn(&scene).0, 0, "a trail sent back in time starts again");
}

/// A textured beam scrolls with the scene's tick: only its pixels change.
#[test]
fn beams_scroll_with_the_tick() {
    let mut scene = RenderScene::new();
    let tex = fx::beam_texture(&mut scene);
    scene.beams.push(Beam {
        start: Vec3::new(-2.5, 0.0, 0.0),
        end: Vec3::new(2.5, 0.0, 0.0),
        width: 0.6,
        texture: Some(tex),
        tile: 1.0,
        scroll: 1.7,
        ..Default::default()
    });
    scene.camera = Camera::orthographic_2d(Vec2::ZERO, 3.0);
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.post.tonemap = Tonemap::None;
    let (a, stats) = frame(&small(), &scene);
    scene.tick = 9;
    let (b, _) = frame(&small(), &scene);
    assert_eq!(stats.strips, 1);
    let changed = differing(&a, &b, 6.0);
    println!("{changed} pixels changed as the beam scrolled");
    assert!(changed > 500);
    // Far from the beam nothing changes.
    assert_eq!(px(&a, 160, 10), px(&b, 160, 10));
}
