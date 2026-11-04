//! Screen-space global illumination (`docs/design.md` §2.6): the ambient
//! occlusion pass's place in the frame, before the main pass
//! (`FramePass::Ao` drawing `fs_ssgi`, `ssgi.wgsl` after `fxpost.wgsl`).
//!
//! Each pixel casts [`ScreenSpaceGlobalIllumination::rays`] rays over the
//! cosine-weighted hemisphere of its prepass normal and marches each in
//! view space through the prepass's view depth. Where a ray passes behind
//! a surface facing it by less than
//! [`ScreenSpaceGlobalIllumination::thickness`], the light
//! that surface sent towards the camera on the previous frame is read
//! from the previous frame's lit image, where the prepass velocity says
//! the hit pixel was then: TAA's history under TAA (so GI noise, rotated
//! per frame, is averaged by TAA's own accumulation), else a copy of the
//! lit image the main pass leaves. Reading last frame's light makes each
//! frame one more bounce: light converges over a few frames to many
//! bounces, and a surface lit this frame lights its neighbours the next.
//!
//! The rays' average is the light a diffuse surface receives from what
//! the screen shows around it: the lit passes add it times the albedo
//! (as the diffuse image-based light, `pbr_fs.wgsl`), and where the rays
//! hit something they take that share from the ambient light (the
//! environment's, the sky's, the baked probes'), which the bounce
//! replaces ([`ScreenSpaceGlobalIllumination::occlusion`]). With
//! [`crate::PostEffects::ao`] the AO's own term does that instead.
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
    /// How far each ray travels, in world units.
    pub max_distance: f32,
    /// How far behind a surface a ray may pass and still hit it (world units).
    pub thickness: f32,
    /// Steps along each ray (4 to 64).
    pub steps: u32,
    /// Rays per pixel (1 to 16).
    pub rays: u32,
    /// How much of the ambient light the rays that hit something take, 0..1:
    /// 1 is physical (the bounce replaces the ambient light from those
    /// directions), 0 adds the bounce over the whole ambient light. Not
    /// read while [`crate::PostEffects::ao`] is on: its term occludes.
    pub occlusion: f32,
    /// Full resolution, or half (a quarter of the rays, then the AO's
    /// depth-aware upsample).
    pub resolution: crate::AoResolution,
}

impl Default for ScreenSpaceGlobalIllumination {
    fn default() -> Self {
        ScreenSpaceGlobalIllumination {
            intensity: 1.0,
            max_distance: 4.0,
            thickness: 0.5,
            steps: 12,
            rays: 4,
            occlusion: 1.0,
            resolution: crate::AoResolution::Full,
        }
    }
}

/// What `fs_ssgi` takes its ambient occlusion from (`FxPost.gi1.w`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GiOcclusion {
    /// The rays' own hits ([`ScreenSpaceGlobalIllumination::occlusion`]).
    Rays,
    /// The scene's GTAO.
    Gtao,
    /// The scene's SSAO or screen AO (`fxp.ao2.y` tells them apart).
    Ssao,
}

impl ScreenSpaceGlobalIllumination {
    /// `FxPost`'s `gi0` and `gi1` rows: (max distance, thickness,
    /// intensity, steps), (rays, the noise's frame offset, 1 when the
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

/// The pixel offset of frame `frame`'s noise (`ign(pos + offset)`):
/// interleaved gradient noise moved by 5.588238 pixels a frame, which
/// keeps it a low-discrepancy sequence over time (Jimenez, "Next
/// generation post processing in Call of Duty", 2014), repeating every 64
/// frames so the f32 stays exact.
pub(crate) fn noise_offset(frame: u64) -> f32 {
    5.588238 * (frame % 64) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_clamp_and_off_is_zero() {
        let g = ScreenSpaceGlobalIllumination {
            steps: 1000,
            rays: 0,
            occlusion: 3.0,
            ..Default::default()
        };
        let [a, b, c] = ScreenSpaceGlobalIllumination::rows(Some(&g), 65, true, GiOcclusion::Gtao);
        assert_eq!(a[3], 64.0);
        assert_eq!(b[0], 1.0);
        assert_eq!(b[1], noise_offset(1));
        assert_eq!(b[2], 1.0);
        assert_eq!(b[3], 1.0);
        assert_eq!(c[0], 1.0);
        assert_eq!(
            ScreenSpaceGlobalIllumination::rows(None, 3, true, GiOcclusion::Rays),
            [[0.0; 4]; 3]
        );
    }

    /// The noise offset differs on every frame of its 64-frame period, so
    /// TAA averages 64 different ray sets, and frame 0 is the still
    /// pattern (no offset) a frame without TAA keeps.
    #[test]
    fn noise_offset_walks_64_frames() {
        assert_eq!(noise_offset(0), 0.0);
        let mut seen: Vec<u32> = (0..64).map(|f| noise_offset(f).to_bits()).collect();
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), 64);
        assert_eq!(noise_offset(64), 0.0);
    }
}
