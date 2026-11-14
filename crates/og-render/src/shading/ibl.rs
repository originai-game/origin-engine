//! Image-based lighting generated on the GPU with render passes: a procedural sky
//! cubemap with mips, its irradiance convolution, GGX-prefiltered specular mips and
//! the split-sum BRDF LUT.
//!
//! The generator is made once per renderer and keeps its pipelines, its
//! textures and their bind groups: a new sky (a day-night cycle moving the
//! sun) rewrites one uniform buffer and re-records the passes into the same
//! textures, and the BRDF LUT, which does not depend on the sky, is drawn
//! once. Every regeneration draws the same passes as the first.
//!
//! A cooked environment map ([`IblGenerator::generate_map`]) takes the sky's
//! place: its levels are copied, turned and scaled, into the same sky and
//! prefiltered cubes (the sky's lower mips downsampled as for the sky), and
//! its irradiance harmonics evaluated into the irradiance cube.

use crate::HDR_FORMAT;
use crate::atmosphere::SkyLook;
use crate::deferred::Deferred;
use glam::Vec3;
pub use og_render_data::ibl::*;

mod bake;
mod probe_filter;

pub(crate) use probe_filter::ProbeFilter;

pub const SKY_SIZE: u32 = 256;
const IRRADIANCE_SIZE: u32 = 32;
pub const PREFILTER_SIZE: u32 = 128;
const LUT_SIZE: u32 = 256;
/// Uniform slots: one per pass, a multiple of the 256-byte dynamic offset
/// alignment that holds [`Params`].
const SLOT: u64 = 1024;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Params {
    face: u32,
    roughness: f32,
    sample_count: u32,
    source_size: f32,
    sun_dir: [f32; 4],
    sun_color: [f32; 4],
    /// The look (`SkyLook::gpu`); zenith.w 0 is the built-in sky.
    look: [[f32; 4]; 5],
    /// An environment map's turn, scale and irradiance
    /// (`environment::env_params`); unused by the sky's passes.
    env: [[f32; 4]; 8],
    /// The physical sky's block (`PhysicalAtmosphere::gpu`; zero: the
    /// gradient or the built-in sky).
    scatter: [[f32; 4]; crate::sky_physical::SCATTER_VEC4S],
}

const _: () = assert!(std::mem::size_of::<Params>() as u64 <= SLOT);

/// The generator's module: its passes, the visibility and the sheen its
/// BRDF table integrates, and the physical sky's functions.
pub(crate) fn ibl_source() -> String {
    format!(
        "{}{}{IBL_WGSL}{SHEEN_WGSL}{}",
        crate::LIBRARY_WGSL,
        crate::GGX_WGSL,
        crate::sky_physical::sky_scatter_wgsl("params.scatter")
    )
}

/// The views the frame binds.
pub struct Ibl {
    /// The sky itself (full mip chain): the background the sky pass draws.
    pub sky: wgpu::TextureView,
    pub irradiance: wgpu::TextureView,
    pub prefiltered: wgpu::TextureView,
    pub brdf_lut: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

pub(crate) fn cube_texture(
    device: &wgpu::Device,
    label: &str,
    size: u32,
    mips: u32,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 6,
        },
        mip_level_count: mips,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: HDR_FORMAT,
        // Copies: the live sky's filtered cubes into the bound ones
        // (sky_capture.rs).
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

fn cube_view(tex: &wgpu::Texture, label: &str) -> wgpu::TextureView {
    tex.create_view(&wgpu::TextureViewDescriptor {
        label: Some(label),
        dimension: Some(wgpu::TextureViewDimension::Cube),
        ..Default::default()
    })
}

fn cube_view_mips(
    tex: &wgpu::Texture,
    label: &str,
    base_mip: u32,
    count: u32,
) -> wgpu::TextureView {
    tex.create_view(&wgpu::TextureViewDescriptor {
        label: Some(label),
        dimension: Some(wgpu::TextureViewDimension::Cube),
        base_mip_level: base_mip,
        mip_level_count: Some(count),
        ..Default::default()
    })
}

fn face_view(tex: &wgpu::Texture, face: u32, mip: u32) -> wgpu::TextureView {
    tex.create_view(&wgpu::TextureViewDescriptor {
        label: Some("ibl face"),
        dimension: Some(wgpu::TextureViewDimension::D2),
        base_mip_level: mip,
        mip_level_count: Some(1),
        base_array_layer: face,
        array_layer_count: Some(1),
        ..Default::default()
    })
}

/// What one pass draws: its target, pipeline, the source it samples and
/// its parameters (all but the sky's, which every pass shares).
struct Pass {
    target: wgpu::TextureView,
    pipeline: usize,
    group: wgpu::BindGroup,
    face: u32,
    roughness: f32,
    sample_count: u32,
}

/// The IBL generator: made once, regenerates for every new sky.
pub struct IblGenerator {
    /// Made on first use ([`IblGenerator::warm`]).
    pipelines: Vec<Deferred<wgpu::RenderPipeline>>,
    layout: wgpu::BindGroupLayout,
    /// The sky mips' level count (the downsample passes follow the sky's).
    sky_mips: u32,
    params: wgpu::Buffer,
    passes: Vec<Pass>,
    lut_pass: Pass,
    lut_drawn: bool,
    /// Kept alive for the bind groups and views.
    _textures: Vec<wgpu::Texture>,
    views: Ibl,
    /// Binding 4's placeholder for every pass but an equirect bake's.
    flat: wgpu::TextureView,
}

impl IblGenerator {
    /// Makes one pipeline not yet made; false when all are
    /// ([`crate::Renderer::warm`]).
    pub(crate) fn warm(&self) -> bool {
        self.pipelines.iter().any(crate::deferred::Warm::warm)
    }

