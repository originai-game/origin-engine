//! Decoded models, materials, primitives and renderer insertion.

use super::*;

/// One level of detail of a [`CookedPrimitive`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Lod {
    pub first: usize,
    pub count: usize,
    /// The level's largest distance from the full mesh, both ways, in the
    /// model's units (0 for LOD 0).
    pub error: f32,
    /// Its range of [`CookedPrimitive::clusters`] (count 0: none).
    pub cluster_first: usize,
    pub cluster_count: usize,
}

/// A decoded mesh primitive: every LOD shares the vertices.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedPrimitive {
    pub material: usize,
    pub vertices: Vec<Vertex>,
    /// Per vertex `xyz` tangent and bitangent sign `w`; empty when not cooked.
    pub tangents: Vec<[f32; 4]>,
    /// All LODs' triangles, LOD 0 first.
    pub indices: Vec<u32>,
    pub lods: Vec<Lod>,
    /// Every level's clusters (og-render's `meshlet`), each a range of
    /// `indices`; empty for a primitive made without them.
    pub clusters: Vec<og_render::meshlet::Cluster>,
    /// Exact local bounds of the source positions.
    pub min: Vec3,
    pub max: Vec3,
    /// The raycast BVH over LOD 0 the cook built (`None`: cooked with
    /// `raycast false`, or a blob of a cook before it): put together over
    /// [`CookedPrimitive::mesh`]`(0)` by [`CookedPrimitive::bvh`].
    pub bvh_parts: Option<(Vec<og_render::geometry::BvhNode>, Vec<u32>)>,
    /// The triangle-mesh collider the cook baked over [`Self::mesh`]`(0)`
    /// (`model { collider true }`): og-physics' `bake_trimesh` bytes, for
    /// `PhysicsSystem::set_baked_trimesh`. `None`: none was baked.
    pub collider: Option<Vec<u8>>,
}

impl CookedPrimitive {
    /// The cooked raycast BVH over `mesh` (LOD 0's, as [`Self::mesh`]
    /// answers it), without building anything; `None` when none was cooked.
    pub fn bvh(&self, mesh: &MeshData) -> Option<Result<og_render::geometry::Bvh, String>> {
        let (nodes, tris) = self.bvh_parts.as_ref()?;
        Some(og_render::geometry::Bvh::from_parts(
            mesh,
            nodes.clone(),
            tris.clone(),
        ))
    }

    /// The geometry of one LOD (0 is the full mesh) for [`RenderScene::add_mesh`].
    pub fn mesh(&self, lod: usize) -> MeshData {
        let l = self.lods[lod.min(self.lods.len() - 1)];
        MeshData {
            vertices: self.vertices.clone(),
            indices: self.indices[l.first..l.first + l.count].to_vec(),
        }
    }

