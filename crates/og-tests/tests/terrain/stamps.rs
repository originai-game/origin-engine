//! Terrain stamps read from rendered frames: a mesh stamped into a
//! terrain is shaded by the terrain's own layers by its own normal (rock
//! on a cliff's face, grass on the valley before it), and within `blend`
//! metres of the ground it meets it takes the ground's look, so the seam
//! where the face meets the valley disappears.

use super::*;

/// A stamp whose transform mirrors its mesh (a negative scale on one
/// axis) draws from outside, as the unmirrored one: the cliff's block is
/// its own mirror image, so the two frames agree. Before, the stamp's
/// pipelines took the mirrored triangles' winding as the back and the
/// block showed its far faces from inside.
#[test]
#[ignore = "requires GPU"]
fn a_mirrored_stamp_draws_its_outside() {
    let plain = ts::cliff_scene(0.0);
    let mut mirrored = ts::cliff_scene(0.0);
    mirrored.world.terrain_stamps[0].transform *= Mat4::from_scale(Vec3::new(-1.0, 1.0, 1.0));
    let (a, _) = render(&plain);
    let (b, _) = render(&mirrored);
    keep("stamp-mirrored", &b);
    let moved = a
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .zip(b.pixels.as_chunks::<4>().0)
        .filter(|(p, q)| (0..3).any(|c| p[c].abs_diff(q[c]) > 8))
        .count();
    assert!(
        moved < 200,
        "{moved} pixels differ from the unmirrored cliff"
    );
}

/// A stamp's face turned down (an overhang's underside, 180° from flat)
/// takes the layer its vertical face takes. The cliff's block is lifted
/// off the valley and seen from under it: its underside is the rock of
/// the 40° to 90° rule, as its face is. Before, a slope past 90° was in
/// no rule's range, no layer had weight and the fallback drew the first
/// layer alone: grass under every overhang.
#[test]
#[ignore = "requires GPU"]
fn an_overhangs_underside_takes_the_steepest_rule() {
    const LIFT: f32 = 14.0;
    let mut scene = ts::cliff_scene(0.0);
    let stamp = &mut scene.world.terrain_stamps[0];
    stamp.transform = Mat4::from_translation(Vec3::Y * LIFT) * stamp.transform;
    let under = Vec3::new(ts::CLIFF_ROCK * 0.5, LIFT, 0.0);
    let face = Vec3::new(ts::CLIFF_ROCK, LIFT + 6.0, 0.0);
    scene.frame.camera = ts::camera_at(Vec3::new(ts::CLIFF_ROCK + 16.0, 2.0, 4.0), under);
    let (img, _) = render(&scene);
    keep("stamp-overhang", &img);
    let under = color_at(&img, project(&scene.frame.camera, under));
    let face = color_at(&img, project(&scene.frame.camera, face));
    // The grass is 0.16, 0.38, 0.08 (green over twice its red), the rock
    // 0.42, 0.4, 0.37 (grey).
    for (name, c) in [("underside", under), ("face", face)] {
        assert!(
            c[1] > 4.0 && c[1] < 1.3 * c[0],
            "the {name} is not the rock: {c:?}"
        );
    }
}
