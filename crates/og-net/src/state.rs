//! A world snapshot read without a world, for what lockstep compares.
//!
//! og-ecs writes snapshots in format 2 (3 with state blocks, 4 taken
//! mid-tick), each type with its field table, so a snapshot describes itself:
//! [`Snapshot::decode`] reads one into entities with their components'
//! packed bytes, resources, event queues, names and state blocks, and
//! [`field_json`] reads any field by its type. Format 1 (no field table) is
//! refused: lockstep only reads snapshots this engine just took.
//!
//! Two peers' worlds are the same simulation when everything but each
//! peer's own session is equal. [`Snapshot::shared_hash`] is FNV-1a over the
//! snapshot with the session left out: the resources that are each peer's
//! own ([`SESSION_RESOURCES`]: input, the player's audio volumes, the view,
//! og.Lockstep with its `localPlayer`), the events each host delivers on its
//! own (input, room, platform answers, asset loads, og.Desync:
//! [`SESSION_EVENTS`]) and the UI's interaction state ([`SESSION_BLOCKS`]:
//! focus and hover follow the local pointer). Everything else (every
//! entity and component, the entity allocator, the other resources with
//! og.Time and og.Rng, the other events, names, and the modules' state:
//! physics, animation, tweens, AI, navigation) is hashed, in snapshot order.
//! [`diff`] compares two snapshots the same way and names every entity,
//! component, field, resource and state block that differs.

use og_core::hash::StableHasher;
use og_core::reflect::{FieldSchema, FieldType, Layout, Schema};
use serde_json::{Map, Value, json};

/// Resources that are each peer's own: never hashed, never diffed.
pub const SESSION_RESOURCES: &[&str] = &[
    "og.Input",
    "og.Pointers",
    "og.Gamepad",
    "og.Audio",
    "og.Camera",
    "og.Lockstep",
];

/// Events each host delivers on its own timeline.
pub const SESSION_EVENTS: &[&str] = &[
    "og.KeyDown",
    "og.KeyUp",
    "og.TextInput",
    "og.PointerDown",
    "og.PointerUp",
    "og.PointerMove",
    "og.Wheel",
    "og.A11yAction",
    "og.RoomJoined",
    "og.RoomPeerJoined",
    "og.RoomPeerLeft",
    "og.RoomMessage",
    "og.Desync",
    "og.PlatformAnswered",
    "og.AssetReady",
    "og.AssetFailed",
    "og.StreamEvent",
    "og.WorldRestored",
];

/// State blocks that follow the local player: the UI's focus and hover.
pub const SESSION_BLOCKS: &[&str] = &["og.ui"];

/// The most differences [`diff`] lists (it says how many more there were).
pub const MAX_DIFFERENCES: usize = 200;

const MAGIC: &[u8; 4] = b"OGWS";

/// One registered type, with its field table and packed layout.
#[derive(Clone, Debug)]
pub struct Type {
    pub name: String,
    pub hash: u64,
    pub schema: Schema,
    pub layout: Layout,
}

/// One live entity: its id bits and its components (type index, packed bytes).
#[derive(Clone, Debug)]
pub struct Row {
    pub entity: u64,
    pub components: Vec<(usize, Vec<u8>)>,
}

/// One event queue: the events delivered this tick and those for the next.
#[derive(Clone, Debug)]
pub struct Queue {
    pub ty: usize,
    pub current: Vec<Vec<u8>>,
    pub pending: Vec<Vec<u8>>,
}

/// A decoded snapshot.
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub format: u32,
    pub types: Vec<Type>,
    /// Every allocator slot: (generation, alive).
    pub slots: Vec<(u32, bool)>,
    pub free: Vec<u32>,
    /// Live entities in storage order (archetype creation order, then rows).
    pub rows: Vec<Row>,
    pub resources: Vec<(usize, Vec<u8>)>,
    pub events: Vec<Queue>,
    pub names: Vec<(u64, String)>,
    pub blocks: Vec<(String, Vec<u8>)>,
    /// The system a mid-tick snapshot (format 4) was taken in.
    pub resume: Option<String>,
}

struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self
            .pos
            .checked_add(n)
            .filter(|&e| e <= self.buf.len())
            .ok_or_else(|| format!("snapshot: truncated at byte {}", self.pos))?;
        let out = &self.buf[self.pos..end];
        self.pos = end;
        Ok(out)
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<usize, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().expect("4 bytes")) as usize)
    }
    fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("8 bytes"),
        ))
    }
    fn count(&mut self, min_item: usize) -> Result<usize, String> {
        let n = self.u32()?;
        if n.saturating_mul(min_item) > self.buf.len() - self.pos {
            return Err(format!("snapshot: count {n} exceeds the data"));
        }
        Ok(n)
    }
    fn sized(&mut self) -> Result<&'a [u8], String> {
        let n = self.u32()?;
        self.take(n)
    }
    fn str(&mut self) -> Result<String, String> {
        String::from_utf8(self.sized()?.to_vec()).map_err(|_| "snapshot: text is not UTF-8".into())
    }
}

impl Snapshot {
    /// Reads a snapshot of format 2, 3 or 4.
    pub fn decode(bytes: &[u8]) -> Result<Snapshot, String> {
        let mut r = Reader { buf: bytes, pos: 0 };
        if r.take(4)? != MAGIC {
            return Err("not a world snapshot".into());
        }
        let format = r.u32()? as u32;
        if !(2..=4).contains(&format) {
            return Err(format!(
                "snapshot format {format}: lockstep reads formats 2 to 4, which carry their field tables"
            ));
        }
        let n = r.count(12)?;
        let mut types = Vec::with_capacity(n);
        for _ in 0..n {
            let name = r.str()?;
            let hash = r.u64()?;
            let version = r.u32()? as u32;
            let fields = r.count(12)?;
            let mut list = Vec::with_capacity(fields);
            for _ in 0..fields {
                let fname = r.str()?;
                let tname = r.str()?;
                let ty = FieldType::parse(&tname)
                    .ok_or_else(|| format!("snapshot: '{name}.{fname}' has type '{tname}'"))?;
                let k = r.count(4)?;
                let mut f = FieldSchema::new(fname, ty);
                for _ in 0..k {
                    f.variants.push(r.str()?);
                }
                list.push(f);
            }
            let schema = Schema {
                name: name.clone(),
                version,
                doc: String::new(),
                fields: list,
            };
            let layout = Layout::of(&schema);
            types.push(Type {
                name,
                hash,
                schema,
                layout,
            });
        }
        let type_at = |i: usize| -> Result<usize, String> {
            if i < types.len() {
                Ok(i)
            } else {
                Err(format!("snapshot: type index {i} out of range"))
            }
        };
        let slot_count = r.count(5)?;
        let mut slots = Vec::with_capacity(slot_count);
        for _ in 0..slot_count {
            let generation = r.u32()? as u32;
            slots.push((generation, r.u8()? == 1));
        }
        let free_count = r.count(4)?;
        let mut free = Vec::with_capacity(free_count);
        for _ in 0..free_count {
            free.push(r.u32()? as u32);
        }
        let arch_count = r.count(8)?;
        let mut rows = Vec::new();
        for _ in 0..arch_count {
            let k = r.count(4)?;
            let mut stored = Vec::with_capacity(k);
            for _ in 0..k {
                stored.push(type_at(r.u32()?)?);
            }
            let count = r.count(8)?;
            let first = rows.len();
            for _ in 0..count {
                rows.push(Row {
                    entity: r.u64()?,
                    components: stored.iter().map(|&t| (t, Vec::new())).collect(),
                });
            }
            for (c, &t) in stored.iter().enumerate() {
                let layout = &types[t].layout;
                for f in 0..layout.field_count() {
                    let size = layout.range(f).len();
                    let column = r.take(count * size)?;
                    for (i, row) in rows[first..].iter_mut().enumerate() {
                        row.components[c]
                            .1
                            .extend_from_slice(&column[i * size..(i + 1) * size]);
                    }
                }
            }
        }
        let n = r.count(8)?;
        let mut resources = Vec::with_capacity(n);
        for _ in 0..n {
            let t = type_at(r.u32()?)?;
            resources.push((t, r.sized()?.to_vec()));
        }
        let n = r.count(12)?;
        let mut events = Vec::with_capacity(n);
        for _ in 0..n {
            let ty = type_at(r.u32()?)?;
            let size = types[ty].layout.size();
            let mut lists = [Vec::new(), Vec::new()];
            for list in &mut lists {
                let k = r.count(size)?;
                for _ in 0..k {
                    list.push(r.take(size)?.to_vec());
                }
            }
            let [current, pending] = lists;
            events.push(Queue {
                ty,
                current,
                pending,
            });
        }
        let n = r.count(12)?;
        let mut names = Vec::with_capacity(n);
        for _ in 0..n {
            let e = r.u64()?;
            names.push((e, r.str()?));
        }
        let mut blocks = Vec::new();
        if format >= 3 {
            let n = r.count(8)?;
            for _ in 0..n {
                let name = r.str()?;
                blocks.push((name, r.sized()?.to_vec()));
            }
        }
        let resume = if format == 4 { Some(r.str()?) } else { None };
        if r.pos != bytes.len() {
            return Err(format!("snapshot: {} trailing bytes", bytes.len() - r.pos));
        }
        Ok(Snapshot {
            format,
            types,
            slots,
            free,
            rows,
            resources,
            events,
            names,
            blocks,
            resume,
        })
    }

