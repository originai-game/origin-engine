//! `og.motion` (docs/design.md §2.7, "Motion generation"): goals for
//! humanoids, kept and finished by the simulation, and the engine's motion
//! model drawing them ([`present`]).
//!
//! Two halves, one rule (§2.4): the simulation never reads the model.
//!
//! - **The goals** are world state: per og.MotionAgent entity its queue,
//!   its current goal and how long it has been current, in the `og.motion`
//!   state block (snapshots, restores and the world hash carry it). The
//!   `og.motion` system finishes goals by fixed rules and, with
//!   `MotionAgent.drive`, moves og.Transform to them with fixed math
//!   (walkTo at its speed in a straight line, place over its duration,
//!   turns at `turnSpeed`). og.motion.status answers from here alone.
//! - **The pose** is presentation ([`present`]): before each frame is
//!   drawn, the model (on the renderer's device) is asked for motion that
//!   reaches where the goals will have the character, a batch for every
//!   agent at once, and the frame's generated pose is re-rooted onto
//!   og.Transform, retargeted to the agent's rig, its feet planted, and
//!   drawn in place of og.anim's joint palette. Nothing writes the world.
//!
//! og.MotionAgent is registered lazily (`super::lazy`): a world that never
//! names it has neither the system nor the state block, so its saves and
//! hash are what they were before og.motion existed.

mod performance;
pub(crate) mod present;
pub use performance::{og_motion_look_at, og_motion_react};

use super::codec::{Layout, RecordKind};
use super::engine::{Engine, engine};
use super::{ok, ok_void, parse, refuse};
use glam::{Quat, Vec3};
use og_core::math::{atan2, sin_cos};
use og_core::{Error, ErrorKind};
use og_ecs::{Entity, QueryDesc, SchemaId, StateBlock, SystemContext, SystemDesc, World};
use og_motion::library::{POSES, index as pose_index};
use serde_json::{Value, json};
use std::any::Any;
use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard};

pub(crate) const SYSTEM: &str = "og.motion";
pub(crate) const AGENT: &str = "og.MotionAgent";
/// Below this speed a walkTo walks, above it runs (WalkGoalDesc.speed).
pub(crate) const RUN_SPEED: f32 = 2.5;
/// A walkTo's acceleration when its desc leaves it at 0, m/s².
const ACCELERATION: f32 = 4.0;
/// A walkTo ends within this distance of its target.
const ARRIVED: f32 = 1e-3;
/// The gravity of a world without og.Physics, m/s².
const GRAVITY: f32 = 9.81;
/// A controller jump that never leaves the ground (blocked, or not
/// grounded at takeoff) lands after this long in the air.
const STUCK: f32 = 0.2;

/// MotionGoalKind's variants, in the IDL's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Pose = 1,
    Place = 2,
    Face = 3,
    WalkTo = 4,
    Jump = 5,
}

const KIND_NAMES: [&str; 6] = ["none", "pose", "place", "face", "walkTo", "jump"];

impl Kind {
    fn from_u32(v: u32) -> Option<Kind> {
        Some(match v {
            1 => Kind::Pose,
            2 => Kind::Place,
            3 => Kind::Face,
            4 => Kind::WalkTo,
            5 => Kind::Jump,
            _ => return None,
        })
    }
}

/// A goal as the game queued it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Goal {
    pub kind: Kind,
    /// Index into [`POSES`].
    pub pose: u32,
    pub target: Vec3,
    pub heading: f32,
    pub duration: f32,
    pub speed: f32,
    pub arrival_heading: Option<f32>,
    pub deceleration: f32,
    /// How fast a walk gains speed, m/s² (infinite: at full speed in its
    /// first tick, as goals saved before MOTION5 did).
    pub acceleration: f32,
    /// A jump's apex over its takeoff, m (`height`), and the seconds before
    /// takeoff (`prepare`); its `duration` is the recovery after landing,
    /// `speed` its horizontal travel in m (`distance`) along `heading`.
    pub height: f32,
    pub prepare: f32,
}

/// Where a jump goal is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JumpStage {
    Prepare,
    Air,
    Landed,
}

/// A jump goal's state in the simulation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Jump {
    pub stage: JumpStage,
    /// The takeoff speed, m/s: the gravity's for the goal's height.
    pub v0: f32,
    /// The gravity it falls by, m/s² (positive).
    pub gravity: f32,
    /// The takeoff height (the arc without a controller returns to it).
    pub base: f32,
    /// Seconds since takeoff, and since landing.
    pub air: f32,
    pub landed: f32,
    /// The vertical speed, m/s: the fixed arc's, or the controller's.
    pub velocity: f32,
    /// A controller has been seen off the ground since takeoff.
    pub left: bool,
    /// How far the Transform stood above the ground under it at takeoff
    /// (a controller's), m: what its feet keep below it in the air.
    pub rest: Option<f32>,
    /// Seconds until it lands, as the tick predicts it: the fixed arc's
    /// remaining flight, or a controller's ballistic arc traced to the
    /// ground under it ([`landing_in`]). `None` when there is no ground to
    /// trace (no physics, nothing below).
    pub eta: Option<f32>,
}

/// The goal being carried out.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Current {
    pub goal: Goal,
    pub elapsed: f32,
    /// Where the Transform was, and its heading, when the goal started.
    pub start: Vec3,
    pub start_heading: f32,
    /// A jump goal's state.
    pub jump: Option<Jump>,
}

