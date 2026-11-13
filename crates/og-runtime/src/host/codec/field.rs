//! One field's bytes: JSON to bytes and back, with its defaults.

use super::*;

pub(super) fn f32_lanes(v: &Value, keys: &[&str], out: &mut Vec<u8>) -> Result<(), String> {
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
pub(super) fn encode_list(f: &FieldLayout, v: &Value, out: &mut Vec<u8>) -> Result<(), String> {
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
pub(super) fn kind_label(kind: Kind) -> &'static str {
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
pub(super) fn encode_number(kind: Kind, v: &Value, out: &mut Vec<u8>) -> Result<(), String> {
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

/// A field's declared default as og-core's lanes ([`FieldSchema::default`]),
/// so what the world fills an added field with on a migrating restore (and
/// the schema reports) is the declared value, not zero: `None` for a field
/// without one, a string or a list.
pub(super) fn default_lanes(f: &FieldLayout) -> Option<Vec<f64>> {
    if f.default.is_null() || f.ty.is_string() || f.ty.is_list() {
        return None;
    }
    let mut bytes = Vec::with_capacity(f.ty.size());
    encode_field(f, &f.default, &mut bytes).ok()?;
    let lane = f.ty.scalar().size();
    Some(
        bytes
            .chunks_exact(lane)
            .map(|b| match f.ty.scalar() {
                og_core::reflect::Scalar::F32 => {
                    f64::from(f32::from_le_bytes(b.try_into().unwrap()))
                }
                og_core::reflect::Scalar::F64 => f64::from_le_bytes(b.try_into().unwrap()),
                og_core::reflect::Scalar::I32 => {
                    f64::from(i32::from_le_bytes(b.try_into().unwrap()))
                }
                og_core::reflect::Scalar::U32 => {
                    f64::from(u32::from_le_bytes(b.try_into().unwrap()))
                }
                og_core::reflect::Scalar::I64 => i64::from_le_bytes(b.try_into().unwrap()) as f64,
                og_core::reflect::Scalar::U64 => u64::from_le_bytes(b.try_into().unwrap()) as f64,
                og_core::reflect::Scalar::Bool | og_core::reflect::Scalar::Utf8 => f64::from(b[0]),
            })
            .collect(),
    )
}

pub(super) fn encode_field(f: &FieldLayout, v: &Value, out: &mut Vec<u8>) -> Result<(), String> {
    if f.ty.is_list() {
        return encode_list(f, v, out);
    }
    match f.kind {
        Kind::F32 | Kind::F64 | Kind::U32 | Kind::I32 => encode_number(f.kind, v, out)?,
        Kind::Ease => {
            let text = v.as_str().unwrap_or("none");
            if !v.is_null() && !v.is_string() {
                return Err(format!("ease {v} must be a timing string"));
            }
            out.extend_from_slice(&og_core::ease::Ease::parse_text(text)?.to_bytes());
        }
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

pub(super) fn f32_at(b: &[u8], i: usize) -> f32 {
    f32::from_le_bytes(b[i * 4..i * 4 + 4].try_into().expect("4 bytes"))
}

pub(super) fn decode_field(f: &FieldLayout, b: &[u8]) -> Value {
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
        Kind::Ease => json!(og_core::ease::Ease::from_bytes(b).map(|ease| ease.to_text())),
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
