//! A view's camera: where it stands for an azimuth, how it projects, and the parts sorted back to front.

use super::*;

pub(crate) struct Camera {
    pub(crate) centre: DVec3,
    pub(crate) right: DVec3,
    pub(crate) up: DVec3,
    /// Into the screen: depth grows along it.
    pub(crate) forward: DVec3,
    pub(crate) scale: f64,
    pub(crate) width: u32,
    pub(crate) height: u32,
    /// Perspective: the eye and the focal length in pixels.
    pub(crate) eye: Option<(DVec3, f64)>,
}

impl Camera {
    /// Screen position in fixed point, and depth (perspective: the
    /// distance from the eye along the view, always positive).
    pub(crate) fn project(&self, p: DVec3) -> (i64, i64, f64) {
        let (sx, sy, z) = match self.eye {
            None => {
                let d = p - self.centre;
                (
                    f64::from(self.width) * 0.5 + d.dot(self.right) * self.scale,
                    f64::from(self.height) * 0.5 - d.dot(self.up) * self.scale,
                    d.dot(self.forward),
                )
            }
            Some((eye, focal)) => {
                let d = p - eye;
                let z = d.dot(self.forward);
                let k = focal / z;
                (
                    f64::from(self.width) * 0.5 + d.dot(self.right) * k,
                    f64::from(self.height) * 0.5 - d.dot(self.up) * k,
                    z,
                )
            }
        };
        (
            (sx * SUB as f64).round() as i64,
            (sy * SUB as f64).round() as i64,
            z,
        )
    }

    pub(crate) fn corner(&self, p: DVec3, light: f64) -> Corner {
        let (x, y, z) = self.project(p);
        Corner {
            x,
            y,
            z,
            light,
            iz: 1.0 / z,
        }
    }
}

/// The triangles of transparent parts (`0 < opacity < 1`) in drawing
/// order: the sum of the corners' depths, largest (farthest) first; ties
/// to the lower part index, then the lower triangle index. Depths are
/// finite (the scene's positions are checked), so `total_cmp` is the
/// plain order.
pub(crate) fn back_to_front(cam: &Camera, parts: &[Drawn]) -> Vec<(usize, usize)> {
    let mut order: Vec<(f64, usize, usize)> = Vec::new();
    for (i, p) in parts.iter().enumerate() {
        if p.opacity == 1.0 || p.opacity == 0.0 {
            continue;
        }
        for (k, t) in p.tris.iter().enumerate() {
            let z = t.iter().map(|&q| cam.project(q).2).fold(0.0, |a, b| a + b);
            order.push((z, i, k));
        }
    }
    order.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    order.into_iter().map(|(_, i, k)| (i, k)).collect()
}
