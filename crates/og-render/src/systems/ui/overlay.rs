//! The overlay pass ([`crate::Overlay`]): screen-space sprites, MSDF text,
//! glass and the UI pass's shapes ([`crate::ui`]) drawn after every post
//! effect into the output, with per-instance clip rectangles (and clip
//! shapes). Sorted like the world's 2D batch (layer, kind, texture,
//! texture array or font; submission order within a run), one instanced
//! draw per run. A UI layer's range of layers sorts into a target of its
//! own, drawn before the overlay and composited by its shapes.

use crate::RenderError;
use crate::deferred::Deferred;
use crate::scene::{FontHandle, Overlay, TextureArrayHandle, TextureHandle};
use crate::sprites::FontLayout;
use crate::ui::build::{self as ub, ShapePipe, ShapeSource, UiGpu, UiTables};
use crate::ui::{UiAlign, UiLayer, UiVectorData, UiVectorHandle};
use glam::Vec2;
use std::collections::HashMap;

/// `struct OverlayIn` in overlay.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct OverlayInstance {
    position_rotation: [f32; 4],
    size_anchor: [f32; 4],
    uv_rect: [f32; 4],
    color: [f32; 4],
    clip: [f32; 4],
    /// MSDF text: x the glow's radius in CSS px, y the weight (CSS px of
    /// thickening), z the pass (0 fill, 1 glow, 2 shadow, 3 outline under
    /// the fill, 4 outline over it), w unused. Zero for sprites and plain
    /// text; a sprite drawn from a texture array has its layer in w
    /// (`shaders/overlay_array.wgsl`).
    effect: [f32; 4],
    /// The glow's, shadow's or outline's linear RGBA.
    effect_color: [f32; 4],
    /// MSDF text: x the outline inside, y the outline outside or the
    /// shadow's softness (CSS px), z the fill's gradient + 1.
    effect2: [f32; 4],
    /// The clip's shape + 1 (0: none).
    clip_shape: u32,
    /// MSDF text under a map: what is left of it after its rotation, an
    /// upper triangle `[x scale − 1, skew, y scale − 1]` applied to the
    /// glyph's quad before it turns (zero: none; sprites and glass).
    shear: [f32; 3],
}

pub(crate) const OVERLAY_STRIDE: u64 = std::mem::size_of::<OverlayInstance>() as u64;

pub(crate) const OVERLAY_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: OVERLAY_STRIDE,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Uint32, 9 => Float32x3],
};

/// What one overlay run draws with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum OverlaySource {
    /// A sprite texture (`None`: solid white).
    Sprite(Option<TextureHandle>),
    Font(FontHandle),
    /// The overlay's glass panels (the blur pyramid, src/glass.rs).
    Glass,
    Array(TextureArrayHandle),
    /// UI shapes ([`crate::ui`]): instances of the UI buffer.
    Ui(ShapeSource),
    /// A cooked vector's mesh, instanced from the UI buffer.
    Vector(UiVectorHandle),
    /// [`crate::ui::UiList::mass`]' batch `n`: mass elements, instanced
    /// from the frame's mass buffer (`shaders/ui_mass.wgsl`).
    Mass(u32),
}

impl OverlaySource {
    /// Whether its instances are the UI buffer's.
    pub fn is_ui(self) -> bool {
        matches!(self, OverlaySource::Ui(_) | OverlaySource::Vector(_))
    }

    /// Reads the blur pyramid of what is under it.
    pub fn needs_pyramid(self) -> bool {
        matches!(
            self,
            OverlaySource::Glass | OverlaySource::Ui(ShapeSource::Backdrop)
        )
    }

    /// Reads a copy of the target as drawn so far.
    pub fn needs_copy(self) -> bool {
        matches!(
            self,
            OverlaySource::Ui(
                ShapeSource::Shape(ShapePipe::Destination, _)
                    | ShapeSource::Composite(_, ShapePipe::Destination)
            )
        )
    }
}

/// Consecutive instances of one target drawn with one pipeline and group.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OverlayRun {
    /// 0: the output; k + 1: [`crate::ui::UiList::layers`]' layer k.
    pub target: u32,
    pub source: OverlaySource,
    /// Into the overlay instances, or the UI instances ([`OverlaySource::is_ui`]).
    pub first: u32,
    pub count: u32,
}

/// A UI layer's target this frame: its rectangle snapped out to device
/// pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LayerPlan {
    pub index: u32,
    /// The top left, CSS px (a device pixel's corner).
    pub origin: [f32; 2],
    /// Device pixels.
    pub size: (u32, u32),
    /// The snapped rectangle, CSS px.
    pub rect: [f32; 4],
    /// Device pixels per CSS px (a world panel's: its target's over its rect).
    pub scale: f32,
    /// A world panel's render target (its texture index), drawn instead of
    /// a target of the UI pass's own.
    pub target: Option<usize>,
    /// Where it composites (0: the output; k + 1: layer k), or for a layer
    /// only a filter reads ([`crate::ui::UiFilterOp::Layer`]), that
    /// filter's layer's.
    pub parent: u32,
}

