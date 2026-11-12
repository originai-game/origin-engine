//! Water (the E5 round). [`Water`]: a planar 3D surface drawn after the
//! opaque scene and the sky (see `shaders/water.wgsl`): a grid on the
//! surface's local xz plane, displaced by an optional height field (the
//! wave simulation of og-physics' `og.WaterSurface`), animated detail
//! normals, colour by the depth of water below (from the depth prepass,
//! which a frame with water always runs), foam where it meets geometry and
//! on steep crests, refraction of a copy of the scene drawn so far, and
//! reflection: a march through the depth prepass, the sky's prefiltered
//! cubemap where that misses.
//!
//! Phase 2 adds, each off by default: a **planar reflection** (the scene
//! drawn again from the camera mirrored in the surface, clipped at the
//! surface by an oblique near plane, at a fraction of the output's
//! resolution: one surface a frame), the directional light's **shadows**
//! on the surface, **caustics** on what lies below it (the lit and terrain
//! shaders read them from the frame), and the **underwater view**: with
//! the camera below the surface the frame's fog becomes the water's
//! (`underwater_color`, `underwater_density`) and the surface, seen from
//! below, shows what is above inside Snell's window and reflects the
//! water's colour outside it.
//!
//! [`Water2d`]: a side-on water body (og-physics' `og.WaterLine`), drawn
//! through the sprite order as a strip: the surface line's columns over a
//! body that darkens with depth and a bright lip along the top.

use crate::pipeline_features::{Features, Variants};
use glam::{Mat4, Vec2, Vec3, Vec4};
use std::sync::Arc;

mod caustics;
mod renderer;
mod sea;
mod surface;
pub use caustics::*;
pub(crate) use renderer::*;
pub(crate) use sea::*;
pub(crate) use surface::*;

/// A height field over a square of `size` metres centred on the surface's
/// origin: `resolution` x `resolution` heights, row-major from -z to +z
/// and -x to +x (og-physics' `waterHeights`).
#[derive(Clone, Debug, PartialEq)]
pub struct WaterHeights {
    pub resolution: u32,
    pub size: f32,
    pub heights: Arc<[f32]>,
}

/// The bodies in a water in its last ticks (og-physics' `Wakes`, not world
/// state): what cut a height field after each of the last ticks (at most
/// [`WAKE_TICKS`]) and the waves' settings, which stir its ripples; or the
/// og.Buoyancy hulls in a sea, whose wakes it draws (the newest tick's).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WaterWakes {
    /// A new world (made, restored or gone back) has a new epoch: the
    /// ripples start calm again.
    pub epoch: u64,
    /// The waves' speed (m/s), the fraction of a wave kept after a second,
    /// the surface's splash factor and the tick's length (s).
    pub wave_speed: f32,
    pub damping: f32,
    pub splash: f32,
    pub dt: f32,
    /// (tick, the bodies cutting the surface after it), oldest first.
    pub ticks: Vec<(u64, Vec<WaterCut>)>,
}

/// A body cutting a water (og-physics' `WaterCut`): its entity; its
/// footprint over the surface's xz relative to its centre, as a box (`min`,
/// `max`) and as the rectangle it turns with (`centre`, `half` its half
/// extents along its own axes, `axis` its own x, unit); its depth below the
/// rest level (on a sea, below the water at its centre) and its vertical
/// speed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaterCut {
    pub entity: u64,
    pub min: Vec2,
    pub max: Vec2,
    pub centre: Vec2,
    pub half: Vec2,
    pub axis: Vec2,
    pub depth: f32,
    pub speed: f32,
}

/// The most ticks [`WaterWakes`] carries (og-physics keeps as many).
pub const WAKE_TICKS: usize = 8;

