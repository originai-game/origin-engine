use super::*;

/// Whether a system's sub-emitters can simulate entirely on the GPU in the
/// Mega-Pool (falls back to CPU only when a child uses `rate_over_time` or
/// more than 2 bursts / >16 cycles on `SubTrigger::Life`, or sprite mode).
fn gpu_subs_ok(e: &ParticleEmitter, subs: &[Option<&ParticleEmitter>]) -> bool {
    e.sub_emitters.iter().zip(subs).all(|(sub, ce)| {
        let Some(ce) = ce else {
            return false;
        };
        if matches!(ce.render, ParticleRender::Sprite { .. }) || ce.trail.is_some() {
            return false;
        }
        if sub.trigger == crate::particles::SubTrigger::Life
            && (ce.rate_over_time.is_some()
                || ce.bursts.len() > 2
                || ce.bursts.iter().any(|b| b.cycles > 16))
        {
            return false;
        }
        true
    })
}

/// Releases every slab owned by a retired or reallocated [`GpuSystem`].
fn release_gpu_system(pool: &mut ParticlePool, g: GpuSystem, max: u32) {
    pool.particle_ranges.release(g.particle_base, max);
    if g.trail_max > 0 {
        pool.trail_ranges.release(g.trail_base, g.trail_max);
    }
    for c in g.children {
        pool.particle_ranges.release(c.particle_base, c.max);
        pool.counter_ranges.release(c.counter_idx, 1);
    }
}

/// Ensures `emitter_idx`'s surface/spline points and grid field live in
/// `points` + `pool.points_shadow`, returning `(shape_len, pool_offset)`.
fn ensure_points(
    points: &mut HashMap<usize, PointsTable>,
    pool: &mut ParticlePool,
    scene: &RenderScene,
    emitter_idx: usize,
    e: &ParticleEmitter,
) -> Result<(usize, u32), RenderError> {
    let fresh = match points.get(&emitter_idx) {
        Some(t) => t.shape != e.shape || t.field != e.field,
        None => true,
    };
    if fresh {
        if let Some(old) = points.remove(&emitter_idx) {
            pool.points_ranges
                .release(old.pool_offset, old.table.len() as u32);
        }
        let mesh = match e.shape {
            EmitterShape::Mesh { mesh } => Some(scene.mesh(mesh).ok_or_else(|| {
                RenderError::InvalidHandle(format!("emitter {emitter_idx}: mesh {}", mesh.index()))
            })?),
            _ => None,
        };
        let mut table = crate::particles::shape_points(&e.shape, mesh);
        let shape_len = table.len();
        if let Some(crate::particles::VectorField::Grid { vectors, .. }) = &e.field {
            table.extend(vectors.iter().map(|v| v.extend(0.0).to_array()));
        }
        let pool_offset = pool.points_ranges.alloc(table.len() as u32);
        let end = pool_offset as usize + table.len();
        if pool.points_shadow.len() < end {
            pool.points_shadow.resize(end, [0.0; 4]);
        }
        pool.points_shadow[pool_offset as usize..end].copy_from_slice(&table);
        pool.points_dirty = true;
        points.insert(
            emitter_idx,
            PointsTable {
                shape: e.shape,
                field: e.field.clone(),
                table,
                shape_len,
                pool_offset,
            },
        );
    }
    let pt = &points[&emitter_idx];
    Ok((pt.shape_len, pt.pool_offset))
}

/// Appends 64-slot tiles for one simulation entry into `out`.
fn push_sim_tiles(out: &mut Vec<[u32; 4]>, entry_idx: u32, max: u32) {
    let mut off = 0u32;
    while off < max {
        out.push([entry_idx, off, 0, 0]);
        off += 64;
    }
}

struct CompactPlanner<'a> {
    tiles: &'a mut Vec<CompactTileGpu>,
    indirect: &'a mut Vec<[u32; 8]>,
    particle_slots: u32,
    trail_slots: u32,
}

impl CompactPlanner<'_> {
    /// Plans compaction for one GPU system (and optionally its paired trail
    /// when `e.trail.is_some_and(|t| t.head)`), returning `(head, trail)`.
    #[allow(clippy::too_many_arguments)]
    fn plan(
        &mut self,
        src_base: u32,
        trail_src_base: Option<u32>,
        slots: &[std::ops::Range<u32>; 2],
        kind: DrawKind,
        mesh_index_count: u32,
        pair_trail: bool,
    ) -> (CompactDraw, Option<CompactDraw>) {
        let total_scan: u32 = slots.iter().map(|r| r.end.saturating_sub(r.start)).sum();
        let dst_base = self.particle_slots;
        self.particle_slots += total_scan;
        let (t_src, t_dst) = match trail_src_base {
            Some(sb) => {
                let db = self.trail_slots;
                self.trail_slots += total_scan;
                (sb, db)
            }
            None => (u32::MAX, 0),
        };
        let head_count = match kind {
            DrawKind::Mesh => mesh_index_count,
            DrawKind::Trail => TRAIL_POINTS as u32 * 6,
            _ => 6,
        };
        let compact_idx = self.indirect.len() as u32;
        self.indirect.push([head_count, 0, 0, 0, 0, 0, 0, 0]);
        let compact_idx2 = if pair_trail {
            let idx = self.indirect.len() as u32;
            self.indirect
                .push([TRAIL_POINTS as u32 * 6, 0, 0, 0, 0, 0, 0, 0]);
            idx
        } else {
            u32::MAX
        };
        let first_tile = self.tiles.len();
        for r in slots.iter().filter(|r| !r.is_empty()) {
            let len = r.end - r.start;
            let mut off = 0u32;
            while off < len {
                self.tiles.push(CompactTileGpu {
                    src_base: src_base + r.start,
                    dst_base,
                    trail_src_base: if t_src == u32::MAX {
                        u32::MAX
                    } else {
                        t_src + r.start
                    },
                    trail_dst_base: t_dst,
                    slot_offset: off,
                    count: len,
                    compact_idx,
                    compact_idx2,
                    first_tile: first_tile as u32,
                    end_tile: 0,
                    live_count: 0,
                    live_offset: 0,
                });
                off += 64;
            }
        }
        let end_tile = self.tiles.len() as u32;
        for tile in &mut self.tiles[first_tile..] {
            tile.end_tile = end_tile;
        }
        let head = CompactDraw {
            particles_offset: u64::from(dst_base) * PARTICLE_STRIDE,
            trails_offset: u64::from(t_dst) * TRAIL_POINTS as u64 * 16,
            indirect_offset: u64::from(compact_idx) * INDIRECT_STRIDE,
            index_count: if kind == DrawKind::Mesh {
                mesh_index_count
            } else {
                0
            },
        };
        let trail = pair_trail.then_some(CompactDraw {
            particles_offset: head.particles_offset,
            trails_offset: head.trails_offset,
            indirect_offset: u64::from(compact_idx2) * INDIRECT_STRIDE,
            index_count: 0,
        });
        (head, trail)
    }
}

const FULL_UV: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

/// Returns the tight non-transparent UV box for a plain single-frame textured
/// billboard emitter without effect materials (100% pixel-identical cutout).
fn billboard_crop(e: &ParticleEmitter, texture_bounds: &impl Fn(usize) -> [f32; 4]) -> [f32; 4] {
    if !matches!(e.render, ParticleRender::Billboard)
        || e.effect.is_some()
        || e.sheet.mode != SheetMode::Off
        || e.sheet.columns > 1
        || e.sheet.rows > 1
    {
        return FULL_UV;
    }
    e.texture.map_or(FULL_UV, |t| texture_bounds(t.index()))
}

