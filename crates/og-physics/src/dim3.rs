//! The 3D profile (rapier3d): conversions and shapes. The step and the
//! queries are `sim.rs`, shared with the 2D profile.

use crate::schema::*;
use crate::{ShapeKey, Shared};
use og_core::glam::{Quat, Vec3};
use og_core::{Error, ErrorKind, Result};
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
        SHAPE_CONVEX | SHAPE_TRIMESH => return Ok(None),
        other => {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("og.Collider.shape {other} is not a shape (0 box, 1 sphere, 2 capsule, 3 cylinder, 4 convex, 5 trimesh)"),
            ))
        }
    }))
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

#[allow(clippy::duplicate_mod)]
#[path = "sim.rs"]
mod sim;
pub(crate) use sim::{step, QueryWorld, Sim};
