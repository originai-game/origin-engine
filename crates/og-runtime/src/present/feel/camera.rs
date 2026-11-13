//! The og.camera system: shake, punch and kick, follow, zones and rails, and its state block.

use super::*;

pub(super) fn camera_op(world: u32, op: CameraOp) -> Result<(), String> {
    engine(world).and_then(|eng| {
        let feel = feel_of(&eng)?;
        lock(&feel.shared).camera_ops.push(op);
        Ok(())
    })
}

pub fn og_camera_shake(world: u32, trauma: f64) -> Result<(), String> {
    if !(trauma.is_finite() && (0.0..=1.0).contains(&trauma)) {
        return Err(format!("trauma {trauma} is outside 0..1"));
    }
    camera_op(world, CameraOp::Shake(trauma as f32))
}

pub fn og_camera_punch(world: u32, amount: f64) -> Result<(), String> {
    if !(amount.is_finite() && (-0.9..=4.0).contains(&amount)) {
        return Err(format!("punch {amount} is outside -0.9..4"));
    }
    camera_op(world, CameraOp::Punch(amount as f32))
}

pub fn og_camera_kick(world: u32, direction: String, distance: f64) -> Result<(), String> {
    let d: Vec3 = crate::host::wire::arg(Some(&direction), "direction")?;
    if !distance.is_finite() || !d.is_finite() {
        return Err("kick needs a finite direction and distance".into());
    }
    camera_op(
        world,
        CameraOp::Kick(d.normalize_or_zero() * distance as f32),
    )
}

/// Applies the queued ops, decays trauma, punch and kick, moves the
/// camera along a rail, then lets the brain blend it to the live virtual
/// camera (vcam).
pub(super) fn camera_system(
    ctx: &mut SystemContext<'_>,
    engine_id: u32,
    shared: &Mutex<FeelShared>,
    r: &Records,
    physics: &og_physics::PhysicsSystem,
) -> og_core::Result<()> {
    let (cid, layout) = &r.camera;
    let ops = std::mem::take(&mut lock(shared).camera_ops);
    let dt = ctx.time().dt;
    let bytes = ctx.resource_bytes(*cid)?;
    let mut cam = layout.decode(&bytes);
    // The camera as the tick found it: what follows writes only trauma,
    // punch, kick and the zone fields before reading the rest.
    let row = record::Camera::from_json(&cam).map_err(invalid)?;
    let mut trauma = row.trauma;
    let mut punch = row.punch;
    let mut kick = row.kick;
    for op in &ops {
        match *op {
            CameraOp::Shake(t) => trauma = (trauma + t).min(1.0),
            CameraOp::Punch(p) => punch = (punch + p).clamp(-0.9, 4.0),
            CameraOp::Kick(k) => kick += k,
        }
    }
    let decay = |v: f32, rate: f32| v * og_core::math::exp(-rate.max(0.0) * dt);
    // The decay starts the tick after the op, so a shake shows at full strength once.
    if ops.is_empty() {
        trauma = (trauma - row.trauma_decay.max(0.0) * dt).max(0.0);
        punch = decay(punch, row.punch_decay);
        kick *= og_core::math::exp(-row.kick_decay.max(0.0) * dt);
    }
    // Below a thousandth they are invisible: snap to rest.
    if punch.abs() < 1e-3 {
        punch = 0.0;
    }
    if kick.length_squared() < 1e-6 {
        kick = Vec3::ZERO;
    }
    cam["trauma"] = json!(trauma);
    cam["punch"] = json!(punch);
    cam["kick"] = json!({ "x": kick.x, "y": kick.y, "z": kick.z });
    camera_zone(ctx, r, &mut cam, &row, dt)?;
    // A rail: the camera eases to the rail point nearest the entity.
    let mut rail: Option<(record::CameraRail, Vec3)> = None;
    for a in ctx.query(0)? {
        if let Some(&e) = a.entities().first() {
            let v: record::CameraRail = r
                .rail
                .1
                .read(&ctx.get_bytes(e, r.rail.0)?)
                .map_err(invalid)?;
            let t: record::Transform = r
                .transform
                .1
                .read(&ctx.get_bytes(e, r.transform.0)?)
                .map_err(invalid)?;
            rail = Some((v, t.position));
            break;
        }
    }
    if let Some((rail, target)) = rail {
        // A path handle (og.path) replaces the inline points.
        let path = rail.path;
        let goal = match crate::bind::timeline::path_of(engine_id, path).filter(|_| path != 0) {
            Some(p) => p.nearest(target).position,
            None => {
                let n = (rail.points as usize).clamp(2, 8);
                let all = [
                    rail.p0, rail.p1, rail.p2, rail.p3, rail.p4, rail.p5, rail.p6, rail.p7,
                ];
                rail_nearest(&all[..n], target)
            }
        };
        let pos = row.position;
        let smooth = rail.smooth_time;
        let k = if smooth > 0.0 {
            (dt / smooth).min(1.0)
        } else {
            1.0
        };
        let ortho = row.orthographic;
        let mut next = pos + (goal - pos) * k;
        if ortho {
            next.z = pos.z;
        }
        cam["position"] = json!({ "x": next.x, "y": next.y, "z": next.z });
        if !ortho {
            let q = look_rotation(next, target);
            cam["quaternion"] = json!({ "x": q.x, "y": q.y, "z": q.z, "w": q.w });
        }
    }
    vcam::brain(ctx, engine_id, r, physics, &mut cam, dt)?;
    let out = layout
        .encode(Some(&bytes), Some(&cam))
        .map_err(|e| Error::new(ErrorKind::Invalid, e))?;
    ctx.set_resource_bytes(*cid, &out)
}

