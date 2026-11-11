//! Scene resource storage and mutation.

use super::*;

mod colors;
mod effects;
mod environment_maps;
mod fonts;
mod materials;
mod meshes;
mod terrain;
mod textures;

impl Default for RenderScene {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderScene {
    /// Whether the renderer captures the sky as it draws it into the
    /// image-based light (the live sky, `sky_capture.rs`): the per-pixel
    /// procedural sky with no environment map lighting the scene. Its
    /// clouds are then in that light already ([`crate::Clouds::light_scene`]).
    pub fn sky_captured(&self) -> bool {
        self.environment_map.is_none()
            && matches!(self.environment.background, Background::Procedural { .. })
    }

    pub fn new() -> Self {
        RenderScene {
            id: NEXT_SCENE_ID.fetch_add(1, Ordering::Relaxed),
            meshes: Vec::new(),
            mesh_revisions: Vec::new(),
            mesh_encodings: Vec::new(),
            mesh_clusters: Vec::new(),
            mesh_foliage: Vec::new(),
            mesh_surfaces: Vec::new(),
            mesh_visual: Vec::new(),
            cloth_subdivisions: 0,
            textures: Vec::new(),
            texture_revisions: Vec::new(),
            materials: Vec::new(),
            material_shaders: Vec::new(),
            shaders: Vec::new(),
            gltf_materials: Vec::new(),
            material_revisions: Vec::new(),
            material_serial: 0,
            material_aliases: Vec::new(),
            material_copies: Vec::new(),
            fonts: Vec::new(),
            font_revisions: Vec::new(),
            instances: Vec::new(),
            camera: Camera::default(),
            hemisphere: ([0.0; 3], [0.0; 3], 0.0),
            light: DirectionalLight::default(),
            lights: Vec::new(),
            environment: Environment::default(),
            environment_map: None,
            look: SceneLook {
                post: PostEffects::default(),
                stylise: crate::Stylise::default(),
                posts: Vec::new(),
                post_params: Vec::new(),
                custom_post: Vec::new(),
                color_filter: None,
            },
            sprites: Vec::new(),
            texts: Vec::new(),
            debug: DebugView::None,
            skins: Vec::new(),
            morphs: Vec::new(),
            emitters: Vec::new(),
            effect_materials: Vec::new(),
            luts: Vec::new(),
            lut_revisions: Vec::new(),
            palettes: Vec::new(),
            conformed: Vec::new(),
            conformed_revision: 0,
            palette_revisions: Vec::new(),
            environment_maps: Vec::new(),
            environment_map_revisions: Vec::new(),
            environment_equirects: Vec::new(),
            texture_arrays: Vec::new(),
            texture_array_revisions: Vec::new(),
            texture_array_filters: Vec::new(),
            trail_styles: Vec::new(),
            terrain_data: Vec::new(),
            terrain_revisions: Vec::new(),
            terrain_edits: Vec::new(),
            instance_layers: Vec::new(),
            skinned_layers: Vec::new(),
            sprite_layers: Vec::new(),
            sprite_light_masks: Vec::new(),
            sprite_facing: Vec::new(),
            text_facing: Vec::new(),
            line_layers: Vec::new(),
            point_layers: Vec::new(),
            particle_layers: Vec::new(),
            camera_layers: 1,
            meshlets: true,
            lod_bias: 1.0,
            lod_debug: false,
            visibility_buffer: true,
            ui: SceneUi {
                overlay: None,
                ui_vectors: Vec::new(),
                ui_materials: Vec::new(),
            },
            origin: glam::Vec3::ZERO,
            views: Vec::new(),
            tick: 0,
            tick_rate: 60.0,
            interpolation: None,
            particles: Vec::new(),
            particle_history: Default::default(),
            colliders: Vec::new(),
            debris: Vec::new(),
            hosted_particles: Vec::new(),
            trails: Vec::new(),
            ribbons: Vec::new(),
            lines: Vec::new(),
            points: Vec::new(),
            css_height: 0.0,
            beams: Vec::new(),
            effect_meshes: Vec::new(),
            decals: Vec::new(),
            decal_atlas: None,
            lighting_2d: None,
            lights_2d: Vec::new(),
            occluders_2d: Vec::new(),
            sprite_fx: Vec::new(),
            sprite_meshes: Vec::new(),
            reflection_probes: Vec::new(),
            shake: CameraShake::default(),
            previous_transforms: Vec::new(),
            instance_levels: Vec::new(),
            previous_joint_palettes: Vec::new(),
            previous_morph_weights: Vec::new(),
            baked_clips: Vec::new(),
            crowds: Vec::new(),
            deformed_vertices: Vec::new(),
            previous_deformed_vertices: Vec::new(),
            camera_cut: false,
            joint_palettes: Vec::new(),
            skinned_instances: Vec::new(),
            atmosphere: SceneAtmosphere {
                fog: None,
                volumetric_fog: None,
                clouds: None,
                aurora: None,
                adaptation: 1.0,
                surface_weather: None,
                wind: None,
                wind_motors: Vec::new(),
                grass_pushers: Vec::new(),
            },
            quality: crate::RenderQuality::default(),
            baked: crate::BakedLighting::default(),
            light_volumes: Vec::new(),
            light_cookies: Vec::new(),
            water: Vec::new(),
            terrains: Vec::new(),
            terrain_stamps: Vec::new(),
            scatters: Vec::new(),
            grass: Vec::new(),
            impostors: Vec::new(),
            water_2d: Vec::new(),
            metaballs: Vec::new(),
            fluids_3d: Vec::new(),
            sprite_order: Vec::new(),
            text_outlines: Vec::new(),
            tile_layers: Vec::new(),
            trail_layers: Vec::new(),
            sprite_arrays: Vec::new(),
            sprite_shadows: Vec::new(),
            sprite_outlines: Vec::new(),
            sprite_shading: Vec::new(),
            texture_filters: Vec::new(),
            texture_sizes: Default::default(),
            texture_normals: Vec::new(),
            paletted: std::collections::HashMap::new(),
            mesh_slots: Slots::default(),
            texture_slots: Slots::default(),
            stream_frames: Default::default(),
            stream_serial: 0,
            #[cfg(target_arch = "wasm32")]
            stream_external: Default::default(),
            material_slots: Slots::default(),
            font_slots: Slots::default(),
            environment_map_slots: Slots::default(),
            texture_array_slots: Slots::default(),
            palette_slots: Slots::default(),
            lut_slots: Slots::default(),
        }
    }

