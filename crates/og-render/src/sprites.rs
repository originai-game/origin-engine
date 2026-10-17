//! The 2D batch: sprites and MSDF text as instanced quads, sorted by
//! (layer, kind, texture) so each run of one texture (or one font) in one
//! layer is a single instanced draw.

use crate::scene::{FontHandle, Glyph, RenderScene, TextureHandle};
use crate::RenderError;
use glam::Vec2;
use std::collections::HashMap;

/// `struct SpriteIn` in sprite.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct SpriteInstance {
    position_rotation: [f32; 4],
    size_anchor: [f32; 4],
    uv_rect: [f32; 4],
    color: [f32; 4],
}

pub(crate) const SPRITE_STRIDE: u64 = std::mem::size_of::<SpriteInstance>() as u64;

pub(crate) const SPRITE_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: SPRITE_STRIDE,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4],
};

/// What one run draws with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum RunSource {
    /// A sprite texture (`None`: solid white).
    Sprite(Option<TextureHandle>),
    Font(FontHandle),
}

/// Consecutive instances drawn with one bind group and one pipeline.
pub(crate) struct SpriteRun {
    pub source: RunSource,
    pub first: u32,
    pub count: u32,
}

/// The glyph table of one font, by codepoint.
pub(crate) struct FontLayout {
    glyphs: HashMap<char, Glyph>,
    line_height: f32,
    atlas_size: Vec2,
}

impl FontLayout {
    pub fn new(glyphs: &[Glyph], line_height: f32, atlas_size: Vec2) -> FontLayout {
        FontLayout {
            glyphs: glyphs.iter().map(|g| (g.codepoint, *g)).collect(),
            line_height,
            atlas_size,
        }
    }
}

/// The sorted instances of a frame's sprites and text.
pub(crate) struct SpriteFrame {
    pub instances: Vec<SpriteInstance>,
    pub runs: Vec<SpriteRun>,
    pub sprites: u32,
    pub glyphs: u32,
    pub missing_glyphs: u32,
}

/// Sorts the scene's sprites and texts into runs. `font(i)` is the layout of
/// the scene's font `i`.
pub(crate) fn build<'a>(
    scene: &RenderScene,
    font: impl Fn(usize) -> Option<&'a FontLayout>,
    texture_count: usize,
) -> Result<SpriteFrame, RenderError> {
    // (layer, source, instance), in submission order.
    let mut keyed: Vec<(i32, RunSource, SpriteInstance)> = Vec::with_capacity(scene.sprites.len());
    for (i, s) in scene.sprites.iter().enumerate() {
        if let Some(t) = s.texture {
            if t.index() >= texture_count {
                return Err(RenderError::InvalidHandle(format!(
                    "sprite {i}: texture {}",
                    t.index()
                )));
            }
        }
        keyed.push((
            s.layer,
            RunSource::Sprite(s.texture),
            SpriteInstance {
                position_rotation: s.position.extend(s.rotation).to_array(),
                size_anchor: [s.size.x, s.size.y, s.anchor.x, s.anchor.y],
                uv_rect: s.uv_rect,
                color: s.color,
            },
        ));
    }
    let sprites = keyed.len() as u32;
    let mut missing_glyphs = 0;
    for (i, text) in scene.texts.iter().enumerate() {
        let font = font(text.font.index()).ok_or_else(|| {
            RenderError::InvalidHandle(format!("text {i}: font {}", text.font.index()))
        })?;
        let (sin, cos) = text.rotation.sin_cos();
        let rotate = |v: Vec2| Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos);
        let origin = text.position.truncate();
        let mut pen = Vec2::ZERO;
        for ch in text.text.chars() {
            if ch == '\n' {
                pen = Vec2::new(0.0, pen.y - font.line_height);
                continue;
            }
            let Some(g) = font.glyphs.get(&ch) else {
                if !ch.is_whitespace() {
                    missing_glyphs += 1;
                }
                continue;
            };
            let [l, b, r, t] = g.plane_bounds;
            if r > l && t > b {
                let corner = (pen + Vec2::new(l, b)) * text.size;
                let [al, at, ar, ab] = g.atlas_bounds;
                keyed.push((
                    text.layer,
                    RunSource::Font(text.font),
                    SpriteInstance {
                        position_rotation: [
                            origin.x + rotate(corner).x,
                            origin.y + rotate(corner).y,
                            text.position.z,
                            text.rotation,
                        ],
                        size_anchor: [(r - l) * text.size, (t - b) * text.size, 0.0, 0.0],
                        uv_rect: [
                            al / font.atlas_size.x,
                            at / font.atlas_size.y,
                            ar / font.atlas_size.x,
                            ab / font.atlas_size.y,
                        ],
                        color: text.color,
                    },
                ));
            }
            pen.x += g.advance;
        }
    }
    let glyphs = keyed.len() as u32 - sprites;
    // Stable: within a (layer, source) run, submission order is kept.
    keyed.sort_by_key(|(layer, source, _)| (*layer, *source));
    // Adjacent entries with one source share a draw, even across layers: the
    // order they draw in is the sorted order either way.
    let mut runs: Vec<SpriteRun> = Vec::new();
    let mut instances = Vec::with_capacity(keyed.len());
    for (index, (_, source, inst)) in keyed.into_iter().enumerate() {
        match runs.last_mut() {
            Some(run) if run.source == source => run.count += 1,
            _ => runs.push(SpriteRun {
                source,
                first: index as u32,
                count: 1,
            }),
        }
        instances.push(inst);
    }
    Ok(SpriteFrame {
        instances,
        runs,
        sprites,
        glyphs,
        missing_glyphs,
    })
}
