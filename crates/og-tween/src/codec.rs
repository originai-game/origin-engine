//! The `og.tween` state block: every running tween, little-endian, in
//! handle order, so equal state gives equal bytes.

use crate::timeline::{Callbacks, Child, Item, Timeline};
use crate::{Ease, Group, GroupKind, Lane, Leaf, Node, Running, State, StepPosition, Target};
use og_core::{Error, ErrorKind, Result};

/// 3: a leaf's repeat delay, and timelines (version 2 still decodes).
const VERSION: u32 = 3;

/// How deep timelines nest in a state block.
const MAX_DEPTH: u32 = 16;

fn u32_(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn f32_(out: &mut Vec<u8>, v: f32) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn floats(out: &mut Vec<u8>, v: &[f32]) {
    u32_(out, v.len() as u32);
    for x in v {
        f32_(out, *x);
    }
}

fn f64_(out: &mut Vec<u8>, v: f64) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn opt_f64(out: &mut Vec<u8>, v: Option<f64>) {
    u32_(out, v.is_some() as u32);
    f64_(out, v.unwrap_or(0.0));
}
fn string(out: &mut Vec<u8>, s: &str) {
    u32_(out, s.len() as u32);
    out.extend_from_slice(s.as_bytes());
}
fn opt_string(out: &mut Vec<u8>, s: &Option<String>) {
    match s {
        None => u32_(out, u32::MAX),
        Some(s) => string(out, s),
    }
}
fn callbacks(out: &mut Vec<u8>, c: &Callbacks) {
    for n in c.names() {
        opt_string(out, n);
    }
}

fn timeline(out: &mut Vec<u8>, t: &Timeline) {
    u32_(out, t.children.len() as u32);
    for c in &t.children {
        f64_(out, c.start);
        u32_(out, c.immediate as u32);
        opt_f64(out, c.last);
        callbacks(out, &c.on);
        match &c.item {
            Item::Tween(l) => {
                u32_(out, 0);
                leaf(out, l);
            }
            Item::Call(name) => {
                u32_(out, 1);
                string(out, name);
            }
            Item::Timeline(nested) => {
                u32_(out, 2);
                timeline(out, nested);
            }
        }
    }
    u32_(out, t.labels.len() as u32);
    for (name, at) in &t.labels {
        string(out, name);
        f64_(out, *at);
    }
    out.extend_from_slice(&t.repeat.to_le_bytes());
    u32_(out, t.yoyo as u32);
    f32_(out, t.repeat_delay);
    f32_(out, t.time_scale);
    u32_(out, t.paused as u32);
    u32_(out, t.reversed as u32);
    u32_(out, t.auto_kill as u32);
    callbacks(out, &t.on);
    f64_(out, t.total);
    opt_f64(out, t.last);
    opt_f64(out, t.last_local);
    u32_(out, t.quiet as u32);
    u32_(out, t.dirty as u32);
    u32_(out, t.recent.is_some() as u32);
    let (a, b) = t.recent.unwrap_or_default();
    f64_(out, a);
    f64_(out, b);
}

fn lane_code(l: Lane) -> u32 {
    match l {
        Lane::F32 => 0,
        Lane::F64 => 1,
        Lane::I32 => 2,
        Lane::U32 => 3,
        Lane::Quat => 4,
    }
}

fn leaf(out: &mut Vec<u8>, l: &Leaf) {
    let t = &l.target;
    out.extend_from_slice(&t.entity.to_le_bytes());
    u32_(out, t.component);
    u32_(out, t.offset);
    u32_(out, lane_code(t.lane));
    u32_(out, t.lanes);
    floats(out, &l.to);
    match &l.from {
        None => u32_(out, u32::MAX),
        Some(f) => floats(out, f),
    }
    u32_(out, l.relative as u32);
    f32_(out, l.duration);
    f32_(out, l.delay);
    u32_(out, l.ease.code());
    match l.ease {
        Ease::CubicBezier(controls) => {
            for v in controls {
                f32_(out, v);
            }
        }
        Ease::Steps { count, position } => {
            u32_(out, count);
            u32_(
                out,
                match position {
                    StepPosition::JumpStart => 0,
                    StepPosition::JumpEnd => 1,
                    StepPosition::JumpNone => 2,
                    StepPosition::JumpBoth => 3,
                },
            );
        }
        _ => {}
    }
    out.extend_from_slice(&l.loops.to_le_bytes());
    u32_(out, l.yoyo as u32);
    f32_(out, l.repeat_delay);
    out.extend_from_slice(&l.elapsed.to_le_bytes());
    u32_(out, l.started as u32);
    floats(out, &l.start);
    floats(out, &l.end);
}

pub(crate) fn encode(state: &State, out: &mut Vec<u8>) {
    u32_(out, VERSION);
    out.extend_from_slice(&state.next.to_le_bytes());
    u32_(out, state.running.len() as u32);
    for (h, r) in &state.running {
        out.extend_from_slice(&h.to_le_bytes());
        u32_(out, r.unscaled as u32);
        u32_(out, r.tag);
        match &r.node {
            Node::Leaf(l) => {
                u32_(out, 0);
                leaf(out, l);
            }
            Node::Group(g) => {
                u32_(out, if g.kind == GroupKind::Sequence { 1 } else { 2 });
                out.extend_from_slice(&g.loops.to_le_bytes());
                u32_(out, g.current);
                u32_(out, g.runs);
                u32_(out, g.children.len() as u32);
                for c in &g.children {
                    leaf(out, c);
                }
            }
            Node::Timeline(t) => {
                u32_(out, 3);
                timeline(out, t);
            }
        }
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
    version: u32,
}

fn bad(what: &str) -> Error {
    Error::new(ErrorKind::Parse, format!("og.tween state block: {what}"))
}

impl Reader<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8]> {
        let b = self
            .bytes
            .get(self.at..self.at + n)
            .ok_or_else(|| bad("truncated"))?;
        self.at += n;
        Ok(b)
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("4 bytes"),
        ))
    }
    fn i32(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(
            self.take(4)?.try_into().expect("4 bytes"),
        ))
    }
    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("8 bytes"),
        ))
    }
    fn f32(&mut self) -> Result<f32> {
        Ok(f32::from_le_bytes(
            self.take(4)?.try_into().expect("4 bytes"),
        ))
    }
    fn f64(&mut self) -> Result<f64> {
        Ok(f64::from_le_bytes(
            self.take(8)?.try_into().expect("8 bytes"),
        ))
    }
    fn opt_f64(&mut self) -> Result<Option<f64>> {
        let some = self.u32()? != 0;
        let v = self.f64()?;
        Ok(some.then_some(v))
    }
    fn string(&mut self) -> Result<String> {
        let n = self.u32()? as usize;
        if n > 4096 {
            return Err(bad("a name is too long"));
        }
        String::from_utf8(self.take(n)?.to_vec()).map_err(|_| bad("a name is not UTF-8"))
    }
    fn opt_string(&mut self) -> Result<Option<String>> {
        if self.u32()? == u32::MAX {
            return Ok(None);
        }
        self.at -= 4;
        self.string().map(Some)
    }
    fn callbacks(&mut self) -> Result<Callbacks> {
        Ok(Callbacks {
            start: self.opt_string()?,
            update: self.opt_string()?,
            complete: self.opt_string()?,
            repeat: self.opt_string()?,
        })
    }
    fn timeline(&mut self, depth: u32) -> Result<Timeline> {
        if depth > MAX_DEPTH {
            return Err(bad("timelines nest too deep"));
        }
        let n = self.u32()?;
        let mut children = Vec::new();
        for _ in 0..n {
            let start = self.f64()?;
            let immediate = self.u32()? != 0;
            let last = self.opt_f64()?;
            let on = self.callbacks()?;
            let item = match self.u32()? {
                0 => Item::Tween(self.leaf()?),
                1 => Item::Call(self.string()?),
                2 => Item::Timeline(Box::new(self.timeline(depth + 1)?)),
                _ => return Err(bad("unknown timeline child")),
            };
            children.push(Child {
                start,
                item,
                on,
                immediate,
                last,
            });
        }
        let n = self.u32()?;
        let mut labels = Vec::new();
        for _ in 0..n {
            labels.push((self.string()?, self.f64()?));
        }
        Ok(Timeline {
            children,
            labels,
            repeat: self.i32()?,
            yoyo: self.u32()? != 0,
            repeat_delay: self.f32()?,
            time_scale: self.f32()?,
            paused: self.u32()? != 0,
            reversed: self.u32()? != 0,
            auto_kill: self.u32()? != 0,
            on: self.callbacks()?,
            total: self.f64()?,
            last: self.opt_f64()?,
            last_local: self.opt_f64()?,
            quiet: self.u32()? != 0,
            dirty: self.u32()? != 0,
            recent: {
                let some = self.u32()? != 0;
                let r = (self.f64()?, self.f64()?);
                some.then_some(r)
            },
        })
    }
    fn floats(&mut self) -> Result<Vec<f32>> {
        let n = self.u32()? as usize;
        if n > 16 {
            return Err(bad("a value list is too long"));
        }
        (0..n).map(|_| self.f32()).collect()
    }
    fn leaf(&mut self) -> Result<Leaf> {
        let entity = self.u64()?;
        let component = self.u32()?;
        let offset = self.u32()?;
        let lane = match self.u32()? {
            0 => Lane::F32,
            1 => Lane::F64,
            2 => Lane::I32,
            3 => Lane::U32,
            4 => Lane::Quat,
            _ => return Err(bad("unknown lane")),
        };
        let lanes = self.u32()?;
        let to = self.floats()?;
        let from = {
            let n = self.u32()?;
            if n == u32::MAX {
                None
            } else {
                self.at -= 4;
                Some(self.floats()?)
            }
        };
        let relative = self.u32()? != 0;
        let duration = self.f32()?;
        let delay = self.f32()?;
        let ease = match self.u32()? {
            22 => Ease::CubicBezier([self.f32()?, self.f32()?, self.f32()?, self.f32()?]),
            23 => Ease::Steps {
                count: self.u32()?,
                position: match self.u32()? {
                    0 => StepPosition::JumpStart,
                    1 => StepPosition::JumpEnd,
                    2 => StepPosition::JumpNone,
                    3 => StepPosition::JumpBoth,
                    _ => return Err(bad("unknown step position")),
                },
            },
            code => Ease::from_code(code).ok_or_else(|| bad("unknown ease"))?,
        };
        if !ease.is_valid() {
            return Err(bad("invalid ease controls"));
        }
        let loops = self.i32()?;
        let yoyo = self.u32()? != 0;
        let repeat_delay = if self.version >= 3 { self.f32()? } else { 0.0 };
        let elapsed = f64::from_le_bytes(self.take(8)?.try_into().expect("8 bytes"));
        let started = self.u32()? != 0;
        let start = self.floats()?;
        let end = self.floats()?;
        Ok(Leaf {
            target: Target {
                entity,
                component,
                offset,
                lane,
                lanes,
            },
            to,
            from,
            relative,
            duration,
            delay,
            ease,
            loops,
            yoyo,
            repeat_delay,
            elapsed,
            started,
            start,
            end,
        })
    }
}

