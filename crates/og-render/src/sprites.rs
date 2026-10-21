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
    /// (untextured particles). A drop shadow (none of those) carries its
    /// ground's slope in yz instead: the rise per metre along x and z; a
    /// cell of a bent one its corners' heights (below).
    fx: [f32; 3],
    /// A drop shadow's projection (`SpriteShadow`): x 1 projects the
    /// standing quad onto the ground, yz the x and z it moves per metre of
    /// height, w the ground's height under the anchor. All 0 for every
    /// other instance. x 2: one cell of a bent shadow (`ShadowBend`): the
    /// cell's column and row in `flash`'s first two bytes, its corners'
    /// heights in w (bottom left), fx.x (bottom right), fx.y (top right)
    /// and fx.z (top left).
    ground: [f32; 4],
    /// The texture array layer an array sprite draws (sprite_array.wgsl;
    /// the other sprite pipelines do not read it).
    layer: u32,
}

/// An instance that stands where it is drawn.
const NO_GROUND: [f32; 4] = [0.0; 4];

pub(crate) const SPRITE_STRIDE: u64 = std::mem::size_of::<SpriteInstance>() as u64;

pub(crate) const SPRITE_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: SPRITE_STRIDE,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4, 4 => Unorm8x4, 5 => Float32x3, 6 => Float32x4],
};

/// [`SPRITE_LAYOUT`] with the instance's layer (location 7), for the array
/// sprite pipelines.
pub(crate) const SPRITE_ARRAY_LAYOUT: wgpu::VertexBufferLayout<'static> =
    wgpu::VertexBufferLayout {
        array_stride: SPRITE_STRIDE,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4, 4 => Unorm8x4, 5 => Float32x3, 6 => Float32x4, 7 => Uint32],
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
    /// False: alpha-blended sprites kept out of exposure, bloom and the
    /// tone curve ([`crate::SpriteFx::tone_mapped`]), drawn with the
    /// untonemapped pipelines.
    pub tone_mapped: bool,
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
    /// Whether some run is untonemapped ([`SpriteRun::tone_mapped`]).
    pub untonemapped: bool,
    /// Glowing sprites again, after `instances` in the buffer, by texture
    /// or array (`RunSource::Sprite` or `RunSource::Array`, additive).
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
        ground: NO_GROUND,
        layer: 0,
    }
}

/// A 2D particle as the batch draws it: its sprite, blend and glow, the
/// soft dot's round edge (an untextured particle's, or an emitter's
/// `round`), and the texture array layer it shows instead of its texture.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ExtraSprite {
    pub sprite: Sprite,
    pub blend: SpriteBlend,
    pub glow: f32,
    pub soft: bool,
    pub array: Option<(TextureArrayHandle, u32)>,
}

/// Sprites drawn from texture arrays, by sprite index (the last entry for
/// an index wins), checked against `count` sprites and the arrays' layers
/// (`array_layers`: `None` for an array not uploaded or removed).
pub(crate) fn array_entries(
    entries: &[crate::SpriteArray],
    count: usize,
    what: &str,
    array_layers: impl Fn(TextureArrayHandle) -> Option<u32>,
) -> Result<HashMap<u32, (TextureArrayHandle, u32)>, RenderError> {
    let mut arrays = HashMap::new();
    for a in entries {
        if a.sprite as usize >= count {
            return Err(RenderError::InvalidHandle(format!(
                "{what} arrays: sprite {} of {count}",
                a.sprite
            )));
        }
        let layers = array_layers(a.array).ok_or_else(|| {
            RenderError::InvalidHandle(format!(
                "{what} {}: texture array {}",
                a.sprite,
                a.array.index()
            ))
        })?;
        if a.layer >= layers {
            return Err(RenderError::InvalidData(format!(
                "{what} {}: layer {} of a texture array of {layers}",
                a.sprite, a.layer
            )));
        }
        arrays.insert(a.sprite, (a.array, a.layer));
    }
    Ok(arrays)
}