/// Whether a 3D billboard system qualifies for the half-resolution soft pass
/// (4x fillrate reduction on large soft clouds/fireballs with bilateral upsample).
fn qualifies_half_res(
    scene: &RenderScene,
    kind: DrawKind,
    blend: ParticleBlend,
    look: Look,
    soft: f32,
    oit_order_free: bool,
) -> bool {
    scene.frame.debug == crate::DebugView::None
        && scene.frame.quality.soft_particles
        && kind == DrawKind::Billboard
        && look.hook.is_none()
        && soft >= 0.25
        && (blend == ParticleBlend::Additive || (blend == ParticleBlend::Alpha && !oit_order_free))
}

impl ParticleRenderer {
    /// Advances every system of the scene, plans batched GPU simulation waves,
    /// alive compaction + indirect draw args, and writes the lean draw uniforms.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RenderScene,
        gpu: bool,
        indirect: bool,
        oit_order_free: bool,
        texture_count: usize,
        texture_bounds: impl Fn(usize) -> [f32; 4],
        eye: Vec3,
        forward: Vec3,
    ) -> Result<ParticleFrame, RenderError> {
        // Complete any deferred pool grow copies from an aborted previous frame.
        self.pool.flush_pending_copies(device, queue);
        self.frame += 1;
        if self.scene_id != scene.id() {
            self.scene_id = scene.id();
            self.systems.clear();
            self.points.clear();
            self.pool.reset_allocations();
        }
        let tick_rate = scene.frame.tick_rate.max(1e-3);
        let mut out = ParticleFrame::default();
        let mut draw_entries = std::mem::take(&mut self.draw_entries);
        draw_entries.clear();
        self.cpu_bytes.clear();
        self.pool.sim_entries_cpu.clear();
        self.pool.sim_tiles_cpu.clear();
        self.pool.sim_dispatch_bytes.clear();
        self.pool.compact_tiles_cpu.clear();
        self.pool.indirect_init_cpu.clear();

        // Compute waves accumulated across all GPU systems this frame.
        let mut shift_list: Vec<(u32, u32)> = Vec::new();
        let mut clear_list: Vec<(u32, u32)> = Vec::new();
        let mut wave_children: Vec<Vec<(u32, u32)>> = Vec::new();
        let mut wave_parents: Vec<Vec<(u32, u32)>> = Vec::new();
        // Sub-emitter descriptors packed after all wave tiles; entries with
        // sub-emitters get patched once `sub_base_offset` is known.
        let mut sub_descs_cpu: Vec<[u32; 4]> = Vec::new();
        let mut sub_patch: Vec<u32> = Vec::new();
        // Compute entries meeting the scene's colliders: their sim_entries range + reach.
        let mut colliding: Vec<(std::ops::Range<u32>, (Vec3, Vec3))> = Vec::new();
        let mut compact_particle_slots = 0u32;
        let mut compact_trail_slots = 0u32;

        for (i, sys) in scene.effects.particles.iter().enumerate() {
            let layers = crate::views::layer_of(&scene.effects.particle_layers, i);
            let history = scene.effects.particle_history.get(&sys.id).map(|h| &**h);
            let e = scene.emitters().get(sys.emitter.index()).ok_or_else(|| {
                RenderError::InvalidHandle(format!(
                    "particle system {i}: emitter {}",
                    sys.emitter.index()
                ))
            })?;
            e.validate().map_err(|m| {
                RenderError::InvalidData(format!("emitter {}: {m}", sys.emitter.index()))
            })?;
            let emission_scale =
                sys.emission.clamp(0.0, 1.0) * e.thinned(scene.frame.quality.particle_emission);
            if let Some(t) = e.texture
                && t.index() >= texture_count
            {
                return Err(RenderError::InvalidHandle(format!(
                    "emitter {}: texture {}",
                    sys.emitter.index(),
                    t.index()
                )));
            }
            let (shape_len, points_base) = ensure_points(
                &mut self.points,
                &mut self.pool,
                scene,
                sys.emitter.index(),
                e,
            )?;
            let grid_base = points_base + shape_len as u32;

            // Validate sub-emitters and meshes ahead of state mutation.
            let subs: Vec<Option<&ParticleEmitter>> = e
                .sub_emitters
                .iter()
                .map(|s| scene.emitters().get(s.emitter.index()))
                .collect();
            let mesh_index_count = match e.render {
                ParticleRender::Mesh { mesh } => {
                    let data = scene.mesh(mesh).ok_or_else(|| {
                        RenderError::InvalidHandle(format!(
                            "emitter {}: mesh {}",
                            sys.emitter.index(),
                            mesh.index()
                        ))
                    })?;
                    out.meshes.push(mesh.index());
                    if data.vertices.is_empty() {
                        0
                    } else {
                        data.indices.len() as u32
                    }
                }
                _ => 0,
            };
            let mut child_mesh_counts: Vec<u32> = Vec::with_capacity(subs.len());
            let mut child_grid_bases: Vec<(u32, u32)> = Vec::with_capacity(subs.len());
            for (k, sub) in subs.iter().enumerate() {
                let Some(ce) = sub else {
                    return Err(RenderError::InvalidHandle(format!(
                        "emitter {}: sub-emitter {k}: emitter {}",
                        sys.emitter.index(),
                        e.sub_emitters[k].emitter.index()
                    )));
                };
                ce.validate().map_err(|m| {
                    RenderError::InvalidData(format!(
                        "emitter {}: sub-emitter {k}: {m}",
                        sys.emitter.index()
                    ))
                })?;
                if ce.texture.is_some_and(|t| t.index() >= texture_count) {
                    return Err(RenderError::InvalidHandle(format!(
                        "emitter {}: sub-emitter {k}: texture",
                        sys.emitter.index()
                    )));
                }
                let cmc = match ce.render {
                    ParticleRender::Mesh { mesh } => {
                        let data = scene.mesh(mesh).ok_or_else(|| {
                            RenderError::InvalidHandle(format!(
                                "emitter {}: sub-emitter {k}: mesh {}",
                                sys.emitter.index(),
                                mesh.index()
                            ))
                        })?;
                        out.meshes.push(mesh.index());
                        if data.vertices.is_empty() {
                            0
                        } else {
                            data.indices.len() as u32
                        }
                    }
                    _ => 0,
                };
                child_mesh_counts.push(cmc);
                let c_idx = e.sub_emitters[k].emitter.index();
                if ce.field.is_some() {
                    let (c_slen, c_pbase) =
                        ensure_points(&mut self.points, &mut self.pool, scene, c_idx, ce)?;
                    child_grid_bases.push((c_pbase, c_pbase + c_slen as u32));
                } else {
                    child_grid_bases.push((0, 0));
                }
            }

            let state = self.systems.entry(sys.id).or_insert_with(|| SystemState {
                emitter: sys.emitter.index(),
                max: e.max_particles,
                gpu: None,
                cpu: CpuParticles::default(),
                seen: 0,
                shift: Vec3::ZERO,
                path: Default::default(),
            });
            let trail_need = if e.trail.is_some() {
                e.max_particles
            } else {
                0
            };
            let stale_gpu = state.emitter != sys.emitter.index()
                || state.max != e.max_particles
                || state
                    .gpu
                    .as_ref()
                    .is_some_and(|g| g.trail_max != trail_need);
            if stale_gpu {
                if let Some(old_gpu) = state.gpu.take() {
                    release_gpu_system(&mut self.pool, old_gpu, state.max);
                }
                *state = SystemState {
                    emitter: sys.emitter.index(),
                    max: e.max_particles,
                    gpu: None,
                    cpu: CpuParticles::default(),
                    seen: 0,
                    shift: Vec3::ZERO,
                    path: Default::default(),
                };
            }
            state.seen = self.frame;
            out.capacity += e.max_particles;
            let pending = std::mem::take(&mut state.shift);
            let (moved, origin) = match e.space {
                SimulationSpace::World => (pending, scene.frame.origin),
                SimulationSpace::Local => (Vec3::ZERO, Vec3::ZERO),
            };
            state.cpu.origin = origin;
            let field_origin = e.field.as_ref().map_or([[0.0; 4]; 2], |f| {
                let (whole, rest) = crate::particles::field_origin(f, origin);
                [
                    [whole[0] as f32, whole[1] as f32, whole[2] as f32, 0.0],
                    rest.extend(0.0).to_array(),
                ]
            });
            let noise_origin = {
                let (whole, rest) = crate::particles::noise_origin(origin, e.noise.frequency);
                [
                    [whole[0] as f32, whole[1] as f32, whole[2] as f32, 0.0],
                    rest.extend(0.0).to_array(),
                ]
            };
            if moved != Vec3::ZERO {
                for p in &mut state.cpu.slots {
                    for (c, d) in p.pos_age.iter_mut().zip(moved.to_array()) {
                        *c += d;
                    }
                }
                for p in &mut state.cpu.trail {
                    for (c, d) in p.iter_mut().zip(moved.to_array()) {
                        *c += d;
                    }
                }
            }
            if pending != Vec3::ZERO {
                for c in &mut state.cpu.children {
                    for p in &mut c.slots {
                        for (x, d) in p.pos_age.iter_mut().zip(pending.to_array()) {
                            *x += d;
                        }
                    }
                }
            }
            let seed = hash_seed(sys.seed);
            let interval = e.trail.map(|t| t.interval(tick_rate));
            let pull = if e.force_fields != 0.0 && e.space == SimulationSpace::World {
                e.force_fields.abs() * fields_peak(&scene.world.force_fields)
            } else {
                0.0
            };
            let hosted = scene
                .effects
                .hosted_particles
                .iter()
                .find(|(id, _)| *id == sys.id)
                .map(|(_, p)| p);
            let colliders: &[crate::particles::SceneCollider] =
                if e.collision.is_some() && e.space == SimulationSpace::World {
                    &scene.world.colliders
                } else {
                    &[]
                };
            let sprite = match e.render {
                ParticleRender::Sprite { layer } => Some((
                    layer,
                    match e.array {
                        Some(a) => {
                            let layers = scene.texture_array(a).map(|d| d.layers.len() as u32);
                            let layers = layers.ok_or_else(|| {
                                RenderError::InvalidHandle(format!(
                                    "emitter {}: texture array {}",
                                    sys.emitter.index(),
                                    a.index()
                                ))
                            })?;
                            Some((a, array_frames(&e.sheet, layers)))
                        }
                        None => None,
                    },
                )),
                _ => None,
            };
            let use_gpu = gpu
                && hosted.is_none()
                && (sprite.is_none() || e.sub_emitters.is_empty())
                && gpu_subs_ok(e, &subs);
            if !use_gpu && let Some(old_gpu) = state.gpu.take() {
                release_gpu_system(&mut self.pool, old_gpu, state.max);
            }
            let meets_colliders = !use_gpu || e.collides_with_colliders();
            if e.collision.is_some() && meets_colliders && scene.world.colliders.is_empty() {
                out.collision_skipped += 1;
            }

            if let Some((layer, array)) = sprite
                && !use_gpu
            {
                let own;
                let cpu_e = if sys.tint == Vec3::ONE && emission_scale == 1.0 {
                    e
                } else {
                    let t = sys.tint;
                    own = ParticleEmitter {
                        color: [
                            e.color[0] * t.x,
                            e.color[1] * t.y,
                            e.color[2] * t.z,
                            e.color[3],
                        ],
                        emission: e.emission * emission_scale,
                        ..e.clone()
                    };
                    &own
                };
                let pt = &self.points[&sys.emitter.index()];
                let shape_pts = &pt.table[..pt.shape_len];
                match hosted {
                    Some(h) => take_hosted(&mut state.cpu, h, e.space, scene.frame.origin),
                    None => {
                        out.ticks += state.cpu.advance(
                            cpu_e,
                            sys,
                            tick_rate,
                            shape_pts,
                            colliders,
                            &subs,
                            None,
                            history,
                            &scene.world.force_fields,
                        )
                    }
                }
                out.cpu_alive += state.cpu.alive() as u32;
                push_sprites(
                    cpu_e,
                    sys,
                    &state.cpu.slots,
                    (layer, layers),
                    array,
                    &mut out.sprites,
                );
                for (c, ce) in state.cpu.children.iter().zip(&subs) {
                    let Some(ce) = ce else { continue };
                    let world = ParticleSystem {
                        transform: Mat4::IDENTITY,
                        ..*sys
                    };
                    let layer = match ce.render {
                        ParticleRender::Sprite { layer } => layer,
                        _ => layer,
                    };
                    let ce = ParticleEmitter {
                        space: SimulationSpace::World,
                        ..(*ce).clone()
                    };
                    push_sprites(
                        &ce,
                        &world,
                        &c.slots,
                        (layer, layers),
                        None,
                        &mut out.sprites,
                    );
                }
                continue;
            }

            let depth = (sys.transform.w_axis.truncate() - eye).dot(forward);
            let textured = e.texture.is_some();
            let to_colliders = use_gpu && e.collides_with_colliders();
            let collide_depth = use_gpu
                && e.collision.is_some()
                && e.space == SimulationSpace::World
                && !to_colliders;
            let kind = match sprite {
                Some(_) => DrawKind::Sprite,
                None => draw_kind(e),
            };
            let (effect, material) = effect_of(scene, e, kind, sys.emitter.index())?;
            let soft = if scene.frame.quality.soft_particles && sprite.is_none() {
                e.soft_distance
                    .max(material.map_or(0.0, |m| m.soft_depth))
                    .max(0.0)
            } else {
                0.0
            };
            let invisible_parent =
                e.size[0] <= 0.0 && e.size[1] <= 0.0 && e.trail.is_none() && sprite.is_none();
            if !invisible_parent {
                out.needs_depth |= soft > 0.0;
                out.soft |= soft > 0.0;
            }
            out.needs_depth |= collide_depth;
            let look = look_of(
                kind,
                material,
                material.and_then(|m| m.shader).map(|h| h.index()),
            );
            let blend = material.and_then(|m| m.blend).unwrap_or_else(|| e.blend());
            if !invisible_parent {
                out.distortion |= kind == DrawKind::Distort;
            }
            let mesh = match e.render {
                ParticleRender::Mesh { mesh } => Some(mesh),
                _ => None,
            };

            if use_gpu {
                let existed = state.gpu.is_some();
                let pool = &mut self.pool;
                let g = state.gpu.get_or_insert_with(|| {
                    let particle_base = pool.particle_ranges.alloc(e.max_particles);
                    let trail_base = if trail_need > 0 {
                        pool.trail_ranges.alloc(trail_need)
                    } else {
                        0
                    };
                    GpuSystem {
                        particle_base,
                        trail_base,
                        trail_max: trail_need,
                        time: 0.0,
                        last_tick: None,
                        prev_transform: None,
                        clock: Default::default(),
                        children: Vec::new(),
                        was_empty: false,
                    }
                });
                // Sync GPU child slab allocations with the emitter's sub_emitters.
                let children_match =
                    g.children.len() == subs.len()
                        && g.children.iter().zip(&e.sub_emitters).zip(&subs).all(
                            |((gc, sub), ce)| {
                                ce.is_some_and(|ce| {
                                    gc.emitter == sub.emitter.index() && gc.max == ce.max_particles
                                })
                            },
                        );
                let mut children_reset = false;
                if !children_match {
                    for old in g.children.drain(..) {
                        pool.particle_ranges.release(old.particle_base, old.max);
                        pool.counter_ranges.release(old.counter_idx, 1);
                    }
                    for (sub, ce) in e.sub_emitters.iter().zip(&subs) {
                        let ce = ce.expect("validated above");
                        let particle_base = pool.particle_ranges.alloc(ce.max_particles);
                        let counter_idx = pool.counter_ranges.alloc(1);
                        g.children.push(GpuChild {
                            emitter: sub.emitter.index(),
                            max: ce.max_particles,
                            particle_base,
                            counter_idx,
                        });
                    }
                    children_reset = true;
                }

                let mut steps = Vec::new();
                let reset = plan_ticks(
                    e,
                    sys,
                    g.last_tick,
                    g.prev_transform,
                    tick_rate,
                    &mut g.clock,
                    history,
                    &mut steps,
                );
                g.last_tick = Some(sys.tick);
                g.prev_transform = Some(sys.transform);
                let live = crate::particles::live_spawns(e, sys, tick_rate, &g.clock);
                let slots = drawn(e.max_particles, Some(live));
                let slots_empty = slots.iter().all(|r| r.is_empty());
                let dormant_empty = !reset
                    && existed
                    && !children_reset
                    && g.children.is_empty()
                    && g.was_empty
                    && slots_empty
                    && steps.iter().all(|s| s.count == 0);
                g.was_empty = slots_empty && !reset;
                let bound = swept_bound(e, sys, tick_rate, &g.clock, &mut state.path, pull);
                let n_points = (shape_len / 2) as u32;
                let local = e.space == SimulationSpace::Local;

                if dormant_empty {
                    continue;
                }

                // Queue origin shifts (skipped when the system is being cleared).
                if !reset && existed && moved != Vec3::ZERO {
                    let mut entry: EmitterSimGpu = bytemuck::Zeroable::zeroed();
                    entry.transform = Mat4::from_translation(moved).to_cols_array_2d();
                    entry.info = [e.max_particles, 0, u32::MAX, 0];
                    entry.pool = [g.particle_base, g.trail_base, 0, 0];
                    entry.trail[0] = e.trail.map_or(0.0, |t| t.lifetime);
                    let idx = pool.sim_entries_cpu.len() as u32;
                    pool.sim_entries_cpu.push(entry);
                    shift_list.push((idx, e.max_particles));
                }
                if !reset && !children_reset && existed && pending != Vec3::ZERO {
                    for gc in &g.children {
                        let mut entry: EmitterSimGpu = bytemuck::Zeroable::zeroed();
                        entry.transform = Mat4::from_translation(pending).to_cols_array_2d();
                        entry.info = [gc.max, 0, u32::MAX, 0];
                        entry.pool = [gc.particle_base, 0, 0, 0];
                        let idx = pool.sim_entries_cpu.len() as u32;
                        pool.sim_entries_cpu.push(entry);
                        shift_list.push((idx, gc.max));
                    }
                }
                // Queue clear on reset or newly allocated slabs.
                if reset || !existed {
                    let mut entry: EmitterSimGpu = bytemuck::Zeroable::zeroed();
                    entry.info = [e.max_particles, 0, u32::MAX, 0];
                    entry.pool = [g.particle_base, g.trail_base, 0, u32::MAX];
                    let idx = pool.sim_entries_cpu.len() as u32;
                    pool.sim_entries_cpu.push(entry);
                    clear_list.push((idx, e.max_particles));
                }
                if reset || children_reset || !existed {
                    for gc in &g.children {
                        let mut entry: EmitterSimGpu = bytemuck::Zeroable::zeroed();
                        entry.info = [gc.max, 0, u32::MAX, 0];
                        entry.pool = [gc.particle_base, 0, 0, gc.counter_idx];
                        let idx = pool.sim_entries_cpu.len() as u32;
                        pool.sim_entries_cpu.push(entry);
                        clear_list.push((idx, gc.max));
                    }
                }

                // Queue tick waves (children integrate first, then parent simulates & spawns children).
                if wave_parents.len() < steps.len() {
                    wave_parents.resize_with(steps.len(), Vec::new);
                    wave_children.resize_with(steps.len(), Vec::new);
                }
                let parent_sim_start = pool.sim_entries_cpu.len() as u32;
                let mut child_colliding: Vec<(usize, u32)> = Vec::new();
                for (wave_idx, s) in steps.iter().enumerate() {
                    // Build child sim entries first so parent can reference their indices.
                    let mut sub_local_base = 0u32;
                    let sub_count = g.children.len() as u32;
                    if sub_count > 0 {
                        sub_local_base = sub_descs_cpu.len() as u32;
                        for (k, (gc, ce)) in g.children.iter().zip(&subs).enumerate() {
                            let ce = ce.expect("validated above");
                            let cseed = crate::particles::child_seed(seed, k);
                            let c_to_col = ce.collides_with_colliders();
                            let c_collide_depth = ce.collision.is_some() && !c_to_col;
                            out.needs_depth |= c_collide_depth;
                            let (c_pbase, c_gbase) = child_grid_bases[k];
                            let mut cs = emitter_sim_gpu(
                                ce,
                                Mat4::IDENTITY,
                                None,
                                None,
                                s.dt,
                                s.time,
                                cseed,
                                emission_scale,
                                0,
                                c_pbase,
                                c_gbase,
                                gc.particle_base,
                                0,
                                c_collide_depth,
                                true,
                            );
                            let c_air = crate::particles::carrying_air(ce, sys.air, None);
                            cs.air = [c_air.x, c_air.y, c_air.z, ce.force_fields];
                            let c_norig = {
                                let (whole, rest) = crate::particles::noise_origin(
                                    scene.frame.origin,
                                    ce.noise.frequency,
                                );
                                [
                                    [whole[0] as f32, whole[1] as f32, whole[2] as f32, 0.0],
                                    rest.extend(0.0).to_array(),
                                ]
                            };
                            let c_forig = ce.field.as_ref().map_or([[0.0; 4]; 2], |f| {
                                let (whole, rest) =
                                    crate::particles::field_origin(f, scene.frame.origin);
                                [
                                    [whole[0] as f32, whole[1] as f32, whole[2] as f32, 0.0],
                                    rest.extend(0.0).to_array(),
                                ]
                            });
                            cs.noise_origin = c_norig;
                            cs.field_origin = c_forig;
                            let child_ei = pool.sim_entries_cpu.len() as u32;
                            pool.sim_entries_cpu.push(cs);
                            wave_children[wave_idx].push((child_ei, gc.max));
                            if c_to_col {
                                child_colliding.push((k, child_ei));
                            }
                            let packed =
                                pack_sub_desc(&e.sub_emitters[k], ce, child_ei, gc.counter_idx);
                            sub_descs_cpu.extend_from_slice(&packed);
                        }
                    }

                    let mut step_gpu = emitter_sim_gpu(
                        e,
                        s.transform,
                        s.prev_transform,
                        Some(s),
                        s.dt,
                        s.time,
                        seed,
                        emission_scale,
                        n_points,
                        points_base,
                        grid_base,
                        g.particle_base,
                        g.trail_base,
                        collide_depth,
                        false,
                    );
                    let air =
                        crate::particles::carrying_air(e, sys.air, local.then_some(s.transform));
                    step_gpu.air = [air.x, air.y, air.z, step_gpu.air[3]];
                    step_gpu.noise_origin = noise_origin;
                    step_gpu.field_origin = field_origin;
                    if let Some(iv) = interval {
                        step_gpu.trail[2] = iv;
                        step_gpu.trail[3] = crate::particles::trail_sample(s.time, s.dt, iv)
                            .map_or(-1.0, |idx| idx as f32);
                        g.time = s.time;
                    }
                    if sub_count > 0 {
                        step_gpu.pool[3] = (sub_count << 20) | sub_local_base;
                    }
                    let parent_ei = pool.sim_entries_cpu.len() as u32;
                    pool.sim_entries_cpu.push(step_gpu);
                    if sub_count > 0 {
                        sub_patch.push(parent_ei);
                    }
                    wave_parents[wave_idx].push((parent_ei, e.max_particles));
                    if to_colliders {
                        colliding
                            .push((parent_ei..parent_ei + 1, reach(e, sys.transform, tick_rate)));
                    }
                }
                let _ = parent_sim_start;
                for (k, child_ei) in child_colliding {
                    if let Some(ce) = subs[k] {
                        let (plo, phi) = reach(e, sys.transform, tick_rate);
                        let (clo, chi) = reach(ce, Mat4::IDENTITY, tick_rate);
                        colliding.push((child_ei..child_ei + 1, (plo + clo, phi + chi)));
                    }
                }

                out.ticks += steps.len() as u32;
                out.any_gpu = true;

                // Parent draw & compaction (skip if invisible zero-size parent or empty live window).
                if let Some((layer, array)) = sprite {
                    if !slots_empty {
                        let draw_entry = draw_entries.len() as u32;
                        let mut draw = emitter_draw_gpu(
                            e,
                            sys.transform,
                            sys.tint,
                            sys.hue,
                            seed,
                            textured,
                            soft,
                            FULL_UV,
                            false,
                        );
                        newest_sample(&mut draw, interval, g.time);
                        draw_entries.push(draw);
                        out.sprite_systems.push(SpriteSystem {
                            draw_entry,
                            particle_base: g.particle_base,
                            layer,
                            layers,
                            blend: match e.blend() {
                                ParticleBlend::Additive => crate::SpriteBlend::Additive,
                                _ => crate::SpriteBlend::Alpha,
                            },
                            glow: e.glow,
                            soft: e.round || (e.texture.is_none() && array.is_none()),
                            texture: if array.is_some() { None } else { e.texture },
                            array: array.map(|(a, _)| a),
                            frames: array.map_or_else(|| sheet_frames(&e.sheet), |(_, n)| n),
                            columns: e.sheet.columns.max(1),
                            rows: e.sheet.rows.max(1),
                            max: e.max_particles,
                            z: sys.transform.w_axis.z,
                        });
                    }
                } else if !invisible_parent && !slots_empty {
                    let draw_entry = draw_entries.len() as u32;
                    let mut draw = emitter_draw_gpu(
                        e,
                        sys.transform,
                        sys.tint,
                        sys.hue,
                        seed,
                        textured,
                        soft,
                        billboard_crop(e, &texture_bounds),
                        false,
                    );
                    newest_sample(&mut draw, interval, g.time);
                    draw_entries.push(draw);
                    let has_trail = e.trail.is_some();
                    let head_drawn = e.trail.is_none_or(|t| t.head);
                    let actual_kind = if has_trail && !head_drawn {
                        DrawKind::Trail
                    } else {
                        kind
                    };
                    let pair_trail = has_trail && head_drawn;
                    let can_compact = indirect
                        && (actual_kind != DrawKind::Mesh || mesh_index_count > 0)
                        && (blend != ParticleBlend::Alpha
                            || oit_order_free
                            || e.max_particles <= 64);
                    let (head_source, trail_source) = if can_compact {
                        let mut planner = CompactPlanner {
                            tiles: &mut pool.compact_tiles_cpu,
                            indirect: &mut pool.indirect_init_cpu,
                            particle_slots: compact_particle_slots,
                            trail_slots: compact_trail_slots,
                        };
                        let (hc, tc) = planner.plan(
                            g.particle_base,
                            has_trail.then_some(g.trail_base),
                            &slots,
                            actual_kind,
                            mesh_index_count,
                            pair_trail,
                        );
                        compact_particle_slots = planner.particle_slots;
                        compact_trail_slots = planner.trail_slots;
                        (VertexSource::Compact(hc), tc.map(VertexSource::Compact))
                    } else {
                        let ps = VertexSource::Pool(PoolSlice {
                            particles_offset: u64::from(g.particle_base) * PARTICLE_STRIDE,
                            trails_offset: u64::from(g.trail_base) * TRAIL_POINTS as u64 * 16,
                        });
                        (ps, pair_trail.then_some(ps))
                    };
                    let (head_effect, head_look, head_mesh) = if actual_kind == DrawKind::Trail {
                        let (ef, mat) = effect_of(scene, e, DrawKind::Trail, sys.emitter.index())?;
                        let lk = look_of(
                            DrawKind::Trail,
                            mat,
                            mat.and_then(|m| m.shader).map(|h| h.index()),
                        );
                        (ef, lk, None)
                    } else {
                        (effect, look, mesh)
                    };
                    let head_half = qualifies_half_res(
                        scene,
                        actual_kind,
                        blend,
                        head_look,
                        soft,
                        oit_order_free,
                    );
                    out.systems.push(FrameSystem {
                        id: sys.id,
                        draw_entry,
                        blend,
                        texture: e.texture,
                        depth,
                        layers,
                        child: None,
                        slots: slots.clone(),
                        source: head_source,
                        bound: Some(bound),
                        kind: actual_kind,
                        mesh: head_mesh,
                        effect: head_effect,
                        look: head_look,
                        half_res: head_half,
                    });
                    if let Some(tsrc) = trail_source {
                        let (ef, mat) = effect_of(scene, e, DrawKind::Trail, sys.emitter.index())?;
                        let lk = look_of(
                            DrawKind::Trail,
                            mat,
                            mat.and_then(|m| m.shader).map(|h| h.index()),
                        );
                        out.systems.push(FrameSystem {
                            id: sys.id,
                            draw_entry,
                            blend,
                            texture: e.texture,
                            depth,
                            layers,
                            child: None,
                            slots,
                            source: tsrc,
                            bound: Some(bound),
                            kind: DrawKind::Trail,
                            mesh: None,
                            effect: ef,
                            look: lk,
                            half_res: false,
                        });
                    }
                }

                // GPU sub-emitter children draws + compaction!
                for (k, (gc, ce)) in g.children.iter().zip(&subs).enumerate() {
                    let Some(ce) = ce else { continue };
                    out.capacity += ce.max_particles;
                    let ckind = draw_kind(ce);
                    out.distortion |= ckind == DrawKind::Distort;
                    let (ceffect, cmaterial) =
                        effect_of(scene, ce, ckind, e.sub_emitters[k].emitter.index())?;
                    let csoft = if scene.frame.quality.soft_particles {
                        ce.soft_distance
                            .max(cmaterial.map_or(0.0, |m| m.soft_depth))
                            .max(0.0)
                    } else {
                        0.0
                    };
                    out.needs_depth |= csoft > 0.0;
                    out.soft |= csoft > 0.0;
                    let clook = look_of(
                        ckind,
                        cmaterial,
                        cmaterial.and_then(|m| m.shader).map(|h| h.index()),
                    );
                    let cblend = cmaterial
                        .and_then(|m| m.blend)
                        .unwrap_or_else(|| ce.blend());
                    let cdraw_entry = draw_entries.len() as u32;
                    let cdraw = emitter_draw_gpu(
                        ce,
                        Mat4::IDENTITY,
                        sys.tint,
                        sys.hue,
                        crate::particles::child_seed(seed, k),
                        ce.texture.is_some(),
                        csoft,
                        billboard_crop(ce, &texture_bounds),
                        true,
                    );
                    draw_entries.push(cdraw);
                    let cslots = [0..gc.max, 0..0];
                    let cmc = child_mesh_counts[k];
                    let can_compact = indirect
                        && (ckind != DrawKind::Mesh || cmc > 0)
                        && (cblend != ParticleBlend::Alpha || oit_order_free || gc.max <= 64);
                    let csource = if can_compact {
                        let mut planner = CompactPlanner {
                            tiles: &mut pool.compact_tiles_cpu,
                            indirect: &mut pool.indirect_init_cpu,
                            particle_slots: compact_particle_slots,
                            trail_slots: compact_trail_slots,
                        };
                        let (hc, _) =
                            planner.plan(gc.particle_base, None, &cslots, ckind, cmc, false);
                        compact_particle_slots = planner.particle_slots;
                        compact_trail_slots = planner.trail_slots;
                        VertexSource::Compact(hc)
                    } else {
                        VertexSource::Pool(PoolSlice {
                            particles_offset: u64::from(gc.particle_base) * PARTICLE_STRIDE,
                            trails_offset: 0,
                        })
                    };
                    out.systems.push(FrameSystem {
                        id: sys.id,
                        draw_entry: cdraw_entry,
                        blend: cblend,
                        texture: ce.texture,
                        depth,
                        layers,
                        child: Some(k),
                        slots: cslots,
                        source: csource,
                        bound: None,
                        kind: ckind,
                        mesh: match ce.render {
                            ParticleRender::Mesh { mesh } => Some(mesh),
                            _ => None,
                        },
                        effect: ceffect,
                        look: clook,
                        half_res: qualifies_half_res(
                            scene,
                            ckind,
                            cblend,
                            clook,
                            csoft,
                            oit_order_free,
                        ),
                    });
                }
            } else {
                // CPU simulation path (hosted events, Tier::Gl, or ParticleSimulation::Cpu).
                let own;
                let cpu_e = if sys.tint == Vec3::ONE && emission_scale == 1.0 {
                    e
                } else {
                    let t = sys.tint;
                    own = ParticleEmitter {
                        color: [
                            e.color[0] * t.x,
                            e.color[1] * t.y,
                            e.color[2] * t.z,
                            e.color[3],
                        ],
                        emission: e.emission * emission_scale,
                        ..e.clone()
                    };
                    &own
                };
                let pt = &self.points[&sys.emitter.index()];
                let shape_pts = &pt.table[..pt.shape_len];
                match hosted {
                    Some(h) => take_hosted(&mut state.cpu, h, e.space, scene.frame.origin),
                    None => {
                        out.ticks += state.cpu.advance(
                            cpu_e,
                            sys,
                            tick_rate,
                            shape_pts,
                            colliders,
                            &subs,
                            None,
                            history,
                            &scene.world.force_fields,
                        )
                    }
                }
                out.cpu_alive += state.cpu.alive() as u32;
                let stepped = state.cpu.last_tick == Some(sys.tick);
                let slots = drawn(
                    state.cpu.slots.len() as u32,
                    stepped.then(|| {
                        crate::particles::live_spawns(cpu_e, sys, tick_rate, &state.cpu.clock)
                    }),
                );
                if !invisible_parent {
                    let packed = Packed {
                        particles: pack(&mut self.cpu_bytes, &state.cpu.slots, 1, &slots),
                        trails: match state.cpu.trail.is_empty() {
                            true => [0; 2],
                            false => {
                                pack(&mut self.cpu_bytes, &state.cpu.trail, TRAIL_POINTS, &slots)
                            }
                        },
                    };
                    let draw_entry = draw_entries.len() as u32;
                    let mut draw = emitter_draw_gpu(
                        e,
                        sys.transform,
                        sys.tint,
                        sys.hue,
                        seed,
                        textured,
                        soft,
                        billboard_crop(e, &texture_bounds),
                        false,
                    );
                    newest_sample(&mut draw, interval, state.cpu.time);
                    draw_entries.push(draw);
                    let cpu_half =
                        qualifies_half_res(scene, kind, blend, look, soft, oit_order_free);
                    out.systems.push(FrameSystem {
                        id: sys.id,
                        draw_entry,
                        blend,
                        texture: e.texture,
                        depth,
                        layers,
                        child: None,
                        slots,
                        source: VertexSource::Cpu(packed),
                        bound: stepped.then(|| {
                            swept_bound(
                                cpu_e,
                                sys,
                                tick_rate,
                                &state.cpu.clock,
                                &mut state.path,
                                pull,
                            )
                        }),
                        kind,
                        mesh,
                        effect,
                        look,
                        half_res: cpu_half,
                    });
                    if !state.cpu.trail.is_empty() {
                        push_trail(&mut out, e, scene, sys.emitter.index())?;
                    }
                }
                for (k, (c, ce)) in state.cpu.children.iter().zip(&subs).enumerate() {
                    let Some(ce) = ce else { continue };
                    if c.slots.is_empty() {
                        continue;
                    }
                    let (alive, live) = child_live(c);
                    out.cpu_alive += alive;
                    let cslots = drawn(c.slots.len() as u32, Some(live));
                    let packed = Packed {
                        particles: pack(&mut self.cpu_bytes, &c.slots, 1, &cslots),
                        trails: [0; 2],
                    };
                    let ckind = draw_kind(ce);
                    out.distortion |= ckind == DrawKind::Distort;
                    let (ceffect, cmaterial) =
                        effect_of(scene, ce, ckind, e.sub_emitters[k].emitter.index())?;
                    let csoft = if scene.frame.quality.soft_particles {
                        ce.soft_distance
                            .max(cmaterial.map_or(0.0, |m| m.soft_depth))
                            .max(0.0)
                    } else {
                        0.0
                    };
                    let clook = look_of(
                        ckind,
                        cmaterial,
                        cmaterial.and_then(|m| m.shader).map(|h| h.index()),
                    );
                    out.needs_depth |= csoft > 0.0;
                    out.soft |= csoft > 0.0;
                    let cblend = cmaterial
                        .and_then(|m| m.blend)
                        .unwrap_or_else(|| ce.blend());
                    let draw_entry = draw_entries.len() as u32;
                    let entry = emitter_draw_gpu(
                        ce,
                        Mat4::IDENTITY,
                        sys.tint,
                        sys.hue,
                        crate::particles::child_seed(seed, k),
                        ce.texture.is_some(),
                        csoft,
                        billboard_crop(ce, &texture_bounds),
                        true,
                    );
                    draw_entries.push(entry);
                    out.systems.push(FrameSystem {
                        id: sys.id,
                        draw_entry,
                        blend: cblend,
                        texture: ce.texture,
                        depth,
                        layers,
                        child: Some(k),
                        slots: cslots,
                        source: VertexSource::Cpu(packed),
                        bound: None,
                        kind: ckind,
                        mesh: match ce.render {
                            ParticleRender::Mesh { mesh } => Some(mesh),
                            _ => None,
                        },
                        effect: ceffect,
                        look: clook,
                        half_res: qualifies_half_res(
                            scene,
                            ckind,
                            cblend,
                            clook,
                            csoft,
                            oit_order_free,
                        ),
                    });
                }
            }
        }

        out.effect_meshes = self.effect_meshes.prepare(
            device,
            queue,
            scene,
            oit_order_free,
            eye,
            forward,
            &mut out.meshes,
            &mut out.needs_depth,
            &mut out.soft,
        )?;

        // Retire absent systems and return their GPU slabs to the Mega-Pool.
        let frame = self.frame;
        let pool = &mut self.pool;
        self.systems.retain(|_, s| {
            let keep = s.seen == frame;
            if !keep && let Some(g) = s.gpu.take() {
                release_gpu_system(pool, g, s.max);
            }
            keep
        });

        // Assemble merged compute wave tiles + dispatch uniforms.
        let mut waves: Vec<SimWave> = Vec::new();
        let mut push_wave = |kind: SimWaveKind,
                             list: &[(u32, u32)],
                             tiles: &mut Vec<[u32; 4]>,
                             dispatch_bytes: &mut Vec<u8>| {
            if list.is_empty() {
                return;
            }
            let tile_offset = tiles.len() as u32;
            for &(ei, max) in list {
                push_sim_tiles(tiles, ei, max);
            }
            let tile_count = tiles.len() as u32 - tile_offset;
            if tile_count == 0 {
                return;
            }
            let dispatch_slot = waves.len() as u32;
            let at = dispatch_bytes.len();
            dispatch_bytes.resize(at + SIM_DISPATCH_STRIDE as usize, 0);
            let d = SimDispatchGpu {
                tile_offset,
                tile_count,
                _pad0: 0,
                _pad1: 0,
            };
            dispatch_bytes[at..at + std::mem::size_of::<SimDispatchGpu>()]
                .copy_from_slice(bytemuck::bytes_of(&d));
            waves.push(SimWave {
                kind,
                dispatch_slot,
                tile_count,
            });
        };
        push_wave(
            SimWaveKind::Shift,
            &shift_list,
            &mut pool.sim_tiles_cpu,
            &mut pool.sim_dispatch_bytes,
        );
        push_wave(
            SimWaveKind::Clear,
            &clear_list,
            &mut pool.sim_tiles_cpu,
            &mut pool.sim_dispatch_bytes,
        );
        for (wc, wp) in wave_children.iter().zip(&wave_parents) {
            push_wave(
                SimWaveKind::Simulate,
                wc,
                &mut pool.sim_tiles_cpu,
                &mut pool.sim_dispatch_bytes,
            );
            push_wave(
                SimWaveKind::Simulate,
                wp,
                &mut pool.sim_tiles_cpu,
                &mut pool.sim_dispatch_bytes,
            );
        }
        // Append sub-emitter descriptors after wave tiles and patch parent offsets.
        if !sub_descs_cpu.is_empty() {
            let sub_base_offset = pool.sim_tiles_cpu.len() as u32;
            pool.sim_tiles_cpu.extend_from_slice(&sub_descs_cpu);
            for ei in sub_patch {
                let meta = pool.sim_entries_cpu[ei as usize].pool[3];
                let count = meta >> 20;
                let local = meta & 0xfffff;
                pool.sim_entries_cpu[ei as usize].pool[3] =
                    (count << 20) | ((sub_base_offset + local) & 0xfffff);
            }
        }
        out.gpu_plan = FrameGpuPlan {
            waves,
            compact_tiles: pool.compact_tiles_cpu.len() as u32,
        };
        out.any_gpu = !out.gpu_plan.waves.is_empty()
            || out.gpu_plan.compact_tiles > 0
            || !out.sprite_systems.is_empty();

        // The unsupported diagnostic is opaque, not emitted light.
        if scene.frame.debug != crate::DebugView::None {
            for s in &mut out.systems {
                if s.kind != DrawKind::Sprite {
                    s.blend = ParticleBlend::Alpha;
                }
            }
        }
        // Far systems first; additive after alpha within the same depth.
        og_core::sort::sort_by(&mut out.systems, |a, b| {
            b.depth
                .total_cmp(&a.depth)
                .then(blend_order(a.blend).cmp(&blend_order(b.blend)))
                .then(a.id.cmp(&b.id))
                .then(a.child.cmp(&b.child))
        });

        // Upload force fields and scene colliders for compute simulation.
        if !out.gpu_plan.waves.is_empty() {
            if !scene.world.force_fields.is_empty() {
                let words = pack_fields(&scene.world.force_fields);
                let bytes: &[u8] = bytemuck::cast_slice(&words);
                if bytes.len() as u64 > self.fields.size() {
                    self.fields = collider_buffer(device, (bytes.len() as u64).next_power_of_two());
                    self.fields_gen += 1;
                }
                crate::traffic::write_buffer(queue, &self.fields, 0, bytes);
            } else {
                crate::traffic::write_buffer(
                    queue,
                    &self.fields,
                    0,
                    bytemuck::cast_slice(&[[0.0f32; 4]]),
                );
            }
            if !colliding.is_empty() {
                let bounds: Vec<(Vec3, Vec3)> = colliding.iter().map(|(_, b)| *b).collect();
                let (words, lists) = crate::scene_colliders::pack(&scene.world.colliders, &bounds);
                for ((range, _), list) in colliding.iter().zip(lists) {
                    for g in
                        &mut self.pool.sim_entries_cpu[range.start as usize..range.end as usize]
                    {
                        g.collision[3] = 1.0;
                        g.info[2] = list;
                    }
                }
                let bytes: &[u8] = bytemuck::cast_slice(&words);
                if bytes.len() as u64 > self.colliders.size() {
                    self.colliders =
                        collider_buffer(device, (bytes.len() as u64).next_power_of_two());
                    self.colliders_gen += 1;
                }
                crate::traffic::write_buffer(queue, &self.colliders, 0, bytes);
            }
        }

        // Grow persistent Mega-Pool buffers and upload simulation/compaction work.
        self.pool.ensure_persistent(device);
        self.pool.upload_points(device, queue);
        self.pool.upload_sim_work(device, queue);
        self.pool
            .upload_compact_work(device, queue, compact_particle_slots, compact_trail_slots);

        // Upload lean draw-only emitter uniforms.
        if !draw_entries.is_empty() {
            let size = EMITTER_DRAW_STRIDE * draw_entries.len() as u64;
            if size > self.emitter_draw_buffer.size() {
                self.emitter_draw_buffer = crate::traffic::create_buffer(
                    device,
                    &wgpu::BufferDescriptor {
                        label: Some("emitters (draw)"),
                        size: size.next_power_of_two(),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    },
                );
                self.emitter_draw_gen += 1;
            }
            let bytes = &mut self.emitter_draw_bytes;
            bytes.resize(size as usize, 0);
            for (i, e) in draw_entries.iter().enumerate() {
                let at = i * EMITTER_DRAW_STRIDE as usize;
                bytes[at..at + std::mem::size_of::<EmitterDrawGpu>()]
                    .copy_from_slice(bytemuck::bytes_of(e));
            }
            crate::traffic::write_buffer(queue, &self.emitter_draw_buffer, 0, bytes);
        }
        if !self.cpu_bytes.is_empty() {
            let size = self.cpu_bytes.len() as u64;
            if size > self.cpu_vertices.size() {
                self.cpu_vertices = cpu_vertex_buffer(device, size.next_power_of_two());
            }
            crate::traffic::write_buffer(queue, &self.cpu_vertices, 0, &self.cpu_bytes);
        }
        self.draw_entries = draw_entries;
        Ok(out)
    }

    /// Writes the parameters of [`ParticleRenderer::expand_sprites`]'
    /// ranges (before the frame's encoder runs).
    pub fn prepare_sprites(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &ParticleFrame,
        ranges: &crate::sprites::GpuSprites,
    ) {
        if ranges.ranges.is_empty() {
            return;
        }
        let size = SPRITE_PARAMS_STRIDE * ranges.ranges.len() as u64;
        if size > self.sprites_params.size() {
            self.sprites_params = crate::traffic::create_buffer(
                device,
                &wgpu::BufferDescriptor {
                    label: Some("particle sprites"),
                    size: size.next_power_of_two(),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                },
            );
            self.sprites_params_gen += 1;
        }
        let bytes = &mut self.pool.sprite_bytes;
        bytes.clear();
        bytes.resize(size as usize, 0);
        for (k, r) in ranges.ranges.iter().enumerate() {
            let s = &frame.sprite_systems[r.system];
            let p = SpriteParamsGpu {
                first: r.first,
                max: r.count,
                frames: s.frames,
                columns: s.columns,
                rows: s.rows,
                array: u32::from(s.array.is_some()),
                soft: u32::from(s.soft),
                particle_base: s.particle_base,
                z: s.z,
                glow: s.glow,
                lit: r.lit,
                _pad1: 0.0,
            };
            let at = k * SPRITE_PARAMS_STRIDE as usize;
            bytes[at..at + std::mem::size_of::<SpriteParamsGpu>()]
                .copy_from_slice(bytemuck::bytes_of(&p));
        }
        crate::traffic::write_buffer(queue, &self.sprites_params, 0, bytes);
    }

    /// Writes the instances of the sprite-mode systems simulated in compute
    /// into `target` using one cached bind group over `particles_pool`.
    pub fn expand_sprites(
        &self,
        device: &wgpu::Device,
        encoder: &mut crate::Encoder,
        frame: &ParticleFrame,
        ranges: &crate::sprites::GpuSprites,
        target: &wgpu::Buffer,
    ) {
        if ranges.ranges.is_empty() {
            return;
        }
        let group = {
            let key = (
                self.emitter_draw_gen,
                self.pool.particles_gen,
                self.sprites_params_gen,
                target.clone(),
            );
            let mut cache = self.pool.cache.lock().expect("particle sim cache");
            if let Some((k, g)) = &cache.sprites
                && *k == key
            {
                g.clone()
            } else {
                let g = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("particle sprites"),
                    layout: &self.sprites_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &self.emitter_draw_buffer,
                                offset: 0,
                                size: wgpu::BufferSize::new(
                                    std::mem::size_of::<EmitterDrawGpu>() as u64
                                ),
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: self.pool.particles.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &self.sprites_params,
                                offset: 0,
                                size: wgpu::BufferSize::new(
                                    std::mem::size_of::<SpriteParamsGpu>() as u64
                                ),
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: target.as_entire_binding(),
                        },
                    ],
                });
                cache.sprites = Some((key, g.clone()));
                g
            }
        };
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("particle sprites"),
            timestamp_writes: None,
        });
        pass.set_pipeline(self.sprites_pipeline.wait());
        for (k, r) in ranges.ranges.iter().enumerate() {
            let s = &frame.sprite_systems[r.system];
            pass.set_bind_group(
                0,
                &group,
                &[
                    (u64::from(s.draw_entry) * EMITTER_DRAW_STRIDE) as u32,
                    (k as u64 * SPRITE_PARAMS_STRIDE) as u32,
                ],
            );
            pass.dispatch_workgroups(r.count.div_ceil(64), 1, 1);
        }
    }
}

