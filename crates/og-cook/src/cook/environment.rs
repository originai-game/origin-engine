//! The environment map cook step (phase 4): a `type environment` .ogt names
//! an HDR image, an equirectangular one or a cube map, which is prefiltered
//! here, at cook time, into og-render's `EnvironmentMapData`
//! (`og_render::bake`): a cube of RGB9E5 levels, level `m` GGX-prefiltered
//! at roughness `m / 4` (level 0 the radiance), and the diffuse irradiance
//! as nine harmonics. The runtime only uploads it (`og.load.environment`).
//!
//! ```text
//! type environment
//! version 1
//! ---
//! source "studio.hdr"   # an equirectangular Radiance .hdr or OpenEXR .exr (linear RGB),
//!                       # or a KTX2 cube map (RGBA16F, RGB9E5, BC6H, ...: `cube_source`)
//! faces "px.hdr" "nx.hdr" "py.hdr" "ny.hdr" "pz.hdr" "nz.hdr"
//!                       # instead of source: six square faces, +x -x +y -y +z -z
//! size 128              # level 0's face size: a power of two from 16 to 2048
//! intensity 1           # multiplies the image
//! rotation 0            # turns it about +y, degrees (what was at +x moves towards -z)
//! samples 256           # GGX samples per texel of each prefiltered level (16 to 4096)
//! prefiltered false     # true: a KTX2 cube's own mip chain is the prefiltered levels
//! ```
//!
//! `prefiltered true` takes a KTX2 cube map whose mip chain is already
//! GGX-prefiltered (cmgen's, IBLBaker's, the glTF IBL sampler's: level `k`
//! of `L` at roughness `k / (L - 1)`) as it is, instead of filtering its
//! radiance again (`og_render::bake::from_prefiltered`): the map's level `m`
//! (roughness `m / 4`) reads the file at lod `m / 4 · (L - 1)`, a chain of
//! five levels texel for texel; `size` is the file's face size (another is
//! refused) and `samples` does not apply.
//!
//! The images are read beside the environment file (or, failing that,
//! under `assets/`); they are part of the environment, not assets of their
//! own: the walk does not list them, and the environment's stamp covers
//! their bytes. An equirectangular image's centre column looks along +x
//! and its three-quarter column along +z, rows from +y down: the
//! equirectangular mapping of three.js (and of most HDR panoramas, whose
//! centre is the view's front once turned). A cube map's faces are
//! KTX2's (and WebGPU's): +x −x +y −y +z −z, each seen from the centre
//! with rows from the top; the +x face's right edge is towards −z. Either
//! is baked the same way (`bake::bake` or `bake::bake_cube`).

use super::{CookError, Source};
use crate::format::{self, BlobKind};
use og_blob::BlobWriter;
use og_core::ogt::Document;
use std::path::{Path, PathBuf};

pub(super) const ENVIRONMENT_TYPE: &str = "environment";

const KEYS: [&str; 7] = [
    "source",
    "faces",
    "size",
    "intensity",
    "rotation",
    "samples",
    "prefiltered",
];

/// What an environment bakes from.
pub(super) enum EnvironmentSource {
    /// An equirectangular `.hdr` or `.exr`.
    Equirect(String),
    /// A KTX2 cube map.
    Ktx2(String),
    /// Six face images, +x −x +y −y +z −z.
    Faces([String; 6]),
}

impl EnvironmentSource {
    fn files(&self) -> Vec<String> {
        match self {
            EnvironmentSource::Equirect(f) | EnvironmentSource::Ktx2(f) => vec![f.clone()],
            EnvironmentSource::Faces(f) => f.to_vec(),
        }
    }
}

fn hdr_image(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".hdr") || lower.ends_with(".exr")
}

fn assets_root(s: &Source) -> PathBuf {
    let mut p = s.path.clone();
    for _ in s.rel.split('/') {
        p.pop();
    }
    p
}

