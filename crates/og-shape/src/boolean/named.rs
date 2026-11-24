//! Booleans on named solids, carrying names through
//! (`docs/design.md` §4, `docs/research.md` §1.3).
//!
//! - Every output face keeps its operand's face names, and with an operand
//!   name also `<name>.<tag>`. Faces a cutting tool leaves get `cut`.
//! - Edges where the operands meet get `cut`, `<target>.cut`, the tool's
//!   name, and the cap names (`top`, `bottom`, `start`, `end`) of the
//!   faces they bound.
//! - An input edge that survives, whole or in pieces, keeps its names.
//! - A named edge left between coplanar faces (within 0.5°) loses them.
//!
//! UVs ([`Solid::uvs`]) are carried the same way: each output triangle
//! lies in one input triangle, and takes its UVs by barycentric
//! interpolation there, so a face the boolean cuts keeps its texture
//! mapping. When only one operand has UVs, the other's faces get (0, 0)
//! at every corner and [`BoolReport::faces_without_uvs`] counts them.
//! Vertex colours ([`Solid::colors`]) likewise: interpolated in the source
//! triangle, white on the faces of an operand without them, which
//! [`BoolReport::faces_without_colors`] counts.
//!
//! An operand whose surface passes through itself is refused
//! ([`BoolError::SelfIntersecting`]): by [`check_operands`] before any
//! work, or by [`boolean`] when its result comes out open. [`self_union`]
//! untangles one.
//!
//! What a result may be (`docs/research.md` §5.4, item 5):
//! - closed: every edge used once each way, after welding by position;
//! - or, where the operands touch along an edge (two boxes sharing only an
//!   edge, a box face on a tube's vertex column), non-manifold: each such
//!   edge is used by two faces of each operand, one each way. That is the
//!   correct union of solids that touch; [`BoolReport::touching_edges`]
//!   counts the edges. Any other non-manifold result is refused
//!   ([`BoolError::NotManifold`]), and an open one always
//!   ([`BoolError::OpenResult`], [`BoolError::OpenTouching`]).
//! - A cut whose tool lies wholly inside the target leaves a void: a
//!   closed inner shell facing in. That is the correct result (the volume
//!   drops by the tool's); [`BoolReport::voids`] counts such shells, and
//!   later booleans keep them ([`fuse`]).

use std::collections::BTreeMap;

use crate::math::{DVec2, DVec3};
use crate::mesh::{EdgeKey, Solid, edge_key};

use crate::boolean::csg::{
    CsgOp, OperandError, csg_points_repaired, csg_points_traced, self_union_points,
};
use crate::boolean::selfcheck;

/// The operation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BoolOp {
    /// A ∪ B.
    #[default]
    Join,
    /// A − B.
    Cut,
    /// A ∩ B.
    Intersect,
}

/// A boolean between a target and a tool.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Boolean<'a> {
    /// The operation.
    pub op: BoolOp,
    /// The target's part name, or "" for none.
    pub target: &'a str,
    /// The tool's name, or "" for none.
    pub tool: &'a str,
}

