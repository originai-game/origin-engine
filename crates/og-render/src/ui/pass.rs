//! The UI pass on the device: the shape family's pipelines for the output,
//! its sRGB view and the layers' format, the frame's tables (instances,
//! gradients, clip shapes, polyline points), the layers' targets and
//! composite groups, custom materials compiled inside an error scope, the
//! vectors' meshes; and the overlay's draw ([`Renderer::draw_overlay_runs`],
//! [`Renderer::draw_ui_layers`]), which draws the overlay's own runs
//! beside the UI's.

use super::{UiBuiltin, UiMask, UiMaterial, UiMaterialShader, ui_material_module_source};
use crate::deferred::Deferred;
use crate::overlay::{OverlayFrame, OverlayRenderer, OverlayRun, OverlaySource};
use crate::ui::build::{ShapePipe, ShapeSource};
use crate::{RenderScene, Renderer};
use std::collections::HashMap;

/// The layers' offscreen format: linear, half float, premultiplied.
pub(crate) const LAYER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// `struct LayerParams` in ui_material.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct LayerParams {
    rect: [f32; 4],
    params: [[f32; 4]; super::UI_MATERIAL_PARAMS],
    info: [f32; 4],
    mask_uv: [f32; 4],
    /// A projected layer's map from the screen (CSS px) to its uv, rows
    /// (`info.w` 1): the inverse of its rect's map onto its quad.
    quad: [[f32; 4]; 3],
}

/// The shape family's pipelines for one target format.
pub(crate) struct UiPipelines {
    normal: Deferred<wgpu::RenderPipeline>,
    multiply: Deferred<wgpu::RenderPipeline>,
    screen: Deferred<wgpu::RenderPipeline>,
    add: Deferred<wgpu::RenderPipeline>,
    destination: Deferred<wgpu::RenderPipeline>,
    backdrop: Deferred<wgpu::RenderPipeline>,
    vector: Deferred<wgpu::RenderPipeline>,
    composite: Deferred<wgpu::RenderPipeline>,
    /// Mass elements (`shaders/ui_mass.wgsl`).
    mass: Deferred<wgpu::RenderPipeline>,
}

/// Which pipelines a target draws with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum TargetKind {
    Output,
    /// The output's sRGB view.
    Linear,
    Layer,
}

/// A custom material's pipelines, or why it is skipped.
enum Compiled {
    Ready {
        pipelines: Vec<(TargetKind, wgpu::RenderPipeline)>,
        pending: Option<std::pin::Pin<Box<dyn std::future::Future<Output = Option<wgpu::Error>>>>>,
    },
    Failed(String),
}

/// A vector's mesh on the device.
struct VectorMesh {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    count: u32,
}

/// A layer's target, kept across frames by size.
struct LayerTarget {
    size: (u32, u32),
    /// Level 0, what the layer draws into.
    view: wgpu::TextureView,
    /// Every level, what its composite samples.
    sampled: wgpu::TextureView,
    /// The levels a blur reads (`None`: one level), filled from level 0
    /// once the layer is drawn.
    chain: Option<crate::mips::MipChain>,
}

/// Whether a layer's target carries mips: the blur and the drop shadow
/// read them, each of their
/// taps prefiltered to its spacing (ui_builtin.wgsl), so no radius
/// undersamples the layer.
fn layer_mips(material: UiMaterial) -> bool {
    matches!(
        material,
        UiMaterial::Builtin(UiBuiltin::Blur | UiBuiltin::DropShadow)
    )
}

/// What one frame bound: the group 0 of the output and of each layer,
/// each layer's target and composite group.
#[derive(Default)]
pub(crate) struct UiFrameGroups {
    main: Option<wgpu::BindGroup>,
    /// By layer index.
    layers: HashMap<u32, (wgpu::BindGroup, usize, wgpu::BindGroup)>,
    backdrop: Option<wgpu::BindGroup>,
    /// The mass elements' camera, templates, table and instances.
    mass: Option<wgpu::BindGroup>,
}

pub(crate) struct UiRenderer {
    output: UiPipelines,
    linear: Option<UiPipelines>,
    layer: UiPipelines,
    backdrop_layout: wgpu::BindGroupLayout,
    layer_layout: wgpu::BindGroupLayout,
    composite_layout: wgpu::PipelineLayout,
    output_format: wgpu::TextureFormat,
    linear_format: Option<wgpu::TextureFormat>,
    sampler: wgpu::Sampler,
    /// 1×1 white: an unused mask, the backdrop group without glass.
    white: wgpu::TextureView,
    dummy_backdrop: wgpu::BindGroup,
    backdrop_params: wgpu::Buffer,
    instances: wgpu::Buffer,
    gradients: wgpu::Buffer,
    clips: wgpu::Buffer,
    points: wgpu::Buffer,
    uniforms: Vec<wgpu::Buffer>,
    layer_params: Vec<wgpu::Buffer>,
    mass_layout: wgpu::BindGroupLayout,
    mass_camera: wgpu::Buffer,
    /// The glyph table's texture and its rows.
    mass_table: Option<(wgpu::Texture, wgpu::TextureView, u32)>,
    mass_instances: wgpu::Buffer,
    targets: Vec<LayerTarget>,
    custom: HashMap<(u64, usize), (UiMaterialShader, Compiled)>,
    vectors: HashMap<(u64, usize), VectorMesh>,
    pub frame: UiFrameGroups,
}

