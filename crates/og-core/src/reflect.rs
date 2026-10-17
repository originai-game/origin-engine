//! Reflection: the single schema every component, resource and event has.
//!
//! A [`Schema`] names a layout (fields with a [`FieldType`]) and carries the
//! metadata the inspector, Tune, the probe's `bounds`, scene text and the wiki
//! read: docs, labels, units, ranges, defaults and the `tunable` flag. It is
//! produced by `#[derive(Reflect)]` in Rust, or built from JSON by a host that
//! declares a layout by name (`defineComponent` in TypeScript), and it is
//! exported as JSON with [`schemas_to_json`].
//!
//! Values move as **packed bytes**: the fields in schema order, each the
//! little-endian bytes of its type, no padding. [`Layout`] computes the field
//! ranges and converts a [`Reflect`] value to and from packed bytes.

use crate::error::{Error, ErrorKind, Result};
use crate::hash::StableHasher;
use serde::{Deserialize, Serialize};
use std::ops::Range;

/// The scalar a field is made of.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Scalar {
    F32,
    F64,
    I32,
    U32,
    I64,
    U64,
}

impl Scalar {
    pub const fn size(self) -> usize {
        match self {
            Scalar::F32 | Scalar::I32 | Scalar::U32 => 4,
            Scalar::F64 | Scalar::I64 | Scalar::U64 => 8,
        }
    }
}

/// The type of one reflected field. JSON spells it in lower case (`"vec3"`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    F32,
    F64,
    I32,
    U32,
    I64,
    U64,
    Vec2,
    Vec3,
    Vec4,
    Quat,
}

impl FieldType {
    pub const fn scalar(self) -> Scalar {
        match self {
            FieldType::F32 | FieldType::Vec2 | FieldType::Vec3 | FieldType::Vec4 => Scalar::F32,
            FieldType::Quat => Scalar::F32,
            FieldType::F64 => Scalar::F64,
            FieldType::I32 => Scalar::I32,
            FieldType::U32 => Scalar::U32,
            FieldType::I64 => Scalar::I64,
            FieldType::U64 => Scalar::U64,
        }
    }

    /// Scalars per value: 1, or 2/3/4 for vectors and quaternions.
    pub const fn lanes(self) -> usize {
        match self {
            FieldType::Vec2 => 2,
            FieldType::Vec3 => 3,
            FieldType::Vec4 | FieldType::Quat => 4,
            _ => 1,
        }
    }

    /// Bytes per value.
    pub const fn size(self) -> usize {
        self.scalar().size() * self.lanes()
    }

    /// The lower-case name used in JSON and in hashes.
    pub const fn name(self) -> &'static str {
        match self {
            FieldType::F32 => "f32",
            FieldType::F64 => "f64",
            FieldType::I32 => "i32",
            FieldType::U32 => "u32",
            FieldType::I64 => "i64",
            FieldType::U64 => "u64",
            FieldType::Vec2 => "vec2",
            FieldType::Vec3 => "vec3",
            FieldType::Vec4 => "vec4",
            FieldType::Quat => "quat",
        }
    }
}

mod sealed {
    pub trait Sealed {}
}

/// A Rust type that can be a reflected field. Sealed: the set is exactly the
/// [`FieldType`]s (`f32`, `f64`, `i32`, `u32`, `i64`, `u64`, `Vec2`, `Vec3`,
/// `Vec4`, `Quat`).
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a reflected field type",
    note = "fields of a #[derive(Reflect)] struct are f32, f64, i32, u32, i64, u64, Vec2, Vec3, Vec4 or Quat"
)]
pub trait FieldValue: Copy + bytemuck::Pod + sealed::Sealed + 'static {
    const TYPE: FieldType;

    fn zero() -> Self {
        bytemuck::Zeroable::zeroed()
    }

    fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }

    fn as_bytes_mut(&mut self) -> &mut [u8] {
        bytemuck::bytes_of_mut(self)
    }
}

macro_rules! field_value {
    ($($t:ty => $ft:ident),* $(,)?) => {$(
        impl sealed::Sealed for $t {}
        impl FieldValue for $t {
            const TYPE: FieldType = FieldType::$ft;
        }
    )*};
}

