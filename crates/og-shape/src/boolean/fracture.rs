//! Voronoi prefracture: a closed mesh cut into the cells of a set of seed
//! points, each cell's cut faces tagged with the neighbouring cell it
//! faces, so the pieces carry Blast's bond description (the surface two
//! neighbours share: its centroid, its normal and its area; NVIDIA Blast,
//! `docs/api/introduction.rst`, BSD-3, `third_party/blast/`).
//!
//! The method follows three-pinata's Voronoi fracture (Daniel Greenheck,
//! MIT, `third_party/three-pinata/`): each cell is the half-space
//! intersection of the bisecting planes between its seed and every other
//! seed (`VoronoiCell.ts` `computeVoronoiCell`), and the mesh is cut to it.
//! Where three-pinata slices the mesh plane by plane and triangulates each
//! cap, here the convex cell is built first (a box clipped by the planes)
//! and the mesh is intersected with it by [`crate::boolean::csg`], whose exact
//! predicates keep every piece closed and the same on every host; the cut
//! faces are the cell's faces kept inside the mesh. A non-convex mesh can
//! leave a cell in several separate pieces, each its own chunk
//! (three-pinata's `findIsolatedGeometry`).
//!
//! Every number is f64 with `+ - * /` and `sqrt`: the same on every host.

use crate::boolean::csg::{CsgOp, csg_traced};
use crate::generators::{Mesh, Placement};
use glam::DVec3;
use std::collections::BTreeMap;

/// A cut face's tag in [`Piece::faces`]: the bounds of the cells (a cell
/// face no neighbour shares).
pub const BOUNDS: u32 = u32::MAX;

/// One chunk: a closed mesh in the input's space.
#[derive(Clone, Debug, Default)]
pub struct Piece {
    /// The cell it lies in.
    pub cell: u32,
    pub mesh: Mesh,
    /// Per triangle: the input material it came from, or `None` for a cut
    /// (interior) face.
    pub material: Vec<Option<u32>>,
    /// Per triangle: the neighbouring cell a cut face faces ([`BOUNDS`] for
    /// none), or `None` for an outer face.
    pub faces: Vec<Option<u32>>,
    pub volume: f64,
    pub centroid: DVec3,
}

/// The surface two pieces share.
#[derive(Clone, Debug, PartialEq)]
pub struct Bond {
    /// Indices into the pieces, `a < b`.
    pub a: u32,
    pub b: u32,
    pub centroid: DVec3,
    /// From a towards b.
    pub normal: DVec3,
    pub area: f64,
    /// The actual overlap, triangulated in model space, not the AABBs of
    /// opposing cut faces. Used to cook contact section moments.
    pub section: Vec<[DVec3; 3]>,
}

/// The pieces of a fracture and the bonds between them.
#[derive(Clone, Debug, Default)]
pub struct Fracture {
    pub pieces: Vec<Piece>,
    pub bonds: Vec<Bond>,
}

/// A convex cell as faces (each a convex polygon, counter-clockwise from
/// outside) with the tag of what made them.
struct Cell {
    faces: Vec<(Vec<DVec3>, DVec3, u32)>,
}

impl Cell {
    fn boxed(lo: DVec3, hi: DVec3) -> Cell {
        let c = |x: usize, y: usize, z: usize| {
            DVec3::new(
                if x == 0 { lo.x } else { hi.x },
                if y == 0 { lo.y } else { hi.y },
                if z == 0 { lo.z } else { hi.z },
            )
        };
        let quad = |v: [DVec3; 4], n: DVec3| (v.to_vec(), n, BOUNDS);
        Cell {
            faces: vec![
                quad([c(1, 0, 0), c(1, 1, 0), c(1, 1, 1), c(1, 0, 1)], DVec3::X),
                quad(
                    [c(0, 0, 0), c(0, 0, 1), c(0, 1, 1), c(0, 1, 0)],
                    DVec3::NEG_X,
                ),
                quad([c(0, 1, 0), c(0, 1, 1), c(1, 1, 1), c(1, 1, 0)], DVec3::Y),
                quad(
                    [c(0, 0, 0), c(1, 0, 0), c(1, 0, 1), c(0, 0, 1)],
                    DVec3::NEG_Y,
                ),
                quad([c(0, 0, 1), c(1, 0, 1), c(1, 1, 1), c(0, 1, 1)], DVec3::Z),
                quad(
                    [c(0, 0, 0), c(0, 1, 0), c(1, 1, 0), c(1, 0, 0)],
                    DVec3::NEG_Z,
                ),
            ],
        }
    }

