//! Image-based lighting generated on the GPU with render passes: a procedural sky
//! cubemap with mips, its irradiance convolution, GGX-prefiltered specular mips and
//! the split-sum BRDF LUT.

use glam::Vec3;

const SKY_SIZE: u32 = 256;
const IRRADIANCE_SIZE: u32 = 32;
pub const PREFILTER_SIZE: u32 = 128;
pub const PREFILTER_MIPS: u32 = 5;
const LUT_SIZE: u32 = 256;
pub const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Params {
    face: u32,
    roughness: f32,
    sample_count: u32,
    source_size: f32,
    sun_dir: [f32; 4],
    sun_color: [f32; 4],
}

pub struct Ibl {
    pub irradiance: wgpu::TextureView,
    pub prefiltered: wgpu::TextureView,
    pub brdf_lut: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

fn cube_texture(device: &wgpu::Device, label: &str, size: u32, mips: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d { width: size, height: size, depth_or_array_layers: 6 },
        mip_level_count: mips,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: HDR_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    })
}

fn cube_view(tex: &wgpu::Texture, label: &str) -> wgpu::TextureView {
    tex.create_view(&wgpu::TextureViewDescriptor { label: Some(label), dimension: Some(wgpu::TextureViewDimension::Cube), ..Default::default() })
}

fn cube_view_mips(tex: &wgpu::Texture, label: &str, base_mip: u32, count: u32) -> wgpu::TextureView {
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

impl Ibl {
    pub fn generate(device: &wgpu::Device, queue: &wgpu::Queue, sun_dir: Vec3, sun_color: Vec3) -> Ibl {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("ibl"), source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/ibl.wgsl").into()) });
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
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::Cube, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("ibl pl"), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
        let make_pipeline = |entry: &str| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(entry),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs"), compilation_options: Default::default(), buffers: &[] },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState { module: &shader, entry_point: Some(entry), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format: HDR_FORMAT, blend: None, write_mask: wgpu::ColorWrites::ALL })] }),
                multiview_mask: None,
                cache: None,
            })
        };
        let sky_pipeline = make_pipeline("fs_sky");
        let down_pipeline = make_pipeline("fs_downsample");
        let irradiance_pipeline = make_pipeline("fs_irradiance");
        let prefilter_pipeline = make_pipeline("fs_prefilter");
        let lut_pipeline = make_pipeline("fs_brdf_lut");

        let sky_mips = SKY_SIZE.ilog2() + 1;
        let sky = cube_texture(device, "sky", SKY_SIZE, sky_mips);
        let irradiance = cube_texture(device, "irradiance", IRRADIANCE_SIZE, 1);
        let prefiltered = cube_texture(device, "prefiltered", PREFILTER_SIZE, PREFILTER_MIPS);
        let lut = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("brdf lut"),
            size: wgpu::Extent3d { width: LUT_SIZE, height: LUT_SIZE, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: HDR_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        // Every pass has its own uniform buffer and bind group (small, one-time work).
        let mut passes: Vec<(wgpu::TextureView, &wgpu::RenderPipeline, wgpu::BindGroup)> = Vec::new();
        let bind = |params: Params, source: &wgpu::TextureView| -> wgpu::BindGroup {
            use wgpu::util::DeviceExt;
            let buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("ibl params"), contents: bytemuck::bytes_of(&params), usage: wgpu::BufferUsages::UNIFORM });
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ibl bg"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: buf.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(source) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&sampler) },
                ],
            })
        };
        let base = Params { face: 0, roughness: 0.0, sample_count: 0, source_size: SKY_SIZE as f32, sun_dir: [sun_dir.x, sun_dir.y, sun_dir.z, 0.0], sun_color: [sun_color.x, sun_color.y, sun_color.z, 0.0] };
        let lut_view = lut.create_view(&Default::default());
        // The sky pass samples nothing, but the layout needs a cube: use the irradiance texture.
        let dummy = cube_view(&irradiance, "dummy");
        for face in 0..6 {
            passes.push((face_view(&sky, face, 0), &sky_pipeline, bind(Params { face, ..base }, &dummy)));
        }
        // Sky mips: each level samples the one above through a view restricted to it.
        let mut sky_level_views = Vec::new();
        for mip in 1..sky_mips {
            let above = cube_view_mips(&sky, "sky above", mip - 1, 1);
            sky_level_views.push(above);
        }
        for mip in 1..sky_mips {
            let above = &sky_level_views[(mip - 1) as usize];
            for face in 0..6 {
                passes.push((face_view(&sky, face, mip), &down_pipeline, bind(Params { face, roughness: 0.0, ..base }, above)));
            }
        }
        let sky_full = cube_view(&sky, "sky");
        for face in 0..6 {
            passes.push((face_view(&irradiance, face, 0), &irradiance_pipeline, bind(Params { face, ..base }, &sky_full)));
        }
        for mip in 0..PREFILTER_MIPS {
            let roughness = mip as f32 / (PREFILTER_MIPS - 1) as f32;
            for face in 0..6 {
                passes.push((face_view(&prefiltered, face, mip), &prefilter_pipeline, bind(Params { face, roughness, sample_count: 128, ..base }, &sky_full)));
            }
        }
        passes.push((lut_view, &lut_pipeline, bind(base, &dummy)));

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("ibl") });
        for (view, pipeline, bg) in &passes {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ibl pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store } })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, bg, &[]);
            pass.draw(0..3, 0..1);
        }
        queue.submit(Some(encoder.finish()));

        Ibl { irradiance: cube_view(&irradiance, "irradiance"), prefiltered: cube_view(&prefiltered, "prefiltered"), brdf_lut: lut.create_view(&Default::default()), sampler }
    }
}
