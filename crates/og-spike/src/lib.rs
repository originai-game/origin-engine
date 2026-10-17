//! Origin Engine spike: the smallest renderer that answers the go/no-go questions
//! (docs/design.md §10): PBR with shadows and IBL on wgpu, the same code native
//! (headless, for executors) and on the web (Worker + OffscreenCanvas).

pub mod ibl;
pub mod scene;
#[cfg(target_arch = "wasm32")]
pub mod web;

use glam::{Mat4, Vec3};
use scene::{Instance, Model, Vertex};
use wgpu::util::DeviceExt;

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const SHADOW_SIZE: u32 = 2048;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    shadow_view_proj: [[f32; 4]; 4],
    view_pos: [f32; 4],
    light_dir: [f32; 4],
    light_color: [f32; 4],
    params: [f32; 4],
}

struct GpuMesh {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    index_count: u32,
    material: usize,
    /// Index into `Renderer::instances`.
    instances: usize,
}

/// One model's instances: the buffer drawn, the rest transforms, and the current ones.
struct GpuInstances {
    buffer: wgpu::Buffer,
    base: Vec<Instance>,
    current: Vec<Instance>,
}

struct GpuMaterial {
    bind_group: wgpu::BindGroup,
}

/// What to render: models with their instance transforms, plus the ground.
pub struct SceneDesc {
    pub models: Vec<(Model, Vec<Instance>)>,
    pub ground_size: f32,
    pub sun_dir: Vec3,
    pub sun_color: Vec3,
}

pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    width: u32,
    height: u32,
    output_format: wgpu::TextureFormat,
    camera_buffer: wgpu::Buffer,
    shadow_camera_buffer: wgpu::Buffer,
    frame_bind_group: wgpu::BindGroup,
    shadow_bind_group: wgpu::BindGroup,
    pbr_pipeline: wgpu::RenderPipeline,
    shadow_pipeline: wgpu::RenderPipeline,
    post_pipeline: wgpu::RenderPipeline,
    post_bind_group: wgpu::BindGroup,
    hdr_view: wgpu::TextureView,
    depth_view: wgpu::TextureView,
    shadow_view: wgpu::TextureView,
    shadow_color_view: wgpu::TextureView,
    meshes: Vec<GpuMesh>,
    instances: Vec<GpuInstances>,
    materials: Vec<GpuMaterial>,
    /// Rotate every instance each frame and re-upload (the moving-objects comparison).
    pub moving: bool,
    pub scene_center: Vec3,
    pub scene_radius: f32,
    sun_dir: Vec3,
    pub frame_index: u32,
    pub triangles: u64,
    /// 0 = lit, 1 = shadow factor, 2 = shadow-space coordinates, 3 = normals.
    pub debug_view: f32,
}

fn texture_from(device: &wgpu::Device, queue: &wgpu::Queue, img: Option<&scene::ImageData>, srgb: bool, fallback: [u8; 4]) -> wgpu::TextureView {
    let (w, h, data): (u32, u32, std::borrow::Cow<[u8]>) = match img {
        Some(i) => (i.width, i.height, std::borrow::Cow::Borrowed(&i.rgba)),
        None => (1, 1, std::borrow::Cow::Owned(fallback.to_vec())),
    };
    let format = if srgb { wgpu::TextureFormat::Rgba8UnormSrgb } else { wgpu::TextureFormat::Rgba8Unorm };
    let tex = device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        &data,
    );
    tex.create_view(&Default::default())
}

impl Renderer {
    pub async fn new(instance: &wgpu::Instance, surface: Option<&wgpu::Surface<'_>>, width: u32, height: u32, output_format: Option<wgpu::TextureFormat>, scene: &SceneDesc) -> Result<Renderer, String> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::HighPerformance, force_fallback_adapter: false, compatible_surface: surface, ..Default::default() })
            .await
            .map_err(|e| format!("no adapter: {e}"))?;
        let info = adapter.get_info();
        log::info!("adapter: {} ({:?}, {:?})", info.name, info.backend, info.device_type);
        let limits = adapter.limits();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor { label: Some("og-spike"), required_features: wgpu::Features::empty(), required_limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(limits), ..Default::default() })
            .await
            .map_err(|e| format!("no device: {e}"))?;
        let output_format = output_format.unwrap_or(wgpu::TextureFormat::Rgba8UnormSrgb);

