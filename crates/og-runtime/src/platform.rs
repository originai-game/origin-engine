//! The IDL's `platform` module (docs/design.md §2.8): calls game code makes
//! by method name, answered by the host. A call is queued with a request
//! number; the host takes the queue (`og_platform_take`), answers each call
//! (`og_platform_answer`: the web shell through the Player, the native host
//! from its local JSON store), and the answers are applied by the
//! `og.platform` system at the start of the next tick, which sends
//! `og.PlatformAnswered`. Every call and answer stays in the engine's log
//! for `og.platform.result` and the probe's `dump_platform`.

use super::engine::{engine, running};
use super::{ok, ok_void, parse, refuse};
use og_core::Reflect;
use og_ecs::{SystemDesc, World};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::BTreeMap;

/// The system's name in the schedule (before the game's systems).
pub const PLATFORM_SYSTEM: &str = "og.platform";

/// `og.PlatformAnswered`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.PlatformAnswered")]
pub struct PlatformAnswered {
    pub request: u32,
    pub ok: bool,
}

/// The most calls kept in the log (the oldest answered ones go first).
const MAX_LOG: usize = 512;
/// The largest stored value, encoded.
const MAX_VALUE: usize = 64 * 1024;

const RETRYABLE: &[&str] = &["offline", "timeout", "unavailable", "rate_limited"];

struct Record {
    method: String,
    args: Value,
    tick: u64,
    /// `{ ok, data }` or `{ ok: false, error }`, and the tick it was applied at.
    answer: Option<(Value, u64)>,
    /// Fire and forget (telemetry): never answered.
    oneway: bool,
}

#[derive(Default)]
struct Platform {
    next: u32,
    outbox: Vec<u32>,
    inbox: Vec<(u32, Value)>,
    log: BTreeMap<u32, Record>,
    host: Option<String>,
    capabilities: Vec<String>,
}

thread_local! {
    static PLATFORMS: RefCell<BTreeMap<u32, Platform>> = const { RefCell::new(BTreeMap::new()) };
}

fn with<T>(engine: u32, f: impl FnOnce(&mut Platform) -> T) -> T {
    PLATFORMS.with(|p| f(p.borrow_mut().entry(engine).or_default()))
}

/// The tick a call is made at: the running system's, or the world's.
fn tick_now(world: u32) -> Result<u64, String> {
    if let Some(f) = running(world) {
        // SAFETY: the frame is published while its system runs.
        return Ok(unsafe { f.ctx() }.time().tick);
    }
    Ok(engine(world)?.world()?.time().tick)
}

fn call(world: u32, method: &str, args: Value, oneway: bool) -> Result<u32, String> {
    let tick = tick_now(world)?;
    Ok(with(world, |p| {
        p.next = p.next.wrapping_add(1).max(1);
        let id = p.next;
        p.log.insert(
            id,
            Record {
                method: method.to_string(),
                args,
                tick,
                answer: None,
                oneway,
            },
        );
        p.outbox.push(id);
        if p.log.len() > MAX_LOG {
            let old: Vec<u32> = p
                .log
                .iter()
                .filter(|(_, r)| r.answer.is_some() || r.oneway)
                .map(|(k, _)| *k)
                .take(p.log.len() - MAX_LOG)
                .collect();
            for k in old {
                p.log.remove(&k);
            }
        }
        id
    }))
}

fn request(world: u32, method: &str, args: Value) -> String {
    match call(world, method, args, false) {
        Ok(id) => ok(json!(id)),
        Err(e) => refuse(e),
    }
}

