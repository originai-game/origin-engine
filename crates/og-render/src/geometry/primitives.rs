//! three.js's parametric primitives beside box, plane and sphere
//! (`shapes.rs`), with three.js's parameters, defaults, vertex order and
//! uvs: `CylinderGeometry`, `ConeGeometry`, `CapsuleGeometry`,
//! `TorusGeometry`, `RingGeometry`, `CircleGeometry` and `TubeGeometry`
//! (along points, a centripetal Catmull-Rom curve through them, as three.js
//! `CatmullRomCurve3`; or along any curve sampled by arc length,
//! [`tube_along`]: an og.path). Every mesh is counter-clockwise seen from outside
//! (front faces), +Y up, centred as three.js centres it.

use crate::scene::{MeshData, Vertex};
use glam::{Vec2, Vec3};
use std::f32::consts::{PI, TAU};

fn vertex(p: Vec3, n: Vec3, uv: Vec2) -> Vertex {
    Vertex {
        position: p.to_array(),
        normal: n.normalize_or(Vec3::Y).to_array(),
        uv: uv.to_array(),
        color: [1.0; 4],
        tangent: [0.0; 4],
    }
}

/// three.js `CylinderGeometry(radiusTop, radiusBottom, height,
/// radialSegments, heightSegments, openEnded, thetaStart, thetaLength)`:
/// centred at the origin along +Y, the side's uvs u around (from +Z
/// towards +X, as three.js) and v up, the caps' uvs a disc.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cylinder {
    pub radius_top: f32,
    pub radius_bottom: f32,
    pub height: f32,
    pub radial_segments: u32,
    pub height_segments: u32,
    pub open_ended: bool,
    pub theta_start: f32,
    pub theta_length: f32,
}

impl Default for Cylinder {
    fn default() -> Self {
        Cylinder {
            radius_top: 1.0,
            radius_bottom: 1.0,
            height: 1.0,
            radial_segments: 32,
            height_segments: 1,
            open_ended: false,
            theta_start: 0.0,
            theta_length: TAU,
        }
    }
}

/// Image coordinates: three.js's uvs have v up; the engine's run v down
/// (glTF), so every v here is flipped once.
fn uv(u: f32, v: f32) -> Vec2 {
    Vec2::new(u, 1.0 - v)
}

pub fn cylinder(c: &Cylinder) -> MeshData {
    let rs = c.radial_segments.max(3);
    let hs = c.height_segments.max(1);
    let half = c.height / 2.0;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    // The side: (hs + 1) rows of (rs + 1) vertices, top row first.
    let slope = (c.radius_bottom - c.radius_top) / c.height.max(1e-12);
    for y in 0..=hs {
        let v = y as f32 / hs as f32;
        let radius = v * (c.radius_bottom - c.radius_top) + c.radius_top;
        for x in 0..=rs {
            let u = x as f32 / rs as f32;
            let theta = u * c.theta_length + c.theta_start;
            let (s, co) = theta.sin_cos();
            let p = Vec3::new(radius * s, -v * c.height + half, radius * co);
            let n = Vec3::new(s, slope, co);
            vertices.push(vertex(p, n, uv(u, 1.0 - v)));
        }
    }
    for x in 0..rs {
        for y in 0..hs {
            let a = y * (rs + 1) + x;
            let b = (y + 1) * (rs + 1) + x;
            let cc = (y + 1) * (rs + 1) + x + 1;
            let d = y * (rs + 1) + x + 1;
            // Skip the degenerate triangle at a pointed end (a cone's tip).
            if c.radius_top > 0.0 || y != 0 {
                indices.extend_from_slice(&[a, b, d]);
            }
            if c.radius_bottom > 0.0 || y != hs - 1 {
                indices.extend_from_slice(&[b, cc, d]);
            }
        }
    }
    if !c.open_ended {
        for top in [true, false] {
            let radius = if top { c.radius_top } else { c.radius_bottom };
            if radius <= 0.0 {
                continue;
            }
            let sign = if top { 1.0 } else { -1.0 };
            let n = Vec3::new(0.0, sign, 0.0);
            let centre = vertices.len() as u32;
            // One centre per segment (three.js), so each wedge's uv is its own.
            for _ in 0..rs {
                vertices.push(vertex(Vec3::new(0.0, half * sign, 0.0), n, uv(0.5, 0.5)));
            }
            let ring = vertices.len() as u32;
            for x in 0..=rs {
                let u = x as f32 / rs as f32;
                let theta = u * c.theta_length + c.theta_start;
                let (s, co) = theta.sin_cos();
                vertices.push(vertex(
                    Vec3::new(radius * s, half * sign, radius * co),
                    n,
                    uv(co * 0.5 + 0.5, s * 0.5 * sign + 0.5),
                ));
            }
            for x in 0..rs {
                let (ci, i) = (centre + x, ring + x);
                if top {
                    indices.extend_from_slice(&[i, i + 1, ci]);
                } else {
                    indices.extend_from_slice(&[ci, i + 1, i]);
                }
            }
        }
    }
    MeshData { vertices, indices }
}