fn storage_buffer(device: &wgpu::Device, label: &str, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size.max(256),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

/// Writes `bytes` (at least one element's worth: a binding is never empty).
fn write_storage(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    buffer: &mut wgpu::Buffer,
    label: &str,
    bytes: &[u8],
) {
    if bytes.len() as u64 > buffer.size() {
        *buffer = storage_buffer(device, label, (bytes.len() as u64).next_power_of_two());
    }
    if !bytes.is_empty() {
        queue.write_buffer(buffer, 0, bytes);
    }
}

fn uniform_buffer(device: &wgpu::Device, label: &str, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

const VECTOR_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<super::UiVectorVertex>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Uint32x2],
};

/// The shape family's module.
pub(crate) fn ui_source() -> String {
    format!("{}{}", super::UI_COMMON_WGSL, UI_WGSL)
}

/// The mass elements' module.
pub(crate) fn mass_source() -> String {
    format!("{}{}", super::UI_COMMON_WGSL, UI_MASS_WGSL)
}

/// The built-in materials' composite module.
pub(crate) fn builtin_source() -> String {
    format!(
        "{}{}{}",
        super::UI_COMMON_WGSL,
        super::UI_MATERIAL_WGSL,
        UI_BUILTIN_WGSL
    )
}

const UI_WGSL: &str = wgsl!("ui.wgsl");
const UI_BUILTIN_WGSL: &str = wgsl!("ui_builtin.wgsl");
const UI_MASS_WGSL: &str = wgsl!("ui_mass.wgsl");

fn blend_state(pipe: Option<ShapePipe>) -> wgpu::BlendState {
    let keep_alpha = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::Zero,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Add,
    };
    match pipe {
        Some(ShapePipe::Multiply) => wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::Src,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: keep_alpha,
        },
        Some(ShapePipe::Screen) => wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrc,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: keep_alpha,
        },
        Some(ShapePipe::Add) => wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: keep_alpha,
        },
        _ => wgpu::BlendState::ALPHA_BLENDING,
    }
}

fn vertex_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ui mass instances"),
        size: size.max(256),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

/// The mass elements' pipeline: one instance per element from the vertex
/// buffer.
fn mass_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    module: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
) -> Deferred<wgpu::RenderPipeline> {
    let (device, layout, module) = (device.clone(), layout.clone(), module.clone());
    Deferred::new(move || {
        const BUFFERS: &[Option<wgpu::VertexBufferLayout<'static>>] =
            &[Some(super::mass::MASS_LAYOUT)];
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ui mass"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_mass"),
                compilation_options: Default::default(),
                buffers: BUFFERS,
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_mass"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: crate::pipeline_cache::of(&device).as_ref(),
        })
    })
}

#[allow(clippy::too_many_arguments)]
fn pipeline(
    device: &wgpu::Device,
    label: &'static str,
    layout: &wgpu::PipelineLayout,
    module: &wgpu::ShaderModule,
    vs: &'static str,
    fs: &'static str,
    format: wgpu::TextureFormat,
    blend: wgpu::BlendState,
    vector: bool,
) -> Deferred<wgpu::RenderPipeline> {
    let (device, layout, module) = (device.clone(), layout.clone(), module.clone());
    Deferred::new(move || {
        const VECTOR: &[Option<wgpu::VertexBufferLayout<'static>>] = &[Some(VECTOR_LAYOUT)];
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some(vs),
                compilation_options: Default::default(),
                buffers: if vector { VECTOR } else { &[] },
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some(fs),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(blend),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: crate::pipeline_cache::of(&device).as_ref(),
        })
    })
}

