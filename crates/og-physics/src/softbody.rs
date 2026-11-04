//! PBD soft volumes (og.SoftBody): a closed surface of points (2D: a ring)
//! integrated by position, held by distance constraints along its edges and
//! one constraint on the volume it encloses (2D: its area), pinned to its
//! entity by `pinTop` and pushed out of solid colliders. One-way, as cloth.
//!
//! Determinism: `+ - * /` and `sqrt` on f32 in a fixed order (edges in
//! index order, the volume's gradient by triangle order); the state is the
//! points with the surface they were built as, saved in the og.physics
//! block's trailer.

use crate::MeshDesc;
use crate::schema_ext::{SOFT_BOX, SOFT_MESH, SoftBody};
use crate::verlet::Solids;
use og_core::glam::{Quat, Vec3};
use og_core::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// What a soft volume was built from: a change rebuilds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SoftKey {
    shape: u32,
    detail: u32,
    mesh: u64,
    radius: u32,
    size: [u32; 3],
    pin_top: u32,
}

impl SoftKey {
    pub fn of(s: &SoftBody) -> SoftKey {
        SoftKey {
            shape: s.shape,
            detail: s.detail,
            mesh: if s.shape == SOFT_MESH { s.mesh } else { 0 },
            radius: s.radius.to_bits(),
            size: s.size.to_array().map(f32::to_bits),
            pin_top: s.pin_top.to_bits(),
        }
    }
}

/// A soft volume's points as the last step left them, and the surface they
/// were built as.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct SoftState {
    pub key: SoftKey,
    pub points: Vec<Vec3>,
    pub previous: Vec<Vec3>,
    /// Each point at rest in the entity's space (pins follow it there).
    rest: Vec<Vec3>,
    /// The points `pinTop` holds, ascending.
    pinned: Vec<u32>,
    /// The surface's triangles, wound outwards (2D: a fan over the ring,
    /// for drawing; the ring is the points in order, anticlockwise).
    pub triangles: Vec<[u32; 3]>,
    /// The edges held (a, b, rest length), a < b, ascending.
    edges: Vec<(u32, u32, f32)>,
    /// The volume (2D: area) at rest.
    pub rest_volume: f32,
    flat: bool,
}

/// The midpoint of edge (a, b) of an icosphere being divided, made once.
fn midpoint(points: &mut Vec<Vec3>, cache: &mut BTreeMap<(u32, u32), u32>, a: u32, b: u32) -> u32 {
    let key = (a.min(b), a.max(b));
    if let Some(&i) = cache.get(&key) {
        return i;
    }
    let p = ((points[a as usize] + points[b as usize]) * 0.5).normalize();
    points.push(p);
    let i = points.len() as u32 - 1;
    cache.insert(key, i);
    i
}

/// A unit icosphere divided `detail` times (12, 42, 162, 642 points).
fn icosphere(detail: u32) -> (Vec<Vec3>, Vec<[u32; 3]>) {
    let t = (1.0 + 5.0f32.sqrt()) * 0.5;
    let mut points: Vec<Vec3> = [
        (-1.0, t, 0.0),
        (1.0, t, 0.0),
        (-1.0, -t, 0.0),
        (1.0, -t, 0.0),
        (0.0, -1.0, t),
        (0.0, 1.0, t),
        (0.0, -1.0, -t),
        (0.0, 1.0, -t),
        (t, 0.0, -1.0),
        (t, 0.0, 1.0),
        (-t, 0.0, -1.0),
        (-t, 0.0, 1.0),
    ]
    .iter()
    .map(|&(x, y, z)| Vec3::new(x, y, z).normalize())
    .collect();
    let mut tris: Vec<[u32; 3]> = vec![
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ];
    for _ in 0..detail.min(3) {
        let mut cache = BTreeMap::new();
        let mut next = Vec::with_capacity(tris.len() * 4);
        for [a, b, c] in tris {
            let ab = midpoint(&mut points, &mut cache, a, b);
            let bc = midpoint(&mut points, &mut cache, b, c);
            let ca = midpoint(&mut points, &mut cache, c, a);
            next.extend_from_slice(&[[a, ab, ca], [b, bc, ab], [c, ca, bc], [ab, bc, ca]]);
        }
        tris = next;
    }
    (points, tris)
}