fn valid_key(key: &str) -> bool {
    (1..=64).contains(&key.len())
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

fn board_of(d: &Value) -> String {
    d["board"]
        .as_str()
        .filter(|b| valid_key(b))
        .unwrap_or("main")
        .to_string()
}

fn count_of(d: &Value) -> u64 {
    d["count"].as_u64().unwrap_or(10).clamp(1, 100)
}

pub fn og_platform_store_get(world: u32, key: String) -> String {
    if !valid_key(&key) {
        return refuse(format!(
            "'{key}' is not a store key: 1 to 64 of letters, digits, '.', '_' and '-'"
        ));
    }
    request(world, "store.get", json!({ "key": key }))
}

pub fn og_platform_store_set(world: u32, desc: Option<String>) -> String {
    let d = match parse(desc.as_deref()) {
        Ok(d) => d,
        Err(e) => return refuse(e),
    };
    let key = d["key"].as_str().unwrap_or("").to_string();
    if !valid_key(&key) {
        return refuse(format!(
            "'{key}' is not a store key: 1 to 64 of letters, digits, '.', '_' and '-'"
        ));
    }
    let value = d.get("value").cloned().unwrap_or(Value::Null);
    let size = value.to_string().len();
    if size > MAX_VALUE {
        return refuse(format!(
            "the value is {size} bytes encoded; a store value is at most {MAX_VALUE}"
        ));
    }
    let revision = d["revision"].as_u64().unwrap_or(0);
    let create = d["create"].as_bool().unwrap_or(false);
    request(
        world,
        "store.set",
        json!({ "key": key, "value": value, "revision": revision, "create": create }),
    )
}

/// The key og.platform's settings helpers use.
pub const SETTINGS_KEY: &str = "og.settings";

pub fn og_platform_load_settings(world: u32) -> u32 {
    call(world, "store.get", json!({ "key": SETTINGS_KEY }), false).unwrap_or(0)
}

pub fn og_platform_save_settings(world: u32, value: String) -> String {
    let v = match parse(Some(&value)) {
        Ok(v) if v.is_object() => v,
        Ok(_) => return refuse("settings are an object: { volume: 0.8, ... }"),
        Err(e) => return refuse(e),
    };
    if v.to_string().len() > MAX_VALUE {
        return refuse(format!("settings are at most {MAX_VALUE} bytes encoded"));
    }
    request(
        world,
        "store.set",
        json!({ "key": SETTINGS_KEY, "value": v, "revision": 0 }),
    )
}

pub fn og_platform_store_list(world: u32) -> u32 {
    call(world, "store.list", json!({}), false).unwrap_or(0)
}

pub fn og_platform_leaderboard_submit(world: u32, desc: Option<String>) -> String {
    let d = match parse(desc.as_deref()) {
        Ok(d) => d,
        Err(e) => return refuse(e),
    };
    let Some(score) = d["score"].as_f64().filter(|s| s.is_finite()) else {
        return refuse("score must be a finite number");
    };
    let order = if d["order"].as_str() == Some("asc") {
        "asc"
    } else {
        "desc"
    };
    request(
        world,
        "leaderboard.submit",
        json!({ "board": board_of(&d), "score": score, "order": order }),
    )
}

fn query(world: u32, method: &str, desc: Option<String>) -> u32 {
    let d = parse(desc.as_deref()).unwrap_or(json!({}));
    call(
        world,
        method,
        json!({ "board": board_of(&d), "count": count_of(&d) }),
        false,
    )
    .unwrap_or(0)
}

pub fn og_platform_leaderboard_top(world: u32, desc: Option<String>) -> u32 {
    query(world, "leaderboard.top", desc)
}

pub fn og_platform_leaderboard_around(world: u32, desc: Option<String>) -> u32 {
    query(world, "leaderboard.around", desc)
}

pub fn og_platform_telemetry(world: u32, name: String, attrs: Option<String>) -> String {
    let named = !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_'));
    if !named {
        return refuse(format!(
            "'{name}' is not an event name: 1 to 64 of letters, digits, '.' and '_'"
        ));
    }
    let attrs = match attrs.as_deref().map(|a| parse(Some(a))).transpose() {
        Ok(a) => a.unwrap_or(json!({})),
        Err(e) => return refuse(e),
    };
    let Some(o) = attrs.as_object() else {
        return refuse("attrs is an object of strings, numbers, booleans or null");
    };
    if o.len() > 16 || o.values().any(|v| v.is_array() || v.is_object()) {
        return refuse("attrs holds at most 16 strings, numbers, booleans or null");
    }
    let key_ok = |k: &str| {
        let mut b = k.bytes();
        b.next().is_some_and(|c| c.is_ascii_lowercase())
            && b.all(|c| c.is_ascii_alphanumeric() || c == b'_')
    };
    if let Some(k) = o.keys().find(|k| !key_ok(k)) {
        return refuse(format!(
            "attribute '{k}' is not a name: a lowercase letter, then letters, digits and '_'"
        ));
    }
    if o.values()
        .any(|v| v.as_str().is_some_and(|t| t.chars().count() > 512))
    {
        return refuse("an attribute's string is at most 512 characters");
    }
    match call(
        world,
        "telemetry",
        json!({ "name": name, "attrs": attrs }),
        true,
    ) {
        Ok(_) => ok_void(),
        Err(e) => refuse(e),
    }
}

pub fn og_platform_point(world: u32, desc: Option<String>) -> String {
    let d = parse(desc.as_deref()).unwrap_or(json!({}));
    let reason = d["reason"].as_str().unwrap_or("").trim().to_string();
    if reason.is_empty() || !valid_key(&reason) {
        return refuse(format!(
            "'{reason}' is not a reason: 1 to 64 of letters, digits, '.', '_' and '-', as `level.completed`"
        ));
    }
    let amount = d["amount"].as_u64().unwrap_or(1).clamp(1, 1000);
    request(
        world,
        "point",
        json!({ "reason": reason, "amount": amount }),
    )
}

pub fn og_platform_ai_mark(world: u32, desc: Option<String>) -> String {
    let d = parse(desc.as_deref()).unwrap_or(json!({}));
    let kind = d["kind"].as_str().unwrap_or("").trim().to_string();
    let label = d["label"].as_str().unwrap_or("").trim().to_string();
    let kind_ok = kind.len() <= 32
        && kind.bytes().next().is_some_and(|c| c.is_ascii_lowercase())
        && kind
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, b'_' | b'-'));
    if !kind_ok || label.is_empty() || label.chars().count() > 120 {
        return refuse(
            "an AI mark needs a kind (1 to 32 of a-z, 0-9, '_' and '-', a letter first: `text`, `image`) and a label of 1 to 120 characters",
        );
    }
    let mut args = json!({ "kind": kind, "label": label });
    if let Some(m) = d["model"].as_str().filter(|m| !m.is_empty()) {
        if m.chars().count() > 80 {
            return refuse("an AI mark's model is at most 80 characters");
        }
        args["model"] = json!(m);
    }
    request(world, "ai.mark", args)
}

