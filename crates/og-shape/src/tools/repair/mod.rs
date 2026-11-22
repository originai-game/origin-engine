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
mod tests {
    use super::*;

    fn cube(offset: DVec3, size: f64) -> Solid {
        let mut s = Solid::default();
        for i in 0..8 {
            let p = DVec3::new((i & 1) as f64, ((i >> 1) & 1) as f64, ((i >> 2) & 1) as f64);
            s.push_vertex(offset + p * size);
        }
        let quads = [
            [0, 2, 3, 1],
            [4, 5, 7, 6],
            [0, 1, 5, 4],
            [2, 6, 7, 3],
            [0, 4, 6, 2],
            [1, 3, 7, 5],
        ];
        for (k, q) in quads.iter().enumerate() {
            s.push_triangle([q[0], q[1], q[2]], k as u32, &["cube"]);
            s.push_triangle([q[0], q[2], q[3]], k as u32, &["cube"]);
        }
        s
    }

    #[test]
    fn repairs_a_broken_cube() {
        let mut s = cube(DVec3::ZERO, 10.0);
        assert!(s.measure().closed);
        // Split one corner into two copies 1e-4 apart, turn two faces
        // over, drop one triangle, add a floater and a sliver.
        let extra = s.push_vertex(DVec3::new(10.0, 0.0, 10.0 + 1e-4));
        s.triangles[11][2] = extra;
        s.triangles[0].swap(1, 2);
        s.triangles[5].swap(1, 2);
        s.triangles.remove(7);
        s.surfaces.remove(7);
        s.face_tags.remove(7);
        s.append(&cube(DVec3::splat(30.0), 0.5));
        let a = s.push_vertex(DVec3::ZERO);
        let b = s.push_vertex(DVec3::new(5.0, 0.0, 0.0));
        let c = s.push_vertex(DVec3::new(10.0, 0.0, 0.0));
        s.push_triangle([a, b, c], 0, &["sliver"]);
        assert!(!s.measure().closed);
        let d = Repair {
            weld: 0.01,
            floaters: Floaters {
                size: 2.0,
                ..Default::default()
            },
            max_hole_edges: 8,
            check_intersections: true,
            ..Default::default()
        };
        let (out, r) = repair(&s, &d).unwrap();
        let m = out.measure();
        assert!(m.closed, "{r:?} {m:?}");
        assert!((m.volume - 1000.0).abs() < 1e-9);
        assert_eq!((r.floaters, r.floater_triangles), (1, 12));
        assert_eq!((r.holes_filled, r.fill_triangles), (1, 1));
        assert_eq!(r.degenerate, 1);
        assert_eq!(r.self_intersections, Some(0));
        assert_eq!(out.faces(&["fill"]).unwrap().len(), 1);
    }

    #[test]
    fn joins_parts_and_mirrors_keep_faces_out() {
        let part = |name: &str, t: Affine| ImportedPart {
            name: name.into(),
            solid: cube(DVec3::ZERO, 1.0),
            transform: t,
            ..Default::default()
        };
        let mirror = Affine {
            x: -DVec3::X,
            t: DVec3::new(-2.0, 0.0, 0.0),
            ..Affine::IDENTITY
        };
        let s = join(&[part("a", Affine::IDENTITY), part("b", mirror)]).unwrap();
        let m = s.measure();
        assert!(m.closed && m.components == 2);
        assert!((m.volume - 2.0).abs() < 1e-12);
    }

    /// Edges whose two triangles give either end different UVs.
    fn uv_seams(s: &Solid) -> usize {
        let uvs = s.uvs.as_ref().unwrap();
        let mut sides: BTreeMap<(u32, u32), Vec<[[u64; 2]; 2]>> = BTreeMap::new();
        for (t, tri) in s.triangles.iter().enumerate() {
            for k in 0..3 {
                let (a, b) = (tri[k], tri[(k + 1) % 3]);
                let (ua, ub) = (uvs[t][k], uvs[t][(k + 1) % 3]);
                let bits = |u: DVec2| [u.x.to_bits(), u.y.to_bits()];
                let side = if a < b {
                    [bits(ua), bits(ub)]
                } else {
                    [bits(ub), bits(ua)]
                };
                sides.entry(edge_key(a, b)).or_default().push(side);
            }
        }
        sides
            .values()
            .filter(|v| v.len() == 2 && v[0] != v[1])
            .count()
    }

