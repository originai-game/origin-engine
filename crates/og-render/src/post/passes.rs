//! Post pipeline construction, uniforms and exposure state.

use super::*;

mod exposure;
mod uniforms;

impl PostPasses {
    /// Starts compiling the passes every frame of a camera draws, the
    /// tonemap and TAA with its sharpening, on worker threads before the
    /// scene's passes are encoded (deferred.rs).
    pub(crate) fn start(&self) {
        use crate::deferred::Start;
        self.tonemap.start();
        for p in [&self.taa, &self.taa_sharpen_pipeline]
            .into_iter()
            .flatten()
        {
            p.start();
        }
    }

    /// Makes one of its pipelines no frame used yet: the tonemap, the
    /// exposure meter and bloom first, then TAA and the effect passes
    /// (Renderer::warm).
    pub(crate) fn warm(&self) -> bool {
        let mut items: Vec<&dyn crate::deferred::Warm> = vec![
            &self.tonemap,
            &self.meter,
            &self.reduce,
            &self.adapt,
            &self.prefilter,
            &self.down,
            &self.up,
        ];
        items.extend(self.taa.iter().map(|p| p as &dyn crate::deferred::Warm));
        items.extend(
            self.taa_sharpen_pipeline
                .iter()
                .map(|p| p as &dyn crate::deferred::Warm),
        );
        for entry in FX_PASSES.iter().map(|(e, _)| e).chain([&AO_BLUR_GATHER]) {
            if let Some(p) = self.fx.get(entry) {
                items.push(p);
            }
        }
        items.extend([
            &self.flare.occlusion as &dyn crate::deferred::Warm,
            &self.flare.flare,
            &self.flare.blur_x,
            &self.flare.blur_y,
        ]);
        crate::deferred::warm_one(&items)
    }

    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output_format: wgpu::TextureFormat,
        antialias: Antialias,
        aa: AaOptions,
        hdr: [f32; 4],
    ) -> PostPasses {
        let sampler_ty = wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering);
        let source_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("post source"),
            entries: &[
                layout_entry(0, float_texture(true)),
                layout_entry(1, sampler_ty),
            ],
        });
        let post_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("post"),
            entries: &[
                layout_entry(0, float_texture(true)),
                layout_entry(1, sampler_ty),
                layout_entry(2, UNIFORM),
                layout_entry(3, float_texture(false)),
                layout_entry(4, float_texture(true)),
                layout_entry(
                    5,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                ),
                layout_entry(6, float_texture(true)),
                // The lens flare (`flare.rs`), black without one.
                layout_entry(7, float_texture(true)),
            ],
        });
        let fx_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fx post"),
            entries: &[
                layout_entry(0, UNIFORM),
                layout_entry(1, sampler_ty),
                layout_entry(2, float_texture(true)),
                layout_entry(3, float_texture(false)),
                layout_entry(4, float_texture(false)),
                layout_entry(5, float_texture(true)),
                // Nearest, for the gathers of the unfilterable
                // view depth (fxpost_gather.wgsl).
                layout_entry(
                    6,
                    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                ),
                // The light volumes' buffer with the probe GI's probes
                // (the frame group's binding 10): the screen GI takes
                // from them what its rays replace (ssgi.wgsl).
                layout_entry(
                    7,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                ),
            ],
        });
        let taa_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("taa"),
            entries: &[
                // The lit image, sampled bilinearly as TAAU's first frame.
                layout_entry(0, float_texture(true)),
                layout_entry(1, float_texture(true)),
                // The depth buffer as an unfilterable float texture: every
                // backend reads the same depth this way.
                layout_entry(2, float_texture(false)),
                layout_entry(3, sampler_ty),
                layout_entry(4, UNIFORM),
                // The depth's own nearest sampler, read at the pixel's
                // centre (taa.wgsl, `depth_sampler`).
                layout_entry(
                    5,
                    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                ),
                // The prepass's gbuffer (velocity) and view depth
                // (placeholders without a prepass).
                layout_entry(6, float_texture(false)),
                layout_entry(7, float_texture(false)),
                // The OIT revealage, TAA's reactive mask (oit.rs; white,
                // nothing transparent, without it).
                layout_entry(8, float_texture(false)),
            ],
        });
        let module = |label: &'static str, body: &str| {
            crate::deferred::module(device, label, fullscreen_source(body))
        };
        let post_shader = module("post", POST_WGSL);
        let luminance_shader = module("luminance", LUMINANCE_WGSL);
        let pipeline_layout = |label: &str, layout: &wgpu::BindGroupLayout| {
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &[Some(layout)],
                immediate_size: 0,
            })
        };
        let source_pl = pipeline_layout("post source", &source_layout);
        let post_pl = pipeline_layout("post", &post_layout);
        // The stylisation passes (stylise.wgsl): the source, its sampler,
        // the parameters, the palette, the canvas and its repeating sampler,
        // the blue-noise tile, the prepass's view depth and normals (ink on
        // depth edges; placeholders without a prepass) and the object
        // pass's positions (object space; a zero texel without one) and
        // the outline target's object ids (the paint pass's object edges;
        // black without a prepass), then the conformed colours' hash table
        // (stylise::conformed_table) and the scene before the stylise
        // passes (the quantize pass's conformed test; the source again in
        // the others).
        let stylise_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("stylise"),
            entries: &[
                layout_entry(0, float_texture(true)),
                layout_entry(1, sampler_ty),
                layout_entry(2, UNIFORM),
                layout_entry(3, float_texture(true)),
                layout_entry(4, float_texture(true)),
                layout_entry(5, sampler_ty),
                layout_entry(6, float_texture(true)),
                layout_entry(7, float_texture(false)),
                layout_entry(8, float_texture(false)),
                layout_entry(
                    9,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ),
                layout_entry(10, float_texture(false)),
                layout_entry(
                    11,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ),
                layout_entry(12, float_texture(false)),
            ],
        });
        let stylise_pl = pipeline_layout("stylise", &stylise_layout);
        let meter = deferred_fullscreen(
            device,
            "meter",
            &luminance_shader,
            "fs_meter",
            &source_pl,
            LUMINANCE_FORMAT,
            None,
        );
        let reduce = deferred_fullscreen(
            device,
            "meter reduce",
            &luminance_shader,
            "fs_reduce",
            &source_pl,
            LUMINANCE_FORMAT,
            None,
        );
        let adapt_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("exposure adapt"),
            entries: &[
                layout_entry(0, float_texture(false)),
                layout_entry(1, float_texture(false)),
                layout_entry(2, UNIFORM),
            ],
        });
        let adapt = deferred_fullscreen(
            device,
            "exposure adapt",
            &module("exposure", EXPOSURE_WGSL),
            "fs_adapt",
            &pipeline_layout("exposure adapt", &adapt_layout),
            EXPOSURE_FORMAT,
            None,
        );
        let exposure_texture = |label: &str, width: u32| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: EXPOSURE_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            })
        };
        let textures = [
            exposure_texture("adapted exposure", 1),
            exposure_texture("adapted exposure", 1),
        ];
        let exposure = ExposureState {
            views: [
                textures[0].create_view(&Default::default()),
                textures[1].create_view(&Default::default()),
            ],
            textures,
            saves: exposure_texture("adapted exposure saves", EXPOSURE_SAVES),
            ring: vec![None; EXPOSURE_SAVES as usize],
            next_id: 1,
            last: None,
        };
        let prefilter = deferred_fullscreen(
            device,
            "bloom prefilter",
            &post_shader,
            "fs_bloom_prefilter",
            &post_pl,
            HDR_FORMAT,
            None,
        );
        let down = deferred_fullscreen(
            device,
            "bloom down",
            &post_shader,
            "fs_bloom_down",
            &post_pl,
            HDR_FORMAT,
            None,
        );
        let additive = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::REPLACE,
        };
        let up = deferred_fullscreen(
            device,
            "bloom up",
            &post_shader,
            "fs_bloom_up",
            &post_pl,
            HDR_FORMAT,
            Some(additive),
        );
        // A non-sRGB output (web canvases) gets the sRGB curve in the shader.
        let tonemap_entry = if output_format.is_srgb() || output_format == HDR_FORMAT {
            "fs"
        } else {
            "fs_srgb_encode"
        };
        let tonemap = deferred_fullscreen(
            device,
            "tonemap",
            &post_shader,
            tonemap_entry,
            &post_pl,
            output_format,
            None,
        );
        let taa_on = antialias == Antialias::Taa;
        let taa_shader = taa_on.then(|| module("taa", TAA_WGSL));
        let taa_pl = pipeline_layout("taa", &taa_layout);
        let taa = taa_shader.as_ref().map(|taa_shader| {
            deferred_fullscreen(
                device,
                "taa",
                taa_shader,
                match aa.taa_history {
                    TaaHistory::Bilinear => "fs",
                    TaaHistory::CatmullRom => "fs_catmull_rom",
                },
                &taa_pl,
                HDR_FORMAT,
                None,
            )
        });
        let taa_sharpen = if taa_on {
            aa.taa_sharpen.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let taa_sharpen_pipeline =
            taa_shader
                .as_ref()
                .filter(|_| taa_sharpen > 0.0)
                .map(|taa_shader| {
                    deferred_fullscreen(
                        device,
                        "taa sharpen",
                        taa_shader,
                        "fs_sharpen",
                        &taa_pl,
                        HDR_FORMAT,
                        None,
                    )
                });
        let repeat_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("stylise canvas"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("post"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let nearest_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("post nearest"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let black_texture = target_texture(
            device,
            "black",
            1,
            1,
            HDR_FORMAT,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        queue.write_texture(
            black_texture.as_image_copy(),
            &[0u8; 8],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(8),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        let uniform = |label: &str, size: usize| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: size as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let fx_shader = module("fx post", &fx_source());
        let fx_pl = pipeline_layout("fx post", &fx_layout);
        let mut fx: HashMap<&'static str, Deferred<wgpu::RenderPipeline>> = FX_PASSES
            .iter()
            .map(|&(entry, format)| {
                (
                    entry,
                    deferred_fullscreen(device, entry, &fx_shader, entry, &fx_pl, format, None),
                )
            })
            .collect();
        fx.insert(
            AO_BLUR_GATHER,
            deferred_fullscreen(
                device,
                AO_BLUR_GATHER,
                &fx_shader,
                AO_BLUR_GATHER,
                &fx_pl,
                AO_FORMAT,
                None,
            ),
        );
        let identity_lut = lut_texture(device, queue, &crate::ColorLut::identity());
        let far_depth = one_texel(
            device,
            queue,
            LINDEPTH_FORMAT,
            bytemuck::bytes_of(&[FAR_DEPTH, 0.0]),
        );
        let white = one_texel(device, queue, AO_FORMAT, &[255]);
        let no_reactive = one_texel(device, queue, wgpu::TextureFormat::Rg8Unorm, &[255, 255]);
        let no_object = one_texel(device, queue, crate::post::OBJECT_FORMAT, &[0; 16]);
        let blue_noise = texture_with_data(
            device,
            queue,
            crate::blue_noise::BLUE_NOISE_SIZE,
            wgpu::TextureFormat::R8Unorm,
            crate::blue_noise::BLUE_NOISE,
        );
        let chain_format = if hdr[0] > 0.0 { HDR_FORMAT } else { LDR_FORMAT };
        let smaa = matches!(antialias, Antialias::Smaa | Antialias::SmaaT2x).then(|| {
            smaa::Smaa::new(
                device,
                queue,
                &source_pl,
                &taa_layout,
                antialias == Antialias::SmaaT2x,
                chain_format,
            )
        });
        PostPasses {
            blue_noise,
            identity_lut,
            adjusted_lut: None,
            far_depth,
            white,
            no_reactive,
            no_object,
            fx_layout,
            fx_buffer: uniform("fx post", std::mem::size_of::<FxPostUniform>()),
            fx_after_buffer: uniform("fx post after taa", std::mem::size_of::<FxPostUniform>()),
            fx,
            flare: FlarePasses::new(device),
            fsr: fsr::FsrPasses::new(device),
            contact: contact::ContactPasses::new(device),
            source_layout,
            post_layout,
            taa_layout,
            meter,
            reduce,
            adapt_layout,
            adapt,
            adapt_buffer: uniform("exposure adapt", std::mem::size_of::<AdaptUniform>()),
            exposure,
            prefilter,
            down,
            up,
            tonemap,
            taa,
            taa_sharpen_pipeline,
            taa_sharpen,
            tonemap_ldr: None,
            chain: HashMap::new(),
            stylise_layout,
            stylise_pl,
            smaa,
            source_pl,
            post_pl,
            post_shader,
            stylise_shader: None,
            fxaa_shader: None,
            output_format,
            hdr,
            chain_format,
            stylise_buffer: uniform("stylise", std::mem::size_of::<StyliseUniform>()),
            repeat_sampler,
            sampler,
            nearest_sampler,
            black: black_texture.create_view(&Default::default()),
            post_buffer: uniform("post", std::mem::size_of::<PostUniform>()),
            taa_buffer: uniform("taa", std::mem::size_of::<TaaUniform>()),
        }
    }

    /// Makes the pipelines of the passes after the tonemap that `passes`
    /// holds and this renderer lacks ([`chain_passes`]): the tonemap into
    /// the LDR target, and each pass into the next target and into the
    /// output (sRGB-encoded in the shader where the output is not sRGB).
    pub fn ensure_chain(&mut self, device: &wgpu::Device, passes: &[FramePass]) {
        let chain = chain_passes(passes);
        if chain.is_empty() {
            return;
        }
        if self.tonemap_ldr.is_none() {
            self.tonemap_ldr = Some(fullscreen_pipeline(
                device,
                "tonemap (ldr)",
                &self.post_shader,
                "fs",
                &self.post_pl,
                self.chain_format,
                None,
            ));
        }
        let srgb = self.output_format.is_srgb() || self.output_format == HDR_FORMAT;
        for pass in chain {
            if self.chain.contains_key(&pass) {
                continue;
            }
            let entry: &str = match pass {
                FramePass::Fxaa | FramePass::SmaaBlend => "fs",
                FramePass::SmaaResolve => "fs_smaa",
                FramePass::Paint => "fs_paint",
                FramePass::Watercolor => "fs_watercolor",
                FramePass::Lines => "fs_lines",
                FramePass::Lens => "fs_lens",
                FramePass::Quantize => "fs_quantize",
                FramePass::Upscale => "fs_upscale",
                _ => unreachable!("not a pass after the tonemap"),
            };
            let encoded = if matches!(pass, FramePass::Fxaa | FramePass::SmaaBlend) {
                "fs_srgb_encode".to_string()
            } else {
                format!("{entry}_srgb")
            };
            let output_entry = if srgb { entry } else { encoded.as_str() };
            let (shader, layout) = if pass == FramePass::Fxaa {
                let shader = self.fxaa_shader.get_or_insert_with(|| {
                    device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some("fxaa"),
                        source: wgpu::ShaderSource::Wgsl(fullscreen_source(FXAA_WGSL).into()),
                    })
                });
                (&*shader, &self.source_pl)
            } else if pass == FramePass::SmaaBlend {
                let smaa = self.smaa.as_ref().expect("SMAA renderer");
                (&smaa.blend_shader, &smaa.blend_pl)
            } else if pass == FramePass::SmaaResolve {
                let temporal = self
                    .smaa
                    .as_ref()
                    .and_then(|s| s.temporal.as_ref())
                    .expect("T2x renderer");
                (&temporal.shader, &temporal.layout)
            } else {
                let shader = self.stylise_shader.get_or_insert_with(|| {
                    crate::unbounded_shader(device, "stylise", fullscreen_source(STYLISE_WGSL))
                });
                (&*shader, &self.stylise_pl)
            };
            let label = pass.name();
            let to_ldr = fullscreen_pipeline(
                device,
                &format!("{label} (ldr)"),
                shader,
                entry,
                layout,
                self.chain_format,
                None,
            );
            let to_output = fullscreen_pipeline(
                device,
                &label,
                shader,
                output_entry,
                layout,
                self.output_format,
                None,
            );
            self.chain.insert(pass, (to_ldr, to_output));
        }
    }
}
