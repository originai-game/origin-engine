//! A look that switches the stylisation (passes added or removed, the pixel
//! grid, the material style: `Stylise::chain_bits`) resets the TAA history
//! as a camera cut does, so the first frame of the new look holds no ghost
//! of the old one; a change of a setting alone keeps it.
//!
//! Ways this could fail: the renderer blends the new look's first frame
//! with the old look's history (`FrameStats::taa_history` true there, and
//! the frame differs from one drawn after `reset_history`); it resets on a
//! setting that does not switch a pass (a posterize level), losing the
//! antialiasing of every frame of an animated look. Broken once by leaving
//! the reset out: the look's first frame reports a history blend and
//! differs from the cut's frame.

use og_render::{
    Antialias, DitherPattern, Quantize, RenderDesc, RenderScene, RenderTarget, Renderer, RgbaImage,
};
use og_tests::scenes;

fn desc() -> RenderDesc {
    RenderDesc {
        width: 320,
        height: 180,
        antialias: Antialias::Taa,
        ..scenes::base_desc()
    }
}

fn quantize(posterize: u32) -> Quantize {
    Quantize {
        posterize,
        palette: None,
        dither: DitherPattern::Bayer4,
        dither_strength: 1.0,
        dither_scale: 1,
    }
}

/// Draws `scene`, returning the frame and whether TAA blended history.
fn draw(renderer: &mut Renderer, scene: &RenderScene) -> (RgbaImage, bool) {
    let stats = renderer
        .render(scene, &RenderTarget::Offscreen)
        .expect("frame");
    (
        renderer.read_offscreen().expect("capture"),
        stats.taa_history,
    )
}

#[test]
fn a_look_that_switches_the_stylisation_resets_the_taa_history() {
    let mut scene = scenes::thin_geometry();
    let mut look = Renderer::new_blocking(&desc()).expect("renderer");
    if look.info().antialias != Antialias::Taa {
        println!("skipped: TAA is not available on this adapter");
        return;
    }
    let mut cut = Renderer::new_blocking(&desc()).expect("renderer");
    // Three frames of the plain look: history from the second on.
    for f in 0..3 {
        let (a, blend_a) = draw(&mut look, &scene);
        let (b, blend_b) = draw(&mut cut, &scene);
        assert_eq!(blend_a, f > 0, "frame {f}");
        assert_eq!(blend_b, f > 0, "frame {f}");
        assert!(a.pixels == b.pixels, "frame {f}: the two renderers agree");
    }
    // The new look: posterized and toon-shaded. `cut` is told it is a cut.
    scene.stylise.quantize = Some(quantize(6));
    scene.stylise.material.toon = true;
    cut.reset_history();
    let (a, blend_a) = draw(&mut look, &scene);
    let (b, blend_b) = draw(&mut cut, &scene);
    assert!(!blend_a, "the new look's first frame blends no history");
    assert!(!blend_b);
    assert!(
        a.pixels == b.pixels,
        "the new look's first frame is the cut's, byte for byte"
    );
    // The next frame blends again, and a setting alone keeps the history.
    assert!(draw(&mut look, &scene).1);
    scene.stylise.quantize = Some(quantize(8));
    assert!(draw(&mut look, &scene).1, "a posterize level is no cut");
    // Back to the plain look: a cut again.
    scene.stylise = Default::default();
    assert!(!draw(&mut look, &scene).1, "the stylisation switched off");
}
