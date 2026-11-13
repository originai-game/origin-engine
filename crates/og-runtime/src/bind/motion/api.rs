//! The og.motion calls.

use super::*;

/// The entity's og.MotionAgent settings, refused for an entity without one.
pub(super) fn agent_of(eng: &Engine, e: Entity) -> Result<Settings, String> {
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
    Ok(Settings::of(&layout.read(&bytes)?))
}

pub(super) fn pose_named(pose: &str) -> Result<u32, String> {
    let name = if pose.is_empty() { "idle" } else { pose };
    pose_index(name).map(|i| i as u32).ok_or_else(|| {
        format!(
            "'{name}' is not a pose of the motion library: {}",
            POSES.join(", ")
        )
    })
}

pub(super) fn finite(p: Vec3, what: &str) -> Result<Vec3, String> {
    if p.is_finite() {
        Ok(p)
    } else {
        Err(format!("{what} is not finite"))
    }
}

/// `key` of the desc `raw` when it is given (not absent or null): the
/// typed `value` read from it.
pub(super) fn given(raw: &Value, key: &str, value: f32) -> Option<f32> {
    (!raw[key].is_null()).then_some(value)
}

/// Queues a goal made from the desc, read as `T` (and its raw JSON, for
/// fields whose absence means something), and the agent's settings.
pub(super) fn queue<T: wire::FromJson + Default>(
    world: u32,
    entity: f64,
    desc: Option<String>,
    make: impl FnOnce(T, &Value, &Settings) -> Result<Goal, String>,
) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        let e = Entity::from_bits(entity as u64);
        let s = agent_of(&eng, e)?;
        let raw = parse(desc.as_deref())?;
        let goal = make(wire::value(&raw, "desc")?, &raw, &s)?;
        if !goal.heading.is_finite() || !goal.duration.is_finite() || goal.duration < 0.0 {
            return Err(
                "the goal's heading or duration is not a finite, non-negative number".into(),
            );
        }
        lock(&shared(&eng))
            .agents
            .entry(e.to_bits())
            .or_default()
            .queue
            .push_back(goal);
        Ok(())
    })()
}

pub(super) fn need_drive(s: &Settings, what: &str) -> Result<(), String> {
    if s.drive {
        Ok(())
    } else {
        Err(format!(
            "og.motion.{what} moves the character, and its og.MotionAgent.drive is off: turn drive on, or move og.Transform yourself and queue og.motion.pose"
        ))
    }
}

pub fn og_motion_pose(world: u32, entity: f64, desc: Option<String>) -> Result<(), String> {
    queue(world, entity, desc, |d: PoseGoalDesc, _, _| {
        Ok(Goal {
            kind: Kind::Pose,
            pose: pose_named(&d.pose)?,
            target: Vec3::ZERO,
            heading: 0.0,
            duration: d.duration,
            speed: 0.0,
            arrival_heading: None,
            deceleration: 0.0,
            acceleration: 0.0,
            height: 0.0,
            prepare: 0.0,
        })
    })
}

pub fn og_motion_place(world: u32, entity: f64, desc: Option<String>) -> Result<(), String> {
    queue(world, entity, desc, |d: PlaceGoalDesc, _, s| {
        need_drive(s, "place")?;
        Ok(Goal {
            kind: Kind::Place,
            pose: pose_named(&d.pose)?,
            target: finite(d.position, "desc.position")?,
            heading: d.heading,
            duration: d.duration,
            speed: 0.0,
            arrival_heading: None,
            deceleration: 0.0,
            acceleration: 0.0,
            height: 0.0,
            prepare: 0.0,
        })
    })
}

pub fn og_motion_face(world: u32, entity: f64, desc: Option<String>) -> Result<(), String> {
    queue(world, entity, desc, |d: FaceGoalDesc, _, s| {
        need_drive(s, "face")?;
        Ok(Goal {
            kind: Kind::Face,
            pose: 0,
            target: Vec3::ZERO,
            heading: d.heading,
            duration: d.duration,
            speed: 0.0,
            arrival_heading: None,
            deceleration: 0.0,
            acceleration: 0.0,
            height: 0.0,
            prepare: 0.0,
        })
    })
}

