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
//! | `ease` | 20: tag + four payload words | [`crate::ease::Ease`] | a `Uint32Array` of stride 5; text crosses the shared timing codec |
//! | `strN` (`str32`, `str64`, ...) | N, the capacity: 1 to [`STR_MAX_CAPACITY`] | [`Str<N>`] ([`Str64`] [`Str256`]) | a `Uint8Array` of stride N |
//! | `S[N]` (`u64[8]`, `f32[4]`) | (N + 1) × the element's: 1 to [`LIST_MAX_CAPACITY`] | [`List<T, N>`] | the element's typed array, of stride (N + 1) × the element's |
//!
//! **Lists** are fixed-capacity lists of one number type (`f32` `f64` `i32`
//! `u32` `i64` `u64`, the last also every handle) stored inline: a count
//! lane, then N element lanes, each lane the element's size. The count lane
//! holds the length as a value of the element type (`3.0` in an `f32[4]`,
//! `3` in a `u64[8]`), so `[]` and `[0]` differ, and the lanes past the length
//! are zero, so the bytes (the world's hash, snapshots) depend on the list
//! alone. The capacity is part of the type (`u64[8]`) and of the layout hash.
//! A list longer than its capacity is refused, never truncated. A list field
//! has no range, no declared default (it starts empty) and is not tunable.
//!
//! **Strings** are fixed-capacity UTF-8 stored inline: the text, then zeros
//! up to the capacity (no interior NUL), so a string is plain component
//! data. Its bytes, and so the world's hash and snapshots, depend on the text
//! alone. The capacity is part of the type (`str32`) and of the layout hash.
//! Text over the capacity is refused with the fix named ([`pack_str`]), never
//! truncated. A string field has no range, no declared default (it starts
//! empty) and is not tunable. **Enums** are a `u32` index; the variant names live in the
//! schema ([`FieldSchema::variants`]) and in the layout hash, so reordering
//! variants is a layout change. **Bools** are one byte; the byte entry points
//! ([`Schema::check_packed`]) refuse anything but 0 and 1, while a column view
//! reads any non-zero byte as true; a list's count past its capacity, or a
//! non-zero lane past its length, is refused there too.

use crate::error::{Error, ErrorKind, Result};
use crate::hash::StableHasher;
use serde::{Deserialize, Serialize};
use std::ops::Range;

mod field_type;
mod field_value;

