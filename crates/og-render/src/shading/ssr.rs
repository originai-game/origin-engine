//! Screen-space reflections (`docs/design.md` §2.6): stochastic,
//! hierarchical and temporally denoised, after FidelityFX SSSR (MIT,
//! `third_party/fidelityfx-sssr`) and Frostbite's stochastic SSR
//! (Stachowiak 2015). One graph pass, `FramePass::Ssr`, between the
//! depth-normal prepass (and the AO passes) and the main pass, records
//! `ssr.wgsl`'s compute passes:
//!
//! 1. the min-max depth pyramid of the prepass's view depth;
//! 2. one ray a pixel ([`ScreenSpaceReflections::resolution`] full), or a
//!    2x2 block (half, the default; the denoiser then runs on the blocks
//!    too, and a joint bilateral upsample by depth, normal and roughness
//!    writes the pixels), drawn from the GGX distribution of visible
//!    normals for the pixel's roughness (its grazing rim left out,
//!    Stachowiak's bias; each pixel's frames walk the R2 sequence) and
//!    marched through the pyramid (FidelityFX's hierarchical march and its
//!    hit validation); a hit
//!    reads the light the previous frame's lit image (TAA's history, or
//!    the main pass's copy that screen-space GI reads too) holds where the
//!    hit point was;
//! 3. the resolve: each pixel weighs its neighbours' rays by its own GGX
//!    lobe over each ray's pdf (Frostbite's ray reuse), so one ray a pixel
//!    resolves a glossy reflection;
//! 4. the spatial prefilter: the FidelityFX denoiser's edge-stopping blur
//!    over 15 Halton neighbours (normal, depth, roughness and distance from
//!    the local mean), wider as the lobe widens; mirrors skip it; at half
//!    resolution a second, a-trous pass at twice the step on rough lobes
//!    (Dammertz et al. 2010), since a block grid has a quarter of the rays;
//! 5. the temporal pass: the FidelityFX reflection denoiser's reprojection
//!    (a mirror follows its reflected point's parallax, a rough surface its
//!    own motion; a disoccluded history is dropped) and its temporal
//!    resolve (the history clipped to the neighbourhood's box, its
//!    variance averaged over four frames as SVGF's so a rough lobe's
//!    sparse tail keeps its spread, and a still history's box centred on
//!    the neighbourhood's mean over the last four frames, NRD's fast
//!    history; accumulated up to 32 samples on rough surfaces, 64 when
//!    still, and 8 on mirrors). The lit shaders read a copy of the
//!    frame's history.
//!
//! The result is the reflected radiance premultiplied by the share of the
//! lobe the screen resolved. Every lit shader that writes the prepass's
//! reflectivity reads it in the frame group (`frame.wgsl` `ssr_tex` and
//! `ssr_reflected`: a read-only storage texture, so it takes none of their
//! sixteen texture slots; `Frame.ssr` on for the main camera) and blends it
//! over the radiance the reflection probes, the probe GI's specular and
//! the environment give (`gi_reflected_from`), before the split-sum's
//! Fresnel: diffuse light is untouched, and where a ray leaves the screen,
//! hits the sky or is too rough to trace, the probes and the sky show.
//! The prepass carries the roughness and metalness (lindepth's second
//! channel); terrain and scattered copies write theirs too.

use bytemuck::{Pod, Zeroable};
use glam::Mat4;

use crate::AoResolution;
use crate::renderer::history::GpuCopy;
use crate::renderer::{sampler_entry, uniform_entry};

const SSR_WGSL: &str = wgsl!("ssr.wgsl");

/// The passes' module source (`shaders.rs` validates it).
pub(crate) fn source() -> String {
    SSR_WGSL.to_string()
}

/// The format of every target the lit shaders and the passes read.
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// Screen-space reflections ([`crate::PostEffects::ssr`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenSpaceReflections {
    /// How far a reflected ray travels, in world units. The hierarchical
    /// march's cost is bounded by `steps`, not by the distance.
    pub max_distance: f32,
    /// How far behind a surface a ray may pass and still hit it (world units).
    pub thickness: f32,
    /// The hierarchical march's iterations (4 to 128).
    pub steps: u32,
    /// The fraction of the screen at its edges over which reflections fade out.
    pub fade: f32,
    /// Scales the reflection (0: none; 1 physical).
    pub intensity: f32,
    /// The roughest surface traced (0 to 1): rougher ones keep the probes'
    /// reflection, and the last quarter below it fades into them.
    pub max_roughness: f32,
    /// Half (the default): one ray a 2x2 block, denoised on the blocks and
    /// upsampled to the pixels by their depth, normal and roughness; Full:
    /// one ray a pixel, denoised at full resolution.
    pub resolution: AoResolution,
}

