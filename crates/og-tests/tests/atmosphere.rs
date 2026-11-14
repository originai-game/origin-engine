//! The E5 round's rendering (`crates/og-render/README.md` §Atmosphere,
//! §Water, §Material shaders): each scene matches its reference for this
//! backend tier, and behaviour read from rendered frames: fog hides the
//! distance, not the foreground; the day-night look darkens from noon to
//! dusk to night and draws stars only at night; water darkens with depth,
//! foams where it meets geometry, ripples with its height field and
//! reflects; the 2D water line follows its surface and draws in the sprite
//! order; metaballs merge particles within reach and keep apart those that
//! are not; a dissolve shader cuts holes by its amount and the grass
//! shader moves only the grass with the clock.

use glam::{Mat4, Vec2, Vec3};
use og_render::{Camera, RenderScene, RenderTarget, Renderer, RgbaImage};
use og_tests::scenes;
use std::path::PathBuf;

fn px(img: &RgbaImage, x: u32, y: u32) -> [f32; 3] {
    let i = ((y * img.width + x) * 4) as usize;
    [
        img.pixels[i] as f32,
        img.pixels[i + 1] as f32,
        img.pixels[i + 2] as f32,
    ]
}

fn luma(p: [f32; 3]) -> f32 {
    0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2]
}

/// A mesh replaced with RenderScene::set_mesh (cloth) draws its new shape
/// on the next frame, in place when the sizes match and in new buffers when
/// they do not; a skinned mesh is refused.
#[test]
#[ignore = "requires GPU"]
fn a_replaced_mesh_draws_its_new_shape() {
    let mut s = RenderScene::new();
    let quad = |x: f32, w: f32| og_render::MeshData {
        vertices: [(x - w, -1.0), (x + w, -1.0), (x + w, 1.0), (x - w, 1.0)]
            .iter()
            .map(|&(px, py)| og_render::Vertex {
                tangent: [0.0; 4],
                position: [px, py, 0.0],
                normal: [0.0, 0.0, 1.0],
                ..Default::default()
            })
            .collect(),
        indices: vec![0, 1, 2, 0, 2, 3],
    };
    let mesh = s.add_mesh(quad(-2.0, 1.0));
    let white = s.add_standard_material(og_render::StandardMaterial {
        unlit: 1,
        side: 2,
        ..Default::default()
    });
    s.add_instance(mesh, white, Mat4::IDENTITY);
    s.lighting.environment.background = og_render::Background::Color(Vec3::ZERO);
    s.frame.camera = Camera {
        position: Vec3::new(0.0, 0.0, 8.0),
        target: Vec3::ZERO,
        ..Default::default()
    };
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let mut draw = |s: &RenderScene| {
        r.render(s, &RenderTarget::Offscreen).expect("render");
        r.read_offscreen().expect("capture")
    };
    let lit = |img: &RgbaImage, x: u32| luma(px(img, x, 360)) > 100.0;
    let a = draw(&s);
    assert!(lit(&a, 420) && !lit(&a, 860), "the quad on the left");
    assert!(s.set_mesh(mesh, quad(2.0, 1.0)));
    let b = draw(&s);
    assert!(!lit(&b, 420) && lit(&b, 860), "moved in place to the right");
    let mut wide = quad(0.0, 3.0);
    wide.vertices.push(wide.vertices[0]);
    assert!(s.set_mesh(mesh, wide));
    let c = draw(&s);
    assert!(lit(&c, 420) && lit(&c, 860), "a bigger mesh in new buffers");
    let skinned = s.add_skinned_mesh(
        quad(0.0, 1.0),
        og_render::SkinData {
            joints: vec![[0; 4]; 4],
            weights: vec![[1.0, 0.0, 0.0, 0.0]; 4],
        },
    );
    assert!(
        !s.set_mesh(skinned, quad(0.0, 1.0)),
        "skinned meshes are refused"
    );
    // Draw refinement changes the GPU geometry, never the submitted solve.
    // TAA exercises the surface-only prepass and its previous-position stream.
    let mut desc = scenes::base_desc();
    desc.antialias = og_render::Antialias::Taa;
    let mut r = Renderer::new_blocking(&desc).expect("surface renderer");
    for (level, x, triangles) in [(0, -2.0, 8), (3, 2.0, 18), (1, -2.0, 2)] {
        let coarse = quad(x, 1.0);
        s.objects.cloth_subdivisions = level;
        assert!(s.set_simulation_surface(
            mesh,
            coarse.clone(),
            og_render::SimulationSurface::SoftBody
        ));
        let stats = r
            .render(&s, &RenderTarget::Offscreen)
            .expect("surface render");
        assert_eq!(stats.triangles, triangles);
        assert!(
            stats.prepass_draws > 0,
            "vertex-only motion needs TAA vectors"
        );
        assert_eq!(s.mesh(mesh), Some(&coarse));
        let image = r.read_offscreen().expect("surface capture");
        assert!(lit(&image, if x < 0.0 { 420 } else { 860 }));
        // Hold the subdivision level while moving the coarse solve, so the
        // second frame updates existing buffers rather than recreating them.
        assert!(s.set_simulation_surface(
            mesh,
            quad(-x, 1.0),
            og_render::SimulationSurface::SoftBody
        ));
        let stats = r
            .render(&s, &RenderTarget::Offscreen)
            .expect("surface moved");
        assert_eq!(stats.triangles, triangles);
        let image = r.read_offscreen().expect("moved surface capture");
        assert!(lit(&image, if x < 0.0 { 860 } else { 420 }));
    }
}

