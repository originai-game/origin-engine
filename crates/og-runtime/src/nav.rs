//! The IDL's `nav` module over og-nav: grids, flow fields, navmeshes
//! (cooked, or built over a flat floor), path queries, obstacles and the
//! agents' paths. The registry is the engine's (`e6::Modules::navs`);
//! og.nav's system and state block are the world's.

use super::engine::{Engine, engine};
use super::{ok, parse, refuse};
use og_core::glam::{Vec2, Vec3};
use og_ecs::{Entity, World};
use og_nav::{
    Algorithm, FlowField, Grid, Heightfield, Nav, NavMesh, NavParams, Obstacle, Plane, Scratch,
};
use serde_json::{Value, json};

fn f(v: &Value, key: &str, default: f32) -> f32 {
    v[key].as_f64().map_or(default, |x| x as f32)
}

fn v3(v: &Value) -> Vec3 {
    Vec3::new(f(v, "x", 0.0), f(v, "y", 0.0), f(v, "z", 0.0))
}

fn j3(v: Vec3) -> Value {
    json!({ "x": v.x, "y": v.y, "z": v.z })
}

fn wire(r: Result<Value, String>) -> String {
    match r {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

fn plane_of(engine: u32) -> Plane {
    super::e6::with(engine, |m| super::e6::plane_of(m.profile)).unwrap_or(Plane::Xz)
}

fn navs(engine: u32) -> Result<og_nav::SharedNavs, String> {
    super::e6::with(engine, |m| m.navs.clone()).ok_or_else(|| format!("no engine {engine}"))
}

fn handle(h: f64) -> u64 {
    if h.is_finite() && h >= 0.0 {
        h as u64
    } else {
        0
    }
}

fn lock<T>(m: &std::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn og_nav_grid(world: u32, desc: Option<String>) -> String {
    wire((|| {
        engine(world)?;
        let d = parse(desc.as_deref())?;
        let w = d["width"].as_u64().unwrap_or(32).clamp(1, 4096) as u32;
        let h = d["height"].as_u64().unwrap_or(32).clamp(1, 4096) as u32;
        let cs = f(&d, "cellSize", 1.0);
        let origin = Vec2::new(f(&d["origin"], "x", 0.0), f(&d["origin"], "y", 0.0));
        let grid = match d["costs"].as_array().filter(|c| !c.is_empty()) {
            None => Grid::new(w, h, cs, origin),
            Some(c) => {
                if c.len() != (w * h) as usize {
                    return Err(format!(
                        "costs has {} values; a {w} × {h} grid needs {} (or none)",
                        c.len(),
                        w * h
                    ));
                }
                let costs = c
                    .iter()
                    .map(|x| x.as_u64().unwrap_or(1).min(255) as u8)
                    .collect();
                Grid::from_costs(w, h, cs, origin, costs).ok_or("the costs do not fit the grid")?
            }
        };
        let n = navs(world)?;
        let h = lock(&n).insert(Nav::Grid(grid));
        Ok(json!(h))
    })())
}

pub fn og_nav_grid_from_tilemap(world: u32, desc: Option<String>) -> String {
    wire(engine(world).and_then(|eng| grid_from_tilemap(&eng, world, desc)))
}

fn grid_from_tilemap(eng: &Engine, world: u32, desc: Option<String>) -> Result<Value, String> {
    if super::engine::running(world).is_some() {
        return Err("og.nav.gridFromTilemap reads the map's components: call it in GameDesc.start or a command, not inside a system".into());
    }
    let d = parse(desc.as_deref())?;
    let e = Entity::from_bits(handle(d["entity"].as_f64().unwrap_or(0.0)));
    let (map, origin) = {
        let st = eng.state.borrow();
        let w = eng.world()?;
        let tm = st
            .layouts
            .get("og.Tilemap")
            .ok_or("this engine has no og.Tilemap")?;
        let tr = st.layouts.get("og.Transform").ok_or("no og.Transform")?;
        let tid = w
            .id_by_name("og.Tilemap")
            .map_err(|e| e.reason().to_string())?;
        let rid = w
            .id_by_name("og.Transform")
            .map_err(|e| e.reason().to_string())?;
        let t = tm.decode(
            &w.get_bytes(e, tid)
                .map_err(|_| "the entity has no og.Tilemap".to_string())?,
        );
        let p = tr.decode(
            &w.get_bytes(e, rid)
                .map_err(|_| "the entity has no og.Transform".to_string())?,
        );
        (
            t["map"].as_f64().map_or(0, |m| m as u64),
            v3(&p["position"]),
        )
    };
    let asset = {
        let st = eng.state.borrow();
        let feel = st.feel.as_ref().ok_or("this engine has no tilemaps")?;
        let tiles = feel.tiles.lock().unwrap_or_else(|e| e.into_inner());
        tiles
            .maps
            .get(&map)
            .cloned()
            .ok_or("the og.Tilemap's map is not loaded: load it with og.load.tilemap")?
    };
    let state = og_tilemap::MapState::new(&asset.asset, &asset.tilesets)
        .map_err(|e| e.reason().to_string())?;
    let names: Vec<String> = d["layers"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let layers: Vec<usize> = (0..asset.asset.layers.len())
        .filter(|&l| {
            if names.is_empty() {
                asset.asset.layers[l].collision
            } else {
                names.contains(&asset.asset.layers[l].name)
            }
        })
        .collect();
    let size = asset.asset.tile_size();
    let (w, h) = (asset.asset.width, asset.asset.height);
    // Cell (0, 0) is the map's bottom-left cell; its lower corner is the map's top-left minus its height.
    let gorigin = Vec2::new(origin.x, origin.y - h as f32 * size.y);
    let mut grid = Grid::new(w, h, size.x, gorigin);
    let mut rects = Vec::new();
    for l in layers {
        rects.extend(og_tilemap::collision_rects(&state, l));
    }
    for y in 0..h {
        for x in 0..w {
            let c = Vec2::new(
                origin.x + (x as f32 + 0.5) * size.x,
                gorigin.y + (y as f32 + 0.5) * size.y,
            );
            let local = c - Vec2::new(origin.x, origin.y);
            if rects.iter().any(|r| {
                local.x > r.min.x && local.x < r.max.x && local.y > r.min.y && local.y < r.max.y
            }) {
                grid.set(x, y, og_nav::BLOCKED);
            }
        }
    }
    let inflate = d["inflate"].as_u64().unwrap_or(0).min(8) as i32;
    if inflate > 0 {
        let base = grid.clone();
        for y in 0..h as i32 {
            for x in 0..w as i32 {
                if base.walkable(x, y) {
                    continue;
                }
                for dy in -inflate..=inflate {
                    for dx in -inflate..=inflate {
                        let (nx, ny) = (x + dx, y + dy);
                        if nx >= 0 && ny >= 0 && nx < w as i32 && ny < h as i32 {
                            grid.set(nx as u32, ny as u32, og_nav::BLOCKED);
                        }
                    }
                }
            }
        }
    }
    let n = navs(world)?;
    let h = lock(&n).insert(Nav::Grid(grid));
    Ok(json!(h))
}

pub fn og_nav_set_cost(world: u32, grid: f64, x: u32, y: u32, cost: u32) -> String {
    match (|| -> Result<(), String> {
        let n = navs(world)?;
        let mut n = lock(&n);
        match n.get_mut(handle(grid)) {
            Some(Nav::Grid(g)) => {
                if g.set(x, y, cost.min(255) as u8) {
                    Ok(())
                } else {
                    Err(format!(
                        "cell {x}, {y} is outside the {} × {} grid",
                        g.width, g.height
                    ))
                }
            }
            _ => Err(format!(
                "{grid} is not a grid (og.nav.grid, og.nav.gridFromTilemap)"
            )),
        }
    })() {
        Ok(()) => super::ok_void(),
        Err(e) => refuse(e),
    }
}

pub fn og_nav_cell_at(world: u32, grid: f64, point: String) -> String {
    let p = parse(Some(&point)).map(|v| v3(&v)).unwrap_or_default();
    let plane = plane_of(world);
    let cell = navs(world)
        .ok()
        .and_then(|n| match lock(&n).get(handle(grid)) {
            Some(Nav::Grid(g)) => g.cell_at(plane.to_plane(p)),
            _ => None,
        });
    match cell {
        Some((x, y)) => json!({ "x": x, "y": y, "inside": true }),
        None => json!({ "x": 0, "y": 0, "inside": false }),
    }
    .to_string()
}

fn path_value(points: &[Vec3], complete: bool, expanded: u32, algorithm: &str) -> Value {
    let length: f32 = points.windows(2).map(|w| w[0].distance(w[1])).sum();
    json!({
        "points": points.iter().map(|p| j3(*p)).collect::<Vec<_>>(),
        "length": length, "complete": complete, "expanded": expanded, "algorithm": algorithm,
    })
}

pub fn og_nav_find_path(world: u32, desc: Option<String>) -> String {
    wire((|| {
        let d = parse(desc.as_deref())?;
        let plane = plane_of(world);
        let from = v3(&d["from"]);
        let to = v3(&d["to"]);
        let n = navs(world)?;
        let n = lock(&n);
        match n.get(handle(d["nav"].as_f64().unwrap_or(0.0))) {
            Some(Nav::Grid(g)) => {
                let algorithm = if d["algorithm"].as_str() == Some("astar") {
                    Algorithm::AStar
                } else {
                    Algorithm::Jps
                };
                let (a, b) = (plane.to_plane(from), plane.to_plane(to));
                let sc = g
                    .nearest_walkable(a, 4)
                    .ok_or("the start is off the grid or walled in")?;
                let gc = g
                    .nearest_walkable(b, 4)
                    .ok_or("the end is off the grid or walled in")?;
                let mut scratch = Scratch::default();
                let p = match g.find_path(sc, gc, algorithm, &mut scratch) {
                    Some(p) => p,
                    None => {
                        return Ok(path_value(
                            &[from],
                            false,
                            0,
                            if algorithm == Algorithm::Jps {
                                "jps"
                            } else {
                                "astar"
                            },
                        ));
                    }
                };
                let smooth = d["smooth"].as_bool().unwrap_or(true);
                let clearance = (f(&d, "radius", 0.0) / g.cell_size).min(0.45);
                let end = if g.cell_at(b) == Some(gc) {
                    b
                } else {
                    g.center(gc.0, gc.1)
                };
                let pts: Vec<Vec3> = g
                    .waypoints(&p, a, end, smooth, clearance)
                    .into_iter()
                    .map(|q| plane.to_world(q, plane.height(from)))
                    .collect();
                Ok(path_value(
                    &pts,
                    g.cell_at(b) == Some(gc),
                    p.expanded,
                    if p.algorithm == Algorithm::Jps {
                        "jps"
                    } else {
                        "astar"
                    },
                ))
            }
            Some(Nav::Mesh(m)) => {
                let extent = Vec3::new(2.0, m.field.params.agent_height.max(2.0), 2.0);
                let p = m
                    .find_path(from, to, extent)
                    .ok_or("the start is not on the navmesh (within 2 m)")?;
                Ok(path_value(&p.points, p.complete, p.expanded, "navmesh"))
            }
            Some(Nav::Flow { .. }) => {
                Err("a flow field answers directions, not paths: use og.nav.flowAt".into())
            }
            None => Err("desc.nav is not a grid or navmesh of this world".into()),
        }
    })())
}

fn cells_of(g: &Grid, plane: Plane, targets: &Value) -> Vec<(u32, u32)> {
    targets
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|t| g.cell_at(plane.to_plane(v3(t))))
                .collect()
        })
        .unwrap_or_default()
}

pub fn og_nav_flow_field(world: u32, desc: Option<String>) -> String {
    wire((|| {
        let d = parse(desc.as_deref())?;
        let plane = plane_of(world);
        let n = navs(world)?;
        let mut n = lock(&n);
        let gh = handle(d["grid"].as_f64().unwrap_or(0.0));
        let Some(Nav::Grid(g)) = n.get(gh) else {
            return Err("desc.grid is not a grid of this world".into());
        };
        let targets = cells_of(g, plane, &d["targets"]);
        let field = FlowField::build(g, &targets);
        Ok(json!(n.insert(Nav::Flow {
            grid: gh,
            targets,
            field
        })))
    })())
}

pub fn og_nav_set_flow_targets(world: u32, field: f64, targets: String) -> String {
    match (|| -> Result<(), String> {
        let plane = plane_of(world);
        let t = parse(Some(&targets))?;
        let n = navs(world)?;
        let mut n = lock(&n);
        let gh = match n.get(handle(field)) {
            Some(Nav::Flow { grid, .. }) => *grid,
            _ => return Err(format!("{field} is not a flow field (og.nav.flowField)")),
        };
        let Some(Nav::Grid(g)) = n.get(gh).cloned() else {
            return Err("the flow field's grid is gone".into());
        };
        let cells = cells_of(&g, plane, &t);
        if let Some(Nav::Flow { targets, field, .. }) = n.get_mut(handle(field)) {
            *field = FlowField::build(&g, &cells);
            *targets = cells;
        }
        Ok(())
    })() {
        Ok(()) => super::ok_void(),
        Err(e) => refuse(e),
    }
}

pub fn og_nav_flow_at(world: u32, field: f64, point: String) -> String {
    let p = parse(Some(&point)).map(|v| v3(&v)).unwrap_or_default();
    let plane = plane_of(world);
    let d = navs(world)
        .ok()
        .and_then(|n| match lock(&n).get(handle(field)) {
            Some(Nav::Flow { field, .. }) => Some(field.sample(plane.to_plane(p))),
            _ => None,
        });
    j3(d.map_or(Vec3::ZERO, |d| plane.to_world(d, 0.0))).to_string()
}

/// `NavMeshInfo`: the bounds are its polygons' (FINDINGS #9 of
/// examples/kestrel-stair: the floor's origin plus the agent's height was
/// not where a baked level's terraces are).
fn mesh_info(h: u64, m: &NavMesh) -> Value {
    let (min, max) = m.bounds();
    json!({ "mesh": h, "polygons": m.poly_count(), "tiles": m.tiles.len(), "min": j3(min), "max": j3(max), "cellSize": m.field.params.cell_size })
}

pub fn og_nav_load(world: u32, path: String) -> String {
    wire(engine(world).and_then(|eng| {
        let id = og_cook::runtime::asset_id(&path).to_string();
        let (field, tiles) = {
            let assets = eng.assets.borrow();
            let (bytes, bundle) = assets.blob(&id)?;
            bundle
                .navmesh(&id, bytes.bytes())
                .map_err(|e| e.to_string())?
        };
        let mesh = NavMesh::from_tiles(field, tiles).ok_or_else(|| {
            format!("{id}: the navmesh's tiles do not fit its heightfield; cook it again")
        })?;
        let n = navs(world)?;
        let mut n = lock(&n);
        let info = mesh_info(0, &mesh);
        let h = n.insert(Nav::Mesh(Box::new(mesh)));
        let mut info = info;
        info["mesh"] = json!(h);
        Ok(info)
    }))
}

pub fn og_nav_build(world: u32, desc: Option<String>) -> String {
    wire((|| {
        engine(world)?;
        let d = parse(desc.as_deref())?;
        let min = v3(&d["min"]);
        let max = if d["max"].is_object() {
            v3(&d["max"])
        } else {
            Vec3::new(16.0, 1.0, 16.0)
        };
        if max.x <= min.x || max.z <= min.z {
            return Err("the floor is empty: max must be beyond min on x and z".into());
        }
        let params = NavParams {
            cell_size: f(&d, "cellSize", 0.25).clamp(0.02, 4.0),
            agent_radius: f(&d, "agentRadius", 0.4).max(0.0),
            agent_height: f(&d, "agentHeight", 1.8).max(0.1),
            tile_size: d["tileSize"].as_u64().unwrap_or(32).clamp(8, 256) as u32,
            ..NavParams::default()
        };
        let cs = params.cell_size;
        let width = ((max.x - min.x) / cs).floor().max(1.0) as u32;
        let depth = ((max.z - min.z) / cs).floor().max(1.0) as u32;
        if (width as u64) * (depth as u64) > 16_000_000 {
            return Err(format!(
                "the floor is {width} × {depth} cells: raise cellSize"
            ));
        }
        let origin = Vec3::new(min.x, min.y, min.z);
        let r = params.agent_radius;
        let boxes: Vec<(Vec3, Vec3)> = d["blocked"]
            .as_array()
            .map(|a| a.iter().map(|b| (v3(&b["min"]), v3(&b["max"]))).collect())
            .unwrap_or_default();
        let rc = params.radius_cells();
        let field = Heightfield::flat(params, origin, width, depth).without(|x, z| {
            // The floor's edge and every box, grown by the agent radius.
            if x < rc || z < rc || x + rc >= width || z + rc >= depth {
                return true;
            }
            let c = Vec3::new(
                origin.x + (x as f32 + 0.5) * cs,
                0.0,
                origin.z + (z as f32 + 0.5) * cs,
            );
            boxes.iter().any(|(lo, hi)| {
                c.x > lo.x - r && c.x < hi.x + r && c.z > lo.z - r && c.z < hi.z + r && hi.y > min.y
            })
        });
        let mesh = NavMesh::build(field);
        let n = navs(world)?;
        let mut n = lock(&n);
        let mut info = mesh_info(0, &mesh);
        info["mesh"] = json!(n.insert(Nav::Mesh(Box::new(mesh))));
        Ok(info)
    })())
}

pub fn og_nav_set_obstacle(world: u32, mesh: f64, id: u32, obstacle: Option<String>) -> String {
    wire((|| {
        let o = match obstacle.as_deref().map(|t| parse(Some(t))).transpose()? {
            Some(v) if v.is_object() => Some(Obstacle {
                min: v3(&v["min"]),
                max: v3(&v["max"]),
            }),
            _ => None,
        };
        let n = navs(world)?;
        let mut n = lock(&n);
        match n.get_mut(handle(mesh)) {
            Some(Nav::Mesh(m)) => {
                let tiles = m.set_obstacle(id, o);
                Ok(json!({ "tiles": tiles, "polygons": m.poly_count() }))
            }
            _ => Err(format!(
                "{mesh} is not a navmesh (og.nav.load, og.nav.build)"
            )),
        }
    })())
}

pub fn og_nav_nearest(world: u32, mesh: f64, point: String, extent: Option<String>) -> String {
    let p = parse(Some(&point)).map(|v| v3(&v)).unwrap_or_default();
    let e = extent
        .as_deref()
        .and_then(|t| parse(Some(t)).ok())
        .map(|v| v3(&v))
        .filter(|e| *e != Vec3::ZERO)
        .unwrap_or(Vec3::splat(2.0));
    let hit = navs(world)
        .ok()
        .and_then(|n| match lock(&n).get(handle(mesh)) {
            Some(Nav::Mesh(m)) => m.nearest(p, e).map(|r| r.1),
            _ => None,
        });
    hit.map_or(Value::Null, j3).to_string()
}

fn path_of(engine: u32, e: Entity) -> Option<og_nav::system::PathState> {
    super::e6::with(engine, |m| m.nav.as_ref().and_then(|n| n.path_of(e))).flatten()
}

pub fn og_nav_path_of(world: u32, entity: f64) -> String {
    match path_of(world, Entity::from_bits(handle(entity))) {
        Some(p) if !p.corners.is_empty() => {
            path_value(&p.corners, p.status != og_nav::STATUS_PARTIAL, 0, "agent").to_string()
        }
        _ => "null".into(),
    }
}

/// `dump_nav`'s JSON: the agent's path, the corner it heads for and its status.
pub(crate) fn path_json(engine: u32, world: &World, e: Entity) -> Option<Value> {
    let p = path_of(engine, e)?;
    let status = ["idle", "moving", "arrived", "unreachable", "partial"]
        .get(p.status as usize)
        .copied()
        .unwrap_or("idle");
    Some(json!({
        "entity": og_probe::Names.id_of(world, e),
        "status": status,
        "corners": p.corners.iter().map(|c| j3(*c)).collect::<Vec<_>>(),
        "next": p.next,
        "target": j3(p.target),
        "age": p.age,
    }))
}
