//! The model's answers as JSON: frames and paths, measures, interference and the posed report.

use super::*;

/// A frame as JSON.
pub fn frame_json(f: &Frame) -> Value {
    let v = |d: DVec3| json!([d.x, d.y, d.z]);
    json!({ "origin": v(f.origin), "x": v(f.x), "y": v(f.y), "z": v(f.z) })
}

pub(crate) fn frames_json(fs: &[Frame]) -> Value {
    json!({ "frames": fs.iter().map(frame_json).collect::<Vec<_>>() })
}

pub(crate) fn path_json(p: &Path) -> Value {
    json!({
        "points": p.points.iter().map(|d| json!([d.x, d.y, d.z])).collect::<Vec<_>>(),
        "closed": p.closed,
    })
}

/// Measures of every part (`docs/design.md` §7) as JSON, in the model's
/// unit, with no thresholds.
pub fn measures_json(m: &Model) -> Result<String, ModelError> {
    Ok(measures_value(m)?.to_string())
}

/// [`measures_json`] with one field added, `interference`: per pose, the
/// rest first and then each of `poses` (joint values; every joint not
/// named at rest), the pairs of parts that pass through each other, touch,
/// or hold one wholly inside ([`Model::interference`]):
///
/// `{"pose": {joint: value}, "pairs": [{"a", "b", "crossings",
/// "touching", "inside" (a part's name or null), "boundsMin",
/// "boundsMax"}]}`
///
/// Counts only: no threshold says what is too much.
pub fn report_json(m: &Model, poses: &[Vec<(String, f64)>]) -> Result<String, ModelError> {
    let mut v = measures_value(m)?;
    let mut list = vec![interference_value(m, &[])?];
    for p in poses {
        list.push(interference_value(m, p)?);
    }
    if let Some(o) = v.as_object_mut() {
        o.insert("interference".into(), Value::Array(list));
    }
    Ok(v.to_string())
}

/// One pose's entry of [`report_json`]'s `interference`.
pub(crate) fn interference_value(m: &Model, pose: &[(String, f64)]) -> Result<Value, ModelError> {
    let v3 = |d: DVec3| json!([d.x, d.y, d.z]);
    let pairs: Vec<Value> = m
        .interference(pose)?
        .into_iter()
        .map(|x| {
            json!({
                "a": x.a,
                "b": x.b,
                "crossings": x.crossings,
                "touching": x.touching,
                "inside": if x.inside.is_empty() { Value::Null } else { Value::String(x.inside) },
                "boundsMin": v3(x.bounds_min),
                "boundsMax": v3(x.bounds_max),
            })
        })
        .collect();
    let pose: Map<String, Value> = pose.iter().map(|(k, x)| (k.clone(), json!(x))).collect();
    Ok(json!({ "pose": pose, "pairs": pairs }))
}

/// The model as posed (`docs/research.md` §20): [`measures_json`]'s
/// fields measured at the model's pose (each joint at its value), and
///
/// - `pose`: every joint that can be posed, with its value;
/// - `posed`: whether any value is not 0 (OBJ and STL are then written
///   posed);
/// - `joints`: per joint, at rest, `{"name", "child", "parent", "value",
///   "pivotToParent", "pivotToChild", "parentMoved", "parentTurned"}`
///   ([`Model::seating`]): how far the pivot lies from the parent's and
///   the child's surfaces (null for no triangles), and how far the parent
///   has moved and turned (radians) off the pivot since the joint was
///   declared;
/// - `interference`: the pairs at the pose, as [`report_json`] gives
///   them, each with `asBuilt`: true when both parts move by the very
///   same motion and were compared where built, false when compared
///   posed (flush faces under different joints can then read as
///   crossing by rounding).
///
/// Measurements only, with no thresholds.
pub fn posed_json(m: &Model) -> Result<String, ModelError> {
    Ok(posed_value(m)?.to_string())
}

