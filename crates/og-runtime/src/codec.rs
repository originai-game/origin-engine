//! Reflected layouts as the hosts see them: which column kind each field has,
//! and the conversion between a value's JSON (what crosses the boundary) and
//! its packed bytes (what `og-ecs` stores).
//!
//! A list field (`list(Asset(), 4)`, the IDL's `List.Post.8`) has its
//! element's kind and a list type (`u64[4]`): its JSON is an array of the
//! element's values, refused past the capacity, never truncated.

use og_core::reflect::{FieldSchema, FieldType, Schema};
use serde_json::{Map, Value, json};

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
    Asset,
    Sound,
    /// Fixed-capacity UTF-8 text: a `str(N)` field (`strN`), or an event's
    /// `String` (a `str256`).
    Str,
    /// An enum: a u32 index into the field's variants.
    Enum,
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
            "Asset" => Kind::Asset,
            "Sound" => Kind::Sound,
            // The animation module's asset handles: u64 like every handle.
            "SkeletonAsset" | "ClipAsset" | "GraphAsset" => Kind::Asset,
            // The effect assets and textures in records (Decal atlas, LUT, emitter).
            "Texture" | "Lut" | "ParticleEmitter" | "EnvironmentMap" | "Palette"
            | "TextureArray" => Kind::Asset,
            // The E4 handles: flipbook sheets, tilemaps, tweens.
            "FlipbookSheet" | "TilemapAsset" | "Tween" | "Rig2D" => Kind::Asset,
            // Footage (og.load.video).
            "Video" => Kind::Asset,
            // The E6 handles: navigation (grids, navmeshes, flow fields) and behaviours.
            "Navigation" | "Behaviour" => Kind::Asset,
            // Phase 2: terrains.
            "TerrainData" => Kind::Asset,
            // Depth S2: paths and timelines.
            "Path" | "TimelineAsset" => Kind::Asset,
            // Depth S8: render targets and custom post passes.
            "RenderTarget" | "Post" => Kind::Asset,
            "String" | "str" => Kind::Str,
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

    /// The og-core field type this kind is stored as: a `bool` is one byte
    /// (a `Uint8Array` column), a `Matrix4` sixteen f32, handles a `u64`, a
    /// `str(N)` field its `capacity` bytes (an event's text without one a
    /// `str256`), an enum a `u32` index; with `list`, a list of at most that
    /// many of a number or a handle.
    pub fn field_type(
        self,
        viewed: bool,
        capacity: Option<usize>,
        list: Option<usize>,
    ) -> Result<FieldType, String> {
        if let Some(n) = list {
            let element = match self {
                Kind::F32 => og_core::Scalar::F32,
                Kind::F64 => og_core::Scalar::F64,
                Kind::U32 => og_core::Scalar::U32,
                Kind::I32 => og_core::Scalar::I32,
                k if k.is_handle() => og_core::Scalar::U64,
                _ => {
                    return Err(
                        "a list holds f32, f64, u32, i32 or a handle (Asset(), Entity(), Post(), ...)"
                            .into(),
                    );
                }
            };
            if capacity.is_some() {
                return Err("a list has a capacity in elements, not bytes".into());
            }
            return FieldType::list(element, n).map_err(|e| e.reason().to_owned());
        }
        if let Some(n) = capacity {
            if self != Kind::Str {
                return Err("only a str field has a capacity".into());
            }
            return FieldType::string(n).map_err(|e| e.reason().to_owned());
        }
        Ok(match self {
            Kind::F32 => FieldType::F32,
            Kind::F64 => FieldType::F64,
            Kind::U32 => FieldType::U32,
            Kind::I32 => FieldType::I32,
            Kind::Vector2 => FieldType::Vec2,
            Kind::Vector3 | Kind::Color => FieldType::Vec3,
            Kind::Quaternion => FieldType::Quat,
            Kind::Entity | Kind::Geometry | Kind::Material | Kind::Asset | Kind::Sound => {
                FieldType::U64
            }
            Kind::Bool => FieldType::Bool,
            Kind::Matrix4 => FieldType::Mat4,
            Kind::Enum => FieldType::Enum,
            Kind::Str if !viewed => FieldType::Str(256),
            Kind::Str => {
                return Err("a string field declares its capacity in bytes: str(32)".into());
            }
        })
    }

    /// Whether the kind is a 64-bit handle (an entity or an asset).
    pub fn is_handle(self) -> bool {
        matches!(
            self,
            Kind::Entity | Kind::Geometry | Kind::Material | Kind::Asset | Kind::Sound
        )
    }

    /// The kind a field of a registered og-core schema is read as (a list's
    /// is its element's).
    pub fn of(ty: FieldType) -> Kind {
        match ty {
            FieldType::List(element, _) => Kind::of(element.field_type()),
            FieldType::F32 => Kind::F32,
            FieldType::F64 => Kind::F64,
            FieldType::I32 => Kind::I32,
            FieldType::U32 => Kind::U32,
            FieldType::I64 | FieldType::U64 => Kind::Entity,
            FieldType::Vec2 => Kind::Vector2,
            FieldType::Vec3 => Kind::Vector3,
            FieldType::Vec4 | FieldType::Quat => Kind::Quaternion,
            FieldType::Bool => Kind::Bool,
            FieldType::Mat4 => Kind::Matrix4,
            FieldType::Enum => Kind::Enum,
            FieldType::Str(_) => Kind::Str,
        }
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
    /// An enum field's variant names, in index order.
    pub variants: Vec<String>,
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
    /// Shared: a layout is cloned on every record call (`running_record`),
    /// and copying each field's strings was a tenth of og.world.get.
    pub fields: std::sync::Arc<Vec<FieldLayout>>,
    /// `(field, byte offset)` by field name, built on the first
    /// [`Layout::slot`] (the fields never change after construction).
    pub(crate) by_name: std::sync::Arc<std::sync::OnceLock<Vec<(usize, usize)>>>,
}

