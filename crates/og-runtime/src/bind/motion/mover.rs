//! Headings and the mover: an agent's settings and one tick of its goal.

use super::*;

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
pub(super) fn wrap(d: f32) -> f32 {
    let (s, c) = sin_cos(d);
    atan2(s, c)
}

/// `from` turned towards `to` by at most `step`.
pub(super) fn turn(from: f32, to: f32, step: f32) -> f32 {
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
    pub(crate) fn of(v: &wire::record::MotionAgent) -> Settings {
        Settings {
            drive: v.drive,
            controller: false,
            gravity: GRAVITY,
            grounded: true,
            vertical: 0.0,
            turn_speed: v.turn_speed,
            lookahead: v.lookahead,
            rig: v.rig,
        }
    }
}

/// A jump's speed across the ground in the air, m/s: its distance over
/// its flat flight.
pub(super) fn jump_ground_speed(g: &Goal, v0: f32, gravity: f32) -> f32 {
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
pub(super) fn entry_speed(g: &Goal, at: &Vec3, next: Option<&Goal>, gravity: f32) -> f32 {
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
