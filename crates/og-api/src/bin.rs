//! `OGB1` binary wire envelope for TypedArrays and raw byte buffers crossing the
//! host boundary (`Self::Bytes` / `Uint8Array`), eliminating JSON number
//! serialization and base64 overhead while preserving exact IEEE-754 bit patterns
//! (including `NaN`/`Infinity` for runtime validation) and full structural errors.
//!
//! Layout (all little-endian, 8-byte aligned):
//! - Header (16 bytes): `magic = b"OGB1"`, `flags: u32 (0)`, `json_len: u32`, `blob_count: u32`.
//! - Blob table (`16 * blob_count` bytes): `(tag: u32, elem_count: u32, byte_offset: u32, byte_len: u32)`.
//!   - `tag`: `0 = Uint8Array`, `1 = Int32Array`, `2 = Uint32Array`, `3 = Float32Array`, `4 = Float64Array`.
//! - UTF-8 JSON skeleton (`json_len` bytes), zero-padded to an 8-byte boundary.
//!   TypedArrays appear in JSON as `{"$b": blob_index}`.
//! - Raw little-endian blob payloads, each starting at an 8-byte boundary so JS
//!   can construct zero-copy TypedArray views over the returned `ArrayBuffer`.

use crate::host::wire::{self, FromJson, ToJson};
use serde_json::{Map, Value};
use std::cell::RefCell;

pub const MAGIC: &[u8; 4] = b"OGB1";

pub const TAG_U8: u32 = 0;
pub const TAG_I32: u32 = 1;
pub const TAG_U32: u32 = 2;
pub const TAG_F32: u32 = 3;
pub const TAG_F64: u32 = 4;

fn elem_size(tag: u32) -> Option<usize> {
    match tag {
        TAG_U8 => Some(1),
        TAG_I32 | TAG_U32 | TAG_F32 => Some(4),
        TAG_F64 => Some(8),
        _ => None,
    }
}

pub fn tag_name(tag: u32) -> &'static str {
    match tag {
        TAG_U8 => "Uint8Array",
        TAG_I32 => "Int32Array",
        TAG_U32 => "Uint32Array",
        TAG_F32 => "Float32Array",
        TAG_F64 => "Float64Array",
        _ => "TypedArray",
    }
}

const fn align8(n: usize) -> usize {
    (n + 7) & !7
}

fn read_u32(buf: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]])
}

#[derive(Clone, Copy)]
struct BlobEntry {
    tag: u32,
    offset: usize,
    len: usize,
}

struct InputPacket {
    buf: Vec<u8>,
    blobs: Vec<BlobEntry>,
}

#[derive(Default)]
struct OutputWriter {
    blobs: Vec<(u32, u32, Vec<u8>)>,
}

thread_local! {
    static BIN_IN: RefCell<Option<InputPacket>> = const { RefCell::new(None) };
    static BIN_OUT: RefCell<Option<OutputWriter>> = const { RefCell::new(None) };
}

struct InputGuard(Option<InputPacket>);

impl InputGuard {
    fn enter(pkt: InputPacket) -> Self {
        let prev = BIN_IN.with(|c| c.borrow_mut().replace(pkt));
        Self(prev)
    }
}

impl Drop for InputGuard {
    fn drop(&mut self) {
        let prev = self.0.take();
        BIN_IN.with(|c| {
            *c.borrow_mut() = prev;
        });
    }
}

struct OutputGuard(Option<OutputWriter>);

impl OutputGuard {
    fn enter() -> Self {
        let prev = BIN_OUT.with(|c| c.borrow_mut().replace(OutputWriter::default()));
        Self(prev)
    }

    fn finish(mut self) -> OutputWriter {
        let prev = self.0.take();
        let writer = BIN_OUT.with(|c| std::mem::replace(&mut *c.borrow_mut(), prev));
        std::mem::forget(self);
        writer.unwrap_or_default()
    }
}

impl Drop for OutputGuard {
    fn drop(&mut self) {
        let prev = self.0.take();
        BIN_OUT.with(|c| {
            *c.borrow_mut() = prev;
        });
    }
}