/// One agent's goals.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Agent {
    pub queue: VecDeque<Goal>,
    pub current: Option<Current>,
    pub completed: u32,
    /// The pose the character holds with no goal: the last finished goal's.
    pub hold: u32,
    /// Bumped whenever the current goal changes (a start, a finish, a clear):
    /// what the presenter watches to follow a new goal.
    pub serial: u32,
    pub performance: performance::Performance,
    /// The mover's speed over the ground, m/s: what the next walk or jump
    /// starts from. A walk into another walk or a jump keeps it through
    /// its target; a goal in place stops it.
    pub speed: f32,
}

/// The `og.motion` state: agents by entity bits (ordered, so saves are).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Goals {
    pub agents: BTreeMap<u64, Agent>,
}

thread_local! {
    static SHARED: RefCell<og_core::IdMap<u32, Arc<Mutex<Goals>>>> = const { RefCell::new(og_core::IdMap::new()) };
}

pub(crate) fn shared(engine: u32) -> Arc<Mutex<Goals>> {
    SHARED.with(|s| s.borrow_mut().entry(engine).or_default().clone())
}

pub(crate) fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn invalid(m: impl Into<String>) -> Error {
    Error::new(ErrorKind::Invalid, m.into())
}

// ── headings and the mover ────────────────────────────────────────────────────

/// A rotation's heading about +Y: 0 looks down +Z, π/2 down +X.
pub(crate) fn heading_of(q: Quat) -> f32 {
    let f = q * Vec3::Z;
    atan2(f.x, f.z)
}

/// The rotation of heading `h`.
pub(crate) fn rotation_of(h: f32) -> Quat {
    let (s, c) = sin_cos(h * 0.5);
    Quat::from_xyzw(0.0, s, 0.0, c)
}

/// The signed angle from `a` to `b`, in -π..π.
fn wrap(d: f32) -> f32 {
    let (s, c) = sin_cos(d);
    atan2(s, c)
}

/// `from` turned towards `to` by at most `step`.
fn turn(from: f32, to: f32, step: f32) -> f32 {
    let d = wrap(to - from);
    if d.abs() <= step {
        to
    } else {
        from + step * d.signum()
    }
}

/// What the mover reads of an agent's og.MotionAgent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Settings {
    pub drive: bool,
    pub controller: bool,
    /// The gravity a jump falls by (m/s², positive): the controller's
    /// (og.Physics gravity times its gravityScale), else og.Physics'.
    pub gravity: f32,
    /// The controller's grounded as the last physics step left it.
    pub grounded: bool,
    /// The controller's vertical speed as the last physics step left it.
    pub vertical: f32,
    pub turn_speed: f32,
    pub lookahead: f32,
    pub rig: u64,
}

impl Settings {
    pub(crate) fn of(v: &Value) -> Settings {
        Settings {
            drive: v["drive"].as_bool().unwrap_or(false),
            controller: false,
            gravity: GRAVITY,
            grounded: true,
            vertical: 0.0,
            turn_speed: v["turnSpeed"].as_f64().unwrap_or(6.0) as f32,
            lookahead: v["lookahead"].as_f64().unwrap_or(0.6) as f32,
            rig: v["rig"].as_f64().map_or(0, |x| x as u64),
        }
    }
}

/// A jump's speed across the ground in the air, m/s: its distance over
/// its flat flight.
fn jump_ground_speed(g: &Goal, v0: f32, gravity: f32) -> f32 {
    if gravity > 0.0 && v0 > 0.0 {
        g.speed * gravity / (2.0 * v0)
    } else {
        0.0
    }
}

/// The speed a walk `g` from `at` arrives at its target with: the speed
/// the next goal starts at. A walk on keeps the slower of the two speeds,
/// less by the corner it turns (none past a right angle); a jump takes
/// off at its flight's speed across the ground; anything else stops.
fn entry_speed(g: &Goal, at: &Vec3, next: Option<&Goal>, gravity: f32) -> f32 {
    let Some(n) = next else { return 0.0 };
    match n.kind {
        Kind::WalkTo => {
            let a = Vec3::new(g.target.x - at.x, 0.0, g.target.z - at.z).normalize_or_zero();
            let b = Vec3::new(n.target.x - g.target.x, 0.0, n.target.z - g.target.z)
                .normalize_or_zero();
            g.speed.min(n.speed) * a.dot(b).max(0.0)
        }
        Kind::Jump => {
            let v0 = (2.0 * gravity.max(0.0) * n.height).sqrt();
            g.speed.min(jump_ground_speed(n, v0, gravity.max(0.0)))
        }
        _ => 0.0,
    }
}

