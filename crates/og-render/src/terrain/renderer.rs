//! Terrain renderer initialization, tile uploads and frame preparation.

use super::*;

impl TerrainRenderer {
    /// Draw the lit pass into a target of `samples` per pixel from now on
    /// (the layer-array variant, made later, too).
    pub(crate) fn set_samples(&mut self, samples: u32) {
        self.samples = samples;
        self.passes.lit.set_samples(samples);
        if let Some((_, passes)) = &self.arrays {
            passes.lit.set_samples(samples);
        }
        for passes in self.holed.iter().flatten() {
            passes.lit.set_samples(samples);
        }
    }

    /// Makes one of its pipelines no frame used yet (Renderer::warm).
    pub(crate) fn warm(&self) -> bool {
        let p = &self.passes;
        crate::deferred::warm_one(&[&*p.lit, &*p.prepass, &*p.shadow])
    }

    /// Drops every terrain's bind group (a layer texture was replaced):
    /// the next prepare makes them again from the current views.
    pub(crate) fn forget_groups(&mut self) {
        for slot in &mut self.slots {
            slot.group = None;
        }
    }

    /// The layer-array variant: the group layout with the albedo (5) and
    /// normal (9) maps as 2D arrays and bindings 6 to 8 and 10 to 12 gone,
    /// its lit, prepass and shadow pipelines, and (kept on `self`) the
    /// one-layer stand-ins for a missing array.
    fn array_variant(
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
        let sampler = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("terrain (layer arrays)"),
            entries: &[
                uniform(0, false),
                uniform(1, true),
                tex(2, false),
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
            &self.frame_layout,
            &self.shadow_layout,
            &layout,
            self.samples,
            terrain_lit_array_source(&self.lit_source),
            terrain_shadow_array_source(),
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
    fn holed_passes(&mut self, device: &wgpu::Device, arrayed: bool) -> TerrainPasses {
        if self.holed[usize::from(arrayed)].is_none() {
            let (layout, lit, shadow) = if arrayed {
                let (layout, _) = self.arrays.as_ref().expect("the variant is made first");
                (
                    layout,
                    terrain_lit_array_source(&self.lit_source),
                    terrain_shadow_array_source(),
                )
            } else {
                (
                    &self.layout,
                    self.lit_source.clone(),
                    terrain_shadow_source(),
                )
            };
            let passes = terrain_passes(
                device,
                &self.frame_layout,
                &self.shadow_layout,
                layout,
                self.samples,
                lit,
                shadow,
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
        let sampler = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };
        let array = wgpu::TextureViewDimension::D2Array;
        let d2 = wgpu::TextureViewDimension::D2;
        let mut entries = vec![
            uniform(0, false),
            uniform(1, true),
            tex(2, false, array),
            tex(3, true, array),
            tex(4, true, array),
        ];
        entries.extend((5..13).map(|b| tex(b, true, d2)));
        entries.extend([sampler(13), sampler(14), uniform(15, false)]);
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("terrain"),
            entries: &entries,
        });
        let passes = terrain_passes(
            device,
            p.frame_layout,
            p.shadow_layout,
            &layout,
            p.samples,
            terrain_lit_source(p.frame_wgsl, p.light_tables_wgsl, p.lighting_wgsl),
            terrain_shadow_source(),
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
            arrays: None,
            holed: [None, None],
            array_stand_ins: None,
            frame_layout: p.frame_layout.clone(),
            shadow_layout: p.shadow_layout.clone(),
            lit_source: terrain_lit_source(p.frame_wgsl, p.light_tables_wgsl, p.lighting_wgsl),
            samples: p.samples,
            layer_sampler,
            map_sampler,
            white: p.white,
            flat_normal: p.flat_normal,
            white_array,
            indices: None,
            scene: 0,
            datas: Vec::new(),
            slots: Vec::new(),
            generations: 0,
            max_layers: device.limits().max_texture_array_layers.min(256),
            uploads: 0,
            resident: 0,
        }
    }

    /// The index buffer (bound by the passes that draw terrain).
    pub(crate) fn index_buffer(&self) -> Option<&wgpu::Buffer> {
        self.indices.as_ref().map(|(_, b, _)| b)
    }

    /// The tile arrays for `layout` with `capacity` slots.
    fn make_data(
        &mut self,
        device: &wgpu::Device,
        layout: TerrainLayout,
        capacity: u32,
        revision: u64,
        range: (f32, f32),
        streamed: bool,
    ) -> GpuData {
        // `capacity` slots.
        let layers = capacity;
        let array = |label, side: u32, mips: u32, format, pages: u32| {
            let t = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: side,
                    height: side,
                    depth_or_array_layers: layers * pages,
                },
                mip_level_count: mips,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let v = t.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                ..Default::default()
            });
            (t, v)
        };
        let (heights, heights_view) = array(
            "terrain heights",
            layout.heights_side(),
            1,
            wgpu::TextureFormat::R32Float,
            1,
        );
        let ns = layout.normals_side();
        let (normals, normals_view) = array(
            "terrain normals",
            ns,
            crate::mips::mip_count(ns, ns),
            NORMALS_FORMAT,
            1,
        );
        // Slot s's page p is layer s · pages + p.
        let control = (layout.control > 0).then(|| {
            let cs = layout.control_side();
            array(
                "terrain control",
                cs,
                crate::mips::mip_count(cs, cs),
                wgpu::TextureFormat::Rgba8Unorm,
                layout.pages.max(1),
            )
        });
        let table = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("terrain tiles"),
            size: TABLE_BYTES,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.generations += 1;
        GpuData {
            revision,
            layout,
            capacity,
            generation: self.generations,
            heights,
            heights_view,
            normals,
            normals_view,
            control,
            table,
            table_cpu: vec![-1.0; MAX_TERRAIN_TILES as usize],
            slots: (0..capacity).map(|_| None).collect(),
            by_tile: Default::default(),
            pyramid: None,
            holes: None,
            range,
            streamed,
            drawn: false,
        }
    }

    /// Writes one tile into slot `slot`.
    fn upload_tile(queue: &wgpu::Queue, gpu: &GpuData, slot: u32, tile: &TerrainTile) {
        fn layer(texture: &wgpu::Texture, mip: u32, slot: u32) -> wgpu::TexelCopyTextureInfo<'_> {
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: mip,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: slot,
                },
                aspect: wgpu::TextureAspect::All,
            }
        }
        let size = |side: u32| wgpu::Extent3d {
            width: side,
            height: side,
            depth_or_array_layers: 1,
        };
        let row = |side: u32, bytes: u32| wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(side * bytes),
            rows_per_image: None,
        };
        let hs = gpu.layout.heights_side();
        queue.write_texture(
            layer(&gpu.heights, 0, slot),
            bytemuck::cast_slice(&tile.heights),
            row(hs, 4),
            size(hs),
        );
        let mut at = 0usize;
        for (mip, side) in crate::terrain_tiles::mip_sides(gpu.layout.normals_side())
            .into_iter()
            .enumerate()
        {
            let n = (side * side * 2) as usize;
            if let Some(texels) = tile.normals.get(at..at + n) {
                queue.write_texture(
                    layer(&gpu.normals, mip as u32, slot),
                    bytemuck::cast_slice(texels),
                    row(side, 4),
                    size(side),
                );
            }
            at += n;
        }
        if let (Some((texture, _)), Some(control)) = (&gpu.control, &tile.control) {
            let pages = gpu.layout.pages.max(1);
            let mut at = 0usize;
            for page in 0..pages {
                for (mip, side) in crate::terrain_tiles::mip_sides(gpu.layout.control_side())
                    .into_iter()
                    .enumerate()
                {
                    let n = (side * side * 4) as usize;
                    if let Some(texels) = control.get(at..at + n) {
                        queue.write_texture(
                            layer(texture, mip as u32, slot * pages + page),
                            texels,
                            row(side, 4),
                            size(side),
                        );
                    }
                    at += n;
                }
            }
        }
    }

    /// Brings every drawn data's slots to this frame's tiles: the tiles
    /// each terrain wants around the camera (made from a field in memory,
    /// or taken from those handed over), uploaded into free slots, the
    /// others evicted; streamed tiles fade in over their parents.
    fn sync(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &crate::RenderScene,
        view: TerrainView,
    ) -> Result<(), crate::RenderError> {
        if self.scene != scene.id() {
            self.scene = scene.id();
            self.datas.clear();
            for s in &mut self.slots {
                s.group = None;
            }
        }
        self.uploads = 0;
        self.resident = 0;
        let (sources, revisions) = scene.terrain_sources();
        self.datas.resize_with(sources.len(), || None);
        // Each data's wanted tiles, over every terrain drawing it, best first.
        let mut wanted: og_core::IdMap<usize, (Vec<u32>, u32)> = Default::default();
        for (ti, t) in scene.terrains.iter().enumerate() {
            let i = t.terrain.index();
            let Some(source) = sources.get(i) else {
                continue;
            };
            let layout = source
                .layout()
                .map_err(|e| crate::RenderError::InvalidData(format!("terrain {i}: {e}")))?;
            // The atlases' layers must stay within `max_layers`. (`max`/`min`,
            // not `clamp`: a device with eight layers or fewer must not
            // panic.)
            let capacity = (t.lod.budget() / layout.slot_bytes().max(1))
                .max(8)
                .min((self.max_layers as u64).max(1))
                .min(layout.tile_count() as u64) as u32;
            let view = TerrainView {
                detail: scene.scatter_detail(ti),
                ..view
            };
            let list = terrain_tiles_wanted(&layout, t, source.range(), &view, capacity as usize);
            let entry = wanted.entry(i).or_insert((Vec::new(), capacity));
            entry.1 = entry.1.max(capacity);
            for tile in list {
                if !entry.0.contains(&tile) {
                    entry.0.push(tile);
                }
            }
        }
        for (i, (list, capacity)) in wanted {
            let source = &sources[i];
            let layout = source.layout().map_err(crate::RenderError::InvalidData)?;
            let streamed = matches!(source, TerrainSource::Stream { .. });
            let keep = self.datas[i]
                .take()
                .filter(|g| g.layout == layout && g.capacity == capacity);
            let mut gpu = match keep {
                Some(mut g) if g.revision == revisions[i] => {
                    g.streamed = streamed;
                    g
                }
                Some(mut g) => {
                    // Edited or replaced: every slot is stale.
                    g.revision = revisions[i];
                    g.slots.iter_mut().for_each(|s| *s = None);
                    g.by_tile.clear();
                    g.pyramid = None;
                    g.holes = None;
                    g.range = source.range();
                    g.streamed = streamed;
                    g
                }
                None => self.make_data(
                    device,
                    layout,
                    capacity,
                    revisions[i],
                    source.range(),
                    streamed,
                ),
            };
            gpu.range = source.range();
            // The tiles to hold, best first: every one after its parent.
            let mut target: Vec<(u32, Arc<TerrainTile>)> = Vec::new();
            let mut chosen = std::collections::HashSet::new();
            for &tile in list.iter().take(capacity as usize) {
                if let Some(p) = layout.parent(tile)
                    && !chosen.contains(&p)
                {
                    continue;
                }
                let data = match source {
                    TerrainSource::Stream { tiles, .. } => match tiles.get(&tile) {
                        Some(t) => t.clone(),
                        None => continue,
                    },
                    TerrainSource::Data(d) => match gpu.by_tile.get(&tile) {
                        Some(&s) => gpu.slots[s].as_ref().expect("held").data.clone(),
                        None => {
                            if gpu.pyramid.is_none()
                                && let Some(c) = &d.control
                            {
                                gpu.pyramid = ControlPyramid::new(&layout, c);
                            }
                            if gpu.holes.is_none() {
                                gpu.holes = d.hole_counts();
                            }
                            Arc::new(d.tile(
                                &layout,
                                tile,
                                gpu.pyramid.as_ref(),
                                gpu.holes.as_ref(),
                            ))
                        }
                    },
                };
                chosen.insert(tile);
                target.push((tile, data));
            }
            // Evict what is not held this frame, or was replaced.
            for s in 0..gpu.slots.len() {
                let stale = match &gpu.slots[s] {
                    Some(st) => !target
                        .iter()
                        .any(|(t, d)| *t == st.tile && Arc::ptr_eq(d, &st.data)),
                    None => false,
                };
                if stale {
                    let st = gpu.slots[s].take().expect("held");
                    gpu.by_tile.remove(&st.tile);
                }
            }
            let fade_in = scene
                .terrains
                .iter()
                .filter(|t| t.terrain.index() == i)
                .all(|t| t.fade_in);
            let fresh_fade = if streamed && gpu.drawn && fade_in {
                0.0
            } else {
                1.0
            };
            if !fade_in {
                for st in gpu.slots.iter_mut().flatten() {
                    st.fade = 1.0;
                }
            }
            for (tile, data) in &target {
                if gpu.by_tile.contains_key(tile) {
                    continue;
                }
                let Some(slot) = gpu.slots.iter().position(Option::is_none) else {
                    break;
                };
                Self::upload_tile(queue, &gpu, slot as u32, data);
                gpu.slots[slot] = Some(SlotTile {
                    tile: *tile,
                    data: data.clone(),
                    fade: fresh_fade,
                });
                gpu.by_tile.insert(*tile, slot);
                self.uploads += 1;
            }
            // Fades advance once their parent has fully faded in.
            let parents_in: Vec<bool> = gpu
                .slots
                .iter()
                .map(|s| match s {
                    Some(st) => layout.parent(st.tile).is_none_or(|p| {
                        gpu.by_tile
                            .get(&p)
                            .and_then(|&ps| gpu.slots[ps].as_ref())
                            .is_some_and(|ps| ps.fade >= 1.0)
                    }),
                    None => false,
                })
                .collect();
            for (s, parent_in) in gpu.slots.iter_mut().zip(parents_in) {
                if let Some(st) = s
                    && st.fade < 1.0
                    && parent_in
                {
                    st.fade = (st.fade + 1.0 / FADE_FRAMES).min(1.0);
                }
            }
            let mut table = vec![0.0f32; MAX_TERRAIN_TILES as usize];
            for (s, st) in gpu.slots.iter().enumerate() {
                if let Some(st) = st {
                    table[st.tile as usize] = (s as f32 + 1.0) * 256.0 + (st.fade * 255.0).round();
                }
            }
            if table != gpu.table_cpu {
                queue.write_buffer(&gpu.table, 0, bytemuck::cast_slice(&table));
                gpu.table_cpu = table;
            }
            gpu.drawn = true;
            self.resident += gpu.by_tile.len() as u32;
            self.datas[i] = Some(gpu);
        }
        Ok(())
    }

    /// Uploads what changed and lays out this frame's levels for every
    /// terrain around `camera`. `textures` resolves a layer's texture
    /// handle (`None`: not a texture of this scene).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare<'t>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &crate::RenderScene,
        camera: Vec3,
        far: f32,
        target_height: u32,
        view_proj: Mat4,
        prev_view_proj: Mat4,
        textures: impl Fn(TextureHandle) -> Option<&'t wgpu::TextureView>,
        arrays: impl Fn(TextureArrayHandle) -> Option<&'t wgpu::Texture>,
    ) -> Result<Vec<TerrainDraw>, crate::RenderError> {
        if scene.terrains.is_empty() {
            self.uploads = 0;
            self.resident = 0;
            return Ok(Vec::new());
        }
        let view = TerrainView {
            camera,
            far,
            ..TerrainView::of(&scene.camera, target_height)
        };
        self.sync(device, queue, scene, view)?;
        let ring = scene
            .terrains
            .iter()
            .map(|t| t.lod.ring_quads())
            .max()
            .unwrap_or(64);
        if self.indices.as_ref().is_none_or(|(n, ..)| *n != ring) {
            use wgpu::util::DeviceExt;
            let (idx, ranges) = ring_indices(ring);
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("terrain indices"),
                contents: bytemuck::cast_slice(&idx),
                usage: wgpu::BufferUsages::INDEX,
            });
            self.indices = Some((ring, buffer, ranges));
        }
        let ranges = self.indices.as_ref().expect("made above").2.clone();
        while self.slots.len() < scene.terrains.len() {
            self.slots.push(Slot {
                uniform: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("terrain"),
                    size: std::mem::size_of::<TerrainGpu>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                levels: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("terrain levels"),
                    size: LEVEL_STRIDE * MAX_TERRAIN_LEVELS as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                group: None,
            });
        }
        let mut draws = Vec::with_capacity(scene.terrains.len());
        for (ti, t) in scene.terrains.iter().enumerate() {
            if !t.visible {
                continue;
            }
            if t.layer_arrays.any() && self.arrays.is_none() {
                let variant = self.array_variant(device, queue);
                self.arrays = Some(variant);
            }
            let di = t.terrain.index();
            // A terrain with holes draws with the hole test's pipelines.
            let holed = self
                .datas
                .get(di)
                .and_then(Option::as_ref)
                .is_some_and(|d| d.layout.holes)
                .then(|| self.holed_passes(device, t.layer_arrays.any()));
            let Some(Some(data)) = self.datas.get(di) else {
                return Err(crate::RenderError::InvalidHandle(format!("terrain {di}")));
            };
            let top = data.layout.levels - 1;
            let (ttx, ttz) = data.layout.tiles(top);
            let top_in = (0..ttz).all(|z| {
                (0..ttx).all(|x| data.by_tile.contains_key(&data.layout.index(top, x, z)))
            });
            if !top_in {
                // A streamed terrain whose top level has not arrived: nothing to draw yet.
                continue;
            }
            let mut layers = [LayerGpu::default(); MAX_TERRAIN_LAYERS];
            let mut handles = [None; 2 * TERRAIN_TEXTURED_LAYERS];
            let count = t.layers.len().min(MAX_TERRAIN_LAYERS);
            let arrayed = t.layer_arrays.any();
            for (k, h) in [t.layer_arrays.albedo, t.layer_arrays.normal]
                .into_iter()
                .enumerate()
            {
                if let Some(h) = h
                    && arrays(h).is_none()
                {
                    return Err(crate::RenderError::InvalidHandle(format!(
                        "terrain {} array {}",
                        ["albedo", "normal"][k],
                        h.index()
                    )));
                }
            }
            for (i, l) in t.layers.iter().take(MAX_TERRAIN_LAYERS).enumerate() {
                if arrayed {
                    // The layers' maps are the arrays' layers: two
                    // textures, the layer indices in slope.w and altitude.w.
                    let albedo = t.layer_arrays.albedo.and(l.albedo_layer);
                    let normal = t.layer_arrays.normal.and(l.normal_layer);
                    let bits = u32::from(albedo.is_some()) | (u32::from(normal.is_some()) << 1);
                    let at = |layer: Option<u32>| layer.unwrap_or(0).min(1 << 24) as f32;
                    layers[i] = LayerGpu {
                        color: l.color.extend(l.roughness.clamp(0.04, 1.0)).to_array(),
                        mapping: [
                            1.0 / l.tile.max(1e-3),
                            f32::from(l.triplanar),
                            l.normal_strength,
                            bits as f32,
                        ],
                        slope: [l.slope[0], l.slope[1], l.slope_fade.max(0.0), at(albedo)],
                        altitude: [
                            l.altitude[0],
                            l.altitude[1],
                            l.altitude_fade.max(0.0),
                            at(normal),
                        ],
                        offset: uv_offset(scene.origin, l.tile),
                    };
                    continue;
                }
                // Past the fourth, a layer without arrays has no texture
                // binding: its colour alone.
                let (albedo, normal) = if i < TERRAIN_TEXTURED_LAYERS {
                    (l.albedo, l.normal)
                } else {
                    (None, None)
                };
                for (k, h) in [(i, albedo), (TERRAIN_TEXTURED_LAYERS + i, normal)] {
                    if let Some(h) = h {
                        if textures(h).is_none() {
                            return Err(crate::RenderError::InvalidHandle(format!(
                                "terrain layer {i} texture {}",
                                h.index()
                            )));
                        }
                        handles[k] = Some(h);
                    }
                }
                let bits = u32::from(albedo.is_some()) | (u32::from(normal.is_some()) << 1);
                layers[i] = LayerGpu {
                    color: l.color.extend(l.roughness.clamp(0.04, 1.0)).to_array(),
                    mapping: [
                        1.0 / l.tile.max(1e-3),
                        f32::from(l.triplanar),
                        l.normal_strength,
                        bits as f32,
                    ],
                    slope: [l.slope[0], l.slope[1], l.slope_fade.max(0.0), 0.0],
                    altitude: [l.altitude[0], l.altitude[1], l.altitude_fade.max(0.0), 0.0],
                    offset: uv_offset(scene.origin, l.tile),
                };
            }
            let lay = data.layout;
            let model = t.transform;
            let n3 = glam::Mat3::from_mat4(model).inverse().transpose();
            let mut pyramid = [[0.0f32; 4]; MAX_TERRAIN_PYRAMID as usize];
            for k in 0..lay.levels {
                let (x, z) = lay.tiles(k);
                pyramid[k as usize] = [lay.first(k) as f32, x as f32, z as f32, 0.0];
            }
            let gpu = TerrainGpu {
                model: model.to_cols_array_2d(),
                normal: [
                    n3.x_axis.extend(0.0).to_array(),
                    n3.y_axis.extend(0.0).to_array(),
                    n3.z_axis.extend(0.0).to_array(),
                ],
                origin: model.w_axis.truncate().extend(count as f32).to_array(),
                extent: [lay.size.x, lay.size.y, lay.columns as f32, lay.rows as f32],
                params: [
                    t.height_blend.max(0.0),
                    f32::from(lay.control > 0),
                    f32::from(lay.holes),
                    0.0,
                ],
                tiles: [
                    lay.tile as f32,
                    lay.control as f32,
                    lay.levels as f32,
                    lay.pages.max(1) as f32,
                ],
                view_proj: view_proj.to_cols_array_2d(),
                prev_view_proj: prev_view_proj.to_cols_array_2d(),
                pyramid,
                layers,
                grass: crate::grass::far_gpu(scene, ti),
            };
            let slot = &mut self.slots[ti];
            queue.write_buffer(&slot.uniform, 0, bytemuck::bytes_of(&gpu));

            // The levels are laid out in the terrain's own space: the camera
            // brought into it, the far plane by the terrain's horizontal scale.
            let clipmap = t.clipmap(lay, camera, far);
            let unit = clipmap.unit;
            let lo = -lay.size * 0.5;
            let cam = clipmap.camera;
            let n = clipmap.ring;
            let list = clipmap.levels;
            let morph_width = clipmap.morph_width;
            let morph_start = clipmap.morph_start;
            let mut level_bytes = vec![0u8; LEVEL_STRIDE as usize * list.len().max(1)];
            let mut out = Vec::with_capacity(list.len());
            let (hmin, hmax) = data.range;
            for (k, l) in list.iter().enumerate() {
                let g = LevelGpu {
                    center: [
                        l.center[0] as f32,
                        l.center[1] as f32,
                        l.step as f32,
                        k as f32,
                    ],
                    grid: [
                        n as f32,
                        morph_start,
                        if l.morph { morph_width } else { 0.0 },
                        0.0,
                    ],
                    camera: [cam.x, cam.y, unit, 0.0],
                };
                let at = k * LEVEL_STRIDE as usize;
                level_bytes[at..at + std::mem::size_of::<LevelGpu>()]
                    .copy_from_slice(bytemuck::bytes_of(&g));
                let indices = match l.hole {
                    None => ranges[0].clone(),
                    Some((ox, oz)) => ranges[1 + (ox + 2 * oz) as usize].clone(),
                };
                // Bounds: the level's square on the terrain, its height range.
                let half = n as f64 * 0.5 * l.step;
                let a = Vec2::new(
                    ((l.center[0] - half).max(0.0) * unit as f64) as f32,
                    ((l.center[1] - half).max(0.0) * unit as f64) as f32,
                );
                let b = Vec2::new(
                    ((l.center[0] + half) * unit as f64).min(lay.size.x as f64) as f32,
                    ((l.center[1] + half) * unit as f64).min(lay.size.y as f64) as f32,
                );
                let box_min = Vec3::new(lo.x + a.x, hmin, lo.y + a.y);
                let box_max = Vec3::new(lo.x + b.x, hmax, lo.y + b.y);
                let (center, radius) = world_sphere(model, box_min, box_max);
                out.push(LevelDraw {
                    offset: (k as u64 * LEVEL_STRIDE) as u32,
                    indices,
                    center,
                    radius,
                });
            }
            queue.write_buffer(&slot.levels, 0, &level_bytes);
            let key: GroupKey = (di, data.generation, handles, t.layer_arrays);
            if slot.group.as_ref().is_none_or(|(k, _)| *k != key) && arrayed {
                let (layout, _) = self.arrays.as_ref().expect("made above");
                let (white, flat) = self
                    .array_stand_ins
                    .as_ref()
                    .expect("made with the variant");
                let view = |h: Option<TextureArrayHandle>, fallback: &wgpu::TextureView| {
                    h.and_then(&arrays).map_or_else(
                        || fallback.clone(),
                        |t| {
                            t.create_view(&wgpu::TextureViewDescriptor {
                                dimension: Some(wgpu::TextureViewDimension::D2Array),
                                ..Default::default()
                            })
                        },
                    )
                };
                let (albedo, normal) = (
                    view(t.layer_arrays.albedo, white),
                    view(t.layer_arrays.normal, flat),
                );
                let control = data.control.as_ref().map_or(&self.white_array, |c| &c.1);
                let level = wgpu::BufferBinding {
                    buffer: &slot.levels,
                    offset: 0,
                    size: wgpu::BufferSize::new(std::mem::size_of::<LevelGpu>() as u64),
                };
                let entries = [
                    (0, slot.uniform.as_entire_binding()),
                    (1, wgpu::BindingResource::Buffer(level)),
                    (2, wgpu::BindingResource::TextureView(&data.heights_view)),
                    (3, wgpu::BindingResource::TextureView(&data.normals_view)),
                    (4, wgpu::BindingResource::TextureView(control)),
                    (5, wgpu::BindingResource::TextureView(&albedo)),
                    (9, wgpu::BindingResource::TextureView(&normal)),
                    (13, wgpu::BindingResource::Sampler(&self.layer_sampler)),
                    (14, wgpu::BindingResource::Sampler(&self.map_sampler)),
                    (15, data.table.as_entire_binding()),
                ]
                .map(|(binding, resource)| wgpu::BindGroupEntry { binding, resource });
                let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("terrain (layer arrays)"),
                    layout,
                    entries: &entries,
                });
                slot.group = Some((key, group));
            }
            if slot.group.as_ref().is_none_or(|(k, _)| *k != key) {
                let white = &self.white;
                let flat = &self.flat_normal;
                let view = |h: Option<TextureHandle>, fallback| match h {
                    Some(h) => textures(h).unwrap_or(fallback),
                    None => fallback,
                };
                let control = data.control.as_ref().map_or(&self.white_array, |c| &c.1);
                let mut resources = vec![
                    slot.uniform.as_entire_binding(),
                    wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &slot.levels,
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<LevelGpu>() as u64),
                    }),
                    wgpu::BindingResource::TextureView(&data.heights_view),
                    wgpu::BindingResource::TextureView(&data.normals_view),
                    wgpu::BindingResource::TextureView(control),
                ];
                for h in &handles[..TERRAIN_TEXTURED_LAYERS] {
                    resources.push(wgpu::BindingResource::TextureView(view(*h, white)));
                }
                for h in &handles[TERRAIN_TEXTURED_LAYERS..] {
                    resources.push(wgpu::BindingResource::TextureView(view(*h, flat)));
                }
                resources.push(wgpu::BindingResource::Sampler(&self.layer_sampler));
                resources.push(wgpu::BindingResource::Sampler(&self.map_sampler));
                resources.push(data.table.as_entire_binding());
                let entries: Vec<_> = resources
                    .into_iter()
                    .enumerate()
                    .map(|(i, resource)| wgpu::BindGroupEntry {
                        binding: i as u32,
                        resource,
                    })
                    .collect();
                let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("terrain"),
                    layout: &self.layout,
                    entries: &entries,
                });
                slot.group = Some((key, group));
            }
            draws.push(TerrainDraw {
                group: slot.group.as_ref().expect("made above").1.clone(),
                passes: match (holed, &self.arrays) {
                    (Some(passes), _) => passes,
                    (None, Some((_, passes))) if arrayed => passes.clone(),
                    _ => self.passes.clone(),
                },
                levels: out,
                cast_shadows: t.cast_shadows,
                terrain: ti,
                arrayed,
            });
        }
        Ok(draws)
    }

    /// Its group layout: the standard one, or (`arrayed`, once made) the
    /// layer-array variant's.
    pub(crate) fn group_layout(&self, arrayed: bool) -> Option<&wgpu::BindGroupLayout> {
        if arrayed {
            self.arrays.as_ref().map(|(l, _)| l)
        } else {
            Some(&self.layout)
        }
    }

    /// A terrain data's size along x and z and its height range, once uploaded.
    pub(crate) fn extent_of(&self, data: usize) -> Option<(Vec2, (f32, f32))> {
        let d = self.datas.get(data)?.as_ref()?;
        Some((d.layout.size, d.range))
    }

    /// The world bounds of the terrains drawn this frame (centre, radius),
    /// for the shadow fit.
    pub(crate) fn bounds(&self, scene: &crate::RenderScene) -> Vec<(Vec3, f32)> {
        scene
            .terrains
            .iter()
            .filter_map(|t| {
                let d = self.datas.get(t.terrain.index())?.as_ref()?;
                let s = d.layout.size;
                let lo = Vec3::new(-s.x * 0.5, d.range.0, -s.y * 0.5);
                let hi = Vec3::new(s.x * 0.5, d.range.1, s.y * 0.5);
                Some(world_sphere(t.transform, lo, hi))
            })
            .collect()
    }
}

