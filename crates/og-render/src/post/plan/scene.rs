//! The passes that draw the scene: the compute before the frame, the
//! shadow maps, the prepass, what the main pass composes (particles, 2D
//! lights, fluids, clouds, reflections, views, the froxels), the main pass
//! itself and what is drawn over the finished frame.

use super::*;

pub(super) fn declare(g: &mut RenderGraph, cx: &PlanCx, p: FramePass) {
    let (passes, cfg) = (cx.passes, cx.cfg);
    let has = |p| cx.has(p);
    let (prepass, contact, ao_depth) = (cx.prepass, cx.contact, cx.ao_depth);
    let (msaa, gi_copy) = (cx.msaa, cx.gi_copy);
    let fields = cx.fields();
    let name = p.name();
    match p {
        // Into their own buffers, before every pass: declared first,
        // reading and writing no graph target, they keep their order at
        // the frame's start and leave the targets' lifetimes as they are.
        FramePass::CrowdCull
        | FramePass::CullCounts
        | FramePass::Wind
        | FramePass::WaterFields
        | FramePass::Trails
        | FramePass::Aurora
        | FramePass::CloudShadow
        | FramePass::Visual
        | FramePass::Surfaces
        | FramePass::TerrainCorners
        | FramePass::Grass
        | FramePass::Scatter => g.pass(name, &[], &[]),
        // Over the finished frame, after every pass that writes it:
        // into the output (or the glass source on its way there) and
        // their own targets, no transient one.
        FramePass::ViewportViews
        | FramePass::UiLayers
        | FramePass::Overlay
        | FramePass::HdrOutput => g.pass(name, &[], &["output"]),
        FramePass::Cascade(_) | FramePass::Shelter => g.pass(name, &[], &["cascades"]),
        FramePass::LocalFace(_) => g.pass(name, &[], &["local-shadows"]),
        FramePass::Prepass => {
            let mut writes = vec!["pre-depth", "gbuffer", "lindepth", "outline-id"];
            if cfg.object_space {
                writes.push("object-space");
            }
            if ao_depth {
                writes.push("ao-depth");
            }
            g.pass(name, &[], &writes);
            // It sets `fx_group`, which holds the main pass's AO (or
            // contact) target and planar reflection: neither may alias
            // a target the prepass draws into.
            let mut bound = Vec::new();
            if has(FramePass::AoBlur) || contact {
                bound.push("ao");
            }
            if has(FramePass::Reflection(0)) {
                bound.push(reflection_target(0, ""));
            }
            g.binds(&bound)
        }
        FramePass::Particles if prepass => g.pass(name, &["lindepth", "gbuffer"], &["particles"]),
        FramePass::Particles => g.pass(name, &[], &["particles"]),
        FramePass::Light2d if cfg.lightdir => g.pass(name, &[], &["lightmap", "lightdir"]),
        FramePass::Light2d => g.pass(name, &[], &["lightmap"]),
        FramePass::Metaballs => g.pass(name, &[], fields),
        FramePass::Fluid3d => g.pass(
            name,
            &["lindepth"],
            &[
                "fluid.depth",
                "fluid.zbuf",
                "fluid.thick",
                "fluid.blur",
                "fluid.smooth",
            ],
        ),
        // Into the probes' own cubes, not graph targets.
        FramePass::Probes => g.pass(name, &["cascades", "local-shadows"], &[]),
        // Into its own cubes and the bound IBL ones.
        FramePass::LiveSky => g.pass(name, &[], &[]),
        // With a prepass the march skips what its surfaces cover.
        FramePass::Clouds if prepass => g.pass(name, &["lindepth"], &["clouds"]),
        FramePass::Clouds => g.pass(name, &[], &["clouds"]),
        // Into the light volumes' buffer, not a graph target.
        FramePass::GiProbes => g.pass(name, &[], &[]),
        FramePass::Reflection(k) => {
            let mut writes = vec![reflection_target(k, ""), reflection_target(k, "-depth")];
            if msaa {
                writes.push(reflection_target(k, "-msaa"));
            }
            g.pass(name, &["cascades", "local-shadows"], &writes)
        }
        // Views write their own targets (outside the graph); the
        // imported "views" orders them before the main pass, whose
        // materials may sample them.
        FramePass::View(_) => g.pass(name, &["cascades", "local-shadows"], &["views"]),
        FramePass::Volumetric => g.pass(name, &["cascades", "local-shadows"], &["froxels"]),
        FramePass::Main => {
            let mut reads = vec!["cascades", "local-shadows"];
            if passes.iter().any(|p| matches!(p, FramePass::View(_))) {
                reads.push("views");
            }
            if has(FramePass::Volumetric) {
                reads.push("froxels");
            }
            if has(FramePass::AoBlur) || contact {
                reads.push("ao");
            }
            if prepass {
                reads.push("lindepth");
            }
            if has(FramePass::Particles) {
                reads.push("particles");
            }
            if has(FramePass::Light2d) {
                reads.push("lightmap");
                if cfg.lightdir {
                    reads.push("lightdir");
                }
            }
            if has(FramePass::Metaballs) {
                reads.extend_from_slice(fields);
            }
            for k in 0..2u8 {
                if has(FramePass::Reflection(k)) {
                    reads.push(reflection_target(k, ""));
                }
            }
            if has(FramePass::Fluid3d) {
                reads.extend(["fluid.smooth", "fluid.thick"]);
            }
            if has(FramePass::Ssr) {
                reads.push("ssr");
            }
            if has(FramePass::Clouds) {
                reads.push("clouds");
            }
            let mut writes = vec!["hdr", "depth"];
            if msaa {
                writes.push("hdr-msaa");
            }
            if cfg.water {
                writes.push("scene-copy");
            }
            if gi_copy {
                writes.push("gi-write");
            }
            if cfg.oit.is_some() {
                writes.extend(["oit.accum", "oit.reveal", "oit.front"]);
            }
            g.pass(name, &reads, &writes)
        }
        FramePass::Glow => g.pass(name, &[], &["glow"]),
        // Into the renderer's own targets (`ssr.rs`), the imported `ssr`.
        FramePass::Ssr => g.pass(
            name,
            &[
                "lindepth",
                "gbuffer",
                if cfg.taa { "history-read" } else { "gi-read" },
            ],
            &["ssr"],
        ),
        _ => unreachable!("{p:?} is not a scene pass"),
    };
}
