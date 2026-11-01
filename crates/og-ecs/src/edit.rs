//! A snapshot opened for editing ([`World::edit_snapshot`]): what a save
//! migration changes before the world restores it, and selections
//! ([`World::snapshot_selection`], [`World::restore_selection`]): the
//! tagged entities and chosen resources of a world, without the rest.
//!
//! An edit holds the snapshot's parts owned: its type table (each type's
//! field table as it was saved), the archetypes in creation order with
//! their rows in storage order, the resources, the names and, for a whole
//! world, the allocator, the event queues, the state blocks and the
//! resume. [`SnapshotEdit::to_bytes`] writes them back in the format they
//! came in, so an edit that changed nothing writes the snapshot it read,
//! byte for byte. The edits keep the rest of the snapshot as it was:
//! renaming or reshaping a type moves its rows into the archetype of their
//! new component set, created where the first such row is, so entity ids,
//! names, the other types' values and iteration order stay.
//!
//! A selection (format 5) has the type table, the types an entity had to
//! carry to be selected, those entities' archetypes, the chosen resources
//! and the entities' names; its entity ids are the saved world's, mapped
//! to new entities when it is restored.

use crate::entity::Entity;
use crate::registry::SchemaId;
use crate::snapshot::{
    FORMAT_SELECTION, MigrationReport, Parsed, Writer, bad, parse_any as parse, plan,
};
use crate::world::{MAX_NAME, World};
use og_core::handle::{HandleAllocator, RawHandle, SlotState};
use og_core::reflect::Schema;
use og_core::{Error, ErrorKind, Result};
use std::collections::{BTreeMap, BTreeSet};

/// One archetype of an edit: its types (indices into the type table) in
/// stored order, its entities, and per type, per field, `rows × size` bytes.
#[derive(Clone, Debug)]
struct Arch {
    types: Vec<usize>,
    entities: Vec<Entity>,
    columns: Vec<Vec<Vec<u8>>>,
    /// Kept when it ends up with no rows (it had none in the snapshot).
    keep: bool,
}

/// One event queue: type, the events delivered this tick, those for the next.
type EventList = (usize, Vec<Vec<u8>>, Vec<Vec<u8>>);

/// A saved entity and its components, converted: (type, packed bytes).
type SavedRow = (Entity, Vec<(SchemaId, Vec<u8>)>);

/// What only a whole world's snapshot has.
#[derive(Clone, Debug)]
struct Whole {
    slots: Vec<SlotState>,
    free: Vec<u32>,
    /// Per type: the events delivered this tick, and those for the next.
    events: Vec<EventList>,
    blocks: Vec<(String, Vec<u8>)>,
    resume: Option<String>,
}

/// One type's values to put into a snapshot ([`SnapshotEdit::replace_types`]):
/// the type's schema, its components by entity and its resource.
#[derive(Clone, Debug)]
pub struct TypeValues {
    pub schema: Schema,
    pub components: Vec<(Entity, Vec<u8>)>,
    pub resource: Option<Vec<u8>>,
}

/// A snapshot or a selection read into owned parts. See the module docs.
#[derive(Clone, Debug)]
pub struct SnapshotEdit {
    types: Vec<Schema>,
    archetypes: Vec<Arch>,
    resources: Vec<(usize, Vec<u8>)>,
    names: Vec<(Entity, String)>,
    whole: Option<Whole>,
    /// A selection's filter (empty for a whole world).
    filter: Vec<usize>,
}

fn sizes(schema: &Schema) -> Vec<usize> {
    schema.fields.iter().map(|f| f.ty.size()).collect()
}

fn write_types(w: &mut Writer, types: &[Schema]) {
    w.u32(types.len());
    for t in types {
        w.sized(t.name.as_bytes());
        w.u64(t.layout_hash());
        w.u32(t.version as usize);
        w.u32(t.fields.len());
        for f in &t.fields {
            w.sized(f.name.as_bytes());
            w.sized(f.ty.name().as_bytes());
            w.u32(f.variants.len());
            for v in &f.variants {
                w.sized(v.as_bytes());
            }
        }
    }
}

