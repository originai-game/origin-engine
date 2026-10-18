//! Tilesets and tilemaps: `.ogt` tilesets and maps, Tiled `.tmj` maps and
//! LDtk `.ldtk` projects into the `og.Tileset` and `og.Tilemap` blobs.
//!
//! The importers make an [`og_tilemap::TilemapAsset`] (native only: they
//! live behind the `cook` feature and never reach the wasm runtime); the
//! blob is the same whatever the source.

use super::CookError;
use crate::format::{self, *};
use crate::runtime::asset_id;
use og_blob::BlobWriter;
use og_core::ogt::Block;
use og_tilemap::{
    Collision, Filter, Layer, MapObject, TileAnim, TileDef, TilemapAsset, Tileset, TilesetRef,
    FLIP_DIAGONAL, FLIP_X, FLIP_Y, TILE_MASK,
};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::BTreeMap;

/// The `tilemap { ... }` block of a map's import settings.
#[derive(Clone, Debug, Default)]
pub struct MapSettings {
    /// World units per tile width (default 1).
    pub unit: Option<f32>,
    /// LDtk: the level to import, by identifier (default the first).
    pub level: Option<String>,
    /// LDtk: the IntGrid values that are solid (default 1).
    pub solid: Vec<i64>,
    /// Every tileset of the map replaced by this tileset `.ogt`.
    pub tileset: Option<String>,
}

impl MapSettings {
    pub fn parse(block: Option<&Block>) -> Result<MapSettings, String> {
        let mut s = MapSettings::default();
        for e in block.iter().flat_map(|b| &b.entries) {
            match e.key.as_str() {
                "unit" => s.unit = Some(e.f32(0).map_err(|_| "tilemap.unit needs a number")?),
                "level" => {
                    s.level = Some(
                        e.values
                            .first()
                            .and_then(|v| v.as_str().or(v.as_word()))
                            .ok_or("tilemap.level needs the level's identifier")?
                            .to_string(),
                    )
                }
                "solid" => {
                    for i in 0..e.values.len() {
                        s.solid.push(
                            e.number(i)
                                .ok()
                                .and_then(|n| n.as_i64().ok())
                                .ok_or("tilemap.solid lists IntGrid values")?,
                        );
                    }
                }
                "tileset" => {
                    s.tileset = Some(
                        e.string(0)
                            .map_err(|_| "tilemap.tileset needs a quoted asset path")?
                            .to_string(),
                    )
                }
                other => {
                    return Err(format!(
                        "unknown key '{other}' in the tilemap import settings"
                    ))
                }
            }
        }
        Ok(s)
    }

    pub fn describe(&self) -> String {
        format!("{:?}", self)
    }
}

/// `rel` (a path in a file at asset id `base`) as an asset id: relative to
/// `base`'s directory, `.` and `..` resolved.
pub fn resolve(base: &str, rel: &str) -> String {
    let rel = rel.replace('\\', "/");
    let mut parts: Vec<&str> = if rel.starts_with('/') {
        Vec::new()
    } else {
        base.rsplit_once('/')
            .map_or(Vec::new(), |(d, _)| d.split('/').collect())
    };
    for p in rel.split('/') {
        match p {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            p => parts.push(p),
        }
    }
    asset_id(&parts.join("/")).to_string()
}

/// `path` with its extension replaced by `.ogt`.
fn sibling_ogt(path: &str) -> String {
    match path.rsplit_once('.') {
        Some((stem, ext)) if !ext.contains('/') => format!("{stem}.ogt"),
        _ => format!("{path}.ogt"),
    }
}

// ── cooking ──────────────────────────────────────────────────────────────

