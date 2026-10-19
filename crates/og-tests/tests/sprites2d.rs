//! The E4 2D batch, read from rendered frames: y-sorted sprites, tile
//! layers (culling, repeat, animated tiles, drawn under their layer's
//! sprites), nearest filtering, nine-slice pieces, text outlines and trails
//! drawn in the sprite order.
//!
//! Ways this could fail: the sort key is ignored (texture grouping wins over
//! y), or it reorders sprites across layers; tiles draw over the sprites of
//! their layer, or chunks outside the view are still drawn (the count) or
//! inside it are culled (holes); a repeated layer leaves the view's edge
//! empty; animated tiles stay on their first frame; a nearest texture is
//! still blended between texels; nine-slice corners stretch with the
//! sprite; an outline is drawn without being asked (old text changes) or
//! not at all; a layered trail is still hidden under the sprites.

use glam::{Vec2, Vec3};
use og_render::{
    Background, Camera, ColorSpace, FrameStats, Mips, RenderScene, RenderTarget, Renderer,
    RgbaImage, Sprite, SpriteOrder, Text, TextOutline, TextureData, TextureFilter, TileAnimation,
    TileChunk, TileLayer, TileQuad, Tonemap, Trail, TrailStyle, nine_slice,
};
use og_tests::scenes;
use std::sync::Arc;

fn frame(scene: &RenderScene, frames: u32) -> (RgbaImage, FrameStats) {
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let mut stats = FrameStats::default();
    for _ in 0..frames.max(1) {
        stats = renderer
            .render(scene, &RenderTarget::Offscreen)
            .expect("render");
    }
    (renderer.read_offscreen().expect("capture"), stats)
}

/// A flat 2D scene: orthographic, 10 units tall, black, untonemapped.
fn flat() -> RenderScene {
    let mut scene = RenderScene::new();
    scene.camera = Camera::orthographic_2d(Vec2::ZERO, 10.0);
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.post.tonemap = Tonemap::None;
    scene
}

/// The pixel at world (x, y) of a `flat()` frame (1280x720, 72 px per unit).
fn at(img: &RgbaImage, x: f32, y: f32) -> [u8; 3] {
    let px = (img.width as f32 / 2.0 + x * 72.0) as u32;
    let py = (img.height as f32 / 2.0 - y * 72.0) as u32;
    let i = ((py * img.width + px) * 4) as usize;
    [img.pixels[i], img.pixels[i + 1], img.pixels[i + 2]]
}

fn solid(rgba: [u8; 4], w: u32, h: u32) -> TextureData {
    TextureData {
        width: w,
        height: h,
        rgba: rgba
            .iter()
            .copied()
            .cycle()
            .take((w * h * 4) as usize)
            .collect(),
        color_space: ColorSpace::Srgb,
        mips: Mips::None,
    }
}

const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

#[test]
fn sort_keys_order_sprites_within_a_layer_across_textures() {
    for (red_key, top) in [(-1.0, [0u8, 0, 255]), (1.0, [255, 0, 0])] {
        let mut scene = flat();
        let white = scene.add_texture(solid([255; 4], 2, 2));
        // Two textures: without keys the batch groups solid before textured.
        scene.add_sprite(Sprite {
            texture: Some(white),
            size: Vec2::splat(4.0),
            color: RED,
            ..Default::default()
        });
        scene.add_sprite(Sprite {
            size: Vec2::splat(4.0),
            color: BLUE,
            ..Default::default()
        });
        // A higher layer always wins, whatever its key.
        scene.add_sprite(Sprite {
            position: Vec3::new(3.0, 0.0, 0.0),
            size: Vec2::splat(1.0),
            color: [0.0, 1.0, 0.0, 1.0],
            layer: 1,
            ..Default::default()
        });
        scene.sprite_order = vec![
            SpriteOrder {
                sprite: 0,
                key: red_key,
            },
            SpriteOrder {
                sprite: 2,
                key: -100.0,
            },
        ];
        let (img, _) = frame(&scene, 1);
        assert_eq!(at(&img, 0.0, 0.0), top, "red key {red_key}");
        assert_eq!(at(&img, 3.0, 0.0), [0, 255, 0]);
    }
}

