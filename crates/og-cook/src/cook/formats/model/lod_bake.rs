//! A reduced level's own parameterisation and surface transferred from
//! LOD0. The atlas xref remains the deformation source of every corner.

use super::{GltfSource, primitive::FVertex};
use crate::cook::texture::{self, Image};
use glam::{Mat3, Vec2, Vec3, Vec4};
use og_bake::transfer::{Mips, Pages, Source, Surface, Target, TransferSettings};

pub(super) struct BakedLevel {
    pub vertices: Vec<FVertex>,
    /// Vertex indices in the input stream, not charted vertex indices.
    pub sources: Vec<u32>,
    pub indices: Vec<u32>,
    pub pages: Pages,
    /// Texture footprint in model units, included in the LOD error.
    pub texel: f32,
}

/// Weld chart wedges before reducing geometry. The source corner of a
/// retained vertex is carried back, but its old UV is not used by the
/// reduced level. Deformation keys and foliage boundaries still apply.
pub(super) fn simplify_geometry(
    vertices: &[FVertex],
    indices: &[u32],
    target: usize,
    error: f32,
    locks: &[bool],
    normal_weight: f32,
) -> Vec<u32> {
    let mut by_position = std::collections::BTreeMap::new();
    let mut sources = Vec::new();
    let mut normals = Vec::<Vec3>::new();
    let mut locked = Vec::new();
    let mut mapped = Vec::with_capacity(indices.len());
    for &i in indices {
        let v = vertices[i as usize];
        let next = sources.len() as u32;
        let position = v.p.map(|x| (x + 0.0).to_bits());
        let at = *by_position.entry((position, v.key)).or_insert_with(|| {
            sources.push(i);
            normals.push(Vec3::ZERO);
            locked.push(false);
            next
        });
        normals[at as usize] += Vec3::from(v.n);
        locked[at as usize] |= locks.get(i as usize).copied().unwrap_or(false);
        mapped.push(at);
    }
    let positions: Vec<_> = sources.iter().map(|&i| vertices[i as usize].p).collect();
    let normals: Vec<_> = normals
        .iter()
        .map(|n| n.normalize_or(Vec3::Z).to_array())
        .collect();
    // Different deformation keys at the same geometric position must not
    // collapse together even though meshoptimizer remaps by position.
    let mut keys = std::collections::BTreeMap::new();
    let mut split = std::collections::BTreeSet::new();
    for &i in &sources {
        let v = vertices[i as usize];
        let p = v.p.map(|x| (x + 0.0).to_bits());
        if *keys.entry(p).or_insert(v.key) != v.key {
            split.insert(p);
        }
    }
    for (i, p) in positions.iter().enumerate() {
        locked[i] |= split.contains(&p.map(|x| (x + 0.0).to_bits()));
    }
    let adapter = meshopt::VertexDataAdapter::new(bytemuck::cast_slice(&positions), 12, 0)
        .expect("three floats per position");
    let reduced = meshopt::simplify_with_attributes_and_locks(
        &mapped,
        &adapter,
        bytemuck::cast_slice(&normals),
        &[normal_weight; 3],
        12,
        &locked,
        target,
        error,
        meshopt::SimplifyOptions::None,
        None,
    );
    // Shading-normal detail is transferred into the new normal map, not
    // discarded. If its attribute error pins geometry above budget, keep
    // the geometric error and deformation/boundary constraints, then bake
    // the original shading normal at every projected surface sample.
    let reduced = if reduced.len() > target {
        meshopt::simplify_with_locks(
            &mapped,
            &adapter,
            &locked,
            target,
            error,
            meshopt::SimplifyOptions::None,
            None,
        )
    } else {
        reduced
    };
    reduced.into_iter().map(|i| sources[i as usize]).collect()
}

