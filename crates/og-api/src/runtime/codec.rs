//! Reflected layouts as the hosts see them: which column kind each field has,
//! and the conversion between a value's JSON (what crosses the boundary) and
//! its packed bytes (what `og-ecs` stores).

use og_core::reflect::{FieldSchema, FieldType, Schema};
use serde_json::{json, Map, Value};

/// A column kind of the IDL (`reflect`, `world`, `render` modules).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    F32,
    F64,
    U32,
    I32,
    Bool,
    Vector2,
    Vector3,
    Quaternion,
    Matrix4,
    Color,
    Entity,
    Geometry,
    Material,
}

impl Kind {
    pub fn parse(name: &str) -> Option<Kind> {
        Some(match name {
            "f32" => Kind::F32,
            "f64" => Kind::F64,
            "u32" => Kind::U32,
            "i32" => Kind::I32,
            "bool" => Kind::Bool,
            "Vector2" => Kind::Vector2,
            "Vector3" => Kind::Vector3,
            "Quaternion" => Kind::Quaternion,
            "Matrix4" => Kind::Matrix4,
            "Color" => Kind::Color,
            "Entity" => Kind::Entity,
            "Geometry" => Kind::Geometry,
            "Material" => Kind::Material,
            _ => return None,
        })
    }

    /// The JSON keys of a struct-valued kind, in lane order.
    fn keys(self) -> &'static [&'static str] {
        match self {
            Kind::Vector2 => &["x", "y"],
            Kind::Vector3 => &["x", "y", "z"],
            Kind::Quaternion => &["x", "y", "z", "w"],
            Kind::Color => &["r", "g", "b"],
            _ => &[],
        }
    }

    /// The og-core field type this kind is stored as. A component's columns
    /// are viewed as typed arrays, so a kind og-core cannot store as the IDL
    /// declares it is refused there; a resource is only read and written as
    /// a value, so a `bool` is stored as a `u32` 0 or 1.
    pub fn field_type(self, viewed: bool) -> Result<FieldType, String> {
        Ok(match self {
            Kind::F32 => FieldType::F32,
            Kind::F64 => FieldType::F64,
            Kind::U32 => FieldType::U32,
            Kind::I32 => FieldType::I32,
            Kind::Vector2 => FieldType::Vec2,
            Kind::Vector3 | Kind::Color => FieldType::Vec3,
            Kind::Quaternion => FieldType::Quat,
            Kind::Entity | Kind::Geometry | Kind::Material => FieldType::U64,
            Kind::Bool if !viewed => FieldType::U32,
            Kind::Bool => {
                return Err("a bool column is a Uint8Array of one byte per entity, and og-core has no one-byte field type yet (FieldType::Bool is requested from Core); use a u32 field of 0 and 1 meanwhile".into())
            }
            Kind::Matrix4 => {
                return Err("a Matrix4 column is sixteen f32 per entity, and og-core has no 4x4 field type yet (FieldType::Mat4 is requested from Core)".into())
            }
        })
    }
}

/// One field of a host layout.
#[derive(Clone, Debug)]
pub struct FieldLayout {
    pub name: String,
    pub kind: Kind,
    pub ty: FieldType,
    pub default: Value,
    pub readonly: bool,
    pub tunable: bool,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub unit: Option<String>,
    pub label: Option<String>,
    pub doc: String,
}

/// What a record is registered as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordKind {
    Component,
    Resource,
    Event,
}

impl RecordKind {
    pub fn as_str(self) -> &'static str {
        match self {
            RecordKind::Component => "component",
            RecordKind::Resource => "resource",
            RecordKind::Event => "event",
        }
    }
}

/// A component, resource or event layout declared by name.
#[derive(Clone, Debug)]
pub struct Layout {
    pub schema: String,
    pub kind: RecordKind,
    pub fields: Vec<FieldLayout>,
}

fn number(v: &Value, what: &str) -> Result<f64, String> {
    v.as_f64()
        .ok_or_else(|| format!("{what} is a number, got {v}"))
}

