//! The targets of a frame's graph, declared in one order: the frame's own,
//! the AO's, what the main pass composes, then the effects'.

use super::*;

const ATTACH_BIND: wgpu::TextureUsages =
    wgpu::TextureUsages::RENDER_ATTACHMENT.union(wgpu::TextureUsages::TEXTURE_BINDING);

fn desc(
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    samples: u32,
    usage: wgpu::TextureUsages,
) -> TargetDesc {
    TargetDesc {
        width,
        height,
        format,
        samples,
        usage,
    }
}

pub(super) fn declare(g: &mut RenderGraph, cx: &PlanCx) {
    frame(g, cx);
    ao(g, cx);
    composed(g, cx);
    effects(g, cx);
    front_depth(g, cx);
}

/// The imported targets, the lit image, the depth, the temporal histories
/// and the prepass.
fn frame(g: &mut RenderGraph, cx: &PlanCx) {
    let cfg = cx.cfg;
    let has = |p| cx.has(p);
    let (msaa, smaa_t2x, gi_copy) = (cx.msaa, cx.smaa_t2x, cx.gi_copy);
    let (w, h) = (cfg.width, cfg.height);
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
                    ATTACH_BIND | wgpu::TextureUsages::COPY_SRC
                } else {
                    ATTACH_BIND
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
    let mut depth_usage =
        if ((cfg.taa || smaa_t2x) && !msaa) || cfg.oit == Some(true) || cfg.transmission {
            ATTACH_BIND
        } else {
            wgpu::TextureUsages::RENDER_ATTACHMENT
        };
    if cfg.taa {
        depth_usage |= wgpu::TextureUsages::COPY_SRC;
    }
    if cfg.oit.is_some() {
        g.transient(
            "oit.accum",
            desc(w, h, crate::oit::ACCUM_FORMAT, cfg.samples, ATTACH_BIND),
        )
        .transient(
            "oit.reveal",
            desc(w, h, crate::oit::REVEAL_FORMAT, cfg.samples, ATTACH_BIND),
        )
        .transient(
            "oit.front",
            desc(w, h, crate::oit::FRONT_FORMAT, cfg.samples, ATTACH_BIND),
        );
    }
    if cfg.fx_half {
        let (hw, hh) = (w.div_ceil(2), h.div_ceil(2));
        g.transient(
            "fx.half_depth",
            desc(hw, hh, LINDEPTH_FORMAT, 1, ATTACH_BIND),
        )
        .transient("fx.half", desc(hw, hh, HDR_FORMAT, 1, ATTACH_BIND));
        if cfg.taa && cfg.oit.is_none() {
            g.transient(
                "fx.reactive",
                desc(w, h, crate::oit::REVEAL_FORMAT, 1, ATTACH_BIND),
            );
        }
    }
    g.transient("depth", desc(w, h, DEPTH_FORMAT, cfg.samples, depth_usage));
    if cfg.taa {
        g.import("history-read")
            .import("history-write")
            .import("history-depth-read")
            .import("history-depth-write");
    }
    if cfg.ssr {
        g.import("ssr");
    }
    // The graph's own texture (FrameGraph::depth_mips).
    if has(FramePass::DepthMips) {
        g.import(crate::post::depth_mips::DEPTH_MIPS_TARGET);
    }
    if smaa_t2x {
        g.import("smaa-read").import("smaa-write");
    }
    // The GI's and the reflections' own copy of the lit image when there is
    // no TAA history.
    if gi_copy {
        g.import("gi-read").import("gi-write");
    }
    if has(FramePass::Prepass) {
        // A single-sampled scene's prepass draws into `depth` itself
        // (shares_prepass_depth).
        if msaa {
            g.transient(
                "pre-depth",
                desc(
                    w,
                    h,
                    DEPTH_FORMAT,
                    1,
                    wgpu::TextureUsages::RENDER_ATTACHMENT,
                ),
            );
        }
        g.transient("gbuffer", desc(w, h, GBUFFER_FORMAT, 1, ATTACH_BIND))
            .transient(
                "lindepth",
                desc(
                    w,
                    h,
                    LINDEPTH_FORMAT,
                    1,
                    // STORAGE_BINDING: the main pass's cards read it
                    // (impostor_kept.wgsl `imp_view_depth`).
                    ATTACH_BIND
                        | wgpu::TextureUsages::COPY_SRC
                        | if cfg.lindepth_storage {
                            wgpu::TextureUsages::STORAGE_BINDING
                        } else {
                            wgpu::TextureUsages::empty()
                        },
                ),
            )
            .transient("outline-id", desc(w, h, OUTLINE_FORMAT, 1, ATTACH_BIND))
            .transient(
                "motion-depth",
                desc(w, h, MOTION_DEPTH_FORMAT, 1, ATTACH_BIND),
            );
        if cfg.object_space {
            g.transient("object-space", desc(w, h, OBJECT_FORMAT, 1, ATTACH_BIND));
        }
    }
}

