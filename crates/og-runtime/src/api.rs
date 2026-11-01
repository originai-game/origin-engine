//! The binding symbols, host-neutral: each function is the body of the
//! trait method of the same name (wire arguments in, wire answer out).

use super::assets::{self, Payload, Status, index_of};
use super::codec::{Layout, RecordKind};
use super::engine::{
    CommandDecl, DECL, Engine, HANDLES, QueryRec, before_structural, declare_game, declare_system,
    engine, queued_structural, running, world_query_views,
};
use super::probe::{self as rprobe, NameOp, with_probe};
use super::{ok, ok_void, parse, refuse};
use og_core::reflect::Reflect;
use og_core::{Rng, Time};
use og_ecs::{Entity, QueryDesc, World};
use serde_json::{Value, json};

mod actions;
mod commands;
mod controls;
mod loading;
mod render;

use super::{engine, environment, extract, feel, fx, look, modules, views};
pub use actions::*;
pub use commands::*;
use controls::vec3;
pub use controls::*;
pub use loading::*;
pub use render::*;

fn err(e: og_core::Error) -> String {
    e.reason().to_string()
}

fn result(r: Result<Value, String>) -> String {
    match r {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

fn result_void(r: Result<(), String>) -> String {
    match r {
        Ok(()) => ok_void(),
        Err(e) => refuse(e),
    }
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

fn forget_name(eng: &Engine, e: Entity) {
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

fn spawn(eng: &Engine, call: &str, desc: &Value) -> Result<Entity, String> {
    let name = desc["name"].as_str().unwrap_or_default().to_string();
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
    if !desc["bare"].as_bool().unwrap_or(false) {
        let (layout, id) = component(eng, "og.Transform")?;
        let mut value = serde_json::Map::new();
        for key in ["position", "quaternion", "scale"] {
            if let Some(v) = desc.get(key).filter(|v| !v.is_null()) {
                value.insert(key.to_string(), v.clone());
            }
        }
        components.push((id, layout.encode(None, Some(&Value::Object(value)))?));
    }
    if let Some(p) = spawn_parent(call, desc)? {
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
    eng.state.borrow_mut().game_spawned.insert(e.to_bits());
    Ok(e)
}

/// `world.spawn` inside a system: queued, applied when the system returns
/// (its id is usable at once); every component it gets must be among the
/// system's writes.
fn spawn_in_system(eng: &Engine, f: &super::engine::Frame, desc: &Value) -> Result<Entity, String> {
    let name = desc["name"].as_str().unwrap_or_default().to_string();
    if !name.is_empty() && eng.state.borrow().names.contains_key(&name) {
        return Err(format!("an entity named '{name}' already exists"));
    }
    let mut components = Vec::new();
    if !desc["bare"].as_bool().unwrap_or(false) {
        let (layout, id) = running_record(eng, "og.Transform", RecordKind::Component)?;
        let mut value = serde_json::Map::new();
        for key in ["position", "quaternion", "scale"] {
            if let Some(v) = desc.get(key).filter(|v| !v.is_null()) {
                value.insert(key.to_string(), v.clone());
            }
        }
        components.push((id, layout.encode(None, Some(&Value::Object(value)))?));
    }
    if let Some(p) = spawn_parent("world.spawn", desc)? {
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
                Some("world.spawn with parent writes og.Parent; add og.Parent to the system's writes")
            } else if reason.ends_with("access to 'og.Transform'") {
                Some("world.spawn writes og.Transform unless the desc says bare: true; add og.Transform to the system's writes")
            } else {
                None
            };
            match cause {
                Some(c) => format!("{reason} ({c})"),
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
    eng.state.borrow_mut().game_spawned.insert(e.to_bits());
    Ok(e)
}

pub fn og_world_spawn(world: u32, desc: Option<String>) -> String {
    if let Some(f) = running(world) {
        return result((|| {
            let eng = engine(world)?;
            let desc = parse(desc.as_deref())?;
            spawn_in_system(&eng, f, &desc).map(entity_number)
        })());
    }
    result((|| {
        let eng = engine(world)?;
        let desc = parse(desc.as_deref())?;
        spawn(&eng, "world.spawn", &desc).map(entity_number)
    })())
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

pub fn og_world_despawn(world: u32, entity_id: f64) -> String {
    result_void((|| {
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
        }
        Ok(())
    })())
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
        return HANDLES.with(|t| {
            let t = t.borrow();
            t.views.items.iter().any(|(owner, rec)| {
                *owner == world && rec.types.contains(&comp) && {
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

fn in_system(what: &str) -> String {
    format!(
        "{what} inside a system needs og-ecs SystemContext packed-byte access (requested from Core); do it in a command, or through the system's views"
    )
}

pub fn og_world_insert(world: u32, entity_id: f64, comp: String, value: Option<String>) -> String {
    if let Some(f) = running(world) {
        return result_void((|| {
            let eng = engine(world)?;
            // SAFETY: the frame is published while its system runs.
            let e = live(&eng, "world.insert", entity_id, |e| {
                Ok(unsafe { f.ctx() }.is_alive(e))
            })?;
            let (layout, id) = running_record(&eng, &comp, RecordKind::Component)?;
            let bytes = layout.encode(None, Some(&parse(value.as_deref())?))?;
            queued_structural();
            // SAFETY: the frame is published while its system runs.
            unsafe { f.ctx() }.insert_bytes(e, id, &bytes).map_err(err)
        })());
    }
    result_void((|| {
        let eng = engine(world)?;
        let e = live(&eng, "world.insert", entity_id, |e| {
            Ok(eng.world()?.is_alive(e))
        })?;
        let (layout, id) = component(&eng, &comp)?;
        let value = parse(value.as_deref())?;
        let bytes = layout.encode(None, Some(&value))?;
        before_structural(&eng);
        eng.world()?.insert_bytes(e, id, &bytes).map_err(err)
    })())
}

pub fn og_world_remove(world: u32, entity_id: f64, comp: String) -> String {
    if let Some(f) = running(world) {
        return result_void((|| {
            let eng = engine(world)?;
            // SAFETY: the frame is published while its system runs.
            let e = live(&eng, "world.remove", entity_id, |e| {
                Ok(unsafe { f.ctx() }.is_alive(e))
            })?;
            let (_, id) = running_record(&eng, &comp, RecordKind::Component)?;
            queued_structural();
            // SAFETY: the frame is published while its system runs.
            unsafe { f.ctx() }.remove_id(e, id).map_err(err)
        })());
    }
    result_void((|| {
        let eng = engine(world)?;
        let e = live(&eng, "world.remove", entity_id, |e| {
            Ok(eng.world()?.is_alive(e))
        })?;
        let (_, id) = component(&eng, &comp)?;
        before_structural(&eng);
        eng.world()?.remove_id(e, id).map_err(err)
    })())
}

pub fn og_world_get(world: u32, entity_id: f64, comp: String) -> String {
    if let Some(f) = running(world) {
        return result((|| {
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
        })());
    }
    result((|| {
        let eng = engine(world)?;
        let e = live(&eng, "world.get", entity_id, |e| {
            Ok(eng.world()?.is_alive(e))
        })?;
        let (layout, id) = component(&eng, &comp)?;
        let bytes = eng.world()?.get_bytes(e, id).map_err(err)?;
        Ok(layout.decode(&bytes))
    })())
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

pub fn og_world_set(world: u32, entity_id: f64, comp: String, value: String) -> String {
    if let Some(f) = running(world) {
        return result_void((|| {
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
        })());
    }
    result_void((|| {
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
    })())
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
    let id = w.id_by_name(name).map_err(err)?;
    let bytes = w.resource_bytes(id).map_err(err)?;
    Ok(layout.decode(&bytes))
}

pub fn og_world_resource(world: u32, res: String) -> String {
    result(engine(world).and_then(|eng| resource_value(&eng, &res)))
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

pub fn og_world_query(world: u32, desc: Option<String>) -> String {
    if running(world).is_some() {
        return refuse(
            "a system receives its query as its first argument; world.query is for commands and callbacks",
        );
    }
    result((|| {
        let eng = engine(world)?;
        let desc = parse(desc.as_deref())?;
        let names = |v: &Value| -> Vec<String> {
            v.as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|n| n.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default()
        };
        let q = QueryDesc {
            read: Vec::new(),
            write: names(&desc["with"]),
            without: names(&desc["without"]),
        };
        for n in q.write.iter().chain(&q.without) {
            component(&eng, n)?;
        }
        let h = HANDLES.with(|t| {
            t.borrow_mut()
                .world_queries
                .push((eng.id, QueryRec::World(q)))
        });
        Ok(json!(h))
    })())
}

fn event_layout(eng: &Engine, name: &str) -> Result<(Layout, og_ecs::SchemaId), String> {
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

pub fn og_world_emit(world: u32, event: String, payload: String) -> String {
    if let Some(f) = running(world) {
        return result_void((|| {
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
        })());
    }
    result_void((|| {
        let eng = engine(world)?;
        let (layout, id) = event_layout(&eng, &event)?;
        let bytes = layout.encode(None, Some(&parse(Some(&payload))?))?;
        eng.world()?.send_bytes(id, &bytes).map_err(err)
    })())
}

pub fn og_world_instantiate(world: u32, asset: f64, desc: Option<String>) -> String {
    if running(world).is_some() {
        return refuse(in_system("world.instantiate"));
    }
    result((|| {
        let eng = engine(world)?;
        let desc = parse(desc.as_deref())?;
        let i = index_of(asset).ok_or_else(|| format!("{asset} is not an asset handle"))?;
        let scene = {
            let a = eng.assets.borrow();
            let entry = a
                .entries
                .get(i)
                .ok_or_else(|| format!("asset {asset} is not alive"))?;
            match (&entry.status, &entry.payload) {
                (Status::Pending, _) => {
                    return Err(format!(
                        "{} is still loading; instantiate it once og.load.status says ready",
                        entry.path
                    ));
                }
                (Status::Failed(e), _) => return Err(e.clone()),
                (_, Payload::Scene(s)) => Some(s.clone()),
                (_, Payload::Gltf { .. }) => None,
                _ => return Err(format!("{} is a bundle, not a scene or a glTF", entry.path)),
            }
        };
        let root = spawn(&eng, "world.instantiate", &desc)?;
        match scene {
            Some(s) => {
                if let Err(e) = assets::spawn_scene(&eng, &s, Some(root)) {
                    let _ = og_world_despawn(world, root.to_bits() as f64);
                    return Err(e);
                }
            }
            None => {
                let (layout, id) = component(&eng, "og.Model")?;
                let fields = eng
                    .assets
                    .borrow()
                    .model_shadows(i)
                    .model_fields(asset as u64);
                let bytes = layout.encode(None, Some(&fields))?;
                before_structural(&eng);
                eng.world()?.insert_bytes(root, id, &bytes).map_err(err)?;
                if let Err(e) = assets::spawn_lights(&eng, i, root) {
                    let _ = og_world_despawn(world, root.to_bits() as f64);
                    return Err(e);
                }
            }
        }
        Ok(entity_number(root))
    })())
}

// ── query and views ─────────────────────────────────────────────────────────

/// `[{ view, count, tick, types }]` of a query handle.
pub fn og_query_archetypes(query: u32) -> String {
    enum Found {
        Frame(Vec<u32>),
        World(u32, QueryDesc),
    }
    let found = HANDLES.with(|t| {
        let t = t.borrow();
        if let Some((e, QueryRec::Frame(views))) = t.frame_queries.get(query) {
            let _ = e;
            return Some(Found::Frame(views.clone()));
        }
        if let Some((e, QueryRec::World(q))) = t.world_queries.get(query) {
            return Some(Found::World(*e, q.clone()));
        }
        None
    });
    let views = match found {
        None => return "[]".into(),
        Some(Found::Frame(v)) => v,
        Some(Found::World(e, q)) => match engine(e).and_then(|eng| world_query_views(&eng, &q)) {
            Ok(v) => v,
            Err(_) => return "[]".into(),
        },
    };
    HANDLES.with(|t| {
        let t = t.borrow();
        let list: Vec<Value> = views
            .iter()
            .filter_map(|h| {
                let (_, r) = t.views.get(*h)?;
                Some(json!({ "view": h, "count": r.count, "tick": r.tick, "types": r.types }))
            })
            .collect();
        Value::Array(list).to_string()
    })
}

pub fn og_query_count(query: u32) -> u32 {
    let list: Value = serde_json::from_str(&og_query_archetypes(query)).unwrap_or(json!([]));
    list.as_array()
        .map(|a| a.iter().filter_map(|v| v["count"].as_u64()).sum::<u64>() as u32)
        .unwrap_or(0)
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

pub fn og_archetype_view_has(view: u32, comp: String) -> bool {
    view_field(view, |r| r.types.contains(&comp)).unwrap_or(false)
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