impl Layout {
    /// A layout from the wire form of `Fields`: a JSON list of
    /// `{ name, kind, default, options }`, in declaration order.
    pub fn from_fields(schema: &str, kind: RecordKind, fields: &Value) -> Result<Layout, String> {
        let list = fields.as_array().ok_or_else(|| {
            format!("'{schema}': fields cross as a list of {{ name, kind, default, options }}")
        })?;
        let mut out = Vec::with_capacity(list.len());
        for f in list {
            let name = f["name"]
                .as_str()
                .ok_or_else(|| format!("'{schema}': a field without a name"))?
                .to_string();
            let kind_name = f["kind"].as_str().unwrap_or_default();
            let k = Kind::parse(kind_name).ok_or_else(|| {
                format!("'{schema}.{name}': `{kind_name}` is not a column kind; use a column factory such as f32() or Vector3()")
            })?;
            let ty = k
                .field_type(kind == RecordKind::Component)
                .map_err(|e| format!("'{schema}.{name}': {e}"))?;
            let o = &f["options"];
            let opt_num = |key: &str| o.get(key).and_then(Value::as_f64);
            let opt_str = |key: &str| {
                o.get(key)
                    .and_then(Value::as_str)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
            };
            out.push(FieldLayout {
                name,
                kind: k,
                ty,
                default: f.get("default").cloned().unwrap_or(Value::Null),
                readonly: false,
                tunable: o.get("tunable").and_then(Value::as_bool).unwrap_or(false),
                min: opt_num("min"),
                max: opt_num("max"),
                step: opt_num("step"),
                unit: opt_str("unit"),
                label: opt_str("label"),
                doc: opt_str("doc").unwrap_or_default(),
            });
        }
        let layout = Layout {
            schema: schema.to_string(),
            kind,
            fields: out,
        };
        // Defaults must encode: refuse a bad one at declaration.
        layout.encode(None, None)?;
        Ok(layout)
    }

    /// A layout of an engine-owned record from the generated table.
    pub fn from_decl(decl: &crate::host::RecordDecl) -> Result<Layout, String> {
        let kind = match decl.kind {
            "component" => RecordKind::Component,
            "resource" => RecordKind::Resource,
            _ => RecordKind::Event,
        };
        let mut fields = Vec::new();
        for f in decl.fields {
            let k = Kind::parse(f.column).ok_or_else(|| {
                format!(
                    "'{}.{}' has type {}, which og-core cannot store yet",
                    decl.schema, f.name, f.column
                )
            })?;
            let ty = k
                .field_type(kind == RecordKind::Component)
                .map_err(|e| format!("'{}.{}': {e}", decl.schema, f.name))?;
            fields.push(FieldLayout {
                name: f.name.to_string(),
                kind: k,
                ty,
                default: serde_json::from_str(f.default).unwrap_or(Value::Null),
                readonly: f.readonly,
                tunable: f.tunable,
                min: None,
                max: None,
                step: None,
                unit: None,
                label: None,
                doc: String::new(),
            });
        }
        Ok(Layout {
            schema: decl.schema.to_string(),
            kind,
            fields,
        })
    }

    /// The og-core schema this layout registers as.
    pub fn schema(&self) -> Schema {
        Schema {
            name: self.schema.clone(),
            version: 1,
            doc: String::new(),
            fields: self
                .fields
                .iter()
                .map(|f| {
                    let mut s = FieldSchema::new(f.name.clone(), f.ty);
                    s.doc = f.doc.clone();
                    s.label = f.label.clone();
                    s.unit = f.unit.clone();
                    s.min = f.min;
                    s.max = f.max;
                    s.tunable = f.tunable;
                    s
                })
                .collect(),
        }
    }

    pub fn size(&self) -> usize {
        self.fields.iter().map(|f| f.ty.size()).sum()
    }

    pub fn field(&self, name: &str) -> Option<(usize, &FieldLayout)> {
        self.fields.iter().enumerate().find(|(_, f)| f.name == name)
    }

    /// Byte offset of field `i` in packed form.
    pub fn offset(&self, i: usize) -> usize {
        self.fields[..i].iter().map(|f| f.ty.size()).sum()
    }

    /// Packed bytes: each field from `value` when it names it, else from
    /// `base` (the current value) or the field's default. Unknown keys are
    /// refused, so a typo is not silently dropped.
    pub fn encode(&self, base: Option<&[u8]>, value: Option<&Value>) -> Result<Vec<u8>, String> {
        let obj = match value {
            None | Some(Value::Null) => None,
            Some(Value::Object(o)) => Some(o),
            Some(other) => {
                return Err(format!(
                    "a '{}' value is an object, got {other}",
                    self.schema
                ))
            }
        };
        if let Some(o) = obj {
            if let Some(k) = o.keys().find(|k| self.field(k).is_none()) {
                return Err(format!("'{}' has no field '{k}'", self.schema));
            }
        }
        let mut out = Vec::with_capacity(self.size());
        for (i, f) in self.fields.iter().enumerate() {
            match obj.and_then(|o| o.get(&f.name)) {
                Some(v) => encode_field(f, v, &mut out)
                    .map_err(|e| format!("'{}.{}': {e}", self.schema, f.name))?,
                None => match base {
                    Some(b) => {
                        let at = self.offset(i);
                        out.extend_from_slice(&b[at..at + f.ty.size()]);
                    }
                    None => encode_field(f, &f.default, &mut out)
                        .map_err(|e| format!("'{}.{}' default: {e}", self.schema, f.name))?,
                },
            }
        }
        Ok(out)
    }

