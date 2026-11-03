//! Draw glow stage.

use super::*;

impl Renderer {
    pub(super) fn frame_draw_glow(
        &self,
        pass: &mut wgpu::RenderPass,
        runs: &[sprites::SpriteRun],
        glow_fx_group: &wgpu::BindGroup,
    ) -> u32 {
        let mut draws = 0;
        pass.set_bind_group(2, glow_fx_group, &[]);
        // Plain sprites first (sorted before arrays), then each array's.
        let mut on_array = None;
        for run in runs {
            // A 2D particle system simulated in compute: its range of its
            // own buffer, as its source.
            let (source, instances) = match run.source {
                RunSource::Particles(k) => (
                    self.particle_sprite_sources[k as usize],
                    &self.particle_sprite_buffer,
                ),
                source => (source, &self.sprite_buffer),
            };
            let array = matches!(source, RunSource::Array(..));
            if on_array != Some(array) {
                pass.set_pipeline(match array {
                    false => &self.sprite_glow_pipeline,
                    true => {
                        &self
                            .sprite_arrays
                            .as_ref()
                            .expect("made before the pass")
                            .glow
                    }
                });
                on_array = Some(array);
            }
            let group = match source {
                RunSource::Sprite(_, Some(t)) => self.cache.sprite_groups[t.index()]
                    .as_ref()
                    .expect("made before the pass"),
                RunSource::Array(_, a) => {
                    &self.cache.sprite_array_groups[a.index()]
                        .as_ref()
                        .expect("made before the pass")
                        .0
                }
                _ => &self.white_sprite_group,
            };
            pass.set_bind_group(1, group, &[]);
            let start = run.first as u64 * SPRITE_STRIDE;
            pass.set_vertex_buffer(
                0,
                instances.slice(start..start + run.count as u64 * SPRITE_STRIDE),
            );
            pass.draw(0..6, 0..run.count);
            draws += 1;
        }
        draws
    }
}
