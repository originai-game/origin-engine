//! Particle simulation, trails and beam frame tests.

use super::*;
use og_render::{ParticleAlign, SheetMode, SpriteSheet};

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

/// A system's tint multiplies its emitter's colour and its emission the
/// emitter's emission, on both simulation paths: the particles scene with
/// every system tinted and at half emission draws, pixel for pixel, what
/// it draws with each emitter's colour and emission multiplied by the
/// same, and not what it draws as it is.
#[test]
#[ignore = "requires GPU"]
fn a_systems_tint_and_emission_draw_as_its_emitters() {
    let (tint, emission) = (Vec3::new(1.0, 0.45, 0.2), 0.5);
    for path in [ParticleSimulation::Auto, ParticleSimulation::Cpu] {
        let desc = RenderDesc {
            particles: path,
            ..small()
        };
        let frame =
            |scene: &mut RenderScene| run(&desc, scene, 40, |s, f| fx::tick_all(s, f as u64)).0;
        let plain = frame(&mut fx::particles_scene());
        let mut tinted = fx::particles_scene();
        for sys in &mut tinted.effects.particles {
            sys.tint = tint;
            sys.emission = emission;
        }
        let tinted = frame(&mut tinted);
        let mut copied = fx::particles_scene();
        for i in 0..copied.effects.particles.len() {
            let mut e = copied
                .emitter(copied.effects.particles[i].emitter)
                .expect("emitter")
                .clone();
            for (c, t) in e.color.iter_mut().zip(tint.to_array()) {
                *c *= t;
            }
            e.emission *= emission;
            copied.effects.particles[i].emitter = copied.add_emitter(e);
        }
        let copied = frame(&mut copied);
        assert_ne!(
            plain.pixels, tinted.pixels,
            "{path:?}: tint and emission drew nothing"
        );
        assert_eq!(
            tinted.pixels, copied.pixels,
            "{path:?}: a system differs from its emitter with its tint and emission"
        );
    }
}

/// Two additive systems with coloured curves, one with the white spark
/// texture and one a soft dot; with `permute`, every colour they carry
/// (the emitter's and its curve's keys) has its channels turned
/// (r, g, b) -> (b, r, g).
fn hue_scene(permute: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    let turn = |c: [f32; 4]| if permute { [c[2], c[0], c[1], c[3]] } else { c };
    let spark = fx::spark_texture(&mut scene);
    for (k, texture) in [Some(spark), None].into_iter().enumerate() {
        let e = scene.add_emitter(ParticleEmitter {
            max_particles: 256,
            rate: 90.0,
            lifetime: [0.8, 1.2],
            speed: [1.5, 2.5],
            size: [0.12, 0.2],
            color_over_life: og_render::ColorCurve::new(&[
                (0.0, turn([1.0, 0.8, 0.3, 1.0])),
                (1.0, turn([0.9, 0.2, 0.05, 0.0])),
            ]),
            color: turn([3.0, 1.2, 0.4, 1.0]),
            texture,
            ..Default::default()
        });
        let at = Mat4::from_translation(Vec3::new(k as f32 * 2.0 - 1.0, 0.0, 0.0));
        scene
            .effects
            .particles
            .push(ParticleSystem::new(k as u64 + 1, e, at, 5 + k as u64, 0));
    }
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 1.0, 5.0),
        target: Vec3::new(0.0, 0.8, 0.0),
        ..Default::default()
    };
    scene
}

