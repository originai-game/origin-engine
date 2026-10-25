//! The IDL's `platform` module (docs/design.md §2.8): calls game code makes
//! by method name, answered by the host. A call is queued with a request
//! number; the host takes the queue (`og_platform_take`), answers each call
//! (`og_platform_answer`: the web shell through the Player, the native host
//! from its local JSON store), and the answers are applied by the
//! `og.platform` system at the start of the next tick, which sends
//! `og.PlatformAnswered`. Every call and answer stays in the engine's log
//! for `og.platform.result` and the probe's `dump_platform`.
//!
//! Time: the host tells the engine its wall clock (UTC ms) when it attaches
//! and with every answer (`time`); `og.platform.now` is the last of those,
//! moved on by the ticks since, so game code reads a wall clock only as
//! host input, which a replay with the same answers repeats.

use super::engine::{engine, running};
use super::{ok, ok_void, parse, refuse};
use og_core::IdMap;
use og_core::Reflect;
use og_ecs::{SystemDesc, World};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::BTreeSet;

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
/// The most calls waiting for an answer: a call past it is refused, so
/// unanswered calls (which the log cannot drop) stay under [`MAX_LOG`].
const MAX_PENDING: usize = 256;
/// The largest stored value, encoded.
const MAX_VALUE: usize = 64 * 1024;
/// The largest answer a host may give, encoded: a stored value or a
/// leaderboard page with room to spare. A longer one answers its call
/// with a `too_large` error.
const MAX_ANSWER: usize = 4 * MAX_VALUE;

const RETRYABLE: &[&str] = &["offline", "timeout", "unavailable", "rate_limited"];

struct Record {
    method: String,
    args: Value,
    tick: u64,
    /// `{ ok, data }` or `{ ok: false, error }`, and the tick it was applied at.
    answer: Option<(Value, u64)>,
    /// Fire and forget (telemetry): never answered.
    oneway: bool,
    /// A slot call (saveSlot, loadSlot): answered by the engine when its
    /// store calls are, never sent to the host.
    internal: bool,
}

/// A slot call in progress: the store calls it made and what they answered.
struct SlotCall {
    slot: String,
    stage: Stage,
}

enum Stage {
    /// Both: the manifest's store.get is in flight.
    Manifest(SlotOp),
    /// saveSlot: the parts' store.set calls left, then the manifest to write.
    Parts {
        left: BTreeSet<u32>,
        manifest: Value,
        revision: u64,
    },
    /// saveSlot: the manifest's store.set.
    Write { parts: usize, bytes: usize },
    /// loadSlot: the parts' store.get calls, in order, and their texts.
    Read {
        parts: Vec<u32>,
        got: IdMap<u32, String>,
        manifest: Value,
        revision: u64,
    },
}

enum SlotOp {
    Save { text: String, meta: Value },
    Load { meta_only: bool },
}

#[derive(Default)]
struct Platform {
    next: u32,
    outbox: Vec<u32>,
    inbox: Vec<(u32, Value)>,
    log: IdMap<u32, Record>,
    host: Option<String>,
    capabilities: Vec<String>,
    /// PlatformInfo.clock: whose time `now` reads.
    clock: Option<String>,
    /// The host's last time (UTC ms) and the tick it held at: `now` moves
    /// it on by the ticks since. Set when the host attaches and by every
    /// answer that carries a time, at the tick's start the answer is applied.
    base: Option<(f64, u64)>,
    /// Slot calls in progress by request, and the store calls each made.
    slots: IdMap<u32, SlotCall>,
    sub: IdMap<u32, u32>,
}

thread_local! {
    static PLATFORMS: RefCell<IdMap<u32, Platform>> = const { RefCell::new(IdMap::new()) };
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
    with(world, |p| issue(p, method, args, tick, oneway, false))
}

/// Logs a call and queues it for the host (an internal one is the
/// engine's to answer and is not sent).
fn issue(
    p: &mut Platform,
    method: &str,
    args: Value,
    tick: u64,
    oneway: bool,
    internal: bool,
) -> Result<u32, String> {
    if !oneway {
        let waiting = p
            .log
            .values()
            .filter(|r| r.answer.is_none() && !r.oneway)
            .count();
        if waiting >= MAX_PENDING {
            return Err(format!(
                "{method}: {waiting} platform calls are waiting for the host's answers; at most {MAX_PENDING} wait at once"
            ));
        }
    }
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
            internal,
        },
    );
    if !internal {
        p.outbox.push(id);
    }
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
        // A one-way call dropped before the host took it is not sent.
        let log = &p.log;
        p.outbox.retain(|id| log.contains_key(id));
    }
    Ok(id)
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

