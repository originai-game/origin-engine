//! Octahedral impostors (`model { impostor true }`, og-render's
//! `impostor.rs`): the model's default scene rendered on the CPU from
//! `grid × grid` directions over the whole sphere, each frame a square of
//! two atlases: the albedo with its coverage in alpha (sRGB), and the
//! normal in the frame's basis with the depth towards the viewer (linear,
//! `n · 0.5 + 0.5`, `0.5 + 0.5 × depth / radius`).
//!
//! Each frame is drawn at twice its size with a depth buffer and box
//! filtered down, so its silhouette is antialiased; texels nothing covers
//! then take their nearest covered neighbour's colour and normal (alpha
//! stays 0), so neither the atlas's filtering nor its mips darken the
//! edges. The albedo is the material's base colour factor times its base
//! colour texture (nearest texel, `TEXCOORD_0`) times the vertex colour:
//! lighting is the renderer's.

use super::GltfSource;
use crate::cook::texture::Image;
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use og_render::impostors::{frame_basis, frame_direction};

/// How a model's impostor is baked: frames per side and pixels per frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImpostorBake {
    pub grid: u32,
    pub frame: u32,
}

impl Default for ImpostorBake {
    fn default() -> Self {
        ImpostorBake {
            grid: 8,
            frame: 128,
        }
    }
}

/// A baked impostor: the two atlases and the sphere they cover.
pub struct Baked {
    pub albedo: Image,
    pub normal: Image,
    pub center: Vec3,
    pub radius: f32,
}

struct Tri {
    p: [Vec3; 3],
    n: [Vec3; 3],
    uv: [Vec2; 3],
    color: [Vec4; 3],
    /// The material's base colour factor and its texture (an image index).
    base: Vec4,
    texture: Option<usize>,
}

fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

/// The default scene's triangles in the model's space, as the rest pose
/// draws them (hidden nodes and their descendants left out, an instanced
/// node at each of its instances).
fn triangles(
    src: &GltfSource,
    instanced: &std::collections::BTreeMap<usize, Vec<Mat4>>,
) -> Vec<Tri> {
    let mut out = Vec::new();
    let Some(scene) = src.doc.default_scene().or_else(|| src.doc.scenes().next()) else {
        return out;
    };
    let mut stack: Vec<(gltf::Node, Mat4)> = scene.nodes().map(|n| (n, Mat4::IDENTITY)).collect();
    while let Some((node, parent)) = stack.pop() {
        if node
            .extension_value("KHR_node_visibility")
            .and_then(|v| v.get("visible")?.as_bool())
            == Some(false)
        {
            continue;
        }
        let (t, r, s) = node.transform().decomposed();
        let world = parent
            * Mat4::from_scale_rotation_translation(
                Vec3::from(s),
                Quat::from_array(r),
                Vec3::from(t),
            );
        let at: Vec<Mat4> = match instanced.get(&node.index()) {
            Some(i) => i.iter().map(|m| world * *m).collect(),
            None => vec![world],
        };
        if let Some(mesh) = node.mesh() {
            for prim in mesh
                .primitives()
                .filter(|p| p.mode() == gltf::mesh::Mode::Triangles)
            {
                let reader = prim.reader(|b| Some(&src.buffers[b.index()].0[..]));
                let Some(pos) = reader
                    .read_positions()
                    .map(|p| p.map(Vec3::from).collect::<Vec<_>>())
                else {
                    continue;
                };
                let normals: Option<Vec<Vec3>> =
                    reader.read_normals().map(|n| n.map(Vec3::from).collect());
                let uvs: Option<Vec<Vec2>> = reader
                    .read_tex_coords(0)
                    .map(|t| t.into_f32().map(Vec2::from).collect());
                let colors: Option<Vec<Vec4>> = reader
                    .read_colors(0)
                    .map(|c| c.into_rgba_f32().map(Vec4::from).collect());
                let index: Vec<u32> = match reader.read_indices() {
                    Some(i) => i.into_u32().collect(),
                    None => (0..pos.len() as u32).collect(),
                };
                let m = prim.material();
                let pbr = m.pbr_metallic_roughness();
                let base = Vec4::from(pbr.base_color_factor());
                let texture = pbr
                    .base_color_texture()
                    .filter(|t| t.tex_coord() == 0)
                    .map(|t| t.texture().source().index());
                for xf in &at {
                    let normal_m =
                        Mat4::from_mat3(glam::Mat3::from_mat4(*xf).inverse().transpose());
                    for k in index.as_chunks::<3>().0 {
                        let v = |j: usize| k[j] as usize;
                        if (0..3).any(|j| v(j) >= pos.len()) {
                            continue;
                        }
                        let p = [0, 1, 2].map(|j| xf.transform_point3(pos[v(j)]));
                        let flat = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
                        let n = [0, 1, 2].map(|j| {
                            normals
                                .as_ref()
                                .and_then(|n| n.get(v(j)))
                                .map(|n| normal_m.transform_vector3(*n).normalize_or(flat))
                                .unwrap_or(flat)
                        });
                        let uv = [0, 1, 2].map(|j| {
                            uvs.as_ref()
                                .and_then(|u| u.get(v(j)))
                                .copied()
                                .unwrap_or_default()
                        });
                        let color = [0, 1, 2].map(|j| {
                            colors
                                .as_ref()
                                .and_then(|c| c.get(v(j)))
                                .copied()
                                .unwrap_or(Vec4::ONE)
                        });
                        out.push(Tri {
                            p,
                            n,
                            uv,
                            color,
                            base,
                            texture,
                        });
                    }
                }
            }
        }
        for c in node.children() {
            stack.push((c, world));
        }
    }
    out
}

