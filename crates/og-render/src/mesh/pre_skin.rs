//! Single-pass compute pre-skinning (`pre_skin.wgsl`): deforms dynamic skinned
//! characters (4-bone LBS + blendshape morphs) once per frame into reserved
//! slots of the joint palette texture before shadow, prepass, main, reflection,
//! probe, and secondary-view passes execute.

use crate::Deferred;
use glam::Mat4;

/// Compute shader source for pre-skinning.
pub(crate) fn source() -> String {
    format!("{}{}", wgsl!("common.wgsl"), wgsl!("pre_skin.wgsl"))
}

/// One unique pre-skin dispatch for a dynamic character pose this frame.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PreSkinJob {
    pub mesh: usize,
    pub vertex_count: u32,
    pub palette_first: u32,
    pub palette_count: u32,
    pub morph_first: u32,
    pub morph_targets: u32,
    pub weights_slot: u32,
    pub out_slot: u32,
    pub transform: Mat4,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PreSkinParams {
    head: [u32; 4],
    morph: [u32; 4],
    model: [[f32; 4]; 4],
}

const PARAM_STRIDE: u64 = 256;

type PreSkinGroupKey = (
    wgpu::Buffer,
    Option<(u64, u64)>,
    wgpu::Buffer,
    wgpu::TextureView,
    wgpu::TextureView,
);

/// Persistent compute pipeline and growable upload buffers for pre-skinning.
pub(crate) struct PreSkinPipelines {
    layout: wgpu::BindGroupLayout,
    pipeline: Deferred<wgpu::ComputePipeline>,
    joints: wgpu::Buffer,
    params: wgpu::Buffer,
    /// Per-dispatch bind group cache keyed by `(mesh_verts, pooled_range, skin_buf, morphs, palette_out)`.
    groups: std::cell::RefCell<std::collections::HashMap<PreSkinGroupKey, wgpu::BindGroup>>,
}

impl PreSkinPipelines {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let storage_ro = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("pre-skin"),
            entries: &[
                storage_ro(0),
                storage_ro(1),
                storage_ro(2),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<PreSkinParams>() as u64,
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        let pipeline = {
            let device = device.clone();
            let layout = layout.clone();
            Deferred::anywhere(move || {
                crate::deferred::note(|| "pre-skin cs_pre_skin".into());
                let module = crate::gpu::shader_module(
                    &device,
                    wgpu::ShaderModuleDescriptor {
                        label: Some("pre-skin"),
                        source: wgpu::ShaderSource::Wgsl(source().into()),
                    },
                );
                let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("pre-skin"),
                    bind_group_layouts: &[Some(&layout)],
                    immediate_size: 0,
                });
                crate::traffic::create_compute_pipeline(
                    &device,
                    &wgpu::ComputePipelineDescriptor {
                        label: Some("pre-skin"),
                        layout: Some(&pl),
                        module: &module,
                        entry_point: Some("cs_pre_skin"),
                        compilation_options: Default::default(),
                        cache: crate::pipeline_cache::of(&device).as_ref(),
                    },
                )
            })
        };
        let joints = crate::traffic::create_buffer(
            device,
            &wgpu::BufferDescriptor {
                label: Some("pre-skin joints"),
                size: 4096,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );
        let params = crate::traffic::create_buffer(
            device,
            &wgpu::BufferDescriptor {
                label: Some("pre-skin params"),
                size: PARAM_STRIDE * 16,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );
        Self {
            layout,
            pipeline,
            joints,
            params,
            groups: Default::default(),
        }
    }

    /// Uploads the frame's joint matrices and per-job uniforms, then dispatches
    /// single-pass compute pre-skinning for all active jobs into `palette_out`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn encode<'a>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut crate::Encoder,
        jobs: &[PreSkinJob],
        joint_matrices: &[[[f32; 4]; 4]],
        mesh_vertex: impl Fn(usize) -> (&'a wgpu::Buffer, Option<(u64, u64)>),
        skins: &[Option<wgpu::Buffer>],
        morphs_view: &wgpu::TextureView,
        palette_out: &wgpu::TextureView,
        section: Option<u32>,
    ) {
        if jobs.is_empty() || joint_matrices.is_empty() {
            return;
        }
        let joints_bytes = bytemuck::cast_slice::<[[f32; 4]; 4], u8>(joint_matrices);
        if joints_bytes.len() as u64 > self.joints.size() {
            self.joints = crate::traffic::create_buffer(
                device,
                &wgpu::BufferDescriptor {
                    label: Some("pre-skin joints"),
                    size: (joints_bytes.len() as u64).next_power_of_two(),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                },
            );
            self.groups.borrow_mut().clear();
        }
        crate::traffic::write_buffer(queue, &self.joints, 0, joints_bytes);

        let params_bytes_len = jobs.len() as u64 * PARAM_STRIDE;
        if params_bytes_len > self.params.size() {
            self.params = crate::traffic::create_buffer(
                device,
                &wgpu::BufferDescriptor {
                    label: Some("pre-skin params"),
                    size: params_bytes_len.next_power_of_two(),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                },
            );
            self.groups.borrow_mut().clear();
        }
        let mut param_buf = vec![0u8; params_bytes_len as usize];
        let sec = section.unwrap_or(0);
        for (idx, job) in jobs.iter().enumerate() {
            let p = PreSkinParams {
                head: [
                    job.vertex_count,
                    job.palette_first,
                    job.palette_count,
                    job.out_slot,
                ],
                morph: [job.morph_first, job.morph_targets, job.weights_slot, sec],
                model: job.transform.to_cols_array_2d(),
            };
            let off = idx * PARAM_STRIDE as usize;
            param_buf[off..off + std::mem::size_of::<PreSkinParams>()]
                .copy_from_slice(bytemuck::bytes_of(&p));
        }
        crate::traffic::write_buffer(queue, &self.params, 0, &param_buf);

        // Keep cache bounded if dynamic meshes churn.
        let mut cache = self.groups.borrow_mut();
        if cache.len() > 256 {
            cache.clear();
        }
        let pipeline = self.pipeline.wait();
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("pre-skin"),
            timestamp_writes: None,
        });
        pass.set_pipeline(pipeline);
        for (idx, job) in jobs.iter().enumerate() {
            let (vertices, pooled) = mesh_vertex(job.mesh);
            let Some(skin_buf) = skins[job.mesh].as_ref() else {
                continue;
            };
            let (offset, size) = pooled.unwrap_or((0, vertices.size()));
            let key = (
                vertices.clone(),
                pooled,
                skin_buf.clone(),
                morphs_view.clone(),
                palette_out.clone(),
            );
            let group = cache.entry(key).or_insert_with(|| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("pre-skin"),
                    layout: &self.layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self.joints.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: vertices,
                                offset,
                                size: wgpu::BufferSize::new(size),
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: skin_buf.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &self.params,
                                offset: 0,
                                size: wgpu::BufferSize::new(
                                    std::mem::size_of::<PreSkinParams>() as u64
                                ),
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::TextureView(morphs_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: wgpu::BindingResource::TextureView(palette_out),
                        },
                    ],
                })
            });
            let dyn_off = (idx as u64 * PARAM_STRIDE) as u32;
            pass.set_bind_group(0, &*group, &[dyn_off]);
            pass.dispatch_workgroups(job.vertex_count.div_ceil(64), 1, 1);
        }
    }
}
