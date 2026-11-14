//! The 2D batch: sprites and MSDF text as instanced quads, sorted by
//! (layer, kind, texture) so each run of one texture (or one font) in one
//! layer is a single instanced draw.

pub(crate) mod light2d;
pub(crate) mod sprite_array;
pub(crate) mod sprite_hooks;
pub(crate) mod sprite_mesh;
pub(crate) mod sprite_normals;
pub(crate) mod sprite_skin;
pub(crate) mod tiles_gpu;

use crate::RenderError;
use crate::effects::SpriteBlend;
use crate::scene::SpriteFacing;
use crate::scene::{
    FontHandle, Glyph, GlyphKey, MsdfFont, RenderScene, Sprite, Text, TextureArrayHandle,
    TextureHandle, TileLayer,
};
use glam::{Vec2, Vec3};
use std::collections::HashMap;

mod build;

pub(crate) use build::*;

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
    /// A pixel outline ([`crate::SpriteOutline`]): xy one logical pixel in
    /// uv units along u and v, z the width in logical pixels (0: none, the
    /// quad as it is), w unused.
    outline: [f32; 4],
    /// The outline's linear colour and alpha.
    outline_color: [u8; 4],
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

/// An instance without a pixel outline.
const NO_OUTLINE: [f32; 4] = [0.0; 4];

pub(crate) const SPRITE_STRIDE: u64 = std::mem::size_of::<SpriteInstance>() as u64;

/// The instance as the sprite pipelines read it: every field, the layer
/// (location 7) read by the array pipelines alone.
pub(crate) const SPRITE_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: SPRITE_STRIDE,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4, 4 => Unorm8x4, 5 => Float32x3, 6 => Float32x4, 7 => Uint32, 8 => Uint32, 9 => Float32x4, 10 => Unorm8x4],
};

/// [`SPRITE_LAYOUT`], for the array sprite pipelines.
pub(crate) const SPRITE_ARRAY_LAYOUT: wgpu::VertexBufferLayout<'static> = SPRITE_LAYOUT;

/// What one run draws with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum RunSource {
    /// A sprite texture (`None`: solid white) and its blend.
    Sprite(SpriteBlend, Option<TextureHandle>),
    /// As `Sprite`, drawn with a material's shader hooks
    /// ([`crate::SpriteShading`], `sprite_hooks.rs`).
    Shaded(SpriteBlend, Option<TextureHandle>, crate::MaterialHandle),
    /// A sprite-mode particle system simulated in compute: range `n` of
    /// [`GpuSprites`], whose instances `particles_sprites.wgsl` writes into
    /// its own buffer (`first`/`count` index that buffer); drawn as the
    /// range's source draws.
    Particles(u32),
    Font(FontHandle),
    /// A 2D trail's strip: `first`/`count` are vertices of the ribbon
    /// buffer, or strip points when the last field is true
    /// ([`crate::ribbons::RibbonRun::points`]); the blend is
    /// `ParticleBlend as u8`.
    Strip(u8, Option<TextureHandle>, bool),
    /// A 2D fluid's metaball composite (the fluid's index this frame).
    Metaballs(u8),
    /// Sprites drawn from layers of a texture array, and their blend.
    Array(SpriteBlend, TextureArrayHandle),
    /// [`RenderScene::sprite_meshes`]' mesh `n`: `first`/`count` are
    /// corners of the mesh buffer (crate::sprite_mesh).
    Mesh(u32),
    /// A persistent tile chunk and its per-view placement.
    Tiles(u32),
}

/// A ribbon run drawn in the sprite order at `layer` (a 2D trail, a 2D
/// water body, a fluid's metaball composite).
pub(crate) struct LayeredStrip {
    pub layer: i32,
    pub source: RunSource,
    pub first: u32,
    pub count: u32,
}

/// The instance ranges a frame's builds take for the sprite-mode systems
/// simulated in compute ([`crate::particles_gpu::SpriteSystem`]): one per
/// system and light map a build draws it with, shared by the frame, the
/// views and the probe faces; the compute pass writes each.
#[derive(Default)]
pub(crate) struct GpuSprites {
    pub ranges: Vec<GpuSpriteRange>,
    /// Instances in all of them.
    pub total: u32,
    /// GPU tile draws across the main frame, views and probe faces.
    /// None retains CPU tile instances on devices without vertex storage.
    pub tiles: Option<Vec<crate::tiles_gpu::TileDraw>>,
}

/// One range of [`GpuSprites`].
pub(crate) struct GpuSpriteRange {
    /// The system's index in the frame's `sprite_systems`.
    pub system: usize,
    /// The light map its instances take (`lit` in [`build`]).
    pub lit: f32,
    pub first: u32,
    pub count: u32,
    /// What its runs draw with: a plain or an array sprite run.
    pub source: RunSource,
}

