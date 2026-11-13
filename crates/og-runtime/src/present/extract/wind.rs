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

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    /// Ways this could fail: each frame makes another copy (the materials
    /// grow without end), a copy is shared between two sways or tops, a
    /// material with its own shader loses it, or the parameters are not
    /// the wind's at the tick (a still copy in a wind, phases that do not
    /// move, or that differ between two runs at the same tick).
    #[test]
    fn a_swaying_material_is_one_wind_copy_written_each_frame() {
        let mut scene = RenderScene::new();
        let leaf = scene.add_standard_material(og_render::StandardMaterial {
            color: Vec3::new(0.2, 0.6, 0.1),
            ..Default::default()
        });
        let mut w = WindModels::default();
        let copy = w.material(&mut scene, leaf, 1.0, 4.0);
        assert_ne!(copy, leaf);
        assert_eq!(w.material(&mut scene, leaf, 1.0, 4.0), copy, "made once");
        assert_ne!(
            w.material(&mut scene, leaf, 0.5, 4.0),
            copy,
            "a sway of its own"
        );
        assert_eq!(
            w.material(&mut scene, leaf, 0.0, 4.0),
            leaf,
            "still: itself"
        );
        assert_eq!(
            scene.standard_material(copy).map(|m| m.color),
            Some(Vec3::new(0.2, 0.6, 0.1)),
            "the copy draws as the material"
        );
        let (shader, _) = scene.material_shader(copy).expect("the wind shader");
        assert_eq!(scene.shaders()[shader.index()].name, "og.wind");
        // A material with its own shader keeps it and stays still.
        let hooked = scene.add_standard_material(Default::default());
        let own = scene.add_shader(og_render::wind_shader());
        scene.set_material_shader(hooked, Some(own));
        assert_eq!(w.material(&mut scene, hooked, 1.0, 4.0), hooked);
        // The wind at a tick: its lean times the sway, the model's top, and
        // phases that move with the tick and repeat for the same tick.
        let wind = Wind {
            strength: 0.3,
            direction: Vec3::new(0.0, 0.0, 2.0),
            speed: 4.0,
            gust: 0.5,
            ..Wind::default()
        };
        let params = |scene: &RenderScene| scene.material_shader(copy).unwrap().1;
        w.present(&mut scene, Some(wind), 120, 60.0);
        let a = params(&scene);
        assert_eq!(a[0], [0.0, 1.0, 0.3, 0.5]);
        assert_eq!(a[1][3], 4.0);
        assert!((a[1][2] - std::f32::consts::TAU / og_render::WIND_WAVELENGTH).abs() < 1e-6);
        w.present(&mut scene, Some(wind), 150, 60.0);
        assert_ne!(params(&scene)[1][0], a[1][0], "the gusts move");
        w.present(&mut scene, Some(wind), 120, 60.0);
        assert_eq!(params(&scene), a, "the same tick, the same sway");
        w.present(&mut scene, None, 120, 60.0);
        assert_eq!(params(&scene)[0][2], 0.0, "no wind: still");
        // An edit of the material reaches its copy.
        let mut red = *scene.standard_material(leaf).unwrap();
        red.color = Vec3::X;
        scene.set_standard_material(leaf, red);
        w.present(&mut scene, None, 120, 60.0);
        assert_eq!(scene.standard_material(copy).unwrap().color, Vec3::X);
        assert!(scene.material_shader(copy).is_some(), "still the wind's");
    }
}
