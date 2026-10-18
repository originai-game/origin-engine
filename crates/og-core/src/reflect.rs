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
//!
//! Field types:
//!
//! | type | bytes | Rust | a column is |
//! |---|---|---|---|
//! | `f32` `f64` `i32` `u32` `i64` `u64` | 4 or 8 | the same | a typed array |
//! | `vec2` `vec3` `vec4` `quat` | 8, 12, 16, 16 | `Vec2` `Vec3` `Vec4` `Quat` | a `Float32Array` of stride 2, 3, 4, 4 |
//! | `mat4` | 64 | `Mat4` (column-major) | a `Float32Array` of stride 16 |
//! | `bool` | 1: `0` or `1` | `bool`, or [`Bool8`] for column views | a `Uint8Array` |
//! | `enum` | 4: a `u32` index into the field's `variants` | `u32` with `#[reflect(variants = [..])]` | a `Uint32Array` |
//! | `str64` `str256` | 64 or 256 | [`Str64`] [`Str256`] | never: events only |
//!
//! **Strings are for events only.** Components are viewed in place as typed
//! arrays and resources are polled as values, while events are copied, so a
//! bounded inline string is carried by events alone: UTF-8, zero-padded to
//! its capacity, the text being the bytes before the first zero (no interior
//! NUL). The world refuses a type with a string field as a component or a
//! resource. **Enums** are a `u32` index; the variant names live in the
//! schema ([`FieldSchema::variants`]) and in the layout hash, so reordering
//! variants is a layout change. **Bools** are one byte; the byte entry points
//! ([`Schema::check_packed`]) refuse anything but 0 and 1, while a column view
//! reads any non-zero byte as true.

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
    /// One byte, 0 or 1.
    Bool,
    /// One byte of UTF-8 text (the lanes of a bounded string).
    Utf8,
}

impl Scalar {
    pub const fn size(self) -> usize {
        match self {
            Scalar::Bool | Scalar::Utf8 => 1,
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
    /// One byte, 0 or 1 (a `Uint8Array` column).
    Bool,
    /// Sixteen f32, column-major (a `Float32Array` column of stride 16).
    Mat4,
    /// A `u32` index into [`FieldSchema::variants`].
    Enum,
    /// UTF-8 text of at most 64 bytes, zero-padded. Events only.
    Str64,
    /// UTF-8 text of at most 256 bytes, zero-padded. Events only.
    Str256,
}

impl FieldType {
    pub const fn scalar(self) -> Scalar {
        match self {
            FieldType::F32 | FieldType::Vec2 | FieldType::Vec3 | FieldType::Vec4 => Scalar::F32,
            FieldType::Quat | FieldType::Mat4 => Scalar::F32,
            FieldType::F64 => Scalar::F64,
            FieldType::I32 => Scalar::I32,
            FieldType::U32 | FieldType::Enum => Scalar::U32,
            FieldType::I64 => Scalar::I64,
            FieldType::U64 => Scalar::U64,
            FieldType::Bool => Scalar::Bool,
            FieldType::Str64 | FieldType::Str256 => Scalar::Utf8,
        }
    }

    /// Scalars per value: 1, 2/3/4 for vectors and quaternions, 16 for a
    /// matrix, the byte capacity (64, 256) for a string.
    pub const fn lanes(self) -> usize {
        match self {
            FieldType::Vec2 => 2,
            FieldType::Vec3 => 3,
            FieldType::Vec4 | FieldType::Quat => 4,
            FieldType::Mat4 => 16,
            FieldType::Str64 => 64,
            FieldType::Str256 => 256,
            _ => 1,
        }
    }

    /// Whether this is a bounded string (`str64`, `str256`), which only
    /// events carry.
    pub const fn is_string(self) -> bool {
        matches!(self, FieldType::Str64 | FieldType::Str256)
    }

    /// Whether packed bytes of this type have values outside the type (a
    /// bool byte other than 0 or 1, an enum index past its variants, a string
    /// that is not zero-padded UTF-8), which [`Schema::check_packed`] refuses.
    pub const fn is_checked(self) -> bool {
        matches!(
            self,
            FieldType::Bool | FieldType::Enum | FieldType::Str64 | FieldType::Str256
        )
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
            FieldType::Bool => "bool",
            FieldType::Mat4 => "mat4",
            FieldType::Enum => "enum",
            FieldType::Str64 => "str64",
            FieldType::Str256 => "str256",
        }
    }
}

mod sealed {
    pub trait Sealed {}
    pub trait SealedField {}
}

/// A Rust type a column can be viewed as (`ArchetypeView::read::<T>`): plain
/// old data whose every bit pattern is a value. Sealed: `f32`, `f64`, `i32`,
/// `u32`, `i64`, `u64`, `Vec2`, `Vec3`, `Vec4`, `Quat`, `Mat4` and [`Bool8`]
/// (a bool column). Every `FieldValue` is a [`ReflectField`]; `bool`,
/// [`Str64`] and [`Str256`] are fields but not views.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a column element type",
    note = "a column is viewed as f32, f64, i32, u32, i64, u64, Vec2, Vec3, Vec4, Quat, Mat4 or Bool8 (for a bool column)"
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
    glam::Mat4 => Mat4, Bool8 => Bool,
}

