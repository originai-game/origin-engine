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
//! [`SESSION_EVENTS`]) and the UI's interaction state and the gameplay
//! actions ([`SESSION_BLOCKS`]: focus and hover follow the local pointer,
//! actions this peer's input and bindings). Everything else (every
//! entity and component, the entity allocator, the other resources with
//! og.Time and og.Rng, the other events, names, and the modules' state:
//! physics, animation, tweens, AI, navigation) is hashed, in snapshot order.
//! [`diff`] compares two snapshots the same way and names every entity,
//! component, field, resource and state block that differs.

use og_core::bytes::ByteReader;
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
    "og.PeerStalled",
    "og.PlatformAnswered",
    "og.PlatformNotice",
    "og.AssetReady",
    "og.AssetFailed",
    "og.StreamEvent",
    "og.WorldRestored",
];

/// State blocks that follow the local player: the UI's focus and hover,
/// and the gameplay actions (read from this peer's input, with this
/// player's bindings).
pub const SESSION_BLOCKS: &[&str] = &["og.ui", "og.input.actions"];

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

/// og-core's [`ByteReader`] with its errors as text; lengths and counts as `usize`.
#[derive(Clone)]
struct Reader<'a>(ByteReader<'a>);

fn text<T>(r: og_core::Result<T>) -> Result<T, String> {
    r.map_err(|e| e.to_string())
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Reader<'a> {
        Reader(ByteReader::new(bytes, "snapshot"))
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        text(self.0.take(n))
    }
    fn u8(&mut self) -> Result<u8, String> {
        text(self.0.u8())
    }
    fn u32(&mut self) -> Result<usize, String> {
        text(self.0.length())
    }
    fn u64(&mut self) -> Result<u64, String> {
        text(self.0.u64())
    }
    fn count(&mut self, min_item: usize) -> Result<usize, String> {
        text(self.0.count(min_item))
    }
    fn sized(&mut self) -> Result<&'a [u8], String> {
        text(self.0.sized())
    }
    fn str(&mut self) -> Result<String, String> {
        text(self.0.string())
    }
}

