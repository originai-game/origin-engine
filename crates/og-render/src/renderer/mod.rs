//! Renderer implementation and GPU resources.

use super::*;

mod assets;
mod batches;
mod buffers;
mod config;
mod init;
mod pipelines;
mod presentation;
mod readback;
mod render;
mod resources;
mod settings;
mod shader_source;
mod uploads;
pub(crate) use buffers::*;
pub use config::*;
pub(crate) use pipelines::*;
pub use presentation::{GpuMemory, MAX_RENDER_SCALE, MIN_RENDER_SCALE, Successor};
pub(crate) use presentation::{LostFlag, texture_bytes, view_bytes};
pub(crate) use resources::*;
pub(crate) use shader_source::*;