/// One element of a `bool` column as a view sees it: the byte itself. `0` is
/// false and anything else true; the engine writes 0 or 1.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(transparent)]
pub struct Bool8(pub u8);

impl Bool8 {
    pub const FALSE: Bool8 = Bool8(0);
    pub const TRUE: Bool8 = Bool8(1);

    pub const fn get(self) -> bool {
        self.0 != 0
    }

    pub fn set(&mut self, on: bool) {
        self.0 = u8::from(on);
    }
}

impl From<bool> for Bool8 {
    fn from(on: bool) -> Self {
        Bool8(u8::from(on))
    }
}

impl From<Bool8> for bool {
    fn from(b: Bool8) -> Self {
        b.get()
    }
}

impl std::fmt::Debug for Bool8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}

/// A Rust type a `#[derive(Reflect)]` field may have: every [`FieldValue`],
/// plus `bool` (a `bool` field), [`Str64`] and [`Str256`] (string fields, for
/// events). An enum field is a `u32` with `#[reflect(variants = [...])]`.
/// Sealed. The two methods are what the derive generates calls to.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a reflected field type",
    note = "fields of a #[derive(Reflect)] struct are f32, f64, i32, u32, i64, u64, Vec2, Vec3, Vec4, Quat, Mat4, bool, Bool8, Str64 or Str256 (an enum is a u32 with #[reflect(variants = [..])])"
)]
pub trait ReflectField: Copy + Send + Sync + sealed::SealedField + 'static {
    const TYPE: FieldType;

    /// Calls `out` once with the packed bytes of the value.
    #[doc(hidden)]
    fn write_packed(&self, out: &mut dyn FnMut(&[u8]));

    /// A value from `input`, called once to fill the packed bytes.
    #[doc(hidden)]
    fn read_packed(input: &mut dyn FnMut(&mut [u8])) -> Self;
}

impl<T: FieldValue + Send + Sync> sealed::SealedField for T {}

impl<T: FieldValue + Send + Sync> ReflectField for T {
    const TYPE: FieldType = <T as FieldValue>::TYPE;

    fn write_packed(&self, out: &mut dyn FnMut(&[u8])) {
        out(self.as_bytes());
    }

    fn read_packed(input: &mut dyn FnMut(&mut [u8])) -> Self {
        let mut v = <T as FieldValue>::zero();
        input(v.as_bytes_mut());
        v
    }
}

impl sealed::SealedField for bool {}

impl ReflectField for bool {
    const TYPE: FieldType = FieldType::Bool;

    fn write_packed(&self, out: &mut dyn FnMut(&[u8])) {
        out(&[u8::from(*self)]);
    }

    fn read_packed(input: &mut dyn FnMut(&mut [u8])) -> Self {
        let mut b = [0u8];
        input(&mut b);
        b[0] != 0
    }
}