    /// GPU bytes of its cubes.
    pub(crate) fn bytes(&self) -> u64 {
        self._textures
            .iter()
            .map(crate::gpu::texture::texture_bytes)
            .sum()
    }
}

impl IblGenerator {
    pub fn new(device: &wgpu::Device) -> IblGenerator {
        // Parsed when its first pipeline is made: each pipeline is made
        // on first use, so a frame compiles only the passes it runs.
        let shader = crate::deferred::module(device, "ibl", ibl_source());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ibl sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ibl layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<Params>() as u64
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // The map blended towards (`EnvironmentBlend`; the source
                // again elsewhere).
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
                // An uncooked environment's equirectangular image
                // (`bake_equirect`; the BRDF LUT as a placeholder elsewhere).
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ibl pl"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let make_pipeline = |entry: &'static str| {
            let (device, pipeline_layout, shader) =
                (device.clone(), pipeline_layout.clone(), shader.clone());
            Deferred::anywhere(move || {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(entry),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs"),
                        compilation_options: Default::default(),
                        buffers: &[],
                    },
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some(entry),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: HDR_FORMAT,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview_mask: None,
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            })
        };
        const SKY: usize = 0;
        const DOWN: usize = 1;
        const IRRADIANCE: usize = 2;
        const PREFILTER: usize = 3;
        const LUT: usize = 4;
        let pipelines = vec![
            make_pipeline("fs_sky"),
            make_pipeline("fs_downsample"),
            make_pipeline("fs_irradiance"),
            make_pipeline("fs_prefilter"),
            make_pipeline("fs_brdf_lut"),
            make_pipeline("fs_env_copy"),
            make_pipeline("fs_env_irradiance"),
            make_pipeline("fs_probe_copy"),
            make_pipeline("fs_equirect"),
        ];
        // The environment's first bake's (sky to the LUT) compile side by
        // side on the compile workers (deferred.rs) while the renderer
        // sets up; the rest are made when first used.
        #[cfg(not(target_arch = "wasm32"))]
        for p in &pipelines[SKY..=LUT] {
            crate::deferred::Start::start(p);
        }

