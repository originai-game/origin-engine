//! The grass's generate pass and its draws.

use super::*;

impl GrassRenderer {
    /// Generates this frame's blades (before every pass that draws them).
    pub(crate) fn encode(
        &self,
        encoder: &mut crate::Encoder,
        frame: &GrassFrame,
        terrain_draws: &[impl GroundDraw],
    ) {
        if frame.fields.is_empty() {
            return;
        }
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("grass"),
            timestamp_writes: None,
        });
        for field in &frame.fields {
            let Some([evaluate, scan, compact]) = &self.generate[usize::from(field.arrayed)] else {
                continue;
            };
            let f = &self.fields[field.slot];
            pass.set_bind_group(0, &f.gen_group, &[]);
            pass.set_bind_group(1, terrain_draws[field.terrain_draw].group(), &[0]);
            pass.set_bind_group(2, &field.maps, &[]);
            pass.set_pipeline(evaluate);
            pass.dispatch_workgroups(field.slices, 1, 1);
            pass.set_pipeline(scan);
            pass.dispatch_workgroups(1, 1, 1);
            pass.set_pipeline(compact);
            pass.dispatch_workgroups(field.slices, 1, 1);
        }
    }

    /// Draws this frame's fields into a pass whose group 0 (and, lit, group
    /// 2: `fx`) the caller bound: the levels of detail that stand within
    /// `reach` of the camera or throw their shadow there (a cascade's
    /// [`crate::shadow_fit::Cascade::reach`]; infinite: all of them). Answers
    /// the draws made.
    pub(crate) fn draw(
        &self,
        pass: &mut wgpu::RenderPass,
        kind: GrassPass,
        frame: &GrassFrame,
        fx: Option<&wgpu::BindGroup>,
        reach: f32,
    ) -> u32 {
        let Some(pipelines) = &self.pipelines else {
            return 0;
        };
        let mut draws = 0;
        for field in &frame.fields {
            if kind == GrassPass::Shadow && !field.cast {
                continue;
            }
            let f = &self.fields[field.slot];
            pass.set_pipeline(&pipelines[kind as usize]);
            pass.set_bind_group(1, &f.draw_group, &[]);
            if kind == GrassPass::Lit
                && let Some(fx) = fx
            {
                pass.set_bind_group(2, fx, &[]);
            }
            pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint32);
            for l in 0..3usize {
                // A level none of whose blades shades what the pass holds
                // (a near cascade's: only its vertices would be spent).
                if l > 0 && field.shades_from[l - 1] > reach {
                    continue;
                }
                pass.draw_indexed_indirect(&f.draws, l as u64 * 20);
                draws += 1;
            }
        }
        draws
    }
}
