//! Native layer payloads. Tailwind values remain structured data, never text.
use super::Layer;
use bincode::Options;
use serde::{Deserialize, Serialize};

impl Layer {
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .serialize(self)
            .map_err(|e| format!("native UI pack: {e}"))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .with_limit(bytes.len() as u64)
            .reject_trailing_bytes()
            .deserialize(bytes)
            .map_err(|e| format!("native UI pack: {e}"))
    }
}

/// A tagged binary value, unlike JSON's deserialize_any representation.
#[derive(Serialize, Deserialize)]
enum Value {
    Null,
    Bool(bool),
    I64(i64),
    U64(u64),
    F64(f64),
    String(String),
    Array(Vec<Value>),
    Object(std::collections::BTreeMap<String, Value>),
}

impl From<serde_json::Value> for Value {
    fn from(value: serde_json::Value) -> Self {
        use serde_json::Value as J;
        match value {
            J::Null => Self::Null,
            J::Bool(b) => Self::Bool(b),
            J::Number(n) => match (n.as_i64(), n.as_u64()) {
                (Some(n), _) => Self::I64(n),
                (_, Some(n)) => Self::U64(n),
                _ => Self::F64(n.as_f64().expect("JSON number")),
            },
            J::String(s) => Self::String(s),
            J::Array(a) => Self::Array(a.into_iter().map(Into::into).collect()),
            J::Object(m) => Self::Object(m.into_iter().map(|(k, v)| (k, v.into())).collect()),
        }
    }
}

impl TryFrom<Value> for serde_json::Value {
    type Error = &'static str;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value {
            Value::Null => Self::Null,
            Value::Bool(b) => Self::Bool(b),
            Value::I64(n) => n.into(),
            Value::U64(n) => n.into(),
            Value::F64(n) => serde_json::Number::from_f64(n)
                .ok_or("nonfinite theme value")?
                .into(),
            Value::String(s) => Self::String(s),
            Value::Array(a) => Self::Array(
                a.into_iter()
                    .map(Self::try_from)
                    .collect::<Result<_, _>>()?,
            ),
            Value::Object(m) => Self::Object(
                m.into_iter()
                    .map(|(k, v)| Ok((k, v.try_into()?)))
                    .collect::<Result<_, Self::Error>>()?,
            ),
        })
    }
}

pub(super) mod themes {
    use super::*;
    type Themes = [serde_json::Map<String, serde_json::Value>; 2];

    pub fn serialize<S: serde::Serializer>(
        value: &Themes,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        value
            .clone()
            .map(|m| Value::from(serde_json::Value::Object(m)))
            .serialize(serializer)
    }

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Themes, D::Error> {
        let values = <[Value; 2]>::deserialize(deserializer)?;
        let mut out: Themes = Default::default();
        for (slot, value) in out.iter_mut().zip(values) {
            let value = serde_json::Value::try_from(value).map_err(serde::de::Error::custom)?;
            *slot = value
                .as_object()
                .ok_or_else(|| serde::de::Error::custom("theme is not an object"))?
                .clone();
        }
        Ok(out)
    }
}
