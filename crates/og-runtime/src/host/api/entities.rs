//! World's entities: named, spawned, despawned and found.

use super::*;

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
pub(super) fn runtime_name(eng: &Engine, e: Entity, name: &str) {
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
pub(super) fn spawn_parent(call: &str, desc: &Value) -> Result<Option<f64>, String> {
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
pub(super) fn spawn_desc(call: &str, desc: &Value) -> Result<crate::host::wire::SpawnDesc, String> {
    spawn_parent(call, desc)?;
    wire::value(desc, "desc")
}

/// The og.Transform fields a desc names (the layout's defaults for the rest).
pub(super) fn spawn_transform(desc: &Value) -> Value {
    let mut value = serde_json::Map::new();
    for key in ["position", "quaternion", "scale"] {
        if let Some(v) = desc.get(key).filter(|v| !v.is_null()) {
            value.insert(key.to_string(), v.clone());
        }
    }
    Value::Object(value)
}

pub(super) fn spawn(eng: &Engine, call: &str, desc: &Value) -> Result<Entity, String> {
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
pub(super) fn spawn_source(source: &str) -> Option<std::sync::Arc<str>> {
    if cfg!(feature = "publish") || source.is_empty() {
        return None;
    }
    Some(source.into())
}

/// `world.spawn` inside a system: queued, applied when the system returns
/// (its id is usable at once); every component it gets must be among the
/// system's writes.
pub(super) fn spawn_in_system(
    eng: &Engine,
    f: &crate::engine::Frame,
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
pub(super) fn with_descendants(world: &World, root: Entity) -> Result<Vec<Entity>, String> {
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

pub fn og_world_find(world: u32, name: String) -> Option<u64> {
    let found = engine(world).ok().and_then(|eng| {
        let st = eng.state.borrow();
        st.names.get(&name).copied()
    });
    found.filter(|&bits| og_world_alive(world, bits as f64))
}

/// `world.prefabOf`: the entity's og.PrefabLink, or null (no link, a dead
/// entity, or a world that never placed a prefab).
pub fn og_world_prefab_of(world: u32, entity_id: f64) -> Value {
    og_world_get(world, entity_id, crate::world::prefab::LINK.to_string()).unwrap_or(Value::Null)
}

pub fn og_world_name_of(world: u32, entity_id: f64) -> Option<String> {
    let name = engine(world).ok().and_then(|eng| {
        let st = eng.state.borrow();
        st.name_of.get(&(entity_id as u64)).cloned()
    });
    name.filter(|_| og_world_alive(world, entity_id))
}
