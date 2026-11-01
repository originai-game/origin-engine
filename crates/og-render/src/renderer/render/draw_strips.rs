//! Draw strips stage.

use super::*;

impl Renderer {
    pub(super) fn frame_draw_strips(
        &self,
        pass: &mut wgpu::RenderPass,
        runs: &[ribbons::RibbonRun],
    ) -> u32 {
        let mut draws = 0;
        for run in runs.iter().filter(|r| r.layer.is_none()) {
            pass.set_pipeline(
                &self.ribbon_pipelines[match run.blend {
                    ParticleBlend::Additive => 0,
                    ParticleBlend::Alpha => 1,
                }],
            );
            pass.set_bind_group(
                1,
                match run.texture {
                    None => &self.white_sprite_group,
                    Some(t) => self.cache.ribbon_groups[t.index()]
                        .as_ref()
                        .expect("made before the pass"),
                },
                &[],
            );
            let stride = std::mem::size_of::<ribbons::RibbonVertex>() as u64;
            let start = run.first as u64 * stride;
            pass.set_vertex_buffer(
                0,
                self.ribbon_buffer
                    .slice(start..start + run.count as u64 * stride),
            );
            pass.draw(0..run.count, 0..1);
            draws += 1;
        }
        draws
    }
}
