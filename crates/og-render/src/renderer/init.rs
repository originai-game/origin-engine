//! Renderer creation and adapter initialization.

use super::*;

impl Renderer {
    /// Times each render-graph pass's CPU encoding with `clock` (ms) and,
    /// where the device has timestamp queries, its GPU time
    /// ([`Renderer::gpu_timings`]); `None` stops timing. Measurement only:
    /// nothing drawn depends on it.
    pub fn set_clock(&mut self, clock: Option<fn() -> f64>) {
        self.clock = clock;
        if clock.is_none() {
            self.pass_ms.clear();
            self.gpu_timer = None;
        } else if self.gpu_timer.is_none() {
            self.gpu_timer = timing::GpuTimer::new(&self.device, &self.queue);
        }
    }

    /// The last frame's render-graph passes in execution order, each with
    /// the clock's reading when its encoding began and the milliseconds it
    /// took; empty without a clock.
    pub fn pass_timings(&self) -> &[PassTiming] {
        &self.pass_ms
    }

    /// Why this device times no pass on the GPU (no timestamp queries), or
    /// `None` when it does once a clock is set.
    pub fn gpu_timing_absent(&self) -> Option<&'static str> {
        gpu_timing_absent(self.device.features())
    }

    /// The GPU time per pass of the frames whose timestamps were read back
    /// since the last call, oldest first: a frame's arrive a frame or two
    /// after it was rendered. With `wait` (native), every frame in flight
    /// is waited for. Empty without a clock or timestamp queries.
    pub fn gpu_timings(&mut self, wait: bool) -> Vec<GpuFrame> {
        match self.gpu_timer.as_mut() {
            Some(t) => t.take(&self.device, &self.queue, wait),
            None => Vec::new(),
        }
    }

    /// The custom post passes of `scene` that failed to compile on this
    /// device, with the device's message: they are skipped
    /// (`post_custom.rs`).
    pub fn post_errors(&self, scene: &RenderScene) -> Vec<(PostHandle, String)> {
        self.custom_posts.errors(scene.id())
    }

    /// A headless renderer (draws to [`RenderTarget::Offscreen`] or caller views).
    pub async fn new(desc: &RenderDesc) -> Result<Renderer, RenderError> {
        if renderer_disabled() {
            return Err(disabled_error());
        }
        let instance = desc.backend.instance();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: power_preference(desc),
                force_fallback_adapter: false,
                compatible_surface: None,
                ..Default::default()
            })
            .await
            .map_err(|e| RenderError::NoAdapter(format!("{:?}: {e}", desc.backend)))?;
        let format = desc.output_format.unwrap_or(DEFAULT_OUTPUT_FORMAT);
        Self::from_adapter(adapter, None, format, desc).await
    }

    /// A renderer that presents to `surface` ([`RenderTarget::Surface`]); `instance`
    /// must be the one the surface was created from (see [`Backend::instance`]).
    pub async fn with_surface(
        desc: &RenderDesc,
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
    ) -> Result<Renderer, RenderError> {
        if renderer_disabled() {
            return Err(disabled_error());
        }
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: power_preference(desc),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .map_err(|e| RenderError::NoAdapter(format!("{:?}: {e}", desc.backend)))?;
        let caps = surface.get_capabilities(&adapter);
        let hdr_space = desc
            .hdr
            .then(|| hdr::surface_space(&caps))
            .flatten()
            .filter(|_| {
                desc.output_format
                    .is_none_or(|f| f == wgpu::TextureFormat::Rgba16Float)
            });
        // An sRGB surface where one is offered (native windows): the
        // hardware encodes and blends in linear light, as offscreen. A
        // WebGPU canvas offers none; its sRGB view (below) does the same.
        let eight_bit = |f: &wgpu::TextureFormat, srgb: bool| {
            f.is_srgb() == srgb
                && matches!(
                    f.remove_srgb_suffix(),
                    wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm
                )
        };
        let format = match hdr_space
            .map(|_| wgpu::TextureFormat::Rgba16Float)
            .or(desc.output_format)
        {
            Some(f) => f,
            None => caps
                .formats
                .iter()
                .copied()
                .find(|f| eight_bit(f, true))
                .or_else(|| caps.formats.iter().copied().find(|f| eight_bit(f, false)))
                .ok_or_else(|| {
                    RenderError::Surface(format!(
                        "no 8-bit RGBA surface format in {:?}",
                        caps.formats
                    ))
                })?,
        };
        // A surface frame can be copied out where the device allows it
        // (frame capture on the web host: `set_keep_frame`).
        let copy = surface
            .get_capabilities(&adapter)
            .usages
            .contains(wgpu::TextureUsages::COPY_SRC);
        let config = wgpu::SurfaceConfiguration {
            usage: if copy {
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC
            } else {
                wgpu::TextureUsages::RENDER_ATTACHMENT
            },
            format,
            color_space: hdr_space.unwrap_or(wgpu::SurfaceColorSpace::Srgb),
            width: desc.width.max(1),
            height: desc.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: linear_view_format(&adapter, format, true)
                .into_iter()
                .collect(),
            desired_maximum_frame_latency: 2,
        };
        Self::from_adapter(adapter, Some((surface, config)), format, desc).await
    }

    async fn from_adapter(
        adapter: wgpu::Adapter,
        surface: Option<(wgpu::Surface<'static>, wgpu::SurfaceConfiguration)>,
        output_format: wgpu::TextureFormat,
        desc: &RenderDesc,
    ) -> Result<Renderer, RenderError> {
        let adapter_info = adapter.get_info();
        let indirect = adapter
            .get_downlevel_capabilities()
            .flags
            .contains(wgpu::DownlevelFlags::INDIRECT_EXECUTION);
        let msaa_ok = supports_msaa(&adapter);
        let antialias = match desc.antialias {
            Antialias::Off => Antialias::Off,
            Antialias::Taa => Antialias::Taa,
            Antialias::Fxaa => Antialias::Fxaa,
            Antialias::Smaa => Antialias::Smaa,
            Antialias::SmaaT2x => Antialias::SmaaT2x,
            Antialias::Tier => Antialias::Taa,
            Antialias::Msaa4 if msaa_ok => Antialias::Msaa4,
            // No multisampled HDR target: the post-process instead.
            Antialias::Msaa4 => Antialias::Fxaa,
        };
        let samples = if antialias == Antialias::Msaa4 {
            MSAA_SAMPLES
        } else {
            1
        };
        // Compute and storage buffers (particles, culling, the skinning
        // palette, the light tables) at the downlevel defaults' counts, but
        // WebGPU's own eight storage buffers a stage (every WebGPU device has
        // them; the engine has no other tier): og.motion's kernels, which run
        // on this device (og-motion's `Gpu::new`), bind up to seven.
        let mut limits = wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits());
        limits.max_storage_buffers_per_shader_stage = limits
            .max_storage_buffers_per_shader_stage
            .max(8)
            .min(adapter.limits().max_storage_buffers_per_shader_stage);
        let shadow_size = desc
            .shadow_map_size
            .clamp(64, limits.max_texture_dimension_2d.max(64));
        let cascades = desc.shadow_cascades.clamp(1, MAX_CASCADES);
        let (mut features, texture_formats) =
            TextureFormats::choose(adapter.features(), desc.texture_compression);
        let cache_dir = pipeline_cache::dir();
        if cache_dir.is_some() && adapter.features().contains(wgpu::Features::PIPELINE_CACHE) {
            features |= wgpu::Features::PIPELINE_CACHE;
        }
        // GPU pass timing (`timing.rs`) where the adapter has timestamps
        // (a publish build has none), and og.motion's slice timing when
        // asked (`RenderDesc::timestamps`), publish too.
        if (cfg!(not(feature = "publish")) || desc.timestamps)
            && adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY)
        {
            features |= wgpu::Features::TIMESTAMP_QUERY;
        }
        let mut info = RenderInfo {
            adapter: adapter_info.name.clone(),
            backend: adapter_info.backend,
            device_type: adapter_info.device_type,
            hdr_supported: surface
                .as_ref()
                .is_some_and(|(s, _)| hdr::surface_space(&s.get_capabilities(&adapter)).is_some()),
            hdr: surface
                .as_ref()
                .is_some_and(|(_, c)| c.color_space.is_hdr()),
            antialias,
            msaa_samples: samples,
            shadow_map_size: shadow_size,
            shadow_cascades: cascades,
            shadowed_lights: MAX_SHADOWED_LIGHTS as u32,
            point_shadows: desc.point_shadows.resolve(),
            texture_formats,
            pipeline_cache: false,
            max_texture_size: limits.max_texture_dimension_2d,
            volumetric: desc.volumetric.unwrap_or(VolumetricMethod::Froxel),
            volumetric_lights: 0,
            sprite_anisotropy: if adapter
                .get_downlevel_capabilities()
                .flags
                .contains(wgpu::DownlevelFlags::ANISOTROPIC_FILTERING)
                && !is_software_adapter(adapter_info.device_type, &adapter_info.name)
            {
                SPRITE_ANISOTROPY
            } else {
                1
            },
            planar_reflections: if WORLD_3D {
                desc.planar_reflections.clamp(1, MAX_PLANAR_REFLECTIONS)
            } else {
                0
            },
            transparency: Transparency::Weighted,
        };
        info.volumetric_lights = volumetric::lights(info.volumetric);
        log::info!(
            "og-render: {} ({:?}, {:?}, antialias {:?}, shadows {}x{}², textures {:?}/{:?})",
            info.adapter,
            info.backend,
            info.device_type,
            info.antialias,
            cascades,
            shadow_size,
            info.texture_formats.color,
            info.texture_formats.normal
        );
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("og-render"),
                required_features: features,
                required_limits: limits,
                ..Default::default()
            })
            .await
            .map_err(|e| RenderError::NoDevice(e.to_string()))?;
        // Before any pipeline is made: every one is made with the cache.
        let pipeline_disk =
            cache_dir.and_then(|dir| pipeline_cache::DiskCache::open(&device, &adapter_info, dir));
        info.pipeline_cache = pipeline_disk.is_some();
        // Natively a validation error panics, as wgpu-core's default
        // handler does (LostFlag::watch keeps that until the device is
        // lost), which is how native hosts and tests fail loudly; logging
        // instead would leave a silently broken frame. The browser's
        // WebGPU reports to its own console.
        let max = device.limits().max_texture_dimension_2d;
        let mut surface = surface;
        if let Some((surface, config)) = &mut surface {
            config.width = config.width.clamp(1, max);
            config.height = config.height.clamp(1, max);
            surface.configure(&device, config);
        }
        let (width, height) = (desc.width.clamp(1, max), desc.height.clamp(1, max));

        // Layouts.
        let [lights_entry, table_entry, refs_entry] = clusters::layout_entries();
        let frame_entries = [
            uniform_entry(0, wgpu::ShaderStages::VERTEX_FRAGMENT),
            texture_entry(1, false, wgpu::TextureViewDimension::D2Array),
            texture_entry(2, false, wgpu::TextureViewDimension::D2Array),
            texture_entry(3, true, wgpu::TextureViewDimension::Cube),
            texture_entry(4, true, wgpu::TextureViewDimension::Cube),
            tex2d(5),
            sampler_entry(6, wgpu::SamplerBindingType::Filtering),
            // The local lights and their clusters: buffers, so the lit
            // pipelines keep their sixteen texture slots.
            lights_entry,
            table_entry,
            refs_entry,
            // Baked light volumes and the rect lights' LTC table
            // (lightmap.rs, area.rs): buffers, as the light tables.
            lightmap::probe_layout_entry(),
            area::ltc_layout_entry(),
        ];
        let frame_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("frame"),
            entries: &frame_entries,
        });
        // Order-independent transparency (oit.rs): the linked lists where
        // the device can keep them, sized against its own limits.
        let oit = oit::OitState::new(
            &device,
            desc.transparency,
            desc.transparency_layers,
            adapter.get_downlevel_capabilities().flags,
            &device.limits(),
            &frame_entries,
        );
        info.transparency = oit.mode;
        // The sky cube, bound by the sky pass alone: in the frame group it
        // would take a texture slot from every lit pipeline, which have
        // sixteen (the downlevel limits the device is asked for).
        let sky_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sky"),
            entries: &[
                texture_entry(0, true, wgpu::TextureViewDimension::Cube),
                // The clouds' noise volume and its repeating sampler (clouds.rs).
                texture_entry(1, true, wgpu::TextureViewDimension::D3),
                sampler_entry(2, wgpu::SamplerBindingType::Filtering),
                // The background's blurriness and map (sky.wgsl SkyParams).
                uniform_entry(3, wgpu::ShaderStages::FRAGMENT),
                // An environment map drawn from its own cube, and the map it
                // blends towards (the sky cube again without a map).
                texture_entry(4, true, wgpu::TextureViewDimension::Cube),
                texture_entry(5, true, wgpu::TextureViewDimension::Cube),
                // The aurora's footprint and table (aurora.rs), read with
                // the frame group's IBL sampler.
                texture_entry(6, true, wgpu::TextureViewDimension::D2),
            ],
        });
        let material_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("material"),
            entries: &[
                // The vertex stage reads a material shader's parameters.
                uniform_entry(0, wgpu::ShaderStages::VERTEX_FRAGMENT),
                tex2d(1),
                tex2d(2),
                tex2d(3),
                tex2d(4),
                tex2d(5),
                tex2d(6),
                sampler_entry(7, wgpu::SamplerBindingType::Filtering),
                tex2d(8),
                tex2d(9),
            ],
        });
        // With two planar reflections the effect group carries the one
        // material shaders read, and the lightmapped pipelines, whose pages
        // would make it a seventeenth texture, take the group without it.
        let planar = planar_slot(info.planar_reflections);
        let fx_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fx"),
            entries: &fx_layout_entries(planar),
        });
        let fx_lean_layout = planar.then(|| {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fx (lightmapped)"),
                entries: &fx_layout_entries(false),
            })
        });
        let sprite_fx_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite fx"),
            entries: &[
                tex2d(0),
                sampler_entry(1, wgpu::SamplerBindingType::Filtering),
                uniform_entry(2, wgpu::ShaderStages::FRAGMENT),
                tex2d(3),
            ],
        });
        let palette_layout = skin::palette_layout(&device);
        let sprite_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite"),
            entries: &[
                tex2d(0),
                sampler_entry(1, wgpu::SamplerBindingType::Filtering),
                uniform_entry(2, wgpu::ShaderStages::FRAGMENT),
            ],
        });
        let shadow_bind_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("shadow"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<ShadowPassUniform>() as u64,
                        ),
                    },
                    count: None,
                }],
            });

        // Samplers and fixed-size targets.
        let material_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("material"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            // Filter grazing-angle footprints along both axes. wgpu clamps
            // this to 1 on devices without ANISOTROPIC_FILTERING.
            anisotropy_clamp: 16,
            ..Default::default()
        });
        // Sprites and the UI's images filter anisotropically where the
        // device can (RenderInfo::sprite_anisotropy): a nine-slice's middle
        // squeezed along one axis reads its mip level by the other one.
        let sprite_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sprite"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            anisotropy_clamp: info.sprite_anisotropy,
            ..Default::default()
        });
        // Trilinear and clamped: the font atlases (an MSDF distance field
        // averaged along a squeezed axis loses its edge: the ink spreads and
        // thins), the decal atlas and the 2D lightmap (sampled as the
        // surfaces under them are, by the material sampler's filter).
        let trilinear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("trilinear"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let sprite_sampler_nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sprite (nearest)"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        // The default quality's cascades; a frame that asks another count
        // makes the array again (`fit_shelter_layer`).
        let cascade_map = ShadowMap::new(
            &device,
            "cascades",
            shadow_size,
            cascades.min(crate::RenderQuality::default().shadow_cascades),
        );
        // Twelve layers to begin with; a frame that needs more grows it
        // (`grow_local_shadows`).
        let local_map = ShadowMap::new(
            &device,
            "local shadows",
            LOCAL_SHADOW_SIZE,
            shadows::INITIAL_LOCAL_LAYERS
                .min(shadows::capacity(
                    info.shadowed_lights as usize,
                    info.point_shadows,
                ))
                .max(1) as u32,
        );

        // Uniforms.
        let frame_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("frame"),
            size: std::mem::size_of::<frame::FrameUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let light_buffers = clusters::buffers(&device);
        let ltc = area::ltc_buffer(&device);
        let volumetric = volumetric::Volumetric::new(&device, info.volumetric);
        // One set per planar reflection the renderer draws (none without
        // the 3D world).
        let slots = 0..info.planar_reflections;
        let reflection_light_buffers: Vec<_> =
            slots.clone().map(|_| clusters::buffers(&device)).collect();
        let reflection_buffer: Vec<_> = slots
            .clone()
            .map(|_| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("frame (planar reflection)"),
                    size: std::mem::size_of::<frame::FrameUniform>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .collect();
        let reflection_clusters: Vec<_> = slots
            .clone()
            .map(|_| clusters::ClusterBuilder::default())
            .collect();
        let reflection_fx_buffer: Vec<_> = slots
            .map(|_| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("fx (planar reflection)"),
                    size: std::mem::size_of::<fx::FxUniform>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .collect();
        let shadow_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("shadow passes"),
            size: SHADOW_SLOT_STRIDE * SHADOW_SLOTS as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow"),
            layout: &shadow_bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &shadow_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(std::mem::size_of::<ShadowPassUniform>() as u64),
                }),
            }],
        });

        // Pipelines.
        // Parsed when the first pipeline that uses them is made
        // (deferred.rs): a frame compiles only what it draws.
        let pbr_shader = deferred::module(
            &device,
            "pbr",
            lit_source_with(&hooks::default_hooks(), planar),
        );
        let sky_shader = deferred::module(&device, "sky", sky_source());
        let sprite_shader = deferred::module(&device, "sprite", sprite_source());
        let ribbon_shader = deferred::module(&device, "ribbon", ribbon_source());
        let shadow_shader =
            deferred::module(&device, "shadow", shadow_source(&hooks::default_hooks()));
        let pipeline_layout = |label: &str, layouts: &[&wgpu::BindGroupLayout]| {
            let layouts: Vec<_> = layouts.iter().map(|l| Some(*l)).collect();
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &layouts,
                immediate_size: 0,
            })
        };
        let pbr_layout = pipeline_layout("pbr", &[&frame_layout, &material_layout, &fx_layout]);
        let lightmaps = lightmap::Lightmaps::new(&device, &queue, &frame_layout);
        let pbr_skinned_layout = pipeline_layout(
            "pbr skinned",
            &[&frame_layout, &material_layout, &fx_layout, &palette_layout],
        );
        let sky_pl = pipeline_layout("sky", &[&frame_layout, &sky_layout]);
        let sprite_pl = pipeline_layout(
            "sprite",
            &[&frame_layout, &sprite_layout, &sprite_fx_layout],
        );
        let ribbon_pl = pipeline_layout("ribbon", &[&frame_layout, &sprite_layout]);
        let shadow_layouts = [
            pipeline_layout("shadow", &[&shadow_bind_layout]),
            pipeline_layout("shadow dissolve", &[&shadow_bind_layout, &material_layout]),
            pipeline_layout(
                "shadow skinned",
                &[&shadow_bind_layout, &material_layout, &palette_layout],
            ),
        ];
        // At the far plane, after the opaque batches: only uncovered pixels pass.
        let sky_state = |fs: &'static str| {
            deferred_hdr(
                &device,
                &sky_pl,
                sky_shader.clone(),
                samples,
                move |layout, module| HdrPipeline {
                    alpha_to_coverage: false,
                    label: "sky",
                    layout,
                    module,
                    vs: "vs_sky",
                    fs,
                    buffers: &[],
                    cull: None,
                    depth_write: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    blend: None,
                    samples,
                },
            )
        };
        let sky_pipeline = sky_state("fs_sky");
        let sky_look_pipeline = sky_state("fs_sky_look");
        let ibl = ibl::IblGenerator::new(&device);
        let cloud_noise = clouds::CloudNoise::placeholder(&device, &queue);
        let sky_params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sky params"),
            size: std::mem::size_of::<SkyParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let aurora_map = aurora::AuroraMap::placeholder(&device, &queue);
        let surfaces =
            ribbons_gpu::GpuTrails::supported(adapter.get_downlevel_capabilities().flags)
                .then(|| surface::SurfacePipelines::new(&device));
        let garments = surfaces
            .as_ref()
            .map(|_| crate::garment_gpu::Pipelines::new(&device));
        let water = water::WaterRenderer::new(
            &device,
            &frame_layout,
            FRAME_WGSL,
            LIGHTING_WGSL,
            samples,
            adapter
                .get_downlevel_capabilities()
                .flags
                .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS),
        );
        let metaballs =
            metaballs::MetaballRenderer::new(&device, &frame_layout, FRAME_WGSL, samples);
        let fluid3d =
            fluid3d::Fluid3dRenderer::new(&device, &frame_layout, &fluid_shared_source(), samples);
        let additive = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        };
        let sprite_state = |label: &'static str, fs: &'static str, blend| {
            deferred_hdr(
                &device,
                &sprite_pl,
                sprite_shader.clone(),
                samples,
                move |layout, module| HdrPipeline {
                    alpha_to_coverage: false,
                    label,
                    layout,
                    module,
                    vs: "vs",
                    fs,
                    buffers: SPRITE_BUFFERS,
                    cull: None,
                    depth_write: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    blend: Some(blend),
                    samples,
                },
            )
        };
        let sprite_pipeline = sprite_state("sprite", "fs", wgpu::BlendState::ALPHA_BLENDING);
        let sprite_additive_pipeline = sprite_state("sprite additive", "fs", additive);
        let sprite_untonemapped_pipeline =
            sprite_state("sprite untonemapped", "fs_untonemapped", UNTONEMAPPED_BLEND);
        // The colour kept, the alpha replaced by 1, over every pixel. No
        // groups: whichever sprite or array group is bound stays valid.
        let cover_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite cover"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let sprite_cover_pipeline = deferred_hdr(
            &device,
            &cover_pl,
            sprite_shader.clone(),
            samples,
            move |layout, module| HdrPipeline {
                alpha_to_coverage: false,
                label: "sprite cover",
                layout,
                module,
                vs: "vs_cover",
                fs: "fs_cover",
                buffers: &[],
                cull: None,
                depth_write: false,
                depth_compare: wgpu::CompareFunction::Always,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::Zero,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                samples,
            },
        );
        let msdf_pipeline = sprite_state("msdf text", "fs_msdf", wgpu::BlendState::ALPHA_BLENDING);
        // The glow buffer: no depth, one sample, added.
        let sprite_glow_pipeline = {
            let (device, sprite_pl, sprite_shader) =
                (device.clone(), sprite_pl.clone(), sprite_shader.clone());
            Deferred::new(move || {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("sprite glow"),
                    layout: Some(&sprite_pl),
                    vertex: wgpu::VertexState {
                        module: &sprite_shader,
                        entry_point: Some("vs"),
                        compilation_options: Default::default(),
                        buffers: SPRITE_BUFFERS,
                    },
                    primitive: Default::default(),
                    depth_stencil: None,
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &sprite_shader,
                        entry_point: Some("fs_glow"),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: HDR_FORMAT,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::One,
                                    dst_factor: wgpu::BlendFactor::One,
                                    operation: wgpu::BlendOperation::Add,
                                },
                                alpha: wgpu::BlendComponent::REPLACE,
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview_mask: None,
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            })
        };
        let ribbon_state = |blend| {
            deferred_hdr(
                &device,
                &ribbon_pl,
                ribbon_shader.clone(),
                samples,
                move |layout, module| HdrPipeline {
                    alpha_to_coverage: false,
                    label: "ribbon",
                    layout,
                    module,
                    vs: "vs",
                    fs: "fs",
                    buffers: RIBBON_BUFFERS,
                    cull: None,
                    depth_write: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    blend: Some(blend),
                    samples,
                },
            )
        };
        let ribbon_pipelines = [
            ribbon_state(additive),
            ribbon_state(wgpu::BlendState::ALPHA_BLENDING),
        ];
        // The strips expanded from their points (ribbons_gpu.rs), where
        // the device can: `vs_strip` reads them from group 2.
        let gpu_trails = ribbons_gpu::GpuTrails::supported(
            adapter.get_downlevel_capabilities().flags,
        )
        .then(|| {
            let points = ribbons_gpu::GpuTrails::draw_layout(&device);
            let strip_pl = pipeline_layout("strip", &[&frame_layout, &sprite_layout, &points]);
            let strip_state = |blend| {
                deferred_hdr(
                    &device,
                    &strip_pl,
                    ribbon_shader.clone(),
                    samples,
                    move |layout, module| HdrPipeline {
                        alpha_to_coverage: false,
                        label: "strip",
                        layout,
                        module,
                        vs: "vs_strip",
                        fs: "fs",
                        buffers: &[],
                        cull: None,
                        depth_write: false,
                        depth_compare: wgpu::CompareFunction::LessEqual,
                        blend: Some(blend),
                        samples,
                    },
                )
            };
            let pipelines = [
                strip_state(additive),
                strip_state(wgpu::BlendState::ALPHA_BLENDING),
            ];
            ribbons_gpu::GpuTrails::new(&device, points, pipelines)
        });
        let particles = particles_gpu::ParticleRenderer::new(
            &device,
            &frame_layout,
            &sprite_layout,
            &fluid_shared_source(),
        );
        let lights_2d = light2d::Light2dRenderer::new(&device);
        let ribbon_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ribbon"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let uniform_buffer = |label: &str, size: usize| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: size as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let fx_buffer = uniform_buffer("fx", std::mem::size_of::<fx::FxUniform>());
        let decal_buffers = fx::DecalBuffers::new(&device);
        let sprite_fx_buffer = uniform_buffer("sprite fx", 32);

        let hdr = surface
            .as_ref()
            .filter(|(_, c)| c.color_space.is_hdr())
            .map(|(_, c)| hdr::Output::new(&device, desc, c.color_space, adapter_info.backend));
        let post = post::PostPasses::new(
            &device,
            &queue,
            output_format,
            antialias,
            post::AaOptions {
                taa_history: desc.taa_history,
                taa_sharpen: desc.taa_sharpen.clamp(0.0, 1.0),
            },
            hdr.as_ref().map_or([0.0; 4], |h| h.curve),
        );
        let mips = mips::MipGenerator::new(&device);
        let default_white = solid_texture(&device, &queue, [255; 4]);
        let default_normal = solid_texture(&device, &queue, [128, 128, 255, 255]);
        let terrain = terrain::TerrainRenderer::new(
            &device,
            &queue,
            terrain::TerrainPipelines {
                frame_layout: &frame_layout,
                shadow_layout: &shadow_bind_layout,
                frame_wgsl: FRAME_WGSL,
                lighting_wgsl: LIGHTING_WGSL,
                light_tables_wgsl: &clusters::wgsl(),
                samples,
                white: default_white.clone(),
                flat_normal: default_normal.clone(),
            },
        );
        let scatter = scatter::ScatterRenderer::new(
            &device,
            scatter::ScatterPipelines {
                frame_layout: &frame_layout,
                material_layout: &material_layout,
                shadow_layout: &shadow_bind_layout,
                fx_layout: &fx_layout,
                lit_wgsl: Box::new(scatter_lit_source),
                shadow_wgsl: scatter_shadow_source(),
                samples,
                blue_noise: post::texture_with_data(
                    &device,
                    &queue,
                    crate::blue_noise::BLUE_NOISE_SIZE,
                    wgpu::TextureFormat::R8Unorm,
                    crate::blue_noise::BLUE_NOISE,
                ),
            },
        );
        let grass = grass::GrassRenderer::new(
            &device,
            grass::GrassPipelines {
                frame_layout: &frame_layout,
                shadow_layout: &shadow_bind_layout,
                fx_layout: &fx_layout,
                samples,
            },
        );
        let vertex_buffer = |label: &str, size: u64| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let instance_buffer = vertex_buffer("instances", INSTANCE_STRIDE * 1024);
        let prev_instance_buffer = vertex_buffer("previous instances", INSTANCE_STRIDE * 1024);
        let skinned_buffer = vertex_buffer("skinned instances", skin::SKINNED_INSTANCE_STRIDE * 64);
        let ribbon_buffer = vertex_buffer("strips", 48 * 1024);
        let sprite_buffer = vertex_buffer("sprites", SPRITE_STRIDE * 1024);
        let particle_sprite_buffer = buffers::particle_sprite_buffer(&device, SPRITE_STRIDE * 64);
        let white_sprite_group = sprite_bind_group(
            &device,
            &sprite_layout,
            &default_white,
            &sprite_sampler,
            [0.0; 4],
        );
        let offscreen_linear = linear_view_format(&adapter, output_format, false);
        let persistent = post::Persistent::new(
            &device,
            width,
            height,
            output_format,
            offscreen_linear,
            antialias,
            post.chain_format,
        );
        let surface_linear = surface
            .as_ref()
            .and_then(|(_, config)| config.view_formats.first().copied());
        let overlay = overlay::OverlayRenderer::new(
            &device,
            output_format,
            offscreen_linear.or(surface_linear),
            &sprite_layout,
        );
        let ui = ui::pass::UiRenderer::new(&device, &queue, &overlay, &sprite_layout);
        let glass = glass::GlassPasses::new(&device, output_format);
        let lines = lines::LineRenderer::new(
            &device,
            samples,
            output_format,
            offscreen_linear.or(surface_linear),
        );

        let lost = LostFlag::watch(&device);
        Ok(Renderer {
            device,
            queue,
            info,
            desc: *desc,
            render_scale: 1.0,
            taau: false,
            lost,
            width,
            height,
            out_width: width,
            out_height: height,
            output_format,
            offscreen_linear,
            surface,
            hdr,
            keep_frame: false,
            capture_spare: Default::default(),
            #[cfg(not(target_arch = "wasm32"))]
            offscreen_submissions: Default::default(),
            samples,
            persistent,
            graph: post::FrameGraph::default(),
            clock: None,
            pass_ms: Vec::new(),
            gpu_timer: None,
            cull_inside: CullScratch::default(),
            shelter_key: None,
            probes: probes::Probes::default(),
            sorted: instances::SortedInstances::default(),
            views: views::Views::default(),
            custom_posts: post_custom::CustomPosts::default(),
            post,
            cascade_map,
            local_map,
            full_shadow_size: shadow_size,
            local_shadow_size: LOCAL_SHADOW_SIZE,
            material_sampler,
            sprite_sampler,
            sprite_sampler_nearest,
            trilinear_sampler,
            frame_layout,
            sky_layout,
            cloud_noise,
            sky_params,
            aurora_map,
            material_layout,
            sprite_layout,
            frame_buffer,
            light_buffers,
            lightmaps,
            ltc,
            cookies: area::Cookies::default(),
            volumetric,
            clusters: clusters::ClusterBuilder::default(),
            reflection_buffer,
            reflection_fx_buffer,
            reflection_light_buffers,
            reflection_clusters,
            shadow_buffer,
            shadow_bind_group,
            pbr_layout,
            pbr_skinned_layout,
            pbr_shader,
            shadow_layouts,
            shadow_bind_layout,
            shadow_shader,
            pbr_pipelines: HashMap::new(),
            shadow_pipelines: HashMap::new(),
            prepass_pipelines: HashMap::new(),
            lean_pipelines: HashMap::new(),
            lean_prepass_pipelines: HashMap::new(),
            compact_shadow_pipelines: HashMap::new(),
            stylise_object: None,
            sky_pipeline,
            sky_look_pipeline,
            water,
            fluid3d,
            terrain,
            scatter,
            grass,
            metaballs,
            ibl,
            sky_capture: None,
            gi_probes: None,
            sprite_pipeline,
            sprite_additive_pipeline,
            sprite_untonemapped_pipeline,
            sprite_cover_pipeline,
            sprite_glow_pipeline,
            msdf_pipeline,
            ribbon_pipelines,
            fx_layout,
            fx_lean_layout,
            fx_buffer,
            decal_buffers,
            decal_records: Vec::new(),
            decal_spheres: Vec::new(),
            decal_bins: clusters::DecalBins::default(),
            sprite_fx_layout,
            sprite_fx_buffer,
            palette_layout,
            palette: skin::GpuPalette::new(),
            skinned_buffer,
            prev_instance_buffer,
            prev_instance_upload: upload::DirtyUpload::default(),
            ribbon_buffer,
            ribbon_sampler,
            lines,
            particles,
            lights_2d,
            trails: ribbons::Trails::default(),
            gpu_trails,
            surfaces,
            garments,
            prev_view_proj: None,
            previous_time: None,
            previous_wind: None,
            previous_sway: Vec::new(),
            shadow_cache: Default::default(),
            shadow_cache_maps: Vec::new(),
            warned_planar: false,
            fx_groups: None,
            mips,
            physical: None,
            array_maps: None,
            transmission: None,
            environment: None,
            default_white,
            default_normal,
            white_sprite_group,
            instance_buffer,
            instance_upload: upload::DirtyUpload::default(),
            sprite_buffer,
            particle_sprite_buffer,
            particle_sprite_sources: Vec::new(),
            overlay,
            ui,
            glass,
            glass_targets: None,
            cache: SceneCache::default(),
            frame_index: 0,
            pipeline_disk,
            desc_particles: desc.particles,
            desc_culling: desc.culling,
            indirect,
            gpu_cull: None,
            meshlet: None,
            culling_counts: false,
            taa_history: None,
            taa_history_frames: 0,
            gi_history: None,
            drawn_origin: Vec3::ZERO,
            stylise_chain: None,
            sprite_arrays: None,
            sprite_meshes: None,
            sprite_skin_supported: sprite_skin::SkinRenderer::supported(
                adapter.get_downlevel_capabilities().flags,
            ),
            sprite_normals: None,
            oit,
            sprite_shader,
            ribbon_shader,
            tiles_gpu: None,
            tiles_gpu_supported: tiles_gpu::Tiles::supported(
                adapter.get_downlevel_capabilities().flags,
            ),
        })
    }
}

/// The adapter's power preference for `desc` ([`RenderDesc::low_power`]).
fn power_preference(desc: &RenderDesc) -> wgpu::PowerPreference {
    if desc.low_power {
        wgpu::PowerPreference::LowPower
    } else {
        wgpu::PowerPreference::HighPerformance
    }
}
