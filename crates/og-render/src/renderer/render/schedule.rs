//! The frame's pass list: which passes run, in the order the render graph
//! is declared in (its ties keep that order), and the capabilities each
//! pass needs (`caps.rs`).
//!
//! [`PASSES`] is the one table of what a pass needs. A pass absent from it
//! needs nothing beyond what every device has. [`frame_passes`] builds the
//! frame's list from what the frame asks for ([`PassDecisions`]) and leaves
//! out every pass the device's capabilities lack; [`Renderer::absent_passes`]
//! names them. The decisions themselves read the capabilities where a pass
//! has a path without them (the CPU particles, the raymarched fog, sorted
//! blending, no screen-space reflections), so what reads a pass's targets
//! is left out with it.

use super::*;

/// What a kind of pass needs of the device.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PassDesc {
    /// A pass of the kind (an instanced kind by its first instance).
    pub pass: FramePass,
    /// The kind's name, as [`Renderer::absent_passes`] reports it.
    pub name: &'static str,
    pub needs: Caps,
}

/// Every kind of graph pass that needs a capability; the rest need none.
pub(crate) const PASSES: &[PassDesc] = &[
    LiveSkyPass::DESC,
    GiProbesPass::DESC,
    // Three colour targets: normals and velocity, linear depth, outline ids.
    PassDesc {
        pass: FramePass::Prepass,
        name: "prepass",
        needs: Caps::MRT,
    },
    PassDesc {
        pass: FramePass::Contact,
        name: "contact",
        needs: Caps::COMPUTE,
    },
    PassDesc {
        pass: FramePass::Particles,
        name: "particles",
        needs: Caps::COMPUTE,
    },
    // Each face's depth prepass: the three targets of the camera's; and
    // the light volumes' grid captures, projected in compute (probe_grid.rs).
    PassDesc {
        pass: FramePass::Probes,
        name: "reflection-probes",
        needs: Caps::MRT.union(Caps::COMPUTE),
    },
    // Its own depth prepass, as the probes' faces.
    PassDesc {
        pass: FramePass::View(0),
        name: "view",
        needs: Caps::MRT,
    },
    PassDesc {
        pass: FramePass::Volumetric,
        name: "volumetric",
        needs: Caps::COMPUTE,
    },
    PassDesc {
        pass: FramePass::Ssr,
        name: "ssr",
        needs: Caps::COMPUTE,
    },
];

impl FramePass {
    /// The capabilities the pass needs ([`PASSES`]).
    pub(crate) fn needs(self) -> Caps {
        let kind = std::mem::discriminant(&self);
        PASSES
            .iter()
            .find(|d| std::mem::discriminant(&d.pass) == kind)
            .map_or(Caps::NONE, |d| d.needs)
    }
}

impl Renderer {
    /// Whether the device has what `pass` needs.
    pub(crate) fn runs(&self, pass: FramePass) -> bool {
        self.info.caps.contains(pass.needs())
    }

    /// The kinds of pass this renderer leaves out, each with the
    /// capabilities its device (under [`RenderDesc::tier`]) lacks for it.
    /// Empty on the full tier.
    pub fn absent_passes(&self) -> impl Iterator<Item = (&'static str, Caps)> + '_ {
        PASSES
            .iter()
            .filter(|d| !self.info.caps.contains(d.needs))
            .map(|d| (d.name, self.info.caps.missing(d.needs)))
    }
}

/// What a frame asks of each pass, before the device's capabilities: what
/// `render` decided from the scene, the quality and the renderer's options.
#[derive(Default)]
pub(crate) struct PassDecisions<'a> {
    pub live_sky: bool,
    pub gi_probes: bool,
    pub cascades: u32,
    pub shelter: bool,
    /// The shadowed local lights: a pass per face, from each one's layer.
    pub local_shadows: &'a [frame::LocalShadow],
    pub prepass: bool,
    /// The AO passes' resolution, when they run.
    pub ao: Option<AoResolution>,
    pub contact: bool,
    pub particles: bool,
    pub light_2d: bool,
    pub metaballs: bool,
    pub fluid_3d: bool,
    pub probes: bool,
    pub reflections: u8,
    pub views: u32,
    pub froxels: bool,
    pub ssr: bool,
    pub depth_bands: bool,
    pub glow: bool,
    pub outline: bool,
    /// Lines outside the silhouettes too (the jump flood).
    pub outline_outside: bool,
    pub taa: bool,
    pub taa_sharpen: bool,
    pub dof: bool,
    pub motion_blur: bool,
    pub screen_blur: bool,
    pub custom_posts: u32,
    pub god_rays: bool,
    pub auto_exposure: bool,
    pub adapt: bool,
    pub bloom: bool,
    /// The bloom chain's levels (`post::bloom_levels`).
    pub bloom_levels: u32,
    pub lens_flare: bool,
    pub flare_sources: bool,
    pub smaa_t2x: bool,
    pub paint: bool,
    pub watercolor: bool,
    pub lines: bool,
    pub lens: bool,
    pub quantize: bool,
    pub fxaa: bool,
    pub smaa: bool,
    pub upscale: bool,
}