/// The og.CameraZone the entity followed by og.Follow2D is in (the highest
/// priority, then the first in query order), into og.Camera's zone fields;
/// entering another one starts a scroll over its scrollTime.
pub(super) fn camera_zone(
    ctx: &mut SystemContext<'_>,
    r: &Records,
    cam: &mut Value,
    row: &record::Camera,
    dt: f32,
) -> og_core::Result<()> {
    let mut target: Option<Vec2> = None;
    for a in ctx.query(2)? {
        if let Some(&e) = a.entities().first() {
            let t: record::Transform = r
                .transform
                .1
                .read(&ctx.get_bytes(e, r.transform.0)?)
                .map_err(invalid)?;
            target = Some(t.position.truncate());
            break;
        }
    }
    let mut best: Option<(i64, Vec2, Vec2, f32)> = None;
    if let Some(p) = target {
        for a in ctx.query(1)? {
            for &e in a.entities() {
                let z: record::CameraZone = r
                    .zone
                    .1
                    .read(&ctx.get_bytes(e, r.zone.0)?)
                    .map_err(invalid)?;
                let (lo, hi) = (z.min, z.max);
                if p.x < lo.x || p.x > hi.x || p.y < lo.y || p.y > hi.y {
                    continue;
                }
                let pr = i64::from(z.priority);
                if best.is_none_or(|b| pr > b.0) {
                    best = Some((pr, lo, hi, z.scroll_time.max(0.0)));
                }
            }
        }
    }
    // `row` is the camera before this function's writes.
    let active = row.zone_active;
    let (lo0, hi0) = (row.zone_min, row.zone_max);
    let mut scroll = row.zone_scroll.clamp(0.0, 1.0);
    let mut time = row.zone_scroll_time.max(0.0);
    let mut from_active = row.zone_from_active;
    let xy = |v: Vec2| json!({ "x": v.x, "y": v.y });
    match best {
        Some((_, lo, hi, t)) if !active || lo != lo0 || hi != hi0 => {
            let scrolls = active && t > 0.0;
            from_active = scrolls;
            cam["zoneFromActive"] = json!(scrolls);
            cam["zoneFromMin"] = xy(if scrolls { lo0 } else { Vec2::ZERO });
            cam["zoneFromMax"] = xy(if scrolls { hi0 } else { Vec2::ZERO });
            cam["zoneActive"] = json!(true);
            cam["zoneMin"] = xy(lo);
            cam["zoneMax"] = xy(hi);
            scroll = if scrolls { 0.0 } else { 1.0 };
            time = t;
        }
        Some(_) => {
            if scroll < 1.0 {
                scroll = if time > 0.0 {
                    (scroll + dt / time).min(1.0)
                } else {
                    1.0
                };
            }
        }
        None => {
            cam["zoneActive"] = json!(false);
            from_active = false;
            cam["zoneFromActive"] = json!(false);
            scroll = 1.0;
        }
    }
    if scroll >= 1.0 && from_active {
        cam["zoneFromActive"] = json!(false);
    }
    cam["zoneScroll"] = json!(scroll);
    cam["zoneScrollTime"] = json!(time);
    Ok(())
}

