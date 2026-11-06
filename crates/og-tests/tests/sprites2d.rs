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
//! not at all; a layered trail is still hidden under the sprites; a tile
//! layer in 2D lighting's range is drawn unlit (hollowbell #9), or one
//! that opts out is lit.

use glam::{Vec2, Vec3};
use og_render::{
    Background, Camera, ColorSpace, FrameStats, Mips, RenderScene, RenderTarget, Renderer,
    RgbaImage, Sprite, TextureData, TextureFilter, TileAnimation, TileChunk, TileLayer, TileQuad,
    Tonemap,
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
        lit: true,
        light_mask: 1,
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

    // The same resident chunk advances, wraps and seeks backwards without
    // a geometry upload; a copy-on-write edit must replace its GPU data.
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    for (time, expected) in [
        (0.0, [255, 0, 0]),
        (0.249, [255, 0, 0]),
        (0.25, [0, 0, 255]),
        (0.5, [255, 0, 0]),
        (-0.125, [0, 0, 255]),
    ] {
        animated.tile_layers[0].time = time;
        let stats = renderer
            .render(&animated, &RenderTarget::Offscreen)
            .unwrap();
        assert_eq!((stats.tiles, stats.tile_chunks), (512, 2));
        assert_eq!(
            at(&renderer.read_offscreen().unwrap(), 0.5, 4.5),
            expected,
            "time {time}"
        );
    }
    let layer = &mut animated.tile_layers[0];
    layer.time = 0.25;
    Arc::make_mut(&mut layer.chunks)[0].animated[0].uvs =
        Arc::from(vec![[0.5, 0.0, 1.0, 1.0], [0.0, 0.0, 0.5, 1.0]]);
    renderer
        .render(&animated, &RenderTarget::Offscreen)
        .unwrap();
    assert_eq!(
        at(&renderer.read_offscreen().unwrap(), 0.5, 4.5),
        [255, 0, 0],
        "edited animation replaces cached UVs"
    );
}

/// R11: a spot light lights its cone along its axis and nothing behind
/// it; a global light lights every lit pixel; a sprite with a light mask
/// takes only the lights whose layers share a bit with it.
///
/// Ways this could fail: a spot drawn as a point light (lit behind it);
/// the cone turned the other way; a global light clipped to its radius or
/// shadowed; a mask ignored (the receiver lit by every light), or the
/// strips mixed up so a mask-2 receiver reads mask 1's light.
#[test]
fn spot_global_and_masked_2d_lights() {
    use og_render::{Light2d, Light2dKind, Lighting2d};
    let white = |scene: &mut RenderScene, x: f32, mask: Option<u32>| {
        let i = scene.sprites.len();
        scene.add_sprite(Sprite {
            position: Vec3::new(x, 0.0, 0.0),
            size: Vec2::new(4.0, 8.0),
            ..Default::default()
        });
        if let Some(m) = mask {
            scene.sprite_light_masks.resize(i, 1);
            scene.sprite_light_masks.push(m);
        }
    };
    let lit = |scene: &mut RenderScene| {
        scene.lighting_2d = Some(Lighting2d {
            ambient: Vec3::splat(0.0),
            min_layer: -10,
            max_layer: 10,
            resolution: 1.0,
            shadow_angles: 1024,
        });
    };
    // A spot at the origin pointing +x, 0.3..0.5 rad either side.
    let mut scene = flat();
    white(&mut scene, -4.0, None);
    white(&mut scene, 4.0, None);
    lit(&mut scene);
    scene.lights_2d.push(Light2d {
        position: Vec2::ZERO,
        radius: 8.0,
        falloff: 1.0,
        kind: Light2dKind::Spot,
        direction: Vec2::X,
        inner_angle: 0.3,
        outer_angle: 0.5,
        cast_shadows: false,
        ..Default::default()
    });
    let (img, _) = frame(&scene, 1);
    let ahead = at(&img, 3.0, 0.0)[0];
    let behind = at(&img, -3.0, 0.0)[0];
    let off_axis = at(&img, 3.0, 3.0)[0];
    assert!(ahead > 100, "on the axis the spot lights: {ahead}");
    assert_eq!(behind, 0, "behind the spot: dark");
    assert_eq!(off_axis, 0, "past its outer angle (45°): dark");

    // A global light reaches every lit pixel at full colour, whatever its
    // position and radius; a mask-2 sprite takes only a layer-2 light.
    let mut scene = flat();
    white(&mut scene, -4.0, None);
    white(&mut scene, 4.0, Some(2));
    lit(&mut scene);
    scene.lights_2d.push(Light2d {
        position: Vec2::new(100.0, 100.0),
        radius: 0.1,
        color: Vec3::new(1.0, 0.0, 0.0),
        intensity: 0.5,
        kind: Light2dKind::Global,
        layers: 1,
        ..Default::default()
    });
    scene.lights_2d.push(Light2d {
        position: Vec2::ZERO,
        radius: 0.1,
        color: Vec3::new(0.0, 0.0, 1.0),
        kind: Light2dKind::Global,
        layers: 2,
        ..Default::default()
    });
    let (img, _) = frame(&scene, 1);
    let left = at(&img, -4.0, 2.0);
    let right = at(&img, 4.0, 2.0);
    assert!(
        left[0] > 150 && left[2] == 0,
        "mask 1 takes the red global light only: {left:?}"
    );
    assert!(
        right[2] > 240 && right[0] == 0,
        "mask 2 takes the blue global light only: {right:?}"
    );
    // The strip edge: the right sprite's left side still reads its strip.
    let edge = at(&img, 2.1, -3.5);
    assert!(
        edge[2] > 240 && edge[0] == 0,
        "no bleed from the next strip: {edge:?}"
    );
}
