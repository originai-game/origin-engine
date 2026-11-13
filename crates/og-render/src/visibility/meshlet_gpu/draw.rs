//! Drawing the surviving clusters: lit, resolved from the visibility buffer and into shadows.

use super::*;

impl MeshletGpu {
    /// Draws the frame's clustered batches of lit view `view` (0 the
    /// camera) into `pass` (the camera's lit pass, or its prepass with
    /// `prepass`; a reflection's, a view's or a probe face's lit pass):
    /// groups 0 and 2 are bound by the caller; `material(b)` is batch `b`'s
    /// material group; `probe(pass, b)` binds its reflection probe.
    /// Returns the draws made.
    pub fn draw<'m>(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        batches: &[Batch],
        view: u32,
        prepass: bool,
        material: impl Fn(&Batch) -> &'m wgpu::BindGroup,
        mut probe: impl FnMut(&mut wgpu::RenderPass<'_>, &Batch),
    ) -> u32 {
        let mut n = 0;
        for d in self.frame.iter().filter(|d| d.view == view && d.lit) {
            let b = &batches[d.batch as usize];
            let pipeline = if prepass {
                self.prepass.get(&b.side)
            } else {
                self.lit.get(&(b.side, b.effects))
            };
            let (Some(pipeline), Some((_, group))) = (pipeline, self.mesh_groups.get(&b.mesh))
            else {
                continue;
            };
            probe(pass, b);
            pass.set_pipeline(pipeline);
            pass.set_bind_group(1, material(b), &[]);
            pass.set_bind_group(3, group, &[d.first_item * 8]);
            pass.draw_indirect(&self.draws, d.row as u64 * 16);
            n += 1;
        }
        n
    }

    /// What the device said when it refused the visibility buffer's resolve
    /// (`vis.rs`): the visibility buffer is off on it from then on.
    pub fn visibility_buffer_refused(&self) -> Option<&str> {
        self.vis_buffer.refused()
    }

    /// Draws the camera's soft batches' resolves into `pass` (the camera's
    /// lit pass, or its prepass with `prepass`): one full-screen triangle
    /// each, shading the visibility buffer's pixels of its batch; groups 0
    /// and 2 bound by the caller. Returns the draws made.
    pub fn draw_resolve<'m>(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        batches: &[Batch],
        prepass: bool,
        material: impl Fn(&Batch) -> &'m wgpu::BindGroup,
        mut probe: impl FnMut(&mut wgpu::RenderPass<'_>, &Batch),
    ) -> u32 {
        // A pipeline the device has not answered for may be refused: its
        // draw would invalidate the frame (vis.rs).
        let Some(group) = self
            .vis_buffer
            .resolve_group
            .as_ref()
            .filter(|_| self.vis_buffer.ready())
        else {
            return 0;
        };
        let mut n = 0;
        for d in self.frame.iter().filter(|d| d.soft) {
            let b = &batches[d.batch as usize];
            let pipeline = if prepass {
                self.vis_buffer.prepass.as_ref()
            } else {
                self.vis_buffer.lit.get(&b.effects)
            };
            let Some(pipeline) = pipeline else {
                continue;
            };
            probe(pass, b);
            pass.set_pipeline(pipeline);
            pass.set_bind_group(1, material(b), &[]);
            pass.set_bind_group(3, group, &[]);
            pass.draw(0..3, d.row..d.row + 1);
            n += 1;
        }
        n
    }

    /// Draws the clustered casters of shadow view `view` into a shadow
    /// pass whose slot (group 0) the caller bound; `material(b)` is batch
    /// `b`'s material group (read by alpha-tested and dissolving casters).
    /// Returns the draws made.
    pub fn draw_shadow<'m>(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        target: crate::ShadowTarget,
        batches: &[Batch],
        view: u32,
        material: impl Fn(&Batch) -> &'m wgpu::BindGroup,
    ) -> u32 {
        let mut n = 0;
        for d in self.frame.iter().filter(|d| d.view == view) {
            let b = &batches[d.batch as usize];
            let (Some(pipeline), Some((_, group))) = (
                self.shadow.get(&(b.side, b.dissolve)),
                self.mesh_groups.get(&b.mesh),
            ) else {
                continue;
            };
            pass.set_pipeline(pipeline.get(target));
            pass.set_bind_group(1, material(b), &[]);
            pass.set_bind_group(3, group, &[d.first_item * 8]);
            pass.draw_indirect(&self.draws, d.row as u64 * 16);
            n += 1;
        }
        n
    }
}

pub(super) fn entry(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}
