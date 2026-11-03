//! The FFT sea (R5, the owner's direction of 2026-11-03): og.Water's wind
//! sea drawn from Tessendorf's FFT of a seeded JONSWAP spectrum in up to
//! three 256² cascades on disjoint bands, the tick's long waves drawn as
//! they are over it.
//!
//! - [`spectrum`]: the spectrum, built on the CPU once per sea (ported from
//!   blue-water's `spectrum.rs`), without the band the tick's long waves
//!   carry and rescaled so the whole is the sea's significant height.
//! - `shaders/ocean_fft.wgsl`: each frame, the spectrum evolved to the
//!   scene's tick time and inverse-transformed in workgroup memory (rows,
//!   then columns, after web-ocean's `sim.js`) into one rgba16float array,
//!   three layers per cascade; `shaders/ocean_fft_mips.wgsl` its mips.
//!
//! - [`foam`]: the persistent foam (`shaders/ocean_foam.wgsl`), stepped
//!   on the tick's time into the fields array's layers after the cascades.
//!
//! Render only: the tick never reads these fields (`og.physics.waterHeight`
//! and og.Buoyancy are og_core::ocean's, evaluated in the tick). Every pass
//! here is compute with its own bind groups, outside the lit layouts; the
//! water pass will sample the fields array in place of its height field.

pub mod foam;
pub mod spectrum;

use spectrum::Spectrum;

/// Texels across a cascade.
pub const N: u32 = 256;
/// Layers per cascade in the fields array.
pub const LAYERS: u32 = 3;
/// Mip levels of the fields (256 down to 16).
pub const MIPS: u32 = 5;
/// Layers after the cascades': the foam's two levels, then its
/// micro-structure ([`foam`]).
pub const FOAM_LAYERS: u32 = 3;

/// The first foam layer of a fields array of `cascades` cascades.
pub fn foam_layer(cascades: u32) -> u32 {
    LAYERS * cascades
}

const FFT_WGSL: &str = wgsl3d!("ocean_fft.wgsl");
const MIPS_WGSL: &str = wgsl3d!("ocean_fft_mips.wgsl");
const FOAM_WGSL: &str = wgsl3d!("ocean_foam.wgsl");

/// The WGSL modules of this pass, for the shader check.
pub fn sources() -> [(&'static str, &'static str); 3] {
    [
        ("ocean fft", FFT_WGSL),
        ("ocean fft mips", MIPS_WGSL),
        ("ocean foam", FOAM_WGSL),
    ]
}

/// `FftParams` in ocean_fft.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct FftGpu {
    tiles: [u32; 4],
    time: [f32; 4],
    dk: [f32; 4],
}

/// The phase of `seconds` in the fold period, in f64 (so a long game keeps
/// its precision): what the shader multiplies each wave's frequency by.
pub fn phase(seconds: f64) -> f64 {
    seconds.rem_euclid(spectrum::FOLD_PERIOD) / spectrum::FOLD_PERIOD
}

/// The GPU half: the spectrum's buffer, the transform's two intermediate
/// buffers, the fields array and the passes that fill it.
pub struct FftOcean {
    cascades: u32,
    uniform: wgpu::Buffer,
    spectrum: wgpu::Buffer,
    rows_group: wgpu::BindGroup,
    columns_group: wgpu::BindGroup,
    mip_groups: Vec<wgpu::BindGroup>,
    rows: wgpu::ComputePipeline,
    columns: wgpu::ComputePipeline,
    mips: wgpu::ComputePipeline,
    fields: wgpu::Texture,
    /// The whole array, every mip: what the water pass samples.
    pub view: wgpu::TextureView,
    dk: [f32; 4],
    lambda: f32,
    foam: Foam,
}

