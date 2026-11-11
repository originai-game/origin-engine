//! The UI's tables and groups, uploaded after the graph is prepared: its
//! depth occluders and mass elements read the graph's view depth, its glass
//! the backdrop pyramid.

use super::*;

pub(super) struct UploadUiInputs<'a> {
    pub scene: &'a RenderScene,
    pub cam: &'a frame::CameraFrame,
    pub overlay_frame: &'a Option<crate::overlay::OverlayFrame>,
    pub ui_uniform: Option<[f32; 8]>,
    pub glass_on: bool,
}

impl Renderer {
    /// Uploads the overlay UI's tables and groups.
    #[allow(clippy::too_many_lines)]
    pub(super) fn upload_ui(
        &mut self,
        UploadUiInputs {
            scene,
            cam,
            overlay_frame,
            ui_uniform,
            glass_on,
        }: UploadUiInputs<'_>,
    ) {
        // The UI's tables and groups: after the graph, whose view depth
        // occluders and mass elements read.
        if let (Some(of), Some(o), Some(u)) = (overlay_frame, &scene.overlay, ui_uniform) {
            let textures = &self.cache.textures;
            let backdrop = self.glass_targets.as_ref().filter(|_| glass_on).map(|g| {
                (
                    &g.pyramid_view,
                    &g.behind_view,
                    &g.source_view,
                    !self.output_format.is_srgb() && self.output_format != HDR_FORMAT,
                )
            });
            let depth = self
                .graph
                .try_view("lindepth")
                .unwrap_or(&self.post.far_depth);
            let scale = overlay::OverlayRenderer::scale(self.out_height, o).max(1e-6);
            let camera = ui::mass::MassCameraGpu {
                view_proj: cam.view_proj.to_cols_array_2d(),
                eye: scene
                    .camera
                    .position
                    .extend(f32::from(u8::from(cam.orthographic)))
                    .to_array(),
                forward: cam.forward.extend(0.0).to_array(),
                css: [
                    self.out_width as f32 / scale,
                    self.out_height as f32 / scale,
                    o.ui.tick_fraction.clamp(0.0, 1.0),
                    0.0,
                ],
                tick: [o.ui.tick, 0, 0, 0],
            };
            self.ui.upload(
                &self.device,
                &self.queue,
                &self.overlay.layout,
                scene,
                of,
                u,
                backdrop,
                |t| textures.get(t.index()).map(|g| g.view.clone()),
                |i| textures.get(i).map(|g| g.view.clone()),
                depth,
                camera,
                &self.mips,
            );
        }
    }
}
