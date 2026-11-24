//! og.Buoyancy: what floats on the water it lies in (an og.Water's sea, a surface): a body's hull, a limp og.Ragdoll's bones, an og.CharacterController.

use super::*;

/// A body's hull for og.Buoyancy, in its own frame: the centres of the
/// cells of its colliders' box (`samples` a side, `lo` its low corner)
/// that lie in a solid collider, with their cell indices, each standing
/// for an equal share (`each`) of the colliders' volume; `cell` is a
/// cell's size.
pub(crate) struct Hull {
    parts: Vec<(rp::SharedShape, rp::Pose)>,
    samples: u32,
    points: Vec<Vec3>,
    lo: Vec3,
    hi: Vec3,
    cell: Vec3,
    each: f32,
    triangles: Vec<[Vec3; 3]>,
}

pub(crate) type HullCache = BTreeMap<(u32, u32), Hull>;

/// Geometry identity includes local collider poses, not the body's motion
/// or material. Holding the Arc also prevents allocation-address reuse.
pub(super) fn cached_hull<'a>(
    cache: &'a mut HullCache,
    colliders: &rp::ColliderSet,
    rb: &rp::RigidBody,
    handle: rp::RigidBodyHandle,
    samples: u32,
) -> Option<&'a Hull> {
    let handle = handle.into_raw_parts();
    let same = cache.get(&handle).is_some_and(|h| {
        h.samples == samples
            && h.parts
                .iter()
                .zip(
                    rb.colliders()
                        .iter()
                        .map(|&c| &colliders[c])
                        .filter(|c| !c.is_sensor()),
                )
                .all(|((shape, at), c)| {
                    std::sync::Arc::ptr_eq(&shape.0, &c.shared_shape().0)
                        && *at == *c.position_wrt_parent().unwrap_or(&rp::Pose::IDENTITY)
                })
            && h.parts.len()
                == rb
                    .colliders()
                    .iter()
                    .filter(|&&c| !colliders[c].is_sensor())
                    .count()
    });
    if !same {
        cache.remove(&handle);
        cache.insert(handle, hull(colliders, rb, samples)?);
    }
    cache.get(&handle)
}

impl Hull {
    pub(super) fn world_triangles(&self, at: &rp::Pose, centre: Vec3) -> Vec<[Vec3; 3]> {
        let offset = unv(at.translation) - centre;
        self.triangles
            .iter()
            .map(|t| t.map(|p| unv(at.rotation * v(p)) + offset))
            .collect()
    }

