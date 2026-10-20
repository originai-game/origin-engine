//! The physical material lobes (three.js `MeshPhysicalMaterial`, glTF
//! `KHR_materials_clearcoat`, `_sheen`, `_transmission`, `_volume`, `_ior`,
//! `_iridescence`, `_anisotropy`): the pipelines a
//! [`StandardMaterial::is_physical`] material draws its lit pass with, and
//! the packing of its maps.
//!
//! A physical material has fourteen maps beyond the base colour, normal,
//! emissive and toon ramp, and a WebGL2 pipeline has sixteen texture slots
//! across its groups (the skinned lit one already uses all sixteen). So
//! its scalar maps are packed, once per set of maps, into the layers of
//! one RGBA8 array texture by a render pass per layer and mip level
//! (`shaders/pack.wgsl`; the implicit derivatives pick each source's mip):
//!
//! | layer | r | g | b | a |
//! |---|---|---|---|---|
//! | standard | metalness (`metalnessMap` b) | roughness (`roughnessMap` g) | `aoMap` r | `dissolveMap` r |
//! | coat | `clearcoatMap` r | `clearcoatRoughnessMap` g | `transmissionMap` r | `thicknessMap` g |
//! | sheen | `sheenColorMap` rgb (stored sRGB) | | | `sheenRoughnessMap` a |
//! | film | `clearcoatNormalMap` xyz (z rebuilt for two channels) | | | `iridescenceMap` r |
//! | anisotropy | `anisotropyMap` rgb | | | `iridescenceThicknessMap` g |
//!
//! Only the layers a material has maps for are made (the uniform names each
//! group's layer, -1 for none), and materials with the same maps share one
//! pack. The physical material group is then the uniform, the base colour,
//! normal, emissive and gradient textures, the pack and the physical
//! uniform: five textures. The physical effect group adds the copy of the
//! opaque scene that transmission refracts: frame 5, material 5, effects 3
//! and skin 1 make fourteen, so every lobe and map draws on WebGL2 too.
//! The shadow, prepass, reflection and scatter passes draw a physical
//! material through its standard group like any other.

use std::collections::HashMap;

use crate::material::StandardMaterial;
use crate::scene::TextureHandle;
use crate::{PbrKey, sampler_entry, tex2d, texture_entry, uniform_entry};

/// The pack's format: linear 8-bit, as the maps it holds.
pub(crate) const PACK_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// `struct PhysicalFactors` in material_physical.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct PhysicalUniform {
    /// xy: clearcoat normal scale, z: 1 with a clearcoat normal map, w: 1
    /// with an anisotropy map.
    pub coat: [f32; 4],
    /// rgb: sheen colour times sheen, w: sheen roughness.
    pub sheen: [f32; 4],
    /// x: transmission, y: thickness, z: ior, w: attenuation distance (0: none).
    pub transmission: [f32; 4],
    /// rgb: attenuation colour.
    pub attenuation: [f32; 4],
    /// x: iridescence, y: the film's ior, zw: its thickness range (nm).
    pub iridescence: [f32; 4],
    /// x: anisotropy, yz: cos and sin of its rotation.
    pub anisotropy: [f32; 4],
    /// The pack layer of the standard, coat, sheen and film groups (-1: none).
    pub layers: [f32; 4],
    /// x: the pack layer of the anisotropy group.
    pub layers2: [f32; 4],
}

/// `struct Pack` in pack.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct PackUniform {
    /// Per output channel: source * 4 + channel, or [`DEFAULT`].
    pub pick: [u32; 4],
    pub fallback: [f32; 4],
    /// x: channels stored sRGB-encoded (a bit each), y: 1 to rebuild b as
    /// a two-channel normal's z.
    pub flags: [u32; 4],
}

/// "Take the fallback" in [`PackUniform::pick`].
const DEFAULT: u32 = 255;

/// One layer of a pack: up to four source textures and its uniform.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PackLayer {
    pub sources: Vec<TextureHandle>,
    pub uniform: PackUniform,
}

/// One map group (a pack layer): each output channel as (slot, source
/// channel), the fallbacks, the sRGB-encoded channels and whether it is
/// a normal map (its z is rebuilt when the source has two channels).
struct Group {
    channels: [(u32, u32); 4],
    fallback: [f32; 4],
    srgb: u32,
    normal: bool,
}

