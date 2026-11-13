//! Which water the wakes' square ([`crate::sea_wakes`]) runs over, and its
//! step: the frame's first og.Water that bodies cut and that draws their
//! waves, a sea, a lake or a river alike. One square: its waves are a
//! picture near the camera, and two stirred waters in one view are rare.
//! A water nothing cuts costs nothing: the square is made with the first
//! cut and rests once its waves are gone.

use super::*;

/// The surface the wakes run over this frame: the first one with wakes to
/// draw (dynamic waves, and no height field of its own: an og.WaterSurface
/// draws its ripples instead) that a body cuts now; with none cut, the
/// one that `held` them last frame, whose waves are still fading.
pub(super) fn stirred(
    waters: &[Water],
    held: Option<usize>,
) -> Option<(usize, &Water, &WaterWakes)> {
    let mut drawn = waters.iter().enumerate().filter_map(|(i, w)| {
        let wakes = w.wakes.as_deref()?;
        (w.dynamic_waves > 0.0 && height_field(w).is_none()).then_some((i, w, wakes))
    });
    let cut = |w: &WaterWakes| w.ticks.iter().any(|(_, cuts)| !cuts.is_empty());
    drawn
        .clone()
        .find(|(_, _, w)| cut(w))
        .or_else(|| drawn.find(|(i, ..)| Some(*i) == held))
}

impl WaterRenderer {
    /// Plans the wakes of [`stirred`]'s surface to its tick, for the frame's
    /// encoder ([`WaterRenderer::encode_fields`]), and sets `self.wake` (the surface's index and the
    /// square's centre, frame xz): none without such a surface, or once its
    /// waves are gone.
    pub(super) fn step_wakes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        waters: &[Water],
        camera: &crate::Camera,
        origin: Vec3,
    ) {
        let held = self.wake.take().map(|(i, _)| i);
        let Some((index, water, wakes)) = stirred(waters, held).filter(|_| self.compute) else {
            return;
        };
        let world = |v: Vec3| glam::DVec2::new(f64::from(v.x), f64::from(v.z));
        let centre = water.transform.w_axis;
        let pass = self
            .wakes
            .get_or_insert_with(|| crate::sea_wakes::SeaWakes::new(device));
        if held.is_some_and(|h| h != index) {
            pass.rest();
        }
        let drawn = pass.update(
            queue,
            wakes,
            world(camera.position) + world(origin),
            Vec2::new(centre.x, centre.z),
            world(origin),
            water.dynamic_waves,
        );
        if drawn {
            self.wake = Some((index, pass.centre(world(origin))));
        }
    }
}
