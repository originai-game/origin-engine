//! The frame's uploads past the instance records: the skinned records and
//! the joint palette, the crowds' card atlases, the strips, sprites and
//! particles, the overlay's instances and uniform, and the 2D lights and
//! occluders, in that order.

use super::*;

pub(super) struct UploadFrameInputs<'a> {
    pub scene: &'a RenderScene,
    pub target: &'a RenderTarget<'a>,
    pub cam: &'a frame::CameraFrame,
    pub crowd_gpu: bool,
    pub skinned_data: &'a [crate::skin::SkinnedInstanceGpu],
    pub joint_matrices: &'a [[[f32; 4]; 4]],
    pub crowd_frame: &'a crate::crowd::CrowdFrame,
    pub scatter_batches: &'a mut scatter::ScatterBatches,
    pub ribbon_frame: &'a crate::ribbons::RibbonFrame,
    pub sprite_frame: &'a sprites::SpriteFrame,
    pub particle_frame: &'a particles_gpu::ParticleFrame,
    pub gpu_sprites: &'a sprites::GpuSprites,
    pub overlay_frame: &'a Option<crate::overlay::OverlayFrame>,
    pub light_slots: &'a crate::light2d::LightSlots,
}

pub(super) struct UploadFrameOutputs {
    pub ui_uniform: Option<[f32; 8]>,
    pub lights_2d: u32,
    pub occluders_2d: u32,
}

impl Renderer {
    /// Uploads the frame's skinned, sprite, particle, overlay and 2D light data.
    #[allow(clippy::too_many_lines)]
    pub(super) fn upload_frame(
        &mut self,
        UploadFrameInputs {
            scene,
            target,
            cam,
            crowd_gpu,
            skinned_data,
            joint_matrices,
            crowd_frame,
            scatter_batches,
            ribbon_frame,
            sprite_frame,
            particle_frame,
            gpu_sprites,
            overlay_frame,
            light_slots,
        }: UploadFrameInputs<'_>,
    ) -> UploadFrameOutputs {
        if !skinned_data.is_empty() || crowd_gpu {
            grow_and_write(
                &self.device,
                &self.queue,
                &mut self.instances.skinned_buffer,
                "skinned instances",
                bytemuck::cast_slice(skinned_data),
            );
            let morphs_changed = std::mem::take(&mut self.cache.morphs_dirty);
            self.instances.palette.write(
                &self.device,
                &self.queue,
                &self.palette_layout,
                joint_matrices,
                &self.cache.morph_data,
                morphs_changed,
            );
        }
        // The crowds' cards' atlases, baked from the palette's clips.
        self.prepare_crowd_cards(scene, crowd_frame, &mut scatter_batches.cards);
        grow_and_write(
            &self.device,
            &self.queue,
            &mut self.strips.ribbon_buffer,
            "strips",
            bytemuck::cast_slice(&ribbon_frame.vertices),
        );
        grow_and_write(
            &self.device,
            &self.queue,
            &mut self.sprite.sprite_buffer,
            "sprites",
            bytemuck::cast_slice(&sprite_frame.instances),
        );
        self.particles
            .upload(&self.device, &self.queue, particle_frame, gpu_sprites);
        let mut ui_uniform = None;
        if let (Some(of), Some(o)) = (overlay_frame, &scene.overlay) {
            grow_and_write(
                &self.device,
                &self.queue,
                &mut self.overlay.buffer,
                "overlay instances",
                bytemuck::cast_slice(&of.instances),
            );
            let u = overlay::OverlayRenderer::uniform(
                self.out_width,
                self.out_height,
                o,
                !self.output_format.is_srgb()
                    && self.output_format != HDR_FORMAT
                    && !self.has_linear_view(target),
            );
            ui_uniform = Some(u);
        }
        // The sprite shaders' light map strips and ambient
        // (`SpriteFxFrame::light`).
        let ambient_2d = scene.lighting_2d.map_or(Vec3::ONE, |l| l.ambient);
        self.queue.write_buffer(
            &self.sprite.sprite_fx_buffer,
            16,
            bytemuck::bytes_of(&[
                light_slots.count() as f32,
                ambient_2d.x,
                ambient_2d.y,
                ambient_2d.z,
            ]),
        );
        let (lights_2d, occluders_2d) = match scene.lighting_2d {
            Some(l) => self.lights_2d.write(
                &self.device,
                &self.queue,
                cam.view_proj,
                &scene.lights_2d,
                &scene.occluders_2d,
                light_slots,
                l.shadow_angles,
            ),
            None => (0, 0),
        };
        UploadFrameOutputs {
            ui_uniform,
            lights_2d,
            occluders_2d,
        }
    }
}