/// Recharts a reduced mesh at its screen-space footprint. `full` is LOD0,
/// never another reduced mesh; `reduced` references the same input vertices.
pub(super) fn bake(
    src: &GltfSource,
    primitive: &gltf::Primitive,
    vertices: &[FVertex],
    full: &[u32],
    reduced: &[u32],
    error: f32,
    reuse_tangents: bool,
) -> Result<BakedLevel, String> {
    let positions: Vec<_> = vertices.iter().map(|v| Vec3::from(v.p)).collect();
    let normals: Vec<_> = vertices.iter().map(|v| Vec3::from(v.n)).collect();
    let material = primitive.material();
    let pbr = material.pbr_metallic_roughness();
    let plan = super::ext::uv_plan(src);
    let transforms = material.index().map(|i| plan.materials[i]);
    let textures = [
        pbr.base_color_texture().map(|t| t.texture()),
        pbr.metallic_roughness_texture().map(|t| t.texture()),
        material.normal_texture().map(|t| t.texture()),
        material.emissive_texture().map(|t| t.texture()),
        material.occlusion_texture().map(|t| t.texture()),
    ];
    let mut source_density = 0.0f64;
    for (slot, texture) in textures.iter().enumerate() {
        let Some(texture) = texture else { continue };
        let image = texture::decode(&src.images[texture.source().index()])?;
        let mut world_area = 0.0f64;
        let mut pixel_area = 0.0f64;
        for triangle in full.as_chunks::<3>().0 {
            let [a, b, c] = triangle.map(|i| positions[i as usize].as_dvec3());
            let area = (b - a).cross(c - a).length();
            if area == 0.0 {
                continue;
            }
            let [a, b, c] = triangle.map(|i| {
                let uv = vertices[i as usize].uv;
                let uv = transforms.map_or(uv, |t| t[slot].apply(uv));
                glam::DVec2::new(
                    uv[0] as f64 * image.width as f64,
                    uv[1] as f64 * image.height as f64,
                )
            });
            world_area += area;
            pixel_area += (b - a).perp_dot(c - a).abs();
        }
        if world_area > 0.0 {
            source_density = source_density.max((pixel_area / world_area).sqrt());
        }
    }
    // At the geometric switch distance, one model-space error covers
    // LOD_PIXEL_ERROR pixels. Never bake more texels per unit than that
    // projected footprint or than the source charts actually contain.
    let source_texel = if source_density > 0.0 {
        (1.0 / source_density) as f32
    } else {
        0.0
    };
    let texel = (error / og_render_data::lod::LOD_PIXEL_ERROR).max(source_texel);
    if texel <= 0.0 {
        return Err("LOD surface has neither geometric error nor source texel density".into());
    }
    let mut input_sources = Vec::new();
    let mut input_map = std::collections::BTreeMap::new();
    let indices = reduced
        .iter()
        .map(|&i| {
            let next = input_sources.len() as u32;
            *input_map.entry(i).or_insert_with(|| {
                input_sources.push(i);
                next
            })
        })
        .collect();
    let unwrap = og_bake::UnwrapMesh {
        positions: input_sources
            .iter()
            .map(|&i| vertices[i as usize].p)
            .collect(),
        // Authored shading seams are transferred through the normal map;
        // they must not become constraints on the new parameterisation.
        normals: Vec::new(),
        indices,
    };
    let atlas = og_bake::unwrap(
        std::slice::from_ref(&unwrap),
        &og_bake::PackSettings {
            texels_per_unit: 1.0 / texel,
            max_size: 4096,
            // xatlas already reserves each chart's bilinear ring.
            padding: 0,
            chart_iterations: 1,
        },
    )?;
    if atlas.pages != 1 {
        return Err(format!(
            "LOD atlas needs {} pages at its switch-distance density; refusing to undersample",
            atlas.pages,
        ));
    }
    let charted = &atlas.meshes[0];
    // Pad the page, not the charts: rounding must not change their density.
    let width = atlas.width.next_power_of_two();
    let height = atlas.height.next_power_of_two();
    let mut output: Vec<_> = charted
        .vertices
        .iter()
        .map(|v| {
            let mut source = vertices[input_sources[v.xref as usize] as usize];
            source.uv = [v.uv[0] / width as f32, v.uv[1] / height as f32];
            // The transferred base colour includes the source vertex colour.
            source.c = [1.0; 4];
            if !reuse_tangents {
                source.t = [0.0; 4];
            }
            source
        })
        .collect();
    let mut sources: Vec<_> = charted
        .vertices
        .iter()
        .map(|v| input_sources[v.xref as usize])
        .collect();
    // A normal map may be baked in any valid interpolated tangent frame.
    // Keeping the source frame lets a level share every source attribute
    // except atlas UV. Only sources without tangents need a new frame.
    sources = charted
        .indices
        .iter()
        .map(|&i| sources[i as usize])
        .collect();
    output = charted
        .indices
        .iter()
        .map(|&i| output[i as usize])
        .collect();
    if !reuse_tangents
        && !mikktspace::generate_tangents(&mut super::primitive::Tangents { v: &mut output })
    {
        return Err("LOD atlas has no valid MikkTSpace tangent frame".into());
    }
    let indices: Vec<_> = (0..output.len() as u32).collect();
    let (output, indices) = super::primitive::weld(&output, &indices, &mut sources);
    let target_positions: Vec<_> = output.iter().map(|v| Vec3::from(v.p)).collect();
    let target_normals: Vec<_> = output.iter().map(|v| Vec3::from(v.n)).collect();
    let target_tangents: Vec<_> = output.iter().map(|v| Vec4::from(v.t)).collect();
    let target_uvs: Vec<_> = output
        .iter()
        .map(|v| Vec2::new(v.uv[0] * width as f32, v.uv[1] * height as f32))
        .collect();

    let base = pbr.base_color_factor();
    let mut images = Vec::new();
    let mut image = |texture: Option<gltf::Texture>, srgb: bool| -> Result<Option<usize>, String> {
        let Some(texture) = texture else {
            return Ok(None);
        };
        let Image {
            width,
            height,
            rgba,
        } = texture::decode(&src.images[texture.source().index()])?;
        let texels = rgba
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| {
                let rgb = [0, 1, 2].map(|k| {
                    if srgb {
                        og_core::color::srgb8_to_linear(p[k])
                    } else {
                        p[k] as f32 / 255.0
                    }
                });
                [rgb[0], rgb[1], rgb[2], p[3] as f32 / 255.0]
            })
            .collect();
        images.push(Mips::new(og_bake::Image {
            width,
            height,
            texels,
        }));
        Ok(Some(images.len() - 1))
    };
    let mut surface = Surface {
        base_color: Vec3::new(base[0], base[1], base[2]),
        opacity: base[3],
        base_map: image(pbr.base_color_texture().map(|t| t.texture()), true)?,
        roughness: pbr.roughness_factor(),
        metallic: pbr.metallic_factor(),
        metallic_roughness_map: image(
            pbr.metallic_roughness_texture().map(|t| t.texture()),
            false,
        )?,
        normal_map: image(material.normal_texture().map(|t| t.texture()), false)?,
        normal_scale: material.normal_texture().map_or(1.0, |t| t.scale()),
        occlusion_map: image(material.occlusion_texture().map(|t| t.texture()), false)?,
        occlusion_strength: material.occlusion_texture().map_or(1.0, |t| t.strength()),
        emissive: Vec3::from(material.emissive_factor())
            * material.emissive_strength().unwrap_or(1.0),
        emissive_map: image(material.emissive_texture().map(|t| t.texture()), true)?,
        ..Default::default()
    };
    let plan = super::ext::uv_plan(src);
    if let Some(index) = material.index() {
        surface.uv_transforms = plan.materials[index].map(|t| {
            let origin = t.apply([0.0, 0.0]);
            let x = t.apply([1.0, 0.0]);
            let y = t.apply([0.0, 1.0]);
            Mat3::from_cols(
                Vec3::new(x[0] - origin[0], x[1] - origin[1], 0.0),
                Vec3::new(y[0] - origin[0], y[1] - origin[1], 0.0),
                Vec3::new(origin[0], origin[1], 1.0),
            )
        });
    }
    let source = og_bake::Mesh {
        positions,
        normals,
        tangents: vertices.iter().map(|v| Vec4::from(v.t)).collect(),
        uvs: vertices.iter().map(|v| Vec2::from(v.uv)).collect(),
        colors: vertices
            .iter()
            .map(|v| Vec3::new(v.c[0], v.c[1], v.c[2]))
            .collect(),
        opacity: vertices.iter().map(|v| v.c[3]).collect(),
        indices: full.to_vec(),
        ..Default::default()
    };
    let refs: Vec<_> = images.iter().collect();
    let pages = og_bake::transfer::transfer(
        Source {
            meshes: &[source],
            surfaces: &[surface],
            images: &refs,
        },
        Target {
            positions: &target_positions,
            normals: &target_normals,
            tangents: &target_tangents,
            uvs: &target_uvs,
            indices: &indices,
        },
        &TransferSettings {
            width,
            height,
            reach: error.max(texel) * 1.25,
            samples: 2,
            texel,
        },
    );
    Ok(BakedLevel {
        vertices: output,
        sources,
        indices,
        pages,
        texel,
    })
}
