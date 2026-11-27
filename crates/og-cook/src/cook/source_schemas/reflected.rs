//! Author record patches from the IDL, without a second field table.
use og_core::reflect::source::{InputSchema, schemars, schemars::JsonSchema};
use og_records::FieldDecl;
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Value, json};
use std::marker::PhantomData;

/// Selects an existing IDL record, never a parallel author field table.
pub trait RecordMarker {
    const NAME: &'static str;
    /// Use a real Reflect layout when this type is not in the IDL record registry.
    /// Layouts carry no readonly or handle-column metadata; none is inferred.
    fn layout() -> Option<og_core::Schema> {
        None
    }
    /// A domain may exclude otherwise writable fields (e.g. animated look values).
    const EXCLUDED: &'static [&'static str] = &[];
    /// Fields the domain requires even though ordinary record patches may omit them.
    const REQUIRED: &'static [&'static str] = &[];
}

/// A validated record patch. There is deliberately no unchecked constructor or
/// mutable map access. Even direct serde decoding uses the reflected schema.
#[derive(Debug)]
pub struct Reflected<M> {
    fields: Map<String, Value>,
    marker: PhantomData<M>,
}

impl<M> Reflected<M> {
    pub fn fields(&self) -> &Map<String, Value> {
        &self.fields
    }

    pub fn into_fields(self) -> Map<String, Value> {
        self.fields
    }
}

fn marked_schema<M: RecordMarker>() -> Result<Value, String> {
    let mut schema = match M::layout() {
        Some(layout) => schema_for_layout(&layout)?,
        None => schema_for_record(M::NAME)?,
    };
    for name in M::EXCLUDED {
        schema["properties"]
            .as_object_mut()
            .expect("record object")
            .remove(*name);
    }
    for name in M::REQUIRED {
        if schema["properties"].get(*name).is_none() {
            return Err(format!(
                "{}: required field {name} is absent or excluded",
                M::NAME
            ));
        }
    }
    schema["required"] = json!(M::REQUIRED);
    Ok(schema)
}

impl<'de, M: RecordMarker> Deserialize<'de> for Reflected<M> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let fields = Map::<String, Value>::deserialize(deserializer)?;
        let schema = marked_schema::<M>()
            .and_then(InputSchema::<Map<String, Value>>::from_document)
            .map_err(serde::de::Error::custom)?;
        schema
            .validate(&Value::Object(fields.clone()))
            .map_err(serde::de::Error::custom)?;
        check_text_capacity(&fields, &schema.document()["properties"])
            .map_err(serde::de::Error::custom)?;
        Ok(Self {
            fields,
            marker: PhantomData,
        })
    }
}

fn check_text_capacity(fields: &Map<String, Value>, properties: &Value) -> Result<(), String> {
    for (name, value) in fields {
        let capacity = properties[name]["x-og-capacity-bytes"]
            .as_u64()
            .unwrap_or(0);
        if capacity > 0 && value.as_str().is_some_and(|s| s.len() as u64 > capacity) {
            return Err(format!("{name}: text exceeds {capacity} UTF-8 bytes"));
        }
    }
    Ok(())
}

impl<M: RecordMarker> JsonSchema for Reflected<M> {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        format!("Reflected_{}", std::any::type_name::<M>()).into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        marked_schema::<M>()
            .expect("supported reflected record")
            .try_into()
            .expect("record schema")
    }
}

/// A self-contained, closed record patch. Every writable field is optional;
/// this is not a complete table row. No external refs or hidden definitions.
/// `name` is the IDL name or its registered schema name (e.g. og.Transform).
pub fn schema_for_record(name: &str) -> Result<Value, String> {
    let record = og_records::RECORDS
        .iter()
        .find(|r| r.name == name || r.schema == name)
        .ok_or_else(|| format!("unknown reflected record {name}"))?;
    let properties = record
        .fields
        .iter()
        .filter(|f| !f.readonly)
        .map(|f| Ok((f.name.to_string(), field(f)?)))
        .collect::<Result<Map<_, _>, String>>()?;
    Ok(json!({
        "type": "object", "properties": properties,
        "additionalProperties": false, "x-og-record": record.schema,
    }))
}

