//! Whole-world snapshot and restore, migrating restore, and the world hash.
//!
//! A snapshot is everything that determines what the world does next: the
//! type table, the entity allocator (every slot's generation and the free
//! list in reuse order), every archetype in creation order with its rows in
//! storage order (so iteration order survives), every resource (including
//! `Time` and the `Rng` states), both event queues and the entity names.
//! Systems are code and are not in it; restore into a world whose schedule
//! and registrations match.
//!
//! Format 2 (little-endian), what [`World::snapshot`] writes for a world
//! without state blocks; format 3 is the same with the blocks section:
//!
//! ```text
//! "OGWS" u32:2 (or 3)
//! types      u32 n, n × { str name, u64 layout_hash, u32 version,
//!                         u32 fields, fields × { str name, str type,
//!                                                u32 k, k × str variant } }
//! allocator  u32 slots, slots × { u32 generation, u8 alive }, u32 n, n × u32 free
//! archetypes u32 n, n × { u32 k, k × u32 type, u32 rows, rows × u64 entity,
//!                         per type, per field: rows × field bytes }
//! resources  u32 n, n × { u32 type, str bytes }
//! events     u32 n, n × { u32 type, u32 current, current × bytes,
//!                                   u32 pending, pending × bytes }
//! names      u32 n, n × { u64 entity, str name }      (entity order)
//! blocks     u32 n, n × { str name, str bytes }       (format 3, added order)
//! ```
//!
//! `str` is a `u32` length then the bytes. Format 1 (the M1 snapshots) has
//! only `str name, u64 layout_hash` per type and no names section; it is
//! still read. [`World::restore`] maps types by name, requires equal layout
//! hashes, validates every count and length, and changes nothing unless the
//! whole snapshot is valid. [`World::restore_migrating`] maps fields by name
//! instead (see [`MigrationReport`]).
//!
//! [`World::world_hash`] is FNV-1a of the *state stream*: the snapshot as
//! format 1 lays it out (type names and layout hashes, no field table), then,
//! only when some entity is named, `"NAME"` and the names section, then,
//! only when the world has state blocks, `"BLOK"` and the blocks section. The field
//! table is a function of the registrations the layout hashes already cover,
//! so two worlds have the same hash exactly when their state is identical,
//! and a world without names or blocks hashes exactly as it did before they
//! existed.

use crate::entity::Entity;
use crate::registry::{Registry, SchemaId};
use crate::state::Block;
use crate::storage::Archetype;
use crate::world::{
    Entities, EventQueue, Location, MAX_NAME, PENDING, ResourceValue, World, resource_to_bytes,
};
use og_core::handle::{HandleAllocator, RawHandle, SlotState};
use og_core::hash::hash_bytes;
use og_core::reflect::{FieldSchema, FieldType, Schema};
use og_core::{Error, ErrorKind, Result};
use std::any::Any;
use std::cell::RefCell;
use std::collections::BTreeMap;

const MAGIC: &[u8; 4] = b"OGWS";
/// The format [`World::snapshot`] writes for a world without state blocks.
const FORMAT: u32 = 2;
/// Format 2 with the state blocks section.
const FORMAT_BLOCKS: u32 = 3;
/// The M1 format: no field table, no names. Read, and the hash stream's.
const FORMAT_STATE: u32 = 1;
const NAMES_TAG: &[u8; 4] = b"NAME";
const BLOCKS_TAG: &[u8; 4] = b"BLOK";

fn field_type(name: &str) -> Option<FieldType> {
    FieldType::parse(name)
}

struct Writer(Vec<u8>);

impl Writer {
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    fn u32(&mut self, v: usize) {
        self.0
            .extend_from_slice(&u32::try_from(v).expect("fits in u32").to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn bytes(&mut self, b: &[u8]) {
        self.0.extend_from_slice(b);
    }
    fn sized(&mut self, b: &[u8]) {
        self.u32(b.len());
        self.bytes(b);
    }
}

struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

fn bad(reason: impl Into<String>) -> Error {
    Error::new(ErrorKind::Parse, format!("snapshot: {}", reason.into()))
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self
            .pos
            .checked_add(n)
            .filter(|&e| e <= self.buf.len())
            .ok_or_else(|| bad(format!("truncated at byte {}", self.pos)))?;
        let out = &self.buf[self.pos..end];
        self.pos = end;
        Ok(out)
    }
    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<usize> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().expect("4 bytes")) as usize)
    }
    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("8 bytes"),
        ))
    }
    /// A count of items at least `min_item` bytes each, checked against what is left.
    fn count(&mut self, min_item: usize) -> Result<usize> {
        let n = self.u32()?;
        if n.saturating_mul(min_item) > self.buf.len() - self.pos {
            return Err(bad(format!(
                "count {n} at byte {} exceeds the data",
                self.pos - 4
            )));
        }
        Ok(n)
    }
    fn sized(&mut self) -> Result<&'a [u8]> {
        let n = self.u32()?;
        self.take(n)
    }
    fn str(&mut self, what: &str) -> Result<&'a str> {
        std::str::from_utf8(self.sized()?).map_err(|_| bad(format!("{what} is not UTF-8")))
    }
}

