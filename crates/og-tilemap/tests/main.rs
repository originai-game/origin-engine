//! og-tilemap's integration tests, one binary: each file under tests/ is a
//! module of it, so an edit relinks one test executable instead of one
//! per file (docs/ci.md §Test binaries). nextest still runs every test
//! in a process of its own. A new test file is a `mod` line here.

mod autotile;
mod chunks;
mod collision;
mod state;
mod support;
mod text;