/// Why a boolean could not be honoured.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoolError {
    /// An operand is open, not oriented or not manifold ("target"/"tool").
    NotClosed(&'static str),
    /// An operand has a position that is not finite.
    NotFinite(&'static str),
    /// The result has no triangles (e.g. an intersection of disjoint solids).
    Empty,
    /// Both operands are skinned, by skeletons that differ: one solid
    /// has one skin.
    TwoSkins,
    /// The result came out open: `boundary` edges are not matched the
    /// other way, though both operands were closed and oriented, neither
    /// passes through itself, and they do not touch: a limit of the
    /// boolean on these operands. The open result is never returned.
    OpenResult {
        /// Directed edges with no twin.
        boundary: usize,
    },
    /// The result came out open where the operands touch: faces of one lie
    /// on faces of the other (coplanar), or meet them without crossing
    /// (tangent). `target` and `tool` name the faces involved (their face
    /// names, sorted; at most six each). The open result is never
    /// returned.
    OpenTouching {
        /// Directed edges with no twin.
        boundary: usize,
        /// The target's faces that touch the tool.
        target: Vec<String>,
        /// The tool's faces that touch the target.
        tool: Vec<String>,
    },
    /// The result is closed but not manifold beyond where the operands
    /// touch along an edge: `edges` edges are used by more than two faces
    /// in another way than two of each operand (`docs/research.md` §5.4,
    /// item 5). It is never returned.
    NotManifold {
        /// Such edges.
        edges: usize,
    },
    /// An operand ("target"/"tool") passes through itself: `pairs` pairs
    /// of its triangles, within one shell, cross where its surface does
    /// not join them (decided exactly, [`crossing_pairs`]). Found by
    /// [`check_operands`] before any work, or by [`boolean`] when the
    /// result came out open; [`self_union`] untangles such a shell.
    SelfIntersecting {
        /// Which operand.
        operand: &'static str,
        /// Crossing pairs of triangles.
        pairs: usize,
    },
}

impl std::fmt::Display for BoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoolError::NotClosed(w) => write!(
                f,
                "the {w} is not a closed, oriented solid; repair it first"
            ),
            BoolError::NotFinite(w) => write!(f, "the {w} has a position that is not finite"),
            BoolError::Empty => f.write_str("the result is empty"),
            BoolError::OpenResult { boundary } => write!(
                f,
                "the result is open ({boundary} unmatched edges); neither operand passes through itself and they do not touch: a limit of the boolean on these operands, which moving the tool slightly may avoid"
            ),
            BoolError::OpenTouching {
                boundary,
                target,
                tool,
            } => write!(
                f,
                "the result is open ({boundary} unmatched edges) where the operands touch along coplanar or tangent faces (target: {}; tool: {}); overlap them by a small amount, or separate them",
                names_or_none(target),
                names_or_none(tool)
            ),
            BoolError::NotManifold { edges } => write!(
                f,
                "the result is not manifold: {edges} edges are shared by more than two faces where the operands touch; overlap them by a small amount, or separate them"
            ),
            BoolError::SelfIntersecting { operand, pairs } => write!(
                f,
                "the {operand} passes through itself ({pairs} crossing triangle pairs); untangle it with self_union first"
            ),
            BoolError::TwoSkins => f.write_str(
                "the target and the tool are skinned by different skeletons, and one solid has one skin: skin them alike, or join first and skin the result",
            ),
        }
    }
}

impl std::error::Error for BoolError {}

fn names_or_none(v: &[String]) -> String {
    if v.is_empty() {
        "unnamed faces".into()
    } else {
        v.join(", ")
    }
}

/// What a boolean did that its result alone does not say
/// ([`boolean_report`]). Its `Debug` text lists `faces_without_colors`
/// only when it is not 0, so reports of solids without colours read as
/// they did before colours existed.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct BoolReport {
    /// Edges of the result shared by four faces, two of each operand:
    /// where the operands touch along an edge. The result is then the
    /// correct non-manifold union, and `measure` reports it not closed.
    pub touching_edges: usize,
    /// Closed shells of the result facing in: voids, as a cut whose tool
    /// lies wholly inside the target leaves.
    pub voids: usize,
    /// Whether the operands touch: faces of one lie on faces of the other,
    /// or meet them without crossing.
    pub touching: bool,
    /// Faces of the result given (0, 0) UVs because they came from the
    /// operand without UVs while the other had them (0 when both or
    /// neither have UVs).
    pub faces_without_uvs: usize,
    /// Faces of the result given white vertex colours because they came
    /// from the operand without colours while the other had them (0 when
    /// both or neither have colours).
    pub faces_without_colors: usize,
    /// Pairs of the result's triangles that still cross after the repair
    /// of rounding errors ([`uncross_output`](crate::boolean::uncross_output)), at
    /// least one with a corner the boolean made: 0 for a result that does
    /// not pass through itself where the operands meet. Pairs an operand
    /// brought crossing are not counted ([`check_operands`] finds them). A
    /// measurement, never a refusal (`docs/research.md` §27).
    pub crossing_pairs: usize,
}

