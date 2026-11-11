//! Scene resource synchronization.

use super::*;

impl Renderer {
    /// The environment maps, texture arrays and palettes the scene added,
    /// refilled or removed since the last frame: each slot whose revision
    /// moved is uploaded again, or freed when the scene removed it.
    fn sync_assets(&mut self, scene: &RenderScene) -> Result<(), RenderError> {
        let revisions = scene.environment_map_revisions();
        for (i, &revision) in revisions.iter().enumerate() {
            let fresh = i >= self.cache.environment_maps.len();
            if !fresh && self.cache.environment_map_revisions[i] == revision {
                continue;
            }
            let gpu = if let Some(image) = scene
                .environment_equirect(i)
                .filter(|_| scene.environment_map_slot(i).0)
            {
                // An uncooked image: prefiltered here, on the GPU.
                image
                    .validate()
                    .map_err(|e| RenderError::InvalidData(format!("environment map {i}: {e}")))?;
                let (texture, view) = self.ibl.bake_equirect(&self.device, &self.queue, image);
                Some(GpuEnvironmentMap {
                    _texture: texture,
                    view,
                    bytes: 8 * environment::EnvironmentMapData::texel_count(image.size) as u64,
                })
            } else if scene.environment_map_slot(i).0 {
                let map = &scene.environment_maps()[i];
                map.validate()
                    .map_err(|e| RenderError::InvalidData(format!("environment map {i}: {e}")))?;
                let (texture, view) = environment::upload(&self.device, &self.queue, map);
                Some(GpuEnvironmentMap {
                    _texture: texture,
                    view,
                    bytes: 4 * environment::EnvironmentMapData::texel_count(map.size) as u64,
                })
            } else {
                None
            };
            if fresh {
                self.cache.environment_maps.push(gpu);
                self.cache.environment_map_revisions.push(revision);
            } else {
                self.cache.environment_maps[i] = gpu;
                self.cache.environment_map_revisions[i] = revision;
            }
        }
        let revisions = scene.texture_array_revisions();
        for (i, &revision) in revisions.iter().enumerate() {
            let fresh = i >= self.cache.texture_arrays.len();
            if !fresh && self.cache.texture_array_revisions[i] == revision {
                continue;
            }
            let (live, generation) = scene.texture_array_slot(i);
            let gpu = if live {
                let data = &scene.texture_arrays()[i];
                let bad = |e: String| RenderError::InvalidData(format!("texture array {i}: {e}"));
                let info = data.info().map_err(bad)?;
                if info.format != TextureFormat::Rgba8
                    && !self.info.texture_formats.supported.contains(&info.format)
                {
                    return Err(bad(format!("{} is not supported here", info.format.name())));
                }
                let mut gpu =
                    texture_array::upload(&self.device, &self.queue, data, info).map_err(bad)?;
                gpu.generation = generation;
                Some(gpu)
            } else {
                None
            };
            if fresh {
                self.cache.texture_arrays.push(gpu);
                self.cache.texture_array_revisions.push(revision);
                self.cache.sprite_array_groups.push(None);
            } else {
                self.cache.texture_arrays[i] = gpu;
                self.cache.texture_array_revisions[i] = revision;
                self.cache.sprite_array_groups[i] = None;
            }
        }
        let revisions = scene.palette_revisions();
        for (i, &revision) in revisions.iter().enumerate() {
            let fresh = i >= self.cache.palettes.len();
            if !fresh && self.cache.palette_revisions[i] == revision {
                continue;
            }
            let view = scene.palette_live(i).then(|| {
                let palette = &scene.palettes()[i];
                let mut rgba = vec![0u8; MAX_PALETTE_COLORS * 4];
                for (k, c) in palette.colors.iter().enumerate() {
                    rgba[k * 4..k * 4 + 3].copy_from_slice(c);
                    rgba[k * 4 + 3] = 255;
                }
                post::texture_from_rgba8(
                    &self.device,
                    &self.queue,
                    "palette",
                    MAX_PALETTE_COLORS as u32,
                    1,
                    wgpu::TextureFormat::Rgba8UnormSrgb,
                    &rgba,
                )
            });
            if fresh {
                self.cache.palettes.push(view);
                self.cache.palette_revisions.push(revision);
            } else {
                self.cache.palettes[i] = view;
                self.cache.palette_revisions[i] = revision;
            }
        }
        Ok(())
    }

