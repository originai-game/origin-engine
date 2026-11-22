//! The ground under the scene and the shadow the scene casts on it.

use super::*;

pub(crate) fn draw_ground(
    t: &mut Target,
    cam: &Camera,
    quad: &[DVec3; 4],
    parts: &[Drawn],
    light: DVec3,
    ground_z: f64,
) {
    let persp = cam.eye.is_some();
    let corner = |p: DVec3| cam.corner(p, 0.0);
    let q = quad.map(corner);
    for tri in [[q[0], q[1], q[2]], [q[0], q[2], q[3]]] {
        let (depth, ground) = (&mut t.depth, &mut t.ground);
        let rgb = &mut t.rgb;
        raster((t.width, t.height), tri, persp, false, |f| {
            let i = f.i;
            if f.z < depth[i] {
                depth[i] = f.z;
                ground[i] = true;
                rgb[i] = GROUND;
            }
        });
    }
    // The shadow: every triangle pressed onto the ground along the light,
    // as dark as the most opaque part casting it there. In perspective it
    // is clipped to the ground square first: a shadow point past it can
    // lie behind the eye.
    let mut shade = vec![0.0f64; t.rgb.len()];
    let dims = (t.width, t.height);
    for p in parts.iter().filter(|p| p.opacity > 0.0) {
        for tri in &p.tris {
            let pressed = tri.map(|q| q - light * ((q.z - ground_z) / light.z));
            let mut cover = |c: [Corner; 3]| {
                raster(dims, c, persp, false, |f| {
                    let i = f.i;
                    if t.ground[i] && p.opacity > shade[i] {
                        shade[i] = p.opacity;
                    }
                });
            };
            if persp {
                let poly = clip_square(&pressed, quad[0], quad[2]);
                for k in 2..poly.len() {
                    cover([poly[0], poly[k - 1], poly[k]].map(corner));
                }
            } else {
                cover(pressed.map(corner));
            }
        }
    }
    for (px, &s) in t.rgb.iter_mut().zip(&shade) {
        if s == 1.0 {
            *px = SHADOW;
        } else if s > 0.0 {
            *px = [0, 1, 2]
                .map(|k| byte((f64::from(px[k]) * (1.0 - s) + f64::from(SHADOW[k]) * s) / 255.0));
        }
    }
}
