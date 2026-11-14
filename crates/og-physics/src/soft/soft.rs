//! Ropes, cloth, soft bodies and garments, and the fluids stepped beside them.

use super::*;

/// Where a tracked body or collider entity stands now (rope ends).
fn position_of(scene: &Scene, entity: u64) -> Option<Vec3> {
    scene
        .bodies
        .iter()
        .find(|b| b.entity.to_bits() == entity)
        .map(|b| b.position)
        .or_else(|| {
            scene
                .statics
                .iter()
                .find(|s| s.entity.to_bits() == entity)
                .map(|s| s.position)
        })
}

/// The bodies' final positions this tick, by entity (controllers moved).
fn final_position(scene: &Scene, out: &StepOut, entity: u64) -> Option<Vec3> {
    for (k, b) in scene.bodies.iter().enumerate() {
        if b.entity.to_bits() == entity {
            return Some(out.moved[k].map_or(b.position, |m| m.0));
        }
    }
    position_of(scene, entity)
}

/// Steps every rope and cloth, fluid and water after the bodies moved.
///
/// Each rope, each cloth and each fluid (alone, or its group's set) steps
/// from its own state and the solid colliders around it, read only, so
/// they step side by side on the lanes once their work gives two tasks of
/// a grain (`crate::lanes`), each one exactly as it would alone; building
/// and rebuilding them stays serial, in entity order.
#[allow(clippy::too_many_arguments)]
pub(crate) fn soft(
    scene: &Scene,
    settings: &Physics,
    dt: f32,
    tick: u64,
    shared: &mut crate::Shared,
    sim: &mut Sim,
    out: &mut StepOut,
    laps: &mut crate::stats::Laps,
) -> Result<()> {
    use crate::lanes::{Soft, soft_each};
    use crate::stats::Phase;
    let grains = crate::lanes::now();
    let flat2 = DIM == 2;
    let gravity = flat(settings.gravity);
    // The world's air pushes each beside its own wind, as it blows where
    // the body hangs (the middle of its points).
    let air = shared.air.clone();
    let air = &air;
    // The force fields accelerate each as gravity does, where it is.
    let forces = shared.forces.clone();
    let forces = &forces;
    // Ropes.
    let live: BTreeSet<u64> = og_core::sort::set(scene.ropes.iter().map(|r| r.0.to_bits()));
    sim.ropes.retain(|e, _| live.contains(e));
    let mut ropes: IdMap<u64, (&Rope, Vec3, Option<Vec3>)> = IdMap::new();
    for (e, position, rope) in &scene.ropes {
        let start = flat_keep(*position + rope.offset, position.z, flat2);
        let end = if rope.end != 0 {
            final_position(scene, out, rope.end)
                .map(|p| flat_keep(p + rope.end_offset, position.z, flat2))
        } else {
            None
        };
        let key = e.to_bits();
        let rebuild = sim
            .ropes
            .get(&key)
            .is_none_or(|s| s.segments != rope.segments || s.length != rope.length);
        if rebuild {
            sim.ropes.insert(key, RopeState::new(rope, start, end));
        }
        ropes.insert(key, (rope, start, end));
    }
    let colliders = &sim.world.colliders;
    let mut jobs: Vec<_> = sim.ropes.iter_mut().map(|(k, s)| (s, &ropes[k])).collect();
    let work = jobs
        .iter()
        .map(|(s, (rope, ..))| s.points.len() * rope.iterations.clamp(1, 64) as usize)
        .sum();
    soft_each(Soft::Ropes, &mut jobs, work, grains.rope_work, |job| {
        let (rope, start, end) = *job.1;
        let state = &mut *job.0;
        let (min, max) = bounds(&state.points, rope.length);
        let near = Near::around(colliders, min, max, rope.radius * 2.0 + 0.1);
        let middle = (min + max) * 0.5;
        let rope = &Rope {
            wind: rope.wind + flat(air.at(middle) + forces.at(middle)),
            ..*rope
        };
        state.step(rope, dt, gravity, start, end, flat2, &near);
    });
    // Cloth.
    let live: BTreeSet<u64> = og_core::sort::set(scene.cloths.iter().map(|c| c.0.to_bits()));
    sim.cloths.retain(|e, _| live.contains(e));
    let mut cloths: IdMap<u64, (&Cloth, Vec3, Quat)> = IdMap::new();
    for (e, position, rotation, cloth) in &scene.cloths {
        let key = e.to_bits();
        let rotation = if flat2 { Quat::IDENTITY } else { *rotation };
        let rebuild = sim.cloths.get(&key).is_none_or(|s| {
            s.columns != cloth.columns
                || s.rows != cloth.rows
                || s.spacing != cloth.spacing
                || s.pins != cloth.pins
        });
        if rebuild {
            sim.cloths
                .insert(key, ClothState::new(cloth, *position, rotation));
        }
        if cloth.simulation == SIMULATION_VISUAL {
            // The renderer simulates it; the tick carries its rest shape.
            sim.cloths
                .get_mut(&key)
                .expect("built above")
                .hold(cloth, *position, rotation);
            continue;
        }
        cloths.insert(key, (cloth, *position, rotation));
    }
    let colliders = &sim.world.colliders;
    let mut jobs: Vec<_> = sim
        .cloths
        .iter_mut()
        .filter_map(|(k, s)| cloths.get(k).map(|c| (s, c)))
        .collect();
    let work = jobs
        .iter()
        .map(|(s, (cloth, ..))| {
            s.points.len()
                * cloth.iterations.clamp(1, 64) as usize
                * cloth.substeps.clamp(1, 16) as usize
        })
        .sum();
    soft_each(Soft::Cloths, &mut jobs, work, grains.cloth_work, |job| {
        let (cloth, position, rotation) = *job.1;
        let state = &mut *job.0;
        let size = cloth.spacing * (cloth.columns.max(cloth.rows)) as f32;
        let (min, max) = bounds(&state.points, size);
        let near = Near::around(colliders, min, max, cloth.radius * 2.0 + 0.1);
        let middle = (min + max) * 0.5;
        let cloth = &Cloth {
            wind: cloth.wind + flat(air.at(middle) + forces.at(middle)),
            ..*cloth
        };
        state.step(cloth, dt, tick, gravity, position, rotation, flat2, &near);
    });
    // Soft volumes (og.SoftBody), kept in the block's trailer.
    let live: BTreeSet<u64> = og_core::sort::set(scene.softs.iter().map(|s| s.0.to_bits()));
    sim.ext.softs.retain(|e, _| live.contains(e));
    let mut softs: IdMap<u64, (&SoftBody, Vec3, Quat)> = IdMap::new();
    for (e, position, rotation, body) in &scene.softs {
        let key = e.to_bits();
        let rebuild = sim
            .ext
            .softs
            .get(&key)
            .is_none_or(|s| s.key != crate::softbody::SoftKey::of(body));
        if rebuild {
            let mesh = (body.shape == SOFT_MESH)
                .then(|| super::super::sim::mesh(shared, body.mesh))
                .flatten();
            let state =
                crate::softbody::SoftState::new(body, *position, *rotation, flat2, mesh.as_deref())
                    .map_err(|e| e.context(format!("og.SoftBody of entity {key}")))?;
            sim.ext.softs.insert(key, state);
        }
        if body.simulation == SIMULATION_VISUAL {
            sim.ext
                .softs
                .get_mut(&key)
                .expect("built above")
                .hold(*position, *rotation);
        } else {
            softs.insert(key, (body, *position, *rotation));
        }
    }
    let colliders = &sim.world.colliders;
    let mut jobs: Vec<_> = sim
        .ext
        .softs
        .iter_mut()
        .filter_map(|(k, s)| softs.get(k).map(|b| (s, b)))
        .collect();
    let work = jobs
        .iter()
        .map(|(s, (body, ..))| {
            s.points.len()
                * body.iterations.clamp(1, 64) as usize
                * body.substeps.clamp(1, 16) as usize
        })
        .sum();
    soft_each(Soft::Cloths, &mut jobs, work, grains.cloth_work, |job| {
        let (body, position, rotation) = *job.1;
        let state = &mut *job.0;
        let (min, max) = bounds(&state.points, 0.0);
        let near = Near::around(colliders, min, max, body.thickness * 2.0 + 0.5);
        let gravity = gravity + flat(forces.at((min + max) * 0.5));
        state.step(body, dt, gravity, position, rotation, &near);
    });
    for (e, _, _, body) in &scene.softs {
        let s = &sim.ext.softs[&e.to_bits()];
        let volume = if body.simulation == SIMULATION_VISUAL {
            1.0
        } else if s.rest_volume > 0.0 {
            s.volume() / s.rest_volume
        } else {
            1.0
        };
        out.softs.push((
            *e,
            SoftBody {
                volume,
                center: s.center(),
                ..*body
            },
        ));
    }
    // Garments (og.Garment, 3D), each towards the pose the host gave it
    // this tick; one given none keeps its points.
    let live: BTreeSet<u64> = og_core::sort::set(scene.garments.iter().map(|g| g.0.to_bits()));
    sim.ext.garments.retain(|e, _| live.contains(e));
    let poses = std::mem::take(&mut shared.garment_poses);
    if !flat2 {
        let mut garments: IdMap<u64, (&Garment, &crate::GarmentPose)> = IdMap::new();
        for (e, garment) in &scene.garments {
            let key = e.to_bits();
            let (Some((rig_key, rig)), Some(pose)) =
                (shared.garment_rigs.get(&key), poses.get(&key))
            else {
                continue;
            };
            let n = rig.rest.len();
            if pose.targets.len() != n || pose.normals.len() != n {
                return Err(og_core::Error::new(
                    og_core::ErrorKind::Invalid,
                    format!(
                        "og.Garment of entity {key}: a pose of {} targets and {} normals for a rig of {n} points",
                        pose.targets.len(),
                        pose.normals.len()
                    ),
                ));
            }
            let rebuild = sim
                .ext
                .garments
                .get(&key)
                .is_none_or(|s| s.key != (*rig_key, garment.pin_top.to_bits()));
            if rebuild {
                sim.ext.garments.insert(
                    key,
                    crate::garment::GarmentState::new(garment, *rig_key, rig, pose),
                );
            }
            if garment.simulation == SIMULATION_VISUAL {
                sim.ext
                    .garments
                    .get_mut(&key)
                    .expect("built above")
                    .hold(pose);
            } else {
                garments.insert(key, (garment, pose));
            }
        }
        let colliders = &sim.world.colliders;
        let mut jobs: Vec<_> = sim
            .ext
            .garments
            .iter_mut()
            .filter_map(|(k, s)| garments.get(k).map(|g| (s, *g)))
            .collect();
        let work = jobs
            .iter()
            .map(|(s, (g, _))| {
                s.points.len()
                    * g.iterations.clamp(1, 64) as usize
                    * g.substeps.clamp(1, 16) as usize
            })
            .sum();
        soft_each(Soft::Cloths, &mut jobs, work, grains.cloth_work, |job| {
            let (garment, pose) = job.1;
            let state = &mut *job.0;
            let (min, max) = bounds(&state.points, garment.max_distance);
            let near = Near::around(colliders, min, max, garment.radius * 2.0 + 0.1);
            let middle = (min + max) * 0.5;
            let garment = &Garment {
                wind: garment.wind + flat(air.at(middle) + forces.at(middle)),
                ..*garment
            };
            state.step(garment, pose, dt, tick, gravity, &near);
        });
    }
    laps.lap(Phase::Soft);
    // Fluids: each alone, or with the others of its group (the first four
    // in entity order); each set steps from its own particles.
    let live: BTreeSet<u64> = og_core::sort::set(scene.fluids.iter().map(|f| f.0.to_bits()));
    sim.fluids.retain(|e, _| live.contains(e));
    let g = Vec2::new(settings.gravity.x, settings.gravity.y);
    let mut groups: IdMap<u32, Vec<usize>> = IdMap::new();
    for (k, (e, position, fluid)) in scene.fluids.iter().enumerate() {
        let key = e.to_bits();
        let centre = Vec2::new(position.x, position.y);
        let rebuild = sim
            .fluids
            .get(&key)
            .is_none_or(|s| s.count != fluid.count || s.seed != fluid.seed);
        if rebuild {
            sim.fluids.insert(key, FluidState::new(fluid, centre));
        }
        if fluid.group != 0 {
            groups.entry(fluid.group).or_default().push(k);
        }
    }
    let mut sets: Vec<Vec<usize>> = Vec::new();
    let mut grouped: BTreeSet<u64> = BTreeSet::new();
    for members in groups.values_mut() {
        og_core::sort::sort_by_key(members, |&k| scene.fluids[k].0.to_bits());
        members.truncate(crate::fluid::GROUP_SIZE);
        if members.len() < 2 {
            continue;
        }
        grouped.extend(members.iter().map(|&k| scene.fluids[k].0.to_bits()));
        sets.push(members.clone());
    }
    for (k, (e, ..)) in scene.fluids.iter().enumerate() {
        if !grouped.contains(&e.to_bits()) {
            sets.push(vec![k]);
        }
    }
    let mut jobs: Vec<FluidSet<'_, FluidState>> = sets
        .iter()
        .map(|members| {
            let taken = members
                .iter()
                .map(|&k| {
                    let key = scene.fluids[k].0.to_bits();
                    (key, sim.fluids.remove(&key).expect("built above"))
                })
                .collect();
            (members.as_slice(), taken)
        })
        .collect();
    let work = jobs
        .iter()
        .map(|(members, taken)| {
            let particles: usize = taken.iter().map(|(_, s)| s.positions.len()).sum();
            let substeps = members
                .iter()
                .map(|&k| scene.fluids[k].2.substeps.clamp(1, 8))
                .max()
                .unwrap_or(1);
            particles * substeps as usize
        })
        .sum();
    let colliders = &sim.world.colliders;
    soft_each(Soft::Fluids, &mut jobs, work, grains.fluid_work, |job| {
        let (members, taken) = (job.0, &mut job.1);
        fluid_set(scene, members, taken, dt, g, forces, colliders);
    });
    for (_, taken) in jobs {
        sim.fluids.extend(taken);
    }
    laps.lap(Phase::Fluids);
    fluids3(scene, settings, forces, dt, sim);
    laps.lap(Phase::Fluids3d);
    Ok(())
}

fn flat_keep(p: Vec3, z: f32, flat2: bool) -> Vec3 {
    if flat2 { Vec3::new(p.x, p.y, z) } else { p }
}

fn bounds(points: &[Vec3], slack: f32) -> (Vec3, Vec3) {
    let (mut min, mut max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for p in points {
        min = min.min(*p);
        max = max.max(*p);
    }
    let grow = Vec3::splat(slack.max(0.0) * 0.25);
    (min - grow, max + grow)
}