impl SnapshotEdit {
    fn from_parsed(p: Parsed<'_>) -> Result<SnapshotEdit> {
        let mut types = Vec::with_capacity(p.types.len());
        for t in &p.types {
            let Some(schema) = &t.schema else {
                return Err(Error::new(
                    ErrorKind::Schema,
                    "snapshot: a format 1 snapshot carries no field table, so it cannot be migrated",
                ));
            };
            types.push(schema.clone());
        }
        let archetypes = p
            .archetypes
            .iter()
            .map(|a| Arch {
                types: a.stored.clone(),
                entities: a.entities.clone(),
                columns: a
                    .columns
                    .iter()
                    .map(|fields| fields.iter().map(|c| c.to_vec()).collect())
                    .collect(),
                keep: a.entities.is_empty(),
            })
            .collect();
        let whole = match p.selection {
            Some(_) => None,
            None => Some(Whole {
                slots: p.alloc.slots().collect(),
                free: p.alloc.free_list().to_vec(),
                events: p
                    .events
                    .iter()
                    .map(|(t, c, n)| {
                        (
                            *t,
                            c.iter().map(|b| b.to_vec()).collect(),
                            n.iter().map(|b| b.to_vec()).collect(),
                        )
                    })
                    .collect(),
                blocks: p
                    .blocks
                    .iter()
                    .map(|(n, b)| (n.to_string(), b.to_vec()))
                    .collect(),
                resume: p.resume.map(str::to_string),
            }),
        };
        Ok(SnapshotEdit {
            types,
            archetypes,
            resources: p.resources.iter().map(|(t, b)| (*t, b.to_vec())).collect(),
            names: p.names,
            whole,
            filter: p.selection.unwrap_or_default(),
        })
    }

    /// Reads a snapshot (or a selection) of format 2 to 5 without a world:
    /// what a host's save migrations open before it checks the save
    /// against its world. Refused for a malformed one, or format 1.
    pub fn from_bytes(bytes: &[u8]) -> Result<SnapshotEdit> {
        SnapshotEdit::from_parsed(parse(&crate::registry::Registry::default(), bytes)?)
    }

    /// Whether this is a selection rather than a whole world.
    pub fn is_selection(&self) -> bool {
        self.whole.is_none()
    }

    /// The saved field table of every type, in table order.
    pub fn schemas(&self) -> &[Schema] {
        &self.types
    }

    fn index(&self, name: &str) -> Option<usize> {
        self.types.iter().position(|t| t.name == name)
    }

    /// The saved field table of a type.
    pub fn schema(&self, name: &str) -> Option<&Schema> {
        self.index(name).map(|i| &self.types[i])
    }

    /// The selection's filter: the types an entity carried to be selected.
    pub fn filter(&self) -> Vec<&str> {
        self.filter
            .iter()
            .map(|&t| self.types[t].name.as_str())
            .collect()
    }

    /// Every entity in storage order.
    pub fn entities(&self) -> Vec<Entity> {
        self.archetypes
            .iter()
            .flat_map(|a| a.entities.iter().copied())
            .collect()
    }

    /// The entity names, in entity order.
    pub fn names(&self) -> &[(Entity, String)] {
        &self.names
    }

    /// Every component of a type, packed, by entity in storage order.
    pub fn components(&self, ty: &str) -> Vec<(Entity, Vec<u8>)> {
        let Some(t) = self.index(ty) else {
            return Vec::new();
        };
        let sizes = sizes(&self.types[t]);
        let mut out = Vec::new();
        for a in &self.archetypes {
            let Some(s) = a.types.iter().position(|&u| u == t) else {
                continue;
            };
            for (row, e) in a.entities.iter().enumerate() {
                let mut value = Vec::with_capacity(sizes.iter().sum());
                for (k, &size) in sizes.iter().enumerate() {
                    value.extend_from_slice(&a.columns[s][k][row * size..(row + 1) * size]);
                }
                out.push((*e, value));
            }
        }
        out
    }

    /// A type's resource, packed.
    pub fn resource(&self, ty: &str) -> Option<&[u8]> {
        let t = self.index(ty)?;
        self.resources
            .iter()
            .find(|(u, _)| *u == t)
            .map(|(_, b)| b.as_slice())
    }

    /// Whether a type has events queued.
    pub fn has_events(&self, ty: &str) -> bool {
        let Some(t) = self.index(ty) else {
            return false;
        };
        self.whole
            .as_ref()
            .is_some_and(|w| w.events.iter().any(|(u, ..)| *u == t))
    }