/// What [`World::restore_migrating`] did beyond a plain restore. Entries
/// name types (`Type`) or fields (`Type.field`), in snapshot type order, then
/// field order. All empty means the snapshot fitted this world exactly.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MigrationReport {
    /// Types whose layout differs between the snapshot and this world; their
    /// components, resources and events were mapped field by field, by name.
    pub migrated: Vec<String>,
    /// Fields this world has and the snapshot's type lacked (or had with
    /// another field type): set to the field's default.
    pub added: Vec<String>,
    /// Fields the snapshot had and this world's type lacks (or has with
    /// another field type): dropped.
    pub removed: Vec<String>,
    /// Types in the snapshot this world does not register: their components,
    /// resources and events were dropped.
    pub dropped: Vec<String>,
    /// Anything else worth reading: a field whose type changed, enum values
    /// whose variant no longer exists (reset to the field's default), a type
    /// that gained a string field and so lost its components.
    pub warnings: Vec<String>,
}

impl MigrationReport {
    /// Whether the snapshot fitted this world exactly.
    pub fn is_exact(&self) -> bool {
        self.migrated.is_empty()
            && self.added.is_empty()
            && self.removed.is_empty()
            && self.dropped.is_empty()
            && self.warnings.is_empty()
    }
}

/// One type of the snapshot's type table.
struct SnapType {
    name: String,
    hash: u64,
    /// The snapshot's own field table (format 2).
    schema: Option<Schema>,
    /// Packed size of each field, from the field table or, in format 1, from
    /// this world's type of the same name and layout hash.
    sizes: Vec<usize>,
}

impl SnapType {
    fn size(&self) -> usize {
        self.sizes.iter().sum()
    }
}

struct SnapArchetype<'a> {
    /// Snapshot type indices in stored order.
    stored: Vec<usize>,
    entities: Vec<Entity>,
    /// Per stored type, per field: `rows × size` bytes.
    columns: Vec<Vec<&'a [u8]>>,
}

/// One event queue: snapshot type, current events, pending events.
type Queue<'a> = (usize, Vec<&'a [u8]>, Vec<&'a [u8]>);

/// A snapshot read and validated, its types not yet mapped onto this world.
struct Parsed<'a> {
    types: Vec<SnapType>,
    alloc: HandleAllocator,
    slot_count: usize,
    archetypes: Vec<SnapArchetype<'a>>,
    resources: Vec<(usize, &'a [u8])>,
    events: Vec<Queue<'a>>,
    names: Vec<(Entity, String)>,
    /// State blocks (format 3): name, saved bytes.
    blocks: Vec<(&'a str, &'a [u8])>,
}

fn not_registered(name: &str) -> Error {
    Error::new(
        ErrorKind::NotFound,
        format!("snapshot: type '{name}' is not registered in this world"),
    )
}

fn other_layout(name: &str) -> Error {
    Error::new(
        ErrorKind::Schema,
        format!("snapshot: type '{name}' has a different layout in this world"),
    )
}

/// How a saved type's fields differ from this world's: ` (Game.coins
/// added; Game.gold removed; Game.speed f32 -> f64)`, empty when only the
/// metadata differs.
fn layout_difference(saved: &Schema, now: &Schema) -> String {
    let mut parts = Vec::new();
    let added: Vec<String> = now
        .fields
        .iter()
        .filter(|f| !saved.fields.iter().any(|s| s.name == f.name))
        .map(|f| format!("{}.{}", now.name, f.name))
        .collect();
    let removed: Vec<String> = saved
        .fields
        .iter()
        .filter(|s| !now.fields.iter().any(|f| f.name == s.name))
        .map(|s| format!("{}.{}", now.name, s.name))
        .collect();
    if !added.is_empty() {
        parts.push(format!("{} added", added.join(", ")));
    }
    if !removed.is_empty() {
        parts.push(format!("{} removed", removed.join(", ")));
    }
    for f in &now.fields {
        if let Some(s) = saved.fields.iter().find(|s| s.name == f.name) {
            if s.ty != f.ty {
                parts.push(format!(
                    "{}.{} {} -> {}",
                    now.name,
                    f.name,
                    s.ty.name(),
                    f.ty.name()
                ));
            } else if s.variants != f.variants {
                parts.push(format!("{}.{}'s variants changed", now.name, f.name));
            }
        }
    }
    if parts.is_empty() {
        let order = saved
            .fields
            .iter()
            .map(|f| &f.name)
            .ne(now.fields.iter().map(|f| &f.name));
        if order {
            parts.push(format!("{}'s fields are in another order", now.name));
        }
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" ({})", parts.join("; "))
    }
}

