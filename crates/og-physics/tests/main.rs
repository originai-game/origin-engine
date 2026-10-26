//! og-physics's integration tests, one binary: each file under tests/ is a
//! module of it, so an edit relinks one test executable instead of one
//! per file (docs/ci.md §Test binaries). nextest still runs every test
//! in a process of its own. A new test file is a `mod` line here.
//! Its own binary instead, declared in Cargo.toml: `lanes.rs`, `pile.rs`.

mod beyond_determinism;
mod bodies;
mod character_ground;
mod contact_hits;
mod controllers;
mod edits;
mod fluid3d;
mod fluid_groups;
mod forces;
mod heightfield;
mod joints;
mod platformer_ground;
mod profile_2d;
mod queries;
mod ragdoll;
mod saves;
mod self_collision;
mod soft;
mod support;
mod time_scale;
mod vehicles;
