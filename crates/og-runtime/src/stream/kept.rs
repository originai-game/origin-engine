//! A persisted cell's entities kept while it is unloaded (`StreamCell.persist`,
//! G3-3), as the og.cells block holds them ([`super::cell_store::Row`]):
//! taken as the cell unloads, checked and spawned again as it loads.

use super::super::codec::Kind;
use super::super::engine::Engine;
use super::cell_store::Row;
use og_ecs::{Entity, World};
use std::collections::BTreeMap;

/// The rows of `order` (a cell's members and their descendants, parents
/// first): each entity's id, name and every component, by type name.
pub(crate) fn capture(eng: &Engine, world: &World, order: &[Entity]) -> Result<Vec<Row>, String> {
    let err = |e: og_core::Error| e.reason().to_string();
    let mut rows = Vec::with_capacity(order.len());
    for &e in order {
        let mut components = Vec::new();
        for id in world.components_of(e).map_err(err)? {
            let schema = world.schema(id).map_err(err)?;
            components.push((
                schema.name.clone(),
                schema.layout_hash(),
                world.get_bytes(e, id).map_err(err)?,
            ));
        }
        og_core::sort::sort_by(&mut components, |a, b| a.0.cmp(&b.0));
        let name = eng
            .state
            .borrow()
            .name_of
            .get(&e.to_bits())
            .cloned()
            .unwrap_or_default();
        rows.push(Row {
            id: e.to_bits(),
            name,
            components,
        });
    }
    Ok(rows)
}

/// Whether every row's components can come back in `world`: each type
/// registered (a lazy engine record is registered here) with the layout
/// it was kept with.
pub(crate) fn check(eng: &Engine, world: &mut World, rows: &[Row]) -> Result<(), String> {
    let mut seen = std::collections::BTreeSet::new();
    for (ty, hash, _) in rows.iter().flat_map(|r| r.components.iter()) {
        if !seen.insert(ty.as_str()) {
            continue;
        }
        super::super::lazy::ensure(eng, world, ty)?;
        let id = world
            .id_by_name(ty)
            .map_err(|_| format!("{ty} is not registered"))?;
        let now = world
            .schema(id)
            .map_err(|e| e.reason().to_string())?
            .layout_hash();
        if now != *hash {
            return Err(format!("{ty} has another layout than when they were kept"));
        }
    }
    Ok(())
}

/// The entity fields of type `ty`: their byte offsets.
fn entity_fields(eng: &Engine, ty: &str) -> Vec<usize> {
    eng.state
        .borrow()
        .layouts
        .get(ty)
        .map(|l| {
            l.fields
                .iter()
                .enumerate()
                .filter(|(_, f)| f.kind == Kind::Entity)
                .map(|(i, _)| l.offset(i))
                .collect()
        })
        .unwrap_or_default()
}

fn read_u64(bytes: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_le_bytes(bytes.get(at..at + 8)?.try_into().ok()?))
}

/// Spawns rows `spawned.len()..end` (`spawned` holds the entities the rows
/// before them spawned, in order), each entity field that holds a kept
/// row's id pointed at that row's new entity. Once the last row is in,
/// the fields that named a row spawned after theirs are pointed too.
pub(crate) fn spawn(
    eng: &Engine,
    world: &mut World,
    rows: &[Row],
    spawned: &mut Vec<Entity>,
    end: usize,
) -> Result<(), String> {
    let err = |e: og_core::Error| e.reason().to_string();
    let index: BTreeMap<u64, usize> = rows.iter().enumerate().map(|(i, r)| (r.id, i)).collect();
    let mut map: BTreeMap<u64, u64> = rows
        .iter()
        .zip(spawned.iter())
        .map(|(r, e)| (r.id, e.to_bits()))
        .collect();
    let mut fields: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    super::super::engine::before_structural(eng);
    for row in &rows[spawned.len()..end.min(rows.len())] {
        let mut components = Vec::with_capacity(row.components.len());
        for (ty, _, bytes) in &row.components {
            let id = world.id_by_name(ty).map_err(err)?;
            let mut bytes = bytes.clone();
            for &at in fields
                .entry(ty.as_str())
                .or_insert_with(|| entity_fields(eng, ty))
                .iter()
            {
                if let Some(to) = read_u64(&bytes, at).and_then(|v| map.get(&v)) {
                    bytes[at..at + 8].copy_from_slice(&to.to_le_bytes());
                }
            }
            components.push((id, bytes));
        }
        let e = world.spawn_bytes(components).map_err(err)?;
        if !row.name.is_empty() && !eng.state.borrow().names.contains_key(&row.name) {
            super::super::api::set_name(eng, e, &row.name);
        }
        map.insert(row.id, e.to_bits());
        spawned.push(e);
    }
    if spawned.len() < rows.len() {
        return Ok(());
    }
    // Forward references: a field naming a row spawned after its own.
    for (j, row) in rows.iter().enumerate() {
        for (ty, _, bytes) in &row.components {
            let offsets = fields
                .entry(ty.as_str())
                .or_insert_with(|| entity_fields(eng, ty))
                .clone();
            let forward: Vec<(usize, u64)> = offsets
                .iter()
                .filter_map(|&at| {
                    let v = read_u64(bytes, at)?;
                    (index.get(&v).copied()? > j).then(|| (at, map[&v]))
                })
                .collect();
            if forward.is_empty() {
                continue;
            }
            let id = world.id_by_name(ty).map_err(err)?;
            let e = spawned[j];
            let mut live = world.get_bytes(e, id).map_err(err)?;
            for (at, to) in forward {
                live[at..at + 8].copy_from_slice(&to.to_le_bytes());
            }
            world.insert_bytes(e, id, &live).map_err(err)?;
        }
    }
    Ok(())
}
