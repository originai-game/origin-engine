//! The [`World`]: entities, archetype storage, resources, events and the
//! system schedule, advanced one fixed tick at a time by [`World::step`].

use crate::entity::Entity;
use crate::query::{QueryAccess, QueryDesc, QueryIter};
use crate::reactive::{HookEvent, Slot};
use crate::registry::{FieldRef, Registry, SchemaId};
use crate::schedule::{Command, System, SystemContext, SystemDesc, SystemInfo};
use crate::spatial::{SpatialDesc, SpatialHash};
use crate::state::{Block, StateBlock};
use crate::storage::Archetype;
use og_core::handle::HandleAllocator;
use og_core::reflect::{Reflect, Schema, schemas_to_json};
use og_core::{Error, ErrorKind, Result, Rng, Time, TimeScale};
use std::cell::{Ref, RefCell};
use std::collections::BTreeMap;
use std::sync::Arc;

/// The tick the last [`World::step`] in this process began, for a host's
/// crash report (a panic hook reads it); nothing in a tick reads it.
static LAST_TICK: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// The tick the last [`World::step`] in this process began (any world), 0
/// before the first: what a host's panic hook names as the panic's tick.
pub fn last_tick() -> u64 {
    LAST_TICK.load(std::sync::atomic::Ordering::Relaxed)
}

/// How to make a world. `Default` is 60 ticks per second, seed 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldDesc {
    /// Fixed ticks per second (`Time::dt` is its inverse). Must be at least 1.
    pub tick_rate: u32,
    /// Seed of the world's [`Rng`] resource.
    pub seed: u32,
}

