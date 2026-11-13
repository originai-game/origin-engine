//! The compact vertex encoding ([`crate::VertexEncoding::Compact`]): 16
//! bytes a vertex for the lean pipelines, against [`crate::Vertex`]'s 64.
//!
//! - position: `Unorm16x4`, x, y and z over the mesh's bounds (w 0). The
//!   bounds' corner and size are not a shader input: they are folded into
//!   the instance transform of every batch that draws the mesh compact
//!   ([`Dequant::matrix`], `instances.rs`), so the vertex stage reads a
//!   point in the unit box and the instance matrix takes it to the world.
//! - normal: `Snorm16x2`, octahedral, of the normal through the
//!   dequantisation's scale (`n * scale`, normalised): the lit stage's
//!   normal matrix (the cofactor of instance x dequantisation) then turns
//!   it back into the mesh normal's direction.
//! - colour: `Unorm8x4`.
//!
//! No uv and no tangent: a plain material samples no map (its uv is never
//! read) and has no normal map. The f32 vertices are what every other
//! pipeline reads; a compact mesh uploads them only when one of those draws
//! it (`Renderer::ensure_full`).

use crate::MeshData;
use glam::{Mat4, Vec2, Vec3};

/// One vertex (`CompactVertexIn` in pbr_lean.wgsl).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct CompactVertex {
    pub position: [u16; 4],
    pub normal: [i16; 2],
    pub color: [u8; 4],
}

/// The mesh-space point of a quantised position `q` (each axis 0..1):
/// `offset + q * scale`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Dequant {
    pub offset: Vec3,
    /// Per axis, the bounds' size (1 where it is flat).
    pub scale: Vec3,
}

impl Dequant {
    /// The unit box to mesh space, as the instance transform's right factor.
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_translation(self.offset) * Mat4::from_scale(self.scale)
    }

    /// Local bounds (centre, half extent) in the unit box, and a radius
    /// that, scaled by the largest axis of (instance x [`Dequant::matrix`]),
    /// is never less than the mesh-space sphere's (GPU culling's sphere
    /// test reads the instance matrix the vertex stage does).
    pub fn unit_bounds(&self, center: Vec3, half: Vec3) -> (Vec3, Vec3, f32) {
        let c = (center - self.offset) / self.scale;
        let e = half / self.scale;
        (c, e, half.length() / self.scale.min_element())
    }
}

/// Where the compact position, `Snorm16x2` normal and `Unorm8x4` colour sit.
pub(crate) const LEAN_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<CompactVertex>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &wgpu::vertex_attr_array![0 => Unorm16x4, 1 => Snorm16x2, 3 => Unorm8x4],
};

/// Position and normal (the lean prepass).
pub(crate) const POSITION_NORMAL_LAYOUT: wgpu::VertexBufferLayout<'static> =
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<CompactVertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Unorm16x4, 1 => Snorm16x2],
    };

/// Position alone (shadow casters, the occluders' depth).
pub(crate) const POSITION_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<CompactVertex>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &wgpu::vertex_attr_array![0 => Unorm16x4],
};

fn unorm16(x: f32) -> u16 {
    (x.clamp(0.0, 1.0) * 65535.0).round() as u16
}

fn snorm16(x: f32) -> i16 {
    (x.clamp(-1.0, 1.0) * 32767.0).round() as i16
}

