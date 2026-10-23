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

fn entity_number(e: Entity) -> Value {
    json!(e.to_bits())
}

/// A component layout by schema name, or why the token is not usable.
fn component(eng: &Engine, name: &str) -> Result<(Layout, og_ecs::SchemaId), String> {
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
    let mut st = eng.state.borrow_mut();
    st.names.insert(name.to_string(), e.to_bits());
    st.name_of.insert(e.to_bits(), name.to_string());
    st.name_ops.push(NameOp::Set(e, name.to_string()));
}

fn forget_name(eng: &Engine, e: Entity) {
    let mut st = eng.state.borrow_mut();
    if let Some(name) = st.name_of.remove(&e.to_bits()) {
        st.names.remove(&name);
        st.name_ops.push(NameOp::Remove(name));
    }
}

fn spawn(eng: &Engine, desc: &Value) -> Result<Entity, String> {
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
    if let Some(p) = desc.get("parent").and_then(Value::as_f64) {
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
    if let Some(p) = desc.get("parent").and_then(Value::as_f64) {
        let (layout, id) = running_record(eng, "og.Parent", RecordKind::Component)?;
        components.push((id, layout.encode(None, Some(&json!({ "entity": p })))?));
    }
    queued_structural();
    // SAFETY: the frame is published while its system runs.
    let e = unsafe { f.ctx() }.spawn_bytes(components).map_err(err)?;
    if !name.is_empty() {
        set_name(eng, e, &name);
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
        spawn(&eng, &desc).map(entity_number)
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
        let e = entity(entity_id)?;
        if let Some(f) = running(world) {
            queued_structural();
            // SAFETY: the frame is published while its system runs.
            return unsafe { f.ctx() }.despawn(e).map_err(err);
        }
        let eng = engine(world)?;
        let w = eng.world()?;
        if !w.is_alive(e) {
            return Err(format!("entity {e} is not alive"));
        }
        let all = with_descendants(w, e)?;
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
            let e = entity(entity_id)?;
            let (layout, id) = running_record(&eng, &comp, RecordKind::Component)?;
            let bytes = layout.encode(None, Some(&parse(value.as_deref())?))?;
            queued_structural();
            // SAFETY: the frame is published while its system runs.
            unsafe { f.ctx() }.insert_bytes(e, id, &bytes).map_err(err)
        })());
    }
    result_void((|| {
        let eng = engine(world)?;
        let e = entity(entity_id)?;
        let (layout, id) = component(&eng, &comp)?;
        let value = parse(value.as_deref())?;
        let w = eng.world()?;
        if !w.is_alive(e) {
            return Err(format!("entity {e} is not alive"));
        }
        let bytes = layout.encode(None, Some(&value))?;
        before_structural(&eng);
        eng.world()?.insert_bytes(e, id, &bytes).map_err(err)
    })())
}

pub fn og_world_remove(world: u32, entity_id: f64, comp: String) -> String {
    if let Some(f) = running(world) {
        return result_void((|| {
            let eng = engine(world)?;
            let e = entity(entity_id)?;
            let (_, id) = running_record(&eng, &comp, RecordKind::Component)?;
            queued_structural();
            // SAFETY: the frame is published while its system runs.
            unsafe { f.ctx() }.remove_id(e, id).map_err(err)
        })());
    }
    result_void((|| {
        let eng = engine(world)?;
        let e = entity(entity_id)?;
        let (_, id) = component(&eng, &comp)?;
        before_structural(&eng);
        eng.world()?.remove_id(e, id).map_err(err)
    })())
}

