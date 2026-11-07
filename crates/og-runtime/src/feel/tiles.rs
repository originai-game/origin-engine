//! Flipbook and tilemap loading, tilemap calls and collider state.

use super::*;

// ── loading ────────────────────────────────────────────────────────────────

/// The sampling a sheet or tileset states for its texture; it wins over the
/// texture's own (`kind pixel` loads nearest, `filter linear` smooths it).
pub(crate) fn set_filter(eng: &Engine, texture: usize, nearest: bool) {
    let mut rs = eng.render.borrow_mut();
    if let Some(&h) = rs.textures.get(texture) {
        let filter = if nearest {
            og_render::TextureFilter::Nearest
        } else {
            og_render::TextureFilter::Linear
        };
        rs.scene.set_texture_filter(h, filter);
    }
}

pub(crate) fn load_flipbook(eng: &Engine, path: &str) -> Result<Value, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    let key = format!("flipbook:{id}");
    if let Some(v) = eng.state.borrow().feel_assets.loaded.get(&key) {
        return Ok(v.clone());
    }
    let sheet = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle
            .flipbook(&id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    // An `array true` sheet draws from the texture array the cook cut its
    // frames into; any other from regions of its texture.
    let image = if sheet.array {
        let array_id = format!("{id}#array");
        let i = super::assets::load_texture_array_index(eng, &array_id)
            .map_err(|e| format!("{id}: texture array {array_id}: {e}"))?;
        SheetImage::Array(i)
    } else {
        let texture = super::assets::load_texture(eng, &sheet.texture)
            .map_err(|e| format!("{id}: texture {}: {e}", sheet.texture))?;
        set_filter(eng, texture, sheet.filter == og_anim::SheetFilter::Nearest);
        SheetImage::Texture(texture)
    };
    let feel = feel_of(eng)?;
    let mut st = eng.state.borrow_mut();
    let handle = st.feel_assets.mint();
    feel.flipbooks
        .add_sheet(handle, sheet.clone())
        .map_err(|e| format!("{id}: {}", e.reason()))?;
    lock(&feel.sheet_textures).insert(handle, image);
    let clips: Vec<Value> = sheet
        .clips
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let mut events: Vec<(u32, String)> = c.events.clone();
            og_core::sort::sort(&mut events);
            json!({
                "clip": i,
                "name": c.name,
                "frames": c.frames.len(),
                "duration": c.duration(),
                "mode": c.mode.name(),
                "events": events.into_iter().map(|e| e.1).collect::<Vec<_>>(),
            })
        })
        .collect();
    let first = sheet.frames.first().map_or([0; 4], |f| f.rect);
    let answer = json!({
        "sheet": handle as f64,
        "texture": match image {
            SheetImage::Texture(t) => super::assets::handle(t) as f64,
            SheetImage::Array(_) => 0.0,
        },
        "array": match image {
            SheetImage::Texture(_) => 0.0,
            SheetImage::Array(a) => super::assets::handle(a) as f64,
        },
        "clips": clips,
        "frames": sheet.frames.len(),
        "frameWidth": first[2],
        "frameHeight": first[3],
    });
    st.feel_assets.sheets.insert(handle, (sheet, Some(image)));
    st.feel_assets.loaded.insert(key, answer.clone());
    Ok(answer)
}

pub fn og_load_flipbook(world: u32, path: String) -> String {
    wire(engine(world).and_then(|eng| load_flipbook(&eng, &path)))
}

/// A map's tilesets and their textures, loaded, and the map checked
/// against them (terrains, every cell): what drawing and colliders read.
fn build_map(eng: &Engine, id: &str, asset: TilemapAsset) -> Result<Arc<MapAsset>, String> {
    let tilesets = {
        let assets = eng.assets.borrow();
        let mut tilesets = Vec::new();
        for t in &asset.tilesets {
            tilesets.push(Arc::new(match t {
                og_tilemap::TilesetRef::Embedded(t) => t.clone(),
                og_tilemap::TilesetRef::Asset(tid) => {
                    let (b, bundle) = assets
                        .blob(tid)
                        .map_err(|e| format!("{id}: tileset {tid}: {e}"))?;
                    bundle
                        .tileset(tid, b.bytes())
                        .map_err(|e| format!("{id}: tileset {tid}: {e}"))?
                }
            }));
        }
        tilesets
    };
    // Check the terrains and every cell against the tilesets now, not at the first tick.
    MapState::new(&asset, &tilesets).map_err(|e| format!("{id}: {}", e.reason()))?;
    let mut textures = Vec::new();
    for t in &tilesets {
        if t.texture.is_empty() {
            textures.push(None);
            continue;
        }
        let i = super::assets::load_texture(eng, &t.texture)
            .map_err(|e| format!("{id}: texture {}: {e}", t.texture))?;
        set_filter(eng, i, t.filter == og_tilemap::Filter::Nearest);
        textures.push(Some(i));
    }
    Ok(Arc::new(MapAsset {
        asset,
        tilesets,
        textures,
    }))
}

