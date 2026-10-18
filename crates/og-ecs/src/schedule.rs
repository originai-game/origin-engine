//! Systems and the schedule.
//!
//! A system is a [`SystemDesc`] (its name, the types it reads and writes, and
//! its queries) plus a function of a [`SystemContext`]. Registration resolves
//! every name and refuses a system whose queries read or write a component it
//! did not declare; at run time every access goes through the context, which
//! refuses anything outside the declaration. Systems run in registration
//! order, one after another; structural changes (spawn, insert, remove,
//! despawn) are commands applied when the system returns, so a system never
//! sees its own structural changes and the next system sees all of them.

use crate::entity::Entity;
use crate::query::{QueryAccess, QueryDesc, QueryIter};
use crate::registry::SchemaId;
use crate::world::{busy, dead, encode_bundle, typed_ref, Bundle, ResourceValue, World};
use og_core::reflect::Reflect;
use og_core::{Error, ErrorKind, Result, Time};
use std::cell::{Ref, RefMut};
use std::sync::Arc;

/// A system's declaration. `reads`/`writes` name types (components,
/// resources, events); a write implies a read. `Time` is readable by every
/// system without a declaration and is never writable.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SystemDesc {
    pub name: String,
    pub reads: Vec<String>,
    pub writes: Vec<String>,
    /// Queries the system iterates, by index, through [`SystemContext::query`].
    pub queries: Vec<QueryDesc>,
}

impl SystemDesc {
    pub fn new(name: impl Into<String>) -> SystemDesc {
        SystemDesc {
            name: name.into(),
            ..SystemDesc::default()
        }
    }

    pub fn read<T: Reflect>(mut self) -> SystemDesc {
        self.reads.push(T::NAME.to_string());
        self
    }

    pub fn write<T: Reflect>(mut self) -> SystemDesc {
        self.writes.push(T::NAME.to_string());
        self
    }

    pub fn query(mut self, query: QueryDesc) -> SystemDesc {
        self.queries.push(query);
        self
    }
}

pub(crate) type SystemFn = Box<dyn Fn(&mut SystemContext<'_>) -> Result<()> + Send>;

pub(crate) struct System {
    pub name: String,
    /// Reads ∪ writes, sorted.
    pub reads: Vec<SchemaId>,
    pub writes: Vec<SchemaId>,
    pub queries: Vec<Arc<QueryAccess>>,
    /// The queries as declared, for [`SystemInfo`].
    pub desc_queries: Vec<QueryDesc>,
    /// Entities each query matched when the system last ran.
    pub matched: Vec<usize>,
    /// The tick the system last ran in.
    pub last_tick: Option<u64>,
    pub run: SystemFn,
}

/// What [`crate::World::system_info`] reports about one system.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemInfo {
    pub name: String,
    /// Declared reads that are not also writes, by type name in id order.
    pub reads: Vec<String>,
    /// Declared writes, by type name in id order.
    pub writes: Vec<String>,
    /// The declared queries.
    pub queries: Vec<QueryDesc>,
    /// Entities its queries matched when it last ran (summed over queries;
    /// an entity two queries match counts twice); 0 before it has run.
    pub matched: usize,
    /// The same, per query.
    pub query_matched: Vec<usize>,
    /// The tick it last ran in (`None` before the first tick). A tick that
    /// failed in an earlier system did not run it.
    pub last_tick: Option<u64>,
}

fn ids(world: &World, names: &[String], system: &str) -> Result<Vec<SchemaId>> {
    let mut out = Vec::with_capacity(names.len());
    for n in names {
        out.push(world.registry.id_by_name(n).map_err(|_| {
            Error::new(
                ErrorKind::NotFound,
                format!("system '{system}' declares unknown type '{n}'"),
            )
        })?);
    }
    out.sort();
    out.dedup();
    Ok(out)
}

