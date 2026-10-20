//! The 2D batch: sprites and MSDF text as instanced quads, sorted by
//! (layer, kind, texture) so each run of one texture (or one font) in one
//! layer is a single instanced draw.

use crate::RenderError;
use crate::effects::SpriteBlend;
use crate::scene::{
    FontHandle, Glyph, MsdfFont, RenderScene, Sprite, Text, TextureArrayHandle, TextureHandle,
    TileLayer,
};
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
    /// The texture array layer an array sprite draws (sprite_array.wgsl;
    /// the other sprite pipelines do not read it).
    layer: u32,
}

pub(crate) const SPRITE_STRIDE: u64 = std::mem::size_of::<SpriteInstance>() as u64;

pub(crate) const SPRITE_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: SPRITE_STRIDE,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4, 4 => Unorm8x4, 5 => Float32x3],
};

/// [`SPRITE_LAYOUT`] with the instance's layer (location 6), for the array
/// sprite pipelines.
pub(crate) const SPRITE_ARRAY_LAYOUT: wgpu::VertexBufferLayout<'static> =
    wgpu::VertexBufferLayout {
        array_stride: SPRITE_STRIDE,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4, 4 => Unorm8x4, 5 => Float32x3, 6 => Uint32],
    };

/// What one run draws with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum RunSource {
    /// A sprite texture (`None`: solid white) and its blend.
    Sprite(SpriteBlend, Option<TextureHandle>),
    Font(FontHandle),
    /// A 2D trail's strip: `first`/`count` are vertices of the ribbon
    /// buffer; the blend is `ParticleBlend as u8`.
    Strip(u8, Option<TextureHandle>),
    /// A 2D fluid's metaball composite (the fluid's index this frame).
    Metaballs(u8),
    /// The overlay's glass panels (the blur pyramid, src/glass.rs).
    Glass,
    /// Sprites drawn from layers of a texture array, and their blend.
    Array(SpriteBlend, TextureArrayHandle),
}

/// A ribbon run drawn in the sprite order at `layer` (a 2D trail, a 2D
/// water body, a fluid's metaball composite).
pub(crate) struct LayeredStrip {
    pub layer: i32,
    pub source: RunSource,
    pub first: u32,
    pub count: u32,
}

/// Consecutive instances drawn with one bind group and one pipeline.
pub(crate) struct SpriteRun {
    pub source: RunSource,
    pub first: u32,
    pub count: u32,
}

/// The glyph table of one font, by codepoint, and its kerning pairs.
pub(crate) struct FontLayout {
    pub glyphs: HashMap<char, Glyph>,
    pub kerning: HashMap<(char, char), f32>,
    pub line_height: f32,
    pub atlas_size: Vec2,
}

impl FontLayout {
    pub fn new(font: &MsdfFont, atlas_size: Vec2) -> FontLayout {
        FontLayout {
            glyphs: font.glyphs.iter().map(|g| (g.codepoint, *g)).collect(),
            kerning: font
                .kerning
                .iter()
                .map(|k| ((k.left, k.right), k.adjust))
                .collect(),
            line_height: font.line_height,
            atlas_size,
        }
    }