fn parse<'a>(registry: &Registry, bytes: &'a [u8]) -> Result<Parsed<'a>> {
    let mut r = Reader { buf: bytes, pos: 0 };
    if r.take(4)? != MAGIC {
        return Err(bad("not a world snapshot (magic)"));
    }
    let format = r.u32()? as u32;
    if ![FORMAT_STATE, FORMAT, FORMAT_BLOCKS].contains(&format) {
        return Err(bad(format!(
            "format {format}, this engine reads {FORMAT_STATE} to {FORMAT_BLOCKS}"
        )));
    }
    // Formats 2 and 3 share everything but the blocks section.
    let blocks_section = format == FORMAT_BLOCKS;
    let format = if blocks_section { FORMAT } else { format };

    let type_count = r.count(12)?;
    let mut types: Vec<SnapType> = Vec::with_capacity(type_count);
    for _ in 0..type_count {
        let name = r.str("type name")?.to_owned();
        let hash = r.u64()?;
        let (schema, sizes) = if format == FORMAT {
            let version = r.u32()? as u32;
            let n = r.count(12)?;
            let mut fields = Vec::with_capacity(n);
            for _ in 0..n {
                let fname = r.str("field name")?;
                let tname = r.str("field type")?;
                let ty = field_type(tname).ok_or_else(|| {
                    bad(format!("field '{name}.{fname}' has unknown type '{tname}'"))
                })?;
                let k = r.count(4)?;
                let mut variants = Vec::with_capacity(k);
                for _ in 0..k {
                    variants.push(r.str("variant name")?.to_owned());
                }
                let mut f = FieldSchema::new(fname, ty);
                f.variants = variants;
                fields.push(f);
            }
            let schema = Schema {
                name: name.clone(),
                version,
                doc: String::new(),
                fields,
            };
            if schema.layout_hash() != hash {
                return Err(bad(format!(
                    "the field table of '{name}' does not match its layout hash"
                )));
            }
            let sizes = schema.fields.iter().map(|f| f.ty.size()).collect();
            (Some(schema), sizes)
        } else {
            let id = registry
                .id_by_name(&name)
                .map_err(|_| not_registered(&name))?;
            if registry.info(id).layout_hash != hash {
                return Err(other_layout(&name));
            }
            let layout = &registry.info(id).layout;
            let sizes = (0..layout.field_count())
                .map(|f| layout.range(f).len())
                .collect();
            (None, sizes)
        };
        if types.iter().any(|t| t.name == name) {
            return Err(bad(format!("type '{name}' appears twice")));
        }
        types.push(SnapType {
            name,
            hash,
            schema,
            sizes,
        });
    }
    let type_count = types.len();
    let type_at = |i: usize| -> Result<usize> {
        if i < type_count {
            Ok(i)
        } else {
            Err(bad(format!("type index {i} out of range")))
        }
    };

    // Entity allocator.
    let slot_count = r.count(5)?;
    let mut slots = Vec::with_capacity(slot_count);
    for _ in 0..slot_count {
        let generation = r.u32()? as u32;
        let alive = match r.u8()? {
            0 => false,
            1 => true,
            v => return Err(bad(format!("slot liveness byte {v}"))),
        };
        slots.push(SlotState { generation, alive });
    }
    let free_count = r.count(4)?;
    let mut free = Vec::with_capacity(free_count);
    for _ in 0..free_count {
        free.push(r.u32()? as u32);
    }
    let alloc = HandleAllocator::from_parts(&slots, &free).map_err(|e| bad(e.reason()))?;

    // Archetypes, in creation order.
    let arch_count = r.count(8)?;
    if arch_count == 0 {
        return Err(bad("no archetypes (the empty archetype is always present)"));
    }
    let mut seen = vec![false; slot_count];
    let mut archetypes = Vec::with_capacity(arch_count);
    let mut placed = 0usize;
    for a in 0..arch_count {
        let k = r.count(4)?;
        let mut stored = Vec::with_capacity(k);
        for _ in 0..k {
            stored.push(type_at(r.u32()?)?);
        }
        let mut sorted = stored.clone();
        sorted.sort();
        if sorted.windows(2).any(|w| w[0] == w[1]) {
            return Err(bad(format!("archetype {a} repeats a type")));
        }
        if (a == 0) != stored.is_empty() {
            return Err(bad("archetype 0 must be the empty archetype, and only it"));
        }
        let rows = r.count(8)?;
        let mut entities = Vec::with_capacity(rows);
        for _ in 0..rows {
            let e = Entity(RawHandle::from_bits(r.u64()?));
            if !alloc.is_alive(e.0) {
                return Err(bad(format!(
                    "archetype {a} holds entity {e}, which is not alive"
                )));
            }
            let s = &mut seen[e.index() as usize];
            if *s {
                return Err(bad(format!("entity {e} appears twice")));
            }
            *s = true;
            entities.push(e);
        }
        placed += rows;
        let mut columns = Vec::with_capacity(k);
        for &t in &stored {
            let mut fields = Vec::with_capacity(types[t].sizes.len());
            for &size in &types[t].sizes {
                let len = rows
                    .checked_mul(size)
                    .ok_or_else(|| bad("column size overflows"))?;
                fields.push(r.take(len)?);
            }
            columns.push(fields);
        }
        archetypes.push(SnapArchetype {
            stored,
            entities,
            columns,
        });
    }
    if placed != alloc.len() {
        return Err(bad(format!(
            "{} entities are alive but {placed} are stored",
            alloc.len()
        )));
    }

    // Resources.
    let resource_count = r.count(8)?;
    let mut resources: Vec<(usize, &[u8])> = Vec::with_capacity(resource_count);
    for _ in 0..resource_count {
        let t = type_at(r.u32()?)?;
        let data = r.sized()?;
        if data.len() != types[t].size() {
            return Err(bad(format!(
                "resource '{}' is {} bytes, its layout {}",
                types[t].name,
                data.len(),
                types[t].size()
            )));
        }
        if resources.iter().any(|(u, _)| *u == t) {
            return Err(bad(format!("resource '{}' appears twice", types[t].name)));
        }
        resources.push((t, data));
    }

    // Events.
    let queue_count = r.count(12)?;
    let mut events: Vec<Queue<'_>> = Vec::with_capacity(queue_count);
    for _ in 0..queue_count {
        let t = type_at(r.u32()?)?;
        if events.iter().any(|(u, _, _)| *u == t) {
            return Err(bad(format!(
                "event queue '{}' appears twice",
                types[t].name
            )));
        }
        let size = types[t].size();
        let mut lists: [Vec<&[u8]>; 2] = [Vec::new(), Vec::new()];
        for list in &mut lists {
            let n = r.count(size)?;
            if size == 0 && n > 1 << 24 {
                return Err(bad(format!("{n} field-less events")));
            }
            for _ in 0..n {
                list.push(r.take(size)?);
            }
        }
        let [current, pending] = lists;
        events.push((t, current, pending));
    }

    // Names (format 2).
    let mut names: Vec<(Entity, String)> = Vec::new();
    if format == FORMAT {
        let n = r.count(12)?;
        for _ in 0..n {
            let e = Entity(RawHandle::from_bits(r.u64()?));
            let name = r.str("entity name")?;
            if !alloc.is_alive(e.0) {
                return Err(bad(format!(
                    "name '{name}' belongs to {e}, which is not alive"
                )));
            }
            if name.is_empty() || name.len() > MAX_NAME {
                return Err(bad(format!("an entity name of {} bytes", name.len())));
            }
            if names.last().is_some_and(|(last, _)| *last >= e) {
                return Err(bad("names are not in entity order, or an entity has two"));
            }
            if names.iter().any(|(_, n)| n == name) {
                return Err(bad(format!("name '{name}' appears twice")));
            }
            names.push((e, name.to_owned()));
        }
    }
    let mut blocks: Vec<(&str, &[u8])> = Vec::new();
    if blocks_section {
        let n = r.count(8)?;
        for _ in 0..n {
            let name = r.str("state block name")?;
            if blocks.iter().any(|(b, _)| *b == name) {
                return Err(bad(format!("state block '{name}' appears twice")));
            }
            blocks.push((name, r.sized()?));
        }
    }
    if r.pos != bytes.len() {
        return Err(bad(format!("{} trailing bytes", bytes.len() - r.pos)));
    }
    Ok(Parsed {
        types,
        alloc,
        slot_count,
        archetypes,
        resources,
        events,
        names,
        blocks,
    })
}