/// The height textures' format (the CPU's heights and the ripples' output).
///
/// rgba16float, the height in r: filterable (water binding 1 is a filterable
/// texture_2d_array, shared with the FFT sea's fields) and a storage format
/// the ripples' compute writes; each is viewed as a 1-layer array there.
pub(crate) const HEIGHT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// A planar 3D water surface. See the module documentation.
#[derive(Clone, Debug, PartialEq)]
pub struct Water {
    /// World from the surface's frame: the water lies on local y = 0,
    /// centred on the origin.
    pub transform: Mat4,
    /// The extent along local x and z, in metres.
    pub size: Vec2,
    /// Grid cells per side (the height field's detail; 1 for a flat sheet).
    /// The renderer draws more where a cell would be longer than 256 m.
    pub segments: u32,
    /// Linear colour of shallow water (tints what is seen through it).
    pub shallow: Vec3,
    /// Linear colour of deep water.
    pub deep: Vec3,
    /// The depth (m) over which water turns from shallow to deep.
    pub depth_fade: f32,
    /// W2: what the water absorbs and scatters of each colour per metre
    /// (1/m): with either set, the body is physical (Beer–Lambert over the
    /// light's path, single scattering of the sun and sky) and `shallow`,
    /// `deep`, `depth_fade` and `opacity` are unused. Zero: those.
    pub absorption: Vec3,
    pub scattering: Vec3,
    /// How opaque deep water gets (0: clear, 1: the deep colour).
    pub opacity: f32,
    /// Linear foam colour.
    pub foam: Vec3,
    /// The width (m of water depth) of the foam band where the water meets
    /// geometry (0: no foam).
    pub foam_width: f32,
    /// Detail waves per metre.
    pub wave_scale: f32,
    /// How fast the detail waves travel.
    pub wave_speed: f32,
    /// How strongly they bend the normal (0: a mirror).
    pub wave_strength: f32,
    /// How far refraction shifts what is seen through the water.
    pub refraction: f32,
    /// How much it reflects (scales the Fresnel term).
    pub reflectivity: f32,
    /// The wave simulation's heights (`None`: flat).
    pub heights: Option<WaterHeights>,
    /// Reflect the scene by drawing it again mirrored in the surface, at
    /// this fraction of the output's resolution (0: off; the screen-space
    /// and cubemap reflections). One surface a frame: the first that asks.
    pub planar_reflection: f32,
    /// How much the directional light's shadows darken the surface's
    /// highlight, foam and colour (0: none, 1: fully).
    pub shadows: f32,
    /// The reflectance seen straight down (Schlick's F0): 0.02 for clean
    /// water; three.js Water.js uses 0.3.
    pub f0: f32,
    /// A floor under the reflected colour, `bias + (1 - bias) · reflected`
    /// (three.js Water.js: 0.1).
    pub reflection_bias: f32,
    /// With the camera below the surface: the water's fog colour and its
    /// density per metre (0: no underwater view).
    pub underwater_color: Vec3,
    pub underwater_density: f32,
    /// Caustics on what lies below the surface (0: none) and their size
    /// (cells per metre). One surface a frame: the first that has them.
    pub caustics: f32,
    pub caustics_scale: f32,
    /// Draw the water over this mesh instead of the `size` grid: its
    /// vertices in the surface's frame (`transform`, scale included), its
    /// normals the surface's. Streams, channels and pools of any outline get
    /// the same depth colour, refraction, foam and reflections. No height
    /// field; the planar reflection still mirrors in the frame's xz plane.
    pub mesh: Option<crate::scene::MeshHandle>,
    /// Ripples from a tangent-space normal map (three.js' water normals:
    /// red +x, green +z, blue up) instead of the travelling waves.
    pub normal_map: Option<crate::TextureHandle>,
    /// The map's repeats per metre (lake) or per uv unit (flow).
    pub normal_scale: f32,
    /// Zero: three.js Water's four drifting layers over world xz. Else
    /// three.js Water2's two layers sliding along it in uv (length: uv per
    /// second), cross-faded, with Water2's look: refraction shifts by
    /// N.xz × 0.05 × `refraction` of the screen, the reflection is tinted by
    /// `shallow` as well, and there is no sun highlight.
    pub flow: Vec2,
    /// three.js Water's sun glint on the mirrored image (spec × sunColor
    /// over the light's intensity); 0: the half-vector highlight. With it
    /// the body is Water.js's too: `deep` is its waterColor, scattered by
    /// N·V, plus its sun diffuse (sunColor = light × glint / 2), shadowed.
    pub glint: f32,
    /// R5: three.js Water2's flow map (rg: the flow's direction per point,
    /// `* 2 - 1`; linear), over the mesh's uv or the grid from its -x -z
    /// corner (u along x, v along z, the image's top at +z). With it the
    /// flow's length is the speed (three.js' flowSpeed; zero: its 0.03) and
    /// the map the direction, for the normal map's two layers and, without
    /// a normal map, the travelling waves (in metres a second).
    pub flow_map: Option<crate::TextureHandle>,
    /// R5: the wind over an open sea, its direction in the world's xz and
    /// its speed (m/s) as its length: a clipmap about the camera displaced
    /// by `og_core::ocean`'s longest Gerstner waves (the tick's) and the FFT
    /// sea's cascades ([`crate::ocean_fft`]) over the absolute world's xz
    /// (a level surface; a mesh surface has none). Zero: no sea.
    pub wind: Vec2,
    /// The sea's height, times a fully developed sea's for the wind (1).
    pub swell: f32,
    /// How far its crests lean and sharpen, 0 (round swell) to 1 (the most
    /// that never folds), whitecaps foaming where they bunch.
    pub choppiness: f32,
    /// W4: how strongly the sea's ageing foam is drawn into streaks along
    /// the wind (its lace gathered into windrows; 0: none).
    pub foam_streaks: f32,
    /// W4: how much spray its breaking crests throw (0: none).
    pub spray: f32,
    /// W3: how high the waves its og.Buoyancy hulls raise are drawn (their
    /// bow waves, Kelvin wakes and contact foam; 0: none, not simulated).
    pub dynamic_waves: f32,
    /// W6: where the camera's near plane crosses the surface, the width
    /// (px) of the meniscus line drawn along the waterline between the air
    /// above and the water's fog below (0: the line alone, no meniscus;
    /// below 0: no waterline, the whole frame above or under as the eye is).
    pub meniscus: f32,
    /// R5: the sea reaches the horizon all round the camera (its waves stay
    /// where they are in the world), not only its `size` rectangle.
    pub endless: bool,
    /// The bodies in the water in its last ticks: over a height field the
    /// renderer draws finer ripples from them where it has compute
    /// (`water_ripples.rs`); on a sea it calms the FFT's chop under each
    /// og.Buoyancy hull (the tick floats none on it). `None`: neither.
    pub wakes: Option<Arc<WaterWakes>>,
    /// W5: the water is a river along this course (`og_core::river`: the
    /// centreline in the frame, its width and speed): drawn as a ribbon
    /// over it in place of the `size` grid, its flow map the current, which
    /// slows to the banks and turns with the course; where the course drops
    /// steeper than 45 degrees the water falls. The tick drifts floaters
    /// with the same current.
    pub river: Option<Arc<og_core::river::River>>,
    /// W1: the seabed under the sea (og.Water.seabed): the waves shoal and
    /// break over it and the shore trains roll in, as the tick floats bodies
    /// on them (og-physics' `WaterBody::shore`). `None`: the open sea.
    pub shore: Option<WaterShore>,
}

