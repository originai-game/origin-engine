//! A game's saves (IDL `World.snapshot`, `World.restore`): the whole world
//! as text a game puts in the platform store, and back.
//!
//! The text is `og-save:1:` and the base64 of a deflated payload: `OGSV`,
//! a `u32` version (1), the `u64` tick it was taken at and the `u64` FNV-1a
//! of the snapshot (a damaged save is refused), then the world snapshot (og-ecs's format, which carries every type's name,
//! layout hash and field table, so a save from a build whose types changed
//! is refused naming them, or mapped field by field with `migrate`).
//!
//! A save is the world; the session is not in it. Restoring keeps the
//! host's input (`og.Input`, `og.Pointers`, `og.Gamepad`) and the player's
//! bus volumes (`og.Audio`), and moves what lives outside the world on the clock with
//! `og.Time`'s jump: the probe's scheduled steps keep their remaining
//! delays, `og.platform.now` its time. Inside a system or a command (the
//! probe is mid-tick) the restore waits for the tick's end
//! ([`after_tick`]); elsewhere it applies at once. Either way the next
//! tick delivers og.WorldRestored ([`before_tick`]).

use super::engine::{Engine, engine, running};
use og_ecs::World;
use serde_json::Value;

const PREFIX: &str = "og-save:";
const MAGIC: &[u8; 4] = b"OGSV";
const VERSION: u32 = 1;
/// The largest save inflated: far over any world the 64 KiB store holds.
const MAX_INFLATED: usize = 64 << 20;

/// The resources a restore keeps from the session.
const SESSION: &[&str] = &["og.Input", "og.Pointers", "og.Gamepad", "og.Audio"];

/// A restore asked for inside a tick, applied when it ends.
pub(crate) struct Pending {
    snapshot: Vec<u8>,
    migrate: bool,
}

fn ok(value: Value) -> String {
    serde_json::json!({ "ok": true, "value": value }).to_string()
}

fn refuse(reason: impl Into<String>) -> String {
    serde_json::json!({ "ok": false, "reason": reason.into() }).to_string()
}

/// `World.snapshot`.
pub fn og_world_snapshot(world: u32) -> String {
    let eng = match engine(world) {
        Ok(e) => e,
        Err(e) => return refuse(e),
    };
    let (bytes, tick) = if let Some(f) = running(world) {
        // SAFETY: the frame is published while its system runs.
        let ctx = unsafe { f.ctx() };
        (ctx.snapshot(), ctx.time().tick)
    } else {
        match eng.world() {
            Ok(w) => (w.snapshot(), w.time().tick),
            Err(e) => return refuse(e),
        }
    };
    ok(Value::String(encode(&bytes, tick)))
}

/// The save text of a world snapshot.
fn encode(snapshot: &[u8], tick: u64) -> String {
    let hash = og_core::hash::hash_bytes(snapshot);
    let mut payload = Vec::with_capacity(snapshot.len() + 24);
    payload.extend_from_slice(MAGIC);
    payload.extend_from_slice(&VERSION.to_le_bytes());
    payload.extend_from_slice(&tick.to_le_bytes());
    payload.extend_from_slice(&hash.to_le_bytes());
    payload.extend_from_slice(snapshot);
    let deflated = miniz_oxide::deflate::compress_to_vec(&payload, 6);
    format!("{PREFIX}{VERSION}:{}", og_probe::base64::encode(&deflated))
}

/// What a save holds: the world snapshot.
fn decode(save: &str) -> Result<Vec<u8>, String> {
    let not_a_save = || {
        "this is not a save: world.restore takes the text world.snapshot returned (`og-save:1:...`), as it was stored".to_string()
    };
    let rest = save.trim().strip_prefix(PREFIX).ok_or_else(not_a_save)?;
    let (version, body) = rest.split_once(':').ok_or_else(not_a_save)?;
    let version: u32 = version.parse().map_err(|_| not_a_save())?;
    if version > VERSION {
        return Err(format!(
            "this save is format {version}, from a newer engine; this one reads format {VERSION}: update the engine the game runs on"
        ));
    }
    let deflated = unbase64(body).ok_or_else(|| {
        "the save's text is damaged (not base64): store and read it back unchanged".to_string()
    })?;
    let payload = miniz_oxide::inflate::decompress_to_vec_with_limit(&deflated, MAX_INFLATED)
        .map_err(|_| {
            "the save's text is damaged (it does not inflate): store and read it back unchanged"
                .to_string()
        })?;
    if payload.len() < 24 || &payload[..4] != MAGIC {
        return Err(not_a_save());
    }
    let snapshot = &payload[24..];
    let hash = u64::from_le_bytes(payload[16..24].try_into().expect("8 bytes"));
    if og_core::hash::hash_bytes(snapshot) != hash {
        return Err("the save's text is damaged (its checksum does not match): store and read it back unchanged".into());
    }
    Ok(snapshot.to_vec())
}

