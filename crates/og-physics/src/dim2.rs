//! The 2D profile (rapier2d): bodies move in the xy plane and rotate about
//! z; a transform keeps its z and its quaternion becomes a rotation about z.
//! The step and the queries are `rigid/sim.rs`, shared with the 3D profile.

use crate::schema::*;
use crate::{ShapeKey, Shared};
use og_core::glam::{Quat, Vec3};
use og_core::{Error, ErrorKind, Result};
use rapier2d::parry::query::{ShapeCastOptions, contact};
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
    let r = c.round_radius.max(0.0);
    Ok(Some(match c.shape {
        SHAPE_BOX => {
            let h = c.size * s * 0.5;
            if r > 0.0 {
                rp::SharedShape::round_cuboid(h.x, h.y, r)
            } else {
                rp::SharedShape::cuboid(h.x, h.y)
            }
        }
        SHAPE_SPHERE => rp::SharedShape::ball(c.radius * s.x.max(s.y)),
        SHAPE_CAPSULE => rp::SharedShape::capsule_y(c.height * 0.5 * s.y, c.radius * s.x),
        SHAPE_DIAMOND | SHAPE_HEXAGON => {
            let h = c.size * s * 0.5;
            let points: Vec<rp::Vector> = polygon(c.shape, h.x, h.y)
                .into_iter()
                .map(|[x, y]| rp::Vector::new(x, y))
                .collect();
            let built = if r > 0.0 {
                rp::SharedShape::round_convex_polyline(points, r)
            } else {
                rp::SharedShape::convex_polyline(points)
            };
            built.ok_or_else(|| degenerate(c))?
        }
        SHAPE_CONVEX | SHAPE_TRIMESH | SHAPE_HEIGHTFIELD | SHAPE_POLYLINE => return Ok(None),
        SHAPE_CYLINDER | SHAPE_CONE => {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "og.Collider.shape {} is 3D only; the 2D profile has 0 box, 1 circle, 2 capsule, 4 polygon, 5 outline, 6 heightfield, 7 diamond, 8 hexagon, 10 polyline",
                    c.shape
                ),
            ));
        }
        other => {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "og.Collider.shape {other} is not a shape (2D: 0 box, 1 circle, 2 capsule, 4 polygon, 5 outline, 6 heightfield, 7 diamond, 8 hexagon, 10 polyline)"
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

/// A 2D height field over the first row (`columns` samples) of `f`, scaled by `s.x` and `s.y`.
pub(crate) fn from_heightfield(f: &crate::HeightfieldDesc, s: Vec3) -> Result<rp::SharedShape> {
    let cols = f.columns as usize;
    if cols < 2 || f.heights.len() < cols || s.x <= 0.0 {
        return Err(Error::new(
            ErrorKind::Invalid,
            "2D heightfield collider needs at least 2 samples and positive x scale",
        ));
    }
    let heights: Vec<f32> = f.heights[..cols].to_vec();
    let field =
        rapier2d::parry::shape::HeightField::new(heights, rp::Vector::new(f.size.x * s.x, s.y));
    Ok(rp::SharedShape::new(field))
}

pub(crate) fn field_chunk(_f: &crate::HeightfieldDesc, _s: Vec3, _k: u32) -> rp::SharedShape {
    unreachable!("2D height fields are not chunked")
}

pub(crate) fn chunk_cache(shared: &mut Shared) -> &mut BTreeMap<ShapeKey, rp::SharedShape> {
    &mut shared.chunks2
}

pub(crate) fn from_mesh(
    shape: u32,
    mesh: &crate::MeshDesc,
    s: Vec3,
    round_radius: f32,
) -> Result<rp::SharedShape> {
    let r = round_radius.max(0.0);
    if shape == SHAPE_TRIMESH {
        return outline_shape(mesh, s);
    }
    if shape == SHAPE_POLYLINE {
        if mesh.positions.len() < 2 {
            return Err(Error::new(
                ErrorKind::Invalid,
                "a polyline collider needs at least 2 vertices",
            ));
        }
        let points: Vec<rp::Vector> = mesh.positions.iter().map(|p| v(*p * s)).collect();
        let segs = crate::dim3::polyline_segments(mesh);
        if segs.as_ref().is_some_and(|s| s.is_empty()) {
            return Err(Error::new(
                ErrorKind::Invalid,
                "a polyline collider has no non-degenerate segments",
            ));
        }
        return Ok(rp::SharedShape::polyline(points, segs));
    }
    let points: Vec<rp::Vector> = mesh.positions.iter().map(|p| v(*p * s)).collect();
    let built = if r > 0.0 {
        rp::SharedShape::round_convex_hull(&points, r)
    } else {
        rp::SharedShape::convex_hull(&points)
    };
    built.ok_or_else(|| {
        Error::new(
            ErrorKind::Invalid,
            "the mesh's convex polygon is degenerate (its xy are collinear or a single point)",
        )
    })
}

pub(crate) fn gyroscopic_builder(b: rp::RigidBodyBuilder, _enabled: bool) -> rp::RigidBodyBuilder {
    b
}

pub(crate) fn set_gyroscopic(_rb: &mut rp::RigidBody, _enabled: bool) {}

pub(crate) fn generic_lock_axes(lock: u32) -> rp::JointAxesMask {
    let mut m = rp::JointAxesMask::empty();
    if lock & LOCK_TRANSLATION_X != 0 {
        m |= rp::JointAxesMask::LIN_X;
    }
    if lock & LOCK_TRANSLATION_Y != 0 {
        m |= rp::JointAxesMask::LIN_Y;
    }
    if lock & LOCK_ROTATION_Z != 0 {
        m |= rp::JointAxesMask::ANG_X;
    }
    m
}

pub(crate) fn set_ball_limits(data: &mut rp::GenericJoint, min: f32, max: f32) {
    data.set_limits(rp::JointAxis::AngX, [min, max]);
}

pub(crate) fn principal_inertia(rb: &rp::RigidBody) -> Vec3 {
    let inv = rb.mass_properties().local_mprops.inv_principal_inertia;
    let z = if inv > 1e-20 { 1.0 / inv } else { 0.0 };
    Vec3::new(0.0, 0.0, z)
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
/// last solver substep, as a linear and an angular impulse magnitude at the
/// joint anchor: its locked axes, limits and motors summed per axis.
pub(crate) fn joint_load(j: &rp::ImpulseJoint, bodies: &rp::RigidBodySet) -> (f32, f32) {
    let axis = |i: usize| j.impulses[i] + j.data.limits[i].impulse + j.data.motors[i].impulse;
    let linear = if j.data.coupled_axes.contains(rp::JointAxesMask::LIN_AXES) {
        axis(0).abs()
    } else {
        (axis(0) * axis(0) + axis(1) * axis(1)).sqrt()
    };
    let _ = bodies;
    (linear, axis(2).abs())
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
#[path = "rigid/sim.rs"]
mod sim;

#[allow(clippy::duplicate_mod)]
#[path = "rigid/debug_lines.rs"]
pub(crate) mod debug_lines;
pub(crate) use sim::{QueryWorld, Sim, load, query_shape, save, step};

#[allow(clippy::duplicate_mod)]
#[path = "ext.rs"]
mod ext;

#[allow(clippy::duplicate_mod)]
#[path = "character/mover.rs"]
mod mover;

#[path = "character/platformer.rs"]
mod platformer;
pub(crate) use platformer::platformers;

#[path = "vehicle/vehicle2.rs"]
mod vehicle;
pub(crate) use vehicle::{VehicleState, vehicles, wheels};

#[allow(clippy::duplicate_mod)]
#[path = "character/ragdoll.rs"]
mod ragdoll;
pub(crate) use ragdoll::{RagdollState, pose_of as ragdoll_pose};