/// The overlay's sorted instances and runs for one frame.
pub(crate) struct OverlayFrame {
    pub instances: Vec<OverlayInstance>,
    pub ui_instances: Vec<UiGpu>,
    pub tables: UiTables,
    /// By target, then layer.
    pub runs: Vec<OverlayRun>,
    /// The layers to draw, each before the target that composites it.
    pub layers: Vec<LayerPlan>,
    /// The gradient of each layer's gradient mask, by layer.
    pub layer_masks: HashMap<u32, u32>,
    /// The backdrop layers drawn ([`crate::ui::UiLayer::backdrop`]).
    pub backdrops: Vec<u32>,
    pub sprites: u32,
    pub glyphs: u32,
    pub missing_glyphs: u32,
    /// Mass elements: each batch's template, their glyph tables and the
    /// instances, in run order.
    pub mass_templates: Vec<crate::ui::mass::TemplateGpu>,
    pub mass_table: Vec<[f32; 4]>,
    pub mass_instances: Vec<crate::ui::UiMassInstance>,
}

impl OverlayFrame {
    /// The output's runs.
    pub fn main_runs(&self) -> &[OverlayRun] {
        let end = self.runs.partition_point(|r| r.target == 0);
        &self.runs[..end]
    }

    /// Layer `k`'s runs.
    pub fn layer_runs(&self, k: u32) -> &[OverlayRun] {
        let lo = self.runs.partition_point(|r| r.target < k + 1);
        let hi = self.runs.partition_point(|r| r.target < k + 2);
        &self.runs[lo..hi]
    }

    /// Whether the output's runs read what is under them (glass, a
    /// backdrop, a blend that reads the destination, a backdrop layer's
    /// capture).
    pub fn reads_backdrop(&self) -> bool {
        self.main_runs()
            .iter()
            .any(|r| r.source.needs_pyramid() || r.source.needs_copy() || self.capture(r).is_some())
    }

    /// The backdrop layer whose composite `run` is: what is under it is
    /// captured before it draws.
    pub fn capture(&self, run: &OverlayRun) -> Option<u32> {
        match run.source {
            OverlaySource::Ui(ShapeSource::Composite(k, _)) if self.backdrops.contains(&k) => {
                Some(k)
            }
            _ => None,
        }
    }
}

/// No clip: far outside any output.
const UNCLIPPED: [f32; 4] = ub::UNCLIPPED;

fn clip_rect(
    overlay: &Overlay,
    heads: &[u32],
    clip: Option<u32>,
    what: &str,
) -> Result<([f32; 4], u32), RenderError> {
    let Some(i) = clip else {
        return Ok((UNCLIPPED, 0));
    };
    let [x, y, w, h] = *overlay.clips.get(i as usize).ok_or_else(|| {
        RenderError::InvalidHandle(format!(
            "overlay {what}: clip {i} of {}",
            overlay.clips.len()
        ))
    })?;
    Ok((
        [x, y, x + w.max(0.0), y + h.max(0.0)],
        heads.get(i as usize).copied().unwrap_or(0),
    ))
}

/// The innermost UI layer whose capture holds `layer` (+ 1; 0: none).
fn target_of(layer: i32, layers: &[UiLayer]) -> u32 {
    let mut best: Option<(i64, u32)> = None;
    for (k, l) in layers.iter().enumerate() {
        let [lo, hi] = l.capture;
        if lo <= layer && layer <= hi {
            let span = hi as i64 - lo as i64;
            if best.is_none_or(|(s, _)| span < s) {
                best = Some((span, k as u32 + 1));
            }
        }
    }
    best.map_or(0, |(_, t)| t)
}

/// Where the overlay's frame goes: the device pixels per CSS pixel and the
/// output's size (a layer's target is snapped to its pixels).
#[derive(Clone, Copy, Debug)]
pub(crate) struct OverlayTarget {
    pub scale: f32,
    pub width: u32,
    pub height: u32,
}

