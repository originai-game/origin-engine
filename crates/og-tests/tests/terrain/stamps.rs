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