    #[test]
    fn fuse_keeps_the_files_uvs_on_the_vertices_it_keeps() {
        // Two textured balls apart: the union cuts nothing, so every
        // corner is an input corner and the seams are the generator's.
        let ball = crate::generators::solids::textured(
            &crate::generators::solids::Generator::Sphere(crate::generators::solids::SphereDesc {
                radius: 7.3,
                ..Default::default()
            }),
        )
        .unwrap();
        let mut two = ball.clone();
        let mut other = ball.clone();
        for p in &mut other.positions {
            *p += DVec3::new(31.7, 0.3, 0.0);
        }
        two.append(&other);
        two.canonicalize().unwrap();
        let d = Repair {
            fuse: true,
            ..Default::default()
        };
        let (out, r) = repair(&two, &d).unwrap();
        assert_eq!(r.fused, Some(Ok(2)));
        assert_eq!(uv_seams(&out), uv_seams(&two));
    }

    #[test]
    fn fuse_untangles_a_shell_folded_through_itself() {
        // A sphere with one vertex pulled through the far side
        // (`docs/research.md` §9.6): closed, one piece, crossing itself.
        let mut ball = crate::generators::solids::sphere(10.0).unwrap();
        let dir = DVec3::new(-0.51, -0.38, 0.77);
        let v = (0..ball.positions.len())
            .max_by(|&a, &b| {
                ball.positions[a]
                    .dot(dir)
                    .total_cmp(&ball.positions[b].dot(dir))
            })
            .unwrap();
        ball.positions[v] = -ball.positions[v] * 1.5;
        let d = Repair {
            fuse: true,
            check_intersections: true,
            ..Default::default()
        };
        let (out, r) = repair(&ball, &d).unwrap();
        assert_eq!(r.fused, Some(Ok(1)));
        assert_eq!(r.self_intersections, Some(0));
        let m = out.measure();
        assert!(m.closed && m.components == 1, "{m:?}");
        // Without fuse it is left, and reported.
        let (_, r) = repair(
            &ball,
            &Repair {
                check_intersections: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(r.self_intersections.unwrap() > 0);
    }

    #[test]
    fn fuse_drops_the_pockets_an_untangling_leaves() {
        // A ball whose vertices are moved by up to 1.5 per axis
        // (splitmix64): it folds through itself, and its self-union is a
        // main shell and three small pockets the surface folded round.
        let mut ball = crate::generators::solids::sphere(10.0).unwrap();
        let mut state = 5u64;
        let mut next = || {
            state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^= z >> 31;
            (z >> 11) as f64 / (1u64 << 52) as f64 - 1.0
        };
        for p in &mut ball.positions {
            *p += DVec3::new(next(), next(), next()) * 1.5;
        }
        let d = Repair {
            fuse: true,
            floaters: Floaters {
                fraction: 0.05,
                ..Default::default()
            },
            check_intersections: true,
            ..Default::default()
        };
        let (out, r) = repair(&ball, &d).unwrap();
        assert_eq!(r.fused, Some(Ok(1)));
        assert_eq!((r.pockets, r.pockets_dropped), (3, 3), "{r:?}");
        assert_eq!(r.self_intersections, Some(0));
        assert_eq!(pieces(&out).1, 1);
        assert!(out.measure().closed);
    }

    #[test]
    fn two_cubes_on_one_edge_come_apart_closed() {
        // Two cubes touching along one edge: four faces on it.
        let mut s = cube(DVec3::ZERO, 10.0);
        s.append(&cube(DVec3::new(10.0, 10.0, 0.0), 10.0));
        s.canonicalize().unwrap();
        assert_eq!(s.measure().non_manifold_edges, 1);
        let (out, r) = repair(
            &s,
            &Repair {
                non_manifold: true,
                check_intersections: true,
                ..Default::default()
            },
        )
        .unwrap();
        let nm = r.non_manifold.unwrap();
        assert_eq!(
            (nm.edges, nm.split, nm.left, nm.unpaired),
            (1, 1, 0, 0),
            "{nm:?}"
        );
        assert_eq!(nm.vertices, 2);
        let m = out.measure();
        assert!(m.closed, "{m:?}");
        assert_eq!(pieces(&out).1, 2);
        assert_eq!(r.self_intersections, Some(0));
        // Each copy moved into its own cube, 1 % of its shortest edge.
        assert!((nm.moved - 0.1).abs() < 1e-12, "{nm:?}");
    }

    #[test]
    fn a_fill_across_a_uv_seam_covers_no_texture() {
        let ball = crate::generators::solids::textured(
            &crate::generators::solids::Generator::Sphere(crate::generators::solids::SphereDesc {
                radius: 7.3,
                ..Default::default()
            }),
        )
        .unwrap();
        // A vertex on the seam (two UVs), away from the poles: its fan
        // removed leaves a hole whose rim crosses the seam.
        let uvs = ball.uvs.as_ref().unwrap();
        let mut at: BTreeMap<u32, Vec<[u64; 2]>> = BTreeMap::new();
        for (t, tri) in ball.triangles.iter().enumerate() {
            for k in 0..3 {
                let u = uvs[t][k];
                at.entry(tri[k])
                    .or_default()
                    .push([u.x.to_bits(), u.y.to_bits()]);
            }
        }
        let v = *at
            .iter()
            .find(|(v, u)| {
                let mut u = (*u).clone();
                u.sort_unstable();
                u.dedup();
                u.len() == 2 && ball.positions[**v as usize].z.abs() < 3.0
            })
            .unwrap()
            .0;
        let mut open = ball.clone();
        let keep: Vec<usize> = (0..open.triangles.len())
            .filter(|&t| !open.triangles[t].contains(&v))
            .collect();
        let remap: Vec<u32> = (0..open.positions.len() as u32).collect();
        open = drop_unused(&rebuild(&open, &keep, &remap, open.positions.clone()));
        let (out, h) = close_holes(&open, 64).unwrap();
        assert_eq!((h.filled, h.uv_seam_fills), (1, 1), "{h:?}");
        let fill = out.faces(&["fill"]).unwrap();
        assert!(!fill.is_empty());
        let uvs = out.uvs.as_ref().unwrap();
        for t in fill {
            let u = uvs[t as usize];
            assert!(u[0] == u[1] && u[1] == u[2], "{u:?}");
        }
        assert!(out.measure().closed);
    }

    #[test]
    fn a_hole_over_the_limit_is_filled_when_flat_and_clear() {
        // A cube without one face: a square hole of four edges.
        let mut s = cube(DVec3::ZERO, 10.0);
        for _ in 0..2 {
            s.triangles.pop();
            s.surfaces.pop();
            s.face_tags.pop();
        }
        let (out, h) = close_holes(&s, 3).unwrap();
        assert_eq!((h.filled, h.large, h.left), (1, 1, 0), "{h:?}");
        assert!(out.measure().closed);
        // Within the limit it is an ordinary fill.
        let (_, h) = close_holes(&s, 4).unwrap();
        assert_eq!((h.filled, h.large), (1, 0));
    }

    /// A closed tetrahedron standing on z = `base` at (5, 5), its tip at
    /// z = `tip`.
    fn spike(base: f64, tip: f64) -> Solid {
        let mut s = Solid::default();
        for p in [
            DVec3::new(4.5, 4.7, base),
            DVec3::new(5.5, 4.7, base),
            DVec3::new(5.0, 5.6, base),
            DVec3::new(5.0, 5.0, tip),
        ] {
            s.push_vertex(p);
        }
        for t in [[0, 2, 1], [0, 1, 3], [1, 2, 3], [2, 0, 3]] {
            s.push_triangle(t, 9, &["spike"]);
        }
        s
    }

    /// A ball with its cap cut away closes round: rings of new vertices
    /// that leave the rim along the ball, to within a hundredth of the
    /// whole ball's volume (a flat fill is a tenth short). A hole in a
    /// flat face stays a flat fill.
    #[test]
    fn a_large_hole_in_a_curved_surface_closes_round() {
        let ball = crate::generators::solids::sphere_desc(&crate::generators::solids::SphereDesc {
            radius: 50.0,
            segments: 48,
            ..Default::default()
        })
        .unwrap();
        let whole = ball.measure().volume;
        let mut open = ball.clone();
        // Without what stands above 0.6 of its radius (the poles are on y).
        for i in (0..open.triangles.len()).rev() {
            let top = open.triangles[i]
                .iter()
                .map(|&v| open.positions[v as usize].y)
                .sum::<f64>()
                / 3.0;
            if top > 30.0 {
                open.triangles.remove(i);
                open.surfaces.remove(i);
                open.face_tags.remove(i);
            }
        }
        let (out, h) = close_holes(&open, 8).unwrap();
        assert_eq!((h.filled, h.large, h.round, h.left), (1, 1, 1, 0), "{h:?}");
        let m = out.measure();
        assert!(m.closed);
        assert!(
            (m.volume - whole).abs() < 0.01 * whole,
            "{} {whole}",
            m.volume
        );
        // Closed flat it would be short by the cap above the cut (of
        // height 20: π h² (3 r − h) / 3), a tenth of the ball.
        let cap = core::f64::consts::PI * 20.0 * 20.0 * (150.0 - 20.0) / 3.0;
        assert!(whole - cap < 0.91 * whole);

        // A hole in a flat face: no tilt at its rim, a flat fill.
        let mut lid = cube(DVec3::ZERO, 10.0);
        for k in [3, 2] {
            lid.triangles.remove(k);
            lid.surfaces.remove(k);
            lid.face_tags.remove(k);
        }
        let (_, h) = close_holes(&lid, 3).unwrap();
        assert_eq!((h.filled, h.large, h.round), (1, 1, 0), "{h:?}");
    }

    /// A large hole with something in its way: a spike that just pokes
    /// through the flat fill is cleared by a fan from a lifted centre;
    /// one that stands far through is closed only when asked, by a fill
    /// that crosses it, and counted.
    #[test]
    fn a_large_hole_is_fanned_round_what_pokes_through_it() {
        let mut open = cube(DVec3::ZERO, 10.0);
        // Without its top (the second quad).
        for k in [3, 2] {
            open.triangles.remove(k);
            open.surfaces.remove(k);
            open.face_tags.remove(k);
        }
        let with = |spike: &Solid| {
            let mut s = open.clone();
            s.append(spike);
            s
        };
        let low = with(&spike(9.0, 10.2));
        let (out, h) = close_holes(&low, 3).unwrap();
        assert_eq!(
            (h.filled, h.large, h.crossing, h.left),
            (1, 1, 0, 0),
            "{h:?}"
        );
        assert_eq!(out.triangles.len(), low.triangles.len() + 4);
        // Whether the fill crosses the rest of the mesh.
        let fill_crosses = |s: &Solid| -> bool {
            let fill = s.faces(&["fill"]).unwrap();
            let rest = Solid {
                positions: s.positions.clone(),
                triangles: (0..s.triangles.len() as u32)
                    .filter(|t| !fill.contains(t))
                    .map(|t| s.triangles[t as usize])
                    .collect(),
                ..Default::default()
            };
            let tris: Vec<[u32; 3]> = fill.iter().map(|&t| s.triangles[t as usize]).collect();
            crosses(&rest, &tris, &[])
        };
        assert!(!fill_crosses(&out));
        let tall = with(&spike(9.0, 15.0));
        let (_, h) = close_holes(&tall, 3).unwrap();
        assert_eq!((h.filled, h.left), (0, 1), "{h:?}");
        let (out, h) = close_holes_through(&tall, 3, true).unwrap();
        assert_eq!(
            (h.filled, h.large, h.crossing, h.left),
            (1, 1, 1, 0),
            "{h:?}"
        );
        assert!(fill_crosses(&out));
    }

    /// Two columns that touch along one edge, three edges long: each of
    /// the three has four faces, and the two in the middle of the chain
    /// have such an edge at both ends. All three split, and the columns
    /// come apart closed.
    #[test]
    fn a_chain_of_non_manifold_edges_splits_along_its_length() {
        let mut s = Solid::default();
        for (ox, oy) in [(0.0, 0.0), (1.0, 1.0)] {
            for z in 0..3 {
                let c = cube(DVec3::new(ox, oy, f64::from(z)), 1.0);
                let base = s.positions.len() as u32;
                for p in &c.positions {
                    s.push_vertex(*p);
                }
                for (i, t) in c.triangles.iter().enumerate() {
                    // The faces between the cubes of a column go.
                    let bottom = i < 2 && z > 0;
                    let top = (2..4).contains(&i) && z < 2;
                    if !bottom && !top {
                        s.push_triangle(t.map(|v| v + base), c.surfaces[i], &["column"]);
                    }
                }
            }
        }
        s.canonicalize().unwrap();
        let (out, nm) = split_non_manifold(&s).unwrap();
        assert_eq!(
            (nm.edges, nm.split, nm.left, nm.unpaired),
            (3, 3, 0, 0),
            "{nm:?}"
        );
        let m = crate::tools::measure(&out, &crate::tools::Measure::default());
        assert!(m.closed && m.non_manifold_edges == 0, "{m:?}");
        assert_eq!(m.pieces.len(), 2);
    }

    /// A skin may hold only part of the skeleton: Mixamo's soldier has a
    /// body skinned to 49 bones and a visor skinned to the neck and head
    /// alone, whose neck has no parent in its own skin. The two are one
    /// skeleton; bones whose parents are named differently are not.
    #[test]
    fn a_skin_of_part_of_the_skeleton_joins_the_whole_one() {
        use crate::math::Frame;
        use crate::mesh::{Bone, Weights};
        let at = |z: f64| Frame {
            origin: DVec3::new(0.0, 0.0, z),
            ..Frame::IDENTITY
        };
        let skinned = |bones: Vec<Bone>, offset: f64| {
            let mut s = cube(DVec3::new(offset, 0.0, 0.0), 1.0);
            let last = bones.len() as u32 - 1;
            s.skin = Some(Skin {
                bones,
                weights: vec![Weights::single(last); s.positions.len()],
            });
            s
        };
        let body = skinned(
            vec![
                Bone::new("spine", None, at(0.0)),
                Bone::new("neck", Some(0), at(5.0)),
            ],
            0.0,
        );
        let visor = skinned(vec![Bone::new("neck", None, at(5.0))], 3.0);
        for parts in [
            [("body", &body), ("visor", &visor)],
            [("visor", &visor), ("body", &body)],
        ] {
            let (bones, maps, _) = merge_skeletons(&parts).unwrap().unwrap();
            let names: Vec<&str> = bones.iter().map(|b| b.name.as_str()).collect();
            assert_eq!(names, ["spine", "neck"]);
            assert_eq!(bones[1].parent, Some(0));
            // Each part's bones, by its own order, in the merged list.
            let visor_at = usize::from(parts[0].0 == "body");
            assert_eq!(maps[visor_at], [1]);
            assert_eq!(maps[1 - visor_at], [0, 1]);
        }
        // A neck hung on another bone is another skeleton.
        let other = skinned(
            vec![
                Bone::new("hips", None, at(0.0)),
                Bone::new("neck", Some(0), at(5.0)),
            ],
            6.0,
        );
        let e = merge_skeletons(&[("body", &body), ("other", &other)]).unwrap_err();
        assert!(e.to_string().contains("neck"), "{e}");
    }

    /// Two meshes skinned to one skeleton, each bound in its own space
    /// (their inverse bind matrices differ by one matrix for every bone),
    /// are one skeleton: the second's vertices go to the first's space.
    /// Bones that differ by different matrices are a conflict.
    #[test]
    fn parts_bound_in_different_spaces_join_on_one_skeleton() {
        use crate::math::Frame;
        use crate::mesh::{Bone, Weights};
        let frame = |x: f64, z: f64| Frame {
            origin: DVec3::new(x, 0.0, z),
            ..Frame::IDENTITY
        };
        let part = |shift: f64, twist: f64| {
            // The same two bones where they stand, and the same cube in
            // the world, its mesh written `shift` along x away: the
            // inverse binds take it back.
            let mut s = cube(DVec3::new(-shift, 0.0, 0.0), 4.0);
            let mut bones = vec![
                Bone::new("hip", None, frame(0.0, 0.0)),
                Bone::new("knee", Some(0), frame(0.0, 5.0)),
            ];
            for (k, b) in bones.iter_mut().enumerate() {
                b.inverse_bind[12] += shift + if k == 1 { twist } else { 0.0 };
            }
            s.skin = Some(Skin {
                bones,
                weights: vec![Weights::single(1); s.positions.len()],
            });
            s
        };
        let (a, b) = (part(0.0, 0.0), part(10.0, 0.0));
        let (bones, maps, fixes) = merge_skeletons(&[("a", &a), ("b", &b)]).unwrap().unwrap();
        assert_eq!(bones.len(), 2);
        assert_eq!(maps, [vec![0, 1], vec![0, 1]]);
        assert_eq!(fixes[0], None);
        let f = fixes[1].expect("a matrix for the second part");
        assert!((f[12] - 10.0).abs() < 1e-9 && f[13] == 0.0 && (f[0] - 1.0).abs() < 1e-12);
        // Joined, the two cubes stand on each other: 8 vertices, not 16.
        let named = |name: &str, solid: &Solid| ImportedPart {
            clips: Vec::new(),
            name: name.into(),
            solid: solid.clone(),
            transform: Affine::IDENTITY,
            material: Default::default(),
            defaults: Vec::new(),
        };
        let joined = join(&[named("a", &a), named("b", &b)]).unwrap();
        assert_eq!(joined.positions.len(), 8);
        joined.skin.as_ref().unwrap().check(8).unwrap();
        // One bone off by another matrix than the other: not one space.
        let e = merge_skeletons(&[("a", &a), ("c", &part(10.0, 3.0))]).unwrap_err();
        assert!(e.to_string().contains("knee"), "{e}");
    }
}
