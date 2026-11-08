//! Batch construction, culling and pipeline selection.

use super::*;

impl Renderer {
    /// Sorts the instances into (mesh, material) batches
    /// ([`instances::SortedInstances`]: from the last frame's sort while the
    /// submission's keys hold); returns the batches and the sorted
    /// instances (transforms, world bounding spheres, submission indices),
    /// lent to the frame until [`Renderer::restore_sorted`].
    pub(crate) fn build_batches(
        &mut self,
        scene: &RenderScene,
    ) -> Result<(Vec<Batch>, instances::Sorted), RenderError> {
        let (meshes, materials) = (&self.cache.meshes, &self.cache.materials);
        let sorted = self.sorted.update(
            scene,
            meshes.len(),
            |m| (meshes[m].bounds, meshes[m].index_count == 0),
            materials.len(),
            |mesh, material| Self::dequant(meshes, materials, mesh, material),
            |material| materials[material].array.is_some(),
            |material| materials[material].wire_only,
        )?;
        let batches = sorted
            .runs
            .iter()
            .map(|r| {
                let m = &self.cache.materials[r.material];
                Batch {
                    mesh: r.mesh,
                    material: r.material,
                    first: r.first,
                    count: r.count,
                    side: m.side,
                    transparent: m.transparent,
                    blend: m.blend,
                    physical: m.physical.is_some(),
                    transmissive: m.transmissive,
                    dissolve: m.dissolve,
                    effects: m.effects,
                    shader: m.shader,
                    lean: m.lean,
                    derivatives: m.flat,
                    compact: r.dequant.is_some(),
                    array: r.array,
                    foliage: scene.mesh_foliage_all()[r.mesh].is_some(),
                    simulated: scene.mesh_surfaces[r.mesh].is_some(),
                    cast: r.cast,
                    layers: r.layers,
                    probe: r.probe,
                    crowd: None,
                }
            })
            .collect();
        Ok((batches, sorted))
    }

    /// How a (mesh, material) run undoes its vertices' quantisation
    /// ([`instances::Run::dequant`]): a plain material drawing a compact
    /// mesh draws its compact vertices; anything else the f32 ones.
    pub(crate) fn dequant(
        meshes: &[GpuMesh],
        materials: &[GpuMaterial],
        mesh: usize,
        material: usize,
    ) -> Option<Mat4> {
        (materials[material].lean && meshes[mesh].foliage.is_none())
            .then(|| meshes[mesh].compact.as_ref().map(|(_, dq)| dq.matrix()))
            .flatten()
    }