/// A tileset `.ogt` into its blob; the texture must be an asset.
pub fn cook_tileset(
    text: &str,
    known: &dyn Fn(&str) -> bool,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let t = Tileset::parse(text).map_err(|e| CookError(e.to_string()))?;
    if !t.texture.is_empty() && !known(asset_id(&t.texture)) {
        return Err(CookError(format!(
            "the tileset's texture '{}' is not under assets/",
            t.texture
        )));
    }
    let canonical = t.to_text();
    let mut w = BlobWriter::new(
        format::Tileset::type_hash(),
        format::Tileset::schema_version(),
        stamp,
    );
    w.table(
        "meta",
        &[TilesetMeta {
            text_bytes: canonical.len() as u32,
        }],
        row_hash::<TilesetMeta>(),
    )
    .raw("text", canonical.into_bytes(), 16);
    let note = format!(
        "{}x{} tiles of {}x{} px, {} terrain(s), {} animated",
        t.columns,
        t.rows,
        t.tile_width,
        t.tile_height,
        t.terrains.len(),
        t.tiles.values().filter(|d| d.anim.is_some()).count()
    );
    Ok((w.finish(), vec![note]))
}

/// Imports a map source (`.ogt`, `.tmj` or `.ldtk` by `rel`'s extension).
pub fn import(
    rel: &str,
    bytes: &[u8],
    settings: &MapSettings,
    known: &dyn Fn(&str) -> bool,
) -> Result<TilemapAsset, String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "a map is text (UTF-8)".to_string())?;
    let ext = rel
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();
    let mut map = match ext.as_str() {
        "tmj" => import_tiled(rel, text, settings, known)?,
        "ldtk" => import_ldtk(rel, text, settings, known)?,
        _ => TilemapAsset::parse(text).map_err(|e| e.to_string())?,
    };
    if ext == "ogt" {
        if let Some(u) = settings.unit {
            map.unit = u;
        }
    }
    if let Some(t) = &settings.tileset {
        for r in &mut map.tilesets {
            *r = TilesetRef::Asset(asset_id(t).to_string());
        }
    }
    for r in &map.tilesets {
        match r {
            TilesetRef::Asset(id) => {
                if !known(asset_id(id)) {
                    return Err(format!(
                        "the map's tileset '{id}' is not under assets/ (a tileset .ogt)"
                    ));
                }
            }
            TilesetRef::Embedded(t) => {
                if !t.texture.is_empty() && !known(&t.texture) {
                    return Err(format!(
                        "the tileset image '{}' is not under assets/",
                        t.texture
                    ));
                }
            }
        }
    }
    map.validate().map_err(|e| e.to_string())?;
    Ok(map)
}

/// The asset ids a map source looks up (with whether each exists), for
/// its cook stamp.
pub fn references(
    rel: &str,
    bytes: &[u8],
    settings: &MapSettings,
    known: &dyn Fn(&str) -> bool,
) -> Vec<String> {
    let seen = RefCell::new(Vec::new());
    let probe = |id: &str| {
        let k = known(id);
        seen.borrow_mut().push(format!("{id}={k}"));
        k
    };
    let _ = import(rel, bytes, settings, &probe);
    let mut v = seen.into_inner();
    v.sort();
    v.dedup();
    v
}

/// A map source into its blob.
pub fn cook_tilemap(
    rel: &str,
    bytes: &[u8],
    settings: &MapSettings,
    known: &dyn Fn(&str) -> bool,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let map = import(rel, bytes, settings, known).map_err(CookError)?;
    let note = format!(
        "{}x{} cells, {} layer(s), {} object(s), {} tileset(s)",
        map.width,
        map.height,
        map.layers.len(),
        map.objects.len(),
        map.tilesets.len()
    );
    Ok((encode(&map, stamp), vec![note]))
}

