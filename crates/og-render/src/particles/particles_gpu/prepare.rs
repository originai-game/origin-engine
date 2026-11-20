use super::*;

impl ParticleRenderer {
    /// Advances every system of the scene and writes the frame's emitter
    /// uniforms. `gpu`: billboard systems simulate in compute.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RenderScene,
        gpu: bool,
        texture_count: usize,
        eye: Vec3,
        forward: Vec3,
    ) -> Result<ParticleFrame, RenderError> {
        self.frame += 1;
        if self.scene_id != scene.id() {
            self.scene_id = scene.id();
            self.systems.clear();
            self.points.clear();
        }
        let tick_rate = scene.frame.tick_rate.max(1e-3);
        let dt = 1.0 / tick_rate;
        let mut out = ParticleFrame::default();
        let mut entries = std::mem::take(&mut self.entries);
        entries.clear();
        self.cpu_bytes.clear();
        // Compute systems meeting the scene's colliders: their entries and
        // the box their particles can reach.
        let mut colliding: Vec<(std::ops::Range<u32>, (Vec3, Vec3))> = Vec::new();
        for (i, sys) in scene.effects.particles.iter().enumerate() {
            let sys_entries = entries.len();
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
            // The system's tint on its emitter's colour, and its share and
            // the frame quality's thinning on its emission, for both paths.
            let own;
            let emission =
                sys.emission.clamp(0.0, 1.0) * e.thinned(scene.frame.quality.particle_emission);
            let e = if sys.tint == Vec3::ONE && emission == 1.0 {
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
                    emission: e.emission * emission,
                    ..e.clone()
                };
                &own
            };
            if let Some(t) = e.texture
                && t.index() >= texture_count
            {
                return Err(RenderError::InvalidHandle(format!(
                    "emitter {}: texture {}",
                    sys.emitter.index(),
                    t.index()
                )));
            }
            // The surface table, rebuilt when the emitter's shape changed.
            let fresh = match self.points.get(&sys.emitter.index()) {
                Some(t) => t.shape != e.shape || t.field != e.field,
                None => true,
            };
            if fresh {
                let mesh = match e.shape {
                    EmitterShape::Mesh { mesh } => Some(scene.mesh(mesh).ok_or_else(|| {
                        RenderError::InvalidHandle(format!(
                            "emitter {}: mesh {}",
                            sys.emitter.index(),
                            mesh.index()
                        ))
                    })?),
                    _ => None,
                };
                let mut table = crate::particles::shape_points(&e.shape, mesh);
                let shape_len = table.len();
                if let Some(crate::particles::VectorField::Grid { vectors, .. }) = &e.field {
                    table.extend(vectors.iter().map(|v| v.extend(0.0).to_array()));
                }
                self.points.insert(
                    sys.emitter.index(),
                    PointsTable {
                        shape: e.shape,
                        field: e.field.clone(),
                        table,
                        shape_len,
                    },
                );
            }
            let pt = &self.points[&sys.emitter.index()];
            // The CPU path reads the shape's table; compute reads the grid after it.
            let (all_points, points) = (&pt.table[..], &pt.table[..pt.shape_len]);
            let state = self.systems.entry(sys.id).or_insert_with(|| SystemState {
                emitter: sys.emitter.index(),
                max: e.max_particles,
                gpu: None,
                cpu: CpuParticles::default(),
                seen: 0,
                shift: Vec3::ZERO,
                path: Default::default(),
            });
            if state.emitter != sys.emitter.index() || state.max != e.max_particles {
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
            // Sub-emitters' particles are in the world whatever the parent's space.
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
            // A trail's seconds between samples.
            let interval = e.trail.map(|t| t.interval(tick_rate));
            // The most the force fields accelerate its particles (m/s²).
            let pull = if e.force_fields != 0.0 && e.space == SimulationSpace::World {
                e.force_fields.abs() * fields_peak(&scene.world.force_fields)
            } else {
                0.0
            };
            let subs: Vec<Option<&ParticleEmitter>> = e
                .sub_emitters
                .iter()
                .map(|s| scene.emitters().get(s.emitter.index()))
                .collect();
            if let ParticleRender::Mesh { mesh } = e.render {
                if scene.mesh(mesh).is_none() {
                    return Err(RenderError::InvalidHandle(format!(
                        "emitter {}: mesh {}",
                        sys.emitter.index(),
                        mesh.index()
                    )));
                }
                out.meshes.push(mesh.index());
            }
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
                if let ParticleRender::Mesh { mesh } = ce.render {
                    if scene.mesh(mesh).is_none() {
                        return Err(RenderError::InvalidHandle(format!(
                            "emitter {}: sub-emitter {k}: mesh {}",
                            sys.emitter.index(),
                            mesh.index()
                        )));
                    }
                    out.meshes.push(mesh.index());
                }
            }
            // A host's own simulation of it (its tick read the events).
            let hosted = scene
                .effects
                .hosted_particles
                .iter()
                .find(|(id, _)| *id == sys.id)
                .map(|(_, p)| p);
            // The CPU path bounces world-space particles off the scene's
            // colliders, as compute does where the emitter names them (and
            // in sprite mode); with none to meet they pass through.
            let colliders: &[crate::particles::SceneCollider] =
                if e.collision.is_some() && e.space == SimulationSpace::World {
                    &scene.world.colliders
                } else {
                    &[]
                };
            let meets_colliders = !gpu || e.collides_with_colliders();
            if e.collision.is_some() && meets_colliders && scene.world.colliders.is_empty() {
                out.collision_skipped += 1;
            }
            // Sprite mode: its layer, and an array emitter's array with its
            // frames (the array's layers).
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
            // Sub-emitters' events and a host's own simulation are the CPU
            // path's; sprite systems without them simulate in compute too
            // and the 2D batch draws them from there (`sprite_systems`).
            let gpu = gpu && e.sub_emitters.is_empty() && hosted.is_none();
            if let Some((layer, array)) = sprite
                && !gpu
            {
                match hosted {
                    Some(h) => take_hosted(&mut state.cpu, h, e.space, scene.frame.origin),
                    None => {
                        out.ticks += state.cpu.advance(
                            e,
                            sys,
                            tick_rate,
                            points,
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
                    e,
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
            // In compute: the depth prepass, unless the emitter meets the
            // scene's colliders (`collider_lists` gives it their list).
            let to_colliders = gpu && e.collides_with_colliders();
            let collide =
                gpu && e.collision.is_some() && e.space == SimulationSpace::World && !to_colliders;
            let kind = match sprite {
                Some(_) => DrawKind::Sprite,
                None => draw_kind(e),
            };
            let (effect, material) = effect_of(scene, e, kind, sys.emitter.index())?;
            // Soft particles fade against the depth behind them, where the
            // frame's quality lets them (RenderQuality::soft_particles).
            let soft = if scene.frame.quality.soft_particles && sprite.is_none() {
                e.soft_distance
                    .max(material.map_or(0.0, |m| m.soft_depth))
                    .max(0.0)
            } else {
                0.0
            };
            out.needs_depth |= soft > 0.0 || collide;
            out.soft |= soft > 0.0;
            let look = look_of(
                kind,
                material,
                material.and_then(|m| m.shader).map(|h| h.index()),
            );
            let blend = material.and_then(|m| m.blend).unwrap_or_else(|| e.blend());
            out.distortion |= kind == DrawKind::Distort;
            let mesh = match e.render {
                ParticleRender::Mesh { mesh } => Some(mesh),
                _ => None,
            };
            // The render origin moved: world-space particles on the GPU
            // move with it before this frame's ticks (the CPU's moved above).
            let shift = (gpu && moved != Vec3::ZERO && state.gpu.is_some()).then(|| {
                let mut entry: EmitterGpu = bytemuck::Zeroable::zeroed();
                entry.transform = Mat4::from_translation(moved).to_cols_array_2d();
                entry.info = [e.max_particles, 0, 0, 0];
                entry.trail[0] = e.trail.map_or(0.0, |t| t.lifetime);
                entries.push(entry);
                entries.len() as u32 - 1
            });
            let first_entry = entries.len() as u32;
            if gpu {
                let g = state.gpu.get_or_insert_with(|| {
                    let buffer = crate::traffic::create_buffer(
                        device,
                        &wgpu::BufferDescriptor {
                            label: Some("particles"),
                            size: PARTICLE_STRIDE * e.max_particles as u64,
                            usage: wgpu::BufferUsages::STORAGE
                                | wgpu::BufferUsages::VERTEX
                                | wgpu::BufferUsages::COPY_SRC
                                | wgpu::BufferUsages::COPY_DST,
                            mapped_at_creation: false,
                        },
                    );
                    let points = crate::buffer_init(
                        device,
                        queue,
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("emitter mesh points"),
                            contents: bytemuck::cast_slice(all_points),
                            usage: wgpu::BufferUsages::STORAGE,
                        },
                    );
                    let rows = if e.trail.is_some() {
                        e.max_particles as u64 * TRAIL_POINTS as u64
                    } else {
                        1
                    };
                    let trail = crate::traffic::create_buffer(
                        device,
                        &wgpu::BufferDescriptor {
                            label: Some("particle trails"),
                            size: 16 * rows,
                            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
                            mapped_at_creation: false,
                        },
                    );
                    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("particle sim system"),
                        layout: &self.sim_system_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: points.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: trail.as_entire_binding(),
                            },
                        ],
                    });
                    GpuSystem {
                        buffer,
                        trail,
                        time: 0.0,
                        group,
                        last_tick: None,
                        prev_transform: None,
                        clock: Default::default(),
                    }
                });
                let (reset, steps) = plan_ticks(
                    e,
                    sys,
                    g.last_tick,
                    g.prev_transform,
                    tick_rate,
                    &mut g.clock,
                    history,
                );
                g.last_tick = Some(sys.tick);
                g.prev_transform = Some(sys.transform);
                let live = crate::particles::live_spawns(e, sys, tick_rate, &g.clock);
                let bound = swept_bound(e, sys, tick_rate, &g.clock, &mut state.path, pull);
                let n_points = (points.len() / 2) as u32;
                let local = e.space == SimulationSpace::Local;
                for s in &steps {
                    let mut step = emitter_gpu(
                        e,
                        s.transform,
                        Some(s),
                        s.dt,
                        seed,
                        n_points,
                        textured,
                        collide,
                        soft,
                    );
                    // The air this step's drag closes on, as the CPU path's.
                    step.air =
                        crate::particles::carrying_air(e, sys.air, local.then_some(s.transform))
                            .extend(0.0)
                            .to_array();
                    if let Some(iv) = interval {
                        step.trail[2] = iv;
                        step.trail[3] = crate::particles::trail_sample(s.time, s.dt, iv)
                            .map_or(-1.0, |i| i as f32);
                        g.time = s.time;
                    }
                    entries.push(step);
                }
                let mut draw = emitter_gpu(
                    e,
                    sys.transform,
                    None,
                    dt,
                    seed,
                    n_points,
                    textured,
                    collide,
                    soft,
                );
                newest_sample(&mut draw, interval, g.time);
                entries.push(draw);
                for g in &mut entries[first_entry as usize..] {
                    g.noise_origin = noise_origin;
                    g.field_origin = field_origin;
                }
                if to_colliders {
                    colliding.push((
                        first_entry..entries.len() as u32,
                        reach(e, sys.transform, tick_rate),
                    ));
                }
                out.ticks += steps.len() as u32;
                out.any_gpu = true;
                out.systems.push(FrameSystem {
                    id: sys.id,
                    first_entry,
                    ticks: steps.len() as u32,
                    reset,
                    shift,
                    gpu: true,
                    blend,
                    texture: e.texture,
                    depth,
                    layers,
                    child: None,
                    slots: drawn(e.max_particles, Some(live)),
                    packed: None,
                    bound: Some(bound),
                    kind,
                    mesh,
                    effect,
                    look,
                });
                push_trail(&mut out, e, scene, sys.emitter.index())?;
                if let Some((layer, array)) = sprite {
                    out.sprite_systems.push(SpriteSystem {
                        id: sys.id,
                        draw_entry: first_entry + steps.len() as u32,
                        layer,
                        layers,
                        blend: match e.blend() {
                            ParticleBlend::Additive => crate::SpriteBlend::Additive,
                            // Sprite mode takes no other (validate).
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
            } else {
                match hosted {
                    Some(h) => take_hosted(&mut state.cpu, h, e.space, scene.frame.origin),
                    None => {
                        out.ticks += state.cpu.advance(
                            e,
                            sys,
                            tick_rate,
                            points,
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
                // A hosted system's state may stand at another tick.
                let slots = drawn(
                    state.cpu.slots.len() as u32,
                    stepped.then(|| {
                        crate::particles::live_spawns(e, sys, tick_rate, &state.cpu.clock)
                    }),
                );
                let packed = Packed {
                    particles: pack(&mut self.cpu_bytes, &state.cpu.slots, 1, &slots),
                    trails: match state.cpu.trail.is_empty() {
                        true => [0; 2],
                        false => pack(&mut self.cpu_bytes, &state.cpu.trail, TRAIL_POINTS, &slots),
                    },
                };
                let mut draw =
                    emitter_gpu(e, sys.transform, None, dt, seed, 0, textured, false, soft);
                newest_sample(&mut draw, interval, state.cpu.time);
                entries.push(draw);
                out.systems.push(FrameSystem {
                    id: sys.id,
                    first_entry,
                    ticks: 0,
                    reset: false,
                    shift: None,
                    gpu: false,
                    blend,
                    texture: e.texture,
                    depth,
                    layers,
                    child: None,
                    slots,
                    packed: Some(packed),
                    bound: stepped.then(|| {
                        swept_bound(e, sys, tick_rate, &state.cpu.clock, &mut state.path, pull)
                    }),
                    kind,
                    mesh,
                    effect,
                    look,
                });
                if !state.cpu.trail.is_empty() {
                    push_trail(&mut out, e, scene, sys.emitter.index())?;
                }
            }
            // Its sub-emitters' particles, each drawn as its own emitter
            // draws (in the world).
            for (k, (c, ce)) in state.cpu.children.iter().zip(&subs).enumerate() {
                let Some(ce) = ce else { continue };
                if c.slots.is_empty() {
                    continue;
                }
                let (alive, live) = child_live(c);
                out.cpu_alive += alive;
                let slots = drawn(c.slots.len() as u32, Some(live));
                let packed = Packed {
                    particles: pack(&mut self.cpu_bytes, &c.slots, 1, &slots),
                    trails: [0; 2],
                };
                let kind = draw_kind(ce);
                out.distortion |= kind == DrawKind::Distort;
                let (effect, material) =
                    effect_of(scene, ce, kind, e.sub_emitters[k].emitter.index())?;
                let soft = if scene.frame.quality.soft_particles {
                    ce.soft_distance
                        .max(material.map_or(0.0, |m| m.soft_depth))
                        .max(0.0)
                } else {
                    0.0
                };
                let look = look_of(
                    kind,
                    material,
                    material.and_then(|m| m.shader).map(|h| h.index()),
                );
                out.needs_depth |= soft > 0.0;
                out.soft |= soft > 0.0;
                let first_entry = entries.len() as u32;
                let mut entry = emitter_gpu(
                    ce,
                    Mat4::IDENTITY,
                    None,
                    dt,
                    crate::particles::child_seed(seed, k),
                    0,
                    ce.texture.is_some(),
                    false,
                    soft,
                );
                entry.ranges2[2] = 0.0;
                entry.noise_origin = noise_origin;
                entries.push(entry);
                out.systems.push(FrameSystem {
                    id: sys.id,
                    first_entry,
                    ticks: 0,
                    reset: false,
                    shift: None,
                    gpu: false,
                    blend: material.and_then(|m| m.blend).unwrap_or_else(|| ce.blend()),
                    texture: ce.texture,
                    depth,
                    layers,
                    child: Some(k),
                    slots,
                    packed: Some(packed),
                    bound: None,
                    kind,
                    mesh: match ce.render {
                        ParticleRender::Mesh { mesh } => Some(mesh),
                        _ => None,
                    },
                    effect,
                    look,
                });
            }
            // Its hue, on every entry it drew with (its children's too).
            for g in &mut entries[sys_entries..] {
                g.counts[3] = sys.hue;
            }
        }
        self.effect_meshes(scene, dt, eye, forward, &mut entries, &mut out)?;
        let frame = self.frame;
        self.systems.retain(|_, s| s.seen == frame);
        // The unsupported diagnostic is opaque, not emitted light. Adding
        // it to the background destroys the checker's black and magenta.
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
                .then(a.id.cmp(&b.id))
                .then(a.child.cmp(&b.child))
        });
        // The force fields, for compute systems that take them.
        if out.any_gpu && !scene.world.force_fields.is_empty() {
            let words = pack_fields(&scene.world.force_fields);
            let bytes: &[u8] = bytemuck::cast_slice(&words);
            if bytes.len() as u64 > self.fields.size() {
                self.fields = collider_buffer(device, (bytes.len() as u64).next_power_of_two());
            }
            crate::traffic::write_buffer(queue, &self.fields, 0, bytes);
        } else if out.any_gpu {
            crate::traffic::write_buffer(
                queue,
                &self.fields,
                0,
                bytemuck::cast_slice(&[[0.0f32; 4]]),
            );
        }
        // The scene's colliders near each colliding system, one list each
        // (`emitter.info.z`), in the buffer the simulation reads.
        if !colliding.is_empty() {
            let bounds: Vec<(Vec3, Vec3)> = colliding.iter().map(|(_, b)| *b).collect();
            let (words, lists) = crate::scene_colliders::pack(&scene.world.colliders, &bounds);
            for ((range, _), list) in colliding.iter().zip(lists) {
                for g in &mut entries[range.start as usize..range.end as usize] {
                    g.collision[3] = 1.0;
                    g.info[2] = list;
                }
            }
            let bytes: &[u8] = bytemuck::cast_slice(&words);
            if bytes.len() as u64 > self.colliders.size() {
                self.colliders = collider_buffer(device, (bytes.len() as u64).next_power_of_two());
            }
            crate::traffic::write_buffer(queue, &self.colliders, 0, bytes);
        }
        if !entries.is_empty() {
            let size = EMITTER_STRIDE * entries.len() as u64;
            if size > self.emitter_buffer.size() {
                self.emitter_buffer = crate::traffic::create_buffer(
                    device,
                    &wgpu::BufferDescriptor {
                        label: Some("emitters"),
                        size: size.next_power_of_two(),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    },
                );
            }
            let bytes = &mut self.emitter_bytes;
            bytes.resize(size as usize, 0);
            for (i, e) in entries.iter().enumerate() {
                let at = i * EMITTER_STRIDE as usize;
                bytes[at..at + std::mem::size_of::<EmitterGpu>()]
                    .copy_from_slice(bytemuck::bytes_of(e));
            }
            crate::traffic::write_buffer(queue, &self.emitter_buffer, 0, bytes);
        }
        if !self.cpu_bytes.is_empty() {
            let size = self.cpu_bytes.len() as u64;
            if size > self.cpu_vertices.size() {
                self.cpu_vertices = cpu_vertex_buffer(device, size.next_power_of_two());
            }
            crate::traffic::write_buffer(queue, &self.cpu_vertices, 0, &self.cpu_bytes);
        }
        self.entries = entries;
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
        }
        let mut bytes = vec![0u8; size as usize];
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
                _pad0: 0,
                z: s.z,
                glow: s.glow,
                lit: r.lit,
                _pad1: 0.0,
            };
            let at = k * SPRITE_PARAMS_STRIDE as usize;
            bytes[at..at + std::mem::size_of::<SpriteParamsGpu>()]
                .copy_from_slice(bytemuck::bytes_of(&p));
        }
        crate::traffic::write_buffer(queue, &self.sprites_params, 0, &bytes);
    }

    /// Writes the instances of the sprite-mode systems simulated in compute
    /// into `target`, each range of `ranges` from its system's ring as it
    /// stands after this frame's ticks (after [`ParticleRenderer::simulate`]
    /// in the same encoder).
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
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("particle sprites"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.sprites_pipeline);
        for (k, r) in ranges.ranges.iter().enumerate() {
            let s = &frame.sprite_systems[r.system];
            let Some(g) = self.systems.get(&s.id).and_then(|st| st.gpu.as_ref()) else {
                continue;
            };
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("particle sprites"),
                layout: &self.sprites_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.emitter_buffer,
                            offset: 0,
                            size: wgpu::BufferSize::new(std::mem::size_of::<EmitterGpu>() as u64),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: g.buffer.as_entire_binding(),
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
            pass.set_bind_group(
                0,
                &group,
                &[
                    (u64::from(s.draw_entry) * EMITTER_STRIDE) as u32,
                    (k as u64 * SPRITE_PARAMS_STRIDE) as u32,
                ],
            );
            pass.dispatch_workgroups(r.count.div_ceil(64), 1, 1);
        }
    }
}

