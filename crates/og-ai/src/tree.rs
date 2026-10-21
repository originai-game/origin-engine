//! Ticking a behaviour: the tree's nodes or the machine's states, one
//! agent at a time, with that agent's memory. Movement is level-triggered:
//! the action that runs this tick says where to go ([`Motion`]); when none
//! does, the agent stops. Game actions are queued commands.

use crate::asset::{Action, Behaviour, Check, Kind, Op, Root, Speed, Value};
use og_core::glam::Vec3;
use std::collections::BTreeMap;

/// What a node answered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Running,
    Success,
    Failure,
}

impl Status {
    pub fn code(self) -> u32 {
        match self {
            Status::Running => 1,
            Status::Success => 2,
            Status::Failure => 3,
        }
    }
}

/// One node's memory.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NodeMem {
    pub running: bool,
    /// Steps: the child it resumes at. Patrol: the route point it heads for.
    pub index: u32,
    /// Repeat: times done.
    pub count: u32,
    /// The tick it started running.
    pub since: u64,
    /// Cooldowns and commands: the first tick it may run again. Patrol and wander: waiting until.
    pub until: u64,
    /// Wander: the point it heads for.
    pub point: Vec3,
}

/// An agent's memory of its behaviour.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AgentMem {
    /// The behaviour handle the memory is for.
    pub behaviour: u64,
    pub started: bool,
    /// The machine has run its initial state's `enter`.
    pub entered: bool,
    /// The machine's state.
    pub state: u32,
    pub state_since: u64,
    pub nodes: Vec<NodeMem>,
    /// Declared keys and keys set since.
    pub keys: BTreeMap<String, Value>,
    /// Where the agent was when it started (wander's centre).
    pub home: Vec3,
    /// NavAgent.speed when it started: what actions without a speed move at.
    pub base_speed: f32,
    /// The node it was running at the end of the last think, and the status.
    pub leaf: u32,
    pub status: u32,
    /// The path to `leaf`, root first.
    pub path: Vec<u32>,
}

/// Where the running action wants the agent this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Motion {
    pub target: Option<Vec3>,
    pub speed: Option<f32>,
}

/// What a think can see of the world.
pub struct Senses<'a> {
    pub plane: og_nav::Plane,
    pub tick: u64,
    pub rate: f32,
    pub entity: u64,
    pub position: Vec3,
    /// The NavAgent's last status, and the target it was for.
    pub nav_status: u32,
    pub nav_target: Vec3,
    pub stop_distance: f32,
    /// A key, `Component.field` or `Resource.field` (built-in keys included).
    pub resolve: &'a dyn Fn(&str, &AgentMem) -> Option<Value>,
    /// Where an entity is.
    pub locate: &'a dyn Fn(u64) -> Option<Vec3>,
}

/// What a think does besides moving: commands to run, warnings to log.
#[derive(Default, Debug)]
pub struct Effects {
    pub motion: Motion,
    pub commands: Vec<(String, Vec<String>)>,
    pub warnings: Vec<String>,
}

