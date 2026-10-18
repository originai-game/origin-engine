//! The E1 effects, asserted on frames and counts: particles (determinism of
//! a 600-tick run on both paths, the CPU and GPU paths agreeing, spawn
//! counts, soft particles, depth collision, 2D emitters), trails and beams,
//! 2D lights and soft shadows, glow, sprite flash, decals, the grading stack
//! (LUT, vignette, chromatic aberration, grain, flash), camera shake,
//! ambient occlusion, depth of field, motion blur, god rays, outlines,
//! toon, dissolve, rim, and GPU skinning with blendshapes.

use glam::{Mat4, Vec2, Vec3};
use og_render::{
    shapes, AmbientOcclusion, AoMethod, Background, Beam, Burst, Camera, CameraShake,
    ChromaticAberration, ColorCurve, ColorGrading, ColorLut, Curve, Decal, DepthOfField,
    DofQuality, EmitterShape, Exposure, FilmGrain, FrameStats, Glow, GodRays, JointPalette,
    Light2d, Lighting2d, Material, MorphTargets, MotionBlur, Occluder2d, Outline,
    ParticleCollision, ParticleEmitter, ParticleRender, ParticleSimulation, ParticleSystem,
    Projection, RenderDesc, RenderScene, RenderTarget, Renderer, RgbaImage, ScreenFlash,
    SkinnedInstance, Sprite, SpriteFx, StandardMaterial, Tier, Tonemap, Trail, TrailStyle,
    Vignette, MAX_DECALS,
};
use og_tests::fx_scenes as fx;
use og_tests::{gfx, scenes};
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-effects")
}

fn small() -> RenderDesc {
    RenderDesc {
        width: 320,
        height: 180,
        ..scenes::base_desc()
    }
}

/// Renders `frames` frames (calling `update(scene, frame)` before each) and
/// returns the last one.
fn run(
    desc: &RenderDesc,
    scene: &mut RenderScene,
    frames: u32,
    update: impl Fn(&mut RenderScene, u32),
) -> (RgbaImage, FrameStats, Renderer) {
    let mut renderer = Renderer::new_blocking(desc).expect("renderer");
    let mut stats = FrameStats::default();
    for f in 0..frames.max(1) {
        update(scene, f);
        stats = renderer
            .render(scene, &RenderTarget::Offscreen)
            .expect("render");
    }
    (renderer.read_offscreen().expect("capture"), stats, renderer)
}

fn frame(desc: &RenderDesc, scene: &RenderScene) -> (RgbaImage, FrameStats) {
    let mut renderer = Renderer::new_blocking(desc).expect("renderer");
    let stats = renderer
        .render(scene, &RenderTarget::Offscreen)
        .expect("render");
    (renderer.read_offscreen().expect("capture"), stats)
}

fn px(img: &RgbaImage, x: u32, y: u32) -> [u8; 3] {
    let i = ((y.min(img.height - 1) * img.width + x.min(img.width - 1)) * 4) as usize;
    [img.pixels[i], img.pixels[i + 1], img.pixels[i + 2]]
}

fn luma(p: [u8; 3]) -> f32 {
    0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32
}

/// Mean luma over a rectangle (x0, y0, x1, y1), exclusive ends.
fn region(img: &RgbaImage, r: (u32, u32, u32, u32)) -> f32 {
    let mut sum = 0.0;
    let mut n = 0.0;
    for y in r.1..r.3 {
        for x in r.0..r.2 {
            sum += luma(px(img, x, y));
            n += 1.0;
        }
    }
    sum / n
}

/// Pixels whose luma differs by more than `by`.
fn differing(a: &RgbaImage, b: &RgbaImage, by: f32) -> usize {
    a.pixels
        .chunks_exact(4)
        .zip(b.pixels.chunks_exact(4))
        .filter(|(p, q)| (luma([p[0], p[1], p[2]]) - luma([q[0], q[1], q[2]])).abs() > by)
        .count()
}

/// The pixel a world point projects to.
fn project(cam: &Camera, w: u32, h: u32, p: Vec3) -> (u32, u32) {
    let view = glam::camera::rh::view::look_at_mat4(cam.position, cam.target, cam.up);
    let proj = match cam.projection {
        Projection::Perspective { fov_y_degrees } => glam::camera::rh::proj::directx::perspective(
            fov_y_degrees.to_radians(),
            w as f32 / h as f32,
            cam.near,
            cam.far,
        ),
        Projection::Orthographic { height } => {
            let hh = height * 0.5;
            let hw = hh * w as f32 / h as f32;
            glam::camera::rh::proj::directx::orthographic(-hw, hw, -hh, hh, cam.near, cam.far)
        }
    };
    let c = proj * view * p.extend(1.0);
    let ndc = c.truncate() / c.w;
    (
        ((ndc.x * 0.5 + 0.5) * w as f32) as u32,
        ((0.5 - ndc.y * 0.5) * h as f32) as u32,
    )
}

fn save(name: &str, img: &RgbaImage) {
    let _ = gfx::save_png(&out_dir().join(format!("{name}.png")), img);
}