/// The rotation that points -Z from `eye` towards `target`, +Y up.
pub(crate) fn look_rotation(eye: Vec3, target: Vec3) -> Quat {
    let forward = (target - eye).normalize_or_zero();
    if forward == Vec3::ZERO {
        return Quat::IDENTITY;
    }
    let z = -forward;
    let x = Vec3::Y.cross(z).normalize_or(Vec3::X);
    let y = z.cross(x);
    Quat::from_mat3(&glam::Mat3::from_cols(x, y, z))
}

/// A point of the Catmull-Rom spline through `p` (ends repeated) at segment
/// `i`, parameter `t`.
pub(super) fn catmull(p: &[Vec3], i: usize, t: f32) -> Vec3 {
    let at = |k: isize| p[k.clamp(0, p.len() as isize - 1) as usize];
    let (p0, p1, p2, p3) = (
        at(i as isize - 1),
        at(i as isize),
        at(i as isize + 1),
        at(i as isize + 2),
    );
    let (t2, t3) = (t * t, t * t * t);
    ((p1 * 2.0)
        + (p2 - p0) * t
        + (p0 * 2.0 - p1 * 5.0 + p2 * 4.0 - p3) * t2
        + (p1 * 3.0 - p0 - p2 * 3.0 + p3) * t3)
        * 0.5
}

/// The rail point nearest `target`: 32 samples per segment, then a refinement.
pub(crate) fn rail_nearest(p: &[Vec3], target: Vec3) -> Vec3 {
    const S: usize = 32;
    let mut best = (f32::MAX, 0usize, 0.0f32);
    for i in 0..p.len() - 1 {
        for s in 0..=S {
            let t = s as f32 / S as f32;
            let d = catmull(p, i, t).distance_squared(target);
            if d < best.0 {
                best = (d, i, t);
            }
        }
    }
    let (_, i, mut t) = best;
    let mut step = 0.5 / S as f32;
    for _ in 0..8 {
        let (a, b) = ((t - step).max(0.0), (t + step).min(1.0));
        let (da, db, dt) = (
            catmull(p, i, a).distance_squared(target),
            catmull(p, i, b).distance_squared(target),
            catmull(p, i, t).distance_squared(target),
        );
        if da < dt && da <= db {
            t = a;
        } else if db < dt {
            t = b;
        }
        step *= 0.5;
    }
    catmull(p, i, t)
}

pub(super) struct CameraBlock(pub(super) Arc<Mutex<FeelShared>>);

impl StateBlock for CameraBlock {
    fn save(&self, out: &mut Vec<u8>) {
        let sh = lock(&self.0);
        out.extend_from_slice(&(sh.camera_ops.len() as u32).to_le_bytes());
        for op in &sh.camera_ops {
            let (k, v) = match *op {
                CameraOp::Shake(t) => (0u32, Vec3::new(t, 0.0, 0.0)),
                CameraOp::Punch(p) => (1, Vec3::new(p, 0.0, 0.0)),
                CameraOp::Kick(v) => (2, v),
            };
            out.extend_from_slice(&k.to_le_bytes());
            for x in v.to_array() {
                out.extend_from_slice(&x.to_le_bytes());
            }
        }
    }

    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn Any + Send>> {
        let mut r = Reader::new(bytes, "og.camera");
        let n = r.u32()?;
        let mut ops = Vec::new();
        for _ in 0..n {
            let k = r.u32()?;
            let v = Vec3::new(r.f32()?, r.f32()?, r.f32()?);
            ops.push(match k {
                0 => CameraOp::Shake(v.x),
                1 => CameraOp::Punch(v.x),
                _ => CameraOp::Kick(v),
            });
        }
        r.end()?;
        Ok(Box::new(ops))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        lock(&self.0).camera_ops = decoded
            .map(|d| {
                *d.downcast::<Vec<CameraOp>>()
                    .expect("decoded by CameraBlock")
            })
            .unwrap_or_default();
    }

    /// Shakes, punches and kicks are offsets: nothing in the world.
    fn shift(&mut self, _: &og_ecs::Shift) {}
}
