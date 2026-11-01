use super::*;

/// `vox-keyframes` (owed row 63): a `.vox` whose block sits under a
/// transform keyed at frames 0 and 2, cooked and posed at each key side by
/// side, matches its reference, and the frame shows the block at both
/// keys: left of its copy's blue post on the left copy (frame 0), right of
/// it and higher on the right copy (frame 2). On the importer that read
/// only a transform's first key, the file cooked static (no clip: the
/// scene is not built) and both copies held the block on the left.
#[test]
fn vox_keyframes_show_the_keyed_block_at_both_keys() {
    let s = check("vox-keyframes").stats;
    assert_eq!(
        (s.skinned_instances, s.joints),
        (2, 2),
        "the block, skinned once per copy"
    );
    let img = gfx::render("vox-keyframes").expect("render").image;
    // The centroid (x, y) of the pixels `is` picks in columns [x0, x1).
    let centroid = |x0: u32, x1: u32, is: &dyn Fn(&[u8]) -> bool| {
        let (mut sx, mut sy, mut n) = (0.0f64, 0.0f64, 0.0f64);
        for y in 0..img.height {
            for x in x0..x1 {
                let i = ((y * img.width + x) * 4) as usize;
                if is(&img.pixels[i..i + 3]) {
                    sx += x as f64;
                    sy += y as f64;
                    n += 1.0;
                }
            }
        }
        assert!(n > 500.0, "pixels to measure in {x0}..{x1}: {n}");
        (sx / n, sy / n)
    };
    let red = |p: &[u8]| p[0] > 120 && p[1] < 80 && p[2] < 80;
    let blue = |p: &[u8]| p[2] > 120 && p[0] < 80 && p[1] < 120;
    let half = img.width / 2;
    let (left_block, left_post) = (centroid(0, half, &red), centroid(0, half, &blue));
    let (right_block, right_post) = (
        centroid(half, img.width, &red),
        centroid(half, img.width, &blue),
    );
    assert!(
        left_block.0 < left_post.0 - 60.0,
        "frame 0: the block left of the post ({left_block:?}, post {left_post:?})"
    );
    assert!(
        right_block.0 > right_post.0 + 60.0,
        "frame 2: the block right of the post ({right_block:?}, post {right_post:?})"
    );
    assert!(
        right_block.1 < left_block.1 - 40.0,
        "frame 2: the block lifted ({right_block:?} against {left_block:?})"
    );
}

/// `lod-levels` (depth S6; reference `lod-levels.png`): the cooked tree at
/// its three distances, each at the level its projected error picks and
/// tinted as debugView lod tints it (green, yellow, orange).
#[test]
fn lod_levels_matches_its_reference() {
    check("lod-levels");
}

/// The tree cooks into the default `lods 1 0.5 0.25`, the three distances
/// pick levels 0, 1 and 2, and the three trees draw well under the
/// triangles of three full ones: a level ignored draws the full mesh; a
/// selection inverted puts the full mesh far away.
#[test]
fn lod_levels_pick_coarser_meshes_with_distance() {
    let (_, levels, (drawn, full)) =
        og_tests_cooked::depth_render_scenes::lod_scene().expect("the tree cooks");
    assert_eq!(levels, [0, 1, 2]);
    assert!(
        (drawn as f64) < 0.65 * full as f64,
        "{drawn} triangles drawn of {full} at level 0"
    );
}
