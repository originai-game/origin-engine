//! Lightmap UVs: xatlas (third_party/xatlas) charts every static mesh in
//! world space and packs the charts into one or more square-ish pages.
//!
//! xatlas splits vertices along chart seams, so each mesh comes back with
//! its own vertices ([`AtlasVertex::xref`]: the input vertex each copies)
//! and triangles over them. The texel density is world units: a metre of
//! surface gets `texels_per_unit` texels whatever the mesh, so a wall and a
//! cup are lit at one resolution.

use std::os::raw::{c_char, c_void};

/// One mesh to chart: world-space positions, normals (seams follow hard
/// normals) and triangles.
#[derive(Clone, Debug, Default)]
pub struct UnwrapMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// How the charts are packed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PackSettings {
    /// Texels per world unit (metre) of surface.
    pub texels_per_unit: f32,
    /// The largest page side; more pages are made when the charts need them.
    pub max_size: u32,
    /// Empty texels around each chart (bilinear filtering and dilation).
    pub padding: u32,
}

impl Default for PackSettings {
    fn default() -> Self {
        PackSettings {
            texels_per_unit: 16.0,
            max_size: 1024,
            padding: 2,
        }
    }
}

/// One output vertex: the input vertex it copies, its page and its uv in
/// texels of that page.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtlasVertex {
    pub xref: u32,
    pub page: u32,
    pub uv: [f32; 2],
}

/// One charted mesh.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AtlasMesh {
    pub vertices: Vec<AtlasVertex>,
    pub indices: Vec<u32>,
}

/// The packed atlas: every page `width` × `height` texels.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Atlas {
    pub width: u32,
    pub height: u32,
    pub pages: u32,
    pub meshes: Vec<AtlasMesh>,
}

#[repr(C)]
struct XChart {
    face_array: *mut u32,
    atlas_index: u32,
    face_count: u32,
    ty: u32,
    material: u32,
}

#[repr(C)]
struct XVertex {
    atlas_index: i32,
    chart_index: i32,
    uv: [f32; 2],
    xref: u32,
}

#[repr(C)]
struct XMesh {
    chart_array: *mut XChart,
    index_array: *mut u32,
    vertex_array: *mut XVertex,
    chart_count: u32,
    index_count: u32,
    vertex_count: u32,
}

#[repr(C)]
struct XAtlas {
    image: *mut u32,
    meshes: *mut XMesh,
    utilization: *mut f32,
    width: u32,
    height: u32,
    atlas_count: u32,
    chart_count: u32,
    mesh_count: u32,
    texels_per_unit: f32,
}

#[repr(C)]
struct XMeshDecl {
    vertex_position_data: *const c_void,
    vertex_normal_data: *const c_void,
    vertex_uv_data: *const c_void,
    index_data: *const c_void,
    face_ignore_data: *const bool,
    face_material_data: *const u32,
    face_vertex_count: *const u8,
    vertex_count: u32,
    vertex_position_stride: u32,
    vertex_normal_stride: u32,
    vertex_uv_stride: u32,
    index_count: u32,
    index_offset: i32,
    face_count: u32,
    index_format: u32,
    epsilon: f32,
}

#[repr(C)]
struct XChartOptions {
    param_func: *const c_void,
    max_chart_area: f32,
    max_boundary_length: f32,
    normal_deviation_weight: f32,
    roundness_weight: f32,
    straightness_weight: f32,
    normal_seam_weight: f32,
    texture_seam_weight: f32,
    max_cost: f32,
    max_iterations: u32,
    use_input_mesh_uvs: bool,
    fix_winding: bool,
}

#[repr(C)]
struct XPackOptions {
    max_chart_size: u32,
    padding: u32,
    texels_per_unit: f32,
    resolution: u32,
    bilinear: bool,
    block_align: bool,
    brute_force: bool,
    create_image: bool,
    rotate_charts_to_axis: bool,
    rotate_charts: bool,
}

const INDEX_FORMAT_UINT32: u32 = 1;

