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
    scene.ui.overlay = Some(o);
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
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

/// Retained streams must render the current values after property edits,
/// shrinking/regrowing and allocation growth, not bytes from an older frame.
#[test]
#[ignore = "requires GPU"]
fn retained_shape_edits_match_fresh_rendering() {
    let desc = RenderDesc {
        width: W,
        height: W,
        ..Default::default()
    };
    let mut retained = Renderer::new_blocking(&desc).expect("retained renderer");
    let mut scene = RenderScene::new();
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    for (step, count) in [3, 3, 1, 3, 90, 3].into_iter().enumerate() {
        let mut overlay = Overlay::new(W as f32, W as f32, 1.0);
        let shapes: Vec<_> = (0..count)
            .map(|i| {
                let mut shape = UiShape::rect(
                    [
                        8.0 + (i % 5) as f32 * 23.0,
                        15.0 + (i / 5) as f32 * 5.0,
                        13.0,
                        19.0,
                    ],
                    2.0,
                    [0.8, 0.3, 0.1, 1.0],
                );
                if step % 2 == 1 && i == 1 {
                    shape.center.y += 29.0;
                    shape.transform = [0.7, 0.0, 0.0, 1.2];
                    shape.opacity = 0.35;
                }
                shape
            })
            .collect();
        overlay.ui.retained_shapes = Some(std::sync::Arc::new(shapes.clone()));
        overlay.key = Some(step as u64);
        scene.ui.overlay = Some(overlay);
        retained
            .render(&scene, &RenderTarget::Offscreen)
            .expect("retained render");
        let actual = retained.read_offscreen().expect("retained pixels").pixels;
        // Repeat the identical key: this also exercises the existing
        // no-upload route immediately after a partial upload.
        retained
            .render(&scene, &RenderTarget::Offscreen)
            .expect("kept render");
        assert_eq!(
            retained.read_offscreen().expect("kept pixels").pixels,
            actual
        );
        // An ordinary stream on a new allocation is the independent
        // control, with no retained snapshot or shared-shape storage.
        let mut control = Overlay::new(W as f32, W as f32, 1.0);
        control.ui.shapes = shapes;
        scene.ui.overlay = Some(control);
        let mut fresh = Renderer::new_blocking(&desc).expect("fresh renderer");
        fresh
            .render(&scene, &RenderTarget::Offscreen)
            .expect("control render");
        let expected = fresh.read_offscreen().expect("control pixels").pixels;
        assert_eq!(actual, expected, "step {step}, count {count}");
    }
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
        orient: true,
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

/// Changing a layer's used extent must not stretch its pixel grid or let
/// the unused part of a reused target enter a composite (including nested
/// layers, custom material samples past the edge, and filter inputs).
#[test]
#[ignore = "requires GPU"]
fn resized_layers_keep_their_pixel_grid_and_clamp_to_the_used_edges() {
    use og_render::{
        UiBuiltin, UiFilter, UiFilterInput, UiFilterOp, UiFilterPrimitive, UiLayer, UiMaterial,
        UiMaterialShader,
    };
    let mut scene = RenderScene::new();
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    let edge = scene.add_ui_material(UiMaterialShader {
        name: "used-edge".into(),
        code: "fn material(m: MaterialInput, p: Params) -> vec4<f32> {
            return layer(m.uv + vec2<f32>(4.0));
        }"
        .into(),
        params: Vec::new(),
    });
    let mut r = Renderer::new_blocking(&RenderDesc {
        width: W,
        height: W,
        ..Default::default()
    })
    .expect("renderer");
    let mut flat = Renderer::new_blocking(&RenderDesc {
        width: W,
        height: W,
        ..Default::default()
    })
    .expect("flat renderer");
    // Both sides of 64, a new bucket, then shrinking into a reused one.
    for width in [63, 64, 65, 97, 33, 64, 63] {
        for filtered in [false, true] {
            let rect = [7.0, 11.0, width as f32, 73.0];
            let inner = [12.0, 19.0, 17.0, 31.0];
            let mut o = Overlay::new(W as f32, W as f32, 1.0);
            o.ui.shapes
                .push(UiShape::rect(rect, 0.0, [1.0, 0.0, 0.0, 1.0]));
            let mut child = UiShape::rect(inner, 0.0, [0.0, 1.0, 0.0, 1.0]);
            child.layer = 1;
            o.ui.shapes.push(child);
            scene.ui.overlay = Some(o.clone());
            flat.render(&scene, &RenderTarget::Offscreen)
                .expect("flat render");
            let expected = flat.read_offscreen().expect("flat pixels").pixels;
            let layer = |rect, capture, material| UiLayer {
                rect,
                capture,
                material,
                params: [[0.0; 4]; 4],
                mask: None,
                target: None,
                ground: None,
                backdrop: false,
                filter: None,
            };
            o.ui.layers
                .push(layer(rect, [0, 2], UiMaterial::Builtin(UiBuiltin::None)));
            o.ui.layers
                .push(layer(inner, [1, 1], UiMaterial::Custom(edge)));
            if filtered {
                o.ui.layers[0].filter = Some(UiFilter {
                    primitives: vec![UiFilterPrimitive {
                        op: UiFilterOp::Offset { d: [0.0, 0.0] },
                        inputs: [UiFilterInput::Source; 2],
                        subregion: [0.0, 0.0, W as f32, W as f32],
                        srgb: false,
                    }],
                    ..Default::default()
                });
                // Sample two pixels to the right with edge duplication.
                // The green child stays green through its final column;
                // using allocated dimensions here would read padding.
                o.ui.layers[1].filter = Some(UiFilter {
                    primitives: vec![UiFilterPrimitive {
                        op: UiFilterOp::Convolve {
                            order: [3, 1],
                            kernel: vec![1.0, 0.0, 0.0],
                            divisor: 1.0,
                            bias: 0.0,
                            target: [0, 0],
                            edge: 0,
                            preserve_alpha: false,
                            kernel_unit: None,
                        },
                        inputs: [UiFilterInput::Source; 2],
                        subregion: [0.0, 0.0, W as f32, W as f32],
                        srgb: false,
                    }],
                    ..Default::default()
                });
            }
            for (index, rect, level) in [(1, inner, 2), (0, rect, 3)] {
                let mut s = UiShape::rect(rect, 0.0, [1.0; 4]);
                s.paint = UiPaint::Layer(index);
                s.layer = level;
                o.ui.shapes.push(s);
            }
            scene.ui.overlay = Some(o);
            r.render(&scene, &RenderTarget::Offscreen)
                .expect("layered render");
            // Custom pipelines can finish asynchronously: settle the
            // published material before checking its pixels.
            og_render::settle_pipelines();
            r.render(&scene, &RenderTarget::Offscreen)
                .expect("settled render");
            assert!(r.ui_material_errors(&scene).is_empty());
            let actual = r.read_offscreen().expect("layered pixels").pixels;
            let changed = actual.iter().zip(&expected).filter(|(a, b)| a != b).count();
            assert_eq!(changed, 0, "width {width}, filter {filtered}");
        }
    }
}
