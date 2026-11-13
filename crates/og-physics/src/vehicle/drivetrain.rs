//! The drivetrain of an `og.Vehicle` with `model` VEHICLE_DRIVETRAIN, for
//! both profiles: an engine with a torque curve driving the wheels through
//! a gearbox (automatic or the game's) and a final drive. It knows nothing
//! of rapier: given the car's forward speed it turns the throttle into a
//! torque at the driven wheels, and keeps the gearbox's state (the gear,
//! the engine's speed and the clutch's time left), which is world state
//! (the og.physics block's vehicle section).

use crate::schema::Vehicle;
use serde::{Deserialize, Serialize};

/// The forward gears an empty `Vehicle.gears` stands for.
const GEARS: [f32; 6] = [3.4, 2.2, 1.6, 1.25, 1.0, 0.82];
/// The torque curve (N·m) an empty `Vehicle.torqueCurve` stands for.
const CURVE: [f32; 8] = [140.0, 170.0, 195.0, 210.0, 215.0, 205.0, 185.0, 150.0];
/// Revolutions per minute of one radian per second.
const RPM: f32 = 60.0 / std::f32::consts::TAU;

/// The gearbox and engine between steps.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Gearbox {
    /// -1 reverse, 0 neutral, 1 and up forward.
    pub gear: i32,
    /// The engine's speed (rpm).
    pub rpm: f32,
    /// What is left of a gear change (s); the engine drives nothing until 0.
    pub shift: f32,
}

/// What the drivetrain does to the wheels this step.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Drive {
    /// The torque at the driven wheels together (N·m): positive pushes
    /// the car forward, negative backward.
    pub torque: f32,
    /// The gear's top wheel speed (rad/s, positive): the rev limit through
    /// this gear, for a motor that drives towards a speed (2D).
    pub wheel_speed: f32,
    /// A share of the brake (0 to 1): throttle against the way the car
    /// rolls brakes it, as the arcade car does.
    pub brake: f32,
}

fn gears(v: &Vehicle) -> &[f32] {
    if v.gears.is_empty() {
        &GEARS
    } else {
        v.gears.as_slice()
    }
}

/// The curve's torque at `rpm`, linear between its points, which are
/// evenly spaced from 0 to maxRpm; 0 at or past maxRpm.
fn torque_at(v: &Vehicle, rpm: f32) -> f32 {
    let curve: &[f32] = if v.torque_curve.is_empty() {
        &CURVE
    } else {
        v.torque_curve.as_slice()
    };
    let max = v.max_rpm.max(1.0);
    if rpm.is_nan() || rpm >= max {
        return 0.0;
    }
    if curve.len() == 1 {
        return curve[0].max(0.0);
    }
    let at = rpm.max(0.0) / max * (curve.len() - 1) as f32;
    let i = (at as usize).min(curve.len() - 2);
    let t = at - i as f32;
    (curve[i] + (curve[i + 1] - curve[i]) * t).max(0.0)
}

/// The gear's ratio: forward gears from `gears`, reverse `reverseRatio`,
/// neutral 0.
fn ratio(v: &Vehicle, gear: i32) -> f32 {
    match gear {
        g if g > 0 => gears(v)
            .get(g as usize - 1)
            .copied()
            .unwrap_or(0.0)
            .max(0.0),
        g if g < 0 => v.reverse_ratio.max(0.0),
        _ => 0.0,
    }
}

