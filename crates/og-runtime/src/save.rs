//! A game's saves (IDL `World.snapshot`, `World.restore`, `World.openSave`):
//! the world, or the part of it a game keeps, as text a game puts in the
//! platform store, and back.
//!
//! The text is `og-save:2:` and the base64 of a deflated payload: `OGSV`,
//! a `u32` version (2), the `u64` tick it was taken at, the `u64` FNV-1a
//! of the snapshot (a damaged save is refused) and the `u32` save version
//! of the game that wrote it (GameDesc.saveVersion), then the world
//! snapshot (og-ecs's format, which carries every type's name, layout hash
//! and field table, so a save from a build whose types changed is refused
//! naming them, or mapped field by field with `migrate`; the session's
//! types below may differ without either, as the host's replace them,
//! and so may an engine type that only gained fields since the save,
//! which take their declared defaults: [`engine_owned`]).
//! Format 1 (`og-save:1:`, engines to 0.7.7) has no save version: it reads
//! as version 1.
//!
//! A restore first upgrades the engine's own state in the save
//! ([`super::save_migrations`]: og.ui's block 9 → 11 ...), so an engine
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

use super::engine::{Engine, engine, running};
use og_ecs::{SnapshotEdit, TypeValues, World};
use serde_json::{Value, json};
use std::cell::RefCell;

const PREFIX: &str = "og-save:";
const MAGIC: &[u8; 4] = b"OGSV";
const VERSION: u32 = 2;
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
}

/// What a save's text holds.
#[derive(Debug)]
struct Save {
    snapshot: Vec<u8>,
    tick: u64,
    /// The game's save version when it was written.
    version: u32,
    /// The engine's migrations [`upgraded`] applied.
    steps: Vec<String>,
}

fn is_selection(snapshot: &[u8]) -> bool {
    snapshot.get(4..8) == Some(&SELECTION.to_le_bytes()[..])
}

fn ok(value: Value) -> String {
    serde_json::json!({ "ok": true, "value": value }).to_string()
}

fn refuse(reason: impl Into<String>) -> String {
    serde_json::json!({ "ok": false, "reason": reason.into() }).to_string()
}

/// A selection's tags and resources, as this world's type ids.
type SelectionIds = (Vec<og_ecs::SchemaId>, Vec<og_ecs::SchemaId>);

/// SnapshotDesc's `with` and `resources`, as this world's type ids.
fn selection_ids(
    desc: &Value,
    id: impl Fn(&str) -> og_core::Result<og_ecs::SchemaId>,
) -> Result<Option<SelectionIds>, String> {
    let names = |k: &str| -> Vec<String> {
        desc[k]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    };
    let (with, resources) = (names("with"), names("resources"));
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
pub fn og_world_snapshot(world: u32, desc: Option<String>) -> String {
    let eng = match engine(world) {
        Ok(e) => e,
        Err(e) => return refuse(e),
    };
    let desc = desc
        .as_deref()
        .and_then(|d| serde_json::from_str::<Value>(d).ok())
        .unwrap_or(Value::Null);
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
        super::probe::flush_names(&eng);
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
        Ok((bytes, tick)) => ok(Value::String(encode(&bytes, tick, version))),
        Err(e) => refuse(e),
    }
}

