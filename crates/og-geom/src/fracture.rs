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
//! and the mesh is intersected with it by [`crate::csg`], whose exact
//! predicates keep every piece closed and the same on every host; the cut
//! faces are the cell's faces kept inside the mesh. A non-convex mesh can
//! leave a cell in several separate pieces, each its own chunk
//! (three-pinata's `findIsolatedGeometry`).
//!
//! Every number is f64 with `+ - * /` and `sqrt`: the same on every host.

use crate::csg::{CsgOp, csg_traced};
use crate::{Mesh, Placement};
use glam::DVec3;
use std::collections::BTreeMap;

/// A cut face's tag in [`Piece::faces`]: the bounds of the cells (a cell
/// face no neighbour shares).
pub const BOUNDS: u32 = u32::MAX;

/// One chunk: a closed mesh in the input's space.
#[derive(Clone, Debug, Default)]
pub struct Piece {
    /// The seed whose cell it lies in.
    pub cell: u32,
    pub mesh: Mesh,
    /// Per triangle: the input material it came from, or `None` for a cut
    /// (interior) face.
    pub material: Vec<Option<u32>>,
    /// Per triangle: the neighbouring seed a cut face faces ([`BOUNDS`] for
    /// none), or `None` for an outer face.
    pub faces: Vec<Option<u32>>,
    pub volume: f64,
    pub centroid: DVec3,
}

/// The surface two pieces share.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bond {
    /// Indices into the pieces, `a < b`.
    pub a: u32,
    pub b: u32,
    pub centroid: DVec3,
    /// From a towards b.
    pub normal: DVec3,
    pub area: f64,
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
    crate::csg::winding(&points, &tris, p) > 0.5
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