    /// GPU bytes of the scene's environment maps, texture arrays, palettes
    /// and colour LUTs (removed ones are freed and count nothing).
    pub(crate) fn asset_bytes(&self) -> u64 {
        let maps: u64 = self
            .cache
            .environment_maps
            .iter()
            .flatten()
            .map(|m| m.bytes)
            .sum();
        let arrays: u64 = self
            .cache
            .texture_arrays
            .iter()
            .flatten()
            .map(|a| a.info.bytes)
            .sum();
        let palettes = self.cache.palettes.iter().flatten().count() as u64 * PALETTE_BYTES;
        let luts: u64 = self.cache.luts.iter().flatten().map(|l| l.1).sum();
        maps + arrays + palettes + luts
    }

    /// The array sprite pipelines and the bind group of `array` (made on
    /// first use; again when the array's slot or filter changed).
    pub(crate) fn sprite_array_group(&mut self, scene: &RenderScene, array: TextureArrayHandle) {
        if self.sprite_arrays.is_none() {
            self.sprite_arrays = Some(sprite_array::SpriteArrayPipelines::new(
                &self.device,
                &self.frame_layout,
                &self.sprite_fx_layout,
                self.samples,
                self.info.without.features(),
            ));
        }
        let i = array.index();
        let filter = scene.texture_array_filter(array);
        if self.cache.sprite_array_groups[i]
            .as_ref()
            .is_some_and(|(_, f)| *f == filter)
        {
            return;
        }
        let Some(gpu) = self.cache.texture_arrays[i].as_ref() else {
            return;
        };
        let sampler = if filter.is_nearest() {
            &self.sprite_sampler_nearest
        } else {
            &self.sprite_sampler
        };
        let group = self.sprite_arrays.as_ref().expect("made above").bind_group(
            &self.device,
            &gpu.texture,
            sampler,
        );
        self.cache.sprite_array_groups[i] = Some((group, filter));
    }

