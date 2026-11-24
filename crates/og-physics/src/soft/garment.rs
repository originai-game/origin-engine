//! Skinned XPBD garments, in world space. A host supplies targets, normals
//! and tapered body capsules from its skeleton before the physics tick.
use crate::schema::Garment;
use crate::verlet::{self, Bend, Solids};
use og_core::glam::Vec3;
use og_core::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GarmentRig {
    pub rest: Vec<Vec3>,
    pub triangles: Vec<[u32; 3]>,
    pub freedom: Vec<f32>,
}

/// World-space, tapered skeleton capsule: radius interpolates from a to b.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GarmentCapsule {
    pub a: Vec3,
    pub b: Vec3,
    pub radius_a: f32,
    pub radius_b: f32,
}

impl GarmentCapsule {
    /// Nearest sphere in a tapered capsule's swept-sphere envelope.
    fn contact(&self, p: Vec3, padding: f32) -> Option<(Vec3, Vec3)> {
        let axis = self.b - self.a;
        let length = axis.length();
        let (ra, rb) = (self.radius_a.max(0.0), self.radius_b.max(0.0));
        if ra.max(rb) == 0.0 {
            return None;
        }
        let along = if length > 1e-9 {
            axis / length
        } else {
            Vec3::Y
        };
        let d = p - self.a;
        let u = d.dot(along);
        let radial = (d - along * u).length();
        let dr = rb - ra;
        let t = if length <= dr.abs() {
            if rb > ra { 1.0 } else { 0.0 }
        } else {
            ((u + dr * radial / (length * length - dr * dr).sqrt()) / length).clamp(0.0, 1.0)
        };
        let center = self.a + axis * t;
        let radius = ra + dr * t + padding;
        let delta = p - center;
        let len = delta.length();
        if len >= radius {
            return None;
        }
        let normal = if len > 1e-9 {
            delta / len
        } else {
            along
                .cross(if along.x.abs() < along.y.abs() {
                    Vec3::X
                } else {
                    Vec3::Y
                })
                .normalize_or_zero()
        };
        Some((center + normal * radius, normal))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GarmentPose {
    pub targets: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub scale: f32,
    pub capsules: Vec<GarmentCapsule>,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GarmentPoints {
    pub points: Vec<Vec3>,
    pub targets: Vec<Vec3>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct GarmentState {
    pub key: (u64, u32),
    pub points: Vec<Vec3>,
    pub velocities: Vec<Vec3>,
    pub inv_mass: Vec<f32>,
    pub targets: Vec<Vec3>,
    reach: Vec<f32>,
    edges: Vec<(u32, u32, f32)>,
    bends: Vec<Bend>,
}
impl GarmentRig {
    pub fn check(&self) -> Result<()> {
        let n = self.rest.len();
        if n == 0
            || self.freedom.len() != n
            || self.rest.iter().any(|p| !p.is_finite())
            || self.freedom.iter().any(|f| !f.is_finite())
            || self.triangles.iter().flatten().any(|&i| i as usize >= n)
        {
            return Err(Error::new(
                ErrorKind::Invalid,
                "garment rig: finite points, one freedom per point and valid triangle indices required",
            ));
        }
        Ok(())
    }
}
impl GarmentState {
    pub fn new(garment: &Garment, key: u64, rig: &GarmentRig, pose: &GarmentPose) -> Self {
        let mut edges = BTreeSet::new();
        for &[a, b, c] in &rig.triangles {
            for (a, b) in [(a, b), (b, c), (c, a)] {
                if a != b {
                    edges.insert((a.min(b), a.max(b)));
                }
            }
        }
        let top = rig.rest.iter().fold(f32::MIN, |m, p| m.max(p.y));
        let reach = rig
            .rest
            .iter()
            .zip(&rig.freedom)
            .map(|(p, &f)| {
                if garment.pin_top > 0.0 && p.y >= top - garment.pin_top {
                    0.0
                } else {
                    f.clamp(0.0, 1.0)
                }
            })
            .collect();
        Self {
            key: (key, garment.pin_top.to_bits()),
            points: pose.targets.clone(),
            velocities: vec![Vec3::ZERO; rig.rest.len()],
            inv_mass: verlet::masses(&rig.rest, &rig.triangles),
            targets: pose.targets.clone(),
            reach,
            edges: edges
                .into_iter()
                .map(|(a, b)| (a, b, (rig.rest[b as usize] - rig.rest[a as usize]).length()))
                .collect(),
            bends: verlet::bends(&rig.rest, &rig.triangles),
        }
    }
    pub fn hold(&mut self, pose: &GarmentPose) {
        self.points.clone_from(&pose.targets);
        self.velocities.fill(Vec3::ZERO);
        self.targets.clone_from(&pose.targets);
    }
    pub fn step(
        &mut self,
        g: &Garment,
        pose: &GarmentPose,
        dt: f32,
        tick: u64,
        gravity: Vec3,
        solids: &dyn Solids,
    ) {
        if dt <= 0.0 || g.substeps == 0 {
            return;
        }
        let n = self.points.len();
        let scale = pose.scale.abs();
        if g.teleport_distance > 0.0
            && self
                .targets
                .iter()
                .zip(&pose.targets)
                .any(|(a, b)| (*a - *b).length() > g.teleport_distance * scale)
        {
            self.hold(pose);
            return;
        }
        let steps = g.substeps.min(16);
        let h = dt / steps as f32;
        let from = std::mem::replace(&mut self.targets, pose.targets.clone());
        let w: Vec<f32> = self
            .inv_mass
            .iter()
            .enumerate()
            .map(|(i, &w)| {
                if self.reach[i] * g.max_distance <= 0.0 || g.density <= 0.0 || scale == 0.0 {
                    0.0
                } else {
                    w / (g.density * scale * scale)
                }
            })
            .collect();
        for s in 0..steps {
            let t = (s + 1) as f32 / steps as f32;
            let target: Vec<Vec3> = (0..n)
                .map(|i| from[i] + (pose.targets[i] - from[i]) * t)
                .collect();
            let before = self.points.clone();
            let mut contacts = vec![Vec::new(); n];
            for i in 0..n {
                if w[i] == 0.0 {
                    self.points[i] = target[i];
                    continue;
                }
                let p = self.points[i];
                let air = (g.wind + solids.air(p)) * (1.0 + g.gust * verlet::gust(p, tick));
                let acceleration = gravity * g.gravity_scale
                    + solids.acceleration(p)
                    + verlet::aero(
                        air - self.velocities[i],
                        pose.normals[i].normalize_or_zero(),
                        g.drag,
                        g.lift,
                    ) / g.density;
                self.velocities[i] = self.velocities[i]
                    * verlet::keep_per_substep(g.damping, dt, steps)
                    + acceleration * h;
                self.points[i] += self.velocities[i] * h;
            }
            let mut lambda = vec![0.0; self.edges.len()];
            let mut bending = vec![0.0; self.bends.len()];
            for _ in 0..g.iterations.min(64) {
                for (k, &(a, b, rest)) in self.edges.iter().enumerate() {
                    let (a, b) = (a as usize, b as usize);
                    verlet::link(
                        &mut self.points,
                        a,
                        b,
                        rest * scale,
                        (w[a], w[b]),
                        g.stiffness,
                        h,
                        &mut lambda[k],
                    );
                }
                for (k, b) in self.bends.iter().enumerate() {
                    verlet::bend(&mut self.points, &w, b, g.bending, h, &mut bending[k]);
                }
                for i in 0..n {
                    if w[i] == 0.0 {
                        continue;
                    }
                    let reach = self.reach[i] * g.max_distance.max(0.0) * scale;
                    let off = self.points[i] - target[i];
                    let len = off.length();
                    if len > reach {
                        self.points[i] = target[i] + off * (reach / len);
                        contacts[i].push(-off / len);
                    }
                    let r = g.backstop_radius.max(0.0) * scale;
                    if r > 0.0 {
                        let center = target[i]
                            - pose.normals[i].normalize_or_zero()
                                * (r + g.backstop.max(0.0) * scale);
                        let d = self.points[i] - center;
                        let len = d.length();
                        if len < r {
                            let normal = if len > 1e-9 {
                                d / len
                            } else {
                                pose.normals[i].normalize_or_zero()
                            };
                            self.points[i] = center + normal * r;
                            contacts[i].push(normal);
                        }
                    }
                    for capsule in &pose.capsules {
                        if let Some((p, normal)) =
                            capsule.contact(self.points[i], g.radius.max(0.0) * scale)
                        {
                            self.points[i] = p;
                            contacts[i].push(normal);
                        }
                    }
                }
            }
            for i in 0..n {
                self.velocities[i] = (self.points[i] - before[i]) / h;
                let skin_velocity = (pose.targets[i] - from[i]) / dt;
                for &normal in &contacts[i] {
                    let into = (self.velocities[i] - skin_velocity).dot(normal).min(0.0);
                    self.velocities[i] -= normal * into;
                }
                if w[i] > 0.0 && g.radius > 0.0 {
                    solids.collide_point(
                        &mut self.points[i],
                        &mut self.velocities[i],
                        g.radius * scale,
                        g.friction.clamp(0.0, 1.0),
                        1.0 / w[i],
                        h,
                    );
                }
            }
        }
    }
}
