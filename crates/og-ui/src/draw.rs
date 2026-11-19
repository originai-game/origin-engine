//! From a laid-out frame to og-render's overlay ([`og_draw::Overlay`]):
//! the UI pass's analytic shapes ([`og_draw::UiShape`]), MSDF text, the
//! UI's clip shapes and offscreen layers (`docs/ui.md` §3.3).
//!
//! Every rectangle, bar, half disc, image and glass panel is a UI shape:
//! a signed distance evaluated per pixel, so its edge is exact at any
//! scale. og-render sorts by (layer, kind, texture), so draw order is kept
//! with **painter levels**: an overlapping command advances a level when
//! the renderer would otherwise reverse its order. Same-batch commands
//! and font→shape crossings already draw in submission order on one level.
//! Other crossings conservatively advance, so the sort cannot reorder
//! anything visible, and a typical screen is two to four draws (§5.7).
//!
//! **Clipping is per pixel**: every command carries its clip rectangle,
//! and the commands between a [`DrawCmd::PushClip`] and its
//! [`DrawCmd::PopClip`] are also cut to its shape, antialiased (a rounded
//! panel's children, nested). The commands between a
//! [`DrawCmd::BeginLayer`] and its [`DrawCmd::EndLayer`] draw into a layer
//! of their own, with levels of their own, and the layer is one command of
//! its parent: its composite, through its material.
//!
//! **Every command carries its element's map** ([`DrawItem::transform`]):
//! og-ui bakes translation and scale into the old commands, and a turn or
//! skew reaches here, where every shape, clip, vector and layer composite
//! takes the whole map (a 2×2 about its centre, so a skew is exact) and
//! text takes it per glyph. A transformed command's clip rectangle is in
//! screen space.

use crate::element::Affine;
use crate::layout::Rect;
use crate::world::WorldCmd;
use og_core::glam::Vec2;
use og_draw::{
    FontHandle, TextureHandle, UiBlend, UiGeometry, UiMaterial, UiMode, UiPaint, UiShape,
    UiVectorHandle,
};

pub(crate) mod draw_codec;
mod emit;
mod order;

pub use emit::{EmitDesc, LAYER_BASE, LAYER_SPAN, MAX_LAYERS, RetainedEmission, drawn_lines};
pub(crate) use emit::{PropertyUpdate, emit, scale_effects, zoom_item, zoom_items, zoom_mass};
pub(crate) use order::{Levels, Spliced, assign_levels_kept};

