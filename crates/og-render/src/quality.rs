//! What a frame may cost ([`RenderQuality`], `RenderScene::quality`): the
//! switches and sizes the renderer reads every frame on top of what the
//! scene asks for, and the graphics presets ([`QualityPreset`]) that set
//! them. The API's RenderSettings carries the same fields one to one
//! (`og.render.applyQuality` writes a preset's row of them), so what the
//! renderer reads is what the world holds: no preset state lies behind
//! the fields.
//!
//! The default is the full frame: every effect the scene asks for, the
//! shadow maps at the size the renderer was made with, MSAA at
//! [`crate::MSAA_SAMPLES`]. Each switch only takes away (or, for the
//! shadow maps' size, scales): a scene that asks for no rect-light shadow
//! draws none whatever [`RenderQuality::rect_shadows`] says.

use crate::shadows::MAX_SHADOWED_LIGHTS;

/// The run-time quality of a frame (`RenderScene::quality`; the API's
/// RenderSettings quality fields). Changing [`Self::shadow_map_scale`] or
/// [`Self::msaa_samples`] remakes the shadow maps or the multisampled
/// targets and pipelines at the next frame; the switches cost nothing to
/// change.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderQuality {
    /// Rect lights with `cast_shadows` cast them; false draws every rect
    /// light unshadowed (each keeps its flag for when it is true again).
    pub rect_shadows: bool,
    /// At most this many point, spot and rect lights cast shadows in a
    /// frame, the brightest at the camera kept; the tier's
    /// [`crate::RenderInfo::shadowed_lights`] caps it (0: none).
    pub shadowed_lights: u32,
    /// The shadow maps at this fraction of their full side: the
    /// directional light's cascades of [`crate::RenderDesc::shadow_map_size`]
    /// and the local lights' layers of [`crate::LOCAL_SHADOW_SIZE`], each
    /// kept within 64 texels and the device's (and 4096 for the cascades).
    pub shadow_map_scale: f32,
    /// Volumetric fog is drawn lit (`RenderScene::volumetric_fog`); false
    /// draws the same medium as the plain distance and height fog
    /// (`RenderScene::fog`).
    pub volumetric_fog: bool,
    /// Particles with a `soft_distance` fade against the scene behind them;
    /// false draws them hard-edged (and asks no depth prepass for them).
    pub soft_particles: bool,
    /// `PostEffects::ssr` draws; false leaves smooth surfaces the
    /// reflection probes' and the environment's reflection.
    pub ssr: bool,
    /// `PostEffects::ao` darkens creases; false draws none.
    pub ambient_occlusion: bool,
    /// Samples per pixel where antialiasing multisamples
    /// ([`crate::Antialias::Msaa4`], the WebGL2 tier's choice):
    /// [`crate::MSAA_SAMPLES`], or 1, which draws FXAA instead
    /// ([`crate::RenderInfo::antialias`] says which is drawn). TAA, FXAA
    /// and no antialiasing take one sample whatever it says.
    pub msaa_samples: u32,
}

impl Default for RenderQuality {
    fn default() -> Self {
        RenderQuality {
            rect_shadows: true,
            shadowed_lights: MAX_SHADOWED_LIGHTS as u32,
            shadow_map_scale: 1.0,
            volumetric_fog: true,
            soft_particles: true,
            ssr: true,
            ambient_occlusion: true,
            msaa_samples: crate::MSAA_SAMPLES,
        }
    }
}

/// A graphics preset: a row of [`RenderQuality`] (the API's
/// GraphicsQuality, `og.render.applyQuality`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum QualityPreset {
    /// The cheapest frame: no rect-light shadows, two shadowed lights,
    /// half-size shadow maps, plain fog, hard particles, no screen-space
    /// reflections or ambient occlusion, one sample per pixel.
    Low,
    /// The full frame ([`RenderQuality::default`]).
    Medium,
    /// The full frame with the shadow maps at twice their size.
    High,
}

impl RenderQuality {
    /// The preset's row.
    pub fn preset(preset: QualityPreset) -> RenderQuality {
        match preset {
            QualityPreset::Low => RenderQuality {
                rect_shadows: false,
                shadowed_lights: 2,
                shadow_map_scale: 0.5,
                volumetric_fog: false,
                soft_particles: false,
                ssr: false,
                ambient_occlusion: false,
                msaa_samples: 1,
            },
            QualityPreset::Medium => RenderQuality::default(),
            QualityPreset::High => RenderQuality {
                shadow_map_scale: 2.0,
                ..RenderQuality::default()
            },
        }
    }

    /// Whether antialiasing that multisamples does (else it is FXAA).
    pub(crate) fn multisample(&self) -> bool {
        self.msaa_samples >= crate::MSAA_SAMPLES
    }

    /// The side of a shadow map of `full` texels at this scale, within 64
    /// and `max`.
    pub(crate) fn shadow_side(&self, full: u32, max: u32) -> u32 {
        let scale = if self.shadow_map_scale.is_finite() {
            self.shadow_map_scale.clamp(0.0, 8.0)
        } else {
            1.0
        };
        ((full as f32 * scale).round() as u32).clamp(64, max.max(64))
    }
}