    /// The names of the state blocks, in saved order.
    pub fn blocks(&self) -> Vec<&str> {
        self.whole
            .iter()
            .flat_map(|w| w.blocks.iter().map(|(n, _)| n.as_str()))
            .collect()
    }

    /// A state block's saved bytes.
    pub fn block(&self, name: &str) -> Option<&[u8]> {
        self.whole
            .as_ref()?
            .blocks
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, b)| b.as_slice())
    }

    /// Replaces a state block's saved bytes. Refused for a block the
    /// snapshot does not have.
    pub fn set_block(&mut self, name: &str, bytes: Vec<u8>) -> Result<()> {
        let slot = self
            .whole
            .as_mut()
            .and_then(|w| w.blocks.iter_mut().find(|(n, _)| n == name))
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::NotFound,
                    format!("snapshot: no state block '{name}'"),
                )
            })?;
        slot.1 = bytes;
        Ok(())
    }

    /// Converts every value of a type (its components, its resource and its
    /// queued events) with `convert`, which takes a value packed as the
    /// saved schema and returns it packed as `schema`, which the type then
    /// has. Nothing changes when a conversion is refused.
    pub fn map_type(
        &mut self,
        name: &str,
        schema: Schema,
        convert: &mut dyn FnMut(&[u8]) -> Result<Vec<u8>>,
    ) -> Result<()> {
        let t = self.index(name).ok_or_else(|| {
            Error::new(ErrorKind::NotFound, format!("snapshot: no type '{name}'"))
        })?;
        let old = sizes(&self.types[t]);
        let new = sizes(&schema);
        let size: usize = new.iter().sum();
        let check = |v: Vec<u8>| -> Result<Vec<u8>> {
            if v.len() == size {
                Ok(v)
            } else {
                Err(bad(format!(
                    "a converted '{name}' is {} bytes, its layout {size}",
                    v.len()
                )))
            }
        };
        let mut archetypes = self.archetypes.clone();
        for a in &mut archetypes {
            let Some(s) = a.types.iter().position(|&u| u == t) else {
                continue;
            };
            let mut columns: Vec<Vec<u8>> = new
                .iter()
                .map(|n| Vec::with_capacity(n * a.entities.len()))
                .collect();
            for row in 0..a.entities.len() {
                let mut value = Vec::new();
                for (k, &n) in old.iter().enumerate() {
                    value.extend_from_slice(&a.columns[s][k][row * n..(row + 1) * n]);
                }
                let v = check(convert(&value)?)?;
                let mut at = 0;
                for (k, &n) in new.iter().enumerate() {
                    columns[k].extend_from_slice(&v[at..at + n]);
                    at += n;
                }
            }
            a.columns[s] = columns;
        }
        let mut resources = self.resources.clone();
        for (u, b) in &mut resources {
            if *u == t {
                *b = check(convert(b)?)?;
            }
        }
        let mut whole = self.whole.clone();
        if let Some(w) = &mut whole {
            for (u, current, pending) in &mut w.events {
                if *u == t {
                    for list in [current, pending] {
                        for b in list.iter_mut() {
                            *b = check(convert(b)?)?;
                        }
                    }
                }
            }
        }
        self.archetypes = archetypes;
        self.resources = resources;
        self.whole = whole;
        self.types[t] = schema;
        Ok(())
    }

    /// Removes the types `remove` names (their components, resources and
    /// event queues) and adds `add`'s types with their values, in one
    /// step: an entity's rows move to the archetype of its new component
    /// set, created where the first such row is. Refused, with nothing
    /// changed, for a type to remove the snapshot lacks, a type to add it
    /// has (after the removal), a value of the wrong size, or a component
    /// of an entity the snapshot does not hold.
    pub fn replace_types(&mut self, remove: &[&str], add: Vec<TypeValues>) -> Result<()> {
        let mut removed = BTreeSet::new();
        for name in remove {
            let t = self.index(name).ok_or_else(|| {
                Error::new(
                    ErrorKind::NotFound,
                    format!("snapshot: no type '{name}' to remove"),
                )
            })?;
            removed.insert(t);
        }
        let live: BTreeSet<Entity> = self.entities().into_iter().collect();
        // The new type table: the kept types in order, then the added ones.
        let mut renumber = vec![usize::MAX; self.types.len()];
        let mut types = Vec::new();
        for (i, t) in self.types.iter().enumerate() {
            if !removed.contains(&i) {
                renumber[i] = types.len();
                types.push(t.clone());
            }
        }
        // Per entity, the added components: (new type index, bytes).
        let mut extra: BTreeMap<Entity, Vec<(usize, Vec<u8>)>> = BTreeMap::new();
        let mut resources: Vec<(usize, Vec<u8>)> = self
            .resources
            .iter()
            .filter(|(t, _)| !removed.contains(t))
            .map(|(t, b)| (renumber[*t], b.clone()))
            .collect();
        for v in add {
            let name = v.schema.name.clone();
            if types.iter().any(|t| t.name == name) {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    format!("snapshot: type '{name}' is already in it"),
                ));
            }
            let size: usize = sizes(&v.schema).iter().sum();
            let at = types.len();
            let mut seen = BTreeSet::new();
            for (e, bytes) in v.components {
                if !live.contains(&e) {
                    return Err(bad(format!(
                        "a '{name}' for entity {e}, which it does not hold"
                    )));
                }
                if !seen.insert(e) {
                    return Err(bad(format!("entity {e} has two '{name}'")));
                }
                if bytes.len() != size {
                    return Err(bad(format!(
                        "a '{name}' of {} bytes, its layout {size}",
                        bytes.len()
                    )));
                }
                extra.entry(e).or_default().push((at, bytes));
            }
            if let Some(bytes) = v.resource {
                if bytes.len() != size {
                    return Err(bad(format!(
                        "a '{name}' resource of {} bytes, its layout {size}",
                        bytes.len()
                    )));
                }
                resources.push((at, bytes));
            }
            types.push(v.schema);
        }
        let new_sizes: Vec<Vec<usize>> = types.iter().map(sizes).collect();
        let mut archetypes: Vec<Arch> = Vec::new();
        let mut by_key: BTreeMap<Vec<usize>, usize> = BTreeMap::new();
        let mut place = |archetypes: &mut Vec<Arch>, stored: Vec<usize>, keep: bool| -> usize {
            let mut key = stored.clone();
            key.sort_unstable();
            *by_key.entry(key).or_insert_with(|| {
                archetypes.push(Arch {
                    columns: stored
                        .iter()
                        .map(|&t| vec![Vec::new(); new_sizes[t].len()])
                        .collect(),
                    types: stored,
                    entities: Vec::new(),
                    keep,
                });
                archetypes.len() - 1
            })
        };
        let whole = self.whole.is_some();
        for (i, a) in self.archetypes.iter().enumerate() {
            let kept: Vec<(usize, usize)> = a
                .types
                .iter()
                .enumerate()
                .filter(|(_, t)| !removed.contains(t))
                .map(|(s, &t)| (s, renumber[t]))
                .collect();
            let base: Vec<usize> = kept.iter().map(|&(_, t)| t).collect();
            // A whole world's first archetype is the empty one, and one
            // that had no rows in the snapshot stays where it was.
            if (whole && i == 0) || a.keep {
                place(&mut archetypes, base.clone(), true);
            }
            for (row, e) in a.entities.iter().enumerate() {
                let added = extra.get(e).map(Vec::as_slice).unwrap_or(&[]);
                let mut stored = base.clone();
                stored.extend(added.iter().map(|(t, _)| *t));
                let at = place(&mut archetypes, stored, false);
                let target = &mut archetypes[at];
                target.entities.push(*e);
                for &(s, t) in &kept {
                    let position = target.types.iter().position(|&u| u == t).expect("own type");
                    for (k, &n) in new_sizes[t].iter().enumerate() {
                        target.columns[position][k]
                            .extend_from_slice(&a.columns[s][k][row * n..(row + 1) * n]);
                    }
                }
                for (t, bytes) in added {
                    let position = target.types.iter().position(|u| u == t).expect("own type");
                    let mut offset = 0;
                    for (k, &n) in new_sizes[*t].iter().enumerate() {
                        target.columns[position][k].extend_from_slice(&bytes[offset..offset + n]);
                        offset += n;
                    }
                }
            }
        }
        let mut whole = self.whole.clone();
        if let Some(w) = &mut whole {
            w.events = w
                .events
                .iter()
                .filter(|(t, ..)| !removed.contains(t))
                .map(|(t, c, p)| (renumber[*t], c.clone(), p.clone()))
                .collect();
        }
        self.filter = self
            .filter
            .iter()
            .filter(|t| !removed.contains(t))
            .map(|t| renumber[*t])
            .collect();
        self.types = types;
        self.archetypes = archetypes;
        self.resources = resources;
        self.whole = whole;
        Ok(())
    }

    /// Adds an entity with the components given (each type by its schema,
    /// which must be the snapshot's for a type it already has) and a name:
    /// a whole world's next entity from its allocator, as a spawn would
    /// take it; a selection's after its highest id. Placed after the rows
    /// of the archetype of its component set (a new one at the end).
    pub fn spawn(
        &mut self,
        name: Option<&str>,
        components: Vec<(Schema, Vec<u8>)>,
    ) -> Result<Entity> {
        if let Some(n) = name {
            if n.is_empty() || n.len() > MAX_NAME {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    format!("an entity name is 1 to {MAX_NAME} bytes"),
                ));
            }
            if self.names.iter().any(|(_, m)| m == n) {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    format!("the name '{n}' is taken in the save"),
                ));
            }
        }
        let mut stored = Vec::new();
        let mut values = Vec::new();
        let mut types = self.types.clone();
        for (schema, bytes) in components {
            let t = match types.iter().position(|t| t.name == schema.name) {
                Some(t) if types[t].layout_hash() == schema.layout_hash() => t,
                Some(_) => {
                    return Err(Error::new(
                        ErrorKind::Schema,
                        format!(
                            "'{}' has another layout in the save than the one given",
                            schema.name
                        ),
                    ));
                }
                None => {
                    types.push(schema.clone());
                    types.len() - 1
                }
            };
            if stored.contains(&t) {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    format!("'{}' given twice", schema.name),
                ));
            }
            let size: usize = sizes(&types[t]).iter().sum();
            if bytes.len() != size {
                return Err(bad(format!(
                    "a '{}' of {} bytes, its layout {size}",
                    schema.name,
                    bytes.len()
                )));
            }
            stored.push(t);
            values.push(bytes);
        }
        let e = match &mut self.whole {
            Some(w) => {
                let mut alloc =
                    HandleAllocator::from_parts(&w.slots, &w.free).map_err(|e| bad(e.reason()))?;
                let e = Entity(alloc.alloc());
                w.slots = alloc.slots().collect();
                w.free = alloc.free_list().to_vec();
                e
            }
            None => {
                let next = self
                    .entities()
                    .iter()
                    .map(|e| e.index() + 1)
                    .max()
                    .unwrap_or(1);
                Entity(RawHandle::from_bits(u64::from(next) | 1 << 32))
            }
        };
        self.types = types;
        let mut key = stored.clone();
        key.sort_unstable();
        let at = match self.archetypes.iter().position(|a| {
            let mut k = a.types.clone();
            k.sort_unstable();
            k == key
        }) {
            Some(at) => at,
            None => {
                self.archetypes.push(Arch {
                    columns: stored
                        .iter()
                        .map(|&t| vec![Vec::new(); self.types[t].fields.len()])
                        .collect(),
                    types: stored.clone(),
                    entities: Vec::new(),
                    keep: false,
                });
                self.archetypes.len() - 1
            }
        };
        let arch = &mut self.archetypes[at];
        arch.entities.push(e);
        for (t, bytes) in stored.iter().zip(&values) {
            let position = arch.types.iter().position(|u| u == t).expect("own type");
            let mut offset = 0;
            for (k, n) in sizes(&self.types[*t]).into_iter().enumerate() {
                arch.columns[position][k].extend_from_slice(&bytes[offset..offset + n]);
                offset += n;
            }
        }
        if let Some(n) = name {
            let i = self.names.partition_point(|(f, _)| *f < e);
            self.names.insert(i, (e, n.to_string()));
        }
        Ok(e)
    }

    /// The snapshot (or selection) as bytes, in its format: a whole world
    /// as format 2, or 3 with state blocks, or 4 taken mid-tick.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut w = Writer(Vec::new());
        w.bytes(b"OGWS");
        let Some(whole) = &self.whole else {
            w.u32(FORMAT_SELECTION as usize);
            write_types(&mut w, &self.types);
            w.u32(self.filter.len());
            for &t in &self.filter {
                w.u32(t);
            }
            self.write_rows(&mut w);
            return w.0;
        };
        let format = match (whole.resume.is_some(), whole.blocks.is_empty()) {
            (true, _) => 4,
            (false, true) => 2,
            (false, false) => 3,
        };
        w.u32(format);
        write_types(&mut w, &self.types);
        w.u32(whole.slots.len());
        for s in &whole.slots {
            w.u32(s.generation as usize);
            w.u8(u8::from(s.alive));
        }
        w.u32(whole.free.len());
        for &i in &whole.free {
            w.u32(i as usize);
        }
        w.u32(self.archetypes.len());
        for a in &self.archetypes {
            Self::write_archetype(&mut w, a);
        }
        Self::write_resources(&mut w, &self.resources);
        w.u32(whole.events.len());
        for (t, current, pending) in &whole.events {
            w.u32(*t);
            for list in [current, pending] {
                w.u32(list.len());
                for e in list {
                    w.bytes(e);
                }
            }
        }
        self.write_names(&mut w);
        if format != 2 {
            w.u32(whole.blocks.len());
            for (name, bytes) in &whole.blocks {
                w.sized(name.as_bytes());
                w.sized(bytes);
            }
        }
        if let Some(r) = &whole.resume {
            w.sized(r.as_bytes());
        }
        w.0
    }

    fn write_archetype(w: &mut Writer, a: &Arch) {
        w.u32(a.types.len());
        for &t in &a.types {
            w.u32(t);
        }
        w.u32(a.entities.len());
        for e in &a.entities {
            w.u64(e.to_bits());
        }
        for fields in &a.columns {
            for c in fields {
                w.bytes(c);
            }
        }
    }

    fn write_resources(w: &mut Writer, resources: &[(usize, Vec<u8>)]) {
        w.u32(resources.len());
        for (t, b) in resources {
            w.u32(*t);
            w.sized(b);
        }
    }

    fn write_names(&self, w: &mut Writer) {
        w.u32(self.names.len());
        for (e, name) in &self.names {
            w.u64(e.to_bits());
            w.sized(name.as_bytes());
        }
    }

    /// A selection's archetypes (those with rows), resources and names.
    fn write_rows(&self, w: &mut Writer) {
        let rows: Vec<&Arch> = self
            .archetypes
            .iter()
            .filter(|a| !a.entities.is_empty())
            .collect();
        w.u32(rows.len());
        for a in rows {
            Self::write_archetype(w, a);
        }
        Self::write_resources(w, &self.resources);
        self.write_names(w);
    }
}