pub use field_type::{
    FieldType, LIST_MAX_CAPACITY, STR_MAX_CAPACITY, Scalar, list_elements, list_len, pack_list,
    pack_str, unpack_str,
};
pub use field_value::{Bool8, FieldValue, List, ListElement, ReflectField, Str, Str64, Str256};

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
    /// Where the field's numbers lie, for og.WorldOrigin's shift (R12): a
    /// [`Worldspace`] in its text form (`world`, `root xz`, `none`); `None`
    /// on a field that is not a coordinate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worldspace: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(default)]
    pub tunable: bool,
    /// One number per lane, when the field has a declared default (an
    /// enum's is the variant index, a bool's 0 or 1; strings and lists have
    /// none).
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
            worldspace: None,
            min: None,
            max: None,
            tunable: false,
            default: None,
            variants: Vec::new(),
        }
    }

    /// The field's [`Worldspace`], `None` without one (or with one that does
    /// not parse for its type, which declaration refuses).
    pub fn space(&self) -> Option<Worldspace> {
        Worldspace::of(self.worldspace.as_deref()?, self.ty).ok()
    }

    /// An `enum` field with these variant names.
    pub fn enumeration(name: impl Into<String>, variants: &[&str]) -> Self {
        let mut f = FieldSchema::new(name, FieldType::Enum);
        f.variants = variants.iter().map(|v| (*v).to_owned()).collect();
        f
    }

    /// The field's default as packed bytes: its declared `default`, or zeros
    /// (`0`, `false`, the first variant, the empty string, the empty list).
    pub fn default_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.ty.size());
        let declared = !self.ty.is_string() && !self.ty.is_list();
        match &self.default {
            Some(values) if declared && values.len() == self.ty.lanes() => {
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
    /// UTF-8 followed only by zeros, a list's count is a whole number up to
    /// its capacity with zeros past it. Other types accept every bit pattern.
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
            FieldType::Ease => crate::ease::Ease::from_bytes(bytes)
                .map(|_| ())
                .ok_or_else(|| {
                    "invalid ease tag, controls, step count/position or noncanonical payload".into()
                }),
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
            FieldType::Str(_) => {
                let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
                if bytes[end..].iter().any(|&b| b != 0) {
                    return Err("a string is followed by non-zero bytes after its end".into());
                }
                std::str::from_utf8(&bytes[..end])
                    .map(|_| ())
                    .map_err(|_| "a string that is not UTF-8".into())
            }
            FieldType::List(element, capacity) => {
                let lane = element.size();
                let n = list_len(self.ty, bytes).ok_or_else(|| {
                    format!("a list's count lane is not a whole number from 0 to {capacity}")
                })?;
                if bytes[lane * (n + 1)..].iter().any(|&b| b != 0) {
                    return Err(format!(
                        "a list of {n} element(s) has non-zero lanes past its length"
                    ));
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

/// How a field's coordinates move when og.WorldOrigin shifts the world
/// (R12, `docs/design.md` §2.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Frame {
    /// A point in the world: every shift moves it.
    World,
    /// In the frame of its entity's Transform: a point in the world on an
    /// entity without og.Parent (moved), relative to the parent on a child
    /// (left).
    Root,
    /// A point in the world under a condition its module knows (og.Joint's
    /// otherAnchor while `other` is 0): the module's state block moves it.
    Hook,
    /// Not a place: an offset, a size, a direction, a screen point.
    None,
}

/// A field's [`Frame`] and the world axis of each of its lanes: the
/// `worldspace` attribute of a Vector2, Vector3, f32 or f64 field, written
/// `<frame> [axes]`: `world`, `root`, `hook` or `none`, then one of `x`,
/// `y`, `z` per lane (a Vector3's default `xyz`, a Vector2's `xy`; a
/// number names its axis, `world y` for an absolute height).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Worldspace {
    pub frame: Frame,
    /// Per lane, the world axis it moves along (0 x, 1 y, 2 z).
    pub axes: [u8; 3],
    pub lanes: u8,
}

impl Worldspace {
    /// Parses the attribute's text for a field of `lanes` lanes (1 to 3).
    pub fn parse(text: &str, lanes: usize) -> std::result::Result<Worldspace, String> {
        let mut words = text.split_whitespace();
        let frame = match words.next() {
            Some("world") => Frame::World,
            Some("root") => Frame::Root,
            Some("hook") => Frame::Hook,
            Some("none") => Frame::None,
            other => {
                return Err(format!(
                    "worldspace `{}`: expected world, root, hook or none",
                    other.unwrap_or("")
                ));
            }
        };
        let named = words.next();
        if words.next().is_some() {
            return Err(format!(
                "worldspace `{text}`: a frame, then at most its axes"
            ));
        }
        let axes = match (named, lanes) {
            (Some(a), _) => a,
            (None, 3) => "xyz",
            (None, 2) => "xy",
            (None, _) if frame == Frame::None => "x",
            (None, _) => {
                return Err(format!(
                    "worldspace `{text}`: a number names its axis (`{} y` for a height)",
                    text.trim()
                ));
            }
        };
        if !(1..=3).contains(&lanes) || axes.len() != lanes {
            return Err(format!(
                "worldspace `{text}`: {} axes for a field of {lanes} lane(s)",
                axes.len()
            ));
        }
        let mut out = [0u8; 3];
        for (i, c) in axes.chars().enumerate() {
            out[i] = match c {
                'x' => 0,
                'y' => 1,
                'z' => 2,
                _ => return Err(format!("worldspace `{text}`: an axis is x, y or z")),
            };
            if out[..i].contains(&out[i]) {
                return Err(format!("worldspace `{text}`: axis `{c}` twice"));
            }
        }
        Ok(Worldspace {
            frame,
            axes: out,
            lanes: lanes as u8,
        })
    }

    /// Parses the attribute for a field of type `ty`: a Vector2, Vector3,
    /// f32 or f64, refused on any other.
    pub fn of(text: &str, ty: FieldType) -> std::result::Result<Worldspace, String> {
        let lanes = match ty {
            FieldType::F32 | FieldType::F64 => 1,
            FieldType::Vec2 => 2,
            FieldType::Vec3 => 3,
            _ => {
                return Err(format!(
                    "worldspace `{text}` is for a Vector2, Vector3, f32 or f64 field"
                ));
            }
        };
        Worldspace::parse(text, lanes)
    }

    /// Whether a shift moves the field on an entity with (`child`) or
    /// without a parent: `world` always, `root` on a root only.
    pub fn moves(&self, child: bool) -> bool {
        match self.frame {
            Frame::World => true,
            Frame::Root => !child,
            Frame::Hook | Frame::None => false,
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

/// One field of a schema as `#[derive(Reflect)]` declares it: static data
/// that [`Schema::declared`] turns into a [`FieldSchema`], so the code that
/// builds a schema is compiled once rather than once per reflected type.
#[doc(hidden)]
pub struct FieldDecl {
    pub name: &'static str,
    pub ty: FieldType,
    pub doc: &'static str,
    pub label: Option<&'static str>,
    pub unit: Option<&'static str>,
    pub worldspace: Option<&'static str>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub tunable: bool,
    pub default: Option<&'static [f64]>,
    pub variants: &'static [&'static str],
}

impl Schema {
    /// The schema of a derived type from its declared fields.
    #[doc(hidden)]
    #[inline(never)]
    pub fn declared(name: &str, version: u32, doc: &str, fields: &[FieldDecl]) -> Schema {
        Schema {
            name: name.to_owned(),
            version,
            doc: doc.to_owned(),
            fields: fields
                .iter()
                .map(|f| FieldSchema {
                    name: f.name.to_owned(),
                    ty: f.ty,
                    doc: f.doc.to_owned(),
                    label: f.label.map(str::to_owned),
                    unit: f.unit.map(str::to_owned),
                    worldspace: f.worldspace.map(str::to_owned),
                    min: f.min,
                    max: f.max,
                    tunable: f.tunable,
                    default: f.default.map(<[f64]>::to_vec),
                    variants: f.variants.iter().map(|v| (*v).to_owned()).collect(),
                })
                .collect(),
        }
    }
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
            if let Some(w) = &f.worldspace
                && let Err(e) = Worldspace::of(w, f.ty)
            {
                return err(format!("field '{at}': {e}"));
            }
            if f.ty == FieldType::Enum {
                if f.variants.is_empty() {
                    return err(format!("enum field '{at}' declares no variants"));
                }
                for (k, v) in f.variants.iter().enumerate() {
                    if !is_identifier(v, true) {
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
            if f.ty == FieldType::Ease {
                if f.min.is_some() || f.max.is_some() || f.tunable {
                    return err(format!(
                        "ease field '{at}' takes a timing value, not a numeric range or Tune slider"
                    ));
                }
                if let Some(default) = &f.default {
                    if default
                        .iter()
                        .any(|v| v.fract() != 0.0 || !(0.0..=u32::MAX as f64).contains(v))
                    {
                        return err(format!(
                            "ease field '{at}' defaults must be canonical payload words"
                        ));
                    }
                    f.check_value(&f.default_bytes())
                        .map_err(|e| Error::new(ErrorKind::Schema, format!("field '{at}': {e}")))?;
                }
            }
            if let Some(capacity) = f.ty.capacity() {
                if !(1..=STR_MAX_CAPACITY).contains(&capacity) {
                    return err(format!(
                        "string field '{at}' has a capacity of {capacity}; a string field holds 1 to {STR_MAX_CAPACITY} bytes"
                    ));
                }
                if f.default.is_some() {
                    return err(format!(
                        "string field '{at}' has a default; a string field starts empty"
                    ));
                }
                if f.min.is_some() || f.max.is_some() {
                    return err(format!(
                        "string field '{at}' has a range; only numeric fields have one (cap the text with the capacity)"
                    ));
                }
                if f.tunable {
                    return err(format!(
                        "string field '{at}' is tunable; Tune and the probe's bounds publish numbers only (set the text with world.set or the probe's set:)"
                    ));
                }
            }
            if let Some((element, capacity)) = f.ty.list_of() {
                if !element.is_list_element() || !(1..=LIST_MAX_CAPACITY).contains(&capacity) {
                    return err(format!(
                        "list field '{at}' is {}; a list holds 1 to {LIST_MAX_CAPACITY} f32, f64, i32, u32, i64 or u64",
                        f.ty.name()
                    ));
                }
                if f.default.is_some() {
                    return err(format!(
                        "list field '{at}' has a default; a list field starts empty"
                    ));
                }
                if f.min.is_some() || f.max.is_some() {
                    return err(format!(
                        "list field '{at}' has a range; only a single number has one"
                    ));
                }
                if f.tunable {
                    return err(format!(
                        "list field '{at}' is tunable; Tune and the probe's bounds publish single numbers only (set the whole list with world.set or the probe's set:)"
                    ));
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
            h.write_str(&f.ty.name());
            if f.ty == FieldType::Enum {
                h.write_u64(f.variants.len() as u64);
                for v in &f.variants {
                    h.write_str(v);
                }
            }
        }
        h.finish()
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

/// A doc string as `#[derive(Reflect)]` writes it into a [`Schema`]: the
/// text, or empty with og-core's `publish` feature (a published game's
/// build, where nothing reads the docs), so the text is not linked there.
#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "publish"))]
macro_rules! reflect_doc {
    ($doc:literal) => {
        $doc
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "publish")]
macro_rules! reflect_doc {
    ($doc:literal) => {
        ""
    };
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
        if self.types.contains(&FieldType::Ease) {
            T::schema().check_packed(bytes)?;
        }
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

#[cfg(test)]
mod reflect_contracts;