impl Snapshot {
    /// Reads a snapshot of format 2, 3 or 4.
    pub fn decode(bytes: &[u8]) -> Result<Snapshot, String> {
        let mut r = Reader::new(bytes);
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
        if !r.0.is_done() {
            return Err(format!("snapshot: {} trailing bytes", r.0.remaining()));
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

/// A type as [`shared_hash`] reads it: its name and where its fields' sizes
/// are in the table's list.
struct HashedType<'a> {
    name: &'a str,
    fields: std::ops::Range<usize>,
    /// Packed size in bytes.
    size: usize,
}

/// The archetypes section. With a hasher, every live entity's id bits and
/// its components (type index, packed bytes) in storage order, each row's
/// fields gathered from their columns; without one, only read. Returns the
/// number of rows.
fn hash_rows(
    r: &mut Reader<'_>,
    types: &[HashedType<'_>],
    sizes: &[usize],
    mut h: Option<&mut StableHasher>,
) -> Result<usize, String> {
    let mut rows = 0;
    // One archetype's components (type, first column) and columns (the
    // bytes of every row's field, the field's size).
    let mut stored: Vec<(usize, usize)> = Vec::new();
    let mut columns: Vec<(&[u8], usize)> = Vec::new();
    let arch_count = r.count(8)?;
    for _ in 0..arch_count {
        let k = r.count(4)?;
        stored.clear();
        for _ in 0..k {
            let t = r.u32()?;
            if t >= types.len() {
                return Err(format!("snapshot: type index {t} out of range"));
            }
            stored.push((t, 0));
        }
        let count = r.count(8)?;
        let entities = r.take(count * 8)?;
        columns.clear();
        for (t, first) in &mut stored {
            *first = columns.len();
            for &size in &sizes[types[*t].fields.clone()] {
                columns.push((r.take(count * size)?, size));
            }
        }
        rows += count;
        let Some(h) = h.as_deref_mut() else { continue };
        for i in 0..count {
            h.write(&entities[i * 8..(i + 1) * 8]);
            for (c, &(t, first)) in stored.iter().enumerate() {
                h.write_u32(t as u32);
                let end = stored.get(c + 1).map_or(columns.len(), |next| next.1);
                for &(column, size) in &columns[first..end] {
                    h.write(&column[i * size..(i + 1) * size]);
                }
            }
        }
    }
    Ok(rows)
}

/// [`Snapshot::shared_hash`] of a snapshot's bytes, read in place: the same
/// value as decoding it first, without building the entities (what lockstep
/// hashes after every tick). Refuses what [`Snapshot::decode`] refuses.
pub fn shared_hash(bytes: &[u8]) -> Result<u64, String> {
    fn text(bytes: &[u8]) -> Result<&str, String> {
        std::str::from_utf8(bytes).map_err(|_| "snapshot: text is not UTF-8".into())
    }
    let mut r = Reader::new(bytes);
    if r.take(4)? != MAGIC {
        return Err("not a world snapshot".into());
    }
    let format = r.u32()? as u32;
    if !(2..=4).contains(&format) {
        return Err(format!(
            "snapshot format {format}: lockstep reads formats 2 to 4, which carry their field tables"
        ));
    }
    let mut h = StableHasher::new();
    let n = r.count(12)?;
    let mut types = Vec::with_capacity(n);
    let mut sizes = Vec::new();
    h.write_u32(n as u32);
    for _ in 0..n {
        let name = text(r.sized()?)?;
        h.write_str(name);
        h.write_u64(r.u64()?);
        let _version = r.u32()?;
        let fields = r.count(12)?;
        let first = sizes.len();
        for _ in 0..fields {
            let fname = text(r.sized()?)?;
            let tname = text(r.sized()?)?;
            let ty = FieldType::parse(tname)
                .ok_or_else(|| format!("snapshot: '{name}.{fname}' has type '{tname}'"))?;
            sizes.push(ty.size());
            for _ in 0..r.count(4)? {
                text(r.sized()?)?;
            }
        }
        types.push(HashedType {
            name,
            fields: first..sizes.len(),
            size: sizes[first..].iter().sum(),
        });
    }
    let type_at = |i: usize| -> Result<&HashedType<'_>, String> {
        types
            .get(i)
            .ok_or_else(|| format!("snapshot: type index {i} out of range"))
    };
    let slots = r.count(5)?;
    h.write_u32(slots as u32);
    for slot in r.take(slots * 5)?.as_chunks::<5>().0 {
        h.write(&slot[..4]);
        h.write(&[u8::from(slot[4] == 1)]);
    }
    let free = r.count(4)?;
    h.write_u32(free as u32);
    h.write(r.take(free * 4)?);
    // The row count leads the rows, and only the archetypes hold it.
    let rows = hash_rows(&mut r.clone(), &types, &sizes, None)?;
    h.write_u32(rows as u32);
    hash_rows(&mut r, &types, &sizes, Some(&mut h))?;
    for _ in 0..r.count(8)? {
        let name = type_at(r.u32()?)?.name;
        let value = r.sized()?;
        if SESSION_RESOURCES.contains(&name) {
            continue;
        }
        h.write_str(name);
        h.write(value);
    }
    for _ in 0..r.count(12)? {
        let ty = type_at(r.u32()?)?;
        let shared = !SESSION_EVENTS.contains(&ty.name);
        if shared {
            h.write_str(ty.name);
        }
        // The events delivered this tick, then those for the next.
        for _ in 0..2 {
            let k = r.count(ty.size)?;
            let list = r.take(k * ty.size)?;
            if shared {
                h.write_u32(k as u32);
                h.write(list);
            }
        }
    }
    for _ in 0..r.count(12)? {
        h.write_u64(r.u64()?);
        h.write_str(text(r.sized()?)?);
    }
    if format >= 3 {
        for _ in 0..r.count(8)? {
            let name = text(r.sized()?)?;
            let block = r.sized()?;
            if SESSION_BLOCKS.contains(&name) {
                continue;
            }
            h.write_str(name);
            h.write_u64(block.len() as u64);
            h.write(block);
        }
    }
    if format == 4 {
        h.write_str(text(r.sized()?)?);
    }
    if !r.0.is_done() {
        return Err(format!("snapshot: {} trailing bytes", r.0.remaining()));
    }
    Ok(h.finish())
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
        FieldType::Ease => json!(
            og_core::ease::Ease::from_bytes(b)
                .expect("validated ease")
                .to_text()
        ),
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

/// What [`restart_diff`] leaves out: a type by name, all of it (an empty
/// list) or the fields it names.
pub type Ignore = BTreeMap<String, Vec<String>>;

fn ignored<'a>(ignore: &'a Ignore, ty: &str) -> Option<&'a [String]> {
    ignore.get(ty).map(Vec::as_slice)
}

/// The entities of two worlds paired as a restart leaves them: by name,
/// then the rest by their place in their component set's storage (spawn
/// order once a set was emptied and filled again), whatever ids the
/// allocator gave them. `now_to_start` maps an entity of `now` to the one
/// of `start` it stands for; `missing` and `extra` are rows left over.
struct Pairing {
    now_to_start: std::collections::HashMap<u64, u64>,
    pairs: Vec<(usize, usize)>,
    missing: Vec<usize>,
    extra: Vec<usize>,
}

impl Snapshot {
    fn name_of(&self, entity: u64) -> Option<&str> {
        self.names
            .iter()
            .find(|(e, _)| *e == entity)
            .map(|(_, n)| n.as_str())
    }

