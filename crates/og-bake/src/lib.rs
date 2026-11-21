//! Origin Engine light baking (docs/design.md §2.6, `og bake lightmaps`).
//!
//! Pure CPU and native: lightmap UVs from xatlas ([`atlas`]), a BVH
//! ([`bvh`]), a progressive diffuse path tracer ([`trace`]) that fills
//! lightmap pages ([`lightmap`]) and spherical-harmonic light volumes
//! ([`volume`]), and the transfer of a detailed surface's look into a
//! simpler mesh's pages ([`transfer`], og-cook's HLOD proxies). The crate knows nothing about cooked files or the ECS:
//! og-cli's `bake_lightmaps.rs` builds a [`scene::Scene`] from a cooked
//! scene and writes what comes back.
//!
//! What is baked is the light that arrives from the scene: light bounced
//! off its surfaces, the sky, emissive surfaces, and the direct light of
//! its static lights (`og_render_data::light::LightMobility`). A stationary
//! light's direct term stays the renderer's, drawn every frame with its
//! shadows and specular highlights, so a baked scene keeps its moving
//! shadows and such a light can still be dimmed or switched off (its
//! bounce stays until the next bake); a movable light is not baked at all.

pub mod atlas;
pub mod bvh;
pub mod lightmap;
pub mod projection;
pub mod scene;
pub mod trace;
pub mod transfer;
pub mod volume;

pub use atlas::{Atlas, AtlasMesh, AtlasVertex, PackSettings, UnwrapMesh, unwrap};
pub use lightmap::{Lightmap, TraceSettings};
pub use scene::{Image, Material, Mesh, Scene};
pub use volume::{Volume, VolumeDesc};
