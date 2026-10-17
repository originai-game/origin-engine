//! Whole-world snapshot and restore, and the world hash.
//!
//! A snapshot is everything that determines what the world does next: the
//! type table (names and layout hashes), the entity allocator (every slot's
//! generation and the free list in reuse order), every archetype in creation
//! order with its rows in storage order (so iteration order survives), every
//! resource (including `Time` and the `Rng` states) and both event queues.
//! Systems are code and are not in it; restore into a world whose schedule
//! and registrations match.
//!
//! Format (little-endian): `"OGWS"`, format version `u32`, then the sections
//! above. Restore maps types by name, requires equal layout hashes, validates
//! every count and length, and changes nothing unless the whole snapshot is
//! valid. [`World::world_hash`] is the FNV-1a hash of the snapshot bytes, so
//! two worlds have the same hash exactly when their snapshots are identical.

use crate::entity::Entity;
use crate::registry::SchemaId;
use crate::storage::Archetype;
use crate::world::{
    resource_to_bytes, Entities, EventQueue, Location, ResourceValue, World, PENDING,
};
use og_core::handle::{HandleAllocator, RawHandle, SlotState};
use og_core::hash::hash_bytes;
use og_core::{Error, ErrorKind, Result};
use std::cell::RefCell;
use std::collections::BTreeMap;

const MAGIC: &[u8; 4] = b"OGWS";
const FORMAT: u32 = 1;

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
}