/// The variants of an IDL enum column (`AudioBus`), from the declared
/// columns: a game's field may be of an enum column no engine record uses.
fn enum_variants(column: &str) -> Option<Vec<String>> {
    og_api::host::COLUMNS
        .iter()
        .find(|c| c.name == column && !c.variants.is_empty())
        .map(|c| c.variants.iter().map(|v| v.to_string()).collect())
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
            let variants = enum_variants(kind_name);
            let k = match variants {
                Some(_) => Kind::Enum,
                None => Kind::parse(kind_name).ok_or_else(|| {
                    format!("'{schema}.{name}': `{kind_name}` is not a column kind; use a column factory such as f32() or Vector3()")
                })?,
            };
            let capacity = match f.get("capacity") {
                None | Some(Value::Null) => None,
                Some(c) => Some(
                    c.as_u64()
                        .and_then(|c| usize::try_from(c).ok())
                        .ok_or_else(|| {
                            format!(
                                "'{schema}.{name}': a capacity is a whole number of bytes, got {c}"
                            )
                        })?,
                ),
            };
            let list = match f.get("list") {
                None | Some(Value::Null) => None,
                Some(c) => Some(
                    c.as_u64()
                        .and_then(|c| usize::try_from(c).ok())
                        .ok_or_else(|| {
                            format!(
                                "'{schema}.{name}': a list's capacity is a whole number of elements, got {c}"
                            )
                        })?,
                ),
            };
            let ty = k
                .field_type(kind == RecordKind::Component, capacity, list)
                .map_err(|e| format!("'{schema}.{name}': {e}"))?;
            let o = &f["options"];
            if list.is_some() && o.get("tunable").and_then(Value::as_bool) == Some(true) {
                return Err(format!(
                    "'{schema}.{name}': a list is never tunable (Tune and the probe's bounds move one number)"
                ));
            }
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
                variants: variants.unwrap_or_default(),
            });
        }
        let layout = Layout {
            schema: schema.to_string(),
            kind,
            fields: out.into(),
            by_name: Default::default(),
        };
        // Defaults must encode: refuse a bad one at declaration.
        layout.encode(None, None)?;
        Ok(layout)
    }

    /// A layout of an engine-owned record from the generated table.
    pub fn from_decl(decl: &og_api::host::RecordDecl) -> Result<Layout, String> {
        let kind = match decl.kind {
            "component" => RecordKind::Component,
            "resource" => RecordKind::Resource,
            _ => RecordKind::Event,
        };
        let mut fields = Vec::new();
        for f in decl.fields {
            let k = if f.variants.is_empty() {
                Kind::parse(f.column).ok_or_else(|| {
                    format!(
                        "'{}.{}' has type {}, which og-core cannot store yet",
                        decl.schema, f.name, f.column
                    )
                })?
            } else {
                Kind::Enum
            };
            // A text field's capacity is the IDL's (`str.48`), a list's too
            // (`List.Post.8`).
            let capacity = (f.capacity > 0).then_some(f.capacity as usize);
            let list = (f.list > 0).then_some(f.list as usize);
            let ty = k
                .field_type(kind == RecordKind::Component, capacity, list)
                .map_err(|e| format!("'{}.{}': {e}", decl.schema, f.name))?;
            fields.push(FieldLayout {
                name: f.name.to_string(),
                kind: k,
                ty,
                default: serde_json::from_str(f.default).unwrap_or(Value::Null),
                readonly: f.readonly,
                tunable: f.tunable,
                min: f.min,
                max: f.max,
                step: f.step,
                unit: (!f.unit.is_empty()).then(|| f.unit.to_string()),
                label: (!f.label.is_empty()).then(|| f.label.to_string()),
                doc: f.doc.to_string(),
                variants: f.variants.iter().map(|v| v.to_string()).collect(),
            });
        }
        Ok(Layout {
            schema: decl.schema.to_string(),
            kind,
            fields: fields.into(),
            by_name: Default::default(),
        })
    }

    /// The layout of a type another module registered (og-input's
    /// `og.Input` and input events), read from the world's schema.
    pub fn from_schema(schema: &Schema, kind: RecordKind) -> Layout {
        Layout {
            schema: schema.name.clone(),
            kind,
            fields: schema
                .fields
                .iter()
                .map(|f| FieldLayout {
                    name: f.name.clone(),
                    kind: Kind::of(f.ty),
                    ty: f.ty,
                    default: Value::Null,
                    readonly: true,
                    tunable: f.tunable,
                    min: f.min,
                    max: f.max,
                    step: None,
                    unit: f.unit.clone(),
                    label: f.label.clone(),
                    doc: f.doc.clone(),
                    variants: f.variants.clone(),
                })
                .collect::<Vec<_>>()
                .into(),
            by_name: Default::default(),
        }
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
                    s.variants = f.variants.clone();
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

    /// The field named `key` and its byte offset in packed form, by binary
    /// search over the names (a per-frame reader's lookup).
    pub fn slot(&self, key: &str) -> Option<(&FieldLayout, usize)> {
        let index = self.by_name.get_or_init(|| {
            let mut at = 0;
            let mut index: Vec<(usize, usize)> = self
                .fields
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let slot = (i, at);
                    at += f.ty.size();
                    slot
                })
                .collect();
            index.sort_by(|a, b| self.fields[a.0].name.cmp(&self.fields[b.0].name));
            index
        });
        let k = index
            .binary_search_by(|(i, _)| self.fields[*i].name.as_str().cmp(key))
            .ok()?;
        let (i, at) = index[k];
        Some((&self.fields[i], at))
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
                ));
            }
        };
        if let Some(o) = obj
            && let Some(k) = o.keys().find(|k| self.field(k).is_none())
        {
            return Err(format!("'{}' has no field '{k}'", self.schema));
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
        for f in self.fields.iter() {
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

/// A record read field by field, as its JSON would answer: implemented by
/// the decoded [`Value`] and by [`Row`], the packed bytes read in place
/// (per-frame extraction reads a few fields of every sprite or row, where
/// decoding each record's whole JSON map was most of the cost).
pub trait Rec {
    /// `value[key].as_f64()`.
    fn num(&self, key: &str) -> Option<f64>;
    /// `value[key].as_i64()`.
    fn int(&self, key: &str) -> Option<i64>;
    /// `value[key].as_u64()`.
    fn uint(&self, key: &str) -> Option<u64>;
    /// `value[key]` as a flag: a bool as itself, a number as `!= 0`, else `None`.
    fn flag(&self, key: &str) -> Option<bool>;
    /// `value[key][lane].as_f64()` (a vector's `x`, a colour's `r`).
    fn lane(&self, key: &str, lane: &str) -> Option<f64>;
    /// `value[key].as_str()`.
    fn text(&self, key: &str) -> Option<std::borrow::Cow<'_, str>>;
}

impl Rec for Value {
    fn num(&self, key: &str) -> Option<f64> {
        self[key].as_f64()
    }
    fn int(&self, key: &str) -> Option<i64> {
        self[key].as_i64()
    }
    fn uint(&self, key: &str) -> Option<u64> {
        self[key].as_u64()
    }
    fn flag(&self, key: &str) -> Option<bool> {
        match &self[key] {
            Value::Bool(b) => Some(*b),
            Value::Number(n) => Some(n.as_f64().is_some_and(|x| x != 0.0)),
            _ => None,
        }
    }
    fn lane(&self, key: &str, lane: &str) -> Option<f64> {
        self[key][lane].as_f64()
    }
    fn text(&self, key: &str) -> Option<std::borrow::Cow<'_, str>> {
        self[key].as_str().map(std::borrow::Cow::Borrowed)
    }
}

/// A record's packed bytes read in place through its layout: each read
/// answers what the same read of [`Layout::decode`]'s JSON would (a
/// non-finite float is JSON's null; an enum is its variant's name).
#[derive(Clone, Copy)]
pub struct Row<'a> {
    pub layout: &'a Layout,
    pub bytes: &'a [u8],
}

