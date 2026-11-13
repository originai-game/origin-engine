//! Images → KTX2 Basis Universal texture blobs: ETC1S (BasisLZ) for colour,
//! UASTC (+ Zstandard) for normals and linear data, with a full mip chain.
//!
//! The encoder is the basis-universal crate (basisu 1.16) when og-cook is
//! built with the `basis` feature; otherwise KTX-Software's `toktx` found on
//! PATH. With neither, texture cooking fails with a message naming both:
//! a texture is never passed through uncompressed.

use super::{CookError, TextureOptions};
use crate::format::{self, *};
use crate::ktx2;
use og_blob::BlobWriter;

/// A decoded source image.
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Decodes a source image to RGBA8, by its bytes: PNG, JPEG, WebP, KTX2
/// (Basis or RGBA8, `KTXorientation` and `KTXswizzle` honoured), DDS (BC1
/// to BC7 and the uncompressed formats; level 0, layer 0), PSD (the
/// flattened composite the file stores, 8 or 16 bits, RGB, greyscale or
/// CMYK) and TGA, the one without a signature (tried when nothing else
/// matches).
pub fn decode(bytes: &[u8]) -> Result<Image, String> {
    if std::str::from_utf8(bytes).is_ok_and(|text| text.contains("<svg")) {
        let tree = resvg::usvg::Tree::from_data(bytes, &resvg::usvg::Options::default())
            .map_err(|e| format!("SVG: {e}"))?;
        let size = tree.size().to_int_size();
        let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
            .ok_or("SVG: image dimensions exceed the rasterizer's limits")?;
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::identity(),
            &mut pixmap.as_mut(),
        );
        // tiny-skia is premultiplied; texture encoding takes straight RGBA.
        let rgba = pixmap
            .pixels()
            .iter()
            .flat_map(|pixel| {
                let p = pixel.demultiply();
                [p.red(), p.green(), p.blue(), p.alpha()]
            })
            .collect();
        return Ok(Image {
            width: size.width(),
            height: size.height(),
            rgba,
        });
    }
    if bytes.starts_with(&ktx2::IDENTIFIER) {
        return super::ktx2_source::decode_image(bytes);
    }
    if bytes.starts_with(b"DDS ") {
        let dds = image_dds::ddsfile::Dds::read(bytes).map_err(|e| format!("DDS: {e}"))?;
        let rgba = image_dds::image_from_dds(&dds, 0).map_err(|e| format!("DDS: {e}"))?;
        return Ok(Image {
            width: rgba.width(),
            height: rgba.height(),
            rgba: rgba.into_raw(),
        });
    }
    if bytes.starts_with(b"8BPS") {
        return super::psd::composite(bytes);
    }
    let img = match image::guess_format(bytes) {
        Ok(_) => image::load_from_memory(bytes),
        Err(_) => image::load_from_memory_with_format(bytes, image::ImageFormat::Tga),
    }
    .map_err(|e| format!("image: {e}"))?;
    let rgba = img.to_rgba8();
    Ok(Image {
        width: rgba.width(),
        height: rgba.height(),
        rgba: rgba.into_raw(),
    })
}

/// A source image's longest side, read from its header alone (a KTX2's,
/// or what the `image` crate reads without decoding); `None` when that is
/// not cheap (DDS, PSD, TGA) or the bytes are not an image.
pub fn longest_side(bytes: &[u8]) -> Option<u32> {
    if bytes.starts_with(&ktx2::IDENTIFIER) {
        return ktx2::parse(bytes).ok().map(|i| i.width.max(i.height));
    }
    let (w, h) = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?
        .into_dimensions()
        .ok()?;
    Some(w.max(h))
}

