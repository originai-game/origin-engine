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
use crate::shadows::{DEFAULT_SHADOWED_LIGHTS, MAX_SHADOWED_LIGHTS};
use crate::volumetric::{FROXELS, RAYMARCH_STEPS};

/// The run-time quality of a frame (`RenderScene::quality`; the API's
/// RenderSettings quality fields). Changing [`Self::shadow_map_scale`],
/// [`Self::msaa_samples`] or the froxel grid's depth
/// ([`Self::volumetric_step_scale`]) remakes the shadow maps, the
/// multisampled targets and pipelines or the froxel volumes at the next
/// frame; the switches and the other scales cost nothing to change.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderQuality {
    /// Spread live sky capture and filtering over 13 frames (low preset).
    /// False keeps environment light and reflections on the current frame;
    /// slicing trades lighting latency for lower peak GPU cost.
    pub sky_capture_sliced: bool,
    /// Rect lights with `cast_shadows` cast them; false draws every rect
    /// light unshadowed (each keeps its flag for when it is true again).
    pub rect_shadows: bool,
    /// At most this many point, spot and rect lights cast shadows in a
    /// frame, the brightest at the camera kept; the renderer's
    /// [`crate::RenderInfo::shadowed_lights`] caps it (0: none).
    pub shadowed_lights: u32,
    /// The directional light's cascades, 1 to [`crate::MAX_CASCADES`]
    /// ([`crate::RenderInfo::shadow_cascades`] caps it): more cover the
    /// same shadow distance with finer near shadows, one cascade pass and
    /// one map of [`crate::RenderDesc::shadow_map_size`] each. Changing it
    /// remakes the cascade array at the next frame. 3 by default.
    pub shadow_cascades: u32,
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
    /// Every particle system's emission times this (0..1), each emitter
    /// kept above it where it would otherwise have only a few particles
    /// alive ([`crate::ParticleEmitter::thinned`]): fewer particles drawn,
    /// and fewer pixels blended where an effect has many. Drawing only:
    /// the tick's particle events and collisions are the full emitter's.
    pub particle_emission: f32,
    /// At most this many composed effects (the API's og.Vfx) drawn at once,
    /// and at most this many particle slots among their emitters: past
    /// either, extraction leaves out the oldest one-shot effects, then the
    /// oldest looping ones. Drawing only: their events, shakes and sounds
    /// are the tick's. The renderer draws whatever it is given; extraction
    /// reads these.
    pub max_vfx: u32,
    pub max_vfx_particles: u32,
    /// `PostEffects::ssr` draws; false leaves smooth surfaces the
    /// reflection probes' and the environment's reflection.
    pub ssr: bool,
    /// `PostEffects::ao` darkens creases; false draws none.
    pub ambient_occlusion: bool,
    /// `PostEffects::gi` bounces light; false draws none.
    pub global_illumination: bool,
    /// The resolution `PostEffects::ao` (and `PostEffects::gi`) is searched at: `None` (the
    /// default) is the scene's [`crate::AmbientOcclusion::resolution`];
    /// `Some` replaces it, the scene's kept for when it is `None` again.
    pub ao_resolution: Option<AoResolution>,
    /// `PostEffects::ssr`'s steps along each ray times this (within the
    /// 4 to 128 the pass takes): 2 is 64 steps for the default 32, finer
    /// contact with thin and distant reflected surfaces.
    pub ssr_step_scale: f32,
    /// Volumetric fog's samples along each view ray times this: the
    /// froxel grid's [`FROXELS`] slices deep (each sliced more thinly, so
    /// a shaft's edge in depth and the fog in front of a near surface
    /// resolve more finely), or the raymarch's [`RAYMARCH_STEPS`] steps
    /// ([`crate::VolumetricMethod::Raymarch`]); within 8 to 256 slices and 4 to
    /// 96 steps.
    pub volumetric_step_scale: f32,
    /// Samples per pixel where antialiasing multisamples
    /// ([`crate::Antialias::Msaa4`]):
    /// [`crate::MSAA_SAMPLES`], or 1, which draws FXAA instead
    /// ([`crate::RenderInfo::antialias`] says which is drawn).
    /// [`crate::RenderInfo::msaa_samples`] caps it, as
    /// [`crate::RenderInfo::shadowed_lights`] caps
    /// [`Self::shadowed_lights`]: where the device did not resolve MSAA
    /// (TAA, FXAA, none, or FXAA for want of a multisampled HDR format)
    /// it is 1, and one sample is drawn whatever this says.
    pub msaa_samples: u32,
    /// The temporal resolve under TAA ([`crate::Upscaler`]): FSR 3.1 below
    /// render scale 1 and TAAU at it (`Auto`: the default and medium), TAAU
    /// alone (low) or FSR 3.1 alone (high), which is sharper and steadier
    /// in motion at more GPU time (docs/design.md §11).
    pub upscaler: crate::Upscaler,
}

