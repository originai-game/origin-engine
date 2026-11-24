//! Edits on named solids, after sand3d's verbs (`docs/design.md` §2).
//!
//! Entry: the functions below, each taking a [`Solid`] and a zero-value
//! desc and returning a new canonical solid or an [`EditError`].
//!
//! | edit | how | names it adds |
//! |---|---|---|
//! | [`split`] | intersect or cut with a half-space | `cut`, `split.*` |
//! | [`hole`] | cut cylinders (and a counterbore or countersink) from the entry point | `hole` on the mouth edges, `hole.side`, `hole.bottom` |
//! | [`pocket`] | cut a profile extruded from a face | `pocket.bottom`, `pocket.side` |
//! | [`seat`] | cut the volume a part fills | `cut`, `<part>.*` |
//! | [`shell`] | cut a solid offset inside the surface | `cut`, `shell.*` |
//! | [`dent`] | press the surface in, subdividing where needed | `dent` on moved faces |
//! | [`displace`] | move the surface along its normals by noise (fbm, ridged, craters) | none |
//! | [`merge`] | append, no boolean | faces as given |

use std::collections::BTreeMap;

use crate::boolean::{
    BoolError, BoolOp, BoolReport, Boolean, boolean, boolean_report, crossing_pairs, self_union,
};
use crate::generators::solids::{Extrude, GenError, Generator, extrude, textured};
use crate::math::DVec2;
use crate::math::{DVec3, Frame};
use crate::mesh::{CornerUvs, Solid, edge_key};
use crate::profile::Profile;
use robust::{Coord3D, orient3d};

mod dent;
pub(crate) mod displace;
mod hole;
mod merge;
mod pocket;
mod seat;
mod shell;
mod split;
pub use dent::*;
pub use displace::{Displace, MAX_OCTAVES, Noise, displace, displace_report};
pub use hole::*;
pub use merge::*;
pub use pocket::*;
pub use seat::*;
pub use shell::*;
pub use split::*;

/// Why an edit could not be honoured.
#[derive(Clone, Debug, PartialEq)]
pub enum EditError {
    /// The boolean underneath refused (see [`BoolError`]).
    Bool(BoolError),
    /// A tool could not be built.
    Gen(GenError),
    /// A size that must be positive was not (the field named).
    NotPositive(&'static str),
    /// An axis or normal of zero length (the field named).
    NoDirection(&'static str),
    /// The axis through a frame's origin, along its −z (into the part),
    /// meets no face: it runs past the part, or inside an opening through
    /// it (a through hole on the same axis).
    Misses {
        /// The frame's origin, as given.
        origin: DVec3,
        /// The axis's direction into the part (the frame's −z).
        direction: DVec3,
        /// The distance from the axis to the nearest edge of the solid.
        clearance: f64,
    },
    /// Nothing named so (the names that exist follow).
    NoSuchFace(Vec<String>, Vec<String>),
    /// A face to open is not flat: cut it flat first.
    NotFlat(String),
    /// The part shares no volume with the host.
    NoOverlap,
    /// The host and the part are skinned, by skeletons that differ: one
    /// solid has one skin.
    TwoSkins,
    /// The solid has UVs and the part it is cut with has none (the part
    /// named), so the cut's new faces would have none: give the part UVs
    /// too, or neither.
    NoUvs(String),
    /// The solid is not closed and manifold: an edge is not between
    /// exactly two faces.
    NotClosed,
    /// The names asked for that no edge carries, and the edge names that
    /// exist.
    NoSuchEdge(Vec<String>, Vec<String>),
    /// What the edit does not do, and what to do instead.
    Unsupported(&'static str),
    /// The inner solid of a shell came out through itself, most of its
    /// walls facing the other way: the walls meet and nothing would be
    /// hollowed (a ring hollowed thicker than its tube).
    InsideOut {
        /// The wall asked for.
        thickness: f64,
    },
    /// A convex solid has no point `thickness` from every face it keeps:
    /// walls that thick meet, and nothing would be hollowed. `short` is
    /// how far the deepest point is from being that far in (0 when the
    /// walls just meet).
    NoRoom {
        /// The wall asked for.
        thickness: f64,
        /// How far short the deepest point is.
        short: f64,
    },
}

impl From<BoolError> for EditError {
    fn from(e: BoolError) -> Self {
        EditError::Bool(e)
    }
}

impl From<GenError> for EditError {
    fn from(e: GenError) -> Self {
        EditError::Gen(e)
    }
}

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditError::Bool(e) => write!(f, "{e}"),
            EditError::Gen(e) => write!(f, "{e}"),
            EditError::NotPositive(n) => write!(f, "{n} must be positive"),
            EditError::NoDirection(n) => write!(f, "{n} has no direction"),
            EditError::Misses {
                origin,
                direction,
                clearance,
            } => write!(
                f,
                "the axis through {origin} along {direction} meets no face of the solid: \
                 its nearest edge is {clearance} away from it (an axis inside a through \
                 hole meets nothing)"
            ),
            EditError::NoSuchFace(m, a) => write!(
                f,
                "no face is named {}; names: {}",
                m.join(", "),
                a.join(", ")
            ),
            EditError::NotFlat(n) => write!(f, "the faces {n} are not flat; cut them flat first"),
            EditError::NoOverlap => f.write_str("the part shares no volume with the host"),
            EditError::TwoSkins => f.write_str(
                "the host and the part are skinned by different skeletons, and one solid has one skin: skin them alike, or seat first and skin the result",
            ),
            EditError::NotClosed => f.write_str(
                "the solid is not closed and manifold (an edge is not between exactly two faces)",
            ),
            EditError::NoSuchEdge(missing, have) => write!(
                f,
                "no edge is named {}; edges are named {}",
                missing.join(", "),
                have.join(", ")
            ),
            EditError::Unsupported(what) => f.write_str(what),
            EditError::InsideOut { thickness } => write!(
                f,
                "walls {thickness} thick meet inside the solid and leave no hollow: its inner \
                 solid comes out through itself"
            ),
            EditError::NoRoom { thickness, short } => write!(
                f,
                "walls {thickness} thick meet inside the solid and leave no hollow: \
                 its deepest point is {short:.3} short of that far in"
            ),
            EditError::NoUvs(what) => write!(
                f,
                "the solid has UVs and {what} has none, so the faces it cuts would have none: \
                 give {what} UVs too (`uvs`), or neither"
            ),
        }
    }
}

