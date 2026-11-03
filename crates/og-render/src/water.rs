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

use glam::{Mat4, Vec2, Vec3, Vec4};
use std::sync::Arc;

/// A height field over a square of `size` metres centred on the surface's
/// origin: `resolution` x `resolution` heights, row-major from -z to +z
/// and -x to +x (og-physics' `waterHeights`).
#[derive(Clone, Debug, PartialEq)]
pub struct WaterHeights {
    pub resolution: u32,
    pub size: f32,
    pub heights: Arc<[f32]>,
    /// The bodies that cut the surface in its last ticks: the renderer
    /// draws finer ripples over the heights from them where it has compute
    /// (`water_ripples.rs`). `None`: the heights alone.
    pub wakes: Option<Arc<WaterWakes>>,
}

/// The ticks that stir a height field's ripples (og-physics' `Wakes`, not
/// world state): the bodies that cut the surface after each of the last
/// ticks (at most [`WAKE_TICKS`]) and the waves' settings.
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

/// A body cutting a height field's rest level: its entity, its footprint
/// (min and max over the surface's xz, relative to its centre), its depth
/// below the rest level and its vertical speed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaterCut {
    pub entity: u64,
    pub min: Vec2,
    pub max: Vec2,
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
    /// R5: the sea reaches the horizon all round the camera (its waves stay
    /// where they are in the world), not only its `size` rectangle.
    pub endless: bool,
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
            endless: false,
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
    /// (m/s), w unused. Its foam streaks lie along it.
    sea_wind: [f32; 4],
    /// The sea's clipmap ([`clipmap`]): x its levels, y cells along a
    /// level's edge, z the horizon ring's radius (m), w 1 when endless.
    clip: [f32; 4],
    /// Per level: its snapped centre (frame xz) and its cell (m).
    levels: [[f32; 4]; CLIP_LEVELS],
    /// The surface's frame from the world: a bounded sea is clamped to its
    /// rectangle in it.
    inverse: [[f32; 4]; 4],
    /// The FFT sea ([`crate::ocean_fft`]): x its cascades, y how many of
    /// the swell's waves (the first, longest) the vertices draw as
    /// Gerstner waves (the tick's long band), z the slope variance finer
    /// than the finest cascade, w unused.
    fft: [f32; 4],
    /// Per cascade: one over its tile (1/m), its uv at the frame's origin
    /// (whole tiles dropped), its tile (m).
    cascades: [[f32; 4]; 3],
    swell: [[f32; 4]; 2 * og_core::ocean::OCEAN_WAVES],
}

/// The scene's clock for the sea, in f64 seconds: its tick over its rate,
/// as the simulation's `og.physics.waterHeight` reads it.
pub(crate) fn seconds(scene: &crate::RenderScene) -> f64 {
    let rate = if scene.tick_rate > 0.0 {
        scene.tick_rate
    } else {
        60.0
    };
    scene.tick as f64 / f64::from(rate)
}

/// The surface's sea, if any (a grid with a wind: [`Water::wind`]).
pub(crate) fn ocean(water: &Water) -> Option<og_core::ocean::Ocean> {
    if water.mesh.is_some() {
        return None;
    }
    og_core::ocean::Ocean::new(water.wind, water.swell, water.choppiness)
}

/// The sea's block of [`WaterGpu`] at `seconds` and the scene's `origin`:
/// the header and the waves, their phases at the origin in f64 so the
/// pattern stays put in the world as the origin moves; and how many of
/// them (the longest) are the tick's long band, which the vertices draw as
/// Gerstner waves under the FFT's cascades (the rest of og_core's waves
/// float things in the tick but are not drawn: the cascades draw that band).
fn ocean_gpu(
    water: &Water,
    seconds: f64,
    origin: Vec3,
) -> ([f32; 4], [[f32; 4]; 2 * og_core::ocean::OCEAN_WAVES], u32) {
    let mut swell = [[0.0; 4]; 2 * og_core::ocean::OCEAN_WAVES];
    let Some(sea) = ocean(water) else {
        return ([0.0; 4], swell, 0);
    };
    let phases = sea.phases(
        seconds,
        glam::DVec2::new(f64::from(origin.x), f64::from(origin.z)),
    );
    for (i, (w, phase)) in sea.waves.iter().zip(phases).enumerate() {
        let k = w.direction * w.k;
        let lean = w.direction * w.lean;
        swell[2 * i] = [k.x, k.y, w.amplitude, phase];
        swell[2 * i + 1] = [lean.x, lean.y, w.k * w.lean, 0.0];
    }
    let params = crate::ocean_fft::spectrum::SeaParameters::from_ocean(
        water.wind,
        water.swell,
        water.choppiness,
        Some(&sea),
    );
    let long = sea
        .waves
        .iter()
        .take_while(|w| f64::from(w.k) < params.long_cut)
        .count() as u32;
    // Whitecaps where the surface's Jacobian falls under WHITECAP_J.
    ([1.0, 0.0, 1.0 - WHITECAP_J, WHITECAP_J], swell, long)
}

/// The Jacobian of the sea's horizontal displacement under which its
/// crests whiten (growing to full white where it reaches zero, a fold).
/// Batch 2's placeholder for the foam that persists (the FFT's batch 3).
const WHITECAP_J: f32 = 0.45;

/// Levels of a sea's clipmap ([`clipmap`]).
const CLIP_LEVELS: usize = 10;
/// Half the edge of the outermost level (m), where the horizon ring takes
/// over.
const CLIP_REACH: f64 = 4096.0;

