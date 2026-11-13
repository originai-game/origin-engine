//! The native host's side of the `og_api::host` traits: its wire types and
//! what it does differently (`HostWire`); every binding is the shared one of
//! `og_runtime::host_impl`, so a member the IDL gains and the shared
//! bindings lack is a build error here. Views, entities and the dispatcher
//! are where the native host differs.

use crate::{NodeArray, NodeBytes, NodeFunction, NodeHooks, NodeHost, NodeShared, NodeViews};
use og_api::host::Wire;
use og_runtime as rt;
use og_runtime::host_impl::HostWire;
use std::rc::Rc;

impl Wire for NodeHost {
    type Array = NodeArray;
    type Views = NodeViews;
    type Function = NodeFunction;
    /// A `Uint8Array` copied either way.
    type Bytes = NodeBytes;
    /// An `ArrayBuffer` JavaScript owns, kept for the thread's life.
    type Shared = NodeShared;
}

impl HostWire for NodeHost {
    fn install(dispatcher: Self::Function) {
        rt::set_hooks(Rc::new(NodeHooks { dispatcher }));
    }
    fn array(values: Vec<u32>) -> Self::Array {
        NodeArray::new(values)
    }
    fn views(columns: Result<Vec<rt::ColumnPtr>, String>) -> Self::Views {
        // One ArrayBuffer per field over the column (or a copy of it, see
        // `NodeViews`). Empty is a refusal.
        NodeViews(columns.unwrap_or_default())
    }
    fn shared() -> Self::Shared {
        NodeShared
    }
    fn bytes_out(bytes: Vec<u8>) -> Self::Bytes {
        NodeBytes(bytes)
    }
    fn bytes_in(bytes: Self::Bytes) -> Vec<u8> {
        bytes.0
    }
    fn destroyed(engine: u32) {
        crate::audio::og_node_audio_close(engine);
        crate::room::forget_engine(engine);
        crate::forget_engine(engine);
    }
}

og_runtime::host_bindings!(NodeHost);