/// What og.load.tilemap and og.tilemap.create answer (TilemapInfo).
fn map_info(handle: u64, asset: &TilemapAsset) -> Value {
    let tile = asset.tile_size();
    let layers: Vec<Value> = asset
        .layers
        .iter()
        .enumerate()
        .map(|(i, l)| {
            json!({
                "name": l.name, "index": i, "z": l.z,
                "parallax": { "x": l.parallax.x, "y": l.parallax.y },
                "collision": l.collision, "terrain": l.terrain.clone().unwrap_or_default(), "visible": l.visible,
                "lit": l.lit,
                "colliderLayers": l.collider_layers, "colliderMask": l.collider_mask,
                "cellColliders": l.cell_colliders,
            })
        })
        .collect();
    let objects: Vec<Value> = asset
        .objects
        .iter()
        .map(|o| {
            let mut props = serde_json::Map::new();
            props.extend(o.properties.iter().map(|(k, v)| (k.clone(), json!(v))));
            json!({
                "name": o.name, "kind": o.kind,
                "position": { "x": o.x, "y": o.y },
                "size": { "x": o.width, "y": o.height },
                "center": { "x": o.x + o.width / 2.0, "y": o.y - o.height / 2.0 },
                "properties": props,
            })
        })
        .collect();
    json!({
        "map": handle as f64,
        "width": asset.width, "height": asset.height,
        "tileWidth": tile.x, "tileHeight": tile.y,
        "layers": layers, "objects": objects,
        "orientation": asset.orientation.name(),
    })
}

fn load_tilemap(eng: &Engine, path: &str) -> Result<Value, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    let key = format!("tilemap:{id}");
    if let Some(v) = eng.state.borrow().feel_assets.loaded.get(&key) {
        return Ok(v.clone());
    }
    let asset = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle
            .tilemap(&id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    let map = build_map(eng, &id, asset)?;
    let feel = feel_of(eng)?;
    let mut st = eng.state.borrow_mut();
    let handle = st.feel_assets.mint();
    let answer = map_info(handle, &map.asset);
    st.feel_assets.maps.insert(handle, map.clone());
    lock(&feel.tiles).maps.insert(handle, map);
    st.feel_assets.loaded.insert(key, answer.clone());
    Ok(answer)
}

pub fn og_load_tilemap(world: u32, path: String) -> String {
    wire(engine(world).and_then(|eng| load_tilemap(&eng, &path)))
}

// ── tilemap calls ──────────────────────────────────────────────────────────

fn with_live<T>(
    world: u32,
    entity: f64,
    f: impl FnOnce(&mut LiveMap) -> Result<T, String>,
) -> Result<T, String> {
    let eng = engine(world)?;
    let feel = feel_of(&eng)?;
    let mut t = lock(&feel.tiles);
    let live = t
        .live
        .get_mut(&(entity as u64))
        .ok_or_else(|| format!("entity {entity} has no adopted og.Tilemap (og.tilemap adopts a map in the first tick it exists)"))?;
    f(live)
}

fn layer_index(live: &LiveMap, name: &str) -> Result<usize, String> {
    live.state.asset().layer(name).ok_or_else(|| {
        let names: Vec<_> = live
            .state
            .asset()
            .layers
            .iter()
            .map(|l| l.name.as_str())
            .collect();
        format!(
            "the map has no layer '{name}' (its layers: {})",
            names.join(", ")
        )
    })
}