impl UiRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        overlay: &OverlayRenderer,
        sprite_layout: &wgpu::BindGroupLayout,
    ) -> UiRenderer {
        let tex = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let sampler_entry = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };
        let uniform_entry = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let backdrop_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ui backdrop"),
            entries: &[tex(0), tex(1), sampler_entry(2), uniform_entry(3)],
        });
        let layer_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ui layer"),
            entries: &[tex(0), sampler_entry(1), uniform_entry(2), tex(3)],
        });
        let shape_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ui shapes"),
            bind_group_layouts: &[
                Some(&overlay.layout),
                Some(sprite_layout),
                Some(&backdrop_layout),
            ],
            immediate_size: 0,
        });
        let composite_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ui composite"),
            bind_group_layouts: &[Some(&overlay.layout), Some(&layer_layout)],
            immediate_size: 0,
        });
        // A uniform and a texture only: group 0 already takes the four
        // storage buffers a stage may have, and a uniform holds 16 KiB.
        let mass_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ui mass"),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let mass_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ui mass"),
            bind_group_layouts: &[
                Some(&overlay.layout),
                Some(sprite_layout),
                Some(&mass_layout),
            ],
            immediate_size: 0,
        });
        let mass_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ui mass"),
            source: wgpu::ShaderSource::Wgsl(mass_source().into()),
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ui"),
            source: wgpu::ShaderSource::Wgsl(ui_source().into()),
        });
        let builtin = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ui builtin materials"),
            source: wgpu::ShaderSource::Wgsl(builtin_source().into()),
        });
        let pipelines = |format| {
            let p = |label, fs, pipe| {
                pipeline(
                    device,
                    label,
                    &shape_layout,
                    &module,
                    "vs_shape",
                    fs,
                    format,
                    blend_state(pipe),
                    false,
                )
            };
            UiPipelines {
                normal: p("ui shapes", "fs_shape", None),
                multiply: p("ui multiply", "fs_shape", Some(ShapePipe::Multiply)),
                screen: p("ui screen", "fs_shape", Some(ShapePipe::Screen)),
                add: p("ui add", "fs_shape", Some(ShapePipe::Add)),
                destination: p("ui blends", "fs_shape_dst", None),
                backdrop: p("ui backdrop", "fs_backdrop", None),
                vector: pipeline(
                    device,
                    "ui vectors",
                    &shape_layout,
                    &module,
                    "vs_vector",
                    "fs_vector",
                    format,
                    wgpu::BlendState::ALPHA_BLENDING,
                    true,
                ),
                composite: pipeline(
                    device,
                    "ui composite",
                    &composite_layout,
                    &builtin,
                    "vs_shape",
                    "fs_layer",
                    format,
                    wgpu::BlendState::ALPHA_BLENDING,
                    false,
                ),
                mass: mass_pipeline(device, &mass_pipeline_layout, &mass_module, format),
            }
        };
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ui"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let white_tex = wgpu::util::DeviceExt::create_texture_with_data(
            device,
            queue,
            &wgpu::TextureDescriptor {
                label: Some("ui white"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &[255, 255, 255, 255],
        );
        let white = white_tex.create_view(&Default::default());
        let backdrop_params = uniform_buffer(device, "ui backdrop", 16);
        let dummy_backdrop = crate::bind_group(
            device,
            "ui backdrop (none)",
            &backdrop_layout,
            &[
                wgpu::BindingResource::TextureView(&white),
                wgpu::BindingResource::TextureView(&white),
                wgpu::BindingResource::Sampler(&sampler),
                backdrop_params.as_entire_binding(),
            ],
        );
        UiRenderer {
            output: pipelines(overlay.output_format),
            linear: overlay.linear_format.map(pipelines),
            layer: pipelines(LAYER_FORMAT),
            backdrop_layout,
            layer_layout,
            composite_layout,
            output_format: overlay.output_format,
            linear_format: overlay.linear_format,
            sampler,
            white,
            dummy_backdrop,
            backdrop_params,
            instances: storage_buffer(device, "ui instances", 256),
            gradients: storage_buffer(device, "ui gradients", 256),
            clips: storage_buffer(device, "ui clips", 256),
            points: storage_buffer(device, "ui points", 256),
            uniforms: Vec::new(),
            layer_params: Vec::new(),
            mass_camera: uniform_buffer(
                device,
                "ui mass camera",
                std::mem::size_of::<super::mass::MassCameraGpu>() as u64,
            ),
            mass_table: None,
            mass_instances: vertex_buffer(device, 4096),
            mass_layout,
            targets: Vec::new(),
            custom: HashMap::new(),
            vectors: HashMap::new(),
            frame: UiFrameGroups::default(),
        }
    }

    fn pipelines(&self, kind: TargetKind) -> &UiPipelines {
        match kind {
            TargetKind::Output => &self.output,
            TargetKind::Linear => self.linear.as_ref().expect("an sRGB view's pipelines"),
            TargetKind::Layer => &self.layer,
        }
    }

    fn format(&self, kind: TargetKind) -> wgpu::TextureFormat {
        match kind {
            TargetKind::Output => self.output_format,
            TargetKind::Linear => self.linear_format.expect("an sRGB view's format"),
            TargetKind::Layer => LAYER_FORMAT,
        }
    }

    /// Compiles the custom materials the frame's layers use (inside an
    /// error scope) and uploads the vectors it draws that are new.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RenderScene,
        frame: &OverlayFrame,
    ) {
        let Some(overlay) = &scene.overlay else {
            return;
        };
        for plan in &frame.layers {
            let UiMaterial::Custom(h) = overlay.ui.layers[plan.index as usize].material else {
                continue;
            };
            let Some(material) = scene.ui_materials.get(h.index()) else {
                continue;
            };
            let key = (scene.id(), h.index());
            if self
                .custom
                .get(&key)
                .is_none_or(|(previous, _)| previous != material)
            {
                let compiled = self.compile(device, material);
                self.custom.insert(key, (material.clone(), compiled));
            }
            let entry = &mut self.custom.get_mut(&key).expect("made above").1;
            if let Compiled::Ready { pending, .. } = entry
                && let Some(f) = pending
            {
                let waker = std::task::Waker::noop();
                let mut cx = std::task::Context::from_waker(waker);
                if let std::task::Poll::Ready(e) = f.as_mut().poll(&mut cx) {
                    *pending = None;
                    if let Some(e) = e {
                        *entry = Compiled::Failed(e.to_string());
                    }
                }
            }
        }
        for run in &frame.runs {
            if let OverlaySource::Vector(h) = run.source {
                let key = (scene.id(), h.index());
                if self.vectors.contains_key(&key) {
                    continue;
                }
                let Some(data) = scene.ui_vectors.get(h.index()) else {
                    continue;
                };
                let init = |label, contents: &[u8], usage| {
                    crate::buffer_init(
                        device,
                        queue,
                        &wgpu::util::BufferInitDescriptor {
                            label: Some(label),
                            contents,
                            usage,
                        },
                    )
                };
                self.vectors.insert(
                    key,
                    VectorMesh {
                        vertices: init(
                            "ui vector vertices",
                            bytemuck::cast_slice(&data.vertices),
                            wgpu::BufferUsages::VERTEX,
                        ),
                        indices: init(
                            "ui vector indices",
                            bytemuck::cast_slice(&data.indices),
                            wgpu::BufferUsages::INDEX,
                        ),
                        count: data.indices.len() as u32,
                    },
                );
            }
        }
    }

    fn compile(&self, device: &wgpu::Device, material: &UiMaterialShader) -> Compiled {
        if let Err(e) = material.check() {
            return Compiled::Failed(e);
        }
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&material.name),
            source: wgpu::ShaderSource::Wgsl(ui_material_module_source(material).into()),
        });
        let mut pipelines = Vec::new();
        let mut kinds = vec![TargetKind::Output, TargetKind::Layer];
        if self.linear_format.is_some() {
            kinds.push(TargetKind::Linear);
        }
        for kind in kinds {
            let p = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&material.name),
                layout: Some(&self.composite_layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs_shape"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some("fs_layer"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: self.format(kind),
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            });
            pipelines.push((kind, p));
        }
        let mut pending = Box::pin(scope.pop());
        // Native devices answer at once; the web's on a later turn (drawn meanwhile: custom_pipeline).
        let waker = std::task::Waker::noop();
        let mut cx = std::task::Context::from_waker(waker);
        let pending = match pending.as_mut().poll(&mut cx) {
            std::task::Poll::Ready(Some(e)) => return Compiled::Failed(e.to_string()),
            std::task::Poll::Ready(None) => None,
            std::task::Poll::Pending => {
                Some(pending as std::pin::Pin<Box<dyn std::future::Future<Output = _>>>)
            }
        };
        Compiled::Ready { pipelines, pending }
    }

    /// The custom materials that failed, with the device's message.
    pub fn errors(&self, scene: u64) -> Vec<(super::UiMaterialHandle, String)> {
        let mut out: Vec<_> = self
            .custom
            .iter()
            .filter(|((s, _), _)| *s == scene)
            .filter_map(|((_, i), (_, c))| match c {
                Compiled::Failed(e) => Some((super::UiMaterialHandle(*i as u32), e.clone())),
                _ => None,
            })
            .collect();
        og_core::sort::sort_by_key(&mut out, |(h, _)| *h);
        out
    }

    fn custom_pipeline(
        &self,
        scene: u64,
        material: super::UiMaterialHandle,
        kind: TargetKind,
    ) -> Option<&wgpu::RenderPipeline> {
        // Drawn while its error scope is still pending, as a custom post pass is
        // (post_custom.rs): the web's pop answers only on a later turn of the
        // browser's event loop, after the frames a harness or the first frame
        // draw in one, and `material.check()` has validated the module already.
        // A scope that then reports an error makes it Failed (`prepare`).
        match self.custom.get(&(scene, material.index())) {
            Some((_, Compiled::Ready { pipelines, .. })) => {
                pipelines.iter().find(|(k, _)| *k == kind).map(|(_, p)| p)
            }
            _ => None,
        }
    }

    /// Writes the frame's tables and uniforms and makes its groups: the
    /// output's (`uniform`), each layer's, their targets and composite
    /// groups; `backdrop`: the glass pyramid and the copy of the target,
    /// when the frame has them, with `decode`.
    #[allow(clippy::too_many_arguments)]
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        overlay_layout: &wgpu::BindGroupLayout,
        scene: &RenderScene,
        frame: &OverlayFrame,
        uniform: [f32; 8],
        backdrop: Option<(&wgpu::TextureView, &wgpu::TextureView, bool)>,
        texture_view: impl Fn(crate::TextureHandle) -> Option<wgpu::TextureView>,
        depth: &wgpu::TextureView,
        camera: super::mass::MassCameraGpu,
        mips: &crate::mips::MipGenerator,
    ) {
        let Some(overlay) = &scene.overlay else {
            return;
        };
        write_storage(
            device,
            queue,
            &mut self.instances,
            "ui instances",
            bytemuck::cast_slice(&frame.ui_instances),
        );
        write_storage(
            device,
            queue,
            &mut self.gradients,
            "ui gradients",
            bytemuck::cast_slice(&frame.tables.gradients),
        );
        write_storage(
            device,
            queue,
            &mut self.clips,
            "ui clips",
            bytemuck::cast_slice(&frame.tables.clips),
        );
        write_storage(
            device,
            queue,
            &mut self.points,
            "ui points",
            bytemuck::cast_slice(&frame.tables.points),
        );
        let need = 1 + frame.layers.len();
        while self.uniforms.len() < need {
            self.uniforms.push(uniform_buffer(device, "ui frame", 32));
        }
        while self.layer_params.len() < frame.layers.len() {
            self.layer_params.push(uniform_buffer(
                device,
                "ui layer",
                std::mem::size_of::<LayerParams>() as u64,
            ));
        }
        let group0 = |buffer: &wgpu::Buffer| {
            crate::bind_group(
                device,
                "ui frame",
                overlay_layout,
                &[
                    buffer.as_entire_binding(),
                    self.clips.as_entire_binding(),
                    self.gradients.as_entire_binding(),
                    self.points.as_entire_binding(),
                    self.instances.as_entire_binding(),
                    wgpu::BindingResource::TextureView(depth),
                ],
            )
        };
        queue.write_buffer(&self.uniforms[0], 0, bytemuck::cast_slice(&uniform));
        let mut groups = UiFrameGroups {
            main: Some(group0(&self.uniforms[0])),
            ..Default::default()
        };
        if !frame.mass_instances.is_empty() {
            let n = frame.mass_templates.len().min(super::UI_MASS_BATCHES);
            let mut camera = camera;
            camera.tick[1] = (n * super::mass::TEMPLATE_TEXELS) as u32;
            queue.write_buffer(&self.mass_camera, 0, bytemuck::bytes_of(&camera));
            let bytes: &[u8] = bytemuck::cast_slice(&frame.mass_instances);
            if bytes.len() as u64 > self.mass_instances.size() {
                self.mass_instances =
                    vertex_buffer(device, (bytes.len() as u64).next_power_of_two());
            }
            queue.write_buffer(&self.mass_instances, 0, bytes);
            // The table: rows of TABLE_WIDTH entries, at least one.
            let width = super::mass::TABLE_WIDTH;
            let mut texels: Vec<[f32; 4]> =
                Vec::with_capacity(n * super::mass::TEMPLATE_TEXELS + frame.mass_table.len());
            for t in &frame.mass_templates[..n] {
                texels.extend(t.texels());
            }
            texels.extend_from_slice(&frame.mass_table);
            let rows = (texels.len() as u32).div_ceil(width).max(1);
            if self.mass_table.as_ref().is_none_or(|t| t.2 < rows) {
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("ui mass table"),
                    size: wgpu::Extent3d {
                        width,
                        height: rows.next_power_of_two(),
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba32Float,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                let view = texture.create_view(&Default::default());
                self.mass_table = Some((texture, view, rows.next_power_of_two()));
            }
            let (texture, view, _) = self.mass_table.as_ref().expect("made above");
            texels.resize((rows * width) as usize, [0.0; 4]);
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                bytemuck::cast_slice(&texels),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 16),
                    rows_per_image: Some(rows),
                },
                wgpu::Extent3d {
                    width,
                    height: rows,
                    depth_or_array_layers: 1,
                },
            );
            groups.mass = Some(crate::bind_group(
                device,
                "ui mass",
                &self.mass_layout,
                &[
                    self.mass_camera.as_entire_binding(),
                    wgpu::BindingResource::TextureView(view),
                ],
            ));
        }
        // Targets: one per layer drawn, reused by size and mips.
        let mut used = vec![false; self.targets.len()];
        let mut target_of = Vec::new();
        let wants_mips = |plan: &crate::overlay::LayerPlan| {
            layer_mips(overlay.ui.layers[plan.index as usize].material)
        };
        for plan in &frame.layers {
            if plan.target.is_some() {
                // A world panel draws into its own render target.
                target_of.push(usize::MAX);
                continue;
            }
            let with_mips = wants_mips(plan);
            let found = self.targets.iter().enumerate().position(|(i, t)| {
                !used[i] && t.size == plan.size && t.chain.is_some() == with_mips
            });
            let i = match found {
                Some(i) => i,
                None => {
                    let texture = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("ui layer"),
                        size: wgpu::Extent3d {
                            width: plan.size.0,
                            height: plan.size.1,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: if with_mips {
                            crate::mips::mip_count(plan.size.0, plan.size.1)
                        } else {
                            1
                        },
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: LAYER_FORMAT,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::TEXTURE_BINDING,
                        view_formats: &[],
                    });
                    let sampled = texture.create_view(&Default::default());
                    let (view, chain) = if with_mips {
                        let chain = mips.chain(device, &texture);
                        (chain.level(0).clone(), Some(chain))
                    } else {
                        (sampled.clone(), None)
                    };
                    self.targets.push(LayerTarget {
                        size: plan.size,
                        view,
                        sampled,
                        chain,
                    });
                    used.push(false);
                    self.targets.len() - 1
                }
            };
            used[i] = true;
            target_of.push(i);
        }
        // Targets no frame used are let go when there are many.
        if self.targets.len() > 16 {
            let mut k = 0;
            self.targets.retain(|_| {
                let keep = used.get(k).copied().unwrap_or(false);
                k += 1;
                keep
            });
            // Indices moved: find them again.
            target_of.clear();
            let mut taken = vec![false; self.targets.len()];
            for plan in &frame.layers {
                if plan.target.is_some() {
                    target_of.push(usize::MAX);
                    continue;
                }
                let i = self
                    .targets
                    .iter()
                    .enumerate()
                    .position(|(i, t)| {
                        !taken[i] && t.size == plan.size && t.chain.is_some() == wants_mips(plan)
                    })
                    .expect("kept above");
                taken[i] = true;
                target_of.push(i);
            }
        }
        for (n, plan) in frame.layers.iter().enumerate() {
            let layer = &overlay.ui.layers[plan.index as usize];
            let u = [
                plan.size.0 as f32,
                plan.size.1 as f32,
                plan.scale,
                0.0,
                plan.origin[0],
                plan.origin[1],
                uniform[6],
                0.0,
            ];
            queue.write_buffer(&self.uniforms[n + 1], 0, bytemuck::cast_slice(&u));
            let builtin = match layer.material {
                UiMaterial::Builtin(b) => builtin_code(b),
                _ => 0.0,
            };
            let mut params = LayerParams {
                rect: plan.rect,
                params: layer.params,
                info: [builtin, 0.0, 0.0, 0.0],
                mask_uv: [0.0, 0.0, 1.0, 1.0],
                quad: [[0.0; 4]; 3],
            };
            if let Some(m) = layer
                .quad
                .as_ref()
                .and_then(super::square_to_quad)
                .and_then(|m| super::invert3(&m))
            {
                params.info[3] = 1.0;
                params.quad = m.map(|r| [r[0], r[1], r[2], 0.0]);
            }
            let mut mask_view = None;
            match &layer.mask {
                Some(UiMask::Gradient(_)) => {
                    params.info[1] = 1.0;
                    params.info[2] =
                        frame.layer_masks.get(&plan.index).copied().unwrap_or(0) as f32;
                }
                Some(UiMask::Image { texture, uv }) => {
                    if let Some(v) = texture_view(*texture) {
                        params.info[1] = 2.0;
                        params.mask_uv = *uv;
                        mask_view = Some(v);
                    }
                }
                None => {}
            }
            queue.write_buffer(&self.layer_params[n], 0, bytemuck::bytes_of(&params));
            // A world panel composites nowhere: its group samples white.
            let target = self
                .targets
                .get(target_of[n])
                .map_or(&self.white, |t| &t.sampled);
            let composite = crate::bind_group(
                device,
                "ui layer composite",
                &self.layer_layout,
                &[
                    wgpu::BindingResource::TextureView(target),
                    wgpu::BindingResource::Sampler(&self.sampler),
                    self.layer_params[n].as_entire_binding(),
                    wgpu::BindingResource::TextureView(mask_view.as_ref().unwrap_or(&self.white)),
                ],
            );
            groups.layers.insert(
                plan.index,
                (group0(&self.uniforms[n + 1]), target_of[n], composite),
            );
        }
        if let Some((pyramid, behind, decode)) = backdrop {
            queue.write_buffer(
                &self.backdrop_params,
                0,
                bytemuck::cast_slice(&[
                    f32::from(u8::from(decode)),
                    0.0,
                    0.0,
                    crate::glass::LEVELS as f32,
                ]),
            );
            groups.backdrop = Some(crate::bind_group(
                device,
                "ui backdrop",
                &self.backdrop_layout,
                &[
                    wgpu::BindingResource::TextureView(pyramid),
                    wgpu::BindingResource::TextureView(behind),
                    wgpu::BindingResource::Sampler(&self.sampler),
                    self.backdrop_params.as_entire_binding(),
                ],
            ));
        }
        self.frame = groups;
    }
}

