//! The graphs of whole frames: lifetimes and order across families.

use super::*;

fn cfg(taa: bool, gi: bool) -> GraphConfig {
    GraphConfig {
        width: 64,
        height: 32,
        post: (64, 32),
        chain_format: LDR_FORMAT,
        samples: 1,
        taa,
        lightmap: (1, 1),
        lightdir: false,
        gtao: true,
        gi,
        ssr: false,
        bent: false,
        water: false,
        water_ao: false,
        transmission: false,
        fluid_fields: 0,
        reflection: [None, None],
        object_space: false,
        ui_depth: false,
        oit: None,
    }
}

/// The OIT targets are the main pass's, and TAA reads the revealage
/// after it as its reactive mask, so it lives until TAA; the linked
/// lists' resolve binds the depth.
#[test]
fn oit_revealage_lives_until_taa() {
    let passes = [FramePass::Main, FramePass::Taa, FramePass::Tonemap];
    for list in [false, true] {
        let config = GraphConfig {
            oit: Some(list),
            ..cfg(true, false)
        };
        let plan = build_graph(&passes, &config).compile().unwrap();
        let names = plan.ordered_names();
        let at = |name| names.iter().position(|&n| n == name).unwrap();
        assert_eq!(
            plan.lifetime("oit.reveal").unwrap(),
            (at("main"), at("taa"))
        );
        assert_eq!(
            plan.lifetime("oit.front").unwrap(),
            (at("main"), at("main"))
        );
        assert_eq!(
            plan.lifetime("oit.accum").unwrap(),
            (at("main"), at("main"))
        );
    }
    let plan = build_graph(&passes, &cfg(true, false)).compile().unwrap();
    assert!(plan.slot("oit.reveal").is_none());
    // Without TAA the depth is bound only for the lists' resolve.
    let passes = [FramePass::Main, FramePass::Tonemap];
    for (oit, bound) in [(None, false), (Some(false), false), (Some(true), true)] {
        let config = GraphConfig {
            oit,
            ..cfg(false, false)
        };
        let plan = build_graph(&passes, &config).compile().unwrap();
        let depth = &plan.physical()[plan.slot("depth").unwrap()];
        assert_eq!(
            depth.usage.contains(wgpu::TextureUsages::TEXTURE_BINDING),
            bound
        );
    }
}

#[test]
fn hdr_chain_keeps_highlights_through_fxaa_and_upscale() {
    let passes = [
        FramePass::Main,
        FramePass::Tonemap,
        FramePass::Fxaa,
        FramePass::Upscale,
    ];
    let mut config = cfg(false, false);
    config.chain_format = HDR_FORMAT;
    let plan = build_graph(&passes, &config).compile().unwrap();
    for i in 0..chain_passes(&passes).len() {
        let slot = plan.slot(&ldr_name(i)).unwrap();
        assert_eq!(plan.physical()[slot].format, HDR_FORMAT);
    }
    assert_eq!(plan.order(), &[0, 1, 2, 3]);
}

/// Screen-space GI in the AO passes' place: the graph compiles with and
/// without TAA (it reads TAA's history, or its own copy the main pass
/// writes), the AO runs before the main pass that samples it, and the
/// AO's targets hold the GI's RGBA instead of the AO's R8.
#[test]
fn gi_frames_compile_with_the_ao_targets_widened() {
    for taa in [false, true] {
        let mut passes = vec![
            FramePass::Prepass,
            FramePass::Ao,
            FramePass::AoBlur,
            FramePass::AoUpsample,
            FramePass::Main,
        ];
        if taa {
            passes.push(FramePass::Taa);
        }
        passes.push(FramePass::Tonemap);
        // The screen GI's bounce, or GTAO's bent normals for the probe
        // GI, take the AO targets' gba.
        for (gi, bent) in [(false, false), (true, false), (false, true)] {
            let plan = build_graph(
                &passes,
                &GraphConfig {
                    bent,
                    ..cfg(taa, gi)
                },
            )
            .compile()
            .unwrap_or_else(|e| panic!("taa {taa} gi {gi} bent {bent}: {e}"));
            let at = |p: FramePass| plan.order().iter().position(|&i| passes[i] == p);
            assert!(at(FramePass::AoUpsample) < at(FramePass::Main));
            let format = plan.physical()[plan.slot("ao").expect("ao realized")].format;
            let want = if gi || bent {
                crate::ssgi::GI_FORMAT
            } else {
                AO_FORMAT
            };
            assert_eq!(format, want, "taa {taa} gi {gi} bent {bent}");
        }
    }
}