pub(crate) fn decode(bytes: &[u8]) -> Result<State> {
    let mut r = Reader {
        bytes,
        at: 0,
        version: 0,
    };
    r.version = r.u32()?;
    if !(2..=VERSION).contains(&r.version) {
        return Err(bad("unknown version"));
    }
    let mut state = State {
        next: r.u64()?,
        ..State::default()
    };
    let n = r.u32()?;
    for _ in 0..n {
        let handle = r.u64()?;
        let unscaled = r.u32()? != 0;
        let tag = r.u32()?;
        let node = match r.u32()? {
            0 => Node::Leaf(r.leaf()?),
            k @ (1 | 2) => {
                let loops = r.i32()?;
                let current = r.u32()?;
                let runs = r.u32()?;
                let m = r.u32()?;
                let children = (0..m).map(|_| r.leaf()).collect::<Result<Vec<_>>>()?;
                Node::Group(Group {
                    kind: if k == 1 {
                        GroupKind::Sequence
                    } else {
                        GroupKind::Parallel
                    },
                    children,
                    loops,
                    current,
                    runs,
                })
            }
            3 if r.version >= 3 => Node::Timeline(Box::new(r.timeline(0)?)),
            _ => return Err(bad("unknown node")),
        };
        state.running.insert(
            handle,
            Running {
                node,
                unscaled,
                tag,
            },
        );
    }
    if r.at != bytes.len() {
        return Err(bad("trailing bytes"));
    }
    Ok(state)
}