/// Sorts an overlay's sprites, texts, glass and UI into runs (the world
/// batch's order). `array_layers(a)`: the layers of an uploaded texture
/// array (`None`: not uploaded or removed). A sprite drawn from an array
/// ([`Overlay::sprite_arrays`]) joins its array's run.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build<'a>(
    overlay: &Overlay,
    font: impl Fn(usize) -> Option<&'a FontLayout>,
    texture_count: usize,
    array_layers: impl Fn(TextureArrayHandle) -> Option<u32>,
    vector: impl Fn(UiVectorHandle) -> Option<&'a UiVectorData>,
    panel_target: impl Fn(crate::TextureHandle) -> Option<crate::TargetTexture>,
    target: OverlayTarget,
    mass_eases: &mut crate::ui::mass::EaseTables,
) -> Result<OverlayFrame, RenderError> {
    let ui = &overlay.ui;
    let arrays = crate::sprites::array_entries(
        &overlay.sprite_arrays,
        overlay.sprites.len(),
        "overlay sprite",
        array_layers,
    )?;
    let mut tables = UiTables::default();
    let mut heads = ub::clip_heads(ui, overlay.clips.len(), &mut tables)?;
    // Occluders: a depth test chained in front of their rectangle's shape.
    let css = Vec2::new(overlay.width.max(1.0), overlay.height.max(1.0));
    let css = if overlay.scale > 0.0 {
        css
    } else {
        // Fitted to the output's height: its width follows the output's.
        Vec2::new(target.width as f32 / target.scale.max(1e-6), css.y)
    };
    for (i, o) in ui.occluders.iter().enumerate() {
        let slot = heads.get_mut(o.rect as usize).ok_or_else(|| {
            RenderError::InvalidHandle(format!(
                "ui occluder {i}: clip rectangle {} of {}",
                o.rect,
                overlay.clips.len()
            ))
        })?;
        *slot = tables.occluder(o, css, *slot);
    }
    let layers = &ui.layers;
    // (target, layer, source, index into `ov` or `uv`)
    let mut keyed: Vec<(u32, i32, OverlaySource, u32)> =
        Vec::with_capacity(overlay.sprites.len() + overlay.texts.len() * 8 + ui.shapes.len());
    let mut ov: Vec<OverlayInstance> = Vec::with_capacity(overlay.sprites.len());
    let mut uv: Vec<UiGpu> = Vec::with_capacity(ui.shapes.len());
    let mut push_ov = |keyed: &mut Vec<_>, layer: i32, source, inst: OverlayInstance| {
        keyed.push((target_of(layer, layers), layer, source, ov.len() as u32));
        ov.push(inst);
    };
    for (i, s) in overlay.sprites.iter().enumerate() {
        let (clip, shape) = clip_rect(overlay, &heads, s.clip, &format!("sprite {i}"))?;
        let mut inst = OverlayInstance {
            position_rotation: [s.position.x, s.position.y, 0.0, s.rotation],
            size_anchor: [s.size.x, s.size.y, s.anchor.x, s.anchor.y],
            uv_rect: s.uv_rect,
            color: s.color,
            clip,
            effect: [0.0; 4],
            effect_color: [0.0; 4],
            effect2: [0.0; 4],
            clip_shape: shape,
            shear: [0.0; 3],
        };
        if let Some(&(array, layer)) = arrays.get(&(i as u32)) {
            inst.effect[3] = layer as f32;
            push_ov(&mut keyed, s.layer, OverlaySource::Array(array), inst);
            continue;
        }
        if let Some(t) = s.texture
            && t.index() >= texture_count
        {
            return Err(RenderError::InvalidHandle(format!(
                "overlay sprite {i}: texture {}",
                t.index()
            )));
        }
        push_ov(&mut keyed, s.layer, OverlaySource::Sprite(s.texture), inst);
    }
    for (i, g) in overlay.glass.iter().enumerate() {
        let (clip, shape) = clip_rect(overlay, &heads, g.clip, &format!("glass {i}"))?;
        push_ov(
            &mut keyed,
            g.layer,
            OverlaySource::Glass,
            OverlayInstance {
                position_rotation: [g.position.x, g.position.y, 0.0, 0.0],
                size_anchor: [g.size.x, g.size.y, 0.0, 0.0],
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                color: g.tint,
                clip,
                effect: [g.radius.max(0.0), g.blur.max(0.0), g.size.x, g.size.y],
                effect_color: [0.0, 0.0, 0.0, g.alpha.clamp(0.0, 1.0)],
                effect2: [0.0; 4],
                clip_shape: shape,
                shear: [0.0; 3],
            },
        );
    }
    let mut sprites = keyed.len() as u32;
    // UI shapes, and the polylines of vectors' strokes.
    let add_shape = |keyed: &mut Vec<_>,
                     uv: &mut Vec<UiGpu>,
                     tables: &mut UiTables,
                     i: usize,
                     s: &crate::ui::UiShape,
                     abs_box: Option<[f32; 4]>|
     -> Result<(), RenderError> {
        let clip = clip_rect(overlay, &heads, s.clip, &format!("ui shape {i}"))?;
        let (mut g, source) = ub::shape(i, s, clip, tables, texture_count, layers.len())?;
        if let (Some(b), crate::ui::UiPaint::Gradient(gr)) = (abs_box, &s.paint) {
            g.extra[1] = tables.gradient(gr, Some(b)) as f32;
        }
        keyed.push((
            target_of(s.layer, layers),
            s.layer,
            OverlaySource::Ui(source),
            uv.len() as u32,
        ));
        uv.push(g);
        Ok(())
    };
    for (i, s) in ui.shapes.iter().enumerate() {
        add_shape(&mut keyed, &mut uv, &mut tables, i, s, None)?;
    }
    for (i, d) in ui.vectors.iter().enumerate() {
        let data = vector(d.vector).ok_or_else(|| {
            RenderError::InvalidHandle(format!("ui vector draw {i}: vector {}", d.vector.0))
        })?;
        if !data.indices.is_empty() {
            let base = tables.gradients.len() as u32;
            for p in &data.paints {
                match p {
                    crate::ui::UiVectorPaint::Solid(c) => tables.solid(*c),
                    crate::ui::UiVectorPaint::Gradient(g) => {
                        tables.gradient(&ub::vector_gradient(g, data.view_box), None)
                    }
                };
            }
            if data.paints.is_empty() {
                tables.solid([0.0, 0.0, 0.0, 1.0]);
            }
            let (clip, shape) = clip_rect(overlay, &heads, d.clip, &format!("ui vector {i}"))?;
            let [x, y, w, h] = d.rect;
            let mut g = UiGpu {
                center_half: [x + 0.5 * w, y + 0.5 * h, 0.5 * w.max(0.0), 0.5 * h.max(0.0)],
                xform: d.transform,
                geo: data.view_box,
                color: d.tint,
                clip,
                extra: [1.0, base as f32, 0.0, d.layer as f32],
                ..Default::default()
            };
            g.kinds[3] = shape;
            keyed.push((
                target_of(d.layer, layers),
                d.layer,
                OverlaySource::Vector(d.vector),
                uv.len() as u32,
            ));
            uv.push(g);
        }
        for (s, b) in ub::vector_strokes(d, data) {
            add_shape(&mut keyed, &mut uv, &mut tables, i, &s, Some(b))?;
        }
    }
    sprites += uv.len() as u32;
    // Mass elements: one run per batch.
    let mut mass_templates = Vec::with_capacity(ui.mass.len());
    let mut mass_table = Vec::new();
    mass_eases.begin_frame();
    for (k, b) in ui.mass.iter().enumerate() {
        let t = &b.template;
        let font_layout = match (&t.kind, t.font) {
            (
                crate::ui::UiMassKind::Number { .. } | crate::ui::UiMassKind::Text { .. },
                Some(f),
            ) => Some(font(f.index()).ok_or_else(|| {
                RenderError::InvalidHandle(format!("ui mass batch {k}: font {}", f.index()))
            })?),
            _ => None,
        };
        if let Some(tex) = t.texture
            && tex.index() >= texture_count
        {
            return Err(RenderError::InvalidHandle(format!(
                "ui mass batch {k}: texture {}",
                tex.index()
            )));
        }
        let clip = clip_rect(overlay, &heads, b.clip, &format!("ui mass batch {k}"))?;
        mass_templates.push(crate::ui::mass::template_gpu(
            b,
            clip,
            font_layout,
            &mut mass_table,
            mass_eases,
        ));
        if !b.instances.is_empty() && k < crate::ui::UI_MASS_BATCHES {
            keyed.push((
                target_of(b.layer, layers),
                b.layer,
                OverlaySource::Mass(k as u32),
                k as u32,
            ));
        }
    }
    mass_eases.end_frame();
    let glyphs_from = keyed.len();
    let effects: HashMap<u32, &crate::ui::UiTextEffects> =
        ui.text_effects.iter().map(|e| (e.text, e)).collect();
    let mut missing_glyphs = 0;
    for (i, text) in overlay.texts.iter().enumerate() {
        let layout = font(text.font.index()).ok_or_else(|| {
            RenderError::InvalidHandle(format!("overlay text {i}: font {}", text.font.index()))
        })?;
        let (clip, shape) = clip_rect(overlay, &heads, text.clip, &format!("text {i}"))?;
        let fx = effects.get(&(i as u32)).copied();
        // The glyphs' map: the effects' transform after the text's rotation
        // (y down: a positive rotation turns clockwise on screen), split
        // into the turn the quad takes and the shear before it.
        let (sin, cos) = text.rotation.sin_cos();
        let map = ub::transform_of(fx.map_or(ub::IDENTITY, |f| f.transform));
        let [ma, mb, mc, md] = map;
        let [a, b, c, d] = [
            ma * cos + mc * sin,
            mb * cos + md * sin,
            -ma * sin + mc * cos,
            -mb * sin + md * cos,
        ];
        let rotate = |v: Vec2| Vec2::new(a * v.x + c * v.y, b * v.x + d * v.y);
        let ux = (a * a + b * b).sqrt().max(1e-6);
        let turn = b.atan2(a);
        let shear = [ux - 1.0, (a * c + b * d) / ux, (a * d - b * c) / ux - 1.0];
        // Widths in CSS px (weight, glow, outline, shadow softness) scale
        // with the map's area.
        let k = (ma * md - mb * mc).abs().sqrt();
        let origin = text.position.truncate();
        let glow = text
            .glow
            .filter(|g| g.radius > 0.0 && g.color[3] > 0.0)
            .map(|g| crate::scene::TextGlow {
                radius: g.radius * k,
                ..g
            });
        let weight = text.weight.max(0.0) * k;
        // The passes in drawing order: every glyph's shadows, then glows,
        // then an outline under the fill, the fill, an outline over it; so
        // no glyph's shadow or glow lies over its neighbour's ink.
        // (pass code, colour, effect2, offset)
        let mut passes: Vec<(f32, [f32; 4], [f32; 4], Vec2)> = Vec::new();
        if let Some(fx) = fx {
            for s in &fx.shadows {
                if s.color[3] > 0.0 {
                    let o = Vec2::new(
                        ma * s.offset.x + mc * s.offset.y,
                        mb * s.offset.x + md * s.offset.y,
                    );
                    passes.push((2.0, s.color, [0.0, s.blur.max(0.0) * k, 0.0, 0.0], o));
                }
            }
        }
        if let Some(g) = glow {
            passes.push((1.0, g.color, [0.0; 4], Vec2::ZERO));
        }
        let outline = fx
            .and_then(|f| f.outline)
            .filter(|o| o.width > 0.0)
            .map(|o| crate::ui::UiTextOutline {
                width: o.width * k,
                ..o
            });
        if let Some(o) = outline.filter(|o| o.align == UiAlign::Outside) {
            passes.push((3.0, o.color, [0.0, o.width, 0.0, 0.0], Vec2::ZERO));
        }
        let fill_gradient = fx
            .and_then(|f| f.fill.as_ref().map(|g| (g, f.fill_box)))
            .map(|(g, b)| tables.gradient(g, Some(b)) as f32 + 1.0)
            .unwrap_or(0.0);
        passes.push((0.0, [0.0; 4], [0.0, 0.0, fill_gradient, 0.0], Vec2::ZERO));
        if let Some(o) = outline.filter(|o| o.align != UiAlign::Outside) {
            let (inner, outer) = match o.align {
                UiAlign::Inside => (o.width, 0.0),
                _ => (0.5 * o.width, 0.5 * o.width),
            };
            passes.push((4.0, o.color, [inner, outer, 0.0, 0.0], Vec2::ZERO));
        }
        for &(pass, pass_color, effect2, offset) in &passes {
            // A page of colour glyphs draws its images once, untinted but
            // for the text's alpha, no gradient; their shadows, glow and
            // outline come from their silhouette page, which draws no fill.
            if (layout.color && pass != 0.0) || (layout.silhouette && pass == 0.0) {
                continue;
            }
            // The pen in em, y down from the first baseline.
            layout.walk(text, layout.line_height, |ch, g, pen, off| {
                let Some(g) = g else {
                    if !ch.is_whitespace() && pass == 0.0 {
                        missing_glyphs += 1;
                    }
                    return;
                };
                let [l, b, r, t] = g.plane_bounds;
                if r > l && t > b {
                    // plane bounds are em y up from the baseline: the glyph's top
                    // left is (l, -t) in y-down em; a placement's y is up too.
                    let corner =
                        rotate((pen + Vec2::new(off.x, -off.y) + Vec2::new(l, -t)) * text.size);
                    let [al, at, ar, ab] = g.atlas_bounds;
                    let glow_radius = glow.map_or(0.0, |gl| gl.radius);
                    let at_layer = text.layer;
                    keyed.push((
                        target_of(at_layer, layers),
                        at_layer,
                        OverlaySource::Font(text.font),
                        ov.len() as u32,
                    ));
                    ov.push(OverlayInstance {
                        position_rotation: [
                            origin.x + corner.x + offset.x,
                            origin.y + corner.y + offset.y,
                            0.0,
                            turn,
                        ],
                        size_anchor: [(r - l) * text.size, (t - b) * text.size, 0.0, 0.0],
                        uv_rect: [
                            al / layout.atlas_size.x,
                            at / layout.atlas_size.y,
                            ar / layout.atlas_size.x,
                            ab / layout.atlas_size.y,
                        ],
                        // A colour page's images keep their colours (those
                        // in the text's colour mix it in, foreground_mix).
                        color: if layout.color && !layout.foreground {
                            [1.0, 1.0, 1.0, text.color[3]]
                        } else {
                            text.color
                        },
                        clip,
                        effect: if layout.color {
                            [0.0; 4]
                        } else {
                            [glow_radius, weight, pass, 0.0]
                        },
                        effect_color: pass_color,
                        effect2: if layout.color { [0.0; 4] } else { effect2 },
                        clip_shape: shape,
                        shear,
                    });
                }
            });
        }
    }
    let glyphs = (keyed.len() - glyphs_from) as u32;
    let mut layer_masks = HashMap::new();
    for (k, l) in layers.iter().enumerate() {
        if let Some(crate::ui::UiMask::Gradient(g)) = &l.mask {
            layer_masks.insert(k as u32, tables.gradient(g, None));
        }
    }
    // Layers: snapped to device pixels, drawn innermost first.
    let mut plans = Vec::new();
    let composite_target = |k: u32| {
        keyed
            .iter()
            .find(|(_, _, s, _)| matches!(s, OverlaySource::Ui(ShapeSource::Composite(j, _)) if *j == k))
            .map(|(t, ..)| *t)
    };
    // A layer a filter reads (an element's feImage) and composites
    // nowhere draws before that filter's layer, as if inside it.
    let read_by = |k: u32| {
        layers.iter().position(|l| {
            l.filter.as_ref().is_some_and(|f| {
                f.primitives
                    .iter()
                    .any(|p| matches!(p.op, crate::ui::UiFilterOp::Layer { layer } if layer == k))
            })
        })
    };
    let owner = |k: u32| {
        composite_target(k).or_else(|| read_by(k).filter(|&j| j as u32 != k).map(|j| j as u32 + 1))
    };
    let mut depth = vec![0u32; layers.len()];
    for k in 0..layers.len() as u32 {
        let mut t = owner(k);
        let mut d = 0;
        while let Some(parent) = t.filter(|&t| t > 0) {
            d += 1;
            if d > layers.len() as u32 {
                return Err(RenderError::InvalidData(format!(
                    "ui layer {k}: composited inside its own capture"
                )));
            }
            t = owner(parent - 1);
        }
        let parent = owner(k).unwrap_or(0);
        depth[k as usize] = d;
        let [x, y, w, h] = layers[k as usize].rect;
        if let Some(t) = layers[k as usize].target {
            // A world panel: its rect over the whole of its target.
            let tt = panel_target(t)
                .filter(|tt| tt.kind == crate::TargetKind::Color(crate::TargetFormat::Rgba16Float))
                .ok_or_else(|| {
                    RenderError::InvalidHandle(format!(
                        "ui layer {k}: target texture {} is not an rgba16float render target",
                        t.index()
                    ))
                })?;
            if w > 0.0 && h > 0.0 {
                plans.push(LayerPlan {
                    index: k,
                    origin: [x, y],
                    size: (tt.width, tt.height),
                    rect: [x, y, w, h],
                    scale: tt.width as f32 / w,
                    target: Some(t.index()),
                    parent,
                });
            }
            continue;
        }
        if let Some(g) = &layers[k as usize].ground {
            // Drawn wherever its rect is, at the size it is seen at: the
            // longest edge's stretch over the rect's, in device pixels,
            // rounded up to a power of two of the screen's scale (a camera
            // that moves resizes it seldom) and capped; at the cap while a
            // corner is behind the camera.
            if w > 0.0 && h > 0.0 && composite_target(k).is_some() {
                let stretch = g.corners.map_or(f32::INFINITY, |q| {
                    ((q[1] - q[0]).length().max((q[2] - q[3]).length()) / w)
                        .max((q[3] - q[0]).length().max((q[2] - q[1]).length()) / h)
                });
                let dpr = target.scale.max(1e-6);
                let cap = crate::ui::UI_QUAD_LAYER_MAX as f32 / w.max(h);
                let mut p = 1.0f32;
                while p < stretch && p < 64.0 {
                    p *= 2.0;
                }
                while p / 2.0 >= stretch && p > 1.0 / 64.0 {
                    p /= 2.0;
                }
                let s = (dpr * p).min(cap).max(1.0 / w.max(h));
                plans.push(LayerPlan {
                    index: k,
                    origin: [x, y],
                    size: (
                        ((w * s).ceil() as u32).max(1),
                        ((h * s).ceil() as u32).max(1),
                    ),
                    rect: [x, y, w, h],
                    scale: s,
                    target: None,
                    parent,
                });
            }
            continue;
        }
        // Its pixel grid and extent: the screen's, or inside a world
        // panel's or a ground's layer (at any depth) that layer's, where
        // what it draws lands (a backdrop there reads that layer).
        let mut root = owner(k).filter(|&t| t > 0);
        let grid = loop {
            let Some(t) = root else {
                break (
                    [0.0; 2],
                    target.scale.max(1e-6),
                    [target.width, target.height],
                );
            };
            let j = t - 1;
            let l = &layers[j as usize];
            if l.target.is_some() || l.ground.is_some() {
                if let Some(p) = plans.iter().find(|p: &&LayerPlan| p.index == j) {
                    break (p.origin, p.scale, [p.size.0, p.size.1]);
                }
                // Not planned (yet): the screen's.
                root = None;
                continue;
            }
            root = owner(j).filter(|&t| t > 0);
        };
        let ([ox, oy], s, [gw, gh]) = grid;
        let x0 = ((x - ox) * s).floor().clamp(0.0, gw as f32);
        let y0 = ((y - oy) * s).floor().clamp(0.0, gh as f32);
        let x1 = ((x - ox + w.max(0.0)) * s).ceil().clamp(0.0, gw as f32);
        let y1 = ((y - oy + h.max(0.0)) * s).ceil().clamp(0.0, gh as f32);
        if x1 > x0 && y1 > y0 && owner(k).is_some() {
            let origin = [ox + x0 / s, oy + y0 / s];
            plans.push(LayerPlan {
                index: k,
                origin,
                size: ((x1 - x0) as u32, (y1 - y0) as u32),
                rect: [origin[0], origin[1], (x1 - x0) / s, (y1 - y0) / s],
                scale: s,
                target: None,
                parent,
            });
        }
    }
    og_core::sort::sort_by_key(&mut plans, |p| std::cmp::Reverse(depth[p.index as usize]));
    // Glass and backdrops draw only in the output, and so do blends that
    // read the destination in a world panel's target; in a layer of the
    // UI pass's own they read a copy of the layer as drawn so far (a
    // group's backdrop, CSS Compositing 1 §3.2). A layer that is not drawn
    // composites nothing.
    let drawn: Vec<u32> = plans.iter().map(|p| p.index).collect();
    let own: Vec<u32> = plans
        .iter()
        .filter(|p| p.target.is_none())
        .map(|p| p.index)
        .collect();
    for (t, _, s, _) in keyed.iter_mut() {
        // Multiply's fixed-function blend takes the destination as opaque;
        // over a layer's transparent pixels it reads it instead.
        if *t > 0
            && let OverlaySource::Ui(ShapeSource::Shape(p @ ShapePipe::Multiply, _)) = s
        {
            *p = ShapePipe::Destination;
        }
    }
    keyed.retain(|(t, _, s, _)| {
        let in_layer_ok =
            *t == 0 || !(s.needs_pyramid() || s.needs_copy() && !own.contains(&(*t - 1)));
        let composite_ok = match s {
            OverlaySource::Ui(ShapeSource::Composite(k, _)) => drawn.contains(k),
            _ => true,
        };
        let target_ok = *t == 0 || drawn.contains(&(*t - 1));
        in_layer_ok && composite_ok && target_ok
    });
    og_core::sort::sort_by_key(&mut keyed, |(t, layer, source, _)| (*t, *layer, *source));
    let mut runs: Vec<OverlayRun> = Vec::new();
    let mut instances = Vec::with_capacity(ov.len());
    let mut ui_instances = Vec::with_capacity(uv.len());
    let mut mass_instances = Vec::new();
    for (t, _, source, index) in keyed {
        if let OverlaySource::Mass(k) = source {
            let first = mass_instances.len() as u32;
            mass_instances.extend(ui.mass[k as usize].instances.iter().map(|i| {
                let mut i = *i;
                i.batch = k;
                i
            }));
            runs.push(OverlayRun {
                target: t,
                source,
                first,
                count: mass_instances.len() as u32 - first,
            });
            continue;
        }
        let first = if source.is_ui() {
            ui_instances.push(uv[index as usize]);
            ui_instances.len() as u32 - 1
        } else {
            instances.push(ov[index as usize]);
            instances.len() as u32 - 1
        };
        match runs.last_mut() {
            Some(run) if run.source == source && run.target == t => run.count += 1,
            _ => runs.push(OverlayRun {
                target: t,
                source,
                first,
                count: 1,
            }),
        }
    }
    Ok(OverlayFrame {
        instances,
        ui_instances,
        tables,
        runs,
        backdrops: plans
            .iter()
            .filter(|p| {
                let l = &layers[p.index as usize];
                l.backdrop && p.target.is_none() && l.ground.is_none()
            })
            .map(|p| p.index)
            .collect(),
        layers: plans,
        layer_masks,
        sprites,
        glyphs,
        missing_glyphs,
        mass_templates,
        mass_table,
        mass_instances,
    })
}