/// If `v` is a binary blob placeholder `{"$b": idx}` while decoding an `OGB1`
/// packet, returns `Some(idx)`.
pub fn blob_index(v: &Value) -> Option<usize> {
    let o = v.as_object()?;
    if o.len() != 1 {
        return None;
    }
    let idx = o.get("$b")?.as_u64()?;
    usize::try_from(idx).ok()
}

/// Returns the tag of blob `idx` in the active input packet, if any.
pub fn blob_tag(idx: usize) -> Option<u32> {
    BIN_IN.with(|c| c.borrow().as_ref()?.blobs.get(idx).map(|b| b.tag))
}

/// Calls `f` with `(tag, raw_bytes)` of blob `idx` in the active input packet.
pub fn with_blob<R>(idx: usize, f: impl FnOnce(u32, &[u8]) -> R) -> Option<R> {
    BIN_IN.with(|c| {
        let guard = c.borrow();
        let pkt = guard.as_ref()?;
        let entry = *pkt.blobs.get(idx)?;
        Some(f(
            entry.tag,
            &pkt.buf[entry.offset..entry.offset + entry.len],
        ))
    })
}

/// Decodes a binary `OGB1` argument (or falls back to plain UTF-8 JSON when
/// not prefixed by `OGB1`) into typed `T`.
pub fn arg<T: FromJson + Default>(bytes: Option<Vec<u8>>, name: &str) -> Result<T, String> {
    let Some(bytes) = bytes else {
        return Ok(T::default());
    };
    if bytes.is_empty() {
        return Ok(T::default());
    }
    if !bytes.starts_with(MAGIC) {
        let text =
            std::str::from_utf8(&bytes).map_err(|e| format!("{name}: invalid utf-8: {e}"))?;
        return wire::arg(Some(text), name);
    }
    if bytes.len() < 16 {
        return Err(format!("{name}: truncated OGB1 header"));
    }
    let _flags = read_u32(&bytes, 4);
    let json_len = read_u32(&bytes, 8) as usize;
    let blob_count = read_u32(&bytes, 12) as usize;
    let table_bytes = blob_count
        .checked_mul(16)
        .ok_or_else(|| format!("{name}: OGB1 blob count overflow"))?;
    let json_start = 16usize
        .checked_add(table_bytes)
        .ok_or_else(|| format!("{name}: OGB1 header overflow"))?;
    let json_end = json_start
        .checked_add(json_len)
        .ok_or_else(|| format!("{name}: OGB1 json length overflow"))?;
    if json_end > bytes.len() {
        return Err(format!("{name}: OGB1 json extends past packet end"));
    }
    let mut blobs = Vec::with_capacity(blob_count);
    for i in 0..blob_count {
        let base = 16 + i * 16;
        let tag = read_u32(&bytes, base);
        let elem_count = read_u32(&bytes, base + 4) as usize;
        let offset = read_u32(&bytes, base + 8) as usize;
        let byte_len = read_u32(&bytes, base + 12) as usize;
        let stride = elem_size(tag)
            .ok_or_else(|| format!("{name}: unknown OGB1 blob tag {tag} at blob {i}"))?;
        if elem_count.checked_mul(stride) != Some(byte_len) {
            return Err(format!("{name}: OGB1 blob {i} length mismatch"));
        }
        let end = offset
            .checked_add(byte_len)
            .ok_or_else(|| format!("{name}: OGB1 blob {i} offset overflow"))?;
        if end > bytes.len() {
            return Err(format!("{name}: OGB1 blob {i} extends past packet end"));
        }
        blobs.push(BlobEntry {
            tag,
            offset,
            len: byte_len,
        });
    }
    let val: Value = serde_json::from_slice(&bytes[json_start..json_end])
        .map_err(|e| format!("{name}: invalid OGB1 json: {e}"))?;
    let _guard = InputGuard::enter(InputPacket { buf: bytes, blobs });
    wire::value(&val, name)
}

