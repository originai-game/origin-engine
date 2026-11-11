//! How a UI image paint samples and places its texture, read back as
//! pixels (no reference frame).
//!
//! What would break:
//! - `nearest` (CSS `image-rendering: pixelated`) still blending texels
//!   with the linear sampler, or mipmapping a scaled-up image
//! - `ScaleDown` scaling a small image up as `Contain` does
//! - `offset` (CSS `object-position` lengths) not moving the image, or
//!   moving it on top of the focal point's fraction the wrong way
//! - a zero `size` not reading the region's own pixels from the texture
use glam::{Vec2, Vec3};
use og_render::{
    Background, ColorSpace, Mips, Overlay, RenderDesc, RenderScene, RenderTarget, Renderer,
    TextureData, Tonemap, UiFit, UiImage, UiPaint, UiShape,
};

const W: u32 = 128;

/// One frame of `images` (each with its box), at one device px per CSS px.
fn render(images: impl FnOnce(&mut RenderScene) -> Vec<([f32; 4], UiImage)>) -> Vec<u8> {
    let mut scene = RenderScene::new();
    let mut o = Overlay::new(W as f32, W as f32, 1.0);
    for (r, im) in images(&mut scene) {
        o.ui.shapes.push(UiShape {
            paint: UiPaint::Image(im),
            ..UiShape::rect(r, 0.0, [1.0; 4])
        });
    }
    scene.overlay = Some(o);
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width: W,
        height: W,
        ..Default::default()
    })
    .expect("renderer");
    renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    renderer.read_offscreen().expect("capture").pixels
}

fn at(px: &[u8], x: u32, y: u32) -> [u8; 4] {
    px[((y * W + x) * 4) as usize..][..4].try_into().unwrap()
}

/// A `w`×`h` texture: white in its left half, black in its right.
fn halves(scene: &mut RenderScene, w: u32, h: u32) -> og_render::TextureHandle {
    let rgba = (0..w * h)
        .flat_map(|i| {
            if i % w < w / 2 {
                [255; 4]
            } else {
                [0, 0, 0, 255]
            }
        })
        .collect();
    scene.add_texture(TextureData {
        width: w,
        height: h,
        rgba,
        color_space: ColorSpace::Srgb,
        mips: Mips::Generate,
    })
}

fn image(texture: og_render::TextureHandle, fit: UiFit) -> UiImage {
    UiImage {
        texture,
        uv: [0.0, 0.0, 1.0, 1.0],
        size: Vec2::ZERO,
        density: 1.0,
        tint: [1.0; 4],
        fit,
        focal: Vec2::splat(0.5),
        offset: Vec2::ZERO,
        luminance: false,
        nearest: false,
    }
}

#[test]
#[ignore = "requires GPU"]
fn nearest_keeps_texels_sharp_and_scale_down_and_offset_place_the_image() {
    // A 2x1 texture stretched over 128 px: linear blends across the middle,
    // nearest keeps a hard edge one px either side of it.
    let edge = |nearest| {
        let px = render(|s| {
            let t = halves(s, 2, 1);
            vec![(
                [0.0, 0.0, 128.0, 128.0],
                UiImage {
                    nearest,
                    ..image(t, UiFit::Fill)
                },
            )]
        });
        (at(&px, 60, 64)[0], at(&px, 67, 64)[0])
    };
    let (l, r) = edge(true);
    assert_eq!((l, r), (255, 0), "nearest");
    let (l, r) = edge(false);
    assert!(l < 250 && r > 5, "linear blends: {l} {r}");

    // An 8x8 image under scale-down in a 64 px box stays 8 px, centred
    // (28..36), then moved 10 px right and 4 px up by an offset; contain
    // would scale it to 64.
    let px = render(|s| {
        let t = halves(s, 8, 8);
        vec![(
            [0.0, 0.0, 64.0, 64.0],
            UiImage {
                offset: Vec2::new(10.0, -4.0),
                ..image(t, UiFit::ScaleDown)
            },
        )]
    });
    // Its white half spans x 38..42, y 24..32.
    assert_eq!(at(&px, 39, 28)[0], 255, "inside the moved image");
    assert!(at(&px, 30, 28)[0] < 5, "where it was centred");
    assert!(at(&px, 39, 34)[0] < 5, "below it");
}
