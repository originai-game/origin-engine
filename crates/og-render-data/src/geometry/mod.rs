//! Geometry made at run time (`docs/design.md` §2.6): three.js
//! `BufferGeometry` from attribute arrays ([`buffer_geometry`]), the
//! parametric primitives beside box, plane and sphere ([`primitives`]), and
//! a BVH per geometry for rays against what is drawn ([`bvh`]).

pub mod bvh;
pub mod primitives;

pub use bvh::{Bvh, BvhNode, TriangleHit};

use crate::scene::{MeshData, Vertex};
use glam::Vec3;

/// A geometry's attributes as three.js `BufferGeometry` holds them: flat
/// arrays, one item per vertex (`positions` xyz, `normals` xyz, `uvs` uv
/// with v down as glTF, `colors` rgb or rgba, `tangents` xyzw), and
/// `indices` three per triangle (none: every three vertices are one).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BufferAttributes<'a> {
    pub positions: &'a [f32],
    pub normals: Option<&'a [f32]>,
    pub uvs: Option<&'a [f32]>,
    pub colors: Option<&'a [f32]>,
    pub tangents: Option<&'a [f32]>,
    pub indices: Option<&'a [u32]>,
}

/// Builds a mesh from attributes: each array checked against the vertex
/// count, the indices against it, normals computed where none are given
/// (three.js `computeVertexNormals`: each vertex the area-weighted sum of
/// its triangles' normals; an unindexed geometry is flat-shaded by it).
/// Tangents are left to the renderer, which generates missing ones.
pub fn buffer_geometry(a: &BufferAttributes) -> Result<MeshData, String> {
    if a.positions.is_empty() || !a.positions.len().is_multiple_of(3) {
        return Err(format!(
            "positions holds {} numbers: a geometry needs xyz triples, at least one",
            a.positions.len()
        ));
    }
    let n = a.positions.len() / 3;
    if a.positions.iter().any(|x| !x.is_finite()) {
        return Err("positions holds a number that is not finite".into());
    }
    let check = |name: &str, arr: Option<&[f32]>, sizes: &[usize]| -> Result<usize, String> {
        let Some(arr) = arr else { return Ok(0) };
        let size = sizes
            .iter()
            .copied()
            .find(|s| arr.len() == s * n)
            .ok_or_else(|| {
                let want: Vec<String> = sizes.iter().map(|s| (s * n).to_string()).collect();
                format!(
                    "{name} holds {} numbers; {n} vertices take {}",
                    arr.len(),
                    want.join(" or ")
                )
            })?;
        if arr.iter().any(|x| !x.is_finite()) {
            return Err(format!("{name} holds a number that is not finite"));
        }
        Ok(size)
    };
    check("normals", a.normals, &[3])?;
    check("uvs", a.uvs, &[2])?;
    let color_size = check("colors", a.colors, &[3, 4])?;
    check("tangents", a.tangents, &[4])?;
    let indices: Vec<u32> = match a.indices {
        Some(ix) => {
            if !ix.len().is_multiple_of(3) {
                return Err(format!(
                    "indices holds {} numbers: triangles are triples",
                    ix.len()
                ));
            }
            if let Some(bad) = ix.iter().find(|&&i| i as usize >= n) {
                return Err(format!("indices holds {bad}, past the {n} vertices"));
            }
            ix.to_vec()
        }
        None => {
            if !n.is_multiple_of(3) {
                return Err(format!(
                    "{n} vertices without indices: an unindexed geometry is triangles, three vertices each"
                ));
            }
            (0..n as u32).collect()
        }
    };
    let mut vertices: Vec<Vertex> = (0..n)
        .map(|i| {
            let f = |arr: Option<&[f32]>, size: usize, k: usize, d: f32| {
                arr.map_or(d, |a| a[i * size + k])
            };
            Vertex {
                position: [
                    a.positions[i * 3],
                    a.positions[i * 3 + 1],
                    a.positions[i * 3 + 2],
                ],
                normal: [0.0, 1.0, 0.0],
                uv: [f(a.uvs, 2, 0, 0.0), f(a.uvs, 2, 1, 0.0)],
                color: [
                    f(a.colors, color_size.max(1), 0, 1.0),
                    f(a.colors, color_size.max(1), 1, 1.0),
                    f(a.colors, color_size.max(1), 2, 1.0),
                    if color_size == 4 {
                        f(a.colors, 4, 3, 1.0)
                    } else {
                        1.0
                    },
                ],
                tangent: [
                    f(a.tangents, 4, 0, 0.0),
                    f(a.tangents, 4, 1, 0.0),
                    f(a.tangents, 4, 2, 0.0),
                    f(a.tangents, 4, 3, 0.0),
                ],
            }
        })
        .collect();
    match a.normals {
        Some(normals) => {
            for (i, v) in vertices.iter_mut().enumerate() {
                let nrm = Vec3::new(normals[i * 3], normals[i * 3 + 1], normals[i * 3 + 2]);
                v.normal = nrm.normalize_or(Vec3::Y).to_array();
            }
        }
        None => compute_normals(&mut vertices, &indices),
    }
    Ok(MeshData { vertices, indices })
}

