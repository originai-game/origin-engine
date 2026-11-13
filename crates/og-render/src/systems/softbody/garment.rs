//! Draw-only linear-blend skinning and welded normal gathering. No solver
//! state is owned here; the tick supplies points and their pinned targets.

use crate::DeformedVertices;
use glam::{Mat4, Vec3};
use std::sync::Arc;

/// Immutable, welded drawing topology, in the model's authoring space.
#[derive(Clone, Debug, PartialEq)]
pub struct GarmentTopology {
    pub rest: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub skin: Vec<([u16; 4], [f32; 4])>,
    pub triangles: Vec<[u32; 3]>,
    /// Each drawn vertex's welded point and normal-facing sign.
    pub vertices: Vec<(u32, f32)>,
}

/// A frame's garment presentation, including procedural motion's palette.
#[derive(Clone, Debug, PartialEq)]
pub struct GarmentDeformation {
    pub key: (u64, usize),
    pub topology: Arc<GarmentTopology>,
    pub palette: Arc<Vec<Mat4>>,
    /// World-space simulated points and their tick's skin targets.
    pub points: Arc<Vec<Vec3>>,
    pub targets: Arc<Vec<Vec3>>,
    pub world: Mat4,
    /// Converts a saved frame to the current instance's space for CPU fallback.
    pub output_transform: Mat4,
    /// Simulated on the GPU (og.Garment's SIMULATION_VISUAL): `points` are
    /// then the skinned targets and the renderer moves the points from
    /// there, for looks only; reach and pins are per welded point.
    pub visual: Option<Arc<crate::VisualBody>>,
}

impl GarmentDeformation {
    /// Coarse CPU fallback, preserving the original drawing algorithm.
    pub fn evaluate(&self) -> DeformedVertices {
        let t = &self.topology;
        let inverse = self.world.inverse();
        let points: Vec<_> = t
            .rest
            .iter()
            .zip(&t.skin)
            .zip(self.points.iter().zip(self.targets.iter()))
            .map(|((r, (joints, weights)), (point, target))| {
                let total: f32 = weights.iter().sum();
                let mut m = Mat4::IDENTITY;
                if total >= 1e-6 && !self.palette.is_empty() {
                    m = Mat4::ZERO;
                    for (j, w) in joints.iter().zip(weights) {
                        m += self.palette[(*j as usize).min(self.palette.len() - 1)] * (w / total);
                    }
                }
                m.transform_point3(*r) + inverse.transform_vector3(*point - *target)
            })
            .collect();
        let mut normals = vec![Vec3::ZERO; points.len()];
        for &[a, b, c] in &t.triangles {
            let (a, b, c) = (a as usize, b as usize, c as usize);
            let face = (points[b] - points[a]).cross(points[c] - points[a]);
            let rest = (t.rest[b] - t.rest[a]).cross(t.rest[c] - t.rest[a]);
            for k in [a, b, c] {
                normals[k] += face
                    * if rest.dot(t.normals[k]) < 0.0 {
                        -1.0
                    } else {
                        1.0
                    };
            }
        }
        for (n, rest) in normals.iter_mut().zip(&t.normals) {
            *n = n.try_normalize().unwrap_or(*rest);
        }
        DeformedVertices {
            positions: t
                .vertices
                .iter()
                .map(|&(i, _)| self.output_transform.transform_point3(points[i as usize]))
                .collect(),
            normals: t
                .vertices
                .iter()
                .map(|&(i, s)| normals[i as usize] * s)
                .collect(),
            garment: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_blends_skin_applies_world_offsets_and_keeps_both_sides() {
        let g = GarmentDeformation {
            key: (1, 0),
            topology: Arc::new(GarmentTopology {
                rest: vec![Vec3::ZERO, Vec3::new(2., 0., 0.), Vec3::new(0., 3., 0.)],
                normals: vec![Vec3::Z; 3],
                skin: vec![([0, 1, 0, 0], [1., 3., 0., 0.]); 3],
                triangles: vec![[0, 1, 2], [2, 1, 0]],
                vertices: vec![(0, 1.), (1, 1.), (2, 1.), (0, -1.)],
            }),
            palette: Arc::new(vec![
                Mat4::IDENTITY,
                Mat4::from_translation(Vec3::new(4., -2., 6.))
                    * Mat4::from_scale(Vec3::new(2., 1., 0.5)),
            ]),
            points: Arc::new(vec![
                Vec3::new(9., 8., 13.),
                Vec3::new(9., 8., 13.),
                Vec3::new(9., 8., 17.),
            ]),
            targets: Arc::new(vec![Vec3::new(7., 8., 9.); 3]),
            world: Mat4::from_translation(Vec3::splat(20.))
                * Mat4::from_scale(Vec3::new(2., 3., 4.)),
            output_transform: Mat4::IDENTITY,
            visual: None,
        };
        let d = g.evaluate();
        assert_eq!(
            d.positions,
            vec![
                Vec3::new(4., -1.5, 5.5),
                Vec3::new(7.5, -1.5, 5.5),
                Vec3::new(4., 1.5, 6.5),
                Vec3::new(4., -1.5, 5.5)
            ]
        );
        let normal = Vec3::new(0., -1., 3.).normalize();
        for n in &d.normals[..3] {
            assert!((*n - normal).length() < 1e-6);
        }
        assert!((d.normals[3] + normal).length() < 1e-6);
        let mut previous = g;
        previous.output_transform = Mat4::from_translation(Vec3::new(-2., 1., 3.));
        assert_eq!(previous.evaluate().positions[1], Vec3::new(5.5, -0.5, 8.5));
    }
}
