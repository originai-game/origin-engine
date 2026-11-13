//! A terrain's GPU data: its tiles uploaded and kept in step with the scene.

use super::*;

impl TerrainRenderer {
    /// The tile arrays for `layout` with `capacity` slots.
    pub(super) fn make_data(
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
        let array = |label, (width, height): (u32, u32), mips: u32, format, pages: u32| {
            let t = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width,
                    height,
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
        // Unfiltered f32 loads need no sampled texture slot. Keep the exact
        // height/horizon layout, leaving the sixteenth texture for decals.
        let heights = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("terrain heights"),
            size: u64::from(layout.heights_side())
                * u64::from(layout.heights_rows())
                * u64::from(layers)
                * 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let ns = layout.normals_side();
        let (normals, normals_view) = array(
            "terrain normals",
            (ns, ns),
            crate::mips::mip_count(ns, ns),
            NORMALS_FORMAT,
            1,
        );
        // Slot s's page p is layer s · pages + p.
        let control = (layout.control > 0).then(|| {
            let cs = layout.control_side();
            array(
                "terrain control",
                (cs, cs),
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
            normals,
            normals_view,
            control,
            table,
            table_cpu: vec![-1.0; MAX_TERRAIN_TILES as usize],
            slots: (0..capacity).map(|_| None).collect(),
            by_tile: Default::default(),
            pyramid: None,
            holes: None,
            height_pyramid: None,
            holes_of: None,
            range,
            streamed,
            drawn: false,
        }
    }

    /// Writes one tile into slot `slot`.
    pub(super) fn upload_tile(queue: &wgpu::Queue, gpu: &GpuData, slot: u32, tile: &TerrainTile) {
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
        let offset = u64::from(slot) * u64::from(hs) * u64::from(gpu.layout.heights_rows()) * 4;
        queue.write_buffer(&gpu.heights, offset, bytemuck::cast_slice(&tile.heights));
        // The horizon words in the rows under the heights as numbers (exact:
        // each under 2^24), the last row padded.
        let rows = gpu.layout.heights_rows() - hs;
        let mut words = vec![0f32; (rows * hs) as usize];
        for (w, v) in words.iter_mut().zip(tile.horizon.iter()) {
            *w = *v as f32;
        }
        queue.write_buffer(
            &gpu.heights,
            offset + u64::from(hs) * u64::from(hs) * 4,
            bytemuck::cast_slice(&words),
        );
        let mut at = 0usize;
        for (mip, side) in crate::terrain_tiles::mip_sides(gpu.layout.normals_side())
            .into_iter()
            .enumerate()
        {
            let n = (side * side) as usize;
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

    /// Each resident tile's fade and the tick they last stepped at, for a
    /// presentation checkpoint (`renderer/history.rs`).
    pub(crate) fn save(&self) -> TerrainSave {
        let mut fades = Vec::new();
        for (i, d) in self.datas.iter().enumerate() {
            for st in d.iter().flat_map(|d| d.slots.iter().flatten()) {
                fades.push((i, st.tile, st.fade, Arc::as_ptr(&st.data) as usize));
            }
        }
        TerrainSave {
            fades,
            tick: self.fade_tick,
        }
    }

    /// Brings back [`TerrainRenderer::save`]'s fades: a tile resident now
    /// with the same data takes its saved fade, and one the save did not
    /// hold is drawn in (it arrived on another branch).
    pub(crate) fn restore(&mut self, saved: &TerrainSave) {
        for (i, d) in self.datas.iter_mut().enumerate() {
            for st in d.iter_mut().flat_map(|d| d.slots.iter_mut().flatten()) {
                let data = Arc::as_ptr(&st.data) as usize;
                st.fade = saved
                    .fades
                    .iter()
                    .find(|f| f.0 == i && f.1 == st.tile && f.3 == data)
                    .map_or(1.0, |f| f.2);
            }
        }
        self.fade_tick = saved.tick;
        // Tiles the save held that are not resident now take their fade
        // when they are uploaded again (the next sync).
        self.restored = Some(saved.clone());
    }

    /// Brings every drawn data's slots to this frame's tiles: the tiles
    /// each terrain wants around the camera (made from a field in memory,
    /// or taken from those handed over), uploaded into free slots, the
    /// others evicted; streamed tiles fade in over their parents.
    pub(super) fn sync(
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
        // The ticks the fades step this frame: those since the last frame
        // stepped them (none on the tick already drawn, a tick back, or the
        // first frame); past [`FADE_CATCH_UP`] every tile is in.
        let elapsed = match self.fade_tick {
            Some(t) if scene.frame.tick >= t => scene.frame.tick - t,
            _ => 0,
        };
        let settle_all = elapsed > FADE_CATCH_UP;
        let fade_ticks = elapsed.min(FADE_CATCH_UP);
        if !self.settled {
            self.fade_tick = Some(scene.frame.tick);
        }
        let (sources, revisions) = scene.terrain_sources();
        self.datas.resize_with(sources.len(), || None);
        // Each data's wanted tiles, over every terrain drawing it, best first.
        let mut wanted: og_core::IdMap<usize, (Vec<u32>, u32)> = Default::default();
        for (ti, t) in scene.world.terrains.iter().enumerate() {
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
            let height_bytes =
                u64::from(layout.heights_side()) * u64::from(layout.heights_rows()) * 4;
            let height_slots = device.limits().max_storage_buffer_binding_size / height_bytes;
            let capacity = (t.lod.budget() / layout.slot_bytes().max(1))
                .max(8)
                .min((self.max_layers as u64).max(1))
                .min(height_slots)
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
                Some(mut g)
                    if let TerrainSource::Data(d, ..) = source
                        && let Some(edits) = scene.terrain_edits_since(i, g.revision) =>
                {
                    // Edited in place: only the tiles the edits reach are
                    // stale, and the pyramids only there.
                    g.revision = revisions[i];
                    let field = |c: u32, r: u32| d.heights[(r * d.columns + c) as usize];
                    for (lo, hi) in edits {
                        if let Some(p) = &mut g.height_pyramid {
                            let (a, b) = layout.samples_in(lo, hi);
                            p.update(&layout, field, a, b);
                        }
                        if let (Some(p), Some(c)) = (&mut g.pyramid, &d.control) {
                            p.update(&layout, c, lo, hi);
                        }
                        crate::terrain_tiles::edited_tiles(&layout, lo, hi, |t| {
                            if let Some(s) = g.by_tile.remove(&t) {
                                g.slots[s] = None;
                            }
                        });
                    }
                    let same_holes = match (&g.holes_of, &d.holes) {
                        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                        (None, None) => true,
                        _ => false,
                    };
                    if !same_holes {
                        g.holes = None;
                        g.holes_of = None;
                    }
                    g.streamed = streamed;
                    g
                }
                Some(mut g) => {
                    // Replaced: every slot is stale.
                    g.revision = revisions[i];
                    g.slots.iter_mut().for_each(|s| *s = None);
                    g.by_tile.clear();
                    g.pyramid = None;
                    g.holes = None;
                    g.holes_of = None;
                    g.height_pyramid = None;
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
                    TerrainSource::Data(d, ..) => match gpu.by_tile.get(&tile) {
                        Some(&s) => gpu.slots[s].as_ref().expect("held").data.clone(),
                        None => {
                            if gpu.pyramid.is_none()
                                && let Some(c) = &d.control
                            {
                                gpu.pyramid = ControlPyramid::new(&layout, c);
                            }
                            if gpu.holes.is_none() {
                                gpu.holes = d.hole_counts();
                                gpu.holes_of = d.holes.clone();
                            }
                            let heights = gpu
                                .height_pyramid
                                .get_or_insert_with(|| d.height_pyramid(&layout));
                            Arc::new(d.tile(
                                &layout,
                                tile,
                                heights,
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
                .world
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
            // The resident tiles' fades step once a tick since the last
            // frame, each once its parent has fully faded in, before this
            // frame's arrivals: a tile's fade is a function of the ticks
            // since it arrived, never of the frames drawn (docs/design.md
            // §2.4). A capture draws them settled and steps nothing.
            if !self.settled {
                let per_tick = 1.0 / (scene.frame.tick_rate.max(1e-3) * FADE_SECONDS);
                for _ in 0..fade_ticks {
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
                            // The last step's rounding is not left short of whole.
                            st.fade += per_tick;
                            if st.fade >= 1.0 - 1e-4 {
                                st.fade = 1.0;
                            }
                        }
                    }
                }
                if settle_all {
                    for st in gpu.slots.iter_mut().flatten() {
                        st.fade = 1.0;
                    }
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
                let ptr = Arc::as_ptr(data) as usize;
                let fade = self
                    .restored
                    .as_ref()
                    .and_then(|r| {
                        r.fades
                            .iter()
                            .find(|f| f.0 == i && f.1 == *tile && f.3 == ptr)
                    })
                    .map_or(fresh_fade, |f| f.2);
                gpu.slots[slot] = Some(SlotTile {
                    tile: *tile,
                    data: data.clone(),
                    fade,
                });
                gpu.by_tile.insert(*tile, slot);
                self.uploads += 1;
            }
            let mut table = vec![0.0f32; MAX_TERRAIN_TILES as usize];
            for (s, st) in gpu.slots.iter().enumerate() {
                if let Some(st) = st {
                    let fade = if self.settled { 1.0 } else { st.fade };
                    table[st.tile as usize] = (s as f32 + 1.0) * 256.0 + (fade * 255.0).round();
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
        if !self.settled {
            self.restored = None;
        }
        Ok(())
    }
}
