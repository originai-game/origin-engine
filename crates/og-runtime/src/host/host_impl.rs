//! The one implementation of the generated `og_api::host` traits that every
//! host shares. Almost every binding forwards to this crate's host-neutral
//! function of the same name; what really differs per host is how a value
//! that aliases or copies engine memory crosses the boundary and how the
//! dispatcher is installed, and a host states only that, as [`HostWire`].
//!
//! A host is a unit type that implements `og_api::host::Wire` and
//! [`HostWire`] and then expands [`host_bindings!`](crate::host_bindings)
//! for itself:
//!
//! ```ignore
//! impl Wire for WebHost { /* the wire types */ }
//! impl og_runtime::host_impl::HostWire for WebHost { /* what differs */ }
//! og_runtime::host_bindings!(WebHost);
//! ```
//!
//! The expansion implements every module trait in full, and the host's
//! generated glue calls each method, so a member the IDL gains and this
//! module lacks is a build error on every host (a missing trait item), not
//! a gap on one of them. A macro rather than a blanket
//! `impl<H: HostWire> WorldApi for H`: the traits are og-api's, and the
//! orphan rule refuses a blanket impl of a foreign trait here.
//!
//! The expansion is one macro per layer: `engine`, `world`, `input`, `present`, `assets`, `ui`, `animation`, `simulation`, `story` and `platform`.

mod animation;
mod assets;
mod engine;
mod input;
mod platform;
mod present;
mod simulation;
mod story;
mod ui;
mod world;

use crate::ColumnPtr;
/// The generated host traits the expansion implements.
#[doc(hidden)]
pub use og_api::host as api;
use og_api::host::Wire;

/// What a host does differently from the shared bindings: how it installs
/// the dispatcher and how the values of its `Wire` types are made.
///
/// Bytes handed to the engine are also read through `&bytes`, so a host's
/// `Wire::Bytes` derefs to `[u8]`.
pub trait HostWire: Wire {
    /// `og_host_set_dispatcher`: install the hooks that call `dispatcher`
    /// (`set_hooks`), and anything else the host sets up with them.
    fn install(dispatcher: Self::Function);
    /// `ArchetypeView.entities` and `.rows`: the runtime's values as the
    /// host's typed array.
    fn array(values: Vec<u32>) -> Self::Array;
    /// `ArchetypeView.column`: the runtime's column pointers (or its
    /// refusal) as the host's views; a refusal answers empty views.
    fn views(columns: Result<Vec<ColumnPtr>, String>) -> Self::Views;
    /// `og_host_shared`: the shared buffer (`crate::host::shared`) as the host
    /// hands it to its loader, installed first where the host owns it.
    fn shared() -> Self::Shared;
    /// Bytes the engine hands out (a resource's packed record, UI replies).
    fn bytes_out(bytes: Vec<u8>) -> Self::Bytes;
    /// Bytes handed to the engine by value (`og.load.*Data`).
    fn bytes_in(bytes: Self::Bytes) -> Vec<u8>;
    /// `og_engine_destroy` destroyed `engine`: drop what the host keeps for
    /// it (its audio output, its probe socket, its room link, its
    /// renderer's records). Engine ids are never reused, so what stays is
    /// held for the life of the thread.
    fn destroyed(engine: u32);
}

/// Implement every `og_api::host` trait but `Wire` for `$host`, a type that
/// implements `Wire` and [`HostWire`](crate::host::host_impl::HostWire).
#[macro_export]
macro_rules! host_bindings {
    ($host:ty) => {
        $crate::__og_host_bindings_engine!($host);
        $crate::__og_host_bindings_world!($host);
        $crate::__og_host_bindings_input!($host);
        $crate::__og_host_bindings_present!($host);
        $crate::__og_host_bindings_assets!($host);
        $crate::__og_host_bindings_ui!($host);
        $crate::__og_host_bindings_animation!($host);
        $crate::__og_host_bindings_simulation!($host);
        $crate::__og_host_bindings_story!($host);
        $crate::__og_host_bindings_platform!($host);
    };
}

/// The answer of a binding that returns `Result<T>`, as both hosts send
/// it: `{ ok: true, value }`, `{ ok: true }` for `Result<void>`, or
/// `{ ok: false, reason }`. Every such answer is written here.
pub fn answer<T: crate::host::wire::ToJson>(r: Result<T, String>) -> String {
    match r {
        Ok(_) if T::VOID => crate::ok_void(),
        Ok(v) => crate::ok(v.to_json()),
        Err(e) => crate::refuse(e),
    }
}

/// `{ ok: false, reason }` from a host's own member (its audio output, its
/// probe socket, its renderer), written as [`answer`] writes a refusal.
pub fn refuse(reason: &str) -> String {
    crate::refuse(reason)
}

/// A string as JSON text, for what a host's own member answers.
pub fn json_string(s: &str) -> String {
    serde_json::Value::String(s.to_string()).to_string()
}

/// The refusal of a host's audio member for an engine without og-audio.
pub fn no_audio(engine: u32) -> String {
    refuse(&format!(
        "engine {engine} has no audio: the runtime did not install og-audio for it (og_audio::install + publish_link at engine creation)"
    ))
}

/// The answer of a binding that returns a structured value outright: its
/// JSON text.
pub fn json<T: crate::host::wire::ToJson>(value: T) -> String {
    value.to_json().to_string()
}