/// A finite f64 as a JSON number answers it (`json!` makes NaN and the
/// infinities null).
fn finite(x: f64) -> Option<f64> {
    x.is_finite().then_some(x)
}

impl<'a> Row<'a> {
    pub fn new(layout: &'a Layout, bytes: &'a [u8]) -> Row<'a> {
        Row { layout, bytes }
    }

    /// The field named `key` and its bytes; `None` for a list, whose JSON
    /// is an array that none of these reads answers.
    fn field(&self, key: &str) -> Option<(&'a FieldLayout, &'a [u8])> {
        let (f, at) = self.layout.slot(key)?;
        if f.ty.is_list() {
            return None;
        }
        self.bytes.get(at..at + f.ty.size()).map(|b| (f, b))
    }

    /// A field as an integer when its JSON is one (an enum index past its
    /// variants is the number itself).
    fn integer(f: &FieldLayout, b: &[u8]) -> Option<i128> {
        let u32_at = || u32::from_le_bytes(b[..4].try_into().expect("4 bytes"));
        match f.kind {
            Kind::U32 => Some(i128::from(u32_at())),
            Kind::I32 => Some(i128::from(u32_at() as i32)),
            Kind::Enum => {
                let i = u32_at() as usize;
                (i >= f.variants.len()).then_some(i as i128)
            }
            Kind::Entity | Kind::Geometry | Kind::Material | Kind::Asset | Kind::Sound => Some(
                i128::from(u64::from_le_bytes(b[..8].try_into().expect("8 bytes"))),
            ),
            _ => None,
        }
    }
}