    /// Keeps the part where `n · x <= d`, its new face tagged `tag`.
    fn clip(&mut self, n: DVec3, d: f64, tag: u32, eps: f64) {
        let mut faces = Vec::new();
        let mut cap: Vec<DVec3> = Vec::new();
        for (poly, normal, t) in self.faces.drain(..) {
            let mut out = Vec::new();
            for i in 0..poly.len() {
                let (p, q) = (poly[i], poly[(i + 1) % poly.len()]);
                let (sp, sq) = (n.dot(p) - d, n.dot(q) - d);
                if sp <= eps {
                    out.push(p);
                }
                if sp.abs() <= eps {
                    cap.push(p);
                }
                if (sp < -eps && sq > eps) || (sp > eps && sq < -eps) {
                    let x = p + (q - p) * (sp / (sp - sq));
                    out.push(x);
                    cap.push(x);
                }
            }
            if out.len() >= 3 {
                faces.push((out, normal, t));
            }
        }
        self.faces = faces;
        let cap = dedup(cap, eps);
        if cap.len() >= 3 {
            self.faces.push((order(cap, n), n, tag));
        }
    }

    fn is_empty(&self) -> bool {
        self.faces.len() < 4
    }

    /// The cell as a closed mesh: each face a fan, flat-shaded, its uvs a
    /// box projection by `uv_scale` (the same on both sides of a shared
    /// face, so the interior texture runs across a crack); the tag of each
    /// triangle.
    fn mesh(&self, eps: f64, uv_scale: f64) -> (Mesh, Vec<u32>) {
        // One position per corner the faces share, so the mesh is closed
        // bit for bit (each face clipped its corners on its own).
        let mut points: Vec<DVec3> = Vec::new();
        let mut id = |p: DVec3| -> DVec3 {
            match points.iter().find(|q| (**q - p).length() <= eps) {
                Some(q) => *q,
                None => {
                    points.push(p);
                    p
                }
            }
        };
        let mut m = Mesh::default();
        let mut tags = Vec::new();
        for (poly, n, tag) in &self.faces {
            let poly: Vec<DVec3> = poly.iter().map(|&p| id(p)).collect();
            let poly = dedup_ring(poly);
            if poly.len() < 3 {
                continue;
            }
            let base = m.positions.len() as u32;
            for &p in &poly {
                m.positions.push(p.as_vec3().to_array());
                m.normals.push(n.as_vec3().to_array());
                m.uvs.push(box_uv(p, *n, uv_scale));
            }
            for k in 1..poly.len() as u32 - 1 {
                m.indices.extend([base, base + k, base + k + 1]);
                tags.push(*tag);
            }
        }
        (m, tags)
    }
}

/// The uv of a cut face at `p`: `p` projected along the axis its normal
/// leans on most, times `scale`.
pub fn box_uv(p: DVec3, n: DVec3, scale: f64) -> [f32; 2] {
    let a = n.abs();
    let (u, v) = if a.x >= a.y && a.x >= a.z {
        (p.z, p.y)
    } else if a.y >= a.z {
        (p.x, p.z)
    } else {
        (p.x, p.y)
    };
    [(u * scale) as f32, (-v * scale) as f32]
}

fn dedup(points: Vec<DVec3>, eps: f64) -> Vec<DVec3> {
    let mut out: Vec<DVec3> = Vec::new();
    for p in points {
        if !out.iter().any(|q| (*q - p).length() <= eps) {
            out.push(p);
        }
    }
    out
}

fn dedup_ring(poly: Vec<DVec3>) -> Vec<DVec3> {
    let mut out: Vec<DVec3> = Vec::with_capacity(poly.len());
    for p in poly {
        if out.last() != Some(&p) {
            out.push(p);
        }
    }
    while out.len() > 1 && out.first() == out.last() {
        out.pop();
    }
    out
}