/// A sea's clipmap about the eye at `eye` (frame coordinates) of `segments`
/// cells along each level's edge, after blue-water's (`geometry.rs`): each
/// level twice the cell and extent of the one inside it, the outermost
/// reaching [`CLIP_REACH`]; each centred on the eye snapped to twice its own
/// cell in the absolute world (the frame plus `origin`, in f64), so its
/// vertices sit on fixed world points as the eye and the origin move, and
/// the level inside it lies on its lattice. Per level: its centre (frame
/// xz) and its cell (m).
fn clipmap(eye: Vec3, origin: Vec3, segments: u32) -> [[f32; 4]; CLIP_LEVELS] {
    let first = 2.0 * CLIP_REACH / f64::from(segments) / f64::from(1u32 << (CLIP_LEVELS - 1));
    let (ox, oz) = (f64::from(origin.x), f64::from(origin.z));
    let (x, z) = (f64::from(eye.x) + ox, f64::from(eye.z) + oz);
    std::array::from_fn(|level| {
        let cell = first * f64::from(1u32 << level);
        let step = 2.0 * cell;
        let snap = |v: f64| (v / step).round() * step;
        [
            (snap(x) - ox) as f32,
            (snap(z) - oz) as f32,
            cell as f32,
            0.0,
        ]
    })
}

/// A sea's clipmap cells along a level's edge and its FFT cascades, by the
/// frame's quality: desktop presets 64 and three, the low preset (phones)
/// 32 and two.
fn sea_tier(quality: &crate::RenderQuality) -> (u32, u32) {
    if quality.volumetric_step_scale < 1.0 {
        (32, 2)
    } else {
        (64, 3)
    }
}

/// The vertices of a sea's draw of `segments` cells per level edge: every
/// level's square of cells (the hole each holds for the level inside it
/// collapses), then the horizon ring's quads round the outermost.
fn sea_vertices(segments: u32) -> u32 {
    CLIP_LEVELS as u32 * segments * segments * 6 + 2 * segments * 6
}

/// Each cascade's [`WaterGpu::cascades`] row at the scene's `origin`: one
/// over its tile, the uv of the frame's origin in it (f64, whole tiles
/// dropped: the pattern stays put in the world as the origin moves) and the
/// tile. Zero past the sea's cascades or without one.
fn cascade_uv(sea: Option<FftSea>, origin: Vec3) -> [[f32; 4]; 3] {
    let mut out = [[0.0; 4]; 3];
    let Some(sea) = sea else {
        return out;
    };
    for (row, &tile) in out.iter_mut().zip(&sea.tiles) {
        if tile > 0.0 {
            let at = |o: f32| (f64::from(o) / tile).rem_euclid(1.0) as f32;
            *row = [(1.0 / tile) as f32, at(origin.x), at(origin.z), tile as f32];
        }
    }
    out
}

/// The IEEE half nearest `v` (rounding half up), for the height fields'
/// rgba16float.
fn f16_bits(v: f32) -> u16 {
    let b = v.to_bits();
    let sign = ((b >> 16) & 0x8000) as u16;
    let exp = ((b >> 23) & 0xff) as i32 - 127 + 15;
    let man = b & 0x7f_ffff;
    if v.is_nan() {
        0x7e00
    } else if exp >= 31 {
        sign | 0x7c00
    } else if exp <= 0 {
        if exp < -10 {
            return sign;
        }
        let m = man | 0x80_0000;
        let shift = (14 - exp) as u32;
        let h = (m >> shift) + ((m >> (shift - 1)) & 1);
        sign | h as u16
    } else {
        let h = ((exp as u32) << 10 | (man >> 13)) + ((man >> 12) & 1);
        sign | h as u16
    }
}

/// The largest height field accepted.
const MAX_RESOLUTION: u32 = 512;