/// Advances `agent` by `dt` seconds: starts the next goal when there is
/// none, carries the current one out (moving `position` and `heading` when
/// `drive`), and finishes it by its rule. Fixed math only: the simulation's
/// mover, which the presenter also runs ahead of time to place the model's
/// goals.
pub(crate) fn advance(
    agent: &mut Agent,
    s: &Settings,
    position: &mut Vec3,
    heading: &mut f32,
    dt: f32,
) {
    agent.performance.advance(dt);
    if agent.current.is_none() {
        let Some(mut goal) = agent.queue.pop_front() else {
            return;
        };
        if goal.kind == Kind::Jump {
            // A jump travels its way, or the way it faces when it starts.
            goal.heading = goal.arrival_heading.unwrap_or(*heading);
        }
        if !matches!(goal.kind, Kind::WalkTo | Kind::Jump) {
            agent.speed = 0.0;
        }
        agent.current = Some(Current {
            goal,
            elapsed: 0.0,
            start: *position,
            start_heading: *heading,
            jump: (goal.kind == Kind::Jump).then(|| Jump {
                stage: JumpStage::Prepare,
                v0: (2.0 * s.gravity.max(0.0) * goal.height).sqrt(),
                gravity: s.gravity.max(0.0),
                base: position.y,
                air: 0.0,
                landed: 0.0,
                velocity: 0.0,
                left: false,
                rest: None,
                eta: None,
            }),
        });
        agent.serial = agent.serial.wrapping_add(1);
    }
    let Some(cur) = agent.current.as_mut() else {
        return;
    };
    cur.elapsed += dt;
    let g = cur.goal;
    let step = s.turn_speed * dt;
    let done = match g.kind {
        Kind::Pose => cur.elapsed >= g.duration,
        Kind::Face => {
            if s.drive {
                *heading = turn(*heading, g.heading, step);
            }
            cur.elapsed >= g.duration && (!s.drive || *heading == g.heading)
        }
        Kind::Place => {
            if s.drive {
                let k = if g.duration > 0.0 {
                    (cur.elapsed / g.duration).min(1.0)
                } else {
                    1.0
                };
                let to = Vec3::new(g.target.x, position.y, g.target.z);
                let from = Vec3::new(cur.start.x, position.y, cur.start.z);
                *position = from + (to - from) * k;
                *heading = turn(*heading, g.heading, step);
            }
            cur.elapsed >= g.duration && (!s.drive || *heading == g.heading)
        }
        Kind::Jump => {
            let j = cur.jump.as_mut().expect("a jump goal carries its state");
            match j.stage {
                JumpStage::Prepare => {
                    if s.drive {
                        *heading = turn(*heading, g.heading, step);
                        // A running jump's preparation keeps running: the
                        // approach speed eases to the flight's across the
                        // ground, along the jump's heading.
                        let to = jump_ground_speed(&g, j.v0, j.gravity);
                        let k = (dt / g.prepare.max(dt)).min(1.0);
                        agent.speed += (to.min(agent.speed) - agent.speed) * k;
                        let (sx, cz) = sin_cos(g.heading);
                        *position += Vec3::new(sx, 0.0, cz) * (agent.speed * dt);
                    }
                    // Takeoff on the tick the preparation ends; the arc
                    // (or the controller's jump) starts with the next.
                    if cur.elapsed >= g.prepare {
                        j.stage = JumpStage::Air;
                        j.base = position.y;
                        j.velocity = j.v0;
                    }
                }
                JumpStage::Air => {
                    j.air += dt;
                    let flight = if j.gravity > 0.0 {
                        2.0 * j.v0 / j.gravity
                    } else {
                        0.0
                    };
                    let landed = if s.controller {
                        j.left |= !s.grounded;
                        j.velocity = s.vertical;
                        s.grounded && (j.left || j.air >= STUCK)
                    } else {
                        let t = j.air.min(flight);
                        j.velocity = j.v0 - j.gravity * t;
                        j.eta = Some(flight - t);
                        if s.drive {
                            position.y = j.base + j.v0 * t - 0.5 * j.gravity * t * t;
                        }
                        j.air >= flight
                    };
                    if s.drive && flight > 0.0 {
                        // Across the ground at the flat flight's speed: the
                        // fixed arc covers exactly the distance, a
                        // controller keeps going until it lands.
                        let moved = if s.controller {
                            dt
                        } else {
                            j.air.min(flight) - (j.air - dt).min(flight)
                        };
                        let (sx, cz) = sin_cos(g.heading);
                        *position += Vec3::new(sx, 0.0, cz) * (g.speed / flight * moved);
                    }
                    if landed {
                        if !s.controller && s.drive {
                            position.y = j.base;
                        }
                        j.stage = JumpStage::Landed;
                        j.velocity = 0.0;
                        agent.speed = 0.0;
                        j.eta = Some(0.0);
                    }
                }
                JumpStage::Landed => j.landed += dt,
            }
            j.stage == JumpStage::Landed && j.landed >= g.duration
        }
        Kind::WalkTo => {
            let to = Vec3::new(g.target.x - position.x, 0.0, g.target.z - position.z);
            let left = to.length();
            if left > ARRIVED {
                *heading = turn(*heading, atan2(to.x, to.z), step);
                // Braking to the speed the next goal starts at: solve
                // d = v*dt, v² = v_end² + 2*a*(left-d), so braking cannot
                // overshoot, even on a long tick. Speed is gained at the
                // goal's acceleration.
                let end = entry_speed(&g, position, agent.queue.front(), s.gravity);
                let braked = if g.deceleration > 0.0 {
                    let adt = g.deceleration * dt;
                    (adt * adt + end * end + 2.0 * g.deceleration * left).sqrt() - adt
                } else {
                    g.speed
                };
                let speed = braked
                    .min(g.speed)
                    .min(agent.speed + g.acceleration * dt)
                    .max(0.0);
                agent.speed = speed;
                let d = (speed * dt).min(left);
                *position += to * (d / left);
            }
            let to = Vec3::new(g.target.x - position.x, 0.0, g.target.z - position.z);
            if (if s.controller { left } else { to.length() }) <= ARRIVED {
                if let Some(arrival) = g.arrival_heading {
                    *heading = turn(*heading, arrival, step);
                    *heading == arrival
                } else {
                    true
                }
            } else {
                false
            }
        }
    };
    if done {
        if g.kind == Kind::WalkTo {
            position.x = g.target.x;
            position.z = g.target.z;
        }
        if g.kind != Kind::Face {
            agent.hold = g.pose;
        }
        agent.current = None;
        agent.completed = agent.completed.wrapping_add(1);
        agent.serial = agent.serial.wrapping_add(1);
    }
}

