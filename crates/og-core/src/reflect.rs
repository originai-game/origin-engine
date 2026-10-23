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
    /// One byte of UTF-8 text (the lanes of a fixed-capacity string).
    Utf8,
}

impl Scalar {
    /// The scalar's field-type name (`f32`, `u64`); `bool` and `utf8` for
    /// the two byte scalars.
    pub const fn name(self) -> &'static str {
        match self {
            Scalar::F32 => "f32",
            Scalar::F64 => "f64",
            Scalar::I32 => "i32",
            Scalar::U32 => "u32",
            Scalar::I64 => "i64",
            Scalar::U64 => "u64",
            Scalar::Bool => "bool",
            Scalar::Utf8 => "utf8",
        }
    }

    /// The single-lane field type of this scalar: a list element read alone
    /// (`u64` for a handle). A UTF-8 byte is a `str1`.
    pub const fn field_type(self) -> FieldType {
        match self {
            Scalar::F32 => FieldType::F32,
            Scalar::F64 => FieldType::F64,
            Scalar::I32 => FieldType::I32,
            Scalar::U32 => FieldType::U32,
            Scalar::I64 => FieldType::I64,
            Scalar::U64 => FieldType::U64,
            Scalar::Bool => FieldType::Bool,
            Scalar::Utf8 => FieldType::Str(1),
        }
    }

    /// Whether a list may hold it: the six number types (a handle is a `u64`).
    pub const fn is_list_element(self) -> bool {
        !matches!(self, Scalar::Bool | Scalar::Utf8)
    }

    /// `n` as one lane of this scalar: the count lane of a list.
    fn count_lane(self, n: usize, out: &mut [u8]) {
        match self {
            Scalar::F32 => out.copy_from_slice(&(n as f32).to_le_bytes()),
            Scalar::F64 => out.copy_from_slice(&(n as f64).to_le_bytes()),
            Scalar::I32 | Scalar::U32 => out.copy_from_slice(&(n as u32).to_le_bytes()),
            Scalar::I64 | Scalar::U64 => out.copy_from_slice(&(n as u64).to_le_bytes()),
            Scalar::Bool | Scalar::Utf8 => out[0] = n as u8,
        }
    }

    /// The whole number one lane of this scalar holds; `None` for a
    /// fraction, a negative, a NaN or a number past `u32`.
    fn whole_lane(self, b: &[u8]) -> Option<u64> {
        let w4 = || <[u8; 4]>::try_from(&b[..4]).ok();
        let w8 = || <[u8; 8]>::try_from(&b[..8]).ok();
        let float =
            |x: f64| (x.fract() == 0.0 && (0.0..=u32::MAX as f64).contains(&x)).then_some(x as u64);
        match self {
            Scalar::F32 => float(f64::from(f32::from_le_bytes(w4()?))),
            Scalar::F64 => float(f64::from_le_bytes(w8()?)),
            Scalar::I32 => u64::try_from(i32::from_le_bytes(w4()?)).ok(),
            Scalar::U32 => Some(u64::from(u32::from_le_bytes(w4()?))),
            Scalar::I64 => u64::try_from(i64::from_le_bytes(w8()?)).ok(),
            Scalar::U64 => Some(u64::from_le_bytes(w8()?)),
            Scalar::Bool | Scalar::Utf8 => Some(u64::from(b[0])),
        }
    }

    pub const fn size(self) -> usize {
        match self {
            Scalar::Bool | Scalar::Utf8 => 1,
            Scalar::F32 | Scalar::I32 | Scalar::U32 => 4,
            Scalar::F64 | Scalar::I64 | Scalar::U64 => 8,
        }
    }
}

/// The most bytes a string field holds (`str4096`).
pub const STR_MAX_CAPACITY: usize = 4096;

/// The most elements a list field holds (`u64[1024]`).
pub const LIST_MAX_CAPACITY: usize = 1024;

/// The type of one reflected field. JSON spells it by [`FieldType::name`]
/// (`"vec3"`, `"str32"`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
    /// UTF-8 text of at most this many bytes (1 to [`STR_MAX_CAPACITY`]),
    /// zero-padded: `str32` is `Str(32)`.
    Str(u16),
    /// A list of at most this many elements (1 to [`LIST_MAX_CAPACITY`]) of
    /// one number type ([`Scalar::is_list_element`]): a count lane, then the
    /// element lanes, zero past the length. `u64[8]` is `List(Scalar::U64, 8)`.
    List(Scalar, u16),
}