impl Default for RenderQuality {
    fn default() -> Self {
        RenderQuality {
            sky_capture_sliced: false,
            rect_shadows: true,
            shadowed_lights: DEFAULT_SHADOWED_LIGHTS as u32,
            shadow_cascades: 3,
            shadow_map_scale: 1.0,
            volumetric_fog: true,
            soft_particles: true,
            particle_emission: 1.0,
            max_vfx: 64,
            max_vfx_particles: 65_536,
            ssr: true,
            ambient_occlusion: true,
            global_illumination: true,
            ao_resolution: None,
            ssr_step_scale: 1.0,
            volumetric_step_scale: 1.0,
            msaa_samples: crate::MSAA_SAMPLES,
            upscaler: crate::Upscaler::Auto,
        }
    }
}

/// A graphics preset: a row of [`RenderQuality`] (the API's
/// GraphicsQuality, `og.render.applyQuality`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum QualityPreset {
    /// The cheapest frame: no rect-light shadows, two shadowed lights,
    /// two cascades, half-size shadow maps, plain fog, hard particles at
    /// half their emission, half the composed effects, no screen-space
    /// reflections, ambient occlusion or global illumination, one sample per pixel; and, for a
    /// game that turns one of those back on, ambient occlusion at half
    /// resolution and half the reflection and fog steps.
    Low,
    /// The full frame ([`RenderQuality::default`]).
    Medium,
    /// The full frame refined: four cascades, every shadowed light the
    /// renderer allows ([`MAX_SHADOWED_LIGHTS`]), the shadow maps at twice
    /// their side, FSR 3.1 for the temporal resolve,
    /// ambient occlusion at full resolution whatever the scene asks,
    /// twice the screen-space reflection steps, twice the volumetric
    /// fog's slices and twice the composed effects.
    High,
}

impl RenderQuality {
    /// The preset's row.
    pub fn preset(preset: QualityPreset) -> RenderQuality {
        match preset {
            QualityPreset::Low => RenderQuality {
                sky_capture_sliced: true,
                rect_shadows: false,
                shadowed_lights: 2,
                shadow_cascades: 2,
                shadow_map_scale: 0.5,
                volumetric_fog: false,
                soft_particles: false,
                particle_emission: 0.5,
                max_vfx: 32,
                max_vfx_particles: 32_768,
                ssr: false,
                ambient_occlusion: false,
                global_illumination: false,
                ao_resolution: Some(AoResolution::Half),
                ssr_step_scale: 0.5,
                volumetric_step_scale: 0.5,
                msaa_samples: 1,
                upscaler: crate::Upscaler::Taau,
            },
            QualityPreset::Medium => RenderQuality::default(),
            QualityPreset::High => RenderQuality {
                shadowed_lights: MAX_SHADOWED_LIGHTS as u32,
                shadow_cascades: crate::MAX_CASCADES,
                shadow_map_scale: 2.0,
                ao_resolution: Some(AoResolution::Full),
                ssr_step_scale: 2.0,
                volumetric_step_scale: 2.0,
                max_vfx: 128,
                max_vfx_particles: 131_072,
                upscaler: crate::Upscaler::Fsr3,
                ..RenderQuality::default()
            },
        }
    }

    /// This quality with every field no costlier than `preset`'s row: each
    /// switch on only where both are, each count and scale the smaller,
    /// ambient occlusion at the lower resolution (automatic quality's
    /// preset steps, which cap the game's own quality and never raise it).
    pub fn capped(&self, preset: QualityPreset) -> RenderQuality {
        let row = RenderQuality::preset(preset);
        let ao = |r: Option<AoResolution>| match r {
            Some(AoResolution::Half) => 0,
            None => 1,
            Some(AoResolution::Full) => 2,
        };
        RenderQuality {
            sky_capture_sliced: self.sky_capture_sliced || row.sky_capture_sliced,
            rect_shadows: self.rect_shadows && row.rect_shadows,
            shadowed_lights: self.shadowed_lights.min(row.shadowed_lights),
            shadow_cascades: self.shadow_cascades.min(row.shadow_cascades),
            shadow_map_scale: self.shadow_map_scale.min(row.shadow_map_scale),
            volumetric_fog: self.volumetric_fog && row.volumetric_fog,
            soft_particles: self.soft_particles && row.soft_particles,
            particle_emission: self.particle_emission.min(row.particle_emission),
            max_vfx: self.max_vfx.min(row.max_vfx),
            max_vfx_particles: self.max_vfx_particles.min(row.max_vfx_particles),
            ssr: self.ssr && row.ssr,
            ambient_occlusion: self.ambient_occlusion && row.ambient_occlusion,
            global_illumination: self.global_illumination && row.global_illumination,
            ao_resolution: if ao(row.ao_resolution) < ao(self.ao_resolution) {
                row.ao_resolution
            } else {
                self.ao_resolution
            },
            ssr_step_scale: self.ssr_step_scale.min(row.ssr_step_scale),
            volumetric_step_scale: self.volumetric_step_scale.min(row.volumetric_step_scale),
            msaa_samples: self.msaa_samples.min(row.msaa_samples),
            upscaler: self.upscaler.capped(preset),
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
