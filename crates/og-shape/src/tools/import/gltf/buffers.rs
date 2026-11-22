//! A glTF document's bytes: buffers by URI (data, base64, files the resolver supplies), views (meshopt decoded) and accessors as f64.

use super::*;

/// A buffer view's or buffer's meshopt extension, under either name.
pub(crate) fn meshopt_of(v: &Value) -> &Value {
    let e = v.get("extensions");
    let c = e.get("EXT_meshopt_compression");
    if c.is_null() {
        e.get("KHR_meshopt_compression")
    } else {
        c
    }
}

/// Component bytes of a glTF component type.
pub(crate) fn component_size(ct: usize) -> Option<usize> {
    match ct {
        5120 | 5121 => Some(1),
        5122 | 5123 => Some(2),
        5125 | 5126 => Some(4),
        _ => None,
    }
}

pub(crate) fn read_component(b: &[u8], ct: usize, normalized: bool) -> f64 {
    match ct {
        5120 => {
            let v = f64::from(b[0] as i8);
            if normalized { (v / 127.0).max(-1.0) } else { v }
        }
        5121 => {
            let v = f64::from(b[0]);
            if normalized { v / 255.0 } else { v }
        }
        5122 => {
            let v = f64::from(i16::from_le_bytes([b[0], b[1]]));
            if normalized {
                (v / 32767.0).max(-1.0)
            } else {
                v
            }
        }
        5123 => {
            let v = f64::from(u16::from_le_bytes([b[0], b[1]]));
            if normalized { v / 65535.0 } else { v }
        }
        5125 => f64::from(u32::from_le_bytes([b[0], b[1], b[2], b[3]])),
        _ => f64::from(f32::from_le_bytes([b[0], b[1], b[2], b[3]])),
    }
}

impl Doc<'_> {
    /// The bytes of a buffer view, and its stride.
    pub(crate) fn view(&self, i: usize) -> Result<(&[u8], Option<usize>), ImportError> {
        let v = self.json.get("bufferViews").at(i);
        if let Some(bytes) = self.decoded.get(&i) {
            return Ok((bytes, v.get("byteStride").index()));
        }
        let buffer = v
            .get("buffer")
            .index()
            .and_then(|b| self.buffers.get(b))
            .ok_or_else(|| fmt_err(format!("bufferView {i} names no buffer")))?;
        let off = v.get("byteOffset").index().unwrap_or(0);
        let len = v
            .get("byteLength")
            .index()
            .ok_or_else(|| fmt_err(format!("bufferView {i} has no byteLength")))?;
        let bytes = buffer
            .get(off..off + len)
            .ok_or_else(|| fmt_err(format!("bufferView {i} runs past its buffer")))?;
        Ok((bytes, v.get("byteStride").index()))
    }

    /// An accessor's values, `width` per element, as f64.
    pub(crate) fn accessor(&self, i: usize) -> Result<(Vec<f64>, usize), ImportError> {
        let a = self.json.get("accessors").at(i);
        if a.is_null() {
            return Err(fmt_err(format!("no accessor {i}")));
        }
        let width = match a.get("type").str().unwrap_or("") {
            "SCALAR" => 1,
            "VEC2" => 2,
            "VEC3" => 3,
            "VEC4" => 4,
            "MAT4" => 16,
            t => return Err(fmt_err(format!("accessor {i} has type {t:?}"))),
        };
        let ct = a.get("componentType").index().unwrap_or(0);
        let size = component_size(ct)
            .ok_or_else(|| fmt_err(format!("accessor {i} has component type {ct}")))?;
        let count = a
            .get("count")
            .index()
            .ok_or_else(|| fmt_err(format!("accessor {i} has no count")))?;
        let normalized = a.get("normalized").bool().unwrap_or(false);
        let elem = width * size;
        let mut out = vec![0.0; count * width];
        if let Some(bv) = a.get("bufferView").index() {
            let (bytes, stride) = self.view(bv)?;
            let stride = stride.unwrap_or(elem);
            let off = a.get("byteOffset").index().unwrap_or(0);
            if count > 0 && off + (count - 1) * stride + elem > bytes.len() {
                return Err(fmt_err(format!("accessor {i} runs past its bufferView")));
            }
            for e in 0..count {
                for c in 0..width {
                    let at = off + e * stride + c * size;
                    out[e * width + c] = read_component(&bytes[at..at + size], ct, normalized);
                }
            }
        }
        let sparse = a.get("sparse");
        if !sparse.is_null() {
            let n = sparse.get("count").index().unwrap_or(0);
            let ind = sparse.get("indices");
            let ict = ind.get("componentType").index().unwrap_or(0);
            let isize = component_size(ict).ok_or_else(|| fmt_err("sparse index type"))?;
            let (ib, _) = self.view(ind.get("bufferView").index().unwrap_or(usize::MAX))?;
            let ioff = ind.get("byteOffset").index().unwrap_or(0);
            let val = sparse.get("values");
            let (vb, _) = self.view(val.get("bufferView").index().unwrap_or(usize::MAX))?;
            let voff = val.get("byteOffset").index().unwrap_or(0);
            if ioff + n * isize > ib.len() || voff + n * elem > vb.len() {
                return Err(fmt_err(format!(
                    "accessor {i}'s sparse data runs past its views"
                )));
            }
            for k in 0..n {
                let at = ioff + k * isize;
                let target = read_component(&ib[at..at + isize], ict, false) as usize;
                if target >= count {
                    return Err(fmt_err(format!(
                        "accessor {i}'s sparse index is out of range"
                    )));
                }
                for c in 0..width {
                    let at = voff + k * elem + c * size;
                    out[target * width + c] = read_component(&vb[at..at + size], ct, normalized);
                }
            }
        }
        Ok((out, width))
    }
}

pub(crate) fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && i + 2 < b.len()
            && let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16)
        {
            out.push(v);
            i += 3;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub(crate) fn base64(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
    let (mut acc, mut bits) = (0u32, 0u32);
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' | b'\n' | b'\r' | b' ' => continue,
            _ => return None,
        };
        acc = (acc << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    Some(out)
}

/// The bytes a URI names: a `data:` URI decoded, or the resolver's file.
pub(crate) fn fetch(uri: &str, resolve: Resolve) -> Result<Vec<u8>, ImportError> {
    if let Some(rest) = uri.strip_prefix("data:") {
        let (head, data) = rest
            .split_once(',')
            .ok_or_else(|| fmt_err("a data URI without ','"))?;
        if head.ends_with(";base64") {
            return base64(data).ok_or_else(|| fmt_err("a data URI that is not base64"));
        }
        return Ok(percent_decode(data).into_bytes());
    }
    let path = percent_decode(uri);
    resolve(&path).map_err(|why| ImportError::External { uri: path, why })
}
