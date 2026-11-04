//! R6's octahedral impostors (og-render's `impostor.rs`): a model's card
//! reads as its mesh does, and a scatter's far copies drawn as the card
//! keep the field looking as it does with every copy a mesh.
//!
//! Ways this could fail: the card samples the frame of the opposite side
//! or a mirrored one (red and blue swapped), blends frames without their
//! coverage (a square or a halo round the ball), lights the card's plane
//! instead of the baked normals (a flat disc), casts no shadow or a
//! square one, or draws the far copies nowhere (the vertex stages' split
//! dropping both).

use super::*;
use og_tests::impostor_scenes as is;

fn ball_pixel(c: [f32; 3]) -> bool {
    (c[0] > c[2] * 1.6 && c[0] > c[1] * 1.6) || (c[2] > c[0] * 1.6 && c[2] > c[1] * 1.2)
}

#[test]
fn an_impostor_draws_as_the_mesh_it_stands_for() {
    let r = check("impostors");
    assert!(
        r.stats.scatter_draws > 0,
        "the card was drawn: {:?}",
        r.stats
    );
    let scene = is::impostors_scene();
    let (img, _) = render(&scene);
    keep("impostors", &img);
    let at = |p: Vec3| color_at(&img, project(&scene.camera, p));
    let sun = scene.light.direction_to_light.normalize();
    let open = at(Vec3::new(0.0, 0.0, 2.5));
    let mut shadows = Vec::new();
    for (name, c) in [("mesh", is::BALL_MESH_AT), ("card", is::BALL_CARD_AT)] {
        let right = at(c + Vec3::new(0.6, 0.0, 0.0));
        let left = at(c + Vec3::new(-0.6, 0.0, 0.0));
        assert!(
            right[0] > right[2] * 1.6,
            "{name}: its right is red: {right:?}"
        );
        assert!(
            left[2] > left[0] * 1.6,
            "{name}: its left is blue: {left:?}"
        );
        // Lit by the baked normals: the side towards the sun (up and
        // right) brighter than the one away from it.
        let lit = at(c + Vec3::new(0.45, 0.6, 0.3));
        let dark = at(c + Vec3::new(0.45, -0.6, 0.3));
        assert!(
            luma(lit) > luma(dark) * 1.4,
            "{name}: shading {lit:?} vs {dark:?}"
        );
        // Nothing past its rim: the sky above it.
        let above = at(c + Vec3::new(0.0, 1.2, 0.0));
        assert!(
            above[2] > above[0] * 1.2 && !ball_pixel(above) || above[2] > 150.0,
            "{name}: above {above:?}"
        );
        // Its shadow on the floor under it, along the sun (the sky's
        // light keeps it at about three quarters of the open floor's):
        // sampled 0.6 m left of its centre, where neither ball covers it
        // (the centre itself is behind the mesh ball from this camera).
        let shadow = at(c - sun * (c.y / sun.y) + Vec3::new(-0.6, 0.0, 0.0));
        assert!(
            luma(shadow) < luma(open) * 0.85,
            "{name}: shadow {shadow:?} vs {open:?}"
        );
        shadows.push(luma(shadow));
    }
    // The card's shadow as dark as the mesh's, and where it is.
    assert!(
        (shadows[0] - shadows[1]).abs() < 0.1 * shadows[0],
        "shadows: mesh {} card {}",
        shadows[0],
        shadows[1]
    );
    // The card reads as the mesh drawn where it stands (the camera sees the
    // left ball from its other side, so that one's blue half faces the sun
    // more): the same colours within a fifth, at points over both halves.
    let (mesh, _) = render(&is::impostors_scene_with(false));
    let c = is::BALL_CARD_AT;
    for (dx, dy) in [
        (0.6, 0.0),
        (-0.6, 0.0),
        (0.45, 0.6),
        (-0.45, 0.4),
        (-0.7, -0.3),
        (-0.3, -0.6),
    ] {
        let q = project(&scene.camera, c + Vec3::new(dx, dy, 0.3));
        let (a, b) = (color_at(&mesh, q), color_at(&img, q));
        let (la, lb) = (luma(a), luma(b));
        assert!(
            (la - lb).abs() < 0.2 * la.max(lb),
            "at ({dx}, {dy}): mesh {a:?} card {b:?}"
        );
    }
}

#[test]
fn far_copies_draw_as_the_impostor() {
    let r = check("impostor-scatter");
    let (cards, with) = render(&is::impostor_scatter_scene(15.0));
    let (meshes, without) = render(&is::impostor_scatter_scene(0.0));
    keep("impostor-scatter", &cards);
    keep("impostor-scatter-meshes", &meshes);
    // The cards were drawn: their own draws past the meshes' chunks.
    assert!(
        with.scatter_draws > without.scatter_draws,
        "{with:?} vs {without:?}"
    );
    assert!(r.stats.scatter_draws == with.scatter_draws);
    // And the field looks as it does with every copy a mesh: as many ball
    // pixels in the far half of the frame (above its middle), and close
    // over the whole frame.
    let count = |img: &RgbaImage| {
        (0..img.height / 2)
            .flat_map(|y| (0..img.width).map(move |x| (x, y)))
            .filter(|&(x, y)| ball_pixel(px(img, x, y)))
            .count() as f32
    };
    let (a, b) = (count(&cards), count(&meshes));
    assert!(b > 2_000.0, "the far half shows balls: {b}");
    assert!(
        (a - b).abs() < 0.15 * b,
        "ball pixels far away: cards {a} vs meshes {b}"
    );
    let e = rmse(&cards, &meshes);
    assert!(e < 0.05, "cards vs meshes: rmse {e}");
}

#[test]
fn impostor_fade_coverage_changes_without_advancing_the_tick() {
    let mut scene = is::impostor_scatter_scene(1.0);
    scene.scatters[0].scatter.distance = 40.0;
    scene.scatters[0].scatter.fade = 0.5;
    scene.light.cast_shadows = false;
    scene.debug = og_render::DebugView::Albedo;
    scene.post = Default::default();
    scene.post.tonemap = Tonemap::None;
    scene.camera.exposure = Exposure::linear(1.0);
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    r.render(&scene, &RenderTarget::Offscreen).unwrap();
    let first = r.read_offscreen().unwrap();
    r.render(&scene, &RenderTarget::Offscreen).unwrap();
    let second = r.read_offscreen().unwrap();
    let coverage = |image: &RgbaImage| {
        image
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| ball_pixel([p[0] as f32, p[1] as f32, p[2] as f32]))
            .collect::<Vec<_>>()
    };
    assert!(
        coverage(&first) != coverage(&second),
        "impostor fade pixels must change between render frames at one tick"
    );
}