fn push_blob(tag: u32, elem_count: usize, raw: Vec<u8>) -> Option<Value> {
    let count = u32::try_from(elem_count).ok()?;
    BIN_OUT.with(|c| {
        let mut guard = c.borrow_mut();
        let w = guard.as_mut()?;
        let idx = w.blobs.len();
        w.blobs.push((tag, count, raw));
        let mut m = Map::new();
        m.insert("$b".into(), Value::from(idx as u64));
        Some(Value::Object(m))
    })
}

/// Serialises a `Float32Array` field as a binary blob when inside [`encode`],
/// falling back to a plain JSON array outside.
pub fn to_bin_f32(v: &[f32]) -> Value {
    let active = BIN_OUT.with(|c| c.borrow().is_some());
    if !active {
        return v.to_json();
    }
    let mut raw = Vec::with_capacity(v.len() * 4);
    for &x in v {
        raw.extend_from_slice(&x.to_le_bytes());
    }
    push_blob(TAG_F32, v.len(), raw).unwrap_or_else(|| v.to_json())
}

/// Serialises a `Float64Array` field as a binary blob when inside [`encode`].
pub fn to_bin_f64(v: &[f64]) -> Value {
    let active = BIN_OUT.with(|c| c.borrow().is_some());
    if !active {
        return v.to_json();
    }
    let mut raw = Vec::with_capacity(v.len() * 8);
    for &x in v {
        raw.extend_from_slice(&x.to_le_bytes());
    }
    push_blob(TAG_F64, v.len(), raw).unwrap_or_else(|| v.to_json())
}

/// Serialises a `Uint32Array` field as a binary blob when inside [`encode`].
pub fn to_bin_u32(v: &[u32]) -> Value {
    let active = BIN_OUT.with(|c| c.borrow().is_some());
    if !active {
        return v.to_json();
    }
    let mut raw = Vec::with_capacity(v.len() * 4);
    for &x in v {
        raw.extend_from_slice(&x.to_le_bytes());
    }
    push_blob(TAG_U32, v.len(), raw).unwrap_or_else(|| v.to_json())
}

/// Serialises an `Int32Array` field as a binary blob when inside [`encode`].
pub fn to_bin_i32(v: &[i32]) -> Value {
    let active = BIN_OUT.with(|c| c.borrow().is_some());
    if !active {
        return v.to_json();
    }
    let mut raw = Vec::with_capacity(v.len() * 4);
    for &x in v {
        raw.extend_from_slice(&x.to_le_bytes());
    }
    push_blob(TAG_I32, v.len(), raw).unwrap_or_else(|| v.to_json())
}

/// Serialises a `Uint8Array` / `Bytes` field as a binary blob when inside [`encode`].
pub fn to_bin_u8(v: &[u8]) -> Value {
    let active = BIN_OUT.with(|c| c.borrow().is_some());
    if !active {
        return v.to_json();
    }
    push_blob(TAG_U8, v.len(), v.to_vec()).unwrap_or_else(|| v.to_json())
}

fn pack(json_bytes: &[u8], blobs: Vec<(u32, u32, Vec<u8>)>) -> Vec<u8> {
    let blob_count = blobs.len();
    let header_end = 16 + blob_count * 16;
    let json_end = header_end + json_bytes.len();
    let mut cursor = align8(json_end);
    let mut offsets = Vec::with_capacity(blob_count);
    for (_, _, raw) in &blobs {
        offsets.push(cursor);
        cursor = align8(cursor + raw.len());
    }
    let mut out = Vec::with_capacity(cursor);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(&(blob_count as u32).to_le_bytes());
    for ((tag, count, raw), &off) in blobs.iter().zip(&offsets) {
        out.extend_from_slice(&tag.to_le_bytes());
        out.extend_from_slice(&count.to_le_bytes());
        out.extend_from_slice(&(off as u32).to_le_bytes());
        out.extend_from_slice(&(raw.len() as u32).to_le_bytes());
    }
    out.extend_from_slice(json_bytes);
    out.resize(align8(out.len()), 0);
    for (_, _, raw) in blobs {
        out.extend_from_slice(&raw);
        out.resize(align8(out.len()), 0);
    }
    out
}

