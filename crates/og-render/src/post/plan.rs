//! Frame pass names and render graph construction.

use super::*;

/// One pass of a frame, as the render graph knows it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum FramePass {
    /// Directional light cascade `i`.
    Cascade(u32),
    /// Local light shadow layer (6 per point light, 1 per spot).
    LocalFace(u32),
    /// The depth-normal prepass (normal, velocity, view depth, outline ids).
    Prepass,
    /// Ambient occlusion (GTAO or SSAO), then its blur.
    Ao,
    AoBlur,
    /// Half-resolution ambient occlusion's depth-aware upsample
    /// ([`crate::AoResolution::Half`]): with it, `ao` and its blur run at
    /// half resolution and this pass writes the full-resolution `ao`.
    AoUpsample,
    /// The particle simulation (compute).
    Particles,
    /// The 2D light map.
    Light2d,
    /// The 2D fluids' metaball field.
    Metaballs,
    /// The 3D fluids' depth, thickness and smoothing (fluid3d.rs).
    Fluid3d,
    /// The reflection probes' captures due this frame (`probes.rs`).
    Probes,
    /// The scene mirrored in a water surface (its planar reflection): slot 0 the one material shaders read, slot 1 a
    /// second surface's (targets `reflection*`, `reflection.1*`).
    Reflection(u8),
    /// Secondary camera `i` of the frame's views (`views.rs`): its lit
    /// pass and its resolve into a render target (a viewport view resolves
    /// after the post chain).
    View(u32),
    /// Volumetric fog's froxel grid lit and summed (compute).
    Volumetric,
    /// Opaque meshes, sky, transparent meshes, particles, strips, sprites
    /// and text into the HDR target.
    Main,
    /// Screen-space reflections over the lit image (`ssr.rs`).
    Ssr,
    /// Glowing sprites into the glow buffer.
    Glow,
    Outline,
    Taa,
    /// The sharpening after TAA (into `hdr.sharp`; the history stays soft).
    TaaSharpen,
    DofCoc,
    DofBlur,
    DofComposite,
    MotionBlur,
    /// The zoom, spin or directional blur of the whole frame
    /// (`ScreenBlur`), after motion blur.
    ScreenBlur,
    /// Custom post pass `i` of the frame (`post_custom.rs`): a game's WGSL
    /// over the lit image, after motion blur and the screen blur, before
    /// exposure and bloom.
    CustomPost(u32),
    GodRaysMask,
    GodRaysBlur,
    Meter,
    /// Metering level `i` into `i + 1`.
    Reduce(u32),
    /// Temporal exposure adaptation: the metered exposure approached from
    /// the previous frame's (`Exposure::Auto` with adapt speeds).
    Adapt,
    BloomPrefilter,
    /// Bloom level `i` into `i + 1`.
    BloomDown(u32),
    /// Bloom level `i + 1` added onto `i`.
    BloomUp(u32),
    /// Flat colour bands by depth over the lit image (`stylise.rs`).
    DepthBands,
    Tonemap,
    /// The passes after the tonemap ([`chain_passes`]), each from one LDR
    /// target into the next, the last into the output: painterly
    /// smoothing, hatching/halftone/ink lines, lens distortion (after the
    /// passes that read the depth, so their lines bend with the image),
    /// posterise and palette quantisation, FXAA, and the pixel grid scaled
    /// up to the output.
    Paint,
    Lines,
    Lens,
    Quantize,
    /// FXAA over the tonemapped image (an alternative to TAA and MSAA).
    Fxaa,
    Upscale,
}

/// The passes after the tonemap that `passes` holds, in the order they run.
pub(crate) fn chain_passes(passes: &[FramePass]) -> Vec<FramePass> {
    [
        FramePass::Paint,
        FramePass::Lines,
        FramePass::Lens,
        FramePass::Quantize,
        FramePass::Fxaa,
        FramePass::Upscale,
    ]
    .into_iter()
    .filter(|p| passes.contains(p))
    .collect()
}

/// The LDR target chain pass `i` reads (the tonemap writes `ldr.0`).
pub(super) fn ldr_name(i: usize) -> String {
    format!("ldr.{i}")
}