field_value! {
    f32 => F32, f64 => F64, i32 => I32, u32 => U32, i64 => I64, u64 => U64,
    glam::Vec2 => Vec2, glam::Vec3 => Vec3, glam::Vec4 => Vec4, glam::Quat => Quat,
}

/// One field of a [`Schema`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldSchema {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: FieldType,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub doc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(default)]
    pub tunable: bool,
    /// One number per lane, when the field has a declared default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<Vec<f64>>,
}

impl FieldSchema {
    /// A field with no metadata.
    pub fn new(name: impl Into<String>, ty: FieldType) -> Self {
        Self {
            name: name.into(),
            ty,
            doc: String::new(),
            label: None,
            unit: None,
            min: None,
            max: None,
            tunable: false,
            default: None,
        }
    }
}

/// The reflected description of a layout: a component, resource or event.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
    pub version: u32,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub doc: String,
    pub fields: Vec<FieldSchema>,
}

/// `[A-Za-z_][A-Za-z0-9_]*`, optionally dot-separated for type names.
fn is_identifier(s: &str, dots: bool) -> bool {
    let segment = |seg: &str| {
        let mut chars = seg.chars();
        matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
            && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    };
    if dots {
        s.split('.').all(segment)
    } else {
        segment(s)
    }
}

impl Schema {
    /// Checks names, ranges and defaults. Called by every registration.
    pub fn validate(&self) -> Result<()> {
        let err = |reason: String| Err(Error::new(ErrorKind::Schema, reason));
        if !is_identifier(&self.name, true) {
            return err(format!(
                "schema name '{}' is not an identifier (letters, digits, '_', dot-separated)",
                self.name
            ));
        }
        if self.version == 0 {
            return err(format!(
                "schema '{}' has version 0; versions start at 1",
                self.name
            ));
        }
        for (i, f) in self.fields.iter().enumerate() {
            let at = format!("{}.{}", self.name, f.name);
            if !is_identifier(&f.name, false) {
                return err(format!("field name '{at}' is not an identifier"));
            }
            if self.fields[..i].iter().any(|g| g.name == f.name) {
                return err(format!("field '{at}' is declared twice"));
            }
            for (what, v) in [("min", f.min), ("max", f.max)] {
                if v.is_some_and(|v| !v.is_finite()) {
                    return err(format!("field '{at}' has a non-finite {what}"));
                }
            }
            if let (Some(min), Some(max)) = (f.min, f.max) {
                if min > max {
                    return err(format!("field '{at}' has min {min} above max {max}"));
                }
            }
            if let Some(default) = &f.default {
                if default.len() != f.ty.lanes() {
                    return err(format!(
                        "field '{at}' of type {} needs {} default value(s), got {}",
                        f.ty.name(),
                        f.ty.lanes(),
                        default.len()
                    ));
                }
                for &v in default {
                    let below = f.min.is_some_and(|min| v < min);
                    let above = f.max.is_some_and(|max| v > max);
                    if !v.is_finite() || below || above {
                        return err(format!("field '{at}' default {v} is outside its range"));
                    }
                }
            }
        }
        Ok(())
    }

    /// Hash of what determines the bytes: name, version, field names and
    /// types. Docs, labels, units, ranges and defaults do not change it.
    pub fn layout_hash(&self) -> u64 {
        let mut h = StableHasher::new();
        h.write_str(&self.name);
        h.write_u32(self.version);
        h.write_u64(self.fields.len() as u64);
        for f in &self.fields {
            h.write_str(&f.name);
            h.write_str(f.ty.name());
        }
        h.finish()
    }

    pub fn field_index(&self, name: &str) -> Option<usize> {
        self.fields.iter().position(|f| f.name == name)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("a schema serializes")
    }

    /// Parses and validates one schema.
    pub fn from_json(text: &str) -> Result<Schema> {
        let schema: Schema = serde_json::from_str(text)
            .map_err(|e| Error::new(ErrorKind::Parse, format!("schema JSON: {e}")))?;
        schema.validate()?;
        Ok(schema)
    }
}

