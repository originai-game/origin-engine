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
//! | [`StateBlock`] | a module's state outside archetypes and resources, saved and restored with the world |
//! | [`SnapshotEdit`], [`TypeValues`] | a snapshot opened for editing: what a save migration changes before a restore |
//! | [`SelectionReport`] | what restoring a selection (tagged entities and chosen resources) did |
//! | [`HookDesc`], [`StateDesc`], [`StateRun`] | add and remove hooks; state machines and the systems bound to their values |
//! | [`SpatialDesc`], [`SpatialHash`], [`Neighbour`] | a spatial hash of entities' positions: radius and k-nearest queries |
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
//!   resource. Change detection counts system runs, never a clock
//!   (`reactive`). [`World::snapshot`] captures everything that determines the
//!   future, [`World::restore`] brings it back, and [`World::world_hash`]
//!   hashes it.

mod edit;
mod entity;
mod query;
mod reactive;
mod registry;
mod schedule;
mod snapshot;
mod spatial;
mod state;
mod storage;
mod world;

pub use edit::{SelectionReport, SnapshotEdit, TypeValues};
pub use entity::Entity;
pub use query::{ArchetypeView, QueryDesc, QueryIter};
pub use reactive::{HookDesc, HookEvent, StateDesc, StateRun, StateWhen};
pub use registry::{FieldRef, SchemaId};
pub use schedule::{SystemContext, SystemDesc, SystemInfo};
pub use snapshot::{MigrationReport, snapshot_type_names};
pub use spatial::{Neighbour, SpatialDesc, SpatialHash};
pub use state::StateBlock;
pub use world::{Bundle, BundleEncoder, MAX_NAME, World, WorldDesc, last_tick};

pub use og_core::{Error, ErrorKind, Reflect, Result};