fn hash3(a: u64, b: u64, c: u64) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for x in [a, b, c] {
        for byte in x.to_le_bytes() {
            h ^= byte as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h
}

/// Starts an agent's memory for a behaviour.
pub fn start(b: &Behaviour, mem: &mut AgentMem, position: Vec3, speed: f32, tick: u64) {
    mem.started = true;
    mem.nodes = vec![NodeMem::default(); b.nodes.len()];
    for (k, v) in &b.keys {
        mem.keys.entry(k.clone()).or_insert(*v);
    }
    mem.home = position;
    mem.base_speed = speed;
    mem.state_since = tick;
    if let Root::Machine { initial, .. } = &b.root {
        mem.state = *initial as u32;
    }
}

fn reset(b: &Behaviour, mem: &mut AgentMem, i: usize) {
    let n = &mut mem.nodes[i];
    n.running = false;
    n.index = 0;
    n.count = 0;
    for c in b.nodes[i].children.clone() {
        reset(b, mem, c);
    }
}

pub fn check(
    c: &Check,
    s: &Senses,
    mem: &AgentMem,
    fx: &mut Effects,
    line: usize,
    name: &str,
) -> bool {
    let missing = |fx: &mut Effects, what: &str| {
        fx.warnings.push(format!("behaviour '{name}' line {line}: no key or field '{what}' (a key of the behaviour, Component.field of the agent or Resource.field); the check fails"));
    };
    // The value compared with: the literal, or the key or field read now.
    let value = match &c.against {
        None => c.value,
        Some(r) => match (s.resolve)(r, mem) {
            Some(v) => v,
            None => {
                missing(fx, r);
                return false;
            }
        },
    };
    match (s.resolve)(&c.operand, mem) {
        Some(v) => c.op.test(v, value),
        None => {
            missing(fx, &c.operand);
            c.op == Op::Not
        }
    }
}

struct Walker<'a, 'b> {
    b: &'a Behaviour,
    s: &'a Senses<'b>,
    fx: &'a mut Effects,
    stack: Vec<u32>,
    leaf: Option<(u32, Vec<u32>)>,
}

impl Walker<'_, '_> {
    fn ticks(&self, seconds: f32) -> u64 {
        (seconds.max(0.0) * self.s.rate).round() as u64
    }

    fn arrived_at(&self, t: Vec3) -> bool {
        self.s.nav_status == og_nav::STATUS_ARRIVED && self.s.nav_target.distance(t) < 0.05
    }

    fn go(&mut self, t: Vec3, speed: &Option<Speed>, mem: &AgentMem) {
        let speed = match speed {
            None => mem.base_speed,
            Some(Speed::Fixed(v)) => *v,
            Some(Speed::Read(k)) => {
                (self.s.resolve)(k, mem).map_or(mem.base_speed, |v| v.num() as f32)
            }
        };
        self.fx.motion = Motion {
            target: Some(t),
            speed: Some(speed),
        };
    }

    fn tick(&mut self, i: usize, mem: &mut AgentMem) -> Status {
        self.stack.push(i as u32);
        let starting = !mem.nodes[i].running;
        if starting {
            mem.nodes[i].since = self.s.tick;
        }
        let st = self.run(i, mem, starting);
        mem.nodes[i].running = st == Status::Running;
        if self.b.nodes[i].children.is_empty() && (st == Status::Running || self.leaf.is_none()) {
            self.leaf = Some((i as u32, self.stack.clone()));
        }
        self.stack.pop();
        st
    }

    fn run(&mut self, i: usize, mem: &mut AgentMem, starting: bool) -> Status {
        let node = &self.b.nodes[i];
        let kids = node.children.clone();
        match &node.kind {
            Kind::Selector => {
                for (k, &c) in kids.iter().enumerate() {
                    let st = self.tick(c, mem);
                    if st != Status::Failure {
                        for &later in &kids[k + 1..] {
                            if mem.nodes[later].running {
                                reset(self.b, mem, later);
                            }
                        }
                        return st;
                    }
                }
                Status::Failure
            }
            Kind::Sequence => {
                for (k, &c) in kids.iter().enumerate() {
                    let st = self.tick(c, mem);
                    if st != Status::Success {
                        for &later in &kids[k + 1..] {
                            if mem.nodes[later].running {
                                reset(self.b, mem, later);
                            }
                        }
                        return st;
                    }
                }
                Status::Success
            }
            Kind::Steps => {
                let mut k = mem.nodes[i].index as usize;
                while k < kids.len() {
                    match self.tick(kids[k], mem) {
                        Status::Success => k += 1,
                        Status::Running => {
                            mem.nodes[i].index = k as u32;
                            return Status::Running;
                        }
                        Status::Failure => {
                            mem.nodes[i].index = 0;
                            return Status::Failure;
                        }
                    }
                }
                mem.nodes[i].index = 0;
                Status::Success
            }
            Kind::Parallel { all } => {
                let (mut ok, mut bad) = (0, 0);
                for &c in &kids {
                    match self.tick(c, mem) {
                        Status::Success => ok += 1,
                        Status::Failure => bad += 1,
                        Status::Running => {}
                    }
                }
                let n = kids.len();
                if *all {
                    if bad > 0 {
                        Status::Failure
                    } else if ok == n {
                        Status::Success
                    } else {
                        Status::Running
                    }
                } else if ok > 0 {
                    Status::Success
                } else if bad == n {
                    Status::Failure
                } else {
                    Status::Running
                }
            }
            Kind::Invert => match self.tick(kids[0], mem) {
                Status::Success => Status::Failure,
                Status::Failure => Status::Success,
                Status::Running => Status::Running,
            },
            Kind::Succeed => match self.tick(kids[0], mem) {
                Status::Running => Status::Running,
                _ => Status::Success,
            },
            Kind::Repeat { times } => match self.tick(kids[0], mem) {
                Status::Success => {
                    mem.nodes[i].count += 1;
                    if *times != 0 && mem.nodes[i].count >= *times {
                        mem.nodes[i].count = 0;
                        Status::Success
                    } else {
                        reset(self.b, mem, kids[0]);
                        Status::Running
                    }
                }
                other => {
                    if other == Status::Failure {
                        mem.nodes[i].count = 0;
                    }
                    other
                }
            },
            Kind::Cooldown { seconds } => {
                if self.s.tick < mem.nodes[i].until {
                    return Status::Failure;
                }
                let st = self.tick(kids[0], mem);
                if st == Status::Success {
                    mem.nodes[i].until = self.s.tick + self.ticks(*seconds).max(1);
                }
                st
            }
            Kind::Timeout { seconds } => {
                if !starting && self.s.tick >= mem.nodes[i].since + self.ticks(*seconds) {
                    reset(self.b, mem, kids[0]);
                    return Status::Failure;
                }
                self.tick(kids[0], mem)
            }
            Kind::Check(c) => {
                if check(c, self.s, mem, self.fx, node.line, &self.b.name) {
                    Status::Success
                } else {
                    Status::Failure
                }
            }
            Kind::Action(a) => self.act(i, a, mem, starting),
        }
    }

    fn point_of(&self, key: &str, mem: &AgentMem) -> Option<Vec3> {
        match (self.s.resolve)(key, mem)? {
            Value::Vec(v) => Some(v),
            Value::Entity(e) if e != 0 => (self.s.locate)(e),
            _ => None,
        }
    }

    fn act(&mut self, i: usize, a: &Action, mem: &mut AgentMem, starting: bool) -> Status {
        let s = self.s;
        let pos = s.position;
        match a {
            Action::MoveTo { key, point, speed } => {
                let t = match key {
                    Some(k) => match self.point_of(k, mem) {
                        Some(p) => p,
                        None => return Status::Failure,
                    },
                    None => *point,
                };
                if self.arrived_at(t) || pos.distance(t) <= s.stop_distance {
                    return Status::Success;
                }
                if !starting
                    && s.nav_status == og_nav::STATUS_UNREACHABLE
                    && s.nav_target.distance(t) < 0.05
                {
                    return Status::Failure;
                }
                self.go(t, speed, mem);
                Status::Running
            }
            Action::Chase { key, speed, reach } => {
                let Some(t) = self.point_of(key, mem) else {
                    return Status::Failure;
                };
                if pos.distance(t) <= *reach {
                    return Status::Success;
                }
                self.go(t, speed, mem);
                Status::Running
            }
            Action::Flee {
                key,
                speed,
                distance,
            } => {
                let Some(t) = self.point_of(key, mem) else {
                    return Status::Failure;
                };
                let away = pos - t;
                let d = away.length();
                if d >= *distance {
                    return Status::Success;
                }
                let dir = if d > 1e-4 { away / d } else { Vec3::X };
                self.go(pos + dir * (*distance - d + 1.0), speed, mem);
                Status::Running
            }
            Action::Patrol {
                route,
                wait,
                speed,
                loops,
            } => {
                let pts = &self.b.routes[*route].1;
                if starting {
                    // Resume at the nearest point.
                    let mut best = (f32::MAX, 0usize);
                    for (k, p) in pts.iter().enumerate() {
                        let d = p.distance(pos);
                        if d < best.0 {
                            best = (d, k);
                        }
                    }
                    mem.nodes[i].index = best.1 as u32;
                    mem.nodes[i].until = 0;
                }
                if s.tick < mem.nodes[i].until {
                    return Status::Running;
                }
                let k = mem.nodes[i].index as usize % pts.len();
                let t = pts[k];
                if self.arrived_at(t) || pos.distance(t) <= s.stop_distance.max(0.15) {
                    if !*loops && k + 1 == pts.len() {
                        return Status::Success;
                    }
                    mem.nodes[i].index = ((k + 1) % pts.len()) as u32;
                    mem.nodes[i].until = s.tick + self.ticks(*wait);
                    return Status::Running;
                }
                self.go(t, speed, mem);
                Status::Running
            }
            Action::Wander {
                radius,
                speed,
                wait,
            } => {
                if starting || mem.nodes[i].point == Vec3::ZERO {
                    mem.nodes[i].point = self.wander_point(mem.home, *radius, 0);
                }
                if s.tick < mem.nodes[i].until {
                    return Status::Running;
                }
                let t = mem.nodes[i].point;
                let stuck =
                    s.nav_status == og_nav::STATUS_UNREACHABLE && s.nav_target.distance(t) < 0.05;
                if self.arrived_at(t) || pos.distance(t) <= s.stop_distance.max(0.15) || stuck {
                    mem.nodes[i].point =
                        self.wander_point(mem.home, *radius, mem.nodes[i].count as u64 + 1);
                    mem.nodes[i].count += 1;
                    mem.nodes[i].until = s.tick + self.ticks(*wait);
                    return Status::Running;
                }
                self.go(t, speed, mem);
                Status::Running
            }
            Action::Wait { seconds } => {
                if s.tick >= mem.nodes[i].since + self.ticks(*seconds) {
                    Status::Success
                } else {
                    Status::Running
                }
            }
            Action::Stop => Status::Success,
            Action::Set { key, value } => {
                mem.keys.insert(key.clone(), *value);
                Status::Success
            }
            Action::Command {
                name,
                args,
                cooldown,
            } => {
                if s.tick < mem.nodes[i].until {
                    return Status::Failure;
                }
                let mut all = vec![s.entity.to_string()];
                all.extend(args.iter().cloned());
                self.fx.commands.push((name.clone(), all));
                mem.nodes[i].until = s.tick + self.ticks(*cooldown).max(1);
                Status::Success
            }
        }
    }

    fn wander_point(&self, home: Vec3, radius: f32, k: u64) -> Vec3 {
        let h = hash3(self.s.entity, self.s.tick, k);
        let a = (h & 0xffff) as f32 / 65536.0 * std::f32::consts::TAU;
        let r = ((h >> 16) & 0xffff) as f32 / 65536.0 * radius;
        let (sn, cs) = og_core::math::sin_cos(a);
        let p = self.s.plane;
        p.to_world(
            p.to_plane(home) + og_core::glam::Vec2::new(cs * r, sn * r),
            p.height(home),
        )
    }
}

/// One think: a machine's transitions then its state's body, or the tree.
/// Returns the status; `mem.leaf`/`mem.path` name the node it ran.
pub fn think(b: &Behaviour, mem: &mut AgentMem, s: &Senses, fx: &mut Effects) -> Status {
    let mut w = Walker {
        b,
        s,
        fx,
        stack: Vec::new(),
        leaf: None,
    };
    let st = match &b.root {
        Root::Tree(root) => w.tick(*root, mem),
        Root::Machine { states, .. } => {
            let cur = (mem.state as usize).min(states.len() - 1);
            if !mem.entered {
                // The first think: enter the initial state.
                mem.entered = true;
                instant(&states[cur].enter, s, mem, w.fx);
            }
            let elapsed = (s.tick - mem.state_since) as f32 / s.rate;
            let mut next = None;
            for t in &states[cur].transitions {
                if elapsed + 1e-6 >= t.after
                    && t.checks.iter().all(|c| check(c, s, mem, w.fx, 0, &b.name))
                {
                    next = Some(t.to);
                    break;
                }
            }
            if let Some(to) = next {
                instant(&states[cur].exit, s, mem, w.fx);
                if let Some(body) = states[cur].body {
                    reset(b, mem, body);
                }
                mem.state = to as u32;
                mem.state_since = s.tick;
                instant(&states[to].enter, s, mem, w.fx);
            }
            match states[mem.state as usize].body {
                Some(body) => w.tick(body, mem),
                None => Status::Running,
            }
        }
    };
    match w.leaf {
        Some((leaf, path)) => {
            mem.leaf = leaf;
            mem.path = path;
        }
        None => {
            mem.path.clear();
        }
    }
    mem.status = st.code();
    st
}

fn instant(actions: &[Action], s: &Senses, mem: &mut AgentMem, fx: &mut Effects) {
    for a in actions {
        match a {
            Action::Set { key, value } => {
                mem.keys.insert(key.clone(), *value);
            }
            Action::Command { name, args, .. } => {
                let mut all = vec![s.entity.to_string()];
                all.extend(args.iter().cloned());
                fx.commands.push((name.clone(), all));
            }
            _ => {}
        }
    }
}
