//! og-core's integration tests, one binary: each file under tests/ is a
//! module of it, so an edit relinks one test executable instead of one
//! per file (docs/ci.md §Test binaries). nextest still runs every test
//! in a process of its own. A new test file is a `mod` line here.

mod glam_scalar;
mod handle;
mod math;
mod ogt;
mod reflect;
mod rng;
