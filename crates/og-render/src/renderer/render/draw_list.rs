//! Draw list stage.

use super::*;

pub(super) struct DrawListInputs<'a, P, F0> {
    pub(super) pass: &'a mut P,
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
        }: DrawListInputs<
            '_,
            impl Encode,
            impl Fn(&wgpu::BindGroup) -> Option<&'r wgpu::BindGroup>,
        >,
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
        // Deduplicate redundant per-batch state binds across sorted batches.
        let mut lightmap_pipe_ptr: Option<*const wgpu::RenderPipeline> = None;
        let mut bindless_pipe_key: Option<(Faces, bool)> = None;
        let mut material_bound: Option<(usize, bool, bool)> = None;
        let mut vb0_bound: Option<(usize, bool, bool)> = None;
        let mut ib_bound: Option<usize> = None;
        let mut vb2_bound: Option<usize> = None;
        #[cfg(not(feature = "publish"))]
        let direct =
            crate::renderer::settings::DEBUG_CULL_DIRECT.load(std::sync::atomic::Ordering::Relaxed);
        #[cfg(feature = "publish")]
        let direct = false;
        let pass_wide = match (list.indirect, culled_gpu) {
            (Some(g), Some(gc)) if !direct && gc.uses_first_instance() => {
                pass.set_vertex_buffer(1, gc.pass_instances_slice(g));
                true
            }
            _ => false,
        };
        let mdi = pass_wide && self.info.caps.contains(Caps::MULTI_DRAW_INDIRECT_COUNT);
        let bindless = self.lit.bindless.as_ref().filter(|_| lit && pass_wide);
        let can_bindless = |batch: &Batch, physical: bool, lightmapped: bool| -> bool {
            let Some(bt) = bindless else {
                return false;
            };
            oit.is_none()
                && !lightmapped
                && !physical
                && !batch.blend
                && !batch.transmissive
                && !batch.array
                && !batch.compact
                && batch.shader.is_none()
                && !batch.dissolve
                && !batch.reveal
                && self.cache.materials[batch.material].bindless_ok
                && bt.bind_group().is_some()
                && bt.pipeline(batch.faces, batch.effects).is_some()
        };
        let lightmap_of = |batch: &Batch, physical: bool, batch_oit: Option<oit::Oit>| {
            Self::lightmap_family(&self.lights.lightmaps, batch, physical)
                .and_then(|f| match batch_oit {
                    Some(m) => {
                        self.oit_pipeline(oit::Source::Lightmap(f, batch.faces, batch.effects), m)
                    }
                    None => self
                        .lights
                        .lightmaps
                        .pipeline(f, (batch.faces, batch.blend, batch.effects)),
                })
                .zip(self.lights.lightmaps.group(batch.mesh))
                // Where the planar reflection takes a slot, a pass
                // without the lean group draws it unbaked.
                .filter(|_| physical || self.fx_lean_layout.is_none() || lean.is_some())
        };
        let mut idx = 0;
        while idx < list.ranges.len() {
            let (b, first, count) = list.ranges[idx];
            let batch = &batches[b as usize];
            // Drawn with its lightmap's pipeline (lit passes only).
            let mut lightmapped = false;
            let mut pos_only = false;
            let mut used_bindless = false;
            let mut lit_key_cur: Option<LitDrawKey> = None;
            let mut shadow_key_cur: Option<(ShadowKey, Option<usize>, bool, bool)> = None;
            let mut shadow_needs_mat = false;
            binder.bind(pass, batch.probe);
            if lit {
                let physical = batch.physical && physical_fx.is_some();
                let key = (
                    batch.faces,
                    batch.blend,
                    batch.effects,
                    batch.shader,
                    physical,
                    batch.lean,
                    batch.dissolve,
                    batch.compact,
                    batch.array,
                );
                let batch_oit = oit.filter(|_| batch.blend);
                let lightmap = lightmap_of(batch, physical, batch_oit);
                lightmapped = lightmap.is_some();
                if let Some((p, (pages, group))) = lightmap {
                    // Lightmapped (lightmap.rs): its pipeline, its
                    // scene's pages at group 3, its uvs below.
                    let ptr = p as *const wgpu::RenderPipeline;
                    if lightmap_pipe_ptr != Some(ptr) {
                        pass.set_pipeline(p);
                        lightmap_pipe_ptr = Some(ptr);
                    }
                    if pages_bound != Some(pages) {
                        pass.set_bind_group(3, group, &[]);
                        pages_bound = Some(pages);
                    }
                    *current = None;
                    bindless_pipe_key = None;
                } else if can_bindless(batch, physical, lightmapped) {
                    let bt = bindless.expect("checked by can_bindless");
                    let bkey = (batch.faces, batch.effects);
                    if bindless_pipe_key != Some(bkey) {
                        pass.set_pipeline(
                            bt.pipeline(batch.faces, batch.effects).expect("checked"),
                        );
                        bindless_pipe_key = Some(bkey);
                    }
                    *current = None;
                    lightmap_pipe_ptr = None;
                    used_bindless = true;
                } else {
                    if *current != Some(key) {
                        let Some(pipeline) = (match batch_oit {
                            Some(m) => {
                                self.oit_pipeline(Self::oit_lit_source(batch, false, physical), m)
                            }
                            None => self.lit_pipeline_of(batch, false, physical),
                        }) else {
                            idx += 1;
                            continue;
                        };
                        pass.set_pipeline(pipeline);
                        *current = Some(key);
                        lightmap_pipe_ptr = None;
                        bindless_pipe_key = None;
                    }
                    lit_key_cur = Some(key);
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
                if used_bindless {
                    let bt = bindless.expect("checked by can_bindless");
                    if material_bound != Some((usize::MAX, false, false)) {
                        pass.set_bind_group(1, bt.bind_group().expect("checked"), &[]);
                        material_bound = Some((usize::MAX, false, false));
                    }
                } else {
                    let mat_key = (batch.material, physical, batch.array);
                    if material_bound != Some(mat_key) {
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
                        material_bound = Some(mat_key);
                    }
                }
            } else {
                let surface_shadow =
                    batch.simulated && self.cache.meshes[batch.mesh].surface.is_some();
                let shadow_dissolve = batch.dissolve || surface_shadow;
                pos_only = batch.shader.is_none() && !batch.array && !shadow_dissolve;
                let key = (
                    (batch.faces, false, shadow_dissolve),
                    batch.shader,
                    batch.compact,
                    batch.array,
                );
                if shadow_key != Some(key) {
                    let Some(set) = match batch.shader {
                        Some(sh) => &self.cache.hook_shadow_pipelines[&(sh, batch.faces, false)],
                        None if batch.array => {
                            &self.lit.array_maps.as_ref().expect("an array batch").shadow
                                [&(batch.faces, batch.dissolve)]
                        }
                        None if batch.compact => {
                            &self.shadows.compact_shadow_pipelines[&batch.faces]
                        }
                        None => &self.shadows.shadow_pipelines[&key.0],
                    }
                    .get() else {
                        idx += 1;
                        continue;
                    };
                    pass.set_pipeline(set);
                    shadow_key = Some(key);
                }
                shadow_key_cur = Some(key);
                shadow_needs_mat = shadow_dissolve || batch.shader.is_some();
                if shadow_needs_mat {
                    let mat_key = (batch.material, false, batch.array);
                    if material_bound != Some(mat_key) {
                        let m = &self.cache.materials[batch.material];
                        let bg = if batch.array {
                            m.array.as_ref().unwrap_or(&m.bind_group)
                        } else {
                            &m.bind_group
                        };
                        pass.set_bind_group(1, bg, &[]);
                        material_bound = Some(mat_key);
                    }
                }
            }
            let mesh = &self.cache.meshes[batch.mesh];
            let vb0_key = (batch.mesh, batch.compact, pos_only);
            if vb0_bound != Some(vb0_key) {
                pass.set_vertex_buffer(
                    0,
                    if pos_only {
                        mesh.position_slice(batch.compact)
                    } else {
                        mesh.vertex_slice(batch.compact)
                    },
                );
                vb0_bound = Some(vb0_key);
            }
            // Transmission identifies an exit by the absolute instance
            // slot, shared with the backface pass (not a transform hash).
            let volume = lit && batch.transmissive && list.indirect.is_none();
            if !pass_wide {
                let start = first as u64 * INSTANCE_STRIDE;
                let instances = match (list.indirect, culled_gpu) {
                    (Some(_), Some(gc)) => &gc.instances,
                    _ => &self.instances.instance_buffer,
                };
                pass.set_vertex_buffer(
                    1,
                    if volume {
                        instances.slice(..)
                    } else {
                        instances.slice(start..start + count as u64 * INSTANCE_STRIDE)
                    },
                );
            }
            if lightmapped
                && vb2_bound != Some(batch.mesh)
                && let Some(uvs) = self.lights.lightmaps.uvs(batch.mesh)
            {
                pass.set_vertex_buffer(2, uvs.slice(..));
                vb2_bound = Some(batch.mesh);
            }
            if ib_bound != Some(batch.mesh) {
                pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
                ib_bound = Some(batch.mesh);
            }
            // Coalesce contiguous batches sharing pipeline, mesh, probe, and
            // bindings (or any materials under bindless / pos-only shadow) into
            // one multi_draw_indexed_indirect submission.
            let mut run_len = 1u32;
            if mdi && !lightmapped {
                let mut prev_b = b;
                while idx + (run_len as usize) < list.ranges.len() {
                    let (nb, _, _) = list.ranges[idx + run_len as usize];
                    if nb != prev_b + 1 {
                        break;
                    }
                    let next = &batches[nb as usize];
                    if next.mesh != batch.mesh
                        || next.compact != batch.compact
                        || (probes.is_some() && next.probe != batch.probe)
                    {
                        break;
                    }
                    let compatible = if lit {
                        let next_physical = next.physical && physical_fx.is_some();
                        let next_oit = oit.filter(|_| next.blend);
                        let next_lm = lightmap_of(next, next_physical, next_oit).is_some();
                        if next_lm {
                            false
                        } else if used_bindless {
                            next.faces == batch.faces
                                && next.effects == batch.effects
                                && can_bindless(next, next_physical, false)
                        } else {
                            let next_key = (
                                next.faces,
                                next.blend,
                                next.effects,
                                next.shader,
                                next_physical,
                                next.lean,
                                next.dissolve,
                                next.compact,
                                next.array,
                            );
                            !can_bindless(next, next_physical, false)
                                && lit_key_cur == Some(next_key)
                                && next.material == batch.material
                        }
                    } else {
                        let next_surface =
                            next.simulated && self.cache.meshes[next.mesh].surface.is_some();
                        let next_dissolve = next.dissolve || next_surface;
                        let next_pos_only = next.shader.is_none() && !next.array && !next_dissolve;
                        let next_key = (
                            (next.faces, false, next_dissolve),
                            next.shader,
                            next.compact,
                            next.array,
                        );
                        next_pos_only == pos_only
                            && shadow_key_cur == Some(next_key)
                            && (!shadow_needs_mat || next.material == batch.material)
                    };
                    if !compatible {
                        break;
                    }
                    prev_b = nb;
                    run_len += 1;
                }
            }
            match (list.indirect, culled_gpu) {
                (Some(_), Some(_)) if direct => pass.draw_indexed(0..mesh.index_count, 0, 0..count),
                (Some(g), Some(gc)) if run_len > 1 => {
                    let (args, at) = gc.draw_args(g, b);
                    pass.multi_draw_indexed_indirect(args, at, run_len)
                }
                (Some(g), Some(gc)) => {
                    let (args, at) = gc.draw_args(g, b);
                    pass.draw_indexed_indirect(args, at)
                }
                _ => pass.draw_indexed(
                    0..mesh.index_count,
                    0,
                    if volume {
                        first..first + count
                    } else {
                        0..count
                    },
                ),
            }
            idx += run_len as usize;
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