/// A sea's seabed ([`Water::shore`]): the field (`og_core::shore`), the
/// trains rolling in on it, where the field's sample (0, 0) lies in the
/// scene's frame (moved with it by `RenderScene::rebase`; the field's own
/// origin is the tick's frame) and how much surf foams where they break.
#[derive(Clone, Debug, PartialEq)]
pub struct WaterShore {
    pub field: Arc<og_core::ocean::Shore>,
    pub trains: og_core::ocean::ShoreTrains,
    pub origin: Vec2,
    pub foam: f32,
}

impl Default for Water {
    fn default() -> Self {
        Water {
            transform: Mat4::IDENTITY,
            size: Vec2::splat(16.0),
            segments: 64,
            shallow: Vec3::new(0.55, 0.85, 0.82),
            deep: Vec3::new(0.02, 0.09, 0.14),
            depth_fade: 2.5,
            opacity: 0.92,
            foam: Vec3::new(0.95, 0.97, 1.0),
            foam_width: 0.35,
            wave_scale: 0.6,
            wave_speed: 1.0,
            wave_strength: 0.35,
            refraction: 1.0,
            reflectivity: 1.0,
            heights: None,
            planar_reflection: 0.0,
            shadows: 0.0,
            f0: 0.02,
            reflection_bias: 0.0,
            underwater_color: Vec3::new(0.03, 0.16, 0.2),
            underwater_density: 0.09,
            caustics: 0.0,
            caustics_scale: 0.6,
            mesh: None,
            normal_map: None,
            normal_scale: 1.0,
            flow: Vec2::ZERO,
            glint: 0.0,
            flow_map: None,
            wind: Vec2::ZERO,
            swell: 1.0,
            choppiness: 0.6,
            absorption: Vec3::ZERO,
            scattering: Vec3::ZERO,
            foam_streaks: 1.0,
            spray: 1.0,
            dynamic_waves: 1.0,
            meniscus: 2.0,
            endless: false,
            wakes: None,
            river: None,
            shore: None,
        }
    }
}

/// A side-on 2D water body. See the module documentation.
#[derive(Clone, Debug, PartialEq)]
pub struct Water2d {
    /// The surface's left end at rest, in world units (the body hangs
    /// below it by `depth`).
    pub origin: Vec2,
    pub width: f32,
    pub depth: f32,
    /// Offsets of the surface from rest, evenly spaced across `width`
    /// (at least two).
    pub surface: Vec<f32>,
    /// Linear rgba at the top and at the bottom of the body.
    pub top: Vec4,
    pub bottom: Vec4,
    /// Linear rgba of the lip along the surface and its thickness.
    pub lip: Vec4,
    pub lip_width: f32,
    /// The sprite layer it is drawn in, with the sprites' ordering.
    pub layer: i32,
    /// World z of the body (with sprites that sort by z).
    pub z: f32,
}

impl Default for Water2d {
    fn default() -> Self {
        Water2d {
            origin: Vec2::ZERO,
            width: 12.0,
            depth: 3.0,
            surface: vec![0.0; 2],
            top: Vec4::new(0.16, 0.5, 0.72, 0.72),
            bottom: Vec4::new(0.02, 0.1, 0.25, 0.92),
            lip: Vec4::new(0.8, 0.95, 1.0, 0.9),
            lip_width: 0.06,
            layer: 0,
            z: 0.0,
        }
    }
}