#[derive(Serialize)]
struct SchemaSet<'a> {
    format: &'static str,
    version: u32,
    types: &'a [Schema],
}

/// The JSON document `og schema` prints:
/// `{"format": "og-schema", "version": 1, "types": [...]}`.
pub fn schemas_to_json(schemas: &[Schema]) -> String {
    serde_json::to_string_pretty(&SchemaSet {
        format: "og-schema",
        version: 1,
        types: schemas,
    })
    .expect("schemas serialize")
}

/// A type with a reflected layout. Implemented by `#[derive(Reflect)]`.
///
/// `write_fields` calls `out(i, bytes)` once per field `i` with that field's
/// bytes; `read_fields` calls `input(i, buf)` once per field to fill it. Both
/// visit fields in schema order.
pub trait Reflect: Sized + Send + Sync + 'static {
    /// The schema name (the type name unless `#[reflect(name = "...")]`).
    const NAME: &'static str;

    fn schema() -> Schema;

    fn write_fields(&self, out: &mut dyn FnMut(usize, &[u8]));

    fn read_fields(input: &mut dyn FnMut(usize, &mut [u8])) -> Self;
}

/// Byte ranges of a schema's fields in packed form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    ranges: Vec<Range<usize>>,
    types: Vec<FieldType>,
    size: usize,
}

impl Layout {
    pub fn of(schema: &Schema) -> Layout {
        let mut ranges = Vec::with_capacity(schema.fields.len());
        let mut offset = 0;
        for f in &schema.fields {
            ranges.push(offset..offset + f.ty.size());
            offset += f.ty.size();
        }
        Layout {
            ranges,
            types: schema.fields.iter().map(|f| f.ty).collect(),
            size: offset,
        }
    }

    /// Packed size in bytes.
    pub fn size(&self) -> usize {
        self.size
    }

    pub fn field_count(&self) -> usize {
        self.ranges.len()
    }

    pub fn range(&self, field: usize) -> Range<usize> {
        self.ranges[field].clone()
    }

    pub fn field_type(&self, field: usize) -> FieldType {
        self.types[field]
    }

    /// `value` as packed bytes. Refused if its `write_fields` disagrees with
    /// this layout (a hand-written `Reflect` impl out of sync with its schema).
    pub fn encode<T: Reflect>(&self, value: &T) -> Result<Vec<u8>> {
        let mut out = vec![0u8; self.size];
        let mut written = vec![false; self.ranges.len()];
        let mut bad = None;
        value.write_fields(&mut |i, bytes| match self.ranges.get(i) {
            Some(r) if r.len() == bytes.len() && !written[i] => {
                out[r.clone()].copy_from_slice(bytes);
                written[i] = true;
            }
            _ => bad = Some(i),
        });
        if let Some(i) = bad.or_else(|| written.iter().position(|w| !w)) {
            return Err(Error::new(
                ErrorKind::Schema,
                format!("{}: field {i} does not match its schema", T::NAME),
            ));
        }
        Ok(out)
    }

    /// A value from packed bytes.
    pub fn decode<T: Reflect>(&self, bytes: &[u8]) -> Result<T> {
        if bytes.len() != self.size {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "{}: expected {} bytes, got {}",
                    T::NAME,
                    self.size,
                    bytes.len()
                ),
            ));
        }
        let mut bad = None;
        let value = T::read_fields(&mut |i, buf| match self.ranges.get(i) {
            Some(r) if r.len() == buf.len() => buf.copy_from_slice(&bytes[r.clone()]),
            _ => bad = Some(i),
        });
        match bad {
            Some(i) => Err(Error::new(
                ErrorKind::Schema,
                format!("{}: field {i} does not match its schema", T::NAME),
            )),
            None => Ok(value),
        }
    }
}

/// `value` as packed bytes in its own schema's layout.
pub fn to_bytes<T: Reflect>(value: &T) -> Result<Vec<u8>> {
    Layout::of(&T::schema()).encode(value)
}

/// A value from packed bytes in its own schema's layout.
pub fn from_bytes<T: Reflect>(bytes: &[u8]) -> Result<T> {
    Layout::of(&T::schema()).decode(bytes)
}
