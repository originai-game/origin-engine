//! Renderer behaviour read from a frame that cooks (og-tests' tests/render.rs
//! has the others): the capture's PNG of the cooked .vox scene.

use og_render::{RenderTarget, Renderer};
use og_tests_cooked::scenes;

/// The PNG the probe's `capture` and `screenshot` answer (`og_render::png`)
/// decodes, through an independent decoder that checks every chunk's CRC,
/// to the captured pixels exactly: the cooked .vox scene at 1280×720, and
/// at 333×7, whose rows are padded on readback and whose filtered rows are
/// not a whole number of the encoder's 8-byte words.
#[test]
fn a_capture_png_decodes_to_the_captured_pixels() {
    let g = scenes::build("vox-keyframes").expect("scene");
    let mut renderer = Renderer::new_blocking(&g.desc).expect("renderer");
    for size in [(1280, 720), (333, 7)] {
        renderer.resize(size.0, size.1);
        renderer
            .render(&g.scene, &RenderTarget::Offscreen)
            .expect("render");
        let image = renderer.read_offscreen().expect("capture");
        assert_eq!((image.width, image.height), size);
        if size.0 == 1280 {
            // Not a blank frame: the round trip carries the scene.
            let colours: std::collections::BTreeSet<&[u8; 4]> =
                image.pixels.as_chunks::<4>().0.iter().collect();
            assert!(colours.len() > 100, "{} colours", colours.len());
        }
        let png = og_render::png(&image);
        let decoded = image::load_from_memory_with_format(&png, image::ImageFormat::Png)
            .expect("a valid PNG")
            .to_rgba8();
        assert_eq!(decoded.dimensions(), size);
        assert!(
            decoded.as_raw() == &image.pixels,
            "{size:?}: the decoded PNG differs from the capture"
        );
    }
}
