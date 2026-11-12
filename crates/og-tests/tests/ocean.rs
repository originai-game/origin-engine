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
/// of og_core::ocean, the rest point found as the tick finds it) is
/// og.physics.waterHeight (`og_core::ocean::Ocean::height`) to 2 cm, and
/// the surface the clipmap's vertices draw differs from it only by what
/// the renderer adds or cannot hold (the FFT's shorter waves, the waves a
/// level's spacing leaves out): under 6% of the sea's height at any point.
/// A body the tick floats there sits in the drawn water with nothing
/// calmed or corrected round it. Prints each point's heights.
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
        let sea = og_core::ocean::Ocean::new(wind, water.swell, choppiness).expect("a sea");
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
                if added.abs() >= 0.06 * hs + 0.02 {
                    failures.push(format!(
                        "{wind} {seconds} s {p}: drawn {added} m off the sea's function on a {hs} m sea"
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
    let sea = og_core::ocean::Ocean::new(water.wind, water.swell, water.choppiness).expect("a sea");
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
