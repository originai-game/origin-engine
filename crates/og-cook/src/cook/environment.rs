//! The environment map cook step (phase 4): a `type environment` .ogt names
//! an equirectangular HDR image, which is prefiltered here, at cook time,
//! into og-render's `EnvironmentMapData` (`og_render::bake`): a cube of
//! RGB9E5 levels, level `m` GGX-prefiltered at roughness `m / 4` (level 0
//! the radiance), and the diffuse irradiance as nine harmonics. The runtime
//! only uploads it (`og.load.environment`).
//!
//! ```text
//! type environment
//! version 1
//! ---
//! source "studio.hdr"   # an equirectangular Radiance .hdr or OpenEXR .exr (linear RGB)
//! size 128              # level 0's face size: a power of two from 16 to 2048
//! intensity 1           # multiplies the image
//! rotation 0            # turns it about +y, degrees (what was at +x moves towards -z)
//! samples 256           # GGX samples per texel of each prefiltered level (16 to 4096)
//! ```
//!
//! The image is read beside the environment file (or, failing that, under
//! `assets/`); it is part of the environment, not an asset of its own: the
//! walk does not list it, and the environment's stamp covers its bytes. Its
//! centre column looks along +x and its three-quarter column along +z, rows
//! from +y down: the equirectangular mapping of three.js (and of most HDR
//! panoramas, whose centre is the view's front once turned).

use super::{CookError, Source};
use crate::format::{self, BlobKind};
use og_blob::BlobWriter;
use og_core::ogt::Document;
use std::path::{Path, PathBuf};

pub(super) const ENVIRONMENT_TYPE: &str = "environment";

const KEYS: [&str; 5] = ["source", "size", "intensity", "rotation", "samples"];

fn assets_root(s: &Source) -> PathBuf {
    let mut p = s.path.clone();
    for _ in s.rel.split('/') {
        p.pop();
    }
    p
}

/// An environment file's settings, read and checked.
pub(super) struct EnvironmentDoc {
    pub source: String,
    pub size: u32,
    pub intensity: f32,
    pub rotation: f32,
    pub samples: u32,
}

fn number(doc: &Document, key: &str, default: f32) -> Result<f32, String> {
    match doc.body.get(key) {
        None => Ok(default),
        Some(e) => match e.values.as_slice() {
            [v] => v
                .as_number()
                .map(|n| n.as_f32())
                .filter(|v| v.is_finite())
                .ok_or_else(|| format!("line {}: '{key}' takes one number", e.line)),
            _ => Err(format!("line {}: '{key}' takes one number", e.line)),
        },
    }
}

pub(super) fn parse(rel: &str, root: &Path, text: &str) -> Result<EnvironmentDoc, String> {
    let doc = Document::parse(text).map_err(|e| e.to_string())?;
    for e in &doc.body.entries {
        if !KEYS.contains(&e.key.as_str()) || e.block.is_some() {
            return Err(format!(
                "line {}: '{}' is not an environment entry: {}",
                e.line,
                e.key,
                KEYS.join(", ")
            ));
        }
    }
    let source = doc
        .body
        .get("source")
        .and_then(|e| e.values.first())
        .and_then(|v| v.as_str())
        .ok_or("an environment names its image: source \"<file>.hdr\" (or .exr)")?;
    let lower = source.to_ascii_lowercase();
    if !(lower.ends_with(".hdr") || lower.ends_with(".exr")) {
        return Err(format!(
            "source \"{source}\": an equirectangular Radiance .hdr or OpenEXR .exr"
        ));
    }
    let size = number(&doc, "size", 128.0)?;
    let (lo, hi) = (
        og_render::ENVIRONMENT_MAP_MIN_SIZE,
        og_render::ENVIRONMENT_MAP_MAX_SIZE,
    );
    if size.fract() != 0.0
        || !(lo as f32..=hi as f32).contains(&size)
        || !(size as u32).is_power_of_two()
    {
        return Err(format!("size {size}: a power of two from {lo} to {hi}"));
    }
    let intensity = number(&doc, "intensity", 1.0)?;
    if intensity < 0.0 {
        return Err(format!("intensity {intensity}: 0 or more"));
    }
    let samples = number(&doc, "samples", 256.0)?;
    if samples.fract() != 0.0 || !(16.0..=4096.0).contains(&samples) {
        return Err(format!("samples {samples}: a whole number from 16 to 4096"));
    }
    Ok(EnvironmentDoc {
        source: super::terrain::resolve(rel, root, source),
        size: size as u32,
        intensity,
        rotation: number(&doc, "rotation", 0.0)?,
        samples: samples as u32,
    })
}