    /// Every level with its clusters and indices, for
    /// [`RenderScene::set_mesh_clusters`] on [`Self::mesh`]`(0)`'s mesh;
    /// `None` when a level has none.
    pub fn mesh_clusters(&self) -> Option<og_render::meshlet::MeshClusters> {
        let levels = self
            .lods
            .iter()
            .map(|l| {
                (l.cluster_count > 0).then(|| og_render::meshlet::ClusterLevel {
                    first: l.first as u32,
                    count: l.count as u32,
                    error: l.error,
                    clusters: l.cluster_first as u32..(l.cluster_first + l.cluster_count) as u32,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        Some(og_render::meshlet::MeshClusters {
            levels,
            clusters: self.clusters.clone(),
            indices: self.indices.clone(),
        })
    }

    pub fn triangle_count(&self, lod: usize) -> usize {
        self.lods[lod.min(self.lods.len() - 1)].count / 3
    }

    /// The geometry of one LOD with only the vertices it uses (a coarse
    /// level's own mesh: [`CookedModel::add_lod_levels`]).
    pub fn mesh_compact(&self, lod: usize) -> MeshData {
        let l = self.lods[lod.min(self.lods.len() - 1)];
        let mut at = vec![u32::MAX; self.vertices.len()];
        let mut vertices = Vec::new();
        let indices = self.indices[l.first..l.first + l.count]
            .iter()
            .map(|&i| {
                if at[i as usize] == u32::MAX {
                    at[i as usize] = vertices.len() as u32;
                    vertices.push(self.vertices[i as usize]);
                }
                at[i as usize]
            })
            .collect();
        MeshData { vertices, indices }
    }
}

/// A glTF mesh: a range of [`CookedModel::primitives`].
#[derive(Clone, Debug, PartialEq)]
pub struct CookedMesh {
    pub name: String,
    pub primitives: std::ops::Range<usize>,
}

/// A material with its textures as indices into [`CookedModel::textures`].
#[derive(Clone, Debug, PartialEq)]
pub struct CookedMaterial {
    pub name: String,
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub normal_scale: f32,
    pub occlusion_strength: f32,
    pub emissive: [f32; 3],
    pub alpha_mode: u32,
    pub alpha_cutoff: f32,
    pub double_sided: bool,
    pub base_color_texture: Option<usize>,
    pub metallic_roughness_texture: Option<usize>,
    pub normal_texture: Option<usize>,
    pub emissive_texture: Option<usize>,
    pub occlusion_texture: Option<usize>,
    /// The `KHR_materials_*` physical extensions (`present` 0 without any).
    pub physical: PhysicalExtensions<usize>,
    /// `KHR_materials_unlit`: drawn unlit (three.js MeshBasicMaterial).
    pub unlit: bool,
    /// Each texture slot's `KHR_texture_transform` (og-render's slot
    /// order, [`og_render::StandardMaterial::uv_transforms`]): identity
    /// where none, or where the cook baked it into the primitives' uvs.
    pub uv_transforms: [og_render::UvTransform; 5],
}

impl CookedMaterial {
    /// Whether it draws as a standard material rather than in its glTF
    /// shape: physical extensions, unlit, or a texture transform to carry.
    pub fn needs_standard(&self) -> bool {
        self.physical.present != 0
            || self.unlit
            || self.uv_transforms.iter().any(|t| !t.is_identity())
    }
}

/// A `KHR_lights_punctual` light on a node of the model.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedLight {
    pub name: String,
    /// Its node in [`CookedModel::nodes`]; it shines along the node's -Z.
    pub node: usize,
    /// [`crate::format::light`].
    pub kind: u32,
    pub color: [f32; 3],
    /// glTF's: candela (point, spot) or lux (directional).
    pub intensity: f32,
    /// Where it reaches zero; 0: glTF's infinite range.
    pub range: f32,
    /// Spot cone half-angles (radians): inner, outer.
    pub cone: [f32; 2],
}

/// A texture a model uses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureRef {
    pub id: String,
    pub kind: u32,
}

/// A node of the model's scene graph.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedNode {
    pub name: String,
    pub parent: Option<usize>,
    pub mesh: Option<usize>,
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    /// Shown at rest: false when the glTF hides it (`KHR_node_visibility`),
    /// which hides its descendants too.
    pub visible: bool,
}

impl CookedNode {
    pub fn local(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

/// A glTF scene: its root nodes.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedScene {
    pub name: String,
    pub roots: Vec<usize>,
}

/// A decoded model (one glTF file).
#[derive(Clone, Debug, PartialEq)]
pub struct CookedModel {
    pub id: String,
    /// In glTF mesh-primitive order (`GltfInfo.geometries`).
    pub primitives: Vec<CookedPrimitive>,
    pub meshes: Vec<CookedMesh>,
    /// In glTF order, then the default material.
    pub materials: Vec<CookedMaterial>,
    pub textures: Vec<TextureRef>,
    /// Parents precede children.
    pub nodes: Vec<CookedNode>,
    pub scenes: Vec<CookedScene>,
    pub default_scene: Option<usize>,
    /// Bounds of the default scene with node transforms applied.
    pub min: Vec3,
    pub max: Vec3,
    pub skipped_animations: u32,
    pub skipped_skins: u32,
    /// `KHR_lights_punctual`: spawned with the model (og-runtime).
    pub lights: Vec<CookedLight>,
    /// `EXT_mesh_gpu_instancing`: per node with instances, their
    /// transforms in the node's space (the node draws only these).
    pub instances: BTreeMap<usize, Vec<Mat4>>,
    /// `KHR_materials_variants`: the variants' names.
    pub variants: Vec<String>,
    /// Per variant, the primitives whose material it changes: (primitive,
    /// material).
    pub variant_materials: Vec<Vec<(usize, usize)>>,
    /// Its octahedral impostor (`model { impostor true }`).
    pub impostor: Option<CookedImpostor>,
}

/// A model's octahedral impostor ([`crate::format::ImpostorRow`]): drawn
/// as its last level far away (og-render's `impostor`).
#[derive(Clone, Debug, PartialEq)]
pub struct CookedImpostor {
    /// The atlases' `og.Texture` ids: albedo and coverage, normal and depth.
    pub albedo: String,
    pub normal: String,
    pub grid: u32,
    pub frame: u32,
    pub center: Vec3,
    pub radius: f32,
    /// In the model's units, as a level's error.
    pub error: f32,
}

/// What [`CookedModel::add_to`] added: per part the mesh, the material and
/// the node's transform within the model, and the node drawing it.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelParts {
    pub parts: Vec<(MeshHandle, MaterialHandle, Mat4)>,
    /// Per part, its node's index in [`CookedModel::nodes`] (the
    /// `#anim` skeleton's node order: a posed model skips the parts of a
    /// node its pose hides).
    pub nodes: Vec<usize>,
    /// Per part, whether it is drawn at rest: false when its node or an
    /// ancestor is hidden ([`CookedNode::visible`]). A model drawn without
    /// a pose skips the others.
    pub shown: Vec<bool>,
    /// Per `KHR_materials_variants` variant, every part's material under
    /// it (the part's own where the variant does not map its primitive).
    pub variants: Vec<Vec<MaterialHandle>>,
    /// The model's textures in the scene, by [`CookedModel::textures`]
    /// index.
    pub textures: Vec<TextureHandle>,
}

impl ModelParts {
    /// Submits one instance of the whole model at `transform`.
    pub fn instance(&self, scene: &mut RenderScene, transform: Mat4) {
        for &(mesh, material, local) in &self.parts {
            scene.add_instance(mesh, material, transform * local);
        }
    }
}

impl CookedModel {
    pub(super) fn decode(id: &str, blob: &Blob) -> Result<CookedModel, LoadError> {
        let meta = one::<ModelMeta>(blob)?;
        let prims = table::<PrimitiveRow>(blob, "primitives")?;
        let lods = table::<LodRow>(blob, "lods")?;
        let cluster_rows = table::<ClusterRow>(blob, "clusters")?;
        let meshes = table::<MeshRow>(blob, "meshes")?;
        let materials = table::<MaterialRow>(blob, "materials")?;
        let textures = table::<TextureRefRow>(blob, "textures")?;
        let nodes = table::<NodeRow>(blob, "nodes")?;
        let scenes = table::<SceneRow>(blob, "scenes")?;
        let roots = blob.table::<u32>("roots", 0)?;
        let light_rows = table::<LightRow>(blob, "lights")?;
        let instance_rows = table::<InstanceRow>(blob, "instances")?;
        let variant_rows = table::<VariantRow>(blob, "variants")?;
        let mapping_rows = table::<VariantMappingRow>(blob, "variant_mappings")?;
        // Optional: a model cooked before impostors has no table.
        let impostor_rows = table::<ImpostorRow>(blob, "impostors").unwrap_or_default();
        let strings = blob.raw("strings")?;
        let vertices = blob.raw("vertices")?;
        let indices = blob.raw("indices")?;
        let bad = |r: String| invalid(blob, r);
        // The raycast BVHs: zlib-compressed rows, as many as the
        // primitives' ranges reach.
        let inflate = |name: &str, len: usize| -> Result<Vec<u8>, LoadError> {
            let z = blob.raw(name)?;
            let out = miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(z, len)
                .map_err(|e| bad(format!("{name}: {e:?}")))?;
            if out.len() != len {
                return Err(bad(format!("{name}: {} bytes, expected {len}", out.len())));
            }
            Ok(out)
        };
        let node_rows = prims
            .iter()
            .map(|p| (p.bvh_node_first + p.bvh_node_count) as usize)
            .max()
            .unwrap_or(0);
        let tri_rows = prims
            .iter()
            .map(|p| (p.bvh_tri_first + p.bvh_tri_count) as usize)
            .max()
            .unwrap_or(0);
        let node_bytes = inflate("bvh_nodes", node_rows * std::mem::size_of::<BvhNodeRow>())?;
        let tri_bytes = inflate("bvh_tris", tri_rows * 4)?;
        let bvh_nodes: Vec<BvhNodeRow> = bytemuck::pod_collect_to_vec(&node_bytes);
        let bvh_tris: Vec<u32> = bytemuck::pod_collect_to_vec(&tri_bytes);
        // Baked colliders (optional): the count, each one's offset and
        // length after the header, then the bytes.
        let colliders: Vec<Option<Vec<u8>>> = match blob.raw("colliders") {
            Err(_) => Vec::new(),
            Ok(c) => {
                let word = |i: usize| -> Result<usize, LoadError> {
                    c.get(i * 4..i * 4 + 4)
                        .map(|b| u32::from_le_bytes(b.try_into().expect("4 bytes")) as usize)
                        .ok_or_else(|| bad("colliders: header out of range".into()))
                };
                let n = word(0)?;
                let data = 4 + n * 8;
                let mut out = Vec::with_capacity(n);
                for i in 0..n {
                    let (at, len) = (word(1 + 2 * i)?, word(2 + 2 * i)?);
                    out.push(match len {
                        0 => None,
                        _ => Some(
                            c.get(data + at..data + at + len)
                                .ok_or_else(|| bad("colliders: bytes out of range".into()))?
                                .to_vec(),
                        ),
                    });
                }
                out
            }
        };
        let s = |o: u32, l: u32| {
            pool_str(strings, o, l)
                .map(str::to_string)
                .ok_or_else(|| bad("string out of range".into()))
        };
        let range = |data: &'_ [u8],
                     o: u32,
                     l: u32,
                     what: &str|
         -> Result<std::ops::Range<usize>, LoadError> {
            let (o, l) = (o as usize, l as usize);
            (o + l <= data.len())
                .then_some(o..o + l)
                .ok_or_else(|| bad(format!("{what} out of range")))
        };
        let mut primitives = Vec::with_capacity(prims.len());
        for (i, p) in prims.iter().enumerate() {
            let vb = &vertices[range(vertices, p.vertex_offset, p.vertex_size, "vertex stream")?];
            let ib = &indices[range(indices, p.index_offset, p.index_size, "index stream")?];
            let raw = meshopt_codec::decode_vertex_buffer(
                vb,
                p.vertex_count as usize,
                p.vertex_stride as usize,
            )
            .map_err(|e| bad(format!("primitive {i}: {e}")))?;
            let idx = meshopt_codec::decode_index_buffer(ib, p.index_count as usize)
                .map_err(|e| bad(format!("primitive {i}: {e}")))?;
            if idx.iter().any(|&v| v >= p.vertex_count) {
                return Err(bad(format!("primitive {i}: index out of range")));
            }
            let (verts, tangents) = dequantize(p, &raw);
            let lod_rows = lods
                .get(p.lod_first as usize..(p.lod_first + p.lod_count) as usize)
                .ok_or_else(|| bad(format!("primitive {i}: LOD range out of range")))?;
            let mut prim_lods = Vec::new();
            let mut prim_clusters = Vec::new();
            for l in lod_rows {
                if (l.index_first + l.index_count) as usize > idx.len() || l.index_count % 3 != 0 {
                    return Err(bad(format!("primitive {i}: LOD indices out of range")));
                }
                if (l.cluster_first + l.cluster_count) as usize > cluster_rows.len() {
                    return Err(bad(format!("primitive {i}: clusters out of range")));
                }
                prim_lods.push(Lod {
                    first: l.index_first as usize,
                    count: l.index_count as usize,
                    error: l.error,
                    cluster_first: prim_clusters.len(),
                    cluster_count: l.cluster_count as usize,
                });
                prim_clusters.extend(
                    cluster_rows
                        [l.cluster_first as usize..(l.cluster_first + l.cluster_count) as usize]
                        .iter()
                        .map(|c| og_render::meshlet::Cluster {
                            first: c.index_first,
                            triangles: c.triangle_count,
                            center: Vec3::new(c.center_x, c.center_y, c.center_z),
                            radius: c.radius,
                            cone_axis: Vec3::new(c.cone_x, c.cone_y, c.cone_z),
                            cone_cutoff: c.cone_cutoff,
                            error: c.error,
                            parent_error: c.parent_error,
                            error_center: Vec3::new(
                                c.error_center_x,
                                c.error_center_y,
                                c.error_center_z,
                            ),
                            error_radius: c.error_radius,
                            group: c.group,
                        }),
                );
            }
            if prim_lods.is_empty() {
                return Err(bad(format!("primitive {i} has no LOD")));
            }
            if p.material as usize >= materials.len() {
                return Err(bad(format!("primitive {i}: material out of range")));
            }
            let min = Vec3::new(p.pos_offset_x, p.pos_offset_y, p.pos_offset_z);
            let bvh_parts = if p.bvh_node_count == 0 {
                None
            } else {
                let nodes = bvh_nodes
                    .get(p.bvh_node_first as usize..(p.bvh_node_first + p.bvh_node_count) as usize)
                    .ok_or_else(|| bad(format!("primitive {i}: BVH nodes out of range")))?;
                let tris = bvh_tris
                    .get(p.bvh_tri_first as usize..(p.bvh_tri_first + p.bvh_tri_count) as usize)
                    .ok_or_else(|| bad(format!("primitive {i}: BVH triangles out of range")))?;
                let off = [p.pos_offset_x, p.pos_offset_y, p.pos_offset_z];
                let scale = [p.pos_scale_x, p.pos_scale_y, p.pos_scale_z];
                // As `dequantize`: the same grid, the same arithmetic.
                let deq =
                    |a: usize, q: u32| off[a] + (q & 0xffff) as u16 as f32 / 65535.0 * scale[a];
                let nodes = nodes
                    .iter()
                    .map(|n| og_render::geometry::BvhNode {
                        min: Vec3::new(
                            deq(0, n.min_xy),
                            deq(1, n.min_xy >> 16),
                            deq(2, n.min_z_max_x),
                        ),
                        max: Vec3::new(
                            deq(0, n.min_z_max_x >> 16),
                            deq(1, n.max_yz),
                            deq(2, n.max_yz >> 16),
                        ),
                        first: n.first,
                        count: n.count,
                    })
                    .collect();
                Some((nodes, tris.to_vec()))
            };
            primitives.push(CookedPrimitive {
                bvh_parts,
                collider: colliders.get(i).cloned().flatten(),
                material: p.material as usize,
                vertices: verts,
                tangents,
                indices: idx,
                lods: prim_lods,
                clusters: prim_clusters,
                min,
                max: min + Vec3::new(p.pos_scale_x, p.pos_scale_y, p.pos_scale_z),
            });
        }
        let tex = |i: u32| -> Result<Option<usize>, LoadError> {
            match opt(i) {
                Some(t) if t >= textures.len() => Err(bad("texture index out of range".into())),
                t => Ok(t),
            }
        };
        let mut mats = Vec::with_capacity(materials.len());
        for m in &materials {
            mats.push(CookedMaterial {
                name: s(m.name_offset, m.name_len)?,
                base_color: [m.base_r, m.base_g, m.base_b, m.base_a],
                metallic: m.metallic,
                roughness: m.roughness,
                normal_scale: m.normal_scale,
                occlusion_strength: m.occlusion_strength,
                emissive: [m.emissive_r, m.emissive_g, m.emissive_b],
                alpha_mode: m.alpha_mode,
                alpha_cutoff: m.alpha_cutoff,
                double_sided: m.double_sided != 0,
                base_color_texture: tex(m.base_color_texture)?,
                metallic_roughness_texture: tex(m.metallic_roughness_texture)?,
                normal_texture: tex(m.normal_texture)?,
                emissive_texture: tex(m.emissive_texture)?,
                occlusion_texture: tex(m.occlusion_texture)?,
                physical: PhysicalExtensions {
                    present: m.physical,
                    clearcoat: m.clearcoat,
                    clearcoat_texture: tex(m.clearcoat_texture)?,
                    clearcoat_roughness: m.clearcoat_roughness,
                    clearcoat_roughness_texture: tex(m.clearcoat_roughness_texture)?,
                    clearcoat_normal_texture: tex(m.clearcoat_normal_texture)?,
                    clearcoat_normal_scale: m.clearcoat_normal_scale,
                    sheen_color: [m.sheen_r, m.sheen_g, m.sheen_b],
                    sheen_color_texture: tex(m.sheen_color_texture)?,
                    sheen_roughness: m.sheen_roughness,
                    sheen_roughness_texture: tex(m.sheen_roughness_texture)?,
                    transmission: m.transmission,
                    transmission_texture: tex(m.transmission_texture)?,
                    thickness: m.thickness,
                    thickness_texture: tex(m.thickness_texture)?,
                    attenuation_distance: m.attenuation_distance,
                    attenuation_color: [m.attenuation_r, m.attenuation_g, m.attenuation_b],
                    ior: m.ior,
                    iridescence: m.iridescence,
                    iridescence_texture: tex(m.iridescence_texture)?,
                    iridescence_ior: m.iridescence_ior,
                    iridescence_thickness: [m.iridescence_min, m.iridescence_max],
                    iridescence_thickness_texture: tex(m.iridescence_thickness_texture)?,
                    anisotropy: m.anisotropy,
                    anisotropy_rotation: m.anisotropy_rotation,
                    anisotropy_texture: tex(m.anisotropy_texture)?,
                },
                unlit: m.unlit != 0,
                uv_transforms: {
                    let t = |o: [f32; 2], sc: [f32; 2], r: f32| og_render::UvTransform {
                        offset: glam::Vec2::from(o),
                        scale: glam::Vec2::from(sc),
                        rotation: r,
                    };
                    [
                        t(
                            [m.map_offset_x, m.map_offset_y],
                            [m.map_scale_x, m.map_scale_y],
                            m.map_rotation,
                        ),
                        t(
                            [m.mr_offset_x, m.mr_offset_y],
                            [m.mr_scale_x, m.mr_scale_y],
                            m.mr_rotation,
                        ),
                        t(
                            [m.normal_offset_x, m.normal_offset_y],
                            [m.normal_scale_x, m.normal_scale_y],
                            m.normal_rotation,
                        ),
                        t(
                            [m.emissive_offset_x, m.emissive_offset_y],
                            [m.emissive_scale_x, m.emissive_scale_y],
                            m.emissive_rotation,
                        ),
                        t(
                            [m.occlusion_offset_x, m.occlusion_offset_y],
                            [m.occlusion_scale_x, m.occlusion_scale_y],
                            m.occlusion_rotation,
                        ),
                    ]
                },
            });
        }
        let mut model_nodes = Vec::with_capacity(nodes.len());
        for (i, n) in nodes.iter().enumerate() {
            let parent = opt(n.parent);
            if parent.is_some_and(|p| p >= i) {
                return Err(bad(format!("node {i}: parent does not precede it")));
            }
            let mesh = opt(n.mesh);
            if mesh.is_some_and(|m| m >= meshes.len()) {
                return Err(bad(format!("node {i}: mesh out of range")));
            }
            model_nodes.push(CookedNode {
                name: s(n.name_offset, n.name_len)?,
                parent,
                mesh,
                translation: Vec3::new(n.tx, n.ty, n.tz),
                rotation: Quat::from_xyzw(n.rx, n.ry, n.rz, n.rw),
                scale: Vec3::new(n.sx, n.sy, n.sz),
                visible: n.visible != 0,
            });
        }
        let mut model_meshes = Vec::with_capacity(meshes.len());
        for m in &meshes {
            let r = m.primitive_first as usize..(m.primitive_first + m.primitive_count) as usize;
            if r.end > primitives.len() {
                return Err(bad("mesh primitive range out of range".into()));
            }
            model_meshes.push(CookedMesh {
                name: s(m.name_offset, m.name_len)?,
                primitives: r,
            });
        }
        let mut model_scenes = Vec::with_capacity(scenes.len());
        for sc in &scenes {
            let r = sc.root_first as usize..(sc.root_first + sc.root_count) as usize;
            let rs = roots
                .get(r)
                .ok_or_else(|| bad("scene roots out of range".into()))?;
            if rs.iter().any(|&n| n as usize >= model_nodes.len()) {
                return Err(bad("scene root out of range".into()));
            }
            model_scenes.push(CookedScene {
                name: s(sc.name_offset, sc.name_len)?,
                roots: rs.iter().map(|&n| n as usize).collect(),
            });
        }
        let mut texture_refs = Vec::with_capacity(textures.len());
        for t in &textures {
            texture_refs.push(TextureRef {
                id: s(t.id_offset, t.id_len)?,
                kind: t.kind,
            });
        }
        let mut lights = Vec::with_capacity(light_rows.len());
        for l in &light_rows {
            if l.node as usize >= model_nodes.len() {
                return Err(bad("light node out of range".into()));
            }
            lights.push(CookedLight {
                name: s(l.name_offset, l.name_len)?,
                node: l.node as usize,
                kind: l.kind,
                color: [l.r, l.g, l.b],
                intensity: l.intensity,
                range: l.range,
                cone: [l.inner_cone, l.outer_cone],
            });
        }
        let mut instances: BTreeMap<usize, Vec<Mat4>> = BTreeMap::new();
        for i in &instance_rows {
            if i.node as usize >= model_nodes.len() {
                return Err(bad("instance node out of range".into()));
            }
            instances.entry(i.node as usize).or_default().push(
                Mat4::from_scale_rotation_translation(
                    Vec3::new(i.sx, i.sy, i.sz),
                    Quat::from_xyzw(i.rx, i.ry, i.rz, i.rw),
                    Vec3::new(i.tx, i.ty, i.tz),
                ),
            );
        }
        let mut variants = Vec::with_capacity(variant_rows.len());
        for v in &variant_rows {
            variants.push(s(v.name_offset, v.name_len)?);
        }
        let mut variant_materials = vec![Vec::new(); variants.len()];
        for m in &mapping_rows {
            let (p, v, mat) = (
                m.primitive as usize,
                m.variant as usize,
                m.material as usize,
            );
            if p >= primitives.len() || v >= variants.len() || mat >= mats.len() {
                return Err(bad("variant mapping out of range".into()));
            }
            variant_materials[v].push((p, mat));
        }
        let impostor = match impostor_rows.first() {
            None => None,
            Some(i) => Some(CookedImpostor {
                albedo: s(i.albedo_offset, i.albedo_len)?,
                normal: s(i.normal_offset, i.normal_len)?,
                grid: i.grid,
                frame: i.frame,
                center: Vec3::new(i.center_x, i.center_y, i.center_z),
                radius: i.radius,
                error: i.error,
            }),
        };
        Ok(CookedModel {
            impostor,
            lights,
            instances,
            variants,
            variant_materials,
            id: id.to_string(),
            primitives,
            meshes: model_meshes,
            materials: mats,
            textures: texture_refs,
            nodes: model_nodes,
            scenes: model_scenes,
            default_scene: opt(meta.default_scene),
            min: Vec3::new(meta.min_x, meta.min_y, meta.min_z),
            max: Vec3::new(meta.max_x, meta.max_y, meta.max_z),
            skipped_animations: meta.skipped_animations,
            skipped_skins: meta.skipped_skins,
        })
    }

