//! og.Buoyancy: a body's hull floated on the water it lies in (an og.Water's sea, a surface, a line).

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

/// The water surface over world point (`x`, `z`): its height (world y)
/// and the water's velocity and acceleration there; the highest of the og.WaterSurface squares
/// and og.Water bodies over it (one entity with both: its rest level and
/// its sea together); `None` over no water. An og.WaterSurface lifts at its
/// rest level, as its own buoyancy does: its waves are the bodies'
/// splashes, and a body riding its own wake would pump them.
fn water_level(
    scene: &Scene,
    waters: &[crate::WaterBody],
    x: f32,
    z: f32,
    seconds: f64,
) -> Option<og_core::ocean::SurfacePoint> {
    use og_core::ocean::SurfacePoint;
    let mut level: Option<SurfacePoint> = None;
    let mut take = |y: f32, p: SurfacePoint| {
        if level.is_none_or(|l| y > l.height) {
            level = Some(SurfacePoint { height: y, ..p });
        }
    };
    let (x64, z64) = (f64::from(x), f64::from(z));
    for (e, position, surface) in &scene.surfaces {
        let half = surface.size * 0.5;
        if (x - position.x).abs() > half || (z - position.z).abs() > half {
            continue;
        }
        let key = e.to_bits();
        let sea = waters
            .iter()
            .find(|w| w.entity == key)
            .and_then(|w| w.surface(x64, z64, seconds))
            .unwrap_or_default();
        take(position.y + sea.height, sea);
    }
    for w in waters {
        if scene.surfaces.iter().any(|s| s.0.to_bits() == w.entity) {
            continue;
        }
        if let Some(p) = w.surface(x64, z64, seconds) {
            take(w.position.y + p.height, p);
        }
    }
    level
}

/// The footprint of a floating body's hull (its box `hull.lo` to `hull.hi`,
/// turned and placed by `at`) in each sea of `waters` it lies over:
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
    let level =
        water_level(scene, waters, centre.x, centre.z, scene.seconds).map_or(low, |l| l.height);
    for w in waters.iter().filter(|w| w.sea.is_some()) {
        let Some(list) = cuts.get_mut(&w.entity) else {
            continue;
        };
        let local = og_core::glam::Vec2::new(centre.x - w.position.x, centre.z - w.position.z);
        if local.x.abs() > w.size.x * 0.5 + extent.x || local.y.abs() > w.size.y * 0.5 + extent.y {
            continue;
        }
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
/// sea (an og.Water with a wind) records the hulls in it for the renderer
/// ([`crate::water::Wakes::record_sea`]; never read by the tick).
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
) {
    let mut cuts: BTreeMap<u64, Vec<crate::water::WaterCut>> = waters
        .iter()
        .filter(|w| w.sea.is_some())
        .map(|w| (w.entity, Vec::new()))
        .collect();
    if DIM == 3 {
        floats(scene, settings, dt, waters, sim, handles, out, &mut cuts);
    }
    for (key, list) in cuts {
        sim.wakes.entry(key).or_default().record_sea(dt, tick, list);
    }
}

/// [`buoyancy`]'s floating bodies, each hull's footprint added to the
/// `cuts` of the seas it lies in.
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
        let Some(&h) = handles.get(&e.to_bits()) else {
            continue;
        };
        let rb = &sim.world.bodies[h];
        if !rb.is_dynamic() {
            continue;
        }
        let Some(hull) = hull(&sim.world.colliders, rb, b.samples) else {
            continue;
        };
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
                water_level(scene, waters, c.x, c.z, scene.seconds)
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
        hull_cuts(
            e.to_bits(),
            &hull,
            &at,
            unv(rb.linvel()).y,
            scene,
            waters,
            cuts,
        );
        out.floats.push((
            *e,
            Buoyancy {
                submerged: under,
                ..*b
            },
        ));
    }
}