/// `WaterUniform` in water.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct WaterGpu {
    model: [[f32; 4]; 4],
    size: [f32; 4],
    shallow: [f32; 4],
    deep: [f32; 4],
    foam: [f32; 4],
    waves: [f32; 4],
    look: [f32; 4],
    screen: [f32; 4],
    /// x: shadows, y: 1 drawn with the planar reflection, z: F0, w: the
    /// reflection bias.
    extra: [f32; 4],
    /// rgb: the underwater colour, w: the glint (0: off).
    under: [f32; 4],
    /// The normal map: x repeats, yz the flow (zero: the lake's layers),
    /// w 1 when there is one.
    normals: [f32; 4],
    /// x: PostEffects.aoDirect, how much the frame's screen AO darkens the
    /// reflection and the body (0: not at all, as the opaque surfaces'
    /// direct light); y: 1 with a flow map; z: 1 drawn over a mesh; w:
    /// one over [`crate::RenderScene::adaptation`], which the reflection of
    /// the sky divides out (it mirrors the sky as drawn).
    ao: [f32; 4],
    /// The lake's four normal-map layers' uv at the scene's origin, whole
    /// repeats dropped (`origin.rs`): their pattern stays put when it moves.
    layers: [[f32; 4]; 2],
    /// The sea ([`Water::wind`]): x 1 with one, y the grid's centre cell (m;
    /// a wave under four cells is left to the fragments), z and w the
    /// crests' bunching where whitecaps start and over which they grow
    /// ([`whitecaps`]); then per
    /// wave (k d.x, k d.z, amplitude, phase at the origin) and (lean d.x,
    /// lean d.z, k lean, unused) ([`ocean_gpu`]).
    ocean: [f32; 4],
    /// The sea's wind: xy its direction in the frame's xz, z its speed
    /// (m/s), w how strongly its foam streaks along it (`foam_streaks`).
    sea_wind: [f32; 4],
    /// The sea's clipmap ([`clipmap`]): x its levels, y cells along a
    /// level's edge, z the horizon ring's radius (m), w 1 when endless.
    clip: [f32; 4],
    /// Per level: its snapped centre (frame xz) and its cell (m).
    levels: [[f32; 4]; CLIP_LEVELS],
    /// The surface's frame from the world: a bounded sea is clamped to its
    /// rectangle in it.
    inverse: [[f32; 4]; 4],
    /// The FFT sea ([`crate::ocean_fft`]): x its cascades, z the slope
    /// variance finer than the finest cascade, w the sea's significant
    /// height (m; its crests' subsurface light).
    fft: [f32; 4],
    /// Per cascade: one over its tile (1/m), its uv at the frame's origin
    /// (whole tiles dropped), its tile (m).
    cascades: [[f32; 4]; 3],
    /// Each cascade's band's middle wavenumber (xyz: what the seabed's
    /// shoaling keeps of it).
    bands: [f32; 4],
    /// The persistent foam's levels ([`crate::ocean_fft::foam::levels`]).
    foam_levels: [[f32; 4]; 2],
    /// The foam micro-structure's uv at the frame's origin ([`foam_uv`]).
    foam_uv: [[f32; 4]; 2],
    swell: [[f32; 4]; 2 * og_core::ocean::OCEAN_WAVES],
    /// The seabed ([`Water::shore`], [`shore_gpu`]): its sample (0, 0) in
    /// the frame's xz, its cell, 1 with one; its columns and rows, the
    /// trains' wavenumber and amplitude; their phase now, the surf's foam.
    shore: [[f32; 4]; 3],
    /// W4: the sea's spray ([`spray_gpu`]).
    spray: [f32; 4],
    /// W2: the water's absorption and scattering per metre (rgb; w 1 when
    /// either is set: the physical body).
    optics: [[f32; 4]; 2],
    /// W6: x 1 when the near plane crosses this surface ([`waterline`]),
    /// y the underwater fog's density, z the meniscus's width (px); w 1 for
    /// a river (W5: its fronts turn with its flow, and it falls).
    waterline: [f32; 4],
    /// W3: the hulls' wakes ([`crate::sea_wakes`]): xy the square's centre
    /// (frame xz), z one over its side (1/m), w 1 with them.
    wake: [f32; 4],
}

/// The spray's droplet slots per population (`SPRAY_SLOTS` in
/// water.wgsl): the fine droplets, then the spume.
const SPRAY_SLOTS: u32 = 4096;
/// The water's clocks fold over this many seconds (f64 here, so a long
/// game keeps its precision in the shader's f32): a whole number of every
/// period they drive (the bank foam's 8 s), so the fold does not show.
const SPRAY_FOLD: f64 = 4096.0;

/// [`WaterGpu::spray`] of `water` at the tick's `seconds`: how much (0
/// without a sea), its clock, the slots of each population drawn (half on
/// the low tier); w the water's clock folded the same way, for every
/// surface (its bank foam's two phases: f32 seconds lose their precision
/// in a long session).
fn spray_gpu(water: &Water, sea: bool, seconds: f64, quality: &crate::RenderQuality) -> [f32; 4] {
    let clock = seconds.rem_euclid(SPRAY_FOLD) as f32;
    if !sea || water.spray <= 0.0 {
        return [0.0, 0.0, 0.0, clock];
    }
    let slots = if sea_tier(quality).1 < 3 {
        SPRAY_SLOTS / 2
    } else {
        SPRAY_SLOTS
    };
    [water.spray, clock, slots as f32, clock]
}

/// The Jacobians under which the drawn sea of `water` lays down foam that
/// persists, and under which a crest is white as it folds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FoamLevels {
    /// Per level of the persistent foam ([`crate::ocean_fft::foam`]): the
    /// Jacobian as that level resolves it, under which it lays foam down.
    pub laid: [f32; 2],
    /// The drawn crest's own, under which it is white as it folds.
    pub fresh: f32,
}

/// [`FoamLevels`] of `water`: the levels a share of the surface falls
/// under, that share whitecap coverage as Monahan and O'Muircheartaigh
/// (1980) measured it at the wind's speed U (W = 3.84·10⁻⁶ U^3.41: 0.7 %
/// at 9 m/s, 10 % at 20 m/s), half of it for the foam laid down and a
/// quarter for the crests breaking now (the persistent foam trails behind
/// them). At a breeze these are near web-ocean's fixed ones (0.68 and 0.48
/// at its foam setting 1 against 0.80 and 0.62 here); a gale's steeper sea
/// reaches those over much of its surface, where web-ocean whitened a third
/// of it.
///
/// Each foam level reads the Jacobian at its own texel (ocean_foam.wgsl
/// `jacobian`: the sea's waves faded at it, the cascades from their mips),
/// much smoother than the drawn crest's at its 2 m: one threshold for all
/// let the coarse level, everything past some 20 m, lay down a small share
/// of the coverage (a gale with two or three whitecaps in view). So each
/// level's is the quantile of the Jacobian as that level resolves it.
///
/// The Jacobian's distribution is sampled over the open sea: og_core's
/// waves, plus the cascades' displacement derivatives, a Gaussian of the covariance their
/// spectrum gives. Seeded, so the same sea always has the same levels.
/// A sea that never bunches enough (a round swell: no horizontal
/// displacement) never foams.
fn foam_thresholds(water: &Water, built: &crate::ocean_fft::spectrum::Spectrum) -> FoamLevels {
    let coverage = 3.84e-6 * f64::from(water.wind.length()).powf(3.41);
    let level = |resolve: f32, share: f64| {
        let jac = jacobians(water, built, resolve);
        let i = ((share.min(1.0) * jac.len() as f64) as usize).min(jac.len() - 1);
        // Never above 0.9: a sea whose surface hardly bunches lays none.
        jac[i].min(0.9) as f32
    };
    let ts = |l: usize| {
        (crate::ocean_fft::foam::LEVEL_SIZE[l] / f64::from(crate::ocean_fft::foam::RES)) as f32
    };
    FoamLevels {
        laid: [level(ts(0), coverage * 0.5), level(ts(1), coverage * 0.5)],
        fresh: level(0.0, coverage * 0.25),
    }
}