    fn type_name(&self, t: usize) -> &str {
        &self.types[t].name
    }

    /// The hash two peers compare: the snapshot without each peer's session
    /// (module docs).
    pub fn shared_hash(&self) -> u64 {
        let mut h = StableHasher::new();
        h.write_u32(self.types.len() as u32);
        for t in &self.types {
            h.write_str(&t.name);
            h.write_u64(t.hash);
        }
        h.write_u32(self.slots.len() as u32);
        for (g, alive) in &self.slots {
            h.write_u32(*g);
            h.write(&[u8::from(*alive)]);
        }
        h.write_u32(self.free.len() as u32);
        for f in &self.free {
            h.write_u32(*f);
        }
        h.write_u32(self.rows.len() as u32);
        for row in &self.rows {
            h.write_u64(row.entity);
            for (t, bytes) in &row.components {
                h.write_u32(*t as u32);
                h.write(bytes);
            }
        }
        for (t, bytes) in &self.resources {
            if SESSION_RESOURCES.contains(&self.type_name(*t)) {
                continue;
            }
            h.write_str(self.type_name(*t));
            h.write(bytes);
        }
        for q in &self.events {
            if SESSION_EVENTS.contains(&self.type_name(q.ty)) {
                continue;
            }
            h.write_str(self.type_name(q.ty));
            for list in [&q.current, &q.pending] {
                h.write_u32(list.len() as u32);
                for e in list {
                    h.write(e);
                }
            }
        }
        for (e, name) in &self.names {
            h.write_u64(*e);
            h.write_str(name);
        }
        for (name, bytes) in &self.blocks {
            if SESSION_BLOCKS.contains(&name.as_str()) {
                continue;
            }
            h.write_str(name);
            h.write_u64(bytes.len() as u64);
            h.write(bytes);
        }
        if let Some(s) = &self.resume {
            h.write_str(s);
        }
        h.finish()
    }

    /// An entity's probe id: its name, or `#index.generation`.
    pub fn entity_id(&self, entity: u64) -> String {
        match self.names.iter().find(|(e, _)| *e == entity) {
            Some((_, n)) => n.clone(),
            None => format!("#{}.{}", entity as u32, entity >> 32),
        }
    }