/// Appends the rows of `ranges`' slots (`per` items a slot) to the
/// frame's CPU vertices; where each range starts (bytes). Only the slots
/// a system draws are sent ([`drawn`]).
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

/// A sub-emitter's living particles and the births they are among (for
/// [`drawn`]): from its oldest living one's to the next one's. Its ring
/// fills in birth order, so slot `s` holds the latest birth `n` with
/// `n % max == s`.
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

/// A host's own simulation of a system as the frame draws it: its slots
/// and its sub-emitters' moved from the host's world to the scene's render
/// origin (`origin.rs`).
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
        }
        for c in &mut cpu.children {
            back(&mut c.slots);
        }
    }
}

/// Sprite-mode particles as sprites (alive slots, in slot order); with
/// `array` (the array and its frames), each from the layer its frame names.
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
        // Sprite mode takes no other (validate).
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

/// The effect material an emitter's particles of `kind` draw with: only
/// billboards and meshes take one (distortion and sprite mode keep their
/// own look). A handle of no material of the scene is refused.
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
fn newest_sample(entry: &mut EmitterGpu, interval: Option<f32>, time: f32) {
    if let Some(iv) = interval {
        let (i, age) = crate::particles::trail_newest(time, iv);
        entry.trail[2] = iv;
        entry.trail[3] = i as f32;
        entry.counts3[2] = age;
    }
}

