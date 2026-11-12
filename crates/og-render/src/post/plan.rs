//! Frame pass names and render graph construction.

use super::*;

/// One pass of a frame, as the render graph knows it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum FramePass {
    /// The compute before the frame's draws, which the passes after it
    /// read (renderer/render/pre_graph.rs): the crowds' cull on the GPU,
    /// the GPU culling's counts copied for their readback, the wind
    /// field's and the trample map's steps, the visual bodies' and fluids'
    /// steps, the deforming surfaces' and garments' vertices, the terrain
    /// levels' corners, the grass blades, and the scattered copies placed
    /// and culled. Into their own buffers, not graph targets.
    CrowdCull,
    CullCounts,
    Wind,
    Visual,
    Surfaces,
    TerrainCorners,
    Grass,
    Scatter,
    /// Directional light cascade `i`.
    Cascade(u32),
    /// Local light shadow layer (6 per point light, 1 per spot).
    LocalFace(u32),
    /// The weather occlusion map, the cascade array's last layer (weather.rs
    /// `shelter_view`): in the frames it is drawn again, not every frame.
    Shelter,
    /// The depth-normal prepass (normal, velocity, view depth, outline ids).
    Prepass,
    /// Ambient occlusion (GTAO or SSAO), then its blur.
    Ao,
    AoBlur,
    /// Half-resolution ambient occlusion's depth-aware upsample
    /// ([`crate::AoResolution::Half`]): with it, `ao` and its blur run at
    /// half resolution and this pass writes the full-resolution `ao`.
    AoUpsample,
    /// Screen-space contact shadows for the shadowed local lights
    /// (contact.wgsl, compute): writes the lights' masks (its own buffer,
    /// contact.rs `mask_buffer`), then `ao`, the AO (from `ao.pre`, the AO
    /// passes' target with it) in r and the masks in gba.
    Contact,
    /// The particle simulation (compute).
    Particles,
    /// The 2D light map.
    Light2d,
    /// The 2D fluids' metaball field.
    Metaballs,
    /// The 3D fluids' depth, thickness and smoothing (fluid3d.rs).
    Fluid3d,
    /// The live sky's step this frame (`sky_capture.rs`): a slice of its
    /// capture of the per-pixel sky and its filtering.
    LiveSky,
    /// The main camera's clouds, marched at half the frame's size into
    /// `clouds` (pass_clouds.rs): the main pass's sky draws them from there.
    Clouds,
    /// The probe GI's trace and blend this frame (`gi_probes.rs`): into the
    /// light volumes' buffer, which every lit pass after it reads.
    GiProbes,
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
    /// Screen-space reflections (`ssr.rs`): compute passes after the
    /// prepass and the AO into the renderer's own targets (the imported
    /// `ssr`), which the main pass's lit shaders read.
    Ssr,
    /// Glowing sprites into the glow buffer.
    Glow,
    Outline,
    /// The outline jump flood's seeds (fxpost.wgsl `fs_outline_seed`):
    /// with `MaterialStyle::outline_place` outside or both, the
    /// silhouette pixels of outlined objects into `outline-jfa.a`.
    OutlineSeed,
    /// Jump `i` of [`JFA_STEPS`] (`fs_jfa_<step>`), between
    /// `outline-jfa.a` and `outline-jfa.b`.
    OutlineJump(u32),
    /// The lines outside the silhouettes from the flood (`fs_outline_wide`).
    OutlineWide,
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
    /// Lens flare (`post/flare.rs`): each flaring light's visible fraction
    /// from the prepass depth (`flare.vis`).
    FlareOcclusion,
    /// Its ghosts and halo, from `bloom.0` before the upsamples add onto it
    /// and the lights' discs, into `flare.raw` (a quarter of the frame).
    Flare,
    /// Its 9-tap Gaussian across (`flare.x`), then down (`flare`, which the
    /// tonemap adds).
    FlareBlurX,
    FlareBlurY,
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
    /// The watercolour wash over the painted frame (stylise.wgsl
    /// `watercolor`).
    Watercolor,
    Lines,
    Lens,
    Quantize,
    /// FXAA over the tonemapped image (an alternative to TAA and MSAA).
    Fxaa,
    SmaaEdges,
    SmaaWeights,
    SmaaBlend,
    SmaaResolve,
    Upscale,
    /// The passes over the finished frame (renderer/render/after_graph.rs):
    /// the viewport views, the UI layers, the overlay with the glass and
    /// the lines drawn among it, and the HDR output's encode.
    ViewportViews,
    UiLayers,
    Overlay,
    HdrOutput,
}

/// The outline jump flood's steps in pixels, one pass each (fxpost.wgsl
/// `fs_jfa_<step>`): from half the widest line (31.875 px) down, with a
/// second step of 1 (JFA+1, Rong and Tan 2006) for the last ties.
pub(crate) const JFA_STEPS: [u32; 6] = [16, 8, 4, 2, 1, 1];