/// A 64x64-cell layer of 1-unit tiles in 16x16 chunks, each tile a uv of
/// one of two texels (a checker by chunk), with one animated quad.
fn checker_layer(texture: og_render::TextureHandle, layer: i32) -> TileLayer {
    let mut chunks = Vec::new();
    for cy in 0..4 {
        for cx in 0..4 {
            let min = Vec2::new(cx as f32 * 16.0, -(cy as f32 + 1.0) * 16.0);
            let uv = if (cx + cy) % 2 == 0 {
                [0.0, 0.0, 0.5, 1.0]
            } else {
                [0.5, 0.0, 1.0, 1.0]
            };
            let quads = (0..256)
                .map(|i| TileQuad {
                    min: min + Vec2::new((i % 16) as f32, (i / 16) as f32),
                    size: Vec2::ONE,
                    uv,
                })
                .collect();
            chunks.push(TileChunk {
                min,
                max: min + Vec2::splat(16.0),
                quads,
                animated: Vec::new(),
            });
        }
    }
    TileLayer {
        texture,
        chunks: Arc::new(chunks),
        origin: Vec3::new(-8.0, 8.0, 0.0),
        layer,
        color: [1.0; 4],
        repeat: [None, None],
        time: 0.0,
    }
}

fn two_texels(scene: &mut RenderScene) -> og_render::TextureHandle {
    // Left texel red, right texel blue.
    let t = scene.add_texture(TextureData {
        width: 2,
        height: 1,
        rgba: vec![255, 0, 0, 255, 0, 0, 255, 255],
        color_space: ColorSpace::Srgb,
        mips: Mips::None,
    });
    scene.set_texture_filter(t, TextureFilter::Nearest);
    t
}

#[test]
fn tile_layers_cull_chunks_draw_under_their_layer_repeat_and_animate() {
    let make = |repeat: [Option<f32>; 2]| {
        let mut scene = flat();
        let tex = two_texels(&mut scene);
        scene.tile_layers.push(TileLayer {
            repeat,
            ..checker_layer(tex, 0)
        });
        // A sprite in the same layer is drawn over the tiles.
        scene.add_sprite(Sprite {
            position: Vec3::new(2.0, 2.0, 0.0),
            size: Vec2::splat(1.0),
            color: [0.0, 1.0, 0.0, 1.0],
            ..Default::default()
        });
        scene
    };
    let (img, stats) = frame(&make([None, None]), 1);
    // The view (x -8.9..8.9, y -5..5) touches two of the sixteen chunks the
    // layer's origin (-8, 8) places: x -8..8 and 8..24 of the top row.
    assert_eq!(stats.tile_chunks, 2, "{stats:?}");
    assert_eq!(stats.tiles, 2 * 256);
    // Chunk (0, 0) is red, chunk (1, 0) blue: the boundary is at x = 8.
    assert_eq!(at(&img, 7.5, 0.0), [255, 0, 0]);
    assert_eq!(at(&img, 8.5, 0.0), [0, 0, 255]);
    assert_eq!(
        at(&img, 2.0, 2.0),
        [0, 255, 0],
        "the sprite is over its layer's tiles"
    );
    // The view's left edge (x = -8.9) is left of the layer: empty...
    assert_eq!(at(&img, -8.7, 0.0), [0, 0, 0]);
    // ...until the layer repeats every 64 units.
    let (img, stats) = frame(&make([Some(64.0), None]), 1);
    assert_eq!(
        at(&img, -8.7, 0.0),
        [0, 0, 255],
        "x -8.7 is cell 63 of the copy to the left"
    );
    assert_eq!(
        stats.tile_chunks, 3,
        "and the last chunk of the copy at -64"
    );

    // An animated quad shows the frame its time falls in.
    let mut animated = flat();
    let tex = two_texels(&mut animated);
    let mut layer = checker_layer(tex, 0);
    let chunks = Arc::make_mut(&mut layer.chunks);
    // Quad 200 of chunk 0 (red) is local (8, -4): world (0..1, 4..5).
    chunks[0].animated.push(TileAnimation {
        quad: 200,
        uvs: Arc::from(vec![[0.0, 0.0, 0.5, 1.0], [0.5, 0.0, 1.0, 1.0]]),
        durations: Arc::from(vec![0.25f32, 0.25]),
    });
    layer.time = 0.3; // the second frame: blue
    animated.tile_layers.push(layer);
    let (img, _) = frame(&animated, 1);
    assert_eq!(at(&img, 0.5, 4.5), [0, 0, 255], "the animated quad");
    assert_eq!(at(&img, 1.5, 4.5), [255, 0, 0], "its neighbour");
}