impl Default for WorldDesc {
    fn default() -> Self {
        WorldDesc {
            tick_rate: 60,
            seed: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Location {
    pub archetype: u32,
    pub row: u32,
}

/// The longest entity name, in bytes.
pub const MAX_NAME: usize = 256;

/// Archetype index of an entity reserved by a running system and placed when
/// the system's commands are applied.
pub(crate) const PENDING: u32 = u32::MAX;

#[derive(Default)]
pub(crate) struct Entities {
    pub alloc: HandleAllocator,
    pub locations: Vec<Location>,
}

impl Entities {
    pub fn reserve(&mut self) -> Entity {
        let e = Entity(self.alloc.alloc());
        let index = e.index() as usize;
        if index >= self.locations.len() {
            self.locations.resize(
                index + 1,
                Location {
                    archetype: PENDING,
                    row: 0,
                },
            );
        }
        self.locations[index] = Location {
            archetype: PENDING,
            row: 0,
        };
        e
    }
}

pub(crate) enum ResourceValue {
    /// A Rust type's value (its type registered through `Reflect`).
    Typed(crate::registry::AnyValue),
    /// Packed bytes of a host-declared layout.
    Raw(Vec<u8>),
}

/// Events of one type: `current` was delivered at the start of this tick,
/// `pending` is what has been sent since and is delivered at the next.
#[derive(Clone, Default)]
pub(crate) struct EventQueue {
    pub current: Vec<Vec<u8>>,
    pub pending: Vec<Vec<u8>>,
}

/// The simulation state and its schedule. See the crate docs for the model.
pub struct World {
    pub(crate) registry: Registry,
    pub(crate) entities: RefCell<Entities>,
    pub(crate) archetypes: Vec<Archetype>,
    pub(crate) archetype_by_types: BTreeMap<Vec<SchemaId>, u32>,
    pub(crate) resources: Vec<Option<RefCell<ResourceValue>>>,
    pub(crate) events: Vec<RefCell<EventQueue>>,
    pub(crate) systems: Vec<System>,
    pub(crate) time_id: SchemaId,
    pub(crate) scale_id: SchemaId,
    /// Bumped by every operation that may move a column or an entity list.
    pub(crate) structural: u64,
    /// Entity names: one name per live entity, one entity per name.
    pub(crate) names: BTreeMap<Entity, String>,
    pub(crate) by_name: BTreeMap<String, Entity>,
    /// State blocks in registration order.
    pub(crate) blocks: Vec<Block>,
    /// The wall clock systems are timed with ([`World::set_clock`]).
    pub(crate) clock: Option<fn() -> f64>,
    /// Restored from a snapshot taken mid-tick: the system the next step
    /// starts at, completing that tick (world state: a snapshot carries it).
    pub(crate) resume: Option<usize>,
    /// Change detection, hooks and state machines (`reactive`).
    pub(crate) reactive: crate::reactive::Reactive,
}

fn two_mut<T>(v: &mut [T], a: usize, b: usize) -> (&mut T, &mut T) {
    assert_ne!(a, b);
    if a < b {
        let (left, right) = v.split_at_mut(b);
        (&mut left[a], &mut right[0])
    } else {
        let (left, right) = v.split_at_mut(a);
        (&mut right[0], &mut left[b])
    }
}

impl World {
    /// A world with `Time` and `Rng` registered (schema names `og.Time` and
    /// `og.Rng`) and present as resources.
    pub fn new(desc: &WorldDesc) -> Result<World> {
        if desc.tick_rate == 0 {
            return Err(Error::new(
                ErrorKind::Invalid,
                "tick_rate must be at least 1",
            ));
        }
        let mut world = World {
            registry: Registry::default(),
            entities: RefCell::new(Entities::default()),
            archetypes: Vec::new(),
            archetype_by_types: BTreeMap::new(),
            resources: Vec::new(),
            events: Vec::new(),
            systems: Vec::new(),
            time_id: SchemaId(0),
            scale_id: SchemaId(0),
            structural: 0,
            names: BTreeMap::new(),
            by_name: BTreeMap::new(),
            blocks: Vec::new(),
            clock: None,
            resume: None,
            reactive: Default::default(),
        };
        world.archetype_for(Vec::new());
        world.time_id = world.register::<Time>()?;
        world.register::<Rng>()?;
        world.scale_id = world.register::<TimeScale>()?;
        world.insert_resource(Time::from_rate(desc.tick_rate))?;
        world.insert_resource(Rng::new(desc.seed))?;
        world.insert_resource(TimeScale::default())?;
        Ok(world)
    }

    // ---- registry ---------------------------------------------------------

    fn grow_tables(&mut self) {
        let n = self.registry.len();
        self.resources.resize_with(n, || None);
        self.events.resize_with(n, Default::default);
    }

    /// Registers a Rust type as a component, resource or event type
    /// (idempotent). Registration order defines [`SchemaId`]s, and with them
    /// the snapshot layout, so register in a fixed order.
    pub fn register<T: Reflect>(&mut self) -> Result<SchemaId> {
        let (id, _) = self.registry.register::<T>()?;
        self.grow_tables();
        // A host value of the layout the Rust type adopted becomes typed.
        if let Some(slot) = self.resources[id.0 as usize].as_mut()
            && let ResourceValue::Raw(bytes) = slot.get_mut()
        {
            let value = self.registry.decode::<T>(id, bytes)?;
            *slot.get_mut() = ResourceValue::Typed(Box::new(value));
        }
        Ok(id)
    }

    /// Registers a layout declared by a host (a TypeScript `defineComponent`)
    /// by its schema. Idempotent for the same name and layout; refused for
    /// the same name with a different layout.
    pub fn register_schema(&mut self, schema: Schema) -> Result<SchemaId> {
        let (id, _) = self.registry.register_schema(schema)?;
        self.grow_tables();
        Ok(id)
    }

    /// The id of a registered Rust type.
    pub fn id<T: Reflect>(&self) -> Result<SchemaId> {
        self.registry.id_of::<T>()
    }

    pub fn id_by_name(&self, name: &str) -> Result<SchemaId> {
        self.registry.id_by_name(name)
    }

    pub fn schema(&self, id: SchemaId) -> Result<&Schema> {
        self.registry.check(id)?;
        Ok(&self.registry.info(id).schema)
    }

    /// Every registered schema, in id order.
    pub fn schemas(&self) -> Vec<Schema> {
        self.registry
            .types
            .iter()
            .map(|t| t.schema.clone())
            .collect()
    }

    /// The `og schema` JSON of every registered type.
    pub fn schemas_json(&self) -> String {
        schemas_to_json(&self.schemas())
    }

    /// A field of a registered Rust type, for column access.
    pub fn field<T: Reflect>(&self, name: &str) -> Result<FieldRef> {
        self.registry.field(self.id::<T>()?, name)
    }

    pub fn field_by_id(&self, id: SchemaId, name: &str) -> Result<FieldRef> {
        self.registry.field(id, name)
    }

    // ---- entities ---------------------------------------------------------

    pub(crate) fn archetype_for(&mut self, types: Vec<SchemaId>) -> u32 {
        if let Some(&id) = self.archetype_by_types.get(&types) {
            return id;
        }
        let id = u32::try_from(self.archetypes.len()).expect("fewer than 2^32 archetypes");
        self.archetypes
            .push(Archetype::new(types.clone(), &self.registry));
        self.archetype_by_types.insert(types, id);
        self.track_archetype(id as usize);
        id
    }

    pub(crate) fn location(&self, e: Entity) -> Result<Location> {
        let entities = self.entities.borrow();
        entities.alloc.check(e.0).map_err(|_| dead(e))?;
        let loc = entities.locations[e.index() as usize];
        if loc.archetype == PENDING {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("entity {e} is spawned by the running system and placed when it ends"),
            ));
        }
        Ok(loc)
    }

    fn set_location(&mut self, e: Entity, archetype: u32, row: usize) {
        self.entities.get_mut().locations[e.index() as usize] = Location {
            archetype,
            row: row as u32,
        };
    }

    /// Sorts and validates spawn components: known types, packed sizes, no
    /// type twice. Done before an id is reserved, so a refused spawn leaves
    /// no trace.
    pub(crate) fn prepare(
        &self,
        mut components: Vec<(SchemaId, Vec<u8>)>,
    ) -> Result<Vec<(SchemaId, Vec<u8>)>> {
        for (id, bytes) in &components {
            self.registry.check(*id)?;
            self.registry.check_value(*id, bytes)?;
        }
        og_core::sort::sort_by_key(&mut components, |(id, _)| *id);
        if let Some(w) = components.windows(2).find(|w| w[0].0 == w[1].0) {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("a bundle holds '{}' twice", self.registry.name(w[0].0)),
            ));
        }
        Ok(components)
    }

    /// Places a live entity with no location into the archetype of
    /// [`Self::prepare`]d components (its tracked components added now).
    pub(crate) fn place(&mut self, e: Entity, components: Vec<(SchemaId, Vec<u8>)>) {
        let types = components.iter().map(|(id, _)| *id).collect();
        let a = self.archetype_for(types);
        let stamp = self.stamp();
        let arch = &mut self.archetypes[a as usize];
        for (position, (id, bytes)) in components.iter().enumerate() {
            let layout = &self.registry.info(*id).layout;
            for (k, c) in arch.columns_of(position).enumerate() {
                arch.columns[c].get_mut().push(&bytes[layout.range(k)]);
            }
        }
        arch.entities.push(e);
        for position in 0..components.len() {
            arch.push_ticks(position, None, stamp);
        }
        let row = arch.len() - 1;
        self.set_location(e, a, row);
        self.structural += 1;
    }

    /// Spawns an entity with a bundle: `()` or a tuple of reflected values.
    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Result<Entity> {
        let components = encode_bundle(&self.registry, bundle)?;
        self.spawn_bytes(components)
    }

    /// Spawns an entity from packed component bytes (the host form of
    /// [`Self::spawn`]), then runs the add hooks of its components; a hook's
    /// refusal is returned, and the entity stays.
    pub fn spawn_bytes(&mut self, components: Vec<(SchemaId, Vec<u8>)>) -> Result<Entity> {
        let e = self.spawn_quiet(components)?;
        let types = self.components_of(e)?;
        self.fire(HookEvent::Add, e, &types)?;
        Ok(e)
    }

    /// [`Self::spawn_bytes`] without hooks (a restore's spawns).
    pub(crate) fn spawn_quiet(&mut self, components: Vec<(SchemaId, Vec<u8>)>) -> Result<Entity> {
        let components = self.prepare(components)?;
        let e = self.entities.get_mut().reserve();
        self.place(e, components);
        Ok(e)
    }

    /// Removes an entity and all its components, running their remove hooks
    /// first (a hook's refusal is returned, and the entity stays). Its id is
    /// dead from now on.
    pub fn despawn(&mut self, e: Entity) -> Result<()> {
        let types = self.components_of(e)?;
        self.fire(HookEvent::Remove, e, &types)?;
        if !self.is_alive(e) {
            return Ok(()); // a hook despawned it
        }
        self.despawn_quiet(e)
    }

    /// [`Self::despawn`] without hooks.
    pub(crate) fn despawn_quiet(&mut self, e: Entity) -> Result<()> {
        let loc = self.location(e)?;
        let types = self.archetypes[loc.archetype as usize].types.clone();
        self.log_removed(e, &types);
        if let Some(moved) = self.archetypes[loc.archetype as usize].swap_remove(loc.row as usize) {
            self.set_location(moved, loc.archetype, loc.row as usize);
        }
        let entities = self.entities.get_mut();
        entities.alloc.free(e.0)?;
        entities.locations[e.index() as usize] = Location {
            archetype: PENDING,
            row: 0,
        };
        if let Some(name) = self.names.remove(&e) {
            self.by_name.remove(&name);
        }
        self.structural += 1;
        Ok(())
    }

    pub fn is_alive(&self, e: Entity) -> bool {
        self.entities.borrow().alloc.is_alive(e.0)
    }

    pub fn entity_count(&self) -> usize {
        self.entities.borrow().alloc.len()
    }

    /// Every live entity, in slot order.
    pub fn entities(&self) -> Vec<Entity> {
        let entities = self.entities.borrow();
        (0..entities.alloc.slot_count() as u32)
            .filter_map(|i| entities.alloc.live_at(i).map(Entity))
            .collect()
    }

    pub fn archetype_count(&self) -> usize {
        self.archetypes.len()
    }

    /// The component types an entity has, in id order.
    pub fn components_of(&self, e: Entity) -> Result<Vec<SchemaId>> {
        let loc = self.location(e)?;
        Ok(self.archetypes[loc.archetype as usize].types.clone())
    }

    /// Adds component `T` to `e`, or overwrites it if present.
    pub fn insert<T: Reflect>(&mut self, e: Entity, value: T) -> Result<()> {
        let id = self.id::<T>()?;
        let bytes = self.registry.encode(id, &value)?;
        self.insert_bytes(e, id, &bytes)
    }

    /// Adds (or overwrites) component `id` from packed bytes. An add runs
    /// the component's add hooks; a hook's refusal is returned, and the
    /// component stays.
    pub fn insert_bytes(&mut self, e: Entity, id: SchemaId, bytes: &[u8]) -> Result<()> {
        if self.insert_quiet(e, id, bytes)? {
            self.fire(HookEvent::Add, e, &[id])?;
        }
        Ok(())
    }

    /// [`Self::insert_bytes`] without hooks: whether it added the component
    /// (rather than overwriting it).
    pub(crate) fn insert_quiet(&mut self, e: Entity, id: SchemaId, bytes: &[u8]) -> Result<bool> {
        self.registry.check(id)?;
        self.registry.check_value(id, bytes)?;
        let loc = self.location(e)?;
        let src = loc.archetype as usize;
        let row = loc.row as usize;
        let layout = &self.registry.info(id).layout;
        let stamp = self.stamp();
        if let Some(position) = self.archetypes[src].position(id) {
            let arch = &mut self.archetypes[src];
            for (k, c) in arch.columns_of(position).enumerate() {
                arch.columns[c]
                    .get_mut()
                    .set_row(row, &bytes[layout.range(k)]);
            }
            arch.diff_row(position, row, stamp);
            return Ok(false);
        }
        let mut types = self.archetypes[src].types.clone();
        let at = types.binary_search(&id).unwrap_err();
        types.insert(at, id);
        let dst = self.archetype_for(types) as usize;
        let layout = &self.registry.info(id).layout;
        let (s, d) = two_mut(&mut self.archetypes, src, dst);
        let mut carried = Vec::with_capacity(d.types.len());
        for (position, &ty) in d.types.iter().enumerate() {
            let dst_cols = d.columns_of(position);
            match s.position(ty) {
                Some(sp) => {
                    for (sc, dc) in s.columns_of(sp).zip(dst_cols) {
                        d.columns[dc]
                            .get_mut()
                            .push(s.columns[sc].get_mut().row(row));
                    }
                    carried.push(s.take_row_ticks(sp, row));
                }
                None => {
                    for (k, dc) in dst_cols.enumerate() {
                        d.columns[dc].get_mut().push(&bytes[layout.range(k)]);
                    }
                    carried.push(None);
                }
            }
        }
        d.entities.push(e);
        for (position, from) in carried.into_iter().enumerate() {
            d.push_ticks(position, from, stamp);
        }
        let new_row = d.len() - 1;
        let moved = s.swap_remove(row);
        if let Some(moved) = moved {
            self.set_location(moved, src as u32, row);
        }
        self.set_location(e, dst as u32, new_row);
        self.structural += 1;
        Ok(true)
    }

    /// Removes component `T` from `e`. Refused if `e` does not have it.
    pub fn remove<T: Reflect>(&mut self, e: Entity) -> Result<()> {
        let id = self.id::<T>()?;
        self.remove_id(e, id)
    }

    /// Removes component `id` from `e`, running its remove hooks first (a
    /// hook's refusal is returned, and the component stays).
    pub fn remove_id(&mut self, e: Entity, id: SchemaId) -> Result<()> {
        self.registry.check(id)?;
        let loc = self.location(e)?;
        if self.archetypes[loc.archetype as usize].has(id) {
            self.fire(HookEvent::Remove, e, &[id])?;
            if !self.is_alive(e) || !self.archetypes[self.location(e)?.archetype as usize].has(id) {
                return Ok(()); // a hook took it away
            }
        }
        self.remove_quiet(e, id)
    }

    /// [`Self::remove_id`] without hooks.
    pub(crate) fn remove_quiet(&mut self, e: Entity, id: SchemaId) -> Result<()> {
        self.registry.check(id)?;
        let loc = self.location(e)?;
        let src = loc.archetype as usize;
        let row = loc.row as usize;
        let mut types = self.archetypes[src].types.clone();
        let Ok(at) = types.binary_search(&id) else {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("entity {e} has no '{}'", self.registry.name(id)),
            ));
        };
        types.remove(at);
        self.log_removed(e, &[id]);
        let dst = self.archetype_for(types) as usize;
        let stamp = self.stamp();
        let (s, d) = two_mut(&mut self.archetypes, src, dst);
        let mut carried = Vec::with_capacity(d.types.len());
        for (position, &ty) in d.types.iter().enumerate() {
            let sp = s.position(ty).expect("subset of the source archetype");
            for (sc, dc) in s.columns_of(sp).zip(d.columns_of(position)) {
                d.columns[dc]
                    .get_mut()
                    .push(s.columns[sc].get_mut().row(row));
            }
            carried.push(s.take_row_ticks(sp, row));
        }
        d.entities.push(e);
        for (position, from) in carried.into_iter().enumerate() {
            d.push_ticks(position, from, stamp);
        }
        let new_row = d.len() - 1;
        if let Some(moved) = s.swap_remove(row) {
            self.set_location(moved, src as u32, row);
        }
        self.set_location(e, dst as u32, new_row);
        self.structural += 1;
        Ok(())
    }

    /// Whether `e` has component `T`.
    pub fn has<T: Reflect>(&self, e: Entity) -> Result<bool> {
        let id = self.id::<T>()?;
        let loc = self.location(e)?;
        Ok(self.archetypes[loc.archetype as usize].has(id))
    }

    /// A copy of component `T` of `e`.
    pub fn get<T: Reflect>(&self, e: Entity) -> Result<T> {
        let id = self.id::<T>()?;
        let bytes = self.get_bytes(e, id)?;
        self.registry.decode(id, &bytes)
    }

    /// Packed bytes of component `id` of `e`.
    pub fn get_bytes(&self, e: Entity, id: SchemaId) -> Result<Vec<u8>> {
        self.registry.check(id)?;
        let loc = self.location(e)?;
        let arch = &self.archetypes[loc.archetype as usize];
        let position = arch.position(id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                format!("entity {e} has no '{}'", self.registry.name(id)),
            )
        })?;
        let mut out = Vec::with_capacity(self.registry.info(id).layout.size());
        if !arch.read_packed(position, loc.row as usize, &mut out) {
            return Err(busy(self.registry.name(id)));
        }
        Ok(out)
    }

    /// Overwrites a component the entity already has, without a structural
    /// change (usable while columns of other components are borrowed).
    pub(crate) fn set_in_place(&self, e: Entity, id: SchemaId, bytes: &[u8]) -> Result<()> {
        self.registry.check(id)?;
        self.registry.check_value(id, bytes)?;
        let loc = self.location(e)?;
        let arch = &self.archetypes[loc.archetype as usize];
        let position = arch.position(id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                format!("entity {e} has no '{}'", self.registry.name(id)),
            )
        })?;
        let layout = &self.registry.info(id).layout;
        for (k, c) in arch.columns_of(position).enumerate() {
            let mut col = arch.columns[c]
                .try_borrow_mut()
                .map_err(|_| busy(self.registry.name(id)))?;
            col.set_row(loc.row as usize, &bytes[layout.range(k)]);
        }
        Ok(())
    }

    // ---- queries ----------------------------------------------------------

    /// Iterates the archetypes matching `desc`, outside the schedule, with
    /// exactly the read and write access `desc` lists.
    pub fn query(&self, desc: &QueryDesc) -> Result<QueryIter<'_>> {
        let access = QueryAccess::resolve(&self.registry, desc, "world query")?;
        for id in access
            .changed
            .iter()
            .chain(&access.added)
            .chain(&access.removed)
        {
            if !self.is_tracked(*id) {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    format!(
                        "world query: '{}' is not tracked, so it has no changes to filter by (a system filtering on it, or World::track, tracks it)",
                        self.registry.name(*id)
                    ),
                ));
            }
        }
        Ok(QueryIter::new(self, Arc::new(access), self.world_since()))
    }

    /// The entities whose `field` (a `u64` entity link, such as a parent)
    /// holds `target`, in entity order (index, then generation).
    pub fn referencing(&self, field: FieldRef, target: Entity) -> Result<Vec<Entity>> {
        referencing(self, field, target, true)
    }

    /// A spatial index of the entities `desc` selects, placed at their
    /// `desc.field` value as it is now ([`crate::spatial`]). Refused for a
    /// field that is not a `vec3` or `vec2`, or an unknown type.
    pub fn spatial(&self, desc: &SpatialDesc) -> Result<SpatialHash> {
        spatial_points(self, desc, true)
    }

    // ---- resources --------------------------------------------------------

    /// Inserts or replaces resource `T`. `T` must be registered.
    pub fn insert_resource<T: Reflect>(&mut self, value: T) -> Result<()> {
        let id = self.id::<T>()?;
        let bytes = self.registry.encode(id, &value)?; // a layout mismatch is refused here, once
        self.registry.check_value(id, &bytes)?;
        self.resources[id.0 as usize] = Some(RefCell::new(ResourceValue::Typed(Box::new(value))));
        Ok(())
    }

    /// Inserts or replaces resource `id` from packed bytes.
    pub fn insert_resource_bytes(&mut self, id: SchemaId, bytes: &[u8]) -> Result<()> {
        self.registry.check(id)?;
        self.registry.check_value(id, bytes)?;
        let info = self.registry.info(id);
        let value = match &info.rust {
            Some(rust) => ResourceValue::Typed((rust.decode)(&info.layout, bytes)?),
            None => ResourceValue::Raw(bytes.to_vec()),
        };
        self.resources[id.0 as usize] = Some(RefCell::new(value));
        Ok(())
    }

    pub fn remove_resource<T: Reflect>(&mut self) -> Result<()> {
        let id = self.id::<T>()?;
        if id == self.time_id {
            return Err(Error::new(
                ErrorKind::Invalid,
                "Time belongs to the world and cannot be removed",
            ));
        }
        match self.resources[id.0 as usize].take() {
            Some(_) => Ok(()),
            None => Err(no_resource(T::NAME)),
        }
    }

    /// Removes resource `id` (the byte form of [`Self::remove_resource`]).
    /// Refused for `Time` and for a resource that is not present.
    pub fn remove_resource_id(&mut self, id: SchemaId) -> Result<()> {
        self.registry.check(id)?;
        if id == self.time_id {
            return Err(Error::new(
                ErrorKind::Invalid,
                "Time belongs to the world and cannot be removed",
            ));
        }
        match self.resources[id.0 as usize].take() {
            Some(_) => Ok(()),
            None => Err(no_resource(self.registry.name(id))),
        }
    }

    /// Whether resource `id` is present.
    pub fn has_resource_id(&self, id: SchemaId) -> bool {
        self.resources
            .get(id.0 as usize)
            .is_some_and(|r| r.is_some())
    }

    pub fn has_resource<T: Reflect>(&self) -> bool {
        self.id::<T>()
            .is_ok_and(|id| self.resources[id.0 as usize].is_some())
    }

    pub(crate) fn resource_cell(&self, id: SchemaId) -> Result<&RefCell<ResourceValue>> {
        self.resources[id.0 as usize]
            .as_ref()
            .ok_or_else(|| no_resource(self.registry.name(id)))
    }

    /// Shared access to resource `T`.
    pub fn resource<T: Reflect>(&self) -> Result<Ref<'_, T>> {
        let id = self.id::<T>()?;
        typed_ref(self.resource_cell(id)?, T::NAME)
    }

    /// Exclusive access to resource `T`.
    pub fn resource_mut<T: Reflect>(&mut self) -> Result<&mut T> {
        let id = self.id::<T>()?;
        match self.resources[id.0 as usize].as_mut().map(RefCell::get_mut) {
            Some(ResourceValue::Typed(value)) => value
                .downcast_mut::<T>()
                .ok_or_else(|| no_resource(T::NAME)),
            _ => Err(no_resource(T::NAME)),
        }
    }

    /// Packed bytes of resource `id`.
    pub fn resource_bytes(&self, id: SchemaId) -> Result<Vec<u8>> {
        self.registry.check(id)?;
        let cell = self.resource_cell(id)?;
        let value = cell
            .try_borrow()
            .map_err(|_| busy(self.registry.name(id)))?;
        resource_to_bytes(&self.registry, id, &value)
    }

    /// The current tick's clock.
    pub fn time(&self) -> Time {
        *self
            .resource::<Time>()
            .expect("Time is a resource of every world")
    }

    /// Slow motion and hit stop (the default when a restored snapshot had none).
    pub fn time_scale(&self) -> TimeScale {
        self.resource::<TimeScale>().map(|s| *s).unwrap_or_default()
    }

    /// The time groups' scales (og.TimeGroups) once a game uses them.
    pub fn time_groups(&self) -> Option<og_core::TimeGroups> {
        self.resource::<og_core::TimeGroups>().ok().map(|g| *g)
    }

    /// The scale of `e`'s time group under `groups`: 1 for an entity in
    /// none (no og.TimeGroup, or the type not registered).
    pub fn group_scale(&self, groups: &og_core::TimeGroups, e: Entity) -> f32 {
        self.get::<og_core::TimeGroup>(e)
            .map_or(1.0, |t| groups.scale(t.group))
    }

    /// Sets a time group's scale, refused before og.TimeGroups is
    /// registered, for a group past 15 or a scale outside 0..10.
    pub fn set_group_scale(&mut self, group: u32, scale: f32) -> Result<()> {
        self.resource_mut::<og_core::TimeGroups>()?
            .set(group, scale)
    }

    /// The scaled length of the current tick ([`TimeScale::dt`]).
    pub fn scaled_dt(&self) -> f32 {
        self.time_scale().dt(&self.time())
    }

    /// Sets the global time scale (0 to [`og_core::time::MAX_TIME_SCALE`]).
    pub fn set_time_scale(&mut self, scale: f32) -> Result<()> {
        let mut s = self.time_scale();
        s.scale = check_scale(scale)?;
        self.insert_resource(s)
    }

    /// Starts (or extends) a hit stop of `ticks` ticks: it never shortens one in progress.
    pub fn hit_stop(&mut self, ticks: u32) -> Result<()> {
        let mut s = self.time_scale();
        s.hit_stop = s.hit_stop.max(ticks);
        self.insert_resource(s)
    }

    // ---- events -----------------------------------------------------------

    /// Sends event `T`; systems read it during the next [`Self::step`].
    pub fn send<T: Reflect>(&mut self, event: T) -> Result<()> {
        let id = self.id::<T>()?;
        let bytes = self.registry.encode(id, &event)?;
        self.registry.check_value(id, &bytes)?;
        self.events[id.0 as usize].get_mut().pending.push(bytes);
        Ok(())
    }

    /// Sends event `id` from packed bytes.
    pub fn send_bytes(&mut self, id: SchemaId, bytes: &[u8]) -> Result<()> {
        self.registry.check(id)?;
        self.registry.check_value(id, bytes)?;
        self.events[id.0 as usize]
            .get_mut()
            .pending
            .push(bytes.to_vec());
        Ok(())
    }

    /// Events of type `T` sent since the start of the last step (by systems
    /// during it, or by the host since); they are delivered at the next step.
    pub fn pending_events<T: Reflect>(&self) -> Result<Vec<T>> {
        let id = self.id::<T>()?;
        let queue = self.events[id.0 as usize].borrow();
        queue
            .pending
            .iter()
            .map(|b| self.registry.decode(id, b))
            .collect()
    }

    /// Packed bytes of the events of `id` sent since the start of the last
    /// step (the byte form of [`Self::pending_events`]).
    pub fn pending_events_bytes(&self, id: SchemaId) -> Result<Vec<Vec<u8>>> {
        self.registry.check(id)?;
        Ok(self.events[id.0 as usize].borrow().pending.clone())
    }

    // ---- names --------------------------------------------------------------

    /// Names a live entity: the probe, the command channel, scenes and
    /// `world.find` address it by this name. One name per entity (naming it
    /// again renames it) and one entity per name: a name another live entity
    /// holds is refused. A name is non-empty, at most 256 bytes, does not
    /// start with `#` and holds no whitespace, `:`, `;` or `=` (the command
    /// grammar could not address it). Names are in snapshots and the world
    /// hash, and leave with the entity when it is despawned.
    pub fn set_name(&mut self, e: Entity, name: &str) -> Result<()> {
        if !self.is_alive(e) {
            return Err(dead(e));
        }
        let addressable = !name.is_empty()
            && name.len() <= MAX_NAME
            && !name.starts_with('#')
            && !name
                .chars()
                .any(|c| c == ':' || c == ';' || c == '=' || c.is_whitespace());
        if !addressable {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "'{name}' is not an entity name: non-empty, at most {MAX_NAME} bytes, no whitespace, ':', ';' or '=', and not starting with '#'"
                ),
            ));
        }
        match self.by_name.get(name) {
            Some(&holder) if holder == e => return Ok(()),
            Some(&holder) => {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    format!("the name '{name}' already belongs to entity {holder}"),
                ));
            }
            None => {}
        }
        if let Some(old) = self.names.insert(e, name.to_owned()) {
            self.by_name.remove(&old);
        }
        self.by_name.insert(name.to_owned(), e);
        Ok(())
    }

    /// Removes an entity's name, if it has one.
    pub fn clear_name(&mut self, e: Entity) -> Result<()> {
        if !self.is_alive(e) {
            return Err(dead(e));
        }
        if let Some(old) = self.names.remove(&e) {
            self.by_name.remove(&old);
        }
        Ok(())
    }

    /// The name of an entity, if it has one.
    pub fn name(&self, e: Entity) -> Option<&str> {
        self.names.get(&e).map(String::as_str)
    }

    /// The live entity holding a name.
    pub fn find(&self, name: &str) -> Option<Entity> {
        self.by_name.get(name).copied()
    }

    /// Every named entity with its name, in entity order (index, then
    /// generation).
    pub fn names(&self) -> Vec<(Entity, &str)> {
        self.names.iter().map(|(e, n)| (*e, n.as_str())).collect()
    }

    // ---- state blocks -------------------------------------------------------

    /// Adds a state block ([`StateBlock`]) under `name`: snapshots save it,
    /// the world hash covers it and restores bring it back. A world with a
    /// block writes snapshot format 3. Refused for an empty name, one longer
    /// than 256 bytes, or one already taken.
    pub fn add_state_block(&mut self, name: &str, block: Box<dyn StateBlock>) -> Result<()> {
        if name.is_empty() || name.len() > MAX_NAME {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("a state block name is 1 to {MAX_NAME} bytes, got '{name}'"),
            ));
        }
        if self.blocks.iter().any(|b| b.name == name) || name == crate::reactive::BLOCK {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("the state block '{name}' is already added"),
            ));
        }
        self.blocks.push(Block {
            name: name.to_owned(),
            block,
        });
        Ok(())
    }

    // ---- structure ----------------------------------------------------------

    /// A counter bumped by every operation that may move a column or an
    /// entity list: a spawn, a despawn, an insert that adds a component, a
    /// remove, a restore (each applied command counts as its operation).
    /// Overwriting a component, writing through a view, resources, events,
    /// names and `Time` do not change it. Views taken while it holds a value
    /// are valid until it changes. It counts operations on this `World`
    /// value; it is not world state (not in snapshots or the hash).
    pub fn structural_changes(&self) -> u64 {
        self.structural
    }

    // ---- schedule ---------------------------------------------------------

    /// Adds a system at the end of the schedule. Refused, with the type and
    /// field named, when a query writes or reads a component the system did
    /// not declare, a declared name is unknown, or the name is taken.
    pub fn add_system<F>(&mut self, desc: SystemDesc, run: F) -> Result<()>
    where
        F: Fn(&mut SystemContext<'_>) -> Result<()> + Send + 'static,
    {
        self.push_system(desc, Box::new(run), None)
    }

    /// [`Self::add_system`] with a run condition: before each tick's run
    /// (after the systems before it, so it sees what they wrote) `condition`
    /// is called with the system's declarations as read-only access, and the
    /// system runs only when it answers true. A skipped run is not a run:
    /// change filters keep reading from the last one.
    pub fn add_system_if<C, F>(&mut self, desc: SystemDesc, condition: C, run: F) -> Result<()>
    where
        C: Fn(&mut SystemContext<'_>) -> Result<bool> + Send + 'static,
        F: Fn(&mut SystemContext<'_>) -> Result<()> + Send + 'static,
    {
        self.push_system(desc, Box::new(run), Some(Box::new(condition)))
    }

    /// Widens scheduled system `system`'s declarations by `writes` (type
    /// names; a write implies a read): a module whose system touches a type
    /// registered only once a game uses it (og.nav's `og.Crowd`) grants it
    /// then, so a world that never uses the type keeps its type table.
    /// Refused, naming the system or type, for an unknown one, and for a
    /// write to Time or og.TimeScale.
    pub fn grant(&mut self, system: &str, writes: &[&str]) -> Result<()> {
        let mut ids = Vec::with_capacity(writes.len());
        for &n in writes {
            let id = self.registry.id_by_name(n).map_err(|_| {
                Error::new(
                    ErrorKind::NotFound,
                    format!("system '{system}' is granted unknown type '{n}'"),
                )
            })?;
            if id == self.time_id || id == self.scale_id {
                return Err(Error::new(
                    ErrorKind::Access,
                    format!("system '{system}' cannot be granted a write to '{n}'"),
                ));
            }
            ids.push(id);
        }
        let s = self
            .systems
            .iter_mut()
            .find(|s| s.name == system)
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::NotFound,
                    format!("no system named '{system}' is scheduled"),
                )
            })?;
        for list in [&mut s.writes, &mut s.reads] {
            list.extend(&ids);
            og_core::sort::sort(list);
            list.dedup();
        }
        if let Some(c) = s.condition.as_mut() {
            c.0.reads.extend(&ids);
            og_core::sort::sort(&mut c.0.reads);
            c.0.reads.dedup();
        }
        Ok(())
    }

    fn push_system(
        &mut self,
        desc: SystemDesc,
        run: crate::schedule::SystemFn,
        condition: Option<crate::reactive::ConditionFn>,
    ) -> Result<()> {
        let filters: Vec<String> = desc
            .queries
            .iter()
            .flat_map(|q| q.changed.iter().chain(&q.added).chain(&q.removed).cloned())
            .collect();
        let read_only = SystemDesc {
            name: desc.name.clone(),
            reads: desc.reads.iter().chain(&desc.writes).cloned().collect(),
            ..SystemDesc::default()
        };
        let mut system = System::resolve(self, desc, run)?;
        if let Some(condition) = condition {
            let access = System::resolve(self, read_only, Box::new(|_| Ok(())))?;
            system.condition = Some(Box::new((access, condition)));
        }
        for name in filters {
            let id = self.registry.id_by_name(&name)?;
            self.track(id)?;
        }
        self.systems.push(system);
        Ok(())
    }

    /// System names in run order.
    pub fn systems(&self) -> Vec<&str> {
        self.systems.iter().map(|s| s.name.as_str()).collect()
    }

    /// Every system in run order: its name, its declared reads and writes
    /// (type names in id order; `reads` excludes what `writes` lists), its
    /// queries, and how many entities its queries matched when it last ran.
    pub fn system_info(&self) -> Vec<SystemInfo> {
        self.systems
            .iter()
            .map(|s| {
                let name = |id: &SchemaId| self.registry.name(*id).to_owned();
                SystemInfo {
                    name: s.name.clone(),
                    reads: s
                        .reads
                        .iter()
                        .filter(|id| s.writes.binary_search(id).is_err())
                        .map(name)
                        .collect(),
                    writes: s.writes.iter().map(name).collect(),
                    queries: s.desc_queries.clone(),
                    matched: s.matched.iter().sum(),
                    query_matched: s.matched.clone(),
                    last_tick: s.last_tick,
                    start: s.start,
                    ms: s.ms,
                    total_ms: s.total_ms,
                    runs: s.runs,
                }
            })
            .collect()
    }

    /// Every timed system's name and the milliseconds of its last run, in
    /// run order: [`World::system_info`]'s `name` and `ms` without the
    /// declarations it copies, for a reader of every frame (the probe's
    /// `state().timings.systems`).
    pub fn system_ms(&self) -> Vec<(String, f64)> {
        self.systems
            .iter()
            .filter_map(|s| s.ms.map(|ms| (s.name.clone(), ms)))
            .collect()
    }

    /// Times every system's run (with its commands applied) with `clock`,
    /// wall-clock milliseconds from any origin, reported by
    /// [`World::system_info`]; `None` stops timing. Measurement only: no
    /// system reads the clock, so the simulation and its hash never depend
    /// on it.
    pub fn set_clock(&mut self, clock: Option<fn() -> f64>) {
        self.clock = clock;
    }

    /// Entities each query of `system` matches now, in query order.
    fn match_counts(&self, system: &System) -> Vec<usize> {
        system
            .queries
            .iter()
            .map(|q| {
                self.archetypes
                    .iter()
                    .filter(|a| q.matches(a))
                    .map(Archetype::len)
                    .sum()
            })
            .collect()
    }

    /// Runs one fixed tick: delivers pending events, runs every system in
    /// order (applying each system's commands when it returns), then advances
    /// `Time`. A system error stops the tick and is returned with the tick and
    /// the system named; the world is left as it was when the system failed
    /// (earlier systems' effects stay; restore a snapshot to undo them).
    ///
    /// The first step after restoring a snapshot a system took (format 4)
    /// completes that snapshot's tick instead: it runs the systems after
    /// the one that took it, with the tick's events as they were, then
    /// advances `Time`, as the tick that saved went on.
    pub fn step(&mut self) -> Result<()> {
        let resumed = self.resume.take();
        if resumed.is_none() {
            for queue in &mut self.events {
                let queue = queue.get_mut();
                queue.current = std::mem::take(&mut queue.pending);
            }
            self.begin_tick_changes();
            self.begin_transitions();
        }
        let tick = self.time().tick;
        LAST_TICK.store(tick, std::sync::atomic::Ordering::Relaxed);
        let slots = self.slots();
        let mut first = 0;
        if let Some(k) = resumed {
            // The saver returned: its writes are compared and its run
            // recorded as its return would have.
            first = slots.len();
            if let Some(saver) = k.checked_sub(1).filter(|&i| i < self.systems.len()) {
                let writes = std::mem::take(&mut self.systems[saver].writes);
                self.diff(Some(&writes), self.reactive.change_tick);
                self.systems[saver].writes = writes;
                self.systems[saver].last_run = self.reactive.change_tick;
                if let Some(p) = slots.iter().position(|s| *s == Slot::Run(saver)) {
                    first = p + 1;
                }
            }
        }
        self.reactive.in_step = true;
        let ran = self.run_slots(&slots[first..], tick);
        self.reactive.in_step = false;
        ran?;
        self.reactive.transitions.clear();
        let time = self.time();
        if let Ok(scale) = self.resource_mut::<TimeScale>() {
            scale.end_tick(&time);
        }
        self.resource_mut::<Time>()?.advance();
        Ok(())
    }

    /// Runs a tick's steps in order (see [`crate::reactive`]).
    fn run_slots(&mut self, slots: &[Slot], tick: u64) -> Result<()> {
        let mut commands = Vec::new();
        for slot in slots {
            let i = match *slot {
                Slot::Set(k) => {
                    let t = self.reactive.transitions[k];
                    self.reactive.states[t.state].current = t.to;
                    continue;
                }
                Slot::Run(i) => i,
            };
            if !self.in_its_state(i) {
                continue;
            }
            let holds = self.condition_holds(i).map_err(|err| {
                err.context(format!(
                    "tick {tick}, run condition of system '{}'",
                    self.systems[i].name
                ))
            })?;
            if !holds {
                continue;
            }
            // Archetype sizes change only with a structural change: the
            // counts of a tick without one are the last tick's.
            if self.systems[i].matched_at != Some(self.structural) {
                let matched = self.match_counts(&self.systems[i]);
                self.systems[i].matched = matched;
                self.systems[i].matched_at = Some(self.structural);
            }
            self.systems[i].last_tick = Some(tick);
            self.reactive.change_tick += 1;
            let this_run = self.reactive.change_tick;
            let t0 = self.clock.map(|now| now());
            let result = {
                let system = &self.systems[i];
                let mut ctx = SystemContext::new(self, system, &mut commands);
                (system.run)(&mut ctx)
            };
            let writes = std::mem::take(&mut self.systems[i].writes);
            self.diff(Some(&writes), this_run);
            self.systems[i].writes = writes;
            self.systems[i].last_run = this_run;
            let failed = match result {
                Err(err) => Some(err),
                Ok(()) => self.apply(&mut commands).err(),
            };
            if let (Some(now), Some(t0)) = (self.clock, t0) {
                let ms = (now() - t0).max(0.0);
                let s = &mut self.systems[i];
                s.start = Some(t0);
                s.ms = Some(ms);
                s.total_ms += ms;
                s.runs += 1;
            }
            if let Some(err) = failed {
                let err = err.context(format!("tick {tick}, system '{}'", self.systems[i].name));
                self.discard(&mut commands);
                return Err(err);
            }
        }
        Ok(())
    }

    /// Applies a system's (or a hook's) commands in order, running the
    /// hooks they set off; on error the rest stay in `commands`.
    pub(crate) fn apply(&mut self, commands: &mut Vec<Command>) -> Result<()> {
        let mut drained = std::mem::take(commands).into_iter();
        while let Some(command) = drained.next() {
            let result = match command {
                Command::Spawn(e, components) => {
                    let types: Vec<SchemaId> = components.iter().map(|(id, _)| *id).collect();
                    self.place(e, components);
                    self.fire(HookEvent::Add, e, &types)
                }
                Command::Insert(e, id, bytes) => self.insert_bytes(e, id, &bytes),
                Command::Remove(e, id) => self.remove_id(e, id),
                Command::Name(e, name) => {
                    // As a name given between ticks: refused (taken, or
                    // the entity gone), it is not given.
                    let _ = self.set_name(e, &name);
                    Ok(())
                }
                Command::State(state, value) => {
                    let s = &mut self.reactive.states[state];
                    s.pending = (value != s.current).then_some(value);
                    Ok(())
                }
                Command::Despawn(e) => {
                    if self.is_alive(e) {
                        self.despawn(e)
                    } else {
                        Ok(()) // despawning twice in one tick is not an error
                    }
                }
            };
            if let Err(err) = result {
                commands.extend(drained);
                return Err(err);
            }
        }
        Ok(())
    }

    /// Drops unapplied commands, freeing entities they reserved.
    pub(crate) fn discard(&mut self, commands: &mut Vec<Command>) {
        for command in commands.drain(..) {
            if let Command::Spawn(e, _) = command {
                let entities = self.entities.get_mut();
                if entities.locations[e.index() as usize].archetype == PENDING {
                    let _ = entities.alloc.free(e.0);
                }
            }
        }
    }
}