/// Bakes the impostor of `src`'s default scene into the sphere about its
/// bounds `(min, max)`.
pub fn bake(
    src: &GltfSource,
    instanced: &std::collections::BTreeMap<usize, Vec<Mat4>>,
    (min, max): (Vec3, Vec3),
    b: ImpostorBake,
) -> Result<Baked, String> {
    let center = (min + max) * 0.5;
    let radius = ((max - min) * 0.5).length().max(1e-3);
    let tris = triangles(src, instanced);
    // The base colour textures, decoded once (linear, premultiplied not).
    let mut images: std::collections::BTreeMap<usize, Image> = std::collections::BTreeMap::new();
    for t in &tris {
        if let Some(i) = t.texture
            && !images.contains_key(&i)
            && let Some(bytes) = src.images.get(i)
        {
            images.insert(i, crate::cook::texture::decode(bytes)?);
        }
    }
    let grid = b.grid;
    let size = grid * b.frame;
    let mut albedo = vec![0u8; (size * size * 4) as usize];
    let mut normal = vec![0u8; (size * size * 4) as usize];
    let ss = 2u32;
    let fine = b.frame * ss;
    for j in 0..grid {
        for i in 0..grid {
            let d = frame_direction(i, j, grid);
            let (right, up) = frame_basis(d);
            // Per fine pixel: depth, albedo (linear rgba) and normal.
            let n_px = (fine * fine) as usize;
            let mut depth = vec![f32::MIN; n_px];
            let mut col = vec![Vec4::ZERO; n_px];
            let mut nrm = vec![Vec3::ZERO; n_px];
            let to_px = |p: Vec3| {
                let q = p - center;
                let x = q.dot(right) / radius;
                let y = q.dot(up) / radius;
                Vec3::new(
                    (x * 0.5 + 0.5) * fine as f32,
                    (0.5 - y * 0.5) * fine as f32,
                    q.dot(d) / radius,
                )
            };
            for t in &tris {
                let s = t.p.map(to_px);
                let area =
                    (s[1].x - s[0].x) * (s[2].y - s[0].y) - (s[1].y - s[0].y) * (s[2].x - s[0].x);
                if area.abs() < 1e-12 {
                    continue;
                }
                let lo = s[0].min(s[1]).min(s[2]);
                let hi = s[0].max(s[1]).max(s[2]);
                let (x0, y0) = (lo.x.floor().max(0.0) as u32, lo.y.floor().max(0.0) as u32);
                let (x1, y1) = (
                    (hi.x.ceil() as i64).clamp(0, fine as i64) as u32,
                    (hi.y.ceil() as i64).clamp(0, fine as i64) as u32,
                );
                for y in y0..y1 {
                    for x in x0..x1 {
                        let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                        let e = |a: Vec3, b: Vec3| {
                            (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x)
                        };
                        let w = Vec3::new(e(s[1], s[2]), e(s[2], s[0]), e(s[0], s[1])) / area;
                        if w.min_element() < 0.0 {
                            continue;
                        }
                        let z = w.x * s[0].z + w.y * s[1].z + w.z * s[2].z;
                        let at = (y * fine + x) as usize;
                        if z <= depth[at] {
                            continue;
                        }
                        depth[at] = z;
                        let uv = t.uv[0] * w.x + t.uv[1] * w.y + t.uv[2] * w.z;
                        let vc = t.color[0] * w.x + t.color[1] * w.y + t.color[2] * w.z;
                        let tex = t
                            .texture
                            .and_then(|i| images.get(&i))
                            .map_or(Vec4::ONE, |img| {
                                let u = uv.x - uv.x.floor();
                                let v = uv.y - uv.y.floor();
                                let tx = ((u * img.width as f32) as u32).min(img.width - 1);
                                let ty = ((v * img.height as f32) as u32).min(img.height - 1);
                                let k = ((ty * img.width + tx) * 4) as usize;
                                let c = |o: usize| f32::from(img.rgba[k + o]) / 255.0;
                                Vec4::new(
                                    srgb_to_linear(c(0)),
                                    srgb_to_linear(c(1)),
                                    srgb_to_linear(c(2)),
                                    c(3),
                                )
                            });
                        let c = t.base * tex * vc;
                        // A cut-out texel (alpha under half) leaves it uncovered.
                        if c.w < 0.5 {
                            depth[at] = f32::MIN;
                            continue;
                        }
                        col[at] = c.truncate().extend(1.0);
                        let mut n = (t.n[0] * w.x + t.n[1] * w.y + t.n[2] * w.z).normalize_or(d);
                        // The side seen: a back face shows its normal turned to the viewer.
                        if n.dot(d) < 0.0 {
                            n = -n;
                        }
                        nrm[at] = Vec3::new(n.dot(right), n.dot(up), n.dot(d));
                    }
                }
            }
            // Box filter down to the frame's pixels and write them.
            let (ox, oy) = (i * b.frame, j * b.frame);
            for y in 0..b.frame {
                for x in 0..b.frame {
                    let (mut c, mut n, mut z, mut k) = (Vec4::ZERO, Vec3::ZERO, 0.0f32, 0.0f32);
                    for sy in 0..ss {
                        for sx in 0..ss {
                            let at = ((y * ss + sy) * fine + x * ss + sx) as usize;
                            if depth[at] > f32::MIN {
                                c += col[at];
                                n += nrm[at];
                                z += depth[at];
                                k += 1.0;
                            }
                        }
                    }
                    let at = (((oy + y) * size + ox + x) * 4) as usize;
                    if k == 0.0 {
                        continue;
                    }
                    let a = k / (ss * ss) as f32;
                    let c = c / k;
                    albedo[at] = (linear_to_srgb(c.x) * 255.0).round() as u8;
                    albedo[at + 1] = (linear_to_srgb(c.y) * 255.0).round() as u8;
                    albedo[at + 2] = (linear_to_srgb(c.z) * 255.0).round() as u8;
                    albedo[at + 3] = (a * 255.0).round() as u8;
                    let n = n.normalize_or(Vec3::Z) * 0.5 + 0.5;
                    let z = (z / k).clamp(-1.0, 1.0) * 0.5 + 0.5;
                    for (o, v) in [n.x, n.y, n.z, z].into_iter().enumerate() {
                        normal[at + o] = (v * 255.0).round() as u8;
                    }
                }
            }
            dilate(&mut albedo, &mut normal, size, ox, oy, b.frame);
        }
    }
    Ok(Baked {
        albedo: Image {
            width: size,
            height: size,
            rgba: albedo,
        },
        normal: Image {
            width: size,
            height: size,
            rgba: normal,
        },
        center,
        radius,
    })
}