// ── the system ───────────────────────────────────────────────────────────────

/// The step at which [`landing_in`] samples an arc, and how far ahead, s.
const LANDING_STEP: f32 = 1.0 / 30.0;
const LANDING_HORIZON: f32 = 3.0;

/// The height of the ground under `at` (the first collider straight down
/// that is not `e`'s), m.
fn ground(q: &og_physics::PhysicsQuery, e: Entity, at: Vec3) -> Option<f32> {
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
fn landing_in(
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

/// Registers og.MotionAgent's module on a world that names it (`super::lazy`):
/// the `og.motion` system, after every system scheduled so far, and its
/// state block.
pub(crate) fn enable(world: &mut World, engine_id: u32) -> og_core::Result<()> {
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
    let sh = shared(engine_id);
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

fn system(
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
        let mut s = Settings::of(&agent.1.decode(&ctx.get_bytes(e, agent.0)?));
        let cb = ctx.get_bytes(e, controller.0).ok();
        s.controller = cb.is_some();
        s.gravity = gravity;
        let c0 = cb.as_ref().map(|b| controller.1.decode(b));
        if let Some(c) = &c0 {
            s.gravity = gravity * c["gravityScale"].as_f64().unwrap_or(1.0) as f32;
            s.grounded = c["grounded"].as_bool().unwrap_or(false);
            s.vertical = c["velocity"]["y"].as_f64().unwrap_or(0.0) as f32;
        }
        let tb = ctx.get_bytes(e, transform.0)?;
        let mut t = transform.1.decode(&tb);
        let (mut position, q) = transform_of(&t);
        let mut heading = heading_of(q);
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
            let r = rotation_of(heading);
            t["quaternion"] = json!({ "x": r.x, "y": r.y, "z": r.z, "w": r.w });
            let tb = transform.1.encode(Some(&tb), Some(&t)).map_err(invalid)?;
            ctx.set_bytes(e, transform.0, &tb)?;
        }
    }
    Ok(())
}

/// og.Transform's position and rotation.
pub(crate) fn transform_of(t: &Value) -> (Vec3, Quat) {
    let n = |v: &Value, k: &str, d: f64| v[k].as_f64().unwrap_or(d) as f32;
    let p = &t["position"];
    let q = &t["quaternion"];
    (
        Vec3::new(n(p, "x", 0.0), n(p, "y", 0.0), n(p, "z", 0.0)),
        Quat::from_xyzw(
            n(q, "x", 0.0),
            n(q, "y", 0.0),
            n(q, "z", 0.0),
            n(q, "w", 1.0),
        )
        .normalize(),
    )
}

// ── the state block ──────────────────────────────────────────────────────────

struct Block(Arc<Mutex<Goals>>);

fn save(g: &Goals, out: &mut Vec<u8>) {
    out.extend_from_slice(b"MOTION5\0");
    let u32_ = |out: &mut Vec<u8>, v: u32| out.extend_from_slice(&v.to_le_bytes());
    let f32_ = |out: &mut Vec<u8>, v: f32| out.extend_from_slice(&v.to_le_bytes());
    let goal = |out: &mut Vec<u8>, g: &Goal| {
        u32_(out, g.kind as u32);
        u32_(out, g.pose);
        for v in [
            g.target.x, g.target.y, g.target.z, g.heading, g.duration, g.speed,
        ] {
            f32_(out, v);
        }
        u32_(out, g.arrival_heading.is_some() as u32);
        f32_(out, g.arrival_heading.unwrap_or(0.0));
        f32_(out, g.deceleration);
        f32_(out, g.height);
        f32_(out, g.prepare);
        f32_(out, g.acceleration);
    };
    u32_(out, g.agents.len() as u32);
    for (e, a) in &g.agents {
        out.extend_from_slice(&e.to_le_bytes());
        for v in [a.completed, a.hold, a.serial, a.queue.len() as u32] {
            u32_(out, v);
        }
        f32_(out, a.speed);
        for q in &a.queue {
            goal(out, q);
        }
        u32_(out, a.current.is_some() as u32);
        if let Some(c) = &a.current {
            goal(out, &c.goal);
            for v in [c.elapsed, c.start.x, c.start.y, c.start.z, c.start_heading] {
                f32_(out, v);
            }
            u32_(out, c.jump.is_some() as u32);
            if let Some(j) = &c.jump {
                u32_(out, j.stage as u32);
                for v in [j.v0, j.gravity, j.base, j.air, j.landed, j.velocity] {
                    f32_(out, v);
                }
                u32_(out, j.left as u32);
                for v in [j.rest, j.eta] {
                    u32_(out, v.is_some() as u32);
                    f32_(out, v.unwrap_or(0.0));
                }
            }
        }
        let performance = serde_json::to_vec(&a.performance).expect("finite performance state");
        u32_(out, performance.len() as u32);
        out.extend_from_slice(&performance);
    }
}

fn load(bytes: &[u8]) -> og_core::Result<Goals> {
    // MOTION5 adds a walk's acceleration and the mover's speed (older
    // goals start at full speed, the mover from rest); MOTION4 added a
    // jump's ground and landing prediction to MOTION3's jumps; MOTION3
    // added jumps to MOTION2's goals; before MOTION2 there was no tag.
    let version = if bytes.starts_with(b"MOTION5\0") {
        5
    } else if bytes.starts_with(b"MOTION4\0") {
        4
    } else if bytes.starts_with(b"MOTION3\0") {
        3
    } else if bytes.starts_with(b"MOTION2\0") {
        2
    } else {
        1
    };
    let modern = version >= 2;
    let mut r = Reader(if modern { &bytes[8..] } else { bytes });
    let mut g = Goals::default();
    for _ in 0..r.u32()? {
        let e = r.u64()?;
        let mut a = Agent {
            completed: r.u32()?,
            hold: r.u32()?,
            serial: r.u32()?,
            ..Agent::default()
        };
        let queued = r.u32()?;
        if version >= 5 {
            a.speed = r.f32()?;
        }
        for _ in 0..queued {
            a.queue.push_back(r.goal(version)?);
        }
        if r.u32()? != 0 {
            let goal = r.goal(version)?;
            let mut current = Current {
                goal,
                elapsed: r.f32()?,
                start: Vec3::new(r.f32()?, r.f32()?, r.f32()?),
                start_heading: r.f32()?,
                jump: None,
            };
            if version >= 3 && r.u32()? != 0 {
                let stage = match r.u32()? {
                    0 => JumpStage::Prepare,
                    1 => JumpStage::Air,
                    2 => JumpStage::Landed,
                    _ => return Err(invalid("an unknown og.motion jump stage")),
                };
                current.jump = Some(Jump {
                    stage,
                    v0: r.f32()?,
                    gravity: r.f32()?,
                    base: r.f32()?,
                    air: r.f32()?,
                    landed: r.f32()?,
                    velocity: r.f32()?,
                    left: r.u32()? != 0,
                    rest: None,
                    eta: None,
                });
                if version >= 4
                    && let Some(j) = current.jump.as_mut()
                {
                    for v in [&mut j.rest, &mut j.eta] {
                        let some = r.u32()? != 0;
                        let x = r.f32()?;
                        *v = some.then_some(x);
                    }
                }
            }
            if current.goal.kind == Kind::Jump && current.jump.is_none() {
                return Err(invalid("an og.motion jump goal without its state"));
            }
            a.current = Some(current);
        }
        if modern {
            let n = r.u32()? as usize;
            let data =
                r.0.get(..n)
                    .ok_or_else(|| invalid("motion performance state is cut short"))?;
            a.performance = serde_json::from_slice(data).map_err(|e| invalid(e.to_string()))?;
            r.0 = &r.0[n..];
        }
        g.agents.insert(e, a);
    }
    if !r.0.is_empty() {
        return Err(invalid("the og.motion state has bytes left over"));
    }
    Ok(g)
}

struct Reader<'a>(&'a [u8]);

impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> og_core::Result<[u8; N]> {
        let (a, b) = self
            .0
            .split_first_chunk::<N>()
            .ok_or_else(|| invalid("the og.motion state is cut short"))?;
        self.0 = b;
        Ok(*a)
    }
    fn u32(&mut self) -> og_core::Result<u32> {
        self.take::<4>().map(u32::from_le_bytes)
    }
    fn u64(&mut self) -> og_core::Result<u64> {
        self.take::<8>().map(u64::from_le_bytes)
    }
    fn f32(&mut self) -> og_core::Result<f32> {
        self.take::<4>().map(f32::from_le_bytes)
    }
    fn goal(&mut self, version: u32) -> og_core::Result<Goal> {
        let kind =
            Kind::from_u32(self.u32()?).ok_or_else(|| invalid("an unknown og.motion goal"))?;
        let pose = self.u32()?;
        if pose as usize >= POSES.len() {
            return Err(invalid("an og.motion goal's pose is past the library"));
        }
        let mut goal = Goal {
            kind,
            pose,
            target: Vec3::new(self.f32()?, self.f32()?, self.f32()?),
            heading: self.f32()?,
            duration: self.f32()?,
            speed: self.f32()?,
            arrival_heading: None,
            deceleration: 0.0,
            acceleration: 0.0,
            height: 0.0,
            prepare: 0.0,
        };
        if version >= 2 {
            let has_heading = self.u32()? != 0;
            let heading = self.f32()?;
            goal.arrival_heading = has_heading.then_some(heading);
            goal.deceleration = self.f32()?;
        }
        if version >= 3 {
            goal.height = self.f32()?;
            goal.prepare = self.f32()?;
        }
        goal.acceleration = if version >= 5 {
            self.f32()?
        } else {
            f32::INFINITY
        };
        Ok(goal)
    }
}

impl StateBlock for Block {
    fn save(&self, out: &mut Vec<u8>) {
        save(&lock(&self.0), out);
    }
    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn Any + Send>> {
        Ok(Box::new(load(bytes)?))
    }
    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        *lock(&self.0) = decoded
            .and_then(|d| d.downcast::<Goals>().ok())
            .map(|b| *b)
            .unwrap_or_default();
    }

    /// A place's or a walk's target and the pose a goal started from are
    /// in the agent's Transform frame: they move with a root.
    fn shift(&mut self, s: &og_ecs::Shift) {
        let mut goals = lock(&self.0);
        for (bits, a) in goals.agents.iter_mut() {
            let d = s.of(*bits);
            let m = |p: &mut Vec3| *p = (p.as_dvec3() + d).as_vec3();
            a.performance.shift(d);
            for g in &mut a.queue {
                if matches!(g.kind, Kind::Place | Kind::WalkTo) {
                    m(&mut g.target);
                }
            }
            if let Some(c) = &mut a.current {
                if matches!(c.goal.kind, Kind::Place | Kind::WalkTo) {
                    m(&mut c.goal.target);
                }
                m(&mut c.start);
            }
        }
    }
}

