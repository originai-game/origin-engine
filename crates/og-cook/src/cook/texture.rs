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
        Ok(out) => Ok(String::from_utf8_lossy(&out.stdout).trim().to_string()),
        Err(_) => Err(CookError(
            "texture cooking needs a Basis Universal encoder: build og-cook with its `basis` feature (the default), \
             or install KTX-Software's `toktx` (https://github.com/KhronosGroup/KTX-Software/releases) on PATH; \
             `og doctor` checks for both"
                .into(),
        )),
    }
}

fn has_alpha(img: &Image) -> bool {
    img.rgba.chunks_exact(4).any(|p| p[3] != 255)
}

#[cfg(feature = "basis")]
fn encode_basisu(img: &Image, k: u32, opts: &TextureOptions) -> Result<Vec<u8>, String> {
    use basis_universal::{BasisTextureFormat, ColorSpace, Compressor, CompressorParams};
    let mut params = CompressorParams::new();
    params.set_print_status_to_stdout(false);
    let uastc = k != kind::COLOR;
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
    let dir = std::env::temp_dir().join(format!(
        "og-cook-toktx-{}-{}",
        std::process::id(),
        img.rgba.len()
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
    if k == kind::COLOR {
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
        cmd.args(["--assign_oetf", "linear"]);
        if k == kind::NORMAL {
            cmd.arg("--normal_mode");
        }
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

/// Cooks an image into a texture blob.
pub fn cook(img: &Image, k: u32, opts: &TextureOptions, stamp: u64) -> Result<Vec<u8>, CookError> {
    if img.width == 0 || img.height == 0 {
        return Err(CookError("image has no pixels".into()));
    }
    let alpha = has_alpha(img);
    let srgb = k == kind::COLOR;
    #[cfg(feature = "basis")]
    let ktx = if opts.force_toktx {
        encode_toktx(img, k, opts)
    } else {
        encode_basisu(img, k, opts).and_then(|basis| {
            ktx2::from_basis(&basis, srgb, opts.zstd, crate::format::COOKER_VERSION)
        })
    };
    #[cfg(not(feature = "basis"))]
    let ktx = encode_toktx(img, k, opts);
    let ktx = ktx.map_err(CookError)?;
    let info = ktx2::parse(&ktx)
        .map_err(|e| CookError(format!("the encoder wrote an invalid KTX2: {e}")))?;
    let meta = TextureMeta {
        width: img.width,
        height: img.height,
        levels: info.levels.len() as u32,
        kind: k,
        codec: if k == kind::COLOR {
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