fn adapter_tier() -> Tier {
    Renderer::new_blocking(&small())
        .expect("renderer")
        .info()
        .tier
}

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
    let (a, sa, _) = run(&desc, &mut fx::particles_scene(), 601, stepped);
    let (b, _, _) = run(&desc, &mut fx::particles_scene(), 601, stepped);
    let (c, _, _) = run(&desc, &mut fx::particles_scene(), 1, |s, _| {
        fx::tick_all(s, 600)
    });
    save(&format!("particles-600-{path:?}"), &a);
    assert!(sa.particle_systems >= 6, "{} systems", sa.particle_systems);
    let lit = a.pixels.chunks_exact(4).filter(|p| p[0] > 200).count();
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
    if adapter_tier() != Tier::Full {
        return assert_cpu_only();
    }
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

fn assert_cpu_only() {
    // The no-compute tier has only the CPU path: Auto must use it.
    let mut scene = RenderScene::new();
    let e = plain_fountain(&mut scene);
    scene
        .particles
        .push(ParticleSystem::new(7, e, Mat4::IDENTITY, 42, 0));
    let (_, stats, _) = run(&small(), &mut scene, 10, |s, f| fx::tick_all(s, f as u64));
    assert!(stats.particles_cpu > 0);
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

/// Particles with collision stay above the ground (full tier); without it
/// they fall through.
#[test]
fn particles_bounce_off_the_depth_buffer() {
    if adapter_tier() != Tier::Full {
        let mut scene = collision_scene(true);
        let (_, stats, _) = run(&small(), &mut scene, 2, |s, f| fx::tick_all(s, f as u64));
        assert_eq!(stats.particle_collision_skipped, 1);
        return;
    }
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

/// Sprite-mode emitters draw through the sprite batch on their layer.
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
    assert_eq!(stats.sprites, stats.particles_cpu);
    assert!(stats.sprites >= 29 && stats.sprite_draws >= 1);
    assert!(img.pixels.chunks_exact(4).any(|p| p[0] > 100));
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
        ..Default::default()
    });
    scene.trails.push(Trail {
        id: 1,
        style,
        position: Vec3::ZERO,
        tick: 0,
    });
    scene.camera = Camera::orthographic_2d(Vec2::ZERO, 6.0);
    let mut r = Renderer::new_blocking(&small()).expect("renderer");
    let mut render =
        |scene: &RenderScene| r.render(scene, &RenderTarget::Offscreen).expect("render");
    let mut last = FrameStats::default();
    for t in 0..10u64 {
        scene.trails[0].tick = t;
        scene.trails[0].position = Vec3::new(t as f32 * 0.2 - 1.0, 0.0, 0.0);
        last = render(&scene);
    }
    assert_eq!(last.strips, 1);
    assert!(
        last.strip_segments >= 8,
        "{} segments while moving",
        last.strip_segments
    );
    // Standing still for longer than the lifetime: the trail collapses.
    scene.trails[0].tick = 40;
    let still = render(&scene);
    assert_eq!(still.strips, 0, "a resting trail draws nothing");
    // Backwards in time: restarts with a single point.
    scene.trails[0].tick = 5;
    scene.trails[0].position = Vec3::new(2.0, 0.0, 0.0);
    assert_eq!(render(&scene).strips, 0);
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

// ---------------------------------------------------------------------------
// 2D lights, glow, flash.
// ---------------------------------------------------------------------------

fn lit_room(shadows: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    scene.add_sprite(Sprite {
        size: Vec2::new(20.0, 12.0),
        color: [0.8, 0.8, 0.8, 1.0],
        ..Default::default()
    });
    let wall = Sprite {
        position: Vec3::new(0.0, 0.0, 0.0),
        size: Vec2::new(0.6, 2.0),
        color: [0.3, 0.2, 0.1, 1.0],
        layer: 1,
        ..Default::default()
    };
    scene.occluders_2d.push(Occluder2d::from_sprite(&wall, 0.0));
    scene.add_sprite(wall);
    scene.lighting_2d = Some(Lighting2d {
        ambient: Vec3::splat(0.05),
        ..Default::default()
    });
    scene.lights_2d.push(Light2d {
        position: Vec2::new(-3.0, 0.0),
        radius: 9.0,
        intensity: 1.0,
        source_radius: 0.4,
        cast_shadows: shadows,
        ..Default::default()
    });
    scene.camera = Camera::orthographic_2d(Vec2::ZERO, 8.0);
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.post.tonemap = Tonemap::None;
    scene
}

