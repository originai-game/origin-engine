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
        // The poles on the axis and the seam on its first angle, bit for bit
        // (sin π and sin 2π are not 0 in f32), so the sphere is closed.
        let (st, ct) = match r {
            0 => (0.0, 1.0),
            r if r == rings => (0.0, -1.0),
            _ => theta.sin_cos(),
        };
        for s in 0..=segments {
            let u = s as f32 / segments as f32;
            let phi =
                if s == segments { 0 } else { s } as f32 / segments as f32 * std::f32::consts::TAU;
            let n = Vec3::new(st * phi.cos(), ct, st * phi.sin());
            vertices.push(Vertex {
                tangent: [0.0; 4],
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
        tangent: [0.0; 4],
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

/// A rectangle of `width` × `height` in the xy plane facing +z, centred at
/// the origin, UVs 0..1 with v down (three.js `PlaneGeometry`, what
/// og.render.planeGeometry makes; turn it -π/2 about x for a floor).
pub fn plane_xy(width: f32, height: f32) -> MeshData {
    let (w, h) = (width / 2.0, height / 2.0);
    let v = |x: f32, y: f32, u: f32, t: f32| Vertex {
        tangent: [0.0; 4],
        position: [x, y, 0.0],
        normal: [0.0, 0.0, 1.0],
        uv: [u, t],
        color: [1.0; 4],
    };
    MeshData {
        vertices: vec![
            v(-w, -h, 0.0, 1.0),
            v(w, -h, 1.0, 1.0),
            v(w, h, 1.0, 0.0),
            v(-w, h, 0.0, 0.0),
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
    }
}

/// Scales a mesh's UVs by `repeat` along u and v, so a texture runs
/// `repeat` times across it (three.js `texture.repeat`; og.render's
/// geometry descs' `uvRepeat`). Material textures sample with repeat
/// wrapping, so the copies tile: a 1,600 m plane at 160 is a texture every
/// 10 m.
pub fn repeat_uv(mesh: &mut MeshData, repeat: glam::Vec2) {
    if repeat == glam::Vec2::ONE {
        return;
    }
    for v in &mut mesh.vertices {
        v.uv[0] *= repeat.x;
        v.uv[1] *= repeat.y;
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
                tangent: [0.0; 4],
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

/// A regular tetrahedron inscribed in a sphere of radius 0.5 centred at the
/// origin, with flat normals (three.js `TetrahedronGeometry(0.5)`): the
/// cheapest mesh particle, a shard or a chip.
pub fn tetrahedron() -> MeshData {
    let k = 0.5 / 3f32.sqrt();
    let c = [
        Vec3::new(k, k, k),
        Vec3::new(-k, -k, k),
        Vec3::new(-k, k, -k),
        Vec3::new(k, -k, -k),
    ];
    let mut vertices = Vec::with_capacity(12);
    let mut indices = Vec::with_capacity(12);
    for (a, b, d) in [(0, 1, 3), (0, 2, 1), (0, 3, 2), (1, 2, 3)] {
        let (a, b, d) = (c[a], c[b], c[d]);
        let mut n = (b - a).cross(d - a).normalize();
        // Counter-clockwise seen from outside: flip a face that faces in.
        let (b, d) = if n.dot(a + b + d) < 0.0 {
            n = -n;
            (d, b)
        } else {
            (b, d)
        };
        let base = vertices.len() as u32;
        for (p, uv) in [(a, [0.5, 0.0]), (b, [0.0, 1.0]), (d, [1.0, 1.0])] {
            vertices.push(Vertex {
                tangent: [0.0; 4],
                position: p.to_array(),
                normal: n.to_array(),
                uv,
                color: [1.0; 4],
            });
        }
        indices.extend_from_slice(&[base, base + 1, base + 2]);
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