// ── save slots (saveSlot, loadSlot) ──────────────────────────────────────

/// The longest slot name: room for a part's suffix (`.b7`) in a key.
const MAX_SLOT_NAME: usize = 60;
/// The most parts a slot's text takes, and so the longest text.
const MAX_PARTS: usize = 8;
/// The largest `meta`, encoded.
const MAX_META: usize = 4 * 1024;
/// What a manifest says it is.
const MANIFEST: &str = "og.slot";

fn valid_slot(slot: &str) -> Result<(), String> {
    if slot.len() <= MAX_SLOT_NAME && valid_key(slot) {
        Ok(())
    } else {
        Err(format!(
            "'{slot}' is not a slot name: 1 to {MAX_SLOT_NAME} of letters, digits, '.', '_' and '-'"
        ))
    }
}

/// `text` cut on character boundaries into parts whose JSON string is at
/// most a store value.
fn split(text: &str) -> Vec<&str> {
    let budget = MAX_VALUE - 2;
    let mut parts = Vec::new();
    let (mut start, mut size) = (0, 0);
    for (at, c) in text.char_indices() {
        let n = match c {
            '"' | '\\' | '\n' | '\r' | '\t' | '\u{8}' | '\u{c}' => 2,
            c if (c as u32) < 0x20 => 6,
            c => c.len_utf8(),
        };
        if size + n > budget {
            parts.push(&text[start..at]);
            (start, size) = (at, 0);
        }
        size += n;
    }
    if start < text.len() || parts.is_empty() {
        parts.push(&text[start..]);
    }
    parts
}

fn checksum(text: &str) -> String {
    format!("{:016x}", og_core::hash::hash_bytes(text.as_bytes()))
}

fn part_key(slot: &str, bank: &str, i: usize) -> String {
    format!("{slot}.{bank}{i}")
}

fn start_slot(world: u32, method: &str, slot: &str, args: Value, op: SlotOp) -> String {
    let tick = match tick_now(world) {
        Ok(t) => t,
        Err(e) => return refuse(e),
    };
    let started = with(world, |p| {
        let outer = issue(p, method, args, tick, false, true)?;
        let get = match issue(p, "store.get", json!({ "key": slot }), tick, false, false) {
            Ok(id) => id,
            Err(e) => {
                p.log.remove(&outer);
                return Err(e);
            }
        };
        p.sub.insert(get, outer);
        p.slots.insert(
            outer,
            SlotCall {
                slot: slot.to_string(),
                stage: Stage::Manifest(op),
            },
        );
        Ok(outer)
    });
    match started {
        Ok(id) => ok(json!(id)),
        Err(e) => refuse(e),
    }
}

pub fn og_platform_save_slot(world: u32, desc: Option<String>) -> String {
    let d = match parse(desc.as_deref()) {
        Ok(d) => d,
        Err(e) => return refuse(e),
    };
    let slot = d["slot"].as_str().unwrap_or("").to_string();
    if let Err(e) = valid_slot(&slot) {
        return refuse(e);
    }
    let text = d["text"].as_str().unwrap_or("").to_string();
    let parts = split(&text).len();
    if parts > MAX_PARTS {
        return refuse(format!(
            "the text is {} bytes, {parts} parts of a store key's 64 KiB; a slot holds at most {MAX_PARTS} (512 KiB)",
            text.len()
        ));
    }
    let meta = d.get("meta").cloned().unwrap_or(Value::Null);
    let meta_size = meta.to_string().len();
    if meta_size > MAX_META {
        return refuse(format!(
            "the meta is {meta_size} bytes encoded; a slot's meta is at most {MAX_META}"
        ));
    }
    let args = json!({ "slot": slot, "bytes": text.len(), "parts": parts, "meta": meta });
    start_slot(world, "slot.save", &slot, args, SlotOp::Save { text, meta })
}

pub fn og_platform_load_slot(world: u32, desc: Option<String>) -> String {
    let d = match parse(desc.as_deref()) {
        Ok(d) => d,
        Err(e) => return refuse(e),
    };
    let slot = d["slot"].as_str().unwrap_or("").to_string();
    if let Err(e) = valid_slot(&slot) {
        return refuse(e);
    }
    let meta_only = d["metaOnly"].as_bool().unwrap_or(false);
    let args = json!({ "slot": slot, "metaOnly": meta_only });
    start_slot(world, "slot.load", &slot, args, SlotOp::Load { meta_only })
}