/// A system's hue turns the colour it draws about the grey axis: a third
/// of a turn carries red to green to blue, so the systems turned 2π/3 draw
/// what they draw with their colours' channels permuted, to within a step
/// of the 8-bit frame, on both simulation paths; unturned, they draw the
/// colours as written.
#[test]
#[ignore = "requires GPU"]
fn a_systems_hue_turns_the_colour_it_draws() {
    for path in [ParticleSimulation::Auto, ParticleSimulation::Cpu] {
        let desc = RenderDesc {
            particles: path,
            ..small()
        };
        let frame =
            |scene: &mut RenderScene| run(&desc, scene, 40, |s, f| fx::tick_all(s, f as u64)).0;
        let plain = frame(&mut hue_scene(false));
        let mut turned = hue_scene(false);
        for sys in &mut turned.effects.particles {
            sys.hue = std::f32::consts::TAU / 3.0;
        }
        let turned = frame(&mut turned);
        let permuted = frame(&mut hue_scene(true));
        let worst = turned
            .pixels
            .iter()
            .zip(&permuted.pixels)
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap_or(0);
        let moved = plain
            .pixels
            .iter()
            .zip(&turned.pixels)
            .filter(|(a, b)| a.abs_diff(**b) > 8)
            .count();
        save(&format!("hue-turned-{path:?}"), &turned);
        assert!(moved > 500, "{path:?}: the hue moved only {moved} channels");
        assert!(
            worst <= 1,
            "{path:?}: turned and permuted differ by {worst} in a channel"
        );
    }
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
        scene.objects.instances.clear();
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
            scene.frame.camera = Camera::orthographic_2d(Vec2::new(0.0, 0.5), 6.0);
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
    scene.frame.camera = Camera {
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
    scene.frame.camera = Camera::orthographic_2d(Vec2::ZERO, 6.0);
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
        wind: 0.6,
        ..Default::default()
    });
    scene.effects.particles.push(ParticleSystem {
        air: Vec3::new(3.0, 0.0, -1.5),
        ..ParticleSystem::new(1, e, Mat4::IDENTITY, 7, 0)
    });
    scene.frame.camera = Camera {
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
/// wrong offset, the inherited velocity or the air that carries them
/// lost) differs by metres.
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

/// A system no view can see costs no draw, and one whose particles reach
/// into view still draws: the particles scene with a fountain far behind
/// its camera draws the same frame with the same draw count as without
/// it; with one off the left edge whose particles a wind carries into
/// view, its frame differs (the wind is what brings them, so a bound that
/// leaves it out culls them).
#[test]
#[ignore = "requires GPU"]
fn a_system_out_of_view_is_not_drawn_and_one_blown_into_it_is() {
    let desc = small();
    let frame = |scene: &mut RenderScene| run(&desc, scene, 150, |s, f| fx::tick_all(s, f as u64));
    let with = |at: Vec3, air: Vec3| {
        let mut scene = fx::particles_scene();
        let e = scene.add_emitter(ParticleEmitter {
            max_particles: 400,
            rate: 150.0,
            lifetime: [2.0, 2.0],
            speed: [0.2, 0.5],
            size: [0.2, 0.3],
            color: [3.0, 2.0, 0.5, 1.0],
            // Only the wind carries them far.
            gravity: Vec3::ZERO,
            noise: og_render::ParticleNoise {
                strength: 0.0,
                ..Default::default()
            },
            drag: 3.0,
            wind: 1.0,
            space: og_render::SimulationSpace::World,
            ..Default::default()
        });
        let mut sys = ParticleSystem::new(900, e, Mat4::from_translation(at), 4, 0);
        sys.air = air;
        scene.effects.particles.push(sys);
        scene
    };
    let (plain, plain_stats, _) = frame(&mut fx::particles_scene());
    let (behind, behind_stats, _) = frame(&mut with(Vec3::new(0.0, 2.0, 40.0), Vec3::ZERO));
    assert_eq!(
        behind_stats.particle_systems,
        plain_stats.particle_systems + 1,
        "simulated"
    );
    assert_eq!(behind_stats.draw_calls, plain_stats.draw_calls, "not drawn");
    let differ = |a: &[u8], b: &[u8]| a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x != y).count();
    assert_eq!(
        differ(&behind.pixels, &plain.pixels),
        0,
        "pixels a system behind the camera changed"
    );
    let (blown, _, _) = frame(&mut with(
        Vec3::new(-13.0, 1.0, 0.0),
        Vec3::new(8.0, 0.0, 0.0),
    ));
    assert!(
        differ(&blown.pixels, &plain.pixels) > 100,
        "its particles blown into view: {} pixels",
        differ(&blown.pixels, &plain.pixels)
    );
}

/// A system's effect material given another texture draws the new one:
/// the renderer keeps a material's bind group only while its textures are
/// the same. Halfway through the run the material's red texture becomes a
/// blue one; the last frame is blue where the particles are, and with no
/// swap it stays red.
#[test]
#[ignore = "requires GPU"]
fn an_effect_material_given_another_texture_draws_it() {
    let solid = |scene: &mut RenderScene, c: [u8; 4]| {
        scene.add_texture(og_render::TextureData {
            width: 4,
            height: 4,
            rgba: c.repeat(16),
            ..Default::default()
        })
    };
    let material = |t| og_render::EffectMaterial {
        main: og_render::EffectTexture {
            texture: Some(t),
            ..Default::default()
        },
        ..Default::default()
    };
    let colour = |swap: bool| {
        let mut scene = RenderScene::new();
        let red = solid(&mut scene, [255, 0, 0, 255]);
        let blue = solid(&mut scene, [0, 0, 255, 255]);
        let m = scene.add_effect_material(material(red));
        let e = scene.add_emitter(ParticleEmitter {
            max_particles: 64,
            rate: 60.0,
            lifetime: [3.0, 3.0],
            speed: [0.0, 0.0],
            size: [1.0, 1.0],
            gravity: Vec3::ZERO,
            // Stacked, they keep the texture's colour.
            blend: og_render::ParticleBlend::Alpha,
            effect: Some(m),
            ..Default::default()
        });
        scene
            .effects
            .particles
            .push(ParticleSystem::new(1, e, Mat4::IDENTITY, 3, 0));
        scene.frame.camera = Camera {
            position: Vec3::new(0.0, 0.0, 4.0),
            target: Vec3::ZERO,
            ..Default::default()
        };
        let (img, _, _) = run(&small(), &mut scene, 40, |s, f| {
            if swap && f == 20 {
                assert!(s.set_effect_material(m, material(blue)));
            }
            fx::tick_all(s, f as u64);
        });
        px(&img, img.width / 2, img.height / 2)
    };
    let (kept, swapped) = (colour(false), colour(true));
    assert!(
        kept[0] > 100 && kept[2] < 40,
        "the red texture does not draw: {kept:?}"
    );
    assert!(
        swapped[2] > 100 && swapped[0] < 40,
        "the material drew its old texture after the swap: {swapped:?}"
    );
}