pub fn og_world_get(world: u32, entity_id: f64, comp: String) -> String {
    if let Some(f) = running(world) {
        return result((|| {
            let eng = engine(world)?;
            let e = entity(entity_id)?;
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
        let e = entity(entity_id)?;
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
            let e = entity(entity_id)?;
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
        let e = entity(entity_id)?;
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
        if name == "og.Input" {
            return ctx
                .resource::<og_probe::Input>()
                .map(|i| input_json(&i))
                .map_err(err);
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
    if name == "og.Input" {
        return w
            .resource::<og_probe::Input>()
            .map(|i| input_json(&i))
            .map_err(err);
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
        let root = spawn(&eng, &desc)?;
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

// ── transform ───────────────────────────────────────────────────────────────

fn vec3(v: &Value) -> glam::Vec3 {
    glam::Vec3::new(
        v["x"].as_f64().unwrap_or(0.0) as f32,
        v["y"].as_f64().unwrap_or(0.0) as f32,
        v["z"].as_f64().unwrap_or(0.0) as f32,
    )
}

fn quat(v: &Value) -> glam::Quat {
    glam::Quat::from_xyzw(
        v["x"].as_f64().unwrap_or(0.0) as f32,
        v["y"].as_f64().unwrap_or(0.0) as f32,
        v["z"].as_f64().unwrap_or(0.0) as f32,
        v["w"].as_f64().unwrap_or(1.0) as f32,
    )
}

fn vec3_json(v: glam::Vec3) -> Value {
    json!({ "x": f64::from(v.x), "y": f64::from(v.y), "z": f64::from(v.z) })
}

fn quat_json(q: glam::Quat) -> Value {
    json!({ "x": f64::from(q.x), "y": f64::from(q.y), "z": f64::from(q.z), "w": f64::from(q.w) })
}

pub fn og_transform_compose(position: String, quaternion: String, scale: String) -> String {
    let p = vec3(&parse(Some(&position)).unwrap_or_default());
    let q = quat(&parse(Some(&quaternion)).unwrap_or_default());
    let s = vec3(&parse(Some(&scale)).unwrap_or_default());
    let m = glam::Mat4::from_scale_rotation_translation(s, q, p);
    let elements: Vec<f64> = m.to_cols_array().iter().map(|&e| f64::from(e)).collect();
    json!({ "elements": elements }).to_string()
}

pub fn og_transform_decompose(matrix: String) -> String {
    let v = parse(Some(&matrix)).unwrap_or_default();
    let mut e = [0f32; 16];
    for (i, x) in e.iter_mut().enumerate() {
        *x = v["elements"][i]
            .as_f64()
            .unwrap_or(if i % 5 == 0 { 1.0 } else { 0.0 }) as f32;
    }
    let (s, q, p) = glam::Mat4::from_cols_array(&e).to_scale_rotation_translation();
    json!({ "position": vec3_json(p), "quaternion": quat_json(q), "scale": vec3_json(s) })
        .to_string()
}

/// The rotation that points -Z from `eye` towards `target`, +Y up by default.
fn look_rotation(eye: glam::Vec3, target: glam::Vec3, up: glam::Vec3) -> glam::Quat {
    let up = if up.length_squared() == 0.0 {
        glam::Vec3::Y
    } else {
        up
    };
    let forward = (target - eye).normalize_or_zero();
    if forward == glam::Vec3::ZERO {
        return glam::Quat::IDENTITY;
    }
    let z = -forward;
    let x = up.cross(z).normalize_or(glam::Vec3::X);
    let y = z.cross(x);
    glam::Quat::from_mat3(&glam::Mat3::from_cols(x, y, z))
}

pub fn og_transform_look_at(eye: String, target: String, up: Option<String>) -> String {
    let eye = vec3(&parse(Some(&eye)).unwrap_or_default());
    let target = vec3(&parse(Some(&target)).unwrap_or_default());
    let up = vec3(&parse(up.as_deref()).unwrap_or_default());
    quat_json(look_rotation(eye, target, up)).to_string()
}

pub fn og_transform_from_axis_angle(axis: String, angle: f64) -> String {
    let axis = vec3(&parse(Some(&axis)).unwrap_or_default());
    quat_json(og_core::math::quat_from_axis_angle(axis, angle as f32)).to_string()
}

// ── rng ─────────────────────────────────────────────────────────────────────

/// Draws from the world's Rng resource. Inside a system the resource must
/// be among its writes (a draw moves its state); otherwise the draw is
/// refused, logged once (`undeclared`) and answers the refused value (NaN).
fn with_rng<T>(world: u32, refused: T, f: impl FnOnce(&mut Rng) -> T) -> T {
    if let Some(frame) = running(world) {
        // SAFETY: the frame is published while its system runs.
        let ctx = unsafe { frame.ctx() };
        return match ctx.resource_mut::<Rng>() {
            Ok(mut r) => f(&mut r),
            Err(_) => {
                if let Ok(eng) = engine(world) {
                    undeclared(&eng, frame.system, "write", Rng::NAME);
                }
                refused
            }
        };
    }
    match engine(world).and_then(|e| {
        e.world()
            .and_then(|w| w.resource_mut::<Rng>().map_err(err))
            .map(f)
    }) {
        Ok(v) => v,
        Err(_) => refused,
    }
}

/// Reads the world's Rng resource without moving it: a fork derives a
/// stream from the seed alone, so a system with og.Rng among its reads may
/// fork (the parent is untouched and the child is the same on every host).
fn read_rng<T>(world: u32, f: impl FnOnce(&Rng) -> T) -> Option<T> {
    if let Some(frame) = running(world) {
        // SAFETY: the frame is published while its system runs.
        let ctx = unsafe { frame.ctx() };
        return match ctx.resource::<Rng>() {
            Ok(r) => Some(f(&r)),
            Err(_) => {
                if let Ok(eng) = engine(world) {
                    undeclared(&eng, frame.system, "read", Rng::NAME);
                }
                None
            }
        };
    }
    engine(world)
        .and_then(|e| {
            e.world()
                .and_then(|w| w.resource::<Rng>().map_err(err))
                .map(|r| f(&r))
        })
        .ok()
}

fn with_stream<T>(stream: u32, refused: T, f: impl FnOnce(&mut Rng) -> T) -> T {
    HANDLES.with(|t| {
        let mut t = t.borrow_mut();
        let base = t.streams.base;
        match stream
            .checked_sub(base)
            .and_then(|i| t.streams.items.get_mut(i as usize))
        {
            Some((_, r)) => f(r),
            None => refused,
        }
    })
}

fn chance(r: &mut Rng, p: f64) -> bool {
    r.next_f64() < p
}

fn integer(r: &mut Rng, low: i32, high: i32) -> i32 {
    if high < low {
        return low;
    }
    r.integer(i64::from(low), i64::from(high)) as i32
}

fn new_stream(world: u32, rng: Rng) -> u32 {
    HANDLES.with(|t| t.borrow_mut().streams.push((world, rng)))
}

pub fn og_rng_stream_get_seed(stream: u32) -> u32 {
    with_stream(stream, 0, |r| r.seed())
}
pub fn og_rng_stream_next(stream: u32) -> f64 {
    with_stream(stream, f64::NAN, Rng::next_f64)
}
pub fn og_rng_stream_range(stream: u32, low: f64, high: f64) -> f64 {
    with_stream(stream, f64::NAN, |r| r.range(low, high))
}
pub fn og_rng_stream_integer(stream: u32, low: i32, high: i32) -> i32 {
    with_stream(stream, 0, |r| integer(r, low, high))
}
pub fn og_rng_stream_chance(stream: u32, p: f64) -> bool {
    with_stream(stream, false, |r| chance(r, p))
}
pub fn og_rng_stream_fork(stream: u32, label: String) -> u32 {
    let owner = HANDLES.with(|t| {
        let t = t.borrow();
        t.streams.get(stream).map(|(w, r)| (*w, r.fork(&label)))
    });
    match owner {
        Some((w, r)) => new_stream(w, r),
        None => 0,
    }
}
pub fn og_rng_next(world: u32) -> f64 {
    with_rng(world, f64::NAN, Rng::next_f64)
}
pub fn og_rng_range(world: u32, low: f64, high: f64) -> f64 {
    with_rng(world, f64::NAN, |r| r.range(low, high))
}
pub fn og_rng_integer(world: u32, low: i32, high: i32) -> i32 {
    with_rng(world, 0, |r| integer(r, low, high))
}
pub fn og_rng_chance(world: u32, p: f64) -> bool {
    with_rng(world, false, |r| chance(r, p))
}
pub fn og_rng_fork(world: u32, label: String) -> u32 {
    match read_rng(world, |r| r.fork(&label)) {
        Some(r) => new_stream(world, r),
        None => 0,
    }
}

// ── audio ───────────────────────────────────────────────────────────────────

/// `og.audio.outputLatency`: og.Audio's `outputLatency`, which the host's
/// sink reports between ticks (`set_audio_latency`). Inside a system og.Audio
/// must be among its reads or writes (refused and logged once otherwise: 0).
pub fn og_audio_output_latency(world: u32) -> f64 {
    if let Some(frame) = running(world) {
        // SAFETY: the frame is published while its system runs.
        return match unsafe { frame.ctx() }.resource::<og_audio::Audio>() {
            Ok(a) => a.output_latency,
            Err(_) => {
                if let Ok(eng) = engine(world) {
                    undeclared(&eng, frame.system, "read", "og.Audio");
                }
                0.0
            }
        };
    }
    engine(world)
        .and_then(|e| {
            e.world().and_then(|w| {
                w.resource::<og_audio::Audio>()
                    .map_err(err)
                    .map(|a| a.output_latency)
            })
        })
        .unwrap_or(0.0)
}

/// The host's sink reports its output latency (seconds from a sample's world
/// time to the speaker: the device's own and the buffering in front of it),
/// between ticks; it is host input, written to og.Audio like an input event.
pub fn set_audio_latency(engine_id: u32, seconds: f64) -> Result<(), String> {
    if running(engine_id).is_some() {
        return Err("the audio latency is host input: report it between ticks".into());
    }
    let seconds = if seconds.is_finite() {
        seconds.clamp(0.0, 10.0)
    } else {
        0.0
    };
    let eng = engine(engine_id)?;
    let w = eng.world()?;
    let a = w.resource_mut::<og_audio::Audio>().map_err(err)?;
    if a.output_latency != seconds {
        a.output_latency = seconds;
    }
    Ok(())
}

// ── input ───────────────────────────────────────────────────────────────────

/// The `og.Input` resource as of this tick (inside a system, through its
/// context; the resource is og-probe's input layer until og-input owns it).
fn input<T>(world: u32, f: impl FnOnce(&og_probe::Input) -> T, refused: T) -> T {
    if let Some(fr) = running(world) {
        // SAFETY: the frame is published while its system runs.
        return match unsafe { fr.ctx() }.resource::<og_probe::Input>() {
            Ok(i) => f(&i),
            Err(_) => refused,
        };
    }
    match engine(world).and_then(|eng| {
        eng.world()?
            .resource::<og_probe::Input>()
            .map(|i| *i)
            .map_err(err)
    }) {
        Ok(i) => f(&i),
        Err(_) => refused,
    }
}

fn button(name: &str) -> og_probe::input::PointerButton {
    use og_probe::input::PointerButton as B;
    match name {
        "left" => B::Left,
        "middle" => B::Middle,
        "right" => B::Right,
        _ => B::None,
    }
}

fn input_json(i: &og_probe::Input) -> Value {
    json!({
        "pointer": { "x": i.pointer.x, "y": i.pointer.y },
        "pointerInside": i.pointer_inside,
        "wheel": { "x": i.wheel.x, "y": i.wheel.y },
    })
}

pub fn og_input_down(world: u32, key: String) -> bool {
    input(world, |i| i.down(&key), false)
}
pub fn og_input_pressed(world: u32, key: String) -> bool {
    input(world, |i| i.pressed(&key), false)
}
pub fn og_input_released(world: u32, key: String) -> bool {
    input(world, |i| i.released(&key), false)
}
pub fn og_input_axis(world: u32, negative: String, positive: String) -> f64 {
    input(world, |i| f64::from(i.axis(&negative, &positive)), 0.0)
}
pub fn og_input_axis2(world: u32, left: String, right: String, down: String, up: String) -> String {
    let v = input(
        world,
        |i| i.axis2(&left, &right, &down, &up),
        og_core::glam::Vec2::ZERO,
    );
    json!({ "x": f64::from(v.x), "y": f64::from(v.y) }).to_string()
}
pub fn og_input_button_down(world: u32, b: String) -> bool {
    input(world, |i| i.button_down(button(&b)), false)
}
pub fn og_input_button_pressed(world: u32, b: String) -> bool {
    input(world, |i| i.button_pressed(button(&b)), false)
}
pub fn og_input_button_released(world: u32, b: String) -> bool {
    input(world, |i| i.button_released(button(&b)), false)
}

/// A copy of an input resource as of this tick (inside a system, through
/// its context, which refuses it unless the system reads it).
fn input_res<R: og_core::Reflect + Copy + Send + Sync + 'static, T>(
    world: u32,
    f: impl FnOnce(&R) -> T,
    refused: T,
) -> T {
    if let Some(fr) = running(world) {
        // SAFETY: the frame is published while its system runs.
        return match unsafe { fr.ctx() }.resource::<R>() {
            Ok(r) => f(&r),
            Err(_) => refused,
        };
    }
    match engine(world).and_then(|eng| eng.world()?.resource::<R>().map(|r| *r).map_err(err)) {
        Ok(r) => f(&r),
        Err(_) => refused,
    }
}

fn vec2_json(v: og_core::glam::Vec2) -> String {
    json!({ "x": f64::from(v.x), "y": f64::from(v.y) }).to_string()
}

pub fn og_pointers_down(world: u32, id: u32) -> bool {
    input_res(world, |p: &og_input::Pointers| p.down(id), false)
}
pub fn og_pointers_pressed(world: u32, id: u32) -> bool {
    input_res(world, |p: &og_input::Pointers| p.is_pressed(id), false)
}
pub fn og_pointers_released(world: u32, id: u32) -> bool {
    input_res(world, |p: &og_input::Pointers| p.is_released(id), false)
}
pub fn og_pointers_position(world: u32, id: u32) -> String {
    vec2_json(input_res(
        world,
        |p: &og_input::Pointers| p.position(id),
        Default::default(),
    ))
}
pub fn og_pointers_start(world: u32, id: u32) -> String {
    vec2_json(input_res(
        world,
        |p: &og_input::Pointers| p.start(id),
        Default::default(),
    ))
}
pub fn og_pointers_count(world: u32) -> u32 {
    input_res(world, |p: &og_input::Pointers| p.count(), 0)
}
pub fn og_gamepad_connected(world: u32, pad: Option<u32>) -> bool {
    input_res(
        world,
        |g: &og_input::Gamepad| g.is_connected(pad.unwrap_or(0)),
        false,
    )
}
fn pad_button(button: &str) -> Option<usize> {
    og_input::gamepad_button(button)
}
pub fn og_gamepad_down(world: u32, button: String, pad: Option<u32>) -> bool {
    pad_button(&button).is_some_and(|b| {
        input_res(
            world,
            |g: &og_input::Gamepad| g.down(b, pad.unwrap_or(0)),
            false,
        )
    })
}
pub fn og_gamepad_pressed(world: u32, button: String, pad: Option<u32>) -> bool {
    pad_button(&button).is_some_and(|b| {
        input_res(
            world,
            |g: &og_input::Gamepad| g.pressed(b, pad.unwrap_or(0)),
            false,
        )
    })
}
pub fn og_gamepad_released(world: u32, button: String, pad: Option<u32>) -> bool {
    pad_button(&button).is_some_and(|b| {
        input_res(
            world,
            |g: &og_input::Gamepad| g.released(b, pad.unwrap_or(0)),
            false,
        )
    })
}
pub fn og_gamepad_axis(world: u32, axis: String, pad: Option<u32>) -> f64 {
    match og_input::GamepadAxis::from_name(&axis) {
        Some(a) => input_res(
            world,
            |g: &og_input::Gamepad| f64::from(g.axis(a, pad.unwrap_or(0))),
            0.0,
        ),
        None => 0.0,
    }
}

// ── camera ──────────────────────────────────────────────────────────────────

/// The play surface in CSS pixels (the unit of og.Input and the probe).
fn surface_size(eng: &Engine) -> (f32, f32) {
    let v = super::modules::viewport_of(eng.id);
    (v.width.max(1.0), v.height.max(1.0))
}

pub fn og_camera_look_at(world: u32, target: String) {
    let _ = (|| -> Result<(), String> {
        let eng = engine(world)?;
        let cam = resource_value(&eng, "og.Camera")?;
        let q = look_rotation(
            vec3(&cam["position"]),
            vec3(&parse(Some(&target))?),
            glam::Vec3::Y,
        );
        let value = json!({ "quaternion": quat_json(q) });
        let answer = super::engine::write_resource(world, "og.Camera", &value.to_string());
        if answer.contains("\"ok\":false") {
            return Err(answer);
        }
        Ok(())
    })();
}

fn view_proj(eng: &Engine) -> Result<(glam::Mat4, glam::Mat4, f32, f32), String> {
    let cam = resource_value(eng, "og.Camera")?;
    let (w, h) = surface_size(eng);
    let rot = quat(&cam["quaternion"]);
    let view = glam::Mat4::from_rotation_translation(rot, vec3(&cam["position"])).inverse();
    let near = cam["near"].as_f64().unwrap_or(0.1) as f32;
    let far = cam["far"].as_f64().unwrap_or(2000.0) as f32;
    let zoom = cam["zoom"].as_f64().unwrap_or(1.0).max(0.01) as f32;
    let proj = if cam["orthographic"].as_bool().unwrap_or(false) {
        let half_h = cam["height"].as_f64().unwrap_or(10.0) as f32 / 2.0 / zoom;
        let half_w = half_h * w / h;
        let r = 1.0 / (near - far);
        glam::Mat4::from_cols(
            glam::Vec4::new(1.0 / half_w, 0.0, 0.0, 0.0),
            glam::Vec4::new(0.0, 1.0 / half_h, 0.0, 0.0),
            glam::Vec4::new(0.0, 0.0, r, 0.0),
            glam::Vec4::new(0.0, 0.0, r * near, 1.0),
        )
    } else {
        // perspective_rh by hand: glam's calls the host libm, og_core::math does not.
        let half = (cam["fov"].as_f64().unwrap_or(50.0) as f32).to_radians() / 2.0;
        let (s, c) = og_core::math::sin_cos(half);
        let f = c / s * zoom;
        let r = far / (near - far);
        glam::Mat4::from_cols(
            glam::Vec4::new(f / (w / h), 0.0, 0.0, 0.0),
            glam::Vec4::new(0.0, f, 0.0, 0.0),
            glam::Vec4::new(0.0, 0.0, r, -1.0),
            glam::Vec4::new(0.0, 0.0, r * near, 0.0),
        )
    };
    Ok((proj * view, (proj * view).inverse(), w, h))
}

pub fn og_camera_world_to_screen(world: u32, point: String) -> String {
    let answer = (|| -> Result<Value, String> {
        let eng = engine(world)?;
        let (vp, _, w, h) = view_proj(&eng)?;
        let p = vp * vec3(&parse(Some(&point))?).extend(1.0);
        if p.w <= 0.0 {
            return Ok(Value::Null);
        }
        let ndc = p.truncate() / p.w;
        Ok(
            json!({ "x": f64::from((ndc.x + 1.0) / 2.0 * w), "y": f64::from((1.0 - ndc.y) / 2.0 * h) }),
        )
    })();
    answer.unwrap_or(Value::Null).to_string()
}

pub fn og_camera_screen_to_ray(world: u32, x: f64, y: f64) -> String {
    let answer = (|| -> Result<Value, String> {
        let eng = engine(world)?;
        let (_, inv, w, h) = view_proj(&eng)?;
        let nx = x as f32 / w * 2.0 - 1.0;
        let ny = 1.0 - y as f32 / h * 2.0;
        let near = inv.project_point3(glam::Vec3::new(nx, ny, 0.0));
        let far = inv.project_point3(glam::Vec3::new(nx, ny, 1.0));
        Ok(
            json!({ "origin": vec3_json(near), "direction": vec3_json((far - near).normalize_or_zero()) }),
        )
    })();
    answer
        .unwrap_or_else(|_| json!({ "origin": vec3_json(glam::Vec3::ZERO), "direction": vec3_json(glam::Vec3::NEG_Z) }))
        .to_string()
}

// ── render ──────────────────────────────────────────────────────────────────

/// A StandardMaterialDesc applied over `base` (fields the desc omits keep
/// their values): og-render's StandardMaterial carries every field under
/// the same name; texture fields are Texture handles and texture-array
/// fields TextureArray handles (null clears them).
pub(crate) fn material(
    base: og_render::StandardMaterial,
    desc: &Value,
    textures: &[og_render::TextureHandle],
    arrays: &[super::assets::LoadedArray],
) -> Result<og_render::StandardMaterial, String> {
    let mut m = base;
    let num = |k: &str| desc[k].as_f64().map(|v| v as f32);
    let flag = |k: &str| desc[k].as_bool().map(u32::from);
    let rgb = |k: &str| {
        let c = &desc[k];
        c.is_object().then(|| {
            glam::Vec3::new(
                c["r"].as_f64().unwrap_or(0.0) as f32,
                c["g"].as_f64().unwrap_or(0.0) as f32,
                c["b"].as_f64().unwrap_or(0.0) as f32,
            )
        })
    };
    let tex = |k: &str| -> Result<Option<u32>, String> {
        match desc.get(k) {
            None => Ok(None),
            Some(Value::Null) => Ok(Some(0)),
            Some(v) => {
                let t = v.as_f64().unwrap_or(-1.0);
                index_of(t)
                    .and_then(|i| textures.get(i))
                    .map(|h| Some(h.slot()))
                    .ok_or_else(|| format!("{k}: texture {t} is not alive"))
            }
        }
    };
    macro_rules! set {
        ($field:ident, $v:expr) => {
            if let Some(v) = $v {
                m.$field = v;
            }
        };
    }
    set!(color, rgb("color"));
    set!(map, tex("map")?);
    set!(metalness, num("metalness"));
    set!(metalnessMap, tex("metalnessMap")?);
    set!(roughness, num("roughness"));
    set!(roughnessMap, tex("roughnessMap")?);
    set!(normalMap, tex("normalMap")?);
    set!(aoMap, tex("aoMap")?);
    set!(emissive, rgb("emissive"));
    set!(emissiveIntensity, num("emissiveIntensity"));
    set!(emissiveMap, tex("emissiveMap")?);
    set!(opacity, num("opacity"));
    set!(transparent, flag("transparent"));
    set!(alphaTest, num("alphaTest"));
    set!(
        side,
        desc["side"].as_str().map(|s| match s {
            "back" => 1,
            "double" => 2,
            _ => 0,
        })
    );
    set!(flatShading, flag("flatShading"));
    set!(wireframe, flag("wireframe"));
    set!(envMapIntensity, num("envMapIntensity"));
    let vec2 = |k: &str, fallback: glam::Vec2| {
        let v = &desc[k];
        v.is_object().then(|| {
            glam::Vec2::new(
                v["x"].as_f64().map_or(fallback.x, |x| x as f32),
                v["y"].as_f64().map_or(fallback.y, |y| y as f32),
            )
        })
    };
    set!(normalScale, vec2("normalScale", glam::Vec2::ONE));
    set!(aoMapIntensity, num("aoMapIntensity"));
    set!(unlit, flag("unlit"));
    set!(styled, flag("styled"));
    set!(toon, flag("toon"));
    set!(gradientMap, tex("gradientMap")?);
    set!(rimColor, rgb("rimColor"));
    set!(rimIntensity, num("rimIntensity"));
    set!(rimPower, num("rimPower"));
    set!(flashColor, rgb("flashColor"));
    set!(flash, num("flash"));
    set!(dissolve, num("dissolve"));
    set!(dissolveMap, tex("dissolveMap")?);
    set!(dissolveEdgeColor, rgb("dissolveEdgeColor"));
    set!(dissolveEdgeWidth, num("dissolveEdgeWidth"));
    set!(dissolveScale, num("dissolveScale"));
    set!(outlineColor, rgb("outlineColor"));
    set!(outlineWidth, num("outlineWidth"));
    set!(clearcoat, num("clearcoat"));
    set!(clearcoatRoughness, num("clearcoatRoughness"));
    set!(clearcoatMap, tex("clearcoatMap")?);
    set!(clearcoatRoughnessMap, tex("clearcoatRoughnessMap")?);
    set!(clearcoatNormalMap, tex("clearcoatNormalMap")?);
    set!(
        clearcoatNormalScale,
        vec2("clearcoatNormalScale", glam::Vec2::ONE)
    );
    set!(sheen, num("sheen"));
    set!(sheenColor, rgb("sheenColor"));
    set!(sheenRoughness, num("sheenRoughness"));
    set!(sheenColorMap, tex("sheenColorMap")?);
    set!(sheenRoughnessMap, tex("sheenRoughnessMap")?);
    set!(transmission, num("transmission"));
    set!(transmissionMap, tex("transmissionMap")?);
    set!(thickness, num("thickness"));
    set!(thicknessMap, tex("thicknessMap")?);
    set!(attenuationDistance, num("attenuationDistance"));
    set!(attenuationColor, rgb("attenuationColor"));
    set!(ior, num("ior"));
    set!(iridescence, num("iridescence"));
    set!(iridescenceIOR, num("iridescenceIOR"));
    set!(
        iridescenceThicknessRange,
        vec2("iridescenceThicknessRange", glam::Vec2::new(100.0, 400.0))
    );
    set!(iridescenceMap, tex("iridescenceMap")?);
    set!(iridescenceThicknessMap, tex("iridescenceThicknessMap")?);
    set!(anisotropy, num("anisotropy"));
    set!(anisotropyRotation, num("anisotropyRotation"));
    set!(anisotropyMap, tex("anisotropyMap")?);
    let array = |k: &str| -> Result<Option<u32>, String> {
        match desc.get(k) {
            None => Ok(None),
            Some(Value::Null) => Ok(Some(0)),
            Some(v) => {
                let a = v.as_f64().unwrap_or(-1.0);
                let loaded = index_of(a)
                    .and_then(|i| arrays.get(i))
                    .ok_or_else(|| format!("{k}: texture array {a} is not loaded in this world"))?;
                let h = loaded.handle.ok_or_else(|| {
                    format!(
                        "{k}: {}'s streaming group is not resident (the array loads with it)",
                        loaded.id
                    )
                })?;
                Ok(Some(h.slot()))
            }
        }
    };
    let layer = |k: &str| desc[k].as_f64().map(|v| v.max(0.0) as u32);
    set!(mapArray, array("mapArray")?);
    set!(mapLayer, layer("mapLayer"));
    set!(normalMapArray, array("normalMapArray")?);
    set!(normalMapLayer, layer("normalMapLayer"));
    Ok(m)
}

/// Material handles are the render state's material index with generation 1.
pub(crate) fn material_handle(index: usize) -> f64 {
    assets::handle(index) as f64
}

pub fn og_render_standard_material(world: u32, desc: Option<String>) -> String {
    result((|| {
        let eng = engine(world)?;
        let desc = parse(desc.as_deref())?;
        // three.js MeshStandardMaterial defaults: white, metalness 0, roughness 1.
        let mut rs = eng.render.borrow_mut();
        let m = material(
            og_render::StandardMaterial::default(),
            &desc,
            &rs.textures,
            &rs.texture_arrays,
        )?;
        let handle = rs.scene.add_standard_material(m);
        rs.materials.push(handle);
        Ok(json!(material_handle(rs.materials.len() - 1)))
    })())
}

pub fn og_render_update_material(world: u32, handle: f64, desc: Option<String>) -> String {
    result_void((|| {
        let eng = engine(world)?;
        let desc = parse(desc.as_deref())?;
        let mut rs = eng.render.borrow_mut();
        let h = index_of(handle)
            .and_then(|i| rs.materials.get(i).copied())
            .ok_or_else(|| format!("material {handle} is not alive"))?;
        let current = *rs
            .scene
            .standard_material(h)
            .ok_or("material is not alive")?;
        let m = material(current, &desc, &rs.textures, &rs.texture_arrays)?;
        rs.scene.set_standard_material(h, m);
        Ok(())
    })())
}

pub fn og_render_capabilities(world: u32) -> String {
    let caps = engine(world).ok().and_then(|eng| {
        let rs = eng.render.borrow();
        rs.backend.as_ref().map(|b| {
            let tier = b.tier();
            let (w, h) = b.size();
            let info = b.info();
            json!({
                "tier": tier.clone(),
                "compute": tier != "webgl2",
                "antialias": b.antialias(),
                "maxLights": og_render::MAX_LIGHTS,
                "maxShadowedLights": info.as_ref().map_or(0, |i| i.shadowed_lights),
                "pointShadows": info.as_ref().map_or("cube", |i| crate::point_shadows_name(i.point_shadows)),
                "width": w, "height": h,
                "stylise": og_render::STYLISE_DRAWN,
                // SSR is the full tier's; WebGL2 reflects the probes and the environment.
                "ssr": tier != "webgl2",
                "reflectionProbes": og_render::MAX_REFLECTION_PROBES,
                "volumetricFog": if tier == "webgl2" { "raymarch" } else { "froxel" },
                "gpuTimestamps": b.gpu_timing_absent().is_none(),
            })
        })
        .map(|mut c| {
            if let (Value::Object(c), Value::Object(v)) = (&mut c, super::views::capabilities(&rs)) {
                c.extend(v);
            }
            c
        })
    });
    caps.unwrap_or_else(|| json!({ "tier": "none", "compute": false, "antialias": "off", "maxLights": 0, "maxShadowedLights": 0, "pointShadows": "cube", "width": 0, "height": 0, "stylise": [], "ssr": false, "reflectionProbes": 0, "renderTargetDepth": false, "maxRenderTargetSize": 0, "maxCustomPosts": 0, "volumetricFog": "raymarch", "gpuTimestamps": false }))
        .to_string()
}

pub fn og_render_text(world: u32, desc: Option<String>) -> String {
    result_void((|| {
        let eng = engine(world)?;
        let d = parse(desc.as_deref())?;
        let Some(font) = d["font"].as_f64() else {
            return Ok(());
        };
        let fi = index_of(font).ok_or_else(|| format!("{font} is not a font handle"))?;
        let render = eng
            .assets
            .borrow()
            .fonts
            .get(fi)
            .map(|f| f.render)
            .ok_or_else(|| format!("font {font} is not alive"))?;
        let tick = og_world_get_tick(world) as u64;
        let num = |v: &Value, k: &str, def: f64| v[k].as_f64().unwrap_or(def) as f32;
        let c = &d["color"];
        let mut rs = eng.render.borrow_mut();
        new_frame(&mut rs, tick);
        let text = d["text"].as_str().unwrap_or_default().to_string();
        let size = num(&d, "size", 1.0);
        let rotation = num(&d, "rotation", 0.0);
        let position = vec3(&d["position"]);
        let color = [
            num(c, "r", 1.0),
            num(c, "g", 1.0),
            num(c, "b", 1.0),
            num(&d, "opacity", 1.0),
        ];
        let layer = d["layer"].as_i64().unwrap_or(0) as i32;
        let screen = d["screen"].as_bool().unwrap_or(false);
        let clip = clip_of(&d["clip"]);
        let outline = num(&d, "outline", 0.0).clamp(0.0, 1.0);
        let oc = &d["outlineColor"];
        let outline_color = [
            num(oc, "r", 0.0),
            num(oc, "g", 0.0),
            num(oc, "b", 0.0),
            color[3],
        ];
        let shift = match d["align"].as_str() {
            Some("center") => 0.5,
            Some("end") => 1.0,
            _ => 0.0,
        };
        if shift == 0.0 {
            rs.texts.push(super::extract::QueuedText {
                font: render,
                text,
                position,
                size,
                rotation,
                color,
                layer,
                screen,
                clip,
                outline,
                outline_color,
            });
            return Ok(());
        }
        // Aligned: each line is placed on its own, moved left along the
        // text's baseline by its advance times `shift`.
        let (metrics, line_height) = font_metrics(&rs, render)?;
        let (sin, cos) = og_core::math::sin_cos(rotation);
        let along = glam::Vec3::new(cos, sin, 0.0);
        // The next line is below: -y in the world, +y on screen (y down,
        // rotation clockwise).
        let down = if screen {
            glam::Vec3::new(-sin, cos, 0.0)
        } else {
            glam::Vec3::new(sin, -cos, 0.0)
        };
        for (i, line) in text.split('\n').enumerate() {
            let w = metrics.measure(line, size);
            rs.texts.push(super::extract::QueuedText {
                font: render,
                text: line.to_string(),
                position: position - along * (w * shift) + down * (i as f32 * line_height * size),
                size,
                rotation,
                color,
                layer,
                screen,
                clip,
                outline,
                outline_color,
            });
        }
        Ok(())
    })())
}

/// Adds a generated geometry; `physics` also registers it as a physics mesh
/// under its handle.
pub(crate) fn add_geometry(
    world: u32,
    mesh: og_render::MeshData,
    physics: bool,
) -> Result<Value, String> {
    let eng = engine(world)?;
    let handle = {
        let mut rs = eng.render.borrow_mut();
        let h = rs.scene.add_mesh(mesh.clone());
        rs.geometries.push(h);
        assets::handle(rs.geometries.len() - 1)
    };
    if physics {
        super::modules::set_mesh(&eng, handle, super::modules::mesh_of(&mesh))?;
    }
    Ok(json!(handle as f64))
}

/// A geometry desc's `uvRepeat` applied (og_render::shapes::repeat_uv).
pub(crate) fn uv_repeat(d: &Value, mesh: &mut og_render::MeshData) -> Result<(), String> {
    let r = &d["uvRepeat"];
    let axis = |k: &str| -> Result<f32, String> {
        let v = r[k].as_f64().unwrap_or(1.0);
        if !v.is_finite() {
            return Err(format!("uvRepeat.{k} must be finite, got {v}"));
        }
        Ok(v as f32)
    };
    og_render::shapes::repeat_uv(mesh, glam::Vec2::new(axis("x")?, axis("y")?));
    Ok(())
}

fn size_of(d: &Value, key: &str, default: f64) -> Result<f32, String> {
    let v = d[key].as_f64().unwrap_or(default);
    if !(v.is_finite() && v >= 0.0) {
        return Err(format!("{key} must be finite and at least 0, got {v}"));
    }
    Ok(v as f32)
}

pub fn og_render_box_geometry(world: u32, desc: Option<String>) -> String {
    result((|| {
        let d = parse(desc.as_deref())?;
        let size = glam::Vec3::new(
            size_of(&d, "width", 1.0)?,
            size_of(&d, "height", 1.0)?,
            size_of(&d, "depth", 1.0)?,
        );
        let mut mesh = og_render::shapes::cuboid(size);
        uv_repeat(&d, &mut mesh)?;
        add_geometry(world, mesh, true)
    })())
}

pub fn og_render_plane_geometry(world: u32, desc: Option<String>) -> String {
    result((|| {
        let d = parse(desc.as_deref())?;
        let mut mesh =
            og_render::shapes::plane_xy(size_of(&d, "width", 1.0)?, size_of(&d, "height", 1.0)?);
        uv_repeat(&d, &mut mesh)?;
        add_geometry(world, mesh, true)
    })())
}

pub fn og_render_sphere_geometry(world: u32, desc: Option<String>) -> String {
    result((|| {
        let d = parse(desc.as_deref())?;
        let radius = size_of(&d, "radius", 1.0)?;
        let seg = d["widthSegments"].as_u64().unwrap_or(32).clamp(3, 512) as u32;
        let rings = d["heightSegments"].as_u64().unwrap_or(16).clamp(2, 512) as u32;
        let mut mesh = og_render::shapes::sphere(seg, rings);
        for v in &mut mesh.vertices {
            for c in &mut v.position {
                *c *= radius * 2.0;
            }
        }
        uv_repeat(&d, &mut mesh)?;
        add_geometry(world, mesh, false)
    })())
}

/// Starts the queued draws of a new tick.
fn new_frame(rs: &mut super::extract::RenderState, tick: u64) {
    if rs.texts_tick != tick {
        rs.texts.clear();
        rs.fx.sprites.clear();
        rs.fx.ribbons.clear();
        rs.texts_tick = tick;
    }
}

/// A `clip` field: `[x, y, width, height]`, or none.
fn clip_of(v: &Value) -> Option<[f32; 4]> {
    let a = v.as_array().filter(|a| a.len() == 4)?;
    let n = |i: usize| a[i].as_f64().unwrap_or(0.0) as f32;
    Some([n(0), n(1), n(2), n(3)])
}

/// A loaded font's layout metrics (its first page carries every advance)
/// and its line height in em.
fn font_metrics(
    rs: &super::extract::RenderState,
    render: usize,
) -> Result<(og_ui::FontMetrics, f32), String> {
    let first = rs
        .fonts
        .get(render)
        .and_then(|p| p.pages.first().copied())
        .ok_or("the font has no pages")?;
    let msdf = rs
        .scene
        .font(first)
        .ok_or("the font's first page is missing")?;
    Ok((og_ui::FontMetrics::from_msdf(msdf), msdf.line_height))
}

pub fn og_render_sprite(world: u32, desc: Option<String>) -> String {
    result_void((|| {
        let eng = engine(world)?;
        let d = parse(desc.as_deref())?;
        let num = |v: &Value, k: &str, def: f64| v[k].as_f64().unwrap_or(def) as f32;
        let tick = og_world_get_tick(world) as u64;
        let mut rs = eng.render.borrow_mut();
        let texture = match d["texture"].as_f64() {
            None => None,
            Some(t) => Some(
                index_of(t)
                    .and_then(|i| rs.textures.get(i).copied())
                    .ok_or_else(|| format!("texture {t} is not alive"))?,
            ),
        };
        // A layer of a texture array in place of the texture (world or screen).
        let array = match d["array"].as_f64() {
            None => None,
            Some(a) => {
                let loaded = index_of(a)
                    .and_then(|i| rs.texture_arrays.get(i))
                    .ok_or_else(|| format!("texture array {a} is not loaded in this world"))?;
                let handle = loaded.handle.ok_or_else(|| {
                    format!(
                        "{}: its streaming group is not resident (the array loads with it)",
                        loaded.id
                    )
                })?;
                let layer = d["arrayLayer"].as_f64().unwrap_or(0.0);
                if !(layer >= 0.0 && layer.fract() == 0.0 && layer < f64::from(loaded.layers)) {
                    return Err(format!(
                        "arrayLayer {layer}: {} has layers 0 to {}",
                        loaded.id,
                        loaded.layers - 1
                    ));
                }
                Some((handle, layer as u32, loaded.size))
            }
        };
        let uv = d["uv"]
            .as_array()
            .filter(|a| a.len() == 4)
            .map(|a| {
                let n = |i: usize| a[i].as_f64().unwrap_or(0.0) as f32;
                [n(0), n(1), n(2), n(3)]
            })
            .unwrap_or([0.0, 0.0, 1.0, 1.0]);
        let c = &d["color"];
        let anchor = &d["anchor"];
        new_frame(&mut rs, tick);
        let fc = &d["flashColor"];
        let flash = [
            if fc.is_object() {
                num(fc, "r", 1.0)
            } else {
                1.0
            },
            if fc.is_object() {
                num(fc, "g", 1.0)
            } else {
                1.0
            },
            if fc.is_object() {
                num(fc, "b", 1.0)
            } else {
                1.0
            },
            num(&d, "flash", 0.0).clamp(0.0, 1.0),
        ];
        let sprite = og_render::Sprite {
            texture,
            position: vec3(&d["position"]),
            size: glam::Vec2::new(num(&d, "width", 1.0), num(&d, "height", 1.0)),
            anchor: glam::Vec2::new(num(anchor, "x", 0.5), num(anchor, "y", 0.5)),
            rotation: num(&d, "rotation", 0.0),
            color: [
                num(c, "r", 1.0),
                num(c, "g", 1.0),
                num(c, "b", 1.0),
                num(&d, "opacity", 1.0),
            ],
            uv_rect: uv,
            layer: d["layer"].as_i64().unwrap_or(0) as i32,
            clip: None,
        };
        // Flips swap the region's edges; a nine-slice becomes its pieces.
        let mut sprite = sprite;
        if d["flipX"].as_bool().unwrap_or(false) {
            sprite.uv_rect.swap(0, 2);
        }
        if d["flipY"].as_bool().unwrap_or(false) {
            sprite.uv_rect.swap(1, 3);
        }
        let screen = d["screen"].as_bool().unwrap_or(false);
        let order = if d["sortY"].as_bool().unwrap_or(false) {
            if screen {
                sprite.position.y
            } else {
                -sprite.position.y
            }
        } else {
            0.0
        };
        let size = match array {
            Some((.., size)) => Some(size),
            None => texture.and_then(|t| rs.scene.texture_size(t)),
        };
        let pieces = match (d["slice"].as_array(), size) {
            (Some(a), Some(size)) if a.len() == 4 => {
                let n = |i: usize| a[i].as_f64().unwrap_or(0.0) as f32;
                og_render::nine_slice(
                    &sprite,
                    [n(0), n(1), n(2), n(3)],
                    size,
                    num(&d, "sliceScale", 0.0625),
                )
            }
            _ => vec![sprite],
        };
        for (i, sprite) in pieces.into_iter().enumerate() {
            rs.fx.sprites.push(super::fx::QueuedSprite {
                sprite,
                flash,
                glow: num(&d, "glow", 0.0).max(0.0),
                additive: d["blend"].as_str() == Some("additive"),
                cast_shadow: i == 0 && d["castShadow"].as_bool().unwrap_or(false),
                screen,
                clip: clip_of(&d["clip"]),
                order,
                array: array.map(|(h, layer, _)| (h, layer)),
                drop_shadow: num(&d, "dropShadow", 0.0).clamp(0.0, 1.0),
                drop_shadow_height: num(&d, "dropShadowHeight", 0.0),
                tone_mapped: d["toneMapped"].as_bool().unwrap_or(true),
            });
        }
        Ok(())
    })())
}

pub fn og_render_ribbon(world: u32, desc: Option<String>) -> String {
    result_void((|| {
        let eng = engine(world)?;
        let d = parse(desc.as_deref())?;
        let num = |v: &Value, k: &str, def: f64| v[k].as_f64().unwrap_or(def) as f32;
        let points: Vec<og_render::RibbonPoint> = d["points"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|p| {
                        let c = &p["color"];
                        let rgb = |k: &str| if c.is_object() { num(c, k, 1.0) } else { 1.0 };
                        og_render::RibbonPoint {
                            position: vec3(&p["position"]),
                            width: num(p, "width", 0.1),
                            color: [rgb("r"), rgb("g"), rgb("b"), num(p, "opacity", 1.0)],
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        if points.len() < 2 {
            return Err(format!(
                "a ribbon needs at least two points, got {}",
                points.len()
            ));
        }
        let tick = og_world_get_tick(world) as u64;
        let mut rs = eng.render.borrow_mut();
        let texture = match d["texture"].as_f64() {
            None => None,
            Some(t) => Some(
                index_of(t)
                    .and_then(|i| rs.textures.get(i).copied())
                    .ok_or_else(|| format!("texture {t} is not alive"))?,
            ),
        };
        let tile = num(&d, "uvTile", 0.0);
        new_frame(&mut rs, tick);
        rs.fx.ribbons.push(og_render::Ribbon {
            points,
            texture,
            uv: if tile > 0.0 {
                og_render::RibbonUv::Tile { length: tile }
            } else {
                og_render::RibbonUv::Stretch
            },
            blend: if d["blend"].as_str() == Some("alpha") {
                og_render::ParticleBlend::Alpha
            } else {
                og_render::ParticleBlend::Additive
            },
        });
        Ok(())
    })())
}

pub fn og_render_measure_text(world: u32, desc: Option<String>) -> String {
    result((|| {
        let eng = engine(world)?;
        let d = parse(desc.as_deref())?;
        let font = d["font"]
            .as_f64()
            .ok_or("TextMeasureDesc.font is null: load one with og.load.font")?;
        let fi = index_of(font).ok_or_else(|| format!("{font} is not a font handle"))?;
        let render = eng
            .assets
            .borrow()
            .fonts
            .get(fi)
            .map(|f| f.render)
            .ok_or_else(|| format!("font {font} is not alive"))?;
        let size = d["size"].as_f64().unwrap_or(1.0) as f32;
        let text = d["text"].as_str().unwrap_or_default();
        let rs = eng.render.borrow();
        let (metrics, line_height) = font_metrics(&rs, render)?;
        let lines: Vec<&str> = text.split('\n').collect();
        let width = lines
            .iter()
            .map(|l| metrics.measure(l, size))
            .fold(0.0f32, f32::max);
        Ok(
            json!({ "width": width, "height": lines.len() as f32 * line_height * size, "lines": lines.len() }),
        )
    })())
}

pub fn og_load_texture(world: u32, path: String) -> String {
    result(engine(world).and_then(|eng| assets::load_texture(&eng, &path).map(asset_value)))
}

// ── load ────────────────────────────────────────────────────────────────────

fn asset_value(i: usize) -> Value {
    json!(assets::handle(i) as f64)
}

pub fn og_load_bundle(world: u32, path: String) -> String {
    result((|| {
        let eng = engine(world)?;
        let i = eng.assets.borrow_mut().open_bundle(&path);
        Ok(asset_value(i))
    })())
}

pub fn og_load_bundle_info(world: u32, asset: f64) -> String {
    result((|| {
        let eng = engine(world)?;
        let a = eng.assets.borrow();
        let e = index_of(asset)
            .and_then(|i| a.entries.get(i))
            .ok_or_else(|| format!("asset {asset} is not alive"))?;
        match (&e.status, &e.payload) {
            (Status::Failed(r), _) => Err(r.clone()),
            (Status::Pending, _) => Err(format!("{} is still loading", e.path)),
            (_, Payload::Bundle { bundle, dir }) => {
                let m = bundle.manifest();
                Ok(json!({ "path": dir, "assets": m.assets.len(), "outputs": m.outputs().count() }))
            }
            _ => Err(format!("{} is not a bundle", e.path)),
        }
    })())
}

pub fn og_load_default_bundle(world: u32) -> String {
    result((|| {
        let eng = engine(world)?;
        let a = eng.assets.borrow();
        let i = a
            .default_bundle
            .ok_or("no cooked bundle: the host gave the engine no project root")?;
        match &a.entries[i].status {
            Status::Failed(r) => Err(r.clone()),
            _ => Ok(asset_value(i)),
        }
    })())
}

pub fn og_load_gltf(world: u32, path: String, desc: Option<String>) -> String {
    result((|| {
        let eng = engine(world)?;
        let shadows = assets::ModelShadows::from_desc(&parse(desc.as_deref())?)?;
        let index = assets::load_gltf(&eng, &path)?;
        // The flags belong to the asset: this load's, for the models
        // instantiated after it.
        eng.assets.borrow_mut().model_shadows.insert(index, shadows);
        Ok(asset_value(index))
    })())
}

pub fn og_load_scene(world: u32, path: String) -> String {
    result(engine(world).and_then(|eng| assets::load_scene(&eng, &path).map(asset_value)))
}

pub fn og_load_lut(world: u32, path: String) -> String {
    result(engine(world).and_then(|eng| super::fx::load_lut(&eng, &path).map(asset_value)))
}

pub fn og_load_look(world: u32, path: String) -> String {
    result(engine(world).and_then(|eng| super::look::load(&eng, &path).map(asset_value)))
}

pub fn og_load_look_info(world: u32, look: f64) -> String {
    result(engine(world).and_then(|eng| super::look::info(&eng, super::look::index(look)?)))
}

pub fn og_load_palette(world: u32, path: String) -> String {
    result(engine(world).and_then(|eng| super::look::load_palette(&eng, &path).map(asset_value)))
}

pub fn og_render_apply_look(world: u32, look: f64) -> String {
    result_void(engine(world).and_then(|eng| super::look::apply(&eng, super::look::index(look)?)))
}

pub fn og_render_look(world: u32) -> String {
    let info = engine(world).ok().and_then(|eng| {
        let applied = eng.render.borrow().fx.look.applied;
        applied.and_then(|i| super::look::info(&eng, i).ok())
    });
    info.unwrap_or(Value::Null).to_string()
}

pub fn og_load_texture_array(world: u32, path: String) -> String {
    result(engine(world).and_then(|eng| assets::load_texture_array(&eng, &path)))
}

pub fn og_load_environment(world: u32, path: String) -> String {
    result(engine(world).and_then(|eng| super::environment::load(&eng, &path).map(asset_value)))
}

pub fn og_load_emitter(world: u32, path: String) -> String {
    result(engine(world).and_then(|eng| super::fx::load_emitter(&eng, &path).map(asset_value)))
}

pub fn og_load_font(world: u32, path: String) -> String {
    result(engine(world).and_then(|eng| assets::load_font(&eng, &path).map(asset_value)))
}

pub fn og_load_font_info(world: u32, font: f64) -> String {
    result((|| {
        let eng = engine(world)?;
        let a = eng.assets.borrow();
        index_of(font)
            .and_then(|i| a.fonts.get(i))
            .map(|f| f.info.clone())
            .ok_or_else(|| format!("font {font} is not alive"))
    })())
}

pub fn og_load_sound(world: u32, path: String) -> String {
    result(engine(world).and_then(|eng| {
        let i = assets::load_sound(&eng, &path)?;
        let handle = eng.assets.borrow().sound_handles[i];
        Ok(json!(handle as f64))
    }))
}

pub fn og_load_sound_info(world: u32, sound: f64) -> String {
    result((|| {
        let eng = engine(world)?;
        let i = assets::sound_index(&eng, sound);
        let a = eng.assets.borrow();
        let s = i
            .and_then(|i| a.sounds.get(i))
            .ok_or_else(|| format!("sound {sound} is not alive"))?;
        let m = &s.meta;
        Ok(json!({
            "rate": m.sample_rate, "channels": m.channels, "frames": m.frames, "chunks": m.chunk_count,
            "seconds": f64::from(m.frames) / f64::from(m.sample_rate.max(1)),
        }))
    })())
}

pub fn og_load_status(world: u32, asset: f64) -> String {
    let status = engine(world).ok().and_then(|eng| {
        let a = eng.assets.borrow();
        index_of(asset)
            .and_then(|i| a.entries.get(i))
            .map(|e| e.status.clone())
    });
    match status {
        Some(Status::Ready) => "ready",
        Some(Status::Pending) => "pending",
        _ => "failed",
    }
    .into()
}

pub fn og_load_error(world: u32, asset: f64) -> String {
    let reason = engine(world).ok().and_then(|eng| {
        let a = eng.assets.borrow();
        match index_of(asset).and_then(|i| a.entries.get(i)) {
            Some(e) => match &e.status {
                Status::Failed(r) => Some(r.clone()),
                _ => None,
            },
            None => Some(format!("{asset} is not an asset handle")),
        }
    });
    json!(reason).to_string()
}

/// A model's clip names from its `#anim` blob, in glTF order; none when the
/// cook wrote no `#anim` (no skins, clips or morph targets).
fn clip_names(a: &assets::Assets, model: &str) -> Result<Vec<String>, String> {
    let id = og_cook::runtime::anim_id(model);
    if a.bundle_for(&id).is_err() {
        return Ok(Vec::new());
    }
    let (bytes, bundle) = a.blob(&id)?;
    bundle
        .clip_names(&id, bytes.bytes())
        .map_err(|e| e.to_string())
}

pub fn og_load_gltf_info(world: u32, asset: f64) -> String {
    result((|| {
        let eng = engine(world)?;
        let a = eng.assets.borrow();
        let e = index_of(asset)
            .and_then(|i| a.entries.get(i))
            .ok_or_else(|| format!("asset {asset} is not alive"))?;
        match &e.payload {
            Payload::Gltf {
                scenes,
                geometries,
                materials,
            } => Ok(json!({
                "scenes": scenes,
                "geometries": geometries.iter().map(|g| *g as f64).collect::<Vec<_>>(),
                "materials": materials.iter().map(|m| *m as f64).collect::<Vec<_>>(),
                "animations": clip_names(&a, &e.path)?,
            })),
            _ => Err(format!("{} is not a glTF", e.path)),
        }
    })())
}

// ── commands ────────────────────────────────────────────────────────────────

pub fn og_commands_register(name: String, handler: u32, desc: Option<String>) -> String {
    if og_probe::command::CORE_ACTIONS
        .iter()
        .any(|(core, _, _)| *core == name)
    {
        return refuse(format!(
            "'{name}' is a core action; register under another name"
        ));
    }
    let valid = og_probe::command::parse(&format!("0:{name}"))
        .is_ok_and(|s| s.len() == 1 && s[0].action == name && s[0].args.is_empty());
    if !valid {
        return refuse(format!(
            "'{name}' is not an action name (snake_case, such as `spawn_wave`)"
        ));
    }
    let desc = parse(desc.as_deref()).unwrap_or_default();
    let decl = CommandDecl {
        handler,
        usage: desc["usage"].as_str().unwrap_or_default().to_string(),
        doc: desc["doc"].as_str().unwrap_or_default().to_string(),
    };
    let taken = DECL.with(|d| {
        let mut d = d.borrow_mut();
        if d.commands.contains_key(&name) {
            return true;
        }
        d.commands.insert(name.clone(), decl.clone());
        false
    });
    if taken {
        return refuse(format!("'{name}' is already registered"));
    }
    // Engines already running take it too.
    for id in super::engine::engine_ids() {
        if let Ok(eng) = engine(id) {
            if eng.state.borrow().commands.contains_key(&name) {
                continue;
            }
            eng.state
                .borrow_mut()
                .commands
                .insert(name.clone(), decl.clone());
            if let Ok(mut probe) = eng.probe.try_borrow_mut() {
                let _ = rprobe::register_one(&mut probe, id, &name, &decl.usage, &decl.doc);
            }
        }
    }
    ok_void()
}

pub fn og_commands_parse(script: String) -> String {
    match og_probe::command::parse(&script) {
        Ok(steps) => ok(Value::Array(steps.iter().map(|s| s.to_json()).collect())),
        Err(e) => refuse(e.to_string()),
    }
}

pub fn og_commands_schedule(world: u32, script: String) -> String {
    result((|| {
        let eng = engine(world)?;
        with_probe(&eng, false, |p, w| p.schedule(w, &script))?
            .map(|r| r.to_json())
            .map_err(|e| e.to_string())
    })())
}

pub fn og_commands_help(world: u32) -> String {
    let list = engine(world)
        .ok()
        .and_then(|eng| {
            eng.probe
                .try_borrow()
                .ok()
                .map(|p| p.help().iter().map(|c| c.to_json()).collect::<Vec<_>>())
        })
        .unwrap_or_else(|| {
            og_probe::command::CORE_ACTIONS
                .iter()
                .map(|(n, u, d)| json!({ "name": n, "usage": u, "doc": d, "core": true }))
                .collect()
        });
    Value::Array(list).to_string()
}

// ── probe ───────────────────────────────────────────────────────────────────

/// One probe member by name through og-probe's JSON entry, so the typed
/// binding symbols answer exactly what the WebSocket and the request ring do.
fn member(probe: u32, method: &str, params: Value) -> Value {
    let frame =
        json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }).to_string();
    let answer = super::engine::call_rpc(probe, &frame);
    let v: Value = serde_json::from_str(&answer).unwrap_or(Value::Null);
    v.get("result").cloned().unwrap_or(Value::Null)
}

pub fn og_runtime_probe_get_version(_probe: u32) -> u32 {
    og_probe::PROBE_VERSION
}

pub fn og_runtime_probe_tick(probe: u32) -> u32 {
    og_world_get_tick(probe)
}

pub fn og_runtime_probe_state(probe: u32) -> String {
    member(probe, "state", json!({})).to_string()
}

pub fn og_runtime_probe_advance(probe: u32, steps: u32) -> u32 {
    let v = member(
        probe,
        "advance",
        json!({ "steps": steps.min(og_probe::PLAYTEST_MAX_TICKS as u32) }),
    );
    v.as_u64()
        .map_or_else(|| og_world_get_tick(probe), |t| t as u32)
}

pub fn og_runtime_probe_pause(probe: u32) -> bool {
    member(probe, "pause", json!({})).as_bool().unwrap_or(false)
}

pub fn og_runtime_probe_resume(probe: u32) -> bool {
    member(probe, "resume", json!({}))
        .as_bool()
        .unwrap_or(false)
}

pub fn og_runtime_probe_input(probe: u32, event: String) -> bool {
    let Ok(ev) = parse(Some(&event)) else {
        return false;
    };
    member(probe, "input", json!({ "event": ev }))
        .as_bool()
        .unwrap_or(false)
}

pub fn og_runtime_probe_accepted(probe: u32) -> u32 {
    member(probe, "accepted", json!({})).as_u64().unwrap_or(0) as u32
}

pub fn og_runtime_probe_pick(probe: u32, x: f64, y: f64) -> String {
    member(probe, "pick", json!({ "x": x, "y": y })).to_string()
}

pub fn og_runtime_probe_entity(probe: u32, id: String) -> String {
    member(probe, "entity", json!({ "id": id })).to_string()
}

pub fn og_runtime_probe_bounds(probe: u32, id: String) -> String {
    member(probe, "bounds", json!({ "id": id })).to_string()
}

pub fn og_runtime_probe_patch(probe: u32, id: String, parameter: String, value: String) -> bool {
    let Ok(value) = parse(Some(&value)) else {
        return false;
    };
    member(
        probe,
        "patch",
        json!({ "id": id, "parameter": parameter, "value": value }),
    )
    .as_bool()
    .unwrap_or(false)
}

pub fn og_runtime_probe_run(probe: u32, program: String) -> String {
    let program = parse(Some(&program)).unwrap_or(json!([]));
    member(probe, "run", json!({ "program": program })).to_string()
}

pub fn og_engine_probe_hello(probe: u32) -> String {
    member(probe, "hello", json!({})).to_string()
}

pub fn og_engine_probe_capture(probe: u32, width: Option<u32>, height: Option<u32>) -> String {
    member(
        probe,
        "capture",
        json!({ "width": width.unwrap_or(0), "height": height.unwrap_or(0) }),
    )
    .to_string()
}

pub fn og_engine_probe_sweep(probe: u32, desc: Option<String>) -> String {
    member(probe, "sweep", parse(desc.as_deref()).unwrap_or(json!({}))).to_string()
}

pub fn og_engine_probe_reload(probe: u32) -> String {
    member(probe, "reload", json!({})).to_string()
}

pub fn og_engine_probe_snapshot(probe: u32) -> String {
    member(probe, "snapshot", json!({})).to_string()
}

pub fn og_engine_probe_restore(probe: u32, id: String) -> bool {
    member(probe, "restore", json!({ "id": id }))
        .as_bool()
        .unwrap_or(false)
}

pub fn og_engine_probe_dump(probe: u32, desc: Option<String>) -> String {
    let mut d = parse(desc.as_deref()).unwrap_or(json!({}));
    if d["what"].is_null() {
        d["what"] = json!("world");
    }
    member(probe, "dump", d).to_string()
}

pub fn og_engine_probe_commands(probe: u32, script: String) -> String {
    member(probe, "commands", json!({ "script": script })).to_string()
}

pub fn og_engine_probe_listen(probe: u32, seconds: Option<f64>) -> String {
    member(
        probe,
        "listen",
        json!({ "seconds": seconds.unwrap_or(1.0) }),
    )
    .to_string()
}

pub fn og_engine_probe_memory(probe: u32) -> String {
    member(probe, "memory", json!({})).to_string()
}

pub fn og_engine_probe_describe(probe: u32) -> String {
    member(probe, "describe", json!({})).to_string()
}

/// A malformed expectation answers null (the call's error names why).
pub fn og_engine_probe_expect(probe: u32, what: String) -> String {
    member(probe, "expect", json!({ "what": what })).to_string()
}

pub fn og_engine_probe_expectations(probe: u32) -> String {
    member(probe, "expectations", json!({})).to_string()
}

pub fn og_engine_probe_save_scene(
    probe: u32,
    write: Option<bool>,
    spawned: Option<bool>,
) -> String {
    member(
        probe,
        "save_scene",
        json!({ "write": write.unwrap_or(false), "spawned": spawned.unwrap_or(false) }),
    )
    .to_string()
}