impl Rec for Row<'_> {
    fn num(&self, key: &str) -> Option<f64> {
        let (f, b) = self.field(key)?;
        match f.kind {
            Kind::F32 => finite(f64::from(f32_at(b, 0))),
            Kind::F64 => finite(f64::from_le_bytes(b[..8].try_into().expect("8 bytes"))),
            _ => Row::integer(f, b).map(|i| i as f64),
        }
    }
    fn int(&self, key: &str) -> Option<i64> {
        let (f, b) = self.field(key)?;
        Row::integer(f, b).and_then(|i| i64::try_from(i).ok())
    }
    fn uint(&self, key: &str) -> Option<u64> {
        let (f, b) = self.field(key)?;
        Row::integer(f, b).and_then(|i| u64::try_from(i).ok())
    }
    fn flag(&self, key: &str) -> Option<bool> {
        let (f, b) = self.field(key)?;
        match f.kind {
            Kind::Bool => Some(b.iter().any(|x| *x != 0)),
            Kind::F32 | Kind::F64 => self.num(key).map(|x| x != 0.0),
            _ => Row::integer(f, b).map(|i| i != 0),
        }
    }
    fn lane(&self, key: &str, lane: &str) -> Option<f64> {
        let (f, b) = self.field(key)?;
        let i = f.kind.keys().iter().position(|k| *k == lane)?;
        finite(f64::from(f32_at(b, i)))
    }
    fn text(&self, key: &str) -> Option<std::borrow::Cow<'_, str>> {
        let (f, b) = self.field(key)?;
        match f.kind {
            Kind::Str => Some(std::borrow::Cow::Borrowed(og_core::unpack_str(b))),
            Kind::Enum => {
                let i = u32::from_le_bytes(b[..4].try_into().expect("4 bytes")) as usize;
                f.variants
                    .get(i)
                    .map(|n| std::borrow::Cow::Borrowed(n.as_str()))
            }
            _ => None,
        }
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

