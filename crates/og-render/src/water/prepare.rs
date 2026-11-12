use super::*;

impl WaterRenderer {
    /// Writes this frame's uniforms and height fields and makes the bind
    /// groups (the scene copy and the depth prepass are this frame's;
    /// `planar`: the surfaces drawn with this frame's planar reflections and
    /// their textures, `fallback` bound where there is none).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare<'t>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        waters: &[Water],
        screen: (u32, u32),
        scene_color: &wgpu::TextureView,
        scene_depth: &wgpu::TextureView,
        planar: &[(usize, &wgpu::TextureView)],
        ao: &wgpu::TextureView,
        ao_direct: f32,
        adaptation: f32,
        origin: Vec3,
        seconds: f64,
        camera: &crate::Camera,
        quality: &crate::RenderQuality,
        fallback: &wgpu::TextureView,
        caustic_grid: &wgpu::Buffer,
        mesh_buffers: &dyn Fn(
            crate::scene::MeshHandle,
        ) -> Option<(wgpu::Buffer, wgpu::Buffer, u32)>,
        textures: &dyn Fn(crate::TextureHandle) -> Option<&'t wgpu::TextureView>,
    ) -> Vec<WaterDraw> {
        while self.slots.len() < waters.len() {
            self.slots.push(Slot {
                uniform: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("water"),
                    size: std::mem::size_of::<WaterGpu>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                face: bytemuck::Zeroable::zeroed(),
                heights: None,
                ripples: None,
            });
        }
        let (w, h) = (screen.0 as f32, screen.1 as f32);
        // The seabeds of this frame's surfaces, each uploaded once.
        let fields: Vec<&Arc<og_core::ocean::Shore>> = waters
            .iter()
            .filter_map(|w| w.shore.as_ref().map(|s| &s.field))
            .collect();
        self.shores
            .retain(|(f, _)| fields.iter().any(|g| Arc::ptr_eq(f, g)));
        for f in fields {
            if !self.shores.iter().any(|(g, _)| Arc::ptr_eq(f, g)) {
                let (_, view) = shore_texture(device, queue, f);
                self.shores.push((f.clone(), view));
            }
        }
        // W5: the rivers' ribbons, each built once for its course and frame.
        self.rivers.retain(|m| {
            waters
                .iter()
                .any(|w| w.river.as_ref().is_some_and(|r| m.fits(r, w.transform)))
        });
        for w in waters {
            if let Some(r) = &w.river
                && !self.rivers.iter().any(|m| m.fits(r, w.transform))
            {
                self.rivers.push(crate::water_river::RiverMesh::new(
                    device,
                    queue,
                    r.clone(),
                    w.transform,
                ));
            }
        }
        let (sea_segments, sea_cascades) = sea_tier(quality);
        self.submitted = (0, 0);
        self.wake_centre = None;
        let sea = self.fft_sea(
            device,
            queue,
            waters,
            sea_cascades,
            seconds,
            camera,
            origin,
            caustic_grid,
        );
        let levels = clipmap(camera.position, origin, sea_segments);
        let line = waterline(waters, camera, seconds, origin);
        let mut draws = Vec::with_capacity(waters.len());
        self.last_sea = None;
        for (index, (water, slot)) in waters.iter().zip(&mut self.slots).enumerate() {
            let reflection = planar.iter().find(|(i, _)| *i == index).map(|(_, v)| *v);
            let segments = grid_segments(water);
            let river = water
                .river
                .as_ref()
                .and_then(|r| self.rivers.iter().find(|m| m.fits(r, water.transform)));
            let mesh = water
                .mesh
                .and_then(mesh_buffers)
                .or_else(|| river.map(|m| (m.vertices.clone(), m.indices.clone(), m.count)));
            let field = height_field(water).filter(|_| mesh.is_none());
            if let Some(f) = field {
                if slot
                    .heights
                    .as_ref()
                    .is_none_or(|(r, ..)| *r != f.resolution)
                {
                    let (t, v) = height_texture(device, f.resolution);
                    slot.heights = Some((f.resolution, t, v));
                }
                let (_, texture, _) = slot.heights.as_ref().expect("made above");
                let texels: Vec<[u16; 4]> =
                    f.heights.iter().map(|&h| [f16_bits(h), 0, 0, 0]).collect();
                queue.write_texture(
                    texture.as_image_copy(),
                    bytemuck::cast_slice(&texels),
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(f.resolution * 8),
                        rows_per_image: None,
                    },
                    wgpu::Extent3d {
                        width: f.resolution,
                        height: f.resolution,
                        depth_or_array_layers: 1,
                    },
                );
            }
            // The ripples over the heights, stepped to this frame's tick.
            match field.and_then(|f| Some((f, water.wakes.as_deref()?))) {
                Some((f, wakes)) if self.compute => {
                    let pipes = self
                        .ripple_pipelines
                        .get_or_insert_with(|| crate::water_ripples::RipplePipelines::new(device));
                    if slot.ripples.as_ref().is_none_or(|r| !r.fits(f)) {
                        // The compute reads the heights' one layer as a plain 2D texture.
                        let (_, texture, _) = slot.heights.as_ref().expect("made above");
                        let coarse = texture.create_view(&Default::default());
                        slot.ripples = Some(crate::water_ripples::Ripples::new(
                            device, pipes, f, &coarse,
                        ));
                    }
                    let ripples = slot.ripples.as_mut().expect("made above");
                    ripples.update(device, queue, pipes, wakes);
                    self.submitted.0 += 1;
                }
                _ => slot.ripples = None,
            }
            let shore_view = shore_view(&self.shores, &self.no_shore, water);
            let normal_view = water.normal_map.and_then(textures);
            // A river's flow map is its current, its length the river's speed.
            let flow_view = match river {
                Some(m) => Some(&m.flow),
                None => water.flow_map.and_then(textures),
            };
            // With a flow map the flow's length is the speed: three.js'
            // flowSpeed 0.03 for a zero flow.
            let flow = match river {
                Some(m) => Vec2::new(m.river.speed.max(1e-3), 0.0),
                None if flow_view.is_some() && water.flow.length_squared() < 1e-12 => {
                    Vec2::new(0.03, 0.0)
                }
                None => water.flow,
            };
            let (mut ocean, swell) = ocean_gpu(water, seconds, origin);
            let drawn_sea = sea.filter(|_| ocean[0] > 0.0);
            if let Some(f) = drawn_sea {
                ocean[2] = f.foam.fresh;
                ocean[3] = f.foam.gains[2];
            }
            let gpu = WaterGpu {
                model: water.transform.to_cols_array_2d(),
                size: [
                    water.size.x.max(0.0),
                    water.size.y.max(0.0),
                    segments as f32,
                    // 2: the ripples' heights, read per pixel.
                    match (field, &slot.ripples) {
                        (Some(_), Some(_)) => 2.0,
                        (Some(_), None) => 1.0,
                        (None, _) => 0.0,
                    },
                ],
                shallow: water.shallow.extend(water.depth_fade.max(1e-3)).to_array(),
                deep: water.deep.extend(water.opacity.clamp(0.0, 1.0)).to_array(),
                foam: water.foam.extend(water.foam_width.max(0.0)).to_array(),
                waves: [
                    water.wave_scale.max(0.0),
                    water.wave_speed,
                    water.wave_strength.max(0.0),
                    water.refraction.max(0.0),
                ],
                look: [
                    water.reflectivity.max(0.0),
                    // The screen-space march on (water.wgsl).
                    1.0,
                    field.map_or(1.0, |f| f.size),
                    field.map_or(2.0, |f| match slot.ripples {
                        Some(_) => crate::water_ripples::Ripples::side(f.resolution) as f32,
                        None => f.resolution as f32,
                    }),
                ],
                screen: [w, h, 1.0 / w.max(1.0), 1.0 / h.max(1.0)],
                extra: [
                    water.shadows.clamp(0.0, 1.0),
                    f32::from(reflection.is_some()),
                    water.f0.clamp(0.0, 1.0),
                    water.reflection_bias.clamp(0.0, 1.0),
                ],
                under: water
                    .underwater_color
                    .extend(water.glint.max(0.0))
                    .to_array(),
                normals: [
                    water.normal_scale.max(0.0),
                    flow.x,
                    flow.y,
                    f32::from(normal_view.is_some()),
                ],
                ao: [
                    ao_direct.clamp(0.0, 1.0),
                    f32::from(flow_view.is_some()),
                    f32::from(mesh.is_some()),
                    1.0 / adaptation.max(1e-3),
                ],
                layers: lake_layers(origin, water.normal_scale.max(0.0)),
                ocean,
                sea_wind: {
                    let d = water.wind.normalize_or_zero();
                    [d.x, d.y, water.wind.length() * ocean[0], water.foam_streaks]
                },
                clip: [
                    CLIP_LEVELS as f32,
                    sea_segments as f32,
                    camera.far * 0.95,
                    f32::from(water.endless),
                ],
                levels,
                inverse: water.transform.inverse().to_cols_array_2d(),
                fft: match drawn_sea {
                    Some(f) => [
                        f.tiles.iter().filter(|t| **t > 0.0).count() as f32,
                        0.0,
                        f.tail,
                        self::ocean(water).map_or(0.0, |o| o.significant_height()),
                    ],
                    None => [0.0; 4],
                },
                bands: drawn_sea.map_or([0.0; 4], |f| f.bands),
                cascades: cascade_uv(drawn_sea, origin),
                foam_levels: match drawn_sea {
                    Some(_) => crate::ocean_fft::foam::levels(glam::DVec2::new(
                        f64::from(origin.x),
                        f64::from(origin.z),
                    )),
                    None => [[0.0; 4]; 2],
                },
                foam_uv: foam_uv(origin, water.wind, seconds),
                swell,
                shore: shore_gpu(water, seconds),
                spray: spray_gpu(water, drawn_sea.is_some(), seconds, quality),
                waterline: [
                    f32::from(line == Some(index)),
                    water.underwater_density.max(0.0),
                    water.meniscus.max(0.0),
                    f32::from(river.is_some()),
                ],
                wake: match self.wake_centre {
                    Some(c) if drawn_sea.is_some() && water.dynamic_waves > 0.0 => {
                        let side = crate::sea_wakes::SIDE as f32 * crate::sea_wakes::CELL;
                        [c.x, c.y, 1.0 / side, 1.0]
                    }
                    _ => [0.0; 4],
                },
                optics: {
                    let on = f32::from(
                        water.absorption.max_element() > 0.0
                            || water.scattering.max_element() > 0.0,
                    );
                    [
                        water.absorption.max(Vec3::ZERO).extend(on).to_array(),
                        water.scattering.max(Vec3::ZERO).extend(on).to_array(),
                    ]
                },
            };
            queue.write_buffer(&slot.uniform, 0, bytemuck::bytes_of(&gpu));
            // A probe face has no planar reflection of its own (that would be
            // a mirrored pass per face); its screen-space reflection marches
            // the face's own depth prepass and copy, as the frame's does,
            // mirroring what the probe sees from its point.
            slot.face = gpu;
            slot.face.extra[1] = 0.0;
            let wake_view = match &self.wakes {
                Some(w) if gpu.wake[3] > 0.5 => &w.view,
                _ => &self.no_wake,
            };
            let heights = match (&slot.ripples, &slot.heights, field, &self.fft) {
                (_, _, _, Some((fft, _))) if drawn_sea.is_some() => &fft.view,
                (Some(r), _, Some(_), _) => &r.view,
                (None, Some((_, _, v)), Some(_), _) => v,
                _ => &self.flat,
            };
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("water"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: slot.uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(heights),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(scene_color),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(scene_depth),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::TextureView(
                            reflection.unwrap_or(fallback),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::TextureView(
                            normal_view.unwrap_or(fallback),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::Sampler(&self.ripple_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: wgpu::BindingResource::TextureView(ao),
                    },
                    wgpu::BindGroupEntry {
                        binding: 9,
                        resource: wgpu::BindingResource::TextureView(flow_view.unwrap_or(fallback)),
                    },
                    wgpu::BindGroupEntry {
                        binding: 10,
                        resource: wgpu::BindingResource::TextureView(shore_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 11,
                        resource: wgpu::BindingResource::TextureView(wake_view),
                    },
                ],
            });
            if drawn_sea.is_some() && self.last_sea.is_none() {
                self.last_sea = Some(group.clone());
            }
            let half = Vec3::new(water.size.x, 0.0, water.size.y) * 0.5;
            let radius = water.transform.transform_vector3(half).length();
            draws.push(WaterDraw {
                group,
                vertices: if drawn_sea.is_some() {
                    sea_vertices(sea_segments)
                } else {
                    segments * segments * 6
                },
                bounds: (water.transform.transform_point3(Vec3::ZERO), radius),
                mesh,
                spray: if gpu.spray[0] > 0.0 {
                    2 * SPRAY_SLOTS
                } else {
                    0
                },
                waterline: line == Some(index),
            });
        }
        draws
    }
}

/// The lake's four layers' uv at `origin` (water_detail.wgsl `map_slope`: world
/// xz times the repeats over each layer's period), in f64 and without the
/// whole repeats: zero at the world's zero.
fn lake_layers(origin: Vec3, repeats: f32) -> [[f32; 4]; 2] {
    let k = f64::from(repeats);
    let at = |o: f32, period: f64| (f64::from(o) * k / period).rem_euclid(1.0) as f32;
    let (x, z) = (origin.x, origin.z);
    [
        [at(x, 103.0), at(z, 103.0), at(x, 107.0), at(z, 107.0)],
        [at(x, 8907.0), at(z, 9803.0), at(x, 1091.0), at(z, 1027.0)],
    ]
}
