//! The UI's paint arena: what og-ui's tree paints, as flat `Pod` quads
//! og-render's paint pass draws with one pipeline (`ui_paint.wgsl`). The
//! tree refills it in place, so a repaint allocates nothing once its
//! vectors have grown; the renderer uploads it only when its generation
//! moves.
//!
//! A quad is a box in CSS pixels with its clip and colour and what fills
//! it ([`PaintKind`]): a solid rounded box, a gradient, an MSDF glyph of
//! one of [`Paint::fonts`] (resolved against the font's pages by the
//! renderer), or a built-in icon drawn as its stroked segments from
//! [`Paint::icons`], a table made once and shared.

use crate::FontHandle;
use bytemuck::{Pod, Zeroable};
use std::sync::Arc;

/// What a [`PaintQuad`] draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum PaintKind {
    /// A box rounded by `radius`, in `color`.
    Solid = 0,
    /// A box rounded by `radius`, filled by gradient `a` of
    /// [`Paint::gradients`], at `color`'s alpha.
    Gradient = 1,
    /// Glyph `a` (its char) of font `b` of [`Paint::fonts`]: `rect`'s x is
    /// the pen and y the baseline, w the size in CSS px; `radius` the
    /// weight's thickening, CSS px.
    Glyph = 2,
    /// Built-in icon segments `a..a + b` of [`Paint::icons`] (24-unit icon
    /// space, 2 units wide, round), in `rect`, in `color`.
    Icon = 3,
}

/// One quad of the arena (`struct PaintIn` in ui_paint.wgsl's terms
/// before the renderer resolves glyphs).
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct PaintQuad {
    /// x, y, width, height, CSS px.
    pub rect: [f32; 4],
    /// x0, y0, x1, y1, CSS px.
    pub clip: [f32; 4],
    /// Linear RGBA, straight alpha.
    pub color: [f32; 4],
    pub radius: f32,
    pub kind: u32,
    pub a: u32,
    pub b: u32,
}

/// A linear gradient: `stops` of [`Paint::stops`] from `first`, along the
/// CSS angle `angle` (radians, 0 up, clockwise).
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct PaintGradient {
    pub angle: f32,
    pub first: u32,
    pub count: u32,
    pub _pad: u32,
}

/// A gradient's stop: its offset (0..1) and its colour, linear RGBA.
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct PaintStop {
    pub color: [f32; 4],
    pub offset: f32,
    pub _pad: [f32; 3],
}

/// A built-in icon's stroke segment, `[ax, ay, bx, by]` in its 24-unit
/// space.
pub type IconSegment = [f32; 4];

/// The UI's paint: the quads in painter's order, their gradients, the
/// fonts their glyphs name and the built-in icons' segments.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Paint {
    /// The CSS px size it was laid out in (the overlay's: fitted to the
    /// output's height).
    pub width: f32,
    pub height: f32,
    pub quads: Vec<PaintQuad>,
    pub gradients: Vec<PaintGradient>,
    pub stops: Vec<PaintStop>,
    /// Each glyph font slot: the font's pages, first first (a run-time
    /// font adds pages as glyphs render; a glyph is drawn from the first
    /// page that has it).
    pub fonts: Vec<Vec<FontHandle>>,
    /// Every built-in icon's segments: one table, shared by every paint.
    pub icons: Option<Arc<[IconSegment]>>,
    /// Bumped each time anything above changes.
    pub generation: u64,
}

impl Paint {
    /// Empties it, keeping every vector's capacity and the icon table.
    pub fn clear(&mut self) {
        self.quads.clear();
        self.gradients.clear();
        self.stops.clear();
    }

    /// Adds a gradient of `stops` (offset, linear RGBA) at `angle`; its
    /// index for [`PaintKind::Gradient`].
    pub fn gradient(&mut self, angle: f32, stops: impl IntoIterator<Item = (f32, [f32; 4])>) -> u32 {
        let first = self.stops.len() as u32;
        self.stops.extend(stops.into_iter().map(|(offset, color)| PaintStop {
            color,
            offset,
            _pad: [0.0; 3],
        }));
        self.gradients.push(PaintGradient {
            angle,
            first,
            count: self.stops.len() as u32 - first,
            _pad: 0,
        });
        self.gradients.len() as u32 - 1
    }

    pub fn is_empty(&self) -> bool {
        self.quads.is_empty()
    }
}