#[test]
fn smaa_t2x_keeps_reprojection_inputs_until_resolve() {
    for (velocity, chain_format) in [
        (false, LDR_FORMAT),
        (true, LDR_FORMAT),
        (false, HDR_FORMAT),
        (true, HDR_FORMAT),
    ] {
        let mut passes = Vec::new();
        if velocity {
            passes.extend([
                FramePass::Prepass,
                FramePass::Ao,
                FramePass::AoBlur,
                FramePass::AoUpsample,
            ]);
        }
        passes.extend([
            FramePass::Main,
            FramePass::Tonemap,
            FramePass::SmaaEdges,
            FramePass::SmaaWeights,
            FramePass::SmaaBlend,
            FramePass::SmaaResolve,
            FramePass::Lens,
            FramePass::Upscale,
        ]);
        let config = GraphConfig {
            chain_format,
            ..cfg(false, velocity)
        };
        let plan = build_graph(&passes, &config).compile().unwrap();
        assert_eq!(
            plan.physical()[plan.slot("ldr.0").unwrap()].format,
            chain_format
        );
        let names = plan.ordered_names();
        let at = |name| names.iter().position(|&n| n == name).unwrap();
        assert!(at("tonemap") < at("smaa-edges"));
        assert!(at("smaa-weights") < at("smaa-blend"));
        assert!(at("smaa-blend") < at("smaa-resolve"));
        assert!(at("smaa-resolve") < at("lens"));
        assert!(at("lens") < at("upscale"));
        assert_eq!(plan.lifetime("depth").unwrap().1, at("smaa-resolve"));
        assert_eq!(plan.lifetime("ldr.0").unwrap().1, at("smaa-blend"));
        assert!(!names.contains(&"taa"));
        if velocity {
            for name in ["gbuffer", "lindepth"] {
                assert_eq!(plan.lifetime(name).unwrap().1, at("smaa-resolve"));
            }
            // GI retains scene radiance, not the display-mapped SMAA frame.
            let hdr = &plan.physical()[plan.slot("hdr").unwrap()];
            assert!(hdr.usage.contains(wgpu::TextureUsages::COPY_SRC));
        }
    }
}

#[test]
fn smaa_keeps_the_color_source_alive_through_weights_and_upscale() {
    for stylise in [false, true] {
        let mut passes = vec![FramePass::Main, FramePass::Tonemap];
        if stylise {
            passes.extend([
                FramePass::Paint,
                FramePass::Lines,
                FramePass::Lens,
                FramePass::Quantize,
            ]);
        }
        passes.extend([
            FramePass::SmaaEdges,
            FramePass::SmaaWeights,
            FramePass::SmaaBlend,
            FramePass::Upscale,
        ]);
        let plan = build_graph(&passes, &cfg(false, false)).compile().unwrap();
        let names = plan.ordered_names();
        let at = |name| names.iter().position(|&n| n == name).unwrap();
        assert!(at("tonemap") < at("smaa-edges"));
        assert!(at("smaa-edges") < at("smaa-weights"));
        assert!(at("smaa-weights") < at("smaa-blend"));
        assert!(at("smaa-blend") < at("upscale"));
        // These are the named slots FrameGraph realizes into views, not
        // merely allocations that the binding lookup cannot reach.
        let realized: HashMap<_, _> = plan.transients().collect();
        let source = realized[ldr_name(if stylise { 4 } else { 0 }).as_str()];
        let edges = realized["smaa.edges"];
        let weights = realized["smaa.weights"];
        if stylise {
            assert!(realized.contains_key("ldr.5"));
        }
        assert!(!realized.contains_key("cascades"));
        assert_ne!(source, edges);
        assert_ne!(source, weights);
        assert_ne!(edges, weights);
        assert_eq!(plan.physical()[edges].format, wgpu::TextureFormat::Rg8Unorm);
        assert_eq!(
            plan.physical()[weights].format,
            wgpu::TextureFormat::Rgba8Unorm
        );
    }
}

/// The lens flare reads `bloom.0` after the prefilter wrote it and
/// before the last upsample adds onto it, with bloom's downsamples and
/// upsamples or without them; its targets are a quarter of the frame
/// after TAA (the output's under TAAU), and the tonemap reads the
/// blurred one.
#[test]
fn lens_flare_reads_the_first_bloom_level_before_the_upsamples() {
    for (bloom, taa) in [(true, false), (false, false), (true, true), (false, true)] {
        let mut passes = vec![FramePass::Prepass, FramePass::Main];
        if taa {
            passes.push(FramePass::Taa);
        }
        passes.extend([
            FramePass::BloomPrefilter,
            FramePass::FlareOcclusion,
            FramePass::Flare,
            FramePass::FlareBlurX,
            FramePass::FlareBlurY,
        ]);
        if bloom {
            passes.extend([FramePass::BloomDown(0), FramePass::BloomUp(0)]);
        }
        passes.push(FramePass::Tonemap);
        let mut config = cfg(taa, false);
        // TAAU: the frame after TAA twice the scene's.
        config.post = if taa { (128, 64) } else { (64, 32) };
        let plan = build_graph(&passes, &config)
            .compile()
            .unwrap_or_else(|e| panic!("bloom {bloom} taa {taa}: {e}"));
        let names = plan.ordered_names();
        let at = |name| names.iter().position(|&n| n == name).unwrap();
        assert!(at("bloom-prefilter") < at("flare"));
        assert!(at("flare-occlusion") < at("flare"));
        assert!(at("flare") < at("flare-blur-x"));
        assert!(at("flare-blur-x") < at("flare-blur-y"));
        assert!(at("flare-blur-y") < at("tonemap"));
        if bloom {
            assert!(at("flare") < at("bloom-up.0"), "{names:?}");
        }
        assert_eq!(plan.lifetime("flare").unwrap().1, at("tonemap"));
        assert_eq!(plan.lifetime("flare.vis").unwrap().1, at("flare"));
        let (pw, ph) = config.post;
        for name in ["flare.raw", "flare.x", "flare"] {
            let d = &plan.physical()[plan.slot(name).unwrap()];
            assert_eq!((d.width, d.height), (pw / 4, ph / 4), "{name}");
            assert_eq!(d.format, FLARE_FORMAT);
        }
        let vis = &plan.physical()[plan.slot("flare.vis").unwrap()];
        assert_eq!((vis.width, vis.height), (FLARE_LIGHTS as u32, 1));
    }
}

