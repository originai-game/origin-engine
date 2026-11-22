//! Named revolve: a profile turned round an axis, its ends and walls named.

use super::*;

/// The axis a turned solid turns about.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Axis {
    /// +y, three.js's lathe axis: the profile's y stays y.
    #[default]
    Y,
    /// +z, sand3d's: the profile's y becomes z, and the round primitives
    /// rest on z = 0.
    Z,
}

/// A solid made about +y, turned a quarter about +x so +y goes to +z:
/// `(x, y, z) → (x, −z, y)`, exact.
pub(crate) fn y_to_z(s: &mut Solid, lift: f64) -> Result<(), GenError> {
    s.map_positions(|p| DVec3::new(p.x, -p.z, p.y + lift), false);
    s.canonicalize()
        .map_err(|_| GenError::NotFinite("positions"))
}

/// Revolution about +y (or +z).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Revolve {
    /// Angle swept, radians; 0 for a full turn.
    pub angle: f64,
    /// The axis; +y by default.
    pub axis: Axis,
    /// Segments a full turn (a part turn gets its share); 0 for the chord
    /// rule ([`crate::profile::curve::segments`]).
    pub segments: u32,
}

/// A profile in the xy plane (x ≥ 0, x the radius) turned about +y,
/// counter-clockwise seen from +y, starting at +x. A partial turn has
/// `start` and `end` caps. With [`Axis::Z`] the profile's y is the
/// height along +z and the turn is counter-clockwise seen from +z, from
/// +x (sand3d's `revolve`).
pub fn revolve(profile: &Profile, d: &Revolve) -> Result<Solid, GenError> {
    revolve_with(profile, d, false)
}

pub(crate) fn revolve_with(profile: &Profile, d: &Revolve, uvs: bool) -> Result<Solid, GenError> {
    let angle = finite(d.angle, "angle")?;
    let per_turn = segments_field(d.segments, "segments")?;
    let full = angle == 0.0 || angle.abs() >= trig::TAU;
    let sweep = if full { trig::TAU } else { angle };
    if profile.regions.is_empty() {
        return Err(ProfileError::Empty.into());
    }
    if profile
        .contours()
        .flat_map(|c| &c.points)
        .any(|p| p.x < 0.0)
    {
        return Err(GenError::CrossesAxis);
    }
    let reach = profile
        .contours()
        .flat_map(|c| &c.points)
        .fold(0.0f64, |m, p| m.max(p.x));
    let steps = count(per_turn, reach, sweep);
    let at = |p: DVec2, k: u32| {
        // The last column of a full turn is the first, exactly.
        let k = if full && k == steps { 0 } else { k };
        let (s, c) = trig::sin_cos(sweep * (k as f64 / steps as f64));
        DVec3::new(p.x * c, p.y, -p.x * s)
    };
    let mut s = Solid::default();
    let mut surface = 0;
    for c in profile.contours() {
        let n = c.points.len();
        let cols: Vec<Vec<u32>> = (0..=steps)
            .map(|k| c.points.iter().map(|p| s.push_vertex(at(*p, k))).collect())
            .collect();
        let (along, total) = arc_lengths(&c.points);
        // Round the axis at the point's radius; along the contour.
        let uv = |i: usize, v: f64, k: usize| {
            DVec2::new(c.points[i].x * (sweep * (k as f64 / steps as f64)), v)
        };
        for (a, b) in runs(c) {
            for i in a..b {
                let (i0, i1) = (i % n, (i + 1) % n);
                let (v0, v1) = (along[i0], arc_end(&along, total, i0));
                for k in 0..steps as usize {
                    let (p, q) = (&cols[k], &cols[k + 1]);
                    let (p0, p1) = (uv(i0, v0, k), uv(i1, v1, k));
                    let (q0, q1) = (uv(i0, v0, k + 1), uv(i1, v1, k + 1));
                    push(
                        &mut s,
                        uvs,
                        [p[i0], q[i0], q[i1]],
                        surface,
                        &["side"],
                        [p0, q0, q1],
                    );
                    push(
                        &mut s,
                        uvs,
                        [p[i0], q[i1], p[i1]],
                        surface,
                        &["side"],
                        [p0, q1, p1],
                    );
                }
            }
            surface += 1;
        }
        for i in 0..n {
            if c.sharp[i] {
                for k in 0..steps as usize {
                    s.tag_edge(cols[k][i], cols[k + 1][i], &["rim"]);
                }
            }
            if !full {
                s.tag_edge(cols[0][i], cols[0][(i + 1) % n], &["start"]);
                s.tag_edge(
                    cols[steps as usize][i],
                    cols[steps as usize][(i + 1) % n],
                    &["end"],
                );
            }
        }
    }
    if !full {
        let fill = profile.triangulate()?;
        let first: Vec<u32> = fill
            .points
            .iter()
            .map(|p| s.push_vertex(at(*p, 0)))
            .collect();
        let last: Vec<u32> = fill
            .points
            .iter()
            .map(|p| s.push_vertex(at(*p, steps)))
            .collect();
        for t in &fill.triangles {
            let profile_uv = |k: [u32; 3]| k.map(|i| fill.points[i as usize]);
            push(
                &mut s,
                uvs,
                t.map(|i| first[i as usize]),
                surface,
                &["start"],
                profile_uv(*t),
            );
            push(
                &mut s,
                uvs,
                [
                    last[t[0] as usize],
                    last[t[2] as usize],
                    last[t[1] as usize],
                ],
                surface + 1,
                &["end"],
                profile_uv([t[0], t[2], t[1]]),
            );
        }
    }
    let mut s = finish(s)?;
    if d.axis == Axis::Z {
        y_to_z(&mut s, 0.0)?;
    }
    Ok(s)
}