/// The frame quality's particle emission thins a system with many
/// particles alive and spares one with a few, on both simulation paths:
/// at 0.5 a busy fountain lights about half the pixels it lights at 1,
/// and a burst of six draws, pixel for pixel, what it draws at 1.
#[test]
#[ignore = "requires GPU"]
fn quality_thins_busy_systems_and_spares_a_handful() {
    // None: the stage without particles.
    let scene = |busy: Option<bool>, emission: f32| {
        let mut scene = RenderScene::new();
        let e = scene.add_emitter(ParticleEmitter {
            max_particles: 2048,
            rate: if busy == Some(true) { 400.0 } else { 0.0 },
            bursts: match busy {
                Some(false) => vec![og_render::Burst::once(0.0, 6)],
                _ => vec![],
            },
            lifetime: [2.0, 2.0],
            speed: [0.5, 2.0],
            spread_degrees: 180.0,
            size: if busy == Some(false) {
                [0.3, 0.4]
            } else {
                [0.05, 0.08]
            },
            gravity: Vec3::ZERO,
            drag: 1.5,
            color: [2.0, 1.0, 0.3, 1.0],
            ..Default::default()
        });
        scene
            .effects
            .particles
            .push(ParticleSystem::new(1, e, Mat4::IDENTITY, 7, 0));
        scene.frame.camera = Camera {
            position: Vec3::new(0.0, 0.0, 4.0),
            target: Vec3::ZERO,
            ..Default::default()
        };
        scene.frame.quality.particle_emission = emission;
        scene
    };
    for path in [ParticleSimulation::Auto, ParticleSimulation::Cpu] {
        let desc = RenderDesc {
            particles: path,
            ..small()
        };
        let draw = |busy: Option<bool>, emission: f32| {
            run(&desc, &mut scene(busy, emission), 30, |s, f| {
                fx::tick_all(s, f as u64)
            })
            .0
        };
        let empty = draw(None, 1.0);
        // The pixels the particles brighten.
        let lit = |img: &RgbaImage| {
            img.pixels
                .chunks(4)
                .zip(empty.pixels.chunks(4))
                .filter(|(p, e)| p[0] as i32 > e[0] as i32 + 20)
                .count()
        };
        let (full, half) = (lit(&draw(Some(true), 1.0)), lit(&draw(Some(true), 0.5)));
        let share = half as f32 / full.max(1) as f32;
        assert!(
            full > 200 && (0.3..0.75).contains(&share),
            "{path:?}: the fountain lit {half} pixels at 0.5 against {full} at 1"
        );
        let (whole, kept) = (draw(Some(false), 1.0), draw(Some(false), 0.5));
        assert!(lit(&whole) > 50, "{path:?}: the burst drew nothing");
        assert!(
            whole.pixels == kept.pixels,
            "{path:?}: the burst of six lost particles at 0.5"
        );
    }
}

/// One still particle 0.4 m across at (0, 1, 0), three times as tall
/// (`height_over_life`), turned by `degrees`, facing as `align` says,
/// before a camera at `eye`.
fn tall_scene(height: bool, degrees: f32, align: ParticleAlign, eye: Vec3) -> RenderScene {
    let mut scene = RenderScene::new();
    let e = scene.add_emitter(ParticleEmitter {
        max_particles: 1,
        rate: 0.0,
        bursts: vec![og_render::Burst::once(0.0, 1)],
        looping: false,
        duration: 10.0,
        lifetime: [10.0, 10.0],
        speed: [0.0, 0.0],
        size: [0.4, 0.4],
        height_over_life: height.then(|| Curve::constant(3.0)),
        color_over_life: og_render::ColorCurve::constant([1.0; 4]),
        color: [4.0, 4.0, 4.0, 1.0],
        rotation_degrees: [degrees, degrees],
        gravity: Vec3::ZERO,
        blend: og_render::ParticleBlend::Additive,
        align,
        ..Default::default()
    });
    scene.effects.particles.push(ParticleSystem::new(
        1,
        e,
        Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0)),
        3,
        0,
    ));
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.frame.camera = Camera {
        position: eye,
        target: Vec3::new(0.0, 1.0, 0.0),
        ..Default::default()
    };
    scene
}

/// A particle with a height curve draws as tall as it says on both paths,
/// and its rotation turns the tall shape (the height is along the
/// particle's own up, not the screen's): upright it is about three times
/// taller than wide, turned 90 degrees three times wider than tall, and
/// without the curve square. A height lost on a path draws a square; a
/// height applied after the rotation draws tall when turned.
#[test]
#[ignore = "requires GPU"]
fn a_height_curve_draws_a_tall_particle_along_its_own_up() {
    for path in [ParticleSimulation::Auto, ParticleSimulation::Cpu] {
        let desc = RenderDesc {
            particles: path,
            ..small()
        };
        let aspect = |height: bool, degrees: f32| {
            let scene = tall_scene(height, degrees, ParticleAlign::Camera, SIDE);
            lit_box(&desc, scene)
                .map(|(w, h)| h as f32 / w as f32)
                .unwrap_or_else(|| panic!("{path:?}: the particle drew nothing"))
        };
        let (square, tall, turned) = (aspect(false, 0.0), aspect(true, 0.0), aspect(true, 90.0));
        println!("{path:?}: square {square:.2}, tall {tall:.2}, turned {turned:.2}");
        assert!((0.8..1.25).contains(&square), "{path:?}: square {square}");
        assert!((2.4..3.6).contains(&tall), "{path:?}: tall {tall}");
        assert!((0.28..0.42).contains(&turned), "{path:?}: turned {turned}");
    }
}

