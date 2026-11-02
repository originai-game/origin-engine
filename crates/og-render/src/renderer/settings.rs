//! Renderer sizing, quality and sample settings.

use super::*;

impl Renderer {
    pub fn info(&self) -> &RenderInfo {
        &self.info
    }

    pub fn size(&self) -> (u32, u32) {
        (self.out_width, self.out_height)
    }

    pub fn output_format(&self) -> wgpu::TextureFormat {
        self.output_format
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// The render graph the last frame was recorded from: its passes in
    /// execution order and its transient textures.
    pub fn frame_graph(&self) -> Option<&GraphPlan> {
        self.graph.plan()
    }

    /// Resizes every size-dependent target (and the surface, if any). TAA
    /// history restarts. Each side is clamped to 1 ..= the device's largest
    /// 2D texture.
    pub fn resize(&mut self, width: u32, height: u32) {
        let max = self.device.limits().max_texture_dimension_2d;
        let (width, height) = (width.clamp(1, max), height.clamp(1, max));
        if (width, height) == (self.out_width, self.out_height) {
            return;
        }
        self.out_width = width;
        self.out_height = height;
        // The scene follows the output until the next frame's stylisation
        // sets its own grid (`scene_size`).
        self.width = width;
        self.height = height;
        self.taau = false;
        self.persistent = post::Persistent::new(
            &self.device,
            width,
            height,
            self.output_format,
            self.offscreen_linear,
            self.info.antialias,
        );
        self.graph.invalidate();
        self.taa_history = None;
        if let Some((surface, config)) = &mut self.surface {
            config.width = width;
            config.height = height;
            surface.configure(&self.device, config);
        }
    }

    /// Brings the shadow maps and the samples per pixel to the frame's
    /// quality ([`RenderScene::quality`]): another
    /// [`RenderQuality::shadow_map_scale`] makes the shadow maps again at
    /// their new sides, another [`RenderQuality::msaa_samples`] switches
    /// MSAA and FXAA; the same quality changes nothing.
    pub(crate) fn apply_quality(&mut self, q: &RenderQuality) {
        let max = self.device.limits().max_texture_dimension_2d;
        let cascade = q.shadow_side(
            self.full_shadow_size,
            max.min(self.full_shadow_size.max(4096)),
        );
        let local = q.shadow_side(LOCAL_SHADOW_SIZE, max);
        if (cascade, local) != (self.info.shadow_map_size, self.local_shadow_size) {
            self.info.shadow_map_size = cascade;
            self.local_shadow_size = local;
            self.cascade_map =
                ShadowMap::new(&self.device, "cascades", cascade, self.info.shadow_cascades);
            self.local_map = ShadowMap::new(
                &self.device,
                "local shadows",
                local,
                self.local_map.layers.len() as u32,
            );
            // The frame groups bind them: ensure_environment makes them again.
            self.environment = None;
        }
        // The device's samples cap the quality's (RenderInfo::msaa_samples):
        // where it resolved no MSAA, one sample whatever the quality asks.
        if self.info.msaa_samples > 1 {
            let samples = if q.multisample() {
                self.info.msaa_samples
            } else {
                1
            };
            if samples != self.samples {
                self.set_samples(samples);
            }
        }
    }

    /// Draws the scene at `samples` per pixel from now on: MSAA at
    /// [`MSAA_SAMPLES`], else one sample and FXAA ([`RenderInfo::antialias`]
    /// says which). Every pipeline that draws into the scene's target is
    /// made again at the count as it is next used (each kept at both
    /// counts where it is a [`Sampled`]), and the frame's targets with it.
    pub(crate) fn set_samples(&mut self, samples: u32) {
        self.samples = samples;
        self.info.antialias = if samples > 1 {
            Antialias::Msaa4
        } else {
            Antialias::Fxaa
        };
        self.graph.invalidate();
        // Made at a count, told the new one.
        for p in [
            &self.sky_pipeline,
            &self.sky_look_pipeline,
            &self.sprite_pipeline,
            &self.sprite_additive_pipeline,
            &self.sprite_untonemapped_pipeline,
            &self.sprite_cover_pipeline,
            &self.msdf_pipeline,
        ]
        .into_iter()
        .chain(&self.ribbon_pipelines)
        {
            p.set_samples(samples);
        }
        self.water.pipeline.set_samples(samples);
        self.water.mesh_pipeline.set_samples(samples);
        self.metaballs.set_samples(samples);
        self.fluid3d.set_samples(samples);
        self.lines.set_samples(samples);
        self.terrain.set_samples(samples);
        self.scatter.set_samples(samples);
        if let Some(a) = &self.sprite_arrays {
            a.set_samples(samples);
        }
        // Made as batches need them: made again at the new count.
        self.forget_lit_pipelines();
        self.sprite_meshes = None;
        self.sprite_normals = None;
    }

    /// Forgets the lit pipelines, which batches make again as they need
    /// them, at the current samples; the plain
    /// one, which some passes take without a batch, is made now.
    fn forget_lit_pipelines(&mut self) {
        self.pbr_pipelines.clear();
        let key = (Side::Front, false, false, false);
        self.pbr_pipelines.insert(
            key,
            pbr_pipeline(
                &self.device,
                (&self.pbr_layout, &self.pbr_skinned_layout),
                &self.pbr_shader,
                key,
                self.samples,
            ),
        );
        self.lean_pipelines.clear();
        self.cache.hook_pipelines.clear();
        self.cache.physical_hook_pipelines.clear();
        if let Some(a) = &mut self.array_maps {
            a.lit.clear();
        }
        if let Some(p) = &mut self.physical {
            p.pipelines.clear();
        }
        self.lightmaps.forget_pipelines();
    }

    /// The size the scene is drawn at this frame: the output's, the
    /// output's at the render scale ([`Renderer::set_render_scale`]), or
    /// the stylisation's logical grid ([`PixelScale::fit`]), with the
    /// whole-number upscale (0: the render scale's, filling the output
    /// bilinearly) and the scaled frame's offset in the output.
    pub(crate) fn pixel_grid(&self, pixel: Option<PixelScale>) -> ((u32, u32), u32, (u32, u32)) {
        let out = (self.out_width, self.out_height);
        let Some(p) = pixel.filter(|p| p.height > 0) else {
            let scaled = render_scaled(out, self.render_scale, self.info.max_texture_size);
            return (scaled, u32::from(scaled == out), (0, 0));
        };
        let f = p.fit(out);
        ((f.width, f.height), f.upscale, f.offset)
    }

    /// Draws the scene at `width` × `height` from now on (the output keeps
    /// its size): the graph, the TAA history and the glass targets follow.
    /// `taau`: TAA resolves it to the output's size (the render scale's
    /// frame under TAA, smaller than the output), where its history then
    /// lives.
    pub(crate) fn scene_size(&mut self, width: u32, height: u32, taau: bool) {
        let taau = taau && self.info.antialias == Antialias::Taa;
        if (width, height) == (self.width, self.height) && taau == self.taau {
            return;
        }
        let kept = taau && self.taau;
        self.width = width;
        self.height = height;
        self.taau = taau;
        self.graph.invalidate();
        // Under TAAU before and after the history is the output's size:
        // a new render scale (dynamic resolution) keeps it.
        if kept {
            return;
        }
        let (hw, hh) = self.history_size();
        self.persistent
            .resize_history(&self.device, hw, hh, self.info.antialias);
        self.taa_history = None;
    }

    /// The size of TAA's history and of the frame after it: the output's
    /// under TAAU, else the scene's.
    pub(crate) fn history_size(&self) -> (u32, u32) {
        if self.taau {
            (self.out_width, self.out_height)
        } else {
            (self.width, self.height)
        }
    }
}

/// The output's size at render scale `scale`, each side within 1 and `max`.
fn render_scaled(out: (u32, u32), scale: f32, max: u32) -> (u32, u32) {
    if scale == 1.0 {
        return out;
    }
    let side = |n: u32| ((n as f32 * scale).round() as u32).clamp(1, max.max(1));
    (side(out.0), side(out.1))
}

#[cfg(test)]
mod tests {
    /// A fractional scale rounds each side on its own (the aspect is kept
    /// to a pixel), never reaches 0, and never passes the device's largest
    /// texture; scale 1 is the output exactly.
    #[test]
    fn the_scaled_scene_rounds_each_side_within_the_device() {
        assert_eq!(super::render_scaled((1920, 1080), 1.0, 8192), (1920, 1080));
        assert_eq!(super::render_scaled((1920, 1080), 0.5, 8192), (960, 540));
        assert_eq!(super::render_scaled((1179, 2556), 0.7, 8192), (825, 1789));
        assert_eq!(super::render_scaled((1, 1), 0.5, 8192), (1, 1));
        assert_eq!(super::render_scaled((5000, 3000), 2.0, 8192), (8192, 6000));
    }
}