impl core::fmt::Debug for BoolReport {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut d = f.debug_struct("BoolReport");
        d.field("touching_edges", &self.touching_edges)
            .field("voids", &self.voids)
            .field("touching", &self.touching)
            .field("faces_without_uvs", &self.faces_without_uvs);
        if self.faces_without_colors != 0 {
            d.field("faces_without_colors", &self.faces_without_colors);
        }
        d.field("crossing_pairs", &self.crossing_pairs).finish()
    }
}

const CAPS: [&str; 4] = ["top", "bottom", "start", "end"];
/// cos 0.5°: faces closer than this are coplanar for the naming rule.
const COPLANAR_COS: f64 = 0.999_961_923_064_171_3;

/// The solid's connected pieces (sharing a vertex after exact welding),
/// each a solid with its names, in canonical order.
pub fn components(s: &Solid) -> Vec<Solid> {
    let mut parent: Vec<u32> = (0..s.positions.len() as u32).collect();
    fn find(p: &mut [u32], mut i: u32) -> u32 {
        while p[i as usize] != i {
            p[i as usize] = p[p[i as usize] as usize];
            i = p[i as usize];
        }
        i
    }
    for t in &s.triangles {
        for k in 1..3 {
            let (a, b) = (find(&mut parent, t[0]), find(&mut parent, t[k]));
            if a != b {
                parent[a.max(b) as usize] = a.min(b);
            }
        }
    }
    let mut groups: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
    for (i, t) in s.triangles.iter().enumerate() {
        groups.entry(find(&mut parent, t[0])).or_default().push(i);
    }
    groups
        .into_values()
        .map(|tris| {
            // The skin rides along whole; canonicalize keeps the weights
            // of the vertices this piece uses.
            let mut c = Solid {
                positions: s.positions.clone(),
                skin: s.skin.clone(),
                morphs: s.morphs.clone(),
                uvs: s.uvs.as_ref().map(|_| Vec::new()),
                colors: s.colors.as_ref().map(|_| Vec::new()),
                ..Default::default()
            };
            for i in tris {
                let tags: Vec<&str> = s.face_tags[i].iter().map(|&x| s.names.name(x)).collect();
                c.push_triangle(s.triangles[i], s.surfaces[i], &tags);
                if let (Some(to), Some(from)) = (&mut c.uvs, &s.uvs) {
                    to[c.triangles.len() - 1] = from[i];
                }
                if let (Some(to), Some(from)) = (&mut c.colors, &s.colors) {
                    to[c.triangles.len() - 1] = from[i];
                }
            }
            for (&(a, b), tags) in &s.edge_tags {
                let tags: Vec<&str> = tags.iter().map(|&x| s.names.name(x)).collect();
                c.tag_edge(a, b, &tags);
            }
            // Canonicalize drops the other pieces' vertices and edges.
            let _ = c.canonicalize();
            c
        })
        .collect()
}

/// Fuses a solid made of several closed shells (`merge`, or a provider's
/// interpenetrating parts) into one: each shell that passes through itself
/// is untangled first ([`self_union`]), then the shells are joined in
/// turn. A shell facing in (a void a cut left) is cut from the rest
/// instead, so the void stays. A solid of one shell that does not pass
/// through itself comes back as it is.
pub fn fuse(s: &Solid) -> Result<Solid, BoolError> {
    if crate::mesh::rides(s) {
        let mut out = fuse(&crate::mesh::bare(s))?;
        let _ = crate::mesh::carry(&[s], &mut out);
        return Ok(out);
    }
    // A shell that crosses itself only by rounding (an earlier boolean's
    // output) may not untangle closed; it is then joined as it is, and the
    // join's own check on its result still holds.
    let untangled = |p: Solid| -> Result<Solid, BoolError> {
        if crossing_pairs(&p) > 0 {
            Ok(self_union(&p).unwrap_or(p))
        } else {
            Ok(p)
        }
    };
    let (shells, voids): (Vec<Solid>, Vec<Solid>) = components(s)
        .into_iter()
        .partition(|p| p.measure().volume >= 0.0);
    let mut parts = shells.into_iter();
    let Some(first) = parts.next() else {
        return Err(BoolError::Empty);
    };
    let mut acc = untangled(first)?;
    for p in parts {
        acc = boolean_once(&acc, &untangled(p)?, &Boolean::default())?.0;
    }
    for mut v in voids {
        for i in 0..v.triangles.len() {
            v.reverse_triangle(i);
        }
        let d = Boolean {
            op: BoolOp::Cut,
            ..Default::default()
        };
        acc = boolean_once(&acc, &v, &d)?.0;
    }
    Ok(acc)
}