/// A list's JSON (an array; `null` is the empty list) as its packed bytes.
fn encode_list(f: &FieldLayout, v: &Value, out: &mut Vec<u8>) -> Result<(), String> {
    let (_, capacity) = f.ty.list_of().expect("a list type");
    let items: &[Value] = match v {
        Value::Null => &[],
        Value::Array(items) => items,
        other => return Err(format!("a list is an array, got {other}")),
    };
    if items.len() > capacity {
        let kind = kind_label(f.kind);
        return Err(format!(
            "{} elements and the field holds {capacity}: remove {}, or declare the field list({kind}(), {}) or larger",
            items.len(),
            items.len() - capacity,
            items.len()
        ));
    }
    let mut lanes = Vec::with_capacity(items.len());
    for (i, item) in items.iter().enumerate() {
        let mut lane = Vec::with_capacity(8);
        encode_number(f.kind, item, &mut lane).map_err(|e| format!("element {i}: {e}"))?;
        lanes.push(lane);
    }
    let refs: Vec<&[u8]> = lanes.iter().map(Vec::as_slice).collect();
    out.extend_from_slice(&og_core::pack_list(f.ty, &refs)?);
    Ok(())
}

/// The factory name of a list's element kind, for messages.
fn kind_label(kind: Kind) -> &'static str {
    match kind {
        Kind::F32 => "f32",
        Kind::F64 => "f64",
        Kind::U32 => "u32",
        Kind::I32 => "i32",
        Kind::Entity => "Entity",
        Kind::Geometry => "Geometry",
        Kind::Material => "Material",
        Kind::Sound => "Sound",
        _ => "Asset",
    }
}

/// One number or handle lane.
fn encode_number(kind: Kind, v: &Value, out: &mut Vec<u8>) -> Result<(), String> {
    match kind {
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
        k if k.is_handle() => {
            let n = number(v, "a handle")?;
            if n.fract() != 0.0 || !(0.0..9_007_199_254_740_992.0).contains(&n) {
                return Err(format!("{n} is not a handle"));
            }
            out.extend_from_slice(&(n as u64).to_le_bytes())
        }
        _ => unreachable!("a number or a handle"),
    }
    Ok(())
}

fn encode_field(f: &FieldLayout, v: &Value, out: &mut Vec<u8>) -> Result<(), String> {
    if f.ty.is_list() {
        return encode_list(f, v, out);
    }
    match f.kind {
        Kind::F32 | Kind::F64 | Kind::U32 | Kind::I32 => encode_number(f.kind, v, out)?,
        Kind::Bool => {
            let b = match v {
                Value::Null => false,
                v => v
                    .as_bool()
                    .ok_or_else(|| format!("expected true or false, got {v}"))?,
            };
            out.push(u8::from(b))
        }
        Kind::Str => {
            let text = match v {
                Value::Null => "",
                v => v
                    .as_str()
                    .ok_or_else(|| format!("expected a string, got {v}"))?,
            };
            out.extend_from_slice(&og_core::pack_str(text, f.ty.size())?)
        }
        Kind::Enum => {
            let index = match v {
                Value::Null => 0,
                Value::String(name) => f
                    .variants
                    .iter()
                    .position(|x| x == name)
                    .ok_or_else(|| format!("'{name}' is not one of {:?}", f.variants))?,
                v => number(v, "an enum index")? as usize,
            };
            out.extend_from_slice(&(index as u32).to_le_bytes())
        }
        Kind::Entity | Kind::Geometry | Kind::Material | Kind::Asset | Kind::Sound => {
            encode_number(f.kind, v, out)?
        }
        Kind::Vector2 | Kind::Vector3 | Kind::Quaternion | Kind::Color => {
            f32_lanes(v, f.kind.keys(), out)?
        }
        Kind::Matrix4 => {
            let elements = v
                .get("elements")
                .and_then(Value::as_array)
                .filter(|a| a.len() == 16)
                .ok_or_else(|| format!("expected {{ elements: [16 numbers] }}, got {v}"))?;
            for e in elements {
                out.extend_from_slice(&(number(e, "an element")? as f32).to_le_bytes());
            }
        }
    }
    Ok(())
}