pub fn og_motion_walk_to(world: u32, entity: f64, desc: Option<String>) -> Result<(), String> {
    queue(world, entity, desc, |d: WalkGoalDesc, raw, s| {
        need_drive(s, "walkTo")?;
        let speed = d.speed;
        if !(speed.is_finite() && speed > 0.0) {
            return Err(format!("desc.speed {speed} is not a positive speed"));
        }
        let arrival_heading = given(raw, "arrivalHeading", d.arrival_heading);
        let (deceleration, acceleration) = (d.deceleration, d.acceleration);
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
            pose: pose_named(&d.pose)?,
            target: finite(d.position, "desc.position")?,
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

pub fn og_motion_jump(world: u32, entity: f64, desc: Option<String>) -> Result<(), String> {
    queue(world, entity, desc, |d: JumpGoalDesc, raw, s| {
        need_drive(s, "jump")?;
        let (height, distance, prepare, recover) = (d.height, d.distance, d.prepare, d.recover);
        let heading = given(raw, "heading", d.heading);
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
            pose: pose_named(&d.pose)?,
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

pub fn og_motion_clear(world: u32, entity: f64) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        let e = Entity::from_bits(entity as u64);
        agent_of(&eng, e)?;
        if let Some(a) = lock(&shared(&eng)).agents.get_mut(&e.to_bits()) {
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
    })()
}

pub fn og_motion_status(
    world: u32,
    entity: f64,
) -> Result<crate::host::wire::MotionStatus, String> {
    (|| {
        let eng = engine(world)?;
        let e = Entity::from_bits(entity as u64);
        agent_of(&eng, e)?;
        let goals = shared(&eng);
        let goals = lock(&goals);
        let a = goals.agents.get(&e.to_bits()).cloned().unwrap_or_default();
        let (goal, pose, target, heading, elapsed) = match a.current {
            None => (MotionGoalKind::None, "", Vec3::ZERO, 0.0, 0.0),
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
                (
                    MotionGoalKind::ALL[g.kind as usize],
                    if g.kind == Kind::Face {
                        ""
                    } else {
                        POSES[g.pose as usize]
                    },
                    target,
                    heading,
                    c.elapsed,
                )
            }
        };
        Ok::<_, String>(MotionStatus {
            goal,
            pose: pose.into(),
            target,
            heading,
            elapsed,
            airborne: a
                .current
                .and_then(|c| c.jump)
                .is_some_and(|j| j.stage == JumpStage::Air),
            queued: a.queue.len() as u32,
            completed: a.completed,
            looking: a.performance.looking(),
            reacting: a.performance.reacting(),
        })
    })()
}

pub fn og_motion_presentation(world: u32, entity: f64) -> Result<serde_json::Value, String> {
    (|| {
        let eng = engine(world)?;
        let e = Entity::from_bits(entity as u64);
        agent_of(&eng, e)?;
        // Game logic never reads og.motion's presentation: on the GPU its
        // results arrive when the GPU answers, so a system that branches
        // on it differs run to run. A development build says so.
        #[cfg(not(feature = "publish"))]
        if crate::engine::running(world).is_some() && present::backend(&eng) == "gpu" {
            present::warn_read(&eng);
        }
        Ok::<_, String>(present::presentation(&eng, e.to_bits()))
    })()
}

pub fn og_motion_capabilities(world: u32) -> Value {
    match engine(world) {
        Ok(eng) => present::capabilities(&eng),
        Err(e) => MotionCapabilities {
            backend: MotionBackend::None,
            reason: e,
            model_bytes: 0.0,
        }
        .to_json(),
    }
}

pub fn og_motion_set_backend(world: u32, backend: String) -> Result<serde_json::Value, String> {
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
        Ok(()) => Ok(Value::Null),
        Err(e) => Err(e),
    }
}

pub fn og_motion_poses() -> Value {
    json!(POSES)
}
