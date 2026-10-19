//! The 3D profile (rapier3d): conversions and shapes. The step and the
//! queries are `sim.rs`, shared with the 2D profile.

use crate::schema::*;
use crate::{ShapeKey, Shared};
use og_core::glam::{Quat, Vec3};
use og_core::{Error, ErrorKind, Result};
pub(crate) use rapier3d::control as ctl;
use rapier3d::parry::query::ShapeCastOptions;
use rapier3d::prelude as rp;
use std::collections::BTreeMap;

pub(crate) const PROFILE_NAME: &str = "3D";

/// Drops what the profile does not simulate (nothing, in 3D).
pub(crate) fn flat(x: Vec3) -> Vec3 {
    x
}

pub(crate) fn v(x: Vec3) -> rp::Vector {
    rp::Vector::new(x.x, x.y, x.z)
}

pub(crate) fn unv(x: rp::Vector) -> Vec3 {
    Vec3::new(x.x, x.y, x.z)
}

pub(crate) fn pose(position: Vec3, q: Quat) -> rp::Pose {
    let q = crate::unit(q);
    rp::Pose::from_parts(v(position), rp::Rotation::from_xyzw(q.x, q.y, q.z, q.w))
}

/// The transform a pose writes back.
pub(crate) fn unpose(pose: &rp::Pose, _before: Vec3) -> (Vec3, Quat) {
    let r = pose.rotation;
    (unv(pose.translation), Quat::from_xyzw(r.x, r.y, r.z, r.w))
}

pub(crate) fn angvel(a: Vec3) -> rp::AngVector {
    v(a)
}

pub(crate) fn unangvel(a: rp::AngVector) -> Vec3 {
    unv(a)
}

pub(crate) fn cache(shared: &mut Shared) -> &mut BTreeMap<ShapeKey, rp::SharedShape> {
    &mut shared.shapes3
}

/// A primitive shape, or `None` for the mesh shapes.
pub(crate) fn primitive(c: &Collider, s: Vec3) -> Result<Option<rp::SharedShape>> {
    Ok(Some(match c.shape {
        SHAPE_BOX => {
            let h = c.size * s * 0.5;
            rp::SharedShape::cuboid(h.x, h.y, h.z)
        }
        SHAPE_SPHERE => rp::SharedShape::ball(c.radius * s.max_element()),
        SHAPE_CAPSULE => rp::SharedShape::capsule_y(c.height * 0.5 * s.y, c.radius * s.x.max(s.z)),
        SHAPE_CYLINDER => rp::SharedShape::cylinder(c.height * 0.5 * s.y, c.radius * s.x.max(s.z)),
        SHAPE_CONVEX | SHAPE_TRIMESH | SHAPE_HEIGHTFIELD => return Ok(None),
        other => {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "og.Collider.shape {other} is not a shape (0 box, 1 sphere, 2 capsule, 3 cylinder, 4 convex, 5 trimesh, 6 height field)"
                ),
            ));
        }
    }))
}

/// A height field's shape, scaled: parry's heights by row (z) and column
/// (x), each cell split along the (-x, -z) to (+x, +z) diagonal (its zig-zag
/// subdivision), the triangles og-render draws.
pub(crate) fn from_heightfield(f: &crate::HeightfieldDesc, s: Vec3) -> Result<rp::SharedShape> {
    use rapier3d::parry::shape::{HeightField, HeightFieldCellStatus};
    use rapier3d::parry::utils::Array2;
    let (rows, cols) = (f.rows as usize, f.columns as usize);
    // Column-major: element (row, column) at row + column · rows.
    let mut data = vec![0.0; rows * cols];
    for r in 0..rows {
        for c in 0..cols {
            data[r + c * rows] = f.heights[r * cols + c];
        }
    }
    let mut field = HeightField::new(
        Array2::new(rows, cols, data),
        rp::Vector::new(f.size.x * s.x, s.y, f.size.y * s.z),
    );
    for r in 0..rows - 1 {
        for c in 0..cols - 1 {
            field.set_cell_status(r, c, HeightFieldCellStatus::ZIGZAG_SUBDIVISION);
        }
    }
    Ok(rp::SharedShape::new(field))
}

