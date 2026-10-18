//! The 2D batch: sprites and MSDF text as instanced quads, sorted by
//! (layer, kind, texture) so each run of one texture (or one font) in one
//! layer is a single instanced draw.

use crate::effects::SpriteBlend;
use crate::scene::{FontHandle, Glyph, RenderScene, Sprite, TextureHandle};
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
    /// rgb: the flash colour, a: how far the sprite mixes towards it.
    flash: [u8; 4],
    /// x: glow strength, y: 1 when 2D lighting applies, z: 1 for a soft dot
    /// (untextured particles).
    fx: [f32; 3],
}

pub(crate) const SPRITE_STRIDE: u64 = std::mem::size_of::<SpriteInstance>() as u64;

pub(crate) const SPRITE_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: SPRITE_STRIDE,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4, 4 => Unorm8x4, 5 => Float32x3],
};

/// What one run draws with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum RunSource {
    /// A sprite texture (`None`: solid white) and its blend.
    Sprite(SpriteBlend, Option<TextureHandle>),
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
    /// Glowing sprites again, after `instances` in the buffer, by texture.
    pub glow_runs: Vec<SpriteRun>,
    pub sprites: u32,
    pub glyphs: u32,
    pub missing_glyphs: u32,
}

/// A sprite's instance with its effects.
const NO_FLASH: [u8; 4] = [255, 255, 255, 0];

fn sprite_instance(
    s: &Sprite,
    flash: [f32; 4],
    glow: f32,
    lit: bool,
    soft: bool,
) -> SpriteInstance {
    let unorm = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    SpriteInstance {
        position_rotation: s.position.extend(s.rotation).to_array(),
        size_anchor: [s.size.x, s.size.y, s.anchor.x, s.anchor.y],
        uv_rect: s.uv_rect,
        color: s.color,
        flash: if flash[3] > 0.0 {
            flash.map(unorm)
        } else {
            NO_FLASH
        },
        fx: [glow, lit as u32 as f32, soft as u32 as f32],
    }
}

/// Sorts the scene's sprites, the frame's 2D particles (`extra`: sprite,
/// blend, glow) and texts into runs. `font(i)` is the layout of the
/// scene's font `i`; `lit` says which layers 2D lighting applies to.
pub(crate) fn build<'a>(
    scene: &RenderScene,
    font: impl Fn(usize) -> Option<&'a FontLayout>,
    texture_count: usize,
    extra: &[(Sprite, SpriteBlend, f32)],
    lit: impl Fn(i32) -> bool,
) -> Result<SpriteFrame, RenderError> {
    // Per-sprite effects by index (the last entry for an index wins).
    let mut fx: HashMap<u32, &crate::SpriteFx> = HashMap::new();
    for f in &scene.sprite_fx {
        if f.sprite as usize >= scene.sprites.len() {
            return Err(RenderError::InvalidHandle(format!(
                "sprite_fx: sprite {} of {}",
                f.sprite,
                scene.sprites.len()
            )));
        }
        fx.insert(f.sprite, f);
    }
    // (layer, source, instance), in submission order.
    let mut keyed: Vec<(i32, RunSource, SpriteInstance)> =
        Vec::with_capacity(scene.sprites.len() + extra.len());
    let mut glowing: Vec<(Option<TextureHandle>, SpriteInstance)> = Vec::new();
    for (i, s) in scene.sprites.iter().enumerate() {
        if let Some(t) = s.texture {
            if t.index() >= texture_count {
                return Err(RenderError::InvalidHandle(format!(
                    "sprite {i}: texture {}",
                    t.index()
                )));
            }
        }
        let found = if fx.is_empty() {
            None
        } else {
            fx.get(&(i as u32))
        };
        let (flash, glow, blend) = match found {
            Some(f) => (f.flash, f.glow.max(0.0), f.blend),
            None => ([1.0, 1.0, 1.0, 0.0], 0.0, SpriteBlend::Alpha),
        };
        let inst = sprite_instance(s, flash, glow, lit(s.layer), false);
        if glow > 0.0 {
            glowing.push((s.texture, inst));
        }
        keyed.push((s.layer, RunSource::Sprite(blend, s.texture), inst));
    }
    for (s, blend, glow) in extra {
        let soft = s.texture.is_none();
        let inst = sprite_instance(s, [1.0, 1.0, 1.0, 0.0], *glow, lit(s.layer), soft);
        if *glow > 0.0 {
            glowing.push((s.texture, inst));
        }
        keyed.push((s.layer, RunSource::Sprite(*blend, s.texture), inst));
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
                        flash: [255, 255, 255, 0],
                        fx: [0.0, 0.0, 0.0],
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
    // The glow pass: by texture (additive, so order does not matter).
    glowing.sort_by_key(|(t, _)| *t);
    let mut glow_runs: Vec<SpriteRun> = Vec::new();
    for (t, inst) in glowing {
        let index = instances.len() as u32;
        match glow_runs.last_mut() {
            Some(run) if run.source == RunSource::Sprite(SpriteBlend::Additive, t) => {
                run.count += 1
            }
            _ => glow_runs.push(SpriteRun {
                source: RunSource::Sprite(SpriteBlend::Additive, t),
                first: index,
                count: 1,
            }),
        }
        instances.push(inst);
    }
    Ok(SpriteFrame {
        instances,
        runs,
        glow_runs,
        sprites,
        glyphs,
        missing_glyphs,
    })
}