    /// The type names of a row's components, sorted: its component set.
    fn set_of(&self, row: &Row) -> Vec<&str> {
        let mut set: Vec<&str> = row
            .components
            .iter()
            .map(|(t, _)| self.type_name(*t))
            .collect();
        set.sort_unstable();
        set
    }

    /// An entity's path: its og.Parent chain's ids joined by `/`
    /// (`level/door`, `#4.1`).
    pub fn entity_path(&self, entity: u64) -> String {
        let parent = self.types.iter().position(|t| t.name == "og.Parent");
        let mut parts = vec![self.entity_id(entity)];
        let mut at = entity;
        while let Some(p) = parent
            && parts.len() < 32
        {
            let Some(row) = self.rows.iter().find(|r| r.entity == at) else {
                break;
            };
            let Some((_, b)) = row.components.iter().find(|(t, _)| *t == p) else {
                break;
            };
            let Some(bits) = b.get(..8) else { break };
            at = u64::from_le_bytes(bits.try_into().expect("8 bytes"));
            if !self.rows.iter().any(|r| r.entity == at) {
                break;
            }
            parts.push(self.entity_id(at));
        }
        parts.reverse();
        parts.join("/")
    }
}

fn pair(start: &Snapshot, now: &Snapshot) -> Pairing {
    use std::collections::HashMap;
    let mut pairs = Vec::new();
    let (mut start_used, mut now_used) =
        (vec![false; start.rows.len()], vec![false; now.rows.len()]);
    // A name held by one entity on each side pairs them.
    let unique = |s: &Snapshot| -> HashMap<String, usize> {
        let mut seen: HashMap<String, Option<usize>> = HashMap::new();
        for (i, row) in s.rows.iter().enumerate() {
            if let Some(n) = s.name_of(row.entity) {
                seen.entry(n.to_owned())
                    .and_modify(|v| *v = None)
                    .or_insert(Some(i));
            }
        }
        seen.into_iter()
            .filter_map(|(n, i)| Some((n, i?)))
            .collect()
    };
    let now_names = unique(now);
    for (name, i) in unique(start) {
        if let Some(&j) = now_names.get(&name) {
            pairs.push((i, j));
            start_used[i] = true;
            now_used[j] = true;
        }
    }
    // The rest within their component set, in storage order.
    let mut sets: HashMap<Vec<&str>, Vec<usize>> = HashMap::new();
    for (j, row) in now.rows.iter().enumerate() {
        if !now_used[j] {
            sets.entry(now.set_of(row)).or_default().push(j);
        }
    }
    let mut next: HashMap<Vec<&str>, usize> = HashMap::new();
    for (i, row) in start.rows.iter().enumerate() {
        if start_used[i] {
            continue;
        }
        let set = start.set_of(row);
        let k = next.entry(set.clone()).or_default();
        if let Some(&j) = sets.get(&set).and_then(|l| l.get(*k)) {
            *k += 1;
            pairs.push((i, j));
            start_used[i] = true;
            now_used[j] = true;
        }
    }
    pairs.sort_unstable();
    Pairing {
        now_to_start: pairs
            .iter()
            .map(|&(i, j)| (now.rows[j].entity, start.rows[i].entity))
            .collect(),
        pairs,
        missing: (0..start.rows.len()).filter(|&i| !start_used[i]).collect(),
        extra: (0..now.rows.len()).filter(|&j| !now_used[j]).collect(),
    }
}

/// Whether a handle lane of `now` stands for the same thing as `start`'s:
/// equal bits, or an entity paired with it.
fn same_handle(start: &[u8], now: &[u8], map: &std::collections::HashMap<u64, u64>) -> bool {
    if start == now {
        return true;
    }
    let bits = |b: &[u8]| u64::from_le_bytes(b.try_into().expect("8 bytes"));
    map.get(&bits(now)) == Some(&bits(start))
}

/// The fields of one record that differ after the pairing, as
/// `{ field, start, now }`, leaving out `skip`.
fn record_changes(
    (start, ts, bs): (&Snapshot, usize, &[u8]),
    (now, tn, bn): (&Snapshot, usize, &[u8]),
    skip: &[String],
    map: &std::collections::HashMap<u64, u64>,
) -> Vec<Value> {
    let (ss, sn) = (&start.types[ts], &now.types[tn]);
    let mut out = Vec::new();
    for (i, f) in ss.schema.fields.iter().enumerate() {
        if skip.contains(&f.name) {
            continue;
        }
        let x = &bs[ss.layout.range(i)];
        let Some(j) = sn.schema.fields.iter().position(|g| g.name == f.name) else {
            out.push(json!({ "field": f.name, "start": field_json(f, x), "now": null }));
            continue;
        };
        let g = &sn.schema.fields[j];
        let y = &bn[sn.layout.range(j)];
        let same = match (f.ty, g.ty) {
            (FieldType::U64, FieldType::U64) => same_handle(x, y, map),
            (
                FieldType::List(og_core::Scalar::U64, _),
                FieldType::List(og_core::Scalar::U64, _),
            ) => {
                let (a, b): (Vec<&[u8]>, Vec<&[u8]>) = (
                    og_core::list_elements(f.ty, x).collect(),
                    og_core::list_elements(g.ty, y).collect(),
                );
                a.len() == b.len() && a.iter().zip(&b).all(|(p, q)| same_handle(p, q, map))
            }
            _ => f.ty == g.ty && x == y,
        };
        if !same {
            out.push(
                json!({ "field": f.name, "start": field_json(f, x), "now": field_json(g, y) }),
            );
        }
    }
    out
}

/// A world's structural hash, the restart check's: entities in their
/// [`pair`] order (start's order; `now` maps through `pairing`), each by its
/// name and components by type name, a handle to a live entity as its place
/// in that order; resources and event queues by name, `ignore` and each
/// peer's session left out. Entity ids, the allocator and module state
/// blocks (which hold ids of their own) are not in it.
fn structural_hash(s: &Snapshot, order: &[usize], ignore: &Ignore) -> u64 {
    use std::collections::HashMap;
    let place: HashMap<u64, u32> = order
        .iter()
        .enumerate()
        .map(|(k, &i)| (s.rows[i].entity, k as u32))
        .collect();
    let record = |h: &mut StableHasher, t: usize, bytes: &[u8], skip: &[String]| {
        let ty = &s.types[t];
        for (i, f) in ty.schema.fields.iter().enumerate() {
            if skip.contains(&f.name) {
                continue;
            }
            h.write_str(&f.name);
            let b = &bytes[ty.layout.range(i)];
            let lanes: Vec<&[u8]> = match f.ty {
                FieldType::U64 => vec![b],
                FieldType::List(og_core::Scalar::U64, _) => {
                    og_core::list_elements(f.ty, b).collect()
                }
                _ => {
                    h.write(b);
                    continue;
                }
            };
            h.write_u32(lanes.len() as u32);
            for lane in lanes {
                match place.get(&u64::from_le_bytes(lane.try_into().expect("8 bytes"))) {
                    Some(&k) => {
                        h.write(&[1]);
                        h.write_u32(k);
                    }
                    None => {
                        h.write(&[0]);
                        h.write(lane);
                    }
                }
            }
        }
    };
    let mut h = StableHasher::new();
    h.write_u32(order.len() as u32);
    for &i in order {
        let row = &s.rows[i];
        h.write_str(s.name_of(row.entity).unwrap_or(""));
        let mut parts: Vec<(&str, usize, &[u8])> = row
            .components
            .iter()
            .map(|(t, b)| (s.type_name(*t), *t, b.as_slice()))
            .filter(|(n, _, _)| ignored(ignore, n).is_none_or(|f| !f.is_empty()))
            .collect();
        parts.sort_by_key(|p| p.0);
        for (name, t, bytes) in parts {
            h.write_str(name);
            record(&mut h, t, bytes, ignored(ignore, name).unwrap_or(&[]));
        }
    }
    let mut resources: Vec<(&str, usize, &[u8])> = s
        .resources
        .iter()
        .map(|(t, b)| (s.type_name(*t), *t, b.as_slice()))
        .filter(|(n, _, _)| {
            !SESSION_RESOURCES.contains(n) && ignored(ignore, n).is_none_or(|f| !f.is_empty())
        })
        .collect();
    resources.sort_by_key(|p| p.0);
    for (name, t, bytes) in resources {
        h.write_str(name);
        record(&mut h, t, bytes, ignored(ignore, name).unwrap_or(&[]));
    }
    let mut events: Vec<&Queue> = s
        .events
        .iter()
        .filter(|q| !SESSION_EVENTS.contains(&s.type_name(q.ty)))
        .collect();
    events.sort_by_key(|q| s.type_name(q.ty));
    for q in events {
        h.write_str(s.type_name(q.ty));
        for list in [&q.current, &q.pending] {
            h.write_u32(list.len() as u32);
            for e in list {
                record(&mut h, q.ty, e, &[]);
            }
        }
    }
    h.finish()
}

/// What differs between a fresh start's world (`start`) and the world after
/// a restart (`now`), structurally: entities paired by name, then in spawn
/// order within their component set; every handle field (u64, and lists of
/// them) read through that pairing; the allocator's generations and free
/// list, each peer's session and `ignore` left out.
///
/// ```text
/// { differs, hash, startHash,
///   components: [{ entity, component, fields: [{ field, start, now }] | only: "start"|"now" }],
///   names: [{ entity, start, now }],
///   entities: { missing: [path], extra: [path] },
///   resources: [{ resource, fields | only }],
///   events: [{ event, start, now }]     (queued counts)
///   blocks: [{ block, startBytes, nowBytes }], unchecked: [block], truncated: n }
/// ```
///
/// Entities are named by path ([`Snapshot::entity_path`], the fresh
/// start's). A module state block (physics, animation) is compared by its
/// bytes while every entity kept its id; once ids moved it holds ids the
/// pairing cannot read, and one that differs is listed in `unchecked`
/// instead. `hash` and `startHash` are each world's structural hash
/// (entities by their paired place): equal when nothing but blocks differs.
pub fn restart_diff(start: &Snapshot, now: &Snapshot, ignore: &Ignore) -> Value {
    let pairing = pair(start, now);
    let map = &pairing.now_to_start;
    let mut truncated = 0usize;
    let mut push = |list: &mut Vec<Value>, v: Value| {
        if list.len() < MAX_DIFFERENCES {
            list.push(v);
        } else {
            truncated += 1;
        }
    };
    let mut components = Vec::new();
    let mut names = Vec::new();
    for &(i, j) in &pairing.pairs {
        let (rs, rn) = (&start.rows[i], &now.rows[j]);
        let path = start.entity_path(rs.entity);
        let (ns, nn) = (start.name_of(rs.entity), now.name_of(rn.entity));
        if ns != nn {
            names.push(json!({ "entity": path, "start": ns, "now": nn }));
        }
        for (t, bytes) in &rs.components {
            let name = start.type_name(*t);
            let skip = ignored(ignore, name).unwrap_or(&[]);
            if ignored(ignore, name).is_some_and(<[String]>::is_empty) {
                continue;
            }
            match rn
                .components
                .iter()
                .find(|(u, _)| now.type_name(*u) == name)
            {
                Some((u, b2)) => {
                    let fields = record_changes((start, *t, bytes), (now, *u, b2), skip, map);
                    if !fields.is_empty() {
                        push(
                            &mut components,
                            json!({ "entity": path, "component": name, "fields": fields }),
                        );
                    }
                }
                None => push(
                    &mut components,
                    json!({ "entity": path, "component": name, "only": "start" }),
                ),
            }
        }
        for (u, _) in &rn.components {
            let name = now.type_name(*u);
            if !ignored(ignore, name).is_some_and(<[String]>::is_empty)
                && !rs
                    .components
                    .iter()
                    .any(|(t, _)| start.type_name(*t) == name)
            {
                push(
                    &mut components,
                    json!({ "entity": path, "component": name, "only": "now" }),
                );
            }
        }
    }
    let missing: Vec<Value> = pairing
        .missing
        .iter()
        .map(|&i| json!(start.entity_path(start.rows[i].entity)))
        .collect();
    let extra: Vec<Value> = pairing
        .extra
        .iter()
        .map(|&j| json!(now.entity_path(now.rows[j].entity)))
        .collect();
    let mut resources = Vec::new();
    for (t, bytes) in &start.resources {
        let name = start.type_name(*t);
        let skip = ignored(ignore, name);
        if SESSION_RESOURCES.contains(&name) || skip.is_some_and(<[String]>::is_empty) {
            continue;
        }
        match now
            .resources
            .iter()
            .find(|(u, _)| now.type_name(*u) == name)
        {
            Some((u, b2)) => {
                let fields =
                    record_changes((start, *t, bytes), (now, *u, b2), skip.unwrap_or(&[]), map);
                if !fields.is_empty() {
                    resources.push(json!({ "resource": name, "fields": fields }));
                }
            }
            None => resources.push(json!({ "resource": name, "only": "start" })),
        }
    }
    for (u, _) in &now.resources {
        let name = now.type_name(*u);
        if !SESSION_RESOURCES.contains(&name)
            && !ignored(ignore, name).is_some_and(<[String]>::is_empty)
            && !start
                .resources
                .iter()
                .any(|(t, _)| start.type_name(*t) == name)
        {
            resources.push(json!({ "resource": name, "only": "now" }));
        }
    }
    let mut events = Vec::new();
    for q in &start.events {
        let name = start.type_name(q.ty);
        if SESSION_EVENTS.contains(&name) {
            continue;
        }
        let theirs = now.events.iter().find(|r| now.type_name(r.ty) == name);
        let same_list = |a: &[Vec<u8>], b: &[Vec<u8>], r: &Queue| {
            a.len() == b.len()
                && a.iter().zip(b).all(|(x, y)| {
                    record_changes((start, q.ty, x), (now, r.ty, y), &[], map).is_empty()
                })
        };
        let same = theirs.is_some_and(|r| {
            same_list(&q.current, &r.current, r) && same_list(&q.pending, &r.pending, r)
        });
        if !same {
            let count = |r: Option<&Queue>| r.map_or(0, |r| r.current.len() + r.pending.len());
            events.push(json!({ "event": name, "start": count(Some(q)), "now": count(theirs) }));
        }
    }
    let ids_kept = pairing
        .pairs
        .iter()
        .all(|&(i, j)| start.rows[i].entity == now.rows[j].entity);
    let (mut blocks, mut unchecked) = (Vec::new(), Vec::new());
    for (name, bytes) in &start.blocks {
        if SESSION_BLOCKS.contains(&name.as_str()) {
            continue;
        }
        let theirs = now.blocks.iter().find(|(n, _)| n == name).map(|(_, b)| b);
        if theirs == Some(bytes) {
            continue;
        }
        if ids_kept || theirs.is_none() {
            blocks.push(json!({ "block": name, "startBytes": bytes.len(), "nowBytes": theirs.map(Vec::len) }));
        } else {
            unchecked.push(json!(name));
        }
    }
    let start_order: Vec<usize> = pairing
        .pairs
        .iter()
        .map(|p| p.0)
        .chain(pairing.missing.iter().copied())
        .collect();
    let now_order: Vec<usize> = pairing
        .pairs
        .iter()
        .map(|p| p.1)
        .chain(pairing.extra.iter().copied())
        .collect();
    let differs = !components.is_empty()
        || !names.is_empty()
        || !missing.is_empty()
        || !extra.is_empty()
        || !resources.is_empty()
        || !events.is_empty()
        || !blocks.is_empty();
    json!({
        "differs": differs,
        "hash": format!("{:016x}", structural_hash(now, &now_order, ignore)),
        "startHash": format!("{:016x}", structural_hash(start, &start_order, ignore)),
        "components": components,
        "names": names,
        "entities": { "missing": missing, "extra": extra },
        "resources": resources,
        "events": events,
        "blocks": blocks,
        "unchecked": unchecked,
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

    /// A fresh start: a named hero, two coins and a Target on the hero
    /// holding the second coin's handle, all in their first slots.
    fn arena() -> Snapshot {
        let mut target = ty("Target", &[]);
        target
            .schema
            .fields
            .push(FieldSchema::new("coin", FieldType::U64));
        target.layout = Layout::of(&target.schema);
        let mut w = world();
        w.types.push(target);
        let target = w.types.len() - 1;
        let coin = |x: f32| Row {
            entity: 0,
            components: vec![(1, floats(&[x]))],
        };
        w.rows = vec![
            Row {
                entity: 0,
                components: vec![
                    (0, floats(&[1.0, 2.0])),
                    (target, 2u64.to_le_bytes().to_vec()),
                ],
            },
            coin(10.0),
            coin(20.0),
        ];
        w.rows[1].entity = 1;
        w.rows[2].entity = 2;
        w.slots = vec![(0, true); 3];
        w.names = vec![(0, "hero".into())];
        w
    }

    /// `w` after despawning every entity and spawning them again in the same
    /// order: new generations, slots taken from the free list's end.
    fn respawned(w: &Snapshot) -> Snapshot {
        let mut r = w.clone();
        let id = |index: u64| (1u64 << 32) | index;
        let new_of = [id(2), id(1), id(0)];
        for (row, new) in r.rows.iter_mut().zip(new_of) {
            row.entity = new;
        }
        let target = r.types.iter().position(|t| t.name == "Target").unwrap();
        r.rows[0].components[1] = (target, new_of[2].to_le_bytes().to_vec());
        r.names = vec![(new_of[0], "hero".into())];
        r.slots = vec![(1, true); 3];
        r
    }

    #[test]
    fn a_restart_that_spawns_its_entities_again_is_the_fresh_start() {
        let start = arena();
        let now = respawned(&start);
        assert_ne!(
            start.portable_hash(&start.portable_table()),
            now.portable_hash(&now.portable_table())
        );
        let d = restart_diff(&start, &now, &Ignore::new());
        assert_eq!(d["differs"], json!(false), "{d:#}");
        assert_eq!(d["hash"], d["startHash"]);
        // The hero's Target holding the first coin instead of the second is a difference.
        let mut wrong = now.clone();
        let target = wrong.rows[0].components[1].0;
        wrong.rows[0].components[1] = (target, ((1u64 << 32) | 1).to_le_bytes().to_vec());
        let d = restart_diff(&start, &wrong, &Ignore::new());
        assert_eq!(d["components"][0]["entity"], json!("hero"));
        assert_eq!(d["components"][0]["component"], json!("Target"));
        assert_ne!(d["hash"], d["startHash"]);
        // A coin left over from the last game is an extra entity, named.
        let mut leak = now.clone();
        leak.rows.push(Row {
            entity: 3,
            components: vec![(1, floats(&[30.0]))],
        });
        leak.slots.push((0, true));
        let d = restart_diff(&start, &leak, &Ignore::new());
        assert_eq!(d["differs"], json!(true));
        assert_eq!(d["entities"]["extra"], json!(["#3.0"]));
        assert_eq!(d["entities"]["missing"], json!([]));
        // A coin the restart forgot is a missing one, by the fresh start's name for it.
        let mut lost = now.clone();
        lost.rows.pop();
        assert_eq!(
            restart_diff(&start, &lost, &Ignore::new())["entities"]["missing"],
            json!(["#2.0"])
        );
        // An ignored field is left out.
        let mut moved = now.clone();
        moved.resources[0].1 = floats(&[9.0]);
        assert_eq!(
            restart_diff(&start, &moved, &Ignore::new())["resources"][0]["resource"],
            json!("Clock")
        );
        let clock = Ignore::from([("Clock".to_owned(), vec![])]);
        assert_eq!(
            restart_diff(&start, &moved, &clock)["differs"],
            json!(false)
        );
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

    /// `s` as og-ecs writes it (the module docs' format), every row of a
    /// run of equal component sets as one archetype.
    fn encode(s: &Snapshot) -> Vec<u8> {
        fn u32_(out: &mut Vec<u8>, v: usize) {
            out.extend_from_slice(&(v as u32).to_le_bytes());
        }
        fn sized(out: &mut Vec<u8>, b: &[u8]) {
            u32_(out, b.len());
            out.extend_from_slice(b);
        }
        let mut out = MAGIC.to_vec();
        u32_(&mut out, s.format as usize);
        u32_(&mut out, s.types.len());
        for t in &s.types {
            sized(&mut out, t.name.as_bytes());
            out.extend_from_slice(&t.hash.to_le_bytes());
            u32_(&mut out, t.schema.version as usize);
            u32_(&mut out, t.schema.fields.len());
            for f in &t.schema.fields {
                sized(&mut out, f.name.as_bytes());
                sized(&mut out, f.ty.name().as_bytes());
                u32_(&mut out, f.variants.len());
                for v in &f.variants {
                    sized(&mut out, v.as_bytes());
                }
            }
        }
        u32_(&mut out, s.slots.len());
        for (generation, alive) in &s.slots {
            u32_(&mut out, *generation as usize);
            out.push(u8::from(*alive));
        }
        u32_(&mut out, s.free.len());
        for f in &s.free {
            u32_(&mut out, *f as usize);
        }
        let stored = |r: &Row| r.components.iter().map(|c| c.0).collect::<Vec<_>>();
        let archetypes: Vec<&[Row]> = s.rows.chunk_by(|a, b| stored(a) == stored(b)).collect();
        u32_(&mut out, archetypes.len());
        for rows in archetypes {
            let types = stored(&rows[0]);
            u32_(&mut out, types.len());
            for t in &types {
                u32_(&mut out, *t);
            }
            u32_(&mut out, rows.len());
            for r in rows {
                out.extend_from_slice(&r.entity.to_le_bytes());
            }
            for (c, t) in types.iter().enumerate() {
                let layout = &s.types[*t].layout;
                for f in 0..layout.field_count() {
                    for r in rows {
                        out.extend_from_slice(&r.components[c].1[layout.range(f)]);
                    }
                }
            }
        }
        u32_(&mut out, s.resources.len());
        for (t, b) in &s.resources {
            u32_(&mut out, *t);
            sized(&mut out, b);
        }
        u32_(&mut out, s.events.len());
        for q in &s.events {
            u32_(&mut out, q.ty);
            for list in [&q.current, &q.pending] {
                u32_(&mut out, list.len());
                for e in list {
                    out.extend_from_slice(e);
                }
            }
        }
        u32_(&mut out, s.names.len());
        for (e, name) in &s.names {
            out.extend_from_slice(&e.to_le_bytes());
            sized(&mut out, name.as_bytes());
        }
        if s.format >= 3 {
            u32_(&mut out, s.blocks.len());
            for (name, b) in &s.blocks {
                sized(&mut out, name.as_bytes());
                sized(&mut out, b);
            }
        }
        if let Some(system) = &s.resume {
            sized(&mut out, system.as_bytes());
        }
        out
    }

    /// [`world`] with what the hash treats specially: an archetype of two
    /// rows with two components (three columns), a dead slot with its free
    /// index, a peer's own resource, event queue and state block beside
    /// shared ones, and the system a mid-tick snapshot was taken in.
    fn busy() -> Snapshot {
        let mut s = world();
        s.format = 4;
        s.types.extend([
            ty("og.Input", &["keys"]),
            ty("Hit", &["a", "b"]),
            ty("og.KeyDown", &["key"]),
        ]);
        s.slots = vec![(0, true), (3, false), (1, true), (0, true)];
        s.free = vec![1];
        s.rows.push(Row {
            entity: 2 | 1 << 32,
            components: vec![(0, floats(&[6.0, 7.0])), (1, floats(&[8.0]))],
        });
        s.rows.push(Row {
            entity: 3,
            components: vec![(1, floats(&[9.0]))],
        });
        s.resources.push((3, floats(&[11.0])));
        s.events = vec![
            Queue {
                ty: 4,
                current: vec![floats(&[1.0, 2.0])],
                pending: vec![floats(&[3.0, 4.0]), floats(&[5.0, 6.0])],
            },
            Queue {
                ty: 5,
                current: vec![floats(&[13.0])],
                pending: vec![],
            },
        ];
        s.blocks.push(("og.ui".into(), vec![4, 5]));
        s.resume = Some("Apply".into());
        s
    }

    #[test]
    fn the_hash_read_from_the_bytes_is_the_decoded_snapshots() {
        for s in [world(), busy()] {
            let bytes = encode(&s);
            // The bytes are the snapshot: decoding gives it back.
            let read = Snapshot::decode(&bytes).expect("a snapshot");
            assert_eq!(format!("{read:?}"), format!("{s:?}"));
            assert_eq!(shared_hash(&bytes), Ok(s.shared_hash()));
        }
        // Each peer's own session is left out, and nothing else.
        let hash = |s: &Snapshot| shared_hash(&encode(s)).expect("a snapshot");
        let base = busy();
        let mut own = busy();
        own.resources[1].1 = floats(&[12.0]);
        own.events[1].current.clear();
        own.blocks[1].1 = vec![6];
        assert_eq!(hash(&own), hash(&base));
        let moved: [fn(&mut Snapshot); 6] = [
            |s| s.rows[2].components[0].1 = floats(&[6.0, 7.5]),
            |s| s.rows.swap(1, 2),
            |s| s.slots[1].0 = 4,
            |s| s.events[0].pending.truncate(1),
            |s| s.blocks[0].1[2] = 0,
            |s| s.resume = Some("Submit".into()),
        ];
        for (i, change) in moved.iter().enumerate() {
            let mut s = busy();
            change(&mut s);
            assert_eq!(shared_hash(&encode(&s)), Ok(s.shared_hash()), "change {i}");
            assert_ne!(hash(&s), hash(&base), "change {i}");
        }
        // What decode refuses is refused.
        let bytes = encode(&base);
        assert!(shared_hash(&bytes[..bytes.len() - 1]).is_err());
        let mut long = bytes.clone();
        long.push(0);
        assert!(shared_hash(&long).is_err());
    }
}
