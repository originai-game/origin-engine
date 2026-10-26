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

/// Decodes PNG or JPEG bytes to RGBA8.
pub fn decode(bytes: &[u8]) -> Result<Image, String> {
    let img = image::load_from_memory(bytes).map_err(|e| format!("image: {e}"))?;
    let rgba = img.to_rgba8();
    Ok(Image {
        width: rgba.width(),
        height: rgba.height(),
        rgba: rgba.into_raw(),
    })
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

/// Fills the colour of every fully transparent texel from its visible
/// neighbourhood (push-pull: a pyramid of the visible texels' mean colour,
/// read back from the finest level that has any). Unseen at full size, that
/// colour is what the mips average into the visible texels: left black (a
/// canvas export premultiplies, so alpha 0 comes out black), leaf and petal
/// cards darkened with distance. Visible texels are untouched.
fn dilate_transparent(img: &Image) -> Option<Image> {
    let (w0, h0) = (img.width as usize, img.height as usize);
    if !img.rgba.as_chunks::<4>().0.iter().any(|p| p[3] == 0) || img.rgba.as_chunks::<4>().0.iter().all(|p| p[3] == 0) {
        return None;
    }
    // level 0: colour and whether it is known
    let mut levels: Vec<(usize, usize, Vec<[f32; 3]>, Vec<bool>)> = Vec::new();
    let c0 = img.rgba.as_chunks::<4>().0.iter().map(|p| [p[0] as f32, p[1] as f32, p[2] as f32]).collect();
    let k0 = img.rgba.as_chunks::<4>().0.iter().map(|p| p[3] > 0).collect();
    levels.push((w0, h0, c0, k0));
    while {
        let (w, h, _, _) = &levels[levels.len() - 1];
        *w > 1 || *h > 1
    } {
        let (w, h, c, k) = &levels[levels.len() - 1];
        let (nw, nh) = ((w + 1) / 2, (h + 1) / 2);
        let mut nc = vec![[0.0f32; 3]; nw * nh];
        let mut nk = vec![false; nw * nh];
        for y in 0..nh {
            for x in 0..nw {
                let (mut sum, mut n) = ([0.0f32; 3], 0.0f32);
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    let (sx, sy) = ((2 * x + dx).min(w - 1), (2 * y + dy).min(h - 1));
                    let i = sy * w + sx;
                    if k[i] {
                        for j in 0..3 { sum[j] += c[i][j]; }
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
    // pull: unknown texels take their parent's colour, coarsest first
    for l in (0..levels.len() - 1).rev() {
        let (pw, _, pc, _) = { let p = &levels[l + 1]; (p.0, p.1, p.2.clone(), p.3.clone()) };
        let (w, h, c, k) = &mut levels[l];
        for y in 0..*h {
            for x in 0..*w {
                let i = y * *w + x;
                if !k[i] { c[i] = pc[(y / 2) * pw + x / 2]; k[i] = true; }
            }
        }
    }
    let c = &levels[0].2;
    let mut rgba = img.rgba.clone();
    for (i, p) in rgba.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        if p[3] == 0 {
            p[0] = c[i][0].round() as u8;
            p[1] = c[i][1].round() as u8;
            p[2] = c[i][2].round() as u8;
        }
    }
    Some(Image { width: img.width, height: img.height, rgba })
}

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
    params.set_generate_mipmaps(opts.mipmaps);
    params.set_mipmap_smallest_dimension(1);
    params
        .source_image_mut(0)
        .init(&img.rgba, img.width, img.height, 4);
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
        cmd.args(["--assign_oetf", if k == kind::COLOR { "srgb" } else { "linear" }]);
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
    let dilated;
    let img = if k == kind::COLOR && alpha && opts.mipmaps {
        match dilate_transparent(img) {
            Some(d) => { dilated = d; &dilated }
            None => img,
        }
    } else {
        img
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
