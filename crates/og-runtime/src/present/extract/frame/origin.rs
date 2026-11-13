//! The render origin: the scene rebased near the camera.

use super::*;

/// The render origin's grid (m): a power of two, so moving a position near
/// a grid point to it is exact.
pub(crate) const RENDER_GRID: f32 = 8192.0;

/// The render origin for a camera at `position` in a world whose zero
/// og.WorldOrigin puts at `zero` of the absolute world (R12; zero without
/// one): the nearest point of the [`RENDER_GRID`] laid in the absolute
/// world, in the world's coordinates. The frame is drawn within 4096 m of
/// the camera on every axis; a shift of the world's zero moves the point
/// in the world's coordinates by the shift, so it stays where it was in the
/// absolute world and nothing drawn moves; and a scene without
/// og.WorldOrigin whose camera stays within 4096 m of the zero is drawn at
/// it, exactly as before render origins.
pub(crate) fn render_origin(position: Vec3, zero: glam::DVec3) -> Vec3 {
    if !position.is_finite() || !zero.is_finite() {
        return Vec3::ZERO;
    }
    let g = f64::from(RENDER_GRID);
    let at = position.as_dvec3() + zero;
    ((at / g).round() * g - zero).as_vec3()
}

/// Moves the frame, built in the world, to its render origin
/// (`og_render::RenderScene::rebase`): the GPU then works on positions
/// within a few kilometres of the camera, so nothing jitters however far
/// from the world's zero it is. The camera looks along `forward` (from its
/// rotation, exact) rather than at the target `position + forward`, which
/// was rounded at the camera's distance from the world's zero, unless
/// something since turned it (the review camera).
pub(super) fn rebase(scene: &mut og_render::RenderScene, origin: Vec3, forward: Option<Vec3>) {
    if origin == Vec3::ZERO {
        return;
    }
    let cam = scene.frame.camera;
    let look = cam.target - cam.position;
    // What rounding position + forward at this distance can do to it.
    let rounding = 4.0 * cam.position.abs().max_element() * f32::EPSILON + 1e-4;
    scene.rebase(origin);
    if let Some(f) = forward
        && (look - f).length() <= rounding
    {
        scene.frame.camera.target = scene.frame.camera.position + f;
    }
}
