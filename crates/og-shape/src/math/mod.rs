//! Foundations of the og-shape kernel.
//!
//! Everything here exists so that the rest of the kernel gives the same
//! bytes on x86-64, aarch64 and wasm32 (`AGENTS.md`, `docs/design.md` §6):
//!
//! - [`DVec3`] and [`Frame`]: f64 vectors and frames, scalar only.
//! - Every transcendental function comes from `og_core::math::double`, the
//!   engine's f64 maths (`+ - * /` and `sqrt` only).
//! - [`predicates`]: exact geometric sides (Shewchuk's predicates).
//! - [`order`]: total, sign-of-zero-insensitive ordering for canonical
//!   output.
//! - The one stable sort is `og_core::sort`, the engine's.
//! - [`hash`]: the output hash the gate compares across targets.
//!
//! The determinism lint (`clippy.toml`) forbids the `std` maths and the
//! hash containers these replace.

pub mod frame;
pub mod hash;
pub mod order;
pub mod predicates;

pub use frame::Frame;
pub use glam::{DVec2, DVec3};
pub use hash::{HashError, OutputHash};
