//! Volumetric fog (docs/design.md §2.6): fog lit as a participating medium.
//!
//! [`crate::RenderScene::volumetric_fog`] replaces the frame's plain fog
//! (`RenderScene::fog`) while it is set, for a perspective camera above
//! water. The medium (volumetric.wgsl): a density per metre plus height
//! fog, scattering its colour times `scattering` of the light it is lit by
//! with a Henyey-Greenstein phase, out to `distance` metres of view depth.
//!
//! - [`VolumetricMethod::Froxel`] (the default): the graph pass `volumetric`
//!   (two compute dispatches after the shadow passes) lights a 160 × 90 ×
//!   64 froxel grid over the view by the sun through its cascades and by
//!   the clustered point, spot and rect lights through their shadows and
//!   cookies, then sums it front to back. Each froxel's light is the mean
//!   of eight stratified samples over its slice's depth, jittered per
//!   froxel and per frame: a slice is metres deep far out, and a single
//!   sample there landed in or out of a shaft column by column (vertical
//!   streaks).
//! - [`VolumetricMethod::Raymarch`] (asked for with
//!   `RenderDesc::volumetric`): no compute pass; the apply pass marches
//!   each pixel's view ray in 24 jittered steps, lit by the sun through its
//!   cascades and by the first [`RAYMARCH_LIGHTS`] point, spot and rect
//!   lights of each step's cluster through their shadows and cookies. It
//!   has no froxel grid to fill and no slice blur, at the cost of fewer
//!   lights per sample and a per-pixel loop.
//!
//! Either way the last thing the `main` pass does is blend the result over
//! the lit image (colour × transmittance + in-scattered light) at each
//! pixel's prepass depth, so opaque surfaces, the sky (at `distance`) and
//! whatever was blended in front of them are all seen through the volume.
//!
//! Under water the same pass draws the water the eye is in
//! ([`Medium::Water`]) instead: the sun's light in it, which the waves
//! focus and whatever stands in the water shadows (volumetric.wgsl
//! `water_in`), reaches the eye as light shafts.

use glam::Vec3;

/// Fog lit as a participating medium ([`crate::RenderScene::volumetric_fog`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VolumetricFog {
    /// The medium's colour (its albedo's tint).
    pub color: Vec3,
    /// Extinction per metre everywhere.
    pub density: f32,
    /// The height below which the height fog is densest.
    pub height: f32,
    /// Extra extinction per metre at and below `height`.
    pub height_density: f32,
    /// How fast the height fog thins above `height`, per metre.
    pub falloff: f32,
    /// The share of the light a scattering event sends on (0 dims only).
    pub scattering: f32,
    /// Henyey-Greenstein g: towards 1 forward (shafts against the light).
    pub anisotropy: f32,
    /// How far the volume reaches, in view depth.
    pub distance: f32,
}

impl Default for VolumetricFog {
    fn default() -> Self {
        VolumetricFog {
            color: Vec3::ONE,
            density: 0.02,
            height: 0.0,
            height_density: 0.0,
            falloff: 0.5,
            scattering: 0.8,
            anisotropy: 0.6,
            distance: 48.0,
        }
    }
}

impl VolumetricFog {
    /// No fog of the scene's own, lit out to `distance`: the medium the
    /// frame's [`FogVolume`]s are drawn in where the scene has no
    /// volumetric fog.
    pub(crate) fn empty(distance: f32) -> VolumetricFog {
        VolumetricFog {
            density: 0.0,
            height_density: 0.0,
            distance,
            ..Default::default()
        }
    }
}

/// Fog inside a region (og.FogVolume): its extinction and colour added to
/// the frame's volumetric medium where its shape reaches, lit as that
/// medium is (the sun through its cascades, the local lights through
/// their shadows).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FogVolume {
    /// Where its entity puts it (frame coordinates).
    pub at: og_env::volume::Placement,
    /// Its shape; `None`: no bounds, the fog is everywhere.
    pub volume: Option<og_env::volume::Volume>,
    /// The fog's colour (its albedo's tint).
    pub color: Vec3,
    /// Extinction per metre inside the shape.
    pub density: f32,
    /// The share of the light a scattering event sends on.
    pub scattering: f32,
}

/// The most [`FogVolume`]s one frame draws: the nearest the camera.
pub const FOG_VOLUMES: usize = 16;