/// The AO's targets, and the contact shadows' in their place.
fn ao(g: &mut RenderGraph, cx: &PlanCx) {
    let cfg = cx.cfg;
    let has = |p| cx.has(p);
    let (contact, ao_out) = (cx.contact, cx.ao_out);
    let (w, h) = (cfg.width, cfg.height);
    let ao_format = if cfg.gi || cfg.bent {
        crate::ssgi::GI_FORMAT
    } else {
        AO_FORMAT
    };
    if has(FramePass::AoUpsample) {
        // Searched and blurred at half resolution (each texel the depth's
        // at twice its coordinate), then upsampled to `ao`.
        let (hw, hh) = (w.div_ceil(2), h.div_ceil(2));
        g.transient("ao.raw", desc(hw, hh, ao_format, 1, ATTACH_BIND))
            .transient("ao.blur", desc(hw, hh, ao_format, 1, ATTACH_BIND))
            .transient(ao_out, desc(w, h, ao_format, 1, ATTACH_BIND));
    } else if has(FramePass::Ao) {
        g.transient("ao.raw", desc(w, h, ao_format, 1, ATTACH_BIND))
            .transient(ao_out, desc(w, h, ao_format, 1, ATTACH_BIND));
    }
    if contact {
        // The contact pass computes the masks (one per light, in its own
        // buffer: contact.rs `mask_buffer`) and composes them with the AO
        // into `ao`.
        let storage = wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING;
        g.transient("ao", desc(w, h, CONTACT_FORMAT, 1, ATTACH_BIND | storage));
    }
}

