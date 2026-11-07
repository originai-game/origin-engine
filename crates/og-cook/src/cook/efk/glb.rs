//! The meshes an imported effect draws, written as glTF binaries: an
//! Effekseer ring (its arc, its inner and outer radius, its colours across
//! as vertex colours and its UVs as Effekseer lays them,
//! `RingRendererBase.h`) and an `.efkmodel` (`Model/Effekseer.Model.cpp`;
//! both MIT, `third_party/effekseer/`).

/// A vertex: position, normal, uv, linear colour.
pub struct V {
    pub p: [f32; 3],
    pub n: [f32; 3],
    pub uv: [f32; 2],
    pub c: [f32; 4],
}

/// A glTF binary of one mesh with one unlit-agnostic primitive.
pub fn write(vs: &[V], indices: &[u32]) -> Vec<u8> {
    let mut bin: Vec<u8> = Vec::new();
    let mut views = Vec::new();
    let mut push = |bytes: Vec<u8>, target: u32| {
        while !bin.len().is_multiple_of(4) {
            bin.push(0);
        }
        views.push(serde_json::json!({
            "buffer": 0, "byteOffset": bin.len(), "byteLength": bytes.len(), "target": target
        }));
        bin.extend_from_slice(&bytes);
        views.len() - 1
    };
    let floats = |f: &dyn Fn(&V) -> Vec<f32>| -> Vec<u8> {
        vs.iter()
            .flat_map(f)
            .flat_map(|x: f32| x.to_le_bytes())
            .collect()
    };
    let p = push(floats(&|v| v.p.to_vec()), 34962);
    let n = push(floats(&|v| v.n.to_vec()), 34962);
    let t = push(floats(&|v| v.uv.to_vec()), 34962);
    let c = push(floats(&|v| v.c.to_vec()), 34962);
    let i = push(
        indices.iter().flat_map(|i| i.to_le_bytes()).collect(),
        34963,
    );
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for v in vs {
        for k in 0..3 {
            lo[k] = lo[k].min(v.p[k]);
            hi[k] = hi[k].max(v.p[k]);
        }
    }
    if vs.is_empty() {
        (lo, hi) = ([0.0; 3], [0.0; 3]);
    }
    let count = vs.len();
    let json = serde_json::json!({
        "asset": { "version": "2.0", "generator": "og vfx import (Effekseer)" },
        "scene": 0,
        "scenes": [{ "nodes": [0] }],
        "nodes": [{ "mesh": 0 }],
        "meshes": [{ "primitives": [{
            "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2, "COLOR_0": 3 },
            "indices": 4,
            "material": 0
        }] }],
        "materials": [{ "pbrMetallicRoughness": { "metallicFactor": 0.0 }, "doubleSided": true }],
        "accessors": [
            { "bufferView": p, "componentType": 5126, "count": count, "type": "VEC3", "min": lo, "max": hi },
            { "bufferView": n, "componentType": 5126, "count": count, "type": "VEC3" },
            { "bufferView": t, "componentType": 5126, "count": count, "type": "VEC2" },
            { "bufferView": c, "componentType": 5126, "count": count, "type": "VEC4" },
            { "bufferView": i, "componentType": 5125, "count": indices.len(), "type": "SCALAR" }
        ],
        "bufferViews": views,
        "buffers": [{ "byteLength": bin.len() }]
    });
    let mut j = serde_json::to_vec(&json).expect("glTF JSON");
    while !j.len().is_multiple_of(4) {
        j.push(b' ');
    }
    while !bin.len().is_multiple_of(4) {
        bin.push(0);
    }
    let total = 12 + 8 + j.len() + 8 + bin.len();
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(b"glTF");
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&(j.len() as u32).to_le_bytes());
    out.extend_from_slice(b"JSON");
    out.extend_from_slice(&j);
    out.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    out.extend_from_slice(b"BIN\0");
    out.extend_from_slice(&bin);
    out
}

