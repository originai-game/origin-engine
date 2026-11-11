//! The binding symbols, host-neutral: each function is the body of the
//! trait method of the same name (wire arguments in, wire answer out).

use super::assets::{self, Payload, Status, index_of};
use super::codec::{Layout, RecordKind};
use super::engine::{
    CommandDecl, DECL, Engine, HANDLES, QueryRec, before_structural, declare_game, declare_system,
    engine, queued_structural, running, world_query_views,
};
use super::probe::{self as rprobe, NameOp, with_probe};
use super::{ok, parse, refuse, wire};
use og_core::reflect::Reflect;
use og_core::{Rng, Time};
use og_ecs::{Entity, QueryDesc, World};
use serde_json::{Value, json};

mod actions;
mod commands;
mod controls;
mod intents;
mod loading;
mod reactive;
mod render;

use super::{engine, environment, extract, feel, fx, look, modules, views};
pub use actions::*;
pub use commands::*;
pub use controls::*;
pub use intents::*;
pub use loading::*;
pub use reactive::*;
pub use render::*;

fn err(e: og_core::Error) -> String {
    e.reason().to_string()
}

fn entity(n: f64) -> Result<Entity, String> {
    if n.fract() != 0.0 || !(1.0..9_007_199_254_740_992.0).contains(&n) {
        return Err(format!("{n} is not an entity"));
    }
    Ok(Entity::from_bits(n as u64))
}

/// Entity `n`, an argument of `call` that must name a live entity (`alive`
/// answers for the running system or the world, or why it cannot), or the refusal each such
/// call words alike. A number below 2^32 has generation 0, which no entity
/// ever has: a bare index, most often read from `Archetype.entities`, so the
/// refusal says so, and the engine log carries it once per system and call
/// (`og run` prints it), so a Result the game ignores still shows.
fn live(
    eng: &Engine,
    call: &str,
    n: f64,
    alive: impl FnOnce(Entity) -> Result<bool, String>,
) -> Result<Entity, String> {
    if let Ok(e) = entity(n)
        && alive(e)?
    {
        return Ok(e);
    }
    let mut reason = format!("{call}: no live entity has the handle {n}");
    if n.fract() == 0.0 && (0.0..4_294_967_296.0).contains(&n) {
        reason.push_str(
            " (Archetype.entities holds (index, generation) pairs; use a.entity(i) for a handle)",
        );
        let frame = running(eng.id);
        let mut st = eng.state.borrow_mut();
        let who = frame
            .and_then(|f| st.systems.get(f.system))
            .map(|s| format!("system '{}', ", s.name))
            .unwrap_or_default();
        if st
            .undeclared_logged
            .insert(format!("{who}{call} bare index"))
        {
            let tick = match frame {
                // SAFETY: the frame is published while its system runs.
                Some(f) => unsafe { f.ctx() }.time().tick,
                None => eng.world().map(|w| w.time().tick).unwrap_or(0),
            };
            st.log
                .push(json!({ "tick": tick, "warning": true, "text": format!("{who}{reason}") }));
        }
    }
    Err(reason)
}

fn entity_number(e: Entity) -> Value {
    json!(e.to_bits())
}

/// A component layout by schema name, or why the token is not usable.
pub(crate) fn component(eng: &Engine, name: &str) -> Result<(Layout, og_ecs::SchemaId), String> {
    super::lazy::ensure(eng, eng.world()?, name)?;
    let st = eng.state.borrow();
    let layout = st.layouts.get(name).cloned();
    let warning = st
        .warnings
        .iter()
        .find(|w| w.starts_with(&format!("{name} ")))
        .cloned();
    drop(st);
    let layout = match layout {
        Some(l) if l.kind == RecordKind::Component => l,
        Some(_) => return Err(format!("'{name}' is a resource, not a component")),
        None => {
            return Err(warning.unwrap_or_else(|| {
                format!("'{name}' is not a registered component: declare it with defineComponent")
            }));
        }
    };
    let id = eng.world()?.id_by_name(name).map_err(err)?;
    Ok((layout, id))
}

/// A record's layout and id while a system runs (the world is borrowed by
/// the step, so the id comes from the runtime's own table).
pub(crate) fn running_record(
    eng: &Engine,
    name: &str,
    kind: RecordKind,
) -> Result<(Layout, og_ecs::SchemaId), String> {
    let st = eng.state.borrow();
    let layout = st
        .layouts
        .get(name)
        .filter(|l| l.kind == kind)
        .cloned()
        .ok_or_else(|| format!("'{name}' is not a registered {}", kind.as_str()))?;
    let id = st
        .names_by_id
        .iter()
        .position(|n| n == name)
        .map(|i| og_ecs::SchemaId::from_index(i as u32))
        .ok_or_else(|| format!("'{name}' is not registered with this world"))?;
    Ok((layout, id))
}

