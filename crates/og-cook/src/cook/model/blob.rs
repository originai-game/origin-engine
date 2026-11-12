use super::*;

/// The cooked texture kind of what a glTF material's texture holds.
pub(super) fn texture_kind(k: og_render_data::gltf::TextureKind) -> u32 {
    match k {
        og_render_data::gltf::TextureKind::Color => kind::COLOR,
        og_render_data::gltf::TextureKind::Normal => kind::NORMAL,
        og_render_data::gltf::TextureKind::Linear => kind::LINEAR,
    }
}

/// A material row's `KHR_texture_transform` fields, per og-render slot.
fn set_uv(row: &mut MaterialRow, t: &[og_render_data::UvTransform; 5]) {
    let f = |t: &og_render_data::UvTransform| {
        (t.offset.x, t.offset.y, t.scale.x, t.scale.y, t.rotation)
    };
    (
        row.map_offset_x,
        row.map_offset_y,
        row.map_scale_x,
        row.map_scale_y,
        row.map_rotation,
    ) = f(&t[0]);
    (
        row.mr_offset_x,
        row.mr_offset_y,
        row.mr_scale_x,
        row.mr_scale_y,
        row.mr_rotation,
    ) = f(&t[1]);
    (
        row.normal_offset_x,
        row.normal_offset_y,
        row.normal_scale_x,
        row.normal_scale_y,
        row.normal_rotation,
    ) = f(&t[2]);
    (
        row.emissive_offset_x,
        row.emissive_offset_y,
        row.emissive_scale_x,
        row.emissive_scale_y,
        row.emissive_rotation,
    ) = f(&t[3]);
    (
        row.occlusion_offset_x,
        row.occlusion_offset_y,
        row.occlusion_scale_x,
        row.occlusion_scale_y,
        row.occlusion_rotation,
    ) = f(&t[4]);
}

/// What a model cook reports besides the blob.
pub struct ModelNotes {
    pub notes: Vec<String>,
    /// LOD 0 triangles and vertices of every primitive, summed.
    pub triangles: usize,
    pub vertices: usize,
}

