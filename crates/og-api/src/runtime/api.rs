//! The binding symbols, host-neutral: each function is the body of the
//! trait method of the same name (wire arguments in, wire answer out).

use super::codec::{Layout, RecordKind};
use super::engine::{
    before_structural, call, declare_game, declare_system, engine, extract_scene, invalidate,
    queued_structural, running, step, world_query_views, CommandDecl, Engine, QueryRec, DECL,
    HANDLES,
};
use super::{ok, ok_void, parse, refuse};
use og_core::reflect::Reflect;
use og_core::{Rng, Time};
use og_ecs::{Entity, QueryDesc, World};
use serde_json::{json, Value};

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
            }))
        }
    };
    let id = eng.world()?.id_by_name(name).map_err(err)?;
    Ok((layout, id))
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

fn set_name(eng: &Engine, e: Entity, name: &str) {
    let mut st = eng.state.borrow_mut();
    st.names.insert(name.to_string(), e.to_bits());
    st.name_of.insert(e.to_bits(), name.to_string());
    st.name_order.push(name.to_string());
}

fn forget_name(eng: &Engine, e: Entity) {
    let mut st = eng.state.borrow_mut();
    if let Some(name) = st.name_of.remove(&e.to_bits()) {
        st.names.remove(&name);
        st.name_order.retain(|n| *n != name);
    }
}

