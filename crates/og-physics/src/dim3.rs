//! The 3D profile (rapier3d): conversions and shapes. The step and the
//! queries are `sim.rs`, shared with the 2D profile.

use crate::schema::*;
use crate::{ShapeKey, Shared};
use og_core::glam::{Quat, Vec3};
use og_core::{Error, ErrorKind, Result};
#[path = "raycast_vehicle.rs"]
pub(crate) mod ctl;
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
        SHAPE_DIAMOND | SHAPE_HEXAGON => {
            let h = c.size * s * 0.5;
            let points: Vec<rp::Vector> = polygon(c.shape, h.x, h.y)
                .into_iter()
                .flat_map(|[x, y]| [rp::Vector::new(x, y, -h.z), rp::Vector::new(x, y, h.z)])
                .collect();
            rp::SharedShape::convex_hull(&points).ok_or_else(|| {
                Error::new(
                    ErrorKind::Invalid,
                    format!(
                        "og.Collider.shape {} needs size.x, size.y and size.z above 0, got {}",
                        c.shape, c.size
                    ),
                )
            })?
        }
        SHAPE_CONVEX | SHAPE_TRIMESH | SHAPE_HEIGHTFIELD => return Ok(None),
        other => {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "og.Collider.shape {other} is not a shape (0 box, 1 sphere, 2 capsule, 3 cylinder, 4 convex, 5 trimesh, 6 height field, 7 diamond, 8 hexagon)"
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

/// A shape built from a registered mesh, scaled.
pub(crate) fn from_mesh(shape: u32, mesh: &crate::MeshDesc, s: Vec3) -> Result<rp::SharedShape> {
    match shape {
        SHAPE_CONVEX => {
            let points: Vec<rp::Vector> = mesh.positions.iter().map(|p| v(*p * s)).collect();
            rp::SharedShape::convex_hull(&points).ok_or_else(|| {
                Error::new(
                    ErrorKind::Invalid,
                    "the mesh's convex hull is degenerate (flat or a single point)",
                )
            })
        }
        _ => trimesh(mesh, s),
    }
}

/// The triangle mesh (rapier's, with its BVH) over `mesh` scaled by `s`:
/// a SHAPE_TRIMESH collider's shape, and what [`bake`] bakes.
pub(crate) fn trimesh(mesh: &crate::MeshDesc, s: Vec3) -> Result<rp::SharedShape> {
    let points: Vec<rp::Vector> = mesh
        .positions
        .iter()
        .map(|p| {
            let p = *p * s;
            rp::Vector::new(p.x, p.y, p.z)
        })
        .collect();
    rp::SharedShape::trimesh(points, mesh.indices.clone()).map_err(|e| {
        Error::new(
            ErrorKind::Invalid,
            format!("the mesh is not a valid triangle mesh: {e}"),
        )
    })
}

/// What starts a baked triangle mesh's bytes: the format and its version,
/// moved with any rapier or parry update that changes `TriMesh`'s
/// serialisation (`a_baked_trimesh_is_the_build_bit_for_bit` pins a bake).
const BAKED: &[u8; 8] = b"OGTM\x01\x00\x00\x00";

/// [`crate::bake_trimesh`].
pub(crate) fn bake(mesh: &crate::MeshDesc) -> Result<Vec<u8>> {
    let shape = trimesh(mesh, Vec3::ONE)?;
    let mesh = shape.as_trimesh().expect("a triangle mesh");
    let mut out = BAKED.to_vec();
    crate::encode(&mut out, mesh)
        .map_err(|e| Error::new(ErrorKind::Invalid, format!("baking a triangle mesh: {e}")))?;
    Ok(out)
}

/// A [`bake`]'s shape.
pub(crate) fn unbake(bytes: &[u8]) -> Result<rp::SharedShape> {
    let body = bytes.strip_prefix(BAKED.as_slice()).ok_or_else(|| {
        Error::new(
            ErrorKind::Invalid,
            "not a baked triangle mesh of this engine's format (cook the model again)",
        )
    })?;
    let mesh: rp::TriMesh = crate::decode(body)
        .map_err(|e| Error::new(ErrorKind::Invalid, format!("a baked triangle mesh: {e}")))?;
    Ok(rp::SharedShape::new(mesh))
}

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
    let linear = unv(f1.rotation * rp::Vector::new(axis(0), axis(1), axis(2)));
    let angular = unv(f1.rotation * rp::Vector::new(axis(3), axis(4), axis(5)));
    let lever = unv(rb2.center_of_mass()) - anchor;
    (linear.length(), (angular + lever.cross(linear)).length())
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

#[path = "vehicle3.rs"]
mod vehicle;
pub(crate) use vehicle::{VehicleState, vehicles, wheels};

#[allow(clippy::duplicate_mod)]
#[path = "ragdoll.rs"]
mod ragdoll;
pub(crate) use ragdoll::{RagdollState, pose_of as ragdoll_pose};

#[cfg(test)]
mod tests {
    use super::*;

    /// A bumpy grid of `n` × `n` cells, two triangles each.
    fn grid(n: u32) -> crate::MeshDesc {
        let w = n + 1;
        let positions = (0..w * w)
            .map(|i| {
                let (x, z) = ((i % w) as f32, (i / w) as f32);
                Vec3::new(
                    x,
                    og_core::math::sin(x * 0.37) + og_core::math::cos(z * 0.21),
                    z,
                )
            })
            .collect();
        let indices = (0..n * n)
            .flat_map(|c| {
                let a = (c / n) * w + c % n;
                [[a, a + 1, a + w], [a + 1, a + w + 1, a + w]]
            })
            .collect();
        crate::MeshDesc { positions, indices }
    }

    #[test]
    fn a_baked_trimesh_is_the_build_bit_for_bit() {
        let mesh = grid(16);
        let built = from_mesh(SHAPE_TRIMESH, &mesh, Vec3::ONE).unwrap();
        let mut want = BAKED.to_vec();
        crate::encode(&mut want, built.as_trimesh().unwrap()).unwrap();
        let bytes = bake(&mesh).unwrap();
        assert_eq!(bytes, want);
        // Decoded, it serialises back to the same bytes: the same BVH.
        let back = unbake(&bytes).unwrap();
        let mut again = BAKED.to_vec();
        crate::encode(&mut again, back.as_trimesh().unwrap()).unwrap();
        assert_eq!(again, bytes);
        // The format is pinned: a rapier or parry update that moves it
        // moves BAKED's version with it.
        assert_eq!(
            og_core::hash::hash_bytes(&bake(&grid(2)).unwrap()),
            2732637514586692152,
            "the bake of a 2 x 2 grid"
        );
        assert!(unbake(&bytes[1..]).is_err());
    }

    #[test]
    fn a_collider_takes_the_bake_only_over_the_mesh_it_was_baked_from() {
        let mesh = grid(4);
        let mut shared = Shared::default();
        shared
            .baked3
            .insert(9, unbake(&bake(&mesh).unwrap()).unwrap());
        let c = Collider {
            shape: SHAPE_TRIMESH,
            mesh: 9,
            ..Collider::default()
        };
        let got = baked(&shared, &c, Vec3::ONE, &mesh).expect("the bake");
        assert!(std::sync::Arc::ptr_eq(&got.0, &shared.baked3[&9].0));
        assert!(
            baked(&shared, &c, Vec3::splat(2.0), &mesh).is_none(),
            "scaled"
        );
        let mut moved = mesh.clone();
        moved.positions[3].y += 1e-6;
        assert!(
            baked(&shared, &c, Vec3::ONE, &moved).is_none(),
            "another mesh"
        );
        let convex = Collider {
            shape: SHAPE_CONVEX,
            ..c
        };
        assert!(baked(&shared, &convex, Vec3::ONE, &mesh).is_none());
    }
}
