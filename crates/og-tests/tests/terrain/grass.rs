//! R6's grass: a default og.Grass grows a meadow over the island's grass
//! layer, lit from behind it lets the sun through, it reaches the horizon
//! as the ground's colour, the wind bends it, and the same tick draws the
//! same blades.
//!
//! Ways this could fail: generation keeps nothing (a wrong density share,
//! an empty draw), the blades draw but black (unlit, a missing group),
//! translucency does nothing, the far tint never reaches the ground, or
//! the blades' order or bend depends on more than the tick (an atomic
//! append, a wall clock).

use super::*;
use og_tests::grass_scenes as gs;

/// Pixels of the frame's lower half where `a` and `b` differ by more than 8.
fn changed_below(a: &RgbaImage, b: &RgbaImage) -> f32 {
    let (mut n, mut all) = (0, 0);
    for y in a.height / 2..a.height {
        for x in 0..a.width {
            let (p, q) = (px(a, x, y), px(b, x, y));
            all += 1;
            if (0..3).any(|k| (p[k] - q[k]).abs() > 8.0) {
                n += 1;
            }
        }
    }
    n as f32 / all as f32
}

/// The mean colour of rows `y0..y1`.
fn mean(img: &RgbaImage, y0: u32, y1: u32) -> [f32; 3] {
    let mut s = [0.0f32; 3];
    let mut n = 0.0;
    for y in y0..y1 {
        for x in 0..img.width {
            let c = px(img, x, y);
            (0..3).for_each(|k| s[k] += c[k]);
            n += 1.0;
        }
    }
    s.map(|v| v / n)
}

fn luma(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

#[test]
fn a_default_field_grows_a_meadow() {
    check("grass-near");
    let with = gs::grass_scene("near");
    let mut without = gs::grass_scene("near");
    without.grass.clear();
    let (a, _) = render(&with);
    let (b, _) = render(&without);
    keep("grass-near", &a);
    keep("grass-near-bare", &b);
    let changed = changed_below(&a, &b);
    assert!(changed > 0.4, "the blades cover {changed} of the ground");
    // Lit and green, not black.
    let c = mean(&a, a.height * 3 / 4, a.height);
    assert!(c[1] > c[0] && c[1] > c[2] && luma(c) > 20.0, "{c:?}");
}

#[test]
fn the_middle_distance_stays_covered() {
    check("grass-mid");
}

/// Past its blades the ground takes the field's colour: the far meadow
/// with a field is greener than without one.
#[test]
fn the_far_field_reads_as_grass() {
    check("grass-far");
    let with = gs::grass_scene("far");
    let mut without = gs::grass_scene("far");
    without.grass.clear();
    let (a, _) = render(&with);
    let (b, _) = render(&without);
    keep("grass-far", &a);
    keep("grass-far-bare", &b);
    let green = |c: [f32; 3]| c[1] / (c[0] + c[2]).max(1.0);
    // The band above the frame's middle: the meadow beyond 100 m.
    let (y0, y1) = (a.height * 2 / 5, a.height / 2);
    let (ga, gb) = (green(mean(&a, y0, y1)), green(mean(&b, y0, y1)));
    assert!(ga > gb, "far meadow: {ga} with the field, {gb} without");
}

#[test]
fn blades_lit_from_behind_let_the_sun_through() {
    check("grass-backlit");
    let lit = gs::grass_scene("backlit");
    let mut opaque = gs::grass_scene("backlit");
    opaque.grass[0].grass.translucency = 0.0;
    let (a, _) = render(&lit);
    let (b, _) = render(&opaque);
    keep("grass-backlit", &a);
    let (la, lb) = (
        luma(mean(&a, a.height / 2, a.height)),
        luma(mean(&b, b.height / 2, b.height)),
    );
    assert!(la > lb * 1.1, "translucent {la}, opaque {lb}");
}

#[test]
fn the_wind_bends_the_blades_and_the_same_tick_draws_the_same() {
    check("grass-wind");
    let (first, _) = render(&gs::grass_wind_scene(90));
    let (again, _) = render(&gs::grass_wind_scene(90));
    assert_eq!(first.pixels, again.pixels, "tick 90 drawn twice");
    let (later, _) = render(&gs::grass_wind_scene(150));
    let (calm, _) = render(&gs::grass_scene("near"));
    keep("grass-wind", &first);
    keep("grass-wind-later", &later);
    assert!(changed_below(&first, &later) > 0.05, "a second later");
    assert!(changed_below(&first, &calm) > 0.1, "against no wind");
}
