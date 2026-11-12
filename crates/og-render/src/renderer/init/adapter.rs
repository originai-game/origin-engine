//! What the renderer reads off its adapter before it makes the device: the
//! capability set (the adapter's, less what the caller treats as absent,
//! under the tier's ceiling), antialiasing and samples, the limits and
//! features it asks for, the shadow sizes, and the RenderInfo it reports.

use super::*;

pub(super) struct AdapterPlanInputs<'a> {
    pub adapter: &'a wgpu::Adapter,
    pub desc: &'a RenderDesc,
    pub surface: Option<&'a (wgpu::Surface<'static>, wgpu::SurfaceConfiguration)>,
}

pub(super) struct AdapterPlanOutputs {
    pub adapter_info: wgpu::AdapterInfo,
    pub caps: Caps,
    pub flags: wgpu::DownlevelFlags,
    pub indirect: bool,
    pub antialias: Antialias,
    pub samples: u32,
    pub limits: wgpu::Limits,
    pub shadow_size: u32,
    pub cascades: u32,
    pub cache_dir: Option<std::path::PathBuf>,
    pub crowd_indirect: bool,
    pub info: RenderInfo,
    pub features: wgpu::Features,
}

impl Renderer {
    /// Reads the adapter's capabilities and plans the device the renderer asks for.
    #[allow(clippy::too_many_lines)]
    pub(super) fn adapter_plan(
        AdapterPlanInputs {
            adapter,
            desc,
            surface,
        }: AdapterPlanInputs<'_>,
    ) -> AdapterPlanOutputs {
        let adapter_info = adapter.get_info();
        // What the adapter has, less what the caller treats as absent,
        // under the tier's ceiling (caps.rs): every decision below about
        // the device reads these.
        let adapter_features = adapter.features() - desc.without_features;
        let adapter_flags = adapter.get_downlevel_capabilities().flags;
        let caps =
            Caps::of(adapter_features, adapter_flags, &adapter.limits()) & desc.tier.ceiling();
        let adapter_features = caps.features(adapter_features);
        let flags = caps.downlevel(adapter_flags);
        // GPU culling and the meshlet path: compute dispatches and indirect draws.
        let indirect = caps.contains(Caps::INDIRECT | Caps::COMPUTE);
        let msaa_ok = supports_msaa(adapter);
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
        use crate::texture::TextureFormatsChoice;
        let (mut features, texture_formats) =
            TextureFormats::choose(adapter_features, desc.texture_compression);
        let cache_dir = pipeline_cache::dir();
        if cache_dir.is_some() && adapter_features.contains(wgpu::Features::PIPELINE_CACHE) {
            features |= wgpu::Features::PIPELINE_CACHE;
        }
        // GPU pass timing (`timing.rs`) where the adapter has timestamps
        // (a publish build has none), and og.motion's slice timing when
        // asked (`RenderDesc::timestamps`), publish too.
        if (cfg!(not(feature = "publish")) || desc.timestamps) && caps.contains(Caps::TIMESTAMPS) {
            features |= wgpu::Features::TIMESTAMP_QUERY;
        }
        // Crowds on the GPU-driven path (crowd_gpu.rs) start each level's
        // indirect draw at the first instance the cull found; without the
        // feature they are drawn through the CPU's skinned lists.
        let crowd_indirect =
            caps.contains(Caps::COMPUTE | Caps::INDIRECT | Caps::INDIRECT_FIRST_INSTANCE);
        if crowd_indirect {
            features |= wgpu::Features::INDIRECT_FIRST_INSTANCE;
        }
        let mut info = RenderInfo {
            adapter: adapter_info.name.clone(),
            backend: adapter_info.backend,
            device_type: adapter_info.device_type,
            hdr_supported: surface
                .is_some_and(|(s, _)| hdr::surface_space(&s.get_capabilities(adapter)).is_some()),
            hdr: surface.is_some_and(|(_, c)| c.color_space.is_hdr()),
            antialias,
            msaa_samples: samples,
            shadow_map_size: shadow_size,
            shadow_cascades: cascades,
            shadowed_lights: MAX_SHADOWED_LIGHTS as u32,
            point_shadows: desc.point_shadows.resolve(),
            texture_formats,
            pipeline_cache: false,
            max_texture_size: limits.max_texture_dimension_2d,
            // The froxel grid is a compute pass: the raymarch without one.
            volumetric: match desc.volumetric {
                _ if !caps.contains(Caps::COMPUTE) => VolumetricMethod::Raymarch,
                v => v.unwrap_or(VolumetricMethod::Froxel),
            },
            volumetric_lights: 0,
            sprite_anisotropy: if flags.contains(wgpu::DownlevelFlags::ANISOTROPIC_FILTERING)
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
            without: desc.without,
            transparency: Transparency::Weighted,
            gpu_crowds: crowd_indirect,
            visual_simulation: ribbons_gpu::GpuTrails::supported(flags),
            indirect_per_draw: desc.indirect_per_draw || webkit(),
            caps,
            tier: desc.tier,
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
        AdapterPlanOutputs {
            adapter_info,
            caps,
            flags,
            indirect,
            antialias,
            samples,
            limits,
            shadow_size,
            cascades,
            cache_dir,
            crowd_indirect,
            info,
            features,
        }
    }
}
