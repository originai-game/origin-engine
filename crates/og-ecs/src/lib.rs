//! Origin Engine world: an archetype ECS advanced at a fixed tick.
//!
//! | item | what it is |
//! |---|---|
//! | [`World`], [`WorldDesc`] | the simulation: entities, components, resources, events, schedule |
//! | [`Entity`] | a generational entity id (`u64`: `generation << 32 \| index`) |
//! | [`SchemaId`], [`FieldRef`] | a registered reflected type, and one of its fields |
//! | [`QueryDesc`], [`QueryIter`], [`ArchetypeView`] | queries yielding per-field column slices |
//! | [`SystemDesc`], [`SystemContext`] | declared systems and what they may touch |
//! | [`Bundle`] | `()` or a tuple of components for spawning |
//!
//! The model:
//!
//! - **Types** are reflected schemas ([`og_core::Reflect`]); Rust types
//!   register with [`World::register`], hosts register a layout by schema with
//!   [`World::register_schema`]. A type can be a component, a resource and an
//!   event; names are unique.
//! - **Storage** is one archetype per component set, one 16-byte-aligned
//!   column per (component, field). Values cross the API as typed values or
//!   as packed bytes (fields in schema order, little-endian).
//! - **Systems** declare what they read and write. Registration refuses a
//!   query over undeclared components; at run time every access is checked
//!   against the declaration. Structural changes are commands applied, in
//!   order, when the system returns: a second despawn of the same entity is a
//!   no-op; an insert or remove after a despawn of the same entity stops the
//!   tick (the commands before it stay applied, the rest are dropped).
//! - **Ticks**: [`World::step`] delivers pending events, runs systems in
//!   order, then advances [`og_core::Time`]. Events sent during tick N are
//!   read during tick N+1.
//! - **Determinism**: iteration follows archetype creation order and row
//!   order; nothing iterates a hash map; the world's [`og_core::Rng`] is a
//!   resource. [`World::snapshot`] captures everything that determines the
//!   future, [`World::restore`] brings it back, and [`World::world_hash`]
//!   hashes it.

mod entity;
mod query;
mod registry;
mod schedule;
mod snapshot;
mod storage;
mod world;

pub use entity::Entity;
pub use query::{ArchetypeView, QueryDesc, QueryIter};
pub use registry::{FieldRef, SchemaId};
pub use schedule::{SystemContext, SystemDesc};
pub use world::{Bundle, BundleEncoder, World, WorldDesc};

pub use og_core::{Error, ErrorKind, Reflect, Result};
