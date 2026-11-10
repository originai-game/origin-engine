use super::*;

impl WaterRenderer {
    /// Starts compiling the pipelines `waters` draw with on worker threads
    /// (deferred.rs): the grid's or the mesh's, as each surface asks.
    /// `features`: the frame's (pipeline_features.rs).
    pub(crate) fn start(&self, waters: &[Water], features: Features) {
        if waters.iter().any(|w| w.mesh.is_some()) {
            self.mesh_pipeline.start(features);
        }
        if waters.iter().any(|w| w.mesh.is_none()) {
            self.pipeline.start(features);
        }
    }

    /// Makes its pipeline if no frame used it yet (Renderer::warm).
    pub(crate) fn warm(&self) -> bool {
        crate::deferred::warm_one(&[
            self.pipeline.full(),
            self.mesh_pipeline.full(),
            &self.spray_pipeline,
        ])
    }

    pub(crate) fn new(
        device: &wgpu::Device,
        frame_layout: &wgpu::BindGroupLayout,
        frame_wgsl: &str,
        lighting_wgsl: &str,
        samples: u32,
        compute: bool,
        caustics_left_out: bool,
    ) -> WaterRenderer {
        let tex = |binding, filterable| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("water"),
            entries: &[
                // (0, 1 and 7 are read by the height probe's compute too: `probe_sea`.)
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT
                        .union(wgpu::ShaderStages::COMPUTE),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // A height field's (a 1-layer rgba16float array, height in r) or the FFT sea's fields (ocean_fft:
                // three layers per cascade, mipped), sampled filtered.
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT
                        .union(wgpu::ShaderStages::COMPUTE),
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                tex(2, true),
                tex(3, false),
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                tex(5, true),
                tex(6, true),
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT
                        .union(wgpu::ShaderStages::COMPUTE),
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // The frame's screen-space AO (white without): the reflection is darkened by it as
                // the scene behind already is (water.wgsl).
                tex(8, true),
                // The flow map (Water::flow_map; the fallback without).
                tex(9, true),
                // The seabed (Water::shore; one sample without), read by the vertices and the height probe too.
                wgpu::BindGroupLayoutEntry {
                    binding: 10,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT
                        .union(wgpu::ShaderStages::COMPUTE),
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // W3: the sea's hulls' wakes (sea_wakes.rs; one calm texel without), read by the vertices and the
                // height probe too.
                wgpu::BindGroupLayoutEntry {
                    binding: 11,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT
                        .union(wgpu::ShaderStages::COMPUTE),
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let module =
            crate::deferred::module(device, "water", water_source(frame_wgsl, lighting_wgsl));
        let probe_module = module.clone();
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("water"),
            bind_group_layouts: &[Some(frame_layout), Some(&layout)],
            immediate_size: 0,
        });
        // The surface's lit pipelines, as the variant for a frame's features
        // and the full one (pipeline_features.rs).
        let lit =
            |label: &'static str,
             vs: &'static str,
             buffers: &'static [Option<wgpu::VertexBufferLayout<'static>>]| {
                let (device, pl, module) = (device.clone(), pl.clone(), module.clone());
                let declared = module.declared;
                Variants::new(declared, move |features| {
                    let (device, pl, module) = (device.clone(), pl.clone(), module.clone());
                    let constants = features.constants(declared);
                    crate::deferred::Sampled::anywhere(samples, move |samples| {
                        use crate::deferred::AsModule;
                        crate::hdr_pipeline_with(
                            &device,
                            &constants,
                            crate::HdrPipeline {
                                alpha_to_coverage: false,
                                label,
                                layout: &pl,
                                module: module.module(),
                                vs,
                                fs: "fs",
                                buffers,
                                cull: None,
                                depth_write: true,
                                depth_compare: wgpu::CompareFunction::Less,
                                blend: None,
                                samples,
                            },
                        )
                    })
                })
            };
        let mesh_pipeline = lit(
            "water mesh",
            "vs_mesh",
            &[Some(crate::POSITION_NORMAL_UV_LAYOUT)],
        );
        let pipeline = lit("water", "vs", &[]);
        let ao_depth_pipelines = {
            let (device, pl, module) = (device.clone(), pl.clone(), module.clone());
            crate::deferred::Deferred::new(move || {
                use crate::deferred::AsModule;
                let m = module.module();
                [
                    ("water ao depth", "vs", &[][..]),
                    (
                        "water mesh ao depth",
                        "vs_mesh",
                        &[Some(crate::POSITION_NORMAL_UV_LAYOUT)][..],
                    ),
                ]
                .map(|(label, vs, buffers)| {
                    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some(label),
                        layout: Some(&pl),
                        vertex: wgpu::VertexState {
                            module: m,
                            entry_point: Some(vs),
                            compilation_options: Default::default(),
                            buffers,
                        },
                        primitive: wgpu::PrimitiveState::default(),
                        depth_stencil: Some(wgpu::DepthStencilState {
                            format: crate::DEPTH_FORMAT,
                            depth_write_enabled: Some(false),
                            depth_compare: Some(wgpu::CompareFunction::Less),
                            stencil: Default::default(),
                            bias: Default::default(),
                        }),
                        multisample: wgpu::MultisampleState::default(),
                        fragment: Some(wgpu::FragmentState {
                            module: m,
                            entry_point: Some("fs_ao_depth"),
                            compilation_options: Default::default(),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: crate::post::LINDEPTH_FORMAT,
                                blend: None,
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        multiview_mask: None,
                        cache: crate::pipeline_cache::of(&device).as_ref(),
                    })
                })
            })
        };
        let spray_pipeline = crate::deferred_hdr(
            device,
            &pl,
            module.clone(),
            samples,
            move |layout, module| crate::HdrPipeline {
                alpha_to_coverage: false,
                label: "water spray",
                layout,
                module,
                vs: "vs_spray",
                fs: "fs_spray",
                buffers: &[],
                cull: None,
                depth_write: false,
                depth_compare: wgpu::CompareFunction::Less,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                samples,
            },
        );
        let waterline_pipeline = crate::deferred_hdr(
            device,
            &pl,
            module.clone(),
            samples,
            move |layout, module| crate::HdrPipeline {
                alpha_to_coverage: false,
                label: "water waterline",
                layout,
                module,
                vs: "vs_waterline",
                fs: "fs_waterline",
                buffers: &[],
                cull: None,
                depth_write: false,
                depth_compare: wgpu::CompareFunction::Always,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                samples,
            },
        );
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("water"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        // The ripples' normal map: tiled, trilinear and 16x anisotropic (three.js water normals are RepeatWrapping
        // with mips): the clamped, mip-less water sampler blocked them up at grazing angles and seamed every tile.
        let ripple_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("water ripples"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            anisotropy_clamp: 16,
            ..Default::default()
        });
        let flat = height_texture(device, 1).1;
        let no_wake = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("water no wake"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&Default::default());
        let no_shore = {
            let t = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("water no shore"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            t.create_view(&Default::default())
        };
        WaterRenderer {
            no_shore,
            shores: Vec::new(),
            ripple_sampler,
            layout,
            pipeline,
            mesh_pipeline,
            spray_pipeline,
            waterline_pipeline,
            ao_depth_pipelines,
            sampler,
            flat,
            slots: Vec::new(),
            ripple_pipelines: None,
            compute: compute && crate::WORLD_3D,
            fft: None,
            module: probe_module,
            last_sea: None,
            submitted: (0, 0),
            wakes: None,
            wake_centre: None,
            rivers: Vec::new(),
            no_wake,
            caustics: None,
            caustics_left_out,
        }
    }

    /// Writes this frame's uniforms and height fields and makes the bind
    /// groups (the scene copy and the depth prepass are this frame's;
    /// `planar`: the surfaces drawn with this frame's planar reflections and
    /// their textures, `fallback` bound where there is none).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare<'t>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        waters: &[Water],
        screen: (u32, u32),
        scene_color: &wgpu::TextureView,
        scene_depth: &wgpu::TextureView,
        planar: &[(usize, &wgpu::TextureView)],
        ao: &wgpu::TextureView,
        ao_direct: f32,
        adaptation: f32,
        origin: Vec3,
        seconds: f64,
        camera: &crate::Camera,
        quality: &crate::RenderQuality,
        fallback: &wgpu::TextureView,
        caustic_grid: &wgpu::Buffer,
        mesh_buffers: &dyn Fn(
            crate::scene::MeshHandle,
        ) -> Option<(wgpu::Buffer, wgpu::Buffer, u32)>,
        textures: &dyn Fn(crate::TextureHandle) -> Option<&'t wgpu::TextureView>,
    ) -> Vec<WaterDraw> {
        while self.slots.len() < waters.len() {
            self.slots.push(Slot {
                uniform: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("water"),
                    size: std::mem::size_of::<WaterGpu>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                face: bytemuck::Zeroable::zeroed(),
                heights: None,
                ripples: None,
            });
        }
        let (w, h) = (screen.0 as f32, screen.1 as f32);
        // The seabeds of this frame's surfaces, each uploaded once.
        let fields: Vec<&Arc<og_core::ocean::Shore>> = waters
            .iter()
            .filter_map(|w| w.shore.as_ref().map(|s| &s.field))
            .collect();
        self.shores
            .retain(|(f, _)| fields.iter().any(|g| Arc::ptr_eq(f, g)));
        for f in fields {
            if !self.shores.iter().any(|(g, _)| Arc::ptr_eq(f, g)) {
                let (_, view) = shore_texture(device, queue, f);
                self.shores.push((f.clone(), view));
            }
        }
        // W5: the rivers' ribbons, each built once for its course and frame.
        self.rivers.retain(|m| {
            waters
                .iter()
                .any(|w| w.river.as_ref().is_some_and(|r| m.fits(r, w.transform)))
        });
        for w in waters {
            if let Some(r) = &w.river
                && !self.rivers.iter().any(|m| m.fits(r, w.transform))
            {
                self.rivers.push(crate::water_river::RiverMesh::new(
                    device,
                    queue,
                    r.clone(),
                    w.transform,
                ));
            }
        }
        let (sea_segments, sea_cascades) = sea_tier(quality);
        self.submitted = (0, 0);
        self.wake_centre = None;
        let sea = self.fft_sea(
            device,
            queue,
            waters,
            sea_cascades,
            seconds,
            camera,
            origin,
            caustic_grid,
        );
        let levels = clipmap(camera.position, origin, sea_segments);
        let line = waterline(waters, camera, seconds, origin);
        let mut draws = Vec::with_capacity(waters.len());
        self.last_sea = None;
        for (index, (water, slot)) in waters.iter().zip(&mut self.slots).enumerate() {
            let reflection = planar.iter().find(|(i, _)| *i == index).map(|(_, v)| *v);
            let segments = grid_segments(water);
            let river = water
                .river
                .as_ref()
                .and_then(|r| self.rivers.iter().find(|m| m.fits(r, water.transform)));
            let mesh = water
                .mesh
                .and_then(mesh_buffers)
                .or_else(|| river.map(|m| (m.vertices.clone(), m.indices.clone(), m.count)));
            let field = height_field(water).filter(|_| mesh.is_none());
            if let Some(f) = field {
                if slot
                    .heights
                    .as_ref()
                    .is_none_or(|(r, ..)| *r != f.resolution)
                {
                    let (t, v) = height_texture(device, f.resolution);
                    slot.heights = Some((f.resolution, t, v));
                }
                let (_, texture, _) = slot.heights.as_ref().expect("made above");
                let texels: Vec<[u16; 4]> =
                    f.heights.iter().map(|&h| [f16_bits(h), 0, 0, 0]).collect();
                queue.write_texture(
                    texture.as_image_copy(),
                    bytemuck::cast_slice(&texels),
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(f.resolution * 8),
                        rows_per_image: None,
                    },
                    wgpu::Extent3d {
                        width: f.resolution,
                        height: f.resolution,
                        depth_or_array_layers: 1,
                    },
                );
            }
            // The ripples over the heights, stepped to this frame's tick.
            match field.and_then(|f| Some((f, water.wakes.as_deref()?))) {
                Some((f, wakes)) if self.compute => {
                    let pipes = self
                        .ripple_pipelines
                        .get_or_insert_with(|| crate::water_ripples::RipplePipelines::new(device));
                    if slot.ripples.as_ref().is_none_or(|r| !r.fits(f)) {
                        // The compute reads the heights' one layer as a plain 2D texture.
                        let (_, texture, _) = slot.heights.as_ref().expect("made above");
                        let coarse = texture.create_view(&Default::default());
                        slot.ripples = Some(crate::water_ripples::Ripples::new(
                            device, pipes, f, &coarse,
                        ));
                    }
                    let ripples = slot.ripples.as_mut().expect("made above");
                    ripples.update(device, queue, pipes, wakes);
                    self.submitted.0 += 1;
                }
                _ => slot.ripples = None,
            }
            let shore_view = shore_view(&self.shores, &self.no_shore, water);
            let normal_view = water.normal_map.and_then(textures);
            // A river's flow map is its current, its length the river's speed.
            let flow_view = match river {
                Some(m) => Some(&m.flow),
                None => water.flow_map.and_then(textures),
            };
            // With a flow map the flow's length is the speed: three.js'
            // flowSpeed 0.03 for a zero flow.
            let flow = match river {
                Some(m) => Vec2::new(m.river.speed.max(1e-3), 0.0),
                None if flow_view.is_some() && water.flow.length_squared() < 1e-12 => {
                    Vec2::new(0.03, 0.0)
                }
                None => water.flow,
            };
            let (mut ocean, swell) = ocean_gpu(water, seconds, origin);
            let drawn_sea = sea.filter(|_| ocean[0] > 0.0);
            let (hull_count, hull_rows) = match drawn_sea {
                Some(_) => hulls(water, camera.position),
                None => (0, [[0.0; 4]; 2 * MAX_HULLS]),
            };
            if let Some(f) = drawn_sea {
                ocean[2] = f.foam.fresh;
            }
            let gpu = WaterGpu {
                model: water.transform.to_cols_array_2d(),
                size: [
                    water.size.x.max(0.0),
                    water.size.y.max(0.0),
                    segments as f32,
                    // 2: the ripples' heights, read per pixel.
                    match (field, &slot.ripples) {
                        (Some(_), Some(_)) => 2.0,
                        (Some(_), None) => 1.0,
                        (None, _) => 0.0,
                    },
                ],
                shallow: water.shallow.extend(water.depth_fade.max(1e-3)).to_array(),
                deep: water.deep.extend(water.opacity.clamp(0.0, 1.0)).to_array(),
                foam: water.foam.extend(water.foam_width.max(0.0)).to_array(),
                waves: [
                    water.wave_scale.max(0.0),
                    water.wave_speed,
                    water.wave_strength.max(0.0),
                    water.refraction.max(0.0),
                ],
                look: [
                    water.reflectivity.max(0.0),
                    // The screen-space march on (water.wgsl).
                    1.0,
                    field.map_or(1.0, |f| f.size),
                    field.map_or(2.0, |f| match slot.ripples {
                        Some(_) => crate::water_ripples::Ripples::side(f.resolution) as f32,
                        None => f.resolution as f32,
                    }),
                ],
                screen: [w, h, 1.0 / w.max(1.0), 1.0 / h.max(1.0)],
                extra: [
                    water.shadows.clamp(0.0, 1.0),
                    f32::from(reflection.is_some()),
                    water.f0.clamp(0.0, 1.0),
                    water.reflection_bias.clamp(0.0, 1.0),
                ],
                under: water
                    .underwater_color
                    .extend(water.glint.max(0.0))
                    .to_array(),
                normals: [
                    water.normal_scale.max(0.0),
                    flow.x,
                    flow.y,
                    f32::from(normal_view.is_some()),
                ],
                ao: [
                    ao_direct.clamp(0.0, 1.0),
                    f32::from(flow_view.is_some()),
                    f32::from(mesh.is_some()),
                    1.0 / adaptation.max(1e-3),
                ],
                layers: lake_layers(origin, water.normal_scale.max(0.0)),
                ocean,
                sea_wind: {
                    let d = water.wind.normalize_or_zero();
                    [d.x, d.y, water.wind.length() * ocean[0], water.foam_streaks]
                },
                clip: [
                    CLIP_LEVELS as f32,
                    sea_segments as f32,
                    camera.far * 0.95,
                    f32::from(water.endless),
                ],
                levels,
                inverse: water.transform.inverse().to_cols_array_2d(),
                fft: match drawn_sea {
                    Some(f) => [
                        f.tiles.iter().filter(|t| **t > 0.0).count() as f32,
                        hull_count as f32,
                        f.tail,
                        self::ocean(water).map_or(0.0, |o| o.significant_height()),
                    ],
                    None => [0.0; 4],
                },
                hulls: hull_rows,
                bands: drawn_sea.map_or([0.0; 4], |f| f.bands),
                cascades: cascade_uv(drawn_sea, origin),
                foam_levels: match drawn_sea {
                    Some(_) => crate::ocean_fft::foam::levels(glam::DVec2::new(
                        f64::from(origin.x),
                        f64::from(origin.z),
                    )),
                    None => [[0.0; 4]; 2],
                },
                foam_uv: foam_uv(origin, water.wind, seconds),
                swell,
                shore: shore_gpu(water, seconds),
                spray: spray_gpu(water, drawn_sea.is_some(), seconds, quality),
                waterline: [
                    f32::from(line == Some(index)),
                    water.underwater_density.max(0.0),
                    water.meniscus.max(0.0),
                    f32::from(river.is_some()),
                ],
                wake: match self.wake_centre {
                    Some(c) if drawn_sea.is_some() && water.dynamic_waves > 0.0 => {
                        let side = crate::sea_wakes::SIDE as f32 * crate::sea_wakes::CELL;
                        [c.x, c.y, 1.0 / side, 1.0]
                    }
                    _ => [0.0; 4],
                },
                optics: {
                    let on = f32::from(
                        water.absorption.max_element() > 0.0
                            || water.scattering.max_element() > 0.0,
                    );
                    [
                        water.absorption.max(Vec3::ZERO).extend(on).to_array(),
                        water.scattering.max(Vec3::ZERO).extend(on).to_array(),
                    ]
                },
            };
            queue.write_buffer(&slot.uniform, 0, bytemuck::bytes_of(&gpu));
            // A probe face has no planar reflection of its own (that would be
            // a mirrored pass per face); its screen-space reflection marches
            // the face's own depth prepass and copy, as the frame's does,
            // mirroring what the probe sees from its point.
            slot.face = gpu;
            slot.face.extra[1] = 0.0;
            let wake_view = match &self.wakes {
                Some(w) if gpu.wake[3] > 0.5 => &w.view,
                _ => &self.no_wake,
            };
            let heights = match (&slot.ripples, &slot.heights, field, &self.fft) {
                (_, _, _, Some((fft, _))) if drawn_sea.is_some() => &fft.view,
                (Some(r), _, Some(_), _) => &r.view,
                (None, Some((_, _, v)), Some(_), _) => v,
                _ => &self.flat,
            };
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("water"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: slot.uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(heights),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(scene_color),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(scene_depth),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::TextureView(
                            reflection.unwrap_or(fallback),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::TextureView(
                            normal_view.unwrap_or(fallback),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::Sampler(&self.ripple_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: wgpu::BindingResource::TextureView(ao),
                    },
                    wgpu::BindGroupEntry {
                        binding: 9,
                        resource: wgpu::BindingResource::TextureView(flow_view.unwrap_or(fallback)),
                    },
                    wgpu::BindGroupEntry {
                        binding: 10,
                        resource: wgpu::BindingResource::TextureView(shore_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 11,
                        resource: wgpu::BindingResource::TextureView(wake_view),
                    },
                ],
            });
            if drawn_sea.is_some() && self.last_sea.is_none() {
                self.last_sea = Some(group.clone());
            }
            let half = Vec3::new(water.size.x, 0.0, water.size.y) * 0.5;
            let radius = water.transform.transform_vector3(half).length();
            draws.push(WaterDraw {
                group,
                vertices: if drawn_sea.is_some() {
                    sea_vertices(sea_segments)
                } else {
                    segments * segments * 6
                },
                bounds: (water.transform.transform_point3(Vec3::ZERO), radius),
                mesh,
                spray: if gpu.spray[0] > 0.0 {
                    2 * SPRAY_SLOTS
                } else {
                    0
                },
                waterline: line == Some(index),
            });
        }
        draws
    }