    /// Where node `n` draws its mesh, in model space (`world` from
    /// [`CookedModel::node_world`]): at each of its `EXT_mesh_gpu_instancing`
    /// instances, or once at its own transform.
    pub fn node_draws(&self, world: &[Mat4], n: usize) -> Vec<Mat4> {
        match self.instances.get(&n) {
            Some(i) => i.iter().map(|m| world[n] * *m).collect(),
            None => vec![world[n]],
        }
    }

    /// The index of the variant named `name`.
    pub fn variant(&self, name: &str) -> Option<usize> {
        self.variants.iter().position(|v| v == name)
    }

    /// Primitive `p`'s material under `variant` (`None`: its own).
    pub fn primitive_material(&self, p: usize, variant: Option<usize>) -> usize {
        variant
            .and_then(|v| self.variant_materials.get(v))
            .and_then(|m| m.iter().find(|(q, _)| *q == p))
            .map_or(self.primitives[p].material, |&(_, m)| m)
    }

    /// Every node's model-space transform.
    pub fn node_world(&self) -> Vec<Mat4> {
        let mut world: Vec<Mat4> = Vec::with_capacity(self.nodes.len());
        for n in &self.nodes {
            let parent = n.parent.map_or(Mat4::IDENTITY, |p| world[p]);
            world.push(parent * n.local());
        }
        world
    }

