//! The og.destruction2d system: terrains cut, pieces detached and spawned.

use super::*;

pub(super) struct Ls {
    pub(super) terrain: (SchemaId, Layout),
    pub(super) piece: (SchemaId, Layout),
    pub(super) detached: (SchemaId, Layout),
    pub(super) transform: (SchemaId, Layout),
    pub(super) rigid: (SchemaId, Layout),
    pub(super) velocity: (SchemaId, Layout),
    pub(super) collider: (SchemaId, Layout),
    pub(super) parent: (SchemaId, Layout),
}

pub(super) const WRITES: [&str; 5] = [
    "og.Transform",
    "og.RigidBody",
    "og.Velocity",
    "og.Collider",
    "og.Parent",
];

/// Registers the `og.destruction2d` system and its state block once every
/// one of [`RECORDS`] is (crate::engine::lazy registers them one after another).
pub(crate) fn enable(world: &mut World, mount: Mount<'_>) -> og_core::Result<()> {
    if world.systems().contains(&SYSTEM) || RECORDS.iter().any(|n| world.id_by_name(n).is_err()) {
        return Ok(());
    }
    let layout = |n: &str, kind: RecordKind| -> og_core::Result<(SchemaId, Layout)> {
        let id = world.id_by_name(n)?;
        // The IDL's layout, with its defaults (a schema read back has none).
        let layout = og_api::host::RECORDS
            .iter()
            .find(|d| d.schema == n)
            .and_then(|d| Layout::from_decl(d).ok());
        let layout = match layout {
            Some(l) => l,
            None => Layout::from_schema(world.schema(id)?, kind),
        };
        Ok((id, layout))
    };
    use RecordKind::{Component as C, Event as E};
    let ls = Ls {
        terrain: layout(TERRAIN, C)?,
        piece: layout(PIECE, C)?,
        detached: layout(DETACHED, E)?,
        transform: layout("og.Transform", C)?,
        rigid: layout("og.RigidBody", C)?,
        velocity: layout("og.Velocity", C)?,
        collider: layout("og.Collider", C)?,
        parent: layout("og.Parent", C)?,
    };
    let sh = mount.modules.destruction2d.0.clone();
    lock(&sh).state = State::default();
    let state = sh.clone();
    // Capture only the shared physics link, not the 3D prop state. Cloning
    // and dropping that state would load destruction3d for a pixel game.
    let physics = mount.modules.destruction.physics.clone();
    let s = |v: &[&str]| v.iter().map(|n| n.to_string()).collect::<Vec<_>>();
    world.add_system(
        SystemDesc {
            name: SYSTEM.into(),
            reads: Vec::new(),
            writes: [s(&RECORDS), s(&WRITES)].concat(),
            queries: vec![QueryDesc {
                write: vec![TERRAIN.to_string()],
                ..QueryDesc::default()
            }],
            ..Default::default()
        },
        move |ctx| run(ctx, &physics, &state, &ls),
    )?;
    if world.systems().contains(&"og.physics") {
        world.move_system_before(SYSTEM, "og.physics")?;
    }
    mount
        .modules
        .destruction2d
        .1
        .store(true, std::sync::atomic::Ordering::Relaxed);
    world.add_state_block(SYSTEM, Box::new(Block(sh)))
}

pub(super) trait Enc {
    fn enc(&self, value: &Value) -> og_core::Result<Vec<u8>>;
}

impl Enc for Layout {
    fn enc(&self, value: &Value) -> og_core::Result<Vec<u8>> {
        self.encode(None, Some(value)).map_err(invalid)
    }
}

/// A piece about to be spawned.
pub(super) struct Cut {
    pub(super) terrain: u64,
    pub(super) sub: Sub,
    /// Its cell (0, 0)'s grain place.
    pub(super) grain: [i32; 2],
    pub(super) look: Look,
    pub(super) body: Body,
    /// Its cell (0, 0)'s corner and its rotation, in the world.
    pub(super) pose: (Vec3, Quat),
    pub(super) velocity: (Vec2, f32),
    pub(super) cause: u32,
    pub(super) min: u32,
}

