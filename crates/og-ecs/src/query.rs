//! Queries: the archetypes holding a set of components, each seen as column
//! slices (one slice per field, one element per entity).
//!
//! A query is described by names ([`QueryDesc`]: `read`, `write`, `without`),
//! resolved against the world's registry. Iterating yields one
//! [`ArchetypeView`] per matching non-empty archetype, in archetype creation
//! order; within a view, rows follow [`ArchetypeView::entities`]. Column
//! access is checked: `read` needs the component in the query's `read` or
//! `write` list, `write` needs it in `write`, and the element type must match
//! the field's [`og_core::FieldType`].

use crate::entity::Entity;
use crate::registry::{FieldRef, Registry, SchemaId};
use crate::storage::Archetype;
use crate::world::{World, busy};
use og_core::reflect::{FieldValue, Reflect};
use og_core::{Error, ErrorKind, Result};
use std::cell::{Ref, RefMut};
use std::sync::Arc;

/// What a query matches and may touch, by type name. `Default` matches every
/// entity and touches nothing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QueryDesc {
    /// Components that must be present and may be read.
    pub read: Vec<String>,
    /// Components that must be present and may be read and written.
    pub write: Vec<String>,
    /// Components that must be absent.
    pub without: Vec<String>,
    /// Change filters: components (required, as `read`) whose value changed
    /// since the system last ran (between ticks: since the last tick began).
    /// A component is tracked from the moment a system filters on it.
    pub changed: Vec<String>,
    /// Components (required, as `read`) added since then.
    pub added: Vec<String>,
    /// Components the entity lost since then (it matches the rest).
    pub removed: Vec<String>,
}

impl QueryDesc {
    pub fn new() -> QueryDesc {
        QueryDesc::default()
    }

    pub fn read<T: Reflect>(mut self) -> QueryDesc {
        self.read.push(T::NAME.to_string());
        self
    }

    pub fn write<T: Reflect>(mut self) -> QueryDesc {
        self.write.push(T::NAME.to_string());
        self
    }

    pub fn without<T: Reflect>(mut self) -> QueryDesc {
        self.without.push(T::NAME.to_string());
        self
    }

    pub fn changed<T: Reflect>(mut self) -> QueryDesc {
        self.changed.push(T::NAME.to_string());
        self
    }

    pub fn added<T: Reflect>(mut self) -> QueryDesc {
        self.added.push(T::NAME.to_string());
        self
    }

    pub fn removed<T: Reflect>(mut self) -> QueryDesc {
        self.removed.push(T::NAME.to_string());
        self
    }
}

/// A resolved [`QueryDesc`].
#[derive(Clone, Debug)]
pub(crate) struct QueryAccess {
    pub owner: String,
    /// Read ∪ write, sorted.
    pub required: Vec<SchemaId>,
    /// Sorted.
    pub writes: Vec<SchemaId>,
    pub without: Vec<SchemaId>,
    pub changed: Vec<SchemaId>,
    pub added: Vec<SchemaId>,
    pub removed: Vec<SchemaId>,
}

fn resolve_names(registry: &Registry, names: &[String], owner: &str) -> Result<Vec<SchemaId>> {
    let mut ids = names
        .iter()
        .map(|n| {
            registry
                .id_by_name(n)
                .map_err(|e| e.context(format!("{owner}: unknown type '{n}'")))
        })
        .collect::<Result<Vec<_>>>()?;
    og_core::sort::sort(&mut ids);
    ids.dedup();
    Ok(ids)
}

impl QueryAccess {
    pub fn resolve(registry: &Registry, desc: &QueryDesc, owner: &str) -> Result<QueryAccess> {
        let read = resolve_names(registry, &desc.read, owner)?;
        let writes = resolve_names(registry, &desc.write, owner)?;
        let without = resolve_names(registry, &desc.without, owner)?;
        let changed = resolve_names(registry, &desc.changed, owner)?;
        let added = resolve_names(registry, &desc.added, owner)?;
        let removed = resolve_names(registry, &desc.removed, owner)?;
        let mut required: Vec<SchemaId> = read
            .iter()
            .chain(&writes)
            .chain(&changed)
            .chain(&added)
            .copied()
            .collect();
        og_core::sort::sort(&mut required);
        required.dedup();
        if let Some(id) = without.iter().find(|id| required.binary_search(id).is_ok()) {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "{owner}: '{}' is both required and excluded, so the query never matches",
                    registry.name(*id)
                ),
            ));
        }
        Ok(QueryAccess {
            owner: owner.to_string(),
            required,
            writes,
            without,
            changed,
            added,
            removed,
        })
    }

    /// Whether the query filters rows by change.
    pub fn filters(&self) -> bool {
        !(self.changed.is_empty() && self.added.is_empty() && self.removed.is_empty())
    }

    pub fn matches(&self, arch: &Archetype) -> bool {
        self.required.iter().all(|id| arch.has(*id)) && !self.without.iter().any(|id| arch.has(*id))
    }
}

/// Iterator over the archetypes a query matches.
pub struct QueryIter<'w> {
    world: &'w World,
    access: Arc<QueryAccess>,
    next: usize,
    /// The change tick filters read after.
    since: u64,
    /// Per `removed` filter, the entities that lost it since `since`.
    removed: Arc<Vec<Vec<Entity>>>,
}