/// The save text of a world snapshot, of the game's save version `version`.
fn encode(snapshot: &[u8], tick: u64, version: u32) -> String {
    let hash = og_core::hash::hash_bytes(snapshot);
    let mut payload = Vec::with_capacity(snapshot.len() + 28);
    payload.extend_from_slice(MAGIC);
    payload.extend_from_slice(&VERSION.to_le_bytes());
    payload.extend_from_slice(&tick.to_le_bytes());
    payload.extend_from_slice(&hash.to_le_bytes());
    payload.extend_from_slice(&version.to_le_bytes());
    payload.extend_from_slice(snapshot);
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
        "this is not a save: world.restore takes the text world.snapshot returned (`og-save:2:...`), as it was stored".to_string()
    };
    let rest = save.trim().strip_prefix(PREFIX).ok_or_else(not_a_save)?;
    let (format, body) = rest.split_once(':').ok_or_else(not_a_save)?;
    let format: u32 = format.parse().map_err(|_| not_a_save())?;
    if format > VERSION || format == 0 {
        return Err(format!(
            "this save is format {format}, from a newer engine; this one reads formats 1 and {VERSION}: update the engine the game runs on"
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
    // Format 1 has no save version: version 1, the first a game has.
    let header = if format == 1 { 24 } else { 28 };
    if payload.len() < header || &payload[..4] != MAGIC || payload[4..8] != format.to_le_bytes() {
        return Err(not_a_save());
    }
    let snapshot = &payload[header..];
    let hash = u64::from_le_bytes(payload[16..24].try_into().expect("8 bytes"));
    if og_core::hash::hash_bytes(snapshot) != hash {
        return Err("the save's text is damaged (its checksum does not match): store and read it back unchanged".into());
    }
    Ok(Save {
        snapshot: snapshot.to_vec(),
        tick: u64::from_le_bytes(payload[8..16].try_into().expect("8 bytes")),
        version: if format == 1 {
            1
        } else {
            u32::from_le_bytes(payload[24..28].try_into().expect("4 bytes"))
        },
        steps: Vec::new(),
    })
}

/// What a save holds: the world snapshot.
fn decode(save: &str) -> Result<Vec<u8>, String> {
    decode_save(save).map(|s| s.snapshot)
}

/// A save's snapshot as this engine writes it: the engine's migrations
/// applied ([`super::save_migrations`]), and the game's save version
/// checked against `current` (a save of a newer version is refused).
fn upgraded(save: &str, current: u32) -> Result<Save, String> {
    let mut s = decode_save(save)?;
    if s.version > current {
        return Err(format!(
            "this save is of the game's save version {}, and this build is at {current}: a newer build of the game wrote it",
            s.version
        ));
    }
    if let Some((bytes, steps)) = super::save_migrations::upgrade(&s.snapshot)? {
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
    let current = eng.state.borrow().save_version;
    let snapshot = match upgraded(&save, current) {
        Ok(s) => {
            // What the engine's migrations did, in the log (`og run` prints it).
            if !s.steps.is_empty() {
                eng.state.borrow_mut().log.push(json!({
                    "tick": og_ecs::last_tick(), "warning": false,
                    "text": format!("world.restore: the engine upgraded the save: {}", s.steps.join("; ")),
                }));
            }
            s.snapshot
        }
        Err(e) => return refuse(e),
    };
    if is_selection(&snapshot) {
        // Checked whole when it applies (restore_selection changes nothing
        // when it refuses); here, only that it reads.
        if let Err(e) = SnapshotEdit::from_bytes(&snapshot) {
            return refuse(refusal(e, migrate));
        }
        if running(world).is_some() || eng.probe.try_borrow_mut().is_err() {
            eng.state.borrow_mut().pending_restore = Some(Pending { snapshot, migrate });
            return ok(Value::Null);
        }
        let w = match eng.world() {
            Ok(w) => w,
            Err(e) => return refuse(e),
        };
        return match apply(&eng, w, &Pending { snapshot, migrate }) {
            Ok(()) => ok(Value::Null),
            Err(e) => refuse(e),
        };
    }
    // The session's types (SESSION) are the host's after a restore, so a
    // save whose only differences are theirs (og.Gamepad's eight pads in a
    // build after a four-pad save) is taken, mapped field by field.
    if let Some(f) = running(world) {
        // SAFETY: the frame is published while its system runs.
        let migrate = match unsafe { f.ctx() }.check_snapshot_replacing(
            &snapshot,
            migrate,
            SESSION,
            &engine_owned,
        ) {
            Ok(session_differs) => migrate || session_differs,
            Err(e) => return refuse(refusal(e, migrate)),
        };
        eng.state.borrow_mut().pending_restore = Some(Pending { snapshot, migrate });
        return ok(Value::Null);
    }
    let w = match eng.world() {
        Ok(w) => w,
        Err(e) => return refuse(e),
    };
    let migrate = match w.check_snapshot_replacing(&snapshot, migrate, SESSION, &engine_owned) {
        Ok(session_differs) => migrate || session_differs,
        Err(e) => return refuse(refusal(e, migrate)),
    };
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

// ── saves opened for the game's migrations (World.openSave, SaveEdit) ─────

/// An open save: its engine, the snapshot opened for editing, the tick it
/// was taken at, and the game's save version it is at.
struct Open {
    engine: u32,
    edit: SnapshotEdit,
    tick: u64,
    version: u32,
}

thread_local! {
    static OPEN: RefCell<Vec<Option<Open>>> = const { RefCell::new(Vec::new()) };
}

/// `World.openSave`: the save with the engine's migrations applied, open
/// for the game's (a handle for the `og_save_edit_*` calls).
pub fn og_world_open_save(world: u32, save: String) -> String {
    let eng = match engine(world) {
        Ok(e) => e,
        Err(e) => return refuse(e),
    };
    let current = eng.state.borrow().save_version;
    let s = match upgraded(&save, current) {
        Ok(s) => s,
        Err(e) => return refuse(e),
    };
    let edit = match SnapshotEdit::from_bytes(&s.snapshot) {
        Ok(e) => e,
        Err(e) => return refuse(format!("the save cannot be opened: {}", e.reason())),
    };
    let open = Open {
        engine: world,
        edit,
        tick: s.tick,
        version: s.version,
    };
    let handle = OPEN.with(|o| {
        let mut o = o.borrow_mut();
        match o.iter().position(Option::is_none) {
            Some(i) => {
                o[i] = Some(open);
                i
            }
            None => {
                o.push(Some(open));
                o.len() - 1
            }
        }
    });
    ok(json!(handle + 1))
}

fn with_open<T>(handle: u32, f: impl FnOnce(&mut Open) -> Result<T, String>) -> Result<T, String> {
    OPEN.with(|o| {
        let mut o = o.borrow_mut();
        let open = (handle as usize)
            .checked_sub(1)
            .and_then(|i| o.get_mut(i))
            .and_then(Option::as_mut)
            .ok_or("this save is closed: open it again with world.openSave")?;
        f(open)
    })
}

fn answer<T: Into<Value>>(r: Result<T, String>) -> String {
    match r {
        Ok(v) => ok(v.into()),
        Err(e) => refuse(e),
    }
}

fn reason(e: og_core::Error) -> String {
    let r = e.reason().to_string();
    r.strip_prefix("snapshot: ").unwrap_or(&r).to_string()
}

/// `SaveEdit.version`.
pub fn og_save_edit_get_version(edit: u32) -> u32 {
    with_open(edit, |o| Ok(o.version)).unwrap_or(0)
}

/// `SaveEdit.types`: every type the save holds, with its saved version
/// and fields.
pub fn og_save_edit_types(edit: u32) -> String {
    answer(with_open(edit, |o| {
        Ok(o.edit
            .schemas()
            .iter()
            .map(|t| {
                json!({
                    "name": t.name,
                    "version": t.version,
                    "fields": t.fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>(),
                })
            })
            .collect::<Vec<_>>())
    }))
}

/// The saved type's layout, for reading its records.
fn saved_layout(edit: &SnapshotEdit, ty: &str) -> Result<super::codec::Layout, String> {
    let schema = edit
        .schema(ty)
        .ok_or_else(|| format!("the save holds no type '{ty}'"))?;
    Ok(super::codec::Layout::from_schema(
        schema,
        super::codec::RecordKind::Component,
    ))
}

/// `SaveEdit.records`: a type's values as the save holds them, components
/// by entity in storage order, then its resource (entity null).
pub fn og_save_edit_records(edit: u32, ty: String) -> String {
    answer(with_open(edit, |o| {
        let layout = saved_layout(&o.edit, &ty)?;
        let mut out: Vec<Value> = o
            .edit
            .components(&ty)
            .iter()
            .map(|(e, b)| json!({ "entity": e.to_bits(), "value": layout.decode(b) }))
            .collect();
        if let Some(b) = o.edit.resource(&ty) {
            out.push(json!({ "entity": null, "value": layout.decode(b) }));
        }
        Ok(out)
    }))
}

/// This build's layout and schema of `ty`, for writing into a save.
fn current(
    engine_id: u32,
    ty: &str,
) -> Result<(super::codec::Layout, og_core::reflect::Schema), String> {
    let eng = engine(engine_id)?;
    let layout = eng.state.borrow().layouts.get(ty).cloned().ok_or_else(|| {
        format!(
            "'{ty}' is not a type of this build: a migration writes the types the game declares now"
        )
    })?;
    let w = eng.world()?;
    let schema = w
        .id_by_name(ty)
        .and_then(|id| w.schema(id).cloned())
        .map_err(|e| e.reason().to_string())?;
    Ok((layout, schema))
}

/// `SaveEdit.setRecords`: replaces a type's values with `records`, written
/// as this build's `into` (default the type itself): each record's fields
/// by name, those it leaves out at their defaults. The type's old values
/// go; its queued events too. A type the save does not hold is added.
pub fn og_save_edit_set_records(edit: u32, ty: String, desc: Option<String>) -> String {
    let d = desc
        .as_deref()
        .and_then(|d| serde_json::from_str::<Value>(d).ok())
        .unwrap_or(Value::Null);
    let into = d["into"]
        .as_str()
        .filter(|s| !s.is_empty())
        .unwrap_or(&ty)
        .to_string();
    answer(with_open(edit, |o| {
        // A type the save does not hold is added (written as `into`).
        let remove: &[&str] = if o.edit.schema(&ty).is_some() {
            &[ty.as_str()]
        } else {
            &[]
        };
        let (layout, schema) = current(o.engine, &into)?;
        let mut values = TypeValues {
            schema,
            components: Vec::new(),
            resource: None,
        };
        for r in d["records"].as_array().into_iter().flatten() {
            let bytes = layout
                .encode(None, Some(&r["value"]))
                .map_err(|e| format!("a record for {into}: {e}"))?;
            match r["entity"].as_u64() {
                Some(e) => values
                    .components
                    .push((og_ecs::Entity::from_bits(e), bytes)),
                None if values.resource.is_none() => values.resource = Some(bytes),
                None => return Err(format!("two {into} resources (records without an entity)")),
            }
        }
        o.edit.replace_types(remove, vec![values]).map_err(reason)
    }))
}

/// `SaveEdit.renameType`.
pub fn og_save_edit_rename_type(edit: u32, from: String, to: String) -> String {
    answer(with_open(edit, |o| {
        let mut schema = o
            .edit
            .schema(&from)
            .cloned()
            .ok_or_else(|| format!("the save holds no type '{from}'"))?;
        schema.name = to.clone();
        let values = TypeValues {
            schema,
            components: o.edit.components(&from),
            resource: o.edit.resource(&from).map(<[u8]>::to_vec),
        };
        o.edit
            .replace_types(&[from.as_str()], vec![values])
            .map_err(reason)
    }))
}

/// `SaveEdit.renameField`.
pub fn og_save_edit_rename_field(edit: u32, ty: String, from: String, to: String) -> String {
    answer(with_open(edit, |o| {
        let mut schema = o
            .edit
            .schema(&ty)
            .cloned()
            .ok_or_else(|| format!("the save holds no type '{ty}'"))?;
        if schema.field_index(&to).is_some() {
            return Err(format!("{ty} already has a field '{to}'"));
        }
        let i = schema
            .field_index(&from)
            .ok_or_else(|| format!("{ty} has no field '{from}' in the save"))?;
        schema.fields[i].name = to;
        o.edit
            .map_type(&ty, schema, &mut |v| Ok(v.to_vec()))
            .map_err(reason)
    }))
}

/// `SaveEdit.dropType`.
pub fn og_save_edit_drop_type(edit: u32, ty: String) -> String {
    answer(with_open(edit, |o| {
        o.edit
            .replace_types(&[ty.as_str()], Vec::new())
            .map_err(reason)
    }))
}

/// `SaveEdit.spawn`: an entity added to the save, its components written
/// as this build's types.
pub fn og_save_edit_spawn(edit: u32, desc: Option<String>) -> String {
    let d = desc
        .as_deref()
        .and_then(|d| serde_json::from_str::<Value>(d).ok())
        .unwrap_or(Value::Null);
    answer(with_open(edit, |o| {
        let mut components = Vec::new();
        for (ty, value) in d["components"].as_object().into_iter().flatten() {
            let (layout, schema) = current(o.engine, ty)?;
            let bytes = layout
                .encode(None, Some(value))
                .map_err(|e| format!("SaveSpawnDesc.components.{ty}: {e}"))?;
            components.push((schema, bytes));
        }
        let name = d["name"].as_str().filter(|n| !n.is_empty());
        o.edit
            .spawn(name, components)
            .map(|e| e.to_bits())
            .map_err(reason)
    }))
}

/// `SaveEdit.text`: the save as text at the game's current save version
/// (what world.restore takes after the migrations ran).
pub fn og_save_edit_text(edit: u32) -> String {
    answer(with_open(edit, |o| {
        let version = engine(o.engine)?.state.borrow().save_version;
        Ok(encode(&o.edit.to_bytes(), o.tick, version))
    }))
}

/// `SaveEdit.close`.
pub fn og_save_edit_close(edit: u32) {
    OPEN.with(|o| {
        if let Some(slot) = (edit as usize)
            .checked_sub(1)
            .and_then(|i| o.borrow_mut().get_mut(i).map(Option::take))
        {
            drop(slot);
        }
    });
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

/// Restores a selection: the entities carrying its tags are replaced by
/// its own, its resources set, the rest of the world (the session too)
/// left as it is. References to a saved entity in an Entity field follow
/// it to its new id. The tick after it delivers og.WorldRestored, `from`
/// and `tick` both the current tick (og.Time is not in a selection).
fn apply_selection(eng: &Engine, world: &mut World, pending: &Pending) -> Result<(), String> {
    let entity_fields = |ty: &str| -> Vec<usize> {
        eng.state
            .borrow()
            .layouts
            .get(ty)
            .map(|l| {
                l.fields
                    .iter()
                    .enumerate()
                    .filter(|(_, f)| f.kind == super::codec::Kind::Entity)
                    .map(|(i, _)| i)
                    .collect()
            })
            .unwrap_or_default()
    };
    let lenient = |ty: &str| SESSION.contains(&ty);
    world
        .restore_selection(
            &pending.snapshot,
            pending.migrate,
            &lenient,
            &engine_owned,
            &entity_fields,
        )
        .map_err(|e| refusal(e, pending.migrate))?;
    let tick = world.time().tick;
    refresh(eng, world, tick, tick);
    Ok(())
}

/// The runtime's own view of the world after a restore: names, views, the
/// platform clock, the renderer's history.
fn refresh(eng: &Engine, world: &World, from: u64, to: u64) {
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
}

fn apply_keeping(
    eng: &Engine,
    world: &mut World,
    pending: &Pending,
    keep: &[&str],
) -> Result<(), String> {
    if is_selection(&pending.snapshot) {
        return apply_selection(eng, world, pending);
    }
    let from = world.time().tick;
    // Nothing for a frame to blend from until the next tick.
    if let Ok(mut rs) = eng.render.try_borrow_mut() {
        rs.interp.forget();
    }
    let kept: Vec<(og_ecs::SchemaId, Vec<u8>)> = SESSION
        .iter()
        .chain(keep.iter())
        .filter_map(|name| {
            let id = world.id_by_name(name).ok()?;
            Some((id, world.resource_bytes(id).ok()?))
        })
        .collect();
    // The player's bindings and dead zones outlive a load (the actions'
    // state is the save's, as og.Input's edges come from what follows).
    let actions = eng.state.borrow().actions.clone();
    let kept_actions = actions.as_ref().map(|a| og_input::actions::lock(a).clone());
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
    og_audio::restored(world).map_err(|e| e.reason().to_string())?;
    if let (Some(a), Some(map)) = (actions, kept_actions) {
        og_input::actions::lock(&a).keep_player(&map);
    }
    let to = world.time().tick;
    refresh(eng, world, from, to);
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
        let save = encode(b"OGWS world bytes", 42, 3);
        assert!(save.starts_with("og-save:2:"));
        let read = decode_save(&save).unwrap();
        assert_eq!(read.snapshot, b"OGWS world bytes");
        assert_eq!(
            (read.tick, read.version),
            (42, 3),
            "the game's save version"
        );
        assert!(
            upgraded(&save, 2).unwrap_err().contains("save version 3"),
            "a newer game's save is refused"
        );
        assert!(decode("{\"chimes\":1}").unwrap_err().contains("not a save"));
        assert!(
            decode("og-save:3:AAAA")
                .unwrap_err()
                .contains("newer engine")
        );
        assert!(decode("og-save:1:@@@").unwrap_err().contains("damaged"));
        assert!(decode("og-save:1:AAAA").unwrap_err().contains("damaged"));
    }
}