/// Tie-break order at equal view depth: alpha/multiply/subtractive before
/// additive so glowing emissives layer cleanly over smoke.
fn blend_order(b: ParticleBlend) -> u8 {
    match b {
        ParticleBlend::Alpha => 0,
        ParticleBlend::Multiply => 1,
        ParticleBlend::Subtractive => 2,
        ParticleBlend::Additive => 3,
    }
}

/// Appends the rows of `ranges`' slots (`per` items a slot) to the
/// frame's CPU vertices; where each range starts (bytes).
fn pack<T: bytemuck::Pod>(
    bytes: &mut Vec<u8>,
    items: &[T],
    per: usize,
    ranges: &[std::ops::Range<u32>; 2],
) -> [u64; 2] {
    let all: &[u8] = bytemuck::cast_slice(items);
    let row = per * std::mem::size_of::<T>();
    ranges.clone().map(|r| {
        let at = bytes.len() as u64;
        bytes.extend_from_slice(&all[r.start as usize * row..r.end as usize * row]);
        at
    })
}

/// A sub-emitter's living particles and the births they are among.
fn child_live(c: &crate::particles::ChildParticles) -> (u32, std::ops::Range<u64>) {
    let m = c.slots.len() as u64;
    let last = c.born.wrapping_sub(1);
    let (mut alive, mut oldest) = (0, c.born);
    for (s, p) in c.slots.iter().enumerate() {
        if p.alive() {
            alive += 1;
            oldest = oldest.min(last - (last % m + m - s as u64) % m);
        }
    }
    (alive, oldest..c.born)
}

