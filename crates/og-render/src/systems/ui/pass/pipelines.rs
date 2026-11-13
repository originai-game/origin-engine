//! The shape family's shader sources, blend states and pipelines.

use super::*;

/// The shape family's module.
pub(crate) fn ui_source() -> String {
    format!(
        "{}{}{}{}",
        super::super::UI_COMMON_WGSL,
        super::super::UI_BLEND_MODES_WGSL,
        super::super::UI_BLEND_WGSL,
        UI_WGSL
    )
}

/// The mass elements' module.
pub(crate) fn mass_source() -> String {
    format!(
        "{}{}{}",
        crate::HASH_WGSL,
        super::super::UI_COMMON_WGSL,
        UI_MASS_WGSL
    )
}

/// The built-in materials' composite module.
pub(crate) fn builtin_source() -> String {
    format!(
        "{}{}{}{}{}{}",
        crate::HASH_WGSL,
        super::super::UI_COMMON_WGSL,
        super::super::UI_BLEND_MODES_WGSL,
        super::super::UI_BLEND_WGSL,
        super::super::UI_MATERIAL_WGSL,
        UI_BUILTIN_WGSL
    )
}

pub(super) const UI_WGSL: &str = wgsl!("ui.wgsl");

pub(super) const UI_BUILTIN_WGSL: &str = wgsl!("ui_builtin.wgsl");

pub(super) const UI_MASS_WGSL: &str = wgsl!("ui_mass.wgsl");

/// The Porter-Duff pipes, in the order of their pipelines.
pub(super) const OPERATORS: [ShapePipe; 4] = [
    ShapePipe::SourceIn,
    ShapePipe::SourceOut,
    ShapePipe::DestinationIn,
    ShapePipe::Xor,
];

pub(super) fn blend_state(pipe: Option<ShapePipe>) -> wgpu::BlendState {
    // The operators' factors on premultiplied colour (the shaders write
    // it for them), alike for colour and alpha.
    let op = |src_factor, dst_factor| {
        let c = wgpu::BlendComponent {
            src_factor,
            dst_factor,
            operation: wgpu::BlendOperation::Add,
        };
        wgpu::BlendState { color: c, alpha: c }
    };
    use wgpu::BlendFactor as F;
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
        // Screen on premultiplied colour is exact over any backdrop
        // (Sc + Dc - Sc·Dc), its alpha source-over; plus-lighter adds both.
        Some(ShapePipe::Screen) => wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrc,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        },
        Some(ShapePipe::Add) => wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        },
        Some(ShapePipe::SourceIn) => op(F::DstAlpha, F::Zero),
        Some(ShapePipe::SourceOut) => op(F::OneMinusDstAlpha, F::Zero),
        Some(ShapePipe::DestinationIn) => op(F::Zero, F::SrcAlpha),
        Some(ShapePipe::Xor) => op(F::OneMinusDstAlpha, F::OneMinusSrcAlpha),
        _ => wgpu::BlendState::ALPHA_BLENDING,
    }
}

pub(super) fn vertex_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ui mass instances"),
        size: size.max(256),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

/// The mass elements' pipeline: one instance per element from the vertex
/// buffer.
pub(super) fn mass_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    module: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
) -> Deferred<wgpu::RenderPipeline> {
    let (device, layout, module) = (device.clone(), layout.clone(), module.clone());
    Deferred::new(move || {
        const BUFFERS: &[Option<wgpu::VertexBufferLayout<'static>>] =
            &[Some(super::super::mass::MASS_LAYOUT)];
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
pub(super) fn pipeline(
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