fn result_of(id: u32, r: &Record) -> Option<Value> {
    let (answer, tick) = r.answer.as_ref()?;
    let ok = answer["ok"].as_bool() == Some(true);
    let mut v = json!({
        "request": id, "method": r.method, "ok": ok,
        "data": if ok { answer.get("data").cloned().unwrap_or(Value::Null) } else { Value::Null },
        "tick": tick,
    });
    if !ok {
        let e = &answer["error"];
        let code = e["code"].as_str().unwrap_or("failed");
        v["error"] = json!({
            "code": code,
            "message": e["message"].as_str().unwrap_or(""),
            "retryable": e["retryable"].as_bool().unwrap_or(RETRYABLE.contains(&code)),
            "revision": e["revision"].as_u64().unwrap_or(0),
        });
    }
    Some(v)
}

pub fn og_platform_result(world: u32, request: u32) -> String {
    with(world, |p| {
        p.log
            .get(&request)
            .and_then(|r| result_of(request, r))
            .unwrap_or(Value::Null)
    })
    .to_string()
}

pub fn og_platform_info(world: u32) -> String {
    with(world, |p| {
        json!({
            "host": p.host.clone().unwrap_or_else(|| "none".into()),
            "capabilities": p.capabilities,
            "pending": p.log.values().filter(|r| r.answer.is_none() && !r.oneway).count(),
        })
    })
    .to_string()
}

