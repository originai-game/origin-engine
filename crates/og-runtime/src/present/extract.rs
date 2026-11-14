//! The world → `RenderScene` extraction and the renderer a host attached.
//!
//! Rendering reads the world after the tick and never writes it back
//! (AGENTS.md: rendering never feeds back into simulation). Everything here
//! takes the world shared, so the probe's `Capture` hook can extract the
//! frame of the current tick from the `&World` it is handed.
//!
//! What is drawn:
//! - entities with `og.Model` (a loaded glTF's default scene) at their world
//!   transform, composed from `og.Transform` down the `og.Parent` chain;
//! - entities with `og.Mesh` (geometry and material handles), when og-core
//!   can register its bool fields;
//! - the first `og.DirectionalLight` (else the renderer's default sun), the
//!   `og.PointLight` / `og.SpotLight` (og-render culls and clusters them);
//! - the `og.Camera` and `og.RenderSettings` resources;
//! - the text `og.render.text` queued during the tick, with cooked
//!   multi-page MSDF fonts mapped to one `MsdfFont` per page.

mod frame;
mod hierarchy;
mod wind;

use crate::{
    author::review, bind::modules, bind::skeleton2d, bind::terrain, bind::video, present::art,
    present::atmosphere, present::baked, present::environment, present::feel, present::fx,
    present::geometry, present::glyphs, present::interpolate, present::probes, present::views,
    world::assets,
};
pub(crate) use frame::extract;
pub use hierarchy::Hierarchy;
pub(crate) use hierarchy::{Blend, blend};

use crate::host::codec::{Cells, Col, ColRow, Layout, Rec, Slot as FieldSlot};
use crate::host::wire::FromJson;
use glam::{Mat4, Quat, Vec3};
use og_core::IdMap;
use og_ecs::{Entity, QueryDesc, World};
use og_render::{FontHandle, MaterialHandle, MeshHandle, RenderScene};
use serde_json::Value;
use std::collections::BTreeMap;

mod backend;
mod columns;
mod fonts;
mod state;

pub use backend::*;
pub(crate) use columns::*;
pub(crate) use fonts::*;
pub(crate) use state::*;

/// The IDL `AoResolutionSetting`'s variants, in order: RenderSettings.aoResolution.
const AO_RESOLUTION_SETTINGS: &[&str] = &["postEffects", "full", "half"];

/// RenderSettings.aoResolution's variant for a quality's AO resolution.
pub(crate) fn ao_resolution_setting(r: Option<og_render::AoResolution>) -> &'static str {
    AO_RESOLUTION_SETTINGS[match r {
        None => 0,
        Some(og_render::AoResolution::Full) => 1,
        Some(og_render::AoResolution::Half) => 2,
    }]
}

/// The IDL `Upscaler`'s variants, in order: RenderSettings.upscaler and
/// RenderCapabilities.upscalers.
pub(crate) const UPSCALERS: &[&str] = &["taau", "fsr3"];

/// RenderSettings.upscaler's variant for a quality's upscaler.
pub(crate) fn upscaler_setting(u: og_render::Upscaler) -> &'static str {
    UPSCALERS[match u {
        og_render::Upscaler::Taau => 0,
        og_render::Upscaler::Fsr3 => 1,
    }]
}

/// The layouts of the drawing components the engine registered.
#[derive(Default)]
pub(crate) struct DrawLayouts {
    pub model: Option<Layout>,
    pub mesh: Option<Layout>,
    pub directional: Option<Layout>,
    pub point: Option<Layout>,
    pub spot: Option<Layout>,
    /// og.RectLight (area lights).
    pub rect: Option<Layout>,
    /// The effect records (`crate::present::fx`).
    pub fx: crate::present::fx::FxLayouts,
    /// The atmosphere's and the physics extras' records (`crate::present::atmosphere`).
    pub atmo: crate::present::atmosphere::AtmoLayouts,
    /// og.Environment (`crate::present::environment`).
    pub environment: Option<Layout>,
    /// og.Terrain and og.TerrainScatter (`crate::bind::terrain`; the 2D
    /// profile draws neither).
    #[cfg(not(feature = "profile2d"))]
    pub terrain: Option<Layout>,
    #[cfg(not(feature = "profile2d"))]
    pub scatter: Option<Layout>,
    #[cfg(not(feature = "profile2d"))]
    pub grass: Option<Layout>,
    #[cfg(not(feature = "profile2d"))]
    pub stamp: Option<Layout>,
    /// og.GrassPusher (`crate::bind::terrain`, the trample map).
    #[cfg(not(feature = "profile2d"))]
    pub grass_pusher: Option<Layout>,
    /// og.GrassType (the kinds og.Grass fields list in `types`).
    #[cfg(not(feature = "profile2d"))]
    pub grass_type: Option<Layout>,
    /// og.Collider: what CPU-simulated particles bounce off.
    pub collider: Option<Layout>,
    /// og.Lines and og.Points (`crate::present::geometry`).
    pub lines: Option<Layout>,
    pub points: Option<Layout>,
    /// og.ReflectionProbe (`crate::present::probes`).
    pub probe: Option<Layout>,
    /// og.LightVolume and the lazy og.RealtimeLightVolume beside it
    /// (`crate::present::probes::extract_light_volumes`).
    pub light_volume: Option<Layout>,
    pub realtime_volume: Option<Layout>,
    /// og.Volume: the box of a reflection probe or light volume on its
    /// entity.
    pub volume: Option<Layout>,
    /// og.SightMap (`crate::bind::sight`): the fog of war's look.
    pub sight: Option<Layout>,
    /// og.Interpolation (`crate::present::interpolate`).
    pub interpolation: Option<Layout>,
    /// og.WorldOrigin (R12): the render origin's grid is in the absolute world.
    pub origin: Option<Layout>,
    /// og.CameraView and og.Mirror (`crate::present::views`).
    pub views: crate::present::views::ViewLayouts,
    /// The engine's physics, for the debug view (`crate::present::geometry`).
    pub physics: Option<og_physics::PhysicsSystem>,
    pub engine: u32,
}

#[cfg(test)]
mod tests;