impl std::error::Error for EditError {}

/// Finite and above zero (NaN fails).
pub(crate) fn positive(v: f64) -> bool {
    v.is_finite() && v > 0.0
}

/// A boolean of `s` with a tool, UVs kept as `s` has them: a tool's
/// UVs are dropped when `s` has none (the result has none either), and a
/// tool without UVs on a solid with them is refused ([`EditError::NoUvs`],
/// naming `what`), never cut in with none.
pub(crate) fn uv_boolean(
    s: &Solid,
    tool: &Solid,
    d: &Boolean,
    what: &str,
) -> Result<(Solid, BoolReport), EditError> {
    if s.uvs.is_none() && tool.uvs.is_some() {
        let mut bare = tool.clone();
        bare.uvs = None;
        return Ok(boolean_report(s, &bare, d)?);
    }
    let (out, report) = if d.op == BoolOp::Cut {
        boolean_report(s, &cut_tool_uvs(tool), d)?
    } else {
        boolean_report(s, tool, d)?
    };
    if report.faces_without_uvs > 0 {
        return Err(EditError::NoUvs(what.into()));
    }
    Ok((out, report))
}

/// What an edit did that its result alone does not say (`hole_report`,
/// `pocket_report`, `seat_report`, `shell_report`, `dent_report`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditReport {
    /// Pairs of the result's triangles that still cross after the repair
    /// of rounding errors, summed over the edit's booleans
    /// ([`BoolReport::crossing_pairs`]); the result's own
    /// ([`crossing_pairs`]) for [`dent`], which makes no boolean, and for a
    /// [`shell`] whose inner solid folded through itself. 0 for a result
    /// that does not pass through itself. A measurement, never a refusal
    /// (`docs/research.md` §27).
    pub crossing_pairs: usize,
}

impl EditReport {
    fn add(&mut self, r: &BoolReport) {
        self.crossing_pairs += r.crossing_pairs;
    }
}

/// A cut turns the tool's faces inside out, and with them their UVs,
/// which would then read mirrored seen from the cavity: the tool's u is
/// negated first, so they read as the tool's own did from outside.
pub fn cut_tool_uvs(tool: &Solid) -> Solid {
    let mut t = tool.clone();
    if let Some(uvs) = &mut t.uvs {
        for q in uvs.iter_mut().flatten() {
            q.x = -q.x;
        }
    }
    t
}

