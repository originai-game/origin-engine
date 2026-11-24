//! og.Buoyancy: what floats on the water it lies in (an og.Water's sea, a surface): a body's hull, a limp og.Ragdoll's bones, an og.CharacterController.

use super::*;

/// A body's hull for og.Buoyancy, in its own frame: the centres of the
/// cells of its colliders' box (`samples` a side, `lo` its low corner)
/// that lie in a solid collider, with their cell indices, each standing
/// for an equal share (`each`) of the colliders' volume; `cell` is a
/// cell's size.
struct Hull {
    points: Vec<(Vec3, [u32; 3])>,
    lo: Vec3,
    hi: Vec3,
    cell: Vec3,
    each: f32,
}

fn hull(colliders: &rp::ColliderSet, rb: &rp::RigidBody, samples: u32) -> Option<Hull> {
    let parts: Vec<&rp::Collider> = rb
        .colliders()
        .iter()
        .map(|&c| &colliders[c])
        .filter(|c| !c.is_sensor())
        .collect();
    let at = |c: &rp::Collider| *c.position_wrt_parent().unwrap_or(&rp::Pose::IDENTITY);
    let mut lo = Vec3::splat(f32::MAX);
    let mut hi = Vec3::splat(f32::MIN);
    for c in &parts {
        let b = c.shape().compute_aabb(&at(c));
        lo = lo.min(unv(b.mins));
        hi = hi.max(unv(b.maxs));
    }
    if parts.is_empty() || lo.x > hi.x {
        return None;
    }
    let n = samples.clamp(1, 12);
    let nz = if DIM == 3 { n } else { 1 };
    let cell = (hi - lo) / Vec3::new(n as f32, n as f32, nz as f32);
    let mut points = Vec::new();
    for k in 0..nz {
        for j in 0..n {
            for i in 0..n {
                let p = lo + cell * Vec3::new(i as f32 + 0.5, j as f32 + 0.5, k as f32 + 0.5);
                if parts.iter().any(|c| c.shape().contains_point(&at(c), v(p))) {
                    points.push((p, [i, j, k]));
                }
            }
        }
    }
    if points.is_empty() {
        let mid = n / 2;
        let p = lo + cell * Vec3::new(mid as f32 + 0.5, mid as f32 + 0.5, (nz / 2) as f32 + 0.5);
        points.push((p, [mid, mid, nz / 2]));
    }
    let volume: f32 = parts
        .iter()
        .map(|c| c.shape().mass_properties(1.0).mass())
        .sum();
    let volume = if volume > 0.0 {
        volume
    } else {
        cell.x * cell.y * cell.z.max(if DIM == 3 { 0.0 } else { 1.0 }) * points.len() as f32
    };
    Some(Hull {
        each: volume / points.len() as f32,
        points,
        lo,
        hi,
        cell,
    })
}

/// Where a box centred at `centre`, reaching `extent` along x and z, lies
/// in the og.Water `w` (its centre in the water's frame); `None` off it.
fn over(
    w: &crate::WaterBody,
    centre: Vec3,
    extent: og_core::glam::Vec2,
) -> Option<og_core::glam::Vec2> {
    let local = og_core::glam::Vec2::new(centre.x - w.position.x, centre.z - w.position.z);
    let over = match &w.river {
        Some(river) => river
            .at(og_core::glam::Vec2::new(centre.x, centre.z))
            .is_some(),
        None if w.mesh.is_some() => w.surface(centre.x.into(), centre.z.into(), 0.0).is_some(),
        None if w.endless => true,
        None => {
            local.x.abs() <= w.size.x * 0.5 + extent.x && local.y.abs() <= w.size.y * 0.5 + extent.y
        }
    };
    over.then_some(local)
}

/// The world box of a body's solid colliders; `None` with none.
pub(super) fn solid_box(world: &rp::PhysicsWorld, rb: &rp::RigidBody) -> Option<(Vec3, Vec3)> {
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);
    for &c in rb.colliders() {
        let c = &world.colliders[c];
        if c.is_sensor() {
            continue;
        }
        let aabb = c.compute_aabb();
        min = min.min(unv(aabb.mins));
        max = max.max(unv(aabb.maxs));
    }
    (min.x <= max.x).then_some((min, max))
}

/// How much of a box from `min` to `max` (world y) lies under `level`, 0..1.
pub(super) fn box_under(min: f32, max: f32, level: f32) -> f32 {
    if max - min > 1e-6 {
        ((level - min) / (max - min)).clamp(0.0, 1.0)
    } else if min < level {
        1.0
    } else {
        0.0
    }
}

