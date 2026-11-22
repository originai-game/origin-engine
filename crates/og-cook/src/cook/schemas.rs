//! The engine's component schemas from the IDL's records, as `og cook`
//! packs scene tables with them.

/// The engine's own components as the cook packs scene tables with them
/// (`Transform`, `Model`, the lights), by the names scenes use.
pub fn engine_schemas() -> Vec<og_core::Schema> {
    use og_core::reflect::{FieldSchema, FieldType};
    let scalar = |f: &og_records::FieldDecl| -> Option<FieldType> {
        Some(match f.column {
            "f32" => FieldType::F32,
            "f64" => FieldType::F64,
            "u32" => FieldType::U32,
            "i32" => FieldType::I32,
            "Vector2" => FieldType::Vec2,
            "Vector3" | "Color" => FieldType::Vec3,
            "Quaternion" => FieldType::Quat,
            "Entity" | "Geometry" | "Material" | "Asset" | "Texture" => FieldType::U64,
            other => {
                return serde_json::from_value(serde_json::Value::String(
                    match other {
                        "bool" => "bool",
                        "Matrix4" => "mat4",
                        o => o,
                    }
                    .into(),
                ))
                .ok();
            }
        })
    };
    let ty = |f: &og_records::FieldDecl| -> Option<FieldType> {
        if !f.variants.is_empty() {
            // An enum column (`ColliderShape`, `LightMobility`): its index.
            return Some(FieldType::Enum);
        }
        if f.capacity > 0 {
            // A text field (`str.48` in the IDL).
            return FieldType::string(f.capacity as usize).ok();
        }
        if f.list > 0 {
            // A list field (`List.u32.4` in the IDL): its element's scalar.
            return FieldType::list(scalar(f)?.scalar(), f.list as usize).ok();
        }
        scalar(f)
    };
    og_records::RECORDS
        .iter()
        .filter(|r| r.kind == "component")
        .filter_map(|r| {
            let fields = r
                .fields
                .iter()
                .map(|f| {
                    ty(f).map(|t| {
                        let mut fs = FieldSchema::new(f.name.to_string(), t);
                        fs.variants = f.variants.iter().map(|v| v.to_string()).collect();
                        // A list starts empty: it declares no default.
                        fs.default = (!t.is_list())
                            .then(|| default_lanes(f, t.lanes()))
                            .flatten();
                        fs
                    })
                })
                .collect::<Option<Vec<_>>>()?;
            Some(og_core::Schema {
                name: r.name.to_string(),
                version: 1,
                doc: String::new(),
                fields,
            })
        })
        .collect()
}

/// A field's IDL default (JSON) as the lanes the cook packs, so a scene
/// row that omits a field gets the component's default, not zero.
fn default_lanes(f: &og_records::FieldDecl, lanes: usize) -> Option<Vec<f64>> {
    use serde_json::Value;
    let v: Value = serde_json::from_str(f.default).ok()?;
    let out: Vec<f64> = match &v {
        Value::Number(n) => vec![n.as_f64()?],
        Value::Bool(b) => vec![f64::from(u8::from(*b))],
        Value::String(s) => vec![f.variants.iter().position(|x| x == s)? as f64],
        Value::Object(m) if m.contains_key("elements") => m["elements"]
            .as_array()?
            .iter()
            .map(Value::as_f64)
            .collect::<Option<_>>()?,
        Value::Object(m) => ["x", "y", "z", "w", "r", "g", "b"]
            .iter()
            .filter_map(|k| m.get(*k))
            .map(Value::as_f64)
            .collect::<Option<_>>()?,
        _ => return None,
    };
    (out.len() == lanes).then_some(out)
}
