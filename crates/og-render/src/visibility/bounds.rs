//! World bounds of what is drawn.

use glam::{Mat4, Vec3};

/// The world bounding sphere (centre, radius) of a box in the space
/// `model` places.
pub(crate) fn world_sphere(model: Mat4, lo: Vec3, hi: Vec3) -> (Vec3, f32) {
    let center = model.transform_point3((lo + hi) * 0.5);
    let mut radius: f32 = 0.0;
    for i in 0..8 {
        let corner = Vec3::new(
            if i & 1 == 0 { lo.x } else { hi.x },
            if i & 2 == 0 { lo.y } else { hi.y },
            if i & 4 == 0 { lo.z } else { hi.z },
        );
        radius = radius.max(model.transform_point3(corner).distance(center));
    }
    (center, radius)
}
