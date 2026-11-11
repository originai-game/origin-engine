//! The stylise passes' frame: the conformed colour table remade when the
//! scene's colours change, the key their groups are cached under (palette,
//! canvas, table), the canvas's measured relief, and their uniform.

use super::*;

pub(super) struct PrepareFrameStyliseInputs<'a> {
    pub scene: &'a RenderScene,
    pub camera: &'a Camera,
    pub cam: &'a frame::CameraFrame,
    pub upscale: u32,
    pub grid_offset: (u32, u32),
    pub lens: Option<LensDistortion>,
    pub prepass: bool,
}

pub(super) struct PrepareFrameStyliseOutputs {
    pub stylise_key: u64,
}

impl Renderer {
    /// Prepares the stylise passes' table, key, canvas statistics and uniform.
    #[allow(clippy::too_many_lines)]
    pub(super) fn prepare_frame_stylise(
        &mut self,
        PrepareFrameStyliseInputs {
            scene,
            camera,
            cam,
            upscale,
            grid_offset,
            lens,
            prepass,
        }: PrepareFrameStyliseInputs<'_>,
    ) -> PrepareFrameStyliseOutputs {
        let st = &scene.stylise;
        // The stylise passes' parameters, palette and canvas.
        let (colors, revision) = scene.conformed_colors();
        if revision != self.conformed.0 {
            let (size, bytes) = crate::stylise::conformed_table(colors);
            self.conformed = (
                revision,
                crate::post::texture_with_data(
                    &self.device,
                    &self.queue,
                    size,
                    wgpu::TextureFormat::R32Uint,
                    &bytes,
                ),
            );
        }
        let stylise_key = {
            // The slot's revision too: a palette added in a freed slot is
            // another texture under the same index.
            let p = st.quantize.and_then(|q| q.palette).map_or(u64::MAX, |p| {
                let revision = self.cache.palette_revisions.get(p.index()).copied();
                p.index() as u64 ^ revision.unwrap_or(0).rotate_left(24)
            });
            let c = st
                .paint
                .and_then(|p| p.canvas)
                .map_or(u64::MAX, |t| t.index() as u64);
            scene.id() ^ p.rotate_left(16) ^ c.rotate_left(40) ^ self.conformed.0.rotate_left(56)
        };
        // The canvas's mean height and spread, measured once per texture
        // revision: the wash reads the paper's relief about them.
        if let Some(t) = st.paint.and_then(|p| p.canvas) {
            let key = Some((t.index(), scene.texture_revision(t.index())));
            if key != self.canvas_stats.0 {
                self.canvas_stats = (key, scene.texture(t).map_or([0.0; 2], canvas_stats));
            }
        }
        self.write_frame_stylise_uniform(
            st,
            scene,
            (camera, cam),
            upscale,
            grid_offset,
            lens,
            prepass,
            self.canvas_stats.1,
        );
        PrepareFrameStyliseOutputs { stylise_key }
    }
}
