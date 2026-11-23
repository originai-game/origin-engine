//! The goals' state block: saved and restored with the world.

use super::*;
use og_core::block_form::{self as bf, BlockForm, Field, Record};
use og_core::bytes::ByteReader;
use serde_json::json;

pub(super) struct Block(pub(super) Arc<Mutex<Goals>>);

pub(super) fn save(g: &Goals, out: &mut Vec<u8>) {
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

pub(super) fn load(bytes: &[u8]) -> og_core::Result<Goals> {
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
    let mut r = ByteReader::new(
        if modern { &bytes[8..] } else { bytes },
        "the og.motion state",
    );
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
            let data = r.sized()?;
            a.performance = serde_json::from_slice(data).map_err(|e| invalid(e.to_string()))?;
        }
        g.agents.insert(e, a);
    }
    if !r.is_done() {
        return Err(invalid("the og.motion state has bytes left over"));
    }
    Ok(g)
}

/// The og.motion block's own values on og-core's reader.
trait Reader {
    fn goal(&mut self, version: u32) -> og_core::Result<Goal>;
}

impl Reader for ByteReader<'_> {
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

    /// One `agent` record per entity: its counters, speed, queue, current
    /// goal and performance (look and reaction), none naming an entity.
    fn form(&self, bytes: &[u8]) -> og_core::Result<BlockForm> {
        let g = load(bytes)?;
        let goal = |g: &Goal| {
            json!({
                "kind": KIND_NAMES[g.kind as usize], "pose": g.pose,
                "target": g.target.to_array(), "heading": g.heading,
                "duration": g.duration, "speed": g.speed,
                "arrivalHeading": g.arrival_heading, "deceleration": g.deceleration,
                "acceleration": g.acceleration, "height": g.height, "prepare": g.prepare,
            })
        };
        let mut out = bf::Records::default();
        for (&e, a) in &g.agents {
            let current = a.current.as_ref().map(|c| {
                json!({
                    "goal": goal(&c.goal), "elapsed": c.elapsed,
                    "start": c.start.to_array(), "startHeading": c.start_heading,
                    "jump": c.jump.as_ref().map(|j| json!({
                        "stage": format!("{:?}", j.stage), "v0": j.v0,
                        "gravity": j.gravity, "base": j.base, "air": j.air,
                        "landed": j.landed, "velocity": j.velocity, "left": j.left,
                        "rest": j.rest, "eta": j.eta,
                    })),
                })
            });
            let queue: Vec<_> = a.queue.iter().map(goal).collect();
            out.entities.push(
                Record::new(e, "agent")
                    .field("completed", Field::of(&a.completed))
                    .field("hold", Field::of(&a.hold))
                    .field("serial", Field::of(&a.serial))
                    .field("speed", Field::of(&a.speed))
                    .field("queue", Field::of(&queue))
                    .field("current", Field::of(&current))
                    .field("performance", Field::of(&a.performance)),
            );
        }
        Ok(BlockForm::Entities(out))
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
