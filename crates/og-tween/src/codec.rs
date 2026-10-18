//! The `og.tween` state block: every running tween, little-endian, in
//! handle order, so equal state gives equal bytes.

use crate::{Ease, Group, GroupKind, Lane, Leaf, Node, Running, State, Target};
use og_core::{Error, ErrorKind, Result};

const VERSION: u32 = 1;

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
    out.extend_from_slice(&l.loops.to_le_bytes());
    u32_(out, l.yoyo as u32);
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
        }
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
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
        let ease = Ease::from_code(self.u32()?).ok_or_else(|| bad("unknown ease"))?;
        let loops = self.i32()?;
        let yoyo = self.u32()? != 0;
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
            elapsed,
            started,
            start,
            end,
        })
    }
}

pub(crate) fn decode(bytes: &[u8]) -> Result<State> {
    let mut r = Reader { bytes, at: 0 };
    if r.u32()? != VERSION {
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