/// three.js `ConeGeometry(radius, height, radialSegments, heightSegments,
/// openEnded, thetaStart, thetaLength)`: a cylinder whose top radius is 0.
pub fn cone(radius: f32, height: f32, radial: u32, rows: u32, open: bool) -> MeshData {
    cylinder(&Cylinder {
        radius_top: 0.0,
        radius_bottom: radius,
        height,
        radial_segments: radial,
        height_segments: rows,
        open_ended: open,
        ..Cylinder::default()
    })
}

/// three.js `CapsuleGeometry(radius, height, capSegments, radialSegments,
/// heightSegments)`: a cylinder of `height` (the straight part, as
/// three.js's `height`) between two hemispheres of `radius`, so its whole
/// extent along +Y is `height + 2 radius`; v runs by arc length from the
/// bottom pole to the top one (three.js r163's lathe of the profile).
pub fn capsule(radius: f32, height: f32, cap: u32, radial: u32, rows: u32) -> MeshData {
    let cap = cap.max(1);
    let radial = radial.max(3);
    let rows = rows.max(1);
    let half = height / 2.0;
    // The profile from the bottom pole up: (x, y, normal angle from -Y).
    let mut profile: Vec<(Vec2, Vec2)> = Vec::new();
    for i in 0..=cap {
        let a = -PI / 2.0 + (i as f32 / cap as f32) * (PI / 2.0);
        let n = Vec2::new(a.cos(), a.sin());
        profile.push((Vec2::new(n.x * radius, n.y * radius - half), n));
    }
    for i in 1..=rows {
        let y = -half + height * i as f32 / rows as f32;
        profile.push((Vec2::new(radius, y), Vec2::X));
    }
    for i in 1..=cap {
        let a = (i as f32 / cap as f32) * (PI / 2.0);
        let n = Vec2::new(a.cos(), a.sin());
        profile.push((Vec2::new(n.x * radius, n.y * radius + half), n));
    }
    // v by arc length along the profile.
    let mut lengths = vec![0.0f32];
    for w in profile.windows(2) {
        let l = lengths.last().copied().unwrap_or(0.0) + (w[1].0 - w[0].0).length();
        lengths.push(l);
    }
    let total = lengths.last().copied().unwrap_or(1.0).max(1e-12);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for (k, &(p, n)) in profile.iter().enumerate() {
        let v = lengths[k] / total;
        for x in 0..=radial {
            let u = x as f32 / radial as f32;
            let theta = u * TAU;
            let (s, c) = theta.sin_cos();
            vertices.push(vertex(
                Vec3::new(p.x * s, p.y, p.x * c),
                Vec3::new(n.x * s, n.y, n.x * c),
                uv(u, v),
            ));
        }
    }
    let row = radial + 1;
    for k in 0..profile.len() as u32 - 1 {
        for x in 0..radial {
            let a = k * row + x;
            let b = a + row;
            // Counter-clockwise from outside: the profile rises, u turns from +Z to +X.
            indices.extend_from_slice(&[a, a + 1, b, a + 1, b + 1, b]);
        }
    }
    MeshData { vertices, indices }
}

/// three.js `TorusGeometry(radius, tube, radialSegments, tubularSegments,
/// arc)`: in the xy plane around +Z, `radius` from the centre to the
/// tube's centre.
pub fn torus(radius: f32, tube: f32, radial: u32, tubular: u32, arc: f32) -> MeshData {
    let radial = radial.max(2);
    let tubular = tubular.max(3);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for j in 0..=radial {
        for i in 0..=tubular {
            let u = i as f32 / tubular as f32 * arc;
            let v = j as f32 / radial as f32 * TAU;
            let p = Vec3::new(
                (radius + tube * v.cos()) * u.cos(),
                (radius + tube * v.cos()) * u.sin(),
                tube * v.sin(),
            );
            let centre = Vec3::new(radius * u.cos(), radius * u.sin(), 0.0);
            vertices.push(vertex(
                p,
                p - centre,
                uv(i as f32 / tubular as f32, j as f32 / radial as f32),
            ));
        }
    }
    let row = tubular + 1;
    for j in 1..=radial {
        for i in 1..=tubular {
            let a = row * j + i - 1;
            let b = row * (j - 1) + i - 1;
            let c = row * (j - 1) + i;
            let d = row * j + i;
            indices.extend_from_slice(&[a, b, d, b, c, d]);
        }
    }
    MeshData { vertices, indices }
}

