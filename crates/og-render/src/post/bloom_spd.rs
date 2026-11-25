//! Compute LDS Single-Pass Downsample (SPD) Bloom (`bloom_spd.wgsl`):
//! replaces up to 11 fragment bloom passes with one compute prefilter + 4-level
//! LDS downsample pass (plus a tiny tail dispatch when 5..6 levels are active)
//! and one compute tent-upsample pass.

use super::fsr::{self, B};
use crate::deferred::Deferred;

const WGSL: &str = concat!(
    wgsl_library!("fullscreen.wgsl"),
    wgsl!("bloom_chain.wgsl"),
    wgsl!("bloom_spd.wgsl"),
);
const WGSL_F16: &str = concat!(
    "enable f16;\n",
    wgsl_library!("fullscreen.wgsl"),
    wgsl!("bloom_chain.wgsl"),
    wgsl!("bloom_spd_f16.wgsl"),
);

const BLOOM_SPD_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

pub(crate) const BLOOM_DOWN_TARGET: &str = "bloom-down";
pub(crate) const BLOOM_UP_TARGET: &str = "bloom-up";

pub(crate) fn source() -> String {
    WGSL.to_string()
}

pub(crate) fn source_f16() -> String {
    WGSL_F16.to_string()
}

/// Persistent compute bloom pyramids at the current post resolution.
/// `down` holds the prefiltered half-res level (`down[0]`, also read by lens
/// flare) and downsampled mips `1..levels`. `up_even` and `up_odd` alternate
/// storage writes during upsampling so no compute pass ever binds two mips of
/// the same texture for read + write simultaneously.
pub(crate) struct BloomSpd {
    post_size: (u32, u32),
    half_size: (u32, u32),
    active_levels: u32,
    down_levels: Vec<wgpu::TextureView>,
    up_even_levels: Vec<wgpu::TextureView>,
    up_odd_levels: Vec<wgpu::TextureView>,
    dummy_storage: wgpu::TextureView,
}

impl BloomSpd {
    pub fn new(device: &wgpu::Device, post_size: (u32, u32)) -> BloomSpd {
        let active_levels = crate::post::bloom_levels(post_size.0, post_size.1);
        let half_size = ((post_size.0 / 2).max(8), (post_size.1 / 2).max(8));
        let down_mips = active_levels.max(4);
        let make_tex = |label: &str, (w, h): (u32, u32), mips: u32| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: mips,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: BLOOM_SPD_FORMAT,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
                texture_binding_view_dimension: None,
            })
        };
        let down = make_tex("bloom spd down", half_size, down_mips);
        let up_even = make_tex("bloom spd up even", half_size, down_mips);
        let odd_size = ((half_size.0 / 2).max(4), (half_size.1 / 2).max(4));
        let up_odd = make_tex("bloom spd up odd", odd_size, (down_mips - 1).max(1));
        let dummy = make_tex("bloom spd dummy", (1, 1), 1);
        BloomSpd {
            post_size,
            half_size,
            active_levels,
            down_levels: crate::mips::level_views(&down, "bloom spd down level"),
            up_even_levels: crate::mips::level_views(&up_even, "bloom spd up even level"),
            up_odd_levels: crate::mips::level_views(&up_odd, "bloom spd up odd level"),
            dummy_storage: dummy.create_view(&Default::default()),
        }
    }

    pub fn post_size(&self) -> (u32, u32) {
        self.post_size
    }

    pub fn half_size(&self) -> (u32, u32) {
        self.half_size
    }

    /// Prefiltered half-resolution level (read by lens flare before upsampling).
    pub fn prefilter_view(&self) -> &wgpu::TextureView {
        &self.down_levels[0]
    }

    /// Final upsampled bloom level (read by tonemap).
    pub fn output_view(&self) -> &wgpu::TextureView {
        if self.active_levels <= 1 {
            &self.down_levels[0]
        } else {
            &self.up_even_levels[0]
        }
    }

    fn level_size(&self, level: u32) -> (u32, u32) {
        (
            (self.half_size.0 >> level).max(1),
            (self.half_size.1 >> level).max(1),
        )
    }

    fn up_view(&self, level: u32) -> &wgpu::TextureView {
        if level.is_multiple_of(2) {
            &self.up_even_levels[level as usize]
        } else {
            &self.up_odd_levels[(level - 1) as usize]
        }
    }
}

pub(crate) struct BloomSpdPass {
    down_layout: wgpu::BindGroupLayout,
    tail_layout: wgpu::BindGroupLayout,
    up_layout: wgpu::BindGroupLayout,
    spd: Deferred<wgpu::ComputePipeline>,
    tail: Deferred<wgpu::ComputePipeline>,
    up: Deferred<wgpu::ComputePipeline>,
}