impl FogVolume {
    /// A radius about the entity that holds all of it, fade included
    /// (infinite without bounds).
    pub(crate) fn reach(&self) -> f32 {
        let scale = self.at.scale.abs();
        self.volume.map_or(f32::INFINITY, |v| {
            let body = if v.radius > 0.0 {
                v.radius * scale.max_element()
            } else {
                (v.size.abs() * scale * 0.5).length()
            };
            body + v.fade.max(0.0)
        })
    }

    /// Its record in the volume's uniform (volumetric.wgsl `FogVolume`):
    /// the shape as `og_env::volume::Volume::outside` reads it, so the
    /// shader's weight is that function's.
    fn record(&self) -> [[f32; 4]; 4] {
        let scale = self.at.scale.abs();
        let (radius, half, fade) = match self.volume {
            None => (-1.0, Vec3::ZERO, 0.0),
            Some(v) if v.radius > 0.0 => (v.radius * scale.max_element(), Vec3::ZERO, v.fade),
            Some(v) => (0.0, v.size.abs() * scale * 0.5, v.fade),
        };
        [
            self.at.position.extend(radius).to_array(),
            self.at.quaternion.normalize().inverse().to_array(),
            half.extend(fade.max(0.0)).to_array(),
            (self.color.max(Vec3::ZERO) * self.scattering.max(0.0))
                .extend(self.density.max(0.0))
                .to_array(),
        ]
    }
}

/// The volumes of `all` a frame draws from `eye` out to `distance`: those
/// that reach into it, the nearest [`FOG_VOLUMES`], and how many more did.
pub(crate) fn nearest_volumes(
    all: &[FogVolume],
    eye: Vec3,
    distance: f32,
) -> (Vec<FogVolume>, usize) {
    let mut near: Vec<(f32, &FogVolume)> = all
        .iter()
        .filter(|v| v.density > 0.0)
        .map(|v| (((v.at.position - eye).length() - v.reach()).max(0.0), v))
        .filter(|(d, _)| *d < distance)
        .collect();
    near.sort_by(|a, b| a.0.total_cmp(&b.0));
    let dropped = near.len().saturating_sub(FOG_VOLUMES);
    (
        near.into_iter()
            .take(FOG_VOLUMES)
            .map(|(_, v)| *v)
            .collect(),
        dropped,
    )
}

/// How far the volume must reach from `eye` to hold every one of `all`
/// (a scene with fog volumes and no volumetric fog of its own): to the far
/// side of the farthest, within what [`VolumetricFog::distance`] allows.
pub(crate) fn volumes_distance(all: &[FogVolume], eye: Vec3) -> f32 {
    all.iter()
        .map(|v| match v.reach() {
            r if r.is_finite() => (v.at.position - eye).length() + r,
            _ => VolumetricFog::default().distance,
        })
        .fold(4.0, f32::max)
        .min(400.0)
}

/// How a renderer draws volumetric fog, on every backend (WebGPU, Metal,
/// Vulkan, DX12).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VolumetricMethod {
    /// A froxel grid lit and integrated in compute (the default).
    Froxel,
    /// A per-pixel raymarch in the apply pass's fragment shader.
    Raymarch,
}

/// The froxel grid: across, down, deep (the depth at
/// [`crate::RenderQuality::volumetric_step_scale`] 1).
pub const FROXELS: [u32; 3] = [160, 90, 64];
/// The raymarch's steps per pixel (at
/// [`crate::RenderQuality::volumetric_step_scale`] 1).
pub const RAYMARCH_STEPS: u32 = 24;
/// The most point, spot and rect lights of a step's cluster that light the
/// raymarch's fog (the first of the cluster's, which keeps the brightest
/// at the camera first): each step evaluates them with their shadows and
/// cookies, so 24 steps × 8 lights bound the fragment's work.
pub const RAYMARCH_LIGHTS: u32 = 8;

/// The lights one sample of the fog is lit by, by method
/// ([`crate::RenderInfo::volumetric_lights`]).
pub(crate) fn lights(method: VolumetricMethod) -> u32 {
    match method {
        VolumetricMethod::Froxel => crate::MAX_CLUSTER_LIGHTS as u32,
        VolumetricMethod::Raymarch => RAYMARCH_LIGHTS,
    }
}

/// What the volumetric pass draws in a frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Medium {
    /// The scene's fog, above water.
    Fog(VolumetricFog),
    /// The water the eye is under, of `density` extinction per metre (its
    /// fog's: `water::underwater_fog`). The frame's fog draws the water's
    /// even light and dims what is seen through it; the volume adds the
    /// sun's own share of that light (`water::underwater_light`), lit
    /// through the waves and the sun's shadows.
    Water { density: f32 },
}

