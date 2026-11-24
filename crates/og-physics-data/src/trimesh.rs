//! A triangle mesh collider's shape and its bake.

use og_core::glam::Vec3;
use og_core::{Error, ErrorKind, Result};
use rapier3d::prelude as rp;

/// The triangle mesh (rapier's, with its BVH) over `mesh` scaled by `s`:
/// a SHAPE_TRIMESH collider's shape, and what [`bake_collider`] bakes.
pub fn trimesh(mesh: &crate::MeshDesc, s: Vec3) -> Result<rp::SharedShape> {
    let points: Vec<rp::Vector> = mesh
        .positions
        .iter()
        .map(|p| {
            let p = *p * s;
            rp::Vector::new(p.x, p.y, p.z)
        })
        .collect();
    let indices = if s.x * s.y * s.z < 0.0 {
        mesh.indices.iter().map(|&[a, b, c]| [a, c, b]).collect()
    } else {
        mesh.indices.clone()
    };
    rp::SharedShape::trimesh_with_flags(points, indices, rp::TriMeshFlags::ORIENTED).map_err(|e| {
        Error::new(
            ErrorKind::Invalid,
            format!("the mesh is not a valid triangle mesh: {e}"),
        )
    })
}

/// Baked collider format; change it with a Parry serialization change.
pub const BAKED: &[u8; 8] = b"OGPC\x01\x00\x00\x00";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum BakeKind {
    #[default]
    Trimesh,
    Convex,
    Vhacd,
}

pub struct BakedCollider {
    pub hash: u64,
    pub convex: bool,
    pub shape: rp::SharedShape,
}

/// Build the collider between ticks, in the cook. A convex decomposition
/// is serialized as its compound of real convex hulls, not voxel samples.
pub fn bake_collider(mesh: &crate::MeshDesc, kind: BakeKind) -> Result<Vec<u8>> {
    mesh.check(0)?;
    let hash = mesh.content_hash();
    let points: Vec<_> = mesh
        .positions
        .iter()
        .map(|p| rp::Vector::new(p.x, p.y, p.z))
        .collect();
    let shape = match kind {
        BakeKind::Trimesh => trimesh(mesh, Vec3::ONE)?,
        BakeKind::Convex => rp::SharedShape::convex_hull(&points).ok_or_else(|| {
            Error::new(
                ErrorKind::Invalid,
                "the collider's convex hull is degenerate",
            )
        })?,
        BakeKind::Vhacd => {
            if mesh.indices.is_empty() {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    "VHACD needs a closed triangle mesh",
                ));
            }
            rp::SharedShape::convex_decomposition(&points, &mesh.indices)
        }
    };
    let mut out = BAKED.to_vec();
    out.extend_from_slice(&hash.to_le_bytes());
    out.push(u8::from(kind != BakeKind::Trimesh));
    crate::encode(&mut out, &shape)
        .map_err(|e| Error::new(ErrorKind::Invalid, format!("baking a triangle mesh: {e}")))?;
    Ok(out)
}

/// A [`bake_collider`]'s shape.
pub fn unbake(bytes: &[u8]) -> Result<BakedCollider> {
    let body = bytes.strip_prefix(BAKED.as_slice()).ok_or_else(|| {
        Error::new(
            ErrorKind::Invalid,
            "not a baked triangle mesh of this engine's format (cook the model again)",
        )
    })?;
    let (hash, body) = body
        .split_first_chunk::<8>()
        .ok_or_else(|| Error::new(ErrorKind::Invalid, "a baked triangle mesh is cut short"))?;
    let (convex, body) = body
        .split_first()
        .ok_or_else(|| Error::new(ErrorKind::Invalid, "a baked collider is cut short"))?;
    if *convex > 1 {
        return Err(Error::new(
            ErrorKind::Invalid,
            "invalid baked collider kind",
        ));
    }
    let shape: rp::SharedShape = crate::decode(body)
        .map_err(|e| Error::new(ErrorKind::Invalid, format!("a baked triangle mesh: {e}")))?;
    Ok(BakedCollider {
        hash: u64::from_le_bytes(*hash),
        convex: *convex != 0,
        shape,
    })
}
