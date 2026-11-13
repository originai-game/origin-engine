//! A frame's terrain stamps.

use super::*;

impl TerrainRenderer {
    /// Hands each drawn terrain its stamps this frame: their meshes
    /// uploaded (again when the scene's mesh changed), their uniforms in
    /// slots of one buffer, culled with their terrain's passes by their
    /// world bounds. A stamp whose terrain is not drawn, or whose mesh is
    /// gone, is not drawn.
    pub(super) fn prepare_stamps(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &crate::RenderScene,
        draws: &mut [TerrainDraw],
    ) {
        let mut slots: Vec<StampGpu> = Vec::new();
        let mut placed: Vec<(usize, StampDraw)> = Vec::new();
        for st in &scene.world.terrain_stamps {
            let Some(k) = draws.iter().position(|d| d.terrain == st.terrain) else {
                continue;
            };
            let Some(data) = scene.mesh(st.mesh) else {
                continue;
            };
            let index = st.mesh.index();
            let revision = scene.mesh_revision(index);
            let fresh = self
                .stamp_meshes
                .get(&index)
                .is_some_and(|m| m.scene == scene.id() && m.revision == revision);
            if !fresh {
                if data.indices.is_empty() || data.vertices.is_empty() {
                    continue;
                }
                let vertices = crate::buffer_init(
                    device,
                    queue,
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("terrain stamp vertices"),
                        contents: bytemuck::cast_slice(&data.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    },
                );
                let indices = crate::buffer_init(
                    device,
                    queue,
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("terrain stamp indices"),
                        contents: bytemuck::cast_slice(&data.indices),
                        usage: wgpu::BufferUsages::INDEX,
                    },
                );
                let bounds = data.bounds().unwrap_or((Vec3::ZERO, Vec3::ZERO));
                self.stamp_meshes.insert(
                    index,
                    StampMesh {
                        scene: scene.id(),
                        revision,
                        vertices,
                        indices,
                        count: data.indices.len() as u32,
                        bounds,
                    },
                );
            }
            let m = &self.stamp_meshes[&index];
            let terrain = &scene.world.terrains[st.terrain];
            let (center, radius) = world_sphere(st.transform, m.bounds.0, m.bounds.1);
            placed.push((
                k,
                StampDraw {
                    offset: (slots.len() as u64 * STAMP_STRIDE) as u32,
                    vertices: m.vertices.clone(),
                    indices: m.indices.clone(),
                    count: m.count,
                    center,
                    radius,
                    cast_shadows: st.cast_shadows && terrain.cast_shadows,
                    mirrored: crate::mirrors(&st.transform),
                },
            ));
            slots.push(StampGpu::new(st.transform, terrain.transform, st.blend));
        }
        if slots.is_empty() {
            return;
        }
        let need = slots.len() as u64;
        if self.stamp_slots.as_ref().is_none_or(|(n, ..)| *n < need) {
            let n = need.next_power_of_two().max(8);
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("terrain stamps"),
                size: n * STAMP_STRIDE,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("terrain stamps"),
                layout: &self.stamp_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<StampGpu>() as u64),
                    }),
                }],
            });
            self.stamp_slots = Some((n, buffer, group));
        }
        let (_, buffer, group) = self.stamp_slots.as_ref().expect("made above");
        let mut bytes = vec![0u8; (need * STAMP_STRIDE) as usize];
        for (i, g) in slots.iter().enumerate() {
            let at = i * STAMP_STRIDE as usize;
            bytes[at..at + std::mem::size_of::<StampGpu>()].copy_from_slice(bytemuck::bytes_of(g));
        }
        queue.write_buffer(buffer, 0, &bytes);
        for (k, d) in placed {
            draws[k].stamps.push(d);
            draws[k].stamp_group = Some(group.clone());
        }
    }
}
