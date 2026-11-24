//! The weather occlusion map: where a roof keeps the rain and snow off.

use super::SurfaceWeather;
use crate::fields::window;
use glam::{DVec2, IVec2, Vec3};

/// How far above and below the camera the shelter map reaches, in metres
/// along the fall.
pub(crate) const SHELTER_REACH: f32 = 400.0;
/// The shelter map's cells per side of its square: the camera moving a
/// cell redraws it.
pub(crate) const SHELTER_CELLS: f32 = 8.0;
/// How wide the drip line's soft edge is, in metres.
pub(crate) const DRIP_EDGE: f32 = 0.6;

/// The weather occlusion map's view (the R4 shelter): an orthographic
/// projection looking along `w.fall` over the `w.shelter` square around
/// `camera` (frame coordinates), centred on the snapped cell of the
/// camera's world position (`camera + origin`), so it moves only when the
/// camera crosses a cell. `None` without shelter. Its depth is what the
/// shadow pass writes for a directional cascade (clip z), and a surface is
/// sheltered where something nearer the sky along the fall was drawn.
/// Returns the view projection and the cell.
pub(crate) fn shelter_view(
    w: &SurfaceWeather,
    camera: Vec3,
    origin: Vec3,
) -> Option<(glam::Mat4, IVec2)> {
    if w.shelter <= 0.0 {
        return None;
    }
    let side = w.shelter.clamp(4.0, 1024.0);
    let cell = side / SHELTER_CELLS;
    let origin_xz = DVec2::new(f64::from(origin.x), f64::from(origin.z));
    let world = DVec2::new(f64::from(camera.x), f64::from(camera.z)) + origin_xz;
    let at = window::camera_cell(world, f64::from(cell));
    let corner = window::corner_from(at, f64::from(cell), origin_xz);
    let centre = Vec3::new(corner.x + 0.5 * cell, camera.y, corner.y + 0.5 * cell);
    let d = w.fall.normalize_or(Vec3::NEG_Y);
    let up = if d.y.abs() > 0.9 { Vec3::Z } else { Vec3::Y };
    let eye = centre - d * SHELTER_REACH;
    let view = glam::camera::rh::view::look_at_mat4(eye, centre, up);
    let h = side * 0.5;
    let proj =
        glam::camera::rh::proj::directx::orthographic(-h, h, -h, h, 0.0, 2.0 * SHELTER_REACH);
    Some((proj * view, at))
}

/// What the shelter map was drawn from: its cell, its fall and side, and
/// the casters inside its volume (their bounding spheres, hashed). The map
/// is drawn again only when this changes: the camera crosses a cell, the
/// weather turns, or geometry under the map moves, appears or goes (a
/// static shadow cache, when the shadows have one, is where this should
/// later live with theirs).
pub(crate) fn shelter_key(
    w: &SurfaceWeather,
    cell: IVec2,
    spheres: impl Iterator<Item = (Vec3, f32)>,
    terrains: usize,
) -> u64 {
    let bits = |v: f32| u64::from(v.to_bits());
    let weather = [w.fall.x, w.fall.y, w.fall.z, w.shelter].map(bits);
    let spheres = spheres.flat_map(|(c, r)| [c.x, c.y, c.z, r].map(bits));
    og_core::hash::hash_words(
        [cell.x as u64, cell.y as u64]
            .into_iter()
            .chain(weather)
            .chain([terrains as u64])
            .chain(spheres),
    )
}