impl System {
    pub fn resolve(world: &World, desc: SystemDesc, run: SystemFn) -> Result<System> {
        let name = desc.name.clone();
        if name.is_empty() {
            return Err(Error::new(ErrorKind::Invalid, "a system needs a name"));
        }
        if world.systems.iter().any(|s| s.name == name) {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("a system named '{name}' is already scheduled"),
            ));
        }
        let writes = ids(world, &desc.writes, &name)?;
        if writes.contains(&world.scale_id) {
            return Err(Error::new(
                ErrorKind::Access,
                format!("system '{name}' declares a write to og.TimeScale; use set_time_scale and hit_stop"),
            ));
        }
        if writes.contains(&world.time_id) {
            return Err(Error::new(
                ErrorKind::Access,
                format!("system '{name}' declares a write to Time; the world advances Time, systems read it"),
            ));
        }
        let mut reads = ids(world, &desc.reads, &name)?;
        reads.extend(&writes);
        reads.sort();
        reads.dedup();
        let mut queries = Vec::with_capacity(desc.queries.len());
        for (i, q) in desc.queries.iter().enumerate() {
            let owner = format!("system '{name}' query {i}");
            let access = QueryAccess::resolve(&world.registry, q, &owner)?;
            for id in &access.writes {
                if writes.binary_search(id).is_err() {
                    return Err(Error::new(
                        ErrorKind::Access,
                        format!(
                            "{owner} writes '{}' but the system does not declare it in writes",
                            world.registry.name(*id)
                        ),
                    ));
                }
            }
            for id in &access.required {
                if reads.binary_search(id).is_err() {
                    return Err(Error::new(
                        ErrorKind::Access,
                        format!(
                            "{owner} reads '{}' but the system does not declare it in reads or writes",
                            world.registry.name(*id)
                        ),
                    ));
                }
            }
            queries.push(Arc::new(access));
        }
        Ok(System {
            name,
            reads,
            writes,
            matched: vec![0; queries.len()],
            queries,
            desc_queries: desc.queries,
            last_tick: None,
            run,
        })
    }
}

pub(crate) enum Command {
    Spawn(Entity, Vec<(SchemaId, Vec<u8>)>),
    Insert(Entity, SchemaId, Vec<u8>),
    Remove(Entity, SchemaId),
    Despawn(Entity),
}

/// What a running system can see and do: exactly what it declared.
pub struct SystemContext<'w> {
    world: &'w World,
    system: &'w System,
    commands: &'w mut Vec<Command>,
}

impl<'w> SystemContext<'w> {
    pub(crate) fn new(
        world: &'w World,
        system: &'w System,
        commands: &'w mut Vec<Command>,
    ) -> Self {
        SystemContext {
            world,
            system,
            commands,
        }
    }

    pub fn name(&self) -> &str {
        &self.system.name
    }

    /// The clock of the tick being run.
    pub fn time(&self) -> Time {
        self.world.time()
    }

    /// Slow motion and hit stop, readable by every system.
    pub fn time_scale(&self) -> og_core::TimeScale {
        self.world.time_scale()
    }

    /// The scaled length of this tick: what physics, animation and tweens
    /// advance by (0 during a hit stop).
    pub fn scaled_dt(&self) -> f32 {
        self.world.scaled_dt()
    }

    /// Sets the global time scale; any system may (an engine service, like
    /// reading `Time`). Later systems of this tick see it.
    pub fn set_time_scale(&self, scale: f32) -> Result<()> {
        let scale = crate::world::check_scale(scale)?;
        self.edit_time_scale(|s| s.scale = scale)
    }

    /// Starts or extends a hit stop of `ticks` ticks (any system may).
    pub fn hit_stop(&self, ticks: u32) -> Result<()> {
        self.edit_time_scale(|s| s.hit_stop = s.hit_stop.max(ticks))
    }

    fn edit_time_scale(&self, f: impl FnOnce(&mut og_core::TimeScale)) -> Result<()> {
        let name = og_core::TimeScale::NAME;
        let cell = self.world.resource_cell(self.world.scale_id)?;
        let mut value = cell
            .try_borrow_mut()
            .map_err(|_| crate::world::busy(name))?;
        match &mut *value {
            crate::world::ResourceValue::Typed(b) => match b.downcast_mut::<og_core::TimeScale>() {
                Some(s) => {
                    f(s);
                    Ok(())
                }
                None => Err(crate::world::no_resource(name)),
            },
            crate::world::ResourceValue::Raw(_) => Err(crate::world::no_resource(name)),
        }
    }

    /// The id of a registered Rust type.
    pub fn id<T: Reflect>(&self) -> Result<SchemaId> {
        self.world.id::<T>()
    }

    fn require(&self, id: SchemaId, write: bool) -> Result<()> {
        let list = if write {
            &self.system.writes
        } else {
            &self.system.reads
        };
        if list.binary_search(&id).is_ok()
            || (!write && (id == self.world.time_id || id == self.world.scale_id))
        {
            return Ok(());
        }
        Err(Error::new(
            ErrorKind::Access,
            format!(
                "system '{}' did not declare {} access to '{}'",
                self.system.name,
                if write { "write" } else { "read" },
                self.world.registry.name(id)
            ),
        ))
    }

