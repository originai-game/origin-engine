//! Frame pass names and render graph construction.

use super::*;

mod config;
mod names;
mod pass;
#[cfg(test)]
mod tests;
pub(crate) use config::*;
pub(crate) use names::*;
pub(crate) use pass::*;

/// The render graph of a frame made of `passes` (in recording order).
pub(crate) fn build_graph(passes: &[FramePass], cfg: &GraphConfig) -> RenderGraph {
    let attach_bind = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
    let desc = |width: u32, height: u32, format, samples, usage| TargetDesc {
        width,
        height,
        format,
        samples,
        usage,
    };
    let (w, h) = (cfg.width, cfg.height);
    let (hw, hh) = ((w / 2).max(1), (h / 2).max(1));
    // The frame after TAA (the output's under TAAU) and its half.
    let (pw, ph) = cfg.post;
    let (phw, phh) = ((pw / 2).max(1), (ph / 2).max(1));
    let mut g = RenderGraph::new();
    g.import("cascades")
        .import("local-shadows")
        .import("views")
        .import("output")
        .import("particles")
        .import("froxels")
        .transient(
            "hdr",
            desc(
                w,
                h,
                HDR_FORMAT,
                1,
                if cfg.water || cfg.transmission || ((cfg.gi || cfg.ssr) && !cfg.taa) {
                    attach_bind | wgpu::TextureUsages::COPY_SRC
                } else {
                    attach_bind
                },
            ),
        );
    if cfg.water {
        g.transient(
            "scene-copy",
            desc(
                w,
                h,
                HDR_FORMAT,
                1,
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            ),
        );
    }
    let msaa = cfg.samples > 1;
    if msaa {
        g.transient(
            "hdr-msaa",
            desc(
                w,
                h,
                HDR_FORMAT,
                cfg.samples,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            ),
        );
    }
    let smaa_t2x = passes.contains(&FramePass::SmaaResolve);
    let depth_usage = if ((cfg.taa || smaa_t2x) && !msaa) || cfg.oit == Some(true) {
        attach_bind
    } else {
        wgpu::TextureUsages::RENDER_ATTACHMENT
    };
    if cfg.oit.is_some() {
        g.transient(
            "oit.accum",
            desc(w, h, crate::oit::ACCUM_FORMAT, cfg.samples, attach_bind),
        )
        .transient(
            "oit.reveal",
            desc(w, h, crate::oit::REVEAL_FORMAT, cfg.samples, attach_bind),
        )
        .transient(
            "oit.front",
            desc(w, h, crate::oit::FRONT_FORMAT, cfg.samples, attach_bind),
        );
    }
    g.transient("depth", desc(w, h, DEPTH_FORMAT, cfg.samples, depth_usage));
    if cfg.taa {
        g.import("history-read").import("history-write");
    }
    if cfg.ssr {
        g.import("ssr");
    }
    if smaa_t2x {
        g.import("smaa-read").import("smaa-write");
    }
    // The GI's and the reflections' own copy of the lit image when there is
    // no TAA history.
    let gi_copy = (cfg.gi || cfg.ssr) && !cfg.taa;
    if gi_copy {
        g.import("gi-read").import("gi-write");
    }
    let has = |p| passes.contains(&p);
    if has(FramePass::Prepass) {
        g.transient(
            "pre-depth",
            desc(
                w,
                h,
                DEPTH_FORMAT,
                1,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            ),
        )
        .transient("gbuffer", desc(w, h, GBUFFER_FORMAT, 1, attach_bind))
        .transient(
            "lindepth",
            desc(
                w,
                h,
                LINDEPTH_FORMAT,
                1,
                attach_bind | wgpu::TextureUsages::COPY_SRC,
            ),
        )
        .transient("outline-id", desc(w, h, OUTLINE_FORMAT, 1, attach_bind));
        if cfg.object_space {
            g.transient(
                "object-space",
                desc(w, h, crate::stylise_object::OBJECT_FORMAT, 1, attach_bind),
            );
        }
    }
    let ao_format = if cfg.gi || cfg.bent {
        crate::ssgi::GI_FORMAT
    } else {
        AO_FORMAT
    };
    // With contact shadows the AO passes end in `ao.pre`, which the contact
    // pass copies into `ao`'s r beside the lights' masks.
    let contact = has(FramePass::Contact);
    let ao_out = ao_target_name(contact);
    if has(FramePass::AoUpsample) {
        // Searched and blurred at half resolution (each texel the depth's
        // at twice its coordinate), then upsampled to `ao`.
        let (hw, hh) = (w.div_ceil(2), h.div_ceil(2));
        g.transient("ao.raw", desc(hw, hh, ao_format, 1, attach_bind))
            .transient("ao.blur", desc(hw, hh, ao_format, 1, attach_bind))
            .transient(ao_out, desc(w, h, ao_format, 1, attach_bind));
    } else if has(FramePass::Ao) {
        g.transient("ao.raw", desc(w, h, ao_format, 1, attach_bind))
            .transient(ao_out, desc(w, h, ao_format, 1, attach_bind));
    }
    if contact {
        // The contact pass computes the masks (one per light, in its own
        // buffer: contact.rs `mask_buffer`) and composes them with the AO
        // into `ao`.
        let storage = wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING;
        g.transient("ao", desc(w, h, CONTACT_FORMAT, 1, attach_bind | storage));
    }
    if has(FramePass::Light2d) {
        let (lw, lh) = cfg.lightmap;
        g.transient("lightmap", desc(lw, lh, HDR_FORMAT, 1, attach_bind));
        if cfg.lightdir {
            g.transient("lightdir", desc(lw, lh, HDR_FORMAT, 1, attach_bind));
        }
    }
    let fields = &FIELD_NAMES[..(cfg.fluid_fields as usize).clamp(1, FIELD_NAMES.len())];
    if has(FramePass::Metaballs) {
        for name in fields {
            g.transient(*name, desc(w, h, HDR_FORMAT, 1, attach_bind));
        }
    }
    if has(FramePass::Fluid3d) {
        g.transient("fluid.depth", desc(w, h, LINDEPTH_FORMAT, 1, attach_bind))
            .transient(
                "fluid.zbuf",
                desc(
                    w,
                    h,
                    DEPTH_FORMAT,
                    1,
                    wgpu::TextureUsages::RENDER_ATTACHMENT,
                ),
            )
            .transient("fluid.thick", desc(w, h, HDR_FORMAT, 1, attach_bind))
            .transient("fluid.blur", desc(w, h, LINDEPTH_FORMAT, 1, attach_bind))
            .transient("fluid.smooth", desc(w, h, LINDEPTH_FORMAT, 1, attach_bind));
    }
    for k in 0..2u8 {
        let Some((rw, rh)) = cfg.reflection[k as usize].filter(|_| has(FramePass::Reflection(k)))
        else {
            continue;
        };
        g.transient(
            reflection_target(k, ""),
            desc(rw, rh, HDR_FORMAT, 1, attach_bind),
        )
        .transient(
            reflection_target(k, "-depth"),
            desc(
                rw,
                rh,
                DEPTH_FORMAT,
                cfg.samples,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            ),
        );
        if msaa {
            g.transient(
                reflection_target(k, "-msaa"),
                desc(
                    rw,
                    rh,
                    HDR_FORMAT,
                    cfg.samples,
                    wgpu::TextureUsages::RENDER_ATTACHMENT,
                ),
            );
        }
    }
    if has(FramePass::Glow) {
        g.transient("glow", desc(hw, hh, HDR_FORMAT, 1, attach_bind));
    }
    if has(FramePass::Clouds) {
        g.transient("clouds", desc(hw, hh, HDR_FORMAT, 1, attach_bind));
    }
    let chain = chain_passes(passes);
    for i in 0..chain.len() {
        g.transient(ldr_name(i), desc(pw, ph, cfg.chain_format, 1, attach_bind));
    }
    if has(FramePass::SmaaBlend) {
        g.transient(
            "smaa.edges",
            desc(pw, ph, wgpu::TextureFormat::Rg8Unorm, 1, attach_bind),
        );
        g.transient(
            "smaa.weights",
            desc(pw, ph, wgpu::TextureFormat::Rgba8Unorm, 1, attach_bind),
        );
    }
    if has(FramePass::DepthBands) {
        g.transient("hdr.bands", desc(w, h, HDR_FORMAT, 1, attach_bind));
    }
    if has(FramePass::OutlineSeed) {
        g.transient("outline-jfa.a", desc(w, h, JFA_FORMAT, 1, attach_bind))
            .transient("outline-jfa.b", desc(w, h, JFA_FORMAT, 1, attach_bind));
    }
    for (pass, name) in [
        (FramePass::Outline, "hdr.outline"),
        (FramePass::OutlineWide, "hdr.wide"),
        (FramePass::TaaSharpen, "hdr.sharp"),
        (FramePass::DofComposite, "hdr.dof"),
        (FramePass::MotionBlur, "hdr.mb"),
        (FramePass::ScreenBlur, "hdr.blur"),
    ] {
        if has(pass) {
            let (tw, th) = if pass.after_taa() { (pw, ph) } else { (w, h) };
            // FSR 3.1's RCAS writes the sharpening's target from compute.
            let usage = if pass == FramePass::TaaSharpen {
                attach_bind | wgpu::TextureUsages::STORAGE_BINDING
            } else {
                attach_bind
            };
            g.transient(name, desc(tw, th, HDR_FORMAT, 1, usage));
        }
    }
    if has(FramePass::DofCoc) {
        g.transient("dof.coc", desc(phw, phh, HDR_FORMAT, 1, attach_bind))
            .transient("dof.blur", desc(phw, phh, HDR_FORMAT, 1, attach_bind));
    }
    let customs = passes
        .iter()
        .filter(|p| matches!(p, FramePass::CustomPost(_)))
        .count();
    for &name in &CUSTOM_POST_NAMES[..customs] {
        g.transient(name, desc(pw, ph, HDR_FORMAT, 1, attach_bind));
    }
    if has(FramePass::GodRaysMask) {
        g.transient("god.mask", desc(phw, phh, HDR_FORMAT, 1, attach_bind))
            .transient("god.blur", desc(phw, phh, HDR_FORMAT, 1, attach_bind));
    }
    let mut size = METER_SIZE;
    for i in 0..meter_levels() {
        g.transient(
            luminance_name(i),
            desc(size, size, LUMINANCE_FORMAT, 1, attach_bind),
        );
        size /= 8;
    }
    for i in 0..bloom_levels(pw, ph) {
        g.transient(
            bloom_name(i),
            desc(
                (phw >> i).max(1),
                (phh >> i).max(1),
                HDR_FORMAT,
                1,
                attach_bind,
            ),
        );
    }
    if has(FramePass::Flare) {
        let (fw, fh) = flare_size(cfg.post);
        for name in ["flare.raw", "flare.x", "flare"] {
            g.transient(name, desc(fw, fh, FLARE_FORMAT, 1, attach_bind));
        }
    }
    if has(FramePass::FlareOcclusion) {
        g.transient(
            "flare.vis",
            desc(FLARE_LIGHTS as u32, 1, FLARE_VIS_FORMAT, 1, attach_bind),
        );
    }
    let names = lit_names(passes);
    let aa = names.aa.unwrap_or("history-write");
    let after_dof = names.after_dof.unwrap_or("history-write");
    let lit = names.last.unwrap_or("history-write");
    let auto = has(FramePass::Meter);
    let bloom = has(FramePass::BloomPrefilter);
    let metered = luminance_name(meter_levels() - 1);
    // What the exposure is read from: the adapted exposure when it adapts.
    let lum = if has(FramePass::Adapt) {
        g.import("exposure");
        "exposure".to_string()
    } else {
        metered.clone()
    };
    let prepass = has(FramePass::Prepass);
    // With water the AO darkens (`GraphConfig::water_ao`): the prepass depth with the water's surfaces in it, which
    // the AO reads (lib.rs, water.rs `draw_ao_depth`); the prepass pass writes it after its own targets.
    let ao_depth = prepass && cfg.water_ao && has(FramePass::Ao);
    if ao_depth {
        g.transient(
            "ao-depth",
            desc(
                w,
                h,
                LINDEPTH_FORMAT,
                1,
                attach_bind | wgpu::TextureUsages::COPY_DST,
            ),
        );
    }
    let mut ao_reads = vec![if ao_depth { "ao-depth" } else { "lindepth" }, "gbuffer"];
    // The GI reads the previous frame's lit image.
    if cfg.gi {
        ao_reads.push(if cfg.taa { "history-read" } else { "gi-read" });
    }
    for &p in passes {
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
            FramePass::Ao => g.pass(name, &ao_reads, &["ao.raw"]),
            FramePass::AoBlur if has(FramePass::AoUpsample) => {
                g.pass(name, &["ao.raw", "lindepth", "gbuffer"], &["ao.blur"])
            }
            FramePass::AoBlur => g.pass(name, &["ao.raw", "lindepth", "gbuffer"], &[ao_out]),
            FramePass::AoUpsample => g.pass(name, &["ao.blur", "lindepth"], &[ao_out]),
            FramePass::Contact if has(FramePass::AoBlur) => {
                g.pass(name, &["ao.pre", "lindepth", "gbuffer"], &["ao"])
            }
            FramePass::Contact => g.pass(name, &["lindepth", "gbuffer"], &["ao"]),
            FramePass::Particles if prepass => {
                g.pass(name, &["lindepth", "gbuffer"], &["particles"])
            }
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
            FramePass::DepthBands => g.pass(name, &["hdr", "lindepth"], &["hdr.bands"]),
            FramePass::Outline => g.pass(
                name,
                &[names.lit, "gbuffer", "lindepth", "outline-id"],
                &["hdr.outline"],
            ),
            FramePass::OutlineSeed => g.pass(
                name,
                &["lindepth", "gbuffer", "outline-id"],
                &["outline-jfa.a"],
            ),
            FramePass::OutlineJump(i) => {
                let (read, write) = jfa_targets(i);
                g.pass(name, &[read, "gbuffer"], &[write])
            }
            FramePass::OutlineWide => g.pass(
                name,
                &["hdr.outline", "outline-jfa.a", "gbuffer"],
                &["hdr.wide"],
            ),
            // With a prepass TAA binds its velocity and view depth.
            // The OIT revealage is its reactive mask (taa.wgsl).
            FramePass::Taa => {
                let mut reads = vec![names.main, "depth", "history-read"];
                if prepass {
                    reads.extend(["gbuffer", "lindepth"]);
                }
                if cfg.oit.is_some() {
                    reads.push("oit.reveal");
                }
                g.pass(name, &reads, &["history-write"])
            }
            FramePass::TaaSharpen if prepass => g.pass(
                name,
                &["history-write", "depth", "gbuffer", "lindepth"],
                &["hdr.sharp"],
            ),
            FramePass::TaaSharpen => g.pass(name, &["history-write", "depth"], &["hdr.sharp"]),
            // Every effect pass binds the prepass's lindepth and gbuffer
            // (`fx_group`), so each declares them: a target written here may
            // not alias a texture the pass still has bound.
            FramePass::DofCoc => g.pass(name, &[aa, "lindepth", "gbuffer"], &["dof.coc"]),
            FramePass::DofBlur => g.pass(name, &["dof.coc", "lindepth", "gbuffer"], &["dof.blur"]),
            FramePass::DofComposite => {
                g.pass(name, &[aa, "dof.blur", "lindepth", "gbuffer"], &["hdr.dof"])
            }
            FramePass::MotionBlur => g.pass(name, &[after_dof, "gbuffer", "lindepth"], &["hdr.mb"]),
            // The prepass's targets are bound when the frame has them
            // (`fx_group`); the screen blur alone does not ask for one.
            FramePass::ScreenBlur => {
                let after_mb = names.after_mb.unwrap_or("history-write");
                if prepass {
                    g.pass(name, &[after_mb, "gbuffer", "lindepth"], &["hdr.blur"])
                } else {
                    g.pass(name, &[after_mb], &["hdr.blur"])
                }
            }
            FramePass::CustomPost(i) => {
                let read = match i {
                    0 => names.before_custom.unwrap_or("history-write"),
                    i => CUSTOM_POST_NAMES[i as usize - 1],
                };
                g.pass(
                    name,
                    &[read, "lindepth", "gbuffer"],
                    &[CUSTOM_POST_NAMES[i as usize]],
                )
            }
            FramePass::GodRaysMask => g.pass(name, &[lit, "lindepth", "gbuffer"], &["god.mask"]),
            FramePass::GodRaysBlur => {
                g.pass(name, &["god.mask", "lindepth", "gbuffer"], &["god.blur"])
            }
            FramePass::Meter => g.pass(name, &[lit], &["luminance.0"]),
            FramePass::Reduce(i) => g.pass(name, &[&luminance_name(i)], &[&luminance_name(i + 1)]),
            FramePass::Adapt => g.pass(name, &[&metered], &["exposure"]),
            FramePass::BloomPrefilter => {
                let mut reads = vec![lit];
                if auto {
                    reads.push(&lum);
                }
                if has(FramePass::Glow) {
                    reads.push("glow");
                }
                g.pass(name, &reads, &["bloom.0"])
            }
            FramePass::BloomDown(i) => g.pass(name, &[&bloom_name(i)], &[&bloom_name(i + 1)]),
            FramePass::BloomUp(i) => g.pass(name, &[&bloom_name(i + 1)], &[&bloom_name(i)]),
            FramePass::FlareOcclusion => g.pass(name, &[lit, "lindepth"], &["flare.vis"]),
            // Declared before the upsamples, so the last of them (onto
            // `bloom.0`) waits for it.
            FramePass::Flare => {
                let mut reads = vec!["bloom.0"];
                if prepass {
                    reads.push("lindepth");
                }
                if auto {
                    reads.push(&lum);
                }
                if has(FramePass::FlareOcclusion) {
                    reads.push("flare.vis");
                }
                g.pass(name, &reads, &["flare.raw"])
            }
            FramePass::FlareBlurX => g.pass(name, &["flare.raw"], &["flare.x"]),
            FramePass::FlareBlurY => g.pass(name, &["flare.x"], &["flare"]),
            FramePass::Tonemap => {
                let mut reads = vec![lit];
                if auto {
                    reads.push(&lum);
                }
                if bloom {
                    reads.push("bloom.0");
                }
                if has(FramePass::GodRaysBlur) {
                    reads.push("god.blur");
                }
                if has(FramePass::FlareBlurY) {
                    reads.push("flare");
                }
                if cfg.ui_depth && prepass {
                    reads.push("lindepth");
                }
                let target = if chain.is_empty() {
                    "output".to_string()
                } else {
                    ldr_name(0)
                };
                g.pass(name, &reads, &[target.as_str()])
            }
            FramePass::SmaaEdges => {
                let i = chain
                    .iter()
                    .position(|&c| c == FramePass::SmaaBlend || c == FramePass::SmaaResolve)
                    .expect("SMAA in chain");
                g.pass(name, &[&ldr_name(i)], &["smaa.edges"])
            }
            FramePass::SmaaWeights => g.pass(name, &["smaa.edges"], &["smaa.weights"]),
            FramePass::SmaaBlend if smaa_t2x => {
                g.pass(name, &["ldr.0", "smaa.weights"], &["smaa-write"])
            }
            FramePass::SmaaResolve => {
                let mut reads = vec!["smaa-write", "smaa-read", "depth"];
                if prepass {
                    reads.extend(["gbuffer", "lindepth"]);
                }
                let write = if chain.len() == 1 { "output" } else { "ldr.1" };
                g.pass(name, &reads, &[write])
            }
            FramePass::Paint
            | FramePass::Watercolor
            | FramePass::Lines
            | FramePass::Lens
            | FramePass::Quantize
            | FramePass::Fxaa
            | FramePass::SmaaBlend
            | FramePass::Upscale => {
                let i = chain.iter().position(|&c| c == p).expect("in the chain");
                let read = ldr_name(i);
                let write = if i + 1 == chain.len() {
                    "output".to_string()
                } else {
                    ldr_name(i + 1)
                };
                // The paint and lines passes bind the prepass
                // (`Bindings::chain`).
                let mut reads = vec![read.as_str()];
                // The quantize and lines passes also read the scene before
                // exposure and the tone curve (conformed art keeps its colour).
                if matches!(p, FramePass::Quantize | FramePass::Lines) {
                    reads.push(untoned(passes));
                }
                if p == FramePass::SmaaBlend {
                    reads.push("smaa.weights");
                }
                if matches!(
                    p,
                    FramePass::Paint | FramePass::Watercolor | FramePass::Lines
                ) && prepass
                {
                    reads.extend(["lindepth", "gbuffer", "outline-id"]);
                    if cfg.object_space {
                        reads.push("object-space");
                    }
                }
                g.pass(name, &reads, &[write.as_str()])
            }
        };
    }
    g
}