fn spawn(eng: &Engine, desc: &Value) -> Result<Entity, String> {
    let name = desc["name"].as_str().unwrap_or_default().to_string();
    if !name.is_empty() {
        if eng.state.borrow().names.contains_key(&name) {
            return Err(format!("an entity named '{name}' already exists"));
        }
        if name.starts_with('#') || name.chars().any(|c| c.is_whitespace() || ":;=".contains(c)) {
            return Err(format!("'{name}' cannot be addressed by the probe: a name has no whitespace, ':', ';' or '=' and does not start with '#'"));
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
    Ok(e)
}

pub fn og_world_spawn(world: u32, desc: Option<String>) -> String {
    if running(world).is_some() {
        return refuse("spawning inside a system needs og-ecs SystemContext::spawn_bytes (requested from Core); spawn in a command meanwhile");
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
    format!("{what} inside a system needs og-ecs SystemContext packed-byte access (requested from Core); do it in a command, or through the system's views")
}

pub fn og_world_insert(world: u32, entity_id: f64, comp: String, value: Option<String>) -> String {
    if running(world).is_some() {
        return refuse(in_system("world.insert"));
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
    if running(world).is_some() {
        return refuse(in_system("world.remove"));
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
    if running(world).is_some() {
        return refuse(in_system("world.get"));
    }
    result((|| {
        let eng = engine(world)?;
        let e = entity(entity_id)?;
        let (layout, id) = component(&eng, &comp)?;
        let bytes = eng.world()?.get_bytes(e, id).map_err(err)?;
        Ok(layout.decode(&bytes))
    })())
}

pub fn og_world_set(world: u32, entity_id: f64, comp: String, value: String) -> String {
    if running(world).is_some() {
        return refuse(in_system("world.set"));
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

fn time_json(t: Time, rate: u32) -> Value {
    json!({ "tick": t.tick, "dt": f64::from(t.dt), "elapsed": t.elapsed, "rate": rate })
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
            return Ok(time_json(ctx.time(), rate));
        }
        if !declared {
            return Err(format!(
                "system '{}' did not declare '{name}' among its reads or writes",
                s.name
            ));
        }
        if name == Rng::NAME {
            return ctx.resource::<Rng>().map(|r| rng_json(&r)).map_err(err);
        }
        return Err(in_system(&format!("reading resource '{name}'")));
    }
    let w = eng.world()?;
    if name == Time::NAME {
        return Ok(time_json(w.time(), rate));
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
        return refuse("a system receives its query as its first argument; world.query is for commands and callbacks");
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

pub fn og_world_events(world: u32, event: String) -> String {
    // The IDL answers a list; a refusal is an empty list.
    let _ = (world, event);
    "[]".into()
}

pub fn og_world_emit(world: u32, event: String, payload: String) -> String {
    if running(world).is_some() {
        return refuse(in_system("world.emit"));
    }
    result_void((|| {
        let eng = engine(world)?;
        let (layout, id) = event_layout(&eng, &event)?;
        let bytes = layout.encode(None, Some(&parse(Some(&payload))?))?;
        eng.world()?.send_bytes(id, &bytes).map_err(err)
    })())
}

pub fn og_world_instantiate(_world: u32, _asset: f64, _desc: Option<String>) -> String {
    refuse("no asset is ever ready yet: loading lands with the cook (M4)")
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
        json!(d
            .borrow()
            .games
            .get(game as usize)
            .map(|g| g.systems.clone())
            .unwrap_or_default())
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
/// be among its writes; otherwise the draw is refused and answers NaN.
fn with_rng<T>(world: u32, refused: T, f: impl FnOnce(&mut Rng) -> T) -> T {
    if let Some(frame) = running(world) {
        // SAFETY: the frame is published while its system runs.
        let ctx = unsafe { frame.ctx() };
        return match ctx.resource_mut::<Rng>() {
            Ok(mut r) => f(&mut r),
            Err(_) => refused,
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
    match with_rng(world, None, |r| Some(r.fork(&label))) {
        Some(r) => new_stream(world, r),
        None => 0,
    }
}

// ── input ───────────────────────────────────────────────────────────────────

fn input<T>(world: u32, f: impl FnOnce(&super::engine::InputState) -> T, refused: T) -> T {
    match engine(world) {
        Ok(eng) => f(&eng.state.borrow().input),
        Err(_) => refused,
    }
}

pub fn og_input_down(world: u32, key: String) -> bool {
    input(world, |i| i.keys.contains(&key), false)
}
pub fn og_input_pressed(world: u32, key: String) -> bool {
    input(world, |i| i.pressed.contains(&key), false)
}
pub fn og_input_released(world: u32, key: String) -> bool {
    input(world, |i| i.released.contains(&key), false)
}
pub fn og_input_axis(world: u32, negative: String, positive: String) -> f64 {
    input(
        world,
        |i| {
            f64::from(u8::from(i.keys.contains(&positive)))
                - f64::from(u8::from(i.keys.contains(&negative)))
        },
        0.0,
    )
}
pub fn og_input_button_down(world: u32, button: String) -> bool {
    input(world, |i| i.buttons.contains(&button), false)
}
pub fn og_input_button_pressed(world: u32, button: String) -> bool {
    input(world, |i| i.buttons_pressed.contains(&button), false)
}

// ── camera ──────────────────────────────────────────────────────────────────

fn surface_size(eng: &Engine) -> (f32, f32) {
    let st = eng.state.borrow();
    let (w, h) = st.backend.as_ref().map_or((1280, 720), |b| b.size());
    (w.max(1) as f32, h.max(1) as f32)
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

fn material(base: og_render::Material, desc: &Value) -> og_render::Material {
    let mut m = base;
    let color = &desc["color"];
    if color.is_object() {
        let opacity = m.base_color[3];
        m.base_color = [
            color["r"].as_f64().unwrap_or(1.0) as f32,
            color["g"].as_f64().unwrap_or(1.0) as f32,
            color["b"].as_f64().unwrap_or(1.0) as f32,
            opacity,
        ];
    }
    if let Some(o) = desc["opacity"].as_f64() {
        m.base_color[3] = o as f32;
    }
    if let Some(v) = desc["metalness"].as_f64() {
        m.metallic = v as f32;
    }
    if let Some(v) = desc["roughness"].as_f64() {
        m.roughness = v as f32;
    }
    let e = &desc["emissive"];
    if e.is_object() {
        m.emissive = [
            e["r"].as_f64().unwrap_or(0.0) as f32,
            e["g"].as_f64().unwrap_or(0.0) as f32,
            e["b"].as_f64().unwrap_or(0.0) as f32,
        ];
    }
    if let Some(t) = desc["alphaTest"].as_f64().filter(|t| *t > 0.0) {
        m.alpha_mode = og_render::AlphaMode::Mask { cutoff: t as f32 };
    }
    m
}

/// Material handles are the scene's material index with generation 1.
fn material_handle(index: usize) -> f64 {
    (index as u64 + (1u64 << 32)) as f64
}

pub fn og_render_standard_material(world: u32, desc: Option<String>) -> String {
    result((|| {
        let eng = engine(world)?;
        let desc = parse(desc.as_deref())?;
        // three.js MeshStandardMaterial defaults: white, metalness 0, roughness 1.
        let base = og_render::Material {
            roughness: 1.0,
            ..og_render::Material::default()
        };
        let mut st = eng.state.borrow_mut();
        let handle = st.scene.add_material(material(base, &desc));
        st.materials.push(handle);
        Ok(json!(material_handle(st.materials.len() - 1)))
    })())
}

pub fn og_render_update_material(world: u32, handle: f64, desc: Option<String>) -> String {
    result_void((|| {
        let eng = engine(world)?;
        let desc = parse(desc.as_deref())?;
        let bits = handle as u64;
        let mut st = eng.state.borrow_mut();
        let index = (bits & 0xffff_ffff) as usize;
        let h = st
            .materials
            .get(index)
            .copied()
            .filter(|_| bits >> 32 == 1)
            .ok_or_else(|| format!("material {handle} is not alive"))?;
        let current = st
            .scene
            .material(h)
            .cloned()
            .ok_or("material is not alive")?;
        st.scene.set_material(h, material(current, &desc));
        Ok(())
    })())
}

pub fn og_render_capabilities(world: u32) -> String {
    let caps = engine(world).ok().and_then(|eng| {
        let st = eng.state.borrow();
        st.backend.as_ref().map(|b| {
            let tier = b.tier();
            let (w, h) = b.size();
            json!({ "tier": tier, "compute": tier != "webgl2", "maxShadowedLights": 1, "width": w, "height": h })
        })
    });
    caps.unwrap_or_else(|| json!({ "tier": "none", "compute": false, "maxShadowedLights": 0, "width": 0, "height": 0 }))
        .to_string()
}

// ── load ────────────────────────────────────────────────────────────────────

const LOADING: &str = "asset loading reads cooked output, which lands with the cook (M4)";

pub fn og_load_gltf(_world: u32, path: String, _desc: Option<String>) -> String {
    refuse(format!("{path}: {LOADING}"))
}
pub fn og_load_scene(_world: u32, path: String) -> String {
    refuse(format!("{path}: {LOADING}"))
}
pub fn og_load_status(_world: u32, _asset: f64) -> String {
    "failed".into()
}
pub fn og_load_gltf_info(_world: u32, _asset: f64) -> String {
    refuse(LOADING)
}

// ── commands ────────────────────────────────────────────────────────────────

const CORE_ACTIONS: &[&str] = &[
    "screenshot",
    "touch_move_to",
    "touch_move",
    "touch_start",
    "touch_end",
    "scroll",
    "key_down",
    "key_up",
    "type",
    "move_to_text",
    "teleport",
    "frame_entity",
    "dump_world",
    "dump_entity",
    "dump_ui",
    "set",
    "seed",
    "pause",
    "resume",
    "advance",
    "quit",
];

pub fn og_commands_register(name: String, handler: u32, desc: Option<String>) -> String {
    let valid = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if !valid {
        return refuse(format!("'{name}' is not a snake_case action name"));
    }
    if CORE_ACTIONS.contains(&name.as_str()) {
        return refuse(format!("'{name}' is a core action"));
    }
    let desc = parse(desc.as_deref()).unwrap_or_default();
    DECL.with(|d| {
        let mut d = d.borrow_mut();
        if d.commands.contains_key(&name) {
            return refuse(format!("'{name}' is already registered"));
        }
        d.commands.insert(
            name,
            CommandDecl {
                handler,
                usage: desc["usage"].as_str().unwrap_or_default().to_string(),
                doc: desc["doc"].as_str().unwrap_or_default().to_string(),
            },
        );
        ok_void()
    })
}

/// `t:action[:args];...` into steps.
fn parse_script(script: &str) -> Result<Vec<Value>, String> {
    let mut out = Vec::new();
    for (i, step) in script.split(';').enumerate() {
        let step = step.trim();
        if step.is_empty() {
            continue;
        }
        let mut parts = step.split(':');
        let at = parts.next().unwrap_or_default();
        let at: f64 = at
            .parse()
            .ok()
            .filter(|t: &f64| t.is_finite() && *t >= 0.0)
            .ok_or_else(|| format!("step {} `{step}`: `{at}` is not a time in seconds", i + 1))?;
        let action = parts
            .next()
            .filter(|a| !a.is_empty())
            .ok_or_else(|| format!("step {} `{step}`: no action after the time", i + 1))?;
        let args: Vec<&str> = parts.collect();
        out.push(json!({ "at": at, "action": action, "args": args }));
    }
    Ok(out)
}

pub fn og_commands_parse(script: String) -> String {
    result(parse_script(&script).map(Value::Array))
}

pub fn og_commands_schedule(_world: u32, script: String) -> String {
    match parse_script(&script) {
        Err(e) => refuse(e),
        Ok(_) => refuse("scheduling on simulation time lands with the probe's command channel (M5); run a registered action now with the engine's command()"),
    }
}

// ── probe ───────────────────────────────────────────────────────────────────

pub fn og_runtime_probe_get_version(_probe: u32) -> u32 {
    5
}

pub fn og_runtime_probe_tick(probe: u32) -> u32 {
    og_world_get_tick(probe)
}

pub fn og_runtime_probe_state(probe: u32) -> String {
    let Ok(eng) = engine(probe) else {
        return "null".into();
    };
    let facts = eng.state.borrow().facts;
    let facts = match facts {
        Some(id) => call(&eng, 2, id, String::new())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(Value::Null),
        None => Value::Null,
    };
    let st = eng.state.borrow();
    let entities: Vec<Value> = st.name_order.iter().map(|n| json!({ "id": n })).collect();
    json!({
        "tick": og_world_get_tick(probe),
        "paused": st.paused,
        "seed": st.seed,
        "timings": { "step": st.step_ms, "render": st.render_ms, "frameMs": st.step_ms + st.render_ms },
        "entities": entities,
        "facts": facts,
    })
    .to_string()
}

pub fn og_runtime_probe_advance(probe: u32, steps: u32) -> u32 {
    if let Ok(eng) = engine(probe) {
        for _ in 0..steps.min(3600) {
            if let Err(e) = step(&eng) {
                eng.state.borrow_mut().warnings.push(e);
                break;
            }
        }
    }
    og_world_get_tick(probe)
}

pub fn og_runtime_probe_pause(probe: u32) -> bool {
    engine(probe)
        .map(|e| e.state.borrow_mut().paused = true)
        .is_ok()
}

pub fn og_runtime_probe_resume(probe: u32) -> bool {
    engine(probe)
        .map(|e| e.state.borrow_mut().paused = false)
        .is_ok()
}

pub fn og_runtime_probe_input(probe: u32, event: String) -> bool {
    let Ok(ev) = parse(Some(&event)) else {
        return false;
    };
    if !matches!(ev["kind"].as_str(), Some("key" | "pointer" | "wheel")) {
        return false;
    }
    match engine(probe) {
        Ok(eng) => {
            let mut st = eng.state.borrow_mut();
            st.input.queue.push(ev);
            st.accepted += 1;
            true
        }
        Err(_) => false,
    }
}

pub fn og_runtime_probe_accepted(probe: u32) -> u32 {
    engine(probe).map_or(0, |e| e.state.borrow().accepted)
}

pub fn og_runtime_probe_pick(_probe: u32, _x: f64, _y: f64) -> String {
    "null".into()
}

fn probe_entity(eng: &Engine, id: &str) -> Option<Entity> {
    if let Some(rest) = id.strip_prefix('#') {
        let (i, g) = rest.split_once('.')?;
        let e = Entity::from_bits((g.parse::<u64>().ok()? << 32) | i.parse::<u64>().ok()?);
        return eng.world().ok()?.is_alive(e).then_some(e);
    }
    let bits = *eng.state.borrow().names.get(id)?;
    let e = Entity::from_bits(bits);
    eng.world().ok()?.is_alive(e).then_some(e)
}

/// `(component, field index, layout)` of every tunable field `e` carries.
fn tunables(eng: &Engine, e: Entity) -> Vec<(String, usize, Layout)> {
    let Ok(w) = eng.world() else {
        return Vec::new();
    };
    let Ok(ids) = w.components_of(e) else {
        return Vec::new();
    };
    let st = eng.state.borrow();
    let mut out = Vec::new();
    for id in ids {
        let Some(name) = st.names_by_id.get(id.index() as usize) else {
            continue;
        };
        if let Some(layout) = st.layouts.get(name) {
            for (i, f) in layout.fields.iter().enumerate() {
                if f.tunable {
                    out.push((name.clone(), i, layout.clone()));
                }
            }
        }
    }
    out
}

pub fn og_runtime_probe_entity(probe: u32, id: String) -> String {
    let Ok(eng) = engine(probe) else {
        return "null".into();
    };
    let Some(e) = probe_entity(&eng, &id) else {
        return "null".into();
    };
    let mut map = serde_json::Map::new();
    for (comp, i, layout) in tunables(&eng, e) {
        let Ok(w) = eng.world() else { break };
        let Ok(cid) = w.id_by_name(&comp) else {
            continue;
        };
        if let Ok(bytes) = w.get_bytes(e, cid) {
            map.insert(
                format!("{}.{}", short(&comp), layout.fields[i].name),
                layout.decode_one(&bytes, i),
            );
        }
    }
    Value::Object(map).to_string()
}

/// `og.Transform` is `Transform` in probe paths, as in scenes.
fn short(name: &str) -> &str {
    name.strip_prefix("og.").unwrap_or(name)
}

pub fn og_runtime_probe_bounds(probe: u32, id: String) -> String {
    let Ok(eng) = engine(probe) else {
        return "null".into();
    };
    let Some(e) = probe_entity(&eng, &id) else {
        return "null".into();
    };
    let mut map = serde_json::Map::new();
    for (comp, i, layout) in tunables(&eng, e) {
        let f = &layout.fields[i];
        let path = format!("{}.{}", short(&comp), f.name);
        let mut t = json!({ "path": path });
        if let Some(l) = &f.label {
            t["label"] = json!(l);
        }
        if let Some(v) = f.min {
            t["min"] = json!(v);
        }
        if let Some(v) = f.max {
            t["max"] = json!(v);
        }
        if let Some(v) = f.step {
            t["step"] = json!(v);
        }
        map.insert(path, t);
    }
    Value::Object(map).to_string()
}

pub fn og_runtime_probe_patch(probe: u32, id: String, parameter: String, value: String) -> bool {
    let Ok(eng) = engine(probe) else { return false };
    let Some(e) = probe_entity(&eng, &id) else {
        return false;
    };
    let Ok(value) = parse(Some(&value)) else {
        return false;
    };
    for (comp, i, layout) in tunables(&eng, e) {
        let f = &layout.fields[i];
        if format!("{}.{}", short(&comp), f.name) != parameter {
            continue;
        }
        let patch = json!({ f.name.clone(): value });
        let answer = og_world_set(probe, e.to_bits() as f64, comp, patch.to_string());
        return answer.starts_with(r#"{"ok":true"#);
    }
    false
}

pub fn og_runtime_probe_run(probe: u32, program: String) -> String {
    let ops = parse(Some(&program)).unwrap_or_default();
    let mut results = Vec::new();
    let mut refused = Value::Null;
    for (i, op) in ops
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .take(256)
        .enumerate()
    {
        let answer = match op["op"].as_str() {
            Some("advance") => json!(og_runtime_probe_advance(
                probe,
                op["steps"].as_u64().unwrap_or(0) as u32
            )),
            Some("input") => json!(og_runtime_probe_input(probe, op["event"].to_string())),
            Some("pause") => json!(og_runtime_probe_pause(probe)),
            Some("resume") => json!(og_runtime_probe_resume(probe)),
            Some("read") => json!({
                "tick": og_runtime_probe_tick(probe),
                "state": serde_json::from_str::<Value>(&og_runtime_probe_state(probe)).unwrap_or_default(),
            }),
            Some("pick") => Value::Null,
            Some("entity") => serde_json::from_str(&og_runtime_probe_entity(
                probe,
                op["id"].as_str().unwrap_or_default().to_string(),
            ))
            .unwrap_or_default(),
            _ => json!(false),
        };
        let stop = answer == json!(false) || (op["op"] == "entity" && answer.is_null());
        results.push(answer);
        if stop {
            refused = json!(i);
            break;
        }
    }
    json!({ "results": results, "refused": refused, "tick": og_runtime_probe_tick(probe) })
        .to_string()
}

pub fn og_engine_probe_hello(probe: u32) -> String {
    let (rate, tier) = engine(probe)
        .map(|e| {
            let st = e.state.borrow();
            (
                st.rate,
                st.backend.as_ref().map_or("none".to_string(), |b| b.tier()),
            )
        })
        .unwrap_or((0, "none".to_string()));
    json!({
        "protocol": 5,
        "wire": 1,
        "substrate": "origin-engine",
        "engine": format!("origin-engine {}", env!("CARGO_PKG_VERSION")),
        "ticksPerSecond": rate,
        "tier": tier,
        "build": "dev",
        "members": ["hello", "tick", "state", "advance", "pause", "resume", "input", "accepted", "entity", "bounds", "patch", "run", "capture", "snapshot", "restore", "dump"],
    })
    .to_string()
}

pub fn og_engine_probe_capture(probe: u32, _width: Option<u32>, _height: Option<u32>) -> String {
    let Ok(eng) = engine(probe) else {
        return "null".into();
    };
    if eng.state.borrow().backend.is_none() || extract_scene(&eng).is_err() {
        return "null".into();
    }
    let tick = og_world_get_tick(probe);
    let mut st = eng.state.borrow_mut();
    let st = &mut *st;
    let Some(image) = st.backend.as_mut().and_then(|b| b.capture(&st.scene)) else {
        return "null".into();
    };
    json!({
        "tick": tick,
        "width": image.width,
        "height": image.height,
        "png": base64(&png(&image)),
    })
    .to_string()
}

pub fn og_engine_probe_sweep(_probe: u32, _desc: Option<String>) -> String {
    "null".into()
}

pub fn og_engine_probe_reload(_probe: u32) -> String {
    "null".into()
}

pub fn og_engine_probe_snapshot(probe: u32) -> String {
    let Ok(eng) = engine(probe) else {
        return "null".into();
    };
    let Ok(w) = eng.world() else {
        return "null".into();
    };
    let bytes = w.snapshot();
    let tick = w.time().tick as u32;
    let hash = format!("{:016x}", w.world_hash());
    let mut st = eng.state.borrow_mut();
    let id = format!("s{}", st.snapshots.len() + 1);
    let size = bytes.len() as u32;
    st.snapshots.push((id.clone(), tick, bytes));
    json!({ "id": id, "tick": tick, "hash": hash, "bytes": size }).to_string()
}

pub fn og_engine_probe_restore(probe: u32, id: String) -> bool {
    let Ok(eng) = engine(probe) else { return false };
    let bytes = eng
        .state
        .borrow()
        .snapshots
        .iter()
        .find(|(s, _, _)| *s == id)
        .map(|(_, _, b)| b.clone());
    let Some(bytes) = bytes else { return false };
    invalidate(true);
    eng.world().is_ok_and(|w| w.restore(&bytes).is_ok())
}

pub fn og_engine_probe_dump(probe: u32, desc: Option<String>) -> String {
    let Ok(eng) = engine(probe) else {
        return "null".into();
    };
    let desc = parse(desc.as_deref()).unwrap_or_default();
    let Ok(w) = eng.world() else {
        return "null".into();
    };
    match desc["what"].as_str().unwrap_or("world") {
        "entity" => {
            let Some(e) = probe_entity(&eng, desc["id"].as_str().unwrap_or_default()) else {
                return "null".into();
            };
            let st = eng.state.borrow();
            let mut comps = serde_json::Map::new();
            for id in w.components_of(e).unwrap_or_default() {
                let Some(name) = st.names_by_id.get(id.index() as usize) else {
                    continue;
                };
                if let (Some(layout), Ok(bytes)) = (st.layouts.get(name), w.get_bytes(e, id)) {
                    comps.insert(short(name).to_string(), layout.decode(&bytes));
                }
            }
            json!({
                "kind": "entity",
                "id": desc["id"],
                "name": st.name_of.get(&e.to_bits()),
                "components": comps,
            })
            .to_string()
        }
        "ui" => json!({ "kind": "ui", "tick": w.time().tick, "elements": [] }).to_string(),
        _ => {
            let st = eng.state.borrow();
            let mut archetypes = Vec::new();
            if let Ok(all) = w.query(&QueryDesc::default()) {
                for a in all {
                    let comps: Vec<&str> = a
                        .types()
                        .iter()
                        .filter_map(|id| st.names_by_id.get(id.index() as usize))
                        .map(|n| short(n))
                        .collect();
                    archetypes.push(json!({ "components": comps, "count": a.len() }));
                }
            }
            let systems: Vec<Value> = st
                .systems
                .iter()
                .map(|s| {
                    json!({ "name": s.name, "matched": s.matched, "reads": s.reads, "writes": s.writes, "ms": s.ms })
                })
                .collect();
            json!({
                "kind": "world",
                "tick": w.time().tick,
                "hash": format!("{:016x}", w.world_hash()),
                "entities": w.entity_count(),
                "archetypes": archetypes,
                "systems": systems,
                "resources": st.resources,
            })
            .to_string()
        }
    }
}

pub fn og_engine_probe_commands(_probe: u32, _script: String) -> String {
    "null".into()
}

// ── small encoders for capture ──────────────────────────────────────────────

fn base64(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for (i, shift) in [18, 12, 6, 0].iter().enumerate() {
            if i <= chunk.len() {
                out.push(T[((n >> shift) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// An uncompressed (stored-deflate) PNG: exact pixels, no codec dependency.
fn png(image: &og_render::RgbaImage) -> Vec<u8> {
    fn crc(data: &[u8]) -> u32 {
        let mut c = 0xffff_ffffu32;
        for &b in data {
            c ^= u32::from(b);
            for _ in 0..8 {
                c = if c & 1 != 0 {
                    0xedb8_8320 ^ (c >> 1)
                } else {
                    c >> 1
                };
            }
        }
        !c
    }
    fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        let mut body = kind.to_vec();
        body.extend_from_slice(data);
        out.extend_from_slice(&body);
        out.extend_from_slice(&crc(&body).to_be_bytes());
    }
    let (w, h) = (image.width as usize, image.height as usize);
    let mut raw = Vec::with_capacity((w * 4 + 1) * h);
    for row in 0..h {
        raw.push(0);
        raw.extend_from_slice(&image.pixels[row * w * 4..(row + 1) * w * 4]);
    }
    let mut z = vec![0x78, 0x01];
    let blocks: Vec<&[u8]> = raw.chunks(65535).collect();
    for (i, b) in blocks.iter().enumerate() {
        z.push(u8::from(i + 1 == blocks.len()));
        z.extend_from_slice(&(b.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(b.len() as u16)).to_le_bytes());
        z.extend_from_slice(b);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for &x in &raw {
        a = (a + u32::from(x)) % 65521;
        b = (b + a) % 65521;
    }
    z.extend_from_slice(&((b << 16) | a).to_be_bytes());
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(w as u32).to_be_bytes());
    ihdr.extend_from_slice(&(h as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &z);
    chunk(&mut out, b"IEND", &[]);
    out
}