/// A limp og.Ragdoll's bones in the water for a tick (og.Buoyancy on its
/// entity): each bone is lifted at its centre of mass by the water it
/// displaces, as much of it as its box is under the level there, and
/// slowed against the moving water; how much of the ragdoll's mass is
/// under. A bone displaces its mass, as a body of the colliders' default
/// density 1 does (its capsule is a stand-in, far thinner than the limb):
/// `waterDensity` 2 floats a ragdoll half under, as it does a crate. Its
/// box cuts the og.Water it straddles. `None` for no ragdoll, or one that
/// follows its animation.
#[allow(clippy::too_many_arguments)]
fn bones(
    scene: &Scene,
    g: f32,
    sub_dt: f32,
    waters: &[crate::WaterBody],
    sim: &mut Sim,
    entity: u64,
    b: &Buoyancy,
    cuts: &mut BTreeMap<u64, Vec<crate::water::WaterCut>>,
    record_out: bool,
    forced_bodies: &mut Vec<rp::RigidBodyHandle>,
) -> Option<f32> {
    let bones = sim
        .ragdolls
        .get(&entity)
        .filter(|s| s.dynamic)?
        .bones
        .clone();
    let mut lo = Vec3::splat(f32::MAX);
    let mut hi = Vec3::splat(f32::MIN);
    let (mut total, mut under, mut fall) = (0.0, 0.0, 0.0);
    let inv_dt = if sub_dt > 0.0 { 1.0 / sub_dt } else { 0.0 };
    for h in bones {
        let rb = &sim.world.bodies[h];
        let Some((min, max)) = solid_box(&sim.world, rb) else {
            continue;
        };
        lo = lo.min(min);
        hi = hi.max(max);
        let mass = rb.mass();
        total += mass;
        let linvel = unv(rb.linvel());
        fall += linvel.y * mass;
        let centre = unv(rb.center_of_mass());
        let Some(water) =
            crate::water::level(&scene.surfaces, waters, centre.x, centre.z, scene.seconds)
        else {
            continue;
        };
        let fraction = box_under(min.y, max.y, water.height);
        if fraction <= 0.0 {
            continue;
        }
        under += fraction * mass;
        let held = Vec3::Y * g + water.acceleration;
        let held = Vec3::new(held.x, held.y.max(0.0), held.z);
        let lift_force = held * (b.water_density * mass * fraction);
        let speed = linvel - water.velocity;
        let drag_factor = 1.0 - og_core::math::exp(-(b.drag.max(0.0) * sub_dt));
        let drag_force = speed * (drag_factor * inv_dt * mass * fraction);
        let rb = &mut sim.world.bodies[h];
        rb.add_force(v(flat(lift_force - drag_force)), true);
        forced_bodies.push(h);
        if b.angular_drag > 0.0 {
            let keep = og_core::math::exp(-(b.angular_drag * fraction * sub_dt));
            let spin = unangvel(rb.angvel());
            rb.set_angvel(angvel(spin * keep), true);
        }
    }
    if total <= 0.0 {
        return Some(0.0);
    }
    if record_out {
        let centre = (lo + hi) * 0.5;
        let level = crate::water::level(&scene.surfaces, waters, centre.x, centre.z, scene.seconds)
            .map_or(lo.y, |l| l.height);
        if lo.y < level && hi.y > level {
            let extent = og_core::glam::Vec2::new(hi.x - lo.x, hi.z - lo.z) * 0.5;
            for w in waters {
                let Some(list) = cuts.get_mut(&w.entity) else {
                    continue;
                };
                let Some(local) = over(w, centre, extent) else {
                    continue;
                };
                list.push(crate::water::WaterCut::boxed(
                    entity,
                    local - extent,
                    local + extent,
                    level - lo.y,
                    fall / total,
                ));
            }
        }
    }
    Some(under / total)
}

