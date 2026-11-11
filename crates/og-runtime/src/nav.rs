//! The IDL's `nav` module over og-nav: grids, flow fields, navmeshes
//! (cooked, or built over a flat floor), path queries, obstacles and the
//! agents' paths. The registry is the engine's (`e6::Modules::navs`);
//! og.nav's system and state block are the world's.

use super::engine::{Engine, engine};
use super::refuse;
use super::wire::{
    self, FlowDesc, NavAlgorithm, NavAreaDesc, NavBox, NavCell, NavGridDesc, NavLinkDesc,
    NavLinkKind, NavMeshDesc, NavMeshInfo, NavPath, NavRebuild, NavShape, NavShapeKind, PathDesc,
    TilemapGridDesc, ToJson,
};
use og_core::glam::{Vec2, Vec3};
use og_ecs::{Entity, World};
use og_nav::{
    Algorithm, Filter, FlowField, Grid, Heightfield, Nav, NavMesh, NavParams, Obstacle, Plane,
    Scratch, Shape,
};
use serde_json::{Value, json};

fn j3(v: Vec3) -> Value {
    json!({ "x": v.x, "y": v.y, "z": v.z })
}

fn wire<T: ToJson>(r: Result<T, String>) -> String {
    match r {
        Ok(v) => wire::ok(&v),
        Err(e) => refuse(e),
    }
}

/// What a navmesh edit rebuilt: its tiles, and the mesh's polygons after.
fn rebuild(tiles: Vec<usize>, m: &NavMesh) -> NavRebuild {
    NavRebuild {
        tiles: tiles.into_iter().map(|t| t as u32).collect(),
        polygons: m.poly_count() as u32,
    }
}

/// og-nav's shape for a NavShape.
fn shape(s: NavShape) -> Result<Shape, String> {
    Ok(match s.kind {
        NavShapeKind::Box => Shape::Box {
            min: s.min,
            max: s.max,
        },
        NavShapeKind::OrientedBox => Shape::Oriented {
            center: s.center,
            half: s.size * 0.5,
            yaw: s.yaw,
        },
        NavShapeKind::Cylinder => Shape::Cylinder {
            base: s.center,
            radius: s.radius.max(0.0),
            height: s.height.max(0.0),
        },
        NavShapeKind::Convex => {
            if s.points.len() < 3 {
                return Err(format!(
                    "a convex shape needs at least three points, got {}",
                    s.points.len()
                ));
            }
            Shape::Convex {
                points: s.points.iter().map(|p| Vec2::new(p.x, p.z)).collect(),
                bottom: s.min.y,
                top: s.max.y,
            }
        }
    })
}

/// A path filter from areaCosts and excluded (PathDesc, og.NavFilter);
/// area ids past 63 exclude nothing.
fn filter(d: &PathDesc) -> Filter {
    let mut excluded = 0u64;
    for &a in d.excluded.iter().filter(|&&a| a < 64) {
        excluded |= 1 << a;
    }
    Filter {
        costs: d.area_costs.clone(),
        excluded,
    }
}

/// A navmesh edit on mesh handle `mesh`: what it rebuilt.
fn edit_mesh(
    world: u32,
    mesh: f64,
    edit: impl FnOnce(&mut NavMesh) -> Vec<usize>,
) -> Result<NavRebuild, String> {
    let n = navs(world)?;
    let mut n = lock(&n);
    match n.get_mut(handle(mesh)) {
        Some(Nav::Mesh(m)) => {
            let tiles = edit(m);
            Ok(rebuild(tiles, m))
        }
        _ => Err(format!(
            "{mesh} is not a navmesh (og.nav.load, og.nav.build)"
        )),
    }
}

pub fn og_nav_carve(world: u32, mesh: f64, id: u32, desc: Option<String>) -> String {
    wire((|| {
        let s = wire::arg::<Option<NavShape>>(desc.as_deref(), "shape")?
            .map(shape)
            .transpose()?;
        edit_mesh(world, mesh, |m| m.carve(id, s))
    })())
}

