//! Save slots (saveSlot, loadSlot): a save split into parts under a manifest, written and read back as a chain of store calls.

use super::*;

/// The longest slot name: room for a part's suffix (`.b7`) in a key.
pub(super) const MAX_SLOT_NAME: usize = 60;

/// The most parts a slot's text takes, and so the longest text: 8 of a
/// key's 64 KiB, 512 KiB, so a slot's two banks fit the platform's 1 MiB
/// a player and game (a 5,000-entity world, half of it physics bodies,
/// saves to about 437 KiB; past that, a selection keeps what the game
/// needs in a few KiB).
pub(super) const MAX_PARTS: usize = 8;

/// The largest `meta`, encoded.
pub(super) const MAX_META: usize = 4 * 1024;

/// What a manifest says it is.
pub(super) const MANIFEST: &str = "og.slot";

pub(super) fn valid_slot(slot: &str) -> Result<(), String> {
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
pub(super) fn split(text: &str) -> Vec<&str> {
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

pub(super) fn checksum(text: &str) -> String {
    format!("{:016x}", og_core::hash::hash_bytes(text.as_bytes()))
}

pub(super) fn part_key(slot: &str, bank: &str, i: usize) -> String {
    format!("{slot}.{bank}{i}")
}

pub(super) fn start_slot(
    world: u32,
    method: &str,
    slot: &str,
    args: Value,
    op: SlotOp,
) -> Result<u32, String> {
    let tick = tick_now(world)?;
    with(world, |p| {
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
    })
}

pub fn og_platform_save_slot(world: u32, desc: Option<String>) -> Result<u32, String> {
    let d: crate::host::wire::SaveSlotDesc = crate::host::wire::arg(desc.as_deref(), "desc")?;
    let slot = d.slot;
    valid_slot(&slot)?;
    let text = d.text;
    let parts = split(&text).len();
    if parts > MAX_PARTS {
        return Err(format!(
            "the text is {} bytes, {parts} parts of a store key's 64 KiB; a slot holds at most {MAX_PARTS} (512 KiB): save a selection (world.snapshot's SnapshotDesc) of what the game keeps",
            text.len()
        ));
    }
    let meta = d.meta;
    let meta_size = meta.to_string().len();
    if meta_size > MAX_META {
        return Err(format!(
            "the meta is {meta_size} bytes encoded; a slot's meta is at most {MAX_META}"
        ));
    }
    let args = json!({ "slot": slot, "bytes": text.len(), "parts": parts, "meta": meta });
    start_slot(world, "slot.save", &slot, args, SlotOp::Save { text, meta })
}

pub fn og_platform_load_slot(world: u32, desc: Option<String>) -> Result<u32, String> {
    let d: crate::host::wire::LoadSlotDesc = crate::host::wire::arg(desc.as_deref(), "desc")?;
    let slot = d.slot;
    valid_slot(&slot)?;
    let meta_only = d.meta_only;
    let args = json!({ "slot": slot, "metaOnly": meta_only });
    start_slot(world, "slot.load", &slot, args, SlotOp::Load { meta_only })
}

pub(super) fn failure(code: &str, message: String) -> Value {
    json!({ "ok": false, "error": { "code": code, "message": message, "retryable": RETRYABLE.contains(&code) } })
}

/// A store call's failure as the slot call's, with the slot named.
pub(super) fn failed_with(answer: &Value, what: &str) -> Value {
    let mut a = answer.clone();
    let e = &mut a["error"];
    let message = e["message"].as_str().unwrap_or("").to_string();
    e["message"] = json!(format!("{what}: {message}"));
    a
}

/// Advances the slot call that made store call `sub`, now answered with
/// `answer`: Some(the slot call and its answer) when it is done.
pub(super) fn advance_slot(
    p: &mut Platform,
    sub: u32,
    answer: &Value,
    tick: u64,
) -> Option<(u32, Value)> {
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