struct Slot {
    uniform: wgpu::Buffer,
    /// The surface as a reflection probe's faces draw it (`face_draws`),
    /// but for the face's size.
    face: WaterGpu,
    heights: Option<(u32, wgpu::Texture, wgpu::TextureView)>,
    /// The ripples drawn over the heights (`water_ripples.rs`).
    ripples: Option<crate::water_ripples::Ripples>,
}

/// Draws [`Water`]: one pipeline, a uniform buffer and height texture per
/// surface, kept across frames.
/// A water surface's lit pipeline (the grid's or the mesh's).
pub(crate) type Lit = Variants<crate::deferred::Sampled<wgpu::RenderPipeline>>;

pub(crate) struct WaterRenderer {
    layout: wgpu::BindGroupLayout,
    pub(crate) pipeline: Lit,
    /// For surfaces drawn over a mesh ([`Water::mesh`]).
    pub(crate) mesh_pipeline: Lit,
    /// A sea's spray (W4: `vs_spray`, `fs_spray`), premultiplied over the
    /// water, tested against its depth and writing none.
    spray_pipeline: crate::deferred::Sampled<wgpu::RenderPipeline>,
    /// W6: the waterline (`vs_waterline`, `fs_waterline`), a full-screen
    /// triangle premultiplied over the frame, untested.
    waterline_pipeline: crate::deferred::Sampled<wgpu::RenderPipeline>,
    /// The surfaces' view depth for the screen AO (`fs_ao_depth`; grid, mesh): into an R32 copy of the prepass
    /// depth, tested against the prepass, writing no depth.
    ao_depth_pipelines: crate::deferred::Deferred<[wgpu::RenderPipeline; 2]>,
    ripple_sampler: wgpu::Sampler,
    sampler: wgpu::Sampler,
    flat: wgpu::TextureView,
    /// A seabed of one sample, bound where a surface has none.
    no_shore: wgpu::TextureView,
    /// The seabeds' textures ([`shore_texture`]) of this frame's surfaces,
    /// with the fields they hold (shared by surfaces and the foam's pass).
    shores: Vec<(Arc<og_core::ocean::Shore>, wgpu::TextureView)>,
    slots: Vec<Slot>,
    /// The ripples' compute, made with the first surface that ripples;
    /// `compute` false (an adapter without compute shaders): none, the
    /// CPU's heights alone.
    ripple_pipelines: Option<crate::water_ripples::RipplePipelines>,
    compute: bool,
    /// The FFT sea ([`crate::ocean_fft`]) of the frame's first surface with
    /// a wind. Every sea of the frame is drawn from it (one wind a scene).
    fft: Option<(crate::ocean_fft::FftOcean, FftSea)>,
    /// The shader, for the height probe ([`WaterRenderer::probe_sea`]), and
    /// the last frame's first sea's bind group it reads (native only).
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    module: crate::deferred::Module,
    last_sea: Option<wgpu::BindGroup>,
    /// W2: the sea's caustics pass, made with the first sea with caustics
    /// (none in a renderer made without them, RenderDesc::without: no lit
    /// module reads its grid).
    caustics: Option<CausticsPass>,
    caustics_left_out: bool,
    /// W3: the sea's hulls' wakes, made with the first sea with hulls in
    /// it, and the square's centre (frame xz) this frame when they ran.
    wakes: Option<crate::sea_wakes::SeaWakes>,
    wake_centre: Option<Vec2>,
    /// What a surface without wakes binds there (one calm texel).
    no_wake: wgpu::TextureView,
    /// W5: the rivers' ribbons of this frame's surfaces.
    rivers: Vec<crate::water_river::RiverMesh>,
    /// What the last [`WaterRenderer::prepare`] submitted ahead of the
    /// frame, and the foam steps it recorded ([`FrameStats::water_submits`]).
    pub(crate) submitted: (u32, u32),
}

/// `CausticParams` in water_caustics.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CausticsGpu {
    grid: [f32; 4],
    sea: [f32; 4],
    cascades: [[f32; 4]; 3],
    swell: [[f32; 4]; 2 * og_core::ocean::OCEAN_WAVES],
}

/// What the FFT sea's spectrum was built for and what the water's uniforms
/// read of it.
#[derive(Clone, Copy)]
struct FftSea {
    /// That sea's wind, swell and choppiness, and the cascades.
    key: (Vec2, f32, f32, u32),
    /// Per cascade its tile (m); 0 past the last.
    tiles: [f64; 3],
    /// The slope variance finer than the finest cascade.
    tail: f32,
    /// The Jacobians under which foam is laid down and a crest is white
    /// ([`foam_thresholds`]).
    foam: FoamLevels,
    /// Each cascade's band's middle wavenumber (rad/m).
    bands: [f32; 4],
}

