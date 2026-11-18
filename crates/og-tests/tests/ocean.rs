//! R5's water (`crates/og-render/README.md` §Water): each scene matches its
//! reference for this backend tier, and behaviour read from the frames: a
//! wind raises a sea (much more light and dark across the water than the
//! same water becalmed) that moves with the tick; a gale's steep crests
//! foam white where a round swell of the same waves does not; the low sun
//! lays a glitter path on the sea under it; a flow map that holds one
//! direction everywhere draws the stream that flow draws without one; a
//! boat's wake shows on a lake as on a sea.

use glam::{Vec2, Vec3};
use og_render::{RenderScene, RenderTarget, Renderer, RgbaImage};
use og_tests::scenes;

/// The mean absolute difference of two frames, per channel.
fn difference(a: &RgbaImage, b: &RgbaImage) -> f32 {
    let sum: f64 = a
        .pixels
        .iter()
        .zip(&b.pixels)
        .map(|(x, y)| f64::from((*x as i32 - *y as i32).unsigned_abs()))
        .sum();
    (sum / a.pixels.len() as f64) as f32
}

/// The GPU draws the sea the tick floats bodies on. At points round the
/// camera, in a breeze and in a gale, the sea's own function evaluated on
/// the GPU from the frame's uniforms (`Renderer::sea_heights`: every wave
/// of og_env::ocean, the rest point found as the tick finds it) is
/// og.physics.waterHeight (`og_env::ocean::Ocean::height`) to 2 cm, and
/// the surface the clipmap's vertices draw differs from it only by the
/// waves a level's spacing cannot hold: a centimetre plus a hundredth of the
/// point's distance from the eye, in a breeze and in a gale (the FFT's
/// ripples shade the surface and never move it; when they did, they stood
/// 2 cm RMS off it at 10 m/s and 14 cm at 20). A body the tick floats there
/// sits in the drawn water with nothing calmed or corrected round it.
/// Prints each point's heights.
#[test]
#[ignore = "requires GPU"]
fn the_drawn_sea_is_the_sea_bodies_float_on() {
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let points: Vec<Vec2> = (0..24)
        .map(|i| Vec2::from_angle(i as f32 * 2.4 - 1.6) * (4.0 + 2.5 * i as f32))
        .collect();
    // Every point is printed before anything is asserted, so one mismatch does not hide the rest.
    let mut failures: Vec<String> = Vec::new();
    for (wind, choppiness) in [(Vec2::new(-3.0, -9.5), 0.6), (Vec2::new(7.0, -18.7), 1.0)] {
        let water = og_render::Water {
            size: Vec2::splat(6000.0),
            wind,
            choppiness,
            foam_width: 0.0,
            depth_fade: 30.0,
            wave_strength: 0.0,
            ..Default::default()
        };
        let sea = og_env::ocean::Ocean::new(wind, water.swell, choppiness).expect("a sea");
        let hs = sea.significant_height();
        for tick in [300u64, 390] {
            let mut scene = RenderScene::new();
            scene.frame.tick = tick;
            scene.frame.tick_rate = 60.0;
            scene.world.water.push(water.clone());
            scene.frame.camera = og_render::Camera {
                position: Vec3::new(0.0, 10.0, 0.0),
                target: Vec3::new(-4.0, 0.0, -60.0),
                far: 3000.0,
                ..Default::default()
            };
            renderer
                .render(&scene, &RenderTarget::Offscreen)
                .expect("render");
            let seconds = tick as f64 / 60.0;
            let drawn = renderer.sea_heights(&points).expect("the sea's heights");
            assert_eq!(drawn.len(), points.len(), "a sea was drawn");
            for (p, [height, own, added]) in points.iter().zip(&drawn) {
                let tick_height = sea.height(p.as_dvec2(), seconds);
                println!(
                    "{wind} {seconds} s {p}: drawn {height:+.3} m ({own:+.3} the sea's function, {added:+.3} the renderer's), the tick's {tick_height:+.3} m"
                );
                if (own - tick_height).abs() >= 0.02 {
                    failures.push(format!(
                        "{wind} {seconds} s {p}: the GPU's sea at {own} m, og.physics.waterHeight {tick_height} m"
                    ));
                }
                let out = 0.01 + 0.01 * p.length();
                if added.abs() >= out {
                    failures.push(format!(
                        "{wind} {seconds} s {p}: drawn {added} m off the sea's function on a {hs} m sea (at most {out} m)"
                    ));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// W8: the sea runs at the display's rate. Two frames of one tick drawn at
/// different interpolation alphas draw different seas (the code before drew
/// both at the tick, identically), and at either alpha the sea's function on
/// the GPU is og.physics.waterHeight at that alpha's time, to 2 cm, so
/// floaters blended between ticks stay on it.
#[test]
#[ignore = "requires GPU"]
fn the_sea_moves_between_ticks_in_step_with_the_tick() {
    let water = og_render::Water {
        size: Vec2::splat(6000.0),
        wind: Vec2::new(7.0, -18.7),
        choppiness: 1.0,
        foam_width: 0.0,
        depth_fade: 30.0,
        wave_strength: 0.0,
        ..Default::default()
    };
    let sea = og_env::ocean::Ocean::new(water.wind, water.swell, water.choppiness).expect("a sea");
    let points: Vec<Vec2> = [(0.0, -20.0), (6.0, -35.0), (-9.0, -50.0), (3.0, -80.0)]
        .iter()
        .map(|&(x, z)| Vec2::new(x, z))
        .collect();
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let mut frames = Vec::new();
    for alpha in [0.1f32, 0.6] {
        let mut scene = RenderScene::new();
        scene.frame.tick = 300;
        scene.frame.tick_rate = 60.0;
        scene.frame.interpolation = Some(alpha);
        scene.world.water.push(water.clone());
        scene.frame.camera = og_render::Camera {
            position: Vec3::new(0.0, 10.0, 0.0),
            target: Vec3::new(-4.0, 0.0, -60.0),
            far: 3000.0,
            ..Default::default()
        };
        renderer
            .render(&scene, &RenderTarget::Offscreen)
            .expect("render");
        frames.push(renderer.read_offscreen().expect("capture"));
        let seconds = (299.0 + f64::from(alpha)) / 60.0;
        let drawn = renderer.sea_heights(&points).expect("the sea's heights");
        assert_eq!(drawn.len(), points.len(), "a sea was drawn");
        for (p, [_, own, _]) in points.iter().zip(&drawn) {
            let tick_height = sea.height(p.as_dvec2(), seconds);
            assert!(
                (own - tick_height).abs() < 0.02,
                "alpha {alpha} at {p}: the GPU's sea at {own} m, waterHeight {tick_height} m"
            );
        }
    }
    let d = difference(&frames[0], &frames[1]);
    println!("one tick at alphas 0.1 and 0.6: mean difference {d}");
    assert!(d > 0.5, "the sea did not move between ticks ({d})");
}

/// Any water draws the bodies that cut it, not a sea alone. `water-wake-lake`
/// (a windless lake, a boat at 5 m/s) two seconds in, against the same
/// frames with nothing recorded as cutting the water: the wake's slopes and
/// its foam change the lake astern of the boat, and nowhere else (the
/// frame's top fifth, the far water and the sky, is the same to a level).
#[test]
#[ignore = "requires GPU"]
fn a_lake_draws_the_wake_of_the_boat_that_cuts_it() {
    use og_tests::ocean_scenes::{ocean_scene, wake_update};
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let mut frames = Vec::new();
    // The uncut run first: the renderer then holds no wake when the cut one starts.
    for cut in [false, true] {
        let mut scene = ocean_scene("water-wake-lake");
        for frame in 0..=120 {
            wake_update(&mut scene, frame);
            if !cut {
                scene.world.water[0].wakes = None;
            }
            renderer
                .render(&scene, &RenderTarget::Offscreen)
                .expect("render");
        }
        frames.push(renderer.read_offscreen().expect("capture"));
    }
    let (calm, cut) = (&frames[0], &frames[1]);
    let row = calm.width as usize * 4;
    let far = (calm.height as usize / 5) * row;
    let band = |image: &RgbaImage, from: usize, to: usize| RgbaImage {
        width: image.width,
        height: ((to - from) / row) as u32,
        pixels: image.pixels[from..to].to_vec(),
    };
    let end = calm.pixels.len();
    let near = difference(&band(calm, far, end), &band(cut, far, end));
    let away = difference(&band(calm, 0, far), &band(cut, 0, far));
    println!(
        "a lake with a wake against the same lake calm: mean difference {near} astern, {away} far off"
    );
    assert!(near > 0.2, "the lake drew no wake ({near})");
    assert!(
        away < 0.02,
        "the wake changed the lake far from the boat ({away})"
    );
}

/// The rings the rain stands on the near water, per square metre of it:
/// over the frame's lower half where a pixel holds at least half the rings
/// (its footprint on the level water under 8 mm, as weather_ripples_held
/// reads it; beyond, the rings fade into their slopes' spread, which moves
/// the glints without a ring), the distinct crests the rain draws (pixels
/// where `rained` differs from `dry` by more than 8 in a channel, those
/// within 3 px of each other one crest, so a packet's neighbouring crests
/// count once) against that water's area. Returns the density, the count and
/// the area in square metres.
fn rings_per_square_metre(
    dry: &RgbaImage,
    rained: &RgbaImage,
    camera: &og_render::Camera,
    level: f32,
) -> (f32, usize, f32) {
    let og_render::Projection::Perspective { fov_y_degrees } = camera.projection else {
        panic!("the lake's camera is a perspective one");
    };
    let (w, h) = (dry.width as usize, dry.height as usize);
    let forward = (camera.target - camera.position).normalize();
    let right = forward.cross(camera.up).normalize();
    let up = right.cross(forward);
    let tan = (fov_y_degrees.to_radians() * 0.5).tan();
    // Where pixel (x, y)'s centre ray meets the level water.
    let hit = |x: f32, y: f32| {
        let nx = (x + 0.5) / w as f32 * 2.0 - 1.0;
        let ny = 1.0 - (y + 0.5) / h as f32 * 2.0;
        let ray = forward + right * (nx * tan * w as f32 / h as f32) + up * (ny * tan);
        (ray.y < 0.0).then(|| camera.position + ray * ((level - camera.position.y) / ray.y))
    };
    let mut area = 0.0f64;
    let mut crest = vec![false; w * h];
    for y in h / 2..h {
        for x in 0..w {
            let (xf, yf) = (x as f32, y as f32);
            let (Some(p), Some(px), Some(py)) = (hit(xf, yf), hit(xf + 1.0, yf), hit(xf, yf + 1.0))
            else {
                continue;
            };
            let (dx, dy) = (px - p, py - p);
            if (dx.abs() + dy.abs()).length() >= 0.008 {
                continue;
            }
            area += f64::from(dx.cross(dy).length());
            let i = y * w + x;
            crest[i] = (0..3).any(|c| dry.pixels[i * 4 + c].abs_diff(rained.pixels[i * 4 + c]) > 8);
        }
    }
    let mut rings = 0;
    let mut stack = Vec::new();
    for start in 0..w * h {
        if !crest[start] {
            continue;
        }
        rings += 1;
        crest[start] = false;
        stack.push(start);
        while let Some(i) = stack.pop() {
            let (x, y) = ((i % w) as isize, (i / w) as isize);
            for ny in y - 3..=y + 3 {
                for nx in x - 3..=x + 3 {
                    if nx < 0 || ny < 0 || nx >= w as isize || ny >= h as isize {
                        continue;
                    }
                    let j = ny as usize * w + nx as usize;
                    if crest[j] {
                        crest[j] = false;
                        stack.push(j);
                    }
                }
            }
        }
    }
    let area = area as f32;
    (rings as f32 / area.max(1e-6), rings, area)
}

/// The rain rings a lake as it rings a puddle (weather_rain.wgsl): seen
/// from the bank, the windless lake under SurfaceWeather's rain stands as
/// many rings on its near water as the rain's drops make. The rain drops
/// 67 a square metre a second on standing water (a drop every half second
/// in each 30 cm cell of three offset grids), and each ring stands some
/// 0.4 s before it fades (its front reaching 7 to 10 cm at 20 cm/s): 27
/// rings a square metre at any instant. A ring reads as about one and a
/// half crests (its packet breaks into arcs where the sky it mirrors is
/// even), so 41 crests a square metre; the floor is 36, nine tenths of
/// that. (On lavapipe the model reads 44.3; the model before it, a drop a
/// second in each 33 cm cell of two grids, stood 18 rings and read 27.3;
/// without the rings, 0.6.) Seen
/// from 16 m up, where no pixel holds a ring, their slopes' spread still
/// dulls the lake's mirror. A renderer made without the weather draws
/// either as the dry lake.
#[test]
#[ignore = "requires GPU"]
fn the_rain_rings_a_lake() {
    use og_render::{LitFeatures, RenderDesc, SurfaceWeather};
    use og_tests::ocean_scenes::{ocean_scene, wake_update};
    let draw = |bank: bool, rain: f32, without: LitFeatures| {
        let desc = RenderDesc {
            without,
            ..scenes::base_desc()
        };
        let mut renderer = Renderer::new_blocking(&desc).expect("renderer");
        let mut scene = ocean_scene("water-wake-lake");
        wake_update(&mut scene, 30);
        // The lake alone: no boat to wet, no wake.
        scene.world.water[0].wakes = None;
        scene.objects.instances.clear();
        if bank {
            scene.frame.camera.position = Vec3::new(0.0, 1.6, 4.0);
            scene.frame.camera.target = Vec3::new(0.0, 0.0, -2.0);
        }
        if rain > 0.0 {
            scene.atmosphere.surface_weather = Some(SurfaceWeather {
                wetness: 1.0,
                rain,
                ..Default::default()
            });
        }
        renderer
            .render_capture(&scene, &RenderTarget::Offscreen)
            .expect("render");
        let level = scene.world.water[0].transform.w_axis.y;
        (
            renderer.read_offscreen().expect("capture"),
            scene.frame.camera,
            level,
        )
    };
    for bank in [true, false] {
        let (dry, camera, level) = draw(bank, 0.0, LitFeatures::NONE);
        let (rained, ..) = draw(bank, 1.0, LitFeatures::NONE);
        if let Ok(dir) = std::env::var("OG_RAIN_LAKE_OUT") {
            for (image, name) in [(&dry, "dry"), (&rained, "rain")] {
                og_tests::gfx::save_png(
                    std::path::Path::new(&format!("{dir}/lake-{bank}-{name}.png")),
                    image,
                )
                .expect("save");
            }
        }
        let left_out = difference(&dry, &draw(bank, 1.0, LitFeatures::SURFACE_WEATHER).0);
        if bank {
            let (density, rings, area) = rings_per_square_metre(&dry, &rained, &camera, level);
            println!(
                "a lake in the rain from the bank: {rings} rings on {area} m² of near water, {density} a square metre; made without the weather {left_out}"
            );
            assert!(
                density > 36.0,
                "the rain stood {density} ring crests a square metre on the lake, where its drops stand 41"
            );
        } else {
            // The water: the frame's lower half.
            let half = dry.pixels.len() / 2;
            let lower = |image: &RgbaImage| RgbaImage {
                width: image.width,
                height: image.height / 2,
                pixels: image.pixels[half..].to_vec(),
            };
            let dulled = difference(&lower(&dry), &lower(&rained));
            println!(
                "a lake in the rain from 16 m up against the same lake dry: mean difference {dulled}; made without the weather {left_out}"
            );
            assert!(
                dulled > 0.3,
                "the rain left the far lake as it was ({dulled})"
            );
        }
        assert!(
            left_out == 0.0,
            "a renderer made without the weather still rains on the lake ({left_out})"
        );
    }
}

/// A lamp over the water lights it as it lights every surface: a red point
/// light 3 m above the night sea ahead of the camera reddens the water
/// under it (its facets mirror it and its body scatters it back), where
/// the sea took the sun and the sky alone before.
#[test]
#[ignore = "requires GPU"]
fn a_lamp_over_the_water_lights_it() {
    use og_tests::ocean_scenes::ocean_scene;
    let red_share = |image: &RgbaImage| {
        let row = image.width as usize * 4;
        let mut sum = [0f64; 3];
        for px in image.pixels[image.height as usize / 2 * row..]
            .as_chunks::<4>()
            .0
        {
            for (s, c) in sum.iter_mut().zip(px) {
                *s += f64::from(*c);
            }
        }
        (sum[0] / (sum[0] + sum[1] + sum[2]).max(1.0)) as f32
    };
    let frame = |scene: &RenderScene| {
        let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
        for _ in 0..3 {
            renderer
                .render(scene, &RenderTarget::Offscreen)
                .expect("render");
        }
        renderer.read_offscreen().expect("capture")
    };
    let mut scene = ocean_scene("water-ocean-night");
    let dark = red_share(&frame(&scene));
    scene.lighting.lights.push(og_render::Light {
        position: Vec3::new(0.0, 3.0, -12.0),
        color: Vec3::new(1.0, 0.05, 0.02),
        intensity: 60.0,
        range: 40.0,
        ..Default::default()
    });
    let lit = red_share(&frame(&scene));
    println!("the night sea's red share: {dark} bare, {lit} under the lamp");
    assert!(
        lit > dark + 0.1,
        "the lamp did not light the water ({dark} bare, {lit} lit)"
    );
}