    /// Adds one sprite (the same as pushing onto [`RenderScene::sprites`]).
    pub fn add_sprite(&mut self, sprite: Sprite) {
        self.sprites.push(sprite);
    }

    /// Every per-frame reference to a removed mesh, material, texture,
    /// font, environment map, texture array, palette or LUT: instances,
    /// skinned instances, sprites (and the arrays they draw from), texts,
    /// the overlay's, the environment map, the colour grading's LUT and the
    /// stylisation's palette (nothing to look at until something was
    /// removed).
    pub(crate) fn stale_handle(&self) -> Option<String> {
        let (m, t, mat, f) = (
            &self.mesh_slots,
            &self.texture_slots,
            &self.material_slots,
            &self.font_slots,
        );
        if !(m.any_removed
            || t.any_removed
            || mat.any_removed
            || f.any_removed
            || self.environment_map_slots.any_removed
            || self.texture_array_slots.any_removed
            || self.palette_slots.any_removed
            || self.lut_slots.any_removed)
        {
            return None;
        }
        if let Some(e) = self.environment_map
            && !self.contains_environment_map(e.map)
        {
            return Some(format!("the environment map {} was removed", e.map.index()));
        }
        if let Some(g) = self.look.post.color_grading
            && !self.contains_lut(g.lut)
        {
            return Some(format!(
                "the colour grading's lut {} was removed",
                g.lut.index()
            ));
        }
        if let Some(p) = self.look.stylise.quantize.and_then(|q| q.palette)
            && !self.contains_palette(p)
        {
            return Some(format!(
                "the stylisation's palette {} was removed",
                p.index()
            ));
        }
        for s in &self.sprite_arrays {
            if !self.contains_texture_array(s.array) {
                return Some(format!(
                    "sprite {}: texture array {} was removed",
                    s.sprite,
                    s.array.index()
                ));
            }
        }
        for s in self.ui.overlay.iter().flat_map(|o| &o.sprite_arrays) {
            if !self.contains_texture_array(s.array) {
                return Some(format!(
                    "overlay sprite {}: texture array {} was removed",
                    s.sprite,
                    s.array.index()
                ));
            }
        }
        let draws = self
            .instances
            .iter()
            .map(|i| (i.mesh, i.material))
            .chain(self.skinned_instances.iter().map(|i| (i.mesh, i.material)));
        for (k, (mesh, material)) in draws.enumerate() {
            if !m.live(mesh.index(), mesh.generation()) {
                return Some(format!("instance {k}: mesh {} was removed", mesh.index()));
            }
            if !mat.live(material.index(), material.generation()) {
                return Some(format!(
                    "instance {k}: material {} was removed",
                    material.index()
                ));
            }
        }
        let overlay = self.ui.overlay.as_ref();
        let sprites = self
            .sprites
            .iter()
            .chain(overlay.into_iter().flat_map(|o| o.sprites.iter()));
        for (k, s) in sprites.enumerate() {
            if let Some(h) = s.texture
                && !t.live(h.index(), h.generation())
            {
                return Some(format!("sprite {k}: texture {} was removed", h.index()));
            }
        }
        let texts = self
            .texts
            .iter()
            .chain(overlay.into_iter().flat_map(|o| o.texts.iter()));
        for (k, text) in texts.enumerate() {
            if !f.live(text.font.index(), text.font.generation()) {
                return Some(format!("text {k}: font {} was removed", text.font.index()));
            }
        }
        for (k, font) in self.fonts.iter().enumerate() {
            if f.live(k, f.generation(k)) && !t.live(font.atlas.index(), font.atlas.generation()) {
                return Some(format!(
                    "font {k}: its atlas texture {} was removed",
                    font.atlas.index()
                ));
            }
        }
        None
    }