/// Points on a plane of normal `n` in counter-clockwise order seen from
/// `+n`, by half-plane and cross product about their centroid (no angle).
fn order(mut pts: Vec<DVec3>, n: DVec3) -> Vec<DVec3> {
    let c = pts.iter().copied().sum::<DVec3>() / pts.len() as f64;
    let u = (pts[0] - c).normalize_or_zero();
    let v = n.cross(u);
    let key = |p: &DVec3| {
        let d = *p - c;
        let (x, y) = (d.dot(u), d.dot(v));
        (
            if y > 0.0 || (y == 0.0 && x >= 0.0) {
                0
            } else {
                1
            },
            x,
            y,
        )
    };
    pts.sort_by(|a, b| {
        let (ha, xa, ya) = key(a);
        let (hb, xb, yb) = key(b);
        ha.cmp(&hb).then_with(|| {
            // a before b when b is counter-clockwise of a.
            let cross = xa * yb - ya * xb;
            0.0f64.total_cmp(&cross)
        })
    });
    pts
}

/// The signed volume and centroid of a closed mesh.
pub fn volume_centroid(m: &Mesh) -> (f64, DVec3) {
    let p = |i: u32| DVec3::from(m.positions[i as usize].map(f64::from));
    let (mut vol, mut sum) = (0.0, DVec3::ZERO);
    for t in m.indices.as_chunks::<3>().0 {
        let (a, b, c) = (p(t[0]), p(t[1]), p(t[2]));
        let v = a.dot(b.cross(c)) / 6.0;
        vol += v;
        sum += (a + b + c) * (v / 4.0);
    }
    let centroid = if vol.abs() > 0.0 {
        sum / vol
    } else {
        DVec3::ZERO
    };
    (vol, centroid)
}

fn tri_area(m: &Mesh, t: &[u32; 3]) -> (f64, DVec3, DVec3) {
    let p = |i: u32| DVec3::from(m.positions[i as usize].map(f64::from));
    let (a, b, c) = (p(t[0]), p(t[1]), p(t[2]));
    let n = (b - a).cross(c - a);
    (n.length() * 0.5, (a + b + c) / 3.0, n)
}

/// Whether `p` lies inside closed `mesh` (its winding number about `p`
/// past one half).
pub fn contains(mesh: &Mesh, p: DVec3) -> bool {
    let points: Vec<DVec3> = mesh
        .positions
        .iter()
        .map(|q| DVec3::from(q.map(f64::from)))
        .collect();
    let tris: Vec<[u32; 3]> = mesh.indices.as_chunks::<3>().0.to_vec();
    crate::boolean::csg::winding(&points, &tris, p) > 0.5
}

/// The bounds of a mesh's positions.
pub fn bounds(m: &Mesh) -> (DVec3, DVec3) {
    m.positions
        .iter()
        .fold((DVec3::INFINITY, DVec3::NEG_INFINITY), |(lo, hi), p| {
            let p = DVec3::from(p.map(f64::from));
            (lo.min(p), hi.max(p))
        })
}

/// A half-space a cell keeps, `normal · x <= d`: the face it makes is
/// tagged `tag`, the cell on its other side ([`BOUNDS`] for none).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane {
    pub normal: DVec3,
    pub d: f64,
    pub tag: u32,
}

impl Plane {
    /// The half-space through `point` facing `normal` (made unit).
    pub fn through(point: DVec3, normal: DVec3, tag: u32) -> Plane {
        let normal = normal.normalize();
        Plane {
            normal,
            d: normal.dot(point),
            tag,
        }
    }
}

/// The Voronoi cells of `seeds` under the distance `|metric * (x - s)|`
/// (`DVec3::ONE`: the usual one; a smaller component stretches the cells
/// along that axis, as wood splits along its grain): cell i is the
/// half-spaces `|M(x - s_i)|² <= |M(x - s_j)|²`, each a plane, since
/// `2 x · M²(s_j - s_i) <= (s_j - s_i) · M²(s_j + s_i)`.
pub fn voronoi_cells(seeds: &[DVec3], metric: DVec3) -> Vec<Vec<Plane>> {
    let m2 = metric * metric;
    seeds
        .iter()
        .enumerate()
        .map(|(i, &s)| {
            seeds
                .iter()
                .enumerate()
                .filter(|&(j, &o)| j != i && o != s)
                .map(|(j, &o)| Plane::through((o + s) * 0.5, m2 * (o - s), j as u32))
                .collect()
        })
        .collect()
}