/// The foam's passes and its history between steps ([`foam`]).
struct Foam {
    uniform: wgpu::Buffer,
    /// The last step's levels (copied from the fields' foam layers), read by
    /// the next.
    history: wgpu::Texture,
    update: wgpu::ComputePipeline,
    update_group: wgpu::BindGroup,
    detail: wgpu::ComputePipeline,
    detail_group: wgpu::BindGroup,
    /// The micro-structure is made with the first step.
    detail_made: bool,
    /// The last step (an index of [`foam::STEP`]) and the camera's texels
    /// then; None until the first (or after a restart).
    last: Option<(i64, [[i32; 2]; 2])>,
}

fn storage(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn storage_texture(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::WriteOnly,
            format: wgpu::TextureFormat::Rgba16Float,
            view_dimension: wgpu::TextureViewDimension::D2Array,
        },
        count: None,
    }
}

impl FftOcean {
    /// The passes for `cascades` tiles (1 to 3).
    pub fn new(device: &wgpu::Device, cascades: u32) -> FftOcean {
        let cascades = cascades.clamp(1, 3);
        let texels = u64::from(N * N * cascades);
        let buffer = |label, size: u64, usage| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage,
                mapped_at_creation: false,
            })
        };
        let uniform = buffer(
            "ocean fft",
            std::mem::size_of::<FftGpu>() as u64,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );
        let spectrum = buffer(
            "ocean fft spectrum",
            texels * 16,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        );
        let row_a = buffer("ocean fft a", texels * 16, wgpu::BufferUsages::STORAGE);
        let row_b = buffer("ocean fft b", texels * 16, wgpu::BufferUsages::STORAGE);
        let fields = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ocean fields"),
            size: wgpu::Extent3d {
                width: N,
                height: N,
                depth_or_array_layers: LAYERS * cascades + FOAM_LAYERS,
            },
            mip_level_count: MIPS,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let level = |mip: u32| {
            fields.create_view(&wgpu::TextureViewDescriptor {
                label: Some("ocean fields level"),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                base_mip_level: mip,
                mip_level_count: Some(1),
                ..Default::default()
            })
        };
        let view = fields.create_view(&wgpu::TextureViewDescriptor {
            label: Some("ocean fields"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        let uniform_entry = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let rows_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ocean fft rows"),
            entries: &[
                uniform_entry,
                storage(1, true),
                storage(2, false),
                storage(3, false),
            ],
        });
        let columns_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ocean fft columns"),
            entries: &[
                uniform_entry,
                storage(4, true),
                storage(5, true),
                storage_texture(6),
            ],
        });
        let mips_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ocean fft mips"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                storage_texture(1),
            ],
        });
        let module = |label: &'static str, source: &str| {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            })
        };
        let fft = module("ocean fft", FFT_WGSL);
        let mips_module = module("ocean fft mips", MIPS_WGSL);
        let pipeline = |label: &'static str,
                        layout: &wgpu::BindGroupLayout,
                        m: &wgpu::ShaderModule,
                        entry: &'static str| {
            let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &[Some(layout)],
                immediate_size: 0,
            });
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(label),
                layout: Some(&pl),
                module: m,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: None,
            })
        };
        let rows = pipeline("ocean fft rows", &rows_layout, &fft, "rows");
        let columns = pipeline("ocean fft columns", &columns_layout, &fft, "columns");
        let mips = pipeline("ocean fft mips", &mips_layout, &mips_module, "main");
        let foam = Foam::new(
            device,
            &fields,
            cascades,
            &uniform_entry,
            &pipeline,
            &module,
        );
        let rows_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ocean fft rows"),
            layout: &rows_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: spectrum.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: row_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: row_b.as_entire_binding(),
                },
            ],
        });
        let level0 = level(0);
        let columns_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ocean fft columns"),
            layout: &columns_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: row_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: row_b.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&level0),
                },
            ],
        });
        let mip_groups = (1..MIPS)
            .map(|m| {
                let (src, dst) = (level(m - 1), level(m));
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("ocean fft mips"),
                    layout: &mips_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&src),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&dst),
                        },
                    ],
                })
            })
            .collect();
        FftOcean {
            cascades,
            uniform,
            spectrum,
            rows_group,
            columns_group,
            mip_groups,
            rows,
            columns,
            mips,
            fields,
            view,
            dk: [0.0; 4],
            lambda: 0.0,
            foam,
        }
    }

    /// The fields array.
    pub fn texture(&self) -> &wgpu::Texture {
        &self.fields
    }

    /// How many cascades the fields hold.
    pub fn cascades(&self) -> u32 {
        self.cascades
    }

    /// Uploads `spectrum`'s amplitudes (its first [`FftOcean::cascades`]
    /// cascades; missing ones are calm).
    pub fn set_spectrum(&mut self, queue: &wgpu::Queue, spectrum: &Spectrum) {
        let per = (N * N * 4) as usize;
        let mut data = vec![0f32; per * self.cascades as usize];
        self.dk = [0.0; 4];
        for (i, c) in spectrum
            .cascades
            .iter()
            .take(self.cascades as usize)
            .enumerate()
        {
            debug_assert_eq!(c.resolution, N);
            data[i * per..(i + 1) * per].copy_from_slice(&c.amplitudes);
            self.dk[i] = (std::f64::consts::TAU / c.size) as f32;
        }
        self.lambda = spectrum.choppiness as f32;
        queue.write_buffer(&self.spectrum, 0, bytemuck::cast_slice(&data));
        // A new sea's foam starts over.
        self.foam.last = None;
    }

    /// Steps the foam up to `seconds` of the scene's clock (its tick over
    /// its rate), then records the fields at `seconds` into `encoder`: each
    /// foam step is submitted on its own, after the fields at its own time
    /// ([`foam`]: the steps since the last frame, or the warm-up after a
    /// restart). `eye` is the camera's absolute world xz (the frame's plus
    /// `origin`), `swell` og_core's waves at a time (`WaterGpu::swell`).
    #[allow(clippy::too_many_arguments)]
    pub fn advance(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        seconds: f64,
        eye: glam::DVec2,
        origin: glam::DVec2,
        sea: &foam::FoamSea,
        swell: &dyn Fn(f64) -> [[f32; 4]; 2 * og_core::ocean::OCEAN_WAVES],
    ) {
        let now = (seconds / foam::STEP).floor() as i64;
        let steps = match self.foam.last {
            Some((last, _)) if now >= last && now - last <= foam::GAP => {
                ((last + 1)..=now).map(|k| (k, 1)).collect()
            }
            _ => {
                self.foam.last = None;
                foam::warm_steps(now)
            }
        };
        let snaps = foam::snaps(eye);
        for (k, span) in steps {
            let at = k as f64 * foam::STEP;
            let mut step = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ocean foam"),
            });
            if !self.foam.detail_made {
                let mut pass = step.begin_compute_pass(&Default::default());
                pass.set_pipeline(&self.foam.detail);
                pass.set_bind_group(0, &self.foam.detail_group, &[]);
                pass.dispatch_workgroups(N / 8, N / 8, 1);
                self.foam.detail_made = true;
            }
            self.encode(queue, &mut step, at);
            let u = foam::step_gpu(
                snaps,
                self.foam.last.map(|(_, s)| s),
                origin,
                (span as f64 * foam::STEP) as f32,
                self.cascades,
                sea,
                swell(at),
            );
            queue.write_buffer(&self.foam.uniform, 0, bytemuck::bytes_of(&u));
            {
                let mut pass = step.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("ocean foam"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&self.foam.update);
                pass.set_bind_group(0, &self.foam.update_group, &[]);
                pass.dispatch_workgroups(foam::RES / 8, foam::RES / 8, 2);
            }
            let first = foam_layer(self.cascades);
            step.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.fields,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: first,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                self.foam.history.as_image_copy(),
                wgpu::Extent3d {
                    width: foam::RES,
                    height: foam::RES,
                    depth_or_array_layers: 2,
                },
            );
            queue.submit([step.finish()]);
            self.foam.last = Some((k, snaps));
        }
        self.encode(queue, encoder, seconds);
    }

    /// Records the fields at `seconds` of the scene's clock (its tick over
    /// its rate) into `encoder`: rows, columns, then the mips (of every
    /// layer, the foam's too).
    fn encode(&self, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, seconds: f64) {
        let u = FftGpu {
            tiles: [N, N.trailing_zeros(), self.cascades, 0],
            time: [phase(seconds) as f32, self.lambda, 0.0, 0.0],
            dk: self.dk,
        };
        queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&u));
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("ocean fft"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.rows);
        pass.set_bind_group(0, &self.rows_group, &[]);
        pass.dispatch_workgroups(N, 1, self.cascades);
        pass.set_pipeline(&self.columns);
        pass.set_bind_group(0, &self.columns_group, &[]);
        pass.dispatch_workgroups(N, 1, self.cascades);
        pass.set_pipeline(&self.mips);
        for (m, group) in self.mip_groups.iter().enumerate() {
            let size = (N >> (m + 1)).max(1);
            pass.set_bind_group(0, group, &[]);
            pass.dispatch_workgroups(
                size.div_ceil(8),
                size.div_ceil(8),
                LAYERS * self.cascades + FOAM_LAYERS,
            );
        }
    }
}

