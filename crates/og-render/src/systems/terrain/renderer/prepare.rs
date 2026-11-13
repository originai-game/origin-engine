//! A frame's terrain: the blocks each view draws and what they read.

use super::*;

impl TerrainRenderer {
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
        if scene.world.terrains.is_empty() {
            self.uploads = 0;
            self.resident = 0;
            return Ok(Vec::new());
        }
        let view = TerrainView {
            camera,
            far,
            ..TerrainView::of(&scene.frame.camera, target_height)
        };
        self.sync(device, queue, scene, view)?;
        // Each terrain's clipmap from this view, and the ring sizes they draw
        // with in the index buffer (built again, all of them one after
        // another, when a new size is needed: rarely, ring sizes being few).
        let clipmaps: Vec<Option<TerrainClipmap>> = (0..scene.world.terrains.len())
            .map(|i| scene.terrain_clipmap(i, &view))
            .collect();
        let missing = |have: &[(u32, RingRanges)]| {
            clipmaps
                .iter()
                .flatten()
                .any(|c| !have.iter().any(|(n, _)| *n == c.ring))
        };
        if self.indices.as_ref().is_none_or(|(_, have)| missing(have)) {
            let mut rings: Vec<u32> = self
                .indices
                .as_ref()
                .map(|(_, have)| have.iter().map(|(n, _)| *n).collect())
                .unwrap_or_default();
            rings.extend(clipmaps.iter().flatten().map(|c| c.ring));
            rings.sort_unstable();
            rings.dedup();
            let mut idx = Vec::new();
            let mut have = Vec::new();
            for n in rings {
                let (i, r) = ring_indices(n);
                have.push((n, r.shifted(idx.len() as u32)));
                idx.extend(i);
            }
            let buffer = crate::buffer_init(
                device,
                queue,
                &wgpu::util::BufferInitDescriptor {
                    label: Some("terrain indices"),
                    contents: bytemuck::cast_slice(&idx),
                    usage: wgpu::BufferUsages::INDEX,
                },
            );
            self.indices = Some((buffer, have));
        }
        while self.slots.len() < scene.world.terrains.len() {
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
                corners: None,
            });
        }
        let mut draws = Vec::with_capacity(scene.world.terrains.len());
        for (ti, t) in scene.world.terrains.iter().enumerate() {
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
                        offset: layer_offset(scene.frame.origin, l),
                        ..layer_look(l, scene.frame.origin)
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
                    offset: layer_offset(scene.frame.origin, l),
                    ..layer_look(l, scene.frame.origin)
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
                    f32::from(t.horizon_shadows),
                ],
                tiles: [
                    lay.tile as f32,
                    lay.control as f32,
                    lay.levels as f32,
                    lay.pages.max(1) as f32,
                ],
                displace: {
                    let across = horizontal_scale(model);
                    let up = model.y_axis.truncate().length().max(1e-6);
                    let reach = if t.displaces() {
                        t.displacement_distance / across
                    } else {
                        0.0
                    };
                    let lift = if self
                        .off
                        .contains(crate::pipeline_features::Features::WEATHER)
                    {
                        0.0
                    } else {
                        crate::terrain::snow_lift(scene)
                    };
                    [reach, up, up / across, lift]
                },
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
            let clipmap = clipmaps[ti].clone().expect("a drawn terrain is laid out");
            let ranges = &self
                .indices
                .as_ref()
                .expect("made above")
                .1
                .iter()
                .find(|(n, _)| *n == clipmap.ring)
                .expect("its ring is in the buffer")
                .1;
            let unit = clipmap.unit;
            let lo = -lay.size * 0.5;
            let cam = clipmap.camera;
            let n = clipmap.ring;
            let list = clipmap.levels;
            // The camera's height in the terrain's space (the displacement's reach).
            let eye_y = model.inverse().transform_point3(camera).y;
            let morph_width = clipmap.morph_width;
            let corner_count = (n + 1) * (n + 1);
            let mut level_bytes = vec![0u8; LEVEL_STRIDE as usize * list.len().max(1)];
            let mut out = Vec::with_capacity(list.len());
            let (hmin, hmax) = data.range;
            let resident = resident_boxes(data);
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
                        l.morph_start,
                        if l.morph { morph_width } else { 0.0 },
                        0.0,
                    ],
                    camera: [cam.x, cam.y, unit, eye_y],
                    corners: {
                        let coarse = list.get(k + 1).map_or([0.0; 2], |c| c.center);
                        [
                            coarse[0] as f32,
                            coarse[1] as f32,
                            (k as u32 * corner_count) as f32,
                            ((k as u32 + 1).min(list.len() as u32 - 1) * corner_count) as f32,
                        ]
                    },
                };
                let at = k * LEVEL_STRIDE as usize;
                level_bytes[at..at + std::mem::size_of::<LevelGpu>()]
                    .copy_from_slice(bytemuck::bytes_of(&g));
                let ring = match l.hole {
                    None => &ranges.variants[0],
                    Some((ox, oz)) => &ranges.variants[1 + (ox + 2 * oz) as usize],
                };
                let blocks = level_blocks(l, n, unit, lay.size, (hmin, hmax))
                    .into_iter()
                    .zip(&ring.blocks)
                    .map(|((mut a, mut b), r)| {
                        (a.y, b.y) = block_range(&resident, a, b).unwrap_or((hmin, hmax));
                        let (lo, hi) = world_box(model, a, b);
                        // A block wholly off the terrain clamps to its
                        // edge: zero-area triangles, never drawn.
                        let off = a.x >= b.x || a.z >= b.z;
                        (if off { r.start..r.start } else { r.clone() }, lo, hi)
                    })
                    .collect();
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
                    center,
                    radius,
                    blocks,
                    skirt: l.morph.then(|| ranges.skirt.clone()),
                });
            }
            queue.write_buffer(&slot.levels, 0, &level_bytes);
            // The corner cache: every level's corners, 16 bytes each.
            let needed = u64::from(corner_count) * list.len().max(1) as u64;
            if slot
                .corners
                .as_ref()
                .is_none_or(|(have, ..)| *have < needed)
            {
                let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("terrain corners"),
                    size: needed * 16,
                    usage: wgpu::BufferUsages::STORAGE,
                    mapped_at_creation: false,
                });
                let group = |layout, binding| {
                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("terrain corners"),
                        layout,
                        entries: &[wgpu::BindGroupEntry {
                            binding,
                            resource: buffer.as_entire_binding(),
                        }],
                    })
                };
                let read = group(&self.corners_layout, 1);
                let write = group(&self.corners_write_layout, 0);
                slot.corners = Some((needed, buffer, read, write));
            }
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
                    (2, data.heights.as_entire_binding()),
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
                    data.heights.as_entire_binding(),
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
            let (_, _, corners, corners_write) = slot.corners.as_ref().expect("made above");
            draws.push(TerrainDraw {
                group: slot.group.as_ref().expect("made above").1.clone(),
                corners: corners.clone(),
                corners_write: corners_write.clone(),
                corner_count,
                passes: match (holed, &self.arrays) {
                    (Some(passes), _) => passes,
                    (None, Some((_, passes))) if arrayed => passes.clone(),
                    _ => self.passes.clone(),
                },
                features: self.features,
                levels: out,
                cast_shadows: t.cast_shadows,
                mirrored: crate::mirrors(&t.transform),
                terrain: ti,
                arrayed,
                stamps: Vec::new(),
                stamp_group: None,
            });
        }
        self.prepare_stamps(device, queue, scene, &mut draws);
        Ok(draws)
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

