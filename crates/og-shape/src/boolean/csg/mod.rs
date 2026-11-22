//! Boolean operations on two closed meshes (union, subtract, intersect),
//! each placed by a transform first. The result is a closed mesh when
//! both inputs are (every edge used twice, in opposite directions).
//!
//! The method, in four steps:
//!
//! 1. Each mesh is welded by exact position, and every pair of triangles
//!    whose boxes overlap is intersected. Which side of a plane a vertex
//!    lies on, whether an edge crosses a triangle and through which of its
//!    features, are all decided by Shewchuk's exact predicates (`robust`'s
//!    `orient3d` and `orient2d`), so two hosts make the same decisions.
//!    A crossing point is computed once, from inputs in a fixed order, and
//!    named by what made it (an edge and a triangle, or two edges), so the
//!    triangles that share it share the same point, bit for bit. Two that
//!    round to one position are one point.
//! 2. Each triangle crossed is cut along its intersection segments by a
//!    constrained Delaunay triangulation (spade) in its own plane, with its
//!    edges, and the points on them, as the outline.
//! 3. The pieces of each mesh are grouped into regions, pieces joined by an
//!    edge no intersection runs along, and each region is classified once:
//!    the other mesh's winding number just in front of and just behind its
//!    largest piece. Two coincident faces see the other surface change
//!    across them; the operation keeps one of them or neither.
//! 4. The kept pieces are emitted, a subtracted mesh's turned inside out,
//!    with the normals, uvs and colours of the triangles they came from.
//!
//! Crossing points and winding numbers are computed in f64 with `+ - * /`,
//! `sqrt` and [`crate::generators::trig::atan2`]: the same on every host.
//!
//! [`csg_points`] runs these steps twice when the first result is open
//! (operands that touch: faces on faces, vertices within an ulp of a face).
//! The second pass orders points on an edge exactly (`exact`), lays a
//! triangle out exactly where its rounded outline is not simple, makes
//! points at one position one point, and classifies faces lying on the
//! other operand by orientation (`docs/research.md` §5.4, item 5). When
//! the result is still open, or has edges used other than once each way,
//! the tool's vertices within 2⁻⁴⁰ of the operands' size of a target
//! vertex are snapped onto it and both passes run again (faces a few ulps
//! apart, placed by different rounded motions; `docs/research.md` §18).

use crate::generators::{Mesh, Placement, normalize, place};
use glam::{DVec2, DVec3};
use robust::{Coord, Coord3D, orient2d, orient3d};
use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation};
use std::collections::{BTreeMap, BTreeSet};

use crate::boolean::{exact, selfcheck, sliver};

mod arrange;
mod classify;
mod emit;
mod intersect;
mod operands;
mod pieces;
mod rescue;
mod self_union;
pub(crate) use arrange::*;
pub(crate) use classify::*;
pub(crate) use emit::*;
pub(crate) use intersect::*;
pub use operands::*;
pub use rescue::*;
pub use self_union::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CsgOp {
    Union,
    Subtract,
    Intersect,
}

/// `a op b`, each placed first. Refused for a singular placement or a
/// position that is not finite.
pub fn csg(a: &Mesh, pa: Placement, b: &Mesh, pb: Placement, op: CsgOp) -> Result<Mesh, String> {
    csg_traced(a, pa, b, pb, op).map(|(m, _)| m)
}

