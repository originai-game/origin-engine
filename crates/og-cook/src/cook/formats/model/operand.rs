//! The imported source's rest geometry, using the same scene traversal and
//! placement rules as model bounds and the shared placed-merge mechanism.
use super::*;
use og_shape::generators::{Mesh, Mirror, Part, Placement};

pub(crate) fn geometry(src: &GltfSource, node: Option<&str>) -> Result<Mesh, String> {
    let instanced = ext::instances(src)?;
    let mut meshes = Vec::new();
    let mut error = None;
    blob::visit_scene(src, &instanced, |prim, ats, n| {
        if node.is_some_and(|name| n.name() != Some(name)) {
            return;
        }
        let read = prim.reader(|b| Some(&src.buffers[b.index()].0));
        let Some(positions) = read.read_positions() else {
            return;
        };
        let positions: Vec<_> = positions.collect();
        let normals = read.read_normals().map(Iterator::collect::<Vec<_>>);
        let uvs = read
            .read_tex_coords(0)
            .map(|v| v.into_f32().collect::<Vec<_>>());
        let colors = read
            .read_colors(0)
            .map(|v| v.into_rgba_f32().collect::<Vec<_>>());
        let tangents = read.read_tangents().map(Iterator::collect::<Vec<_>>);
        let indices = read
            .read_indices()
            .map(|v| v.into_u32().collect::<Vec<_>>());
        let data = og_render_data::geometry::buffer_geometry(
            &og_render_data::geometry::BufferAttributes {
                positions: bytemuck::cast_slice(&positions),
                normals: normals.as_deref().map(bytemuck::cast_slice),
                uvs: uvs.as_deref().map(bytemuck::cast_slice),
                colors: colors.as_deref().map(bytemuck::cast_slice),
                tangents: tangents.as_deref().map(bytemuck::cast_slice),
                indices: indices.as_deref(),
            },
        );
        let data = match data {
            Ok(v) => v,
            Err(e) => {
                error = Some(e);
                return;
            }
        };
        let mesh = Mesh {
            positions: data.vertices.iter().map(|v| v.position).collect(),
            normals: data.vertices.iter().map(|v| v.normal).collect(),
            uvs: data.vertices.iter().map(|v| v.uv).collect(),
            colors: data.vertices.iter().map(|v| v.color).collect(),
            tangents: tangents.unwrap_or_default(),
            indices: data.indices,
        };
        for at in ats {
            // Keep the full affine map: nested non-uniform scales can
            // shear, which decomposing to a TRS would lose.
            match og_shape::generators::place(&mesh, at.as_dmat4()) {
                Ok(p) => meshes.push(Mesh {
                    positions: p.positions.iter().map(|v| v.as_vec3().to_array()).collect(),
                    normals: p.normals.iter().map(|v| v.as_vec3().to_array()).collect(),
                    uvs: p.uvs.iter().map(|v| v.map(|v| v as f32)).collect(),
                    colors: p.colors.iter().map(|v| v.map(|v| v as f32)).collect(),
                    tangents: p.tangents.iter().map(|v| v.map(|v| v as f32)).collect(),
                    indices: p.indices,
                }),
                Err(e) => error = Some(e),
            }
        }
    });
    if let Some(e) = error {
        return Err(e);
    }
    if meshes.is_empty() {
        return Err(format!(
            "modelGeometry: no visible triangle geometry{}",
            node.map_or(String::new(), |n| format!(" at node '{n}'"))
        ));
    }
    let parts: Vec<_> = meshes
        .iter()
        .map(|mesh| Part {
            mesh,
            placement: Placement::default(),
            mirror: Mirror::default(),
        })
        .collect();
    og_shape::generators::merge(&parts)
}
