//! What a frame may cost ([`RenderQuality`], `RenderScene::quality`): the
//! switches and sizes the renderer reads every frame on top of what the
//! scene asks for, and the graphics presets ([`QualityPreset`]) that set
//! them. The API's RenderSettings carries the same fields one to one
//! (`og.render.applyQuality` writes a preset's row of them), so what the
//! renderer reads is what the world holds: no preset state lies behind
//! the fields.
//!
//! The default is the full frame: every effect the scene asks for at the
//! settings it asks for, the shadow maps at the size the renderer was
//! made with, MSAA at [`crate::MSAA_SAMPLES`]. Each switch only takes
//! away, each scale only scales what the scene asks for: a scene that
//! asks for no rect-light shadow, or no screen-space reflections, draws
//! none whatever [`RenderQuality::rect_shadows`] or
//! [`RenderQuality::ssr_step_scale`] says.

use crate::AoResolution;
use crate::shadows::MAX_SHADOWED_LIGHTS;
use crate::volumetric::{FROXELS, RAYMARCH_STEPS};

/// The run-time quality of a frame (`RenderScene::quality`; the API's
/// RenderSettings quality fields). Changing [`Self::shadow_map_scale`],
/// [`Self::msaa_samples`] or the froxel grid's depth
/// ([`Self::volumetric_step_scale`]) remakes the shadow maps, the
/// multisampled targets and pipelines or the froxel volumes at the next
/// frame; the switches and the other scales cost nothing to change.
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
    /// The resolution `PostEffects::ao` is searched at: `None` (the
    /// default) is the scene's [`crate::AmbientOcclusion::resolution`];
    /// `Some` replaces it, the scene's kept for when it is `None` again.
    pub ao_resolution: Option<AoResolution>,
    /// `PostEffects::ssr`'s steps along each ray times this (within the
    /// 4 to 128 the pass takes): 2 is 64 steps for the default 32, finer
    /// contact with thin and distant reflected surfaces.
    pub ssr_step_scale: f32,
    /// Volumetric fog's samples along each view ray times this: the
    /// froxel grid's [`FROXELS`] slices deep on the full tier (each
    /// sliced more thinly, so a shaft's edge in depth and the fog in
    /// front of a near surface resolve more finely), the raymarch's
    /// [`RAYMARCH_STEPS`] steps on WebGL2; within 8 to 256 slices and 4 to
    /// 96 steps.
    pub volumetric_step_scale: f32,
    /// Samples per pixel where antialiasing multisamples
    /// ([`crate::Antialias::Msaa4`], the WebGL2 tier's choice):
    /// [`crate::MSAA_SAMPLES`], or 1, which draws FXAA instead
    /// ([`crate::RenderInfo::antialias`] says which is drawn).
    /// [`crate::RenderInfo::msaa_samples`] caps it, as
    /// [`crate::RenderInfo::shadowed_lights`] caps
    /// [`Self::shadowed_lights`]: where the device did not resolve MSAA
    /// (TAA, FXAA, none, or FXAA for want of a multisampled HDR format)
    /// it is 1, and one sample is drawn whatever this says.
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
            ao_resolution: None,
            ssr_step_scale: 1.0,
            volumetric_step_scale: 1.0,
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
    /// reflections or ambient occlusion, one sample per pixel; and, for a
    /// game that turns one of those back on, ambient occlusion at half
    /// resolution and half the reflection and fog steps.
    Low,
    /// The full frame ([`RenderQuality::default`]).
    Medium,
    /// The full frame refined: the shadow maps at twice their side,
    /// ambient occlusion at full resolution whatever the scene asks,
    /// twice the screen-space reflection steps and twice the volumetric
    /// fog's slices.
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
                ao_resolution: Some(AoResolution::Half),
                ssr_step_scale: 0.5,
                volumetric_step_scale: 0.5,
                msaa_samples: 1,
            },
            QualityPreset::Medium => RenderQuality::default(),
            QualityPreset::High => RenderQuality {
                shadow_map_scale: 2.0,
                ao_resolution: Some(AoResolution::Full),
                ssr_step_scale: 2.0,
                volumetric_step_scale: 2.0,
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
        scaled(full, self.shadow_map_scale, 8.0).clamp(64, max.max(64))
    }

    /// The steps of screen-space reflections that ask for `steps`.
    pub(crate) fn ssr_steps(&self, steps: u32) -> u32 {
        scaled(steps.clamp(4, 128), self.ssr_step_scale, 32.0).clamp(4, 128)
    }

    /// The froxel grid's slices deep.
    pub(crate) fn froxel_slices(&self) -> u32 {
        scaled(FROXELS[2], self.volumetric_step_scale, 4.0).clamp(8, 256)
    }

    /// The raymarch's steps per pixel.
    pub(crate) fn raymarch_steps(&self) -> u32 {
        scaled(RAYMARCH_STEPS, self.volumetric_step_scale, 4.0).clamp(4, 96)
    }
}

/// `n` times `scale` (within 0 and `max`; not finite: 1), rounded.
fn scaled(n: u32, scale: f32, max: f32) -> u32 {
    let scale = if scale.is_finite() {
        scale.clamp(0.0, max)
    } else {
        1.0
    };
    (n as f32 * scale).round() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Medium is the default, so a frame drawn before any preset is
    /// medium's; and at the default every scale is the identity, so the
    /// scene's own settings are drawn exactly.
    #[test]
    fn medium_is_the_default_and_the_default_scales_nothing() {
        let q = RenderQuality::preset(QualityPreset::Medium);
        assert_eq!(q, RenderQuality::default());
        assert_eq!(q.ao_resolution, None);
        for steps in [4, 7, 32, 100, 128] {
            assert_eq!(q.ssr_steps(steps), steps);
        }
        assert_eq!(q.ssr_steps(200), 128);
        assert_eq!(q.froxel_slices(), FROXELS[2]);
        assert_eq!(q.raymarch_steps(), RAYMARCH_STEPS);
        assert_eq!(q.shadow_side(2048, 4096), 2048);
    }

    /// High refines every scale (within the passes' bounds) and asks
    /// full-resolution AO; low halves them and asks half.
    #[test]
    fn high_doubles_and_low_halves_the_steps() {
        let high = RenderQuality::preset(QualityPreset::High);
        assert_eq!(high.ao_resolution, Some(AoResolution::Full));
        assert_eq!(high.ssr_steps(32), 64);
        assert_eq!(high.ssr_steps(100), 128);
        assert_eq!(high.froxel_slices(), 128);
        assert_eq!(high.raymarch_steps(), 48);
        assert_eq!(high.shadow_side(2048, 4096), 4096);
        let low = RenderQuality::preset(QualityPreset::Low);
        assert_eq!(low.ao_resolution, Some(AoResolution::Half));
        assert_eq!(low.ssr_steps(32), 16);
        assert_eq!(low.ssr_steps(4), 4);
        assert_eq!(low.froxel_slices(), 32);
        assert_eq!(low.raymarch_steps(), 12);
        let odd = RenderQuality {
            volumetric_step_scale: f32::NAN,
            ssr_step_scale: 1000.0,
            ..RenderQuality::default()
        };
        assert_eq!(odd.froxel_slices(), FROXELS[2]);
        assert_eq!(odd.ssr_steps(32), 128);
    }
}
