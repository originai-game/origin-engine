use crate::error::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};

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
    pub(super) fn count_lane(self, n: usize, out: &mut [u8]) {
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
    /// Canonical timing tag and four payload words (20 bytes).
    Ease,
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
                "unknown field type '{name}': f32, f64, i32, u32, i64, u64, vec2, vec3, vec4, quat, bool, mat4, enum, ease, strN (N bytes, 1 to {STR_MAX_CAPACITY}) or a list S[N] of f32, f64, i32, u32, i64 or u64 (N elements, 1 to {LIST_MAX_CAPACITY})"
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
            FieldType::U32 | FieldType::Enum | FieldType::Ease => Scalar::U32,
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
            FieldType::Ease => 5,
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
            FieldType::Bool
                | FieldType::Enum
                | FieldType::Ease
                | FieldType::Str(_)
                | FieldType::List(..)
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
            FieldType::Ease => "ease",
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
            "ease" => FieldType::Ease,
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