/// An extrusion for a tool, with UVs when the solid it cuts has them.
pub(crate) fn tool_extrude(
    profile: &crate::profile::Profile,
    d: &Extrude,
    uvs: bool,
) -> Result<Solid, GenError> {
    if uvs {
        textured(&Generator::Extrude(profile, *d))
    } else {
        extrude(profile, d)
    }
}

pub(crate) fn size(s: &Solid) -> f64 {
    let m = s.measure();
    (m.bounds_max - m.bounds_min).length().max(1.0)
}

pub(crate) fn placed(mut tool: Solid, frame: &Frame) -> Solid {
    tool.map_positions(|p| frame.point_to_world(p), false);
    tool
}

/// Where the line through `origin` along `-dir` first enters the solid
/// from the `dir` side: the hit farthest along `dir`. Whether the line
/// meets a triangle is decided exactly: the line, through `origin` and
/// `origin - dir`, meets it when `orient3d` of the line with each of its
/// edges has one sign or is zero, so a line through an edge or a corner
/// meets every triangle there (`docs/research.md` §13.6, item 13). A
/// triangle the line lies in is skipped. `Err` gives the distance from the
/// line to the nearest edge when it meets none.
pub(crate) fn entry(s: &Solid, origin: DVec3, dir: DVec3) -> Result<DVec3, f64> {
    let c3 = |p: DVec3| Coord3D {
        x: p.x,
        y: p.y,
        z: p.z,
    };
    let d = -dir;
    let (p0, p1) = (c3(origin), c3(origin + d));
    let mut best: Option<f64> = None;
    for t in &s.triangles {
        let [a, b, c] = t.map(|v| s.positions[v as usize]);
        let side = [(a, b), (b, c), (c, a)].map(|(x, y)| orient3d(p0, p1, c3(x), c3(y)));
        if side.iter().any(|&x| x > 0.0) && side.iter().any(|&x| x < 0.0) {
            continue;
        }
        // Distance along -dir from the triangle's plane equation, which is
        // exact for axis-aligned faces (the hit point lands on the face).
        let n = (b - a).cross(c - a);
        let nd = n.dot(d);
        if side == [0.0; 3] || nd == 0.0 {
            continue;
        }
        let dist = n.dot(a - origin) / nd;
        if best.is_none_or(|b| dist < b) {
            best = Some(dist);
        }
    }
    best.map(|dist| origin + d * dist)
        .ok_or_else(|| clearance(s, origin, dir))
}

/// The distance from the line through `origin` along `dir` to the nearest
/// edge of `s` (infinite for a solid with no triangles).
pub(crate) fn clearance(s: &Solid, origin: DVec3, dir: DVec3) -> f64 {
    let u = dir.normalize_or_zero();
    let off = |v: DVec3| v - u * v.dot(u);
    let mut near = f64::INFINITY;
    for t in &s.triangles {
        for k in 0..3 {
            let (a, b) = (
                s.positions[t[k] as usize],
                s.positions[t[(k + 1) % 3] as usize],
            );
            let (w, e) = (off(a - origin), off(b - a));
            let ee = e.dot(e);
            let f = if ee > 0.0 {
                (-w.dot(e) / ee).clamp(0.0, 1.0)
            } else {
                0.0
            };
            near = near.min((w + e * f).length());
        }
    }
    near
}

/// The entry point of frame `f`'s axis, or [`EditError::Misses`].
pub(crate) fn entry_of(s: &Solid, f: &Frame, axis: &Frame, span: f64) -> Result<DVec3, EditError> {
    entry(s, f.origin + axis.z * 2.0 * span, axis.z).map_err(|clearance| EditError::Misses {
        origin: f.origin,
        direction: -axis.z + DVec3::ZERO,
        clearance,
    })
}

// ── split ───────────────────────────────────────────────────────────────────

// ── hole ────────────────────────────────────────────────────────────────────

// ── pocket ──────────────────────────────────────────────────────────────────

// ── seat ────────────────────────────────────────────────────────────────────

// ── shell ───────────────────────────────────────────────────────────────────

// ── dent ────────────────────────────────────────────────────────────────────

// ── merge ───────────────────────────────────────────────────────────────────

pub mod subdivide;
pub use subdivide::{MAX_LEVELS, Subdivide, subdivide};