/// Encodes a JSON skeleton built inside `f` together with any TypedArray blobs
/// pushed by `to_bin_*` into an 8-byte-aligned `OGB1` packet.
pub fn encode(f: impl FnOnce() -> Value) -> Vec<u8> {
    let guard = OutputGuard::enter();
    let value = f();
    let writer = guard.finish();
    let json_bytes = serde_json::to_vec(&value).unwrap_or_else(|_| b"null".to_vec());
    pack(&json_bytes, writer.blobs)
}

/// Encodes a structured value `T` as an `OGB1` packet.
pub fn value<T: ToJson>(v: &T) -> Vec<u8> {
    encode(|| v.to_json())
}

/// Encodes `Result<T, String>` (`{ ok: true, value }` or `{ ok: false, reason }`)
/// as an `OGB1` packet.
pub fn answer<T: ToJson>(r: Result<T, String>) -> Vec<u8> {
    encode(|| match r {
        Ok(_) if T::VOID => serde_json::json!({ "ok": true }),
        Ok(v) => serde_json::json!({ "ok": true, "value": v.to_json() }),
        Err(reason) => serde_json::json!({ "ok": false, "reason": reason }),
    })
}

/// Encodes `Result<Vec<f32>, String>` directly as `{ ok: true, value: Float32Array }`.
pub fn answer_f32s(r: Result<Vec<f32>, String>) -> Vec<u8> {
    encode(|| match r {
        Ok(v) => serde_json::json!({ "ok": true, "value": to_bin_f32(&v) }),
        Err(reason) => serde_json::json!({ "ok": false, "reason": reason }),
    })
}

/// Encodes `Result<Vec<u8>, String>` directly as `{ ok: true, value: Uint8Array }`.
pub fn answer_bytes(r: Result<Vec<u8>, String>) -> Vec<u8> {
    encode(|| match r {
        Ok(v) => serde_json::json!({ "ok": true, "value": to_bin_u8(&v) }),
        Err(reason) => serde_json::json!({ "ok": false, "reason": reason }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_typed_blobs_preserving_nan_and_alignment() {
        let floats = vec![1.5f32, -0.25, f32::NAN, f32::INFINITY, 42.0];
        let ints = vec![-7i32, 0, 123456];
        let bytes = vec![1u8, 2, 255];
        let packet = encode(|| {
            serde_json::json!({
                "f": to_bin_f32(&floats),
                "i": to_bin_i32(&ints),
                "u": to_bin_u8(&bytes),
            })
        });
        assert!(packet.starts_with(MAGIC));
        assert_eq!(packet.len() % 8, 0);
        // Check every blob offset is 8-byte aligned.
        let blob_count = read_u32(&packet, 12) as usize;
        assert_eq!(blob_count, 3);
        for k in 0..blob_count {
            let off = read_u32(&packet, 16 + k * 16 + 8) as usize;
            assert_eq!(off % 8, 0);
        }
        // Decode through wire::FromJson
        #[derive(Default)]
        struct Boxed {
            f: Vec<f32>,
            i: Vec<i32>,
            u: Vec<u8>,
        }
        impl FromJson for Boxed {
            fn from_json(v: &Value) -> Result<Self, String> {
                let o = wire::object(v)?;
                Ok(Self {
                    f: wire::field(o, "f", Vec::new())?,
                    i: wire::field(o, "i", Vec::new())?,
                    u: wire::field(o, "u", Vec::new())?,
                })
            }
        }
        let decoded: Boxed = arg(Some(packet), "test").expect("decode");
        assert_eq!(decoded.f.len(), 5);
        assert_eq!(decoded.f[0], 1.5);
        assert_eq!(decoded.f[1], -0.25);
        assert!(decoded.f[2].is_nan());
        assert!(decoded.f[3].is_infinite());
        assert_eq!(decoded.f[4], 42.0);
        assert_eq!(decoded.i, ints);
        assert_eq!(decoded.u, bytes);
    }
}
