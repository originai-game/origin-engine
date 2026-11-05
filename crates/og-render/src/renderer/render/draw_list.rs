//! Draw list stage.

use super::*;

pub(super) struct DrawListInputs<'a, 'p, F0> {
    pub(super) pass: &'a mut wgpu::RenderPass<'p>,
    pub(super) list: &'a DrawList,
    pub(super) lit: bool,
    pub(super) current: &'a mut Option<LitDrawKey>,
    pub(super) physical_fx: Option<(&'a wgpu::BindGroup, &'a wgpu::BindGroup)>,
    pub(super) probes: Option<probe_select::ProbeGroups<'a>>,
    pub(super) fx: Option<&'a wgpu::BindGroup>,
    pub(super) batches: &'a [Batch],
    pub(super) culled_gpu: Option<&'a cull_gpu::GpuCulling>,
    pub(super) fx_lean_of: &'a F0,
    /// The main camera's order-independent pass (oit.rs) this list's
    /// blended batches draw into, with their OIT pipelines.
    pub(super) oit: Option<oit::Oit>,
}

impl Renderer {
    pub(super) fn frame_draw_list<'r>(
        &self,
        DrawListInputs {
            pass,
            list,
            lit,
            current,
            physical_fx,
            probes,
            fx,
            batches,
            culled_gpu,
            fx_lean_of,
            oit,
        }: DrawListInputs<'_, '_, impl Fn(&wgpu::BindGroup) -> Option<&'r wgpu::BindGroup>>,
    ) {
        let mut shadow_key: Option<(ShadowKey, Option<usize>, bool, bool)> = None;
        let mut physical_bound = false;
        // The pass's effect group without the planar reflection, bound
        // for its standard lightmapped batches (`fx_lean_group`).
        let lean = fx.and_then(fx_lean_of);
        let mut lean_bound = false;
        // The batch's reflection probe's frame group (probe_select.rs).
        let mut binder = probe_select::Binder::new(probes);
        // The lightmap pages bound at group 3 (lightmap.rs: one
        // group per baked scene).
        let mut pages_bound: Option<usize> = None;
        for &(b, first, count) in &list.ranges {
            let batch = &batches[b as usize];
            // Drawn with its lightmap's pipeline (lit passes only).
            let mut lightmapped = false;
            binder.bind(pass, batch.probe);
            if lit {
                let physical = batch.physical && physical_fx.is_some();
                let key = (
                    batch.side,
                    batch.blend,
                    batch.effects,
                    batch.shader,
                    physical,
                    batch.lean,
                    batch.derivatives,
                    batch.compact,
                    batch.array,
                );
                let oit = oit.filter(|_| batch.blend);
                let lightmap = Self::lightmap_family(&self.lightmaps, batch, physical)
                    .and_then(|f| match oit {
                        Some(m) => self
                            .oit_pipeline(oit::Source::Lightmap(f, batch.side, batch.effects), m),
                        None => self
                            .lightmaps
                            .pipeline(f, (batch.side, batch.blend, batch.effects)),
                    })
                    .zip(self.lightmaps.group(batch.mesh))
                    // Where the planar reflection takes a slot, a pass
                    // without the lean group draws it unbaked.
                    .filter(|_| physical || self.fx_lean_layout.is_none() || lean.is_some());
                lightmapped = lightmap.is_some();
                if let Some((p, (pages, group))) = lightmap {
                    // Lightmapped (lightmap.rs): its pipeline, its
                    // scene's pages at group 3, its uvs below.
                    pass.set_pipeline(p);
                    if pages_bound != Some(pages) {
                        pass.set_bind_group(3, group, &[]);
                        pages_bound = Some(pages);
                    }
                    *current = None;
                } else if *current != Some(key) {
                    pass.set_pipeline(match oit {
                        Some(m) => self
                            .oit_pipeline(Self::oit_lit_source(batch, false, physical), m)
                            .expect("made by ensure_oit_pipelines"),
                        None => self.lit_pipeline_of(batch, false, physical),
                    });
                    *current = Some(key);
                }
                match (lightmapped && !physical, lean, fx) {
                    (true, Some(l), _) if !lean_bound => {
                        pass.set_bind_group(2, l, &[]);
                        lean_bound = true;
                        physical_bound = false;
                    }
                    (false, _, Some(f)) if lean_bound => {
                        pass.set_bind_group(2, f, &[]);
                        lean_bound = false;
                    }
                    _ => {}
                }
                if let Some((fx, pfx)) = physical_fx
                    && physical != physical_bound
                {
                    pass.set_bind_group(2, if physical { pfx } else { fx }, &[]);
                    physical_bound = physical;
                    lean_bound = false;
                }
                let m = &self.cache.materials[batch.material];
                pass.set_bind_group(
                    1,
                    match (&m.physical, &m.array) {
                        (Some(g), _) if physical => g,
                        (_, Some(g)) => g,
                        _ => &m.bind_group,
                    },
                    &[],
                );
            } else {
                let key = (
                    (batch.side, false, batch.dissolve),
                    batch.shader,
                    batch.compact,
                    batch.array,
                );
                if shadow_key != Some(key) {
                    pass.set_pipeline(match batch.shader {
                        Some(sh) => &self.cache.hook_shadow_pipelines[&(sh, batch.side, false)],
                        None if batch.array => {
                            &self.array_maps.as_ref().expect("an array batch").shadow
                                [&(batch.side, batch.dissolve)]
                        }
                        None if batch.compact => &self.compact_shadow_pipelines[&batch.side],
                        None => &self.shadow_pipelines[&key.0],
                    });
                    shadow_key = Some(key);
                }
                if batch.dissolve || batch.shader.is_some() {
                    pass.set_bind_group(1, &self.cache.materials[batch.material].bind_group, &[]);
                }
            }
            let mesh = &self.cache.meshes[batch.mesh];
            pass.set_vertex_buffer(0, mesh.vertex_buffer(batch.compact).slice(..));
            // The batch's instances as a buffer slice from its first.
            let start = first as u64 * INSTANCE_STRIDE;
            let instances = match (list.indirect, culled_gpu) {
                (Some(_), Some(gc)) => &gc.instances,
                _ => &self.instance_buffer,
            };
            pass.set_vertex_buffer(
                1,
                instances.slice(start..start + count as u64 * INSTANCE_STRIDE),
            );
            if lightmapped && let Some(uvs) = self.lightmaps.uvs(batch.mesh) {
                pass.set_vertex_buffer(2, uvs.slice(..));
            }
            pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
            #[cfg(not(feature = "publish"))]
            let direct = crate::renderer::settings::DEBUG_CULL_DIRECT
                .load(std::sync::atomic::Ordering::Relaxed);
            #[cfg(feature = "publish")]
            let direct = false;
            match (list.indirect, culled_gpu) {
                (Some(_), Some(_)) if direct => pass.draw_indexed(0..mesh.index_count, 0, 0..count),
                (Some(g), Some(gc)) => {
                    let (args, at) = gc.draw_args(g, b);
                    pass.draw_indexed_indirect(args, at)
                }
                _ => pass.draw_indexed(0..mesh.index_count, 0, 0..count),
            }
        }
        if let Some((fx, _)) = physical_fx
            && physical_bound
        {
            pass.set_bind_group(2, fx, &[]);
        }
        if let (true, Some(f)) = (lean_bound, fx) {
            pass.set_bind_group(2, f, &[]);
        }
        binder.reset(pass);
    }
}
