//! Edges drawn over the shaded view.

use super::*;

/// Real edges, one pixel wide, where nothing nearer hides them.
pub(crate) fn draw_edges(t: &mut Target, cam: &Camera, edges: &[(DVec3, DVec3)], radius: f64) {
    // A line on a face is as near as the face: allow a little depth.
    let bias = radius * 0.006;
    let (w, h) = (i64::from(t.width), i64::from(t.height));
    for &(a, b) in edges {
        let (ax, ay, az) = cam.project(a);
        let (bx, by, bz) = cam.project(b);
        let (dx, dy) = (bx - ax, by - ay);
        let steps = (dx.abs().max(dy.abs()) / SUB).max(1);
        for i in 0..=steps {
            let x = (ax + dx * i / steps).div_euclid(SUB);
            let y = (ay + dy * i / steps).div_euclid(SUB);
            if x < 0 || y < 0 || x >= w || y >= h {
                continue;
            }
            let k = (y * w + x) as usize;
            let s = i as f64 / steps as f64;
            // In perspective, 1 / depth is what varies linearly on screen.
            let z = if cam.eye.is_some() {
                1.0 / ((1.0 - s) / az + s / bz)
            } else {
                az + (bz - az) * s
            };
            if z - bias <= t.depth[k] {
                t.rgb[k] = scale(t.rgb[k], 2, 5);
            }
        }
    }
}
