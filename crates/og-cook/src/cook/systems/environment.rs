//! The environment map cook step: a typed content declaration names
//! an HDR or sRGB image, equirectangular or six faces, or a cube map, prefiltered
//! here, at cook time, into og-render's `EnvironmentMapData`
//! (`og_render_data::bake`): a cube of RGB9E5 levels, level `m` GGX-prefiltered
//! at roughness `m / 4` (level 0 the radiance), and the diffuse irradiance
//! as nine harmonics. The runtime only uploads it (`og.load.environment`).
//!
//! ```ts
//! import {file} from '@origingame/engine/author';
//! export default file.environment({source: {equirect: 'assets/studio.hdr'}, size: 128});
//! ```
//!
//! `sun extract` takes the image's sun out of the map
//! (`og_render_data::bake::BakeDesc::sun`) and stores its direction, colour
//! and intensity in the blob (`og.load.environmentSun`), for the game's
//! directional light: lit by the map alone a sun casts no shadow, and a
//! directional light added beside it counts the sun twice.
//!
//! `prefiltered true` takes a KTX2 cube map whose mip chain is already
//! GGX-prefiltered (cmgen's, IBLBaker's, the glTF IBL sampler's: level `k`
//! of `L` at roughness `k / (L - 1)`) as it is, instead of filtering its
//! radiance again (`og_render_data::bake::from_prefiltered`): the map's level `m`
//! (roughness `m / 4`) reads the file at lod `m / 4 · (L - 1)`, a chain of
//! five levels texel for texel; `size` is the file's face size (another is
//! refused) and `samples` does not apply.
//!
//! Images have explicit canonical paths under their module's assets/ or
//! sources/; they are part of the environment, not assets of their
//! own: the walk does not list them, and the environment's stamp covers
//! their bytes. An equirectangular image's centre column looks along +x
//! and its three-quarter column along +z, rows from +y down: the
//! equirectangular mapping of three.js (and of most HDR panoramas, whose
//! centre is the view's front once turned). A cube map's faces are
//! KTX2's (and WebGPU's): +x −x +y −y +z −z, each seen from the centre
//! with rows from the top; the +x face's right edge is towards −z. Either
//! is baked the same way (`bake::bake` or `bake::bake_cube`).

use super::{
    Action, CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, assets_root, stamp,
};
use crate::cook::source_schemas::world::source_data::{EnvironmentInput, Sun};
use crate::format::{self, Area, BlobKind};
use og_blob::BlobWriter;
use std::collections::BTreeMap;
use std::path::Path;

#[path = "environment/image.rs"]
mod prepare;

