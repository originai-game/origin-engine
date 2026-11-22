//! hole: a bore with an optional counterbore or countersink, entered exactly at the surface.

use super::*;

/// Round holes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Hole {
    /// Diameter.
    pub diameter: f64,
    /// Where: each frame's origin above the part, its +z pointing out of
    /// the material. The hole starts where its axis enters the part.
    pub at: Vec<Frame>,
    /// Depth from the entry; 0 for right through.
    pub depth: f64,
    /// Counterbore `(diameter, depth)`; zeros for none.
    pub counterbore: (f64, f64),
    /// Countersink `(diameter, full angle in radians)`; zeros for none.
    pub countersink: (f64, f64),
}

/// A frame looking down at `(x, y)` from above: what a plain point means.
pub fn above(x: f64, y: f64) -> Frame {
    Frame {
        origin: DVec3::new(x, y, 0.0),
        ..Frame::IDENTITY
    }
}

/// A tool along a frame's −z: from `over` above the entry to `depth` below,
/// its polygon turned by `turn` radians about the axis.
#[allow(clippy::too_many_arguments)]
pub(crate) fn bore(
    entry: DVec3,
    axis: &Frame,
    turn: f64,
    r_bottom: f64,
    r_top: f64,
    depth: f64,
    over: f64,
    n: u32,
    uvs: bool,
) -> Result<Solid, EditError> {
    let circle = crate::profile::circle(&crate::profile::Circle {
        radius: r_bottom,
        segments: n,
    })
    .map_err(GenError::from)?;
    let c = tool_extrude(
        &circle,
        &Extrude {
            height: depth + over,
            taper: 1.0 - r_top / r_bottom,
            ..Default::default()
        },
        uvs,
    )?;
    let f = Frame {
        origin: entry - axis.z * depth,
        ..*axis
    };
    let f = if turn == 0.0 {
        f
    } else {
        f.turn(0.0, 0.0, turn)
    };
    Ok(placed(c, &f))
}

/// Holes of `d.diameter` into the solid. Mouth edges get `hole`; the bore
/// `hole.side`, a blind floor `hole.bottom`.
pub fn hole(s: &Solid, d: &Hole) -> Result<Solid, EditError> {
    hole_report(s, d).map(|(s, _)| s)
}

/// [`hole`], with what the result alone does not say ([`EditReport`]).
pub fn hole_report(s: &Solid, d: &Hole) -> Result<(Solid, EditReport), EditError> {
    if crate::mesh::rides(s) {
        let (mut out, report) = hole_report(&crate::mesh::bare(s), d)?;
        let _ = crate::mesh::carry(&[s], &mut out);
        return Ok((out, report));
    }
    if !positive(d.diameter) {
        return Err(EditError::NotPositive("diameter"));
    }
    let span = size(s);
    let over = span * 1e-3;
    let mut out = s.clone();
    let mut report = EditReport::default();
    for f in &d.at {
        let axis = Frame::from_normal(DVec3::ZERO, f.z, f.x).ok_or(EditError::NoDirection("at"))?;
        let e = entry_of(&out, f, &axis, span)?;
        let depth = if d.depth > 0.0 { d.depth } else { 4.0 * span };
        let r = d.diameter / 2.0;
        // One segment count for every tool of this hole, so a countersink's
        // narrow end is the bore's own polygon.
        let widest = r.max(d.counterbore.0 / 2.0).max(d.countersink.0 / 2.0);
        let n = crate::profile::curve::segments(widest, crate::math::trig::TAU);
        let uvs = s.uvs.is_some();
        // The tools turned by `turn` about the hole's axis.
        let tools_at = |turn: f64| -> Result<Vec<Solid>, EditError> {
            let mut tools = vec![bore(e, &axis, turn, r, r, depth, over, n, uvs)?];
            let (cb_d, cb_depth) = d.counterbore;
            if cb_d > 0.0 && cb_depth > 0.0 {
                tools.push(bore(
                    e,
                    &axis,
                    turn,
                    cb_d / 2.0,
                    cb_d / 2.0,
                    cb_depth,
                    over,
                    n,
                    uvs,
                )?);
            }
            let (cs_d, cs_angle) = d.countersink;
            if cs_d > 2.0 * r && cs_angle > 0.0 {
                // The cone runs on below the bore's radius (to 0.9 r), so it
                // crosses the bore wall instead of touching it along a ring.
                // It is turned half a segment about the axis: its generators
                // then run between the bore's columns, so no rim point is made
                // twice an ulp apart (column through cone, generator through
                // wall), and the rim does not cross itself in either cut order
                // (`docs/research.md` §18.5, §23).
                let slope = crate::math::trig::tan(cs_angle / 2.0);
                let h = (cs_d / 2.0 - 0.9 * r) / slope;
                tools.push(bore(
                    e,
                    &axis,
                    turn + crate::math::trig::PI / n as f64,
                    0.9 * r,
                    cs_d / 2.0 + over * slope,
                    h,
                    over,
                    n,
                    uvs,
                )?);
            }
            Ok(tools)
        };
        // As it stands; and where the cut comes out open or not
        // manifold (the tool's wall through an edge two other faces
        // already meet on: a third hole through the crossing of two),
        // again with the tool turned a share of one of its segments
        // about its axis, which moves its corners off that edge and
        // changes no measure of the hole.
        let step = crate::math::trig::TAU / n as f64;
        let mut first: Option<EditError> = None;
        let mut cut = None;
        for share in [0.0, 1.0 / 7.0, 1.0 / 3.0, 3.0 / 5.0] {
            let attempt = (|| {
                let mut at = out.clone();
                let mut r = EditReport::default();
                for t in tools_at(share * step)? {
                    let (next, b) = uv_boolean(
                        &at,
                        &t,
                        &Boolean {
                            op: BoolOp::Cut,
                            target: "",
                            tool: "hole",
                        },
                        "the hole",
                    )?;
                    at = next;
                    r.add(&b);
                }
                Ok((at, r))
            })();
            match attempt {
                Ok(done) => {
                    cut = Some(done);
                    break;
                }
                Err(
                    e @ EditError::Bool(
                        BoolError::NotManifold { .. }
                        | BoolError::OpenResult { .. }
                        | BoolError::OpenTouching { .. },
                    ),
                ) => {
                    first.get_or_insert(e);
                }
                Err(e) => return Err(e),
            }
        }
        let Some((next, r)) = cut else {
            return Err(first.unwrap_or(EditError::NotClosed));
        };
        out = next;
        report.crossing_pairs += r.crossing_pairs;
    }
    Ok((out, report))
}
