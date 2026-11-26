//! Compute 256-bin luminance histogram auto-exposure
//! (`exposure_hist.wgsl`): replaces the 3-pass fragment meter + reductions
//! with a single compute pass (histogram + its mean over every bin, the
//! fragment meter's mean: nothing is trimmed).

use super::fsr::{self, B};
use crate::deferred::Deferred;

const WGSL: &str = wgsl_library!("exposure_hist.wgsl");
const WGSL_SG_BODY: &str = wgsl_library!("exposure_hist_sg.wgsl");

/// Storage format of the 1x1 metered luminance target (`Rgba16Float` is
/// universally valid as a WebGPU Compat write-only storage texture and
/// matches `fullscreen.wgsl`'s `metered_log` r/g layout).
pub(crate) const HIST_METER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

pub(crate) fn source() -> String {
    WGSL.to_string()
}

pub(crate) fn source_sg() -> String {
    format!("{}{WGSL_SG_BODY}", crate::cull_gpu::SUBGROUP_HEADER)
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct HistParams {
    flags: [f32; 4],
}

pub(crate) struct ExposureHistPass {
    layout: wgpu::BindGroupLayout,
    histogram: Deferred<wgpu::ComputePipeline>,
    average: Deferred<wgpu::ComputePipeline>,
    params: wgpu::Buffer,
    bins: wgpu::Buffer,
    subgroup: bool,
}

impl ExposureHistPass {
    pub fn new(device: &wgpu::Device) -> ExposureHistPass {
        let subgroup = crate::cull_gpu::supports_subgroup_256(device);
        let bindings = [
            B::Uniform,
            B::Tex(false),
            B::Rw,
            B::Storage(HIST_METER_FORMAT),
        ];
        let entries: Vec<_> = bindings
            .iter()
            .enumerate()
            .map(|(i, &b)| wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: fsr::binding_type(b),
                count: None,
            })
            .collect();
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("exposure histogram"),
            entries: &entries,
        });
        let wgsl = if subgroup {
            source_sg()
        } else {
            WGSL.to_string()
        };
        let module = crate::deferred::module(device, "exposure histogram", wgsl);
        let make = |label: &'static str, entry: &'static str| {
            let (device, layout, module) = (device.clone(), layout.clone(), module.clone());
            Deferred::new(move || {
                let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(label),
                    bind_group_layouts: &[Some(&layout)],
                    immediate_size: 0,
                });
                crate::traffic::create_compute_pipeline(
                    &device,
                    &wgpu::ComputePipelineDescriptor {
                        label: Some(label),
                        layout: Some(&pl),
                        module: module.get(),
                        entry_point: Some(entry),
                        compilation_options: Default::default(),
                        cache: crate::pipeline_cache::of(&device).as_ref(),
                    },
                )
            })
        };
        let params = crate::traffic::create_buffer(
            device,
            &wgpu::BufferDescriptor {
                label: Some("exposure hist params"),
                size: std::mem::size_of::<HistParams>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );
        // Zero-initialized 256 x u32 atomic histogram; `cs_average` zeros it
        // back to 0 at the end of every frame via `atomicExchange`.
        let bins = crate::traffic::create_buffer(
            device,
            &wgpu::BufferDescriptor {
                label: Some("exposure hist bins"),
                size: 256 * 4,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            },
        );
        let histogram = make("exposure histogram", "cs_histogram");
        let average = make("exposure average", "cs_average");
        ExposureHistPass {
            layout,
            histogram,
            average,
            params,
            bins,
            subgroup,
        }
    }

    pub fn write_params(&self, queue: &wgpu::Queue, untonemapped: bool) {
        let u = HistParams {
            flags: [f32::from(u8::from(untonemapped)), 0.0, 0.0, 0.0],
        };
        crate::traffic::write_buffer(queue, &self.params, 0, bytemuck::bytes_of(&u));
    }

    pub fn bind(
        &self,
        device: &wgpu::Device,
        source: &wgpu::TextureView,
        metered_out: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("exposure histogram"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.params.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(source),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.bins.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(metered_out),
                },
            ],
        })
    }

    pub fn encode(
        &self,
        encoder: &mut crate::Encoder,
        group: &wgpu::BindGroup,
        size: (u32, u32),
    ) -> u32 {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("meter histogram"),
            timestamp_writes: None,
        });
        let tile_y = if self.subgroup { 32 } else { 16 };
        pass.set_pipeline(self.histogram.wait());
        pass.set_bind_group(0, group, &[]);
        pass.dispatch_workgroups(size.0.div_ceil(32), size.1.div_ceil(tile_y), 1);
        pass.set_pipeline(self.average.wait());
        pass.set_bind_group(0, group, &[]);
        pass.dispatch_workgroups(1, 1, 1);
        2
    }
}
