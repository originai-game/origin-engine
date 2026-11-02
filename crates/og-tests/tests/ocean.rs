//! R5's water (`crates/og-render/README.md` §Water): each scene matches its
//! reference for this backend tier, and behaviour read from the frames: a
//! wind raises a sea (much more light and dark across the water than the
//! same water becalmed) that moves with the tick; a gale's steep crests
//! foam white where a round swell of the same waves does not; the low sun
//! lays a glitter path on the sea under it; a flow map that holds one
//! direction everywhere draws the stream that flow draws without one.

use glam::Vec2;
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

/// The standard deviation of the luma over a rectangle.
fn contrast(img: &RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32) -> f32 {
    let l = lumas(img, x0, y0, x1, y1);
    let m = mean(&l);
    (l.iter().map(|v| (v - m).powi(2)).sum::<f32>() / l.len() as f32).sqrt()
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
    // The breeze's sea against the same water becalmed (the detail ripples
    // alone): the swell's lit and shaded faces spread the water's light.
    let sea = frame(&ocean_scene("water-ocean"));
    let calm = frame(&with_sea("water-ocean", Some(Vec2::ZERO), None));
    let water = |img: &RgbaImage| contrast(img, 0, 420, 1280, 720);
    assert!(
        water(&sea) > 1.5 * water(&calm),
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
    // (choppiness 0) do not.
    let gale = frame(&ocean_scene("water-ocean-storm"));
    let round = frame(&with_sea("water-ocean-storm", None, Some(0.0)));
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
    // The stream without its map, flowing straight down at the same speed,
    // differs from the mapped one: the map turns the ripples round the
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
            let w = &mut s.water[0];
            if !normal_map {
                w.normal_map = None;
            }
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