impl FramePass {
    pub fn name(self) -> String {
        match self {
            FramePass::Cascade(i) => format!("cascade.{i}"),
            FramePass::LocalFace(i) => format!("local-shadow.{i}"),
            FramePass::Prepass => "prepass".into(),
            FramePass::Ao => "ao".into(),
            FramePass::AoBlur => "ao-blur".into(),
            FramePass::AoUpsample => "ao-upsample".into(),
            FramePass::Particles => "particles".into(),
            FramePass::Light2d => "lights-2d".into(),
            FramePass::Metaballs => "metaballs".into(),
            FramePass::Fluid3d => "fluid-3d".into(),
            FramePass::Probes => "reflection-probes".into(),
            FramePass::Reflection(k) => reflection_target(k, "").into(),
            FramePass::View(i) => format!("view-{i}"),
            FramePass::Volumetric => "volumetric".into(),
            FramePass::Main => "main".into(),
            FramePass::Ssr => "ssr".into(),
            FramePass::Glow => "glow".into(),
            FramePass::Outline => "outline".into(),
            FramePass::Taa => "taa".into(),
            FramePass::TaaSharpen => "taa-sharpen".into(),
            FramePass::DofCoc => "dof-coc".into(),
            FramePass::DofBlur => "dof-blur".into(),
            FramePass::DofComposite => "dof-composite".into(),
            FramePass::MotionBlur => "motion-blur".into(),
            FramePass::ScreenBlur => "screen-blur".into(),
            FramePass::CustomPost(i) => format!("custom-post.{i}"),
            FramePass::GodRaysMask => "god-rays-mask".into(),
            FramePass::GodRaysBlur => "god-rays-blur".into(),
            FramePass::Meter => "meter".into(),
            FramePass::Reduce(i) => format!("meter-reduce.{i}"),
            FramePass::Adapt => "exposure-adapt".into(),
            FramePass::BloomPrefilter => "bloom-prefilter".into(),
            FramePass::BloomDown(i) => format!("bloom-down.{i}"),
            FramePass::BloomUp(i) => format!("bloom-up.{i}"),
            FramePass::Tonemap => "tonemap".into(),
            FramePass::DepthBands => "depth-bands".into(),
            FramePass::Paint => "paint".into(),
            FramePass::Lines => "lines".into(),
            FramePass::Lens => "lens".into(),
            FramePass::Quantize => "quantize".into(),
            FramePass::Fxaa => "fxaa".into(),
            FramePass::Upscale => "upscale".into(),
        }
    }
}

/// What the transient targets depend on besides the passes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GraphConfig {
    pub width: u32,
    pub height: u32,
    pub samples: u32,
    pub taa: bool,
    /// The 2D light map's size.
    pub lightmap: (u32, u32),
    /// GTAO (or SSAO).
    pub gtao: bool,
    /// The main pass draws water or a 3D fluid: it copies the scene drawn
    /// so far.
    pub water: bool,
    /// The screen AO sees the water's surfaces (`ao-depth`; water.rs
    /// `draw_ao_depth`): water drawn with `PostEffects::ao.direct` above 0,
    /// the three.js-era AO that darkened the water's pixel whole. At 0 the
    /// water, drawn after the AO is applied, is outside it.
    pub water_ao: bool,
    /// The main pass copies the opaque scene for transmissive materials
    /// (into the renderer's own mip-chained target, not a graph target).
    pub transmission: bool,
    /// Metaball field targets the frame's fluids need (1 to
    /// `metaballs::FIELDS` when it draws any).
    pub fluid_fields: u32,
    /// The planar reflections' sizes (`Some` with [`FramePass::Reflection`] of that slot).
    pub reflection: [Option<(u32, u32)>; 2],
    /// Stylise lays a pattern in object space: the prepass also writes
    /// `object-space` (stylise_object.rs), which the paint and lines
    /// passes read.
    pub object_space: bool,
    /// The overlay reads the prepass's view depth (UI occluders and mass
    /// elements, after every pass): `lindepth` lives to the frame's end.
    pub ui_depth: bool,
}

/// The metaball field targets, by field.
pub(crate) const FIELD_NAMES: [&str; crate::FIELDS] =
    ["metaballs", "metaballs.1", "metaballs.2", "metaballs.3"];

pub(super) fn luminance_name(i: u32) -> String {
    format!("luminance.{i}")
}

pub(super) fn bloom_name(i: u32) -> String {
    format!("bloom.{i}")
}