/// A streamed texture's mip tail (G3-3), from the blob of its whole chain
/// (what `<id>#mips` holds): the chain's levels from the first whose
/// longest side is at most [`MIP_TAIL`](crate::runtime::MIP_TAIL), byte for
/// byte, and the whole chain's size in the raw section `mips`. A chain
/// with no such level (no mips) keeps its last. The report's line too.
pub fn tail(whole: &[u8], stamp: u64) -> Result<(Vec<u8>, String), CookError> {
    let bad = |e: String| CookError(format!("the whole chain's blob: {e}"));
    let blob = og_blob::Blob::parse(whole, "texture").map_err(|e| bad(e.to_string()))?;
    let meta = blob
        .table::<TextureMeta>("meta", row_hash::<TextureMeta>())
        .map_err(|e| bad(e.to_string()))?
        .first()
        .copied()
        .ok_or_else(|| bad("no meta row".into()))?;
    let ktx = blob.raw("ktx2").map_err(|e| bad(e.to_string()))?;
    let levels = ktx2::parse(ktx).map_err(bad)?.levels.len() as u32;
    let side = |l: u32| (meta.width >> l).max(1).max((meta.height >> l).max(1));
    let from = (0..levels)
        .find(|&l| side(l) <= crate::runtime::MIP_TAIL)
        .unwrap_or(levels.saturating_sub(1));
    let tail_meta = TextureMeta {
        width: (meta.width >> from).max(1),
        height: (meta.height >> from).max(1),
        levels: levels - from,
        ..meta
    };
    let chain = crate::runtime::MipChain {
        width: meta.width,
        height: meta.height,
        levels,
    };
    let mut w = BlobWriter::new(
        format::Texture::type_hash(),
        format::Texture::schema_version(),
        stamp,
    );
    w.table("meta", &[tail_meta], row_hash::<TextureMeta>())
        .raw("ktx2", ktx2::tail(ktx, from as usize).map_err(bad)?, 16)
        .raw("mips", chain.to_bytes(), 4);
    let note = if side(from) > crate::runtime::MIP_TAIL {
        format!(
            "streamed: no level of its {levels} is at most {} texels (no mips), so its tail is the whole texture",
            crate::runtime::MIP_TAIL
        )
    } else {
        format!(
            "streamed: the tail is levels {from} to {} ({}x{}), #mips the whole chain ({}x{}, {levels} levels)",
            levels - 1,
            tail_meta.width,
            tail_meta.height,
            meta.width,
            meta.height
        )
    };
    Ok((w.finish(), note))
}

/// Which encoder cooks textures, for the stamp and the report.
pub fn encoder_name(opts: &TextureOptions) -> Result<String, CookError> {
    #[cfg(feature = "basis")]
    if !opts.force_toktx {
        return Ok("basisu 1.16 (basis-universal crate)".into());
    }
    let _ = opts;
    toktx_version().map(|v| format!("toktx {v}"))
}

fn toktx_version() -> Result<String, CookError> {
    match std::process::Command::new("toktx").arg("--version").output() {
        // toktx 4 prints its version on stderr.
        Ok(out) => Ok(format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout).trim(),
            String::from_utf8_lossy(&out.stderr).trim()
        )),
        Err(_) => Err(CookError(
            "texture cooking needs a Basis Universal encoder: build og-cook with its `basis` feature (the default), \
             or install KTX-Software's `toktx` (https://github.com/KhronosGroup/KTX-Software/releases) on PATH; \
             `og doctor` checks for both"
                .into(),
        )),
    }
}

/// A level of [`dilate_transparent`]'s pyramid: width, height, colours and
/// whether each is known.
type Level = (usize, usize, Vec<[f32; 3]>, Vec<bool>);

