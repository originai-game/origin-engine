//! SMAA 1x, high preset. Reference and WGSL port: MIT; see
//! third_party/smaa/LICENSE{,-wgsl}.txt, including the lookup textures.

use super::*;

pub(super) const EDGES: &str = concat!(
    include_str!("../../../../third_party/smaa/edges.wgsl"),
    wgsl!("smaa_edges.wgsl")
);
pub(super) const WEIGHTS: &str = concat!(
    include_str!("../../../../third_party/smaa/weights.wgsl"),
    wgsl!("smaa_weights.wgsl")
);
pub(super) const BLEND: &str = concat!(
    include_str!("../../../../third_party/smaa/velocity.wgsl"),
    include_str!("../../../../third_party/smaa/blend.wgsl"),
    wgsl!("smaa_blend.wgsl")
);

pub(super) struct Smaa {
    pub edges: wgpu::RenderPipeline,
    pub weights: wgpu::RenderPipeline,
    pub weights_layout: wgpu::BindGroupLayout,
    pub blend_layout: wgpu::BindGroupLayout,
    pub blend_pl: wgpu::PipelineLayout,
    pub blend_shader: wgpu::ShaderModule,
    pub area: wgpu::TextureView,
    pub search: wgpu::TextureView,
}

impl Smaa {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        source_pl: &wgpu::PipelineLayout,
    ) -> Self {
        use wgpu::util::DeviceExt;
        let layout = |label, count| {
            let entries: Vec<_> = (0..count)
                .map(|i| {
                    layout_entry(
                        i,
                        if i == 1 {
                            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
                        } else {
                            float_texture(true)
                        },
                    )
                })
                .collect();
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(label),
                entries: &entries,
            })
        };
        let pl = |layout: &wgpu::BindGroupLayout| {
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("smaa"),
                bind_group_layouts: &[Some(layout)],
                immediate_size: 0,
            })
        };
        let shader = |label, body| {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(fullscreen_source(body).into()),
            })
        };
        let weights_layout = layout("smaa weights", 4);
        let blend_layout = layout("smaa blend", 3);
        let edges = fullscreen_pipeline(
            device,
            "smaa edges",
            &shader("smaa edges", EDGES),
            "fs",
            source_pl,
            wgpu::TextureFormat::Rg8Unorm,
            None,
        );
        let weights = fullscreen_pipeline(
            device,
            "smaa weights",
            &shader("smaa weights", WEIGHTS),
            "fs",
            &pl(&weights_layout),
            wgpu::TextureFormat::Rgba8Unorm,
            None,
        );
        let lookup = |label, width, height, format, bytes: &[u8]| {
            device
                .create_texture_with_data(
                    queue,
                    &wgpu::TextureDescriptor {
                        label: Some(label),
                        size: wgpu::Extent3d {
                            width,
                            height,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format,
                        usage: wgpu::TextureUsages::TEXTURE_BINDING,
                        view_formats: &[],
                    },
                    wgpu::util::TextureDataOrder::LayerMajor,
                    bytes,
                )
                .create_view(&Default::default())
        };
        Self {
            edges,
            weights,
            weights_layout,
            blend_pl: pl(&blend_layout),
            blend_layout,
            blend_shader: shader("smaa blend", BLEND),
            area: lookup(
                "smaa area",
                160,
                560,
                wgpu::TextureFormat::Rg8Unorm,
                include_bytes!("../../../../third_party/smaa/area.bin"),
            ),
            search: lookup(
                "smaa search",
                64,
                16,
                wgpu::TextureFormat::R8Unorm,
                include_bytes!("../../../../third_party/smaa/search.bin"),
            ),
        }
    }
}