/// [`csg`], with where each output triangle came from: `(0, t)` a piece of
/// triangle `t` of `a`, `(1, t)` of `b` (one entry per output triangle, in
/// order). A fracture tells its outer faces from its cut faces by it.
pub fn csg_traced(
    a: &Mesh,
    pa: Placement,
    b: &Mesh,
    pb: Placement,
    op: CsgOp,
) -> Result<(Mesh, Vec<(u8, u32)>), String> {
    let mut points = Vec::new();
    let mut at = BTreeMap::new();
    let sa = weld(a, pa, &mut points, &mut at).map_err(|e| format!("a: {e}"))?;
    let sb = weld(b, pb, &mut points, &mut at).map_err(|e| format!("b: {e}"))?;
    let (c, kept) = arrange(points, sa, sb, op, false, 0.0);
    // 4. What the operation keeps.
    let colors = !a.colors.is_empty() || !b.colors.is_empty();
    let mut out = Mesh::default();
    let mut origins: Vec<(u8, u32)> = Vec::new();
    let mut vertex_at: BTreeMap<Vec<u32>, u32> = BTreeMap::new();
    for p in &kept {
        let flip = p.flip;
        let solid = &c.solids[p.mesh as usize];
        origins.push((p.mesh, solid.source[p.tri as usize]));
        let t = solid.tris[p.tri as usize];
        let corners = &solid.corners[p.tri as usize];
        let axis = drop_axis(c.tri_normal(t));
        let tv = t.map(|i| project(c.points[i as usize], axis));
        let den = (tv[1] - tv[0]).perp_dot(tv[2] - tv[0]);
        let ids = if flip {
            [p.ids[0], p.ids[2], p.ids[1]]
        } else {
            p.ids
        };
        for id in ids {
            let pos = c.points[id as usize];
            let q = project(pos, axis);
            let (w1, w2) = if den != 0.0 {
                (
                    (q - tv[0]).perp_dot(tv[2] - tv[0]) / den,
                    (tv[1] - tv[0]).perp_dot(q - tv[0]) / den,
                )
            } else {
                (0.0, 0.0)
            };
            let w0 = 1.0 - w1 - w2;
            let mut normal =
                normalize(corners[0].normal * w0 + corners[1].normal * w1 + corners[2].normal * w2);
            if flip {
                normal = -normal;
            }
            let uv = [0, 1]
                .map(|k| corners[0].uv[k] * w0 + corners[1].uv[k] * w1 + corners[2].uv[k] * w2);
            let color = [0, 1, 2, 3].map(|k| {
                corners[0].color[k] * w0 + corners[1].color[k] * w1 + corners[2].color[k] * w2
            });
            let pf = pos.as_vec3().to_array();
            let nf = normal.as_vec3().to_array();
            let uf = uv.map(|x| x as f32);
            let cf = color.map(|x| x as f32);
            let mut key: Vec<u32> = pf
                .iter()
                .chain(&nf)
                .chain(&uf)
                .map(|x| (x + 0.0).to_bits())
                .collect();
            if colors {
                key.extend(cf.map(|x| x.to_bits()));
            }
            let index = *vertex_at.entry(key).or_insert_with(|| {
                out.positions.push(pf);
                out.normals.push(nf);
                out.uvs.push(uf);
                if colors {
                    out.colors.push(cf);
                }
                (out.positions.len() - 1) as u32
            });
            out.indices.push(index);
        }
    }
    Ok((out, origins))
}

/// The f64 core of [`csg`] on welded operands: points and triangles in,
/// points, triangles and each output triangle's source `(operand,
/// triangle)` out. Unlike [`csg`], an operand that is not closed and
/// oriented after exact welding is refused (`docs/research.md` §5.4).
#[allow(clippy::type_complexity)]
pub fn csg_points(
    a: (&[DVec3], &[[u32; 3]]),
    b: (&[DVec3], &[[u32; 3]]),
    op: CsgOp,
) -> Result<(Vec<DVec3>, Vec<[u32; 3]>, Vec<(u8, u32)>), OperandError> {
    csg_points_traced(a, b, op).map(|r| (r.points, r.tris, r.origins))
}

/// What [`csg_points_traced`] returns.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CsgOutput {
    /// The points the triangles use.
    pub points: Vec<DVec3>,
    /// The result's triangles.
    pub tris: Vec<[u32; 3]>,
    /// Each triangle's source `(operand, input triangle)`.
    pub origins: Vec<(u8, u32)>,
    /// Pairs `(A's triangle, B's triangle)`, input indices, that lie in one
    /// plane and overlap, in the order found.
    pub coplanar: Vec<(u32, u32)>,
    /// Pairs that meet without crossing: one lies on one side of the
    /// other's plane, touching it (tangent faces, a vertex on a face).
    pub touching: Vec<(u32, u32)>,
    /// Whether the second pass (`docs/research.md` §5.4, item 5) built the
    /// result: the first came out open.
    pub second_pass: bool,
    /// Directed edges, after welding by exact position, used more often
    /// one way than the other: 0 for a closed result.
    pub unmatched: usize,
    /// Edges used four times, by two faces of each operand, one each way:
    /// where the operands touch along an edge.
    pub touching_edges: usize,
    /// Other edges used other than once each way.
    pub non_manifold: usize,
    /// Whether the result was built with the tool's vertices snapped onto
    /// the target's within a rounding error: the operands as given came
    /// out open or not manifold, and the snapped ones closer to closed
    /// (`docs/research.md` §18).
    pub snapped: bool,
}