/// One thing to draw, in CSS pixels.
#[derive(Clone, Debug, PartialEq)]
pub enum DrawCmd {
    /// Glass: the frame under `rect` blurred by `blur` CSS px, its
    /// saturation scaled (1: as it is), tinted (`tint`: linear RGB, alpha
    /// how much covers the blur), with its `material` (hairline, shadows,
    /// solid fallback; the default none), masked to the rounded rectangle,
    /// at `alpha`: a backdrop ([`og_draw::UiMode::Backdrop`]).
    Glass {
        rect: Rect,
        radius: f32,
        blur: f32,
        saturation: f32,
        tint: [f32; 4],
        material: og_draw::UiGlass,
        alpha: f32,
        clip: Rect,
    },
    /// A rectangle with rounded corners (radius 0: square).
    Quad {
        rect: Rect,
        radius: f32,
        /// Linear RGBA.
        color: [f32; 4],
        clip: Rect,
    },
    /// A textured rectangle (`texture: None` is solid).
    Image {
        rect: Rect,
        texture: Option<TextureHandle>,
        uv: [f32; 4],
        color: [f32; 4],
        clip: Rect,
    },
    /// A solid bar `w`×`h` centred at `center`, rotated clockwise on screen by
    /// `angle` radians (chevrons, ticks, crosses).
    Bar {
        center: (f32, f32),
        w: f32,
        h: f32,
        angle: f32,
        color: [f32; 4],
        clip: Rect,
    },
    /// One line of text: `x` the pen start, `baseline` its baseline.
    Text {
        /// The emitting UI node's key, for inspection; zero for drawing
        /// without an element. Kept with the line through paint reuse.
        source: u64,
        x: f32,
        baseline: f32,
        size: f32,
        text: String,
        color: [f32; 4],
        font: FontHandle,
        /// The line box, for overlap.
        bounds: Rect,
        clip: Rect,
        /// Added to each glyph's advance, em (the type step's tracking).
        tracking: f32,
        /// Thickening at the glyphs' edge, CSS px (a faux bold).
        weight: f32,
        /// A glow from the font's distance field: linear RGBA and radius, CSS px.
        glow: Option<([f32; 4], f32)>,
        /// A visual line's placements (bidi runs, shaped glyphs), one per
        /// character of `text`; empty: the font's advances and kerning.
        placements: Vec<og_draw::GlyphPlacement>,
    },
    /// A [`DrawCmd::Text`] with silhouette shadows, a distance-field
    /// outline and a gradient fill (§5.6).
    StyledText {
        text: Box<DrawCmd>,
        effects: Box<og_draw::UiTextEffects>,
    },
    /// Any UI shape: geometry, paint, mode, opacity and blend. Its `layer`
    /// and `clip` are set when it is emitted.
    Shape {
        shape: Box<UiShape>,
        clip: Rect,
    },
    /// A cooked vector contained and centred in `rect`.
    Vector {
        vector: UiVectorHandle,
        rect: Rect,
        transform: [f32; 4],
        tint: [f32; 4],
        clip: Rect,
    },
    /// Cuts what follows, until its [`DrawCmd::PopClip`], to a shape
    /// centred on `center` with half extent `half` (nested cuts multiply).
    PushClip {
        geometry: UiGeometry,
        center: (f32, f32),
        half: (f32, f32),
    },
    PopClip,
    /// What follows, until its [`DrawCmd::EndLayer`], draws offscreen.
    /// `rect` (the union of what it draws, grown by `expand`) is set when
    /// levels are assigned; with a `mask` it is the box the mask is fitted
    /// to, under the element's map. With a `filter` (an SVG `<filter>`'s region
    /// and graph, CSS px) the layer is the filter region instead and
    /// composites the graph's result. `role` says what it is for.
    BeginLayer {
        material: UiMaterial,
        params: [[f32; 4]; og_draw::UI_MATERIAL_PARAMS],
        mask: Option<og_draw::UiMask>,
        expand: f32,
        rect: Rect,
        filter: Option<Box<(Rect, og_draw::UiFilter)>>,
        role: LayerRole,
    },
    /// The layer's composite: `geometry` over `rect` (`None`: the layer's
    /// whole rectangle), at `opacity` with `blend`. With `ground`, the
    /// world panel of that key the UI draws itself: the layer is laid onto
    /// the quad its panel is seen at this tick ([`UiFrame::emit`] reads it
    /// from the frame), and nowhere when it has none (behind the camera).
    EndLayer {
        geometry: Option<UiGeometry>,
        rect: Rect,
        opacity: f32,
        blend: UiBlend,
        clip: Rect,
        ground: Option<Ground>,
    },
    /// World-space UI's commands (`crate::world`): occlusion brackets,
    /// mass elements and particles, a world panel's target.
    World(crate::world::WorldCmd),
}

/// What a [`DrawCmd::BeginLayer`] is for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LayerRole {
    /// It draws what follows until its end and composites it.
    #[default]
    Draw,
    /// CSS `backdrop-filter`: it draws nothing; its filter runs over what
    /// is under its composite where the composite draws
    /// ([`og_draw::UiLayer::backdrop`]), its region its filter's.
    Backdrop,
    /// The `n`-th element an `feImage` of the enclosing layer's filter
    /// reads ([`crate::svg::filter::element_layer`]): what follows is
    /// drawn and composited nowhere; the filter reads it.
    Image(u32),
    /// A CSS `element()` image's group (CSS Images 4 §2.7): node `node`'s
    /// box layer `slot` ([`crate::paint::ElementDraw::element_layer`]),
    /// filled with the element's drawing once the frame's command pass has
    /// drawn it, then an ordinary [`LayerRole::Draw`]. Its node is
    /// `u32::MAX` until the pass places it.
    Element { node: u32, slot: u32 },
}

