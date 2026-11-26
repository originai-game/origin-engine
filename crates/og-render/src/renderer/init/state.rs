//! The stage states `from_adapter` assembles, each from the outputs of the
//! init stages that make its parts (init/*.rs).

use super::*;

impl Visibility {
    /// Where the frame culls ([`RenderDesc::culling`]) and whether the
    /// adapter can on the GPU (init/adapter.rs); the GPU paths are made by
    /// the first frame that uses them.
    pub(super) fn new(indirect: bool) -> Visibility {
        Visibility {
            cull_inside: CullScratch::default(),
            indirect,
            gpu_cull: None,
            meshlet: None,
            culling_counts: false,
            records: cfg!(not(feature = "publish")).then(Default::default),
            lean: false,
        }
    }
}

impl Shadows {
    /// The shadow passes' state: the cascade and local arrays
    /// (init/samplers.rs), the cascades' full side (init/adapter.rs), the
    /// shadow uniform and its group (init/uniforms.rs), and the shadow
    /// layouts and shader (init/shaders.rs). No pipeline, cache or cookie
    /// yet.
    pub(super) fn new(
        (cascade_map, local_map): (ShadowMap, ShadowMap),
        full_shadow_size: u32,
        (shadow_buffer, shadow_bind_group): (wgpu::Buffer, wgpu::BindGroup),
        shadow_layouts: [wgpu::PipelineLayout; 3],
        shadow_shader: deferred::Module,
    ) -> Shadows {
        Shadows {
            cascade_map,
            local_map,
            full_shadow_size,
            local_shadow_size: LOCAL_SHADOW_SIZE,
            shadow_buffer,
            shadow_bind_group,
            shadow_layouts,
            shadow_shader,
            shadow_pipelines: HashMap::new(),
            compact_shadow_pipelines: HashMap::new(),
            shadow_cache: Default::default(),
            shadow_keep: None,
            shadow_cache_maps: Vec::new(),
            shadow_hashes: Default::default(),
            shelter_key: None,
            cookies: area::Cookies::default(),
        }
    }
}

