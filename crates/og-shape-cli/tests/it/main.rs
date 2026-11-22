//! og-shape-cli's integration tests: the commands, and the golden hashes
//! of every og-shape operation (the one crate that sees the kernel, the
//! formats and the commands).

mod cli;
#[cfg(not(target_arch = "wasm32"))]
mod fbx;
mod golden;
