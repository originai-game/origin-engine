//! Terrain material v2 read from rendered frames: hex-tiling breaks a
//! layer's repeat, and past `projectSlope` a steep face is sampled by the
//! projections facing it instead of streaking.

use super::*;

/// Per pixel luma of a frame (0..255).
fn lumas(img: &RgbaImage) -> Vec<f32> {
    (0..img.height)
        .flat_map(|y| (0..img.width).map(move |x| (x, y)))
        .map(|(x, y)| luma(px(img, x, y)))
        .collect()
}

/// The correlation of a frame's luma with itself moved by `(dx, dy)`
/// pixels, over the frame's middle half.
fn correlation(img: &RgbaImage, dx: u32, dy: u32) -> f32 {
    let l = lumas(img);
    let w = img.width;
    let (x0, y0) = (img.width / 4, img.height / 4);
    let (x1, y1) = (x0 + img.width / 2, y0 + img.height / 2);
    let (mut a, mut b) = (Vec::new(), Vec::new());
    for y in y0..y1 {
        for x in x0..x1 {
            a.push(l[(y * w + x) as usize]);
            b.push(l[((y + dy) * w + x + dx) as usize]);
        }
    }
    let n = a.len() as f32;
    let (ma, mb) = (a.iter().sum::<f32>() / n, b.iter().sum::<f32>() / n);
    let (mut c, mut va, mut vb) = (0.0, 0.0, 0.0);
    for (p, q) in a.iter().zip(&b) {
        c += (p - ma) * (q - mb);
        va += (p - ma) * (p - ma);
        vb += (q - mb) * (q - mb);
    }
    c / (va * vb).sqrt().max(1e-6)
}

/// A layer's 4 m repeat seen from above: with the plain repeat the frame
/// correlates with itself one period over (about 1); hex-tiling turns and
/// moves each hexagon of it, so the period no longer shows (under 0.5).
#[test]
fn anti_tiling_breaks_the_repeat() {
    let check = check("terrain-tiling");
    assert!(check.stats.terrain_draws > 0);
    let (plain, _) = render(&ts::tiling_scene(0.0));
    let (tiled, _) = render(&ts::tiling_scene(1.0));
    keep("tiling-plain", &plain);
    keep("tiling-hex", &tiled);
    let period = (ts::V2_TILE / ts::TILING_VIEW * plain.height as f32).round() as u32;
    let repeat = |img: &RgbaImage| correlation(img, period, 0).max(correlation(img, 0, period));
    let (before, after) = (repeat(&plain), repeat(&tiled));
    assert!(
        before > 0.9,
        "the plain repeat correlates {before} a period over"
    );
    assert!(
        after < 0.5,
        "hex-tiled, it correlates {after} a period over"
    );
}

#[test]
fn cell_variation_leaves_opaque_normal_maps_unchanged() {
    for anti_tiling in [0.0, 1.0] {
        let mut scene = ts::tiling_scene(anti_tiling);
        let normal = scene.add_texture(og_render::TextureData {
            width: 1,
            height: 1,
            rgba: vec![128, 128, 255, 255],
            mips: og_render::Mips::None,
            color_space: og_render::ColorSpace::Linear,
        });
        scene.terrains[0].layers[0].normal = Some(normal);
        let (plain, _) = render(&scene);
        scene.terrains[0].layers[0].cell_variation = 1.0;
        let (varied, _) = render(&scene);
        assert!(plain.pixels == varied.pixels, "anti_tiling {anti_tiling}");
    }
}

/// How square a frame's pattern is over its middle: the larger of the
/// mean horizontal and vertical luma steps over the smaller (1 for a
/// square checker seen square-on, large for streaks).
fn aspect(img: &RgbaImage) -> f32 {
    let l = lumas(img);
    let w = img.width;
    let (mut ex, mut ey) = (0.0f32, 0.0f32);
    for y in img.height * 3 / 10..img.height * 7 / 10 {
        for x in img.width * 3 / 10..img.width * 7 / 10 {
            let at = |x: u32, y: u32| l[(y * w + x) as usize];
            ex += (at(x + 1, y) - at(x, y)).abs();
            ey += (at(x, y + 1) - at(x, y)).abs();
        }
    }
    ex.max(ey) / ex.min(ey).max(1e-3)
}

/// An 85° wall under a checker: sampled from above its cells smear into
/// vertical streaks (aspect past 5); past `projectSlope` 45 the
/// projections facing the wall keep them square (within 1.5).
#[test]
fn a_steep_face_is_sampled_square() {
    let check = check("terrain-steep");
    assert!(check.stats.terrain_draws > 0);
    let (planar, _) = render(&ts::steep_scene(0.0));
    let (biplanar, _) = render(&ts::steep_scene(45.0));
    keep("steep-planar", &planar);
    keep("steep-biplanar", &biplanar);
    let (before, after) = (aspect(&planar), aspect(&biplanar));
    assert!(before > 5.0, "from above the wall streaks: aspect {before}");
    assert!(
        after < 1.5,
        "biplanar, the wall's cells are square: aspect {after}"
    );
}
