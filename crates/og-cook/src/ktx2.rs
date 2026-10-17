//! KTX2 (Khronos Texture 2.0) as the cook writes and the runtime reads it.
//!
//! Three flavours, all single-layer 2D textures with a full mip chain:
//!
//! - **ETC1S / BasisLZ** (`vkFormat` 0, supercompression 1): colour.
//! - **UASTC / Zstandard** (`vkFormat` 0, supercompression 2): normals and
//!   linear data.
//! - **RGBA8 / zlib** (`VK_FORMAT_R8G8B8A8_UNORM` or `_SRGB`, supercompression
//!   3): lossless (MSDF atlases), decodable in pure Rust on every target.
//!
//! The basis-universal crate (basisu 1.16) only emits `.basis` files, so
//! [`from_basis`] rebuilds the KTX2 container from the `.basis` bits exactly
//! as basisu's own `create_ktx2_file` does (same DFD, same global data
//! layout); [`to_basis`] is the inverse the native transcoder needs.

/// The 12-byte KTX2 identifier.
pub const IDENTIFIER: [u8; 12] = [
    0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
];
pub const SS_NONE: u32 = 0;
pub const SS_BASISLZ: u32 = 1;
pub const SS_ZSTD: u32 = 2;
pub const SS_ZLIB: u32 = 3;
pub const VK_FORMAT_R8G8B8A8_UNORM: u32 = 37;
pub const VK_FORMAT_R8G8B8A8_SRGB: u32 = 43;
const HEADER_SIZE: usize = 80;

/// One mip level's byte range in the file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Level {
    pub offset: usize,
    pub length: usize,
    pub uncompressed_length: usize,
}

/// The parts of a KTX2 file the engine uses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Info {
    pub vk_format: u32,
    pub width: u32,
    pub height: u32,
    pub supercompression: u32,
    /// Level 0 first.
    pub levels: Vec<Level>,
    /// KHR_DF transfer function: 1 linear, 2 sRGB.
    pub transfer: u32,
    /// Basis colour model of the DFD: 163 ETC1S, 166 UASTC (0 otherwise).
    pub color_model: u32,
    /// Whether the DFD declares an alpha channel.
    pub has_alpha: bool,
    pub sgd: (usize, usize),
}

fn u32_at(b: &[u8], at: usize) -> Result<u32, String> {
    b.get(at..at + 4)
        .map(|s| u32::from_le_bytes(s.try_into().expect("4 bytes")))
        .ok_or_else(|| format!("KTX2 truncated at byte {at}"))
}
fn u64_at(b: &[u8], at: usize) -> Result<usize, String> {
    b.get(at..at + 8)
        .map(|s| u64::from_le_bytes(s.try_into().expect("8 bytes")) as usize)
        .ok_or_else(|| format!("KTX2 truncated at byte {at}"))
}

