//! glTF 2.0 import (feature `gltf`): every triangle primitive of the default
//! scene becomes a mesh with its node transform baked in, glTF materials
//! become [`Material`]s (with their `KHR_materials_*` physical extensions,
//! [`physical_extensions`], which the cook reads too) and their images
//! [`TextureData`]. A bridge until the cook (M4) produces `.ogb` meshes.

use crate::RenderError;
use crate::material::{PhysicalExtensions, StandardMaterial};
use crate::scene::{
    AlphaMode, ColorSpace, Material, MaterialHandle, MeshData, MeshHandle, Mips, RenderScene,
    TextureData, TextureHandle, Vertex,
};
use glam::{Mat4, Quat, Vec3};
use std::collections::HashMap;

/// One primitive: its geometry (in model space) and the index of its material
/// in [`GltfModel::materials`].
#[derive(Clone, Debug)]
pub struct GltfPrimitive {
    pub mesh: MeshData,
    pub material: usize,
}

/// A loaded model. Material texture handles index [`GltfModel::textures`]
/// until [`GltfModel::add_to`] maps them into a scene.
#[derive(Clone, Debug)]
pub struct GltfModel {
    pub primitives: Vec<GltfPrimitive>,
    pub materials: Vec<Material>,
    /// Each material's physical extensions (`present` 0 without any), in
    /// [`GltfModel::materials`] order.
    pub physical: Vec<PhysicalExtensions<TextureHandle>>,
    pub textures: Vec<TextureData>,
    /// Model-space bounds over every primitive.
    pub min: Vec3,
    pub max: Vec3,
}

/// The scene handles of one model's parts, in [`GltfModel::primitives`] order.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelParts {
    pub parts: Vec<(MeshHandle, MaterialHandle)>,
}

impl ModelParts {
    /// Submits one instance of the whole model at `transform`.
    pub fn instance(&self, scene: &mut RenderScene, transform: Mat4) {
        for &(mesh, material) in &self.parts {
            scene.add_instance(mesh, material, transform);
        }
    }
}

impl GltfModel {
    /// Sets how every texture of the model is mipmapped when uploaded.
    pub fn set_mips(&mut self, mips: Mips) {
        for t in &mut self.textures {
            t.mips = mips;
        }
    }

    /// Adds the model's textures, materials and meshes to `scene`.
    pub fn add_to(&self, scene: &mut RenderScene) -> ModelParts {
        let textures: Vec<TextureHandle> = self
            .textures
            .iter()
            .map(|t| scene.add_texture(t.clone()))
            .collect();
        let remap = |h: Option<TextureHandle>| h.map(|h| textures[h.index()]);
        let materials: Vec<MaterialHandle> = self
            .materials
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let m = Material {
                    base_color_texture: remap(m.base_color_texture),
                    metallic_roughness_texture: remap(m.metallic_roughness_texture),
                    normal_texture: remap(m.normal_texture),
                    emissive_texture: remap(m.emissive_texture),
                    ..m.clone()
                };
                match self.physical.get(i).filter(|p| p.present != 0) {
                    Some(p) => {
                        let mut standard = StandardMaterial::from(&m);
                        p.map(|h| Some(textures[h.index()])).apply(&mut standard);
                        scene.add_standard_material(standard)
                    }
                    None => scene.add_material(m),
                }
            })
            .collect();
        let parts = self
            .primitives
            .iter()
            .map(|p| (scene.add_mesh(p.mesh.clone()), materials[p.material]))
            .collect();
        ModelParts { parts }
    }
}