/// A running system reached for something it did not declare (`access` is
/// `read` or `write`, `name` the record): the reason, worded the same on
/// every path, and one engine log line per system, access and name, so a
/// publish build (where the TypeScript runtime refuses instead of throwing)
/// says it once in the engine log rather than every tick.
pub(crate) fn undeclared(eng: &Engine, system: usize, access: &str, name: &str) -> String {
    let mut st = eng.state.borrow_mut();
    let sys = st
        .systems
        .get(system)
        .map_or_else(|| "?".to_string(), |s| s.name.clone());
    let list = if access == "write" {
        "writes"
    } else {
        "reads or writes"
    };
    let reason =
        format!("system '{sys}' did not declare '{name}' among its {list} ({access} refused)");
    if st
        .undeclared_logged
        .insert(format!("{sys} {access} {name}"))
    {
        let tick = eng.world().map(|w| w.time().tick).unwrap_or(0);
        st.log.push(json!({ "tick": tick, "warning": true, "text": format!("{reason}; declare it in the system's {list}") }));
    }
    reason
}

// ── reflect ─────────────────────────────────────────────────────────────────

fn define_record(name: String, fields: &str, kind: RecordKind) -> String {
    let declared = (|| {
        if name.starts_with("og.") {
            return Err(format!(
                "'{name}': the og. prefix belongs to the engine's own records"
            ));
        }
        let fields = parse(Some(fields))?;
        let layout = Layout::from_fields(&name, kind, &fields)?;
        layout.schema().validate().map_err(err)?;
        Ok(layout)
    })();
    DECL.with(|d| {
        let mut d = d.borrow_mut();
        match declared {
            Ok(layout) => {
                if let Some(prev) = d.records.iter().find(|l| l.schema == name) {
                    if prev.schema().layout_hash() != layout.schema().layout_hash()
                        || prev.kind != layout.kind
                    {
                        let reason = format!("'{name}' is declared twice with different layouts");
                        d.errors.push(reason.clone());
                        return refuse(reason);
                    }
                } else {
                    d.records.push(layout);
                }
                ok(json!({ "kind": kind.as_str(), "name": name }))
            }
            Err(reason) => {
                d.errors.push(reason.clone());
                refuse(reason)
            }
        }
    })
}

pub fn og_reflect_define_component(name: String, fields: String) -> String {
    define_record(name, &fields, RecordKind::Component)
}

pub fn og_reflect_define_resource(name: String, fields: String) -> String {
    define_record(name, &fields, RecordKind::Resource)
}

pub fn og_reflect_define_event(name: String, fields: String) -> String {
    define_record(name, &fields, RecordKind::Event)
}

// ── world ───────────────────────────────────────────────────────────────────

pub fn og_world_get_tick(world: u32) -> u32 {
    if let Some(f) = running(world) {
        // SAFETY: the frame is published while its system runs.
        return unsafe { f.ctx() }.time().tick as u32;
    }
    engine(world)
        .and_then(|e| e.world().map(|w| w.time().tick as u32))
        .unwrap_or(0)
}

pub(crate) fn set_name(eng: &Engine, e: Entity, name: &str) {
    runtime_name(eng, e, name);
    eng.state
        .borrow_mut()
        .name_ops
        .push(NameOp::Set(e, name.to_string()));
}

/// The runtime's own view of a name (og.world.find reads it at once).
fn runtime_name(eng: &Engine, e: Entity, name: &str) {
    let mut st = eng.state.borrow_mut();
    st.names.insert(name.to_string(), e.to_bits());
    st.name_of.insert(e.to_bits(), name.to_string());
}

pub(crate) fn forget_name(eng: &Engine, e: Entity) {
    let mut st = eng.state.borrow_mut();
    if let Some(name) = st.name_of.remove(&e.to_bits()) {
        st.names.remove(&name);
        st.name_ops.push(NameOp::Remove(name));
    }
}

/// A desc's `parent` (SpawnDesc, of `call`): none when absent or null, else
/// its number; anything else is refused as the runtime refuses a value that
/// is not an entity where one is expected (generated/abi.ts `notEntity`).
fn spawn_parent(call: &str, desc: &Value) -> Result<Option<f64>, String> {
    let pass = "pass the number world.spawn, world.find or a.entity(i) returned";
    let what = match desc.get("parent") {
        None | Some(Value::Null) => return Ok(None),
        Some(Value::Number(n)) => return Ok(n.as_f64()),
        Some(Value::Object(o)) if o.contains_key("ok") => {
            "a Result, not an entity (world.spawn answers { ok, value }: pass its value)"
                .to_string()
        }
        Some(Value::String(_)) => format!("a string, not an entity ({pass})"),
        Some(Value::Bool(_)) => format!("a boolean, not an entity ({pass})"),
        Some(_) => format!("an object, not an entity ({pass})"),
    };
    Err(format!("{call}: parent is {what}"))
}