/// A host's own simulation of a system moved to the scene's render origin.
fn take_hosted(
    cpu: &mut CpuParticles,
    hosted: &CpuParticles,
    space: SimulationSpace,
    origin: Vec3,
) {
    cpu.clone_from(hosted);
    let back = |slots: &mut [ParticleState]| {
        for p in slots {
            for (c, o) in p.pos_age.iter_mut().zip(origin.to_array()) {
                *c -= o;
            }
        }
    };
    if origin != Vec3::ZERO {
        if space == SimulationSpace::World {
            back(&mut cpu.slots);
            for p in &mut cpu.trail {
                for (c, o) in p.iter_mut().zip(origin.to_array()) {
                    *c -= o;
                }
            }
        }
        for c in &mut cpu.children {
            back(&mut c.slots);
        }
    }
}

/// Sprite-mode particles as sprites (alive slots, in slot order).
fn push_sprites(
    e: &ParticleEmitter,
    sys: &ParticleSystem,
    slots: &[ParticleState],
    (layer, layers): (i32, u32),
    array: Option<(crate::TextureArrayHandle, u32)>,
    out: &mut Vec<ExtraSprite>,
) {
    let columns = e.sheet.columns.max(1);
    let rows = e.sheet.rows.max(1);
    let blend = match e.blend() {
        ParticleBlend::Additive => crate::SpriteBlend::Additive,
        _ => crate::SpriteBlend::Alpha,
    };
    let z = sys.transform.w_axis.z;
    let frames = array.map_or_else(|| sheet_frames(&e.sheet), |(_, n)| n);
    let soft = e.round || (e.texture.is_none() && array.is_none());
    for p in slots.iter().filter(|p| p.alive()) {
        let (size, color, rotation, frame) = appearance(e, p, frames);
        let mut pos = Vec3::from_slice(&p.pos_age[..3]);
        if e.space == SimulationSpace::Local {
            pos = sys.transform.transform_point3(pos);
        }
        let (col, row) = (frame % columns, frame / columns);
        let (du, dv) = (1.0 / columns as f32, 1.0 / rows as f32);
        let uv_rect = match array {
            Some(_) => [0.0, 0.0, 1.0, 1.0],
            None => [
                col as f32 * du,
                row as f32 * dv,
                (col + 1) as f32 * du,
                (row + 1) as f32 * dv,
            ],
        };
        out.push(ExtraSprite {
            sprite: Sprite {
                texture: if array.is_some() { None } else { e.texture },
                position: Vec3::new(pos.x, pos.y, z),
                size: Vec2::from(size),
                anchor: Vec2::splat(0.5),
                rotation,
                color,
                uv_rect,
                layer,
                clip: None,
            },
            blend,
            glow: e.glow,
            soft,
            array: array.map(|(a, _)| (a, frame)),
            layers,
        });
    }
}