/// With a trail, the system `out` just took draws its particles' trails
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
        // Still simulated as it was; drawn as its trails.
        last.kind = DrawKind::Trail;
        last.mesh = None;
        (last.effect, last.look) = (effect, look);
        return Ok(());
    }
    let trail = FrameSystem {
        id: last.id,
        first_entry: last.first_entry + last.ticks,
        ticks: 0,
        reset: false,
        shift: None,
        gpu: last.gpu,
        blend: last.blend,
        texture: last.texture,
        depth: last.depth,
        layers: last.layers,
        child: None,
        slots: last.slots.clone(),
        packed: last.packed,
        bound: last.bound,
        kind: DrawKind::Trail,
        mesh: None,
        effect,
        look,
    };
    out.systems.push(trail);
    Ok(())
}

/// The key of an effect mesh's state among the systems': its id with the
/// top bit set, apart from a particle system on the same entity.
const EFFECT_MESH_KEY: u64 = 1 << 63;

impl ParticleRenderer {
    /// The scene's effect meshes as one-particle mesh systems: their
    /// particle uploaded with the mesh's life and random value, their
    /// entry placing the mesh whole (`emitter.look.z` 2).
    fn effect_meshes(
        &mut self,
        scene: &RenderScene,
        dt: f32,
        eye: Vec3,
        forward: Vec3,
        entries: &mut Vec<EmitterGpu>,
        out: &mut ParticleFrame,
    ) -> Result<(), RenderError> {
        for (i, m) in scene.effects.effect_meshes.iter().enumerate() {
            let material = scene.effect_material(m.material).ok_or_else(|| {
                RenderError::InvalidHandle(format!(
                    "effect mesh {i}: effect material {}",
                    m.material.index()
                ))
            })?;
            if scene.mesh(m.mesh).is_none() {
                return Err(RenderError::InvalidHandle(format!(
                    "effect mesh {i}: mesh {}",
                    m.mesh.index()
                )));
            }
            out.meshes.push(m.mesh.index());
            let seed = hash_seed(m.seed);
            // Alive whatever its life (a particle at its lifetime is dead).
            let particle = ParticleState {
                pos_age: [0.0, 0.0, 0.0, m.life.clamp(0.0, 1.0) * 0.99999],
                vel_life: [0.0, 0.0, 0.0, 1.0],
                misc: [1.0, 0.0, 0.0, crate::particles::hash_unit(seed)],
            };
            let key = m.id | EFFECT_MESH_KEY;
            let state = self.systems.entry(key).or_insert_with(|| SystemState {
                emitter: usize::MAX,
                max: 1,
                gpu: None,
                cpu: CpuParticles::default(),
                seen: 0,
                shift: Vec3::ZERO,
                path: Default::default(),
            });
            state.seen = self.frame;
            let packed = Packed {
                particles: pack(
                    &mut self.cpu_bytes,
                    std::slice::from_ref(&particle),
                    1,
                    &drawn(1, None),
                ),
                trails: [0; 2],
            };
            let soft = if scene.frame.quality.soft_particles {
                material.soft_depth.max(0.0)
            } else {
                0.0
            };
            out.needs_depth |= soft > 0.0;
            out.soft |= soft > 0.0;
            let look = ParticleEmitter {
                render: ParticleRender::Mesh { mesh: m.mesh },
                size_over_life: crate::particles::Curve::constant(1.0),
                color_over_life: crate::particles::ColorCurve::constant([1.0; 4]),
                color: m.color,
                gravity: Vec3::ZERO,
                ..Default::default()
            };
            let first_entry = entries.len() as u32;
            let mut entry = emitter_gpu(&look, m.transform, None, dt, seed, 0, false, false, soft);
            entry.look[2] = 2.0;
            entry.counts[3] = m.hue;
            entry.axis[3] = m.billboard as u32 as f32;
            entries.push(entry);
            out.systems.push(FrameSystem {
                id: key,
                first_entry,
                ticks: 0,
                reset: false,
                shift: None,
                gpu: false,
                blend: material.blend.unwrap_or(m.blend),
                texture: None,
                depth: (m.transform.w_axis.truncate() - eye).dot(forward),
                layers: m.layers,
                child: None,
                slots: drawn(1, None),
                packed: Some(packed),
                bound: None,
                kind: DrawKind::Mesh,
                mesh: Some(m.mesh),
                effect: Some(m.material),
                look: look_of(
                    DrawKind::Mesh,
                    Some(material),
                    material.shader.map(|h| h.index()),
                ),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sub_emitters_draw_covers_its_living_particles_across_the_wrap() {
        // Eight slots after 13 births: slots 0..5 hold births 8..13, slots
        // 5..8 births 5..8. Births 5 and 6 (slots 5, 6) are dead.
        let living = |alive: bool| ParticleState {
            pos_age: [0.0, 0.0, 0.0, if alive { 0.0 } else { 2.0 }],
            vel_life: [0.0, 0.0, 0.0, 1.0],
            misc: [0.0; 4],
        };
        let c = crate::particles::ChildParticles {
            slots: (0..8).map(|s| living(s != 5 && s != 6)).collect(),
            born: 13,
        };
        let (alive, live) = child_live(&c);
        assert_eq!((alive, live.clone()), (6, 7..13));
        assert_eq!(drawn(8, Some(live)), [0..5, 7..8]);
        let none = crate::particles::ChildParticles {
            slots: vec![living(false); 8],
            born: 3,
        };
        assert_eq!(drawn(8, Some(child_live(&none).1)), [3..3, 0..0]);
    }
}