#[test]
fn a_nearest_texture_is_not_blended_between_texels() {
    let draw = |filter| {
        let mut scene = flat();
        let t = scene.add_texture(TextureData {
            width: 2,
            height: 1,
            rgba: vec![255, 0, 0, 255, 0, 0, 255, 255],
            color_space: ColorSpace::Srgb,
            mips: Mips::None,
        });
        scene.set_texture_filter(t, filter);
        scene.add_sprite(Sprite {
            texture: Some(t),
            size: Vec2::new(8.0, 4.0),
            ..Default::default()
        });
        frame(&scene, 1).0
    };
    let (nearest, linear) = (draw(TextureFilter::Nearest), draw(TextureFilter::Linear));
    // Just left of the middle: one texel exactly with nearest, a blend with linear.
    assert_eq!(at(&nearest, -0.3, 0.0), [255, 0, 0]);
    let l = at(&linear, -0.3, 0.0);
    assert!(l[0] < 250 && l[2] > 5, "linear blends: {l:?}");
}

#[test]
fn nine_slice_corners_keep_their_size_and_the_centre_stretches() {
    // 8x8 texels: a 2-texel red border around blue.
    let mut rgba = Vec::new();
    for y in 0..8 {
        for x in 0..8 {
            let border = x < 2 || y < 2 || x >= 6 || y >= 6;
            rgba.extend_from_slice(if border {
                &[255, 0, 0, 255]
            } else {
                &[0, 0, 255, 255]
            });
        }
    }
    let mut scene = flat();
    let t = scene.add_texture(TextureData {
        width: 8,
        height: 8,
        rgba,
        color_space: ColorSpace::Srgb,
        mips: Mips::None,
    });
    scene.set_texture_filter(t, TextureFilter::Nearest);
    let panel = Sprite {
        texture: Some(t),
        size: Vec2::new(12.0, 6.0),
        ..Default::default()
    };
    // 2 texels at 0.25 units each: a 0.5-unit border on any size.
    let pieces = nine_slice(&panel, [2.0, 2.0, 2.0, 2.0], (8, 8), 0.25);
    assert_eq!(pieces.len(), 9);
    for p in pieces {
        scene.add_sprite(p);
    }
    let (img, _) = frame(&scene, 1);
    assert_eq!(at(&img, -5.8, 0.0), [255, 0, 0], "left border");
    assert_eq!(at(&img, -5.3, 0.0), [0, 0, 255], "inside 0.5 from the edge");
    assert_eq!(at(&img, 0.0, 2.8), [255, 0, 0], "top border");
    assert_eq!(at(&img, 0.0, 2.3), [0, 0, 255]);
    assert_eq!(at(&img, 5.8, -2.8), [255, 0, 0], "corner");
    // A flat panel thinner than its insets clamps the border to half.
    assert_eq!(
        nine_slice(
            &Sprite {
                size: Vec2::new(0.6, 6.0),
                ..panel
            },
            [2.0; 4],
            (8, 8),
            0.25
        )
        .len(),
        6
    );
}

