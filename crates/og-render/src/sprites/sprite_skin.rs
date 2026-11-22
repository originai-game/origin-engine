//! 2D linear blend skinning follows `skin.rs`: CPU bones, vertex-stage
//! weighting. GPUSkinning (chengkehan, MIT) is the architectural reference;
//! unlike its fixed influence count, rig2d keeps every bone-local influence.
//! Group 3 is private to these mesh pipelines; sprite and 3D layouts do not change.

use crate::gpu::data_texture::DataTexture;
use crate::{RenderScene, SpriteBlend};

pub(crate) fn source() -> String {
    format!(
        "{}{}",
        crate::sprite_mesh::source(),
        wgsl!("sprite_skin.wgsl")
    )
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Influence {
    position: [f32; 2],
    weight: f32,
    bone: u32,
}

pub(crate) struct SkinRenderer {
    layout: wgpu::BindGroupLayout,
    /// Data textures (data_texture.rs) the vertex stage reads.
    bones: DataTexture,
    influences: DataTexture,
    group: wgpu::BindGroup,
    pipelines: [wgpu::RenderPipeline; 2],
    normals: Option<[wgpu::RenderPipeline; 2]>,
}

impl SkinRenderer {
    pub fn supported(flags: wgpu::DownlevelFlags) -> bool {
        flags.contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)
    }

    pub fn new(device: &wgpu::Device, layouts: [&wgpu::BindGroupLayout; 3], samples: u32) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite skin"),
            entries: &[
                crate::gpu::data_texture::data_entry(0, wgpu::ShaderStages::VERTEX, false),
                crate::gpu::data_texture::data_entry(1, wgpu::ShaderStages::VERTEX, true),
            ],
        });
        let texture = |label, format| {
            DataTexture::new(device, label, format, 1, wgpu::TextureUsages::COPY_DST)
        };
        let bones = texture("sprite bones", wgpu::TextureFormat::Rgba32Float);
        let influences = texture("sprite influences", wgpu::TextureFormat::Rgba32Uint);
        let group = group(device, &layout, &bones, &influences);
        let pipelines = pipelines(device, layouts, &layout, samples, false);
        Self {
            layout,
            bones,
            influences,
            group,
            pipelines,
            normals: None,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RenderScene,
        normal_layouts: Option<[&wgpu::BindGroupLayout; 3]>,
        samples: u32,
    ) {
        if self.normals.is_none()
            && let Some(layouts) = normal_layouts
        {
            self.normals = Some(pipelines(device, layouts, &self.layout, samples, true));
        }
        let mut bones = Vec::new();
        let mut influences = Vec::new();
        for mesh in &scene.sprite.sprite_meshes {
            if let Some(skin) = &mesh.skin {
                let base = bones.len() as u32;
                bones.extend(skin.bones.iter().map(|m| m.to_cols_array()));
                influences.extend(skin.influences.iter().map(|i| Influence {
                    position: i.position.to_array(),
                    weight: i.weight,
                    bone: base + i.bone,
                }));
            }
        }
        let grew =
            self.bones
                .grow_and_write(device, queue, "sprite bones", bytemuck::cast_slice(&bones))
                | self.influences.grow_and_write(
                    device,
                    queue,
                    "sprite influences",
                    bytemuck::cast_slice(&influences),
                );
        if grew {
            self.group = group(device, &self.layout, &self.bones, &self.influences);
        }
    }

    pub fn bind(&self, pass: &mut wgpu::RenderPass<'_>, blend: SpriteBlend, normal: bool) {
        let pipelines = if normal {
            self.normals.as_ref().expect("prepared before draw")
        } else {
            &self.pipelines
        };
        pass.set_pipeline(&pipelines[usize::from(blend == SpriteBlend::Additive)]);
        pass.set_bind_group(3, &self.group, &[]);
    }
}

fn group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    bones: &DataTexture,
    influences: &DataTexture,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("sprite skin"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&bones.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&influences.view),
            },
        ],
    })
}

fn pipelines(
    device: &wgpu::Device,
    layouts: [&wgpu::BindGroupLayout; 3],
    skin: &wgpu::BindGroupLayout,
    samples: u32,
    normal: bool,
) -> [wgpu::RenderPipeline; 2] {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("sprite skin"),
        bind_group_layouts: &[
            Some(layouts[0]),
            Some(layouts[1]),
            Some(layouts[2]),
            Some(skin),
        ],
        immediate_size: 0,
    });
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("sprite skin"),
        source: wgpu::ShaderSource::Wgsl(source().into()),
    });
    [
        wgpu::BlendState::ALPHA_BLENDING,
        wgpu::BlendState {
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
        },
    ]
    .map(|blend| {
        crate::hdr_pipeline(
            device,
            crate::HdrPipeline {
                alpha_to_coverage: false,
                label: "sprite skin",
                layout: &layout,
                module: &module,
                vs: "vs_mesh_skin",
                fs: if normal { "fs_mesh_normal" } else { "fs_mesh" },
                buffers: &[Some(crate::sprite_mesh::MESH_LAYOUT)],
                cull: None,
                depth_write: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                blend: Some(blend),
                samples,
            },
        )
    })
}