/// Fills the colour of every fully transparent texel from its visible
/// neighbourhood (push-pull: a pyramid of the visible texels' mean colour,
/// read back from the finest level that has any). Unseen at full size, that
/// colour is what the mips average into the visible texels: left black (a
/// canvas export premultiplies, so alpha 0 comes out black), leaf and petal
/// cards darkened with distance. Visible texels are untouched. `None` when
/// no texel is transparent, or every one is.
fn dilate_transparent(img: &Image) -> Option<Image> {
    let texels = img.rgba.as_chunks::<4>().0;
    if !texels.iter().any(|p| p[3] == 0) || texels.iter().all(|p| p[3] == 0) {
        return None;
    }
    // Push: level 0 is the image, each next one its 2x2 visible means.
    let mut levels: Vec<Level> = vec![(
        img.width as usize,
        img.height as usize,
        texels
            .iter()
            .map(|p| [p[0] as f32, p[1] as f32, p[2] as f32])
            .collect(),
        texels.iter().map(|p| p[3] > 0).collect(),
    )];
    while let Some((w, h, c, k)) = levels.last().filter(|l| l.0 > 1 || l.1 > 1) {
        let (w, h) = (*w, *h);
        let (nw, nh) = (w.div_ceil(2), h.div_ceil(2));
        let mut nc = vec![[0.0f32; 3]; nw * nh];
        let mut nk = vec![false; nw * nh];
        for y in 0..nh {
            for x in 0..nw {
                let (mut sum, mut n) = ([0.0f32; 3], 0.0f32);
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    let (sx, sy) = ((2 * x + dx).min(w - 1), (2 * y + dy).min(h - 1));
                    let i = sy * w + sx;
                    if k[i] {
                        for j in 0..3 {
                            sum[j] += c[i][j];
                        }
                        n += 1.0;
                    }
                }
                if n > 0.0 {
                    nc[y * nw + x] = [sum[0] / n, sum[1] / n, sum[2] / n];
                    nk[y * nw + x] = true;
                }
            }
        }
        levels.push((nw, nh, nc, nk));
    }
    // Pull: unknown texels take their parent's colour, coarsest first.
    for l in (0..levels.len() - 1).rev() {
        let (finer, coarser) = levels.split_at_mut(l + 1);
        let (pw, _, pc, _) = &coarser[0];
        let (w, h, c, k) = &mut finer[l];
        for y in 0..*h {
            for x in 0..*w {
                let i = y * *w + x;
                if !k[i] {
                    c[i] = pc[(y / 2) * pw + x / 2];
                    k[i] = true;
                }
            }
        }
    }
    let c = &levels[0].2;
    let mut rgba = img.rgba.clone();
    for (p, c) in rgba.as_chunks_mut::<4>().0.iter_mut().zip(c) {
        if p[3] == 0 {
            p[0] = c[0].round() as u8;
            p[1] = c[1].round() as u8;
            p[2] = c[2].round() as u8;
        }
    }
    Some(Image {
        width: img.width,
        height: img.height,
        rgba,
    })
}

/// A cut-out's mips ([`TextureOptions::cutout`]), before [`keep_coverage`]: each level half the one before,
/// rounded down to at least 1, each texel the level before sampled bilinearly at its centre. That is the plain mean
/// of the 2 × 2 texels under it where a size is even, and the two texels nearest the centre where it is odd: a card
/// 8 texels by 3 whose middle row holds the leaves halves to 4 × 1 from that row, where a filter that reaches the
/// rows above and below (an area box, or the encoder's Kaiser, which also wraps round the edges) averages in two
/// nearly transparent ones and takes the alpha below the cut-off. Clamped at the edges, never wrapping round them;
/// colour in linear light on an sRGB map, alpha (and linear data) as stored. This is the chain a WebGL
/// `generateMipmap` gives on the GPUs measured (NVIDIA's linear blit). Levels 1 to 1 × 1.
#[cfg_attr(not(feature = "basis"), allow(dead_code))]
fn cutout_mips(img: &Image, srgb: bool) -> Vec<Image> {
    let to_lin: Vec<f32> = (0..256)
        .map(|v| {
            let c = v as f32 / 255.0;
            if !srgb {
                c
            } else if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        })
        .collect();
    let to_byte = |c: f32| -> u8 {
        let c = c.clamp(0.0, 1.0);
        let e = if !srgb {
            c
        } else if c <= 0.0031308 {
            c * 12.92
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        };
        (e * 255.0).round() as u8
    };
    // Per axis: for each target texel, the two source texels round its
    // centre and their bilinear weights (clamped at the edge).
    let spans = |n: usize, m: usize| -> Vec<[(usize, f32); 2]> {
        (0..m)
            .map(|i| {
                let x = ((i as f32 + 0.5) * n as f32 / m as f32 - 0.5).max(0.0);
                let j = (x.floor() as usize).min(n - 1);
                let f = x - j as f32;
                [(j, 1.0 - f), ((j + 1).min(n - 1), f)]
            })
            .collect()
    };
    let mut out = Vec::new();
    let (mut w, mut h) = (img.width as usize, img.height as usize);
    // The level before: colour in linear light, alpha as stored.
    let mut px: Vec<[f32; 4]> = img
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| {
            [
                to_lin[p[0] as usize],
                to_lin[p[1] as usize],
                to_lin[p[2] as usize],
                p[3] as f32 / 255.0,
            ]
        })
        .collect();
    while w > 1 || h > 1 {
        let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
        let (sx, sy) = (spans(w, nw), spans(h, nh));
        let mut next = vec![[0.0f32; 4]; nw * nh];
        for (y, ry) in sy.iter().enumerate() {
            for (x, rx) in sx.iter().enumerate() {
                let acc = &mut next[y * nw + x];
                for &(j, wy) in ry {
                    for &(i, wx) in rx {
                        let p = px[j * w + i];
                        for c in 0..4 {
                            acc[c] += p[c] * wx * wy;
                        }
                    }
                }
            }
        }
        let rgba = next
            .iter()
            .flat_map(|p| {
                [
                    to_byte(p[0]),
                    to_byte(p[1]),
                    to_byte(p[2]),
                    (p[3].clamp(0.0, 1.0) * 255.0).round() as u8,
                ]
            })
            .collect();
        out.push(Image {
            width: nw as u32,
            height: nh as u32,
            rgba,
        });
        (w, h, px) = (nw, nh, next);
    }
    out
}

