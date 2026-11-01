//! `png_best`, the encoder of every reference frame.

use og_render::{RgbaImage, png_best};

fn decode(bytes: &[u8]) -> (image::ColorType, image::RgbaImage) {
    let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Png).expect("a PNG");
    (img.color(), img.to_rgba8())
}

/// A frame with gradients, noise and flat areas, every pixel opaque or not.
fn frame(width: u32, height: u32, opaque: bool) -> RgbaImage {
    let mut pixels = Vec::new();
    let mut seed = 0x9e37_79b9u32;
    for y in 0..height {
        for x in 0..width {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            let noise = if x > width / 2 { (seed & 7) as u8 } else { 0 };
            let a = if opaque { 255 } else { (x * 255 / width) as u8 };
            pixels.extend_from_slice(&[
                (x * 3) as u8 ^ noise,
                (y * 5) as u8,
                if y > height / 2 { 40 } else { noise },
                a,
            ]);
        }
    }
    RgbaImage {
        width,
        height,
        pixels,
    }
}

#[test]
fn png_best_decodes_to_the_same_pixels_rgb_when_opaque() {
    for (opaque, color) in [
        (true, image::ColorType::Rgb8),
        (false, image::ColorType::Rgba8),
    ] {
        let img = frame(97, 41, opaque);
        let (got, back) = decode(&png_best(&img));
        assert_eq!(got, color, "opaque {opaque}");
        assert_eq!((back.width(), back.height()), (97, 41));
        assert!(
            back.as_raw() == &img.pixels,
            "opaque {opaque}: the pixels decode exactly"
        );
        let fast = og_render::png(&img);
        assert!(
            png_best(&img).len() < fast.len(),
            "smaller than the capture's fast PNG"
        );
    }
}