/// Cooks a parsed glTF into a model blob. `texture_ids` are the asset ids
/// of [`GltfSource::textures`], in order. A `proxy` (a streaming group's
/// stand-in, `#proxy`) keeps each primitive's coarsest LOD and no textures.
/// `id` is the model's asset id: with `opts.impostor`, its impostor's
/// atlases are `<id>#impostor` and `<id>#impostor-normal` ([`impostor_ids`]).
pub fn cook(
    src: &GltfSource,
    texture_ids: &[String],
    opts: &MeshOptions,
    stamp: u64,
    proxy: bool,
    id: &str,
) -> Result<(Vec<u8>, ModelNotes), CookError> {
    let doc = &src.doc;
    let mut notes = src.warnings.clone();
    let mut strings = Strings(Vec::new());
    let plan = super::ext::uv_plan(src);
    notes.extend(plan.notes.iter().cloned());
    let tex_of = |t: Option<gltf::texture::Texture>, k: u32, cutout: Option<u8>| -> u32 {
        t.filter(|_| !proxy)
            .and_then(|t| {
                src.textures.iter().position(|u| {
                    u.image == t.source().index() && u.kind == k && u.cutout == cutout
                })
            })
            .map_or(NONE, |i| i as u32)
    };
    // An untextured style's colours are baked into the vertices
    // (`conform::mesh`): its materials draw white without a base texture.
    let baked = opts
        .conform
        .as_ref()
        .is_some_and(|c| c.bakes() && c.applies(src));
    let mut materials = Vec::new();
    let mut unlit = 0;
    for m in doc.materials() {
        let pbr = m.pbr_metallic_roughness();
        let e = m.emissive_factor();
        let strength = m.emissive_strength().unwrap_or(1.0);
        let (name_offset, name_len) = strings.add(m.name().unwrap_or(""));
        let mut b = pbr.base_color_factor();
        if baked {
            b = [1.0, 1.0, 1.0, b[3]];
        }
        let p = og_render_data::gltf::physical_extensions(doc, &m, |t, k| {
            tex_of(Some(t), texture_kind(k), None)
        });
        let t = |x: Option<u32>| x.unwrap_or(NONE);
        materials.push(MaterialRow {
            physical: p.present,
            clearcoat: p.clearcoat,
            clearcoat_roughness: p.clearcoat_roughness,
            clearcoat_normal_scale: p.clearcoat_normal_scale,
            sheen_r: p.sheen_color[0],
            sheen_g: p.sheen_color[1],
            sheen_b: p.sheen_color[2],
            sheen_roughness: p.sheen_roughness,
            transmission: p.transmission,
            thickness: p.thickness,
            attenuation_distance: p.attenuation_distance,
            attenuation_r: p.attenuation_color[0],
            attenuation_g: p.attenuation_color[1],
            attenuation_b: p.attenuation_color[2],
            ior: p.ior,
            iridescence: p.iridescence,
            iridescence_ior: p.iridescence_ior,
            iridescence_min: p.iridescence_thickness[0],
            iridescence_max: p.iridescence_thickness[1],
            anisotropy: p.anisotropy,
            anisotropy_rotation: p.anisotropy_rotation,
            clearcoat_texture: t(p.clearcoat_texture),
            clearcoat_roughness_texture: t(p.clearcoat_roughness_texture),
            clearcoat_normal_texture: t(p.clearcoat_normal_texture),
            sheen_color_texture: t(p.sheen_color_texture),
            sheen_roughness_texture: t(p.sheen_roughness_texture),
            transmission_texture: t(p.transmission_texture),
            thickness_texture: t(p.thickness_texture),
            iridescence_texture: t(p.iridescence_texture),
            iridescence_thickness_texture: t(p.iridescence_thickness_texture),
            anisotropy_texture: t(p.anisotropy_texture),
            base_r: b[0],
            base_g: b[1],
            base_b: b[2],
            base_a: b[3],
            emissive_r: e[0] * strength,
            emissive_g: e[1] * strength,
            emissive_b: e[2] * strength,
            metallic: pbr.metallic_factor(),
            roughness: pbr.roughness_factor(),
            normal_scale: m.normal_texture().map_or(1.0, |t| t.scale()),
            occlusion_strength: m.occlusion_texture().map_or(1.0, |t| t.strength()),
            alpha_mode: match m.alpha_mode() {
                gltf::material::AlphaMode::Opaque => alpha::OPAQUE,
                gltf::material::AlphaMode::Mask => alpha::MASK,
                gltf::material::AlphaMode::Blend => alpha::BLEND,
            },
            alpha_cutoff: m.alpha_cutoff().unwrap_or(0.5),
            double_sided: m.double_sided() as u32,
            base_color_texture: tex_of(
                pbr.base_color_texture()
                    .map(|t| t.texture())
                    .filter(|_| !baked),
                kind::COLOR,
                m.index()
                    .and_then(|i| src.cutouts.get(i).copied().flatten()),
            ),
            metallic_roughness_texture: tex_of(
                pbr.metallic_roughness_texture().map(|t| t.texture()),
                kind::LINEAR,
                None,
            ),
            normal_texture: tex_of(m.normal_texture().map(|t| t.texture()), kind::NORMAL, None),
            emissive_texture: tex_of(m.emissive_texture().map(|t| t.texture()), kind::COLOR, None),
            occlusion_texture: tex_of(
                m.occlusion_texture().map(|t| t.texture()),
                kind::LINEAR,
                None,
            ),
            name_offset,
            name_len,
            unlit: super::ext::unlit(&src.json, m.index().unwrap_or(usize::MAX)) as u32,
            ..Default::default()
        });
        let row = materials.last_mut().expect("pushed");
        unlit += row.unlit as usize;
        set_uv(row, &plan.materials[m.index().unwrap_or(0)]);
    }
    if unlit > 0 {
        notes.push(format!("KHR_materials_unlit: {unlit} unlit material(s)"));
    }
    // The default material (glTF's, matching og-render's gltf bridge).
    let default_material = materials.len() as u32;
    let (name_offset, name_len) = strings.add("default");
    let grey = if baked { 1.0 } else { 0.8 };
    materials.push(MaterialRow {
        base_r: grey,
        base_g: grey,
        base_b: grey,
        base_a: 1.0,
        roughness: 0.6,
        normal_scale: 1.0,
        occlusion_strength: 1.0,
        alpha_cutoff: 0.5,
        base_color_texture: NONE,
        metallic_roughness_texture: NONE,
        normal_texture: NONE,
        emissive_texture: NONE,
        occlusion_texture: NONE,
        clearcoat_texture: NONE,
        clearcoat_roughness_texture: NONE,
        clearcoat_normal_texture: NONE,
        sheen_color_texture: NONE,
        sheen_roughness_texture: NONE,
        transmission_texture: NONE,
        thickness_texture: NONE,
        iridescence_texture: NONE,
        iridescence_thickness_texture: NONE,
        anisotropy_texture: NONE,
        name_offset,
        name_len,
        ..Default::default()
    });
    set_uv(
        materials.last_mut().expect("pushed"),
        &[og_render_data::UvTransform::IDENTITY; 5],
    );

    let mut prims: Vec<PrimitiveRow> = Vec::new();
    let mut mappings: Vec<VariantMappingRow> = Vec::new();
    let mut lods: Vec<LodRow> = Vec::new();
    let mut clusters: Vec<ClusterRow> = Vec::new();
    let mut meshes: Vec<MeshRow> = Vec::new();
    let mut vertices: Vec<u8> = Vec::new();
    let mut indices: Vec<u8> = Vec::new();
    let mut cluster_indices: Vec<u8> = Vec::new();
    let mut foliage_rows: Vec<FoliageRow> = Vec::new();
    let mut foliage: Vec<u8> = Vec::new();
    let mut bvh_nodes: Vec<BvhNodeRow> = Vec::new();
    let mut bvh_tris: Vec<u32> = Vec::new();
    let lod_meshes = msft_lod(doc, &mut notes);
    let mut changes = Vec::new();
    let mut hulls: Vec<u8> = Vec::new();
    // Triangle primitives that cooked to no geometry (every triangle
    // degenerate, or none left by the art contract).
    let mut emptied: Vec<String> = Vec::new();
    for mesh in doc.meshes() {
        let first = prims.len() as u32;
        for (pi, prim) in mesh.primitives().enumerate() {
            let what = format!("mesh '{}' primitive {pi}", mesh.name().unwrap_or(""));
            if prim.mode() != gltf::mesh::Mode::Triangles {
                notes.push(format!(
                    "{what}: mode {:?} is not cooked (triangles only)",
                    prim.mode()
                ));
                continue;
            }
            let material = prim
                .material()
                .index()
                .map_or(default_material, |i| i as u32);
            let normal_mapped = !proxy && prim.material().normal_texture().is_some();
            // MSFT_lod: the same primitive of each coarser mesh is a level.
            let mut levels = vec![prim.clone()];
            for &m in lod_meshes.get(&mesh.index()).into_iter().flatten() {
                match doc.meshes().nth(m).and_then(|lm| lm.primitives().nth(pi)) {
                    Some(lp) if lp.mode() == gltf::mesh::Mode::Triangles => levels.push(lp),
                    _ => {
                        notes.push(format!(
                            "{what}: MSFT_lod mesh {m} has no triangle primitive {pi}; its levels stop there"
                        ));
                        break;
                    }
                }
            }
            let bake = plan.bake.get(&(mesh.index(), pi)).copied();
            match cook_primitive(src, &levels, material, normal_mapped, opts, proxy, bake)
                .map_err(|e| CookError(format!("{what}: {e}")))?
            {
                Some(mut c) => {
                    while !vertices.len().is_multiple_of(4) {
                        vertices.push(0);
                    }
                    while !indices.len().is_multiple_of(4) {
                        indices.push(0);
                    }
                    c.row.vertex_offset = vertices.len() as u32;
                    c.row.vertex_size = c.vertices.len() as u32;
                    c.row.index_offset = indices.len() as u32;
                    c.row.index_size = c.indices.len() as u32;
                    c.row.cluster_index_offset = cluster_indices.len() as u32;
                    c.row.cluster_index_size = c.cluster_indices.len() as u32;
                    c.row.lod_first = lods.len() as u32;
                    c.row.bvh_node_first = bvh_nodes.len() as u32;
                    c.row.bvh_node_count = c.bvh_nodes.len() as u32;
                    c.row.bvh_tri_first = bvh_tris.len() as u32;
                    c.row.bvh_tri_count = c.bvh_tris.len() as u32;
                    bvh_nodes.extend_from_slice(&c.bvh_nodes);
                    bvh_tris.extend_from_slice(&c.bvh_tris);
                    vertices.extend_from_slice(&c.vertices);
                    indices.extend_from_slice(&c.indices);
                    cluster_indices.extend_from_slice(&c.cluster_indices);
                    let cluster_base = clusters.len() as u32;
                    lods.extend(c.lods.iter().map(|l| LodRow {
                        cluster_first: l.cluster_first + cluster_base,
                        ..*l
                    }));
                    clusters.extend_from_slice(&c.clusters);
                    if let Some((f, leaves)) = c.foliage.take() {
                        let stream = super::primitive::encode_vertices(
                            bytemuck::cast_slice(&f),
                            f.len(),
                            32,
                        )
                        .map_err(|e| CookError(format!("{what}: foliage: {e}")))?;
                        while !foliage.len().is_multiple_of(4) {
                            foliage.push(0);
                        }
                        foliage_rows.push(FoliageRow {
                            primitive: prims.len() as u32,
                            stream_offset: foliage.len() as u32,
                            stream_size: stream.len() as u32,
                            flags: leaves as u32,
                        });
                        foliage.extend_from_slice(&stream);
                    }
                    changes.extend(c.conformed.take());
                    hulls.extend((c.hull.len() as u32).to_le_bytes());
                    hulls.extend(c.hull.iter().flatten().flat_map(|x| x.to_le_bytes()));
                    let json_prim = &src.json["meshes"][mesh.index()]["primitives"][pi];
                    for (material, variant) in super::ext::mappings(json_prim) {
                        mappings.push(VariantMappingRow {
                            primitive: prims.len() as u32,
                            variant: variant as u32,
                            material: material as u32,
                        });
                    }
                    prims.push(c.row);
                }
                None => {
                    notes.push(format!("{what}: no geometry"));
                    emptied.push(what);
                }
            }
        }
        let (name_offset, name_len) = strings.add(mesh.name().unwrap_or(""));
        meshes.push(MeshRow {
            primitive_first: first,
            primitive_count: prims.len() as u32 - first,
            name_offset,
            name_len,
        });
    }

    let NodeOrder { new_index, rows } = node_order(doc, &mut |s| strings.add(s))?;
    let nodes = rows;
    // Skinned mesh nodes' bind poses, by cooked node (format's NodeBindRow).
    let skin_binds: Vec<NodeBindRow> = super::skin_binds(src)
        .into_iter()
        .map(|(n, m)| {
            let (s, r, t) = m.to_scale_rotation_translation();
            NodeBindRow {
                node: new_index[n],
                tx: t.x,
                ty: t.y,
                tz: t.z,
                rx: r.x,
                ry: r.y,
                rz: r.z,
                rw: r.w,
                sx: s.x,
                sy: s.y,
                sz: s.z,
            }
        })
        .collect();
    // EXT_mesh_gpu_instancing: each instanced node's instances, in cooked node order.
    let instanced = super::ext::instances(src).map_err(CookError)?;
    let mut instances: Vec<InstanceRow> = Vec::new();
    let mut by_new: Vec<(u32, &Vec<Mat4>)> =
        instanced.iter().map(|(&n, m)| (new_index[n], m)).collect();
    by_new.sort_by_key(|(n, _)| *n);
    for (node, mats) in by_new {
        for m in mats {
            let (s, r, t) = m.to_scale_rotation_translation();
            instances.push(InstanceRow {
                node,
                tx: t.x,
                ty: t.y,
                tz: t.z,
                rx: r.x,
                ry: r.y,
                rz: r.z,
                rw: r.w,
                sx: s.x,
                sy: s.y,
                sz: s.z,
            });
        }
    }
    if !instances.is_empty() {
        notes.push(format!(
            "EXT_mesh_gpu_instancing: {} instance(s) of {} node(s), drawn as instances of one mesh",
            instances.len(),
            instanced.len()
        ));
    }
    // KHR_lights_punctual.
    let lights: Vec<LightRow> = super::ext::lights(&src.json, &mut notes)
        .into_iter()
        .map(|l| {
            let (name_offset, name_len) = strings.add(&l.name);
            LightRow {
                node: new_index[l.node],
                kind: l.kind,
                r: l.color[0],
                g: l.color[1],
                b: l.color[2],
                intensity: l.intensity,
                range: l.range,
                inner_cone: l.cone[0],
                outer_cone: l.cone[1],
                name_offset,
                name_len,
            }
        })
        .collect();
    if !lights.is_empty() {
        notes.push(format!(
            "KHR_lights_punctual: {} light(s), spawned with the model",
            lights.len()
        ));
    }
    // KHR_materials_variants.
    let variants: Vec<VariantRow> = super::ext::variants(&src.json)
        .iter()
        .map(|n| {
            let (name_offset, name_len) = strings.add(n);
            VariantRow {
                name_offset,
                name_len,
            }
        })
        .collect();
    mappings.retain(|m| {
        (m.variant as usize) < variants.len() && (m.material as usize) < default_material as usize
    });
    if !variants.is_empty() {
        notes.push(format!(
            "KHR_materials_variants: {} variant(s), {} primitive mapping(s)",
            variants.len(),
            mappings.len()
        ));
    }
    let mut scenes = Vec::new();
    let mut roots: Vec<u32> = Vec::new();
    for sc in doc.scenes() {
        let first = roots.len() as u32;
        roots.extend(sc.nodes().map(|n| new_index[n.index()]));
        let (name_offset, name_len) = strings.add(sc.name().unwrap_or(""));
        scenes.push(SceneRow {
            root_first: first,
            root_count: roots.len() as u32 - first,
            name_offset,
            name_len,
        });
    }
    let default_scene = doc
        .default_scene()
        .map(|s| s.index() as u32)
        .unwrap_or(if scenes.is_empty() { NONE } else { 0 });

    let (lo, hi) = scene_bounds(src, &instanced);
    let skipped_animations = doc.animations().count() as u32;
    let skipped_skins = doc.skins().count() as u32;
    if skipped_animations + skipped_skins > 0 {
        notes.push(format!(
            "{skipped_animations} animation(s) and {skipped_skins} skin(s): in the #anim blob; this blob holds the bind pose"
        ));
    }
    let mut texture_refs = Vec::new();
    // `ArtTexels.filter nearest` cooks colour textures as pixel art
    // (`conform::mesh::texture`): the loader samples them nearest.
    let nearest = opts
        .conform
        .as_ref()
        .is_some_and(|c| c.filter == super::conform::mesh::Filter::Nearest);
    for (i, u) in src.textures.iter().enumerate().filter(|_| !proxy) {
        let (id_offset, id_len) = strings.add(&texture_ids[i]);
        texture_refs.push(TextureRefRow {
            id_offset,
            id_len,
            kind: if nearest && u.kind == kind::COLOR {
                kind::PIXEL
            } else {
                u.kind
            },
        });
    }
    // The impostor: its atlases' ids and the sphere they cover (the
    // bounds', as og-runtime's levels use), its error an atlas texel.
    let mut impostors = Vec::new();
    let mut impostor_materials = Vec::new();
    if let Some(b) = opts.impostor.filter(|_| !proxy).map(|b| b.resolved(src)) {
        let [albedo, normal, material] = impostor_ids(id);
        let (material_offset, material_len) = strings.add(&material);
        impostor_materials.push(ImpostorMaterialRow {
            material_offset,
            material_len,
        });
        let (albedo_offset, albedo_len) = strings.add(&albedo);
        let (normal_offset, normal_len) = strings.add(&normal);
        let center = (lo + hi) * 0.5;
        let radius = ((hi - lo) * 0.5).length().max(1e-3);
        impostors.push(ImpostorRow {
            albedo_offset,
            albedo_len,
            normal_offset,
            normal_len,
            grid: b.grid,
            frame: b.frame,
            center_x: center.x,
            center_y: center.y,
            center_z: center.z,
            radius,
            error: 2.0 * radius / b.frame as f32,
        });
        notes.push(format!(
            "impostor: {g}×{g} frames of {f} px ({s}×{s} atlas), one texel {e:.3} units",
            g = b.grid,
            f = b.frame,
            s = b.grid * b.frame,
            e = 2.0 * radius / b.frame as f32
        ));
    }
    let meta = ModelMeta {
        primitive_count: prims.len() as u32,
        mesh_count: meshes.len() as u32,
        material_count: materials.len() as u32,
        texture_count: texture_refs.len() as u32,
        node_count: nodes.len() as u32,
        scene_count: scenes.len() as u32,
        default_scene,
        skipped_animations,
        skipped_skins,
        min_x: lo.x,
        min_y: lo.y,
        min_z: lo.z,
        max_x: hi.x,
        max_y: hi.y,
        max_z: hi.z,
    };
    let mut w = BlobWriter::new(
        format::Model::type_hash(),
        format::Model::schema_version(),
        stamp,
    );
    w.table("meta", &[meta], row_hash::<ModelMeta>())
        .table("primitives", &prims, row_hash::<PrimitiveRow>())
        .table("lods", &lods, row_hash::<LodRow>())
        .table("clusters", &clusters, row_hash::<ClusterRow>())
        .table("meshes", &meshes, row_hash::<MeshRow>())
        .table("materials", &materials, row_hash::<MaterialRow>())
        .table("textures", &texture_refs, row_hash::<TextureRefRow>())
        .table("nodes", &nodes, row_hash::<NodeRow>())
        .table("scenes", &scenes, row_hash::<SceneRow>())
        .table("roots", &roots, 0)
        .raw("strings", strings.0, 1)
        .raw("vertices", vertices, 16)
        .raw("indices", indices, 16)
        .raw("cluster_indices", cluster_indices, 16)
        .raw("bvh_nodes", zlib(bytemuck::cast_slice(&bvh_nodes)), 4)
        .raw("bvh_tris", zlib(bytemuck::cast_slice(&bvh_tris)), 4)
        .table("lights", &lights, row_hash::<LightRow>())
        .table("instances", &instances, row_hash::<InstanceRow>())
        .table("variants", &variants, row_hash::<VariantRow>())
        .table(
            "variant_mappings",
            &mappings,
            row_hash::<VariantMappingRow>(),
        )
        .table("impostors", &impostors, row_hash::<ImpostorRow>());
    // Only a model baked with the material atlas (G5) has the table.
    if !impostor_materials.is_empty() {
        w.table(
            "impostor_materials",
            &impostor_materials,
            row_hash::<ImpostorMaterialRow>(),
        );
    }
    if !skin_binds.is_empty() {
        w.table("skin_binds", &skin_binds, row_hash::<NodeBindRow>());
    }
    if !foliage_rows.is_empty() {
        w.table("foliage_rows", &foliage_rows, row_hash::<FoliageRow>())
            .raw("foliage", foliage, 16);
    }
    // The art contract's outline hull (`conform::mesh` gives the layout).
    if let Some(c) = &opts.conform
        && let Some(l) = &c.line
        && !changes.is_empty()
    {
        let mut section = super::conform::mesh::hull_header(l, prims.len());
        section.extend(hulls);
        w.raw("outline", section, 4);
    }
    // A model whose triangles all cooked away would load as one with no
    // mesh: refused here, where its cause is known, not at load.
    if prims.is_empty() && !emptied.is_empty() {
        return Err(CookError(format!(
            "nothing left to draw: {} cooked to no triangles (every triangle has no area{})",
            emptied.join(", "),
            if opts.conform.is_some() {
                ", or none is left by the art contract"
            } else {
                ""
            }
        )));
    }
    if let Some(c) = &opts.conform {
        notes.extend(super::conform::mesh::notes(c, src, &changes));
    }
    let bvh_bytes = w_len_of_bvh(&bvh_nodes, &bvh_tris);
    let tris: usize = prims
        .iter()
        .map(|p| lods[p.lod_first as usize].index_count as usize / 3)
        .sum();
    let tris_lod: Vec<usize> = (0..prims.iter().map(|p| p.lod_count).max().unwrap_or(1) as usize)
        .map(|l| {
            prims
                .iter()
                .map(|p| {
                    let k = (l as u32).min(p.lod_count - 1);
                    lods[(p.lod_first + k) as usize].index_count as usize / 3
                })
                .sum()
        })
        .collect();
    notes.push(format!(
        "{} primitives, {tris} triangles; LOD triangles {:?}; {} materials, {} textures, {} nodes",
        prims.len(),
        tris_lod,
        materials.len(),
        texture_refs.len(),
        nodes.len()
    ));
    notes.push(if opts.raycast {
        format!(
            "raycast BVH: {} nodes, {bvh_bytes} bytes (`model {{ raycast false }}` leaves it out)",
            bvh_nodes.len()
        )
    } else {
        "raycast false: no BVH cooked; the first og.render.raycast that reaches it builds one"
            .into()
    });
    let bytes = if opts.collider {
        let (section, baked) = colliders(&w.clone().finish())?;
        notes.push(format!(
            "collider: {baked} triangle mesh(es) baked, {} bytes (`model {{ collider false }}` leaves them out)",
            section.len()
        ));
        w.raw("colliders", section, 4);
        w.finish()
    } else {
        w.finish()
    };
    let vertices = prims.iter().map(|p| p.vertex_count as usize).sum();
    Ok((
        bytes,
        ModelNotes {
            notes,
            triangles: tris,
            vertices,
        },
    ))
}