/// [`spawn`] for the engine's own entities (a glTF's lights).
pub(crate) fn spawn_entity(eng: &Engine, call: &str, desc: &Value) -> Result<Entity, String> {
    spawn(eng, call, desc)
}

/// A SpawnDesc read from `desc` (of `call`): its parent checked first, so a
/// value that is not an entity is refused as [`spawn_parent`] words it.
fn spawn_desc(call: &str, desc: &Value) -> Result<crate::wire::SpawnDesc, String> {
    spawn_parent(call, desc)?;
    wire::value(desc, "desc")
}

/// The og.Transform fields a desc names (the layout's defaults for the rest).
fn spawn_transform(desc: &Value) -> Value {
    let mut value = serde_json::Map::new();
    for key in ["position", "quaternion", "scale"] {
        if let Some(v) = desc.get(key).filter(|v| !v.is_null()) {
            value.insert(key.to_string(), v.clone());
        }
    }
    Value::Object(value)
}

fn spawn(eng: &Engine, call: &str, desc: &Value) -> Result<Entity, String> {
    let d = spawn_desc(call, desc)?;
    let name = d.name;
    if !name.is_empty() {
        if eng.state.borrow().names.contains_key(&name) {
            return Err(format!("an entity named '{name}' already exists"));
        }
        if name.starts_with('#') || name.chars().any(|c| c.is_whitespace() || ":;=".contains(c)) {
            return Err(format!(
                "'{name}' cannot be addressed by the probe: a name has no whitespace, ':', ';' or '=' and does not start with '#'"
            ));
        }
    }
    let mut components = Vec::new();
    if !d.bare {
        let (layout, id) = component(eng, "og.Transform")?;
        components.push((id, layout.encode(None, Some(&spawn_transform(desc)))?));
    }
    if let Some(p) = d.parent.map(|p| p as f64) {
        let parent = entity(p)?;
        if !eng.world()?.is_alive(parent) {
            return Err(format!("the parent {parent} is not alive"));
        }
        let (layout, id) = component(eng, "og.Parent")?;
        components.push((id, layout.encode(None, Some(&json!({ "entity": p })))?));
    }
    before_structural(eng);
    let e = eng.world()?.spawn_bytes(components).map_err(err)?;
    if !name.is_empty() {
        set_name(eng, e, &name);
    }
    eng.state
        .borrow_mut()
        .game_spawned
        .insert(e.to_bits(), spawn_source(&d.source));
    Ok(e)
}

/// A desc's `source` (SpawnDesc): the line that spawned the entity, kept
/// outside a publish build.
fn spawn_source(source: &str) -> Option<std::sync::Arc<str>> {
    if cfg!(feature = "publish") || source.is_empty() {
        return None;
    }
    Some(source.into())
}

/// `world.spawn` inside a system: queued, applied when the system returns
/// (its id is usable at once); every component it gets must be among the
/// system's writes.
fn spawn_in_system(
    eng: &Engine,
    f: &super::engine::Frame,
    call: &str,
    desc: &Value,
) -> Result<Entity, String> {
    let d = spawn_desc(call, desc)?;
    let name = d.name;
    if !name.is_empty() {
        let st = eng.state.borrow();
        if st.names.contains_key(&name) || st.held_names.contains(&name) {
            return Err(format!("an entity named '{name}' already exists"));
        }
    }
    let mut components = Vec::new();
    if !d.bare {
        let (layout, id) = running_record(eng, "og.Transform", RecordKind::Component)?;
        components.push((id, layout.encode(None, Some(&spawn_transform(desc)))?));
    }
    if let Some(p) = d.parent.map(|p| p as f64) {
        let (layout, id) = running_record(eng, "og.Parent", RecordKind::Component)?;
        components.push((id, layout.encode(None, Some(&json!({ "entity": p })))?));
    }
    queued_structural();
    // SAFETY: the frame is published while its system runs.
    let e = unsafe { f.ctx() }
        .spawn_bytes(components)
        .map_err(|e| {
            // The system never named og.Transform or og.Parent itself: say
            // that spawn writes them.
            let access = e.kind() == og_core::ErrorKind::Access;
            let reason = err(e);
            let cause = if !access {
                None
            } else if reason.ends_with("access to 'og.Parent'") {
                Some("with parent writes og.Parent; add og.Parent to the system's writes")
            } else if reason.ends_with("access to 'og.Transform'") {
                Some("writes og.Transform unless the desc says bare: true; add og.Transform to the system's writes")
            } else {
                None
            };
            match cause {
                Some(c) => format!("{reason} ({call} {c})"),
                None => reason,
            }
        })?;
    if !name.is_empty() {
        // Into the world with the spawn, when the system returns: a
        // snapshot a later system takes this tick has it.
        runtime_name(eng, e, &name);
        // SAFETY: as above.
        unsafe { f.ctx() }.set_name(e, &name);
    }
    eng.state
        .borrow_mut()
        .game_spawned
        .insert(e.to_bits(), spawn_source(&d.source));
    Ok(e)
}