/// Validates the header, level index and DFD of a KTX2 file.
pub fn parse(bytes: &[u8]) -> Result<Info, String> {
    if bytes.len() < HEADER_SIZE || bytes[..12] != IDENTIFIER {
        return Err("not a KTX2 file".into());
    }
    let vk_format = u32_at(bytes, 12)?;
    let width = u32_at(bytes, 20)?;
    let height = u32_at(bytes, 24)?;
    let (depth, layers, faces) = (u32_at(bytes, 28)?, u32_at(bytes, 32)?, u32_at(bytes, 36)?);
    let level_count = u32_at(bytes, 40)?.max(1) as usize;
    let supercompression = u32_at(bytes, 44)?;
    let (dfd_off, dfd_len) = (u32_at(bytes, 48)? as usize, u32_at(bytes, 52)? as usize);
    let sgd = (u64_at(bytes, 64)?, u64_at(bytes, 72)?);
    if width == 0 || height == 0 || depth > 1 || layers > 1 || faces != 1 {
        return Err("only single-layer 2D KTX2 textures are supported".into());
    }
    if level_count > 32 {
        return Err("too many mip levels".into());
    }
    let mut levels = Vec::with_capacity(level_count);
    for i in 0..level_count {
        let at = HEADER_SIZE + i * 24;
        let level = Level {
            offset: u64_at(bytes, at)?,
            length: u64_at(bytes, at + 8)?,
            uncompressed_length: u64_at(bytes, at + 16)?,
        };
        if level
            .offset
            .checked_add(level.length)
            .is_none_or(|e| e > bytes.len())
        {
            return Err(format!("KTX2 level {i} runs past the end of the file"));
        }
        levels.push(level);
    }
    if sgd.0.checked_add(sgd.1).is_none_or(|e| e > bytes.len()) {
        return Err("KTX2 supercompression global data runs past the end".into());
    }
    let dfd = bytes
        .get(dfd_off..dfd_off + dfd_len)
        .filter(|d| d.len() >= 44)
        .ok_or("KTX2 data format descriptor is missing or short")?;
    let word = |i: usize| u32::from_le_bytes(dfd[i * 4..i * 4 + 4].try_into().expect("4 bytes"));
    let model_bits = word(3);
    let color_model = model_bits & 0xff;
    let transfer = (model_bits >> 16) & 0xff;
    let samples = (dfd.len() - 28) / 16;
    let channel = |s: usize| (word(7 + s * 4) >> 24) & 0x0f;
    let has_alpha = match color_model {
        163 => samples > 1,
        166 => channel(0) == 3,
        _ => samples >= 4,
    };
    Ok(Info {
        vk_format,
        width,
        height,
        supercompression,
        levels,
        transfer,
        color_model,
        has_alpha,
        sgd,
    })
}

fn kvd(writer: &str) -> Vec<u8> {
    let key = b"KTXwriter\0";
    let mut value = writer.as_bytes().to_vec();
    value.push(0);
    let mut out = Vec::new();
    out.extend_from_slice(&((key.len() + value.len()) as u32).to_le_bytes());
    out.extend_from_slice(key);
    out.extend_from_slice(&value);
    while out.len() % 4 != 0 {
        out.push(0);
    }
    out
}

struct Parts<'a> {
    vk_format: u32,
    type_size: u32,
    width: u32,
    height: u32,
    supercompression: u32,
    dfd: Vec<u8>,
    sgd: Vec<u8>,
    /// (stored bytes, uncompressed length), level 0 first.
    levels: Vec<(Vec<u8>, usize)>,
    level_align: usize,
    writer: &'a str,
}

fn assemble(p: Parts) -> Vec<u8> {
    let n = p.levels.len();
    let mut out = vec![0u8; HEADER_SIZE + 24 * n];
    let dfd_off = out.len();
    out.extend_from_slice(&p.dfd);
    let kvd = kvd(p.writer);
    let kvd_off = out.len();
    out.extend_from_slice(&kvd);
    let mut sgd_off = 0;
    if !p.sgd.is_empty() {
        while !out.len().is_multiple_of(8) {
            out.push(0);
        }
        sgd_off = out.len();
        out.extend_from_slice(&p.sgd);
    }
    let mut index = vec![(0usize, 0usize, 0usize); n];
    // Smallest level first, as the specification recommends for streaming.
    for level in (0..n).rev() {
        while !out.len().is_multiple_of(p.level_align) {
            out.push(0);
        }
        index[level] = (out.len(), p.levels[level].0.len(), p.levels[level].1);
        out.extend_from_slice(&p.levels[level].0);
    }
    let mut h = Vec::with_capacity(HEADER_SIZE);
    h.extend_from_slice(&IDENTIFIER);
    for v in [
        p.vk_format,
        p.type_size,
        p.width,
        p.height,
        0,
        0,
        1,
        n as u32,
        p.supercompression,
    ] {
        h.extend_from_slice(&v.to_le_bytes());
    }
    for v in [
        dfd_off as u32,
        p.dfd.len() as u32,
        kvd_off as u32,
        kvd.len() as u32,
    ] {
        h.extend_from_slice(&v.to_le_bytes());
    }
    h.extend_from_slice(&(sgd_off as u64).to_le_bytes());
    h.extend_from_slice(&(p.sgd.len() as u64).to_le_bytes());
    out[..HEADER_SIZE].copy_from_slice(&h);
    for (i, (off, len, raw)) in index.iter().enumerate() {
        let at = HEADER_SIZE + i * 24;
        out[at..at + 8].copy_from_slice(&(*off as u64).to_le_bytes());
        out[at + 8..at + 16].copy_from_slice(&(*len as u64).to_le_bytes());
        out[at + 16..at + 24].copy_from_slice(&(*raw as u64).to_le_bytes());
    }
    out
}

