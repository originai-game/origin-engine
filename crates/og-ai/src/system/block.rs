//! The `og.ai` state block: memories, blackboards and pending noises.

use super::*;

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

fn put_u32(o: &mut Vec<u8>, v: u32) {
    o.extend_from_slice(&v.to_le_bytes());
}
fn put_u64(o: &mut Vec<u8>, v: u64) {
    o.extend_from_slice(&v.to_le_bytes());
}
fn put_f32(o: &mut Vec<u8>, v: f32) {
    o.extend_from_slice(&v.to_le_bytes());
}
fn put_v3(o: &mut Vec<u8>, v: Vec3) {
    for x in v.to_array() {
        put_f32(o, x);
    }
}
fn put_str(o: &mut Vec<u8>, s: &str) {
    put_u32(o, s.len() as u32);
    o.extend_from_slice(s.as_bytes());
}
fn put_value(o: &mut Vec<u8>, v: Value) {
    match v {
        Value::Num(n) => {
            o.push(0);
            o.extend_from_slice(&n.to_le_bytes());
        }
        Value::Bool(b) => {
            o.push(1);
            o.push(u8::from(b));
        }
        Value::Vec(v) => {
            o.push(2);
            put_v3(o, v);
        }
        Value::Entity(e) => {
            o.push(3);
            put_u64(o, e);
        }
    }
}

struct Rd<'a> {
    b: &'a [u8],
    at: usize,
}
impl Rd<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8]> {
        if self.at + n > self.b.len() {
            return Err(Error::new(
                ErrorKind::Invalid,
                "the og.ai state block is truncated",
            ));
        }
        let s = &self.b[self.at..self.at + n];
        self.at += n;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn f32(&mut self) -> Result<f32> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn v3(&mut self) -> Result<Vec3> {
        Ok(Vec3::new(self.f32()?, self.f32()?, self.f32()?))
    }
    fn string(&mut self) -> Result<String> {
        let n = self.u32()? as usize;
        String::from_utf8(self.take(n)?.to_vec())
            .map_err(|_| Error::new(ErrorKind::Invalid, "the og.ai state block has a bad key"))
    }
    fn value(&mut self) -> Result<Value> {
        Ok(match self.u8()? {
            0 => Value::Num(f64::from_le_bytes(self.take(8)?.try_into().unwrap())),
            1 => Value::Bool(self.u8()? != 0),
            2 => Value::Vec(self.v3()?),
            _ => Value::Entity(self.u64()?),
        })
    }
}

impl StateBlock for AiBlock {
    fn save(&self, o: &mut Vec<u8>) {
        let st = lock(&self.0.state);
        put_u32(o, st.mems.len() as u32);
        for (k, m) in &st.mems {
            put_u64(o, *k);
            put_u64(o, m.behaviour);
            o.push(u8::from(m.started));
            o.push(u8::from(m.entered));
            put_u32(o, m.state);
            put_u64(o, m.state_since);
            put_u32(o, m.nodes.len() as u32);
            for n in &m.nodes {
                o.push(u8::from(n.running));
                put_u32(o, n.index);
                put_u32(o, n.count);
                put_u64(o, n.since);
                put_u64(o, n.until);
                put_v3(o, n.point);
            }
            put_u32(o, m.keys.len() as u32);
            for (k, v) in &m.keys {
                put_str(o, k);
                put_value(o, *v);
            }
            put_v3(o, m.home);
            put_f32(o, m.base_speed);
            put_u32(o, m.leaf);
            put_u32(o, m.status);
            put_u32(o, m.path.len() as u32);
            for p in &m.path {
                put_u32(o, *p);
            }
        }
        for map in [&st.seen_tick, &st.heard_tick] {
            put_u32(o, map.len() as u32);
            for (k, v) in map {
                put_u64(o, *k);
                put_u64(o, *v);
            }
        }
        put_u32(o, st.forward.len() as u32);
        for (k, v) in &st.forward {
            put_u64(o, *k);
            put_v3(o, *v);
        }
        put_u32(o, st.noises.len() as u32);
        for n in &st.noises {
            put_v3(o, n.position);
            put_f32(o, n.loudness);
            put_u64(o, n.source);
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
            put_u32(&mut b, pending.len() as u32);
            for (k, m) in pending {
                put_u64(&mut b, *k);
                put_u32(&mut b, m.results.len() as u32);
                for (node, ok) in &m.results {
                    put_u32(&mut b, *node);
                    b.push(u8::from(*ok));
                }
            }
            sections.push((TAG_RESULTS, b));
        }
        if st.origin != Vec3::ZERO {
            let mut b = Vec::new();
            put_v3(&mut b, st.origin);
            sections.push((TAG_ORIGIN, b));
        }
        og_ecs::trailer::write(o, &sections);
    }

    fn decode(&self, bytes: &[u8]) -> Result<Box<dyn Any + Send>> {
        let mut r = Rd { b: bytes, at: 0 };
        let mut mems = IdMap::new();
        for _ in 0..r.u32()? {
            let k = r.u64()?;
            let mut m = AgentMem {
                behaviour: r.u64()?,
                started: r.u8()? != 0,
                entered: r.u8()? != 0,
                state: r.u32()?,
                state_since: r.u64()?,
                ..AgentMem::default()
            };
            for _ in 0..r.u32()? {
                m.nodes.push(NodeMem {
                    running: r.u8()? != 0,
                    index: r.u32()?,
                    count: r.u32()?,
                    since: r.u64()?,
                    until: r.u64()?,
                    point: r.v3()?,
                });
            }
            for _ in 0..r.u32()? {
                let key = r.string()?;
                let v = r.value()?;
                m.keys.insert(key, v);
            }
            m.home = r.v3()?;
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
            forward.insert(k, r.v3()?);
        }
        let mut noises = Vec::new();
        for _ in 0..r.u32()? {
            noises.push(Noise {
                position: r.v3()?,
                loudness: r.f32()?,
                source: r.u64()?,
            });
        }
        let mut origin = Vec3::ZERO;
        for (tag, body) in og_ecs::trailer::read(SYSTEM_NAME, &r.b[r.at..])? {
            if tag == TAG_ORIGIN {
                origin = Rd { b: body, at: 0 }.v3()?;
            }
            if tag == TAG_RESULTS {
                let mut r = Rd { b: body, at: 0 };
                for _ in 0..r.u32()? {
                    let k = r.u64()?;
                    let mut results = BTreeMap::new();
                    for _ in 0..r.u32()? {
                        let node = r.u32()?;
                        results.insert(node, r.u8()? != 0);
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
}
