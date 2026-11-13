//! Material bind groups.

use super::*;

impl Renderer {
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
}
