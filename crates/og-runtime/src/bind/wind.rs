//! og.Wind, the world's wind: read from the world in one place, and what
//! each thing that moves in the air takes from it, so vegetation, clouds,
//! weather and the soft bodies cannot disagree about where the air goes.
//!
//! The tick reads it, and the og.WindMotors pushing on it, once as it
//! starts ([`before_tick`]): the air ropes, cloths, garments and bodies
//! with an airDrag move through in physics, and the particle systems the
//! tick simulates are carried by. Extraction reads them for what is drawn
//! ([`extract`] for vegetation, [`frame_air`] for the renderer's particles).
//! Each of those adds its record's own `wind` to it.

use crate::host::codec::Layout;
use crate::host::wire::{FromJson, WindMotorKind, record};
use crate::present::fx::{kept_rows, resource};
use glam::Vec3;
use og_ecs::World;
use og_env::air::{Air, Motor};
use std::sync::{Arc, Mutex};

/// The air of one engine's world this tick, as [`before_tick`] read it:
/// shared with the systems the tick runs.
#[derive(Clone, Default)]
pub(crate) struct ModuleState(Arc<Mutex<Air>>);

impl ModuleState {
    /// Where the air moves at `p` this tick (m/s).
    pub(crate) fn at(&self, p: Vec3) -> Vec3 {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .at(p)
    }
}

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
fn kind_of(kind: WindMotorKind) -> og_env::air::MotorKind {
    match kind {
        WindMotorKind::Directional => og_env::air::MotorKind::Directional,
        WindMotorKind::Vortex => og_env::air::MotorKind::Vortex,
        WindMotorKind::Omni => og_env::air::MotorKind::Omni,
    }
}

/// An og.WindMotor at its entity's world place, pushing along `forward`.
fn motor(v: &record::WindMotor, position: Vec3, forward: Vec3) -> Motor {
    Motor {
        position,
        direction: forward,
        radius: v.radius,
        strength: v.strength,
        kind: kind_of(v.kind),
    }
}

/// As each tick starts: the world's air (og.Wind and every og.WindMotor at
/// its entity's world pose, in entity order), kept for the tick's systems
/// and given to the physics that moves ropes, cloths, garments and bodies
/// through it.
pub(crate) fn before_tick(eng: &crate::engine::Engine, world: &World) {
    let (wind, motors, physics) = {
        let st = eng.state.borrow();
        (
            st.layouts.get("og.Wind").cloned(),
            st.layouts.get("og.WindMotor").cloned(),
            st.physics.clone(),
        )
    };
    let r = crate::bind::physics_ext::Reader::World(world);
    let motors = r
        .rows(motors.as_ref())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(e, v)| {
            let v = record::WindMotor::from_json(&v).ok()?;
            let (position, turn, _) = r.pose(og_ecs::Entity::from_bits(e))?;
            Some(motor(&v, position, turn * Vec3::NEG_Z))
        })
        .collect();
    let air = Air {
        wind: WorldWind::read(world, wind.as_ref()).velocity,
        motors,
        ..Default::default()
    };
    let mut kept = eng
        .modules
        .air
        .0
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(p) = physics {
        p.set_air(air.clone());
    }
    *kept = air;
}

/// The air a frame draws: og.Wind and every og.WindMotor where the frame's
/// hierarchy places its entity, in entity order.
pub(crate) fn frame_air(
    world: &World,
    wind: Option<&Layout>,
    motors: Option<&Layout>,
    h: &mut crate::present::extract::Hierarchy,
) -> (WorldWind, Air) {
    let world_wind = WorldWind::read(world, wind);
    let motors = kept_rows(world, motors)
        .into_iter()
        .filter_map(|(e, v)| {
            let v = record::WindMotor::from_json(&v).ok()?;
            let m = h.world_of(e);
            Some(motor(
                &v,
                m.transform_point3(Vec3::ZERO),
                m.transform_vector3(Vec3::NEG_Z).normalize_or_zero(),
            ))
        })
        .collect();
    let air = Air {
        wind: world_wind.velocity,
        motors,
        ..Default::default()
    };
    (world_wind, air)
}

/// og.Wind and the og.WindMotors into `scene`: what vegetation sways in.
/// Returns the world's wind for what else moves in it this frame.
pub(crate) fn extract(
    world: &World,
    scene: &mut og_render::RenderScene,
    wind: Option<&Layout>,
    motors: Option<&Layout>,
    h: &mut crate::present::extract::Hierarchy,
) -> WorldWind {
    // og.Wind: what vegetation sways in here; the clouds, the weather and
    // the visual cloths below add their own to it. og.WindMotor: local
    // pushes on the simulated wind field, placed by their entities
    // (wind_field.rs keeps the nearest).
    let (world_wind, air) = frame_air(world, wind, motors, h);
    scene.atmosphere.wind = world_wind.vegetation();
    scene.atmosphere.wind_motors = air.motors;
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