pub(crate) fn dead(e: Entity) -> Error {
    Error::new(ErrorKind::DeadHandle, format!("entity {e} is not alive"))
}

pub(crate) fn busy(name: &str) -> Error {
    Error::new(
        ErrorKind::Access,
        format!(
            "'{name}' is borrowed by the running code (a column or resource view is still held)"
        ),
    )
}

pub(crate) fn no_resource(name: &str) -> Error {
    Error::new(
        ErrorKind::NotFound,
        format!("no '{name}' resource in the world"),
    )
}

pub(crate) fn typed_ref<'a, T: Reflect>(
    cell: &'a RefCell<ResourceValue>,
    name: &str,
) -> Result<Ref<'a, T>> {
    let value = cell.try_borrow().map_err(|_| busy(name))?;
    Ref::filter_map(value, |v| match v {
        ResourceValue::Typed(b) => b.downcast_ref::<T>(),
        ResourceValue::Raw(_) => None,
    })
    .map_err(|_| no_resource(name))
}

pub(crate) fn resource_to_bytes(
    registry: &Registry,
    id: SchemaId,
    value: &ResourceValue,
) -> Result<Vec<u8>> {
    let info = registry.info(id);
    match value {
        ResourceValue::Raw(bytes) => Ok(bytes.clone()),
        ResourceValue::Typed(any) => {
            let rust = info
                .rust
                .as_ref()
                .expect("typed resources have a Rust type");
            (rust.encode)(&info.layout, any.as_ref())
        }
    }
}

