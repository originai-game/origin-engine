//! The 2D batch: sprites and MSDF text as instanced quads, sorted by
//! (layer, kind, texture) so each run of one texture (or one font) in one
//! layer is a single instanced draw.

use crate::RenderError;
use crate::effects::SpriteBlend;
use crate::scene::SpriteFacing;
use crate::scene::{
    FontHandle, Glyph, MsdfFont, RenderScene, Sprite, Text, TextureArrayHandle, TextureHandle,
    TileLayer,
};
use glam::{Vec2, Vec3};
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
    /// How the quad stands (`FACING_*`): the xy plane, a billboard facing
    /// the camera that draws it, or upright facing the sun (a faced
    /// sprite's drop shadow).
    facing: u32,
}

/// [`SpriteInstance::facing`]: the world's xy plane facing +z.
const FACING_NONE: u32 = 0;
/// The camera's view plane ([`crate::SpriteFacing::Camera`]).
const FACING_CAMERA: u32 = 1;
/// Upright, turned about y towards the camera ([`crate::SpriteFacing::CameraY`]).
const FACING_CAMERA_Y: u32 = 2;
/// Upright, turned about y to face the sun: a faced sprite's drop shadow
/// ([`sun_card_right`]).
const FACING_SUN: u32 = 3;

fn facing_code(f: SpriteFacing) -> u32 {
    match f {
        SpriteFacing::None => FACING_NONE,
        SpriteFacing::Camera => FACING_CAMERA,
        SpriteFacing::CameraY => FACING_CAMERA_Y,
    }
}

/// A faced sprite's facing ([`RenderScene::sprite_facing`]; past its end,
/// none).
pub(crate) fn facing_of(list: &[SpriteFacing], i: usize) -> SpriteFacing {
    list.get(i).copied().unwrap_or_default()
}

/// The world axes a sprite's quad x and y run along when it faces a camera
/// whose right and up are `right` and `up` (sprite.wgsl `face_axes`, which
/// the GPU computes the same way): the xy plane, the view plane, or
/// upright with x the camera's right laid level (+x when the camera's right
/// points straight up or down).
pub(crate) fn face_axes(facing: SpriteFacing, right: Vec3, up: Vec3) -> (Vec3, Vec3) {
    match facing {
        SpriteFacing::None => (Vec3::X, Vec3::Y),
        SpriteFacing::Camera => (right, up),
        SpriteFacing::CameraY => (level(Vec3::new(right.x, 0.0, right.z)), Vec3::Y),
    }
}

/// A horizontal direction normalised, +x when it has no length.
fn level(h: Vec3) -> Vec3 {
    if h.length_squared() > 1e-8 {
        h.normalize()
    } else {
        Vec3::X
    }
}

/// The x axis of the upright quad that casts a faced sprite's drop
/// shadow: the card turned about y to face the sun towards
/// `direction_to_light` (the sun as a camera's right), so the shadow is
/// the sprite's whole shape from every camera; +x with the sun overhead.
pub fn sun_card_right(direction_to_light: Vec3) -> Vec3 {
    let l = direction_to_light;
    level(Vec3::new(l.z, 0.0, -l.x))
}

/// The eye a frame's sprites are built for: faced sprites sort back to
/// front from it (by view depth along `forward`, or by distance without
/// one: a reflection probe's six faces) and are culled to `frustum` when
/// there is one.
pub(crate) struct SpriteEye<'a> {
    pub position: Vec3,
    pub forward: Option<Vec3>,
    pub frustum: Option<&'a crate::frame::Frustum>,
}

impl SpriteEye<'_> {
    fn depth(&self, p: Vec3) -> f32 {
        match self.forward {
            Some(f) => (p - self.position).dot(f),
            None => (p - self.position).length(),
        }
    }
}

/// A sprite's bounds: the sphere round its anchor through its farthest
/// corner, whatever way it is turned.
fn sprite_radius(s: &Sprite) -> f32 {
    let far = glam::Vec2::new(
        s.anchor.x.max(1.0 - s.anchor.x),
        s.anchor.y.max(1.0 - s.anchor.y),
    );
    (far * s.size).length()
}

/// An instance that stands where it is drawn.
const NO_GROUND: [f32; 4] = [0.0; 4];

