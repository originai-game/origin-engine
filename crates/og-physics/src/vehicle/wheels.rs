//! Wheel layout shared by both hosts; zero mount captures the initial local pose.
use crate::{gather::WheelChild, schema::*};
use og_core::glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Spec {
    pub entity: u64,
    pub mount: Vec3,
    pub radius: f32,
    pub rest: f32,
    pub travel: f32,
    pub stiffness: f32,
    pub compression_damping: f32,
    pub rebound_damping: f32,
    pub grip: f32,
    pub steer: f32,
    pub drive: bool,
    pub brake: bool,
    pub handbrake: bool,
}

pub(crate) fn specs(v: &Vehicle, children: &[WheelChild], old: &[Spec], flat: bool) -> Vec<Spec> {
    if !children.is_empty() {
        return children
            .iter()
            .map(|c| {
                let w = c.wheel;
                let mount = if w.mount == Vec3::ZERO {
                    old.iter()
                        .find(|s| s.entity == c.entity.to_bits())
                        .map_or(c.mount, |s| s.mount)
                } else {
                    c.mount
                };
                Spec {
                    entity: c.entity.to_bits(),
                    mount,
                    radius: w.radius.max(0.01),
                    rest: w.suspension_rest.max(0.0),
                    travel: w.suspension_travel.max(0.0),
                    stiffness: w.suspension_stiffness.max(0.0),
                    compression_damping: w.compression_damping.max(0.0),
                    rebound_damping: w.rebound_damping.max(0.0),
                    grip: w.grip.max(0.0),
                    steer: w.steer,
                    drive: w.drive,
                    brake: w.brake,
                    handbrake: w.handbrake,
                }
            })
            .collect();
    }
    let n = if flat { 2 } else { 4 };
    (0..n)
        .map(|i| {
            let front = if flat { i == 0 } else { i < 2 };
            let along = if front {
                v.wheel_base * 0.5
            } else {
                -v.wheel_base * 0.5
            };
            Spec {
                entity: 0,
                mount: if flat {
                    Vec3::new(along, -v.mount_height, 0.0)
                } else {
                    Vec3::new(
                        if i % 2 == 0 {
                            v.track * 0.5
                        } else {
                            -v.track * 0.5
                        },
                        -v.mount_height,
                        along,
                    )
                },
                radius: v.wheel_radius.max(0.01),
                rest: v.suspension_rest.max(0.0),
                travel: v.suspension_travel.max(0.0),
                stiffness: v.suspension_stiffness.max(0.0),
                compression_damping: v.compression_damping.max(0.0),
                rebound_damping: v.rebound_damping.max(0.0),
                grip: v.grip.max(0.0),
                steer: if front { 1.0 } else { 0.0 },
                drive: match v.drive {
                    DRIVE_FRONT => front,
                    DRIVE_ALL => true,
                    _ => !front,
                },
                brake: true,
                handbrake: !front,
            }
        })
        .collect()
}

impl Spec {
    pub(crate) fn drive(
        self,
        v: &Vehicle,
        omega: f32,
        slip: f32,
        flat: bool,
    ) -> crate::drivetrain::Wheel {
        crate::drivetrain::Wheel {
            omega,
            inertia: 0.5 * v.wheel_mass.max(0.01) * self.radius * self.radius,
            radius: self.radius,
            front: if flat {
                self.mount.x > 0.0
            } else {
                self.mount.z > 0.0
            },
            drive: self.drive,
            brake: self.brake,
            handbrake: self.handbrake,
            slip,
        }
    }
}
