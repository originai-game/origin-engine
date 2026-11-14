//! R10's virtual cameras: og.VirtualCamera shots, og.CameraBrain picking
//! the live one and blending og.Camera to it, and collision that keeps a
//! shot clear of colliders. Run at the end of the og.camera system, after
//! rails and zones.
//!
//! All three records are lazy (crate::engine::lazy): a world without
//! virtual cameras keeps its type table, saves and hash. Every value the
//! brain keeps is a record field (the vcams' controller state, the brain's
//! blend), so snapshots, restore and the hash cover it with no state block.
//! The pose is computed from the tick's state alone, with og-core's
//! deterministic math and no randomness, and it writes only og.Camera,
//! og.PostEffects' depth of field (for a shot with focus), its own records
//! and og.CameraCut: the simulation reads none of them unless the game
//! does (§2.4).

use super::*;
use og_physics::{PhysicsSystem, QueryFilter, SHAPE_SPHERE, ShapeCastDesc, ShapeDesc};

pub(crate) const VCAM: &str = "og.VirtualCamera";
pub(crate) const BRAIN: &str = "og.CameraBrain";
pub(crate) const CUT: &str = "og.CameraCut";
const POST: &str = "og.PostEffects";
/// Registered together (crate::engine::lazy::TOGETHER).
pub(crate) const RECORDS: [&str; 3] = [VCAM, BRAIN, CUT];

/// The layouts of the lazy records (and og.PostEffects), resolved at
/// assembly; their ids are looked up per tick, once registered.
#[derive(Clone)]
pub(crate) struct Layouts {
    vcam: Option<Layout>,
    brain: Option<Layout>,
    cut: Option<Layout>,
    post: Option<Layout>,
}

impl Layouts {
    pub(crate) fn of(layouts: &BTreeMap<String, Layout>) -> Layouts {
        Layouts {
            vcam: layouts.get(VCAM).cloned(),
            brain: layouts.get(BRAIN).cloned(),
            cut: layouts.get(CUT).cloned(),
            post: layouts.get(POST).cloned(),
        }
    }
}

/// The lazy grant: og.camera writes the records (and og.PostEffects, for
/// focus); og.CameraBrain starts at its declared defaults.
pub(crate) fn enable(world: &mut World, name: &str) -> og_core::Result<()> {
    let mut names = vec![name];
    if name == BRAIN {
        let id = world.id_by_name(BRAIN)?;
        if !world.has_resource_id(id) {
            let bytes = og_api::host::RECORDS
                .iter()
                .find(|d| d.schema == BRAIN)
                .and_then(|d| Layout::from_decl(d).ok())
                .and_then(|l| l.encode(None, None).ok())
                .ok_or_else(|| Error::new(ErrorKind::NotFound, "og.CameraBrain has no layout"))?;
            world.insert_resource_bytes(id, &bytes)?;
        }
        if world.id_by_name(POST).is_ok() {
            names.push(POST);
        }
    }
    world.grant(CAMERA_SYSTEM, &names)
}

/// A shot's pose: where the camera is, how it is turned, its field of
/// view and its focus (distance, range) if it has one.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Pose {
    position: Vec3,
    rotation: Quat,
    fov: f32,
    focus: Option<(f32, f32)>,
}

/// `q` normalized (a zero quaternion: the identity).
fn unit(q: Quat) -> Quat {
    if q.length_squared() > 0.0 {
        q.normalize()
    } else {
        Quat::IDENTITY
    }
}

fn jv3(v: Vec3) -> Value {
    json!({ "x": v.x, "y": v.y, "z": v.z })
}

fn jq(q: Quat) -> Value {
    json!({ "x": q.x, "y": q.y, "z": q.z, "w": q.w })
}

fn ease_of(text: &str) -> Ease {
    Ease::from_text(text).expect("validated reflected timing value")
}

fn entity(bits: u64) -> Option<Entity> {
    match bits {
        0 => None,
        b => Some(Entity::from_bits(b)),
    }
}

/// An entity's og.Transform position and rotation, if it is alive and
/// has one.
fn transform(
    ctx: &SystemContext<'_>,
    r: &Records,
    e: Entity,
) -> og_core::Result<Option<(Vec3, Quat)>> {
    if !ctx.is_alive(e) {
        return Ok(None);
    }
    let Ok(b) = ctx.get_bytes(e, r.transform.0) else {
        return Ok(None);
    };
    let t: record::Transform = r.transform.1.read(&b).map_err(invalid)?;
    Ok(Some((t.position, t.quaternion.normalize())))
}