/// The jump flood target jump `i` reads and the one it writes: from the
/// seeds' `outline-jfa.a`, back and forth, the last (an even count) back
/// in `.a`, which `OutlineWide` reads.
pub(crate) fn jfa_targets(i: u32) -> (&'static str, &'static str) {
    if i.is_multiple_of(2) {
        ("outline-jfa.a", "outline-jfa.b")
    } else {
        ("outline-jfa.b", "outline-jfa.a")
    }
}

/// The passes after the tonemap that `passes` holds, in the order they run.
pub(crate) fn chain_passes(passes: &[FramePass]) -> Vec<FramePass> {
    let temporal = passes.contains(&FramePass::SmaaResolve);
    [
        FramePass::SmaaResolve,
        FramePass::Paint,
        FramePass::Watercolor,
        FramePass::Lines,
        FramePass::Lens,
        FramePass::Quantize,
        FramePass::Fxaa,
        FramePass::SmaaBlend,
        FramePass::Upscale,
    ]
    .into_iter()
    .filter(|p| passes.contains(p) && !(temporal && *p == FramePass::SmaaBlend))
    .collect()
}

/// The lit image the tonemap reads (under TAA with nothing after it, the
/// image TAA reads, whose history moves between frame slots): the scene in linear light before
/// exposure and the tone curve, which the quantize and lines passes read
/// beside their source to tell conformed art (`Scene::add_conformed_colors`) by the
/// colour it was drawn in, whatever curve the look draws it through.
pub(crate) fn untoned(passes: &[FramePass]) -> &'static str {
    let names = lit_names(passes);
    names.last.unwrap_or(names.main)
}

/// The LDR target chain pass `i` reads (the tonemap writes `ldr.0`).
pub(super) fn ldr_name(i: usize) -> String {
    format!("ldr.{i}")
}

impl FramePass {
    /// Whether what the pass draws is drawn again next frame, so a draw it
    /// skips costs that one frame: the passes encoded in the web host's
    /// frame span (fork s60), whose pipelines compile asynchronously there.
    /// Not the passes whose output later frames keep: the shadow maps
    /// (their static casters are kept), the weather map, the live sky's
    /// slices, the probe GI, the reflection probes' captures and the
    /// secondary views; nor the compute before the frame's draws, encoded
    /// before its span opens.
    pub fn redrawn(self) -> bool {
        !matches!(
            self,
            FramePass::CrowdCull
                | FramePass::CullCounts
                | FramePass::Wind
                | FramePass::Visual
                | FramePass::Surfaces
                | FramePass::TerrainCorners
                | FramePass::Grass
                | FramePass::Scatter
                | FramePass::Cascade(_)
                | FramePass::LocalFace(_)
                | FramePass::Shelter
                | FramePass::LiveSky
                | FramePass::GiProbes
                | FramePass::Probes
                | FramePass::View(_)
        )
    }

    /// Whether the pass runs after TAA, on the frame TAA resolved: at the
    /// output's size under TAAU ([`GraphConfig::post`]).
    pub fn after_taa(self) -> bool {
        matches!(
            self,
            FramePass::TaaSharpen
                | FramePass::DofCoc
                | FramePass::DofBlur
                | FramePass::DofComposite
                | FramePass::MotionBlur
                | FramePass::ScreenBlur
                | FramePass::CustomPost(_)
                | FramePass::GodRaysMask
                | FramePass::GodRaysBlur
                | FramePass::Meter
                | FramePass::Reduce(_)
                | FramePass::Adapt
                | FramePass::BloomPrefilter
                | FramePass::BloomDown(_)
                | FramePass::BloomUp(_)
                | FramePass::FlareOcclusion
                | FramePass::Flare
                | FramePass::FlareBlurX
                | FramePass::FlareBlurY
                | FramePass::Tonemap
                | FramePass::Paint
                | FramePass::Watercolor
                | FramePass::Lines
                | FramePass::Lens
                | FramePass::Quantize
                | FramePass::Fxaa
                | FramePass::SmaaEdges
                | FramePass::SmaaWeights
                | FramePass::SmaaBlend
                | FramePass::SmaaResolve
                | FramePass::Upscale
        )
    }

