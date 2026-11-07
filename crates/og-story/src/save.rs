//! The story's saved form: `OGST`, a version byte, then every field of
//! [`Story`] but its warnings, little-endian, strings as a `u32`
//! length and UTF-8, maps and sets in key order, so equal stories save
//! equal bytes (the world hash compares them). Places are saved by id
//! (dialogue, node, line key, quest, stage, objective), never by index.

use crate::bark::{Ambient, Request, Said};
use crate::expr::Val;
use crate::story::{ChoiceValues, Conv, Event, QuestState, Roll, Shown, Story, Wait};
use crate::yarn::saliency::Saliency;
use og_core::Rng;
use std::collections::{BTreeMap, BTreeSet};

const MAGIC: &[u8; 4] = b"OGST";
/// The saved form's version.
/// 2 (G2-8 batch 2) adds each line's voice timing, barks, ambient
/// barkers and the voices playing; 3 (R10) each line's camera shot and
/// the solo the story holds. Version 4 replaces offsets with stable
/// continuation IDs and saves detour stacks. Older versions remain readable
/// for quest state; their dialogue continuations are not migrated to Yarn.
/// Version 5 records source-node exits separately from IR-block traversal.
/// Version 6 checkpoints offered option values, including random arguments.
/// Version 7 retains the chosen option's arguments in dialogue history.
/// Version 8 checkpoints option reason arguments and check targets.
pub const SAVE_VERSION: u8 = 8;

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
    fn opt_u32(&mut self, v: Option<u32>) {
        match v {
            Some(x) => {
                self.u8(1);
                self.u32(x);
            }
            None => self.u8(0),
        }
    }
    fn args(&mut self, a: &[(String, Val)]) {
        self.u32(a.len() as u32);
        for (k, v) in a {
            self.str(k);
            self.val(v);
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
    fn opt_u32(&mut self) -> Result<Option<u32>, String> {
        Ok(if self.bool()? {
            Some(self.u32()?)
        } else {
            None
        })
    }
    fn args(&mut self) -> Result<Vec<(String, Val)>, String> {
        let mut out = Vec::new();
        for _ in 0..self.count()? {
            let k = self.str()?;
            out.push((k, self.val()?));
        }
        Ok(out)
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
        w.u32(self.tick);
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
            w.u32(c.stack.len() as u32);
            for continuation in &c.stack {
                w.str(continuation);
            }
            w.u32(c.active_nodes.len() as u32);
            for frame in &c.active_nodes {
                w.u32(frame.len() as u32);
                for node in frame {
                    w.str(node);
                }
            }
            w.u8(match c.wait {
                Wait::Line => 0,
                Wait::Choices => 1,
                Wait::Ended => 2,
            });
            w.vars(&c.locals);
            w.u32(c.choice_values.len() as u32);
            for (id, value) in &c.choice_values {
                w.str(id);
                w.u8(u8::from(value.visible));
                w.u8(u8::from(value.enabled));
                w.f64(value.chance);
                w.args(&value.args);
                w.args(&value.reason_args);
                w.u8(u8::from(value.target.is_some()));
                if let Some(target) = value.target {
                    w.f64(target);
                }
            }
            w.u32(c.cast.len() as u32);
            for (k, e) in &c.cast {
                w.str(k);
                w.u64(*e);
            }
            w.set(&c.visited);
            w.u32(c.history.len() as u32);
            for s in &c.history {
                w.str(&s.node);
                w.str(&s.speaker);
                w.str(&s.key);
                w.args(&s.args);
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
                w.u32(s.voice_ticks);
                w.opt_u32(s.voice_start);
                w.str(&s.shot);
                w.args(&s.chosen_args);
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
                Event::Bark {
                    entity,
                    bark,
                    key,
                    seq,
                } => {
                    w.u8(5);
                    w.u64(*entity);
                    w.str(bark);
                    w.str(key);
                    w.u32(*seq);
                }
            }
        }
        // Version 2: barks, ambient barkers, cooldowns and voices.
        w.u32(self.barks.len() as u32);
        for (e, b) in &self.barks {
            w.u64(*e);
            w.str(&b.dialogue);
            w.str(&b.bark);
            w.str(&b.key);
            w.args(&b.args);
            w.str(&b.speaker);
            w.str(&b.voice);
            w.u32(b.seq);
            w.u32(b.start);
            w.u32(b.until);
            w.u32(b.priority as u32);
            w.u32(b.voice_ticks);
            w.opt_u32(b.voice_start);
        }
        w.u32(self.bark_requests.len() as u32);
        for q in &self.bark_requests {
            w.u64(q.entity);
            w.str(&q.topic);
            w.str(&q.dialogue);
        }
        w.u32(self.ambient.len() as u32);
        for (e, a) in &self.ambient {
            w.u64(*e);
            w.str(&a.topic);
            w.str(&a.dialogue);
            w.u32(a.every);
            w.u32(a.jitter);
            w.u32(a.next);
        }
        for m in [&self.bark_last, &self.bark_cycle] {
            w.u32(m.len() as u32);
            for (k, t) in m {
                w.str(k);
                w.u32(*t);
            }
        }
        w.u32(self.bark_last_by.len() as u32);
        for ((k, e), t) in &self.bark_last_by {
            w.str(k);
            w.u64(*e);
            w.u32(*t);
        }
        w.set(&self.bark_once);
        w.u32(self.voices.len() as u32);
        for (k, (seq, e)) in &self.voices {
            w.str(k);
            w.u32(*seq);
            w.u64(*e);
        }
        // Version 3: the solo shot the story holds.
        match self.camera {
            Some((found, wrote)) => {
                w.u8(1);
                w.u64(found);
                w.u64(wrote);
            }
            None => w.u8(0),
        }
        w.u32(self.visits.len() as u32);
        for (node, count) in &self.visits {
            w.str(node);
            w.u32(*count);
        }
        w.set(&self.once);
        w.u8(match self.saliency {
            Saliency::First => 0,
            Saliency::Best => 1,
            Saliency::BestLeastRecentlyViewed => 2,
            Saliency::RandomBestLeastRecentlyViewed => 3,
            Saliency::ConditionalFirstWeighted => 4,
        });
        w.u32(self.content_views.len() as u32);
        for (key, count) in &self.content_views {
            w.str(key);
            w.u32(*count);
        }
    }

    /// A story from its saved form; no warning is pending.
    pub fn load(bytes: &[u8]) -> Result<Story, String> {
        let rest = bytes
            .strip_prefix(MAGIC)
            .ok_or("the story's saved state does not start with OGST")?;
        let mut r = R { b: rest, at: 0 };
        let v = r.u8()?;
        if !(1..=SAVE_VERSION).contains(&v) {
            return Err(format!(
                "the story's saved state has version {v}; this engine reads {SAVE_VERSION}"
            ));
        }
        let seed = r.u32()?;
        let state = r.u32()?;
        let mut rng = Rng::new(seed);
        rng.restore(state);
        let mut s = Story::new(rng);
        if v >= 4 {
            s.tick = r.u32()?;
        }
        s.seq = r.u32()?;
        s.next_conv = r.u32()?;
        s.globals = r.vars()?;
        s.seen = r.set()?;
        s.read = r.set()?;
        for _ in 0..r.count()? {
            let id = r.u32()?;
            let dialogue = r.str()?;
            let node = r.str()?;
            let mut stack = Vec::new();
            let pc = if v < 4 {
                r.u32()?
            } else {
                for _ in 0..r.count()? {
                    stack.push(r.str()?);
                }
                0
            };
            let mut active_nodes = Vec::new();
            if v >= 5 {
                for _ in 0..r.count()? {
                    let mut frame = Vec::new();
                    for _ in 0..r.count()? {
                        frame.push(r.str()?);
                    }
                    active_nodes.push(frame);
                }
            }
            let wait = match r.u8()? {
                0 => Wait::Line,
                1 => Wait::Choices,
                2 => Wait::Ended,
                w => return Err(format!("the story's saved state has a wait of kind {w}")),
            };
            let locals = r.vars()?;
            let mut choice_values = BTreeMap::new();
            if v >= 6 {
                for _ in 0..r.count()? {
                    choice_values.insert(
                        r.str()?,
                        ChoiceValues {
                            visible: r.bool()?,
                            enabled: r.bool()?,
                            chance: r.f64()?,
                            args: r.args()?,
                            reason_args: if v >= 8 { r.args()? } else { Vec::new() },
                            target: if v >= 8 && r.bool()? {
                                Some(r.f64()?)
                            } else {
                                None
                            },
                        },
                    );
                }
            }
            let mut cast = BTreeMap::new();
            for _ in 0..r.count()? {
                let k = r.str()?;
                cast.insert(k, r.u64()?);
            }
            let visited = r.set()?;
            let mut history = Vec::new();
            for _ in 0..r.count()? {
                let node = r.str()?;
                let step = if v < 4 { r.u32()? } else { 0 };
                let speaker = r.str()?;
                let key = r.str()?;
                let args = r.args()?;
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
                let (voice_ticks, voice_start) = if v >= 2 {
                    (r.u32()?, r.opt_u32()?)
                } else {
                    (0, None)
                };
                let shot = if v >= 3 { r.str()? } else { String::new() };
                let chosen_args = if v >= 7 { r.args()? } else { Vec::new() };
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
                    chosen_args,
                    voice_ticks,
                    voice_start,
                    shot,
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
                    choice_values,
                    cast,
                    visited,
                    history,
                    timer,
                    last_roll,
                    stack,
                    active_nodes,
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
                5 => Event::Bark {
                    entity: r.u64()?,
                    bark: r.str()?,
                    key: r.str()?,
                    seq: r.u32()?,
                },
                t => return Err(format!("the story's saved state has an event of kind {t}")),
            };
            s.events.push(e);
        }
        if v >= 2 {
            for _ in 0..r.count()? {
                let e = r.u64()?;
                let dialogue = r.str()?;
                let bark = r.str()?;
                if v < 4 {
                    r.u32()?;
                }
                let said = Said {
                    dialogue,
                    bark,
                    key: r.str()?,
                    args: r.args()?,
                    speaker: r.str()?,
                    voice: r.str()?,
                    seq: r.u32()?,
                    start: r.u32()?,
                    until: r.u32()?,
                    priority: r.u32()? as i32,
                    voice_ticks: r.u32()?,
                    voice_start: r.opt_u32()?,
                };
                s.barks.insert(e, said);
            }
            for _ in 0..r.count()? {
                s.bark_requests.push(Request {
                    entity: r.u64()?,
                    topic: r.str()?,
                    dialogue: r.str()?,
                });
            }
            for _ in 0..r.count()? {
                let e = r.u64()?;
                let a = Ambient {
                    topic: r.str()?,
                    dialogue: r.str()?,
                    every: r.u32()?,
                    jitter: r.u32()?,
                    next: r.u32()?,
                };
                s.ambient.insert(e, a);
            }
            for i in 0..2 {
                let mut m = BTreeMap::new();
                for _ in 0..r.count()? {
                    let k = r.str()?;
                    m.insert(k, r.u32()?);
                }
                if i == 0 {
                    s.bark_last = m;
                } else {
                    s.bark_cycle = m;
                }
            }
            for _ in 0..r.count()? {
                let k = r.str()?;
                let e = r.u64()?;
                s.bark_last_by.insert((k, e), r.u32()?);
            }
            s.bark_once = r.set()?;
            for _ in 0..r.count()? {
                let k = r.str()?;
                let seq = r.u32()?;
                s.voices.insert(k, (seq, r.u64()?));
            }
        }
        if v >= 3 && r.bool()? {
            s.camera = Some((r.u64()?, r.u64()?));
        }
        if v >= 5 {
            for _ in 0..r.count()? {
                let node = r.str()?;
                s.visits.insert(node, r.u32()?);
            }
            s.once = r.set()?;
            s.saliency = match r.u8()? {
                0 => Saliency::First,
                1 => Saliency::Best,
                2 => Saliency::BestLeastRecentlyViewed,
                3 => Saliency::RandomBestLeastRecentlyViewed,
                4 => Saliency::ConditionalFirstWeighted,
                _ => return Err("invalid saved saliency policy".into()),
            };
            for _ in 0..r.count()? {
                let key = r.str()?;
                s.content_views.insert(key, r.u32()?);
            }
        }
        if r.at != r.b.len() {
            return Err("the story's saved state has bytes past its end".into());
        }
        Ok(s)
    }
}