/// Each block's decoded state from a snapshot (`None`: the snapshot has no
/// bytes for it). A snapshot block this world lacks is refused, or, when
/// `report` is given (a migrating restore), dropped and reported.
fn decode_blocks(
    blocks: &[Block],
    saved: &[(&str, &[u8])],
    mut report: Option<&mut MigrationReport>,
) -> Result<Vec<Option<Box<dyn Any + Send>>>> {
    for (name, _) in saved {
        if !blocks.iter().any(|b| b.name == *name) {
            match report.as_deref_mut() {
                Some(r) => r.dropped.push(format!("state block {name}")),
                None => {
                    return Err(Error::new(
                        ErrorKind::NotFound,
                        format!("snapshot: state block '{name}' is not added to this world"),
                    ));
                }
            }
        }
    }
    blocks
        .iter()
        .map(|b| match saved.iter().find(|(n, _)| *n == b.name) {
            Some((_, bytes)) => b
                .block
                .decode(bytes)
                .map(Some)
                .map_err(|e| e.context(format!("snapshot: state block '{}'", b.name))),
            None => {
                if let Some(r) = report.as_deref_mut() {
                    r.warnings.push(format!(
                        "state block {} is not in the snapshot; it starts empty",
                        b.name
                    ));
                }
                Ok(None)
            }
        })
        .collect()
}

/// Where one field of this world's type comes from.
#[derive(Clone, Debug)]
enum Source {
    /// The snapshot field with this index, byte for byte.
    Copy(usize),
    /// The snapshot enum field with this index, its indices mapped by
    /// variant name (`None`: the variant is gone, use the default).
    Enum(usize, Vec<Option<u32>>),
    /// The snapshot string field with this index, of another capacity:
    /// its text when it fits, else the default (empty).
    Text(usize),
    /// The field's default.
    Default,
}

/// How one snapshot type becomes one of this world's types.
struct Plan {
    target: SchemaId,
    /// `None`: the layouts are identical, copy everything.
    fields: Option<Vec<Source>>,
    /// Byte ranges of the snapshot type's fields.
    source_ranges: Vec<std::ops::Range<usize>>,
    /// Default bytes of each of this world's fields.
    defaults: Vec<Vec<u8>>,
}

impl Plan {
    fn mapped_enum(&self, k: usize, map: &[Option<u32>], raw: &[u8], out: &mut Vec<u8>) {
        let i = u32::from_le_bytes(raw.try_into().expect("an enum is 4 bytes"));
        match map.get(i as usize).copied().flatten() {
            Some(j) => out.extend_from_slice(&j.to_le_bytes()),
            None => out.extend_from_slice(&self.defaults[k]),
        }
    }

    /// A string of another capacity as field `k`: its text, zero-padded,
    /// or the default when the text does not fit.
    fn text(&self, k: usize, raw: &[u8], out: &mut Vec<u8>) {
        let size = self.defaults[k].len();
        let text = og_core::unpack_str(raw).as_bytes();
        if text.len() <= size {
            out.extend_from_slice(text);
            out.resize(out.len() + size - text.len(), 0);
        } else {
            out.extend_from_slice(&self.defaults[k]);
        }
    }

