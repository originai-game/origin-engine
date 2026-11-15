//! Binary values for UI patch batches. The tags are shared with
//! packages/engine/src/ui/reactive-wire.ts; no JSON text crosses this path.

use og_core::bytes::{ByteReader, ByteWriter};
use serde_json::{Map, Value};

const LIMIT: usize = 16 * 1024 * 1024;
const DEPTH: usize = 64;

pub fn encode(value: &Value) -> Result<Vec<u8>, String> {
    fn write(value: &Value, out: &mut Vec<u8>, depth: usize) -> Result<(), String> {
        if depth > DEPTH {
            return Err("UI patch nesting exceeds 64".into());
        }
        // A length past u32 is refused here, before og-core's writer would
        // panic on it.
        let fits = |n: usize, why: &str| u32::try_from(n).map(|_| ()).map_err(|_| why.to_string());
        let text = |s: &str, out: &mut Vec<u8>| -> Result<(), String> {
            fits(s.len(), "UI patch string is too long")?;
            ByteWriter::new(out).str(s);
            Ok(())
        };
        let mut w = ByteWriter::new(out);
        match value {
            Value::Null => w.u8(0),
            Value::Bool(false) => w.u8(1),
            Value::Bool(true) => w.u8(2),
            Value::Number(n) => {
                w.u8(3);
                let n = n
                    .as_f64()
                    .filter(|n| n.is_finite())
                    .ok_or("UI patch number must be finite")?;
                w.f64(n);
            }
            Value::String(s) => {
                w.u8(4);
                text(s, out)?;
            }
            Value::Array(a) => {
                w.u8(5);
                fits(a.len(), "UI patch array is too long")?;
                w.length(a.len());
                for value in a {
                    write(value, out, depth + 1)?;
                }
            }
            Value::Object(o) => {
                w.u8(6);
                fits(o.len(), "UI patch object is too large")?;
                w.length(o.len());
                for (key, value) in o {
                    text(key, out)?;
                    write(value, out, depth + 1)?;
                }
            }
        }
        if out.len() > LIMIT {
            return Err("UI patch exceeds 16 MiB".into());
        }
        Ok(())
    }
    let mut out = Vec::new();
    write(value, &mut out, 0)?;
    Ok(out)
}

pub fn decode(bytes: &[u8]) -> Result<Value, String> {
    struct Reader<'a>(ByteReader<'a>);
    fn why(e: og_core::Error) -> String {
        e.reason().to_string()
    }
    impl Reader<'_> {
        /// Every value is at least its tag's byte.
        fn count(&mut self) -> Result<usize, String> {
            self.0.count(1).map_err(why)
        }
        fn text(&mut self) -> Result<String, String> {
            self.0.string().map_err(why)
        }
        fn value(&mut self, depth: usize) -> Result<Value, String> {
            if depth > DEPTH {
                return Err("UI patch nesting exceeds 64".into());
            }
            Ok(match self.0.u8().map_err(why)? {
                0 => Value::Null,
                1 => Value::Bool(false),
                2 => Value::Bool(true),
                3 => {
                    let n = self.0.f64().map_err(why)?;
                    // A script has one number type; an integral one reads as
                    // an integer, as its JSON does, so integer fields
                    // (`TextSpan.position`, counts, indices) take it.
                    const EXACT: f64 = 9_007_199_254_740_992.0;
                    if n.fract() == 0.0 && n.abs() <= EXACT {
                        if n < 0.0 {
                            Value::Number((n as i64).into())
                        } else {
                            Value::Number((n as u64).into())
                        }
                    } else {
                        Value::Number(
                            serde_json::Number::from_f64(n)
                                .ok_or("UI patch number must be finite")?,
                        )
                    }
                }
                4 => Value::String(self.text()?),
                5 => {
                    let n = self.count()?;
                    let mut a = Vec::with_capacity(n.min(1024));
                    for _ in 0..n {
                        a.push(self.value(depth + 1)?);
                    }
                    Value::Array(a)
                }
                6 => {
                    let n = self.count()?;
                    let mut o = Map::new();
                    for _ in 0..n {
                        let key = self.text()?;
                        if o.insert(key, self.value(depth + 1)?).is_some() {
                            return Err("duplicate UI patch field".into());
                        }
                    }
                    Value::Object(o)
                }
                _ => return Err("unknown UI patch value tag".into()),
            })
        }
    }
    if bytes.len() > LIMIT {
        return Err("UI patch exceeds 16 MiB".into());
    }
    let mut reader = Reader(ByteReader::new(bytes, "UI patch"));
    let value = reader.value(0)?;
    reader.0.finish().map_err(why)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn binary_values_preserve_text_numbers_and_reject_truncation() {
        let value = serde_json::json!({"id":"背包/⚔️", "values":[null, false, true, -1.25, 4294967297_u64, -3, 0.5, ""]});
        let bytes = encode(&value).unwrap();
        assert_eq!(decode(&bytes).unwrap(), value);
        // A script's whole numbers arrive as integers whichever way they
        // were written, so a u32 field such as a span's position takes them.
        let span = serde_json::json!({"name":"gold","position":2.0,"length":4.0,"properties":{}});
        #[derive(serde::Deserialize)]
        struct TextSpan {
            position: u32,
            length: u32,
        }
        let span: TextSpan =
            serde_json::from_value(decode(&encode(&span).unwrap()).unwrap()).unwrap();
        assert_eq!((span.position, span.length), (2, 4));
        for end in 0..bytes.len() {
            assert!(
                decode(&bytes[..end]).is_err(),
                "accepted truncation at {end}"
            );
        }
        let mut bad = bytes.clone();
        bad.push(0);
        assert!(decode(&bad).is_err());
        assert!(decode(&[5, 255, 255, 255, 255]).is_err());
        let mut nan = vec![3];
        nan.extend_from_slice(&f64::NAN.to_le_bytes());
        assert!(decode(&nan).is_err());
    }
}
