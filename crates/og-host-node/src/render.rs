//! `og render`'s members (docs/design.md §2.9), development only:
//!
//! - `og_node_render_frame`: the frame the loop drew last as RGBA8 rows (a
//!   Buffer of width × height × 4 bytes at the offscreen target's size),
//!   read back without drawing it again; empty without one;
//! - `og_node_settle_pipelines`: waits for every pipeline compile started
//!   so far (og_render::settle_pipelines), for a test that compares
//!   display frames byte for byte.

// napi registers the exports outside `cfg(test)` only.
#![cfg_attr(test, allow(dead_code))]

use napi::Env;
use napi::bindgen_prelude::BufferSlice;
use napi_derive::napi;

/// The last frame drawn, RGBA8 rows top first; empty without a renderer
/// or before the first frame. With a header, two little-endian u32s carry
/// the actual width and height, including a resize during the run.
#[napi(js_name = "og_node_render_frame")]
pub fn og_node_render_frame(
    env: &Env,
    engine: u32,
    header: Option<bool>,
) -> napi::Result<BufferSlice<'_>> {
    let bytes = match og_runtime::render_frame(engine) {
        Ok(f) if header == Some(true) => {
            let mut bytes = Vec::with_capacity(8 + f.rgba.len());
            bytes.extend_from_slice(&f.width.to_le_bytes());
            bytes.extend_from_slice(&f.height.to_le_bytes());
            bytes.extend_from_slice(&f.rgba);
            bytes
        }
        Ok(f) => f.rgba,
        Err(_) => Vec::new(),
    };
    // JS owns the copy and Rust frees the readback before returning. An
    // external Buffer defers freeing its Vec to a napi finalizer; Bun queues
    // those until the event loop runs, so og render's synchronous loop keeps
    // one native RGBA allocation per frame even when GC collects the wrappers.
    BufferSlice::copy_from(env, bytes)
}

/// Waits until every pipeline compile the process started has finished
/// (og_render::settle_pipelines): the display frames after it draw with
/// the full pipeline variants.
#[napi(js_name = "og_node_settle_pipelines")]
pub fn og_node_settle_pipelines() {
    og_render::settle_pipelines();
}
