//! og.Water as the simulation reads it: a lake, a sea or a river, and its surface at a point.

use og_core::glam::Vec3;
use std::sync::Arc;

/// A body of water from an og.Water (a renderer's record the host reads),
/// for og.Buoyancy: a square of `size` centred on `position` (its y the
/// rest level), flat or raised by its `sea` (og.Water.wind), whose waves
/// are the absolute world's (`zero`, og.WorldOrigin's, added to a point):
/// the sea's own function ([`og_env::ocean::Ocean::surface`]), the one
/// the renderer draws. With a `shore` (W1: the
/// seabed og.Water.seabed names, in the frame's xz, and the trains rolling
/// in on it) the sea feels the depth ([`og_env::ocean::Ocean::surface_near`]):
/// a body in the shallows rides the shoaled, broken sea the renderer draws.
/// With a `river` (W5: og.Water.river, in the frame's xz) the water is the
/// river's ribbon alone, at its level along the course, its current the
/// water's velocity there, so a body drifts down it at the speed it is
/// drawn flowing.
#[derive(Clone, Debug, PartialEq)]
pub struct WaterBody {
    pub entity: u64,
    pub position: Vec3,
    pub size: og_core::glam::Vec2,
    pub sea: Option<og_env::ocean::Ocean>,
    pub zero: og_core::glam::DVec3,
    pub shore: Option<(Arc<og_env::ocean::Shore>, og_env::ocean::ShoreTrains)>,
    pub river: Option<Arc<og_env::river::River>>,
}

impl WaterBody {
    /// The surface's height over the rest level at world point (`x`, `z`)
    /// at `seconds` of the world's clock (its tick over its rate); `None`
    /// outside the square.
    pub fn height(&self, x: f64, z: f64, seconds: f64) -> Option<f32> {
        self.surface(x, z, seconds).map(|s| s.height)
    }

    /// The most the surface stands over or under its rest level anywhere
    /// (m): nothing for flat water, for a sea every wave's amplitude and
    /// its shore trains' together (shoaling only lowers a wave). A body
    /// further from the rest level than this is clear of the water.
    pub fn reach(&self) -> f32 {
        let Some(sea) = &self.sea else {
            return 0.0;
        };
        let trains = self.shore.as_ref().map_or(0.0, |(_, t)| t.amplitude);
        sea.waves.iter().map(|w| w.amplitude).sum::<f32>() + trains
    }

    /// [`Self::height`] with the water's velocity and acceleration there
    /// (its particles' orbital motion,
    /// [`og_env::ocean::Ocean::surface`]; still water without a
    /// sea).
    pub fn surface(&self, x: f64, z: f64, seconds: f64) -> Option<og_env::ocean::SurfacePoint> {
        if let Some(river) = &self.river {
            let at = river.at(og_core::glam::Vec2::new(x as f32, z as f32))?;
            return Some(og_env::ocean::SurfacePoint {
                height: at.level - self.position.y,
                velocity: Vec3::new(at.velocity.x, 0.0, at.velocity.y),
                acceleration: Vec3::ZERO,
            });
        }
        let half = self.size * 0.5;
        let local = og_core::glam::Vec2::new(
            (x - f64::from(self.position.x)) as f32,
            (z - f64::from(self.position.z)) as f32,
        );
        if !(local.x.abs() <= half.x && local.y.abs() <= half.y) {
            return None;
        }
        let Some(sea) = &self.sea else {
            return Some(og_env::ocean::SurfacePoint::default());
        };
        let coast = self
            .shore
            .as_ref()
            .map(|(shore, trains)| og_env::ocean::Coast {
                shore,
                trains: *trains,
                zero: og_core::glam::DVec2::new(self.zero.x, self.zero.z),
            });
        Some(sea.surface_near(
            og_core::glam::DVec2::new(x + self.zero.x, z + self.zero.z),
            seconds,
            coast.as_ref(),
        ))
    }
}