impl Default for ScreenSpaceReflections {
    fn default() -> Self {
        ScreenSpaceReflections {
            max_distance: 100.0,
            thickness: 0.3,
            steps: 64,
            fade: 0.15,
            intensity: 1.0,
            max_roughness: 0.6,
            resolution: AoResolution::Half,
        }
    }
}

impl ScreenSpaceReflections {
    /// The main camera's `Frame.ssr` row: x 1 with reflections, y the
    /// intensity.
    pub(crate) fn frame_row(s: Option<&ScreenSpaceReflections>) -> [f32; 4] {
        match s {
            Some(s) => [1.0, s.intensity.clamp(0.0, 1.0), 0.0, 0.0],
            None => [0.0; 4],
        }
    }

    /// How many full-resolution pixels a ray texel spans in x and y.
    fn divisor(&self) -> u32 {
        match self.resolution {
            AoResolution::Full => 1,
            AoResolution::Half => 2,
        }
    }
}

/// `ssr.wgsl`'s `Ssr`.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SsrUniform {
    proj: [[f32; 4]; 4],
    inv_proj: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    inv_view: [[f32; 4]; 4],
    prev_view_proj: [[f32; 4]; 4],
    screen: [f32; 4],
    trace: [f32; 4],
    params: [f32; 4],
    flags: [f32; 4],
    ray: [f32; 4],
    chain: [f32; 4],
}

/// The camera of the frame the reflections are traced for.
pub(crate) struct SsrCamera {
    /// World to view.
    pub view: Mat4,
    /// World to clip, unjittered (the prepass's velocity is too).
    pub view_proj: Mat4,
    pub near: f32,
    pub far: f32,
}

/// The pixel of its 2x2 block a half-resolution ray traces on `frame`:
/// the four in turn, so four frames cover every pixel.
fn block_offset(frame: u64) -> [f32; 2] {
    [[0.0, 0.0], [1.0, 1.0], [1.0, 0.0], [0.0, 1.0]][(frame % 4) as usize]
}

/// The min-max depth pyramid's levels above the prepass's view depth
/// (level 0): two dispatches of four (`ssr.wgsl` `hiz_down`), the coarsest
/// a cell of 256 pixels.
const PYRAMID_LEVELS: u32 = 8;

/// The pyramid's level 1 for a `width` x `height` screen: half the screen
/// padded to 256 pixels, so that every level halves exactly.
fn pyramid_size(width: u32, height: u32) -> (u32, u32) {
    let pad = 1 << PYRAMID_LEVELS;
    (
        width.div_ceil(pad) * pad / 2,
        height.div_ceil(pad) * pad / 2,
    )
}

struct Pipelines {
    common: wgpu::BindGroupLayout,
    layouts: [wgpu::BindGroupLayout; 8],
    passes: [wgpu::ComputePipeline; 8],
    sampler: wgpu::Sampler,
}

/// The passes' indices in [`Pipelines`].
const HIZ_BASE: usize = 0;
const HIZ_REDUCE: usize = 1;
const TRACE: usize = 2;
const RESOLVE: usize = 3;
const PREFILTER: usize = 4;
const TEMPORAL: usize = 5;
const UPSAMPLE: usize = 6;
const PREFILTER_WIDE: usize = 7;