/// Pairs of triangles, within one shell ([`components`]), that cross where
/// the surface does not join them, after welding by exact position: they
/// share no vertex and meet, or share a vertex or an edge and overlap
/// beyond it ([`selfcheck::crossing`], exact predicates). Zero for a shell
/// that does not pass through itself; separate shells passing through
/// each other are not counted ([`fuse`] joins those).
pub fn crossing_pairs(s: &Solid) -> usize {
    // Each triangle's shell, as `components` finds them.
    let mut parent: Vec<u32> = (0..s.positions.len() as u32).collect();
    fn find(p: &mut [u32], mut i: u32) -> u32 {
        while p[i as usize] != i {
            p[i as usize] = p[p[i as usize] as usize];
            i = p[i as usize];
        }
        i
    }
    for t in &s.triangles {
        for k in 1..3 {
            let (a, b) = (find(&mut parent, t[0]), find(&mut parent, t[k]));
            if a != b {
                parent[a.max(b) as usize] = a.min(b);
            }
        }
    }
    let shell: Vec<u32> = s
        .triangles
        .iter()
        .map(|t| find(&mut parent, t[0]))
        .collect();
    let (w, _) = weld(&s.positions);
    let tris: Vec<[u32; 3]> = s
        .triangles
        .iter()
        .map(|t| t.map(|v| w[v as usize]))
        .collect();
    let mut points = vec![DVec3::ZERO; s.positions.len()];
    for (i, &p) in s.positions.iter().enumerate() {
        points[w[i] as usize] = p;
    }
    // Triangles welded down to an edge or a point are left out, as the
    // booleans drop them.
    let live: Vec<u32> = (0..tris.len() as u32)
        .filter(|&i| {
            let t = tris[i as usize];
            t[0] != t[1] && t[1] != t[2] && t[0] != t[2]
        })
        .collect();
    let live_tris: Vec<[u32; 3]> = live.iter().map(|&i| tris[i as usize]).collect();
    selfcheck::self_crossings(&points, &live_tris)
        .into_iter()
        .filter(|&(i, j)| shell[live[i as usize] as usize] == shell[live[j as usize] as usize])
        .count()
}

/// Untangles a solid whose surface passes through itself: its outside
/// surface, around the space its surface winds round at least once
/// ([`self_union_points`]). Separate shells passing through each other are
/// joined too. Names are carried as [`fuse`] carries them: every face
/// keeps its face names and surface; edges along which the surface
/// crossed itself are named `cut`, plus the cap names of the faces they
/// bound; pieces of named input edges keep their names.
///
/// Refused: a solid that is not closed and oriented after
/// welding, an empty result, or an open one.
pub fn self_union(s: &Solid) -> Result<Solid, BoolError> {
    if crate::mesh::rides(s) {
        let mut out = self_union(&crate::mesh::bare(s))?;
        let _ = crate::mesh::carry(&[s], &mut out);
        return Ok(out);
    }
    let u = self_union_points((&s.positions, &s.triangles)).map_err(|e| match e {
        OperandError::NotClosed(_) => BoolError::NotClosed("target"),
        OperandError::NotFinite(_) => BoolError::NotFinite("target"),
    })?;
    if u.tris.is_empty() {
        return Err(BoolError::Empty);
    }
    let boundary = open_edges(&u.points, &u.tris);
    if boundary > 0 {
        return Err(BoolError::OpenResult { boundary });
    }
    let origins: Vec<(u8, u32)> = u.source.iter().map(|&t| (0, t)).collect();
    let seams: std::collections::BTreeSet<EdgeKey> =
        u.seams.iter().map(|&[a, b]| edge_key(a, b)).collect();
    named_result(
        u.points,
        &u.tris,
        &origins,
        [(s, ""), (s, "")],
        &Boolean::default(),
        |e, _| seams.contains(&e),
    )
}