    /// The value of packed bytes as JSON.
    pub fn decode(&self, bytes: &[u8]) -> Value {
        let mut map = Map::new();
        let mut at = 0;
        for f in &self.fields {
            let size = f.ty.size();
            map.insert(f.name.clone(), decode_field(f, &bytes[at..at + size]));
            at += size;
        }
        Value::Object(map)
    }

    /// One field of packed bytes as JSON.
    pub fn decode_one(&self, bytes: &[u8], i: usize) -> Value {
        let at = self.offset(i);
        let f = &self.fields[i];
        decode_field(f, &bytes[at..at + f.ty.size()])
    }
}

fn f32_lanes(v: &Value, keys: &[&str], out: &mut Vec<u8>) -> Result<(), String> {
    let o = v
        .as_object()
        .ok_or_else(|| format!("expected {{ {} }}, got {v}", keys.join(", ")))?;
    for k in keys {
        let n = number(o.get(*k).unwrap_or(&Value::Null), k)?;
        out.extend_from_slice(&(n as f32).to_le_bytes());
    }
    Ok(())
}

fn encode_field(f: &FieldLayout, v: &Value, out: &mut Vec<u8>) -> Result<(), String> {
    match f.kind {
        Kind::F32 => out.extend_from_slice(&(number(v, "the value")? as f32).to_le_bytes()),
        Kind::F64 => out.extend_from_slice(&number(v, "the value")?.to_le_bytes()),
        Kind::U32 => {
            let n = number(v, "the value")?;
            if n.fract() != 0.0 || !(0.0..=u32::MAX as f64).contains(&n) {
                return Err(format!("{n} is not a u32"));
            }
            out.extend_from_slice(&(n as u32).to_le_bytes())
        }
        Kind::I32 => {
            let n = number(v, "the value")?;
            if n.fract() != 0.0 || !(i32::MIN as f64..=i32::MAX as f64).contains(&n) {
                return Err(format!("{n} is not an i32"));
            }
            out.extend_from_slice(&(n as i32).to_le_bytes())
        }
        Kind::Bool => {
            let b = v
                .as_bool()
                .ok_or_else(|| format!("expected true or false, got {v}"))?;
            out.extend_from_slice(&u32::from(b).to_le_bytes())
        }
        Kind::Entity | Kind::Geometry | Kind::Material => {
            let n = number(v, "a handle")?;
            if n.fract() != 0.0 || !(0.0..9_007_199_254_740_992.0).contains(&n) {
                return Err(format!("{n} is not a handle"));
            }
            out.extend_from_slice(&(n as u64).to_le_bytes())
        }
        Kind::Vector2 | Kind::Vector3 | Kind::Quaternion | Kind::Color => {
            f32_lanes(v, f.kind.keys(), out)?
        }
        Kind::Matrix4 => return Err("Matrix4 fields cannot be stored yet".into()),
    }
    Ok(())
}

fn f32_at(b: &[u8], i: usize) -> f32 {
    f32::from_le_bytes(b[i * 4..i * 4 + 4].try_into().expect("4 bytes"))
}

fn decode_field(f: &FieldLayout, b: &[u8]) -> Value {
    match f.kind {
        Kind::F32 => json!(f32_at(b, 0) as f64),
        Kind::F64 => json!(f64::from_le_bytes(b[..8].try_into().expect("8 bytes"))),
        Kind::U32 => json!(u32::from_le_bytes(b[..4].try_into().expect("4 bytes"))),
        Kind::I32 => json!(i32::from_le_bytes(b[..4].try_into().expect("4 bytes"))),
        Kind::Bool => json!(u32::from_le_bytes(b[..4].try_into().expect("4 bytes")) != 0),
        Kind::Entity | Kind::Geometry | Kind::Material => {
            json!(u64::from_le_bytes(b[..8].try_into().expect("8 bytes")))
        }
        Kind::Vector2 | Kind::Vector3 | Kind::Quaternion | Kind::Color => {
            let mut m = Map::new();
            for (i, k) in f.kind.keys().iter().enumerate() {
                m.insert(k.to_string(), json!(f32_at(b, i) as f64));
            }
            Value::Object(m)
        }
        Kind::Matrix4 => Value::Null,
    }
}