/// Values a spawn takes: `()` or a tuple of up to eight reflected values.
pub trait Bundle {
    #[doc(hidden)]
    fn encode(self, encoder: &mut BundleEncoder<'_>) -> Result<()>;
}

/// Collects a bundle's packed components (an implementation detail of [`Bundle`]).
pub struct BundleEncoder<'r> {
    registry: &'r Registry,
    out: Vec<(SchemaId, Vec<u8>)>,
}

impl BundleEncoder<'_> {
    pub fn push<T: Reflect>(&mut self, value: &T) -> Result<()> {
        let id = self.registry.id_of::<T>()?;
        self.out.push((id, self.registry.encode(id, value)?));
        Ok(())
    }
}

pub(crate) fn encode_bundle<B: Bundle>(
    registry: &Registry,
    bundle: B,
) -> Result<Vec<(SchemaId, Vec<u8>)>> {
    let mut encoder = BundleEncoder {
        registry,
        out: Vec::new(),
    };
    bundle.encode(&mut encoder)?;
    Ok(encoder.out)
}

impl Bundle for () {
    fn encode(self, _: &mut BundleEncoder<'_>) -> Result<()> {
        Ok(())
    }
}

macro_rules! bundle_tuple {
    ($($name:ident),+) => {
        impl<$($name: Reflect),+> Bundle for ($($name,)+) {
            #[allow(non_snake_case)]
            fn encode(self, encoder: &mut BundleEncoder<'_>) -> Result<()> {
                let ($($name,)+) = self;
                $(encoder.push(&$name)?;)+
                Ok(())
            }
        }
    };
}

