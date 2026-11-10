use super::*;

impl RenderScene {
    /// Adds a particle emitter description (an asset).
    pub fn add_emitter(&mut self, emitter: ParticleEmitter) -> EmitterHandle {
        self.emitters.push(emitter);
        EmitterHandle(self.emitters.len() as u32 - 1)
    }

    /// Replaces an emitter (running systems keep their particles while the
    /// capacity is unchanged). `false` for a handle of another scene.
    pub fn set_emitter(&mut self, handle: EmitterHandle, emitter: ParticleEmitter) -> bool {
        match self.emitters.get_mut(handle.index()) {
            Some(e) => {
                *e = emitter;
                true
            }
            None => false,
        }
    }

    pub fn emitter(&self, handle: EmitterHandle) -> Option<&ParticleEmitter> {
        self.emitters.get(handle.index())
    }

    /// Adds an effect material (`effect.rs`): the look particles, trails,
    /// ribbons, beams and effect meshes name.
    pub fn add_effect_material(
        &mut self,
        material: crate::EffectMaterial,
    ) -> crate::EffectMaterialHandle {
        self.effect_materials.push(material);
        crate::EffectMaterialHandle(self.effect_materials.len() as u32 - 1)
    }

    /// Replaces an effect material. `false` for a handle of another scene.
    pub fn set_effect_material(
        &mut self,
        handle: crate::EffectMaterialHandle,
        material: crate::EffectMaterial,
    ) -> bool {
        match self.effect_materials.get_mut(handle.index()) {
            Some(m) => {
                *m = material;
                true
            }
            None => false,
        }
    }

    pub fn effect_material(
        &self,
        handle: crate::EffectMaterialHandle,
    ) -> Option<&crate::EffectMaterial> {
        self.effect_materials.get(handle.index())
    }

    pub fn effect_materials(&self) -> &[crate::EffectMaterial] {
        &self.effect_materials
    }

    /// Adds a trail style (an asset).
    pub fn add_trail_style(&mut self, style: TrailStyle) -> TrailStyleHandle {
        self.trail_styles.push(style);
        TrailStyleHandle(self.trail_styles.len() as u32 - 1)
    }

    pub(crate) fn emitters(&self) -> &[ParticleEmitter] {
        &self.emitters
    }

    pub(crate) fn trail_styles(&self) -> &[TrailStyle] {
        &self.trail_styles
    }
}
