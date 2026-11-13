//! Where a camera-centred map sits: a square of cells snapped to whole
//! cells of the absolute world.

use glam::{DVec2, IVec2, Vec2};

/// The cell the camera stands in, in whole cells of `cell` metres of the
/// absolute world (so the scene's origin moving does not move it).
pub(crate) fn camera_cell(camera: DVec2, cell: f64) -> IVec2 {
    let c = (camera / cell).floor();
    IVec2::new(c.x as i32, c.y as i32)
}

/// The first cell of a window of `n` cells a side centred on the camera.
pub(crate) fn first_cell(camera: DVec2, cell: f64, n: u32) -> IVec2 {
    camera_cell(camera, cell) - IVec2::splat(n as i32 / 2)
}

/// The window's min corner in the absolute world (m).
pub(crate) fn corner(first: IVec2, cell: f64) -> DVec2 {
    first.as_dvec2() * cell
}

/// The window's min corner from the scene's `origin` (m), as a shader
/// reads it.
pub(crate) fn corner_from(first: IVec2, cell: f64, origin: DVec2) -> Vec2 {
    (corner(first, cell) - origin).as_vec2()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The window holds the camera at its middle cell, stays put while the
    /// camera moves inside a cell, and moves by whole cells: on either
    /// side of zero alike.
    #[test]
    fn the_window_is_snapped_to_whole_world_cells() {
        let at = |x: f64| first_cell(DVec2::new(x, 0.0), 0.25, 256).x;
        assert_eq!(at(0.0), -128);
        assert_eq!(at(0.24), -128);
        assert_eq!(at(0.25), -127);
        assert_eq!(at(-0.01), -129);
        assert_eq!(at(-0.25), -129);
        assert_eq!(at(-0.26), -130);
    }

    /// The corner a shader reads is the same world point whatever the
    /// scene's origin.
    #[test]
    fn the_corner_does_not_move_with_the_origin() {
        let first = first_cell(DVec2::new(1000.3, -2000.7), 0.5, 64);
        let world = corner(first, 0.5);
        for origin in [DVec2::ZERO, DVec2::new(1024.0, -2048.0)] {
            let rel = corner_from(first, 0.5, origin).as_dvec2();
            assert!((rel + origin - world).length() < 1e-3);
        }
    }
}
