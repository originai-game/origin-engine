//! Origin Engine `og-api`: the host contract generated from the API declared
//! in `api/<module>.ogt`, the traits `og-host-web` and `og-host-node` implement
//! (`og api gen` writes `src/generated/host.rs`), and the engine-owned record
//! tables, re-exported from `og-records`. The IDL, its validator and
//! generators are `og-idl`; the host-neutral runtime the hosts wrap is
//! `og-runtime`. See `crates/og-api/README.md`.
//!
//! This crate depends only on `og-records` and changes only when the
//! generated Rust does, so neither the generators nor the runtime rebuild
//! what depends on the contract alone; `og`'s cook reads only the records.

/// The host contract generated from the IDL: the traits `og-host-web` and
/// `og-host-node` implement (`og api gen` writes `src/generated/host.rs`).
#[rustfmt::skip]
#[path = "generated/host.rs"]
pub mod host;
