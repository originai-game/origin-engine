//! Midpoint local shadow maps (`docs/design.md` §2.6): what a spot's,
//! a cube face's, a paraboloid hemisphere's or a rect light's shadow layer
//! holds is the depth halfway between its casters' near and far sides
//! (Woo, "The Shadow Depth Map Revisited", Graphics Gems III, 1992), so a
//! surface never compares against its own depth: it lies before the
//! midpoint of the caster it belongs to, and a receiver behind a caster
//! lies past it.
//!
//! A face is drawn twice by the same caster pipelines, which cull their
//! front faces and so keep each caster's far side. The first raster is
//! the face as it is, into its layer of the local array; the second is the
//! face mirrored in x (its view projection's x row negated, a paraboloid's
//! sign in `ShadowPass.paraboloid.z`, shadow_pass.wgsl `shadow_mirror`),
//! into the same layer of [`LocalMid`]'s array: the mirror flips every
//! triangle's winding, so the culling keeps the near sides instead. Both
//! rasters go through the static shadow cache as layers of their own;
//! once both are drawn, [`LocalMid::resolve`] writes the midpoint into the
//! live layer (shadow_mid.wgsl). A double-sided caster is drawn whole by
//! both and resolves to its own plane, a closed one to its inside.
//!
//! The cascades keep their single raster of far sides and compare closed
//! receivers past their own depth (lighting.wgsl `shadow_closed`).

use super::resources::ShadowMap;

const WORKGROUP: u32 = 8;

/// The near-side rasters of the local shadow layers and the pass that
/// resolves each layer's midpoint.
pub(crate) struct LocalMid {
    layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
    /// The near rasters (one per shadow layer of the local array), made
    /// for the array they resolve into, and each layer's bind group.
    near: Option<Near>,
}

pub(crate) struct Near {
    /// The local array the bind groups write.
    of: wgpu::Texture,
    pub(crate) map: ShadowMap,
    groups: Vec<wgpu::BindGroup>,
}

impl LocalMid {
    pub(crate) fn new(device: &wgpu::Device) -> LocalMid {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("local shadow midpoint"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::R32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("local shadow midpoint"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/shadow_mid.wgsl").into()),
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("local shadow midpoint"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("local shadow midpoint"),
            layout: Some(&pl),
            module: &module,
            entry_point: Some("resolve"),
            compilation_options: Default::default(),
            cache: crate::pipeline_cache::of(device).as_ref(),
        });
        LocalMid {
            layout,
            pipeline,
            near: None,
        }
    }

    /// The near rasters for `local`'s first `layers` layers (its shadow
    /// layers; cookies follow them), made again when the array was.
    pub(crate) fn ensure(&mut self, device: &wgpu::Device, local: &ShadowMap, layers: usize) {
        let layers = layers.min(local.layers.len());
        if self
            .near
            .as_ref()
            .is_some_and(|n| n.of == local.texture && n.groups.len() >= layers)
        {
            return;
        }
        let size = local.texture.width();
        let map = ShadowMap::new(device, "local shadow near sides", size, layers as u32);
        let groups = (0..layers)
            .map(|l| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("local shadow midpoint"),
                    layout: &self.layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&local.layers[l]),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&map.layers[l]),
                        },
                    ],
                })
            })
            .collect();
        self.near = Some(Near {
            of: local.texture.clone(),
            map,
            groups,
        });
    }

    /// The near rasters ([`LocalMid::ensure`] made them this frame).
    pub(crate) fn near(&self) -> &ShadowMap {
        &self.near.as_ref().expect("ensured before the passes").map
    }

    /// The near rasters' memory, with their depth buffer.
    pub(crate) fn bytes(&self) -> u64 {
        self.near.as_ref().map_or(0, |n| {
            super::texture_bytes(&n.map.texture) + super::view_bytes(&n.map.depth)
        })
    }

    /// Writes layer `layer`'s midpoint from both its rasters.
    pub(crate) fn resolve(&self, encoder: &mut wgpu::CommandEncoder, layer: usize) {
        let near = self.near.as_ref().expect("ensured before the passes");
        let size = near.map.texture.width().div_ceil(WORKGROUP);
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("local shadow midpoint"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &near.groups[layer], &[]);
        pass.dispatch_workgroups(size, size, 1);
    }
}

/// `view_proj` mirrored in x: the clip position's x negated, which flips
/// every triangle's winding (the near-side raster).
pub(crate) fn mirrored(view_proj: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut m = view_proj;
    for column in &mut m {
        column[0] = -column[0];
    }
    m
}
