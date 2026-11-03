//! R5's water (`crates/og-render/README.md` §Water): each scene matches its
//! reference for this backend tier, and behaviour read from the frames: a
//! wind raises a sea (much more light and dark across the water than the
//! same water becalmed) that moves with the tick; a gale's steep crests
//! foam white where a round swell of the same waves does not; the low sun
//! lays a glitter path on the sea under it; a flow map that holds one
//! direction everywhere draws the stream that flow draws without one.

use glam::{Vec2, Vec3};
use og_render::{RenderScene, RenderTarget, Renderer, RgbaImage, TextureData};
use og_tests::gfx;
use og_tests::ocean_scenes::{OCEAN_SCENES, flow_map_scene, ocean_scene};
use og_tests::scenes;
use std::path::PathBuf;

fn check(scene: &str) {
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-gfx");
    match gfx::check(scene, &out) {
        Ok(report) => println!("{report}"),
        Err(e) => panic!("{e}"),
    }
}

fn frame(scene: &RenderScene) -> RgbaImage {
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    r.render(scene, &RenderTarget::Offscreen).expect("render");
    r.read_offscreen().expect("capture")
}

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

/// The luma of every pixel of a rectangle.
fn lumas(img: &RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32) -> Vec<f32> {
    (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| (x, y)))
        .map(|(x, y)| luma(px(img, x, y)))
        .collect()
}

fn mean(l: &[f32]) -> f32 {
    l.iter().sum::<f32>() / l.len() as f32
}

/// The mean luma step between neighbouring pixels over a rectangle: the
/// fine light and dark of waves, which a mirror's smooth reflection of the
/// sky has little of.
fn texture(img: &RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32) -> f32 {
    let mut sum = 0.0;
    for y in y0..y1 - 1 {
        for x in x0..x1 - 1 {
            let l = luma(px(img, x, y));
            sum += (luma(px(img, x + 1, y)) - l).abs() + (luma(px(img, x, y + 1)) - l).abs();
        }
    }
    sum / ((x1 - x0 - 1) * (y1 - y0 - 1)) as f32
}

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

/// `name` with its sea's wind and choppiness replaced.
fn with_sea(name: &str, wind: Option<Vec2>, choppiness: Option<f32>) -> RenderScene {
    let mut s = ocean_scene(name);
    for w in &mut s.water {
        if let Some(v) = wind {
            w.wind = v;
        }
        if let Some(c) = choppiness {
            w.choppiness = c;
        }
    }
    s
}

#[test]
fn ocean_scenes_match_their_references_and_read_as_a_sea() {
    for scene in OCEAN_SCENES {
        check(scene);
    }
    // The breeze's sea against the same water becalmed (a mirror of the
    // sky): the waves' lit and shaded faces break its light up pixel to
    // pixel. (The spread of the luma alone, 1.42x on Metal, mostly measured
    // the sky's own gradient in both.)
    let sea = frame(&ocean_scene("water-ocean"));
    let calm = frame(&with_sea("water-ocean", Some(Vec2::ZERO), None));
    let water = |img: &RgbaImage| texture(img, 0, 420, 1280, 720);
    assert!(
        water(&sea) > 3.0 * water(&calm),
        "a wind raises a sea: {} vs {}",
        water(&sea),
        water(&calm)
    );
    // Half a second later the waves have moved on.
    let mut later = ocean_scene("water-ocean");
    later.tick += 30;
    let moved = difference(&frame(&later), &sea);
    assert!(moved > 1.0, "the sea moves with the tick: {moved}");

    // The gale's crests foam white; the same waves as a round swell
    // (choppiness 0) do not. Both without their reflection, so no mirror
    // of the bright overcast counts as foam.
    let matte = |mut s: RenderScene| {
        for w in &mut s.water {
            w.reflectivity = 0.0;
        }
        frame(&s)
    };
    let gale = matte(ocean_scene("water-ocean-storm"));
    let round = matte(with_sea("water-ocean-storm", None, Some(0.0)));
    let foam = |img: &RgbaImage| {
        let band = lumas(img, 0, 380, 1280, 720);
        let m = mean(&band);
        (380..720)
            .flat_map(|y| (0..1280).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                let p = px(img, x, y);
                let lo = p[0].min(p[1]).min(p[2]);
                let hi = p[0].max(p[1]).max(p[2]);
                luma(p) > 1.5 * m && lo > 0.75 * hi
            })
            .count()
    };
    assert!(
        foam(&gale) > 4 * foam(&round) + 500,
        "whitecaps on the gale's crests: {} vs {}",
        foam(&gale),
        foam(&round)
    );

    // The sun's glitter path: the column of sea under the low sun (the
    // camera faces it) is much brighter than the sea to the side.
    let sunset = frame(&ocean_scene("water-ocean-sunset"));
    let path = mean(&lumas(&sunset, 600, 400, 680, 620));
    let side = mean(&lumas(&sunset, 160, 400, 240, 620));
    assert!(
        path > 1.5 * side,
        "a glitter path under the sun: {path} vs {side}"
    );
}