/// A wall between a 2D light and the floor shades the floor behind it with
/// a soft penumbra; the light side is unaffected; ambient fills the rest.
#[test]
fn two_d_lights_cast_soft_shadows_from_occluders() {
    let (lit, stats) = frame(&small(), &lit_room(true));
    let (open, _) = frame(&small(), &lit_room(false));
    save("light2d", &lit);
    assert_eq!((stats.lights_2d, stats.occluders_2d), (1, 1));
    let cam = Camera::orthographic_2d(Vec2::ZERO, 8.0);
    let at = |p: Vec2| project(&cam, 320, 180, p.extend(0.0));
    let behind = at(Vec2::new(1.5, 0.0));
    let front = at(Vec2::new(-1.5, 0.0));
    let (b, bo) = (
        luma(px(&lit, behind.0, behind.1)),
        luma(px(&open, behind.0, behind.1)),
    );
    let (f, fo) = (
        luma(px(&lit, front.0, front.1)),
        luma(px(&open, front.0, front.1)),
    );
    println!("behind the wall {b} (open {bo}); in front {f} (open {fo})");
    // The umbra receives only the ambient light (0.04 linear: 56 in sRGB).
    assert!(b < 60.0 && bo > 120.0, "the umbra is not dark");
    assert!((f - fo).abs() < 2.0, "the lit side changed");
    // Soft: along a vertical line behind the wall, intermediate values.
    let column: Vec<f32> = (0..90).map(|y| luma(px(&lit, behind.0 + 20, y))).collect();
    let lo = column.iter().cloned().fold(f32::MAX, f32::min);
    let hi = column.iter().cloned().fold(0.0, f32::max);
    let mid = column
        .iter()
        .filter(|&&v| v > lo + 0.2 * (hi - lo) && v < hi - 0.2 * (hi - lo))
        .count();
    println!("penumbra pixels in the column: {mid}");
    assert!(mid >= 3, "the shadow edge is hard");
}

/// Layers above the lit range are drawn as authored.
#[test]
fn two_d_lighting_leaves_ui_layers_alone() {
    let mut scene = lit_room(true);
    scene.lights_2d.clear();
    scene.add_sprite(Sprite {
        position: Vec3::new(2.0, 2.0, 0.0),
        color: [0.5, 0.5, 0.5, 1.0],
        layer: 200,
        ..Default::default()
    });
    let (img, _) = frame(&small(), &scene);
    let cam = Camera::orthographic_2d(Vec2::ZERO, 8.0);
    let (x, y) = project(&cam, 320, 180, Vec3::new(2.0, 2.0, 0.0));
    let ui = px(&img, x, y);
    let floor = px(&img, 20, 170);
    println!("ui {ui:?} floor {floor:?}");
    // 0.5 linear is 188 in sRGB: drawn as authored.
    assert!(
        (ui[0] as i32 - 188).abs() <= 2,
        "the UI sprite was lit: {ui:?}"
    );
    // The floor: 0.8 x ambient 0.05 = 0.04 linear, 56 in sRGB.
    assert!(
        (floor[0] as i32 - 56).abs() <= 3,
        "the floor should be ambient-dark: {floor:?}"
    );
}

/// A glowing sprite under an opaque one still glows around it; a flashed
/// sprite turns towards the flash colour.
#[test]
fn glow_is_order_independent_and_flash_tints() {
    let build = |glow: f32, flash: f32| {
        let mut scene = RenderScene::new();
        scene.add_sprite(Sprite {
            size: Vec2::splat(0.6),
            color: [1.0, 0.8, 0.2, 1.0],
            ..Default::default()
        });
        scene.add_sprite(Sprite {
            size: Vec2::splat(0.8),
            color: [0.2, 0.2, 0.25, 1.0],
            layer: 1,
            ..Default::default()
        });
        scene.add_sprite(Sprite {
            position: Vec3::new(2.0, 0.0, 0.0),
            color: [0.1, 0.2, 0.8, 1.0],
            ..Default::default()
        });
        scene.sprite_fx = vec![
            SpriteFx {
                sprite: 0,
                glow,
                ..Default::default()
            },
            SpriteFx {
                sprite: 2,
                flash: [1.0, 1.0, 1.0, flash],
                ..Default::default()
            },
        ];
        scene.post.glow = Some(Glow::default());
        scene.camera = Camera::orthographic_2d(Vec2::ZERO, 6.0);
        scene.environment.background = Background::Color(Vec3::ZERO);
        scene.post.tonemap = Tonemap::None;
        scene
    };
    let (glowing, stats) = frame(&small(), &build(3.0, 0.0));
    let (plain, plain_stats) = frame(&small(), &build(0.0, 0.0));
    let (flashed, _) = frame(&small(), &build(0.0, 1.0));
    assert!(stats.graph_passes > plain_stats.graph_passes, "glow passes");
    // Around the covering sprite (just outside it): glow.
    let around = region(&glowing, (160 - 28, 90 - 3, 160 - 22, 90 + 3));
    let around_plain = region(&plain, (160 - 28, 90 - 3, 160 - 22, 90 + 3));
    println!("around the cover: glow {around}, none {around_plain}");
    assert!(around > around_plain + 10.0);
    let cam = Camera::orthographic_2d(Vec2::ZERO, 6.0);
    let (x, y) = project(&cam, 320, 180, Vec3::new(2.0, 0.0, 0.0));
    assert_eq!(px(&flashed, x, y), [255, 255, 255]);
    assert!(px(&plain, x, y)[2] > px(&plain, x, y)[0] + 50);
}

// ---------------------------------------------------------------------------
// Decals.
// ---------------------------------------------------------------------------