impl Foam {
    fn new(
        device: &wgpu::Device,
        fields: &wgpu::Texture,
        cascades: u32,
        uniform_entry: &wgpu::BindGroupLayoutEntry,
        pipeline: &dyn Fn(
            &'static str,
            &wgpu::BindGroupLayout,
            &wgpu::ShaderModule,
            &'static str,
        ) -> wgpu::ComputePipeline,
        module: &dyn Fn(&'static str, &str) -> wgpu::ShaderModule,
    ) -> Foam {
        let first = foam_layer(cascades);
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ocean foam"),
            size: std::mem::size_of::<foam::FoamGpu>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let history = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ocean foam history"),
            size: wgpu::Extent3d {
                width: foam::RES,
                height: foam::RES,
                depth_or_array_layers: 2,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = |label, dimension, base_mip_level, mips, base_array_layer, layers| {
            fields.create_view(&wgpu::TextureViewDescriptor {
                label: Some(label),
                dimension: Some(dimension),
                base_mip_level,
                mip_level_count: mips,
                base_array_layer,
                array_layer_count: Some(layers),
                ..Default::default()
            })
        };
        // The update samples the cascades' layers (every mip) while it writes
        // the foam's (mip 0): disjoint parts of the one array.
        let waves = view(
            "ocean foam waves",
            wgpu::TextureViewDimension::D2Array,
            0,
            None,
            0,
            first,
        );
        let out = view(
            "ocean foam out",
            wgpu::TextureViewDimension::D2Array,
            0,
            Some(1),
            first,
            2,
        );
        let detail_out = view(
            "ocean foam detail",
            wgpu::TextureViewDimension::D2,
            0,
            Some(1),
            first + 2,
            1,
        );
        let sampled = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2Array,
                multisampled: false,
            },
            count: None,
        };
        let update_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ocean foam"),
            entries: &[
                *uniform_entry,
                sampled(1),
                sampled(2),
                storage_texture(3),
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let detail_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ocean foam detail"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba16Float,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            }],
        });
        // Tiled (the levels are toroidal), trilinear.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ocean foam"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let m = module("ocean foam", FOAM_WGSL);
        let update = pipeline("ocean foam", &update_layout, &m, "update");
        let detail = pipeline("ocean foam detail", &detail_layout, &m, "detail");
        let history_view = history.create_view(&wgpu::TextureViewDescriptor {
            label: Some("ocean foam history"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        let update_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ocean foam"),
            layout: &update_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&waves),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&history_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&out),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        let detail_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ocean foam detail"),
            layout: &detail_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(&detail_out),
            }],
        });
        Foam {
            uniform,
            history,
            update,
            update_group,
            detail,
            detail_group,
            detail_made: false,
            last: None,
        }
    }
}
