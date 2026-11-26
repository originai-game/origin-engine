//! Origin Engine `og-api`: the host contract generated from the API declared
//! in `api/<module>.ogt`, the traits `og-host-web` and `og-host-node` implement
//! (`og api gen` writes `src/generated/host.rs`), and the engine-owned record
//! tables, re-exported from `og-records`. The IDL, its validator and
//! generators are `og-idl`; the host-neutral runtime the hosts wrap is
//! `og-runtime`. See `crates/og-api/README.md`.
//!
//! The wire readers and binary envelope are shared by runtime bindings and
//! offline generators, so both consume the IDL's defaults and validation.
//! The cook's record-only consumers still depend on `og-records` directly.

/// The OGB1 binary wire envelope shared by hosts and offline generators.
pub mod bin;
/// Typed wire arguments generated from the IDL, with common JSON readers.
pub mod wire;

/// The host contract generated from the IDL: the traits `og-host-web` and
/// `og-host-node` implement (`og api gen` writes `src/generated/host.rs`).
#[rustfmt::skip]
#[path = "generated/host.rs"]
pub mod host;

/// The IDL's module constants, generated from `api/<module>.ogt`
/// (`og api gen` writes `src/generated/consts.rs`; the TypeScript side's is
/// `packages/engine/src/generated/consts.ts`).
#[rustfmt::skip]
#[path = "generated/consts.rs"]
pub mod consts;