/// The `colliders` section of a model blob written without one (`blob`),
/// and how many it baked: each primitive's triangle-mesh collider baked
/// over LOD 0 as the runtime decodes the blob (og-bundle's
/// `CookedPrimitive::mesh(0)`, the geometry physics registers), so
/// physics takes it where a build would make the same shape. The section
/// is the primitive count, each one's offset and length after the header
/// (0: none), then the bytes.
fn colliders(blob: &[u8]) -> Result<(Vec<u8>, usize), CookError> {
    let words = og_blob::AlignedBytes::from_slice(blob);
    let model = crate::runtime::Bundle::decode_model("model", words.as_slice())
        .map_err(|e| CookError(format!("collider: {e}")))?;
    let mut baked = Vec::new();
    for (i, p) in model.primitives.iter().enumerate() {
        let m = p.mesh(0);
        let mesh = og_physics_data::MeshDesc {
            positions: m.vertices.iter().map(|v| Vec3::from(v.position)).collect(),
            indices: m.indices.chunks(3).map(|t| [t[0], t[1], t[2]]).collect(),
        };
        baked.push(
            og_physics_data::bake_trimesh(&mesh)
                .map_err(|e| CookError(format!("primitive {i}: collider: {}", e.reason())))?,
        );
    }
    let mut section = (baked.len() as u32).to_le_bytes().to_vec();
    let mut at = 0u32;
    for c in &baked {
        section.extend(at.to_le_bytes());
        section.extend((c.len() as u32).to_le_bytes());
        at += c.len() as u32;
    }
    for c in &baked {
        section.extend_from_slice(c);
    }
    Ok((section, baked.len()))
}