pub(crate) fn posed_value(m: &Model) -> Result<Value, ModelError> {
    let pose = m.pose_values();
    let mut v = parts_value(m, &m.measure_posed()?);
    let v3 = |d: DVec3| json!([d.x, d.y, d.z]);
    let joints: Vec<Value> = m
        .seating()?
        .into_iter()
        .map(|j| {
            json!({
                "name": j.joint,
                "child": j.child,
                "parent": j.parent,
                "value": j.value,
                "pivotToParent": j.pivot_to_parent,
                "pivotToChild": j.pivot_to_child,
                "parentMoved": j.parent_moved,
                "parentTurned": j.parent_turned,
            })
        })
        .collect();
    let pairs: Vec<Value> = m
        .interference(&pose)?
        .into_iter()
        .map(|x| {
            json!({
                "a": x.a,
                "b": x.b,
                "crossings": x.crossings,
                "touching": x.touching,
                "inside": if x.inside.is_empty() { Value::Null } else { Value::String(x.inside) },
                "boundsMin": v3(x.bounds_min),
                "boundsMax": v3(x.bounds_max),
                "asBuilt": x.as_built,
            })
        })
        .collect();
    let values: Map<String, Value> = pose.iter().map(|(k, x)| (k.clone(), json!(x))).collect();
    if let Some(o) = v.as_object_mut() {
        o.insert("pose".into(), Value::Object(values));
        o.insert("posed".into(), Value::Bool(m.posed()));
        o.insert("joints".into(), Value::Array(joints));
        o.insert("interference".into(), Value::Array(pairs));
    }
    Ok(v)
}

pub(crate) fn measures_value(m: &Model) -> Result<Value, ModelError> {
    Ok(parts_value(m, &m.measure()?))
}

pub(crate) fn parts_value(m: &Model, parts: &[crate::model::PartMeasures]) -> Value {
    let v3 = |d: DVec3| json!([d.x, d.y, d.z]);
    let mut lo = DVec3::splat(f64::INFINITY);
    let mut hi = DVec3::splat(f64::NEG_INFINITY);
    let (mut tris, mut verts, mut area, mut volume) = (0usize, 0usize, 0.0, 0.0);
    let mut list = Vec::new();
    let (mut coverage, mut overlap): (Option<f64>, Option<f64>) = (None, None);
    for p in parts {
        let x = &p.measures;
        if x.vertices > 0 {
            lo = lo.min(x.bounds_min);
            hi = hi.max(x.bounds_max);
        }
        tris += x.triangles;
        verts += x.vertices;
        area += x.area;
        volume += x.volume;
        list.push(json!({
            "name": p.name,
            "triangles": x.triangles,
            "vertices": x.vertices,
            "boundsMin": v3(x.bounds_min),
            "boundsMax": v3(x.bounds_max),
            "area": x.area,
            "volume": x.volume,
            "closed": x.closed,
            "boundaryEdges": x.boundary_edges,
            "nonManifoldEdges": x.non_manifold_edges,
            "degenerateTriangles": x.degenerate_triangles,
            "components": x.components,
        }));
        // UVs only on a part that has them, so other reports are as they
        // were: measured as exported (divided by the map's size).
        if let Some(u) = m.uv_measures(&p.name)
            && let Some(Value::Object(o)) = list.last_mut()
        {
            coverage = Some(coverage.map_or(u.coverage, |c| c.min(u.coverage)));
            overlap = Some(overlap.map_or(u.overlap, |c| c.max(u.overlap)));
            o.insert(
                "uv".into(),
                json!({
                    "area": u.area,
                    "coverage": u.coverage,
                    "overlap": u.overlap,
                    "overlappingPairs": u.overlapping_pairs,
                    "flipped": u.flipped,
                    "zeroArea": u.zero_area,
                    "outside": u.outside,
                }),
            );
        }
    }
    let empty = tris == 0;
    let unit = match m.unit() {
        Unit::Millimetre => "mm",
        Unit::Centimetre => "cm",
        Unit::Metre => "m",
    };
    json!({
        "name": m.name(),
        "unit": unit,
        "parts": list,
        "joints": m.joint_names(),
        "triangles": tris,
        "vertices": verts,
        "boundsMin": if empty { Value::Null } else { v3(lo) },
        "boundsMax": if empty { Value::Null } else { v3(hi) },
        "pivot": v3(DVec3::ZERO),
        "area": area,
        "volume": volume,
        // Over the parts with UVs: the least coverage, the most overlap
        // (each part's UVs are its own map's); null when none has UVs.
        "uvCoverage": coverage.map_or(Value::Null, Value::from),
        "uvOverlap": overlap.map_or(Value::Null, Value::from),
        "textureBytes": 0,
    })
}
