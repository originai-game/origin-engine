//! Shared engine, gearbox, differential and wheel torque solve, independent of hosts.
use crate::schema::*;
use serde::{Deserialize, Serialize};

const GEARS: [f32; 6] = [3.4, 2.2, 1.6, 1.25, 1.0, 0.82];
const CURVE: [f32; 8] = [140.0, 170.0, 195.0, 210.0, 215.0, 205.0, 185.0, 150.0];
const RPM: f32 = 60.0 / std::f32::consts::TAU;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Gearbox {
    pub gear: i32,
    pub rpm: f32,
    pub shift: f32,
    /// Requested direction while braking to a standstill, zero otherwise.
    pub direction: i32,
}

/// Wheel speed is positive forwards in both profiles.
pub(crate) struct Wheel {
    pub omega: f32,
    pub inertia: f32,
    pub radius: f32,
    pub front: bool,
    pub drive: bool,
    pub brake: bool,
    pub handbrake: bool,
    pub slip: f32,
}

pub(crate) struct Spin {
    /// Angular velocity after drive/differential, before brakes and contact.
    pub omega: f32,
    /// Available brake angular impulse, solved with the tyre reaction.
    pub brake: f32,
}

pub(crate) fn differential(v: &Vehicle, wheels: &[Wheel], omega: &mut [f32], dt: f32) {
    if v.differential == 0 {
        return;
    }
    let inertia: f32 = wheels.iter().filter(|w| w.drive).map(|w| w.inertia).sum();
    let mean = wheels
        .iter()
        .zip(omega.iter())
        .filter(|(w, _)| w.drive)
        .map(|(w, speed)| speed * w.inertia)
        .sum::<f32>()
        / inertia.max(1.0e-6);
    let blend = if v.differential == 2 {
        1.0
    } else {
        1.0 - og_core::math::exp(-v.differential_strength.max(0.0) * dt)
    };
    for (w, speed) in wheels.iter().zip(omega) {
        if w.drive {
            *speed += (mean - *speed) * blend;
        }
    }
}

fn gears(v: &Vehicle) -> &[f32] {
    if v.gears.is_empty() {
        &GEARS
    } else {
        v.gears.as_slice()
    }
}

fn ratio(v: &Vehicle, gear: i32) -> f32 {
    if gear < 0 {
        -v.reverse_ratio.max(0.0)
    } else if gear > 0 {
        gears(v)
            .get(gear as usize - 1)
            .copied()
            .unwrap_or(0.0)
            .max(0.0)
    } else {
        0.0
    }
}

fn torque_at(v: &Vehicle, rpm: f32) -> f32 {
    let c = if v.torque_curve.is_empty() {
        &CURVE[..]
    } else {
        v.torque_curve.as_slice()
    };
    let at = (rpm / v.max_rpm.max(1.0)).clamp(0.0, 1.0) * (c.len() - 1) as f32;
    let i = (at as usize).min(c.len() - 1);
    let torque = c[i] + (c[(i + 1).min(c.len() - 1)] - c[i]) * (at - i as f32);
    // Soft fuel cut; engine drag and clutch remain active above redline.
    torque.max(0.0) * ((v.max_rpm - rpm) / (v.max_rpm * 0.05).max(1.0)).clamp(0.0, 1.0)
}