/// The names of the lit image as the effects hand it on: after the main
/// pass (and outlines), after TAA (`None`: the history; its sharpening:
/// `hdr.sharp`), after depth of field and motion blur.
pub(super) struct LitNames {
    /// The lit image after the main pass and screen-space reflections.
    pub(super) base: &'static str,
    /// The lit image after the main pass, reflections and the depth bands.
    pub(super) lit: &'static str,
    pub(super) main: &'static str,
    /// `None` when TAA writes the history and nothing sharpens it.
    pub(super) aa: Option<&'static str>,
    pub(super) after_dof: Option<&'static str>,
    /// The lit image after motion blur (the screen blur reads it).
    pub(super) after_mb: Option<&'static str>,
    /// The lit image the first custom post pass reads (after motion blur
    /// and the screen blur).
    pub(super) before_custom: Option<&'static str>,
    pub(super) last: Option<&'static str>,
}

/// The targets the custom post passes write, in order (`post_custom.rs`).
pub(crate) const CUSTOM_POST_NAMES: [&str; crate::post_custom::MAX_CUSTOM_POSTS] = [
    "hdr.post.0",
    "hdr.post.1",
    "hdr.post.2",
    "hdr.post.3",
    "hdr.post.4",
    "hdr.post.5",
    "hdr.post.6",
    "hdr.post.7",
];