macro_rules! inline_string {
    ($(#[$doc:meta])* $name:ident, $cap:literal, $ft:ident) => {
        $(#[$doc])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name([u8; $cap]);

        impl $name {
            /// The capacity in bytes of UTF-8.
            pub const CAPACITY: usize = $cap;

            /// The empty string.
            pub const EMPTY: $name = $name([0; $cap]);

            /// `text` as an inline string. Refused when it is longer than
            /// the capacity in UTF-8 bytes or holds a NUL character.
            pub fn new(text: &str) -> Result<$name> {
                let bytes = text.as_bytes();
                if bytes.len() > $cap {
                    return Err(Error::new(
                        ErrorKind::Invalid,
                        format!(
                            "'{text}' is {} bytes of UTF-8; a {} holds at most {}",
                            bytes.len(),
                            FieldType::$ft.name(),
                            $cap
                        ),
                    ));
                }
                if bytes.contains(&0) {
                    return Err(Error::new(
                        ErrorKind::Invalid,
                        format!("a {} cannot hold a NUL character", FieldType::$ft.name()),
                    ));
                }
                let mut out = [0u8; $cap];
                out[..bytes.len()].copy_from_slice(bytes);
                Ok($name(out))
            }

            /// The text: the bytes before the first zero (the longest valid
            /// UTF-8 prefix of them, for bytes that never passed a check).
            pub fn as_str(&self) -> &str {
                let end = self.0.iter().position(|&b| b == 0).unwrap_or($cap);
                match std::str::from_utf8(&self.0[..end]) {
                    Ok(s) => s,
                    Err(e) => std::str::from_utf8(&self.0[..e.valid_up_to()]).unwrap_or_default(),
                }
            }

            /// The packed bytes.
            pub fn as_bytes(&self) -> &[u8; $cap] {
                &self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                $name::EMPTY
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(self.as_str(), f)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl sealed::SealedField for $name {}

        impl ReflectField for $name {
            const TYPE: FieldType = FieldType::$ft;

            fn write_packed(&self, out: &mut dyn FnMut(&[u8])) {
                out(&self.0);
            }

            fn read_packed(input: &mut dyn FnMut(&mut [u8])) -> Self {
                let mut b = [0u8; $cap];
                input(&mut b);
                $name(b)
            }
        }
    };
}

inline_string!(
    /// A `str64` field: UTF-8 text of at most 64 bytes stored inline,
    /// zero-padded. Events only (see the module docs).
    Str64,
    64,
    Str64
);

inline_string!(
    /// A `str256` field: UTF-8 text of at most 256 bytes stored inline,
    /// zero-padded. Events only (see the module docs).
    Str256,
    256,
    Str256
);

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
    /// One number per lane, when the field has a declared default (an
    /// enum's is the variant index, a bool's 0 or 1; strings have none).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<Vec<f64>>,
    /// The variant names of an `enum` field, by index; empty for every other
    /// type.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<String>,
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
            variants: Vec::new(),
        }
    }

    /// An `enum` field with these variant names.
    pub fn enumeration(name: impl Into<String>, variants: &[&str]) -> Self {
        let mut f = FieldSchema::new(name, FieldType::Enum);
        f.variants = variants.iter().map(|v| (*v).to_owned()).collect();
        f
    }

    /// The field's default as packed bytes: its declared `default`, or zeros
    /// (`0`, `false`, the first variant, the empty string).
    pub fn default_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.ty.size());
        match &self.default {
            Some(values) if !self.ty.is_string() && values.len() == self.ty.lanes() => {
                for &v in values {
                    match self.ty.scalar() {
                        Scalar::F32 => out.extend_from_slice(&(v as f32).to_le_bytes()),
                        Scalar::F64 => out.extend_from_slice(&v.to_le_bytes()),
                        Scalar::I32 => out.extend_from_slice(&(v as i32).to_le_bytes()),
                        Scalar::U32 => out.extend_from_slice(&(v as u32).to_le_bytes()),
                        Scalar::I64 => out.extend_from_slice(&(v as i64).to_le_bytes()),
                        Scalar::U64 => out.extend_from_slice(&(v as u64).to_le_bytes()),
                        Scalar::Bool | Scalar::Utf8 => out.push(u8::from(v != 0.0)),
                    }
                }
            }
            _ => out.resize(self.ty.size(), 0),
        }
        out
    }

    /// `Ok` when `bytes` (this field's packed bytes) hold a value of the
    /// type: a bool is 0 or 1, an enum index names a variant, a string is
    /// UTF-8 followed only by zeros. Other types accept every bit pattern.
    pub fn check_value(&self, bytes: &[u8]) -> std::result::Result<(), String> {
        if bytes.len() != self.ty.size() {
            return Err(format!(
                "{} bytes for a {} of {}",
                bytes.len(),
                self.ty.name(),
                self.ty.size()
            ));
        }
        match self.ty {
            FieldType::Bool if bytes[0] > 1 => Err(format!("bool byte {}, not 0 or 1", bytes[0])),
            FieldType::Enum => {
                let i = u32::from_le_bytes(bytes.try_into().expect("4 bytes")) as usize;
                if i < self.variants.len() {
                    Ok(())
                } else {
                    Err(format!(
                        "enum index {i}, but there are {} variants",
                        self.variants.len()
                    ))
                }
            }
            FieldType::Str64 | FieldType::Str256 => {
                let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
                if bytes[end..].iter().any(|&b| b != 0) {
                    return Err("a string is followed by non-zero bytes after its end".into());
                }
                std::str::from_utf8(&bytes[..end])
                    .map(|_| ())
                    .map_err(|_| "a string that is not UTF-8".into())
            }
            _ => Ok(()),
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
            if let (Some(min), Some(max)) = (f.min, f.max)
                && min > max
            {
                return err(format!("field '{at}' has min {min} above max {max}"));
            }
            if f.ty == FieldType::Enum {
                if f.variants.is_empty() {
                    return err(format!("enum field '{at}' declares no variants"));
                }
                for (k, v) in f.variants.iter().enumerate() {
                    if !is_identifier(v, false) {
                        return err(format!(
                            "enum field '{at}' has variant '{v}', which is not an identifier"
                        ));
                    }
                    if f.variants[..k].contains(v) {
                        return err(format!("enum field '{at}' declares variant '{v}' twice"));
                    }
                }
            } else if !f.variants.is_empty() {
                return err(format!(
                    "field '{at}' of type {} lists variants; only an enum field has them",
                    f.ty.name()
                ));
            }
            if f.ty.is_string() && f.default.is_some() {
                return err(format!(
                    "string field '{at}' has a default; a string field starts empty"
                ));
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
                let v = default[0];
                let valid = match f.ty {
                    FieldType::Bool => v == 0.0 || v == 1.0,
                    FieldType::Enum => v.fract() == 0.0 && v >= 0.0 && v < f.variants.len() as f64,
                    _ => true,
                };
                if !valid {
                    return err(format!(
                        "field '{at}' default {v} is not a valid {} value",
                        f.ty.name()
                    ));
                }
            }
        }
        Ok(())
    }

    /// Hash of what determines the bytes and their meaning: name, version,
    /// field names and types, and an enum field's variant names in order.
    /// Docs, labels, units, ranges and defaults do not change it.
    pub fn layout_hash(&self) -> u64 {
        let mut h = StableHasher::new();
        h.write_str(&self.name);
        h.write_u32(self.version);
        h.write_u64(self.fields.len() as u64);
        for f in &self.fields {
            h.write_str(&f.name);
            h.write_str(f.ty.name());
            if f.ty == FieldType::Enum {
                h.write_u64(f.variants.len() as u64);
                for v in &f.variants {
                    h.write_str(v);
                }
            }
        }
        h.finish()
    }

    /// Whether any field is a string (`str64`, `str256`): such a type is
    /// carried by events only.
    pub fn has_strings(&self) -> bool {
        self.fields.iter().any(|f| f.ty.is_string())
    }

    /// Packed bytes of every field's default ([`FieldSchema::default_bytes`]).
    pub fn default_bytes(&self) -> Vec<u8> {
        self.fields.iter().flat_map(|f| f.default_bytes()).collect()
    }

    /// `Ok` when `bytes` is a packed value of this schema: the right size,
    /// and every bool, enum and string field holding a value of its type
    /// ([`FieldSchema::check_value`]). The reason names the field.
    pub fn check_packed(&self, bytes: &[u8]) -> Result<()> {
        let size: usize = self.fields.iter().map(|f| f.ty.size()).sum();
        if bytes.len() != size {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "'{}' is {size} bytes packed, got {}",
                    self.name,
                    bytes.len()
                ),
            ));
        }
        let mut at = 0;
        for f in &self.fields {
            let n = f.ty.size();
            if f.ty.is_checked() {
                f.check_value(&bytes[at..at + n]).map_err(|reason| {
                    Error::new(
                        ErrorKind::Invalid,
                        format!("{}.{}: {reason}", self.name, f.name),
                    )
                })?;
            }
            at += n;
        }
        Ok(())
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