    /// Uploads whatever the scene added (or changed) since the last frame.
    pub(crate) fn sync(&mut self, scene: &RenderScene) -> Result<(), RenderError> {
        if self.cache.scene_id != scene.id() {
            self.cache = SceneCache {
                scene_id: scene.id(),
                ..Default::default()
            };
            self.history.taa_history = None;
            self.gi_history = None;
            self.ssr.reset();
            self.history.previous_time = None;
            self.history.previous_wind = None;
            self.previous_sway.clear();
            self.shadow_cache = Default::default();
        }
        for index in self.cache.shaders.len()..scene.shaders().len() {
            let (lit, shadow) = custom::module_sources(&scene.shaders()[index], self.planar_slot());
            // Without the features the renderer is made without
            // (pipeline_features.rs), in every pipeline made from it.
            let lit = self.info.without.features().leave_out(lit);
            let module = shader(&self.device, "pbr (material shader)", lit);
            self.cache.shaders.push(module);
            let module = shader(&self.device, "shadow (material shader)", shadow);
            self.cache.shadow_shaders.push(module);
        }
        self.refresh_meshes(scene);
        for index in self.cache.meshes.len()..scene.meshes().len() {
            let mesh = &scene.meshes()[index];
            let n = mesh.vertices.len();
            // Checked before anything joins the cache: `meshes`, `morph_ranges`
            // and `skins` stay the same length, and a bad mesh fails again
            // next frame instead of shifting its neighbours' entries.
            let morph = scene.morphs()[index].as_ref();
            if let Some(m) = morph {
                let targets = m.positions.len();
                let bad = targets == 0
                    || targets > MAX_MORPH_TARGETS
                    || m.positions.iter().any(|t| t.len() != n)
                    || m.normals
                        .as_ref()
                        .is_some_and(|ns| ns.len() != targets || ns.iter().any(|t| t.len() != n));
                if bad {
                    return Err(RenderError::InvalidData(format!(
                        "mesh {index}: {targets} morph targets (1 to {MAX_MORPH_TARGETS}, one delta per vertex each, {n} vertices)"
                    )));
                }
            }
            // A morphed mesh without a skin: all-zero weights (bind pose).
            let zero_skin;
            let source = match &scene.skins()[index] {
                None if morph.is_some() => {
                    zero_skin = SkinData {
                        joints: vec![[0; 4]; n],
                        weights: vec![[0.0; 4]; n],
                    };
                    Some(&zero_skin)
                }
                other => other.as_ref(),
            };
            if let Some(skin) = source
                && (skin.joints.len() != n || skin.weights.len() != n)
            {
                return Err(RenderError::InvalidData(format!(
                    "mesh {index}: a skin of {} joints and {} weights for {n} vertices",
                    skin.joints.len(),
                    skin.weights.len()
                )));
            }
            let compact = scene.mesh_encodings()[index] == VertexEncoding::Compact
                && source.is_none()
                && morph.is_none();
            let mut gpu = match scene.mesh_surfaces[index] {
                Some(kind) => {
                    self.upload_surface(mesh, kind, surface::subdivisions(scene.cloth_subdivisions))
                }
                None => self.upload_mesh(mesh, compact),
            };
            gpu.revision = scene.mesh_revisions()[index];
            self.cache.meshes.push(gpu);
            let mut range = (0u32, 0u32);
            if let Some(m) = morph {
                range = (self.cache.morph_data.len() as u32, m.positions.len() as u32);
                pack_morphs(m, n, &mut self.cache.morph_data);
                self.cache.morphs_dirty = true;
            }
            self.cache.morph_ranges.push(range);
            let skin = match source {
                None => None,
                Some(skin) => {
                    let data: Vec<skin::SkinVertex> = skin
                        .joints
                        .iter()
                        .zip(&skin.weights)
                        .map(|(&joints, &weights)| skin::SkinVertex { joints, weights })
                        .collect();
                    let mut bytes = bytemuck::cast_slice(&data).to_vec();
                    if bytes.is_empty() {
                        bytes = vec![0u8; 24];
                    }
                    Some(crate::buffer_init(
                        &self.device,
                        &self.queue,
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("skin"),
                            contents: &bytes,
                            usage: wgpu::BufferUsages::VERTEX,
                        },
                    ))
                }
            };
            self.cache.skins.push(skin);
            self.cache.joint_boxes.push(
                scene.skins()[index]
                    .as_ref()
                    .map(|k| {
                        let positions: Vec<[f32; 3]> =
                            mesh.vertices.iter().map(|v| v.position).collect();
                        skin::joint_boxes(&positions, k)
                    })
                    .unwrap_or_default(),
            );
        }
        // Every simulation surface is uploaded, new ones too.
        self.refresh_visual(scene);
        self.sync_assets(scene)?;
        let revisions = scene.lut_revisions();
        for (i, &revision) in revisions.iter().enumerate() {
            let fresh = i >= self.cache.luts.len();
            if !fresh && self.cache.lut_revisions[i] == revision {
                continue;
            }
            let gpu = if scene.lut_live(i) {
                let lut = &scene.luts()[i];
                lut.validate()
                    .map_err(|e| RenderError::InvalidData(format!("lut {i}: {e}")))?;
                let bytes = u64::from(lut.size).pow(3) * 4;
                Some((post::lut_texture(&self.device, &self.queue, lut), bytes))
            } else {
                None
            };
            if fresh {
                self.cache.luts.push(gpu);
                self.cache.lut_revisions.push(revision);
            } else {
                self.cache.luts[i] = gpu;
                self.cache.lut_revisions[i] = revision;
            }
        }
        let upload_budget = UPLOAD_BUDGET.load(std::sync::atomic::Ordering::Relaxed);
        let mut uploaded = 0u64;
        if self.cache.textures.len() < scene.textures().len() {
            // Every texture is checked before any is recorded: an error
            // never drops an encoder holding earlier textures' mip passes.
            for index in self.cache.textures.len()..scene.textures().len() {
                if scene.texture_live(index) {
                    self.check_texture(index, &scene.textures()[index])?;
                }
            }
            let mut encoder = Encoder::untimed(self.device.create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("texture upload"),
                },
            ));
            for index in self.cache.textures.len()..scene.textures().len() {
                // Fork s63: past the frame's budget, a stand-in now and the
                // texture later.
                let source = &scene.textures()[index];
                let bytes = if scene.texture_live(index) {
                    source_upload_bytes(source)
                } else {
                    0
                };
                let defer = upload_budget > 0
                    && bytes > 0
                    && uploaded > 0
                    && uploaded + bytes > upload_budget;
                let (gpu, revision) = if defer {
                    (self.stand_in(source, &mut encoder), DEFERRED_REVISION)
                } else {
                    uploaded += bytes;
                    (
                        self.texture_for(scene, index, &mut encoder),
                        scene.texture_revisions()[index],
                    )
                };
                self.cache.textures.push(gpu);
                self.cache.texture_revisions.push(revision);
                self.cache.sprite_groups.push(None);
                self.cache.ribbon_groups.push(None);
            }
            self.queue.submit(Some(encoder.finish()));
        }
        // Replaced textures: uploaded again, and everything bound to the old
        // view (materials, sprite and trail groups, fonts, the effect
        // groups' decal atlas, terrain layers) made again.
        let mut replaced: Vec<usize> = (0..self.cache.textures.len())
            .filter(|&i| self.cache.texture_revisions[i] != scene.texture_revisions()[i])
            .collect();
        // Fork s63: the deferred textures (and replaced ones) within what is
        // left of the budget.
        if upload_budget > 0 {
            let mut kept = Vec::with_capacity(replaced.len());
            for i in replaced {
                let bytes = if scene.texture_live(i) {
                    source_upload_bytes(&scene.textures()[i])
                } else {
                    0
                };
                if bytes == 0 || uploaded == 0 || uploaded + bytes <= upload_budget {
                    uploaded += bytes;
                    kept.push(i);
                }
            }
            replaced = kept;
        }
        TEXTURES_DEFERRED.store(
            self.cache
                .texture_revisions
                .iter()
                .enumerate()
                .filter(|&(i, &r)| r == DEFERRED_REVISION && !replaced.contains(&i))
                .count() as u32,
            std::sync::atomic::Ordering::Relaxed,
        );
        if !replaced.is_empty() {
            for &index in &replaced {
                if scene.texture_live(index) {
                    self.check_texture(index, &scene.textures()[index])?;
                }
            }
            let mut encoder = Encoder::untimed(self.device.create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("texture replace"),
                },
            ));
            for &index in &replaced {
                let gpu = self.texture_for(scene, index, &mut encoder);
                self.cache.textures[index] = gpu;
                self.cache.texture_revisions[index] = scene.texture_revisions()[index];
                self.cache.sprite_groups[index] = None;
                self.cache.ribbon_groups[index] = None;
            }
            self.queue.submit(Some(encoder.finish()));
            self.fx_groups = None;
            self.terrain.forget_groups();
        }
        // Stream textures' new frames, written in place (no rebinding). A
        // texture uploaded again (its slot replaced) takes its frame again.
        for &index in &replaced {
            self.cache.stream_serials.remove(&index);
        }
        for (&index, (serial, rgba)) in scene.stream_frames() {
            if self.cache.stream_serials.get(&index) == Some(serial) {
                continue;
            }
            let Some(TextureSource::Target(t)) = scene.textures().get(index) else {
                continue;
            };
            let Some(gpu) = self.cache.textures.get(index) else {
                continue;
            };
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: gpu.view.texture(),
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(t.width * 4),
                    rows_per_image: Some(t.height),
                },
                wgpu::Extent3d {
                    width: t.width,
                    height: t.height,
                    depth_or_array_layers: 1,
                },
            );
            self.cache.stream_serials.insert(index, *serial);
        }
        #[cfg(target_arch = "wasm32")]
        self.copy_stream_frames(scene);
        // A pack holding a replaced texture is made again with its materials.
        let replaced_slot = |slot: &u32| {
            TextureHandle::from_slot(*slot).is_some_and(|h| replaced.contains(&h.index()))
        };
        self.cache
            .packs
            .retain(|key, _| !key.iter().any(replaced_slot));
        let samples_replaced = |m: &StandardMaterial| {
            m.textures()
                .iter()
                .chain(m.physical_textures().iter())
                .flatten()
                .any(|h| replaced.contains(&h.index()))
        };
        // The scene's material style: every lit material takes it where it
        // leaves a field at its default, so a change restyles them all.
        let restyle = self.cache.material_style != scene.stylise.material;
        self.cache.material_style = scene.stylise.material;
        // Every material as it was and its uniform up to date: nothing to do
        // (a pass over thousands of materials costs most of a millisecond
        // on the web, engine s96).
        let serial = scene.material_serial();
        let unchanged =
            self.cache.material_serial == Some(serial) && !restyle && replaced.is_empty();
        // The materials whose parameters this frame stamps (their motion
        // this frame): the frame draws each with its own uniform, apart from
        // its aliases (SortedInstances::set_params_moved).
        let mut params_moved = Vec::new();
        for (i, (material, revision)) in scene.materials().enumerate() {
            if unchanged {
                break;
            }
            if let Some(m) = self.cache.materials.get_mut(i)
                && m.revision == revision
                && !samples_replaced(material)
                && !restyle
            {
                // Only the shader's parameters changed: into the uniform as
                // they are (MaterialUniform::custom), the group kept.
                let params = scene
                    .material_shader(MaterialHandle::from_index(i))
                    .map(|h| h.1);
                // With the ones the frames before drew and this frame's
                // number, in one write (engine s98, s100; the three are the
                // uniform's tail, asserted beside MaterialUniform):
                // `previous_shader_params()` reads them in this frame only,
                // so the parameters' motion stops on the next frame, a paused
                // wind's too, with nothing written then. Parameters a game
                // moves every few ticks (wind, sun) change on thousands of
                // materials, and on the web a call costs the Worker and the
                // GPU process more than its bytes.
                if let Some(p) = params.filter(|_| m.params != params) {
                    let change = material::ParamsChange {
                        custom: p,
                        previous_custom: m.params.unwrap_or(p),
                        stamp: [self.frame_index as u32, 0, 0, 0],
                    };
                    self.queue.write_buffer(
                        &m.factors,
                        std::mem::offset_of!(material::MaterialUniform, custom) as u64,
                        bytemuck::bytes_of(&change),
                    );
                    m.params = params;
                    params_moved.push(i as u32);
                }
                continue;
            }
            let styled = material::styled(material, &scene.stylise.material);
            let material = &styled;
            let hooked = scene.material_shader(MaterialHandle::from_index(i));
            if material.draws_array_maps() && hooked.is_none() && self.array_maps.is_none() {
                self.array_maps = Some(material::ArrayMaps::new(
                    &self.device,
                    &self.queue,
                    [&self.frame_layout, &self.fx_layout, &self.palette_layout],
                    self.info
                        .without
                        .features()
                        .leave_out(lit_source_with(&hooks::default_hooks(), self.planar_slot())),
                    shadow_source(&hooks::default_hooks()),
                ));
            }
            // A colour/texture edit rebuilds the group, not the vertex's
            // history: the parameters the frames before drew go into the new
            // uniform with this frame's number (engine s100). A different
            // shader has no comparable previous state.
            let previous_params = self
                .cache
                .materials
                .get(i)
                .filter(|m| m.shader == hooked.map(|h| h.0.index()))
                .and_then(|m| m.params);
            let (bind_group, array, factors) =
                self.material_bind_group(i, material, hooked.map(|h| h.1), previous_params)?;
            if previous_params.is_some_and(|p| Some(p) != hooked.map(|h| h.1)) {
                params_moved.push(i as u32);
            }
            // A full shader (custom.rs) draws a physical material with its
            // own entries: no physical pipelines.
            let full = hooked.is_some_and(|(sh, _)| {
                scene.shaders()[sh.index()].mode != hooks::ShaderMode::Hooks
            });
            let physical = if material.is_physical() && !full {
                if let Some((sh, _)) = hooked
                    && !self.cache.physical_shaders.contains_key(&sh.index())
                {
                    let hooks = scene.shaders()[sh.index()].hooks_wgsl();
                    let module = shader(
                        &self.device,
                        "physical (material shader)",
                        self.info
                            .without
                            .features()
                            .leave_out(physical_source(&hooks)),
                    );
                    self.cache.physical_shaders.insert(sh.index(), module);
                }
                Some(self.physical_bind_group(i, material, &factors)?)
            } else {
                None
            };
            let transmissive = physical.is_some() && material.is_transmissive();
            let gpu = GpuMaterial {
                revision,
                params: hooked.map(|h| h.1),
                factors,
                bind_group,
                physical,
                array,
                side: material.side(),
                transparent: material.is_transparent() || transmissive,
                blend: material.is_transparent(),
                transmissive,
                // Also alpha-tested materials: they cast through the shadow
                // pass that reads the material (fs_dissolve), cut by alpha.
                dissolve: material.dissolves() || material.alphaTest > 0.0,
                effects: material.toon != 0
                    || material.rimIntensity > 0.0
                    || material.flash > 0.0
                    || material.dissolves(),
                outline: material.outlineWidth > 0.0,
                shader: hooked.map(|h| h.0.index()),
                // An array-map instance's transform carries its layer: the
                // occluder depth pass reads transforms as they are.
                occluder: !material.is_transparent()
                    && !transmissive
                    && !material.dissolves()
                    && material.alphaTest <= 0.0
                    && hooked.is_none()
                    && !material.draws_array_maps(),
                lean: false,
                flat: material.flatShading != 0,
                wire_only: material.wireframe == 1,
                wireframe: (material.wireframe != 0).then(|| crate::lines::Wire {
                    color: (material.wireframeColor != glam::Vec3::ZERO).then(|| {
                        let c = material.wireframeColor;
                        [c.x, c.y, c.z, material.opacity]
                    }),
                    angle: material.wireframeAngle.clamp(0.0, 180.0),
                }),
            };
            let gpu = GpuMaterial {
                // The material style's surface reads the mesh's own
                // position, which the lean stages do not hand on.
                lean: gpu.occluder
                    && gpu.physical.is_none()
                    && !gpu.effects
                    && !gpu.outline
                    && !(scene.stylise.material.surface_on()
                        && material.unlit == 0
                        && material.styled != 0)
                    && material.textures().iter().all(Option::is_none),
                ..gpu
            };
            if i < self.cache.materials.len() {
                self.cache.materials[i] = gpu;
            } else {
                self.cache.materials.push(gpu);
            }
        }
        if !unchanged {
            self.cache.material_serial = Some(serial);
        }
        // Also when the pass was skipped: nothing is stamped in that frame.
        self.sorted.set_params_moved(params_moved);
        let stale_fonts: Vec<usize> = (0..self.cache.fonts.len())
            .filter(|&i| {
                replaced.contains(&scene.fonts()[i].atlas.index())
                    || self.cache.fonts[i].revision != scene.font_revisions()[i]
            })
            .collect();
        self.cache
            .fonts
            .truncate(stale_fonts.first().copied().unwrap_or(usize::MAX));
        let start = self.cache.fonts.len();
        for (i, font) in scene.fonts()[start..].iter().enumerate() {
            let index = start + i;
            let atlas = font.atlas.index();
            let (Some(gpu), Some((w, h))) = (
                self.cache.textures.get(atlas),
                scene.textures().get(atlas).map(TextureSource::size),
            ) else {
                return Err(RenderError::InvalidHandle(format!(
                    "font {index}: atlas texture {atlas}"
                )));
            };
            let atlas_size = Vec2::new(w as f32, h as f32);
            let bind_group = sprite_bind_group(
                &self.device,
                &self.queue,
                &self.sprite_layout,
                &gpu.view,
                &self.trilinear_sampler,
                // -1: a page of colour glyph pairs (foreground_mix).
                [
                    if font.foreground {
                        -1.0
                    } else {
                        font.distance_range
                    },
                    atlas_size.x,
                    atlas_size.y,
                    0.0,
                ],
            );
            self.cache.fonts.push(GpuFont {
                layout: FontLayout::new(font, atlas_size),
                bind_group,
                revision: scene.font_revisions()[index],
            });
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
impl Renderer {
    /// The web's decoded VideoFrames into their stream textures, on the GPU
    /// (the browser converts YUV to sRGB RGBA as it copies), each closed
    /// once its copy is issued.
    fn copy_stream_frames(&mut self, scene: &RenderScene) {
        let frames = std::mem::take(&mut *scene.stream_external_cell().borrow_mut());
        for (index, frame) in frames {
            let Some(TextureSource::Target(t)) = scene.textures().get(index) else {
                frame.close();
                continue;
            };
            let Some(gpu) = self.cache.textures.get(index) else {
                frame.close();
                continue;
            };
            let (w, h) = (
                frame.display_width().min(t.width),
                frame.display_height().min(t.height),
            );
            let source = wgpu::CopyExternalImageSourceInfo {
                source: wgpu::ExternalImageSource::VideoFrame(frame),
                origin: wgpu::Origin2d::ZERO,
                flip_y: false,
            };
            self.queue.copy_external_image_to_texture(
                &source,
                wgpu::CopyExternalImageDestInfo {
                    texture: gpu.view.texture(),
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                    color_space: wgpu::PredefinedColorSpace::Srgb,
                    premultiplied_alpha: false,
                },
                wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
            );
            if let wgpu::ExternalImageSource::VideoFrame(f) = &source.source {
                f.close();
            }
        }
    }
}

/// Appends a mesh's morph targets to the morph deltas in their sparse form
/// (`skin.rs`, the palette buffer's layout): `vertices` headers (first
/// entry, entries), then each vertex's entries for the targets that move
/// it, (position delta, target) and (normal delta, 0).
pub(crate) fn pack_morphs(m: &MorphTargets, vertices: usize, out: &mut Vec<[f32; 4]>) {
    let base = out.len();
    out.resize(base + vertices, [0.0; 4]);
    for v in 0..vertices {
        let first = out.len();
        for (t, positions) in m.positions.iter().enumerate() {
            let dp = positions[v];
            let dn = m.normals.as_ref().map_or(Vec3::ZERO, |ns| ns[t][v]);
            if dp != Vec3::ZERO || dn != Vec3::ZERO {
                out.push(dp.extend(t as f32).to_array());
                out.push(dn.extend(0.0).to_array());
            }
        }
        out[base + v] = [first as f32, ((out.len() - first) / 2) as f32, 0.0, 0.0];
    }
}

#[cfg(test)]
mod morph_tests {
    use super::*;

    /// The shader's walk (`skin_storage.wgsl` `morphed`) over the packed
    /// deltas and weight slots: what a vertex moves by.
    fn walk(deltas: &[[f32; 4]], base: usize, v: usize, slots: &[[[f32; 4]; 4]]) -> Vec3 {
        let h = deltas[base + v];
        let (first, count) = (h[0] as usize, h[1] as usize);
        let mut p = Vec3::ZERO;
        for k in 0..count {
            let d = deltas[first + k * 2];
            let t = d[3] as usize;
            let w = slots[t / 16][(t % 16) / 4][t % 4];
            p += Vec3::new(d[0], d[1], d[2]) * w;
        }
        p
    }

    /// 40 targets, each moving a few vertices: the sparse deltas and the
    /// 16-a-slot weights give every vertex the dense weighted sum, past the
    /// old limit of 8 targets and after another mesh's deltas.
    #[test]
    fn sparse_morphs_sum_every_weighted_target() {
        let (n, targets) = (7, 40);
        let positions: Vec<Vec<Vec3>> = (0..targets)
            .map(|t| {
                (0..n)
                    .map(|v| {
                        if (t + v) % 5 == 0 {
                            Vec3::new(t as f32, v as f32, 1.0)
                        } else {
                            Vec3::ZERO
                        }
                    })
                    .collect()
            })
            .collect();
        let m = MorphTargets {
            positions: positions.clone(),
            normals: None,
        };
        let mut deltas = vec![[9.0; 4]; 3];
        pack_morphs(&m, n, &mut deltas);
        let weights: Vec<f32> = (0..targets).map(|t| 0.1 + t as f32 * 0.01).collect();
        let slots = crate::skin::weight_slots(&weights, targets);
        assert_eq!(slots.len(), 3);
        for v in 0..n {
            let dense: Vec3 = positions.iter().zip(&weights).map(|(t, w)| t[v] * w).sum();
            assert!(
                walk(&deltas, 3, v, &slots).abs_diff_eq(dense, 1e-4),
                "vertex {v}"
            );
        }
        // Only the moving (vertex, target) pairs are stored.
        let moving = (0..targets)
            .flat_map(|t| (0..n).map(move |v| (t, v)))
            .filter(|(t, v)| (t + v) % 5 == 0)
            .count();
        assert_eq!(deltas.len(), 3 + n + moving * 2);
    }
}