/// Gives a frame's uncovered texels their nearest covered neighbour's
/// colour and normal (alpha kept 0), a few texels out from the silhouette.
fn dilate(albedo: &mut [u8], normal: &mut [u8], size: u32, ox: u32, oy: u32, frame: u32) {
    let mut filled: Vec<bool> = (0..frame * frame)
        .map(|k| albedo[((((oy + k / frame) * size) + ox + k % frame) * 4 + 3) as usize] > 0)
        .collect();
    if !filled.iter().any(|f| *f) {
        return;
    }
    for _ in 0..8 {
        let before = filled.clone();
        for y in 0..frame {
            for x in 0..frame {
                if before[(y * frame + x) as usize] {
                    continue;
                }
                let from = [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)]
                    .into_iter()
                    .map(|(dx, dy)| (x as i32 + dx, y as i32 + dy))
                    .find(|&(nx, ny)| {
                        nx >= 0
                            && ny >= 0
                            && (nx as u32) < frame
                            && (ny as u32) < frame
                            && before[(ny as u32 * frame + nx as u32) as usize]
                    });
                if let Some((nx, ny)) = from {
                    let src = ((((oy + ny as u32) * size) + ox + nx as u32) * 4) as usize;
                    let dst = ((((oy + y) * size) + ox + x) * 4) as usize;
                    for o in 0..3 {
                        albedo[dst + o] = albedo[src + o];
                    }
                    for o in 0..4 {
                        normal[dst + o] = normal[src + o];
                    }
                    filled[(y * frame + x) as usize] = true;
                }
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A GLB of two unit boxes: red about (1, 0, 0) and blue about (-1, 0,
    /// 0), each its own mesh and material.
    pub(crate) fn two_boxes() -> Vec<u8> {
        let mut bin: Vec<u8> = Vec::new();
        let mut views = Vec::new();
        let mut accessors = Vec::new();
        let mut meshes = Vec::new();
        for (k, x) in [1.0f32, -1.0].into_iter().enumerate() {
            let mut pos: Vec<[f32; 3]> = Vec::new();
            let mut nrm: Vec<[f32; 3]> = Vec::new();
            let mut idx: Vec<u32> = Vec::new();
            for axis in 0..3 {
                for sign in [-1.0f32, 1.0] {
                    let n = Vec3::from(std::array::from_fn::<f32, 3, _>(|i| {
                        if i == axis { sign } else { 0.0 }
                    }));
                    let (a, b) = (
                        n.any_orthonormal_vector(),
                        n.cross(n.any_orthonormal_vector()),
                    );
                    let base = pos.len() as u32;
                    for (s, t) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                        let p = (n + a * s + b * t) * 0.5 + Vec3::new(x, 0.0, 0.0);
                        pos.push(p.to_array());
                        nrm.push(n.to_array());
                    }
                    // Wound counter-clockwise seen from outside.
                    let ccw = a.cross(b).dot(n) > 0.0;
                    let q = if ccw {
                        [0, 1, 2, 0, 2, 3]
                    } else {
                        [0, 2, 1, 0, 3, 2]
                    };
                    idx.extend(q.iter().map(|i| base + i));
                }
            }
            let (lo, hi) = pos.iter().fold(
                (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)),
                |(l, h), p| (l.min(Vec3::from(*p)), h.max(Vec3::from(*p))),
            );
            let mut push = |bytes: &[u8]| {
                let at = bin.len();
                bin.extend_from_slice(bytes);
                while !bin.len().is_multiple_of(4) {
                    bin.push(0);
                }
                views.push(
                    serde_json::json!({"buffer": 0, "byteOffset": at, "byteLength": bytes.len()}),
                );
                views.len() - 1
            };
            let pv = push(bytemuck::cast_slice(&pos));
            let nv = push(bytemuck::cast_slice(&nrm));
            let iv = push(bytemuck::cast_slice(&idx));
            accessors.push(serde_json::json!({"bufferView": pv, "componentType": 5126, "count": pos.len(), "type": "VEC3", "min": lo.to_array(), "max": hi.to_array()}));
            accessors.push(serde_json::json!({"bufferView": nv, "componentType": 5126, "count": nrm.len(), "type": "VEC3"}));
            accessors.push(serde_json::json!({"bufferView": iv, "componentType": 5125, "count": idx.len(), "type": "SCALAR"}));
            meshes.push(serde_json::json!({"primitives": [{"attributes": {"POSITION": 3 * k, "NORMAL": 3 * k + 1}, "indices": 3 * k + 2, "material": k}]}));
        }
        let json = serde_json::json!({
            "asset": {"version": "2.0"},
            "scene": 0,
            "scenes": [{"nodes": [0, 1]}],
            "nodes": [{"mesh": 0}, {"mesh": 1}],
            "meshes": meshes,
            "materials": [
                {"pbrMetallicRoughness": {"baseColorFactor": [1.0, 0.0, 0.0, 1.0]}},
                {"pbrMetallicRoughness": {"baseColorFactor": [0.0, 0.0, 1.0, 1.0]}}
            ],
            "accessors": accessors,
            "bufferViews": views,
            "buffers": [{"byteLength": bin.len()}]
        });
        let mut j = serde_json::to_vec(&json).unwrap();
        while !j.len().is_multiple_of(4) {
            j.push(b' ');
        }
        let mut out = Vec::new();
        out.extend_from_slice(b"glTF");
        out.extend_from_slice(&2u32.to_le_bytes());
        out.extend_from_slice(&((12 + 8 + j.len() + 8 + bin.len()) as u32).to_le_bytes());
        out.extend_from_slice(&(j.len() as u32).to_le_bytes());
        out.extend_from_slice(b"JSON");
        out.extend_from_slice(&j);
        out.extend_from_slice(&(bin.len() as u32).to_le_bytes());
        out.extend_from_slice(b"BIN\0");
        out.extend_from_slice(&bin);
        out
    }

    /// Texel (x, y) of frame (i, j) of a `grid`-sided atlas of `frame` px.
    fn texel(
        img: &Image,
        grid: u32,
        frame: u32,
        (i, j): (u32, u32),
        (x, y): (u32, u32),
    ) -> [u8; 4] {
        let at = (((j * frame + y) * grid * frame + i * frame + x) * 4) as usize;
        img.rgba[at..at + 4].try_into().unwrap()
    }

    /// Ways this could fail: a frame drawn from the opposite direction
    /// (the far box in front), mirrored against og-render's frame basis
    /// (the boxes swapped left and right seen from above), the normal or
    /// depth not in the frame's basis, the coverage not cut where nothing
    /// is, or no colour bled out past the silhouette.
    #[test]
    fn frames_show_the_model_from_their_directions() {
        let src = super::super::load(&two_boxes(), std::path::Path::new(".")).unwrap();
        let (grid, frame) = (3, 32);
        let b = src.impostor(ImpostorBake { grid, frame }).unwrap();
        assert_eq!((b.albedo.width, b.normal.height), (96, 96));
        // The bounds' sphere: x from -1.5 to 1.5, y and z ±0.5.
        assert!(b.center.length() < 1e-5);
        assert!((b.radius - Vec3::new(1.5, 0.5, 0.5).length()).abs() < 1e-4);
        let red = |c: [u8; 4]| c[0] > 200 && c[1] < 40 && c[2] < 40 && c[3] == 255;
        let blue = |c: [u8; 4]| c[2] > 200 && c[0] < 40 && c[1] < 40 && c[3] == 255;
        let mid = (frame / 2, frame / 2);
        // Frame (2, 1) looks from +x: the red box's face, facing the
        // viewer, nearer than the centre (0.5 + 0.5 × 1.5 / r = 0.95).
        assert!(red(texel(&b.albedo, grid, frame, (2, 1), mid)));
        let n = texel(&b.normal, grid, frame, (2, 1), mid);
        assert!(
            n[2] > 245 && (n[0] as i32 - 128).abs() < 4 && (n[1] as i32 - 128).abs() < 4,
            "{n:?}"
        );
        assert!((n[3] as i32 - 242).abs() <= 2, "{n:?}");
        // Frame (0, 1) from -x: the blue box in front.
        assert!(blue(texel(&b.albedo, grid, frame, (0, 1), mid)));
        // Frame (1, 1) from +y: right is -x (og-render's frame basis), so
        // red on the left and blue on the right; facing up.
        let (right, _) = og_render::impostors::frame_basis(Vec3::Y);
        assert!(right.x < -0.99);
        let left_px = (
            frame / 2 - (frame as f32 / 2.0 / b.radius) as u32,
            frame / 2,
        );
        let right_px = (
            frame / 2 + (frame as f32 / 2.0 / b.radius) as u32,
            frame / 2,
        );
        assert!(red(texel(&b.albedo, grid, frame, (1, 1), left_px)));
        assert!(blue(texel(&b.albedo, grid, frame, (1, 1), right_px)));
        assert!(texel(&b.normal, grid, frame, (1, 1), left_px)[2] > 245);
        // A frame's corner: uncovered, its colour bled from the boxes.
        let corner = texel(&b.albedo, grid, frame, (1, 1), (0, 0));
        assert_eq!(corner[3], 0);
        // Just past the top edge seen from above (z = 0.5 on the image's
        // up), the red bleeds out with alpha 0.
        let edge = (
            left_px.0,
            frame / 2 - (frame as f32 / 2.0 * 0.5 / b.radius) as u32 - 2,
        );
        let c = texel(&b.albedo, grid, frame, (1, 1), edge);
        assert!(c[3] == 0 && c[0] > 200, "{c:?}");
    }
}
