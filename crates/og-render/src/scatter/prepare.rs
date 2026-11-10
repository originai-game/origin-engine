use super::*;

impl ScatterRenderer {
    /// Groups the scene's model impostors by atlas, uploads their
    /// transforms and writes each group's uniform at the slots after the
    /// scatters'.
    #[allow(clippy::too_many_arguments)]
    fn prepare_impostors(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &crate::RenderScene,
        base: ScatterGpu,
        groups: &[(crate::Impostor, Vec<usize>)],
        first_slot: usize,
    ) -> Vec<ImpostorBatch> {
        let total: usize = groups.iter().map(|g| g.1.len()).sum();
        if total == 0 {
            return Vec::new();
        }
        let size = (total * std::mem::size_of::<crate::instances::GpuTransform>()) as u64;
        if self.impostor_instances.as_ref().is_none_or(|b| b.1 < size) {
            let cap = size.next_power_of_two();
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("impostor instances"),
                size: cap,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.impostor_instances = Some((buffer, cap));
        }
        let mut transforms = Vec::with_capacity(total);
        let mut out = Vec::with_capacity(groups.len());
        let light = scene.light.direction_to_light.normalize_or(Vec3::Y);
        for (k, (imp, members)) in groups.iter().enumerate() {
            let first = transforms.len() as u32;
            let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
            let mut grow = 0.0f32;
            let mut cast = false;
            for &i in members {
                let d = &scene.impostors[i];
                let c = d.transform.transform_point3(imp.center);
                let s = d
                    .transform
                    .to_scale_rotation_translation()
                    .0
                    .abs()
                    .max_element();
                lo = lo.min(c);
                hi = hi.max(c);
                grow = grow.max(imp.radius * s);
                cast |= d.cast_shadows;
                transforms.push(crate::instances::gpu_transform(&d.transform));
            }
            let slot = first_slot + k;
            let mut gpu = base;
            gpu.impostor(imp, 0.0, light, self.samples > 1);
            queue.write_buffer(&self.uniform, slot as u64 * SLOT, bytemuck::bytes_of(&gpu));
            out.push(ImpostorBatch {
                material: imp.material.index(),
                cast_shadows: cast,
                offset: (slot as u64 * SLOT) as u32,
                first,
                count: members.len() as u32,
                center: (lo + hi) * 0.5,
                radius: (hi - lo).length() * 0.5 + grow,
            });
        }
        if let Some((buffer, _)) = &self.impostor_instances {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&transforms));
        }
        for pass in [ScatterPass::Lit, ScatterPass::Prepass, ScatterPass::Shadow] {
            self.pipeline(device, None, Side::Double, pass, true);
        }
        out
    }

    /// Builds the chunks this frame needs and answers what to draw.
    /// `side_of` answers a material's side (`None`: not of this scene);
    /// `mesh_bounds` a mesh's radius about its origin, its top above it
    /// and its index count (`None`: no geometry).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &crate::RenderScene,
        camera: Vec3,
        view_proj: Mat4,
        prev_view_proj: Mat4,
        frame_index: u64,
        (wind_grid, lean_cap, trample): (crate::wind_field::WindGrid, f32, [f32; 4]),
        side_of: impl Fn(usize) -> Option<Side>,
        mesh_bounds: impl Fn(usize) -> Option<(f32, f32, u32)>,
        shader_of: impl Fn(usize) -> Option<usize>,
        hooks_of: impl Fn(usize) -> String,
        (card_inputs, cpu_cards): (&[CardInput], &[crate::skin::SkinnedInstanceGpu]),
        (terrain, terrain_draws): (
            &crate::terrain::TerrainRenderer,
            &[crate::terrain::TerrainDraw],
        ),
    ) -> ScatterBatches {
        self.frame += 1;
        self.built = 0;
        // A frame that recorded jobs and failed before encoding them: their
        // chunks are placed again.
        if let Some(place) = &mut self.place {
            if place.pending() {
                for k in &self.placed_chunks {
                    self.chunks.remove(k);
                }
            }
            place.begin();
        }
        self.placed_chunks.clear();
        let noise = crossfade_noise_offset(frame_index);
        let mut batches = Vec::new();
        let wind = scene.wind.filter(|w| w.strength > 0.0);
        let previous = self.last_tick.unwrap_or(scene.tick);
        self.last_tick = Some(scene.tick);
        queue.write_buffer(
            &self.wind_uniform,
            0,
            bytemuck::bytes_of(&crate::wind::field_gpu(
                wind,
                scene.tick,
                previous,
                scene.tick_rate,
                scene.origin,
                wind_grid,
            )),
        );
        // How long trampled copies take to stand again: the scene's first
        // grass field's (`trample.rs`), else its default.
        let recovery = scene
            .grass
            .first()
            .map_or(crate::Grass::default().recovery, |g| g.grass.recovery);
        // `lean_cap`: the most a copy leans this frame (wind_field.rs; its
        // chunks' bounds widen by its tip's reach).
        // The model impostors by atlas, in the order each first appears.
        let mut groups: Vec<(crate::Impostor, Vec<usize>)> = Vec::new();
        for (i, d) in scene.impostors.iter().enumerate() {
            match groups.iter_mut().find(|g| g.0 == d.impostor) {
                Some(g) => g.1.push(i),
                None => groups.push((d.impostor, vec![i])),
            }
        }
        let slots = (scene.scatters.len() + groups.len() + card_inputs.len()) as u64;
        if self.slots < slots {
            self.slots = slots.next_power_of_two();
            let (u, g) = Self::slots(
                device,
                &self.uniform_layout,
                [&self.blue_noise, &self.wind_uniform],
                [&self.wind_field, &self.trample],
                self.slots,
            );
            self.uniform = u;
            self.group = g;
        }
        // A model impostor is drawn at any distance (the level pick
        // decides), never faded or swayed.
        let base = ScatterGpu {
            camera: camera.extend(f32::MAX).to_array(),
            fade: [f32::MAX, noise, 0.0, 0.0],
            view_proj: view_proj.to_cols_array_2d(),
            prev_view_proj: prev_view_proj.to_cols_array_2d(),
            wind: [0.0; 4],
            wind_shape: [0.0, 0.0, scene.origin.x, scene.origin.z],
            impostor: [0.0; 4],
            impostor_grid: [0.0; 4],
            impostor_light: [0.0; 4],
            trample: [0.0; 4],
        };
        let impostors =
            self.prepare_impostors(device, queue, scene, base, &groups, scene.scatters.len());
        let light = scene.light.direction_to_light.normalize_or(Vec3::Y);
        let cards = self.prepare_cards(
            device,
            queue,
            base,
            light,
            card_inputs,
            cpu_cards,
            scene.scatters.len() + groups.len(),
        );
        self.culled_chunks.clear();
        let mut cull_items = Vec::new();
        // Per batch, per chunk, its item among `cull_items`.
        let mut planned: Vec<Vec<Option<usize>>> = Vec::new();
        let view = crate::frame::Frustum::from_view_proj(view_proj);
        if scene.scatters.is_empty() {
            if let Some(cull) = &mut self.cull {
                cull.prepare(device, queue, &[]);
            }
            self.chunks.retain(|_, c| self.frame - c.used < KEEP_FRAMES);
            self.shared.clear();
            return ScatterBatches {
                scatters: batches,
                impostors,
                cards,
            };
        }
        let (sources, revisions) = scene.terrain_sources();
        let mut budget = CHUNKS_PER_FRAME;
        let mut gpu_budget = GPU_CHUNKS_PER_FRAME;
        self.shared.resize(scene.scatters.len(), None);
        // Each scatter's terrain draw and parameters: what places before it.
        let placing: Vec<(usize, TerrainScatter)> = scene
            .scatters
            .iter()
            .map(|d| (d.terrain, d.scatter))
            .collect();
        for (si, d) in scene.scatters.iter().enumerate() {
            let Some(t) = scene.terrains.get(d.terrain) else {
                continue;
            };
            let di = t.terrain.index();
            let (Some(source), Some(side), Some((mesh_r, mesh_top, index_count))) = (
                sources.get(di),
                side_of(d.material.index()),
                mesh_bounds(d.mesh.index()),
            ) else {
                continue;
            };
            let s = d.scatter;
            if !s.usable(t.layers.len()) {
                continue;
            }
            let layout = match source.layout() {
                Ok(l) => l,
                Err(_) => continue,
            };
            let model = t.transform;
            let along = horizontal_scale(model);
            let local = model.inverse().transform_point3(camera);
            let eye = Vec2::new(local.x, local.z);
            let reach = s.distance / along;
            let cell = s.cell();
            let span = cell * CHUNK_CELLS as f32;
            let half = layout.size * 0.5;
            let cells = (layout.size / cell).ceil();
            let (nx, nz) = (
                (cells.x as u32).div_ceil(CHUNK_CELLS),
                (cells.y as u32).div_ceil(CHUNK_CELLS),
            );
            let lo = ((eye - reach + half) / span).floor().max(Vec2::ZERO);
            let hi = ((eye + reach + half) / span).ceil();
            let (x0, z0) = (lo.x as u32, lo.y as u32);
            let (x1, z1) = (
                (hi.x.max(0.0) as u32).min(nx),
                (hi.y.max(0.0) as u32).min(nz),
            );
            let ctx = ScatterContext::of(
                &placing,
                si,
                t.layers.len(),
                scene.water.iter().map(|w| {
                    let mut m = w.transform;
                    m.w_axis += scene.origin.extend(0.0);
                    (m, w.size, w.endless)
                }),
            );
            let mut params: Vec<u32> = std::iter::once(&s)
                .chain(&ctx.before)
                .flat_map(|o| o.placement_bits())
                .collect();
            if ctx.reads_water(&s) {
                for (m, half) in &ctx.water {
                    params.extend(m.to_cols_array().map(f32::to_bits));
                    params.extend(half.to_array().map(f32::to_bits));
                }
            }
            // The copies are placed in the world (the terrain's placement
            // plus the scene's origin: exact, as the origin is on a coarse
            // grid) and then moved by the origin, so each copy's place, and
            // the wind's phase hashed from its bits, is the same bits
            // wherever the origin is (`origin.rs`).
            let world = scene.origin != Vec3::ZERO;
            let placed_on = if world {
                let mut w = t.clone();
                w.transform.w_axis += scene.origin.extend(0.0);
                std::borrow::Cow::Owned(w)
            } else {
                std::borrow::Cow::Borrowed(t)
            };
            let transform = placed_on.transform.to_cols_array().map(f32::to_bits);
            let origin = scene.origin.to_array().map(f32::to_bits);
            let layers: Vec<u32> = t
                .layers
                .iter()
                .flat_map(|l| {
                    [
                        l.slope[0],
                        l.slope[1],
                        l.slope_fade,
                        l.altitude[0],
                        l.altitude[1],
                        l.altitude_fade,
                    ]
                })
                .map(f32::to_bits)
                .collect();
            // Its chunks' shared key: the last frame's while it is the same.
            let scatter_key = ScatterKey {
                data: di,
                revision: revisions[di],
                params,
                transform,
                origin,
                layers,
                mesh: d.mesh.index(),
            };
            let kept = self.shared[si]
                .as_ref()
                .filter(|k| *k.key == scatter_key)
                .cloned();
            let shared = match kept {
                Some(k) => k,
                None => {
                    let k = SharedKey::new(scatter_key);
                    self.shared[si] = Some(k.clone());
                    k
                }
            };
            let settle = !t.fade_in;
            let streamed = matches!(source, TerrainSource::Stream { .. });
            let tiles = match source {
                TerrainSource::Stream { tiles, .. } => Some(tiles),
                TerrainSource::Data(..) => None,
            };
            // Placed on the GPU (`TerrainScatter::gpu`) where compute runs
            // and the terrain is drawn: its terrain draw, the waters from
            // the frame and the scatters placed before it, then its own.
            let gpu = self
                .place
                .as_ref()
                .filter(|_| s.gpu)
                .and_then(|_| terrain_draws.iter().position(|td| td.terrain == d.terrain))
                .and_then(|k| {
                    let arrayed = terrain_draws[k].arrayed;
                    Some((k, arrayed, terrain.group_layout(arrayed)?))
                });
            let frame_waters: Vec<(Mat4, Vec2)> = match gpu {
                Some(_) => scene
                    .water
                    .iter()
                    .map(|w| {
                        let half = if w.endless {
                            Vec2::INFINITY
                        } else {
                            w.size * 0.5
                        };
                        (w.transform.inverse(), half)
                    })
                    .collect(),
                None => Vec::new(),
            };
            let chain: Vec<&TerrainScatter> = ctx.before.iter().chain([&s]).collect();
            let mut keys = Vec::new();
            // Nearest chunks first, so a budgeted frame builds those.
            let mut order: Vec<(u32, u32)> = (z0..z1)
                .flat_map(|z| (x0..x1).map(move |x| (x, z)))
                .collect();
            let centre = |(x, z): (u32, u32)| {
                (Vec2::new(x as f32 + 0.5, z as f32 + 0.5) * span - half - eye).length()
            };
            og_core::sort::sort_by(&mut order, |a, b| {
                centre(*a).total_cmp(&centre(*b)).then(a.cmp(b))
            });
            for (cx, cz) in order {
                // The chunk's rectangle in the terrain's space: past the reach, nothing.
                let a = Vec2::new(cx as f32, cz as f32) * span - half;
                let b = a + span;
                if (eye.clamp(a, b) - eye).length() > reach {
                    continue;
                }
                // The GPU's job for it (`None`: the CPU places it), which
                // reads the level-0 tiles the GPU holds under its chain.
                let plan = gpu.and_then(|_| {
                    let (x, z) = (cx * CHUNK_CELLS, cz * CHUNK_CELLS);
                    crate::scatter_place::plan(&crate::scatter_place::PlaceInput {
                        model: t.transform,
                        size: layout.size,
                        layers: chain.clone(),
                        cells: (
                            x,
                            z,
                            CHUNK_CELLS.min(cells.x as u32 - x),
                            CHUNK_CELLS.min(cells.y as u32 - z),
                        ),
                        waters: &frame_waters,
                        index_count,
                    })
                });
                // The GPU places it once it holds the level-0 tiles under
                // the job's reach; until then (a sparse scatter's huge
                // chunk reaching past the tiles the terrain wants) the CPU
                // does, so no chunk in range is missing.
                let gpu_under = plan.as_ref().and_then(|p| {
                    terrain.level0_slots(di, &level0_under(&layout, p.reads.0, p.reads.1))
                });
                let plan = plan.filter(|_| gpu_under.is_some());
                let cpu_under: Option<Vec<usize>> = match tiles {
                    None => Some(Vec::new()),
                    Some(tiles) => level0_under(&layout, a, b)
                        .into_iter()
                        .map(|i| tiles.get(&i).map(|t| Arc::as_ptr(t) as usize))
                        .collect(),
                };
                let key_of = |under: Vec<usize>, gpu: bool| ChunkKey {
                    scatter: shared.clone(),
                    under,
                    chunk: (cx, cz),
                    gpu,
                };
                let gpu_key = gpu_under.map(|u| key_of(u, true));
                let cpu_key = cpu_under.map(|u| key_of(u, false));
                // A chunk already placed either way is kept: the CPU's and
                // the GPU's copies are the same.
                let cached = [&gpu_key, &cpu_key]
                    .into_iter()
                    .flatten()
                    .find(|k| self.chunks.contains_key(*k))
                    .cloned();
                let Some(key) = cached.or(gpu_key).or(cpu_key) else {
                    continue;
                };
                if let Some(c) = self.chunks.get_mut(&key) {
                    c.used = self.frame;
                    // A placed chunk's draw takes the mesh's index count.
                    if let (Some(p), Some(b)) = (&mut c.placed, &c.buffer)
                        && p.index_count != index_count
                    {
                        p.index_count = index_count;
                        queue.write_buffer(
                            b,
                            crate::scatter_place::TAIL,
                            bytemuck::bytes_of(&index_count),
                        );
                    }
                    keys.push(key);
                    continue;
                }
                if let (Some(plan), Some((k, arrayed, terrain_layout))) = (plan, gpu) {
                    if streamed && !settle && gpu_budget == 0 {
                        continue;
                    }
                    gpu_budget = gpu_budget.saturating_sub(1);
                    self.built += 1;
                    let place = self.place.as_mut().expect("planned with it");
                    let buffer = crate::scatter_place::ScatterPlace::chunk_buffer(device);
                    let group = place.chunk_group(device, &buffer);
                    place.push(device, terrain_layout, (k, arrayed), plan);
                    // Its bounds: its rectangle over the data's height range.
                    let (h0, h1) = source.range();
                    let (center, radius) = crate::terrain::world_sphere(
                        t.transform,
                        Vec3::new(a.x, h0, a.y),
                        Vec3::new(b.x.min(half.x), h1, b.y.min(half.y)),
                    );
                    let grow = mesh_r * s.scale[1].max(s.scale[0]);
                    let cull_group = self.cull.as_ref().map(|c| c.chunk_group(device, &buffer));
                    self.placed_chunks.push(key.clone());
                    self.chunks.insert(
                        key.clone(),
                        Chunk {
                            buffer: Some(buffer),
                            cull_group,
                            count: 0,
                            placed: Some(Placed { group, index_count }),
                            center,
                            radius: radius + grow,
                            used: self.frame,
                        },
                    );
                    keys.push(key);
                    continue;
                }
                if streamed && !settle && budget == 0 {
                    continue;
                }
                let mut placed = Vec::new();
                let mut complete = true;
                let tile_ground;
                let ground: &dyn Ground = match source {
                    TerrainSource::Data(d, ..) => d,
                    TerrainSource::Stream { tiles, .. } => {
                        tile_ground = TileGround { layout, tiles };
                        &tile_ground
                    }
                };
                'cells: for z in cz * CHUNK_CELLS..((cz + 1) * CHUNK_CELLS).min(cells.y as u32) {
                    for x in cx * CHUNK_CELLS..((cx + 1) * CHUNK_CELLS).min(cells.x as u32) {
                        match placed_on.copy_ruled(ground, &s, &ctx.before, &ctx.water, x, z) {
                            None => {
                                complete = false;
                                break 'cells;
                            }
                            Some(Some((_, mut m, _))) => {
                                if world {
                                    m.w_axis -= scene.origin.extend(0.0);
                                }
                                placed.push(m)
                            }
                            Some(None) => {}
                        }
                    }
                }
                if !complete {
                    continue;
                }
                budget = budget.saturating_sub(1);
                self.built += 1;
                let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
                for m in &placed {
                    let p = m.w_axis.truncate();
                    lo = lo.min(p);
                    hi = hi.max(p);
                }
                let grow = mesh_r * s.scale[1].max(s.scale[0]);
                let (center, radius) = if placed.is_empty() {
                    (Vec3::ZERO, 0.0)
                } else {
                    ((lo + hi) * 0.5, (hi - lo).length() * 0.5 + grow)
                };
                let transforms: Vec<crate::instances::GpuTransform> =
                    placed.iter().map(crate::instances::gpu_transform).collect();
                let buffer = (!transforms.is_empty()).then(|| {
                    crate::buffer_init(
                        device,
                        queue,
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("scatter chunk"),
                            contents: bytemuck::cast_slice(&transforms),
                            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
                        },
                    )
                });
                let cull_group = self
                    .cull
                    .as_ref()
                    .zip(buffer.as_ref())
                    .map(|(cull, b)| cull.chunk_group(device, b));
                self.chunks.insert(
                    key.clone(),
                    Chunk {
                        buffer,
                        cull_group,
                        count: transforms.len() as u32,
                        placed: None,
                        center,
                        radius,
                        used: self.frame,
                    },
                );
                keys.push(key);
            }
            // The wind: its lean on this scatter, and the furthest a copy
            // moves (the tip of the tallest at the top of a gust).
            // A trample map parts the copies by up to 1.2 of their height
            // (scatter_common.wgsl scatter_sway), wind or not.
            let parting = if trample[3] > 0.0 { 1.2 } else { 0.0 };
            let moves = lean_cap > 0.0 || parting > 0.0;
            let (wind_row, sway) = if s.sway > 0.0 && mesh_top > 0.0 && moves {
                let d = wind.map_or(Vec3::X, |w| w.direction);
                let dir = Vec2::new(d.x, d.z).normalize_or(Vec2::X);
                let tip = (lean_cap * s.sway + parting) * mesh_top * s.scale[1].max(s.scale[0]);
                ([dir.x, dir.y, s.sway, recovery], tip)
            } else {
                ([0.0; 4], 0.0)
            };
            let mut gpu = ScatterGpu {
                camera: camera.extend(s.distance).to_array(),
                fade: [s.fade_from(), noise, 0.0, 0.0],
                view_proj: view_proj.to_cols_array_2d(),
                prev_view_proj: prev_view_proj.to_cols_array_2d(),
                wind: wind_row,
                wind_shape: [0.0, mesh_top, scene.origin.x, scene.origin.z],
                impostor: [0.0; 4],
                impostor_grid: [0.0; 4],
                impostor_light: [0.0; 4],
                trample,
            };
            let impostor = d.impostor.filter(|_| d.impostor_distance > 0.0);
            if let Some(i) = &impostor {
                gpu.impostor(i, d.impostor_distance, light, self.samples > 1);
                for pass in [ScatterPass::Lit, ScatterPass::Prepass, ScatterPass::Shadow] {
                    self.pipeline(device, None, Side::Double, pass, true);
                }
            }
            queue.write_buffer(&self.uniform, si as u64 * SLOT, bytemuck::bytes_of(&gpu));
            let shader = shader_of(d.material.index());
            if let Some(sh) = shader
                && !self.hooked.contains_key(&sh)
            {
                let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("scatter (material shader)"),
                    source: wgpu::ShaderSource::Wgsl((self.lit_wgsl)(&hooks_of(sh)).into()),
                });
                self.hooked.insert(sh, module);
            }
            for pass in [ScatterPass::Lit, ScatterPass::Prepass, ScatterPass::Shadow] {
                self.pipeline(device, shader, side, pass, false);
            }
            // The chunks the camera may see, culled copy by copy on the GPU.
            let culled = keys
                .iter()
                .map(|k| {
                    let c = self.chunks.get(k)?;
                    c.cull_group.as_ref()?;
                    if !view.intersects_sphere(c.center, c.radius + sway) {
                        return None;
                    }
                    self.culled_chunks.push(k.clone());
                    cull_items.push(crate::scatter_cull::CullItem {
                        to_terrain: t.transform.inverse(),
                        extent: layout.size,
                        copies: c.count,
                        placed: c.placed.is_some(),
                        index_count,
                        radius: mesh_r,
                        moves: sway,
                        distance: s.distance,
                        density_map: s.density_map,
                    });
                    Some(cull_items.len() - 1)
                })
                .collect::<Vec<_>>();
            planned.push(culled);
            batches.push(ScatterBatch {
                culled: Vec::new(),
                mesh: d.mesh.index(),
                material: d.material.index(),
                shader,
                cast_shadows: d.cast_shadows,
                side,
                offset: (si as u64 * SLOT) as u32,
                chunks: keys,
                sway,
                impostor: impostor.map(|i| i.material.index()),
            });
        }
        self.chunks.retain(|_, c| self.frame - c.used < KEEP_FRAMES);
        if let Some(place) = &mut self.place {
            place.upload(device, queue);
        }
        // Where each culled chunk draws from, in the order planned.
        if let Some(cull) = &mut self.cull {
            let places = cull.prepare(device, queue, &cull_items);
            for (b, items) in batches.iter_mut().zip(planned) {
                b.culled = items.into_iter().map(|i| i.map(|i| places[i])).collect();
            }
        }
        ScatterBatches {
            scatters: batches,
            impostors,
            cards,
        }
    }

    /// The crowds' cards: each its uniform at the slots from `first_slot`
    /// (its sphere, views, frames and elevation), the CPU path's records
    /// uploaded, and their pipelines made.
    #[allow(clippy::too_many_arguments)]
    fn prepare_cards(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        base: ScatterGpu,
        light: Vec3,
        inputs: &[CardInput],
        cpu: &[crate::skin::SkinnedInstanceGpu],
        first_slot: usize,
    ) -> Vec<CardBatch> {
        if inputs.is_empty() {
            return Vec::new();
        }
        if !cpu.is_empty() {
            let size = std::mem::size_of_val(cpu) as u64;
            if self.card_instances.as_ref().is_none_or(|b| b.1 < size) {
                let cap = size.next_power_of_two();
                let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("crowd card records"),
                    size: cap,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                self.card_instances = Some((buffer, cap));
            }
            let (buffer, _) = self.card_instances.as_ref().expect("made above");
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(cpu));
        }
        let out = inputs
            .iter()
            .enumerate()
            .map(|(k, c)| {
                let slot = first_slot + k;
                let mut gpu = base;
                gpu.impostor = c.center.extend(c.radius).to_array();
                gpu.impostor_grid = [0.0, 0.0, f32::from(u8::from(self.samples > 1)), 0.0];
                gpu.fade[2] = c.views as f32;
                gpu.fade[3] = c.frames as f32;
                gpu.impostor_light = light.extend(c.elevation).to_array();
                queue.write_buffer(&self.uniform, slot as u64 * SLOT, bytemuck::bytes_of(&gpu));
                CardBatch {
                    input: *c,
                    offset: (slot as u64 * SLOT) as u32,
                    group: None,
                }
            })
            .collect();
        for pass in [ScatterPass::Lit, ScatterPass::Prepass] {
            let kind = pass as u8 + CARD_KIND;
            if !self.pipelines.contains_key(&(None, Side::Double, kind)) {
                let p = self.card_pipeline(device, pass, crate::ShadowTarget::Cascade);
                self.pipelines.insert((None, Side::Double, kind), p);
            }
        }
        let key = (None, Side::Double, ScatterPass::Shadow as u8 + CARD_KIND);
        if !self.shadow_pipelines.contains_key(&key) {
            let set = crate::ShadowSet::new(|t| self.card_pipeline(device, ScatterPass::Shadow, t));
            self.shadow_pipelines.insert(key, set);
        }
        out
    }
}
