//! A material library's maps from images an image model drew (`og
//! materials brief` and `og materials make`). Authoring time only: the
//! engine writes the requests and processes what comes back; generating
//! the images is the platform's (docs/platform-asks.md, ask 32).
//!
//! A library's material that states a `brief` (what its maps show) and
//! names its `map` is one request. The image the platform draws for it
//! lands beside the map as `<map>.source.png`; `make` turns it into the
//! map itself, periodic across (and up, unless it is a `facade` module,
//! which repeats along a street but stands on the ground), and, with a
//! `relief`, into the `normalMap` too. Nothing here judges an image: the
//! report gives the seam before and after against the step between
//! neighbours inside, the size and the texels per metre.

mod maps;
mod periodic;

#[cfg(test)]
mod tests;

use super::systems::district::parse;

/// One image a library asks for.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Request {
    pub material: String,
    /// Also make these maps as virtual cook inputs.
    pub generate: bool,
    /// The art brief kind: `material` (a seamless surface) or `facade` (a
    /// module that repeats along a street).
    pub kind: &'static str,
    pub subject: String,
    /// The metres the image covers, across and up.
    pub tile: [f64; 2],
    /// Where the drawn image goes, and the maps made from it (asset ids).
    pub source: String,
    pub map: String,
    pub normal_map: Option<String>,
    pub relief: f64,
    pub roughness_map: Option<String>,
    pub roughness_range: Option<[f64; 2]>,
    pub ao_map: Option<String>,
    pub occlusion: Option<[f64; 2]>,
}

/// The requests of a library's text.
pub fn requests(text: &str) -> Result<Vec<Request>, String> {
    let mut out = Vec::new();
    for m in parse::materials(text)? {
        if m.brief.is_none() && m.generate.is_none() {
            continue;
        }
        let subject = m.brief.unwrap_or_default();
        let named = |k: &str| -> Result<Option<String>, String> {
            match m.fields.get(k) {
                None => Ok(None),
                Some(e) => e
                    .string(0)
                    .map(|s| Some(crate::runtime::asset_id(s).to_string()))
                    .map_err(|e| format!("material `{}`: {k}: {e}", m.name)),
            }
        };
        let map = named("map")?.ok_or_else(|| {
            format!(
                "material `{}` states a brief but names no `map` for it to make",
                m.name
            )
        })?;
        let normal_map = named("normalMap")?;
        if m.relief != 0.0 && normal_map.is_none() {
            return Err(format!(
                "material `{}` states a relief but names no `normalMap` for it",
                m.name
            ));
        }
        let Some(stem) = map.strip_suffix(".png") else {
            return Err(format!(
                "material `{}`: the map `{map}` it makes is a .png",
                m.name
            ));
        };
        let roughness_map = named("metalnessMap")?;
        let ao_map = named("aoMap")?;
        if m.roughness_range.is_some() && roughness_map.is_none() {
            return Err(format!(
                "material `{}`: roughnessRange needs a metalnessMap destination",
                m.name
            ));
        }
        if m.occlusion.is_some() && (ao_map.is_none() || m.relief == 0.0) {
            return Err(format!(
                "material `{}`: occlusion needs an aoMap destination and nonzero relief",
                m.name
            ));
        }
        out.push(Request {
            material: m.name,
            generate: m.generate.is_some(),
            kind: if m.facade { "facade" } else { "material" },
            subject,
            tile: m.tile,
            source: m.generate.unwrap_or_else(|| format!("{stem}.source.png")),
            map,
            normal_map: if m.relief != 0.0 { normal_map } else { None },
            relief: m.relief,
            roughness_map: if m.roughness_range.is_some() {
                roughness_map
            } else {
                None
            },
            roughness_range: m.roughness_range,
            ao_map: if m.occlusion.is_some() { ao_map } else { None },
            occlusion: m.occlusion,
        });
    }
    // All destinations stay under assets/ and are distinct, including
    // sources: one request must never overwrite another request's input.
    let mut paths = std::collections::BTreeSet::new();
    for r in &out {
        for id in std::iter::once(&r.source).chain(r.destinations()) {
            if id.contains(['\\', ':'])
                || (id != &r.source && !id.ends_with(".png"))
                || id.split('/').any(|part| matches!(part, "" | "." | ".."))
            {
                return Err(format!(
                    "material `{}`: `{id}` must be a .png under assets/",
                    r.material
                ));
            }
            if !paths.insert(id) {
                return Err(format!("material `{}`: `{id}` is used twice", r.material));
            }
        }
    }
    Ok(out)
}

impl Request {
    pub fn destinations(&self) -> impl Iterator<Item = &String> {
        std::iter::once(&self.map)
            .chain(&self.normal_map)
            .chain(&self.roughness_map)
            .chain(&self.ao_map)
    }
}

