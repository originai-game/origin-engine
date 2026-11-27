//! Packed record rows retain canonical defaults and typed content relocations.
//! The numeric layout alone cannot distinguish an asset from an ordinary u64.
use super::{Field, Record};
use og_core::{FieldSchema, FieldType, Layout, Schema};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PackedRecord {
    pub schema: Schema,
    defaults: Vec<u8>,
    columns: BTreeMap<String, String>,
}

impl PackedRecord {
    pub fn column(&self, name: &str) -> Option<&str> {
        self.columns.get(name).map(String::as_str)
    }

    pub fn new(record: &Record) -> Result<Self, String> {
        let mut fields = Vec::new();
        let mut defaults = Vec::new();
        let mut columns = BTreeMap::new();
        for field in &record.fields {
            let column = og_records::COLUMNS
                .iter()
                .find(|c| c.name == field.kind)
                .ok_or_else(|| format!("{}: unknown column {}", record.name, field.kind))?;
            let scalar = match field.kind.as_str() {
                "Ease" => FieldType::Ease,
                _ if !column.variants.is_empty() => FieldType::Enum,
                "f32" => FieldType::F32,
                "f64" => FieldType::F64,
                "u32" => FieldType::U32,
                "i32" => FieldType::I32,
                "bool" => FieldType::Bool,
                "Vector2" => FieldType::Vec2,
                "Vector3" | "Color" => FieldType::Vec3,
                "Quaternion" => FieldType::Quat,
                "Matrix4" => FieldType::Mat4,
                "str" => FieldType::string(field.capacity.unwrap_or(256) as usize)
                    .map_err(|e| e.to_string())?,
                _ => FieldType::U64,
            };
            let ty = match field.list {
                Some(capacity) => FieldType::list(scalar.scalar(), capacity as usize)
                    .map_err(|e| e.to_string())?,
                None => scalar,
            };
            let mut packed = FieldSchema::new(&field.name, ty);
            if ty == FieldType::Enum {
                packed.variants = column.variants.iter().map(|v| (*v).into()).collect();
            }
            packed.min = field.options.get("min").and_then(Value::as_f64);
            packed.max = field.options.get("max").and_then(Value::as_f64);
            packed.worldspace = field
                .options
                .get("worldspace")
                .and_then(Value::as_str)
                .map(str::to_owned);
            let default = runtime_default(&packed, &field.default)?;
            defaults.extend_from_slice(&default);
            // Reflection's numeric defaults also serve inspector and restore.
            if !ty.is_string() && !ty.is_list() && ty != FieldType::Ease {
                packed.default = Some(
                    default
                        .chunks_exact(ty.scalar().size())
                        .map(|b| {
                            use og_core::reflect::Scalar;
                            match ty.scalar() {
                                Scalar::F32 => f32::from_le_bytes(b.try_into().unwrap()) as f64,
                                Scalar::F64 => f64::from_le_bytes(b.try_into().unwrap()),
                                Scalar::U32 => u32::from_le_bytes(b.try_into().unwrap()) as f64,
                                Scalar::I32 => i32::from_le_bytes(b.try_into().unwrap()) as f64,
                                Scalar::U64 => u64::from_le_bytes(b.try_into().unwrap()) as f64,
                                Scalar::I64 => i64::from_le_bytes(b.try_into().unwrap()) as f64,
                                Scalar::Bool | Scalar::Utf8 => b[0] as f64,
                            }
                        })
                        .collect(),
                );
            }
            columns.insert(field.name.clone(), field.kind.clone());
            fields.push(packed);
        }
        let schema = Schema {
            name: record.name.clone(),
            version: 1,
            doc: String::new(),
            fields,
        };
        schema.validate().map_err(|e| e.to_string())?;
        schema.check_packed(&defaults).map_err(|e| e.to_string())?;
        Ok(Self {
            schema,
            defaults,
            columns,
        })
    }

    /// Only authored fields are patched. A returned relocation retains its
    /// content identity until the runtime invokes the column's actual loader.
    pub fn pack(
        &self,
        patch: &Map<String, Value>,
    ) -> Result<(Vec<u8>, Map<String, Value>), String> {
        let mut bytes = self.defaults.clone();
        let layout = Layout::of(&self.schema);
        let mut bindings = Map::new();
        for (name, value) in patch {
            let index = self
                .schema
                .field_index(name)
                .ok_or_else(|| format!("{} has no field {name}", self.schema.name))?;
            let field = &self.schema.fields[index];
            let column = &self.columns[name];
            fn reference(value: &Value) -> bool {
                match value {
                    Value::Object(m) => m.contains_key("$ogRef") || m.values().any(reference),
                    Value::Array(a) => a.iter().any(reference),
                    _ => false,
                }
            }
            if reference(value) {
                bindings.insert(name.clone(), value.clone());
                continue;
            }
            let value = author_value(column, value)?;
            let packed = runtime_default(field, &value)
                .map_err(|e| format!("{}.{name}: {e}", self.schema.name))?;
            bytes[layout.range(index)].copy_from_slice(&packed);
        }
        Ok((bytes, bindings))
    }
}