/// A disc alone against black, so neither scattered sky nor automatic
/// exposure can hide a lost HDR peak or manufacture its bloom halo.
fn solar_probe() -> RenderScene {
    let mut scene = RenderScene::new();
    let sun = Vec3::new(0.0, 0.5, -0.8660254);
    scene.lighting.environment.sky.sun_direction = sun;
    scene.lighting.environment.sky.sun_color = Vec3::ONE;
    scene.lighting.environment.sky.look = Some(og_render::SkyLook {
        physical: Some(og_render::PhysicalAtmosphere::EARTH),
        sun_size: 2.0,
        sun_intensity: 40.0,
        brightness: 0.0,
        stars: 0.0,
        moon_size: 0.0,
        ..Default::default()
    });
    scene.lighting.environment.background = og_render::Background::Procedural {
        sun_direction: sun,
        sun_color: Vec3::ONE,
    };
    scene.frame.camera = Camera {
        position: Vec3::ZERO,
        target: sun,
        projection: og_render::Projection::Perspective {
            fov_y_degrees: 30.0,
        },
        exposure: og_render::Exposure::Manual { ev: -6.0 },
        ..Default::default()
    };
    scene.look.post.tonemap = og_render::Tonemap::None;
    scene
}

fn linear_pixel(image: &RgbaImage, x: u32, y: u32) -> [f32; 3] {
    px(image, x, y).map(|v| {
        let s = v / 255.0;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    })
}

#[test]
#[ignore = "requires GPU"]
fn a_moving_hdr_sun_stays_stable_through_temporal_resolves() {
    for (name, aa, scale) in [
        ("taa", og_render::Antialias::Taa, 1.0),
        ("taau", og_render::Antialias::Taa, 0.67),
        ("smaa-t2x", og_render::Antialias::SmaaT2x, 1.0),
    ] {
        let mut scene = solar_probe();
        scene
            .lighting
            .environment
            .sky
            .look
            .as_mut()
            .unwrap()
            .sun_size = 0.5;
        scene.frame.camera.projection = og_render::Projection::Perspective {
            fov_y_degrees: 50.0,
        };
        let sun = scene.lighting.environment.sky.sun_direction;
        let mut renderer = Renderer::new_blocking(&og_render::RenderDesc {
            width: 320,
            height: 180,
            antialias: aa,
            ..scenes::base_desc()
        })
        .unwrap();
        renderer.set_render_scale(scale);
        let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join("solar-hdr")
            .join(name);
        std::fs::create_dir_all(&out).unwrap();
        let mut energy = Vec::new();
        let mut largest_lag = 0.0f32;
        for frame in 0..48 {
            // Warm history first, then pan continuously through subpixels.
            let angle = (frame as f32 - 8.0).max(0.0) * 0.002;
            scene.frame.camera.target = glam::Quat::from_rotation_y(angle) * sun;
            scene.frame.tick = frame;
            renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
            let image = renderer.read_offscreen().unwrap();
            std::fs::write(out.join(format!("{frame:03}.png")), og_render::png(&image)).unwrap();
            if frame < 8 {
                continue;
            }
            let forward = scene.frame.camera.target.normalize();
            let right = forward.cross(scene.frame.camera.up).normalize();
            let up = right.cross(forward);
            let z = sun.dot(forward) * 25f32.to_radians().tan();
            let expected = Vec2::new(
                159.5 + sun.dot(right) / z * 90.0,
                89.5 - sun.dot(up) / z * 90.0,
            );
            let mut total = 0.0;
            let mut centroid = Vec2::ZERO;
            let mut trail = 0.0;
            for y in 0..180 {
                for x in 0..320 {
                    let w = luma(linear_pixel(&image, x, y));
                    total += w;
                    let p = Vec2::new(x as f32, y as f32);
                    centroid += p * w;
                    if p.distance(expected) > 6.0 {
                        trail += w;
                    }
                }
            }
            assert!(total > 0.2, "{name}: solar radiance disappeared");
            largest_lag = largest_lag.max((centroid / total).distance(expected));
            assert!(trail < total * 0.01, "{name}: a detached HDR history trail");
            energy.push(total);
        }
        let min = energy.iter().copied().fold(f32::INFINITY, f32::min);
        let max = energy.iter().copied().fold(0.0, f32::max);
        println!(
            "{name}: solar energy {min}..{max}, max centroid error {largest_lag}px; frames {}",
            out.display()
        );
        assert!(max < min * 1.6, "{name}: subpixel solar energy flickers");
        assert!(largest_lag < 2.0, "{name}: solar history lags the camera");
    }
}

