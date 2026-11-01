//! Mesh, texture, material and environment uploads.

use super::*;

impl Renderer {
    /// The f32 vertices of `mesh` on the GPU (its tangents generated where
    /// it lacks them); a 16-byte placeholder for a mesh without vertices.
    fn upload_vertices(&self, mesh: &MeshData) -> wgpu::Buffer {
        let tangents;
        let mesh = if mesh.lacks_tangents() {
            let mut m = mesh.clone();
            m.generate_tangents();
            tangents = m;
            &tangents
        } else {
            mesh
        };
        let bytes: &[u8] = bytemuck::cast_slice(&mesh.vertices);
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertices"),
                contents: if bytes.is_empty() { &[0u8; 16] } else { bytes },
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            })
    }

    /// `mesh` on the GPU; `compact`: as [`VertexEncoding::Compact`], whose
    /// f32 vertices wait for a pipeline that reads them.
    pub(crate) fn upload_mesh(&self, mesh: &MeshData, compact: bool) -> GpuMesh {
        // Never create an empty buffer: an empty mesh keeps a 16-byte placeholder and draws nothing.
        let bytes = |b: &[u8]| {
            if b.is_empty() {
                vec![0u8; 16]
            } else {
                b.to_vec()
            }
        };
        let compact = compact
            .then(|| compact::encode(mesh))
            .flatten()
            .map(|(v, dq)| {
                let buffer = self
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("compact vertices"),
                        contents: bytemuck::cast_slice(&v),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                (buffer, dq)
            });
        let full = compact.is_none();
        let vertices = if full {
            self.upload_vertices(mesh)
        } else {
            self.upload_vertices(&MeshData::default())
        };
        let indices = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("indices"),
                contents: &bytes(bytemuck::cast_slice(&mesh.indices)),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });
        let bounds = mesh
            .bounds()
            .map(|(lo, hi)| ((lo + hi) * 0.5, (hi - lo) * 0.5));
        GpuMesh {
            vertices,
            full,
            compact,
            indices,
            index_count: if mesh.vertices.is_empty() {
                0
            } else {
                mesh.indices.len() as u32
            },
            bounds,
            revision: 0,
        }
    }

    /// Uploads meshes the scene replaced since they were last uploaded: in
    /// place when the sizes match, else as new buffers.
    pub(crate) fn refresh_meshes(&mut self, scene: &RenderScene) {
        for (i, &revision) in scene.mesh_revisions().iter().enumerate() {
            let Some(gpu) = self.cache.meshes.get(i) else {
                break;
            };
            if gpu.revision == revision {
                continue;
            }
            let source = &scene.meshes()[i];
            if let Some(k) = &scene.skins()[i] {
                let positions: Vec<[f32; 3]> = source.vertices.iter().map(|v| v.position).collect();
                self.cache.joint_boxes[i] = skin::joint_boxes(&positions, k);
            }
            let compact = scene.mesh_encodings()[i] == VertexEncoding::Compact
                && scene.skins()[i].is_none()
                && scene.morphs()[i].is_none();
            if compact || gpu.compact.is_some() {
                let mut fresh = self.upload_mesh(source, compact);
                fresh.revision = revision;
                self.cache.meshes[i] = fresh;
                continue;
            }
            let tangents;
            let mesh = if source.lacks_tangents() {
                let mut m = source.clone();
                m.generate_tangents();
                tangents = m;
                &tangents
            } else {
                source
            };
            let vb: &[u8] = bytemuck::cast_slice(&mesh.vertices);
            let ib: &[u8] = bytemuck::cast_slice(&mesh.indices);
            if !vb.is_empty()
                && vb.len() as u64 == gpu.vertices.size()
                && ib.len() as u64 == gpu.indices.size()
            {
                self.queue.write_buffer(&gpu.vertices, 0, vb);
                self.queue.write_buffer(&gpu.indices, 0, ib);
                let bounds = mesh
                    .bounds()
                    .map(|(lo, hi)| ((lo + hi) * 0.5, (hi - lo) * 0.5));
                let gpu = &mut self.cache.meshes[i];
                gpu.bounds = bounds;
                gpu.revision = revision;
            } else {
                let mut fresh = self.upload_mesh(mesh, false);
                fresh.revision = revision;
                self.cache.meshes[i] = fresh;
            }
        }
    }

    /// Uploads a compact mesh's f32 vertices, for a pipeline that reads
    /// [`Vertex`] (a material that is not plain, a scatter, a skinned
    /// instance); nothing for a mesh that has them.
    pub(crate) fn ensure_full(&mut self, scene: &RenderScene, mesh: usize) {
        if self.cache.meshes.get(mesh).is_none_or(|m| m.full) {
            return;
        }
        let vertices = self.upload_vertices(&scene.meshes()[mesh]);
        let gpu = &mut self.cache.meshes[mesh];
        gpu.vertices = vertices;
        gpu.full = true;
    }

    /// Checks a texture before it is uploaded: its data, its format on this
    /// adapter and its size against the device's limit.
    pub(crate) fn check_texture(
        &self,
        index: usize,
        source: &TextureSource,
    ) -> Result<(), RenderError> {
        match source {
            TextureSource::Rgba(texture) => {
                let expected = texture.width as usize * texture.height as usize * 4;
                if texture.width == 0 || texture.height == 0 || texture.rgba.len() != expected {
                    return Err(RenderError::InvalidData(format!(
                        "texture {index}: {}x{} with {} bytes (expected {expected})",
                        texture.width,
                        texture.height,
                        texture.rgba.len()
                    )));
                }
            }
            TextureSource::Encoded(upload) => {
                let desc = &upload.desc;
                desc.validate(&upload.levels)
                    .map_err(|e| RenderError::InvalidData(format!("texture {index}: {e}")))?;
                if !self.info.texture_formats.supported.contains(&desc.format) {
                    return Err(RenderError::Unsupported(format!(
                        "texture {index}: {:?} on {} {:?} (supported: {:?}; transcode to RenderInfo::texture_formats)",
                        desc.format,
                        self.info.adapter,
                        self.info.backend,
                        self.info.texture_formats.supported
                    )));
                }
            }
            TextureSource::Target(_) => {}
        }
        let (width, height) = source.size();
        let max = self.device.limits().max_texture_dimension_2d;
        if width > max || height > max {
            return Err(RenderError::Unsupported(format!(
                "texture {index}: {width}x{height} exceeds this device's {max}x{max} (resize it, or cook a smaller level 0)"
            )));
        }
        Ok(())
    }

    /// Uploads a texture [`Renderer::check_texture`] accepted.
    pub(crate) fn upload_source(
        &mut self,
        source: &TextureSource,
        encoder: &mut crate::Encoder,
    ) -> GpuTexture {
        match source {
            TextureSource::Rgba(texture) => GpuTexture {
                view: self.upload_texture(texture, encoder),
                format: TextureFormat::Rgba8,
                bytes: rgba_texture_bytes(texture),
            },
            TextureSource::Encoded(upload) => self.upload_encoded(upload),
            TextureSource::Target(t) => GpuTexture {
                view: targets::create(&self.device, *t).create_view(&Default::default()),
                format: TextureFormat::Rgba8,
                bytes: t.bytes(),
            },
        }
    }

    fn upload_texture(
        &mut self,
        texture: &TextureData,
        encoder: &mut crate::Encoder,
    ) -> wgpu::TextureView {
        let format = match texture.color_space {
            ColorSpace::Srgb => wgpu::TextureFormat::Rgba8UnormSrgb,
            ColorSpace::Linear => wgpu::TextureFormat::Rgba8Unorm,
        };
        let mip_level_count = match texture.mips {
            Mips::Generate => mips::mip_count(texture.width, texture.height),
            Mips::None => 1,
        };
        let mut usage = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
        if mip_level_count > 1 {
            usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
        }
        let size = wgpu::Extent3d {
            width: texture.width,
            height: texture.height,
            depth_or_array_layers: 1,
        };
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &texture.rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(texture.width * 4),
                rows_per_image: Some(texture.height),
            },
            size,
        );
        if mip_level_count > 1 {
            self.mips.generate(&self.device, encoder, &tex);
        }
        tex.create_view(&Default::default())
    }

    /// A texture whose levels arrive encoded (no mip generation).
    fn upload_encoded(&self, upload: &TextureUpload) -> GpuTexture {
        let desc = &upload.desc;
        let format = desc.format.wgpu(desc.color_space).expect("checked");
        let texture = self.device.create_texture_with_data(
            &self.queue,
            &wgpu::TextureDescriptor {
                label: Some("texture (encoded)"),
                size: wgpu::Extent3d {
                    width: desc.width,
                    height: desc.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: desc.mip_levels,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &upload.levels.concat(),
        );
        GpuTexture {
            view: texture.create_view(&Default::default()),
            format: desc.format,
            bytes: upload.levels.iter().map(|l| l.len() as u64).sum(),
        }
    }

    /// A material's standard group, its array-map group when it draws with
    /// array maps (made by the caller's [`material::ArrayMaps`]), and its
    /// uniform buffer.
    #[allow(clippy::type_complexity)]
    pub(crate) fn material_bind_group(
        &self,
        index: usize,
        m: &StandardMaterial,
        custom: Option<[[f32; 4]; MAX_SHADER_PARAMS]>,
    ) -> Result<(wgpu::BindGroup, Option<wgpu::BindGroup>, wgpu::Buffer), RenderError> {
        let [
            map,
            metalness,
            roughness,
            normal,
            emissive,
            ao,
            gradient,
            dissolve,
        ] = m.textures();
        let texture = |t: Option<TextureHandle>| -> Result<Option<&GpuTexture>, RenderError> {
            match t {
                None => Ok(None),
                Some(h) => self.cache.textures.get(h.index()).map(Some).ok_or_else(|| {
                    RenderError::InvalidHandle(format!(
                        "material {index} uses texture {} (slot {})",
                        h.index(),
                        h.slot()
                    ))
                }),
            }
        };
        let view = |t: Option<TextureHandle>, fallback: &wgpu::TextureView| {
            texture(t).map(|g| g.map_or_else(|| fallback.clone(), |g| g.view.clone()))
        };
        let base = view(map, &self.default_white)?;
        let metal = view(metalness, &self.default_white)?;
        let rough = view(roughness, &self.default_white)?;
        let normal_gpu = texture(normal)?;
        let normal_view =
            normal_gpu.map_or_else(|| self.default_normal.clone(), |g| g.view.clone());
        let emissive = view(emissive, &self.default_white)?;
        let ao = view(ao, &self.default_white)?;
        let gradient = view(gradient, &self.default_white)?;
        let dissolve = view(dissolve, &self.default_white)?;
        let two_channel = normal_gpu.is_some_and(|g| g.format.two_channel());
        let mut uniform: MaterialUniform = material::uniform(m, two_channel);
        if let Some(c) = custom {
            uniform.custom = c;
        }
        // The array maps' views (a material shader keeps the standard path).
        let arrays = match self.array_maps.as_ref() {
            Some(a) if custom.is_none() && m.draws_array_maps() => {
                let [map_array, normal_array] = m.array_maps().map(|h| {
                    h.map(|h| {
                        self.cache
                            .texture_arrays
                            .get(h.index())
                            .and_then(Option::as_ref)
                            .filter(|g| g.generation == h.generation())
                            .ok_or_else(|| {
                                RenderError::InvalidHandle(format!(
                                    "material {index} uses texture array {} (slot {})",
                                    h.index(),
                                    h.slot()
                                ))
                            })
                    })
                    .transpose()
                });
                let (map_array, normal_array) = (map_array?, normal_array?);
                let view = |g: Option<&texture_array::GpuTextureArray>,
                            fallback: &wgpu::TextureView| {
                    g.map_or_else(
                        || fallback.clone(),
                        |g| {
                            g.texture.create_view(&wgpu::TextureViewDescriptor {
                                dimension: Some(wgpu::TextureViewDimension::D2Array),
                                ..Default::default()
                            })
                        },
                    )
                };
                material::array_uniform(
                    &mut uniform,
                    m,
                    normal_array.is_some_and(|g| g.info.format.two_channel()),
                );
                Some((
                    a,
                    view(map_array, &a.white),
                    view(normal_array, &a.flat_normal),
                ))
            }
            _ => None,
        };
        let factors = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("material"),
                contents: bytemuck::bytes_of(&uniform),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let entries = [
            factors.as_entire_binding(),
            wgpu::BindingResource::TextureView(&base),
            wgpu::BindingResource::TextureView(&metal),
            wgpu::BindingResource::TextureView(&rough),
            wgpu::BindingResource::TextureView(&normal_view),
            wgpu::BindingResource::TextureView(&emissive),
            wgpu::BindingResource::TextureView(&ao),
            wgpu::BindingResource::Sampler(&self.material_sampler),
            wgpu::BindingResource::TextureView(&gradient),
            wgpu::BindingResource::TextureView(&dissolve),
        ];
        let group = bind_group(&self.device, "material", &self.material_layout, &entries);
        let array = arrays.map(|(a, base, normal)| {
            let mut entries = entries.clone();
            entries[1] = wgpu::BindingResource::TextureView(&base);
            entries[4] = wgpu::BindingResource::TextureView(&normal);
            bind_group(&self.device, "material (array maps)", &a.layout, &entries)
        });
        Ok((group, array, factors))
    }

    /// A physical material's group for its lit pass (`physical.rs`): its
    /// standard uniform `factors`, the base colour, normal, emissive and
    /// gradient textures, its pack (made here unless a material with the
    /// same maps made it) and its physical uniform.
    pub(crate) fn physical_bind_group(
        &mut self,
        index: usize,
        m: &StandardMaterial,
        factors: &wgpu::Buffer,
    ) -> Result<wgpu::BindGroup, RenderError> {
        if self.physical.is_none() {
            self.physical = Some(physical::PhysicalPipelines::new(
                &self.device,
                &self.frame_layout,
                &self.palette_layout,
                physical_source(&hooks::default_hooks()),
            ));
        }
        let textures = &self.cache.textures;
        let gpu = |h: TextureHandle| {
            textures.get(h.index()).ok_or_else(|| {
                RenderError::InvalidHandle(format!(
                    "material {index} uses texture {} (slot {})",
                    h.index(),
                    h.slot()
                ))
            })
        };
        let (key, layers_of, layers) = physical::pack_plan(m, |h| {
            textures
                .get(h.index())
                .is_some_and(|g| g.format.two_channel())
        });
        let physical = self.physical.as_ref().expect("made above");
        let pack = match self.cache.packs.get(&key) {
            Some(view) => view.clone(),
            None if layers.is_empty() => physical.empty_pack.clone(),
            None => {
                let mut size = (1, 1);
                let mut sources = Vec::with_capacity(layers.len());
                for layer in &layers {
                    let mut views: [wgpu::TextureView; 4] =
                        std::array::from_fn(|_| self.default_white.clone());
                    for (k, &h) in layer.sources.iter().enumerate() {
                        let view = &gpu(h)?.view;
                        let t = view.texture();
                        size = (size.0.max(t.width()), size.1.max(t.height()));
                        views[k] = view.clone();
                    }
                    sources.push((layer.uniform, views));
                }
                let mut encoder = Encoder::untimed(self.device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor {
                        label: Some("material pack"),
                    },
                ));
                let view = physical.pack(
                    &self.device,
                    &mut encoder,
                    &sources,
                    size,
                    &self.material_sampler,
                );
                self.queue.submit(Some(encoder.finish()));
                self.cache.packs.insert(key, view.clone());
                view
            }
        };
        let view =
            |t: u32, fallback: &wgpu::TextureView| -> Result<wgpu::TextureView, RenderError> {
                match TextureHandle::from_slot(t) {
                    None => Ok(fallback.clone()),
                    Some(h) => Ok(gpu(h)?.view.clone()),
                }
            };
        let base = view(m.map, &self.default_white)?;
        let normal = view(m.normalMap, &self.default_normal)?;
        let emissive = view(m.emissiveMap, &self.default_white)?;
        let gradient = view(m.gradientMap, &self.default_white)?;
        let uniform = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("physical material"),
                contents: bytemuck::bytes_of(&physical::uniform(m, layers_of)),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        Ok(bind_group(
            &self.device,
            "physical material",
            &physical.material_layout,
            &[
                factors.as_entire_binding(),
                wgpu::BindingResource::TextureView(&base),
                wgpu::BindingResource::TextureView(&normal),
                wgpu::BindingResource::TextureView(&emissive),
                wgpu::BindingResource::Sampler(&self.material_sampler),
                wgpu::BindingResource::TextureView(&gradient),
                wgpu::BindingResource::TextureView(&pack),
                uniform.as_entire_binding(),
            ],
        ))
    }

    /// Grows the local shadow array to its full capacity when the scene's
    /// shadowed lights could need more layers than it has (once per
    /// renderer: the frame bind groups are made again with it).
    pub(crate) fn grow_local_shadows(&mut self, scene: &RenderScene) {
        let capacity =
            shadows::capacity(self.info.shadowed_lights as usize, self.info.point_shadows);
        // Cookies take layers past every shadow layer there can be
        // (area.rs), so the array grows to the full capacity with them.
        let cookies = area::Cookies::layers(scene) as usize;
        if self.local_map.layers.len() >= capacity + cookies {
            return;
        }
        // At most every shadowed light of the scene's, and the capacity.
        let needed = if cookies > 0 {
            capacity + cookies
        } else {
            scene
                .lights
                .iter()
                .filter(|l| l.cast_shadows)
                .map(|l| match l.kind {
                    LightKind::Point => self.info.point_shadows.layers(),
                    LightKind::Spot { .. } => 1,
                    LightKind::Rect { two_sided, .. } => 1 + two_sided as usize,
                })
                .sum::<usize>()
                .min(capacity)
        };
        if needed <= self.local_map.layers.len() {
            return;
        }
        self.local_map = ShadowMap::new(
            &self.device,
            "local shadows",
            self.local_shadow_size,
            if cookies > 0 { needed } else { capacity } as u32,
        );
        self.environment = None;
    }

    /// (Re)generates the image-based lighting when the sky, or the
    /// environment map that replaces it, changed.
    pub(crate) fn ensure_environment(&mut self, scene: &RenderScene) {
        let sky = &scene.environment.sky;
        let map = scene.environment_map.filter(|m| {
            self.cache
                .environment_maps
                .get(m.map.index())
                .is_some_and(Option::is_some)
        });
        let key = match map {
            Some(m) => EnvironmentKey::Map {
                scene: scene.id(),
                map: m.map.index(),
                revision: self.cache.environment_map_revisions[m.map.index()],
                rotation: m.rotation.to_bits(),
                background: m.background_intensity.to_bits(),
            },
            None => EnvironmentKey::Sky(*sky),
        };
        if self.environment.as_ref().is_some_and(|(k, ..)| *k == key) {
            return;
        }
        match map {
            Some(m) => self.ibl.generate_map(
                &self.device,
                &self.queue,
                &self.cache.environment_maps[m.map.index()]
                    .as_ref()
                    .expect("filtered above")
                    .view,
                &scene.environment_maps()[m.map.index()].irradiance,
                m.rotation,
                m.background_intensity.max(0.0),
            ),
            None => self.ibl.generate(
                &self.device,
                &self.queue,
                sky.sun_direction,
                sky.sun_color,
                sky.look.as_ref(),
            ),
        }
        let ibl = self.ibl.views();
        let entries = [
            self.frame_buffer.as_entire_binding(),
            wgpu::BindingResource::TextureView(&self.cascade_map.array_view),
            wgpu::BindingResource::TextureView(&self.local_map.array_view),
            wgpu::BindingResource::TextureView(&ibl.irradiance),
            wgpu::BindingResource::TextureView(&ibl.prefiltered),
            wgpu::BindingResource::TextureView(&ibl.brdf_lut),
            wgpu::BindingResource::Sampler(&ibl.sampler),
            self.light_buffers[0].as_entire_binding(),
            self.light_buffers[1].as_entire_binding(),
            self.light_buffers[2].as_entire_binding(),
            self.lightmaps.probe_buffer.as_entire_binding(),
            self.ltc.as_entire_binding(),
        ];
        let group = bind_group(&self.device, "frame", &self.frame_layout, &entries);
        let reflection_group = (0..self.reflection_buffer.len()).map(|k| {
            let mut reflected = entries.clone();
            reflected[0] = self.reflection_buffer[k].as_entire_binding();
            reflected[8] = self.reflection_light_buffers[k][1].as_entire_binding();
            reflected[9] = self.reflection_light_buffers[k][2].as_entire_binding();
            bind_group(
                &self.device,
                "frame (planar reflection)",
                &self.frame_layout,
                &reflected,
            )
        });
        let reflection_group = reflection_group.collect();
        let sky_group = bind_group(
            &self.device,
            "sky",
            &self.sky_layout,
            &[wgpu::BindingResource::TextureView(&ibl.sky)],
        );
        self.environment = Some((key, group, sky_group, reflection_group));
    }
}