/// Cuts closed `mesh` (`material` per triangle) into the Voronoi cells of
/// `seeds` (see [`voronoi_cells`] for `metric`, and [`cut`]).
pub fn voronoi(
    mesh: &Mesh,
    material: &[u32],
    seeds: &[DVec3],
    metric: DVec3,
    uv_scale: f64,
) -> Result<Fracture, String> {
    if seeds.is_empty() {
        return Err("a fracture needs at least one seed".into());
    }
    cut(mesh, material, &voronoi_cells(seeds, metric), uv_scale)
}

/// Cuts closed `mesh` (`material` per triangle) into convex `cells`, each
/// the half-spaces it keeps: the cells are expected to tile space, a
/// cell's face tagged with the cell across it, so neighbours bond.
/// `uv_scale` maps the cut faces' box projection to uvs (1: a texture
/// repeat a metre). Pieces come in cell order ([`Piece::cell`]), a cell's
/// separate parts in order of their first triangle; pieces smaller than a
/// millionth of the mesh's volume are dropped. Refused for an open mesh
/// (no volume) or no cells.
pub fn cut(
    mesh: &Mesh,
    material: &[u32],
    cells: &[Vec<Plane>],
    uv_scale: f64,
) -> Result<Fracture, String> {
    if cells.is_empty() {
        return Err("a fracture needs at least one cell".into());
    }
    if material.len() != mesh.triangle_count() {
        return Err("one material a triangle".into());
    }
    let (whole, _) = volume_centroid(mesh);
    if whole <= 0.0 {
        return Err(
            "the mesh encloses no volume: a fracture needs a closed, outward-facing mesh".into(),
        );
    }
    let (lo, hi) = bounds(mesh);
    let size = (hi - lo).max_element().max(1e-6);
    let eps = size * 1e-9;
    // The cells' box: past the mesh, so no cell face lies on its surface.
    let (blo, bhi) = (lo - size * 0.125, hi + size * 0.125);
    let mut pieces: Vec<Piece> = Vec::new();
    for (i, planes) in cells.iter().enumerate() {
        let mut cell = Cell::boxed(blo, bhi);
        for p in planes {
            cell.clip(p.normal, p.d, p.tag, eps);
            if cell.is_empty() {
                break;
            }
        }
        if cell.is_empty() {
            continue;
        }
        let (cm, tags) = cell.mesh(eps, uv_scale);
        let (cut, origins) = csg_traced(
            mesh,
            Placement::default(),
            &cm,
            Placement::default(),
            CsgOp::Intersect,
        )
        .map_err(|e| format!("cell {i}: {e}"))?;
        for part in parts(&cut, size * 1e-6) {
            let mut piece = Piece {
                cell: i as u32,
                ..Piece::default()
            };
            let mut remap: BTreeMap<u32, u32> = BTreeMap::new();
            for &t in &part {
                let tri = &cut.indices[t * 3..t * 3 + 3];
                for &v in tri {
                    let n = *remap.entry(v).or_insert_with(|| {
                        let k = v as usize;
                        piece.mesh.positions.push(cut.positions[k]);
                        piece.mesh.normals.push(cut.normals[k]);
                        piece.mesh.uvs.push(cut.uvs[k]);
                        (piece.mesh.positions.len() - 1) as u32
                    });
                    piece.mesh.indices.push(n);
                }
                match origins[t] {
                    (0, src) => {
                        piece.material.push(Some(material[src as usize]));
                        piece.faces.push(None);
                    }
                    (_, src) => {
                        piece.material.push(None);
                        piece.faces.push(Some(tags[src as usize]));
                    }
                }
            }
            let (v, c) = volume_centroid(&piece.mesh);
            if v <= whole * 1e-6 {
                continue;
            }
            piece.volume = v;
            piece.centroid = c;
            pieces.push(piece);
        }
    }
    let bonds = bonds(&pieces);
    Ok(Fracture { pieces, bonds })
}