fn to_rgba(img: &gltf::image::Data) -> Vec<u8> {
    use gltf::image::Format;
    let n = (img.width * img.height) as usize;
    let mut rgba = Vec::with_capacity(n * 4);
    let f32_at = |p: &[u8], i: usize| {
        (f32::from_le_bytes([p[i], p[i + 1], p[i + 2], p[i + 3]]).clamp(0.0, 1.0) * 255.0) as u8
    };
    match img.format {
        Format::R8G8B8A8 => rgba.extend_from_slice(&img.pixels),
        Format::R8G8B8 => img
            .pixels
            .as_chunks::<3>()
            .0
            .iter()
            .for_each(|p| rgba.extend_from_slice(&[p[0], p[1], p[2], 255])),
        Format::R8G8 => img
            .pixels
            .as_chunks::<2>()
            .0
            .iter()
            .for_each(|p| rgba.extend_from_slice(&[p[0], p[1], 0, 255])),
        Format::R8 => img
            .pixels
            .iter()
            .for_each(|&p| rgba.extend_from_slice(&[p, p, p, 255])),
        Format::R16G16B16A16 => img
            .pixels
            .as_chunks::<8>()
            .0
            .iter()
            .for_each(|p| rgba.extend_from_slice(&[p[1], p[3], p[5], p[7]])),
        Format::R16G16B16 => img
            .pixels
            .as_chunks::<6>()
            .0
            .iter()
            .for_each(|p| rgba.extend_from_slice(&[p[1], p[3], p[5], 255])),
        Format::R16G16 => img
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .for_each(|p| rgba.extend_from_slice(&[p[1], p[3], 0, 255])),
        Format::R16 => img
            .pixels
            .as_chunks::<2>()
            .0
            .iter()
            .for_each(|p| rgba.extend_from_slice(&[p[1], p[1], p[1], 255])),
        Format::R32G32B32A32FLOAT => img.pixels.as_chunks::<16>().0.iter().for_each(|p| {
            rgba.extend_from_slice(&[f32_at(p, 0), f32_at(p, 4), f32_at(p, 8), f32_at(p, 12)])
        }),
        Format::R32G32B32FLOAT => {
            img.pixels.as_chunks::<12>().0.iter().for_each(|p| {
                rgba.extend_from_slice(&[f32_at(p, 0), f32_at(p, 4), f32_at(p, 8), 255])
            })
        }
    }
    rgba
}

/// What a texture a glTF material samples holds: how it is decoded (and cooked).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureKind {
    /// sRGB colour.
    Color,
    /// A tangent-space normal map.
    Normal,
    /// Linear data (masks, factors).
    Linear,
}

/// The material's physical extensions (`KHR_materials_clearcoat`,
/// `_sheen`, `_transmission`, `_volume`, `_ior`, `_iridescence`,
/// `_anisotropy`), each texture as `texture` makes it from the glTF
/// texture and what it holds. `present` is 0 for a material with none.
pub fn physical_extensions<T>(
    doc: &gltf::Document,
    m: &gltf::Material,
    mut texture: impl FnMut(gltf::Texture, TextureKind) -> T,
) -> PhysicalExtensions<T> {
    use gltf::json::Value;
    type Ext<T> = PhysicalExtensions<T>;
    let mut e = Ext::default();
    let num = |v: &Value, k: &str, fallback: f32| {
        v.get(k)
            .and_then(Value::as_f64)
            .map_or(fallback, |x| x as f32)
    };
    let rgb = |v: &Value, k: &str, fallback: [f32; 3]| {
        let mut c = fallback;
        if let Some(a) = v.get(k).and_then(Value::as_array) {
            for (o, x) in c.iter_mut().zip(a) {
                *o = x.as_f64().map_or(*o, |x| x as f32);
            }
        }
        c
    };
    let mut tex = |v: &Value, k: &str, kind: TextureKind| {
        v.get(k)
            .and_then(|t| t.get("index"))
            .and_then(Value::as_u64)
            .and_then(|i| doc.textures().nth(i as usize))
            .map(|t| texture(t, kind))
    };
    use TextureKind::{Color, Linear, Normal};
    if let Some(v) = m.extension_value("KHR_materials_clearcoat") {
        e.present |= Ext::<T>::CLEARCOAT;
        e.clearcoat = num(v, "clearcoatFactor", 0.0);
        e.clearcoat_texture = tex(v, "clearcoatTexture", Linear);
        e.clearcoat_roughness = num(v, "clearcoatRoughnessFactor", 0.0);
        e.clearcoat_roughness_texture = tex(v, "clearcoatRoughnessTexture", Linear);
        e.clearcoat_normal_texture = tex(v, "clearcoatNormalTexture", Normal);
        e.clearcoat_normal_scale = v
            .get("clearcoatNormalTexture")
            .map_or(1.0, |t| num(t, "scale", 1.0));
    }
    if let Some(v) = m.extension_value("KHR_materials_sheen") {
        e.present |= Ext::<T>::SHEEN;
        e.sheen_color = rgb(v, "sheenColorFactor", [0.0; 3]);
        e.sheen_color_texture = tex(v, "sheenColorTexture", Color);
        e.sheen_roughness = num(v, "sheenRoughnessFactor", 0.0);
        e.sheen_roughness_texture = tex(v, "sheenRoughnessTexture", Linear);
    }
    if let Some(v) = m.extension_value("KHR_materials_transmission") {
        e.present |= Ext::<T>::TRANSMISSION;
        e.transmission = num(v, "transmissionFactor", 0.0);
        e.transmission_texture = tex(v, "transmissionTexture", Linear);
    }
    if let Some(v) = m.extension_value("KHR_materials_volume") {
        e.present |= Ext::<T>::VOLUME;
        e.thickness = num(v, "thicknessFactor", 0.0);
        e.thickness_texture = tex(v, "thicknessTexture", Linear);
        // Absent is +infinity: no absorption.
        e.attenuation_distance = num(v, "attenuationDistance", 0.0);
        e.attenuation_color = rgb(v, "attenuationColor", [1.0; 3]);
    }
    if let Some(v) = m.extension_value("KHR_materials_ior") {
        e.present |= Ext::<T>::IOR;
        e.ior = num(v, "ior", 1.5);
    }
    if let Some(v) = m.extension_value("KHR_materials_iridescence") {
        e.present |= Ext::<T>::IRIDESCENCE;
        e.iridescence = num(v, "iridescenceFactor", 0.0);
        e.iridescence_texture = tex(v, "iridescenceTexture", Linear);
        e.iridescence_ior = num(v, "iridescenceIor", 1.3);
        e.iridescence_thickness = [
            num(v, "iridescenceThicknessMinimum", 100.0),
            num(v, "iridescenceThicknessMaximum", 400.0),
        ];
        e.iridescence_thickness_texture = tex(v, "iridescenceThicknessTexture", Linear);
    }
    if let Some(v) = m.extension_value("KHR_materials_anisotropy") {
        e.present |= Ext::<T>::ANISOTROPY;
        e.anisotropy = num(v, "anisotropyStrength", 0.0);
        e.anisotropy_rotation = num(v, "anisotropyRotation", 0.0);
        e.anisotropy_texture = tex(v, "anisotropyTexture", Linear);
    }
    e
}