/// Where a layer's composite is laid by a quad of the frame
/// ([`crate::GroundQuad`] of that key) rather than by its command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ground {
    /// A world panel's, under the screen's levels.
    Panel(u64),
    /// The texts and badges of the world panel of that key, each a layer
    /// laid with its panel's quad: over every panel's other drawing, so a
    /// ground's label reads over the rings and lines of the grounds about
    /// it, and over its own.
    Label(u64),
    /// A CSS 3D plane's, in the screen's levels where it stands (and, in a
    /// 3D rendering context, sorted by depth among the context's planes).
    Plane(u64),
}

/// A command and the painter level it draws on (in its layer's levels,
/// for the commands between a layer's start and end).
#[derive(Clone, Debug, PartialEq)]
pub struct DrawItem {
    pub level: i32,
    pub cmd: DrawCmd,
    /// The map from the command's coordinates to the screen: the identity,
    /// except under an element's rotation or skew (og-ui bakes translation
    /// and scale into the old commands; a shape, vector, clip or layer
    /// keeps any map). Its clip is in screen space.
    pub transform: Affine,
}

/// How a command batches in og-render's overlay (its sort key there).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Batch {
    /// A UI shape: its pipeline (blend) and texture.
    Ui(u8, Option<TextureHandle>),
    Backdrop,
    Font(FontHandle),
    /// A text's offscreen shadows followed by its glyphs.
    ShadowFont,
    Vector(UiVectorHandle),
    /// A layer's composite (each its own).
    Composite(u32),
    /// Mass elements (particles), each batch its own draw.
    Mass,
}

impl Batch {
    fn step_after(self, earlier: Self) -> i32 {
        if matches!(self, Self::ShadowFont) || matches!(earlier, Self::ShadowFont) {
            return 1;
        }
        // OverlaySource::Font sorts before every UI ShapeSource, including
        // backdrop and composite shapes. Keep ambiguous shape/pipeline
        // crossings conservative: their exact source depends on the target.
        i32::from(
            self != earlier
                && !matches!(
                    (earlier, self),
                    (
                        Self::Font(_),
                        Self::Ui(..) | Self::Backdrop | Self::Composite(_)
                    )
                ),
        )
    }
}

fn pipe_of(blend: UiBlend) -> u8 {
    match blend {
        UiBlend::Normal => 0,
        UiBlend::Multiply => 1,
        UiBlend::Screen => 2,
        UiBlend::Add => 3,
        _ => 4,
    }
}

/// The smallest rectangle holding both.
fn union(a: &Rect, b: &Rect) -> Rect {
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    Rect::new(
        x,
        y,
        a.right().max(b.right()) - x,
        a.bottom().max(b.bottom()) - y,
    )
}

