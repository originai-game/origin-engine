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
use std::collections::BTreeMap;

/// Resources that are each peer's own: never hashed, never diffed.
pub const SESSION_RESOURCES: &[&str] = &[
    "og.Input",
    "og.Pointers",
    "og.Gamepad",
    "og.PointerLock",
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

    /// The portable hash's domain this snapshot spans: every type's name
    /// with its field names, in snapshot order (`og replay record` keeps it
    /// in the trace, `og replay compare` hashes a newer engine's world
    /// within it).
    pub fn portable_table(&self) -> BTreeMap<String, Vec<String>> {
        self.types
            .iter()
            .map(|t| {
                let fields = t.schema.fields.iter().map(|f| f.name.clone()).collect();
                (t.name.clone(), fields)
            })
            .collect()
    }

    /// One record's fields within `fields`, each by name with its bytes; a
    /// field the table names and this type no longer has hashes as missing.
    fn portable_record(&self, h: &mut StableHasher, t: usize, bytes: &[u8], fields: &[String]) {
        let ty = &self.types[t];
        for name in fields {
            h.write_str(name);
            match ty.schema.fields.iter().position(|f| &f.name == name) {
                Some(i) => {
                    let b = bytes.get(ty.layout.range(i)).unwrap_or(&[]);
                    h.write_u64(b.len() as u64);
                    h.write(b);
                }
                None => h.write_u64(u64::MAX),
            }
        }
    }

    /// [`Snapshot::shared_hash`] within `table` (a [`Snapshot::portable_table`]),
    /// independent of layout hashes and storage order: two engines' worlds
    /// hash equal when every type and field the table names holds the same
    /// bytes, whatever types or fields the newer one added; one the table
    /// names that is gone hashes differently. Entities by id, their
    /// components by type name, fields by name; resources, event queues and
    /// state blocks by name (each peer's session left out); names; the
    /// entity allocator.
    pub fn portable_hash(&self, table: &BTreeMap<String, Vec<String>>) -> u64 {
        let mut h = StableHasher::new();
        let mut rows: Vec<&Row> = self.rows.iter().collect();
        rows.sort_by_key(|r| r.entity);
        h.write_u32(rows.len() as u32);
        for row in rows {
            h.write_u64(row.entity);
            let mut parts: Vec<(&str, &[String], usize, &[u8])> = row
                .components
                .iter()
                .filter_map(|(t, b)| {
                    let name = self.type_name(*t);
                    table
                        .get(name)
                        .map(|f| (name, f.as_slice(), *t, b.as_slice()))
                })
                .collect();
            parts.sort_by_key(|p| p.0);
            h.write_u32(parts.len() as u32);
            for (name, fields, t, bytes) in parts {
                h.write_str(name);
                self.portable_record(&mut h, t, bytes, fields);
            }
        }
        let mut resources: Vec<(&str, &[String], usize, &[u8])> = self
            .resources
            .iter()
            .filter_map(|(t, b)| {
                let name = self.type_name(*t);
                if SESSION_RESOURCES.contains(&name) {
                    return None;
                }
                table
                    .get(name)
                    .map(|f| (name, f.as_slice(), *t, b.as_slice()))
            })
            .collect();
        resources.sort_by_key(|p| p.0);
        h.write_u32(resources.len() as u32);
        for (name, fields, t, bytes) in resources {
            h.write_str(name);
            self.portable_record(&mut h, t, bytes, fields);
        }
        let mut events: Vec<(&str, &[String], &Queue)> = self
            .events
            .iter()
            .filter_map(|q| {
                let name = self.type_name(q.ty);
                if SESSION_EVENTS.contains(&name) {
                    return None;
                }
                table.get(name).map(|f| (name, f.as_slice(), q))
            })
            .collect();
        events.sort_by_key(|p| p.0);
        h.write_u32(events.len() as u32);
        for (name, fields, q) in events {
            h.write_str(name);
            for list in [&q.current, &q.pending] {
                h.write_u32(list.len() as u32);
                for e in list {
                    self.portable_record(&mut h, q.ty, e, fields);
                }
            }
        }
        let mut blocks: Vec<&(String, Vec<u8>)> = self
            .blocks
            .iter()
            .filter(|(n, _)| !SESSION_BLOCKS.contains(&n.as_str()))
            .collect();
        blocks.sort_by(|a, b| a.0.cmp(&b.0));
        h.write_u32(blocks.len() as u32);
        for (name, bytes) in blocks {
            h.write_str(name);
            h.write_u64(bytes.len() as u64);
            h.write(bytes);
        }
        let mut names: Vec<&(u64, String)> = self.names.iter().collect();
        names.sort();
        h.write_u32(names.len() as u32);
        for (e, name) in names {
            h.write_u64(*e);
            h.write_str(name);
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
/// enum's variant name, text up to its first zero byte, a list field's
/// elements up to its length.
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
        // The elements up to its length, each as its number type alone.
        FieldType::List(element, _) => Value::Array(
            og_core::list_elements(f.ty, b)
                .map(|e| field_json(&FieldSchema::new("", element.field_type()), e))
                .collect(),
        ),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn ty(name: &str, fields: &[&str]) -> Type {
        let schema = Schema {
            name: name.into(),
            version: 1,
            doc: String::new(),
            fields: fields
                .iter()
                .map(|f| FieldSchema::new(*f, FieldType::F32))
                .collect(),
        };
        Type {
            name: name.into(),
            hash: og_core::hash::hash_bytes(fields.join(",").as_bytes()),
            layout: Layout::of(&schema),
            schema,
        }
    }

    fn floats(xs: &[f32]) -> Vec<u8> {
        xs.iter().flat_map(|x| x.to_le_bytes()).collect()
    }

    /// Two entities with a Position each (one with a Speed too) and an
    /// og.Time-like resource, in the older engine's types.
    fn world() -> Snapshot {
        Snapshot {
            format: 3,
            types: vec![
                ty("Position", &["x", "y"]),
                ty("Speed", &["v"]),
                ty("Clock", &["t"]),
            ],
            slots: vec![(0, true), (0, true)],
            free: vec![],
            rows: vec![
                Row {
                    entity: 0,
                    components: vec![(0, floats(&[1.0, 2.0]))],
                },
                Row {
                    entity: 1,
                    components: vec![(0, floats(&[3.0, 4.0])), (1, floats(&[5.0]))],
                },
            ],
            resources: vec![(2, floats(&[0.5]))],
            events: vec![],
            names: vec![(1, "player".into())],
            blocks: vec![("og.physics".into(), vec![1, 2, 3])],
            resume: None,
        }
    }

    #[test]
    fn a_field_the_table_does_not_name_leaves_the_portable_hash_alone() {
        let old = world();
        let table = old.portable_table();
        // The newer engine's Position gains `z` (between x and y) and Speed's hash moves.
        let mut new = world();
        new.types[0] = ty("Position", &["x", "z", "y"]);
        new.types[1].hash ^= 1;
        new.rows[0].components[0].1 = floats(&[1.0, 9.0, 2.0]);
        new.rows[1].components[0].1 = floats(&[3.0, 7.0, 4.0]);
        assert_eq!(old.portable_hash(&table), new.portable_hash(&table));
        assert_ne!(old.shared_hash(), new.shared_hash());
        // Within the newer table, `z` counts.
        assert_ne!(
            old.portable_hash(&new.portable_table()),
            new.portable_hash(&new.portable_table())
        );
    }

    #[test]
    fn a_table_fields_value_or_a_removed_field_moves_the_portable_hash() {
        let old = world();
        let table = old.portable_table();
        let mut moved = world();
        moved.rows[1].components[0].1 = floats(&[3.0, 4.5]);
        assert_ne!(old.portable_hash(&table), moved.portable_hash(&table));
        let mut resource = world();
        resource.resources[0].1 = floats(&[0.75]);
        assert_ne!(old.portable_hash(&table), resource.portable_hash(&table));
        let mut block = world();
        block.blocks[0].1[0] = 9;
        assert_ne!(old.portable_hash(&table), block.portable_hash(&table));
        // Position loses `y`: the table still names it.
        let mut removed = world();
        removed.types[0] = ty("Position", &["x"]);
        removed.rows[0].components[0].1 = floats(&[1.0]);
        removed.rows[1].components[0].1 = floats(&[3.0]);
        assert_ne!(old.portable_hash(&table), removed.portable_hash(&table));
    }

    #[test]
    fn storage_order_leaves_the_portable_hash_alone() {
        let old = world();
        let table = old.portable_table();
        // Types registered in another order, archetypes and rows stored in
        // another order, an entity's components in another order.
        let mut new = world();
        new.types = vec![
            ty("Clock", &["t"]),
            ty("Speed", &["v"]),
            ty("Position", &["x", "y"]),
        ];
        new.rows = vec![
            Row {
                entity: 1,
                components: vec![(1, floats(&[5.0])), (2, floats(&[3.0, 4.0]))],
            },
            Row {
                entity: 0,
                components: vec![(2, floats(&[1.0, 2.0]))],
            },
        ];
        new.resources = vec![(0, floats(&[0.5]))];
        assert_eq!(old.portable_hash(&table), new.portable_hash(&table));
        assert_ne!(old.shared_hash(), new.shared_hash());
        // A component moved to the other entity is a different world.
        let mut swapped = new.clone();
        swapped.rows[0].components.remove(0);
        swapped.rows[1].components.push((1, floats(&[5.0])));
        assert_ne!(old.portable_hash(&table), swapped.portable_hash(&table));
    }

    #[test]
    fn a_list_field_reads_as_its_elements_up_to_its_length() {
        let ty = FieldType::list(og_core::Scalar::U64, 4).unwrap();
        let f = FieldSchema::new("passes", ty);
        let (seven, zero) = (7u64.to_le_bytes(), 0u64.to_le_bytes());
        let two = og_core::pack_list(ty, &[&seven, &zero]).unwrap();
        assert_eq!(field_json(&f, &two), json!([7, 0]));
        let none = og_core::pack_list(ty, &[]).unwrap();
        assert_eq!(field_json(&f, &none), json!([]));
        let ty = FieldType::list(og_core::Scalar::F32, 2).unwrap();
        let half = 0.5f32.to_le_bytes();
        let one = og_core::pack_list(ty, &[&half]).unwrap();
        assert_eq!(field_json(&FieldSchema::new("w", ty), &one), json!([0.5]));
    }
}
