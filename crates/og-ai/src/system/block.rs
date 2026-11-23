//! The `og.ai` state block: memories, blackboards and pending noises.

use super::*;
use og_core::bytes::{ByteReader, ByteWriter};

pub(super) struct AiBlock(pub(super) AiSystem);

/// The og.ai trailer's tag for game actions' pending results (G2-4).
const TAG_RESULTS: u16 = 1;
/// The trailer's section of [`State::origin`], when it is not zero.
const TAG_ORIGIN: u16 = 2;

#[derive(Debug)]
struct Saved {
    mems: IdMap<u64, AgentMem>,
    forward: IdMap<u64, Vec3>,
    seen: IdMap<u64, u64>,
    heard: IdMap<u64, u64>,
    noises: Vec<Noise>,
    origin: Vec3,
}

fn put_value(w: &mut ByteWriter<'_>, v: Value) {
    match v {
        Value::Num(n) => {
            w.u8(0);
            w.f64(n);
        }
        Value::Bool(b) => {
            w.u8(1);
            w.bool(b);
        }
        Value::Vec(v) => {
            w.u8(2);
            w.vec3(v);
        }
        Value::Entity(e) => {
            w.u8(3);
            w.u64(e);
        }
    }
}

fn read_value(r: &mut ByteReader<'_>) -> Result<Value> {
    Ok(match r.u8()? {
        0 => Value::Num(r.f64()?),
        1 => Value::Bool(r.bool()?),
        2 => Value::Vec(r.vec3()?),
        _ => Value::Entity(r.u64()?),
    })
}

impl StateBlock for AiBlock {
    fn save(&self, o: &mut Vec<u8>) {
        let st = lock(&self.0.state);
        let mut w = ByteWriter::new(o);
        w.u32(st.mems.len() as u32);
        for (k, m) in &st.mems {
            w.u64(*k);
            w.u64(m.behaviour);
            w.bool(m.started);
            w.bool(m.entered);
            w.u32(m.state);
            w.u64(m.state_since);
            w.u32(m.nodes.len() as u32);
            for n in &m.nodes {
                w.bool(n.running);
                w.u32(n.index);
                w.u32(n.count);
                w.u64(n.since);
                w.u64(n.until);
                w.vec3(n.point);
            }
            w.u32(m.keys.len() as u32);
            for (k, v) in &m.keys {
                w.str(k);
                put_value(&mut w, *v);
            }
            w.vec3(m.home);
            w.f32(m.base_speed);
            w.u32(m.leaf);
            w.u32(m.status);
            w.u32(m.path.len() as u32);
            for p in &m.path {
                w.u32(*p);
            }
        }
        for map in [&st.seen_tick, &st.heard_tick] {
            w.u32(map.len() as u32);
            for (k, v) in map {
                w.u64(*k);
                w.u64(*v);
            }
        }
        w.u32(st.forward.len() as u32);
        for (k, v) in &st.forward {
            w.u64(*k);
            w.vec3(*v);
        }
        w.u32(st.noises.len() as u32);
        for n in &st.noises {
            w.vec3(n.position);
            w.f32(n.loudness);
            w.u64(n.source);
        }
        // The trailer (og_ecs::trailer), written only when a section has
        // something: tag 1, game actions the game finished that have not
        // answered yet.
        let pending: Vec<(&u64, &AgentMem)> = st
            .mems
            .iter()
            .filter(|(_, m)| !m.results.is_empty())
            .collect();
        let mut sections = Vec::new();
        if !pending.is_empty() {
            let mut b = Vec::new();
            let mut bw = ByteWriter::new(&mut b);
            bw.u32(pending.len() as u32);
            for (k, m) in pending {
                bw.u64(*k);
                bw.u32(m.results.len() as u32);
                for (node, ok) in &m.results {
                    bw.u32(*node);
                    bw.bool(*ok);
                }
            }
            sections.push((TAG_RESULTS, b));
        }
        if st.origin != Vec3::ZERO {
            let mut b = Vec::new();
            let mut bw = ByteWriter::new(&mut b);
            bw.vec3(st.origin);
            sections.push((TAG_ORIGIN, b));
        }
        og_ecs::trailer::write(o, &sections);
    }