// ── the API ──────────────────────────────────────────────────────────────────

fn wire_void(r: Result<(), String>) -> String {
    match r {
        Ok(()) => ok_void(),
        Err(e) => refuse(e),
    }
}

/// The entity's og.MotionAgent settings, refused for an entity without one.
fn agent_of(eng: &Engine, e: Entity) -> Result<Settings, String> {
    let w = eng.world()?;
    if !w.is_alive(e) {
        return Err(format!("entity {} is not alive", e.to_bits()));
    }
    let no = || format!("entity {} has no og.MotionAgent", e.to_bits());
    let id = w.id_by_name(AGENT).map_err(|_| no())?;
    let bytes = w.get_bytes(e, id).map_err(|_| no())?;
    let layout = eng
        .state
        .borrow()
        .layouts
        .get(AGENT)
        .cloned()
        .ok_or("og.MotionAgent has no layout")?;
    Ok(Settings::of(&layout.decode(&bytes)))
}

fn pose_named(desc: &Value) -> Result<u32, String> {
    let name = desc["pose"]
        .as_str()
        .filter(|s| !s.is_empty())
        .unwrap_or("idle");
    pose_index(name).map(|i| i as u32).ok_or_else(|| {
        format!(
            "'{name}' is not a pose of the motion library: {}",
            POSES.join(", ")
        )
    })
}

fn finite(v: &Value, what: &str) -> Result<Vec3, String> {
    let n = |k: &str| v[k].as_f64().unwrap_or(0.0) as f32;
    let p = Vec3::new(n("x"), n("y"), n("z"));
    if p.is_finite() {
        Ok(p)
    } else {
        Err(format!("{what} is not finite"))
    }
}

fn number(desc: &Value, key: &str, default: f32) -> f32 {
    desc[key].as_f64().map_or(default, |v| v as f32)
}

/// Queues a goal made from the parsed desc (and the agent's settings).
fn queue(
    world: u32,
    entity: f64,
    desc: Option<String>,
    make: impl FnOnce(&Value, &Settings) -> Result<Goal, String>,
) -> String {
    wire_void((|| {
        let eng = engine(world)?;
        let e = Entity::from_bits(entity as u64);
        let s = agent_of(&eng, e)?;
        let desc = parse(desc.as_deref())?;
        let goal = make(&desc, &s)?;
        if !goal.heading.is_finite() || !goal.duration.is_finite() || goal.duration < 0.0 {
            return Err(
                "the goal's heading or duration is not a finite, non-negative number".into(),
            );
        }
        lock(&shared(world))
            .agents
            .entry(e.to_bits())
            .or_default()
            .queue
            .push_back(goal);
        Ok(())
    })())
}

fn need_drive(s: &Settings, what: &str) -> Result<(), String> {
    if s.drive {
        Ok(())
    } else {
        Err(format!(
            "og.motion.{what} moves the character, and its og.MotionAgent.drive is off: turn drive on, or move og.Transform yourself and queue og.motion.pose"
        ))
    }
}

pub fn og_motion_pose(world: u32, entity: f64, desc: Option<String>) -> String {
    queue(world, entity, desc, |d, _| {
        Ok(Goal {
            kind: Kind::Pose,
            pose: pose_named(d)?,
            target: Vec3::ZERO,
            heading: 0.0,
            duration: number(d, "duration", 1.5),
            speed: 0.0,
            arrival_heading: None,
            deceleration: 0.0,
            acceleration: 0.0,
            height: 0.0,
            prepare: 0.0,
        })
    })
}

pub fn og_motion_place(world: u32, entity: f64, desc: Option<String>) -> String {
    queue(world, entity, desc, |d, s| {
        need_drive(s, "place")?;
        Ok(Goal {
            kind: Kind::Place,
            pose: pose_named(d)?,
            target: finite(&d["position"], "desc.position")?,
            heading: number(d, "heading", 0.0),
            duration: number(d, "duration", 1.5),
            speed: 0.0,
            arrival_heading: None,
            deceleration: 0.0,
            acceleration: 0.0,
            height: 0.0,
            prepare: 0.0,
        })
    })
}

pub fn og_motion_face(world: u32, entity: f64, desc: Option<String>) -> String {
    queue(world, entity, desc, |d, s| {
        need_drive(s, "face")?;
        Ok(Goal {
            kind: Kind::Face,
            pose: 0,
            target: Vec3::ZERO,
            heading: number(d, "heading", 0.0),
            duration: number(d, "duration", 0.8),
            speed: 0.0,
            arrival_heading: None,
            deceleration: 0.0,
            acceleration: 0.0,
            height: 0.0,
            prepare: 0.0,
        })
    })
}

pub fn og_motion_walk_to(world: u32, entity: f64, desc: Option<String>) -> String {
    queue(world, entity, desc, |d, s| {
        need_drive(s, "walkTo")?;
        let speed = number(d, "speed", 1.4);
        if !(speed.is_finite() && speed > 0.0) {
            return Err(format!("desc.speed {speed} is not a positive speed"));
        }
        let arrival_heading = d["arrivalHeading"].as_f64().map(|h| h as f32);
        let deceleration = number(d, "deceleration", 3.0);
        let acceleration = number(d, "acceleration", 0.0);
        if arrival_heading.is_some_and(|h| !h.is_finite())
            || !deceleration.is_finite()
            || deceleration < 0.0
            || !acceleration.is_finite()
            || acceleration < 0.0
        {
            return Err(
                "arrivalHeading must be finite, deceleration and acceleration non-negative".into(),
            );
        }
        Ok(Goal {
            kind: Kind::WalkTo,
            pose: pose_named(d)?,
            target: finite(&d["position"], "desc.position")?,
            heading: 0.0,
            duration: 0.0,
            speed: speed.min(10.0),
            arrival_heading,
            deceleration: if deceleration == 0.0 {
                3.0
            } else {
                deceleration
            },
            acceleration: if acceleration == 0.0 {
                ACCELERATION
            } else {
                acceleration
            },
            height: 0.0,
            prepare: 0.0,
        })
    })
}

