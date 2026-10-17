//! Generated geometry and textures: the primitives a scene needs before any
//! asset is loaded (three.js `SphereGeometry`, `PlaneGeometry`).

use crate::scene::{ColorSpace, MeshData, Mips, TextureData, Vertex};
use glam::Vec3;

/// A UV sphere of radius 0.5 centred at the origin: `segments` around the
/// equator, `rings` from pole to pole (`sphere(16, 12)` is 384 triangles).
pub fn sphere(segments: u32, rings: u32) -> MeshData {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for r in 0..=rings {
        let v = r as f32 / rings as f32;
        let theta = v * std::f32::consts::PI;
        for s in 0..=segments {
            let u = s as f32 / segments as f32;
            let phi = u * std::f32::consts::TAU;
            let n = Vec3::new(
                theta.sin() * phi.cos(),
                theta.cos(),
                theta.sin() * phi.sin(),
            );
            vertices.push(Vertex {
                position: (n * 0.5).to_array(),
                normal: n.to_array(),
                uv: [u, v],
                color: [1.0; 4],
            });
        }
    }
    for r in 0..rings {
        for s in 0..segments {
            let a = r * (segments + 1) + s;
            let b = a + segments + 1;
            indices.extend_from_slice(&[a, a + 1, b, a + 1, b + 1, b]);
        }
    }
    MeshData { vertices, indices }
}

/// A horizontal square of side `size` centred at the origin, facing +y, with
/// UVs running 0..`uv_repeat` across it.
pub fn plane(size: f32, uv_repeat: f32) -> MeshData {
    let h = size / 2.0;
    let v = |x: f32, z: f32, u: f32, w: f32| Vertex {
        position: [x, 0.0, z],
        normal: [0.0, 1.0, 0.0],
        uv: [u * uv_repeat, w * uv_repeat],
        color: [1.0; 4],
    };
    MeshData {
        vertices: vec![
            v(-h, -h, 0.0, 0.0),
            v(h, -h, 1.0, 0.0),
            v(h, h, 1.0, 1.0),
            v(-h, h, 0.0, 1.0),
        ],
        indices: vec![0, 2, 1, 0, 3, 2],
    }
}

/// An axis-aligned box of `size` centred at the origin, with flat normals and
/// UVs 0..1 on every face (three.js `BoxGeometry`).
pub fn cuboid(size: Vec3) -> MeshData {
    let h = size * 0.5;
    let mut vertices = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);
    // (normal, u axis, v axis) per face; corners = n*h ± u*h ± v*h.
    let faces = [
        (Vec3::X, Vec3::NEG_Z, Vec3::Y),
        (Vec3::NEG_X, Vec3::Z, Vec3::Y),
        (Vec3::Y, Vec3::X, Vec3::NEG_Z),
        (Vec3::NEG_Y, Vec3::X, Vec3::Z),
        (Vec3::Z, Vec3::X, Vec3::Y),
        (Vec3::NEG_Z, Vec3::NEG_X, Vec3::Y),
    ];
    for (n, u, v) in faces {
        let base = vertices.len() as u32;
        for (cu, cv) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            let p = (n + u * cu + v * cv) * h;
            vertices.push(Vertex {
                position: p.to_array(),
                normal: n.to_array(),
                uv: [(cu + 1.0) * 0.5, (1.0 - cv) * 0.5],
                color: [1.0; 4],
            });
        }
        // Counter-clockwise seen from outside (u x v = n).
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    MeshData { vertices, indices }
}

/// A grey checkerboard of `size`x`size` texels with `cells` squares per side
/// (sRGB, mips generated).
pub fn checker(size: u32, cells: u32) -> TextureData {
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let on = ((x * cells / size) + (y * cells / size)).is_multiple_of(2);
            let v = if on { 170 } else { 120 };
            rgba.extend_from_slice(&[v, v, v, 255]);
        }
    }
    TextureData {
        width: size,
        height: size,
        rgba,
        color_space: ColorSpace::Srgb,
        mips: Mips::Generate,
    }
}
