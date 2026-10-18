//! The IDL's `ai` module over og-ai: behaviours (cooked or from text),
//! blackboard access, noises and descriptions. The behaviour registry is
//! the engine's (`e6::Modules::behaviours`); og.ai's system and memories are
//! the world's.

use super::engine::engine;
use super::{ok, ok_void, parse, refuse};
use og_ai::{AiSystem, Behaviour, Root, Value as AiValue};
use og_core::glam::Vec3;
use og_ecs::{Entity, World};
use serde_json::{Value, json};
use std::sync::Arc;

fn wire(r: Result<Value, String>) -> String {
    match r {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

fn handle(h: f64) -> u64 {
    if h.is_finite() && h >= 0.0 {
        h as u64
    } else {
        0
    }
}

fn system(engine: u32) -> Option<AiSystem> {
    super::e6::with(engine, |m| m.ai.clone()).flatten()
}

fn info(h: u64, b: &Behaviour) -> Value {
    let (kind, states) = match &b.root {
        Root::Tree(_) => ("tree", Vec::new()),
        Root::Machine { states, .. } => {
            ("machine", states.iter().map(|s| s.name.clone()).collect())
        }
    };
    json!({
        "behaviour": h, "name": b.name, "kind": kind, "states": states,
        "nodes": b.nodes.iter().map(|n| n.name.clone()).collect::<Vec<_>>(),
        "keys": b.keys.iter().map(|k| k.0.clone()).collect::<Vec<_>>(),
    })
}

fn insert(world: u32, b: Behaviour) -> Result<Value, String> {
    let reg = super::e6::with(world, |m| m.behaviours.clone())
        .ok_or_else(|| format!("no engine {world}"))?;
    let mut r = reg.lock().unwrap_or_else(|e| e.into_inner());
    let h = r.insert(b);
    let b = r.get(h).expect("just inserted");
    Ok(info(h, &b))
}

pub fn og_ai_load(world: u32, path: String) -> String {
    wire(engine(world).and_then(|eng| {
        let id = og_cook::runtime::asset_id(&path).to_string();
        let text = {
            let assets = eng.assets.borrow();
            let (bytes, bundle) = assets.blob(&id)?;
            bundle
                .behaviour_text(&id, bytes.bytes())
                .map_err(|e| e.to_string())?
        };
        let b = Behaviour::parse(&text).map_err(|e| format!("{id}: {e}"))?;
        insert(world, b)
    }))
}

pub fn og_ai_parse(world: u32, text: String) -> String {
    wire(engine(world).and_then(|_| insert(world, Behaviour::parse(&text)?)))
}

fn to_json(v: AiValue) -> Value {
    match v {
        AiValue::Num(n) => json!(n),
        AiValue::Bool(b) => json!(b),
        AiValue::Vec(v) => json!({ "x": v.x, "y": v.y, "z": v.z }),
        AiValue::Entity(e) => json!({ "entity": e }),
    }
}

fn from_json(v: &Value) -> Option<AiValue> {
    match v {
        Value::Number(n) => Some(AiValue::Num(n.as_f64()?)),
        Value::Bool(b) => Some(AiValue::Bool(*b)),
        Value::Object(o) if o.contains_key("entity") => {
            Some(AiValue::Entity(o["entity"].as_f64()? as u64))
        }
        Value::Object(o) => Some(AiValue::Vec(Vec3::new(
            o.get("x")?.as_f64()? as f32,
            o.get("y").and_then(Value::as_f64).unwrap_or(0.0) as f32,
            o.get("z").and_then(Value::as_f64).unwrap_or(0.0) as f32,
        ))),
        _ => None,
    }
}

pub fn og_ai_get(world: u32, entity: f64, key: String) -> String {
    system(world)
        .and_then(|s| s.get(Entity::from_bits(handle(entity)), &key))
        .map_or(Value::Null, to_json)
        .to_string()
}

pub fn og_ai_set(world: u32, entity: f64, key: String, value: String) -> String {
    let r = (|| -> Result<(), String> {
        if og_ai::BUILTIN_KEYS.contains(&key.as_str()) {
            return Err(format!(
                "'{key}' is written by og.ai every think; set a key of your own"
            ));
        }
        let v = from_json(&parse(Some(&value))?)
            .ok_or("the value is a number, a boolean, a point {x, y, z} or an entity {entity}")?;
        let s = system(world).ok_or("this world has no og.ai")?;
        if s.set(Entity::from_bits(handle(entity)), &key, v) {
            Ok(())
        } else {
            Err("the agent has not thought yet: set its keys from the tick after its first".into())
        }
    })();
    match r {
        Ok(()) => ok_void(),
        Err(e) => refuse(e),
    }
}

pub fn og_ai_noise(world: u32, desc: Option<String>) -> String {
    let r = (|| -> Result<(), String> {
        let d = parse(desc.as_deref())?;
        let p = &d["position"];
        let at = Vec3::new(
            p["x"].as_f64().unwrap_or(0.0) as f32,
            p["y"].as_f64().unwrap_or(0.0) as f32,
            p["z"].as_f64().unwrap_or(0.0) as f32,
        );
        let s = system(world).ok_or("this world has no og.ai")?;
        s.noise(og_ai::Noise {
            position: at,
            loudness: d["loudness"].as_f64().unwrap_or(1.0).clamp(0.0, 100.0) as f32,
        });
        Ok(())
    })();
    match r {
        Ok(()) => ok_void(),
        Err(e) => refuse(e),
    }
}

fn describe_value(world_id: u32, e: Entity, perception: Option<(bool, u64)>) -> Option<Value> {
    let s = system(world_id)?;
    let mem = s.memory(e)?;
    let reg = super::e6::with(world_id, |m| m.behaviours.clone())?;
    let b: Arc<Behaviour> = reg
        .lock()
        .unwrap_or_else(|x| x.into_inner())
        .get(mem.behaviour)?;
    let d = s.describe(e)?;
    let (kind, state) = match &b.root {
        Root::Tree(_) => ("tree", String::new()),
        Root::Machine { .. } => ("machine", d.state_name(&b)),
    };
    let status = ["idle", "running", "success", "failure"]
        .get(mem.status as usize)
        .copied()
        .unwrap_or("idle");
    let blackboard: serde_json::Map<String, Value> = mem
        .keys
        .iter()
        .map(|(k, v)| (k.clone(), to_json(*v)))
        .collect();
    let (sees, target) = perception.unwrap_or((false, 0));
    Some(json!({
        "behaviour": b.name, "kind": kind, "state": state,
        "node": b.node_name(mem.leaf as usize), "path": d.path(&b), "status": status,
        "blackboard": blackboard, "sees": sees, "target": target,
    }))
}

fn perception_of(world: &World, e: Entity) -> Option<(bool, u64)> {
    world
        .get::<og_ai::Perception>(e)
        .ok()
        .map(|p| (p.sees, p.target))
}

pub fn og_ai_describe(world: u32, entity: f64) -> String {
    let e = Entity::from_bits(handle(entity));
    let p = engine(world).ok().and_then(|eng| {
        if super::engine::running(world).is_some() {
            return None;
        }
        eng.world().ok().and_then(|w| perception_of(w, e))
    });
    describe_value(world, e, p)
        .unwrap_or(Value::Null)
        .to_string()
}

/// `dump_ai`'s JSON.
pub(crate) fn describe_json(engine: u32, world: &World, e: Entity) -> Option<Value> {
    let mut v = describe_value(engine, e, perception_of(world, e))?;
    v["entity"] = json!(og_probe::Names.id_of(world, e));
    if let Ok(p) = world.get::<og_ai::Perception>(e) {
        v["perception"] = json!({
            "sees": p.sees, "target": p.target, "lastSeen": [p.last_seen.x, p.last_seen.y, p.last_seen.z],
            "sinceSeen": p.since_seen, "heard": p.heard, "sinceHeard": p.since_heard,
        });
    }
    if let Ok(a) = world.get::<og_ai::AiAgent>(e) {
        v["changes"] = json!(a.changes);
    }
    Some(v)
}
