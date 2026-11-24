//! Profile constructors. Each takes a zero-value desc struct; the shapes
//! are centred on the origin unless their points say otherwise.

use crate::math::DVec2;

use og_core::math::double;

use crate::profile::curve::{arc, segments};
use crate::profile::{Contour, Profile, ProfileError};

fn positive(v: f64, name: &'static str) -> Result<f64, ProfileError> {
    if !v.is_finite() {
        Err(ProfileError::NotFinite(name))
    } else if v <= 0.0 {
        Err(ProfileError::NotPositive(name))
    } else {
        Ok(v)
    }
}

/// A rectangle, optionally with rounded corners.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    /// Size along x.
    pub width: f64,
    /// Size along y.
    pub height: f64,
    /// Corner radius; 0 for sharp corners. At most half the smaller side.
    pub radius: f64,
}

/// A rectangle centred on the origin.
pub fn rect(d: &Rect) -> Result<Profile, ProfileError> {
    let (w, h) = (
        positive(d.width, "width")? / 2.0,
        positive(d.height, "height")? / 2.0,
    );
    if d.radius == 0.0 {
        let p = vec![
            DVec2::new(-w, -h),
            DVec2::new(w, -h),
            DVec2::new(w, h),
            DVec2::new(-w, h),
        ];
        return Ok(Profile::from_contour(Contour::corners(p)));
    }
    let r = positive(d.radius, "radius")?;
    if r > w.min(h) {
        return Err(ProfileError::RadiusTooLarge(0));
    }
    let n = segments(r, double::PI / 2.0);
    let mut points = Vec::new();
    let corners = [
        (w - r, -h + r, -double::PI / 2.0),
        (w - r, h - r, 0.0),
        (-w + r, h - r, double::PI / 2.0),
        (-w + r, -h + r, double::PI),
    ];
    for (cx, cy, start) in corners {
        let mut a = arc(DVec2::new(cx, cy), r, start, double::PI / 2.0, n);
        // Where two quarter arcs meet with no straight between, keep one.
        if let (Some(last), Some(first)) = (points.last(), a.first())
            && *last == *first
        {
            a.remove(0);
        }
        points.extend(a);
    }
    if points.first() == points.last() {
        points.pop();
    }
    Ok(Profile::from_contour(Contour::smooth(points)))
}

/// A circle.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Circle {
    /// The radius.
    pub radius: f64,
    /// Segments; 0 for the chord rule of [`crate::profile::curve::segments`].
    pub segments: u32,
}

/// A circle centred on the origin, its first point on +x.
pub fn circle(d: &Circle) -> Result<Profile, ProfileError> {
    let r = positive(d.radius, "radius")?;
    let n = if d.segments >= 3 {
        d.segments
    } else {
        segments(r, double::TAU)
    };
    let mut p = arc(DVec2::ZERO, r, 0.0, double::TAU, n);
    p.pop();
    Ok(Profile::from_contour(Contour::smooth(p)))
}

/// An ellipse.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ellipse {
    /// Semi-axis along x.
    pub rx: f64,
    /// Semi-axis along y.
    pub ry: f64,
    /// Segments; 0 for the chord rule at the larger radius.
    pub segments: u32,
}

/// An ellipse centred on the origin.
pub fn ellipse(d: &Ellipse) -> Result<Profile, ProfileError> {
    let (rx, ry) = (positive(d.rx, "rx")?, positive(d.ry, "ry")?);
    let n = if d.segments >= 3 {
        d.segments
    } else {
        segments(rx.max(ry), double::TAU)
    };
    let p = (0..n)
        .map(|i| {
            let (s, c) = double::sin_cos(double::TAU * (i as f64 / n as f64));
            DVec2::new(rx * c, ry * s)
        })
        .collect();
    Ok(Profile::from_contour(Contour::smooth(p)))
}

/// A slot: a rectangle with semicircular ends, along x.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Slot {
    /// Overall length, end to end; at least `width`.
    pub length: f64,
    /// Width (the ends' diameter).
    pub width: f64,
}

/// A slot centred on the origin.
pub fn slot(d: &Slot) -> Result<Profile, ProfileError> {
    let w = positive(d.width, "width")?;
    let l = positive(d.length, "length")?;
    if l < w {
        return Err(ProfileError::RadiusTooLarge(0));
    }
    let r = w / 2.0;
    let half = (l - w) / 2.0;
    if half == 0.0 {
        return circle(&Circle {
            radius: r,
            segments: 0,
        });
    }
    let n = segments(r, double::PI);
    let mut p = arc(DVec2::new(half, 0.0), r, -double::PI / 2.0, double::PI, n);
    p.extend(arc(
        DVec2::new(-half, 0.0),
        r,
        double::PI / 2.0,
        double::PI,
        n,
    ));
    Ok(Profile::from_contour(Contour::smooth(p)))
}

