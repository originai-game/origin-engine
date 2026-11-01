//! The UI pass (`docs/ui.md` U1; `src/ui_scenes.rs`): each scene against its
//! reference frame, and what each primitive must draw read from the frame
//! at the pixels where a wrong implementation differs (a corner cut the
//! wrong way, a gradient mixed in sRGB, a ring without its hole, a clip
//! that leaks, a blur that is not one).

use og_tests::gfx::{self, Rendered};
use og_tests::scenes::{self, GfxScene};
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("gfx");
    std::fs::create_dir_all(&dir).expect("out dir");
    dir
}

fn check(scene: &str) {
    match gfx::check(scene, &out_dir()) {
        Ok(report) => println!("{report}"),
        Err(e) => panic!("{e}"),
    }
}

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

/// The luminance's standard deviation over a region.
fn spread(r: &Rendered, x: std::ops::Range<u32>, y: std::ops::Range<u32>) -> f32 {
    let mut v = Vec::new();
    for py in y {
        for qx in x.clone() {
            let p = px(r, qx, py);
            v.push(0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2]);
        }
    }
    let mean = v.iter().sum::<f32>() / v.len() as f32;
    (v.iter().map(|l| (l - mean).powi(2)).sum::<f32>() / v.len() as f32).sqrt()
}

/// The cell a primitive sits in (`ui_scenes::cell`).
fn cell(col: u32, row: u32) -> (u32, u32) {
    (40 + col * 200, 40 + row * 170)
}

const BACKING: [f32; 4] = [0.03, 0.035, 0.045, 1.0];

#[test]
fn ui_shapes_match_their_reference() {
    check("ui-shapes");
}

#[test]
fn ui_paints_match_their_reference() {
    check("ui-paints");
}

#[test]
fn ui_effects_match_their_reference() {
    check("ui-effects");
}

#[test]
fn ui_layers_match_their_reference() {
    check("ui-layers");
}

#[test]
fn ui_text_effects_match_their_reference() {
    check("ui-text-effects");
}

#[test]
fn ui_vector_matches_its_reference() {
    check("ui-vector");
}

#[test]
fn ui_world_matches_its_reference() {
    check("ui-world");
}

/// Corner shapes cut where their geometry says: 12 px in from a 28 px
/// corner a round corner is inside, a bevel's and a notch's cut leave the
/// card; 4 px in, the squircle (radius 48, continuous) is still outside
/// where the round one of 28 would be too. A ring has its hole and an arc
/// stops at its sweep. The whole UI is a handful of draws.
#[test]
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

/// A gradient mixes in OKLab: blue to yellow is at the OKLab middle half
/// way, not the sRGB or linear one; a nine-slice keeps its 24 px corners
/// (orange) and puts the edge (teal) between them; `contain` with the
/// focal point at the top leaves the bottom of its card empty.
#[test]
fn ui_paints_mix_in_oklab_and_place_images() {
    let r = render("ui-paints");
    let (x, y) = cell(0, 0);
    // The card is inset 14 px; its middle is 76 px in.
    let mid = px(&r, x + 14 + 76, y + 75);
    let lin = |c: [u8; 3]| {
        c.map(|c| {
            let c = f32::from(c) / 255.0;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        })
    };
    let (a, b) = (
        og_render::oklab(lin([33, 66, 255])),
        og_render::oklab(lin([255, 230, 0])),
    );
    let m = og_render::oklab_to_linear([
        0.5 * (a[0] + b[0]),
        0.5 * (a[1] + b[1]),
        0.5 * (a[2] + b[2]),
    ]);
    let want = srgb8([m[0], m[1], m[2], 1.0]);
    let blue = lin([33, 66, 255]);
    let yellow = lin([255, 230, 0]);
    let linear_mid = srgb8([
        0.5 * (blue[0] + yellow[0]),
        0.5 * (blue[1] + yellow[1]),
        0.5 * (blue[2] + yellow[2]),
        1.0,
    ]);
    println!("middle {mid:?}: OKLab {want:?}, linear {linear_mid:?}");
    assert!(
        near(mid, want, 4.0),
        "{mid:?} is not the OKLab middle {want:?}"
    );
    assert!(!near(mid, linear_mid, 6.0), "mixed in linear light");
    // The nine-slice (cell 4, row 1): the card is 152×122 from (14, 14).
    let (x, y) = cell(4, 1);
    let (cx, cy) = (x + 14, y + 14);
    let orange = srgb8([
        lin([245, 124, 0])[0],
        lin([245, 124, 0])[1],
        lin([245, 124, 0])[2],
        1.0,
    ]);
    let teal = srgb8([
        lin([0, 150, 136])[0],
        lin([0, 150, 136])[1],
        lin([0, 150, 136])[2],
        1.0,
    ]);
    assert!(
        near(px(&r, cx + 2, cy + 20), orange, 3.0),
        "a corner: {:?}",
        px(&r, cx + 2, cy + 20)
    );
    assert!(
        near(px(&r, cx + 76, cy + 4), teal, 3.0),
        "the top edge: {:?}",
        px(&r, cx + 76, cy + 4)
    );
    // Contain at the top (cell 2, row 1): the 2:1 picture fills 152×76
    // from the top; below it the card is empty.
    let (x, y) = cell(2, 1);
    assert!(
        near(px(&r, x + 90, y + 14 + 110), srgb8(BACKING), 2.0),
        "contain left the bottom empty"
    );
}