/// Tuple author values become the runtime column's ordinary JSON shape;
/// record defaults already have that shape and never pass through this step.
fn author_value(column: &str, value: &Value) -> Result<Value, String> {
    if column == "Color"
        && let Some(text) = value.as_str()
    {
        let [r, g, b, _] = og_core::color::parse_hex(text).ok_or("invalid hex color")?;
        return Ok(
            serde_json::json!({"r":og_core::color::srgb8_to_linear(r),"g":og_core::color::srgb8_to_linear(g),"b":og_core::color::srgb8_to_linear(b)}),
        );
    }
    let keys: &[&str] = match column {
        "Vector2" => &["x", "y"],
        "Vector3" => &["x", "y", "z"],
        "Quaternion" => &["x", "y", "z", "w"],
        "Color" => &["r", "g", "b"],
        "Matrix4" => return Ok(serde_json::json!({"elements":value})),
        _ => return Ok(value.clone()),
    };
    let values = value
        .as_array()
        .filter(|v| v.len() == keys.len())
        .ok_or("expected a column tuple")?;
    Ok(Value::Object(
        keys.iter()
            .zip(values)
            .map(|(k, v)| ((*k).into(), v.clone()))
            .collect(),
    ))
}

fn runtime_default(field: &FieldSchema, value: &Value) -> Result<Vec<u8>, String> {
    if value.is_null() {
        return Ok(field.default_bytes());
    }
    let ty = field.ty;
    if let Some((element, _)) = ty.list_of() {
        let values = value.as_array().ok_or("a list is an array")?;
        let scalar = FieldSchema::new(&field.name, element.field_type());
        let packed = values
            .iter()
            .map(|v| runtime_default(&scalar, v))
            .collect::<Result<Vec<_>, _>>()?;
        return og_core::pack_list(ty, &packed.iter().map(Vec::as_slice).collect::<Vec<_>>());
    }
    if ty.is_string() {
        return og_core::pack_str(
            value.as_str().ok_or("a string field needs text")?,
            ty.size(),
        );
    }
    if ty == FieldType::Ease {
        return og_core::ease::Ease::parse_text(value.as_str().ok_or("an ease needs text")?)
            .map(|e| e.to_bytes().to_vec());
    }
    if ty == FieldType::Enum {
        let index = field
            .variants
            .iter()
            .position(|s| Some(s.as_str()) == value.as_str())
            .ok_or("unknown enum variant")?;
        return Ok((index as u32).to_le_bytes().to_vec());
    }
    let numbers = if ty == FieldType::Bool {
        vec![if value
            .as_bool()
            .ok_or("a boolean field needs true or false")?
        {
            1.
        } else {
            0.
        }]
    } else if ty.lanes() == 1 {
        vec![value.as_f64().ok_or("a numeric field needs a number")?]
    } else if ty == FieldType::Mat4 {
        value["elements"]
            .as_array()
            .ok_or("a matrix needs elements")?
            .iter()
            .map(|v| v.as_f64().ok_or("a matrix element is a number"))
            .collect::<Result<Vec<_>, _>>()?
    } else {
        let keys: &[&str] = match ty {
            FieldType::Vec2 => &["x", "y"],
            FieldType::Vec3 if value.get("r").is_some() => &["r", "g", "b"],
            FieldType::Vec3 => &["x", "y", "z"],
            _ => &["x", "y", "z", "w"],
        };
        keys.iter()
            .map(|k| value[k].as_f64().ok_or("a vector lane is a number"))
            .collect::<Result<Vec<_>, _>>()?
    };
    if numbers.len() != ty.lanes() {
        return Err("wrong column lane count".into());
    }
    let mut out = Vec::with_capacity(ty.size());
    for number in numbers {
        use og_core::reflect::Scalar;
        match ty.scalar() {
            Scalar::F32 => out.extend_from_slice(&(number as f32).to_le_bytes()),
            Scalar::F64 => out.extend_from_slice(&number.to_le_bytes()),
            Scalar::I32
                if number.fract() == 0.
                    && (i32::MIN as f64..=i32::MAX as f64).contains(&number) =>
            {
                out.extend_from_slice(&(number as i32).to_le_bytes())
            }
            Scalar::U32 if number.fract() == 0. && (0.0..=u32::MAX as f64).contains(&number) => {
                out.extend_from_slice(&(number as u32).to_le_bytes())
            }
            Scalar::U64
                if number.fract() == 0. && (0.0..9_007_199_254_740_992.0).contains(&number) =>
            {
                out.extend_from_slice(&(number as u64).to_le_bytes())
            }
            Scalar::Bool => out.push(u8::from(number != 0.)),
            _ => return Err("number does not fit its column".into()),
        }
    }
    field.check_value(&out)?;
    Ok(out)
}

impl super::Records {
    pub fn packed(&self) -> Result<BTreeMap<String, PackedRecord>, String> {
        let mut out = BTreeMap::new();
        for record in og_records::RECORDS.iter().filter(|r| r.kind != "event") {
            let record = Record {
                kind: record.kind.into(),
                name: record.schema.into(),
                fields: record
                    .fields
                    .iter()
                    .map(|f| Field {
                        name: f.name.into(),
                        kind: f.column.into(),
                        default: serde_json::from_str(f.default).expect("IDL default"),
                        options: [
                            ("min", f.min.map(Value::from)),
                            ("max", f.max.map(Value::from)),
                            (
                                "worldspace",
                                (!f.worldspace.is_empty()).then(|| Value::from(f.worldspace)),
                            ),
                        ]
                        .into_iter()
                        .filter_map(|(key, value)| value.map(|value| (key.into(), value)))
                        .collect(),
                        capacity: (f.capacity > 0).then_some(f.capacity),
                        list: (f.list > 0).then_some(f.list),
                    })
                    .collect(),
            };
            out.insert(record.name.clone(), PackedRecord::new(&record)?);
        }
        for declaration in self
            .declarations
            .values()
            .filter(|d| d.record.kind != "event")
        {
            out.insert(
                declaration.record.name.clone(),
                PackedRecord::new(&declaration.record)?,
            );
        }
        Ok(out)
    }
}