/// The `og.Tilemap` blob of a map.
pub fn encode(map: &TilemapAsset, stamp: u64) -> Vec<u8> {
    let mut strings: Vec<u8> = Vec::new();
    let mut s = |text: &str| -> (u32, u32) {
        let o = strings.len() as u32;
        strings.extend_from_slice(text.as_bytes());
        (o, text.len() as u32)
    };
    let mut layers = Vec::new();
    let mut cells: Vec<u8> = Vec::new();
    let mut offset = 0u32;
    for l in &map.layers {
        let (no, nl) = s(&l.name);
        let (to, tl) = l.terrain.as_deref().map_or((0, 0), &mut s);
        let mut flags = 0;
        for (on, bit) in [
            (l.visible, layer_flag::VISIBLE),
            (l.collision, layer_flag::COLLISION),
            (l.repeat_x, layer_flag::REPEAT_X),
            (l.repeat_y, layer_flag::REPEAT_Y),
            (l.terrain.is_some(), layer_flag::TERRAIN),
        ] {
            if on {
                flags |= bit;
            }
        }
        layers.push(MapLayerRow {
            name_offset: no,
            name_len: nl,
            tileset: l.tileset,
            terrain_offset: to,
            terrain_len: tl,
            parallax_x: l.parallax.x,
            parallax_y: l.parallax.y,
            offset_x: l.offset.x,
            offset_y: l.offset.y,
            opacity: l.opacity,
            flags,
            z: l.z,
            cell_offset: offset,
        });
        for c in &l.cells {
            cells.extend_from_slice(&c.to_le_bytes());
        }
        offset += l.cells.len() as u32;
    }
    let mut objects = Vec::new();
    let mut props = Vec::new();
    for o in &map.objects {
        let (no, nl) = s(&o.name);
        let (ko, kl) = s(&o.kind);
        let first = props.len() as u32;
        for (k, v) in &o.properties {
            let (a, b) = s(k);
            let (c, d) = s(v);
            props.push(MapPropRow {
                key_offset: a,
                key_len: b,
                value_offset: c,
                value_len: d,
            });
        }
        objects.push(MapObjectRow {
            name_offset: no,
            name_len: nl,
            kind_offset: ko,
            kind_len: kl,
            x: o.x,
            y: o.y,
            width: o.width,
            height: o.height,
            prop_first: first,
            prop_count: o.properties.len() as u32,
        });
    }
    let tilesets: Vec<MapTilesetRow> = map
        .tilesets
        .iter()
        .map(|t| match t {
            TilesetRef::Asset(id) => {
                let (o, l) = s(id);
                MapTilesetRow {
                    embedded: 0,
                    offset: o,
                    len: l,
                }
            }
            TilesetRef::Embedded(t) => {
                let (o, l) = s(&t.to_text());
                MapTilesetRow {
                    embedded: 1,
                    offset: o,
                    len: l,
                }
            }
        })
        .collect();
    let meta = TilemapMeta {
        width: map.width,
        height: map.height,
        tile_width_px: map.tile_width_px,
        tile_height_px: map.tile_height_px,
        unit: map.unit,
        layer_count: layers.len() as u32,
        object_count: objects.len() as u32,
        tileset_count: tilesets.len() as u32,
        prop_count: props.len() as u32,
    };
    let mut w = BlobWriter::new(
        format::Tilemap::type_hash(),
        format::Tilemap::schema_version(),
        stamp,
    );
    w.table("meta", &[meta], row_hash::<TilemapMeta>())
        .table("layers", &layers, row_hash::<MapLayerRow>())
        .raw("cells", cells, 16)
        .table("objects", &objects, row_hash::<MapObjectRow>())
        .table("props", &props, row_hash::<MapPropRow>())
        .table("tilesets", &tilesets, row_hash::<MapTilesetRow>())
        .raw("strings", strings, 4);
    w.finish()
}

// ── JSON helpers ─────────────────────────────────────────────────────────

fn u(v: &Value, k: &str) -> Option<u64> {
    v.get(k).and_then(Value::as_u64)
}
fn f(v: &Value, k: &str, d: f64) -> f64 {
    v.get(k).and_then(Value::as_f64).unwrap_or(d)
}
fn st<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}
fn need_u(v: &Value, k: &str, what: &str) -> Result<u32, String> {
    u(v, k)
        .and_then(|x| u32::try_from(x).ok())
        .ok_or_else(|| format!("{what} has no whole-number '{k}'"))
}

fn value_text(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Tiled's `properties: [{name, type, value}]` by name.
fn tiled_props(v: &Value) -> BTreeMap<String, Value> {
    v.get("properties")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|p| {
            Some((
                p.get("name")?.as_str()?.to_string(),
                p.get("value")?.clone(),
            ))
        })
        .collect()
}

fn base64(text: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0;
    for c in text.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' | b' ' | b'\n' | b'\r' | b'\t' => continue,
            _ => return Err("the layer's base64 data has a character outside base64".into()),
        };
        buf = buf << 6 | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    Ok(out)
}

// ── Tiled ────────────────────────────────────────────────────────────────

struct TiledTileset {
    firstgid: u32,
    index: u32,
    count: u32,
}

