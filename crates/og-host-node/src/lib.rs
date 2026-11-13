//! Origin Engine native host (`docs/design.md` §2.1, §5): the binding
//! symbols of `crates/og-api/api/<module>.ogt` as a napi addon (`origin-engine.node`),
//! headless, for `og run` and the executors. `packages/engine/src/node/`
//! loads it under bun or node.
//!
//! - `src/generated/glue.rs` (`og api gen`) exports every symbol under its
//!   snake_case name and calls the `og_api::host` trait method of the same
//!   name; `src/host.rs` states the native wire types and expands the
//!   traits' shared implementation (`og_runtime::host_impl`), so a member
//!   missing there is a build error.
//! - Column views alias engine memory: each field is an external
//!   ArrayBuffer over the column. The buffer is kept while the column stays
//!   where it is, and detached before anything can move it (a structural
//!   change, a restore), so a kept typed array reads as empty and never
//!   reaches moved memory. Where the runtime refuses external buffers (a V8
//!   sandbox), or with `OG_NODE_VIEWS=copy`, each field is copied into an
//!   ArrayBuffer (kept the same way) as the view is made and writable ones
//!   are copied back when the system call ends.
//! - `og_node_attach_offscreen` gives an engine an offscreen renderer
//!   (`GameDesc.render` picks the antialiasing, `OG_GFX_CULLING=cpu` forces
//!   the CPU cull); the probe's `capture` and
//!   the `screenshot` command render offscreen at the asked size and read it
//!   back blocking (`og_render::Renderer::read_offscreen`);
//!   `og_node_wait_gpu` waits for its GPU work, for benches that time a frame.
//! - `og_node_scripted_frames` marks an engine's frames as a script's (`og
//!   run` without `--realtime`): drawn at the configured quality, never
//!   sampled by automatic quality.
//! - og-probe is mounted by the runtime (`og_runtime`); this host adds
//!   its loopback WebSocket (`og_node_probe_listen`, `og_node_probe_serve`,
//!   og-probe's `ProbeServer` at `127.0.0.1:<port>/probe`), cooked files read
//!   from disk (`Hooks::read_asset`) and og-bundle's Basis transcoder for
//!   textures (the `Hooks::texture` default with og-bundle's `transcode`).

/// The exports, one per binding symbol (`og api gen`).
#[doc(hidden)]
#[rustfmt::skip]
#[path = "generated/glue.rs"]
pub mod glue;
mod audio;
mod hooks;
mod host;
mod offscreen;
mod probe;
#[cfg(not(feature = "publish"))]
mod render;
#[cfg(not(feature = "publish"))]
mod replay;
mod resident;
mod room;
#[cfg(not(feature = "publish"))]
mod ui_check;
mod wire;

pub(crate) use hooks::NodeHooks;
use offscreen::DEVICES;
use probe::SERVERS;
pub use wire::{EnvScope, NodeArray, NodeBytes, NodeFunction, NodeShared, NodeViews};

/// The engine's heap, counted for the probe's `memory` (og-core's `alloc`).
#[global_allocator]
static HEAP: og_runtime::CountingAlloc<std::alloc::System> =
    og_runtime::CountingAlloc(std::alloc::System);

/// The native host: a unit type, since the engines live in the runtime's
/// thread-local tables.
pub struct NodeHost;

/// The engine is destroyed: its probe socket stops and its renderer's
/// device is let go of.
pub(crate) fn forget_engine(engine: u32) {
    SERVERS.with(|s| s.borrow_mut().remove(&engine));
    DEVICES.with(|d| d.borrow_mut().remove(&engine));
}
