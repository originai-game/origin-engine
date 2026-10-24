//! The terrain cook step (phase 2; tiles for streaming in phase 3): a
//! `type terrain` .ogt names its heightmap, the heights its black and
//! white stand for, its size, an optional control map and up to four
//! layered materials. The field is cooked into og-render's tile pyramid
//! (`og_render::TerrainLayout`: level k every 2^k-th sample, tiles of
//! `tile` quads): each tile's `(tile + 3)²` heights quantised to 16 bits
//! over the terrain's whole range (so a sample has the same value in every
//! tile and level that holds it) and zlib'd on its own, and with a control
//! map its `(control + 2)²` texels of the map resampled for the tiles, zlib'd
//! on their own; the layers into rows naming their textures (assets cooked
//! on their own). Each tile can be read and decoded without the others.
//!
//! ```text
//! type terrain
//! version 1
//! ---
//! heightmap "island-height.png"   # 16-bit (or 8-bit) grey PNG, or an EXR (its first channel)
//! height 0 320                    # the metres black and white stand for (EXR: 0 1, metres as stored)
//! size 2048 2048                  # metres along x and z (default: one metre per sample)
//! control "island-control.png"    # optional: RGBA, one channel per layer
//! tile 128                        # quads per tile side (16 to 1024)
//! heightBlend 0.6
//! castShadows true
//! lod {
//!   spacing 0                     # the finest level's spacing in metres (0: the samples')
//!   ring 64                       # quads per side of every clipmap level
//! }
//! layers "tiles.ogt"             # optional: a texture array (type texture-array, beside the terrain or under assets/) of the layers' albedos
//! layerNormals "tiles-n.ogt"      # optional: a texture array (kind normal) of their normal maps
//! layer {
//!   albedo "textures/grass.png"   # colour texture (alpha: the layer's height)
//!   normal "textures/rock-n.png"  # a normal map (give it `texture { kind normal }`)
//!   albedoLayer 0                 # with layers: its albedo's layer (default: its place, 0 to 3)
//!   normalLayer 0                 # with layerNormals: its normal map's layer (default: its place)
//!   color 1 1 1
//!   roughness 0.95
//!   tile 6                        # metres per repeat
//!   triplanar false
//!   normalStrength 1
//!   slope 0 34                    # degrees from flat where it shows
//!   slopeFade 6
//!   altitude -1000000 185         # metres above the terrain's origin
//!   altitudeFade 25
//! }
//! ```
//!
//! Or the terrain makes its field instead of reading one: a `generate`
//! block in place of `heightmap` (refused beside it), whose samples are a
//! heightmap's (`height` and `size` keep their meaning: the field is
//! normalised so its lowest sample stands at `height`'s low and its
//! highest at its high, before erosion, rivers and the shore, and no
//! sample leaves that range), built by
//! `terrain_generate` with only IEEE-exact arithmetic, so every host makes
//! the same field. With `biome` rules it writes the control map (refused
//! beside `control`); without them there is none, and the layers' own
//! slope and altitude rules place them. Every line, with its default:
//!
//! ```text
//! generate {
//!   samples 257 257          # columns and rows (2 to 4097)
//!   seed 1                   # a whole number: another seed, another terrain
//!   noise {
//!     octaves 6              # 1 to 12
//!     frequency 3            # features across the terrain's longer side
//!     lacunarity 2           # each octave's frequency over the last's (1 to 8)
//!     gain 0.5               # each octave's amplitude over the last's (0 to 1)
//!     warp 0                 # domain warp, in features (0: none)
//!   }
//!   mask island 0.4          # land fades to the low over this share of the half size
//!                            # or: mask ridge 1 (that share of the noise ridged); default no mask
//!   erosion {                # hydraulic erosion by seeded drops (absent: none)
//!     drops 40000
//!     lifetime 30            # steps a drop lives
//!     radius 3               # samples a drop erodes around it
//!     inertia 0.05           # 0: down the slope, 1: straight on
//!     capacity 4             # sediment carried per unit of speed, water and drop
//!     deposition 0.3         # share of the excess sediment dropped per step
//!     erosion 0.3            # share of the free capacity eroded per step
//!     evaporation 0.01       # share of the water lost per step
//!     gravity 4
//!   }
//!   thermal {                # thermal erosion (absent: none)
//!     talus 35               # degrees: steeper slopes slump
//!     iterations 20
//!     rate 0.5               # share of the excess moved per iteration
//!   }
//!   terraces {               # (absent: none)
//!     step 10                # metres between terraces
//!     blend 0.4              # 0: hard steps, 1: smooth ramps
//!   }
//!   sea 0                    # metres above the origin (absent: no sea); og.terrain.load's seaLevel
//!   shore 2                  # metres either side of the sea level flattened (needs sea)
//!   rivers {                 # flow gathered over the filled field, carved (absent: none)
//!     threshold 400          # samples draining through a sample before it is a river
//!     depth 2                # metres carved where the river is full
//!     width 6                # metres across where it is full
//!   }
//!   moisture {               # what a biome's moisture reads, 0 to 1
//!     flow 1                 # weight of the flow gathered (a / (a + scale))
//!     scale 64               # samples draining through for half the flow's share
//!     noise 0.5              # weight of a noise
//!     frequency 6            # its features across the terrain
//!   }
//!   biome {                  # any number; each writes its layer's control channel
//!     layer 0                # the layer it writes (0 to 3), required
//!     weight 1               # the channel where every rule holds (0 to 1)
//!     altitude -1000000 1000000   # metres above the origin
//!     altitudeFade 2
//!     slope 0 90             # degrees from flat
//!     slopeFade 4
//!     moisture 0 1
//!     moistureFade 0.1
//!   }
//! }
//! ```
//!
//! A channel is the most any of its layer's biomes gives a sample (its
//! weight times its three bands); the control map has the field's samples.
//!
//! The heightmap and the control map are read relative to the terrain file
//! (or, failing that, `assets/`); they are part of the terrain, not
//! textures of their own.
//!
//! With `layers` (and `layerNormals`) the layers' maps are layers of two
//! texture arrays, cooked on their own: the terrain samples two textures
//! instead of up to eight. A layer then names no `albedo` or `normal` of
//! its own; `albedoLayer` and `normalLayer` pick its layers. Rows of the heightmap run from -z (the image's
//! top) to +z, columns from -x to +x.