        let ibl = ibl::Ibl::generate(&device, &queue, scene.sun_dir, scene.sun_color);

        // Bounds over every instance of every model (and the ground).
        let mut min = Vec3::new(-scene.ground_size / 2.0, 0.0, -scene.ground_size / 2.0);
        let mut max = Vec3::new(scene.ground_size / 2.0, 0.0, scene.ground_size / 2.0);
        let mut triangles = 0u64;
        for (model, instances) in &scene.models {
            let corners = [model.min, model.max];
            let prim_tris: u64 = model.primitives.iter().map(|p| p.indices.len() as u64 / 3).sum();
            triangles += prim_tris * instances.len() as u64;
            for inst in instances {
                let m = Mat4::from_cols_array_2d(&inst.model);
                for i in 0..8 {
                    let c = Vec3::new(corners[i & 1].x, corners[(i >> 1) & 1].y, corners[(i >> 2) & 1].z);
                    let w = m.transform_point3(c);
                    min = min.min(w);
                    max = max.max(w);
                }
            }
        }
        let scene_center = (min + max) / 2.0;
        let scene_radius = (max - min).length() / 2.0;

        // Materials: glTF materials of every model, then the ground's checker.
        let material_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("material"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                tex_entry(1),
                tex_entry(2),
                tex_entry(3),
                tex_entry(4),
                wgpu::BindGroupLayoutEntry { binding: 5, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
            ],
        });
        let material_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("material sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let mut materials = Vec::new();
        let mut material_base = Vec::new();
        let make_material = |m: &scene::Material| {
            let factors = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("factors"), contents: bytemuck::bytes_of(&m.factors), usage: wgpu::BufferUsages::UNIFORM });
            let base = texture_from(&device, &queue, m.base_color.as_ref(), true, [255, 255, 255, 255]);
            let mr = texture_from(&device, &queue, m.metallic_roughness.as_ref(), false, [255, 255, 255, 255]);
            let normal = texture_from(&device, &queue, m.normal.as_ref(), false, [128, 128, 255, 255]);
            let emissive = texture_from(&device, &queue, m.emissive.as_ref(), true, [255, 255, 255, 255]);
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("material bg"),
                layout: &material_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: factors.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&base) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&mr) },
                    wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&normal) },
                    wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&emissive) },
                    wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::Sampler(&material_sampler) },
                ],
            });
            GpuMaterial { bind_group }
        };
        for (model, _) in &scene.models {
            material_base.push(materials.len());
            for m in &model.materials {
                materials.push(make_material(m));
            }
        }
        let ground_material = materials.len();
        let mut ground_mat = scene::Material::plain([1.0, 1.0, 1.0, 1.0], 0.0, 0.85);
        ground_mat.base_color = Some(scene::checker(512, 16));
        materials.push(make_material(&ground_mat));

        // Meshes and per-model instance buffers.
        let mut meshes = Vec::new();
        let mut instance_sets: Vec<GpuInstances> = Vec::new();
        let upload = |prim: &scene::Primitive, material: usize, instances: usize| -> GpuMesh {
            // Bake the node transform into the vertices so instances stay a single matrix.
            let m = prim.transform;
            let verts: Vec<Vertex> = prim
                .vertices
                .iter()
                .map(|v| Vertex { position: m.transform_point3(Vec3::from(v.position)).to_array(), normal: m.transform_vector3(Vec3::from(v.normal)).normalize_or_zero().to_array(), ..*v })
                .collect();
            GpuMesh {
                vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("vb"), contents: bytemuck::cast_slice(&verts), usage: wgpu::BufferUsages::VERTEX }),
                indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("ib"), contents: bytemuck::cast_slice(&prim.indices), usage: wgpu::BufferUsages::INDEX }),
                index_count: prim.indices.len() as u32,
                material,
                instances,
            }
        };
        let make_instances = |list: &[Instance]| GpuInstances {
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("instances"), contents: bytemuck::cast_slice(list), usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST }),
            base: list.to_vec(),
            current: list.to_vec(),
        };
        for (mi, (model, instances)) in scene.models.iter().enumerate() {
            instance_sets.push(make_instances(instances));
            for prim in &model.primitives {
                meshes.push(upload(prim, material_base[mi] + prim.material, mi));
            }
        }
        let ground = scene::ground(scene.ground_size, scene.ground_size / 2.0);
        instance_sets.push(make_instances(&[Instance { model: Mat4::IDENTITY.to_cols_array_2d() }]));
        meshes.push(upload(&ground, ground_material, instance_sets.len() - 1));
        triangles += 2;

        // Targets.
        let hdr = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("hdr"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: ibl::HDR_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let shadow = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow"),
            size: wgpu::Extent3d { width: SHADOW_SIZE, height: SHADOW_SIZE, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let shadow_color = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow depth as colour"),
            size: wgpu::Extent3d { width: SHADOW_SIZE, height: SHADOW_SIZE, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let hdr_view = hdr.create_view(&Default::default());
        let depth_view = depth.create_view(&Default::default());
        let shadow_view = shadow.create_view(&Default::default());
        let shadow_color_view = shadow_color.create_view(&Default::default());
        // Depth is read with explicit compares, so the sampler is a plain non-filtering one
        // (WebGPU forbids a filtering sampler on a depth texture).
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Frame bind group: camera, shadow map, IBL.
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("camera"), size: std::mem::size_of::<CameraUniform>() as u64, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let shadow_camera_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("shadow camera"), size: 64, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let frame_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("frame"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::VERTEX_FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: false }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering), count: None },
                wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::Cube, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 4, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::Cube, multisampled: false }, count: None },
                tex_entry(5),
                wgpu::BindGroupLayoutEntry { binding: 6, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
            ],
        });
        let frame_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("frame bg"),
            layout: &frame_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: camera_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&shadow_color_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&shadow_sampler) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&ibl.irradiance) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&ibl.prefiltered) },
                wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::TextureView(&ibl.brdf_lut) },
                wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::Sampler(&ibl.sampler) },
            ],
        });
        let shadow_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow layout"),
            entries: &[wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::VERTEX, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None }],
        });
        let shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("shadow bg"), layout: &shadow_layout, entries: &[wgpu::BindGroupEntry { binding: 0, resource: shadow_camera_buffer.as_entire_binding() }] });

        // Pipelines.
        let pbr_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("pbr"), source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/pbr.wgsl").into()) });
        let shadow_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("shadow"), source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/shadow.wgsl").into()) });
        let post_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("post"), source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/post.wgsl").into()) });
        let pbr_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("pbr pl"), bind_group_layouts: &[Some(&frame_layout), Some(&material_layout)], immediate_size: 0 });
        let pbr_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pbr"),
            layout: Some(&pbr_layout),
            vertex: wgpu::VertexState { module: &pbr_shader, entry_point: Some("vs"), compilation_options: Default::default(), buffers: &[Some(Vertex::LAYOUT), Some(Instance::LAYOUT)] },
            primitive: wgpu::PrimitiveState { cull_mode: Some(wgpu::Face::Back), ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState { format: DEPTH_FORMAT, depth_write_enabled: Some(true), depth_compare: Some(wgpu::CompareFunction::Less), stencil: Default::default(), bias: Default::default() }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState { module: &pbr_shader, entry_point: Some("fs"), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format: ibl::HDR_FORMAT, blend: None, write_mask: wgpu::ColorWrites::ALL })] }),
            multiview_mask: None,
            cache: None,
        });
        let shadow_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("shadow pl"), bind_group_layouts: &[Some(&shadow_layout)], immediate_size: 0 });
        let shadow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shadow"),
            layout: Some(&shadow_pl),
            vertex: wgpu::VertexState { module: &shadow_shader, entry_point: Some("vs"), compilation_options: Default::default(), buffers: &[Some(Vertex::LAYOUT), Some(Instance::LAYOUT)] },
            primitive: wgpu::PrimitiveState { cull_mode: Some(wgpu::Face::Front), ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState { format: DEPTH_FORMAT, depth_write_enabled: Some(true), depth_compare: Some(wgpu::CompareFunction::Less), stencil: Default::default(), bias: wgpu::DepthBiasState { constant: 2, slope_scale: 2.0, clamp: 0.0 } }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState { module: &shadow_shader, entry_point: Some("fs"), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format: wgpu::TextureFormat::R32Float, blend: None, write_mask: wgpu::ColorWrites::RED })] }),
            multiview_mask: None,
            cache: None,
        });
        let post_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("post layout"),
            entries: &[tex_entry(0), wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None }],
        });
        let post_sampler = device.create_sampler(&wgpu::SamplerDescriptor { label: Some("post sampler"), mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, ..Default::default() });
        let post_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("post bg"),
            layout: &post_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&hdr_view) }, wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&post_sampler) }],
        });
        let post_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("post pl"), bind_group_layouts: &[Some(&post_layout)], immediate_size: 0 });
        let post_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("post"),
            layout: Some(&post_pl),
            vertex: wgpu::VertexState { module: &post_shader, entry_point: Some("vs"), compilation_options: Default::default(), buffers: &[] },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState { module: &post_shader, entry_point: Some("fs"), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format: output_format, blend: None, write_mask: wgpu::ColorWrites::ALL })] }),
            multiview_mask: None,
            cache: None,
        });

        Ok(Renderer {
            device,
            queue,
            width,
            height,
            output_format,
            camera_buffer,
            shadow_camera_buffer,
            frame_bind_group,
            shadow_bind_group,
            pbr_pipeline,
            shadow_pipeline,
            post_pipeline,
            post_bind_group,
            hdr_view,
            depth_view,
            shadow_view,
            shadow_color_view,
            meshes,
            instances: instance_sets,
            materials,
            moving: false,
            scene_center,
            scene_radius,
            sun_dir: scene.sun_dir.normalize(),
            frame_index: 0,
            triangles,
            debug_view: 0.0,
        })
    }

    pub fn output_format(&self) -> wgpu::TextureFormat {
        self.output_format
    }

    /// Camera orbiting the scene; `time` in seconds drives the orbit so frames differ.
    fn update_camera(&mut self, time: f32) {
        let angle = 0.6 + time * 0.25;
        let dist = self.scene_radius * 1.05;
        let eye = self.scene_center + Vec3::new(angle.cos() * dist, self.scene_radius * 0.5, angle.sin() * dist);
        let view = glam::camera::rh::view::look_at_mat4(eye, self.scene_center, Vec3::Y);
        let proj = glam::camera::rh::proj::directx::perspective(50f32.to_radians(), self.width as f32 / self.height as f32, 0.1, self.scene_radius * 8.0);
        // Shadow: orthographic box around the scene from the sun.
        let r = self.scene_radius;
        let light_eye = self.scene_center + self.sun_dir * r * 2.0;
        let light_view = glam::camera::rh::view::look_at_mat4(light_eye, self.scene_center, Vec3::Y);
        let light_proj = glam::camera::rh::proj::directx::orthographic(-r, r, -r, r, 0.1, r * 4.0);
        let shadow_vp = light_proj * light_view;
        let uniform = CameraUniform {
            view_proj: (proj * view).to_cols_array_2d(),
            shadow_view_proj: shadow_vp.to_cols_array_2d(),
            view_pos: scene::vec4(eye, 1.0),
            light_dir: scene::vec4(self.sun_dir, 0.0),
            light_color: [4.0, 3.8, 3.4, 1.0 / SHADOW_SIZE as f32],
            params: [0.55, (ibl::PREFILTER_MIPS - 1) as f32, time, self.debug_view],
        };
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&uniform));
        self.queue.write_buffer(&self.shadow_camera_buffer, 0, bytemuck::bytes_of(&shadow_vp.to_cols_array_2d()));
    }

    /// Rotates every model instance about Y by a per-instance rate and re-uploads the buffer.
    fn update_instances(&mut self, time: f32) {
        let n = self.instances.len();
        for (k, set) in self.instances.iter_mut().enumerate() {
            if k + 1 == n {
                continue; // the ground
            }
            for (i, (base, cur)) in set.base.iter().zip(set.current.iter_mut()).enumerate() {
                let rate = 0.5 + (i % 7) as f32 * 0.2;
                let m = Mat4::from_cols_array_2d(&base.model) * Mat4::from_rotation_y(time * rate);
                cur.model = m.to_cols_array_2d();
            }
            self.queue.write_buffer(&set.buffer, 0, bytemuck::cast_slice(&set.current));
        }
    }

    /// Records one frame into `target` (the swapchain view or the readback texture).
    pub fn render(&mut self, target: &wgpu::TextureView, time: f32) {
        self.update_camera(time);
        if self.moving {
            self.update_instances(time);
        }
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadow"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &self.shadow_color_view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::WHITE), store: wgpu::StoreOp::Store } })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { view: &self.shadow_view, depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }), stencil_ops: None }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.shadow_pipeline);
            pass.set_bind_group(0, &self.shadow_bind_group, &[]);
            for mesh in &self.meshes {
                let set = &self.instances[mesh.instances];
                pass.set_vertex_buffer(0, mesh.vertices.slice(..));
                pass.set_vertex_buffer(1, set.buffer.slice(..));
                pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..mesh.index_count, 0, 0..set.base.len() as u32);
            }
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("pbr"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &self.hdr_view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.5, g: 0.65, b: 0.9, a: 1.0 }), store: wgpu::StoreOp::Store } })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { view: &self.depth_view, depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Discard }), stencil_ops: None }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pbr_pipeline);
            pass.set_bind_group(0, &self.frame_bind_group, &[]);
            for mesh in &self.meshes {
                pass.set_bind_group(1, &self.materials[mesh.material].bind_group, &[]);
                let set = &self.instances[mesh.instances];
                pass.set_vertex_buffer(0, mesh.vertices.slice(..));
                pass.set_vertex_buffer(1, set.buffer.slice(..));
                pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..mesh.index_count, 0, 0..set.base.len() as u32);
            }
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("post"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: target, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store } })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.post_pipeline);
            pass.set_bind_group(0, &self.post_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));
        self.frame_index += 1;
    }

    pub fn draw_calls(&self) -> usize {
        self.meshes.len() * 2 + 1
    }
}

