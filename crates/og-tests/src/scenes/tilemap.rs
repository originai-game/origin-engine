//! Chunked tilemap reference scene and camera update.

use super::*;

/// A 256x256 map of 1-unit tiles in four layers, drawn as og-tilemap chunks.
pub fn tilemap_256() -> RenderScene {
    use og_tilemap::{Layer, MapState, TilemapAsset, Tileset, TilesetRef, chunk_quads};
    use std::sync::Arc;
    let mut scene = RenderScene::new();
    // A 128x128 atlas of 16 tiles of 32 px, each a distinct colour.
    let mut rgba = Vec::with_capacity(128 * 128 * 4);
    for y in 0..128u32 {
        for x in 0..128u32 {
            let t = (y / 32) * 4 + x / 32;
            let edge = x % 32 == 0 || y % 32 == 0;
            let c = [
                (t * 37 % 200 + 40) as u8,
                (t * 71 % 200 + 40) as u8,
                (t * 113 % 200 + 40) as u8,
            ];
            let k = if edge { 2 } else { 1 };
            rgba.extend_from_slice(&[c[0] / k, c[1] / k, c[2] / k, 255]);
        }
    }
    let texture = scene.add_texture(TextureData {
        width: 128,
        height: 128,
        rgba,
        color_space: ColorSpace::Srgb,
        mips: Mips::None,
    });
    scene.set_texture_filter(texture, og_render::TextureFilter::Nearest);
    let tileset = Tileset::parse(
        "type tileset\nversion 1\n---\ntexture \"t.png\"\ngrid 32 32 4 4\ntile 15 {\n  frames 15 14 13\n  fps 4\n}\n",
    )
    .expect("tileset");
    let n = 256u32;
    let mut layers = Vec::new();
    for l in 0..4u32 {
        let mut layer = Layer::new(format!("l{l}"), n, n, l as i32);
        for (i, c) in layer.cells.iter_mut().enumerate() {
            let (x, y) = (i as u32 % n, i as u32 / n);
            let h = (x.wrapping_mul(73_856_093)
                ^ y.wrapping_mul(19_349_663)
                ^ l.wrapping_mul(83_492_791))
                % 100;
            // Layer 0 is full ground, the others sparser decoration.
            let filled = l == 0 || h < [100, 40, 20, 10][l as usize];
            *c = if filled { 1 + (h % 16) } else { 0 };
        }
        layers.push(layer);
    }
    let asset = TilemapAsset {
        orientation: Default::default(),
        width: n,
        height: n,
        tile_width_px: 32,
        tile_height_px: 32,
        unit: 1.0,
        tilesets: vec![TilesetRef::Embedded(tileset.clone())],
        layers,
        objects: Vec::new(),
    };
    let state = MapState::new(&asset, &[Arc::new(tileset)]).expect("map");
    let (cw, ch) = state.chunks();
    for l in 0..4usize {
        let mut chunks = Vec::new();
        for cy in 0..ch {
            for cx in 0..cw {
                let q = chunk_quads(&state, l, cx, cy, 128, 128);
                let mut min = Vec2::splat(f32::MAX);
                let mut max = Vec2::splat(f32::MIN);
                for t in &q.quads {
                    min = min.min(t.min);
                    max = max.max(t.min + t.size);
                }
                chunks.push(og_render::TileChunk {
                    min,
                    max,
                    quads: q
                        .quads
                        .iter()
                        .map(|t| og_render::TileQuad {
                            min: t.min,
                            size: t.size,
                            uv: t.uv,
                        })
                        .collect(),
                    animated: q
                        .animated
                        .iter()
                        .map(|a| og_render::TileAnimation {
                            quad: a.quad,
                            uvs: a.frames.clone(),
                            durations: a.durations.clone(),
                        })
                        .collect(),
                });
            }
        }
        scene.sprite.tile_layers.push(og_render::TileLayer {
            texture,
            chunks: Arc::new(chunks),
            origin: Vec3::new(0.0, 256.0, 0.0),
            layer: l as i32 - 10,
            color: [1.0; 4],
            repeat: [None, None],
            time: 0.0,
            lit: true,
            light_mask: 1,
        });
    }
    scene.frame.camera = Camera::orthographic_2d(Vec2::new(64.0, 128.0), 45.0);
    scene.lighting.environment.background = Background::Color(Vec3::new(0.02, 0.02, 0.03));
    scene.look.post.tonemap = Tonemap::None;
    scene
}

/// Pans the tilemap camera diagonally; layer 3 scrolls at half the camera's
/// speed and the animated tiles follow the frame's time.
pub fn pan_tilemap(scene: &mut RenderScene, frame: u32) {
    let t = frame as f32;
    let c = Vec2::new(64.0 + (t * 0.37) % 120.0, 128.0 - (t * 0.21) % 60.0);
    scene.frame.camera = Camera::orthographic_2d(c, 45.0);
    for (i, l) in scene.sprite.tile_layers.iter_mut().enumerate() {
        l.time = t / 60.0;
        l.origin = if i == 3 {
            Vec3::new(c.x * 0.5, 256.0 + c.y * 0.5, 0.0)
        } else {
            Vec3::new(0.0, 256.0, 0.0)
        };
    }
}