/// The frame's passes in declaration order, less those `caps` lacks what
/// they need for.
pub(crate) fn frame_passes(d: &PassDecisions<'_>, caps: Caps) -> Vec<FramePass> {
    let mut passes: Vec<FramePass> = Vec::new();
    let mut push = |p: FramePass| {
        if caps.contains(p.needs()) {
            passes.push(p);
        }
    };
    // The live sky's step first: what the frame's passes sample is its
    // last complete capture (sky_capture.rs).
    if d.live_sky {
        push(FramePass::LiveSky);
    }
    if d.gi_probes {
        push(FramePass::GiProbes);
    }
    for i in 0..d.cascades {
        push(FramePass::Cascade(i));
    }
    if d.shelter {
        push(FramePass::Shelter);
    }
    for ls in d.local_shadows {
        for face in 0..ls.faces.len() {
            push(FramePass::LocalFace((ls.layer + face) as u32));
        }
    }
    if d.prepass {
        push(FramePass::Prepass);
    }
    if let Some(resolution) = d.ao {
        push(FramePass::Ao);
        push(FramePass::AoBlur);
        if resolution == AoResolution::Half {
            push(FramePass::AoUpsample);
        }
    }
    if d.contact {
        push(FramePass::Contact);
    }
    if d.particles {
        push(FramePass::Particles);
    }
    if d.light_2d {
        push(FramePass::Light2d);
    }
    if d.metaballs {
        push(FramePass::Metaballs);
    }
    if d.fluid_3d {
        push(FramePass::Fluid3d);
    }
    if d.probes {
        push(FramePass::Probes);
    }
    for k in 0..d.reflections {
        push(FramePass::Reflection(k));
    }
    for i in 0..d.views {
        push(FramePass::View(i));
    }
    if d.froxels {
        push(FramePass::Volumetric);
    }
    if d.ssr {
        push(FramePass::Ssr);
    }
    push(FramePass::Main);
    if d.depth_bands {
        push(FramePass::DepthBands);
    }
    if d.glow {
        push(FramePass::Glow);
    }
    if d.outline {
        push(FramePass::Outline);
        // Lines outside the silhouettes: the jump flood (fxpost.wgsl).
        if d.outline_outside {
            push(FramePass::OutlineSeed);
            for i in 0..post::JFA_STEPS.len() as u32 {
                push(FramePass::OutlineJump(i));
            }
            push(FramePass::OutlineWide);
        }
    }
    if d.taa {
        push(FramePass::Taa);
        if d.taa_sharpen {
            push(FramePass::TaaSharpen);
        }
    }
    if d.dof {
        push(FramePass::DofCoc);
        push(FramePass::DofBlur);
        push(FramePass::DofComposite);
    }
    if d.motion_blur {
        push(FramePass::MotionBlur);
    }
    if d.screen_blur {
        push(FramePass::ScreenBlur);
    }
    for i in 0..d.custom_posts {
        push(FramePass::CustomPost(i));
    }
    if d.god_rays {
        push(FramePass::GodRaysMask);
        push(FramePass::GodRaysBlur);
    }
    if d.auto_exposure {
        push(FramePass::Meter);
        for i in 0..post::meter_levels() - 1 {
            push(FramePass::Reduce(i));
        }
        if d.adapt {
            push(FramePass::Adapt);
        }
    }
    // The lens flare reads the chain's first level before the upsamples
    // add onto it: with bloom off, the prefilter runs for it alone.
    if d.bloom || d.lens_flare {
        push(FramePass::BloomPrefilter);
    }
    if d.lens_flare {
        if d.flare_sources {
            push(FramePass::FlareOcclusion);
        }
        push(FramePass::Flare);
        push(FramePass::FlareBlurX);
        push(FramePass::FlareBlurY);
    }
    if d.bloom {
        for i in 0..d.bloom_levels - 1 {
            push(FramePass::BloomDown(i));
        }
        for i in (0..d.bloom_levels - 1).rev() {
            push(FramePass::BloomUp(i));
        }
    }
    push(FramePass::Tonemap);
    // Temporal resolve precedes distortion: motion and depth remain in
    // scene coordinates. The overlay never enters either history slot.
    if d.smaa_t2x {
        push(FramePass::SmaaEdges);
        push(FramePass::SmaaWeights);
        push(FramePass::SmaaBlend);
        push(FramePass::SmaaResolve);
    }
    // The stylisation chain (post.rs `chain_passes`), then FXAA, then the
    // pixel grid scaled up to the output.
    if d.paint {
        push(FramePass::Paint);
    }
    if d.watercolor {
        push(FramePass::Watercolor);
    }
    if d.lines {
        push(FramePass::Lines);
    }
    if d.lens {
        push(FramePass::Lens);
    }
    if d.quantize {
        push(FramePass::Quantize);
    }
    if d.fxaa {
        push(FramePass::Fxaa);
    }
    if d.smaa {
        push(FramePass::SmaaEdges);
        push(FramePass::SmaaWeights);
        push(FramePass::SmaaBlend);
    }
    // Under TAAU the frame after TAA is already the output's size.
    if d.upscale {
        push(FramePass::Upscale);
    }
    passes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn light(layer: usize, faces: usize) -> frame::LocalShadow {
        frame::LocalShadow {
            light: 0,
            record: 0,
            layer,
            position: Vec3::ZERO,
            range: 1.0,
            paraboloid: false,
            faces: vec![(Mat4::IDENTITY, Frustum::from_view_proj(Mat4::IDENTITY)); faces],
        }
    }

    /// A frame that asks for every pass.
    fn everything(local_shadows: &[frame::LocalShadow]) -> PassDecisions<'_> {
        PassDecisions {
            live_sky: true,
            gi_probes: true,
            cascades: 3,
            shelter: true,
            local_shadows,
            prepass: true,
            ao: Some(AoResolution::Half),
            contact: true,
            particles: true,
            light_2d: true,
            metaballs: true,
            fluid_3d: true,
            probes: true,
            reflections: 2,
            views: 2,
            froxels: true,
            ssr: true,
            depth_bands: true,
            glow: true,
            outline: true,
            outline_outside: true,
            taa: true,
            taa_sharpen: true,
            dof: true,
            motion_blur: true,
            screen_blur: true,
            custom_posts: 2,
            god_rays: true,
            auto_exposure: true,
            adapt: true,
            bloom: true,
            bloom_levels: 5,
            lens_flare: true,
            flare_sources: true,
            smaa_t2x: false,
            paint: true,
            watercolor: true,
            lines: true,
            lens: true,
            quantize: true,
            fxaa: true,
            smaa: false,
            upscale: true,
        }
    }

    #[test]
    fn the_full_tier_keeps_every_pass_asked_for() {
        let lights = [light(0, 6), light(6, 1)];
        let passes = frame_passes(&everything(&lights), Caps::FULL);
        for d in PASSES {
            assert!(
                passes
                    .iter()
                    .any(|p| std::mem::discriminant(p) == std::mem::discriminant(&d.pass)),
                "{} left out on the full tier",
                d.name
            );
        }
        assert_eq!(
            passes
                .iter()
                .filter(|p| matches!(p, FramePass::LocalFace(_)))
                .count(),
            7
        );
        assert_eq!(passes.last(), Some(&FramePass::Upscale));
    }

    #[test]
    fn a_limited_tier_leaves_out_exactly_what_it_lacks_in_order() {
        let lights = [light(0, 6), light(6, 1)];
        let full = frame_passes(&everything(&lights), Caps::FULL);
        for tier in [Tier::MiniGame, Tier::Gl] {
            let caps = tier.ceiling();
            let limited = frame_passes(&everything(&lights), caps);
            let expected: Vec<FramePass> = full
                .iter()
                .copied()
                .filter(|p| caps.contains(p.needs()))
                .collect();
            assert_eq!(limited, expected, "{tier:?}");
        }
        let gl = frame_passes(&everything(&lights), Tier::Gl.ceiling());
        for pass in [
            FramePass::GiProbes,
            FramePass::Contact,
            FramePass::Particles,
            FramePass::Volumetric,
            FramePass::Ssr,
        ] {
            assert!(!gl.contains(&pass), "{pass:?} on the GL tier");
        }
        // What needs nothing stays.
        assert!(gl.contains(&FramePass::Main) && gl.contains(&FramePass::Cascade(2)));
        let mini = frame_passes(&everything(&lights), Tier::MiniGame.ceiling());
        assert!(mini.contains(&FramePass::Ssr) && !mini.contains(&FramePass::Prepass));
        assert!(!mini.iter().any(|p| matches!(p, FramePass::View(_))));
    }

    #[test]
    fn instanced_kinds_share_their_needs() {
        assert_eq!(FramePass::View(3).needs(), Caps::MRT);
        assert_eq!(FramePass::Cascade(1).needs(), Caps::NONE);
        assert_eq!(FramePass::Main.needs(), Caps::NONE);
    }
}
