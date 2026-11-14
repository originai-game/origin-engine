//! The reflected types of water: the height-field surface, the 2D line,
//! the splash event and buoyancy.

use og_core::Reflect;
use og_core::glam::Vec3;
use serde::{Deserialize, Serialize};

/// Height-field water (3D): a `resolution` × `resolution` grid of heights
/// over a `size` square centred on the entity (its y is the rest level),
/// waves travelling at `waveSpeed` and fading by `damping`. Dynamic bodies
/// crossing it push waves (splashes; og.Splash events) and float by
/// `buoyancy`. The heights are world state; an og.Water on the same entity
/// draws them.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(name = "og.WaterSurface", version = 2)]
pub struct WaterSurface {
    /// Grid points along each side.
    #[reflect(min = 8, max = 256, default = 64)]
    pub resolution: u32,
    /// The side of the square of water.
    #[reflect(min = 0.1, unit = "m", default = 16, tunable)]
    pub size: f32,
    /// How fast waves travel.
    #[reflect(name = "waveSpeed", min = 0, unit = "m/s", default = 3, tunable)]
    pub wave_speed: f32,
    /// How fast waves die (0: no loss).
    #[reflect(min = 0, max = 100, unit = "1/s", default = 0.9, tunable)]
    pub damping: f32,
    /// How strongly bodies push waves.
    #[reflect(min = 0, default = 1, tunable)]
    pub splash: f32,
    /// The upward force on a submerged body per cubic metre of its box under water (0: bodies sink).
    #[reflect(min = 0, unit = "N/m3", default = 9.81, tunable)]
    pub buoyancy: f32,
    /// How much a submerged body's velocity is slowed.
    #[reflect(min = 0, unit = "1/s", default = 1.5, tunable)]
    pub drag: f32,
}

impl Default for WaterSurface {
    fn default() -> Self {
        WaterSurface {
            resolution: 64,
            size: 16.0,
            wave_speed: 3.0,
            damping: 0.9,
            splash: 1.0,
            buoyancy: 9.81,
            drag: 1.5,
        }
    }
}

/// 2D water for side-scrollers: a `width` × `depth` pool whose top edge
/// (at the entity's y, centred on its x) is `columns` springs, each pulled
/// to its rest height by `tension`, damped by `damping` and passing waves to
/// its neighbours by `spread`. Bodies (and characters) crossing the surface
/// splash it (og.Splash events) and float by `buoyancy`. The springs are
/// world state; an og.Water2D on the same entity draws it.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(name = "og.WaterLine")]
pub struct WaterLine {
    /// The pool's width.
    #[reflect(min = 0.1, unit = "m", default = 12, tunable)]
    pub width: f32,
    /// The pool's depth below the surface.
    #[reflect(min = 0, unit = "m", default = 3, tunable)]
    pub depth: f32,
    /// Springs along the surface.
    #[reflect(min = 4, max = 512, default = 96)]
    pub columns: u32,
    /// How hard each spring pulls back to rest.
    #[reflect(min = 0, default = 60, tunable)]
    pub tension: f32,
    /// How fast a spring's motion dies.
    #[reflect(min = 0, default = 2.5, tunable)]
    pub damping: f32,
    /// How much of a spring's height its neighbours feel each pass (waves travel).
    #[reflect(min = 0, max = 0.5, default = 0.25, tunable)]
    pub spread: f32,
    /// How strongly crossing bodies splash.
    #[reflect(min = 0, default = 0.04, tunable)]
    pub splash: f32,
    /// The upward force on a submerged body per square metre of its box under water (0: bodies sink).
    #[reflect(min = 0, unit = "N/m2", default = 12, tunable)]
    pub buoyancy: f32,
    /// How much a submerged body's velocity is slowed.
    #[reflect(min = 0, unit = "1/s", default = 2, tunable)]
    pub drag: f32,
}

impl Default for WaterLine {
    fn default() -> Self {
        WaterLine {
            width: 12.0,
            depth: 3.0,
            columns: 96,
            tension: 60.0,
            damping: 2.5,
            spread: 0.25,
            splash: 0.04,
            buoyancy: 12.0,
            drag: 2.0,
        }
    }
}

/// Something crossed a water surface (an og.Water, og.WaterSurface or
/// og.WaterLine): for splash particles and sounds. Read the next tick.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Splash")]
pub struct Splash {
    /// The water's entity.
    pub water: u64,
    /// What crossed it.
    pub entity: u64,
    /// Where it crossed the surface.
    #[reflect(unit = "m")]
    pub point: Vec3,
    /// Its vertical speed (negative going in).
    #[reflect(unit = "m/s")]
    pub speed: f32,
}

/// Floats a dynamic rigid body (3D profile) on the water under it: an
/// og.WaterSurface, an og.Water (a flat lake, or a sea with a wind: its
/// waves as drawn) or both on one entity. Its hull is its colliders' box
/// cut into `samples` cells a side, kept where a cell's centre lies in a
/// collider; each cell under the surface at its point pushes up by the
/// water it displaces (Archimedes: `waterDensity` × gravity × volume) at
/// that point, so a body rights itself and pitches with the waves, and is
/// slowed by `drag` and `angularDrag` as far as it is under. Lazily
/// registered: a world without it keeps its hash. A body with it takes no
/// og.WaterSurface `buoyancy` (only its splash).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Buoyancy")]
pub struct Buoyancy {
    /// The water's density, in og.Collider.density's units: the default 2 floats a body of the colliders' default density 1 half under, as wood floats; at 1 the same body hangs just under the surface.
    #[reflect(name = "waterDensity", min = 0, unit = "kg/m3", default = 2, tunable)]
    pub water_density: f32,
    /// Cells along each side of its colliders' box (1: one point at its centre; up to 12³ points): more follow its shape and the waves more closely.
    #[reflect(min = 1, max = 12, default = 4)]
    pub samples: u32,
    /// How much of its speed through the water it loses each second when fully under (in part when in part); off-centre cells slow its spin too.
    #[reflect(min = 0, max = 100, unit = "1/s", default = 4, tunable)]
    pub drag: f32,
    /// How much of its spin it loses each second when fully under (in part when in part).
    #[reflect(
        name = "angularDrag",
        min = 0,
        max = 100,
        unit = "1/s",
        default = 3,
        tunable
    )]
    pub angular_drag: f32,
    /// Written: the fraction of its hull under water after the last step (0 afloat above it or out of water, 1 sunk).
    #[reflect(min = 0, max = 1)]
    pub submerged: f32,
}

impl Default for Buoyancy {
    fn default() -> Self {
        Buoyancy {
            water_density: 2.0,
            samples: 4,
            drag: 4.0,
            angular_drag: 3.0,
            submerged: 0.0,
        }
    }
}
