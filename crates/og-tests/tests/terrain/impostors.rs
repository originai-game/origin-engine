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

/// Ways this could fail (G5): the material atlas never read (the card
/// then as dark against the sun as a solid one), its translucency taken
/// from the wrong channel (roughness, 0.85, would light even the opaque
/// card), the backlight added on the sunlit side instead of the far one,
/// or the card's roughness from the atlas differing from the material's
/// (the atlas at translucency 0 then reading unlike no atlas).
#[test]
#[ignore = "requires GPU"]
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
#[ignore = "requires GPU"]
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
