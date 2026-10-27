//! Origin Engine light baking (docs/design.md §2.6, `og bake lightmaps`).
//!
//! Pure CPU and native: lightmap UVs from xatlas ([`atlas`]), a BVH
//! ([`bvh`]), a progressive diffuse path tracer ([`trace`]) that fills
//! lightmap pages ([`lightmap`]) and spherical-harmonic light volumes
//! ([`volume`]). The crate knows nothing about cooked files or the ECS:
//! og-cli's `bake_lightmaps.rs` builds a [`scene::Scene`] from a cooked
//! scene and writes what comes back.
//!
//! What is baked is the light that arrives from the scene: light bounced
//! off its surfaces, the sky, emissive surfaces. The lights' direct term
//! stays the renderer's, drawn every frame with its shadows and specular
//! highlights, so a baked scene keeps its moving shadows and its lights can
//! still be switched off (their bounce stays until the next bake).

pub mod atlas;
pub mod bvh;
pub mod lightmap;
pub mod projection;
pub mod scene;
pub mod trace;
pub mod volume;

pub use atlas::{Atlas, AtlasMesh, AtlasVertex, PackSettings, UnwrapMesh, unwrap};
pub use lightmap::{Lightmap, TraceSettings};
pub use scene::{Image, Light, Material, Mesh, Scene};
pub use volume::{Volume, VolumeDesc};