    pub(super) fn volume(&self) -> f32 {
        self.each * self.points.len() as f32
    }
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
                    points.push(p);
                }
            }
        }
    }
    if points.is_empty() {
        let mid = n / 2;
        let p = lo + cell * Vec3::new(mid as f32 + 0.5, mid as f32 + 0.5, (nz / 2) as f32 + 0.5);
        points.push(p);
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
    // A partially triangulated compound would undercount its volume.
    let triangles = parts
        .iter()
        .map(|c| {
            Some(
                super::super::water_geometry::triangles(c.shared_shape())?
                    .into_iter()
                    .map(|t| t.map(|p| unv(at(c).transform_point(v(p)))))
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Option<Vec<_>>>()
        .unwrap_or_default()
        .into_iter()
        .flatten()
        .collect();
    Some(Hull {
        triangles,
        parts: parts
            .iter()
            .map(|c| (c.shared_shape().clone(), at(c)))
            .collect(),
        samples,
        each: volume / points.len() as f32,
        points,
        lo,
        hi,
        cell,
    })
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

/// A limp og.Ragdoll's bones in the water for a tick (og.Buoyancy on its
/// entity): each bone uses its own cached collider-volume samples and
/// local water level, including its own shoreline cut. Returns the wet
/// mass fraction, not a fictitious volume derived from the bone's mass.
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
    let (mut total, mut under) = (0.0, 0.0);
    for h in bones {
        let mass = sim.world.bodies[h].mass();
        total += mass;
        under += mass
            * float(
                scene,
                g,
                sub_dt,
                waters,
                sim,
                h,
                entity,
                b,
                cuts,
                record_out,
                forced_bodies,
            )
            .unwrap_or(0.0);
    }
    if total <= 0.0 {
        return Some(0.0);
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
    for w in waters {
        let Some(list) = cuts.get_mut(&w.entity) else {
            continue;
        };
        let Some(surface) = w.surface(centre.x.into(), centre.z.into(), scene.seconds) else {
            continue;
        };
        let level = w.position.y + surface.height;
        let depth = level - low;
        if w.depth <= 0.0 || depth <= 0.0 || level >= 2.0 * centre.y - low {
            continue;
        }
        let local = og_core::glam::Vec2::new(centre.x - w.position.x, centre.z - w.position.z);
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
    sim.ext.water_hulls.retain(|&(i, g), _| {
        sim.world
            .bodies
            .contains(rp::RigidBodyHandle::from_raw_parts(i, g))
    });
    super::surfaces::surface_buoyancy_substep(scene, settings, sub_dt, sim, handles, forced_bodies);
    if scene.floats.is_empty() {
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
    let hull = cached_hull(
        &mut sim.ext.water_hulls,
        &sim.world.colliders,
        rb,
        h,
        b.samples,
    )?;
    let at = *rb.position();
    let com = unv(rb.center_of_mass());
    let linvel = unv(rb.linvel());
    let angvel_snap = unangvel(rb.angvel());
    let mass = rb.mass();
    // A cell's half height in the world, its box turned with the body.
    let up = [Vec3::X, Vec3::Y, Vec3::Z].map(|a| unv(at.rotation * v(a)).y.abs());
    let half = 0.5 * (up[0] * hull.cell.x + up[1] * hull.cell.y + up[2] * hull.cell.z);
    let share = 1.0 / hull.points.len() as f32;
    let inv_dt = if sub_dt > 0.0 { 1.0 / sub_dt } else { 0.0 };
    let drag_rate = (1.0 - og_core::math::exp(-(b.drag.max(0.0) * sub_dt))) * inv_dt;
    let mut under = 0.0;
    let mut net_force = Vec3::ZERO;
    let mut net_torque = Vec3::ZERO;
    for p in &hull.points {
        let w = unv(at.transform_point(v(*p)));
        let Some(volume) = crate::water::level(
            &scene.surfaces,
            &scene.lines,
            waters,
            w,
            half,
            scene.seconds,
        ) else {
            continue;
        };
        let water = volume.surface;
        let level = water.height;
        let fraction = if half > 1e-6 {
            ((level.min(w.y + half) - volume.bottom.max(w.y - half)) / (2.0 * half)).clamp(0.0, 1.0)
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
        // Distributed drag acts on translation only. Applying explicit
        // point-velocity drag can amplify spin when m*r²/I is large;
        // rotational drag is integrated exponentially once below.
        let rel_vel = linvel - water.velocity;
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
        if b.angular_drag > 0.0 || b.drag > 0.0 {
            let keep =
                og_core::math::exp(-((b.angular_drag.max(0.0) + b.drag.max(0.0)) * under * sub_dt));
            rb.set_angvel(angvel(angvel_snap * keep), true);
        }
    }
    if record_out {
        hull_cuts(entity, hull, &at, linvel.y, scene, waters, cuts);
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
    let water = crate::water::level(
        &scene.surfaces,
        &scene.lines,
        waters,
        centre,
        (max.y - min.y) * 0.5,
        scene.seconds,
    );
    let under = water.as_ref().map_or(0.0, |w| {
        ((w.surface.height.min(max.y) - w.bottom.max(min.y)) / (max.y - min.y).max(1e-6))
            .clamp(0.0, 1.0)
    });
    Some((b, under, water.map(|w| w.surface).unwrap_or_default()))
}