/// What the main pass composes: the 2D lights, the fluids, the planar
/// reflections, the glow and the clouds.
fn composed(g: &mut RenderGraph, cx: &PlanCx) {
    let cfg = cx.cfg;
    let has = |p| cx.has(p);
    let msaa = cx.msaa;
    let fields = cx.fields();
    let (w, h) = (cfg.width, cfg.height);
    let (hw, hh) = ((w / 2).max(1), (h / 2).max(1));
    if has(FramePass::Light2d) {
        let (lw, lh) = cfg.lightmap;
        g.transient("lightmap", desc(lw, lh, HDR_FORMAT, 1, ATTACH_BIND));
        if cfg.lightdir {
            g.transient("lightdir", desc(lw, lh, HDR_FORMAT, 1, ATTACH_BIND));
        }
    }
    if has(FramePass::Metaballs) {
        for name in fields {
            g.transient(*name, desc(w, h, HDR_FORMAT, 1, ATTACH_BIND));
        }
    }
    if has(FramePass::Fluid3d) {
        g.transient("fluid.depth", desc(w, h, LINDEPTH_FORMAT, 1, ATTACH_BIND))
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
            .transient("fluid.thick", desc(w, h, HDR_FORMAT, 1, ATTACH_BIND))
            .transient("fluid.blur", desc(w, h, LINDEPTH_FORMAT, 1, ATTACH_BIND))
            .transient("fluid.smooth", desc(w, h, LINDEPTH_FORMAT, 1, ATTACH_BIND));
    }
    for k in 0..2u8 {
        let Some((rw, rh)) = cfg.reflection[k as usize].filter(|_| has(FramePass::Reflection(k)))
        else {
            continue;
        };
        g.transient(
            reflection_target(k, ""),
            desc(rw, rh, HDR_FORMAT, 1, ATTACH_BIND),
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
        g.transient("glow", desc(hw, hh, HDR_FORMAT, 1, ATTACH_BIND));
    }
    if has(FramePass::Clouds) {
        // Copied into the clouds' history after the resolve (clouds.rs `CloudsTemporal`).
        let (cw, ch) = crate::clouds::low_size(w, h);
        g.transient(
            "clouds",
            desc(
                cw,
                ch,
                HDR_FORMAT,
                1,
                ATTACH_BIND | wgpu::TextureUsages::COPY_SRC,
            ),
        );
        let (qw, qh) = crate::clouds::quarter_size((cw, ch));
        g.transient("clouds.quarter", desc(qw, qh, HDR_FORMAT, 1, ATTACH_BIND));
    }
}

/// The effects' targets, from the chain after the tonemap back to the
/// exposure.
fn effects(g: &mut RenderGraph, cx: &PlanCx) {
    let (passes, cfg) = (cx.passes, cx.cfg);
    let has = |p| cx.has(p);
    let chain = &cx.chain;
    let (w, h) = (cfg.width, cfg.height);
    // The frame after TAA (the output's under TAAU) and its half.
    let (pw, ph) = cfg.post;
    let (phw, phh) = ((pw / 2).max(1), (ph / 2).max(1));
    for i in 0..chain.len() {
        g.transient(ldr_name(i), desc(pw, ph, cfg.chain_format, 1, ATTACH_BIND));
    }
    if has(FramePass::SmaaBlend) {
        g.transient(
            "smaa.edges",
            desc(pw, ph, wgpu::TextureFormat::Rg8Unorm, 1, ATTACH_BIND),
        );
        g.transient(
            "smaa.weights",
            desc(pw, ph, wgpu::TextureFormat::Rgba8Unorm, 1, ATTACH_BIND),
        );
    }
    if has(FramePass::DepthBands) {
        g.transient("hdr.bands", desc(w, h, HDR_FORMAT, 1, ATTACH_BIND));
    }
    if has(FramePass::DepthRange(0)) {
        for (name, size) in [
            ("depth.range0", 64),
            ("depth.range1", 8),
            ("depth.range2", 1),
        ] {
            g.transient(name, desc(size, size, HDR_FORMAT, 1, ATTACH_BIND));
        }
    }
    if has(FramePass::FogOfWar) {
        g.transient("hdr.sight", desc(w, h, HDR_FORMAT, 1, ATTACH_BIND));
    }
    if has(FramePass::OutlineSeed) {
        g.transient("outline-jfa.a", desc(w, h, JFA_FORMAT, 1, ATTACH_BIND))
            .transient("outline-jfa.b", desc(w, h, JFA_FORMAT, 1, ATTACH_BIND));
    }
    for (pass, name) in [
        (FramePass::Outline, "hdr.outline"),
        (FramePass::OutlineWide, "hdr.wide"),
        (FramePass::DofComposite, "hdr.dof"),
        (FramePass::MotionBlur, "hdr.mb"),
        (FramePass::ScreenBlur, "hdr.blur"),
    ] {
        if has(pass) {
            let (tw, th) = if pass.after_taa() { (pw, ph) } else { (w, h) };
            g.transient(name, desc(tw, th, HDR_FORMAT, 1, ATTACH_BIND));
        }
    }
    if has(FramePass::ScreenBlur) && cfg.zoom_blur {
        for name in ["zoom.0", "zoom.1"] {
            g.transient(name, desc(pw, ph, HDR_FORMAT, 1, ATTACH_BIND));
        }
    }
    if has(FramePass::DofCoc) {
        g.transient("dof.coc", desc(phw, phh, HDR_FORMAT, 1, ATTACH_BIND))
            .transient("dof.blur", desc(phw, phh, HDR_FORMAT, 1, ATTACH_BIND));
    }
    if has(FramePass::MotionBlur) {
        let (mw, mh) = (pw.div_ceil(cfg.motion_tile), ph.div_ceil(cfg.motion_tile));
        g.transient("motion.x", desc(mw, ph, MOTION_TILE_FORMAT, 1, ATTACH_BIND))
            .transient(
                "motion.tile",
                desc(mw, mh, MOTION_TILE_FORMAT, 1, ATTACH_BIND),
            )
            .transient(
                "motion.neighbor",
                desc(mw, mh, MOTION_TILE_FORMAT, 1, ATTACH_BIND),
            );
    }
    let customs = passes
        .iter()
        .filter(|p| matches!(p, FramePass::CustomPost(_)))
        .count();
    for &name in &CUSTOM_POST_NAMES[..customs] {
        g.transient(name, desc(pw, ph, HDR_FORMAT, 1, ATTACH_BIND));
    }
    if has(FramePass::GodRaysMask) {
        g.transient(
            "god.mask",
            desc(phw, phh, cfg.rgb_hdr_format, 1, ATTACH_BIND),
        )
        .transient(
            "god.blur",
            desc(phw, phh, cfg.rgb_hdr_format, 1, ATTACH_BIND),
        )
        .transient(
            "god.resolve",
            desc(phw, phh, cfg.rgb_hdr_format, 1, ATTACH_BIND),
        );
    }
    if has(FramePass::Reduce(0)) {
        let mut size = METER_SIZE;
        for i in 0..meter_levels() {
            g.transient(
                luminance_name(i),
                desc(size, size, LUMINANCE_FORMAT, 1, ATTACH_BIND),
            );
            size /= 8;
        }
    } else if has(FramePass::Meter) {
        g.transient(
            luminance_name(0),
            desc(
                1,
                1,
                crate::post::exposure_hist::HIST_METER_FORMAT,
                1,
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            ),
        );
    }
    if cfg.compute_bloom {
        if has(FramePass::BloomPrefilter) {
            g.import(crate::post::bloom_spd::BLOOM_DOWN_TARGET);
        }
        if has(FramePass::BloomUp(0)) {
            g.import(crate::post::bloom_spd::BLOOM_UP_TARGET);
        }
    } else {
        for i in 0..bloom_levels(pw, ph) {
            g.transient(
                bloom_name(i),
                desc(
                    (phw >> i).max(1),
                    (phh >> i).max(1),
                    cfg.rgb_hdr_format,
                    1,
                    ATTACH_BIND,
                ),
            );
        }
    }
    if has(FramePass::Flare) {
        let (fw, fh) = flare_size(cfg.post);
        for name in ["flare.raw", "flare.x", "flare"] {
            g.transient(name, desc(fw, fh, FLARE_FORMAT, 1, ATTACH_BIND));
        }
    }
    if has(FramePass::FlareOcclusion) {
        g.transient(
            "flare.vis",
            desc(FLARE_LIGHTS as u32, 1, FLARE_VIS_FORMAT, 1, ATTACH_BIND),
        );
    }
    if has(FramePass::Adapt) {
        g.import("exposure");
    }
}

/// The prepass depth with the water's surfaces in front, which the AO and
/// the volumetric fog read ([`PlanCx::front_depth`]).
fn front_depth(g: &mut RenderGraph, cx: &PlanCx) {
    let cfg = cx.cfg;
    let (w, h) = (cfg.width, cfg.height);
    if cx.front_depth {
        g.transient(
            "front-depth",
            desc(
                w,
                h,
                LINDEPTH_FORMAT,
                1,
                ATTACH_BIND | wgpu::TextureUsages::COPY_DST,
            ),
        );
    }
}
