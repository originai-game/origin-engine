//! The frame group's entries: the one list every frame group is made from
//! (frame.wgsl, `Renderer::frame_layout`).

/// What a frame group binds of its own, beside what every frame group
/// shares ([`Renderer::frame_entries`]).
pub(crate) struct FrameGroupParts<'a> {
    /// Its camera's frame uniform.
    pub uniform: wgpu::BindingResource<'a>,
    /// Its camera's light clusters: the table and the references.
    pub clusters: [&'a wgpu::Buffer; 2],
    /// Its irradiance and prefiltered cubes.
    pub cubes: [&'a wgpu::TextureView; 2],
    /// The screen-space reflections it reads (ssr.rs: the frame's or none).
    pub ssr: &'a wgpu::TextureView,
}

impl crate::Renderer {
    /// A frame group's entries in binding order (frame.wgsl, `frame_layout`):
    /// the one list every frame group is made from.
    pub(crate) fn frame_entries<'a>(
        &'a self,
        parts: FrameGroupParts<'a>,
    ) -> Vec<wgpu::BindingResource<'a>> {
        let ibl = self.lights.ibl.views();
        vec![
            parts.uniform,
            wgpu::BindingResource::TextureView(&self.shadows.cascade_map.array_view),
            wgpu::BindingResource::TextureView(&self.shadows.local_map.array_view),
            wgpu::BindingResource::TextureView(parts.cubes[0]),
            wgpu::BindingResource::TextureView(parts.cubes[1]),
            wgpu::BindingResource::TextureView(&ibl.brdf_lut),
            wgpu::BindingResource::Sampler(&ibl.sampler),
            self.lights.light_buffers[0].as_entire_binding(),
            parts.clusters[0].as_entire_binding(),
            parts.clusters[1].as_entire_binding(),
            self.lights.lightmaps.probe_buffer.as_entire_binding(),
            self.lights.ltc.as_entire_binding(),
            self.lights.caustic_grid.as_entire_binding(),
            wgpu::BindingResource::TextureView(parts.ssr),
        ]
    }
}