/// Henyey-Greenstein g of the sun's light in water as the volume scatters
/// it. Sea water's single scattering is far more forward than this (g
/// about 0.9, Petzold's measurements in Mobley, Light and Water, 1994,
/// ch. 3), but the share the volume draws stands for the sun's light
/// after every order of scattering, each of which spreads it.
const WATER_ANISOTROPY: f32 = 0.35;
/// How far the water's volume reaches, in the water's extinction lengths
/// (what lies past it reaches the eye at under 5 %).
const WATER_REACH: f32 = 3.0;

impl Medium {
    /// The pipeline constants of the fog's pipelines (`water` false) or
    /// the water's: `VOL_WATER` in volumetric.wgsl.
    fn constants(water: bool) -> &'static [(&'static str, f64)] {
        if water {
            &[("VOL_WATER", 1.0)]
        } else {
            &[("VOL_WATER", 0.0)]
        }
    }

    fn uniform(&self) -> ([f32; 4], [f32; 4], f32) {
        match *self {
            Medium::Fog(fog) => (
                (fog.color.max(Vec3::ZERO) * fog.scattering.max(0.0))
                    .extend(fog.anisotropy.clamp(-0.95, 0.95))
                    .to_array(),
                [
                    fog.density.max(0.0),
                    fog.height,
                    fog.height_density.max(0.0),
                    fog.falloff.max(1e-4),
                ],
                fog.distance.max(1.0),
            ),
            Medium::Water { density } => (
                [1.0, 1.0, 1.0, WATER_ANISOTROPY],
                [0.0; 4],
                (WATER_REACH / density.max(1e-3)).clamp(4.0, 96.0),
            ),
        }
    }
}

const FROXEL_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct VolUniform {
    scatter: [f32; 4],
    medium: [f32; 4],
    extent: [f32; 4],
    grid: [f32; 4],
    /// x: how many of `volumes` are set.
    local: [f32; 4],
    volumes: [[[f32; 4]; 4]; FOG_VOLUMES],
}

struct Froxels {
    frame_layout: wgpu::BindGroupLayout,
    inject_layout: wgpu::BindGroupLayout,
    integrate_layout: wgpu::BindGroupLayout,
    inject_pipeline: wgpu::PipelineLayout,
    /// The inject pipeline of the fog and of the water, each made on the
    /// first frame with its medium.
    inject: [Option<wgpu::ComputePipeline>; 2],
    integrate: wgpu::ComputePipeline,
    scatter: wgpu::TextureView,
    sum: wgpu::TextureView,
    /// The volumes' slices deep ([`crate::RenderQuality::froxel_slices`]).
    slices: u32,
    sampler: wgpu::Sampler,
}