/// Raises the alpha of each cut-out level that keeps fewer texels at or above the cut-off `cut` (the least alpha
/// byte the material keeps) than the first level does, in share (alpha-to-coverage mips: Castaño 2010, NVTT's
/// `scaleAlphaToCoverage`, here upwards only): the texel that brings the level to that share, rounded up to a
/// whole texel, comes to [`COVERAGE_MARGIN`] over the cut-off, and every alpha is scaled with it, so a
/// box-filtered card's thin leaves stay drawn when minified instead of fading under the cut-off. A level that
/// keeps the share is left as it is: the bilinear chain alone is what WebGL draws, so a game tuned there keeps its
/// look.
#[cfg_attr(not(feature = "basis"), allow(dead_code))]
fn keep_coverage(levels: &mut [Image], first: &Image, cut: u8) {
    let kept = |img: &Image| {
        img.rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[3] >= cut)
            .count() as u64
    };
    let (k0, n0) = (
        kept(first),
        u64::from(first.width) * u64::from(first.height),
    );
    for l in levels {
        let n = u64::from(l.width) * u64::from(l.height);
        let need = (k0 * n).div_ceil(n0);
        if kept(l) >= need {
            continue;
        }
        let mut alpha: Vec<u8> = l.rgba.as_chunks::<4>().0.iter().map(|p| p[3]).collect();
        alpha.sort_unstable_by(|a, b| b.cmp(a));
        let at = alpha[need as usize - 1];
        if at == 0 {
            continue;
        }
        let scale = f32::from(cut.saturating_add(COVERAGE_MARGIN)) / f32::from(at);
        for p in l.rgba.as_chunks_mut::<4>().0 {
            p[3] = (f32::from(p[3]) * scale).round().min(255.0) as u8;
        }
    }
}

/// How far over the cut-off [`keep_coverage`] raises a level, in alpha bytes: about the alpha error ETC1S and
/// UASTC make there. Raised to the cut-off itself, 7 of the 389 levels (of 16 texels or more) of a game's 57
/// cut-out textures (daguanyuan's) still lost half their coverage once encoded in ETC1S, and 1 in UASTC; raised 8
/// over it, none did.
const COVERAGE_MARGIN: u8 = 8;

fn has_alpha(img: &Image) -> bool {
    img.rgba.as_chunks::<4>().0.iter().any(|p| p[3] != 255)
}