impl Serialize for FieldType {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.name())
    }
}

impl<'de> Deserialize<'de> for FieldType {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let name = String::deserialize(d)?;
        FieldType::parse(&name).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "unknown field type '{name}': f32, f64, i32, u32, i64, u64, vec2, vec3, vec4, quat, bool, mat4, enum, strN (N bytes, 1 to {STR_MAX_CAPACITY}) or a list S[N] of f32, f64, i32, u32, i64 or u64 (N elements, 1 to {LIST_MAX_CAPACITY})"
            ))
        })
    }
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name())
    }
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
            FieldType::Str(_) => Scalar::Utf8,
            FieldType::List(element, _) => element,
        }
    }

    /// Scalars per value: 1, 2/3/4 for vectors and quaternions, 16 for a
    /// matrix, the byte capacity for a string, the capacity plus the count
    /// lane for a list.
    pub const fn lanes(self) -> usize {
        match self {
            FieldType::Vec2 => 2,
            FieldType::Vec3 => 3,
            FieldType::Vec4 | FieldType::Quat => 4,
            FieldType::Mat4 => 16,
            FieldType::Str(capacity) => capacity as usize,
            FieldType::List(_, capacity) => capacity as usize + 1,
            _ => 1,
        }
    }

    /// A list type of at most `capacity` elements of `element`, refused for
    /// an element that is not a number type or a capacity outside 1 to
    /// [`LIST_MAX_CAPACITY`].
    pub fn list(element: Scalar, capacity: usize) -> Result<FieldType> {
        if !element.is_list_element() {
            return Err(Error::new(
                ErrorKind::Schema,
                format!(
                    "a list holds f32, f64, i32, u32, i64 or u64 (a handle is a u64), not {}",
                    element.name()
                ),
            ));
        }
        if !(1..=LIST_MAX_CAPACITY).contains(&capacity) {
            return Err(Error::new(
                ErrorKind::Schema,
                format!(
                    "a list field holds 1 to {LIST_MAX_CAPACITY} elements, got a capacity of {capacity}"
                ),
            ));
        }
        Ok(FieldType::List(element, capacity as u16))
    }

    /// Whether this is a list (`u64[8]`).
    pub const fn is_list(self) -> bool {
        matches!(self, FieldType::List(..))
    }

    /// A list's element type and capacity in elements; `None` for every
    /// other type.
    pub const fn list_of(self) -> Option<(Scalar, usize)> {
        match self {
            FieldType::List(element, capacity) => Some((element, capacity as usize)),
            _ => None,
        }
    }

    /// A string type of `capacity` bytes, refused outside 1 to
    /// [`STR_MAX_CAPACITY`].
    pub fn string(capacity: usize) -> Result<FieldType> {
        if (1..=STR_MAX_CAPACITY).contains(&capacity) {
            Ok(FieldType::Str(capacity as u16))
        } else {
            Err(Error::new(
                ErrorKind::Schema,
                format!(
                    "a string field holds 1 to {STR_MAX_CAPACITY} bytes, got a capacity of {capacity}"
                ),
            ))
        }
    }

    /// Whether this is a string (`strN`).
    pub const fn is_string(self) -> bool {
        matches!(self, FieldType::Str(_))
    }

    /// A string's capacity in bytes; `None` for every other type.
    pub const fn capacity(self) -> Option<usize> {
        match self {
            FieldType::Str(capacity) => Some(capacity as usize),
            _ => None,
        }
    }

    /// Whether packed bytes of this type have values outside the type (a
    /// bool byte other than 0 or 1, an enum index past its variants, a string
    /// that is not zero-padded UTF-8, a list whose count or tail is not
    /// one), which [`Schema::check_packed`] refuses.
    pub const fn is_checked(self) -> bool {
        matches!(
            self,
            FieldType::Bool | FieldType::Enum | FieldType::Str(_) | FieldType::List(..)
        )
    }

    /// Bytes per value.
    pub const fn size(self) -> usize {
        self.scalar().size() * self.lanes()
    }

    /// The lower-case name used in JSON, snapshots and hashes: `f32`,
    /// `vec3`, `str32`, `u64[8]`.
    pub fn name(self) -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(match self {
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
            FieldType::Str(64) => "str64",
            FieldType::Str(256) => "str256",
            FieldType::Str(capacity) => return format!("str{capacity}").into(),
            FieldType::List(element, capacity) => {
                return format!("{}[{capacity}]", element.name()).into();
            }
        })
    }

    /// The type a [`FieldType::name`] names; `None` for an unknown name, a
    /// string capacity outside 1 to [`STR_MAX_CAPACITY`] or a list that
    /// [`FieldType::list`] refuses.
    pub fn parse(name: &str) -> Option<FieldType> {
        if let Some((element, rest)) = name.split_once('[') {
            let digits = rest.strip_suffix(']')?;
            // One spelling per type: no sign, no leading zero.
            if digits.is_empty()
                || !digits.bytes().all(|b| b.is_ascii_digit())
                || digits.starts_with('0')
            {
                return None;
            }
            let element = match FieldType::parse(element)? {
                FieldType::F32 => Scalar::F32,
                FieldType::F64 => Scalar::F64,
                FieldType::I32 => Scalar::I32,
                FieldType::U32 => Scalar::U32,
                FieldType::I64 => Scalar::I64,
                FieldType::U64 => Scalar::U64,
                _ => return None,
            };
            return FieldType::list(element, digits.parse().ok()?).ok();
        }
        Some(match name {
            "f32" => FieldType::F32,
            "f64" => FieldType::F64,
            "i32" => FieldType::I32,
            "u32" => FieldType::U32,
            "i64" => FieldType::I64,
            "u64" => FieldType::U64,
            "vec2" => FieldType::Vec2,
            "vec3" => FieldType::Vec3,
            "vec4" => FieldType::Vec4,
            "quat" => FieldType::Quat,
            "bool" => FieldType::Bool,
            "mat4" => FieldType::Mat4,
            "enum" => FieldType::Enum,
            other => {
                let digits = other.strip_prefix("str")?;
                // One spelling per type: no sign, no leading zero.
                if !digits.bytes().all(|b| b.is_ascii_digit()) || digits.starts_with('0') {
                    return None;
                }
                return FieldType::string(digits.parse().ok()?).ok();
            }
        })
    }
}