        let sky_mips = SKY_SIZE.ilog2() + 1;
        let sky = cube_texture(device, "sky", SKY_SIZE, sky_mips);
        let irradiance = cube_texture(device, "irradiance", IRRADIANCE_SIZE, 1);
        let prefiltered = cube_texture(device, "prefiltered", PREFILTER_SIZE, PREFILTER_MIPS);
        let lut = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("brdf lut"),
            size: wgpu::Extent3d {
                width: LUT_SIZE,
                height: LUT_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: HDR_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let count = 6 * (sky_mips as usize + 1 + PREFILTER_MIPS as usize) + 1;
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ibl params"),
            size: SLOT * count as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // binding 4's placeholder: never a pass's target (the LUT's own
        // pass would read and write it at once).
        let placeholder = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ibl equirect placeholder"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: HDR_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let flat = placeholder.create_view(&Default::default());
        let group = |source: &wgpu::TextureView| -> wgpu::BindGroup {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ibl bg"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &params,
                            offset: 0,
                            size: wgpu::BufferSize::new(std::mem::size_of::<Params>() as u64),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(source),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(source),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&flat),
                    },
                ],
            })
        };
        let pass = |target, pipeline, group, face, roughness, sample_count| Pass {
            target,
            pipeline,
            group,
            face,
            roughness,
            sample_count,
        };
        // The sky pass samples nothing, but the layout needs a cube: the irradiance texture.
        let dummy = cube_view(&irradiance, "dummy");
        let mut passes = Vec::new();
        for face in 0..6 {
            passes.push(pass(
                face_view(&sky, face, 0),
                SKY,
                group(&dummy),
                face,
                0.0,
                0,
            ));
        }
        // Sky mips: each level samples the one above through a view restricted to it.
        for mip in 1..sky_mips {
            let above = cube_view_mips(&sky, "sky above", mip - 1, 1);
            for face in 0..6 {
                passes.push(pass(
                    face_view(&sky, face, mip),
                    DOWN,
                    group(&above),
                    face,
                    0.0,
                    0,
                ));
            }
        }
        let sky_full = cube_view(&sky, "sky");
        for face in 0..6 {
            passes.push(pass(
                face_view(&irradiance, face, 0),
                IRRADIANCE,
                group(&sky_full),
                face,
                0.0,
                0,
            ));
        }
        for mip in 0..PREFILTER_MIPS {
            let roughness = mip as f32 / (PREFILTER_MIPS - 1) as f32;
            for face in 0..6 {
                passes.push(pass(
                    face_view(&prefiltered, face, mip),
                    PREFILTER,
                    group(&sky_full),
                    face,
                    roughness,
                    128,
                ));
            }
        }
        let lut_pass = pass(
            lut.create_view(&Default::default()),
            LUT,
            group(&dummy),
            0,
            0.0,
            0,
        );
        let views = Ibl {
            sky: cube_view(&sky, "sky"),
            irradiance: cube_view(&irradiance, "irradiance"),
            prefiltered: cube_view(&prefiltered, "prefiltered"),
            brdf_lut: lut.create_view(&Default::default()),
            sampler,
        };
        IblGenerator {
            pipelines,
            layout,
            sky_mips,
            params,
            passes,
            lut_pass,
            lut_drawn: false,
            _textures: vec![sky, irradiance, prefiltered, lut, placeholder],
            views,
            flat,
        }
    }

    /// The views the frame binds (the same textures after every regeneration).
    pub fn views(&self) -> &Ibl {
        &self.views
    }

    /// The bound irradiance and prefiltered cubes themselves (the live
    /// sky copies its filtered cubes into them: sky_capture.rs).
    pub(crate) fn bound(&self) -> (&wgpu::Texture, &wgpu::Texture) {
        (&self._textures[1], &self._textures[2])
    }

    /// Draws the sky of `sun_dir`, `sun_color` and `look` (`None`: the
    /// built-in sky) and its convolutions into the textures.
    pub fn generate(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sun_dir: Vec3,
        sun_color: Vec3,
        look: Option<&SkyLook>,
    ) {
        let scatter = look
            .and_then(|l| l.physical.as_ref())
            .map_or([[0.0; 4]; crate::sky_physical::SCATTER_VEC4S], |p| p.gpu());
        let look = look.map_or([[0.0; 4]; 5], |l| l.gpu(true, sun_color));
        let base = Params {
            face: 0,
            roughness: 0.0,
            sample_count: 0,
            source_size: SKY_SIZE as f32,
            sun_dir: [sun_dir.x, sun_dir.y, sun_dir.z, 0.0],
            sun_color: [sun_color.x, sun_color.y, sun_color.z, 0.0],
            look,
            env: [[0.0; 4]; 8],
            scatter,
        };
        let lut = !self.lut_drawn;
        let all: Vec<&Pass> = self
            .passes
            .iter()
            .chain(lut.then_some(&self.lut_pass))
            .collect();
        let mut bytes = vec![0u8; SLOT as usize * all.len()];
        for (i, p) in all.iter().enumerate() {
            let params = Params {
                face: p.face,
                roughness: p.roughness,
                sample_count: p.sample_count,
                ..base
            };
            let at = i * SLOT as usize;
            bytes[at..at + std::mem::size_of::<Params>()]
                .copy_from_slice(bytemuck::bytes_of(&params));
        }
        queue.write_buffer(&self.params, 0, &bytes);
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("ibl") });
        for (i, p) in all.iter().enumerate() {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ibl pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &p.target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipelines[p.pipeline]);
            pass.set_bind_group(0, &p.group, &[(i as u64 * SLOT) as u32]);
            pass.draw(0..3, 0..1);
        }
        queue.submit(Some(encoder.finish()));
        self.lut_drawn = true;
    }

    /// Fills the textures from a cooked environment map instead of the sky:
    /// `map` (a cube view of its [`crate::ENVIRONMENT_MAP_LEVELS`] levels)
    /// turned by `rotation` radians about +y, level 0 into the sky cube
    /// scaled by `background` (its mips downsampled as for the sky), level
    /// `m` into prefiltered mip `m`, and `irradiance` (the map's harmonics)
    /// into the irradiance cube.
    /// `blend`: a second map and its weight (0..1), mixed into every
    /// level as the light and background blend (its harmonics already
    /// mixed into `irradiance` by the caller).
    #[allow(clippy::too_many_arguments)]
    pub fn generate_map(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        map: &wgpu::TextureView,
        blend: Option<(&wgpu::TextureView, f32)>,
        irradiance: &[[f32; 3]; 9],
        rotation: f32,
        background: f32,
    ) {
        let (to, weight) = blend.unwrap_or((map, 0.0));
        const DOWN: usize = 1;
        const ENV_COPY: usize = 5;
        const ENV_IRRADIANCE: usize = 6;
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("environment map"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.params,
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<Params>() as u64),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(map),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.views.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(to),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&self.flat),
                },
            ],
        });
        // The sky's passes in order: sky faces, sky mips, irradiance, prefilter.
        let down = 6 * (self.sky_mips as usize - 1);
        let (irr, pre) = (6 + down, 12 + down);
        // (pass whose target is drawn, pipeline, group, source level, scale)
        let mut draws: Vec<(usize, usize, &wgpu::BindGroup, f32, f32)> = Vec::new();
        for face in 0..6 {
            draws.push((face, ENV_COPY, &group, 0.0, background));
        }
        for (i, pass) in self.passes.iter().enumerate().take(irr).skip(6) {
            draws.push((i, DOWN, &pass.group, 0.0, 1.0));
        }
        for face in 0..6 {
            draws.push((irr + face, ENV_IRRADIANCE, &group, 0.0, 1.0));
        }
        for mip in 0..PREFILTER_MIPS as usize {
            for face in 0..6 {
                draws.push((pre + mip * 6 + face, ENV_COPY, &group, mip as f32, 1.0));
            }
        }
        let lut = !self.lut_drawn;
        let count = draws.len() + lut as usize;
        let mut bytes = vec![0u8; SLOT as usize * count];
        for (i, &(pass, pipeline, _, level, scale)) in draws.iter().enumerate() {
            let p = &self.passes[pass];
            let params = Params {
                face: p.face,
                // The downsample passes read the level above.
                roughness: if pipeline == DOWN { p.roughness } else { level },
                sample_count: 0,
                source_size: 0.0,
                sun_dir: [0.0; 4],
                sun_color: [0.0; 4],
                look: [[0.0; 4]; 5],
                env: crate::environment::env_params(rotation, scale, weight, irradiance),
                scatter: [[0.0; 4]; crate::sky_physical::SCATTER_VEC4S],
            };
            let at = i * SLOT as usize;
            bytes[at..at + std::mem::size_of::<Params>()]
                .copy_from_slice(bytemuck::bytes_of(&params));
        }
        queue.write_buffer(&self.params, 0, &bytes);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ibl from a map"),
        });
        let lut_draw = lut.then_some(&self.lut_pass);
        let all = draws
            .iter()
            .map(|&(pass, pipeline, group, ..)| (&self.passes[pass].target, pipeline, group))
            .chain(lut_draw.map(|p| (&p.target, p.pipeline, &p.group)));
        for (i, (target, pipeline, group)) in all.enumerate() {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ibl map pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipelines[pipeline]);
            pass.set_bind_group(0, group, &[(i as u64 * SLOT) as u32]);
            pass.draw(0..3, 0..1);
        }
        queue.submit(Some(encoder.finish()));
        self.lut_drawn = true;
    }
}

/// The IBL generator's module (`crate::shaders` validates it).
pub(crate) const IBL_WGSL: &str = wgsl!("ibl.wgsl");
#[cfg(test)]
mod tests;