/// The apply pipeline (`froxel`: the grid's, else the raymarch's) of the
/// medium whose `constants` ([`Medium::constants`]) it is made with.
fn apply_pipeline(
    device: &wgpu::Device,
    module: &wgpu::ShaderModule,
    layout: &wgpu::PipelineLayout,
    froxel: bool,
    constants: &[(&str, f64)],
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("volumetric apply"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module,
            entry_point: Some("vs_volumetric"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module,
            entry_point: Some(if froxel { "fs_froxel" } else { "fs_raymarch" }),
            compilation_options: wgpu::PipelineCompilationOptions {
                constants,
                ..Default::default()
            },
            targets: &[Some(wgpu::ColorTargetState {
                format: crate::HDR_FORMAT,
                // colour × transmittance + in-scattered light.
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::SrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::Zero,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: crate::pipeline_cache::of(device).as_ref(),
    })
}

/// A froxel compute pipeline at `entry_point` of the module.
fn compute_pipeline(
    device: &wgpu::Device,
    module: &wgpu::ShaderModule,
    layout: &wgpu::PipelineLayout,
    label: &str,
    entry_point: &str,
    constants: &[(&str, f64)],
) -> wgpu::ComputePipeline {
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        module,
        entry_point: Some(entry_point),
        compilation_options: wgpu::PipelineCompilationOptions {
            constants,
            ..Default::default()
        },
        cache: crate::pipeline_cache::of(device).as_ref(),
    })
}

impl Froxels {
    /// The grid's layouts, volumes and the integrate pipeline (the inject
    /// pipelines are each medium's: [`Volumetric::ensure`]).
    fn new(device: &wgpu::Device, module: &wgpu::ShaderModule) -> Froxels {
        let compute = wgpu::ShaderStages::COMPUTE;
        let table_ty = wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        };
        let frame_layout_c = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("volumetric frame"),
            entries: &[
                entry(0, compute, uniform_ty()),
                entry(
                    1,
                    compute,
                    texture_ty(wgpu::TextureViewDimension::D2Array, false),
                ),
                entry(
                    2,
                    compute,
                    texture_ty(wgpu::TextureViewDimension::D2Array, false),
                ),
                entry(7, compute, table_ty),
                entry(8, compute, table_ty),
                entry(9, compute, table_ty),
                // The water's light (caustics.wgsl `water_focus`).
                entry(12, compute, uniform_ty()),
                // The clouds' shadow (lighting.wgsl `cloud_shadow`).
                crate::clouds::CloudShadow::layout_entry(14, compute),
            ],
        });
        let inject_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("volumetric inject"),
            entries: &[entry(0, compute, uniform_ty()), storage_3d(1)],
        });
        let integrate_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("volumetric integrate"),
            entries: &[
                entry(0, compute, uniform_ty()),
                entry(
                    2,
                    compute,
                    texture_ty(wgpu::TextureViewDimension::D3, false),
                ),
                storage_3d(3),
            ],
        });
        let layout = |label, group: &wgpu::BindGroupLayout| {
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &[Some(&frame_layout_c), Some(group)],
                immediate_size: 0,
            })
        };
        let inject_pipeline = layout("volumetric inject", &inject_layout);
        // The integrate pass reads no medium.
        let integrate = compute_pipeline(
            device,
            module,
            &layout("volumetric integrate", &integrate_layout),
            "volumetric integrate",
            "integrate",
            &[],
        );
        Froxels {
            frame_layout: frame_layout_c,
            inject_layout,
            integrate_layout,
            inject_pipeline,
            inject: [None, None],
            integrate,
            scatter: froxel_volume(device, "froxel scatter", FROXELS[2]),
            sum: froxel_volume(device, "froxel sum", FROXELS[2]),
            slices: FROXELS[2],
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("froxels"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
        }
    }
}

/// A froxel volume `slices` deep.
fn froxel_volume(device: &wgpu::Device, label: &str, slices: u32) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: FROXELS[0],
                height: FROXELS[1],
                depth_or_array_layers: slices,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: FROXEL_FORMAT,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
        .create_view(&Default::default())
}

/// The renderer's volumetric fog: made on the first frame that has it.
pub(crate) struct Volumetric {
    pub method: VolumetricMethod,
    uniform: wgpu::Buffer,
    froxels: Option<Froxels>,
    apply_layout: wgpu::BindGroupLayout,
    /// The module and the apply pipeline's layout, made on first use.
    module: Option<(wgpu::ShaderModule, wgpu::PipelineLayout)>,
    /// The apply pipeline of the fog and of the water, each made on the
    /// first frame with its medium.
    apply: [Option<wgpu::RenderPipeline>; 2],
    /// Whether the frame's medium is the water.
    water: bool,
    /// The lit features the renderer is made without (pipeline_features.rs).
    off: crate::pipeline_features::Features,
    /// Whether it has said that more fog volumes reach the view than it draws.
    warned_volumes: bool,
}

/// A bind group of `(binding, resource)` pairs (layouts whose bindings
/// skip numbers).
pub(crate) fn bind_at(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::BindGroupLayout,
    resources: &[(u32, wgpu::BindingResource)],
) -> wgpu::BindGroup {
    let entries: Vec<_> = resources
        .iter()
        .map(|(binding, r)| wgpu::BindGroupEntry {
            binding: *binding,
            resource: r.clone(),
        })
        .collect();
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &entries,
    })
}

fn storage_3d(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::WriteOnly,
            format: FROXEL_FORMAT,
            view_dimension: wgpu::TextureViewDimension::D3,
        },
        count: None,
    }
}

fn entry(
    binding: u32,
    visibility: wgpu::ShaderStages,
    ty: wgpu::BindingType,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty,
        count: None,
    }
}

fn uniform_ty() -> wgpu::BindingType {
    wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
    }
}

fn texture_ty(dim: wgpu::TextureViewDimension, filterable: bool) -> wgpu::BindingType {
    wgpu::BindingType::Texture {
        sample_type: wgpu::TextureSampleType::Float { filterable },
        view_dimension: dim,
        multisampled: false,
    }
}