/// `text` as the packed bytes of a string field of `capacity` bytes: its
/// UTF-8, then zeros. Refused, with the fix named, when it is longer than the
/// capacity or holds a NUL character; never truncated.
pub fn pack_str(text: &str, capacity: usize) -> std::result::Result<Vec<u8>, String> {
    let bytes = text.as_bytes();
    if bytes.len() > capacity {
        let shown: String = text.chars().take(24).collect();
        let more = if shown.len() < text.len() { "..." } else { "" };
        let fix = if bytes.len() <= STR_MAX_CAPACITY {
            format!(
                "shorten the text, or declare the field str({}) or larger",
                bytes.len()
            )
        } else {
            format!("shorten the text: a string field holds at most {STR_MAX_CAPACITY} bytes")
        };
        return Err(format!(
            "'{shown}{more}' is {} bytes of UTF-8 and the field holds {capacity}: {fix}",
            bytes.len()
        ));
    }
    if bytes.contains(&0) {
        return Err("a string field cannot hold a NUL character (\\0)".into());
    }
    let mut out = vec![0u8; capacity];
    out[..bytes.len()].copy_from_slice(bytes);
    Ok(out)
}

/// The text of a string field's packed bytes: the bytes before the first
/// zero (the longest valid UTF-8 prefix of them, for bytes that never passed
/// [`FieldSchema::check_value`]).
pub fn unpack_str(bytes: &[u8]) -> &str {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    match std::str::from_utf8(&bytes[..end]) {
        Ok(s) => s,
        Err(e) => std::str::from_utf8(&bytes[..e.valid_up_to()]).unwrap_or_default(),
    }
}

/// The length a list field's packed bytes hold: its count lane, read as a
/// whole number of the element type. `None` for a type that is not a list,
/// or a count lane that is not a whole number from 0 to the capacity.
pub fn list_len(ty: FieldType, bytes: &[u8]) -> Option<usize> {
    let (element, capacity) = ty.list_of()?;
    let n = element.whole_lane(bytes.get(..element.size())?)?;
    (n <= capacity as u64).then_some(n as usize)
}