/// The clouds have a place in the world and keep their shapes as the
/// clock runs. Drawn over the fair-weather sky looking up:
///
/// - the frame a tick before the wind's offset wraps (four tiles of the
///   noise) and the frame a tick after differ as two neighbouring ticks
///   anywhere do. The shape, the blend and the high layer were read at
///   scales four tiles are no period of, so every cloud changed shape at
///   the wrap (every 50 minutes of clock at the default wind);
/// - once the wind has carried the clouds 1200 m, an eye 1200 m downwind
///   sees what the eye at the origin saw before: they stand over the
///   world, where they followed the eye before (the same sky over every
///   place), and they go with the wind, where they went against it.
#[test]
#[ignore = "requires GPU"]
fn the_clouds_stand_over_the_world_and_keep_their_shapes() {
    let draw_with = |tick: u64, eye_x: f32, high: bool| {
        let mut scene = og_tests::sky_scenes::sky_scene("clouds-fair");
        let c = scene.atmosphere.clouds.as_mut().expect("clouds");
        c.wind = Vec3::new(12.0, 0.0, 0.0);
        if !high {
            c.layer = 0.0;
        }
        scene.objects.instances.clear();
        scene.frame.tick = tick;
        scene.frame.tick_rate = 60.0;
        scene.frame.camera = Camera {
            position: Vec3::new(eye_x, 2.0, 0.0),
            target: Vec3::new(eye_x, 2.0 + 1.0, -1.0),
            ..Default::default()
        };
        let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
        renderer
            .render_capture(&scene, &RenderTarget::Offscreen)
            .expect("render");
        renderer.read_offscreen().expect("capture")
    };
    let draw = |tick: u64, eye_x: f32| draw_with(tick, eye_x, true);
    let apart = |a: &RgbaImage, b: &RgbaImage| {
        let sum: f64 = a
            .pixels
            .iter()
            .zip(&b.pixels)
            .map(|(x, y)| f64::from(x.abs_diff(*y)))
            .sum();
        sum / a.pixels.len() as f64
    };
    // The default scale's wrap: 36 km at 12 m/s, 3000 s in.
    let scale = og_render::Clouds::default().scale;
    let wrap = (f64::from(scale) * 4.0 / 12.0 * 60.0) as u64;
    let steady = apart(&draw(1000, 0.0), &draw(1001, 0.0));
    let across = apart(&draw(wrap - 1, 0.0), &draw(wrap, 0.0));
    let later = apart(&draw(1000, 0.0), &draw(1000 + 100 * 60, 0.0));
    println!("one tick apart: {steady:.3}; across the wrap: {across:.3}; 100 s apart: {later:.3}");
    assert!(
        later > 2.0,
        "the clouds do not move with the wind ({later:.3})"
    );
    assert!(
        across < 2.0 * steady + 0.2,
        "the clouds change shape where the wind's offset wraps ({across:.3} against {steady:.3} a tick)"
    );
    // 100 s of a 12 m/s wind is 1200 m (the high layer's wind is twice
    // that, so without it).
    let carried = draw_with(1000 + 100 * 60, 1200.0, false);
    let walked = apart(&carried, &draw_with(1000, 0.0, false));
    println!(
        "the eye 1200 m downwind of the clouds carried 1200 m, against the eye and the clouds before: {walked:.3}"
    );
    assert!(
        walked < 2.0 * steady + 0.2,
        "the clouds follow the eye or go against the wind ({walked:.3} against {steady:.3} a tick)"
    );
}
