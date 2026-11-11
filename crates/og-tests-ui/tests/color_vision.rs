//! The colour-vision filter: the frames og-render draws, against `references/<tier>/`.
//! What og-ui lays out and builds for them is checked in
//! crates/og-ui/tests/color_vision.rs.

use crate::support;
use support::*;

/// The filter on the GPU is the CPU's ([`og_render::ColorFilter::apply_srgb8`]) on every
/// pixel, the overlay's and the scene's alike: one frame rendered with no
/// filter, then with each mode, compared pixel for pixel.
#[test]
#[ignore = "requires GPU"]
fn the_frame_is_the_cpu_filter_of_the_unfiltered_frame() {
    use og_render::*;
    for (width, height) in [(1280u32, 720u32), (390, 844)] {
        let (mut scene, desc) = with_test_font(|f| color_vision_scene(f, width, height));
        let mut renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
        let mut frame = |scene: &RenderScene| {
            renderer.render(scene, &RenderTarget::Offscreen).unwrap();
            renderer.read_offscreen().unwrap().pixels
        };
        let plain = frame(&scene);
        for vision in [
            ColorVision::Protanopia,
            ColorVision::Deuteranopia,
            ColorVision::Tritanopia,
        ] {
            for filter in [ColorVisionFilter::Correct, ColorVisionFilter::Simulate] {
                let f = ColorFilter::new(vision, filter, 1.0).unwrap();
                scene.look.color_filter = Some(f);
                let got = frame(&scene);
                let mut worst = 0u8;
                let mut moved = 0usize;
                for (p, g) in plain.chunks(4).zip(got.chunks(4)) {
                    let o = f.apply_srgb8([p[0], p[1], p[2]]);
                    for k in 0..3 {
                        worst = worst.max(o[k].abs_diff(g[k]));
                    }
                    moved += usize::from(p[..3] != g[..3]);
                }
                assert!(
                    worst <= 2,
                    "{vision:?} {filter:?}: a pixel {worst}/255 off the CPU filter"
                );
                assert!(
                    moved > plain.len() / 8,
                    "{vision:?} {filter:?}: only {moved} pixels changed"
                );
                if filter == ColorVisionFilter::Correct {
                    image::save_buffer(
                        out_dir().join(format!("color-vision-{}-{width}.png", vision.name())),
                        &got,
                        width,
                        height,
                        image::ColorType::Rgba8,
                    )
                    .unwrap();
                }
            }
        }
        scene.look.color_filter = None;
        assert_eq!(frame(&scene), plain, "no filter leaves the frame as it was");
    }
}