/// The packed bytes of a list field of type `ty` holding `elements` (each
/// one lane: the element's little-endian bytes), the lanes past them zero.
/// Refused, with the fix named, past the capacity; never truncated.
pub fn pack_list(ty: FieldType, elements: &[&[u8]]) -> std::result::Result<Vec<u8>, String> {
    let (element, capacity) = ty
        .list_of()
        .ok_or_else(|| format!("{} is not a list type", ty.name()))?;
    if elements.len() > capacity {
        return Err(format!(
            "{} elements and the field holds {capacity}: remove {}, or declare the field {}[{}] or larger",
            elements.len(),
            elements.len() - capacity,
            element.name(),
            elements.len()
        ));
    }
    let lane = element.size();
    let mut out = vec![0u8; ty.size()];
    element.count_lane(elements.len(), &mut out[..lane]);
    for (i, e) in elements.iter().enumerate() {
        if e.len() != lane {
            return Err(format!(
                "element {i} is {} bytes, a {} is {lane}",
                e.len(),
                element.name()
            ));
        }
        out[lane * (i + 1)..lane * (i + 2)].copy_from_slice(e);
    }
    Ok(out)
}

/// The element lanes of a list field's packed bytes, up to its length (none
/// for bytes whose count lane [`list_len`] does not read).
pub fn list_elements(ty: FieldType, bytes: &[u8]) -> impl Iterator<Item = &[u8]> {
    let lane = ty.scalar().size();
    let n = list_len(ty, bytes).unwrap_or(0);
    bytes
        .get(lane..)
        .unwrap_or_default()
        .chunks_exact(lane)
        .take(n)
}

mod sealed {
    pub trait Sealed {}
    pub trait SealedField {}
}

/// A Rust type a column can be viewed as (`ArchetypeView::read::<T>`): plain
/// old data whose every bit pattern is a value. Sealed: `f32`, `f64`, `i32`,
/// `u32`, `i64`, `u64`, `Vec2`, `Vec3`, `Vec4`, `Quat`, `Mat4` and [`Bool8`]
/// (a bool column). Every `FieldValue` is a [`ReflectField`]; `bool` and
/// [`Str<N>`] are fields but not views (a string column is read through
/// [`unpack_str`]).
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
/// plus `bool` (a `bool` field) and [`Str<N>`] (a `strN` field). An enum field is a `u32` with `#[reflect(variants = [...])]`.
/// Sealed. The two methods are what the derive generates calls to.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a reflected field type",
    note = "fields of a #[derive(Reflect)] struct are f32, f64, i32, u32, i64, u64, Vec2, Vec3, Vec4, Quat, Mat4, bool, Bool8 or Str<N> (an enum is a u32 with #[reflect(variants = [..])])"
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

/// A string field: UTF-8 text of at most `N` bytes stored inline,
/// zero-padded (`str32` is `Str<32>`). `N` is 1 to [`STR_MAX_CAPACITY`],
/// checked at compile time.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Str<const N: usize>([u8; N]);

/// A `str64` field.
pub type Str64 = Str<64>;

/// A `str256` field.
pub type Str256 = Str<256>;

impl<const N: usize> Str<N> {
    /// The capacity in bytes of UTF-8.
    pub const CAPACITY: usize = N;

    /// The empty string.
    pub const EMPTY: Self = Str([0; N]);

    /// `text` as an inline string. Refused when it is longer than the
    /// capacity in UTF-8 bytes or holds a NUL character ([`pack_str`]).
    pub fn new(text: &str) -> Result<Self> {
        let packed = pack_str(text, N).map_err(|reason| Error::new(ErrorKind::Invalid, reason))?;
        let mut out = [0u8; N];
        out.copy_from_slice(&packed);
        Ok(Str(out))
    }

    /// The text: the bytes before the first zero ([`unpack_str`]).
    pub fn as_str(&self) -> &str {
        unpack_str(&self.0)
    }

    /// The packed bytes.
    pub fn as_bytes(&self) -> &[u8; N] {
        &self.0
    }
}

impl<const N: usize> Default for Str<N> {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl<const N: usize> std::fmt::Debug for Str<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_str(), f)
    }
}