    /// Per node, whether it is drawn at rest: it and every ancestor visible.
    pub fn node_shown(&self) -> Vec<bool> {
        let mut shown: Vec<bool> = Vec::with_capacity(self.nodes.len());
        for n in &self.nodes {
            shown.push(n.visible && n.parent.is_none_or(|p| shown[p]));
        }
        shown
    }

    /// The nodes of `scene` (all descendants of its roots) in depth-first order.
    pub fn scene_nodes(&self, scene: usize) -> Vec<usize> {
        let mut children: Vec<Vec<usize>> = vec![Vec::new(); self.nodes.len()];
        for (i, n) in self.nodes.iter().enumerate() {
            if let Some(p) = n.parent {
                children[p].push(i);
            }
        }
        let mut out = Vec::new();
        let mut stack: Vec<usize> = self
            .scenes
            .get(scene)
            .map(|s| s.roots.iter().rev().copied().collect())
            .unwrap_or_default();
        while let Some(n) = stack.pop() {
            out.push(n);
            stack.extend(children[n].iter().rev());
        }
        out
    }

    /// Material `index` as the renderer draws it, sampling `textures` (the
    /// handles of [`CookedModel::textures`], in order; a missing one is
    /// drawn without its texture). A streamed model's full material over
    /// its proxy's.
    pub fn material(&self, index: usize, textures: &[TextureHandle]) -> Material {
        let m = &self.materials[index];
        let tex = |i: Option<usize>| i.and_then(|i| textures.get(i).copied());
        Material {
            base_color: m.base_color,
            base_color_texture: tex(m.base_color_texture),
            metallic: m.metallic,
            roughness: m.roughness,
            metallic_roughness_texture: tex(m.metallic_roughness_texture),
            normal_texture: tex(m.normal_texture),
            normal_scale: m.normal_scale,
            emissive: m.emissive,
            emissive_texture: tex(m.emissive_texture),
            alpha_mode: if m.alpha_mode == alpha::MASK {
                AlphaMode::Mask {
                    cutoff: m.alpha_cutoff,
                }
            } else {
                AlphaMode::Opaque
            },
        }
    }

