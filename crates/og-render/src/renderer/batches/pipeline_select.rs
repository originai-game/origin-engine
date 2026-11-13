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
    ) -> &wgpu::RenderPipeline {
        if b.lean && !skinned {
            return self.lit.lean_pipelines[&(b.faces, b.effects, b.derivatives, b.compact)]
                .get(self.frame_features);
        }
        let key = (b.faces, b.blend, skinned, b.effects);
        if b.array && !physical {
            return &self.lit.array_maps.as_ref().expect("an array batch").lit[&key];
        }
        match (physical, b.shader) {
            (true, None) => {
                &self
                    .lit
                    .physical
                    .as_ref()
                    .expect("a physical batch")
                    .pipelines[&key]
            }
            (true, Some(sh)) => &self.cache.physical_hook_pipelines[&(sh, key)],
            (false, Some(sh)) => {
                &self.cache.hook_pipelines[&(sh, b.faces, b.blend, b.effects, skinned)]
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
                .map(|p| p.get());
        }
        if let Some(sh) = b.shader {
            return self
                .cache
                .hook_prepass_pipelines
                .get(&(sh, b.faces, skinned))
                .map(|p| p.get());
        }
        if b.lean && !skinned {
            return self
                .prepass
                .lean_prepass_pipelines
                .get(&(b.faces, b.derivatives, b.compact))
                .map(|p| p.get(self.frame_features));
        }
        if b.array {
            return self
                .lit
                .array_maps
                .as_ref()
                .and_then(|a| a.prepass.get(&(b.faces, skinned)))
                .map(|p| p.get());
        }
        self.prepass
            .prepass_pipelines
            .get(&(b.faces, skinned))
            .map(|p| p.get(self.frame_features))
    }

    /// Makes the lit, shadow and prepass pipelines this frame's batches need,
    /// with `local_shadows` (a shadowed local light in the scene) the local
    /// layers' shadow pipelines of every caster too (shadow_target.rs).
    /// Fork s60: asynchronous on the WebGPU tier while the host asks for it.
    pub(crate) fn ensure_pipelines(
        &mut self,
        batches: &[Batch],
        skinned: &[Batch],
        prepass: bool,
        local_shadows: bool,
    ) {
        frame_pipelines(|| {
            self.ensure_pipelines_now(batches, skinned, prepass);
            if local_shadows {
                self.ensure_local_shadow_pipelines();
            }
        });
    }

    /// The local layers' pipelines of every caster's shadow set made so far.
    fn ensure_local_shadow_pipelines(&self) {
        let array = self.lit.array_maps.iter().flat_map(|a| a.shadow.values());
        for set in self
            .shadows
            .shadow_pipelines
            .values()
            .chain(self.cache.hook_shadow_pipelines.values())
            .chain(self.shadows.compact_shadow_pipelines.values())
            .chain(array)
        {
            set.local();
        }
    }

    fn ensure_pipelines_now(&mut self, batches: &[Batch], skinned: &[Batch], prepass: bool) {
        let (features, declared) = (self.frame_features, self.lit.pbr_shader.declared);
        for (slot, work) in self.pipeline_jobs(batches, skinned, prepass) {
            use Made::{Pipeline as P, Shadows as S};
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
                        self.lit
                            .array_maps
                            .as_mut()
                            .expect("array material")
                            .shadow
                            .insert(k, set);
                    }
                    Slot::HookShadow(k) => {
                        self.cache.hook_shadow_pipelines.insert(k, set);
                    }
                    Slot::CompactShadow(k) => {
                        self.shadows.compact_shadow_pipelines.insert(k, set);
                    }
                    Slot::Shadow(k) => {
                        self.shadows.shadow_pipelines.insert(k, set);
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