/// How a selection was restored ([`World::restore_selection`]).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SelectionReport {
    /// Live entities that carried the selection's types: despawned.
    pub despawned: usize,
    /// The saved entities, spawned anew, in saved order: (saved id, new id).
    pub spawned: Vec<(Entity, Entity)>,
    /// The resources set from the selection.
    pub resources: Vec<String>,
    /// What was mapped field by field (types of another layout, dropped
    /// types and fields).
    pub migration: MigrationReport,
}

impl World {
    /// Reads a snapshot (or a selection) into an editable [`SnapshotEdit`].
    /// Refused for a malformed one, or one of format 1 (no field table).
    pub fn edit_snapshot(&self, bytes: &[u8]) -> Result<SnapshotEdit> {
        SnapshotEdit::from_parsed(parse(&self.registry, bytes)?)
    }

    /// The entities that carry every type of `with` (and their names), and
    /// the resources `resources` names that the world has, as a selection
    /// (format 5, see the module docs): its type table holds only the types
    /// those use. [`Self::restore_selection`] takes it back.
    pub fn snapshot_selection(&self, with: &[SchemaId], resources: &[SchemaId]) -> Result<Vec<u8>> {
        self.selection(with, resources, false)
    }

    /// [`Self::snapshot_selection`] from inside a running system, whose
    /// columns its host holds while it runs.
    pub(crate) fn selection_in_system(
        &self,
        with: &[SchemaId],
        resources: &[SchemaId],
    ) -> Result<Vec<u8>> {
        self.selection(with, resources, true)
    }