#[test]
fn decals_project_onto_surfaces_and_are_capped() {
    let base = fx::decals_scene();
    let desc = scenes::base_desc();
    let (with, stats) = frame(&desc, &base);
    let mut none = fx::decals_scene();
    none.decals.clear();
    let (without, _) = frame(&desc, &none);
    save("decals", &with);
    assert_eq!(stats.decals, base.decals.len() as u32);
    // The scorch at (-1.6, 0, 0.6) darkens the ground; the rune glows.
    let scorch = project(&base.camera, 1280, 720, Vec3::new(-1.6, 0.0, 0.6));
    let rune = project(&base.camera, 1280, 720, Vec3::new(0.2 + 0.8, 0.0, 1.6));
    let (s1, s0) = (
        luma(px(&with, scorch.0, scorch.1)),
        luma(px(&without, scorch.0, scorch.1)),
    );
    println!("scorch {s1} vs {s0}");
    assert!(s1 < 0.5 * s0);
    let changed = differing(&with, &without, 8.0);
    assert!(changed > 20_000, "decals changed {changed} pixels");
    let r = px(&with, rune.0, rune.1);
    println!("rune ring pixel {r:?}");
    // Beyond MAX_DECALS they are dropped and counted.
    let mut many = fx::decals_scene();
    many.decals = vec![Decal::default(); MAX_DECALS + 5];
    let (_, s) = frame(&small(), &many);
    assert_eq!((s.decals, s.decals_dropped), (MAX_DECALS as u32, 5));
}

// ---------------------------------------------------------------------------
// The grading stack and screen effects.
// ---------------------------------------------------------------------------

fn colour_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let q = |c: [f32; 4], x: f32| Sprite {
        position: Vec3::new(x, 0.0, 0.0),
        size: Vec2::new(1.5, 4.0),
        color: c,
        ..Default::default()
    };
    scene.add_sprite(q([0.8, 0.1, 0.05, 1.0], -3.0));
    scene.add_sprite(q([0.1, 0.6, 0.1, 1.0], -1.0));
    scene.add_sprite(q([0.05, 0.1, 0.8, 1.0], 1.0));
    scene.add_sprite(q([0.5, 0.5, 0.5, 1.0], 3.0));
    scene.camera = Camera::orthographic_2d(Vec2::ZERO, 5.0);
    scene.environment.background = Background::Color(Vec3::splat(0.2));
    scene.post.tonemap = Tonemap::None;
    scene
}

/// The identity LUT changes nothing beyond 8-bit rounding; a red/blue swap
/// LUT turns the red bar blue (so the table's axes are read right).
#[test]
fn colour_grading_reads_the_lut() {
    let (plain, _) = frame(&small(), &colour_scene());
    let mut id = colour_scene();
    let lut = id.add_lut(ColorLut::identity());
    id.post.color_grading = Some(ColorGrading {
        lut,
        intensity: 1.0,
    });
    let (graded, _) = frame(&small(), &id);
    let worst = plain
        .pixels
        .iter()
        .zip(&graded.pixels)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap_or(0);
    println!("identity LUT: largest channel change {worst}");
    assert!(worst <= 3, "identity LUT changed a channel by {worst}");
    let mut swap = colour_scene();
    let lut = swap.add_lut(ColorLut::from_fn(|c| Vec3::new(c.z, c.y, c.x)));
    swap.post.color_grading = Some(ColorGrading {
        lut,
        intensity: 1.0,
    });
    let (swapped, _) = frame(&small(), &swap);
    let cam = colour_scene().camera;
    let (x, y) = project(&cam, 320, 180, Vec3::new(-3.0, 0.0, 0.0));
    let (before, after) = (px(&plain, x, y), px(&swapped, x, y));
    println!("red bar {before:?} -> {after:?}");
    assert!(before[0] > before[2] + 100 && after[2] > after[0] + 100);
    // The .cube parser: an identity cube resampled to 32³ is still identity.
    let mut cube = String::from("LUT_3D_SIZE 2\n");
    for b in 0..2 {
        for g in 0..2 {
            for r in 0..2 {
                cube.push_str(&format!("{r} {g} {b}\n"));
            }
        }
    }
    assert_eq!(
        ColorLut::from_cube(&cube).expect("cube"),
        ColorLut::identity()
    );
}

/// Vignette darkens the corners, not the centre; chromatic aberration
/// fringes edges near the corners more than at the centre; grain is the
/// same for a tick and changes with it; a full flash is the flash colour.
#[test]
fn vignette_aberration_grain_and_flash() {
    let mut base = colour_scene();
    base.environment.background = Background::Color(Vec3::splat(0.5));
    let (plain, _) = frame(&small(), &base);
    let mut v = colour_scene();
    v.environment.background = Background::Color(Vec3::splat(0.5));
    v.post.vignette = Some(Vignette {
        intensity: 0.8,
        ..Default::default()
    });
    let (vig, _) = frame(&small(), &v);
    assert!(luma(px(&vig, 2, 2)) < 0.6 * luma(px(&plain, 2, 2)));
    assert!((luma(px(&vig, 160, 90)) - luma(px(&plain, 160, 90))).abs() < 3.0);

    let mut ca = base_scene_with(|s| {
        s.post.chromatic_aberration = Some(ChromaticAberration { intensity: 6.0 })
    });
    let (fringed, _) = frame(&small(), &ca);
    let diff_at = |x0: u32, x1: u32| {
        let mut d = 0u32;
        for y in 40..140 {
            for x in x0..x1 {
                let (a, b) = (px(&plain, x, y), px(&fringed, x, y));
                d += a[0].abs_diff(b[0]) as u32 + a[2].abs_diff(b[2]) as u32;
            }
        }
        d
    };
    let (edge, centre) = (diff_at(20, 60), diff_at(140, 180));
    println!("aberration: near the edge {edge}, at the centre {centre}");
    assert!(edge > 3 * centre.max(1));
    ca.post.chromatic_aberration = None;

    let grainy = |tick: u64| {
        let mut s = base_scene_with(|s| {
            s.post.film_grain = Some(FilmGrain {
                intensity: 0.1,
                size: 1.0,
            })
        });
        s.tick = tick;
        frame(&small(), &s).0
    };
    let (g1, g1b, g2) = (grainy(1), grainy(1), grainy(2));
    assert_eq!(g1.pixels, g1b.pixels, "grain is a function of the tick");
    assert!(
        differing(&g1, &g2, 2.0) > 10_000,
        "grain did not change with the tick"
    );
    assert!(differing(&g1, &plain, 2.0) > 10_000);

    let mut f = colour_scene();
    f.post.flash = Some(ScreenFlash {
        color: Vec3::new(1.0, 0.0, 0.0),
        amount: 1.0,
    });
    let (flashed, _) = frame(&small(), &f);
    assert!(flashed
        .pixels
        .chunks_exact(4)
        .all(|p| p[0] == 255 && p[1] == 0 && p[2] == 0));
}