/// A cut cell's solid parts: its triangles grouped by shared positions
/// ([`split`]), each open group (triangles the CSG left unwelded to the
/// rest, a few bits off: no solid of their own, and a hull with no volume
/// would make a body physics cannot weigh) folded into the closed part
/// with the vertex nearest its own.
fn parts(m: &Mesh, tol: f64) -> Vec<Vec<usize>> {
    let at = |v: u32| DVec3::from(m.positions[v as usize].map(f64::from));
    let key = |p: DVec3| (p / tol).round().as_i64vec3().to_array();
    let points = |g: &[usize]| -> Vec<DVec3> {
        g.iter()
            .flat_map(|&t| (0..3).map(move |k| t * 3 + k))
            .map(|i| at(m.indices[i]))
            .collect()
    };
    let closed = |g: &[usize]| {
        let mut edges: BTreeMap<([i64; 3], [i64; 3]), i32> = BTreeMap::new();
        for &t in g {
            let tri = &m.indices[t * 3..t * 3 + 3];
            for k in 0..3 {
                let (a, b) = (key(at(tri[k])), key(at(tri[(k + 1) % 3])));
                if a != b {
                    *edges.entry((a.min(b), a.max(b))).or_default() += 1;
                }
            }
        }
        !edges.is_empty() && edges.values().all(|&n| n >= 2)
    };
    let (mut solid, open): (Vec<Vec<usize>>, Vec<Vec<usize>>) = split(m)
        .into_iter()
        .partition(|g| closed(g) && spans(&points(g), tol));
    if solid.is_empty() {
        // Nothing closed: keep the cell whole if its triangles span a
        // solid, and drop it if they lie in a plane (a sliver of CSG
        // noise, with no volume to weigh).
        let all = open.concat();
        return if spans(&points(&all), tol) {
            vec![all]
        } else {
            Vec::new()
        };
    }
    for g in open {
        let p = at(m.indices[g[0] * 3]);
        let near = |s: &Vec<usize>| {
            points(s)
                .into_iter()
                .map(|q| q.distance_squared(p))
                .fold(f64::INFINITY, f64::min)
        };
        let best = (0..solid.len())
            .min_by(|&a, &b| near(&solid[a]).total_cmp(&near(&solid[b])))
            .expect("one solid part");
        solid[best].extend(g);
    }
    solid
}

/// Whether points `pts` span a solid: four of them lie off one plane by
/// more than `tol`.
fn spans(pts: &[DVec3], tol: f64) -> bool {
    let Some(&a) = pts.first() else { return false };
    let Some(&b) = pts
        .iter()
        .max_by(|p, q| p.distance_squared(a).total_cmp(&q.distance_squared(a)))
    else {
        return false;
    };
    let ab = b - a;
    let off = |p: &DVec3| (*p - a).cross(ab).length_squared();
    let Some(&c) = pts.iter().max_by(|p, q| off(p).total_cmp(&off(q))) else {
        return false;
    };
    let n = ab.cross(c - a);
    if n.length() <= tol * ab.length() {
        return false;
    }
    let n = n.normalize();
    pts.iter().any(|p| n.dot(*p - a).abs() > tol)
}

/// A mesh's triangles grouped into parts joined by shared positions.
fn split(m: &Mesh) -> Vec<Vec<usize>> {
    let n = m.triangle_count();
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(p: &mut [usize], mut i: usize) -> usize {
        while p[i] != i {
            p[i] = p[p[i]];
            i = p[i];
        }
        i
    }
    let mut owner: BTreeMap<[u32; 3], usize> = BTreeMap::new();
    for t in 0..n {
        for k in 0..3 {
            let p = m.positions[m.indices[t * 3 + k] as usize];
            let key = p.map(|x| (x + 0.0).to_bits());
            match owner.get(&key) {
                Some(&o) => {
                    let (a, b) = (find(&mut parent, t), find(&mut parent, o));
                    if a != b {
                        parent[a.max(b)] = a.min(b);
                    }
                }
                None => {
                    owner.insert(key, t);
                }
            }
        }
    }
    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for t in 0..n {
        let r = find(&mut parent, t);
        groups.entry(r).or_default().push(t);
    }
    groups.into_values().collect()
}

