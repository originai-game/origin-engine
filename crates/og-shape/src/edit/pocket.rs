//! pocket: a profile sunk into a face to a depth.

use super::*;

/// A profile cut in from a face.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pocket {
    /// The outline, in the frames' xy plane.
    pub profile: Profile,
    /// Depth from the entry; 0 for right through.
    pub depth: f64,
    /// Where: frames as for [`Hole::at`]; empty for one over the origin.
    pub at: Vec<Frame>,
}

/// The profile cut into the solid from where each frame's axis enters it.
/// Floor and walls are named `pocket.bottom` and `pocket.side`.
pub fn pocket(s: &Solid, d: &Pocket) -> Result<Solid, EditError> {
    pocket_report(s, d).map(|(s, _)| s)
}

/// [`pocket`], with what the result alone does not say ([`EditReport`]).
pub fn pocket_report(s: &Solid, d: &Pocket) -> Result<(Solid, EditReport), EditError> {
    if crate::mesh::rides(s) {
        let (mut out, report) = pocket_report(&crate::mesh::bare(s), d)?;
        let _ = crate::mesh::carry(&[s], &mut out);
        return Ok((out, report));
    }
    let span = size(s);
    let over = span * 1e-3;
    let frames = if d.at.is_empty() {
        vec![above(0.0, 0.0)]
    } else {
        d.at.clone()
    };
    let mut out = s.clone();
    let mut report = EditReport::default();
    for f in &frames {
        let axis = Frame::from_normal(DVec3::ZERO, f.z, f.x).ok_or(EditError::NoDirection("at"))?;
        let e = entry_of(&out, f, &axis, span)?;
        let depth = if d.depth > 0.0 { d.depth } else { 4.0 * span };
        let tool = tool_extrude(
            &d.profile,
            &Extrude {
                height: depth + over,
                ..Default::default()
            },
            s.uvs.is_some(),
        )?;
        let frame = Frame {
            origin: e - axis.z * depth,
            ..axis
        };
        let (next, r) = uv_boolean(
            &out,
            &placed(tool, &frame),
            &Boolean {
                op: BoolOp::Cut,
                target: "",
                tool: "pocket",
            },
            "the pocket",
        )?;
        out = next;
        report.add(&r);
    }
    Ok((out, report))
}