/// The eye level with the particle, 4 m in front, and one 5 m away
/// looking down at it from 53 degrees above the horizon.
const SIDE: Vec3 = Vec3::new(0.0, 1.0, 4.0);
const ABOVE: Vec3 = Vec3::new(0.0, 5.0, 3.0);

/// The width and height in pixels of what a scene's particles light, run
/// ten frames; `None` when they light nothing.
fn lit_box(desc: &RenderDesc, mut scene: RenderScene) -> Option<(u32, u32)> {
    let (img, _, _) = run(desc, &mut scene, 10, |s, f| fx::tick_all(s, f as u64));
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
    for y in 0..img.height {
        for x in 0..img.width {
            if luma(px(&img, x, y)) > 60.0 {
                (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
            }
        }
    }
    (x1 >= x0).then(|| (x1 - x0 + 1, y1 - y0 + 1))
}

/// An upright particle stands on its axis whatever the camera's pitch, and
/// a facing one lies across its normal, on both paths. Seen from above, a
/// tall camera-facing particle is three times taller than wide but an
/// upright one is shortened as its axis tips away from the eye (about
/// 1.8); a tall particle lying flat on the ground (`facing 0 1 0`) is
/// edge-on from the side and draws at most a line, and from above lies
/// along the emitter's x (its up when its normal is +y), about a quarter
/// as deep on the screen as it is wide. A mode drawn as camera-facing
/// fails each.
#[test]
#[ignore = "requires GPU"]
fn upright_and_facing_particles_keep_their_axis_whatever_the_camera() {
    let upright = ParticleAlign::Upright { axis: Vec3::Y };
    let flat = ParticleAlign::Facing { normal: Vec3::Y };
    for path in [ParticleSimulation::Auto, ParticleSimulation::Cpu] {
        let desc = RenderDesc {
            particles: path,
            ..small()
        };
        let aspect = |align: ParticleAlign, height: bool, eye: Vec3| {
            let (w, h) = lit_box(&desc, tall_scene(height, 0.0, align, eye))
                .unwrap_or_else(|| panic!("{path:?}: {align:?} from {eye} drew nothing"));
            h as f32 / w as f32
        };
        let facing = aspect(ParticleAlign::Camera, true, ABOVE);
        let standing = aspect(upright, true, ABOVE);
        let level = aspect(upright, true, SIDE);
        let lying = aspect(flat, true, ABOVE);
        let edge = lit_box(&desc, tall_scene(true, 0.0, flat, SIDE));
        println!(
            "{path:?}: camera {facing:.2}, upright {standing:.2} (level {level:.2}), flat {lying:.2}, edge-on {edge:?}"
        );
        assert!((2.4..3.6).contains(&facing), "{path:?}: camera {facing}");
        assert!(
            (1.5..2.3).contains(&standing),
            "{path:?}: upright {standing}"
        );
        assert!(
            (2.4..3.6).contains(&level),
            "{path:?}: upright level {level}"
        );
        assert!((0.18..0.4).contains(&lying), "{path:?}: flat {lying}");
        assert!(
            edge.is_none_or(|(w, h)| h <= 3 && w > 10 * h),
            "{path:?}: edge-on {edge:?}"
        );
    }
}

/// A camera-facing effect mesh (`EffectMesh::billboard`) turns to the eye
/// with its scale kept in the camera's frame, on both paths: a quad 2 m
/// wide and 1 m tall in its xy plane, seen along its edge from +x, draws
/// nothing as placed but a shape twice as wide as tall when facing; seen
/// face on it is the same shape either way. A flag dropped draws nothing
/// edge-on; its scale taken in the mesh's own frame draws it square or
/// tall.
#[test]
#[ignore = "requires GPU"]
fn a_camera_facing_effect_mesh_turns_to_the_eye_with_its_scale() {
    let scene = |billboard: bool, eye: Vec3| {
        let mut scene = RenderScene::new();
        let mesh = scene.add_mesh(og_render::shapes::plane_xy(1.0, 1.0));
        let material = scene.add_effect_material(og_render::EffectMaterial::default());
        scene.effects.effect_meshes.push(og_render::EffectMesh {
            id: 1,
            mesh,
            material,
            transform: Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0))
                * Mat4::from_scale(Vec3::new(2.0, 1.0, 1.0)),
            life: 0.5,
            seed: 1,
            color: [4.0, 4.0, 4.0, 1.0],
            blend: og_render::ParticleBlend::Additive,
            layers: 1,
            hue: 0.0,
            billboard,
        });
        scene.lighting.environment.background = Background::Color(Vec3::ZERO);
        scene.frame.camera = Camera {
            position: eye,
            target: Vec3::new(0.0, 1.0, 0.0),
            ..Default::default()
        };
        scene
    };
    let edge_eye = Vec3::new(5.0, 1.0, 0.0);
    let face_eye = Vec3::new(0.0, 1.0, 5.0);
    for path in [ParticleSimulation::Auto, ParticleSimulation::Cpu] {
        let desc = RenderDesc {
            particles: path,
            ..small()
        };
        let aspect = |b: Option<(u32, u32)>| b.map(|(w, h)| h as f32 / w as f32);
        let placed_edge = lit_box(&desc, scene(false, edge_eye));
        let facing_edge = aspect(lit_box(&desc, scene(true, edge_eye)));
        let placed_face = aspect(lit_box(&desc, scene(false, face_eye)));
        let facing_face = aspect(lit_box(&desc, scene(true, face_eye)));
        println!(
            "{path:?}: edge-on placed {placed_edge:?} facing {facing_edge:?}; face on placed {placed_face:?} facing {facing_face:?}"
        );
        assert!(
            placed_edge.is_none_or(|(w, h)| w <= 3 && h > 10 * w),
            "{path:?}: placed edge-on {placed_edge:?}"
        );
        for (what, a) in [
            ("facing edge-on", facing_edge),
            ("placed face on", placed_face),
            ("facing face on", facing_face),
        ] {
            assert!(
                a.is_some_and(|a| (0.4..0.62).contains(&a)),
                "{path:?}: {what} {a:?}"
            );
        }
    }
}