/// The module's source for a method (`froxel`: the froxel entry points,
/// else the raymarch's).
pub(crate) fn source(froxel: bool) -> String {
    format!(
        "{}{}{}{}{}{}",
        crate::FRAME_WGSL,
        crate::clusters::wgsl(),
        crate::LIGHTING_WGSL,
        crate::AREA_WGSL,
        wgsl3d!("volumetric.wgsl"),
        if froxel {
            wgsl3d!("volumetric_froxel.wgsl").to_string()
        } else {
            format!(
                "const RAYMARCH_LIGHTS: u32 = {RAYMARCH_LIGHTS}u;\n{}",
                wgsl3d!("volumetric_raymarch.wgsl")
            )
        }
    )
}

impl Volumetric {
    /// GPU bytes of the froxel volumes, when made.
    pub(crate) fn bytes(&self) -> u64 {
        self.froxels.as_ref().map_or(0, |f| {
            crate::gpu::texture::view_bytes(&f.scatter) + crate::gpu::texture::view_bytes(&f.sum)
        })
    }
}

impl Volumetric {
    /// `off`: the lit features the renderer is made without
    /// (pipeline_features.rs), out of its module.
    pub fn new(
        device: &wgpu::Device,
        method: VolumetricMethod,
        off: crate::pipeline_features::Features,
    ) -> Volumetric {
        let froxel = method == VolumetricMethod::Froxel;
        let fragment = wgpu::ShaderStages::FRAGMENT;
        let mut entries = vec![
            entry(0, fragment | wgpu::ShaderStages::COMPUTE, uniform_ty()),
            entry(
                4,
                fragment,
                texture_ty(wgpu::TextureViewDimension::D2, false),
            ),
        ];
        if froxel {
            entries.push(entry(
                5,
                fragment,
                texture_ty(wgpu::TextureViewDimension::D3, true),
            ));
            entries.push(entry(
                6,
                fragment,
                wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            ));
        }
        let apply_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("volumetric apply"),
            entries: &entries,
        });
        Volumetric {
            method,
            uniform: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("volumetric"),
                size: std::mem::size_of::<VolUniform>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            froxels: None,
            apply_layout,
            module: None,
            apply: [None, None],
            water: false,
            off,
            warned_volumes: false,
        }
    }

    /// Makes the module, the froxel volumes and the pipelines of the
    /// medium (`water`) on first use; `frame_layout`: the lit frame group's.
    fn ensure(&mut self, device: &wgpu::Device, frame_layout: &wgpu::BindGroupLayout, water: bool) {
        let froxel = self.method == VolumetricMethod::Froxel;
        if self.module.is_none() {
            let label = if froxel {
                "volumetric"
            } else {
                "volumetric raymarch"
            };
            let module = crate::unbounded_shader(device, label, self.off.leave_out(source(froxel)));
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("volumetric apply"),
                bind_group_layouts: &[Some(frame_layout), Some(&self.apply_layout)],
                immediate_size: 0,
            });
            if froxel {
                self.froxels = Some(Froxels::new(device, &module));
            }
            self.module = Some((module, layout));
        }
        let Some((module, layout)) = &self.module else {
            return;
        };
        let constants = Medium::constants(water);
        if self.apply[water as usize].is_none() {
            self.apply[water as usize] =
                Some(apply_pipeline(device, module, layout, froxel, constants));
        }
        if let Some(f) = &mut self.froxels
            && f.inject[water as usize].is_none()
        {
            f.inject[water as usize] = Some(compute_pipeline(
                device,
                module,
                &f.inject_pipeline,
                "volumetric inject",
                "inject",
                constants,
            ));
        }
    }

    /// Writes the frame's medium, with the fog volumes of `local` nearest
    /// `eye` in the scene's fog; `size`: the HDR target's. Another depth
    /// of the froxel grid at `quality` makes its volumes again.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame_layout: &wgpu::BindGroupLayout,
        medium: &Medium,
        (local, eye): (&[FogVolume], Vec3),
        size: (u32, u32),
        tick: u64,
        quality: &crate::RenderQuality,
    ) {
        self.water = matches!(medium, Medium::Water { .. });
        self.ensure(device, frame_layout, self.water);
        let slices = quality.froxel_slices();
        if let Some(f) = &mut self.froxels
            && f.slices != slices
        {
            f.scatter = froxel_volume(device, "froxel scatter", slices);
            f.sum = froxel_volume(device, "froxel sum", slices);
            f.slices = slices;
        }
        let (scatter, fog, distance) = medium.uniform();
        // The water's medium is the water alone.
        let (near, dropped) = match medium {
            Medium::Fog(_) => nearest_volumes(local, eye, distance),
            Medium::Water { .. } => (Vec::new(), 0),
        };
        if dropped > 0 && !self.warned_volumes {
            self.warned_volumes = true;
            log::warn!(
                "og-render: {} og.FogVolume regions reach into the view; the nearest {FOG_VOLUMES} are drawn",
                near.len() + dropped
            );
        }
        let mut volumes = [[[0.0; 4]; 4]; FOG_VOLUMES];
        for (slot, v) in volumes.iter_mut().zip(&near) {
            *slot = v.record();
        }
        let u = VolUniform {
            scatter,
            medium: fog,
            extent: [
                distance,
                quality.raymarch_steps() as f32,
                size.0 as f32,
                size.1 as f32,
            ],
            grid: [
                FROXELS[0] as f32,
                FROXELS[1] as f32,
                slices as f32,
                (tick % 64) as f32,
            ],
            local: [near.len() as f32, 0.0, 0.0, 0.0],
            volumes,
        };
        queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&u));
    }

    /// The froxel dispatches (nothing with the raymarch): after the
    /// shadow passes, whose maps they read.
    #[allow(clippy::too_many_arguments)]
    pub fn encode_froxels(
        &self,
        device: &wgpu::Device,
        encoder: &mut crate::Encoder,
        frame: &wgpu::Buffer,
        cascades: &wgpu::TextureView,
        locals: &wgpu::TextureView,
        tables: [&wgpu::Buffer; 3],
        (caustics, cloud_shadow): (&wgpu::Buffer, &wgpu::TextureView),
    ) {
        let Some(f) = &self.froxels else {
            return;
        };
        let Some(inject_pipeline) = &f.inject[self.water as usize] else {
            return;
        };
        let frame_group = bind_at(
            device,
            "volumetric frame",
            &f.frame_layout,
            &[
                (0, frame.as_entire_binding()),
                (1, wgpu::BindingResource::TextureView(cascades)),
                (2, wgpu::BindingResource::TextureView(locals)),
                (7, tables[0].as_entire_binding()),
                (8, tables[1].as_entire_binding()),
                (9, tables[2].as_entire_binding()),
                (12, caustics.as_entire_binding()),
                (14, wgpu::BindingResource::TextureView(cloud_shadow)),
            ],
        );
        let inject = bind_at(
            device,
            "volumetric inject",
            &f.inject_layout,
            &[
                (0, self.uniform.as_entire_binding()),
                (1, wgpu::BindingResource::TextureView(&f.scatter)),
            ],
        );
        let integrate = bind_at(
            device,
            "volumetric integrate",
            &f.integrate_layout,
            &[
                (0, self.uniform.as_entire_binding()),
                (2, wgpu::BindingResource::TextureView(&f.scatter)),
                (3, wgpu::BindingResource::TextureView(&f.sum)),
            ],
        );
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("volumetric"),
            timestamp_writes: None,
        });
        pass.set_bind_group(0, &frame_group, &[]);
        pass.set_pipeline(inject_pipeline);
        pass.set_bind_group(1, &inject, &[]);
        pass.dispatch_workgroups(FROXELS[0].div_ceil(8), FROXELS[1].div_ceil(8), f.slices);
        pass.set_pipeline(&f.integrate);
        pass.set_bind_group(1, &integrate, &[]);
        pass.dispatch_workgroups(FROXELS[0].div_ceil(8), FROXELS[1].div_ceil(8), 1);
    }

    /// Blends the volume over `hdr` at the depths of `depth` (the prepass's
    /// view depth).
    pub fn encode_apply(
        &self,
        device: &wgpu::Device,
        encoder: &mut crate::Encoder,
        frame_group: &wgpu::BindGroup,
        hdr: &wgpu::TextureView,
        depth: &wgpu::TextureView,
    ) {
        let Some(pipeline) = &self.apply[self.water as usize] else {
            return;
        };
        let mut resources = vec![
            (0, self.uniform.as_entire_binding()),
            (4, wgpu::BindingResource::TextureView(depth)),
        ];
        if let Some(f) = &self.froxels {
            resources.push((5, wgpu::BindingResource::TextureView(&f.sum)));
            resources.push((6, wgpu::BindingResource::Sampler(&f.sampler)));
        }
        let group = bind_at(device, "volumetric apply", &self.apply_layout, &resources);
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("volumetric apply"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: hdr,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, frame_group, &[]);
        pass.set_bind_group(1, &group, &[]);
        pass.draw(0..3, 0..1);
    }
}