/// `e`, when it is an entity, with its [`transform`].
fn placed(
    ctx: &SystemContext<'_>,
    r: &Records,
    e: Option<Entity>,
) -> og_core::Result<Option<(Entity, (Vec3, Quat))>> {
    let Some(e) = e else {
        return Ok(None);
    };
    Ok(transform(ctx, r, e)?.map(|t| (e, t)))
}

fn lerp(a: f32, b: f32, w: f32) -> f32 {
    a + (b - a) * w
}

/// Eases `from` towards `to` over roughly `smooth` seconds (0 snaps).
fn approach(from: f32, to: f32, dt: f32, smooth: f32) -> f32 {
    if smooth > 0.0 {
        from + (to - from) * (dt / smooth).min(1.0)
    } else {
        to
    }
}

/// One shot's pose this tick, updating its controller state in `v`
/// (`eased`, `pull`): follow with smoothing, look, collision, roll.
/// `snap` (just enabled) skips the smoothing.
fn evaluate(
    ctx: &SystemContext<'_>,
    r: &Records,
    e: Entity,
    (v, row): (&mut Value, &record::VirtualCamera),
    dt: f32,
    snap: bool,
    physics: Option<&og_physics::PhysicsQuery>,
) -> og_core::Result<Pose> {
    let own = transform(ctx, r, e)?.unwrap_or((Vec3::ZERO, Quat::IDENTITY));
    let follow = placed(ctx, r, entity(row.follow))?;
    let look_offset = row.look_offset;
    let mut position = own.0;
    if let Some((_, (fp, fr))) = follow {
        let offset = row.follow_offset;
        // With followRotation the offset is right, up and behind the
        // followed entity, so the default sits behind what faces +Z.
        let offset = if row.follow_rotation {
            og_core::facing::right(fr) * offset.x + og_core::facing::up(fr) * offset.y
                - og_core::facing::forward(fr) * offset.z
        } else {
            offset
        };
        let goal = fp + offset;
        // A shot without `eased` (the IDL's default is zero) starts at its
        // goal, as the hand read had it.
        let eased = if v.get("eased").is_some() {
            row.eased
        } else {
            goal
        };
        let smooth = row.smooth_time.max(0.0);
        position = if snap {
            goal
        } else {
            Vec3::new(
                approach(eased.x, goal.x, dt, smooth),
                approach(eased.y, goal.y, dt, smooth),
                approach(eased.z, goal.z, dt, smooth),
            )
        };
    }
    v["eased"] = jv3(position);
    let look = placed(ctx, r, entity(row.look_at))?.map(|(l, t)| (l, t.0 + look_offset));
    // Collision keeps the way from the look point (else the followed
    // entity's, at lookOffset) to the camera clear.
    let origin = look.or_else(|| follow.map(|(fe, (fp, _))| (fe, fp + look_offset)));
    let mut pull = 0.0;
    if row.collide
        && let (Some(q), Some((excluded, o))) = (physics, origin)
    {
        let d = position - o;
        let dist = d.length();
        if dist > 1e-4 {
            let dir = d / dist;
            let desc = ShapeCastDesc {
                shape: ShapeDesc {
                    shape: SHAPE_SPHERE,
                    radius: row.radius.max(0.0),
                    ..ShapeDesc::default()
                },
                position: o,
                rotation: Quat::IDENTITY,
                direction: dir,
                max_distance: dist,
                filter: QueryFilter {
                    mask: row.collision_mask,
                    include_sensors: false,
                    exclude: Some(follow.map_or(excluded, |(fe, _)| fe)),
                },
            };
            let clear = match q.shape_cast(&desc) {
                Ok(Some(hit)) => hit.distance.clamp(0.0, dist),
                _ => dist,
            };
            let want = dist - clear;
            let held = row.pull.max(0.0);
            // In at once, out over recoverTime.
            pull = if snap || want >= held {
                want
            } else {
                approach(held, want, dt, row.recover_time.max(0.0))
            };
            pull = pull.clamp(0.0, dist);
            position = o + dir * (dist - pull);
        }
    }
    v["pull"] = json!(pull);
    let mut rotation = match look {
        Some((_, p)) => og_core::facing::camera_look(position, p, og_core::facing::UP),
        None => own.1,
    };
    let roll = row.roll;
    if roll != 0.0 {
        let (s, c) = og_core::math::sin_cos(roll * (std::f32::consts::PI / 360.0));
        rotation = (rotation * Quat::from_xyzw(0.0, 0.0, s, c)).normalize();
    }
    let focus = row.focus;
    Ok(Pose {
        position,
        rotation,
        fov: row.fov,
        focus: (focus > 0.0).then(|| (focus, row.focus_range.max(0.0))),
    })
}

