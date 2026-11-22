//! Repair of imported meshes: apply transforms, join, weld, remove
//! degenerate and duplicate triangles, remove floaters, orient faces
//! consistently, close small holes, and fuse interpenetrating shells.
//!
//! Non-manifold edges (more than two faces) are split into manifold
//! sheets when asked ([`split_non_manifold`]), after orienting.
//!
//! Entry: [`repair`] runs the steps in that order from a [`Repair`] desc
//! and reports what each changed in a [`RepairReport`]. Every step is also
//! its own function.
//!
//! Names: every surviving triangle keeps its face names and surface;
//! welding keeps the names of edges whose two ends survive. Triangles that
//! close a hole are named `fill` plus the face names around the hole and
//! take the surface of the first triangle on its rim. [`join`] keeps each
//! part's names (imported faces carry their part's name).
//!
//! Skins (`Solid::skin`) are carried through every step: weights follow
//! their vertices, vertices merged into one take the mean of their weights
//! (`Skin::weld`), and a vertex a hole fill adds takes the blend of its
//! rim's. [`apply_transform`] bakes a rigid transform into the vertices,
//! the bones' rest frames and their inverse bind matrices; [`join`] merges
//! skeletons by bone name. What cannot be kept is an error value
//! ([`RepairError::Skinned`]): a scaled or mirroring transform on a
//! skinned part, bones of one name that disagree, skinned and unskinned
//! parts joined. The self-union of [`Repair::fuse`] carries the skin as
//! `crate::boolean` does: a new vertex takes the weights of the skinned
//! surface nearest it.
//!
//! UVs (`Solid::uvs`, per triangle corner) are carried through every
//! step: a kept triangle keeps its corners' UVs (welding moves positions,
//! never UVs), a turned triangle turns its UVs with it, and the fuse step
//! interpolates them on the faces it cuts (`crate::boolean`); a corner it
//! leaves on an input vertex gets that vertex's own UV back (the nearest
//! of its UVs, so a seam keeps its sides), not an interpolation, so
//! neighbours agree and no seam appears where the file had none. A hole fill
//! takes, at each rim vertex, the UV its rim triangle gives it, and its
//! centre vertex (if any) their mean; a fill whose rim crosses a UV seam
//! would fan across the texture's atlas, so it takes one UV at every
//! corner instead ([`RepairReport::uv_seam_fills`] counts those).
//!
//! Vertex colours (`Solid::colors`) are carried the same way as UVs: a
//! kept triangle keeps its corners' colours, the fuse step interpolates
//! them on cut faces and gives a corner on an input vertex that vertex's
//! own colour back, and a hole fill takes its rim's colours (a fan's
//! centre their mean), also across a UV seam.
//! [`join`] gives the triangles of parts without UVs (0, 0) at every
//! corner when another part has them: zero-area UV triangles, which
//! `measure` counts.
//!
//! A single closed surface that folds through itself is untangled by the
//! fuse step too (`crate::boolean::self_union`: the space it winds round at
//! least once); the pockets an untangling leaves as shells of their own
//! are counted, and dropped when they are floaters. An open one cannot
//! be: [`repair`] reports its intersecting triangle pairs and leaves it,
//! and booleans refuse it (`docs/research.md` §5.4, §9, §32).

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;

use crate::math::order::sorted_order;
use crate::math::predicates::{orient2d, orient3d};
use crate::math::{DVec2, DVec3, HashError};
use crate::mesh::skin::Mat4;
use crate::mesh::{Bone, CornerColors, Skin, Solid, Tag, WHITE, Weights, edge_key};

use crate::tools::import::{Affine, ImportedPart};
use crate::tools::intersect::{degenerate, self_intersections};

mod attributes;
mod degenerate;
mod floaters;
mod holes;
mod intersections;
mod manifold;
mod orient;
mod rebuild;
mod transform;
mod weld;
pub(crate) use attributes::*;
pub use degenerate::*;
pub use floaters::*;
pub use holes::*;
pub(crate) use intersections::*;
pub use manifold::*;
pub use orient::*;
pub(crate) use rebuild::*;
pub use transform::*;
pub use weld::*;

/// Which small disconnected pieces [`remove_floaters`] drops. The zero
/// value drops none.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Floaters {
    /// Drops a piece whose bounding-box diagonal is below this, in mm.
    pub size: f64,
    /// Drops a piece whose diagonal is below this fraction of the largest
    /// piece's.
    pub fraction: f64,
}

