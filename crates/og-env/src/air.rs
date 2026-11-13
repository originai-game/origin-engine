//! The air of a world: its wind (og.Wind) and the local pushes on it
//! (og.WindMotor), as one function of a point. Everything the tick moves
//! through the air asks [`Air::at`], so ropes, cloths, bodies and fluids
//! cannot disagree about where it blows; the renderer's simulated wind
//! field drives vegetation with the same motors and the same falloff
//! (wind_field.wgsl's `motor_velocity`).
//!
//! Additions, a square root and a division: the same on every host.

use og_core::glam::Vec3;

/// How a motor pushes the air.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MotorKind {
    /// Outward from its centre.
    #[default]
    Omni,
    /// Along its direction.
    Directional,
    /// Around its centre, counter-clockwise seen down the air's axis.
    Vortex,
}

/// A local push on the air (og.WindMotor), in world space.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Motor {
    pub position: Vec3,
    /// A directional motor's direction (unit).
    pub direction: Vec3,
    /// How far it reaches (m); its push fades to nothing at its edge.
    pub radius: f32,
    /// Its push at its centre (m/s).
    pub strength: f32,
    pub kind: MotorKind,
}

impl Motor {
    /// Its push at `p` (m/s), a vortex turning about `axis`: within its
    /// radius, fading to its edge as `(1 - d / r)²`.
    pub fn at(&self, p: Vec3, axis: Vec3) -> Vec3 {
        let r = self.radius.max(1e-3);
        let d = p - self.position;
        let dist = d.length();
        if dist >= r || !self.strength.is_finite() {
            return Vec3::ZERO;
        }
        let fall = (1.0 - dist / r) * (1.0 - dist / r);
        let out = if dist > 1e-4 { d / dist } else { Vec3::ZERO };
        let dir = match self.kind {
            MotorKind::Omni => out,
            MotorKind::Directional => self.direction,
            MotorKind::Vortex => axis.cross(out),
        };
        dir * (self.strength * fall)
    }
}

/// The air of a world this tick. Still by default.
#[derive(Clone, Debug, PartialEq)]
pub struct Air {
    /// The world's wind (m/s), the same everywhere.
    pub wind: Vec3,
    /// The local pushes, in entity order.
    pub motors: Vec<Motor>,
    /// What a vortex turns about: the world's up (y in 3D, z in 2D).
    pub axis: Vec3,
}

impl Default for Air {
    fn default() -> Self {
        Air {
            wind: Vec3::ZERO,
            motors: Vec::new(),
            axis: Vec3::Y,
        }
    }
}

impl Air {
    /// Whether nothing moves anywhere.
    pub fn is_still(&self) -> bool {
        self.wind == Vec3::ZERO && self.motors.is_empty()
    }

    /// Where the air moves at `p` and how fast (m/s): the wind plus every
    /// motor's push there, in their order.
    pub fn at(&self, p: Vec3) -> Vec3 {
        self.motors
            .iter()
            .fold(self.wind, |u, m| u + m.at(p, self.axis))
    }

    /// What a thing moving at `velocity` through the air at `p` gains in
    /// `dt` seconds with `drag` per second: it is carried towards the
    /// air's velocity, never past it (the implicit step, stable for any
    /// drag).
    pub fn carry(&self, p: Vec3, velocity: Vec3, drag: f32, dt: f32) -> Vec3 {
        let k = drag.max(0.0) * dt;
        (self.at(p) - velocity) * (k / (1.0 + k))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn motor(kind: MotorKind) -> Motor {
        Motor {
            position: Vec3::new(10.0, 0.0, 0.0),
            direction: Vec3::NEG_Z,
            radius: 4.0,
            strength: 8.0,
            kind,
        }
    }

    /// A motor pushes within its radius only, its full strength at its
    /// centre's side fading by the square to its edge; each kind its way.
    #[test]
    fn a_motor_pushes_its_way_and_fades_to_its_edge() {
        let p = Vec3::new(12.0, 0.0, 0.0);
        assert_eq!(
            motor(MotorKind::Omni).at(p, Vec3::Y),
            Vec3::new(2.0, 0.0, 0.0)
        );
        assert_eq!(
            motor(MotorKind::Directional).at(p, Vec3::Y),
            Vec3::new(0.0, 0.0, -2.0)
        );
        // Counter-clockwise seen from above: at +x the air goes to -z.
        assert_eq!(
            motor(MotorKind::Vortex).at(p, Vec3::Y),
            Vec3::new(0.0, 0.0, -2.0)
        );
        assert_eq!(
            motor(MotorKind::Omni).at(Vec3::new(14.0, 0.0, 0.0), Vec3::Y),
            Vec3::ZERO
        );
        // An omni blast lifts what is above it.
        let up = motor(MotorKind::Omni).at(Vec3::new(10.0, 1.0, 0.0), Vec3::Y);
        assert!(up.y > 4.0 && up.x == 0.0);
    }

    /// The air is the wind plus the motors; drag carries a thing towards
    /// the air's velocity and never past it.
    #[test]
    fn the_air_adds_up_and_carries_without_overshoot() {
        let air = Air {
            wind: Vec3::new(0.0, 0.0, 3.0),
            motors: vec![motor(MotorKind::Omni)],
            ..Air::default()
        };
        assert!(!air.is_still() && Air::default().is_still());
        assert_eq!(air.at(Vec3::ZERO), Vec3::new(0.0, 0.0, 3.0));
        assert_eq!(air.at(Vec3::new(12.0, 0.0, 0.0)), Vec3::new(2.0, 0.0, 3.0));
        assert_eq!(air.carry(Vec3::ZERO, Vec3::ZERO, 0.0, 1.0), Vec3::ZERO);
        let dv = air.carry(Vec3::ZERO, Vec3::ZERO, 1000.0, 1.0);
        assert!(dv.z > 2.99 && dv.z <= 3.0, "{dv}");
        // Already moving with the air: nothing to gain.
        assert_eq!(air.carry(Vec3::ZERO, air.wind, 5.0, 0.1), Vec3::ZERO);
    }
}
