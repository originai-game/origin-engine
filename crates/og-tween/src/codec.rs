//! The `og.tween` state block: every running tween, little-endian, in
//! handle order, so equal state gives equal bytes.

use crate::timeline::{Callbacks, Child, Item, Timeline};
use crate::{Ease, Group, GroupKind, Lane, Leaf, Node, Running, State, StepPosition, Target};
use og_core::bytes::{ByteReader, ByteWriter};
use og_core::{Error, ErrorKind, Result};

/// 3: a leaf's repeat delay, and timelines (version 2 still decodes).
const VERSION: u32 = 3;

/// How deep timelines nest in a state block.
const MAX_DEPTH: u32 = 16;

fn floats(w: &mut ByteWriter<'_>, v: &[f32]) {
    w.u32(v.len() as u32);
    for x in v {
        w.f32(*x);
    }
}

fn opt_f64(w: &mut ByteWriter<'_>, v: Option<f64>) {
    w.u32(v.is_some() as u32);
    w.f64(v.unwrap_or(0.0));
}
fn opt_string(w: &mut ByteWriter<'_>, s: &Option<String>) {
    match s {
        None => w.u32(u32::MAX),
        Some(s) => w.str(s),
    }
}
fn callbacks(w: &mut ByteWriter<'_>, c: &Callbacks) {
    for n in c.names() {
        opt_string(w, n);
    }
}

fn timeline(w: &mut ByteWriter<'_>, t: &Timeline) {
    w.u32(t.children.len() as u32);
    for c in &t.children {
        w.f64(c.start);
        w.u32(c.immediate as u32);
        opt_f64(w, c.last);
        callbacks(w, &c.on);
        match &c.item {
            Item::Tween(l) => {
                w.u32(0);
                leaf(w, l);
            }
            Item::Call(name) => {
                w.u32(1);
                w.str(name);
            }
            Item::Timeline(nested) => {
                w.u32(2);
                timeline(w, nested);
            }
        }
    }
    w.u32(t.labels.len() as u32);
    for (name, at) in &t.labels {
        w.str(name);
        w.f64(*at);
    }
    w.i32(t.repeat);
    w.u32(t.yoyo as u32);
    w.f32(t.repeat_delay);
    w.f32(t.time_scale);
    w.u32(t.paused as u32);
    w.u32(t.reversed as u32);
    w.u32(t.auto_kill as u32);
    callbacks(w, &t.on);
    w.f64(t.total);
    opt_f64(w, t.last);
    opt_f64(w, t.last_local);
    w.u32(t.quiet as u32);
    w.u32(t.dirty as u32);
    w.u32(t.recent.is_some() as u32);
    let (a, b) = t.recent.unwrap_or_default();
    w.f64(a);
    w.f64(b);
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

fn leaf(w: &mut ByteWriter<'_>, l: &Leaf) {
    let t = &l.target;
    w.u64(t.entity);
    w.u32(t.component);
    w.u32(t.offset);
    w.u32(lane_code(t.lane));
    w.u32(t.lanes);
    floats(w, &l.to);
    match &l.from {
        None => w.u32(u32::MAX),
        Some(f) => floats(w, f),
    }
    w.u32(l.relative as u32);
    w.f32(l.duration);
    w.f32(l.delay);
    w.u32(l.ease.code());
    match l.ease {
        Ease::CubicBezier(controls) => {
            for v in controls {
                w.f32(v);
            }
        }
        Ease::Steps { count, position } => {
            w.u32(count);
            w.u32(match position {
                StepPosition::JumpStart => 0,
                StepPosition::JumpEnd => 1,
                StepPosition::JumpNone => 2,
                StepPosition::JumpBoth => 3,
            });
        }
        Ease::Points(p) => {
            w.u32(p.len() as u32);
            for v in p.iter().flatten() {
                w.f32(*v);
            }
        }
        _ => {}
    }
    w.i32(l.loops);
    w.u32(l.yoyo as u32);
    w.f32(l.repeat_delay);
    w.f64(l.elapsed);
    w.u32(l.started as u32);
    floats(w, &l.start);
    floats(w, &l.end);
}

pub(crate) fn encode(state: &State, out: &mut Vec<u8>) {
    let w = &mut ByteWriter::new(out);
    w.u32(VERSION);
    w.u64(state.next);
    w.u32(state.running.len() as u32);
    for (h, r) in &state.running {
        w.u64(*h);
        w.u32(r.unscaled as u32);
        w.u32(r.tag);
        match &r.node {
            Node::Leaf(l) => {
                w.u32(0);
                leaf(w, l);
            }
            Node::Group(g) => {
                w.u32(if g.kind == GroupKind::Sequence { 1 } else { 2 });
                w.i32(g.loops);
                w.u32(g.current);
                w.u32(g.runs);
                w.u32(g.children.len() as u32);
                for c in &g.children {
                    leaf(w, c);
                }
            }
            Node::Timeline(t) => {
                w.u32(3);
                timeline(w, t);
            }
        }
    }
}

/// og-core's [`ByteReader`] (its primitives, by `Deref`) and the block's own values.
struct Reader<'a> {
    bytes: ByteReader<'a>,
    version: u32,
}

impl<'a> std::ops::Deref for Reader<'a> {
    type Target = ByteReader<'a>;
    fn deref(&self) -> &ByteReader<'a> {
        &self.bytes
    }
}

impl std::ops::DerefMut for Reader<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bytes
    }
}

fn bad(what: &str) -> Error {
    Error::new(ErrorKind::Parse, format!("og.tween state block: {what}"))
}

impl Reader<'_> {
    fn opt_f64(&mut self) -> Result<Option<f64>> {
        let some = self.u32()? != 0;
        let v = self.f64()?;
        Ok(some.then_some(v))
    }
    fn string(&mut self) -> Result<String> {
        let n = self.u32()?;
        self.string_of(n)
    }
    /// A string of `n` bytes, the length already read.
    fn string_of(&mut self, n: u32) -> Result<String> {
        if n > 4096 {
            return Err(bad("a name is too long"));
        }
        String::from_utf8(self.take(n as usize)?.to_vec()).map_err(|_| bad("a name is not UTF-8"))
    }
    fn opt_string(&mut self) -> Result<Option<String>> {
        match self.u32()? {
            u32::MAX => Ok(None),
            n => self.string_of(n).map(Some),
        }
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
        let n = self.u32()?;
        self.floats_of(n)
    }
    /// `n` floats, the count already read.
    fn floats_of(&mut self, n: u32) -> Result<Vec<f32>> {
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
            match self.u32()? {
                u32::MAX => None,
                n => Some(self.floats_of(n)?),
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
            24 => {
                let n = self.u32()? as usize;
                if !(2..=1024).contains(&n) {
                    return Err(bad("invalid linear() points"));
                }
                let mut p = Vec::with_capacity(n);
                for _ in 0..n {
                    p.push([self.f32()?, self.f32()?]);
                }
                Ease::linear(&p)
            }
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
        bytes: ByteReader::new(bytes, "og.tween state block"),
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
    if !r.is_done() {
        return Err(bad("trailing bytes"));
    }
    Ok(state)
}