/// three.js `computeVertexNormals`: every vertex the normalised sum of its
/// triangles' (area-weighted) normals; a vertex in no triangle keeps +Y.
pub fn compute_normals(vertices: &mut [Vertex], indices: &[u32]) {
    let mut sums = vec![Vec3::ZERO; vertices.len()];
    for t in indices.as_chunks::<3>().0 {
        let [a, b, c] = [t[0], t[1], t[2]].map(|i| i as usize);
        let (pa, pb, pc) = (
            Vec3::from(vertices[a].position),
            Vec3::from(vertices[b].position),
            Vec3::from(vertices[c].position),
        );
        let face = (pb - pa).cross(pc - pa);
        for i in [a, b, c] {
            sums[i] += face;
        }
    }
    for (v, s) in vertices.iter_mut().zip(sums) {
        v.normal = s.normalize_or(Vec3::Y).to_array();
    }
}

/// What og.render.geometryInfo reports: counts and bounds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeometryInfo {
    pub vertices: u32,
    pub triangles: u32,
    pub min: Vec3,
    pub max: Vec3,
}

pub fn info(mesh: &MeshData) -> GeometryInfo {
    let (min, max) = mesh.bounds().unwrap_or((Vec3::ZERO, Vec3::ZERO));
    GeometryInfo {
        vertices: mesh.vertices.len() as u32,
        triangles: (mesh.indices.len() / 3) as u32,
        min,
        max,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_normals_are_computed_and_shared_vertices_average() {
        // Two triangles folded along x: their shared edge's normals are the mean.
        let positions = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 1.0, 1.0];
        let m = buffer_geometry(&BufferAttributes {
            positions: &positions,
            indices: Some(&[0, 1, 2, 1, 0, 3]),
            ..Default::default()
        })
        .unwrap();
        // Triangle 0's cross product is (0, 1, 0), triangle 1's (0, 1, -1):
        // a vertex of both takes their area-weighted sum.
        let n2 = Vec3::from(m.vertices[2].normal);
        assert!(n2.distance(Vec3::Y) < 1e-6, "{n2}");
        let n0 = Vec3::from(m.vertices[0].normal);
        let want = Vec3::new(0.0, 2.0, -1.0).normalize();
        assert!(n0.distance(want) < 1e-5, "{n0} vs {want}");
    }

    #[test]
    fn attributes_are_checked_against_the_vertex_count() {
        let p = [0.0; 9];
        let bad = |a: BufferAttributes| buffer_geometry(&a).unwrap_err();
        assert!(
            bad(BufferAttributes {
                positions: &p[..8],
                ..Default::default()
            })
            .contains("triples")
        );
        assert!(
            bad(BufferAttributes {
                positions: &p,
                uvs: Some(&[0.0; 5]),
                ..Default::default()
            })
            .contains("6")
        );
        assert!(
            bad(BufferAttributes {
                positions: &p,
                indices: Some(&[0, 1, 3]),
                ..Default::default()
            })
            .contains("past")
        );
        // rgb and rgba colours both fit.
        let rgb = buffer_geometry(&BufferAttributes {
            positions: &p,
            colors: Some(&[0.5; 9]),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(rgb.vertices[0].color, [0.5, 0.5, 0.5, 1.0]);
        let rgba = buffer_geometry(&BufferAttributes {
            positions: &p,
            colors: Some(&[0.25; 12]),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(rgba.vertices[2].color, [0.25; 4]);
        assert_eq!(rgba.indices, vec![0, 1, 2]);
    }
}