/// The effect material an emitter's particles of `kind` draw with.
fn effect_of<'s>(
    scene: &'s RenderScene,
    e: &ParticleEmitter,
    kind: DrawKind,
    emitter: usize,
) -> Result<
    (
        Option<crate::EffectMaterialHandle>,
        Option<&'s crate::EffectMaterial>,
    ),
    RenderError,
> {
    let Some(h) = e.effect else {
        return Ok((None, None));
    };
    let m = scene.effect_material(h).ok_or_else(|| {
        RenderError::InvalidHandle(format!("emitter {emitter}: effect material {}", h.index()))
    })?;
    Ok(match kind {
        DrawKind::Billboard | DrawKind::Mesh | DrawKind::Trail => (Some(h), Some(m)),
        DrawKind::Distort | DrawKind::Sprite => (None, None),
    })
}

/// A draw entry's trail: its seconds between samples and its newest
/// sample at the system's `time`.
fn newest_sample(entry: &mut EmitterDrawGpu, interval: Option<f32>, time: f32) {
    if let Some(iv) = interval {
        let (i, age) = crate::particles::trail_newest(time, iv);
        entry.trail[2] = iv;
        entry.trail[3] = i as f32;
        entry.counts2[2] = age;
    }
}

/// With a trail, the CPU system `out` just took draws its particles' trails
/// too, from its draw entry, or (`head` off) them instead of its particles.
fn push_trail(
    out: &mut ParticleFrame,
    e: &ParticleEmitter,
    scene: &RenderScene,
    emitter: usize,
) -> Result<(), RenderError> {
    let Some(t) = e.trail else {
        return Ok(());
    };
    let (effect, material) = effect_of(scene, e, DrawKind::Trail, emitter)?;
    let look = look_of(
        DrawKind::Trail,
        material,
        material.and_then(|m| m.shader).map(|h| h.index()),
    );
    let last = out.systems.last_mut().expect("the system's");
    if !t.head {
        last.kind = DrawKind::Trail;
        last.mesh = None;
        last.half_res = false;
        (last.effect, last.look) = (effect, look);
        return Ok(());
    }
    let trail = FrameSystem {
        id: last.id,
        draw_entry: last.draw_entry,
        blend: last.blend,
        texture: last.texture,
        depth: last.depth,
        layers: last.layers,
        child: None,
        slots: last.slots.clone(),
        source: last.source,
        bound: last.bound,
        kind: DrawKind::Trail,
        mesh: None,
        effect,
        look,
        half_res: false,
    };
    out.systems.push(trail);
    Ok(())
}