struct Targets {
    size: (u32, u32),
    divisor: u32,
    ray_size: (u32, u32),
    /// The pyramid's levels, and all of them in one view.
    hiz_levels: Vec<wgpu::TextureView>,
    hiz: wgpu::TextureView,
    ray_radiance: wgpu::TextureView,
    ray_hit: wgpu::TextureView,
    resolved: wgpu::TextureView,
    filtered: wgpu::TextureView,
    /// Ping-ponged by the frame's parity.
    history: [wgpu::TextureView; 2],
    history_textures: [wgpu::Texture; 2],
    meta: [wgpu::TextureView; 2],
    /// The neighbourhood's variance, accumulated over frames.
    spread: [wgpu::TextureView; 2],
    /// The neighbourhood's mean over the last four frames (the anti-lag's).
    fast: [wgpu::TextureView; 2],
    /// What the lit shaders read: the frame's history (full resolution)
    /// or its upsample (half), copied.
    lit: wgpu::TextureView,
    lit_texture: wgpu::Texture,
    /// At half resolution, what the upsample writes before the copy.
    upsampled: Option<(wgpu::TextureView, wgpu::Texture)>,
    /// The denoiser's guide: normal, depth and roughness in one texel.
    guide: wgpu::TextureView,
}

fn texture(
    device: &wgpu::Device,
    label: &str,
    (width, height): (u32, u32),
    format: wgpu::TextureFormat,
    mips: u32,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: mips,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

impl Targets {
    /// What the temporal pass carries to the next frame: the history,
    /// its meta, spread and fast mean, both of each pair.
    fn history_resources(&self) -> Vec<&wgpu::Texture> {
        self.history
            .iter()
            .chain(&self.meta)
            .chain(&self.spread)
            .chain(&self.fast)
            .map(wgpu::TextureView::texture)
            .collect()
    }

    fn new(device: &wgpu::Device, size: (u32, u32), divisor: u32) -> Targets {
        let ray_size = (size.0.div_ceil(divisor), size.1.div_ceil(divisor));
        let hiz = texture(
            device,
            "ssr depth pyramid",
            pyramid_size(size.0, size.1),
            wgpu::TextureFormat::Rg32Float,
            PYRAMID_LEVELS,
        );
        let hiz_levels = (0..PYRAMID_LEVELS)
            .map(|level| {
                hiz.create_view(&wgpu::TextureViewDescriptor {
                    base_mip_level: level,
                    mip_level_count: Some(1),
                    ..Default::default()
                })
            })
            .collect();
        let view = |label: &str, size, format| {
            texture(device, label, size, format, 1).create_view(&Default::default())
        };
        // The denoiser runs on the ray grid; at half resolution the
        // upsample writes the full-resolution image the lit shaders read.
        let history_textures = [0, 1].map(|_| texture(device, "ssr history", ray_size, FORMAT, 1));
        let lit_texture = texture(device, "ssr", size, FORMAT, 1);
        Targets {
            size,
            divisor,
            ray_size,
            hiz_levels,
            hiz: hiz.create_view(&Default::default()),
            ray_radiance: view("ssr rays", ray_size, FORMAT),
            ray_hit: view("ssr ray hits", ray_size, FORMAT),
            resolved: view("ssr resolved", ray_size, FORMAT),
            filtered: view("ssr filtered", ray_size, FORMAT),
            history: [0, 1].map(|i| history_textures[i].create_view(&Default::default())),
            history_textures,
            meta: [0, 1].map(|_| view("ssr history meta", ray_size, FORMAT)),
            spread: [0, 1].map(|_| view("ssr history spread", ray_size, FORMAT)),
            fast: [0, 1].map(|_| view("ssr history fast", ray_size, FORMAT)),
            lit: lit_texture.create_view(&Default::default()),
            lit_texture,
            upsampled: (divisor > 1).then(|| {
                let t = texture(device, "ssr upsampled", size, FORMAT, 1);
                (t.create_view(&Default::default()), t)
            }),
            guide: view("ssr guide", ray_size, FORMAT),
        }
    }
}

/// [`SsrRenderer::save`]: the targets' size and divisor, the copy of
/// their accumulation, and the frame that wrote it.
pub(crate) struct SsrSave {
    size: (u32, u32),
    divisor: u32,
    copy: GpuCopy,
    history: Option<(u64, Mat4)>,
}

impl SsrSave {
    pub(crate) fn bytes(&self) -> u64 {
        self.copy.bytes()
    }
}

/// The reflections' passes, targets and history (`Renderer::ssr`).
pub(crate) struct SsrRenderer {
    pipelines: Option<Pipelines>,
    targets: Option<Targets>,
    uniform: wgpu::Buffer,
    /// What the lit shaders read in a frame without reflections: one
    /// texel of nothing.
    off: wgpu::TextureView,
    /// The frame whose temporal pass wrote the history, and its camera.
    history: Option<(u64, Mat4)>,
    /// Bumped when the view the lit shaders read changes.
    pub generation: u64,
}

impl SsrRenderer {
    pub fn new(device: &wgpu::Device) -> SsrRenderer {
        SsrRenderer {
            pipelines: None,
            targets: None,
            uniform: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ssr"),
                size: std::mem::size_of::<SsrUniform>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            off: texture(device, "ssr off", (1, 1), FORMAT, 1).create_view(&Default::default()),
            history: None,
            generation: 0,
        }
    }

    /// What the main camera's frame groups bind at `ssr_tex` (frame.wgsl):
    /// the lit shaders' read, which `Frame.ssr` says holds this frame's
    /// reflections or not.
    pub fn frame_view(&self) -> &wgpu::TextureView {
        self.targets.as_ref().map_or(&self.off, |t| &t.lit)
    }

    /// What every other camera's frame groups bind there: one texel of
    /// nothing.
    pub fn off_view(&self) -> &wgpu::TextureView {
        &self.off
    }

    /// Makes the targets of a frame of `size` with `s`, before the frame
    /// groups that bind them are made; true when they are new (the
    /// groups must be made again).
    pub fn ensure_targets(
        &mut self,
        device: &wgpu::Device,
        s: &ScreenSpaceReflections,
        size: (u32, u32),
    ) -> bool {
        let divisor = s.divisor();
        if self
            .targets
            .as_ref()
            .is_some_and(|t| t.size == size && t.divisor == divisor)
        {
            return false;
        }
        self.targets = Some(Targets::new(device, size, divisor));
        self.history = None;
        self.generation += 1;
        true
    }

    /// The accumulation a frame hands the next, copied for a presentation
    /// checkpoint (`renderer/history.rs`); `None` without targets.
    pub(crate) fn save(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Option<SsrSave> {
        let t = self.targets.as_ref()?;
        Some(SsrSave {
            size: t.size,
            divisor: t.divisor,
            copy: GpuCopy::of(device, encoder, &t.history_resources(), &[]),
            history: self.history,
        })
    }

    /// Brings back [`SsrRenderer::save`]'s accumulation into the same
    /// targets; true when it came back or neither had one (the history
    /// starts afresh otherwise).
    pub(crate) fn restore(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        saved: Option<&SsrSave>,
    ) -> bool {
        self.history = None;
        match (saved, &self.targets) {
            (None, _) => true,
            (Some(s), Some(t)) if s.size == t.size && s.divisor == t.divisor => {
                let back = s.copy.back(encoder, &t.history_resources(), &[]);
                if back {
                    self.history = s.history;
                }
                back
            }
            _ => false,
        }
    }

    /// Forgets the history (a resize, a new scene, a readback).
    pub fn reset(&mut self) {
        self.history = None;
    }

    /// The layout entry of the lit shaders' read (the frame group's
    /// `ssr_tex`, frame.wgsl).
    pub fn lit_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::ReadOnly,
                format: FORMAT,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None,
        }
    }

    /// Writes the uniform of a frame of `size` with `s` (its targets made
    /// by [`Self::ensure_targets`]); `prev_lit`: the previous frame's lit
    /// image is there to read.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        s: &ScreenSpaceReflections,
        size: (u32, u32),
        camera: &SsrCamera,
        frame: u64,
        prev_lit: bool,
    ) {
        if self.pipelines.is_none() {
            self.pipelines = Some(Pipelines::new(device));
        }
        let t = self.targets.as_ref().expect("ensure_targets");
        // The history is the previous frame's, or none.
        let previous = self.history.filter(|(f, _)| f + 1 == frame);
        let proj = camera.view_proj * camera.view.inverse();
        let u = SsrUniform {
            proj: proj.to_cols_array_2d(),
            inv_proj: proj.inverse().to_cols_array_2d(),
            view: camera.view.to_cols_array_2d(),
            inv_view: camera.view.inverse().to_cols_array_2d(),
            prev_view_proj: previous
                .map_or(camera.view_proj, |(_, m)| m)
                .to_cols_array_2d(),
            screen: [
                size.0 as f32,
                size.1 as f32,
                1.0 / size.0 as f32,
                1.0 / size.1 as f32,
            ],
            trace: [
                s.max_distance.max(0.01),
                s.thickness.max(0.001),
                s.steps.clamp(4, 128) as f32,
                s.max_roughness.clamp(0.0, 1.0),
            ],
            params: [
                s.fade.clamp(0.0, 0.5),
                camera.near,
                camera.far,
                (frame % 256) as f32,
            ],
            flags: [
                prev_lit as u32 as f32,
                previous.is_some() as u32 as f32,
                t.divisor as f32,
                // The march's levels: the view depth and the pyramid's.
                (PYRAMID_LEVELS + 1) as f32,
            ],
            ray: {
                let [x, y] = block_offset(frame);
                [t.ray_size.0 as f32, t.ray_size.1 as f32, x, y]
            },
            chain: [
                t.ray_size.0 as f32,
                t.ray_size.1 as f32,
                1.0 / t.ray_size.0 as f32,
                1.0 / t.ray_size.1 as f32,
            ],
        };
        queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&u));
        self.history = Some((frame, camera.view_proj));
    }

    /// Records the passes of `frame` (after [`SsrRenderer::prepare`]):
    /// `depth` and `gbuffer` the prepass's, `prev_lit` the previous frame's
    /// lit image (a placeholder when there is none).
    pub fn encode(
        &self,
        device: &wgpu::Device,
        encoder: &mut crate::Encoder,
        depth: &wgpu::TextureView,
        gbuffer: &wgpu::TextureView,
        prev_lit: &wgpu::TextureView,
        frame: u64,
    ) {
        let (Some(p), Some(t)) = (&self.pipelines, &self.targets) else {
            return;
        };
        let tex = wgpu::BindingResource::TextureView;
        let common = crate::volumetric::bind_at(
            device,
            "ssr",
            &p.common,
            &[
                (0, self.uniform.as_entire_binding()),
                (1, tex(depth)),
                (2, tex(gbuffer)),
                (3, wgpu::BindingResource::Sampler(&p.sampler)),
            ],
        );
        let slot = (frame % 2) as usize;
        let group = |pass: usize, resources: &[(u32, wgpu::BindingResource)]| {
            crate::volumetric::bind_at(device, "ssr pass", &p.layouts[pass], resources)
        };
        // The four levels a pyramid dispatch writes from `first` (ssr.wgsl
        // hiz_out0 to hiz_out3).
        let outs = |first: usize| -> [(u32, wgpu::BindingResource); 4] {
            let at = [0u32, 19, 20, 21];
            std::array::from_fn(|k| (at[k], tex(&t.hiz_levels[first + k])))
        };
        let base = group(HIZ_BASE, &outs(0));
        let mut reduce_entries = outs(4).to_vec();
        reduce_entries.push((1, tex(&t.hiz_levels[3])));
        let reduce = group(HIZ_REDUCE, &reduce_entries);
        let trace = group(
            TRACE,
            &[
                (2, tex(&t.hiz)),
                (3, tex(prev_lit)),
                (4, tex(&t.ray_radiance)),
                (5, tex(&t.ray_hit)),
                (22, tex(&t.guide)),
            ],
        );
        let resolve = group(
            RESOLVE,
            &[
                (6, tex(&t.ray_radiance)),
                (7, tex(&t.ray_hit)),
                (8, tex(&t.resolved)),
                (23, tex(&t.guide)),
            ],
        );
        let prefilter = group(
            PREFILTER,
            &[
                (9, tex(&t.resolved)),
                (15, tex(&t.filtered)),
                (23, tex(&t.guide)),
            ],
        );
        // At half resolution the wide pass filters the prefilter's output
        // into the ray radiance, which nothing reads after the resolve.
        let wide = t.divisor > 1;
        let prefilter_wide = group(
            PREFILTER_WIDE,
            &[
                (9, tex(&t.filtered)),
                (15, tex(&t.ray_radiance)),
                (23, tex(&t.guide)),
            ],
        );
        let temporal = group(
            TEMPORAL,
            &[
                (7, tex(&t.ray_hit)),
                (9, tex(if wide { &t.ray_radiance } else { &t.filtered })),
                (16, tex(&t.resolved)),
                (10, tex(&t.history[1 - slot])),
                (11, tex(&t.meta[1 - slot])),
                (12, tex(&t.history[slot])),
                (13, tex(&t.meta[slot])),
                (17, tex(&t.spread[1 - slot])),
                (18, tex(&t.spread[slot])),
                (14, tex(&t.fast[slot])),
                (24, tex(&t.fast[1 - slot])),
            ],
        );
        let upsample = group(
            UPSAMPLE,
            &[
                (10, tex(&t.history[slot])),
                (23, tex(&t.guide)),
                (25, tex(t.upsampled.as_ref().map_or(&t.lit, |u| &u.0))),
            ],
        );
        let groups = |(w, h): (u32, u32)| (w.div_ceil(8), h.div_ceil(8));
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("ssr"),
            timestamp_writes: None,
        });
        pass.set_bind_group(0, &common, &[]);
        let mut dispatch = |pipeline: usize, group: &wgpu::BindGroup, size: (u32, u32)| {
            pass.set_pipeline(&p.passes[pipeline]);
            pass.set_bind_group(1, group, &[]);
            let (x, y) = groups(size);
            pass.dispatch_workgroups(x, y, 1);
        };
        // A group of 8x8 writes 8x8 texels of its first level.
        let (w, h) = pyramid_size(t.size.0, t.size.1);
        dispatch(HIZ_BASE, &base, (w, h));
        dispatch(HIZ_REDUCE, &reduce, (w >> 4, h >> 4));
        dispatch(TRACE, &trace, t.ray_size);
        dispatch(RESOLVE, &resolve, t.ray_size);
        dispatch(PREFILTER, &prefilter, t.ray_size);
        if wide {
            dispatch(PREFILTER_WIDE, &prefilter_wide, t.ray_size);
        }
        dispatch(TEMPORAL, &temporal, t.ray_size);
        if t.upsampled.is_some() {
            dispatch(UPSAMPLE, &upsample, t.size);
        }
        drop(pass);
        // The lit shaders read a view that is the same every frame, and
        // only ever written by a copy: they read it as a read-only storage
        // texture, and a compute pass writing it as a write-only one left
        // D3D12 with no barrier either way (both are its UNORDERED_ACCESS
        // state, and wgpu-hal's dx12 backend adds a UAV barrier only after
        // read-write storage), so the main pass raced the upsample.
        let source = t
            .upsampled
            .as_ref()
            .map_or(&t.history_textures[slot], |u| &u.1);
        encoder.copy_texture_to_texture(
            source.as_image_copy(),
            t.lit_texture.as_image_copy(),
            source.size(),
        );
    }
}

