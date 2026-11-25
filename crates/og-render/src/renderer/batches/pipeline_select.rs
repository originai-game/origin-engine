//! Which pipeline a batch draws with in each pass, and making the ones a frame needs.

use super::*;

impl Renderer {
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
    ) -> Option<&wgpu::RenderPipeline> {
        if b.lean && !skinned {
            return self.lit.lean_pipelines[&(b.faces, b.effects, b.dissolve, b.compact)]
                .get(self.frame_features);
        }
        let key = (b.faces, b.blend, skinned, b.effects, b.dissolve);
        if b.array && !physical {
            return self.lit.array_maps.as_ref().expect("an array batch").lit[&key].get();
        }
        match (physical, b.shader) {
            (true, None) => self
                .lit
                .physical
                .as_ref()
                .expect("a physical batch")
                .pipelines[&key]
                .get(),
            (true, Some(sh)) => {
                let hook_key = (b.faces, b.blend, skinned, b.effects, true);
                self.cache.physical_hook_pipelines[&(sh, hook_key)].get()
            }
            (false, Some(sh)) => {
                self.cache.hook_pipelines[&(sh, b.faces, b.blend, b.effects, skinned)].get()
            }
            (false, None) => self.lit.pbr_pipelines[&key].get(self.frame_features),
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
                .get(&(b.shader, b.faces, b.array))
                .and_then(|p| p.get());
        }
        if let Some(sh) = b.shader {
            return self
                .cache
                .hook_prepass_pipelines
                .get(&(sh, b.faces, skinned))
                .and_then(|p| p.get());
        }
        if b.lean && !skinned {
            return self
                .prepass
                .lean_prepass_pipelines
                .get(&(b.faces, b.dissolve, b.compact))
                .and_then(|p| p.get(self.frame_features));
        }
        if b.array {
            return self
                .lit
                .array_maps
                .as_ref()
                .and_then(|a| a.prepass.get(&(b.faces, skinned, b.dissolve)))
                .and_then(|p| p.get());
        }
        self.prepass
            .prepass_pipelines
            .get(&(b.faces, skinned, b.dissolve))
            .and_then(|p| p.get(self.frame_features))
    }

    /// Makes the lit, shadow and prepass pipelines this frame's batches
    /// need. A caster's shadow set is made whole: its cascade pipeline and,
    /// where the renderer draws local lights' shadows
    /// ([`RenderInfo::shadowed_lights`]), its local layers', since any light
    /// may start casting on a later frame (shadow_target.rs).
    pub(crate) fn ensure_pipelines(&mut self, batches: &[Batch], skinned: &[Batch], prepass: bool) {
        self.ensure_pipelines_now(batches, skinned, prepass);
    }

    /// Starts the pipelines of `ahead` (the batches of instances not drawn
    /// yet, [`Renderer::ahead_batches`]) as a frame's own are started, and
    /// waits for none: natively they compile on the compile workers while
    /// the frames go on, and [`Renderer::pipelines_compiling`] says when
    /// they are made, on every host.
    pub(crate) fn ensure_ahead(&mut self, ahead: &[Batch], prepass: bool) {
        self.instances.ahead = !ahead.is_empty();
        if !ahead.is_empty() {
            self.ensure_pipelines_now(ahead, &[], prepass);
        }
    }

    /// Whether a batch pipeline started for this frame's features is not
    /// made yet, without waiting for any. A pass waits for the pipeline of
    /// a batch it draws, so at the end of a frame only those made ahead
    /// ([`Renderer::ensure_ahead`]) can still compile.
    pub(crate) fn pipelines_compiling(&self) -> bool {
        let f = self.frame_features;
        let varying = self
            .lit
            .lean_pipelines
            .values()
            .chain(self.lit.pbr_pipelines.values())
            .chain(self.prepass.lean_prepass_pipelines.values())
            .chain(self.prepass.prepass_pipelines.values())
            .any(|v| !v.ready(f));
        let array = self.lit.array_maps.iter();
        let physical = self.lit.physical.iter();
        let once = (self.cache.surface_prepass_pipelines.values())
            .chain(self.cache.hook_pipelines.values())
            .chain(self.cache.hook_prepass_pipelines.values())
            .chain(self.cache.physical_hook_pipelines.values())
            .chain(
                array
                    .clone()
                    .flat_map(|a| a.lit.values().chain(a.prepass.values())),
            )
            .chain(physical.flat_map(|p| p.pipelines.values()))
            .any(|p| p.poll().is_none());
        let shadows = (self.shadows.shadow_pipelines.values())
            .chain(self.shadows.compact_shadow_pipelines.values())
            .chain(self.cache.hook_shadow_pipelines.values())
            .chain(array.flat_map(|a| a.shadow.values()))
            .any(|s| s.poll().is_none());
        varying || once || shadows
    }

    /// Makes the pipelines of one loaded material's batch
    /// ([`Renderer::loaded_batches`]) that no frame has made: its lit,
    /// prepass and shadow ones and, blended, its order-independent one in
    /// the mode a frame of this size runs. False when none is left
    /// ([`Renderer::warm`]): something spawned with a material the game
    /// loaded at start then draws its first frame without compiling.
    pub(crate) fn warm_loaded_materials(&mut self) -> bool {
        let (read, loaded, prepass) = std::mem::take(&mut self.loaded);
        let samples = self.samples;
        let oit = self.info.caps.contains(Caps::MRT | Caps::INDEPENDENT_BLEND);
        let mode = loaded
            .iter()
            .any(|b| b.transparent && b.blend)
            .then(|| {
                oit.then(|| {
                    self.oit
                        .frame_mode(&self.device, &self.queue, self.width, self.height)
                })
            })
            .flatten();
        let mut made = false;
        for b in &loaded {
            let lit = !self
                .pipeline_jobs(std::slice::from_ref(b), &[], prepass)
                .is_empty();
            let physical = b.physical && self.lit.physical.is_some();
            let source = Self::oit_lit_source(b, false, physical);
            let blended = mode.filter(|_| b.transparent && b.blend).filter(|&m| {
                let modes: &[oit::Oit] = if m == oit::Oit::Front {
                    &[oit::Oit::Front, oit::Oit::Mark]
                } else {
                    &[m]
                };
                (modes.iter()).any(|&m| !self.oit.pipelines.contains_key(&(source, m, samples)))
            });
            if lit {
                self.ensure_pipelines_now(std::slice::from_ref(b), &[], prepass);
            }
            if let Some(m) = blended {
                self.ensure_oit_pipelines(m, std::slice::from_ref(b), &[], &[], &[], &[], &[]);
            }
            if lit || blended.is_some() {
                made = true;
                break;
            }
        }
        self.loaded = (read, loaded, prepass);
        made
    }

    fn ensure_pipelines_now(&mut self, batches: &[Batch], skinned: &[Batch], prepass: bool) {
        let (features, declared) = (self.frame_features, self.lit.pbr_shader.declared);
        for (slot, work) in self.pipeline_jobs(batches, skinned, prepass) {
            let job = match work {
                Work::Once(job) => job,
                Work::Variants(job) => {
                    let v = variants(declared, features, job);
                    match slot {
                        Slot::Lean(k) => self.lit.lean_pipelines.insert(k, v),
                        Slot::LeanPrepass(k) => self.prepass.lean_prepass_pipelines.insert(k, v),
                        Slot::Pbr(k) => self.lit.pbr_pipelines.insert(k, v),
                        Slot::Prepass(k) => self.prepass.prepass_pipelines.insert(k, v),
                        _ => unreachable!("only the built-in lit and prepass slots vary"),
                    };
                    continue;
                }
            };
            // Each is asked for here, ahead of the pass that first sets it
            // (gpu/deferred.rs).
            let p = deferred(move || match job() {
                Made::Pipeline(p) => p,
            });
            match slot {
                Slot::ArrayShadow(k) => {
                    self.lit
                        .array_maps
                        .as_mut()
                        .expect("array material")
                        .shadow
                        .insert(k, p);
                }
                Slot::HookShadow(k) => {
                    self.cache.hook_shadow_pipelines.insert(k, p);
                }
                Slot::CompactShadow(k) => {
                    self.shadows.compact_shadow_pipelines.insert(k, p);
                }
                Slot::Shadow(k) => {
                    self.shadows.shadow_pipelines.insert(k, p);
                }
                Slot::SurfacePrepass(k) => {
                    self.cache.surface_prepass_pipelines.insert(k, p);
                }
                Slot::ArrayLit(k) => {
                    self.lit
                        .array_maps
                        .as_mut()
                        .expect("array material")
                        .lit
                        .insert(k, p);
                }
                Slot::ArrayPrepass(k) => {
                    self.lit
                        .array_maps
                        .as_mut()
                        .expect("array material")
                        .prepass
                        .insert(k, p);
                }
                Slot::Physical(k) => {
                    self.lit
                        .physical
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
                _ => unreachable!("the shadow and varying slots are matched above"),
            }
        }
    }
}