fn tiled_embedded(rel: &str, t: &Value, name: &str) -> Result<Tileset, String> {
    let what = format!("embedded tileset '{name}'");
    let columns = need_u(t, "columns", &what)?.max(1);
    let count = need_u(t, "tilecount", &what)?;
    let mut ts = Tileset {
        texture: {
            let image = st(t, "image");
            if image.is_empty() {
                return Err(format!(
                    "{what} has no single image (image collections are not supported)"
                ));
            }
            resolve(rel, image)
        },
        tile_width: need_u(t, "tilewidth", &what)?,
        tile_height: need_u(t, "tileheight", &what)?,
        columns,
        rows: count.div_ceil(columns).max(1),
        margin: u(t, "margin").unwrap_or(0) as u32,
        spacing: u(t, "spacing").unwrap_or(0) as u32,
        filter: Filter::Nearest,
        tiles: BTreeMap::new(),
        terrains: Vec::new(),
    };
    for tile in t
        .get("tiles")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let id = need_u(tile, "id", &what)?;
        let mut def = TileDef::default();
        if tiled_props(tile).get("solid").and_then(Value::as_bool) == Some(true) {
            def.collision = Some(Collision::Full);
        }
        if let Some(o) = tile
            .get("objectgroup")
            .and_then(|g| g.get("objects"))
            .and_then(Value::as_array)
            .and_then(|a| a.first())
        {
            let (x, y, w, h) = (
                f(o, "x", 0.0) as f32,
                f(o, "y", 0.0) as f32,
                f(o, "width", 0.0) as f32,
                f(o, "height", 0.0) as f32,
            );
            if w > 0.0 && h > 0.0 {
                def.collision = Some(
                    if x <= 0.0
                        && y <= 0.0
                        && w >= ts.tile_width as f32
                        && h >= ts.tile_height as f32
                    {
                        Collision::Full
                    } else {
                        Collision::Rect { x, y, w, h }
                    },
                );
            }
        }
        if let Some(frames) = tile.get("animation").and_then(Value::as_array) {
            let mut a = TileAnim {
                frames: Vec::new(),
                durations: Vec::new(),
            };
            for fr in frames {
                a.frames.push(need_u(fr, "tileid", &what)?);
                a.durations.push(f(fr, "duration", 100.0) as f32 / 1000.0);
            }
            if !a.frames.is_empty() {
                def.anim = Some(a);
            }
        }
        if def != TileDef::default() {
            ts.tiles.insert(id, def);
        }
    }
    ts.validate().map_err(|e| format!("{what}: {e}"))?;
    Ok(ts)
}

/// A Tiled JSON map (`.tmj`, orthogonal, finite, uncompressed layer data).
pub fn import_tiled(
    rel: &str,
    text: &str,
    settings: &MapSettings,
    known: &dyn Fn(&str) -> bool,
) -> Result<TilemapAsset, String> {
    let j: Value = serde_json::from_str(text).map_err(|e| format!("Tiled JSON: {e}"))?;
    if st(&j, "orientation") != "orthogonal" {
        return Err(format!(
            "the map's orientation is '{}': only orthogonal Tiled maps are supported",
            st(&j, "orientation")
        ));
    }
    if j.get("infinite").and_then(Value::as_bool) == Some(true) {
        return Err(
            "the map is infinite: untick Map > Map Properties > Infinite in Tiled and save again"
                .into(),
        );
    }
    let width = need_u(&j, "width", "the map")?;
    let height = need_u(&j, "height", "the map")?;
    let tw = need_u(&j, "tilewidth", "the map")?;
    let th = need_u(&j, "tileheight", "the map")?;
    let unit = settings.unit.unwrap_or(1.0);
    let px = unit / tw.max(1) as f32; // world units per pixel
    let mut map = TilemapAsset {
        width,
        height,
        tile_width_px: tw,
        tile_height_px: th,
        unit,
        tilesets: Vec::new(),
        layers: Vec::new(),
        objects: Vec::new(),
    };
    let mut sets: Vec<TiledTileset> = Vec::new();
    for t in j
        .get("tilesets")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let firstgid = need_u(t, "firstgid", "a tileset")?;
        let source = st(t, "source");
        let (r, count) = if !source.is_empty() {
            let id = sibling_ogt(&resolve(rel, source));
            if !known(&id) {
                return Err(format!(
                    "the map's tileset '{source}' needs a tileset .ogt at assets/{id} (collision, animation and terrains live there)"
                ));
            }
            (TilesetRef::Asset(id), u32::MAX)
        } else {
            let name = st(t, "name").to_string();
            let ts = tiled_embedded(rel, t, &name)?;
            let c = ts.count();
            (TilesetRef::Embedded(ts), c)
        };
        sets.push(TiledTileset {
            firstgid,
            index: map.tilesets.len() as u32,
            count,
        });
        map.tilesets.push(r);
    }
    sets.sort_by_key(|s| s.firstgid);
    let mut tile_layers = 0i32;
    walk_tiled(
        j.get("layers")
            .and_then(Value::as_array)
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        glam::Vec2::ZERO,
        &mut map,
        &sets,
        px,
        &mut tile_layers,
    )?;
    Ok(map)
}