/// A closed patch from an actual packed Reflect layout. Unlike IDL records,
/// layouts do not identify content references or readonly fields, so numeric
/// slots remain numbers. Unsupported columns fail rather than accepting any.
pub fn schema_for_layout(layout: &og_core::Schema) -> Result<Value, String> {
    use og_core::reflect::FieldType;
    let mut properties = Map::new();
    for f in &layout.fields {
        let mut value = match f.ty {
            FieldType::Str(capacity) => json!({"type": "string", "maxLength": capacity,
                "x-og-capacity-bytes": capacity, "pattern": "^[^\\u0000]*$"}),
            FieldType::Enum if !f.variants.is_empty() => {
                json!({"type": "string", "enum": f.variants})
            }
            FieldType::Enum | FieldType::Ease => {
                return Err(format!(
                    "{}.{}: unsupported {} layout column",
                    layout.name, f.name, f.ty
                ));
            }
            ty => {
                let mut scalar = scalar(ty.scalar());
                numeric_range(&mut scalar, f.min, f.max);
                match ty {
                    FieldType::List(_, capacity) => {
                        json!({"type": "array", "items": scalar, "maxItems": capacity})
                    }
                    ty if ty.lanes() > 1 => json!({"type": "array", "items": scalar,
                        "minItems": ty.lanes(), "maxItems": ty.lanes()}),
                    _ => scalar,
                }
            }
        };
        value["description"] = json!(f.doc);
        if let Some(unit) = &f.unit {
            value["x-og-unit"] = json!(unit);
        }
        properties.insert(f.name.clone(), value);
    }
    Ok(json!({"type": "object", "additionalProperties": false,
        "properties": properties, "x-og-layout": layout.name, "description": layout.doc}))
}

fn scalar(ty: og_core::reflect::Scalar) -> Value {
    use og_core::reflect::Scalar;
    match ty {
        Scalar::F32 => number(),
        Scalar::F64 => json!({"type": "number"}),
        Scalar::I32 => json!({"type": "integer", "minimum": i32::MIN, "maximum": i32::MAX}),
        Scalar::U32 => json!({"type": "integer", "minimum": 0, "maximum": u32::MAX}),
        Scalar::I64 => json!({"type": "integer", "minimum": i64::MIN, "maximum": i64::MAX}),
        Scalar::U64 => json!({"type": "integer", "minimum": 0, "maximum": u64::MAX}),
        Scalar::Bool => json!({"type": "boolean"}),
        Scalar::Utf8 => json!({"type": "string", "maxLength": 1, "x-og-capacity-bytes": 1}),
    }
}

pub(crate) fn numeric_range(value: &mut Value, min: Option<f64>, max: Option<f64>) {
    if let Some(min) = min {
        let existing = value["minimum"].as_f64().unwrap_or(f64::NEG_INFINITY);
        value["minimum"] = json!(min.max(existing));
    }
    if let Some(max) = max {
        let existing = value["maximum"].as_f64().unwrap_or(f64::INFINITY);
        value["maximum"] = json!(max.min(existing));
    }
}

fn field(f: &FieldDecl) -> Result<Value, String> {
    let mut value =
        column_schema(f.column, f.variants, f.capacity).map_err(|e| format!("{}: {e}", f.name))?;
    if matches!(f.column, "f32" | "f64" | "u32" | "i32") {
        numeric_range(&mut value, f.min, f.max);
    } else if matches!(f.column, "Vector2" | "Vector3" | "Quaternion" | "Matrix4") {
        numeric_range(&mut value["items"], f.min, f.max);
    } else if f.column == "Color" {
        numeric_range(&mut value["anyOf"][0]["items"], f.min, f.max);
    }
    if f.list > 0 {
        value = json!({"type": "array", "items": value, "maxItems": f.list});
    }
    value["description"] = json!(f.doc);
    if !f.unit.is_empty() {
        value["x-og-unit"] = json!(f.unit);
    }
    if let Some(step) = f.step {
        value["x-og-step"] = json!(step);
    }
    Ok(value)
}