/// How far one metre along the terrain's x and z reaches in the world (the
/// larger of the two): what the far plane is divided by in its own space.
pub(crate) fn horizontal_scale(model: Mat4) -> f32 {
    model
        .x_axis
        .truncate()
        .length()
        .max(model.z_axis.truncate().length())
        .max(1e-6)
}

/// The world bounding sphere (centre, radius) of a box in the terrain's
/// space.
pub(crate) fn world_sphere(model: Mat4, lo: Vec3, hi: Vec3) -> (Vec3, f32) {
    let center = model.transform_point3((lo + hi) * 0.5);
    let mut radius: f32 = 0.0;
    for i in 0..8 {
        let corner = Vec3::new(
            if i & 1 == 0 { lo.x } else { hi.x },
            if i & 2 == 0 { lo.y } else { hi.y },
            if i & 4 == 0 { lo.z } else { hi.z },
        );
        radius = radius.max(model.transform_point3(corner).distance(center));
    }
    (center, radius)
}

/// Draws the levels of `draws` that touch `visible` (group 1: the terrain;
/// group 0 bound by the caller, or per level the reflection probe's frame
/// group `probes` picks: probe_select.rs). Returns (draws, triangles).
pub(crate) fn draw_levels(
    pass: &mut wgpu::RenderPass,
    draws: &[TerrainDraw],
    indices: &wgpu::Buffer,
    which: TerrainPass,
    visible: impl Fn(Vec3, f32) -> bool,
    mut probes: Option<&mut crate::probe_select::Binder>,
) -> (u32, u64) {
    let mut calls = 0;
    let mut triangles = 0;
    pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
    for d in draws {
        if which == TerrainPass::Shadow && !d.cast_shadows {
            continue;
        }
        // Each terrain's own pipeline: the standard one, or the layer-array
        // variant's.
        pass.set_pipeline(match which {
            TerrainPass::Lit => d.passes.lit.get(),
            TerrainPass::Prepass => d.passes.prepass.get(),
            TerrainPass::Shadow => d.passes.shadow.get(),
        });
        for l in &d.levels {
            if !visible(l.center, l.radius) {
                continue;
            }
            if let Some(b) = probes.as_deref_mut() {
                b.at(pass, l.center, l.radius);
            }
            pass.set_bind_group(1, &d.group, &[l.offset]);
            pass.draw_indexed(l.indices.clone(), 0, 0..1);
            calls += 1;
            triangles += (l.indices.len() / 3) as u64;
        }
    }
    if let Some(b) = probes {
        b.reset(pass);
    }
    (calls, triangles)
}

/// A layer's uv at the scene's origin, in f64 and without the whole
/// repeats, so the layers sample the same texels wherever the origin is
/// (`origin.rs`): zero at the world's zero.
fn uv_offset(origin: Vec3, tile: f32) -> [f32; 4] {
    // The shader's own repeats per metre (`mapping.x`, f32).
    let k = f64::from(1.0 / tile.max(1e-3));
    let at = |o: f32| (f64::from(o) * k).rem_euclid(1.0) as f32;
    [at(origin.x), at(origin.y), at(origin.z), 0.0]
}
