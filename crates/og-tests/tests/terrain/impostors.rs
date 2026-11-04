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

/// Ways this could fail (G5): the material atlas never read (the card
/// then as dark against the sun as a solid one), its translucency taken
/// from the wrong channel (roughness, 0.85, would light even the opaque
/// card), the backlight added on the sunlit side instead of the far one,
/// or the card's roughness from the atlas differing from the material's
/// (the atlas at translucency 0 then reading unlike no atlas).
#[test]
fn a_leaf_card_lets_the_sun_through_from_behind() {
    let (leaf, _) = render(&is::impostor_backlit_scene(Some(0.8)));
    keep("impostor-backlit", &leaf);
    let (solid, _) = render(&is::impostor_backlit_scene(Some(0.0)));
    let (plain, _) = render(&is::impostor_backlit_scene(None));
    let scene = is::impostor_backlit_scene(None);
    // The card's face towards the camera, away from the sun: points over
    // both halves, inside its silhouette and off the seam between them
    // (x = 0, where a pixel mixes the two colours).
    let points = [
        (0.2, 0.0),
        (0.5, 0.2),
        (-0.5, 0.2),
        (0.4, -0.4),
        (-0.4, -0.4),
        (-0.2, 0.6),
    ];
    let mean = |img: &RgbaImage| {
        points
            .iter()
            .map(|&(dx, dy)| {
                let c = color_at(
                    img,
                    project(&scene.camera, is::BACKLIT_AT + Vec3::new(dx, dy, 0.8)),
                );
                assert!(ball_pixel(c), "the card covers ({dx}, {dy}): {c:?}");
                luma(c)
            })
            .sum::<f32>()
            / points.len() as f32
    };
    let (l, s, p) = (mean(&leaf), mean(&solid), mean(&plain));
    assert!(l > s * 1.1, "backlit leaves {l} against a solid card {s}");
    assert!(
        (s - p).abs() < 0.05 * p.max(s),
        "atlas at translucency 0 {s} against none {p}"
    );
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
fn copies_inside_the_impostor_band_draw_without_holes() {
    // The terrain spans +/-24 m, seen from z=26: every copy lies inside
    // 72 m (the start of an 80 m switch's crossfade). Both renders must
    // therefore keep every mesh pixel, in the prepass and lit pass alike.
    let (with, _) = render(&is::impostor_scatter_scene(80.0));
    let (without, _) = render(&is::impostor_scatter_scene(0.0));
    let changed = with
        .pixels
        .iter()
        .zip(&without.pixels)
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        changed, 0,
        "enabling far impostors must not change near copies"
    );
}

/// Draws `scene` `frames` times at one tick with TAA (a 640 × 360 target),
/// returning the last frame, or `None` where the adapter has no TAA.
fn taa_frames(scene: &RenderScene, frames: usize) -> Option<RgbaImage> {
    let desc = og_render::RenderDesc {
        width: 640,
        height: 360,
        antialias: og_render::Antialias::Taa,
        ..scenes::base_desc()
    };
    let mut r = Renderer::new_blocking(&desc).expect("renderer");
    if r.info().antialias != og_render::Antialias::Taa {
        return None;
    }
    for _ in 0..frames {
        r.render(scene, &RenderTarget::Offscreen).expect("render");
    }
    assert_eq!(r.history().1, frames as u64, "TAA held every frame");
    Some(r.read_offscreen().expect("capture"))
}

/// The mean absolute 3 × 3 Laplacian of the luma over rows `y0..y1`: the
/// frame's 1 px grain.
fn grain(img: &RgbaImage, y0: u32, y1: u32) -> f32 {
    let (mut sum, mut n) = (0.0f32, 0u32);
    for y in y0.max(1)..y1.min(img.height - 1) {
        for x in 1..img.width - 1 {
            let c = luma(px(img, x, y));
            let around = luma(px(img, x - 1, y))
                + luma(px(img, x + 1, y))
                + luma(px(img, x, y - 1))
                + luma(px(img, x, y + 1));
            sum += (4.0 * c - around).abs();
            n += 1;
        }
    }
    sum / n.max(1) as f32
}

/// The scene of the crossfade tests: copies 36 to 40 m out (the rows above
/// the frame's middle) between their mesh and their card.
fn crossfade_scene() -> RenderScene {
    let mut scene = is::impostor_scatter_scene(40.0);
    scene.light.cast_shadows = false;
    scene
}

/// The mesh-to-impostor crossfade thresholds its pixels by blue noise
/// turned every render frame, so at one tick two frames drop different
/// pixels (TAA integrates them into the blend). Broken before by the
/// static interleaved-gradient threshold, which dropped the same pixels
/// every frame: grain TAA could not resolve.
#[test]
fn the_crossfade_turns_its_noise_every_render_frame() {
    let mut scene = crossfade_scene();
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
        "the crossfade's pixels change between render frames at one tick"
    );
}

/// Under TAA the crossfade resolves to a blend: after 16 frames at one
/// tick the far half of the frame (the copies in the crossfade) holds no
/// more 1 px grain than the same field drawn all as meshes, within 15 %.
/// The static threshold left every crossfading copy dotted (the site's
/// midground grain).
#[test]
fn taa_resolves_the_crossfade_without_grain() {
    let scene = crossfade_scene();
    let Some(crossfade) = taa_frames(&scene, 16) else {
        println!("skipped: TAA is not available on this adapter");
        return;
    };
    let meshes = taa_frames(&is::impostor_scatter_scene(0.0), 16).expect("TAA");
    keep("impostor-crossfade-taa", &crossfade);
    keep("impostor-crossfade-taa-meshes", &meshes);
    let (g, m) = (grain(&crossfade, 0, 180), grain(&meshes, 0, 180));
    println!("far-half grain: crossfade {g:.3}, meshes {m:.3}");
    assert!(
        g < 1.15 * m + 0.05,
        "crossfade grain {g:.3} vs meshes {m:.3}"
    );
}

/// G2: impostor cards sway with the wind field as their meshes do: the
/// field drawn all as cards changes when the wind blows. Before G2 a card
/// stood still in any wind (its mesh swayed, so the crossfade jumped).
#[test]
fn impostor_cards_sway_in_the_wind() {
    let mut scene = is::impostor_scatter_scene(1.0);
    scene.scatters[0].scatter.sway = 1.0;
    scene.light.cast_shadows = false;
    let still = render(&scene).0;
    scene.wind = Some(og_render::Wind {
        strength: 0.4,
        direction: Vec3::X,
        gust: 0.0,
        turbulence: 0.0,
        ..og_render::Wind::default()
    });
    scene.tick = 60;
    let (blown, stats) = render(&scene);
    keep("impostor-cards-wind", &blown);
    assert!(stats.scatter_draws > 0);
    let changed = still
        .pixels
        .chunks(4)
        .zip(blown.pixels.chunks(4))
        .filter(|(a, b)| a.iter().zip(*b).any(|(x, y)| x.abs_diff(*y) > 24))
        .count();
    println!("cards in the wind: {changed} pixels moved");
    assert!(
        changed > 500,
        "the cards stood still in the wind ({changed} pixels)"
    );
}