/// Grid cells per side: the surface's own `segments`, or more where a cell
/// would be longer than [`og_core::ocean::GRID_MAX_CELL`] (at most
/// [`og_core::ocean::GRID_MAX_SEGMENTS`]).
fn grid_segments(water: &Water) -> u32 {
    let scale = |axis: Vec4| axis.truncate().length();
    let extent = (water.size.x * scale(water.transform.x_axis))
        .max(water.size.y * scale(water.transform.z_axis));
    og_core::ocean::grid_segments(water.segments, extent)
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
pub(crate) struct WaterRenderer {
    layout: wgpu::BindGroupLayout,
    pub(crate) pipeline: crate::deferred::Sampled<wgpu::RenderPipeline>,
    /// For surfaces drawn over a mesh ([`Water::mesh`]).
    pub(crate) mesh_pipeline: crate::deferred::Sampled<wgpu::RenderPipeline>,
    /// The surfaces' view depth for the screen AO (`fs_ao_depth`; grid, mesh): into an R32 copy of the prepass
    /// depth, tested against the prepass, writing no depth.
    ao_depth_pipelines: crate::deferred::Deferred<[wgpu::RenderPipeline; 2]>,
    ripple_sampler: wgpu::Sampler,
    sampler: wgpu::Sampler,
    flat: wgpu::TextureView,
    slots: Vec<Slot>,
    /// The ripples' compute, made with the first surface that ripples;
    /// `compute` false (an adapter without compute shaders): none, the
    /// CPU's heights alone.
    ripple_pipelines: Option<crate::water_ripples::RipplePipelines>,
    compute: bool,
    /// The FFT sea ([`crate::ocean_fft`]) of the frame's first surface with
    /// a wind. Every sea of the frame is drawn from it (one wind a scene).
    fft: Option<(crate::ocean_fft::FftOcean, FftSea)>,
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
}

/// One surface ready to draw.
pub(crate) struct WaterDraw {
    group: wgpu::BindGroup,
    vertices: u32,
    /// The surface's world bounding sphere (its reflection probe: probe_select.rs).
    bounds: (Vec3, f32),
    /// Its mesh's vertex and index buffers and index count.
    mesh: Option<(wgpu::Buffer, wgpu::Buffer, u32)>,
}

impl WaterRenderer {
    /// Makes its pipeline if no frame used it yet (Renderer::warm).
    pub(crate) fn warm(&self) -> bool {
        crate::deferred::warm_one(&[&self.pipeline, &self.mesh_pipeline])
    }

    pub(crate) fn new(
        device: &wgpu::Device,
        frame_layout: &wgpu::BindGroupLayout,
        frame_wgsl: &str,
        lighting_wgsl: &str,
        samples: u32,
        compute: bool,
    ) -> WaterRenderer {
        let tex = |binding, filterable| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("water"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // A height field's (a 1-layer rgba16float array, height in r) or the FFT sea's fields (ocean_fft:
                // three layers per cascade, mipped), sampled filtered.
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                tex(2, true),
                tex(3, false),
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                tex(5, true),
                tex(6, true),
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // The frame's screen-space AO (white without): the reflection is darkened by it as
                // the scene behind already is (water.wgsl).
                tex(8, true),
                // The flow map (Water::flow_map; the fallback without).
                tex(9, true),
            ],
        });
        let module =
            crate::deferred::module(device, "water", water_source(frame_wgsl, lighting_wgsl));
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("water"),
            bind_group_layouts: &[Some(frame_layout), Some(&layout)],
            immediate_size: 0,
        });
        let mesh_pipeline = crate::deferred_hdr(
            device,
            &pl,
            module.clone(),
            samples,
            move |layout, module| crate::HdrPipeline {
                alpha_to_coverage: false,
                label: "water mesh",
                layout,
                module,
                vs: "vs_mesh",
                fs: "fs",
                buffers: &[Some(crate::POSITION_NORMAL_UV_LAYOUT)],
                cull: None,
                depth_write: true,
                depth_compare: wgpu::CompareFunction::Less,
                blend: None,
                samples,
            },
        );
        let ao_depth_pipelines = {
            let (device, pl, module) = (device.clone(), pl.clone(), module.clone());
            crate::deferred::Deferred::new(move || {
                use crate::deferred::AsModule;
                let m = module.module();
                [
                    ("water ao depth", "vs", &[][..]),
                    (
                        "water mesh ao depth",
                        "vs_mesh",
                        &[Some(crate::POSITION_NORMAL_UV_LAYOUT)][..],
                    ),
                ]
                .map(|(label, vs, buffers)| {
                    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some(label),
                        layout: Some(&pl),
                        vertex: wgpu::VertexState {
                            module: m,
                            entry_point: Some(vs),
                            compilation_options: Default::default(),
                            buffers,
                        },
                        primitive: wgpu::PrimitiveState::default(),
                        depth_stencil: Some(wgpu::DepthStencilState {
                            format: crate::DEPTH_FORMAT,
                            depth_write_enabled: Some(false),
                            depth_compare: Some(wgpu::CompareFunction::Less),
                            stencil: Default::default(),
                            bias: Default::default(),
                        }),
                        multisample: wgpu::MultisampleState::default(),
                        fragment: Some(wgpu::FragmentState {
                            module: m,
                            entry_point: Some("fs_ao_depth"),
                            compilation_options: Default::default(),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: crate::post::LINDEPTH_FORMAT,
                                blend: None,
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        multiview_mask: None,
                        cache: crate::pipeline_cache::of(&device).as_ref(),
                    })
                })
            })
        };
        let pipeline = crate::deferred_hdr(device, &pl, module, samples, move |layout, module| {
            crate::HdrPipeline {
                alpha_to_coverage: false,
                label: "water",
                layout,
                module,
                vs: "vs",
                fs: "fs",
                buffers: &[],
                cull: None,
                depth_write: true,
                depth_compare: wgpu::CompareFunction::Less,
                blend: None,
                samples,
            }
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("water"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        // The ripples' normal map: tiled, trilinear and 16x anisotropic (three.js water normals are RepeatWrapping
        // with mips): the clamped, mip-less water sampler blocked them up at grazing angles and seamed every tile.
        let ripple_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("water ripples"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            anisotropy_clamp: 16,
            ..Default::default()
        });
        let flat = height_texture(device, 1).1;
        WaterRenderer {
            ripple_sampler,
            layout,
            pipeline,
            mesh_pipeline,
            ao_depth_pipelines,
            sampler,
            flat,
            slots: Vec::new(),
            ripple_pipelines: None,
            compute: compute && crate::WORLD_3D,
            fft: None,
        }
    }

    /// Writes this frame's uniforms and height fields and makes the bind
    /// groups (the scene copy and the depth prepass are this frame's;
    /// `planar`: the surfaces drawn with this frame's planar reflections and
    /// their textures, `fallback` bound where there is none).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare<'t>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        waters: &[Water],
        screen: (u32, u32),
        scene_color: &wgpu::TextureView,
        scene_depth: &wgpu::TextureView,
        planar: &[(usize, &wgpu::TextureView)],
        ao: &wgpu::TextureView,
        ao_direct: f32,
        adaptation: f32,
        origin: Vec3,
        seconds: f64,
        camera: &crate::Camera,
        quality: &crate::RenderQuality,
        fallback: &wgpu::TextureView,
        mesh_buffers: &dyn Fn(
            crate::scene::MeshHandle,
        ) -> Option<(wgpu::Buffer, wgpu::Buffer, u32)>,
        textures: &dyn Fn(crate::TextureHandle) -> Option<&'t wgpu::TextureView>,
    ) -> Vec<WaterDraw> {
        while self.slots.len() < waters.len() {
            self.slots.push(Slot {
                uniform: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("water"),
                    size: std::mem::size_of::<WaterGpu>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                face: bytemuck::Zeroable::zeroed(),
                heights: None,
                ripples: None,
            });
        }
        let (w, h) = (screen.0 as f32, screen.1 as f32);
        let (sea_segments, sea_cascades) = sea_tier(quality);
        let sea = self.fft_sea(device, queue, waters, sea_cascades, seconds);
        let levels = clipmap(camera.position, origin, sea_segments);
        let mut draws = Vec::with_capacity(waters.len());
        for (index, (water, slot)) in waters.iter().zip(&mut self.slots).enumerate() {
            let reflection = planar.iter().find(|(i, _)| *i == index).map(|(_, v)| *v);
            let segments = grid_segments(water);
            let mesh = water.mesh.and_then(mesh_buffers);
            let field = height_field(water).filter(|_| mesh.is_none());
            if let Some(f) = field {
                if slot
                    .heights
                    .as_ref()
                    .is_none_or(|(r, ..)| *r != f.resolution)
                {
                    let (t, v) = height_texture(device, f.resolution);
                    slot.heights = Some((f.resolution, t, v));
                }
                let (_, texture, _) = slot.heights.as_ref().expect("made above");
                let texels: Vec<[u16; 4]> =
                    f.heights.iter().map(|&h| [f16_bits(h), 0, 0, 0]).collect();
                queue.write_texture(
                    texture.as_image_copy(),
                    bytemuck::cast_slice(&texels),
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(f.resolution * 8),
                        rows_per_image: None,
                    },
                    wgpu::Extent3d {
                        width: f.resolution,
                        height: f.resolution,
                        depth_or_array_layers: 1,
                    },
                );
            }
            // The ripples over the heights, stepped to this frame's tick.
            match field.and_then(|f| Some((f, f.wakes.as_deref()?))) {
                Some((f, wakes)) if self.compute => {
                    let pipes = self
                        .ripple_pipelines
                        .get_or_insert_with(|| crate::water_ripples::RipplePipelines::new(device));
                    if slot.ripples.as_ref().is_none_or(|r| !r.fits(f)) {
                        // The compute reads the heights' one layer as a plain 2D texture.
                        let (_, texture, _) = slot.heights.as_ref().expect("made above");
                        let coarse = texture.create_view(&Default::default());
                        slot.ripples = Some(crate::water_ripples::Ripples::new(
                            device, pipes, f, &coarse,
                        ));
                    }
                    let ripples = slot.ripples.as_mut().expect("made above");
                    ripples.update(device, queue, pipes, wakes);
                }
                _ => slot.ripples = None,
            }
            let normal_view = water.normal_map.and_then(textures);
            let flow_view = water.flow_map.and_then(textures);
            // With a flow map the flow's length is the speed: three.js'
            // flowSpeed 0.03 for a zero flow.
            let flow = if flow_view.is_some() && water.flow.length_squared() < 1e-12 {
                Vec2::new(0.03, 0.0)
            } else {
                water.flow
            };
            let (ocean, swell, long) = ocean_gpu(water, seconds, origin);
            let drawn_sea = sea.filter(|_| ocean[0] > 0.0);
            let gpu = WaterGpu {
                model: water.transform.to_cols_array_2d(),
                size: [
                    water.size.x.max(0.0),
                    water.size.y.max(0.0),
                    segments as f32,
                    // 2: the ripples' heights, read per pixel.
                    match (field, &slot.ripples) {
                        (Some(_), Some(_)) => 2.0,
                        (Some(_), None) => 1.0,
                        (None, _) => 0.0,
                    },
                ],
                shallow: water.shallow.extend(water.depth_fade.max(1e-3)).to_array(),
                deep: water.deep.extend(water.opacity.clamp(0.0, 1.0)).to_array(),
                foam: water.foam.extend(water.foam_width.max(0.0)).to_array(),
                waves: [
                    water.wave_scale.max(0.0),
                    water.wave_speed,
                    water.wave_strength.max(0.0),
                    water.refraction.max(0.0),
                ],
                look: [
                    water.reflectivity.max(0.0),
                    // The screen-space march on (water.wgsl).
                    1.0,
                    field.map_or(1.0, |f| f.size),
                    field.map_or(2.0, |f| match slot.ripples {
                        Some(_) => crate::water_ripples::Ripples::side(f.resolution) as f32,
                        None => f.resolution as f32,
                    }),
                ],
                screen: [w, h, 1.0 / w.max(1.0), 1.0 / h.max(1.0)],
                extra: [
                    water.shadows.clamp(0.0, 1.0),
                    f32::from(reflection.is_some()),
                    water.f0.clamp(0.0, 1.0),
                    water.reflection_bias.clamp(0.0, 1.0),
                ],
                under: water
                    .underwater_color
                    .extend(water.glint.max(0.0))
                    .to_array(),
                normals: [
                    water.normal_scale.max(0.0),
                    flow.x,
                    flow.y,
                    f32::from(normal_view.is_some()),
                ],
                ao: [
                    ao_direct.clamp(0.0, 1.0),
                    f32::from(flow_view.is_some()),
                    f32::from(mesh.is_some()),
                    1.0 / adaptation.max(1e-3),
                ],
                layers: lake_layers(origin, water.normal_scale.max(0.0)),
                ocean,
                sea_wind: {
                    let d = water.wind.normalize_or_zero();
                    [d.x, d.y, water.wind.length() * ocean[0], 0.0]
                },
                clip: [
                    CLIP_LEVELS as f32,
                    sea_segments as f32,
                    camera.far * 0.95,
                    f32::from(water.endless),
                ],
                levels,
                inverse: water.transform.inverse().to_cols_array_2d(),
                fft: match drawn_sea {
                    Some(f) => [
                        f.tiles.iter().filter(|t| **t > 0.0).count() as f32,
                        long as f32,
                        f.tail,
                        0.0,
                    ],
                    None => [0.0; 4],
                },
                cascades: cascade_uv(drawn_sea, origin),
                swell,
            };
            queue.write_buffer(&slot.uniform, 0, bytemuck::bytes_of(&gpu));
            // A probe face has no planar reflection of its own (that would be
            // a mirrored pass per face); its screen-space reflection marches
            // the face's own depth prepass and copy, as the frame's does,
            // mirroring what the probe sees from its point.
            slot.face = gpu;
            slot.face.extra[1] = 0.0;
            let heights = match (&slot.ripples, &slot.heights, field, &self.fft) {
                (_, _, _, Some((fft, _))) if drawn_sea.is_some() => &fft.view,
                (Some(r), _, Some(_), _) => &r.view,
                (None, Some((_, _, v)), Some(_), _) => v,
                _ => &self.flat,
            };
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("water"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: slot.uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(heights),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(scene_color),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(scene_depth),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::TextureView(
                            reflection.unwrap_or(fallback),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::TextureView(
                            normal_view.unwrap_or(fallback),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::Sampler(&self.ripple_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: wgpu::BindingResource::TextureView(ao),
                    },
                    wgpu::BindGroupEntry {
                        binding: 9,
                        resource: wgpu::BindingResource::TextureView(flow_view.unwrap_or(fallback)),
                    },
                ],
            });
            let half = Vec3::new(water.size.x, 0.0, water.size.y) * 0.5;
            let radius = water.transform.transform_vector3(half).length();
            draws.push(WaterDraw {
                group,
                vertices: if drawn_sea.is_some() {
                    sea_vertices(sea_segments)
                } else {
                    segments * segments * 6
                },
                bounds: (water.transform.transform_point3(Vec3::ZERO), radius),
                mesh,
            });
        }
        draws
    }

    /// The surfaces as the faces of a reflection probe's capture of face
    /// size `size` draw them (after [`WaterRenderer::prepare`]): through
    /// `below`, the copy of the face drawn so far, and over `scene_depth`,
    /// the face's depth prepass, so the face refracts what lies below,
    /// foams at the edges and reflects the face in screen
    /// space as the frame does; without the planar reflection.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn face_draws<'a>(
        &self,
        device: &wgpu::Device,
        waters: &[Water],
        size: u32,
        below: &wgpu::TextureView,
        scene_depth: &wgpu::TextureView,
        white: &wgpu::TextureView,
        textures: &dyn Fn(crate::scene::TextureHandle) -> Option<&'a wgpu::TextureView>,
    ) -> Vec<WaterDraw> {
        use wgpu::util::DeviceExt;
        let n = size as f32;
        waters
            .iter()
            .zip(&self.slots)
            .map(|(water, slot)| {
                let sea = slot.face.fft[0] > 0.0;
                let heights = match (&slot.ripples, &slot.heights, height_field(water), &self.fft) {
                    (_, _, _, Some((fft, _))) if sea => &fft.view,
                    (Some(r), _, Some(_), _) => &r.view,
                    (None, Some((_, _, v)), Some(_), _) => v,
                    _ => &self.flat,
                };
                let mut face = slot.face;
                face.screen = [n, n, 1.0 / n, 1.0 / n];
                let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("water (probe face)"),
                    contents: bytemuck::bytes_of(&face),
                    usage: wgpu::BufferUsages::UNIFORM,
                });
                let tex = wgpu::BindingResource::TextureView;
                // No planar reflection nor screen AO in a face: its own
                // copy and white in their slots.
                let entries = [
                    uniform.as_entire_binding(),
                    tex(heights),
                    tex(below),
                    tex(scene_depth),
                    wgpu::BindingResource::Sampler(&self.sampler),
                    tex(below),
                    tex(water.normal_map.and_then(textures).unwrap_or(white)),
                    wgpu::BindingResource::Sampler(&self.ripple_sampler),
                    tex(white),
                    tex(water.flow_map.and_then(textures).unwrap_or(white)),
                ];
                let entries: Vec<wgpu::BindGroupEntry> = entries
                    .into_iter()
                    .enumerate()
                    .map(|(i, resource)| wgpu::BindGroupEntry {
                        binding: i as u32,
                        resource,
                    })
                    .collect();
                let half = Vec3::new(water.size.x, 0.0, water.size.y) * 0.5;
                WaterDraw {
                    group: device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("water (probe face)"),
                        layout: &self.layout,
                        entries: &entries,
                    }),
                    // A surface over a mesh is left out of the faces (the
                    // faces' draws carry no mesh buffers).
                    vertices: if water.mesh.is_some() {
                        0
                    } else if sea {
                        sea_vertices(slot.face.clip[1] as u32)
                    } else {
                        grid_segments(water) * grid_segments(water) * 6
                    },
                    mesh: None,
                    bounds: (
                        water.transform.transform_point3(Vec3::ZERO),
                        water.transform.transform_vector3(half).length(),
                    ),
                }
            })
            .collect()
    }

    /// The FFT sea of the frame's first surface with a wind, its fields
    /// computed at `seconds` (the scene's tick time, so a frame is the
    /// tick's whatever the wall clock) and submitted ahead of the frame:
    /// its spectrum rebuilt when that sea's wind, swell or choppiness or the
    /// cascades change. None without a sea.
    fn fft_sea(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        waters: &[Water],
        cascades: u32,
        seconds: f64,
    ) -> Option<FftSea> {
        use crate::ocean_fft::{FftOcean, spectrum};
        let (water, sea) = waters.iter().find_map(|w| Some((w, ocean(w)?)))?;
        let key = (water.wind, water.swell, water.choppiness, cascades);
        if self.fft.as_ref().is_none_or(|(_, f)| f.key != key) {
            let mut fft = match self.fft.take() {
                Some((f, ..)) if f.cascades() == cascades => f,
                _ => FftOcean::new(device, cascades),
            };
            let params = spectrum::SeaParameters::from_ocean(
                water.wind,
                water.swell,
                water.choppiness,
                Some(&sea),
            );
            let built = spectrum::build(&spectrum::cascades(cascades as usize, &params), &params);
            fft.set_spectrum(queue, &built);
            let mut tiles = [0.0; 3];
            for (t, c) in tiles.iter_mut().zip(&built.cascades) {
                *t = c.size;
            }
            let info = FftSea {
                key,
                tiles,
                tail: built.tail_slope_variance as f32,
            };
            self.fft = Some((fft, info));
        }
        let (fft, info) = self.fft.as_ref().expect("made above");
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ocean fft"),
        });
        fft.encode(queue, &mut encoder, seconds);
        queue.submit([encoder.finish()]);
        Some(*info)
    }

    /// Draws the surfaces' view depth for the screen AO (`fs_ao_depth`; group 0: the frame). Returns the draw calls.
    pub(crate) fn draw_ao_depth(&self, pass: &mut wgpu::RenderPass, draws: &[WaterDraw]) -> u32 {
        let [grid, mesh] = self.ao_depth_pipelines.get();
        for d in draws {
            pass.set_bind_group(1, &d.group, &[]);
            match &d.mesh {
                Some((vertices, indices, count)) => {
                    pass.set_pipeline(mesh);
                    pass.set_vertex_buffer(0, vertices.slice(..));
                    pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..*count, 0, 0..1);
                }
                None => {
                    pass.set_pipeline(grid);
                    pass.draw(0..d.vertices, 0..1);
                }
            }
        }
        draws.len() as u32
    }

    /// Draws the surfaces (group 0: the frame, or each surface's reflection
    /// probe's: probe_select.rs). Returns the draw calls.
    pub(crate) fn draw(
        &self,
        pass: &mut wgpu::RenderPass,
        draws: &[WaterDraw],
        probes: &mut crate::probe_select::Binder,
    ) -> u32 {
        for d in draws {
            probes.at(pass, d.bounds.0, d.bounds.1);
            pass.set_bind_group(1, &d.group, &[]);
            match &d.mesh {
                Some((vertices, indices, count)) => {
                    pass.set_pipeline(&self.mesh_pipeline);
                    pass.set_vertex_buffer(0, vertices.slice(..));
                    pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..*count, 0, 0..1);
                }
                None => {
                    pass.set_pipeline(&self.pipeline);
                    pass.draw(0..d.vertices, 0..1);
                }
            }
        }
        probes.reset(pass);
        draws.len() as u32
    }
}

