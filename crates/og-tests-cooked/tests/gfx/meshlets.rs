//! The meshlet path (R1 batches 2 and 3; og_tests_cooked::meshlet_scenes): the
//! clustered tree's level picked and its clusters culled on the GPU.

use super::*;
use og_tests::compare::{Tolerance, compare};

/// `meshlet-forest` (reference `meshlet-forest.png`): 132 trees tinted by
/// the level the GPU picked, green near, then yellow, then orange.
#[test]
fn meshlet_forest_matches_its_reference() {
    check("meshlet-forest");
}

/// The forest's frame holds all three tints, the coarser levels nearer
/// the horizon: the canopies stand above the eye, so a nearer one stands
/// higher in the frame; only the sky side of the horizon is counted (the
/// trunks, whose levels barely differ, draw the coarsest everywhere, and
/// stand mostly below it). A GPU that drew every tree at level 0 shows
/// only green, one that inverted the choice puts orange highest. Lit, the
/// tints read green (77, 211, 99), yellow (215, 211, 77) and orange
/// (218, 176, 50) on Metal: told apart by hue, not by a fixed channel.
#[test]
fn meshlet_forest_tints_coarser_levels_farther() {
    let img = gfx::render("meshlet-forest").expect("render").image;
    // The horizon is at row 352 of 720; the canopies' bottoms above it.
    let sky_side = img.height * 350 / 720;
    // The mean row of the pixels `is` picks, and how many.
    let rows = |is: &dyn Fn(i32, i32, i32) -> bool| {
        let (mut sum, mut n) = (0.0f64, 0.0f64);
        for y in 0..sky_side {
            for x in 0..img.width {
                let i = ((y * img.width + x) * 4) as usize;
                let p = &img.pixels[i..i + 3];
                if is(p[0] as i32, p[1] as i32, p[2] as i32) {
                    sum += y as f64;
                    n += 1.0;
                }
            }
        }
        (sum / n.max(1.0), n)
    };
    let green = |r: i32, g: i32, b: i32| g > 120 && g > r + 60 && g > b + 60;
    let yellow = |r: i32, g: i32, b: i32| r > 150 && (r - g).abs() < 20 && b < r - 80;
    let orange = |r: i32, g: i32, b: i32| r > 150 && (25..80).contains(&(r - g)) && b < g - 60;
    let (g, ng) = rows(&green);
    let (yl, ny) = rows(&yellow);
    let (o, no) = rows(&orange);
    assert!(
        ng > 2000.0 && ny > 200.0 && no > 50.0,
        "tints: green {ng}, yellow {ny}, orange {no}"
    );
    assert!(
        g < yl && g < o,
        "rows: green {g:.0}, yellow {yl:.0}, orange {o:.0}"
    );
}

/// `meshlet-occlusion` (reference `meshlet-occlusion.png`): the forest
/// behind a wall over the left two thirds, culled against the pyramid on
/// its third frame; the visible third whole.
#[test]
fn meshlet_occlusion_matches_its_reference() {
    check("meshlet-occlusion");
}

/// Two-phase occlusion never removes a visible pixel: the third frame
/// through the meshlet path matches the regular path's. A cluster neither
/// phase draws (a slot read from the wrong half, a late phase that skips
/// what the early one did not draw) shows as a hole in the forest.
#[test]
fn meshlet_occlusion_matches_the_regular_path() {
    let render = |meshlets: bool| {
        let mut g = og_tests_cooked::scenes::build("meshlet-occlusion").expect("built");
        g.scene = og_tests_cooked::meshlet_scenes::occlusion_scene(meshlets).expect("the forest");
        og_tests::gfx::render_built(&mut g).expect("render")
    };
    let (on, off) = (render(true), render(false));
    let c = compare(
        &on.image.pixels,
        &off.image.pixels,
        on.image.width,
        on.image.height,
        &Tolerance::default(),
    )
    .expect("same size");
    c.check(&Tolerance::default())
        .unwrap_or_else(|e| panic!("meshlet occlusion against the regular path: {e}"));
}

/// `meshlet-near` (reference `meshlet-near.png`): one tree 6 m away.
#[test]
fn meshlet_near_matches_its_reference() {
    check("meshlet-near");
}

/// The tree 6 m away draws its level 0 through the meshlet path as the
/// regular path draws it: pulled vertices, transforms and clusters that
/// came apart (a wrong attribute, a cluster's range off by one, a culled
/// cluster that faces the eye) fail the comparison.
#[test]
fn meshlet_near_matches_the_regular_path() {
    let render = |meshlets: bool| {
        let mut g = og_tests_cooked::scenes::build("meshlet-near").expect("built");
        g.scene = og_tests_cooked::meshlet_scenes::near_scene(meshlets).expect("the tree");
        og_tests::gfx::render_built(&mut g).expect("render")
    };
    let (on, off) = (render(true), render(false));
    let c = compare(
        &on.image.pixels,
        &off.image.pixels,
        on.image.width,
        on.image.height,
        &Tolerance::default(),
    )
    .expect("same size");
    c.check(&Tolerance::default())
        .unwrap_or_else(|e| panic!("meshlet path against the regular path: {e}"));
}

/// `meshlet-shadows` (reference `meshlet-shadows.png`): five trees' long
/// sun shadows and the spot's, cast through the meshlet path.
#[test]
fn meshlet_shadows_match_their_reference() {
    check("meshlet-shadows");
}

/// The clustered casters' shadows (every cascade and the spot's face) as
/// the regular path casts them: a shadow view that culled a cluster it
/// should keep shows as a hole in a shadow, a caster drawn twice or with
/// the wrong pipeline as acne or a missing shadow.
#[test]
fn meshlet_shadows_match_the_regular_path() {
    let render = |meshlets: bool| {
        let mut g = og_tests_cooked::scenes::build("meshlet-shadows").expect("built");
        g.scene = og_tests_cooked::meshlet_scenes::shadow_scene(meshlets).expect("the trees");
        og_tests::gfx::render_built(&mut g).expect("render")
    };
    let (on, off) = (render(true), render(false));
    let c = compare(
        &on.image.pixels,
        &off.image.pixels,
        on.image.width,
        on.image.height,
        &Tolerance::default(),
    )
    .expect("same size");
    c.check(&Tolerance::default())
        .unwrap_or_else(|e| panic!("meshlet shadows against the regular path: {e}"));
}

/// `meshlet-views` (reference `meshlet-views.png`): the forest, a view
/// from above on the right half and a chrome ball in a reflection probe,
/// each tinted by the level its own view picked.
#[test]
fn meshlet_views_match_their_reference() {
    check("meshlet-views");
}

/// The view from above and the probe's faces draw their clusters as the
/// regular path draws the same instances (every level 0, bias 0): a view
/// row with the camera's planes or eye, or a pass that drew a batch twice
/// or not at all, fails the comparison.
#[test]
fn meshlet_views_match_the_regular_path() {
    let render = |meshlets: bool| {
        let mut g = og_tests_cooked::scenes::build("meshlet-views").expect("built");
        g.scene = og_tests_cooked::meshlet_scenes::views_scene(meshlets, 0.0).expect("the forest");
        og_tests::gfx::render_built(&mut g).expect("render")
    };
    let (on, off) = (render(true), render(false));
    let c = compare(
        &on.image.pixels,
        &off.image.pixels,
        on.image.width,
        on.image.height,
        &Tolerance::default(),
    )
    .expect("same size");
    c.check(&Tolerance::default())
        .unwrap_or_else(|e| panic!("meshlet views against the regular path: {e}"));
}
