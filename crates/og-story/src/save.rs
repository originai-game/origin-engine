//! The story's saved form: `OGST`, a version byte, then every field of
//! [`Story`] but its tick and warnings, little-endian, strings as a `u32`
//! length and UTF-8, maps and sets in key order, so equal stories save
//! equal bytes (the world hash compares them). Places are saved by id
//! (dialogue, node, line key, quest, stage, objective), never by index.

use crate::expr::Val;
use crate::story::{Conv, Event, QuestState, Roll, Shown, Story, Wait};
use og_core::Rng;
use std::collections::{BTreeMap, BTreeSet};

const MAGIC: &[u8; 4] = b"OGST";
/// The saved form's version.
pub const SAVE_VERSION: u8 = 1;

struct W<'a>(&'a mut Vec<u8>);

impl W<'_> {
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn f64(&mut self, v: f64) {
        self.u64(v.to_bits());
    }
    fn str(&mut self, s: &str) {
        self.u32(s.len() as u32);
        self.0.extend_from_slice(s.as_bytes());
    }
    fn val(&mut self, v: &Val) {
        match v {
            Val::Num(n) => {
                self.u8(0);
                self.f64(*n);
            }
            Val::Bool(b) => {
                self.u8(1);
                self.u8(u8::from(*b));
            }
            Val::Str(s) => {
                self.u8(2);
                self.str(s);
            }
        }
    }
    fn vars(&mut self, m: &BTreeMap<String, Val>) {
        self.u32(m.len() as u32);
        for (k, v) in m {
            self.str(k);
            self.val(v);
        }
    }
    fn set(&mut self, s: &BTreeSet<String>) {
        self.u32(s.len() as u32);
        for k in s {
            self.str(k);
        }
    }
    fn roll(&mut self, r: &Roll) {
        self.str(&r.stat);
        self.f64(r.target);
        self.u32(r.roll);
        self.u8(u8::from(r.passed));
    }
}

struct R<'a> {
    b: &'a [u8],
    at: usize,
}

impl R<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], String> {
        let end = self.at.checked_add(n).filter(|e| *e <= self.b.len());
        let end = end.ok_or("the story's saved state is cut short")?;
        let s = &self.b[self.at..end];
        self.at = end;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn bool(&mut self) -> Result<bool, String> {
        Ok(self.u8()? != 0)
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn f64(&mut self) -> Result<f64, String> {
        Ok(f64::from_bits(self.u64()?))
    }
    fn f32(&mut self) -> Result<f32, String> {
        Ok(f32::from_bits(self.u32()?))
    }
    fn str(&mut self) -> Result<String, String> {
        let n = self.u32()? as usize;
        String::from_utf8(self.take(n)?.to_vec())
            .map_err(|_| "a string in the story's saved state is not UTF-8".to_string())
    }
    fn count(&mut self) -> Result<usize, String> {
        let n = self.u32()? as usize;
        // Every element takes at least a byte: a count past what is left is corrupt.
        if n > self.b.len() - self.at {
            return Err("the story's saved state has a count past its end".into());
        }
        Ok(n)
    }
    fn val(&mut self) -> Result<Val, String> {
        Ok(match self.u8()? {
            0 => Val::Num(self.f64()?),
            1 => Val::Bool(self.bool()?),
            2 => Val::Str(self.str()?),
            t => return Err(format!("the story's saved state has a value of kind {t}")),
        })
    }
    fn vars(&mut self) -> Result<BTreeMap<String, Val>, String> {
        let mut m = BTreeMap::new();
        for _ in 0..self.count()? {
            let k = self.str()?;
            m.insert(k, self.val()?);
        }
        Ok(m)
    }
    fn set(&mut self) -> Result<BTreeSet<String>, String> {
        let mut s = BTreeSet::new();
        for _ in 0..self.count()? {
            s.insert(self.str()?);
        }
        Ok(s)
    }
    fn roll(&mut self) -> Result<Roll, String> {
        Ok(Roll {
            stat: self.str()?,
            target: self.f64()?,
            roll: self.u32()?,
            passed: self.bool()?,
        })
    }
}

fn status_code(s: &str) -> u8 {
    match s {
        "active" => 0,
        "completed" => 1,
        _ => 2,
    }
}

fn status_of(c: u8) -> Result<&'static str, String> {
    Ok(match c {
        0 => "active",
        1 => "completed",
        2 => "failed",
        _ => return Err(format!("the story's saved state has a quest status {c}")),
    })
}

