//! The pipeline jobs of a frame's batches: one per slot wanted and not yet made.

use super::*;

/// Queues a slot's pipeline job in `jobs` when the slot is wanted, neither
/// `held` nor queued, and only then evaluates `work`: making a job clones the
/// device, its layouts and module and boxes its builder, which a frame whose
/// pipelines are all made did for every batch and pass, to drop
/// ([`Renderer::pipeline_jobs`]; engine s109).
macro_rules! add_job {
    ($jobs:expr, $slot:expr, $held:expr, $work:expr $(,)?) => {{
        let slot = $slot;
        if !$held && !$jobs.iter().any(|(s, _)| *s == slot) {
            $jobs.push((slot, $work));
        }
    }};
}

impl Renderer {
    /// The pipelines this frame's batches need and no map holds yet, each
    /// with the slot it goes in, once each; each job owns what it makes its
    /// pipeline from, so it can run on another thread.
    pub(super) fn pipeline_jobs(
        &self,
        batches: &[Batch],
        skinned: &[Batch],
        prepass: bool,
    ) -> Vec<(Slot, Work)> {
        let device = &self.device;
        let samples = self.samples;
        let pbr = &(
            self.lit.pbr_layout.clone(),
            self.lit.pbr_skinned_layout.clone(),
        );
        let pbr_shader = &self.lit.pbr_shader;
        let shadow_shader = &self.shadows.shadow_shader;
        let shadow_layouts = &self.shadows.shadow_layouts;
        let mut jobs: Vec<(Slot, Work)> = Vec::new();
        let all = batches
            .iter()
            .map(|b| (b, false))
            .chain(skinned.iter().map(|b| (b, true)));
        for (b, is_skinned) in all {
            let key = (b.shader, b.faces, b.array);
            if prepass
                && !is_skinned
                && !b.transparent
                && self.cache.meshes[b.mesh].surface.is_some()
            {
                add_job!(
                    jobs,
                    Slot::SurfacePrepass(key),
                    self.cache.surface_prepass_pipelines.contains_key(&key),
                    {
                        let (layouts, module) = if b.array {
                            let a = self.lit.array_maps.as_ref().expect("array material");
                            (a.layouts.clone(), Shader::Made(a.module.clone()))
                        } else {
                            (
                                pbr.clone(),
                                b.shader.map_or(Shader::Lazy(pbr_shader.clone()), |sh| {
                                    Shader::Made(self.cache.shaders[sh].clone())
                                }),
                            )
                        };
                        let (device, faces, array) = (device.clone(), b.faces, b.array);
                        Work::Once(Box::new(move || {
                            Made::Pipeline(surface_prepass_pipeline(
                                &device,
                                (&layouts.0, &layouts.1),
                                &module,
                                faces,
                                array,
                            ))
                        }))
                    }
                );
            }
            if b.array {
                // The array-map variant: the standard builders over its
                // module and layouts (`material::ArrayMaps`).
                let a = self
                    .lit
                    .array_maps
                    .as_ref()
                    .expect("made with the material");
                let key = (b.faces, b.blend, is_skinned, b.effects);
                add_job!(jobs, Slot::ArrayLit(key), a.lit.contains_key(&key), {
                    let (d, layouts, module) =
                        (device.clone(), a.layouts.clone(), a.module.clone());
                    Work::Once(Box::new(move || {
                        Made::Pipeline(pbr_pipeline(
                            &d,
                            (&layouts.0, &layouts.1),
                            &module,
                            key,
                            samples,
                        ))
                    }))
                });
                let key = (b.faces, is_skinned);
                if prepass && !b.transparent {
                    add_job!(
                        jobs,
                        Slot::ArrayPrepass(key),
                        a.prepass.contains_key(&key),
                        {
                            let (d, layouts, module, side) =
                                (device.clone(), a.layouts.clone(), a.module.clone(), b.faces);
                            Work::Once(Box::new(move || {
                                Made::Pipeline(prepass_pipeline(
                                    &d,
                                    (&layouts.0, &layouts.1),
                                    &module,
                                    side,
                                    is_skinned,
                                    None,
                                    None,
                                    &[],
                                ))
                            }))
                        }
                    );
                }
                let key = (b.faces, b.dissolve);
                if !is_skinned {
                    add_job!(jobs, Slot::ArrayShadow(key), a.shadow.contains_key(&key), {
                        let (d, layouts, module) = (
                            device.clone(),
                            shadow_layouts.clone(),
                            a.shadow_module.clone(),
                        );
                        let (side, dissolve) = key;
                        Work::Once(Box::new(move || {
                            Made::Shadows(shadow_pipeline(
                                &d,
                                &layouts,
                                &module,
                                (side, false, dissolve),
                                false,
                                false,
                            ))
                        }))
                    });
                }
            }
            if b.physical {
                // The physical lit pipeline (the standard ones below draw the
                // material's reflection).
                let key = (b.faces, b.blend, is_skinned, b.effects);
                let physical = self.lit.physical.as_ref().expect("made with the material");
                match b.shader {
                    None => {
                        add_job!(
                            jobs,
                            Slot::Physical(key),
                            physical.pipelines.contains_key(&key),
                            {
                                let layouts = physical.layouts.clone();
                                let d = device.clone();
                                let module = physical.module.clone();
                                Work::Once(Box::new(move || {
                                    Made::Pipeline(lit_pipeline(
                                        &d,
                                        (&layouts.0, &layouts.1),
                                        &module,
                                        key,
                                        samples,
                                        true,
                                        &[],
                                    ))
                                }))
                            }
                        );
                    }
                    Some(sh) => {
                        add_job!(
                            jobs,
                            Slot::PhysicalHook((sh, key)),
                            self.cache.physical_hook_pipelines.contains_key(&(sh, key)),
                            {
                                let layouts = physical.layouts.clone();
                                let d = device.clone();
                                let module = self.cache.physical_shaders[&sh].clone();
                                Work::Once(Box::new(move || {
                                    Made::Pipeline(lit_pipeline(
                                        &d,
                                        (&layouts.0, &layouts.1),
                                        &module,
                                        key,
                                        samples,
                                        true,
                                        &[],
                                    ))
                                }))
                            }
                        );
                    }
                }
            }
            if let Some(sh) = b.shader {
                let key = (sh, b.faces, b.blend, b.effects, is_skinned);
                let lit = (b.faces, b.blend, is_skinned, b.effects);
                add_job!(
                    jobs,
                    Slot::Hook(key),
                    self.cache.hook_pipelines.contains_key(&key),
                    {
                        let (d, layouts, module) =
                            (device.clone(), pbr.clone(), self.cache.shaders[sh].clone());
                        Work::Once(Box::new(move || {
                            Made::Pipeline(pbr_pipeline(
                                &d,
                                (&layouts.0, &layouts.1),
                                &module,
                                lit,
                                samples,
                            ))
                        }))
                    }
                );
                // Shadows and the prepass see the material shader's offset.
                let key = (sh, b.faces, is_skinned);
                let shadow = (b.faces, is_skinned, b.dissolve);
                add_job!(
                    jobs,
                    Slot::HookShadow(key),
                    self.cache.hook_shadow_pipelines.contains_key(&key),
                    {
                        let (d, layouts, module) = (
                            device.clone(),
                            shadow_layouts.clone(),
                            self.cache.shadow_shaders[sh].clone(),
                        );
                        Work::Once(Box::new(move || {
                            Made::Shadows(shadow_pipeline(
                                &d, &layouts, &module, shadow, true, false,
                            ))
                        }))
                    }
                );
                if prepass && !b.transparent {
                    add_job!(
                        jobs,
                        Slot::HookPrepass(key),
                        self.cache.hook_prepass_pipelines.contains_key(&key),
                        {
                            let (d, layouts, module, side) = (
                                device.clone(),
                                pbr.clone(),
                                self.cache.shaders[sh].clone(),
                                b.faces,
                            );
                            Work::Once(Box::new(move || {
                                Made::Pipeline(prepass_pipeline(
                                    &d,
                                    (&layouts.0, &layouts.1),
                                    &module,
                                    side,
                                    is_skinned,
                                    None,
                                    None,
                                    &[],
                                ))
                            }))
                        }
                    );
                }
            }
            if b.lean && !is_skinned {
                // A plain material: the lean lit and prepass pipelines (its
                // shadows take the shadow pipeline below, lean as well).
                let key = (b.faces, b.effects, b.derivatives, b.compact);
                add_job!(
                    jobs,
                    Slot::Lean(key),
                    self.lit.lean_pipelines.contains_key(&key),
                    {
                        let (d, layout, module) = (
                            device.clone(),
                            self.lit.pbr_layout.clone(),
                            pbr_shader.clone(),
                        );
                        Work::Variants(variant_job(move |off: &[(&'static str, f64)]| {
                            Made::Pipeline(lean_pipeline(&d, &layout, &module, key, samples, off))
                        }))
                    }
                );
                let key = (b.faces, b.derivatives, b.compact);
                if prepass {
                    add_job!(
                        jobs,
                        Slot::LeanPrepass(key),
                        self.prepass.lean_prepass_pipelines.contains_key(&key),
                        {
                            let (d, layouts, module) =
                                (device.clone(), pbr.clone(), pbr_shader.clone());
                            let (side, lean) = (b.faces, (b.derivatives, b.compact));
                            Work::Variants(variant_job(move |off: &[(&'static str, f64)]| {
                                Made::Pipeline(prepass_pipeline(
                                    &d,
                                    (&layouts.0, &layouts.1),
                                    &module,
                                    side,
                                    false,
                                    Some(lean),
                                    None,
                                    off,
                                ))
                            }))
                        }
                    );
                }
                if b.compact {
                    add_job!(
                        jobs,
                        Slot::CompactShadow(b.faces),
                        self.shadows.compact_shadow_pipelines.contains_key(&b.faces),
                        {
                            let (d, layouts, module, side) = (
                                device.clone(),
                                shadow_layouts.clone(),
                                shadow_shader.clone(),
                                b.faces,
                            );
                            Work::Once(Box::new(move || {
                                Made::Shadows(shadow_pipeline(
                                    &d,
                                    &layouts,
                                    &module,
                                    (side, false, false),
                                    false,
                                    true,
                                ))
                            }))
                        }
                    );
                }
            }
            let key = (b.faces, b.blend, is_skinned, b.effects);
            if !(b.lean && !is_skinned) {
                add_job!(
                    jobs,
                    Slot::Pbr(key),
                    self.lit.pbr_pipelines.contains_key(&key),
                    {
                        let (d, layouts, module) =
                            (device.clone(), pbr.clone(), pbr_shader.clone());
                        Work::Variants(variant_job(move |off: &[(&'static str, f64)]| {
                            Made::Pipeline(lit_pipeline(
                                &d,
                                (&layouts.0, &layouts.1),
                                &module,
                                key,
                                samples,
                                false,
                                off,
                            ))
                        }))
                    }
                );
            }
            let key = (b.faces, is_skinned, b.dissolve);
            add_job!(
                jobs,
                Slot::Shadow(key),
                self.shadows.shadow_pipelines.contains_key(&key),
                {
                    let (d, layouts, module) = (
                        device.clone(),
                        shadow_layouts.clone(),
                        shadow_shader.clone(),
                    );
                    Work::Once(Box::new(move || {
                        Made::Shadows(shadow_pipeline(&d, &layouts, &module, key, false, false))
                    }))
                }
            );
            if prepass && !b.transparent && !(b.lean && !is_skinned) {
                let key = (b.faces, is_skinned);
                add_job!(
                    jobs,
                    Slot::Prepass(key),
                    self.prepass.prepass_pipelines.contains_key(&key),
                    {
                        let (d, layouts, module) =
                            (device.clone(), pbr.clone(), pbr_shader.clone());
                        Work::Variants(variant_job(move |off: &[(&'static str, f64)]| {
                            Made::Pipeline(prepass_pipeline(
                                &d,
                                (&layouts.0, &layouts.1),
                                &module,
                                key.0,
                                key.1,
                                None,
                                None,
                                off,
                            ))
                        }))
                    }
                );
            }
        }
        jobs
    }
}

/// Where a batch pipeline [`Renderer::ensure_pipelines`] made goes: the map
/// and its key.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Slot {
    SurfacePrepass((Option<usize>, Faces, bool)),
    ArrayLit(PbrKey),
    ArrayPrepass((Faces, bool)),
    ArrayShadow((Faces, bool)),
    Physical(PbrKey),
    PhysicalHook((usize, PbrKey)),
    Hook((usize, Faces, bool, bool, bool)),
    HookShadow((usize, Faces, bool)),
    HookPrepass((usize, Faces, bool)),
    Lean(LeanKey),
    LeanPrepass((Faces, bool, bool)),
    CompactShadow(Faces),
    Pbr(PbrKey),
    Shadow(ShadowKey),
    Prepass((Faces, bool)),
}

/// What a batch pipeline's job makes.
pub(crate) enum Made {
    Pipeline(wgpu::RenderPipeline),
    Shadows(ShadowSet),
}

/// A batch pipeline's making, which owns what it reads: run on the
/// compile workers natively (deferred.rs).
#[cfg(not(target_arch = "wasm32"))]
pub(super) type Job = Box<dyn FnOnce() -> Made + Send>;

#[cfg(target_arch = "wasm32")]
pub(super) type Job = Box<dyn FnOnce() -> Made>;

/// A built-in lit or prepass pipeline's making with the constants that
/// leave features out (pipeline_features.rs).
#[cfg(not(target_arch = "wasm32"))]
pub(crate) type VariantJob = Arc<dyn Fn(&[(&'static str, f64)]) -> Made + Send + Sync>;

#[cfg(target_arch = "wasm32")]
pub(crate) type VariantJob = std::rc::Rc<dyn Fn(&[(&'static str, f64)]) -> Made>;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn variant_job(
    job: impl Fn(&[(&'static str, f64)]) -> Made + Send + Sync + 'static,
) -> VariantJob {
    Arc::new(job)
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn variant_job(job: impl Fn(&[(&'static str, f64)]) -> Made + 'static) -> VariantJob {
    std::rc::Rc::new(job)
}

/// What [`Renderer::ensure_pipelines`] makes for a slot.
pub(super) enum Work {
    Once(Job),
    Variants(VariantJob),
}

/// A material shader's module, or the built-in one parsed on first use.
pub(super) enum Shader {
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

/// A built-in lit or prepass pipeline in its variants (pipeline_features.rs):
/// started for `features` on the compile workers natively, the full one
/// made at once on the web.
pub(crate) fn variants(
    declared: Features,
    features: Features,
    job: VariantJob,
) -> Variants<Deferred<wgpu::RenderPipeline>> {
    let v = Variants::new(declared, move |f: Features| {
        let job = job.clone();
        Deferred::anywhere(move || match job(&f.constants(declared)) {
            Made::Pipeline(p) => p,
            Made::Shadows(_) => unreachable!("a lit slot's job makes a pipeline"),
        })
    });
    v.start(features);
    #[cfg(target_arch = "wasm32")]
    v.get(features);
    v
}

/// A batch pipeline made by `make`: started on the compile workers
/// natively, made at once on the web (inside the host's frame span, which
/// asks for it asynchronously).
pub(super) fn deferred<T: crate::deferred::AnywhereOnce + 'static>(
    make: impl FnOnce() -> T + crate::deferred::AnywhereOnce + 'static,
) -> Deferred<T> {
    let d = Deferred::anywhere(make);
    #[cfg(not(target_arch = "wasm32"))]
    crate::deferred::Start::start(&d);
    #[cfg(target_arch = "wasm32")]
    d.get();
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A slot's job is made once, and only for a slot neither held nor
    /// queued: a frame whose pipelines are all made makes none.
    #[test]
    fn a_pipeline_job_is_made_only_for_a_wanted_slot() {
        let mut jobs: Vec<(Slot, Work)> = Vec::new();
        let mut made = 0;
        let mut job = || {
            made += 1;
            Work::Once(Box::new(|| unreachable!("a test job is never run")))
        };
        let front = Slot::Shadow((Side::Front.into(), false, false));
        let back = Slot::Shadow((Side::Back.into(), false, false));
        add_job!(jobs, front, true, job());
        add_job!(jobs, back, true, job());
        assert_eq!(jobs.len(), 0, "held slots queue nothing");
        add_job!(jobs, front, false, job());
        add_job!(jobs, front, false, job());
        add_job!(
            jobs,
            Slot::Prepass((Side::Front.into(), false)),
            false,
            job()
        );
        add_job!(jobs, back, true, job());
        assert_eq!(jobs.len(), 2, "one job per wanted slot");
        assert!(jobs[0].0 == front && jobs[1].0 == Slot::Prepass((Side::Front.into(), false)));
        assert_eq!(made, 2, "a job is made only for a slot it queues");
    }
}