/// The sea under a dim sky is no brighter than that sky: its body is lit by
/// the light that reaches it and its reflection mirrors the sky as drawn
/// (not lifted by the twilight's adaptation, which only the surfaces take).
/// Before, the sea glowed a daylight teal under a dusk storm and the night.
#[test]
fn the_sea_darkens_with_the_sky() {
    for name in ["water-ocean-dusk-storm", "water-ocean-night"] {
        check(name);
        let img = frame(&ocean_scene(name));
        let sky = mean(&lumas(&img, 0, 60, 1280, 260));
        let sea = mean(&lumas(&img, 0, 440, 1280, 720));
        assert!(sea < sky, "{name}: the sea ({sea}) under its sky ({sky})");
    }
}

/// The boats sample's floaters (tests/evidence/r5-water/boats) sit in the
/// sea the GPU draws: at their poses at 5 s and 6.5 s
/// (`boats/waterline.txt`, the run's dump), the drawn surface's height
/// under each, read back from the GPU (`Renderer::sea_heights`), has the
/// tick's waves drawn as og.physics.waterHeight floats them on (to 2 cm:
/// og_core::ocean at the tick's grid cell) and every buoy crosses the drawn
/// surface. Prints each floater's clearance against the drawn sea and the
/// tick's, and the highest keel corner: the cascades' chop (3 to 14 m
/// waves the tick does not float hulls on) can still bare a corner by a few
/// centimetres, which hull wave suppression is to close.
#[test]
fn the_floaters_sit_in_the_drawn_sea() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/evidence/r5-water/boats/waterline.txt"
    ))
    .expect("the floaters' poses");
    let water = og_render::Water {
        size: Vec2::splat(6000.0),
        segments: 256,
        wind: Vec2::new(7.0, -18.7),
        choppiness: 1.0,
        foam_width: 0.0,
        depth_fade: 30.0,
        wave_strength: 0.0,
        ..Default::default()
    };
    let sea = og_core::ocean::Ocean::new(water.wind, water.swell, water.choppiness).expect("a sea");
    // og-physics' WaterBody::surface: the grid the tick floats bodies on.
    let extent = water.size.max_element();
    let even = extent / og_core::ocean::grid_segments(water.segments, extent) as f32;
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let mut worst_keel = f32::MIN;
    for tick in [300u64, 390] {
        let mut scene = RenderScene::new();
        scene.tick = tick;
        scene.tick_rate = 60.0;
        scene.water.push(water.clone());
        scene.camera = og_render::Camera {
            position: Vec3::new(0.0, 10.0, 0.0),
            target: Vec3::new(-4.0, 0.0, -60.0),
            far: 3000.0,
            ..Default::default()
        };
        renderer
            .render(&scene, &RenderTarget::Offscreen)
            .expect("render");
        let seconds = tick as f64 / 60.0;
        // (name, point, kind): a buoy's centre, or a hull's keel corner.
        let mut points: Vec<(String, Vec3, bool)> = Vec::new();
        for line in text.lines().filter(|l| !l.starts_with('#')) {
            let f: Vec<&str> = line.split_whitespace().collect();
            if f[0].parse::<u64>() != Ok(tick) {
                continue;
            }
            let v: Vec<f32> = f[2..]
                .iter()
                .map(|x| x.parse().expect("a number"))
                .collect();
            let at = Vec3::new(v[0], v[1], v[2]);
            let q = glam::Quat::from_xyzw(v[3], v[4], v[5], v[6]);
            if f[1].starts_with("boat") {
                for (x, z) in [(-1.2, -3.0), (1.2, -3.0), (-1.2, 3.0), (1.2, 3.0)] {
                    points.push((f[1].to_string(), at + q * Vec3::new(x, -0.5, z), true));
                }
            } else {
                points.push((f[1].to_string(), at, false));
            }
        }
        let xz: Vec<Vec2> = points.iter().map(|(_, p, _)| Vec2::new(p.x, p.z)).collect();
        let drawn = renderer.sea_heights(&xz).expect("the sea's heights");
        assert_eq!(drawn.len(), points.len(), "a sea was drawn");
        for ((name, p, keel), [height, swell, cascades]) in points.iter().zip(&drawn) {
            let cell = og_core::ocean::grid_cell(Vec2::new(p.x, p.z), water.size * 0.5, even);
            let tick_height = sea.height_on_grid(
                glam::DVec2::new(f64::from(p.x), f64::from(p.z)),
                seconds,
                cell,
            );
            let what = if *keel { "keel corner" } else { "centre" };
            println!(
                "{seconds} s {name} {what}: {:+.3} m over the drawn sea ({:+.3} the tick's waves, {:+.3} the cascades), {:+.3} m over the tick's",
                p.y - height,
                swell,
                cascades,
                p.y - tick_height
            );
            assert!(
                (swell - tick_height).abs() < 0.02,
                "{seconds} s {name}: the tick's waves drawn at {swell} m, og.physics.waterHeight {tick_height} m"
            );
            if *keel {
                worst_keel = worst_keel.max(p.y - height);
            } else {
                assert!(
                    (p.y - height).abs() < 0.6,
                    "{seconds} s {name}: its centre {} m off the drawn sea (radius 0.6)",
                    p.y - height
                );
            }
        }
    }
    println!("highest keel corner over the drawn sea: {worst_keel:+.3} m");
}

