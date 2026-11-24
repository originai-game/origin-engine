//! Screen-space global illumination (`docs/design.md` §2.6): the ambient
//! occlusion pass's place in the frame, before the main pass
//! (`FramePass::Ao` drawing `fs_ssgi`, `ssgi.wgsl` after `fxpost.wgsl`).
//!
//! Each pixel gathers the light the screen shows around it over horizon
//! slices through its view ray, as GTAO gathers its visibility, after
//! Therrien, Levesque and Gilet, "Screen Space Indirect Lighting with
//! Visibility Bitmask" (2023): [`ScreenSpaceGlobalIllumination::rays`]
//! directions, two to a slice, each walked
//! [`ScreenSpaceGlobalIllumination::steps`] taps through the prepass's
//! view depth. A tap's surface, taken
//! [`ScreenSpaceGlobalIllumination::thickness`] deep, covers the sectors
//! of the slice's half circle about the normal it spans, and the sectors
//! nothing nearer covered take the light that surface sent towards the
//! camera on the previous frame, where it faces the pixel. That light is
//! read from the previous frame's lit image, where the prepass velocity
//! says the tap's pixel was then: TAA's history under TAA (so GI noise,
//! moved per frame, is averaged by TAA's own accumulation), else a copy
//! of the lit image the main pass leaves. Reading last frame's light
//! makes each frame one more bounce: light converges over a few frames to
//! many bounces, and a surface lit this frame lights its neighbours the
//! next.
//!
//! The sectors' sum is the light a diffuse surface receives from what
//! the screen shows around it: the lit passes add it times the albedo
//! (as the diffuse image-based light, `pbr_fs.wgsl`), and the covered
//! sectors take their share from the ambient light (the environment's,
//! the sky's, the baked probes'), which the bounce replaces
//! ([`ScreenSpaceGlobalIllumination::occlusion`]). With
//! [`crate::PostEffects::ao`] the AO's own term does that instead.
//!
//! `og-tests`' `gi::reference` measures the gather against a path-traced
//! gi-room, and `gi::cost` its pass.
//!
//! The result shares the AO's targets (`ao.raw`, `ao.blur`, `ao`), as
//! [`GI_FORMAT`] instead of the AO's R8: r the occlusion (what every lit
//! pass reads as the AO), gba the bounce. So the lit passes read it through
//! the effect group's AO texture and no binding is added to them.

/// The AO targets' format while the frame draws screen-space GI: r the
/// ambient occlusion, gba the bounced light (linear, before exposure).
pub(crate) const GI_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// Screen-space global illumination ([`crate::PostEffects::gi`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenSpaceGlobalIllumination {
    /// Scales the bounced light (0: none; 1 is physical).
    pub intensity: f32,
    /// How far the gather reaches, in world units. A direction also ends
    /// where it leaves the screen (its taps are spread over what it
    /// crosses on screen), and with the probe GI on, one probe spacing
    /// past its pixel: farther, the probes give the light.
    pub max_distance: f32,
    /// How far behind the depth buffer a surface is taken to extend (world
    /// units).
    pub thickness: f32,
    /// Depth taps along each direction (4 to 64).
    pub steps: u32,
    /// Directions gathered per pixel (1 to 16), two to a slice through
    /// the view ray: an odd count gathers as the next even one.
    pub rays: u32,
    /// How much of the ambient light the directions a surface covers take,
    /// 0..1: 1 is physical (the bounce replaces the ambient light from
    /// those directions), 0 adds the bounce over the whole ambient light.
    /// Not read while [`crate::PostEffects::ao`] is on: its term occludes.
    pub occlusion: f32,
    /// Full resolution, or half (a quarter of the pixels gather, then the
    /// AO's depth-aware upsample).
    pub resolution: crate::AoResolution,
}

impl Default for ScreenSpaceGlobalIllumination {
    fn default() -> Self {
        ScreenSpaceGlobalIllumination {
            intensity: 1.0,
            max_distance: 16.0,
            thickness: 0.5,
            steps: 16,
            rays: 2,
            occlusion: 1.0,
            resolution: crate::AoResolution::Full,
        }
    }
}

/// What `fs_ssgi` takes its ambient occlusion from (`FxPost.gi1.w`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GiOcclusion {
    /// The gather's own covered sectors
    /// ([`ScreenSpaceGlobalIllumination::occlusion`]).
    Rays,
    /// The scene's GTAO.
    Gtao,
    /// The scene's SSAO or screen AO (`fxp.ao2.y` tells them apart).
    Ssao,
}

impl ScreenSpaceGlobalIllumination {
    /// `FxPost`'s `gi0` and `gi1` rows: (max distance, thickness,
    /// intensity, steps), (rays, the noise's frame, 1 when the
    /// previous frame's lit image is there to read, where the occlusion
    /// comes from: 0 the rays, 1 GTAO, 2 SSAO), and `gi2`: (occlusion).
    pub(crate) fn rows(
        g: Option<&ScreenSpaceGlobalIllumination>,
        frame: u64,
        history: bool,
        ao: GiOcclusion,
    ) -> [[f32; 4]; 3] {
        match g {
            Some(g) => [
                [
                    g.max_distance.max(0.01),
                    g.thickness.max(0.001),
                    g.intensity.max(0.0),
                    g.steps.clamp(4, 64) as f32,
                ],
                [
                    g.rays.clamp(1, 16) as f32,
                    noise_offset(frame),
                    history as u32 as f32,
                    match ao {
                        GiOcclusion::Rays => 0.0,
                        GiOcclusion::Gtao => 1.0,
                        GiOcclusion::Ssao => 2.0,
                    },
                ],
                [g.occlusion.clamp(0.0, 1.0), 0.0, 0.0, 0.0],
            ],
            None => [[0.0; 4]; 3],
        }
    }
}

/// How many steps along the R2 sequence frame `frame` moves the pixels'
/// noise (`ssgi.wgsl` `gi_noise`), so a pixel's slices, taps and sector
/// dither over the frames TAA averages stay evenly spread; repeating
/// every 64 frames so the f32 stays exact.
pub(crate) fn noise_offset(frame: u64) -> f32 {
    (frame % 64) as f32
}