/// Outlines outside the silhouette: the seeds, six jumps back and forth
/// between the flood's two targets and the composite run in that order
/// after the outline pass, the last jump writes the `.a` the composite
/// reads (an odd step count would leave the field in `.b`, the
/// composite reading the next-to-last jump), and TAA takes the
/// composite's image.
#[test]
fn the_outline_flood_runs_between_the_outline_pass_and_taa() {
    let mut passes = vec![
        FramePass::Prepass,
        FramePass::Main,
        FramePass::Outline,
        FramePass::OutlineSeed,
    ];
    passes.extend((0..JFA_STEPS.len() as u32).map(FramePass::OutlineJump));
    passes.extend([FramePass::OutlineWide, FramePass::Taa, FramePass::Tonemap]);
    let plan = build_graph(&passes, &cfg(true, false)).compile().unwrap();
    let names = plan.ordered_names();
    let at = |name: &str| names.iter().position(|&n| n == name).unwrap();
    assert!(at("outline") < at("outline-seed"));
    for i in 0..JFA_STEPS.len() {
        let before = if i == 0 {
            "outline-seed".to_string()
        } else {
            format!("outline-jump.{}", i - 1)
        };
        assert!(at(&before) < at(&format!("outline-jump.{i}")));
    }
    assert!(at(&format!("outline-jump.{}", JFA_STEPS.len() - 1)) < at("outline-wide"));
    assert!(at("outline-wide") < at("taa"));
    assert_eq!(jfa_targets(JFA_STEPS.len() as u32 - 1).1, "outline-jfa.a");
    assert_eq!(lit_names(&passes).main, "hdr.wide");
    let format = |name| plan.slot(name).map(|s| plan.physical()[s].format);
    assert_eq!(format("outline-jfa.a"), Some(JFA_FORMAT));
    assert_eq!(format("outline-jfa.b"), Some(JFA_FORMAT));
    assert_ne!(plan.slot("outline-jfa.a"), plan.slot("outline-jfa.b"));
}

/// Contact shadows write `ao` (the AO in r, the lights' masks in gba),
/// after the AO passes, which then end in `ao.pre`, and before the main
/// pass that samples it; without AO the contact pass alone writes it.
#[test]
fn contact_shadows_take_the_ao_target_before_the_main_pass() {
    for ao in [false, true] {
        let mut passes = vec![FramePass::Prepass];
        if ao {
            passes.extend([FramePass::Ao, FramePass::AoBlur]);
        }
        passes.extend([FramePass::Contact, FramePass::Main, FramePass::Tonemap]);
        let plan = build_graph(&passes, &cfg(false, false))
            .compile()
            .unwrap_or_else(|e| panic!("ao {ao}: {e}"));
        let at = |p: FramePass| plan.order().iter().position(|&i| passes[i] == p);
        assert!(at(FramePass::Contact) < at(FramePass::Main));
        let format = |name| plan.slot(name).map(|s| plan.physical()[s].format);
        assert_eq!(format("ao"), Some(CONTACT_FORMAT), "ao {ao}");
        // The prepass holds `ao` in the fx group: it may not share a
        // texture with `outline-id` (the same format, unread without
        // an outline pass), which the prepass draws into.
        assert_ne!(plan.slot("ao"), plan.slot("outline-id"), "ao {ao}");
        if ao {
            assert!(at(FramePass::AoBlur) < at(FramePass::Contact));
            assert_eq!(format("ao.pre"), Some(AO_FORMAT));
            // Realized as a view, so the AO passes write it, not `ao`.
            assert!(plan.transients().any(|(name, _)| name == "ao.pre"));
        } else {
            assert_eq!(format("ao.pre"), None);
        }
    }
}