    fn selection(
        &self,
        with: &[SchemaId],
        resources: &[SchemaId],
        unguarded: bool,
    ) -> Result<Vec<u8>> {
        for id in with.iter().chain(resources) {
            if id.0 as usize >= self.registry.len() {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("type {} is not registered", id.0),
                ));
            }
        }
        let mut table: Vec<SchemaId> = Vec::new();
        let index = |id: SchemaId, table: &mut Vec<SchemaId>| -> usize {
            match table.iter().position(|&u| u == id) {
                Some(i) => i,
                None => {
                    table.push(id);
                    table.len() - 1
                }
            }
        };
        let filter: Vec<usize> = with.iter().map(|&id| index(id, &mut table)).collect();
        let mut archetypes = Vec::new();
        let mut selected = BTreeSet::new();
        for arch in &self.archetypes {
            if arch.len() == 0 || !with.iter().all(|id| arch.types.contains(id)) {
                continue;
            }
            let types: Vec<usize> = arch.types.iter().map(|&id| index(id, &mut table)).collect();
            let mut columns: Vec<Vec<Vec<u8>>> = Vec::with_capacity(types.len());
            for position in 0..arch.types.len() {
                let mut fields = Vec::new();
                for c in arch.columns_of(position) {
                    let column = &arch.columns[c];
                    let bytes = match column.try_borrow() {
                        Ok(col) => col.bytes().to_vec(),
                        Err(_) if unguarded => {
                            // SAFETY: as in the snapshot's writer: only the
                            // running system holds a column mutably, and it
                            // is blocked in the call that asked for this.
                            unsafe { &*column.as_ptr() }.bytes().to_vec()
                        }
                        Err(_) => panic!("a column is borrowed outside a system"),
                    };
                    fields.push(bytes);
                }
                columns.push(fields);
            }
            selected.extend(arch.entities.iter().copied());
            archetypes.push(Arch {
                types,
                entities: arch.entities.clone(),
                columns,
                keep: false,
            });
        }
        let mut saved = Vec::new();
        for &id in resources {
            let Some(value) = self.resources[id.0 as usize].as_ref() else {
                continue;
            };
            let bytes = crate::world::resource_to_bytes(&self.registry, id, &value.borrow())
                .expect("resources were validated against their layout on insert");
            saved.push((index(id, &mut table), bytes));
        }
        let edit = SnapshotEdit {
            types: table
                .iter()
                .map(|&id| self.registry.info(id).schema.clone())
                .collect(),
            archetypes,
            resources: saved,
            names: self
                .names
                .iter()
                .filter(|(e, _)| selected.contains(*e))
                .map(|(e, n)| (*e, n.clone()))
                .collect(),
            whole: None,
            filter,
        };
        Ok(edit.to_bytes())
    }

    /// Restores a selection: despawns the live entities that carry every
    /// type of its filter, spawns its entities anew in saved order with
    /// their components and names, and sets its resources. In each
    /// spawned entity's components, the fields `entity_fields` lists for
    /// the type (field indices of this world's schema) holding a saved
    /// entity's id are pointed at that entity's new id; other ids are kept.
    ///
    /// Types are mapped by name. One of another layout is mapped field by
    /// field ([`Self::restore_migrating`]'s rules) when `migrate` is set or
    /// `lenient` names it, or when `grown` names it and it only gained
    /// fields (each added one at its default, as
    /// [`Self::check_snapshot_replacing`] takes it), else refused; one this
    /// world does not register is dropped when `migrate` is set or
    /// `lenient` names it. Refused, with nothing changed, for a
    /// malformed selection, a whole-world snapshot, or a name a live entity
    /// keeps.
    pub fn restore_selection(
        &mut self,
        bytes: &[u8],
        migrate: bool,
        lenient: &dyn Fn(&str) -> bool,
        grown: &dyn Fn(&str) -> bool,
        entity_fields: &dyn Fn(&str) -> Vec<usize>,
    ) -> Result<SelectionReport> {
        let parsed = parse(&self.registry, bytes)?;
        let Some(filter) = parsed.selection.clone() else {
            return Err(Error::new(
                ErrorKind::Invalid,
                "snapshot: a whole world, not a selection: restore it with World::restore",
            ));
        };
        let mut report = SelectionReport::default();
        let mut plans = Vec::with_capacity(parsed.types.len());
        for t in &parsed.types {
            let free = migrate || lenient(&t.name);
            let Ok(id) = self.registry.id_by_name(&t.name) else {
                if !free {
                    return Err(Error::new(
                        ErrorKind::NotFound,
                        format!(
                            "snapshot: type '{}' is not registered in this world",
                            t.name
                        ),
                    ));
                }
                report.migration.dropped.push(t.name.clone());
                plans.push(None);
                continue;
            };
            let info = self.registry.info(id);
            let only_grown = grown(&t.name)
                && t.schema
                    .as_ref()
                    .is_some_and(|saved| crate::snapshot::only_added(saved, &info.schema));
            if info.layout_hash != t.hash && !free && !only_grown {
                let detail = t
                    .schema
                    .as_ref()
                    .map(|s| crate::snapshot::layout_difference(s, &self.registry.info(id).schema))
                    .unwrap_or_default();
                return Err(Error::new(
                    ErrorKind::Schema,
                    format!(
                        "snapshot: type '{}' has a different layout in this world{detail}",
                        t.name
                    ),
                ));
            }
            plans.push(Some(plan(&self.registry, t, id, &mut report.migration)?));
        }
        let filter_ids: Vec<SchemaId> = filter
            .iter()
            .filter_map(|&t| plans[t].as_ref().map(|p| p.target))
            .collect();
        if filter_ids.len() != filter.len() {
            return Err(Error::new(
                ErrorKind::NotFound,
                "snapshot: the selection's filter names a type this world does not register",
            ));
        }
        // The live entities it replaces.
        let replaced: Vec<Entity> = self
            .archetypes
            .iter()
            .filter(|a| filter_ids.iter().all(|id| a.types.contains(id)))
            .flat_map(|a| a.entities.iter().copied())
            .collect();
        let replaced_set: BTreeSet<Entity> = replaced.iter().copied().collect();
        for (_, name) in &parsed.names {
            if let Some(&e) = self.by_name.get(name)
                && !replaced_set.contains(&e)
            {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    format!(
                        "the selection names an entity '{name}', and {e}, which it does not replace, has that name"
                    ),
                ));
            }
        }
        // Every saved entity's components, converted, in saved order.
        let mut rows: Vec<SavedRow> = Vec::new();
        for a in &parsed.archetypes {
            for row in 0..a.entities.len() {
                let mut components = Vec::new();
                for (s, &t) in a.stored.iter().enumerate() {
                    let Some(p) = &plans[t] else { continue };
                    let mut value = Vec::new();
                    for (k, &n) in parsed.types[t].sizes.iter().enumerate() {
                        value.extend_from_slice(&a.columns[s][k][row * n..(row + 1) * n]);
                    }
                    let mut out = Vec::new();
                    p.convert(&value, &mut out);
                    components.push((p.target, out));
                }
                og_core::sort::sort_by_key(&mut components, |(id, _)| *id);
                rows.push((a.entities[row], components));
            }
        }
        let mut resources = Vec::new();
        for &(t, data) in &parsed.resources {
            let Some(p) = &plans[t] else { continue };
            let mut out = Vec::new();
            p.convert(data, &mut out);
            resources.push((p.target, out));
        }
        // Validated: nothing below is refused.
        for e in &replaced {
            self.despawn(*e)?;
        }
        report.despawned = replaced.len();
        let mut map: BTreeMap<u64, u64> = BTreeMap::new();
        let mut spawned = Vec::with_capacity(rows.len());
        for (saved, components) in &rows {
            let e = self.spawn_bytes(components.clone())?;
            map.insert(saved.to_bits(), e.to_bits());
            spawned.push((*saved, e));
        }
        for ((_, components), (_, e)) in rows.iter().zip(&spawned) {
            for (id, bytes) in components {
                let info = self.registry.info(*id);
                let fields = entity_fields(&info.schema.name);
                if fields.is_empty() {
                    continue;
                }
                let mut value = bytes.clone();
                let mut changed = false;
                for f in fields {
                    let range = info.layout.range(f);
                    if range.len() != 8 {
                        continue;
                    }
                    let bits =
                        u64::from_le_bytes(value[range.clone()].try_into().expect("8 bytes"));
                    if let Some(&to) = map.get(&bits) {
                        value[range].copy_from_slice(&to.to_le_bytes());
                        changed = true;
                    }
                }
                if changed {
                    self.insert_bytes(*e, *id, &value)?;
                }
            }
        }
        for (saved, name) in &parsed.names {
            let e = Entity(RawHandle::from_bits(map[&saved.to_bits()]));
            self.set_name(e, name)?;
        }
        for (id, bytes) in resources {
            self.insert_resource_bytes(id, &bytes)?;
            report
                .resources
                .push(self.registry.info(id).schema.name.clone());
        }
        report.spawned = spawned;
        Ok(report)
    }
}