    /// The handle of the scene's `index`-th mesh (meshes are numbered in
    /// the order they were added).
    pub fn mesh_handle(&self, index: usize) -> Option<MeshHandle> {
        let generation = self.mesh_slots.generation(index);
        self.mesh_slots
            .live(index, generation)
            .then(|| MeshHandle::from_parts(index, generation))
    }

    /// How many times the mesh at `index` was replaced (or removed).
    /// Bumped by every [`Self::set_mesh`] of the mesh at `index`.
    pub fn mesh_revision(&self, index: usize) -> u64 {
        self.mesh_revisions.get(index).copied().unwrap_or(0)
    }

    /// How many times the texture at `index` was replaced (or removed).
    pub fn texture_revision(&self, index: usize) -> u64 {
        self.texture_revisions.get(index).copied().unwrap_or(0)
    }

    /// How many times the terrain at `index` was replaced or edited.
    pub fn terrain_revision(&self, index: usize) -> u64 {
        self.terrain_revisions.get(index).copied().unwrap_or(0)
    }

    pub fn mesh(&self, handle: MeshHandle) -> Option<&MeshData> {
        if !self.contains_mesh(handle) {
            return None;
        }
        self.meshes.get(handle.index())
    }

    /// An RGBA8 texture (`None` for a [`TextureUpload`] or a foreign handle).
    pub fn texture(&self, handle: TextureHandle) -> Option<&TextureData> {
        match self.textures.get(handle.index())? {
            TextureSource::Rgba(t) => Some(t),
            TextureSource::Encoded(_) | TextureSource::Target(_) => None,
        }
    }

    /// How a texture's texels are sampled: decoded from sRGB or as stored
    /// (a render target's are linear); `None` for a foreign handle.
    pub fn texture_color_space(&self, handle: TextureHandle) -> Option<ColorSpace> {
        Some(match self.textures.get(handle.index())? {
            TextureSource::Rgba(t) => t.color_space,
            TextureSource::Encoded(u) => u.desc.color_space,
            TextureSource::Target(_) => ColorSpace::Linear,
        })
    }

    /// A pre-encoded texture (`None` for an RGBA8 one or a foreign handle).
    pub fn texture_upload(&self, handle: TextureHandle) -> Option<&TextureUpload> {
        match self.textures.get(handle.index())? {
            TextureSource::Encoded(t) => Some(t),
            TextureSource::Rgba(_) | TextureSource::Target(_) => None,
        }
    }

    /// Width and height of any texture (a streamed one's whole chain's,
    /// [`RenderScene::set_texture_size`], whatever it holds now).
    pub fn texture_size(&self, handle: TextureHandle) -> Option<(u32, u32)> {
        let size = self.textures.get(handle.index()).map(TextureSource::size)?;
        Some(self.texture_sizes.get(&handle).copied().unwrap_or(size))
    }

    /// The size [`RenderScene::texture_size`] reports for `handle` whatever
    /// levels it holds (`None`: its own): a texture streamed by its mips
    /// (G3-3) reports its whole chain's while it holds its tail, so a
    /// sprite, an image or a layout sized from it does not change when
    /// the levels above arrive. False for a bad handle.
    pub fn set_texture_size(&mut self, handle: TextureHandle, size: Option<(u32, u32)>) -> bool {
        if !self.texture_slots.live(handle.index(), handle.generation()) {
            return false;
        }
        match size {
            Some(s) => self.texture_sizes.insert(handle, s),
            None => self.texture_sizes.remove(&handle),
        };
        true
    }

