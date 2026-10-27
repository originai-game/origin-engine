//! `og replay`'s members (packages/engine/src/node/replay.ts), development
//! only:
//!
//! - `og_node_replay_hash`: the world's portable hash (og-net's
//!   `Snapshot::portable_hash`) within a type and field table, as
//!   `{"hash":"<016x>"}`; with an empty table, within the world's own,
//!   returned beside it (`{"hash","table"}`) for the trace to keep;
//! - `og_node_replay_diff`: what differs between a recorded save
//!   (`og-save:1:...`, local) and the world now (remote): og-net's
//!   `state::diff` as JSON, or `{"error": reason}`.

// napi registers the exports outside `cfg(test)` only.
#![cfg_attr(test, allow(dead_code))]

use napi_derive::napi;
use og_net::Snapshot;
use og_net::state::diff;
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
