//! The 3D profile (rapier3d): conversions and shapes. The step and the
//! queries are `rigid/sim.rs`, shared with the 2D profile.

use crate::schema::*;
use crate::{ShapeKey, Shared};
use og_core::glam::{Quat, Vec3};
use og_core::{Error, ErrorKind, Result};
use rapier3d::parry::query::{ShapeCastOptions, contact};
use rapier3d::prelude as rp;
use std::collections::BTreeMap;

pub(crate) const PROFILE_NAME: &str = "3D";

#[path = "water/geometry3.rs"]
pub(crate) mod water_geometry;

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
    let r = c.round_radius.max(0.0);
    Ok(Some(match c.shape {
        SHAPE_BOX => {
            let h = c.size * s * 0.5;
            if r > 0.0 {
                rp::SharedShape::round_cuboid(h.x, h.y, h.z, r)
            } else {
                rp::SharedShape::cuboid(h.x, h.y, h.z)
            }
        }
        SHAPE_SPHERE => rp::SharedShape::ball(c.radius * s.max_element()),
        SHAPE_CAPSULE => rp::SharedShape::capsule_y(c.height * 0.5 * s.y, c.radius * s.x.max(s.z)),
        SHAPE_CYLINDER => {
            let (hh, rad) = (c.height * 0.5 * s.y, c.radius * s.x.max(s.z));
            if r > 0.0 {
                rp::SharedShape::round_cylinder(hh, rad, r)
            } else {
                rp::SharedShape::cylinder(hh, rad)
            }
        }
        SHAPE_CONE => {
            let (hh, rad) = (c.height * 0.5 * s.y, c.radius * s.x.max(s.z));
            if r > 0.0 {
                rp::SharedShape::round_cone(hh, rad, r)
            } else {
                rp::SharedShape::cone(hh, rad)
            }
        }
        SHAPE_DIAMOND | SHAPE_HEXAGON => {
            let h = c.size * s * 0.5;
            let points: Vec<rp::Vector> = polygon(c.shape, h.x, h.y)
                .into_iter()
                .flat_map(|[x, y]| [rp::Vector::new(x, y, -h.z), rp::Vector::new(x, y, h.z)])
                .collect();
            let built = if r > 0.0 {
                rp::SharedShape::round_convex_hull(&points, r)
            } else {
                rp::SharedShape::convex_hull(&points)
            };
            built.ok_or_else(|| {
                Error::new(
                    ErrorKind::Invalid,
                    format!(
                        "og.Collider.shape {} needs size.x, size.y and size.z above 0, got {}",
                        c.shape, c.size
                    ),
                )
            })?
        }
        SHAPE_CONVEX | SHAPE_TRIMESH | SHAPE_HEIGHTFIELD | SHAPE_POLYLINE => return Ok(None),
        other => {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "og.Collider.shape {other} is not a shape (0 box, 1 sphere, 2 capsule, 3 cylinder, 4 convex, 5 trimesh, 6 height field, 7 diamond, 8 hexagon, 9 cone, 10 polyline)"
                ),
            ));
        }
    }))
}

/// A height field's shape, scaled: parry's heights by row (z) and column
/// (x), using the same adaptive diagonal as og-render; a hole's cell removed.
pub(crate) fn from_heightfield(f: &crate::HeightfieldDesc, s: Vec3) -> Result<rp::SharedShape> {
    let all = ([0, f.columns - 1], [0, f.rows - 1]);
    Ok(field_rect(f, s, all))
}

/// Chunk `k` of a height field ([`crate::FIELD_CHUNK`]), scaled: the
/// field's heights, diagonals and holes over the chunk's cells, centred on
/// the chunk ([`crate::chunk_offset`] places it).
pub(crate) fn field_chunk(f: &crate::HeightfieldDesc, s: Vec3, k: u32) -> rp::SharedShape {
    field_rect(f, s, crate::chunk_samples(f, k))
}

pub(crate) fn chunk_cache(shared: &mut Shared) -> &mut BTreeMap<ShapeKey, rp::SharedShape> {
    &mut shared.chunks3
}