fn tex_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry { binding, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None }
}

/// The spike's reference scene: `count` instances of each model on a grid over a ground plane.
/// `spacing` is the grid step (3.0 for the glTF models, 1.2 for the sphere field).
pub fn reference_scene(models: Vec<Model>, count_per_model: u32, spacing: f32) -> SceneDesc {
    let mut placed = Vec::new();
    let n = models.len().max(1) as f32;
    for (k, model) in models.into_iter().enumerate() {
        let extent = model.max - model.min;
        let scale = 2.0 / extent.max_element().max(1e-3);
        let mut instances = scene::grid_instances(count_per_model, spacing, scale, model.min);
        // Offset each model's grid so they interleave.
        let offset = Vec3::new(0.0, 0.0, (k as f32 - (n - 1.0) / 2.0) * spacing * 0.5);
        for inst in &mut instances {
            let m = Mat4::from_translation(offset) * Mat4::from_cols_array_2d(&inst.model);
            inst.model = m.to_cols_array_2d();
        }
        placed.push((model, instances));
    }
    let side = (count_per_model as f32).sqrt().ceil() * spacing + 6.0;
    SceneDesc { models: placed, ground_size: side.max(12.0), sun_dir: Vec3::new(0.45, 0.8, 0.35), sun_color: Vec3::new(1.0, 0.95, 0.85) }
}
