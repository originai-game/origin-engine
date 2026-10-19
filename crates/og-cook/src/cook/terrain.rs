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
//! layer {
//!   albedo "textures/grass.png"   # colour texture (alpha: the layer's height)
//!   normal "textures/rock-n.png"  # a normal map (give it `texture { kind normal }`)
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
//! The heightmap and the control map are read relative to the terrain file
//! (or, failing that, `assets/`); they are part of the terrain, not
//! textures of their own. Rows of the heightmap run from -z (the image's
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
    pub heightmap: String,
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
}

const KEYS: [&str; 10] = [
    "heightmap",
    "height",
    "size",
    "control",
    "tile",
    "heightBlend",
    "castShadows",
    "collision",
    "lod",
    "layer",
];
const LAYER_KEYS: [&str; 11] = [
    "albedo",
    "normal",
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
    let heightmap = string_of(body, "heightmap")
        .ok_or("a terrain names its heightmap: heightmap \"<file>.png\" (16-bit grey) or .exr")?;
    let exr = heightmap.to_ascii_lowercase().ends_with(".exr");
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
        let layer = og_render::TerrainLayer {
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
    Ok(TerrainDoc {
        heightmap: resolve(rel, root, &heightmap),
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
    })
}

/// The heights in metres a terrain file's heightmap stands for (what the
/// runtime's terrain draws, before quantisation): `min + v · (max - min)`.
pub(super) fn heights(doc: &TerrainDoc, map: &Heightmap) -> Vec<f32> {
    let [lo, hi] = doc.height;
    map.values.iter().map(|v| lo + v * (hi - lo)).collect()
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
    let read = |rel: &str, what: &str| {
        std::fs::read(root.join(rel)).map_err(|_| {
            CookError(format!(
                "the {what} '{rel}' does not exist beside the terrain or under assets/"
            ))
        })
    };
    let map =
        decode_heightmap(&doc.heightmap, &read(&doc.heightmap, "heightmap")?).map_err(CookError)?;
    let h = heights(&doc, &map);
    let (columns, rows) = (map.columns, map.rows);
    let size = doc
        .size
        .unwrap_or([(columns - 1) as f32, (rows - 1) as f32]);
    let mut missing = Vec::new();
    for (a, n, _) in &doc.layers {
        for t in [a, n].into_iter().flatten() {
            if !known(crate::runtime::asset_id(t)) {
                missing.push(t.clone());
            }
        }
    }
    if !missing.is_empty() {
        return Err(CookError(format!(
            "the terrain names texture(s) not under assets/: {}",
            missing.join(", ")
        )));
    }

    let control = match &doc.control {
        None => None,
        Some(c) => {
            let img = super::texture::decode(&read(c, "control map")?)
                .map_err(|e| CookError(format!("{c}: {e}")))?;
            Some(og_render::TerrainControl {
                width: img.width,
                height: img.height,
                rgba: img.rgba.into(),
            })
        }
    };
    let (control_w, control_h) = control.as_ref().map_or((0, 0), |c| (c.width, c.height));
    let field = og_render::TerrainData {
        size: glam::Vec2::new(size[0], size[1]),
        columns,
        rows,
        heights: h.into(),
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
            }
        })
        .collect();
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
    if map.exr {
        notes.push("heights from an EXR's first channel".into());
    }
    Ok((w.finish(), notes))
}