/// The overlay's sprite and MSDF pipelines for one target format.
pub(crate) struct OverlayPipelines {
    pub sprite: Deferred<wgpu::RenderPipeline>,
    pub msdf: Deferred<wgpu::RenderPipeline>,
    pub glass: Deferred<wgpu::RenderPipeline>,
}

/// The overlay's pipelines and buffers: in the output's format and, for a
/// non-sRGB output the backend can view as sRGB, in that sRGB format; and
/// in the UI layers' format.
pub(crate) struct OverlayRenderer {
    pub output: OverlayPipelines,
    pub linear: Option<OverlayPipelines>,
    pub layer: OverlayPipelines,
    /// Group 0 (the uniform and the UI tables, ui_common.wgsl), and the
    /// formats of `output` and `linear`: what the array sprite variant
    /// (src/sprite_array.rs) is made with.
    pub layout: wgpu::BindGroupLayout,
    pub output_format: wgpu::TextureFormat,
    pub linear_format: Option<wgpu::TextureFormat>,
    pub buffer: wgpu::Buffer,
    pub mass_eases: crate::ui::mass::EaseTables,
}

impl OverlayRenderer {
    /// Makes one of its pipelines no frame used yet, the output's first (Renderer::warm).
    pub(crate) fn warm(&self) -> bool {
        let o = &self.output;
        crate::deferred::warm_one(&[&o.sprite, &o.msdf, &o.glass])
            || self
                .linear
                .as_ref()
                .is_some_and(|l| crate::deferred::warm_one(&[&l.sprite, &l.msdf, &l.glass]))
    }