    /// One component's fields as JSON, by name.
    pub fn record_json(&self, t: usize, bytes: &[u8]) -> Value {
        let ty = &self.types[t];
        let mut out = Map::new();
        for (i, f) in ty.schema.fields.iter().enumerate() {
            let r = ty.layout.range(i);
            if let Some(b) = bytes.get(r) {
                out.insert(f.name.clone(), field_json(f, b));
            }
        }
        Value::Object(out)
    }
}

fn f32_at(b: &[u8], i: usize) -> f32 {
    f32::from_le_bytes(b[i * 4..i * 4 + 4].try_into().expect("4 bytes"))
}

fn num(x: f32) -> Value {
    if x.is_finite() {
        json!(x)
    } else {
        json!(x.to_string())
    }
}

/// A field's packed bytes as JSON: numbers, vectors as lists, a bool, an
/// enum's variant name, text up to its first zero byte.
pub fn field_json(f: &FieldSchema, b: &[u8]) -> Value {
    match f.ty {
        FieldType::F32 => num(f32_at(b, 0)),
        FieldType::F64 => {
            let x = f64::from_le_bytes(b[..8].try_into().expect("8 bytes"));
            if x.is_finite() {
                json!(x)
            } else {
                json!(x.to_string())
            }
        }
        FieldType::I32 => json!(i32::from_le_bytes(b[..4].try_into().expect("4 bytes"))),
        FieldType::U32 => json!(u32::from_le_bytes(b[..4].try_into().expect("4 bytes"))),
        FieldType::I64 => json!(i64::from_le_bytes(b[..8].try_into().expect("8 bytes"))),
        FieldType::U64 => json!(u64::from_le_bytes(b[..8].try_into().expect("8 bytes"))),
        FieldType::Vec2 | FieldType::Vec3 | FieldType::Vec4 | FieldType::Quat | FieldType::Mat4 => {
            Value::Array((0..b.len() / 4).map(|i| num(f32_at(b, i))).collect())
        }
        FieldType::Bool => json!(b[0] != 0),
        FieldType::Enum => {
            let i = u32::from_le_bytes(b[..4].try_into().expect("4 bytes")) as usize;
            match f.variants.get(i) {
                Some(v) => json!(v),
                None => json!(i),
            }
        }
        FieldType::Str(_) => {
            let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
            json!(String::from_utf8_lossy(&b[..end]))
        }
    }
}

/// The fields of one record that differ, as `{ field, local, remote }`.
fn field_changes(
    a: &Snapshot,
    ta: usize,
    ba: &[u8],
    b: &Snapshot,
    tb: usize,
    bb: &[u8],
) -> Vec<Value> {
    let (sa, sb) = (&a.types[ta], &b.types[tb]);
    let mut out = Vec::new();
    for (i, f) in sa.schema.fields.iter().enumerate() {
        let ra = sa.layout.range(i);
        let other = sb.schema.fields.iter().position(|g| g.name == f.name);
        match other {
            Some(j) => {
                let rb = sb.layout.range(j);
                let (x, y) = (&ba[ra], &bb[rb]);
                if x != y {
                    out.push(json!({
                        "field": f.name,
                        "local": field_json(f, x),
                        "remote": field_json(&sb.schema.fields[j], y),
                    }));
                }
            }
            None => out
                .push(json!({ "field": f.name, "local": field_json(f, &ba[ra]), "remote": null })),
        }
    }
    out
}

