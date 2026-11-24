//! Half-resolution soft particle depth downsample and depth-aware bilateral
//! upsample (`shaders/fx_upsample.wgsl`).
//!
//! Large soft-depth billboard clouds/smoke/fireballs draw into a half-res HDR
//! target (`fx.half`, premultiplied radiance in `.rgb` and remaining background
//! transmittance in `.a`) against a 2x2 max/min linear depth target
//! (`fx.half_depth`), cutting overdraw fillrate 4x while preserving crisp depth
//! silhouettes on composite onto the full-resolution HDR target.

use crate::deferred::{Deferred, Sampled};
use std::sync::Mutex;

pub(crate) const FX_UPSAMPLE_WGSL: &str = wgsl!("fx_upsample.wgsl");

/// Keeps the HDR target's alpha (`tonemapped` mask) untouched.
const KEEP_ALPHA: wgpu::BlendComponent = wgpu::BlendComponent {
    src_factor: wgpu::BlendFactor::Zero,
    dst_factor: wgpu::BlendFactor::One,
    operation: wgpu::BlendOperation::Add,
};

/// Premultiplied radiance + transmittance composite (`C + hdr * T`).
const COMPOSITE_BLEND: wgpu::BlendState = wgpu::BlendState {
    color: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::SrcAlpha,
        operation: wgpu::BlendOperation::Add,
    },
    alpha: KEEP_ALPHA,
};

type DownsampleKey = wgpu::TextureView;
type UpsampleKey = (wgpu::TextureView, wgpu::TextureView, wgpu::TextureView);

pub(crate) struct HalfResFx {
    downsample_layout: wgpu::BindGroupLayout,
    upsample_layout: wgpu::BindGroupLayout,
    downsample: Deferred<wgpu::RenderPipeline>,
    pub(crate) upsample: Sampled<wgpu::RenderPipeline>,
    reactive: Deferred<wgpu::RenderPipeline>,
    downsample_cache: Mutex<Option<(DownsampleKey, wgpu::BindGroup)>>,
    upsample_cache: Mutex<Option<(UpsampleKey, wgpu::BindGroup)>>,
}

