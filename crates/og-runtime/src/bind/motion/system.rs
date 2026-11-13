//! The og.motion system: where a jump lands, and the tick that moves every agent.

use super::*;

/// The step at which [`landing_in`] samples an arc, and how far ahead, s.
pub(super) const LANDING_STEP: f32 = 1.0 / 30.0;

pub(super) const LANDING_HORIZON: f32 = 3.0;

/// The height of the ground under `at` (the first collider straight down
/// that is not `e`'s), m.
pub(super) fn ground(q: &og_physics::PhysicsQuery, e: Entity, at: Vec3) -> Option<f32> {
    q.raycast(&og_physics::RaycastDesc {
        origin: at,
        direction: Vec3::NEG_Y,
        max_distance: 0.0,
        filter: og_physics::QueryFilter {
            mask: 0,
            include_sensors: false,
            exclude: Some(e),
        },
    })
    .ok()
    .flatten()
    .map(|h| at.y - h.distance)
}

/// Seconds until an agent in the air at `at`, its feet `rest` below it,
/// rising at `v` under gravity `g` and crossing the ground at `across`,
/// lands: its ballistic arc stepped every [`LANDING_STEP`] with the ground
/// traced under each step, to the first where the feet meet it (between
/// steps, linearly). It is the landing a trajectory predicts (distance
/// matching's), not the start height: a ledge up lands early, a drop late.
/// `None` when nothing is below within [`LANDING_HORIZON`].
pub(super) fn landing_in(
    q: &og_physics::PhysicsQuery,
    e: Entity,
    at: Vec3,
    rest: f32,
    v: f32,
    g: f32,
    across: Vec3,
) -> Option<f32> {
    let feet = |t: f32| at.y - rest + v * t - 0.5 * g * t * t;
    let mut t = 0.0;
    while t < LANDING_HORIZON {
        let next = t + LANDING_STEP;
        let p = at + across * next;
        let y = feet(next);
        if let Some(floor) = ground(q, e, Vec3::new(p.x, y + rest, p.z))
            && y <= floor
        {
            let before = feet(t);
            let k = if before > y {
                ((before - floor) / (before - y)).clamp(0.0, 1.0)
            } else {
                1.0
            };
            return Some(t + k * LANDING_STEP);
        }
        t = next;
    }
    None
}

/// Registers og.MotionAgent's module on a world that names it (`crate::engine::lazy`):
/// the `og.motion` system, after every system scheduled so far, and its
/// state block.
pub(crate) fn enable(world: &mut World, mount: Mount<'_>) -> og_core::Result<()> {
    let engine_id = mount.id;
    if world.systems().contains(&SYSTEM) {
        return Ok(());
    }
    let layout = |world: &World, n: &str| -> og_core::Result<(SchemaId, Layout)> {
        let id = world.id_by_name(n)?;
        Ok((
            id,
            Layout::from_schema(world.schema(id)?, RecordKind::Component),
        ))
    };
    let agent = layout(world, AGENT)?;
    let transform = layout(world, "og.Transform")?;
    let controller = layout(world, "og.CharacterController")?;
    let sh = mount.modules.motion.0.clone();
    *lock(&sh) = Goals::default();
    let state = sh.clone();
    world.add_system(
        SystemDesc {
            name: SYSTEM.into(),
            reads: vec![AGENT.to_string(), "og.Physics".to_string()],
            writes: vec![
                "og.Transform".to_string(),
                "og.CharacterController".to_string(),
            ],
            queries: vec![QueryDesc {
                read: vec![AGENT.to_string()],
                write: vec!["og.Transform".to_string()],
                ..QueryDesc::default()
            }],
            ..Default::default()
        },
        move |ctx| system(ctx, engine_id, &state, &agent, &transform, &controller),
    )?;
    world.move_system_before(SYSTEM, "og.physics")?;
    world.add_state_block(SYSTEM, Box::new(Block(sh)))
}