pub(super) fn lit_names(passes: &[FramePass]) -> LitNames {
    let has = |p| passes.contains(&p);
    let base = if has(FramePass::Ssr) {
        "hdr.ssr"
    } else {
        "hdr"
    };
    let lit = if has(FramePass::DepthBands) {
        "hdr.bands"
    } else {
        base
    };
    let main = if has(FramePass::Outline) {
        "hdr.outline"
    } else {
        lit
    };
    let aa = if has(FramePass::TaaSharpen) {
        Some("hdr.sharp")
    } else {
        (!has(FramePass::Taa)).then_some(main)
    };
    let after_dof = if has(FramePass::DofComposite) {
        Some("hdr.dof")
    } else {
        aa
    };
    let after_mb = if has(FramePass::MotionBlur) {
        Some("hdr.mb")
    } else {
        after_dof
    };
    let before_custom = if has(FramePass::ScreenBlur) {
        Some("hdr.blur")
    } else {
        after_mb
    };
    let customs = passes
        .iter()
        .filter(|p| matches!(p, FramePass::CustomPost(_)))
        .count();
    let last = match customs {
        0 => before_custom,
        n => Some(CUSTOM_POST_NAMES[n - 1]),
    };
    LitNames {
        base,
        lit,
        main,
        aa,
        after_dof,
        after_mb,
        before_custom,
        last,
    }
}

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
                if cfg.water || cfg.transmission {
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
    let depth_usage = if cfg.taa && !msaa {
        attach_bind
    } else {
        wgpu::TextureUsages::RENDER_ATTACHMENT
    };
    g.transient("depth", desc(w, h, DEPTH_FORMAT, cfg.samples, depth_usage));
    if cfg.taa {
        g.import("history-read").import("history-write");
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
    if has(FramePass::AoUpsample) {
        // Searched and blurred at half resolution (each texel the depth's
        // at twice its coordinate), then upsampled to `ao`.
        let (hw, hh) = (w.div_ceil(2), h.div_ceil(2));
        g.transient("ao.raw", desc(hw, hh, AO_FORMAT, 1, attach_bind))
            .transient("ao.blur", desc(hw, hh, AO_FORMAT, 1, attach_bind))
            .transient("ao", desc(w, h, AO_FORMAT, 1, attach_bind));
    } else if has(FramePass::Ao) {
        g.transient("ao.raw", desc(w, h, AO_FORMAT, 1, attach_bind))
            .transient("ao", desc(w, h, AO_FORMAT, 1, attach_bind));
    }
    if has(FramePass::Light2d) {
        let (lw, lh) = cfg.lightmap;
        g.transient("lightmap", desc(lw, lh, HDR_FORMAT, 1, attach_bind));
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
    let chain = chain_passes(passes);
    for i in 0..chain.len() {
        g.transient(ldr_name(i), desc(w, h, LDR_FORMAT, 1, attach_bind));
    }
    if has(FramePass::DepthBands) {
        g.transient("hdr.bands", desc(w, h, HDR_FORMAT, 1, attach_bind));
    }
    if has(FramePass::Ssr) {
        g.transient("hdr.ssr", desc(w, h, HDR_FORMAT, 1, attach_bind));
    }
    for (pass, name) in [
        (FramePass::Outline, "hdr.outline"),
        (FramePass::TaaSharpen, "hdr.sharp"),
        (FramePass::DofComposite, "hdr.dof"),
        (FramePass::MotionBlur, "hdr.mb"),
        (FramePass::ScreenBlur, "hdr.blur"),
    ] {
        if has(pass) {
            g.transient(name, desc(w, h, HDR_FORMAT, 1, attach_bind));
        }
    }
    if has(FramePass::DofCoc) {
        g.transient("dof.coc", desc(hw, hh, HDR_FORMAT, 1, attach_bind))
            .transient("dof.blur", desc(hw, hh, HDR_FORMAT, 1, attach_bind));
    }
    let customs = passes
        .iter()
        .filter(|p| matches!(p, FramePass::CustomPost(_)))
        .count();
    for &name in &CUSTOM_POST_NAMES[..customs] {
        g.transient(name, desc(w, h, HDR_FORMAT, 1, attach_bind));
    }
    if has(FramePass::GodRaysMask) {
        g.transient("god.mask", desc(hw, hh, HDR_FORMAT, 1, attach_bind))
            .transient("god.blur", desc(hw, hh, HDR_FORMAT, 1, attach_bind));
    }
    let mut size = METER_SIZE;
    for i in 0..meter_levels() {
        g.transient(
            luminance_name(i),
            desc(size, size, LUMINANCE_FORMAT, 1, attach_bind),
        );
        size /= 8;
    }
    for i in 0..bloom_levels(w, h) {
        g.transient(
            bloom_name(i),
            desc(
                (hw >> i).max(1),
                (hh >> i).max(1),
                HDR_FORMAT,
                1,
                attach_bind,
            ),
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
    let ao_reads: &[&str] = if ao_depth {
        &["ao-depth", "gbuffer"]
    } else {
        &["lindepth", "gbuffer"]
    };
    for &p in passes {
        let name = p.name();
        match p {
            FramePass::Cascade(_) => g.pass(name, &[], &["cascades"]),
            FramePass::LocalFace(_) => g.pass(name, &[], &["local-shadows"]),
            FramePass::Prepass => {
                let mut writes = vec!["pre-depth", "gbuffer", "lindepth", "outline-id"];
                if cfg.object_space {
                    writes.push("object-space");
                }
                if ao_depth {
                    writes.push("ao-depth");
                }
                g.pass(name, &[], &writes)
            }
            FramePass::Ao => g.pass(name, ao_reads, &["ao.raw"]),
            FramePass::AoBlur if has(FramePass::AoUpsample) => {
                g.pass(name, &["ao.raw", "lindepth", "gbuffer"], &["ao.blur"])
            }
            FramePass::AoBlur => g.pass(name, &["ao.raw", "lindepth", "gbuffer"], &["ao"]),
            FramePass::AoUpsample => g.pass(name, &["ao.blur", "lindepth"], &["ao"]),
            FramePass::Particles if prepass => {
                g.pass(name, &["lindepth", "gbuffer"], &["particles"])
            }
            FramePass::Particles => g.pass(name, &[], &["particles"]),
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
                if has(FramePass::AoBlur) {
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
                let mut writes = vec!["hdr", "depth"];
                if msaa {
                    writes.push("hdr-msaa");
                }
                if cfg.water {
                    writes.push("scene-copy");
                }
                g.pass(name, &reads, &writes)
            }
            FramePass::Glow => g.pass(name, &[], &["glow"]),
            FramePass::Ssr => g.pass(name, &["hdr", "lindepth", "gbuffer"], &["hdr.ssr"]),
            FramePass::DepthBands => g.pass(name, &[names.base, "lindepth"], &["hdr.bands"]),
            FramePass::Outline => g.pass(
                name,
                &[names.lit, "gbuffer", "lindepth", "outline-id"],
                &["hdr.outline"],
            ),
            FramePass::Taa => g.pass(
                name,
                &[names.main, "depth", "history-read"],
                &["history-write"],
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
            FramePass::Paint
            | FramePass::Lines
            | FramePass::Lens
            | FramePass::Quantize
            | FramePass::Fxaa
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
                if matches!(p, FramePass::Paint | FramePass::Lines) && prepass {
                    reads.extend(["lindepth", "gbuffer"]);
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