pub fn og_motion_jump(world: u32, entity: f64, desc: Option<String>) -> String {
    queue(world, entity, desc, |d, s| {
        need_drive(s, "jump")?;
        let height = number(d, "height", 0.5);
        let distance = number(d, "distance", 0.0);
        let prepare = number(d, "prepare", 0.25);
        let recover = number(d, "recover", 0.35);
        let heading = d["heading"].as_f64().map(|h| h as f32);
        if !(height.is_finite() && (0.0..=5.0).contains(&height))
            || !(distance.is_finite() && (0.0..=20.0).contains(&distance))
            || !(prepare.is_finite() && prepare >= 0.0)
            || !(recover.is_finite() && recover >= 0.0)
            || heading.is_some_and(|h| !h.is_finite())
        {
            return Err(
                "a jump's height (0 to 5 m), distance (0 to 20 m), prepare and recover (seconds, non-negative) and heading must be finite".into(),
            );
        }
        Ok(Goal {
            kind: Kind::Jump,
            pose: pose_named(d)?,
            target: Vec3::ZERO,
            heading: 0.0,
            duration: recover,
            speed: distance,
            arrival_heading: heading,
            deceleration: 0.0,
            acceleration: 0.0,
            height,
            prepare,
        })
    })
}

pub fn og_motion_clear(world: u32, entity: f64) -> String {
    wire_void((|| {
        let eng = engine(world)?;
        let e = Entity::from_bits(entity as u64);
        agent_of(&eng, e)?;
        if let Some(a) = lock(&shared(world)).agents.get_mut(&e.to_bits()) {
            a.queue.clear();
            a.speed = 0.0;
            if let Some(c) = a.current.take() {
                if c.goal.kind != Kind::Face {
                    a.hold = c.goal.pose;
                }
                a.serial = a.serial.wrapping_add(1);
            }
        }
        Ok(())
    })())
}