/// three.js `RingGeometry(innerRadius, outerRadius, thetaSegments,
/// phiSegments, thetaStart, thetaLength)`: a flat ring in the xy plane
/// facing +Z, uvs mapped from the outer square.
pub fn ring(
    inner: f32,
    outer: f32,
    theta_segments: u32,
    phi_segments: u32,
    start: f32,
    length: f32,
) -> MeshData {
    let ts = theta_segments.max(3);
    let ps = phi_segments.max(1);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let step = (outer - inner) / ps as f32;
    for j in 0..=ps {
        let r = inner + step * j as f32;
        for i in 0..=ts {
            let a = start + i as f32 / ts as f32 * length;
            let (s, c) = a.sin_cos();
            let p = Vec3::new(r * c, r * s, 0.0);
            let o = outer.max(1e-12);
            vertices.push(vertex(
                p,
                Vec3::Z,
                uv(p.x / o * 0.5 + 0.5, p.y / o * 0.5 + 0.5),
            ));
        }
    }
    let row = ts + 1;
    for j in 0..ps {
        for i in 0..ts {
            let a = j * row + i;
            let b = a + row;
            indices.extend_from_slice(&[a, b, b + 1, a, b + 1, a + 1]);
        }
    }
    MeshData { vertices, indices }
}

/// three.js `CircleGeometry(radius, segments, thetaStart, thetaLength)`: a
/// disc (or a sector) in the xy plane facing +Z.
pub fn circle(radius: f32, segments: u32, start: f32, length: f32) -> MeshData {
    let seg = segments.max(3);
    let mut vertices = vec![vertex(Vec3::ZERO, Vec3::Z, uv(0.5, 0.5))];
    let mut indices = Vec::new();
    for i in 0..=seg {
        let a = start + i as f32 / seg as f32 * length;
        let (s, c) = a.sin_cos();
        vertices.push(vertex(
            Vec3::new(radius * c, radius * s, 0.0),
            Vec3::Z,
            uv(c * 0.5 + 0.5, s * 0.5 + 0.5),
        ));
    }
    for i in 1..=seg {
        indices.extend_from_slice(&[i, i + 1, 0]);
    }
    MeshData { vertices, indices }
}

/// A centripetal Catmull-Rom curve through `points` (three.js
/// `CatmullRomCurve3` with `curveType` `centripetal`), sampled by its
/// parameter t in 0..1 (`closed`: the last point joins the first).
pub struct CatmullRom<'a> {
    pub points: &'a [Vec3],
    pub closed: bool,
}

