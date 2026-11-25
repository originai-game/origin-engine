//! Deterministic saved state block for `og.flow` (`FlowBlock`).
//!
//! Keeps queued flow ops, touch gestures, inspected hotspot records, transcript
//! backlog, cross-segment persistent BGM state, and the project-declared flow
//! handle inside the world snapshot and hash so save/restore and replay are
//! exact across every tick.

use super::{ModuleState, bad, lock};
use crate::host::wire::{
    FlowGestureDesc, FlowJumpDesc, FlowMusicDesc, FlowTranscriptLine, OneOf, TextSpan,
};
use og_core::bytes::ByteReader;
use og_ecs::StateBlock;
use serde_json::{Value, json};
use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};

/// Maximum transcript lines retained per flow.
pub(super) const MAX_HISTORY: usize = 500;

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Op {
    Choose(u32),
    Play,
    Pause,
    Skip,
    Stop,
    /// og.flow.advance: cut now, by this outcome or the window's choice.
    Advance(Option<String>),
    /// og.flow.setSpeed: playback rate multiplier in 0.25..=4.0.
    SetSpeed(f32),
    /// og.flow.seek: seek within the current segment in 0..=cut.
    Seek(f32),
    /// og.flow.inspect: trigger an active inspect hotspot without cutting.
    Inspect(String),
    /// og.flow.setMusic: start, crossfade or stop cross-segment BGM.
    SetMusic(FlowMusicDesc),
    /// og.flow.jump: branch rewind / chapter jump.
    Jump(FlowJumpDesc),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct FlowHistory {
    pub next_seq: u32,
    pub last_caption_ids: BTreeSet<String>,
    pub lines: Vec<FlowTranscriptLine>,
}

impl FlowHistory {
    pub fn push(
        &mut self,
        node: String,
        speaker: String,
        text: String,
        spans: Vec<TextSpan>,
        chosen: bool,
    ) {
        self.next_seq = self.next_seq.saturating_add(1);
        self.lines.push(FlowTranscriptLine {
            seq: self.next_seq,
            node,
            speaker,
            text,
            spans,
            chosen,
        });
        if self.lines.len() > MAX_HISTORY {
            let drop_n = self.lines.len() - MAX_HISTORY;
            self.lines.drain(..drop_n);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct MusicState {
    pub sound: String,
    pub entity: u64,
    pub prev_entity: u64,
    pub volume: f32,
    pub duck: f32,
    pub fade_ticks: u32,
    pub fade_left: u32,
    pub current_duck: f32,
}

impl Default for MusicState {
    fn default() -> Self {
        Self {
            sound: String::new(),
            entity: 0,
            prev_entity: 0,
            volume: 1.0,
            duck: 0.4,
            fade_ticks: 0,
            fade_left: 0,
            current_duck: 1.0,
        }
    }
}

/// All deterministic simulation state `og.flow` owns outside its ECS components.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct Saved {
    pub ops: Vec<(u64, Op)>,
    pub gestures: Vec<(u64, FlowGestureDesc)>,
    /// Inspected hotspots per flow entity (`"<node>:<hotspot>"`).
    pub inspected: BTreeMap<u64, BTreeSet<String>>,
    /// Transcript backlog per flow entity.
    pub history: BTreeMap<u64, FlowHistory>,
    /// Cross-segment persistent BGM per flow entity.
    pub music: BTreeMap<u64, MusicState>,
    /// The flow project.ogt's `flow` started (`og.flow.current`).
    pub declared: Option<u64>,
}

impl Saved {
    pub fn remove_flow(&mut self, flow: u64) {
        self.ops.retain(|(k, _)| *k != flow);
        self.gestures.retain(|(k, _)| *k != flow);
        self.inspected.remove(&flow);
        self.history.remove(&flow);
        self.music.remove(&flow);
        if self.declared == Some(flow) {
            self.declared = None;
        }
    }
}

fn push_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn push_u64(out: &mut Vec<u8>, v: u64) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn push_f32(out: &mut Vec<u8>, v: f32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn push_str(out: &mut Vec<u8>, s: &str) {
    push_u32(out, s.len() as u32);
    out.extend_from_slice(s.as_bytes());
}

fn push_span(out: &mut Vec<u8>, s: &TextSpan) {
    push_str(out, &s.name);
    push_u32(out, s.position);
    push_u32(out, s.length);
    push_u32(out, s.properties.len() as u32);
    for (k, v) in &s.properties {
        push_str(out, k);
        match v {
            OneOf::A(t) => {
                out.push(0);
                push_str(out, t);
            }
            OneOf::B(OneOf::A(n)) => {
                out.push(1);
                out.extend_from_slice(&n.to_le_bytes());
            }
            OneOf::B(OneOf::B(b)) => {
                out.push(2);
                out.push(u8::from(*b));
            }
        }
    }
}

fn read_span(b: &mut ByteReader<'_>) -> og_core::Result<TextSpan> {
    let name = b.string()?;
    let position = b.u32()?;
    let length = b.u32()?;
    let count = b.u32()?;
    let mut properties = BTreeMap::new();
    for _ in 0..count {
        let k = b.string()?;
        let tag = b.u8()?;
        let v = match tag {
            0 => OneOf::A(b.string()?),
            1 => OneOf::B(OneOf::A(b.f64()?)),
            2 => OneOf::B(OneOf::B(b.u8()? != 0)),
            _ => return Err(bad("unknown og.flow span property tag")),
        };
        properties.insert(k, v);
    }
    Ok(TextSpan {
        name,
        position,
        length,
        properties,
    })
}

pub(super) fn save(s: &Saved, out: &mut Vec<u8>) {
    // Ops
    push_u32(out, s.ops.len() as u32);
    for (e, op) in &s.ops {
        push_u64(out, *e);
        match op {
            Op::Choose(i) => {
                out.push(0);
                push_u32(out, *i);
            }
            Op::Play => out.push(1),
            Op::Pause => out.push(2),
            Op::Skip => out.push(3),
            Op::Stop => out.push(4),
            Op::Advance(o) => {
                out.push(5);
                push_str(out, o.as_deref().unwrap_or(""));
            }
            Op::SetSpeed(sp) => {
                out.push(6);
                push_f32(out, *sp);
            }
            Op::Seek(t) => {
                out.push(7);
                push_f32(out, *t);
            }
            Op::Inspect(id) => {
                out.push(8);
                push_str(out, id);
            }
            Op::SetMusic(m) => {
                out.push(9);
                push_str(out, &m.sound);
                push_f32(out, m.volume);
                push_f32(out, m.fade);
                push_f32(out, m.duck);
            }
            Op::Jump(j) => {
                out.push(10);
                push_str(out, &j.node);
                push_str(out, &j.vars.to_string());
            }
        }
    }
    // Gestures
    push_u32(out, s.gestures.len() as u32);
    for (e, g) in &s.gestures {
        push_u64(out, *e);
        push_str(out, &g.kind);
        push_str(out, &g.direction);
        push_f32(out, g.progress);
    }
    // Inspected
    push_u32(out, s.inspected.len() as u32);
    for (e, set) in &s.inspected {
        push_u64(out, *e);
        push_u32(out, set.len() as u32);
        for key in set {
            push_str(out, key);
        }
    }
    // History
    push_u32(out, s.history.len() as u32);
    for (e, h) in &s.history {
        push_u64(out, *e);
        push_u32(out, h.next_seq);
        push_u32(out, h.last_caption_ids.len() as u32);
        for id in &h.last_caption_ids {
            push_str(out, id);
        }
        push_u32(out, h.lines.len() as u32);
        for l in &h.lines {
            push_u32(out, l.seq);
            push_str(out, &l.node);
            push_str(out, &l.speaker);
            push_str(out, &l.text);
            out.push(u8::from(l.chosen));
            push_u32(out, l.spans.len() as u32);
            for sp in &l.spans {
                push_span(out, sp);
            }
        }
    }
    // Music
    push_u32(out, s.music.len() as u32);
    for (e, m) in &s.music {
        push_u64(out, *e);
        push_str(out, &m.sound);
        push_u64(out, m.entity);
        push_u64(out, m.prev_entity);
        push_f32(out, m.volume);
        push_f32(out, m.duck);
        push_u32(out, m.fade_ticks);
        push_u32(out, m.fade_left);
        push_f32(out, m.current_duck);
    }
    // Declared
    push_u64(out, s.declared.unwrap_or(0));
}

pub(super) fn load_state(bytes: &[u8]) -> og_core::Result<Saved> {
    let mut b = ByteReader::new(bytes, "the og.flow state");
    let mut s = Saved::default();
    for _ in 0..b.u32()? {
        let e = b.u64()?;
        let op = match b.u8()? {
            0 => Op::Choose(b.u32()?),
            1 => Op::Play,
            2 => Op::Pause,
            3 => Op::Skip,
            4 => Op::Stop,
            5 => {
                let o = b.string()?;
                Op::Advance((!o.is_empty()).then_some(o))
            }
            6 => Op::SetSpeed(b.f32()?),
            7 => Op::Seek(b.f32()?),
            8 => Op::Inspect(b.string()?),
            9 => Op::SetMusic(FlowMusicDesc {
                sound: b.string()?,
                volume: b.f32()?,
                fade: b.f32()?,
                duck: b.f32()?,
            }),
            10 => {
                let node = b.string()?;
                let raw = b.string()?;
                let vars: Value = serde_json::from_str(&raw).unwrap_or(Value::Null);
                Op::Jump(FlowJumpDesc { node, vars })
            }
            _ => return Err(bad("unknown og.flow op")),
        };
        s.ops.push((e, op));
    }
    for _ in 0..b.u32()? {
        let e = b.u64()?;
        s.gestures.push((
            e,
            FlowGestureDesc {
                kind: b.string()?,
                direction: b.string()?,
                progress: b.f32()?,
            },
        ));
    }
    for _ in 0..b.u32()? {
        let e = b.u64()?;
        let mut set = BTreeSet::new();
        for _ in 0..b.u32()? {
            set.insert(b.string()?);
        }
        s.inspected.insert(e, set);
    }
    for _ in 0..b.u32()? {
        let e = b.u64()?;
        let next_seq = b.u32()?;
        let mut last_caption_ids = BTreeSet::new();
        for _ in 0..b.u32()? {
            last_caption_ids.insert(b.string()?);
        }
        let mut lines = Vec::new();
        for _ in 0..b.u32()? {
            let seq = b.u32()?;
            let node = b.string()?;
            let speaker = b.string()?;
            let text = b.string()?;
            let chosen = b.u8()? != 0;
            let mut spans = Vec::new();
            for _ in 0..b.u32()? {
                spans.push(read_span(&mut b)?);
            }
            lines.push(FlowTranscriptLine {
                seq,
                node,
                speaker,
                text,
                spans,
                chosen,
            });
        }
        s.history.insert(
            e,
            FlowHistory {
                next_seq,
                last_caption_ids,
                lines,
            },
        );
    }
    for _ in 0..b.u32()? {
        let e = b.u64()?;
        s.music.insert(
            e,
            MusicState {
                sound: b.string()?,
                entity: b.u64()?,
                prev_entity: b.u64()?,
                volume: b.f32()?,
                duck: b.f32()?,
                fade_ticks: b.u32()?,
                fade_left: b.u32()?,
                current_duck: b.f32()?,
            },
        );
    }
    let decl = b.u64()?;
    s.declared = (decl != 0).then_some(decl);
    if !b.is_done() {
        return Err(bad("the og.flow state has bytes left over"));
    }
    Ok(s)
}

pub(super) struct FlowBlock(pub(super) ModuleState);

impl StateBlock for FlowBlock {
    fn save(&self, out: &mut Vec<u8>) {
        save(&lock(&self.0).saved, out);
    }

    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn Any + Send>> {
        Ok(Box::new(load_state(bytes)?))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let next = decoded
            .and_then(|d| d.downcast::<Saved>().ok())
            .map(|b| *b)
            .unwrap_or_default();
        let mut rt = lock(&self.0);
        rt.saved = next;
    }

    fn form(&self, bytes: &[u8]) -> og_core::Result<og_core::block_form::BlockForm> {
        use og_core::block_form::{BlockForm, Field, Record, Records};
        let s = load_state(bytes)?;
        let mut out = Records::default();
        for (i, (e, op)) in s.ops.iter().enumerate() {
            let label = match op {
                Op::Choose(c) => json!({ "choose": c }),
                Op::Play => json!("play"),
                Op::Pause => json!("pause"),
                Op::Skip => json!("skip"),
                Op::Stop => json!("stop"),
                Op::Advance(o) => json!({ "advance": o }),
                Op::SetSpeed(sp) => json!({ "setSpeed": sp }),
                Op::Seek(t) => json!({ "seek": t }),
                Op::Inspect(id) => json!({ "inspect": id }),
                Op::SetMusic(m) => json!({ "setMusic": m.sound }),
                Op::Jump(j) => json!({ "jump": j.node }),
            };
            out.entities.push(
                Record::new(*e, "op")
                    .key(Field::of(&i))
                    .field("op", Field::Value(label)),
            );
        }
        for (i, (e, g)) in s.gestures.iter().enumerate() {
            out.entities.push(
                Record::new(*e, "gesture")
                    .key(Field::of(&i))
                    .field("kind", Field::of(&g.kind))
                    .field("direction", Field::of(&g.direction))
                    .field("progress", Field::of(&g.progress)),
            );
        }
        for (&e, set) in &s.inspected {
            out.entities.push(
                Record::new(e, "inspected")
                    .field("hotspots", Field::of(&set.iter().collect::<Vec<_>>())),
            );
        }
        for (&e, h) in &s.history {
            out.entities.push(
                Record::new(e, "history")
                    .field("nextSeq", Field::of(&h.next_seq))
                    .field("count", Field::of(&h.lines.len())),
            );
        }
        for (&e, m) in &s.music {
            out.entities.push(
                Record::new(e, "music")
                    .field("sound", Field::of(&m.sound))
                    .field("entity", Field::Entity(m.entity))
                    .field("prevEntity", Field::Entity(m.prev_entity))
                    .field("volume", Field::of(&m.volume))
                    .field("duck", Field::of(&m.duck))
                    .field("fadeTicks", Field::of(&m.fade_ticks))
                    .field("fadeLeft", Field::of(&m.fade_left))
                    .field("currentDuck", Field::of(&m.current_duck)),
            );
        }
        out.global
            .push(("declared".into(), Field::Entity(s.declared.unwrap_or(0))));
        Ok(BlockForm::Entities(out))
    }

    fn shift(&mut self, _: &og_ecs::Shift) {}
}
