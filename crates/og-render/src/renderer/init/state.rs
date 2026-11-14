//! The stage states `from_adapter` assembles, each from the outputs of the
//! init stages that make its parts (init/*.rs).

use super::*;

impl Visibility {
    /// Where the frame culls ([`RenderDesc::culling`]) and whether the
    /// adapter can on the GPU (init/adapter.rs); the GPU paths are made by
    /// the first frame that uses them.
    pub(super) fn new(desc_culling: Culling, indirect: bool) -> Visibility {
        Visibility {
            cull_inside: CullScratch::default(),
            desc_culling,
            indirect,
            gpu_cull: None,
            meshlet: None,
            culling_counts: false,
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
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: INSTANCE_STRIDE * 1024,
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            })
        };
        let instance_buffer = instance_records("instances");
        let prev_instance_buffer = instance_records("previous instances");
        let skinned_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("skinned instances"),
            size: skin::SKINNED_INSTANCE_STRIDE * 64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        InstanceRecords {
            sorted: instances::SortedInstances::default(),
            instance_buffer,
            instance_upload: upload::DirtyUpload::default(),
            prev_instance_buffer,
            prev_instance_upload: upload::DirtyUpload::default(),
            skinned_buffer,
            palette: skin::GpuPalette::new(),
            previous_sway: Vec::new(),
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
        clouds_low: clouds::CloudsLow,
        (cloud_noise, cloud_shadow): (clouds::CloudNoise, clouds::CloudShadow),
        sky_params: wgpu::Buffer,
        aurora_map: aurora::AuroraMap,
    ) -> SkyState {
        SkyState {
            sky_layout,
            cloud_noise,
            cloud_shadow,
            sky_params,
            aurora_map,
            sky_pipeline,
            sky_look_pipeline,
            clouds_low,
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
        (frame_buffer, light_buffers): (wgpu::Buffer, [wgpu::Buffer; 3]),
        (ltc, caustic_grid): (wgpu::Buffer, wgpu::Buffer),
        lightmaps: lightmap::Lightmaps,
        ibl: ibl::IblGenerator,
        (fx_buffer, decal_buffers): (wgpu::Buffer, crate::decals::DecalBuffers),
        off: pipeline_features::Features,
    ) -> Lights {
        Lights {
            frame_buffer,
            light_buffers,
            clusters: {
                let mut c = clusters::ClusterBuilder::default();
                c.skip_rects = off.contains(crate::pipeline_features::Features::AREA);
                c
            },
            lightmaps,
            ltc,
            caustic_grid,
            ibl,
            environment: None,
            decal_buffers,
            decal_records: Vec::new(),
            decal_spheres: Vec::new(),
            decal_bins: clusters::DecalBins::default(),
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
        reflection_light_buffers: Vec<[wgpu::Buffer; 3]>,
        reflection_clusters: Vec<clusters::ClusterBuilder>,
    ) -> ReflectionTargets {
        ReflectionTargets {
            reflection_buffer,
            reflection_fx_buffer,
            reflection_light_buffers,
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
            scatter,
            grass,
        } = ground;
        WorldRenderers {
            water,
            fluid3d,
            terrain,
            scatter,
            grass,
            metaballs,
            wind_field,
            trample,
            visual_fluids,
            surfaces,
            garments,
            visual,
            crowd_indirect,
            crowd_gpu: None,
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
        ribbon_buffer: wgpu::Buffer,
        ribbon_sampler: wgpu::Sampler,
        ribbon_shader: deferred::Module,
    ) -> StripState {
        let StripPipelinesOutputs {
            ribbon_pipelines,
            gpu_trails,
        } = pipelines;
        StripState {
            ribbon_pipelines,
            ribbon_buffer,
            ribbon_sampler,
            ribbon_shader,
            trails: ribbons::Trails::default(),
            gpu_trails,
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
