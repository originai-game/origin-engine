//! The main pass's depth seeded from the prepass's.
//!
//! A frame with a prepass has drawn every opaque surface's depth before
//! the main pass shades one. Without multisampling the main pass starts
//! from it: after its clear, one triangle writes the prepass's depth, a
//! little farther (`depth_seed.wgsl` SLACK), into the main depth, so a
//! fragment hidden behind an opaque surface fails the depth test before its
//! lit shader runs. The lit pipelines are the ones every other pass uses
//! (Less, writing depth): the nearest lit fragment still wins each pixel and
//! the main depth ends as it did, so nothing downstream changes. Nothing
//! here needs a lit vertex to land on its prepass one to the bit, only
//! within the slack.
//!
//! What it does need: the prepass keeps no fragment the lit pass discards
//! (the prepass's alpha cut and dissolve cut at least what the lit pass
//! cuts), or what lies behind that fragment is hidden by nothing.

use crate::{HdrPipeline, hdr_pipeline};

/// The seed's pipeline and its group layout (the prepass's depth).
pub(crate) struct DepthSeed {
    layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

/// The colour target as it is: the seed writes depth only.
const KEEP: wgpu::BlendComponent = wgpu::BlendComponent {
    src_factor: wgpu::BlendFactor::Zero,
    dst_factor: wgpu::BlendFactor::One,
    operation: wgpu::BlendOperation::Add,
};

impl DepthSeed {
    pub fn new(device: &wgpu::Device) -> DepthSeed {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("depth seed"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("depth seed"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let module = crate::shader(device, "depth seed", DEPTH_SEED_WGSL.to_string());
        let pipeline = hdr_pipeline(
            device,
            HdrPipeline {
                label: "depth seed",
                layout: &pipeline_layout,
                module: &module,
                vs: "vs_seed",
                fs: "fs_seed",
                buffers: &[],
                cull: None,
                depth_write: true,
                depth_compare: wgpu::CompareFunction::Always,
                blend: Some(wgpu::BlendState {
                    color: KEEP,
                    alpha: KEEP,
                }),
                samples: 1,
                alpha_to_coverage: false,
            },
        );
        DepthSeed { layout, pipeline }
    }

    /// Seeds `pass`'s depth from `pre_depth`, the prepass's of this frame;
    /// first in the main pass, over its clear. Leaves group 0 its own.
    pub fn draw(
        &self,
        device: &wgpu::Device,
        pass: &mut wgpu::RenderPass<'_>,
        pre_depth: &wgpu::TextureView,
    ) {
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("depth seed"),
            layout: &self.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(pre_depth),
            }],
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.draw(0..3, 0..1);
    }
}

/// The seed's module source (`shaders.rs` validates it).
pub(crate) const DEPTH_SEED_WGSL: &str = wgsl!("depth_seed.wgsl");
