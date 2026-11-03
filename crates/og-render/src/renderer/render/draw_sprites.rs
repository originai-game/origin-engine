//! Draw sprites stage.

use super::*;

impl Renderer {
    pub(super) fn frame_draw_sprites(
        &self,
        pass: &mut wgpu::RenderPass,
        runs: &[sprites::SpriteRun],
        fx: &wgpu::BindGroup,
        fields: &[wgpu::BindGroup],
        view: bool,
        scene: &RenderScene,
    ) -> u32 {
        let mut draws = 0;
        pass.set_bind_group(2, fx, &[]);
        let mut current: Option<(bool, SpriteBlend, bool)> = None;
        let mut covered = false;
        for run in runs {
            // A 2D particle system simulated in compute draws its range of
            // its own buffer as its source draws.
            let (source, instances) = match run.source {
                RunSource::Particles(k) => (
                    self.particle_sprite_sources[k as usize],
                    &self.particle_sprite_buffer,
                ),
                source => (source, &self.sprite_buffer),
            };
            // A view keeps no HDR alpha: every run is tone mapped there.
            let tone_mapped = run.tone_mapped || view;
            if !tone_mapped && !covered {
                // The HDR alpha to 1 before the first untonemapped run.
                pass.set_pipeline(&self.sprite_cover_pipeline);
                pass.draw(0..3, 0..1);
                draws += 1;
                covered = true;
                current = None;
            }
            if let RunSource::Metaballs(i) = source
                && let Some(group) = self.metaballs.field_of(i).and_then(|f| fields.get(f))
            {
                self.metaballs.composite(pass, group, i);
                draws += 1;
                pass.set_bind_group(2, fx, &[]);
                current = None;
                continue;
            }
            if let RunSource::Strip(blend, texture) = source {
                pass.set_pipeline(
                    &self.ribbon_pipelines[usize::from(blend != ParticleBlend::Additive as u8)],
                );
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
                let stride = std::mem::size_of::<ribbons::RibbonVertex>() as u64;
                let start = run.first as u64 * stride;
                pass.set_vertex_buffer(
                    0,
                    self.ribbon_buffer
                        .slice(start..start + run.count as u64 * stride),
                );
                pass.draw(0..run.count, 0..1);
                draws += 1;
                // The sprite pipeline and its groups again.
                pass.set_bind_group(2, fx, &[]);
                current = None;
                continue;
            }
            if let RunSource::Mesh(i) = source {
                let mesh = &scene.sprite_meshes[i as usize];
                if let Some(group) = self.normal_group(scene, mesh.texture) {
                    let normals = self.sprite_normals.as_ref().expect("made with the group");
                    pass.set_pipeline(normals.mesh(mesh.blend));
                    pass.set_bind_group(1, group, &[]);
                    let start = run.first as u64 * sprite_mesh::MESH_STRIDE;
                    pass.set_vertex_buffer(
                        0,
                        self.sprite_meshes
                            .as_ref()
                            .expect("made before the pass")
                            .buffer
                            .slice(start..start + run.count as u64 * sprite_mesh::MESH_STRIDE),
                    );
                    pass.draw(0..run.count, 0..1);
                    draws += 1;
                    current = None;
                    continue;
                }
                pass.set_bind_group(
                    1,
                    match mesh.texture {
                        None => &self.white_sprite_group,
                        Some(t) => self.cache.sprite_groups[t.index()]
                            .as_ref()
                            .expect("made before the pass"),
                    },
                    &[],
                );
                self.sprite_meshes
                    .as_ref()
                    .expect("made before the pass")
                    .draw(pass, mesh.blend, run.first, run.count);
                draws += 1;
                current = None;
                continue;
            }
            if let RunSource::Array(blend, a) = source {
                let arrays = self.sprite_arrays.as_ref().expect("made before the pass");
                pass.set_pipeline(arrays.pipeline(blend, tone_mapped));
                let (group, _) = self.cache.sprite_array_groups[a.index()]
                    .as_ref()
                    .expect("made before the pass");
                pass.set_bind_group(1, group, &[]);
                let start = run.first as u64 * SPRITE_STRIDE;
                pass.set_vertex_buffer(
                    0,
                    instances.slice(start..start + run.count as u64 * SPRITE_STRIDE),
                );
                pass.draw(0..6, 0..run.count);
                draws += 1;
                current = None;
                continue;
            }
            if let RunSource::Sprite(blend, t) = source
                && let Some(group) = self.normal_group(scene, t)
            {
                let normals = self.sprite_normals.as_ref().expect("made with the group");
                pass.set_pipeline(normals.sprite(blend, tone_mapped));
                pass.set_bind_group(1, group, &[]);
                let start = run.first as u64 * SPRITE_STRIDE;
                pass.set_vertex_buffer(
                    0,
                    instances.slice(start..start + run.count as u64 * SPRITE_STRIDE),
                );
                pass.draw(0..6, 0..run.count);
                draws += 1;
                current = None;
                continue;
            }
            let (msdf, blend, group) = match source {
                RunSource::Strip(..)
                | RunSource::Metaballs(_)
                | RunSource::Mesh(_)
                | RunSource::Array(..)
                | RunSource::Particles(_) => {
                    unreachable!("drawn above")
                }
                RunSource::Sprite(blend, None) => (false, blend, &self.white_sprite_group),
                RunSource::Sprite(blend, Some(t)) => (
                    false,
                    blend,
                    self.cache.sprite_groups[t.index()]
                        .as_ref()
                        .expect("made before the pass"),
                ),
                RunSource::Font(f) => (
                    true,
                    SpriteBlend::Alpha,
                    &self.cache.fonts[f.index()].bind_group,
                ),
            };
            if current != Some((msdf, blend, tone_mapped)) {
                pass.set_pipeline(match (msdf, blend, tone_mapped) {
                    (true, _, _) => &self.msdf_pipeline,
                    (false, SpriteBlend::Alpha, false) => &self.sprite_untonemapped_pipeline,
                    (false, SpriteBlend::Alpha, true) => &self.sprite_pipeline,
                    (false, SpriteBlend::Additive, _) => &self.sprite_additive_pipeline,
                });
                current = Some((msdf, blend, tone_mapped));
            }
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