/// One pass of the arrangement (`robust`: the second, §5.4 item 5), with
/// pairs of triangles within `flat` of each other's planes left uncut
/// (0: none; [`csg_points_repaired`]).
fn one_pass(
    a: (&[DVec3], &[[u32; 3]]),
    b: (&[DVec3], &[[u32; 3]]),
    op: CsgOp,
    robust: bool,
    flat: f64,
) -> Result<CsgOutput, OperandError> {
    let mut points = Vec::new();
    let mut at = BTreeMap::new();
    let sa = weld64(0, false, a, &mut points, &mut at)?;
    let sb = weld64(1, false, b, &mut points, &mut at)?;
    let (c, kept) = arrange(points, sa, sb, op, robust, flat);
    let (points, tris, origins, _) = emit(&c, &kept);
    let w = weld_ids(&points);
    // Zero-area triangles on one line (`sliver`): flipped away.
    let (tris, origins): (Vec<[u32; 3]>, Vec<(u8, u32)>) = sliver::flip(&points, &w, &tris)
        .into_iter()
        .map(|(t, src)| (t, origins[src]))
        .unzip();
    let mut counts = edge_counts(&w, &tris, &origins);
    let (tris, origins) = if counts.non_manifold + counts.touching > 0 {
        let (t, o) = cancel_fins(&w, tris, origins);
        counts = edge_counts(&w, &t, &o);
        (t, o)
    } else {
        (tris, origins)
    };
    let source = |pairs: &[TriPair]| {
        let mut v: Vec<(u32, u32)> = Vec::new();
        for &((ma, ia), (_, ib)) in pairs {
            let (sa, sb) = (
                c.solids[0].source[ia as usize],
                c.solids[1].source[ib as usize],
            );
            if ma == 0 && !v.contains(&(sa, sb)) {
                v.push((sa, sb));
            }
        }
        v
    };
    Ok(CsgOutput {
        coplanar: source(&c.coplanar_pairs),
        touching: source(&c.touch_pairs),
        points,
        tris,
        origins,
        second_pass: robust,
        unmatched: counts.unmatched,
        touching_edges: counts.touching,
        non_manifold: counts.non_manifold,
        snapped: false,
    })
}

/// How far a result is from closed: its unmatched edges (a union of two
/// closed solids is never empty: a region that leaked across the operands'
/// seam can drop every piece and balance); then its edges not used once
/// each way: where the operands touch along an edge, or a crack a
/// rounding error wide.
fn rank(o: &CsgOutput, op: CsgOp) -> (usize, usize) {
    let open = if op == CsgOp::Union && o.tris.is_empty() {
        usize::MAX
    } else {
        o.unmatched
    };
    (open, o.touching_edges + o.non_manifold)
}

