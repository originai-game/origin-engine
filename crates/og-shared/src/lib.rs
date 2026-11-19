//! Origin Engine shared worlds (docs/design.md §2.12): visitors of one
//! world see each other's avatars in real time, near them only.
//!
//! - [`protocol`]: the binary messages, version 1, and the quantized avatar state.
//! - [`shard`]: one server's state with no IO: joins, movement checks, the
//!   interest grid, rate tiers under the byte cap, delta baselines.
//! - [`view`]: a client's side: remote avatars, acknowledgements, proposals.
//! - [`auth`]: token verifiers (`none`, `hs256`, `jwks`), offline.
//! - [`config`]: the `world-server` `.ogt` file.
//! - `store` (native): where the server keeps shared objects across restarts.
//! - `server` and `bots` (native): `og world-server` over WebSocket and
//!   WebTransport, and the load test; `cert` a self-signed WebTransport
//!   certificate a browser takes by hash.
//!
//! - `client` (native, the `client` feature): one connection to a server.
//!
//! Everything but `protocol`, `view` and `wire` is the `server` feature's.

#[cfg(feature = "server")]
pub mod auth;
#[cfg(all(feature = "server", not(target_arch = "wasm32")))]
pub mod bots;
#[cfg(all(feature = "server", not(target_arch = "wasm32")))]
pub mod cert;
#[cfg(all(feature = "client", not(target_arch = "wasm32")))]
pub mod client;
#[cfg(feature = "server")]
pub mod config;
pub mod protocol;
#[cfg(all(feature = "server", not(target_arch = "wasm32")))]
pub mod server;
#[cfg(feature = "server")]
pub mod shard;
#[cfg(all(feature = "server", not(target_arch = "wasm32")))]
pub mod store;
pub mod view;
pub mod wire;
#[cfg(all(
    any(feature = "server", feature = "client"),
    not(target_arch = "wasm32")
))]
pub mod wt;
