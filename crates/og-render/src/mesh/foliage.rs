//! Cooked branch/leaf attributes and the wind contract shared by tree passes.
//!
//! Anchor-coherent detail motion follows SeedThree (MIT, SkyeShark 2026;
//! third_party/vegetation/SeedThree.LICENSE). Conservative deformation bounds
//! follow threejs-nanites' per-type wind-margin approach (MIT, Victor Zakharov
//! 2026; third_party/vegetation/threejs-nanites.LICENSE), derived here for
//! rotations rather than its translations. GPU Gems 3 chapter 16 and Pivot
//! Painter are technique references only; no Epic/NVIDIA source is included.

use crate::{WIND_WAVELENGTH, Wind};
use glam::Vec3;
pub use og_render_data::foliage::*;

/// Binding-free deformation include for meshlet and conventional vertex paths.
/// Prepend [`WIND_WGSL`] once: the gust sampler is shared with grass.
pub const WGSL: &str = wgsl!("foliage.wgsl");
pub const WIND_WGSL: &str = wgsl!("wind_common.wgsl");
/// The simulated wind field's lean the trunk bends by (after [`WIND_WGSL`]).
pub const WIND_LEAN_WGSL: &str = wgsl!("wind_lean.wgsl");

/// Both drawn states, shared by meshlets and conventional instancing.
pub(crate) fn foliage_winds(
    scene: &crate::RenderScene,
    previous: Option<(Wind, u64, f32)>,
    grid: crate::wind_field::WindGrid,
) -> ([Wind; 2], [FoliageWind; 2]) {
    let current = scene.atmosphere.wind.unwrap_or(Wind {
        strength: 0.0,
        ..Wind::default()
    });
    let (old, tick, rate) = previous.unwrap_or((current, scene.frame.tick, scene.frame.tick_rate));
    (
        [current, old],
        [
            FoliageWind::new(
                current,
                scene.frame.tick,
                scene.frame.tick_rate,
                0.0,
                scene.frame.origin,
                grid,
            ),
            FoliageWind::new(old, tick, rate, 0.0, scene.frame.origin, grid),
        ],
    )
}

/// `foliage.wgsl`'s explicit input; no bindings in that include.
/// Build the previous value from the last *drawn* wind, tick and rate, with the
/// current render origin (previous transforms have already been rebased).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FoliageWind {
    pub dir_strength: [f32; 4],
    /// gust, model height, and the two reduced travelling-wave phases.
    pub gust_top: [f32; 4],
    /// xyz: render origin; w: travelling-wave wavenumber.
    pub origin: [f32; 4],
    /// Its own drawn state's gust scroll, not the current wind at an old tick.
    field: crate::wind::WindFieldGpu,
}

impl FoliageWind {
    /// og.Wind at `tick`, read with the simulated field of grid `grid`
    /// (`wind_field.rs`; [`crate::wind_field::NO_GRID`]: analytic alone).
    pub fn new(
        wind: Wind,
        tick: u64,
        tick_rate: f32,
        top: f32,
        origin: Vec3,
        grid: crate::wind_field::WindGrid,
    ) -> Self {
        let phase = crate::wind_phases(&wind, tick, tick_rate);
        let direction = Vec3::new(wind.direction.x, 0.0, wind.direction.z).normalize_or(Vec3::X);
        Self {
            dir_strength: direction.extend(wind.strength.max(0.0)).to_array(),
            gust_top: [wind.gust.clamp(0.0, 1.0), top, phase[0], phase[1]],
            origin: origin
                .extend(std::f32::consts::TAU / WIND_WAVELENGTH)
                .to_array(),
            field: crate::wind::field_gpu(Some(wind), tick, tick, tick_rate, origin, grid),
        }
    }
}
