//! The 2D profile (rapier2d): bodies move in the xy plane and rotate about
//! z; a transform keeps its z and its quaternion becomes a rotation about z.
//! The step and the queries are `sim.rs`, shared with the 3D profile.

use crate::schema::*;
use crate::{ShapeKey, Shared};
use og_core::glam::{Quat, Vec3};
use og_core::{Error, ErrorKind, Result};
use rapier2d::parry::query::ShapeCastOptions;
use rapier2d::prelude as rp;
use std::collections::BTreeMap;

pub(crate) const PROFILE_NAME: &str = "2D";

/// Drops what the profile does not simulate (z).
pub(crate) fn flat(x: Vec3) -> Vec3 {
    Vec3::new(x.x, x.y, 0.0)
}

pub(crate) fn v(x: Vec3) -> rp::Vector {
    rp::Vector::new(x.x, x.y)
}

pub(crate) fn unv(x: rp::Vector) -> Vec3 {
    Vec3::new(x.x, x.y, 0.0)
}

/// The rotation about z of a quaternion: the direction its x axis takes in
/// the xy plane (exact for a pure z rotation; a tilted one is projected).
pub(crate) fn rot(q: Quat) -> rp::Rotation {
    let q = crate::unit(q);
    let c = 1.0 - 2.0 * (q.y * q.y + q.z * q.z);
    let s = 2.0 * (q.x * q.y + q.w * q.z);
    let len = (c * c + s * s).sqrt();
    if len > 1e-12 && len.is_finite() {
        rp::Rotation::from_cos_sin_unchecked(c / len, s / len)
    } else {
        rp::Rotation::IDENTITY
    }
}

/// The quaternion of a rotation about z by the angle whose cosine and sine
/// are `c`, `s` (half-angle identities: square roots only, no trigonometry).
pub(crate) fn unrot(r: rp::Rotation) -> Quat {
    let (c, s) = (r.re, r.im);
    let (z, w) = if c >= 0.0 {
        let w = ((1.0 + c) * 0.5).sqrt();
        (s / (2.0 * w), w)
    } else {
        let z = ((1.0 - c) * 0.5).sqrt().copysign(s);
        (z, s / (2.0 * z))
    };
    Quat::from_xyzw(0.0, 0.0, z, w).normalize()
}

pub(crate) fn pose(position: Vec3, q: Quat) -> rp::Pose {
    rp::Pose::from_parts(v(position), rot(q))
}

/// The transform a pose writes back: x and y move, z stays.
pub(crate) fn unpose(pose: &rp::Pose, before: Vec3) -> (Vec3, Quat) {
    let t = pose.translation;
    (Vec3::new(t.x, t.y, before.z), unrot(pose.rotation))
}

pub(crate) fn angvel(a: Vec3) -> rp::AngVector {
    a.z
}

pub(crate) fn unangvel(a: rp::AngVector) -> Vec3 {
    Vec3::new(0.0, 0.0, a)
}

pub(crate) fn cache(shared: &mut Shared) -> &mut BTreeMap<ShapeKey, rp::SharedShape> {
    &mut shared.shapes2
}

/// A primitive shape, or `None` for the mesh shapes.
pub(crate) fn primitive(c: &Collider, s: Vec3) -> Result<Option<rp::SharedShape>> {
    Ok(Some(match c.shape {
        SHAPE_BOX => {
            let h = c.size * s * 0.5;
            rp::SharedShape::cuboid(h.x, h.y)
        }
        SHAPE_SPHERE => rp::SharedShape::ball(c.radius * s.x.max(s.y)),
        SHAPE_CAPSULE => rp::SharedShape::capsule_y(c.height * 0.5 * s.y, c.radius * s.x),
        SHAPE_CONVEX => return Ok(None),
        SHAPE_CYLINDER | SHAPE_TRIMESH | SHAPE_HEIGHTFIELD => {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "og.Collider.shape {} is 3D only; the 2D profile has 0 box, 1 circle, 2 capsule, 4 polygon",
                    c.shape
                ),
            ));
        }
        other => {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "og.Collider.shape {other} is not a shape (2D: 0 box, 1 circle, 2 capsule, 4 polygon)"
                ),
            ));
        }
    }))
}