    fn decode(&self, bytes: &[u8]) -> Result<Box<dyn Any + Send>> {
        let mut r = ByteReader::new(bytes, "the og.ai state block");
        let mut mems = IdMap::new();
        for _ in 0..r.u32()? {
            let k = r.u64()?;
            let mut m = AgentMem {
                behaviour: r.u64()?,
                started: r.bool()?,
                entered: r.bool()?,
                state: r.u32()?,
                state_since: r.u64()?,
                ..AgentMem::default()
            };
            for _ in 0..r.u32()? {
                m.nodes.push(NodeMem {
                    running: r.bool()?,
                    index: r.u32()?,
                    count: r.u32()?,
                    since: r.u64()?,
                    until: r.u64()?,
                    point: r.vec3()?,
                });
            }
            for _ in 0..r.u32()? {
                let key = r.string()?;
                let v = read_value(&mut r)?;
                m.keys.insert(key, v);
            }
            m.home = r.vec3()?;
            m.base_speed = r.f32()?;
            m.leaf = r.u32()?;
            m.status = r.u32()?;
            for _ in 0..r.u32()? {
                m.path.push(r.u32()?);
            }
            mems.insert(k, m);
        }
        let mut maps = [IdMap::new(), IdMap::new()];
        for map in &mut maps {
            for _ in 0..r.u32()? {
                let k = r.u64()?;
                map.insert(k, r.u64()?);
            }
        }
        let mut forward = IdMap::new();
        for _ in 0..r.u32()? {
            let k = r.u64()?;
            forward.insert(k, r.vec3()?);
        }
        let mut noises = Vec::new();
        for _ in 0..r.u32()? {
            noises.push(Noise {
                position: r.vec3()?,
                loudness: r.f32()?,
                source: r.u64()?,
            });
        }
        let mut origin = Vec3::ZERO;
        for (tag, body) in og_ecs::trailer::read(SYSTEM_NAME, r.rest())? {
            if tag == TAG_ORIGIN {
                origin = ByteReader::new(body, "the og.ai state block").vec3()?;
            }
            if tag == TAG_RESULTS {
                let mut r = ByteReader::new(body, "the og.ai state block");
                for _ in 0..r.u32()? {
                    let k = r.u64()?;
                    let mut results = BTreeMap::new();
                    for _ in 0..r.u32()? {
                        let node = r.u32()?;
                        results.insert(node, r.bool()?);
                    }
                    if let Some(m) = mems.get_mut(&k) {
                        m.results = results;
                    }
                }
            }
        }
        let [seen, heard] = maps;
        Ok(Box::new(Saved {
            mems,
            forward,
            seen,
            heard,
            noises,
            origin,
        }))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let mut st = lock(&self.0.state);
        match decoded.and_then(|d| d.downcast::<Saved>().ok()) {
            Some(s) => {
                st.mems = s.mems;
                st.forward = s.forward;
                st.seen_tick = s.seen;
                st.heard_tick = s.heard;
                st.noises = s.noises;
                st.origin = s.origin;
            }
            None => {
                st.origin = Vec3::ZERO;
                st.mems.clear();
                st.forward.clear();
                st.seen_tick.clear();
                st.heard_tick.clear();
                st.noises.clear();
            }
        }
        st.commands.clear();
    }

    /// Every agent's wander point, home and og.ai's own place keys, the
    /// noises not yet heard, and the offset the behaviours' authored
    /// points (routes, MoveTo) are read at. A key the game set is the
    /// game's value: og.OriginShifted says when to move it.
    fn shift(&mut self, s: &og_ecs::Shift) {
        let mut st = lock(&self.0.state);
        for m in st.mems.values_mut() {
            for n in &mut m.nodes {
                n.point = s.moved(n.point);
            }
            m.home = s.moved(m.home);
            for k in ["position", "targetPosition", "lastSeen", "heardAt"] {
                if let Some(Value::Vec(v)) = m.keys.get_mut(k) {
                    *v = s.moved(*v);
                }
            }
        }
        for n in &mut st.noises {
            n.position = s.moved(n.position);
        }
        st.origin = s.moved(st.origin);
    }

