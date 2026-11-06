//! The UI pass (`docs/ui.md` U1; `src/ui_scenes.rs`): each scene against its
//! reference frame, and what each primitive must draw read from the frame
//! at the pixels where a wrong implementation differs (a corner cut the
//! wrong way, a gradient mixed in sRGB, a ring without its hole, a clip
//! that leaks, a blur that is not one).

use og_tests::gfx::{self, Rendered};
use og_tests::scenes::{self, GfxScene};

fn render(scene: &str) -> Rendered {
    let mut built: GfxScene = scenes::build(scene).expect("scene");
    gfx::render_built(&mut built).expect("render")
}

/// The sRGB bytes of pixel (x, y) (the overlay is 1280×720 CSS px on a
/// 1280×720 frame: one CSS px a pixel).
fn px(r: &Rendered, x: u32, y: u32) -> [f32; 3] {
    let i = ((y * r.image.width + x) * 4) as usize;
    let p = &r.image.pixels;
    [f32::from(p[i]), f32::from(p[i + 1]), f32::from(p[i + 2])]
}

fn srgb8(linear: [f32; 4]) -> [f32; 3] {
    let e = |c: f32| {
        let c = c.clamp(0.0, 1.0);
        255.0
            * if c <= 0.003_130_8 {
                c * 12.92
            } else {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            }
    };
    [e(linear[0]), e(linear[1]), e(linear[2])]
}

fn near(a: [f32; 3], b: [f32; 3], within: f32) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() <= within)
}

/// The cell a primitive sits in (`ui_scenes::cell`).
fn cell(col: u32, row: u32) -> (u32, u32) {
    (40 + col * 200, 40 + row * 170)
}

const BACKING: [f32; 4] = [0.03, 0.035, 0.045, 1.0];