/// The drop-shadow projection of a scene's sun (`SpriteInstance::ground`):
/// `None` with the sun at or below the horizon. A low sun is raised to
/// about 11° so a shadow is at most five times the sprite's height.
pub(crate) fn shadow_ground(direction_to_light: glam::Vec3) -> Option<[f32; 4]> {
    let l = direction_to_light;
    if l.y.is_nan() || l.y <= 1e-4 {
        return None;
    }
    let ly = l.y.max(0.2 * l.length());
    Some([1.0, -l.x / ly, -l.z / ly, crate::SPRITE_SHADOW_LIFT])
}

/// The cells of a bent drop shadow (`ShadowBend`), each `shadow` (the
/// whole quad's instance) marked as a cell (`ground.x` 2) with its column
/// and row and its corners' heights raised by `lift`. The shader computes
/// each corner from the whole quad, so neighbouring cells meet exactly.
fn bent_shadow(
    shadow: SpriteInstance,
    bend: &crate::ShadowBend,
    lift: f32,
) -> impl Iterator<Item = SpriteInstance> + '_ {
    let h = move |row: usize, column: usize| {
        let v = bend.heights[row][column];
        if v.is_finite() { v + lift } else { lift }
    };
    (0..crate::SHADOW_BEND_ROWS - 1).flat_map(move |row| {
        (0..crate::SHADOW_BEND_COLUMNS - 1).map(move |column| {
            let mut cell = shadow;
            cell.ground = [2.0, shadow.ground[1], shadow.ground[2], h(row, column)];
            cell.fx = [
                h(row, column + 1),
                h(row + 1, column + 1),
                h(row + 1, column),
            ];
            cell.flash = [column as u8, row as u8, 0, 0];
            cell
        })
    })
}

/// The world rectangle an orthographic camera sees (for culling tiles).
#[derive(Clone, Copy, Debug)]
pub(crate) struct ViewRect {
    pub min: Vec2,
    pub max: Vec2,
}