#[cfg(feature = "basis")]
fn encode_basisu(img: &Image, k: u32, opts: &TextureOptions) -> Result<Vec<u8>, String> {
    use basis_universal::{BasisTextureFormat, ColorSpace, Compressor, CompressorParams};
    let mut params = CompressorParams::new();
    params.set_print_status_to_stdout(false);
    let uastc = k != kind::COLOR || opts.color_uastc;
    params.set_basis_format(if uastc {
        BasisTextureFormat::UASTC4x4
    } else {
        BasisTextureFormat::ETC1S
    });
    if uastc {
        params.set_uastc_quality_level(opts.uastc_level);
        params.set_rdo_uastc(opts.uastc_rdo);
    } else {
        params.set_etc1s_quality_level(opts.etc1s_quality);
    }
    let space = if k == kind::COLOR {
        ColorSpace::Srgb
    } else {
        ColorSpace::Linear
    };
    params.set_color_space(space);
    params.set_mip_color_space(space);
    if k == kind::NORMAL {
        params.tune_for_normal_maps();
    }
    params
        .source_image_mut(0)
        .init(&img.rgba, img.width, img.height, 4);
    if let Some(cut) = opts.cutout.filter(|_| opts.mipmaps) {
        // A cut-out's own levels, handed to basisu whole.
        params.set_generate_mipmaps(false);
        let mut levels = cutout_mips(img, k == kind::COLOR);
        keep_coverage(&mut levels, img, cut);
        for (i, l) in levels.iter().enumerate() {
            params
                .source_mipmap_image_mut(0, i as u32)
                .init(&l.rgba, l.width, l.height, 4);
        }
    } else {
        params.set_generate_mipmaps(opts.mipmaps);
        params.set_mipmap_smallest_dimension(1);
    }
    // One thread per texture: the cook parallelises across assets, and a
    // single-threaded encode is bit-for-bit reproducible.
    let mut c = Compressor::new(1);
    unsafe {
        if !c.init(&params) {
            return Err("basisu rejected the parameters".into());
        }
        c.process().map_err(|e| format!("basisu: {e:?}"))?;
    }
    Ok(c.basis_file().to_vec())
}

