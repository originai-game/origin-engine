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
use og_tests::harness::{luma_f32, out_dir, px_f32};
use og_tests::scenes;

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
    let lit = |img: &RgbaImage, x: u32| luma_f32(px_f32(img, x, 360)) > 100.0;
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
        // The counts stay on the GPU until a reader asks for them.
        r.request_culling_counts();
        let mut stats = r
            .render(&s, &RenderTarget::Offscreen)
            .expect("surface render");
        r.finish_stats(&mut stats);
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
    px_f32(image, x, y).map(|v| {
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
        let out = out_dir("solar-hdr").join(name);
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
                    let w = luma_f32(linear_pixel(&image, x, y));
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

/// The clouds' history goes with a renderer snapshot
/// (`og_render::Renderer::save_state`): the display frames after a restore
/// are the frames the uninterrupted run drew next, byte for byte, though
/// frames looking elsewhere were drawn between. Each frame marches a quarter
/// of the cloud target and reprojects the rest from the frame before, so a
/// history left out of the snapshot carries the other view's clouds into the
/// restored one.
#[test]
#[ignore = "requires GPU"]
fn a_restore_brings_back_the_clouds_history() {
    let mut scene = og_tests::sky_scenes::sky_scene("clouds-fair");
    scene.objects.instances.clear();
    let draw = |r: &mut Renderer, s: &RenderScene, frames: u32| {
        for _ in 0..frames {
            r.render(s, &RenderTarget::Offscreen).expect("render");
        }
        r.read_offscreen().expect("capture")
    };
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    // Every pipeline in its full variant first (a display frame draws with a
    // leaner one until it has compiled).
    r.render_capture(&scene, &RenderTarget::Offscreen)
        .expect("capture");
    draw(&mut r, &scene, 3);
    let saved = r.save_state();
    let next = draw(&mut r, &scene, 2);
    let camera = scene.frame.camera;
    scene.frame.camera.target += Vec3::new(4.0, -0.5, 2.0);
    draw(&mut r, &scene, 3);
    scene.frame.camera = camera;
    assert_eq!(r.restore_state(saved), og_render::Restored::Whole);
    let again = draw(&mut r, &scene, 2);
    assert!(
        again.pixels == next.pixels,
        "the frames after the restore are not the uninterrupted run's"
    );
}

/// The clouds cast their shadows on the world (`og_render::Clouds::shadows`):
/// - a place on the ground is sunlit where the sun is seen from it between
///   the clouds and shaded where a cloud stands before it, at places
///   kilometres apart, after the wind has carried the clouds 12 km and
///   under a sun well off the zenith: the shadow lies under its cloud,
///   along the sun's ray, and goes with it;
/// - without the shadows every place is lit alike.
#[test]
#[ignore = "requires GPU"]
fn the_clouds_shade_the_ground_under_them() {
    let scene_at = |place: Vec3, shadows: f32| {
        let mut scene = og_tests::sky_scenes::sky_scene("clouds-fair");
        let c = scene.atmosphere.clouds.as_mut().expect("clouds");
        c.wind = Vec3::new(12.0, 0.0, 5.0);
        c.layer = 0.0;
        c.shadows = shadows;
        scene.atmosphere.adaptation = Default::default();
        scene.objects.instances.clear();
        let ground = scene.add_mesh(og_render::shapes::plane(200.0, 1.0));
        let grey = scene.add_standard_material(og_render::StandardMaterial {
            color: Vec3::splat(0.35),
            roughness: 0.9,
            ..Default::default()
        });
        scene.add_instance(ground, grey, Mat4::from_translation(place));
        scene.frame.tick = 60_000;
        scene.frame.tick_rate = 60.0;
        scene
    };
    let capture = |scene: &RenderScene| {
        let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
        renderer
            .render_capture(scene, &RenderTarget::Offscreen)
            .expect("render");
        renderer.read_offscreen().expect("capture")
    };
    // The middle of the frame.
    let middle = |img: &RgbaImage| {
        let (cx, cy) = (img.width / 2, img.height / 2);
        let mut sum = 0.0;
        for y in cy - 8..=cy + 8 {
            for x in cx - 8..=cx + 8 {
                sum += luma_f32(px_f32(img, x, y));
            }
        }
        sum / (17.0 * 17.0)
    };
    // The ground under the eye, seen from 30 m over it.
    let ground = |place: Vec3, shadows: f32| {
        let mut scene = scene_at(place, shadows);
        scene.frame.camera = Camera {
            position: place + Vec3::Y * 30.0,
            target: place,
            up: Vec3::NEG_Z,
            ..Default::default()
        };
        middle(&capture(&scene))
    };
    // The sun from that place, in a narrow view, exposed so its disc does
    // not clip (at the scene's own exposure a thin cloud before it still
    // drew it white).
    let sun = |place: Vec3| {
        let mut scene = scene_at(place, 1.0);
        let to_light = scene.lighting.light.direction_to_light.normalize();
        scene.frame.camera = Camera {
            position: place + Vec3::Y * 2.0,
            target: place + Vec3::Y * 2.0 + to_light,
            projection: og_render::Projection::Perspective { fov_y_degrees: 8.0 },
            exposure: og_render::Exposure::Manual { ev: -9.0 },
            ..Default::default()
        };
        middle(&capture(&scene))
    };
    let places: Vec<Vec3> = (0..16)
        .map(|i| Vec3::new(i as f32 * 1700.0 - 9000.0, 0.0, i as f32 * 900.0 - 3000.0))
        .collect();
    let mut rows = Vec::new();
    for p in &places {
        let sky_mean = sun(*p);
        let lit = ground(*p, 1.0);
        let plain = ground(*p, 0.0);
        println!(
            "at {:>6.0} {:>6.0}: sun {sky_mean:6.1}, ground {lit:6.1}, without shadows {plain:6.1}",
            p.x, p.z
        );
        rows.push((sky_mean, lit, plain));
    }
    // The sun's disc fills the middle of the narrow view: about as bright as
    // the brightest where nothing stands before it (a wisp that dims it by a
    // fifth or less is finer than the shadow map's 70 m texel), a quarter of
    // that or less behind a cloud, and between the two behind a cloud's thin
    // edge.
    let open_sun = rows.iter().map(|r| r.0).fold(0.0, f32::max);
    let open = |sky: f32| sky > 0.8 * open_sun;
    let covered = |sky: f32| sky < 0.25 * open_sun;
    let shaded = rows.iter().filter(|r| covered(r.0)).count();
    let clear = rows.iter().filter(|r| open(r.0)).count();
    assert!(
        shaded >= 3 && clear >= 3,
        "{shaded} of {} places under a cloud, {clear} under an open sun: the places do not tell the shadows from the sun",
        rows.len()
    );
    let plain = rows.iter().map(|r| r.2).sum::<f32>() / rows.len() as f32;
    for (p, (sky, lit, unshaded)) in places.iter().zip(&rows) {
        assert!(
            (unshaded - plain).abs() < 0.05 * plain,
            "without the shadows the ground at {p} is lit {unshaded}, {plain} on average"
        );
        if covered(*sky) {
            assert!(
                *lit < 0.75 * unshaded,
                "a cloud stands before the sun at {p} and the ground is lit {lit} of {unshaded}"
            );
        } else if open(*sky) {
            assert!(
                (lit - unshaded).abs() < 0.03 * unshaded,
                "the sun is open at {p} and the ground is lit {lit} of {unshaded}"
            );
        } else {
            assert!(
                *lit < 0.97 * unshaded,
                "a cloud's edge stands before the sun at {p} and the ground is lit {lit} of {unshaded}"
            );
        }
    }
}

/// A fixed aurora and clock must not depend on the GPU workgroups' order,
/// including when all six curtains lie close together.
#[test]
#[ignore = "requires GPU"]
fn an_aurora_at_the_same_tick_has_identical_pixels() {
    for (name, crowded) in [
        ("aurora-arcs", false),
        ("aurora-breakup", false),
        ("aurora-arcs", true),
    ] {
        let mut built = scenes::build(name).expect("aurora scene");
        if crowded {
            let aurora = built.scene.atmosphere.aurora.as_mut().expect("aurora");
            aurora.curtains = 6;
            aurora.spread = 0.0;
        }
        let expected = og_tests::gfx::render_built(&mut built).expect("first render");
        assert!(
            expected
                .image
                .pixels
                .as_chunks::<4>()
                .0
                .iter()
                .any(|p| p[1] > 32)
        );
        for repeat in 1..=4 {
            let actual = og_tests::gfx::render_built(&mut built).expect("repeat render");
            assert!(
                actual.image.pixels == expected.image.pixels,
                "{name}, crowded={crowded}, repeat {repeat}: a fixed tick changed pixels"
            );
        }
    }
}

/// A night cloud takes the aurora's light (sky.wgsl aurora_cloud_light): a
/// closed overcast under a breakup overhead, which hides the display
/// itself, is lit by it through the cloud as diffuse light through a
/// scattering slab, green the most as the display is. The test sets the
/// display and the camera itself, so the aurora scene's composition can
/// move: the breakup's curtains 60 km out, under the overcast's base, and
/// the camera tilted 28° up into them over the colonnade. Over the frame's
/// upper third its linear light rises by between a fortieth and half of
/// what the display adds to the open sky there: the display's mean over
/// the sky above the cloud, a fraction of it through the deck (it was not
/// lit at all, a black cut-out against the aurora).
#[test]
#[ignore = "requires GPU"]
fn a_night_cloud_takes_the_auroras_light() {
    let frame = |cover: Option<f32>, aurora: bool| {
        let mut built = scenes::build("aurora-breakup").expect("aurora scene");
        built.scene.atmosphere.aurora = Some(og_render::Aurora {
            activity: 0.95,
            energy: 0.9,
            intensity: 1.5,
            distance: 60_000.0,
            ..og_render::Aurora::default()
        });
        built.scene.frame.camera.position = Vec3::new(0.0, 2.2, 4.0);
        built.scene.frame.camera.target = Vec3::new(0.0, 25.4, -40.0);
        built.scene.atmosphere.clouds = cover.map(|coverage| og_render::Clouds {
            coverage,
            layer: 0.0,
            ..og_render::Clouds::default()
        });
        if !aurora {
            built.scene.atmosphere.aurora = None;
        }
        og_tests::gfx::render_built(&mut built)
            .expect("render")
            .image
    };
    // The linear light over the frame's upper third, per channel.
    let sky = |img: &RgbaImage| {
        let linear = |v: f32| {
            let c = v / 255.0;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        let mut s = [0.0f32; 3];
        for y in 0..img.height / 3 {
            for x in 0..img.width {
                let p = px_f32(img, x, y);
                (0..3).for_each(|k| s[k] += linear(p[k]));
            }
        }
        s.map(|v| v / (img.width * img.height / 3) as f32)
    };
    let rise = |a: [f32; 3], b: [f32; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let open = rise(sky(&frame(None, true)), sky(&frame(None, false)));
    let clouded = rise(
        sky(&frame(Some(0.97), true)),
        sky(&frame(Some(0.97), false)),
    );
    println!("the aurora adds {open:?} to the open sky, {clouded:?} to the overcast");
    assert!(
        luma_f32(clouded) > luma_f32(open) / 40.0
            && luma_f32(clouded) < luma_f32(open) / 2.0
            && clouded[1] > clouded[0]
            && clouded[1] > clouded[2],
        "the overcast under the aurora rises {clouded:?}, the open sky {open:?}"
    );
}

/// Far cumulus at night under og.DayNight's moon (Infinity World's sea at
/// 23:24) is lit by the air in front of it as the sky behind it is: by
/// the moon and the night's floor, not by the set sun alone. Seen level
/// over a night sea under a high full moon, the band of far cloud over
/// the horizon keeps over a third of the clear sky's light there, and the
/// moonlit air's blue (it was a near-black, brown band, cut out of the
/// sky down to the horizon: the air the cloud hides was replaced by air
/// lit by the sun under the horizon).
#[test]
#[ignore = "requires GPU"]
fn far_night_clouds_take_the_moonlit_air() {
    use og_render::{Background, Environment, Exposure, ProceduralSky};
    let frame = |cover: Option<f32>| {
        let mut built = scenes::build("water-ocean-night").expect("night sea");
        let look = og_render::day_night(23.4, 25.0, 90.0)
            .with_moon(1.6, 0.5)
            .with_atmosphere(&og_render::PhysicalAtmosphere::EARTH);
        let scene = &mut built.scene;
        let key = look.key();
        scene.lighting.light.direction_to_light = key.direction_to_light;
        scene.lighting.light.color = key.color;
        scene.lighting.light.intensity = key.intensity;
        scene.lighting.environment = Environment {
            sky: ProceduralSky {
                sun_direction: look.sun_direction,
                sun_color: look.sun_color,
                look: Some(look.sky),
            },
            intensity: look.ambient,
            background: Background::Procedural {
                sun_direction: look.sun_direction,
                sun_color: look.sun_color,
            },
            ..Environment::default()
        };
        scene.atmosphere.adaptation = look.adaptation;
        scene.atmosphere.fog = None;
        scene.atmosphere.clouds = cover.map(|coverage| og_render::Clouds {
            coverage,
            ..og_render::Clouds::default()
        });
        // Level, 10 m over the sea, looking north with the moon high in
        // the south-east behind.
        scene.frame.camera = Camera {
            position: Vec3::new(0.0, 10.0, 0.0),
            target: Vec3::new(0.0, 10.0, -100.0),
            far: 5000.0,
            exposure: Exposure::linear(1.5),
            ..Default::default()
        };
        og_tests::gfx::render_built(&mut built)
            .expect("render")
            .image
    };
    let linear = |v: f32| {
        let c = v / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    // The mean linear light over rows `y0..y1`.
    let rows = |img: &RgbaImage, y0: u32, y1: u32| {
        let mut s = [0.0f32; 3];
        for y in y0..y1 {
            for x in 0..img.width {
                let p = px_f32(img, x, y);
                (0..3).for_each(|k| s[k] += linear(p[k]));
            }
        }
        s.map(|v| v / (img.width * (y1 - y0)) as f32)
    };
    let (clear, clouded) = (frame(None), frame(Some(0.34)));
    let h = clear.height;
    // The 20 rows over the horizon, where the far cloud stands.
    let band = |img: &RgbaImage| rows(img, h / 2 - 20, h / 2);
    let (open, banked) = (band(&clear), band(&clouded));
    println!("over the horizon: clear {open:?}, under far cumulus {banked:?}");
    assert!(
        luma_f32(banked) > luma_f32(open) / 3.0 && banked[2] > banked[0],
        "the far cloud over the horizon is {banked:?}, the clear sky there {open:?}"
    );
}

/// Under a physical sky at night the clouds are lit by the moon, as by
/// day by the sun: looking towards a high moon (23:24) and a rising one
/// (18:33) over the night sea, the cumulus stand brighter than the
/// night sky they hide (1.20 and 1.23 times its light). They were lit by
/// the set sun alone, so only the sky's ambient reached them and they
/// stood darker than it (0.83 and 0.87).
#[test]
#[ignore = "requires GPU"]
fn the_moon_lights_the_clouds_at_night() {
    use og_render::{Background, Environment, Exposure, ProceduralSky};
    let linear = |v: f32| {
        let c = v / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let mut ratios = Vec::new();
    for hours in [23.4, 18.55] {
        let look = og_render::day_night(hours, 25.0, 90.0)
            .with_moon(1.6, 0.5)
            .with_atmosphere(&og_render::PhysicalAtmosphere::EARTH);
        let key = look.key();
        let moon = key.direction_to_light;
        assert!(
            moon.dot(look.sun_direction) < 0.0,
            "the moon keys {hours} h"
        );
        let frame = |cover: Option<f32>| {
            let mut built = scenes::build("water-ocean-night").expect("night sea");
            let scene = &mut built.scene;
            scene.lighting.light.direction_to_light = key.direction_to_light;
            scene.lighting.light.color = key.color;
            scene.lighting.light.intensity = key.intensity;
            scene.lighting.environment = Environment {
                sky: ProceduralSky {
                    sun_direction: look.sun_direction,
                    sun_color: look.sun_color,
                    look: Some(look.sky),
                },
                intensity: look.ambient,
                background: Background::Procedural {
                    sun_direction: look.sun_direction,
                    sun_color: look.sun_color,
                },
                ..Environment::default()
            };
            scene.atmosphere.adaptation = look.adaptation;
            scene.atmosphere.fog = None;
            scene.atmosphere.clouds = cover.map(|coverage| og_render::Clouds {
                coverage,
                ..og_render::Clouds::default()
            });
            // 10 m over the sea, looking 24° up towards the moon's bearing.
            let bearing = Vec3::new(moon.x, 0.0, moon.z).normalize();
            scene.frame.camera = Camera {
                position: Vec3::new(0.0, 10.0, 0.0),
                target: Vec3::new(0.0, 10.0, 0.0) + (bearing + Vec3::Y * 0.45) * 100.0,
                far: 5000.0,
                exposure: Exposure::linear(1.5),
                ..Default::default()
            };
            og_tests::gfx::render_built(&mut built)
                .expect("render")
                .image
        };
        let (clear, clouded) = (frame(None), frame(Some(0.34)));
        // The sky's pixels the clouds change, over the horizon.
        let (mut sky, mut cloud, mut n) = (0.0, 0.0, 0u32);
        for y in 0..clear.height / 2 {
            for x in 0..clear.width {
                let a = luma_f32(px_f32(&clear, x, y).map(linear));
                let b = luma_f32(px_f32(&clouded, x, y).map(linear));
                if (a - b).abs() > 0.1 * a.max(b) {
                    (sky, cloud, n) = (sky + a, cloud + b, n + 1);
                }
            }
        }
        let ratio = cloud / sky.max(1e-6);
        println!("{hours} h: the clouds {ratio:.3} of the sky they hide ({n} pixels)");
        assert!(
            n * 400 > clear.width * clear.height / 2,
            "{hours} h: only {n} pixels of cloud"
        );
        ratios.push((hours, ratio));
    }
    assert!(
        ratios.iter().all(|&(_, r)| r > 1.0),
        "the moonlit clouds over the night sky they hide (hours, ratio): {ratios:?}"
    );
}