impl<const N: usize> std::fmt::Display for Str<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<const N: usize> sealed::SealedField for Str<N> {}

impl<const N: usize> ReflectField for Str<N> {
    const TYPE: FieldType = {
        assert!(
            N >= 1 && N <= STR_MAX_CAPACITY,
            "a Str<N> holds 1 to STR_MAX_CAPACITY (4096) bytes"
        );
        FieldType::Str(N as u16)
    };

    fn write_packed(&self, out: &mut dyn FnMut(&[u8])) {
        out(&self.0);
    }

    fn read_packed(input: &mut dyn FnMut(&mut [u8])) -> Self {
        let mut b = [0u8; N];
        input(&mut b);
        Str(b)
    }
}

/// A number type a [`List`] holds: `f32`, `f64`, `i32`, `u32`, `i64`,
/// `u64` (a handle is a `u64`). Sealed.
pub trait ListElement: FieldValue + Send + Sync {
    #[doc(hidden)]
    const SCALAR: Scalar;
}

macro_rules! list_element {
    ($($t:ty => $s:ident),* $(,)?) => {$(
        impl ListElement for $t {
            const SCALAR: Scalar = Scalar::$s;
        }
    )*};
}

list_element! { f32 => F32, f64 => F64, i32 => I32, u32 => U32, i64 => I64, u64 => U64 }

/// A list field: at most `N` values of `T` stored inline (`u64[8]` is
/// `List<u64, 8>`). `N` is 1 to [`LIST_MAX_CAPACITY`], checked at compile
/// time. The slots past the length are kept zero, so two lists with the same
/// elements are the same bytes.
#[derive(Clone, Copy, PartialEq)]
pub struct List<T: ListElement, const N: usize> {
    items: [T; N],
    len: usize,
}

impl<T: ListElement, const N: usize> List<T, N> {
    /// The capacity in elements.
    pub const CAPACITY: usize = N;

    /// The empty list.
    pub fn new() -> Self {
        List {
            items: [T::zero(); N],
            len: 0,
        }
    }

    /// A list of `items`, refused past the capacity ([`pack_list`]'s reason).
    pub fn from_slice(items: &[T]) -> Result<Self> {
        let mut out = Self::new();
        for &x in items {
            out.push(x)?;
        }
        Ok(out)
    }

    /// Appends `x`, refused when the list is full.
    pub fn push(&mut self, x: T) -> Result<()> {
        if self.len == N {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("the list is full: it holds {N}"),
            ));
        }
        self.items[self.len] = x;
        self.len += 1;
        Ok(())
    }

    /// Empties the list.
    pub fn clear(&mut self) {
        self.items = [T::zero(); N];
        self.len = 0;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_slice(&self) -> &[T] {
        &self.items[..self.len]
    }
}

impl<T: ListElement, const N: usize> Default for List<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ListElement + std::fmt::Debug, const N: usize> std::fmt::Debug for List<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}

impl<T: ListElement, const N: usize> sealed::SealedField for List<T, N> {}

impl<T: ListElement, const N: usize> ReflectField for List<T, N> {
    const TYPE: FieldType = {
        assert!(
            N >= 1 && N <= LIST_MAX_CAPACITY,
            "a List<T, N> holds 1 to LIST_MAX_CAPACITY (1024) elements"
        );
        FieldType::List(T::SCALAR, N as u16)
    };

    fn write_packed(&self, out: &mut dyn FnMut(&[u8])) {
        let mut bytes = vec![0u8; <Self as ReflectField>::TYPE.size()];
        let lane = T::SCALAR.size();
        T::SCALAR.count_lane(self.len, &mut bytes[..lane]);
        for (i, x) in self.items.iter().enumerate() {
            bytes[lane * (i + 1)..lane * (i + 2)].copy_from_slice(x.as_bytes());
        }
        out(&bytes);
    }

    fn read_packed(input: &mut dyn FnMut(&mut [u8])) -> Self {
        let ty = <Self as ReflectField>::TYPE;
        let mut bytes = vec![0u8; ty.size()];
        input(&mut bytes);
        let mut out = Self::new();
        for e in list_elements(ty, &bytes) {
            let mut x = T::zero();
            x.as_bytes_mut().copy_from_slice(e);
            out.items[out.len] = x;
            out.len += 1;
        }
        out
    }
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
