//! An FBX geometry read into a solid in the kernel's space: polygons triangulated, layers by mapping and reference.

use super::*;

/// The inverse of a signed permutation applied to `v`.
pub(crate) fn back(p: &impl Fn(DVec3) -> DVec3, v: DVec3) -> DVec3 {
    let mut out = DVec3::ZERO;
    for k in 0..3 {
        let mut e = DVec3::ZERO;
        e[k] = 1.0;
        out[k] = p(e).dot(v);
    }
    out
}

pub(crate) type Mesh = (
    Solid,
    Option<Vec<[DVec2; 3]>>,
    Option<Vec<og_shape::mesh::CornerColors>>,
    usize,
);

/// The space a skin was read into: what a clip needs to follow it. Bone
/// `b`'s skinning matrix when its model stands at `world` (kernel space)
/// is `out · world · unlink[b] · mesh[b]`.
pub(crate) struct FbxSpace {
    /// Each bone's model, in the skin's bone order.
    pub(crate) ids: Vec<i64>,
    /// Each bone's bind transform (`TransformLink`), inverted.
    pub(crate) unlink: Vec<Affine>,
    /// Each cluster's mesh bind transform (`Transform`); the identity
    /// where the vertices were placed by it already.
    pub(crate) mesh: Vec<Affine>,
    /// From the world back to where the solid's vertices stand.
    pub(crate) out: Affine,
}

pub(crate) fn mesh(
    g: &Node,
    name: &str,
    geo: &Affine,
    unit: f64,
    to_kernel: &impl Fn(DVec3) -> DVec3,
    mirrored: bool,
) -> Result<Mesh, ImportError> {
    let verts = g.array("Vertices").unwrap_or(&[]);
    let index = g.array("PolygonVertexIndex").unwrap_or(&[]);
    let mut solid = Solid::default();
    for v in verts.as_chunks::<3>().0 {
        let p = to_kernel(geo.apply(DVec3::new(v[0], v[1], v[2]))) * unit;
        if !p.is_finite() {
            return Err(ImportError::NotFinite(name.to_string()));
        }
        solid.push_vertex(p);
    }
    let n = solid.positions.len();
    // UVs: the first layer.
    let uv_layer = g.all("LayerElementUV").next();
    let uv_values = uv_layer.and_then(|l| l.array("UV"));
    let uv_index = uv_layer.and_then(|l| l.array("UVIndex"));
    let uv_mapping = uv_layer
        .and_then(|l| l.text("MappingInformationType"))
        .unwrap_or("");
    let uv_ref = uv_layer
        .and_then(|l| l.text("ReferenceInformationType"))
        .unwrap_or("");
    let uv_at = |corner: usize, vertex: usize| -> Option<DVec2> {
        let values = uv_values?;
        let k = match uv_mapping {
            "ByPolygonVertex" => corner,
            "ByVertice" | "ByVertex" | "ByControlPoint" => vertex,
            _ => return None,
        };
        let k = if uv_ref == "Direct" {
            k
        } else {
            *uv_index?.get(k)? as usize
        };
        Some(DVec2::new(*values.get(k * 2)?, *values.get(k * 2 + 1)?))
    };
    // Vertex colours: the first layer, RGBA as stored (taken as linear).
    let color_layer = g.all("LayerElementColor").next();
    let color_values = color_layer.and_then(|l| l.array("Colors"));
    let color_index = color_layer.and_then(|l| l.array("ColorIndex"));
    let color_mapping = color_layer
        .and_then(|l| l.text("MappingInformationType"))
        .unwrap_or("");
    let color_ref = color_layer
        .and_then(|l| l.text("ReferenceInformationType"))
        .unwrap_or("");
    let color_at = |corner: usize, vertex: usize| -> Option<[f64; 4]> {
        let values = color_values?;
        let k = match color_mapping {
            "ByPolygonVertex" => corner,
            "ByVertice" | "ByVertex" | "ByControlPoint" => vertex,
            _ => return None,
        };
        let k = if color_ref == "Direct" {
            k
        } else {
            *color_index?.get(k)? as usize
        };
        let c = values.get(k * 4..k * 4 + 4)?;
        c.iter()
            .all(|v| v.is_finite())
            .then(|| [c[0], c[1], c[2], c[3]])
    };
    let mut colors: Vec<og_shape::mesh::CornerColors> = Vec::new();
    let mut all_color = color_values.is_some();
    // Materials per polygon.
    let mat_layer = g.all("LayerElementMaterial").next();
    let mats = mat_layer.and_then(|l| l.array("Materials"));
    let by_polygon = mat_layer.and_then(|l| l.text("MappingInformationType")) == Some("ByPolygon");
    let mut uvs: Vec<[DVec2; 3]> = Vec::new();
    let mut all_uv = uv_values.is_some();
    let mut polys = 0;
    let mut corners: Vec<u32> = Vec::new();
    let mut corner_ids: Vec<usize> = Vec::new();
    let mut polygon = 0usize;
    for (c, &raw) in index.iter().enumerate() {
        let raw = raw as i64;
        let (v, last) = if raw < 0 {
            ((-raw - 1) as usize, true)
        } else {
            (raw as usize, false)
        };
        if v >= n {
            return Err(fmt_err(format!(
                "{name}: a polygon names a vertex that does not exist"
            )));
        }
        corners.push(v as u32);
        corner_ids.push(c);
        if !last {
            continue;
        }
        if corners.len() >= 3 {
            if corners.len() > 3 {
                polys += 1;
            }
            let surface = if by_polygon {
                mats.and_then(|m| m.get(polygon)).map_or(0, |&m| m as u32)
            } else {
                0
            };
            for t in triangulate(&solid.positions, &corners) {
                let k = t.map(|v| corners.iter().position(|&x| x == v).unwrap_or(0));
                let t = if mirrored { [t[0], t[2], t[1]] } else { t };
                let k = if mirrored { [k[0], k[2], k[1]] } else { k };
                solid.push_triangle(t, surface, &[name]);
                let uv = k.map(|j| uv_at(corner_ids[j], corners[j] as usize));
                if uv.iter().all(Option::is_some) {
                    uvs.push(uv.map(|u| u.unwrap_or(DVec2::ZERO)));
                } else {
                    all_uv = false;
                    uvs.push([DVec2::ZERO; 3]);
                }
                let color = k.map(|j| color_at(corner_ids[j], corners[j] as usize));
                if color.iter().all(Option::is_some) {
                    colors.push(
                        color.map(|c| c.map_or(og_shape::mesh::WHITE, og_shape::mesh::stored)),
                    );
                } else {
                    all_color = false;
                    colors.push([og_shape::mesh::WHITE; 3]);
                }
            }
        }
        corners.clear();
        corner_ids.clear();
        polygon += 1;
    }
    Ok((
        solid,
        all_uv.then_some(uvs),
        all_color.then_some(colors),
        polys,
    ))
}