fn base_scene_with(f: impl Fn(&mut RenderScene)) -> RenderScene {
    let mut s = colour_scene();
    s.environment.background = Background::Color(Vec3::splat(0.5));
    f(&mut s);
    s
}

/// Camera shake moves the view (not the image): the frame shifts, and a
/// zero shake is the unshaken frame; trauma shake is deterministic per tick.
#[test]
fn camera_shake_moves_the_view() {
    let scene = colour_scene();
    let (still, _) = frame(&small(), &scene);
    let mut shaken = colour_scene();
    shaken.shake = CameraShake {
        offset: Vec2::new(0.05, 0.0),
        roll_degrees: 0.0,
    };
    let (moved, _) = frame(&small(), &shaken);
    // 5% of the view height (5 units) is 0.25 units: about 9 pixels.
    let edge = |img: &RgbaImage| {
        (0..320)
            .find(|&x| px(img, x, 90)[0] > 150)
            .expect("red bar")
    };
    let (a, b) = (edge(&still), edge(&moved));
    println!("red bar's left edge: {a} -> {b}");
    assert!((7..=11).contains(&(a as i32 - b as i32)));
    shaken.shake = CameraShake::default();
    assert_eq!(frame(&small(), &shaken).0.pixels, still.pixels);
    let s = |t| CameraShake::trauma(0.8, t, 60.0, 9, 0.05, 3.0, 10.0);
    assert_eq!(s(17), s(17));
    assert_ne!(s(17), s(18));
    assert_eq!(
        CameraShake::trauma(0.0, 5, 60.0, 9, 0.05, 3.0, 10.0),
        CameraShake::default()
    );
}

// ---------------------------------------------------------------------------
// Ambient occlusion, depth of field, motion blur, god rays.
// ---------------------------------------------------------------------------

/// Both AO methods darken the contact under a sphere and the wall corner
/// while open ground barely changes.
#[test]
fn ambient_occlusion_darkens_contacts() {
    let desc = scenes::base_desc();
    let mut off = fx::ao_scene(AoMethod::Gtao);
    off.post.ao = None;
    let (plain, plain_stats) = frame(&desc, &off);
    for method in [AoMethod::Gtao, AoMethod::Ssao] {
        let scene = fx::ao_scene(method);
        let (img, stats) = frame(&desc, &scene);
        save(&format!("ao-{method:?}"), &img);
        assert!(stats.prepass_draws > 0 && stats.graph_passes >= plain_stats.graph_passes + 3);
        let contact = project(&scene.camera, 1280, 720, Vec3::new(0.3 + 0.52, 0.03, 0.2));
        let open = project(&scene.camera, 1280, 720, Vec3::new(1.5, 0.0, 2.0));
        let c = (
            luma(px(&img, contact.0, contact.1)),
            luma(px(&plain, contact.0, contact.1)),
        );
        let o = (
            luma(px(&img, open.0, open.1)),
            luma(px(&plain, open.0, open.1)),
        );
        println!(
            "{method:?}: contact {} vs {}, open {} vs {}",
            c.0, c.1, o.0, o.1
        );
        assert!(c.0 < c.1 - 6.0, "{method:?} did not darken the contact");
        assert!(
            (o.0 - o.1).abs() < 0.04 * o.1 + 2.0,
            "{method:?} darkened open ground"
        );
    }
}

/// Local contrast: mean squared difference between horizontal neighbours
/// (a blur spreads an edge's step over more pixels, which lowers it).
fn sharpness(img: &RgbaImage, r: (u32, u32, u32, u32)) -> f32 {
    let mut sum = 0.0;
    let mut n = 0.0;
    for y in r.1..r.3 {
        for x in r.0..r.2 - 1 {
            sum += (luma(px(img, x, y)) - luma(px(img, x + 1, y))).powi(2);
            n += 1.0;
        }
    }
    sum / n
}

