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

/// The most a drawn level's error may cover on the screen, in pixels of
/// the render target's height, at bias 1.
pub const LOD_PIXEL_ERROR: f32 = 1.0;

/// How far past its threshold (a fraction of it) the projected error must
/// go before the level changes.
pub const LOD_HYSTERESIS: f32 = 0.15;

/// The most levels a chain keeps (the cook writes at most eight).
pub const MAX_LOD_LEVELS: usize = 8;

/// How many pixels of a view `view_height` pixels tall one unit of length
/// covers at the near side of the sphere `radius` about `center` (the
/// nearest the model's surface can be); `f32::MAX` when the camera is
/// inside the sphere.
pub fn pixels_per_unit(camera: &Camera, center: Vec3, radius: f32, view_height: f32) -> f32 {
    match camera.projection {
        Projection::Orthographic { height } => view_height / height.max(1e-6),
        Projection::Perspective { fov_y_degrees } => {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A chain like a 2 m voxel tree's: 3 cm off at level 1, 12 cm at 2.
    const TREE: [f32; 3] = [0.0, 0.03, 0.12];

    #[test]
    fn levels_follow_the_projected_error() {
        // 720 px, fov 50°, 10 m: 77 px/m. Level 1 would show 2.3 px: the
        // full mesh. At 40 m (19 px/m) level 1 shows 0.58 px and level 2
        // 2.3 px: level 1. At 200 m (3.9 px/m) level 2 shows 0.46 px.
        assert_eq!(select(&TREE, 77.0, 1.0, None), 0);
        assert_eq!(select(&TREE, 19.0, 1.0, None), 1);
        assert_eq!(select(&TREE, 3.9, 1.0, None), 2);
        // A bias of 2 keeps detail twice as far; 0 keeps the full mesh.
        assert_eq!(select(&TREE, 19.0, 2.0, None), 0);
        assert_eq!(select(&TREE, 9.5, 2.0, None), 1);
        assert_eq!(select(&TREE, 0.01, 0.0, None), 0);
        // One level: nothing to choose. The camera inside the sphere: the full mesh.
        assert_eq!(select(&[0.0], 0.01, 1.0, None), 0);
        assert_eq!(select(&TREE, f32::MAX, 1.0, None), 0);
        // A level no worse than the one before counts as the one before.
        assert_eq!(select(&[0.0, 0.03, 0.0], 19.0, 1.0, None), 2);
        assert_eq!(select(&[0.0, 0.03, 0.0], 77.0, 1.0, None), 0);
    }

    #[test]
    fn a_level_is_kept_near_its_boundary() {
        // Level 1 shows 0.96 px at 32 px/m: level 1 from nothing, but a
        // model that drew level 0 keeps it until the error is well under
        // a pixel, and one that drew level 1 keeps it until well over.
        assert_eq!(select(&TREE, 32.0, 1.0, None), 1);
        assert_eq!(select(&TREE, 32.0, 1.0, Some(0)), 0);
        assert_eq!(select(&TREE, 28.0, 1.0, Some(0)), 1);
        assert_eq!(select(&TREE, 36.0, 1.0, Some(1)), 1);
        assert_eq!(select(&TREE, 39.0, 1.0, Some(1)), 0);
    }

    #[test]
    fn pixels_per_unit_is_the_view_height_over_the_near_side() {
        let camera = Camera {
            position: Vec3::ZERO,
            projection: Projection::Perspective {
                fov_y_degrees: 90.0,
            },
            ..Camera::default()
        };
        // tan 45° = 1: at 10 m a 720 px view spans 20 m, 36 px/m; a sphere
        // of radius 2 is read at its near side, 8 m, 45 px/m.
        let s = pixels_per_unit(&camera, Vec3::new(0.0, 0.0, -10.0), 0.0, 720.0);
        assert!((s - 36.0).abs() < 1e-3, "{s}");
        let s = pixels_per_unit(&camera, Vec3::new(0.0, 0.0, -10.0), 2.0, 720.0);
        assert!((s - 45.0).abs() < 1e-3, "{s}");
        assert_eq!(pixels_per_unit(&camera, Vec3::ZERO, 1.0, 720.0), f32::MAX);
        let ortho = Camera {
            projection: Projection::Orthographic { height: 20.0 },
            ..camera
        };
        assert!(
            (pixels_per_unit(&ortho, Vec3::new(5.0, 0.0, -100.0), 2.0, 720.0) - 36.0).abs() < 1e-6
        );
    }
}
