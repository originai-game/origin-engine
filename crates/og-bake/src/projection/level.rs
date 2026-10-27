//! The level being textured: its triangles in world space (y up), their
//! material uvs and part labels, and a BVH over them.

use crate::bvh::Bvh;
use glam::{Vec2, Vec3};

/// The static geometry of a level, one triangle list.
pub struct Level {
    pub positions: Vec<Vec3>,
    /// Per vertex; empty: the faces' own normals.
    pub normals: Vec<Vec3>,
    /// The material uvs the texture is painted into (glTF convention: (0, 0)
    /// at the image's top left).
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u32>,
    /// Per triangle: an index into `labels`.
    pub triangle_label: Vec<u16>,
    /// Part names (glTF material names, else mesh names), what prompts read.
    pub labels: Vec<String>,
    pub min: Vec3,
    pub max: Vec3,
    bvh: Bvh,
}

/// A surface a ray met, its normals turned to the side the ray came from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceHit {
    pub triangle: u32,
    /// Distance along the (unit) ray.
    pub t: f32,
    pub position: Vec3,
    pub normal: Vec3,
    pub face: Vec3,
    pub uv: Vec2,
    pub label: u16,
}

impl Level {
    /// `uvs` and `normals` are per vertex (normals may be empty);
    /// `triangle_label` per triangle (empty: every triangle label 0).
    pub fn new(
        positions: Vec<Vec3>,
        normals: Vec<Vec3>,
        uvs: Vec<Vec2>,
        indices: Vec<u32>,
        mut triangle_label: Vec<u16>,
        mut labels: Vec<String>,
    ) -> Level {
        let triangles = indices.len() / 3;
        if triangle_label.len() != triangles {
            triangle_label = vec![0; triangles];
        }
        if labels.is_empty() {
            labels.push("level".into());
        }
        let corners: Vec<[Vec3; 3]> = indices
            .as_chunks::<3>()
            .0
            .iter()
            .map(|t| t.map(|i| positions[i as usize]))
            .collect();
        let (mut min, mut max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        for p in &positions {
            min = min.min(*p);
            max = max.max(*p);
        }
        let normals = if normals.len() == positions.len() {
            normals
        } else {
            Vec::new()
        };
        Level {
            positions,
            normals,
            uvs,
            indices,
            triangle_label,
            labels,
            min,
            max,
            bvh: Bvh::new(corners),
        }
    }

    pub fn triangles(&self) -> usize {
        self.indices.len() / 3
    }

    pub fn corners(&self, t: usize) -> [u32; 3] {
        [
            self.indices[3 * t],
            self.indices[3 * t + 1],
            self.indices[3 * t + 2],
        ]
    }

    /// The triangle's geometric normal (by its winding).
    pub fn face_normal(&self, t: usize) -> Vec3 {
        let [a, b, c] = self.corners(t).map(|i| self.positions[i as usize]);
        (b - a).cross(c - a).normalize_or(Vec3::Y)
    }

    /// The interpolated shading normal at barycentrics `w` (of corners 0, 1, 2).
    pub fn normal_at(&self, t: usize, w: Vec3) -> Vec3 {
        let face = self.face_normal(t);
        if self.normals.is_empty() {
            return face;
        }
        let [a, b, c] = self.corners(t).map(|i| self.normals[i as usize]);
        let n = (a * w.x + b * w.y + c * w.z).normalize_or(face);
        if n.dot(face) < 0.0 { face } else { n }
    }

    pub fn position_at(&self, t: usize, w: Vec3) -> Vec3 {
        let [a, b, c] = self.corners(t).map(|i| self.positions[i as usize]);
        a * w.x + b * w.y + c * w.z
    }

    pub fn uv_at(&self, t: usize, w: Vec3) -> Vec2 {
        if self.uvs.len() != self.positions.len() {
            return Vec2::ZERO;
        }
        let [a, b, c] = self.corners(t).map(|i| self.uvs[i as usize]);
        a * w.x + b * w.y + c * w.z
    }

    /// The closest surface along a unit ray within `t_max`.
    pub fn hit(&self, origin: Vec3, dir: Vec3, t_max: f32) -> Option<SurfaceHit> {
        let h = self.bvh.closest(origin, dir, t_max)?;
        let t = h.triangle as usize;
        let w = Vec3::new(1.0 - h.u - h.v, h.u, h.v);
        let mut face = self.face_normal(t);
        let mut normal = self.normal_at(t, w);
        if face.dot(dir) > 0.0 {
            face = -face;
            normal = -normal;
        }
        Some(SurfaceHit {
            triangle: h.triangle,
            t: h.t,
            position: origin + dir * h.t,
            normal,
            face,
            uv: self.uv_at(t, w),
            label: self.triangle_label[t],
        })
    }

    pub fn occluded(&self, origin: Vec3, dir: Vec3, t_max: f32) -> bool {
        self.bvh.occluded(origin, dir, t_max)
    }

    /// How far rays start off a surface against self-hits.
    pub fn epsilon(&self) -> f32 {
        ((self.max - self.min).max_element() * 1e-5).max(1e-4)
    }
}