    /// Material `index` with its physical extensions, as a standard
    /// material: what [`CookedModel::add_to`] adds for a material that has
    /// any (and the same as [`CookedModel::material`] for one without).
    pub fn standard_material(&self, index: usize, textures: &[TextureHandle]) -> StandardMaterial {
        let mut m = StandardMaterial::from(&self.material(index, textures));
        let c = &self.materials[index];
        c.physical.map(|i| textures.get(i).copied()).apply(&mut m);
        m.unlit = c.unlit as u32;
        m.set_uv_transforms(c.uv_transforms);
        m
    }

    /// Adds the model's textures, materials and LOD-0 meshes to `scene` and
    /// returns the parts of its default scene (every mesh node's primitives
    /// with the node's transform). `textures` are the decoded
    /// [`CookedModel::textures`], in order.
    pub fn add_to(&self, scene: &mut RenderScene, textures: &[TextureData]) -> ModelParts {
        let sources: Vec<_> = textures.iter().cloned().map(TextureSource::Rgba).collect();
        self.add_to_with(scene, &sources)
    }

    /// Adds every primitive's coarser levels (1 on) to the scene as meshes
    /// of their own, after whatever [`CookedModel::add_to`] added: per
    /// primitive, each level's mesh and its error, the largest distance from
    /// level 0 in the model's units (empty for a primitive of one level).
    pub fn add_lod_levels(&self, scene: &mut RenderScene) -> Vec<Vec<(MeshHandle, f32)>> {
        self.primitives
            .iter()
            .map(|p| {
                (1..p.lods.len())
                    .map(|l| (scene.add_mesh(p.mesh_compact(l)), p.lods[l].error))
                    .collect()
            })
            .collect()
    }

