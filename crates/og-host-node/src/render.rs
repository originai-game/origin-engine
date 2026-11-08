//! `og render`'s members (docs/design.md §2.9), development only:
//!
//! - `og_node_render_frame`: the frame the loop drew last as RGBA8 rows (a
//!   Buffer of width × height × 4 bytes at the offscreen target's size),
//!   read back without drawing it again; empty without one;
//! - `og_node_render_snapshot`: the world as a render snapshot (a Buffer),
//!   which `og_node_render_restore` brings into another process of the same
//!   game and script (wire JSON: the tick, or the reason);
//! - `og_node_settle_pipelines`: waits for every pipeline compile started
//!   so far (og_render::settle_pipelines), for a test that compares
//!   display frames byte for byte.

// napi registers the exports outside `cfg(test)` only.
#![cfg_attr(test, allow(dead_code))]

use napi::bindgen_prelude::Buffer;
use napi_derive::napi;

/// The last frame drawn, RGBA8 rows top first; empty without a renderer
/// or before the first frame. With a header, two little-endian u32s carry
/// the actual width and height, including a resize during the run.
#[napi(js_name = "og_node_render_frame")]
pub fn og_node_render_frame(engine: u32, header: Option<bool>) -> Buffer {
    match og_runtime::render_frame(engine) {
        Ok(f) if header == Some(true) => {
            let mut bytes = Vec::with_capacity(8 + f.rgba.len());
            bytes.extend_from_slice(&f.width.to_le_bytes());
            bytes.extend_from_slice(&f.height.to_le_bytes());
            bytes.extend_from_slice(&f.rgba);
            bytes.into()
        }
        Ok(f) => f.rgba.into(),
        Err(_) => Vec::new().into(),
    }
}

/// The world as a render snapshot; empty when the engine refused.
#[napi(js_name = "og_node_render_snapshot")]
pub fn og_node_render_snapshot(engine: u32) -> Buffer {
    og_runtime::render_snapshot(engine)
        .unwrap_or_default()
        .into()
}

/// Restores a render snapshot: `{ok, value: tick}` or `{ok: false, reason}`.
#[napi(js_name = "og_node_render_restore")]
pub fn og_node_render_restore(engine: u32, snapshot: Buffer) -> String {
    match og_runtime::render_restore(engine, &snapshot) {
        Ok(tick) => serde_json::json!({ "ok": true, "value": tick }).to_string(),
        Err(reason) => serde_json::json!({ "ok": false, "reason": reason }).to_string(),
    }
}

/// Waits until every pipeline compile the process started has finished
/// (og_render::settle_pipelines): the display frames after it draw with
/// the full pipeline variants.
#[napi(js_name = "og_node_settle_pipelines")]
pub fn og_node_settle_pipelines() {
    og_render::settle_pipelines();
}