fn unfilterable_2d(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

impl HalfResFx {
    pub(crate) fn new(device: &wgpu::Device, samples: u32) -> Self {
        let module = crate::deferred::module(device, "fx upsample", FX_UPSAMPLE_WGSL.to_string());
        let downsample_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fx half depth downsample"),
            entries: &[unfilterable_2d(0)],
        });
        let upsample_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fx half bilateral upsample"),
            entries: &[unfilterable_2d(0), unfilterable_2d(1), unfilterable_2d(2)],
        });
        let downsample = {
            let (device, layout, module) =
                (device.clone(), downsample_layout.clone(), module.clone());
            Deferred::anywhere(move || {
                let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("fx half depth downsample"),
                    bind_group_layouts: &[Some(&layout)],
                    immediate_size: 0,
                });
                crate::traffic::create_render_pipeline(
                    &device,
                    &wgpu::RenderPipelineDescriptor {
                        label: Some("fx half depth downsample"),
                        layout: Some(&pl),
                        vertex: wgpu::VertexState {
                            module: module.get(),
                            entry_point: Some("vs"),
                            compilation_options: Default::default(),
                            buffers: &[],
                        },
                        primitive: wgpu::PrimitiveState::default(),
                        depth_stencil: None,
                        multisample: wgpu::MultisampleState::default(),
                        fragment: Some(wgpu::FragmentState {
                            module: module.get(),
                            entry_point: Some("fs_downsample"),
                            compilation_options: Default::default(),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: wgpu::TextureFormat::Rg32Float,
                                blend: None,
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        multiview_mask: None,
                        cache: crate::pipeline_cache::of(&device).as_ref(),
                    },
                )
            })
        };
        let upsample_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fx half bilateral upsample"),
            bind_group_layouts: &[Some(&upsample_layout)],
            immediate_size: 0,
        });
        let upsample = {
            let (device, pl, module) = (device.clone(), upsample_pl.clone(), module.clone());
            Sampled::anywhere(samples, move |samples| {
                crate::hdr_pipeline(
                    &device,
                    crate::HdrPipeline {
                        alpha_to_coverage: false,
                        label: "fx half bilateral upsample",
                        layout: &pl,
                        module: module.get(),
                        vs: "vs",
                        fs: "fs_upsample",
                        buffers: &[],
                        cull: None,
                        depth_write: false,
                        depth_compare: wgpu::CompareFunction::Always,
                        blend: Some(COMPOSITE_BLEND),
                        samples,
                    },
                )
            })
        };
        let reactive = {
            let (device, pl, module) = (device.clone(), upsample_pl, module);
            Deferred::anywhere(move || {
                crate::traffic::create_render_pipeline(
                    &device,
                    &wgpu::RenderPipelineDescriptor {
                        label: Some("fx half reactive"),
                        layout: Some(&pl),
                        vertex: wgpu::VertexState {
                            module: module.get(),
                            entry_point: Some("vs"),
                            compilation_options: Default::default(),
                            buffers: &[],
                        },
                        primitive: wgpu::PrimitiveState::default(),
                        depth_stencil: None,
                        multisample: wgpu::MultisampleState::default(),
                        fragment: Some(wgpu::FragmentState {
                            module: module.get(),
                            entry_point: Some("fs_reactive"),
                            compilation_options: Default::default(),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: crate::oit::REVEAL_FORMAT,
                                blend: Some(wgpu::BlendState {
                                    color: wgpu::BlendComponent {
                                        src_factor: wgpu::BlendFactor::Zero,
                                        dst_factor: wgpu::BlendFactor::Src,
                                        operation: wgpu::BlendOperation::Add,
                                    },
                                    alpha: KEEP_ALPHA,
                                }),
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        multiview_mask: None,
                        cache: crate::pipeline_cache::of(&device).as_ref(),
                    },
                )
            })
        };
        HalfResFx {
            downsample_layout,
            upsample_layout,
            downsample,
            upsample,
            reactive,
            downsample_cache: Mutex::new(None),
            upsample_cache: Mutex::new(None),
        }
    }

    pub(crate) fn start(&self) {
        crate::deferred::Start::start(&self.downsample);
        crate::deferred::Start::start(&self.upsample);
        crate::deferred::Start::start(&self.reactive);
    }

    pub(crate) fn set_samples(&self, samples: u32) {
        self.upsample.set_samples(samples);
    }

    fn upsample_group(
        &self,
        device: &wgpu::Device,
        full_depth: &wgpu::TextureView,
        fx_half: &wgpu::TextureView,
        half_depth: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        let key = (full_depth.clone(), fx_half.clone(), half_depth.clone());
        let mut cache = self.upsample_cache.lock().expect("fx half upsample");
        if let Some((k, g)) = &*cache
            && *k == key
        {
            g.clone()
        } else {
            let g = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fx half bilateral upsample"),
                layout: &self.upsample_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(full_depth),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(fx_half),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(half_depth),
                    },
                ],
            });
            *cache = Some((key, g.clone()));
            g
        }
    }

    /// Downsamples full-resolution `full_depth` (`lindepth`) into half-res
    /// `half_depth` (`fx.half_depth`, storing `(max_d, min_d)`).
    pub(crate) fn encode_downsample(
        &self,
        device: &wgpu::Device,
        encoder: &mut crate::Encoder,
        full_depth: &wgpu::TextureView,
        half_depth: &wgpu::TextureView,
    ) -> u32 {
        let Some(pipeline) = self.downsample.get() else {
            return 0;
        };
        let group = {
            let mut cache = self.downsample_cache.lock().expect("fx half downsample");
            if let Some((k, g)) = &*cache
                && *k == *full_depth
            {
                g.clone()
            } else {
                let g = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("fx half depth downsample"),
                    layout: &self.downsample_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(full_depth),
                    }],
                });
                *cache = Some((full_depth.clone(), g.clone()));
                g
            }
        };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("fx half depth"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: half_depth,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::DontCare(unsafe { wgpu::LoadOpDontCare::enabled() }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.draw(0..3, 0..1);
        1
    }

    /// Composites half-res `fx_half` onto the active HDR render pass using
    /// depth-aware bilateral weights against `full_depth` and `half_depth`.
    pub(crate) fn composite(
        &self,
        device: &wgpu::Device,
        pass: &mut wgpu::RenderPass<'_>,
        full_depth: &wgpu::TextureView,
        fx_half: &wgpu::TextureView,
        half_depth: &wgpu::TextureView,
    ) -> u32 {
        let Some(pipeline) = self.upsample.get() else {
            return 0;
        };
        let group = self.upsample_group(device, full_depth, fx_half, half_depth);
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.draw(0..3, 0..1);
        1
    }

    /// Multiplies half-res soft FX reactivity into `target` (`oit.reveal` when
    /// `clear == false`, or `fx.reactive` cleared to `(1, 1)` when `clear == true`)
    /// so TAA and FSR 3.1 suppress ghosting over fast additive/soft FX.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn encode_reactive(
        &self,
        device: &wgpu::Device,
        encoder: &mut crate::Encoder,
        full_depth: &wgpu::TextureView,
        fx_half: &wgpu::TextureView,
        half_depth: &wgpu::TextureView,
        target: &wgpu::TextureView,
        clear: bool,
    ) -> u32 {
        let Some(pipeline) = self.reactive.get() else {
            return 0;
        };
        let group = self.upsample_group(device, full_depth, fx_half, half_depth);
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("fx half reactive"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: if clear {
                        wgpu::LoadOp::Clear(wgpu::Color::WHITE)
                    } else {
                        wgpu::LoadOp::Load
                    },
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.draw(0..3, 0..1);
        1
    }
}