/// The map groups in layer order.
fn groups(m: &StandardMaterial) -> [Group; 5] {
    let group = |channels, fallback, srgb, normal| Group {
        channels,
        fallback,
        srgb,
        normal,
    };
    [
        group(
            [
                (m.metalnessMap, 2),
                (m.roughnessMap, 1),
                (m.aoMap, 0),
                (m.dissolveMap, 0),
            ],
            [1.0; 4],
            0,
            false,
        ),
        group(
            [
                (m.clearcoatMap, 0),
                (m.clearcoatRoughnessMap, 1),
                (m.transmissionMap, 0),
                (m.thicknessMap, 1),
            ],
            [1.0; 4],
            0,
            false,
        ),
        group(
            [
                (m.sheenColorMap, 0),
                (m.sheenColorMap, 1),
                (m.sheenColorMap, 2),
                (m.sheenRoughnessMap, 3),
            ],
            [1.0; 4],
            0b0111,
            false,
        ),
        group(
            [
                (m.clearcoatNormalMap, 0),
                (m.clearcoatNormalMap, 1),
                (m.clearcoatNormalMap, 2),
                (m.iridescenceMap, 0),
            ],
            [0.5, 0.5, 1.0, 1.0],
            0,
            true,
        ),
        group(
            [
                (m.anisotropyMap, 0),
                (m.anisotropyMap, 1),
                (m.anisotropyMap, 2),
                (m.iridescenceThicknessMap, 1),
            ],
            [1.0, 0.5, 1.0, 1.0],
            0,
            false,
        ),
    ]
}

/// The slots of every map a pack holds, in [`groups`] order: what makes two
/// materials' packs the same.
pub(crate) type PackKey = [u32; 20];

/// The material's pack: its key, the layer of each group (-1: none) and
/// the layers to make. `two_channel` tells whether a texture is a
/// two-channel (BC5, EAC RG11) one.
pub(crate) fn pack_plan(
    m: &StandardMaterial,
    two_channel: impl Fn(TextureHandle) -> bool,
) -> (PackKey, [f32; 5], Vec<PackLayer>) {
    let mut key = [0; 20];
    let mut indices = [-1.0; 5];
    let mut layers = Vec::new();
    for (g, group) in groups(m).into_iter().enumerate() {
        let Group {
            channels,
            fallback,
            srgb,
            normal,
        } = group;
        for (c, (slot, _)) in channels.iter().enumerate() {
            key[g * 4 + c] = *slot;
        }
        if channels.iter().all(|(slot, _)| *slot == 0) {
            continue;
        }
        let mut sources: Vec<TextureHandle> = Vec::new();
        let mut pick = [DEFAULT; 4];
        for (c, &(slot, channel)) in channels.iter().enumerate() {
            let Some(h) = TextureHandle::from_slot(slot) else {
                continue;
            };
            let at = sources.iter().position(|&s| s == h).unwrap_or_else(|| {
                sources.push(h);
                sources.len() - 1
            });
            pick[c] = at as u32 * 4 + channel;
        }
        let rebuild = normal && TextureHandle::from_slot(channels[0].0).is_some_and(&two_channel);
        indices[g] = layers.len() as f32;
        layers.push(PackLayer {
            sources,
            uniform: PackUniform {
                pick,
                fallback,
                flags: [srgb, rebuild as u32, 0, 0],
            },
        });
    }
    (key, indices, layers)
}

