//! The sprite, normal-map and ribbon bind groups a frame's batches draw with.

use super::*;

impl Renderer {
    /// Texture `t`'s normal-mapped group (sprite_normals.rs) when 2D
    /// lighting is on and it has a normal map, made or made again when
    /// what it reads changed; the pipelines on first use.
    /// Material `m`'s sprite pipelines and parameter group
    /// (`sprite_hooks.rs`).
    pub(crate) fn sprite_hooks_prepare(&mut self, scene: &RenderScene, m: MaterialHandle) {
        let Some((shader, params)) = scene.sprite_shader(m) else {
            return;
        };
        let hooks = self.sprite.sprite_hooks.get_or_insert_with(|| {
            sprite_hooks::SpriteHooks::new(
                &self.device,
                [
                    &self.frame_layout,
                    &self.sprite.sprite_layout,
                    &self.sprite.sprite_fx_layout,
                ],
                self.samples,
                self.info.without.features(),
            )
        });
        hooks.prepare_shader(
            &self.device,
            shader.index(),
            &scene.shaders()[shader.index()],
        );
        hooks.prepare_material(&self.device, &self.queue, m.index(), params);
    }

    pub(crate) fn sprite_normal_group(&mut self, scene: &RenderScene, t: TextureHandle) {
        let Some(n) = scene
            .texture_normal(t)
            .filter(|_| scene.sprite.lighting_2d.is_some())
        else {
            return;
        };
        let (i, ni) = (t.index(), n.index());
        if ni >= self.cache.textures.len() {
            return;
        }
        let filter = self.cache.filters.get(i).copied().unwrap_or_default();
        let key = (
            self.cache.texture_revisions[i],
            ni,
            self.cache.texture_revisions[ni],
            filter.is_nearest(),
            filter == TextureFilter::CleanEdge,
        );
        let normals = self.sprite.sprite_normals.get_or_insert_with(|| {
            sprite_normals::SpriteNormals::new(
                &self.device,
                [&self.frame_layout, &self.sprite.sprite_fx_layout],
                self.samples,
            )
        });
        if normals.groups.len() <= i {
            normals.groups.resize_with(i + 1, || None);
        }
        if normals.groups[i].as_ref().is_some_and(|(_, k)| *k == key) {
            return;
        }
        let params = [0.0, 0.0, 0.0, key.4 as u32 as f32];
        let buffer = crate::buffer_init(
            &self.device,
            &self.queue,
            &wgpu::util::BufferInitDescriptor {
                label: Some("sprite params (normal map)"),
                contents: bytemuck::bytes_of(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            },
        );
        let group = bind_group(
            &self.device,
            "sprite (normal map)",
            &normals.layout,
            &[
                wgpu::BindingResource::TextureView(&self.cache.textures[i].view),
                wgpu::BindingResource::Sampler(if key.3 {
                    &self.sprite.sprite_sampler_nearest
                } else {
                    &self.sprite.sprite_sampler
                }),
                buffer.as_entire_binding(),
                wgpu::BindingResource::TextureView(&self.cache.textures[ni].view),
            ],
        );
        normals.groups[i] = Some((group, key));
    }

    /// Texture `t`'s normal-mapped group if this frame draws it so.
    pub(crate) fn normal_group(
        &self,
        scene: &RenderScene,
        t: Option<TextureHandle>,
    ) -> Option<&wgpu::BindGroup> {
        let t = t?;
        scene.sprite.lighting_2d?;
        scene.texture_normal(t)?;
        self.sprite.sprite_normals.as_ref()?.group(t.index())
    }

    pub(crate) fn sprite_group(&mut self, texture: Option<TextureHandle>) -> &wgpu::BindGroup {
        let Some(t) = texture else {
            return &self.sprite.white_sprite_group;
        };
        let filter = self
            .cache
            .filters
            .get(t.index())
            .copied()
            .unwrap_or_default();
        let slot = &mut self.cache.sprite_groups[t.index()];
        if slot.is_none() {
            *slot = Some(sprite_bind_group(
                &self.device,
                &self.queue,
                &self.sprite.sprite_layout,
                &self.cache.textures[t.index()].view,
                if filter.is_nearest() {
                    &self.sprite.sprite_sampler_nearest
                } else {
                    &self.sprite.sprite_sampler
                },
                // sprite.wgsl's `msdf.w`: 1 samples through cleanEdge.
                [
                    0.0,
                    0.0,
                    0.0,
                    (filter == TextureFilter::CleanEdge) as u32 as f32,
                ],
            ));
        }
        slot.as_ref().expect("made above")
    }

    pub(crate) fn ribbon_group(&mut self, texture: Option<TextureHandle>) {
        let Some(t) = texture else {
            return;
        };
        if self.cache.ribbon_groups[t.index()].is_none() {
            self.cache.ribbon_groups[t.index()] = Some(sprite_bind_group(
                &self.device,
                &self.queue,
                &self.sprite.sprite_layout,
                &self.cache.textures[t.index()].view,
                &self.strips.ribbon_sampler,
                [0.0; 4],
            ));
        }
    }
}