/// One surface ready to draw.
pub(crate) struct WaterDraw {
    group: wgpu::BindGroup,
    vertices: u32,
    /// The spray's sprites after it (W4; 0: none).
    spray: u32,
    /// W6: the waterline pass after it (the near plane crosses it).
    waterline: bool,
    /// The surface's world bounding sphere (its reflection probe: probe_select.rs).
    bounds: (Vec3, f32),
    /// Its mesh's vertex and index buffers and index count.
    mesh: Option<(wgpu::Buffer, wgpu::Buffer, u32)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// water.wgsl's `ocean_displace` with every wave kept, on the CPU.
    fn drawn(swell: &[[f32; 4]], frame_xz: Vec2) -> Vec3 {
        let mut d = Vec3::ZERO;
        for w in swell.chunks(2) {
            let (a, b) = (w[0], w[1]);
            let th = a[0] * frame_xz.x + a[1] * frame_xz.y + a[3];
            d += Vec3::new(b[0] * th.cos(), a[2] * th.sin(), b[1] * th.cos());
        }
        d
    }

    /// A display frame between ticks draws the sea at the time its blended
    /// transforms are drawn at, between the last two ticks (the code before
    /// drew every such frame at the last tick: the sea stepped at the
    /// tick's rate on a faster display); a frame at the tick draws the tick.
    #[test]
    fn the_sea_runs_at_the_display_rate_between_ticks() {
        let mut scene = crate::RenderScene::new();
        scene.frame.tick = 600;
        scene.frame.tick_rate = 60.0;
        assert_eq!(seconds(&scene), 10.0);
        scene.frame.interpolation = Some(0.25);
        assert!((seconds(&scene) - 599.25 / 60.0).abs() < 1e-12);
        scene.frame.interpolation = Some(1.0);
        assert_eq!(seconds(&scene), 10.0);
        // Two frames of one tick at different alphas draw different seas.
        let water = Water {
            size: Vec2::splat(2000.0),
            wind: Vec2::new(9.0, 3.0),
            ..Default::default()
        };
        let sea = ocean(&water).expect("a sea");
        let at = glam::DVec2::new(13.0, -7.0);
        let mut height = |a: f32| {
            scene.frame.interpolation = Some(a);
            sea.height(at, seconds(&scene))
        };
        assert!((height(0.2) - height(0.7)).abs() > 1e-3);
        // Monotonic across a tick: the frame after the tick's last blends on.
        let late = 599.95 / 60.0;
        let mut next = crate::RenderScene::new();
        next.frame.tick = 601;
        next.frame.tick_rate = 60.0;
        next.frame.interpolation = Some(0.05);
        assert!(seconds(&next) > late);
    }