/// A regular polygon.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Polygon {
    /// Number of sides, at least 3.
    pub sides: u32,
    /// Distance from the centre to each corner.
    pub radius: f64,
    /// Distance across flats (twice the centre-to-side distance), a nut's
    /// or a hex bar's size; 0 gives the size by `radius` instead. Give
    /// one of the two.
    pub flats: f64,
}

/// A regular polygon centred on the origin, a corner on +x.
pub fn polygon(d: &Polygon) -> Result<Profile, ProfileError> {
    if d.sides < 3 {
        return Err(ProfileError::TooFewPoints(d.sides as usize));
    }
    let r = if d.flats == 0.0 {
        positive(d.radius, "radius")?
    } else {
        if d.radius != 0.0 {
            return Err(ProfileError::Conflict("radius and flats"));
        }
        // The apothem is flats / 2 = r cos(π / n).
        positive(d.flats, "flats")? / 2.0 / double::cos(double::PI / d.sides as f64)
    };
    let mut p = arc(DVec2::ZERO, r, 0.0, double::TAU, d.sides);
    p.pop();
    Ok(Profile::from_contour(Contour::corners(p)))
}

/// A point of a [`Poly`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PolyPoint {
    /// x.
    pub x: f64,
    /// y.
    pub y: f64,
    /// Corner radius; 0 for a sharp corner.
    pub radius: f64,
    /// The edge from this point to the next is a circular arc whose
    /// middle stands `bulge` half-chords away from the straight edge (the
    /// polyline bulge of DXF and CAD, tan of a quarter of the arc's angle):
    /// positive bows out of the outline, negative into it; 0 is straight,
    /// 1 is a half circle, and 0.3 on a 370 mm edge rises 55.5 mm.
    pub bulge: f64,
}

/// A polygon through given points, each corner sharp or rounded.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Poly {
    /// The corners, in order (either winding).
    pub points: Vec<PolyPoint>,
}

/// A polygon with per-corner fillets. A fillet is tangent to both edges;
/// two fillets on one edge must fit along it.
pub fn poly(d: &Poly) -> Result<Profile, ProfileError> {
    let n = d.points.len();
    if n < 3 {
        return Err(ProfileError::TooFewPoints(n));
    }
    for p in &d.points {
        if !(p.x.is_finite() && p.y.is_finite() && p.radius.is_finite() && p.bulge.is_finite()) {
            return Err(ProfileError::NotFinite("points"));
        }
    }
    // Wind counter-clockwise first, so every convex corner turns left.
    // `bulge[i]` belongs to the edge from point i to point i + 1; on the
    // reversed list that edge runs from n − 2 − i to n − 1 − i.
    let mut pts = d.points.clone();
    let mut bulge: Vec<f64> = pts.iter().map(|p| p.bulge).collect();
    let area2: f64 = (0..n)
        .map(|i| pts[i].x * pts[(i + 1) % n].y - pts[i].y * pts[(i + 1) % n].x)
        .sum();
    if area2 < 0.0 {
        pts.reverse();
        bulge = (0..n)
            .map(|k| d.points[(2 * n - 2 - k) % n].bulge)
            .collect();
    }
    let at = |i: usize| DVec2::new(pts[i % n].x, pts[i % n].y);
    // The distance from each corner to its tangent points.
    let mut trim = vec![0.0; n];
    for i in 0..n {
        let r = pts[i].radius;
        if r <= 0.0 {
            continue;
        }
        if bulge[i] != 0.0 || bulge[(i + n - 1) % n] != 0.0 {
            return Err(ProfileError::ArcCorner(i));
        }
        let (prev, p, next) = (at(i + n - 1), at(i), at(i + 1));
        let (u, v) = (
            (prev - p).normalize_or_zero(),
            (next - p).normalize_or_zero(),
        );
        let cos = u.dot(v).clamp(-1.0, 1.0);
        let half = double::acos(cos) / 2.0;
        if half <= 0.0 || half >= double::PI / 2.0 {
            return Err(ProfileError::RadiusTooLarge(i));
        }
        trim[i] = r / double::tan(half);
    }
    for i in 0..n {
        let edge = (at(i + 1) - at(i)).length();
        if trim[i] + trim[(i + 1) % n] > edge {
            return Err(ProfileError::RadiusTooLarge(i));
        }
    }
    let mut c = Contour::default();
    for i in 0..n {
        let (prev, p, next) = (at(i + n - 1), at(i), at(i + 1));
        if trim[i] == 0.0 {
            c.points.push(p);
            c.sharp.push(true);
            if bulge[i] != 0.0 {
                arc_edge(&mut c, p, next, bulge[i])?;
            }
            continue;
        }
        let (u, v) = ((prev - p).normalize(), (next - p).normalize());
        let (t0, t1) = (p + u * trim[i], p + v * trim[i]);
        let r = pts[i].radius;
        // The centre lies along the bisector, r / sin(half) from the corner.
        let bis = (u + v).normalize();
        let half = double::acos(u.dot(v).clamp(-1.0, 1.0)) / 2.0;
        let center = p + bis * (r / double::sin(half));
        let a0 = double::atan2(t0.y - center.y, t0.x - center.x);
        let a1 = double::atan2(t1.y - center.y, t1.x - center.x);
        // A left turn (convex corner) sweeps counter-clockwise.
        let left = (p - prev).perp_dot(next - p) > 0.0;
        let mut sweep = a1 - a0;
        if left && sweep < 0.0 {
            sweep += double::TAU;
        }
        if !left && sweep > 0.0 {
            sweep -= double::TAU;
        }
        let seg = segments(r, sweep);
        for q in arc(center, r, a0, sweep, seg) {
            c.points.push(q);
            c.sharp.push(false);
        }
    }
    Ok(Profile::from_contour(c))
}