/// Cuts closed `mesh` (`material` per triangle) into the Voronoi cells of
/// `seeds`. `uv_scale` maps the cut faces' box projection to uvs (1: a
/// texture repeat a metre). Pieces come in seed order, a cell's separate
/// parts in order of their first triangle; pieces smaller than a
/// millionth of the mesh's volume are dropped. Refused for an open mesh
/// (no volume) or no seeds.
pub fn voronoi(
    mesh: &Mesh,
    material: &[u32],
    seeds: &[DVec3],
    uv_scale: f64,
) -> Result<Fracture, String> {
    if seeds.is_empty() {
        return Err("a fracture needs at least one seed".into());
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
    for (i, &s) in seeds.iter().enumerate() {
        let mut cell = Cell::boxed(blo, bhi);
        for (j, &o) in seeds.iter().enumerate() {
            if j == i || o == s {
                continue;
            }
            let n = (o - s).normalize();
            let d = n.dot((o + s) * 0.5);
            cell.clip(n, d, j as u32, eps);
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
        for part in split(&cut) {
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
/// area-weighted centroid sum, area-weighted normal sum).
fn faces_by_neighbour(p: &Piece) -> BTreeMap<u32, (f64, DVec3, DVec3)> {
    let mut out: BTreeMap<u32, (f64, DVec3, DVec3)> = BTreeMap::new();
    for (t, tri) in p.mesh.indices.as_chunks::<3>().0.iter().enumerate() {
        let Some(Some(j)) = p.faces.get(t).copied() else {
            continue;
        };
        if j == BOUNDS {
            continue;
        }
        let (a, c, n) = tri_area(&p.mesh, tri);
        let e = out.entry(j).or_default();
        e.0 += a;
        e.1 += c * a;
        e.2 += n;
    }
    out
}

/// The bonds between pieces of neighbouring cells: a piece's cut faces
/// towards cell j bond it to the piece of cell j whose faces towards its
/// cell lie nearest.
fn bonds(pieces: &[Piece]) -> Vec<Bond> {
    let faces: Vec<_> = pieces.iter().map(faces_by_neighbour).collect();
    let mut by_cell: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
    for (k, p) in pieces.iter().enumerate() {
        by_cell.entry(p.cell).or_default().push(k);
    }
    let mut out = Vec::new();
    for (a, pa) in pieces.iter().enumerate() {
        for (&j, &(area, csum, nsum)) in &faces[a] {
            if j <= pa.cell || area <= 0.0 {
                continue;
            }
            let centroid = csum / area;
            let Some(candidates) = by_cell.get(&j) else {
                continue;
            };
            let best = candidates
                .iter()
                .filter_map(|&b| {
                    let &(ab, cb, _) = faces[b].get(&pa.cell)?;
                    (ab > 0.0).then(|| (b, (cb / ab - centroid).length_squared()))
                })
                .min_by(|x, y| x.1.total_cmp(&y.1).then(x.0.cmp(&y.0)));
            let Some((b, _)) = best else { continue };
            // The faces of a towards j face out of a: towards b.
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
            });
        }
    }
    out.sort_by_key(|x| (x.a, x.b));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube(h: f32) -> Mesh {
        // 24 vertices, flat faces, outward.
        let mut m = Mesh::default();
        let faces: [([f32; 3], [f32; 3], [f32; 3]); 6] = [
            ([1., 0., 0.], [0., 1., 0.], [0., 0., 1.]),
            ([-1., 0., 0.], [0., 0., 1.], [0., 1., 0.]),
            ([0., 1., 0.], [0., 0., 1.], [1., 0., 0.]),
            ([0., -1., 0.], [1., 0., 0.], [0., 0., 1.]),
            ([0., 0., 1.], [1., 0., 0.], [0., 1., 0.]),
            ([0., 0., -1.], [0., 1., 0.], [1., 0., 0.]),
        ];
        for (n, u, v) in faces {
            let b = m.positions.len() as u32;
            for (su, sv) in [(-1., -1.), (1., -1.), (1., 1.), (-1., 1.)] {
                let p: [f32; 3] = std::array::from_fn(|k| (n[k] + u[k] * su + v[k] * sv) * h);
                m.positions.push(p);
                m.normals.push(n);
                m.uvs.push([0.0, 0.0]);
            }
            m.indices.extend([b, b + 1, b + 2, b, b + 2, b + 3]);
        }
        m
    }

    #[test]
    fn cells_tile_the_mesh_and_bond_their_neighbours() {
        let m = cube(0.5);
        let seeds = [
            DVec3::new(-0.3, -0.2, 0.1),
            DVec3::new(0.25, -0.25, -0.2),
            DVec3::new(0.1, 0.3, 0.2),
            DVec3::new(-0.2, 0.25, -0.3),
        ];
        let f = voronoi(&m, &[0; 12], &seeds, 1.0).unwrap();
        assert_eq!(f.pieces.len(), 4);
        let total: f64 = f.pieces.iter().map(|p| p.volume).sum();
        assert!(
            (total - 1.0).abs() < 1e-5,
            "volumes sum to the cube: {total}"
        );
        // Every piece has outer faces of material 0 and cut faces.
        for p in &f.pieces {
            assert!(p.material.contains(&Some(0)));
            assert!(p.material.contains(&None));
        }
        // Each bond's normal points from a's centroid towards b's.
        assert!(f.bonds.len() >= 3);
        for b in &f.bonds {
            let (pa, pb) = (&f.pieces[b.a as usize], &f.pieces[b.b as usize]);
            assert!(b.normal.dot(pb.centroid - pa.centroid) > 0.0);
            assert!(b.area > 0.0);
        }
    }

    #[test]
    fn a_seed_outside_the_mesh_leaves_no_piece() {
        let m = cube(0.5);
        let seeds = [
            DVec3::new(-0.2, 0.0, 0.0),
            DVec3::new(0.2, 0.0, 0.0),
            DVec3::new(5.0, 0.0, 0.0),
        ];
        let f = voronoi(&m, &[0; 12], &seeds, 1.0).unwrap();
        assert_eq!(f.pieces.len(), 2);
        assert_eq!(f.bonds.len(), 1);
        let b = f.bonds[0];
        assert!(
            (b.area - 1.0).abs() < 1e-6,
            "the x = 0 plane of a unit cube: {}",
            b.area
        );
        assert!((b.normal - DVec3::X).length() < 1e-9);
    }
}
