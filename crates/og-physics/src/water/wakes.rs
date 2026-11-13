//! What the renderer reads of the bodies in a water: their cuts, tick by tick.

use super::Floater;
use crate::schema::WaterSurface;
use og_core::glam::{Vec2, Vec3};

/// A body cutting a surface in one tick, for the renderer: an
/// og.WaterSurface's ripples, an og.Water's wakes and hull wave suppression. Its
/// entity; its footprint in the surface's frame (xz relative to the
/// entity), as a box (`min`, `max`) and as the rectangle it turns with
/// (`centre`, `half` its half extents along its own axes, `axis` its own x
/// in xz, unit); how deep it reaches below the rest level (on a sea, below
/// the water at its centre) and its vertical speed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaterCut {
    pub entity: u64,
    pub min: Vec2,
    pub max: Vec2,
    pub centre: Vec2,
    pub half: Vec2,
    pub axis: Vec2,
    pub depth: f32,
    pub speed: f32,
}

impl WaterCut {
    /// A cut whose footprint is its box, unturned.
    pub fn boxed(entity: u64, min: Vec2, max: Vec2, depth: f32, speed: f32) -> WaterCut {
        WaterCut {
            entity,
            min,
            max,
            centre: (min + max) * 0.5,
            half: (max - min) * 0.5,
            axis: Vec2::X,
            depth,
            speed,
        }
    }
}

/// What the renderer reads of the bodies in a water: the bodies that cut
/// it in each of the last [`Wakes::TICKS`] ticks; of an og.WaterSurface
/// (its ripple layer) with the wave settings they travel at, of an og.Water
/// the boxes at its surface and the og.Buoyancy hulls in it (its wakes and
/// hull wave suppression). Not world state: never saved, never read
/// by the tick. A world made or restored starts a new `epoch`, so the
/// ripples drawn from it start again too.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Wakes {
    pub epoch: u64,
    /// The waves' speed as the height field runs them (held under its grid's
    /// limit), m/s.
    pub wave_speed: f32,
    /// The fraction of a wave kept after a second.
    pub damping: f32,
    pub splash: f32,
    /// The tick's length, s.
    pub dt: f32,
    /// (tick, the bodies that cut the surface after it), oldest first.
    pub ticks: std::collections::VecDeque<(u64, Vec<WaterCut>)>,
}

impl Wakes {
    /// How many ticks are kept: the most the ripples catch up in one frame.
    pub const TICKS: usize = 8;

    /// Records `tick`'s cuts: the floaters that straddle the rest level of
    /// a surface of side `size` at `centre`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record(
        &mut self,
        w: &WaterSurface,
        size: f32,
        cell: f32,
        dt: f32,
        tick: u64,
        centre: Vec3,
        floaters: &[Floater],
    ) {
        self.wave_speed = w.wave_speed.min(0.5 * cell / dt.max(1e-6));
        self.damping = w.damping.clamp(0.0, 1.0);
        self.splash = w.splash;
        self.dt = dt;
        let half = size * 0.5;
        let cuts = floaters
            .iter()
            .filter(|f| {
                f.min.y < centre.y
                    && f.max.y > centre.y
                    && f.max.x >= centre.x - half
                    && f.min.x <= centre.x + half
                    && f.max.z >= centre.z - half
                    && f.min.z <= centre.z + half
            })
            .map(|f| {
                WaterCut::boxed(
                    f.entity,
                    Vec2::new(f.min.x - centre.x, f.min.z - centre.z),
                    Vec2::new(f.max.x - centre.x, f.max.z - centre.z),
                    centre.y - f.min.y,
                    f.velocity.y,
                )
            })
            .collect();
        self.push(tick, cuts);
    }

    /// Records `tick`'s bodies in an og.Water (`cuts`: the boxes at its
    /// surface and og.Buoyancy's hulls): its waves are its own function, so
    /// no wave settings travel with them.
    pub(crate) fn record_water(&mut self, dt: f32, tick: u64, cuts: Vec<WaterCut>) {
        self.wave_speed = 0.0;
        self.damping = 0.0;
        self.splash = 0.0;
        self.dt = dt;
        self.push(tick, cuts);
    }

    fn push(&mut self, tick: u64, cuts: Vec<WaterCut>) {
        static EPOCHS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        // A tick that does not follow the last one (the world went back):
        // the ripples start again.
        if self.epoch == 0 || self.ticks.back().is_some_and(|(t, _)| *t >= tick) {
            self.epoch = EPOCHS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            self.ticks.clear();
        }
        self.ticks.push_back((tick, cuts));
        while self.ticks.len() > Self::TICKS {
            self.ticks.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wakes_keep_the_last_ticks_cuts_and_a_world_that_went_back_starts_a_new_epoch() {
        let w = WaterSurface::default();
        let crate_box = Floater {
            entity: 7,
            min: Vec3::new(0.5, -0.25, 1.0),
            max: Vec3::new(1.5, 0.75, 2.0),
            centre: Vec3::new(1.0, 0.25, 1.5),
            velocity: Vec3::new(0.0, -2.0, 0.0),
            dynamic: true,
            hull: false,
        };
        let above = Floater {
            entity: 8,
            min: Vec3::new(0.0, 1.0, 0.0),
            max: Vec3::new(1.0, 2.0, 1.0),
            ..crate_box
        };
        let centre = Vec3::new(10.0, 0.0, -4.0);
        let moved = |f: Floater| Floater {
            min: f.min + centre,
            max: f.max + centre,
            centre: f.centre + centre,
            ..f
        };
        let mut wakes = Wakes::default();
        for tick in 1..=12 {
            wakes.record(
                &w,
                16.0,
                0.25,
                1.0 / 60.0,
                tick,
                centre,
                &[moved(crate_box), moved(above)],
            );
        }
        let epoch = wakes.epoch;
        assert!(epoch != 0);
        let ticks: Vec<u64> = wakes.ticks.iter().map(|(t, _)| *t).collect();
        assert_eq!(ticks, (5..=12).collect::<Vec<u64>>());
        // Only the body across the rest level, in the surface's frame.
        let (_, cuts) = wakes.ticks.back().unwrap();
        assert_eq!(cuts.len(), 1);
        assert_eq!(cuts[0].entity, 7);
        assert_eq!(cuts[0].min, Vec2::new(0.5, 1.0));
        assert_eq!(cuts[0].max, Vec2::new(1.5, 2.0));
        assert_eq!(cuts[0].depth, 0.25);
        // The world went back (a restore, a rollback): a new epoch, the old ticks gone.
        wakes.record(&w, 16.0, 0.25, 1.0 / 60.0, 9, centre, &[moved(crate_box)]);
        assert_ne!(wakes.epoch, epoch);
        assert_eq!(wakes.ticks.len(), 1);
    }
}
