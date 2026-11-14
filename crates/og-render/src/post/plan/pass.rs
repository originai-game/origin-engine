//! The passes of a frame, as the render graph names them.

use super::*;

/// One pass of a frame, as the render graph knows it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum FramePass {
    /// The compute before the frame's draws, which the passes after it
    /// read (renderer/render/pre_graph.rs): the crowds' cull on the GPU,
    /// the GPU culling's counts copied for their readback, the wind
    /// field's and the trample map's steps, the water's fields (the sea's
    /// spectrum, foam and caustics, the wakes, the ripples), the trails'
    /// histories, the aurora's footprint, the visual bodies' and fluids'
    /// steps, the deforming surfaces' and garments' vertices, the terrain
    /// levels' corners, the grass blades, and the scattered copies placed
    /// and culled. Into their own buffers, not graph targets.
    CrowdCull,
    CullCounts,
    Wind,
    WaterFields,
    Trails,
    Aurora,
    CloudShadow,
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
pub(in crate::post) fn ldr_name(i: usize) -> String {
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
                | FramePass::WaterFields
                | FramePass::Trails
                | FramePass::Aurora
                | FramePass::CloudShadow
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
            FramePass::WaterFields => "water fields".into(),
            FramePass::Trails => "trails".into(),
            FramePass::Aurora => "aurora".into(),
            FramePass::CloudShadow => "cloud shadow".into(),
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