    /// The vertices draw the sea the tick floats bodies on: water.wgsl's
    /// `ocean_displace` (every wave of og_core, each left out only where
    /// the clipmap's vertex spacing cannot hold it, the rest point found as
    /// the tick finds it) is [`og_core::ocean::Ocean::height`] to a
    /// centimetre plus the amplitudes of the waves that spacing leaves out:
    /// nothing in a gale to past 10 m, and at any distance, were they all
    /// to crest at one point, under a hundredth of it (half a degree from
    /// the eye; their RMS a fifth of that, a pixel of a 720p frame). The waves the cascades add start
    /// past the sea's shortest wave, so none is drawn twice, and are under
    /// 2 % of its height in RMS. With a grid of the tick's own under both
    /// (r5-fft), the cascades redrew what the grid faded and the two were
    /// 0.2 m RMS apart in a gale at the middle of the surface and 0.8 m at
    /// its edge.
    #[test]
    fn the_vertices_draw_the_sea_the_tick_floats_bodies_on() {
        let eye = Vec3::new(0.0, 10.0, 0.0);
        let (segments, cascades) = sea_tier(&crate::RenderQuality::default());
        let levels = clipmap(eye, Vec3::ZERO, segments);
        // water.wgsl sea_vertex's spacing d metres from the clipmap's centre: the first level that holds it, its cell
        // growing to twice at its edge (the geomorph).
        let spacing = |d: f32| {
            let (cell, half) = levels
                .iter()
                .map(|l| (l[2], l[2] * segments as f32 * 0.5))
                .find(|(_, half)| *half >= d)
                .expect("inside the clipmap");
            let m = ((d / half - 0.7) / 0.25).clamp(0.0, 1.0);
            cell * (1.0 + m * m * (3.0 - 2.0 * m))
        };
        for (wind, choppiness) in [
            (Vec2::new(-3.0, -4.0), 0.6),
            (Vec2::new(-3.0, -9.5), 0.6),
            (Vec2::new(7.0, -18.7), 1.0),
        ] {
            let water = Water {
                size: Vec2::splat(6000.0),
                wind,
                choppiness,
                ..Default::default()
            };
            let sea = ocean(&water).expect("a sea");
            let hs = sea.significant_height();
            // The amplitudes of the waves a spacing leaves out.
            let left = |cell: f32| -> f32 {
                sea.waves
                    .iter()
                    .map(|w| w.amplitude * (1.0 - sea_keep(w.k, cell)))
                    .sum()
            };
            let mut whole = 0;
            for seconds in [0.0, 3.0, 4.5, 61.25] {
                let (_, swell) = ocean_gpu(&water, seconds, Vec3::ZERO);
                let drawn = |p: Vec2| {
                    let cell = spacing(p.distance(Vec2::new(eye.x, eye.z)));
                    let mut d = Vec3::ZERO;
                    for w in swell.chunks(2) {
                        let (a, b) = (w[0], w[1]);
                        let k = Vec2::new(a[0], a[1]);
                        let th = k.dot(p) + a[3];
                        let f = sea_keep(k.length(), cell);
                        d += Vec3::new(b[0] * th.cos(), a[2] * th.sin(), b[1] * th.cos()) * f;
                    }
                    d
                };
                for i in 0..48 {
                    let r = 1.5 * 1.1f32.powi(i);
                    let at = Vec2::from_angle(i as f32 * 2.4) * r;
                    let mut rest = at;
                    for _ in 0..8 {
                        let d = drawn(rest);
                        rest = at - Vec2::new(d.x, d.z);
                    }
                    let got = drawn(rest).y;
                    let tick = sea.height(at.as_dvec2(), seconds);
                    // The rest point lies within the crests' lean of `at`.
                    let out = left(spacing(r + 0.5 * hs));
                    whole += usize::from(out == 0.0);
                    assert!(
                        (got - tick).abs() < 0.01 + out,
                        "{wind} at {seconds} s, {at}: drawn {got} m, the tick's {tick} m ({out} m left out)"
                    );
                    assert!(
                        out < 0.01 * r.max(2.0),
                        "{wind}: {r} m from the eye the vertices leave {out} m of a {hs} m sea out"
                    );
                }
            }
            println!("{wind}: Hs {hs} m, every wave drawn at {whole} of 192 points");
            assert!(wind.length() < 15.0 || whole >= 100, "{wind}: {whole}");
            let built = sea_spectrum(&water, &sea, cascades);
            let shortest = og_core::ocean::shortest_wavenumber(wind.length());
            let mut variance = 0.0;
            for c in &built.cascades {
                assert!(
                    c.band.lo >= f64::from(shortest) * 0.999,
                    "{wind}: a cascade from {} rad/m over the sea's waves to {shortest}",
                    c.band.lo
                );
                variance += c.variance;
            }
            println!("{wind}: the cascades {} m RMS", variance.sqrt());
            assert!(
                variance.sqrt() < 0.02 * f64::from(hs),
                "{wind}: the cascades draw {} m RMS over a {hs} m sea",
                variance.sqrt()
            );
        }
    }

    /// The foam's levels follow the wind's whitecap coverage: a breeze's
    /// crest whitens where the surface has bunched to some six tenths of
    /// its area without folding, before the foam it lays down does, the
    /// gale's steeper sea foams only where it bunches much further, and a
    /// round swell of the same gale (no horizontal displacement, a Jacobian
    /// of 1 everywhere) never.
    #[test]
    fn foam_follows_the_winds_whitecap_coverage() {
        let thresholds = |wind: Vec2, choppiness: f32| {
            let water = Water {
                size: Vec2::splat(6000.0),
                segments: 256,
                wind,
                choppiness,
                ..Default::default()
            };
            let sea = ocean(&water).expect("a sea");
            foam_thresholds(&water, &sea_spectrum(&water, &sea, 3))
        };
        let breeze = thresholds(Vec2::new(-3.0, -8.5), 0.6);
        let gale = thresholds(Vec2::new(7.0, -18.7), 1.0);
        println!("breeze {breeze:?}, gale {gale:?}");
        assert!(breeze.fresh > 0.4 && breeze.fresh < 0.75, "{breeze:?}");
        assert!(breeze.fresh < breeze.laid[0]);
        assert!(
            gale.laid[0] < breeze.laid[0] - 0.15 && gale.fresh < breeze.fresh - 0.15,
            "{gale:?}"
        );
        assert!(gale.fresh > 0.0, "{gale:?}");
        let round = FoamLevels {
            laid: [0.9; 2],
            fresh: 0.9,
        };
        assert_eq!(thresholds(Vec2::new(7.0, -18.7), 0.0), round);
    }

    /// Each persistent foam level lays foam on the whitecap coverage's half
    /// of the surface as it resolves the Jacobian (ocean_foam.wgsl reads the
    /// cascades from their mips at its texel): the gale's coarse level, 2 m
    /// texels, is much smoother than its crests, and under the fine level's
    /// threshold it laid only a small part of it (r5 at 88d021868: two or
    /// three whitecaps in a gale's view).
    #[test]
    fn each_foam_level_lays_the_whitecap_coverage_it_resolves() {
        let water = Water {
            size: Vec2::splat(6000.0),
            segments: 256,
            wind: Vec2::new(7.0, -18.7),
            choppiness: 1.0,
            ..Default::default()
        };
        let sea = ocean(&water).expect("a sea");
        let built = sea_spectrum(&water, &sea, 3);
        let levels = foam_thresholds(&water, &built);
        let half_w = 3.84e-6 * f64::from(water.wind.length()).powf(3.41) * 0.5;
        let share = |jac: &[f64], under: f32| {
            jac.iter().filter(|&&j| j < f64::from(under)).count() as f64 / jac.len() as f64
        };
        for (l, &under) in levels.laid.iter().enumerate() {
            let ts = (crate::ocean_fft::foam::LEVEL_SIZE[l]
                / f64::from(crate::ocean_fft::foam::RES)) as f32;
            let jac = jacobians(&water, &built, ts);
            let got = share(&jac, under);
            let fine = share(&jac, levels.laid[0]);
            println!(
                "level {l} ({ts} m): threshold {under}, lays {got:.4} (W/2 {half_w:.4}); under level 0's {fine:.4}"
            );
            assert!(
                (got / half_w - 1.0).abs() < 0.05,
                "level {l}: {got} of the surface, not {half_w}"
            );
        }
        let coarse = jacobians(&water, &built, 2.0);
        assert!(
            levels.laid[1] > levels.laid[0] + 0.05 && share(&coarse, levels.laid[0]) < 0.5 * half_w,
            "{levels:?}"
        );
    }