fn flat_normals(positions: &[[f32; 3]]) -> Vec<[f32; 3]> {
    let mut out = vec![[0.0, 1.0, 0.0]; positions.len()];
    for (i, t) in positions.as_chunks::<3>().0.iter().enumerate() {
        let (a, b, c) = (Vec3::from(t[0]), Vec3::from(t[1]), Vec3::from(t[2]));
        let n = (b - a).cross(c - a).normalize_or_zero().to_array();
        out[i * 3..i * 3 + 3].copy_from_slice(&[n, n, n]);
    }
    out
}

/// Loads a binary glTF (`.glb`, or `.gltf` with embedded buffers).
pub fn load_glb(bytes: &[u8]) -> Result<GltfModel, RenderError> {
    let (doc, buffers, images) =
        gltf::import_slice(bytes).map_err(|e| RenderError::InvalidData(format!("gltf: {e}")))?;
    let mut textures: Vec<TextureData> = Vec::new();
    let mut texture_of: HashMap<(usize, bool), TextureHandle> = HashMap::new();
    let mut texture =
        |source: usize, srgb: bool, textures: &mut Vec<TextureData>| -> TextureHandle {
            *texture_of.entry((source, srgb)).or_insert_with(|| {
                let img = &images[source];
                textures.push(TextureData {
                    width: img.width,
                    height: img.height,
                    rgba: to_rgba(img),
                    color_space: if srgb {
                        ColorSpace::Srgb
                    } else {
                        ColorSpace::Linear
                    },
                    mips: Mips::Generate,
                });
                // The scene-independent index; `add_to` maps it to a scene handle.
                TextureHandle::from_index(textures.len() - 1)
            })
        };
    let mut materials: Vec<Material> = Vec::new();
    let mut physical = Vec::new();
    for m in doc.materials() {
        physical.push(physical_extensions(&doc, &m, |t, kind| {
            texture(
                t.source().index(),
                kind == TextureKind::Color,
                &mut textures,
            )
        }));
        let pbr = m.pbr_metallic_roughness();
        let emissive = m.emissive_factor();
        let strength = m.emissive_strength().unwrap_or(1.0);
        materials.push(Material {
            base_color: pbr.base_color_factor(),
            base_color_texture: pbr
                .base_color_texture()
                .map(|t| texture(t.texture().source().index(), true, &mut textures)),
            metallic: pbr.metallic_factor(),
            roughness: pbr.roughness_factor(),
            metallic_roughness_texture: pbr
                .metallic_roughness_texture()
                .map(|t| texture(t.texture().source().index(), false, &mut textures)),
            normal_texture: m
                .normal_texture()
                .map(|t| texture(t.texture().source().index(), false, &mut textures)),
            normal_scale: m.normal_texture().map(|t| t.scale()).unwrap_or(1.0),
            emissive: [
                emissive[0] * strength,
                emissive[1] * strength,
                emissive[2] * strength,
            ],
            emissive_texture: m
                .emissive_texture()
                .map(|t| texture(t.texture().source().index(), true, &mut textures)),
            alpha_mode: match m.alpha_mode() {
                gltf::material::AlphaMode::Mask => AlphaMode::Mask {
                    cutoff: m.alpha_cutoff().unwrap_or(0.5),
                },
                _ => AlphaMode::Opaque,
            },
        });
    }
    let default_material = materials.len();
    materials.push(Material {
        base_color: [0.8, 0.8, 0.8, 1.0],
        metallic: 0.0,
        roughness: 0.6,
        ..Default::default()
    });

    let mut primitives = Vec::new();
    let (mut min, mut max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    let scene = doc
        .default_scene()
        .or_else(|| doc.scenes().next())
        .ok_or_else(|| RenderError::InvalidData("gltf: no scene".into()))?;
    let mut stack: Vec<(gltf::Node, Mat4)> = scene.nodes().map(|n| (n, Mat4::IDENTITY)).collect();
    while let Some((node, parent)) = stack.pop() {
        let (t, r, s) = node.transform().decomposed();
        let world = parent
            * Mat4::from_scale_rotation_translation(
                Vec3::from(s),
                Quat::from_array(r),
                Vec3::from(t),
            );
        if let Some(mesh) = node.mesh() {
            for prim in mesh.primitives() {
                if prim.mode() != gltf::mesh::Mode::Triangles {
                    continue;
                }
                let reader = prim.reader(|b| Some(&buffers[b.index()]));
                let Some(positions) = reader
                    .read_positions()
                    .map(|p| p.collect::<Vec<[f32; 3]>>())
                else {
                    continue;
                };
                let normals: Vec<[f32; 3]> = reader
                    .read_normals()
                    .map(|n| n.collect())
                    .unwrap_or_else(|| flat_normals(&positions));
                let uvs: Vec<[f32; 2]> = reader
                    .read_tex_coords(0)
                    .map(|u| u.into_f32().collect())
                    .unwrap_or_default();
                let tangents: Vec<[f32; 4]> = reader
                    .read_tangents()
                    .map(|t| t.collect())
                    .unwrap_or_default();
                let colors: Vec<[f32; 4]> = reader
                    .read_colors(0)
                    .map(|c| c.into_rgba_f32().collect())
                    .unwrap_or_default();
                let indices: Vec<u32> = reader
                    .read_indices()
                    .map(|i| i.into_u32().collect())
                    .unwrap_or_else(|| (0..positions.len() as u32).collect());
                // Bake the node transform so an instance is a single matrix.
                let vertices: Vec<Vertex> = positions
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        let position = world.transform_point3(Vec3::from(*p));
                        min = min.min(position);
                        max = max.max(position);
                        let normal = world
                            .transform_vector3(Vec3::from(
                                normals.get(i).copied().unwrap_or([0.0, 1.0, 0.0]),
                            ))
                            .normalize_or_zero();
                        // The file's tangent, or none (generated at upload).
                        let tangent = tangents.get(i).map_or([0.0; 4], |t| {
                            let v = world.transform_vector3(Vec3::new(t[0], t[1], t[2]));
                            v.normalize_or_zero().extend(t[3]).to_array()
                        });
                        Vertex {
                            tangent,
                            position: position.to_array(),
                            normal: normal.to_array(),
                            uv: uvs.get(i).copied().unwrap_or([0.0, 0.0]),
                            color: colors.get(i).copied().unwrap_or([1.0; 4]),
                        }
                    })
                    .collect();
                primitives.push(GltfPrimitive {
                    mesh: MeshData { vertices, indices },
                    material: prim.material().index().unwrap_or(default_material),
                });
            }
        }
        for child in node.children() {
            stack.push((child, world));
        }
    }
    if primitives.is_empty() {
        return Err(RenderError::InvalidData(
            "gltf: no triangle primitives".into(),
        ));
    }
    Ok(GltfModel {
        primitives,
        materials,
        physical,
        textures,
        min,
        max,
    })
}
