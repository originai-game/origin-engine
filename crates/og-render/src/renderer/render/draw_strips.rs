//! Draw strips stage.

use super::*;

/// Which of the main camera's strips a draw takes: every one, the alpha
/// ones into the OIT pass (oit.rs), or the additive ones after its resolve.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum StripPart {
    All,
    Oit(oit::Oit),
    Additive,
}

impl Renderer {
    pub(super) fn frame_draw_strips(
        &self,
        pass: &mut wgpu::RenderPass,
        runs: &[ribbons::RibbonRun],
        diagnostic: bool,
        part: StripPart,
    ) -> u32 {
        let mut draws = 0;
        for run in runs.iter().filter(|r| r.layer.is_none()) {
            let blend = if diagnostic {
                ParticleBlend::Alpha
            } else {
                run.blend
            };
            let alpha = blend == ParticleBlend::Alpha;
            let pipeline = match part {
                StripPart::All => None,
                StripPart::Additive if alpha => continue,
                StripPart::Additive => None,
                StripPart::Oit(_) if !alpha => continue,
                StripPart::Oit(mode) => {
                    let source = if run.points && self.strips.gpu_trails.is_some() {
                        oit::Source::Strip
                    } else {
                        oit::Source::Ribbon
                    };
                    match self.oit_pipeline(source, mode) {
                        Some(p) => Some(p),
                        None => continue,
                    }
                }
            };
            self.draw_strip_run(
                pass,
                pipeline,
                blend as u8,
                run.texture,
                run.points,
                run.first,
                run.count,
                run.effect,
            );
            draws += 1;
        }
        draws
    }

    /// One run of strips (`RibbonRun`, or a layered 2D trail's
    /// `RunSource::Strip`): from the strip points (`points`, faced in the
    /// vertex shader; group 2 is theirs) or the strip vertices.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_strip_run(
        &self,
        pass: &mut wgpu::RenderPass,
        // An OIT pipeline in place of the run's own (oit.rs).
        pipeline: Option<&wgpu::RenderPipeline>,
        blend: u8,
        texture: Option<TextureHandle>,
        points: bool,
        first: u32,
        count: u32,
        // The effect material it draws with (effect.rs).
        effect: Option<crate::EffectMaterialHandle>,
    ) {
        let (group, offset) = self.particles.gpu.effects.bind(effect);
        pass.set_bind_group(3, group, &[offset]);
        let alpha = usize::from(blend != ParticleBlend::Additive as u8);
        let gpu = self.strips.gpu_trails.as_ref().filter(|_| points);
        pass.set_pipeline(match (pipeline, gpu) {
            (Some(p), _) => p,
            (None, Some(g)) => &g.pipelines[alpha],
            (None, None) => &self.strips.ribbon_pipelines[alpha],
        });
        pass.set_bind_group(
            1,
            match texture {
                None => &self.white_sprite_group,
                Some(t) => self.cache.ribbon_groups[t.index()]
                    .as_ref()
                    .expect("made before the pass"),
            },
            &[],
        );
        match gpu {
            Some(g) => {
                pass.set_bind_group(2, &g.draw_group, &[]);
                // Six vertices a point: its segment to the next.
                pass.draw(first * 6..(first + count) * 6, 0..1);
            }
            None => {
                pass.set_bind_group(2, &self.particles.gpu.effects.empty_group, &[]);
                let stride = std::mem::size_of::<ribbons::RibbonVertex>() as u64;
                let start = first as u64 * stride;
                pass.set_vertex_buffer(
                    0,
                    self.strips
                        .ribbon_buffer
                        .slice(start..start + count as u64 * stride),
                );
                pass.draw(0..count, 0..1);
            }
        }
    }
}