impl Story {
    /// Appends the saved form.
    pub fn save(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(MAGIC);
        let mut w = W(out);
        w.u8(SAVE_VERSION);
        w.u32(self.rng.seed());
        w.u32(self.rng.save());
        w.u32(self.seq);
        w.u32(self.next_conv);
        w.vars(&self.globals);
        w.set(&self.seen);
        w.set(&self.read);
        w.u32(self.convs.len() as u32);
        for (id, c) in &self.convs {
            w.u32(*id);
            w.str(&c.dialogue);
            w.str(&c.node);
            w.u32(c.pc);
            w.u8(match c.wait {
                Wait::Line => 0,
                Wait::Choices => 1,
                Wait::Ended => 2,
            });
            w.vars(&c.locals);
            w.u32(c.cast.len() as u32);
            for (k, e) in &c.cast {
                w.str(k);
                w.u64(*e);
            }
            w.set(&c.visited);
            w.u32(c.history.len() as u32);
            for s in &c.history {
                w.str(&s.node);
                w.u32(s.step);
                w.str(&s.speaker);
                w.str(&s.key);
                w.u32(s.args.len() as u32);
                for (k, v) in &s.args {
                    w.str(k);
                    w.val(v);
                }
                w.str(&s.voice);
                w.str(&s.expression);
                w.u32(s.seq);
                w.u8(u8::from(s.seen));
                match &s.chosen {
                    Some((i, id)) => {
                        w.u8(1);
                        w.u32(*i);
                        w.str(id);
                    }
                    None => w.u8(0),
                }
            }
            match c.timer {
                Some((l, t)) => {
                    w.u8(1);
                    w.u32(l);
                    w.u32(t);
                }
                None => w.u8(0),
            }
            match &c.last_roll {
                Some(r) => {
                    w.u8(1);
                    w.roll(r);
                }
                None => w.u8(0),
            }
        }
        w.u32(self.quests.len() as u32);
        for q in &self.quests {
            w.str(&q.id);
            w.str(&q.stage);
            w.u8(status_code(q.status));
            w.u32(q.counts.len() as u32);
            for (k, n) in &q.counts {
                w.str(k);
                w.u32(*n);
            }
            w.set(&q.failed);
            w.u8(u8::from(q.tracked));
            w.u8(u8::from(q.is_new));
            w.u8(u8::from(q.updated));
            w.u32(q.updated_tick);
            match q.marker {
                Some((e, p)) => {
                    w.u8(1);
                    w.u64(e);
                    for x in p {
                        w.u32(x.to_bits());
                    }
                }
                None => w.u8(0),
            }
        }
        w.u32(self.events.len() as u32);
        for e in &self.events {
            match e {
                Event::Line { conv, seq } => {
                    w.u8(0);
                    w.u32(*conv);
                    w.u32(*seq);
                }
                Event::Ended { conv } => {
                    w.u8(1);
                    w.u32(*conv);
                }
                Event::Check { conv, roll } => {
                    w.u8(2);
                    w.u32(*conv);
                    w.roll(roll);
                }
                Event::Quest { quest, status } => {
                    w.u8(3);
                    w.str(quest);
                    w.u8(status_code(status));
                }
                Event::Emit { conv, name, arg } => {
                    w.u8(4);
                    w.u32(*conv);
                    w.str(name);
                    w.str(arg);
                }
            }
        }
    }

