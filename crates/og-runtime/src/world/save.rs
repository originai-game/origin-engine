//! A game's saves (IDL `World.snapshot`, `World.restore`, `World.openSave`):
//! the world, or the part of it a game keeps, as text a game puts in the
//! platform store, and back.
//!
//! The text is `og-save:3:` and the base64 of a deflated payload: `OGSV`,
//! a `u32` version (3), the `u64` tick it was taken at, the `u64` FNV-1a
//! of the body (a damaged save is refused) and the `u32` save version
//! of the game that wrote it (GameDesc.saveVersion), then the length-prefixed world
//! snapshot (og-ecs's format, which carries every type's name, layout hash
//! and field table, so a save from a build whose types changed is refused
//! naming them, or mapped field by field with `migrate`; the session's
//! types below may differ without either, as the host's replace them,
//! and so may an engine type that only gained fields since the save,
//! which take their declared defaults: [`engine_owned`]).
//! The body ends with a portable UI presentation checkpoint, outside the
//! simulation hash. Other envelope versions are refused without conversion.
//!
//! A restore first upgrades the engine's own state in the save
//! ([`crate::world::save_migrations`]: og.ui's block 9 → 11 ...), so an engine
//! upgrade never refuses a player's save for what the engine changed. The
//! game's migrations (GameDesc.migrations) run in TypeScript before it,
//! on a save opened with `World.openSave`: [`og_world_open_save`] and the
//! `og_save_edit_*` calls edit it, and its text carries the game's current
//! save version. A save of a newer game version is refused.
//!
//! A selection (SnapshotDesc.with, .resources) is the entities carrying
//! the tags and the chosen resources: restoring it replaces those entities
//! and sets those resources, and leaves the rest of the world as it is
//! (og-ecs `World::restore_selection`).
//!
//! A save is the world; the session is not in it. Restoring keeps the
//! host's input (`og.Input`, `og.Pointers`, `og.Gamepad`, `og.PointerLock`) and the player's
//! bus volumes (`og.Audio`), and moves what lives outside the world on the clock with
//! `og.Time`'s jump: the probe's scheduled steps keep their remaining
//! delays, `og.platform.now` its time. Inside a system or a command (the
//! probe is mid-tick) the restore waits for the tick's end
//! ([`after_tick`]); elsewhere it applies at once. Either way the next
//! tick delivers og.WorldRestored ([`before_tick`]).

use crate::engine::{Engine, HANDLES, engine, running};
use crate::host::wire::{SaveRecord, SaveType};
use og_ecs::{SnapshotEdit, TypeValues, World};
use serde_json::{Value, json};

mod apply;
mod edit;

pub(crate) use apply::*;
pub use edit::*;

const PREFIX: &str = "og-save:";
const MAGIC: &[u8; 4] = b"OGSV";
const VERSION: u32 = 3;
/// og-ecs's selection format: the snapshot's `u32` after its magic.
const SELECTION: u32 = 5;
/// The largest save inflated: far over any world the 64 KiB store holds.
const MAX_INFLATED: usize = 64 << 20;

/// The resources a restore keeps from the session.
const SESSION: &[&str] = &[
    "og.Input",
    "og.Pointers",
    "og.Gamepad",
    "og.PointerLock",
    "og.Audio",
];

/// Whether a type is the engine's own (an og.* component, resource or
/// event: the IDL's records and the modules'; a game's names cannot start
/// with `og.`). A save restores without `migrate` when such a type only
/// gained fields since it was written, each added field at its declared
/// default (og.RenderSettings' new fields in every save of the build
/// before); a field removed, renamed or retyped still needs an engine
/// migration, and a game's own types stay strict.
fn engine_owned(name: &str) -> bool {
    name.starts_with("og.")
}

/// A restore asked for inside a tick, applied when it ends.
pub(crate) struct Pending {
    snapshot: Vec<u8>,
    migrate: bool,
    /// The save's input (RestoreDesc.session) instead of the host's.
    session: bool,
    ui: Option<crate::present::presentation::UiCheckpoint>,
}

/// What a save's text holds.
#[derive(Debug)]
struct Save {
    snapshot: Vec<u8>,
    ui: Option<crate::present::presentation::UiCheckpoint>,
    tick: u64,
    /// The game's save version when it was written.
    version: u32,
    /// The engine's migrations [`upgraded`] applied.
    steps: Vec<String>,
}

fn is_selection(snapshot: &[u8]) -> bool {
    snapshot.get(4..8) == Some(&SELECTION.to_le_bytes()[..])
}

/// A selection's tags and resources, as this world's type ids.
type SelectionIds = (Vec<og_ecs::SchemaId>, Vec<og_ecs::SchemaId>);

