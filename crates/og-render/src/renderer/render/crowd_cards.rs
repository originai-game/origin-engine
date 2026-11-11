//! Crowd cards' inputs and atlases (crowd.rs **Impostors**,
//! `crowd_impostor.rs`).

use super::*;
use crate::crowd_impostor::{AtlasKey, BakeGpu, BakePart, CrowdImpostors};

impl Renderer {
    /// The frame's crowds' cards as the scatter renderer draws them: each
    /// card's sphere, atlas shape and draw, and its CPU records among
    /// `cpu_runs` (`(card, first, count)`).
    pub(super) fn crowd_card_inputs(
        frame: &crate::crowd::CrowdFrame,
        cpu_runs: &[(u32, u32, u32)],
    ) -> Vec<scatter::CardInput> {
        frame
            .cards
            .iter()
            .enumerate()
            .map(|(k, c)| {
                let (views, frames, _, elevation) = c.impostor.resolved();
                scatter::CardInput {
                    center: c.center,
                    radius: c.radius,
                    views,
                    frames,
                    elevation: elevation.to_radians(),
                    draw: frame.card_draw(k),
                    cpu: cpu_runs
                        .iter()
                        .find(|r| r.0 == k as u32)
                        .map(|r| (r.1, r.2)),
                    cast_shadows: c.cast_shadows,
                }
            })
            .collect()
    }

    /// Bakes each card's atlas the first frame it is needed (into its own
    /// submission, before the frame's) and gives each card its material
    /// group: the atlas and its parts' colours and roughness. Needs the
    /// palette's group (the baked clips).
    pub(super) fn prepare_crowd_cards(
        &mut self,
        scene: &RenderScene,
        frame: &crate::crowd::CrowdFrame,
        cards: &mut [scatter::CardBatch],
    ) {
        if cards.is_empty() {
            return;
        }
        let Some(palette) = self.instances.palette.bind_group.clone() else {
            return;
        };
        let imp = self.world.crowd_impostors.get_or_insert_with(|| {
            CrowdImpostors::new(&self.device, &self.material_layout, &self.palette_layout)
        });
        imp.begin_frame();
        let mut encoder = Encoder::untimed(self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("crowd impostors"),
            },
        ));
        for (card, batch) in frame.cards.iter().zip(cards.iter_mut()) {
            let clip = &scene.objects.baked_clips[card.impostor.clip as usize];
            let (views, frames, size, elevation) = card.impostor.resolved();
            let crowd = &scene.objects.crowds[card.crowd as usize];
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(bytemuck::cast_slice::<_, u8>(&clip.frames), &mut hasher);
            let maps: Vec<Option<usize>> = crowd
                .parts
                .iter()
                .map(|p| {
                    scene
                        .standard_material(p.material)
                        .and_then(|m| m.textures()[0])
                        .map(|h| h.index())
                })
                .collect();
            let key = AtlasKey {
                parts: card
                    .parts
                    .iter()
                    .zip(&maps)
                    .map(|(&(mesh, _), &map)| (mesh, self.cache.meshes[mesh].revision, map))
                    .collect(),
                clip: card.impostor.clip,
                generation: std::hash::Hasher::finish(&hasher),
                tiles: [views, frames, size, elevation.to_bits()],
                sphere: [
                    card.center.x.to_bits(),
                    card.center.y.to_bits(),
                    card.center.z.to_bits(),
                    card.radius.to_bits(),
                ],
            };
            let bake = BakeGpu {
                clip: [
                    self.instances.palette.baked_offset(card.impostor.clip) as u32,
                    clip.joints,
                    clip.frame_count() as u32,
                    0,
                ],
                tiles: [views, frames, size, 0],
                sphere: card.center.extend(card.radius).to_array(),
                view: [
                    elevation.to_radians().sin(),
                    elevation.to_radians().cos(),
                    0.0,
                    0.0,
                ],
            };
            let meshes = &self.cache.meshes;
            let skins = &self.cache.skins;
            let materials = &self.cache.materials;
            let parts: Vec<BakePart> = card
                .parts
                .iter()
                .filter_map(|&(mesh, material)| {
                    let m = &meshes[mesh];
                    Some(BakePart {
                        vertices: &m.vertices,
                        indices: &m.indices,
                        skin: skins[mesh].as_ref()?,
                        index_count: m.index_count,
                        material: &materials[material].bind_group,
                    })
                })
                .collect();
            let atlas = imp.atlas(
                &self.device,
                &self.queue,
                &mut encoder,
                &mut self.mips,
                &key,
                bake,
                &parts,
                &palette,
            );
            // Each part's colour (linear) and roughness, then the parts' count.
            let mut custom = [[0.0f32; 4]; crate::MAX_SHADER_PARAMS];
            for (slot, p) in custom.iter_mut().zip(&crowd.parts) {
                if let Some(m) = scene.standard_material(p.material) {
                    *slot = [m.color.x, m.color.y, m.color.z, m.roughness];
                }
            }
            custom[4][0] = crowd.parts.len() as f32;
            let bits: Vec<u32> = custom.iter().flatten().map(|f| f.to_bits()).collect();
            let group = atlas.groups.entry(bits).or_insert_with(|| {
                let mut uniform = material::uniform(&StandardMaterial::default(), false);
                uniform.custom = custom;
                uniform.previous_custom = custom;
                let factors = crate::buffer_init(
                    &self.device,
                    &self.queue,
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("crowd card"),
                        contents: bytemuck::bytes_of(&uniform),
                        usage: wgpu::BufferUsages::UNIFORM,
                    },
                );
                let white = wgpu::BindingResource::TextureView(&self.default_white);
                bind_group(
                    &self.device,
                    "crowd card",
                    &self.material_layout,
                    &[
                        factors.as_entire_binding(),
                        wgpu::BindingResource::TextureView(&atlas.albedo),
                        white.clone(),
                        white.clone(),
                        wgpu::BindingResource::TextureView(&atlas.data),
                        white.clone(),
                        white.clone(),
                        wgpu::BindingResource::Sampler(&self.material_sampler),
                        white.clone(),
                        white,
                    ],
                )
            });
            batch.group = Some(group.clone());
        }
        self.queue.submit(Some(encoder.finish()));
    }
}
