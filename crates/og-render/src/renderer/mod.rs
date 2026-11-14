//! Renderer implementation and GPU resources.

use super::*;

mod assets;
mod batches;
pub(crate) use batches::{InstanceRecords, Visibility};
mod buffers;
mod config;
pub(crate) mod history;
mod init;
mod pipelines;
pub(crate) use pipelines::{LitPipelines, PrepassState};
mod presentation;
mod readback;
mod render;
pub(crate) use render::Particles;
mod resources;
pub(crate) use resources::{Lights, Shadows};
mod settings;
mod shader_source;
mod uploads;
pub(crate) use buffers::*;
pub use config::*;
pub use history::Restored;
pub(crate) use pipelines::*;
pub(crate) use presentation::LostFlag;
pub use presentation::{GpuMemory, MAX_RENDER_SCALE, MIN_RENDER_SCALE, Successor};
pub(crate) use resources::*;
pub(crate) use shader_source::*;