impl BloomSpdPass {
    pub fn new(device: &wgpu::Device) -> BloomSpdPass {
        let make_layout = |label: &str, entries: &[(u32, B)]| {
            let list: Vec<_> = entries
                .iter()
                .map(|&(binding, b)| wgpu::BindGroupLayoutEntry {
                    binding,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: fsr::binding_type(b),
                    count: None,
                })
                .collect();
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(label),
                entries: &list,
            })
        };
        let down_layout = make_layout(
            "bloom spd down",
            &[
                (0, B::Uniform),
                (1, B::Sampler),
                (2, B::Tex(true)),
                (3, B::Tex(false)),
                (4, B::Tex(true)),
                (5, B::Storage(BLOOM_SPD_FORMAT)),
                (6, B::Storage(BLOOM_SPD_FORMAT)),
                (7, B::Storage(BLOOM_SPD_FORMAT)),
                (8, B::Storage(BLOOM_SPD_FORMAT)),
            ],
        );
        let tail_layout = make_layout(
            "bloom spd tail",
            &[
                (1, B::Sampler),
                (2, B::Tex(true)),
                (5, B::Storage(BLOOM_SPD_FORMAT)),
                (6, B::Storage(BLOOM_SPD_FORMAT)),
            ],
        );
        let up_layout = make_layout(
            "bloom spd up",
            &[
                (0, B::Uniform),
                (1, B::Sampler),
                (2, B::Tex(false)),
                (3, B::Tex(true)),
                (5, B::Storage(BLOOM_SPD_FORMAT)),
            ],
        );
        let wgsl = if device.features().contains(wgpu::Features::SHADER_F16) {
            WGSL_F16
        } else {
            WGSL
        };
        let module = crate::deferred::module(device, "bloom spd", wgsl.to_string());
        let make_pipe =
            |label: &'static str, layout: &wgpu::BindGroupLayout, entry: &'static str| {
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
        let spd = make_pipe("bloom spd", &down_layout, "cs_bloom_spd");
        let tail = make_pipe("bloom spd tail", &tail_layout, "cs_bloom_spd_tail");
        let up = make_pipe("bloom spd up", &up_layout, "cs_bloom_up");
        BloomSpdPass {
            down_layout,
            tail_layout,
            up_layout,
            spd,
            tail,
            up,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn bind_down(
        &self,
        device: &wgpu::Device,
        post_buffer: &wgpu::Buffer,
        sampler: &wgpu::Sampler,
        source: &wgpu::TextureView,
        luminance: &wgpu::TextureView,
        glow: &wgpu::TextureView,
        spd: &BloomSpd,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bloom spd down"),
            layout: &self.down_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: post_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(source),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(luminance),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(glow),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(&spd.down_levels[0]),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&spd.down_levels[1]),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::TextureView(&spd.down_levels[2]),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::TextureView(&spd.down_levels[3]),
                },
            ],
        })
    }

    pub fn bind_tail(
        &self,
        device: &wgpu::Device,
        sampler: &wgpu::Sampler,
        spd: &BloomSpd,
    ) -> Option<wgpu::BindGroup> {
        if spd.active_levels <= 4 {
            return None;
        }
        let l5 = spd.down_levels.get(5).unwrap_or(&spd.dummy_storage);
        Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bloom spd tail"),
            layout: &self.tail_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&spd.down_levels[3]),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(&spd.down_levels[4]),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(l5),
                },
            ],
        }))
    }

    pub fn bind_ups(
        &self,
        device: &wgpu::Device,
        post_buffer: &wgpu::Buffer,
        sampler: &wgpu::Sampler,
        spd: &BloomSpd,
    ) -> Vec<(wgpu::BindGroup, (u32, u32))> {
        if spd.active_levels <= 1 {
            return Vec::new();
        }
        let last = spd.active_levels - 2;
        (0..=last)
            .rev()
            .map(|i| {
                let lower = if i == last {
                    &spd.down_levels[(i + 1) as usize]
                } else {
                    spd.up_view(i + 1)
                };
                let dst = spd.up_view(i);
                let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("bloom spd up"),
                    layout: &self.up_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: post_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(
                                &spd.down_levels[i as usize],
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(lower),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: wgpu::BindingResource::TextureView(dst),
                        },
                    ],
                });
                (bg, spd.level_size(i))
            })
            .collect()
    }

    pub fn encode_down(
        &self,
        encoder: &mut crate::Encoder,
        down: &wgpu::BindGroup,
        tail: Option<&wgpu::BindGroup>,
        spd: &BloomSpd,
    ) -> u32 {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("bloom spd"),
            timestamp_writes: None,
        });
        let (hw, hh) = spd.half_size();
        pass.set_pipeline(self.spd.wait());
        pass.set_bind_group(0, down, &[]);
        pass.dispatch_workgroups(hw.div_ceil(8), hh.div_ceil(8), 1);
        let mut dispatches = 1;
        if let Some(t) = tail {
            let (tw, th) = spd.level_size(4);
            pass.set_pipeline(self.tail.wait());
            pass.set_bind_group(0, t, &[]);
            pass.dispatch_workgroups(tw.div_ceil(8), th.div_ceil(8), 1);
            dispatches += 1;
        }
        dispatches
    }

    pub fn encode_up(
        &self,
        encoder: &mut crate::Encoder,
        ups: &[(wgpu::BindGroup, (u32, u32))],
    ) -> u32 {
        if ups.is_empty() {
            return 0;
        }
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("bloom spd up"),
            timestamp_writes: None,
        });
        pass.set_pipeline(self.up.wait());
        for (bg, (w, h)) in ups {
            pass.set_bind_group(0, bg, &[]);
            pass.dispatch_workgroups(w.div_ceil(8), h.div_ceil(8), 1);
        }
        ups.len() as u32
    }
}
