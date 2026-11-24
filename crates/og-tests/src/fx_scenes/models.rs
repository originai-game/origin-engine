//! Stylised materials, skinned models and per-instance looks.

use super::*;

// ---------------------------------------------------------------------------
// The stylised look: toon, outlines, rim light, dissolve, flash.
// ---------------------------------------------------------------------------

pub(crate) fn torus(major: f32, minor: f32, segments: u32, sides: u32) -> MeshData {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for i in 0..=segments {
        let u = i as f32 / segments as f32 * std::f32::consts::TAU;
        for j in 0..=sides {
            let v = j as f32 / sides as f32 * std::f32::consts::TAU;
            let n = Vec3::new(u.cos() * v.cos(), v.sin(), u.sin() * v.cos());
            let c = Vec3::new(u.cos(), 0.0, u.sin()) * major;
            vertices.push(Vertex {
                tangent: [0.0; 4],
                position: (c + n * minor).to_array(),
                normal: n.to_array(),
                uv: [i as f32 / segments as f32, j as f32 / sides as f32],
                color: [1.0; 4],
            });
        }
    }
    for i in 0..segments {
        for j in 0..sides {
            let a = i * (sides + 1) + j;
            let b = a + sides + 1;
            indices.extend_from_slice(&[a, a + 1, b, a + 1, b + 1, b]);
        }
    }
    MeshData { vertices, indices }
}

pub(crate) fn ramp_texture(scene: &mut RenderScene) -> TextureHandle {
    // Four hard bands, cool shadow to warm light.
    let rgba: Vec<u8> = (0..8)
        .flat_map(|i| {
            let v = [40u8, 40, 110, 110, 190, 190, 255, 255][i];
            [v, v, v, 255]
        })
        .collect();
    scene.add_texture(TextureData {
        width: 8,
        height: 1,
        rgba,
        color_space: ColorSpace::Linear,
        mips: Mips::None,
    })
}

// ---------------------------------------------------------------------------
// GPU skinning.
// ---------------------------------------------------------------------------

/// Bones of the skinned cylinder.
pub const CYLINDER_BONES: usize = 4;
/// Its height (bones split it evenly along +y).
pub const CYLINDER_HEIGHT: f32 = 2.4;

/// A capped cylinder along +y with `rings` rings, skinned to
/// [`CYLINDER_BONES`] bones: each vertex blends the two bones nearest its
/// height.
pub fn skinned_cylinder(radius: f32, rings: u32, sides: u32) -> (MeshData, SkinData) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut joints = Vec::new();
    let mut weights = Vec::new();
    let seg = CYLINDER_HEIGHT / CYLINDER_BONES as f32;
    let skin_at = |y: f32| {
        let f = (y / seg - 0.5).clamp(0.0, (CYLINDER_BONES - 1) as f32);
        let a = (f.floor() as u16).min(CYLINDER_BONES as u16 - 1);
        let b = (a + 1).min(CYLINDER_BONES as u16 - 1);
        let w = f - a as f32;
        ([a, b, 0, 0], [1.0 - w, w, 0.0, 0.0])
    };
    for r in 0..=rings {
        let y = r as f32 / rings as f32 * CYLINDER_HEIGHT;
        for s in 0..=sides {
            let a = s as f32 / sides as f32 * std::f32::consts::TAU;
            let n = Vec3::new(a.cos(), 0.0, a.sin());
            vertices.push(Vertex {
                tangent: [0.0; 4],
                position: [n.x * radius, y, n.z * radius],
                normal: n.to_array(),
                uv: [s as f32 / sides as f32, 1.0 - r as f32 / rings as f32],
                color: [1.0; 4],
            });
            let (j, w) = skin_at(y);
            joints.push(j);
            weights.push(w);
        }
    }
    for r in 0..rings {
        for s in 0..sides {
            let a = r * (sides + 1) + s;
            let b = a + sides + 1;
            indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    // Caps.
    for (y, up) in [(0.0f32, -1.0f32), (CYLINDER_HEIGHT, 1.0)] {
        let center = vertices.len() as u32;
        vertices.push(Vertex {
            tangent: [0.0; 4],
            position: [0.0, y, 0.0],
            normal: [0.0, up, 0.0],
            uv: [0.5, 0.5],
            color: [1.0; 4],
        });
        let (j, w) = skin_at(y);
        joints.push(j);
        weights.push(w);
        for s in 0..=sides {
            let a = s as f32 / sides as f32 * std::f32::consts::TAU;
            vertices.push(Vertex {
                tangent: [0.0; 4],
                position: [a.cos() * radius, y, a.sin() * radius],
                normal: [0.0, up, 0.0],
                uv: [0.5 + a.cos() * 0.5, 0.5 + a.sin() * 0.5],
                color: [1.0; 4],
            });
            joints.push(j);
            weights.push(w);
        }
        for s in 0..sides {
            let (p, q) = (center + 1 + s, center + 2 + s);
            if up > 0.0 {
                indices.extend_from_slice(&[center, q, p]);
            } else {
                indices.extend_from_slice(&[center, p, q]);
            }
        }
    }
    (
        vertices_mesh(vertices, indices),
        SkinData { joints, weights },
    )
}

fn vertices_mesh(vertices: Vec<Vertex>, indices: Vec<u32>) -> MeshData {
    MeshData { vertices, indices }
}

/// The palette that bends the cylinder: each bone turns by `bend` radians
/// about z (and `twist` about y) relative to its parent, at its base.
pub fn bend_palette(bend: f32, twist: f32) -> JointPalette {
    let seg = CYLINDER_HEIGHT / CYLINDER_BONES as f32;
    let mut matrices = Vec::with_capacity(CYLINDER_BONES);
    let mut global = Mat4::IDENTITY;
    for b in 0..CYLINDER_BONES {
        // Bone b's rest frame sits at height b * seg.
        let local = if b == 0 {
            Mat4::from_rotation_y(twist)
        } else {
            Mat4::from_translation(Vec3::Y * seg)
                * Mat4::from_rotation_z(bend)
                * Mat4::from_rotation_y(twist)
        };
        global *= local;
        let inverse_bind = Mat4::from_translation(-Vec3::Y * seg * b as f32);
        matrices.push(global * inverse_bind);
    }
    JointPalette { matrices }
}