/// The surface of a box of `size`, each face a grid of `n` × `n` cells,
/// its points shared along the edges.
fn box_surface(size: Vec3, n: u32) -> (Vec<Vec3>, Vec<[u32; 3]>) {
    let mut index: BTreeMap<[u32; 3], u32> = BTreeMap::new();
    let mut points = Vec::new();
    let mut tris = Vec::new();
    let mut at = |c: [u32; 3], points: &mut Vec<Vec3>| -> u32 {
        *index.entry(c).or_insert_with(|| {
            let f = |k: usize| (c[k] as f32 / n as f32 - 0.5) * size[k];
            points.push(Vec3::new(f(0), f(1), f(2)));
            points.len() as u32 - 1
        })
    };
    // Each face: its axis, the side it is on, and two in-plane axes whose
    // cross product points along the axis (flipped on the low side).
    for axis in 0..3usize {
        let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
        for side in [0, n] {
            for j in 0..n {
                for i in 0..n {
                    let corner = |di: u32, dj: u32| {
                        let mut c = [0u32; 3];
                        c[axis] = side;
                        c[u] = i + di;
                        c[v] = j + dj;
                        c
                    };
                    let a = at(corner(0, 0), &mut points);
                    let b = at(corner(1, 0), &mut points);
                    let c = at(corner(1, 1), &mut points);
                    let d = at(corner(0, 1), &mut points);
                    if side == n {
                        tris.extend_from_slice(&[[a, b, c], [a, c, d]]);
                    } else {
                        tris.extend_from_slice(&[[a, c, b], [a, d, c]]);
                    }
                }
            }
        }
    }
    (points, tris)
}

/// A mesh's vertices with those at the same position merged (a glTF
/// splits them along seams), and its triangles over them.
fn welded(mesh: &MeshDesc) -> (Vec<Vec3>, Vec<[u32; 3]>) {
    let mut index: BTreeMap<[u32; 3], u32> = BTreeMap::new();
    let mut points = Vec::new();
    let remap: Vec<u32> = mesh
        .positions
        .iter()
        .map(|p| {
            *index
                .entry(p.to_array().map(f32::to_bits))
                .or_insert_with(|| {
                    points.push(*p);
                    points.len() as u32 - 1
                })
        })
        .collect();
    let tris = mesh
        .indices
        .iter()
        .filter_map(|t| {
            let m = t.map(|i| remap.get(i as usize).copied());
            let [Some(a), Some(b), Some(c)] = m else {
                return None;
            };
            (a != b && b != c && c != a).then_some([a, b, c])
        })
        .collect();
    (points, tris)
}

/// A closed surface's volume: the signed tetrahedra from the origin.
fn volume3(points: &[Vec3], tris: &[[u32; 3]]) -> f32 {
    let mut v = 0.0;
    for &[a, b, c] in tris {
        let (pa, pb, pc) = (points[a as usize], points[b as usize], points[c as usize]);
        v += pa.dot(pb.cross(pc));
    }
    v / 6.0
}

/// A ring's area in xy (positive anticlockwise).
fn area2(points: &[Vec3]) -> f32 {
    let n = points.len();
    let mut a = 0.0;
    for i in 0..n {
        let (p, q) = (points[i], points[(i + 1) % n]);
        a += p.x * q.y - q.x * p.y;
    }
    a * 0.5
}