pub(super) fn system(
    ctx: &mut SystemContext<'_>,
    engine_id: u32,
    state: &Mutex<Goals>,
    agent: &(SchemaId, Layout),
    transform: &(SchemaId, Layout),
    controller: &(SchemaId, Layout),
) -> og_core::Result<()> {
    let dt = ctx.scaled_dt();
    let mut rows = Vec::new();
    for a in ctx.query(0)? {
        rows.extend_from_slice(a.entities());
    }
    let gravity = ctx
        .resource::<og_physics::Physics>()
        .map_or(GRAVITY, |p| -p.gravity.y);
    // The colliders as this tick's start recorded them, for the ground a
    // controller jump lands on (the same view og.nav and og.ai cast in).
    let physics = engine(engine_id)
        .ok()
        .and_then(|eng| eng.state.borrow().physics.clone());
    let query = physics.as_ref().and_then(|p| p.tick_query().ok());
    let mut goals = lock(state);
    // Agents whose entity lost og.MotionAgent (or died): forgotten.
    goals
        .agents
        .retain(|k, _| rows.iter().any(|e| e.to_bits() == *k));
    for e in rows {
        let Some(a) = goals.agents.get_mut(&e.to_bits()) else {
            continue;
        };
        let mut s = Settings::of(&agent.1.read(&ctx.get_bytes(e, agent.0)?).map_err(invalid)?);
        let cb = ctx.get_bytes(e, controller.0).ok();
        s.controller = cb.is_some();
        s.gravity = gravity;
        let c0 = cb.as_ref().map(|b| controller.1.decode(b));
        if let Some(c) = &c0 {
            let row = wire::record::CharacterController::from_json(c).map_err(invalid)?;
            s.gravity = gravity * row.gravity_scale;
            s.grounded = row.grounded;
            s.vertical = row.velocity.y;
        }
        let tb = ctx.get_bytes(e, transform.0)?;
        let mut t = transform.1.decode(&tb);
        let (mut position, q) =
            transform_of(&wire::record::Transform::from_json(&t).map_err(invalid)?);
        let mut heading = facing::heading(q);
        let (p0, h0) = (position, heading);
        advance(a, &s, &mut position, &mut heading, dt);
        if s.controller
            && let Some(cur) = a.current.as_mut().filter(|c| c.goal.kind == Kind::Jump)
            && let Some(j) = cur.jump.as_mut()
            && j.stage == JumpStage::Air
        {
            // On the takeoff tick the controller still stands where it
            // will leave the ground from.
            if j.rest.is_none() {
                j.rest = query
                    .as_ref()
                    .and_then(|q| ground(q, e, p0))
                    .map(|g| p0.y - g);
            }
            let flight = 2.0 * j.v0 / j.gravity.max(1e-6);
            let (sx, cz) = sin_cos(cur.goal.heading);
            let across = Vec3::new(sx, 0.0, cz) * (cur.goal.speed / flight);
            j.eta = query
                .as_ref()
                .zip(j.rest)
                .and_then(|(q, rest)| landing_in(q, e, p0, rest, j.velocity, j.gravity, across));
        }
        if s.drive
            && let (Some(cb), Some(mut c)) = (cb, c0)
        {
            let velocity = if dt > 0.0 {
                (position - p0) / dt
            } else {
                Vec3::ZERO
            };
            c["move"] = json!({"x": velocity.x, "y": 0.0, "z": velocity.z});
            // A jump goal holds the controller's jump on its takeoff tick
            // only (the controller starts a jump on the rising edge), at the
            // speed whose discrete arc (velocity, then position, each tick)
            // peaks at the goal's height.
            if let Some(cur) = a.current.as_ref().filter(|c| c.goal.kind == Kind::Jump)
                && let Some(j) = cur.jump
            {
                let takeoff = j.stage == JumpStage::Air && j.air == 0.0;
                c["jump"] = json!(takeoff);
                if takeoff {
                    c["jumpSpeed"] = json!(j.v0 + 0.5 * j.gravity * dt);
                }
            }
            let bytes = controller.1.encode(Some(&cb), Some(&c)).map_err(invalid)?;
            ctx.set_bytes(e, controller.0, &bytes)?;
            // The controller alone writes translation after collisions,
            // gravity and ground snapping; goals see that result next tick.
            position = p0;
        }
        if s.drive && (position != p0 || heading != h0) {
            t["position"] = json!({ "x": position.x, "y": position.y, "z": position.z });
            let r = facing::from_heading(heading);
            t["quaternion"] = json!({ "x": r.x, "y": r.y, "z": r.z, "w": r.w });
            let tb = transform.1.encode(Some(&tb), Some(&t)).map_err(invalid)?;
            ctx.set_bytes(e, transform.0, &tb)?;
        }
    }
    Ok(())
}

/// og.Transform's position and rotation.
pub(crate) fn transform_of(t: &wire::record::Transform) -> (Vec3, Quat) {
    (t.position, t.quaternion.normalize())
}
