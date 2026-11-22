//! Decimation by edge collapse with quadric error metrics (Garland and
//! Heckbert 1997), and LOD chains. Written here, in f64, deterministic
//! (`docs/research.md` §9 says why not meshoptimizer-rs).
//!
//! - Each vertex carries the area-weighted quadric of its triangles'
//!   planes. Open borders and features (edges between surfaces, between
//!   different face names, named edges) add a heavy quadric of the plane
//!   through the edge square to its face, so they stay in place.
//! - Vertices are interior, on one border or feature line, or corners
//!   (where lines meet or end, on a non-manifold edge, or where two fans
//!   of triangles meet at a point). A line vertex
//!   only slides along its line into the next vertex on it; a corner never
//!   goes. `lock_border` makes every open-border vertex a corner.
//! - A collapse goes to the kept vertex's position, or for two interior
//!   vertices to whichever of the two ends and the midpoint costs least.
//! - A collapse is refused when it would break the link condition (the
//!   two ends share a neighbour that is not across one of the edge's
//!   triangles), so a closed manifold stays closed and manifold; when it
//!   would turn a triangle over (its normal turns by more than 90°) or
//!   leave it with no area; and beyond `max_error`.
//! - A collapse is refused when it would make the surface pass through
//!   itself: each triangle it rewrites is tested, exactly
//!   (`crate::boolean::selfcheck::crossing`), against the triangles near it,
//!   found in a spatial hash over the triangles' boxes ([`Grid`]), and
//!   against the other rewritten ones. On an input that already crosses
//!   itself, a collapse that keeps a crossing is taken only when it adds
//!   no crossing partner and no pair: the count never rises.
//! - Collapses are taken cheapest first; ties go to the smaller edge key,
//!   then the smaller vertex. The order depends only on the input; a
//!   refused collapse is only skipped, so where none would fold the
//!   result is the one without the test.
//! - When the target is not reached, the report says why: each edge left
//!   is counted under the first rule that refuses its cheapest collapse
//!   ([`Blocked`]).
//!
//! Skins are carried: each collapse gives the kept vertex the blend of
//! the two ends' weights by where it lands (`Weights::blend`: its own
//! weights at its own position, half and half at the midpoint). Bone
//! borders are kept by a penalty, not a ban: removing a vertex costs, on
//! top of its quadric error, the error of moving the surface by its
//! largest weight difference to any neighbour (on any bone) times 0.5 %
//! of the bounding-box diagonal. A joint so keeps the rows of vertices it
//! bends with while the triangle target is still met (a ban cannot meet
//! it). The reported `error` leaves the penalty out.
//! `Decimate::cross_bone_borders` drops the penalty; the cost of keeping
//! borders, in surface error at the same triangle count, is measured in
//! `docs/research.md` §11.5.
//!
//! Names: every triangle left is an input triangle with corners moved, so
//! it keeps its surface and face names. Edge names follow their vertices
//! and survive on edges that still exist.
//!
//! UVs (`Solid::uvs`, per triangle corner) are carried. A UV seam (an
//! edge whose two triangles give either end different UVs) is a feature
//! line like a surface border, so a seam vertex only slides along its
//! seam and the seam stays where it was. A collapse gives each corner of
//! the removed vertex the UV the kept vertex has on the same side of any
//! seam (matched through the edge's own triangles); a collapse to the
//! midpoint gives both ends' corners the mean of their UVs. A collapse
//! whose corners cannot be matched (more than one UV side on one end, no
//! seam to say which) or that would turn a UV triangle over or flatten it
//! (`orient2d`, exact) is refused, as a turned-over face is. With
//! `Decimate::merge_charts`, islands that agree across a seam up to a
//! rigid move of the UV plane are one chart ([`merge_charts`]): that seam
//! is no feature line, and a corner across it takes its new UV through
//! the two islands' moves (`docs/research.md` §32).
//!
//! Vertex colours (`Solid::colors`, per corner) are carried with the UVs:
//! a corner's UV and colour are matched together, so a hard colour
//! border (an edge whose two triangles give either end different
//! colours) is a feature line as a UV seam is, and a collapse gives the
//! removed vertex's corners the kept vertex's colour on the same side (the
//! mean at a midpoint). A seam inside a merged chart is still a feature
//! line where the colours differ across it, and a corner across it takes
//! its colour from its own side, not through the chart's move. A smooth
//! colour costs nothing extra: a penalty
//! for removing a vertex whose colour differs from its ring's (as a bone
//! border's, §11.5) raised both the surface error and the colour error
//! on the generated meshes measured (`docs/research.md` §29.6).