/// A UI shape's bounds: its quad grown by what its mode draws outside it,
/// turned by its transform.
fn shape_bounds(s: &UiShape) -> Rect {
    let (mut center, mut half) = (s.center, s.half);
    if let UiGeometry::Polyline {
        points,
        width,
        join,
        ..
    } = &s.geometry
    {
        let (mut lo, mut hi) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
        for p in points {
            lo = lo.min(*p);
            hi = hi.max(*p);
        }
        if points.is_empty() {
            return Rect::default();
        }
        let limit = match join {
            og_draw::UiJoin::Miter(l) => l.max(1.5),
            _ => 1.5,
        };
        let m = s.transform;
        let mid = (lo + hi) * 0.5;
        center += Vec2::new(m[0] * mid.x + m[2] * mid.y, m[1] * mid.x + m[3] * mid.y);
        half = (hi - lo) * 0.5 + Vec2::splat(0.5 * width * limit);
    }
    if let UiGeometry::Path(path) = &s.geometry {
        // A path's box is its curves' (controls included), not its paint's.
        let Some((lo, hi)) = path.bounds() else {
            return Rect::default();
        };
        let m = s.transform;
        let mid = (lo + hi) * 0.5;
        center += Vec2::new(m[0] * mid.x + m[2] * mid.y, m[1] * mid.x + m[3] * mid.y);
        half = (hi - lo) * 0.5;
    }
    let mut grow = 1.0;
    let mut shift = Vec2::ZERO;
    match s.mode {
        UiMode::Stroke(st) => {
            let off = match st.align {
                og_draw::UiAlign::Inside => -0.5,
                og_draw::UiAlign::Center => 0.0,
                og_draw::UiAlign::Outside => 0.5,
            } * st.width;
            grow += (0.5 * st.width + off).max(0.0);
        }
        UiMode::Shadow {
            offset,
            blur,
            spread,
        } => {
            grow += 1.5 * blur.max(0.0) + spread.max(0.0);
            shift = offset;
        }
        _ => {}
    }
    let m = s.transform;
    let (hx, hy) = (half.x + grow, half.y + grow);
    let ex = m[0].abs() * hx + m[2].abs() * hy;
    let ey = m[1].abs() * hx + m[3].abs() * hy;
    let shift = Vec2::new(
        m[0] * shift.x + m[2] * shift.y,
        m[1] * shift.x + m[3] * shift.y,
    );
    let c = center + shift;
    union(
        &Rect::new(c.x - ex, c.y - ey, 2.0 * ex, 2.0 * ey),
        &Rect::new(center.x - ex, center.y - ey, 2.0 * ex, 2.0 * ey),
    )
}

/// The 2×2 part of `m` as og-render's `[a, b, c, d]`.
fn linear(m: &Affine) -> [f32; 4] {
    [m.a, m.b, m.c, m.d]
}

/// `l · r` of two 2×2 matrices in og-render's order.
fn mul2(l: [f32; 4], r: [f32; 4]) -> [f32; 4] {
    [
        l[0] * r[0] + l[2] * r[1],
        l[1] * r[0] + l[3] * r[1],
        l[0] * r[2] + l[2] * r[3],
        l[1] * r[2] + l[3] * r[3],
    ]
}

/// A UI shape under the map `m`: its centre moved, its local space
/// composed with the map's linear part (rotation, scale and skew).
fn mapped_shape(mut s: UiShape, m: &Affine) -> UiShape {
    if m.is_identity() {
        return s;
    }
    let (x, y) = m.apply(s.center.x, s.center.y);
    s.center = Vec2::new(x, y);
    s.transform = mul2(linear(m), s.transform);
    s
}

/// A command's bounds on the screen under its map. Only the top-level
/// overlay supplies its local device-pixel width; other targets keep the
/// broad bounds, since their independent raster scale is not known here.
pub(crate) fn mapped_bounds(c: &DrawCmd, m: &Affine, pixel_width: Option<f32>) -> Rect {
    match c {
        DrawCmd::Shape { shape, clip } => {
            let mapped;
            let s = if m.is_identity() {
                &**shape
            } else {
                mapped = mapped_shape((**shape).clone(), m);
                &mapped
            };
            let mut bounds = shape_bounds(s);
            if let Some(px) = pixel_width
                && px.is_finite()
                && px > 0.0
                && s.transform == [1.0, 0.0, 0.0, 1.0]
                && matches!(s.geometry, UiGeometry::Rect { .. })
                && let UiMode::Shadow {
                    offset,
                    blur,
                    spread,
                } = s.mode
            {
                // A round corner's rounded_box_shadow in ui_common.wgsl
                // integrates only across ±3σ in Y. Its erf approximation
                // has no matching compact X support, so retain the broad
                // X bounds. Any other corner (a bevel, a notch, a CSS
                // superellipse) takes the erf of the distance to its
                // shape, which lies inside the same box: 3σ past the
                // spread box it is under 0.14 %, the reach the broad
                // bounds give a shadow without spread. A shrunk shadow
                // (a negative spread) so ends where a round one does.
                let half = (s.half.y + spread).max(0.0);
                let reach = 3.0 * (0.5 * blur.max(0.0)).max(0.5 * px);
                // Outward device-pixel and arithmetic guards preserve
                // coverage at the CPU/GPU floating-point boundary.
                let guard =
                    px + 8.0 * f32::EPSILON * (s.center.y.abs() + offset.y.abs() + half + reach);
                let lo = s.center.y + offset.y - half - reach - guard;
                let hi = s.center.y + offset.y + half + reach + guard;
                if lo.is_finite() && hi.is_finite() {
                    bounds = bounds.intersect(&Rect::new(bounds.x, lo, bounds.width, hi - lo));
                }
            }
            bounds.intersect(clip)
        }
        _ if m.is_identity() => c.bounds(),
        _ => m.map_rect(&c.unclipped()).intersect(c.clip()),
    }
}

