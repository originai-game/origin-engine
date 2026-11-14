//! A frame's grass: each field's chunks planned, generated and uploaded.

use super::*;

impl GrassRenderer {
    /// A field's buffers holding at least `slices` slices (made again,
    /// larger, when not).
    fn field_gpu(&mut self, device: &wgpu::Device, slot: usize, slices: u32) {
        if self.fields.get(slot).is_some_and(|f| f.slice_cap >= slices) {
            return;
        }
        let slice_cap = slices.next_power_of_two().clamp(64, MAX_SLICES);
        let blade_cap = (slice_cap * SLICE).min(MAX_BLADES);
        let make = |label, size: u64, usage| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage,
                mapped_at_creation: false,
            })
        };
        let storage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        let uniform = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
        let generator = make(
            "grass generate",
            std::mem::size_of::<GenGpu>() as u64,
            uniform,
        );
        let draw = make("grass", std::mem::size_of::<DrawGpu>() as u64, uniform);
        let slices_buf = make("grass slices", slice_cap as u64 * 16, storage);
        let scratch = make(
            "grass scratch",
            slice_cap as u64 * SLICE as u64 * 16,
            wgpu::BufferUsages::STORAGE,
        );
        let counts = make(
            "grass counts",
            slice_cap as u64 * 12,
            wgpu::BufferUsages::STORAGE,
        );
        let instances = make(
            "grass blades",
            (blade_cap as u64 + 1) * 16,
            wgpu::BufferUsages::STORAGE,
        );
        let draws = make(
            "grass draws",
            60,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT,
        );
        let group = |label, layout, buffers: &[&wgpu::Buffer]| {
            let entries: Vec<_> = buffers
                .iter()
                .enumerate()
                .map(|(i, b)| wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource: b.as_entire_binding(),
                })
                .collect();
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout,
                entries: &entries,
            })
        };
        let mut gen_entries: Vec<_> = [
            &generator,
            &self.tile,
            &slices_buf,
            &scratch,
            &counts,
            &instances,
            &draws,
        ]
        .iter()
        .enumerate()
        .map(|(i, b)| wgpu::BindGroupEntry {
            binding: i as u32,
            resource: b.as_entire_binding(),
        })
        .collect();
        gen_entries.push(wgpu::BindGroupEntry {
            binding: 7,
            resource: wgpu::BindingResource::TextureView(&self.wind_field),
        });
        gen_entries.push(wgpu::BindGroupEntry {
            binding: 8,
            resource: wgpu::BindingResource::TextureView(&self.trample),
        });
        let gen_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("grass generate"),
            layout: &self.gen_layout,
            entries: &gen_entries,
        });
        let draw_group = group("grass", &self.draw_layout, &[&draw, &instances]);
        let f = FieldGpu {
            generator,
            draw,
            slices: slices_buf,
            draws,
            slice_cap,
            blade_cap,
            gen_group,
            draw_group,
        };
        if slot < self.fields.len() {
            self.fields[slot] = f;
        } else {
            self.fields.push(f);
        }
    }

    /// Plans this frame's fields over the drawn terrains and writes their
    /// uniforms and slices.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare<'t>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &crate::RenderScene,
        terrain: &dyn GroundGpu,
        terrain_draws: &[impl GroundDraw],
        view: GrassView,
        frustum: &crate::frame::Frustum,
        textures: &dyn Fn(crate::TextureHandle) -> Option<&'t wgpu::TextureView>,
    ) -> GrassFrame {
        let mut frame = GrassFrame::default();
        let previous = self.last_tick.unwrap_or(scene.frame.tick);
        self.last_tick = (!scene.world.grass.is_empty()).then_some(scene.frame.tick);
        let wind = crate::wind::field_gpu(
            scene.atmosphere.wind,
            scene.frame.tick,
            previous,
            scene.frame.tick_rate,
            scene.frame.origin,
            view.wind_grid,
        );
        let l = scene
            .lighting
            .light
            .direction_to_light
            .normalize_or(Vec3::Y);
        let sun_on = scene.lighting.light.intensity > 0.0;
        // Where a blade's shadow lands, per metre of its height.
        let reach = Vec3::new(-l.x, -l.y, -l.z) / l.y.max(0.2);
        let lod_rows = LOD_ROWS;
        for d in &scene.world.grass {
            let g = d.grass;
            let Some(t) = scene.world.terrains.get(d.terrain) else {
                continue;
            };
            let Some(k) = terrain_draws
                .iter()
                .position(|td| td.terrain() == d.terrain)
            else {
                continue;
            };
            let td = &terrain_draws[k];
            let Some((size, range)) = terrain.extent_of(t.terrain.index()) else {
                continue;
            };
            if !g.usable() || (g.layer as usize) >= t.layers.len().max(1) {
                continue;
            }
            let Some(terrain_layout) = terrain.group_layout(td.arrayed()) else {
                continue;
            };
            let a = usize::from(td.arrayed());
            if self.generate[a].is_none() {
                self.generate[a] = Some(self.make_generate(device, terrain_layout, td.arrayed()));
            }
            let cast = g.cast_shadows && scene.lighting.light.cast_shadows && sun_on;
            // With its shadow: the box swept along the sun's reach.
            let shadow = reach * g.tallest() * f32::from(u8::from(cast));
            let visible = |lo: Vec3, hi: Vec3| {
                frustum.intersects_box(lo.min(lo + shadow), hi.max(hi + shadow))
            };
            let Plan { slices, density } = plan(
                &g,
                &PlanView {
                    model: t.transform,
                    size,
                    range,
                    camera: view.camera,
                    orthographic: view.frame.orthographic,
                    visible: &visible,
                },
            );
            if slices.is_empty() {
                continue;
            }
            let slot = frame.fields.len();
            self.field_gpu(device, slot, slices.len() as u32);
            let f = &self.fields[slot];
            let along = horizontal_scale(t.transform);
            let layout = FieldLayout::new(&g, along);
            let lod = DistanceLod::for_radius(g.distance);
            let extent = g.distance + g.tallest() * 4.0 + CHUNK;
            let origin = view.camera.extend(extent).to_array();
            let planes = frustum.planes();
            let generator = GenGpu {
                planes: std::array::from_fn(|i| {
                    planes.get(i).copied().unwrap_or(Vec4::ZERO).to_array()
                }),
                eye: view
                    .camera
                    .extend(f32::from(u8::from(view.frame.orthographic)))
                    .to_array(),
                origin,
                lod: [lod.lod_start[0], lod.lod_start[1], lod.band, lod.radius],
                lod_density: [
                    lod.density_at[0],
                    lod.density_at[1],
                    lod.mid_range,
                    lod.fade_start,
                ],
                field: [
                    layout.candidates,
                    layout.density,
                    layout.tile,
                    layout.tiles_per_side as f32,
                ],
                chunk: [layout.chunk, g.layer as f32, seed_bits(g.seed), lod.widest],
                shape: [
                    g.tallest(),
                    g.height_variation.clamp(0.0, 1.0),
                    g.lean,
                    g.clump_lean,
                ],
                clump: [
                    g.clump_size.max(0.05) / along,
                    g.tufts.clamp(0.0, 1.0),
                    HEIGHT_RANGE,
                    g.widest(),
                ],
                shadow: reach.extend(f32::from(u8::from(cast))).to_array(),
                motion: [
                    g.sway.max(0.0),
                    g.stiffness.clamp(0.0, 1.0),
                    density,
                    g.recovery.max(0.0),
                ],
                trample: view.trample,
                kinds: kinds_gen(&g),
                maps: [
                    f32::from(u8::from(g.type_map.is_some())),
                    f32::from(u8::from(g.density_map.is_some())),
                    f32::from(u8::from(g.height_map.is_some())),
                    0.0,
                ],
                counts: [
                    slices.len() as u32,
                    f.blade_cap,
                    lod_rows[0],
                    lod_rows[1] | (lod_rows[2] << 8),
                ],
                wind,
            };
            let proj = view.frame.proj;
            let per_pixel = 2.0 / (proj.y_axis.y.abs().max(1e-6) * view.height.max(1) as f32);
            let [base_color, tip_color, dry_color, variation] = look_gpu(&g);
            let draw = DrawGpu {
                view_proj: view.frame.view_proj.to_cols_array_2d(),
                prev_view_proj: view.prev_view_proj.to_cols_array_2d(),
                eye: generator.eye,
                forward: view.frame.forward.extend(per_pixel).to_array(),
                origin,
                limits: [
                    HEIGHT_RANGE,
                    MIN_WIDTH_PX,
                    TURN_TO_EYE,
                    // Without the weather on surfaces no snow lies on it.
                    if self
                        .off
                        .contains(crate::pipeline_features::Features::WEATHER)
                    {
                        0.0
                    } else {
                        snow_weight(scene)
                    },
                ],
                sun: l.extend(f32::from(u8::from(sun_on))).to_array(),
                shape: [
                    g.tallest(),
                    g.widest(),
                    g.curve.clamp(0.0, 1.0),
                    g.flutter.max(0.0),
                ],
                base_color,
                tip_color,
                dry_color,
                variation,
                rows: [lod_rows[0], lod_rows[1], lod_rows[2], 0],
                // The canopy the sun crosses to reach a blade's lower part
                // (grass.wgsl grass_canopy).
                canopy: {
                    let [extinction, top] = canopy(&g);
                    [extinction, top, 0.0, 0.0]
                },
                kinds: kinds_draw(&g),
                wind,
            };
            queue.write_buffer(&f.generator, 0, bytemuck::bytes_of(&generator));
            queue.write_buffer(&f.draw, 0, bytemuck::bytes_of(&draw));
            queue.write_buffer(&f.slices, 0, bytemuck::cast_slice(&slices));
            let view_of =
                |m: Option<crate::TextureHandle>| m.and_then(textures).unwrap_or(&self.white);
            let maps = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("grass maps"),
                layout: &self.maps_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(view_of(g.type_map)),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(view_of(g.density_map)),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(view_of(g.height_map)),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&self.map_sampler),
                    },
                ],
            });
            frame.fields.push(GrassField {
                slot,
                maps,
                terrain_draw: k,
                arrayed: td.arrayed(),
                slices: slices.len() as u32,
                cast,
                // A level starts half a dither band before its distance;
                // a blade there reaches up to its length towards the
                // camera and throws its shadow `shadow` further.
                shades_from: lod.lod_start.map(|d| {
                    let blade = g.tallest() * (1.0 + g.lean.abs() + g.clump_lean.abs());
                    d - lod.band * 0.5 - blade - shadow.length()
                }),
            });
        }
        if !frame.fields.is_empty() && self.pipelines.is_none() {
            self.pipelines = Some(self.make_pipelines(device));
        }
        frame
    }
}