fn mip_count(width: u32, height: u32) -> usize {
    32 - width.max(height).leading_zeros() as usize
}

fn box_down(rgba: &[u8], w: u32, h: u32) -> (Vec<u8>, u32, u32) {
    let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
    let mut out = vec![0u8; (nw * nh * 4) as usize];
    for y in 0..nh {
        for x in 0..nw {
            for c in 0..4 {
                let mut sum = 0u32;
                let mut n = 0u32;
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    let (sx, sy) = ((x * 2 + dx).min(w - 1), (y * 2 + dy).min(h - 1));
                    sum += rgba[((sy * w + sx) * 4 + c) as usize] as u32;
                    n += 1;
                }
                out[((y * nw + x) * 4 + c) as usize] = ((sum + n / 2) / n) as u8;
            }
        }
    }
    (out, nw, nh)
}

fn rgba8_dfd(srgb: bool) -> Vec<u8> {
    // Basic DFD block for R8G8B8A8 (KHR_DF_MODEL_RGBSDA, four 8-bit samples).
    let mut words: Vec<u32> = vec![
        0,                                                 // total size, patched below
        0,                                                 // vendor 0, descriptor type 0
        2 | (88 << 16),                                    // version 2, block size 24 + 16 * 4
        1 | (1 << 8) | ((if srgb { 2 } else { 1 }) << 16), // RGBSDA, BT709, transfer
        0,                                                 // texel block 1x1x1x1
        4,                                                 // 4 bytes per plane 0
        0,
    ];
    for (i, channel) in [0u32, 1, 2, 15].iter().enumerate() {
        // KHR_DF_SAMPLE_DATATYPE_LINEAR: alpha stays linear in an sRGB texture.
        let linear_alpha = if *channel == 15 && srgb { 0x10 } else { 0 };
        words.push(((i as u32) * 8) | (7 << 16) | ((*channel | linear_alpha) << 24));
        words.push(0);
        words.push(0);
        words.push(255);
    }
    words[0] = (words.len() * 4) as u32;
    words.iter().flat_map(|w| w.to_le_bytes()).collect()
}

/// An RGBA8 texture with a box-filtered mip chain, each level zlib-compressed
/// (deterministic: miniz level 9). Rows top to bottom.
pub fn write_rgba8_zlib(
    width: u32,
    height: u32,
    rgba: &[u8],
    srgb: bool,
    mips: bool,
    writer: &str,
) -> Vec<u8> {
    assert_eq!(rgba.len(), (width * height * 4) as usize);
    let mut levels = Vec::new();
    let (mut level, mut w, mut h) = (rgba.to_vec(), width, height);
    let count = if mips { mip_count(width, height) } else { 1 };
    for i in 0..count {
        levels.push((
            miniz_oxide::deflate::compress_to_vec_zlib(&level, 9),
            level.len(),
        ));
        if i + 1 < count {
            (level, w, h) = box_down(&level, w, h);
        }
    }
    assemble(Parts {
        vk_format: if srgb {
            VK_FORMAT_R8G8B8A8_SRGB
        } else {
            VK_FORMAT_R8G8B8A8_UNORM
        },
        type_size: 1,
        width,
        height,
        supercompression: SS_ZLIB,
        dfd: rgba8_dfd(srgb),
        sgd: Vec::new(),
        levels,
        level_align: 1,
        writer,
    })
}