impl<'w> QueryIter<'w> {
    pub(crate) fn new(world: &'w World, access: Arc<QueryAccess>, since: u64) -> QueryIter<'w> {
        let removed = access
            .removed
            .iter()
            .map(|id| world.removed_since(*id, since))
            .collect();
        QueryIter {
            world,
            access,
            next: 0,
            since,
            removed: Arc::new(removed),
        }
    }

    /// Rows of `arch` that pass the change filters, in row order.
    fn filtered(&self, arch: &Archetype) -> Option<Vec<u32>> {
        if !self.access.filters() {
            return None;
        }
        let since = self.since;
        let positions = |ids: &[SchemaId]| -> Vec<usize> {
            ids.iter().filter_map(|id| arch.position(*id)).collect()
        };
        let changed = positions(&self.access.changed);
        let added = positions(&self.access.added);
        let rows = (0..arch.len())
            .filter(|&row| {
                changed
                    .iter()
                    .all(|&p| arch.row_ticks(p, row).is_some_and(|(_, c)| c > since))
                    && added
                        .iter()
                        .all(|&p| arch.row_ticks(p, row).is_some_and(|(a, _)| a > since))
                    && self
                        .removed
                        .iter()
                        .all(|list| list.binary_search(&arch.entities[row]).is_ok())
            })
            .map(|row| row as u32)
            .collect();
        Some(rows)
    }

    /// Entities matched, summed over archetypes.
    pub fn entity_count(self) -> usize {
        self.map(|v| v.len()).sum()
    }
}

impl<'w> Iterator for QueryIter<'w> {
    type Item = ArchetypeView<'w>;

    fn next(&mut self) -> Option<ArchetypeView<'w>> {
        while let Some(arch) = self.world.archetypes.get(self.next) {
            self.next += 1;
            if !arch.entities.is_empty() && self.access.matches(arch) {
                let rows = self.filtered(arch);
                if rows.as_ref().is_some_and(|r| r.is_empty()) {
                    continue;
                }
                return Some(ArchetypeView {
                    registry: &self.world.registry,
                    arch,
                    access: self.access.clone(),
                    rows,
                });
            }
        }
        None
    }
}

/// One matching archetype: its entities and its columns.
pub struct ArchetypeView<'w> {
    registry: &'w Registry,
    arch: &'w Archetype,
    access: Arc<QueryAccess>,
    /// The rows the change filters pass; `None` without filters (all).
    rows: Option<Vec<u32>>,
}

impl<'w> ArchetypeView<'w> {
    pub fn len(&self) -> usize {
        self.arch.len()
    }

    pub fn is_empty(&self) -> bool {
        self.arch.len() == 0
    }

    /// Row order of every column in this view.
    pub fn entities(&self) -> &'w [Entity] {
        &self.arch.entities
    }

    /// The rows the query's change filters pass, in row order (every row
    /// for a query without filters). Columns still hold every row.
    pub fn rows(&self) -> Vec<u32> {
        match &self.rows {
            Some(r) => r.clone(),
            None => (0..self.arch.len() as u32).collect(),
        }
    }

    /// Whether the query filters rows by change ([`Self::rows`]).
    pub fn filtered(&self) -> bool {
        self.rows.is_some()
    }

    /// Whether this archetype has component `id` (whether or not the query
    /// names it).
    pub fn has(&self, id: SchemaId) -> bool {
        self.arch.has(id)
    }

    /// The component types of this archetype, in id order.
    pub fn types(&self) -> &'w [SchemaId] {
        &self.arch.types
    }

    fn column(&self, field: FieldRef, write: bool) -> Result<usize> {
        self.registry.check_field(field)?;
        let allowed = if write {
            &self.access.writes
        } else {
            &self.access.required
        };
        if allowed.binary_search(&field.component).is_err() {
            return Err(Error::new(
                ErrorKind::Access,
                format!(
                    "{}: no {} access to {} (declare '{}' in the query's {})",
                    self.access.owner,
                    if write { "write" } else { "read" },
                    self.registry.field_name(field),
                    self.registry.name(field.component),
                    if write {
                        "write list"
                    } else {
                        "read or write list"
                    },
                ),
            ));
        }
        Ok(self
            .arch
            .column_index(field.component, field.field)
            .expect("a matched archetype holds every required component"))
    }

    fn check_type<T: FieldValue>(&self, field: FieldRef) -> Result<()> {
        if T::TYPE.scalar() == field.ty.scalar() && T::TYPE.lanes() == field.ty.lanes() {
            Ok(())
        } else {
            Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "{} is {}, not {}",
                    self.registry.field_name(field),
                    field.ty.name(),
                    T::TYPE.name()
                ),
            ))
        }
    }

    /// The column of `field` as a typed slice.
    pub fn read<T: FieldValue>(&self, field: FieldRef) -> Result<Ref<'w, [T]>> {
        self.check_type::<T>(field)?;
        let bytes = self.read_bytes(field)?;
        Ok(Ref::map(bytes, |b| bytemuck::cast_slice::<u8, T>(b)))
    }

    /// The column of `field` as a mutable typed slice.
    pub fn write<T: FieldValue>(&self, field: FieldRef) -> Result<RefMut<'w, [T]>> {
        self.check_type::<T>(field)?;
        let bytes = self.write_bytes(field)?;
        Ok(RefMut::map(bytes, |b| bytemuck::cast_slice_mut::<u8, T>(b)))
    }

    /// The column of `field` as raw little-endian bytes (`len × size`).
    pub fn read_bytes(&self, field: FieldRef) -> Result<Ref<'w, [u8]>> {
        let c = self.column(field, false)?;
        let col = self.arch.columns[c]
            .try_borrow()
            .map_err(|_| busy(&self.registry.field_name(field)))?;
        Ok(Ref::map(col, |c| c.bytes()))
    }

    pub fn write_bytes(&self, field: FieldRef) -> Result<RefMut<'w, [u8]>> {
        let c = self.column(field, true)?;
        let col = self.arch.columns[c]
            .try_borrow_mut()
            .map_err(|_| busy(&self.registry.field_name(field)))?;
        Ok(RefMut::map(col, |c| c.bytes_mut()))
    }
}