/// The same author column semantics for IDL records and pure game declarations.
/// Packed layouts alone cannot distinguish e.g. Texture from an ordinary u64.
pub(crate) fn column_schema(
    column: &str,
    variants: &[&str],
    capacity: u32,
) -> Result<Value, String> {
    Ok(if column == "ClipAsset" {
        let mut asset = schemars::schema_for!(super::ContentRef).to_value();
        asset["x-og-ref-kinds"] = json!(["model"]);
        json!({"type":"object", "properties":{"asset":asset,"clip":{"type":"string","minLength":1}},
            "required":["asset","clip"],"additionalProperties":false})
    } else if column == "Ease" {
        json!({"type": "string", "description": "Named or CSS timing curve; validated by the native easing parser."})
    } else if !variants.is_empty() {
        json!({"type": "string", "enum": variants})
    } else {
        match column {
            "bool" => json!({"type": "boolean"}),
            "f32" => number(),
            "f64" => json!({"type": "number"}),
            "u32" => json!({"type": "integer", "minimum": 0, "maximum": u32::MAX}),
            "i32" => json!({"type": "integer", "minimum": i32::MIN, "maximum": i32::MAX}),
            "str" | "String" => {
                let mut text = json!({"type": "string", "x-og-capacity-bytes": capacity});
                if capacity > 0 {
                    text["maxLength"] = json!(capacity);
                    text["pattern"] = json!("^[^\\u0000]*$");
                }
                text
            }
            "Vector2" => vector(2),
            "Vector3" => vector(3),
            "Color" => json!({"anyOf": [vector(3), {
                "type": "string", "pattern": "^#[0-9a-fA-F]{6}$"
            }]}),
            "Quaternion" => vector(4),
            "Matrix4" => vector(16),
            column => {
                if let Some(kinds) = content_kinds(column) {
                    // A content identity is never the numeric runtime handle.
                    let mut token = schemars::schema_for!(super::ContentRef).to_value();
                    token["x-og-ref-kinds"] = json!(kinds);
                    token["x-og-handle-column"] = json!(column);
                    token
                } else if runtime_handle(column) {
                    json!({"type": "integer", "minimum": 0, "maximum": u64::MAX,
                        "x-og-handle-column": column})
                } else {
                    return Err(format!("unsupported reflected column {column}"));
                }
            }
        }
    })
}

fn vector(count: usize) -> Value {
    json!({"type": "array", "items": number(), "minItems": count, "maxItems": count})
}

fn number() -> Value {
    json!({"type": "number", "minimum": -f32::MAX, "maximum": f32::MAX})
}

fn content_kinds(column: &str) -> Option<&'static [&'static str]> {
    Some(match column {
        "Texture" => &["texture"],
        "TextureArray" => &["texture-array"],
        "Sound" => &["sound"],
        "EnvironmentMap" => &["environment"],
        "Post" => &["post"],
        "EffectMaterial" => &["effect"],
        "ParticleEmitter" => &["particles"],
        "FlipbookSheet" => &["flipbook"],
        "VfxAsset" => &["vfx"],
        "TerrainData" => &["terrain"],
        "Video" => &["video"],
        "TilemapAsset" => &["tilemap"],
        "TimelineAsset" => &["timeline"],
        "Rig2D" => &["rig2d"],
        "Rig" => &["rig"],
        "GraphAsset" => &["animgraph"],
        "ClipAsset" | "SkeletonAsset" => &["model"],
        "Behaviour" => &["behaviour"],
        // Palette/LUT are loaded binary sources, not new author factory kinds.
        "Palette" => &["palette"],
        "Lut" => &["lut"],
        _ => return None,
    })
}

fn runtime_handle(column: &str) -> bool {
    matches!(
        column,
        "Entity"
            | "Geometry"
            | "Material"
            | "Asset"
            | "RenderTarget"
            | "Navigation"
            | "Path"
            | "Tween"
            | "SharedState"
            | "Simulation"
            | "TableRow"
    )
}

#[cfg(test)]
#[path = "reflected/tests.rs"]
mod tests;