fn failure(code: &str, message: String) -> Value {
    json!({ "ok": false, "error": { "code": code, "message": message, "retryable": RETRYABLE.contains(&code) } })
}

/// A store call's failure as the slot call's, with the slot named.
fn failed_with(answer: &Value, what: &str) -> Value {
    let mut a = answer.clone();
    let e = &mut a["error"];
    let message = e["message"].as_str().unwrap_or("").to_string();
    e["message"] = json!(format!("{what}: {message}"));
    a
}

/// Advances the slot call that made store call `sub`, now answered with
/// `answer`: Some(the slot call and its answer) when it is done.
fn advance_slot(p: &mut Platform, sub: u32, answer: &Value, tick: u64) -> Option<(u32, Value)> {
    let outer = p.sub.remove(&sub)?;
    let mut call = p.slots.remove(&outer)?;
    let ok = answer["ok"].as_bool() == Some(true);
    let slot = call.slot.clone();
    let done = |v: Value| Some((outer, v));
    match call.stage {
        Stage::Manifest(op) => {
            if !ok {
                return done(failed_with(answer, &format!("reading slot '{slot}'")));
            }
            let value = answer["data"]["value"].clone();
            let revision = answer["data"]["revision"].as_u64().unwrap_or(0);
            let is_manifest = value[MANIFEST].as_u64() == Some(1);
            match op {
                SlotOp::Load { meta_only } => {
                    if value.is_null() {
                        return done(json!({ "ok": true, "data": {
                            "slot": slot, "text": null, "meta": null, "parts": 0, "bytes": 0, "revision": 0,
                        } }));
                    }
                    if !is_manifest {
                        return done(failure(
                            "invalid",
                            format!(
                                "the key '{slot}' holds no slot manifest: write the slot with og.platform.saveSlot"
                            ),
                        ));
                    }
                    let parts = value["parts"].as_u64().unwrap_or(0) as usize;
                    let bank = value["bank"].as_str().unwrap_or("a").to_string();
                    if meta_only || parts == 0 {
                        return done(json!({ "ok": true, "data": {
                            "slot": slot, "text": if meta_only { Value::Null } else { json!("") },
                            "meta": value["meta"].clone(), "parts": parts,
                            "bytes": value["bytes"].as_u64().unwrap_or(0), "revision": revision,
                        } }));
                    }
                    let mut subs = Vec::new();
                    for i in 0..parts.min(MAX_PARTS) {
                        match issue(
                            p,
                            "store.get",
                            json!({ "key": part_key(&slot, &bank, i) }),
                            tick,
                            false,
                            false,
                        ) {
                            Ok(id) => {
                                p.sub.insert(id, outer);
                                subs.push(id);
                            }
                            Err(e) => return done(failure("failed", e)),
                        }
                    }
                    call.stage = Stage::Read {
                        parts: subs,
                        got: IdMap::new(),
                        manifest: value,
                        revision,
                    };
                }
                SlotOp::Save { text, meta } => {
                    if !value.is_null() && !is_manifest {
                        return done(failure(
                            "invalid",
                            format!(
                                "the key '{slot}' holds something else than a slot: save the slot under another name"
                            ),
                        ));
                    }
                    let bank = if value["bank"].as_str() == Some("a") {
                        "b"
                    } else {
                        "a"
                    };
                    let parts = split(&text);
                    let mut left = BTreeSet::new();
                    for (i, part) in parts.iter().enumerate() {
                        let args = json!({ "key": part_key(&slot, bank, i), "value": part, "revision": 0, "create": false });
                        match issue(p, "store.set", args, tick, false, false) {
                            Ok(id) => {
                                p.sub.insert(id, outer);
                                left.insert(id);
                            }
                            Err(e) => return done(failure("failed", e)),
                        }
                    }
                    let manifest = json!({
                        (MANIFEST): 1, "bank": bank, "parts": parts.len(), "bytes": text.len(),
                        "hash": checksum(&text), "meta": meta,
                    });
                    call.stage = Stage::Parts {
                        left,
                        manifest,
                        revision,
                    };
                }
            }
        }
        Stage::Parts {
            mut left,
            manifest,
            revision,
        } => {
            left.remove(&sub);
            if !ok {
                // The parts still in flight are no one's now.
                for id in &left {
                    p.sub.remove(id);
                }
                return done(failed_with(answer, &format!("writing slot '{slot}'")));
            }
            if left.is_empty() {
                // At the revision read: another save landing meanwhile is a
                // conflict, and a slot never written must still be new.
                let args = json!({ "key": slot, "value": manifest, "revision": revision, "create": revision == 0 });
                let parts = manifest["parts"].as_u64().unwrap_or(0) as usize;
                let bytes = manifest["bytes"].as_u64().unwrap_or(0) as usize;
                match issue(p, "store.set", args, tick, false, false) {
                    Ok(id) => {
                        p.sub.insert(id, outer);
                        call.stage = Stage::Write { parts, bytes };
                    }
                    Err(e) => return done(failure("failed", e)),
                }
            } else {
                call.stage = Stage::Parts {
                    left,
                    manifest,
                    revision,
                };
            }
        }
        Stage::Write { parts, bytes } => {
            if !ok {
                let mut a = failed_with(answer, &format!("writing slot '{slot}'"));
                if a["error"]["code"].as_str() == Some("conflict") {
                    a["error"]["message"] = json!(format!(
                        "another save of slot '{slot}' landed while this one was written: the slot holds that one; load it, or save again"
                    ));
                }
                return done(a);
            }
            return done(json!({ "ok": true, "data": {
                "slot": slot, "parts": parts, "bytes": bytes,
                "revision": answer["data"]["revision"].as_u64().unwrap_or(0),
            } }));
        }
        Stage::Read {
            parts,
            mut got,
            manifest,
            revision,
        } => {
            if !ok {
                for id in &parts {
                    p.sub.remove(id);
                }
                return done(failed_with(answer, &format!("reading slot '{slot}'")));
            }
            got.insert(
                sub,
                answer["data"]["value"].as_str().unwrap_or("").to_string(),
            );
            if got.len() < parts.len() {
                call.stage = Stage::Read {
                    parts,
                    got,
                    manifest,
                    revision,
                };
            } else {
                let text: String = parts
                    .iter()
                    .filter_map(|id| got.get(id).map(String::as_str))
                    .collect();
                let whole = Some(text.len() as u64) == manifest["bytes"].as_u64()
                    && manifest["hash"].as_str() == Some(checksum(&text).as_str());
                if !whole {
                    return done(failure(
                        "invalid",
                        format!(
                            "slot '{slot}''s parts do not match its manifest (two devices saving it at once): save the slot again"
                        ),
                    ));
                }
                return done(json!({ "ok": true, "data": {
                    "slot": slot, "text": text, "meta": manifest["meta"].clone(),
                    "parts": parts.len(), "bytes": text.len(), "revision": revision,
                } }));
            }
        }
    }
    p.slots.insert(outer, call);
    None
}