/// Each piece's cut faces towards each neighbouring cell: (area,
/// Per-triangle cut face summary: `(area, centroid, weighted normal, aabb min, aabb max)`.
type CutFace = (DVec3, [DVec3; 3]);

fn cut_faces_by_neighbour(p: &Piece) -> BTreeMap<u32, Vec<CutFace>> {
    let mut out: BTreeMap<u32, Vec<CutFace>> = BTreeMap::new();
    for (t, tri) in p.mesh.indices.as_chunks::<3>().0.iter().enumerate() {
        let Some(Some(j)) = p.faces.get(t).copied() else {
            continue;
        };
        if j == BOUNDS {
            continue;
        }
        let (a, _, n) = tri_area(&p.mesh, tri);
        if a <= 0.0 {
            continue;
        }
        let p0 = DVec3::from_array(p.mesh.positions[tri[0] as usize].map(f64::from));
        let p1 = DVec3::from_array(p.mesh.positions[tri[1] as usize].map(f64::from));
        let p2 = DVec3::from_array(p.mesh.positions[tri[2] as usize].map(f64::from));
        out.entry(j)
            .or_default()
            .push((n.normalize_or_zero(), [p0, p1, p2]));
    }
    out
}

/// Sutherland–Hodgman clipping in the common contact plane. Opposing
/// meshes may triangulate the same non-convex section differently.
fn overlap(a: [DVec3; 3], b: [DVec3; 3], normal: DVec3) -> Vec<[DVec3; 3]> {
    let mut polygon = a.to_vec();
    let winding = (b[1] - b[0]).cross(b[2] - b[0]).dot(normal).signum();
    for i in 0..3 {
        let (p, q) = (b[i], b[(i + 1) % 3]);
        let side = |x: DVec3| (q - p).cross(x - p).dot(normal) * winding;
        let mut next = Vec::new();
        for (&x, &y) in polygon.iter().zip(polygon.iter().cycle().skip(1)) {
            let (sx, sy) = (side(x), side(y));
            if sx >= 0.0 {
                next.push(x);
            }
            if (sx >= 0.0) != (sy >= 0.0) {
                next.push(x + (y - x) * (sx / (sx - sy)));
            }
        }
        polygon = next;
        if polygon.len() < 3 {
            return Vec::new();
        }
    }
    (1..polygon.len() - 1)
        .map(|i| [polygon[0], polygon[i], polygon[i + 1]])
        .collect()
}

/// The bonds between pieces of neighbouring cells: every pair of pieces
/// `(a, b)` in adjacent cells whose opposing cut faces actually touch within
/// geometric tolerance gets a bond carrying the contacting face subset's
/// area, centroid, and normal. Non-convex pieces touching multiple parts of
/// one neighbour cell produce one bond per contacting pair, and faces
/// separated by empty air never form phantom bonds.
fn bonds(pieces: &[Piece]) -> Vec<Bond> {
    const CONTACT_EPS: f64 = 2.0e-3;
    let faces: Vec<_> = pieces.iter().map(cut_faces_by_neighbour).collect();
    let mut by_cell: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
    for (k, p) in pieces.iter().enumerate() {
        by_cell.entry(p.cell).or_default().push(k);
    }
    let mut out = Vec::new();
    for (a, pa) in pieces.iter().enumerate() {
        for (&j, tris_a) in &faces[a] {
            if j <= pa.cell || tris_a.is_empty() {
                continue;
            }
            let Some(candidates) = by_cell.get(&j) else {
                continue;
            };
            for &b in candidates {
                let Some(tris_b) = faces[b].get(&pa.cell) else {
                    continue;
                };
                if tris_b.is_empty() {
                    continue;
                }
                let (mut area, mut csum, mut nsum) = (0.0, DVec3::ZERO, DVec3::ZERO);
                let mut section = Vec::new();
                for &(tn, ta) in tris_a {
                    for &(bn, tb) in tris_b {
                        if tn.dot(bn) > -0.99
                            || tb.iter().any(|q| (*q - ta[0]).dot(tn).abs() > CONTACT_EPS)
                        {
                            continue;
                        }
                        for tri in overlap(ta, tb, tn) {
                            let a = (tri[1] - tri[0]).cross(tri[2] - tri[0]).length() * 0.5;
                            if a <= 1e-12 {
                                continue;
                            }
                            area += a;
                            csum += (tri[0] + tri[1] + tri[2]) * (a / 3.0);
                            nsum += tn * a;
                            section.push(tri);
                        }
                    }
                }
                if area <= 0.0 {
                    continue;
                }
                let centroid = csum / area;
                let normal = nsum.normalize_or_zero();
                let (lo, hi, normal) = if a < b {
                    (a, b, normal)
                } else {
                    (b, a, -normal)
                };
                out.push(Bond {
                    a: lo as u32,
                    b: hi as u32,
                    centroid,
                    normal,
                    area,
                    section,
                });
            }
        }
    }
    out.sort_by_key(|x| (x.a, x.b));
    out
}