fn encode_toktx(img: &Image, k: u32, opts: &TextureOptions) -> Result<Vec<u8>, String> {
    toktx_version().map_err(|e| e.0)?;
    // One directory per encode: two textures of one size cook at once.
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "og-cook-toktx-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let (src, dst) = (dir.join("in.png"), dir.join("out.ktx2"));
    image::save_buffer(
        &src,
        &img.rgba,
        img.width,
        img.height,
        image::ColorType::Rgba8,
    )
    .map_err(|e| e.to_string())?;
    let mut cmd = std::process::Command::new("toktx");
    cmd.arg("--t2");
    if k == kind::COLOR && !opts.color_uastc {
        cmd.args([
            "--encode",
            "etc1s",
            "--clevel",
            "1",
            "--qlevel",
            &opts.etc1s_quality.to_string(),
        ]);
        cmd.args(["--assign_oetf", "srgb"]);
    } else {
        cmd.args([
            "--encode",
            "uastc",
            "--uastc_quality",
            &opts.uastc_level.to_string(),
        ]);
        if let Some(rdo) = opts.uastc_rdo {
            cmd.args(["--uastc_rdo_l", &rdo.to_string()]);
        }
        if opts.zstd {
            cmd.args(["--zcmp", "3"]);
        }
        // Not --normal_mode, which would keep only x (in RGB) and y (in
        // alpha): `cook` has already copied y into alpha and kept xyz.
        cmd.args([
            "--assign_oetf",
            if k == kind::COLOR { "srgb" } else { "linear" },
        ]);
    }
    if opts.mipmaps {
        cmd.arg("--genmipmap");
    }
    cmd.arg(&dst).arg(&src);
    let out = cmd.output().map_err(|e| format!("toktx: {e}"))?;
    let result = if out.status.success() {
        std::fs::read(&dst).map_err(|e| e.to_string())
    } else {
        Err(format!(
            "toktx failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    };
    let _ = std::fs::remove_dir_all(&dir);
    result
}

/// `toktx`'s KTX2 with its `KTXorientation` and `KTXswizzle` honoured
/// (`ktx2_source::honour`: re-encoded flipped and swizzled when not the
/// identity).
fn toktx(img: &Image, k: u32, opts: &TextureOptions) -> Result<Vec<u8>, String> {
    let ktx = encode_toktx(img, k, opts)?;
    super::ktx2_source::honour(ktx, |fixed| encode_toktx(fixed, k, opts))
}

/// Cooks an image into a texture blob.
pub fn cook(img: &Image, k: u32, opts: &TextureOptions, stamp: u64) -> Result<Vec<u8>, CookError> {
    if img.width == 0 || img.height == 0 {
        return Err(CookError("image has no pixels".into()));
    }
    let alpha = has_alpha(img);
    if k == kind::PIXEL {
        // Pixel art: lossless RGBA8, sRGB, one level (the runtime samples
        // it nearest; mips would blur sprite sheets across frames).
        let ktx = ktx2::write_rgba8_zlib(
            img.width,
            img.height,
            &img.rgba,
            true,
            false,
            crate::format::COOKER_VERSION,
        );
        let meta = TextureMeta {
            width: img.width,
            height: img.height,
            levels: 1,
            kind: k,
            codec: codec::RGBA8,
            color_space: 0,
            has_alpha: alpha as u32,
            supercompression: 0,
        };
        let mut w = BlobWriter::new(
            format::Texture::type_hash(),
            format::Texture::schema_version(),
            stamp,
        );
        w.table("meta", &[meta], row_hash::<TextureMeta>())
            .raw("ktx2", ktx, 16);
        return Ok(w.finish());
    }
    let srgb = k == kind::COLOR;
    // A normal map keeps x, y, z in RGB (RGBA8, BC7, ASTC and ETC2 read
    // them) and y again in alpha, where Basis transcoders take a
    // two-channel format's second channel from (BC5 and EAC RG11 are R and
    // A): `has_alpha` then says the map carries y there.
    let normal;
    let (img, alpha) = if k == kind::NORMAL {
        let mut rgba = img.rgba.clone();
        for p in rgba.as_chunks_mut::<4>().0 {
            p[3] = p[1];
        }
        normal = Image {
            width: img.width,
            height: img.height,
            rgba,
        };
        (&normal, true)
    } else {
        (img, alpha)
    };
    // Transparent texels of a mipmapped colour texture take their visible
    // neighbours' colour, so the mips do not average black into the edges.
    let dilated;
    let img = match (k == kind::COLOR && alpha && opts.mipmaps)
        .then(|| dilate_transparent(img))
        .flatten()
    {
        Some(d) => {
            dilated = d;
            &dilated
        }
        None => img,
    };
    #[cfg(feature = "basis")]
    let ktx = if opts.force_toktx {
        toktx(img, k, opts)
    } else {
        encode_basisu(img, k, opts).and_then(|basis| {
            ktx2::from_basis(&basis, srgb, opts.zstd, crate::format::COOKER_VERSION)
        })
    };
    #[cfg(not(feature = "basis"))]
    let ktx = toktx(img, k, opts);
    let ktx = ktx.map_err(CookError)?;
    let info = ktx2::parse(&ktx)
        .map_err(|e| CookError(format!("the encoder wrote an invalid KTX2: {e}")))?;
    let meta = TextureMeta {
        width: img.width,
        height: img.height,
        levels: info.levels.len() as u32,
        kind: k,
        codec: if k == kind::COLOR && !opts.color_uastc {
            codec::ETC1S
        } else {
            codec::UASTC
        },
        color_space: if srgb { 0 } else { 1 },
        has_alpha: alpha as u32,
        supercompression: info.supercompression,
    };
    let mut w = BlobWriter::new(
        format::Texture::type_hash(),
        format::Texture::schema_version(),
        stamp,
    );
    w.table("meta", &[meta], row_hash::<TextureMeta>())
        .raw("ktx2", ktx, 16);
    Ok(w.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transparent_texels_take_their_nearest_visible_colour() {
        // Red, two transparent black texels, half-transparent blue: each
        // hidden texel takes the colour of the visible one beside it (not
        // the mean of both), keeps alpha 0, and the visible ones keep theirs.
        let img = Image {
            width: 4,
            height: 1,
            rgba: vec![255, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 128],
        };
        let d = dilate_transparent(&img).expect("has transparent texels");
        assert_eq!(
            d.rgba,
            [255, 0, 0, 255, 255, 0, 0, 0, 0, 0, 255, 0, 0, 0, 255, 128]
        );
        let opaque = Image {
            width: 1,
            height: 1,
            rgba: vec![9, 9, 9, 255],
        };
        assert!(dilate_transparent(&opaque).is_none());
    }

    fn alpha_image(width: u32, height: u32, alpha: &[u8]) -> Image {
        Image {
            width,
            height,
            rgba: alpha.iter().flat_map(|&a| [255, 255, 255, a]).collect(),
        }
    }

    fn alphas(img: &Image) -> Vec<u8> {
        img.rgba.as_chunks::<4>().0.iter().map(|p| p[3]).collect()
    }

    /// A bamboo spray card (daguanyuan's zhu_spray: 1024 × 384, alpha-tested at 0.35, three quarters of its texels
    /// transparent, its leaves across the middle) at 32 × 12: white leaves in rows 4 to 7, three columns in four.
    fn bamboo_card() -> Image {
        let alpha: Vec<u8> = (0..12)
            .flat_map(|y| {
                (0..32).map(move |x| {
                    if (4..8).contains(&y) && x % 4 != 3 {
                        255
                    } else {
                        0
                    }
                })
            })
            .collect();
        alpha_image(32, 12, &alpha)
    }

    #[test]
    fn cutout_mips_halve_to_one_texel_with_the_centre_taps() {
        // 4 x 2, alpha 1 0 1 1 / 0 0 1 1: level 1 (2 x 1) is the 2 x 2
        // means 0.25 and 1, level 2 (1 x 1) their mean 0.625.
        let m = cutout_mips(
            &alpha_image(4, 2, &[255, 0, 255, 255, 0, 0, 255, 255]),
            true,
        );
        let sizes: Vec<_> = m.iter().map(|l| (l.width, l.height)).collect();
        assert_eq!(sizes, [(2, 1), (1, 1)]);
        assert_eq!((m[0].rgba[3], m[0].rgba[7], m[1].rgba[3]), (64, 255, 159));
        assert_eq!(&m[1].rgba[0..3], &[255, 255, 255]);
        // Colour averages in linear light on an sRGB map (black and white:
        // linear 0.5, sRGB 188), as stored on linear data (128).
        let bw = Image {
            width: 2,
            height: 1,
            rgba: vec![0, 0, 0, 255, 255, 255, 255, 255],
        };
        assert_eq!(cutout_mips(&bw, true)[0].rgba[0], 188);
        assert_eq!(cutout_mips(&bw, false)[0].rgba[0], 128);
        // A mid grey shows the light it averages in: black and sRGB 128
        // meet at 92 (linear 0.108), not at 137 (the stored values' mean
        // encoded again) or 64 (the stored values' mean).
        let grey = Image {
            width: 2,
            height: 1,
            rgba: vec![0, 0, 0, 255, 128, 128, 128, 255],
        };
        assert_eq!(cutout_mips(&grey, true)[0].rgba[0], 92);
        // Odd sizes take the taps nearest the centre, clamped, never
        // wrapping: 3 -> 1 is the middle texel; 5 -> 2 samples at 0.75 and
        // 3.25 (0.25 a0 + 0.75 a1, 0.75 a3 + 0.25 a4).
        assert_eq!(
            cutout_mips(&alpha_image(3, 1, &[0, 200, 255]), false)[0].rgba[3],
            200
        );
        let five = &cutout_mips(&alpha_image(5, 1, &[0, 100, 200, 40, 0]), false)[0];
        assert_eq!((five.width, five.rgba[3], five.rgba[7]), (2, 75, 30));
        // The bamboo card: 16 x 6 holds its leaf rows as pairs of columns
        // with two leaves (255) and one (128); 8 x 3 their mean, 191, in its
        // middle row and nothing above or below; 4 x 1 that row alone, not
        // the three rows' mean (64, under the cut-off 0.35 = 89), and so to
        // 1 x 1: every level keeps a texel at or above the cut-off where
        // its leaves are.
        let m = cutout_mips(&bamboo_card(), true);
        let sizes: Vec<_> = m.iter().map(|l| (l.width, l.height)).collect();
        assert_eq!(sizes, [(16, 6), (8, 3), (4, 1), (2, 1), (1, 1)]);
        let leaves: Vec<u8> = (0..16)
            .map(|x| if x % 2 == 0 { 255 } else { 128 })
            .collect();
        let none = [0u8; 16];
        assert_eq!(
            alphas(&m[0]),
            [&none[..], &none, &leaves, &leaves, &none, &none].concat()
        );
        assert_eq!(alphas(&m[1]), [[0; 8], [191; 8], [0; 8]].concat());
        assert_eq!(alphas(&m[2]), [191; 4]);
        assert_eq!((alphas(&m[3]), alphas(&m[4])), (vec![191; 2], vec![191]));
        assert!(
            m.iter()
                .all(|l| l.rgba.as_chunks::<4>().0.iter().all(|p| p[..3] == [255; 3]))
        );
    }

    #[test]
    fn keep_coverage_raises_only_the_levels_that_lose_it() {
        // A thin stalk: one column of alpha 200 in 8 x 8, cut at 0.5 (128),
        // 1 texel in 8 kept. The chain halves it to 100 (4 x 4), 50 (2 x 2)
        // and 25 (1 x 1), every level under the cut-off; raised, each level's
        // brightest texels come to 136 (the cut-off and the margin, 8), every
        // alpha scaled with them: the column at 4 x 4 (4 texels in 16, at
        // least the 2 the share asks), one texel in two at 2 x 2, the last one.
        let stalk: Vec<u8> = (0..64).map(|i| if i % 8 == 2 { 200 } else { 0 }).collect();
        let first = alpha_image(8, 8, &stalk);
        let mut m = cutout_mips(&first, true);
        assert_eq!(alphas(&m[0]), [[0, 100, 0, 0]; 4].concat());
        assert_eq!(
            (alphas(&m[1]), alphas(&m[2])),
            (vec![50, 0, 50, 0], vec![25])
        );
        keep_coverage(&mut m, &first, 128);
        assert_eq!(alphas(&m[0]), [[0, 136, 0, 0]; 4].concat());
        assert_eq!(
            (alphas(&m[1]), alphas(&m[2])),
            (vec![136, 0, 136, 0], vec![136])
        );
        // Texels of 100 and 50 where one is asked for: the 100 comes to 136,
        // the 50 with it to 68, still under the cut-off.
        let mut two = vec![alpha_image(2, 1, &[100, 50])];
        keep_coverage(&mut two, &alpha_image(2, 1, &[255, 0]), 128);
        assert_eq!(alphas(&two[0]), [136, 68]);
        // The bamboo card keeps its share at every level of the chain (a
        // quarter at the first, a third or more after), so it is untouched.
        let card = bamboo_card();
        let chain = cutout_mips(&card, true);
        let mut kept = cutout_mips(&card, true);
        keep_coverage(&mut kept, &card, 90);
        assert!(chain.iter().zip(&kept).all(|(a, b)| a.rgba == b.rgba));
    }

    /// A cut-out's levels reach the encoded texture, and only a cut-out's: the bamboo card through basisu keeps a
    /// texel at or above its cut-off at every level as a cut-out, where as an ordinary colour texture the encoder's
    /// own filter (Kaiser, wrapping round the edges) spreads the leaf rows over the transparent ones until, from
    /// 4 x 1 down, no texel reaches the cut-off and the card is cut away.
    #[cfg(feature = "basis")]
    #[test]
    fn only_a_cut_out_keeps_its_coverage_through_the_encoded_levels() {
        use basis_universal::{TranscodeParameters, Transcoder, TranscoderTextureFormat};
        let encoded = |cutout: Option<u8>| -> Vec<(u32, Vec<u8>)> {
            let opts = TextureOptions {
                color_uastc: true,
                cutout,
                ..TextureOptions::default()
            };
            let basis = encode_basisu(&bamboo_card(), kind::COLOR, &opts).unwrap();
            let mut t = Transcoder::new();
            t.prepare_transcoding(&basis).unwrap();
            let levels = (0..t.image_level_count(&basis, 0))
                .map(|level_index| {
                    let width = t
                        .image_level_description(&basis, 0, level_index)
                        .unwrap()
                        .original_width;
                    let rgba = t
                        .transcode_image_level(
                            &basis,
                            TranscoderTextureFormat::RGBA32,
                            TranscodeParameters {
                                image_index: 0,
                                level_index,
                                ..Default::default()
                            },
                        )
                        .unwrap();
                    (
                        width,
                        rgba.as_chunks::<4>().0.iter().map(|p| p[3]).collect(),
                    )
                })
                .collect();
            t.end_transcoding();
            levels
        };
        // The share of a level's texels at or above the cut-off (0.35: 90).
        let coverage = |a: &[u8]| a.iter().filter(|&&a| a >= 90).count() as f32 / a.len() as f32;
        let cutout = encoded(Some(90));
        assert_eq!(cutout.len(), 6, "32x12 to 1x1");
        assert_eq!(coverage(&cutout[0].1), 0.25);
        for (i, (width, a)) in cutout.iter().enumerate() {
            assert!(
                coverage(a) >= 0.25,
                "cut-out level {i} ({width} wide): {a:?}"
            );
        }
        let plain = encoded(None);
        assert_eq!(plain.len(), 6);
        for (i, (width, a)) in plain.iter().enumerate().skip(3) {
            assert_eq!(coverage(a), 0.0, "ordinary level {i} ({width} wide): {a:?}");
        }
    }
}