/// The surface's plane in world space: (unit normal facing up out of the
/// water, a point on it).
pub(crate) fn plane(water: &Water) -> (Vec3, Vec3) {
    let up = water
        .transform
        .transform_vector3(Vec3::Y)
        .normalize_or(Vec3::Y);
    (up, water.transform.transform_point3(Vec3::ZERO))
}

/// The surface as the screen AO and the planar reflection see it: its plane (normal, a point) and its world xz extent.
/// A surface over a mesh ([`Water::mesh`], `mesh` its data) lies at the mean height of the vertices it draws (its
/// indices; the entity's own origin can be anywhere, e.g. a mesh in world coordinates under an entity at 0) and spans
/// their extent; a grid spans its size around the entity.
pub(crate) fn surface(
    water: &Water,
    mesh: Option<&crate::scene::MeshData>,
) -> (Vec3, Vec3, Vec2, Vec2) {
    let (up, origin) = plane(water);
    if let Some(m) = mesh.filter(|m| !m.indices.is_empty()) {
        let (mut sum, mut lo, mut hi) = (Vec3::ZERO, Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
        let mut n = 0.0;
        for &i in &m.indices {
            let Some(v) = m.vertices.get(i as usize) else {
                continue;
            };
            let p = Vec3::from(v.position);
            sum += p;
            n += 1.0;
            let w = water.transform.transform_point3(p);
            lo = lo.min(Vec2::new(w.x, w.z));
            hi = hi.max(Vec2::new(w.x, w.z));
        }
        if n > 0.0 {
            return (up, water.transform.transform_point3(sum / n), lo, hi);
        }
    }
    let h = water.size * 0.5;
    let (mut lo, mut hi) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
    for c in [
        Vec3::new(-h.x, 0.0, -h.y),
        Vec3::new(h.x, 0.0, -h.y),
        Vec3::new(-h.x, 0.0, h.y),
        Vec3::new(h.x, 0.0, h.y),
    ] {
        let w = water.transform.transform_point3(c);
        lo = lo.min(Vec2::new(w.x, w.z));
        hi = hi.max(Vec2::new(w.x, w.z));
    }
    (up, origin, lo, hi)
}

/// The surface's height field when it is one the renderer uses: 2 to
/// [`MAX_RESOLUTION`] heights per side, all of them, over a positive size.
fn height_field(water: &Water) -> Option<&WaterHeights> {
    water.heights.as_ref().filter(|f| {
        f.resolution >= 2
            && f.resolution <= MAX_RESOLUTION
            && f.heights.len() == (f.resolution * f.resolution) as usize
            && f.size > 0.0
    })
}

/// The surface's height above its rest plane at local xz (the height
/// field, bilinear; 0 without one).
fn local_height(water: &Water, local: Vec2) -> f32 {
    let Some(f) = height_field(water) else {
        return 0.0;
    };
    let n = f.resolution as usize;
    let cell = f.size / (n - 1) as f32;
    let g =
        ((local + Vec2::splat(f.size * 0.5)) / cell).clamp(Vec2::ZERO, Vec2::splat((n - 1) as f32));
    let i = g.floor().min(Vec2::splat((n - 2) as f32));
    let t = g - i;
    let (x, z) = (i.x as usize, i.y as usize);
    let at = |x: usize, z: usize| f.heights[z * n + x];
    let lo = at(x, z) + (at(x + 1, z) - at(x, z)) * t.x;
    let hi = at(x, z + 1) + (at(x + 1, z + 1) - at(x, z + 1)) * t.x;
    lo + (hi - lo) * t.y
}

/// The surface the camera at `eye` is below (inside its extent, under its
/// waves), if any, among those with an underwater view.
/// `seconds` and the scene's `origin` place the sea's waves ([`Water::wind`]).
pub(crate) fn underwater(waters: &[Water], eye: Vec3, seconds: f64, origin: Vec3) -> Option<usize> {
    waters.iter().position(|w| {
        if w.underwater_density.is_nan() || w.underwater_density <= 0.0 {
            return false;
        }
        let local = w.transform.inverse().transform_point3(eye);
        let sea = ocean(w).map_or(0.0, |sea| {
            let at = (eye + origin).as_dvec3();
            sea.height(glam::DVec2::new(at.x, at.z), seconds)
        });
        local.x.abs() <= w.size.x * 0.5
            && local.z.abs() <= w.size.y * 0.5
            && local.y < local_height(w, Vec2::new(local.x, local.z)) + sea
    })
}

/// The fog of the underwater view: the water's colour, thickening by its
/// density from the eye, the sky left clear (Snell's window shows it).
pub(crate) fn underwater_fog(w: &Water) -> crate::Fog {
    crate::Fog {
        color: w.underwater_color,
        density: w.underwater_density,
        start: 0.0,
        end: 0.0,
        max_opacity: 1.0,
        height_density: 0.0,
        sky: 0.0,
        sun_color: w.underwater_color,
        sun_power: 8.0,
        ..Default::default()
    }
}

/// The frame's caustics: the first surface that has them, as
/// `FrameUniform::caustics` (height, strength, scale, fade; then the xz
/// rectangle; then the two cell layers at the scene's `origin`, each its
/// whole cells and the rest, so the pattern stays put when it moves:
/// `origin.rs`). Zero without one.
pub(crate) fn caustics(waters: &[Water], origin: Vec3) -> [[f32; 4]; 4] {
    let Some(w) = waters.iter().find(|w| w.caustics > 0.0) else {
        return [[0.0; 4]; 4];
    };
    let (_, at) = plane(w);
    let mut lo = Vec2::splat(f32::MAX);
    let mut hi = Vec2::splat(f32::MIN);
    for (x, z) in [(-0.5f32, -0.5f32), (0.5, -0.5), (-0.5, 0.5), (0.5, 0.5)] {
        let p = w
            .transform
            .transform_point3(Vec3::new(x * w.size.x, 0.0, z * w.size.y));
        lo = lo.min(Vec2::new(p.x, p.z));
        hi = hi.max(Vec2::new(p.x, p.z));
    }
    let scale = w.caustics_scale.max(1e-3);
    // lighting.wgsl `caustics`: the cells of `p.xz * scale`, and of 1.37
    // times that.
    let cells = |k: f64| {
        let at = |o: f32| f64::from(o) * f64::from(scale) * k;
        let (x, z) = (at(origin.x), at(origin.z));
        let (wx, wz) = (x.floor(), z.floor());
        [wx as f32, wz as f32, (x - wx) as f32, (z - wz) as f32]
    };
    [
        [at.y, w.caustics, scale, 0.15],
        [lo.x, lo.y, hi.x, hi.y],
        cells(1.0),
        cells(f64::from(1.37f32)),
    ]
}

/// The camera mirrored in the plane through `at` with unit normal `n`.
pub(crate) fn mirrored_camera(camera: &crate::Camera, n: Vec3, at: Vec3) -> crate::Camera {
    let point = |p: Vec3| p - n * (2.0 * (p - at).dot(n));
    let dir = |d: Vec3| d - n * (2.0 * d.dot(n));
    crate::Camera {
        position: point(camera.position),
        target: point(camera.target),
        up: dir(camera.up),
        ..*camera
    }
}

/// `proj` with its near plane replaced by the world plane (`n`, `at`)
/// seen through `view` (Lengyel's oblique near-plane clipping, for depth
/// 0..1): what lies below the plane is clipped and the far plane is
/// tilted to keep the depth range. `lift` raises the plane (m) to hide
/// the seam where geometry meets the water.
pub(crate) fn oblique(proj: Mat4, view: Mat4, n: Vec3, at: Vec3, lift: f32) -> Mat4 {
    let world = n.extend(-(n.dot(at) + lift));
    // Planes transform by the inverse transpose.
    let c = view.inverse().transpose() * world;
    let q = proj.inverse() * Vec4::new(c.x.signum(), c.y.signum(), 1.0, 1.0);
    let c = c * (1.0 / c.dot(q));
    let mut rows = [proj.row(0), proj.row(1), c, proj.row(3)];
    if !rows[2].is_finite() {
        rows[2] = proj.row(2);
    }
    Mat4::from_cols(rows[0], rows[1], rows[2], rows[3]).transpose()
}

/// A height field's texture: `n` × `n` rgba16float, the height in r,
/// viewed as a 1-layer array (water.wgsl `heights`; rgba16float is
/// filterable, as the FFT sea's fields bound there are sampled, and a
/// storage format for a compute pass writing it).
fn height_texture(device: &wgpu::Device, n: u32) -> (wgpu::Texture, wgpu::TextureView) {
    let t = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("water heights"),
        size: wgpu::Extent3d {
            width: n,
            height: n,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: HEIGHT_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let v = t.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
    });
    (t, v)
}

