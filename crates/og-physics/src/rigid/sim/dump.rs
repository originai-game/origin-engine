//! Diagnostic snapshot of the active rapier simulation (`dump_physics`).

use super::*;
use crate::{
    PhysicsBodyCounts, PhysicsBodyDump, PhysicsColliderCounts, PhysicsContactDump, PhysicsDump,
    PhysicsJointDump, PhysicsRagdollDump, PhysicsVehicleDump,
};

fn body_kind_name(t: rp::RigidBodyType) -> &'static str {
    match t {
        rp::RigidBodyType::Dynamic => "dynamic",
        rp::RigidBodyType::Fixed => "fixed",
        rp::RigidBodyType::KinematicPositionBased => "kinematicPosition",
        rp::RigidBodyType::KinematicVelocityBased => "kinematicVelocity",
        rp::RigidBodyType::SoftFrame => "softFrame",
    }
}

fn joint_kind_name(kind: u32) -> &'static str {
    match kind {
        0 => "fixed",
        1 => "hinge",
        2 => "slider",
        3 => "spring",
        4 => "rope",
        5 => "ball",
        _ => "unknown",
    }
}

impl Sim {
    /// Captures a read-only diagnostic snapshot of all bodies, colliders,
    /// joints, active contacts/sensors, and secondary solvers.
    pub(crate) fn dump(&self) -> PhysicsDump {
        let mut counts = PhysicsBodyCounts {
            tracked: self.bodies.len(),
            rapier: self.world.bodies.len(),
            ..Default::default()
        };
        let mut bodies = Vec::with_capacity(self.bodies.len());
        let mut body_entries: Vec<(u64, &Tracked)> =
            self.bodies.iter().map(|(&e, t)| (e, t)).collect();
        body_entries.sort_by_key(|(e, _)| *e);
        for (entity, tracked) in body_entries {
            let Some(rb) = self.world.bodies.get(tracked.handle) else {
                continue;
            };
            let ty = rb.body_type();
            match ty {
                rp::RigidBodyType::Dynamic => counts.dynamic += 1,
                rp::RigidBodyType::Fixed => counts.fixed += 1,
                rp::RigidBodyType::KinematicPositionBased
                | rp::RigidBodyType::KinematicVelocityBased
                | rp::RigidBodyType::SoftFrame => counts.kinematic += 1,
            }
            let sleeping = rb.is_sleeping();
            if sleeping {
                counts.sleeping += 1;
            } else {
                counts.awake += 1;
            }
            let (position, _) = unpose(rb.position(), tracked.position);
            bodies.push(PhysicsBodyDump {
                entity,
                kind: body_kind_name(ty),
                sleeping,
                position,
                linear_velocity: unv(rb.linvel()),
                angular_velocity: unangvel(rb.angvel()),
                colliders: rb.colliders().len(),
            });
        }

        let sensors = self
            .world
            .colliders
            .iter()
            .filter(|(_, c)| c.is_sensor())
            .count();
        let collider_counts = PhysicsColliderCounts {
            total: self.world.colliders.len(),
            statics: self.statics.len(),
            sensors,
            compounds: self.ext.compounds.len(),
        };

        let mut joints: Vec<PhysicsJointDump> = self
            .joints
            .iter()
            .map(|(&entity, tj)| PhysicsJointDump {
                entity,
                kind: joint_kind_name(tj.kind()),
                other: tj.other_entity(),
            })
            .collect();
        joints.sort_by_key(|j| j.entity);

        let mut contacts = Vec::new();
        for edge in self
            .world
            .narrow_phase
            .contact_graph()
            .raw_graph()
            .raw_edges()
        {
            let pair = &edge.weight;
            if !pair.has_any_active_contact() {
                continue;
            }
            let c1 = &self.world.colliders[pair.collider1];
            let c2 = &self.world.colliders[pair.collider2];
            let (e1, e2) = (entity_of(c1).to_bits(), entity_of(c2).to_bits());
            let (mut point, mut normal) = (Vec3::ZERO, Vec3::ZERO);
            if let Some((manifold, contact)) = pair.find_deepest_contact() {
                point = unv(c1.position().transform_point(contact.local_p1));
                normal = unv(manifold.data.normal);
            }
            let (a, b, normal) = if e1 <= e2 {
                (e1, e2, normal)
            } else {
                (e2, e1, -normal)
            };
            let manifolds = pair.manifolds().len();
            let points: usize = pair.manifolds().iter().map(|m| m.points.len()).sum();
            let deepest_dist = pair
                .manifolds()
                .iter()
                .flat_map(|m| m.points.iter().map(|p| p.dist))
                .fold(f32::INFINITY, f32::min);
            contacts.push(PhysicsContactDump {
                a,
                b,
                sensor: false,
                manifolds,
                points,
                deepest_dist: if deepest_dist.is_finite() {
                    deepest_dist
                } else {
                    0.0
                },
                point,
                normal,
            });
        }
        for (h1, h2, intersecting) in self.world.narrow_phase.intersection_pairs() {
            if !intersecting {
                continue;
            }
            let (Some(c1), Some(c2)) = (self.world.colliders.get(h1), self.world.colliders.get(h2))
            else {
                continue;
            };
            let (e1, e2) = (entity_of(c1).to_bits(), entity_of(c2).to_bits());
            let (a, b) = if e1 <= e2 { (e1, e2) } else { (e2, e1) };
            contacts.push(PhysicsContactDump {
                a,
                b,
                sensor: true,
                manifolds: 0,
                points: 0,
                deepest_dist: 0.0,
                point: Vec3::ZERO,
                normal: Vec3::ZERO,
            });
        }
        contacts.sort_by_key(|c| (c.sensor, c.a, c.b));

        let mut vehicles: Vec<PhysicsVehicleDump> = self
            .vehicles
            .keys()
            .map(|&entity| {
                let ws = super::super::wheels(self, entity).unwrap_or_default();
                let grounded_wheels = ws.iter().filter(|w| w.contact).count();
                PhysicsVehicleDump {
                    entity,
                    wheels: ws.len(),
                    grounded_wheels,
                }
            })
            .collect();
        vehicles.sort_by_key(|v| v.entity);

        let mut ragdolls: Vec<PhysicsRagdollDump> = self
            .ragdolls
            .iter()
            .map(|(&entity, r)| PhysicsRagdollDump {
                entity,
                bones: r.bones.len(),
                dynamic: r.dynamic,
            })
            .collect();
        ragdolls.sort_by_key(|r| r.entity);

        let sorted_counts = |iter: Box<dyn Iterator<Item = (u64, usize)> + '_>| {
            let mut v: Vec<(u64, usize)> = iter.collect();
            v.sort_by_key(|(e, _)| *e);
            v
        };
        let ropes = sorted_counts(Box::new(
            self.ropes.iter().map(|(&e, r)| (e, r.points.len())),
        ));
        let cloths = sorted_counts(Box::new(
            self.cloths.iter().map(|(&e, c)| (e, c.points.len())),
        ));
        let soft_bodies = sorted_counts(Box::new(
            self.ext.softs.iter().map(|(&e, s)| (e, s.points.len())),
        ));
        let garments = sorted_counts(Box::new(
            self.ext.garments.iter().map(|(&e, g)| (e, g.points.len())),
        ));
        let fluids2d = sorted_counts(Box::new(
            self.fluids.iter().map(|(&e, f)| (e, f.positions.len())),
        ));
        let fluids3d = sorted_counts(Box::new(
            self.fluids3.iter().map(|(&e, f)| (e, f.positions.len())),
        ));

        PhysicsDump {
            profile: PROFILE_NAME,
            gravity: self.gravity(),
            stats: Default::default(),
            body_counts: counts,
            collider_counts,
            bodies,
            joints,
            contacts,
            vehicles,
            ragdolls,
            ropes,
            cloths,
            soft_bodies,
            garments,
            fluids2d,
            fluids3d,
            water_surfaces: self.surfaces.len(),
            water_lines: self.lines.len(),
        }
    }
}