    /// One packed value of the snapshot type as this world's type.
    fn convert(&self, value: &[u8], out: &mut Vec<u8>) {
        let Some(fields) = &self.fields else {
            out.extend_from_slice(value);
            return;
        };
        for (k, source) in fields.iter().enumerate() {
            match source {
                Source::Copy(f) => out.extend_from_slice(&value[self.source_ranges[*f].clone()]),
                Source::Enum(f, map) => {
                    self.mapped_enum(k, map, &value[self.source_ranges[*f].clone()], out)
                }
                Source::Text(f) => self.text(k, &value[self.source_ranges[*f].clone()], out),
                Source::Default => out.extend_from_slice(&self.defaults[k]),
            }
        }
    }

    /// Field `k` of this world's type for `rows` rows, from the snapshot
    /// type's columns.
    fn column(&self, k: usize, rows: usize, columns: &[&[u8]], out: &mut Vec<u8>) {
        match self.fields.as_ref().map(|f| &f[k]) {
            None => out.extend_from_slice(columns[k]),
            Some(Source::Copy(f)) => out.extend_from_slice(columns[*f]),
            Some(Source::Enum(f, map)) => {
                for raw in columns[*f].as_chunks::<4>().0 {
                    self.mapped_enum(k, map, raw, out);
                }
            }
            Some(Source::Text(f)) => {
                let n = self.source_ranges[*f].len();
                for raw in columns[*f].chunks_exact(n) {
                    self.text(k, raw, out);
                }
            }
            Some(Source::Default) => {
                for _ in 0..rows {
                    out.extend_from_slice(&self.defaults[k]);
                }
            }
        }
    }
}

fn plan(
    registry: &Registry,
    t: &SnapType,
    target: SchemaId,
    report: &mut MigrationReport,
) -> Result<Plan> {
    let info = registry.info(target);
    let mut offset = 0;
    let source_ranges = t
        .sizes
        .iter()
        .map(|&n| {
            offset += n;
            offset - n..offset
        })
        .collect();
    let defaults = info
        .schema
        .fields
        .iter()
        .map(FieldSchema::default_bytes)
        .collect();
    if info.layout_hash == t.hash {
        return Ok(Plan {
            target,
            fields: None,
            source_ranges,
            defaults,
        });
    }
    let Some(snap) = &t.schema else {
        return Err(Error::new(
            ErrorKind::Schema,
            format!(
                "snapshot: type '{}' has a different layout in this world, and a format 1 snapshot carries no field names to migrate it by",
                t.name
            ),
        ));
    };
    report.migrated.push(t.name.clone());
    let mut fields = Vec::with_capacity(info.schema.fields.len());
    for f in &info.schema.fields {
        let at = format!("{}.{}", t.name, f.name);
        let source = match snap.field_index(&f.name) {
            None => {
                report.added.push(at);
                Source::Default
            }
            Some(i)
                if snap.fields[i].ty.is_string()
                    && f.ty.is_string()
                    && snap.fields[i].ty != f.ty =>
            {
                if snap.fields[i].ty.size() > f.ty.size() {
                    report.warnings.push(format!(
                        "{at} changed from {} to {}; a text longer than {} bytes starts empty",
                        snap.fields[i].ty.name(),
                        f.ty.name(),
                        f.ty.size()
                    ));
                }
                Source::Text(i)
            }
            Some(i) if snap.fields[i].ty != f.ty => {
                report.warnings.push(format!(
                    "{at} changed type from {} to {}; it starts at its default",
                    snap.fields[i].ty.name(),
                    f.ty.name()
                ));
                report.removed.push(at.clone());
                report.added.push(at);
                Source::Default
            }
            Some(i) if f.ty == FieldType::Enum && snap.fields[i].variants != f.variants => {
                let map: Vec<Option<u32>> = snap.fields[i]
                    .variants
                    .iter()
                    .map(|v| f.variants.iter().position(|w| w == v).map(|j| j as u32))
                    .collect();
                let gone: Vec<&str> = snap.fields[i]
                    .variants
                    .iter()
                    .zip(&map)
                    .filter(|(_, m)| m.is_none())
                    .map(|(v, _)| v.as_str())
                    .collect();
                if !gone.is_empty() {
                    report.warnings.push(format!(
                        "{at}: variant(s) {} no longer exist; values holding them start at the default",
                        gone.join(", ")
                    ));
                }
                Source::Enum(i, map)
            }
            Some(i) => Source::Copy(i),
        };
        fields.push(source);
    }
    for f in &snap.fields {
        if info.schema.field_index(&f.name).is_none() {
            report.removed.push(format!("{}.{}", t.name, f.name));
        }
    }
    Ok(Plan {
        target,
        fields: Some(fields),
        source_ranges,
        defaults,
    })
}

fn resource_value(registry: &Registry, id: SchemaId, data: &[u8]) -> Result<ResourceValue> {
    let info = registry.info(id);
    Ok(match &info.rust {
        Some(rust) => ResourceValue::Typed((rust.decode)(&info.layout, data)?),
        None => ResourceValue::Raw(data.to_vec()),
    })
}