fn unorm8(x: f32) -> u8 {
    (x.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// A unit vector's octahedral coordinates (each -1..1).
pub(crate) fn oct_encode(n: Vec3) -> Vec2 {
    let n = n / (n.x.abs() + n.y.abs() + n.z.abs()).max(1e-20);
    if n.z >= 0.0 {
        Vec2::new(n.x, n.y)
    } else {
        let sign = |v: f32| if v >= 0.0 { 1.0 } else { -1.0 };
        Vec2::new((1.0 - n.y.abs()) * sign(n.x), (1.0 - n.x.abs()) * sign(n.y))
    }
}

/// [`oct_encode`]'s inverse, as pbr_lean.wgsl's `oct_decode` computes it.
#[cfg(test)]
fn oct_decode(e: Vec2) -> Vec3 {
    let mut v = Vec3::new(e.x, e.y, 1.0 - e.x.abs() - e.y.abs());
    let t = (-v.z).max(0.0);
    v.x += if v.x >= 0.0 { -t } else { t };
    v.y += if v.y >= 0.0 { -t } else { t };
    v.normalize()
}

/// `mesh` quantised, and how to undo the positions' quantisation. `None`
/// for a mesh without vertices.
pub(crate) fn encode(mesh: &MeshData) -> Option<(Vec<CompactVertex>, Dequant)> {
    let (lo, hi) = mesh.bounds()?;
    let size = hi - lo;
    let scale = Vec3::select(size.cmpgt(Vec3::ZERO), size, Vec3::ONE);
    let dequant = Dequant { offset: lo, scale };
    let vertices = mesh
        .vertices
        .iter()
        .map(|v| {
            let q = (Vec3::from(v.position) - lo) / scale;
            let n = (Vec3::from(v.normal) * scale).normalize_or(Vec3::Y);
            let o = oct_encode(n);
            CompactVertex {
                position: [unorm16(q.x), unorm16(q.y), unorm16(q.z), 0],
                normal: [snorm16(o.x), snorm16(o.y)],
                color: v.color.map(unorm8),
            }
        })
        .collect();
    Some((vertices, dequant))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes;

    /// Positions come back within half a step of the bounds' 65535th, the
    /// normals' direction (through the instance's cofactor, as the lit
    /// stage turns them) within a few thousandths of a radian, colours
    /// within half of 1/255.
    #[test]
    fn compact_vertices_decode_close_to_the_mesh() {
        let mut mesh = shapes::cuboid(Vec3::new(4.0, 0.5, 1.5));
        mesh.vertices
            .extend(shapes::sphere(16, 12).vertices.iter().map(|v| {
                let mut v = *v;
                v.color = [0.25, 0.5, 0.75, 1.0];
                v
            }));
        let (compact, dq) = encode(&mesh).expect("vertices");
        assert_eq!(std::mem::size_of::<CompactVertex>(), 16);
        let d = dq.matrix();
        let step = dq.scale / 65535.0;
        // The cofactor of the dequantisation's 3x3 (diagonal): det * inverse.
        let cofactor = Vec3::new(
            dq.scale.y * dq.scale.z,
            dq.scale.x * dq.scale.z,
            dq.scale.x * dq.scale.y,
        );
        for (v, c) in mesh.vertices.iter().zip(&compact) {
            let q = Vec3::new(
                c.position[0] as f32,
                c.position[1] as f32,
                c.position[2] as f32,
            ) / 65535.0;
            let p = d.transform_point3(q);
            let err = (p - Vec3::from(v.position)).abs();
            assert!(
                err.cmple(step * 0.5 + 1e-6).all(),
                "{p} against {:?}",
                v.position
            );
            let e = Vec2::new(c.normal[0] as f32, c.normal[1] as f32) / 32767.0;
            let n = (oct_decode(e) * cofactor).normalize();
            let angle = n
                .dot(Vec3::from(v.normal).normalize())
                .clamp(-1.0, 1.0)
                .acos();
            assert!(angle < 2e-3, "normal {n} against {:?}", v.normal);
            for (k, &byte) in c.color.iter().enumerate() {
                assert!((byte as f32 / 255.0 - v.color[k]).abs() <= 0.5 / 255.0 + 1e-6);
            }
        }
        // Flat along an axis: that axis keeps scale 1 and its offset exactly.
        let plane = shapes::plane(3.0, 1.0);
        let (_, dq) = encode(&plane).expect("vertices");
        assert_eq!(dq.scale.y, 1.0);
        assert!(encode(&MeshData::default()).is_none());
    }

    /// The unit box's sphere radius, through any instance's largest axis,
    /// covers the mesh-space sphere through the instance.
    #[test]
    fn unit_bounds_never_shrink_the_culling_sphere() {
        let dq = Dequant {
            offset: Vec3::new(-1.0, -0.05, -0.05),
            scale: Vec3::new(2.0, 0.1, 0.1),
        };
        let (center, half) = (Vec3::ZERO, Vec3::new(1.0, 0.05, 0.05));
        let (c, e, r) = dq.unit_bounds(center, half);
        assert!((c - Vec3::splat(0.5)).abs().max_element() < 1e-6);
        assert!((e - Vec3::splat(0.5)).abs().max_element() < 1e-6);
        for m in [
            Mat4::IDENTITY,
            Mat4::from_scale(Vec3::new(0.1, 10.0, 10.0)),
            Mat4::from_scale(Vec3::new(10.0, 0.1, 0.1)),
        ] {
            let md = m * dq.matrix();
            let axis = |m: &Mat4| {
                m.x_axis
                    .truncate()
                    .length()
                    .max(m.y_axis.truncate().length())
                    .max(m.z_axis.truncate().length())
            };
            assert!(r * axis(&md) >= half.length() * axis(&m) * 0.9999);
        }
    }
}