/// `target op tool`, with names carried through. An operand of several
/// shells is fused first ([`fuse`]). [`boolean_report`] also says what the
/// result alone does not.
///
/// Refused: operands skinned by different skeletons ([`BoolError::TwoSkins`]); an open result,
/// never returned; a non-manifold one beyond where the operands touch
/// along an edge ([`BoolError::NotManifold`]). An open result is
/// diagnosed: an operand that passes through itself
/// ([`BoolError::SelfIntersecting`]), operands that touch
/// ([`BoolError::OpenTouching`]), or neither ([`BoolError::OpenResult`]).
/// [`check_operands`] refuses an operand that passes through itself before
/// any work.
pub fn boolean(target: &Solid, tool: &Solid, d: &Boolean) -> Result<Solid, BoolError> {
    boolean_report(target, tool, d).map(|(s, _)| s)
}

/// [`boolean`], with what the result alone does not say: where the
/// operands touch, and the voids left ([`BoolReport`]).
pub fn boolean_report(
    target: &Solid,
    tool: &Solid,
    d: &Boolean,
) -> Result<(Solid, BoolReport), BoolError> {
    if crate::mesh::rides(target) || crate::mesh::rides(tool) {
        // Morph targets and the skin ride the result: each of its
        // vertices moves as the operand's surface it lies on does there.
        crate::mesh::one_skin(&[target, tool]).map_err(|_| BoolError::TwoSkins)?;
        let (mut out, report) =
            boolean_report(&crate::mesh::bare(target), &crate::mesh::bare(tool), d)?;
        let _ = crate::mesh::carry(&[target, tool], &mut out);
        return Ok((out, report));
    }
    let fused = |s: &Solid| {
        if components(s).len() > 1 {
            fuse(s)
        } else {
            Ok(s.clone())
        }
    };
    match boolean_once(&fused(target)?, &fused(tool)?, d) {
        Err(e @ (BoolError::OpenResult { .. } | BoolError::OpenTouching { .. })) => {
            // Diagnose before blaming: an operand that passes through
            // itself first, then the touching faces the error names.
            check_operands(target, tool)?;
            Err(e)
        }
        r => r,
    }
}

/// Refuses, before any work, an operand with a shell that passes through
/// itself ([`BoolError::SelfIntersecting`], counted by
/// [`crossing_pairs`]); the target is checked first. [`boolean`] does not
/// call it up front: the kernel's own boolean outputs can cross themselves
/// by a rounding error where crossing points meet (an ulp, along a
/// countersink's rim), and booleans on them come out closed
/// (`docs/research.md` §5.4). Call it on imported or generated meshes;
/// it costs a sweep over each operand's triangles.
pub fn check_operands(target: &Solid, tool: &Solid) -> Result<(), BoolError> {
    for (s, operand) in [(target, "target"), (tool, "tool")] {
        let pairs = crossing_pairs(s);
        if pairs > 0 {
            return Err(BoolError::SelfIntersecting { operand, pairs });
        }
    }
    Ok(())
}

/// Whether [`boolean`] repairs a result that crosses itself by a
/// rounding error ([`uncross_output`](crate::boolean::uncross_output)), and builds
/// faces a rounding error apart as one surface where that leaves it
/// crossing ([`csg_points_repaired`]): on (`docs/research.md` §23.2, §27).
/// A result that does not cross itself is returned as built.
const UNCROSS: bool = true;

