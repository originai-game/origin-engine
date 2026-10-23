//! A scene's baked lighting as `og bake lightmaps` writes it beside the
//! scene (docs/design.md §2.6): `<scene>.lightmap.ogt`, the text the cook
//! and the runtime read, and `<scene>.lightmap.ktx2`, the lightmap pages
//! (RGBA16F array layers, zlib-supercompressed, linear irradiance over π).
//!
//! ```text
//! type lightmap
//! version 1
//! ---
//! scene "scenes/cornell.ogt"
//! scene-hash "0123456789abcdef"   # the cooked scene's hash the bake read
//! pages "cornell.lightmap.ktx2"   # beside this file
//! samples 256
//! seconds 12.5
//! model "box" {                   # a static model entity, by name
//!   asset "models/box.glb"
//!   hash "…"                      # the cooked model's hash the bake read
//!   part {                        # per model part, in the model's order
//!     vertices 24                 # the part's own vertex count
//!     xref 0 1 2 …                # per lightmap vertex: the part's vertex
//!     indices 0 1 2 …
//!     uv 0.1 0.2 0 …              # per lightmap vertex: u, v, page
//!   }
//! }
//! volume "probes" {               # a LightVolume entity, by name
//!   min -2 0 -2
//!   max 2 4 2
//!   counts 5 5 5
//!   probes …                      # 12 numbers a probe, x fastest
//! }
//! ```
//!
//! The cook packs the text and the pages into one `og.Lightmap` blob, so
//! the runtime reads both from the bundle; the `.ktx2` is consumed by it
//! (never cooked as a texture of its own).

use og_core::ogt::{Block, Document, Value};

/// The `.ogt` type of a lightmap file.
pub const LIGHTMAP_TYPE: &str = "lightmap";