    /// The archetypes of declared query `index`.
    pub fn query(&self, index: usize) -> Result<QueryIter<'w>> {
        let access = self.system.queries.get(index).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                format!(
                    "system '{}' declares {} queries, not {}",
                    self.system.name,
                    self.system.queries.len(),
                    index + 1
                ),
            )
        })?;
        Ok(QueryIter::new(self.world, access.clone()))
    }

    pub fn resource<T: Reflect>(&self) -> Result<Ref<'w, T>> {
        let id = self.world.id::<T>()?;
        self.require(id, false)?;
        typed_ref(self.world.resource_cell(id)?, T::NAME)
    }

    pub fn resource_mut<T: Reflect>(&self) -> Result<RefMut<'w, T>> {
        let id = self.world.id::<T>()?;
        self.require(id, true)?;
        let cell = self.world.resource_cell(id)?;
        let value = cell
            .try_borrow_mut()
            .map_err(|_| crate::world::busy(T::NAME))?;
        RefMut::filter_map(value, |v| match v {
            ResourceValue::Typed(b) => b.downcast_mut::<T>(),
            ResourceValue::Raw(_) => None,
        })
        .map_err(|_| crate::world::no_resource(T::NAME))
    }

    /// Events of type `T` delivered this tick (sent during the previous tick
    /// or by the host before this one).
    pub fn events<T: Reflect>(&self) -> Result<Vec<T>> {
        let id = self.world.id::<T>()?;
        self.require(id, false)?;
        let queue = self.world.events[id.0 as usize].borrow();
        queue
            .current
            .iter()
            .map(|b| self.world.registry.decode(id, b))
            .collect()
    }

    /// Sends event `T`; it is delivered at the next tick.
    pub fn send<T: Reflect>(&self, event: T) -> Result<()> {
        let id = self.world.id::<T>()?;
        self.require(id, true)?;
        let bytes = self.world.registry.encode(id, &event)?;
        self.world.registry.check_value(id, &bytes)?;
        self.world.events[id.0 as usize]
            .borrow_mut()
            .pending
            .push(bytes);
        Ok(())
    }

    pub fn is_alive(&self, e: Entity) -> bool {
        self.world.is_alive(e)
    }

    /// A copy of component `T` of any entity (declared read).
    pub fn get<T: Reflect>(&self, e: Entity) -> Result<T> {
        let id = self.world.id::<T>()?;
        self.require(id, false)?;
        self.world.get(e)
    }

    /// Overwrites component `T` of an entity that has it, immediately
    /// (declared write; not a structural change).
    pub fn set<T: Reflect>(&self, e: Entity, value: T) -> Result<()> {
        let id = self.world.id::<T>()?;
        self.require(id, true)?;
        let bytes = self.world.registry.encode(id, &value)?;
        self.world.set_in_place(e, id, &bytes)
    }

    /// Spawns an entity when the system returns. The id is usable at once
    /// in this system's other commands.
    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Result<Entity> {
        let components = self
            .world
            .prepare(encode_bundle(&self.world.registry, bundle)?)?;
        for (id, _) in &components {
            self.require(*id, true)?;
        }
        let e = self.world.entities.borrow_mut().reserve();
        self.commands.push(Command::Spawn(e, components));
        Ok(e)
    }

    /// Adds or overwrites component `T` of `e` when the system returns.
    pub fn insert<T: Reflect>(&mut self, e: Entity, value: T) -> Result<()> {
        let id = self.world.id::<T>()?;
        self.require(id, true)?;
        if !self.world.is_alive(e) {
            return Err(dead(e));
        }
        self.world.registry.check_not_event_only(id, "component")?;
        let bytes = self.world.registry.encode(id, &value)?;
        self.world.registry.check_value(id, &bytes)?;
        self.commands.push(Command::Insert(e, id, bytes));
        Ok(())
    }

    /// Removes component `T` from `e` when the system returns.
    pub fn remove<T: Reflect>(&mut self, e: Entity) -> Result<()> {
        let id = self.world.id::<T>()?;
        self.require(id, true)?;
        if !self.world.is_alive(e) {
            return Err(dead(e));
        }
        self.commands.push(Command::Remove(e, id));
        Ok(())
    }

    // ---- packed bytes: the host form of the calls above, by SchemaId -----

    /// Spawns an entity from packed component bytes when the system returns
    /// (the byte form of [`Self::spawn`]; every component declared written).
    pub fn spawn_bytes(&mut self, components: Vec<(SchemaId, Vec<u8>)>) -> Result<Entity> {
        let components = self.world.prepare(components)?;
        for (id, _) in &components {
            self.require(*id, true)?;
        }
        let e = self.world.entities.borrow_mut().reserve();
        self.commands.push(Command::Spawn(e, components));
        Ok(e)
    }

    /// Adds or overwrites component `id` of `e` from packed bytes when the
    /// system returns (declared write).
    pub fn insert_bytes(&mut self, e: Entity, id: SchemaId, bytes: &[u8]) -> Result<()> {
        self.world.registry.check(id)?;
        self.require(id, true)?;
        if !self.world.is_alive(e) {
            return Err(dead(e));
        }
        self.world.registry.check_not_event_only(id, "component")?;
        self.world.registry.check_value(id, bytes)?;
        self.commands.push(Command::Insert(e, id, bytes.to_vec()));
        Ok(())
    }

    /// Removes component `id` from `e` when the system returns (declared
    /// write).
    pub fn remove_id(&mut self, e: Entity, id: SchemaId) -> Result<()> {
        self.world.registry.check(id)?;
        self.require(id, true)?;
        if !self.world.is_alive(e) {
            return Err(dead(e));
        }
        self.commands.push(Command::Remove(e, id));
        Ok(())
    }

    /// Packed bytes of component `id` of any entity (declared read).
    pub fn get_bytes(&self, e: Entity, id: SchemaId) -> Result<Vec<u8>> {
        self.world.registry.check(id)?;
        self.require(id, false)?;
        self.world.get_bytes(e, id)
    }

    /// Overwrites component `id` of an entity that has it from packed bytes,
    /// immediately (declared write; not a structural change).
    pub fn set_bytes(&self, e: Entity, id: SchemaId, bytes: &[u8]) -> Result<()> {
        self.world.registry.check(id)?;
        self.require(id, true)?;
        self.world.set_in_place(e, id, bytes)
    }

    /// Packed bytes of resource `id` (declared read; `Time` always).
    pub fn resource_bytes(&self, id: SchemaId) -> Result<Vec<u8>> {
        self.world.registry.check(id)?;
        self.require(id, false)?;
        self.world.resource_bytes(id)
    }

    /// Overwrites resource `id`, which must be present, from packed bytes,
    /// immediately (declared write; never `Time`).
    pub fn set_resource_bytes(&self, id: SchemaId, bytes: &[u8]) -> Result<()> {
        let registry = &self.world.registry;
        registry.check(id)?;
        self.require(id, true)?;
        registry.check_value(id, bytes)?;
        let name = registry.name(id);
        let cell = self.world.resource_cell(id)?;
        let mut value = cell.try_borrow_mut().map_err(|_| busy(name))?;
        let info = registry.info(id);
        *value = match &info.rust {
            Some(rust) => ResourceValue::Typed((rust.decode)(&info.layout, bytes)?),
            None => ResourceValue::Raw(bytes.to_vec()),
        };
        Ok(())
    }

    /// Packed bytes of the events of `id` delivered this tick, in the order
    /// they were sent (declared read).
    pub fn events_bytes(&self, id: SchemaId) -> Result<Vec<Vec<u8>>> {
        self.world.registry.check(id)?;
        self.require(id, false)?;
        let queue = self.world.events[id.0 as usize]
            .try_borrow()
            .map_err(|_| busy(self.world.registry.name(id)))?;
        Ok(queue.current.clone())
    }

    /// Sends event `id` from packed bytes; it is delivered at the next tick
    /// (declared write).
    pub fn send_bytes(&self, id: SchemaId, bytes: &[u8]) -> Result<()> {
        self.world.registry.check(id)?;
        self.require(id, true)?;
        self.world.registry.check_value(id, bytes)?;
        self.world.events[id.0 as usize]
            .try_borrow_mut()
            .map_err(|_| busy(self.world.registry.name(id)))?
            .pending
            .push(bytes.to_vec());
        Ok(())
    }

    /// Whether resource `id` is present (declared read).
    pub fn has_resource_id(&self, id: SchemaId) -> Result<bool> {
        self.world.registry.check(id)?;
        self.require(id, false)?;
        Ok(self.world.has_resource_id(id))
    }

    /// The name of an entity, if it has one (names are readable by every
    /// system; they change only between ticks).
    pub fn name_of(&self, e: Entity) -> Option<&'w str> {
        self.world.name(e)
    }

    /// The live entity holding a name.
    pub fn find(&self, name: &str) -> Option<Entity> {
        self.world.find(name)
    }

    /// Despawns `e` when the system returns (any system may despawn). A
    /// second despawn of `e` in the same batch is a no-op; an `insert` or
    /// `remove` of `e` queued after it fails when applied and stops the tick.
    pub fn despawn(&mut self, e: Entity) -> Result<()> {
        if !self.world.is_alive(e) {
            return Err(dead(e));
        }
        self.commands.push(Command::Despawn(e));
        Ok(())
    }
}