/// Depth of field keeps the focus band sharp and blurs the near ground; the
/// high quality gathers more taps (a different, smoother result).
#[test]
fn depth_of_field_blurs_outside_the_focus_band() {
    let desc = scenes::base_desc();
    let low = fx::dof_scene(DofQuality::Low).expect("scene");
    let (blurred, _) = frame(&desc, &low);
    let mut off = fx::dof_scene(DofQuality::Low).expect("scene");
    off.post.depth_of_field = None;
    let (sharp, _) = frame(&desc, &off);
    let (high, _) = frame(&desc, &fx::dof_scene(DofQuality::High).expect("scene"));
    // The near ground (bottom rows) and the focused figure (z = 2 - 2.6 * 2... the 3rd).
    let near = (200, 650, 1080, 715);
    let (n_on, n_off) = (sharpness(&blurred, near), sharpness(&sharp, near));
    println!("near ground sharpness: dof {n_on}, off {n_off}");
    assert!(n_on < 0.5 * n_off);
    let focus = project(
        &low.camera,
        1280,
        720,
        Vec3::new(-2.0 + 0.8 * 2.0, 1.0, 2.0 - 2.6 * 2.0),
    );
    let r = (focus.0 - 30, focus.1 - 30, focus.0 + 30, focus.1 + 30);
    let (f_on, f_off) = (sharpness(&blurred, r), sharpness(&sharp, r));
    println!("focused figure sharpness: dof {f_on}, off {f_off}");
    assert!(f_on > 0.7 * f_off);
    assert!(
        differing(&blurred, &high, 3.0) > 1_000,
        "the quality levels draw the same"
    );
}

/// With the previous transforms equal to the current ones and a still
/// camera, motion blur changes nothing; a moving object smears.
#[test]
fn motion_blur_follows_object_and_camera_motion() {
    let desc = small();
    let mut still = fx::motion_blur_scene();
    still.camera = Camera {
        position: Vec3::new(0.0, 2.4, 6.0),
        target: Vec3::new(0.0, 0.8, -1.0),
        ..Default::default()
    };
    still.previous_transforms = still.instances.iter().map(|i| i.transform).collect();
    let (a, _, _) = run(&desc, &mut still, 2, |_, _| {});
    let mut none = fx::motion_blur_scene();
    none.camera = still.camera;
    none.post.motion_blur = None;
    let (b, _, _) = run(&desc, &mut none, 2, |_, _| {});
    assert_eq!(a.pixels, b.pixels, "static motion blur altered the frame");
    let mut moving = fx::motion_blur_scene();
    let (m, _, _) = run(&desc, &mut moving, 4, fx::move_blur);
    let mut moving_off = fx::motion_blur_scene();
    moving_off.post.motion_blur = None;
    let (m0, _, _) = run(&desc, &mut moving_off, 4, fx::move_blur);
    save("motion-blur", &m);
    let whole = (0, 0, 320, 180);
    let (s1, s0) = (sharpness(&m, whole), sharpness(&m0, whole));
    println!("sharpness with motion blur {s1}, without {s0}");
    assert!(s1 < 0.8 * s0);
}

/// God rays brighten the shafts between columns; with the sun behind the
/// camera nothing changes.
#[test]
fn god_rays_shine_between_the_columns() {
    let desc = small();
    let scene = fx::god_rays_scene();
    let (rays, _) = frame(&desc, &scene);
    let mut off = fx::god_rays_scene();
    off.post.god_rays = None;
    let (plain, _) = frame(&desc, &off);
    let gain = region(&rays, (100, 40, 220, 120)) - region(&plain, (100, 40, 220, 120));
    println!("god rays: +{gain} luma around the sun");
    assert!(gain > 8.0);
    let mut behind = fx::god_rays_scene();
    behind.light.direction_to_light = Vec3::new(0.1, 0.3, 1.0);
    let (b_rays, _) = frame(&desc, &behind);
    behind.post.god_rays = None;
    let (b_plain, _) = frame(&desc, &behind);
    assert_eq!(differing(&b_rays, &b_plain, 1.0), 0);
}

// ---------------------------------------------------------------------------
// Stylised: outlines, toon, rim, dissolve, flash.
// ---------------------------------------------------------------------------

fn sphere_scene(m: StandardMaterial) -> RenderScene {
    let mut scene = RenderScene::new();
    let sphere = scene.add_mesh(shapes::sphere(48, 32));
    let mat = scene.add_standard_material(m);
    scene.add_instance(sphere, mat, Mat4::from_scale(Vec3::splat(2.0)));
    scene.environment.background = Background::Color(Vec3::new(0.6, 0.7, 0.9));
    scene.camera = Camera {
        position: Vec3::new(0.0, 0.0, 4.0),
        target: Vec3::ZERO,
        exposure: Exposure::linear(0.8),
        ..Default::default()
    };
    scene
}

fn red() -> StandardMaterial {
    StandardMaterial {
        color: Vec3::new(0.8, 0.2, 0.15),
        roughness: 0.5,
        ..Default::default()
    }
}

