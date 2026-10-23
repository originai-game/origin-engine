//! Screen-space reflections (`docs/design.md` §2.6): the full tier's pass
//! after the main pass (`FramePass::Ssr`, `ssr.wgsl` after `fxpost.wgsl`).
//!
//! Each glossy pixel marches its reflected ray in view space through the
//! depth-normal prepass's view depth; where the ray passes behind a surface
//! by less than [`ScreenSpaceReflections::thickness`] the lit image there
//! is blended in, weighted by Fresnel (from the material's metalness) and
//! the square of its gloss (1 - roughness), and faded at the screen's
//! edges, towards [`ScreenSpaceReflections::max_distance`] and for rays
//! turning back towards the camera (what the screen cannot show). The
//! prepass carries the surface's roughness and metalness factors (not
//! their maps) in its view depth target's second channel; terrain,
//! scattered copies and unlit materials carry none and reflect nothing.
//!
//! A ray that finds nothing keeps what the frame's reflection probes and
//! environment map put there. WebGL2 (the GL tier) draws no SSR:
//! `RenderCapabilities.ssr` is false there and smooth surfaces show the
//! probes and the environment alone.

/// Screen-space reflections ([`crate::PostEffects::ssr`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenSpaceReflections {
    /// How far a reflected ray travels, in world units.
    pub max_distance: f32,
    /// How far behind a surface a ray may pass and still hit it (world units).
    pub thickness: f32,
    /// Steps along the ray (4 to 128).
    pub steps: u32,
    /// The fraction of the screen at its edges over which reflections fade out.
    pub fade: f32,
    /// Scales the reflection (0: none).
    pub intensity: f32,
}

impl Default for ScreenSpaceReflections {
    fn default() -> Self {
        ScreenSpaceReflections {
            max_distance: 20.0,
            thickness: 0.3,
            steps: 32,
            fade: 0.15,
            intensity: 1.0,
        }
    }
}

impl ScreenSpaceReflections {
    /// `FxPost`'s `ssr` and `ssr2` rows: (max distance, thickness, edge
    /// fade, intensity), (steps).
    pub(crate) fn rows(s: Option<&ScreenSpaceReflections>) -> [[f32; 4]; 2] {
        match s {
            Some(s) => [
                [
                    s.max_distance.max(0.01),
                    s.thickness.max(0.001),
                    s.fade.clamp(0.0, 0.5),
                    s.intensity.max(0.0),
                ],
                [s.steps.clamp(4, 128) as f32, 0.0, 0.0, 0.0],
            ],
            None => [[0.0; 4]; 2],
        }
    }
}

/// What the prepass writes into its view depth's second channel for a
/// surface of `roughness` and `metalness` (`pbr_prepass.wgsl` packs the
/// same; `ssr.wgsl`'s `ssr_surface` unpacks it): the gloss (1 - roughness)
/// and the metalness in 255ths, the metalness times 256 above the gloss.
#[cfg(test)]
fn pack(roughness: f32, metalness: f32) -> f32 {
    let gloss = ((1.0 - roughness.clamp(0.0, 1.0)) * 255.0).round();
    gloss + 256.0 * (metalness.clamp(0.0, 1.0) * 255.0).round()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unpack(g: f32) -> (f32, f32) {
        let metal = (g / 256.0).floor();
        ((g - metal * 256.0) / 255.0, metal / 255.0)
    }

    /// The packed reflectivity survives the round trip through an f32 for
    /// every 255th, and a surface that wrote nothing (terrain) unpacks as
    /// no gloss.
    #[test]
    fn reflectivity_round_trips() {
        for r in 0..=255 {
            for m in [0, 1, 128, 254, 255] {
                let (rough, metal) = (r as f32 / 255.0, m as f32 / 255.0);
                let (gloss, back) = unpack(pack(rough, metal));
                assert!((gloss - (1.0 - rough)).abs() < 1e-6, "{r} {m}");
                assert!((back - metal).abs() < 1e-6, "{r} {m}");
            }
        }
        assert_eq!(unpack(0.0), (0.0, 0.0));
    }

    #[test]
    fn rows_clamp_and_off_is_zero() {
        let s = ScreenSpaceReflections {
            steps: 1000,
            fade: 2.0,
            ..Default::default()
        };
        let [a, b] = ScreenSpaceReflections::rows(Some(&s));
        assert_eq!(b[0], 128.0);
        assert_eq!(a[2], 0.5);
        assert_eq!(ScreenSpaceReflections::rows(None), [[0.0; 4]; 2]);
    }
}
