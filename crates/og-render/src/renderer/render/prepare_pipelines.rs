//! Prepare pipelines stage.

use super::*;

pub(super) struct PrepareFramePipelinesInputs<'a> {
    pub(super) scene: &'a RenderScene,
    pub(super) ao: Option<AmbientOcclusion>,
    pub(super) batches: &'a mut [Batch],
    pub(super) skinned_batches: &'a mut [Batch],
    pub(super) skinned_visible: &'a mut [Batch],
    pub(super) view_skinned: &'a mut [Vec<Batch>],
    pub(super) probe_skinned: &'a mut [Vec<Batch>],
    pub(super) particle_frame: &'a particles_gpu::ParticleFrame,
    pub(super) view_plans: &'a [views::ViewPlan],
    pub(super) probe_frame: &'a probes::ProbeFrame,
    pub(super) fluids_3d: bool,
    pub(super) scene_water: &'a [Water],
    pub(super) prepass: bool,
}

impl Renderer {
    pub(super) fn prepare_frame_pipelines(
        &mut self,
        PrepareFramePipelinesInputs {
            scene,
            ao,
            batches,
            skinned_batches,
            skinned_visible,
            view_skinned,
            probe_skinned,
            particle_frame,
            view_plans,
            probe_frame,
            fluids_3d,
            scene_water,
            prepass,
        }: PrepareFramePipelinesInputs<'_>,
    ) -> (bool, bool, bool) {
        // Decals and screen-space AO need the effect shader on every batch;
        // decals take screen derivatives (the lean pipelines keep the discard).
        if ao.is_some() || !scene.decals.is_empty() {
            let decals = !scene.decals.is_empty();
            for b in batches.iter_mut().chain(skinned_batches.iter_mut()) {
                b.effects = true;
                b.derivatives |= decals;
            }
            for b in skinned_visible
                .iter_mut()
                .chain(view_skinned.iter_mut().flatten())
                .chain(probe_skinned.iter_mut().flatten())
            {
                b.effects = true;
                b.derivatives |= decals;
            }
        }
        // A target's depth texture, and the depth a view's soft particles
        // fade against, are drawn by the view's own prepass (views.rs):
        // its pipelines are the main prepass's.
        let view_depth = particle_frame.soft
            || view_plans
                .iter()
                .any(|p| matches!(p.dest, views::Dest::Texture { depth: Some(_), .. }));
        // A due capture's faces draw their own depth prepass for the water
        // and 3D fluids over them, which also show the copy of the face
        // drawn so far through them, and for their soft particles to fade
        // against (probes.rs `FaceTargets`): only when a capture is due.
        let due = !probe_frame.due.is_empty();
        let face_fluid = due && fluids_3d;
        let face_copy = due && (!scene_water.is_empty() || fluids_3d);
        let face_depth = face_copy || (due && particle_frame.soft);
        if face_depth {
            self.probes
                .ensure_targets(&self.device, probe_frame, face_copy, face_fluid);
        }
        self.ensure_pipelines(
            batches,
            skinned_batches,
            prepass || view_depth || face_depth,
        );
        // A view may draw a skinned batch the main camera culled or hides.
        for v in &*view_skinned {
            self.ensure_pipelines(&[], v, view_depth);
        }
        for v in &*probe_skinned {
            self.ensure_pipelines(&[], v, face_depth);
        }
        // Lightmapped batches (lightmap.rs): their family's pipeline, and a
        // physical one's standard family too (passes without the physical
        // groups, views and probes, draw it standard).
        for b in &*batches {
            let physical = [b.physical, false];
            for physical in &physical[..if b.physical { 2 } else { 1 }] {
                let Some(family) = Self::lightmap_family(&self.lightmaps, b, *physical) else {
                    continue;
                };
                use lightmap::Family;
                // Without the planar reflection, where it takes a slot.
                let lightmap_fx = self.fx_lean_layout.as_ref().unwrap_or(&self.fx_layout);
                let (groups, module) = match family {
                    Family::Standard | Family::Compact => {
                        ([&self.material_layout, lightmap_fx], &self.pbr_shader)
                    }
                    Family::Shader(sh) => (
                        [&self.material_layout, lightmap_fx],
                        &self.cache.shaders[sh],
                    ),
                    Family::Array => {
                        let a = self.array_maps.as_ref().expect("an array batch");
                        ([&a.layout, lightmap_fx], &a.module)
                    }
                    Family::Physical | Family::PhysicalShader(_) => {
                        let p = self.physical.as_ref().expect("a physical batch");
                        let module = match family {
                            Family::PhysicalShader(sh) => &self.cache.physical_shaders[&sh],
                            _ => &p.module,
                        };
                        ([&p.material_layout, &p.fx_layout], module)
                    }
                };
                self.lightmaps.ensure_pipeline(
                    &self.device,
                    family,
                    groups,
                    module,
                    (b.side, b.blend, b.effects),
                    self.samples,
                );
            }
        }
        // A pattern in object space: the object pass after the prepass.
        let object_space = prepass && scene.stylise.in_object_space();
        if object_space {
            let object = self
                .stylise_object
                .get_or_insert_with(|| stylise_object::ObjectPass::new(&self.device));
            let opaque = batches.iter().filter(|b| !b.transparent);
            for (side, skinned) in opaque
                .map(|b| (b.side, false))
                .chain(skinned_visible.iter().map(|b| (b.side, true)))
            {
                object.ensure(
                    &self.device,
                    (&self.pbr_layout, &self.pbr_skinned_layout),
                    side,
                    skinned,
                );
            }
        }
        (face_fluid, face_depth, object_space)
    }
}