impl InstanceRecords {
    /// The instance records' buffers at their first sizes: the
    /// instances' and the previous frame's (the debris layer writes its
    /// pieces' transforms into them in compute, `debris.rs`), then the
    /// skinned instances'.
    pub(super) fn new(device: &wgpu::Device) -> InstanceRecords {
        let instance_records = |label: &str| {
            crate::traffic::create_buffer(
                device,
                &wgpu::BufferDescriptor {
                    label: Some(label),
                    size: INSTANCE_STRIDE * 1024,
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::STORAGE,
                    mapped_at_creation: false,
                },
            )
        };
        let instance_buffer = instance_records("instances");
        let prev_instance_buffer = instance_records("previous instances");
        let skinned_buffer = crate::traffic::create_buffer(
            device,
            &wgpu::BufferDescriptor {
                label: Some("skinned instances"),
                size: skin::SKINNED_INSTANCE_STRIDE * 64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );
        InstanceRecords {
            sorted: instances::SortedInstances::default(),
            instance_buffer,
            instance_upload: upload::DirtyUpload::default(),
            prev_instance_buffer,
            prev_instance_upload: upload::DirtyUpload::default(),
            skinned_buffer,
            palette: skin::GpuPalette::new(),
            previous_sway: Vec::new(),
            sway_serial: None,
            ahead: false,
            debris: Default::default(),
        }
    }
}

impl SpriteState {
    /// The sprites' state: their pipelines (init/sprite_pipelines.rs),
    /// samplers (init/samplers.rs), layouts (init/layouts.rs), effects
    /// uniform and instance buffer, the white group and their shader
    /// (init/shaders.rs); the optional renderers are made by the first
    /// frame that draws with them, where `flags` allows.
    pub(super) fn new(
        pipelines: SpritePipelinesOutputs,
        (sprite_sampler, sprite_sampler_nearest): (wgpu::Sampler, wgpu::Sampler),
        (sprite_layout, sprite_fx_layout): (wgpu::BindGroupLayout, wgpu::BindGroupLayout),
        (sprite_fx_buffer, sprite_buffer): (wgpu::Buffer, wgpu::Buffer),
        white_sprite_group: wgpu::BindGroup,
        flags: wgpu::DownlevelFlags,
        sprite_shader: deferred::Module,
    ) -> SpriteState {
        let SpritePipelinesOutputs {
            additive: _,
            sprite_pipeline,
            sprite_additive_pipeline,
            sprite_untonemapped_pipeline,
            sprite_cover_pipeline,
            plate_pipeline,
            msdf_pipeline,
            sprite_glow_pipeline,
        } = pipelines;
        SpriteState {
            sprite_sampler,
            sprite_sampler_nearest,
            sprite_layout,
            sprite_pipeline,
            sprite_additive_pipeline,
            sprite_untonemapped_pipeline,
            sprite_cover_pipeline,
            plate_pipeline,
            sprite_glow_pipeline,
            msdf_pipeline,
            sprite_fx_layout,
            sprite_fx_buffer,
            sprite_buffer,
            white_sprite_group,
            sprite_arrays: None,
            sprite_meshes: None,
            sprite_skin_supported: sprite_skin::SkinRenderer::supported(flags),
            sprite_normals: None,
            sprite_hooks: None,
            tiles_gpu: None,
            tiles_gpu_supported: tiles_gpu::Tiles::supported(flags),
            sprite_shader,
        }
    }
}

impl SkyState {
    /// The sky's state: its layout (init/layouts.rs), and its pipelines,
    /// cloud noise, parameters and aurora map (init/sky.rs); the live
    /// sky's capture is made by the first frame that draws the sky.
    pub(super) fn new(
        sky_layout: wgpu::BindGroupLayout,
        (sky_pipeline, sky_look_pipeline): (
            Sampled<wgpu::RenderPipeline>,
            Sampled<wgpu::RenderPipeline>,
        ),
        (clouds_low, ground): (clouds::CloudsLow, ground_heights::GroundCatcher),
        (cloud_noise, cloud_shadow): (clouds::CloudNoise, clouds::CloudShadow),
        sky_params: wgpu::Buffer,
        (aurora_map, sky_lut): (aurora::AuroraMap, crate::media::sky_lut::SkyLut),
    ) -> SkyState {
        SkyState {
            sky_layout,
            cloud_noise,
            cloud_shadow,
            sky_params,
            aurora_map,
            sky_lut,
            sky_pipeline,
            sky_look_pipeline,
            clouds_low,
            ground,
            sky_capture: None,
        }
    }
}

impl Lights {
    /// The frame's lighting: the frame uniform and the light tables, the
    /// LTC table and the caustics grid (init/uniforms.rs), the lightmaps
    /// (init/shaders.rs), the IBL (init/sky.rs), and the effect uniform
    /// and decal buffers; the clusters skip the rect lights the renderer
    /// was made without (`off`).
    pub(super) fn new(
        (frame_buffer, light_tables): (wgpu::Buffer, wgpu::Buffer),
        (ltc, caustic_grid): (wgpu::Buffer, wgpu::Buffer),
        lightmaps: lightmap::Lightmaps,
        ibl: ibl::IblGenerator,
        (fx_buffer, decal_buffers): (wgpu::Buffer, crate::decals::DecalBuffers),
        off: pipeline_features::Features,
    ) -> Lights {
        Lights {
            frame_buffer,
            light_tables,
            records: Vec::new(),
            clusters: {
                let mut c = clusters::ClusterBuilder::default();
                c.skip_rects = off.contains(crate::pipeline_features::Features::AREA);
                c
            },
            static_far: Default::default(),
            lightmaps,
            ltc,
            caustic_grid,
            ibl,
            environment: None,
            environment_unbaked: false,
            decal_buffers,
            decal_records: Vec::new(),
            decal_spheres: Vec::new(),
            decal_bins: clusters::DecalBins::default(),
            decal_atlas: None,
            decal_copies: None,
            fx_buffer,
            fx_groups: None,
        }
    }
}

impl ReflectionTargets {
    /// The planar reflection slots' buffers and clusters (init/uniforms.rs).
    pub(super) fn new(
        reflection_buffer: Vec<wgpu::Buffer>,
        reflection_fx_buffer: Vec<wgpu::Buffer>,
        reflection_light_tables: Vec<wgpu::Buffer>,
        reflection_clusters: Vec<clusters::ClusterBuilder>,
    ) -> ReflectionTargets {
        ReflectionTargets {
            reflection_buffer,
            reflection_fx_buffer,
            reflection_light_tables,
            reflection_clusters,
            warned_planar: false,
        }
    }
}

impl LitPipelines {
    /// The lit passes' layouts and shader (init/shaders.rs); every
    /// pipeline is made at first use.
    pub(super) fn new(
        pbr_layout: wgpu::PipelineLayout,
        pbr_skinned_layout: wgpu::PipelineLayout,
        pbr_shader: deferred::Module,
    ) -> LitPipelines {
        LitPipelines {
            pbr_layout,
            pbr_skinned_layout,
            pbr_shader,
            pbr_pipelines: HashMap::new(),
            plain_effects: false,
            lean_pipelines: HashMap::new(),
            physical: None,
            array_maps: None,
            bindless: None,
            warned_without: LitFeatures::NONE,
        }
    }
}

impl PrepassState {
    /// No prepass pipeline yet: each is made at first use.
    pub(super) fn new() -> PrepassState {
        PrepassState {
            prepass_pipelines: HashMap::new(),
            lean_prepass_pipelines: HashMap::new(),
            stylise_object: None,
        }
    }
}

impl WorldRenderers {
    /// The 3D world's renderers: the simulations, water and fluids
    /// (init/world_sims.rs), the terrain, scatter and grass
    /// (init/world_ground.rs), and whether crowds can be culled and drawn
    /// on the GPU (init/adapter.rs); the crowds' GPU path and impostors
    /// are made by the first frame that uses them.
    pub(super) fn new(
        sims: WorldSimsOutputs,
        ground: WorldGroundOutputs,
        crowd_indirect: bool,
    ) -> WorldRenderers {
        let WorldSimsOutputs {
            surfaces,
            garments,
            pre_skin,
            visual,
            visual_fluids,
            water,
            vegetation_compute: _,
            wind_field,
            trample,
            metaballs,
            fluid3d,
        } = sims;
        let WorldGroundOutputs {
            terrain,
            terrain_fx,
            scatter,
            grass,
        } = ground;
        WorldRenderers {
            water,
            fluid3d,
            terrain,
            terrain_fx,
            scatter,
            grass,
            metaballs,
            wind_field,
            trample,
            visual_fluids,
            surfaces,
            garments,
            pre_skin,
            visual,
            crowd_indirect,
            crowd_gpu: None,
            crowd_tables: Default::default(),
            crowd_impostors: None,
        }
    }
}

impl StripState {
    /// The strips' state: their pipelines and GPU trails
    /// (init/strips.rs), vertex buffer, sampler and shader
    /// (init/shaders.rs); no CPU trail yet.
    pub(super) fn new(
        pipelines: StripPipelinesOutputs,
        ribbon_pl: wgpu::PipelineLayout,
        additive: wgpu::BlendState,
        ribbon_buffer: wgpu::Buffer,
        ribbon_sampler: wgpu::Sampler,
        ribbon_shader: deferred::Module,
    ) -> StripState {
        let StripPipelinesOutputs {
            ribbon_pipelines,
            strip_pl,
            gpu_trails,
        } = pipelines;
        StripState {
            ribbon_pipelines,
            ribbon_pl,
            strip_pl,
            additive,
            hook_modules: HashMap::new(),
            hook_pipelines: HashMap::new(),
            ribbon_buffer,
            ribbon_sampler,
            ribbon_shader,
            trails: ribbons::Trails::default(),
            gpu_trails,
        }
    }