    /// Walks a text's characters with the pen (em; lines step by
    /// `line_step`, the line height down the overlay or up the world):
    /// `f(char, glyph or None when the font lacks it, pen, offset)`. The pen
    /// moves by the text's placements when it has them, else by each glyph's
    /// advance, the kerning with the character before it and the tracking
    /// (a missing glyph takes no space and breaks the kerning pair).
    pub fn walk(
        &self,
        text: &Text,
        line_step: f32,
        mut f: impl FnMut(char, Option<&Glyph>, Vec2, Vec2),
    ) {
        let mut pen = Vec2::ZERO;
        let mut prev: Option<char> = None;
        for (i, ch) in text.text.chars().enumerate() {
            if ch == '\n' {
                pen = Vec2::new(0.0, pen.y + line_step);
                prev = None;
                continue;
            }
            let g = self.glyphs.get(&ch);
            if let Some(p) = text.placements.get(i) {
                f(ch, g, pen, Vec2::new(p.x, p.y));
                pen.x += p.advance;
                continue;
            }
            let Some(g) = g else {
                f(ch, None, pen, Vec2::ZERO);
                prev = None;
                continue;
            };
            if let Some(k) = prev.and_then(|p| self.kerning.get(&(p, ch))) {
                pen.x += k;
            }
            f(ch, Some(g), pen, Vec2::ZERO);
            pen.x += g.advance + text.tracking;
            prev = Some(ch);
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
    /// Tile quads drawn (after culling) and chunks they came from.
    pub tiles: u32,
    pub tile_chunks: u32,
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
        layer: 0,
    }
}

/// The world rectangle an orthographic camera sees (for culling tiles).
#[derive(Clone, Copy, Debug)]
pub(crate) struct ViewRect {
    pub min: Vec2,
    pub max: Vec2,
}

/// Appends the instances of a tile layer's chunks inside `view` (all of
/// them without one), per repeat, and returns (quads, chunks).
fn tile_instances(
    layer: &TileLayer,
    view: Option<ViewRect>,
    out: &mut Vec<SpriteInstance>,
) -> (u32, u32) {
    let origin = layer.origin.truncate();
    let mut offsets: Vec<Vec2> = vec![Vec2::ZERO];
    if let Some(v) = view {
        // Repeats: every copy of the layer that can touch the view.
        let mut bounds = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
        for c in layer.chunks.iter() {
            bounds.0 = bounds.0.min(c.min);
            bounds.1 = bounds.1.max(c.max);
        }
        for axis in 0..2 {
            let Some(period) = layer.repeat[axis].filter(|p| *p > 0.0) else {
                continue;
            };
            let lo = (v.min[axis] - origin[axis] - bounds.1[axis]) / period;
            let hi = (v.max[axis] - origin[axis] - bounds.0[axis]) / period;
            let (lo, hi) = (lo.floor() as i64, hi.ceil() as i64);
            let mut next = Vec::new();
            for o in &offsets {
                for k in lo.max(-4096)..=hi.min(lo.max(-4096) + 4096) {
                    let mut n = *o;
                    n[axis] = k as f32 * period;
                    next.push(n);
                }
            }
            offsets = next;
        }
    }
    let (mut quads, mut chunks) = (0u32, 0u32);
    for off in offsets {
        let at = origin + off;
        for c in layer.chunks.iter() {
            if let Some(v) = view
                && (at.x + c.max.x < v.min.x
                    || at.x + c.min.x > v.max.x
                    || at.y + c.max.y < v.min.y
                    || at.y + c.min.y > v.max.y)
            {
                continue;
            }
            chunks += 1;
            let first = out.len();
            for q in &c.quads {
                out.push(SpriteInstance {
                    position_rotation: [at.x + q.min.x, at.y + q.min.y, layer.origin.z, 0.0],
                    size_anchor: [q.size.x, q.size.y, 0.0, 0.0],
                    uv_rect: q.uv,
                    color: layer.color,
                    flash: NO_FLASH,
                    fx: [0.0, 0.0, 0.0],
                    layer: 0,
                });
            }
            for a in &c.animated {
                let total: f32 = a.durations.iter().sum();
                if total <= 0.0 || a.uvs.is_empty() {
                    continue;
                }
                let mut t = layer.time.rem_euclid(total);
                let mut frame = 0;
                for (i, d) in a.durations.iter().enumerate() {
                    if t < *d {
                        frame = i;
                        break;
                    }
                    t -= d;
                    frame = i;
                }
                if let Some(inst) = out.get_mut(first + a.quad as usize) {
                    inst.uv_rect = a.uvs[frame.min(a.uvs.len() - 1)];
                }
            }
            quads += c.quads.len() as u32;
        }
    }
    (quads, chunks)
}

/// Sorts the scene's sprites, the frame's 2D particles (`extra`: sprite,
/// blend, glow) and texts into runs, with the tile layers and the layered
/// strips placed before the sprites of their layer. `font(i)` is the layout
/// of the scene's font `i`; `array_layers(a)` the layers of an uploaded
/// texture array (`None`: not uploaded or removed); `lit` says which layers
/// 2D lighting applies to; `view` culls tile chunks (orthographic cameras).
/// A sprite drawn from an array ([`RenderScene::sprite_arrays`]) joins its
/// array's run and never the glow pass.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build<'a>(
    scene: &RenderScene,
    font: impl Fn(usize) -> Option<&'a FontLayout>,
    texture_count: usize,
    array_layers: impl Fn(TextureArrayHandle) -> Option<u32>,
    extra: &[(Sprite, SpriteBlend, f32)],
    lit: impl Fn(i32) -> bool,
    view: Option<ViewRect>,
    strips: &[LayeredStrip],
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
    let mut order: HashMap<u32, f32> = HashMap::new();
    for o in &scene.sprite_order {
        if o.sprite as usize >= scene.sprites.len() {
            return Err(RenderError::InvalidHandle(format!(
                "sprite_order: sprite {} of {}",
                o.sprite,
                scene.sprites.len()
            )));
        }
        order.insert(o.sprite, if o.key.is_finite() { o.key } else { 0.0 });
    }
    // Sprites drawn from texture arrays, by index (the last entry wins).
    let mut arrays: HashMap<u32, (TextureArrayHandle, u32)> = HashMap::new();
    for a in &scene.sprite_arrays {
        if a.sprite as usize >= scene.sprites.len() {
            return Err(RenderError::InvalidHandle(format!(
                "sprite_arrays: sprite {} of {}",
                a.sprite,
                scene.sprites.len()
            )));
        }
        let layers = array_layers(a.array).ok_or_else(|| {
            RenderError::InvalidHandle(format!(
                "sprite {}: texture array {}",
                a.sprite,
                a.array.index()
            ))
        })?;
        if a.layer >= layers {
            return Err(RenderError::InvalidData(format!(
                "sprite {}: layer {} of a texture array of {layers}",
                a.sprite, a.layer
            )));
        }
        arrays.insert(a.sprite, (a.array, a.layer));
    }
    let outlines: HashMap<u32, &crate::TextOutline> =
        scene.text_outlines.iter().map(|o| (o.text, o)).collect();
    // (layer, key, source, instance), in submission order.
    let mut keyed: Vec<(i32, f32, RunSource, SpriteInstance)> =
        Vec::with_capacity(scene.sprites.len() + extra.len());
    let mut glowing: Vec<(Option<TextureHandle>, SpriteInstance)> = Vec::new();
    for (i, s) in scene.sprites.iter().enumerate() {
        if let Some(t) = s.texture
            && t.index() >= texture_count
        {
            return Err(RenderError::InvalidHandle(format!(
                "sprite {i}: texture {}",
                t.index()
            )));
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
        let mut inst = sprite_instance(s, flash, glow, lit(s.layer), false);
        let key = order.get(&(i as u32)).copied().unwrap_or(0.0);
        if let Some(&(array, layer)) = arrays.get(&(i as u32)) {
            inst.layer = layer;
            keyed.push((s.layer, key, RunSource::Array(blend, array), inst));
            continue;
        }
        if glow > 0.0 {
            glowing.push((s.texture, inst));
        }
        keyed.push((s.layer, key, RunSource::Sprite(blend, s.texture), inst));
    }
    for (s, blend, glow) in extra {
        let soft = s.texture.is_none();
        let inst = sprite_instance(s, [1.0, 1.0, 1.0, 0.0], *glow, lit(s.layer), soft);
        if *glow > 0.0 {
            glowing.push((s.texture, inst));
        }
        keyed.push((s.layer, 0.0, RunSource::Sprite(*blend, s.texture), inst));
    }
    let sprites = keyed.len() as u32;
    let mut missing_glyphs = 0;
    for (i, text) in scene.texts.iter().enumerate() {
        let font = font(text.font.index()).ok_or_else(|| {
            RenderError::InvalidHandle(format!("text {i}: font {}", text.font.index()))
        })?;
        // An outline rides on the glyph's flash colour and glow strength
        // (which plain glyphs leave unused).
        let (outline_color, outline) = match outlines.get(&(i as u32)) {
            Some(o) if o.width > 0.0 => (
                o.color.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8),
                o.width.min(1.0),
            ),
            _ => ([255, 255, 255, 0], 0.0),
        };
        let (sin, cos) = text.rotation.sin_cos();
        let rotate = |v: Vec2| Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos);
        let origin = text.position.truncate();
        font.walk(text, -font.line_height, |ch, g, pen, offset| {
            let Some(g) = g else {
                if !ch.is_whitespace() {
                    missing_glyphs += 1;
                }
                return;
            };
            let [l, b, r, t] = g.plane_bounds;
            if r > l && t > b {
                let corner = (pen + offset + Vec2::new(l, b)) * text.size;
                let [al, at, ar, ab] = g.atlas_bounds;
                keyed.push((
                    text.layer,
                    0.0,
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
                        flash: outline_color,
                        fx: [outline, 0.0, 0.0],
                        layer: 0,
                    },
                ));
            }
        });
    }
    let glyphs = keyed.len() as u32 - sprites;
    // Stable: within a (layer, key, source) run, submission order is kept.
    keyed.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)).then(a.2.cmp(&b.2)));
    // Tile layers and layered strips, each before the sprites of its layer
    // (tiles first), in submission order within a layer.
    let mut under: Vec<(i32, u8, usize)> = Vec::new();
    for (i, t) in scene.tile_layers.iter().enumerate() {
        if t.texture.index() >= texture_count {
            return Err(RenderError::InvalidHandle(format!(
                "tile layer {i}: texture {}",
                t.texture.index()
            )));
        }
        under.push((t.layer, 0, i));
    }
    for (i, s) in strips.iter().enumerate() {
        under.push((s.layer, 1, i));
    }
    under.sort_by_key(|u| (u.0, u.1));
    let mut runs: Vec<SpriteRun> = Vec::new();
    let mut instances: Vec<SpriteInstance> = Vec::with_capacity(keyed.len());
    let push = |runs: &mut Vec<SpriteRun>, source: RunSource, first: u32, count: u32| {
        if count == 0 {
            return;
        }
        match runs.last_mut() {
            Some(run)
                if run.source == source
                    && !matches!(source, RunSource::Strip(..) | RunSource::Metaballs(_))
                    && run.first + run.count == first =>
            {
                run.count += count
            }
            _ => runs.push(SpriteRun {
                source,
                first,
                count,
            }),
        }
    };
    let (mut tiles, mut tile_chunks) = (0u32, 0u32);
    let mut next_under = 0;
    let mut place_under = |below: i32,
                           instances: &mut Vec<SpriteInstance>,
                           runs: &mut Vec<SpriteRun>,
                           next_under: &mut usize| {
        while *next_under < under.len() && under[*next_under].0 <= below {
            let (_, kind, i) = under[*next_under];
            if kind == 0 {
                let layer = &scene.tile_layers[i];
                let first = instances.len() as u32;
                let (q, c) = tile_instances(layer, view, instances);
                tiles += q;
                tile_chunks += c;
                push(
                    runs,
                    RunSource::Sprite(SpriteBlend::Alpha, Some(layer.texture)),
                    first,
                    q,
                );
            } else {
                let s = &strips[i];
                push(runs, s.source, s.first, s.count);
            }
            *next_under += 1;
        }
    };
    for (layer, _, source, inst) in keyed {
        place_under(layer, &mut instances, &mut runs, &mut next_under);
        let index = instances.len() as u32;
        instances.push(inst);
        push(&mut runs, source, index, 1);
    }
    place_under(i32::MAX, &mut instances, &mut runs, &mut next_under);
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
        tiles,
        tile_chunks,
    })
}