/// SnapshotDesc's `with` and `resources`, as this world's type ids.
fn selection_ids(
    desc: &crate::host::wire::SnapshotDesc,
    id: impl Fn(&str) -> og_core::Result<og_ecs::SchemaId>,
) -> Result<Option<SelectionIds>, String> {
    let names = |list: &[Value]| -> Vec<String> {
        list.iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect()
    };
    let (with, resources) = (names(&desc.with), names(&desc.resources));
    if with.is_empty() && resources.is_empty() {
        return Ok(None);
    }
    let ids = |list: &[String], what: &str| -> Result<Vec<og_ecs::SchemaId>, String> {
        list.iter()
            .map(|n| {
                id(n).map_err(|_| {
                    format!("SnapshotDesc.{what} names '{n}', which this world does not register")
                })
            })
            .collect()
    };
    Ok(Some((ids(&with, "with")?, ids(&resources, "resources")?)))
}

/// `World.snapshot`.
pub fn og_world_snapshot(world: u32, desc: Option<String>) -> Result<serde_json::Value, String> {
    let eng = engine(world)?;
    let desc: crate::host::wire::SnapshotDesc = crate::host::wire::arg(desc.as_deref(), "desc")?;
    let version = eng.state.borrow().save_version;
    let taken = if let Some(f) = running(world) {
        // SAFETY: the frame is published while its system runs.
        let ctx = unsafe { f.ctx() };
        match selection_ids(&desc, |n| ctx.id_by_name(n)) {
            Ok(None) => Ok((ctx.snapshot(), ctx.time().tick)),
            Ok(Some((with, resources))) => ctx
                .snapshot_selection(&with, &resources)
                .map(|b| (b, ctx.time().tick))
                .map_err(|e| e.reason().to_string()),
            Err(e) => Err(e),
        }
    } else {
        // Names given since the last tick began (a spawn in GameDesc.start or
        // a command) go into the world first, so the save holds them.
        crate::author::probe::flush_names(&eng);
        match eng.world() {
            Ok(w) => match selection_ids(&desc, |n| w.id_by_name(n)) {
                Ok(None) => Ok((w.snapshot(), w.time().tick)),
                Ok(Some((with, resources))) => w
                    .snapshot_selection(&with, &resources)
                    .map(|b| (b, w.time().tick))
                    .map_err(|e| e.reason().to_string()),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    };
    match taken {
        Ok((bytes, tick)) => {
            let ui = (!is_selection(&bytes))
                .then(|| {
                    eng.state
                        .borrow()
                        .ui
                        .as_ref()
                        .map(crate::present::presentation::UiCheckpoint::save)
                })
                .flatten();
            Ok(Value::String(encode(&bytes, tick, version, ui.as_ref())))
        }
        Err(e) => Err(e),
    }
}

/// The save text of a world snapshot, of the game's save version `version`.
fn encode(
    snapshot: &[u8],
    tick: u64,
    version: u32,
    ui: Option<&crate::present::presentation::UiCheckpoint>,
) -> String {
    let ui = serde_json::to_vec(&ui).expect("UI checkpoint contains bytes only");
    let mut body = Vec::with_capacity(snapshot.len() + ui.len() + 4);
    body.extend_from_slice(&(snapshot.len() as u32).to_le_bytes());
    body.extend_from_slice(snapshot);
    body.extend_from_slice(&ui);
    let hash = og_core::hash::hash_bytes(&body);
    let mut payload = Vec::with_capacity(body.len() + 28);
    payload.extend_from_slice(MAGIC);
    payload.extend_from_slice(&VERSION.to_le_bytes());
    payload.extend_from_slice(&tick.to_le_bytes());
    payload.extend_from_slice(&hash.to_le_bytes());
    payload.extend_from_slice(&version.to_le_bytes());
    payload.extend_from_slice(&body);
    let deflated = miniz_oxide::deflate::compress_to_vec(&payload, 6);
    format!("{PREFIX}{VERSION}:{}", og_probe::base64::encode(&deflated))
}

/// The world snapshot a save's text holds (`og replay compare` diffs a
/// recorded save against a replayed world), or why it is not a save.
pub fn save_snapshot_bytes(save: &str) -> Result<Vec<u8>, String> {
    decode(save)
}

/// What a save holds.
fn decode_save(save: &str) -> Result<Save, String> {
    let not_a_save = || {
        "this is not a save: world.restore takes the text world.snapshot returned (`og-save:3:...`), as it was stored".to_string()
    };
    let rest = save.trim().strip_prefix(PREFIX).ok_or_else(not_a_save)?;
    let (format, body) = rest.split_once(':').ok_or_else(not_a_save)?;
    let format: u32 = format.parse().map_err(|_| not_a_save())?;
    if format != VERSION {
        return Err(format!(
            "save format {format} is incompatible with engine save format {VERSION}; save migration is not supported"
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
    let header = 28;
    if payload.len() < header || &payload[..4] != MAGIC || payload[4..8] != format.to_le_bytes() {
        return Err(not_a_save());
    }
    let body = &payload[header..];
    let hash = u64::from_le_bytes(payload[16..24].try_into().expect("8 bytes"));
    if og_core::hash::hash_bytes(body) != hash {
        return Err("the save's text is damaged (its checksum does not match): store and read it back unchanged".into());
    }
    let size = body.get(..4).ok_or_else(not_a_save)?;
    let size = u32::from_le_bytes(size.try_into().expect("4 bytes")) as usize;
    let end = size.checked_add(4).ok_or_else(not_a_save)?;
    let snapshot = body.get(4..end).ok_or_else(not_a_save)?;
    let ui: Option<crate::present::presentation::UiCheckpoint> =
        serde_json::from_slice(&body[end..])
            .map_err(|e| format!("invalid UI presentation checkpoint: {e}"))?;
    if let Some(ui) = &ui {
        ui.validate()?;
    }
    Ok(Save {
        snapshot: snapshot.to_vec(),
        ui,
        tick: u64::from_le_bytes(payload[8..16].try_into().expect("8 bytes")),
        version: u32::from_le_bytes(payload[24..28].try_into().expect("4 bytes")),
        steps: Vec::new(),
    })
}

/// What a save holds: the world snapshot.
fn decode(save: &str) -> Result<Vec<u8>, String> {
    decode_save(save).map(|s| s.snapshot)
}

/// A save's snapshot as this engine writes it: the engine's migrations
/// applied ([`crate::world::save_migrations`]), and the game's save version
/// checked against `current` (a save of a newer version is refused).
fn upgraded(save: &str, current: u32) -> Result<Save, String> {
    let mut s = decode_save(save)?;
    if s.version > current {
        return Err(format!(
            "this save is of the game's save version {}, and this build is at {current}: a newer build of the game wrote it",
            s.version
        ));
    }
    if let Some((bytes, steps)) = crate::world::save_migrations::upgrade(&s.snapshot)? {
        s.snapshot = bytes;
        s.steps = steps;
    }
    Ok(s)
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
pub fn og_world_restore(
    world: u32,
    save: String,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    let eng = engine(world)?;
    let crate::host::wire::RestoreDesc { migrate, session } =
        crate::host::wire::arg(desc.as_deref(), "desc")?;
    let current = eng.state.borrow().save_version;
    let (snapshot, ui) = {
        let s = upgraded(&save, current)?;
        // What the engine's migrations did, in the log (`og run` prints it).
        if !s.steps.is_empty() {
            eng.state.borrow_mut().log.push(json!({
                "tick": og_ecs::last_tick(), "warning": false,
                "text": format!("world.restore: the engine upgraded the save: {}", s.steps.join("; ")),
            }));
        }
        (s.snapshot, s.ui)
    };
    if is_selection(&snapshot) {
        // Checked whole when it applies (restore_selection changes nothing
        // when it refuses); here, only that it reads.
        if let Err(e) = SnapshotEdit::from_bytes(&snapshot) {
            return Err(refusal(e, migrate));
        }
        if running(world).is_some() || eng.probe.try_borrow_mut().is_err() {
            eng.state.borrow_mut().pending_restore = Some(Pending {
                snapshot,
                migrate,
                session,
                ui,
            });
            return Ok(Value::Null);
        }
        let w = eng.world()?;
        return match apply(
            &eng,
            w,
            &Pending {
                snapshot,
                migrate,
                session,
                ui,
            },
        ) {
            Ok(()) => Ok(Value::Null),
            Err(e) => Err(e),
        };
    }
    // The session's types (SESSION) are the host's after a restore, so a
    // save whose only differences are theirs (og.Gamepad's eight pads in a
    // build after a four-pad save) is taken, mapped field by field.
    if let Some(f) = running(world) {
        // A save that holds a lazy record the world has not registered
        // (crate::engine::lazy): registered and checked as the tick ends.
        if crate::engine::lazy::snapshot_needs(&eng, &snapshot) {
            eng.state.borrow_mut().pending_restore = Some(Pending {
                snapshot,
                migrate,
                session,
                ui,
            });
            return Ok(Value::Null);
        }
        // SAFETY: the frame is published while its system runs.
        let migrate = match unsafe { f.ctx() }.check_snapshot_replacing(
            &snapshot,
            migrate,
            SESSION,
            &engine_owned,
        ) {
            Ok(session_differs) => migrate || session_differs,
            Err(e) => return Err(refusal(e, migrate)),
        };
        eng.state.borrow_mut().pending_restore = Some(Pending {
            snapshot,
            migrate,
            session,
            ui,
        });
        return Ok(Value::Null);
    }
    let w = eng.world()?;
    crate::engine::lazy::before_restore(&eng, w, &snapshot);
    let migrate = match w.check_snapshot_replacing(&snapshot, migrate, SESSION, &engine_owned) {
        Ok(session_differs) => migrate || session_differs,
        Err(e) => return Err(refusal(e, migrate)),
    };
    // A command runs inside the probe's tick: the restore waits for its end.
    let Ok(mut probe) = eng.probe.try_borrow_mut() else {
        eng.state.borrow_mut().pending_restore = Some(Pending {
            snapshot,
            migrate,
            session,
            ui,
        });
        return Ok(Value::Null);
    };
    let from = w.time().tick;
    let result = apply(
        &eng,
        w,
        &Pending {
            snapshot,
            migrate,
            session,
            ui,
        },
    );
    probe.shift_schedule(from, w.time().tick);
    match result {
        Ok(()) => Ok(Value::Null),
        Err(e) => Err(e),
    }
}

// ── saves opened for the game's migrations (World.openSave, SaveEdit) ─────

#[cfg(test)]
mod tests;
