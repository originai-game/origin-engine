//! Displacement (T4) read from rendered frames: a layer's height map moves
//! the drawn ground near the camera, its skyline included, and the finer
//! levels that carry it leave no cracks.

use super::*;

/// Whether a pixel is the magenta background.
fn magenta(p: [f32; 3]) -> bool {
    p[0] > 220.0 && p[1] < 40.0 && p[2] > 220.0
}

/// `scene` against a magenta background, with no fog and no tonemap.
fn on_magenta(mut scene: RenderScene) -> RenderScene {
    scene.atmosphere.fog = None;
    scene.lighting.environment.background = Background::Color(Vec3::new(1.0, 0.0, 1.0));
    scene.look.post.tonemap = Tonemap::None;
    scene
}

/// Looking down at the cobbles on the crest from 2 m up, the levels finer
/// than the samples that carry the displacement meet without a crack
/// wherever the camera stands (across their snapping steps): no
/// background shows through the ground.
#[test]
#[ignore = "requires GPU"]
fn displaced_levels_leave_no_cracks() {
    let mut scene = on_magenta(ts::displaced_scene(ts::DISPLACED_RELIEF));
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    for step in 0..6 {
        let x = step as f32 * 0.37;
        let z = ts::DISPLACED_CREST + 1.0 + step as f32 * 0.29;
        scene.frame.camera.position = Vec3::new(x, ts::displaced_height(z) + 2.0, z);
        scene.frame.camera.target = Vec3::new(x + 0.3, ts::displaced_height(z - 1.0), z - 1.0);
        renderer
            .render(&scene, &RenderTarget::Offscreen)
            .expect("render");
        let img = renderer.read_offscreen().expect("capture");
        if step == 0 {
            save("og-terrain", "displaced-cracks", &img);
        }
        let holes = (0..img.height)
            .flat_map(|y| (0..img.width).map(move |x| (x, y)))
            .filter(|&(x, y)| magenta(px_f32(&img, x, y)))
            .count();
        assert_eq!(holes, 0, "step {step}: background through the ground");
    }
}

/// A display frame draws without a pipeline that is not ready
/// (og-render gpu/deferred.rs): while the pipeline that fills the
/// terrain's corner cache (before the frame's passes) is not ready, the
/// terrain is left out and the frame says so; a frame that waits draws the
/// ground whole. Fails if a pass drew the levels from a cache nothing
/// filled (stray or collapsed triangles over the background), if a frame
/// drawn without the terrain did not count it, or if a frame that waits
/// left it out.
#[test]
#[ignore = "requires GPU"]
fn a_terrain_whose_corner_pipeline_is_not_ready_is_left_out_whole() {
    let mut scene = on_magenta(ts::displaced_scene(ts::DISPLACED_RELIEF));
    let z = ts::DISPLACED_CREST + 1.0;
    scene.frame.camera.position = Vec3::new(0.0, ts::displaced_height(z) + 2.0, z);
    scene.frame.camera.target = Vec3::new(0.3, ts::displaced_height(z - 1.0), z - 1.0);
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    // The background pixels, what the frame left for a later one, and all.
    let mut draw = |display: bool| {
        scene.frame.display = display;
        renderer
            .render(&scene, &RenderTarget::Offscreen)
            .expect("render");
        let img = renderer.read_offscreen().expect("capture");
        let background = (0..img.height)
            .flat_map(|y| (0..img.width).map(move |x| (x, y)))
            .filter(|&(x, y)| magenta(px_f32(&img, x, y)))
            .count();
        (
            background,
            og_render::frame_skipped(),
            (img.width * img.height) as usize,
        )
    };
    // The first frame's corner pipeline compiles on a worker meanwhile: it
    // is either ready in time or left out whole, never half drawn.
    let display = draw(true);
    let waited = draw(false);
    println!("background, skipped: display {display:?}, waited {waited:?}");
    assert!(
        (display.0 == 0 && display.1 == 0) || (display.0 == display.2 && display.1 > 0),
        "a display frame drew the terrain partly, or left it out unsaid: {display:?}"
    );
    assert_eq!(
        (waited.0, waited.1),
        (0, 0),
        "the ground in a frame that waits"
    );
}

/// Each sett its own stone: the layer's `cellVariation` gives each cell its
/// normal map names (its alpha) its own brightness and warmth, so across
/// the near setts' tops (the brighter half of 12-pixel blocks of the lower
/// third of terrain-displaced) the blocks' warmth ((r - b) over their sum)
/// and their brightness (relative to its mean) spread far more than the
/// texture's own tones alone spread them (the scene with `cell_variation`
/// 0: the code before drew that, every stone the same blue-grey).
#[test]
#[ignore = "requires GPU"]
fn each_sett_takes_its_own_tone() {
    let spread = |cells: f32| {
        let mut scene = ts::displaced_scene(ts::DISPLACED_RELIEF);
        scene.world.terrains[0].layers[0].cell_variation = cells;
        let (img, _) = render(&scene);
        let b = 12;
        let mut blocks: Vec<[f32; 3]> = Vec::new();
        for by in (img.height * 2 / 3..img.height - b).step_by(b as usize) {
            for bx in (0..img.width - b).step_by(b as usize) {
                let mut c = [0.0f32; 3];
                for y in by..by + b {
                    for x in bx..bx + b {
                        let p = px_f32(&img, x, y);
                        c = [c[0] + p[0], c[1] + p[1], c[2] + p[2]];
                    }
                }
                blocks.push(c.map(|v| v / (b * b) as f32));
            }
        }
        blocks.sort_by(|a, b| luma_f32(*a).total_cmp(&luma_f32(*b)));
        let tops = &blocks[blocks.len() / 2..];
        let n = tops.len() as f32;
        let std = |f: &dyn Fn(&[f32; 3]) -> f32| {
            let m = tops.iter().map(f).sum::<f32>() / n;
            (
                (tops.iter().map(|c| (f(c) - m).powi(2)).sum::<f32>() / n).sqrt(),
                m,
            )
        };
        let (warm, _) = std(&|c| (c[0] - c[2]) / (c[0] + c[1] + c[2]).max(1.0));
        let (bright, m) = std(&|c| luma_f32(*c));
        (warm, bright / m.max(1.0))
    };
    let (warm0, bright0) = spread(0.0);
    let (warm1, bright1) = spread(1.0);
    println!(
        "setts: warmth spreads {warm0} without cell variation, {warm1} with; brightness {bright0} and {bright1}"
    );
    assert!(
        warm1 > 1.8 * warm0,
        "the setts share one warmth ({warm1} against {warm0})"
    );
    assert!(
        bright1 > 1.3 * bright0,
        "the setts share one brightness ({bright1} against {bright0})"
    );
}
