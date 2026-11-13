//! What the world shows of each agent ([`Seen`]) and the goal the model is
//! asked for next ([`predict`]).

use super::*;

/// How far ahead of the simulation the model's next goal is placed.
pub(crate) const HORIZON: f32 = 1.2;
/// The shortest model goal, s: [`MIN_TOKENS`] tokens less the frames at
/// their boundaries ([`token_counts`]).
pub(crate) const MIN_SEGMENT: f32 =
    (MIN_TOKENS * POSES_PER_TOKEN - 2 * BOUNDARY_FRAMES - 1) as f32 / RATE;

/// What an agent is doing, as [`Footing`] counts it: a walk or a run (by
/// the speed it asks for), a turn, a jump's takeoff, flight or landing,
/// or holding a pose.
pub(crate) fn segment_of(agent: Option<&super::Agent>, drive: bool) -> &'static str {
    let Some(c) = agent.and_then(|g| g.current) else {
        return "hold";
    };
    match c.goal.kind {
        Kind::WalkTo if c.goal.speed >= RUN_SPEED => "run",
        Kind::WalkTo => "walk",
        Kind::Face | Kind::Place if drive => "turn",
        Kind::Jump => match c.jump.map(|j| j.stage) {
            Some(JumpStage::Air) => "air",
            Some(JumpStage::Landed) => "landing",
            _ => "takeoff",
        },
        _ => "hold",
    }
}

/// One og.MotionAgent as this frame reads it.
#[derive(Clone)]
pub(crate) struct Seen {
    pub(crate) entity: u64,
    pub(crate) settings: Settings,
    pub(crate) position: Vec3,
    pub(crate) rotation: glam::Quat,
    pub(crate) scale: f32,
    pub(crate) skeleton: u64,
    pub(crate) skin: u32,
}

pub(crate) fn seen(eng: &Engine) -> Vec<Seen> {
    let Ok(world) = eng.world() else {
        return Vec::new();
    };
    let st = eng.state.borrow();
    let (Some(agent), Some(transform), Some(skeleton)) = (
        st.layouts.get(AGENT),
        st.layouts.get("og.Transform"),
        st.layouts.get("og.Skeleton"),
    ) else {
        return Vec::new();
    };
    let (Ok(tid), Ok(sid)) = (
        world.id_by_name("og.Transform"),
        world.id_by_name("og.Skeleton"),
    ) else {
        return Vec::new();
    };
    rows(world, agent)
        .into_iter()
        .filter_map(|(bits, a)| {
            let e = og_ecs::Entity::from_bits(bits);
            let t: record::Transform = transform.read(&world.get_bytes(e, tid).ok()?).ok()?;
            let s: record::Skeleton = skeleton.read(&world.get_bytes(e, sid).ok()?).ok()?;
            let (position, rotation) = transform_of(&t);
            Some(Seen {
                entity: bits,
                settings: Settings::of(&record::MotionAgent::from_json(&a).ok()?),
                position,
                rotation,
                scale: t.scale.x,
                skeleton: s.asset,
                skin: s.skin,
            })
        })
        .collect()
}

/// Whether library pose `pose` is one [`gait`] gives.
pub(crate) fn is_gait(pose: usize) -> bool {
    pose == gait(0.0) || pose == gait(RUN_SPEED)
}

/// The pose a moving character is in at `speed`.
pub(crate) fn gait(speed: f32) -> usize {
    if speed >= RUN_SPEED { 2 } else { 1 }
}

/// The library pose a driven agent is in, moving at `speed`: a place or
/// pose goal asks for its own pose even while the mover carries the
/// character there; only a walk is walked or run.
pub(crate) fn pose_of(agent: &super::Agent, speed: f32) -> usize {
    match agent.current {
        // A jump asks for the library's jump through the preparation and
        // the rise, so the model reaches it at the apex, then for the
        // jump's pose from the fall on.
        Some(c) if c.goal.kind == Kind::Jump => match c.jump {
            Some(j)
                if j.stage == JumpStage::Landed
                    || (j.stage == JumpStage::Air && j.velocity <= 0.0) =>
            {
                c.goal.pose as usize
            }
            _ => JUMP,
        },
        Some(c) if matches!(c.goal.kind, Kind::Place | Kind::Pose) => c.goal.pose as usize,
        // A walk is walked or run at the speed it asks for, through its
        // start and its braking: a run that brakes into a jump is a run.
        Some(c) if c.goal.kind == Kind::WalkTo && speed > 0.3 => gait(c.goal.speed),
        _ if speed > 0.3 => gait(speed),
        Some(c) if c.goal.kind == Kind::WalkTo => c.goal.pose as usize,
        _ => agent.hold as usize,
    }
}

/// The next model goal of an agent whose generated frames run `queued`
/// seconds past now: where and facing which way the simulation will have
/// it, the library pose it will be in, and how many seconds after those
/// frames that is. A driven agent's mover is run past the queued frames,
/// then on to where its pose first changes, at most [`HORIZON`]:
/// the model reaches each pose of the course in
/// turn (a run, then the jump at its landing) when the simulation does,
/// instead of skipping a short goal for the one after it.
pub(crate) fn predict(
    goals: &Goals,
    a: &Seen,
    shown: &Shown,
    queued: f32,
) -> (Vec3, f32, usize, f32) {
    let h0 = heading_of(a.rotation);
    match goals.agents.get(&a.entity) {
        Some(agent) if a.settings.drive => {
            let mut agent = agent.clone();
            let (mut p, mut h) = (a.position, h0);
            let step = 1.0 / RATE;
            for _ in 0..(queued * RATE) as usize {
                advance(&mut agent, &a.settings, &mut p, &mut h, step);
            }
            // The rest of the pose the last goal asked for (the walk or run
            // the model is in, or a held pose) merges into the next when
            // less than the shortest generation of it is left: asked for on
            // its own, the model spends that generation on it and falls
            // behind the simulation, and the goal after asks it to lunge. A
            // new pose keeps its goal however short, a change of gait (a
            // run between two walks) included, and a jump's rise keeps its
            // own: the model reaches the jump's pose at the apex.
            let last_asked = shown.issued.back().map(|i| i.pose);
            let carries_on = |pose: usize| pose != JUMP && last_asked == Some(pose);
            let mut last = None;
            for k in 0..(HORIZON * RATE) as usize {
                let (p0, h0) = (p, h);
                advance(&mut agent, &a.settings, &mut p, &mut h, step);
                let pose = pose_of(&agent, (p - p0).length() / step);
                if let Some(prev) = last
                    && pose != prev
                    && (k as f32 * step >= MIN_SEGMENT || !carries_on(prev))
                {
                    return (p0, h0, prev, k as f32 * step);
                }
                last = Some(pose);
            }
            (p, h, last.unwrap_or(agent.hold as usize), HORIZON)
        }
        agent => {
            let v = Vec3::new(shown.velocity.x, 0.0, shown.velocity.z);
            let speed = v.length();
            if speed > 0.3 {
                (
                    a.position + v * (queued + HORIZON),
                    og_core::math::atan2(v.x, v.z),
                    gait(speed),
                    HORIZON,
                )
            } else {
                let pose = agent.map_or(0, |g| match g.current {
                    Some(c) if c.goal.kind == Kind::Pose => c.goal.pose as usize,
                    _ => g.hold as usize,
                });
                (a.position, h0, pose, HORIZON)
            }
        }
    }
}

/// How far `v` leans from straight up, rad.
pub(crate) fn lean_of(v: Vec3) -> f32 {
    og_core::math::acos(v.normalize_or_zero().y.clamp(-1.0, 1.0))
}