impl GpuSprites {
    /// The range of `system` lit by `lit`, taken on first use.
    fn take(&mut self, system: usize, s: &crate::particles_gpu::SpriteSystem, lit: f32) -> u32 {
        if let Some(k) = self
            .ranges
            .iter()
            .position(|r| r.system == system && r.lit.to_bits() == lit.to_bits())
        {
            return k as u32;
        }
        self.ranges.push(GpuSpriteRange {
            system,
            lit,
            first: self.total,
            count: s.max,
            source: match s.array {
                Some(a) => RunSource::Array(s.blend, a),
                None => RunSource::Sprite(s.blend, s.texture),
            },
        });
        self.total += s.max;
        self.ranges.len() as u32 - 1
    }
}

/// Consecutive instances drawn with one bind group and one pipeline.
#[derive(Clone, Copy)]
pub(crate) struct SpriteRun {
    pub source: RunSource,
    pub first: u32,
    pub count: u32,
    /// False: alpha-blended sprites kept out of exposure, bloom and the
    /// tone curve ([`crate::SpriteFx::tone_mapped`]), drawn with the
    /// untonemapped pipelines.
    pub tone_mapped: bool,
}

/// The glyph table of one font, by key, and its kerning pairs.
pub(crate) struct FontLayout {
    pub glyphs: HashMap<GlyphKey, Glyph>,
    pub kerning: HashMap<(char, char), f32>,
    pub line_height: f32,
    pub atlas_size: Vec2,
    /// A page of colour glyphs ([`MsdfFont::distance_range`] 0).
    pub color: bool,
    /// Its glyphs use the text's colour ([`MsdfFont::foreground`]).
    pub foreground: bool,
    /// A colour glyph silhouette page (MsdfFont::silhouette): effects only.
    pub silhouette: bool,
}

impl FontLayout {
    pub fn new(font: &MsdfFont, atlas_size: Vec2) -> FontLayout {
        FontLayout {
            glyphs: font.glyphs.iter().map(|g| (g.key, *g)).collect(),
            kerning: font
                .kerning
                .iter()
                .map(|k| ((k.left, k.right), k.adjust))
                .collect(),
            line_height: font.line_height,
            atlas_size,
            color: font.distance_range == 0.0,
            foreground: font.foreground,
            silhouette: font.silhouette,
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
            if let Some(p) = text.placements.get(i) {
                let key = p.glyph.map_or(GlyphKey::Char(ch), GlyphKey::Id);
                f(ch, self.glyphs.get(&key), pen, Vec2::new(p.x, p.y));
                pen.x += p.advance;
                continue;
            }
            let Some(g) = self.glyphs.get(&GlyphKey::Char(ch)) else {
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

fn sprite_instance(s: &Sprite, flash: [f32; 4], glow: f32, lit: f32, soft: bool) -> SpriteInstance {
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
        fx: [glow, lit, soft as u32 as f32],
        ground: NO_GROUND,
        layer: 0,
        facing: FACING_NONE,
        outline: NO_OUTLINE,
        outline_color: [0; 4],
    }
}

/// Gives `inst` the pixel outline `o` for a texture of `size` texels
/// (none without a texture, or at width 0).
fn outline(inst: &mut SpriteInstance, o: &crate::SpriteOutline, size: Option<(u32, u32)>) {
    let Some((w, h)) = size else {
        return;
    };
    let width = o.width.min(crate::MAX_SPRITE_OUTLINE);
    if width.is_nan() || width <= 0.0 || w == 0 || h == 0 {
        return;
    }
    let pixel = if o.pixel > 0.0 { o.pixel } else { 1.0 };
    inst.outline = [pixel / w as f32, pixel / h as f32, width, 0.0];
    inst.outline_color = o.color.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
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

#[cfg(test)]
mod tests;

/// The batch's modules for [`crate::shader_sources`]: a label and its WGSL.
pub(crate) fn shader_sources(add: &mut dyn FnMut(&str, String)) {
    add("sprite", crate::sprite_source());
    // A sprite drawn with a material shader: every hook, parameters and
    // the sprite-only `sprite_texel` (sprite_hooks.rs).
    let example = crate::sprite_hooks::example();
    add("sprite hooks", crate::sprite_hooks::source(&example));
    // Without vertex_offset: the built-in one fills in.
    let colour_only = crate::MaterialShader {
        code: example
            .code
            .split("fn surface_color")
            .nth(1)
            .map_or_else(String::new, |rest| format!("fn surface_color{rest}")),
        ..example
    };
    add(
        "sprite hooks (colour only)",
        crate::sprite_hooks::source(&colour_only),
    );
    add("sprite array", crate::sprite_array::source());
    add("sprite mesh", crate::sprite_mesh::source());
    add("sprite skin", crate::sprite_skin::source());
    add("tiles", crate::tiles_gpu::source());
    add(
        "overlay array",
        crate::sprite_array::OVERLAY_ARRAY_WGSL.to_string(),
    );
    add("lights 2d", crate::light2d::LIGHT2D_WGSL.to_string());
    add(
        "lights 2d shadows",
        crate::light2d::LIGHT2D_SHADOW_WGSL.to_string(),
    );
}
