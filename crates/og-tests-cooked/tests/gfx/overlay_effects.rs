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