pub fn og_world_spawn(world: u32, desc: Option<String>) -> Result<serde_json::Value, String> {
    if let Some(f) = running(world) {
        return (|| {
            let eng = engine(world)?;
            let desc = parse(desc.as_deref())?;
            spawn_in_system(&eng, f, "world.spawn", &desc).map(entity_number)
        })();
    }
    (|| {
        let eng = engine(world)?;
        let desc = parse(desc.as_deref())?;
        spawn(&eng, "world.spawn", &desc).map(entity_number)
    })()
}

/// `e` and every entity whose og.Parent chain reaches it, children first.
fn with_descendants(world: &World, root: Entity) -> Result<Vec<Entity>, String> {
    let mut parents: Vec<(Entity, Entity)> = Vec::new();
    if let Ok(id) = world.id_by_name("og.Parent") {
        let field = world.field_by_id(id, "entity").map_err(err)?;
        let q = QueryDesc {
            read: vec!["og.Parent".into()],
            ..QueryDesc::default()
        };
        for a in world.query(&q).map_err(err)? {
            let p = a.read::<u64>(field).map_err(err)?;
            for (child, parent) in a.entities().iter().zip(p.iter()) {
                parents.push((*child, Entity::from_bits(*parent)));
            }
        }
    }
    let mut out = vec![root];
    let mut i = 0;
    while i < out.len() {
        let cur = out[i];
        for (child, parent) in &parents {
            if *parent == cur && !out.contains(child) {
                out.push(*child);
            }
        }
        i += 1;
    }
    out.reverse();
    Ok(out)
}

pub fn og_world_despawn(world: u32, entity_id: f64) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        if let Some(f) = running(world) {
            // SAFETY: the frame is published while its system runs.
            let e = live(&eng, "world.despawn", entity_id, |e| {
                Ok(unsafe { f.ctx() }.is_alive(e))
            })?;
            queued_structural();
            // SAFETY: as above.
            return unsafe { f.ctx() }.despawn(e).map_err(err);
        }
        let e = live(&eng, "world.despawn", entity_id, |e| {
            Ok(eng.world()?.is_alive(e))
        })?;
        let all = with_descendants(eng.world()?, e)?;
        before_structural(&eng);
        for d in all {
            eng.world()?.despawn(d).map_err(err)?;
            forget_name(&eng, d);
            eng.state.borrow_mut().game_spawned.remove(&d.to_bits());
        }
        Ok(())
    })()
}

