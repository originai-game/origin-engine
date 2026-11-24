//! Mesh levels of detail (docs/design.md §2.6): which of a model's cooked
//! levels to draw, from how far each level's error would show on the
//! screen.
//!
//! The cook simplifies every model mesh into levels (`model { lods 1 0.5
//! 0.25 }` by default, or a glTF's own `MSFT_lod` levels) and measures
//! each level's error: the largest distance between the level's surface
//! and the full mesh's, both ways, in the model's units. A level is drawn
//! while that error, projected to the screen at the model's distance,
//! stays under [`LOD_PIXEL_ERROR`] pixels: a 3 cm error on a prop 10 m
//! away at 720p is 2.3 px and keeps the full mesh, the same prop at 100 m
//! draws its coarsest level. The projected error is multiplied by the
//! bias (`RenderSettings.lodBias` × `og.Model.lodBias`) before it is
//! compared, so 2 keeps detail twice as far and 0 keeps the full mesh.
//!
//! Selection by the triangle fraction a level keeps (depth S6: level `k`
//! below `1.2 × fraction` of the view height) was replaced here: it drew a
//! 0.7 m lantern or a 2 m voxel tree at a quarter of its triangles while
//! it still covered a third of the screen, and the error the cook had
//! measured went unread (debt round 2, the examples' reference drift).
//!
//! **Hysteresis.** A level is left only once its projected error has
//! passed the threshold by [`LOD_HYSTERESIS`] of it, so a model resting on
//! a boundary does not alternate between levels from frame to frame; the
//! caller keeps the level each model drew last ([`select`]'s `previous`).
//! The choice reads the camera and the model's transform alone, never the
//! simulation, and nothing it decides feeds back.

use crate::scene::{Camera, Projection};
use glam::Vec3;
pub use og_render_data::lod::*;

/// How many pixels of a view `view_height` pixels tall one unit of length
/// covers at the near side of the sphere `radius` about `center` (the
/// nearest the model's surface can be); `f32::MAX` when the camera is
/// inside the sphere.
pub fn pixels_per_unit(camera: &Camera, center: Vec3, radius: f32, view_height: f32) -> f32 {
    match camera.projection {
        Projection::Orthographic { height } => view_height / height.max(1e-6),
        Projection::Perspective { fov_y_degrees }
        | Projection::PerspectiveShifted { fov_y_degrees, .. } => {
            let d = (center - camera.position).length() - radius;
            if d <= 1e-6 {
                return f32::MAX;
            }
            let half = (fov_y_degrees.to_radians() * 0.5).tan().max(1e-6);
            view_height / (2.0 * d * half)
        }
    }
}

/// The level to draw of a chain whose levels stand `errors` from the full
/// mesh (level 0 first, 0; the model's units), seen at `pixels_per_unit`
/// (the entity's scale folded in) and `bias`; `previous` is the level
/// drawn last frame (`None`: the first frame, no hysteresis). The coarsest
/// level whose projected error stays under [`LOD_PIXEL_ERROR`] is drawn;
/// a level's error counts as at least the one before it.
pub fn select(errors: &[f32], pixels_per_unit: f32, bias: f32, previous: Option<usize>) -> usize {
    if bias <= 0.0 || errors.len() < 2 || !pixels_per_unit.is_finite() {
        return 0;
    }
    let mut level = 0;
    let mut error = 0.0f32;
    for (k, &e) in errors.iter().enumerate().skip(1) {
        error = error.max(e);
        let screen = error * pixels_per_unit * bias;
        // Moving to a coarser level than the last one waits until its
        // error is well under the threshold; returning to a finer one,
        // until the coarser one's error is well over it.
        let threshold = match previous {
            Some(p) if k <= p => LOD_PIXEL_ERROR * (1.0 + LOD_HYSTERESIS),
            Some(_) => LOD_PIXEL_ERROR * (1.0 - LOD_HYSTERESIS),
            None => LOD_PIXEL_ERROR,
        };
        if screen < threshold {
            level = k;
        } else {
            break;
        }
    }
    level
}

/// The colour `RenderOptions.debugView lod` gives a level: green the full
/// mesh, yellow, orange, red from level 3 on.
pub fn debug_color(level: usize) -> Vec3 {
    match level {
        0 => Vec3::new(0.1, 0.85, 0.15),
        1 => Vec3::new(0.95, 0.85, 0.1),
        2 => Vec3::new(1.0, 0.45, 0.05),
        _ => Vec3::new(0.9, 0.08, 0.08),
    }
}
