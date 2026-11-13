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

mod draw;
mod pipelines;
mod setup;
mod upload;

pub(crate) use pipelines::*;
use upload::*;

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
    /// A projected layer's drape (`info.w` 1, 2 on its plane only,
    /// [`super::UiDrape`]): its
    /// rows `a` with the scale's x and y and the reach in w, then its
    /// rows `b`.
    drape: [[f32; 4]; 6],
}

/// The shape family's pipelines for one target format.
pub(crate) struct UiPipelines {
    normal: Deferred<wgpu::RenderPipeline>,
    multiply: Deferred<wgpu::RenderPipeline>,
    screen: Deferred<wgpu::RenderPipeline>,
    add: Deferred<wgpu::RenderPipeline>,
    destination: Deferred<wgpu::RenderPipeline>,
    /// The Porter-Duff operators, for shapes and for composites, in
    /// [`OPERATORS`]' order.
    operators: [Deferred<wgpu::RenderPipeline>; 4],
    composite_operators: [Deferred<wgpu::RenderPipeline>; 4],
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
    texture: wgpu::Texture,
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
    /// What a layer's blends read: [`UiRenderer::layer_behind`].
    layer_backdrop: Option<wgpu::BindGroup>,
    /// The filters of the layers that have one, by layer index.
    filters: HashMap<u32, super::filter::Prepared>,
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
    /// The copy of a layer as drawn so far that its blends read, as large
    /// as the largest layer that has them; its group's parameters.
    layer_behind: Option<(wgpu::Texture, (u32, u32))>,
    layer_backdrop_params: wgpu::Buffer,
    filters: super::filter::FilterRunner,
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

impl UiRenderer {
    /// Compiles the custom materials the frame's layers use (inside an
    /// error scope) and uploads the vectors it draws that are new.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RenderScene,
        frame: &OverlayFrame,
    ) {
        let Some(overlay) = &scene.ui.overlay else {
            return;
        };
        for plan in &frame.layers {
            let UiMaterial::Custom(h) = overlay.ui.layers[plan.index as usize].material else {
                continue;
            };
            let Some(material) = scene.ui.ui_materials.get(h.index()) else {
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
                let Some(data) = scene.ui.ui_vectors.get(h.index()) else {
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
}
