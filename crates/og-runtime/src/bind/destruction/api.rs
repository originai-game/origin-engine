//! The og.destruction calls.

use super::*;

pub(super) fn row_of<T: FromJson>(eng: &Engine, name: &str, e: Entity) -> Option<T> {
    let w = eng.world().ok()?;
    let id = w.id_by_name(name).ok()?;
    let bytes = w.get_bytes(e, id).ok()?;
    let layout = eng.state.borrow().layouts.get(name).cloned()?;
    layout.read(&bytes).ok()
}

/// The og.Destructible `e` names: itself, or the one its og.Chunk is of.
pub(super) fn destructible_of(eng: &Engine, e: Entity, chunks: bool) -> Result<Entity, String> {
    let w = eng.world()?;
    if !w.is_alive(e) {
        return Err(format!("entity {} is not alive", e.to_bits()));
    }
    if row_of::<record::Destructible>(eng, DESTRUCTIBLE, e).is_some() {
        return Ok(e);
    }
    if chunks && let Some(c) = row_of::<record::Chunk>(eng, CHUNK, e) {
        return Ok(Entity::from_bits(c.destructible));
    }
    Err(format!(
        "entity {} has no og.Destructible{}",
        e.to_bits(),
        if chunks { " or og.Chunk" } else { "" }
    ))
}

pub(super) fn point(p: Vec3, what: &str) -> Result<[f32; 3], String> {
    if p.is_finite() {
        Ok(p.to_array())
    } else {
        Err(format!("{what} is not finite"))
    }
}

pub(super) fn radius(r: f32, key: &str) -> Result<f32, String> {
    if r.is_finite() && r >= 0.0 {
        Ok(r)
    } else {
        Err(format!(
            "desc.{key} {r} is not a finite, non-negative number"
        ))
    }
}

pub fn og_destruction_damage(world: u32, entity: f64, desc: Option<String>) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        let target = destructible_of(&eng, Entity::from_bits(entity as u64), true)?;
        let d: DamageDesc = wire::arg(desc.as_deref(), "desc")?;
        let op = Op::Damage {
            entity: target.to_bits(),
            point: point(d.point, "desc.point")?,
            radius: radius(d.radius, "radius")?.max(1e-3),
            inner: radius(d.inner_radius, "innerRadius")?,
            amount: radius(d.amount, "amount")?,
            impulse: point(d.impulse, "desc.impulse")?,
        };
        lock(&eng.modules.destruction.state).ops.push(op);
        Ok(())
    })()
}

pub fn og_destruction_explode(world: u32, desc: Option<String>) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        let d: ExplosionDesc = wire::arg(desc.as_deref(), "desc")?;
        let op = Op::Explode {
            point: point(d.point, "desc.point")?,
            radius: radius(d.radius, "radius")?.max(1e-3),
            inner: radius(d.inner_radius, "innerRadius")?,
            damage: radius(d.damage, "damage")?,
            impulse: radius(d.impulse, "impulse")?,
        };
        // A world with no destructible has nothing to break.
        if eng.world()?.systems().contains(&SYSTEM) {
            lock(&eng.modules.destruction.state).ops.push(op);
        }
        Ok(())
    })()
}

pub fn og_destruction_fracture(world: u32, entity: f64) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        let target = destructible_of(&eng, Entity::from_bits(entity as u64), false)?;
        lock(&eng.modules.destruction.state).ops.push(Op::Fracture {
            entity: target.to_bits(),
        });
        Ok(())
    })()
}

pub fn og_destruction_info(
    world: u32,
    entity: f64,
) -> Result<crate::host::wire::DestructibleInfo, String> {
    (|| {
        let eng = engine(world)?;
        let e = destructible_of(&eng, Entity::from_bits(entity as u64), false)?;
        let model: record::Model = row_of(&eng, "og.Model", e)
            .ok_or_else(|| format!("entity {} has no og.Model", e.to_bits()))?;
        let asset = model.asset;
        let m = &eng.modules.destruction;
        let a = lock(&m.assets).get(&asset).cloned().ok_or_else(|| {
            format!(
                "entity {}'s og.Model is not loaded, or was not cooked with a `fracture` block",
                e.to_bits()
            )
        })?;
        let (inst, debris, crumbs) = {
            let st = lock(&m.state);
            let inst = st
                .props
                .get(&e.to_bits())
                .filter(|p| p.asset == asset)
                .map(|p| p.inst.clone())
                .unwrap_or_else(|| Instance::new(&a));
            let k = e.to_bits();
            (
                inst,
                st.debris.iter().filter(|d| d.prop == k).count(),
                st.crumbs.iter().filter(|c| c.prop == k).count(),
            )
        };
        let total: f32 = a.bonds.iter().map(|b| b.area).sum();
        let left: f32 = a
            .bonds
            .iter()
            .zip(&inst.bonds)
            .map(|(b, h)| b.area * h)
            .sum();
        let support = a.support as usize;
        Ok::<_, String>(DestructibleInfo {
            chunks: a.support,
            children: (a.chunks.len() - support) as u32,
            bonds: a.bonds.len() as u32,
            intact: inst.bonds.iter().filter(|&&h| h > 0.0).count() as u32,
            islands: inst.islands(&a).len() as u32,
            anchored: (0..support)
                .filter(|&c| inst.anchors[c] > 0.0 && !inst.shattered[c])
                .count() as u32,
            health: if total > 0.0 { left / total } else { 1.0 },
            debris: debris as u32,
            crumbs: crumbs as u32,
        })
    })()
}

pub fn og_destruction_material(
    world: u32,
    kind: u32,
) -> Result<crate::host::wire::DestructibleMaterial, String> {
    engine(world)?;
    let Some(m) = MATERIALS.get(kind as usize) else {
        return Err(format!("{kind} is none of og.MATERIAL_*"));
    };
    Ok(DestructibleMaterial {
        toughness: m[0],
        strength: m[1],
        friction: m[2],
        tension: m[3],
        density: m[4],
        impact_threshold: m[5],
        impact_radius: m[6],
    })
}

pub fn og_destruction_capabilities(world: u32) -> crate::host::wire::DestructionCapabilities {
    // og.Destructible.debris is the renderer's debris layer and a GPU
    // particle burst: drawn where a renderer runs compute passes (every
    // renderer the engine makes: og.render.capabilities' compute).
    let compute = engine(world).is_ok_and(|e| e.render.borrow().backend.is_some());
    DestructionCapabilities {
        visual_debris: compute,
        max_visual_debris: og_render::MAX_DEBRIS as u32,
        visual_debris_contacts: "world".into(),
    }
}