/// The material's physical uniform, with its groups' pack layers.
pub(crate) fn uniform(m: &StandardMaterial, layers: [f32; 5]) -> PhysicalUniform {
    let sheen = m.sheenColor.max(glam::Vec3::ZERO) * m.sheen.clamp(0.0, 1.0);
    let range = m.iridescenceThicknessRange.max(glam::Vec2::ZERO);
    PhysicalUniform {
        coat: [
            m.clearcoatNormalScale.x,
            m.clearcoatNormalScale.y,
            (m.clearcoatNormalMap != 0) as u32 as f32,
            (m.anisotropyMap != 0) as u32 as f32,
        ],
        sheen: [sheen.x, sheen.y, sheen.z, m.sheenRoughness.clamp(0.0, 1.0)],
        transmission: [
            m.transmission.clamp(0.0, 1.0),
            m.thickness.max(0.0),
            m.ior.clamp(1.0, 2.333),
            m.attenuationDistance.max(0.0),
        ],
        attenuation: [
            m.attenuationColor.x,
            m.attenuationColor.y,
            m.attenuationColor.z,
            0.0,
        ],
        iridescence: [
            m.iridescence.clamp(0.0, 1.0),
            m.iridescenceIOR.clamp(1.0, 2.333),
            range.x,
            range.y,
        ],
        anisotropy: [
            m.anisotropy.clamp(0.0, 1.0),
            m.anisotropyRotation.cos(),
            m.anisotropyRotation.sin(),
            0.0,
        ],
        layers: [layers[0], layers[1], layers[2], layers[3]],
        layers2: [layers[4], 0.0, 0.0, 0.0],
    }
}

/// The physical lit pipelines' layouts, shader and pipelines, and the pack
/// pass. Made when a scene first draws a physical material.
pub(crate) struct PhysicalPipelines {
    /// Group 1: the material (see the module docs).
    pub material_layout: wgpu::BindGroupLayout,
    /// Group 2: the effects plus the scene copy transmission reads.
    pub fx_layout: wgpu::BindGroupLayout,
    /// Static and skinned.
    pub layouts: (wgpu::PipelineLayout, wgpu::PipelineLayout),
    /// The physical shader with the built-in hooks.
    pub module: wgpu::ShaderModule,
    pub pipelines: HashMap<PbrKey, wgpu::RenderPipeline>,
    /// A pack for a material without packed maps (two layers: WebGL2).
    pub empty_pack: wgpu::TextureView,
    pack_layout: wgpu::BindGroupLayout,
    pack_pipeline: wgpu::RenderPipeline,
}

impl PhysicalPipelines {
    pub fn new(
        device: &wgpu::Device,
        frame_layout: &wgpu::BindGroupLayout,
        palette_layout: &wgpu::BindGroupLayout,
        source: String,
    ) -> PhysicalPipelines {
        let material_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("physical material"),
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::VERTEX_FRAGMENT),
                tex2d(1),
                tex2d(2),
                tex2d(3),
                sampler_entry(4, wgpu::SamplerBindingType::Filtering),
                tex2d(5),
                texture_entry(6, true, wgpu::TextureViewDimension::D2Array),
                uniform_entry(7, wgpu::ShaderStages::FRAGMENT),
            ],
        });
        let fx_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("physical fx"),
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::VERTEX_FRAGMENT),
                tex2d(1),
                tex2d(2),
                sampler_entry(3, wgpu::SamplerBindingType::Filtering),
                tex2d(4),
            ],
        });
        let layout = |label, groups: &[&wgpu::BindGroupLayout]| {
            let groups: Vec<_> = groups.iter().map(|l| Some(*l)).collect();
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &groups,
                immediate_size: 0,
            })
        };
        let layouts = (
            layout("physical", &[frame_layout, &material_layout, &fx_layout]),
            layout(
                "physical skinned",
                &[frame_layout, &material_layout, &fx_layout, palette_layout],
            ),
        );
        let module = crate::shader(device, "physical", source);
        let pack_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("pack"),
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::FRAGMENT),
                tex2d(1),
                tex2d(2),
                tex2d(3),
                tex2d(4),
                sampler_entry(5, wgpu::SamplerBindingType::Filtering),
            ],
        });
        let pack_module = crate::shader(device, "pack", PACK_WGSL.to_string());
        let pack_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pack"),
            layout: Some(&layout("pack", &[&pack_layout])),
            vertex: wgpu::VertexState {
                module: &pack_module,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &pack_module,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: PACK_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let empty_pack = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("empty pack"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: crate::gl_safe_array_layers(1),
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: PACK_FORMAT,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                ..Default::default()
            });
        PhysicalPipelines {
            material_layout,
            fx_layout,
            layouts,
            module,
            pipelines: HashMap::new(),
            empty_pack,
            pack_layout,
            pack_pipeline,
        }
    }

    /// Records the passes that pack `layers` (each with its sources' views,
    /// four, the unused ones a placeholder) into a new array texture of
    /// `size` with a full mip chain; returns its array view.
    pub fn pack(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        layers: &[(PackUniform, [wgpu::TextureView; 4])],
        (width, height): (u32, u32),
        sampler: &wgpu::Sampler,
    ) -> wgpu::TextureView {
        use wgpu::util::DeviceExt;
        let levels = crate::mips::mip_count(width, height);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("material pack"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: crate::gl_safe_array_layers(layers.len() as u32),
            },
            mip_level_count: levels,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: PACK_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        for (layer, (uniform, views)) in layers.iter().enumerate() {
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("pack"),
                contents: bytemuck::bytes_of(uniform),
                usage: wgpu::BufferUsages::UNIFORM,
            });
            let group = crate::bind_group(
                device,
                "pack",
                &self.pack_layout,
                &[
                    buffer.as_entire_binding(),
                    wgpu::BindingResource::TextureView(&views[0]),
                    wgpu::BindingResource::TextureView(&views[1]),
                    wgpu::BindingResource::TextureView(&views[2]),
                    wgpu::BindingResource::TextureView(&views[3]),
                    wgpu::BindingResource::Sampler(sampler),
                ],
            );
            for level in 0..levels {
                let target = texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("pack level"),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    base_mip_level: level,
                    mip_level_count: Some(1),
                    base_array_layer: layer as u32,
                    array_layer_count: Some(1),
                    ..Default::default()
                });
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("pack"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &target,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                pass.set_pipeline(&self.pack_pipeline);
                pass.set_bind_group(0, &group, &[]);
                pass.draw(0..3, 0..1);
            }
        }
        texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("material pack"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        })
    }
}