/// `from` blended to `to` at curve weight `w`.
fn mix(from: Pose, to: Pose, w: f32) -> Pose {
    Pose {
        position: from.position + (to.position - from.position) * w,
        rotation: og_core::math::quat_slerp(from.rotation, to.rotation, w),
        fov: lerp(from.fov, to.fov, w),
        focus: match (from.focus, to.focus) {
            (Some(a), Some(b)) => Some((lerp(a.0, b.0, w), lerp(a.1, b.1, w))),
            (a, None) => a,
            (None, b) => b,
        },
    }
}

/// The brain: picks the live shot, advances the blend and writes the pose
/// into `cam` (og.Camera, decoded). Nothing happens before og.VirtualCamera
/// is registered.
pub(crate) fn brain(
    ctx: &mut SystemContext<'_>,
    engine_id: u32,
    r: &Records,
    physics: &PhysicsSystem,
    cam: &mut Value,
    dt: f32,
) -> og_core::Result<()> {
    let l = &r.vcams;
    let (Some(vl), Some(bl)) = (&l.vcam, &l.brain) else {
        return Ok(());
    };
    let (Ok(vid), Ok(bid)) = (ctx.id_by_name(VCAM), ctx.id_by_name(BRAIN)) else {
        return Ok(());
    };
    if !ctx.has_resource_id(bid)? {
        return Ok(());
    }
    let bad = |e: String| Error::new(ErrorKind::Invalid, e);
    let brain_bytes = ctx.resource_bytes(bid)?;
    let mut brain = bl.decode(&brain_bytes);
    let brain_row = brain_of(&brain).map_err(bad)?;
    let mut activations = brain_row.activations;
    // Every shot, in query order.
    let mut shots: Vec<(Entity, Vec<u8>, Value, record::VirtualCamera)> = Vec::new();
    let q = QueryDesc {
        write: vec![VCAM.into()],
        ..QueryDesc::default()
    };
    for a in ctx.query_with(&q)? {
        for &e in a.entities() {
            let bytes = ctx.get_bytes(e, vid)?;
            let v = vl.decode(&bytes);
            let row = record::VirtualCamera::from_json(&v).map_err(bad)?;
            shots.push((e, bytes, v, row));
        }
    }
    // A shot not evaluated last tick (just enabled, or a disabled solo
    // shot) snaps to its goal; one just enabled gets the next activation,
    // so the later enabled wins a priority tie.
    let snapped: Vec<bool> = shots.iter().map(|(.., row)| !row.primed).collect();
    for (_, _, v, row) in &mut shots {
        if row.enabled && !row.primed {
            activations = activations.wrapping_add(1);
            v["activated"] = json!(activations);
            row.activated = activations;
        }
    }
    brain["activations"] = json!(activations);
    // The pick: the solo shot, else the highest priority, the later
    // enabled, the first in query order.
    let solo = entity(brain_row.solo).filter(|s| shots.iter().any(|(e, ..)| e == s));
    let mut best: Option<(usize, i64, u64)> = None;
    for (i, (.., row)) in shots.iter().enumerate() {
        if !row.enabled {
            continue;
        }
        let key = (i64::from(row.priority), u64::from(row.activated));
        if best.is_none_or(|b| key > (b.1, b.2)) {
            best = Some((i, key.0, key.1));
        }
    }
    let live = solo.or_else(|| best.map(|b| shots[b.0].0));
    let prev = entity(brain_row.live);
    let order: Vec<Entity> = shots.iter().map(|s| s.0).collect();
    let index = |e: Entity| order.iter().position(|x| *x == e);
    let physics = physics.tick_query().ok();
    // Every enabled shot (and the live and outgoing ones) keeps its follow
    // state current, so a cut to a standby shot lands where it should.
    let from_entity = entity(brain_row.from);
    let mut poses: Vec<Option<Pose>> = vec![None; shots.len()];
    for (i, (e, _, v, row)) in shots.iter_mut().enumerate() {
        let e = *e;
        let involved = Some(e) == live || Some(e) == prev || Some(e) == from_entity;
        let evaluated = row.enabled || involved;
        v["primed"] = json!(evaluated);
        if !evaluated {
            continue;
        }
        let collide = if involved { physics.as_ref() } else { None };
        poses[i] = Some(evaluate(ctx, r, e, (v, &*row), dt, snapped[i], collide)?);
    }
    let mut blend = brain_row.blend.clamp(0.0, 1.0);
    let mut from = from_entity;
    if live != prev {
        let solo_blend = brain_row.solo_blend;
        let (duration, curve) = match live.and_then(index) {
            // A blend to the solo shot by the solo's own (a timeline's shot key).
            Some(_) if live == solo && solo_blend >= 0.0 => {
                (solo_blend, ease_of(&brain_row.solo_ease))
            }
            Some(i) if shots[i].3.blend_time >= 0.0 => {
                (shots[i].3.blend_time, ease_of(&shots[i].3.blend_ease))
            }
            _ => (
                brain_row.blend_time.max(0.0),
                ease_of(&brain_row.blend_ease),
            ),
        };
        // The first live shot, a shot going away and a zero blend cut.
        let blends = prev.is_some() && live.is_some() && duration > 0.0;
        if blends && blend < 1.0 {
            // Interrupted: from the blended pose as shown, held.
            // What the camera shows now: the last pose written.
            let shown = record::Camera::from_json(cam).map_err(bad)?;
            let p = Pose {
                position: shown.position,
                rotation: unit(shown.quaternion),
                fov: shown.fov,
                focus: if brain_row.focus_held {
                    post_focus(ctx, l).map_err(bad)?
                } else {
                    None
                },
            };
            brain["fromPosition"] = jv3(p.position);
            brain["fromQuaternion"] = jq(p.rotation);
            brain["fromFov"] = json!(p.fov);
            brain["fromFocus"] = json!(p.focus.map_or(0.0, |x| x.0));
            brain["fromFocusRange"] = json!(p.focus.map_or(0.0, |x| x.1));
            from = None;
        } else {
            from = prev.filter(|_| blends);
        }
        blend = if blends { 0.0 } else { 1.0 };
        brain["blendDuration"] = json!(if blends { duration } else { 0.0 });
        brain["blendCurve"] = json!(curve.to_text());
        if let (Some(cl), Ok(cid)) = (&l.cut, ctx.id_by_name(CUT)) {
            let ev = json!({
                "from": prev.map_or(0, Entity::to_bits),
                "to": live.map_or(0, Entity::to_bits),
                "blend": if blends { duration } else { 0.0 },
            });
            ctx.send_bytes(cid, &cl.encode(None, Some(&ev)).map_err(bad)?)?;
        }
    } else if blend < 1.0 {
        let duration = brain_row.blend_duration;
        blend = if duration > 0.0 {
            (blend + dt / duration).min(1.0)
        } else {
            1.0
        };
    }
    if blend >= 1.0 {
        from = None;
    }
    brain["live"] = json!(live.map_or(0, Entity::to_bits));
    brain["from"] = json!(from.map_or(0, Entity::to_bits));
    brain["blend"] = json!(blend);
    // The brain as this tick's writes left it: the blend's curve and the
    // held pose.
    let now = brain_of(&brain).map_err(bad)?;
    // The pose: the live shot's, mixed from the outgoing shot (or the held
    // pose: an interrupted blend, or an outgoing shot that went away).
    let out = live.and_then(index).and_then(|i| poses[i]).map(|to| {
        if blend >= 1.0 {
            return to;
        }
        let w = ease_of(&now.blend_curve).apply(blend);
        let from_pose = from
            .and_then(index)
            .and_then(|i| poses[i])
            .unwrap_or_else(|| held(&now));
        mix(from_pose, to, w)
    });
    // A held pose outlives the shot it came from.
    if from.is_some_and(|e| index(e).and_then(|i| poses[i]).is_none()) {
        brain["from"] = json!(0);
    }
    let drives = brain_row.enabled && !crate::bind::timeline::owns_camera(engine_id);
    if drives {
        if let Some(p) = out {
            cam["position"] = jv3(p.position);
            cam["quaternion"] = jq(p.rotation);
            cam["fov"] = json!(p.fov);
        }
        let focus_held = brain_row.focus_held;
        focus(ctx, l, &mut brain, focus_held, out.and_then(|p| p.focus))?;
    }
    for (e, bytes, v, _) in &shots {
        let next = vl.encode(Some(bytes), Some(v)).map_err(bad)?;
        if next != *bytes {
            ctx.set_bytes(*e, vid, &next)?;
        }
    }
    let next = bl.encode(Some(&brain_bytes), Some(&brain)).map_err(bad)?;
    if next != brain_bytes {
        ctx.set_resource_bytes(bid, &next)?;
    }
    Ok(())
}

