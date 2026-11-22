//! The file formats og-shape's tools read and write: glTF, OBJ, FBX, STL,
//! USD and URDF; Draco and meshopt; KTX2 (UASTC, ETC1S) with Zstandard;
//! PNG, JPEG and WebP. A different link set from the kernel: only the cook
//! and the `og` tool depend on this crate, never the runtime
//! (`crates/og-core/tests/layers.rs`). Every reader returns errors as
//! values; [`texture::CODEC`] is what the kernel's preview and bake take.

pub mod draco;
pub mod fbx;
pub mod gltf;
pub mod import;
pub mod jpeg;
pub mod json;
pub mod ktx2;
pub mod meshopt;
pub mod model;
pub mod obj;
pub mod png;
pub mod stl;
pub mod texture;
pub mod urdf;
pub mod usd;
pub mod webp;
pub mod zstd;

pub use import::import;
pub use model::{AtlasExport, ModelFiles};
