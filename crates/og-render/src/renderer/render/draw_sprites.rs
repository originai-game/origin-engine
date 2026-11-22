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
        // The main pass's plate or authored sky set the HDR alpha already.
        let mut covered =
            !view && (self.plate_group(scene).is_some() || self.authored_environment(scene));
        for run in runs {
            // A 2D particle system simulated in compute draws its range of
            // its own buffer as its source draws.
            let (source, instances) = match run.source {
                RunSource::Particles(k) => (
                    self.particles.sprite_sources[k as usize],
                    &self.particles.sprite_buffer,
                ),
                source => (source, &self.sprite.sprite_buffer),
            };
            // A view keeps no HDR alpha: every run is tone mapped there.
            let tone_mapped = run.tone_mapped || view;
            if !tone_mapped && !covered {
                // The HDR alpha to 1 before the first untonemapped run.
                let Some(cover) = self.sprite.sprite_cover_pipeline.get() else {
                    continue;
                };
                pass.set_pipeline(cover);
                pass.draw(0..3, 0..1);
                draws += 1;
                covered = true;
                current = None;
            }
            if let RunSource::Metaballs(i) = source
                && let Some(group) = self.world.metaballs.field_of(i).and_then(|f| fields.get(f))
            {
                self.world.metaballs.composite(pass, group, i);
                draws += 1;
                pass.set_bind_group(2, fx, &[]);
                current = None;
                continue;
            }
            if let RunSource::Strip(blend, texture, points) = source {
                self.draw_strip_run(
                    pass, None, blend, texture, points, run.first, run.count, None,
                );
                draws += 1;
                // The sprite pipeline and its groups again.
                pass.set_bind_group(2, fx, &[]);
                current = None;
                continue;
            }
            if let RunSource::Tiles(i) = source {
                let tiles = self
                    .sprite
                    .tiles_gpu
                    .as_ref()
                    .expect("prepared before draw");
                let texture = tiles.textures[i as usize];
                let normal = self.normal_group(scene, Some(texture));
                pass.set_bind_group(
                    1,
                    normal.unwrap_or_else(|| {
                        self.cache.sprite_groups[texture.index()]
                            .as_ref()
                            .expect("prepared")
                    }),
                    &[],
                );
                tiles.draw(pass, i, run.count, normal.is_some());
                draws += 1;
                current = None;
                continue;
            }
            if let RunSource::Mesh(i) = source {
                let mesh = &scene.sprite.sprite_meshes[i as usize];
                if let Some(group) = self.normal_group(scene, mesh.texture) {
                    let normals = self
                        .sprite
                        .sprite_normals
                        .as_ref()
                        .expect("made with the group");
                    let meshes = self
                        .sprite
                        .sprite_meshes
                        .as_ref()
                        .expect("made before the pass");
                    if let Some(skin) = &meshes.skin {
                        skin.bind(pass, mesh.blend, true);
                    } else {
                        pass.set_pipeline(normals.mesh(mesh.blend));
                    }
                    pass.set_bind_group(1, group, &[]);
                    let start = run.first as u64 * sprite_mesh::MESH_STRIDE;
                    pass.set_vertex_buffer(
                        0,
                        self.sprite
                            .sprite_meshes
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
                        None => &self.sprite.white_sprite_group,
                        Some(t) => self.cache.sprite_groups[t.index()]
                            .as_ref()
                            .expect("made before the pass"),
                    },
                    &[],
                );
                self.sprite
                    .sprite_meshes
                    .as_ref()
                    .expect("made before the pass")
                    .draw(pass, mesh.blend, run.first, run.count);
                draws += 1;
                current = None;
                continue;
            }
            if let RunSource::Array(blend, a) = source {
                let arrays = self
                    .sprite
                    .sprite_arrays
                    .as_ref()
                    .expect("made before the pass");
                let Some(pipeline) = arrays.pipeline(blend, tone_mapped) else {
                    continue;
                };
                pass.set_pipeline(pipeline);
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
            // A sprite drawn with a material shader (prepared before the
            // pass); while its pipelines are not ready, or when the device
            // refused them, it draws plain.
            let source = match source {
                RunSource::Shaded(blend, t, m) => {
                    let hooks = self.sprite.sprite_hooks.as_ref();
                    let shader = scene.sprite_shader(m).map(|(s, _)| s.index());
                    let pipeline = hooks
                        .zip(shader)
                        .and_then(|(h, s)| h.pipeline(s, blend, tone_mapped));
                    if let (Some(pipeline), Some(params)) =
                        (pipeline, hooks.and_then(|h| h.group(m.index())))
                    {
                        pass.set_pipeline(pipeline);
                        pass.set_bind_group(
                            1,
                            match t {
                                None => &self.sprite.white_sprite_group,
                                Some(t) => self.cache.sprite_groups[t.index()]
                                    .as_ref()
                                    .expect("made before the pass"),
                            },
                            &[],
                        );
                        pass.set_bind_group(3, params, &[]);
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
                    RunSource::Sprite(blend, t)
                }
                source => source,
            };
            if let RunSource::Sprite(blend, t) = source
                && let Some(group) = self.normal_group(scene, t)
            {
                let normals = self
                    .sprite
                    .sprite_normals
                    .as_ref()
                    .expect("made with the group");
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
                | RunSource::Tiles(_)
                | RunSource::Array(..)
                | RunSource::Shaded(..)
                | RunSource::Particles(_) => {
                    unreachable!("drawn above")
                }
                RunSource::Sprite(blend, None) => (false, blend, &self.sprite.white_sprite_group),
                RunSource::Sprite(blend, Some(t)) => (
                    false,
                    blend,
                    self.cache.sprite_groups[t.index()]
                        .as_ref()
                        .expect("made before the pass"),
                ),
                RunSource::Glyphs(tier, page) => {
                    let Some(group) = self.paint.glyph_group(tier, page) else {
                        continue;
                    };
                    // A colour page's glyphs are images: the sprite pipeline.
                    let colour = tier == og_draw::glyphs::GlyphTier::Color;
                    (!colour, SpriteBlend::Alpha, group)
                }
            };
            if current != Some((msdf, blend, tone_mapped)) {
                let Some(pipeline) = match (msdf, blend, tone_mapped) {
                    (true, _, _) => &self.sprite.msdf_pipeline,
                    (false, SpriteBlend::Alpha, false) => &self.sprite.sprite_untonemapped_pipeline,
                    (false, SpriteBlend::Alpha, true) => &self.sprite.sprite_pipeline,
                    (false, SpriteBlend::Additive, _) => &self.sprite.sprite_additive_pipeline,
                }
                .get() else {
                    continue;
                };
                pass.set_pipeline(pipeline);
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