/// Appends the instances of a tile layer's chunks inside `view` (all of
/// them without one), per repeat, and returns (quads, chunks). `lit`: the
/// tiles take the 2D light map as a lit sprite does.
fn tile_instances(
    layer: &TileLayer,
    view: Option<ViewRect>,
    lit: bool,
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
                    fx: [0.0, lit as u32 as f32, 0.0],
                    ground: NO_GROUND,
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

/// Sorts the scene's sprites, the frame's 2D particles (`extra`) and texts
/// into runs, with the tile layers and the layered strips placed before
/// the sprites of their layer. `font(i)` is the layout of the scene's font
/// `i`; `array_layers(a)` the layers of an uploaded texture array (`None`:
/// not uploaded or removed); `lit` says which layers 2D lighting applies
/// to; `view` culls tile chunks (orthographic cameras). A sprite or
/// particle drawn from an array ([`RenderScene::sprite_arrays`]) joins its
/// array's run, and the glow pass by array when it glows.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build<'a>(
    scene: &RenderScene,
    font: impl Fn(usize) -> Option<&'a FontLayout>,
    texture_count: usize,
    array_layers: impl Fn(TextureArrayHandle) -> Option<u32>,
    extra: &[ExtraSprite],
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
    let arrays = array_entries(
        &scene.sprite_arrays,
        scene.sprites.len(),
        "sprite",
        &array_layers,
    )?;
    // Drop shadows by sprite index (the last entry wins), under a sun.
    let mut shadows: HashMap<u32, (f32, crate::ShadowGround)> = HashMap::new();
    for s in &scene.sprite_shadows {
        if s.sprite as usize >= scene.sprites.len() {
            return Err(RenderError::InvalidHandle(format!(
                "sprite_shadows: sprite {} of {}",
                s.sprite,
                scene.sprites.len()
            )));
        }
        shadows.insert(s.sprite, (s.opacity.clamp(0.0, 1.0), s.ground));
    }
    let ground = shadow_ground(scene.light.direction_to_light).filter(|_| !shadows.is_empty());
    let mut shadow_count = 0u32;
    let outlines: HashMap<u32, &crate::TextOutline> =
        scene.text_outlines.iter().map(|o| (o.text, o)).collect();
    // (layer, key, source, untonemapped, instance), in submission order.
    let mut keyed: Vec<(i32, f32, RunSource, bool, SpriteInstance)> =
        Vec::with_capacity(scene.sprites.len() + extra.len());
    let mut glowing: Vec<(RunSource, SpriteInstance)> = Vec::new();
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
        let (flash, glow, blend, tone_mapped) = match found {
            Some(f) => (
                f.flash,
                f.glow.max(0.0),
                f.blend,
                // Additive light is always the scene's.
                f.tone_mapped || f.blend == SpriteBlend::Additive,
            ),
            None => ([1.0, 1.0, 1.0, 0.0], 0.0, SpriteBlend::Alpha, true),
        };
        let mut inst = sprite_instance(s, flash, glow, lit(s.layer), false);
        let key = order.get(&(i as u32)).copied().unwrap_or(0.0);
        // Its drop shadow: the same quad and texel alpha in black, projected
        // onto the ground, before every sprite of the layer.
        if let (Some(g), Some(&(opacity, on))) = (ground, shadows.get(&(i as u32)))
            && opacity > 0.0
        {
            let mut shadow = sprite_instance(s, [1.0, 1.0, 1.0, 0.0], 0.0, false, false);
            shadow.color = [0.0, 0.0, 0.0, opacity * s.color[3]];
            let height = if on.height.is_finite() {
                on.height
            } else {
                0.0
            };
            shadow.ground = [g[0], g[1], g[2], g[3] + height];
            let [rise_x, rise_z] = on.slope();
            shadow.fx = [0.0, rise_x, rise_z];
            let source = match arrays.get(&(i as u32)) {
                Some(&(array, layer)) => {
                    shadow.layer = layer;
                    RunSource::Array(SpriteBlend::Alpha, array)
                }
                None => RunSource::Sprite(SpriteBlend::Alpha, s.texture),
            };
            match on.bend {
                // Bent over the ground: one instance per cell of the quad.
                Some(bend) => {
                    for cell in bent_shadow(shadow, &bend, g[3]) {
                        keyed.push((s.layer, f32::MIN, source, false, cell));
                        shadow_count += 1;
                    }
                }
                None => {
                    keyed.push((s.layer, f32::MIN, source, false, shadow));
                    shadow_count += 1;
                }
            }
        }
        if let Some(&(array, layer)) = arrays.get(&(i as u32)) {
            inst.layer = layer;
            if glow > 0.0 {
                glowing.push((RunSource::Array(SpriteBlend::Additive, array), inst));
            }
            keyed.push((
                s.layer,
                key,
                RunSource::Array(blend, array),
                !tone_mapped,
                inst,
            ));
            continue;
        }
        if glow > 0.0 {
            glowing.push((RunSource::Sprite(SpriteBlend::Additive, s.texture), inst));
        }
        keyed.push((
            s.layer,
            key,
            RunSource::Sprite(blend, s.texture),
            !tone_mapped,
            inst,
        ));
    }
    for e in extra {
        let s = &e.sprite;
        let mut inst = sprite_instance(s, [1.0, 1.0, 1.0, 0.0], e.glow, lit(s.layer), e.soft);
        let array = match e.array {
            Some((array, layer)) => {
                let layers = array_layers(array).ok_or_else(|| {
                    RenderError::InvalidHandle(format!(
                        "a 2D particle's texture array {}",
                        array.index()
                    ))
                })?;
                if layer >= layers {
                    return Err(RenderError::InvalidData(format!(
                        "a 2D particle: layer {layer} of a texture array of {layers}"
                    )));
                }
                inst.layer = layer;
                Some(array)
            }
            None => None,
        };
        let run = |b: SpriteBlend| match array {
            Some(a) => RunSource::Array(b, a),
            None => RunSource::Sprite(b, s.texture),
        };
        if e.glow > 0.0 {
            glowing.push((run(SpriteBlend::Additive), inst));
        }
        keyed.push((s.layer, 0.0, run(e.blend), false, inst));
    }
    let sprites = keyed.len() as u32 - shadow_count;
    let after_sprites = keyed.len() as u32;
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
                    false,
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
                        ground: NO_GROUND,
                        layer: 0,
                    },
                ));
            }
        });
    }
    let glyphs = keyed.len() as u32 - after_sprites;
    // Stable: within a (layer, key, source, tone) run, submission order is kept.
    keyed.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then(a.1.total_cmp(&b.1))
            .then(a.2.cmp(&b.2))
            .then(a.3.cmp(&b.3))
    });
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
    let push = |runs: &mut Vec<SpriteRun>,
                source: RunSource,
                tone_mapped: bool,
                first: u32,
                count: u32| {
        if count == 0 {
            return;
        }
        match runs.last_mut() {
            Some(run)
                if run.source == source
                    && run.tone_mapped == tone_mapped
                    && !matches!(source, RunSource::Strip(..) | RunSource::Metaballs(_))
                    && run.first + run.count == first =>
            {
                run.count += count
            }
            _ => runs.push(SpriteRun {
                source,
                first,
                count,
                tone_mapped,
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
                // Lit as the sprites of its layer are, unless it opts out.
                let (q, c) = tile_instances(layer, view, layer.lit && lit(layer.layer), instances);
                tiles += q;
                tile_chunks += c;
                push(
                    runs,
                    RunSource::Sprite(SpriteBlend::Alpha, Some(layer.texture)),
                    true,
                    first,
                    q,
                );
            } else {
                let s = &strips[i];
                push(runs, s.source, true, s.first, s.count);
            }
            *next_under += 1;
        }
    };
    for (layer, _, source, untonemapped, inst) in keyed {
        place_under(layer, &mut instances, &mut runs, &mut next_under);
        let index = instances.len() as u32;
        instances.push(inst);
        push(&mut runs, source, !untonemapped, index, 1);
    }
    place_under(i32::MAX, &mut instances, &mut runs, &mut next_under);
    // The glow pass: by texture or array (additive, so order does not
    // matter; a stable sort keeps the plain textures' order as before).
    glowing.sort_by_key(|(source, _)| *source);
    let mut glow_runs: Vec<SpriteRun> = Vec::new();
    for (source, inst) in glowing {
        let index = instances.len() as u32;
        match glow_runs.last_mut() {
            Some(run) if run.source == source => run.count += 1,
            _ => glow_runs.push(SpriteRun {
                source,
                first: index,
                count: 1,
                tone_mapped: true,
            }),
        }
        instances.push(inst);
    }
    let untonemapped = runs.iter().any(|r| !r.tone_mapped);
    Ok(SpriteFrame {
        instances,
        runs,
        untonemapped,
        glow_runs,
        sprites,
        glyphs,
        missing_glyphs,
        tiles,
        tile_chunks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SpriteShadow;
    use glam::Vec3;

    fn frame(scene: &RenderScene) -> SpriteFrame {
        build(scene, |_| None, 0, |_| None, &[], |_| false, None, &[]).expect("a frame")
    }

    #[test]
    fn a_drop_shadow_draws_first_in_its_layer_projected_along_the_sun() {
        let mut scene = RenderScene::new();
        scene.light.direction_to_light = Vec3::new(1.0, 2.0, -0.5);
        let sprite = |x: f32, layer: i32| Sprite {
            position: Vec3::new(x, 0.0, 0.0),
            size: glam::Vec2::new(1.0, 2.0),
            anchor: glam::Vec2::new(0.5, 0.0),
            color: [1.0, 1.0, 1.0, 0.5],
            layer,
            ..Default::default()
        };
        scene.sprites = vec![sprite(0.0, 1), sprite(2.0, 1), sprite(4.0, 0)];
        scene.sprite_shadows = vec![SpriteShadow {
            sprite: 1,
            opacity: 0.8,
            ..Default::default()
        }];
        let f = frame(&scene);
        assert_eq!(f.sprites, 3, "a shadow is not a sprite of the frame");
        // Layer 0's sprite, then layer 1: the shadow before both sprites.
        let xs: Vec<f32> = f.instances.iter().map(|i| i.position_rotation[0]).collect();
        assert_eq!(xs, [4.0, 2.0, 0.0, 2.0]);
        let shadow = &f.instances[1];
        assert_eq!(shadow.color, [0.0, 0.0, 0.0, 0.4]);
        // Per metre of height: -x/y and -z/y of the direction to the sun.
        assert_eq!(shadow.ground, [1.0, -0.5, 0.25, crate::SPRITE_SHADOW_LIFT]);
        assert!(f.instances.iter().filter(|i| i.ground[0] > 0.0).count() == 1);
        // No sun above the horizon, no shadow; an index past the sprites is refused.
        scene.light.direction_to_light = Vec3::new(0.0, -1.0, 0.0);
        assert_eq!(frame(&scene).instances.len(), 3);
        scene.sprite_shadows[0].sprite = 3;
        assert!(build(&scene, |_| None, 0, |_| None, &[], |_| false, None, &[]).is_err());
    }

    #[test]
    fn a_drop_shadow_lies_on_its_ground_at_its_height_and_slope() {
        use crate::ShadowGround;
        let mut scene = RenderScene::new();
        scene.light.direction_to_light = Vec3::new(1.0, 2.0, -0.5);
        scene.sprites = vec![
            Sprite {
                position: Vec3::new(0.0, 2.0, 0.0),
                size: glam::Vec2::new(1.0, 2.0),
                ..Default::default()
            };
            2
        ];
        scene.sprite_shadows = vec![
            SpriteShadow {
                sprite: 0,
                opacity: 1.0,
                ground: ShadowGround::level(2.0),
            },
            SpriteShadow {
                sprite: 1,
                opacity: 1.0,
                ground: ShadowGround {
                    height: 1.5,
                    normal: Vec3::new(-0.5, 1.0, 0.25).normalize(),
                    bend: None,
                },
            },
        ];
        let f = frame(&scene);
        let shadows: Vec<_> = f.instances.iter().filter(|i| i.ground[0] > 0.0).collect();
        assert_eq!(shadows.len(), 2);
        // A level ground at 2 m: no slope, the lift over it.
        assert_eq!(shadows[0].ground[3], 2.0 + crate::SPRITE_SHADOW_LIFT);
        assert_eq!(shadows[0].fx, [0.0, 0.0, 0.0]);
        // A slope: the rise per metre along x and z, carried in fx.yz.
        assert_eq!(shadows[1].ground[3], 1.5 + crate::SPRITE_SHADOW_LIFT);
        let [_, rx, rz] = shadows[1].fx;
        assert!(
            (rx - 0.5).abs() < 1e-5 && (rz + 0.25).abs() < 1e-5,
            "{rx} {rz}"
        );
        // A cliff is taken at about 70 degrees; a normal pointing down is level.
        let cliff = ShadowGround {
            height: 0.0,
            normal: Vec3::X,
            bend: None,
        };
        assert_eq!(cliff.slope(), [0.0, 0.0]);
        let steep = ShadowGround {
            height: 0.0,
            normal: Vec3::new(-1.0, 0.1, 0.0),
            bend: None,
        };
        assert!(
            (2.5..3.0).contains(&steep.slope()[0]),
            "{:?}",
            steep.slope()
        );
    }

    #[test]
    fn a_fitted_ground_follows_the_terrain_under_the_shadow() {
        use crate::ShadowGround;
        let sun = Vec3::new(0.0, 1.0, -1.0);
        let sprite = Sprite {
            position: Vec3::new(1.0, 0.0, 2.0),
            size: glam::Vec2::new(1.0, 2.0),
            ..Default::default()
        };
        // A plane rising 0.5 per metre along x and falling 0.25 along z.
        let plane = |x: f32, z: f32| 1.0 + 0.5 * x - 0.25 * z;
        let g = ShadowGround::fit(plane, &sprite, sun);
        assert!((g.height - plane(1.0, 2.0)).abs() < 1e-5, "{g:?}");
        let [rx, rz] = g.slope();
        assert!(
            (rx - 0.5).abs() < 1e-4 && (rz + 0.25).abs() < 1e-4,
            "{rx} {rz}"
        );
        // A hill under the middle of the shadow lifts the plane over it.
        let hill = |_x: f32, z: f32| if (z - 3.0).abs() < 0.5 { 0.3 } else { 0.0 };
        let g = ShadowGround::fit(hill, &sprite, sun);
        assert!((g.height - 0.3).abs() < 1e-5, "{g:?}");
        // No sun above the horizon: level at the anchor's ground.
        let g = ShadowGround::fit(plane, &sprite, Vec3::NEG_Y);
        assert_eq!(g, ShadowGround::level(plane(1.0, 2.0)));
    }

    #[test]
    fn a_shadow_bends_over_a_hollow_narrower_than_itself() {
        use crate::{SHADOW_BEND_COLUMNS, SHADOW_BEND_ROWS, ShadowGround};
        // The sun behind, the shadow falling 1 m along +z per metre of
        // height: a 2 m sprite's shadow runs from z 2 to z 4.
        let sun = Vec3::new(0.0, 1.0, -1.0);
        let sprite = Sprite {
            position: Vec3::new(1.0, 0.0, 2.0),
            size: glam::Vec2::new(1.0, 2.0),
            anchor: glam::Vec2::new(0.5, 0.0),
            ..Default::default()
        };
        // Level ground: the plane, no bend.
        let g = ShadowGround::fit(|_, _| 0.0, &sprite, sun);
        assert_eq!(g, ShadowGround::level(0.0));
        // A pit 0.5 m deep under the middle of the shadow, 0.6 m across.
        let pit = |_x: f32, z: f32| if (z - 3.0).abs() < 0.3 { -0.5 } else { 0.0 };
        let g = ShadowGround::fit(pit, &sprite, sun);
        let bend = g.bend.expect("a pit narrower than the shadow bends it");
        // Row 2 is the quad's middle, 1 m up: its ray enters the pit at
        // z 3 and meets the far wall 0.3 m down (z 3.3); the base and the
        // tip land on the level ground.
        for column in 0..SHADOW_BEND_COLUMNS {
            assert_eq!(bend.heights[0][column], 0.0);
            assert!((bend.heights[2][column] + 0.3).abs() < 1e-3, "{bend:?}");
            assert!(bend.heights[SHADOW_BEND_ROWS - 1][column].abs() < 1e-3);
        }
        // Drawn as eight cells of the quad, each corner at its height.
        let mut scene = RenderScene::new();
        scene.light.direction_to_light = sun;
        scene.sprites = vec![sprite];
        scene.sprite_shadows = vec![SpriteShadow {
            sprite: 0,
            opacity: 0.5,
            ground: g,
        }];
        let f = frame(&scene);
        assert_eq!(f.sprites, 1);
        let cells: Vec<_> = f.instances.iter().filter(|i| i.ground[0] > 1.5).collect();
        assert_eq!(cells.len(), 8, "4 x 2 cells");
        let lift = crate::SPRITE_SHADOW_LIFT;
        // The cell in row 1, column 0: its top corners are row 2's.
        let cell = cells
            .iter()
            .find(|c| c.flash[..2] == [0, 1])
            .expect("row 1, column 0");
        assert_eq!(cell.ground[3], bend.heights[1][0] + lift);
        assert_eq!(
            cell.fx,
            [
                bend.heights[1][1] + lift,
                bend.heights[2][1] + lift,
                bend.heights[2][0] + lift,
            ]
        );
        // A ridge is bent over too, not only a pit.
        let ridge = |_x: f32, z: f32| if (z - 3.5).abs() < 0.2 { 0.4 } else { 0.0 };
        assert!(ShadowGround::fit(ridge, &sprite, sun).bend.is_some());
    }

    #[test]
    fn a_low_sun_casts_at_most_five_heights() {
        let g = shadow_ground(Vec3::new(1.0, 0.01, 0.0)).unwrap();
        assert!((g[1].abs() - 5.0).abs() < 0.01, "{g:?}");
        assert_eq!(shadow_ground(Vec3::new(1.0, 0.0, 0.0)), None);
    }
}