pub(super) fn quat_of(t: &record::Transform) -> Quat {
    t.quaternion.normalize()
}

pub(super) fn run(
    ctx: &mut SystemContext<'_>,
    physics: &Mutex<Option<og_physics::PhysicsSystem>>,
    sh: &Mutex<Shared>,
    l: &Ls,
) -> og_core::Result<()> {
    let physics = lock(physics).clone();
    let tick = ctx.time().tick;
    let mut rows = Vec::new();
    for a in ctx.query(0)? {
        rows.extend_from_slice(a.entities());
    }
    let mut g = lock(sh);
    let st = &mut g.state;
    let mut gone: Vec<u64> = Vec::new();
    let mut settings: BTreeMap<u64, Settings> = BTreeMap::new();
    // Every terrain: filled the first tick it is seen (or when its shape
    // changes), its colliders rebuilt when what they are built from moves.
    for e in rows {
        let key = e.to_bits();
        let s = Settings::of(&row(&l.terrain.1, &ctx.get_bytes(e, l.terrain.0)?)?);
        let origin = match ctx.get_bytes(e, l.transform.0) {
            Ok(b) => row::<record::Transform>(&l.transform.1, &b)?.position,
            Err(_) => Vec3::ZERO,
        }
        .to_array();
        if st.terrains.get(&key).is_none_or(|t| t.shape != s.shape) {
            if let Some(old) = st.terrains.remove(&key) {
                gone.extend(old.colliders.values().map(|c| c.0));
            }
            st.terrains.insert(key, s.filled(origin));
        }
        let t = st.terrains.get_mut(&key).expect("inserted above");
        if t.look != s.look {
            t.revision += 1;
            t.updates.clear();
            t.updates
                .push((t.revision, [0, 0, t.grid.width(), t.grid.height()]));
        }
        t.look = s.look;
        if t.origin != origin || t.built != s.built() {
            t.origin = origin;
            t.built = s.built();
            let [w, h, chunk] = t.shape;
            t.grid = PixelTerrain::from_cells(w, h, chunk, t.grid.cells().to_vec())
                .expect("its own cells");
        }
        settings.insert(key, s);
    }
    let dropped: Vec<u64> = st
        .terrains
        .keys()
        .filter(|k| !settings.contains_key(k))
        .copied()
        .collect();
    for k in dropped {
        let t = st.terrains.remove(&k).expect("listed above");
        gone.extend(t.colliders.values().map(|c| c.0));
    }
    let before = st.pieces.len();
    st.pieces.retain(|k, p| {
        let live = ctx.is_alive(Entity::from_bits(*k));
        if !live {
            gone.extend_from_slice(&p.colliders);
        }
        live
    });
    let mut any = before != st.pieces.len() || !gone.is_empty();

    // The queued changes, in the order they were made.
    let ops = std::mem::take(&mut st.ops);
    let mut changed: BTreeSet<u64> = BTreeSet::new();
    let mut exploded: BTreeSet<u64> = BTreeSet::new();
    let mut blasts: Vec<([f32; 2], f32, f32)> = Vec::new();
    let mut cuts: Vec<Cut> = Vec::new();
    let to_cell = |t: &Ground, p: [f32; 2]| {
        (
            (p[0] - t.origin[0]) / t.look.cell,
            (p[1] - t.origin[1]) / t.look.cell,
        )
    };
    let corner_of = |t: &Ground, o: [i32; 2]| {
        Vec3::new(
            t.origin[0] + o[0] as f32 * t.look.cell,
            t.origin[1] + o[1] as f32 * t.look.cell,
            t.origin[2],
        )
    };
    for (i, op) in ops.iter().enumerate() {
        let op_seed = splitmix(tick ^ (i as u64) << 40);
        match *op {
            Op::Carve {
                entity,
                point,
                radius,
            }
            | Op::Fill {
                entity,
                point,
                radius,
                ..
            } => {
                let Some(t) = st.terrains.get_mut(&entity) else {
                    continue;
                };
                let value = match op {
                    Op::Fill { bedrock: true, .. } => BEDROCK,
                    Op::Fill { .. } => GROUND,
                    _ => 0,
                };
                let (x, y) = to_cell(t, point);
                if t.grid.circle(x, y, radius / t.look.cell, value) > 0 {
                    changed.insert(entity);
                }
            }
            Op::Explode {
                point,
                radius,
                debris,
                cells,
                speed,
            } => {
                let reach = radius + debris;
                blasts.push((point, reach.max(1e-3), speed));
                for (&k, t) in st.terrains.iter_mut() {
                    let s = &settings[&k];
                    let (x, y) = to_cell(t, point);
                    let (r, rd) = (radius / t.look.cell, reach / t.look.cell);
                    let carved = t.grid.circle(x, y, r, 0);
                    // The ring round the hole, cut into pieces.
                    let (r2, rd2) = (r * r, rd * rd);
                    let lo = |c: f32| (c - rd).floor().max(0.0) as u32;
                    let x1 = ((x + rd).ceil().max(0.0) as u32).min(t.grid.width());
                    let y1 = ((y + rd).ceil().max(0.0) as u32).min(t.grid.height());
                    let mut ring = Vec::new();
                    for cy in lo(y)..y1 {
                        for cx in lo(x)..x1 {
                            let (dx, dy) = (cx as f32 + 0.5 - x, cy as f32 + 0.5 - y);
                            let d2 = dx * dx + dy * dy;
                            let c = t.grid.get(cx as i64, cy as i64);
                            if d2 > r2 && d2 <= rd2 && c != 0 && c != BEDROCK {
                                ring.push((cx as i32, cy as i32, c));
                            }
                        }
                    }
                    if carved == 0 && ring.is_empty() {
                        continue;
                    }
                    changed.insert(k);
                    exploded.insert(k);
                    for &(cx, cy, _) in &ring {
                        t.grid.set(cx as u32, cy as u32, 0);
                    }
                    let n = (ring.len() as u32).div_ceil(cells.max(1));
                    let seed = op_seed ^ u64::from(t.look.seed) ^ k;
                    for sub in voronoi(&ring, n, seed) {
                        cuts.push(Cut {
                            terrain: k,
                            grain: sub.origin,
                            pose: (corner_of(t, sub.origin), Quat::IDENTITY),
                            velocity: (Vec2::ZERO, 0.0),
                            look: t.look,
                            body: s.body,
                            cause: CAUSE_EXPLOSION,
                            min: s.min_piece,
                            sub,
                        });
                    }
                }
            }
            Op::Fracture { entity, pieces } => {
                if let Some(t) = st.terrains.get_mut(&entity) {
                    let s = &settings[&entity];
                    let mut all = Vec::new();
                    for y in 0..t.grid.height() {
                        for x in 0..t.grid.width() {
                            let c = t.grid.get(x as i64, y as i64);
                            if c != 0 && c != BEDROCK {
                                all.push((x as i32, y as i32, c));
                            }
                        }
                    }
                    for &(x, y, _) in &all {
                        t.grid.set(x as u32, y as u32, 0);
                    }
                    if !all.is_empty() {
                        changed.insert(entity);
                    }
                    let seed = op_seed ^ u64::from(t.look.seed) ^ entity;
                    for sub in voronoi(&all, pieces, seed) {
                        cuts.push(Cut {
                            terrain: entity,
                            grain: sub.origin,
                            pose: (corner_of(t, sub.origin), Quat::IDENTITY),
                            velocity: (Vec2::ZERO, 0.0),
                            look: t.look,
                            body: s.body,
                            cause: CAUSE_FRACTURE,
                            min: s.min_piece,
                            sub,
                        });
                    }
                } else if let Some(p) = st.pieces.get(&entity) {
                    let e = Entity::from_bits(entity);
                    let tr: record::Transform =
                        row(&l.transform.1, &ctx.get_bytes(e, l.transform.0)?)?;
                    let (pos, rot) = (tr.position, quat_of(&tr));
                    let (lin, spin) = match ctx.get_bytes(e, l.velocity.0) {
                        Ok(b) => {
                            let v: record::Velocity = row(&l.velocity.1, &b)?;
                            (v.linear.truncate(), v.angular.z)
                        }
                        Err(_) => (Vec2::ZERO, 0.0),
                    };
                    let mut all = Vec::new();
                    for y in 0..p.height {
                        for x in 0..p.width {
                            let c = p.cells[(y * p.width + x) as usize];
                            if c != 0 {
                                all.push((x as i32, y as i32, c));
                            }
                        }
                    }
                    let cell = p.look.cell;
                    let whole = physics
                        .as_ref()
                        .and_then(|ph| ph.mass_properties(e))
                        .map_or(pos, |mp| mp.center_of_mass);
                    let seed = op_seed ^ u64::from(p.look.seed) ^ entity;
                    let min = settings.get(&p.terrain).map_or(1, |s| s.min_piece);
                    for sub in voronoi(&all, pieces, seed) {
                        let local =
                            Vec3::new(sub.origin[0] as f32, sub.origin[1] as f32, 0.0) * cell;
                        let r = pos + rot * (local + sub.centre().extend(0.0) * cell) - whole;
                        // v + ω × r, in the plane.
                        let v = lin + Vec2::new(-r.y, r.x) * spin;
                        cuts.push(Cut {
                            terrain: p.terrain,
                            grain: [p.grain[0] + sub.origin[0], p.grain[1] + sub.origin[1]],
                            pose: (pos + rot * local, rot),
                            velocity: (v, spin),
                            look: p.look,
                            body: p.body,
                            cause: CAUSE_FRACTURE,
                            min,
                            sub,
                        });
                    }
                    if let Some(p) = st.pieces.remove(&entity) {
                        gone.extend(p.colliders);
                    }
                    ctx.despawn(e)?;
                    queued_structural();
                    any = true;
                }
            }
        }
    }

    // What no longer stands falls.
    for &k in &changed {
        let t = st.terrains.get_mut(&k).expect("changed terrains exist");
        let s = &settings[&k];
        for island in t.grid.detach() {
            let count = island.cells.iter().filter(|&&c| c != 0).count() as u32;
            let sub = Sub {
                origin: [island.origin[0] as i32, island.origin[1] as i32],
                width: island.width,
                height: island.height,
                cells: island.cells,
                count,
            };
            cuts.push(Cut {
                terrain: k,
                grain: sub.origin,
                pose: (corner_of(t, sub.origin), Quat::IDENTITY),
                velocity: (Vec2::ZERO, 0.0),
                look: t.look,
                body: s.body,
                cause: if exploded.contains(&k) {
                    CAUSE_EXPLOSION
                } else {
                    CAUSE_CARVE
                },
                min: s.min_piece,
                sub,
            });
        }
    }
    // Read and write each surviving body's velocity once: ECS writes are
    // deferred, so updating it once per explosion loses earlier impulses.
    for &k in st.pieces.keys() {
        let e = Entity::from_bits(k);
        let (Ok(tb), Ok(vb)) = (
            ctx.get_bytes(e, l.transform.0),
            ctx.get_bytes(e, l.velocity.0),
        ) else {
            continue;
        };
        let tr: record::Transform = row(&l.transform.1, &tb)?;
        let at = physics
            .as_ref()
            .and_then(|ph| ph.mass_properties(e))
            .map_or(tr.position, |mp| mp.center_of_mass)
            .truncate();
        let push = pushed(&blasts, at);
        if push == Vec2::ZERO {
            continue;
        }
        let mut vel = l.velocity.1.decode(&vb);
        let now = record::Velocity::from_json(&vel).map_err(invalid)?;
        vel["linear"] = jv(now.linear + push.extend(0.0));
        ctx.set_bytes(e, l.velocity.0, &l.velocity.1.enc(&vel)?)?;
    }
    any |= !cuts.is_empty();
    for mut cut in cuts {
        let centre = cut.pose.0 + cut.pose.1 * (cut.sub.centre().extend(0.0) * cut.look.cell);
        cut.velocity.0 += pushed(&blasts, centre.truncate());
        let (terrain, count, cause) = (cut.terrain, cut.sub.count, cut.cause);
        let piece = if count < cut.min.max(1) {
            0
        } else {
            let max_pieces = settings.get(&cut.terrain).map_or(0, |s| s.max_pieces);
            spawn_piece(ctx, l, st, cut, tick, max_pieces)?
        };
        let ev = json!({
            "terrain": terrain as f64, "piece": piece as f64, "cells": count,
            "point": jv(centre), "cause": cause,
        });
        ctx.send_bytes(l.detached.0, &l.detached.1.enc(&ev)?)?;
    }

    // Each chunk a change touched collides as its new outline.
    for (&k, t) in st.terrains.iter_mut() {
        let dirty = t.grid.take_dirty();
        if dirty.is_empty() && !changed.contains(&k) {
            continue;
        }
        any = true;
        t.revision += 1;
        let s = &settings[&k];
        let c = t.grid.chunk_size();
        let (cx, _) = t.grid.chunks();
        let rect = dirty
            .iter()
            .map(|&k| {
                let (x, y) = ((k % cx) * c, (k / cx) * c);
                [
                    x,
                    y,
                    (x + c).min(t.grid.width()),
                    (y + c).min(t.grid.height()),
                ]
            })
            .reduce(|a, b| {
                [
                    a[0].min(b[0]),
                    a[1].min(b[1]),
                    a[2].max(b[2]),
                    a[3].max(b[3]),
                ]
            });
        if let Some([x, y, x1, y1]) = rect {
            if t.updates.len() == 64 {
                t.updates.remove(0);
            }
            t.updates.push((t.revision, [x, y, x1 - x, y1 - y]));
        }
        for chunk in dirty {
            if let Some((e, _)) = t.colliders.remove(&chunk) {
                gone.push(e);
            }
            let Some(mesh) = chunk_polyline(&t.grid, chunk, s.tolerance, t.look.cell) else {
                continue;
            };
            let handle = og_physics::GENERATED_MESH
                | (mesh.content_hash() & (og_physics::GENERATED_MESH - 1));
            if let Some(p) = &physics {
                p.set_polyline(handle, mesh)?;
            }
            let at = Vec3::new(
                t.origin[0] + ((chunk % cx) * c) as f32 * t.look.cell,
                t.origin[1] + ((chunk / cx) * c) as f32 * t.look.cell,
                t.origin[2],
            );
            let collider = json!({
                "shape": og_physics::SHAPE_POLYLINE, "mesh": handle as f64,
                "friction": s.body.friction, "layers": s.body.layers, "mask": s.body.mask,
            });
            let e = ctx.spawn_bytes(vec![
                (
                    l.transform.0,
                    l.transform.1.enc(&json!({ "position": jv(at) }))?,
                ),
                (l.collider.0, l.collider.1.enc(&collider)?),
            ])?;
            queued_structural();
            t.colliders.insert(chunk, (e.to_bits(), handle));
        }
        let solid = t.grid.solid();
        let e = Entity::from_bits(k);
        let mut v = l.terrain.1.decode(&ctx.get_bytes(e, l.terrain.0)?);
        if record::PixelTerrain::from_json(&v).map_err(invalid)?.solid != solid {
            v["solid"] = json!(solid);
            ctx.set_bytes(e, l.terrain.0, &l.terrain.1.enc(&v)?)?;
        }
    }

    // Past a terrain's maxPieces, its oldest pieces go.
    let caps: BTreeMap<_, _> = st
        .pieces
        .values()
        .map(|p| {
            (
                p.terrain,
                settings
                    .get(&p.terrain)
                    .map_or(p.max_pieces, |s| s.max_pieces),
            )
        })
        .collect();
    for (&k, &max_pieces) in &caps {
        let mut mine: Vec<(u64, u64)> = st
            .pieces
            .iter()
            .filter(|(_, p)| p.terrain == k)
            .map(|(&e, p)| (p.tick, e))
            .collect();
        if mine.len() <= max_pieces as usize {
            continue;
        }
        mine.sort();
        for &(_, e) in &mine[..mine.len() - max_pieces as usize] {
            if let Some(p) = st.pieces.remove(&e) {
                gone.extend(p.colliders);
            }
            ctx.despawn(Entity::from_bits(e))?;
            queued_structural();
            any = true;
        }
    }
    for e in gone {
        let e = Entity::from_bits(e);
        if ctx.is_alive(e) {
            ctx.despawn(e)?;
            queued_structural();
        }
    }
    if any {
        // Generated meshes no collider names any more leave physics.
        let named: BTreeSet<u64> = st
            .terrains
            .values()
            .flat_map(|t| t.colliders.values().map(|c| c.1))
            .collect();
        if let Some(p) = &physics {
            for unused in st.meshes.difference(&named) {
                p.remove_mesh(*unused);
            }
        }
        st.meshes = named;
    }
    Ok(())
}

