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
        // Fork s74: on the web written with writeBuffer straight from wasm
        // memory (buffer_init): no 1.8 GB of short-lived ArrayBuffers in a
        // 1072-cell boot.
        crate::buffer_init(
            &self.device,
            &self.queue,
            &wgpu::util::BufferInitDescriptor {
                label: Some("vertices"),
                contents: if bytes.is_empty() { &[0u8; 16] } else { bytes },
                // STORAGE: the meshlet path pulls them (meshlet.wgsl).
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
            },
        )
    }

    /// `mesh` on the GPU; `compact`: as [`VertexEncoding::Compact`], whose
    /// f32 vertices wait for a pipeline that reads them.
    pub(crate) fn upload_mesh(&self, mesh: &MeshData, compact: bool) -> GpuMesh {
        // Never create an empty buffer: an empty mesh keeps a 16-byte placeholder and draws nothing.
        let compact = compact
            .then(|| compact::encode(mesh))
            .flatten()
            .map(|(v, dq)| {
                let buffer = crate::buffer_init(
                    &self.device,
                    &self.queue,
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("compact vertices"),
                        contents: bytemuck::cast_slice(&v),
                        usage: wgpu::BufferUsages::VERTEX,
                    },
                );
                (buffer, dq)
            });
        let full = compact.is_none();
        let vertices = if full {
            self.upload_vertices(mesh)
        } else {
            self.upload_vertices(&MeshData::default())
        };
        let indices = crate::buffer_init(
            &self.device,
            &self.queue,
            &wgpu::util::BufferInitDescriptor {
                label: Some("indices"),
                contents: if mesh.indices.is_empty() {
                    &[0u8; 16]
                } else {
                    bytemuck::cast_slice(&mesh.indices)
                },
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            },
        );
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
            foliage: None,
            surface: None,
        }
    }

    pub(crate) fn upload_surface(
        &self,
        mesh: &MeshData,
        kind: SimulationSurface,
        level: u32,
    ) -> GpuMesh {
        let count = mesh.indices.len() as u64 * u64::from(level * level);
        let bytes = count * std::mem::size_of::<Vertex>() as u64;
        let Some(pipelines) =
            self.world.surfaces.as_ref().filter(|_| {
                bytes > 0 && bytes <= self.device.limits().max_storage_buffer_binding_size
            })
        else {
            return self.upload_mesh(&surface::fallback(mesh, kind), false);
        };
        let vertices = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("simulation surface vertices"),
            size: bytes,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let indices: Vec<u32> = (0..count as u32).collect();
        let indices = crate::buffer_init(
            &self.device,
            &self.queue,
            &wgpu::util::BufferInitDescriptor {
                label: Some("simulation surface indices"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            },
        );
        let surface = pipelines.mesh(&self.device, &self.queue, mesh, &vertices, level);
        GpuMesh {
            vertices,
            indices,
            index_count: count as u32,
            full: true,
            compact: None,
            bounds: surface::bounds(mesh),
            revision: 0,
            foliage: None,
            surface: Some(surface),
        }
    }

    /// Use the same conservative box for CPU sorting and every GPU cull pass.
    /// A shared mesh takes the largest current/previous sway of its instances;
    /// this avoids adding a per-instance cull stream just for the margin.
    pub(crate) fn refresh_foliage_bounds(&mut self, scene: &RenderScene) {
        let winds = crate::foliage::foliage_winds(
            scene,
            self.history.previous_wind,
            crate::wind_field::NO_GRID,
        )
        .0;
        let mut sway = vec![0.0f32; self.cache.meshes.len()];
        let previous = if self.instances.previous_sway.len() == scene.objects.instances.len() {
            self.instances.previous_sway.as_slice()
        } else {
            &[]
        };
        for (i, inst) in scene.objects.instances.iter().enumerate() {
            if scene.mesh_foliage(inst.mesh).is_some() {
                sway[inst.mesh.index()] = sway[inst.mesh.index()]
                    .max(inst.sway)
                    .max(previous.get(i).copied().unwrap_or(inst.sway));
            }
        }
        for (i, mesh) in self.cache.meshes.iter_mut().enumerate() {
            let Some(source) = &scene.mesh_foliage_all()[i] else {
                if mesh.foliage.take().is_some() {
                    // set_mesh may have changed the geometry as well.
                    mesh.bounds = scene.meshes()[i]
                        .bounds()
                        .map(|(lo, hi)| ((lo + hi) * 0.5, (hi - lo) * 0.5));
                }
                continue;
            };
            if mesh
                .foliage
                .as_ref()
                .is_none_or(|f| !std::sync::Arc::ptr_eq(&f.source, source))
            {
                let data = &scene.meshes()[i];
                mesh.foliage = Some(FoliageMesh {
                    source: source.clone(),
                    rest: data
                        .bounds()
                        .map(|(lo, hi)| ((lo + hi) * 0.5, (hi - lo) * 0.5)),
                    deformation: source
                        .bounds(data.vertices.iter().map(|v| Vec3::from_array(v.position))),
                });
            }
            let f = mesh.foliage.as_ref().unwrap();
            let margin = winds
                .iter()
                .map(|w| {
                    f.deformation.deform_margin(
                        crate::wind_field::lean_cap(Some(*w).filter(|w| w.strength > 0.0), false)
                            .max(self.world.wind_field.lean_cap()),
                    )
                })
                .fold(0.0f32, f32::max)
                * sway[i].max(1.0);
            mesh.bounds = f
                .rest
                .map(|(center, half)| (center, half + Vec3::splat(margin)));
        }
    }

    /// Plans and writes the frame of every visual body: the simulation
    /// surfaces the scene made visual ([`RenderScene::set_visual_body`]),
    /// their rest shapes this tick, the colliders near them and their
    /// conservative bounds. After [`Renderer::refresh_meshes`].
    pub(crate) fn refresh_visual(&mut self, scene: &RenderScene) {
        let Renderer {
            world,
            cache,
            device,
            queue,
            ..
        } = self;
        let Some(pipelines) = world.visual.as_mut() else {
            return;
        };
        let mut bodies = Vec::new();
        for (i, gpu) in cache.meshes.iter_mut().enumerate() {
            let Some(surface) = gpu.surface.as_mut() else {
                continue;
            };
            let Some(body) = scene.mesh_visual.get(i).and_then(Option::as_ref) else {
                surface.visual = None;
                continue;
            };
            let kind = scene.mesh_surfaces[i];
            let (rest, triangles) = crate::visual_sim::rest_of(&scene.meshes()[i]);
            if surface
                .visual
                .as_ref()
                .is_none_or(|v| !v.topology.matches(kind, body, &rest, &triangles))
            {
                let topology = crate::visual_sim::Topology::build(kind, body, &rest, &triangles);
                surface.visual = Some(pipelines.mesh(device, queue, topology));
            }
            let v = surface.visual.as_mut().expect("made above");
            v.bounds = crate::visual_sim::grown_bounds(&rest, v.topology.margin);
            let (lo, hi) = v.bounds;
            gpu.bounds = Some(((lo + hi) * 0.5, (hi - lo) * 0.5));
            bodies.push((i, rest));
        }
        // Visual garments: within their reach of this frame's skinned pose.
        let garments: Vec<_> = scene
            .objects
            .deformed_vertices
            .iter()
            .filter_map(|d| d.garment.as_ref())
            .filter_map(|g| {
                let body = g.visual.as_ref()?;
                let reach = body.reach.iter().fold(0.0f32, |m, &r| m.max(r));
                let margin = reach + body.radius.max(0.0) + body.backstop.max(0.0);
                Some((g.key, crate::visual_sim::grown_bounds(&g.targets, margin)))
            })
            .collect();
        pipelines.garment_lists.clear();
        if bodies.is_empty() && garments.is_empty() {
            return;
        }
        let mut bounds: Vec<_> = bodies
            .iter()
            .map(|&(i, _)| {
                let s = cache.meshes[i].surface.as_ref().expect("a surface");
                s.visual.as_ref().expect("a visual body").bounds
            })
            .collect();
        bounds.extend(garments.iter().map(|&(_, b)| b));
        let mut lists = pipelines.write_colliders(device, queue, &scene.world.colliders, &bounds);
        let garment_lists = lists.split_off(bodies.len());
        pipelines.garment_lists = garments
            .iter()
            .map(|&(key, _)| key)
            .zip(garment_lists)
            .collect();
        let dt = if scene.frame.tick_rate > 0.0 {
            1.0 / scene.frame.tick_rate
        } else {
            1.0 / 60.0
        };
        for ((i, rest), list) in bodies.into_iter().zip(lists) {
            let body = scene.mesh_visual[i].as_ref().expect("visual");
            let surface = cache.meshes[i].surface.as_mut().expect("a surface");
            let crate::surface::SurfaceMesh { source, visual, .. } = surface;
            pipelines.prepare(
                device,
                queue,
                visual.as_mut().expect("a visual body"),
                body,
                &rest,
                &[],
                scene.frame.tick,
                dt,
                list,
                Mat4::IDENTITY,
                source,
            );
        }
    }

    /// Uploads meshes the scene replaced since they were last uploaded: in
    /// place when the sizes match, else as new buffers.
    pub(crate) fn refresh_meshes(&mut self, scene: &RenderScene) {
        for (i, &revision) in scene.mesh_revisions().iter().enumerate() {
            let Some(gpu) = self.cache.meshes.get(i) else {
                break;
            };
            let source = &scene.meshes()[i];
            if let Some(kind) = scene.mesh_surfaces[i] {
                let level = surface::subdivisions(scene.objects.cloth_subdivisions);
                if let Some(surface) = gpu.surface.as_ref().filter(|s| s.matches(source, level)) {
                    // A visual body's vertices are its GPU simulation's.
                    if gpu.revision != revision && scene.mesh_visual[i].is_none() {
                        surface.update(&self.queue, source);
                    }
                    let gpu = &mut self.cache.meshes[i];
                    gpu.bounds = surface::bounds(source);
                    gpu.revision = revision;
                } else {
                    let mut fresh = self.upload_surface(source, kind, level);
                    fresh.revision = revision;
                    self.cache.meshes[i] = fresh;
                }
                continue;
            }
            if gpu.surface.is_some() {
                let mut fresh = self.upload_mesh(source, false);
                fresh.revision = revision;
                self.cache.meshes[i] = fresh;
                continue;
            }
            if gpu.revision == revision {
                continue;
            }
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
    /// Texture slot `index` on the GPU: its source uploaded, or for a
    /// removed slot the shared fallback (`default_white`, allocated once
    /// for the renderer): the slot's own texture is dropped and freed, and
    /// a stale material or a draw in flight still has something to sample.
    pub(crate) fn texture_for(
        &mut self,
        scene: &crate::RenderScene,
        index: usize,
        encoder: &mut crate::Encoder,
    ) -> GpuTexture {
        if scene.texture_live(index) {
            return self.upload_source(&scene.textures()[index], encoder);
        }
        GpuTexture {
            view: self.default_white.clone(),
            format: TextureFormat::Rgba8,
            bytes: 0,
            target: false,
        }
    }

    /// Fork s63: a deferred texture's 1x1 stand-in until a later frame
    /// uploads it: mid grey and transparent for colour, a flat normal for a
    /// two-channel map, white occlusion and roughness for another linear one.
    pub(crate) fn stand_in(
        &mut self,
        source: &TextureSource,
        encoder: &mut crate::Encoder,
    ) -> GpuTexture {
        let (srgb, two_channel) = match source {
            TextureSource::Rgba(t) => (t.color_space == ColorSpace::Srgb, false),
            TextureSource::Encoded(u) => (
                u.desc.color_space == ColorSpace::Srgb,
                u.desc.format.two_channel(),
            ),
            TextureSource::Target(_) => (false, false),
        };
        let rgba = if srgb {
            vec![128, 128, 128, 0]
        } else if two_channel {
            vec![128, 128, 255, 255]
        } else {
            vec![255, 255, 0, 255]
        };
        let data = TextureData {
            width: 1,
            height: 1,
            rgba,
            color_space: if srgb {
                ColorSpace::Srgb
            } else {
                ColorSpace::Linear
            },
            mips: Mips::Generate,
        };
        GpuTexture {
            view: self.upload_texture(&data, encoder),
            format: TextureFormat::Rgba8,
            bytes: 4,
            target: false,
        }
    }

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
                target: false,
            },
            TextureSource::Encoded(upload) => self.upload_encoded(upload),
            TextureSource::Target(t) => GpuTexture {
                view: targets::create(&self.device, *t).create_view(&Default::default()),
                format: TextureFormat::Rgba8,
                bytes: t.bytes(),
                target: true,
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
        use crate::texture::TextureFormatGpu;
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
            target: false,
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
        previous: Option<[[f32; 4]; MAX_SHADER_PARAMS]>,
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
            // The parameters the frames before drew, read in this frame
            // only (engine s100): a colour or texture edit keeps the
            // vertex's motion.
            uniform.previous_custom = previous.unwrap_or(c);
            uniform.stamp = [self.frame_index as u32, 0, 0, 0];
        }
        // The array maps' views (a material shader keeps the standard path).
        let arrays = match self.lit.array_maps.as_ref() {
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
        let factors = crate::buffer_init(
            &self.device,
            &self.queue,
            &wgpu::util::BufferInitDescriptor {
                label: Some("material"),
                contents: bytemuck::bytes_of(&uniform),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        );
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
        if self.lit.physical.is_none() {
            self.lit.physical = Some(physical::PhysicalPipelines::new(
                &self.device,
                &self.frame_layout,
                &self.palette_layout,
                self.info
                    .without
                    .features()
                    .leave_out(physical_source(&hooks::default_hooks())),
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
        let physical = self.lit.physical.as_ref().expect("made above");
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
                    &self.queue,
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
        let uniform = crate::buffer_init(
            &self.device,
            &self.queue,
            &wgpu::util::BufferInitDescriptor {
                label: Some("physical material"),
                contents: bytemuck::bytes_of(&physical::uniform(m, layers_of)),
                usage: wgpu::BufferUsages::UNIFORM,
            },
        );
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

    /// Gives the cascade array one more layer, last, for the weather
    /// occlusion map while the scene's surface weather has shelter, and
    /// takes it away again when it has none, in front of it the frame's
    /// cascades ([`Self::frame_cascades`]); the frame groups are made again
    /// when the layer count changes.
    pub(crate) fn fit_shelter_layer(&mut self, scene: &RenderScene) {
        // Only the weather on surfaces reads it (RenderDesc::without).
        let shelter = !self.info.without.contains(LitFeatures::SURFACE_WEATHER)
            && scene
                .atmosphere
                .surface_weather
                .is_some_and(|w| w.shelter > 0.0);
        let wanted = self.frame_cascades(scene) + u32::from(shelter);
        if self.shadows.cascade_map.layers.len() as u32 == wanted {
            return;
        }
        self.shadows.cascade_map = ShadowMap::new(
            &self.device,
            "cascades",
            self.info.shadow_map_size,
            wanted,
            wgpu::TextureFormat::R32Float,
        );
        self.shadows.shelter_key = None;
        self.shadows.shadow_cache.forget_layers();
        self.lights.environment = None;
    }

    /// The cascades a frame of `scene` draws: its quality's count within
    /// the renderer's ([`crate::RenderInfo::shadow_cascades`]), at least one.
    pub(crate) fn frame_cascades(&self, scene: &RenderScene) -> u32 {
        scene
            .frame
            .quality
            .shadow_cascades
            .min(self.info.shadow_cascades)
            .max(1)
    }

    /// Grows the local shadow array when the scene's shadowed lights could
    /// need more layers than it has ([`shadows::grown_layers`]; the frame
    /// bind groups are made again with it). It never shrinks.
    pub(crate) fn grow_local_shadows(&mut self, scene: &RenderScene) {
        let capacity =
            shadows::capacity(self.info.shadowed_lights as usize, self.info.point_shadows);
        // Cookies take layers past every shadow layer there can be
        // (area.rs), so the array grows to the full capacity with them.
        let cookies = area::Cookies::layers(scene) as usize;
        if self.shadows.local_map.layers.len() >= capacity + cookies {
            return;
        }
        // Every shadowed light of the scene's, within the frame's budget.
        let lights = scene
            .frame
            .quality
            .shadowed_lights
            .min(self.info.shadowed_lights) as usize;
        let wanted: Vec<usize> = scene
            .lighting
            .lights
            .iter()
            .filter(|l| l.cast_shadows && self.kept_kind(l))
            .map(|l| match l.kind {
                LightKind::Point => self.info.point_shadows.layers(),
                LightKind::Spot { .. } => 1,
                LightKind::Rect { two_sided, .. } => 1 + two_sided as usize,
            })
            .collect();
        let needed = if cookies > 0 {
            capacity + cookies
        } else {
            shadows::grown_layers(&wanted, lights, self.info.point_shadows, capacity)
        };
        if needed <= self.shadows.local_map.layers.len() {
            return;
        }
        self.shadows.local_map = ShadowMap::new(
            &self.device,
            "local shadows",
            self.shadows.local_shadow_size,
            needed as u32,
            LOCAL_SHADOW_FORMAT,
        );
        self.shadows.shadow_cache.forget_layers();
        self.lights.environment = None;
    }

    /// (Re)generates the image-based lighting when the sky, or the
    /// environment map that replaces it, changed.
    /// The environment map a frame draws with, when it is uploaded, and the
    /// map it blends towards (its index and drawn weight) while that is
    /// uploaded too and the weight moves off the first
    /// (`EnvironmentBlend::drawn_weight`).
    fn drawn_environment(
        &self,
        scene: &RenderScene,
    ) -> Option<(crate::EnvironmentMap, Option<(usize, f32)>)> {
        let live = |i: usize| {
            self.cache
                .environment_maps
                .get(i)
                .is_some_and(Option::is_some)
        };
        let map = scene
            .lighting
            .environment_map
            .filter(|m| live(m.map.index()))?;
        let blend = map
            .blend
            .filter(|b| live(b.to.index()))
            .map(|b| (b.to.index(), b.drawn_weight()))
            .filter(|b| b.1 > 0.0);
        Some((map, blend))
    }

    /// The sky pass's uniform for this frame: the background's blurriness
    /// and the map it draws (`SkyParams`).
    pub(crate) fn write_sky_params(&self, scene: &RenderScene) {
        let map = self.drawn_environment(scene).map(|(m, blend)| SkyMap {
            rotation: m.rotation,
            scale: m.background_intensity.max(0.0),
            blend: blend.map_or(0.0, |b| b.1),
        });
        let params = SkyParams::new(&scene.lighting.environment, crate::ibl::PREFILTER_MIPS, map);
        self.queue
            .write_buffer(&self.sky.sky_params, 0, bytemuck::bytes_of(&params));
    }

    pub(crate) fn ensure_environment(&mut self, scene: &RenderScene) {
        let sky = &scene.lighting.environment.sky;
        let drawn = self.drawn_environment(scene);
        let map = drawn.map(|d| d.0);
        let blend = drawn.and_then(|d| d.1);
        // The per-pixel sky without an environment map is live: captured
        // as it is drawn, a slice a frame (sky_capture.rs).
        let live = map.is_none()
            && matches!(
                scene.lighting.environment.background,
                Background::Procedural { .. }
            );
        if !live {
            self.sky.sky_capture = None;
        }
        let key = match map {
            None if live => EnvironmentKey::Live,
            Some(m) => EnvironmentKey::Map {
                scene: scene.id(),
                map: m.map.index(),
                revision: self.cache.environment_map_revisions[m.map.index()],
                rotation: m.rotation.to_bits(),
                background: m.background_intensity.to_bits(),
                blend: blend
                    .map(|(i, w)| (i, self.cache.environment_map_revisions[i], w.to_bits())),
            },
            None => EnvironmentKey::Sky(*sky),
        };
        // The first frame with clouds draws their noise and makes the sky
        // group again with it.
        if scene.atmosphere.clouds.is_some() && !self.sky.cloud_noise.made {
            self.sky.cloud_noise.make(&self.device, &self.queue);
            self.lights.environment = None;
        }
        // The aurora's footprint follows the clock every frame; the first
        // frame with one makes its texture, and the sky group again. It is
        // drawn by compute: without it the sky keeps the empty footprint.
        if let Some(a) = &scene.atmosphere.aurora
            && self.info.caps.contains(crate::Caps::COMPUTE)
        {
            let made = self.sky.aurora_map.made();
            let viewer = scene.frame.camera.position + scene.frame.origin;
            let rate = if scene.frame.tick_rate > 0.0 {
                scene.frame.tick_rate
            } else {
                60.0
            };
            self.sky.aurora_map.update(
                &self.device,
                &self.queue,
                a,
                scene.frame.tick as f64 / f64::from(rate),
                glam::Vec2::new(viewer.x, viewer.z),
            );
            if !made {
                self.lights.environment = None;
            }
        }
        if self
            .lights
            .environment
            .as_ref()
            .is_some_and(|(k, ..)| *k == key)
        {
            return;
        }
        let view = |i: usize| {
            &self.cache.environment_maps[i]
                .as_ref()
                .expect("filtered above")
                .view
        };
        match map {
            Some(m) => {
                let first = &scene.environment_maps()[m.map.index()].irradiance;
                let irradiance = match blend {
                    Some((i, w)) => environment::mix_irradiance(
                        first,
                        &scene.environment_maps()[i].irradiance,
                        w,
                    ),
                    None => *first,
                };
                self.lights.ibl.generate_map(
                    &self.device,
                    &self.queue,
                    view(m.map.index()),
                    blend.map(|(i, w)| (view(i), w)),
                    &irradiance,
                    m.rotation,
                    m.background_intensity.max(0.0),
                )
            }
            None => self.lights.ibl.generate(
                &self.device,
                &self.queue,
                sky.sun_direction,
                sky.sun_color,
                sky.look.as_ref(),
            ),
        }
        // The bound cubes are the sky without its clouds again: the live
        // sky captures it whole the next frame.
        if let Some(c) = &mut self.sky.sky_capture {
            c.restart();
        }
        let ibl = self.lights.ibl.views();
        let entries = [
            self.lights.frame_buffer.as_entire_binding(),
            wgpu::BindingResource::TextureView(&self.shadows.cascade_map.array_view),
            wgpu::BindingResource::TextureView(&self.shadows.local_map.array_view),
            wgpu::BindingResource::TextureView(&ibl.irradiance),
            wgpu::BindingResource::TextureView(&ibl.prefiltered),
            wgpu::BindingResource::TextureView(&ibl.brdf_lut),
            wgpu::BindingResource::Sampler(&ibl.sampler),
            self.lights.light_buffers[0].as_entire_binding(),
            self.lights.light_buffers[1].as_entire_binding(),
            self.lights.light_buffers[2].as_entire_binding(),
            self.lights.lightmaps.probe_buffer.as_entire_binding(),
            self.lights.ltc.as_entire_binding(),
            self.lights.caustic_grid.as_entire_binding(),
            // The main camera's reflections (ssr.rs; `Frame.ssr` switches
            // them, off in the planar reflections' uniforms).
            wgpu::BindingResource::TextureView(self.ssr.frame_view()),
        ];
        let group = bind_group(&self.device, "frame", &self.frame_layout, &entries);
        let reflection_group = (0..self.reflections.reflection_buffer.len()).map(|k| {
            let mut reflected = entries.clone();
            reflected[0] = self.reflections.reflection_buffer[k].as_entire_binding();
            reflected[8] = self.reflections.reflection_light_buffers[k][1].as_entire_binding();
            reflected[9] = self.reflections.reflection_light_buffers[k][2].as_entire_binding();
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
            &self.sky.sky_layout,
            &[
                wgpu::BindingResource::TextureView(&ibl.sky),
                wgpu::BindingResource::TextureView(&self.sky.cloud_noise.view),
                wgpu::BindingResource::Sampler(&self.sky.cloud_noise.sampler),
                self.sky.sky_params.as_entire_binding(),
                wgpu::BindingResource::TextureView(map.map_or(&ibl.sky, |m| view(m.map.index()))),
                wgpu::BindingResource::TextureView(blend.map_or(&ibl.sky, |b| view(b.0))),
                wgpu::BindingResource::TextureView(&self.sky.aurora_map.view),
            ],
        );
        self.lights.environment = Some((key, group, sky_group, reflection_group));
        if live && self.sky.sky_capture.is_none() {
            self.sky.sky_capture = Some(crate::sky_capture::SkyCapture::new(
                &self.device,
                &self.queue,
                &self.lights.ibl,
                [&self.frame_layout, &self.sky.sky_layout],
                self.info.without.features(),
            ));
        }
    }
}