impl Pipelines {
    fn new(device: &wgpu::Device) -> Pipelines {
        let compute = wgpu::ShaderStages::COMPUTE;
        let texture = |binding, filterable| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: compute,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let storage = |binding, format| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: compute,
            ty: wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::WriteOnly,
                format,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None,
        };
        let layout = |label, entries: &[wgpu::BindGroupLayoutEntry]| {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(label),
                entries,
            })
        };
        let mut uniform = uniform_entry(0, compute);
        uniform.visibility = compute;
        let mut sampler = sampler_entry(3, wgpu::SamplerBindingType::Filtering);
        sampler.visibility = compute;
        let common = layout(
            "ssr",
            &[uniform, texture(1, false), texture(2, false), sampler],
        );
        let rg32 = wgpu::TextureFormat::Rg32Float;
        let layouts = [
            layout(
                "ssr hiz base",
                &[
                    storage(0, rg32),
                    storage(19, rg32),
                    storage(20, rg32),
                    storage(21, rg32),
                ],
            ),
            layout(
                "ssr hiz reduce",
                &[
                    storage(0, rg32),
                    storage(19, rg32),
                    storage(20, rg32),
                    storage(21, rg32),
                    texture(1, false),
                ],
            ),
            layout(
                "ssr trace",
                &[
                    texture(2, false),
                    texture(3, true),
                    storage(4, FORMAT),
                    storage(5, FORMAT),
                    storage(22, FORMAT),
                ],
            ),
            layout(
                "ssr resolve",
                &[
                    texture(6, false),
                    texture(7, false),
                    storage(8, FORMAT),
                    texture(23, false),
                ],
            ),
            layout(
                "ssr prefilter",
                &[texture(9, true), storage(15, FORMAT), texture(23, false)],
            ),
            layout(
                "ssr temporal",
                &[
                    texture(7, false),
                    texture(9, false),
                    texture(16, false),
                    texture(10, true),
                    texture(11, false),
                    storage(12, FORMAT),
                    storage(13, FORMAT),
                    storage(14, FORMAT),
                    texture(17, false),
                    storage(18, FORMAT),
                    texture(24, false),
                ],
            ),
            layout(
                "ssr upsample",
                &[texture(10, true), texture(23, false), storage(25, FORMAT)],
            ),
            layout(
                "ssr prefilter wide",
                &[texture(9, true), storage(15, FORMAT), texture(23, false)],
            ),
        ];
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ssr"),
            source: wgpu::ShaderSource::Wgsl(SSR_WGSL.into()),
        });
        let entries = [
            "hiz_base",
            "hiz_reduce",
            "trace",
            "resolve",
            "prefilter",
            "temporal",
            "upsample",
            "prefilter_wide",
        ];
        let passes = std::array::from_fn(|i| {
            let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(entries[i]),
                bind_group_layouts: &[Some(&common), Some(&layouts[i])],
                immediate_size: 0,
            });
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entries[i]),
                layout: Some(&pl),
                module: &module,
                entry_point: Some(entries[i]),
                // Every pass writes its shared memory before it reads it
                // (behind a barrier), so the zeroing WebGPU asks for is a
                // second barrier and a store of every byte for nothing (the
                // browsers zero it regardless).
                compilation_options: wgpu::PipelineCompilationOptions {
                    zero_initialize_workgroup_memory: false,
                    ..Default::default()
                },
                cache: None,
            })
        });
        Pipelines {
            common,
            layouts,
            passes,
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("ssr"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What the prepass writes into its view depth's second channel for a
    /// surface of `roughness` and `metalness` (pbr_prepass.wgsl packs the
    /// same; `ssr.wgsl`'s `ssr_roughness` unpacks it).
    fn pack(roughness: f32, metalness: f32) -> f32 {
        let gloss = ((1.0 - roughness.clamp(0.0, 1.0)) * 255.0).round();
        gloss + 256.0 * (metalness.clamp(0.0, 1.0) * 255.0).round()
    }

    /// ssr.wgsl's `ssr_roughness`: the low 16 bits (the painterly detail
    /// byte lies above them).
    fn roughness(packed: f32) -> f32 {
        let g = packed - (packed / 65536.0).floor() * 65536.0;
        let metal = (g / 256.0).floor();
        1.0 - (g - metal * 256.0) / 255.0
    }

    /// The packed roughness survives the round trip through an f32 for
    /// every 255th, with every metalness and painterly detail byte above
    /// it, and a surface that wrote nothing (an unlit material) unpacks as
    /// the roughest, which is never traced.
    #[test]
    fn roughness_round_trips() {
        for r in 0..=255 {
            for m in [0, 1, 128, 254, 255] {
                let (rough, metal) = (r as f32 / 255.0, m as f32 / 255.0);
                for detail in [0.0, 1.0, 127.0, 255.0] {
                    let back = roughness(pack(rough, metal) + 65536.0 * detail);
                    assert!((back - rough).abs() < 1e-6, "{r} {m} {detail}");
                }
            }
        }
        assert_eq!(roughness(0.0), 1.0);
    }

    /// The pyramid covers the screen and halves exactly at every level, so
    /// a coarse texel bounds every pixel under it with no row left out.
    #[test]
    fn pyramid_halves_exactly_over_the_screen() {
        for (w, h) in [(1, 1), (1920, 1080), (1280, 720), (2560, 1440), (257, 3)] {
            let (pw, ph) = pyramid_size(w, h);
            assert!(pw * 2 >= w && ph * 2 >= h, "{w}x{h}");
            for k in 0..PYRAMID_LEVELS {
                assert_eq!((pw >> k) << k, pw, "{w}x{h} level {}", k + 1);
                assert_eq!((ph >> k) << k, ph, "{w}x{h} level {}", k + 1);
            }
        }
        assert_eq!(pyramid_size(1920, 1080), (1024, 640));
    }

    /// A half-resolution ray visits every pixel of its block in four frames.
    #[test]
    fn half_resolution_covers_the_block() {
        let mut seen: Vec<[u32; 2]> = (0..4).map(|f| block_offset(f).map(|v| v as u32)).collect();
        seen.sort();
        assert_eq!(seen, [[0, 0], [0, 1], [1, 0], [1, 1]]);
    }
}