/// The pieces of `f` joined into `groups` (each a list of piece indices;
/// together a partition of the pieces): a group's piece is its members'
/// triangles less the cut faces between them (a face towards a cell in
/// the same group), its volume and centroid theirs; the bonds between
/// groups sum the bonds of their members. A group's [`Piece::cell`] is
/// its first member's. Join whole cells: the faces between the separate
/// parts of one cell are not told apart.
pub fn merge(f: &Fracture, groups: &[Vec<u32>]) -> Fracture {
    let mut group_of = vec![u32::MAX; f.pieces.len()];
    for (g, members) in groups.iter().enumerate() {
        for &m in members {
            group_of[m as usize] = g as u32;
        }
    }
    let mut pieces = Vec::with_capacity(groups.len());
    for members in groups {
        let cells: std::collections::BTreeSet<u32> =
            members.iter().map(|&m| f.pieces[m as usize].cell).collect();
        let mut out = Piece {
            cell: members.first().map_or(0, |&m| f.pieces[m as usize].cell),
            ..Piece::default()
        };
        let mut sum = DVec3::ZERO;
        for &m in members {
            let p = &f.pieces[m as usize];
            out.volume += p.volume;
            sum += p.centroid * p.volume;
            let mut remap: BTreeMap<u32, u32> = BTreeMap::new();
            for (t, tri) in p.mesh.indices.as_chunks::<3>().0.iter().enumerate() {
                if let Some(Some(j)) = p.faces.get(t)
                    && cells.contains(j)
                {
                    continue;
                }
                for &v in tri {
                    let n = *remap.entry(v).or_insert_with(|| {
                        let k = v as usize;
                        out.mesh.positions.push(p.mesh.positions[k]);
                        out.mesh.normals.push(p.mesh.normals[k]);
                        out.mesh.uvs.push(p.mesh.uvs[k]);
                        (out.mesh.positions.len() - 1) as u32
                    });
                    out.mesh.indices.push(n);
                }
                out.material.push(p.material[t]);
                out.faces.push(p.faces[t]);
            }
        }
        out.centroid = if out.volume > 0.0 {
            sum / out.volume
        } else {
            DVec3::ZERO
        };
        pieces.push(out);
    }
    // Bonds between groups; centroid and normal accumulate area-weighted
    // sums until all contact sections of the pair have been collected.
    let mut by = BTreeMap::<_, Bond>::new();
    for b in &f.bonds {
        let (ga, gb) = (group_of[b.a as usize], group_of[b.b as usize]);
        if ga == gb || ga == u32::MAX || gb == u32::MAX {
            continue;
        }
        let (key, n) = if ga < gb {
            ((ga, gb), b.normal)
        } else {
            ((gb, ga), -b.normal)
        };
        let e = by.entry(key).or_insert_with(|| Bond {
            a: key.0,
            b: key.1,
            centroid: DVec3::ZERO,
            normal: DVec3::ZERO,
            area: 0.0,
            section: Vec::new(),
        });
        e.area += b.area;
        e.centroid += b.centroid * b.area;
        e.normal += n * b.area;
        e.section.extend_from_slice(&b.section);
    }
    let bonds = by
        .into_values()
        .filter(|b| b.area > 0.0)
        .map(|mut b| {
            b.centroid /= b.area;
            b.normal = b.normal.normalize_or_zero();
            b
        })
        .collect();
    Fracture { pieces, bonds }
}
