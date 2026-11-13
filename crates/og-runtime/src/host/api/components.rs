//! World's components: tested, inserted, removed, read and written.

use super::*;

pub fn og_world_has(world: u32, entity_id: f64, comp: String) -> bool {
    let Ok(e) = entity(entity_id) else {
        return false;
    };
    let Ok(eng) = engine(world) else { return false };
    if running(world).is_some() {
        // Inside a system: answered from the system's own views.
        let st = eng.state.borrow();
        let Some(id) = st.names_by_id.iter().position(|n| *n == comp) else {
            return false;
        };
        return HANDLES.with(|t| {
            let t = t.borrow();
            let has = |(start, n): (u32, u32)| {
                t.view_types[start as usize..(start + n) as usize].contains(&(id as u32))
            };
            t.views.items.iter().any(|(owner, rec)| {
                *owner == world && has(rec.types) && {
                    // SAFETY: the views are live while the system runs.
                    let ents =
                        unsafe { std::slice::from_raw_parts(rec.entities, rec.count as usize) };
                    ents.contains(&e)
                }
            })
        });
    }
    let Ok(w) = eng.world() else { return false };
    match w.id_by_name(&comp) {
        Ok(id) => w.components_of(e).is_ok_and(|c| c.contains(&id)),
        Err(_) => false,
    }
}

pub fn og_world_insert(
    world: u32,
    entity_id: f64,
    comp: String,
    value: Option<String>,
) -> Result<(), String> {
    if let Some(f) = running(world) {
        return (|| {
            let eng = engine(world)?;
            // SAFETY: the frame is published while its system runs.
            let e = live(&eng, "world.insert", entity_id, |e| {
                Ok(unsafe { f.ctx() }.is_alive(e))
            })?;
            let (layout, id) = running_record(&eng, &comp, RecordKind::Component)?;
            let value = parse(value.as_deref())?;
            let bytes = layout.encode(None, Some(&value))?;
            layout.check_ranges(&value)?;
            queued_structural();
            // SAFETY: the frame is published while its system runs.
            unsafe { f.ctx() }.insert_bytes(e, id, &bytes).map_err(err)
        })();
    }
    (|| {
        let eng = engine(world)?;
        let e = live(&eng, "world.insert", entity_id, |e| {
            Ok(eng.world()?.is_alive(e))
        })?;
        let (layout, id) = component(&eng, &comp)?;
        let value = parse(value.as_deref())?;
        let bytes = layout.encode(None, Some(&value))?;
        layout.check_ranges(&value)?;
        before_structural(&eng);
        eng.world()?.insert_bytes(e, id, &bytes).map_err(err)
    })()
}

pub fn og_world_remove(world: u32, entity_id: f64, comp: String) -> Result<(), String> {
    if let Some(f) = running(world) {
        return (|| {
            let eng = engine(world)?;
            // SAFETY: the frame is published while its system runs.
            let e = live(&eng, "world.remove", entity_id, |e| {
                Ok(unsafe { f.ctx() }.is_alive(e))
            })?;
            let (_, id) = running_record(&eng, &comp, RecordKind::Component)?;
            queued_structural();
            // SAFETY: the frame is published while its system runs.
            unsafe { f.ctx() }.remove_id(e, id).map_err(err)
        })();
    }
    (|| {
        let eng = engine(world)?;
        let e = live(&eng, "world.remove", entity_id, |e| {
            Ok(eng.world()?.is_alive(e))
        })?;
        let (_, id) = component(&eng, &comp)?;
        before_structural(&eng);
        eng.world()?.remove_id(e, id).map_err(err)
    })()
}

pub fn og_world_get(world: u32, entity_id: f64, comp: String) -> Result<serde_json::Value, String> {
    if let Some(f) = running(world) {
        return (|| {
            let eng = engine(world)?;
            // SAFETY: the frame is published while its system runs.
            let e = live(&eng, "world.get", entity_id, |e| {
                Ok(unsafe { f.ctx() }.is_alive(e))
            })?;
            let (layout, id) = running_record(&eng, &comp, RecordKind::Component)?;
            // SAFETY: the frame is published while its system runs.
            let bytes = unsafe { f.ctx() }
                .get_bytes(e, id)
                .map_err(|e| held_by_view(e, &comp, "world.get"))?;
            Ok(layout.decode(&bytes))
        })();
    }
    (|| {
        let eng = engine(world)?;
        let e = live(&eng, "world.get", entity_id, |e| {
            Ok(eng.world()?.is_alive(e))
        })?;
        let (layout, id) = component(&eng, &comp)?;
        let bytes = eng.world()?.get_bytes(e, id).map_err(err)?;
        Ok(layout.decode(&bytes))
    })()
}

/// `world.get`/`world.set` inside a system on an entity whose column of
/// `comp` a view of the system holds: the reason, with the way through.
pub(super) fn held_by_view(e: og_core::Error, comp: &str, call: &str) -> String {
    if e.kind() == og_core::ErrorKind::Access && e.reason().contains("is borrowed") {
        return format!(
            "{call} of '{comp}' is refused: this system took a column view of '{comp}' (a.column) on this entity's archetype, and the view holds the column until the system returns; read and write the entity through that view's arrays (its row: a.entities), or call {call} before taking the view"
        );
    }
    err(e)
}