/// A ring in the xy plane facing +z, of outer radius 1: `inner` the inner
/// radius, `center` where its middle band lies (0 inner, 1 outer), the arc
/// from `start` to `end` degrees (Effekseer's 0 is +y), `segments` around;
/// colours (linear) at the outer edge, the middle band and the inner edge.
/// Its v runs 0 at the outer edge to 1 at the inner, u 0 to 1 along the arc.
pub fn ring(
    inner: f32,
    center: f32,
    start: f32,
    end: f32,
    segments: u32,
    colors: [[f32; 4]; 3],
) -> Vec<u8> {
    let segments = segments.clamp(3, 256);
    let mid = inner + (1.0 - inner) * center;
    let rows = [
        (1.0, 0.0, colors[0]),
        (mid, 0.5, colors[1]),
        (inner, 1.0, colors[2]),
    ];
    let mut vs = Vec::new();
    for i in 0..=segments {
        let u = i as f32 / segments as f32;
        let a = (start + 90.0 + (end - start) * u).to_radians();
        let (s, c) = og_core::math::sin_cos(a);
        for (r, v, col) in rows {
            vs.push(V {
                p: [c * r, s * r, 0.0],
                n: [0.0, 0.0, 1.0],
                uv: [u, v],
                c: col,
            });
        }
    }
    let mut idx = Vec::new();
    for i in 0..segments {
        for row in 0..2 {
            let a = i * 3 + row;
            let b = a + 3;
            idx.extend_from_slice(&[a, b, b + 1, a, b + 1, a + 1]);
        }
    }
    write(&vs, &idx)
}

/// An `.efkmodel`'s first frame as a glTF binary.
pub fn efkmodel(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut at = 0usize;
    let mut take = |n: usize| -> Result<&[u8], String> {
        let s = bytes.get(at..at + n).ok_or("the .efkmodel ends early")?;
        at += n;
        Ok(s)
    };
    let i32_ = |b: &[u8]| i32::from_le_bytes(b.try_into().expect("4 bytes"));
    let f =
        |b: &[u8], k: usize| f32::from_le_bytes(b[k * 4..k * 4 + 4].try_into().expect("4 bytes"));
    let version = i32_(take(4)?);
    if !(0..=6).contains(&version) {
        return Err(format!(".efkmodel version {version} (0 to 6 are read)"));
    }
    if version == 2 || version >= 5 {
        take(4)?;
    }
    take(4)?; // model count
    if version >= 5 {
        take(4)?; // frame count: the first frame is read
    }
    let count = usize::try_from(i32_(take(4)?)).map_err(|_| "a negative vertex count")?;
    let size = if version >= 6 {
        68
    } else {
        56 + if version >= 1 { 4 } else { 0 }
    };
    if count.saturating_mul(size) > bytes.len() {
        return Err("the .efkmodel's vertex count runs past its end".into());
    }
    let mut vs = Vec::with_capacity(count);
    for _ in 0..count {
        let v = take(size)?;
        let col = if version >= 1 {
            &v[size - 4..]
        } else {
            &[255u8; 4][..]
        };
        let lin = |c: u8| og_core::math::pow(f32::from(c) / 255.0, 2.2);
        vs.push(V {
            p: [f(v, 0), f(v, 1), f(v, 2)],
            n: [f(v, 3), f(v, 4), f(v, 5)],
            uv: [f(v, 12), f(v, 13)],
            c: [
                lin(col[0]),
                lin(col[1]),
                lin(col[2]),
                f32::from(col[3]) / 255.0,
            ],
        });
    }
    let faces = usize::try_from(i32_(take(4)?)).map_err(|_| "a negative face count")?;
    let mut idx = Vec::with_capacity(faces * 3);
    for _ in 0..faces {
        let fb = take(12)?;
        for k in 0..3 {
            let i = i32_(&fb[k * 4..k * 4 + 4]);
            if i < 0 || i as usize >= count {
                return Err("an .efkmodel face names a vertex it does not have".into());
            }
            idx.push(i as u32);
        }
    }
    Ok(write(&vs, &idx))
}