/// Corner shapes cut where their geometry says: 12 px in from a 28 px
/// corner a round corner is inside, a bevel's and a notch's cut leave the
/// card; 4 px in, the squircle (radius 48, continuous) is still outside
/// where the round one of 28 would be too. A ring has its hole and an arc
/// stops at its sweep. The whole UI is a handful of draws.
#[test]
#[ignore = "requires GPU"]
fn ui_shapes_cut_their_corners_rings_and_arcs() {
    let r = render("ui-shapes");
    let back = srgb8(BACKING);
    let ink = |rgb: [u8; 3]| {
        let l = |c: u8| {
            let c = f32::from(c) / 255.0;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        srgb8([l(rgb[0]), l(rgb[1]), l(rgb[2]), 1.0])
    };
    // The shapes' top left corners are 20 px into their cells.
    let corner = |col: u32, d: u32| {
        let (x, y) = cell(col, 0);
        px(&r, x + 20 + d, y + 20 + d)
    };
    assert!(
        near(corner(0, 12), ink([255, 183, 77]), 2.0),
        "round corner: {:?}",
        corner(0, 12)
    );
    assert!(
        near(corner(2, 12), back, 2.0),
        "bevel corner: {:?}",
        corner(2, 12)
    );
    assert!(
        near(corner(3, 12), back, 2.0),
        "notch corner: {:?}",
        corner(3, 12)
    );
    assert!(
        near(corner(1, 4), back, 2.0),
        "squircle corner: {:?}",
        corner(1, 4)
    );
    // Four radii: top left 0 (square), bottom left 60 (cut deep).
    let (x, y) = cell(4, 0);
    assert!(near(px(&r, x + 21, y + 21), ink([239, 83, 80]), 2.0));
    assert!(near(px(&r, x + 24, y + 126), back, 2.0));
    // The ring (cell 2, row 1; 130 px square from (25, 10)) has a hole.
    let (x, y) = cell(2, 1);
    let (cx, cy) = (x + 25 + 65, y + 10 + 65);
    assert!(near(px(&r, cx, cy), back, 2.0), "the ring's hole is drawn");
    assert!(
        near(px(&r, cx + 57, cy), ink([186, 104, 200]), 2.0),
        "the ring"
    );
    // The 120° arc from 12 o'clock: right of centre at 3 o'clock is drawn,
    // left of centre at 9 o'clock is not.
    let (x, y) = cell(4, 1);
    let (cx, cy) = (x + 90, y + 75);
    assert!(
        near(px(&r, cx + 52, cy), ink([239, 83, 80]), 2.0),
        "the arc"
    );
    assert!(near(px(&r, cx - 52, cy), back, 2.0), "past the arc's sweep");
    // The dotted ring (cell 2, row 3; r 65 about (530, 625), dots every
    // 16.1 px from 12 o'clock): its outline is no whole number of periods,
    // so the first dot (x 526..534) overlaps the last (x 520..528). The
    // first dot is round on both sides of the outline's start: x 529 is in
    // it, not cut off at the seam.
    assert!(
        near(px(&r, 529, 560), ink([186, 104, 200]), 2.0),
        "the dot at the dotted ring's start is cut at the seam: {:?}",
        px(&r, 529, 560)
    );
    println!(
        "sprite draws {} sprites {}",
        r.stats.sprite_draws, r.stats.sprites
    );
    assert!(
        r.stats.sprite_draws <= 2,
        "{} UI draws",
        r.stats.sprite_draws
    );
}

/// World-space UI (`docs/ui.md` §14, `ui_scenes::world`): occluders hide a
/// panel behind the wall and keep the one against the sky; the bars draw
/// their back, ghost and fill where their values put them; the mass
/// elements' numbers and names draw at all.
#[test]
#[ignore = "requires GPU"]
fn ui_world_occludes_and_draws_mass_elements() {
    let r = render("ui-world");
    let pink = srgb8([0.9, 0.3, 0.6, 1.0]);
    // The left panel: only sky behind it at 20 m.
    assert!(near(px(&r, 320, 360), pink, 6.0), "{:?}", px(&r, 320, 360));
    // The right one: the wall (12 m) is in front of 20 m.
    assert!(
        !near(px(&r, 640, 360), pink, 40.0),
        "{:?}",
        px(&r, 640, 360)
    );
    // Bars from x = 80, 120 px: the first's fill to 0.5 (green), its
    // ghost to 0.75 (pale), its back past it (dark).
    let fill = px(&r, 110, 86);
    let ghost = px(&r, 160, 86);
    let back = px(&r, 190, 86);
    assert!(fill[1] > fill[0] + 60.0, "fill {fill:?}");
    assert!(ghost[0] > 200.0 && ghost[1] > 180.0, "ghost {ghost:?}");
    // 85 % opaque over the sky (136, 155, 197).
    assert!(back.iter().all(|&c| c < 110.0), "back {back:?}");
    // The third bar's fill reaches 0.9 (x 188) and has no ghost.
    assert!(px(&r, 185, 134)[1] > 120.0, "{:?}", px(&r, 185, 134));
    // The score's glyphs: gold ink in its band at the top centre.
    let gold = (40..90)
        .flat_map(|y| (560..720).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let p = px(&r, x, y);
            // (1, 0.85, 0.3) linear: about (255, 237, 149) in sRGB.
            p[0] > 220.0 && p[1] > 200.0 && p[2] < 180.0
        })
        .count();
    assert!(gold > 200, "{gold} gold pixels in the score");
    // Damage numbers: white, gold and red ink over the lower half.
    let red = (360..720)
        .flat_map(|y| (0..1280).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let p = px(&r, x, y);
            p[0] > 200.0 && p[1] < 120.0 && p[2] < 120.0
        })
        .count();
    assert!(red > 500, "{red} red pixels of damage numbers");
    // Confetti: cyan particles flung around (300, 200), none at its
    // centre any more and none past the farthest travel (about 180 px).
    let cyan = |x0: u32, x1: u32, y0: u32, y1: u32| {
        (y0..y1)
            .flat_map(|y| (x0..x1).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                let p = px(&r, x, y);
                p[0] < 150.0 && p[1] > 220.0 && p[2] > 230.0
            })
            .count()
    };
    let ring = cyan(150, 450, 60, 360);
    assert!(ring > 1500, "{ring} cyan confetti pixels");
    assert!(
        cyan(290, 310, 190, 210) < 20,
        "confetti still at the centre"
    );
}