/// The image an environment reads (by asset path), for the stamp and so the
/// walk does not list it.
pub(super) fn environment_files(rel: &str, root: &Path, text: &str) -> Vec<String> {
    parse(rel, root, text)
        .map(|d| vec![d.source])
        .unwrap_or_default()
}

/// The stamp's dependency: the image with its hash.
pub(super) fn environment_dependencies(s: &Source) -> String {
    let root = assets_root(s);
    environment_files(&s.rel, &root, &String::from_utf8_lossy(&s.bytes))
        .into_iter()
        .map(|f| {
            format!(
                "{f}={:016x}",
                std::fs::read(root.join(&f))
                    .map(|b| super::hash_bytes(&b))
                    .unwrap_or(0)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Cooks one `type environment` file.
pub(super) fn cook_environment(
    s: &Source,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let root = assets_root(s);
    let text = std::str::from_utf8(&s.bytes)
        .map_err(|_| CookError("an environment is text (UTF-8)".into()))?;
    let doc = parse(&s.rel, &root, text).map_err(CookError)?;
    let path = root.join(&doc.source);
    let bytes = std::fs::read(&path).map_err(|e| CookError(format!("{}: {e}", doc.source)))?;
    let img = image::load_from_memory(&bytes)
        .map_err(|e| CookError(format!("{}: {e}", doc.source)))?
        .to_rgb32f();
    let (width, height) = (img.width(), img.height());
    if width < 2 * height.saturating_sub(1) || width > 2 * height + 1 {
        return Err(CookError(format!(
            "{} is {width} × {height}: an equirectangular image is twice as wide as it is tall",
            doc.source
        )));
    }
    let rgb = img.into_raw();
    if let Some(i) = rgb.iter().position(|v| !v.is_finite()) {
        return Err(CookError(format!(
            "{}: pixel {} holds a value that is not a finite number",
            doc.source,
            i / 3
        )));
    }
    let map = og_render::bake::bake(
        &og_render::bake::Equirect {
            width,
            height,
            rgb: &rgb,
        },
        &og_render::bake::BakeDesc {
            size: doc.size,
            intensity: doc.intensity,
            rotation_degrees: doc.rotation,
            samples: doc.samples,
        },
    )
    .map_err(|e| CookError(format!("{}: {e}", doc.source)))?;
    let meta = format::EnvironmentMeta {
        size: map.size,
        levels: og_render::ENVIRONMENT_MAP_LEVELS,
        texel_count: map.texels.len() as u32,
        source_width: width,
        source_height: height,
        intensity: doc.intensity,
        rotation: doc.rotation,
        samples: doc.samples,
    };
    let sh: Vec<format::EnvironmentShRow> = map
        .irradiance
        .iter()
        .map(|[r, g, b]| format::EnvironmentShRow {
            r: *r,
            g: *g,
            b: *b,
        })
        .collect();
    let texels: Vec<u8> = map.texels.iter().flat_map(|t| t.to_le_bytes()).collect();
    let packed = miniz_oxide::deflate::compress_to_vec_zlib(&texels, 6);
    let mut w = BlobWriter::new(
        format::EnvironmentMap::type_hash(),
        format::EnvironmentMap::schema_version(),
        stamp,
    );
    w.table(
        "meta",
        &[meta],
        format::row_hash::<format::EnvironmentMeta>(),
    )
    .table(
        "irradiance",
        &sh,
        format::row_hash::<format::EnvironmentShRow>(),
    )
    .raw("texels", packed.clone(), 16);
    let note = format!(
        "environment: {} {width} × {height} into a {}² cube of {} levels (RGB9E5, {} KiB, {} KiB zlib'd), {} samples a texel",
        doc.source,
        map.size,
        og_render::ENVIRONMENT_MAP_LEVELS,
        texels.len() / 1024,
        packed.len() / 1024,
        doc.samples
    );
    Ok((w.finish(), vec![note]))
}