    /// The sea the shader draws at a frame point is og_core's at the same
    /// place in the absolute world (frame + the scene's origin) and the
    /// same tick, wherever the origin has moved to: what the simulation's
    /// height query floats a boat on.
    #[test]
    fn the_drawn_sea_is_the_simulations_wherever_the_origin_is() {
        let water = Water {
            size: Vec2::splat(800.0),
            wind: Vec2::new(3.0, -8.0),
            choppiness: 0.8,
            ..Default::default()
        };
        let sea = ocean(&water).expect("a sea");
        let seconds = 4321.25;
        for origin in [Vec3::ZERO, Vec3::new(51_200.0, 0.0, -25_600.0)] {
            let (head, swell) = ocean_gpu(&water, seconds, origin);
            assert_eq!(head[0], 1.0);
            for p in [
                Vec2::new(0.0, 0.0),
                Vec2::new(37.5, -120.0),
                Vec2::new(-210.0, 66.0),
            ] {
                let at = glam::DVec2::new(
                    f64::from(origin.x) + f64::from(p.x),
                    f64::from(origin.z) + f64::from(p.y),
                );
                let phases = sea.phases(seconds, at);
                let want = sea.displacement(Vec2::ZERO, &phases);
                let got = drawn(&swell, p);
                assert!(
                    (got - want).length() < 2e-3,
                    "{origin} {p}: {got} vs {want}"
                );
            }
        }
        // A surface over a mesh, or without a wind, has no sea.
        let mut calm = water.clone();
        calm.wind = Vec2::ZERO;
        assert_eq!(ocean_gpu(&calm, seconds, Vec3::ZERO).0, [0.0; 4]);
    }

    /// The shore a sea with a seabed wets (W1): the band its surf runs up,
    /// half the sea's significant height times the trains' strength, over
    /// its seabed's rectangle in the frame, which moves with the frame's
    /// render origin as the field does; none without a seabed.
    /// W5: the clock the water's bank foam drifts by keeps a millisecond
    /// after 10⁶ s (twelve days) of tick time, and runs on smoothly across
    /// its fold: f32 seconds, which the foam read before, step by 1/16 s
    /// there, which drew the drifting patches jumping.
    #[test]
    fn the_waters_clock_keeps_its_precision_in_a_long_session() {
        let water = Water::default();
        let q = crate::RenderQuality::default();
        let at = |s: f64| spray_gpu(&water, false, s, &q)[3];
        let t: f64 = 1.0e6 + 0.123_456;
        let exact = t.rem_euclid(SPRAY_FOLD);
        assert!(
            (f64::from(at(t)) - exact).abs() < 1e-3,
            "{} against {exact}",
            at(t)
        );
        assert!(
            (f64::from(t as f32) - t).abs() > 1e-3,
            "f32 seconds would hold it too"
        );
        // Across the fold the bank foam's 8 s phase runs on.
        let phase = |s: f64| (f64::from(at(s)) / 8.0).fract();
        let before = phase(SPRAY_FOLD * 300.0 - 0.01);
        let after = phase(SPRAY_FOLD * 300.0 + 0.01);
        assert!(
            ((after - before).rem_euclid(1.0) - 0.0025).abs() < 1e-3,
            "{before} to {after}"
        );
    }

    #[test]
    fn a_seabed_wets_the_shore_its_surf_runs_up() {
        let depth: Vec<f32> = (0..16 * 8).map(|i| 5.0 - (i % 16) as f32).collect();
        let field = Arc::new(
            og_core::ocean::Shore::new(Vec2::new(-10.0, 4.0), 2.0, 16, 8, &depth).unwrap(),
        );
        let sea = Water {
            transform: Mat4::from_translation(Vec3::new(0.0, 1.5, 0.0)),
            wind: Vec2::new(10.0, 0.0),
            ..Default::default()
        };
        assert_eq!(
            caustics(std::slice::from_ref(&sea), Vec3::ZERO, Vec3::ZERO)[4],
            [0.0; 4]
        );
        let hs = ocean(&sea).unwrap().significant_height();
        let mut beach = sea;
        beach.shore = Some(WaterShore {
            trains: og_core::ocean::ShoreTrains::new(hs, 1.0, 0.0),
            origin: field.origin + Vec2::new(3.0, -1.0),
            field,
            foam: 1.0,
        });
        let rows = caustics(&[beach], Vec3::ZERO, Vec3::ZERO);
        assert_eq!(rows[4][0], 1.5);
        assert!((rows[4][1] - 0.5 * hs).abs() < 1e-4, "{:?}", rows[4]);
        assert!(rows[4][2] > 0.0);
        assert_eq!(rows[5], [-7.0, 3.0, 23.0, 17.0]);
    }
}