use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap};
use std::fmt;

use crate::boolean::selfcheck::{crossing, degenerate};
use crate::math::predicates::orient2d;
use crate::math::{DVec2, DVec3};
use crate::mesh::{CornerColors, EdgeKey, Skin, Solid, Weights, edge_key};

use crate::tools::repair::drop_unused;

mod charts;
mod collapse;
mod guard;
mod mesh;
mod quadric;
pub use charts::*;
pub(crate) use collapse::*;
pub(crate) use guard::*;
pub(crate) use mesh::*;
pub(crate) use quadric::*;

/// How far to decimate. The zero value asks for nothing and is an error;
/// set `ratio` or `target`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Decimate {
    /// Keep this fraction of the triangles (0 < ratio ≤ 1); 0 is unset.
    pub ratio: f64,
    /// Keep at most this many triangles; 0 is unset. With `ratio` too,
    /// the smaller count wins.
    pub target: usize,
    /// Refuse any collapse that would move the surface more than this, in
    /// mm (the root-mean-square distance its quadric measures); 0 is no
    /// limit.
    pub max_error: f64,
    /// Never move vertices on open borders (holes, the edge of a sheet).
    pub lock_border: bool,
    /// On a skinned solid, collapse across bone-weight borders as freely as
    /// anywhere (off by default: removing a vertex whose weights differ
    /// from a neighbour's costs extra, so joints keep their rows).
    pub cross_bone_borders: bool,
    /// On a solid with UVs: UV islands that meet along an edge and agree
    /// there up to a rigid move in UV space (a rotation and a translation,
    /// not a mirror), within this distance in UV units, are one chart for
    /// the seam rule ([`merge_charts`]): their shared seam is not a
    /// feature line, and a corner across it gets its UV through the move.
    /// The texture is not rewritten. 0: off, every seam stays.
    pub merge_charts: f64,
    /// On a solid with vertex colours, collapse across hard colour
    /// borders as freely as anywhere (off by default: a border is a
    /// feature line, as a UV seam is). A removed vertex's corners then
    /// take the kept vertex's colour on the collapsed edge (the mean of
    /// its colours there when they differ; at a midpoint, the mean with
    /// their own), so a border moves and blurs by up to an edge. For a
    /// mesh whose colours were sampled per vertex, where borders mark no
    /// feature (`docs/research.md` §33).
    pub cross_color_borders: bool,
}

/// What a decimation did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DecimateReport {
    /// Triangles before.
    pub triangles_before: usize,
    /// Triangles after.
    pub triangles: usize,
    /// The count asked for.
    pub target: usize,
    /// The largest error of a collapse taken, mm (root-mean-square
    /// distance to the planes the collapsed vertices stood for).
    pub error: f64,
    /// UV islands (triangles joined across edges whose two sides give
    /// both ends the same UVs), and the charts they make with
    /// `merge_charts` (the same count without it). 0 without UVs.
    pub uv_islands: usize,
    /// See `uv_islands`.
    pub uv_charts: usize,
    /// Collapses not taken because the surface would have passed through
    /// itself (each attempt counts; an edge can be tried again after its
    /// neighbourhood changes).
    pub folds_refused: usize,
    /// When `triangles` is above `target`: the edges left, each counted
    /// under the rule that refuses its cheapest collapse. All zero when
    /// the target was reached.
    pub blocked: Blocked,
}

/// Why the edges left after a decimation that missed its target cannot
/// collapse: each under the first rule that refuses its cheapest
/// collapse, in this order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Blocked {
    /// Neither end may move: corners, or a border or feature line (a
    /// surface or name border, a named edge, a UV seam) the edge does not
    /// run along; or longer than the short-edge limit.
    pub fixed: usize,
    /// Beyond `max_error`.
    pub error: usize,
    /// The ends share a neighbour off the edge's triangles: collapsing
    /// would pinch the surface.
    pub link: usize,
    /// A triangle would turn over or lose its area.
    pub turn: usize,
    /// The UVs cannot follow: a corner's side of a UV seam is unclear, or
    /// a UV triangle would turn over or flatten.
    pub uv: usize,
    /// The surface would pass through itself.
    pub fold: usize,
    /// Collapsible now, but not tried again: its last try was refused,
    /// and what refused it changed in a collapse that re-planned only the
    /// edges at the vertex it kept.
    pub not_retried: usize,
}