pub fn og_motion_status(world: u32, entity: f64) -> String {
    let r = (|| {
        let eng = engine(world)?;
        let e = Entity::from_bits(entity as u64);
        agent_of(&eng, e)?;
        let goals = shared(world);
        let goals = lock(&goals);
        let a = goals.agents.get(&e.to_bits()).cloned().unwrap_or_default();
        let v3 = |p: Vec3| json!({ "x": p.x, "y": p.y, "z": p.z });
        let mut status = match a.current {
            None => json!({
                "goal": "none", "pose": "", "target": v3(Vec3::ZERO), "heading": 0.0,
                "elapsed": 0.0, "queued": a.queue.len(), "completed": a.completed,
            }),
            Some(c) => {
                let g = c.goal;
                let (target, heading) = match g.kind {
                    Kind::Pose => (c.start, c.start_heading),
                    Kind::Face => (c.start, g.heading),
                    Kind::Place => (g.target, g.heading),
                    Kind::WalkTo => {
                        let d = g.target - c.start;
                        (g.target, atan2(d.x, d.z))
                    }
                    Kind::Jump => {
                        let (sx, cz) = sin_cos(g.heading);
                        (c.start + Vec3::new(sx, 0.0, cz) * g.speed, g.heading)
                    }
                };
                json!({
                    "goal": KIND_NAMES[g.kind as usize],
                    "pose": if g.kind == Kind::Face { "" } else { POSES[g.pose as usize] },
                    "target": v3(target),
                    "heading": heading,
                    "elapsed": c.elapsed,
                    "queued": a.queue.len(),
                    "completed": a.completed,
                })
            }
        };
        status["airborne"] = json!(
            a.current
                .and_then(|c| c.jump)
                .is_some_and(|j| j.stage == JumpStage::Air)
        );
        status["looking"] = json!(a.performance.looking());
        status["reacting"] = json!(a.performance.reacting());
        Ok::<_, String>(status)
    })();
    match r {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

pub fn og_motion_presentation(world: u32, entity: f64) -> String {
    let r = (|| {
        let eng = engine(world)?;
        let e = Entity::from_bits(entity as u64);
        agent_of(&eng, e)?;
        Ok::<_, String>(present::presentation(&eng, e.to_bits()))
    })();
    match r {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

pub fn og_motion_capabilities(world: u32) -> String {
    match engine(world) {
        Ok(eng) => present::capabilities(&eng).to_string(),
        Err(e) => json!({ "backend": "none", "reason": e, "modelBytes": 0 }).to_string(),
    }
}

pub fn og_motion_set_backend(world: u32, backend: String) -> String {
    let r = (|| {
        let eng = engine(world)?;
        let gpu = match backend.as_str() {
            "cpu" => false,
            "gpu" => true,
            other => return Err(format!("backend is {other:?}: cpu or gpu")),
        };
        present::set_backend(&eng, gpu);
        Ok::<_, String>(())
    })();
    match r {
        Ok(()) => ok(Value::Null),
        Err(e) => refuse(e),
    }
}

pub fn og_motion_poses() -> String {
    json!(POSES).to_string()
}

/// The probe's `dump_motion`: every agent's goals (the simulation's) and
/// what the model is doing for it (the presenter's: frames generated
/// ahead, the batches, their timings).
pub(crate) fn dump(engine_id: u32) -> Value {
    let eng = match engine(engine_id) {
        Ok(eng) => eng,
        Err(e) => return json!({ "error": e }),
    };
    let goals = lock(&shared(engine_id)).clone();
    let entities = eng
        .world()
        .ok()
        .and_then(|world| {
            let state = eng.state.borrow();
            let layout = state.layouts.get(AGENT)?;
            Some(crate::extract::rows(world, layout))
        })
        .unwrap_or_default();
    let agents: Vec<Value> = entities
        .into_iter()
        .map(|(e, _)| {
            let a = goals.agents.get(&e).cloned().unwrap_or_default();
            json!({
                "entity": e as f64,
                "presentation": present::presentation(&eng, e),
                "goal": a.current.map_or("none", |c| KIND_NAMES[c.goal.kind as usize]),
                "elapsed": a.current.map_or(0.0, |c| c.elapsed),
                "queued": a.queue.len(),
                "completed": a.completed,
                "hold": POSES[a.hold as usize],
                // A jump's state: its stage, seconds in the air and the
                // predicted seconds to its landing.
                "jump": a.current.and_then(|c| c.jump).map(|j| json!({
                    "stage": (["prepare", "air", "landed"])[j.stage as usize],
                    "air": j.air,
                    "landingIn": j.eta,
                })),
            })
        })
        .collect();
    let model = present::dump(&eng);
    json!({ "agents": agents, "model": model })
}

pub(crate) fn register_commands(probe: &mut og_probe::Probe, engine: u32) {
    let _ = probe.register_command(
        "motion_backend",
        og_probe::CommandDesc {
            usage: "motion_backend:<cpu|gpu>".into(),
            doc: "og.motion.setBackend from the probe: cpu (the default, the same poses on every host) or gpu (the opt-in, the same poses only on one host and GPU)".into(),
        },
        move |_world, args| {
            let eng = super::engine::engine(engine)?;
            match args.first().map(String::as_str) {
                Some("cpu") => present::set_backend(&eng, false),
                Some("gpu") => present::set_backend(&eng, true),
                _ => return Err("write motion_backend:cpu or motion_backend:gpu".into()),
            }
            Ok(())
        },
    );
    let _ = probe.register_dump(
        "dump_motion",
        og_probe::CommandDesc {
            usage: "dump_motion[:path]".into(),
            doc: "og.motion: every agent's goals as the simulation keeps them, and the motion model's side (presentation only): whether its weights are resident and on which backend, each agent's frames generated ahead and shown, the batches run and their GPU times".into(),
        },
        0,
        move |_world, _args| Ok(dump(engine)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn walk(acceleration: f32) -> Goal {
        Goal {
            kind: Kind::WalkTo,
            pose: 0,
            target: Vec3::new(2.0, 0.0, 0.0),
            heading: 0.0,
            duration: 0.0,
            speed: 1.4,
            arrival_heading: None,
            deceleration: 3.0,
            acceleration,
            height: 0.0,
            prepare: 0.0,
        }
    }

    #[test]
    fn a_motion4_save_restores_with_its_walks_at_full_speed_from_rest() {
        // MOTION4 by hand: one agent, one queued walk, no current goal.
        let mut b = b"MOTION4\0".to_vec();
        let u = |b: &mut Vec<u8>, v: u32| b.extend_from_slice(&v.to_le_bytes());
        let f = |b: &mut Vec<u8>, v: f32| b.extend_from_slice(&v.to_le_bytes());
        u(&mut b, 1);
        b.extend_from_slice(&7u64.to_le_bytes());
        for v in [0, 0, 0, 1] {
            u(&mut b, v);
        }
        let g = walk(0.0);
        u(&mut b, g.kind as u32);
        u(&mut b, g.pose);
        for v in [2.0, 0.0, 0.0, 0.0, 0.0, 1.4] {
            f(&mut b, v);
        }
        u(&mut b, 0);
        f(&mut b, 0.0);
        f(&mut b, 3.0);
        f(&mut b, 0.0);
        f(&mut b, 0.0);
        u(&mut b, 0);
        let perf = serde_json::to_vec(&performance::Performance::default()).unwrap();
        u(&mut b, perf.len() as u32);
        b.extend_from_slice(&perf);
        let mut goals = load(&b).unwrap();
        let a = goals.agents.get_mut(&7).unwrap();
        assert_eq!(a.speed, 0.0);
        assert_eq!(a.queue[0], walk(f32::INFINITY));
        // As MOTION4 ran it: at full speed in the first tick.
        let s = Settings::of(&json!({ "drive": true }));
        let (mut p, mut h) = (Vec3::ZERO, std::f32::consts::FRAC_PI_2);
        advance(a, &s, &mut p, &mut h, 1.0 / 60.0);
        assert!((p.x - 1.4 / 60.0).abs() < 1e-6, "{}", p.x);

        // MOTION5 keeps a walk's acceleration and the mover's speed.
        a.queue.push_back(walk(ACCELERATION));
        let mut out = Vec::new();
        save(&goals, &mut out);
        assert!(out.starts_with(b"MOTION5\0"));
        assert_eq!(load(&out).unwrap(), goals);
    }
}