/// Level `level` of an RGBA8 KTX2 file (supercompression none or zlib).
pub fn decode_rgba8(bytes: &[u8], level: usize) -> Result<(u32, u32, Vec<u8>), String> {
    let info = parse(bytes)?;
    if info.vk_format != VK_FORMAT_R8G8B8A8_UNORM && info.vk_format != VK_FORMAT_R8G8B8A8_SRGB {
        return Err(format!("KTX2 vkFormat {} is not RGBA8", info.vk_format));
    }
    let l = *info.levels.get(level).ok_or("no such mip level")?;
    let (w, h) = ((info.width >> level).max(1), (info.height >> level).max(1));
    let stored = &bytes[l.offset..l.offset + l.length];
    let pixels = match info.supercompression {
        SS_NONE => stored.to_vec(),
        SS_ZLIB => {
            miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(stored, (w * h * 4) as usize)
                .map_err(|e| format!("KTX2 zlib level {level}: {e:?}"))?
        }
        s => return Err(format!("KTX2 supercompression {s} is not lossless RGBA8")),
    };
    if pixels.len() != (w * h * 4) as usize {
        return Err(format!(
            "KTX2 level {level} has {} bytes, expected {}",
            pixels.len(),
            w * h * 4
        ));
    }
    Ok((w, h, pixels))
}

// ── .basis <-> KTX2 ──────────────────────────────────────────────────────

/// basisu's `crc16` (the .basis header and slice checksums).
#[cfg(feature = "transcode")]
fn crc16(bytes: &[u8]) -> u16 {
    let mut crc: u16 = !0;
    for &b in bytes {
        let q = (b as u16) ^ (crc >> 8);
        let k = (q >> 4) ^ q;
        crc = (((crc << 8) ^ k) ^ (k << 5)) ^ (k << 12);
    }
    !crc
}

#[cfg(any(feature = "basis", feature = "transcode"))]
#[allow(dead_code)]
mod basis_file {
    //! The packed `.basis` layout of basisu 1.16 (`basisu_file_headers.h`).
    pub const HEADER: usize = 77;
    pub const SLICE: usize = 23;
    pub const FLAG_ETC1S: u32 = 1;
    pub const FLAG_HAS_ALPHA_SLICES: u32 = 4;
    pub const FLAG_SRGB: u32 = 16;
    pub const SLICE_HAS_ALPHA: u32 = 1;