/// A static model's lightmap coordinates, per part.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BakedPart {
    /// The part's own vertex count (a mismatch makes the bake stale).
    pub vertices: u32,
    /// Per lightmap vertex: the part's vertex it copies.
    pub xref: Vec<u32>,
    pub indices: Vec<u32>,
    /// Per lightmap vertex: u, v (0..1, v down) and the page.
    pub uv: Vec<[f32; 3]>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BakedModel {
    /// The scene entity's name.
    pub entity: String,
    pub asset: String,
    /// The cooked model's hash when baked.
    pub hash: u64,
    pub parts: Vec<BakedPart>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BakedVolume {
    pub entity: String,
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub counts: [u32; 3],
    /// Per probe, red, green and blue as `[c1.x, c1.y, c1.z, c0]`.
    pub probes: Vec<[[f32; 4]; 3]>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LightmapFile {
    pub scene: String,
    /// The cooked scene's hash when baked.
    pub scene_hash: u64,
    /// The pages file, relative to this one.
    pub pages: String,
    pub samples: u32,
    pub seconds: f32,
    pub models: Vec<BakedModel>,
    pub volumes: Vec<BakedVolume>,
}

/// Numbers per line in the text (keys repeat; the parser concatenates).
const PER_LINE: usize = 48;

fn numbers<T: std::fmt::Display>(out: &mut String, indent: &str, key: &str, values: &[T]) {
    if values.is_empty() {
        out.push_str(&format!("{indent}{key}\n"));
    }
    for chunk in values.chunks(PER_LINE) {
        out.push_str(indent);
        out.push_str(key);
        for v in chunk {
            out.push(' ');
            out.push_str(&v.to_string());
        }
        out.push('\n');
    }
}

fn hex(v: u64) -> String {
    format!("{v:016x}")
}

fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

impl LightmapFile {
    pub fn to_text(&self) -> String {
        let mut o = String::from("type lightmap\nversion 1\n---\n");
        o.push_str("# Written by og bake lightmaps: the scene's baked lighting. Bake again\n# after changing the scene or its static models.\n");
        o.push_str(&format!("scene {}\n", quote(&self.scene)));
        o.push_str(&format!("scene-hash {}\n", quote(&hex(self.scene_hash))));
        o.push_str(&format!("pages {}\n", quote(&self.pages)));
        o.push_str(&format!("samples {}\n", self.samples));
        o.push_str(&format!("seconds {}\n", self.seconds));
        for m in &self.models {
            o.push_str(&format!("model {} {{\n", quote(&m.entity)));
            o.push_str(&format!("  asset {}\n", quote(&m.asset)));
            o.push_str(&format!("  hash {}\n", quote(&hex(m.hash))));
            for p in &m.parts {
                o.push_str("  part {\n");
                o.push_str(&format!("    vertices {}\n", p.vertices));
                numbers(&mut o, "    ", "xref", &p.xref);
                numbers(&mut o, "    ", "indices", &p.indices);
                let uv: Vec<f32> = p.uv.iter().flatten().copied().collect();
                numbers(&mut o, "    ", "uv", &uv);
                o.push_str("  }\n");
            }
            o.push_str("}\n");
        }
        for v in &self.volumes {
            o.push_str(&format!("volume {} {{\n", quote(&v.entity)));
            numbers(&mut o, "  ", "min", &v.min);
            numbers(&mut o, "  ", "max", &v.max);
            numbers(&mut o, "  ", "counts", &v.counts);
            let p: Vec<f32> = v.probes.iter().flatten().flatten().copied().collect();
            numbers(&mut o, "  ", "probes", &p);
            o.push_str("}\n");
        }
        o
    }

    pub fn parse(text: &str) -> Result<LightmapFile, String> {
        let doc = Document::parse(text).map_err(|e| e.to_string())?;
        if doc.ty != LIGHTMAP_TYPE {
            return Err(format!("type '{}' is not a lightmap", doc.ty));
        }
        let b = &doc.body;
        let s = |b: &Block, k: &str| -> Result<String, String> {
            b.get(k)
                .and_then(|e| e.values.first())
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| format!("'{k}' is missing"))
        };
        let h = |b: &Block, k: &str| -> Result<u64, String> {
            u64::from_str_radix(&s(b, k)?, 16).map_err(|_| format!("'{k}' is not a hash"))
        };
        let nums = |b: &Block, k: &str| -> Result<Vec<f64>, String> {
            let mut out = Vec::new();
            for e in b.get_all(k) {
                for v in &e.values {
                    match v {
                        Value::Number(n) => out.push(n.as_f64()),
                        _ => return Err(format!("'{k}': numbers only")),
                    }
                }
            }
            Ok(out)
        };
        let block = |e: &og_core::ogt::Entry| -> Result<Block, String> {
            e.block
                .clone()
                .ok_or_else(|| "a block is missing".to_string())
        };
        let mut file = LightmapFile {
            scene: s(b, "scene")?,
            scene_hash: h(b, "scene-hash")?,
            pages: s(b, "pages")?,
            samples: nums(b, "samples")?.first().copied().unwrap_or(0.0) as u32,
            seconds: nums(b, "seconds")?.first().copied().unwrap_or(0.0) as f32,
            ..Default::default()
        };
        for e in b.get_all("model") {
            let mb = block(e)?;
            let mut m = BakedModel {
                entity: e
                    .values
                    .first()
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                asset: s(&mb, "asset")?,
                hash: h(&mb, "hash")?,
                parts: Vec::new(),
            };
            for pe in mb.get_all("part") {
                let pb = block(pe)?;
                let uv = nums(&pb, "uv")?;
                let p = BakedPart {
                    vertices: nums(&pb, "vertices")?.first().copied().unwrap_or(0.0) as u32,
                    xref: nums(&pb, "xref")?.into_iter().map(|v| v as u32).collect(),
                    indices: nums(&pb, "indices")?
                        .into_iter()
                        .map(|v| v as u32)
                        .collect(),
                    uv: uv
                        .as_chunks::<3>()
                        .0
                        .iter()
                        .map(|c| [c[0] as f32, c[1] as f32, c[2] as f32])
                        .collect(),
                };
                if p.uv.len() != p.xref.len() {
                    return Err(format!(
                        "model '{}': {} uvs for {} vertices",
                        m.entity,
                        p.uv.len(),
                        p.xref.len()
                    ));
                }
                m.parts.push(p);
            }
            file.models.push(m);
        }
        for e in b.get_all("volume") {
            let vb = block(e)?;
            let v3 = |k: &str| -> Result<[f32; 3], String> {
                match nums(&vb, k)?.as_slice() {
                    [x, y, z] => Ok([*x as f32, *y as f32, *z as f32]),
                    _ => Err(format!("'{k}' takes three numbers")),
                }
            };
            let c = v3("counts")?.map(|c| c as u32);
            let p = nums(&vb, "probes")?;
            let probes: Vec<[[f32; 4]; 3]> = p
                .as_chunks::<12>()
                .0
                .iter()
                .map(|q| {
                    let f = |i: usize| q[i] as f32;
                    [
                        [f(0), f(1), f(2), f(3)],
                        [f(4), f(5), f(6), f(7)],
                        [f(8), f(9), f(10), f(11)],
                    ]
                })
                .collect();
            if probes.len() != (c[0] * c[1] * c[2]) as usize {
                return Err("a volume's probes do not fill its counts".into());
            }
            file.volumes.push(BakedVolume {
                entity: e
                    .values
                    .first()
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                min: v3("min")?,
                max: v3("max")?,
                counts: c,
                probes,
            });
        }
        Ok(file)
    }
}

/// The lightmap file of a scene asset (`scenes/a.ogt` → `scenes/a.lightmap.ogt`).
pub fn lightmap_id(scene: &str) -> String {
    format!("{}.lightmap.ogt", scene.trim_end_matches(".ogt"))
}

/// The pages file's name beside the lightmap file of `scene`.
pub fn pages_name(scene: &str) -> String {
    let stem = scene.trim_end_matches(".ogt");
    let base = stem.rsplit('/').next().unwrap_or(stem);
    format!("{base}.lightmap.ktx2")
}

const VK_FORMAT_R16G16B16A16_SFLOAT: u32 = 97;

/// Lightmap pages as a KTX2 file: RGBA16F (`f16` bits), `width × height`,
/// one array layer per page, zlib-supercompressed.
pub fn pages_to_ktx2(width: u32, height: u32, pages: &[Vec<u16>]) -> Vec<u8> {
    crate::ktx2::write_layers_rgba16f(width, height, pages, "og bake lightmaps")
}

/// [`pages_to_ktx2`]'s file read back: (width, height, pages of RGBA f16 bits).
pub fn pages_from_ktx2(bytes: &[u8]) -> Result<(u32, u32, Vec<Vec<u16>>), String> {
    // The header and level 0's index, read directly (ktx2::parse takes
    // single-layer textures only).
    let word = |at: usize| -> Result<u32, String> {
        bytes
            .get(at..at + 4)
            .map(|b| u32::from_le_bytes(b.try_into().expect("4 bytes")))
            .ok_or_else(|| "the pages file is cut short".to_string())
    };
    if bytes.get(..12)
        != Some(
            &[
                0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
            ][..],
        )
    {
        return Err("the pages file is not KTX2".into());
    }
    if word(12)? != VK_FORMAT_R16G16B16A16_SFLOAT {
        return Err(format!(
            "the lightmap pages are VkFormat {} (RGBA16F is 97)",
            word(12)?
        ));
    }
    let (width, height) = (word(20)?, word(24)?);
    let layers = word(32)?.max(1);
    let supercompression = word(44)?;
    // Level 0's byte offset and length are u64 in the file; a usize on
    // wasm32 is 32 bits, so they are read as u64 and then fitted.
    let u64_at = |at: usize| -> Result<usize, String> {
        let v = u64::from(word(at)?) | u64::from(word(at + 4)?) << 32;
        usize::try_from(v).map_err(|_| "the level does not fit in memory".to_string())
    };
    let offset = u64_at(80)?;
    let length = u64_at(88)?;
    let stored = bytes
        .get(offset..offset + length)
        .ok_or("the level is cut short")?;
    struct I {
        width: u32,
        height: u32,
        supercompression: u32,
    }
    let info = I {
        width,
        height,
        supercompression,
    };
    let texels = (info.width * info.height) as usize * 4;
    let raw = match info.supercompression {
        0 => stored.to_vec(),
        crate::ktx2::SS_ZLIB => miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(
            stored,
            texels * 2 * layers as usize,
        )
        .map_err(|e| format!("zlib: {e:?}"))?,
        s => return Err(format!("supercompression {s} is not read")),
    };
    if raw.len() != texels * 2 * layers as usize {
        return Err("the pages are cut short".into());
    }
    let halves: Vec<u16> = raw
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| u16::from_le_bytes(*b))
        .collect();
    Ok((
        info.width,
        info.height,
        halves.chunks(texels).map(<[u16]>::to_vec).collect(),
    ))
}