    pub fn new(
        device: &wgpu::Device,
        output_format: wgpu::TextureFormat,
        linear_format: Option<wgpu::TextureFormat>,
        sprite_layout: &wgpu::BindGroupLayout,
    ) -> OverlayRenderer {
        let storage = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("overlay"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                storage(1),
                storage(2),
                storage(3),
                storage(4),
                // The frame's view depth (occluders, mass elements).
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("overlay"),
            bind_group_layouts: &[Some(&layout), Some(sprite_layout)],
            immediate_size: 0,
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("overlay"),
            source: wgpu::ShaderSource::Wgsl(overlay_source().into()),
        });
        const BUFFERS: &[Option<wgpu::VertexBufferLayout<'static>>] = &[Some(OVERLAY_LAYOUT)];
        let pipeline = |label: &'static str, fs: &'static str, format: wgpu::TextureFormat| {
            let (device, pl, module) = (device.clone(), pl.clone(), module.clone());
            Deferred::new(move || {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(label),
                    layout: Some(&pl),
                    vertex: wgpu::VertexState {
                        module: &module,
                        entry_point: Some("vs"),
                        compilation_options: Default::default(),
                        buffers: BUFFERS,
                    },
                    primitive: Default::default(),
                    depth_stencil: None,
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &module,
                        entry_point: Some(fs),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview_mask: None,
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            })
        };
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("overlay instances"),
            size: OVERLAY_STRIDE * 256,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let pipelines = |format| OverlayPipelines {
            sprite: pipeline("overlay sprite", "fs", format),
            msdf: pipeline("overlay msdf", "fs_msdf", format),
            glass: pipeline("overlay glass", "fs_glass", format),
        };
        OverlayRenderer {
            output: pipelines(output_format),
            linear: linear_format.map(pipelines),
            layer: pipelines(crate::ui::pass::LAYER_FORMAT),
            layout,
            output_format,
            linear_format,
            buffer,
            mass_eases: Default::default(),
        }
    }