pub fn og_tilemap_set_tile(world: u32, desc: Option<String>) -> String {
    wire_void((|| {
        let d = parse(desc.as_deref())?;
        let entity = d["entity"]
            .as_f64()
            .ok_or("desc.entity is the entity with the og.Tilemap")?;
        with_live(world, entity, |live| {
            let layer = layer_index(live, d["layer"].as_str().unwrap_or_default())?;
            let (x, y) = (
                d["x"].as_u64().unwrap_or(0) as u32,
                d["y"].as_u64().unwrap_or(0) as u32,
            );
            let tile = d["tile"].as_i64().unwrap_or(-1) as i32;
            live.state.set(layer, x, y, tile).map_err(err)?;
            if live.state.asset().layers[layer].collision {
                live.dirty = true;
            }
            Ok(())
        })
    })())
}

pub fn og_tilemap_fill(world: u32, desc: Option<String>) -> String {
    wire_void((|| {
        let d = parse(desc.as_deref())?;
        let entity = d["entity"]
            .as_f64()
            .ok_or("desc.entity is the entity with the og.Tilemap")?;
        let tiles: Vec<i32> = d["tiles"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|v| {
                        v.as_i64()
                            .unwrap_or(-1)
                            .clamp(i32::MIN as i64, i32::MAX as i64) as i32
                    })
                    .collect()
            })
            .unwrap_or_default();
        with_live(world, entity, |live| {
            let layer = layer_index(live, d["layer"].as_str().unwrap_or_default())?;
            let (x, y, width) = (
                d["x"].as_u64().unwrap_or(0).min(u32::MAX as u64) as u32,
                d["y"].as_u64().unwrap_or(0).min(u32::MAX as u64) as u32,
                d["width"].as_u64().unwrap_or(0).min(u32::MAX as u64) as u32,
            );
            live.state.fill(layer, x, y, width, &tiles).map_err(err)?;
            if live.state.asset().layers[layer].collision {
                live.dirty = true;
            }
            Ok(())
        })
    })())
}

/// Run-time map handles: their own range, numbered by the world's counter.
const RUNTIME_MAPS: u64 = 1 << 33;

fn u32_at(v: &Value, key: &str, default: u32) -> u32 {
    v[key]
        .as_u64()
        .map_or(default, |n| n.min(u64::from(u32::MAX)) as u32)
}

