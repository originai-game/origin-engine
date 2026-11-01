use super::*;

/// `terrain-generated`: a terrain the cook made (`generate`) draws as a
/// heightmap's does, its biomes' control map placing the layers and its
/// water at the cooked sea level: open sea at the frame's foot, a beach
/// strip at the island's shore (sand below 7 m, just above the 4 m sea),
/// meadows, rock on the slopes and snow on the top, then its reference.
#[test]
fn a_generated_terrain_draws_its_sea_shore_and_biomes() {
    let r = gfx::render("terrain-generated").expect("render");
    keep("terrain-generated", &r.image);
    assert!(r.stats.terrain_draws > 0, "{:?}", r.stats);
    let img = &r.image;
    for (x0, x1) in [(40, 240), (1040, 1240)] {
        let mut c = [0.0f32; 3];
        for y in 620..700 {
            for x in x0..x1 {
                let p = px(img, x, y);
                (0..3).for_each(|k| c[k] += p[k] / (200.0 * 80.0));
            }
        }
        assert!(c[2] > c[0] + 30.0, "the sea at x {x0}..{x1}: {c:?}");
    }
    let (mut sand, mut snow, mut grass) = (0, 0, 0);
    for y in 280..560 {
        for x in 320..960 {
            let [r, g, b] = px(img, x, y);
            if r.min(g).min(b) > 215.0 {
                snow += 1;
            } else if r >= g && g >= b && r - b >= 18.0 && r > 165.0 {
                sand += 1;
            } else if g > r + 8.0 && g > b + 8.0 {
                grass += 1;
            }
        }
    }
    // Read on lavapipe: 1,485 sand, 747 snow and 29,436 grass pixels.
    assert!(sand > 700, "the beach: {sand} pixels");
    assert!(snow > 300, "the snow: {snow} pixels");
    assert!(grass > 15000, "the meadows: {grass} pixels");
    check("terrain-generated");
}