    /// A story from its saved form; the tick is 0 and no warning is pending.
    pub fn load(bytes: &[u8]) -> Result<Story, String> {
        let rest = bytes
            .strip_prefix(MAGIC)
            .ok_or("the story's saved state does not start with OGST")?;
        let mut r = R { b: rest, at: 0 };
        let v = r.u8()?;
        if v != SAVE_VERSION {
            return Err(format!(
                "the story's saved state has version {v}; this engine reads {SAVE_VERSION}"
            ));
        }
        let seed = r.u32()?;
        let state = r.u32()?;
        let mut rng = Rng::new(seed);
        rng.restore(state);
        let mut s = Story::new(rng);
        s.seq = r.u32()?;
        s.next_conv = r.u32()?;
        s.globals = r.vars()?;
        s.seen = r.set()?;
        s.read = r.set()?;
        for _ in 0..r.count()? {
            let id = r.u32()?;
            let dialogue = r.str()?;
            let node = r.str()?;
            let pc = r.u32()?;
            let wait = match r.u8()? {
                0 => Wait::Line,
                1 => Wait::Choices,
                2 => Wait::Ended,
                w => return Err(format!("the story's saved state has a wait of kind {w}")),
            };
            let locals = r.vars()?;
            let mut cast = BTreeMap::new();
            for _ in 0..r.count()? {
                let k = r.str()?;
                cast.insert(k, r.u64()?);
            }
            let visited = r.set()?;
            let mut history = Vec::new();
            for _ in 0..r.count()? {
                let node = r.str()?;
                let step = r.u32()?;
                let speaker = r.str()?;
                let key = r.str()?;
                let mut args = Vec::new();
                for _ in 0..r.count()? {
                    let k = r.str()?;
                    args.push((k, r.val()?));
                }
                let voice = r.str()?;
                let expression = r.str()?;
                let seq = r.u32()?;
                let seen = r.bool()?;
                let chosen = if r.bool()? {
                    let i = r.u32()?;
                    Some((i, r.str()?))
                } else {
                    None
                };
                history.push(Shown {
                    node,
                    step,
                    speaker,
                    key,
                    args,
                    voice,
                    expression,
                    seq,
                    seen,
                    chosen,
                });
            }
            let timer = if r.bool()? {
                Some((r.u32()?, r.u32()?))
            } else {
                None
            };
            let last_roll = if r.bool()? { Some(r.roll()?) } else { None };
            s.convs.insert(
                id,
                Conv {
                    dialogue,
                    node,
                    pc,
                    wait,
                    locals,
                    cast,
                    visited,
                    history,
                    timer,
                    last_roll,
                },
            );
        }
        for _ in 0..r.count()? {
            let id = r.str()?;
            let stage = r.str()?;
            let status = status_of(r.u8()?)?;
            let mut counts = BTreeMap::new();
            for _ in 0..r.count()? {
                let k = r.str()?;
                counts.insert(k, r.u32()?);
            }
            let failed = r.set()?;
            let tracked = r.bool()?;
            let is_new = r.bool()?;
            let updated = r.bool()?;
            let updated_tick = r.u32()?;
            let marker = if r.bool()? {
                let e = r.u64()?;
                Some((e, [r.f32()?, r.f32()?, r.f32()?]))
            } else {
                None
            };
            s.quests.push(QuestState {
                id,
                stage,
                status,
                counts,
                failed,
                tracked,
                is_new,
                updated,
                updated_tick,
                marker,
            });
        }
        for _ in 0..r.count()? {
            let e = match r.u8()? {
                0 => Event::Line {
                    conv: r.u32()?,
                    seq: r.u32()?,
                },
                1 => Event::Ended { conv: r.u32()? },
                2 => Event::Check {
                    conv: r.u32()?,
                    roll: r.roll()?,
                },
                3 => Event::Quest {
                    quest: r.str()?,
                    status: status_of(r.u8()?)?,
                },
                4 => Event::Emit {
                    conv: r.u32()?,
                    name: r.str()?,
                    arg: r.str()?,
                },
                t => return Err(format!("the story's saved state has an event of kind {t}")),
            };
            s.events.push(e);
        }
        if r.at != r.b.len() {
            return Err("the story's saved state has bytes past its end".into());
        }
        Ok(s)
    }
}
