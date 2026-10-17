//! glTF loading into GPU-ready meshes and materials, plus the generated ground.

use glam::{Mat4, Quat, Vec3, Vec4};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2, 3 => Float32x4],
    };
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct Instance {
    pub model: [[f32; 4]; 4],
}

impl Instance {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Instance>() as u64,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &wgpu::vertex_attr_array![4 => Float32x4, 5 => Float32x4, 6 => Float32x4, 7 => Float32x4],
    };
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct MaterialFactors {
    pub base_color: [f32; 4],
    pub mr: [f32; 4],
    pub emissive: [f32; 4],
}

#[derive(Clone, Debug)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Material {
    pub factors: MaterialFactors,
    pub base_color: Option<ImageData>,
    pub metallic_roughness: Option<ImageData>,
    pub normal: Option<ImageData>,
    pub emissive: Option<ImageData>,
}

impl Material {
    pub fn plain(base: [f32; 4], metallic: f32, roughness: f32) -> Self {
        Material {
            factors: MaterialFactors { base_color: base, mr: [metallic, roughness, 1.0, 0.0], emissive: [0.0, 0.0, 0.0, -1.0] },
            base_color: None,
            metallic_roughness: None,
            normal: None,
            emissive: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Primitive {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: usize,
    /// The node's world transform inside the glTF scene.
    pub transform: Mat4,
}

#[derive(Clone, Debug, Default)]
pub struct Model {
    pub primitives: Vec<Primitive>,
    pub materials: Vec<Material>,
    pub min: Vec3,
    pub max: Vec3,
}

fn to_rgba(img: &gltf::image::Data) -> ImageData {
    use gltf::image::Format;
    let n = (img.width * img.height) as usize;
    let mut rgba = Vec::with_capacity(n * 4);
    match img.format {
        Format::R8G8B8A8 => rgba.extend_from_slice(&img.pixels),
        Format::R8G8B8 => {
            for p in img.pixels.chunks_exact(3) {
                rgba.extend_from_slice(&[p[0], p[1], p[2], 255]);
            }
        }
        Format::R8G8 => {
            for p in img.pixels.chunks_exact(2) {
                rgba.extend_from_slice(&[p[0], p[1], 0, 255]);
            }
        }
        Format::R8 => {
            for p in img.pixels.iter() {
                rgba.extend_from_slice(&[*p, *p, *p, 255]);
            }
        }
        Format::R16G16B16A16 => {
            for p in img.pixels.chunks_exact(8) {
                rgba.extend_from_slice(&[p[1], p[3], p[5], p[7]]);
            }
        }
        Format::R16G16B16 => {
            for p in img.pixels.chunks_exact(6) {
                rgba.extend_from_slice(&[p[1], p[3], p[5], 255]);
            }
        }
        Format::R16G16 => {
            for p in img.pixels.chunks_exact(4) {
                rgba.extend_from_slice(&[p[1], p[3], 0, 255]);
            }
        }
        Format::R16 => {
            for p in img.pixels.chunks_exact(2) {
                rgba.extend_from_slice(&[p[1], p[1], p[1], 255]);
            }
        }
        Format::R32G32B32A32FLOAT => {
            for p in img.pixels.chunks_exact(16) {
                let f = |i: usize| (f32::from_le_bytes([p[i], p[i + 1], p[i + 2], p[i + 3]]).clamp(0.0, 1.0) * 255.0) as u8;
                rgba.extend_from_slice(&[f(0), f(4), f(8), f(12)]);
            }
        }
        Format::R32G32B32FLOAT => {
            for p in img.pixels.chunks_exact(12) {
                let f = |i: usize| (f32::from_le_bytes([p[i], p[i + 1], p[i + 2], p[i + 3]]).clamp(0.0, 1.0) * 255.0) as u8;
                rgba.extend_from_slice(&[f(0), f(4), f(8), 255]);
            }
        }
    }
    ImageData { width: img.width, height: img.height, rgba }
}

/// Loads every mesh primitive of the default scene with its world transform.
pub fn load_glb(bytes: &[u8]) -> Result<Model, String> {
    let (doc, buffers, images) = gltf::import_slice(bytes).map_err(|e| format!("gltf: {e}"))?;
    let image_of = |info: Option<gltf::texture::Info>| info.map(|t| to_rgba(&images[t.texture().source().index()]));
    let mut materials: Vec<Material> = doc
        .materials()
        .map(|m| {
            let pbr = m.pbr_metallic_roughness();
            let normal = m.normal_texture().map(|n| (to_rgba(&images[n.texture().source().index()]), n.scale()));
            let cutoff = match m.alpha_mode() {
                gltf::material::AlphaMode::Mask => m.alpha_cutoff().unwrap_or(0.5),
                _ => -1.0,
            };
            let emissive = m.emissive_factor();
            let strength = m.emissive_strength().unwrap_or(1.0);
            Material {
                factors: MaterialFactors {
                    base_color: pbr.base_color_factor(),
                    mr: [pbr.metallic_factor(), pbr.roughness_factor(), normal.as_ref().map(|n| n.1).unwrap_or(1.0), if normal.is_some() { 1.0 } else { 0.0 }],
                    emissive: [emissive[0] * strength, emissive[1] * strength, emissive[2] * strength, cutoff],
                },
                base_color: image_of(pbr.base_color_texture()),
                metallic_roughness: image_of(pbr.metallic_roughness_texture()),
                normal: normal.map(|n| n.0),
                emissive: m.emissive_texture().map(|t| to_rgba(&images[t.texture().source().index()])),
            }
        })
        .collect();
    let default_material = materials.len();
    materials.push(Material::plain([0.8, 0.8, 0.8, 1.0], 0.0, 0.6));

    let mut model = Model { materials, min: Vec3::splat(f32::MAX), max: Vec3::splat(f32::MIN), ..Default::default() };
    let scene = doc.default_scene().or_else(|| doc.scenes().next()).ok_or("gltf: no scene")?;
    let mut stack: Vec<(gltf::Node, Mat4)> = scene.nodes().map(|n| (n, Mat4::IDENTITY)).collect();
    while let Some((node, parent)) = stack.pop() {
        let (t, r, s) = node.transform().decomposed();
        let local = Mat4::from_scale_rotation_translation(Vec3::from(s), Quat::from_array(r), Vec3::from(t));
        let world = parent * local;
        if let Some(mesh) = node.mesh() {
            for prim in mesh.primitives() {
                if prim.mode() != gltf::mesh::Mode::Triangles {
                    continue;
                }
                let reader = prim.reader(|b| Some(&buffers[b.index()]));
                let positions: Vec<[f32; 3]> = match reader.read_positions() {
                    Some(p) => p.collect(),
                    None => continue,
                };
                let normals: Vec<[f32; 3]> = reader.read_normals().map(|n| n.collect()).unwrap_or_else(|| flat_normals(&positions));
                let uvs: Vec<[f32; 2]> = reader.read_tex_coords(0).map(|u| u.into_f32().collect()).unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);
                let colors: Vec<[f32; 4]> = reader.read_colors(0).map(|c| c.into_rgba_f32().collect()).unwrap_or_else(|| vec![[1.0; 4]; positions.len()]);
                let indices: Vec<u32> = reader.read_indices().map(|i| i.into_u32().collect()).unwrap_or_else(|| (0..positions.len() as u32).collect());
                let vertices: Vec<Vertex> = positions
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        let wp = world.transform_point3(Vec3::from(*p));
                        model.min = model.min.min(wp);
                        model.max = model.max.max(wp);
                        Vertex { position: *p, normal: normals.get(i).copied().unwrap_or([0.0, 1.0, 0.0]), uv: uvs.get(i).copied().unwrap_or([0.0, 0.0]), color: colors.get(i).copied().unwrap_or([1.0; 4]) }
                    })
                    .collect();
                model.primitives.push(Primitive { vertices, indices, material: prim.material().index().unwrap_or(default_material), transform: world });
            }
        }
        for child in node.children() {
            stack.push((child, world));
        }
    }
    if model.primitives.is_empty() {
        return Err("gltf: no triangle primitives".into());
    }
    Ok(model)
}