/// A built-in material's code in ui_builtin.wgsl.
fn builtin_code(b: UiBuiltin) -> f32 {
    match b {
        UiBuiltin::None => 0.0,
        UiBuiltin::Dissolve => 1.0,
        UiBuiltin::Shine => 2.0,
        UiBuiltin::Hologram => 3.0,
        UiBuiltin::Glitch => 4.0,
        UiBuiltin::Ripple => 5.0,
        UiBuiltin::Pixelate => 6.0,
        UiBuiltin::ColorAdjust => 7.0,
        UiBuiltin::Outline => 8.0,
        UiBuiltin::Blur => 9.0,
        UiBuiltin::ColorMatrix => 10.0,
        UiBuiltin::DropShadow => 11.0,
    }
}

impl Renderer {
    /// Draws `runs` (one target's) into `pass` with `group0` bound, in
    /// `kind`'s pipelines. Returns the draws.
    pub(crate) fn draw_overlay_runs<'p>(
        &'p self,
        pass: &mut wgpu::RenderPass<'p>,
        scene: &RenderScene,
        runs: &[OverlayRun],
        group0: &'p wgpu::BindGroup,
        kind: TargetKind,
    ) -> u32 {
        let overlay_pipes = match kind {
            TargetKind::Output => &self.overlay.output,
            TargetKind::Linear => self
                .overlay
                .linear
                .as_ref()
                .expect("an sRGB view's pipelines"),
            TargetKind::Layer => &self.overlay.layer,
        };
        let ui = self.ui.pipelines(kind);
        let materials = scene.overlay.as_ref().map(|o| &o.ui.layers);
        pass.set_bind_group(0, group0, &[]);
        let backdrop = self
            .ui
            .frame
            .backdrop
            .as_ref()
            .unwrap_or(&self.ui.dummy_backdrop);
        let mut draws = 0;
        let sprite_group = |t: Option<crate::TextureHandle>| match t {
            None => &self.white_sprite_group,
            Some(t) => self.cache.sprite_groups[t.index()]
                .as_ref()
                .expect("made before the pass"),
        };
        for run in runs {
            let range = run.first..run.first + run.count;
            match run.source {
                OverlaySource::Sprite(_)
                | OverlaySource::Font(_)
                | OverlaySource::Glass
                | OverlaySource::Array(_) => {
                    let (pipeline, group) = match run.source {
                        OverlaySource::Sprite(t) => (&*overlay_pipes.sprite, sprite_group(t)),
                        OverlaySource::Font(f) => {
                            let font = &self.cache.fonts[f.index()];
                            // A colour page's glyphs are images.
                            let pipeline = if font.layout.color {
                                &*overlay_pipes.sprite
                            } else {
                                &*overlay_pipes.msdf
                            };
                            (pipeline, &font.bind_group)
                        }
                        OverlaySource::Glass => (
                            &*overlay_pipes.glass,
                            &self
                                .glass_targets
                                .as_ref()
                                .expect("made with the overlay's glass")
                                .overlay_group,
                        ),
                        OverlaySource::Array(a) => {
                            // A layer has no array pipelines: its arrays are the output's.
                            if kind == TargetKind::Layer {
                                continue;
                            }
                            (
                                self.sprite_arrays
                                    .as_ref()
                                    .expect("made before the pass")
                                    .overlay(kind == TargetKind::Linear),
                                &self.cache.sprite_array_groups[a.index()]
                                    .as_ref()
                                    .expect("made before the pass")
                                    .0,
                            )
                        }
                        _ => unreachable!("matched above"),
                    };
                    pass.set_pipeline(pipeline);
                    pass.set_bind_group(1, group, &[]);
                    let stride = crate::overlay::OVERLAY_STRIDE;
                    let start = run.first as u64 * stride;
                    pass.set_vertex_buffer(
                        0,
                        self.overlay
                            .buffer
                            .slice(start..start + run.count as u64 * stride),
                    );
                    pass.draw(0..6, 0..run.count);
                }
                OverlaySource::Ui(ShapeSource::Shape(pipe, t)) => {
                    pass.set_pipeline(match pipe {
                        ShapePipe::Normal => &ui.normal,
                        ShapePipe::Multiply => &ui.multiply,
                        ShapePipe::Screen => &ui.screen,
                        ShapePipe::Add => &ui.add,
                        ShapePipe::Destination => &ui.destination,
                    });
                    pass.set_bind_group(1, sprite_group(t), &[]);
                    pass.set_bind_group(2, backdrop, &[]);
                    pass.draw(0..6, range);
                }
                OverlaySource::Ui(ShapeSource::Backdrop | ShapeSource::SolidBackdrop) => {
                    pass.set_pipeline(&ui.backdrop);
                    pass.set_bind_group(1, &self.white_sprite_group, &[]);
                    pass.set_bind_group(2, backdrop, &[]);
                    pass.draw(0..6, range);
                }
                OverlaySource::Ui(ShapeSource::Composite(k)) => {
                    let Some((_, _, group)) = self.ui.frame.layers.get(&k) else {
                        continue;
                    };
                    let pipeline = match materials
                        .and_then(|l| l.get(k as usize))
                        .map(|l| l.material)
                    {
                        Some(UiMaterial::Custom(h)) => {
                            match self.ui.custom_pipeline(scene.id(), h, kind) {
                                Some(p) => p,
                                // A material that failed (or is compiling) draws plain.
                                None => &ui.composite,
                            }
                        }
                        _ => &ui.composite,
                    };
                    pass.set_pipeline(pipeline);
                    pass.set_bind_group(1, group, &[]);
                    pass.draw(0..6, range);
                }
                OverlaySource::Mass(k) => {
                    let (Some(group), Some(batch)) = (
                        self.ui.frame.mass.as_ref(),
                        scene
                            .overlay
                            .as_ref()
                            .and_then(|o| o.ui.mass.get(k as usize)),
                    ) else {
                        continue;
                    };
                    let t = &batch.template;
                    let texture = match t.kind {
                        super::UiMassKind::Number { .. } | super::UiMassKind::Text { .. } => {
                            t.font.map(|f| &self.cache.fonts[f.index()].bind_group)
                        }
                        _ => None,
                    }
                    .unwrap_or_else(|| sprite_group(t.texture));
                    let stride = std::mem::size_of::<super::UiMassInstance>() as u64;
                    pass.set_pipeline(&ui.mass);
                    pass.set_bind_group(1, texture, &[]);
                    pass.set_bind_group(2, group, &[]);
                    pass.set_vertex_buffer(
                        0,
                        self.ui.mass_instances.slice(
                            run.first as u64 * stride..(run.first + run.count) as u64 * stride,
                        ),
                    );
                    pass.draw(0..6 * t.quads(), 0..run.count);
                }
                OverlaySource::Vector(h) => {
                    let Some(mesh) = self.ui.vectors.get(&(scene.id(), h.index())) else {
                        continue;
                    };
                    pass.set_pipeline(&ui.vector);
                    pass.set_bind_group(1, &self.white_sprite_group, &[]);
                    pass.set_bind_group(2, backdrop, &[]);
                    pass.set_vertex_buffer(0, mesh.vertices.slice(..));
                    pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..mesh.count, 0, range);
                }
            }
            draws += 1;
        }
        draws
    }

    /// Draws the frame's UI layers into their targets, innermost first.
    pub(crate) fn draw_ui_layers(
        &self,
        encoder: &mut crate::Encoder,
        scene: &RenderScene,
        frame: &OverlayFrame,
    ) -> u32 {
        let mut draws = 0;
        for plan in &frame.layers {
            let Some((group0, target, _)) = self.ui.frame.layers.get(&plan.index) else {
                continue;
            };
            let view = match plan.target {
                Some(t) => &self.cache.textures[t].view,
                None => &self.ui.targets[*target].view,
            };
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ui layer"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            draws += self.draw_overlay_runs(
                &mut pass,
                scene,
                frame.layer_runs(plan.index),
                group0,
                TargetKind::Layer,
            );
            drop(pass);
            // A blur's levels, from what the layer drew.
            if plan.target.is_none()
                && let Some(chain) = &self.ui.targets[*target].chain
            {
                self.mips.record_chain(encoder, chain);
                draws += chain.level_count() - 1;
            }
        }
        draws
    }

    /// The frame's group 0 for the output.
    pub(crate) fn ui_main_group(&self) -> Option<&wgpu::BindGroup> {
        self.ui.frame.main.as_ref()
    }

    /// The custom UI materials ([`RenderScene::add_ui_material`]) that
    /// failed to compile on the device, with its message; they composite
    /// as the plain layer.
    pub fn ui_material_errors(
        &self,
        scene: &RenderScene,
    ) -> Vec<(super::UiMaterialHandle, String)> {
        self.ui.errors(scene.id())
    }
}