impl CatmullRom<'_> {
    pub fn point(&self, t: f32) -> Vec3 {
        let n = self.points.len();
        if n == 0 {
            return Vec3::ZERO;
        }
        if n == 1 {
            return self.points[0];
        }
        let l = n as f32;
        let p = (l - if self.closed { 0.0 } else { 1.0 }) * t.clamp(0.0, 1.0);
        let mut i = p.floor() as isize;
        let mut w = p - i as f32;
        if !self.closed && w == 0.0 && i as usize == n - 1 {
            i = n as isize - 2;
            w = 1.0;
        }
        let at = |k: isize| -> Vec3 { self.points[k.rem_euclid(n as isize) as usize] };
        let p1 = at(i);
        let p2 = at(i + 1);
        let p0 = if self.closed || i > 0 {
            at(i - 1)
        } else {
            p1 * 2.0 - p2
        };
        let p3 = if self.closed || ((i + 2) as usize) < n {
            at(i + 2)
        } else {
            p2 * 2.0 - p1
        };
        // Centripetal: knots by the square root of the chord lengths.
        let mut dt0 = p0.distance_squared(p1).powf(0.25);
        let mut dt1 = p1.distance_squared(p2).powf(0.25);
        let mut dt2 = p2.distance_squared(p3).powf(0.25);
        if dt1 < 1e-4 {
            dt1 = 1.0;
        }
        if dt0 < 1e-4 {
            dt0 = dt1;
        }
        if dt2 < 1e-4 {
            dt2 = dt1;
        }
        let comp = |x0: f32, x1: f32, x2: f32, x3: f32| -> f32 {
            let mut t1 = (x1 - x0) / dt0 - (x2 - x0) / (dt0 + dt1) + (x2 - x1) / dt1;
            let mut t2 = (x2 - x1) / dt1 - (x3 - x1) / (dt1 + dt2) + (x3 - x2) / dt2;
            t1 *= dt1;
            t2 *= dt1;
            let (c0, c1) = (x1, t1);
            let c2 = -3.0 * x1 + 3.0 * x2 - 2.0 * t1 - t2;
            let c3 = 2.0 * x1 - 2.0 * x2 + t1 + t2;
            c0 + c1 * w + c2 * w * w + c3 * w * w * w
        };
        Vec3::new(
            comp(p0.x, p1.x, p2.x, p3.x),
            comp(p0.y, p1.y, p2.y, p3.y),
            comp(p0.z, p1.z, p2.z, p3.z),
        )
    }

    /// The unit tangent at t (a central difference, as three.js `getTangent`).
    pub fn tangent(&self, t: f32) -> Vec3 {
        let d = 1e-4;
        let (a, b) = ((t - d).max(0.0), (t + d).min(1.0));
        (self.point(b) - self.point(a)).normalize_or(Vec3::Z)
    }

    /// The cumulative chord lengths at `divisions + 1` even steps of t
    /// (three.js `getLengths`, 200 divisions by default).
    pub fn lengths(&self, divisions: usize) -> Vec<f32> {
        let mut out = Vec::with_capacity(divisions + 1);
        let mut last = self.point(0.0);
        let mut sum = 0.0;
        out.push(0.0);
        for k in 1..=divisions {
            let p = self.point(k as f32 / divisions as f32);
            sum += p.distance(last);
            out.push(sum);
            last = p;
        }
        out
    }
}

/// three.js `getUtoTmapping`: the curve parameter t at which a share `u`
/// of the length (`lengths` from [`CatmullRom::lengths`]) is reached.
pub fn u_to_t(lengths: &[f32], u: f32) -> f32 {
    let n = lengths.len();
    if n < 2 {
        return u;
    }
    let target = u * lengths[n - 1];
    // The last sample below the target (a binary search, as three.js's).
    let (mut lo, mut hi) = (0isize, n as isize - 1);
    while lo <= hi {
        let i = lo + (hi - lo) / 2;
        let c = lengths[i as usize] - target;
        if c < 0.0 {
            lo = i + 1;
        } else if c > 0.0 {
            hi = i - 1;
        } else {
            hi = i;
            break;
        }
    }
    let i = hi.max(0) as usize;
    if lengths[i] == target || i + 1 >= n {
        return i as f32 / (n - 1) as f32;
    }
    let (before, after) = (lengths[i], lengths[i + 1]);
    let fraction = if after > before {
        (target - before) / (after - before)
    } else {
        0.0
    };
    (i as f32 + fraction) / (n - 1) as f32
}

/// three.js `TubeGeometry(path, tubularSegments, radius, radialSegments,
/// closed)` along a centripetal Catmull-Rom curve through `points`, sampled
/// by arc length as three.js samples it (`getPointAt`, `getTangentAt`).
/// Fewer than two points is an empty mesh.
pub fn tube(points: &[Vec3], tubular: u32, radius: f32, radial: u32, closed: bool) -> MeshData {
    if points.len() < 2 {
        return MeshData::default();
    }
    let curve = CatmullRom { points, closed };
    let lengths = curve.lengths(200);
    tube_along(
        |u| {
            let t = u_to_t(&lengths, u);
            (curve.point(t), curve.tangent(t))
        },
        tubular,
        radius,
        radial,
        closed,
    )
}