/// The height field over columns `c0..=c1` and rows `r0..=r1` of `f`.
fn field_rect(
    f: &crate::HeightfieldDesc,
    s: Vec3,
    ([c0, c1], [r0, r1]): ([u32; 2], [u32; 2]),
) -> rp::SharedShape {
    use rapier3d::parry::shape::{HeightField, HeightFieldCellStatus};
    use rapier3d::parry::utils::Array2;
    let cols = f.columns as usize;
    let (c0, c1, r0, r1) = (c0 as usize, c1 as usize, r0 as usize, r1 as usize);
    let (nr, nc) = (r1 - r0 + 1, c1 - c0 + 1);
    let h = &f.heights;
    // Column-major: element (row, column) at row + column · rows.
    let mut data = vec![0.0; nr * nc];
    for r in 0..nr {
        for c in 0..nc {
            data[r + c * nr] = h[(r0 + r) * cols + c0 + c];
        }
    }
    let extent = if (nc, nr) == (cols, f.rows as usize) {
        f.size
    } else {
        crate::field_cell(f) * og_core::glam::Vec2::new((nc - 1) as f32, (nr - 1) as f32)
    };
    let mut field = HeightField::new(
        Array2::new(nr, nc, data),
        rp::Vector::new(extent.x * s.x, s.y, extent.y * s.z),
    );
    for r in r0..r1 {
        for c in c0..c1 {
            let hole = f
                .holes
                .as_ref()
                .is_some_and(|holes| holes[r * (cols - 1) + c] != 0);
            let mut status = HeightFieldCellStatus::empty();
            if og_core::math::heightfield_zigzag(
                h[r * cols + c],
                h[r * cols + c + 1],
                h[(r + 1) * cols + c],
                h[(r + 1) * cols + c + 1],
            ) {
                status |= HeightFieldCellStatus::ZIGZAG_SUBDIVISION;
            }
            if hole {
                status |= HeightFieldCellStatus::CELL_REMOVED;
            }
            field.set_cell_status(r - r0, c - c0, status);
        }
    }
    rp::SharedShape::new(field)
}

/// Segments extracted from a `MeshDesc` for [`SHAPE_POLYLINE`]: empty `indices`
/// gives a single open chain `0..n`; each `[a, b, c]` with `c == a || c == b`
/// yields a single segment `[a, b]`, otherwise the three triangle edges.
pub(crate) fn polyline_segments(mesh: &crate::MeshDesc) -> Option<Vec<[u32; 2]>> {
    if mesh.indices.is_empty() {
        return None;
    }
    let mut segs = Vec::with_capacity(mesh.indices.len());
    for &[a, b, c] in &mesh.indices {
        if c == a || c == b {
            if a != b {
                segs.push([a, b]);
            }
        } else {
            segs.push([a, b]);
            segs.push([b, c]);
            segs.push([c, a]);
        }
    }
    Some(segs)
}

/// A shape built from a registered mesh, scaled.
pub(crate) fn from_mesh(
    shape: u32,
    mesh: &crate::MeshDesc,
    s: Vec3,
    round_radius: f32,
) -> Result<rp::SharedShape> {
    let r = round_radius.max(0.0);
    match shape {
        SHAPE_CONVEX => {
            let points: Vec<rp::Vector> = mesh.positions.iter().map(|p| v(*p * s)).collect();
            let built = if r > 0.0 {
                rp::SharedShape::round_convex_hull(&points, r)
            } else {
                rp::SharedShape::convex_hull(&points)
            };
            built.ok_or_else(|| {
                Error::new(
                    ErrorKind::Invalid,
                    "the mesh's convex hull is degenerate (flat or a single point)",
                )
            })
        }
        SHAPE_POLYLINE => {
            if mesh.positions.len() < 2 {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    "a polyline collider needs at least 2 vertices",
                ));
            }
            let points: Vec<rp::Vector> = mesh.positions.iter().map(|p| v(*p * s)).collect();
            let segs = polyline_segments(mesh);
            if segs.as_ref().is_some_and(|s| s.is_empty()) {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    "a polyline collider has no non-degenerate segments",
                ));
            }
            Ok(rp::SharedShape::polyline(points, segs))
        }
        _ => trimesh(mesh, s),
    }
}

pub(crate) fn gyroscopic_builder(b: rp::RigidBodyBuilder, enabled: bool) -> rp::RigidBodyBuilder {
    b.gyroscopic_forces_enabled(enabled)
}

pub(crate) fn set_gyroscopic(rb: &mut rp::RigidBody, enabled: bool) {
    rb.enable_gyroscopic_forces(enabled);
}

pub(crate) fn generic_lock_axes(lock: u32) -> rp::JointAxesMask {
    let mut m = rp::JointAxesMask::empty();
    if lock & LOCK_TRANSLATION_X != 0 {
        m |= rp::JointAxesMask::LIN_X;
    }
    if lock & LOCK_TRANSLATION_Y != 0 {
        m |= rp::JointAxesMask::LIN_Y;
    }
    if lock & LOCK_TRANSLATION_Z != 0 {
        m |= rp::JointAxesMask::LIN_Z;
    }
    if lock & LOCK_ROTATION_X != 0 {
        m |= rp::JointAxesMask::ANG_X;
    }
    if lock & LOCK_ROTATION_Y != 0 {
        m |= rp::JointAxesMask::ANG_Y;
    }
    if lock & LOCK_ROTATION_Z != 0 {
        m |= rp::JointAxesMask::ANG_Z;
    }
    m
}