pub(crate) const SPRITE_STRIDE: u64 = std::mem::size_of::<SpriteInstance>() as u64;

/// The instance as the sprite pipelines read it: every field, the layer
/// (location 7) read by the array pipelines alone.
pub(crate) const SPRITE_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: SPRITE_STRIDE,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4, 4 => Unorm8x4, 5 => Float32x3, 6 => Float32x4, 7 => Uint32, 8 => Uint32],
};

/// [`SPRITE_LAYOUT`], for the array sprite pipelines.
pub(crate) const SPRITE_ARRAY_LAYOUT: wgpu::VertexBufferLayout<'static> = SPRITE_LAYOUT;

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
    /// [`RenderScene::sprite_meshes`]' mesh `n`: `first`/`count` are
    /// corners of the mesh buffer (crate::sprite_mesh).
    Mesh(u32),
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
    /// Returns each line's width (em): where the pen ends it.
    pub fn walk(
        &self,
        text: &Text,
        line_step: f32,
        mut f: impl FnMut(char, Option<&Glyph>, Vec2, Vec2),
    ) -> Vec<f32> {
        let mut pen = Vec2::ZERO;
        let mut prev: Option<char> = None;
        let mut widths = Vec::new();
        for (i, ch) in text.text.chars().enumerate() {
            if ch == '\n' {
                widths.push(pen.x);
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
        widths.push(pen.x);
        widths
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
    /// Faced sprites left out because their bounds miss the eye's frustum
    /// ([`RenderScene::sprite_facing`]).
    pub culled: u32,
    /// Billboards drawn among the transparent meshes rather than in the
    /// layer order: faced sprites and faced text without a key of their
    /// own, tone mapped, far to near by the eye's depth (the transparent
    /// meshes' sort key), each a run of one instance with that depth
    /// ([`draw_runs_of`] merges neighbours of one source).
    pub billboards: Vec<(f32, SpriteRun)>,
    /// Those billboards' drop shadows, drawn before the transparent meshes
    /// (the shadows lie on the opaque ground, and drawn after a billboard,
    /// which writes no depth, they would darken it).
    pub billboard_shadows: Vec<SpriteRun>,
}

/// A pass's billboards and their drop shadows
/// ([`SpriteFrame::billboards`], [`SpriteFrame::billboard_shadows`]).
pub(crate) type Billboards = (Vec<(f32, SpriteRun)>, Vec<SpriteRun>);

/// `frame`'s billboards with their instances moved to `base` in the buffer.
pub(crate) fn billboards_at(frame: &SpriteFrame, base: u32) -> Billboards {
    let at = |r: &SpriteRun| SpriteRun {
        source: r.source,
        first: r.first + base,
        count: r.count,
        tone_mapped: r.tone_mapped,
    };
    (
        frame.billboards.iter().map(|(d, r)| (*d, at(r))).collect(),
        frame.billboard_shadows.iter().map(at).collect(),
    )
}

/// The billboards' depths, far first (`Renderer::sort_transparent`).
pub(crate) fn depths(billboards: &[(f32, SpriteRun)]) -> Vec<f32> {
    billboards.iter().map(|b| b.0).collect()
}

/// Billboards of [`SpriteFrame::billboards`] as draw runs: neighbours of
/// one source in one run.
pub(crate) fn draw_runs_of(billboards: &[(f32, SpriteRun)]) -> Vec<SpriteRun> {
    let mut runs: Vec<SpriteRun> = Vec::new();
    for (_, r) in billboards {
        match runs.last_mut() {
            Some(last) if last.source == r.source && last.first + last.count == r.first => {
                last.count += r.count
            }
            _ => runs.push(SpriteRun {
                source: r.source,
                first: r.first,
                count: r.count,
                tone_mapped: r.tone_mapped,
            }),
        }
    }
    runs
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
        facing: FACING_NONE,
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
    /// Its system's three.js layer mask ([`RenderScene::particle_layers`]).
    pub layers: u32,
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
                    facing: FACING_NONE,
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
/// array's run, and the glow pass by array when it glows. `mask` is the
/// camera's three.js layer mask: a world sprite or 2D particle whose own
/// ([`RenderScene::sprite_layers`], [`ExtraSprite::layers`]) shares no bit
/// with it is left out, with its shadow and glow (`views.rs`). A faced
/// sprite ([`RenderScene::sprite_facing`]) is keyed back to front from
/// `eye` unless it has a key of its own, and left out (its glow too, not
/// its drop shadow) when its bounds miss the eye's frustum.
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
    mask: u32,
    eye: &SpriteEye,
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
    let mut culled = 0u32;
    let outlines: HashMap<u32, &crate::TextOutline> =
        scene.text_outlines.iter().map(|o| (o.text, o)).collect();
    let text_facing: HashMap<u32, &crate::TextFacing> =
        scene.text_facing.iter().map(|t| (t.text, t)).collect();
    // (layer, key, source, untonemapped, instance), in submission order.
    let mut keyed: Vec<(i32, f32, RunSource, bool, SpriteInstance)> =
        Vec::with_capacity(scene.sprites.len() + extra.len());
    let mut glowing: Vec<(RunSource, SpriteInstance)> = Vec::new();
    // Billboards (depth, layer, source, instance) and their drop shadows.
    let mut boards: Vec<(f32, i32, RunSource, SpriteInstance)> = Vec::new();
    let mut board_shadows: Vec<(RunSource, SpriteInstance)> = Vec::new();
    for (i, s) in scene.sprites.iter().enumerate() {
        if let Some(t) = s.texture
            && t.index() >= texture_count
        {
            return Err(RenderError::InvalidHandle(format!(
                "sprite {i}: texture {}",
                t.index()
            )));
        }
        if crate::views::layer_of(&scene.sprite_layers, i) & mask == 0 {
            continue;
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
        let facing = facing_of(&scene.sprite_facing, i);
        inst.facing = facing_code(facing);
        let faced = facing != SpriteFacing::None;
        // A billboard of its own depth sorts with the transparent meshes
        // (`billboards`); with a key, or untonemapped (its cover pass is
        // the sprite order's), it keeps the layer order.
        let board = faced && tone_mapped && !order.contains_key(&(i as u32));
        // A billboard sorts back to front from the eye within its layer.
        let key = match order.get(&(i as u32)) {
            Some(&k) => k,
            None if faced => -eye.depth(s.position),
            None => 0.0,
        };
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
            if faced {
                shadow.facing = FACING_SUN;
            }
            let source = match arrays.get(&(i as u32)) {
                Some(&(array, layer)) => {
                    shadow.layer = layer;
                    RunSource::Array(SpriteBlend::Alpha, array)
                }
                None => RunSource::Sprite(SpriteBlend::Alpha, s.texture),
            };
            let cells = match on.bend {
                // Bent over the ground: one instance per cell of the quad.
                Some(bend) => bent_shadow(shadow, &bend, g[3]).collect(),
                None => vec![shadow],
            };
            for cell in cells {
                if board {
                    board_shadows.push((source, cell));
                } else {
                    keyed.push((s.layer, f32::MIN, source, false, cell));
                    shadow_count += 1;
                }
            }
        }
        if faced
            && let Some(frustum) = eye.frustum
            && !frustum.intersects_sphere(s.position, sprite_radius(s))
        {
            culled += 1;
            continue;
        }
        if let Some(&(array, layer)) = arrays.get(&(i as u32)) {
            inst.layer = layer;
            if glow > 0.0 {
                glowing.push((RunSource::Array(SpriteBlend::Additive, array), inst));
            }
            if board {
                boards.push((
                    eye.depth(s.position),
                    s.layer,
                    RunSource::Array(blend, array),
                    inst,
                ));
                continue;
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
        if board {
            boards.push((
                eye.depth(s.position),
                s.layer,
                RunSource::Sprite(blend, s.texture),
                inst,
            ));
            continue;
        }
        keyed.push((
            s.layer,
            key,
            RunSource::Sprite(blend, s.texture),
            !tone_mapped,
            inst,
        ));
    }
    let sprite_boards = boards.len() as u32;
    for e in extra.iter().filter(|e| e.layers & mask != 0) {
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
    // The sprite meshes, sorted as sprites are; each is a run of its own
    // corners (`mesh_ranges`), not an instance.
    let mesh_ranges = crate::sprite_mesh::ranges(scene, texture_count)?;
    let mesh_count = scene.sprite_meshes.len() as u32;
    for (i, m) in scene.sprite_meshes.iter().enumerate() {
        let key = if m.key.is_finite() { m.key } else { 0.0 };
        let placeholder =
            sprite_instance(&Sprite::default(), [1.0, 1.0, 1.0, 0.0], 0.0, false, false);
        keyed.push((m.layer, key, RunSource::Mesh(i as u32), false, placeholder));
    }
    let sprites = keyed.len() as u32 - shadow_count - mesh_count + sprite_boards;
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
        // Faced text (`text_facing`) turns as one panel about its pivot:
        // every glyph anchored there, its corner the anchor's offset (each
        // line moved back by `align` of its width); a billboard sorting
        // with the transparent meshes.
        let (facing, align) = text_facing
            .get(&(i as u32))
            .map_or((SpriteFacing::None, 0.0), |t| (t.facing, t.align));
        let widths = if facing != SpriteFacing::None && align != 0.0 {
            font.walk(text, -font.line_height, |_, _, _, _| {})
        } else {
            Vec::new()
        };
        let depth = eye.depth(text.position);
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
                let size = [(r - l) * text.size, (t - b) * text.size];
                let mut inst = SpriteInstance {
                    position_rotation: [
                        origin.x + rotate(corner).x,
                        origin.y + rotate(corner).y,
                        text.position.z,
                        text.rotation,
                    ],
                    size_anchor: [size[0], size[1], 0.0, 0.0],
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
                    facing: FACING_NONE,
                };
                if facing == SpriteFacing::None {
                    keyed.push((text.layer, 0.0, RunSource::Font(text.font), false, inst));
                } else {
                    let line = (pen.y / -font.line_height).round() as usize;
                    let back = widths.get(line).map_or(0.0, |w| w * align * text.size);
                    let corner = corner - Vec2::new(back, 0.0);
                    inst.position_rotation[0] = origin.x;
                    inst.position_rotation[1] = origin.y;
                    inst.size_anchor[2] = -corner.x / size[0];
                    inst.size_anchor[3] = -corner.y / size[1];
                    inst.facing = facing_code(facing);
                    boards.push((depth, text.layer, RunSource::Font(text.font), inst));
                }
            }
        });
    }
    let glyphs = keyed.len() as u32 - after_sprites + boards.len() as u32 - sprite_boards;
    // Stable: within a (layer, key, source, tone) run, submission order is kept.
    og_core::sort::sort_by(&mut keyed, |a, b| {
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
    og_core::sort::sort_by_key(&mut under, |u| (u.0, u.1));
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
        if let RunSource::Mesh(i) = source {
            let (first, count) = mesh_ranges[i as usize];
            push(&mut runs, source, true, first, count);
            continue;
        }
        let index = instances.len() as u32;
        instances.push(inst);
        push(&mut runs, source, !untonemapped, index, 1);
    }
    place_under(i32::MAX, &mut instances, &mut runs, &mut next_under);
    // The billboards' shadows, then the billboards far to near (ties by
    // layer, then as submitted), after the layer runs in the buffer.
    let mut billboard_shadows: Vec<SpriteRun> = Vec::new();
    for (source, inst) in board_shadows {
        let index = instances.len() as u32;
        instances.push(inst);
        push(&mut billboard_shadows, source, true, index, 1);
    }
    og_core::sort::sort_by(&mut boards, |a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut billboards: Vec<(f32, SpriteRun)> = Vec::with_capacity(boards.len());
    for (depth, _, source, inst) in boards {
        billboards.push((
            depth,
            SpriteRun {
                source,
                first: instances.len() as u32,
                count: 1,
                tone_mapped: true,
            },
        ));
        instances.push(inst);
    }
    // The glow pass: by texture or array (additive, so order does not
    // matter; a stable sort keeps the plain textures' order as before).
    og_core::sort::sort_by_key(&mut glowing, |(source, _)| *source);
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
        culled,
        billboards,
        billboard_shadows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SpriteShadow;
    use glam::Vec3;

    /// An eye at the origin with no frustum, sorting by distance.
    const NO_EYE: SpriteEye<'static> = SpriteEye {
        position: Vec3::ZERO,
        forward: None,
        frustum: None,
    };

    #[test]
    fn faced_sprites_sort_back_to_front_cull_to_the_frustum_and_cast_sun_facing_shadows() {
        let mut scene = RenderScene::new();
        scene.light.direction_to_light = Vec3::new(1.0, 1.0, 0.0);
        let at = |z: f32, layer: i32| Sprite {
            position: Vec3::new(0.0, 0.0, z),
            layer,
            ..Default::default()
        };
        // Submitted near to far, a flat one between, one far out to the side.
        scene.sprites = vec![
            at(-2.0, 0),
            at(-6.0, 0),
            at(-4.0, 0),
            at(-9.0, 0),
            at(-3.0, 0),
        ];
        scene.sprites[3].position.x = 500.0;
        scene.sprite_facing = vec![
            SpriteFacing::CameraY,
            SpriteFacing::Camera,
            SpriteFacing::None,
            SpriteFacing::Camera,
            SpriteFacing::Camera,
        ];
        // Its own key wins over the depth.
        scene.sprite_order = vec![crate::SpriteOrder {
            sprite: 4,
            key: 5.0,
        }];
        scene.sprite_shadows = vec![crate::SpriteShadow {
            sprite: 0,
            opacity: 1.0,
            ..Default::default()
        }];
        let camera = crate::frame::camera_frame(&crate::Camera::default(), 16.0 / 9.0, Vec2::ZERO);
        let frustum = crate::frame::Frustum::from_view_proj(camera.view_proj);
        let eye = SpriteEye {
            position: crate::Camera::default().position,
            forward: Some(camera.forward),
            frustum: Some(&frustum),
        };
        let f = build(
            &scene,
            |_| None,
            0,
            |_| None,
            &[],
            |_| false,
            None,
            &[],
            1,
            &eye,
        )
        .expect("a frame");
        assert_eq!(
            (f.sprites, f.culled),
            (4, 1),
            "the one off to the side is culled"
        );
        // In the layer order the key-0 flat sprite, then the keyed
        // billboard (its key keeps it there); the others sort with the
        // transparent meshes: their shadow, then they far to near, each
        // with its view depth.
        let zs = |runs: &mut dyn Iterator<Item = &SpriteRun>| -> Vec<(f32, u32)> {
            runs.flat_map(|r| &f.instances[r.first as usize..(r.first + r.count) as usize])
                .map(|i| (i.position_rotation[2], i.facing))
                .collect()
        };
        assert_eq!(
            zs(&mut f.runs.iter()),
            [(-4.0, FACING_NONE), (-3.0, FACING_CAMERA)]
        );
        assert_eq!(zs(&mut f.billboard_shadows.iter()), [(-2.0, FACING_SUN)]);
        assert_eq!(
            zs(&mut f.billboards.iter().map(|b| &b.1)),
            [(-6.0, FACING_CAMERA), (-2.0, FACING_CAMERA_Y)]
        );
        let depth = |z: f32| (Vec3::new(0.0, 0.0, z) - eye.position).dot(camera.forward);
        let depths: Vec<f32> = f.billboards.iter().map(|b| b.0).collect();
        assert_eq!(depths, [depth(-6.0), depth(-2.0)], "view depths, far first");
        // By distance without a forward (a probe's faces), still far first.
        let f = build(
            &scene,
            |_| None,
            0,
            |_| None,
            &[],
            |_| false,
            None,
            &[],
            1,
            &NO_EYE,
        )
        .expect("a frame");
        assert_eq!(f.culled, 0, "no frustum, nothing culled");
        let first = f.billboards[0].1.first as usize;
        assert_eq!(f.instances[first].position_rotation[0], 500.0);
        // The sun-facing card: the sun as a camera's right.
        assert!((sun_card_right(Vec3::new(1.0, 1.0, 0.0)) - Vec3::NEG_Z).length() < 1e-6);
        assert_eq!(sun_card_right(Vec3::Y), Vec3::X, "overhead: +x");
        // A camera looking straight down keeps the upright card's x level.
        let (x, y) = face_axes(SpriteFacing::CameraY, Vec3::new(0.6, 0.8, 0.0), Vec3::Z);
        assert!((x - Vec3::X).length() < 1e-6 && y == Vec3::Y);
    }

    #[test]
    fn a_sprite_anchored_elsewhere_keeps_its_corners() {
        // A nine-slice piece off the panel's anchor, turned with the panel.
        let piece = Sprite {
            position: Vec3::new(2.0, 1.0, -3.0),
            size: Vec2::new(0.5, 2.0),
            anchor: Vec2::new(0.0, 1.0),
            rotation: 0.7,
            ..Default::default()
        };
        let corners = |s: &Sprite| {
            let (sin, cos) = s.rotation.sin_cos();
            [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)].map(|(u, v)| {
                let l = (Vec2::new(u, v) - s.anchor) * s.size;
                s.position.truncate() + Vec2::new(l.x * cos - l.y * sin, l.x * sin + l.y * cos)
            })
        };
        let moved = piece.anchored_at(Vec3::new(-1.0, 4.0, 9.0));
        assert_eq!(moved.position, Vec3::new(-1.0, 4.0, -3.0), "its z is kept");
        for (a, b) in corners(&piece).iter().zip(corners(&moved)) {
            assert!((*a - b).length() < 1e-5, "{a} moved to {b}");
        }
    }

    fn frame(scene: &RenderScene) -> SpriteFrame {
        build(
            scene,
            |_| None,
            0,
            |_| None,
            &[],
            |_| false,
            None,
            &[],
            1,
            &NO_EYE,
        )
        .expect("a frame")
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
        assert!(
            build(
                &scene,
                |_| None,
                0,
                |_| None,
                &[],
                |_| false,
                None,
                &[],
                1,
                &NO_EYE
            )
            .is_err()
        );
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
    fn a_sprite_mesh_is_one_run_in_its_layers_sprite_order() {
        use crate::{SpriteMesh, SpriteMeshVertex};
        let mut scene = RenderScene::new();
        let at = |x: f32, layer: i32| Sprite {
            position: Vec3::new(x, 0.0, 0.0),
            layer,
            ..Default::default()
        };
        scene.sprites = vec![at(0.0, 2), at(1.0, 0), at(2.0, 1)];
        let corner = SpriteMeshVertex {
            color: [1.0; 4],
            ..Default::default()
        };
        // Layer 1, after the layer's sprites (key 1 > their 0); two triangles.
        scene.sprite_meshes = vec![SpriteMesh {
            vertices: vec![corner; 4],
            indices: vec![0, 1, 2, 2, 3, 0],
            layer: 1,
            key: 1.0,
            ..Default::default()
        }];
        let f = frame(&scene);
        assert_eq!(f.sprites, 3, "a mesh is not a sprite");
        assert_eq!(f.instances.len(), 3, "nor an instance");
        // Layers 0 and 1's sprites share a texture and batch into one run;
        // the mesh follows them and breaks the run before layer 2's sprite.
        let kinds: Vec<_> = f.runs.iter().map(|r| r.source).collect();
        assert_eq!(kinds.len(), 3, "{kinds:?}");
        assert_eq!((f.runs[0].first, f.runs[0].count), (0, 2));
        let xs: Vec<f32> = f.instances.iter().map(|i| i.position_rotation[0]).collect();
        assert_eq!(xs, [1.0, 2.0, 0.0]);
        assert_eq!(
            kinds[1],
            RunSource::Mesh(0),
            "between layer 1's sprite and layer 2's"
        );
        assert_eq!((f.runs[1].first, f.runs[1].count), (0, 6));
        assert_eq!((f.runs[2].first, f.runs[2].count), (2, 1));
        // An index past its vertices is refused.
        scene.sprite_meshes[0].indices[5] = 9;
        assert!(
            build(
                &scene,
                |_| None,
                0,
                |_| None,
                &[],
                |_| false,
                None,
                &[],
                1,
                &NO_EYE
            )
            .is_err()
        );
    }

    #[test]
    fn a_low_sun_casts_at_most_five_heights() {
        let g = shadow_ground(Vec3::new(1.0, 0.01, 0.0)).unwrap();
        assert!((g[1].abs() - 5.0).abs() < 0.01, "{g:?}");
        assert_eq!(shadow_ground(Vec3::new(1.0, 0.0, 0.0)), None);
    }
}
