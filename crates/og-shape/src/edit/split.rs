//! split: a solid cut by a plane, one side kept or both.

use super::*;

/// Which side of the plane [`split`] keeps.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Keep {
    /// On the side the frame's normal (+z) points to.
    #[default]
    Above,
    /// The other side.
    Below,
}

/// The solid cut by the plane through `frame.origin` normal to `frame.z`.
pub fn split(s: &Solid, frame: &Frame, keep: Keep) -> Result<Solid, EditError> {
    if crate::mesh::rides(s) {
        let mut out = split(&crate::mesh::bare(s), frame, keep)?;
        let _ = crate::mesh::carry(&[s], &mut out);
        return Ok(out);
    }
    let l = 4.0 * size(s)
        + 4.0 * (frame.origin - s.positions.first().copied().unwrap_or_default()).length();
    let desc = crate::generators::solids::BoxDesc {
        width: 2.0 * l,
        depth: 2.0 * l,
        height: l,
    };
    // With UVs, the cut face takes the box's bottom: planar, in mm on the
    // plane's own x and y.
    let half = if s.uvs.is_some() {
        textured(&Generator::Box(desc))?
    } else {
        crate::generators::solids::cuboid(&desc)?
    };
    let tool = placed(half, frame);
    let op = match keep {
        Keep::Above => BoolOp::Intersect,
        Keep::Below => BoolOp::Cut,
    };
    uv_boolean(
        s,
        &tool,
        &Boolean {
            op,
            target: "",
            tool: "split",
        },
        "the split",
    )
    .map(|(s, _)| s)
}
