//! Owned author data, independent of a JS realm and of source provenance.
//!
//! The private cache layout is `OGAV\x01`, then a tagged tree: null/false/true
//! (0..2), f64 (3), UTF-16 string (4), array (5), object (6), numeric buffer
//! (16..24). Counts are u32; numbers and code units are little-endian. Objects
//! use UTF-16 key order, arrays retain order. No text-number expansion occurs.
//! The encoder in `../generator_value.js` is the only writer; both sources
//! participate in the action identity. This is not a published save format.

use serde_json::Value as Json;
use std::collections::BTreeMap;

// Keep one output at most a quarter of the realm's 256 MiB limit, allowing
// room for source data and the writer's growing buffer. These are per-entry
// bounds, not project limits: split worlds into content entries. Node and
// depth caps also bound native tree allocation/recursion for compact input.
const MAX_BYTES: usize = 64 * 1024 * 1024;
const MAX_NODES: usize = 1_000_000;
const MAX_DEPTH: usize = 128;
const MAGIC: &[u8] = b"OGAV\x01";

/// Canonical bytes owned by Rust, with no borrowed VM values or handles.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Data {
    bytes: Vec<u8>,
}

impl Data {
    /// Validate bytes from the evaluator or its authenticated action cache.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, String> {
        let data = Self { bytes };
        data.read()?;
        Ok(data)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Buffers borrow this owned allocation, never the JS heap. Object/string
    /// metadata is decoded on demand; samples are not expanded into nodes.
    pub fn root(&self) -> Value<'_> {
        self.read().expect("Data was validated on construction")
    }

    fn read(&self) -> Result<Value<'_>, String> {
        if self.bytes.len() > MAX_BYTES || !self.bytes.starts_with(MAGIC) {
            return Err("invalid author IR header or output exceeds 64 MiB".into());
        }
        let mut reader = Reader {
            bytes: &self.bytes[MAGIC.len()..],
            nodes: 0,
        };
        let value = reader.value(0)?;
        if !reader.bytes.is_empty() {
            return Err("trailing author IR bytes".into());
        }
        Ok(value)
    }
}

/// Structural data. Only numeric buffers retain a distinct element type;
/// schema-driven f32/f64 conversion remains the domain decoder's job.
#[derive(Debug, PartialEq)]
pub enum Value<'a> {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value<'a>>),
    Object(BTreeMap<String, Value<'a>>),
    Buffer(Buffer<'a>),
}

impl Value<'_> {
    /// Projection for existing schema/serde and `og script --json` consumers.
    /// Numeric expansion is explicit here, not at the VM/cache boundary.
    pub fn into_json(self) -> Json {
        match self {
            Self::Null => Json::Null,
            Self::Bool(value) => value.into(),
            Self::Number(value) => number(value),
            Self::String(value) => value.into(),
            Self::Array(values) => values.into_iter().map(Self::into_json).collect(),
            Self::Object(values) => values
                .into_iter()
                .map(|(key, value)| (key, value.into_json()))
                .collect(),
            Self::Buffer(buffer) => buffer.numbers().map(number).collect(),
        }
    }
}