pub fn og_world_alive(world: u32, entity_id: f64) -> bool {
    let Ok(e) = entity(entity_id) else {
        return false;
    };
    if let Some(f) = running(world) {
        // SAFETY: the frame is published while its system runs.
        return unsafe { f.ctx() }.is_alive(e);
    }
    engine(world)
        .and_then(|eng| eng.world().map(|w| w.is_alive(e)))
        .unwrap_or(false)
}

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
fn held_by_view(e: og_core::Error, comp: &str, call: &str) -> String {
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
        crate::shared::with(|s| {
            s.get_mut(..bytes.len())?.copy_from_slice(&bytes);
            Some(())
        })?;
        Some(super::engine::layout_epoch())
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
    let current = super::engine::layout_epoch();
    if epoch != current {
        return current;
    }
    // The record with the named ranges taken from the shared buffer.
    let patched = |mut bytes: Vec<u8>| -> Option<Vec<u8>> {
        crate::shared::with(|s| {
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

fn time_json(t: Time, scale: og_core::TimeScale, rate: u32) -> Value {
    let mut v =
        json!({ "tick": t.tick, "dt": f64::from(t.dt), "elapsed": t.elapsed, "rate": rate });
    if let (Value::Object(o), Value::Object(s)) = (&mut v, super::feel::time_scale_json(scale, t)) {
        o.extend(s);
    }
    v
}

fn rng_json(r: &Rng) -> Value {
    json!({ "seed": r.seed(), "state": r.save() })
}

/// A resource's current value as JSON, outside or inside a system.
pub(crate) fn resource_value(eng: &Engine, name: &str) -> Result<Value, String> {
    let rate = eng.state.borrow().rate;
    if let Some(f) = running(eng.id) {
        // SAFETY: the frame is published while its system runs.
        let ctx = unsafe { f.ctx() };
        let st = eng.state.borrow();
        let s = &st.systems[f.system];
        let declared = s.reads.iter().chain(&s.writes).any(|r| r == name);
        if name == Time::NAME {
            return Ok(time_json(ctx.time(), ctx.time_scale(), rate));
        }
        if !declared {
            let system = f.system;
            drop(st);
            return Err(undeclared(eng, system, "read", name));
        }
        if name == Rng::NAME {
            return ctx.resource::<Rng>().map(|r| rng_json(&r)).map_err(err);
        }
        drop(st);
        let (layout, id) = running_record(eng, name, RecordKind::Resource)?;
        let bytes = ctx.resource_bytes(id).map_err(err)?;
        return Ok(layout.decode(&bytes));
    }
    let w = eng.world()?;
    if name == Time::NAME {
        return Ok(time_json(w.time(), w.time_scale(), rate));
    }
    if name == Rng::NAME {
        return w.resource::<Rng>().map(|r| rng_json(&r)).map_err(err);
    }
    let layout = eng
        .state
        .borrow()
        .layouts
        .get(name)
        .filter(|l| l.kind == RecordKind::Resource)
        .cloned()
        .ok_or_else(|| format!("'{name}' is not a registered resource"))?;
    // A lazy resource no game has named yet (og.TimeGroups): its default,
    // read without registering it, so reading leaves the hash alone.
    let Ok(id) = w.id_by_name(name) else {
        if crate::lazy::is_deferred(eng, name) {
            return Ok(layout.decode(&layout.encode(None, None)?));
        }
        return Err(format!("no type named '{name}'"));
    };
    let bytes = w.resource_bytes(id).map_err(err)?;
    Ok(layout.decode(&bytes))
}

pub fn og_world_resource(world: u32, res: String) -> Result<serde_json::Value, String> {
    engine(world).and_then(|eng| resource_value(&eng, &res))
}

pub fn og_world_find(world: u32, name: String) -> String {
    let found = engine(world).ok().and_then(|eng| {
        let st = eng.state.borrow();
        st.names.get(&name).copied()
    });
    match found {
        Some(bits) if og_world_alive(world, bits as f64) => json!(bits).to_string(),
        _ => "null".into(),
    }
}

/// `world.prefabOf`: the entity's og.PrefabLink, or null (no link, a dead
/// entity, or a world that never placed a prefab).
pub fn og_world_prefab_of(world: u32, entity_id: f64) -> Value {
    og_world_get(world, entity_id, super::prefab::LINK.to_string()).unwrap_or(Value::Null)
}

pub fn og_world_name_of(world: u32, entity_id: f64) -> String {
    let name = engine(world).ok().and_then(|eng| {
        let st = eng.state.borrow();
        st.name_of.get(&(entity_id as u64)).cloned()
    });
    match name {
        Some(n) if og_world_alive(world, entity_id) => json!(n).to_string(),
        _ => "null".into(),
    }
}

pub fn og_world_query(world: u32, desc: Option<String>) -> Result<serde_json::Value, String> {
    if running(world).is_some() {
        return Err("a system receives its query as its first argument; world.query is for commands and callbacks".into());
    }
    (|| {
        let eng = engine(world)?;
        let desc: wire::QueryDesc = wire::arg(desc.as_deref(), "desc")?;
        let names = |v: &[Value]| -> Vec<String> {
            v.iter()
                .filter_map(|n| n.as_str().map(str::to_string))
                .collect()
        };
        let q = QueryDesc {
            read: Vec::new(),
            write: names(&desc.with),
            without: names(&desc.without),
            changed: names(&desc.changed),
            added: names(&desc.added),
            removed: names(&desc.removed),
        };
        for n in q
            .write
            .iter()
            .chain(&q.without)
            .chain(&q.changed)
            .chain(&q.added)
            .chain(&q.removed)
        {
            component(&eng, n)?;
        }
        // Refusals (a filter on an untracked component) now, not as an
        // empty list later.
        eng.world()?.query(&q).map_err(err)?;
        let h = HANDLES.with(|t| {
            t.borrow_mut().world_queries.push((
                eng.id,
                QueryRec::World {
                    desc: q,
                    views: (0, 0),
                },
            ))
        });
        Ok(json!(h))
    })()
}

fn event_layout(eng: &Engine, name: &str) -> Result<(Layout, og_ecs::SchemaId), String> {
    super::lazy::ensure(eng, eng.world()?, name)?;
    let st = eng.state.borrow();
    match st.layouts.get(name) {
        Some(l) if l.kind == RecordKind::Event => {
            let l = l.clone();
            drop(st);
            let id = eng.world()?.id_by_name(name).map_err(err)?;
            Ok((l, id))
        }
        _ => Err(st
            .warnings
            .iter()
            .find(|w| w.starts_with(&format!("{name} ")))
            .cloned()
            .unwrap_or_else(|| format!("'{name}' is not a registered event"))),
    }
}

/// `world.events`: inside a system, the events delivered this tick (sent
/// during the previous one), which the system must declare among its reads;
/// between ticks, the events sent since the last tick, which the next tick
/// delivers. The IDL answers a list, so a refusal is an empty list.
pub fn og_world_events(world: u32, event: String) -> String {
    let list = (|| -> Result<Vec<Value>, String> {
        let eng = engine(world)?;
        let bytes = if let Some(f) = running(world) {
            if crate::platform::is_platform_event(&event) && crate::platform::withheld(world) {
                return Ok(Vec::new());
            }
            let (layout, id) = running_record(&eng, &event, RecordKind::Event)?;
            // SAFETY: the frame is published while its system runs.
            let all = unsafe { f.ctx() }
                .events_bytes(id)
                .map_err(|_| undeclared(&eng, f.system, "read", &event))?;
            (layout, all)
        } else {
            let (layout, id) = event_layout(&eng, &event)?;
            let all = eng.world()?.pending_events_bytes(id).map_err(err)?;
            (layout, all)
        };
        Ok(bytes.1.iter().map(|b| bytes.0.decode(b)).collect())
    })();
    Value::Array(list.unwrap_or_default()).to_string()
}

pub fn og_world_emit(world: u32, event: String, payload: String) -> Result<(), String> {
    if let Some(f) = running(world) {
        return (|| {
            let eng = engine(world)?;
            let (layout, id) = running_record(&eng, &event, RecordKind::Event)?;
            let declared = eng.state.borrow().systems[f.system].writes.contains(&event);
            if !declared {
                return Err(undeclared(&eng, f.system, "write", &event));
            }
            let bytes = layout.encode(None, Some(&parse(Some(&payload))?))?;
            // SAFETY: the frame is published while its system runs.
            unsafe { f.ctx() }
                .send_bytes(id, &bytes)
                .map_err(|_| undeclared(&eng, f.system, "write", &event))
        })();
    }
    (|| {
        let eng = engine(world)?;
        let (layout, id) = event_layout(&eng, &event)?;
        let bytes = layout.encode(None, Some(&parse(Some(&payload))?))?;
        eng.world()?.send_bytes(id, &bytes).map_err(err)
    })()
}

/// A ready asset `world.instantiate` places: its index and its scene
/// (none: a glTF).
fn instantiable(
    eng: &Engine,
    asset: f64,
) -> Result<(usize, Option<og_bundle::runtime::SceneDesc>), String> {
    let i = index_of(asset).ok_or_else(|| format!("{asset} is not an asset handle"))?;
    let a = eng.assets.borrow();
    let entry = a
        .entries
        .get(i)
        .ok_or_else(|| format!("asset {asset} is not alive"))?;
    match (&entry.status, &entry.payload) {
        (Status::Pending, _) => Err(format!(
            "{} is still loading; instantiate it once og.load.status says ready",
            entry.path
        )),
        (Status::Failed(e), _) => Err(e.clone()),
        (_, Payload::Scene(s)) => Ok((i, Some((**s).clone()))),
        (_, Payload::Gltf { .. }) => Ok((i, None)),
        _ => Err(format!("{} is a bundle, not a scene or a glTF", entry.path)),
    }
}

/// What follows an instance's root: the scene's entities under it, or the
/// glTF's og.Model on it and its lights. Refused, the root goes with what
/// spawned.
fn fill_instance(
    eng: &Engine,
    root: Entity,
    asset: f64,
    index: usize,
    scene: Option<&og_bundle::runtime::SceneDesc>,
) -> Result<(), String> {
    let filled = (|| match scene {
        Some(s) => assets::spawn_scene(eng, s, Some(root)).map(|_| ()),
        None => {
            let (layout, id) = component(eng, "og.Model")?;
            let fields = eng
                .assets
                .borrow()
                .model_shadows(index)
                .model_fields(asset as u64);
            let bytes = layout.encode(None, Some(&fields))?;
            before_structural(eng);
            eng.world()?.insert_bytes(root, id, &bytes).map_err(err)?;
            assets::spawn_lights(eng, index, root)
        }
    })();
    if filled.is_err() {
        let _ = og_world_despawn(eng.id, root.to_bits() as f64);
    }
    filled
}

/// An instance a system asked for, until its deferred placement runs: the
/// scene's names are held for it. Dropped unrun (the system failed, and its
/// commands were discarded with the root), the names are free again and the
/// root's own name is forgotten.
struct Unplaced {
    engine: u32,
    names: Vec<String>,
    /// The root, until it is placed.
    root: Option<Entity>,
}

impl Drop for Unplaced {
    fn drop(&mut self) {
        let Ok(eng) = engine(self.engine) else { return };
        let Ok(mut st) = eng.state.try_borrow_mut() else {
            return;
        };
        for n in &self.names {
            st.held_names.remove(n);
        }
        drop(st);
        if let Some(root) = self.root {
            forget_name(&eng, root);
        }
    }
}

/// `world.instantiate` inside a system. The root is spawned as
/// `world.spawn` spawns one there: queued, its id usable at once, with
/// og.Transform (and og.Parent, given a parent) among the system's writes.
/// What follows it is placed on the whole world when the system returns,
/// right after the root (og-ecs `SystemContext::defer`), so it needs no
/// writes of the system, and the systems after it this tick, and a
/// snapshot they take, have the scene. The scene's names are held from the
/// call on: a second instance of it in the same system is refused here,
/// as between ticks. What can still fail then (a sub-scene or a prefab
/// that does not load) takes the root with it and is a warning in the
/// engine log, since the call has answered.
fn instantiate_in_system(
    eng: &Engine,
    f: &super::engine::Frame,
    asset: f64,
    desc: &Value,
) -> Result<Entity, String> {
    let (index, scene) = instantiable(eng, asset)?;
    let names = match &scene {
        Some(s) => {
            let root = spawn_desc("world.instantiate", desc)?.name;
            assets::names_free(eng, s, &root)?
        }
        None => Vec::new(),
    };
    let root = spawn_in_system(eng, f, "world.instantiate", desc)?;
    eng.state
        .borrow_mut()
        .held_names
        .extend(names.iter().cloned());
    let mut unplaced = Unplaced {
        engine: eng.id,
        names,
        root: Some(root),
    };
    let id = eng.id;
    // SAFETY: the frame is published while its system runs.
    unsafe { f.ctx() }.defer(move |_| {
        // The root is in the world, and the names are the scene's to take.
        unplaced.root = None;
        drop(unplaced);
        // The step's borrow of the world is not used here: the scene goes
        // in through the engine, as it does between ticks.
        let Ok(eng) = engine(id) else { return Ok(()) };
        if let Err(e) = fill_instance(&eng, root, asset, index, scene.as_ref()) {
            let tick = eng.world().map_or(0, |w| w.time().tick);
            eng.state.borrow_mut().log.push(json!({
                "tick": tick, "warning": true,
                "text": format!("world.instantiate: {e}; its root (entity {}) is despawned", root.to_bits()),
            }));
        }
        // Into the world now, not at the next probe call: a snapshot a
        // later system takes this tick has the scene's names.
        rprobe::flush_names(&eng);
        Ok(())
    });
    Ok(root)
}

pub fn og_world_instantiate(
    world: u32,
    asset: f64,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    (|| {
        let eng = engine(world)?;
        let desc = parse(desc.as_deref())?;
        if let Some(f) = running(world) {
            return instantiate_in_system(&eng, f, asset, &desc).map(entity_number);
        }
        let (index, scene) = instantiable(&eng, asset)?;
        let root = spawn(&eng, "world.instantiate", &desc)?;
        fill_instance(&eng, root, asset, index, scene.as_ref())?;
        Ok(entity_number(root))
    })()
}

// ── query and views ─────────────────────────────────────────────────────────

/// Writes the archetypes of a query handle, from its `from`th, to the
/// shared buffer at word `word`, and answers how many it wrote. The words:
/// how many archetypes the query has, how many follow, their tick and the
/// layout epoch; then per archetype its view handle, its entity count, how
/// many types it has and their schema id indices. `from` 0 makes a world
/// query's views; a later `from` goes on through the views it made.
pub(crate) fn describe_query(query: u32, from: u32, word: usize) -> u32 {
    let range = HANDLES.with(|t| {
        let t = t.borrow();
        if let Some((_, QueryRec::Frame { first, len })) = t.frame_queries.get(query) {
            return Some(Ok((*first, *len)));
        }
        match t.world_queries.get(query) {
            Some((_, QueryRec::World { views, .. })) if from > 0 => Some(Ok(*views)),
            Some((e, QueryRec::World { desc, .. })) => Some(Err((*e, desc.clone()))),
            _ => None,
        }
    });
    let (first, len) = match range {
        None => (0, 0),
        Some(Ok(views)) => views,
        Some(Err((e, desc))) => {
            let made = engine(e)
                .and_then(|eng| world_query_views(&eng, &desc))
                .unwrap_or((0, 0));
            HANDLES.with(|t| {
                if let Some((_, QueryRec::World { views, .. })) =
                    t.borrow_mut().world_queries.get_mut(query)
                {
                    *views = made;
                }
            });
            made
        }
    };
    HANDLES.with(|t| {
        let t = t.borrow();
        crate::shared::with(|b| {
            let mut w = crate::shared::Words::at(b, word);
            let head = w.index();
            // Retired views (a stale handle) describe as none.
            let live = t.views.get(first).map(|(_, r)| r.tick);
            w.put(if live.is_some() { len } else { 0 });
            w.put(0);
            w.put(live.unwrap_or(0));
            w.put(super::engine::layout_epoch());
            let mut written = 0;
            for h in (from..len).map(|k| first.wrapping_add(k)) {
                let Some((_, r)) = t.views.get(h) else { break };
                let (start, n) = r.types;
                if !w.fits(3 + n as usize) {
                    break;
                }
                w.put(h);
                w.put(r.count);
                w.put(n);
                for id in &t.view_types[start as usize..(start + n) as usize] {
                    w.put(*id);
                }
                written += 1;
            }
            w.set(head + 1, written);
            written
        })
    })
}

/// `og_engine_query_archetypes`.
pub fn og_engine_query_archetypes(query: u32, from: u32) -> u32 {
    describe_query(query, from, 0)
}

/// `og_engine_component_id`: the schema id index of a component this
/// world has registered, `u32::MAX` for any other name. Asking registers
/// nothing.
pub fn og_engine_component_id(world: u32, component: String) -> u32 {
    engine(world)
        .ok()
        .and_then(|eng| {
            let st = eng.state.borrow();
            st.layouts
                .get(&component)
                .filter(|l| l.kind == RecordKind::Component)?;
            st.names_by_id.iter().position(|n| *n == component)
        })
        .map_or(u32::MAX, |i| i as u32)
}

pub fn og_query_count(query: u32) -> u32 {
    let world = HANDLES.with(|t| {
        let t = t.borrow();
        if let Some((_, QueryRec::Frame { first, len })) = t.frame_queries.get(query) {
            let counts = (0..*len).filter_map(|k| t.views.get(first.wrapping_add(k)));
            return Err(counts.map(|(_, r)| r.count).sum());
        }
        match t.world_queries.get(query) {
            Some((e, QueryRec::World { desc, .. })) => Ok((*e, desc.clone())),
            _ => Err(0),
        }
    });
    match world {
        Err(count) => count,
        Ok((e, desc)) => engine(e)
            .and_then(|eng| Ok(eng.world()?.query(&desc).map_err(err)?.entity_count() as u32))
            .unwrap_or(0),
    }
}

fn view_field<T>(view: u32, f: impl FnOnce(&super::engine::ViewRecord) -> T) -> Option<T> {
    HANDLES.with(|t| t.borrow().views.get(view).map(|(_, r)| f(r)))
}

pub fn og_archetype_view_get_count(view: u32) -> u32 {
    view_field(view, |r| r.count).unwrap_or(0)
}

pub fn og_archetype_view_get_tick(view: u32) -> u32 {
    view_field(view, |r| r.tick).unwrap_or(0)
}

pub fn og_archetype_view_entity(view: u32, index: u32) -> f64 {
    view_field(view, |r| {
        if index >= r.count {
            return 0.0;
        }
        // SAFETY: the record is live.
        unsafe { (*r.entities.add(index as usize)).to_bits() as f64 }
    })
    .unwrap_or(0.0)
}

// ── system and game ─────────────────────────────────────────────────────────

pub fn og_system_define_system(name: String, desc: String, run: u32) -> u32 {
    let desc = parse(Some(&desc)).unwrap_or(Value::Null);
    declare_system(name, &desc, run)
}

fn system_field(system: u32, f: impl FnOnce(&super::engine::SystemDecl) -> Value) -> String {
    DECL.with(|d| {
        d.borrow()
            .systems
            .get(system as usize)
            .map(f)
            .unwrap_or(Value::Null)
            .to_string()
    })
}

pub fn og_system_get_name(system: u32) -> String {
    DECL.with(|d| {
        d.borrow()
            .systems
            .get(system as usize)
            .map(|s| s.name.clone())
            .unwrap_or_default()
    })
}

pub fn og_system_get_reads(system: u32) -> String {
    system_field(system, |s| {
        json!(s.reads.iter().map(|t| &t.name).collect::<Vec<_>>())
    })
}

pub fn og_system_get_writes(system: u32) -> String {
    system_field(system, |s| {
        json!(s.writes.iter().map(|t| &t.name).collect::<Vec<_>>())
    })
}

pub fn og_game_game(desc: Option<String>) -> u32 {
    let desc = parse(desc.as_deref()).unwrap_or(Value::Null);
    declare_game(&desc)
}

pub fn og_game_get_tick(game: u32) -> u32 {
    DECL.with(|d| d.borrow().games.get(game as usize).map_or(0, |g| g.tick))
}

pub fn og_game_get_systems(game: u32) -> String {
    DECL.with(|d| {
        json!(
            d.borrow()
                .games
                .get(game as usize)
                .map(|g| g.systems.clone())
                .unwrap_or_default()
        )
        .to_string()
    })
}