/// A tube along any curve: `at(u)` answers the point and its unit tangent
/// at a share u (0 to 1) of the curve's length (an og.path's `sample`, a
/// curve's `getPointAt` and `getTangentAt`). Rings of `radial` vertices
/// oriented by parallel-transported frames (three.js
/// `computeFrenetFrames`), u along the tube and v around it; `closed`
/// joins the last ring to the first (the curve's end is its start).
pub fn tube_along(
    at: impl Fn(f32) -> (Vec3, Vec3),
    tubular: u32,
    radius: f32,
    radial: u32,
    closed: bool,
) -> MeshData {
    let segs = tubular.max(1);
    let radial = radial.max(3);
    let ts: Vec<f32> = (0..=segs).map(|i| i as f32 / segs as f32).collect();
    let samples: Vec<(Vec3, Vec3)> = ts.iter().map(|&u| at(u)).collect();
    let tangents: Vec<Vec3> = samples.iter().map(|s| s.1.normalize_or(Vec3::Z)).collect();
    // The first normal: across the tangent's smallest component (three.js).
    let t0 = tangents[0];
    let a = t0.abs();
    let seed = if a.x <= a.y && a.x <= a.z {
        Vec3::X
    } else if a.y <= a.z {
        Vec3::Y
    } else {
        Vec3::Z
    };
    let mut normals = vec![t0.cross(seed).normalize_or(Vec3::Y)];
    normals[0] = t0.cross(normals[0]).normalize_or(Vec3::Y);
    for i in 1..ts.len() {
        let mut n = normals[i - 1];
        let axis = tangents[i - 1].cross(tangents[i]);
        if axis.length() > f32::EPSILON {
            let angle = tangents[i - 1].dot(tangents[i]).clamp(-1.0, 1.0).acos();
            n = glam::Quat::from_axis_angle(axis.normalize(), angle) * n;
        }
        normals.push(n);
    }
    if closed {
        // Spread the twist between the ends over the length, so the seam meets.
        let last = normals.len() - 1;
        let mut theta = normals[0].dot(normals[last]).clamp(-1.0, 1.0).acos() / segs as f32;
        if tangents[0].dot(normals[0].cross(normals[last])) > 0.0 {
            theta = -theta;
        }
        for (i, n) in normals.iter_mut().enumerate().skip(1) {
            *n = glam::Quat::from_axis_angle(tangents[i], theta * i as f32) * *n;
        }
    }
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for i in 0..ts.len() {
        // A closed tube's last ring is its first again (three.js).
        let p = samples[if closed && i == segs as usize { 0 } else { i }].0;
        let (n, b) = {
            let k = if closed && i == segs as usize { 0 } else { i };
            let n = normals[k];
            (n, tangents[k].cross(n).normalize_or(Vec3::X))
        };
        for j in 0..=radial {
            let v = j as f32 / radial as f32 * TAU;
            let (s, c) = v.sin_cos();
            let dir = (n * -c + b * s).normalize_or(Vec3::Y);
            vertices.push(vertex(
                p + dir * radius,
                dir,
                uv(i as f32 / segs as f32, j as f32 / radial as f32),
            ));
        }
    }
    let row = radial + 1;
    for j in 1..=segs {
        for i in 1..=radial {
            let a = row * (j - 1) + (i - 1);
            let b = row * j + (i - 1);
            let c = row * j + i;
            let d = row * (j - 1) + i;
            indices.extend_from_slice(&[a, b, d, b, c, d]);
        }
    }
    MeshData { vertices, indices }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every triangle faces away from the solid's axis (its normal agrees
    /// with the vertex normals): front faces outside.
    fn faces_out(m: &MeshData) {
        for t in m.indices.chunks(3) {
            let [a, b, c] = [t[0], t[1], t[2]].map(|i| Vec3::from(m.vertices[i as usize].position));
            let face = (b - a).cross(c - a);
            if face.length() < 1e-8 {
                continue;
            }
            let n: Vec3 = t
                .iter()
                .map(|&i| Vec3::from(m.vertices[i as usize].normal))
                .sum();
            assert!(face.dot(n) > 0.0, "a triangle faces inwards: {a} {b} {c}");
        }
    }

    fn bounds(m: &MeshData) -> (Vec3, Vec3) {
        m.bounds().expect("vertices")
    }

    #[test]
    fn cylinder_matches_three_js_counts_and_extent() {
        let m = cylinder(&Cylinder::default());
        // three.js: (32 + 1) * 2 side vertices, then per cap 32 centres + 33 ring.
        assert_eq!(m.vertices.len(), 33 * 2 + 2 * (32 + 33));
        assert_eq!(m.indices.len() / 3, 32 * 2 + 32 * 2);
        let (lo, hi) = bounds(&m);
        assert!((hi.y - 0.5).abs() < 1e-6 && (lo.y + 0.5).abs() < 1e-6);
        assert!((hi.x - 1.0).abs() < 1e-3 && (hi.z - 1.0).abs() < 1e-6);
        faces_out(&m);
    }

    #[test]
    fn cone_has_no_top_cap_and_faces_out() {
        let m = cone(1.0, 2.0, 16, 1, false);
        // The side's top row is the tip: its triangles are skipped there.
        assert_eq!(m.indices.len() / 3, 16 + 16);
        faces_out(&m);
        let (lo, hi) = bounds(&m);
        assert!((hi.y - 1.0).abs() < 1e-6 && (lo.y + 1.0).abs() < 1e-6);
    }

    #[test]
    fn capsule_extent_is_height_plus_two_radii() {
        let m = capsule(0.5, 2.0, 4, 8, 1);
        let (lo, hi) = bounds(&m);
        assert!(
            (hi.y - 1.5).abs() < 1e-5 && (lo.y + 1.5).abs() < 1e-5,
            "{lo} {hi}"
        );
        faces_out(&m);
    }

    #[test]
    fn torus_ring_circle_face_their_way() {
        let t = torus(1.0, 0.4, 12, 48, TAU);
        assert_eq!(t.vertices.len(), 13 * 49);
        faces_out(&t);
        let (lo, hi) = bounds(&t);
        assert!((hi.x - 1.4).abs() < 1e-4 && (hi.z - 0.4).abs() < 1e-4 && lo.z < -0.39);
        let r = ring(0.5, 1.0, 32, 1, 0.0, TAU);
        faces_out(&r);
        let c = circle(1.0, 32, 0.0, TAU);
        assert_eq!(c.vertices.len(), 34);
        faces_out(&c);
    }

    #[test]
    fn catmull_rom_passes_through_its_points() {
        let pts = [
            Vec3::ZERO,
            Vec3::new(1.0, 2.0, 0.0),
            Vec3::new(3.0, 0.0, 1.0),
            Vec3::new(4.0, 1.0, -2.0),
        ];
        let c = CatmullRom {
            points: &pts,
            closed: false,
        };
        for (k, p) in pts.iter().enumerate() {
            let at = c.point(k as f32 / 3.0);
            assert!(at.distance(*p) < 1e-4, "point {k}: {at} vs {p}");
        }
    }

    #[test]
    fn a_tube_keeps_its_radius_from_the_curve() {
        let pts = [
            Vec3::ZERO,
            Vec3::new(2.0, 1.0, 0.0),
            Vec3::new(4.0, 0.0, 2.0),
        ];
        let m = tube(&pts, 32, 0.25, 8, false);
        assert_eq!(m.vertices.len(), 33 * 9);
        let c = CatmullRom {
            points: &pts,
            closed: false,
        };
        let lengths = c.lengths(200);
        for (i, ring) in m.vertices.chunks(9).enumerate() {
            let centre = c.point(u_to_t(&lengths, i as f32 / 32.0));
            for v in ring {
                let d = Vec3::from(v.position).distance(centre);
                assert!((d - 0.25).abs() < 1e-3, "ring {i}: {d}");
            }
        }
        faces_out(&m);
        // Rings by arc length (three.js getPointAt): the gaps between ring
        // centres are even, though the points' spacing is not.
        let centres: Vec<Vec3> = m
            .vertices
            .chunks(9)
            .map(|r| r[..8].iter().map(|v| Vec3::from(v.position)).sum::<Vec3>() / 8.0)
            .collect();
        let gaps: Vec<f32> = centres.windows(2).map(|w| w[0].distance(w[1])).collect();
        let (lo, hi) = gaps
            .iter()
            .fold((f32::MAX, 0.0f32), |(a, b), &g| (a.min(g), b.max(g)));
        assert!(hi / lo < 1.02, "ring gaps {lo} to {hi}");
    }

    #[test]
    fn u_to_t_inverts_the_length_table() {
        let lengths = [0.0, 1.0, 3.0, 4.0];
        assert_eq!(u_to_t(&lengths, 0.0), 0.0);
        assert_eq!(u_to_t(&lengths, 1.0), 1.0);
        // Half the length (2) is half way through the second step.
        assert!((u_to_t(&lengths, 0.5) - 0.5).abs() < 1e-6);
        // A quarter (1) is the end of the first step exactly.
        assert!((u_to_t(&lengths, 0.25) - 1.0 / 3.0).abs() < 1e-6);
        assert!((u_to_t(&lengths, 0.125) - 1.0 / 6.0).abs() < 1e-6);
    }
}