    /// The uniform (`UiFrame`) for `overlay` on an output of
    /// `width`×`height` device pixels: its scale, or (0) the scale that
    /// fits its height to the output.
    pub fn uniform(width: u32, height: u32, overlay: &Overlay, encode: bool) -> [f32; 8] {
        let scale = Self::scale(height, overlay);
        [
            width as f32,
            height as f32,
            scale,
            encode as u32 as f32,
            0.0,
            0.0,
            overlay.ui.time,
            0.0,
        ]
    }

    /// Device pixels per CSS pixel of `overlay` on an output `height` tall.
    pub fn scale(height: u32, overlay: &Overlay) -> f32 {
        if overlay.scale > 0.0 {
            overlay.scale
        } else if overlay.height > 0.0 {
            height as f32 / overlay.height
        } else {
            1.0
        }
    }
}

/// The texture arrays of the array runs (for bind groups made before the pass).
pub(crate) fn run_arrays(frame: &OverlayFrame) -> impl Iterator<Item = TextureArrayHandle> + '_ {
    frame.runs.iter().filter_map(|r| match r.source {
        OverlaySource::Array(a) => Some(a),
        _ => None,
    })
}

/// The texture of a sprite or image run (for bind groups made before the pass).
pub(crate) fn run_textures(
    frame: &OverlayFrame,
) -> impl Iterator<Item = Option<TextureHandle>> + '_ {
    frame.runs.iter().filter_map(|r| match r.source {
        OverlaySource::Sprite(t) => Some(t),
        OverlaySource::Ui(ShapeSource::Shape(_, t)) => Some(t),
        _ => None,
    })
}

/// The overlay's module: the UI pass's common code, then overlay.wgsl
/// (`crate::shaders` validates it).
pub(crate) fn overlay_source() -> String {
    format!(
        "{}{}{}",
        crate::LIBRARY_WGSL,
        crate::ui::UI_COMMON_WGSL,
        OVERLAY_WGSL
    )
}

const OVERLAY_WGSL: &str = wgsl!("overlay.wgsl");
