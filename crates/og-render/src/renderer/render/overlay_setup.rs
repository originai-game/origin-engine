//! Overlay setup stage.

use super::*;

impl Renderer {
    pub(super) fn prepare_frame_overlay(
        &mut self,
        scene: &RenderScene,
        overlay_frame: &Option<overlay::OverlayFrame>,
    ) -> bool {
        if let Some(of) = overlay_frame {
            let mut textures: Vec<_> = overlay::run_textures(of).collect();
            // Mass elements' icon textures (and the white one).
            if let Some(o) = &scene.overlay {
                for r in &of.runs {
                    if let overlay::OverlaySource::Mass(k) = r.source {
                        textures.push(o.ui.mass[k as usize].template.texture);
                    }
                }
            }
            for t in textures {
                self.sprite_group(t);
            }
            let arrays: Vec<_> = overlay::run_arrays(of).collect();
            for a in &arrays {
                self.sprite_array_group(scene, *a);
            }
            if !arrays.is_empty() {
                self.sprite_arrays
                    .as_mut()
                    .expect("made with the groups")
                    .make_overlay(
                        &self.device,
                        &self.overlay.layout,
                        self.overlay.output_format,
                        self.overlay.linear_format,
                    );
            }
            self.ui.prepare(&self.device, &self.queue, scene, of);
            // A blurred layer's levels (ui/pass.rs).
            if !of.layers.is_empty() {
                self.mips
                    .prepare(&self.device, crate::ui::pass::LAYER_FORMAT);
            }
        }
        // Glass: the tonemap renders into the glass source, which becomes
        // the output and the blur pyramid after the post passes (glass, a
        // UI backdrop, a blend that reads what is under it).
        let glass_on = overlay_frame
            .as_ref()
            .is_some_and(overlay::OverlayFrame::reads_backdrop);
        if glass_on
            && self
                .glass_targets
                .as_ref()
                .is_none_or(|g| g.size != (self.out_width, self.out_height))
        {
            self.glass_targets = Some(self.glass.targets(
                &self.device,
                &self.queue,
                self.output_format,
                self.offscreen_linear,
                self.out_width,
                self.out_height,
                &self.sprite_layout,
            ));
        }
        glass_on
    }

    pub(super) fn prepare_frame_lines(
        &mut self,
        scene: &RenderScene,
        cam: &frame::CameraFrame,
    ) -> lines::LineFrame {
        // Lines, points and wireframes (lines.rs): made and uploaded now,
        // drawn in the main pass and over the finished frame.
        let any_wire = self.cache.materials.iter().any(|m| m.wireframe.is_some());
        let materials = &self.cache.materials;
        let line_frame = self
            .lines
            .prepare(&self.device, &self.queue, scene, any_wire, |m| {
                materials.get(m).and_then(|g| g.wireframe)
            });
        self.lines.set_scene_uniform(
            &self.queue,
            &lines::LineUniform::new(
                cam.jittered_view_proj,
                cam.proj,
                self.width,
                self.height,
                scene.css_height,
                false,
            ),
        );
        line_frame
    }
}
