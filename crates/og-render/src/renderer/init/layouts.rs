//! The renderer's bind group layouts: the frame group's (with OIT's arena
//! where the device has the linked lists), the sky's, the materials', the
//! effects' and their lean twin (two planar reflections), the sprites' and
//! their effects', the joint palette's and the shadow passes'.

use super::*;

pub(super) struct LayoutsInputs<'a> {
    pub device: &'a wgpu::Device,
    pub desc: &'a RenderDesc,
    pub flags: wgpu::DownlevelFlags,
    pub info: &'a RenderInfo,
}

pub(super) struct LayoutsOutputs {
    pub frame_layout: wgpu::BindGroupLayout,
    pub oit: oit::OitState,
    pub sky_layout: wgpu::BindGroupLayout,
    pub material_layout: wgpu::BindGroupLayout,
    pub planar: bool,
    pub fx_layout: wgpu::BindGroupLayout,
    pub fx_lean_layout: Option<wgpu::BindGroupLayout>,
    pub sprite_fx_layout: wgpu::BindGroupLayout,
    pub palette_layout: wgpu::BindGroupLayout,
    pub sprite_layout: wgpu::BindGroupLayout,
    pub shadow_bind_layout: wgpu::BindGroupLayout,
}

impl Renderer {
    /// Makes the renderer's bind group layouts.
    #[allow(clippy::too_many_lines)]
    pub(super) fn layouts(
        LayoutsInputs {
            device,
            desc,
            flags,
            info,
        }: LayoutsInputs<'_>,
    ) -> LayoutsOutputs {
        // Layouts.
        let [lights_entry, table_entry, refs_entry] = clusters::layout_entries();
        let frame_entries = [
            uniform_entry(0, wgpu::ShaderStages::VERTEX_FRAGMENT),
            texture_entry(1, false, wgpu::TextureViewDimension::D2Array),
            texture_entry(2, false, wgpu::TextureViewDimension::D2Array),
            texture_entry(3, true, wgpu::TextureViewDimension::Cube),
            texture_entry(4, true, wgpu::TextureViewDimension::Cube),
            tex2d(5),
            sampler_entry(6, wgpu::SamplerBindingType::Filtering),
            // The local lights and their clusters: buffers, so the lit
            // pipelines keep their sixteen texture slots.
            lights_entry,
            table_entry,
            refs_entry,
            // Baked light volumes and the rect lights' LTC table
            // (lightmap.rs, area.rs): buffers, as the light tables.
            lightmap::probe_layout_entry(),
            area::ltc_layout_entry(),
            // A sea's caustics (W2, water.rs `Caustics`): a uniform, as the
            // storage buffers a lit pipeline may bind are spent.
            water::caustic_layout_entry(),
            // The main camera's screen-space reflections (ssr.rs): a
            // read-only storage texture, so it takes no texture slot.
            ssr::SsrRenderer::lit_layout_entry(13),
        ];
        let frame_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("frame"),
            entries: &frame_entries,
        });
        // Order-independent transparency (oit.rs): the linked lists where
        // the device can keep them, sized against its own limits.
        let oit = oit::OitState::new(
            device,
            desc.transparency,
            desc.transparency_layers,
            flags,
            &device.limits(),
            &frame_entries,
        );
        // The sky cube, bound by the sky pass alone: in the frame group it
        // would take a texture slot from every lit pipeline, which have
        // sixteen (the downlevel limits the device is asked for).
        let sky_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sky"),
            entries: &[
                texture_entry(0, true, wgpu::TextureViewDimension::Cube),
                // The clouds' noise volume and its repeating sampler
                // (clouds.rs); the vertex stage marches the overcast
                // strip's shared grey through it (sky.wgsl vs_sky_look).
                wgpu::BindGroupLayoutEntry {
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ..texture_entry(1, true, wgpu::TextureViewDimension::D3)
                },
                wgpu::BindGroupLayoutEntry {
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ..sampler_entry(2, wgpu::SamplerBindingType::Filtering)
                },
                // The background's blurriness and map (sky.wgsl SkyParams).
                uniform_entry(3, wgpu::ShaderStages::FRAGMENT),
                // An environment map drawn from its own cube, and the map it
                // blends towards (the sky cube again without a map).
                texture_entry(4, true, wgpu::TextureViewDimension::Cube),
                texture_entry(5, true, wgpu::TextureViewDimension::Cube),
                // The aurora's footprint and table (aurora.rs), read with
                // the frame group's IBL sampler.
                texture_entry(6, true, wgpu::TextureViewDimension::D2),
            ],
        });
        let material_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("material"),
            entries: &[
                // The vertex stage reads a material shader's parameters.
                uniform_entry(0, wgpu::ShaderStages::VERTEX_FRAGMENT),
                tex2d(1),
                tex2d(2),
                tex2d(3),
                tex2d(4),
                tex2d(5),
                tex2d(6),
                sampler_entry(7, wgpu::SamplerBindingType::Filtering),
                tex2d(8),
                tex2d(9),
            ],
        });
        // With two planar reflections the effect group carries the one
        // material shaders read, and the lightmapped pipelines, whose pages
        // would make it a seventeenth texture, take the group without it.
        let planar = planar_slot(info.planar_reflections);
        let fx_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fx"),
            entries: &fx_layout_entries(planar),
        });
        let fx_lean_layout = planar.then(|| {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fx (lightmapped)"),
                entries: &fx_layout_entries(false),
            })
        });
        let sprite_fx_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite fx"),
            entries: &[
                tex2d(0),
                sampler_entry(1, wgpu::SamplerBindingType::Filtering),
                uniform_entry(2, wgpu::ShaderStages::FRAGMENT),
                tex2d(3),
            ],
        });
        let palette_layout = skin::palette_layout(device);
        let sprite_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite"),
            entries: &[
                tex2d(0),
                sampler_entry(1, wgpu::SamplerBindingType::Filtering),
                uniform_entry(2, wgpu::ShaderStages::FRAGMENT),
            ],
        });
        let shadow_bind_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("shadow"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<ShadowPassUniform>() as u64,
                        ),
                    },
                    count: None,
                }],
            });
        LayoutsOutputs {
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
        }
    }
}
