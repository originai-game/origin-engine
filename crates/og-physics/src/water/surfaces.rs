//! Water before the step: every og.Water, og.WaterSurface and og.WaterLine splashed by the bodies in it.

use super::*;

/// The bodies water acts on, from the rapier world: every dynamic body (it
/// floats), every controller (it splashes) and every og.Ragdoll with
/// og.Buoyancy (it splashes), with its colliders' box, in entity order.
fn floaters(
    scene: &Scene,
    sim: &mut Sim,
    handles: &super::super::sim::Handles<'_>,
) -> Vec<Floater> {
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
            let c = &sim.world.colliders[c];
            if c.is_sensor() {
                continue;
            }
            let aabb = c.compute_aabb();
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
        let centre = (min + max) * 0.5;
        let samples = scene
            .floats
            .iter()
            .find(|f| f.0 == b.entity)
            .map_or(4, |f| f.1.samples);
        let hull = super::buoyancy::cached_hull(
            &mut sim.ext.water_hulls,
            &sim.world.colliders,
            rb,
            h,
            samples,
        );
        let triangles = hull.map_or_else(Vec::new, |h| h.world_triangles(rb.position(), centre));
        let volume = hull.map_or(0.0, |h| h.volume());
        list.push(Floater {
            entity: b.entity.to_bits(),
            min,
            max,
            centre: (min + max) * 0.5,
            velocity: flat(velocity),
            dynamic,
            hull: floats,
            triangles,
            volume,
        });
    }
    // A ragdoll with og.Buoyancy (its bones are bodies of no entity): its
    // bones' box splashes; its lift and its cut are `buoyancy`'s.
    let bodies = list.len();
    for (e, buoyancy) in &scene.floats {
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
        let (mut triangles, mut volume) = (Vec::new(), 0.0);
        for &h in &state.bones {
            let rb = &sim.world.bodies[h];
            let Some((lo, hi)) = super::buoyancy::solid_box(&sim.world, rb) else {
                continue;
            };
            min = min.min(lo);
            max = max.max(hi);
            velocity += unv(rb.linvel()) * rb.mass();
            mass += rb.mass();
            if let Some(hull) = super::buoyancy::cached_hull(
                &mut sim.ext.water_hulls,
                &sim.world.colliders,
                rb,
                h,
                buoyancy.samples,
            ) {
                triangles.extend(hull.world_triangles(rb.position(), Vec3::ZERO));
                volume += hull.volume();
            }
        }
        if min.x > max.x || mass <= 0.0 {
            continue;
        }
        let centre = (min + max) * 0.5;
        for triangle in &mut triangles {
            for point in triangle {
                *point -= centre;
            }
        }
        list.push(Floater {
            entity: key,
            min,
            max,
            centre,
            velocity: flat(velocity / mass),
            dynamic: false,
            hull: true,
            triangles,
            volume,
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
        let crossings = state.step(surface, dt, *position, &list);
        let (size, cell) = (state.size, state.cell());
        sim.wakes
            .entry(key)
            .or_default()
            .record(surface, size, cell, dt, tick, *position, &list);
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
        let crossings = state.step(line, dt, g, centre, &list);
        for c in crossings {
            out.splashes.push(Splash {
                water: key,
                entity: c.entity,
                point: c.point,
                speed: c.speed,
            });
        }
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

/// Automatic pool buoyancy shares the substep force lifetime with hull
/// buoyancy. Each body chooses one enclosing pool, highest level first;
/// overlaps never add a second lift or read a velocity modified by water.
pub(super) fn surface_buoyancy_substep(
    scene: &Scene,
    settings: &Physics,
    dt: f32,
    sim: &mut Sim,
    handles: &super::super::sim::Handles<'_>,
    forced: &mut Vec<rp::RigidBodyHandle>,
) {
    if scene.surfaces.is_empty() && scene.lines.is_empty() {
        return;
    }
    for f in floaters(scene, sim, handles).iter().filter(|f| f.dynamic) {
        let surfaces = scene
            .surfaces
            .iter()
            .filter(|(_, p, w)| {
                let half = w.size * 0.5;
                w.depth > 0.0
                    && f.min.x <= p.x + half
                    && f.max.x >= p.x - half
                    && f.min.z <= p.z + half
                    && f.max.z >= p.z - half
                    && f.min.y <= p.y
                    && f.max.y >= p.y - w.depth
            })
            .map(|(_, p, w)| {
                (
                    p.y,
                    w.depth,
                    w.buoyancy,
                    w.drag,
                    *p - Vec3::new(w.size * 0.5, 0.0, w.size * 0.5),
                    *p + Vec3::new(w.size * 0.5, 0.0, w.size * 0.5),
                )
            });
        let lines = scene
            .lines
            .iter()
            .filter(|(_, p, w)| {
                w.depth > 0.0
                    && f.min.x <= p.x + w.width * 0.5
                    && f.max.x >= p.x - w.width * 0.5
                    && f.min.y <= p.y
                    && f.max.y >= p.y - w.depth
            })
            .map(|(_, p, w)| {
                (
                    p.y,
                    w.depth,
                    w.buoyancy,
                    w.drag,
                    Vec3::new(p.x - w.width * 0.5, p.y, f.min.z),
                    Vec3::new(p.x + w.width * 0.5, p.y, f.max.z),
                )
            });
        let mut best: Option<(f32, f32, f32, f32, Vec3, Vec3)> = None;
        for pool in surfaces.chain(lines) {
            if best.is_none_or(|b| pool.0 > b.0) {
                best = Some(pool);
            }
        }
        let Some((level, depth, lift, drag, min, max)) = best else {
            continue;
        };
        let (volume, centre, _) = f.immersed(level, depth, min, max);
        if volume <= 0.0 || f.volume <= 0.0 {
            continue;
        }
        let fraction = (volume / f.volume).clamp(0.0, 1.0);
        let h = *handles.get(&f.entity).expect("floater has a body");
        let rb = &mut sim.world.bodies[h];
        let com = unv(rb.center_of_mass());
        let buoyancy = Vec3::Y * (lift * volume * (-settings.gravity.y / 9.81));
        let keep = og_core::math::exp(-drag.max(0.0) * fraction * dt);
        let resistance = if dt > 0.0 {
            f.velocity * (rb.mass() * (1.0 - keep) / dt)
        } else {
            Vec3::ZERO
        };
        rb.add_force(v(flat(buoyancy - resistance)), true);
        rb.add_torque(angvel((centre - com).cross(buoyancy)), true);
        rb.set_angvel(angvel(unangvel(rb.angvel()) * keep), true);
        forced.push(h);
    }
}