pub(crate) fn set_ball_limits(data: &mut rp::GenericJoint, min: f32, max: f32) {
    let swing = max.abs().max(min.abs());
    data.set_limits(rp::JointAxis::AngX, [-swing, swing]);
    data.set_limits(rp::JointAxis::AngY, [min, max]);
    data.set_limits(rp::JointAxis::AngZ, [-swing, swing]);
}

pub(crate) fn principal_inertia(rb: &rp::RigidBody) -> Vec3 {
    let inv = rb.mass_properties().local_mprops.inv_principal_inertia;
    let to_i = |x: f32| if x > 1e-20 { 1.0 / x } else { 0.0 };
    Vec3::new(to_i(inv.x), to_i(inv.y), to_i(inv.z))
}

pub(crate) use og_physics_data::trimesh::{trimesh, unbake};

/// The baked shape of a SHAPE_TRIMESH collider at scale 1 whose mesh is
/// exactly the one baked (else `None`: it is built as before).
pub(crate) fn baked(
    shared: &Shared,
    c: &Collider,
    s: Vec3,
    mesh: &crate::MeshDesc,
) -> Option<rp::SharedShape> {
    if c.shape != SHAPE_TRIMESH || s != Vec3::ONE {
        return None;
    }
    let shape = shared.baked3.get(&c.mesh)?;
    let tri = shape.as_trimesh()?;
    let same =
        tri.indices() == mesh.indices.as_slice()
            && tri.vertices().len() == mesh.positions.len()
            && tri.vertices().iter().zip(&mesh.positions).all(|(a, b)| {
                unv(*a).to_array().map(f32::to_bits) == b.to_array().map(f32::to_bits)
            });
    same.then(|| shape.clone())
}

/// The profile's dimensions.
pub(crate) const DIM: usize = 3;

/// Whether a point's projection on `c` can only say "inside" when it lies
/// on the surface: a triangle mesh without oriented pseudo-normals (parry's
/// triangle reports inside only for a point it projects onto itself). A
/// point farther than `radius` from such a shape is never pushed, so its
/// projection may stop at `radius` ([`crate::ext`]'s `Near`).
pub(crate) fn surface_only(c: &rp::Collider) -> bool {
    c.shape()
        .as_trimesh()
        .is_some_and(|m| m.pseudo_normals_if_oriented().is_none())
}

/// For a height-field collider with `p` over it: the point of its surface
/// straight above or below `p` (along the field's up) and the surface's
/// unit normal there, pointing out of the ground. `None` for other shapes
/// and off the field's edges. Parry projects a point onto the nearest
/// triangle and never says it is inside, so a point that fell below the
/// surface would be pushed further down; this tells which side it is on.
pub(crate) fn heightfield_surface(c: &rp::Collider, p: Vec3) -> Option<(Vec3, Vec3)> {
    let field = c.shape().as_heightfield()?;
    let pose = c.position();
    let local = pose.inverse_transform_point(v(p));
    let (i, j) = field.cell_at_point(local)?;
    let (first, second) = field.triangles_at(i, j);
    // The triangle under `local` in xz (or the nearer one, at an edge).
    let mut best: Option<(f32, rp::Vector, rp::Vector)> = None;
    for t in [first, second].into_iter().flatten() {
        let n = (t.b - t.a).cross(t.c - t.a);
        if n.y.abs() <= 1e-12 {
            continue;
        }
        let n = if n.y < 0.0 { -n } else { n };
        let (e1x, e1z) = (t.b.x - t.a.x, t.b.z - t.a.z);
        let (e2x, e2z) = (t.c.x - t.a.x, t.c.z - t.a.z);
        let det = e1x * e2z - e2x * e1z;
        if det.abs() <= 1e-12 {
            continue;
        }
        let (px, pz) = (local.x - t.a.x, local.z - t.a.z);
        let u = (px * e2z - e2x * pz) / det;
        let w = (e1x * pz - px * e1z) / det;
        let outside = (-u).max(-w).max(u + w - 1.0).max(0.0);
        if best.is_none_or(|(o, ..)| outside < o) {
            let y = t.a.y - (n.x * px + n.z * pz) / n.y;
            best = Some((
                outside,
                rp::Vector::new(local.x, y, local.z),
                n / n.length(),
            ));
        }
    }
    let (_, q, up) = best?;
    Some((unv(pose.transform_point(q)), unv(pose.rotation * up)))
}

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
pub(crate) fn ragdoll_joint(
    frame1: rp::Pose,
    frame2: rp::Pose,
    bone: &crate::RagdollBone,
    limit: f32,
) -> rp::GenericJoint {
    let axes = if bone.joint_kind == 1 {
        rp::JointAxesMask::LOCKED_REVOLUTE_AXES
    } else {
        rp::JointAxesMask::LOCKED_SPHERICAL_AXES
    };
    let mut j = rp::GenericJointBuilder::new(axes)
        .local_frame1(frame1)
        .local_frame2(frame2)
        .contacts_enabled(false)
        .build();
    let swing = if bone.swing > 0.0 { bone.swing } else { limit };
    if bone.joint_kind == 0 && swing > 0.0 {
        j.coupled_axes = rp::JointAxesMask::ANG_Y | rp::JointAxesMask::ANG_Z;
        j.set_limits(rp::JointAxis::AngY, [0.0, swing]);
    }
    if bone.twist_min != 0.0 || bone.twist_max != 0.0 {
        j.set_limits(rp::JointAxis::AngX, [bone.twist_min, bone.twist_max]);
    } else if limit > 0.0 {
        j.set_limits(rp::JointAxis::AngX, [-limit, limit]);
    }
    j
}