/// Why a decimation could not run.
#[derive(Clone, Debug, PartialEq)]
pub enum DecimateError {
    /// Neither a ratio nor a target was given, or a value is out of range.
    BadDesc(String),
    /// A position is NaN or infinite.
    NotFinite,
    /// No longer returned: skins are carried (phase 5b part 2). Kept so
    /// that code matching on it still compiles.
    Skinned,
}

impl fmt::Display for DecimateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecimateError::BadDesc(w) => write!(f, "decimate: {w}"),
            DecimateError::NotFinite => f.write_str("decimate: a position is not finite"),
            DecimateError::Skinned => f.write_str(
                "decimate: the solid is skinned; decimating would break its skin, so it is refused",
            ),
        }
    }
}

impl std::error::Error for DecimateError {}

pub(crate) fn target_of(d: &Decimate, n: usize) -> Result<usize, DecimateError> {
    if !(d.ratio >= 0.0 && d.ratio <= 1.0) {
        return Err(DecimateError::BadDesc("ratio must be in 0..=1".into()));
    }
    if !(d.max_error >= 0.0 && d.max_error.is_finite()) {
        return Err(DecimateError::BadDesc(
            "max_error must be finite and >= 0".into(),
        ));
    }
    if !(d.merge_charts >= 0.0 && d.merge_charts.is_finite()) {
        return Err(DecimateError::BadDesc(
            "merge_charts must be finite and >= 0".into(),
        ));
    }
    let by_ratio = (d.ratio > 0.0).then(|| (d.ratio * n as f64).ceil() as usize);
    let by_target = (d.target > 0).then_some(d.target);
    match (by_ratio, by_target) {
        (None, None) => Err(DecimateError::BadDesc(
            "give a ratio or a triangle target".into(),
        )),
        (Some(a), Some(b)) => Ok(a.min(b)),
        (Some(a), None) | (None, Some(a)) => Ok(a),
    }
}

/// Decimates `s` towards the triangle count `d` asks for. The target may
/// not be reached when every remaining collapse is refused (features,
/// the link condition, the error limit); the report says how many
/// triangles are left. A solid already at or under the target comes back
/// unchanged.
pub fn decimate(s: &Solid, d: &Decimate) -> Result<(Solid, DecimateReport), DecimateError> {
    let n = s.triangles.len();
    let target = target_of(d, n)?;
    decimate_to(s, target, d)
}

/// Collapses every edge shorter than `length` mm that can go without
/// breaking the mesh (the same rules as [`decimate`]: the link condition,
/// no turned-over triangles, features slide only along themselves). For
/// the slivers a boolean leaves, whose vertices f32 export would merge
/// into non-manifold edges. Returns the solid and how many vertices went.
pub fn collapse_short_edges(s: &Solid, length: f64) -> Result<(Solid, usize), DecimateError> {
    if !(length >= 0.0 && length.is_finite()) {
        return Err(DecimateError::BadDesc(
            "an edge length must be finite and >= 0".into(),
        ));
    }
    if length == 0.0 || s.triangles.is_empty() {
        return Ok((s.clone(), 0));
    }
    let before = drop_unused(s).positions.len();
    let (out, _) = collapse(s, 0, &Decimate::default(), length)?;
    let gone = before - out.positions.len();
    Ok((out, gone))
}

pub(crate) fn decimate_to(
    s: &Solid,
    target: usize,
    d: &Decimate,
) -> Result<(Solid, DecimateReport), DecimateError> {
    collapse(s, target, d, 0.0)
}

/// A chain of levels of detail: level `k` decimates level `k − 1` (level
/// 0 decimates `s`) towards `levels[k]`. A ratio is taken of `s`'s
/// triangle count, so `[1, 0.5, 0.25]` gives the full mesh, a half and a
/// quarter.
pub fn lods(s: &Solid, levels: &[Decimate]) -> Result<Vec<(Solid, DecimateReport)>, DecimateError> {
    let n = s.triangles.len();
    let mut out: Vec<(Solid, DecimateReport)> = Vec::with_capacity(levels.len());
    for d in levels {
        let target = target_of(d, n)?;
        let prev = out.last().map_or(s, |l| &l.0);
        let (next, mut r) = decimate_to(prev, target, d)?;
        if let Some(l) = out.last() {
            r.error = r.error.max(l.1.error);
        }
        out.push((next, r));
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