fn unbase64(text: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    let mut acc = 0u32;
    let mut bits = 0;
    for c in text.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => break,
            b' ' | b'\n' | b'\r' | b'\t' => continue,
            _ => return None,
        };
        acc = (acc << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    Some(out)
}

/// The refusal for a save this world does not take, with its fix.
fn refusal(e: og_core::Error, migrate: bool) -> String {
    let reason = e.reason().to_string();
    let reason = reason
        .strip_prefix("snapshot: ")
        .unwrap_or(&reason)
        .to_string();
    match e.kind() {
        og_core::ErrorKind::Schema if !migrate => format!(
            "the save does not fit this build: {reason}; pass {{ migrate: true }} to map it field by field (added fields take their defaults, removed ones are dropped), or keep a saved type's fields stable between releases"
        ),
        og_core::ErrorKind::NotFound if !migrate => format!(
            "the save does not fit this build: {reason} (declare it as the build that saved did, or pass {{ migrate: true }} to drop it)"
        ),
        _ => format!("the save cannot be restored: {reason}"),
    }
}

/// `World.restore`.
pub fn og_world_restore(world: u32, save: String, desc: Option<String>) -> String {
    let eng = match engine(world) {
        Ok(e) => e,
        Err(e) => return refuse(e),
    };
    let migrate = desc
        .as_deref()
        .and_then(|d| serde_json::from_str::<Value>(d).ok())
        .and_then(|d| d["migrate"].as_bool())
        .unwrap_or(false);
    let snapshot = match decode(&save) {
        Ok(s) => s,
        Err(e) => return refuse(e),
    };
    if let Some(f) = running(world) {
        // SAFETY: the frame is published while its system runs.
        if let Err(e) = unsafe { f.ctx() }.check_snapshot(&snapshot, migrate) {
            return refuse(refusal(e, migrate));
        }
        eng.state.borrow_mut().pending_restore = Some(Pending { snapshot, migrate });
        return ok(Value::Null);
    }
    let w = match eng.world() {
        Ok(w) => w,
        Err(e) => return refuse(e),
    };
    if let Err(e) = w.check_snapshot(&snapshot, migrate) {
        return refuse(refusal(e, migrate));
    }
    // A command runs inside the probe's tick: the restore waits for its end.
    let Ok(mut probe) = eng.probe.try_borrow_mut() else {
        eng.state.borrow_mut().pending_restore = Some(Pending { snapshot, migrate });
        return ok(Value::Null);
    };
    let from = w.time().tick;
    let result = apply(&eng, w, &Pending { snapshot, migrate });
    probe.shift_schedule(from, w.time().tick);
    match result {
        Ok(()) => ok(Value::Null),
        Err(e) => refuse(e),
    }
}

/// The probe's end-of-tick hook: a restore asked for during the tick.
pub(crate) fn after_tick(eng: &Engine, world: &mut World) {
    let Some(pending) = eng.state.borrow_mut().pending_restore.take() else {
        return;
    };
    if let Err(e) = apply(eng, world, &pending) {
        let tick = world.time().tick;
        eng.state.borrow_mut().log.push(serde_json::json!({
            "tick": tick, "warning": true, "text": format!("world.restore: {e}"),
        }));
    }
}