/// An environment file's settings, read and checked.
pub(super) struct EnvironmentDoc {
    pub source: EnvironmentSource,
    /// The size given (`None`: 128, or a prefiltered file's own).
    pub size: Option<u32>,
    pub intensity: f32,
    pub rotation: f32,
    pub samples: u32,
    /// The KTX2's own mip chain is the prefiltered levels.
    pub prefiltered: bool,
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
    let source = match (doc.body.get("source"), doc.body.get("faces")) {
        (Some(_), Some(f)) => {
            return Err(format!(
                "line {}: an environment has a source or six faces, not both",
                f.line
            ));
        }
        (None, Some(f)) => {
            let names: Vec<&str> = f.values.iter().filter_map(|v| v.as_str()).collect();
            if names.len() != 6 || f.values.len() != 6 {
                return Err(format!(
                    "line {}: faces takes six images, +x -x +y -y +z -z: faces \"px.hdr\" \"nx.hdr\" \"py.hdr\" \"ny.hdr\" \"pz.hdr\" \"nz.hdr\"",
                    f.line
                ));
            }
            if let Some(bad) = names.iter().find(|n| !hdr_image(n)) {
                return Err(format!(
                    "line {}: face \"{bad}\": a face is a Radiance .hdr or OpenEXR .exr",
                    f.line
                ));
            }
            EnvironmentSource::Faces(std::array::from_fn(|i| {
                super::terrain::resolve(rel, root, names[i])
            }))
        }
        (Some(e), None) => {
            let source = e.values.first().and_then(|v| v.as_str()).ok_or_else(|| {
                format!(
                    "line {}: source \"<file>.hdr\" (or .exr, or a .ktx2 cube map)",
                    e.line
                )
            })?;
            if source.to_ascii_lowercase().ends_with(".ktx2") {
                EnvironmentSource::Ktx2(super::terrain::resolve(rel, root, source))
            } else if hdr_image(source) {
                EnvironmentSource::Equirect(super::terrain::resolve(rel, root, source))
            } else {
                return Err(format!(
                    "source \"{source}\": an equirectangular Radiance .hdr or OpenEXR .exr, or a KTX2 cube map"
                ));
            }
        }
        (None, None) => {
            return Err(
                "an environment names its image: source \"<file>.hdr\" (or .exr, or a .ktx2 cube map), or faces with six images"
                    .into(),
            );
        }
    };
    let prefiltered = match doc.body.get("prefiltered") {
        None => false,
        Some(e) => match e.values.as_slice() {
            [v] => v
                .as_bool()
                .ok_or_else(|| format!("line {}: 'prefiltered' takes true or false", e.line))?,
            _ => {
                return Err(format!(
                    "line {}: 'prefiltered' takes true or false",
                    e.line
                ));
            }
        },
    };
    if prefiltered {
        if !matches!(source, EnvironmentSource::Ktx2(_)) {
            return Err(
                "prefiltered true takes a KTX2 cube map whose mip chain is already prefiltered: source \"<file>.ktx2\""
                    .into(),
            );
        }
        if let Some(e) = doc.body.get("samples") {
            return Err(format!(
                "line {}: samples is the cook's prefilter; prefiltered true takes the file's levels as they are",
                e.line
            ));
        }
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
        source,
        size: doc.body.get("size").map(|_| size as u32),
        intensity,
        rotation: number(&doc, "rotation", 0.0)?,
        samples: samples as u32,
        prefiltered,
    })
}

/// The images an environment reads (by asset path), for the stamp and so
/// the walk does not list them.
pub(super) fn environment_files(rel: &str, root: &Path, text: &str) -> Vec<String> {
    parse(rel, root, text)
        .map(|d| d.source.files())
        .unwrap_or_default()
}