unsafe extern "C" {
    fn xatlasCreate() -> *mut XAtlas;
    fn xatlasDestroy(atlas: *mut XAtlas);
    fn xatlasAddMesh(atlas: *mut XAtlas, decl: *const XMeshDecl, hint: u32) -> u32;
    fn xatlasAddMeshErrorString(error: u32) -> *const c_char;
    fn xatlasGenerate(atlas: *mut XAtlas, chart: *const XChartOptions, pack: *const XPackOptions);
    fn xatlasMeshDeclInit(decl: *mut XMeshDecl);
    fn xatlasChartOptionsInit(options: *mut XChartOptions);
    fn xatlasPackOptionsInit(options: *mut XPackOptions);
}

/// Charts `meshes` and packs them at `settings`' density: one page sized to
/// fit when the charts fit in `max_size`, else pages of `max_size`.
pub fn unwrap(meshes: &[UnwrapMesh], settings: &PackSettings) -> Result<Atlas, String> {
    if meshes.is_empty() {
        return Ok(Atlas::default());
    }
    let first = generate(meshes, settings, 0)?;
    if first.width <= settings.max_size && first.height <= settings.max_size && first.pages <= 1 {
        return Ok(first);
    }
    generate(meshes, settings, settings.max_size)
}

fn generate(
    meshes: &[UnwrapMesh],
    settings: &PackSettings,
    resolution: u32,
) -> Result<Atlas, String> {
    for (i, m) in meshes.iter().enumerate() {
        if m.indices.len() % 3 != 0 || m.indices.is_empty() {
            return Err(format!(
                "mesh {i}: {} indices is not a list of triangles",
                m.indices.len()
            ));
        }
        if !m.normals.is_empty() && m.normals.len() != m.positions.len() {
            return Err(format!(
                "mesh {i}: {} normals for {} positions",
                m.normals.len(),
                m.positions.len()
            ));
        }
        if let Some(&bad) = m.indices.iter().find(|&&x| x as usize >= m.positions.len()) {
            return Err(format!(
                "mesh {i}: index {bad} past its {} vertices",
                m.positions.len()
            ));
        }
    }
    // SAFETY: the declarations point into `meshes`, which outlives the atlas;
    // xatlas copies what it needs in xatlasAddMesh and every output array is
    // read before xatlasDestroy.
    unsafe {
        let atlas = xatlasCreate();
        if atlas.is_null() {
            return Err("xatlas: out of memory".into());
        }
        for (i, m) in meshes.iter().enumerate() {
            let mut decl = std::mem::zeroed::<XMeshDecl>();
            xatlasMeshDeclInit(&mut decl);
            decl.vertex_position_data = m.positions.as_ptr().cast();
            decl.vertex_position_stride = 12;
            decl.vertex_count = m.positions.len() as u32;
            if !m.normals.is_empty() {
                decl.vertex_normal_data = m.normals.as_ptr().cast();
                decl.vertex_normal_stride = 12;
            }
            decl.index_data = m.indices.as_ptr().cast();
            decl.index_count = m.indices.len() as u32;
            decl.index_format = INDEX_FORMAT_UINT32;
            let err = xatlasAddMesh(atlas, &decl, meshes.len() as u32);
            if err != 0 {
                let why = std::ffi::CStr::from_ptr(xatlasAddMeshErrorString(err))
                    .to_string_lossy()
                    .into_owned();
                xatlasDestroy(atlas);
                return Err(format!("xatlas: mesh {i}: {why}"));
            }
        }
        let mut chart = std::mem::zeroed::<XChartOptions>();
        xatlasChartOptionsInit(&mut chart);
        let mut pack = std::mem::zeroed::<XPackOptions>();
        xatlasPackOptionsInit(&mut pack);
        pack.padding = settings.padding;
        pack.texels_per_unit = settings.texels_per_unit.max(1e-3);
        pack.resolution = resolution;
        pack.bilinear = true;
        pack.block_align = false;
        pack.create_image = false;
        xatlasGenerate(atlas, &chart, &pack);
        let a = &*atlas;
        let mut out = Atlas {
            width: a.width,
            height: a.height,
            pages: a.atlas_count,
            meshes: Vec::with_capacity(a.mesh_count as usize),
        };
        for mi in 0..a.mesh_count as usize {
            let m = &*a.meshes.add(mi);
            let verts = std::slice::from_raw_parts(m.vertex_array, m.vertex_count as usize);
            let indices = std::slice::from_raw_parts(m.index_array, m.index_count as usize);
            out.meshes.push(AtlasMesh {
                vertices: verts
                    .iter()
                    .map(|v| AtlasVertex {
                        xref: v.xref,
                        page: v.atlas_index.max(0) as u32,
                        uv: v.uv,
                    })
                    .collect(),
                indices: indices.to_vec(),
            });
        }
        xatlasDestroy(atlas);
        if out.meshes.len() != meshes.len() {
            return Err(format!(
                "xatlas: {} meshes out of {}",
                out.meshes.len(),
                meshes.len()
            ));
        }
        if out.pages == 0 || out.width == 0 || out.height == 0 {
            return Err("xatlas: the charts did not pack (degenerate meshes?)".into());
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A unit cube with hard edges: 24 vertices, 12 triangles.
    pub(crate) fn cube(size: f32) -> UnwrapMesh {
        let mut m = UnwrapMesh::default();
        let faces: [([f32; 3], [f32; 3], [f32; 3]); 6] = [
            ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]),
            ([-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]),
            ([0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]),
            ([0.0, -1.0, 0.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0]),
            ([0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
            ([0.0, 0.0, -1.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
        ];
        for (n, u, v) in faces {
            let base = m.positions.len() as u32;
            for (a, b) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                let p = [0, 1, 2].map(|k| (n[k] + u[k] * a + v[k] * b) * size * 0.5);
                m.positions.push(p);
                m.normals.push(n);
            }
            m.indices
                .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        m
    }

    #[test]
    fn charts_keep_every_triangle_and_stay_in_the_page() {
        let meshes = [cube(1.0), cube(2.0)];
        let a = unwrap(&meshes, &PackSettings::default()).unwrap();
        assert_eq!(a.meshes.len(), 2);
        for (m, src) in a.meshes.iter().zip(&meshes) {
            assert_eq!(m.indices.len(), src.indices.len(), "every triangle kept");
            for v in &m.vertices {
                assert!((v.xref as usize) < src.positions.len());
                assert!(v.uv[0] >= 0.0 && v.uv[0] <= a.width as f32);
                assert!(v.uv[1] >= 0.0 && v.uv[1] <= a.height as f32);
            }
            // Each output triangle copies an input triangle's corners.
            for (t, s) in m.indices.chunks(3).zip(src.indices.chunks(3)) {
                let got: Vec<u32> = t.iter().map(|&i| m.vertices[i as usize].xref).collect();
                assert_eq!(got, s.to_vec());
            }
        }
        // 16 texels a metre: a 2 m cube's faces need about 32 x 32 texels each.
        assert!(
            a.width >= 32 && a.height >= 32,
            "{} x {}",
            a.width,
            a.height
        );
    }

    #[test]
    fn the_same_input_packs_the_same_way() {
        let meshes = [cube(1.0), cube(3.0)];
        let a = unwrap(&meshes, &PackSettings::default()).unwrap();
        let b = unwrap(&meshes, &PackSettings::default()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn charts_past_the_largest_page_spill_into_more_pages() {
        let meshes = [cube(8.0)];
        let s = PackSettings {
            texels_per_unit: 16.0,
            max_size: 128,
            padding: 2,
        };
        let a = unwrap(&meshes, &s).unwrap();
        assert_eq!((a.width, a.height), (128, 128));
        assert!(
            a.pages > 1,
            "six 128-texel faces need more than one 128² page"
        );
    }
}