/// The convex polygon of a registered mesh's xy, scaled.
/// Height fields are 3D only (refused by [`primitive`] before this).
pub(crate) fn from_heightfield(_f: &crate::HeightfieldDesc, _s: Vec3) -> Result<rp::SharedShape> {
    Err(Error::new(
        ErrorKind::Invalid,
        "og.Collider.shape 6 (a height field) is 3D only",
    ))
}

pub(crate) fn from_mesh(_shape: u32, mesh: &crate::MeshDesc, s: Vec3) -> Result<rp::SharedShape> {
    let points: Vec<rp::Vector> = mesh.positions.iter().map(|p| v(*p * s)).collect();
    rp::SharedShape::convex_hull(&points).ok_or_else(|| {
        Error::new(
            ErrorKind::Invalid,
            "the mesh's convex polygon is degenerate (its xy are collinear or a single point)",
        )
    })
}

/// The profile's dimensions.
pub(crate) const DIM: usize = 2;

/// Height fields are 3D only.
pub(crate) fn heightfield_surface(_c: &rp::Collider, _p: Vec3) -> Option<(Vec3, Vec3)> {
    None
}

/// The rotation of a joint's frame whose x is `axis` in the xy plane.
pub(crate) fn joint_frame(axis: Vec3) -> rp::Rotation {
    let len = (axis.x * axis.x + axis.y * axis.y).sqrt();
    if len > 1e-9 && len.is_finite() {
        rp::Rotation::from_cos_sin_unchecked(axis.x / len, axis.y / len)
    } else {
        rp::Rotation::IDENTITY
    }
}

/// A ball joint in the plane is a hinge.
pub(crate) fn ball_axes() -> rp::JointAxesMask {
    rp::JointAxesMask::LOCKED_REVOLUTE_AXES
}

/// A hinge's angle: frame `f2`'s rotation relative to `f1`'s.
pub(crate) fn hinge_angle(f1: &rp::Pose, f2: &rp::Pose) -> f32 {
    let r = f1.rotation.inverse() * f2.rotation;
    og_core::math::atan2(r.im, r.re)
}

/// A ragdoll bone's joint to its parent: a hinge about z (contacts off)
/// bending `limit` either way from the pose it was made in.
pub(crate) fn ragdoll_joint(frame1: rp::Pose, limit: f32) -> rp::GenericJoint {
    rp::GenericJointBuilder::new(rp::JointAxesMask::LOCKED_REVOLUTE_AXES)
        .local_frame1(frame1)
        .local_frame2(rp::Pose::IDENTITY)
        .contacts_enabled(false)
        .limits(rp::JointAxis::AngX, [-limit, limit])
        .build()
}

pub(crate) fn ragdolls(
    scene: &crate::gather::Scene,
    dt: f32,
    sim: &mut Sim,
    shared: &Shared,
    out: &mut crate::StepOut,
) -> Result<()> {
    ragdoll::ragdolls(scene, dt, sim, &shared.rigs, &shared.ragdoll_targets, out)
}

#[allow(clippy::duplicate_mod)]
#[path = "sim.rs"]
mod sim;
pub(crate) use sim::{QueryWorld, Sim, step};

#[allow(clippy::duplicate_mod)]
#[path = "ext.rs"]
mod ext;

#[allow(clippy::duplicate_mod)]
#[path = "mover.rs"]
mod mover;

#[path = "platformer.rs"]
mod platformer;
pub(crate) use platformer::platformers;

#[path = "vehicle2.rs"]
mod vehicle;
pub(crate) use vehicle::{VehicleState, vehicles, wheels};

#[allow(clippy::duplicate_mod)]
#[path = "ragdoll.rs"]
mod ragdoll;
pub(crate) use ragdoll::{RagdollState, pose_of as ragdoll_pose};