/// The probe's start-of-tick hook: after a restore, the tick that follows
/// delivers og.WorldRestored (`from`, the tick the world was at, and
/// `tick`, the saved one), so systems can reset what lives in the session
/// (request numbers, settings read from the store). Sent here rather than
/// when the restore applies, so the world between the two hashes as the
/// save did.
pub(crate) fn before_tick(eng: &Engine, world: &mut World) {
    let Some((from, to)) = eng.state.borrow_mut().restored.take() else {
        return;
    };
    let Some(layout) = eng.state.borrow().layouts.get(RESTORED).cloned() else {
        return;
    };
    let sent = world.id_by_name(RESTORED).and_then(|id| {
        let v = serde_json::json!({ "from": from, "tick": to });
        let bytes = layout
            .encode(None, Some(&v))
            .map_err(|e| og_core::Error::new(og_core::ErrorKind::Invalid, e))?;
        world.send_bytes(id, &bytes)
    });
    if let Err(e) = sent {
        eng.state.borrow_mut().log.push(serde_json::json!({
            "tick": to, "warning": true, "text": format!("{RESTORED}: {e}"),
        }));
    }
}

/// The event a restore delivers the next tick.
const RESTORED: &str = "og.WorldRestored";

/// Restores a checked save, keeping the session, and brings the runtime's
/// own view of the world (names, views, the platform clock, the renderer's
/// history) along.
fn apply(eng: &Engine, world: &mut World, pending: &Pending) -> Result<(), String> {
    apply_keeping(eng, world, pending, &[])
}

/// A peer's world snapshot (og.net's lockstep start or late join), restored
/// as a save is, keeping the session and the resources `keep` names too.
pub(crate) fn restore_session(
    eng: &Engine,
    world: &mut World,
    snapshot: &[u8],
    keep: &[&str],
) -> Result<(), String> {
    world
        .check_snapshot(snapshot, false)
        .map_err(|e| refusal(e, false))?;
    let pending = Pending {
        snapshot: snapshot.to_vec(),
        migrate: false,
    };
    apply_keeping(eng, world, &pending, keep)
}

fn apply_keeping(
    eng: &Engine,
    world: &mut World,
    pending: &Pending,
    keep: &[&str],
) -> Result<(), String> {
    let from = world.time().tick;
    let kept: Vec<(og_ecs::SchemaId, Vec<u8>)> = SESSION
        .iter()
        .chain(keep.iter())
        .filter_map(|name| {
            let id = world.id_by_name(name).ok()?;
            Some((id, world.resource_bytes(id).ok()?))
        })
        .collect();
    if pending.migrate {
        world
            .restore_migrating(&pending.snapshot)
            .map_err(|e| refusal(e, true))?;
    } else {
        world
            .restore(&pending.snapshot)
            .map_err(|e| refusal(e, false))?;
    }
    for (id, bytes) in kept {
        let _ = world.insert_resource_bytes(id, &bytes);
    }
    let to = world.time().tick;
    {
        let mut st = eng.state.borrow_mut();
        st.names.clear();
        st.name_of.clear();
        st.name_ops.clear();
        for (e, name) in world.names() {
            st.names.insert(name.to_string(), e.to_bits());
            st.name_of.insert(e.to_bits(), name.to_string());
        }
    }
    eng.state.borrow_mut().restored = Some((from, to));
    super::platform::shift_ticks(eng.id, from, to);
    if let Ok(mut rs) = eng.render.try_borrow_mut()
        && let Some(b) = rs.backend.as_mut()
    {
        // A cut: the renderer's history (exposure, TAA) starts afresh.
        b.restore_state(0);
    }
    super::engine::invalidate(true);
    super::engine::clear_world_queries();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_round_trips_every_length() {
        for n in 0..70u8 {
            let bytes: Vec<u8> = (0..n).map(|i| i.wrapping_mul(37) ^ n).collect();
            assert_eq!(unbase64(&og_probe::base64::encode(&bytes)).unwrap(), bytes);
        }
        assert!(unbase64("ab!c").is_none());
    }

    #[test]
    fn a_save_decodes_to_its_snapshot_and_other_text_is_refused_with_the_fix() {
        let save = encode(b"OGWS world bytes", 42);
        assert!(save.starts_with("og-save:1:"));
        assert_eq!(decode(&save).unwrap(), b"OGWS world bytes");
        assert!(decode("{\"chimes\":1}").unwrap_err().contains("not a save"));
        assert!(
            decode("og-save:9:AAAA")
                .unwrap_err()
                .contains("newer engine")
        );
        assert!(decode("og-save:1:@@@").unwrap_err().contains("damaged"));
        assert!(decode("og-save:1:AAAA").unwrap_err().contains("damaged"));
    }
}