/// Shadows darken under a card and not inside it; glass blurs what is
/// under it (the stripes' contrast drops by most of itself under the clear
/// glass, while the same stripes beside it keep theirs).
#[test]
fn ui_effects_shadow_and_blur_what_is_under_them() {
    let r = render("ui-effects");
    let (x, y) = cell(0, 0);
    // Card 0 is 136×106 from (22, 22); its key shadow falls 14 px below.
    let below = px(&r, x + 90, y + 22 + 106 + 10);
    let band = px(&r, x + 90, y + 22 + 106 + 60);
    assert!(
        below[0] < band[0] - 15.0,
        "no shadow under the card: {below:?} vs {band:?}"
    );
    let glass = spread(&r, 80..400, 450..640);
    let bare = spread(&r, 860..1240, 680..715);
    println!("luminance spread under glass {glass:.1}, bare stripes {bare:.1}");
    assert!(
        glass < 0.4 * bare,
        "the glass does not blur: {glass} vs {bare}"
    );
}

/// A clip to a circle cuts the rows at its edge (antialiased: one mixed
/// pixel), a gradient mask fades its card, and the dissolve opens holes.
#[test]
fn ui_layers_clip_mask_and_dissolve() {
    let r = render("ui-layers");
    let back = srgb8(BACKING);
    let (x, y) = cell(0, 0);
    // The circle is 140 px from (20, 5): its corner region is backing,
    // its middle the rows.
    assert!(near(px(&r, x + 24, y + 9), back, 2.0), "the clip leaks");
    assert!(
        !near(px(&r, x + 90, y + 75), back, 10.0),
        "the clip cut it all"
    );
    // The edge along the circle's 3 o'clock: inside, one mixed pixel, outside.
    let row = px(&r, x + 158, y + 75);
    let out = px(&r, x + 162, y + 75);
    assert!(
        !near(row, back, 10.0) && near(out, back, 2.0),
        "{row:?} {out:?}"
    );
    // The mask (cell 2, row 0): the card's top is brighter than its bottom.
    let (x, y) = cell(2, 0);
    let top = px(&r, x + 30, y + 20);
    let bottom = px(&r, x + 30, y + 134);
    assert!(top[2] > bottom[2] + 30.0, "no fade: {top:?} {bottom:?}");
    // The dissolve (cell 3, row 0) shows the backing through its card.
    let (x, y) = cell(3, 0);
    let holes = (y + 20..y + 130)
        .flat_map(|py| (x + 20..x + 160).map(move |qx| (qx, py)))
        .filter(|&(qx, py)| near(px(&r, qx, py), back, 3.0))
        .count();
    assert!(holes > 1000, "{holes} pixels dissolved");
}

/// The cooked emblem's even-odd ring keeps its hole.
#[test]
fn ui_vector_fills_by_its_rule() {
    let r = render("ui-vector");
    // The 640 px emblem: view box 120 at 640/120 px a unit from (600, 40).
    let k = 640.0 / 120.0;
    let at = |ux: f32, uy: f32| px(&r, (600.0 + ux * k) as u32, (40.0 + uy * k) as u32);
    let ring = at(60.0, 56.0 - 26.0);
    let hole = at(60.0 + 16.0, 56.0 + 6.0);
    println!("ring {ring:?} hole {hole:?}");
    assert!(ring[2] > ring[0] + 40.0, "the ring is not blue: {ring:?}");
    assert!(hole[0] > hole[2] + 60.0, "the hole is filled: {hole:?}");
}

/// World-space UI (`docs/ui.md` §14, `ui_scenes::world`): occluders hide a
/// panel behind the wall and keep the one against the sky; the bars draw
/// their back, ghost and fill where their values put them; the mass
/// elements' numbers and names draw at all.
#[test]
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

#[test]
fn ui_panel_matches_its_reference() {
    check("ui-panel");
}

/// A world panel (`docs/ui.md` §14, `ui_scenes::panel`): the UI layer with
/// a target draws into it, so the quad shows the panel's card, bar and
/// disc; and it composites nowhere, so its rect on the screen shows the
/// scene.
#[test]
fn ui_panel_draws_into_its_target_and_not_the_screen() {
    let r = render("ui-panel");
    // The quad's centre: the teal card (0, 0.35, 0.4) as emitted radiance
    // (about (0, 160, 170) in sRGB), the sky's reflection added.
    let card = px(&r, 640, 300);
    assert!(
        card[0] < 60.0 && card[1] > 150.0 && card[2] > 160.0,
        "card {card:?}"
    );
    // Green bar on the quad's lower third, red disc at its top left.
    let green = (380..500)
        .flat_map(|y| (400..840).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let p = px(&r, x, y);
            p[1] > 200.0 && p[0] < 150.0 && p[2] < 170.0
        })
        .count();
    assert!(green > 3000, "{green} green bar pixels on the quad");
    let red = (200..330)
        .flat_map(|y| (400..560).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let p = px(&r, x, y);
            p[0] > 200.0 && p[1] < 130.0 && p[2] < 130.0
        })
        .count();
    assert!(red > 400, "{red} red disc pixels on the quad");
    // The panel's rect on the screen (760, 380, 480 × 320): the ground and
    // sky, no teal card.
    let teal = (400..680)
        .flat_map(|y| (800..1220).map(move |x| (x, y)))
        .filter(|&(x, y)| near(px(&r, x, y), srgb8([0.0, 0.35, 0.4, 1.0]), 12.0))
        .count();
    assert!(
        teal < 50,
        "{teal} teal pixels: the panel composited on the screen"
    );
}