pub fn og_nav_set_area(world: u32, mesh: f64, id: u32, desc: Option<String>) -> String {
    wire((|| {
        let a = match wire::arg::<Option<NavAreaDesc>>(desc.as_deref(), "desc")? {
            Some(d) => {
                if d.area > 63 {
                    return Err(format!("an area id is 0 to 63, got {}", d.area));
                }
                Some((shape(d.shape)?, d.area as u8))
            }
            None => None,
        };
        edit_mesh(world, mesh, |m| m.set_area(id, a))
    })())
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
        let d: NavGridDesc = wire::arg(desc.as_deref(), "desc")?;
        let w = d.width.clamp(1, 4096);
        let h = d.height.clamp(1, 4096);
        let (cs, origin) = (d.cell_size, d.origin);
        let grid = if d.costs.is_empty() {
            Grid::new(w, h, cs, origin)
        } else {
            let c = &d.costs;
            if c.len() != (w * h) as usize {
                return Err(format!(
                    "costs has {} values; a {w} × {h} grid needs {} (or none)",
                    c.len(),
                    w * h
                ));
            }
            let costs = c.iter().map(|&x| x.min(255) as u8).collect();
            Grid::from_costs(w, h, cs, origin, costs).ok_or("the costs do not fit the grid")?
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
    let d: TilemapGridDesc = wire::arg(desc.as_deref(), "desc")?;
    let e = Entity::from_bits(d.entity);
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
            wire::value::<Vec3>(&p["position"], "position")?,
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
    let names = &d.layers;
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
    let inflate = d.inflate.min(8) as i32;
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
    let p: Vec3 = wire::arg(Some(&point), "point").unwrap_or_default();
    let plane = plane_of(world);
    let cell = navs(world)
        .ok()
        .and_then(|n| match lock(&n).get(handle(grid)) {
            Some(Nav::Grid(g)) => g.cell_at(plane.to_plane(p)),
            _ => None,
        });
    wire::text_of(&match cell {
        Some((x, y)) => NavCell { x, y, inside: true },
        None => NavCell {
            x: 0,
            y: 0,
            inside: false,
        },
    })
}

fn path_value(
    points: &[Vec3],
    complete: bool,
    expanded: u32,
    algorithm: &str,
    links: &[og_nav::PathLink],
) -> NavPath {
    NavPath {
        points: points.to_vec(),
        length: points.windows(2).map(|w| w[0].distance(w[1])).sum(),
        complete,
        expanded,
        algorithm: algorithm.into(),
        links: links.iter().map(|l| l.corner).collect(),
    }
}

/// og-nav's off-mesh link for a NavLinkDesc.
fn link_desc(d: NavLinkDesc) -> og_nav::Link {
    og_nav::Link {
        start: d.start,
        end: d.end,
        kind: match d.kind {
            NavLinkKind::Jump => og_nav::LinkKind::Jump,
            NavLinkKind::Ladder => og_nav::LinkKind::Ladder,
            NavLinkKind::Door => og_nav::LinkKind::Door,
            NavLinkKind::Teleport => og_nav::LinkKind::Teleport,
        },
        bidirectional: d.bidirectional,
        cost: d.cost.max(0.0),
        open: d.open,
        duration: d.duration.max(0.0),
        game: d.game,
    }
}

/// og.NavLinkReached joins the world (a lazy record) once a mesh has a link.
fn links_used(world: u32) -> Result<(), String> {
    let eng = engine(world)?;
    super::lazy::ensure(
        &eng,
        eng.world()?,
        <og_nav::NavLinkReached as og_core::Reflect>::NAME,
    )
}

pub fn og_nav_set_link(world: u32, mesh: f64, id: u32, desc: Option<String>) -> String {
    wire((|| {
        let l = wire::arg::<Option<NavLinkDesc>>(desc.as_deref(), "desc")?.map(link_desc);
        if l.is_some() {
            links_used(world)?;
        }
        let n = navs(world)?;
        let mut n = lock(&n);
        match n.get_mut(handle(mesh)) {
            Some(Nav::Mesh(m)) => Ok(json!(m.set_link(id, l))),
            _ => Err(format!(
                "{mesh} is not a navmesh (og.nav.load, og.nav.build)"
            )),
        }
    })())
}

pub fn og_nav_set_link_open(world: u32, mesh: f64, id: u32, open: bool) -> String {
    wire((|| {
        let n = navs(world)?;
        let mut n = lock(&n);
        match n.get_mut(handle(mesh)) {
            Some(Nav::Mesh(m)) => {
                if m.set_link_open(id, open) {
                    Ok(Value::Null)
                } else {
                    Err(format!("navmesh {mesh} has no link {id}"))
                }
            }
            _ => Err(format!(
                "{mesh} is not a navmesh (og.nav.load, og.nav.build)"
            )),
        }
    })())
}

pub fn og_nav_finish_link(world: u32, entity: f64) -> bool {
    super::e6::with(world, |m| {
        m.nav
            .as_ref()
            .is_some_and(|n| n.finish_link(Entity::from_bits(handle(entity))))
    })
    .unwrap_or(false)
}

pub fn og_nav_find_path(world: u32, desc: Option<String>) -> String {
    wire((|| {
        let d: PathDesc = wire::arg(desc.as_deref(), "desc")?;
        let plane = plane_of(world);
        let (from, to) = (d.from, d.to);
        let n = navs(world)?;
        let n = lock(&n);
        match n.get(d.nav) {
            Some(Nav::Grid(g)) => {
                let algorithm = match d.algorithm {
                    NavAlgorithm::Astar => Algorithm::AStar,
                    NavAlgorithm::Jps => Algorithm::Jps,
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
                            &[],
                        ));
                    }
                };
                let smooth = d.smooth;
                let clearance = (d.radius / g.cell_size).min(0.45);
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
                    &[],
                ))
            }
            Some(Nav::Mesh(m)) => {
                let extent = Vec3::new(2.0, m.field.params.agent_height.max(2.0), 2.0);
                let p = m
                    .find_path_filtered(from, to, extent, &filter(&d))
                    .ok_or("the start is not on the navmesh (within 2 m)")?;
                Ok(path_value(
                    &p.points, p.complete, p.expanded, "navmesh", &p.links,
                ))
            }
            Some(Nav::Flow { .. }) => {
                Err("a flow field answers directions, not paths: use og.nav.flowAt".into())
            }
            None => Err("desc.nav is not a grid or navmesh of this world".into()),
        }
    })())
}

