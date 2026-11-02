//! Normal-mapped 2D receivers (R11): a texture with a normal map
//! ([`crate::RenderScene::set_texture_normal`]) is drawn, in a lit 2D
//! layer, through the sprite module's `fs_normal` entry points (sprite.wgsl;
//! `fs_mesh_normal` for the 2D rig meshes), which light it by how its
//! normal faces the lights: the light map's light, scaled by the normal
//! against the lights' mean direction (light2d.wgsl `fs_dir`, the frame's
//! `lightdir` target, kept only while a texture has a normal map). Their
//! group 1 is the sprite group with the normal map at binding 3, one per
//! texture. Every other texture, and every frame without 2D lighting,
//! draws through the sprite pipelines as before, unchanged. Made on the
//! first frame that draws one, so a renderer that never does pays nothing.

use crate::effects::SpriteBlend;

/// The pipelines, the layout of their texture groups and the groups.
pub(crate) struct SpriteNormals {
    pub layout: wgpu::BindGroupLayout,
    /// Alpha, additive and untonemapped sprites.
    sprite: [wgpu::RenderPipeline; 3],
    /// Alpha and additive meshes.
    mesh: [wgpu::RenderPipeline; 2],
    /// Per texture index: its group and what it was made from.
    pub groups: Vec<Option<(wgpu::BindGroup, GroupKey)>>,
}

/// What a texture's group reads: its revision, its normal map's index and
/// revision, and whether it samples nearest and through cleanEdge.
pub(crate) type GroupKey = (u64, usize, u64, bool, bool);

impl SpriteNormals {
    /// `layouts`: the sprite pipelines' frame and light-map groups (0, 2).
    pub fn new(
        device: &wgpu::Device,
        layouts: [&wgpu::BindGroupLayout; 2],
        samples: u32,
    ) -> SpriteNormals {
        let tex = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite (normal map)"),
            entries: &[
                tex(0),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                tex(3),
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite (normal map)"),
            bind_group_layouts: &[Some(layouts[0]), Some(&layout), Some(layouts[1])],
            immediate_size: 0,
        });
        // The mesh module holds the sprite module whole.
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sprite (normal map)"),
            source: wgpu::ShaderSource::Wgsl(crate::sprite_mesh::source().into()),
        });
        let mesh_buffers = [Some(crate::sprite_mesh::MESH_LAYOUT)];
        let pipeline = |label, mesh: bool, fs, blend| {
            crate::hdr_pipeline(
                device,
                crate::HdrPipeline {
                    alpha_to_coverage: false,
                    label,
                    layout: &pl,
                    module: &module,
                    vs: if mesh { "vs_mesh" } else { "vs" },
                    fs,
                    buffers: if mesh {
                        &mesh_buffers
                    } else {
                        crate::SPRITE_BUFFERS
                    },
                    cull: None,
                    depth_write: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    blend: Some(blend),
                    samples,
                },
            )
        };
        let additive = wgpu::BlendState {
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
        };
        let alpha = wgpu::BlendState::ALPHA_BLENDING;
        SpriteNormals {
            sprite: [
                pipeline("sprite (normal map)", false, "fs_normal", alpha),
                pipeline("sprite additive (normal map)", false, "fs_normal", additive),
                pipeline(
                    "sprite untonemapped (normal map)",
                    false,
                    "fs_normal_untonemapped",
                    crate::UNTONEMAPPED_BLEND,
                ),
            ],
            mesh: [
                pipeline("sprite mesh (normal map)", true, "fs_mesh_normal", alpha),
                pipeline(
                    "sprite mesh additive (normal map)",
                    true,
                    "fs_mesh_normal",
                    additive,
                ),
            ],
            layout,
            groups: Vec::new(),
        }
    }

    /// Texture `index`'s group, if it has one this frame.
    pub fn group(&self, index: usize) -> Option<&wgpu::BindGroup> {
        self.groups.get(index)?.as_ref().map(|(g, _)| g)
    }

    /// The sprite pipeline for `blend`, tone mapped or not.
    pub fn sprite(&self, blend: SpriteBlend, tone_mapped: bool) -> &wgpu::RenderPipeline {
        match (blend, tone_mapped) {
            (SpriteBlend::Alpha, true) => &self.sprite[0],
            (SpriteBlend::Additive, _) => &self.sprite[1],
            (SpriteBlend::Alpha, false) => &self.sprite[2],
        }
    }

    /// The mesh pipeline for `blend`.
    pub fn mesh(&self, blend: SpriteBlend) -> &wgpu::RenderPipeline {
        match blend {
            SpriteBlend::Alpha => &self.mesh[0],
            SpriteBlend::Additive => &self.mesh[1],
        }
    }
}