/// A one-pixel flow map pointing all one way (rg `* 2 - 1`).
fn uniform_map(scene: &mut RenderScene, rg: [u8; 2]) -> og_render::TextureHandle {
    scene.add_texture(TextureData {
        width: 4,
        height: 4,
        rgba: [rg[0], rg[1], 128, 255].repeat(16),
        color_space: og_render::ColorSpace::Linear,
        mips: og_render::Mips::None,
    })
}

#[test]
fn a_flow_map_turns_the_flow_and_one_way_is_the_flow_itself() {
    check("water-flow-map");
    // The stream without its map, flowing straight down at the far field's
    // speed, differs from the mapped one: the map turns the waves round the
    // boulder.
    let mapped = frame(&ocean_scene("water-flow-map"));
    let mut plain = RenderScene::new();
    flow_map_scene(&mut plain, false);
    let straight = frame(&plain);
    assert!(
        difference(&mapped, &straight) > 0.5,
        "the map turns the flow: {}",
        difference(&mapped, &straight)
    );
    // A map pointing down the stream everywhere (r 0.5, g 0: -z) at speed
    // 0.25 draws what the flow (0, -0.25) draws, with a normal map and
    // with the travelling waves (no normal map) alike.
    for normal_map in [true, false] {
        let make = |map: bool| {
            let mut s = RenderScene::new();
            flow_map_scene(&mut s, false);
            let handle = map.then(|| uniform_map(&mut s, [128, 0]));
            let ripples = normal_map.then(|| og_tests::fork_scenes::ripple_map(&mut s));
            let w = &mut s.water[0];
            w.normal_map = ripples;
            w.flow_map = handle;
            w.flow = Vec2::new(0.0, -0.25);
            s
        };
        let one_way = frame(&make(true));
        let flow = frame(&make(false));
        let d = difference(&one_way, &flow);
        assert!(
            d < 0.6,
            "a one-way map is the flow (normal map {normal_map}): {d}"
        );
        // And it is the flow's direction that matters: the same map turned
        // across the stream (+x) draws something else.
        let across = {
            let mut s = make(false);
            let h = uniform_map(&mut s, [255, 128]);
            s.water[0].flow_map = Some(h);
            frame(&s)
        };
        assert!(
            difference(&across, &flow) > 2.0 * d + 0.3,
            "a map across the stream is not the flow down it (normal map {normal_map})"
        );
    }
}