/// A TilemapDesc on the wire as an empty map.
fn desc_map(d: &Value) -> Result<TilemapAsset, String> {
    let (w, h) = (u32_at(d, "width", 16), u32_at(d, "height", 16));
    if w == 0 || h == 0 || u64::from(w) * u64::from(h) > 1 << 24 {
        return Err(format!(
            "desc.width x desc.height is {w} x {h}: 1 to 16,777,216 cells"
        ));
    }
    let orientation = match d["orientation"].as_str() {
        None => og_tilemap::Orientation::Orthogonal,
        Some(o) => og_tilemap::Orientation::named(o).ok_or_else(|| {
            format!(
                "desc.orientation '{o}' is not orthogonal, isometric, staggered, hex or hexFlat"
            )
        })?,
    };
    let tilesets: Vec<og_tilemap::TilesetRef> = d["tilesets"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|t| {
                    og_tilemap::TilesetRef::Asset(
                        og_bundle::runtime::asset_id(t.as_str().unwrap_or_default()).to_string(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let mut layers: Vec<og_tilemap::Layer> = Vec::new();
    for l in d["layers"].as_array().into_iter().flatten() {
        let name = l["name"].as_str().unwrap_or_default();
        if name.is_empty() || layers.iter().any(|o| o.name == name) {
            return Err(format!(
                "every layer needs a name of its own ('{name}' is empty or taken)"
            ));
        }
        let mut layer = og_tilemap::Layer::new(
            name,
            w,
            h,
            l["z"]
                .as_i64()
                .unwrap_or(0)
                .clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        );
        layer.tileset = u32_at(l, "tileset", 0);
        let terrain = l["terrain"].as_str().unwrap_or_default();
        layer.terrain = (!terrain.is_empty()).then(|| terrain.to_string());
        layer.collision = l["collision"].as_bool().unwrap_or(false);
        layer.collider_layers = u32_at(l, "colliderLayers", 0);
        layer.collider_mask = u32_at(l, "colliderMask", 0);
        layer.cell_colliders = l["cellColliders"].as_bool().unwrap_or(false);
        layer.visible = l["visible"].as_bool().unwrap_or(true);
        layers.push(layer);
    }
    let asset = TilemapAsset {
        orientation,
        width: w,
        height: h,
        tile_width_px: u32_at(d, "tileWidth", 16),
        tile_height_px: u32_at(d, "tileHeight", 16),
        unit: d["unit"].as_f64().map_or(1.0, |u| u as f32),
        tilesets,
        layers,
        objects: Vec::new(),
    };
    asset.validate().map_err(err)?;
    Ok(asset)
}

pub fn og_tilemap_create(world: u32, desc: Option<String>) -> String {
    wire((|| {
        let eng = engine(world)?;
        let feel = feel_of(&eng)?;
        let asset = desc_map(&parse(desc.as_deref())?)?;
        let map = build_map(&eng, "og.tilemap.create", asset)?;
        let mut t = lock(&feel.tiles);
        t.next_runtime += 1;
        let handle = RUNTIME_MAPS | t.next_runtime;
        t.maps.insert(handle, map.clone());
        t.runtime.insert(handle);
        drop(t);
        let answer = map_info(handle, &map.asset);
        eng.state.borrow_mut().feel_assets.maps.insert(handle, map);
        Ok(answer)
    })())
}

pub fn og_tilemap_resize(world: u32, desc: Option<String>) -> String {
    wire_void((|| {
        let eng = engine(world)?;
        let feel = feel_of(&eng)?;
        let d = parse(desc.as_deref())?;
        let handle = d["map"].as_f64().unwrap_or(0.0) as u64;
        let (w, h) = (u32_at(&d, "width", 16), u32_at(&d, "height", 16));
        let clamp = |k: &str| d[k].as_i64().unwrap_or(0).clamp(-(1 << 24), 1 << 24) as i32;
        let (dx, dy) = (clamp("x"), clamp("y"));
        let mut t = lock(&feel.tiles);
        let old = t
            .maps
            .get(&handle)
            .cloned()
            .ok_or_else(|| format!("desc.map {handle} is not a loaded or created map"))?;
        let asset = old.asset.resized(w, h, dx, dy).map_err(err)?;
        // Every placed copy first, so a refusal leaves them all as they were.
        let mut states = Vec::new();
        for (&k, live) in t.live.iter().filter(|(_, l)| l.map == handle) {
            states.push((k, live.state.resize(&asset, dx, dy).map_err(err)?));
        }
        for (k, state) in states {
            let live = t.live.get_mut(&k).expect("listed above");
            live.state = state;
            live.dirty = true;
        }
        let map = Arc::new(MapAsset {
            asset,
            tilesets: old.tilesets.clone(),
            textures: old.textures.clone(),
        });
        if !t.runtime.contains(&handle) {
            t.originals.insert(handle, old);
        }
        t.runtime.insert(handle);
        t.maps.insert(handle, map.clone());
        drop(t);
        let mut st = eng.state.borrow_mut();
        let info = map_info(handle, &map.asset);
        for v in st.feel_assets.loaded.values_mut() {
            if v["map"].as_f64() == Some(handle as f64) && v.get("tileWidth").is_some() {
                *v = info.clone();
            }
        }
        st.feel_assets.maps.insert(handle, map);
        Ok(())
    })())
}

pub fn og_tilemap_neighbours(world: u32, map: f64, x: u32, y: u32) -> String {
    let cells = engine(world)
        .and_then(|eng| feel_of(&eng))
        .ok()
        .and_then(|feel| lock(&feel.tiles).maps.get(&(map as u64)).cloned())
        .map(|m| og_tilemap::neighbours(&m.asset, x, y))
        .unwrap_or_default();
    json!(
        cells
            .iter()
            .map(|&(x, y)| json!({ "x": x, "y": y, "inside": true }))
            .collect::<Vec<_>>()
    )
    .to_string()
}

pub fn og_tilemap_get_tile(world: u32, entity: f64, layer: String, x: u32, y: u32) -> i32 {
    with_live(world, entity, |live| {
        Ok(live.state.get(layer_index(live, &layer)?, x, y))
    })
    .unwrap_or(-1)
}

pub fn og_tilemap_cell_at(world: u32, entity: f64, point: String) -> String {
    let p = parse(Some(&point))
        .map(|v| v3(&v, Vec3::ZERO))
        .unwrap_or_default();
    let cell = with_live(world, entity, |live| {
        let local = Vec2::new(p.x - live.origin.x, p.y - live.origin.y);
        Ok(og_tilemap::world_to_cell(live.state.asset(), local))
    })
    .ok()
    .flatten();
    match cell {
        Some((x, y)) => json!({ "x": x, "y": y, "inside": true }),
        None => json!({ "x": 0, "y": 0, "inside": false }),
    }
    .to_string()
}

pub fn og_tilemap_cell_center(world: u32, entity: f64, x: u32, y: u32) -> String {
    let c = with_live(world, entity, |live| {
        let c = og_tilemap::cell_center(live.state.asset(), x, y);
        Ok(Vec3::new(
            live.origin.x + c.x,
            live.origin.y + c.y,
            live.origin.z,
        ))
    })
    .unwrap_or(Vec3::ZERO);
    json!({ "x": c.x, "y": c.y, "z": c.z }).to_string()
}

// ── the og.tilemap system ──────────────────────────────────────────────────

fn encode_value(l: &Layout, v: &Value) -> og_core::Result<Vec<u8>> {
    l.encode(None, Some(v))
        .map_err(|e| Error::new(ErrorKind::Invalid, e))
}

/// A collider entity's packed components.
type Spawn = Vec<(SchemaId, Vec<u8>)>;

/// The physics mesh of a merged solid region (og_tilemap::map_regions): in
/// 2D its triangles (a 2D triangle mesh collides as its outline), in 3D the
/// closed prism over them, 1 deep about the map's z (caps, and walls along
/// the outline). Its handle is generated ([`og_physics::GENERATED_MESH`])
/// from the geometry, so equal regions share one mesh and a replay or a
/// restore names the same handle.
fn region_mesh(region: og_tilemap::MapRegion, profile: u32) -> (u64, og_physics::MeshDesc) {
    let flat = og_physics::MeshDesc {
        positions: region
            .positions
            .iter()
            .map(|p| Vec3::new(p.x, p.y, 0.0))
            .collect(),
        indices: region.triangles,
    };
    let mesh = if profile == og_physics::PROFILE_3D {
        let mut positions = Vec::with_capacity(region.positions.len() * 2);
        for p in &region.positions {
            positions.push(Vec3::new(p.x, p.y, -0.5));
            positions.push(Vec3::new(p.x, p.y, 0.5));
        }
        let mut indices = Vec::new();
        for &[a, b, c] in &flat.indices {
            indices.push([2 * a + 1, 2 * b + 1, 2 * c + 1]);
            indices.push([2 * a, 2 * c, 2 * b]);
        }
        for ids in og_physics::outline(&flat) {
            for k in 0..ids.len() {
                let (a, b) = (ids[k], ids[(k + 1) % ids.len()]);
                indices.push([2 * a, 2 * b, 2 * b + 1]);
                indices.push([2 * a, 2 * b + 1, 2 * a + 1]);
            }
        }
        og_physics::MeshDesc { positions, indices }
    } else {
        flat
    };
    // FNV-1a over the geometry's 32-bit words.
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |x: u32| {
        h = (h ^ x as u64).wrapping_mul(0x0000_0100_0000_01b3);
    };
    for c in mesh.positions.iter().flat_map(|p| p.to_array()) {
        eat(c.to_bits());
    }
    for &i in mesh.indices.iter().flatten() {
        eat(i);
    }
    let handle = og_physics::GENERATED_MESH | (h & (og_physics::GENERATED_MESH - 1));
    (handle, mesh)
}

pub(super) fn tilemap_system(
    ctx: &mut SystemContext<'_>,
    tiles: &Mutex<TileShared>,
    r: &Records,
    physics: &og_physics::PhysicsSystem,
    profile: u32,
) -> og_core::Result<()> {
    let mut rows = Vec::new();
    for a in ctx.query(0)? {
        rows.extend_from_slice(a.entities());
    }
    let mut t = lock(tiles);
    let mut seen = BTreeSet::new();
    let mut gone: Vec<u64> = Vec::new();
    for e in rows {
        let key = e.to_bits();
        let v = r.tilemap.1.decode(&ctx.get_bytes(e, r.tilemap.0)?);
        let tr = r.transform.1.decode(&ctx.get_bytes(e, r.transform.0)?);
        let map = bits(&v["map"]);
        let Some(asset) = t.maps.get(&map).cloned() else {
            continue;
        };
        seen.insert(key);
        let origin = v3(&tr["position"], Vec3::ZERO);
        let collision = on(&v, "collision", true);
        let layers = v["colliderLayers"].as_u64().unwrap_or(1) as u32;
        let friction = f(&v, "friction", 0.5);
        let fresh = t.live.get(&key).is_none_or(|l| l.map != map);
        if fresh {
            let old = t.live.remove(&key).map(|l| l.colliders).unwrap_or_default();
            gone.extend(old);
            let state = MapState::new(&asset.asset, &asset.tilesets)?;
            t.live.insert(
                key,
                LiveMap {
                    map,
                    state,
                    colliders: Vec::new(),
                    origin,
                    collision,
                    layers,
                    friction,
                    dirty: true,
                },
            );
        }
        let live = t.live.get_mut(&key).expect("inserted above");
        if live.origin != origin
            || live.collision != collision
            || live.layers != layers
            || live.friction != friction
        {
            live.origin = origin;
            live.collision = collision;
            live.layers = layers;
            live.friction = friction;
            live.dirty = true;
        }
    }
    let dropped: Vec<u64> = t
        .live
        .keys()
        .filter(|k| !seen.contains(k))
        .copied()
        .collect();
    for k in dropped {
        gone.extend(t.live.remove(&k).map(|l| l.colliders).unwrap_or_default());
    }
    let mut spawns: Vec<(u64, Spawn)> = Vec::new();
    // The generated meshes the colliders built now name.
    let mut meshes: BTreeSet<u64> = BTreeSet::new();
    let rebuilt = t.live.values().any(|l| l.dirty) || !gone.is_empty();
    for (&key, live) in t.live.iter_mut().filter(|(_, l)| l.dirty) {
        live.dirty = false;
        gone.append(&mut live.colliders);
        if !live.collision {
            continue;
        }
        for region in og_tilemap::map_regions(&live.state, live.layers) {
            let (p, layers, mask) = (region.origin, region.layers, region.mask);
            let (handle, mesh) = region_mesh(region, profile);
            physics.set_mesh(handle, mesh)?;
            meshes.insert(handle);
            let tj = json!({ "position": { "x": live.origin.x + p.x, "y": live.origin.y + p.y, "z": live.origin.z } });
            let cj = json!({
                "shape": og_physics::SHAPE_TRIMESH, "mesh": handle,
                "friction": live.friction, "layers": layers, "mask": mask,
            });
            spawns.push((
                key,
                vec![
                    (r.transform.0, encode_value(&r.transform.1, &tj)?),
                    (r.collider.0, encode_value(&r.collider.1, &cj)?),
                ],
            ));
        }
        for mc in og_tilemap::map_colliders(&live.state, live.layers) {
            let rect = mc.rect;
            let c = (rect.min + rect.max) * 0.5;
            let mut size = rect.max - rect.min;
            let position =
                json!({ "x": live.origin.x + c.x, "y": live.origin.y + c.y, "z": live.origin.z });
            // A flat-top hexagon is the pointy-top one turned a quarter.
            let (shape, tj) = match mc.shape {
                og_tilemap::MapShape::Box => (0, json!({ "position": position })),
                og_tilemap::MapShape::Diamond => (7, json!({ "position": position })),
                og_tilemap::MapShape::Hexagon { flat: false } => {
                    (8, json!({ "position": position }))
                }
                og_tilemap::MapShape::Hexagon { flat: true } => {
                    size = Vec2::new(size.y, size.x);
                    let q = std::f32::consts::FRAC_1_SQRT_2;
                    (
                        8,
                        json!({ "position": position, "quaternion": { "x": 0.0, "y": 0.0, "z": q, "w": q } }),
                    )
                }
            };
            let cj = json!({
                "shape": shape, "size": { "x": size.x, "y": size.y, "z": 1.0 },
                "friction": live.friction, "layers": mc.layers, "mask": mc.mask,
            });
            spawns.push((
                key,
                vec![
                    (r.transform.0, encode_value(&r.transform.1, &tj)?),
                    (r.collider.0, encode_value(&r.collider.1, &cj)?),
                ],
            ));
        }
    }
    if rebuilt {
        // Generated meshes no collider names any more leave physics: those
        // of the colliders that stay (a map not rebuilt, or restored) are
        // read from their og.Collider, the rest were just built.
        let gone_set: BTreeSet<u64> = gone.iter().copied().collect();
        for live in t.live.values() {
            for &c in &live.colliders {
                let e = Entity::from_bits(c);
                if gone_set.contains(&c) || !ctx.is_alive(e) {
                    continue;
                }
                let v = r.collider.1.decode(&ctx.get_bytes(e, r.collider.0)?);
                let mesh = bits(&v["mesh"]);
                if mesh & og_physics::GENERATED_MESH != 0 {
                    meshes.insert(mesh);
                }
            }
        }
        for unused in t.generated.difference(&meshes).copied().collect::<Vec<_>>() {
            physics.remove_mesh(unused);
        }
        t.generated = meshes;
    }
    for g in gone {
        let e = Entity::from_bits(g);
        if ctx.is_alive(e) {
            ctx.despawn(e)?;
            queued_structural();
        }
    }
    for (key, comps) in spawns {
        let e = ctx.spawn_bytes(comps)?;
        queued_structural();
        if let Some(live) = t.live.get_mut(&key) {
            live.colliders.push(e.to_bits());
        }
    }
    Ok(())
}

/// og.tilemap's state: every adopted map's placement and edits, then (only
/// while a map made or resized at run time exists) a trailer
/// (og_ecs::trailer): section 1 those maps whole, section 2 the create
/// counter.
pub(super) struct TileBlock(pub(super) Arc<Mutex<TileShared>>, pub(super) u32);

const RUNTIME_SECTION: u16 = 1;
const COUNTER_SECTION: u16 = 2;

impl StateBlock for TileBlock {
    fn save(&self, out: &mut Vec<u8>) {
        let t = lock(&self.0);
        out.extend_from_slice(&(t.live.len() as u32).to_le_bytes());
        for (k, l) in &t.live {
            out.extend_from_slice(&k.to_le_bytes());
            out.extend_from_slice(&l.map.to_le_bytes());
            for x in l.origin.to_array() {
                out.extend_from_slice(&x.to_le_bytes());
            }
            out.extend_from_slice(&(l.collision as u32 | (l.dirty as u32) << 1).to_le_bytes());
            out.extend_from_slice(&l.layers.to_le_bytes());
            out.extend_from_slice(&l.friction.to_le_bytes());
            out.extend_from_slice(&(l.colliders.len() as u32).to_le_bytes());
            for c in &l.colliders {
                out.extend_from_slice(&c.to_le_bytes());
            }
            let mut s = Vec::new();
            l.state.encode(&mut s);
            put_bytes(out, &s);
        }
        if t.runtime.is_empty() && t.next_runtime == 0 {
            return;
        }
        let mut maps = Vec::new();
        maps.extend_from_slice(&(t.runtime.len() as u32).to_le_bytes());
        for h in &t.runtime {
            maps.extend_from_slice(&h.to_le_bytes());
            let mut a = Vec::new();
            if let Some(m) = t.maps.get(h) {
                m.asset.encode(&mut a);
            }
            put_bytes(&mut maps, &a);
        }
        og_ecs::trailer::write(
            out,
            &[
                (RUNTIME_SECTION, maps),
                (COUNTER_SECTION, t.next_runtime.to_le_bytes().to_vec()),
            ],
        );
    }

    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn Any + Send>> {
        let (maps, originals) = {
            let t = lock(&self.0);
            (t.maps.clone(), t.originals.clone())
        };
        let mut r = Reader::new(bytes, "og.tilemap");
        let n = r.u32()?;
        type Saved<'a> = (u64, u64, Vec3, u32, u32, f32, Vec<u64>, &'a [u8]);
        let mut saved: Vec<Saved> = Vec::new();
        for _ in 0..n {
            let key = r.u64()?;
            let map = r.u64()?;
            let origin = Vec3::new(r.f32()?, r.f32()?, r.f32()?);
            let flags = r.u32()?;
            let layers = r.u32()?;
            let friction = r.f32()?;
            let colliders = (0..r.u32()?)
                .map(|_| r.u64())
                .collect::<og_core::Result<Vec<_>>>()?;
            saved.push((
                key,
                map,
                origin,
                flags,
                layers,
                friction,
                colliders,
                r.bytes()?,
            ));
        }
        let mut runtime: Vec<(u64, Arc<MapAsset>)> = Vec::new();
        let mut next_runtime = 0;
        for (tag, body) in og_ecs::trailer::read(TILEMAP_SYSTEM, r.rest())? {
            let mut b = Reader::new(body, "og.tilemap");
            match tag {
                RUNTIME_SECTION => {
                    for _ in 0..b.u32()? {
                        let h = b.u64()?;
                        let asset = TilemapAsset::decode(b.bytes()?)?;
                        // The map this session has, when it is the saved one.
                        let have = maps.get(&h).filter(|m| m.asset == asset).cloned();
                        let map = match have {
                            Some(m) => m,
                            None => {
                                let eng = engine(self.1)
                                    .map_err(|e| Error::new(ErrorKind::NotFound, e))?;
                                build_map(&eng, "og.tilemap state block", asset)
                                    .map_err(|e| Error::new(ErrorKind::NotFound, e))?
                            }
                        };
                        runtime.push((h, map));
                    }
                    b.end()?;
                }
                COUNTER_SECTION => {
                    next_runtime = b.u64()?;
                    b.end()?;
                }
                // A later section this engine does not know: skipped.
                _ => {}
            }
        }
        let mut live = IdMap::new();
        for (key, map, origin, flags, layers, friction, colliders, s) in saved {
            // The map as the save saw it: its own copy of a run-time map, a
            // resized map as loaded, or this session's.
            let asset = runtime
                .iter()
                .find(|(h, _)| *h == map)
                .map(|(_, m)| m)
                .or_else(|| originals.get(&map))
                .or_else(|| maps.get(&map))
                .ok_or_else(|| {
                    Error::new(
                        ErrorKind::NotFound,
                        format!("og.tilemap state block: map {map} is not loaded in this engine"),
                    )
                })?;
            let state = MapState::decode(&asset.asset, &asset.tilesets, s)?;
            live.insert(
                key,
                LiveMap {
                    map,
                    state,
                    colliders,
                    origin,
                    collision: flags & 1 != 0,
                    layers,
                    friction,
                    dirty: flags & 2 != 0,
                },
            );
        }
        Ok(Box::new(LiveMaps(live, runtime, next_runtime)))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let LiveMaps(live, runtime, next) = decoded
            .map(|d| *d.downcast::<LiveMaps>().expect("decoded by TileBlock"))
            .unwrap_or_else(|| LiveMaps(IdMap::new(), Vec::new(), 0));
        let mut t = lock(&self.0);
        // A resized loaded map the save does not hold goes back to its load.
        let back: Vec<u64> = t
            .originals
            .keys()
            .filter(|h| !runtime.iter().any(|(r, _)| r == *h))
            .copied()
            .collect();
        let mut maps: Vec<(u64, Arc<MapAsset>)> = Vec::new();
        for h in back {
            if let Some(m) = t.originals.remove(&h) {
                maps.push((h, m));
            }
        }
        t.runtime = runtime.iter().map(|(h, _)| *h).collect();
        t.next_runtime = next;
        maps.extend(runtime);
        for (h, m) in &maps {
            t.maps.insert(*h, m.clone());
        }
        t.live = live;
        drop(t);
        if let Ok(eng) = engine(self.1)
            && let Ok(mut st) = eng.state.try_borrow_mut()
        {
            for (h, m) in maps {
                st.feel_assets.maps.insert(h, m);
            }
        }
    }

    /// Each live map's origin is its entity's og.Transform.position: it
    /// moves with a root, as the Transform does (so the map does not
    /// rebuild its colliders, which the shift moved too).
    fn shift(&mut self, s: &og_ecs::Shift) {
        let mut t = lock(&self.0);
        for (k, l) in t.live.iter_mut() {
            l.origin = s.moved_of(*k, l.origin);
        }
    }
}

struct LiveMaps(IdMap<u64, LiveMap>, Vec<(u64, Arc<MapAsset>)>, u64);