    /// The surfaces as the faces of a reflection probe's capture of face
    /// size `size` draw them (after [`WaterRenderer::prepare`]): through
    /// `below`, the copy of the face drawn so far, and over `scene_depth`,
    /// the face's depth prepass, so the face refracts what lies below,
    /// foams at the edges and reflects the face in screen
    /// space as the frame does; without the planar reflection.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn face_draws<'a>(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        waters: &[Water],
        size: u32,
        below: &wgpu::TextureView,
        scene_depth: &wgpu::TextureView,
        white: &wgpu::TextureView,
        textures: &dyn Fn(crate::scene::TextureHandle) -> Option<&'a wgpu::TextureView>,
    ) -> Vec<WaterDraw> {
        let n = size as f32;
        waters
            .iter()
            .zip(&self.slots)
            .map(|(water, slot)| {
                let sea = slot.face.fft[0] > 0.0;
                let heights = match (&slot.ripples, &slot.heights, height_field(water), &self.fft) {
                    (_, _, _, Some((fft, _))) if sea => &fft.view,
                    (Some(r), _, Some(_), _) => &r.view,
                    (None, Some((_, _, v)), Some(_), _) => v,
                    _ => &self.flat,
                };
                let mut face = slot.face;
                face.screen = [n, n, 1.0 / n, 1.0 / n];
                let uniform = crate::buffer_init(
                    device,
                    queue,
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("water (probe face)"),
                        contents: bytemuck::bytes_of(&face),
                        usage: wgpu::BufferUsages::UNIFORM,
                    },
                );
                let tex = wgpu::BindingResource::TextureView;
                // No planar reflection nor screen AO in a face: its own
                // copy and white in their slots.
                let entries = [
                    uniform.as_entire_binding(),
                    tex(heights),
                    tex(below),
                    tex(scene_depth),
                    wgpu::BindingResource::Sampler(&self.sampler),
                    tex(below),
                    tex(water.normal_map.and_then(textures).unwrap_or(white)),
                    wgpu::BindingResource::Sampler(&self.ripple_sampler),
                    tex(white),
                    tex(water.flow_map.and_then(textures).unwrap_or(white)),
                    tex(shore_view(&self.shores, &self.no_shore, water)),
                    tex(match &self.wakes {
                        Some(w) if face.wake[3] > 0.5 => &w.view,
                        _ => &self.no_wake,
                    }),
                ];
                let entries: Vec<wgpu::BindGroupEntry> = entries
                    .into_iter()
                    .enumerate()
                    .map(|(i, resource)| wgpu::BindGroupEntry {
                        binding: i as u32,
                        resource,
                    })
                    .collect();
                let half = Vec3::new(water.size.x, 0.0, water.size.y) * 0.5;
                WaterDraw {
                    group: device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("water (probe face)"),
                        layout: &self.layout,
                        entries: &entries,
                    }),
                    // A surface over a mesh is left out of the faces (the
                    // faces' draws carry no mesh buffers).
                    vertices: if water.mesh.is_some() {
                        0
                    } else if sea {
                        sea_vertices(slot.face.clip[1] as u32)
                    } else {
                        grid_segments(water) * grid_segments(water) * 6
                    },
                    mesh: None,
                    // The spray is the camera's alone (its droplets are about the view), as the waterline is.
                    spray: 0,
                    waterline: false,
                    bounds: (
                        water.transform.transform_point3(Vec3::ZERO),
                        water.transform.transform_vector3(half).length(),
                    ),
                }
            })
            .collect()
    }

    /// The FFT sea of the frame's first surface with a wind, its fields
    /// computed at `seconds` (the scene's tick time, so a frame is the
    /// tick's whatever the wall clock) and submitted ahead of the frame:
    /// its spectrum rebuilt when that sea's wind, swell or choppiness or the
    /// cascades change. None without a sea.
    #[allow(clippy::too_many_arguments)]
    fn fft_sea(
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
        let centre = water.transform.w_axis;
        let finest = tick_cell(water, Vec2::new(centre.x, centre.z));
        let key = (water.wind, water.swell, water.choppiness, cascades, finest);
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
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ocean fft"),
        });
        let foam_sea = crate::ocean_fft::foam::FoamSea {
            tick: [centre.x, centre.z, tick_even_cell(water), 0.0],
            half: [water.size.x * 0.5, water.size.y * 0.5, 0.0, 0.0],
            cascades: cascade_uv(Some(info), origin),
            bands: info.bands,
            wind: water.wind,
            thresholds: info.foam.laid,
            gain: FOAM_GAIN,
            streaks: water.foam_streaks,
        };
        let world = |v: Vec3| glam::DVec2::new(f64::from(v.x), f64::from(v.z));
        let steps = fft.advance(
            device,
            queue,
            &mut encoder,
            seconds,
            world(camera.position) + world(origin),
            world(origin),
            &foam_sea,
            &|at| ocean_gpu(water, at, origin).1,
            (shore_view(&self.shores, &self.no_shore, water), &|at| {
                shore_gpu(water, at)
            }),
        );
        // W3: the hulls' wakes, stepped to this frame's tick in the same submit.
        if let Some(wakes) = water
            .wakes
            .as_deref()
            .filter(|_| water.dynamic_waves > 0.0 && self.compute)
        {
            let pass = self
                .wakes
                .get_or_insert_with(|| crate::sea_wakes::SeaWakes::new(device));
            pass.update(
                queue,
                &mut encoder,
                wakes,
                world(camera.position) + world(origin),
                Vec2::new(centre.x, centre.z),
                world(origin),
                water.dynamic_waves,
            );
            self.wake_centre = Some(pass.centre(world(origin)));
        }
        // W2: the sea's curvature about the camera, for the caustics under it.
        if water.caustics > 0.0 && self.compute && !self.caustics_left_out {
            let pass = self
                .caustics
                .get_or_insert_with(|| CausticsPass::new(device));
            let params = CausticsGpu {
                grid: self::caustic_grid(camera.position, origin),
                sea: [fft.cascades() as f32, tick_even_cell(water), 0.0, 0.0],
                cascades: cascade_uv(Some(info), origin),
                bands: info.bands,
            };
            pass.encode(
                device,
                queue,
                &mut encoder,
                &fft.view,
                &params,
                caustic_grid,
            );
        }
        queue.submit([encoder.finish()]);
        self.submitted = (self.submitted.0 + 1, steps as u32);
        Some(info)
    }

    /// The drawn sea's height at `points` (frame xz) as the last frame drew
    /// its first sea (`probe_sea` in water.wgsl): submitted, into a buffer to
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

    /// Draws the surfaces' view depth for the screen AO (`fs_ao_depth`; group 0: the frame). Returns the draw calls.
    pub(crate) fn draw_ao_depth(&self, pass: &mut wgpu::RenderPass, draws: &[WaterDraw]) -> u32 {
        let [grid, mesh] = self.ao_depth_pipelines.get();
        for d in draws {
            pass.set_bind_group(1, &d.group, &[]);
            match &d.mesh {
                Some((vertices, indices, count)) => {
                    pass.set_pipeline(mesh);
                    pass.set_vertex_buffer(0, vertices.slice(..));
                    pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..*count, 0, 0..1);
                }
                None => {
                    pass.set_pipeline(grid);
                    pass.draw(0..d.vertices, 0..1);
                }
            }
        }
        draws.len() as u32
    }

    /// Draws the surfaces (group 0: the frame, or each surface's reflection
    /// probe's: probe_select.rs). Returns the draw calls.
    pub(crate) fn draw(
        &self,
        pass: &mut wgpu::RenderPass,
        draws: &[WaterDraw],
        probes: &mut crate::probe_select::Binder,
        // The frame's features (pipeline_features.rs).
        features: Features,
    ) -> u32 {
        for d in draws {
            probes.at(pass, d.bounds.0, d.bounds.1);
            pass.set_bind_group(1, &d.group, &[]);
            match &d.mesh {
                Some((vertices, indices, count)) => {
                    pass.set_pipeline(self.mesh_pipeline.get(features));
                    pass.set_vertex_buffer(0, vertices.slice(..));
                    pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..*count, 0, 0..1);
                }
                None => {
                    pass.set_pipeline(self.pipeline.get(features));
                    pass.draw(0..d.vertices, 0..1);
                }
            }
        }
        probes.reset(pass);
        // The spray over every surface (it lies in front of the water it leaves).
        let mut calls = draws.len() as u32;
        for d in draws.iter().filter(|d| d.spray > 0) {
            pass.set_pipeline(&self.spray_pipeline);
            pass.set_bind_group(1, &d.group, &[]);
            pass.draw(0..6, 0..d.spray);
            calls += 1;
        }
        // The waterline over all of it.
        for d in draws.iter().filter(|d| d.waterline) {
            pass.set_pipeline(&self.waterline_pipeline);
            pass.set_bind_group(1, &d.group, &[]);
            pass.draw(0..3, 0..1);
            calls += 1;
        }
        calls
    }
}