// ── the host's side (lifecycle members) ─────────────────────────────────

/// The calls made since the last take: `[{ request, method, args, tick }]`.
pub fn og_platform_take(engine: u32) -> String {
    with(engine, |p| {
        let ids = std::mem::take(&mut p.outbox);
        Value::Array(
            ids.into_iter()
                .filter_map(|id| {
                    let r = p.log.get(&id)?;
                    Some(json!({ "request": id, "method": r.method, "args": r.args, "tick": r.tick }))
                })
                .collect(),
        )
    })
    .to_string()
}

/// Answers one call; applied at the next tick's start. False for an
/// unknown, one-way or already answered request, or a malformed answer.
pub fn og_platform_answer(engine: u32, request: u32, answer: String) -> bool {
    let Ok(v) = serde_json::from_str::<Value>(&answer) else {
        return false;
    };
    if v["ok"].as_bool().is_none() {
        return false;
    }
    with(engine, |p| {
        let open = p
            .log
            .get(&request)
            .is_some_and(|r| r.answer.is_none() && !r.oneway)
            && !p.inbox.iter().any(|(id, _)| *id == request);
        if open {
            p.inbox.push((request, v));
        }
        open
    })
}

/// What answers this engine's calls: `{ host, capabilities }`.
pub fn og_platform_set_host(engine: u32, info: String) {
    let v: Value = serde_json::from_str(&info).unwrap_or(Value::Null);
    with(engine, |p| {
        p.host = v["host"].as_str().map(str::to_string);
        p.capabilities = v["capabilities"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|c| c.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
    });
}

// ── the system and the probe ─────────────────────────────────────────────

/// Registers og.PlatformAnswered and the og.platform system (front).
pub(crate) fn mount(world: &mut World, engine_id: u32) -> Result<(), String> {
    world
        .register::<PlatformAnswered>()
        .map_err(|e| e.reason().to_string())?;
    world
        .add_system(
            SystemDesc::new(PLATFORM_SYSTEM).write::<PlatformAnswered>(),
            move |ctx| {
                let tick = ctx.time().tick;
                let arrived = with(engine_id, |p| {
                    let inbox = std::mem::take(&mut p.inbox);
                    inbox
                        .into_iter()
                        .filter_map(|(id, v)| {
                            let r = p.log.get_mut(&id)?;
                            let ok = v["ok"].as_bool() == Some(true);
                            r.answer = Some((v, tick));
                            Some((id, ok))
                        })
                        .collect::<Vec<_>>()
                });
                for (request, ok) in arrived {
                    ctx.send(PlatformAnswered { request, ok })?;
                }
                Ok(())
            },
        )
        .map_err(|e| e.reason().to_string())
}

pub(crate) fn retire(engine: u32) {
    PLATFORMS.with(|p| p.borrow_mut().remove(&engine));
}

/// `dump_platform`'s JSON: the host, its capabilities and every call.
pub(crate) fn dump(engine: u32) -> Value {
    with(engine, |p| {
        json!({
            "host": p.host.clone().unwrap_or_else(|| "none".into()),
            "capabilities": p.capabilities,
            "calls": p.log.iter().map(|(id, r)| {
                let mut c = json!({ "request": id, "method": r.method, "args": r.args, "tick": r.tick, "oneway": r.oneway });
                match (&r.answer, r.oneway) {
                    (_, true) => c["state"] = json!("sent"),
                    (None, _) => c["state"] = json!("pending"),
                    (Some(_), _) => {
                        c["state"] = json!("answered");
                        c["result"] = result_of(*id, r).unwrap_or(Value::Null);
                    }
                }
                c
            }).collect::<Vec<_>>(),
        })
    })
}