    /// Every texture `material` samples: its maps and its physical
    /// extensions' (texture arrays aside). Empty for a bad handle.
    pub fn material_textures(&self, material: MaterialHandle) -> Vec<TextureHandle> {
        self.standard_material(material)
            .map(|m| {
                m.textures()
                    .into_iter()
                    .chain(m.physical_textures())
                    .flatten()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The material as added with [`RenderScene::add_material`] or
    /// [`RenderScene::set_material`]; `None` for one that is a
    /// [`StandardMaterial`] (read it with [`RenderScene::standard_material`]).
    pub fn material(&self, handle: MaterialHandle) -> Option<&Material> {
        self.gltf_materials.get(handle.index())?.as_ref()
    }

    /// Any material, in the form it is drawn.
    pub fn standard_material(&self, handle: MaterialHandle) -> Option<&StandardMaterial> {
        self.materials.get(handle.index())
    }

    /// Adds a material shader (`hooks.rs`). The cook validates shader
    /// assets; one added here directly is compiled when first drawn.
    pub fn add_shader(&mut self, shader: crate::MaterialShader) -> ShaderHandle {
        self.shaders.push(shader);
        ShaderHandle(self.shaders.len() as u32 - 1)
    }

    pub fn shaders(&self) -> &[crate::MaterialShader] {
        &self.shaders
    }

    /// Draws `material` with `shader` (`None`: the built-in hooks again),
    /// its parameters at the shader's defaults. False for a bad handle.
    pub fn set_material_shader(
        &mut self,
        material: MaterialHandle,
        shader: Option<ShaderHandle>,
    ) -> bool {
        let i = material.index();
        if !self.material_slots.live(i, material.generation())
            || shader.is_some_and(|s| s.index() >= self.shaders.len())
        {
            return false;
        }
        if self.material_shaders.len() <= i {
            self.material_shaders.resize(i + 1, None);
        }
        self.material_shaders[i] = shader.map(|s| (s, self.shaders[s.index()].defaults()));
        self.material_revisions[i] += 1;
        self.material_serial += 1;
        self.restamp_aliases(i);
        true
    }

    /// Sets parameter `name` of `material`'s shader (as many numbers as its
    /// type takes; missing ones are 0). False when the material has no
    /// shader or the shader no such parameter. Leaves the material's
    /// revision: the renderer writes changed parameters into the material's
    /// uniform in place (a new buffer and bind group per change cost the
    /// browser's GPU process about 1 ms each). A change parts the material
    /// from the copies it batches with ([`RenderScene::alias_material`])
    /// until they hold the same parameters again.
    pub fn set_shader_param(
        &mut self,
        material: MaterialHandle,
        name: &str,
        value: &[f32],
    ) -> bool {
        let i = material.index();
        let Some(Some((shader, params))) = self.material_shaders.get_mut(i) else {
            return false;
        };
        let Some((slot, kind)) = self.shaders[shader.index()].param(name) else {
            return false;
        };
        let mut v = [0.0; 4];
        for (d, s) in v.iter_mut().zip(value.iter().take(kind.count())) {
            *d = *s;
        }
        if params[slot] == v {
            return true;
        }
        params[slot] = v;
        self.material_serial += 1;
        self.restamp_aliases(i);
        true
    }

    /// `material`'s shader and parameters as a mesh draws it: none for a
    /// shader of [`crate::ShaderMode::Sprite`], which only sprites run.
    pub fn material_shader(
        &self,
        material: MaterialHandle,
    ) -> Option<(ShaderHandle, [[f32; 4]; crate::MAX_SHADER_PARAMS])> {
        self.material_shaders
            .get(material.index())
            .copied()
            .flatten()
            .filter(|(s, _)| self.shaders[s.index()].mode != crate::ShaderMode::Sprite)
    }

    /// `material`'s shader and parameters as a sprite draws it
    /// ([`crate::SpriteShading`]): a shader of hooks or of
    /// [`crate::ShaderMode::Sprite`], none for a full shader.
    pub fn sprite_shader(
        &self,
        material: MaterialHandle,
    ) -> Option<(ShaderHandle, [[f32; 4]; crate::MAX_SHADER_PARAMS])> {
        self.material_shaders
            .get(material.index())
            .copied()
            .flatten()
            .filter(|_| {
                self.material_slots
                    .live(material.index(), material.generation())
            })
            .filter(|(s, _)| {
                matches!(
                    self.shaders[s.index()].mode,
                    crate::ShaderMode::Hooks | crate::ShaderMode::Sprite
                )
            })
    }

    pub fn mesh_count(&self) -> usize {
        self.meshes.len()
    }

    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }

    /// Every material's handle, in insertion order (tools that edit all of them).
    /// Every live material's handle, in slot order.
    pub fn material_handles(&self) -> impl Iterator<Item = MaterialHandle> + use<> {
        let live: Vec<MaterialHandle> = (0..self.materials.len())
            .filter_map(|i| {
                let generation = self.material_slots.generation(i);
                self.material_slots
                    .live(i, generation)
                    .then(|| MaterialHandle::from_parts(i, generation))
            })
            .collect();
        live.into_iter()
    }

    pub fn material_count(&self) -> usize {
        self.materials.len()
    }

    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    pub(crate) fn meshes(&self) -> &[MeshData] {
        &self.meshes
    }

    pub(crate) fn fonts(&self) -> &[MsdfFont] {
        &self.fonts
    }

    pub(crate) fn textures(&self) -> &[TextureSource] {
        &self.textures
    }

    /// [`RenderScene::material_serial`]'s count of material changes.
    pub(crate) fn material_serial(&self) -> u64 {
        self.material_serial
    }

    pub(crate) fn materials(&self) -> impl Iterator<Item = (&StandardMaterial, u64)> {
        self.materials
            .iter()
            .zip(self.material_revisions.iter().copied())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ColorSpace, Material, MaterialShader, Mips, ParamKind, RenderScene, ShaderMode,
        ShaderParam, StandardMaterial, TextureArrayData, TextureData, TextureSource,
    };

    /// Every edit of a material moves the serial the renderer's per-frame
    /// material pass waits for (engine s96): an edit that left it would
    /// never reach the GPU. A shader parameter set to the value it holds
    /// leaves it, so a quiet frame skips the pass.
    #[test]
    fn every_material_edit_moves_the_material_serial() {
        let mut scene = RenderScene::new();
        let mut last = scene.material_serial();
        let mut moved = |scene: &RenderScene, edit: &str| {
            let now = scene.material_serial();
            assert_ne!(now, last, "{edit} left the material serial");
            last = now;
        };
        let texture = scene.add_texture(TextureData {
            width: 1,
            height: 1,
            rgba: vec![255; 4],
            color_space: ColorSpace::Srgb,
            mips: Mips::None,
        });
        let a = scene.add_material(Material {
            base_color_texture: Some(texture),
            ..Material::default()
        });
        moved(&scene, "add_material");
        let b = scene.add_standard_material(StandardMaterial::default());
        moved(&scene, "add_standard_material");
        assert!(scene.set_material(b, Material::default()));
        moved(&scene, "set_material");
        assert!(scene.set_standard_material(b, StandardMaterial::default()));
        moved(&scene, "set_standard_material");
        let shader = scene.add_shader(MaterialShader {
            name: "wind".into(),
            code: String::new(),
            params: vec![ShaderParam {
                name: "env".into(),
                kind: ParamKind::Vec4,
                default: [0.0; 4],
            }],
            mode: ShaderMode::Hooks,
        });
        assert!(scene.set_material_shader(b, Some(shader)));
        moved(&scene, "set_material_shader");
        assert!(scene.set_shader_param(b, "env", &[0.5, 0.0, 0.0, 0.0]));
        moved(&scene, "set_shader_param");
        let held = scene.material_serial();
        assert!(scene.set_shader_param(b, "env", &[0.5, 0.0, 0.0, 0.0]));
        assert_eq!(
            scene.material_serial(),
            held,
            "the same value again moved it"
        );
        assert!(scene.remove_texture(texture));
        moved(&scene, "remove_texture of a texture a material samples");
        let layer = TextureSource::Rgba(TextureData {
            width: 1,
            height: 1,
            rgba: vec![255; 4],
            color_space: ColorSpace::Srgb,
            mips: Mips::None,
        });
        let array = scene
            .add_texture_array(TextureArrayData {
                layers: vec![layer.clone(), layer],
            })
            .unwrap();
        scene.add_standard_material(StandardMaterial {
            mapArray: array.slot(),
            ..StandardMaterial::default()
        });
        moved(&scene, "add_standard_material with an array map");
        assert!(scene.remove_texture_array(array));
        moved(
            &scene,
            "remove_texture_array of an array a material samples",
        );
        assert!(scene.remove_material(a));
        moved(&scene, "remove_material");
        scene.add_material(Material::default());
        moved(&scene, "add_material into a freed slot");
    }
}
