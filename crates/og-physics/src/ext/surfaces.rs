//! og.WaterSurface and og.WaterLine: splashed by and pushing the bodies in them.

use super::*;

/// The bodies water acts on, from the rapier world: every dynamic body (it
/// floats) and every controller (it splashes), with its colliders' box.
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
        });
    }
    list
}

/// Water before the step: each surface and line is splashed by and pushes
/// the bodies in it (buoyancy and drag on dynamic bodies), then its waves
/// travel one tick.
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
) -> Result<()> {
    // The wakes of every surface and sea still there (a sea's are og.Buoyancy's: `buoyancy`).
    let live: BTreeSet<u64> = og_core::sort::set(
        scene
            .surfaces
            .iter()
            .map(|s| s.0.to_bits())
            .chain(waters.iter().filter(|w| w.sea.is_some()).map(|w| w.entity)),
    );
    sim.wakes.retain(|e, _| live.contains(e));
    if scene.surfaces.is_empty() && scene.lines.is_empty() {
        sim.surfaces.clear();
        sim.lines.clear();
        return Ok(());
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
    Ok(())
}
