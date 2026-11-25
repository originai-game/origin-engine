//! Draw skinned stage.

use super::*;

pub(super) struct DrawSkinnedInputs<'a, P> {
    pub(super) pass: &'a mut P,
    pub(super) list: &'a [Batch],
    pub(super) transparent: bool,
    /// Of the transparent batches, only the blended (`Some(true)`: the
    /// order-independent pass, oit.rs) or only the transmissive ones
    /// (`Some(false)`); `None`: both.
    pub(super) blended: Option<bool>,
    pub(super) material_group: u32,
    pub(super) palette_at: u32,
    pub(super) pipelines: &'a [Option<&'a wgpu::RenderPipeline>],
    pub(super) physical_fx: Option<(&'a wgpu::BindGroup, &'a wgpu::BindGroup)>,
    pub(super) probes: Option<probe_select::ProbeGroups<'a>>,
    pub(super) palette_group: Option<&'a wgpu::BindGroup>,
    /// GPU culling's pass whose skinned draws these are: `list` is then
    /// every skinned batch, in order, and each draws the instances the
    /// pass kept (`cull_gpu::SkinnedCull`). `None`: `list`'s ranges of the
    /// skinned instance buffer.
    pub(super) indirect: Option<(&'a cull_gpu::GpuCulling, u32)>,
    /// The crowd cull and this pass's row in it (`crowd_gpu.rs`): a crowd
    /// batch ([`Batch::crowd`]) draws that row's records of it; `None`:
    /// the pass draws no crowd batch.
    pub(super) crowds: Option<(&'a crowd_gpu::CrowdCulling, u32)>,
}

impl Renderer {
    pub(super) fn frame_draw_skinned(
        &self,
        DrawSkinnedInputs {
            pass,
            list,
            transparent,
            blended,
            material_group,
            palette_at,
            pipelines,
            physical_fx,
            probes,
            palette_group,
            indirect,
            crowds,
        }: DrawSkinnedInputs<'_, impl Encode>,
    ) -> u32 {
        let Some(palette) = palette_group else {
            return 0;
        };
        let mut draws = 0;
        let mut physical_bound = false;
        let mut palette_bound = false;
        let mut pipe_ptr: Option<*const wgpu::RenderPipeline> = None;
        let mut mat_bound: Option<(usize, bool)> = None;
        let mut mesh_bound: Option<usize> = None;
        // Each batch's reflection probe (probe_select.rs), in its pose's
        // bounds as the instance batches.
        let mut binder = probe_select::Binder::new(probes);
        // The shadow layouts put the palette at 2: shadows draw an
        // array-map material with its standard group, and only casters.
        let shadow = palette_at == 2;
        for (k, (batch, pipeline)) in list.iter().zip(pipelines).enumerate() {
            if batch.transparent != transparent
                || (shadow && !batch.cast)
                || blended.is_some_and(|b| b != batch.blend)
            {
                continue;
            }
            if batch.crowd.is_some() && crowds.is_none() {
                continue;
            }
            // Not ready yet (a display frame, gpu/deferred.rs).
            let Some(pipeline) = *pipeline else {
                continue;
            };
            let physical = batch.physical && physical_fx.is_some();
            binder.bind(pass, batch.probe);
            let p_ptr = pipeline as *const wgpu::RenderPipeline;
            if pipe_ptr != Some(p_ptr) {
                pass.set_pipeline(pipeline);
                pipe_ptr = Some(p_ptr);
            }
            if let Some((fx, pfx)) = physical_fx
                && physical != physical_bound
            {
                pass.set_bind_group(2, if physical { pfx } else { fx }, &[]);
                physical_bound = physical;
            }
            let mat_key = (batch.material, physical);
            if mat_bound != Some(mat_key) {
                let m = &self.cache.materials[batch.material];
                pass.set_bind_group(
                    material_group,
                    match (&m.physical, &m.array) {
                        (Some(g), _) if physical => g,
                        (_, Some(g)) if !shadow => g,
                        _ => &m.bind_group,
                    },
                    &[],
                );
                mat_bound = Some(mat_key);
            }
            if !palette_bound {
                pass.set_bind_group(palette_at, palette, &[]);
                palette_bound = true;
            }
            let mesh = &self.cache.meshes[batch.mesh];
            if mesh_bound != Some(batch.mesh) {
                let skin = self.cache.skins[batch.mesh]
                    .as_ref()
                    .expect("a skinned mesh");
                pass.set_vertex_buffer(0, mesh.vertex_slice(false));
                pass.set_vertex_buffer(2, skin.slice(..));
                pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
                mesh_bound = Some(batch.mesh);
            }
            match (batch.crowd, crowds) {
                // A crowd's draw: the row's records from its start, the
                // level's members from the first instance the cull wrote.
                (Some(d), Some((cc, row))) => {
                    let start = cc.row_offset(row);
                    pass.set_vertex_buffer(1, cc.records.slice(start..cc.row_offset(row + 1)));
                    let (args, at) = cc.draw_args(row, d);
                    pass.draw_indexed_indirect(args, at);
                }
                _ => {
                    let (records, first) = match indirect {
                        Some((gc, g)) => (&gc.skinned, gc.skinned_region(g, batch.first)),
                        None => (&self.instances.skinned_buffer, batch.first),
                    };
                    let start = first as u64 * skin::SKINNED_INSTANCE_STRIDE;
                    let end = start + batch.count as u64 * skin::SKINNED_INSTANCE_STRIDE;
                    pass.set_vertex_buffer(1, records.slice(start..end));
                    match indirect {
                        Some((gc, g)) => {
                            let (args, at) = gc.skinned_draw_args(g, k as u32);
                            pass.draw_indexed_indirect(args, at)
                        }
                        None => pass.draw_indexed(0..mesh.index_count, 0, 0..batch.count),
                    }
                }
            }
            draws += 1;
        }
        if let Some((fx, _)) = physical_fx
            && physical_bound
        {
            pass.set_bind_group(2, fx, &[]);
        }
        binder.reset(pass);
        draws
    }
}
