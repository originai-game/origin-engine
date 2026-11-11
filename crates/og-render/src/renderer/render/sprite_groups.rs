//! The sprites' resources for the frame: every run's texture, normal map,
//! hook and array groups (the camera's, the views' and the probe faces'),
//! the GPU tile chunks' pipelines and draws, the sprite meshes' corners, and
//! the particle systems' texture groups, in that order.

use super::*;

pub(super) struct PrepareSpriteGroupsInputs<'a> {
    pub scene: &'a RenderScene,
    pub sprite_frame: &'a sprites::SpriteFrame,
    pub view_sprites: &'a [Vec<sprites::SpriteRun>],
    pub view_glow_runs: &'a [Vec<sprites::SpriteRun>],
    pub face_sprites: &'a [Vec<sprites::SpriteRun>],
    pub view_boards: &'a [sprites::Billboards],
    pub face_boards: &'a [sprites::Billboards],
    pub gpu_sprites: &'a sprites::GpuSprites,
    pub light_slots: &'a crate::light2d::LightSlots,
    pub lit_range: Option<(i32, i32)>,
    pub particle_frame: &'a particles_gpu::ParticleFrame,
}

impl Renderer {
    /// Makes the groups and buffers the frame's sprite runs, tiles and sprite meshes draw with.
    #[allow(clippy::too_many_lines)]
    pub(super) fn prepare_sprite_groups(
        &mut self,
        PrepareSpriteGroupsInputs {
            scene,
            sprite_frame,
            view_sprites,
            view_glow_runs,
            face_sprites,
            view_boards,
            face_boards,
            gpu_sprites,
            light_slots,
            lit_range,
            particle_frame,
        }: PrepareSpriteGroupsInputs<'_>,
    ) {
        let lit = |layer: i32, mask: u32| light_slots.lit(lit_range, layer, mask);
        for run in sprite_frame
            .runs
            .iter()
            .chain(&sprite_frame.glow_runs)
            .chain(view_sprites.iter().flatten())
            .chain(view_glow_runs.iter().flatten())
            .chain(face_sprites.iter().flatten())
            .chain(sprite_frame.billboards.iter().map(|b| &b.1))
            .chain(&sprite_frame.billboard_shadows)
            .chain(
                view_boards
                    .iter()
                    .chain(face_boards)
                    .flat_map(|(b, s)| b.iter().map(|b| &b.1).chain(s)),
            )
        {
            // A 2D particle system simulated in compute: as its source.
            let source = match run.source {
                RunSource::Particles(k) => gpu_sprites.ranges[k as usize].source,
                RunSource::Tiles(k) => RunSource::Sprite(
                    SpriteBlend::Alpha,
                    Some(
                        scene.tile_layers
                            [gpu_sprites.tiles.as_ref().expect("GPU tiles")[k as usize].layer]
                            .texture,
                    ),
                ),
                source => source,
            };
            match source {
                RunSource::Sprite(_, t) => {
                    self.sprite_group(t);
                    if let Some(t) = t {
                        self.sprite_normal_group(scene, t);
                    }
                }
                RunSource::Shaded(_, t, m) => {
                    self.sprite_group(t);
                    self.sprite_hooks_prepare(scene, m);
                }
                RunSource::Array(_, a) => self.sprite_array_group(scene, a),
                RunSource::Mesh(i) => {
                    let t = scene.sprite_meshes[i as usize].texture;
                    self.sprite_group(t);
                    if let Some(t) = t {
                        self.sprite_normal_group(scene, t);
                    }
                }
                _ => {}
            }
        }
        if let Some(draws) = &gpu_sprites.tiles
            && (!draws.is_empty() || self.tiles_gpu.is_some())
        {
            // Fork s60: the tile chunks' pipelines are drawn by the main pass.
            frame_pipelines(|| {
                let tiles = self.tiles_gpu.get_or_insert_with(|| {
                    tiles_gpu::Tiles::new(
                        &self.device,
                        [
                            &self.frame_layout,
                            &self.sprite_layout,
                            &self.sprite_fx_layout,
                        ],
                        self.samples,
                        self.info.without.features(),
                    )
                });
                tiles.prepare(
                    &self.device,
                    &self.queue,
                    scene,
                    draws,
                    self.sprite_normals
                        .as_ref()
                        .map(|n| [&self.frame_layout, &n.layout, &self.sprite_fx_layout]),
                    self.samples,
                );
            });
        }
        // The sprite meshes' corners (sprite_mesh.rs), lit as their layers' sprites.
        if !scene.sprite_meshes.is_empty() {
            let corners = sprite_mesh::vertices(scene, lit, self.sprite_skin_supported);
            let meshes = self.sprite_meshes.get_or_insert_with(|| {
                // Fork s60: drawn by the main pass, as the tiles are.
                frame_pipelines(|| {
                    sprite_mesh::SpriteMeshRenderer::new(
                        &self.device,
                        [
                            &self.frame_layout,
                            &self.sprite_layout,
                            &self.sprite_fx_layout,
                        ],
                        self.samples,
                        self.sprite_skin_supported,
                        self.info.without.features(),
                    )
                })
            });
            if let Some(skin) = &mut meshes.skin {
                skin.prepare(
                    &self.device,
                    &self.queue,
                    scene,
                    self.sprite_normals
                        .as_ref()
                        .map(|n| [&self.frame_layout, &n.layout, &self.sprite_fx_layout]),
                    self.samples,
                );
            }
            grow_and_write(
                &self.device,
                &self.queue,
                &mut meshes.buffer,
                "sprite meshes",
                bytemuck::cast_slice(&corners),
            );
        }
        for s in &particle_frame.systems {
            self.sprite_group(s.texture);
        }
    }
}