/// The texture pack pass's module (`crate::shaders` validates it).
pub(crate) const PACK_WGSL: &str = include_str!("../shaders/pack.wgsl");

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(i: u32) -> u32 {
        TextureHandle::from_index(i as usize).slot()
    }

    #[test]
    fn a_pack_holds_only_the_groups_with_maps() {
        let m = StandardMaterial {
            sheen: 1.0,
            clearcoatMap: slot(0),
            transmissionMap: slot(0),
            thicknessMap: slot(1),
            anisotropyMap: slot(2),
            ..Default::default()
        };
        let (_, indices, layers) = pack_plan(&m, |_| false);
        assert_eq!(indices, [-1.0, 0.0, -1.0, -1.0, 1.0]);
        assert_eq!(layers.len(), 2);
        // One source for clearcoat and transmission, another for thickness.
        assert_eq!(layers[0].sources.len(), 2);
        assert_eq!(layers[0].uniform.pick, [0, DEFAULT, 0, 4 + 1]);
        assert_eq!(layers[1].uniform.pick, [0, 1, 2, DEFAULT]);
        assert_eq!(layers[1].uniform.fallback, [1.0, 0.5, 1.0, 1.0]);
    }

    #[test]
    fn a_two_channel_coat_normal_is_rebuilt_and_sheen_is_stored_srgb() {
        let m = StandardMaterial {
            clearcoatNormalMap: slot(3),
            sheenColorMap: slot(4),
            ..Default::default()
        };
        let (key, indices, layers) = pack_plan(&m, |h| h.index() == 3);
        assert_eq!(indices, [-1.0, -1.0, 0.0, 1.0, -1.0]);
        assert_eq!(layers[0].uniform.flags, [0b0111, 0, 0, 0]);
        assert_eq!(layers[1].uniform.flags, [0, 1, 0, 0]);
        assert_eq!(key[8], slot(4));
        assert!(m.is_physical());
        assert!(!m.is_transmissive());
    }

    #[test]
    fn the_standard_material_is_not_physical() {
        let mut m = StandardMaterial {
            clearcoat: 1.0,
            clearcoatRoughness: 0.2,
            ..Default::default()
        };
        assert!(
            !m.is_physical(),
            "the scalar coat draws on the standard path"
        );
        m.ior = 1.45;
        assert!(m.is_physical());
        m.toon = 1;
        assert!(!m.is_physical(), "toon replaces the lighting model");
        let t = StandardMaterial {
            transmission: 1.0,
            ..Default::default()
        };
        assert!(t.is_transmissive());
    }
}