impl World {
    /// The whole world as bytes. See the module docs for what is included.
    pub fn snapshot(&self) -> Vec<u8> {
        let mut w = Writer(Vec::new());
        w.bytes(MAGIC);
        w.u32(FORMAT as usize);

        w.u32(self.registry.len());
        for t in &self.registry.types {
            w.sized(t.schema.name.as_bytes());
            w.u64(t.layout_hash);
        }

        let entities = self.entities.borrow();
        w.u32(entities.alloc.slot_count());
        for slot in entities.alloc.slots() {
            w.u32(slot.generation as usize);
            w.u8(u8::from(slot.alive));
        }
        w.u32(entities.alloc.free_list().len());
        for &i in entities.alloc.free_list() {
            w.u32(i as usize);
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
                w.bytes(c.borrow().bytes());
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
        w.0
    }

    /// The stable hash of the world: FNV-1a of [`Self::snapshot`].
    pub fn world_hash(&self) -> u64 {
        hash_bytes(&self.snapshot())
    }

    /// Replaces the world's state (entities, components, resources, events)
    /// with a snapshot's. Systems and registrations stay. Refused, with
    /// nothing changed, when the snapshot is malformed or names a type this
    /// world lacks or has with another layout.
    pub fn restore(&mut self, bytes: &[u8]) -> Result<()> {
        let mut r = Reader { buf: bytes, pos: 0 };
        if r.take(4)? != MAGIC {
            return Err(bad("not a world snapshot (magic)"));
        }
        let format = r.u32()?;
        if format != FORMAT as usize {
            return Err(bad(format!("format {format}, this engine reads {FORMAT}")));
        }

        // Types: snapshot index -> this world's id.
        let type_count = r.count(12)?;
        let mut map = Vec::with_capacity(type_count);
        for _ in 0..type_count {
            let name =
                std::str::from_utf8(r.sized()?).map_err(|_| bad("type name is not UTF-8"))?;
            let hash = r.u64()?;
            let id = self.registry.id_by_name(name).map_err(|_| {
                Error::new(
                    ErrorKind::NotFound,
                    format!("snapshot: type '{name}' is not registered in this world"),
                )
            })?;
            if self.registry.info(id).layout_hash != hash {
                return Err(Error::new(
                    ErrorKind::Schema,
                    format!("snapshot: type '{name}' has a different layout in this world"),
                ));
            }
            map.push(id);
        }
        let mapped = |i: usize| {
            map.get(i)
                .copied()
                .ok_or_else(|| bad(format!("type index {i} out of range")))
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
        let mut locations = vec![
            Location {
                archetype: PENDING,
                row: 0
            };
            slot_count
        ];

        // Archetypes, in creation order.
        let arch_count = r.count(8)?;
        if arch_count == 0 {
            return Err(bad("no archetypes (the empty archetype is always present)"));
        }
        let mut archetypes = Vec::with_capacity(arch_count);
        let mut by_types = BTreeMap::new();
        let mut placed = 0usize;
        for a in 0..arch_count {
            let k = r.count(4)?;
            let mut stored = Vec::with_capacity(k);
            for _ in 0..k {
                stored.push(mapped(r.u32()?)?);
            }
            let mut types = stored.clone();
            types.sort();
            if types.windows(2).any(|w| w[0] == w[1]) {
                return Err(bad(format!("archetype {a} repeats a type")));
            }
            if (a == 0) != types.is_empty() {
                return Err(bad("archetype 0 must be the empty archetype, and only it"));
            }
            if by_types.insert(types.clone(), a as u32).is_some() {
                return Err(bad(format!("archetype {a} duplicates another")));
            }
            let mut arch = Archetype::new(types, &self.registry);
            let rows = r.count(8)?;
            for row in 0..rows {
                let e = Entity(RawHandle::from_bits(r.u64()?));
                if !alloc.is_alive(e.0) {
                    return Err(bad(format!(
                        "archetype {a} holds entity {e}, which is not alive"
                    )));
                }
                let loc = &mut locations[e.index() as usize];
                if loc.archetype != PENDING {
                    return Err(bad(format!("entity {e} appears twice")));
                }
                *loc = Location {
                    archetype: a as u32,
                    row: row as u32,
                };
                arch.entities.push(e);
            }
            placed += rows;
            for id in &stored {
                let position = arch.position(*id).expect("own type");
                for c in arch.columns_of(position) {
                    let col = arch.columns[c].get_mut();
                    let len = rows * col.size();
                    col.fill_from(rows, r.take(len)?);
                }
            }
            archetypes.push(arch);
        }
        if placed != alloc.len() {
            return Err(bad(format!(
                "{} entities are alive but {placed} are stored",
                alloc.len()
            )));
        }

        // Resources.
        let mut resources: Vec<Option<RefCell<ResourceValue>>> =
            (0..self.registry.len()).map(|_| None).collect();
        let resource_count = r.count(8)?;
        for _ in 0..resource_count {
            let id = mapped(r.u32()?)?;
            let data = r.sized()?;
            self.registry
                .check_size(id, data)
                .map_err(|e| bad(e.reason()))?;
            let info = self.registry.info(id);
            let value = match &info.rust {
                Some(rust) => ResourceValue::Typed((rust.decode)(&info.layout, data)?),
                None => ResourceValue::Raw(data.to_vec()),
            };
            if resources[id.0 as usize]
                .replace(RefCell::new(value))
                .is_some()
            {
                return Err(bad(format!(
                    "resource '{}' appears twice",
                    self.registry.name(id)
                )));
            }
        }
        if resources[self.time_id.0 as usize].is_none() {
            return Err(bad("no Time resource"));
        }

        // Events.
        let mut events: Vec<RefCell<EventQueue>> = (0..self.registry.len())
            .map(|_| Default::default())
            .collect();
        let queue_count = r.count(12)?;
        for _ in 0..queue_count {
            let id = mapped(r.u32()?)?;
            let size = self.registry.info(id).layout.size();
            let queue = events[id.0 as usize].get_mut();
            if !queue.current.is_empty() || !queue.pending.is_empty() {
                return Err(bad(format!(
                    "event queue '{}' appears twice",
                    self.registry.name(id)
                )));
            }
            for list in [&mut queue.current, &mut queue.pending] {
                let n = r.count(size)?;
                if size == 0 && n > 1 << 24 {
                    return Err(bad(format!("{n} field-less events")));
                }
                for _ in 0..n {
                    list.push(r.take(size)?.to_vec());
                }
            }
        }
        if r.pos != bytes.len() {
            return Err(bad(format!("{} trailing bytes", bytes.len() - r.pos)));
        }

        *self.entities.get_mut() = Entities { alloc, locations };
        self.archetypes = archetypes;
        self.archetype_by_types = by_types;
        self.resources = resources;
        self.events = events;
        Ok(())
    }
}