/// Each particle of an emitter with a second colour curve takes its own
/// share between the two, on both paths: 400 still particles red by the
/// first curve and blue by the second draw some nearly red, some nearly
/// blue and some between. One curve dropped draws them all red; one
/// share for every particle (the mean) draws them all purple.
#[test]
#[ignore = "requires GPU"]
fn a_second_colour_curve_gives_each_particle_its_own_colour_between() {
    let scene = || {
        let mut scene = RenderScene::new();
        let e = scene.add_emitter(ParticleEmitter {
            max_particles: 400,
            rate: 0.0,
            bursts: vec![og_render::Burst::once(0.0, 400)],
            looping: false,
            duration: 10.0,
            lifetime: [10.0, 10.0],
            speed: [0.0, 0.0],
            size: [0.07, 0.07],
            shape: og_render::EmitterShape::Box {
                half_extents: Vec3::new(1.6, 0.9, 0.0),
            },
            color_over_life: og_render::ColorCurve::constant([1.0, 0.0, 0.0, 1.0]),
            random_color_over_life: Some(og_render::ColorCurve::constant([0.0, 0.0, 1.0, 1.0])),
            gravity: Vec3::ZERO,
            blend: og_render::ParticleBlend::Alpha,
            ..Default::default()
        });
        scene.effects.particles.push(ParticleSystem::new(
            1,
            e,
            Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            5,
            0,
        ));
        scene.lighting.environment.background = Background::Color(Vec3::ZERO);
        scene.frame.camera = Camera {
            position: Vec3::new(0.0, 1.0, 4.0),
            target: Vec3::new(0.0, 1.0, 0.0),
            ..Default::default()
        };
        scene
    };
    for path in [ParticleSimulation::Auto, ParticleSimulation::Cpu] {
        let desc = RenderDesc {
            particles: path,
            ..small()
        };
        let (img, _, _) = run(&desc, &mut scene(), 10, |s, f| fx::tick_all(s, f as u64));
        let (mut red, mut blue, mut between) = (0, 0, 0);
        for p in img.pixels.chunks(4) {
            let (r, b) = (p[0] as f32, p[2] as f32);
            if r.max(b) < 80.0 {
                continue;
            }
            if r > 1.6 * b {
                red += 1;
            } else if b > 1.6 * r {
                blue += 1;
            } else {
                between += 1;
            }
        }
        let lit = (red + blue + between) as f32;
        println!("{path:?}: {red} red, {blue} blue, {between} between of {lit}");
        assert!(lit > 500.0, "{path:?}: {lit} lit");
        for (what, n) in [("red", red), ("blue", blue), ("between", between)] {
            assert!(n as f32 > 0.15 * lit, "{path:?}: {n} {what} of {lit}");
        }
    }
}

/// One particle flying +x at 2 m/s from x = -1 for half a second, 0.2 m
/// across, with a trail of 0.5 s (`trail`), before a camera 4 m away.
fn trailing_scene(trail: Option<og_render::ParticleTrail>) -> RenderScene {
    let mut scene = RenderScene::new();
    let e = scene.add_emitter(ParticleEmitter {
        max_particles: 4,
        rate: 0.0,
        bursts: vec![og_render::Burst::once(0.0, 1)],
        looping: false,
        duration: 10.0,
        lifetime: [10.0, 10.0],
        speed: [2.0, 2.0],
        shape: og_render::EmitterShape::Cone {
            angle_degrees: 0.0,
            radius: 0.0,
        },
        size: [0.2, 0.2],
        color_over_life: og_render::ColorCurve::constant([1.0; 4]),
        color: [4.0, 4.0, 4.0, 1.0],
        gravity: Vec3::ZERO,
        blend: og_render::ParticleBlend::Additive,
        trail,
        ..Default::default()
    });
    // The cone's +y turned onto +x.
    scene.effects.particles.push(ParticleSystem::new(
        1,
        e,
        Mat4::from_translation(Vec3::new(-1.0, 1.0, 0.0))
            * Mat4::from_rotation_z(-std::f32::consts::FRAC_PI_2),
        3,
        0,
    ));
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 1.0, 4.0),
        target: Vec3::new(0.0, 1.0, 0.0),
        ..Default::default()
    };
    scene
}