/// A material outline draws a dark ring inside the silhouette of its width.
#[test]
fn outlines_follow_silhouettes() {
    let (plain, _) = frame(&small(), &sphere_scene(red()));
    let (lined, stats) = frame(
        &small(),
        &sphere_scene(StandardMaterial {
            outlineWidth: 3.0,
            outlineColor: Vec3::ZERO,
            ..red()
        }),
    );
    assert!(stats.prepass_draws > 0);
    // The rim at the sphere's right edge (radius 1 at distance 4).
    let row = 90;
    let edge = (160..320)
        .rev()
        .find(|&x| px(&plain, x, row) != px(&plain, 319, row))
        .expect("edge");
    let inside = edge - 1;
    println!(
        "edge at {edge}: plain {:?} outlined {:?}",
        px(&plain, inside, row),
        px(&lined, inside, row)
    );
    assert!(luma(px(&lined, inside, row)) < 15.0);
    assert!(
        luma(px(&lined, 160, 90)) > 30.0,
        "the outline covered the middle"
    );
    // The scene-wide outline alone works too.
    let mut global = sphere_scene(red());
    global.post.outline = Some(Outline::default());
    let (g, _) = frame(&small(), &global);
    println!(
        "scene-wide outline around the edge: {:?}",
        (edge - 4..=edge + 1)
            .map(|x| px(&g, x, row))
            .collect::<Vec<_>>()
    );
    assert!(luma(px(&g, edge, row)) < 30.0 && luma(px(&g, inside, row)) < 30.0);
}

/// Toon shading quantizes the light: far fewer distinct shades across the
/// lit sphere than PBR; rim light brightens the edge; flash and dissolve.
#[test]
fn toon_rim_flash_and_dissolve() {
    let shades = |img: &RgbaImage| {
        let mut set = std::collections::BTreeSet::new();
        for x in 100..220 {
            set.insert(luma(px(img, x, 90)) as u32 / 3);
        }
        set.len()
    };
    let (pbr, _) = frame(&small(), &sphere_scene(red()));
    let (toon, _) = frame(
        &small(),
        &sphere_scene(StandardMaterial { toon: 1, ..red() }),
    );
    println!(
        "distinct shades: pbr {}, toon {}",
        shades(&pbr),
        shades(&toon)
    );
    assert!(shades(&toon) * 2 < shades(&pbr));
    let (rim, _) = frame(
        &small(),
        &sphere_scene(StandardMaterial {
            rimIntensity: 2.0,
            ..red()
        }),
    );
    let edge = (160..320)
        .rev()
        .find(|&x| px(&pbr, x, 90) != px(&pbr, 319, 90))
        .expect("edge")
        - 2;
    assert!(luma(px(&rim, edge, 90)) > luma(px(&pbr, edge, 90)) + 20.0);
    let (flash, _) = frame(
        &small(),
        &sphere_scene(StandardMaterial {
            flash: 1.0,
            ..red()
        }),
    );
    assert!(px(&flash, 160, 90).iter().all(|&c| c > 220));
    // Dissolve: 1 is gone (the background), 0.5 has holes and a glowing edge.
    let (gone, _) = frame(
        &small(),
        &sphere_scene(StandardMaterial {
            dissolve: 1.0,
            ..red()
        }),
    );
    assert_eq!(px(&gone, 160, 90), px(&gone, 2, 2));
    let (half, _) = frame(
        &small(),
        &sphere_scene(StandardMaterial {
            dissolve: 0.5,
            ..red()
        }),
    );
    let disc: Vec<[u8; 3]> = (130..190)
        .flat_map(|x| (60..120).map(move |y| (x, y)))
        .map(|(x, y)| px(&half, x, y))
        .collect();
    let holes = disc.iter().filter(|&&p| p == px(&half, 2, 2)).count();
    let glowing = disc.iter().filter(|p| p[0] > 240 && p[1] > 120).count();
    println!("dissolve 0.5: {holes} background pixels, {glowing} glowing");
    assert!(holes > 300 && glowing > 20);
}

// ---------------------------------------------------------------------------
// Skinning and blendshapes.
// ---------------------------------------------------------------------------