/// The points strictly inside an arc edge from `a` to `b` on a
/// counter-clockwise outline, its middle `s` to the right of the chord
/// (out of the outline) or, negative, to the left.
fn arc_edge(c: &mut Contour, a: DVec2, b: DVec2, bulge: f64) -> Result<(), ProfileError> {
    let chord = b - a;
    let len = chord.length();
    if len <= 0.0 {
        return Err(ProfileError::NotPositive("the chord of an arc edge"));
    }
    // The sagitta is the bulge in half-chords. Radius from the chord and
    // the sagitta; the centre lies r − s from the chord's middle, into the
    // outline (left of the travel).
    let half = len / 2.0;
    let s = bulge * half;
    let r = (half * half + s * s) / (2.0 * s.abs());
    let left = DVec2::new(-chord.y, chord.x) / len;
    let center = (a + b) / 2.0 + left * (r - s.abs()) * s.signum();
    // Seen from the centre the arc turns counter-clockwise when it bows
    // out of a counter-clockwise outline: 4 atan(2s / chord), signed.
    let sweep = 4.0 * double::atan2(s, half);
    let a0 = double::atan2(a.y - center.y, a.x - center.x);
    let mut pts = arc(center, r, a0, sweep, segments(r, sweep));
    pts.pop();
    for q in pts.into_iter().skip(1) {
        c.points.push(q);
        c.sharp.push(false);
    }
    Ok(())
}

/// A superellipse: |x/rx|ⁿ + |y/ry|ⁿ = 1.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Superellipse {
    /// Semi-axis along x.
    pub rx: f64,
    /// Semi-axis along y.
    pub ry: f64,
    /// The exponent n: 2 is an ellipse, larger is squarer.
    pub exponent: f64,
    /// Segments; 0 for the chord rule at the larger radius.
    pub segments: u32,
}

/// A superellipse centred on the origin.
pub fn superellipse(d: &Superellipse) -> Result<Profile, ProfileError> {
    let (rx, ry) = (positive(d.rx, "rx")?, positive(d.ry, "ry")?);
    let e = positive(d.exponent, "exponent")?;
    let n = if d.segments >= 4 {
        d.segments
    } else {
        segments(rx.max(ry), double::TAU)
    };
    let p = (0..n)
        .map(|i| {
            let (s, c) = double::sin_cos(double::TAU * (i as f64 / n as f64));
            let f = |v: f64| v.signum() * double::pow(v.abs(), 2.0 / e);
            DVec2::new(rx * f(c), ry * f(s))
        })
        .collect();
    Ok(Profile::from_contour(Contour::smooth(p)))
}

/// A closed smooth curve through points.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Spline {
    /// The points the curve passes through, in order, at least 3; the
    /// curve closes from the last back to the first.
    pub points: Vec<DVec2>,
}

/// A closed Catmull-Rom spline through `d.points` (as cubic Béziers, each
/// span flattened by [`crate::profile::curve::bezier_segments`]). Every point is
/// smooth: no corners.
pub fn spline(d: &Spline) -> Result<Profile, ProfileError> {
    let p = &d.points;
    let n = p.len();
    if n < 3 {
        return Err(ProfileError::TooFewPoints(n));
    }
    if p.iter().any(|q| !q.is_finite()) {
        return Err(ProfileError::NotFinite("points"));
    }
    let mut out = Vec::new();
    for i in 0..n {
        let (p0, p1) = (p[(i + n - 1) % n], p[i]);
        let (p2, p3) = (p[(i + 1) % n], p[(i + 2) % n]);
        let c1 = p1 + (p2 - p0) / 6.0;
        let c2 = p2 - (p3 - p1) / 6.0;
        out.push(p1);
        let mut span = crate::profile::curve::cubic(
            p1,
            c1,
            c2,
            p2,
            crate::profile::curve::bezier_segments(&[p1, c1, c2, p2]),
        );
        span.pop();
        out.extend(span);
    }
    out.dedup();
    let c = Contour::smooth(out);
    if c.points.len() < 3 || c.signed_area2() == 0.0 {
        return Err(ProfileError::Empty);
    }
    Ok(Profile::from_contour(c))
}