impl DrawCmd {
    /// What it fills, when it is a flat area of colour: a quad, an untextured
    /// image, or a UI shape's fill (a gradient as the mean of its stops), as
    /// its rectangle and linear colour with its opacity. Shadows, strokes,
    /// glows and textures answer `None`. How a test reads what a label is
    /// drawn over.
    pub fn filled(&self) -> Option<(Rect, [f32; 4])> {
        match self {
            DrawCmd::Quad { rect, color, .. }
            | DrawCmd::Image {
                rect,
                texture: None,
                color,
                ..
            } => Some((*rect, *color)),
            DrawCmd::Shape { shape, .. } if shape.mode == og_draw::UiMode::Fill => {
                let mut c = match &shape.paint {
                    og_draw::UiPaint::Solid(c) => *c,
                    og_draw::UiPaint::Gradient(g) if !g.stops.is_empty() => {
                        let n = g.stops.len() as f32;
                        let mut m = [0.0; 4];
                        for s in &g.stops {
                            for (a, b) in m.iter_mut().zip(s.color) {
                                *a += b / n;
                            }
                        }
                        m
                    }
                    _ => return None,
                };
                c[3] *= shape.opacity;
                let (cx, cy, hx, hy) = (shape.center.x, shape.center.y, shape.half.x, shape.half.y);
                Some((Rect::new(cx - hx, cy - hy, 2.0 * hx, 2.0 * hy), c))
            }
            _ => None,
        }
    }

    /// Its clip rectangle (a bracket's: everything).
    pub(crate) fn clip(&self) -> &Rect {
        const ALL: Rect = Rect {
            x: -1e9,
            y: -1e9,
            width: 2e9,
            height: 2e9,
        };
        match self {
            DrawCmd::Glass { clip, .. }
            | DrawCmd::Quad { clip, .. }
            | DrawCmd::Image { clip, .. }
            | DrawCmd::Bar { clip, .. }
            | DrawCmd::Text { clip, .. }
            | DrawCmd::Shape { clip, .. }
            | DrawCmd::Vector { clip, .. }
            | DrawCmd::EndLayer { clip, .. } => clip,
            DrawCmd::StyledText { text, .. } => text.clip(),
            DrawCmd::World(w) => match w {
                WorldCmd::Mass { clip, .. } => clip,
                _ => &ALL,
            },
            DrawCmd::PushClip { .. } | DrawCmd::PopClip | DrawCmd::BeginLayer { .. } => &ALL,
        }
    }

    /// Its clip rectangle, to set (a bracket has none).
    pub(crate) fn clip_mut(&mut self) -> Option<&mut Rect> {
        match self {
            DrawCmd::Glass { clip, .. }
            | DrawCmd::Quad { clip, .. }
            | DrawCmd::Image { clip, .. }
            | DrawCmd::Bar { clip, .. }
            | DrawCmd::Text { clip, .. }
            | DrawCmd::Shape { clip, .. }
            | DrawCmd::Vector { clip, .. }
            | DrawCmd::EndLayer { clip, .. } => Some(clip),
            DrawCmd::StyledText { text, .. } => text.clip_mut(),
            DrawCmd::World(w) => match w {
                WorldCmd::Mass { clip, .. } => Some(clip),
                _ => None,
            },
            DrawCmd::PushClip { .. } | DrawCmd::PopClip | DrawCmd::BeginLayer { .. } => None,
        }
    }

