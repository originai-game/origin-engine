//! The union of a mesh with itself: a surface folded through itself made one closed solid.

use super::*;

/// A self-union's output ([`self_union_points`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SelfUnion {
    /// The points the triangles use.
    pub points: Vec<DVec3>,
    /// The triangles, facing out.
    pub tris: Vec<[u32; 3]>,
    /// Each triangle's source triangle in the input.
    pub source: Vec<u32>,
    /// Edges `[a, b]`, `a < b`, along which the surface crossed itself.
    pub seams: Vec<[u32; 2]>,
}

/// The outside surface of a closed, oriented mesh that may pass through
/// itself: the arrangement [`csg_points`] builds for two operands, built
/// for one. Every pair of its triangles that crosses ([`selfcheck`]) is
/// intersected and cut, and each region is classified by the mesh's own
/// winding number. What is kept bounds the space of winding number at
/// least 1, facing out of it; a surface where the winding goes from 1 to 2,
/// or from -1 to 0, is dropped. Where faces lie on one another, the one
/// from the lowest-numbered triangle stands for them.
///
/// The operand is refused as [`csg_points`] refuses one (operand 0),
/// except that a directed edge may be used more than once if its reverse
/// is used as often (two copies of a face).
pub fn self_union_points(mesh: (&[DVec3], &[[u32; 3]])) -> Result<SelfUnion, OperandError> {
    let mut points = Vec::new();
    let mut at = BTreeMap::new();
    let s = weld64(0, true, mesh, &mut points, &mut at)?;
    let (c, kept) = arrange_self(points, s);
    let (points, tris, origins, remap) = emit(&c, &kept);
    let seams = c
        .crossings
        .iter()
        .filter_map(|&(a, b)| {
            let (x, y) = (*remap.get(&a)?, *remap.get(&b)?);
            Some([x.min(y), x.max(y)])
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    Ok(SelfUnion {
        points,
        tris,
        source: origins.into_iter().map(|o| o.1).collect(),
        seams,
    })
}