fn flat_normals(positions: &[[f32; 3]]) -> Vec<[f32; 3]> {
    let mut out = vec![[0.0, 1.0, 0.0]; positions.len()];
    for tri in positions.chunks_exact(3).enumerate() {
        let (i, t) = tri;
        let a = Vec3::from(t[0]);
        let b = Vec3::from(t[1]);
        let c = Vec3::from(t[2]);
        let n = (b - a).cross(c - a).normalize_or_zero().to_array();
        for k in 0..3 {
            out[i * 3 + k] = n;
        }
    }
    out
}

/// A UV sphere of radius 0.5 (`segments` around, `rings` down) with a plain material:
/// the light mesh for the many-objects comparison.
pub fn sphere(segments: u32, rings: u32) -> Model {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for r in 0..=rings {
        let v = r as f32 / rings as f32;
        let theta = v * std::f32::consts::PI;
        for s in 0..=segments {
            let u = s as f32 / segments as f32;
            let phi = u * std::f32::consts::TAU;
            let n = Vec3::new(theta.sin() * phi.cos(), theta.cos(), theta.sin() * phi.sin());
            vertices.push(Vertex { position: (n * 0.5).to_array(), normal: n.to_array(), uv: [u, v], color: [1.0; 4] });
        }
    }
    for r in 0..rings {
        for s in 0..segments {
            let a = r * (segments + 1) + s;
            let b = a + segments + 1;
            indices.extend_from_slice(&[a, a + 1, b, a + 1, b + 1, b]);
        }
    }
    Model {
        primitives: vec![Primitive { vertices, indices, material: 0, transform: Mat4::IDENTITY }],
        materials: vec![Material::plain([0.85, 0.35, 0.2, 1.0], 0.1, 0.45)],
        min: Vec3::splat(-0.5),
        max: Vec3::splat(0.5),
    }
}

