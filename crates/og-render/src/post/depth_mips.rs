//! The view depth's MIP chain (depth_mips.wgsl): XeGTAO's depth prefilter
//! (Intel, MIT, `third_party/xegtao`) as one compute pass, `depth-mips`,
//! before the ambient occlusion. Level 0 is the view depth the AO searches
//! (the prepass's, or with the water's surfaces in front), each level after
//! it one depth for each 2 x 2 of the one below, weighted towards the
//! farthest within the AO's falloff. GTAO's horizon search (fxpost.wgsl
//! `gtao_bent`) reads its far taps from the coarser levels, so a tap many
//! pixels out reads a texel its neighbours' taps share instead of a cache
//! line of its own.
//!
//! [`DepthMips`] is the chain's one owner: the frame graph keeps it at the
//! frame's size ([`crate::post::FrameGraph::depth_mips`]) and the
//! `depth-mips` pass fills it each frame the AO runs. A pass that reads it
//! declares a read of the graph's `depth-mips` target, which orders it
//! after the fill. The levels are a filtered depth, not a conservative
//! bound: a march that must not step past a surface (screen-space
//! reflections) wants the min-max pyramid `shading/ssr.rs` builds.

use super::fsr::{self, B};
use crate::deferred::Deferred;

const WGSL: &str = wgsl_library!("fxpost_uniform.wgsl", "depth_mips.wgsl");

/// The chain's levels: XeGTAO has five; WebGPU's default limits allow four
/// storage textures in a stage, and one pass writes them all.
pub(crate) const DEPTH_MIP_LEVELS: u32 = 4;
/// View depth in metres (r), as the prepass's `lindepth` holds it.
pub(crate) const DEPTH_MIPS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;
/// The graph target a pass reads to see the filled chain.
pub(crate) const DEPTH_MIPS_TARGET: &str = "depth-mips";

/// The module's source, for validation (`shaders.rs`).
pub(crate) fn source() -> String {
    WGSL.to_string()
}

/// The chain at one screen size: the texture with all its levels in one
/// view (what a shader samples, `textureLoad(t, px >> level, level)`) and
/// each level alone (what the pass stores to).
pub(crate) struct DepthMips {
    size: (u32, u32),
    view: wgpu::TextureView,
    levels: Vec<wgpu::TextureView>,
}

impl DepthMips {
    /// At least 8 x 8, so every level has a texel (a frame smaller than
    /// that reads its edge pixels again past it).
    pub fn new(device: &wgpu::Device, (width, height): (u32, u32)) -> DepthMips {
        let (width, height) = (width.max(8), height.max(8));
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth mips"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: DEPTH_MIP_LEVELS,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_MIPS_FORMAT,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
            texture_binding_view_dimension: None,
        });
        DepthMips {
            size: (width, height),
            view: texture.create_view(&Default::default()),
            levels: crate::mips::level_views(&texture, "depth mips level"),
        }
    }

    /// Its level 0's size: the frame's, at least 8 x 8.
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// Every level in one view, for reading.
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

/// The pass's layout and pipeline.
pub(crate) struct DepthMipsPass {
    pub layout: wgpu::BindGroupLayout,
    pipeline: Deferred<wgpu::ComputePipeline>,
}

impl DepthMipsPass {
    pub fn new(device: &wgpu::Device) -> DepthMipsPass {
        let entries: Vec<_> = [B::Uniform, B::Tex(false)]
            .into_iter()
            .chain((0..DEPTH_MIP_LEVELS).map(|_| B::Storage(DEPTH_MIPS_FORMAT)))
            .enumerate()
            .map(|(i, b)| wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: fsr::binding_type(b),
                count: None,
            })
            .collect();
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("depth mips"),
            entries: &entries,
        });
        let module = crate::deferred::module(device, "depth mips", WGSL.to_string());
        let pipeline = {
            let (device, layout) = (device.clone(), layout.clone());
            Deferred::new(move || {
                let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("depth mips"),
                    bind_group_layouts: &[Some(&layout)],
                    immediate_size: 0,
                });
                crate::traffic::create_compute_pipeline(
                    &device,
                    &wgpu::ComputePipelineDescriptor {
                        label: Some("depth mips"),
                        layout: Some(&pl),
                        module: module.get(),
                        entry_point: Some("cs_depth_mips"),
                        compilation_options: Default::default(),
                        cache: crate::pipeline_cache::of(&device).as_ref(),
                    },
                )
            })
        };
        DepthMipsPass { layout, pipeline }
    }

    /// The group that fills `mips` from `source` (the view depth, r) with
    /// the effect passes' uniform (`fx`: the AO's radius, which the levels'
    /// weights fall off over).
    pub fn bind(
        &self,
        device: &wgpu::Device,
        fx: &wgpu::Buffer,
        source: &wgpu::TextureView,
        mips: &DepthMips,
    ) -> wgpu::BindGroup {
        let mut entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: fx.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(source),
            },
        ];
        entries.extend(
            (0..DEPTH_MIP_LEVELS as usize).map(|l| wgpu::BindGroupEntry {
                binding: 2 + l as u32,
                resource: wgpu::BindingResource::TextureView(&mips.levels[l]),
            }),
        );
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("depth mips"),
            layout: &self.layout,
            entries: &entries,
        })
    }

    /// Fills the chain: one 8 x 8 workgroup per 16 x 16 pixels.
    pub fn encode(&self, encoder: &mut crate::Encoder, group: &wgpu::BindGroup, size: (u32, u32)) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(DEPTH_MIPS_TARGET),
            timestamp_writes: None,
        });
        pass.set_pipeline(self.pipeline.wait());
        pass.set_bind_group(0, group, &[]);
        pass.dispatch_workgroups(size.0.div_ceil(16), size.1.div_ceil(16), 1);
    }
}

