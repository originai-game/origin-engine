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
    dt: f32,
    waters: &[crate::WaterBody],
    sim: &mut Sim,
    entity: u64,
    b: &Buoyancy,
    cuts: &mut BTreeMap<u64, Vec<crate::water::WaterCut>>,
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
    for h in bones {
        let rb = &sim.world.bodies[h];
        let Some((min, max)) = solid_box(&sim.world, rb) else {
            continue;
        };
        lo = lo.min(min);
        hi = hi.max(max);
        let mass = rb.mass();
        total += mass;
        fall += unv(rb.linvel()).y * mass;
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
        // As a hull's cell: ρV(g + a) up, and its speed through the water lost.
        let held = Vec3::Y * g + water.acceleration;
        let held = Vec3::new(held.x, held.y.max(0.0), held.z);
        let lift = held * (b.water_density * mass * fraction * dt);
        let rb = &mut sim.world.bodies[h];
        let speed = unv(rb.linvel()) - water.velocity;
        let slow = (b.drag * dt).min(1.0) * mass * fraction;
        rb.apply_impulse(v(flat(lift - speed * slow)), true);
        let keep = 1.0 - (b.angular_drag * fraction * dt).min(1.0);
        let spin = unangvel(rb.angvel());
        rb.set_angvel(angvel(spin * keep), true);
    }
    if total <= 0.0 {
        return Some(0.0);
    }
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
            depth: level - low,
            speed,
        });
    }
}

/// og.Buoyancy before the step (3D): each floating body's hull cells under
/// the water at their points push it up by the water they displace, at
/// those points (a torque that rights and pitches it), and slow it as far
/// as they are under; its `submerged` is written. Cells and bodies in a
/// fixed order, og-core's deterministic math: the same on every run. Each
/// og.Water then records the bodies cutting it for the renderer (`cuts`,
/// the tick's from [`water`], the hulls' footprints added here;
/// [`crate::water::Wakes::record_water`]; never read by the tick).
#[allow(clippy::too_many_arguments)]
pub(crate) fn buoyancy(
    scene: &Scene,
    settings: &Physics,
    dt: f32,
    tick: u64,
    waters: &[crate::WaterBody],
    sim: &mut Sim,
    handles: &super::super::sim::Handles<'_>,
    out: &mut StepOut,
    mut cuts: BTreeMap<u64, Vec<crate::water::WaterCut>>,
) {
    if DIM == 3 {
        floats(scene, settings, dt, waters, sim, handles, out, &mut cuts);
    }
    for (key, list) in cuts {
        sim.wakes
            .entry(key)
            .or_default()
            .record_water(dt, tick, list);
    }
}

/// [`buoyancy`]'s floating bodies, each hull's footprint added to the
/// `cuts` of the waters it lies in.
#[allow(clippy::too_many_arguments)]
fn floats(
    scene: &Scene,
    settings: &Physics,
    dt: f32,
    waters: &[crate::WaterBody],
    sim: &mut Sim,
    handles: &super::super::sim::Handles<'_>,
    out: &mut StepOut,
    cuts: &mut BTreeMap<u64, Vec<crate::water::WaterCut>>,
) {
    let g = -settings.gravity.y;
    for (e, b) in &scene.floats {
        let key = e.to_bits();
        let hull = handles
            .get(&key)
            .and_then(|&h| float(scene, g, dt, waters, sim, h, key, b, cuts));
        // Limp, its bones are what is in the water.
        let bones = bones(scene, g, dt, waters, sim, key, b, cuts);
        if let Some(submerged) = bones.or(hull) {
            out.floats.push((*e, Buoyancy { submerged, ..*b }));
        }
    }
}

/// One floating body's hull in the water for a tick: its lift, drag and
/// cut; how much of it is under. `None` for a body that is not dynamic or
/// has no solid collider.
#[allow(clippy::too_many_arguments)]
fn float(
    scene: &Scene,
    g: f32,
    dt: f32,
    waters: &[crate::WaterBody],
    sim: &mut Sim,
    h: rp::RigidBodyHandle,
    entity: u64,
    b: &Buoyancy,
    cuts: &mut BTreeMap<u64, Vec<crate::water::WaterCut>>,
) -> Option<f32> {
    {
        let rb = &sim.world.bodies[h];
        if !rb.is_dynamic() {
            return None;
        }
        let hull = hull(&sim.world.colliders, rb, b.samples)?;
        let at = *rb.position();
        // A cell's half height in the world, its box turned with the body.
        let up = [Vec3::X, Vec3::Y, Vec3::Z].map(|a| unv(at.rotation * v(a)).y.abs());
        let half = 0.5 * (up[0] * hull.cell.x + up[1] * hull.cell.y + up[2] * hull.cell.z);
        // The water level is read once per column of cells along the body's
        // axis nearest the world's up, at the column's middle: a sea's
        // height costs some 300 sines a point.
        let column = if up[1] >= up[0] && up[1] >= up[2] {
            1
        } else if up[0] >= up[2] {
            0
        } else {
            2
        };
        let middle =
            (hull.lo + hull.cell * Vec3::splat(b.samples.clamp(1, 12) as f32) * 0.5)[column];
        // Each column's level and the water's velocity there (its
        // particles' orbit): a hull is slowed against the moving water, not
        // still water, so it is carried round with it as a float is and
        // rides through a crest. (Against the level's rise at a fixed point,
        // the wave's phase speed times its slope, steep crests flung hulls
        // clear of the water.)
        let mut levels: BTreeMap<[u32; 2], Option<og_core::ocean::SurfacePoint>> = BTreeMap::new();
        let mass = rb.mass();
        let share = 1.0 / hull.points.len() as f32;
        let mut under = 0.0;
        let mut pushes = Vec::with_capacity(hull.points.len());
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
            pushes.push((w, fraction, water));
        }
        let rb = &mut sim.world.bodies[h];
        for (w, fraction, water) in pushes {
            // Archimedes in moving water (Froude–Krylov): the pressure that
            // buoys a cell pushes it as hard as it accelerates the water it
            // displaces against gravity, ρV(g + a), so where a crest falls
            // away the lift falls with it and a floater sinks with the water
            // at its draft instead of being flung clear.
            let held = Vec3::Y * g + water.acceleration;
            let held = Vec3::new(held.x, held.y.max(0.0), held.z);
            let lift = held * (b.water_density * hull.each * fraction * dt);
            let flow = water.velocity;
            // The drag: its share of the mass losing `drag` of its speed
            // through the water a second, never more than all of it.
            let speed = unv(rb.velocity_at_point(v(w))) - flow;
            let slow = (b.drag * dt).min(1.0) * mass * share * fraction;
            rb.apply_impulse_at_point(v(flat(lift - speed * slow)), v(w), true);
        }
        if under > 0.0 {
            let keep = 1.0 - (b.angular_drag * under * dt).min(1.0);
            let spin = unangvel(rb.angvel());
            rb.set_angvel(angvel(spin * keep), true);
        }
        hull_cuts(entity, &hull, &at, unv(rb.linvel()).y, scene, waters, cuts);
        Some(under)
    }
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
) -> Option<(Buoyancy, f32, og_core::ocean::SurfacePoint)> {
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