fn number(value: f64) -> Json {
    // Retain integer JSON values where exact. Bounds are exclusive because
    // f64 rounds u64::MAX and i64::MAX to the next power of two.
    if value.fract() == 0.0 && value >= 0.0 && value < u64::MAX as f64 {
        Json::from(value as u64)
    } else if value.fract() == 0.0 && value >= i64::MIN as f64 && value < 0.0 {
        Json::from(value as i64)
    } else {
        Json::from(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Element {
    I8,
    U8,
    U8Clamped,
    I16,
    U16,
    I32,
    U32,
    F32,
    F64,
}

impl Element {
    pub fn width(self) -> usize {
        match self {
            Self::I8 | Self::U8 | Self::U8Clamped => 1,
            Self::I16 | Self::U16 => 2,
            Self::I32 | Self::U32 | Self::F32 => 4,
            Self::F64 => 8,
        }
    }
}

/// A validated, little-endian view; only the typed array's slice is included,
/// never unrelated bytes in its backing ArrayBuffer. Do not assume alignment.
#[derive(Debug, PartialEq)]
pub struct Buffer<'a> {
    pub element: Element,
    bytes: &'a [u8],
}

impl<'a> Buffer<'a> {
    pub fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }

    pub fn len(&self) -> usize {
        self.bytes.len() / self.element.width()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn numbers(&self) -> impl ExactSizeIterator<Item = f64> + '_ {
        self.bytes
            .chunks_exact(self.element.width())
            .map(|bytes| match self.element {
                Element::I8 => f64::from(bytes[0] as i8),
                Element::U8 | Element::U8Clamped => f64::from(bytes[0]),
                Element::I16 => f64::from(i16::from_le_bytes(bytes.try_into().unwrap())),
                Element::U16 => f64::from(u16::from_le_bytes(bytes.try_into().unwrap())),
                Element::I32 => f64::from(i32::from_le_bytes(bytes.try_into().unwrap())),
                Element::U32 => f64::from(u32::from_le_bytes(bytes.try_into().unwrap())),
                Element::F32 => f64::from(f32::from_le_bytes(bytes.try_into().unwrap())),
                Element::F64 => f64::from_le_bytes(bytes.try_into().unwrap()),
            })
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    nodes: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], String> {
        let (head, tail) = self
            .bytes
            .split_at_checked(count)
            .ok_or("truncated author IR")?;
        self.bytes = tail;
        Ok(head)
    }

    fn count(&mut self) -> Result<usize, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()) as usize)
    }

    fn string(&mut self) -> Result<String, String> {
        let count = self.count()?;
        let bytes = self.take(count.checked_mul(2).ok_or("author string size overflow")?)?;
        char::decode_utf16(
            bytes
                .as_chunks::<2>()
                .0
                .iter()
                .map(|b| u16::from_le_bytes(*b)),
        )
        .collect::<Result<String, _>>()
        .map_err(|e| format!("invalid Unicode in author IR: {e}"))
    }

    fn value(&mut self, depth: usize) -> Result<Value<'a>, String> {
        self.nodes += 1;
        if self.nodes > MAX_NODES {
            return Err("author IR exceeds one million nodes".into());
        }
        let tag = self.take(1)?[0];
        match tag {
            0 => Ok(Value::Null),
            1 | 2 => Ok(Value::Bool(tag == 2)),
            3 => {
                let value = f64::from_le_bytes(self.take(8)?.try_into().unwrap());
                if !value.is_finite() || value == 0.0 && value.is_sign_negative() {
                    return Err("non-finite or non-canonical author number".into());
                }
                Ok(Value::Number(value))
            }
            4 => Ok(Value::String(self.string()?)),
            _ if depth >= MAX_DEPTH => Err("author IR nesting exceeds 128".into()),
            5 | 6 => {
                let count = self.count()?;
                if count > self.bytes.len() || count > MAX_NODES - self.nodes {
                    return Err(
                        "author IR collection exceeds its remaining bytes/node budget".into(),
                    );
                }
                if tag == 5 {
                    return (0..count)
                        .map(|_| self.value(depth + 1))
                        .collect::<Result<_, _>>()
                        .map(Value::Array);
                }
                let mut values = BTreeMap::new();
                let mut previous: Option<String> = None;
                for _ in 0..count {
                    let key = self.string()?;
                    if previous
                        .as_ref()
                        .is_some_and(|prev| prev.encode_utf16().cmp(key.encode_utf16()).is_ge())
                    {
                        return Err(
                            "author IR object keys must be unique and in UTF-16 order".into()
                        );
                    }
                    previous = Some(key.clone());
                    values.insert(key, self.value(depth + 1)?);
                }
                Ok(Value::Object(values))
            }
            16..=24 => {
                let element = match tag {
                    16 => Element::I8,
                    17 => Element::U8,
                    18 => Element::U8Clamped,
                    19 => Element::I16,
                    20 => Element::U16,
                    21 => Element::I32,
                    22 => Element::U32,
                    23 => Element::F32,
                    24 => Element::F64,
                    _ => unreachable!(),
                };
                let count = self.count()?;
                let bytes = self.take(
                    count
                        .checked_mul(element.width())
                        .ok_or("author buffer size overflow")?,
                )?;
                let buffer = Buffer { element, bytes };
                if matches!(element, Element::F32 | Element::F64)
                    && buffer.numbers().any(|value| !value.is_finite())
                {
                    return Err("non-finite author buffer sample".into());
                }
                Ok(Value::Buffer(buffer))
            }
            _ => Err(format!("unknown author IR tag {tag}")),
        }
    }
}