/// A particle's trail follows where it went, ends at it, narrows to its
/// tail and fades, on both paths: after half a second the particle alone
/// is a dot at x = 0; its trail alone (`head` off) reaches back towards
/// its start without passing the dot's right edge, is thicker at the
/// particle than halfway back, and draws nothing of the dot's own round
/// shape ahead of it; with the head both show. The two paths draw it
/// within a few percent of each other. A trail drawn from the wrong end
/// of its ring starts at the tail; one with no cut or no taper is as thick
/// at its tail as at its head.
#[test]
#[ignore = "requires GPU"]
fn a_particles_trail_follows_its_path_and_narrows_to_its_tail() {
    let only = og_render::ParticleTrail {
        lifetime: 0.5,
        width: 1.0,
        head: false,
    };
    let mut lit_counts = Vec::new();
    for path in [ParticleSimulation::Auto, ParticleSimulation::Cpu] {
        let desc = RenderDesc {
            particles: path,
            ..small()
        };
        let draw = |trail| {
            run(&desc, &mut trailing_scene(trail), 31, |s, f| {
                fx::tick_all(s, f as u64)
            })
            .0
        };
        let (dot, streak, both) = (
            draw(None),
            draw(Some(only)),
            draw(Some(og_render::ParticleTrail { head: true, ..only })),
        );
        save(&format!("trail-{path:?}"), &streak);
        let lit = |img: &RgbaImage| {
            let mut cols = vec![0u32; img.width as usize];
            for y in 0..img.height {
                for x in 0..img.width {
                    if luma(px(img, x, y)) > 40.0 {
                        cols[x as usize] += 1;
                    }
                }
            }
            cols
        };
        let span = |c: &[u32]| {
            let x0 = c.iter().position(|&n| n > 0);
            let x1 = c.iter().rposition(|&n| n > 0);
            x0.zip(x1)
                .unwrap_or_else(|| panic!("{path:?}: nothing drawn"))
        };
        let (d, s, b) = (lit(&dot), lit(&streak), lit(&both));
        let ((d0, d1), (s0, s1), (b0, b1)) = (span(&d), span(&s), span(&b));
        let per_metre = (d1 - d0) as f32 / 0.2;
        let at = |metres_back: f32| s[(s1 as f32 - metres_back * per_metre).max(0.0) as usize];
        println!(
            "{path:?}: dot {d0}..{d1}, trail {s0}..{s1} ({:.2} m), both {b0}..{b1}; thickness 0.1 m back {}, 0.4 m back {}",
            (s1 - s0) as f32 / per_metre,
            at(0.1),
            at(0.4)
        );
        assert!(s1 <= d1 + 2, "{path:?}: the trail passes the particle");
        assert!(
            (s1 - s0) as f32 > 0.6 * per_metre,
            "{path:?}: the trail is {} px long",
            s1 - s0
        );
        assert!(
            at(0.1) > at(0.4) + 1,
            "{path:?}: no narrower towards its tail"
        );
        assert!(
            b0 <= s0 + 1 && b1 >= d1 - 1,
            "{path:?}: with its head {b0}..{b1}"
        );
        lit_counts.push(s.iter().sum::<u32>() as f32);
    }
    let (g, c) = (lit_counts[0], lit_counts[1]);
    assert!(
        (g - c).abs() < 0.05 * g.max(c),
        "compute {g} against CPU {c} pixels"
    );
}