fn cells_of(g: &Grid, plane: Plane, targets: &[Vec3]) -> Vec<(u32, u32)> {
    targets
        .iter()
        .filter_map(|&t| g.cell_at(plane.to_plane(t)))
        .collect()
}

pub fn og_nav_flow_field(world: u32, desc: Option<String>) -> String {
    wire((|| {
        let d: FlowDesc = wire::arg(desc.as_deref(), "desc")?;
        let plane = plane_of(world);
        let n = navs(world)?;
        let mut n = lock(&n);
        let gh = d.grid;
        let Some(Nav::Grid(g)) = n.get(gh) else {
            return Err("desc.grid is not a grid of this world".into());
        };
        let targets = cells_of(g, plane, &d.targets);
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
        let t: Vec<Vec3> = wire::arg(Some(&targets), "targets")?;
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
    let p: Vec3 = wire::arg(Some(&point), "point").unwrap_or_default();
    let plane = plane_of(world);
    let d = navs(world)
        .ok()
        .and_then(|n| match lock(&n).get(handle(field)) {
            Some(Nav::Flow { field, .. }) => Some(field.sample(plane.to_plane(p))),
            _ => None,
        });
    wire::text_of(&d.map_or(Vec3::ZERO, |d| plane.to_world(d, 0.0)))
}

/// `NavMeshInfo`: the bounds are its polygons' (FINDINGS #9 of
/// kestrel-stair: the floor's origin plus the agent's height was
/// not where a baked level's terraces are).
fn mesh_info(h: u64, m: &NavMesh) -> NavMeshInfo {
    let (min, max) = m.bounds();
    NavMeshInfo {
        mesh: h,
        polygons: m.poly_count() as u32,
        tiles: m.tiles.len() as u32,
        min,
        max,
        cell_size: m.field.params.cell_size,
        profile: String::new(),
    }
}

pub fn og_nav_load(world: u32, path: String, profile: Option<String>) -> String {
    wire(engine(world).and_then(|eng| {
        let id = og_bundle::runtime::asset_id(&path).to_string();
        let want = profile.unwrap_or_default();
        let (field, tiles, areas, links, profile) = {
            let assets = eng.assets.borrow();
            let (bytes, bundle) = assets.blob(&id)?;
            let names = bundle
                .navmesh_profiles(&id, bytes.bytes())
                .map_err(|e| e.to_string())?;
            let bytes = bundle
                .navmesh_profile(&id, bytes.bytes(), &want)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| {
                    if names.is_empty() {
                        format!("{id} has one unnamed agent, no profile '{want}': omit the profile")
                    } else {
                        format!(
                            "{id} has no agent profile '{want}': {}",
                            names
                                .iter()
                                .map(|n| if n.is_empty() { "(unnamed)" } else { n })
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    }
                })?;
            let (field, tiles) = bundle.navmesh(&id, bytes).map_err(|e| e.to_string())?;
            let areas = bundle
                .navmesh_areas(&id, bytes)
                .map_err(|e| e.to_string())?;
            let links = bundle
                .navmesh_links(&id, bytes)
                .map_err(|e| e.to_string())?;
            let profile = if want.is_empty() {
                names.into_iter().next().unwrap_or_default()
            } else {
                want
            };
            (field, tiles, areas, links, profile)
        };
        let mesh = NavMesh::from_tiles_with_areas(field, tiles, areas)
            .ok_or_else(|| {
                format!("{id}: the navmesh's tiles do not fit its heightfield; cook it again")
            })?
            .with_links(links.clone());
        if !links.is_empty() {
            super::lazy::ensure(
                &eng,
                eng.world()?,
                <og_nav::NavLinkReached as og_core::Reflect>::NAME,
            )?;
        }
        let n = navs(world)?;
        let mut n = lock(&n);
        let mut info = mesh_info(0, &mesh);
        info.mesh = n.insert(Nav::Mesh(Box::new(mesh)));
        info.profile = profile;
        Ok(info)
    }))
}

pub fn og_nav_build(world: u32, desc: Option<String>) -> String {
    wire((|| {
        engine(world)?;
        let d: NavMeshDesc = wire::arg(desc.as_deref(), "desc")?;
        let (min, max) = (d.min, d.max);
        if max.x <= min.x || max.z <= min.z {
            return Err("the floor is empty: max must be beyond min on x and z".into());
        }
        let params = NavParams {
            cell_size: d.cell_size.clamp(0.02, 4.0),
            agent_radius: d.agent_radius.max(0.0),
            agent_height: d.agent_height.max(0.1),
            tile_size: d.tile_size.clamp(8, 256),
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
        let boxes: Vec<(Vec3, Vec3)> = d.blocked.iter().map(|b| (b.min, b.max)).collect();
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
        info.mesh = n.insert(Nav::Mesh(Box::new(mesh)));
        Ok(info)
    })())
}

pub fn og_nav_set_obstacle(world: u32, mesh: f64, id: u32, obstacle: Option<String>) -> String {
    wire((|| {
        let o = wire::arg::<Option<NavBox>>(obstacle.as_deref(), "obstacle")?.map(|b| Obstacle {
            min: b.min,
            max: b.max,
        });
        let n = navs(world)?;
        let mut n = lock(&n);
        match n.get_mut(handle(mesh)) {
            Some(Nav::Mesh(m)) => {
                let tiles = m.set_obstacle(id, o);
                Ok(rebuild(tiles, m))
            }
            _ => Err(format!(
                "{mesh} is not a navmesh (og.nav.load, og.nav.build)"
            )),
        }
    })())
}

pub fn og_nav_nearest(world: u32, mesh: f64, point: String, extent: Option<String>) -> String {
    let p: Vec3 = wire::arg(Some(&point), "point").unwrap_or_default();
    let e = wire::arg::<Vec3>(extent.as_deref(), "extent")
        .ok()
        .filter(|e| *e != Vec3::ZERO)
        .unwrap_or(Vec3::splat(2.0));
    let hit = navs(world)
        .ok()
        .and_then(|n| match lock(&n).get(handle(mesh)) {
            Some(Nav::Mesh(m)) => m.nearest(p, e).map(|r| r.1),
            _ => None,
        });
    wire::text_of(&hit)
}

fn path_of(engine: u32, e: Entity) -> Option<og_nav::system::PathState> {
    super::e6::with(engine, |m| m.nav.as_ref().and_then(|n| n.path_of(e))).flatten()
}

pub fn og_nav_path_of(world: u32, entity: f64) -> String {
    match path_of(world, Entity::from_bits(handle(entity))) {
        Some(p) if !p.corners.is_empty() => wire::text_of(&path_value(
            &p.corners,
            p.status != og_nav::STATUS_PARTIAL,
            0,
            "agent",
            &p.links,
        )),
        _ => "null".into(),
    }
}

/// `dump_nav`'s JSON: the agent's path, the corner it heads for and its status.
pub(crate) fn path_json(engine: u32, world: &World, e: Entity) -> Option<Value> {
    let p = path_of(engine, e)?;
    let status = [
        "idle",
        "moving",
        "arrived",
        "unreachable",
        "partial",
        "onLink",
    ]
    .get(p.status as usize)
    .copied()
    .unwrap_or("idle");
    let mut v = json!({
        "entity": og_probe::Names.id_of(world, e),
        "status": status,
        "corners": p.corners.iter().map(|c| j3(*c)).collect::<Vec<_>>(),
        "next": p.next,
        "target": j3(p.target),
        "age": p.age,
    });
    // Off-mesh links (G2-4), only on a path that crosses one.
    if !p.links.is_empty() {
        v["links"] = json!(p.links.iter().map(|l| l.corner).collect::<Vec<_>>());
    }
    if let Some(t) = p.traverse {
        v["crossing"] = json!({ "link": t.link, "kind": t.kind.name(), "to": j3(t.to) });
    }
    Some(v)
}

/// The lazy records og.nav reads (super::lazy): granted to its system.
pub(crate) fn enable_filters(w: &mut World) -> og_core::Result<()> {
    og_nav::enable_filters(w)
}

pub(crate) fn enable_crowds(w: &mut World) -> og_core::Result<()> {
    og_nav::enable_crowds(w)
}

pub(crate) fn enable_links(w: &mut World) -> og_core::Result<()> {
    og_nav::enable_links(w)
}
