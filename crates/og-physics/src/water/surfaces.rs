//! Water before the step: every og.Water, og.WaterSurface and og.WaterLine splashed by the bodies in it.

use super::*;

/// The bodies water acts on, from the rapier world: every dynamic body (it
/// floats), every controller (it splashes) and every og.Ragdoll with
/// og.Buoyancy (it splashes), with its colliders' box, in entity order.
fn floaters(scene: &Scene, sim: &Sim, handles: &super::super::sim::Handles<'_>) -> Vec<Floater> {
    let mut list = Vec::new();
    for b in &scene.bodies {
        let moving = b.body.kind == BODY_DYNAMIC && b.control.is_none();
        if !(moving || b.control.is_some()) {
            continue;
        }
        // A body with og.Buoyancy floats by its own hull (`buoyancy`): the
        // surface only takes its splash.
        let floats = scene
            .floats
            .binary_search_by_key(&b.entity.to_bits(), |f| f.0.to_bits())
            .is_ok();
        let dynamic = moving && !floats;
        let Some(&h) = handles.get(&b.entity.to_bits()) else {
            continue;
        };
        let rb = &sim.world.bodies[h];
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        for &c in rb.colliders() {
            let aabb = sim.world.colliders[c].compute_aabb();
            min = min.min(unv(aabb.mins));
            max = max.max(unv(aabb.maxs));
        }
        if min.x > max.x {
            continue;
        }
        if DIM == 2 {
            min.z = 0.0;
            max.z = 0.0;
        }
        let velocity = match b.control {
            Some(crate::gather::Control::Character(c)) => c.velocity,
            Some(crate::gather::Control::Platformer(p)) => p.velocity,
            None => unv(rb.linvel()),
        };
        list.push(Floater {
            entity: b.entity.to_bits(),
            min,
            max,
            centre: (min + max) * 0.5,
            velocity: flat(velocity),
            dynamic,
            hull: floats,
        });
    }
    // A ragdoll with og.Buoyancy (its bones are bodies of no entity): its
    // bones' box splashes; its lift and its cut are `buoyancy`'s.
    let bodies = list.len();
    for (e, _) in &scene.floats {
        let key = e.to_bits();
        let Some(state) = sim.ragdolls.get(&key) else {
            continue;
        };
        if list[..bodies].iter().any(|f| f.entity == key) {
            continue;
        }
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        let (mut velocity, mut mass) = (Vec3::ZERO, 0.0);
        for &h in &state.bones {
            let rb = &sim.world.bodies[h];
            let Some((lo, hi)) = super::buoyancy::solid_box(&sim.world, rb) else {
                continue;
            };
            min = min.min(lo);
            max = max.max(hi);
            velocity += unv(rb.linvel()) * rb.mass();
            mass += rb.mass();
        }
        if min.x > max.x || mass <= 0.0 {
            continue;
        }
        list.push(Floater {
            entity: key,
            min,
            max,
            centre: (min + max) * 0.5,
            velocity: flat(velocity / mass),
            dynamic: false,
            hull: true,
        });
    }
    if list.len() > bodies {
        og_core::sort::sort_by_key(&mut list, |f| f.entity);
    }
    list
}

/// Water before the step: each surface and line is splashed by and pushes
/// the bodies in it (buoyancy and drag on dynamic bodies), then its waves
/// travel one tick; each og.Water (`waters`, 3D) is crossed and cut by the
/// bodies at its surface ([`crate::water::submersion`]). Every crossing is
/// an og.Splash. Returns the og.Water cuts, for [`buoyancy`] to add its
/// hulls to and record.
#[allow(clippy::too_many_arguments)]
pub(crate) fn water(
    scene: &Scene,
    settings: &Physics,
    dt: f32,
    tick: u64,
    waters: &[crate::WaterBody],
    sim: &mut Sim,
    handles: &super::super::sim::Handles<'_>,
    out: &mut StepOut,
) -> Result<BTreeMap<u64, Vec<crate::water::WaterCut>>> {
    let waters = if DIM == 3 { waters } else { &[] };
    // The wakes of every surface and og.Water still there.
    let live: BTreeSet<u64> = og_core::sort::set(
        scene
            .surfaces
            .iter()
            .map(|s| s.0.to_bits())
            .chain(waters.iter().map(|w| w.entity)),
    );
    sim.wakes.retain(|e, _| live.contains(e));
    if scene.surfaces.is_empty() && scene.lines.is_empty() && waters.is_empty() {
        sim.surfaces.clear();
        sim.lines.clear();
        sim.ext.under.clear();
        return Ok(BTreeMap::new());
    }
    let list = floaters(scene, sim, handles);
    let g = -settings.gravity.y;
    let mut forces: Vec<WaterForce> = Vec::new();
    sim.surfaces.retain(|e, _| live.contains(e));
    for (e, position, surface) in &scene.surfaces {
        let key = e.to_bits();
        let rebuild = sim
            .surfaces
            .get(&key)
            .is_none_or(|s| s.resolution != surface.resolution || s.size != surface.size);
        if rebuild {
            sim.surfaces.insert(key, SurfaceState::new(surface));
        }
        let state = sim.surfaces.get_mut(&key).expect("inserted");
        let (f, crossings) = state.step(surface, dt, g, *position, &list);
        let (size, cell) = (state.size, state.cell());
        sim.wakes
            .entry(key)
            .or_default()
            .record(surface, size, cell, dt, tick, *position, &list);
        forces.extend(f);
        for c in crossings {
            out.splashes.push(Splash {
                water: key,
                entity: c.entity,
                point: c.point,
                speed: c.speed,
            });
        }
    }
    let live: BTreeSet<u64> = og_core::sort::set(scene.lines.iter().map(|s| s.0.to_bits()));
    sim.lines.retain(|e, _| live.contains(e));
    for (e, position, line) in &scene.lines {
        let key = e.to_bits();
        let rebuild = sim
            .lines
            .get(&key)
            .is_none_or(|s| s.columns != line.columns || s.width != line.width);
        if rebuild {
            sim.lines.insert(key, LineState::new(line));
        }
        let state = sim.lines.get_mut(&key).expect("inserted");
        let centre = Vec2::new(position.x, position.y);
        let (f, crossings) = state.step(line, dt, g, centre, &list);
        forces.extend(f);
        for c in crossings {
            out.splashes.push(Splash {
                water: key,
                entity: c.entity,
                point: c.point,
                speed: c.speed,
            });
        }
    }
    for f in forces {
        let Some(&h) = handles.get(&f.entity) else {
            continue;
        };
        let rb = &mut sim.world.bodies[h];
        let linvel = rb.linvel();
        rb.set_linvel(linvel * f.keep, true);
        rb.apply_impulse(v(flat(f.impulse)), true);
    }
    // An og.Water with an og.WaterSurface is that surface's: it has splashed above.
    let alone = waters
        .iter()
        .filter(|w| !scene.surfaces.iter().any(|s| s.0.to_bits() == w.entity));
    let wet = crate::water::submersion::step(alone, &list, scene.seconds, dt, &mut sim.ext.under);
    for (water, c) in wet.crossings {
        out.splashes.push(Splash {
            water,
            entity: c.entity,
            point: c.point,
            speed: c.speed,
        });
    }
    Ok(wet.cuts)
}