fn walk_tiled(
    layers: &[Value],
    parent_offset: glam::Vec2,
    map: &mut TilemapAsset,
    sets: &[TiledTileset],
    px: f32,
    tile_layers: &mut i32,
) -> Result<(), String> {
    for l in layers {
        let name = st(l, "name").to_string();
        let offset = parent_offset
            + glam::Vec2::new(
                f(l, "offsetx", 0.0) as f32 * px,
                -(f(l, "offsety", 0.0) as f32) * px,
            );
        match st(l, "type") {
            "group" => walk_tiled(
                l.get("layers")
                    .and_then(Value::as_array)
                    .map(|a| a.as_slice())
                    .unwrap_or(&[]),
                offset,
                map,
                sets,
                px,
                tile_layers,
            )?,
            "tilelayer" => {
                let gids = tiled_data(l, &name)?;
                let n = (map.width * map.height) as usize;
                if gids.len() != n {
                    return Err(format!(
                        "layer '{name}' has {} cells, the map {n}",
                        gids.len()
                    ));
                }
                let props = tiled_props(l);
                let terrain = props
                    .get("autotile")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                let mut layer = Layer::new(name.clone(), map.width, map.height, *tile_layers);
                *tile_layers += 1;
                let mut tileset: Option<u32> = None;
                for (i, &g) in gids.iter().enumerate() {
                    let id = g & 0x1FFF_FFFF;
                    if id == 0 {
                        continue;
                    }
                    if g & FLIP_DIAGONAL != 0 {
                        return Err(format!(
                            "layer '{name}' has a rotated (diagonally flipped) tile at cell {}; maps support horizontal and vertical flips only",
                            i
                        ));
                    }
                    let set = sets
                        .iter()
                        .rev()
                        .find(|s| s.firstgid <= id)
                        .ok_or_else(|| format!("layer '{name}': gid {id} belongs to no tileset"))?;
                    match tileset {
                        None => tileset = Some(set.index),
                        Some(t) if t != set.index => {
                            return Err(format!(
                            "layer '{name}' mixes tiles of two tilesets; a layer uses one tileset"
                        ))
                        }
                        _ => {}
                    }
                    let local = id - set.firstgid;
                    if local >= set.count {
                        return Err(format!(
                            "layer '{name}': gid {id} is past its tileset's tiles"
                        ));
                    }
                    layer.cells[i] = if terrain.is_some() {
                        1
                    } else {
                        if local + 1 > TILE_MASK {
                            return Err(format!("layer '{name}': tile {local} is too large"));
                        }
                        (local + 1) | (g & (FLIP_X | FLIP_Y))
                    };
                }
                layer.tileset = tileset.unwrap_or(0);
                layer.terrain = terrain;
                layer.offset = offset;
                layer.parallax =
                    glam::Vec2::new(f(l, "parallaxx", 1.0) as f32, f(l, "parallaxy", 1.0) as f32);
                layer.opacity = f(l, "opacity", 1.0) as f32;
                layer.visible = l.get("visible").and_then(Value::as_bool).unwrap_or(true);
                layer.collision = props
                    .get("collision")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                layer.repeat_x = l.get("repeatx").and_then(Value::as_bool).unwrap_or(false)
                    || props
                        .get("repeatx")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                layer.repeat_y = l.get("repeaty").and_then(Value::as_bool).unwrap_or(false)
                    || props
                        .get("repeaty")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                if let Some(z) = props.get("z").and_then(Value::as_i64) {
                    layer.z = z as i32;
                }
                if map.tilesets.is_empty() {
                    return Err(format!(
                        "layer '{name}' has tiles but the map has no tileset"
                    ));
                }
                map.layers.push(layer);
            }
            "objectgroup" => {
                for o in l
                    .get("objects")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    let (w, h) = (f(o, "width", 0.0) as f32, f(o, "height", 0.0) as f32);
                    let mut y = f(o, "y", 0.0) as f32;
                    if o.get("gid").is_some() {
                        y -= h; // tile objects are anchored at their bottom-left
                    }
                    let kind = if !st(o, "type").is_empty() {
                        st(o, "type")
                    } else {
                        st(o, "class")
                    };
                    map.objects.push(MapObject {
                        name: st(o, "name").to_string(),
                        kind: kind.to_string(),
                        x: f(o, "x", 0.0) as f32 * px + offset.x,
                        y: -y * px + offset.y,
                        width: w * px,
                        height: h * px,
                        properties: tiled_props(o)
                            .into_iter()
                            .filter_map(|(k, v)| Some((k, value_text(&v)?)))
                            .collect(),
                    });
                }
            }
            _ => {} // image layers draw nothing here
        }
    }
    Ok(())
}