/// A ground quad of `size` centred at the origin, with tiled UVs.
pub fn ground(size: f32, tiles: f32) -> Primitive {
    let h = size / 2.0;
    let v = |x: f32, z: f32, u: f32, w: f32| Vertex { position: [x, 0.0, z], normal: [0.0, 1.0, 0.0], uv: [u * tiles, w * tiles], color: [1.0; 4] };
    Primitive {
        vertices: vec![v(-h, -h, 0.0, 0.0), v(h, -h, 1.0, 0.0), v(h, h, 1.0, 1.0), v(-h, h, 0.0, 1.0)],
        indices: vec![0, 2, 1, 0, 3, 2],
        material: 0,
        transform: Mat4::IDENTITY,
    }
}

/// A checker texture for the ground so the frame shows texture filtering and shadows.
pub fn checker(size: u32, cells: u32) -> ImageData {
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let c = ((x * cells / size) + (y * cells / size)) % 2 == 0;
            let v = if c { 170 } else { 120 };
            rgba.extend_from_slice(&[v, v, v, 255]);
        }
    }
    ImageData { width: size, height: size, rgba }
}

/// Lays `count` instances of a model of extent `extent` on a grid with `spacing`.
pub fn grid_instances(count: u32, spacing: f32, scale: f32, extent_min: Vec3) -> Vec<Instance> {
    let side = (count as f32).sqrt().ceil() as u32;
    let mut out = Vec::with_capacity(count as usize);
    for i in 0..count {
        let x = (i % side) as f32 - (side as f32 - 1.0) / 2.0;
        let z = (i / side) as f32 - (side as f32 - 1.0) / 2.0;
        let rot = Quat::from_rotation_y((i as f32 * 0.7).sin() * 1.2);
        let m = Mat4::from_translation(Vec3::new(x * spacing, -extent_min.y * scale, z * spacing)) * Mat4::from_quat(rot) * Mat4::from_scale(Vec3::splat(scale));
        out.push(Instance { model: m.to_cols_array_2d() });
    }
    out
}

pub fn vec4(v: Vec3, w: f32) -> [f32; 4] {
    Vec4::new(v.x, v.y, v.z, w).to_array()
}