const COLOR_MIPS_WGSL: &str = wgsl_library!("color_mips.wgsl");

/// Prefiltered half..sixteenth colour pyramid levels written in one dispatch.
pub(crate) const COLOR_MIP_LEVELS: u32 = 4;
const COLOR_MIPS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// The colour pyramid module's source, for validation (`shaders.rs`).
pub(crate) fn color_source() -> String {
    COLOR_MIPS_WGSL.to_string()
}

/// Prefiltered colour pyramid of the previous frame's lit image for SSGI.
pub(crate) struct ColorMips {
    source_size: (u32, u32),
    view: wgpu::TextureView,
    levels: Vec<wgpu::TextureView>,
}

impl ColorMips {
    pub fn new(device: &wgpu::Device, (width, height): (u32, u32)) -> ColorMips {
        let source_size = (width.max(16), height.max(16));
        let half = (
            source_size.0.div_ceil(2).max(8),
            source_size.1.div_ceil(2).max(8),
        );
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("color mips"),
            size: wgpu::Extent3d {
                width: half.0,
                height: half.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: COLOR_MIP_LEVELS,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: COLOR_MIPS_FORMAT,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
            texture_binding_view_dimension: None,
        });
        ColorMips {
            source_size,
            view: texture.create_view(&Default::default()),
            levels: crate::mips::level_views(&texture, "color mips level"),
        }
    }

    pub fn source_size(&self) -> (u32, u32) {
        self.source_size
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

/// Single-dispatch LDS colour pyramid pass (`color_mips.wgsl`).
pub(crate) struct ColorMipsPass {
    pub layout: wgpu::BindGroupLayout,
    pipeline: Deferred<wgpu::ComputePipeline>,
}

impl ColorMipsPass {
    pub fn new(device: &wgpu::Device) -> ColorMipsPass {
        let entries: Vec<_> = [B::Tex(false)]
            .into_iter()
            .chain((0..COLOR_MIP_LEVELS).map(|_| B::Storage(COLOR_MIPS_FORMAT)))
            .enumerate()
            .map(|(i, b)| wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: fsr::binding_type(b),
                count: None,
            })
            .collect();
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("color mips"),
            entries: &entries,
        });
        let module = crate::deferred::module(device, "color mips", COLOR_MIPS_WGSL.to_string());
        let pipeline = {
            let (device, layout) = (device.clone(), layout.clone());
            Deferred::new(move || {
                let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("color mips"),
                    bind_group_layouts: &[Some(&layout)],
                    immediate_size: 0,
                });
                crate::traffic::create_compute_pipeline(
                    &device,
                    &wgpu::ComputePipelineDescriptor {
                        label: Some("color mips"),
                        layout: Some(&pl),
                        module: module.get(),
                        entry_point: Some("cs_color_mips"),
                        compilation_options: Default::default(),
                        cache: crate::pipeline_cache::of(&device).as_ref(),
                    },
                )
            })
        };
        ColorMipsPass { layout, pipeline }
    }

    pub fn bind(
        &self,
        device: &wgpu::Device,
        source: &wgpu::TextureView,
        mips: &ColorMips,
    ) -> wgpu::BindGroup {
        let mut entries = vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(source),
        }];
        entries.extend(
            (0..COLOR_MIP_LEVELS as usize).map(|l| wgpu::BindGroupEntry {
                binding: 1 + l as u32,
                resource: wgpu::BindingResource::TextureView(&mips.levels[l]),
            }),
        );
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("color mips"),
            layout: &self.layout,
            entries: &entries,
        })
    }

    pub fn encode(&self, encoder: &mut crate::Encoder, group: &wgpu::BindGroup, size: (u32, u32)) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("color-mips"),
            timestamp_writes: None,
        });
        pass.set_pipeline(self.pipeline.wait());
        pass.set_bind_group(0, group, &[]);
        pass.dispatch_workgroups(size.0.div_ceil(16), size.1.div_ceil(16), 1);
    }
}