/// A shape built from a registered mesh, scaled.
pub(crate) fn from_mesh(shape: u32, mesh: &crate::MeshDesc, s: Vec3) -> Result<rp::SharedShape> {
    let points: Vec<rp::Vector> = mesh.positions.iter().map(|p| v(*p * s)).collect();
    match shape {
        SHAPE_CONVEX => rp::SharedShape::convex_hull(&points).ok_or_else(|| {
            Error::new(
                ErrorKind::Invalid,
                "the mesh's convex hull is degenerate (flat or a single point)",
            )
        }),
        _ => rp::SharedShape::trimesh(points, mesh.indices.clone()).map_err(|e| {
            Error::new(
                ErrorKind::Invalid,
                format!("the mesh is not a valid triangle mesh: {e}"),
            )
        }),
    }
}

/// The profile's dimensions.
pub(crate) const DIM: usize = 3;

/// The rotation of a joint's frame whose x is `axis` (the hinge or slide axis).
pub(crate) fn joint_frame(axis: Vec3) -> rp::Rotation {
    let q = crate::arc(Vec3::X, axis.normalize_or(Vec3::X));
    rp::Rotation::from_xyzw(q.x, q.y, q.z, q.w)
}

/// A rotation as the profile's: here the quaternion itself, as it is (a
/// capsule's frame from `arc`, already unit: renormalising it would move
/// its last bits, and the E5 ragdolls with them).
pub(crate) fn rot(q: Quat) -> rp::Rotation {
    rp::Rotation::from_xyzw(q.x, q.y, q.z, q.w)
}

/// A ragdoll bone's joint to its parent: a ball joint (contacts off) with
/// `limit` of swing about x and z and half that of twist about y.
pub(crate) fn ragdoll_joint(frame1: rp::Pose, limit: f32) -> rp::GenericJoint {
    rp::GenericJointBuilder::new(rp::JointAxesMask::LOCKED_SPHERICAL_AXES)
        .local_frame1(frame1)
        .local_frame2(rp::Pose::IDENTITY)
        .contacts_enabled(false)
        .limits(rp::JointAxis::AngX, [-limit, limit])
        .limits(rp::JointAxis::AngY, [-limit * 0.5, limit * 0.5])
        .limits(rp::JointAxis::AngZ, [-limit, limit])
        .build()
}

/// The axes a ball joint locks.
pub(crate) fn ball_axes() -> rp::JointAxesMask {
    rp::JointAxesMask::LOCKED_SPHERICAL_AXES
}

/// A hinge's angle: the rotation of frame `f2` about frame `f1`'s x.
pub(crate) fn hinge_angle(f1: &rp::Pose, f2: &rp::Pose) -> f32 {
    let r = f1.rotation.inverse() * f2.rotation;
    let (x, w) = if r.w < 0.0 { (-r.x, -r.w) } else { (r.x, r.w) };
    2.0 * og_core::math::atan2(x, w)
}

/// The 2D platformer controller is 2D only.
pub(crate) fn platformers(
    _scene: &crate::gather::Scene,
    _settings: &Physics,
    _dt: f32,
    _sim: &mut Sim,
    _poses: &sim::BodyPoses,
    _out: &mut crate::StepOut,
) -> Result<()> {
    Ok(())
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

#[path = "vehicle3.rs"]
mod vehicle;
pub(crate) use vehicle::{VehicleState, vehicles, wheels};

#[allow(clippy::duplicate_mod)]
#[path = "ragdoll.rs"]
mod ragdoll;
pub(crate) use ragdoll::{RagdollState, pose_of as ragdoll_pose};