fn boolean_once(
    target: &Solid,
    tool: &Solid,
    d: &Boolean,
) -> Result<(Solid, BoolReport), BoolError> {
    let op = match d.op {
        BoolOp::Join => CsgOp::Union,
        BoolOp::Cut => CsgOp::Subtract,
        BoolOp::Intersect => CsgOp::Intersect,
    };
    let a = (&target.positions[..], &target.triangles[..]);
    let b = (&tool.positions[..], &tool.triangles[..]);
    let (out, crossing_pairs) = if UNCROSS {
        csg_points_repaired(a, b, op).map(|(out, u)| (out, u.after))
    } else {
        csg_points_traced(a, b, op).map(|out| (out, 0))
    }
    .map_err(|e| match e {
        OperandError::NotClosed(0) => BoolError::NotClosed("target"),
        OperandError::NotClosed(_) => BoolError::NotClosed("tool"),
        OperandError::NotFinite(0) => BoolError::NotFinite("target"),
        OperandError::NotFinite(_) => BoolError::NotFinite("tool"),
    })?;
    let touching = !out.coplanar.is_empty() || !out.touching.is_empty();
    if out.tris.is_empty() {
        return Err(BoolError::Empty);
    }
    // Never return open output (`docs/design.md` §6): every directed edge,
    // after welding by exact position, needs a twin the other way.
    let boundary = out.unmatched;
    if boundary > 0 {
        if !touching {
            return Err(BoolError::OpenResult { boundary });
        }
        let names = |s: &Solid, tris: &mut dyn Iterator<Item = u32>| {
            let mut v: Vec<String> = Vec::new();
            for t in tris {
                for &x in s.face_tags[t as usize].iter() {
                    let n = s.names.name(x).to_string();
                    if !v.contains(&n) {
                        v.push(n);
                    }
                }
            }
            v.sort();
            v.truncate(6);
            v
        };
        let pairs: Vec<(u32, u32)> = out.coplanar.iter().chain(&out.touching).copied().collect();
        return Err(BoolError::OpenTouching {
            boundary,
            target: names(target, &mut pairs.iter().map(|p| p.0)),
            tool: names(tool, &mut pairs.iter().map(|p| p.1)),
        });
    }
    // Directed edges that balance can still be used four times or more
    // (two coincident faces kept, a pinch): only an edge where the operands
    // touch, two faces of each, one each way, is a correct non-manifold
    // result.
    if out.non_manifold > 0 {
        return Err(BoolError::NotManifold {
            edges: out.non_manifold,
        });
    }
    let touching_edges = out.touching_edges;
    let faces_without_uvs = if target.uvs.is_some() != tool.uvs.is_some() {
        let bare = u8::from(target.uvs.is_some());
        out.origins.iter().filter(|o| o.0 == bare).count()
    } else {
        0
    };
    let faces_without_colors = if target.colors.is_some() != tool.colors.is_some() {
        let bare = u8::from(target.colors.is_some());
        out.origins.iter().filter(|o| o.0 == bare).count()
    } else {
        0
    };
    let s = named_result(
        out.points,
        &out.tris,
        &out.origins,
        [(target, d.target), (tool, d.tool)],
        d,
        |_, ms| ms.contains(&0) && ms.contains(&1),
    )?;
    let voids = void_shells(&s);
    Ok((
        s,
        BoolReport {
            touching_edges,
            voids,
            touching,
            faces_without_uvs,
            faces_without_colors,
            crossing_pairs,
        },
    ))
}

/// Shells (pieces sharing a vertex) whose signed volume is negative: they
/// face in, voids inside another shell.
fn void_shells(s: &Solid) -> usize {
    let mut parent: Vec<u32> = (0..s.positions.len() as u32).collect();
    fn find(p: &mut [u32], mut i: u32) -> u32 {
        while p[i as usize] != i {
            p[i as usize] = p[p[i as usize] as usize];
            i = p[i as usize];
        }
        i
    }
    for t in &s.triangles {
        for k in 1..3 {
            let (a, b) = (find(&mut parent, t[0]), find(&mut parent, t[k]));
            if a != b {
                parent[a.max(b) as usize] = a.min(b);
            }
        }
    }
    let mut volume: BTreeMap<u32, f64> = BTreeMap::new();
    for t in &s.triangles {
        let [a, b, c] = t.map(|v| s.positions[v as usize]);
        *volume.entry(find(&mut parent, t[0])).or_default() += a.dot(b.cross(c)) / 6.0;
    }
    volume.values().filter(|&&v| v < 0.0).count()
}

