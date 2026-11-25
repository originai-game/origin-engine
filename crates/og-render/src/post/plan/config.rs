//! What a frame's targets depend on besides its passes.

/// What the transient targets depend on besides the passes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GraphConfig {
    pub width: u32,
    pub height: u32,
    /// The size of the frame after TAA ([`super::FramePass::after_taa`]): the
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
    /// The screen AO sees the water's surfaces (`front-depth`; water.rs
    /// `draw_ao_depth`): water drawn with `PostEffects::ao.direct` above 0,
    /// the three.js-era AO that darkened the water's pixel whole. At 0 the
    /// water, drawn after the AO is applied, is outside it.
    pub water_ao: bool,
    /// The volumetric fog ends each pixel's ray at the water's surface
    /// (`front-depth`): water drawn under it. Blended in the main pass,
    /// the water is not in the prepass depth, so the fog ran on to the
    /// sky or the bed behind it, through and under the water.
    pub water_fog: bool,
    /// The main pass copies the opaque scene for transmissive materials
    /// (into the renderer's own mip-chained target, not a graph target).
    pub transmission: bool,
    /// Iterated zoom (other screen blurs stay one pass).
    pub zoom_blur: bool,
    /// Motion's tile width in output pixels: ceil(maxPixels / 2), bounded
    /// by the output's longest axis. NeighborMax spans the adjacent tiles.
    pub motion_tile: u32,
    /// Metaball field targets the frame's fluids need (1 to
    /// `metaballs::FIELDS` when it draws any).
    pub fluid_fields: u32,
    /// The planar reflections' sizes (`Some` with [`super::FramePass::Reflection`] of that slot).
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
    /// [`crate::Caps::RG32_STORAGE`]: the main pass's scatter cards read
    /// `lindepth` as storage (impostor_kept.wgsl), so it is made with
    /// that usage, which a device without it refuses for its format.
    pub lindepth_storage: bool,
    /// Large soft-depth billboard particles run in a half-resolution HDR
    /// target (`fx.half` + `fx.half_depth`) and composite bilaterally onto `hdr`.
    pub fx_half: bool,
    /// FSR 3.1 compute RCAS writes `hdr.sharp` via a storage binding.
    pub fsr: bool,
}