use super::{CookError, Source};
use crate::format::{self, BlobKind};
use og_blob::BlobWriter;
use og_core::ogt::{Block, Document};
use std::path::{Path, PathBuf};

pub(super) const TERRAIN_TYPE: &str = "terrain";

/// The most layers a terrain blends (og-render's `MAX_TERRAIN_LAYERS`).
const MAX_LAYERS: usize = og_render::MAX_TERRAIN_LAYERS;

fn assets_root(s: &Source) -> PathBuf {
    let mut p = s.path.clone();
    for _ in s.rel.split('/') {
        p.pop();
    }
    p
}

/// The asset path (under `assets/`) a terrain file's `name` refers to:
/// beside the terrain file when it is there, else under `assets/`.
pub(super) fn resolve(terrain_rel: &str, root: &Path, name: &str) -> String {
    let dir = terrain_rel.rsplit_once('/').map_or("", |(d, _)| d);
    let beside = if dir.is_empty() {
        name.to_string()
    } else {
        format!("{dir}/{name}")
    };
    if root.join(&beside).is_file() {
        beside
    } else {
        name.to_string()
    }
}

fn string_of(b: &Block, key: &str) -> Option<String> {
    b.get(key)
        .and_then(|e| e.values.first())
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

fn numbers(b: &Block, key: &str) -> Option<Vec<f32>> {
    let e = b.get(key)?;
    e.values
        .iter()
        .map(|v| v.as_number().map(|n| n.as_f32()))
        .collect()
}

fn number(b: &Block, key: &str, default: f32) -> Result<f32, String> {
    match numbers(b, key) {
        None => Ok(default),
        Some(v) if v.len() == 1 && v[0].is_finite() => Ok(v[0]),
        Some(_) => Err(format!("'{key}' takes one number")),
    }
}

fn pair(b: &Block, key: &str, default: [f32; 2]) -> Result<[f32; 2], String> {
    match numbers(b, key) {
        None => Ok(default),
        Some(v) if v.len() == 2 && v.iter().all(|x| x.is_finite()) => Ok([v[0], v[1]]),
        Some(_) => Err(format!("'{key}' takes two numbers")),
    }
}

fn flag(b: &Block, key: &str, default: bool) -> Result<bool, String> {
    match b.get(key).and_then(|e| e.values.first()) {
        None => Ok(default),
        Some(v) => v
            .as_bool()
            .ok_or_else(|| format!("'{key}' takes true or false")),
    }
}

/// The files a terrain reads besides itself (its heightmap and control
/// map, by asset path), for the stamp and so the walk does not cook them
/// as textures.
pub(super) fn terrain_files(rel: &str, root: &Path, text: &str) -> Vec<String> {
    let Ok(doc) = Document::parse(text) else {
        return Vec::new();
    };
    ["heightmap", "control"]
        .iter()
        .filter_map(|k| string_of(&doc.body, k))
        .map(|n| resolve(rel, root, &n))
        .collect()
}

/// The stamp's dependencies: each file the terrain reads, with its hash,
/// and whether each layer texture is an asset.
pub(super) fn terrain_dependencies(s: &Source, known: &dyn Fn(&str) -> bool) -> String {
    let root = assets_root(s);
    let text = String::from_utf8_lossy(&s.bytes);
    let mut out: Vec<String> = terrain_files(&s.rel, &root, &text)
        .into_iter()
        .map(|f| {
            format!(
                "{f}={:016x}",
                std::fs::read(root.join(&f))
                    .map(|b| super::hash_bytes(&b))
                    .unwrap_or(0)
            )
        })
        .collect();
    if let Ok(doc) = Document::parse(&text) {
        for k in ["layers", "layerNormals"] {
            if let Some(t) = string_of(&doc.body, k) {
                let t = resolve(&s.rel, &root, &t);
                out.push(format!("{t}={}", known(crate::runtime::asset_id(&t))));
            }
        }
        for l in doc.body.get_all("layer") {
            if let Some(b) = &l.block {
                for k in ["albedo", "normal"] {
                    if let Some(t) = string_of(b, k) {
                        out.push(format!("{t}={}", known(crate::runtime::asset_id(&t))));
                    }
                }
            }
        }
    }
    out.join("\n")
}

/// A decoded heightmap: samples in 0..1 (PNG) or as stored (EXR), row-major.
pub(super) struct Heightmap {
    pub columns: u32,
    pub rows: u32,
    pub values: Vec<f32>,
    pub exr: bool,
}

/// Decodes a heightmap: a grey (or any) PNG to its first channel at 16
/// bits in 0..1, an EXR to its first channel as stored.
pub(super) fn decode_heightmap(name: &str, bytes: &[u8]) -> Result<Heightmap, String> {
    let exr = name.to_ascii_lowercase().ends_with(".exr");
    let img = image::load_from_memory(bytes).map_err(|e| format!("{name}: {e}"))?;
    let (columns, rows) = (img.width(), img.height());
    if columns < 2 || rows < 2 {
        return Err(format!(
            "{name} is {columns} × {rows}: a heightmap needs at least 2 × 2"
        ));
    }
    let values: Vec<f32> = if exr {
        img.to_rgb32f().into_raw().chunks(3).map(|p| p[0]).collect()
    } else {
        img.to_luma16()
            .into_raw()
            .into_iter()
            .map(|v| v as f32 / 65535.0)
            .collect()
    };
    if values.iter().any(|v| !v.is_finite()) {
        return Err(format!("{name} holds a value that is not a finite number"));
    }
    Ok(Heightmap {
        columns,
        rows,
        values,
        exr,
    })
}

/// A terrain file's settings, read and checked.
pub(super) struct TerrainDoc {
    /// The heightmap (an asset path), or none: `generate` makes the field.
    pub heightmap: Option<String>,
    pub generate: Option<super::terrain_generate::Generate>,
    pub control: Option<String>,
    pub height: [f32; 2],
    pub size: Option<[f32; 2]>,
    pub tile: u32,
    pub height_blend: f32,
    pub cast_shadows: bool,
    pub collision: bool,
    pub lod_spacing: f32,
    pub lod_ring: u32,
    pub layers: Vec<(Option<String>, Option<String>, og_render::TerrainLayer)>,
    /// `layers` and `layerNormals`: the texture arrays (asset paths).
    pub albedo_array: Option<String>,
    pub normal_array: Option<String>,
}

const KEYS: [&str; 13] = [
    "heightmap",
    "generate",
    "height",
    "size",
    "control",
    "tile",
    "heightBlend",
    "castShadows",
    "collision",
    "lod",
    "layer",
    "layers",
    "layerNormals",
];
const LAYER_KEYS: [&str; 13] = [
    "albedo",
    "normal",
    "albedoLayer",
    "normalLayer",
    "color",
    "roughness",
    "tile",
    "triplanar",
    "normalStrength",
    "slope",
    "slopeFade",
    "altitude",
    "altitudeFade",
];

pub(super) fn parse(rel: &str, root: &Path, text: &str) -> Result<TerrainDoc, String> {
    let doc = Document::parse(text).map_err(|e| e.to_string())?;
    let body = &doc.body;
    for e in &body.entries {
        if !KEYS.contains(&e.key.as_str()) {
            return Err(format!(
                "line {}: '{}' is not a terrain entry: {}",
                e.line,
                e.key,
                KEYS.join(", ")
            ));
        }
    }
    let heightmap = string_of(body, "heightmap");
    let generate = match (body.get("generate"), body.get("heightmap")) {
        (Some(g), Some(h)) => {
            return Err(format!(
                "line {}: a terrain makes its field (generate) or reads it (heightmap, line {}), not both",
                g.line, h.line
            ));
        }
        (Some(g), None) => Some(super::terrain_generate::parse(g)?),
        (None, _) => None,
    };
    if heightmap.is_none() && generate.is_none() {
        return Err("a terrain names its heightmap: heightmap \"<file>.png\" (16-bit grey) or .exr, or makes it: generate { ... }".into());
    }
    let exr = heightmap
        .as_ref()
        .is_some_and(|h| h.to_ascii_lowercase().ends_with(".exr"));
    let height = pair(body, "height", if exr { [0.0, 1.0] } else { [0.0, 100.0] })?;
    let size = match numbers(body, "size") {
        None => None,
        Some(v) if v.len() == 2 && v.iter().all(|x| x.is_finite() && *x > 0.0) => {
            Some([v[0], v[1]])
        }
        Some(_) => return Err("'size' takes two positive numbers (metres along x and z)".into()),
    };
    let tile = number(body, "tile", og_render::TERRAIN_TILE as f32)?;
    if !(16.0..=1024.0).contains(&tile) || tile.fract() != 0.0 {
        return Err(format!(
            "'tile' is a whole number of quads from 16 to 1024, got {tile}"
        ));
    }
    let lod = body.get("lod").and_then(|e| e.block.as_ref());
    let (lod_spacing, lod_ring) = match lod {
        Some(b) => (
            number(b, "spacing", 0.0)?.max(0.0),
            number(b, "ring", 0.0)?.max(0.0) as u32,
        ),
        None => (0.0, 0),
    };
    let albedo_array = string_of(body, "layers");
    let normal_array = string_of(body, "layerNormals");
    for (key, name) in [("layers", &albedo_array), ("layerNormals", &normal_array)] {
        if let Some(e) = body.get(key)
            && name
                .as_deref()
                .is_none_or(|n| !n.to_ascii_lowercase().ends_with(".ogt"))
        {
            return Err(format!(
                "line {}: {key} names a texture array, an .ogt of type texture-array: {key} \"tiles.ogt\"",
                e.line
            ));
        }
    }
    let arrayed = albedo_array.is_some() || normal_array.is_some();
    let mut layers = Vec::new();
    for e in body.get_all("layer") {
        let Some(b) = &e.block else {
            return Err(format!(
                "line {}: a layer is a block: layer {{ ... }}",
                e.line
            ));
        };
        for k in &b.entries {
            if !LAYER_KEYS.contains(&k.key.as_str()) {
                return Err(format!(
                    "line {}: '{}' is not a layer entry: {}",
                    k.line,
                    k.key,
                    LAYER_KEYS.join(", ")
                ));
            }
        }
        let d = og_render::TerrainLayer::default();
        let color = match numbers(b, "color") {
            None => [1.0, 1.0, 1.0],
            Some(v) if v.len() == 3 => [v[0], v[1], v[2]],
            Some(_) => return Err(format!("line {}: 'color' takes three numbers", e.line)),
        };
        let place = layers.len() as f32;
        let index = |key: &str, array: &Option<String>| -> Result<Option<u32>, String> {
            match (b.get(key), array) {
                (Some(k), None) => Err(format!(
                    "line {}: '{key}' picks a layer of the terrain's {}: name it first",
                    k.line,
                    if key == "albedoLayer" {
                        "layers \"<array>.ogt\""
                    } else {
                        "layerNormals \"<array>.ogt\""
                    }
                )),
                (_, None) => Ok(None),
                (_, Some(_)) => {
                    let v = number(b, key, place)?;
                    if v < 0.0 || v.fract() != 0.0 {
                        return Err(format!("'{key}' is a whole layer number from 0, got {v}"));
                    }
                    Ok(Some(v as u32))
                }
            }
        };
        if arrayed && let Some(k) = b.get("albedo").or_else(|| b.get("normal")) {
            return Err(format!(
                "line {}: with layer arrays (layers, layerNormals) a layer's maps are their layers: albedoLayer and normalLayer, not '{}'",
                k.line, k.key
            ));
        }
        let layer = og_render::TerrainLayer {
            albedo_layer: index("albedoLayer", &albedo_array)?,
            normal_layer: index("normalLayer", &normal_array)?,
            albedo: None,
            normal: None,
            color: color.into(),
            roughness: number(b, "roughness", d.roughness)?,
            tile: number(b, "tile", d.tile)?,
            triplanar: flag(b, "triplanar", false)?,
            normal_strength: number(b, "normalStrength", 1.0)?,
            slope: pair(b, "slope", d.slope)?,
            slope_fade: number(b, "slopeFade", d.slope_fade)?,
            altitude: pair(b, "altitude", d.altitude)?,
            altitude_fade: number(b, "altitudeFade", d.altitude_fade)?,
        };
        layers.push((string_of(b, "albedo"), string_of(b, "normal"), layer));
    }
    if layers.len() > MAX_LAYERS {
        return Err(format!(
            "a terrain blends at most {MAX_LAYERS} layers, this one has {}",
            layers.len()
        ));
    }
    if let Some(g) = &generate {
        if let (Some(b), Some(c)) = (g.biomes.first(), body.get("control")) {
            return Err(format!(
                "line {}: biomes write the terrain's control map: drop 'control' (line {}) or the biomes",
                b.line, c.line
            ));
        }
        if let Some(b) = g.biomes.iter().find(|b| b.layer as usize >= layers.len()) {
            return Err(format!(
                "line {}: the biome writes layer {} but the terrain has {} layer(s)",
                b.line,
                b.layer,
                layers.len()
            ));
        }
    }
    Ok(TerrainDoc {
        heightmap: heightmap.map(|h| resolve(rel, root, &h)),
        generate,
        control: string_of(body, "control").map(|c| resolve(rel, root, &c)),
        height,
        size,
        tile: tile as u32,
        height_blend: number(body, "heightBlend", 0.0)?.max(0.0),
        cast_shadows: flag(body, "castShadows", true)?,
        collision: flag(body, "collision", true)?,
        lod_spacing,
        lod_ring,
        layers,
        albedo_array: albedo_array.map(|a| resolve(rel, root, &a)),
        normal_array: normal_array.map(|a| resolve(rel, root, &a)),
    })
}

/// A terrain's samples, read from its heightmap (and control map) or made
/// by its generator.
pub(super) struct Field {
    pub columns: u32,
    pub rows: u32,
    /// Metres above the origin (what the runtime's terrain draws, before
    /// quantisation): a heightmap's `min + v · (max - min)`.
    pub heights: Vec<f32>,
    pub control: Option<og_render::TerrainControl>,
    /// Its extent: `size`, or a metre per sample.
    pub size: [f32; 2],
    pub exr: bool,
    /// Made by `generate`: its sea level and its rivers' length in metres.
    pub generated: Option<(Option<f32>, f32)>,
}

/// Reads (or makes) a terrain file's samples.
pub(super) fn field(doc: &TerrainDoc, root: &Path) -> Result<Field, String> {
    let read = |rel: &str, what: &str| {
        std::fs::read(root.join(rel)).map_err(|_| {
            format!("the {what} '{rel}' does not exist beside the terrain or under assets/")
        })
    };
    let control = || -> Result<Option<og_render::TerrainControl>, String> {
        match &doc.control {
            None => Ok(None),
            Some(c) => {
                let img = super::texture::decode(&read(c, "control map")?)
                    .map_err(|e| format!("{c}: {e}"))?;
                Ok(Some(og_render::TerrainControl {
                    width: img.width,
                    height: img.height,
                    rgba: img.rgba.into(),
                }))
            }
        }
    };
    let size_of = |columns: u32, rows: u32| {
        doc.size
            .unwrap_or([(columns - 1) as f32, (rows - 1) as f32])
    };
    if let Some(g) = &doc.generate {
        let size = size_of(g.columns, g.rows);
        let made = super::terrain_generate::generate(g, size, doc.height);
        let control = match made.control {
            Some(rgba) => Some(og_render::TerrainControl {
                width: g.columns,
                height: g.rows,
                rgba: rgba.into(),
            }),
            None => control()?,
        };
        return Ok(Field {
            columns: g.columns,
            rows: g.rows,
            heights: made.heights,
            control,
            size,
            exr: false,
            generated: Some((g.sea, made.river_length)),
        });
    }
    let name = doc
        .heightmap
        .as_deref()
        .expect("parse: a heightmap or generate");
    let map = decode_heightmap(name, &read(name, "heightmap")?)?;
    let [lo, hi] = doc.height;
    Ok(Field {
        columns: map.columns,
        rows: map.rows,
        heights: map.values.iter().map(|v| lo + v * (hi - lo)).collect(),
        control: control()?,
        size: size_of(map.columns, map.rows),
        exr: map.exr,
        generated: None,
    })
}

pub(super) fn cook_terrain(
    s: &Source,
    known: &dyn Fn(&str) -> bool,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let text =
        std::str::from_utf8(&s.bytes).map_err(|_| CookError("a terrain is text (UTF-8)".into()))?;
    let root = assets_root(s);
    let doc = parse(&s.rel, &root, text).map_err(CookError)?;
    let source = field(&doc, &root).map_err(CookError)?;
    let (columns, rows, size) = (source.columns, source.rows, source.size);
    let (sea, river_length) = source.generated.unwrap_or((None, 0.0));
    let mut missing = Vec::new();
    for (a, n, _) in &doc.layers {
        for t in [a, n].into_iter().flatten() {
            if !known(crate::runtime::asset_id(t)) {
                missing.push(t.clone());
            }
        }
    }
    for t in [&doc.albedo_array, &doc.normal_array].into_iter().flatten() {
        if !known(crate::runtime::asset_id(t)) {
            missing.push(t.clone());
        }
    }
    if !missing.is_empty() {
        return Err(CookError(format!(
            "the terrain names texture(s) not under assets/: {}",
            missing.join(", ")
        )));
    }

    let control = source.control;
    let (control_w, control_h) = control.as_ref().map_or((0, 0), |c| (c.width, c.height));
    let field = og_render::TerrainData {
        size: glam::Vec2::new(size[0], size[1]),
        columns,
        rows,
        heights: source.heights.into(),
        control,
    };
    field.check().map_err(CookError)?;
    let layout = field.layout(doc.tile).map_err(CookError)?;
    let (lo_all, hi_all) = field.range();
    let span = (hi_all - lo_all).max(f32::MIN_POSITIVE);
    let pyramid = field
        .control
        .as_ref()
        .and_then(|c| og_render::ControlPyramid::new(&layout, c));
    // Every tile of every level: its heights quantised over the terrain's
    // range, its control texels, each zlib'd on its own.
    let mut tiles = Vec::new();
    let mut packed = Vec::new();
    let mut control_packed = Vec::new();
    for index in 0..layout.tile_count() {
        let (level, x, z) = layout.locate(index).expect("in the layout");
        let heights = field.tile_heights(&layout, index);
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        let mut q = Vec::with_capacity(heights.len() * 2);
        for v in &heights {
            lo = lo.min(*v);
            hi = hi.max(*v);
            let n = (((v - lo_all) / span) * 65535.0)
                .round()
                .clamp(0.0, 65535.0) as u16;
            q.extend_from_slice(&n.to_le_bytes());
        }
        let z_heights = miniz_oxide::deflate::compress_to_vec_zlib(&q, 6);
        let (control_offset, control_bytes) = match &pyramid {
            Some(p) => {
                let zc = miniz_oxide::deflate::compress_to_vec_zlib(&p.tile(&layout, index), 6);
                let at = control_packed.len() as u32;
                control_packed.extend_from_slice(&zc);
                (at, zc.len() as u32)
            }
            None => (0, 0),
        };
        tiles.push(format::TerrainTileRow {
            index,
            level,
            x,
            z,
            min: lo,
            max: hi,
            offset: packed.len() as u32,
            bytes: z_heights.len() as u32,
            control_offset,
            control_bytes,
        });
        packed.extend_from_slice(&z_heights);
    }
    let mut strings = String::new();
    let mut put = |s: &Option<String>| -> (u32, u32) {
        match s {
            None => (format::NONE, 0),
            Some(s) => {
                let at = strings.len() as u32;
                strings.push_str(crate::runtime::asset_id(s));
                (at, s.len() as u32)
            }
        }
    };
    let layers: Vec<format::TerrainLayerRow> = doc
        .layers
        .iter()
        .map(|(a, n, l)| {
            let (albedo_at, albedo_len) = put(a);
            let (normal_at, normal_len) = put(n);
            format::TerrainLayerRow {
                albedo_at,
                albedo_len,
                normal_at,
                normal_len,
                r: l.color.x,
                g: l.color.y,
                b: l.color.z,
                roughness: l.roughness,
                tile: l.tile,
                triplanar: u32::from(l.triplanar),
                normal_strength: l.normal_strength,
                slope_min: l.slope[0],
                slope_max: l.slope[1],
                slope_fade: l.slope_fade,
                altitude_min: l.altitude[0],
                altitude_max: l.altitude[1],
                altitude_fade: l.altitude_fade,
                albedo_layer: l.albedo_layer.unwrap_or(format::NONE),
                normal_layer: l.normal_layer.unwrap_or(format::NONE),
            }
        })
        .collect();
    let (albedo_array_at, albedo_array_len) = put(&doc.albedo_array);
    let (normal_array_at, normal_array_len) = put(&doc.normal_array);
    let meta = format::TerrainMeta {
        size_x: size[0],
        size_z: size[1],
        columns,
        rows,
        tile: layout.tile,
        control: layout.control,
        levels: layout.levels,
        height_min: lo_all,
        height_max: hi_all,
        layers: layers.len() as u32,
        control_width: control_w,
        control_height: control_h,
        height_blend: doc.height_blend,
        lod_spacing: doc.lod_spacing,
        lod_ring: doc.lod_ring,
        cast_shadows: u32::from(doc.cast_shadows),
        collision: u32::from(doc.collision),
        albedo_array_at,
        albedo_array_len,
        normal_array_at,
        normal_array_len,
        generated: u32::from(source.generated.is_some()),
        sea: u32::from(sea.is_some()),
        sea_level: sea.unwrap_or(0.0),
        river_length,
    };
    let mut w = BlobWriter::new(
        format::Terrain::type_hash(),
        format::Terrain::schema_version(),
        stamp,
    );
    w.table("meta", &[meta], format::row_hash::<format::TerrainMeta>())
        .table(
            "tiles",
            &tiles,
            format::row_hash::<format::TerrainTileRow>(),
        )
        .table(
            "layers",
            &layers,
            format::row_hash::<format::TerrainLayerRow>(),
        )
        .raw("strings", strings.into_bytes(), 16)
        .raw("heights", packed, 16)
        .raw("control", control_packed, 16);
    let mut notes = vec![format!(
        "terrain: {columns} × {rows} samples over {} × {} m, heights {lo_all:.2} to {hi_all:.2} m, {} tiles of {} quads in {} levels, {} layer(s){}",
        size[0],
        size[1],
        tiles.len(),
        layout.tile,
        layout.levels,
        layers.len(),
        if control_w > 0 {
            format!(", control map {control_w} × {control_h}")
        } else {
            String::new()
        }
    )];
    if source.exr {
        notes.push("heights from an EXR's first channel".into());
    }
    if let Some(g) = &doc.generate {
        notes.push(format!(
            "generated: seed {}, {}{} m of rivers",
            g.seed,
            sea.map_or(String::new(), |s| format!("sea at {s} m, ")),
            river_length.round()
        ));
    }
    Ok((w.finish(), notes))
}