/// An f32 as an IEEE half (round to nearest; overflow to infinity).
pub fn f32_to_f16(x: f32) -> u16 {
    let b = x.to_bits();
    let sign = ((b >> 16) & 0x8000) as u16;
    let exp = ((b >> 23) & 0xff) as i32;
    let mant = b & 0x7f_ffff;
    if exp == 0xff {
        return sign | 0x7c00 | if mant != 0 { 0x200 } else { 0 };
    }
    let e = exp - 127 + 15;
    if e >= 0x1f {
        return sign | 0x7c00;
    }
    if e <= 0 {
        if e < -10 {
            return sign;
        }
        let m = (mant | 0x80_0000) >> (1 - e);
        return sign | ((m >> 13) + ((m >> 12) & 1)) as u16;
    }
    let half = sign as u32 | ((e as u32) << 10) | (mant >> 13);
    (half + ((mant >> 12) & 1)) as u16
}

/// An IEEE half as an f32.
pub fn f16_to_f32(h: u16) -> f32 {
    let sign = if h & 0x8000 != 0 { -1.0 } else { 1.0 };
    let e = ((h >> 10) & 31) as i32;
    let m = (h & 1023) as f32;
    sign * match e {
        0 => m * (2.0f32).powi(-24),
        31 => f32::INFINITY,
        _ => (1.0 + m / 1024.0) * (2.0f32).powi(e - 15),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_round_trips() {
        let f = LightmapFile {
            scene: "scenes/cornell.ogt".into(),
            scene_hash: 0x0123_4567_89ab_cdef,
            pages: "cornell.lightmap.ktx2".into(),
            samples: 64,
            seconds: 1.5,
            models: vec![BakedModel {
                entity: "box".into(),
                asset: "models/box.glb".into(),
                hash: 7,
                parts: vec![BakedPart {
                    vertices: 3,
                    xref: (0..100).map(|i| i % 3).collect(),
                    indices: vec![0, 1, 2],
                    uv: (0..100).map(|i| [i as f32 / 100.0, 0.25, 1.0]).collect(),
                }],
            }],
            volumes: vec![BakedVolume {
                entity: "probes".into(),
                min: [-1.0, 0.0, -1.0],
                max: [1.0, 2.0, 1.0],
                counts: [2, 2, 2],
                probes: (0..8).map(|i| [[i as f32, 0.5, -0.25, 1.0]; 3]).collect(),
            }],
        };
        assert_eq!(LightmapFile::parse(&f.to_text()).unwrap(), f);
    }

    #[test]
    fn pages_round_trip() {
        let pages = vec![vec![0x3c00u16; 4 * 4 * 4], vec![0x4000u16; 4 * 4 * 4]];
        let bytes = pages_to_ktx2(4, 4, &pages);
        assert_eq!(pages_from_ktx2(&bytes).unwrap(), (4, 4, pages));
        assert_eq!(f16_to_f32(0x3c00), 1.0);
        assert_eq!(f16_to_f32(0xc000), -2.0);
        for x in [0.0f32, 1.0, -2.5, 0.333, 1000.0, 1e-5] {
            assert!(
                (f16_to_f32(f32_to_f16(x)) - x).abs() <= x.abs() * 1e-3 + 1e-7,
                "{x}"
            );
        }
    }

    #[test]
    fn names() {
        assert_eq!(
            lightmap_id("scenes/cornell.ogt"),
            "scenes/cornell.lightmap.ogt"
        );
        assert_eq!(pages_name("scenes/cornell.ogt"), "cornell.lightmap.ktx2");
    }
}
