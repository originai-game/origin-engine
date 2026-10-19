//! glTF 2.0 import (feature `gltf`): every triangle primitive of the default
//! scene becomes a mesh with its node transform baked in, glTF materials
//! become [`Material`]s and their images [`TextureData`]. A bridge until the
//! cook (M4) produces `.ogb` meshes.

use crate::RenderError;
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
            .map(|m| {
                scene.add_material(Material {
                    base_color_texture: remap(m.base_color_texture),
                    metallic_roughness_texture: remap(m.metallic_roughness_texture),
                    normal_texture: remap(m.normal_texture),
                    emissive_texture: remap(m.emissive_texture),
                    ..m.clone()
                })
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
    for m in doc.materials() {
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
        textures,
        min,
        max,
    })
}