/// What [`repair`] does. The zero value welds exact duplicates only,
/// removes degenerate and duplicate triangles, orients faces and leaves
/// pieces and holes alone.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Repair {
    /// Welds vertices on open edges closer than this, in mm (0: exact
    /// duplicates only); see [`Weld`].
    pub weld: f64,
    /// Welds every vertex within `weld`, not only those on open edges.
    pub weld_all: bool,
    /// Small pieces to drop.
    pub floaters: Floaters,
    /// Splits edges with more than two faces into manifold sheets
    /// ([`split_non_manifold`]) after orienting, then drops the pieces
    /// that come apart and are floaters (by `floaters`), before holes are
    /// closed.
    pub non_manifold: bool,
    /// Closes holes whose rim has at most this many edges (0: none).
    pub max_hole_edges: usize,
    /// Closes a larger hole that no clear fill closes all the same, by
    /// a fill that crosses the mesh (a mesh with faces behind its
    /// holes). `fuse`, which untangles what that leaves, does this by
    /// itself; without it the crossings stay.
    pub holes_through: bool,
    /// Gives every piece still open after the holes are closed a second
    /// sheet this far under it, in mm, and a rim along its open edges
    /// ([`crate::tools::thicken`]; 0: none), so a sheet encloses a volume.
    pub thicken: f64,
    /// Joins closed, interpenetrating shells into one by self-union
    /// (`crate::boolean::fuse`) once the rest is done, and untangles a shell
    /// that passes through itself. A union that is not closed (shells
    /// touching along an edge) is not taken: the shells are kept, and the
    /// report says why.
    pub fuse: bool,
    /// Collapses edges shorter than this, in mm, where that keeps the mesh
    /// manifold (0: none): the slivers a self-union leaves, whose vertices
    /// f32 export would merge.
    pub min_edge: f64,
    /// Counts self-intersecting triangle pairs at the end (exact; costs a
    /// sweep over the triangles).
    pub check_intersections: bool,
}

/// What each step of [`repair`] changed.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RepairReport {
    /// Vertices that went in the weld (merged, exact duplicates
    /// included, or used by no triangle).
    pub welded: usize,
    /// Triangles dropped for having no area.
    pub degenerate: usize,
    /// Triangles dropped for repeating another (same corners, same
    /// winding).
    pub duplicates: usize,
    /// Pieces dropped as floaters.
    pub floaters: usize,
    /// Triangles those pieces had.
    pub floater_triangles: usize,
    /// Triangles turned over to agree with their neighbours or to face
    /// out.
    pub flipped: usize,
    /// What splitting non-manifold edges did, if asked.
    pub non_manifold: Option<NonManifold>,
    /// Holes closed, and triangles added to close them.
    pub holes_filled: usize,
    /// Of `holes_filled`, holes with more than `max_hole_edges` edges:
    /// closed by a fill that is flat and crosses nothing, by a round
    /// fill or a fan from a lifted centre that crosses nothing, or
    /// (`holes_through`, or `fuse`) by a fill that crosses.
    pub large_holes_filled: usize,
    /// Of those, the ones whose fill crosses the mesh.
    pub crossing_fills: usize,
    /// Of the large holes filled, the ones closed round: rings of new
    /// vertices that leave the rim along the surface.
    pub round_fills: usize,
    /// Triangles added by closing holes.
    pub fill_triangles: usize,
    /// Holes left open: too large, or with a rim that does not close (it
    /// runs into a non-manifold edge).
    pub holes_left: usize,
    /// The outcome of fusing, if asked: `Some(Ok(pieces before))`, or why
    /// it was not done.
    pub fused: Option<Result<usize, String>>,
    /// Shells the fuse step's untangling left besides each untangled
    /// shell's largest: pockets the surface folded round.
    pub pockets: usize,
    /// Of those, dropped as floaters (by `floaters`), and their
    /// triangles.
    pub pockets_dropped: usize,
    /// See `pockets_dropped`.
    pub pocket_triangles_dropped: usize,
    /// Vertices removed by collapsing short edges.
    pub short_edges: usize,
    /// Self-intersecting triangle pairs at the end, if asked.
    pub self_intersections: Option<usize>,
    /// Filled holes whose rim crosses a UV seam (a rim vertex with
    /// different UVs on its two rim triangles): the fill takes one UV at
    /// every corner (one colour, covering no area of the texture), as a
    /// fan across the seam would span the atlas. 0 on a solid without
    /// UVs.
    pub uv_seam_fills: usize,
    /// What thickening the open pieces did, if it was asked for.
    pub thickened: Option<crate::tools::thicken::Thickened>,
}

/// Why repair could not run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepairError {
    /// A position is NaN.
    NaN,
    /// A weld distance or floater size is negative or not finite.
    BadDistance,
    /// A skin that cannot be kept: what and why (a skinned part placed by
    /// a scaled or mirroring transform, bones of one name that disagree,
    /// skinned and unskinned parts joined).
    Skinned(String),
}

