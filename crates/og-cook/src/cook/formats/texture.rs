//! Images → KTX2 Basis Universal texture blobs: ETC1S (BasisLZ) for colour,
//! UASTC (+ Zstandard) for normals and linear data, with a full mip chain.
//!
//! The encoder is the basis-universal crate (basisu 1.16) when og-cook is
//! built with the `basis` feature; otherwise KTX-Software's `toktx` found on
//! PATH. With neither, texture cooking fails with a message naming both:
//! a texture is never passed through uncompressed.

use super::{
    Action, CookError, Cx, Exec, Job, KindDef, Out, Path, PlanState, Planned, Source, Split,
    TextureOptions, conform, ext, ktx2_source, mesh_texture, normal2d, not_split, settings_block,
    stamp, sway2d, texture_kind, texture_stream,
};
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
        // `currentColor` is white, the tint, as in a cooked vector: a
        // single-colour SVG is a mask its draw colours.
        let options = resvg::usvg::Options {
            style_sheet: Some("svg { color: white }".into()),
            ..Default::default()
        };
        let tree =
            resvg::usvg::Tree::from_data(bytes, &options).map_err(|e| format!("SVG: {e}"))?;
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

/// A source image's EXIF orientation (1 to 8; 1 upright), read from its
/// metadata alone: what a JPEG, PNG (`eXIf`), WebP or TIFF carries. The
/// texels stay as stored; the texture keeps it for the draw to apply
/// (CSS `image-orientation`).
pub fn orientation(bytes: &[u8]) -> u32 {
    use image::ImageDecoder;
    image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()
        .and_then(|r| r.into_decoder().ok())
        .and_then(|mut d| d.orientation().ok())
        .map_or(1, |o| u32::from(o.to_exif()))
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
    if let Some(o) = blob.raw_opt("orientation") {
        w.raw("orientation", o, 4);
    }
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
        return Ok(
            "basisu 1.16 (basis-universal crate, UASTC level as set, SSE colour distance fixed, fp-contract=off)".into(),
        );
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
pub(crate) fn dilate_transparent(img: &Image) -> Option<Image> {
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

/// Whether every texel on the image's border is fully transparent.
fn clear_border(img: &Image) -> bool {
    let (w, h) = (img.width as usize, img.height as usize);
    let a = |x: usize, y: usize| img.rgba[(y * w + x) * 4 + 3];
    w > 0
        && h > 0
        && (0..w).all(|x| a(x, 0) == 0 && a(x, h - 1) == 0)
        && (0..h).all(|y| a(0, y) == 0 && a(w - 1, y) == 0)
}

fn has_alpha(img: &Image) -> bool {
    img.rgba.as_chunks::<4>().0.iter().any(|p| p[3] != 255)
}

#[cfg(feature = "basis")]
fn encode_basisu(img: &Image, k: u32, opts: &TextureOptions) -> Result<Vec<u8>, String> {
    use basis_universal::{BasisTextureFormat, ColorSpace, Compressor, CompressorParams};
    let mut params = CompressorParams::new();
    params.set_print_status_to_stdout(false);
    let uastc = opts.uastc(k);
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
    let clear = clear_border(img);
    if opts.mipmaps && (opts.cutout.is_some() || clear) {
        // Our own levels, handed to basisu whole: a cut-out's keep their
        // coverage; an image with a clear border (a UI plate, a sprite)
        // keeps its clear texels clear, where basisu's Kaiser filter spreads
        // alpha a texel or two past every edge (a chamfer's clear corner
        // took alpha 39 at level 2, a dark line beside it on screen).
        params.set_generate_mipmaps(false);
        let mut levels = cutout_mips(img, k == kind::COLOR);
        if let Some(cut) = opts.cutout {
            keep_coverage(&mut levels, img, cut);
        }
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
    if !opts.uastc(k) {
        cmd.args([
            "--encode",
            "etc1s",
            "--clevel",
            "1",
            "--qlevel",
            &opts.etc1s_quality.to_string(),
        ]);
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
    }
    // `cook` supplies either xyz/y or the ETC1S normal-mode xxx/y layout.
    cmd.args([
        "--assign_oetf",
        if k == kind::COLOR { "srgb" } else { "linear" },
    ]);
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
    cook_oriented(img, k, opts, stamp, 1)
}

/// [`cook`] for an image whose source carries EXIF orientation
/// `orientation` ([`orientation`]): kept in the raw section `orientation`
/// (a little-endian u32) when it is not 1, the texels as stored.
pub fn cook_oriented(
    img: &Image,
    k: u32,
    opts: &TextureOptions,
    stamp: u64,
    orientation: u32,
) -> Result<Vec<u8>, CookError> {
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
        if orientation != 1 {
            w.raw("orientation", orientation.to_le_bytes(), 4);
        }
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
            if opts.data_etc1s {
                p[1] = p[0];
                p[2] = p[0];
            }
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
        codec: if !opts.uastc(k) {
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
    if orientation != 1 {
        w.raw("orientation", orientation.to_le_bytes(), 4);
    }
    Ok(w.finish())
}

/// An image file, a `.ktx2` as it is, or a sprite's 2D normal map.
pub(crate) static TEXTURE: KindDef = KindDef {
    name: "texture",
    extensions: &[
        "png", "jpg", "jpeg", "ktx2", "webp", "tga", "psd", "dds", "svg",
    ],
    types: &[],
    by_content: true,
    streams: true,
    plan: plan_texture,
};

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&TEXTURE];

fn plan_texture<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx {
        options,
        group_of,
        tex_desc,
        ..
    } = *cx;
    let PlanState { early_errors, .. } = state;
    match texture_kind(s).and_then(|k| Ok((k, texture_stream(s, k)?))) {
        Ok((k, stream)) => {
            // A mesh's texture under the art contract: its desc too.
            let mesh = mesh_texture(s, options)
                .map(|c| c.texture_desc())
                .unwrap_or_default();
            let kind = k.to_le_bytes();
            let st = |word: &[u8]| {
                let mut parts: Vec<&[u8]> = vec![b"texture", tex_desc.as_bytes(), &hash, &kind];
                if !mesh.is_empty() {
                    parts.push(mesh.as_bytes());
                }
                if !word.is_empty() {
                    parts.push(word);
                }
                stamp(&parts)
            };
            let source = s.conformed.as_deref().map_or(&s.bytes[..], |b| &b[..]);
            match stream.then(|| not_split(source, k, group_of.get(&s.rel))) {
                Some(None) => {
                    let mips = st(b"mips");
                    outs.push((
                        s.rel.clone(),
                        format::Texture::NAME,
                        st(b"tail"),
                        Box::new(TextureJob(s, k, Split::Tail(mips))),
                    ));
                    outs.push((
                        crate::runtime::mips_id(&s.rel),
                        format::Texture::NAME,
                        mips,
                        Box::new(TextureJob(s, k, Split::Mips)),
                    ));
                }
                why => outs.push((
                    s.rel.clone(),
                    format::Texture::NAME,
                    st(b""),
                    Box::new(TextureJob(s, k, Split::Whole(why.flatten()))),
                )),
            }
            // Its normal map for 2D lighting (`normal auto` or a file).
            match normal2d::setting(settings_block(s, "texture")) {
                Ok(None) => {}
                Ok(Some(setting)) => {
                    let file = match &setting {
                        normal2d::NormalSetting::Auto => Ok(None),
                        normal2d::NormalSetting::File(name) => {
                            let path = s.path.parent().unwrap_or(Path::new(".")).join(name);
                            std::fs::read(&path)
                                .map(Some)
                                .map_err(|e| format!("normal \"{name}\": {e}"))
                        }
                    };
                    match file {
                        Ok(file) => {
                            let st = stamp(&[
                                b"texture normal",
                                tex_desc.as_bytes(),
                                &hash,
                                file.as_deref().unwrap_or(b"auto"),
                            ]);
                            outs.push((
                                format!("{}#normal", s.rel),
                                format::Texture::NAME,
                                st,
                                Box::new(TextureNormalJob(s, file)),
                            ));
                        }
                        Err(e) => early_errors.push((s.rel.clone(), "texture", e)),
                    }
                }
                Err(e) => early_errors.push((s.rel.clone(), "texture", e)),
            }
            // Its parts for og.Wind (`sway clumps`, `sway tiers`, a map).
            match sway2d::setting(settings_block(s, "texture")) {
                Ok(None) => {}
                Ok(Some(setting)) => {
                    let map = setting.map.as_ref().map_or(Ok(None), |name| {
                        let path = s.path.parent().unwrap_or(Path::new(".")).join(name);
                        std::fs::read(&path)
                            .map(Some)
                            .map_err(|e| format!("sway \"{name}\": {e}"))
                    });
                    match map {
                        Ok(map) => {
                            let st = stamp(&[
                                b"texture sway 1",
                                tex_desc.as_bytes(),
                                &hash,
                                &kind,
                                map.as_deref().unwrap_or(b"found"),
                            ]);
                            outs.push((
                                crate::runtime::sway_id(&s.rel),
                                format::Texture::NAME,
                                st,
                                Box::new(TextureSwayJob(s, k, setting.plant, map)),
                            ));
                        }
                        Err(e) => early_errors.push((s.rel.clone(), "texture", e)),
                    }
                }
                Err(e) => early_errors.push((s.rel.clone(), "texture", e)),
            }
        }
        Err(e) => early_errors.push((s.rel.clone(), "texture", e)),
    }
}

/// An image file's or a `.ktx2`'s output: the whole chain, or its part
/// of a streamed one.
struct TextureJob<'a>(&'a Source, u32, Split);

impl Job for TextureJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let TextureJob(s, k, split) = self;
        let Exec { options, .. } = ex;
        if s.rel.ends_with(".ktx2") {
            let explicit = settings_block(s, "texture")
                .and_then(|b| b.get("kind"))
                .map(|_| *k);
            return split_texture(s, 0, split, p, |stamp| {
                ktx2_source::cook(&s.bytes, explicit, &options.texture, stamp)
            });
        }
        split_texture(s, 0, split, p, |stamp| {
            let bytes = s.conformed.as_deref().unwrap_or(&s.bytes);
            let img = decode(bytes).map_err(CookError)?;
            if let Some(c) = mesh_texture(s, options) {
                let mut t = options.texture.clone();
                let (img, k, mips, note) = conform::mesh::texture(&c, None, img, *k, t.mipmaps);
                t.mipmaps = mips;
                let mut notes = vec![format!("{}x{} as a mesh's texture", img.width, img.height)];
                notes.extend(note.map(|n| match n.strip_prefix("conform: ") {
                    Some(w) if w != conform::mesh::UNCHANGED => {
                        format!("conform: {}; {w}", conform::mesh::DEFAULT_MESH_TEXTURE)
                    }
                    _ => n,
                }));
                return cook(&img, k, &t, stamp).map(|b| (b, notes));
            }
            let note = format!(
                "{}x{} {}",
                img.width,
                img.height,
                match *k {
                    format::kind::NORMAL => "normal (UASTC)",
                    format::kind::LINEAR => "linear (UASTC)",
                    format::kind::PIXEL => "pixel art (RGBA8, no mips)",
                    _ => "colour (ETC1S)",
                }
            );
            cook_oriented(&img, *k, &options.texture, stamp, orientation(bytes))
                .map(|b| (b, vec![note]))
        })
    }

    fn action(&self) -> Action<'_> {
        let TextureJob(s, kind, split) = self;
        Action::Input(
            s.conformed.as_deref().unwrap_or(&s.bytes).as_slice(),
            format!("{kind} {split:?} {} {:?}", ext(&s.rel), s.settings),
            true,
        )
    }
}

