//! The 2D and 3D particle fluids, alone or in their groups.

use super::*;

/// A fluid set as it steps: its members (indices into the scene's fluids)
/// and their states, taken out of the simulation by entity.
pub(super) type FluidSet<'a, S> = (&'a [usize], Vec<(u64, S)>);

/// One tick of a 2D fluid alone (`members` one index into `scene.fluids`)
/// or of a group's fluids together, against the solids around them.
pub(super) fn fluid_set(
    scene: &Scene,
    members: &[usize],
    taken: &mut [(u64, FluidState)],
    dt: f32,
    g: Vec2,
    colliders: &rp::ColliderSet,
) {
    let grouped = members.len() > 1;
    let (mut min, mut max) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
    let (mut z, mut reach) = (0.0f32, 0.0f32);
    for (slot, ((_, state), &k)) in taken.iter_mut().zip(members).enumerate() {
        let (_, position, fluid) = &scene.fluids[k];
        if grouped {
            state.join(slot as u32);
        } else {
            state.leave();
        }
        for p in &state.positions {
            min = min.min(*p);
            max = max.max(*p);
        }
        if slot == 0 {
            z = position.z;
        }
        reach = reach.max(fluid.radius * 2.0);
    }
    let near = Near::around(
        colliders,
        Vec3::new(min.x, min.y, z),
        Vec3::new(max.x, max.y, z),
        reach,
    );
    let mut list: Vec<crate::fluid::Body<'_, Vec2>> = taken
        .iter_mut()
        .zip(members.iter())
        .map(|((_, state), &k)| {
            let (_, position, fluid) = &scene.fluids[k];
            state.body(fluid, Vec2::new(position.x, position.y))
        })
        .collect();
    crate::fluid::step(&mut list, dt, g, z, &near);
}

/// Steps the 3D fluids (the 3D profile only: the 2D one keeps none), each
/// alone or with the others of its group, as the 2D fluids are grouped:
/// each set from its own particles, side by side on the lanes as
/// [`soft`] steps the 2D ones.
pub(super) fn fluids3(scene: &Scene, settings: &Physics, dt: f32, sim: &mut Sim) {
    use crate::fluid::Fluid3State;
    if DIM != 3 {
        sim.fluids3.clear();
        return;
    }
    let live: BTreeSet<u64> = og_core::sort::set(scene.fluids3.iter().map(|f| f.0.to_bits()));
    sim.fluids3.retain(|e, _| live.contains(e));
    // Each fluid's set: its group's first four (entity order), or itself.
    let mut sets: BTreeMap<(u32, u64), Vec<usize>> = BTreeMap::new();
    let mut groups: IdMap<u32, Vec<usize>> = IdMap::new();
    for (k, (e, position, fluid)) in scene.fluids3.iter().enumerate() {
        let key = e.to_bits();
        if fluid.simulation == SIMULATION_VISUAL {
            // The renderer's (SIMULATION_VISUAL): no particles in the
            // tick, in no group.
            let empty = Fluid3D {
                count: 0,
                emit_rate: 0.0,
                ..*fluid
            };
            if sim
                .fluids3
                .get(&key)
                .is_none_or(|s| !s.positions.is_empty())
            {
                sim.fluids3.insert(key, Fluid3State::new(&empty, *position));
            }
            continue;
        }
        let rebuild = sim
            .fluids3
            .get(&key)
            .is_none_or(|s| s.count != fluid.count || s.seed != fluid.seed);
        if rebuild {
            sim.fluids3.insert(key, Fluid3State::new(fluid, *position));
        }
        if fluid.group != 0 {
            groups.entry(fluid.group).or_default().push(k);
        } else {
            sets.insert((0, key), vec![k]);
        }
    }
    for (group, mut members) in groups {
        og_core::sort::sort_by_key(&mut members, |&k| scene.fluids3[k].0.to_bits());
        for (rank, &k) in members.iter().enumerate() {
            if rank >= crate::fluid::GROUP_SIZE {
                sets.insert((0, scene.fluids3[k].0.to_bits()), vec![k]);
            }
        }
        members.truncate(crate::fluid::GROUP_SIZE);
        if members.len() == 1 {
            sets.insert((0, scene.fluids3[members[0]].0.to_bits()), members);
        } else {
            sets.insert((group, 0), members);
        }
    }
    let mut jobs: Vec<FluidSet<'_, Fluid3State>> = sets
        .values()
        .map(|members| {
            let taken = members
                .iter()
                .map(|&k| {
                    let key = scene.fluids3[k].0.to_bits();
                    (key, sim.fluids3.remove(&key).expect("built above"))
                })
                .collect();
            (members.as_slice(), taken)
        })
        .collect();
    let substeps_of = |members: &[usize]| {
        members
            .iter()
            .map(|&k| scene.fluids3[k].2.substeps.clamp(1, 8))
            .max()
            .unwrap_or(1)
    };
    let work = jobs
        .iter()
        .map(|(members, taken)| {
            let particles: usize = taken.iter().map(|(_, s)| s.positions.len()).sum();
            particles * substeps_of(members) as usize
        })
        .sum();
    let per_task = crate::lanes::now().fluid_work;
    let colliders = &sim.world.colliders;
    let gravity = flat(settings.gravity);
    let soft = crate::lanes::Soft::Fluids3d;
    crate::lanes::soft_each(soft, &mut jobs, work, per_task, |job| {
        let (members, taken) = (job.0, &mut job.1);
        let grouped = members.len() > 1;
        let (mut min, mut max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        let mut reach = 0.0f32;
        for (slot, ((_, state), &k)) in taken.iter_mut().zip(members).enumerate() {
            let (_, position, fluid) = &scene.fluids3[k];
            if grouped {
                state.join(slot as u32);
            } else {
                state.leave();
            }
            for p in &state.positions {
                min = min.min(*p);
                max = max.max(*p);
            }
            // The emitter's particles appear this tick: its cube too.
            if fluid.emit_rate > 0.0 {
                let at = *position + fluid.emit_offset;
                let r = Vec3::splat(fluid.emit_radius.max(0.0));
                min = min.min(at - r);
                max = max.max(at + r);
            }
            reach = reach.max(fluid.radius.max(crate::fluid::MIN_RADIUS) * 2.0);
        }
        if min.x > max.x {
            return;
        }
        // A particle moves at most half the radius a substep.
        let substeps = substeps_of(members) as f32;
        let near = Near::around(colliders, min, max, reach * (1.0 + substeps * 0.25));
        for ((_, state), &k) in taken.iter_mut().zip(members.iter()) {
            let (_, position, fluid) = &scene.fluids3[k];
            state.emit(fluid, *position, dt);
        }
        let mut list: Vec<crate::fluid::Body<'_, Vec3>> = taken
            .iter_mut()
            .zip(members.iter())
            .map(|((_, state), &k)| {
                let (_, position, fluid) = &scene.fluids3[k];
                state.body(fluid, *position)
            })
            .collect();
        crate::fluid::step(&mut list, dt, gravity, 0.0, &near);
        drop(list);
        for ((_, state), &k) in taken.iter_mut().zip(members.iter()) {
            let (_, position, fluid) = &scene.fluids3[k];
            state.sink(fluid, *position);
        }
    });
    for (_, taken) in jobs {
        sim.fluids3.extend(taken);
    }
}