/// Every resident tile's square in the terrain's space (grown by one of
/// its level's samples: its border) and its height range.
pub(super) fn resident_boxes(data: &GpuData) -> Vec<(Vec2, Vec2, (f32, f32))> {
    let lay = &data.layout;
    let cell = lay.cell();
    let lo = -lay.size * 0.5;
    data.slots
        .iter()
        .flatten()
        .filter_map(|st| {
            let (k, tx, tz) = lay.locate(st.tile)?;
            let span = (lay.tile * lay.step(k)) as f32;
            let border = lay.step(k) as f32;
            let a = Vec2::new(tx as f32 * span - border, tz as f32 * span - border);
            let b = a + Vec2::splat(span + 2.0 * border);
            Some((lo + a * cell, lo + b * cell, st.data.range))
        })
        .collect()
}

/// The heights a clipmap block from `a` to `b` (the terrain's space) can
/// draw: each point is drawn from a resident tile (the finest there, or
/// mixed with its parent while it fades in), so the union of the ranges
/// of the resident tiles it meets; `None` when it meets none.
pub(super) fn block_range(
    resident: &[(Vec2, Vec2, (f32, f32))],
    a: Vec3,
    b: Vec3,
) -> Option<(f32, f32)> {
    resident
        .iter()
        .filter(|(lo, hi, _)| lo.x <= b.x && hi.x >= a.x && lo.y <= b.z && hi.y >= a.z)
        .map(|(_, _, r)| *r)
        .reduce(|p, q| (p.0.min(q.0), p.1.max(q.1)))
}

/// The world box (min, max) around a box in the terrain's space.
pub(crate) fn world_box(model: Mat4, lo: Vec3, hi: Vec3) -> (Vec3, Vec3) {
    (0..8).fold((Vec3::MAX, Vec3::MIN), |(a, b), i| {
        let corner = Vec3::new(
            if i & 1 == 0 { lo.x } else { hi.x },
            if i & 2 == 0 { lo.y } else { hi.y },
            if i & 4 == 0 { lo.z } else { hi.z },
        );
        let p = model.transform_point3(corner);
        (a.min(p), b.max(p))
    })
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

/// A layer's `offset`: its uv at the scene's origin ([`uv_offset`]), w its
/// displacement.
pub(super) fn layer_offset(origin: Vec3, l: &crate::TerrainLayer) -> [f32; 4] {
    let mut o = uv_offset(origin, l.tile);
    o[3] = if l.displacement.is_finite() {
        l.displacement
    } else {
        0.0
    };
    o
}

/// A layer's uv at the scene's origin, in f64 and without the whole
/// repeats, so the layers sample the same texels wherever the origin is
/// (`origin.rs`): zero at the world's zero.
pub(in super::super) fn uv_offset(origin: Vec3, tile: f32) -> [f32; 4] {
    // The shader's own repeats per metre (`mapping.x`, f32).
    let k = f64::from(1.0 / tile.max(1e-3));
    let at = |o: f32| (f64::from(o) * k).rem_euclid(1.0) as f32;
    [at(origin.x), at(origin.y), at(origin.z), 0.0]
}