/// What an environment bakes from.
#[derive(serde::Deserialize, og_core::reflect::source::schemars::JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum EnvironmentSource {
    /// An equirectangular HDR or sRGB image.
    Equirect(#[schemars(length(min = 1), extend("x-og-input" = "bytes"))] String),
    /// A KTX2 cube map.
    Ktx2(#[schemars(length(min = 1), extend("x-og-input" = "bytes"))] String),
    /// Six face images, +x −x +y −y +z −z.
    Faces(#[schemars(inner(length(min = 1)), extend("x-og-input" = "bytes"))] [String; 6]),
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

fn environment_image(name: &str) -> bool {
    hdr_image(name)
        || [".png", ".jpg", ".jpeg", ".webp"]
            .iter()
            .any(|ext| name.to_ascii_lowercase().ends_with(ext))
}

/// Both ranges enter the same float prefilter. LDR texture bytes are sRGB,
/// not linear radiance; alpha has no meaning on a sky.
fn decode_image(name: &str, bytes: &[u8]) -> Result<image::Rgb32FImage, CookError> {
    if hdr_image(name) {
        return image::load_from_memory(bytes)
            .map(|img| img.to_rgb32f())
            .map_err(|e| CookError(format!("{name}: {e}")));
    }
    let img = super::decode_image(bytes).map_err(|e| CookError(format!("{name}: {e}")))?;
    let rgb = img
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|p| {
            [p[0], p[1], p[2]].map(|v| {
                let v = v as f32 / 255.0;
                if v <= 0.04045 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            })
        })
        .collect();
    Ok(image::Rgb32FImage::from_raw(img.width, img.height, rgb).expect("decoded dimensions"))
}

/// An environment file's settings, read and checked.
pub(crate) struct EnvironmentDoc {
    pub source: EnvironmentSource,
    /// The size given (`None`: 128, or a prefiltered file's own).
    pub size: Option<u32>,
    pub intensity: f32,
    pub rotation: f32,
    pub samples: u32,
    /// The KTX2's own mip chain is the prefiltered levels.
    pub prefiltered: bool,
    /// `sun extract`.
    pub sun: bool,
    pub upper: bool,
    pub ground: Option<glam::Vec3>,
    pub expand: f32,
    pub sun_boost: f32,
}

fn declaration(input: EnvironmentInput) -> Result<EnvironmentDoc, String> {
    let source = input.source;
    match &source {
        EnvironmentSource::Equirect(name) if !environment_image(name) => {
            return Err("equirect takes an HDR/EXR or sRGB PNG/JPG/WebP image".into());
        }
        EnvironmentSource::Ktx2(name) if !name.to_ascii_lowercase().ends_with(".ktx2") => {
            return Err("ktx2 takes a KTX2 cube map".into());
        }
        EnvironmentSource::Faces(names) => {
            if names.iter().any(|name| !environment_image(name)) {
                return Err("a face is HDR/EXR or sRGB PNG/JPG/WebP".into());
            }
            if names.iter().any(|name| hdr_image(name)) && names.iter().any(|name| !hdr_image(name))
            {
                return Err("faces must all be HDR/EXR or all be sRGB PNG/JPG/WebP".into());
            }
        }
        _ => {}
    }
    let prefiltered = input.prefiltered.unwrap_or(false);
    if prefiltered {
        if !matches!(source, EnvironmentSource::Ktx2(_)) {
            return Err("prefiltered takes a KTX2 cube map with a prefiltered mip chain".into());
        }
        if input.samples.is_some() {
            return Err(
                "samples is forbidden with prefiltered: the file supplies its levels".into(),
            );
        }
    }
    if matches!(input.sun, Some(Sun::Extract)) && prefiltered {
        return Err(
            "sun extract must precede prefiltering; a prefiltered chain is taken as it is".into(),
        );
    }
    if let Some(size) = input.size
        && !size.is_power_of_two()
    {
        return Err(format!("size {size}: expected a power of two"));
    }
    let upper = input.hemisphere.is_some();
    if input.ground.is_some() && !upper {
        return Err("ground takes hemisphere upper".into());
    }
    if upper && !matches!(&source, EnvironmentSource::Equirect(_)) {
        return Err("hemisphere takes an equirectangular source, not faces or KTX2".into());
    }
    let ldr = match &source {
        EnvironmentSource::Equirect(s) => !hdr_image(s),
        EnvironmentSource::Faces(f) => f.iter().all(|s| !hdr_image(s)),
        _ => false,
    };
    // LDR defaults to extracting the estimated disc, HDR keeps its prior contract.
    let sun = match input.sun {
        Some(Sun::Extract) => true,
        Some(Sun::Keep) => false,
        None => ldr,
    };
    Ok(EnvironmentDoc {
        source,
        size: input.size,
        intensity: input.intensity.unwrap_or(1.0),
        rotation: input.rotation.unwrap_or(0.0),
        samples: input.samples.unwrap_or(256),
        prefiltered,
        sun,
        upper,
        ground: input.ground.map(glam::Vec3::from_array),
        expand: input.expand.unwrap_or(0.25),
        sun_boost: input.sun_boost.unwrap_or(1.0),
    })
}

/// Images already owned by an environment are not standalone scan assets.
/// Only the base assets directory is scanned; module inputs are explicit.
pub(crate) fn environment_files(s: &Source, assets: &Path) -> Vec<String> {
    let root = assets_root(s);
    s.input::<EnvironmentInput>()
        .map(|input| input.source.files())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|file| {
            crate::cook::source_path::checked(&root, &file)
                .ok()?
                .strip_prefix(assets)
                .ok()
                .map(|file| file.to_string_lossy().replace('\\', "/"))
        })
        .collect()
}

/// Both LDR layouts share solar estimation, range expansion and the bake.
/// With `sun extract`, `no_sun` is set to why the LDR image gave no sun.
fn bake_image(
    mut img: image::Rgb32FImage,
    source_width: u32,
    ldr: bool,
    doc: &EnvironmentDoc,
    desc: &og_render_data::bake::BakeDesc,
    no_sun: &mut Option<String>,
) -> Result<og_render_data::EnvironmentMapData, CookError> {
    let (width, height) = img.dimensions();
    let background = ldr.then(|| img.clone());
    let mut sun = Default::default();
    if ldr {
        if doc.sun {
            match prepare::take_sun(&mut img, doc.sun_boost) {
                Ok(s) => sun = s,
                Err(why) => *no_sun = Some(why),
            }
        }
        prepare::expand(img.as_mut(), doc.expand);
    }
    let bake = |img: &image::Rgb32FImage, settings| {
        og_render_data::bake::bake(
            &og_render_data::bake::Equirect {
                width,
                height,
                rgb: img.as_raw(),
            },
            &settings,
        )
        .map_err(CookError)
    };
    let mut map = bake(
        &img,
        og_render_data::bake::BakeDesc {
            sun: doc.sun && !ldr,
            ..*desc
        },
    )?;
    if ldr {
        sun.direction = (glam::Quat::from_rotation_y(doc.rotation.to_radians())
            * glam::Vec3::from_array(sun.direction))
        .to_array();
        sun.intensity *= doc.intensity;
        map.sun = sun;
    }
    if let Some(img) = background {
        // Use the original width, not an odd panorama's rounded 2:1 width.
        let upper = source_width.div_ceil(4).clamp(1, 2048).next_power_of_two();
        map.background_size = if upper > 1 && source_width < 3 * upper {
            upper / 2
        } else {
            upper
        };
        map.background = og_render_data::bake::background(
            &og_render_data::bake::Equirect {
                width,
                height,
                rgb: img.as_raw(),
            },
            map.background_size,
            doc.rotation,
        )
        .map_err(CookError)?;
    }
    Ok(map)
}

/// Cooks the checked declaration and the exact images captured for its stamp.
fn cook_environment(
    doc: &EnvironmentDoc,
    images: &BTreeMap<String, Vec<u8>>,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let read = |f: &str| images[f].as_slice();
    let finite = |name: &str, rgb: &[f32]| match rgb.iter().position(|v| !v.is_finite()) {
        Some(i) => Err(CookError(format!(
            "{name}: pixel {} holds a value that is not a finite number",
            i / 3
        ))),
        None => Ok(()),
    };
    let mut desc = og_render_data::bake::BakeDesc {
        size: doc.size.unwrap_or(128),
        intensity: doc.intensity,
        rotation_degrees: doc.rotation,
        samples: doc.samples,
        sun: doc.sun,
    };
    let mut facts = Vec::new();
    let mut diagnostics = serde_json::Value::Null;
    let mut no_sun = None;
    let (map, width, height, what) = match &doc.source {
        EnvironmentSource::Equirect(source) => {
            let img = decode_image(source, read(source))?;
            finite(source, img.as_raw())?;
            diagnostics = prepare::facts(&img, doc.upper);
            facts.push(format!("environment facts: {diagnostics}"));
            let ldr = !hdr_image(source);
            let source_width = img.width();
            let img = prepare::panorama(source, img, doc)?;
            let (width, height) = img.dimensions();
            let map = bake_image(img, source_width, ldr, doc, &desc, &mut no_sun)?;
            (map, width, height, format!("{source} {width} × {height}"))
        }
        EnvironmentSource::Ktx2(source) if doc.prefiltered => {
            let cube = super::cube_source::read_ktx2_cube_levels(read(source))
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
            let map = og_render_data::bake::from_prefiltered(
                &og_render_data::bake::PrefilteredCube {
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
            let cube = super::cube_source::read_ktx2_cube(read(source))
                .map_err(|e| CookError(format!("{source}: {e}")))?;
            let map = og_render_data::bake::bake_cube(
                &og_render_data::bake::CubeFaces {
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
                let img = decode_image(face, read(face))?;
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
            let panorama = prepare::cube_panorama(size, &rgb);
            diagnostics = prepare::facts(&panorama, false);
            facts.push(format!(
                "environment facts (faces projected to panorama): {diagnostics}"
            ));
            let map = if faces.iter().all(|f| !hdr_image(f)) {
                bake_image(panorama, 4 * size, true, doc, &desc, &mut no_sun)?
            } else {
                og_render_data::bake::bake_cube(
                    &og_render_data::bake::CubeFaces { size, rgb: &rgb },
                    &desc,
                )
                .map_err(|e| CookError(format!("{}: {e}", faces[0])))?
            };
            (
                map,
                size,
                size,
                format!("six faces of {size}² ({} …)", faces[0]),
            )
        }
    };
    let samples = if doc.prefiltered { 0 } else { doc.samples };
    // Why `sun extract` found no sun, in the report (environment.facts and
    // environment.sun) and the cook's message: an HDR image's sun is every
    // texel within 5° of the brightest over sixteen times the rest's mean.
    let no_sun = (doc.sun && map.sun.intensity <= 0.0).then(|| {
        no_sun.unwrap_or_else(|| {
            "none bright enough: no texel near the brightest stands over sixteen times the mean of the rest".into()
        })
    });
    if let (Some(why), serde_json::Value::Object(facts)) = (&no_sun, &mut diagnostics) {
        facts.insert("sunMissing".into(), serde_json::json!(why));
    }
    let (blob, texels, packed) = environment_blob(
        &map,
        (width, height),
        doc.intensity,
        doc.rotation,
        samples,
        stamp,
        Some(&diagnostics),
    );
    let sun = match (doc.sun, map.sun) {
        (false, _) => String::new(),
        (true, s) if s.intensity > 0.0 => format!(
            "; sun extract: intensity {:.3}, colour {:.2} {:.2} {:.2}, towards {:.3} {:.3} {:.3}",
            s.intensity,
            s.color[0],
            s.color[1],
            s.color[2],
            s.direction[0],
            s.direction[1],
            s.direction[2]
        ),
        (true, _) => format!(
            "; sun extract: no measurable sun was found ({}), so the image is kept whole",
            no_sun.as_deref().unwrap_or("none bright enough")
        ),
    };
    let note = format!(
        "environment: {what} into a {}² cube of {} levels (RGB9E5, {} KiB, {} KiB zlib'd){}{sun}",
        map.size,
        og_render_data::ENVIRONMENT_MAP_LEVELS,
        texels / 1024,
        packed / 1024,
        if doc.prefiltered {
            String::new()
        } else {
            format!(", {} samples a texel", doc.samples)
        }
    );
    facts.push(format!(
        "environment sun: direction {:?}, color {:?}, intensity {:.6}, angularSize {:.6} rad{}",
        map.sun.direction,
        map.sun.color,
        map.sun.intensity,
        map.sun.angular_size,
        if !map.background.is_empty() {
            " (LDR relative-radiance estimate, not measured lux)"
        } else {
            ""
        }
    ));
    facts.insert(0, note);
    Ok((blob, facts))
}

/// An environment blob of `map`: its meta (the source's size, the
/// cook's intensity, turn and samples; 0 samples: levels taken as given),
/// irradiance and zlib'd RGB9E5 texels; with the texels' raw and packed bytes.
pub(crate) fn environment_blob(
    map: &og_render_data::EnvironmentMapData,
    (width, height): (u32, u32),
    intensity: f32,
    rotation: f32,
    samples: u32,
    stamp: u64,
    diagnostics: Option<&serde_json::Value>,
) -> (Vec<u8>, usize, usize) {
    let meta = format::EnvironmentMeta {
        size: map.size,
        levels: og_render_data::ENVIRONMENT_MAP_LEVELS,
        texel_count: map.texels.len() as u32,
        source_width: width,
        source_height: height,
        intensity,
        rotation,
        samples,
        sun_x: map.sun.direction[0],
        sun_y: map.sun.direction[1],
        sun_z: map.sun.direction[2],
        sun_r: map.sun.color[0],
        sun_g: map.sun.color[1],
        sun_b: map.sun.color[2],
        sun_intensity: map.sun.intensity,
        sun_angular_size: map.sun.angular_size,
        background_size: map.background_size,
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
    if let Some(facts) = diagnostics {
        w.raw("diagnostics", facts.to_string().into_bytes(), 1);
    }
    if !map.background.is_empty() {
        let bytes: Vec<u8> = map
            .background
            .iter()
            .flat_map(|t| t.to_le_bytes())
            .collect();
        w.raw(
            "background",
            miniz_oxide::deflate::compress_to_vec_zlib(&bytes, 6),
            16,
        );
    }
    (w.finish(), raw, zipped)
}

pub(crate) static ENVIRONMENT: KindDef = KindDef {
    name: "environment",
    extensions: &[],
    types: &[],
    by_content: true,
    streams: true,
    plan: plan_environment,
};

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&ENVIRONMENT];

fn plan_environment<'a>(
    _cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let prepared = (|| -> Result<_, String> {
        let doc = declaration(s.input::<EnvironmentInput>()?)?;
        let root = assets_root(s);
        let mut images = BTreeMap::new();
        for file in doc.source.files() {
            if !(file.starts_with("assets/") || file.starts_with("sources/")) {
                return Err(
                    "environment inputs belong in the owning module's assets/ or sources/".into(),
                );
            }
            if images.contains_key(&file) {
                continue;
            }
            let path = crate::cook::source_path::checked(&root, &file)
                .map_err(|e| format!("{file}: {e}"))?;
            let bytes = std::fs::read(path).map_err(|e| format!("{file}: {e}"))?;
            images.insert(file, bytes);
        }
        Ok((doc, images))
    })();
    let (doc, images) = match prepared {
        Ok(prepared) => prepared,
        Err(error) => {
            state
                .early_errors
                .push((s.rel.clone(), "environment", error));
            return;
        }
    };
    let mut parts = vec![b"environment typed 1".as_slice(), &hash];
    parts.extend(images.values().map(Vec::as_slice));
    let st = stamp(&[Area::Lighting], &parts);
    outs.push((
        s.rel.clone(),
        format::EnvironmentMap::NAME,
        st,
        Box::new(EnvironmentJob {
            source: s,
            doc,
            images,
        }),
    ));
}

/// An environment's one output: its maps.
struct EnvironmentJob<'a> {
    source: &'a Source,
    doc: EnvironmentDoc,
    images: BTreeMap<String, Vec<u8>>,
}

impl Job for EnvironmentJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        cook_environment(&self.doc, &self.images, p.stamp)
    }
    fn action(&self) -> Action<'_> {
        Action::Input(&self.source.bytes, false)
    }
}
