//! Mesh levels of detail (docs/design.md §2.6): which of a model's cooked
//! levels to draw, from the size it projects to on the screen.
//!
//! The cook simplifies every model mesh into levels, each a fraction of the
//! full mesh's triangles (`model { lods 1 0.5 0.25 }` by default, or a
//! glTF's own `MSFT_lod` levels). A level is drawn while the model's
//! bounding sphere covers less of the view's height than
//! [`LOD_FULL_SIZE`] times the level's fraction: with the default levels,
//! the full mesh above 60 %, half its triangles down to 30 %, a quarter
//! below (Unity's default LOD group thresholds). The size is multiplied by
//! the bias (`RenderSettings.lodBias` × `og.Model.lodBias`; 0 keeps the
//! full mesh) before it is compared.
//!
//! **Hysteresis.** A level is left only once the size has passed its
//! threshold by [`LOD_HYSTERESIS`] of it, so a model resting on a
//! boundary does not alternate between levels from frame to frame; the
//! caller keeps the level each model drew last ([`select`]'s `previous`).
//! The choice reads the camera and the model's transform alone, never the
//! simulation, and nothing it decides feeds back.

use crate::scene::{Camera, Projection};
use glam::Vec3;

/// The view height fraction at which a level of fraction 1 would stop
/// being enough; level k is drawn below `LOD_FULL_SIZE × fraction_k`.
pub const LOD_FULL_SIZE: f32 = 1.2;

/// How far past its threshold (a fraction of it) the size must go before
/// the level changes.
pub const LOD_HYSTERESIS: f32 = 0.15;

/// The most levels a chain keeps (the cook writes at most eight).
pub const MAX_LOD_LEVELS: usize = 8;

/// The fraction of the view's height a sphere of `radius` at `center`
/// covers (above 1 when the camera is inside it or close).
pub fn projected_size(camera: &Camera, center: Vec3, radius: f32) -> f32 {
    match camera.projection {
        Projection::Orthographic { height } => 2.0 * radius / height.max(1e-6),
        Projection::Perspective { fov_y_degrees } => {
            let d = (center - camera.position).length();
            if d <= radius {
                return f32::MAX;
            }
            let half = (fov_y_degrees.to_radians() * 0.5).tan().max(1e-6);
            radius / (d * half)
        }
    }
}

/// The level to draw of a chain whose levels keep `fractions` of the full
/// mesh's triangles (level 0 first, 1), at projected `size` and `bias`;
/// `previous` is the level drawn last frame (`None`: the first frame, no
/// hysteresis).
pub fn select(fractions: &[f32], size: f32, bias: f32, previous: Option<usize>) -> usize {
    if bias <= 0.0 || fractions.len() < 2 {
        return 0;
    }
    let size = size * bias;
    let mut level = 0;
    for (k, &f) in fractions.iter().enumerate().skip(1) {
        let threshold = LOD_FULL_SIZE * f;
        // Moving to a coarser level than the last one waits until the size
        // is well below the threshold; returning to a finer one, well above.
        let threshold = match previous {
            Some(p) if k <= p => threshold * (1.0 + LOD_HYSTERESIS),
            Some(_) => threshold * (1.0 - LOD_HYSTERESIS),
            None => threshold,
        };
        if size < threshold {
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

    const DEFAULT: [f32; 3] = [1.0, 0.5, 0.25];

    #[test]
    fn levels_follow_the_projected_size() {
        // 60 % and 30 % of the view height are the default boundaries.
        assert_eq!(select(&DEFAULT, 0.9, 1.0, None), 0);
        assert_eq!(select(&DEFAULT, 0.5, 1.0, None), 1);
        assert_eq!(select(&DEFAULT, 0.1, 1.0, None), 2);
        // A bias of 2 keeps detail twice as far; 0 keeps the full mesh.
        assert_eq!(select(&DEFAULT, 0.5, 2.0, None), 0);
        assert_eq!(select(&DEFAULT, 0.01, 0.0, None), 0);
        // One level: nothing to choose.
        assert_eq!(select(&[1.0], 0.01, 1.0, None), 0);
    }

    #[test]
    fn a_level_is_kept_near_its_boundary() {
        // Just under the 30 % boundary: level 2 from nothing, but a model
        // that drew level 1 keeps it until well past it, and back again.
        let near = 0.29;
        assert_eq!(select(&DEFAULT, near, 1.0, None), 2);
        assert_eq!(select(&DEFAULT, near, 1.0, Some(1)), 1);
        assert_eq!(select(&DEFAULT, 0.25, 1.0, Some(1)), 2);
        assert_eq!(select(&DEFAULT, 0.32, 1.0, Some(2)), 2);
        assert_eq!(select(&DEFAULT, 0.36, 1.0, Some(2)), 1);
    }

    #[test]
    fn the_size_is_the_sphere_over_the_view_height() {
        let camera = Camera {
            position: Vec3::ZERO,
            projection: Projection::Perspective {
                fov_y_degrees: 90.0,
            },
            ..Camera::default()
        };
        // tan 45° = 1: radius 1 at 10 m covers a tenth of the height.
        let s = projected_size(&camera, Vec3::new(0.0, 0.0, -10.0), 1.0);
        assert!((s - 0.1).abs() < 1e-5, "{s}");
        assert_eq!(projected_size(&camera, Vec3::ZERO, 1.0), f32::MAX);
        let ortho = Camera {
            projection: Projection::Orthographic { height: 20.0 },
            ..camera
        };
        assert!((projected_size(&ortho, Vec3::new(5.0, 0.0, -100.0), 2.0) - 0.2).abs() < 1e-6);
    }
}