/// og.CameraBrain as its IDL row. An absent fromFov read 50 and an absent
/// fromFocusRange 1 by hand, where the IDL's defaults are 0: kept.
fn brain_of(v: &Value) -> Result<record::CameraBrain, String> {
    let base = record::CameraBrain {
        from_fov: 50.0,
        from_focus_range: 1.0,
        ..Default::default()
    };
    record::CameraBrain::from_json_over(v, base)
}

/// The brain's held pose: an interrupted blend starts from it.
fn held(brain: &record::CameraBrain) -> Pose {
    let focus = brain.from_focus;
    Pose {
        position: brain.from_position,
        rotation: unit(brain.from_quaternion),
        fov: brain.from_fov,
        focus: (focus > 0.0).then_some((focus, brain.from_focus_range)),
    }
}

/// og.PostEffects' focus (distance, range) while depth of field is on.
fn post_focus(ctx: &SystemContext<'_>, l: &Layouts) -> Result<Option<(f32, f32)>, String> {
    let (Some(pl), Ok(id)) = (l.post.as_ref(), ctx.id_by_name(POST)) else {
        return Ok(None);
    };
    let Ok(bytes) = ctx.resource_bytes(id) else {
        return Ok(None);
    };
    // An absent dofFocusDistance read 0 and an absent dofFocusRange 1 by
    // hand, where the IDL's defaults are 10 and 2: kept.
    let base = record::PostEffects {
        dof_focus_distance: 0.0,
        dof_focus_range: 1.0,
        ..Default::default()
    };
    let p = record::PostEffects::from_json_over(&pl.decode(&bytes), base)?;
    Ok(p.depth_of_field
        .then_some((p.dof_focus_distance, p.dof_focus_range)))
}

