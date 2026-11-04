//! og-tests's integration tests, one binary: each file under tests/ is a
//! module of it, so an edit relinks one test executable instead of one
//! per file (docs/ci.md §Test binaries). nextest still runs every test
//! in a process of its own. A new test file is a `mod` line here.
//! Its own binary instead, declared in Cargo.toml: `render.rs`.

mod atmosphere;
mod cluster_compute;
mod culling;
mod debug_views;
mod effects;
mod exposure;
mod fluids3d;
mod fork_fields;
mod fsr;
mod geometry;
mod gfx;
mod graph;
mod hooks;
mod light;
mod lights;
mod materials;
mod moon;
mod ocean;
mod perf;
mod physical;
mod presentation;
mod probes;
mod removal;
mod render_origin;
mod shadows;
mod sprites2d;
mod stylise_line_toon_paint;
mod stylise_space;
mod taa_look;
mod taau;
mod targets;
mod terrain;
mod ui;
mod upload;
mod wasm;