/// A sprite's 2D normal map: its own file's, or one made from its colour.
struct TextureNormalJob<'a>(&'a Source, Option<Vec<u8>>);

impl Job for TextureNormalJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let TextureNormalJob(s, file) = self;
        let Exec { options, .. } = ex;
        {
            let img = match file {
                Some(bytes) => decode(bytes).map_err(CookError)?,
                None => super::normal2d::auto(
                    &decode(s.conformed.as_deref().unwrap_or(&s.bytes)).map_err(CookError)?,
                ),
            };
            let note = format!(
                "{}x{} 2D normal map ({})",
                img.width,
                img.height,
                if file.is_some() { "its file" } else { "auto" }
            );
            cook(&img, format::kind::NORMAL, &options.texture, p.stamp).map(|b| (b, vec![note]))
        }
    }
}

/// A 2D plant's wind parts (`sway`): the parts packed into one texture of
/// the source's kind, with their table in its `sway` section.
struct TextureSwayJob<'a>(&'a Source, u32, sway2d::Plant, Option<Vec<u8>>);

impl Job for TextureSwayJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let TextureSwayJob(s, k, plant, map) = self;
        let Exec { options, .. } = ex;
        let img = decode(s.conformed.as_deref().unwrap_or(&s.bytes)).map_err(CookError)?;
        let map = map
            .as_deref()
            .map(|b| decode(b).map_err(|e| CookError(format!("the parts map: {e}"))))
            .transpose()?;
        let (parts_img, parts) =
            sway2d::split(&img, *plant, map.as_ref(), *k == format::kind::PIXEL)
                .map_err(CookError)?;
        let moving = parts.parts.iter().filter(|p| p.flutter > 0.0).count();
        let note = format!(
            "{}x{} wind parts: {moving} {} ({}), {} that stand or bend",
            parts_img.width,
            parts_img.height,
            match plant {
                sway2d::Plant::Clumps => "clumps",
                sway2d::Plant::Tiers => "tiers",
            },
            if map.is_some() { "its map" } else { "found" },
            parts.parts.len() - moving,
        );
        let blob = cook(&parts_img, *k, &options.texture, p.stamp)?;
        with_sway(&blob, &parts, p.stamp).map(|b| (b, vec![note]))
    }
}

