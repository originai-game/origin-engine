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

fn load_flipbook(eng: &Engine, path: &str) -> Result<Value, String> {
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

fn load_tilemap(eng: &Engine, path: &str) -> Result<Value, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    let key = format!("tilemap:{id}");
    if let Some(v) = eng.state.borrow().feel_assets.loaded.get(&key) {
        return Ok(v.clone());
    }
    let (asset, tilesets) = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        let asset = bundle
            .tilemap(&id, bytes.bytes())
            .map_err(|e| e.to_string())?;
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
        (asset, tilesets)
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
    let feel = feel_of(eng)?;
    let map = Arc::new(MapAsset {
        asset: asset.clone(),
        tilesets,
        textures,
    });
    let mut st = eng.state.borrow_mut();
    let handle = st.feel_assets.mint();
    st.feel_assets.maps.insert(handle, map.clone());
    lock(&feel.tiles).maps.insert(handle, map);
    let answer = json!({
        "map": handle as f64,
        "width": asset.width, "height": asset.height,
        "tileWidth": tile.x, "tileHeight": tile.y,
        "layers": layers, "objects": objects,
    });
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

/// A collider entity's packed components.
type Spawn = Vec<(SchemaId, Vec<u8>)>;

pub(super) fn tilemap_system(
    ctx: &mut SystemContext<'_>,
    tiles: &Mutex<TileShared>,
    r: &Records,
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
    for (&key, live) in t.live.iter_mut().filter(|(_, l)| l.dirty) {
        live.dirty = false;
        gone.append(&mut live.colliders);
        if !live.collision {
            continue;
        }
        for mc in og_tilemap::map_colliders(&live.state, live.layers) {
            let rect = mc.rect;
            let c = (rect.min + rect.max) * 0.5;
            let size = rect.max - rect.min;
            let tj = json!({ "position": { "x": live.origin.x + c.x, "y": live.origin.y + c.y, "z": live.origin.z } });
            let cj = json!({
                "shape": 0, "size": { "x": size.x, "y": size.y, "z": 1.0 },
                "friction": live.friction, "layers": mc.layers, "mask": mc.mask,
            });
            let enc = |l: &Layout, v: &Value| {
                l.encode(None, Some(v))
                    .map_err(|e| Error::new(ErrorKind::Invalid, e))
            };
            spawns.push((
                key,
                vec![
                    (r.transform.0, enc(&r.transform.1, &tj)?),
                    (r.collider.0, enc(&r.collider.1, &cj)?),
                ],
            ));
        }
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

pub(super) struct TileBlock(pub(super) Arc<Mutex<TileShared>>);

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
    }

    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn Any + Send>> {
        let maps = lock(&self.0).maps.clone();
        let mut r = Reader::new(bytes, "og.tilemap");
        let n = r.u32()?;
        let mut live = IdMap::new();
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
            let s = r.bytes()?;
            let asset = maps.get(&map).ok_or_else(|| {
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
        r.end()?;
        Ok(Box::new(LiveMaps(live)))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        lock(&self.0).live = decoded
            .map(|d| d.downcast::<LiveMaps>().expect("decoded by TileBlock").0)
            .unwrap_or_default();
    }
}

struct LiveMaps(IdMap<u64, LiveMap>);
