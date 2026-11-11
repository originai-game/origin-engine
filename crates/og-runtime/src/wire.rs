//! Typed wire arguments: what a binding takes as JSON, read into the Rust
//! types `og api gen` writes from the IDL (`generated/wire.rs`, re-exported
//! here), so a field's name, type and default are the IDL's and a binding
//! that reads a field the IDL does not declare fails to build.
//!
//! The rules every reader follows: an absent or `null` value takes its
//! default; a value of the wrong JSON type, an integer that is not whole or
//! is out of range, an unknown enum variant or union tag is refused with
//! the path of the field (`desc.origin.x: expected a number, got "1"`);
//! fields the IDL does not declare are ignored.

use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

#[rustfmt::skip]
#[path = "generated/wire.rs"]
mod types;
pub use types::*;

/// A value read from its JSON wire form.
pub trait FromJson: Sized {
    fn from_json(v: &Value) -> Result<Self, String>;
    /// `v` read over `base`: an object's absent fields keep `base`'s, so a
    /// partial value (`color: { r: 0 }`) fills the rest from its field's
    /// IDL default, not from the type's zero. Values that are not objects
    /// are read whole.
    fn from_json_over(v: &Value, base: Self) -> Result<Self, String> {
        let _ = base;
        Self::from_json(v)
    }
    /// Whether `v` has this type's shape, for [`OneOf`] to choose between
    /// readers that ignore unknown fields: a vector is not an object
    /// whose keys are all others' (`{ shape, holes }` is no Vector2).
    fn reads(v: &Value) -> bool {
        let _ = v;
        true
    }
}

/// The JSON argument `text` (absent, empty or `null`: the default) read as
/// `T`; a refusal names the parameter, `name`, and the field's path.
pub fn arg<T: FromJson + Default>(text: Option<&str>, name: &str) -> Result<T, String> {
    value(&crate::parse(text)?, name)
}

/// An already parsed JSON argument read as `T`, as [`arg`].
pub fn value<T: FromJson + Default>(v: &Value, name: &str) -> Result<T, String> {
    if v.is_null() {
        return Ok(T::default());
    }
    T::from_json(v).map_err(|e| at(name, e).replacen(PATH, ": ", 1))
}

/// Separates a refusal's field path from its message until [`value`]
/// shows it (`desc.points[2].x: expected a number, got "1"`).
const PATH: char = '\u{1}';

/// A refusal `e` from inside `segment` (a field name, or `[i]`).
pub fn at(segment: &str, e: String) -> String {
    match e.split_once(PATH) {
        Some((path, _)) if path.starts_with('[') => format!("{segment}{e}"),
        Some(_) => format!("{segment}.{e}"),
        None => format!("{segment}{PATH}{e}"),
    }
}

/// Field `key` of `o`, or `default` when it is absent or `null`.
pub fn field<T: FromJson>(o: &Map<String, Value>, key: &str, default: T) -> Result<T, String> {
    match o.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(v) => T::from_json_over(v, default).map_err(|e| at(key, e)),
    }
}

/// `v` as an object.
pub fn object(v: &Value) -> Result<&Map<String, Value>, String> {
    v.as_object()
        .ok_or_else(|| format!("expected an object, got {}", shown(v)))
}

/// `v` as a string.
pub fn text(v: &Value) -> Result<&str, String> {
    v.as_str()
        .ok_or_else(|| format!("expected a string, got {}", shown(v)))
}

/// `v` as an enum's variant name: a string, else refused naming the
/// variants (an enum is never its index).
pub fn variant<'a>(v: &'a Value, what: &str, names: &[&str]) -> Result<&'a str, String> {
    v.as_str().ok_or_else(|| {
        format!(
            "expected {} {what} name ({}), got {}",
            article(what),
            names.join(", "),
            shown(v)
        )
    })
}

fn article(what: &str) -> &'static str {
    if what.starts_with(['A', 'E', 'I', 'O', 'U']) {
        "an"
    } else {
        "a"
    }
}

/// The refusal of an unknown enum variant or union tag.
pub fn unknown_variant(what: &str, got: &str, names: &[&str]) -> String {
    format!(
        "'{got}' is not {} {what}: {}",
        article(what),
        names.join(", ")
    )
}