pub(crate) fn ragdoll_shape(kind: u32, length: f32, radius: f32) -> rp::SharedShape {
    match kind {
        1 => rp::SharedShape::cuboid(radius, length * 0.5, radius),
        2 => rp::SharedShape::ball(radius),
        _ => rp::SharedShape::capsule_y((length * 0.5 - radius).max(0.0), radius),
    }
}

pub(crate) fn ragdoll_angvel(rb: &mut rp::RigidBody, angular: Vec3) {
    rb.set_angvel(v(angular), true);
}

pub(crate) fn ragdoll_motor(j: &mut rp::GenericJoint, target: rp::Pose, k: f32, d: f32) {
    let (_, q) = unpose(&target, Vec3::ZERO);
    let q = if q.w < 0.0 { -q } else { q };
    j.motor_axes = rp::JointAxesMask::empty();
    for (axis, component) in [
        (rp::JointAxis::AngX, q.x),
        (rp::JointAxis::AngY, q.y),
        (rp::JointAxis::AngZ, q.z),
    ] {
        if !j.locked_axes.contains(axis.into()) && (k > 0.0 || d > 0.0) {
            // Match Rapier's independent angular-coordinate error, not Euler
            // angles. Force-based PD uses the actual inertia about the joint,
            // including COM offset, rather than the body's bare rotational inertia.
            let component = component.clamp(-1.0, 1.0);
            let angle = 2.0
                * og_core::math::atan2(component, (1.0 - component * component).max(0.0).sqrt());
            j.set_motor_model(axis, rp::MotorModel::ForceBased);
            j.set_motor_position(axis, angle, k.max(0.0), d.max(0.0));
        }
    }
}

/// The axes a ball joint locks.
/// The impulse a joint's solver put on its body (rapier's body 2) in its
/// last solver substep, as a linear and an angular impulse magnitude at the
/// joint anchor: its locked axes, limits and motors summed per axis.
pub(crate) fn joint_load(j: &rp::ImpulseJoint, bodies: &rp::RigidBodySet) -> (f32, f32) {
    let axis = |i: usize| j.impulses[i] + j.data.limits[i].impulse + j.data.motors[i].impulse;
    let linear = if j.data.coupled_axes.contains(rp::JointAxesMask::LIN_AXES) {
        // Springs and ropes couple linear axes into LinX along the anchor-to-anchor line.
        axis(0).abs()
    } else {
        Vec3::new(axis(0), axis(1), axis(2)).length()
    };
    let angular = Vec3::new(axis(3), axis(4), axis(5)).length();
    let _ = bodies;
    (linear, angular)
}

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
    scene: &crate::gather::Scene,
    _settings: &Physics,
    _dt: f32,
    _sim: &mut Sim,
    _handles: &sim::Handles<'_>,
    _out: &mut crate::StepOut,
) -> Result<()> {
    if scene
        .bodies
        .iter()
        .any(|b| matches!(b.control, Some(crate::gather::Control::Platformer(_))))
    {
        return Err(Error::new(
            ErrorKind::Invalid,
            "og.PlatformerController requires the 2D physics profile",
        ));
    }
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
#[path = "rigid/sim.rs"]
mod sim;

#[allow(clippy::duplicate_mod)]
#[path = "rigid/debug_lines.rs"]
pub(crate) mod debug_lines;
pub(crate) use sim::{Sim, load, query_shape, save, step};

#[allow(clippy::duplicate_mod)]
#[path = "ext.rs"]
mod ext;

#[allow(clippy::duplicate_mod)]
#[path = "character/mover.rs"]
mod mover;

#[path = "vehicle/vehicle3.rs"]
mod vehicle;
pub(crate) use vehicle::{VehicleState, readback as vehicle_readback, vehicles, wheels};

#[allow(clippy::duplicate_mod)]
#[path = "character/ragdoll.rs"]
mod ragdoll;
pub(crate) use ragdoll::{
    RagdollState, impulse as ragdoll_impulse, pose_of as ragdoll_pose, readout as ragdoll_readout,
};
