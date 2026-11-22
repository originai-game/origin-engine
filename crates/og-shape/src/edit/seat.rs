//! seat: a part sunk into a host so the two meet on a named face.

use super::*;

/// The host with the volume `part` fills taken out; `part` is untouched.
/// Refused if they share no volume.
pub fn seat(host: &Solid, part: &Solid, part_name: &str) -> Result<Solid, EditError> {
    seat_report(host, part, part_name).map(|(s, _)| s)
}

/// [`seat`], with what the result alone does not say ([`EditReport`]).
pub fn seat_report(
    host: &Solid,
    part: &Solid,
    part_name: &str,
) -> Result<(Solid, EditReport), EditError> {
    if crate::mesh::rides(host) || crate::mesh::rides(part) {
        crate::mesh::one_skin(&[host, part]).map_err(|_| EditError::TwoSkins)?;
        let (mut out, report) = seat_report(
            &crate::mesh::bare(host),
            &crate::mesh::bare(part),
            part_name,
        )?;
        let _ = crate::mesh::carry(&[host, part], &mut out);
        return Ok((out, report));
    }
    let shared = boolean(
        host,
        part,
        &Boolean {
            op: BoolOp::Intersect,
            ..Default::default()
        },
    );
    match shared {
        Err(BoolError::Empty) => return Err(EditError::NoOverlap),
        Err(e) => return Err(e.into()),
        Ok(v) if v.measure().volume <= 0.0 => return Err(EditError::NoOverlap),
        Ok(_) => {}
    }
    let (out, r) = uv_boolean(
        host,
        part,
        &Boolean {
            op: BoolOp::Cut,
            target: "",
            tool: part_name,
        },
        &format!("the part {part_name:?}"),
    )?;
    let mut report = EditReport::default();
    report.add(&r);
    Ok((out, report))
}
