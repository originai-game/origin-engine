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
    colliders: &PointWorld<'_>,
) {
    let grouped = members.len() > 1;
    let mut z = 0.0;
    for (slot, ((_, state), &k)) in taken.iter_mut().zip(members).enumerate() {
        let (_, position, _) = &scene.fluids[k];
        if grouped {
            state.join(slot as u32);
        } else {
            state.leave();
        }
        if slot == 0 {
            z = position.z;
        }
    }
    let near = Near::new(colliders);
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
pub(super) fn fluids3(
    scene: &Scene,
    settings: &Physics,
    forces: &og_env::force::Forces,
    air: &og_env::air::Air,
    dt: f32,
    reactions: &Reactions,
    sim: &mut Sim,
) {
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
            .is_none_or(|s| s.count != fluid.count || s.seed != fluid.seed || s.size != fluid.size);
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
    let point_world = PointWorld {
        world: &sim.world,
        forces,
        air,
        reactions,
    };
    let colliders = &point_world;
    let gravity = flat(settings.gravity);
    let soft = crate::lanes::Soft::Fluids3d;
    crate::lanes::soft_each(soft, &mut jobs, work, per_task, |job| {
        let (members, taken) = (job.0, &mut job.1);
        let grouped = members.len() > 1;
        for (slot, ((_, state), &k)) in taken.iter_mut().zip(members).enumerate() {
            let (e, position, fluid) = &scene.fluids3[k];
            if grouped {
                state.join(slot as u32);
            } else {
                state.leave();
            }
            let rotation = scene
                .fluid_rotations
                .get(&e.to_bits())
                .copied()
                .unwrap_or(Quat::IDENTITY);
            // Truncation must run even for an empty, disabled emitter.
            state.emit(fluid, *position, rotation, dt);
        }
        let near = Near::new(colliders);
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
            let (e, position, fluid) = &scene.fluids3[k];
            let rotation = scene
                .fluid_rotations
                .get(&e.to_bits())
                .copied()
                .unwrap_or(Quat::IDENTITY);
            state.sink(fluid, *position, rotation);
        }
    });
    for (_, taken) in jobs {
        sim.fluids3.extend(taken);
    }
}
