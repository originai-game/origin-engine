//! Native baked-lighting records and RGBA16F KTX2 pages. Author declarations
//! are checked TypeScript; neither the cook nor runtime accepts author text.
//! Scene source and model geometry hashes stale a bake only when its inputs move.

pub(crate) mod cooked;

/// Geometry identity independent of cooker revision, BVH and texture options.
/// Tangents are omitted because the renderer generates them.
pub fn geometry_hash<'a>(parts: impl IntoIterator<Item = &'a og_render_data::MeshData>) -> u64 {
    let mut h = og_core::hash::StableHasher::new();
    for data in parts {
        h.write_u64(data.vertices.len() as u64);
        for v in &data.vertices {
            for x in v
                .position
                .iter()
                .chain(&v.normal)
                .chain(&v.uv)
                .chain(&v.color)
            {
                h.write(&x.to_bits().to_le_bytes());
            }
        }
        h.write_u64(data.indices.len() as u64);
        for i in &data.indices {
            h.write(&i.to_le_bytes());
        }
    }
    h.finish()
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BakedPart {
    /// Original part vertex count, used to detect stale bakes.
    pub vertices: u32,
    /// Original vertex copied by each lightmap vertex.
    pub xref: Vec<u32>,
    pub indices: Vec<u32>,
    /// u, v (0..1, v down), page per lightmap vertex.
    pub uv: Vec<[f32; 3]>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BakedModel {
    /// Scene entity name; empty only for HLOD proxies, which bind by asset.
    pub entity: String,
    pub asset: String,
    pub hash: u64,
    pub parts: Vec<BakedPart>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BakedVolume {
    pub entity: String,
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub fade: f32,
    pub counts: [u32; 3],
    /// Per probe, RGB coefficients [c1.x, c1.y, c1.z, c0], x fastest.
    pub probes: Vec<[[f32; 4]; 3]>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LightmapFile {
    pub scene: String,
    pub scene_hash: u64,
    /// Canonical path under the declaration's owning module.
    pub pages: String,
    pub samples: u32,
    pub seconds: f32,
    pub models: Vec<BakedModel>,
    pub volumes: Vec<BakedVolume>,
}

const VK_FORMAT_R16G16B16A16_SFLOAT: u32 = 97;

/// Linear irradiance over π as zlib-supercompressed RGBA16F array layers.
pub fn pages_to_ktx2(width: u32, height: u32, pages: &[Vec<u16>]) -> Vec<u8> {
    crate::ktx2::write_layers_rgba16f(width, height, pages, "og bake lightmaps")
}

/// Read [`pages_to_ktx2`]: (width, height, RGBA half-float bits per layer).
pub fn pages_from_ktx2(bytes: &[u8]) -> Result<(u32, u32, Vec<Vec<u16>>), String> {
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
    let u64_at = |at: usize| -> Result<usize, String> {
        let v = u64::from(word(at)?) | u64::from(word(at + 4)?) << 32;
        usize::try_from(v).map_err(|_| "the level does not fit in memory".to_string())
    };
    let offset = u64_at(80)?;
    let length = u64_at(88)?;
    let stored = offset
        .checked_add(length)
        .and_then(|end| bytes.get(offset..end))
        .ok_or("the level is cut short")?;
    let texels = (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(4))
        .filter(|n| *n > 0)
        .ok_or("invalid lightmap dimensions")?;
    let size = texels
        .checked_mul(2)
        .and_then(|n| n.checked_mul(layers as usize))
        .ok_or("lightmap pages overflow")?;
    let raw = match supercompression {
        0 => stored.to_vec(),
        crate::ktx2::SS_ZLIB => {
            miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(stored, size)
                .map_err(|e| format!("zlib: {e:?}"))?
        }
        s => return Err(format!("supercompression {s} is not read")),
    };
    if raw.len() != size {
        return Err("the pages are cut short".into());
    }
    let halves: Vec<u16> = raw
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| u16::from_le_bytes(*b))
        .collect();
    Ok((
        width,
        height,
        halves.chunks(texels).map(<[u16]>::to_vec).collect(),
    ))
}

pub use og_core::half::{f16_to_f32, f32_to_f16};