/// Drives og.PostEffects' depth of field with the shot's focus, holding
/// what it found (`held`: the brain's focusHeld as the tick found it);
/// hands it back when no shot has focus.
fn focus(
    ctx: &mut SystemContext<'_>,
    l: &Layouts,
    brain: &mut Value,
    held: bool,
    want: Option<(f32, f32)>,
) -> og_core::Result<()> {
    if want.is_none() && !held {
        return Ok(());
    }
    let (Some(pl), Ok(id)) = (&l.post, ctx.id_by_name(POST)) else {
        return Ok(());
    };
    if !ctx.has_resource_id(id)? {
        return Ok(());
    }
    let bytes = ctx.resource_bytes(id)?;
    let mut p = pl.decode(&bytes);
    match want {
        Some((distance, range)) => {
            if !held {
                brain["focusHeld"] = json!(true);
                brain["heldDepthOfField"] = p["depthOfField"].clone();
                brain["heldFocusDistance"] = p["dofFocusDistance"].clone();
                brain["heldFocusRange"] = p["dofFocusRange"].clone();
            }
            p["depthOfField"] = json!(true);
            p["dofFocusDistance"] = json!(distance);
            p["dofFocusRange"] = json!(range);
        }
        None => {
            brain["focusHeld"] = json!(false);
            p["depthOfField"] = brain["heldDepthOfField"].clone();
            p["dofFocusDistance"] = brain["heldFocusDistance"].clone();
            p["dofFocusRange"] = brain["heldFocusRange"].clone();
        }
    }
    let out = pl
        .encode(Some(&bytes), Some(&p))
        .map_err(|e| Error::new(ErrorKind::Invalid, e))?;
    if out != bytes {
        ctx.set_resource_bytes(id, &out)?;
    }
    Ok(())
}