    /// What it covers before its clip.
    pub(crate) fn unclipped(&self) -> Rect {
        let mut c = self.clone();
        if let Some(clip) = c.clip_mut() {
            *clip = Rect::new(-1e9, -1e9, 2e9, 2e9);
        }
        c.bounds()
    }

    fn batch(&self) -> Batch {
        match self {
            DrawCmd::Glass { .. } => Batch::Backdrop,
            DrawCmd::Quad { .. } | DrawCmd::Bar { .. } | DrawCmd::Image { texture: None, .. } => {
                Batch::Ui(0, None)
            }
            DrawCmd::Image {
                texture: Some(t), ..
            } => Batch::Ui(0, Some(*t)),
            DrawCmd::Text { font, .. } => Batch::Font(*font),
            DrawCmd::StyledText { text, effects } => {
                if effects.shadows.is_empty() {
                    text.batch()
                } else {
                    Batch::ShadowFont
                }
            }
            DrawCmd::Shape { shape, .. } => {
                if matches!(shape.mode, UiMode::Backdrop { .. }) {
                    return Batch::Backdrop;
                }
                let texture = match &shape.paint {
                    UiPaint::Image(im) => Some(im.texture),
                    _ => None,
                };
                Batch::Ui(pipe_of(shape.blend), texture)
            }
            DrawCmd::Vector { vector, .. } => Batch::Vector(*vector),
            DrawCmd::World(_) => Batch::Mass,
            // Brackets: never levelled (empty bounds).
            DrawCmd::PushClip { .. }
            | DrawCmd::PopClip
            | DrawCmd::BeginLayer { .. }
            | DrawCmd::EndLayer { .. } => Batch::Ui(0, None),
        }
    }

    fn bounds(&self) -> Rect {
        match self {
            DrawCmd::Quad { rect, clip, .. }
            | DrawCmd::Image { rect, clip, .. }
            | DrawCmd::Glass { rect, clip, .. }
            | DrawCmd::Vector { rect, clip, .. } => rect.intersect(clip),
            DrawCmd::Text { bounds, clip, .. } => bounds.intersect(clip),
            DrawCmd::StyledText { text, effects } => {
                let DrawCmd::Text {
                    bounds, size, clip, ..
                } = &**text
                else {
                    return text.bounds();
                };
                let mut b = *bounds;
                let reach = effects.outline.map_or(0.0, |o| o.width);
                b = b.outset(reach);
                let ink = bounds.outset(*size);
                for s in &effects.shadows {
                    let moved = Rect::new(
                        ink.x + s.offset.x,
                        ink.y + s.offset.y,
                        ink.width,
                        ink.height,
                    )
                    .outset(1.5 * s.blur.max(0.0));
                    b = union(&b, &moved);
                }
                b.intersect(clip)
            }
            DrawCmd::Shape { shape, clip } => shape_bounds(shape).intersect(clip),
            DrawCmd::World(w) => match w {
                WorldCmd::Mass { bounds, clip, .. } => bounds.intersect(clip),
                _ => Rect::default(),
            },
            DrawCmd::Bar {
                center, w, h, clip, ..
            } => {
                let d = (w * w + h * h).sqrt();
                Rect::new(center.0 - d / 2.0, center.1 - d / 2.0, d, d).intersect(clip)
            }
            DrawCmd::PushClip { .. }
            | DrawCmd::PopClip
            | DrawCmd::BeginLayer { .. }
            | DrawCmd::EndLayer { .. } => Rect::default(),
        }
    }
}

/// Assigns painter levels in submission order (see the module
/// documentation); each command's bounds are taken under its map.
#[cfg(any(test, debug_assertions))]
pub(crate) fn assign_levels(
    cmds: Vec<(DrawCmd, Affine)>,
    pixel_width: Option<f32>,
) -> Vec<DrawItem> {
    assign_levels_kept(cmds, pixel_width).0
}

#[cfg(test)]
mod levels;
