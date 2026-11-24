//! Plane clipping and signed tetrahedron integration. All coordinates are
//! relative to the hull centre to retain precision after origin shifts.
use og_core::glam::Vec3;

/// Sutherland-Hodgman clipping, preserving the input's winding.
fn plane(poly: &[Vec3], axis: usize, limit: f32, sign: f32) -> Vec<Vec3> {
    let mut out = Vec::new();
    let Some(&last) = poly.last() else { return out };
    let mut a = last;
    let mut da = (a[axis] - limit) * sign;
    for &b in poly {
        let db = (b[axis] - limit) * sign;
        if (da <= 0.0) != (db <= 0.0) {
            let mut p = a + (b - a) * (da / (da - db));
            p[axis] = limit;
            out.push(p);
        }
        if db <= 0.0 {
            out.push(b);
        }
        a = b;
        da = db;
    }
    out
}

/// Close each clipped mesh with reverse-wound cut edges. The cap's fan
/// centre need only lie in the plane: signed integration cancels overlaps,
/// including those between disconnected collider parts.
pub(crate) fn bounded(triangles: &[[Vec3; 3]], min: Vec3, max: Vec3) -> Vec<[Vec3; 3]> {
    let mut mesh = triangles.to_vec();
    for (axis, limit, sign) in [
        (0, min.x, -1.0),
        (0, max.x, 1.0),
        (2, min.z, -1.0),
        (2, max.z, 1.0),
    ] {
        let mut next = Vec::new();
        let mut edges = Vec::new();
        for triangle in &mesh {
            let polygon = plane(triangle, axis, limit, sign);
            for i in 1..polygon.len().saturating_sub(1) {
                next.push([polygon[0], polygon[i], polygon[i + 1]]);
            }
            for i in 0..polygon.len() {
                let a = polygon[i];
                let b = polygon[(i + 1) % polygon.len()];
                if a[axis] == limit && b[axis] == limit {
                    edges.push((a, b));
                }
            }
        }
        if !edges.is_empty() {
            let mut centre = Vec3::ZERO;
            for &(a, b) in &edges {
                centre += a + b;
            }
            centre /= 2.0 * edges.len() as f32;
            centre[axis] = limit;
            for (a, b) in edges {
                next.push([b, a, centre]);
            }
        }
        mesh = next;
    }
    mesh
}

/// Volume under a level, with its first moment and waterplane area. The
/// missing cap lies in the integration origin's plane, so contributes
/// zero signed tetrahedron volume. No convex-hull rebuild or trig sort.
pub(crate) fn under(triangles: &[[Vec3; 3]], level: f32) -> (f32, Vec3, f32) {
    let origin = Vec3::new(0.0, level, 0.0);
    let (mut volume, mut moment, mut area) = (0.0, Vec3::ZERO, 0.0);
    for triangle in triangles {
        let polygon = plane(triangle, 1, level, 1.0);
        for i in 1..polygon.len().saturating_sub(1) {
            let a = polygon[0] - origin;
            let b = polygon[i] - origin;
            let c = polygon[i + 1] - origin;
            let v = a.dot(b.cross(c)) / 6.0;
            volume += v;
            moment += (origin + (a + b + c) * 0.25) * v;
        }
        // The cut edges bound the horizontal cap. Their reverse winding
        // supplies its signed xz area; absolute value handles either mesh winding.
        for i in 0..polygon.len() {
            let a = polygon[i];
            let b = polygon[(i + 1) % polygon.len()];
            if a.y == level && b.y == level {
                area += (a.x * b.z - b.x * a.z) * 0.5;
            }
        }
    }
    if volume < 0.0 {
        (-volume, -moment, area.abs())
    } else {
        (volume, moment, area.abs())
    }
}

/// A hull's displaced volume and centre in a finite-depth pool.
pub(crate) fn immersed(triangles: &[[Vec3; 3]], level: f32, depth: f32) -> (f32, Vec3, f32) {
    let (top, top_m, area) = under(triangles, level);
    let (bottom, bottom_m, _) = under(triangles, level - depth);
    let volume = (top - bottom).max(0.0);
    let centre = if volume > 0.0 {
        (top_m - bottom_m) / volume
    } else {
        Vec3::ZERO
    };
    (volume, centre, area)
}
