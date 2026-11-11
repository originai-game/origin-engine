//! The reactive and spatial members of the world (`World.children`,
//! `removed`, `spatial`, `state`, `setState`, `SpatialIndex`,
//! `ArchetypeView.rows`) and `og.defineHook`: og-ecs's change detection,
//! hooks, state machines and spatial hash, as both hosts reach them.

use super::{
    RecordKind, component, engine, entity, err, parse, result, result_void, running, running_record,
};
use crate::engine::{DECL, HANDLES, declare_hook};
use crate::wire;
use og_core::reflect::FieldType;
use og_ecs::{Entity, FieldRef, SchemaId, SpatialDesc};
use serde_json::{Value, json};

/// og.Parent's `entity` field, inside a system or out.
fn parent_field(eng: &crate::engine::Engine, in_system: bool) -> Result<FieldRef, String> {
    let (_, id) = if in_system {
        running_record(eng, "og.Parent", RecordKind::Component)?
    } else {
        component(eng, "og.Parent")?
    };
    Ok(FieldRef {
        component: id,
        field: 0,
        ty: FieldType::U64,
    })
}

fn entity_list(list: Vec<Entity>) -> Value {
    Value::Array(list.into_iter().map(|e| json!(e.to_bits())).collect())
}

pub fn og_world_children(world: u32, entity_id: f64) -> String {
    let list = (|| -> Result<Vec<Entity>, String> {
        let eng = engine(world)?;
        let Ok(e) = entity(entity_id) else {
            return Ok(Vec::new());
        };
        if let Some(f) = running(world) {
            let field = parent_field(&eng, true)?;
            // SAFETY: the frame is published while its system runs.
            let ctx = unsafe { f.ctx() };
            if !ctx.is_alive(e) {
                return Ok(Vec::new());
            }
            return ctx.referencing(field, e).map_err(err);
        }
        let field = parent_field(&eng, false)?;
        let w = eng.world()?;
        if !w.is_alive(e) {
            return Ok(Vec::new());
        }
        w.referencing(field, e).map_err(err)
    })();
    entity_list(list.unwrap_or_default()).to_string()
}

pub fn og_world_removed(world: u32, comp: String) -> String {
    result((|| {
        let eng = engine(world)?;
        if let Some(f) = running(world) {
            let (_, id) = running_record(&eng, &comp, RecordKind::Component)?;
            // SAFETY: the frame is published while its system runs.
            let list = unsafe { f.ctx() }
                .removed(id)
                .map_err(|e| format!("world.removed: {}", err(e)))?;
            return Ok(entity_list(list));
        }
        let (_, id) = component(&eng, &comp)?;
        let w = eng.world()?;
        if !w.is_tracked(id) {
            return Err(format!(
                "world.removed: '{comp}' is not tracked: a system filtering on it (SystemDesc.changed, added or removed) tracks it"
            ));
        }
        Ok(entity_list(w.removed(id).map_err(err)?))
    })())
}

fn ids(
    eng: &crate::engine::Engine,
    list: &[Value],
    in_system: bool,
) -> Result<Vec<SchemaId>, String> {
    let mut out = Vec::new();
    for name in list.iter().filter_map(Value::as_str) {
        let (_, id) = if in_system {
            running_record(eng, name, RecordKind::Component)?
        } else {
            component(eng, name)?
        };
        out.push(id);
    }
    Ok(out)
}

pub fn og_world_spatial(world: u32, desc: Option<String>) -> String {
    result((|| {
        let eng = engine(world)?;
        let desc: wire::SpatialDesc = wire::arg(desc.as_deref(), "desc")?;
        let in_system = running(world).is_some();
        let (_, transform) = if in_system {
            running_record(&eng, "og.Transform", RecordKind::Component)?
        } else {
            component(&eng, "og.Transform")?
        };
        let spatial = SpatialDesc {
            field: FieldRef {
                component: transform,
                field: 0,
                ty: FieldType::Vec3,
            },
            cell: desc.cell,
            with: ids(&eng, &desc.with, in_system)?,
            without: ids(&eng, &desc.without, in_system)?,
        };
        let index = match running(world) {
            // SAFETY: the frame is published while its system runs.
            Some(f) => unsafe { f.ctx() }.spatial(&spatial).map_err(|e| {
                format!(
                    "world.spatial: {} (a system reaches positions through og.Transform among its reads or writes)",
                    err(e)
                )
            })?,
            None => eng.world()?.spatial(&spatial).map_err(err)?,
        };
        let h = HANDLES.with(|t| t.borrow_mut().spatial.push((world, index)));
        Ok(json!(h))
    })())
}

fn with_index<T>(h: u32, f: impl FnOnce(&og_ecs::SpatialHash) -> T) -> Option<T> {
    HANDLES.with(|t| t.borrow().spatial.get(h).map(|(_, i)| f(i)))
}

fn neighbours(list: Vec<og_ecs::Neighbour>) -> String {
    Value::Array(
        list.into_iter()
            .map(|n| json!({ "entity": n.entity.to_bits(), "distance": n.distance }))
            .collect(),
    )
    .to_string()
}

fn center(v: &str) -> [f32; 3] {
    wire::arg::<glam::Vec3>(Some(v), "center")
        .unwrap_or_default()
        .to_array()
}

pub fn og_spatial_index_get_count(h: u32) -> u32 {
    with_index(h, |i| i.len() as u32).unwrap_or(0)
}

pub fn og_spatial_index_within(h: u32, c: String, radius: f64) -> String {
    let c = center(&c);
    neighbours(with_index(h, |i| i.within(c, radius as f32)).unwrap_or_default())
}

pub fn og_spatial_index_nearest(h: u32, c: String, count: u32) -> String {
    let c = center(&c);
    neighbours(with_index(h, |i| i.nearest(c, count as usize)).unwrap_or_default())
}

pub fn og_world_state(world: u32, name: String) -> String {
    let value = (|| -> Result<String, String> {
        if let Some(f) = running(world) {
            // SAFETY: the frame is published while its system runs.
            return unsafe { f.ctx() }
                .state(&name)
                .map(str::to_string)
                .map_err(err);
        }
        let eng = engine(world)?;
        eng.world()?.state(&name).map(str::to_string).map_err(err)
    })();
    match value {
        Ok(v) => json!(v).to_string(),
        Err(_) => "null".into(),
    }
}

pub fn og_world_set_state(world: u32, name: String, value: String) -> String {
    result_void((|| {
        if let Some(f) = running(world) {
            // SAFETY: the frame is published while its system runs.
            return unsafe { f.ctx() }.set_state(&name, &value).map_err(err);
        }
        let eng = engine(world)?;
        eng.world()?.set_state(&name, &value).map_err(err)
    })())
}

/// The rows of view `view` its query's change filters pass (every row
/// without filters); empty for a stale view.
pub fn view_rows(view: u32) -> Vec<u32> {
    HANDLES.with(|t| {
        let t = t.borrow();
        let Some((_, rec)) = t.views.get(view) else {
            return Vec::new();
        };
        match &rec.rows {
            Some(rows) => rows.clone(),
            None => (0..rec.count).collect(),
        }
    })
}

pub fn og_hook_get_name(hook: u32) -> String {
    DECL.with(|d| {
        d.borrow()
            .hooks
            .get(hook as usize)
            .map(|h| h.name.clone())
            .unwrap_or_default()
    })
}

pub fn og_system_define_hook(name: String, comp: String, desc: String, run: u32) -> u32 {
    let desc = parse(Some(&desc)).unwrap_or(Value::Null);
    declare_hook(name, comp, &desc, run)
}