/// An answer's or a hello's time: finite UTC milliseconds above 0.
fn time_of(v: &Value) -> Option<f64> {
    v["time"].as_f64().filter(|t| t.is_finite() && *t > 0.0)
}

fn result_of(id: u32, r: &Record) -> Option<Value> {
    let (answer, tick) = r.answer.as_ref()?;
    let ok = answer["ok"].as_bool() == Some(true);
    let mut v = json!({
        "request": id, "method": r.method, "ok": ok,
        "data": if ok { answer.get("data").cloned().unwrap_or(Value::Null) } else { Value::Null },
        "tick": tick,
        "time": time_of(answer).unwrap_or(0.0),
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
            "clock": p.clock.clone().unwrap_or_else(|| "none".into()),
        })
    })
    .to_string()
}

/// The fixed tick's length in milliseconds of the engine's world.
fn tick_ms(world: u32) -> f64 {
    engine(world).map_or(1000.0 / 60.0, |e| {
        1000.0 / f64::from(e.state.borrow().rate.max(1))
    })
}

/// Moves the clock's base to a time the host gave at `tick`. The same
/// clock never goes back: a time older than the base moved on by the ticks
/// since (a harness that advanced faster than the host's clock, or answers
/// stamped by the Player and by the browser in turn) keeps the moved-on
/// base. A different clock (`clock` names another kind) starts afresh.
fn rebase(p: &mut Platform, time: f64, tick: u64, clock: Option<&str>, tick_ms: f64) {
    let switched = clock.is_some_and(|c| p.clock.as_deref() != Some(c));
    let time = match p.base {
        Some((t, at)) if !switched => time.max(t + tick.saturating_sub(at) as f64 * tick_ms),
        _ => time,
    };
    p.base = Some((time, tick));
}