/// The output of a boolean (or self-union) as a solid with names:
/// `origins` gives each triangle's `(operand, source triangle)`, and
/// `seam` whether an output edge, with its triangles' operands, is where
/// the operands met.
fn named_result(
    points: Vec<DVec3>,
    tris: &[[u32; 3]],
    origins: &[(u8, u32)],
    operands: [(&Solid, &str); 2],
    d: &Boolean,
    seam: impl Fn(EdgeKey, &[u8]) -> bool,
) -> Result<Solid, BoolError> {
    let (target, tool) = (operands[0].0, operands[1].0);
    let surface_offset = target.surfaces.iter().max().map_or(0, |s| s + 1);
    let mut out = Solid {
        positions: points,
        ..Default::default()
    };
    for (t, &(m, src)) in tris.iter().zip(origins) {
        let (solid, name) = operands[m as usize];
        let mut tags: Vec<String> = solid.face_tags[src as usize]
            .iter()
            .map(|&x| solid.names.name(x).to_string())
            .collect();
        if !name.is_empty() {
            let qualified: Vec<String> = tags
                .iter()
                .filter(|t| !t.contains('.'))
                .map(|t| format!("{name}.{t}"))
                .collect();
            tags.extend(qualified);
        }
        if m == 1 && d.op == BoolOp::Cut {
            tags.push("cut".into());
        }
        let refs: Vec<&str> = tags.iter().map(String::as_str).collect();
        let surface = solid.surfaces[src as usize] + if m == 1 { surface_offset } else { 0 };
        out.push_triangle(*t, surface, &refs);
        if target.uvs.is_some() || tool.uvs.is_some() {
            let uv = solid.uvs.as_ref().map_or([DVec2::ZERO; 3], |u| {
                let at = solid.triangles[src as usize].map(|v| solid.positions[v as usize]);
                t.map(|v| interpolate(at, u[src as usize], out.positions[v as usize]))
            });
            let n = out.triangles.len();
            out.uvs
                .get_or_insert_with(Vec::new)
                .resize(n, [DVec2::ZERO; 3]);
            if let Some(u) = &mut out.uvs {
                u[n - 1] = uv;
            }
        }
        if let Some(c) = &solid.colors {
            let at = solid.triangles[src as usize].map(|v| solid.positions[v as usize]);
            let c = c[src as usize].map(crate::mesh::wide);
            let color = t.map(|v| {
                let w = barycentric(at, out.positions[v as usize]);
                crate::mesh::stored(core::array::from_fn(|k| {
                    c[0][k] * w[0] + c[1][k] * w[1] + c[2][k] * w[2]
                }))
            });
            let n = out.triangles.len();
            out.set_colors(n - 1, color);
        }
    }
    // Edges: each output edge's two triangles.
    let mut by_edge: BTreeMap<EdgeKey, Vec<usize>> = BTreeMap::new();
    for (i, t) in out.triangles.iter().enumerate() {
        for k in 0..3 {
            by_edge
                .entry(edge_key(t[k], t[(k + 1) % 3]))
                .or_default()
                .push(i);
        }
    }
    let normal = |s: &Solid, t: [u32; 3]| {
        let [a, b, c] = t.map(|v| s.positions[v as usize]);
        (b - a).cross(c - a).normalize_or_zero()
    };
    let input_edges: [Vec<(DVec3, DVec3, Vec<String>)>; 2] = [target, tool].map(|s| {
        s.edge_tags
            .iter()
            .map(|(&(a, b), tags)| {
                (
                    s.positions[a as usize],
                    s.positions[b as usize],
                    tags.iter().map(|&x| s.names.name(x).to_string()).collect(),
                )
            })
            .collect()
    });
    let mut edge_names: Vec<(EdgeKey, Vec<String>)> = Vec::new();
    for (&(u, v), ts) in &by_edge {
        let (pu, pv) = (out.positions[u as usize], out.positions[v as usize]);
        let mut names: Vec<String> = Vec::new();
        let ms: Vec<u8> = ts.iter().map(|&t| origins[t].0).collect();
        if seam((u, v), &ms) {
            names.push("cut".into());
            if !d.target.is_empty() {
                names.push(format!("{}.cut", d.target));
            }
            if !d.tool.is_empty() {
                names.push(d.tool.to_string());
            }
            for &t in ts {
                let (solid, _) = operands[origins[t].0 as usize];
                for &x in solid.face_tags[origins[t].1 as usize].iter() {
                    let n = solid.names.name(x);
                    if CAPS.contains(&n) {
                        names.push(n.to_string());
                    }
                }
            }
        }
        // Pieces of named input edges keep their names.
        for &m in &ms {
            for (p, q, tags) in &input_edges[m as usize] {
                if on_segment(*p, *q, pu) && on_segment(*p, *q, pv) {
                    names.extend(tags.iter().cloned());
                }
            }
        }
        if names.is_empty() {
            continue;
        }
        // Coplanar faces on both sides: not a real edge any more.
        if ts.len() == 2
            && normal(&out, out.triangles[ts[0]]).dot(normal(&out, out.triangles[ts[1]]))
                > COPLANAR_COS
        {
            continue;
        }
        names.sort();
        names.dedup();
        edge_names.push(((u, v), names));
    }
    for ((u, v), names) in edge_names {
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        out.tag_edge(u, v, &refs);
    }
    out.canonicalize()
        .map_err(|_| BoolError::NotFinite("result"))?;
    Ok(out)
}

