//! The web host's side of the `og_api::host` traits: its wire types and
//! what it does differently (`HostWire`); every binding is the shared one of
//! `og_runtime::host_impl`, so a member the IDL gains and the shared
//! bindings lack is a build error here. Views, entities and the dispatcher
//! are where the web differs.

use crate::{WebHooks, WebHost};
use og_api::host::Wire;
use og_runtime as rt;
use og_runtime::host_impl::HostWire;
use std::rc::Rc;

impl Wire for WebHost {
    /// Copied into a fresh `Uint32Array` by wasm-bindgen.
    type Array = Vec<u32>;
    /// Byte offsets of each field's column in `memory.buffer`.
    type Views = Vec<u32>;
    type Function = js_sys::Function;
    /// Copied by wasm-bindgen: a fresh `Uint8Array` out, a copy of one in.
    type Bytes = Vec<u8>;
    /// The buffer's byte offset in `memory.buffer`.
    type Shared = u32;
}

impl HostWire for WebHost {
    fn install(dispatcher: Self::Function) {
        rt::set_hooks(Rc::new(WebHooks { dispatcher }));
        rt::set_physics_clock(crate::clock_ms);
        og_lanes::set_trace_clock(crate::epoch_ms);
    }
    fn array(values: Vec<u32>) -> Self::Array {
        values
    }
    fn views(columns: Result<Vec<rt::ColumnPtr>, String>) -> Self::Views {
        // Byte offsets into the wasm memory, one per field; the loader makes
        // typed arrays over `memory.buffer` at them. Empty is a refusal.
        columns
            .map(|cols| cols.iter().map(|c| c.ptr as usize as u32).collect())
            .unwrap_or_default()
    }
    fn shared() -> Self::Shared {
        // The runtime's own buffer, in the module's memory like the columns.
        rt::shared::address() as usize as u32
    }
    fn bytes_out(bytes: Vec<u8>) -> Self::Bytes {
        bytes
    }
    fn bytes_in(bytes: Self::Bytes) -> Vec<u8> {
        bytes
    }
}

og_runtime::host_bindings!(WebHost);
