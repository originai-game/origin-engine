//! From a laid-out frame to og-render's overlay ([`og_render::Overlay`]):
//! the UI pass's analytic shapes ([`og_render::UiShape`]), MSDF text, the
//! UI's clip shapes and offscreen layers (`docs/ui.md` §5).
//!
//! Every rectangle, bar, half disc, image and glass panel is a UI shape:
//! a signed distance evaluated per pixel, so its edge is exact at any
//! scale. og-render sorts by (layer, kind, texture), so draw order is kept
//! with **painter levels**: a command's level is one above every earlier
//! command it overlaps that batches differently, and equal to that of an
//! overlapping earlier command of its own batch. Commands of different
//! batches on one level never overlap, so the sort cannot reorder anything
//! visible, and a typical screen is two to four draws (§5.7).
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
use og_render::{
    FontHandle, Overlay, Text, TextureHandle, UiBlend, UiGeometry, UiMaterial, UiMode, UiPaint,
    UiShape, UiVectorHandle,
};

/// One thing to draw, in CSS pixels.
#[derive(Clone, Debug, PartialEq)]
pub enum DrawCmd {
    /// Glass: the frame under `rect` blurred by `blur` CSS px, its
    /// saturation scaled (1: as it is), tinted (`tint`: linear RGB, alpha
    /// how much covers the blur), with its `material` (hairline, shadows,
    /// solid fallback; the default none), masked to the rounded rectangle,
    /// at `alpha`: a backdrop ([`og_render::UiMode::Backdrop`]).
    Glass {
        rect: Rect,
        radius: f32,
        blur: f32,
        saturation: f32,
        tint: [f32; 4],
        material: og_render::UiGlass,
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
    /// Half a disc of `radius` around `center`, on the side `angle` points to
    /// (0: right, clockwise on screen). Pies and rings are two of them.
    Half {
        center: (f32, f32),
        radius: f32,
        angle: f32,
        color: [f32; 4],
        clip: Rect,
    },
    /// One line of text: `x` the pen start, `baseline` its baseline.
    Text {
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
        placements: Vec<og_render::GlyphPlacement>,
    },
    /// A [`DrawCmd::Text`] with effects drawn from its font's distance
    /// field: shadows, an outline, a gradient fill (§5.6).
    StyledText {
        text: Box<DrawCmd>,
        effects: Box<og_render::UiTextEffects>,
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
    /// levels are assigned.
    BeginLayer {
        material: UiMaterial,
        params: [[f32; 4]; og_render::UI_MATERIAL_PARAMS],
        mask: Option<og_render::UiMask>,
        expand: f32,
        rect: Rect,
    },
    /// The layer's composite: `geometry` over `rect` (`None`: the layer's
    /// whole rectangle), at `opacity` with `blend`.
    EndLayer {
        geometry: Option<UiGeometry>,
        rect: Rect,
        opacity: f32,
        blend: UiBlend,
        clip: Rect,
    },
    /// World-space UI's commands (`crate::world`): occlusion brackets,
    /// mass elements and particles, a world panel's target.
    World(Box<crate::world::WorldCmd>),
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
    Vector(UiVectorHandle),
    /// A layer's composite (each its own).
    Composite(u32),
    /// Mass elements (particles), each batch its own draw.
    Mass,
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
            og_render::UiJoin::Miter(l) => l.max(1.5),
            _ => 1.5,
        };
        let m = s.transform;
        let mid = (lo + hi) * 0.5;
        center += Vec2::new(m[0] * mid.x + m[2] * mid.y, m[1] * mid.x + m[3] * mid.y);
        half = (hi - lo) * 0.5 + Vec2::splat(0.5 * width * limit);
    }
    let mut grow = 1.0;
    let mut shift = Vec2::ZERO;
    match s.mode {
        UiMode::Stroke(st) => {
            let off = match st.align {
                og_render::UiAlign::Inside => -0.5,
                og_render::UiAlign::Center => 0.0,
                og_render::UiAlign::Outside => 0.5,
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

/// A command's bounds on the screen under its map.
fn mapped_bounds(c: &DrawCmd, m: &Affine) -> Rect {
    if m.is_identity() {
        return c.bounds();
    }
    match c {
        DrawCmd::Shape { shape, clip } => {
            shape_bounds(&mapped_shape((**shape).clone(), m)).intersect(clip)
        }
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
            DrawCmd::Shape { shape, .. } if shape.mode == og_render::UiMode::Fill => {
                let mut c = match &shape.paint {
                    og_render::UiPaint::Solid(c) => *c,
                    og_render::UiPaint::Gradient(g) if !g.stops.is_empty() => {
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
            | DrawCmd::Half { clip, .. }
            | DrawCmd::Text { clip, .. }
            | DrawCmd::Shape { clip, .. }
            | DrawCmd::Vector { clip, .. }
            | DrawCmd::EndLayer { clip, .. } => clip,
            DrawCmd::StyledText { text, .. } => text.clip(),
            DrawCmd::World(w) => match &**w {
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
            | DrawCmd::Half { clip, .. }
            | DrawCmd::Text { clip, .. }
            | DrawCmd::Shape { clip, .. }
            | DrawCmd::Vector { clip, .. }
            | DrawCmd::EndLayer { clip, .. } => Some(clip),
            DrawCmd::StyledText { text, .. } => text.clip_mut(),
            DrawCmd::World(w) => match &mut **w {
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
            DrawCmd::Quad { .. }
            | DrawCmd::Bar { .. }
            | DrawCmd::Half { .. }
            | DrawCmd::Image { texture: None, .. } => Batch::Ui(0, None),
            DrawCmd::Image {
                texture: Some(t), ..
            } => Batch::Ui(0, Some(*t)),
            DrawCmd::Text { font, .. } => Batch::Font(*font),
            DrawCmd::StyledText { text, .. } => text.batch(),
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
                let DrawCmd::Text { bounds, clip, .. } = &**text else {
                    return text.bounds();
                };
                let mut b = *bounds;
                let reach = effects.outline.map_or(0.0, |o| o.width);
                b = b.outset(reach);
                for s in &effects.shadows {
                    let moved = Rect::new(b.x + s.offset.x, b.y + s.offset.y, b.width, b.height)
                        .outset(s.blur.max(0.0));
                    b = union(&b, &moved);
                }
                b.intersect(clip)
            }
            DrawCmd::Shape { shape, clip } => shape_bounds(shape).intersect(clip),
            DrawCmd::World(w) => match &**w {
                WorldCmd::Mass { bounds, clip, .. } => bounds.intersect(clip),
                _ => Rect::default(),
            },
            DrawCmd::Bar {
                center, w, h, clip, ..
            } => {
                let d = (w * w + h * h).sqrt();
                Rect::new(center.0 - d / 2.0, center.1 - d / 2.0, d, d).intersect(clip)
            }
            DrawCmd::Half {
                center,
                radius,
                clip,
                ..
            } => Rect::new(
                center.0 - radius,
                center.1 - radius,
                2.0 * radius,
                2.0 * radius,
            )
            .intersect(clip),
            DrawCmd::PushClip { .. }
            | DrawCmd::PopClip
            | DrawCmd::BeginLayer { .. }
            | DrawCmd::EndLayer { .. } => Rect::default(),
        }
    }
}

/// One command of a level space, or a layer: its start and end in the
/// space, its own items between.
enum Entry {
    Cmd(DrawCmd, Affine),
    /// A clip bracket: drawn where it stands, never levelled.
    Bracket(DrawCmd, Affine),
    Layer {
        begin: Box<(DrawCmd, Affine)>,
        items: Vec<DrawItem>,
        end: Box<(DrawCmd, Affine)>,
        bounds: Rect,
    },
    /// A world panel: its items drawn into its target.
    Panel {
        begin: Box<(DrawCmd, Affine)>,
        items: Vec<DrawItem>,
        end: Box<(DrawCmd, Affine)>,
    },
}

/// Collects one space's entries from `cmds` until its `EndLayer` (`None`:
/// the frame's end), levelling each layer's own space on the way.
fn collect(
    cmds: &mut std::vec::IntoIter<(DrawCmd, Affine)>,
    layers: &mut u32,
) -> (Vec<Entry>, Option<(DrawCmd, Affine)>) {
    let mut entries = Vec::new();
    while let Some((cmd, m)) = cmds.next() {
        match cmd {
            DrawCmd::PushClip { .. } | DrawCmd::PopClip => entries.push(Entry::Bracket(cmd, m)),
            DrawCmd::World(ref w) if matches!(**w, WorldCmd::BeginPanel { .. }) => {
                // A world panel: its own space of levels, off the screen
                // (it levels nothing there).
                let (inner, end) = collect(cmds, layers);
                let items = levelled(inner, layers, None);
                let end = end.unwrap_or((
                    DrawCmd::World(Box::new(WorldCmd::EndPanel)),
                    Affine::IDENTITY,
                ));
                entries.push(Entry::Panel {
                    begin: Box::new((cmd, m)),
                    items,
                    end: Box::new(end),
                });
            }
            DrawCmd::World(ref w) if matches!(**w, WorldCmd::EndPanel) => {
                return (entries, Some((cmd, m)));
            }
            DrawCmd::World(ref w) if !matches!(**w, WorldCmd::Mass { .. }) => {
                entries.push(Entry::Bracket(cmd, m))
            }
            DrawCmd::BeginLayer { .. } => {
                let (inner, end) = collect(cmds, layers);
                let mut bounds = Rect::default();
                let items = levelled(inner, layers, Some(&mut bounds));
                let mut begin = cmd;
                let end = end.unwrap_or((
                    DrawCmd::EndLayer {
                        geometry: None,
                        rect: bounds,
                        opacity: 1.0,
                        blend: UiBlend::Normal,
                        clip: Rect::new(-1e9, -1e9, 2e9, 2e9),
                    },
                    Affine::IDENTITY,
                ));
                // The layer's rectangle is on the screen: the union of what
                // its items draw there, whatever their maps.
                if let DrawCmd::BeginLayer { rect, expand, .. } = &mut begin {
                    let r = bounds.outset(*expand);
                    *rect = r;
                    bounds = r;
                }
                if let (DrawCmd::EndLayer { clip, .. }, _) = &end {
                    bounds = bounds.intersect(clip);
                }
                entries.push(Entry::Layer {
                    begin: Box::new((begin, m)),
                    items,
                    end: Box::new(end),
                    bounds,
                });
            }
            DrawCmd::EndLayer { .. } => return (entries, Some((cmd, m))),
            other => entries.push(Entry::Cmd(other, m)),
        }
    }
    (entries, None)
}

/// Levels a space's entries and flattens them; `drawn` gets the union of
/// what they draw.
fn levelled(entries: Vec<Entry>, layers: &mut u32, drawn: Option<&mut Rect>) -> Vec<DrawItem> {
    let keyed: Vec<(Rect, Batch)> = entries
        .iter()
        .map(|e| match e {
            Entry::Cmd(c, m) => (mapped_bounds(c, m), c.batch()),
            Entry::Bracket(..) | Entry::Panel { .. } => (Rect::default(), Batch::Ui(0, None)),
            Entry::Layer { bounds, .. } => {
                *layers += 1;
                (*bounds, Batch::Composite(*layers))
            }
        })
        .collect();
    let levels = assign(&keyed);
    if let Some(u) = drawn {
        let mut acc: Option<Rect> = None;
        for (r, _) in &keyed {
            if !r.is_empty() {
                acc = Some(acc.map_or(*r, |a| union(&a, r)));
            }
        }
        *u = acc.unwrap_or_default();
    }
    let mut out = Vec::with_capacity(entries.len());
    for (entry, level) in entries.into_iter().zip(levels) {
        match (entry, level) {
            (Entry::Bracket(cmd, transform), _) => out.push(DrawItem {
                level: 0,
                cmd,
                transform,
            }),
            (Entry::Panel { begin, items, end }, _) => {
                out.push(DrawItem {
                    level: 0,
                    cmd: begin.0,
                    transform: begin.1,
                });
                out.extend(items);
                out.push(DrawItem {
                    level: 0,
                    cmd: end.0,
                    transform: end.1,
                });
            }
            (Entry::Cmd(cmd, transform), Some(level)) => out.push(DrawItem {
                level,
                cmd,
                transform,
            }),
            (
                Entry::Layer {
                    begin, items, end, ..
                },
                Some(level),
            ) => {
                out.push(DrawItem {
                    level,
                    cmd: begin.0,
                    transform: begin.1,
                });
                out.extend(items);
                out.push(DrawItem {
                    level,
                    cmd: end.0,
                    transform: end.1,
                });
            }
            // Nothing to draw.
            (_, None) => {}
        }
    }
    out
}

/// Assigns painter levels in submission order (see the module
/// documentation); each command's bounds are taken under its map.
pub(crate) fn assign_levels(cmds: Vec<(DrawCmd, Affine)>) -> Vec<DrawItem> {
    let mut layers = 0;
    let mut it = cmds.into_iter();
    let mut entries = Vec::new();
    loop {
        let (more, end) = collect(&mut it, &mut layers);
        entries.extend(more);
        // A stray end closes nothing.
        if end.is_none() {
            break;
        }
    }
    levelled(entries, &mut layers, None)
}

/// The levels of `keyed` (bounds and batch) in submission order: a command
/// goes above every earlier one it overlaps (on the same level when they
/// share a batch); `None` for empty bounds. Each placed rectangle is filed
/// in the cells of a coarse grid it touches, and a command tests only the
/// rectangles of its own cells: a screen transition's dissolve is 1,620
/// quads over the frame, and testing each against every earlier command was
/// 7 to 17 ms of a dev tick (Lantern Vale's playtests, tests/evidence/P4/perf-2).
fn assign(keyed: &[(Rect, Batch)]) -> Vec<Option<i32>> {
    let bounds: Vec<Rect> = keyed.iter().map(|(r, _)| *r).collect();
    let grid = Grid::over(&bounds);
    let mut cells: Vec<Vec<u32>> = vec![Vec::new(); grid.cols * grid.rows];
    let mut out = Vec::with_capacity(keyed.len());
    let mut placed: Vec<(Rect, Batch, i32)> = Vec::with_capacity(keyed.len());
    for &(bounds, batch) in keyed {
        if bounds.is_empty() {
            out.push(None);
            continue;
        }
        let (cols, rows) = grid.span(&bounds);
        let mut level = 0;
        for row in rows.clone() {
            for col in cols.clone() {
                for &p in &cells[row * grid.cols + col] {
                    let (r, b, l) = &placed[p as usize];
                    if r.overlaps(&bounds) {
                        level = level.max(if *b == batch { *l } else { *l + 1 });
                    }
                }
            }
        }
        let id = placed.len() as u32;
        for row in rows {
            for col in cols.clone() {
                cells[row * grid.cols + col].push(id);
            }
        }
        placed.push((bounds, batch, level));
        out.push(Some(level));
    }
    out
}

/// [`assign`]'s grid: about one cell for every four commands, at
/// most 64 by 64 cells of at least 32 CSS px, over the commands' bounds. A rectangle is filed in every cell its closed
/// extent touches (clamped to the grid), so two rectangles that overlap
/// share a cell.
struct Grid {
    x: f32,
    y: f32,
    cell_w: f32,
    cell_h: f32,
    cols: usize,
    rows: usize,
}

impl Grid {
    fn over(bounds: &[Rect]) -> Grid {
        let (mut x0, mut y0) = (f32::MAX, f32::MAX);
        let (mut x1, mut y1) = (f32::MIN, f32::MIN);
        for r in bounds.iter().filter(|r| !r.is_empty()) {
            if r.x.is_finite() && r.right().is_finite() {
                x0 = x0.min(r.x);
                x1 = x1.max(r.right());
            }
            if r.y.is_finite() && r.bottom().is_finite() {
                y0 = y0.min(r.y);
                y1 = y1.max(r.bottom());
            }
        }
        // About one cell for every four commands, each at least 32 CSS px
        // across: a grid of the frame's 920 cells cost more to fill (a
        // backdrop is filed in every cell) than a HUD of a few dozen
        // commands cost to test pair by pair (tests/evidence/r4/perf-ui).
        let cells = (bounds.len() / 4).clamp(1, 64 * 64) as f32;
        let area = ((x1 - x0).max(1.0) * (y1 - y0).max(1.0)).max(1.0);
        let side = (area / cells).sqrt().max(32.0);
        let axis = |lo: f32, hi: f32| {
            if hi <= lo {
                return (0.0, 1.0, 1);
            }
            let n = ((hi - lo) / side).ceil().clamp(1.0, 64.0);
            (lo, (hi - lo) / n, n as usize)
        };
        let (x, cell_w, cols) = axis(x0, x1);
        let (y, cell_h, rows) = axis(y0, y1);
        Grid {
            x,
            y,
            cell_w,
            cell_h,
            cols,
            rows,
        }
    }

    /// The columns and rows `r` touches.
    fn span(
        &self,
        r: &Rect,
    ) -> (
        std::ops::RangeInclusive<usize>,
        std::ops::RangeInclusive<usize>,
    ) {
        // `as` saturates (NaN is 0), and the clamp keeps every index in the grid.
        let at = |v: f32, o: f32, size: f32, n: usize| {
            (((v - o) / size).floor() as i64).clamp(0, n as i64 - 1) as usize
        };
        (
            at(r.x, self.x, self.cell_w, self.cols)..=at(r.right(), self.x, self.cell_w, self.cols),
            at(r.y, self.y, self.cell_h, self.rows)
                ..=at(r.bottom(), self.y, self.cell_h, self.rows),
        )
    }
}

/// Where a frame's shapes and text go.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EmitDesc {
    /// The overlay layer of painter level 0; level n draws on `layer + n`.
    /// Put the UI above other overlay sprites with a high base. A UI layer
    /// ([`DrawCmd::BeginLayer`]) takes [`LAYER_SPAN`] overlay layers of its
    /// own from `layer` + [`LAYER_BASE`].
    pub layer: i32,
    /// The UI clock in seconds (`Ui::seconds`): patterns scroll and layer
    /// materials move by it.
    pub time: f32,
}

/// Past the main levels: where the UI's offscreen layers take their
/// overlay layers.
pub const LAYER_BASE: i32 = 1 << 19;
/// The overlay layers of one offscreen layer (its painter levels).
pub const LAYER_SPAN: i32 = 4096;
/// The most offscreen layers one frame draws; past them a layer's subtree
/// draws in place.
pub const MAX_LAYERS: u32 = 120;

/// A rectangle as og-render's `[x0, y0, width, height]` key.
fn key(r: &Rect) -> [u32; 4] {
    [
        r.x.to_bits(),
        r.y.to_bits(),
        r.width.to_bits(),
        r.height.to_bits(),
    ]
}

/// A cut to a shape in force: its geometry, its own record in the UI's
/// clip list and the record of the cut under it.
struct Cut {
    id: u32,
    record: u32,
    below: Option<u32>,
    geometry: UiGeometry,
    center: Vec2,
    half: Vec2,
    transform: [f32; 4],
}

/// A clip rectangle's key: its bits, the cut and the depth test in force.
type ClipKey = ([u32; 4], Option<u32>, Option<u32>);

struct Clips {
    known: std::collections::HashMap<ClipKey, u32>,
    cuts: Vec<Cut>,
    next_cut: u32,
    /// Depth tests in force (`WorldCmd::Occlude`): id, point, depth, alpha.
    occluders: Vec<(u32, Vec2, f32, f32)>,
}

impl Clips {
    /// The overlay clip index of a rectangle under the cuts in force,
    /// adding it (and its cut's shape) the first time.
    fn index(&mut self, overlay: &mut Overlay, clip: &Rect) -> u32 {
        let top = self.cuts.last();
        let occ = self.occluders.last().copied();
        let k = (key(clip), top.map(|c| c.id), occ.map(|o| o.0));
        if let Some(i) = self.known.get(&k) {
            return *i;
        }
        let i = overlay.add_clip([clip.x, clip.y, clip.width, clip.height]);
        if let Some((_, at, depth, alpha)) = occ {
            overlay.ui.occluders.push(og_render::UiOccluder {
                rect: i,
                at,
                depth,
                alpha,
                bias: 0.25,
                soft: 0.5,
            });
        }
        if let Some(c) = top {
            overlay.ui.clips.push(og_render::UiClip {
                rect: i,
                geometry: c.geometry.clone(),
                center: c.center,
                half: c.half,
                transform: c.transform,
                parent: c.below,
            });
        }
        self.known.insert(k, i);
        i
    }

    /// Starts a cut, its shape under `m`: its own record (on a clip
    /// rectangle of its own) is the parent of the cuts inside it.
    fn push(
        &mut self,
        overlay: &mut Overlay,
        geometry: UiGeometry,
        center: Vec2,
        half: Vec2,
        m: &Affine,
    ) {
        let below = self.cuts.last().map(|c| c.record);
        let rect = overlay.add_clip([-1e9, -1e9, 2e9, 2e9]);
        let (cx, cy) = m.apply(center.x, center.y);
        let center = Vec2::new(cx, cy);
        let transform = linear(m);
        overlay.ui.clips.push(og_render::UiClip {
            rect,
            geometry: geometry.clone(),
            center,
            half,
            transform,
            parent: below,
        });
        self.next_cut += 1;
        self.cuts.push(Cut {
            id: self.next_cut,
            record: overlay.ui.clips.len() as u32 - 1,
            below,
            geometry,
            center,
            half,
            transform,
        });
    }
}

/// A space of levels: its base overlay layer and its UI layer.
struct Space {
    base: i32,
    layer: Option<u32>,
}

/// A UI shape, laid on `layer` and clip `clip`.
fn placed(mut shape: UiShape, layer: i32, clip: u32) -> UiShape {
    shape.layer = layer;
    shape.clip = Some(clip);
    shape
}

fn rect_shape(rect: Rect, radius: f32, color: [f32; 4]) -> UiShape {
    UiShape::rect([rect.x, rect.y, rect.width, rect.height], radius, color)
}

/// `item` moved by `d` CSS px: its map translated, its clip and depth
/// test point with it.
/// A frame's commands drawn `unit` times larger (the og-ui unit, docs/ui.md
/// §15.2): laid out in units, emitted in CSS px. A command that keeps
/// rectangles upright takes the scale into its values ([`crate::ui::bake`]);
/// a turned, skewed or UI-pass command takes it over its map, its
/// screen-space clip, layer rectangle and points scaled with it. Blur
/// radii, a mass element's size and where screen-space ones stand grow too.
pub(crate) fn zoom_items(mut items: Vec<DrawItem>, unit: f32) -> Vec<DrawItem> {
    if unit == 1.0 {
        return items;
    }
    let s = Affine::scale(unit, unit);
    for item in &mut items {
        let cmd = std::mem::replace(&mut item.cmd, DrawCmd::PopClip);
        let (mut cmd, rest) = if item.transform.is_identity() {
            crate::ui::bake(cmd, s)
        } else {
            (cmd, s.mul(&item.transform))
        };
        // Not baked: its clip is the screen's, and the screen is scaled.
        if !rest.is_identity()
            && let Some(c) = cmd.clip_mut()
        {
            *c = s.map_rect(c);
        }
        match &mut cmd {
            DrawCmd::Glass { blur, .. } => *blur *= unit,
            DrawCmd::Shape { shape, .. } => {
                if let UiMode::Backdrop { blur, .. } = &mut shape.mode {
                    *blur *= unit;
                }
            }
            DrawCmd::BeginLayer { rect, expand, .. } => {
                *rect = s.map_rect(rect);
                *expand *= unit;
            }
            DrawCmd::World(w) => match &mut **w {
                WorldCmd::Occlude { at, .. } => *at = (at.0 * unit, at.1 * unit),
                WorldCmd::Mass { batch, bounds, .. } => {
                    *bounds = s.map_rect(bounds);
                    zoom_mass(batch, unit);
                }
                WorldCmd::BeginPanel { rect, .. } => *rect = s.map_rect(rect),
                _ => {}
            },
            _ => {}
        }
        item.cmd = cmd;
        item.transform = rest;
    }
    items
}

/// A mass batch drawn `unit` times larger ([`zoom_items`]): each
/// instance's size, and for screen-space ones where it stands and how far
/// it travels.
pub(crate) fn zoom_mass(b: &mut og_render::UiMassBatch, unit: f32) {
    let screen = b.space == og_render::UiMassSpace::Screen;
    for i in &mut b.instances {
        i.scale *= unit;
        if screen {
            i.position[0] *= unit;
            i.position[1] *= unit;
            i.aux[0] *= unit;
            i.aux[1] *= unit;
            i.aux[3] *= unit;
            i.velocity[0] *= unit;
            i.velocity[1] *= unit;
        }
    }
}

fn shifted(item: &DrawItem, d: Vec2) -> DrawItem {
    let mut out = item.clone();
    out.transform = Affine::translate(d.x, d.y).mul(&item.transform);
    if let Some(c) = out.cmd.clip_mut() {
        c.x += d.x;
        c.y += d.y;
    }
    if let DrawCmd::World(w) = &mut out.cmd
        && let WorldCmd::Occlude { at, .. } = &mut **w
    {
        *at = (at.0 + d.x, at.1 + d.y);
    }
    out
}

pub(crate) fn emit(
    items: &[DrawItem],
    desc: &EmitDesc,
    overlay: &mut Overlay,
    shift: &dyn Fn(u64) -> Option<Vec2>,
) {
    // The anchored elements open (`WorldCmd::Anchor`): each one's shift.
    let mut anchors: Vec<Vec2> = Vec::new();
    let mut clips = Clips {
        known: std::collections::HashMap::new(),
        cuts: Vec::new(),
        next_cut: 0,
        occluders: Vec::new(),
    };
    overlay.ui.time = desc.time;
    let mut spaces = vec![Space {
        base: desc.layer,
        layer: None,
    }];
    for item in items {
        if let DrawCmd::World(w) = &item.cmd {
            match **w {
                WorldCmd::Anchor { key } => {
                    let outer = anchors.last().copied().unwrap_or(Vec2::ZERO);
                    anchors.push(outer + shift(key).unwrap_or(Vec2::ZERO));
                    continue;
                }
                WorldCmd::EndAnchor => {
                    anchors.pop();
                    continue;
                }
                _ => {}
            }
        }
        let moved;
        let item = match anchors.last() {
            Some(&d) if d != Vec2::ZERO => {
                moved = shifted(item, d);
                &moved
            }
            _ => item,
        };
        let space = spaces.last().expect("the frame's space");
        let layer = space.base + item.level;
        let m = &item.transform;
        // A shape laid on this item's layer and clip, under its map.
        let put = |overlay: &mut Overlay, s: UiShape, c: u32| {
            overlay.ui.shapes.push(placed(mapped_shape(s, m), layer, c));
        };
        match &item.cmd {
            DrawCmd::PushClip {
                geometry,
                center,
                half,
            } => clips.push(
                overlay,
                geometry.clone(),
                Vec2::new(center.0, center.1),
                Vec2::new(half.0, half.1),
                m,
            ),
            DrawCmd::PopClip => {
                clips.cuts.pop();
            }
            DrawCmd::World(w) => match &**w {
                WorldCmd::Occlude { at, depth, alpha } => {
                    clips.next_cut += 1;
                    let id = clips.next_cut;
                    clips
                        .occluders
                        .push((id, Vec2::new(at.0, at.1), *depth, *alpha));
                }
                WorldCmd::EndOcclude => {
                    clips.occluders.pop();
                }
                WorldCmd::Mass { batch, clip, .. } => {
                    let c = clips.index(overlay, clip);
                    overlay.ui.mass.push(og_render::UiMassBatch {
                        layer,
                        clip: Some(c),
                        ..(**batch).clone()
                    });
                }
                WorldCmd::BeginPanel { target, rect } => {
                    let k = overlay.ui.layers.len() as u32;
                    if k >= MAX_LAYERS {
                        // No room: its items draw nowhere visible (off the screen).
                        spaces.push(Space {
                            base: layer,
                            layer: None,
                        });
                        continue;
                    }
                    let base = desc.layer + LAYER_BASE + k as i32 * LAYER_SPAN;
                    overlay.ui.layers.push(og_render::UiLayer {
                        capture: [base, base + LAYER_SPAN - 1],
                        rect: [rect.x, rect.y, rect.width, rect.height],
                        material: UiMaterial::Plain,
                        params: [[0.0; 4]; og_render::UI_MATERIAL_PARAMS],
                        mask: None,
                        target: Some(*target),
                    });
                    spaces.push(Space {
                        base,
                        layer: Some(k),
                    });
                }
                WorldCmd::EndPanel => {
                    // Composited nowhere: the mesh shows its target.
                    spaces.pop();
                }
                // Read above.
                WorldCmd::Anchor { .. } | WorldCmd::EndAnchor => {}
            },
            DrawCmd::BeginLayer {
                material,
                params,
                mask,
                rect,
                ..
            } => {
                let k = overlay.ui.layers.len() as u32;
                if k >= MAX_LAYERS || rect.is_empty() {
                    // Drawn in place, over what its composite would be over.
                    spaces.push(Space {
                        base: layer,
                        layer: None,
                    });
                    continue;
                }
                let base = desc.layer + LAYER_BASE + k as i32 * LAYER_SPAN;
                overlay.ui.layers.push(og_render::UiLayer {
                    capture: [base, base + LAYER_SPAN - 1],
                    rect: [rect.x, rect.y, rect.width, rect.height],
                    material: *material,
                    params: *params,
                    mask: mask.clone(),
                    target: None,
                });
                spaces.push(Space {
                    base,
                    layer: Some(k),
                });
            }
            DrawCmd::EndLayer {
                geometry,
                rect,
                opacity,
                blend,
                clip,
            } => {
                let Some(done) = spaces.pop() else {
                    continue;
                };
                let Some(k) = done.layer else {
                    continue;
                };
                let parent = spaces.last().map_or(desc.layer, |s| s.base);
                let [lx, ly, lw, lh] = overlay.ui.layers[k as usize].rect;
                // A composite's shape is the element's, under its map; the
                // whole layer is its rectangle on the screen.
                let (geometry, r, map) = match geometry {
                    Some(g) => (g.clone(), *rect, *m),
                    None => (
                        UiGeometry::Rect {
                            radii: [0.0; 4],
                            corner: og_render::UiCorner::Round,
                        },
                        Rect::new(lx, ly, lw, lh),
                        Affine::IDENTITY,
                    ),
                };
                let c = clips.index(overlay, clip);
                let s = UiShape {
                    geometry,
                    center: Vec2::new(r.x + r.width / 2.0, r.y + r.height / 2.0),
                    half: Vec2::new(r.width / 2.0, r.height / 2.0),
                    transform: [1.0, 0.0, 0.0, 1.0],
                    paint: UiPaint::Layer(k),
                    mode: UiMode::Fill,
                    opacity: *opacity,
                    blend: *blend,
                    layer: parent + item.level,
                    clip: Some(c),
                };
                overlay.ui.shapes.push(mapped_shape(s, &map));
            }
            DrawCmd::Glass {
                rect,
                radius,
                blur,
                saturation,
                tint,
                material,
                alpha,
                clip,
            } => {
                let c = clips.index(overlay, clip);
                let mut s = rect_shape(*rect, *radius, *tint);
                s.mode = UiMode::Backdrop {
                    blur: *blur,
                    saturation: *saturation,
                    brightness: 1.0,
                    glass: *material,
                };
                s.opacity = *alpha;
                put(overlay, s, c);
            }
            DrawCmd::Quad {
                rect,
                radius,
                color,
                clip,
            } => {
                if rect.is_empty() {
                    continue;
                }
                let c = clips.index(overlay, clip);
                put(overlay, rect_shape(*rect, *radius, *color), c);
            }
            DrawCmd::Image {
                rect,
                texture,
                uv,
                color,
                clip,
            } => {
                let c = clips.index(overlay, clip);
                let mut s = rect_shape(*rect, 0.0, *color);
                if let Some(t) = texture {
                    s.paint = UiPaint::Image(og_render::UiImage {
                        texture: *t,
                        uv: *uv,
                        size: Vec2::new(rect.width, rect.height),
                        tint: *color,
                        fit: og_render::UiFit::Fill,
                        focal: Vec2::splat(0.5),
                    });
                }
                put(overlay, s, c);
            }
            DrawCmd::Bar {
                center,
                w,
                h,
                angle,
                color,
                clip,
            } => {
                let c = clips.index(overlay, clip);
                let (sin, cos) = angle.sin_cos();
                let mut s = rect_shape(
                    Rect::new(center.0 - w / 2.0, center.1 - h / 2.0, *w, *h),
                    0.0,
                    *color,
                );
                s.transform = [cos, sin, -sin, cos];
                put(overlay, s, c);
            }
            DrawCmd::Half {
                center,
                radius,
                angle,
                color,
                clip,
            } => {
                let c = clips.index(overlay, clip);
                let mut s = rect_shape(
                    Rect::new(
                        center.0 - radius,
                        center.1 - radius,
                        2.0 * radius,
                        2.0 * radius,
                    ),
                    0.0,
                    *color,
                );
                s.geometry = UiGeometry::Arc {
                    start: angle - std::f32::consts::FRAC_PI_2,
                    sweep: std::f32::consts::PI,
                    thickness: *radius,
                    cap: og_render::UiCap::Butt,
                };
                put(overlay, s, c);
            }
            DrawCmd::Shape { shape, clip } => {
                let c = clips.index(overlay, clip);
                put(overlay, (**shape).clone(), c);
            }
            DrawCmd::Vector {
                vector,
                rect,
                transform,
                tint,
                clip,
            } => {
                let c = clips.index(overlay, clip);
                // Its rectangle's centre moved by the map, its local space
                // (about that centre) composed with the map's linear part.
                let (cx, cy) = m.apply(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
                overlay.ui.vectors.push(og_render::UiVectorDraw {
                    vector: *vector,
                    rect: [
                        cx - rect.width / 2.0,
                        cy - rect.height / 2.0,
                        rect.width,
                        rect.height,
                    ],
                    transform: mul2(linear(m), *transform),
                    tint: *tint,
                    layer,
                    clip: Some(c),
                });
            }
            DrawCmd::Text { .. } => {
                let at = overlay.texts.len() as u32;
                if emit_text(&item.cmd, layer, &mut clips, overlay) && !m.is_identity() {
                    // No effects: the map alone.
                    overlay.ui.text_effects.push(og_render::UiTextEffects {
                        text: at,
                        transform: linear(m),
                        ..Default::default()
                    });
                    map_text(overlay, at, m);
                }
            }
            DrawCmd::StyledText { text, effects } => {
                let at = overlay.texts.len() as u32;
                if emit_text(text, layer, &mut clips, overlay) {
                    let mut fx = (**effects).clone();
                    fx.text = at;
                    if !m.is_identity() {
                        fx.transform = linear(m);
                        let b = Rect::new(
                            fx.fill_box[0],
                            fx.fill_box[1],
                            fx.fill_box[2],
                            fx.fill_box[3],
                        );
                        let b = m.map_rect(&b);
                        fx.fill_box = [b.x, b.y, b.width, b.height];
                        map_text(overlay, at, m);
                    }
                    overlay.ui.text_effects.push(fx);
                }
            }
        }
    }
}

/// A styled text's effects under an upright map (translation and scale)
/// baked into its text: offsets scaled per axis, widths by the text's
/// height scale (as its size), the gradient's box moved.
pub(crate) fn scale_effects(fx: &mut og_render::UiTextEffects, m: &Affine) {
    let sy = m.d.abs();
    for s in &mut fx.shadows {
        s.offset = Vec2::new(s.offset.x * m.a, s.offset.y * m.d);
        s.blur *= sy;
    }
    if let Some(o) = &mut fx.outline {
        o.width *= sy;
    }
    let b = Rect::new(
        fx.fill_box[0],
        fx.fill_box[1],
        fx.fill_box[2],
        fx.fill_box[3],
    );
    let b = m.map_rect(&b);
    fx.fill_box = [b.x, b.y, b.width, b.height];
}

/// Moves text `at`'s pen by the map: its glyphs take the map's linear part
/// from its [`og_render::UiTextEffects::transform`].
fn map_text(overlay: &mut Overlay, at: u32, m: &Affine) {
    let t = &mut overlay.texts[at as usize];
    let (x, y) = m.apply(t.position.x, t.position.y);
    t.position.x = x;
    t.position.y = y;
}

/// Pushes a text command's text; false when it is not one.
fn emit_text(cmd: &DrawCmd, layer: i32, clips: &mut Clips, overlay: &mut Overlay) -> bool {
    let DrawCmd::Text {
        x,
        baseline,
        size,
        text,
        color,
        font,
        clip,
        tracking,
        weight,
        glow,
        placements,
        ..
    } = cmd
    else {
        return false;
    };
    let c = clips.index(overlay, clip);
    overlay.texts.push(Text {
        font: *font,
        text: text.clone(),
        position: og_core::glam::Vec3::new(*x, *baseline, 0.0),
        size: *size,
        rotation: 0.0,
        color: *color,
        layer,
        clip: Some(c),
        tracking: *tracking,
        weight: *weight,
        glow: glow.map(|(color, radius)| og_render::TextGlow { color, radius }),
        placements: placements.clone(),
    });
    true
}

#[cfg(test)]
mod levels {
    use super::*;

    fn plain(cmds: Vec<DrawCmd>) -> Vec<(DrawCmd, Affine)> {
        cmds.into_iter().map(|c| (c, Affine::IDENTITY)).collect()
    }

    /// The levels as every command tested against every earlier one (what
    /// the grid must answer).
    fn quadratic(cmds: &[DrawCmd]) -> Vec<i32> {
        let mut placed: Vec<(Rect, Batch, i32)> = Vec::new();
        let mut out = Vec::new();
        for cmd in cmds {
            let bounds = cmd.bounds();
            if bounds.is_empty() {
                continue;
            }
            let batch = cmd.batch();
            let mut level = 0;
            for (r, b, l) in &placed {
                if r.overlaps(&bounds) {
                    level = level.max(if *b == batch { *l } else { *l + 1 });
                }
            }
            placed.push((bounds, batch, level));
            out.push(level);
        }
        out
    }

    const ALL: Rect = Rect {
        x: -1e9,
        y: -1e9,
        width: 2e9,
        height: 2e9,
    };

    fn quad(rect: Rect) -> DrawCmd {
        DrawCmd::Quad {
            rect,
            radius: 0.0,
            color: [1.0; 4],
            clip: ALL,
        }
    }

    fn image(rect: Rect, slot: u32) -> DrawCmd {
        DrawCmd::Image {
            rect,
            texture: og_render::TextureHandle::from_slot(slot),
            uv: [0.0, 0.0, 1.0, 1.0],
            color: [1.0; 4],
            clip: ALL,
        }
    }

    #[test]
    fn the_grid_assigns_the_levels_of_every_pair_tested() {
        let mut seed = 0x2545_f491_4f6c_dd1d_u64;
        let mut rand = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed >> 11) as f32 / (1u64 << 53) as f32
        };
        let mut cases: Vec<Vec<DrawCmd>> = Vec::new();
        // Random rectangles of two batches, some touching edge to edge, some
        // far off the frame, some empty or not finite.
        for n in [1, 7, 60, 400] {
            let cmds = (0..n)
                .map(|i| {
                    let (x, y) = (rand() * 1400.0 - 60.0, rand() * 800.0 - 40.0);
                    let (w, h) = (rand() * 300.0, rand() * 200.0);
                    let r = match i % 23 {
                        0 => Rect::new(x, y, 0.0, h),
                        1 => Rect::new(f32::NAN, y, w, h),
                        2 => Rect::new(x, y, f32::INFINITY, h),
                        3 => Rect::new(-1e7, y, 2e7, h),
                        4 => Rect::new((x / 24.0).floor() * 24.0, 0.0, 24.0, 24.0),
                        _ => Rect::new(x, y, w, h),
                    };
                    if i % 3 == 0 { image(r, i % 2) } else { quad(r) }
                })
                .collect();
            cases.push(cmds);
        }
        // A screen under a dissolve's grid of cells, as a transition draws it.
        let mut screen = vec![quad(Rect::new(0.0, 0.0, 1280.0, 720.0))];
        screen.extend((0..20).map(|i| image(Rect::new(40.0 * i as f32, 30.0, 120.0, 40.0), 0)));
        for y in 0..30 {
            for x in 0..54 {
                screen.push(quad(Rect::new(
                    x as f32 * 24.0,
                    y as f32 * 24.0,
                    24.0,
                    24.0,
                )));
            }
        }
        cases.push(screen);
        for cmds in cases {
            let want = quadratic(&cmds);
            let got: Vec<i32> = assign_levels(plain(cmds)).iter().map(|i| i.level).collect();
            assert_eq!(got, want);
        }
    }

    fn begin() -> DrawCmd {
        DrawCmd::BeginLayer {
            material: UiMaterial::Plain,
            params: [[0.0; 4]; og_render::UI_MATERIAL_PARAMS],
            mask: None,
            expand: 4.0,
            rect: Rect::default(),
        }
    }

    fn end() -> DrawCmd {
        DrawCmd::EndLayer {
            geometry: None,
            rect: Rect::default(),
            opacity: 1.0,
            blend: UiBlend::Normal,
            clip: ALL,
        }
    }

    /// A layer levels its own commands from 0 and is one command of its
    /// parent, over what it overlaps there; its rectangle is what it draws.
    #[test]
    fn a_layer_is_one_command_of_its_parent_with_levels_of_its_own() {
        let cmds = vec![
            quad(Rect::new(0.0, 0.0, 100.0, 100.0)),
            image(Rect::new(10.0, 10.0, 20.0, 20.0), 1),
            begin(),
            quad(Rect::new(20.0, 20.0, 40.0, 40.0)),
            image(Rect::new(25.0, 25.0, 10.0, 10.0), 1),
            quad(Rect::new(30.0, 30.0, 10.0, 10.0)),
            end(),
            quad(Rect::new(50.0, 50.0, 10.0, 10.0)),
        ];
        let items = assign_levels(plain(cmds));
        let levels: Vec<i32> = items.iter().map(|i| i.level).collect();
        // Outer: quad 0, image 1 (a texture is its own batch), layer 2 (over
        // the image), quad 3 (over the layer); inside: quad 0, image 1, quad 2.
        assert_eq!(levels, [0, 1, 2, 0, 1, 2, 2, 3]);
        let DrawCmd::BeginLayer { rect, .. } = &items[2].cmd else {
            panic!("{:?}", items[2]);
        };
        assert_eq!(*rect, Rect::new(16.0, 16.0, 48.0, 48.0));
    }

    /// The emitted layer captures its own range of overlay layers, and its
    /// composite draws at its level in the parent; a cut adds a clip shape
    /// for what is inside it, nested in the cut around it.
    #[test]
    fn layers_and_cuts_emit_their_ranges_and_shapes() {
        let rounded = |r: f32| UiGeometry::Rect {
            radii: [r; 4],
            corner: og_render::UiCorner::Round,
        };
        let cmds = vec![
            quad(Rect::new(0.0, 0.0, 100.0, 100.0)),
            DrawCmd::PushClip {
                geometry: rounded(10.0),
                center: (50.0, 50.0),
                half: (50.0, 50.0),
            },
            DrawCmd::PushClip {
                geometry: rounded(4.0),
                center: (50.0, 50.0),
                half: (20.0, 20.0),
            },
            begin(),
            quad(Rect::new(40.0, 40.0, 20.0, 20.0)),
            end(),
            DrawCmd::PopClip,
            DrawCmd::PopClip,
            quad(Rect::new(0.0, 0.0, 10.0, 10.0)),
        ];
        let items = assign_levels(plain(cmds));
        let mut overlay = Overlay::new(200.0, 200.0, 1.0);
        emit(
            &items,
            &EmitDesc {
                layer: 100,
                time: 2.5,
            },
            &mut overlay,
            &|_| None,
        );
        let ui = &overlay.ui;
        assert_eq!(ui.time, 2.5);
        assert_eq!(ui.layers.len(), 1);
        let base = 100 + LAYER_BASE;
        assert_eq!(ui.layers[0].capture, [base, base + LAYER_SPAN - 1]);
        // Background, the layer's quad, its composite, the corner quad.
        assert_eq!(ui.shapes.len(), 4);
        assert_eq!(ui.shapes[1].layer, base);
        assert_eq!(ui.shapes[2].paint, UiPaint::Layer(0));
        assert_eq!(ui.shapes[2].layer, 101);
        // Two cuts' own records, and the shape of each rectangle cut: the
        // layer's quad and its composite inside both cuts.
        let inner = ui.shapes[1].clip.expect("clipped");
        let shape = ui.clips.iter().find(|c| c.rect == inner).expect("its cut");
        assert_eq!(shape.geometry, rounded(4.0));
        let outer = shape.parent.expect("nested") as usize;
        assert_eq!(ui.clips[outer].geometry, rounded(10.0));
        // The corner quad, after both cuts end, is cut by neither.
        let after = ui.shapes[3].clip.expect("a rectangle");
        assert!(ui.clips.iter().all(|c| c.rect != after));
    }
}