/// A content address for authoring inputs or output integrity.
pub fn hash(bytes: &[u8]) -> String {
    ring::digest::digest(&ring::digest::SHA256, bytes)
        .as_ref()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// A versioned key for the source bytes and every processing option.
pub fn cache_key(r: &Request, source: &[u8]) -> String {
    let mut bytes = b"og-materials-v1\0".to_vec();
    bytes.extend(serde_json::to_vec(r).expect("finite request fields"));
    bytes.extend(source);
    hash(&bytes)
}

/// What `make` did with one image.
#[derive(Clone, Debug, PartialEq)]
pub struct Made {
    /// The maps' size: the image's sides, each down to a power of two.
    pub width: u32,
    pub height: u32,
    /// The mean step, 0 to 1, between the edges the map wraps over.
    pub seam_before: f64,
    pub seam_after: f64,
    /// The mean step between neighbours inside the map: a seam near it
    /// does not show.
    pub interior: f64,
    /// Texels per metre, across and up.
    pub density: [f64; 2],
    /// The files written, by asset id, with their bytes (PNG).
    pub files: Vec<(String, Vec<u8>)>,
}

/// The maps of `r` from the image drawn for it (PNG, JPEG or WebP).
pub fn make(r: &Request, source: &[u8]) -> Result<Made, String> {
    let img = super::decode_image(source).map_err(|e| format!("{}: {e}", r.source))?;
    if img.width < 8 || img.height < 8 {
        return Err(format!(
            "{} is {} x {}: a map is made from an image 8 texels or more on each side",
            r.source, img.width, img.height
        ));
    }
    // Generators answer near the size asked, not at it: each side is
    // resampled down to its power of two, as the cook's mips want.
    let down = |n: u32| 1u32 << (31 - n.leading_zeros());
    let (w, h) = (down(img.width), down(img.height));
    let rgba = if (w, h) == (img.width, img.height) {
        img.rgba
    } else {
        let full = image::RgbaImage::from_raw(img.width, img.height, img.rgba)
            .ok_or_else(|| format!("{}: a short image", r.source))?;
        image::imageops::resize(&full, w, h, image::imageops::FilterType::Lanczos3).into_raw()
    };
    let (w, h) = (w as usize, h as usize);
    let wrap_y = r.kind != "facade";
    let mut rgb = maps::Rgb::from_rgba8(w, h, &rgba);
    let seam_before = rgb.seam(wrap_y);
    rgb.periodic(wrap_y);
    let seam_after = rgb.seam(wrap_y);
    let interior = rgb.interior(wrap_y);
    let mut files = vec![(r.map.clone(), png(w, h, &rgb.to_rgb8())?)];
    let lum = rgb.luminance();
    if let Some(n) = &r.normal_map {
        let normal = maps::normal_map(&lum, w, h, r.tile, r.relief, wrap_y);
        files.push((n.clone(), png(w, h, &normal)?));
    }
    if let (Some(id), Some([black, white])) = (&r.roughness_map, r.roughness_range) {
        let packed: Vec<u8> = lum
            .iter()
            .flat_map(|l| {
                [
                    255,
                    ((black + (white - black) * l) * 255.0).round() as u8,
                    255,
                ]
            })
            .collect();
        files.push((id.clone(), png(w, h, &packed)?));
    }
    if let (Some(id), Some(options)) = (&r.ao_map, r.occlusion) {
        let ao = maps::occlusion_map(&lum, w, h, r.relief, options, wrap_y);
        files.push((id.clone(), png(w, h, &ao)?));
    }
    Ok(Made {
        width: w as u32,
        height: h as u32,
        seam_before,
        seam_after,
        interior,
        density: [w as f64 / r.tile[0], h as f64 / r.tile[1]],
        files,
    })
}

fn png(w: usize, h: usize, rgb: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    image::ImageEncoder::write_image(
        image::codecs::png::PngEncoder::new(&mut out),
        rgb,
        w as u32,
        h as u32,
        image::ExtendedColorType::Rgb8,
    )
    .map_err(|e| e.to_string())?;
    Ok(out)
}

/// Map generation is a cook input node, never a write into the author's assets.
pub(crate) fn expand(sources: &mut Vec<super::Source>) {
    use super::{Source, assets_root, cache, systems::district, texture};
    let mut occupied: std::collections::BTreeSet<_> =
        sources.iter().map(|s| s.rel.clone()).collect();
    let mut extra = Vec::new();
    for s in sources
        .iter_mut()
        .filter(|s| s.kind == &district::MATERIALS && s.error.is_none())
    {
        let root = assets_root(s);
        let made = (|| {
            let requests = requests(&String::from_utf8_lossy(&s.bytes))?;
            let mut files = Vec::new();
            for r in requests.into_iter().filter(|r| r.generate) {
                let source = std::fs::read(root.join(&r.source))
                    .map_err(|e| format!("{}: {e}", r.source))?;
                let options = serde_json::to_vec(&r).map_err(|e| e.to_string())?;
                let key = cache::key(
                    &[crate::format::Area::Texture],
                    "material-maps-1",
                    &[&options, &source],
                );
                let store = cache::default_path().map(cache::Store::new);
                let cached = store
                    .as_ref()
                    .and_then(|s| s.action(&key))
                    .and_then(|(bytes, _)| {
                        serde_json::from_slice::<Vec<(String, Vec<u8>)>>(&bytes).ok()
                    });
                let maps = match cached {
                    Some(maps) => maps,
                    None => {
                        let maps = make(&r, &source)?.files;
                        if let Some(store) = &store {
                            let bytes = serde_json::to_vec(&maps).map_err(|e| e.to_string())?;
                            let _ = store.put_action(&key, &bytes, &[]);
                        }
                        maps
                    }
                };
                files.extend(maps);
            }
            for (id, _) in &files {
                if occupied.contains(id) || root.join(id).exists() {
                    return Err(format!(
                        "generated material map '{id}' collides with an authored source or another output"
                    ));
                }
            }
            Ok::<_, String>(files)
        })();
        match made {
            Err(error) => s.error = Some(error),
            Ok(files) => {
                for (rel, bytes) in files {
                    occupied.insert(rel.clone());
                    extra.push(Source {
                        path: root.join(&rel),
                        rel,
                        kind: &texture::TEXTURE,
                        bytes,
                        settings: None,
                        settings_files: Vec::new(),
                        hash: 0,
                        gltf: None,
                        error: None,
                        encoded: Default::default(),
                        whole: Default::default(),
                        model: Default::default(),
                        video: None,
                        timeline: None,
                        fields: Default::default(),
                        conformed: None,
                        fracture: None,
                    });
                }
            }
        }
    }
    sources.extend(extra);
}
