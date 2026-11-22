//! Named extrude: a profile pushed along +z, caps and walls named.

use super::*;

/// Extrusion along +z.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Extrude {
    /// Height along +z.
    pub height: f64,
    /// Turn of the top relative to the bottom, radians, about z.
    pub twist: f64,
    /// Shrink of the top: its scale is `1 - taper` (0 keeps it, 1 makes
    /// an apex, negative widens it).
    pub taper: f64,
    /// Centre the solid on z = 0 instead of starting there.
    pub both_sides: bool,
}

/// A profile extruded along +z: `bottom` at z = 0, `top` at `height`.
pub fn extrude(profile: &Profile, d: &Extrude) -> Result<Solid, GenError> {
    extrude_with(profile, d, false)
}

pub(crate) fn extrude_with(profile: &Profile, d: &Extrude, uvs: bool) -> Result<Solid, GenError> {
    let h = positive(d.height, "height")?;
    let twist = finite(d.twist, "twist")?;
    let taper = finite(d.taper, "taper")?;
    if taper > 1.0 {
        return Err(GenError::NotPositive("1 - taper"));
    }
    if profile.regions.is_empty() {
        return Err(ProfileError::Empty.into());
    }
    let reach = profile
        .contours()
        .flat_map(|c| &c.points)
        .fold(0.0f64, |m, p| m.max(p.length()));
    let layers = if twist == 0.0 {
        1
    } else {
        segments(reach, twist)
    };
    let z0 = if d.both_sides { -h / 2.0 } else { 0.0 };
    let at = |p: DVec2, layer: u32| {
        let t = layer as f64 / layers as f64;
        let (s, c) = double::sin_cos(twist * t);
        let k = 1.0 - taper * t;
        DVec3::new((c * p.x - s * p.y) * k, (s * p.x + c * p.y) * k, z0 + h * t)
    };
    let mut s = Solid::default();
    // Caps.
    let fill = profile.triangulate()?;
    let bottom: Vec<u32> = fill
        .points
        .iter()
        .map(|p| s.push_vertex(at(*p, 0)))
        .collect();
    let top: Vec<u32> = fill
        .points
        .iter()
        .map(|p| s.push_vertex(at(*p, layers)))
        .collect();
    let planar = |t: [u32; 3], s: &Solid| t.map(|v| s.positions[v as usize].truncate());
    for t in &fill.triangles {
        let b = [
            bottom[t[0] as usize],
            bottom[t[2] as usize],
            bottom[t[1] as usize],
        ];
        let uv = planar(b, &s);
        push(&mut s, uvs, b, 0, &["bottom"], uv);
        let u = [top[t[0] as usize], top[t[1] as usize], top[t[2] as usize]];
        let uv = planar(u, &s);
        push(&mut s, uvs, u, 1, &["top"], uv);
    }
    // Walls, one surface per run between corners.
    let mut surface = 2;
    for region in &profile.regions {
        for (ci, c) in std::iter::once(&region.outer)
            .chain(&region.holes)
            .enumerate()
        {
            let tags: &[&str] = if ci == 0 {
                &["side"]
            } else {
                &["side", "inner"]
            };
            let n = c.points.len();
            let ring: Vec<Vec<u32>> = (0..=layers)
                .map(|l| c.points.iter().map(|p| s.push_vertex(at(*p, l))).collect())
                .collect();
            let (along, total) = arc_lengths(&c.points);
            for (a, b) in runs(c) {
                for i in a..b {
                    let (i0, i1) = (i % n, (i + 1) % n);
                    let (u0, u1) = (along[i0], arc_end(&along, total, i0));
                    for l in 0..layers as usize {
                        let (p, q) = (&ring[l], &ring[l + 1]);
                        let v0 = h * (l as f64 / layers as f64);
                        let v1 = h * ((l + 1) as f64 / layers as f64);
                        let (a0, a1) = (DVec2::new(u0, v0), DVec2::new(u1, v0));
                        let (b0, b1) = (DVec2::new(u0, v1), DVec2::new(u1, v1));
                        push(
                            &mut s,
                            uvs,
                            [p[i0], p[i1], q[i1]],
                            surface,
                            tags,
                            [a0, a1, b1],
                        );
                        push(
                            &mut s,
                            uvs,
                            [p[i0], q[i1], q[i0]],
                            surface,
                            tags,
                            [a0, b1, b0],
                        );
                    }
                }
                surface += 1;
            }
            for i in 0..n {
                s.tag_edge(ring[0][i], ring[0][(i + 1) % n], &["bottom"]);
                s.tag_edge(
                    ring[layers as usize][i],
                    ring[layers as usize][(i + 1) % n],
                    &["top"],
                );
                if c.sharp[i] {
                    for l in 0..layers as usize {
                        s.tag_edge(ring[l][i], ring[l + 1][i], &["side"]);
                    }
                }
            }
        }
    }
    finish(s)
}