/// Spawns a cut as an og.PixelPiece: a dynamic body whose collider is the
/// exact rectangle decomposition of its cells, as compound children.
pub(super) fn spawn_piece(
    ctx: &mut SystemContext<'_>,
    l: &Ls,
    st: &mut State,
    cut: Cut,
    tick: u64,
    max_pieces: u32,
) -> og_core::Result<u64> {
    let Cut {
        terrain,
        sub,
        grain,
        look,
        body,
        pose: (position, rotation),
        velocity: (linear, angular),
        ..
    } = cut;
    let q = json!({ "x": rotation.x, "y": rotation.y, "z": rotation.z, "w": rotation.w });
    let velocity = json!({
        "linear": jv(linear.extend(0.0)),
        "angular": { "x": 0.0, "y": 0.0, "z": angular },
    });
    let piece = json!({
        "terrain": terrain as f64, "width": sub.width, "height": sub.height,
        "cells": sub.count, "tick": tick.min(u64::from(u32::MAX)) as u32,
    });
    let e = ctx.spawn_bytes(vec![
        (
            l.transform.0,
            l.transform
                .1
                .enc(&json!({ "position": jv(position), "quaternion": q }))?,
        ),
        (l.rigid.0, l.rigid.1.enc(&json!({ "kind": "dynamic" }))?),
        (l.velocity.0, l.velocity.1.enc(&velocity)?),
        (l.piece.0, l.piece.1.enc(&piece)?),
    ])?;
    let mut colliders = Vec::new();
    for [x, y, w, h] in piece_rectangles(sub.width, sub.height, &sub.cells) {
        let collider = json!({
            "shape": og_physics::SHAPE_BOX,
            "size": jv(Vec3::new(w as f32 * look.cell, h as f32 * look.cell, 1.0)),
            "friction": body.friction, "density": body.density,
            "layers": body.layers, "mask": body.mask,
        });
        let at = Vec3::new(x as f32 + 0.5 * w as f32, y as f32 + 0.5 * h as f32, 0.0) * look.cell;
        let child = ctx.spawn_bytes(vec![
            (
                l.transform.0,
                l.transform.1.enc(&json!({"position": jv(at)}))?,
            ),
            (
                l.parent.0,
                l.parent.1.enc(&json!({"entity": e.to_bits() as f64}))?,
            ),
            (l.collider.0, l.collider.1.enc(&collider)?),
        ])?;
        colliders.push(child.to_bits());
    }
    queued_structural();
    st.pieces.insert(
        e.to_bits(),
        Piece {
            terrain,
            width: sub.width,
            height: sub.height,
            cells: sub.cells,
            grain,
            look,
            body,
            colliders,
            max_pieces,
            tick,
        },
    );
    Ok(e.to_bits())
}
