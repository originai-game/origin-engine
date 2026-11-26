//! `og replay`'s members (packages/engine/src/node/replay.ts), development
//! only:
//!
//! - `og_node_replay_hash`: the world's portable hash (og-net's
//!   `Snapshot::portable_hash`) within a type and field table, as
//!   `{"hash":"<016x>"}`; with an empty table, within the world's own,
//!   returned beside it (`{"hash","table"}`) for the trace to keep;
//! - `og_node_replay_diff`: what differs between a recorded save
//!   (`og-save:2:...`, local) and the world now (remote): og-net's
//!   `state::diff` as JSON, or `{"error": reason}`;
//! - `og_node_restart_diff`: og-net's `state::restart_diff` of a fresh
//!   start's save (start) against the world now, within an ignore table,
//!   each module block in its per-entity form and registry handles by
//!   what they name, for `og run --restart-at`.

// napi registers the exports outside `cfg(test)` only.
#![cfg_attr(test, allow(dead_code))]

use napi_derive::napi;
use og_net::Snapshot;
use og_net::state::{Ignore, diff, restart_diff, restart_kept};
use serde_json::json;
use std::collections::BTreeMap;

fn world(engine: u32) -> Result<Snapshot, String> {
    Snapshot::decode(&og_runtime::world_snapshot_bytes(engine)?)
}

fn hash(engine: u32, table_json: &str) -> Result<serde_json::Value, String> {
    let snapshot = world(engine)?;
    if table_json.trim().is_empty() {
        let table = snapshot.portable_table();
        let h = snapshot.portable_hash(&table);
        return Ok(json!({ "hash": format!("{h:016x}"), "table": table }));
    }
    let table: BTreeMap<String, Vec<String>> =
        serde_json::from_str(table_json).map_err(|e| format!("the table: {e}"))?;
    let h = snapshot.portable_hash(&table);
    Ok(json!({ "hash": format!("{h:016x}") }))
}

/// The world's portable hash within `table_json` (`{type: [field, ...]}`),
/// or within its own table, returned too, when that is empty.
#[napi(js_name = "og_node_replay_hash")]
pub fn og_node_replay_hash(engine: u32, table_json: String) -> String {
    match hash(engine, &table_json) {
        Ok(v) => v.to_string(),
        Err(e) => json!({ "error": e }).to_string(),
    }
}

/// og-net's diff of a recorded save (local) against the world (remote).
#[napi(js_name = "og_node_replay_diff")]
pub fn og_node_replay_diff(engine: u32, save_text: String) -> String {
    let diffed = og_runtime::save_snapshot_bytes(&save_text)
        .and_then(|b| Snapshot::decode(&b))
        .and_then(|recorded| Ok(diff(&recorded, &world(engine)?)));
    match diffed {
        Ok(v) => v.to_string(),
        Err(e) => json!({ "error": e }).to_string(),
    }
}

/// og-net's structural restart diff of a fresh start's save against the
/// world, leaving out `ignore_json` (`{type: [field, ...]}`, `[]` the whole
/// type, the engine's own) and the game's `keep_json` (`["Type.field", ...]`,
/// the progress it carries across a restart, which must exist), each kept
/// field reported under `kept` with its start and restart values.
#[napi(js_name = "og_node_restart_diff")]
pub fn og_node_restart_diff(
    engine: u32,
    save_text: String,
    ignore_json: String,
    keep_json: String,
) -> String {
    let diffed = serde_json::from_str::<Ignore>(&ignore_json)
        .map_err(|e| format!("the ignore table: {e}"))
        .and_then(|ignore| {
            let keep = serde_json::from_str::<Vec<String>>(&keep_json)
                .map_err(|e| format!("the keep list: {e}"))?;
            Ok((ignore, keep))
        })
        .and_then(|(mut ignore, keep)| {
            let mut start = Snapshot::decode(&og_runtime::save_snapshot_bytes(&save_text)?)?;
            let mut now = world(engine)?;
            // A snapshot carries no units: the tick numbers are named here, so
            // each reads from its own world's clock.
            let ticks = og_runtime::world_tick_fields(engine);
            for s in [&mut start, &mut now] {
                for t in &mut s.types {
                    for f in &mut t.schema.fields {
                        if ticks
                            .iter()
                            .any(|(ty, field)| *ty == t.name && *field == f.name)
                        {
                            f.unit = Some("tick".into());
                        }
                    }
                }
            }
            let (kept_ignore, kept) = restart_kept(&start, &now, &keep)?;
            // The queue of an event the input path delivered is input, as og.KeyDown's is.
            for name in og_runtime::world_input_events(engine) {
                ignore.insert(name, Vec::new());
            }
            for (ty, fields) in kept_ignore {
                match ignore.get_mut(&ty) {
                    // An empty list already leaves the whole type out.
                    Some(entry) if entry.is_empty() => {}
                    Some(entry) => entry.extend(fields),
                    None => {
                        ignore.insert(ty, fields);
                    }
                }
            }
            // Each module block read by the world's own block code, by entity.
            let forms = |name: &str, bytes: &[u8]| {
                let blocks = [(name.to_owned(), bytes.to_vec())];
                match og_runtime::world_block_forms(engine, &blocks) {
                    Ok(mut v) => v.pop().flatten(),
                    Err(e) => Some(Err(e)),
                }
            };
            // A grid's or a behaviour's handle by what it names.
            let names = |h: u64| og_runtime::world_handle_names(engine, h);
            let mut d = restart_diff(&start, &now, &ignore, &forms, &names);
            d["kept"] = json!(kept);
            Ok(d)
        });
    match diffed {
        Ok(v) => v.to_string(),
        Err(e) => json!({ "error": e }).to_string(),
    }
}
