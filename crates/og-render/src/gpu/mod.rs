//! The device and what lives on it: capabilities, targets, textures, the pipeline cache, objects made on first use.

pub(crate) mod buffers;
pub mod caps;
pub(crate) mod copy;
pub(crate) mod deferred;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod headless;
pub(crate) mod indirect_own;
pub(crate) mod layout;
pub(crate) mod mips;
pub(crate) mod pipeline_cache;
#[cfg(any(target_os = "macos", target_os = "linux", windows))]
pub mod shared_frames;
pub(crate) mod targets;
pub(crate) mod texture;
pub(crate) mod texture_array;
pub(crate) mod video;