impl fmt::Display for RepairError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepairError::NaN => f.write_str("a position is NaN"),
            RepairError::BadDistance => {
                f.write_str("a weld distance or size is negative or not finite")
            }
            RepairError::Skinned(w) => {
                write!(f, "{w}: the skin cannot be kept, so this is refused")
            }
        }
    }
}

impl std::error::Error for RepairError {}

impl From<HashError> for RepairError {
    fn from(_: HashError) -> Self {
        RepairError::NaN
    }
}

/// Runs the repair steps in order (weld, degenerate and duplicate
/// triangles, floaters, orientation, holes, thickening, fuse, short edges)
/// and reports
/// what each changed. The result is in canonical order.
pub fn repair(s: &Solid, d: &Repair) -> Result<(Solid, RepairReport), RepairError> {
    repair_owned(s.clone(), d)
}

/// [`repair`], taking the solid: a caller done with its input gives it
/// up, and the steps work on one copy less (300 MiB on 600 000 triangles
/// with colours, `docs/research.md` §36). The result is the same.
pub fn repair_owned(s: Solid, d: &Repair) -> Result<(Solid, RepairReport), RepairError> {
    let mut r = RepairReport::default();
    // Each step's input is dropped as soon as its output stands: a mesh
    // of 600 000 triangles with colours is some 300 MiB, and nine of them
    // held to the end were most of the peak (`docs/research.md` §36).
    let (mut s, welded) = weld_owned(
        s,
        &Weld {
            distance: d.weld,
            all: d.weld_all,
        },
    )?;
    r.welded = welded;
    let (next, deg, dup) = remove_degenerate(&s)?;
    s = next;
    r.degenerate = deg;
    r.duplicates = dup;
    let (next, f, ft) = remove_floaters(&s, &d.floaters)?;
    s = next;
    r.floaters = f;
    r.floater_triangles = ft;
    let (next, flipped) = orient_owned(s)?;
    s = next;
    r.flipped = flipped;
    if d.non_manifold {
        let (next, mut nm) = split_non_manifold(&s)?;
        s = next;
        // Sheets apart now may be floaters, and orientation can now run
        // across the edges that were split.
        let (next, f, ft) = remove_floaters(&s, &d.floaters)?;
        s = next;
        nm.sheets_dropped = f;
        nm.sheet_triangles_dropped = ft;
        let (next, more) = orient_owned(s)?;
        s = next;
        r.flipped += more;
        r.non_manifold = Some(nm);
    }
    if d.max_hole_edges > 0 {
        let (next, h) = close_holes_through(&s, d.max_hole_edges, d.holes_through || d.fuse)?;
        s = next;
        r.uv_seam_fills = h.uv_seam_fills;
        r.holes_filled = h.filled;
        r.large_holes_filled = h.large;
        r.crossing_fills = h.crossing;
        r.round_fills = h.round;
        r.fill_triangles = h.triangles;
        r.holes_left = h.left;
        // Filling can join pieces that were apart; orient once more.
        let (next, more) = orient_owned(s)?;
        s = next;
        r.flipped += more;
    }
    if d.thicken > 0.0 {
        let (next, t) = crate::tools::thicken::thicken(&s, d.thicken)?;
        s = next;
        r.thickened = Some(t);
    }
    let s = if d.fuse {
        let n = pieces(&s).1;
        // One shell is left as it is unless it is closed and passes
        // through itself: then fuse untangles it (`crate::boolean::self_union`).
        let tangled = n == 1 && s.measure().closed && crate::boolean::crossing_pairs(&s) > 0;
        if n <= 1 && !tangled {
            r.fused = Some(Ok(n));
            s
        } else {
            match fuse_shells(&s, &d.floaters, &mut r) {
                Ok(f) if f.measure().closed => {
                    r.fused = Some(Ok(n));
                    keep_input_colors(&s, keep_input_uvs(&s, f)?)?
                }
                Ok(f) => {
                    let m = f.measure();
                    r.fused = Some(Err(format!(
                        "the union is not closed ({} open, {} non-manifold edges); the shells are kept apart",
                        m.boundary_edges, m.non_manifold_edges
                    )));
                    s
                }
                Err(e) => {
                    r.fused = Some(Err(e.to_string()));
                    s
                }
            }
        }
    } else {
        s
    };
    let s = if d.min_edge > 0.0 {
        let (s, gone) = crate::tools::decimate::collapse_short_edges(&s, d.min_edge)
            .map_err(|_| RepairError::BadDistance)?;
        r.short_edges = gone;
        s
    } else {
        s
    };
    if d.check_intersections {
        // On vertices welded where positions are equal, as `measure` counts.
        let (positions, triangles) = crate::tools::measure::weld_exact(&s);
        r.self_intersections = Some(self_intersections(&positions, &triangles).len());
    }
    Ok((s, r))
}

#[cfg(test)]
mod tests;
