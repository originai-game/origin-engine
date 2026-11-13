//! The og.destruction 2D calls.

use super::*;

/// Whether `e` is alive with record `name`: between ticks from the world, inside a
/// system through its context, which reads only what the system declared
/// (the calls are queued for og.destruction2d after the game's systems, so
/// a system may make them: a hit frame carves, a blast in the tick).
pub(super) fn has(eng: &Engine, name: &str, e: Entity) -> Result<bool, String> {
    if let Some(f) = running(eng.id) {
        // SAFETY: the frame is published while its system runs.
        let ctx = unsafe { f.ctx() };
        if !ctx.is_alive(e) {
            return Err(format!("entity {} is not alive", e.to_bits()));
        }
        let Ok(id) = ctx.id_by_name(name) else {
            return Ok(false);
        };
        return match ctx.get_bytes(e, id) {
            Ok(_) => Ok(true),
            Err(err) if err.kind() == ErrorKind::Access => Err(format!(
                "{err}: inside a system, declare {name} among its reads"
            )),
            Err(_) => Ok(false),
        };
    }
    let w = eng.world()?;
    if !w.is_alive(e) {
        return Err(format!("entity {} is not alive", e.to_bits()));
    }
    Ok(w.id_by_name(name)
        .is_ok_and(|id| w.get_bytes(e, id).is_ok()))
}

pub(super) fn terrain_of(eng: &Engine, entity: f64) -> Result<u64, String> {
    let e = Entity::from_bits(entity as u64);
    if has(eng, TERRAIN, e)? {
        Ok(e.to_bits())
    } else {
        Err(format!("entity {} has no og.PixelTerrain", e.to_bits()))
    }
}

pub(super) fn point2(p: Vec3) -> Result<[f32; 2], String> {
    if p.is_finite() {
        Ok([p.x, p.y])
    } else {
        Err(format!("the point {p} is not finite"))
    }
}

pub(super) fn size(r: f32, key: &str) -> Result<f32, String> {
    if r.is_finite() && r >= 0.0 {
        Ok(r)
    } else {
        Err(format!(
            "desc.{key} {r} is not a finite, non-negative number"
        ))
    }
}

pub(super) fn queue(eng: &Engine, op: Op) {
    lock(&eng.modules.destruction2d.0).state.ops.push(op);
}

pub fn og_destruction_carve2d(world: u32, entity: f64, desc: Option<String>) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        let entity = terrain_of(&eng, entity)?;
        let d: Carve2dDesc = wire::arg(desc.as_deref(), "desc")?;
        queue(
            &eng,
            Op::Carve {
                entity,
                point: point2(d.point)?,
                radius: size(d.radius, "radius")?,
            },
        );
        Ok(())
    })()
}

pub fn og_destruction_fill2d(world: u32, entity: f64, desc: Option<String>) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        let entity = terrain_of(&eng, entity)?;
        let d: Fill2dDesc = wire::arg(desc.as_deref(), "desc")?;
        queue(
            &eng,
            Op::Fill {
                entity,
                point: point2(d.point)?,
                radius: size(d.radius, "radius")?,
                bedrock: d.bedrock,
            },
        );
        Ok(())
    })()
}

pub fn og_destruction_explode2d(world: u32, desc: Option<String>) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        let d: Explosion2dDesc = wire::arg(desc.as_deref(), "desc")?;
        let op = Op::Explode {
            point: point2(d.point)?,
            radius: size(d.radius, "radius")?,
            debris: size(d.debris, "debris")?,
            cells: d.piece_cells.max(4),
            speed: size(d.speed, "speed")?,
        };
        // A world with no pixel terrain (its records, registered with
        // og.destruction2d, unknown) has nothing to blow up. Inside a
        // system the system's context answers, as the world cannot.
        let registered = match running(eng.id) {
            // SAFETY: the frame is published while its system runs.
            Some(f) => unsafe { f.ctx() }.id_by_name(TERRAIN).is_ok(),
            None => eng.world()?.systems().contains(&SYSTEM),
        };
        if registered {
            queue(&eng, op);
        }
        Ok(())
    })()
}

pub fn og_destruction_fracture2d(
    world: u32,
    entity: f64,
    desc: Option<String>,
) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        let e = Entity::from_bits(entity as u64);
        if !has(&eng, TERRAIN, e)? && !has(&eng, PIECE, e)? {
            return Err(format!(
                "entity {} has no og.PixelPiece or og.PixelTerrain",
                e.to_bits()
            ));
        }
        let d: Fracture2dDesc = wire::arg(desc.as_deref(), "desc")?;
        queue(
            &eng,
            Op::Fracture {
                entity: e.to_bits(),
                pieces: d.pieces.clamp(1, 256),
            },
        );
        Ok(())
    })()
}

pub fn og_destruction_cell2d(
    world: u32,
    entity: f64,
    point: String,
) -> Result<serde_json::Value, String> {
    let r = (|| -> Result<u32, String> {
        let eng = engine(world)?;
        let key = terrain_of(&eng, entity)?;
        let p = point2(wire::arg(Some(&point), "point")?)?;
        let g = lock(&eng.modules.destruction2d.0);
        let t = g.state.terrains.get(&key).ok_or_else(|| {
            format!("entity {key}'s og.PixelTerrain is not filled yet (the tick after it is added)")
        })?;
        let (x, y) = (
            ((p[0] - t.origin[0]) / t.look.cell).floor(),
            ((p[1] - t.origin[1]) / t.look.cell).floor(),
        );
        Ok(u32::from(t.grid.get(x as i64, y as i64)))
    })();
    match r {
        Ok(v) => Ok(json!(v)),
        Err(e) => Err(e),
    }
}