    /// The draw ranges of one pass: per batch (`include`d), every instance
    /// whose sphere touches `frustum`. With `whole`, a batch that loses
    /// less than a tenth of its instances is drawn whole from the sorted
    /// instances (cheaper than copying its transforms); otherwise, and
    /// always without `whole`, its visible instances are appended to
    /// `transforms`. A frame under GPU culling passes `whole` false: its
    /// sorted instances live in the culling's buffer, and the CPU lists
    /// (views, probe faces) draw only what they appended, from the
    /// instance buffer. `scratch` is the caller's, reused across the
    /// frame's passes; a batch drawn whole is marked in its `whole`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn cull(
        &self,
        batches: &[Batch],
        spheres: &[(Vec3, f32)],
        transforms: &mut Vec<instances::GpuTransform>,
        origins: &mut Vec<u32>,
        scratch: &mut CullScratch,
        frustum: &Frustum,
        whole: bool,
        include: impl Fn(&Batch) -> bool,
    ) -> (DrawList, u32) {
        let mut list = DrawList::default();
        let mut culled = 0;
        let mut visible = Vec::new();
        let inside = &mut scratch.inside;
        scratch.whole.resize(batches.len(), false);
        // Only the included batches' spheres are tested: they are marked
        // first, and the test leaves an unmarked one alone (no read of it
        // follows). On the lanes only from a six-figure count of bounds.
        inside.clear();
        inside.resize(spheres.len(), false);
        for b in batches.iter().filter(|b| include(b)) {
            inside[b.first as usize..(b.first + b.count) as usize].fill(true);
        }
        frustum.test_spheres(inside, spheres);
        for (b_index, b) in batches.iter().enumerate() {
            if !include(b) {
                continue;
            }
            let range = b.first as usize..(b.first + b.count) as usize;
            visible.clear();
            visible.extend(range.clone().filter(|&k| inside[k]));
            culled += b.count - visible.len() as u32;
            let index_count = self.cache.meshes[b.mesh].index_count;
            list.in_view_triangles += (index_count / 3) as u64 * visible.len() as u64;
            let (first, count) = if visible.is_empty() {
                continue;
            } else if whole && visible.len() as u32 * 10 >= b.count * 9 {
                scratch.whole[b_index] = true;
                (b.first, b.count)
            } else {
                let first = transforms.len() as u32;
                for &k in &visible {
                    transforms.push(transforms[k]);
                    origins.push(origins[k]);
                }
                (first, visible.len() as u32)
            };
            list.ranges.push((b_index as u32, first, count));
            list.instances += count;
            list.triangles += (index_count / 3) as u64 * count as u64;
        }
        (list, culled)
    }

    /// Whether the effect group carries the planar reflection material
    /// shaders read ([`planar_slot`] for this renderer's planar
    /// reflections).
    pub(crate) fn planar_slot(&self) -> bool {
        planar_slot(self.info.planar_reflections)
    }

    /// An effect group without the planar reflection, for the lightmapped
    /// pipelines (`fx_lean_layout`); `None` where the effect group has no
    /// planar reflection and serves them itself.
    pub(crate) fn fx_lean_group(
        &self,
        buffer: &wgpu::Buffer,
        ao: &wgpu::TextureView,
        atlas: &wgpu::TextureView,
    ) -> Option<wgpu::BindGroup> {
        Some(crate::fx::fx_group(
            &self.device,
            "fx (lightmapped)",
            self.fx_lean_layout.as_ref()?,
            &[
                buffer.as_entire_binding(),
                wgpu::BindingResource::TextureView(ao),
                wgpu::BindingResource::TextureView(atlas),
                wgpu::BindingResource::Sampler(&self.trilinear_sampler),
            ],
            &self.decal_buffers,
        ))
    }

    /// The transparent instances in view, sorted back to front by the view
    /// depth of their bounds' centre (ties keep submission order), as runs of
    /// one (mesh, material) appended to `transforms`. `billboards`: the
    /// depths, far first, of the billboards drawn among them (the pass's
    /// [`sprites::SpriteFrame::billboards`]): a run is cut where one falls
    /// between two of its instances, and [`DrawList::before`] says how many
    /// draw before each run (a billboard as far as a mesh draws after it).
    /// Returns the list and how many were outside the view.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn sort_transparent(
        &self,
        batches: &[Batch],
        spheres: &[(Vec3, f32)],
        submission: &[u32],
        transforms: &mut Vec<instances::GpuTransform>,
        origins: &mut Vec<u32>,
        frustum: &Frustum,
        eye: Vec3,
        forward: Vec3,
        layers: u32,
        billboards: &[f32],
    ) -> (DrawList, u32) {
        let mut keyed: Vec<(f32, u32, usize, u32)> = Vec::new();
        let mut culled = 0;
        let drawn = |b: &Batch| b.transparent && b.layers & layers != 0;
        for (b_index, b) in batches.iter().enumerate().filter(|(_, b)| drawn(b)) {
            for k in b.first as usize..(b.first + b.count) as usize {
                let (c, r) = spheres[k];
                if frustum.intersects_sphere(c, r) {
                    keyed.push(((c - eye).dot(forward), submission[k], k, b_index as u32));
                } else {
                    culled += 1;
                }
            }
        }
        og_core::sort::sort_by(&mut keyed, |a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        let mut list = DrawList::default();
        let mut farther = 0;
        for (depth, _, k, b) in keyed {
            let at = transforms.len() as u32;
            transforms.push(transforms[k]);
            origins.push(origins[k]);
            let before = farther;
            while farther < billboards.len() && billboards[farther] > depth {
                farther += 1;
            }
            match list.ranges.last_mut() {
                Some(r) if r.0 == b && r.1 + r.2 == at && farther == before => r.2 += 1,
                _ => {
                    list.ranges.push((b, at, 1));
                    list.before.push(farther as u32);
                }
            }
            list.instances += 1;
            list.triangles += (self.cache.meshes[batches[b as usize].mesh].index_count / 3) as u64;
        }
        (list, culled)
    }

    /// World bounds of every instance as (center, radius), from their box
    /// ([`instances::fold_box`]; `None`: no instance).
    pub(crate) fn instance_bounds(bounds: Option<(Vec3, Vec3)>) -> (Vec3, f32) {
        let Some((lo, hi)) = bounds else {
            return (Vec3::ZERO, 1.0);
        };
        ((lo + hi) * 0.5, ((hi - lo).length() * 0.5).max(1e-3))
    }

    /// Texture `t`'s normal-mapped group (sprite_normals.rs) when 2D
    /// lighting is on and it has a normal map, made or made again when
    /// what it reads changed; the pipelines on first use.
    /// Material `m`'s sprite pipelines and parameter group
    /// (`sprite_hooks.rs`).
    pub(crate) fn sprite_hooks_prepare(&mut self, scene: &RenderScene, m: MaterialHandle) {
        let Some((shader, params)) = scene.sprite_shader(m) else {
            return;
        };
        let hooks = self.sprite_hooks.get_or_insert_with(|| {
            sprite_hooks::SpriteHooks::new(
                &self.device,
                [
                    &self.frame_layout,
                    &self.sprite_layout,
                    &self.sprite_fx_layout,
                ],
                self.samples,
            )
        });
        hooks.prepare_shader(
            &self.device,
            shader.index(),
            &scene.shaders()[shader.index()],
        );
        hooks.prepare_material(&self.device, &self.queue, m.index(), params);
    }

    pub(crate) fn sprite_normal_group(&mut self, scene: &RenderScene, t: TextureHandle) {
        let Some(n) = scene
            .texture_normal(t)
            .filter(|_| scene.lighting_2d.is_some())
        else {
            return;
        };
        let (i, ni) = (t.index(), n.index());
        if ni >= self.cache.textures.len() {
            return;
        }
        let filter = self.cache.filters.get(i).copied().unwrap_or_default();
        let key = (
            self.cache.texture_revisions[i],
            ni,
            self.cache.texture_revisions[ni],
            filter.is_nearest(),
            filter == TextureFilter::CleanEdge,
        );
        let normals = self.sprite_normals.get_or_insert_with(|| {
            sprite_normals::SpriteNormals::new(
                &self.device,
                [&self.frame_layout, &self.sprite_fx_layout],
                self.samples,
            )
        });
        if normals.groups.len() <= i {
            normals.groups.resize_with(i + 1, || None);
        }
        if normals.groups[i].as_ref().is_some_and(|(_, k)| *k == key) {
            return;
        }
        let params = [0.0, 0.0, 0.0, key.4 as u32 as f32];
        let buffer = crate::buffer_init(
            &self.device,
            &self.queue,
            &wgpu::util::BufferInitDescriptor {
                label: Some("sprite params (normal map)"),
                contents: bytemuck::bytes_of(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            },
        );
        let group = bind_group(
            &self.device,
            "sprite (normal map)",
            &normals.layout,
            &[
                wgpu::BindingResource::TextureView(&self.cache.textures[i].view),
                wgpu::BindingResource::Sampler(if key.3 {
                    &self.sprite_sampler_nearest
                } else {
                    &self.sprite_sampler
                }),
                buffer.as_entire_binding(),
                wgpu::BindingResource::TextureView(&self.cache.textures[ni].view),
            ],
        );
        normals.groups[i] = Some((group, key));
    }

    /// Texture `t`'s normal-mapped group if this frame draws it so.
    pub(crate) fn normal_group(
        &self,
        scene: &RenderScene,
        t: Option<TextureHandle>,
    ) -> Option<&wgpu::BindGroup> {
        let t = t?;
        scene.lighting_2d?;
        scene.texture_normal(t)?;
        self.sprite_normals.as_ref()?.group(t.index())
    }

    pub(crate) fn sprite_group(&mut self, texture: Option<TextureHandle>) -> &wgpu::BindGroup {
        let Some(t) = texture else {
            return &self.white_sprite_group;
        };
        let filter = self
            .cache
            .filters
            .get(t.index())
            .copied()
            .unwrap_or_default();
        let slot = &mut self.cache.sprite_groups[t.index()];
        if slot.is_none() {
            *slot = Some(sprite_bind_group(
                &self.device,
                &self.queue,
                &self.sprite_layout,
                &self.cache.textures[t.index()].view,
                if filter.is_nearest() {
                    &self.sprite_sampler_nearest
                } else {
                    &self.sprite_sampler
                },
                // sprite.wgsl's `msdf.w`: 1 samples through cleanEdge.
                [
                    0.0,
                    0.0,
                    0.0,
                    (filter == TextureFilter::CleanEdge) as u32 as f32,
                ],
            ));
        }
        slot.as_ref().expect("made above")
    }

    /// The lightmapped pipeline family a batch draws with (lightmap.rs):
    /// every batch of a lightmapped mesh, its kind's (`physical`: drawn
    /// with the physical pipelines in this pass, as `lit_pipeline_of`).
    pub(crate) fn lightmap_family(
        lightmaps: &lightmap::Lightmaps,
        b: &Batch,
        physical: bool,
    ) -> Option<lightmap::Family> {
        use lightmap::Family;
        lightmaps.uvs(b.mesh)?;
        Some(match (physical, b.shader) {
            (true, None) => Family::Physical,
            (true, Some(sh)) => Family::PhysicalShader(sh),
            (false, _) if b.array => Family::Array,
            (false, Some(sh)) => Family::Shader(sh),
            (false, None) if b.compact => Family::Compact,
            (false, None) => Family::Standard,
        })
    }

    /// The lit pipeline of a batch (made by [`Renderer::ensure_pipelines`]):
    /// its material shader's or the built-in one, physical or standard.
    pub(crate) fn lit_pipeline_of(
        &self,
        b: &Batch,
        skinned: bool,
        physical: bool,
    ) -> &wgpu::RenderPipeline {
        if b.lean && !skinned {
            return &self.lean_pipelines[&(b.side, b.effects, b.derivatives, b.compact)];
        }
        let key = (b.side, b.blend, skinned, b.effects);
        if b.array && !physical {
            return &self.array_maps.as_ref().expect("an array batch").lit[&key];
        }
        match (physical, b.shader) {
            (true, None) => &self.physical.as_ref().expect("a physical batch").pipelines[&key],
            (true, Some(sh)) => &self.cache.physical_hook_pipelines[&(sh, key)],
            (false, Some(sh)) => {
                &self.cache.hook_pipelines[&(sh, b.side, b.blend, b.effects, skinned)]
            }
            (false, None) => &self.pbr_pipelines[&key],
        }
    }

    /// The depth prepass pipeline of a batch (made by `ensure_pipelines`
    /// with its prepass): a material shader's, the lean or array-map
    /// variant, or the standard one; `None` before it is made.
    pub(crate) fn prepass_pipeline_of(
        &self,
        b: &Batch,
        skinned: bool,
    ) -> Option<&wgpu::RenderPipeline> {
        if !skinned && b.simulated && self.cache.meshes[b.mesh].surface.is_some() {
            return self
                .cache
                .surface_prepass_pipelines
                .get(&(b.shader, b.side, b.array))
                .map(|p| p.get());
        }
        if let Some(sh) = b.shader {
            return self
                .cache
                .hook_prepass_pipelines
                .get(&(sh, b.side, skinned))
                .map(|p| p.get());
        }
        if b.lean && !skinned {
            return self
                .lean_prepass_pipelines
                .get(&(b.side, b.derivatives, b.compact))
                .map(|p| p.get());
        }
        if b.array {
            return self
                .array_maps
                .as_ref()
                .and_then(|a| a.prepass.get(&(b.side, skinned)))
                .map(|p| p.get());
        }
        self.prepass_pipelines
            .get(&(b.side, skinned))
            .map(|p| p.get())
    }

    /// Makes the lit, shadow and prepass pipelines this frame's batches need.
    /// Fork s60: asynchronous on the WebGPU tier while the host asks for it.
    pub(crate) fn ensure_pipelines(&mut self, batches: &[Batch], skinned: &[Batch], prepass: bool) {
        frame_pipelines(|| self.ensure_pipelines_now(batches, skinned, prepass));
    }

    fn ensure_pipelines_now(&mut self, batches: &[Batch], skinned: &[Batch], prepass: bool) {
        for (slot, job) in self.pipeline_jobs(batches, skinned, prepass) {
            use Made::{Pipeline as P, Shadows as S};
            // Natively each compiles on the compile workers (deferred.rs)
            // and the pass that first sets it waits for it; on the web it
            // is made here, inside the host's frame span.
            if let Slot::ArrayShadow(_)
            | Slot::HookShadow(_)
            | Slot::CompactShadow(_)
            | Slot::Shadow(_) = slot
            {
                let set = deferred(move || match job() {
                    S(set) => set,
                    P(_) => unreachable!("a shadow slot's job makes shadows"),
                });
                match slot {
                    Slot::ArrayShadow(k) => {
                        self.array_maps
                            .as_mut()
                            .expect("array material")
                            .shadow
                            .insert(k, set);
                    }
                    Slot::HookShadow(k) => {
                        self.cache.hook_shadow_pipelines.insert(k, set);
                    }
                    Slot::CompactShadow(k) => {
                        self.compact_shadow_pipelines.insert(k, set);
                    }
                    Slot::Shadow(k) => {
                        self.shadow_pipelines.insert(k, set);
                    }
                    _ => unreachable!("a shadow slot"),
                }
                continue;
            }
            let p = deferred(move || match job() {
                P(p) => p,
                S(_) => unreachable!("a pipeline slot's job makes a pipeline"),
            });
            match slot {
                Slot::SurfacePrepass(k) => {
                    self.cache.surface_prepass_pipelines.insert(k, p);
                }
                Slot::ArrayLit(k) => {
                    self.array_maps
                        .as_mut()
                        .expect("array material")
                        .lit
                        .insert(k, p);
                }
                Slot::ArrayPrepass(k) => {
                    self.array_maps
                        .as_mut()
                        .expect("array material")
                        .prepass
                        .insert(k, p);
                }
                Slot::Physical(k) => {
                    self.physical
                        .as_mut()
                        .expect("physical material")
                        .pipelines
                        .insert(k, p);
                }
                Slot::PhysicalHook(k) => {
                    self.cache.physical_hook_pipelines.insert(k, p);
                }
                Slot::Hook(k) => {
                    self.cache.hook_pipelines.insert(k, p);
                }
                Slot::HookPrepass(k) => {
                    self.cache.hook_prepass_pipelines.insert(k, p);
                }
                Slot::Lean(k) => {
                    self.lean_pipelines.insert(k, p);
                }
                Slot::LeanPrepass(k) => {
                    self.lean_prepass_pipelines.insert(k, p);
                }
                Slot::Pbr(k) => {
                    self.pbr_pipelines.insert(k, p);
                }
                Slot::Prepass(k) => {
                    self.prepass_pipelines.insert(k, p);
                }
                _ => unreachable!("the shadow slots are matched above"),
            }
        }
    }

    /// The pipelines this frame's batches need and no map holds yet, each
    /// with the slot it goes in, once each; each job owns what it makes its
    /// pipeline from, so it can run on another thread.
    fn pipeline_jobs(
        &self,
        batches: &[Batch],
        skinned: &[Batch],
        prepass: bool,
    ) -> Vec<(Slot, Job)> {
        let device = &self.device;
        let samples = self.samples;
        let pbr = &(self.pbr_layout.clone(), self.pbr_skinned_layout.clone());
        let pbr_shader = &self.pbr_shader;
        let shadow_shader = &self.shadow_shader;
        let shadow_layouts = &self.shadow_layouts;
        let mut jobs: Vec<(Slot, Job)> = Vec::new();
        let mut add = |slot: Slot, held: bool, job: Job| {
            if !held && !jobs.iter().any(|(s, _)| *s == slot) {
                jobs.push((slot, job));
            }
        };
        let all = batches
            .iter()
            .map(|b| (b, false))
            .chain(skinned.iter().map(|b| (b, true)));
        for (b, is_skinned) in all {
            let key = (b.shader, b.side, b.array);
            if prepass
                && !is_skinned
                && !b.transparent
                && self.cache.meshes[b.mesh].surface.is_some()
            {
                let (layouts, module) = if b.array {
                    let a = self.array_maps.as_ref().expect("array material");
                    (a.layouts.clone(), Shader::Made(a.module.clone()))
                } else {
                    (
                        pbr.clone(),
                        b.shader.map_or(Shader::Lazy(pbr_shader.clone()), |sh| {
                            Shader::Made(self.cache.shaders[sh].clone())
                        }),
                    )
                };
                let (device, side, array) = (device.clone(), b.side, b.array);
                add(
                    Slot::SurfacePrepass(key),
                    self.cache.surface_prepass_pipelines.contains_key(&key),
                    Box::new(move || {
                        Made::Pipeline(surface_prepass_pipeline(
                            &device,
                            (&layouts.0, &layouts.1),
                            &module,
                            side,
                            array,
                        ))
                    }),
                );
            }
            if b.array {
                // The array-map variant: the standard builders over its
                // module and layouts (`material::ArrayMaps`).
                let a = self.array_maps.as_ref().expect("made with the material");
                let key = (b.side, b.blend, is_skinned, b.effects);
                let (d, layouts, module) = (device.clone(), a.layouts.clone(), a.module.clone());
                add(
                    Slot::ArrayLit(key),
                    a.lit.contains_key(&key),
                    Box::new(move || {
                        Made::Pipeline(pbr_pipeline(
                            &d,
                            (&layouts.0, &layouts.1),
                            &module,
                            key,
                            samples,
                        ))
                    }),
                );
                let key = (b.side, is_skinned);
                if prepass && !b.transparent {
                    let (d, layouts, module, side) =
                        (device.clone(), a.layouts.clone(), a.module.clone(), b.side);
                    add(
                        Slot::ArrayPrepass(key),
                        a.prepass.contains_key(&key),
                        Box::new(move || {
                            Made::Pipeline(prepass_pipeline(
                                &d,
                                (&layouts.0, &layouts.1),
                                &module,
                                side,
                                is_skinned,
                                None,
                                None,
                            ))
                        }),
                    );
                }
                let key = (b.side, b.dissolve);
                if !is_skinned {
                    let (d, layouts, module) = (
                        device.clone(),
                        shadow_layouts.clone(),
                        a.shadow_module.clone(),
                    );
                    let (side, dissolve) = key;
                    add(
                        Slot::ArrayShadow(key),
                        a.shadow.contains_key(&key),
                        Box::new(move || {
                            Made::Shadows(shadow_pipeline(
                                &d,
                                &layouts,
                                &module,
                                (side, false, dissolve),
                                false,
                                false,
                            ))
                        }),
                    );
                }
            }
            if b.physical {
                // The physical lit pipeline (the standard ones below draw the
                // material's reflection).
                let key = (b.side, b.blend, is_skinned, b.effects);
                let physical = self.physical.as_ref().expect("made with the material");
                let layouts = physical.layouts.clone();
                let d = device.clone();
                match b.shader {
                    None => {
                        let module = physical.module.clone();
                        add(
                            Slot::Physical(key),
                            physical.pipelines.contains_key(&key),
                            Box::new(move || {
                                Made::Pipeline(lit_pipeline(
                                    &d,
                                    (&layouts.0, &layouts.1),
                                    &module,
                                    key,
                                    samples,
                                    true,
                                ))
                            }),
                        );
                    }
                    Some(sh) => {
                        let module = self.cache.physical_shaders[&sh].clone();
                        add(
                            Slot::PhysicalHook((sh, key)),
                            self.cache.physical_hook_pipelines.contains_key(&(sh, key)),
                            Box::new(move || {
                                Made::Pipeline(lit_pipeline(
                                    &d,
                                    (&layouts.0, &layouts.1),
                                    &module,
                                    key,
                                    samples,
                                    true,
                                ))
                            }),
                        );
                    }
                }
            }
            if let Some(sh) = b.shader {
                let key = (sh, b.side, b.blend, b.effects, is_skinned);
                let lit = (b.side, b.blend, is_skinned, b.effects);
                let (d, layouts, module) =
                    (device.clone(), pbr.clone(), self.cache.shaders[sh].clone());
                add(
                    Slot::Hook(key),
                    self.cache.hook_pipelines.contains_key(&key),
                    Box::new(move || {
                        Made::Pipeline(pbr_pipeline(
                            &d,
                            (&layouts.0, &layouts.1),
                            &module,
                            lit,
                            samples,
                        ))
                    }),
                );
                // Shadows and the prepass see the material shader's offset.
                let key = (sh, b.side, is_skinned);
                let shadow = (b.side, is_skinned, b.dissolve);
                let (d, layouts, module) = (
                    device.clone(),
                    shadow_layouts.clone(),
                    self.cache.shadow_shaders[sh].clone(),
                );
                add(
                    Slot::HookShadow(key),
                    self.cache.hook_shadow_pipelines.contains_key(&key),
                    Box::new(move || {
                        Made::Shadows(shadow_pipeline(&d, &layouts, &module, shadow, true, false))
                    }),
                );
                if prepass && !b.transparent {
                    let (d, layouts, module, side) = (
                        device.clone(),
                        pbr.clone(),
                        self.cache.shaders[sh].clone(),
                        b.side,
                    );
                    add(
                        Slot::HookPrepass(key),
                        self.cache.hook_prepass_pipelines.contains_key(&key),
                        Box::new(move || {
                            Made::Pipeline(prepass_pipeline(
                                &d,
                                (&layouts.0, &layouts.1),
                                &module,
                                side,
                                is_skinned,
                                None,
                                None,
                            ))
                        }),
                    );
                }
            }
            if b.lean && !is_skinned {
                // A plain material: the lean lit and prepass pipelines (its
                // shadows take the shadow pipeline below, lean as well).
                let key = (b.side, b.effects, b.derivatives, b.compact);
                let (d, layout, module) =
                    (device.clone(), self.pbr_layout.clone(), pbr_shader.clone());
                add(
                    Slot::Lean(key),
                    self.lean_pipelines.contains_key(&key),
                    Box::new(move || {
                        Made::Pipeline(lean_pipeline(&d, &layout, &module, key, samples))
                    }),
                );
                let key = (b.side, b.derivatives, b.compact);
                if prepass {
                    let (d, layouts, module) = (device.clone(), pbr.clone(), pbr_shader.clone());
                    let (side, lean) = (b.side, (b.derivatives, b.compact));
                    add(
                        Slot::LeanPrepass(key),
                        self.lean_prepass_pipelines.contains_key(&key),
                        Box::new(move || {
                            Made::Pipeline(prepass_pipeline(
                                &d,
                                (&layouts.0, &layouts.1),
                                &module,
                                side,
                                false,
                                Some(lean),
                                None,
                            ))
                        }),
                    );
                }
                if b.compact {
                    let (d, layouts, module, side) = (
                        device.clone(),
                        shadow_layouts.clone(),
                        shadow_shader.clone(),
                        b.side,
                    );
                    add(
                        Slot::CompactShadow(side),
                        self.compact_shadow_pipelines.contains_key(&side),
                        Box::new(move || {
                            Made::Shadows(shadow_pipeline(
                                &d,
                                &layouts,
                                &module,
                                (side, false, false),
                                false,
                                true,
                            ))
                        }),
                    );
                }
            }
            let key = (b.side, b.blend, is_skinned, b.effects);
            if !(b.lean && !is_skinned) {
                let (d, layouts, module) = (device.clone(), pbr.clone(), pbr_shader.clone());
                add(
                    Slot::Pbr(key),
                    self.pbr_pipelines.contains_key(&key),
                    Box::new(move || {
                        Made::Pipeline(pbr_pipeline(
                            &d,
                            (&layouts.0, &layouts.1),
                            &module,
                            key,
                            samples,
                        ))
                    }),
                );
            }
            let key = (b.side, is_skinned, b.dissolve);
            let (d, layouts, module) = (
                device.clone(),
                shadow_layouts.clone(),
                shadow_shader.clone(),
            );
            add(
                Slot::Shadow(key),
                self.shadow_pipelines.contains_key(&key),
                Box::new(move || {
                    Made::Shadows(shadow_pipeline(&d, &layouts, &module, key, false, false))
                }),
            );
            if prepass && !b.transparent && !(b.lean && !is_skinned) {
                let key = (b.side, is_skinned);
                let (d, layouts, module) = (device.clone(), pbr.clone(), pbr_shader.clone());
                add(
                    Slot::Prepass(key),
                    self.prepass_pipelines.contains_key(&key),
                    Box::new(move || {
                        Made::Pipeline(prepass_pipeline(
                            &d,
                            (&layouts.0, &layouts.1),
                            &module,
                            key.0,
                            key.1,
                            None,
                            None,
                        ))
                    }),
                );
            }
        }
        jobs
    }

    /// The skinned instances as (mesh, material) batches over the skinned
    /// instance buffer, their instance data, and the frame's joint matrices.
    #[allow(clippy::type_complexity)]
    pub(crate) fn build_skinned(
        &mut self,
        scene: &RenderScene,
        probes: &[(Vec3, Vec3)],
    ) -> Result<SkinnedFrame, RenderError> {
        let mut offsets = Vec::with_capacity(scene.joint_palettes.len());
        let mut matrices = Vec::new();
        let mut garments = Vec::new();
        for p in &scene.joint_palettes {
            offsets.push((matrices.len() as u32, p.matrices.len() as u32));
            matrices.extend(p.matrices.iter().map(|m| m.to_cols_array_2d()));
        }
        if matrices.len() > MAX_JOINTS {
            return Err(RenderError::InvalidData(format!(
                "{} joints in the frame's palettes (at most {MAX_JOINTS})",
                matrices.len()
            )));
        }
        for (i, s) in scene.skinned_instances.iter().enumerate() {
            let mesh = s.mesh.index();
            if mesh >= self.cache.meshes.len() || self.cache.skins[mesh].is_none() {
                return Err(RenderError::InvalidHandle(format!(
                    "skinned instance {i}: mesh {mesh} has neither a skin nor morph targets"
                )));
            }
            if s.material.index() >= self.cache.materials.len() {
                return Err(RenderError::InvalidHandle(format!(
                    "skinned instance {i}: material {}",
                    s.material.index()
                )));
            }
            match &s.baked {
                Some(b) => {
                    for c in [b.clip, b.previous_clip] {
                        let ok = scene.baked_clips.get(c as usize).is_some_and(|c| {
                            c.joints > 0
                                && c.frames.len() % c.joints as usize == 0
                                && c.frame_count() >= 2
                        });
                        if !ok {
                            return Err(RenderError::InvalidData(format!(
                                "skinned instance {i}: baked clip {c} of {} (a clip holds whole frames, at least two)",
                                scene.baked_clips.len()
                            )));
                        }
                    }
                    if s.deformed.is_some() {
                        return Err(RenderError::InvalidData(format!(
                            "skinned instance {i}: a baked clip and deformed vertices"
                        )));
                    }
                }
                None if s.palette as usize >= offsets.len() => {
                    return Err(RenderError::InvalidHandle(format!(
                        "skinned instance {i}: palette {} of {}",
                        s.palette,
                        offsets.len()
                    )));
                }
                None => {}
            }
        }
        // The clips the skinned instances and the crowds play, now or the
        // frame before (a crowd's members mostly share one: marked, not
        // inserted one by one).
        let mut marked = vec![false; scene.baked_clips.len()];
        let mut mark = |b: &skin::BakedPose| {
            for c in [b.clip, b.previous_clip] {
                if let Some(m) = marked.get_mut(c as usize) {
                    *m = true;
                }
            }
        };
        scene
            .skinned_instances
            .iter()
            .filter_map(|s| s.baked.as_ref())
            .for_each(&mut mark);
        scene
            .crowds
            .iter()
            .flat_map(|c| &c.members)
            .for_each(|m| mark(&m.pose));
        // And the clips crowds' impostors are baked from.
        for i in scene.crowds.iter().filter_map(|c| c.impostor.as_ref()) {
            if let Some(m) = marked.get_mut(i.clip as usize) {
                *m = true;
            }
        }
        let used: std::collections::BTreeSet<u32> = marked
            .iter()
            .enumerate()
            .filter(|(_, m)| **m)
            .map(|(i, _)| i as u32)
            .collect();
        let baked_joints: usize = used
            .iter()
            .map(|&i| scene.baked_clips[i as usize].frames.len())
            .sum();
        if baked_joints + matrices.len() > MAX_JOINTS {
            return Err(RenderError::InvalidData(format!(
                "{} joints in the frame's palettes and bakes (at most {MAX_JOINTS})",
                baked_joints + matrices.len()
            )));
        }
        self.palette
            .prepare_bakes(&self.device, &self.queue, &scene.baked_clips, &used);
        let mut order: Vec<usize> = (0..scene.skinned_instances.len()).collect();
        // Each instance's reflection probe: the one holding the centre of
        // its posed bounds, its joints' boxes moved by its palette
        // (skin.rs; a morphed mesh without a skin: its bind pose's), so a
        // figure walked out of a room by its root joint takes the room it
        // is in (probe_select.rs).
        let probe_of: Vec<u8> = scene
            .skinned_instances
            .iter()
            .map(|s| {
                let mesh = s.mesh.index();
                // A baked instance: its bind pose's centre.
                let posed = match s.baked {
                    Some(_) => None,
                    None => self.cache.joint_boxes.get(mesh).and_then(|b| {
                        let palette = &scene.joint_palettes[s.palette as usize].matrices;
                        skin::posed_centre(b, palette, &s.transform)
                    }),
                };
                let centre = posed.unwrap_or_else(|| {
                    self.cache
                        .meshes
                        .get(mesh)
                        .and_then(|m| m.bounds)
                        .map_or(s.transform.w_axis.truncate(), |(c, _)| {
                            s.transform.transform_point3(c)
                        })
                });
                probe_select::pick(probes, centre)
            })
            .collect();
        // A (mesh, material)'s casters first, its non-casters a batch of
        // their own (left out of the shadow passes), then by probe.
        og_core::sort::sort_by_key(&mut order, |&i| {
            let s = &scene.skinned_instances[i];
            (
                s.mesh,
                s.material,
                !s.cast_shadows,
                views::layer_of(&scene.skinned_layers, i),
                probe_of[i],
            )
        });
        let mut batches: Vec<Batch> = Vec::new();
        let mut data = Vec::with_capacity(order.len());
        let mut boxes = Vec::with_capacity(order.len());
        // Each (mesh, baked clip)'s box over every frame, once a frame.
        let mut clip_boxes: std::collections::HashMap<(usize, u32), Option<(Vec3, Vec3)>> =
            Default::default();
        let mut previous_slots = Vec::new();
        let mut baked_moved = false;
        let taa = self.info.antialias.temporal() || scene.post.motion_blur.is_some();
        for &i in &order {
            let s = &scene.skinned_instances[i];
            let (mesh, material, cast) = (s.mesh.index(), s.material.index(), s.cast_shadows);
            let layers = views::layer_of(&scene.skinned_layers, i);
            let probe = probe_of[i];
            if self.cache.meshes[mesh].index_count == 0 {
                continue;
            }
            match batches.last_mut() {
                Some(b)
                    if b.mesh == mesh
                        && b.material == material
                        && b.cast == cast
                        && b.layers == layers
                        && b.probe == probe =>
                {
                    b.count += 1;
                    b.simulated |= s.deformed.is_some();
                }
                _ => {
                    let m = &self.cache.materials[material];
                    batches.push(Batch {
                        mesh,
                        material,
                        first: data.len() as u32,
                        count: 1,
                        side: m.side,
                        transparent: m.transparent,
                        blend: m.blend,
                        physical: m.physical.is_some(),
                        transmissive: m.transmissive,
                        dissolve: m.dissolve,
                        effects: m.effects,
                        shader: m.shader,
                        lean: false,
                        derivatives: m.flat,
                        compact: false,
                        array: m.array.is_some(),
                        foliage: false,
                        simulated: s.deformed.is_some(),
                        cast,
                        layers,
                        probe,
                        crowd: None,
                    });
                }
            }
            let (morph_first, targets) = self.cache.morph_ranges[mesh];
            let joints = self
                .cache
                .joint_boxes
                .get(mesh)
                .map_or(&[][..], Vec::as_slice);
            let clip = s.baked.and_then(|b| {
                *clip_boxes
                    .entry((mesh, b.clip))
                    .or_insert_with(|| skin::clip_box(joints, &scene.baked_clips[b.clip as usize]))
            });
            let morphing = targets > 0
                && s.morph_weights[..targets as usize]
                    .iter()
                    .any(|&w| w != 0.0);
            boxes.push(skin::skinned_box(
                s,
                self.cache.meshes[mesh].bounds,
                joints,
                match s.baked {
                    Some(_) => &[],
                    None => &scene.joint_palettes[s.palette as usize].matrices,
                },
                clip,
                morphing,
            ));
            if let Some(b) = s.baked {
                // Bakes stay in their own buffer; only the previous
                // transform belongs to the frame's dynamic palette.
                let clip = &scene.baked_clips[b.clip as usize];
                let (k, t) = clip.at(b.frame);
                let now = self.palette.baked_offset(b.clip) + k * clip.joints as usize;
                let before = &scene.baked_clips[b.previous_clip as usize];
                let (pk, pt) = before.at(b.previous_frame);
                let then = self.palette.baked_offset(b.previous_clip) + pk * before.joints as usize;
                // Motion effects read it; otherwise a crowd costs its clips alone.
                let mut transform_at = 0;
                if taa {
                    transform_at = matrices.len() + 1;
                    matrices.push(b.previous_transform.to_cols_array_2d());
                    previous_slots.push(b.previous_transform.to_cols_array_2d());
                }
                baked_moved |= b.clip != b.previous_clip
                    || b.frame != b.previous_frame
                    || b.previous_transform != s.transform;
                data.push(skin::SkinnedInstanceGpu {
                    transform: instances::gpu_columns(&s.transform),
                    palette: [now as f32, clip.joints as f32, morph_first as f32, -1.0 - t],
                    morph0: [0.0; 4],
                    morph1: [transform_at as f32, then as f32, pt, 0.0],
                    look: {
                        let [flash, color, tint] = instances::fx_bits(&s.fx);
                        [
                            instances::layer_bits(0.0, s.receive_shadows),
                            flash,
                            color,
                            tint,
                        ]
                    },
                });
                continue;
            }
            let (first, count) = offsets[s.palette as usize];
            // Its weights' slots after the joints (`skin.rs`, the layout);
            // none while every weight is zero.
            let targets_n = targets as usize;
            let mut weights_at = 0.0;
            if targets_n > 0 && s.morph_weights[..targets_n].iter().any(|&w| w != 0.0) {
                weights_at = (matrices.len() + 1) as f32;
                matrices.extend(skin::weight_slots(&s.morph_weights, targets_n));
                let before = scene
                    .previous_morph_weights
                    .get(i)
                    .filter(|_| scene.previous_morph_weights.len() == scene.skinned_instances.len())
                    .unwrap_or(&s.morph_weights);
                previous_slots.extend(skin::weight_slots(before, targets_n));
            }
            // Its own vertices (a garment), after its weights.
            let mut deformed_at = 0.0;
            if let Some(d) = s.deformed {
                let Some(now) = scene.deformed_vertices.get(d as usize) else {
                    return Err(RenderError::InvalidData(format!(
                        "skinned instance {i}: deformed vertices {d} of {}",
                        scene.deformed_vertices.len()
                    )));
                };
                let n = scene.mesh(s.mesh).map_or(0, |m| m.vertices.len());
                if now.vertex_count() != n || (now.garment.is_none() && now.normals.len() != n) {
                    return Err(RenderError::InvalidData(format!(
                        "skinned instance {i}: {} deformed positions and {} normals for a mesh of {n} vertices",
                        now.positions.len(),
                        now.normals.len()
                    )));
                }
                deformed_at = (matrices.len() + 1) as f32;
                let before = scene
                    .previous_deformed_vertices
                    .get(d as usize)
                    .filter(|b| {
                        scene.previous_deformed_vertices.len() == scene.deformed_vertices.len()
                            && b.vertex_count() == n
                            && (b.garment.is_some() || b.normals.len() == n)
                    })
                    .unwrap_or(now);
                if self.garments.is_some() && now.garment.is_some() {
                    garments.push((d, matrices.len() as u32));
                    matrices.resize(matrices.len() + n.div_ceil(2), [[0.; 4]; 4]);
                    previous_slots.resize(previous_slots.len() + n.div_ceil(2), [[0.; 4]; 4]);
                } else {
                    matrices.extend(skin::deformed_slots(now));
                    previous_slots.extend(skin::deformed_slots(before));
                }
            }
            data.push(skin::SkinnedInstanceGpu {
                transform: instances::gpu_columns(&s.transform),
                palette: [
                    first as f32,
                    count as f32,
                    morph_first as f32,
                    targets as f32,
                ],
                morph0: [weights_at, deformed_at, 0.0, 0.0],
                morph1: [0.0; 4],
                look: {
                    let [flash, color, tint] = instances::fx_bits(&s.fx);
                    [
                        instances::layer_bits(0.0, s.receive_shadows),
                        flash,
                        color,
                        tint,
                    ]
                },
            });
        }
        if matrices.len() + baked_joints > MAX_JOINTS {
            return Err(RenderError::InvalidData(format!(
                "{} joints, blendshape weight and deformed vertex slots in the frame's palette (at most {MAX_JOINTS})",
                matrices.len() + baked_joints
            )));
        }
        Ok(SkinnedFrame {
            batches,
            garments,
            data,
            boxes,
            matrices,
            previous_slots,
            baked_moved,
        })
    }

    /// The scene's crowds as the crowd cull's tables (crowd.rs `build`),
    /// their previous transforms with them when `motion`.
    pub(crate) fn build_crowds(
        &self,
        scene: &RenderScene,
        motion: bool,
    ) -> Result<crate::crowd::CrowdFrame, RenderError> {
        let cache = &self.cache;
        crate::crowd::build(
            &scene.crowds,
            &scene.baked_clips,
            &|m| {
                Some(crate::crowd::MeshInfo {
                    skinned: cache.skins.get(m)?.is_some(),
                    index_count: cache.meshes.get(m)?.index_count,
                    joints: cache.joint_boxes.get(m).map_or(&[][..], Vec::as_slice),
                })
            },
            cache.materials.len(),
            &|m| cache.materials[m].shader.is_some(),
            &|c| self.palette.baked_offset(c),
            motion,
            scene.lod_bias,
        )
    }

    /// One skinned batch per crowd draw (a part at a level): no instances
    /// of its own; on the GPU path it draws the crowd cull's records
    /// ([`Batch::crowd`]), on the CPU path [`Self::append_crowds`] gives it
    /// runs of records.
    pub(crate) fn crowd_batches(
        &self,
        scene: &RenderScene,
        frame: &crate::crowd::CrowdFrame,
    ) -> Vec<Batch> {
        frame
            .draws
            .iter()
            .enumerate()
            .map(|(k, d)| {
                let m = &self.cache.materials[d.material];
                let c = &scene.crowds[d.crowd as usize];
                Batch {
                    mesh: d.mesh,
                    material: d.material,
                    first: 0,
                    count: 0,
                    side: m.side,
                    transparent: m.transparent,
                    blend: m.blend,
                    physical: m.physical.is_some(),
                    transmissive: m.transmissive,
                    dissolve: m.dissolve,
                    effects: m.effects,
                    shader: m.shader,
                    lean: false,
                    derivatives: m.flat,
                    compact: false,
                    array: m.array.is_some(),
                    foliage: false,
                    simulated: false,
                    cast: c.cast_shadows,
                    layers: c.layers,
                    probe: crate::probe_select::NONE,
                    crowd: Some(k as u32),
                }
            })
            .collect()
    }

    /// The crowds drawn as skinned instances (the CPU path, crowd.rs):
    /// each member at the camera's level, a record per part, appended to
    /// `skinned` as batches of `crowd_batches`' kinds with their boxes.
    /// `previous_at`: the palette slot of the first member's previous
    /// transform (TAA). Answers the card-level members' records, by card
    /// (`(card, first, count)` runs over them).
    #[allow(clippy::type_complexity)]
    pub(crate) fn append_crowds(
        &self,
        scene: &RenderScene,
        frame: &crate::crowd::CrowdFrame,
        crowd_batches: &[Batch],
        skinned: &mut SkinnedFrame,
        previous_at: Option<u32>,
    ) -> (Vec<(u32, u32, u32)>, Vec<crate::skin::SkinnedInstanceGpu>) {
        let crate::crowd::CpuCrowds {
            runs,
            data,
            boxes,
            card_runs,
            cards,
        } = crate::crowd::cpu_records(
            &scene.crowds,
            frame,
            &scene.camera,
            self.height as f32,
            scene.lod_bias,
            previous_at,
        );
        let base = skinned.data.len() as u32;
        for (draw, first, count) in runs {
            skinned.batches.push(Batch {
                first: base + first,
                count,
                crowd: None,
                ..crowd_batches[draw as usize]
            });
        }
        skinned.data.extend(data);
        skinned.boxes.extend(boxes);
        skinned.baked_moved |= frame.moved;
        (card_runs, cards)
    }

    /// The skinned batches the camera sees: instances whose posed box
    /// touches `frustum`, appended to `frame.data` as new runs.
    /// Only the batches on a layer of `layers` (`views.rs`).
    pub(crate) fn cull_skinned(
        frame: &mut SkinnedFrame,
        frustum: &Frustum,
        layers: u32,
    ) -> (Vec<Batch>, u32) {
        let mut visible = Vec::new();
        let mut culled = 0;
        for b in frame.batches.iter().filter(|b| b.layers & layers != 0) {
            let first = frame.data.len() as u32;
            for k in b.first as usize..(b.first + b.count) as usize {
                let (lo, hi) = frame.boxes[k];
                if frustum.intersects_box(lo, hi) {
                    let d = frame.data[k];
                    frame.data.push(d);
                } else {
                    culled += 1;
                }
            }
            let count = frame.data.len() as u32 - first;
            if count > 0 {
                visible.push(Batch { first, count, ..*b });
            }
        }
        (visible, culled)
    }

    pub(crate) fn ribbon_group(&mut self, texture: Option<TextureHandle>) {
        let Some(t) = texture else {
            return;
        };
        if self.cache.ribbon_groups[t.index()].is_none() {
            self.cache.ribbon_groups[t.index()] = Some(sprite_bind_group(
                &self.device,
                &self.queue,
                &self.sprite_layout,
                &self.cache.textures[t.index()].view,
                &self.ribbon_sampler,
                [0.0; 4],
            ));
        }
    }
}

/// Where a batch pipeline [`Renderer::ensure_pipelines`] made goes: the map
/// and its key.
#[derive(Clone, Copy, PartialEq)]
enum Slot {
    SurfacePrepass((Option<usize>, Side, bool)),
    ArrayLit(PbrKey),
    ArrayPrepass((Side, bool)),
    ArrayShadow((Side, bool)),
    Physical(PbrKey),
    PhysicalHook((usize, PbrKey)),
    Hook((usize, Side, bool, bool, bool)),
    HookShadow((usize, Side, bool)),
    HookPrepass((usize, Side, bool)),
    Lean(LeanKey),
    LeanPrepass((Side, bool, bool)),
    CompactShadow(Side),
    Pbr(PbrKey),
    Shadow(ShadowKey),
    Prepass((Side, bool)),
}

/// What a batch pipeline's job makes.
enum Made {
    Pipeline(wgpu::RenderPipeline),
    Shadows(ShadowSet),
}

/// A batch pipeline's making, which owns what it reads: run on the
/// compile workers natively (deferred.rs).
#[cfg(not(target_arch = "wasm32"))]
type Job = Box<dyn FnOnce() -> Made + Send>;
#[cfg(target_arch = "wasm32")]
type Job = Box<dyn FnOnce() -> Made>;

/// A material shader's module, or the built-in one parsed on first use.
enum Shader {
    Made(wgpu::ShaderModule),
    Lazy(crate::deferred::Module),
}

impl std::ops::Deref for Shader {
    type Target = wgpu::ShaderModule;
    fn deref(&self) -> &wgpu::ShaderModule {
        match self {
            Shader::Made(m) => m,
            Shader::Lazy(m) => m.get(),
        }
    }
}

/// A batch pipeline made by `make`: started on the compile workers
/// natively, made at once on the web (inside the host's frame span, which
/// asks for it asynchronously).
fn deferred<T: Send + 'static>(
    make: impl FnOnce() -> T + crate::deferred::AnywhereOnce + 'static,
) -> Deferred<T> {
    let d = Deferred::anywhere(make);
    #[cfg(not(target_arch = "wasm32"))]
    crate::deferred::Start::start(&d);
    #[cfg(target_arch = "wasm32")]
    d.get();
    d
}