    /// [`CookedModel::add_lod_levels`] for a model added with
    /// [`CookedModel::add_to_shared`]: a primitive whose mesh came from a
    /// model added before (`meshes`, its answer) takes that mesh's levels, so
    /// the shared copies batch at every level too.
    pub fn add_lod_levels_shared(
        &self,
        scene: &mut RenderScene,
        meshes: &[MeshHandle],
        shared: &mut SharedParts,
    ) -> Vec<Vec<(MeshHandle, f32)>> {
        self.primitives
            .iter()
            .zip(meshes)
            .map(|(p, mesh)| {
                shared
                    .lods
                    .entry(*mesh)
                    .or_insert_with(|| {
                        (1..p.lods.len())
                            .map(|l| (scene.add_mesh(p.mesh_compact(l)), p.lods[l].error))
                            .collect()
                    })
                    .clone()
            })
            .collect()
    }

    /// [`CookedModel::add_to`] with textures of either kind (transcoded to
    /// the device's format: [`CookedTexture::to_texture_source`]).
    pub fn add_to_with(&self, scene: &mut RenderScene, textures: &[TextureSource]) -> ModelParts {
        self.add_to_shared(scene, textures, &[], None).0
    }

    /// [`CookedModel::add_to_with`], sharing with the models added before
    /// through `shared`: a texture of the same key (`texture_keys`, in
    /// texture order: its cooked bytes' hash), a material of the same name
    /// and value, and a mesh of the same vertices and indices reuse that
    /// model's handle, so copies of one tree in many models draw in one
    /// instanced batch and upload once. Also answers each primitive's mesh
    /// (shared ones are not the consecutive handles a model adds alone).
    pub fn add_to_shared(
        &self,
        scene: &mut RenderScene,
        textures: &[TextureSource],
        texture_keys: &[u64],
        mut shared: Option<&mut SharedParts>,
    ) -> (ModelParts, Vec<MeshHandle>) {
        let handles: Vec<_> = textures
            .iter()
            .enumerate()
            .map(
                |(i, t)| match (shared.as_deref_mut(), texture_keys.get(i)) {
                    (Some(s), Some(&key)) => *s
                        .textures
                        .entry(key)
                        .or_insert_with(|| scene.add_texture_source(t.clone())),
                    _ => scene.add_texture_source(t.clone()),
                },
            )
            .collect();
        // Pixel art (an art contract's `ArtTexels.filter nearest`) is
        // sampled nearest, as og.load.texture samples it.
        for (h, t) in handles.iter().zip(&self.textures) {
            if t.kind == crate::format::kind::PIXEL {
                scene.set_texture_filter(*h, og_render::TextureFilter::Nearest);
            }
        }
        // A material with physical extensions, unlit or a texture
        // transform is added as the standard material it draws as, the
        // others in their glTF shape.
        let materials: Vec<MaterialHandle> = (0..self.materials.len())
            .map(|i| {
                let value = match self.materials[i].needs_standard() {
                    false => SharedMaterial::Gltf(self.material(i, &handles)),
                    true => SharedMaterial::Standard(self.standard_material(i, &handles)),
                };
                let name = &self.materials[i].name;
                if let Some(s) = shared.as_deref_mut()
                    && let Some((_, h)) = s
                        .materials
                        .get(name)
                        .and_then(|l| l.iter().find(|(v, _)| *v == value))
                {
                    return *h;
                }
                let h = match &value {
                    SharedMaterial::Gltf(m) => scene.add_material(m.clone()),
                    SharedMaterial::Standard(m) => scene.add_standard_material(*m),
                };
                if let Some(s) = shared.as_deref_mut() {
                    s.materials
                        .entry(name.clone())
                        .or_default()
                        .push((value, h));
                }
                h
            })
            .collect();
        let meshes: Vec<MeshHandle> = self
            .primitives
            .iter()
            .map(|p| {
                let mesh = p.mesh(0);
                let Some(s) = shared
                    .as_deref_mut()
                    .filter(|_| mesh.vertices.len() <= SHARED_MESH_VERTICES)
                else {
                    return scene.add_mesh(mesh);
                };
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                bytemuck::cast_slice::<_, u8>(&mesh.vertices).hash(&mut hasher);
                mesh.indices.hash(&mut hasher);
                let key = hasher.finish();
                let same = s.meshes.get(&key).and_then(|hs| {
                    hs.iter()
                        .copied()
                        .find(|h| scene.mesh(*h).is_some_and(|m| *m == mesh))
                });
                same.unwrap_or_else(|| {
                    let h = scene.add_mesh(mesh);
                    s.meshes.entry(key).or_default().push(h);
                    h
                })
            })
            .collect();
        // Each mesh's levels as clusters, for the meshlet path (og-render's
        // `meshlet`): its instances' levels are then picked on the GPU.
        for (p, &m) in self.primitives.iter().zip(&meshes) {
            if let Some(c) = p.mesh_clusters() {
                scene.set_mesh_clusters(m, Some(c));
            }
        }
        let world = self.node_world();
        let node_shown = self.node_shown();
        let mut parts = Vec::new();
        let mut nodes = Vec::new();
        let mut shown = Vec::new();
        let mut variants = vec![Vec::new(); self.variants.len()];
        if let Some(s) = self.default_scene {
            for n in self.scene_nodes(s) {
                if let Some(m) = self.nodes[n].mesh {
                    // An instanced node: one part per instance, all of one
                    // mesh and material (the renderer draws them instanced).
                    for at in self.node_draws(&world, n) {
                        for p in self.meshes[m].primitives.clone() {
                            parts.push((meshes[p], materials[self.primitives[p].material], at));
                            nodes.push(n);
                            shown.push(node_shown[n]);
                            for (v, out) in variants.iter_mut().enumerate() {
                                out.push(materials[self.primitive_material(p, Some(v))]);
                            }
                        }
                    }
                }
            }
        }
        (
            ModelParts {
                parts,
                nodes,
                shown,
                variants,
                textures: handles,
            },
            meshes,
        )
    }
}

/// Meshes bigger than this are not compared ([`SharedParts`]): one of a kind.
const SHARED_MESH_VERTICES: usize = 65_536;

/// What models added with [`CookedModel::add_to_shared`] share: textures by
/// key, materials by name and value, meshes by vertices and indices, and a
/// shared mesh's coarser levels.
#[derive(Default)]
pub struct SharedParts {
    textures: std::collections::HashMap<u64, TextureHandle>,
    materials: std::collections::HashMap<String, Vec<(SharedMaterial, MaterialHandle)>>,
    meshes: std::collections::HashMap<u64, Vec<MeshHandle>>,
    lods: std::collections::HashMap<MeshHandle, Vec<(MeshHandle, f32)>>,
}

impl SharedParts {
    /// Whether a texture of this key is already in the scene (its source
    /// then need not be decoded again).
    pub fn has_texture(&self, key: u64) -> bool {
        self.textures.contains_key(&key)
    }
}

/// A material as models compare it: short-lived, built once per material.
#[allow(clippy::large_enum_variant)]
#[derive(PartialEq)]
enum SharedMaterial {
    Gltf(Material),
    Standard(StandardMaterial),
}

fn oct_decode(x: f32, y: f32) -> Vec3 {
    let mut v = Vec3::new(x, y, 1.0 - x.abs() - y.abs());
    if v.z < 0.0 {
        let (ox, oy) = (v.x, v.y);
        v.x = (1.0 - oy.abs()) * ox.signum();
        v.y = (1.0 - ox.abs()) * oy.signum();
    }
    v.normalize_or_zero()
}

fn dequantize(p: &PrimitiveRow, raw: &[u8]) -> (Vec<Vertex>, Vec<[f32; 4]>) {
    let stride = p.vertex_stride as usize;
    let off = [p.pos_offset_x, p.pos_offset_y, p.pos_offset_z];
    let scale = [p.pos_scale_x, p.pos_scale_y, p.pos_scale_z];
    let u16_at = |v: &[u8], at: usize| u16::from_le_bytes([v[at], v[at + 1]]);
    let i16_at = |v: &[u8], at: usize| i16::from_le_bytes([v[at], v[at + 1]]);
    let snorm = |q: i16| (q as f32 / 32767.0).max(-1.0);
    let mut verts = Vec::with_capacity(p.vertex_count as usize);
    let mut tangents = Vec::new();
    for v in raw.chunks_exact(stride) {
        let mut position = [0.0f32; 3];
        for a in 0..3 {
            position[a] = off[a] + u16_at(v, a * 2) as f32 / 65535.0 * scale[a];
        }
        let normal = oct_decode(snorm(i16_at(v, 8)), snorm(i16_at(v, 10))).to_array();
        let mut at = 12;
        let mut uv = [0.0f32; 2];
        if p.attributes & attr::UV != 0 {
            uv = [
                p.uv_offset_x + u16_at(v, at) as f32 / 65535.0 * p.uv_scale_x,
                p.uv_offset_y + u16_at(v, at + 2) as f32 / 65535.0 * p.uv_scale_y,
            ];
            at += 4;
        }
        let mut color = [1.0f32; 4];
        if p.attributes & attr::COLOR != 0 {
            for (c, out) in color.iter_mut().enumerate() {
                *out = v[at + c] as f32 / 255.0;
            }
            at += 4;
        }
        // The cook's MikkTSpace tangent, or none (the renderer generates one).
        let mut tangent = [0.0f32; 4];
        if p.attributes & attr::TANGENT != 0 {
            let t = oct_decode(snorm(i16_at(v, at)), snorm(i16_at(v, at + 2)));
            let w = if i16_at(v, at + 4) < 0 { -1.0 } else { 1.0 };
            tangent = [t.x, t.y, t.z, w];
            tangents.push(tangent);
        }
        verts.push(Vertex {
            tangent,
            position,
            normal,
            uv,
            color,
        });
    }
    (verts, tangents)
}
