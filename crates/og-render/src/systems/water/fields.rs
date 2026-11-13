use super::*;

impl WaterRenderer {
    /// The FFT sea of the frame's first surface with a wind, its fields
    /// planned at `seconds` (the scene's tick time, so a frame is the
    /// tick's whatever the wall clock) for the frame's encoder
    /// ([`WaterRenderer::encode_fields`]): its spectrum rebuilt when that sea's wind, swell or choppiness or the
    /// cascades change. None without a sea.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn fft_sea(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        waters: &[Water],
        cascades: u32,
        seconds: f64,
        camera: &crate::Camera,
        origin: Vec3,
        caustic_grid: &wgpu::Buffer,
    ) -> Option<FftSea> {
        use crate::ocean_fft::FftOcean;
        let (water, sea) = waters.iter().find_map(|w| Some((w, ocean(w)?)))?;
        let key = (water.wind, water.swell, water.choppiness, cascades);
        if self.fft.as_ref().is_none_or(|(_, f)| f.key != key) {
            let mut fft = match self.fft.take() {
                Some((f, ..)) if f.cascades() == cascades => f,
                _ => FftOcean::new(device, cascades),
            };
            let built = sea_spectrum(water, &sea, cascades);
            fft.set_spectrum(queue, &built);
            let mut tiles = [0.0; 3];
            for (t, c) in tiles.iter_mut().zip(&built.cascades) {
                *t = c.size;
            }
            let mut bands = [0.0; 4];
            for (b, c) in bands.iter_mut().zip(&built.cascades) {
                *b = (c.band.lo.max(1e-6) * c.band.hi).sqrt() as f32;
            }
            let info = FftSea {
                key,
                bands,
                tiles,
                tail: built.tail_slope_variance as f32,
                foam: foam_thresholds(water, &built),
            };
            self.fft = Some((fft, info));
        }
        let (fft, info) = self.fft.as_mut().expect("made above");
        let info = *info;
        let foam_sea = crate::ocean_fft::foam::FoamSea {
            cascades: cascade_uv(Some(info), origin),
            bands: info.bands,
            wind: water.wind,
            thresholds: info.foam.laid,
            gains: [info.foam.gains[0], info.foam.gains[1]],
            least_gain: FOAM_GAIN,
            streaks: water.foam_streaks,
        };
        let world = |v: Vec3| glam::DVec2::new(f64::from(v.x), f64::from(v.z));
        let steps = fft.advance(
            device,
            queue,
            seconds,
            world(camera.position) + world(origin),
            world(origin),
            &foam_sea,
            &|at| ocean_gpu(water, at, origin).1,
            (shore_view(&self.shores, &self.no_shore, water), &|at| {
                shore_gpu(water, at)
            }),
        );
        // W2: the sea's curvature about the camera, for the caustics under it.
        if water.caustics > 0.0 && self.compute && !self.caustics_left_out {
            let pass = self
                .caustics
                .get_or_insert_with(|| CausticsPass::new(device));
            let params = CausticsGpu {
                grid: self::caustic_grid(camera.position, origin),
                sea: [fft.cascades() as f32, 0.0, 0.0, 0.0],
                cascades: cascade_uv(Some(info), origin),
                swell: ocean_gpu(water, seconds, origin).1,
            };
            pass.plan(device, queue, &fft.view, &params, caustic_grid);
        }
        self.foam_steps = steps as u32;
        Some(info)
    }

    /// Records the compute [`WaterRenderer::prepare`] planned, in the
    /// frame's encoder before any pass that draws water or reads the
    /// caustics: the sea's fields and foam, its caustics, the wakes, each
    /// rippling surface's ripples. Answers the GPU queries they wrote.
    pub(crate) fn encode_fields(
        &mut self,
        encoder: &mut crate::Encoder,
    ) -> Option<std::ops::Range<u32>> {
        let q0 = encoder.used();
        if let Some((fft, _)) = &mut self.fft {
            fft.encode(encoder);
        }
        if let Some(caustics) = &mut self.caustics {
            caustics.encode(encoder);
        }
        if let Some(wakes) = &mut self.wakes {
            wakes.encode(encoder);
        }
        if let Some(pipes) = &self.ripple_pipelines {
            for ripples in self.slots.iter_mut().filter_map(|s| s.ripples.as_mut()) {
                ripples.encode(encoder, pipes);
            }
        }
        (encoder.timed() && encoder.used() > q0).then(|| q0..encoder.used())
    }

    /// The drawn sea's height at `points` (frame xz) as the last frame drew
    /// its first sea (`probe_sea` in water_sea.wgsl): submitted, into a buffer to
    /// map, per point (the height, the tick's waves' share, the cascades'
    /// share, 0). None without a sea that frame.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub(crate) fn probe_sea(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        points: &[Vec2],
    ) -> Option<wgpu::Buffer> {
        use crate::deferred::AsModule;
        let group = self.last_sea.as_ref()?;
        let storage = |binding, read_only| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let io_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("water probe"),
            entries: &[storage(0, true), storage(1, false)],
        });
        let empty = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("water probe"),
            entries: &[],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("water probe"),
            bind_group_layouts: &[Some(&empty), Some(&self.layout), Some(&io_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("water probe"),
            layout: Some(&pl),
            module: self.module.module(),
            entry_point: Some("probe_sea"),
            compilation_options: Default::default(),
            cache: None,
        });
        let input: Vec<[f32; 2]> = points.iter().map(|p| p.to_array()).collect();
        let input = crate::buffer_init(
            device,
            queue,
            &wgpu::util::BufferInitDescriptor {
                label: Some("water probe in"),
                contents: bytemuck::cast_slice(&input),
                usage: wgpu::BufferUsages::STORAGE,
            },
        );
        let size = (points.len().max(1) * 16) as u64;
        let output = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("water probe out"),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let read = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("water probe read"),
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let io = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("water probe"),
            layout: &io_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: output.as_entire_binding(),
                },
            ],
        });
        let none = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("water probe"),
            layout: &empty,
            entries: &[],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("water probe"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &none, &[]);
            pass.set_bind_group(1, group, &[]);
            pass.set_bind_group(2, &io, &[]);
            pass.dispatch_workgroups((points.len() as u32).div_ceil(64), 1, 1);
        }
        encoder.copy_buffer_to_buffer(&output, 0, &read, 0, size);
        queue.submit([encoder.finish()]);
        Some(read)
    }
}