/// The procedurally skinned cylinder: an identity palette draws the bind
/// pose (the same pixels as the unskinned mesh), a bent palette moves it,
/// shadows follow, out-of-view instances are culled with widened bounds.
#[test]
fn skinned_cylinder_bends_with_its_palette() {
    let desc = small();
    let (mesh, skin) = fx::skinned_cylinder(0.3, 24, 24);
    let build = |palette: JointPalette, skinned: bool| {
        let mut scene = RenderScene::new();
        let ground = scene.add_mesh(shapes::plane(10.0, 1.0));
        let white = scene.add_material(Material::default());
        scene.add_instance(ground, white, Mat4::IDENTITY);
        let m = scene.add_material(Material {
            base_color: [0.8, 0.3, 0.5, 1.0],
            ..Default::default()
        });
        if skinned {
            let h = scene.add_skinned_mesh(mesh.clone(), skin.clone());
            scene.joint_palettes.push(palette);
            scene
                .skinned_instances
                .push(SkinnedInstance::new(h, m, Mat4::IDENTITY, 0));
        } else {
            let h = scene.add_mesh(mesh.clone());
            scene.add_instance(h, m, Mat4::IDENTITY);
        }
        scene.camera = Camera {
            position: Vec3::new(0.0, 1.4, 5.0),
            target: Vec3::new(0.0, 1.2, 0.0),
            ..Default::default()
        };
        scene
    };
    let identity = JointPalette::from(vec![Mat4::IDENTITY; fx::CYLINDER_BONES]);
    let (rest, stats) = frame(&desc, &build(identity.clone(), true));
    let (plain, _) = frame(&desc, &build(identity, false));
    assert_eq!(
        (stats.skinned_instances, stats.joints),
        (1, fx::CYLINDER_BONES as u32)
    );
    let d = differing(&rest, &plain, 4.0);
    println!("bind pose vs unskinned: {d} pixels differ");
    assert!(d < 40, "the bind pose is not the mesh ({d} pixels)");
    let (bent, _) = frame(&desc, &build(fx::bend_palette(0.5, 0.0), true));
    save("skinned-bent", &bent);
    let moved = differing(&rest, &bent, 8.0);
    assert!(moved > 1_500, "bending moved {moved} pixels");
    // Bending to the left (+z rotation): the top leans towards -x.
    // The cylinder's pink (red and blue above green): its mean x above the
    // base moves left when bent (+z rotation leans towards -x).
    let mean_x = |img: &RgbaImage| {
        let (mut sum, mut n) = (0.0f32, 0.0f32);
        for y in 0..180 {
            for x in 0..320 {
                let [r, g, b] = px(img, x, y).map(i32::from);
                if r > g + 30 && b > g + 10 && y < 120 {
                    sum += x as f32;
                    n += 1.0;
                }
            }
        }
        sum / n.max(1.0)
    };
    let (rx, bx) = (mean_x(&rest), mean_x(&bent));
    println!("mean x of the cylinder: rest {rx}, bent {bx}");
    assert!(bx + 10.0 < rx);
    // Far outside the view: culled.
    let mut away = build(fx::bend_palette(0.3, 0.0), true);
    away.skinned_instances[0].transform = Mat4::from_translation(Vec3::new(60.0, 0.0, 0.0));
    let (_, s) = frame(&desc, &away);
    assert_eq!(s.culled, 1);
}

/// Blendshapes: a target that pushes the cylinder's top up draws it taller
/// at weight 1 and unchanged at weight 0 (also on a mesh without a skin).
#[test]
fn blendshapes_move_vertices_by_their_weights() {
    let desc = small();
    let (mesh, _) = fx::skinned_cylinder(0.3, 24, 24);
    let deltas: Vec<Vec3> = mesh
        .vertices
        .iter()
        .map(|v| Vec3::new(0.0, v.position[1] * 0.4, 0.0))
        .collect();
    let build = |w: f32| {
        let mut scene = RenderScene::new();
        let m = scene.add_material(Material {
            base_color: [0.8, 0.3, 0.5, 1.0],
            ..Default::default()
        });
        let h = scene.add_mesh(mesh.clone());
        assert!(scene.set_morph_targets(
            h,
            MorphTargets {
                positions: vec![deltas.clone()],
                normals: None,
            }
        ));
        let mut inst = SkinnedInstance::new(h, m, Mat4::IDENTITY, 0);
        inst.morph_weights[0] = w;
        scene.joint_palettes.push(JointPalette::default());
        scene.skinned_instances.push(inst);
        scene.environment.background = Background::Color(Vec3::ZERO);
        scene.camera = Camera {
            position: Vec3::new(0.0, 1.6, 6.0),
            target: Vec3::new(0.0, 1.6, 0.0),
            ..Default::default()
        };
        scene
    };
    let top = |img: &RgbaImage| {
        (0..180)
            .find(|&y| (0..320).any(|x| luma(px(img, x, y)) > 20.0))
            .expect("drawn")
    };
    let (rest, _) = frame(&desc, &build(0.0));
    let (tall, _) = frame(&desc, &build(1.0));
    println!("top row: rest {}, weight 1 {}", top(&rest), top(&tall));
    assert!(top(&tall) + 10 < top(&rest));
    // A bad target count is refused.
    let mut bad = RenderScene::new();
    let h = bad.add_mesh(mesh.clone());
    bad.set_morph_targets(
        h,
        MorphTargets {
            positions: vec![vec![Vec3::ZERO; 3]],
            normals: None,
        },
    );
    let mut r = Renderer::new_blocking(&desc).expect("renderer");
    assert!(r.render(&bad, &RenderTarget::Offscreen).is_err());
}

/// Particles, trails and the post stack all off: no extra passes (the
/// features-off budget), counted from the frame graph.
#[test]
fn effects_off_add_no_passes() {
    let built = scenes::build("gltf-reference").expect("scene");
    let (_, stats) = frame(&built.desc, &built.scene);
    let mut with = scenes::build("gltf-reference").expect("scene");
    with.scene.post.ao = Some(AmbientOcclusion::default());
    with.scene.post.depth_of_field = Some(DepthOfField::default());
    with.scene.post.motion_blur = Some(MotionBlur::default());
    with.scene.post.god_rays = Some(GodRays::default());
    let (_, all) = frame(&with.desc, &with.scene);
    println!(
        "graph passes: off {}, on {}",
        stats.graph_passes, all.graph_passes
    );
    assert_eq!(stats.prepass_draws, 0);
    assert_eq!(all.graph_passes, stats.graph_passes + 1 + 2 + 3 + 1 + 2);
    let _ = Curve::default();
}