/// The restored state, committed only once all of it is built.
struct State {
    alloc: HandleAllocator,
    locations: Vec<Location>,
    archetypes: Vec<Archetype>,
    by_types: BTreeMap<Vec<SchemaId>, u32>,
    resources: Vec<Option<RefCell<ResourceValue>>>,
    events: Vec<RefCell<EventQueue>>,
    names: Vec<(Entity, String)>,
    /// Per state block, in added order.
    blocks: Vec<Option<Box<dyn Any + Send>>>,
}

impl World {
    /// The snapshot (`state == false`) or the state stream the hash covers.
    fn write(&self, state: bool) -> Vec<u8> {
        self.write_columns(state, false)
    }

    /// The writer; `unguarded` reads a column the running system holds
    /// mutably as it is ([`crate::SystemContext::snapshot`]).
    fn write_columns(&self, state: bool, unguarded: bool) -> Vec<u8> {
        let mut w = Writer(Vec::new());
        w.bytes(MAGIC);
        let format = match (state, self.blocks.is_empty()) {
            (true, _) => FORMAT_STATE,
            (false, true) => FORMAT,
            (false, false) => FORMAT_BLOCKS,
        };
        w.u32(format as usize);

        w.u32(self.registry.len());
        for t in &self.registry.types {
            w.sized(t.schema.name.as_bytes());
            w.u64(t.layout_hash);
            if !state {
                w.u32(t.schema.version as usize);
                w.u32(t.schema.fields.len());
                for f in &t.schema.fields {
                    w.sized(f.name.as_bytes());
                    w.sized(f.ty.name().as_bytes());
                    w.u32(f.variants.len());
                    for v in &f.variants {
                        w.sized(v.as_bytes());
                    }
                }
            }
        }

        let entities = self.entities.borrow();
        // Inside a system, an entity its queued spawn reserved is in no
        // archetype yet: the snapshot has it freed (as a despawn would), so
        // the world it restores to is whole.
        let pending = |i: usize| {
            unguarded
                && entities
                    .locations
                    .get(i)
                    .is_some_and(|l| l.archetype == PENDING)
        };
        w.u32(entities.alloc.slot_count());
        let mut freed = Vec::new();
        for (i, slot) in entities.alloc.slots().enumerate() {
            if slot.alive && pending(i) {
                w.u32(slot.generation as usize + 1);
                w.u8(0);
                if slot.generation + 1 < og_core::handle::RETIRED_GENERATION {
                    freed.push(i);
                }
            } else {
                w.u32(slot.generation as usize);
                w.u8(u8::from(slot.alive));
            }
        }
        w.u32(entities.alloc.free_list().len() + freed.len());
        for &i in entities.alloc.free_list() {
            w.u32(i as usize);
        }
        for i in freed.into_iter().rev() {
            w.u32(i);
        }

        w.u32(self.archetypes.len());
        for arch in &self.archetypes {
            w.u32(arch.types.len());
            for id in &arch.types {
                w.u32(id.0 as usize);
            }
            w.u32(arch.len());
            for e in &arch.entities {
                w.u64(e.to_bits());
            }
            for c in &arch.columns {
                match c.try_borrow() {
                    Ok(col) => w.bytes(col.bytes()),
                    Err(_) if unguarded => {
                        // SAFETY: only the running system holds a column
                        // mutably (its typed views), and it is not running
                        // while this does: its host is blocked in the call
                        // that asked for the snapshot, on this thread, so
                        // nothing writes the column meanwhile and the
                        // shared reference ends before this returns.
                        w.bytes(unsafe { &*c.as_ptr() }.bytes());
                    }
                    Err(_) => panic!("a column is borrowed outside a system"),
                }
            }
        }

        let present: Vec<usize> = (0..self.resources.len())
            .filter(|&i| self.resources[i].is_some())
            .collect();
        w.u32(present.len());
        for i in present {
            let id = SchemaId(i as u32);
            let value = self.resources[i].as_ref().expect("present").borrow();
            let bytes = resource_to_bytes(&self.registry, id, &value)
                .expect("resources were validated against their layout on insert");
            w.u32(i);
            w.sized(&bytes);
        }

        let queues: Vec<usize> = (0..self.events.len())
            .filter(|&i| {
                let q = self.events[i].borrow();
                !q.current.is_empty() || !q.pending.is_empty()
            })
            .collect();
        w.u32(queues.len());
        for i in queues {
            let q = self.events[i].borrow();
            w.u32(i);
            for list in [&q.current, &q.pending] {
                w.u32(list.len());
                for e in list.iter() {
                    w.bytes(e);
                }
            }
        }

        if !state || !self.names.is_empty() {
            if state {
                w.bytes(NAMES_TAG);
            }
            w.u32(self.names.len());
            for (e, name) in &self.names {
                w.u64(e.to_bits());
                w.sized(name.as_bytes());
            }
        }

        if !self.blocks.is_empty() {
            if state {
                w.bytes(BLOCKS_TAG);
            }
            w.u32(self.blocks.len());
            let mut saved = Vec::new();
            for b in &self.blocks {
                saved.clear();
                b.block.save(&mut saved);
                w.sized(b.name.as_bytes());
                w.sized(&saved);
            }
        }
        w.0
    }

    /// The whole world as bytes (format 2, or 3 with state blocks). See the
    /// module docs for what is included.
    pub fn snapshot(&self) -> Vec<u8> {
        self.write(false)
    }