fn f32_at(b: &[u8], i: usize) -> f32 {
    f32::from_le_bytes(b[i * 4..i * 4 + 4].try_into().expect("4 bytes"))
}

fn decode_field(f: &FieldLayout, b: &[u8]) -> Value {
    if f.ty.is_list() {
        // The elements up to its length, each as its kind reads alone.
        let one = |e: &[u8]| match f.kind {
            Kind::F32 => json!(f32_at(e, 0) as f64),
            Kind::F64 => json!(f64::from_le_bytes(e[..8].try_into().expect("8 bytes"))),
            Kind::U32 => json!(u32::from_le_bytes(e[..4].try_into().expect("4 bytes"))),
            Kind::I32 => json!(i32::from_le_bytes(e[..4].try_into().expect("4 bytes"))),
            _ => json!(u64::from_le_bytes(e[..8].try_into().expect("8 bytes"))),
        };
        return Value::Array(og_core::list_elements(f.ty, b).map(one).collect());
    }
    match f.kind {
        Kind::F32 => json!(f32_at(b, 0) as f64),
        Kind::F64 => json!(f64::from_le_bytes(b[..8].try_into().expect("8 bytes"))),
        Kind::U32 => json!(u32::from_le_bytes(b[..4].try_into().expect("4 bytes"))),
        Kind::I32 => json!(i32::from_le_bytes(b[..4].try_into().expect("4 bytes"))),
        Kind::Bool => json!(b.iter().any(|x| *x != 0)),
        Kind::Str => json!(og_core::unpack_str(b)),
        Kind::Enum => {
            let i = u32::from_le_bytes(b[..4].try_into().expect("4 bytes")) as usize;
            f.variants.get(i).map_or_else(|| json!(i), |n| json!(n))
        }
        Kind::Entity | Kind::Geometry | Kind::Material | Kind::Asset | Kind::Sound => {
            json!(u64::from_le_bytes(b[..8].try_into().expect("8 bytes")))
        }
        Kind::Vector2 | Kind::Vector3 | Kind::Quaternion | Kind::Color => {
            let mut m = Map::new();
            for (i, k) in f.kind.keys().iter().enumerate() {
                m.insert(k.to_string(), json!(f32_at(b, i) as f64));
            }
            Value::Object(m)
        }
        Kind::Matrix4 => {
            json!({ "elements": (0..16).map(|i| f32_at(b, i) as f64).collect::<Vec<_>>() })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use og_api::host::{FieldDecl, RecordDecl};

    fn field(
        name: &'static str,
        column: &'static str,
        default: &'static str,
        capacity: u32,
    ) -> FieldDecl {
        FieldDecl {
            name,
            column,
            default,
            readonly: false,
            tunable: false,
            min: None,
            max: None,
            step: None,
            unit: "",
            label: "",
            doc: "",
            variants: &[],
            capacity,
            list: 0,
        }
    }

    /// An engine record's text field (`str.24` in the IDL) is stored at the
    /// IDL's capacity, and its default encodes and decodes as the text.
    #[test]
    fn a_row_reads_what_the_decoded_json_answers() {
        // Every engine record, at its defaults and with its bytes scrambled
        // (NaNs, infinities, enum indices past their variants, odd text).
        let mut seed = 0x9e37_79b9_7f4a_7c15_u64;
        let mut checked = 0;
        for decl in og_api::host::RECORDS {
            let Ok(layout) = Layout::from_decl(decl) else {
                continue;
            };
            let defaults = layout.encode(None, None).unwrap();
            let scrambled: Vec<u8> = (0..defaults.len())
                .map(|_| {
                    seed ^= seed << 13;
                    seed ^= seed >> 7;
                    seed ^= seed << 17;
                    seed as u8
                })
                .collect();
            for bytes in [&defaults, &scrambled] {
                let value = layout.decode(bytes);
                let row = Row::new(&layout, bytes);
                let names = layout.fields.iter().map(|f| f.name.as_str());
                for key in names.chain(["missing"]) {
                    let at = format!("{}.{key}", layout.schema);
                    assert_eq!(
                        row.num(key).map(f64::to_bits),
                        value.num(key).map(f64::to_bits),
                        "{at} num"
                    );
                    assert_eq!(row.int(key), value.int(key), "{at} int");
                    assert_eq!(row.uint(key), value.uint(key), "{at} uint");
                    assert_eq!(row.flag(key), value.flag(key), "{at} flag");
                    assert_eq!(row.text(key), value.text(key), "{at} text");
                    for lane in ["x", "y", "z", "w", "r", "g", "b", "elements"] {
                        assert_eq!(
                            row.lane(key, lane).map(f64::to_bits),
                            value.lane(key, lane).map(f64::to_bits),
                            "{at}.{lane}"
                        );
                    }
                    checked += 1;
                }
            }
        }
        assert!(checked > 500, "{checked} fields read");
    }

    #[test]
    fn an_engine_record_text_field_takes_the_idl_capacity() {
        let fields: &'static [FieldDecl] = Box::leak(Box::new([
            field("label", "str", "\"Ada\"", 24),
            field("size", "f32", "1", 0),
        ]));
        let decl = RecordDecl {
            kind: "component",
            name: "Tag",
            schema: "og.Tag",
            fields,
        };
        let layout = Layout::from_decl(&decl).unwrap();
        assert_eq!(layout.fields[0].ty, FieldType::Str(24));
        assert_eq!(layout.fields[1].ty, FieldType::F32);
        let bytes = layout.encode(None, None).unwrap();
        assert_eq!(bytes.len(), 24 + 4);
        assert_eq!(&bytes[..3], b"Ada");
        assert!(bytes[3..24].iter().all(|b| *b == 0));
    }

    /// A list field (`List.Post.8` in the IDL) is its element's kind at the
    /// IDL's capacity: an array of handles in and out, `[]` and `[0]` apart,
    /// refused past the capacity with the fix and never truncated.
    #[test]
    fn an_engine_record_list_field_takes_the_idl_capacity() {
        let mut passes = field("passes", "Post", "[]", 0);
        passes.list = 3;
        let mut weights = field("weights", "f32", "[]", 0);
        weights.list = 2;
        let fields: &'static [FieldDecl] = Box::leak(Box::new([passes, weights]));
        let decl = RecordDecl {
            kind: "resource",
            name: "Chain",
            schema: "og.Chain",
            fields,
        };
        let layout = Layout::from_decl(&decl).unwrap();
        assert_eq!(
            layout.fields[0].ty,
            FieldType::List(og_core::Scalar::U64, 3)
        );
        assert_eq!(
            layout.fields[1].ty,
            FieldType::List(og_core::Scalar::F32, 2)
        );
        let empty = layout.encode(None, None).unwrap();
        assert_eq!(empty.len(), 8 * 4 + 4 * 3);
        assert_eq!(
            layout.decode(&empty),
            json!({ "passes": [], "weights": [] })
        );
        let big = 4_294_967_298u64;
        let value = json!({ "passes": [big, 0], "weights": [0.5] });
        let bytes = layout.encode(None, Some(&value)).unwrap();
        assert_eq!(layout.decode(&bytes), value);
        assert_eq!(bytes[..8], 2u64.to_le_bytes());
        let zero = layout
            .encode(None, Some(&json!({ "passes": [0] })))
            .unwrap();
        assert_ne!(zero, empty, "[0] is not []");
        let long = layout
            .encode(Some(&bytes), Some(&json!({ "passes": [1, 2, 3, 4] })))
            .unwrap_err();
        assert!(
            long.contains("og.Chain.passes': 4 elements and the field holds 3")
                && long.contains("list(Asset(), 4)"),
            "{long}"
        );
        let bad = layout
            .encode(None, Some(&json!({ "weights": "x" })))
            .unwrap_err();
        assert!(bad.contains("a list is an array"), "{bad}");
        // A row reads no list field: its JSON is an array.
        let row = Row::new(&layout, &bytes);
        assert_eq!((row.num("passes"), row.uint("passes")), (None, None));
    }

    /// PostEffects.custom holds what og-render runs, so the two cannot drift.
    #[test]
    fn post_effects_custom_is_a_list_of_every_pass_og_render_runs() {
        let post = og_api::host::RECORDS
            .iter()
            .find(|r| r.schema == "og.PostEffects")
            .unwrap();
        let custom = post.fields.iter().find(|f| f.name == "custom").unwrap();
        assert_eq!(
            (custom.column, custom.list as usize),
            ("Post", og_render::MAX_CUSTOM_POSTS)
        );
        assert!(
            !post
                .fields
                .iter()
                .any(|f| f.name.starts_with("custom") && f.name != "custom")
        );
    }
}
