//! Draw skinned stage.

use super::*;

pub(super) struct DrawSkinnedInputs<'a, 'p> {
    pub(super) pass: &'a mut wgpu::RenderPass<'p>,
    pub(super) list: &'a [Batch],
    pub(super) transparent: bool,
    pub(super) material_group: u32,
    pub(super) palette_at: u32,
    pub(super) pipelines: &'a [&'a wgpu::RenderPipeline],
    pub(super) physical_fx: Option<(&'a wgpu::BindGroup, &'a wgpu::BindGroup)>,
    pub(super) probes: Option<probe_select::ProbeGroups<'a>>,
    pub(super) palette_group: Option<&'a wgpu::BindGroup>,
    /// GPU culling's pass whose skinned draws these are: `list` is then
    /// every skinned batch, in order, and each draws the instances the
    /// pass kept (`cull_gpu::SkinnedCull`). `None`: `list`'s ranges of the
    /// skinned instance buffer.
    pub(super) indirect: Option<(&'a cull_gpu::GpuCulling, u32)>,
}

impl Renderer {
    pub(super) fn frame_draw_skinned(
        &self,
        DrawSkinnedInputs {
            pass,
            list,
            transparent,
            material_group,
            palette_at,
            pipelines,
            physical_fx,
            probes,
            palette_group,
            indirect,
        }: DrawSkinnedInputs<'_, '_>,
    ) -> u32 {
        let Some(palette) = palette_group else {
            return 0;
        };
        let mut draws = 0;
        let mut physical_bound = false;
        // Each batch's reflection probe (probe_select.rs), in its pose's
        // bounds as the instance batches.
        let mut binder = probe_select::Binder::new(probes);
        // The shadow layouts put the palette at 2: shadows draw an
        // array-map material with its standard group, and only casters.
        let shadow = palette_at == 2;
        for (k, (batch, pipeline)) in list.iter().zip(pipelines).enumerate() {
            if batch.transparent != transparent || (shadow && !batch.cast) {
                continue;
            }
            let physical = batch.physical && physical_fx.is_some();
            binder.bind(pass, batch.probe);
            pass.set_pipeline(pipeline);
            if let Some((fx, pfx)) = physical_fx
                && physical != physical_bound
            {
                pass.set_bind_group(2, if physical { pfx } else { fx }, &[]);
                physical_bound = physical;
            }
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
            pass.set_bind_group(palette_at, palette, &[]);
            let mesh = &self.cache.meshes[batch.mesh];
            let skin = self.cache.skins[batch.mesh]
                .as_ref()
                .expect("a skinned mesh");
            pass.set_vertex_buffer(0, mesh.vertices.slice(..));
            let (records, first) = match indirect {
                Some((gc, g)) => (&gc.skinned, gc.skinned_region(g, batch.first)),
                None => (&self.skinned_buffer, batch.first),
            };
            let start = first as u64 * skin::SKINNED_INSTANCE_STRIDE;
            pass.set_vertex_buffer(
                1,
                records.slice(start..start + batch.count as u64 * skin::SKINNED_INSTANCE_STRIDE),
            );
            pass.set_vertex_buffer(2, skin.slice(..));
            pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
            match indirect {
                Some((gc, g)) => {
                    pass.draw_indexed_indirect(&gc.args, gc.skinned_draw_offset(g, k as u32))
                }
                None => pass.draw_indexed(0..mesh.index_count, 0, 0..batch.count),
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