/// Bounds of the default scene's vertices with node transforms, from the
/// source positions: what og-render's glTF bridge computes, less the
/// nodes hidden at rest (`KHR_node_visibility` `visible: false`) and
/// what is under them, which the rest pose does not draw (an animated
/// `.vox`'s frames after 0). Zero for a scene with no vertices.
pub(super) fn scene_bounds(
    src: &GltfSource,
    instanced: &std::collections::BTreeMap<usize, Vec<Mat4>>,
) -> (Vec3, Vec3) {
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    let doc = &src.doc;
    // A skinned mesh's vertices are placed by its bind pose, not its node.
    let binds = super::skin_binds(src);
    if let Some(scene) = doc.default_scene().or_else(|| doc.scenes().next()) {
        let mut stack: Vec<(gltf::Node, Mat4)> =
            scene.nodes().map(|n| (n, Mat4::IDENTITY)).collect();
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
            // An instanced node draws its mesh at each instance only.
            let at: Vec<Mat4> = match (binds.get(&node.index()), instanced.get(&node.index())) {
                (Some(b), _) => vec![*b],
                (None, Some(i)) => i.iter().map(|m| world * *m).collect(),
                (None, None) => vec![world],
            };
            if let Some(mesh) = node.mesh() {
                for prim in mesh
                    .primitives()
                    .filter(|p| p.mode() == gltf::mesh::Mode::Triangles)
                {
                    let reader = prim.reader(|b| Some(&src.buffers[b.index()].0[..]));
                    if let Some(ps) = reader.read_positions() {
                        for p in ps {
                            for m in &at {
                                let w = m.transform_point3(Vec3::from(p));
                                lo = lo.min(w);
                                hi = hi.max(w);
                            }
                        }
                    }
                }
            }
            for c in node.children() {
                stack.push((c, world));
            }
        }
    }
    if lo.x > hi.x {
        (lo, hi) = (Vec3::ZERO, Vec3::ZERO);
    }
    (lo, hi)
}

/// The asset ids of model `id`'s impostor atlases: the albedo, then the
/// normal and depth.
pub fn impostor_ids(id: &str) -> [String; 3] {
    [
        format!("{id}#impostor"),
        format!("{id}#impostor-normal"),
        format!("{id}#impostor-material"),
    ]
}
