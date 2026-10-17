//! The [`World`]: entities, archetype storage, resources, events and the
//! system schedule, advanced one fixed tick at a time by [`World::step`].

use crate::entity::Entity;
use crate::query::{QueryAccess, QueryDesc, QueryIter};
use crate::registry::{FieldRef, Registry, SchemaId};
use crate::schedule::{Command, System, SystemContext, SystemDesc};
use crate::storage::Archetype;
use og_core::handle::HandleAllocator;
use og_core::reflect::{schemas_to_json, Reflect, Schema};
use og_core::{Error, ErrorKind, Result, Rng, Time};
use std::cell::{Ref, RefCell};
use std::collections::BTreeMap;
use std::sync::Arc;

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
    /// A world with `Time` and `Rng` registered and present as resources.
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
        };
        world.archetype_for(Vec::new());
        world.time_id = world.register::<Time>()?;
        world.register::<Rng>()?;
        world.insert_resource(Time::from_rate(desc.tick_rate))?;
        world.insert_resource(Rng::new(desc.seed))?;
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
        if let Some(slot) = self.resources[id.0 as usize].as_mut() {
            if let ResourceValue::Raw(bytes) = slot.get_mut() {
                let value = self.registry.decode::<T>(id, bytes)?;
                *slot.get_mut() = ResourceValue::Typed(Box::new(value));
            }
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
            self.registry.check_size(*id, bytes)?;
        }
        components.sort_by_key(|(id, _)| *id);
        if let Some(w) = components.windows(2).find(|w| w[0].0 == w[1].0) {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("a bundle holds '{}' twice", self.registry.name(w[0].0)),
            ));
        }
        Ok(components)
    }

    /// Places a live entity with no location into the archetype of
    /// [`Self::prepare`]d components.
    pub(crate) fn place(&mut self, e: Entity, components: Vec<(SchemaId, Vec<u8>)>) {
        let types = components.iter().map(|(id, _)| *id).collect();
        let a = self.archetype_for(types);
        let arch = &mut self.archetypes[a as usize];
        for (position, (id, bytes)) in components.iter().enumerate() {
            let layout = &self.registry.info(*id).layout;
            for (k, c) in arch.columns_of(position).enumerate() {
                arch.columns[c].get_mut().push(&bytes[layout.range(k)]);
            }
        }
        arch.entities.push(e);
        let row = arch.len() - 1;
        self.set_location(e, a, row);
    }

    /// Spawns an entity with a bundle: `()` or a tuple of reflected values.
    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Result<Entity> {
        let components = encode_bundle(&self.registry, bundle)?;
        self.spawn_bytes(components)
    }

    /// Spawns an entity from packed component bytes (the host form of [`Self::spawn`]).
    pub fn spawn_bytes(&mut self, components: Vec<(SchemaId, Vec<u8>)>) -> Result<Entity> {
        let components = self.prepare(components)?;
        let e = self.entities.get_mut().reserve();
        self.place(e, components);
        Ok(e)
    }

    /// Removes an entity and all its components. Its id is dead from now on.
    pub fn despawn(&mut self, e: Entity) -> Result<()> {
        let loc = self.location(e)?;
        if let Some(moved) = self.archetypes[loc.archetype as usize].swap_remove(loc.row as usize) {
            self.set_location(moved, loc.archetype, loc.row as usize);
        }
        let entities = self.entities.get_mut();
        entities.alloc.free(e.0)?;
        entities.locations[e.index() as usize] = Location {
            archetype: PENDING,
            row: 0,
        };
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

    /// Adds (or overwrites) component `id` from packed bytes.
    pub fn insert_bytes(&mut self, e: Entity, id: SchemaId, bytes: &[u8]) -> Result<()> {
        self.registry.check(id)?;
        self.registry.check_size(id, bytes)?;
        let loc = self.location(e)?;
        let src = loc.archetype as usize;
        let row = loc.row as usize;
        let layout = &self.registry.info(id).layout;
        if let Some(position) = self.archetypes[src].position(id) {
            let arch = &mut self.archetypes[src];
            for (k, c) in arch.columns_of(position).enumerate() {
                arch.columns[c]
                    .get_mut()
                    .set_row(row, &bytes[layout.range(k)]);
            }
            return Ok(());
        }
        let mut types = self.archetypes[src].types.clone();
        let at = types.binary_search(&id).unwrap_err();
        types.insert(at, id);
        let dst = self.archetype_for(types) as usize;
        let layout = &self.registry.info(id).layout;
        let (s, d) = two_mut(&mut self.archetypes, src, dst);
        for (position, &ty) in d.types.iter().enumerate() {
            let dst_cols = d.columns_of(position);
            match s.position(ty) {
                Some(sp) => {
                    for (sc, dc) in s.columns_of(sp).zip(dst_cols) {
                        d.columns[dc]
                            .get_mut()
                            .push(s.columns[sc].get_mut().row(row));
                    }
                }
                None => {
                    for (k, dc) in dst_cols.enumerate() {
                        d.columns[dc].get_mut().push(&bytes[layout.range(k)]);
                    }
                }
            }
        }
        d.entities.push(e);
        let new_row = d.len() - 1;
        let moved = s.swap_remove(row);
        if let Some(moved) = moved {
            self.set_location(moved, src as u32, row);
        }
        self.set_location(e, dst as u32, new_row);
        Ok(())
    }

    /// Removes component `T` from `e`. Refused if `e` does not have it.
    pub fn remove<T: Reflect>(&mut self, e: Entity) -> Result<()> {
        let id = self.id::<T>()?;
        self.remove_id(e, id)
    }

    /// Removes component `id` from `e`.
    pub fn remove_id(&mut self, e: Entity, id: SchemaId) -> Result<()> {
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
        let dst = self.archetype_for(types) as usize;
        let (s, d) = two_mut(&mut self.archetypes, src, dst);
        for (position, &ty) in d.types.iter().enumerate() {
            let sp = s.position(ty).expect("subset of the source archetype");
            for (sc, dc) in s.columns_of(sp).zip(d.columns_of(position)) {
                d.columns[dc]
                    .get_mut()
                    .push(s.columns[sc].get_mut().row(row));
            }
        }
        d.entities.push(e);
        let new_row = d.len() - 1;
        if let Some(moved) = s.swap_remove(row) {
            self.set_location(moved, src as u32, row);
        }
        self.set_location(e, dst as u32, new_row);
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
        self.registry.check_size(id, bytes)?;
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
        Ok(QueryIter::new(self, Arc::new(access)))
    }

    // ---- resources --------------------------------------------------------

    /// Inserts or replaces resource `T`. `T` must be registered.
    pub fn insert_resource<T: Reflect>(&mut self, value: T) -> Result<()> {
        let id = self.id::<T>()?;
        self.registry.encode(id, &value)?; // a layout mismatch is refused here, once
        self.resources[id.0 as usize] = Some(RefCell::new(ResourceValue::Typed(Box::new(value))));
        Ok(())
    }

    /// Inserts or replaces resource `id` from packed bytes.
    pub fn insert_resource_bytes(&mut self, id: SchemaId, bytes: &[u8]) -> Result<()> {
        self.registry.check(id)?;
        self.registry.check_size(id, bytes)?;
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

    // ---- events -----------------------------------------------------------

    /// Sends event `T`; systems read it during the next [`Self::step`].
    pub fn send<T: Reflect>(&mut self, event: T) -> Result<()> {
        let id = self.id::<T>()?;
        let bytes = self.registry.encode(id, &event)?;
        self.events[id.0 as usize].get_mut().pending.push(bytes);
        Ok(())
    }

    /// Sends event `id` from packed bytes.
    pub fn send_bytes(&mut self, id: SchemaId, bytes: &[u8]) -> Result<()> {
        self.registry.check(id)?;
        self.registry.check_size(id, bytes)?;
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

    // ---- schedule ---------------------------------------------------------

    /// Adds a system at the end of the schedule. Refused, with the type and
    /// field named, when a query writes or reads a component the system did
    /// not declare, a declared name is unknown, or the name is taken.
    pub fn add_system<F>(&mut self, desc: SystemDesc, run: F) -> Result<()>
    where
        F: Fn(&mut SystemContext<'_>) -> Result<()> + Send + 'static,
    {
        let system = System::resolve(self, desc, Box::new(run))?;
        self.systems.push(system);
        Ok(())
    }

    /// System names in run order.
    pub fn systems(&self) -> Vec<&str> {
        self.systems.iter().map(|s| s.name.as_str()).collect()
    }

    /// Runs one fixed tick: delivers pending events, runs every system in
    /// order (applying each system's commands when it returns), then advances
    /// `Time`. A system error stops the tick and is returned with the tick and
    /// the system named; the world is left as it was when the system failed
    /// (earlier systems' effects stay; restore a snapshot to undo them).
    pub fn step(&mut self) -> Result<()> {
        for queue in &mut self.events {
            let queue = queue.get_mut();
            queue.current = std::mem::take(&mut queue.pending);
        }
        let tick = self.time().tick;
        let mut commands = Vec::new();
        for i in 0..self.systems.len() {
            let result = {
                let system = &self.systems[i];
                let mut ctx = SystemContext::new(self, system, &mut commands);
                (system.run)(&mut ctx)
            };
            let failed = match result {
                Err(err) => Some(err),
                Ok(()) => self.apply(&mut commands).err(),
            };
            if let Some(err) = failed {
                let err = err.context(format!("tick {tick}, system '{}'", self.systems[i].name));
                self.discard(&mut commands);
                return Err(err);
            }
        }
        self.resource_mut::<Time>()?.advance();
        Ok(())
    }

    /// Applies a system's commands in order; on error the rest stay in `commands`.
    fn apply(&mut self, commands: &mut Vec<Command>) -> Result<()> {
        let mut drained = std::mem::take(commands).into_iter();
        while let Some(command) = drained.next() {
            let result = match command {
                Command::Spawn(e, components) => {
                    self.place(e, components);
                    Ok(())
                }
                Command::Insert(e, id, bytes) => self.insert_bytes(e, id, &bytes),
                Command::Remove(e, id) => self.remove_id(e, id),
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
    fn discard(&mut self, commands: &mut Vec<Command>) {
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