/// `v` for a message, cut short.
fn shown(v: &Value) -> String {
    let s = v.to_string();
    if s.chars().count() > 40 {
        format!("{}…", s.chars().take(40).collect::<String>())
    } else {
        s
    }
}

fn number(v: &Value) -> Result<f64, String> {
    v.as_f64()
        .ok_or_else(|| format!("expected a number, got {}", shown(v)))
}

/// A whole number in `lo..=hi`.
fn whole(v: &Value, lo: f64, hi: f64, what: &str) -> Result<f64, String> {
    let n = number(v)?;
    if n.fract() == 0.0 && n >= lo && n <= hi {
        Ok(n)
    } else {
        Err(format!("expected {what}, got {}", shown(v)))
    }
}

impl FromJson for bool {
    fn reads(v: &Value) -> bool {
        v.is_boolean()
    }
    fn from_json(v: &Value) -> Result<Self, String> {
        v.as_bool()
            .ok_or_else(|| format!("expected true or false, got {}", shown(v)))
    }
}

impl FromJson for f32 {
    fn reads(v: &Value) -> bool {
        v.is_number()
    }
    fn from_json(v: &Value) -> Result<Self, String> {
        number(v).map(|n| n as f32)
    }
}

impl FromJson for f64 {
    fn reads(v: &Value) -> bool {
        v.is_number()
    }
    fn from_json(v: &Value) -> Result<Self, String> {
        number(v)
    }
}

impl FromJson for u32 {
    fn reads(v: &Value) -> bool {
        v.is_number()
    }
    fn from_json(v: &Value) -> Result<Self, String> {
        whole(
            v,
            0.0,
            u32::MAX as f64,
            "a whole number from 0 to 4294967295",
        )
        .map(|n| n as u32)
    }
}

impl FromJson for i32 {
    fn reads(v: &Value) -> bool {
        v.is_number()
    }
    fn from_json(v: &Value) -> Result<Self, String> {
        whole(
            v,
            i32::MIN as f64,
            i32::MAX as f64,
            "a whole number from -2147483648 to 2147483647",
        )
        .map(|n| n as i32)
    }
}

impl FromJson for u8 {
    fn reads(v: &Value) -> bool {
        v.is_number()
    }
    fn from_json(v: &Value) -> Result<Self, String> {
        whole(v, 0.0, 255.0, "a whole number from 0 to 255").map(|n| n as u8)
    }
}

/// A handle: a JavaScript number, exact below 2^53.
impl FromJson for u64 {
    fn reads(v: &Value) -> bool {
        v.is_number()
    }
    fn from_json(v: &Value) -> Result<Self, String> {
        if let Some(n) = v.as_u64() {
            return Ok(n);
        }
        whole(v, 0.0, 9007199254740991.0, "a handle").map(|n| n as u64)
    }
}

impl FromJson for String {
    fn reads(v: &Value) -> bool {
        v.is_string()
    }
    fn from_json(v: &Value) -> Result<Self, String> {
        text(v).map(str::to_string)
    }
}

impl FromJson for Value {
    fn from_json(v: &Value) -> Result<Self, String> {
        Ok(v.clone())
    }
}

impl<T: FromJson> FromJson for Option<T> {
    fn from_json(v: &Value) -> Result<Self, String> {
        match v {
            Value::Null => Ok(None),
            v => T::from_json(v).map(Some),
        }
    }
    fn reads(v: &Value) -> bool {
        v.is_null() || T::reads(v)
    }
    fn from_json_over(v: &Value, base: Self) -> Result<Self, String> {
        match (v, base) {
            (Value::Null, _) => Ok(None),
            (v, Some(b)) => T::from_json_over(v, b).map(Some),
            (v, None) => T::from_json(v).map(Some),
        }
    }
}

impl<T: FromJson> FromJson for Vec<T> {
    fn from_json(v: &Value) -> Result<Self, String> {
        let a = v
            .as_array()
            .ok_or_else(|| format!("expected a list, got {}", shown(v)))?;
        a.iter()
            .enumerate()
            .map(|(i, x)| T::from_json(x).map_err(|e| at(&format!("[{i}]"), e)))
            .collect()
    }
    fn reads(v: &Value) -> bool {
        v.as_array().is_some_and(|a| a.iter().all(T::reads))
    }
}

