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

/// No shape is baked for the 2D profile (its meshes are outlines).
pub(crate) fn baked(
    _shared: &Shared,
    _c: &Collider,
    _s: Vec3,
    _mesh: &crate::MeshDesc,
) -> Option<rp::SharedShape> {
    None
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
        SHAPE_DIAMOND | SHAPE_HEXAGON => {
            let h = c.size * s * 0.5;
            let points: Vec<rp::Vector> = polygon(c.shape, h.x, h.y)
                .into_iter()
                .map(|[x, y]| rp::Vector::new(x, y))
                .collect();
            rp::SharedShape::convex_polyline(points).ok_or_else(|| degenerate(c))?
        }
        SHAPE_CONVEX | SHAPE_TRIMESH => return Ok(None),
        SHAPE_CYLINDER | SHAPE_HEIGHTFIELD => {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "og.Collider.shape {} is 3D only; the 2D profile has 0 box, 1 circle, 2 capsule, 4 polygon, 5 outline, 7 diamond, 8 hexagon",
                    c.shape
                ),
            ));
        }
        other => {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "og.Collider.shape {other} is not a shape (2D: 0 box, 1 circle, 2 capsule, 4 polygon, 5 outline, 7 diamond, 8 hexagon)"
                ),
            ));
        }
    }))
}

fn degenerate(c: &Collider) -> Error {
    Error::new(
        ErrorKind::Invalid,
        format!(
            "og.Collider.shape {} needs size.x and size.y above 0, got {} x {}",
            c.shape, c.size.x, c.size.y
        ),
    )
}

/// The convex polygon of a registered mesh's xy, scaled.
/// Height fields are 3D only (refused by [`primitive`] before this).
pub(crate) fn from_heightfield(_f: &crate::HeightfieldDesc, _s: Vec3) -> Result<rp::SharedShape> {
    Err(Error::new(
        ErrorKind::Invalid,
        "og.Collider.shape 6 (a height field) is 3D only",
    ))
}

pub(crate) fn field_chunk(_f: &crate::HeightfieldDesc, _s: Vec3, _k: u32) -> rp::SharedShape {
    unreachable!("height fields are refused in 2D before they are built")
}

pub(crate) fn chunk_cache(shared: &mut Shared) -> &mut BTreeMap<ShapeKey, rp::SharedShape> {
    &mut shared.chunks2
}

pub(crate) fn from_mesh(shape: u32, mesh: &crate::MeshDesc, s: Vec3) -> Result<rp::SharedShape> {
    if shape == SHAPE_TRIMESH {
        return outline_shape(mesh, s);
    }
    let points: Vec<rp::Vector> = mesh.positions.iter().map(|p| v(*p * s)).collect();
    rp::SharedShape::convex_hull(&points).ok_or_else(|| {
        Error::new(
            ErrorKind::Invalid,
            "the mesh's convex polygon is degenerate (its xy are collinear or a single point)",
        )
    })
}

/// A 2D triangle mesh: the polyline of its outline ([`crate::outline`]),
/// one closed chain per loop, scaled.
fn outline_shape(mesh: &crate::MeshDesc, s: Vec3) -> Result<rp::SharedShape> {
    let scaled = crate::MeshDesc {
        positions: mesh.positions.iter().map(|p| *p * s).collect(),
        indices: mesh.indices.clone(),
    };
    let mut points = Vec::new();
    let mut segments = Vec::new();
    for l in crate::outline::outline(&scaled) {
        let start = points.len() as u32;
        let n = l.len() as u32;
        points.extend(l.iter().map(|&i| v(scaled.positions[i as usize])));
        segments.extend((0..n).map(|i| [start + i, start + (i + 1) % n]));
    }
    if segments.is_empty() {
        return Err(Error::new(
            ErrorKind::Invalid,
            "the mesh has no outline in the xy plane (no triangle of non-zero area there)",
        ));
    }
    Ok(rp::SharedShape::polyline(points, Some(segments)))
}

/// The profile's dimensions.
pub(crate) const DIM: usize = 2;

/// A 2D triangle is solid (a point inside it projects onto an edge as
/// inside), so every 2D projection is a whole one (see dim3's).
pub(crate) fn surface_only(_c: &rp::Collider) -> bool {
    false
}

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
/// The impulse a joint's solver put on its body (rapier's body 2) in its
/// last solver substep, as a force-like and a torque-like size: its locked
/// axes, limits and motors summed per axis, turned to world space, the
/// torque about the joint's anchor (rapier applies the linear part at the
/// body's centre of mass, so its lever to the anchor is added).
pub(crate) fn joint_load(j: &rp::ImpulseJoint, bodies: &rp::RigidBodySet) -> (f32, f32) {
    let axis = |i: usize| j.impulses[i] + j.data.limits[i].impulse + j.data.motors[i].impulse;
    let f1 = *bodies[j.body1()].position() * j.data.local_frame1;
    let rb2 = &bodies[j.body2()];
    let anchor = unv((*rb2.position() * j.data.local_frame2).translation);
    let linear = unv(f1.rotation * rp::Vector::new(axis(0), axis(1)));
    let lever = unv(rb2.center_of_mass()) - anchor;
    (linear.length(), (axis(2) + lever.cross(linear).z).abs())
}

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

#[allow(clippy::duplicate_mod)]
#[path = "debug_lines.rs"]
pub(crate) mod debug_lines;
pub(crate) use sim::{QueryWorld, Sim, load, save, step};

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