    /// Per agent: its `mind` (the behaviour handle it runs, the machine's
    /// state, each node's memory, home, speed, leaf and pending results),
    /// its `blackboard` (every key, an entity value as an entity) and its
    /// `perception` (the way it last moved, whether it ever saw or heard).
    /// A noise is a `noise` record on its source (one with none is in the
    /// global `noises`). A behaviour is named by what it parsed to, never
    /// its handle's number. The ticks a state and a running node started at
    /// and an agent last saw or heard at are [`Field::Tick`]s, a cooldown's
    /// or wait's end an [`Field::Until`]: compared relative to each world's
    /// clock, which a restart does not turn back.
    fn form(&self, bytes: &[u8]) -> Result<og_core::block_form::BlockForm> {
        use og_core::block_form::{BlockForm, Field, Record, Records};
        use serde_json::json;
        let s = self
            .decode(bytes)?
            .downcast::<Saved>()
            .expect("decoded by AiBlock::decode");
        let mut out = Records::default();
        let behaviours = lock(&self.0.behaviours);
        for (k, m) in &s.mems {
            let nodes: Vec<_> = m
                .nodes
                .iter()
                .map(|n| {
                    json!({ "running": n.running, "index": n.index, "count": n.count,
                            "point": n.point.to_array() })
                })
                .collect();
            let mut mind = Record::new(*k, "mind");
            // Each node's clock: when it started running, and until when it waits.
            for (i, n) in m.nodes.iter().enumerate() {
                let since = if n.running { n.since } else { 0 };
                mind = mind
                    .field(&format!("node {i} since"), Field::Tick(since))
                    .field(&format!("node {i} until"), Field::Until(n.until));
            }
            let behaviour = behaviours
                .name(m.behaviour)
                .unwrap_or_else(|| format!("unknown {:#x}", m.behaviour));
            out.entities.push(
                mind.field("behaviour", Field::of(&behaviour))
                    .field("stateSince", Field::Tick(m.state_since))
                    .field("started", Field::of(&m.started))
                    .field("entered", Field::of(&m.entered))
                    .field("state", Field::of(&m.state))
                    .field("nodes", Field::of(&nodes))
                    .field("home", Field::of(&m.home.to_array()))
                    .field("baseSpeed", Field::of(&m.base_speed))
                    .field("leaf", Field::of(&m.leaf))
                    .field("status", Field::of(&m.status))
                    .field("path", Field::of(&m.path))
                    .field("results", Field::of(&m.results)),
            );
            let mut board = Record::new(*k, "blackboard");
            for (name, v) in &m.keys {
                let f = match *v {
                    Value::Num(n) => Field::of(&n),
                    Value::Bool(b) => Field::of(&b),
                    Value::Vec(v) => Field::of(&v.to_array()),
                    Value::Entity(e) => Field::Entity(e),
                };
                board = board.field(name, f);
            }
            out.entities.push(board);
        }
        let mut sensed: Vec<u64> = s
            .forward
            .keys()
            .chain(s.seen.keys())
            .chain(s.heard.keys())
            .copied()
            .collect();
        sensed.sort_unstable();
        sensed.dedup();
        for k in sensed {
            out.entities.push(
                Record::new(k, "perception")
                    .field(
                        "forward",
                        Field::of(&s.forward.get(&k).map(|v| v.to_array())),
                    )
                    .field(
                        "lastSeen",
                        Field::Tick(s.seen.get(&k).copied().unwrap_or(0)),
                    )
                    .field(
                        "lastHeard",
                        Field::Tick(s.heard.get(&k).copied().unwrap_or(0)),
                    ),
            );
        }
        let mut unsourced = Vec::new();
        let mut index: BTreeMap<u64, u32> = BTreeMap::new();
        for n in &s.noises {
            let at = (n.position.to_array(), n.loudness);
            if n.source == 0 {
                unsourced.push(at);
                continue;
            }
            let i = index.entry(n.source).or_insert(0);
            out.entities.push(
                Record::new(n.source, "noise")
                    .key(Field::of(&*i))
                    .field("position", Field::of(&at.0))
                    .field("loudness", Field::of(&at.1)),
            );
            *i += 1;
        }
        out.global
            .push(("noises".to_owned(), Field::of(&unsourced)));
        out.global
            .push(("origin".to_owned(), Field::of(&s.origin.to_array())));
        Ok(BlockForm::Entities(out))
    }
}