/// What differs between this peer's snapshot (`local`) and another's
/// (`remote`) of the same tick, leaving out each peer's session:
///
/// ```text
/// { differs, components: [{ entity, component, fields: [{ field, local, remote }] }],
///   entities: { onlyLocal: [id], onlyRemote: [id] },
///   resources: [{ resource, fields: [...] }],
///   events: [{ event, local, remote }]    (queued counts)
///   blocks: [{ block, localBytes, remoteBytes }]   (module state: physics, animation, ...)
///   allocator: bool, truncated: n }
/// ```
///
/// Components are listed in the local snapshot's entity order, so the first
/// one is the first that differs in storage order.
pub fn diff(local: &Snapshot, remote: &Snapshot) -> Value {
    let mut components = Vec::new();
    let mut truncated = 0usize;
    let mut push = |list: &mut Vec<Value>, v: Value| {
        if list.len() < MAX_DIFFERENCES {
            list.push(v);
        } else {
            truncated += 1;
        }
    };
    fn by_entity(s: &Snapshot, e: u64) -> Option<&Row> {
        s.rows.iter().find(|r| r.entity == e)
    }
    let mut only_local = Vec::new();
    for row in &local.rows {
        let Some(other) = by_entity(remote, row.entity) else {
            only_local.push(json!(local.entity_id(row.entity)));
            continue;
        };
        for (t, bytes) in &row.components {
            let name = local.type_name(*t);
            let theirs = other
                .components
                .iter()
                .find(|(u, _)| remote.type_name(*u) == name);
            match theirs {
                Some((u, b2)) => {
                    let fields = field_changes(local, *t, bytes, remote, *u, b2);
                    if !fields.is_empty() {
                        push(
                            &mut components,
                            json!({ "entity": local.entity_id(row.entity), "component": name, "fields": fields }),
                        );
                    }
                }
                None => push(
                    &mut components,
                    json!({ "entity": local.entity_id(row.entity), "component": name, "only": "local", "local": local.record_json(*t, bytes) }),
                ),
            }
        }
        for (u, b2) in &other.components {
            let name = remote.type_name(*u);
            if !row
                .components
                .iter()
                .any(|(t, _)| local.type_name(*t) == name)
            {
                push(
                    &mut components,
                    json!({ "entity": local.entity_id(row.entity), "component": name, "only": "remote", "remote": remote.record_json(*u, b2) }),
                );
            }
        }
    }
    let only_remote: Vec<Value> = remote
        .rows
        .iter()
        .filter(|r| by_entity(local, r.entity).is_none())
        .map(|r| json!(remote.entity_id(r.entity)))
        .collect();
    let mut resources = Vec::new();
    for (t, bytes) in &local.resources {
        let name = local.type_name(*t);
        if SESSION_RESOURCES.contains(&name) {
            continue;
        }
        if let Some((u, b2)) = remote
            .resources
            .iter()
            .find(|(u, _)| remote.type_name(*u) == name)
        {
            let fields = field_changes(local, *t, bytes, remote, *u, b2);
            if !fields.is_empty() {
                resources.push(json!({ "resource": name, "fields": fields }));
            }
        } else {
            resources.push(json!({ "resource": name, "only": "local" }));
        }
    }
    let mut events = Vec::new();
    for q in &local.events {
        let name = local.type_name(q.ty);
        if SESSION_EVENTS.contains(&name) {
            continue;
        }
        let theirs = remote
            .events
            .iter()
            .find(|r| remote.type_name(r.ty) == name);
        let same = theirs.is_some_and(|r| r.current == q.current && r.pending == q.pending);
        if !same {
            let count = |r: Option<&Queue>| r.map_or(0, |r| r.current.len() + r.pending.len());
            events.push(json!({ "event": name, "local": count(Some(q)), "remote": count(theirs) }));
        }
    }
    let mut blocks = Vec::new();
    for (name, bytes) in &local.blocks {
        if SESSION_BLOCKS.contains(&name.as_str()) {
            continue;
        }
        let theirs = remote
            .blocks
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, b)| b);
        if theirs != Some(bytes) {
            blocks.push(json!({ "block": name, "localBytes": bytes.len(), "remoteBytes": theirs.map(Vec::len) }));
        }
    }
    let allocator = local.slots != remote.slots || local.free != remote.free;
    let differs = !components.is_empty()
        || !only_local.is_empty()
        || !only_remote.is_empty()
        || !resources.is_empty()
        || !events.is_empty()
        || !blocks.is_empty()
        || allocator;
    json!({
        "differs": differs,
        "components": components,
        "entities": { "onlyLocal": only_local, "onlyRemote": only_remote },
        "resources": resources,
        "events": events,
        "blocks": blocks,
        "allocator": allocator,
        "truncated": truncated,
    })
}