    /// [`Self::snapshot`] from inside a running system, whose columns its
    /// host holds while it runs ([`crate::SystemContext::snapshot`]).
    pub(crate) fn snapshot_in_system(&self) -> Vec<u8> {
        self.write_columns(false, true)
    }

    /// Whether [`Self::restore`] (or, with `migrate`, [`Self::restore_migrating`])
    /// would take `bytes`, without changing anything: the same refusals,
    /// a type of another layout naming the fields that differ.
    pub fn check_snapshot(&self, bytes: &[u8], migrate: bool) -> Result<()> {
        let parsed = parse(&self.registry, bytes)?;
        if migrate {
            let mut report = MigrationReport::default();
            for t in &parsed.types {
                if let Ok(id) = self.registry.id_by_name(&t.name) {
                    plan(&self.registry, t, id, &mut report)?;
                }
            }
            decode_blocks(&self.blocks, &parsed.blocks, Some(&mut report))?;
            return Ok(());
        }
        for t in &parsed.types {
            let id = self
                .registry
                .id_by_name(&t.name)
                .map_err(|_| not_registered(&t.name))?;
            let info = self.registry.info(id);
            if info.layout_hash != t.hash {
                let detail = t
                    .schema
                    .as_ref()
                    .map(|saved| layout_difference(saved, &info.schema))
                    .unwrap_or_default();
                return Err(Error::new(
                    ErrorKind::Schema,
                    format!(
                        "snapshot: type '{}' has a different layout in this world{detail}",
                        t.name
                    ),
                ));
            }
        }
        decode_blocks(&self.blocks, &parsed.blocks, None)?;
        Ok(())
    }

    /// The stable hash of the world: FNV-1a of its state stream (the module
    /// docs say exactly what that is). Equal exactly when two worlds' state
    /// is identical; a snapshot restored hashes as the world it came from.
    pub fn world_hash(&self) -> u64 {
        hash_bytes(&self.write(true))
    }

    /// Replaces the world's state (entities, components, resources, events,
    /// names, state blocks) with a snapshot's. Systems and registrations stay. Refused,
    /// with nothing changed, when the snapshot is malformed or names a type
    /// this world lacks or has with another layout.
    pub fn restore(&mut self, bytes: &[u8]) -> Result<()> {
        let parsed = parse(&self.registry, bytes)?;
        let mut map = Vec::with_capacity(parsed.types.len());
        for t in &parsed.types {
            let id = self
                .registry
                .id_by_name(&t.name)
                .map_err(|_| not_registered(&t.name))?;
            if self.registry.info(id).layout_hash != t.hash {
                return Err(other_layout(&t.name));
            }
            map.push(id);
        }

        let mut locations = vec![
            Location {
                archetype: PENDING,
                row: 0
            };
            parsed.slot_count
        ];
        let mut archetypes = Vec::with_capacity(parsed.archetypes.len());
        let mut by_types = BTreeMap::new();
        for (a, snap) in parsed.archetypes.iter().enumerate() {
            let stored: Vec<SchemaId> = snap.stored.iter().map(|&t| map[t]).collect();
            let mut types = stored.clone();
            types.sort();
            if by_types.insert(types.clone(), a as u32).is_some() {
                return Err(bad(format!("archetype {a} duplicates another")));
            }
            let mut arch = Archetype::new(types, &self.registry);
            let rows = snap.entities.len();
            for (row, e) in snap.entities.iter().enumerate() {
                locations[e.index() as usize] = Location {
                    archetype: a as u32,
                    row: row as u32,
                };
            }
            arch.entities = snap.entities.clone();
            for (s, id) in stored.iter().enumerate() {
                let position = arch.position(*id).expect("own type");
                for (c, data) in arch.columns_of(position).zip(&snap.columns[s]) {
                    arch.columns[c].get_mut().fill_from(rows, data);
                }
            }
            archetypes.push(arch);
        }

        let mut resources: Vec<Option<RefCell<ResourceValue>>> =
            (0..self.registry.len()).map(|_| None).collect();
        for &(t, data) in &parsed.resources {
            let id = map[t];
            resources[id.0 as usize] =
                Some(RefCell::new(resource_value(&self.registry, id, data)?));
        }
        if resources[self.time_id.0 as usize].is_none() {
            return Err(bad("no Time resource"));
        }

        let mut events: Vec<RefCell<EventQueue>> = (0..self.registry.len())
            .map(|_| Default::default())
            .collect();
        for (t, current, pending) in &parsed.events {
            let queue = events[map[*t].0 as usize].get_mut();
            queue.current = current.iter().map(|b| b.to_vec()).collect();
            queue.pending = pending.iter().map(|b| b.to_vec()).collect();
        }
        let blocks = decode_blocks(&self.blocks, &parsed.blocks, None)?;

        self.commit(State {
            alloc: parsed.alloc,
            locations,
            archetypes,
            by_types,
            resources,
            events,
            names: parsed.names,
            blocks,
        });
        Ok(())
    }