/// [`csg_points`], with where the operands touch.
///
/// The first pass is the plain arrangement. When its result is open
/// (operands that touch: faces on faces, a vertex an ulp off a face), the
/// arrangement is built again by a second pass, and its result is taken
/// if it has fewer unmatched edges. The second pass keeps crossings that
/// round to one position apart, orders points on an edge exactly, lays a
/// triangle out exactly where its rounded outline is not simple, and then
/// makes points at one position one point, collapsing the runs between
/// them (`docs/research.md` §5.4, item 5). Every closed first-pass result
/// is returned as before. If the result is still open, or not manifold,
/// the tool's vertices within a rounding error of the target's are
/// snapped onto them ([`snap_onto`]) and both passes run again; the
/// snapped result is taken if it has fewer unmatched edges, or as many and
/// fewer edges not used once each way (a union is never empty).
/// [`CsgOutput::snapped`] says so.
pub fn csg_points_traced(
    a: (&[DVec3], &[[u32; 3]]),
    b: (&[DVec3], &[[u32; 3]]),
    op: CsgOp,
) -> Result<CsgOutput, OperandError> {
    let pass = |robust: bool, b: (&[DVec3], &[[u32; 3]])| one_pass(a, b, op, robust, 0.0);
    let open = |o: &CsgOutput| rank(o, op).0;
    let rank = |o: &CsgOutput| rank(o, op);
    let first = pass(false, b)?;
    // Operands that touch can make crossings a rounding error apart that
    // the first pass cannot keep apart; an open result there is built
    // again by the second pass. Every closed result is the first pass's,
    // unless it is left in several shells and the snap below joins them.
    if rank(&first) == (0, 0) {
        return Ok(rejoined(a, b, first, &pass));
    }
    let best = if open(&first) == 0 {
        first
    } else {
        let second = pass(true, b)?;
        if open(&second) < open(&first) {
            second
        } else {
            first
        }
    };
    if rank(&best) == (0, 0) {
        return Ok(best);
    }
    // Surfaces a rounding error apart that are not exactly one (faces
    // placed by different rounded motions): the tool's vertices within a
    // rounding error of the target's are snapped onto them, and both
    // passes run again on the snapped tool. That result is taken only if
    // it is closer to closed and manifold (`docs/research.md` §18).
    let Some(snapped) = snap_onto(a.0, b.0) else {
        return Ok(best);
    };
    let Ok(third) = pass(false, (&snapped, b.1)) else {
        return Ok(best);
    };
    let third = if open(&third) == 0 {
        third
    } else {
        match pass(true, (&snapped, b.1)) {
            Ok(fourth) if rank(&fourth) < rank(&third) => fourth,
            _ => third,
        }
    };
    Ok(if rank(&third) < rank(&best) {
        CsgOutput {
            snapped: true,
            ..third
        }
    } else {
        best
    })
}

/// [`csg_points_traced`], repaired as [`boolean`](crate::boolean::boolean) repairs
/// it ([`uncross_output`]), with what the repair did.
///
/// Where the operands touch and that result is open, not manifold, or
/// still crosses itself after the repair, faces of the operands may lie
/// within a rounding error of each other without being one plane (two
/// surfaces placed by different rounded motions), so each cuts the other
/// into pieces that fold over one another. Both passes then run once more
/// with each pair of triangles that lie within the snap distance of each
/// other's planes, every corner, without being exactly one plane, left
/// uncut (one surface; regions do not join such triangles to the rest).
/// That result, repaired, is taken only if, by exact predicates, it is
/// closed, manifold and crosses itself in fewer pairs; an open or
/// non-manifold result is replaced only by one that does not cross itself
/// at all (`docs/research.md` §27).
pub fn csg_points_repaired(
    a: (&[DVec3], &[[u32; 3]]),
    b: (&[DVec3], &[[u32; 3]]),
    op: CsgOp,
) -> Result<(CsgOutput, crate::boolean::uncross::Uncrossed), OperandError> {
    let built = csg_points_traced(a, b, op)?;
    let closed = rank(&built, op) == (0, 0);
    let (out, u) = uncross_output(a.0, b.0, built);
    if (out.coplanar.is_empty() && out.touching.is_empty()) || (closed && u.after == 0) {
        return Ok((out, u));
    }
    let tol = snap_distance(a.0, b.0);
    let mut best: Option<(CsgOutput, crate::boolean::uncross::Uncrossed)> = None;
    for robust in [false, true] {
        let Ok(o) = one_pass(a, b, op, robust, tol) else {
            continue;
        };
        if rank(&o, op) != (0, 0) || o.tris.is_empty() {
            continue;
        }
        let (o, v) = uncross_output(a.0, b.0, o);
        let bar = best
            .as_ref()
            .map_or(if closed { u.after } else { 1 }, |b| b.1.after);
        if v.after < bar {
            best = Some((o, v));
        }
        if v.after == 0 {
            break;
        }
    }
    Ok(best.unwrap_or((out, u)))
}