/// Integrate drive, differential, brakes and resistance together. The tyre
/// solver then exchanges angular momentum with the contact, not a target speed.
pub(crate) fn step(
    v: &Vehicle,
    g: &mut Gearbox,
    speed: f32,
    wheels: &[Wheel],
    dt: f32,
) -> Vec<Spin> {
    let throttle = v.throttle.clamp(-1.0, 1.0);
    let count = wheels.iter().filter(|w| w.drive).count().max(1) as f32;
    let front_count = wheels.iter().filter(|w| w.drive && w.front).count() as f32;
    let rear_count = count - front_count;
    let weights: Vec<f32> = wheels
        .iter()
        .map(|w| {
            if !w.drive {
                0.0
            } else if v.awd_bias > 0.0 && front_count > 0.0 && rear_count > 0.0 {
                if w.front {
                    v.awd_bias.clamp(0.0, 1.0) / front_count
                } else {
                    (1.0 - v.awd_bias.clamp(0.0, 1.0)) / rear_count
                }
            } else {
                1.0 / count
            }
        })
        .collect();
    let omega = wheels
        .iter()
        .zip(&weights)
        .map(|(w, s)| w.omega * s)
        .sum::<f32>();
    let requested = if throttle == 0.0 {
        0
    } else {
        throttle.signum() as i32
    };
    // One direction transition for arcade and drivetrain, with no speed dead zone.
    g.direction = if requested != 0 && speed * (requested as f32) < -0.1 {
        requested
    } else {
        0
    };
    let against = g.direction != 0;
    let mut total = 0.0;
    if v.model == VEHICLE_DRIVETRAIN {
        g.shift = (g.shift - dt).max(0.0);
        let top = gears(v).len() as i32;
        let mut want = if v.automatic {
            g.gear
        } else {
            v.gear.clamp(-1, top)
        };
        let engine_rpm = |gear| (omega * ratio(v, gear) * v.final_drive * RPM).max(0.0);
        if v.automatic {
            if !against && requested != 0 && g.gear.signum() != requested {
                want = requested;
            } else if g.gear > 0 && speed >= 0.0 && g.shift == 0.0 {
                let rpm = engine_rpm(want);
                if rpm >= v.shift_up_rpm && want < top && engine_rpm(want + 1) > v.shift_down_rpm {
                    want += 1;
                } else {
                    while want > 1
                        && engine_rpm(want) < v.shift_down_rpm
                        && engine_rpm(want - 1) < v.shift_up_rpm.min(v.max_rpm)
                    {
                        want -= 1;
                    }
                }
            }
        }
        if want != g.gear {
            // First engagement is immediate; subsequent shifts ramp the clutch.
            g.shift = if g.gear == 0 {
                0.0
            } else {
                v.clutch_time.max(0.0)
            };
            g.gear = want;
        }
        if v.engine_inertia > 0.0 {
            let idle = v.idle_rpm.max(0.0) / RPM;
            let mut engine = (g.rpm / RPM).max(idle);
            let pedal =
                if against || (requested != 0 && requested != g.gear.signum() && g.gear != 0) {
                    0.0
                } else {
                    throttle.abs()
                };
            let drag = v.engine_braking.max(0.0) * (1.0 - pedal) * dt / v.engine_inertia;
            engine =
                (engine + torque_at(v, engine * RPM) * pedal * dt / v.engine_inertia + drag * idle)
                    / (1.0 + drag);
            let through = ratio(v, g.gear) * v.final_drive.max(0.0);
            let engagement = if v.clutch_time > 0.0 {
                1.0 - g.shift / v.clutch_time
            } else {
                1.0
            };
            let inv_w = wheels
                .iter()
                .zip(&weights)
                .map(|(w, s)| s * s / w.inertia.max(1.0e-6))
                .sum::<f32>();
            let impulse =
                (engine - omega * through) / (1.0 / v.engine_inertia + through * through * inv_w);
            let limit = v.clutch_strength.max(0.0) * engagement.clamp(0.0, 1.0) * dt;
            let impulse = if through == 0.0 {
                0.0
            } else {
                impulse.clamp(-limit, limit)
            };
            engine -= impulse / v.engine_inertia;
            g.rpm = engine.max(idle) * RPM;
            total = impulse * through / dt;
        } else {
            g.rpm = 0.0;
        }
    } else if !against {
        total = throttle
            * v.engine_force.max(0.0)
            * count
            * if v.max_speed > 0.0 {
                (1.0 - speed.abs() / v.max_speed).clamp(0.0, 1.0)
            } else {
                0.0
            };
    }
    let brake = (v.brake + if against { throttle.abs() } else { 0.0 }).clamp(0.0, 1.0);
    let n_brake = wheels.iter().filter(|w| w.brake).count() as f32;
    let n_front = wheels.iter().filter(|w| w.brake && w.front).count() as f32;
    let n_rear = n_brake - n_front;
    let mut speeds: Vec<_> = wheels
        .iter()
        .zip(weights)
        .map(|(w, weight)| {
            w.omega
                + total * weight * (1.0 - v.tcs.clamp(0.0, 1.0) * w.slip) * dt
                    / w.inertia.max(1.0e-6)
        })
        .collect();
    differential(v, wheels, &mut speeds, dt);
    wheels
        .iter()
        .zip(speeds)
        .map(|(w, omega)| {
            let bias = if v.brake_bias > 0.0 && n_front > 0.0 && n_rear > 0.0 {
                n_brake
                    * if w.front {
                        v.brake_bias.clamp(0.0, 1.0) / n_front
                    } else {
                        (1.0 - v.brake_bias.clamp(0.0, 1.0)) / n_rear
                    }
            } else {
                1.0
            };
            let braking = if w.brake {
                brake * bias * (1.0 - v.abs.clamp(0.0, 1.0) * w.slip)
            } else {
                0.0
            } + if w.handbrake {
                v.handbrake.clamp(0.0, 1.0)
            } else {
                0.0
            };
            let resistance =
                (braking * v.brake_force.max(0.0) + v.rolling_resistance.max(0.0) * w.radius) * dt;
            Spin {
                omega,
                brake: resistance,
            }
        })
        .collect()
}
