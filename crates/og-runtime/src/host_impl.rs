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
//! The expansion is split by module wave: `base` (Lifecycle through
//! ProbeApi), `e6`, `p2` and `depth`.

mod base;
mod depth;
mod e6;
mod p2;

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
    /// Bytes the engine hands out (a resource's packed record, UI replies).
    fn bytes_out(bytes: Vec<u8>) -> Self::Bytes;
    /// Bytes handed to the engine by value (`og.load.*Data`).
    fn bytes_in(bytes: Self::Bytes) -> Vec<u8>;
}

/// Implement every `og_api::host` trait but `Wire` for `$host`, a type that
/// implements `Wire` and [`HostWire`](crate::host_impl::HostWire).
#[macro_export]
macro_rules! host_bindings {
    ($host:ty) => {
        $crate::__og_host_bindings_base!($host);
        $crate::__og_host_bindings_e6!($host);
        $crate::__og_host_bindings_p2!($host);
        $crate::__og_host_bindings_depth!($host);
    };
}