/// The footprint of a floating body's hull (its box `hull.lo` to `hull.hi`,
/// turned and placed by `at`) in each og.Water of `cuts` it lies over:
/// the rectangle its own x and z axes span over the water's xz, its box,
/// and how far it reaches under the water at its centre.
fn hull_cuts(
    entity: u64,
    hull: &Hull,
    at: &rp::Pose,
    speed: f32,
    scene: &Scene,
    waters: &[crate::WaterBody],
    cuts: &mut BTreeMap<u64, Vec<crate::water::WaterCut>>,
) {
    let size = hull.hi - hull.lo;
    let centre = unv(at.transform_point(v(hull.lo + size * 0.5)));
    let turn = |a: Vec3| unv(at.rotation * v(a));
    let (ax, ay, az) = (turn(Vec3::X), turn(Vec3::Y), turn(Vec3::Z));
    let flat = |a: Vec3| og_core::glam::Vec2::new(a.x, a.z);
    let axis = flat(ax).try_normalize().unwrap_or(og_core::glam::Vec2::X);
    let side = og_core::glam::Vec2::new(-axis.y, axis.x);
    // The box's half extents along `axis` and across it.
    let half = size * 0.5;
    let reach = |d: og_core::glam::Vec2| {
        (flat(ax).dot(d) * half.x).abs()
            + (flat(ay).dot(d) * half.y).abs()
            + (flat(az).dot(d) * half.z).abs()
    };
    let foot = og_core::glam::Vec2::new(reach(axis), reach(side));
    let extent =
        og_core::glam::Vec2::new(reach(og_core::glam::Vec2::X), reach(og_core::glam::Vec2::Y));
    let low = centre.y - (ax.y.abs() * half.x + ay.y.abs() * half.y + az.y.abs() * half.z);
    let level = crate::water::level(&scene.surfaces, waters, centre.x, centre.z, scene.seconds)
        .map_or(low, |l| l.height);
    let depth = level - low;
    if depth <= 0.0 {
        return;
    }
    for w in waters {
        let Some(list) = cuts.get_mut(&w.entity) else {
            continue;
        };
        let Some(local) = over(w, centre, extent) else {
            continue;
        };
        list.push(crate::water::WaterCut {
            entity,
            min: local - extent,
            max: local + extent,
            centre: local,
            half: foot,
            axis,
            depth,
            speed,
        });
    }
}

/// Applies og.Buoyancy forces and torques inside each substep using a single
/// velocity snapshot per body so hull cells do not mutate velocity mid-loop.
/// When `record_out` is true (the final substep), writes `out.floats` and
/// appends hull footprints to `cuts`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn buoyancy_substep(
    scene: &Scene,
    settings: &Physics,
    sub_dt: f32,
    waters: &[crate::WaterBody],
    sim: &mut Sim,
    handles: &super::super::sim::Handles<'_>,
    out: &mut StepOut,
    cuts: &mut BTreeMap<u64, Vec<crate::water::WaterCut>>,
    record_out: bool,
    forced_bodies: &mut Vec<rp::RigidBodyHandle>,
) {
    if DIM != 3 || scene.floats.is_empty() {
        return;
    }
    let g = -settings.gravity.y;
    for (e, b) in &scene.floats {
        let key = e.to_bits();
        let hull = handles.get(&key).and_then(|&h| {
            float(
                scene,
                g,
                sub_dt,
                waters,
                sim,
                h,
                key,
                b,
                cuts,
                record_out,
                forced_bodies,
            )
        });
        let bones = bones(
            scene,
            g,
            sub_dt,
            waters,
            sim,
            key,
            b,
            cuts,
            record_out,
            forced_bodies,
        );
        if record_out && let Some(submerged) = bones.or(hull) {
            out.floats.push((*e, Buoyancy { submerged, ..*b }));
        }
    }
}

/// Records the tick's water surface cuts for the renderer's wakes/ripples
/// once per tick after the substep loop.
pub(crate) fn record_water_wakes(
    sim: &mut Sim,
    dt: f32,
    tick: u64,
    cuts: BTreeMap<u64, Vec<crate::water::WaterCut>>,
) {
    for (key, list) in cuts {
        sim.wakes
            .entry(key)
            .or_default()
            .record_water(dt, tick, list);
    }
}