/// A height field's texture: `n` × `n` rgba16float, the height in r,
/// viewed as a 1-layer array (water.wgsl `heights`; rgba16float is
/// filterable, as the FFT sea's fields bound there are sampled, and a
/// storage format for a compute pass writing it).
fn height_texture(device: &wgpu::Device, n: u32) -> (wgpu::Texture, wgpu::TextureView) {
    let t = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("water heights"),
        size: wgpu::Extent3d {
            width: n,
            height: n,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: HEIGHT_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let v = t.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
    });
    (t, v)
}

/// The water module after the frame group and the lighting.
pub(crate) fn water_source(frame_wgsl: &str, lighting_wgsl: &str) -> String {
    format!(
        "{frame_wgsl}{lighting_wgsl}{}{}",
        crate::ocean_fft::SHORE_WGSL,
        wgsl3d!("water.wgsl")
    )
}

/// The lake's four layers' uv at `origin` (water.wgsl `map_slope`: world
/// xz times the repeats over each layer's period), in f64 and without the
/// whole repeats: zero at the world's zero.
fn lake_layers(origin: Vec3, repeats: f32) -> [[f32; 4]; 2] {
    let k = f64::from(repeats);
    let at = |o: f32, period: f64| (f64::from(o) * k / period).rem_euclid(1.0) as f32;
    let (x, z) = (origin.x, origin.z);
    [
        [at(x, 103.0), at(z, 103.0), at(x, 107.0), at(z, 107.0)],
        [at(x, 8907.0), at(z, 9803.0), at(x, 1091.0), at(z, 1027.0)],
    ]
}