    /// Compiles (and caches) a strip shader module with `hook`'s effect hook.
    pub(crate) fn hook_module(
        &mut self,
        device: &wgpu::Device,
        hook: usize,
        shaders: &[MaterialShader],
    ) -> wgpu::ShaderModule {
        self.hook_modules
            .entry(hook)
            .or_insert_with(|| {
                crate::gpu::shader_module(
                    device,
                    wgpu::ShaderModuleDescriptor {
                        label: Some("ribbon (effect hook)"),
                        source: wgpu::ShaderSource::Wgsl(
                            ribbons_gpu::ribbon_source_with(shaders.get(hook)).into(),
                        ),
                    },
                )
            })
            .clone()
    }

    /// Ensures non-OIT hooked strip pipelines for every hooked run in `runs`.
    pub(crate) fn ensure_hook_pipelines(
        &mut self,
        device: &wgpu::Device,
        runs: &[ribbons::RibbonRun],
        samples: u32,
        diagnostic: bool,
        shaders: &[MaterialShader],
    ) {
        let alpha_blend = wgpu::BlendState {
            color: wgpu::BlendComponent::OVER,
            alpha: self.additive.alpha,
        };
        for run in runs {
            let Some(hook) = run.hook else {
                continue;
            };
            let points = run.points && self.gpu_trails.is_some();
            let blend = if diagnostic {
                ParticleBlend::Alpha
            } else {
                run.blend
            };
            let alpha_idx = usize::from(blend != ParticleBlend::Additive);
            let key = (points, alpha_idx, samples, hook);
            if self.hook_pipelines.contains_key(&key) {
                continue;
            }
            let layout = if points {
                self.strip_pl
                    .as_ref()
                    .expect("gpu_trails has strip_pl")
                    .clone()
            } else {
                self.ribbon_pl.clone()
            };
            let module = self.hook_module(device, hook, shaders);
            let blend_state = if alpha_idx == 0 {
                self.additive
            } else {
                alpha_blend
            };
            let device = device.clone();
            let p = Deferred::anywhere(move || {
                hdr_pipeline(
                    &device,
                    HdrPipeline {
                        alpha_to_coverage: false,
                        label: if points {
                            "strip (hook)"
                        } else {
                            "ribbon (hook)"
                        },
                        layout: &layout,
                        module: &module,
                        vs: if points { "vs_strip" } else { "vs" },
                        fs: if alpha_idx == 0 { "fs" } else { "fs_alpha" },
                        buffers: if points { &[] } else { RIBBON_BUFFERS },
                        cull: None,
                        depth_write: false,
                        depth_compare: wgpu::CompareFunction::GreaterEqual,
                        blend: Some(blend_state),
                        samples,
                    },
                )
            });
            crate::deferred::Start::start(&p);
            self.hook_pipelines.insert(key, p);
        }
    }
}

impl history::FrameHistory {
    /// No frame drawn yet: no history.
    pub(super) fn new() -> history::FrameHistory {
        history::FrameHistory {
            prev_view_proj: None,
            previous_time: None,
            previous_wind: None,
            taa_history: None,
            taa_history_frames: 0,
            drawn_origin: Vec3::ZERO,
        }
    }
}
