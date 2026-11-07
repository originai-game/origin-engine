//! Binary values for UI patch batches. The tags are shared with
//! packages/engine/src/ui/reactive-wire.ts; no JSON text crosses this path.

use serde_json::{Map, Value};

const LIMIT: usize = 16 * 1024 * 1024;
const DEPTH: usize = 64;

pub fn encode(value: &Value) -> Result<Vec<u8>, String> {
    fn write(value: &Value, out: &mut Vec<u8>, depth: usize) -> Result<(), String> {
        if depth > DEPTH {
            return Err("UI patch nesting exceeds 64".into());
        }
        let text = |s: &str, out: &mut Vec<u8>| -> Result<(), String> {
            let n = u32::try_from(s.len()).map_err(|_| "UI patch string is too long")?;
            out.extend_from_slice(&n.to_le_bytes());
            out.extend_from_slice(s.as_bytes());
            Ok(())
        };
        match value {
            Value::Null => out.push(0),
            Value::Bool(false) => out.push(1),
            Value::Bool(true) => out.push(2),
            Value::Number(n) => {
                out.push(3);
                let n = n
                    .as_f64()
                    .filter(|n| n.is_finite())
                    .ok_or("UI patch number must be finite")?;
                out.extend_from_slice(&n.to_le_bytes());
            }
            Value::String(s) => {
                out.push(4);
                text(s, out)?;
            }
            Value::Array(a) => {
                out.push(5);
                out.extend_from_slice(
                    &u32::try_from(a.len())
                        .map_err(|_| "UI patch array is too long")?
                        .to_le_bytes(),
                );
                for value in a {
                    write(value, out, depth + 1)?;
                }
            }
            Value::Object(o) => {
                out.push(6);
                out.extend_from_slice(
                    &u32::try_from(o.len())
                        .map_err(|_| "UI patch object is too large")?
                        .to_le_bytes(),
                );
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
    struct Reader<'a> {
        bytes: &'a [u8],
        at: usize,
    }
    impl Reader<'_> {
        fn take(&mut self, count: usize) -> Result<&[u8], String> {
            let end = self
                .at
                .checked_add(count)
                .ok_or("UI patch length overflow")?;
            let value = self.bytes.get(self.at..end).ok_or("truncated UI patch")?;
            self.at = end;
            Ok(value)
        }
        fn count(&mut self) -> Result<usize, String> {
            let n = u32::from_le_bytes(self.take(4)?.try_into().expect("four bytes")) as usize;
            if n > self.bytes.len() - self.at {
                return Err("truncated UI patch sequence".into());
            }
            Ok(n)
        }
        fn text(&mut self) -> Result<String, String> {
            let n = self.count()?;
            std::str::from_utf8(self.take(n)?)
                .map(str::to_owned)
                .map_err(|_| "UI patch string is not UTF-8".into())
        }
        fn value(&mut self, depth: usize) -> Result<Value, String> {
            if depth > DEPTH {
                return Err("UI patch nesting exceeds 64".into());
            }
            Ok(match self.take(1)?[0] {
                0 => Value::Null,
                1 => Value::Bool(false),
                2 => Value::Bool(true),
                3 => {
                    let n = f64::from_le_bytes(self.take(8)?.try_into().expect("eight bytes"));
                    Value::Number(
                        serde_json::Number::from_f64(n).ok_or("UI patch number must be finite")?,
                    )
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
    let mut reader = Reader { bytes, at: 0 };
    let value = reader.value(0)?;
    if reader.at != bytes.len() {
        return Err("trailing UI patch bytes".into());
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn binary_values_preserve_text_numbers_and_reject_truncation() {
        let value = serde_json::json!({"id":"背包/⚔️", "values":[null, false, true, -1.25, 4294967297.0, ""]});
        let bytes = encode(&value).unwrap();
        assert_eq!(decode(&bytes).unwrap(), value);
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
