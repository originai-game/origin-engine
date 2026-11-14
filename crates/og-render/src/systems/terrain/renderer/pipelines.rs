//! The terrain renderer's layouts and pipelines.

use super::*;

pub(super) fn heights_entry() -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: 2,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

impl TerrainRenderer {
    /// The layer-array variant: the group layout with the albedo (5) and
    /// normal (9) maps as 2D arrays and bindings 6 to 8 and 10 to 12 gone,
    /// its lit, prepass and shadow pipelines, and (kept on `self`) the
    /// one-layer stand-ins for a missing array.
    pub(super) fn array_variant(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> (wgpu::BindGroupLayout, TerrainPasses) {
        // Compute too: grass generation reads the terrain (grass.rs).
        let vf = wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE;
        let tex = |binding, filterable| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: vf,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable },
                view_dimension: wgpu::TextureViewDimension::D2Array,
                multisampled: false,
            },
            count: None,
        };
        let uniform = |binding, dynamic| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: vf,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: dynamic,
                min_binding_size: None,
            },
            count: None,
        };
        // The vertex stage too: the displacement reads the layers' maps.
        let sampler = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: vf,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("terrain (layer arrays)"),
            entries: &[
                uniform(0, false),
                uniform(1, true),
                heights_entry(),
                tex(3, true),
                tex(4, true),
                tex(5, true),
                tex(9, true),
                sampler(13),
                sampler(14),
                uniform(15, false),
            ],
        });
        let passes = terrain_passes(
            device,
            TerrainPassLayouts {
                frame: &self.frame_layout,
                shadow: &self.shadow_layout,
                terrain: &layout,
                stamp: &self.stamp_layout,
                fx: &self.fx.layout,
                corners: &self.corners_layout,
                corners_write: &self.corners_write_layout,
            },
            self.samples,
            terrain_array_source(&self.lit_source),
            terrain_shadow_array_source(),
            terrain_array_source(&terrain_corners_source()),
            false,
        );
        let solid = |label, rgba: [u8; 4]| {
            wgpu::util::DeviceExt::create_texture_with_data(
                device,
                queue,
                &wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                },
                wgpu::util::TextureDataOrder::LayerMajor,
                &rgba,
            )
            .create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                ..Default::default()
            })
        };
        self.array_stand_ins = Some((
            solid("terrain arrays: white", [255; 4]),
            solid("terrain arrays: flat normal", [128, 128, 255, 255]),
        ));
        (layout, passes)
    }

    /// The passes with the hole test, for the standard layout or (with
    /// `arrayed`) the layer-array variant's, made when a terrain with holes
    /// first needs them (the variant's layout made before).
    pub(super) fn holed_passes(&mut self, device: &wgpu::Device, arrayed: bool) -> TerrainPasses {
        if self.holed[usize::from(arrayed)].is_none() {
            let (layout, lit, shadow, corners) = if arrayed {
                let (layout, _) = self.arrays.as_ref().expect("the variant is made first");
                (
                    layout,
                    terrain_array_source(&self.lit_source),
                    terrain_shadow_array_source(),
                    terrain_array_source(&terrain_corners_source()),
                )
            } else {
                (
                    &self.layout,
                    self.lit_source.clone(),
                    terrain_shadow_source(),
                    terrain_corners_source(),
                )
            };
            let passes = terrain_passes(
                device,
                TerrainPassLayouts {
                    frame: &self.frame_layout,
                    shadow: &self.shadow_layout,
                    terrain: layout,
                    stamp: &self.stamp_layout,
                    fx: &self.fx.layout,
                    corners: &self.corners_layout,
                    corners_write: &self.corners_write_layout,
                },
                self.samples,
                lit,
                shadow,
                corners,
                true,
            );
            self.holed[usize::from(arrayed)] = Some(passes);
        }
        self.holed[usize::from(arrayed)]
            .clone()
            .expect("made above")
    }

    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        p: TerrainPipelines,
    ) -> TerrainRenderer {
        // Compute too: grass generation reads the terrain (grass.rs).
        let vf = wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE;
        let tex = |binding, filterable, dim| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: vf,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable },
                view_dimension: dim,
                multisampled: false,
            },
            count: None,
        };
        let uniform = |binding, dynamic| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: vf,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: dynamic,
                min_binding_size: None,
            },
            count: None,
        };
        // The vertex stage too: the displacement reads the layers' maps.
        let sampler = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: vf,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };
        let array = wgpu::TextureViewDimension::D2Array;
        let d2 = wgpu::TextureViewDimension::D2;
        let mut entries = vec![
            uniform(0, false),
            uniform(1, true),
            heights_entry(),
            tex(3, true, array),
            tex(4, true, array),
        ];
        entries.extend((5..13).map(|b| tex(b, true, d2)));
        entries.extend([sampler(13), sampler(14), uniform(15, false)]);
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("terrain"),
            entries: &entries,
        });
        let stamp_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("terrain stamps"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let storage = |read_only, visibility| {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("terrain corners"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: u32::from(read_only),
                    visibility,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            })
        };
        // Read at binding 1 (the stamps' passes have their uniform at 0 of
        // group 2), written at 0 of the writer's group 0.
        let corners_layout = storage(true, wgpu::ShaderStages::VERTEX);
        let corners_write_layout = storage(false, wgpu::ShaderStages::COMPUTE);
        let fx = fx::TerrainFx::new(device);
        let passes = terrain_passes(
            device,
            TerrainPassLayouts {
                frame: p.frame_layout,
                shadow: p.shadow_layout,
                terrain: &layout,
                stamp: &stamp_layout,
                fx: &fx.layout,
                corners: &corners_layout,
                corners_write: &corners_write_layout,
            },
            p.samples,
            p.off.leave_out(terrain_lit_source()),
            terrain_shadow_source(),
            terrain_corners_source(),
            false,
        );
        let layer_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("terrain layers"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            anisotropy_clamp: 8,
            ..Default::default()
        });
        let map_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("terrain maps"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        // One white layer, viewed as a 2D array.
        let white_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("terrain no control"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            white_tex.as_image_copy(),
            &[255; 4],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        let white_array = white_tex.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        TerrainRenderer {
            layout,
            passes,
            fx,
            settled: false,
            features: crate::pipeline_features::Features::ALL,
            off: p.off,
            arrays: None,
            holed: [None, None],
            array_stand_ins: None,
            frame_layout: p.frame_layout.clone(),
            shadow_layout: p.shadow_layout.clone(),
            lit_source: p.off.leave_out(terrain_lit_source()),
            samples: p.samples,
            layer_sampler,
            map_sampler,
            white: p.white,
            flat_normal: p.flat_normal,
            white_array,
            indices: None,
            scene: 0,
            datas: Vec::new(),
            fade_tick: None,
            restored: None,
            slots: Vec::new(),
            generations: 0,
            max_layers: device.limits().max_texture_array_layers.min(256),
            uploads: 0,
            resident: 0,
            corners_layout,
            corners_write_layout,
            stamp_layout,
            stamp_slots: None,
            stamp_meshes: Default::default(),
        }
    }
}