/// The barycentric weights of `p` in triangle `at`, as [`interpolate`]
/// measures them; `[1, 0, 0]` for a degenerate triangle.
fn barycentric(at: [DVec3; 3], p: DVec3) -> [f64; 3] {
    let [a, b, c] = at;
    let n = (b - a).cross(c - a);
    let whole = n.dot(n);
    if whole <= 0.0 || whole.is_nan() {
        return [1.0, 0.0, 0.0];
    }
    [
        (b - p).cross(c - p).dot(n) / whole,
        (c - p).cross(a - p).dot(n) / whole,
        (a - p).cross(b - p).dot(n) / whole,
    ]
}

/// The UV at `p`, a point in the plane of triangle `at` (to rounding),
/// by barycentric interpolation of its corners' `uv`: areas measured
/// along the triangle's normal. A degenerate triangle gives its first
/// corner's UV.
fn interpolate(at: [DVec3; 3], uv: [DVec2; 3], p: DVec3) -> DVec2 {
    let [a, b, c] = at;
    let n = (b - a).cross(c - a);
    let whole = n.dot(n);
    if whole <= 0.0 || whole.is_nan() {
        return uv[0];
    }
    let la = (b - p).cross(c - p).dot(n) / whole;
    let lb = (c - p).cross(a - p).dot(n) / whole;
    let lc = (a - p).cross(b - p).dot(n) / whole;
    uv[0] * la + uv[1] * lb + uv[2] * lc
}

/// Directed edges, after welding by exact position, used more often one
/// way than the other (zero-area triangles skipped).
fn open_edges(points: &[DVec3], tris: &[[u32; 3]]) -> usize {
    let (w, _) = weld(points);
    let mut directed: BTreeMap<(u32, u32), i64> = BTreeMap::new();
    for t in tris {
        let t = t.map(|v| w[v as usize]);
        if t[0] == t[1] || t[1] == t[2] || t[0] == t[2] {
            continue;
        }
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            *directed.entry((a.min(b), a.max(b))).or_default() += if a < b { 1 } else { -1 };
        }
    }
    directed.values().map(|n| n.unsigned_abs() as usize).sum()
}

/// Each point's id after welding by exact position (ids in first-seen
/// order), and how many there are.
fn weld(points: &[DVec3]) -> (Vec<u32>, usize) {
    let mut id: BTreeMap<[u64; 3], u32> = BTreeMap::new();
    let w = points
        .iter()
        .map(|p| {
            let n = id.len() as u32;
            *id.entry([
                (p.x + 0.0).to_bits(),
                (p.y + 0.0).to_bits(),
                (p.z + 0.0).to_bits(),
            ])
            .or_insert(n)
        })
        .collect();
    (w, id.len())
}

/// Is `x` on the segment `p`–`q` (to 1e-9 of its length)?
fn on_segment(p: DVec3, q: DVec3, x: DVec3) -> bool {
    let d = q - p;
    let l2 = d.length_squared();
    if l2 == 0.0 {
        return false;
    }
    let w = x - p;
    let t = w.dot(d);
    if t < -1e-9 * l2 || t > l2 * (1.0 + 1e-9) {
        return false;
    }
    d.cross(w).length_squared() <= 1e-18 * l2 * l2
}