/// One step of the drivetrain: the gearbox shifts (by itself or to the
/// game's `gear`), the engine's speed follows the wheels while a gear is
/// in (or free-revs towards the throttle when not), and the curve's torque
/// at that speed goes through the gear and the final drive. `speed` is the
/// car's forward speed (m/s), `radius` its driven wheels' radius.
pub(crate) fn step(v: &Vehicle, g: &mut Gearbox, speed: f32, radius: f32, dt: f32) -> Drive {
    let top = gears(v).len() as i32;
    let throttle = if v.throttle.is_finite() {
        v.throttle.clamp(-1.0, 1.0)
    } else {
        0.0
    };
    let radius = radius.max(0.01);
    let final_drive = v.final_drive.max(0.0);
    let engine = |gear: i32| speed.abs() / radius * ratio(v, gear) * final_drive * RPM;
    g.shift = (g.shift - dt).max(0.0);
    let mut want = if v.automatic {
        g.gear.clamp(-1, top)
    } else {
        v.gear.clamp(-1, top)
    };
    if v.automatic {
        if throttle < 0.0 && speed < 0.5 && want >= 0 {
            want = -1;
        } else if throttle > 0.0 && want <= 0 && speed > -0.5 {
            want = 1;
        } else if want >= 1 && g.shift == 0.0 {
            let rpm = engine(want);
            if rpm >= v.shift_up_rpm && want < top {
                want += 1;
            } else if rpm <= v.shift_down_rpm && want > 1 {
                want -= 1;
            }
        }
    }
    if want != g.gear {
        g.gear = want;
        g.shift = v.clutch_time.max(0.0);
    }
    let idle = v.idle_rpm.max(0.0);
    let max = v.max_rpm.max(1.0);
    // The pedal in the gear's direction (either way in neutral).
    let pedal = match g.gear {
        0 => throttle.abs(),
        gear if gear > 0 => throttle.max(0.0),
        _ => (-throttle).max(0.0),
    };
    let engaged = g.gear != 0 && g.shift == 0.0 && ratio(v, g.gear) > 0.0;
    if engaged {
        g.rpm = engine(g.gear).max(idle);
    } else {
        // Free-revving: half a second from idle to the limit.
        let target = idle + pedal * (max - idle).max(0.0);
        let rate = (max - idle).max(0.0) * 2.0 * dt;
        g.rpm += (target - g.rpm).clamp(-rate, rate);
    }
    let sign = g.gear.signum() as f32;
    let through = ratio(v, g.gear) * final_drive;
    let torque = if engaged {
        torque_at(v, g.rpm) * pedal * through * sign
    } else {
        0.0
    };
    let wheel_speed = if through > 0.0 {
        max / RPM / through
    } else {
        0.0
    };
    // Throttle against the motion: forward gears rolling forward and
    // pressed back, or reverse rolling back and pressed forward.
    let against = (g.gear >= 0 && throttle < 0.0 && speed > 1.0)
        || (g.gear < 0 && throttle > 0.0 && speed < -1.0);
    Drive {
        torque,
        wheel_speed,
        brake: if against { throttle.abs() } else { 0.0 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::VEHICLE_DRIVETRAIN;

    fn car() -> Vehicle {
        Vehicle {
            model: VEHICLE_DRIVETRAIN,
            ..Vehicle::default()
        }
    }

    #[test]
    fn the_curve_interpolates_and_stops_at_the_limit() {
        let v = car();
        // Points at 0, 971.4, 1942.9 ... 6800 rpm.
        assert_eq!(torque_at(&v, 0.0), 140.0);
        let step = 6800.0 / 7.0;
        assert!((torque_at(&v, step * 0.5) - 155.0).abs() < 1e-3);
        assert!((torque_at(&v, step * 4.0) - 215.0).abs() < 1e-3);
        assert_eq!(torque_at(&v, 6800.0), 0.0);
        assert_eq!(torque_at(&v, 7000.0), 0.0);
    }

    #[test]
    fn an_automatic_box_goes_into_first_shifts_up_and_down_and_reverses() {
        let v = car();
        let mut g = Gearbox::default();
        let dt = 1.0 / 60.0;
        let d = step(&Vehicle { throttle: 1.0, ..v }, &mut g, 0.0, 0.4, dt);
        assert_eq!(g.gear, 1);
        assert_eq!(d.torque, 0.0, "nothing drives while the clutch is out");
        for _ in 0..15 {
            step(&Vehicle { throttle: 1.0, ..v }, &mut g, 0.0, 0.4, dt);
        }
        let d = step(&Vehicle { throttle: 1.0, ..v }, &mut g, 0.0, 0.4, dt);
        // At a standstill the engine idles: 900 rpm on the curve, through
        // first (3.4) and the final drive (3.6).
        let at_idle = 140.0 + (170.0 - 140.0) * (900.0 / (6800.0 / 7.0));
        assert!(
            (d.torque - at_idle * 3.4 * 3.6).abs() < 0.01,
            "{}",
            d.torque
        );
        // First at 6,000 rpm: 6000 / (3.4 * 3.6) wheel rpm.
        let fast = 6000.0 / RPM / (3.4 * 3.6) * 0.4 + 0.01;
        step(&Vehicle { throttle: 1.0, ..v }, &mut g, fast, 0.4, dt);
        assert_eq!(g.gear, 2);
        for _ in 0..20 {
            step(&Vehicle { throttle: 0.0, ..v }, &mut g, 3.0, 0.4, dt);
        }
        assert_eq!(g.gear, 1, "slow in second shifts down");
        // Pressed back at a standstill: reverse, and it drives backwards.
        for _ in 0..20 {
            step(
                &Vehicle {
                    throttle: -1.0,
                    ..v
                },
                &mut g,
                0.0,
                0.4,
                dt,
            );
        }
        let d = step(
            &Vehicle {
                throttle: -1.0,
                ..v
            },
            &mut g,
            -0.2,
            0.4,
            dt,
        );
        assert_eq!(g.gear, -1);
        assert!(d.torque < 0.0);
        // Pressed forward while rolling back fast brakes instead.
        let d = step(&Vehicle { throttle: 1.0, ..v }, &mut g, -3.0, 0.4, dt);
        assert_eq!(g.gear, -1);
        assert_eq!(d.brake, 1.0);
    }

    #[test]
    fn a_manual_box_takes_the_games_gear_and_the_limiter_cuts_the_torque() {
        let v = Vehicle {
            automatic: false,
            throttle: 1.0,
            gear: 1,
            ..car()
        };
        let mut g = Gearbox::default();
        for _ in 0..20 {
            step(&v, &mut g, 1.0, 0.4, 1.0 / 60.0);
        }
        assert_eq!(g.gear, 1);
        // Past the limit in first: no torque, and the box stays in first.
        let d = step(&v, &mut g, 30.0, 0.4, 1.0 / 60.0);
        assert!(g.rpm > 6800.0);
        assert_eq!((g.gear, d.torque), (1, 0.0));
        // Neutral free-revs towards the throttle.
        let n = Vehicle { gear: 0, ..v };
        for _ in 0..120 {
            step(&n, &mut g, 0.0, 0.4, 1.0 / 60.0);
        }
        assert_eq!(g.gear, 0);
        assert!((g.rpm - 6800.0).abs() < 1.0, "{}", g.rpm);
    }
}
