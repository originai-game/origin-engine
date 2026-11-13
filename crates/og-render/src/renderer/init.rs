//! Renderer creation and adapter initialization.

use super::*;

mod sprite_pipelines;
use sprite_pipelines::{SpritePipelinesInputs, SpritePipelinesOutputs};
mod layouts;
use layouts::{LayoutsInputs, LayoutsOutputs};
mod samplers;
use samplers::{SamplersAndShadowMapsInputs, SamplersAndShadowMapsOutputs};
mod uniforms;
use uniforms::{UniformBuffersInputs, UniformBuffersOutputs};
mod sky;
use sky::{SkyResourcesInputs, SkyResourcesOutputs};
mod strips;
use strips::{StripPipelinesInputs, StripPipelinesOutputs};
mod adapter;
use adapter::{AdapterPlanInputs, AdapterPlanOutputs};
mod world_sims;
use world_sims::{WorldSimsInputs, WorldSimsOutputs};
mod world_ground;
use world_ground::{WorldGroundInputs, WorldGroundOutputs};
mod shaders;

mod state;
use shaders::{ShadersAndLayoutsInputs, ShadersAndLayoutsOutputs};
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
        } else if self.gpu_timer.is_none() && self.info.caps.contains(Caps::TIMESTAMPS) {
            self.gpu_timer = timing::GpuTimer::new(&self.device, &self.queue);
        }
    }

    /// The last frame's render-graph passes in execution order, each with
    /// the clock's reading when its encoding began and the milliseconds it
    /// took; empty without a clock.
    pub fn pass_timings(&self) -> &[PassTiming] {
        &self.pass_ms
    }

    /// What this device said when it refused the visibility buffer's
    /// resolve pipelines, or `None` (it made them, or no clustered mesh
    /// has asked for them yet). Once refused, the camera's small clusters
    /// draw on the hardware meshlet path, as with
    /// `RenderScene::visibility_buffer` off.
    pub fn visibility_buffer_refused(&self) -> Option<&str> {
        self.visibility
            .meshlet
            .as_ref()?
            .visibility_buffer_refused()
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

    /// The material shaders whose sprite pipelines failed to compile on
    /// this device, by shader index, with the device's message: their
    /// sprites draw plain (`sprite_hooks.rs`).
    pub fn sprite_shader_errors(&self) -> Vec<(usize, String)> {
        self.sprite
            .sprite_hooks
            .as_ref()
            .map_or_else(Vec::new, |h| h.errors())
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
        Self::from_adapter(adapter, None, format, desc, None).await
    }

    /// A fresh offscreen run. Mutable resources are
    /// constructed again; only the device and compatible pipelines survive.
    /// A changed description or lost device takes the ordinary cold path.
    pub async fn restart(self, desc: &RenderDesc) -> Result<Renderer, RenderError> {
        if renderer_disabled() {
            return Err(disabled_error());
        }
        if self.lost().is_some() || self.desc != *desc {
            return Self::new(desc).await;
        }
        let mut fresh = Self::from_adapter(
            self.adapter.clone(),
            None,
            self.output_format,
            &self.desc,
            Some((self.device.clone(), self.queue.clone(), self.pipeline_disk)),
        )
        .await?;
        if self.samples == fresh.samples {
            fresh.lit.pbr_pipelines = self.lit.pbr_pipelines;
            fresh.shadows.shadow_pipelines = self.shadows.shadow_pipelines;
            fresh.prepass.prepass_pipelines = self.prepass.prepass_pipelines;
            fresh.lit.lean_pipelines = self.lit.lean_pipelines;
            fresh.prepass.lean_prepass_pipelines = self.prepass.lean_prepass_pipelines;
            fresh.shadows.compact_shadow_pipelines = self.shadows.compact_shadow_pipelines;
            fresh.sky.sky_pipeline = self.sky.sky_pipeline;
            fresh.sky.sky_look_pipeline = self.sky.sky_look_pipeline;
            fresh.sky.clouds_low = self.sky.clouds_low;
            fresh.sprite.sprite_pipeline = self.sprite.sprite_pipeline;
            fresh.sprite.sprite_additive_pipeline = self.sprite.sprite_additive_pipeline;
            fresh.sprite.sprite_untonemapped_pipeline = self.sprite.sprite_untonemapped_pipeline;
            fresh.sprite.sprite_cover_pipeline = self.sprite.sprite_cover_pipeline;
            fresh.sprite.sprite_glow_pipeline = self.sprite.sprite_glow_pipeline;
            fresh.sprite.msdf_pipeline = self.sprite.msdf_pipeline;
            fresh.strips.ribbon_pipelines = self.strips.ribbon_pipelines;
        }
        Ok(fresh)
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
        // (frame capture on the web host: `set_keep_frame`). A WebGPU
        // canvas always allows COPY_SRC (GPUCanvasConfiguration.usage), but
        // wgpu 30's web backend reports RENDER_ATTACHMENT alone and passes
        // the configured usage through unchecked, so the web asks for it.
        let copy = cfg!(target_arch = "wasm32")
            || surface
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
        Self::from_adapter(adapter, Some((surface, config)), format, desc, None).await
    }

    async fn from_adapter(
        adapter: wgpu::Adapter,
        surface: Option<(wgpu::Surface<'static>, wgpu::SurfaceConfiguration)>,
        output_format: wgpu::TextureFormat,
        desc: &RenderDesc,
        shared: Option<(wgpu::Device, wgpu::Queue, Option<pipeline_cache::DiskCache>)>,
    ) -> Result<Renderer, RenderError> {
        // The adapter's capabilities and the device's plan (init/adapter.rs).
        let AdapterPlanOutputs {
            adapter_info,
            mut caps,
            flags,
            indirect,
            antialias,
            samples,
            limits,
            shadow_size,
            cascades,
            cache_dir,
            crowd_indirect,
            mut info,
            features,
        } = Self::adapter_plan(AdapterPlanInputs {
            adapter: &adapter,
            desc,
            surface: surface.as_ref(),
        });
        let (device, queue, pipeline_disk) = match shared {
            Some(pair) => pair,
            None => {
                let (device, queue) = adapter
                    .request_device(&wgpu::DeviceDescriptor {
                        label: Some("og-render"),
                        required_features: features,
                        required_limits: limits,
                        ..Default::default()
                    })
                    .await
                    .map_err(|e| RenderError::NoDevice(e.to_string()))?;
                let disk = cache_dir
                    .and_then(|dir| pipeline_cache::DiskCache::open(&device, &adapter_info, dir));
                (device, queue, disk)
            }
        };
        // Before any pipeline is made: every one is made with the cache.
        info.pipeline_cache = pipeline_disk.is_some();
        // Timestamps only where the device was made with them (not asked
        // for in a publish build, or a shared device without them).
        if !device.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
            caps = caps.without(Caps::TIMESTAMPS);
        }
        info.caps = caps;
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

        // The bind group layouts (init/layouts.rs).
        let LayoutsOutputs {
            frame_layout,
            oit,
            sky_layout,
            material_layout,
            planar,
            fx_layout,
            fx_lean_layout,
            sprite_fx_layout,
            palette_layout,
            sprite_layout,
            shadow_bind_layout,
        } = Self::layouts(LayoutsInputs {
            device: &device,
            desc,
            flags,
            info: &info,
        });
        // What the OIT state could make (oit.rs), read by the frame.
        info.transparency = oit.mode;
        // Samplers and shadow maps (init/samplers.rs).
        let SamplersAndShadowMapsOutputs {
            material_sampler,
            sprite_sampler,
            trilinear_sampler,
            sprite_sampler_nearest,
            cascade_map,
            local_map,
        } = Self::samplers_and_shadow_maps(SamplersAndShadowMapsInputs {
            device: &device,
            info: &info,
            shadow_size,
            cascades,
        });
        // Uniform buffers (init/uniforms.rs).
        let UniformBuffersOutputs {
            frame_buffer,
            light_buffers,
            ltc,
            caustic_grid,
            volumetric,
            reflection_light_buffers,
            reflection_buffer,
            reflection_clusters,
            reflection_fx_buffer,
            shadow_buffer,
            shadow_bind_group,
        } = Self::uniform_buffers(UniformBuffersInputs {
            device: &device,
            queue: &queue,
            info: &info,
            shadow_bind_layout: &shadow_bind_layout,
        });
        // Shader modules and shared pipeline layouts (init/shaders.rs).
        let ShadersAndLayoutsOutputs {
            off,
            pbr_shader,
            sky_shader,
            sprite_shader,
            ribbon_shader,
            shadow_shader,
            pbr_layout,
            lightmaps,
            pbr_skinned_layout,
            sky_pl,
            sprite_pl,
            effects,
            ribbon_pl,
            shadow_layouts,
        } = Self::shaders_and_layouts(ShadersAndLayoutsInputs {
            device: &device,
            queue: &queue,
            info: &info,
            planar,
            frame_layout: &frame_layout,
            material_layout: &material_layout,
            fx_layout: &fx_layout,
            palette_layout: &palette_layout,
            sky_layout: &sky_layout,
            sprite_layout: &sprite_layout,
            sprite_fx_layout: &sprite_fx_layout,
            shadow_bind_layout: &shadow_bind_layout,
        });
        // The sky's pipelines and resources (init/sky.rs).
        let SkyResourcesOutputs {
            sky_pipeline,
            sky_look_pipeline,
            clouds_low,
            ibl,
            cloud_noise,
            sky_params,
            aurora_map,
        } = Self::sky_resources(SkyResourcesInputs {
            frame_layout: &frame_layout,
            sky_layout: &sky_layout,
            device: &device,
            queue: &queue,
            sky_pl: &sky_pl,
            sky_shader: &sky_shader,
            samples,
        });
        // The world's simulations, water and fluids (init/world_sims.rs).
        let sims = Self::world_sims(WorldSimsInputs {
            device: &device,
            flags,
            frame_layout: &frame_layout,
            off,
            samples,
        });
        // The sprites' pipelines (init/sprite_pipelines.rs).
        let sprite_pipelines = Self::sprite_pipelines(SpritePipelinesInputs {
            device: &device,
            sprite_pl: &sprite_pl,
            sprite_shader: &sprite_shader,
            samples,
        });
        // The strips' pipelines (init/strips.rs).
        let strip_pipelines = Self::strip_pipelines(StripPipelinesInputs {
            device: &device,
            ribbon_pl: &ribbon_pl,
            ribbon_shader: &ribbon_shader,
            samples,
            additive: sprite_pipelines.additive,
            flags,
            frame_layout: &frame_layout,
            sprite_layout: &sprite_layout,
            effects_layout: &effects.layout,
        });
        let particles = Particles::new(
            &device,
            effects,
            &frame_layout,
            &sprite_layout,
            &off.leave_out(fluid_shared_source()),
            desc.particles,
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
            .map(|(_, c)| {
                hdr::Output::new(&device, &queue, desc, c.color_space, adapter_info.backend)
            });
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
        let conformed = {
            let (size, bytes) = crate::stylise::conformed_table(&[]);
            (
                0,
                crate::post::texture_with_data(
                    &device,
                    &queue,
                    size,
                    wgpu::TextureFormat::R32Uint,
                    &bytes,
                ),
            )
        };
        let default_normal = solid_texture(&device, &queue, [128, 128, 255, 255]);
        // The terrain, scatter and grass (init/world_ground.rs).
        let ground = Self::world_ground(WorldGroundInputs {
            device: &device,
            queue: &queue,
            frame_layout: &frame_layout,
            shadow_bind_layout: &shadow_bind_layout,
            material_layout: &material_layout,
            fx_layout: &fx_layout,
            off,
            samples,
            default_white: &default_white,
            default_normal: &default_normal,
            vegetation_compute: sims.vegetation_compute,
            info: &info,
            wind_field: &sims.wind_field,
            trample: &sims.trample,
        });
        let instances = InstanceRecords::new(&device);
        let vertex_buffer = |label: &str, size: u64| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let ribbon_buffer = vertex_buffer("strips", 48 * 1024);
        let sprite_buffer = vertex_buffer("sprites", SPRITE_STRIDE * 1024);
        let white_sprite_group = sprite_bind_group(
            &device,
            &queue,
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
        let glass = glass::GlassPasses::new(&device, &queue, output_format);
        let lines = lines::LineRenderer::new(
            &device,
            &material_layout,
            samples,
            output_format,
            offscreen_linear.or(surface_linear),
        );

        let lost = LostFlag::watch(&device);
        let ssr = ssr::SsrRenderer::new(&device);
        Ok(Renderer {
            adapter: adapter.clone(),
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
            visibility: Visibility::new(desc.culling, indirect),
            shadows: Shadows::new(
                (cascade_map, local_map),
                shadow_size,
                (shadow_buffer, shadow_bind_group),
                shadow_layouts,
                shadow_shader,
            ),
            probes: probes::Probes::default(),
            instances,
            views: views::Views::default(),
            custom_posts: post_custom::CustomPosts::default(),
            post,
            material_sampler,
            sprite: SpriteState::new(
                sprite_pipelines,
                (sprite_sampler, sprite_sampler_nearest),
                (sprite_layout, sprite_fx_layout),
                (sprite_fx_buffer, sprite_buffer),
                white_sprite_group,
                flags,
                sprite_shader,
            ),
            trilinear_sampler,
            frame_layout,
            sky: SkyState::new(
                sky_layout,
                (sky_pipeline, sky_look_pipeline),
                clouds_low,
                cloud_noise,
                sky_params,
                aurora_map,
            ),
            material_layout,
            lights: Lights::new(
                (frame_buffer, light_buffers),
                (ltc, caustic_grid),
                lightmaps,
                ibl,
                (fx_buffer, decal_buffers),
                off,
            ),
            volumetric,
            reflections: ReflectionTargets::new(
                reflection_buffer,
                reflection_fx_buffer,
                reflection_light_buffers,
                reflection_clusters,
            ),
            lit: LitPipelines::new(pbr_layout, pbr_skinned_layout, pbr_shader),
            shadow_bind_layout,
            frame_features: crate::pipeline_features::Features::ALL,
            capturing: false,
            prepass: PrepassState::new(),
            world: WorldRenderers::new(sims, ground, crowd_indirect),
            gi_probes: None,
            strips: StripState::new(
                strip_pipelines,
                ribbon_buffer,
                ribbon_sampler,
                ribbon_shader,
            ),
            fx_layout,
            fx_lean_layout,
            palette_layout,
            lines,
            particles,
            lights_2d,
            history: history::FrameHistory::new(),
            mips,
            transmission: None,
            conformed,
            canvas_stats: (None, [0.0; 2]),
            default_white,
            default_normal,
            overlay,
            ui,
            glass,
            glass_targets: None,
            cache: SceneCache::default(),
            frame_index: 0,
            phase_base: 0,
            saves: Default::default(),
            pipeline_disk,
            gi_history: None,
            ssr,
            stylise_chain: None,
            oit,
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

/// Whether the renderer runs in WebKit (`navigator.userAgent`, in a Worker
/// or a page): its indirect draws need buffers of their own
/// ([`RenderInfo::indirect_per_draw`]).
#[cfg(target_arch = "wasm32")]
fn webkit() -> bool {
    let get = |o: &wasm_bindgen::JsValue, k: &str| js_sys::Reflect::get(o, &k.into()).ok();
    get(&js_sys::global().into(), "navigator")
        .and_then(|n| get(&n, "userAgent"))
        .and_then(|u| u.as_string())
        .is_some_and(|u| is_webkit(&u))
}

#[cfg(not(target_arch = "wasm32"))]
fn webkit() -> bool {
    false
}

/// Whether a user agent string is WebKit's (Safari, and every browser on
/// iOS): `AppleWebKit/` without Blink's `Chrome/`, which Chromium's
/// desktop and Android agents carry beside it.
#[cfg(any(target_arch = "wasm32", test))]
fn is_webkit(user_agent: &str) -> bool {
    user_agent.contains("AppleWebKit/") && !user_agent.contains("Chrome/")
}

#[cfg(test)]
mod webkit_tests {
    #[test]
    fn webkit_is_safari_and_ios_not_blink_or_gecko() {
        let webkit = [
            // Safari 27.2 on macOS (the former test sample
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/27.2 Safari/605.1.15",
            // Chrome on iOS: WebKit underneath.
            "Mozilla/5.0 (iPhone; CPU iPhone OS 27_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/150.0.0.0 Mobile/15E148 Safari/604.1",
        ];
        let not = [
            // Chrome on the same Mac (adapter-chrome.json): AppleWebKit/537.36 beside Chrome/.
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) HeadlessChrome/150.0.0.0 Safari/537.36",
            "Mozilla/5.0 (Linux; Android 16; Pixel 10) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Mobile Safari/537.36",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 15.0; rv:150.0) Gecko/20100101 Firefox/150.0",
        ];
        for ua in webkit {
            assert!(super::is_webkit(ua), "{ua}");
        }
        for ua in not {
            assert!(!super::is_webkit(ua), "{ua}");
        }
    }
}
