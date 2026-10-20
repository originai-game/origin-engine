//! Reflected field values as JSON and as command text, both ways, through
//! the schema: the one path `entity`, `bounds`, `patch`, `set` and the dumps
//! share.

use og_core::{FieldSchema, FieldType, Layout, Schema};
use serde_json::{Map, Number, Value};

/// A number as JSON. An `f32` is written by its shortest decimal form, so
/// `0.1f32` reads `0.1`; a non-finite value has no JSON form and is `null`.
fn real(v: f64) -> Value {
    Number::from_f64(v).map_or(Value::Null, Value::Number)
}

fn f32_json(v: f32) -> Value {
    if !v.is_finite() {
        return Value::Null;
    }
    real(format!("{v}").parse::<f64>().unwrap_or(f64::from(v)))
}

fn word<const N: usize>(bytes: &[u8], at: usize) -> [u8; N] {
    let mut out = [0u8; N];
    out.copy_from_slice(&bytes[at..at + N]);
    out
}

/// One field's packed bytes as JSON: a number, an array for vectors and
/// quaternions (`[x, y, z, w]`), the text of a string.
pub(crate) fn field_json(ty: FieldType, bytes: &[u8]) -> Value {
    match ty {
        FieldType::F32 => f32_json(f32::from_le_bytes(word(bytes, 0))),
        FieldType::F64 => real(f64::from_le_bytes(word(bytes, 0))),
        FieldType::I32 => Value::from(i32::from_le_bytes(word(bytes, 0))),
        FieldType::U32 => Value::from(u32::from_le_bytes(word(bytes, 0))),
        FieldType::I64 => Value::from(i64::from_le_bytes(word(bytes, 0))),
        FieldType::U64 => Value::from(u64::from_le_bytes(word(bytes, 0))),
        FieldType::Vec2 | FieldType::Vec3 | FieldType::Vec4 | FieldType::Quat | FieldType::Mat4 => {
            Value::Array(
                (0..ty.lanes())
                    .map(|i| f32_json(f32::from_le_bytes(word(bytes, i * 4))))
                    .collect(),
            )
        }
        FieldType::Bool => Value::Bool(bytes[0] != 0),
        FieldType::Enum => Value::from(u32::from_le_bytes(word(bytes, 0))),
        FieldType::Str(_) => Value::from(og_core::unpack_str(bytes)),
    }
}

/// Every field of a component's packed value, by field name.
pub(crate) fn component_json(schema: &Schema, bytes: &[u8]) -> Map<String, Value> {
    let layout = Layout::of(schema);
    schema
        .fields
        .iter()
        .enumerate()
        .map(|(i, f)| (f.name.clone(), field_json(f.ty, &bytes[layout.range(i)])))
        .collect()
}

/// The probe's writes (`set`, `patch`, a sweep, Tune) are refused outside
/// the field's declared range, as scene text is when cooked; game code's
/// own writes are stored as given (FieldOptions: nothing clamps writes).
fn in_range(field: &FieldSchema, v: f64) -> Result<(), String> {
    let widen = "the probe writes only inside a field's declared range; widen min/max in its FieldOptions to try it";
    if let Some(min) = field.min.filter(|min| v < *min) {
        return Err(format!("{v} is below the minimum {min} ({widen})"));
    }
    if let Some(max) = field.max.filter(|max| v > *max) {
        return Err(format!("{v} is above the maximum {max} ({widen})"));
    }
    Ok(())
}

fn lane(
    field: &FieldSchema,
    value: &Value,
    scalar: FieldType,
    out: &mut Vec<u8>,
) -> Result<(), String> {
    let Some(v) = value.as_f64().filter(|v| v.is_finite()) else {
        return Err(format!("{value} is not a finite number"));
    };
    in_range(field, v)?;
    let integral = |lo: f64, hi: f64| -> Result<f64, String> {
        if v.fract() != 0.0 || v < lo || v > hi {
            Err(format!("{v} is not a {} value", scalar.name()))
        } else {
            Ok(v)
        }
    };
    match scalar {
        FieldType::F32 => {
            if v.abs() > f64::from(f32::MAX) {
                return Err(format!("{v} does not fit an f32"));
            }
            out.extend_from_slice(&(v as f32).to_le_bytes());
        }
        FieldType::F64 => out.extend_from_slice(&v.to_le_bytes()),
        FieldType::I32 => out.extend_from_slice(
            &(integral(f64::from(i32::MIN), f64::from(i32::MAX))? as i32).to_le_bytes(),
        ),
        FieldType::U32 => {
            out.extend_from_slice(&(integral(0.0, f64::from(u32::MAX))? as u32).to_le_bytes())
        }
        // 64-bit integers keep their exact JSON integer when it has one.
        FieldType::I64 => match value.as_i64() {
            Some(i) => out.extend_from_slice(&i.to_le_bytes()),
            None => out.extend_from_slice(&(integral(-9.007e15, 9.007e15)? as i64).to_le_bytes()),
        },
        FieldType::U64 => match value.as_u64() {
            Some(u) => out.extend_from_slice(&u.to_le_bytes()),
            None => out.extend_from_slice(&(integral(0.0, 9.007e15)? as u64).to_le_bytes()),
        },
        _ => unreachable!("lanes are scalars"),
    }
    Ok(())
}