/// A subtractive particle takes its colour away from what is behind it
/// and a multiply one tints what is behind by its colour, on both paths:
/// one large still particle, its colour (0.5, 0.1, 0.1), over a grey
/// background. Subtracted, the grey loses more red than green; multiplied,
/// it keeps more red than green and every channel falls; added, every
/// channel rises. Drawn alpha blended instead, it would show its own dull
/// red; a subtraction drawn the wrong way round, red.
#[test]
#[ignore = "requires GPU"]
fn subtractive_and_multiply_particles_change_what_is_behind() {
    let scene = |blend: og_render::ParticleBlend| {
        let mut scene = RenderScene::new();
        let e = scene.add_emitter(ParticleEmitter {
            max_particles: 1,
            rate: 0.0,
            bursts: vec![og_render::Burst::once(0.0, 1)],
            looping: false,
            duration: 10.0,
            lifetime: [10.0, 10.0],
            speed: [0.0, 0.0],
            size: [1.5, 1.5],
            color_over_life: og_render::ColorCurve::constant([1.0; 4]),
            color: [0.5, 0.1, 0.1, 1.0],
            gravity: Vec3::ZERO,
            texture: None,
            blend,
            ..Default::default()
        });
        scene.effects.particles.push(ParticleSystem::new(
            1,
            e,
            Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            3,
            0,
        ));
        scene.lighting.environment.background = Background::Color(Vec3::splat(0.6));
        scene.frame.camera = Camera {
            position: Vec3::new(0.0, 1.0, 4.0),
            target: Vec3::new(0.0, 1.0, 0.0),
            ..Default::default()
        };
        scene
    };
    for path in [ParticleSimulation::Auto, ParticleSimulation::Cpu] {
        let desc = RenderDesc {
            particles: path,
            ..small()
        };
        let centre = |blend| {
            let (img, _, _) = run(&desc, &mut scene(blend), 10, |s, f| {
                fx::tick_all(s, f as u64)
            });
            let (w, h) = (img.width, img.height);
            (
                px(&img, 2, 2).map(f32::from),
                px(&img, w / 2, h / 2).map(f32::from),
            )
        };
        let (bg, sub) = centre(og_render::ParticleBlend::Subtractive);
        let (_, mul) = centre(og_render::ParticleBlend::Multiply);
        let (_, add) = centre(og_render::ParticleBlend::Additive);
        println!(
            "{path:?}: behind {bg:?}, subtractive {sub:?}, multiply {mul:?}, additive {add:?}"
        );
        let drop = |c: [f32; 3], i: usize| bg[i] - c[i];
        assert!(
            drop(sub, 0) > drop(sub, 1) + 20.0 && drop(sub, 1) > 5.0,
            "{path:?}: subtractive {sub:?} over {bg:?}"
        );
        assert!(
            drop(mul, 0) > 10.0 && drop(mul, 1) > drop(mul, 0) + 20.0,
            "{path:?}: multiply {mul:?} over {bg:?}"
        );
        assert!(
            (0..3).all(|i| add[i] > bg[i] + 5.0),
            "{path:?}: additive {add:?}"
        );
    }
}

/// A particle whose collision says `die` dies where it first hits, on
/// both paths: the fountain over a collider plane keeps dozens of
/// particles bouncing back up off it without, and with `die` none rise
/// off the ground and far fewer are alive; none sinks through it either
/// way. A hit that only bounces keeps them; a death that waits for the
/// next tick lets them through the plane.
#[test]
#[ignore = "requires GPU"]
fn a_particle_that_dies_on_collision_is_gone_where_it_hits() {
    let run_with = |path: ParticleSimulation, die: bool| {
        let mut scene = collision_scene(true);
        scene.objects.instances.clear();
        let e = scene.effects.particles[0].emitter;
        let mut emitter = scene.emitter(e).expect("emitter").clone();
        emitter.collision = Some(ParticleCollision {
            against: og_render::CollisionTarget::Colliders,
            die,
            ..Default::default()
        });
        scene.set_emitter(e, emitter);
        scene.world.colliders = vec![og_render::SceneCollider::Plane {
            normal: Vec3::Y,
            offset: 0.0,
        }];
        let desc = RenderDesc {
            particles: path,
            ..small()
        };
        let (_, _, r) = run(&desc, &mut scene, 90, |s, f| fx::tick_all(s, f as u64));
        let live: Vec<_> = r
            .read_particles(1)
            .expect("system")
            .into_iter()
            .filter(|p| p.alive())
            .collect();
        let lowest = live.iter().map(|p| p.pos_age[1]).fold(f32::MAX, f32::min);
        let bounced = live
            .iter()
            .filter(|p| p.vel_life[1] > 0.0 && p.pos_age[1] < 1.0)
            .count();
        (live.len(), bounced, lowest)
    };
    for path in [ParticleSimulation::Auto, ParticleSimulation::Cpu] {
        let (kept, kept_bounced, kept_low) = run_with(path, false);
        let (left, bounced, low) = run_with(path, true);
        println!(
            "{path:?}: bouncing {kept} alive, {kept_bounced} rising, lowest {kept_low}; dying {left} alive, {bounced} rising, lowest {low}"
        );
        assert!(kept_bounced > 5, "{path:?}: {kept_bounced} bounced");
        assert_eq!(bounced, 0, "{path:?}: {bounced} rose off the ground");
        assert!(left * 3 < kept * 2, "{path:?}: {left} alive against {kept}");
        assert!(
            kept_low > -0.1 && low > -0.1,
            "{path:?}: sank to {kept_low} or {low}"
        );
    }
}