impl<T: FromJson + Default + Copy, const N: usize> FromJson for [T; N] {
    fn from_json(v: &Value) -> Result<Self, String> {
        let items: Vec<T> = Vec::from_json(v)?;
        if items.len() != N {
            return Err(format!("expected {N} values, got {}", items.len()));
        }
        let mut out = [T::default(); N];
        out.copy_from_slice(&items);
        Ok(out)
    }
}

impl<T: FromJson> FromJson for BTreeMap<String, T> {
    fn from_json(v: &Value) -> Result<Self, String> {
        object(v)?
            .iter()
            .map(|(k, x)| {
                T::from_json(x)
                    .map(|t| (k.clone(), t))
                    .map_err(|e| at(k, e))
            })
            .collect()
    }
}

/// `OneOf<A, B>`: an `A` when the value reads as one, else a `B`.
#[derive(Clone, Debug, PartialEq)]
pub enum OneOf<A, B> {
    A(A),
    B(B),
}

impl<A: Default, B> Default for OneOf<A, B> {
    fn default() -> Self {
        OneOf::A(A::default())
    }
}

/// An `A` when the value has `A`'s shape ([`FromJson::reads`]) and reads
/// as one, else a `B`. A value of `A`'s shape refused inside (a field
/// path, such as `[1].x: expected a number`) is refused with that.
impl<A: FromJson, B: FromJson> FromJson for OneOf<A, B> {
    fn from_json(v: &Value) -> Result<Self, String> {
        let ea = match A::from_json(v) {
            Ok(a) if A::reads(v) => return Ok(OneOf::A(a)),
            // It has A's shape and failed inside: A's refusal.
            Err(e) if A::reads(v) && e.contains(PATH) => return Err(e),
            Ok(_) => None,
            Err(e) => Some(e),
        };
        B::from_json(v).map(OneOf::B).map_err(|eb| match ea {
            None => eb,
            Some(ea) if ea == eb || eb.contains(PATH) => eb,
            Some(ea) => format!("{ea}; or {eb}"),
        })
    }
    fn reads(v: &Value) -> bool {
        A::reads(v) || B::reads(v)
    }
}

fn components<const N: usize>(
    v: &Value,
    keys: [&str; N],
    defaults: [f32; N],
) -> Result<[f32; N], String> {
    let o = object(v)?;
    let mut out = defaults;
    for (i, k) in keys.iter().enumerate() {
        out[i] = field(o, k, defaults[i])?;
    }
    Ok(out)
}

/// Whether `v` is an object that names one of `keys`, or names nothing.
fn names_any(v: &Value, keys: &[&str]) -> bool {
    v.as_object()
        .is_some_and(|o| o.is_empty() || keys.iter().any(|k| o.contains_key(*k)))
}

impl FromJson for Vec2 {
    fn from_json(v: &Value) -> Result<Self, String> {
        components(v, ["x", "y"], [0.0; 2]).map(Vec2::from_array)
    }
    fn reads(v: &Value) -> bool {
        names_any(v, &["x", "y"])
    }
    fn from_json_over(v: &Value, base: Self) -> Result<Self, String> {
        components(v, ["x", "y"], base.to_array()).map(Vec2::from_array)
    }
}

impl FromJson for Vec3 {
    fn reads(v: &Value) -> bool {
        names_any(v, &["x", "y", "z"])
    }
    fn from_json(v: &Value) -> Result<Self, String> {
        components(v, ["x", "y", "z"], [0.0; 3]).map(Vec3::from_array)
    }
    fn from_json_over(v: &Value, base: Self) -> Result<Self, String> {
        components(v, ["x", "y", "z"], base.to_array()).map(Vec3::from_array)
    }
}

impl FromJson for Vec4 {
    fn reads(v: &Value) -> bool {
        names_any(v, &["x", "y", "z", "w"])
    }
    fn from_json(v: &Value) -> Result<Self, String> {
        components(v, ["x", "y", "z", "w"], [0.0; 4]).map(Vec4::from_array)
    }
    fn from_json_over(v: &Value, base: Self) -> Result<Self, String> {
        components(v, ["x", "y", "z", "w"], base.to_array()).map(Vec4::from_array)
    }
}

