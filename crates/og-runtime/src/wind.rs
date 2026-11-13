//! og.Wind, the world's wind: read from the world in one place, and what
//! each thing that moves in the air takes from it, so vegetation, clouds,
//! weather and the soft bodies cannot disagree about where the air goes.
//!
//! The tick reads it, and the og.WindMotors pushing on it, for physics
//! ([`before_tick`]: the air ropes, cloths, garments and bodies with an
//! airDrag move through); extraction reads them for what is drawn
//! (`super::atmosphere`). Each of those adds its record's own `wind` to it.

use super::codec::Layout;
use super::fx::{kept_rows, resource};
use super::wire::{FromJson, WindMotorKind, record};
use glam::Vec3;
use og_ecs::World;

/// How much faster the air carries the clouds than it blows at the ground.
const ALOFT: f32 = 2.0;

/// The world's wind this tick: still without an og.Wind.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct WorldWind {
    /// Where the air moves over the ground and how fast (m/s, horizontal).
    pub velocity: Vec3,
    gust: f32,
    turbulence: f32,
    gust_size: f32,
}

impl WorldWind {
    /// og.Wind as the world holds it.
    pub(crate) fn read(world: &World, layout: Option<&Layout>) -> WorldWind {
        resource(world, layout)
            .and_then(|w| record::Wind::from_json(&w).ok())
            .map_or_else(WorldWind::default, |w| WorldWind {
                velocity: Vec3::new(w.direction.x, 0.0, w.direction.z).normalize_or_zero()
                    * w.speed.max(0.0),
                gust: w.gust,
                turbulence: w.turbulence,
                gust_size: w.gust_size,
            })
    }

    /// What vegetation sways in: `None` in still air.
    pub(crate) fn vegetation(self) -> Option<og_render::Wind> {
        let speed = self.velocity.length();
        (speed > 0.0).then(|| og_render::Wind {
            gust: self.gust,
            turbulence: self.turbulence,
            gust_size: self.gust_size,
            ..og_render::Wind::blowing(self.velocity / speed, speed)
        })
    }

    /// What carries the clouds (m/s).
    pub(crate) fn aloft(self) -> Vec3 {
        self.velocity * ALOFT
    }
}

/// How a motor pushes, as the air takes it.
fn kind_of(kind: WindMotorKind) -> og_core::air::MotorKind {
    match kind {
        WindMotorKind::Directional => og_core::air::MotorKind::Directional,
        WindMotorKind::Vortex => og_core::air::MotorKind::Vortex,
        WindMotorKind::Omni => og_core::air::MotorKind::Omni,
    }
}

/// As each tick starts: the world's air (og.Wind and every og.WindMotor at
/// its entity's world pose, in entity order) to the physics that moves
/// ropes, cloths, garments and bodies through it.
pub(crate) fn before_tick(
    physics: &og_physics::PhysicsSystem,
    r: &super::physics_ext::Reader<'_>,
    world: &World,
    wind: Option<&Layout>,
    motors: Option<&Layout>,
) {
    let motors = r
        .rows(motors)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(e, v)| {
            let v = record::WindMotor::from_json(&v).ok()?;
            let (position, turn, _) = r.pose(og_ecs::Entity::from_bits(e))?;
            Some(og_core::air::Motor {
                position,
                direction: turn * Vec3::NEG_Z,
                radius: v.radius,
                strength: v.strength,
                kind: kind_of(v.kind),
            })
        })
        .collect();
    physics.set_air(og_core::air::Air {
        wind: WorldWind::read(world, wind).velocity,
        motors,
        ..Default::default()
    });
}

/// og.Wind and the og.WindMotors into `scene`: what vegetation sways in.
/// Returns the world's wind for what else moves in it this frame.
pub(crate) fn extract(
    world: &World,
    scene: &mut og_render::RenderScene,
    wind: Option<&Layout>,
    motors: Option<&Layout>,
    h: &mut super::extract::Hierarchy,
) -> WorldWind {
    // og.Wind: what vegetation sways in here; the clouds, the weather and
    // the visual cloths below add their own to it.
    let world_wind = WorldWind::read(world, wind);
    scene.atmosphere.wind = world_wind.vegetation();
    // og.WindMotor: local pushes on the simulated wind field, placed by
    // their entities (wind_field.rs keeps the nearest).
    scene.atmosphere.wind_motors.clear();
    for (e, v) in kept_rows(world, motors) {
        let Ok(v) = record::WindMotor::from_json(&v) else {
            continue;
        };
        let m = h.world_of(e);
        scene.atmosphere.wind_motors.push(og_render::WindMotor {
            position: m.transform_point3(Vec3::ZERO),
            direction: m.transform_vector3(Vec3::NEG_Z).normalize_or_zero(),
            radius: v.radius,
            strength: v.strength,
            kind: kind_of(v.kind),
        });
    }
    world_wind
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Vegetation leans by the one rate per m/s, along the wind; still air
    /// gives no wind at all (the field then runs for motors alone).
    #[test]
    fn vegetation_leans_by_the_speed() {
        let w = WorldWind {
            velocity: Vec3::new(0.0, 0.0, -8.0),
            gust: 0.4,
            turbulence: 0.1,
            gust_size: 20.0,
        };
        let v = w.vegetation().expect("a wind");
        assert_eq!(v.direction, Vec3::NEG_Z);
        assert_eq!(v.speed, 8.0);
        assert!((v.strength - 8.0 * og_render::Wind::LEAN_PER_MPS).abs() < 1e-6);
        assert_eq!((v.gust, v.turbulence, v.gust_size), (0.4, 0.1, 20.0));
        assert_eq!(w.aloft(), Vec3::new(0.0, 0.0, -16.0));
        assert!(WorldWind::default().vegetation().is_none());
    }
}
