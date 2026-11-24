//! Wind on placed models (R6, `og.Model.sway`): a swaying instance draws
//! each part with a copy of its material running og-render's wind shader
//! (`og_render::wind_shader`, a `vertex_offset` hook), one copy per
//! (material, sway, model top), whose parameters are written once a frame
//! from og.Wind and the tick. Presentation only: the copies are render
//! state, and nothing here reads or writes the world.

use og_core::IdMap;
use og_render::{MaterialHandle, RenderScene, ShaderHandle, Wind};
use std::collections::HashMap;

/// The swaying models' state: each model's top and the wind copies made.
#[derive(Default)]
pub(crate) struct WindModels {
    /// Each model's top above its origin (its bounds' max y), by asset bits.
    pub tops: IdMap<u64, f32>,
    shader: Option<ShaderHandle>,
    /// The copy of a material for a sway and a top (their bits).
    copies: HashMap<(MaterialHandle, u32, u32), MaterialHandle>,
}

impl WindModels {
    /// The material `material` draws with on a model of top `top` swaying
    /// by `sway`: its wind copy, made the first time; the material itself
    /// when it has its own shader (or no standard form).
    pub fn material(
        &mut self,
        scene: &mut RenderScene,
        material: MaterialHandle,
        sway: f32,
        top: f32,
    ) -> MaterialHandle {
        if sway <= 0.0 || top <= 0.0 || scene.material_shader(material).is_some() {
            return material;
        }
        let key = (material, sway.to_bits(), top.to_bits());
        if let Some(&m) = self.copies.get(&key) {
            return m;
        }
        let Some(standard) = scene.standard_material(material).copied() else {
            return material;
        };
        let shader = *self
            .shader
            .get_or_insert_with(|| scene.add_shader(og_render::wind_shader()));
        let copy = scene.add_standard_material(standard);
        scene.set_material_shader(copy, Some(shader));
        self.copies.insert(key, copy);
        copy
    }

    /// Writes this frame's wind into every copy (`wind` at `tick`), and
    /// the material it copies when that was edited since.
    pub fn present(&self, scene: &mut RenderScene, wind: Option<Wind>, tick: u64, tick_rate: f32) {
        let wind = wind.filter(|w| w.strength > 0.0);
        let phase = wind.map_or([0.0; 2], |w| og_render::wind_phases(&w, tick, tick_rate));
        let k = std::f32::consts::TAU / og_render::WIND_WAVELENGTH;
        for (&(base, sway, top), &copy) in &self.copies {
            if let Some(m) = scene.standard_material(base).copied()
                && scene.standard_material(copy) != Some(&m)
            {
                scene.set_standard_material(copy, m);
            }
            let row = match wind {
                Some(w) => {
                    let d =
                        glam::Vec2::new(w.direction.x, w.direction.z).normalize_or(glam::Vec2::X);
                    [
                        d.x,
                        d.y,
                        w.strength * f32::from_bits(sway),
                        w.gust.clamp(0.0, 1.0),
                    ]
                }
                None => [0.0; 4],
            };
            scene.set_shader_param(copy, "wind", &row);
            scene.set_shader_param(copy, "wave", &[phase[0], phase[1], k, f32::from_bits(top)]);
        }
    }
}