fn tiled_data(l: &Value, name: &str) -> Result<Vec<u32>, String> {
    let compression = st(l, "compression");
    if !compression.is_empty() {
        return Err(format!(
            "layer '{name}' is {compression}-compressed: set Map > Map Properties > Tile Layer Format to CSV or Base64 (uncompressed) in Tiled and save again"
        ));
    }
    match l.get("data") {
        Some(Value::Array(a)) => a
            .iter()
            .map(|v| {
                v.as_u64()
                    .map(|x| x as u32)
                    .ok_or_else(|| format!("layer '{name}': data holds a non-number"))
            })
            .collect(),
        Some(Value::String(s)) if st(l, "encoding") == "base64" => {
            let b = base64(s)?;
            if b.len() % 4 != 0 {
                return Err(format!(
                    "layer '{name}': base64 data is not whole u32 cells"
                ));
            }
            Ok(b.chunks_exact(4)
                .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect())
        }
        _ => Err(format!(
            "layer '{name}' has no data (chunked layers of infinite maps are not supported)"
        )),
    }
}

// ── LDtk ─────────────────────────────────────────────────────────────────

/// An LDtk project (`.ldtk`): one level into a map.
pub fn import_ldtk(
    rel: &str,
    text: &str,
    settings: &MapSettings,
    known: &dyn Fn(&str) -> bool,
) -> Result<TilemapAsset, String> {
    let j: Value = serde_json::from_str(text).map_err(|e| format!("LDtk JSON: {e}"))?;
    if j.get("externalLevels").and_then(Value::as_bool) == Some(true) {
        return Err("the project saves levels in separate files: untick 'Save levels to separate files' in LDtk".into());
    }
    let levels = j
        .get("levels")
        .and_then(Value::as_array)
        .ok_or("the project has no levels")?;
    let level = match &settings.level {
        Some(id) => levels
            .iter()
            .find(|l| st(l, "identifier") == id)
            .ok_or_else(|| format!("the project has no level '{id}'"))?,
        None => levels.first().ok_or("the project has no levels")?,
    };
    let instances = level
        .get("layerInstances")
        .and_then(Value::as_array)
        .ok_or("the level has no layer instances")?;
    let grid = instances
        .iter()
        .find_map(|l| u(l, "__gridSize"))
        .or_else(|| u(&j, "defaultGridSize"))
        .ok_or("the level's layers name no grid size")? as u32;
    let width = need_u(level, "pxWid", "the level")?.div_ceil(grid);
    let height = need_u(level, "pxHei", "the level")?.div_ceil(grid);
    let unit = settings.unit.unwrap_or(1.0);
    let px = unit / grid as f32;
    let solid = if settings.solid.is_empty() {
        vec![1]
    } else {
        settings.solid.clone()
    };
    let defs: Vec<&Value> = j
        .get("defs")
        .and_then(|d| d.get("tilesets"))
        .and_then(Value::as_array)
        .map(|a| a.iter().collect())
        .unwrap_or_default();
    let mut map = TilemapAsset {
        width,
        height,
        tile_width_px: grid,
        tile_height_px: grid,
        unit,
        tilesets: Vec::new(),
        layers: Vec::new(),
        objects: Vec::new(),
    };
    // Tileset uid → (index in map.tilesets, columns, grid).
    let mut by_uid: BTreeMap<u64, (u32, u32, u32)> = BTreeMap::new();
    let mut collision_set: Option<u32> = None;
    let n = (width * height) as usize;
    // LDtk lists the top layer first: draw them in reverse.
    for (z, l) in instances.iter().rev().enumerate() {
        let name = st(l, "__identifier").to_string();
        if u(l, "__gridSize").is_some_and(|g| g as u32 != grid) {
            return Err(format!(
                "layer '{name}' has another grid size than the level's ({grid} px)"
            ));
        }
        let offset = glam::Vec2::new(
            f(l, "__pxTotalOffsetX", 0.0) as f32 * px,
            -(f(l, "__pxTotalOffsetY", 0.0) as f32) * px,
        );
        let visible = l.get("visible").and_then(Value::as_bool).unwrap_or(true);
        let opacity = f(l, "__opacity", 1.0) as f32;
        let ty = st(l, "__type");
        if ty == "Entities" {
            for e in l
                .get("entityInstances")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let p = e.get("px").and_then(Value::as_array);
                let (ex, ey) = (
                    p.and_then(|a| a.first())
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0) as f32,
                    p.and_then(|a| a.get(1))
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0) as f32,
                );
                let (w, h) = (f(e, "width", 0.0) as f32, f(e, "height", 0.0) as f32);
                let pivot = e.get("__pivot").and_then(Value::as_array);
                let (pvx, pvy) = (
                    pivot
                        .and_then(|a| a.first())
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0) as f32,
                    pivot
                        .and_then(|a| a.get(1))
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0) as f32,
                );
                let mut properties = Vec::new();
                let mut name = st(e, "iid").to_string();
                for fi in e
                    .get("fieldInstances")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    let key = st(fi, "__identifier").to_string();
                    if let Some(v) = fi.get("__value").and_then(value_text) {
                        if key == "name" {
                            name = v.clone();
                        }
                        properties.push((key, v));
                    }
                }
                map.objects.push(MapObject {
                    name,
                    kind: st(e, "__identifier").to_string(),
                    x: (ex - pvx * w) * px + offset.x,
                    y: -(ey - pvy * h) * px + offset.y,
                    width: w * px,
                    height: h * px,
                    properties,
                });
            }
            continue;
        }
        let key = if ty == "Tiles" {
            "gridTiles"
        } else {
            "autoLayerTiles"
        };
        let tiles = l
            .get(key)
            .and_then(Value::as_array)
            .filter(|a| !a.is_empty());
        if let (Some(tiles), Some(uid)) = (tiles, u(l, "__tilesetDefUid")) {
            let (index, columns, tgrid) = match by_uid.get(&uid) {
                Some(v) => *v,
                None => {
                    let d = defs
                        .iter()
                        .find(|d| u(d, "uid") == Some(uid))
                        .ok_or_else(|| format!("layer '{name}' names tileset {uid}, which the project does not define"))?;
                    let tgrid = need_u(d, "tileGridSize", "a tileset")?;
                    let image = resolve(rel, st(d, "relPath"));
                    let sibling = sibling_ogt(&image);
                    let columns;
                    let r = if known(&sibling) {
                        columns = need_u(d, "__cWid", "a tileset")?;
                        TilesetRef::Asset(sibling)
                    } else {
                        columns = need_u(d, "__cWid", "a tileset")?.max(1);
                        let ts = Tileset {
                            texture: image,
                            tile_width: tgrid,
                            tile_height: tgrid,
                            columns,
                            rows: need_u(d, "__cHei", "a tileset")?.max(1),
                            margin: u(d, "padding").unwrap_or(0) as u32,
                            spacing: u(d, "spacing").unwrap_or(0) as u32,
                            filter: Filter::Nearest,
                            tiles: BTreeMap::new(),
                            terrains: Vec::new(),
                        };
                        TilesetRef::Embedded(ts)
                    };
                    let v = (map.tilesets.len() as u32, columns, tgrid);
                    map.tilesets.push(r);
                    by_uid.insert(uid, v);
                    v
                }
            };
            let mut layer = Layer::new(name.clone(), width, height, z as i32);
            let (pad, spc) = defs
                .iter()
                .find(|d| u(d, "uid") == Some(uid))
                .map(|d| {
                    (
                        u(d, "padding").unwrap_or(0) as u32,
                        u(d, "spacing").unwrap_or(0) as u32,
                    )
                })
                .unwrap_or((0, 0));
            for t in tiles {
                let p = t.get("px").and_then(Value::as_array);
                let (x, y) = (
                    p.and_then(|a| a.first())
                        .and_then(Value::as_u64)
                        .unwrap_or(0) as u32
                        / grid,
                    p.and_then(|a| a.get(1))
                        .and_then(Value::as_u64)
                        .unwrap_or(0) as u32
                        / grid,
                );
                if x >= width || y >= height {
                    continue;
                }
                let id = match u(t, "t") {
                    Some(id) => id as u32,
                    None => {
                        let s = t.get("src").and_then(Value::as_array);
                        let sx = s
                            .and_then(|a| a.first())
                            .and_then(Value::as_u64)
                            .unwrap_or(0) as u32;
                        let sy = s
                            .and_then(|a| a.get(1))
                            .and_then(Value::as_u64)
                            .unwrap_or(0) as u32;
                        let step = tgrid + spc;
                        (sy.saturating_sub(pad) / step) * columns + sx.saturating_sub(pad) / step
                    }
                };
                let fl = u(t, "f").unwrap_or(0);
                let mut c = id + 1;
                if fl & 1 != 0 {
                    c |= FLIP_X;
                }
                if fl & 2 != 0 {
                    c |= FLIP_Y;
                }
                layer.cells[(y * width + x) as usize] = c;
            }
            layer.tileset = index;
            layer.offset = offset;
            layer.opacity = opacity;
            layer.visible = visible;
            map.layers.push(layer);
        }
        if ty == "IntGrid" {
            let csv = l.get("intGridCsv").and_then(Value::as_array);
            if let Some(csv) = csv {
                if csv.len() != n {
                    return Err(format!(
                        "IntGrid layer '{name}' has {} cells, the level {n}",
                        csv.len()
                    ));
                }
                let mut layer = Layer::new(format!("{name}.collision"), width, height, z as i32);
                let mut any = false;
                for (i, v) in csv.iter().enumerate() {
                    if v.as_i64().is_some_and(|v| solid.contains(&v)) {
                        layer.cells[i] = 1;
                        any = true;
                    }
                }
                if any {
                    let index = *collision_set.get_or_insert_with(|| {
                        let mut tiles = BTreeMap::new();
                        tiles.insert(
                            0,
                            TileDef {
                                collision: Some(Collision::Full),
                                anim: None,
                            },
                        );
                        map.tilesets.push(TilesetRef::Embedded(Tileset {
                            texture: String::new(),
                            tile_width: grid,
                            tile_height: grid,
                            columns: 1,
                            rows: 1,
                            margin: 0,
                            spacing: 0,
                            filter: Filter::Nearest,
                            tiles,
                            terrains: Vec::new(),
                        }));
                        map.tilesets.len() as u32 - 1
                    });
                    layer.tileset = index;
                    layer.offset = offset;
                    layer.visible = false;
                    layer.collision = true;
                    map.layers.push(layer);
                }
            }
        }
    }
    Ok(map)
}

/// What a tileset's stamp covers beyond its bytes: whether its texture exists.
pub fn references_of_tileset(bytes: &[u8], known: &dyn Fn(&str) -> bool) -> String {
    std::str::from_utf8(bytes)
        .ok()
        .and_then(|t| Tileset::parse(t).ok())
        .map(|t| format!("{}={}", t.texture, known(asset_id(&t.texture))))
        .unwrap_or_default()
}