pub fn og_world_set(world: u32, entity_id: f64, comp: String, value: String) -> Result<(), String> {
    if let Some(f) = running(world) {
        return (|| {
            let eng = engine(world)?;
            // SAFETY: the frame is published while its system runs.
            let e = live(&eng, "world.set", entity_id, |e| {
                Ok(unsafe { f.ctx() }.is_alive(e))
            })?;
            let (layout, id) = running_record(&eng, &comp, RecordKind::Component)?;
            let value = parse(Some(&value))?;
            // SAFETY: the frame is published while its system runs.
            let ctx = unsafe { f.ctx() };
            let held = |e| held_by_view(e, &comp, "world.set");
            let current = ctx.get_bytes(e, id).map_err(held)?;
            let bytes = layout.encode(Some(&current), Some(&value))?;
            ctx.set_bytes(e, id, &bytes).map_err(held)
        })();
    }
    (|| {
        let eng = engine(world)?;
        let e = live(&eng, "world.set", entity_id, |e| {
            Ok(eng.world()?.is_alive(e))
        })?;
        let (layout, id) = component(&eng, &comp)?;
        let value = parse(Some(&value))?;
        let w = eng.world()?;
        let current = w.get_bytes(e, id).map_err(err)?;
        let bytes = layout.encode(Some(&current), Some(&value))?;
        // Overwriting a present component moves nothing, but the runtime
        // treats every insert as structural so views never see a half-write.
        before_structural(&eng);
        eng.world()?.insert_bytes(e, id, &bytes).map_err(err)
    })()
}

/// `og_engine_read_component`: the packed bytes of component `component`
/// (its number, `og_engine_component_id`) of an entity, written to the
/// start of the shared buffer. Answers the layout epoch they were read
/// under, or 0 when refused: `og_world_get` then says why, so nothing is
/// logged or registered here.
pub fn og_engine_read_component(world: u32, entity_id: f64, component: u32) -> u32 {
    let read = || -> Option<u32> {
        let eng = engine(world).ok()?;
        let e = entity(entity_id).ok()?;
        let id = og_ecs::SchemaId::from_index(component);
        let bytes = match running(world) {
            // SAFETY: the frame is published while its system runs.
            Some(f) => unsafe { f.ctx() }.get_bytes(e, id).ok()?,
            None => eng.world().ok()?.get_bytes(e, id).ok()?,
        };
        crate::host::shared::with(|s| {
            s.get_mut(..bytes.len())?.copy_from_slice(&bytes);
            Some(())
        })?;
        Some(crate::engine::layout_epoch())
    };
    read().unwrap_or(0)
}

/// `og_engine_write_component`: overwrites the byte ranges the shared
/// buffer names in a component the entity has. The record is at the
/// buffer's start; from the next word boundary after it come a count and
/// that many `(offset, size)` word pairs, the fields the value named. The
/// other bytes keep their value, as `og_world_set` keeps the fields a value
/// does not name. Answers the layout epoch: the write happened only when
/// that is `epoch`, the one the bytes were made with. 0 when refused:
/// `og_world_set` then says why.
pub fn og_engine_write_component(world: u32, entity_id: f64, component: u32, epoch: u32) -> u32 {
    let current = crate::engine::layout_epoch();
    if epoch != current {
        return current;
    }
    // The record with the named ranges taken from the shared buffer.
    let patched = |mut bytes: Vec<u8>| -> Option<Vec<u8>> {
        crate::host::shared::with(|s| {
            let word = |at: usize| -> Option<usize> {
                Some(u32::from_le_bytes(s.get(at..at + 4)?.try_into().ok()?) as usize)
            };
            let mut at = bytes.len().next_multiple_of(4);
            for _ in 0..word(at)? {
                let from = word(at + 4)?;
                let to = from.checked_add(word(at + 8)?)?;
                bytes.get_mut(from..to)?.copy_from_slice(&s[from..to]);
                at += 8;
            }
            Some(bytes)
        })
    };
    let write = || -> Option<()> {
        let eng = engine(world).ok()?;
        let e = entity(entity_id).ok()?;
        let id = og_ecs::SchemaId::from_index(component);
        if let Some(f) = running(world) {
            // SAFETY: the frame is published while its system runs.
            let ctx = unsafe { f.ctx() };
            let bytes = patched(ctx.get_bytes(e, id).ok()?)?;
            return ctx.set_bytes(e, id, &bytes).ok();
        }
        // The entity has the component, so this overwrites and adds nothing.
        let bytes = patched(eng.world().ok()?.get_bytes(e, id).ok()?)?;
        before_structural(&eng);
        eng.world().ok()?.insert_bytes(e, id, &bytes).ok()
    };
    write().map_or(0, |()| current)
}