/// A cooked texture's blob again with the parts table in its `sway`
/// section.
fn with_sway(
    texture: &[u8],
    parts: &crate::runtime::SwayParts,
    stamp: u64,
) -> Result<Vec<u8>, CookError> {
    let bad = |e: String| CookError(format!("the parts' texture blob: {e}"));
    let blob = og_blob::Blob::parse(texture, "texture").map_err(|e| bad(e.to_string()))?;
    let meta = blob
        .table::<TextureMeta>("meta", row_hash::<TextureMeta>())
        .map_err(|e| bad(e.to_string()))?
        .first()
        .copied()
        .ok_or_else(|| bad("no meta row".into()))?;
    let ktx = blob.raw("ktx2").map_err(|e| bad(e.to_string()))?;
    let mut w = BlobWriter::new(
        format::Texture::type_hash(),
        format::Texture::schema_version(),
        stamp,
    );
    w.table("meta", &[meta], row_hash::<TextureMeta>())
        .raw("ktx2", ktx.to_vec(), 16)
        .raw("sway", parts.to_bytes(), 4);
    Ok(w.finish())
}

/// A texture output by how it stands to its source's mip chain (G3-3):
/// `cook` makes the whole chain's blob under a stamp and its notes. A
/// streamed texture's tail and `#mips` share one cook of the whole chain
/// (under the `#mips` output's stamp, [`Source::whole_texture`]): `#mips`
/// is that blob as it is, without the notes, which its tail reports with
/// its own line.
pub(crate) fn split_texture(
    s: &Source,
    image: usize,
    split: &Split,
    p: &Planned,
    cook: impl FnOnce(u64) -> Result<(Vec<u8>, Vec<String>), CookError>,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    match split {
        Split::Whole(why) => cook(p.stamp).map(|(b, mut notes)| {
            notes.extend(why.iter().map(|w| format!("{}: {w}", p.id)));
            (b, notes)
        }),
        Split::Mips => s
            .whole_texture(image, p.stamp, || cook(p.stamp))
            .map(|(b, _)| (b, Vec::new())),
        Split::Tail(mips) => {
            let (whole, mut notes) = s.whole_texture(image, *mips, || cook(*mips))?;
            let (tail, note) = tail(&whole, p.stamp)?;
            notes.push(format!("{}: {note}", p.id));
            Ok((tail, notes))
        }
    }
}
