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
    scene.fog = None;
    scene.environment.background = Background::Color(Vec3::new(1.0, 0.0, 1.0));
    scene.post.tonemap = Tonemap::None;
    scene
}

/// The skyline's row in each of the frame's middle columns (the first
/// row from the top that is not the background), and its spread about
/// the straight line that fits it best (pixels, RMS).
fn skyline_roughness(img: &RgbaImage) -> f32 {
    let (x0, x1) = (img.width / 8, img.width * 7 / 8);
    let rows: Vec<(f32, f32)> = (x0..x1)
        .map(|x| {
            let y = (0..img.height)
                .find(|&y| !magenta(px(img, x, y)))
                .unwrap_or(img.height);
            (x as f32, y as f32)
        })
        .collect();
    let n = rows.len() as f32;
    let (mx, my) = rows
        .iter()
        .fold((0.0, 0.0), |a, p| (a.0 + p.0 / n, a.1 + p.1 / n));
    let (mut sxy, mut sxx) = (0.0, 0.0);
    for (x, y) in &rows {
        sxy += (x - mx) * (y - my);
        sxx += (x - mx) * (x - mx);
    }
    let slope = sxy / sxx;
    let sq: f32 = rows
        .iter()
        .map(|(x, y)| (y - (my + slope * (x - mx))).powi(2))
        .sum();
    (sq / n).sqrt()
}

/// The crest 4 m out is the skyline. Undisplaced it is a straight line
/// (the ground is the same all along x); with 10 cm of cobbles it rises
/// and falls over each cobble, several pixels at that distance.
#[test]
fn displacement_roughens_the_skyline() {
    let report = check("terrain-displaced");
    assert!(report.stats.terrain_draws > 0);
    let (flat, _) = render(&on_magenta(ts::displaced_scene(0.0)));
    let (cobbled, _) = render(&on_magenta(ts::displaced_scene(ts::DISPLACED_RELIEF)));
    keep("displaced-skyline-flat", &flat);
    keep("displaced-skyline-cobbled", &cobbled);
    let (before, after) = (skyline_roughness(&flat), skyline_roughness(&cobbled));
    assert!(before < 0.5, "the flat crest's skyline strays {before} px");
    assert!(
        after > 2.0,
        "the cobbled crest's skyline strays only {after} px"
    );
}

/// Looking down at the cobbles on the crest from 2 m up, the levels finer
/// than the samples that carry the displacement meet without a crack
/// wherever the camera stands (across their snapping steps): no
/// background shows through the ground.
#[test]
fn displaced_levels_leave_no_cracks() {
    let mut scene = on_magenta(ts::displaced_scene(ts::DISPLACED_RELIEF));
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    for step in 0..6 {
        let x = step as f32 * 0.37;
        let z = ts::DISPLACED_CREST + 1.0 + step as f32 * 0.29;
        scene.camera.position = Vec3::new(x, ts::displaced_height(z) + 2.0, z);
        scene.camera.target = Vec3::new(x + 0.3, ts::displaced_height(z - 1.0), z - 1.0);
        renderer
            .render(&scene, &RenderTarget::Offscreen)
            .expect("render");
        let img = renderer.read_offscreen().expect("capture");
        if step == 0 {
            keep("displaced-cracks", &img);
        }
        let holes = (0..img.height)
            .flat_map(|y| (0..img.width).map(move |x| (x, y)))
            .filter(|&(x, y)| magenta(px(&img, x, y)))
            .count();
        assert_eq!(holes, 0, "step {step}: background through the ground");
    }
}