/// Appends the strips of the 2D water bodies to `frame` as runs in the
/// sprite order: the body (a gradient from `top` at the surface to
/// `bottom`) and the lip along the surface (soft across its width).
pub(crate) fn water_2d_strips(waters: &[Water2d], frame: &mut crate::ribbons::RibbonFrame) {
    use crate::ribbons::{RibbonRun, RibbonVertex};
    for w in waters {
        let n = w.surface.len();
        if n < 2 || w.width.is_nan() || w.depth.is_nan() || w.width <= 0.0 || w.depth <= 0.0 {
            continue;
        }
        let first = frame.vertices.len() as u32;
        let at = |i: usize| {
            let x = w.origin.x + w.width * i as f32 / (n - 1) as f32;
            let h = w.surface[i];
            (x, w.origin.y + if h.is_finite() { h } else { 0.0 })
        };
        let bottom_y = w.origin.y - w.depth;
        let top = w.top.to_array();
        let bottom = w.bottom.to_array();
        let lip = w.lip.to_array();
        let half = w.lip_width.max(0.0) * 0.5;
        for i in 0..n - 1 {
            let (x0, y0) = at(i);
            let (x1, y1) = at(i + 1);
            let v = |x: f32, y: f32| glam::Vec3::new(x, y, w.z);
            // Body: two triangles from the surface down.
            let (a, b, c, d) = (v(x0, y0), v(x1, y1), v(x0, bottom_y), v(x1, bottom_y));
            for (p, col) in [
                (a, top),
                (c, bottom),
                (b, top),
                (b, top),
                (c, bottom),
                (d, bottom),
            ] {
                frame
                    .vertices
                    .push(RibbonVertex::new(p, 0.5, 0.5, col, true));
            }
            if half > 0.0 {
                let (a, b) = (v(x0, y0 + half), v(x1, y1 + half));
                let (c, d) = (v(x0, y0 - half), v(x1, y1 - half));
                for (p, vv) in [(a, 0.0), (c, 1.0), (b, 0.0), (b, 0.0), (c, 1.0), (d, 1.0)] {
                    frame
                        .vertices
                        .push(RibbonVertex::new(p, 0.5, vv, lip, false));
                }
            }
        }
        frame.runs.push(RibbonRun {
            texture: None,
            blend: crate::ParticleBlend::Alpha,
            layer: Some(w.layer),
            first,
            count: frame.vertices.len() as u32 - first,
        });
    }
}

/// The water module after the frame group and the lighting.
pub(crate) fn water_source(frame_wgsl: &str, lighting_wgsl: &str) -> String {
    format!("{frame_wgsl}{lighting_wgsl}{}", wgsl3d!("water.wgsl"))
}

/// The lake's four layers' uv at `origin` (water.wgsl `map_slope`: world
/// xz times the repeats over each layer's period), in f64 and without the
/// whole repeats: zero at the world's zero.
fn lake_layers(origin: Vec3, repeats: f32) -> [[f32; 4]; 2] {
    let k = f64::from(repeats);
    let at = |o: f32, period: f64| (f64::from(o) * k / period).rem_euclid(1.0) as f32;
    let (x, z) = (origin.x, origin.z);
    [
        [at(x, 103.0), at(z, 103.0), at(x, 107.0), at(z, 107.0)],
        [at(x, 8907.0), at(z, 9803.0), at(x, 1091.0), at(z, 1027.0)],
    ]
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
            let (head, swell, long) = ocean_gpu(&water, seconds, origin);
            assert_eq!(head[0], 1.0);
            assert!(long > 0 && (long as usize) < og_core::ocean::OCEAN_WAVES);
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
}