/// A JSON value as one field's packed bytes, refused (with the reason) for
/// a value of another kind, a non-finite or non-integral number, or one
/// outside the schema's range. Nothing is clamped or guessed.
pub(crate) fn json_field(field: &FieldSchema, value: &Value) -> Result<Vec<u8>, String> {
    let lanes = field.ty.lanes();
    match field.ty {
        FieldType::Bool => {
            return match value.as_bool().or_else(|| {
                value
                    .as_f64()
                    .filter(|v| *v == 0.0 || *v == 1.0)
                    .map(|v| v == 1.0)
            }) {
                Some(on) => Ok(vec![u8::from(on)]),
                None => Err("a bool takes true, false, 0 or 1".into()),
            };
        }
        FieldType::Str(_) => {
            let text = value
                .as_str()
                .ok_or_else(|| format!("a {} takes a string, got {value}", field.ty.name()))?;
            return og_core::pack_str(text, field.ty.size());
        }
        _ => {}
    }
    let scalar = match field.ty {
        FieldType::Vec2 | FieldType::Vec3 | FieldType::Vec4 | FieldType::Quat | FieldType::Mat4 => {
            FieldType::F32
        }
        FieldType::Enum => FieldType::U32,
        other => other,
    };
    let mut out = Vec::with_capacity(field.ty.size());
    if lanes == 1 {
        lane(field, value, scalar, &mut out)?;
    } else {
        let items = value
            .as_array()
            .filter(|a| a.len() == lanes)
            .ok_or_else(|| format!("a {} takes an array of {lanes} numbers", field.ty.name()))?;
        for item in items {
            lane(field, item, scalar, &mut out)?;
        }
    }
    Ok(out)
}

/// Command text as the JSON value of `field` for [`json_field`]: a string
/// field takes the text as it is (`set:Prefs.name=Ada Lovelace`), or a JSON
/// string literal for escapes and the empty string (`set:Prefs.name=""`);
/// every other field takes [`text_json`].
pub(crate) fn command_value(field: &FieldSchema, text: &str) -> Result<Value, String> {
    if !field.ty.is_string() {
        return text_json(text);
    }
    if text.starts_with('"') {
        return serde_json::from_str::<String>(text)
            .map(Value::String)
            .map_err(|e| format!("{text} is not a JSON string literal ({e}); write the text bare, or quote it as JSON"));
    }
    Ok(Value::String(text.to_owned()))
}

/// Command text as JSON for [`json_field`]: `1.5`, `true` or `false` for a
/// bool, or `1,2,3` for a vector.
pub(crate) fn text_json(text: &str) -> Result<Value, String> {
    match text.trim() {
        "true" => return Ok(Value::Bool(true)),
        "false" => return Ok(Value::Bool(false)),
        _ => {}
    }
    let numbers = text
        .split(',')
        .map(|t| {
            let t = t.trim();
            serde_json::from_str::<Value>(t)
                .ok()
                .filter(Value::is_number)
                .ok_or_else(|| format!("'{t}' is not a number (or true or false)"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(if numbers.len() == 1 {
        numbers.into_iter().next().expect("one")
    } else {
        Value::Array(numbers)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn command_text_takes_true_and_false_for_a_bool() {
        let field = FieldSchema::new("ink", FieldType::Bool);
        for (text, byte) in [("true", 1), ("false", 0), (" true ", 1), ("1", 1), ("0", 0)] {
            let v = text_json(text).unwrap();
            assert_eq!(json_field(&field, &v).unwrap(), vec![byte], "{text}");
        }
        assert!(text_json("yes").is_err());
        assert!(json_field(&field, &text_json("2").unwrap()).is_err());
    }

    #[test]
    fn a_write_outside_the_range_is_refused_naming_the_range_and_the_fix() {
        let mut field = FieldSchema::new("cameraHeight", FieldType::F32);
        field.min = Some(4.0);
        field.max = Some(20.0);
        let low = json_field(&field, &json!(3.0)).unwrap_err();
        assert!(low.contains("below the minimum 4"), "{low}");
        assert!(low.contains("FieldOptions"), "{low}");
        let high = json_field(&field, &json!(21.0)).unwrap_err();
        assert!(high.contains("above the maximum 20"), "{high}");
        assert_eq!(
            json_field(&field, &json!(4.0)).unwrap(),
            4.0f32.to_le_bytes()
        );
    }
}
