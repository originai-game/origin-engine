//! The effect bind groups (`FxGroups`): the effects uniform, the AO and
//! planar reflection views, the decal atlas and buffers, the sprites' light
//! map and directions; rebuilt only when what they read changed.

use super::*;

impl Renderer {
    /// Remakes the effect bind groups when their key changed. `ao`: the AO
    /// target is drawn (AO, GI or contact shadows); `reflection`: the frame
    /// draws a planar reflection.
    pub(super) fn update_fx_groups(&mut self, scene: &RenderScene, ao: bool, reflection: bool) {
        // The effect bind groups, rebuilt only when what they read changed.
        let fx_key = (
            self.graph.generation,
            scene.id(),
            scene.effects.decal_atlas,
            self.lights.decal_buffers.generation,
            ao,
            scene.sprite.lighting_2d.is_some(),
            reflection,
        );
        if self.lights.fx_groups.as_ref().map(|g| g.key) != Some(fx_key) {
            let atlas_view = scene.effects.decal_atlas.map_or(&self.default_white, |t| {
                &self.cache.textures[t.index()].view
            });
            self.world.terrain.fx.bind(
                &self.device,
                atlas_view,
                &self.trilinear_sampler,
                &self.lights.decal_buffers,
            );
            let ao_view = if ao {
                self.graph.view("ao")
            } else {
                &self.post.white
            };
            // This frame's planar reflection, for the material shaders that
            // read it (pbr.wgsl `planar_reflection`; fx `reserved` says
            // whether it is there).
            let planar_view = if reflection {
                self.graph.view("reflection")
            } else {
                &self.post.white
            };
            let lightmap_view = if scene.sprite.lighting_2d.is_some() {
                self.graph.view("lightmap")
            } else {
                &self.default_white
            };
            // The lights' directions (normal-mapped receivers), black
            // (none) without.
            let lightdir_view = self.graph.try_view("lightdir").unwrap_or(&self.post.black);
            let sprite_group = |light: &wgpu::TextureView, dir, label: &str| {
                bind_group(
                    &self.device,
                    label,
                    &self.sprite.sprite_fx_layout,
                    &[
                        wgpu::BindingResource::TextureView(light),
                        wgpu::BindingResource::Sampler(&self.trilinear_sampler),
                        self.sprite.sprite_fx_buffer.as_entire_binding(),
                        wgpu::BindingResource::TextureView(dir),
                    ],
                )
            };
            let groups = FxGroups {
                key: fx_key,
                fx: fx::fx_group(
                    &self.device,
                    "fx",
                    &self.fx_layout,
                    &[
                        self.lights.fx_buffer.as_entire_binding(),
                        wgpu::BindingResource::TextureView(ao_view),
                        wgpu::BindingResource::TextureView(atlas_view),
                        wgpu::BindingResource::Sampler(&self.trilinear_sampler),
                        wgpu::BindingResource::TextureView(planar_view),
                    ][..if self.planar_slot() { 5 } else { 4 }],
                    &self.lights.decal_buffers,
                ),
                fx_lean: self.fx_lean_group(&self.lights.fx_buffer, ao_view, atlas_view),
                sprite: sprite_group(lightmap_view, lightdir_view, "sprite fx"),
                // The glow pass draws into a target that may alias the light
                // map: its sprites read a placeholder instead.
                glow: sprite_group(&self.default_white, &self.post.black, "sprite fx (glow)"),
            };
            self.lights.fx_groups = Some(groups);
        }
    }
}