/// Particles take their `forceFields` share of the og.ForceField
/// acceleration where they are, on both paths and alike: a sideways field
/// over the whole world carries the fountain along +x by twice as much at
/// share 1 as at 0.5, not at all at 0, and a field whose region is far
/// away pushes nothing. A path that drops the fields, ignores the share or
/// ignores the region fails one of these.
#[test]
#[ignore = "requires GPU"]
fn particles_take_their_share_of_the_force_fields_where_they_are() {
    use og_env::force::{Field, FieldKind, Forces};
    use og_env::volume::{Placement, Volume};
    let field = |position: Vec3, volume: Option<Volume>| Field {
        at: Placement {
            position,
            quaternion: glam::Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        volume,
        kind: FieldKind::Directional,
        direction: Vec3::X,
        strength: 8.0,
    };
    let run_with = |path: ParticleSimulation, share: f32, far: bool| {
        let mut scene = collision_scene(false);
        let e = scene.effects.particles[0].emitter;
        let mut emitter = scene.emitter(e).expect("emitter").clone();
        emitter.force_fields = share;
        scene.set_emitter(e, emitter);
        scene.world.force_fields = Forces {
            fields: vec![if far {
                field(
                    Vec3::new(60.0, 0.0, 0.0),
                    Some(Volume {
                        size: Vec3::splat(4.0),
                        radius: 0.0,
                        fade: 0.0,
                    }),
                )
            } else {
                field(Vec3::ZERO, None)
            }],
        };
        let desc = RenderDesc {
            particles: path,
            ..small()
        };
        let (_, _, r) = run(&desc, &mut scene, 90, |s, f| fx::tick_all(s, f as u64));
        let live: Vec<_> = r
            .read_particles(1)
            .expect("system")
            .into_iter()
            .filter(|p| p.alive())
            .collect();
        assert!(live.len() > 50, "{path:?}: {} alive", live.len());
        live.iter().map(|p| p.vel_life[0]).sum::<f32>() / live.len() as f32
    };
    let mut full = Vec::new();
    for path in [ParticleSimulation::Auto, ParticleSimulation::Cpu] {
        let none = run_with(path, 0.0, false);
        let half = run_with(path, 0.5, false);
        let one = run_with(path, 1.0, false);
        let far = run_with(path, 1.0, true);
        println!("{path:?}: mean vx at share 0 {none}, 0.5 {half}, 1 {one}; far field {far}");
        assert!(
            none.abs() < 0.3 && far.abs() < 0.3,
            "{path:?}: {none} {far}"
        );
        assert!(one > 3.0, "{path:?}: share 1 gives {one}");
        assert!(
            (one / half - 2.0).abs() < 0.3,
            "{path:?}: share 1 {one} against 0.5 {half}"
        );
        full.push(one);
    }
    assert!(
        (full[0] - full[1]).abs() < 0.1 * full[1],
        "the paths disagree: {full:?}"
    );
}

/// A blended sheet cross-fades each frame into the next, on both paths:
/// one particle on a two-frame sheet (red, then blue) played once over its
/// one-second life. A quarter through, halfway between the frames, it
/// draws red and blue alike, where an unblended sheet draws red; three
/// quarters through, in its last frame, it holds blue instead of fading
/// back into the first frame.
#[test]
#[ignore = "requires GPU"]
fn a_blended_sheet_fades_into_the_next_frame_and_holds_the_last() {
    let draw = |path: ParticleSimulation, blend: bool, ticks: u32| {
        let mut scene = RenderScene::new();
        let mut rgba = Vec::new();
        for _ in 0..4 {
            for x in 0..8 {
                rgba.extend(if x < 4 {
                    [255, 0, 0, 255]
                } else {
                    [0, 0, 255, 255]
                });
            }
        }
        let sheet = scene.add_texture(og_render::TextureData {
            width: 8,
            height: 4,
            rgba,
            ..Default::default()
        });
        let e = scene.add_emitter(ParticleEmitter {
            max_particles: 1,
            rate: 0.0,
            bursts: vec![og_render::Burst::once(0.0, 1)],
            looping: false,
            duration: 10.0,
            lifetime: [1.0, 1.0],
            speed: [0.0, 0.0],
            size: [1.5, 1.5],
            color_over_life: og_render::ColorCurve::constant([1.0; 4]),
            gravity: Vec3::ZERO,
            texture: Some(sheet),
            sheet: SpriteSheet {
                columns: 2,
                rows: 1,
                frames: 2,
                mode: SheetMode::OverLife { cycles: 1.0 },
                blend,
            },
            blend: og_render::ParticleBlend::Alpha,
            ..Default::default()
        });
        scene.effects.particles.push(ParticleSystem::new(
            1,
            e,
            Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            3,
            0,
        ));
        scene.frame.camera = Camera {
            position: Vec3::new(0.0, 1.0, 4.0),
            target: Vec3::new(0.0, 1.0, 0.0),
            ..Default::default()
        };
        let desc = RenderDesc {
            particles: path,
            ..small()
        };
        let (img, _, _) = run(&desc, &mut scene, ticks + 1, |s, f| {
            fx::tick_all(s, f as u64)
        });
        let [r, _, b] = px(&img, img.width / 2, img.height / 2).map(f32::from);
        (r, b)
    };
    for path in [ParticleSimulation::Auto, ParticleSimulation::Cpu] {
        let mid = draw(path, true, 15);
        let stepped = draw(path, false, 15);
        let last = draw(path, true, 45);
        println!("{path:?}: halfway {mid:?}, unblended {stepped:?}, last frame {last:?}");
        assert!(
            mid.0 > 60.0 && mid.1 > 60.0 && (mid.0 - mid.1).abs() < 0.35 * mid.0.max(mid.1),
            "{path:?}: halfway between the frames {mid:?}"
        );
        assert!(
            stepped.0 > 150.0 && stepped.1 < 15.0,
            "{path:?}: unblended {stepped:?}"
        );
        assert!(
            last.1 > 150.0 && last.0 < 15.0,
            "{path:?}: the last frame {last:?}"
        );
    }
}