/// The stamp's dependency: the images with their hashes.
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
    let read = |f: &str| std::fs::read(root.join(f)).map_err(|e| CookError(format!("{f}: {e}")));
    let finite = |name: &str, rgb: &[f32]| match rgb.iter().position(|v| !v.is_finite()) {
        Some(i) => Err(CookError(format!(
            "{name}: pixel {} holds a value that is not a finite number",
            i / 3
        ))),
        None => Ok(()),
    };
    let mut desc = og_render::bake::BakeDesc {
        size: doc.size.unwrap_or(128),
        intensity: doc.intensity,
        rotation_degrees: doc.rotation,
        samples: doc.samples,
    };
    let (map, width, height, what) = match &doc.source {
        EnvironmentSource::Equirect(source) => {
            let img = image::load_from_memory(&read(source)?)
                .map_err(|e| CookError(format!("{source}: {e}")))?
                .to_rgb32f();
            let (width, height) = (img.width(), img.height());
            if width < 2 * height.saturating_sub(1) || width > 2 * height + 1 {
                return Err(CookError(format!(
                    "{source} is {width} × {height}: an equirectangular image is twice as wide as it is tall"
                )));
            }
            let rgb = img.into_raw();
            finite(source, &rgb)?;
            let map = og_render::bake::bake(
                &og_render::bake::Equirect {
                    width,
                    height,
                    rgb: &rgb,
                },
                &desc,
            )
            .map_err(|e| CookError(format!("{source}: {e}")))?;
            (map, width, height, format!("{source} {width} × {height}"))
        }
        EnvironmentSource::Ktx2(source) if doc.prefiltered => {
            let cube = super::cube_source::read_ktx2_cube_levels(&read(source)?)
                .map_err(|e| CookError(format!("{source}: {e}")))?;
            let n = cube.size;
            if cube.levels.len() < 2 {
                return Err(CookError(format!(
                    "{source} has one level: prefiltered true takes the file's own mip chain (at least two levels); without it the cook prefilters level 0"
                )));
            }
            if let Some(size) = doc.size
                && size != n
            {
                return Err(CookError(format!(
                    "size {size}: {source}'s faces are {n}², and a prefiltered chain is taken at its own size (leave size out)"
                )));
            }
            desc.size = n;
            let map = og_render::bake::from_prefiltered(
                &og_render::bake::PrefilteredCube {
                    size: n,
                    levels: &cube.levels,
                },
                &desc,
            )
            .map_err(|e| CookError(format!("{source}: {e}")))?;
            (
                map,
                n,
                n,
                format!(
                    "{source} (a {} cube map of 6 × {n}², its {} prefiltered levels taken as they are)",
                    cube.format,
                    cube.levels.len()
                ),
            )
        }
        EnvironmentSource::Ktx2(source) => {
            let cube = super::cube_source::read_ktx2_cube(&read(source)?)
                .map_err(|e| CookError(format!("{source}: {e}")))?;
            let map = og_render::bake::bake_cube(
                &og_render::bake::CubeFaces {
                    size: cube.size,
                    rgb: &cube.rgb,
                },
                &desc,
            )
            .map_err(|e| CookError(format!("{source}: {e}")))?;
            let n = cube.size;
            (
                map,
                n,
                n,
                format!("{source} (a {} cube map of 6 × {n}²)", cube.format),
            )
        }
        EnvironmentSource::Faces(faces) => {
            let mut rgb = Vec::new();
            let mut size = 0;
            for (i, face) in faces.iter().enumerate() {
                let img = image::load_from_memory(&read(face)?)
                    .map_err(|e| CookError(format!("{face}: {e}")))?
                    .to_rgb32f();
                let (w, h) = (img.width(), img.height());
                if w != h {
                    return Err(CookError(format!(
                        "{face} is {w} × {h}: a cube map's faces are square"
                    )));
                }
                if i == 0 {
                    size = w;
                } else if w != size {
                    return Err(CookError(format!(
                        "{face} is {w} × {w} and {} is {size} × {size}: a cube map's faces are one size",
                        faces[0]
                    )));
                }
                let face_rgb = img.into_raw();
                finite(face, &face_rgb)?;
                rgb.extend(face_rgb.into_iter().map(|v| v.max(0.0)));
            }
            let map =
                og_render::bake::bake_cube(&og_render::bake::CubeFaces { size, rgb: &rgb }, &desc)
                    .map_err(|e| CookError(format!("{}: {e}", faces[0])))?;
            (
                map,
                size,
                size,
                format!("six faces of {size}² ({} …)", faces[0]),
            )
        }
    };
    let samples = if doc.prefiltered { 0 } else { doc.samples };
    let (blob, texels, packed) = environment_blob(
        &map,
        (width, height),
        doc.intensity,
        doc.rotation,
        samples,
        stamp,
    );
    let note = format!(
        "environment: {what} into a {}² cube of {} levels (RGB9E5, {} KiB, {} KiB zlib'd){}",
        map.size,
        og_render::ENVIRONMENT_MAP_LEVELS,
        texels / 1024,
        packed / 1024,
        if doc.prefiltered {
            String::new()
        } else {
            format!(", {} samples a texel", doc.samples)
        }
    );
    Ok((blob, vec![note]))
}

/// A `type environment` blob of `map`: its meta (the source's size, the
/// cook's intensity, turn and samples; 0 samples: levels taken as given),
/// irradiance and zlib'd RGB9E5 texels; with the texels' raw and packed bytes.
pub(super) fn environment_blob(
    map: &og_render::EnvironmentMapData,
    (width, height): (u32, u32),
    intensity: f32,
    rotation: f32,
    samples: u32,
    stamp: u64,
) -> (Vec<u8>, usize, usize) {
    let meta = format::EnvironmentMeta {
        size: map.size,
        levels: og_render::ENVIRONMENT_MAP_LEVELS,
        texel_count: map.texels.len() as u32,
        source_width: width,
        source_height: height,
        intensity,
        rotation,
        samples,
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
    let (raw, zipped) = (texels.len(), packed.len());
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
    .raw("texels", packed, 16);
    (w.finish(), raw, zipped)
}