/// A quaternion; an absent `w` is 1 (the IDL's zero value is the identity).
impl FromJson for Quat {
    fn reads(v: &Value) -> bool {
        names_any(v, &["x", "y", "z", "w"])
    }
    fn from_json(v: &Value) -> Result<Self, String> {
        components(v, ["x", "y", "z", "w"], [0.0, 0.0, 0.0, 1.0]).map(Quat::from_array)
    }
    fn from_json_over(v: &Value, base: Self) -> Result<Self, String> {
        components(v, ["x", "y", "z", "w"], base.to_array()).map(Quat::from_array)
    }
}

/// A matrix: `{ elements }`, sixteen numbers in column-major order.
impl FromJson for Mat4 {
    fn from_json(v: &Value) -> Result<Self, String> {
        let o = object(v)?;
        let e: [f32; 16] = field(o, "elements", Mat4::IDENTITY.to_cols_array())?;
        Ok(Mat4::from_cols_array(&e))
    }
}

/// A value written in its JSON wire form: the IDL's fields, an optional
/// field only when it is set, numbers as `json!` writes them (so a typed
/// answer is the text the hand-written `json!` answered).
pub trait ToJson {
    fn to_json(&self) -> Value;
}

/// `{ ok: true, value }` of a typed value: a successful `Result<T>`.
pub fn ok<T: ToJson + ?Sized>(value: &T) -> String {
    crate::ok(value.to_json())
}

/// A typed value as wire text.
pub fn text_of<T: ToJson + ?Sized>(value: &T) -> String {
    value.to_json().to_string()
}

macro_rules! to_json_from {
    ($($t:ty),*) => {$(
        impl ToJson for $t {
            fn to_json(&self) -> Value {
                Value::from(*self)
            }
        }
    )*};
}
to_json_from!(bool, f32, f64, u8, u32, i32, u64);

impl ToJson for String {
    fn to_json(&self) -> Value {
        Value::String(self.clone())
    }
}

impl ToJson for str {
    fn to_json(&self) -> Value {
        Value::String(self.to_string())
    }
}

impl ToJson for Value {
    fn to_json(&self) -> Value {
        self.clone()
    }
}

impl ToJson for () {
    fn to_json(&self) -> Value {
        Value::Null
    }
}

impl<T: ToJson> ToJson for Option<T> {
    fn to_json(&self) -> Value {
        self.as_ref().map_or(Value::Null, ToJson::to_json)
    }
}

impl<T: ToJson> ToJson for [T] {
    fn to_json(&self) -> Value {
        Value::Array(self.iter().map(ToJson::to_json).collect())
    }
}

impl<T: ToJson> ToJson for Vec<T> {
    fn to_json(&self) -> Value {
        self.as_slice().to_json()
    }
}

impl<T: ToJson, const N: usize> ToJson for [T; N] {
    fn to_json(&self) -> Value {
        self.as_slice().to_json()
    }
}

impl<T: ToJson> ToJson for BTreeMap<String, T> {
    fn to_json(&self) -> Value {
        Value::Object(self.iter().map(|(k, v)| (k.clone(), v.to_json())).collect())
    }
}

impl<A: ToJson, B: ToJson> ToJson for OneOf<A, B> {
    fn to_json(&self) -> Value {
        match self {
            OneOf::A(a) => a.to_json(),
            OneOf::B(b) => b.to_json(),
        }
    }
}

fn object_of<const N: usize>(keys: [&str; N], values: [f32; N]) -> Value {
    Value::Object(
        keys.iter()
            .zip(values)
            .map(|(k, v)| (k.to_string(), Value::from(v)))
            .collect(),
    )
}

impl ToJson for Vec2 {
    fn to_json(&self) -> Value {
        object_of(["x", "y"], self.to_array())
    }
}

impl ToJson for Vec3 {
    fn to_json(&self) -> Value {
        object_of(["x", "y", "z"], self.to_array())
    }
}

impl ToJson for Vec4 {
    fn to_json(&self) -> Value {
        object_of(["x", "y", "z", "w"], self.to_array())
    }
}

impl ToJson for Quat {
    fn to_json(&self) -> Value {
        object_of(["x", "y", "z", "w"], self.to_array())
    }
}

/// `{ elements }`, column-major, as three.js Matrix4.
impl ToJson for Mat4 {
    fn to_json(&self) -> Value {
        let mut o = Map::new();
        o.insert("elements".into(), self.to_cols_array().to_json());
        Value::Object(o)
    }
}
