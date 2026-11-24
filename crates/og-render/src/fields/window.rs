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