/// One floating body's hull in the water for one substep: computes net lift,
/// drag force, and torque from a single pre-substep velocity snapshot and
/// applies them through `add_force` / `add_torque`.
#[allow(clippy::too_many_arguments)]
fn float(
    scene: &Scene,
    g: f32,
    sub_dt: f32,
    waters: &[crate::WaterBody],
    sim: &mut Sim,
    h: rp::RigidBodyHandle,
    entity: u64,
    b: &Buoyancy,
    cuts: &mut BTreeMap<u64, Vec<crate::water::WaterCut>>,
    record_out: bool,
    forced_bodies: &mut Vec<rp::RigidBodyHandle>,
) -> Option<f32> {
    let rb = &sim.world.bodies[h];
    if !rb.is_dynamic() {
        return None;
    }
    let hull = hull(&sim.world.colliders, rb, b.samples)?;
    let at = *rb.position();
    let com = unv(rb.center_of_mass());
    let linvel = unv(rb.linvel());
    let angvel_snap = unangvel(rb.angvel());
    let mass = rb.mass();
    // A cell's half height in the world, its box turned with the body.
    let up = [Vec3::X, Vec3::Y, Vec3::Z].map(|a| unv(at.rotation * v(a)).y.abs());
    let half = 0.5 * (up[0] * hull.cell.x + up[1] * hull.cell.y + up[2] * hull.cell.z);
    let column = if up[1] >= up[0] && up[1] >= up[2] {
        1
    } else if up[0] >= up[2] {
        0
    } else {
        2
    };
    let middle = (hull.lo + hull.cell * Vec3::splat(b.samples.clamp(1, 12) as f32) * 0.5)[column];
    let mut levels: BTreeMap<[u32; 2], Option<og_env::ocean::SurfacePoint>> = BTreeMap::new();
    let share = 1.0 / hull.points.len() as f32;
    let inv_dt = if sub_dt > 0.0 { 1.0 / sub_dt } else { 0.0 };
    let drag_rate = (1.0 - og_core::math::exp(-(b.drag.max(0.0) * sub_dt))) * inv_dt;
    let mut under = 0.0;
    let mut net_force = Vec3::ZERO;
    let mut net_torque = Vec3::ZERO;
    for (p, index) in &hull.points {
        let w = unv(at.transform_point(v(*p)));
        let key = match column {
            0 => [index[1], index[2]],
            1 => [index[0], index[2]],
            _ => [index[0], index[1]],
        };
        let level = *levels.entry(key).or_insert_with(|| {
            let mut q = *p;
            q[column] = middle;
            let c = unv(at.transform_point(v(q)));
            crate::water::level(&scene.surfaces, waters, c.x, c.z, scene.seconds)
        });
        let Some(water) = level else {
            continue;
        };
        let level = water.height;
        let fraction = if half > 1e-6 {
            ((level - (w.y - half)) / (2.0 * half)).clamp(0.0, 1.0)
        } else if w.y < level {
            1.0
        } else {
            0.0
        };
        if fraction <= 0.0 {
            continue;
        }
        under += fraction * share;
        let held = Vec3::Y * g + water.acceleration;
        let held = Vec3::new(held.x, held.y.max(0.0), held.z);
        let lift_force = held * (b.water_density * hull.each * fraction);
        let point_vel = linvel + angvel_snap.cross(w - com);
        let rel_vel = point_vel - water.velocity;
        let drag_force = rel_vel * (drag_rate * mass * share * fraction);
        let cell_force = flat(lift_force - drag_force);
        net_force += cell_force;
        net_torque += (w - com).cross(cell_force);
    }
    if under > 0.0 {
        let rb = &mut sim.world.bodies[h];
        if net_force != Vec3::ZERO {
            rb.add_force(v(net_force), true);
            forced_bodies.push(h);
        }
        if net_torque != Vec3::ZERO {
            rb.add_torque(angvel(net_torque), true);
            forced_bodies.push(h);
        }
        if b.angular_drag > 0.0 {
            let keep = og_core::math::exp(-(b.angular_drag * under * sub_dt));
            let spin = unangvel(rb.angvel());
            rb.set_angvel(angvel(spin * keep), true);
        }
    }
    if record_out {
        hull_cuts(entity, &hull, &at, linvel.y, scene, waters, cuts);
    }
    Some(under)
}

/// The water an og.CharacterController with og.Buoyancy is in (3D): its
/// og.Buoyancy, how much of its colliders' box is under the level at its
/// centre, and the water there. `None` without og.Buoyancy or over no
/// water.
pub(super) fn swims(
    scene: &Scene,
    waters: &[crate::WaterBody],
    sim: &Sim,
    h: rp::RigidBodyHandle,
    entity: u64,
) -> Option<(Buoyancy, f32, og_env::ocean::SurfacePoint)> {
    if DIM != 3 {
        return None;
    }
    let i = scene
        .floats
        .binary_search_by_key(&entity, |f| f.0.to_bits())
        .ok()?;
    let b = scene.floats[i].1;
    let (min, max) = solid_box(&sim.world, &sim.world.bodies[h])?;
    let centre = (min + max) * 0.5;
    let water = crate::water::level(&scene.surfaces, waters, centre.x, centre.z, scene.seconds);
    let under = water.map_or(0.0, |w| box_under(min.y, max.y, w.height));
    Some((b, under, water.unwrap_or_default()))
}