    /// Restores a snapshot taken by a world whose types have since changed
    /// (a hot reload): types are mapped by name and, where a layout differs,
    /// fields by name. A field this world added starts at its default, a
    /// field the snapshot had and this world dropped is dropped, a field
    /// whose type changed starts at its default, enum values are mapped by
    /// variant name, and a type this world no longer registers is dropped
    /// with its components, resources and events. A resource of a type the
    /// snapshot does not know keeps its current value. Entity ids, iteration
    /// order (archetypes that only a dropped type kept apart are merged, in
    /// snapshot order), names, `Time` and the rng states are kept.
    ///
    /// Refused, with nothing changed, for a malformed snapshot, or a format
    /// 1 snapshot whose types do not all fit this world exactly (format 1
    /// has no field names). The report says what was mapped; for a snapshot
    /// that fits exactly it is empty and the result equals [`Self::restore`].
    pub fn restore_migrating(&mut self, bytes: &[u8]) -> Result<MigrationReport> {
        let parsed = parse(&self.registry, bytes).map_err(|e| match e.kind() {
            ErrorKind::NotFound | ErrorKind::Schema => e.context(
                "a format 1 snapshot carries no field names, so it restores only into a world whose types fit it exactly",
            ),
            _ => e,
        })?;
        let mut report = MigrationReport::default();
        let mut plans: Vec<Option<Plan>> = Vec::with_capacity(parsed.types.len());
        for t in &parsed.types {
            let Ok(id) = self.registry.id_by_name(&t.name) else {
                report.dropped.push(t.name.clone());
                plans.push(None);
                continue;
            };
            let p = plan(&self.registry, t, id, &mut report)?;
            plans.push(Some(p));
        }
        let storable = |t: usize| plans[t].as_ref();

        let mut locations = vec![
            Location {
                archetype: PENDING,
                row: 0
            };
            parsed.slot_count
        ];
        let mut archetypes: Vec<Archetype> = Vec::new();
        let mut by_types: BTreeMap<Vec<SchemaId>, u32> = BTreeMap::new();
        let mut buf = Vec::new();
        for snap in &parsed.archetypes {
            // (index in snap.stored, plan) of the types that survive.
            let mut kept: Vec<(usize, &Plan)> = snap
                .stored
                .iter()
                .enumerate()
                .filter_map(|(s, &t)| storable(t).map(|p| (s, p)))
                .collect();
            kept.sort_by_key(|(_, p)| p.target);
            let types: Vec<SchemaId> = kept.iter().map(|(_, p)| p.target).collect();
            let a = match by_types.get(&types) {
                Some(&a) => a,
                None => {
                    let a = archetypes.len() as u32;
                    archetypes.push(Archetype::new(types.clone(), &self.registry));
                    by_types.insert(types, a);
                    a
                }
            };
            let arch = &mut archetypes[a as usize];
            let rows = snap.entities.len();
            for e in &snap.entities {
                locations[e.index() as usize] = Location {
                    archetype: a,
                    row: arch.entities.len() as u32,
                };
                arch.entities.push(*e);
            }
            for (position, (s, p)) in kept.iter().enumerate() {
                for (k, c) in arch.columns_of(position).enumerate() {
                    buf.clear();
                    p.column(k, rows, &snap.columns[*s], &mut buf);
                    arch.columns[c].get_mut().extend_from(rows, &buf);
                }
            }
        }

        let mut resources: Vec<Option<RefCell<ResourceValue>>> =
            (0..self.registry.len()).map(|_| None).collect();
        for &(t, data) in &parsed.resources {
            let Some(p) = storable(t) else { continue };
            buf.clear();
            p.convert(data, &mut buf);
            let value = resource_value(&self.registry, p.target, &buf)?;
            resources[p.target.0 as usize] = Some(RefCell::new(value));
        }
        // Types the snapshot does not know keep their current resource.
        let unknown: Vec<usize> = (0..self.registry.len())
            .filter(|&i| {
                let name = &self.registry.types[i].schema.name;
                !parsed.types.iter().any(|t| &t.name == name)
            })
            .collect();
        let time = self.time_id.0 as usize;
        if resources[time].is_none() && !(unknown.contains(&time) && self.resources[time].is_some())
        {
            return Err(bad("no Time resource"));
        }

        let mut events: Vec<RefCell<EventQueue>> = (0..self.registry.len())
            .map(|_| Default::default())
            .collect();
        for (t, current, pending) in &parsed.events {
            let Some(p) = &plans[*t] else { continue };
            let size = self.registry.info(p.target).layout.size();
            let queue = events[p.target.0 as usize].get_mut();
            for (from, to) in [(current, &mut queue.current), (pending, &mut queue.pending)] {
                for value in from {
                    let mut out = Vec::with_capacity(size);
                    p.convert(value, &mut out);
                    to.push(out);
                }
            }
        }

        let blocks = decode_blocks(&self.blocks, &parsed.blocks, Some(&mut report))?;

        // Nothing below fails.
        for i in unknown {
            resources[i] = self.resources[i].take();
        }
        self.commit(State {
            alloc: parsed.alloc,
            locations,
            archetypes,
            by_types,
            resources,
            events,
            names: parsed.names,
            blocks,
        });
        Ok(report)
    }

    fn commit(&mut self, state: State) {
        *self.entities.get_mut() = Entities {
            alloc: state.alloc,
            locations: state.locations,
        };
        self.archetypes = state.archetypes;
        self.archetype_by_types = state.by_types;
        self.resources = state.resources;
        self.events = state.events;
        self.by_name = state.names.iter().map(|(e, n)| (n.clone(), *e)).collect();
        self.names = state.names.into_iter().collect();
        for (b, decoded) in self.blocks.iter_mut().zip(state.blocks) {
            b.block.apply(decoded);
        }
        self.structural += 1;
    }
}