impl SoftState {
    /// Built at the entity's pose (`origin`, `rotation`); `mesh` is the
    /// registered mesh SOFT_MESH names. 2D builds a ring in the xy plane.
    pub fn new(
        soft: &SoftBody,
        origin: Vec3,
        rotation: Quat,
        flat: bool,
        mesh: Option<&MeshDesc>,
    ) -> Result<SoftState> {
        let detail = soft.detail.min(7);
        let (mut rest, mut triangles): (Vec<Vec3>, Vec<[u32; 3]>);
        if flat {
            rest = if soft.shape == SOFT_BOX {
                let n = detail + 1;
                let (w, h) = (soft.size.x.max(0.01), soft.size.y.max(0.01));
                let mut ring = Vec::new();
                let corners = [
                    Vec3::new(-w, -h, 0.0) * 0.5,
                    Vec3::new(w, -h, 0.0) * 0.5,
                    Vec3::new(w, h, 0.0) * 0.5,
                    Vec3::new(-w, h, 0.0) * 0.5,
                ];
                for k in 0..4 {
                    let (a, b) = (corners[k], corners[(k + 1) % 4]);
                    for i in 0..n {
                        ring.push(a + (b - a) * (i as f32 / n as f32));
                    }
                }
                ring
            } else if soft.shape == SOFT_MESH {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    "og.SoftBody.shape SOFT_MESH (2) is 3D only: the 2D profile takes SOFT_SPHERE or SOFT_BOX",
                ));
            } else {
                let n = 12u32 << detail.min(3);
                let r = soft.radius.max(0.01);
                (0..n)
                    .map(|i| {
                        let a = std::f32::consts::TAU * i as f32 / n as f32;
                        let (s, c) = og_core::math::sin_cos(a);
                        Vec3::new(c * r, s * r, 0.0)
                    })
                    .collect()
            };
            triangles = (1..rest.len() as u32 - 1).map(|i| [0, i, i + 1]).collect();
        } else {
            (rest, triangles) = match soft.shape {
                SOFT_BOX => box_surface(soft.size.max(Vec3::splat(0.01)), detail + 1),
                SOFT_MESH => {
                    let mesh = mesh.ok_or_else(|| {
                        Error::new(
                            ErrorKind::NotFound,
                            format!(
                                "og.SoftBody.mesh {} is not a registered mesh (og.physics.setMesh)",
                                soft.mesh
                            ),
                        )
                    })?;
                    welded(mesh)
                }
                _ => {
                    let (p, t) = icosphere(detail);
                    let r = soft.radius.max(0.01);
                    (p.into_iter().map(|p| p * r).collect(), t)
                }
            };
            if rest.len() < 4 || triangles.len() < 4 {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    format!("og.SoftBody.mesh {} has no closed surface", soft.mesh),
                ));
            }
        }
        // Wound outwards: a positive volume (2D: an anticlockwise ring).
        let rest_volume = if flat {
            if area2(&rest) < 0.0 {
                rest.reverse();
            }
            area2(&rest)
        } else {
            if volume3(&rest, &triangles) < 0.0 {
                for t in &mut triangles {
                    t.swap(1, 2);
                }
            }
            volume3(&rest, &triangles)
        };
        let mut edges: Vec<(u32, u32)> = if flat {
            let n = rest.len() as u32;
            (0..n)
                .map(|i| (i.min((i + 1) % n), i.max((i + 1) % n)))
                .collect()
        } else {
            triangles
                .iter()
                .flat_map(|&[a, b, c]| [(a, b), (b, c), (c, a)])
                .map(|(a, b)| (a.min(b), a.max(b)))
                .collect()
        };
        edges.sort_unstable();
        edges.dedup();
        let edges = edges
            .into_iter()
            .map(|(a, b)| (a, b, (rest[b as usize] - rest[a as usize]).length()))
            .collect();
        let pinned = if soft.pin_top > 0.0 {
            let top = rest.iter().fold(f32::MIN, |m, p| m.max(p.y));
            (0..rest.len() as u32)
                .filter(|&i| rest[i as usize].y >= top - soft.pin_top)
                .collect()
        } else {
            Vec::new()
        };
        let rotation = if flat { Quat::IDENTITY } else { rotation };
        let points: Vec<Vec3> = rest.iter().map(|p| origin + rotation * *p).collect();
        Ok(SoftState {
            key: SoftKey::of(soft),
            previous: points.clone(),
            points,
            rest,
            pinned,
            triangles,
            edges,
            rest_volume,
            flat,
        })
    }

    /// Every point at rest on the entity, at rest (SIMULATION_VISUAL).
    pub fn hold(&mut self, origin: Vec3, rotation: Quat) {
        let rotation = if self.flat { Quat::IDENTITY } else { rotation };
        for (p, r) in self.points.iter_mut().zip(&self.rest) {
            *p = origin + rotation * *r;
        }
        self.previous.clone_from(&self.points);
    }

    /// The points `pinTop` holds, ascending.
    pub fn pinned(&self) -> &[u32] {
        &self.pinned
    }

    /// Whether it is a ring in the xy plane (2D).
    pub fn flat(&self) -> bool {
        self.flat
    }

    /// Its volume (2D: area) now.
    pub fn volume(&self) -> f32 {
        if self.flat {
            area2(&self.points)
        } else {
            volume3(&self.points, &self.triangles)
        }
    }

    /// The mean of its points.
    pub fn center(&self) -> Vec3 {
        let sum = self.points.iter().fold(Vec3::ZERO, |s, p| s + *p);
        sum / self.points.len().max(1) as f32
    }

    /// One tick of `substeps` steps: integrate with gravity, then
    /// `iterations` passes of pins, edges, the volume and collisions; then
    /// friction against what it touches.
    pub fn step(
        &mut self,
        soft: &SoftBody,
        dt: f32,
        gravity: Vec3,
        origin: Vec3,
        rotation: Quat,
        solids: &dyn Solids,
    ) {
        let n = self.points.len();
        let flat = self.flat;
        let z = origin.z;
        let rotation = if flat { Quat::IDENTITY } else { rotation };
        let planar = |p: Vec3| if flat { Vec3::new(p.x, p.y, z) } else { p };
        let substeps = soft.substeps.clamp(1, 16);
        let ds = dt / substeps as f32;
        let damping = soft.damping.clamp(0.0, 1.0);
        let keep = if substeps == 1 {
            damping
        } else {
            og_core::math::pow(damping, 1.0 / substeps as f32)
        };
        let g = gravity * soft.gravity_scale;
        let stiffness = soft.stiffness.clamp(0.0, 1.0);
        let target = self.rest_volume * soft.pressure.clamp(0.0, 4.0);
        let volume_stiffness = if soft.pressure > 0.0 {
            soft.volume_stiffness.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let thickness = soft.thickness.max(0.0);
        let mut w = vec![1.0f32; n];
        for &i in &self.pinned {
            w[i as usize] = 0.0;
        }
        let mut grads = vec![Vec3::ZERO; n];
        for _ in 0..substeps {
            for ((p, prev), &wi) in self.points.iter_mut().zip(&mut self.previous).zip(&w) {
                let at = *p;
                if wi == 0.0 {
                    *prev = at;
                    continue;
                }
                let v = (at - *prev) * keep;
                *prev = at;
                *p = planar(at + v + g * (ds * ds));
            }
            for _ in 0..soft.iterations.clamp(1, 64) {
                for &i in &self.pinned {
                    self.points[i as usize] = origin + rotation * self.rest[i as usize];
                }
                if stiffness > 0.0 {
                    for &(a, b, rest) in &self.edges {
                        let (a, b) = (a as usize, b as usize);
                        let (wa, wb) = (w[a], w[b]);
                        let sum = wa + wb;
                        if sum <= 0.0 {
                            continue;
                        }
                        let d = self.points[b] - self.points[a];
                        let len = d.length();
                        if len <= 1e-9 {
                            continue;
                        }
                        let k = stiffness * (len - rest) / (len * sum);
                        self.points[a] += d * (k * wa);
                        self.points[b] -= d * (k * wb);
                    }
                }
                if volume_stiffness > 0.0 {
                    self.restore_volume(target, volume_stiffness, &w, &mut grads);
                }
                if thickness > 0.0 {
                    for (p, &wi) in self.points.iter_mut().zip(&w) {
                        if wi == 0.0 {
                            continue;
                        }
                        if let Some((out, _)) = solids.push_out(*p, thickness) {
                            *p = planar(out);
                        }
                    }
                }
            }
            let friction = soft.friction.clamp(0.0, 1.0);
            if friction > 0.0 && thickness > 0.0 {
                for i in 0..n {
                    let p = self.points[i];
                    if let Some((_, nrm)) = solids.push_out(p, thickness * 1.05) {
                        let v = p - self.previous[i];
                        let slide = v - nrm * nrm.dot(v);
                        self.previous[i] += slide * friction;
                    }
                }
            }
        }
    }

    /// One pass of the volume constraint (Müller et al. 2007): every point
    /// moved along the volume's gradient by the share that restores it.
    fn restore_volume(&mut self, target: f32, k: f32, w: &[f32], grads: &mut [Vec3]) {
        let n = self.points.len();
        let c = self.volume() - target;
        if self.flat {
            for (i, g) in grads.iter_mut().enumerate() {
                let (prev, next) = (self.points[(i + n - 1) % n], self.points[(i + 1) % n]);
                *g = Vec3::new(next.y - prev.y, prev.x - next.x, 0.0) * 0.5;
            }
        } else {
            grads.fill(Vec3::ZERO);
            for &[a, b, c] in &self.triangles {
                let (pa, pb, pc) = (
                    self.points[a as usize],
                    self.points[b as usize],
                    self.points[c as usize],
                );
                grads[a as usize] += pb.cross(pc) / 6.0;
                grads[b as usize] += pc.cross(pa) / 6.0;
                grads[c as usize] += pa.cross(pb) / 6.0;
            }
        }
        let mut denom = 0.0;
        for (wi, g) in w.iter().zip(grads.iter()) {
            denom += wi * g.length_squared();
        }
        if denom <= 1e-12 {
            return;
        }
        let lambda = -k * c / denom;
        for i in 0..n {
            if w[i] > 0.0 {
                self.points[i] += grads[i] * (lambda * w[i]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Nothing;
    impl Solids for Nothing {
        fn push_out(&self, _: Vec3, _: f32) -> Option<(Vec3, Vec3)> {
            None
        }
    }

    #[test]
    fn surfaces_are_closed_and_wound_outwards() {
        for detail in 0..4 {
            let s = SoftBody {
                detail,
                ..SoftBody::default()
            };
            let st = SoftState::new(&s, Vec3::ZERO, Quat::IDENTITY, false, None).unwrap();
            assert_eq!(st.points.len(), [12, 42, 162, 642][detail as usize]);
            // Every edge of a closed surface has two triangles.
            assert_eq!(st.edges.len() * 2, st.triangles.len() * 3);
            let sphere = 4.0 / 3.0 * std::f32::consts::PI * 0.125;
            assert!(st.rest_volume > 0.0 && st.rest_volume < sphere);
        }
        let b = SoftBody {
            shape: SOFT_BOX,
            size: Vec3::new(2.0, 1.0, 0.5),
            detail: 3,
            ..SoftBody::default()
        };
        let st = SoftState::new(&b, Vec3::ZERO, Quat::IDENTITY, false, None).unwrap();
        assert_eq!(st.points.len(), 6 * 16 + 2); // (n+1)^3 - (n-1)^3, n = 4
        assert_eq!(st.edges.len() * 2, st.triangles.len() * 3);
        assert!((st.rest_volume - 1.0).abs() < 1e-5, "{}", st.rest_volume);
        let ring = SoftState::new(&b, Vec3::ZERO, Quat::IDENTITY, true, None).unwrap();
        assert!(
            (ring.rest_volume - 2.0).abs() < 1e-5,
            "{}",
            ring.rest_volume
        );
    }

    #[test]
    fn a_squeezed_volume_is_restored_and_pressure_inflates_it() {
        for flat in [false, true] {
            let s = SoftBody {
                detail: 1,
                gravity_scale: 0.0,
                ..SoftBody::default()
            };
            let mut st = SoftState::new(&s, Vec3::ZERO, Quat::IDENTITY, flat, None).unwrap();
            for p in &mut st.points {
                p.y *= 0.5;
            }
            st.previous = st.points.clone();
            assert!(st.volume() < st.rest_volume * 0.55);
            for _ in 0..30 {
                st.step(
                    &s,
                    1.0 / 60.0,
                    Vec3::ZERO,
                    Vec3::ZERO,
                    Quat::IDENTITY,
                    &Nothing,
                );
            }
            let ratio = st.volume() / st.rest_volume;
            assert!((ratio - 1.0).abs() < 0.02, "flat {flat}: {ratio}");
            let soft_edges = SoftBody {
                pressure: 1.5,
                stiffness: 0.05,
                ..s
            };
            for _ in 0..60 {
                st.step(
                    &soft_edges,
                    1.0 / 60.0,
                    Vec3::ZERO,
                    Vec3::ZERO,
                    Quat::IDENTITY,
                    &Nothing,
                );
            }
            let ratio = st.volume() / st.rest_volume;
            assert!(ratio > 1.2, "flat {flat}: inflated to {ratio}");
        }
    }
}
