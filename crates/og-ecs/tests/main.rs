//! og-ecs's integration tests, one binary: each file under tests/ is a
//! module of it, so an edit relinks one test executable instead of one
//! per file (docs/ci.md §Test binaries). nextest still runs every test
//! in a process of its own. A new test file is a `mod` line here.

mod bouncing_boxes;
mod churn;
mod field_types;
mod host_schema;
mod introspection;
mod migrate;
mod packed_access;
mod schedule;
mod shared_schema;
mod snapshot;
mod state_blocks;