bundle_tuple!(A);
bundle_tuple!(A, B);
bundle_tuple!(A, B, C);
bundle_tuple!(A, B, C, D);
bundle_tuple!(A, B, C, D, E);
bundle_tuple!(A, B, C, D, E, F);
bundle_tuple!(A, B, C, D, E, F, G);
bundle_tuple!(A, B, C, D, E, F, G, H);

/// A valid [`TimeScale::scale`].
pub(crate) fn check_scale(scale: f32) -> Result<f32> {
    if scale.is_finite() && (0.0..=og_core::time::MAX_TIME_SCALE).contains(&scale) {
        Ok(scale)
    } else {
        Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "time scale {scale} is outside 0..={}",
                og_core::time::MAX_TIME_SCALE
            ),
        ))
    }
}

/// The points of `desc` in `world` (`guarded`: refuse a borrowed column,
/// else read it in place, as a running system's snapshot does).
pub(crate) fn spatial_points(
    world: &World,
    desc: &SpatialDesc,
    guarded: bool,
) -> Result<SpatialHash> {
    let field = desc.field;
    world.registry.check_field(field)?;
    for id in desc.with.iter().chain(&desc.without) {
        world.registry.check(*id)?;
    }
    if !matches!(
        field.ty,
        og_core::reflect::FieldType::Vec3 | og_core::reflect::FieldType::Vec2
    ) {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "a spatial index places entities by a vec3 or vec2 field; {} is {}",
                world.registry.field_name(field),
                field.ty.name()
            ),
        ));
    }
    let mut points = Vec::new();
    for arch in &world.archetypes {
        if !arch.has(field.component)
            || !desc.with.iter().all(|id| arch.has(*id))
            || desc.without.iter().any(|id| arch.has(*id))
            || arch.entities.is_empty()
        {
            continue;
        }
        let c = arch
            .column_index(field.component, field.field)
            .expect("the archetype has the component");
        let mut read = |bytes: &[u8]| {
            let size = bytes.len() / arch.len();
            for (row, e) in arch.entities.iter().enumerate() {
                if let Some(p) =
                    crate::spatial::position(field.ty, &bytes[row * size..(row + 1) * size])
                {
                    points.push((*e, p));
                }
            }
        };
        match arch.columns[c].try_borrow() {
            Ok(col) => read(col.bytes()),
            Err(_) if !guarded => {
                // SAFETY: as in `World::write_columns`: only the running
                // system holds a column mutably, blocked in this call.
                read(unsafe { &*arch.columns[c].as_ptr() }.bytes())
            }
            Err(_) => return Err(busy(&world.registry.field_name(field))),
        }
    }
    SpatialHash::new(points, desc.cell)
}

/// [`World::referencing`] (`guarded` as in [`spatial_points`]).
pub(crate) fn referencing(
    world: &World,
    field: FieldRef,
    target: Entity,
    guarded: bool,
) -> Result<Vec<Entity>> {
    world.registry.check_field(field)?;
    if field.ty != og_core::reflect::FieldType::U64 {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!("{} is not an entity link", world.registry.field_name(field)),
        ));
    }
    let want = target.to_bits().to_le_bytes();
    let mut out = Vec::new();
    for arch in &world.archetypes {
        let Some(c) = arch.column_index(field.component, field.field) else {
            continue;
        };
        let mut scan = |bytes: &[u8]| {
            for (row, e) in arch.entities.iter().enumerate() {
                if bytes[row * 8..row * 8 + 8] == want {
                    out.push(*e);
                }
            }
        };
        match arch.columns[c].try_borrow() {
            Ok(col) => scan(col.bytes()),
            // SAFETY: as in `World::write_columns`.
            Err(_) if !guarded => scan(unsafe { &*arch.columns[c].as_ptr() }.bytes()),
            Err(_) => return Err(busy(&world.registry.field_name(field))),
        }
    }
    og_core::sort::sort(&mut out);
    Ok(out)
}