    pub fn name(self) -> String {
        match self {
            FramePass::CrowdCull => "cull: crowds".into(),
            FramePass::CullCounts => "cull-counts".into(),
            FramePass::Wind => "wind".into(),
            FramePass::Visual => "visual simulation".into(),
            FramePass::Surfaces => "surfaces".into(),
            FramePass::TerrainCorners => "terrain corners".into(),
            FramePass::Grass => "grass".into(),
            FramePass::Scatter => "scatter".into(),
            FramePass::Cascade(i) => format!("cascade.{i}"),
            FramePass::LocalFace(i) => format!("local-shadow.{i}"),
            FramePass::Shelter => "shelter".into(),
            FramePass::Prepass => "prepass".into(),
            FramePass::Ao => "ao".into(),
            FramePass::AoBlur => "ao-blur".into(),
            FramePass::AoUpsample => "ao-upsample".into(),
            FramePass::Contact => "contact".into(),
            FramePass::Particles => "particles".into(),
            FramePass::Light2d => "lights-2d".into(),
            FramePass::Metaballs => "metaballs".into(),
            FramePass::Fluid3d => "fluid-3d".into(),
            FramePass::LiveSky => "live-sky".into(),
            FramePass::Clouds => "clouds".into(),
            FramePass::GiProbes => "gi-probes".into(),
            FramePass::Probes => "reflection-probes".into(),
            FramePass::Reflection(k) => reflection_target(k, "").into(),
            FramePass::View(i) => format!("view-{i}"),
            FramePass::Volumetric => "volumetric".into(),
            FramePass::Main => "main".into(),
            FramePass::Ssr => "ssr".into(),
            FramePass::Glow => "glow".into(),
            FramePass::Outline => "outline".into(),
            FramePass::OutlineSeed => "outline-seed".into(),
            FramePass::OutlineJump(i) => format!("outline-jump.{i}"),
            FramePass::OutlineWide => "outline-wide".into(),
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
            FramePass::FlareOcclusion => "flare-occlusion".into(),
            FramePass::Flare => "flare".into(),
            FramePass::FlareBlurX => "flare-blur-x".into(),
            FramePass::FlareBlurY => "flare-blur-y".into(),
            FramePass::Tonemap => "tonemap".into(),
            FramePass::DepthBands => "depth-bands".into(),
            FramePass::Paint => "paint".into(),
            FramePass::Watercolor => "watercolor".into(),
            FramePass::Lines => "lines".into(),
            FramePass::Lens => "lens".into(),
            FramePass::Quantize => "quantize".into(),
            FramePass::Fxaa => "fxaa".into(),
            FramePass::SmaaEdges => "smaa-edges".into(),
            FramePass::SmaaWeights => "smaa-weights".into(),
            FramePass::SmaaBlend => "smaa-blend".into(),
            FramePass::SmaaResolve => "smaa-resolve".into(),
            FramePass::Upscale => "upscale".into(),
            FramePass::ViewportViews => "views".into(),
            FramePass::UiLayers => "ui-layers".into(),
            FramePass::Overlay => "overlay".into(),
            FramePass::HdrOutput => "hdr-output".into(),
        }
    }
}

/// What the transient targets depend on besides the passes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GraphConfig {
    pub width: u32,
    pub height: u32,
    /// The size of the frame after TAA ([`FramePass::after_taa`]): the
    /// output's under TAAU (the scene drawn at the render scale and
    /// resolved up to the output by TAA), else the scene's.
    pub post: (u32, u32),
    pub chain_format: wgpu::TextureFormat,
    pub samples: u32,
    pub taa: bool,
    /// The 2D light map's size.
    pub lightmap: (u32, u32),
    /// The 2D light map keeps the lights' directions too (`lightdir`, the
    /// same size), for normal-mapped receivers.
    pub lightdir: bool,
    /// GTAO (or SSAO).
    pub gtao: bool,
    /// Screen-space global illumination (`ssgi.rs`) draws in the AO
    /// passes' place: their targets hold [`crate::ssgi::GI_FORMAT`], the
    /// AO pass reads the previous frame's lit image (TAA's history, or
    /// without TAA `gi-read`, which the main pass's copy into `gi-write`
    /// fills for the next frame).
    pub gi: bool,
    /// Screen-space reflections (`ssr.rs`) trace before the main pass and
    /// read the previous frame's lit image as the GI does (TAA's history,
    /// or `gi-read`); the main pass reads their result (imported `ssr`).
    pub ssr: bool,
    /// GTAO writes its bent normals beside the AO for the probe GI
    /// (fxpost.wgsl `fs_gtao_bent`): the AO targets hold
    /// [`crate::ssgi::GI_FORMAT`], blurred over all four channels.
    pub bent: bool,
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
    /// The main pass draws its blended surfaces order independently
    /// (oit.rs) into `oit.accum` and `oit.reveal`, which TAA reads as its
    /// reactive mask; `Some(true)`: with the linked lists, whose resolve
    /// reads the depth.
    pub oit: Option<bool>,
}

/// The target the AO passes end in: `ao`, or with contact shadows
/// `ao.pre`, which the contact pass reads.
pub(super) fn ao_target_name(contact: bool) -> &'static str {
    if contact { "ao.pre" } else { "ao" }
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
    /// The lit image after the main pass and the depth bands.
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
    let lit = if has(FramePass::DepthBands) {
        "hdr.bands"
    } else {
        "hdr"
    };
    let main = if has(FramePass::OutlineWide) {
        "hdr.wide"
    } else if has(FramePass::Outline) {
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

#[cfg(test)]
mod tests {
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
}