    pub fn get(b: &[u8], at: usize, n: usize) -> u32 {
        (0..n).fold(0u32, |v, i| v | (b[at + i] as u32) << (8 * i))
    }
    pub fn put(b: &mut [u8], at: usize, n: usize, v: u32) {
        for i in 0..n {
            b[at + i] = (v >> (8 * i)) as u8;
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct Slice {
        pub level: u32,
        pub alpha: bool,
        pub width: u32,
        pub height: u32,
        pub offset: usize,
        pub size: usize,
    }
}

#[cfg(feature = "basis")]
fn basis_dfd(uastc: bool, alpha: bool, srgb: bool, supercompressed: bool) -> Vec<u8> {
    // basisu 1.16 `g_ktx2_*_dfd` tables, fixed up as its `get_dfd` does.
    const ETC1S: [u8; 44] = [
        0x2C, 0, 0, 0, 0, 0, 0, 0, 0x2, 0, 0x28, 0, 0xA3, 0x1, 0x2, 0, 0x3, 0x3, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0x3F, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF,
    ];
    const ETC1S_ALPHA: [u8; 60] = [
        0x3C, 0, 0, 0, 0, 0, 0, 0, 0x2, 0, 0x38, 0, 0xA3, 0x1, 0x2, 0, 0x3, 0x3, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0x3F, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0x40, 0, 0x3F,
        0xF, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF,
    ];
    const UASTC: [u8; 44] = [
        0x2C, 0, 0, 0, 0, 0, 0, 0, 0x2, 0, 0x28, 0, 0xA6, 0x1, 0x2, 0, 0x3, 0x3, 0, 0, 0x10, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0x7F, 0x4, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF,
    ];
    let mut dfd = match (uastc, alpha) {
        (true, _) => UASTC.to_vec(),
        (false, false) => ETC1S.to_vec(),
        (false, true) => ETC1S_ALPHA.to_vec(),
    };
    let word =
        |d: &[u8], i: usize| u32::from_le_bytes(d[i * 4..i * 4 + 4].try_into().expect("4 bytes"));
    let set =
        |d: &mut Vec<u8>, i: usize, v: u32| d[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
    let bits = (word(&dfd, 3) & !(0xff << 16)) | ((if srgb { 2 } else { 1 }) << 16);
    set(&mut dfd, 3, bits);
    if supercompressed {
        let plane = word(&dfd, 5) & !0xff;
        set(&mut dfd, 5, plane);
    }
    if uastc {
        let chan = (word(&dfd, 7) & !(0xf << 24)) | ((if alpha { 3 } else { 0 }) << 24);
        set(&mut dfd, 7, chan);
    }
    dfd
}

/// A KTX2 file from a single-image `.basis` file (ETC1S becomes BasisLZ,
/// UASTC is optionally Zstandard-supercompressed).
#[cfg(feature = "basis")]
pub fn from_basis(basis: &[u8], srgb: bool, zstd: bool, writer: &str) -> Result<Vec<u8>, String> {
    use basis_file::*;
    if basis.len() < HEADER || get(basis, 0, 2) != (b's' as u32) | ((b'B' as u32) << 8) {
        return Err("not a .basis file".into());
    }
    let total_slices = get(basis, 14, 3) as usize;
    let tex_format = get(basis, 20, 1);
    let uastc = tex_format == 1;
    let slice_desc = get(basis, 65, 4) as usize;
    let mut slices = Vec::with_capacity(total_slices);
    for i in 0..total_slices {
        let at = slice_desc + i * SLICE;
        if at + SLICE > basis.len() {
            return Err(".basis slice table truncated".into());
        }
        if get(basis, at, 3) != 0 {
            return Err("only single-image .basis files convert to KTX2 here".into());
        }
        let s = Slice {
            level: get(basis, at + 3, 1),
            alpha: get(basis, at + 4, 1) & SLICE_HAS_ALPHA != 0,
            width: get(basis, at + 5, 2),
            height: get(basis, at + 7, 2),
            offset: get(basis, at + 13, 4) as usize,
            size: get(basis, at + 17, 4) as usize,
        };
        if s.offset + s.size > basis.len() {
            return Err(".basis slice data truncated".into());
        }
        slices.push(s);
    }
    let level_count = slices
        .iter()
        .map(|s| s.level + 1)
        .max()
        .ok_or(".basis has no slices")? as usize;
    let base = slices
        .iter()
        .find(|s| s.level == 0)
        .ok_or(".basis has no level 0")?;
    let (width, height) = (base.width, base.height);
    let alpha = slices.iter().any(|s| s.alpha);
    let mut level_data: Vec<Vec<u8>> = vec![Vec::new(); level_count];
    // (rgb offset, rgb len, alpha offset, alpha len) per level, inside the level data.
    let mut image_descs = vec![[0u32; 4]; level_count];
    for s in &slices {
        let l = s.level as usize;
        let offset = level_data[l].len() as u32;
        if s.alpha {
            image_descs[l][2] = offset;
            image_descs[l][3] = s.size as u32;
        } else {
            image_descs[l][0] = offset;
            image_descs[l][1] = s.size as u32;
        }
        level_data[l].extend_from_slice(&basis[s.offset..s.offset + s.size]);
    }
    let (supercompression, sgd, levels, align) = if uastc {
        if zstd {
            let levels = level_data
                .into_iter()
                .map(|d| {
                    let n = d.len();
                    (
                        ruzstd::encoding::compress_to_vec(
                            &d[..],
                            ruzstd::encoding::CompressionLevel::Fastest,
                        ),
                        n,
                    )
                })
                .collect();
            (SS_ZSTD, Vec::new(), levels, 1)
        } else {
            let levels = level_data
                .into_iter()
                .map(|d| {
                    let n = d.len();
                    (d, n)
                })
                .collect();
            (SS_NONE, Vec::new(), levels, 16)
        }
    } else {
        let range = |ofs: usize, size: usize| -> Result<&[u8], String> {
            basis
                .get(ofs..ofs + size)
                .ok_or_else(|| ".basis codebook out of range".to_string())
        };
        let endpoints = range(get(basis, 41, 4) as usize, get(basis, 45, 3) as usize)?;
        let selectors = range(get(basis, 50, 4) as usize, get(basis, 54, 3) as usize)?;
        let tables = range(get(basis, 57, 4) as usize, get(basis, 61, 4) as usize)?;
        let mut sgd = Vec::new();
        sgd.extend_from_slice(&(get(basis, 39, 2) as u16).to_le_bytes());
        sgd.extend_from_slice(&(get(basis, 48, 2) as u16).to_le_bytes());
        for len in [endpoints.len(), selectors.len(), tables.len(), 0] {
            sgd.extend_from_slice(&(len as u32).to_le_bytes());
        }
        for d in &image_descs {
            sgd.extend_from_slice(&0u32.to_le_bytes());
            for v in d {
                sgd.extend_from_slice(&v.to_le_bytes());
            }
        }
        sgd.extend_from_slice(endpoints);
        sgd.extend_from_slice(selectors);
        sgd.extend_from_slice(tables);
        let levels = level_data.into_iter().map(|d| (d, 0)).collect();
        (SS_BASISLZ, sgd, levels, 1)
    };
    Ok(assemble(Parts {
        vk_format: 0,
        type_size: 1,
        width,
        height,
        supercompression,
        dfd: basis_dfd(uastc, alpha, srgb, supercompression != SS_NONE),
        sgd,
        levels,
        level_align: align,
        writer,
    }))
}

/// The `.basis` file a Basis Universal KTX2 file was built from (the inverse
/// of [`from_basis`]), for basisu's `.basis` transcoder.
#[cfg(feature = "transcode")]
pub fn to_basis(ktx2: &[u8]) -> Result<Vec<u8>, String> {
    use basis_file::*;
    let info = parse(ktx2)?;
    let uastc = match info.color_model {
        163 => false,
        166 => true,
        m => return Err(format!("KTX2 colour model {m} is not Basis Universal")),
    };
    let mut slices: Vec<(Slice, Vec<u8>)> = Vec::new();
    let (mut endpoints, mut selectors, mut tables) = (&[][..], &[][..], &[][..]);
    let (mut endpoint_count, mut selector_count) = (0u32, 0u32);
    let level_dims = |l: usize| ((info.width >> l).max(1), (info.height >> l).max(1));
    let level_bytes = |l: usize| -> Result<Vec<u8>, String> {
        let lv = info.levels[l];
        let stored = &ktx2[lv.offset..lv.offset + lv.length];
        match info.supercompression {
            SS_ZSTD => {
                let mut out = Vec::with_capacity(lv.uncompressed_length);
                let mut decoder = ruzstd::decoding::StreamingDecoder::new(stored)
                    .map_err(|e| format!("KTX2 zstd level {l}: {e}"))?;
                std::io::Read::read_to_end(&mut decoder, &mut out)
                    .map_err(|e| format!("KTX2 zstd level {l}: {e}"))?;
                Ok(out)
            }
            SS_NONE | SS_BASISLZ => Ok(stored.to_vec()),
            s => Err(format!("KTX2 supercompression {s} is not Basis Universal")),
        }
    };
    if uastc {
        for l in 0..info.levels.len() {
            let (w, h) = level_dims(l);
            let data = level_bytes(l)?;
            slices.push((
                Slice {
                    level: l as u32,
                    alpha: false,
                    width: w,
                    height: h,
                    offset: 0,
                    size: data.len(),
                },
                data,
            ));
        }
    } else {
        let sgd = &ktx2[info.sgd.0..info.sgd.0 + info.sgd.1];
        let rd = |at: usize, n: usize| -> Result<u32, String> {
            sgd.get(at..at + n)
                .map(|_| get(sgd, at, n))
                .ok_or_else(|| "KTX2 BasisLZ global data truncated".to_string())
        };
        endpoint_count = rd(0, 2)?;
        selector_count = rd(2, 2)?;
        let (el, sl, tl) = (rd(4, 4)? as usize, rd(8, 4)? as usize, rd(12, 4)? as usize);
        let descs = 20;
        let n = info.levels.len();
        let cb = descs + n * 20;
        endpoints = sgd.get(cb..cb + el).ok_or("BasisLZ endpoints truncated")?;
        selectors = sgd
            .get(cb + el..cb + el + sl)
            .ok_or("BasisLZ selectors truncated")?;
        tables = sgd
            .get(cb + el + sl..cb + el + sl + tl)
            .ok_or("BasisLZ tables truncated")?;
        for l in 0..n {
            let (w, h) = level_dims(l);
            let data = level_bytes(l)?;
            let d = descs + l * 20;
            for (alpha, off, len) in [
                (false, rd(d + 4, 4)?, rd(d + 8, 4)?),
                (true, rd(d + 12, 4)?, rd(d + 16, 4)?),
            ] {
                if len == 0 {
                    continue;
                }
                let bytes = data
                    .get(off as usize..(off + len) as usize)
                    .ok_or("BasisLZ slice out of range")?
                    .to_vec();
                slices.push((
                    Slice {
                        level: l as u32,
                        alpha,
                        width: w,
                        height: h,
                        offset: 0,
                        size: bytes.len(),
                    },
                    bytes,
                ));
            }
        }
    }
    let slice_desc = HEADER;
    let endpoint_ofs = slice_desc + slices.len() * SLICE;
    let selector_ofs = endpoint_ofs + endpoints.len();
    let tables_ofs = selector_ofs + selectors.len();
    let mut data_ofs = tables_ofs + tables.len();
    let mut out = vec![0u8; data_ofs];
    let has_alpha = slices.iter().any(|(s, _)| s.alpha);
    for (i, (s, bytes)) in slices.iter().enumerate() {
        let at = slice_desc + i * SLICE;
        put(&mut out, at, 3, 0);
        put(&mut out, at + 3, 1, s.level);
        put(
            &mut out,
            at + 4,
            1,
            if s.alpha { SLICE_HAS_ALPHA } else { 0 },
        );
        put(&mut out, at + 5, 2, s.width);
        put(&mut out, at + 7, 2, s.height);
        put(&mut out, at + 9, 2, s.width.div_ceil(4));
        put(&mut out, at + 11, 2, s.height.div_ceil(4));
        put(&mut out, at + 13, 4, data_ofs as u32);
        put(&mut out, at + 17, 4, bytes.len() as u32);
        put(&mut out, at + 21, 2, crc16(bytes) as u32);
        data_ofs += bytes.len();
    }
    out[endpoint_ofs..selector_ofs].copy_from_slice(endpoints);
    out[selector_ofs..tables_ofs].copy_from_slice(selectors);
    out[tables_ofs..tables_ofs + tables.len()].copy_from_slice(tables);
    for (_, bytes) in &slices {
        out.extend_from_slice(bytes);
    }
    let mut flags = 0;
    if !uastc {
        flags |= FLAG_ETC1S;
    }
    if has_alpha {
        flags |= FLAG_HAS_ALPHA_SLICES;
    }
    if info.transfer == 2 {
        flags |= FLAG_SRGB;
    }
    put(&mut out, 0, 2, (b's' as u32) | ((b'B' as u32) << 8));
    put(&mut out, 2, 2, 0x13);
    put(&mut out, 4, 2, HEADER as u32);
    let data_size = (out.len() - HEADER) as u32;
    put(&mut out, 8, 4, data_size);
    put(&mut out, 14, 3, slices.len() as u32);
    put(&mut out, 17, 3, 1);
    put(&mut out, 20, 1, uastc as u32);
    put(&mut out, 21, 2, flags);
    put(&mut out, 39, 2, endpoint_count);
    if !endpoints.is_empty() {
        put(&mut out, 41, 4, endpoint_ofs as u32);
        put(&mut out, 45, 3, endpoints.len() as u32);
    }
    put(&mut out, 48, 2, selector_count);
    if !selectors.is_empty() {
        put(&mut out, 50, 4, selector_ofs as u32);
        put(&mut out, 54, 3, selectors.len() as u32);
    }
    if !tables.is_empty() {
        put(&mut out, 57, 4, tables_ofs as u32);
        put(&mut out, 61, 4, tables.len() as u32);
    }
    put(&mut out, 65, 4, slice_desc as u32);
    let data_crc = crc16(&out[HEADER..]);
    put(&mut out, 12, 2, data_crc as u32);
    let header_crc = crc16(&out[8..HEADER]);
    put(&mut out, 6, 2, header_crc as u32);
    Ok(out)
}