#[test]
fn an_outline_surrounds_text_only_when_asked() {
    let draw = |outline: Option<TextOutline>| {
        let mut scene = flat();
        let font = scenes::load_font(&mut scene).expect("font");
        scene.texts.push(Text {
            position: Vec3::new(-3.0, -1.0, 0.0),
            size: 3.0,
            color: [1.0, 1.0, 1.0, 1.0],
            ..Text::new(font, "OI")
        });
        scene.text_outlines.extend(outline);
        frame(&scene, 1).0
    };
    let plain = draw(None);
    let outlined = draw(Some(TextOutline {
        text: 0,
        width: 1.0,
        color: [1.0, 0.0, 0.0, 1.0],
    }));
    let count = |img: &RgbaImage, f: &dyn Fn(&[u8; 4]) -> bool| {
        img.pixels
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| f(p))
            .count()
    };
    let red = |p: &[u8; 4]| p[0] > 200 && p[1] < 60 && p[2] < 60;
    let white = |p: &[u8; 4]| p[0] > 200 && p[1] > 200 && p[2] > 200;
    assert_eq!(count(&plain, &red), 0);
    assert!(
        count(&outlined, &red) > 500,
        "{} outline pixels",
        count(&outlined, &red)
    );
    // The fill is still white where it was.
    let (wp, wo) = (count(&plain, &white), count(&outlined, &white));
    assert!(wo as f32 > 0.8 * wp as f32, "fill {wp} -> {wo}");
}

#[test]
fn a_layered_trail_draws_in_the_sprite_order() {
    let draw = |layer: Option<i32>| {
        let mut scene = flat();
        // A background sprite on layer 0 covers the view.
        scene.add_sprite(Sprite {
            size: Vec2::new(20.0, 12.0),
            color: [0.0, 0.0, 0.3, 1.0],
            ..Default::default()
        });
        let style = scene.add_trail_style(TrailStyle {
            lifetime: 2.0,
            width: og_render::Curve::linear(0.6, 0.6),
            color: og_render::ColorCurve::new(&[
                (0.0, [1.0, 0.5, 0.0, 1.0]),
                (1.0, [1.0, 0.5, 0.0, 1.0]),
            ]),
            ..Default::default()
        });
        let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
        for tick in 0..30u64 {
            scene.tick = tick;
            scene.trails = vec![Trail {
                id: 7,
                style,
                position: Vec3::new(-4.0 + tick as f32 * 0.25, 0.0, 0.0),
                tick,
            }];
            scene.trail_layers = layer.map(|l| vec![(7u64, l)]).unwrap_or_default();
            renderer
                .render(&scene, &RenderTarget::Offscreen)
                .expect("render");
        }
        renderer.read_offscreen().expect("capture")
    };
    let hidden = draw(None);
    let shown = draw(Some(1));
    // The trail adds orange over the dark blue background (additive).
    let orange = |p: [u8; 3]| p[0] > 200 && p[1] > 80;
    assert!(
        !orange(at(&hidden, 0.0, 0.0)),
        "under the layer-0 sprite: {:?}",
        at(&hidden, 0.0, 0.0)
    );
    assert!(
        orange(at(&shown, 0.0, 0.0)),
        "over it on layer 1: {:?}",
        at(&shown, 0.0, 0.0)
    );
}

/// A streamed texture replaces its placeholder in place: the same handle,
/// drawn by the same renderer, shows the new pixels on the next frame
/// (and the old ones again when it is put back). Ways this could fail:
/// the cached sprite group keeps the first upload, or a replacement of
/// another size is refused.
#[test]
fn a_replaced_texture_reaches_the_sprites_that_draw_it() {
    let mut scene = flat();
    let tex = scene.add_texture(solid([255, 0, 0, 255], 1, 1));
    scene.add_sprite(Sprite {
        texture: Some(tex),
        size: Vec2::splat(4.0),
        ..Default::default()
    });
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let mut draw = |scene: &RenderScene| {
        renderer
            .render(scene, &RenderTarget::Offscreen)
            .expect("render");
        at(&renderer.read_offscreen().expect("capture"), 0.0, 0.0)
    };
    assert_eq!(draw(&scene), [255, 0, 0]);
    assert!(scene.set_texture(tex, solid([0, 0, 255, 255], 8, 8)));
    assert_eq!(draw(&scene), [0, 0, 255], "the replacement is drawn");
    assert!(scene.set_texture(tex, solid([255, 0, 0, 255], 1, 1)));
    assert_eq!(draw(&scene), [255, 0, 0], "and the placeholder again");
    let mut other = flat();
    other.add_texture(solid([0; 4], 1, 1));
    let foreign = other.add_texture(solid([0; 4], 1, 1));
    assert!(
        !scene.set_texture(foreign, solid([0; 4], 1, 1)),
        "no such texture"
    );
}
