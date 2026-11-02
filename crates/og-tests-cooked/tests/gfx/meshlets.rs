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

/// The forest's frame holds all three tints, the full mesh's green below
/// the coarser levels' (nearer trees stand lower in the frame from an eye
/// above the ground looking level): a GPU that drew every tree at level 0
/// shows only green, one that inverted the choice puts orange nearest.
#[test]
fn meshlet_forest_tints_coarser_levels_farther() {
    let img = gfx::render("meshlet-forest").expect("render").image;
    // The mean row of the pixels `is` picks, and how many.
    let rows = |is: &dyn Fn(&[u8]) -> bool| {
        let (mut sum, mut n) = (0.0f64, 0.0f64);
        for y in 0..img.height {
            for x in 0..img.width {
                let i = ((y * img.width + x) * 4) as usize;
                if is(&img.pixels[i..i + 3]) {
                    sum += y as f64;
                    n += 1.0;
                }
            }
        }
        (sum / n.max(1.0), n)
    };
    let green = |p: &[u8]| p[1] > 120 && p[0] < 100 && p[2] < 100;
    let yellow = |p: &[u8]| p[0] > 150 && p[1] > 130 && p[2] < 80;
    let orange = |p: &[u8]| p[0] > 150 && p[1] > 50 && p[1] < 120 && p[2] < 60;
    let (g, ng) = rows(&green);
    let (yl, ny) = rows(&yellow);
    let (o, no) = rows(&orange);
    assert!(
        ng > 2000.0 && ny > 200.0 && no > 50.0,
        "tints: green {ng}, yellow {ny}, orange {no}"
    );
    assert!(
        g > yl && yl > o,
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