/// og.Time jumped from tick `from` to `to` (a game restored a save): the
/// clock keeps the time it had, moving on from `to`.
pub(crate) fn shift_ticks(engine: u32, from: u64, to: u64) {
    if from == to {
        return;
    }
    let ms = tick_ms(engine);
    with(engine, |p| {
        if let Some((time, at)) = p.base {
            p.base = Some((time + from.saturating_sub(at) as f64 * ms, to));
        }
    });
}

pub fn og_platform_now(world: u32) -> f64 {
    let Some((time, at)) = with(world, |p| p.base) else {
        return 0.0;
    };
    let tick = tick_now(world).unwrap_or(at);
    time + tick.saturating_sub(at) as f64 * tick_ms(world)
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
/// An answer longer than [`MAX_ANSWER`] answers the call with a
/// `too_large` error instead.
pub fn og_platform_answer(engine: u32, request: u32, answer: String) -> bool {
    let v = if answer.len() > MAX_ANSWER {
        json!({ "ok": false, "error": { "code": "too_large", "message": format!(
            "the host's answer is {} bytes encoded; an answer is at most {MAX_ANSWER}",
            answer.len()
        ) } })
    } else {
        let Ok(v) = serde_json::from_str::<Value>(&answer) else {
            return false;
        };
        v
    };
    if v["ok"].as_bool().is_none() {
        return false;
    }
    with(engine, |p| {
        let open = p
            .log
            .get(&request)
            .is_some_and(|r| r.answer.is_none() && !r.oneway && !r.internal)
            && !p.inbox.iter().any(|(id, _)| *id == request);
        if open {
            p.inbox.push((request, v));
        }
        open
    })
}

/// What answers this engine's calls: `{ host, capabilities, time?, clock? }`.
/// The time (UTC ms) is the host's now, between ticks: `now` reads it from
/// the current tick on.
pub fn og_platform_set_host(engine: u32, info: String) {
    let v: Value = serde_json::from_str(&info).unwrap_or(Value::Null);
    let tick = tick_now(engine).unwrap_or(0);
    let ms = tick_ms(engine);
    with(engine, |p| {
        if let Some(t) = time_of(&v) {
            rebase(p, t, tick, v["clock"].as_str(), ms);
        }
        if let Some(c) = v["clock"].as_str() {
            p.clock = Some(c.to_string());
        }
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
                let ms = tick_ms(engine_id);
                let arrived = with(engine_id, |p| {
                    let inbox = std::mem::take(&mut p.inbox);
                    let mut arrived = Vec::new();
                    for (id, v) in inbox {
                        if !p.log.contains_key(&id) {
                            continue;
                        }
                        let ok = v["ok"].as_bool() == Some(true);
                        if let Some(t) = time_of(&v) {
                            rebase(p, t, tick, v["clock"].as_str(), ms);
                        }
                        if let Some(c) = v["clock"].as_str() {
                            p.clock = Some(c.to_string());
                        }
                        // A slot call's store call: the slot call moves on,
                        // and only its own answer is sent.
                        let slot = p.sub.contains_key(&id);
                        let done = slot.then(|| advance_slot(p, id, &v, tick)).flatten();
                        let host_time = time_of(&v);
                        if let Some(r) = p.log.get_mut(&id) {
                            r.answer = Some((v, tick));
                        }
                        if let Some((outer, mut answer)) = done {
                            // Every answer carries the host's time (og-platform-sdk
                            // §1): a slot call's, that of the store answer that
                            // ended it, or the clock moved on to this tick.
                            if time_of(&answer).is_none()
                                && let Some(t) = host_time.or_else(|| {
                                    p.base
                                        .map(|(t, at)| t + tick.saturating_sub(at) as f64 * ms)
                                })
                            {
                                answer["time"] = json!(t);
                            }
                            let ok = answer["ok"].as_bool() == Some(true);
                            if let Some(r) = p.log.get_mut(&outer) {
                                r.answer = Some((answer, tick));
                            }
                            arrived.push((outer, ok));
                        }
                        if !slot {
                            arrived.push((id, ok));
                        }
                    }
                    arrived
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn value(answer: &str) -> Value {
        let v: Value = serde_json::from_str(answer).expect("JSON");
        assert_eq!(v["ok"], json!(true), "{answer}");
        v["value"].clone()
    }

    fn engine_at_60() -> u32 {
        let game = crate::og_game_game(Some(json!({ "tick": 60 }).to_string()));
        value(&crate::og_engine_create(game, "{}".into()))
            .as_u64()
            .expect("an engine id") as u32
    }

    fn host(engine: u32, time: f64, clock: &str) {
        og_platform_set_host(
            engine,
            json!({ "host": "test", "capabilities": ["store"], "time": time, "clock": clock })
                .to_string(),
        );
    }

    fn advance(engine: u32, steps: u32) {
        value(&crate::og_engine_advance(engine, steps));
    }

    #[test]
    fn now_never_goes_back_on_one_clock_and_starts_afresh_on_another() {
        let e = engine_at_60();
        host(e, 1_000_000.0, "browser");
        assert_eq!(og_platform_now(e), 1_000_000.0);
        // A harness advance runs ahead of the host's clock.
        advance(e, 600);
        let ahead = og_platform_now(e);
        assert!(ahead >= 1_009_999.0, "{ahead}");
        // The host retells an older time on the same clock.
        host(e, 1_000_500.0, "browser");
        let retold = og_platform_now(e);
        assert!(
            retold >= ahead,
            "set_host moved now back: {ahead} → {retold}"
        );
        // An answer stamped earlier still (browser-stamped: no clock).
        let id = value(&og_platform_store_get(e, "k".into()))
            .as_u64()
            .unwrap() as u32;
        assert!(og_platform_answer(
            e,
            id,
            json!({ "ok": true, "data": null, "time": 1_000_600.0 }).to_string()
        ));
        advance(e, 1);
        let answered = og_platform_now(e);
        assert!(
            answered >= retold,
            "an answer moved now back: {retold} → {answered}"
        );
        let result: Value = serde_json::from_str(&og_platform_result(e, id)).unwrap();
        assert_eq!(result["ok"], json!(true), "{result}");
        // Another clock is another time line: it is taken as told.
        host(e, 5.0, "fixed");
        assert_eq!(og_platform_now(e), 5.0);
    }

    #[test]
    fn calls_waiting_and_answers_are_bounded() {
        let e = engine_at_60();
        host(e, 1.0, "fixed");
        let first = value(&og_platform_store_get(e, "k".into()))
            .as_u64()
            .unwrap() as u32;
        // An answer past MAX_ANSWER answers its call with too_large.
        let long = json!({ "ok": true, "data": "x".repeat(MAX_ANSWER) }).to_string();
        assert!(og_platform_answer(e, first, long));
        advance(e, 1);
        let result: Value = serde_json::from_str(&og_platform_result(e, first)).unwrap();
        assert_eq!(result["ok"], json!(false), "{result}");
        assert_eq!(result["error"]["code"], json!("too_large"), "{result}");
        // MAX_PENDING calls wait; the next is refused by name.
        for _ in 0..MAX_PENDING {
            value(&og_platform_store_get(e, "k".into()));
        }
        let refused: Value = serde_json::from_str(&og_platform_store_get(e, "k".into())).unwrap();
        assert_eq!(refused["ok"], json!(false), "{refused}");
        let reason = refused["reason"].as_str().unwrap_or_default();
        assert!(reason.contains("waiting"), "{reason}");
        assert_eq!(og_platform_store_list(e), 0);
        // One-way telemetry is not held back by them.
        let sent: Value =
            serde_json::from_str(&og_platform_telemetry(e, "t".into(), None)).unwrap();
        assert_eq!(sent["ok"], json!(true), "{sent}");
    }

    /// A store as a host keeps it: key → (value, revision). Answers every
    /// call taken, except the store.set calls `fail` names by key.
    fn serve(e: u32, store: &mut BTreeMap<String, (Value, u64)>, fail: &[&str]) -> usize {
        let calls: Vec<Value> = serde_json::from_str(&og_platform_take(e)).unwrap();
        for c in &calls {
            let id = c["request"].as_u64().unwrap() as u32;
            let a = &c["args"];
            let key = a["key"].as_str().unwrap_or("").to_string();
            let answer = match c["method"].as_str().unwrap() {
                "store.get" => {
                    let (value, revision) = store.get(&key).cloned().unwrap_or((Value::Null, 0));
                    json!({ "ok": true, "data": { "key": key, "value": value, "revision": revision } })
                }
                "store.set" if fail.contains(&key.as_str()) => {
                    json!({ "ok": false, "error": { "code": "offline", "message": "no network" } })
                }
                "store.set" => {
                    let at = store.get(&key).map_or(0, |v| v.1);
                    let create = a["create"].as_bool() == Some(true);
                    let revision = a["revision"].as_u64().unwrap_or(0);
                    if (create && at != 0) || (!create && revision != 0 && revision != at) {
                        json!({ "ok": false, "error": { "code": "conflict", "message": "moved", "revision": at } })
                    } else {
                        store.insert(key.clone(), (a["value"].clone(), at + 1));
                        json!({ "ok": true, "data": { "key": key, "revision": at + 1 } })
                    }
                }
                m => panic!("unexpected {m}"),
            };
            assert!(og_platform_answer(e, id, answer.to_string()));
        }
        calls.len()
    }

    /// [`serve`], every answer stamped with the host's `time` as og run's
    /// host does.
    fn serve_at(e: u32, store: &mut BTreeMap<String, (Value, u64)>, time: f64) {
        let calls: Vec<Value> = serde_json::from_str(&og_platform_take(e)).unwrap();
        for c in &calls {
            let id = c["request"].as_u64().unwrap() as u32;
            let a = &c["args"];
            let key = a["key"].as_str().unwrap_or("").to_string();
            let data = match c["method"].as_str().unwrap() {
                "store.get" => {
                    let (value, revision) = store.get(&key).cloned().unwrap_or((Value::Null, 0));
                    json!({ "key": key, "value": value, "revision": revision })
                }
                "store.set" => {
                    let at = store.get(&key).map_or(0, |v| v.1);
                    store.insert(key.clone(), (a["value"].clone(), at + 1));
                    json!({ "key": key, "revision": at + 1 })
                }
                m => panic!("unexpected {m}"),
            };
            let answer = json!({ "ok": true, "data": data, "time": time });
            assert!(og_platform_answer(e, id, answer.to_string()));
        }
    }

    #[test]
    fn a_slot_calls_answer_carries_the_hosts_time_as_a_store_answer_does() {
        // brasswick FINDINGS #7: under og run, store.get answered the
        // host's clock while slot.load and slot.save answered time 0.
        let e = engine_at_60();
        host(e, 1_000_000.0, "fixed");
        let mut store = BTreeMap::new();
        let mut settle_at = |id: u32, time: f64| {
            for _ in 0..8 {
                serve_at(e, &mut store, time);
                advance(e, 1);
                let r: Value = serde_json::from_str(&og_platform_result(e, id)).unwrap();
                if !r.is_null() {
                    return r;
                }
            }
            panic!("request {id} was never answered");
        };
        let get = request_id(&og_platform_store_get(e, "progress".into()));
        assert_eq!(settle_at(get, 1_000_500.0)["time"], json!(1_000_500.0));
        let save = request_id(&og_platform_save_slot(
            e,
            Some(
                json!({ "slot": "slot1", "text": "og-save:1:abc", "meta": { "at": 1 } })
                    .to_string(),
            ),
        ));
        let r = settle_at(save, 1_002_000.0);
        assert_eq!(r["ok"], json!(true), "{r}");
        assert_eq!(r["time"], json!(1_002_000.0), "slot.save: {r}");
        for meta_only in [true, false] {
            let load = request_id(&og_platform_load_slot(
                e,
                Some(json!({ "slot": "slot1", "metaOnly": meta_only }).to_string()),
            ));
            let r = settle_at(load, 1_003_000.0);
            assert_eq!(r["ok"], json!(true), "{r}");
            assert_eq!(
                r["time"],
                json!(1_003_000.0),
                "slot.load (metaOnly {meta_only}): {r}"
            );
        }
    }

    /// Serves and ticks until request `id` is answered.
    fn settle(e: u32, id: u32, store: &mut BTreeMap<String, (Value, u64)>, fail: &[&str]) -> Value {
        for _ in 0..8 {
            serve(e, store, fail);
            advance(e, 1);
            let r: Value = serde_json::from_str(&og_platform_result(e, id)).unwrap();
            if !r.is_null() {
                return r;
            }
        }
        panic!("request {id} was never answered");
    }

    fn request_id(answer: &str) -> u32 {
        value(answer).as_u64().unwrap() as u32
    }

    #[test]
    fn a_slot_takes_a_text_over_a_key_in_banks_and_a_save_cut_short_leaves_the_last_one() {
        let e = engine_at_60();
        host(e, 1.0, "fixed");
        let mut store = BTreeMap::new();
        // Never saved: no text, no meta.
        let id = request_id(&og_platform_load_slot(
            e,
            Some(json!({ "slot": "slot1" }).to_string()),
        ));
        let r = settle(e, id, &mut store, &[]);
        assert_eq!(r["ok"], json!(true), "{r}");
        assert_eq!(r["data"]["text"], Value::Null, "{r}");
        // 150 KB with quotes and non-ASCII: three parts, each a store value.
        let text: String = (0..150_000).map(|i| ['a', '"', '灯', 'z'][i % 4]).collect();
        let save = |e, text: &str, place: u32| {
            request_id(&og_platform_save_slot(
                e,
                Some(
                    json!({ "slot": "slot1", "text": text, "meta": { "place": place } })
                        .to_string(),
                ),
            ))
        };
        let r = settle(e, save(e, &text, 1), &mut store, &[]);
        assert_eq!(r["ok"], json!(true), "{r}");
        let parts = r["data"]["parts"].as_u64().unwrap();
        assert!(parts >= 3, "{r}");
        assert!(store.contains_key("slot1.a0") && !store.contains_key("slot1.b0"));
        for (k, (v, _)) in &store {
            assert!(v.to_string().len() <= MAX_VALUE, "{k} is over a key");
        }
        // Only the slot call's answer reaches the game as an event; its own
        // record was never sent to the host.
        let dump = dump(e);
        let slot_call = dump["calls"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["method"] == "slot.save")
            .unwrap();
        assert_eq!(slot_call["state"], json!("answered"));
        // A second save goes in the other bank; one of its parts refused,
        // the manifest still names the first, whole.
        let other: String = text.chars().rev().collect();
        let r = settle(e, save(e, &other, 2), &mut store, &["slot1.b1"]);
        assert_eq!(r["ok"], json!(false), "{r}");
        assert_eq!(r["error"]["code"], json!("offline"), "{r}");
        assert!(
            r["error"]["message"]
                .as_str()
                .unwrap()
                .contains("writing slot 'slot1'"),
            "{r}"
        );
        let id = request_id(&og_platform_load_slot(
            e,
            Some(json!({ "slot": "slot1" }).to_string()),
        ));
        let r = settle(e, id, &mut store, &[]);
        assert_eq!(r["data"]["text"].as_str(), Some(text.as_str()));
        assert_eq!(r["data"]["meta"], json!({ "place": 1 }));
        // Saved again in full: the b bank, and the meta alone reads without the parts.
        let r = settle(e, save(e, &other, 2), &mut store, &[]);
        assert_eq!(r["ok"], json!(true), "{r}");
        assert_eq!(store["slot1"].0["bank"], json!("b"));
        let id = request_id(&og_platform_load_slot(
            e,
            Some(json!({ "slot": "slot1", "metaOnly": true }).to_string()),
        ));
        let r = settle(e, id, &mut store, &[]);
        assert_eq!(
            [r["data"]["meta"].clone(), r["data"]["text"].clone()],
            [json!({ "place": 2 }), Value::Null]
        );
        let id = request_id(&og_platform_load_slot(
            e,
            Some(json!({ "slot": "slot1" }).to_string()),
        ));
        assert_eq!(
            settle(e, id, &mut store, &[])["data"]["text"].as_str(),
            Some(other.as_str())
        );
        // A part changed under the manifest is refused, not joined.
        store.get_mut("slot1.b0").unwrap().0 = json!("tampered");
        let id = request_id(&og_platform_load_slot(
            e,
            Some(json!({ "slot": "slot1" }).to_string()),
        ));
        let r = settle(e, id, &mut store, &[]);
        assert_eq!(r["error"]["code"], json!("invalid"), "{r}");
        // Refused at the call: a bad name, a text over eight parts.
        let bad: Value = serde_json::from_str(&og_platform_save_slot(
            e,
            Some(json!({ "slot": "a b", "text": "x" }).to_string()),
        ))
        .unwrap();
        assert_eq!(bad["ok"], json!(false));
        let huge = "x".repeat(MAX_PARTS * MAX_VALUE);
        let bad: Value = serde_json::from_str(&og_platform_save_slot(
            e,
            Some(json!({ "slot": "s", "text": huge }).to_string()),
        ))
        .unwrap();
        assert!(
            bad["reason"].as_str().unwrap().contains("at most 8"),
            "{bad}"
        );
    }
}
