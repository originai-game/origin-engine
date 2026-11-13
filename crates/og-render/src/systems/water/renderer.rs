use super::*;

impl WaterRenderer {
    /// Starts compiling the pipelines `waters` draw with on worker threads
    /// (deferred.rs): the grid's or the mesh's, as each surface asks.
    /// `features`: the frame's (pipeline_features.rs).
    pub(crate) fn start(&self, waters: &[Water], features: Features) {
        if waters.iter().any(|w| w.mesh.is_some()) {
            self.mesh_pipeline.start(features);
        }
        if waters.iter().any(|w| w.mesh.is_none()) {
            self.pipeline.start(features);
        }
    }

    /// Makes its pipeline if no frame used it yet (Renderer::warm).
    pub(crate) fn warm(&self) -> bool {
        crate::deferred::warm_one(&[
            self.pipeline.full(),
            self.mesh_pipeline.full(),
            &self.spray_pipeline,
        ])
    }

    /// The surfaces as the faces of a reflection probe's capture of face
    /// size `size` draw them (after [`WaterRenderer::prepare`]): through
    /// `below`, the copy of the face drawn so far, and over `scene_depth`,
    /// the face's depth prepass, so the face refracts what lies below,
    /// foams at the edges and reflects the face in screen
    /// space as the frame does; without the planar reflection.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn face_draws<'a>(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        waters: &[Water],
        size: u32,
        below: &wgpu::TextureView,
        scene_depth: &wgpu::TextureView,
        white: &wgpu::TextureView,
        textures: &dyn Fn(crate::scene::TextureHandle) -> Option<&'a wgpu::TextureView>,
    ) -> Vec<WaterDraw> {
        let n = size as f32;
        waters
            .iter()
            .zip(&self.slots)
            .map(|(water, slot)| {
                let sea = slot.face.fft[0] > 0.0;
                let heights = match (&slot.ripples, &slot.heights, height_field(water), &self.fft) {
                    (_, _, _, Some((fft, _))) if sea => &fft.view,
                    (Some(r), _, Some(_), _) => &r.view,
                    (None, Some((_, _, v)), Some(_), _) => v,
                    _ => &self.flat,
                };
                let mut face = slot.face;
                face.screen = [n, n, 1.0 / n, 1.0 / n];
                let uniform = crate::buffer_init(
                    device,
                    queue,
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("water (probe face)"),
                        contents: bytemuck::bytes_of(&face),
                        usage: wgpu::BufferUsages::UNIFORM,
                    },
                );
                let tex = wgpu::BindingResource::TextureView;
                // No planar reflection nor screen AO in a face: its own
                // copy and white in their slots.
                let entries = [
                    uniform.as_entire_binding(),
                    tex(heights),
                    tex(below),
                    tex(scene_depth),
                    wgpu::BindingResource::Sampler(&self.sampler),
                    tex(below),
                    tex(water.normal_map.and_then(textures).unwrap_or(white)),
                    wgpu::BindingResource::Sampler(&self.ripple_sampler),
                    tex(white),
                    tex(water.flow_map.and_then(textures).unwrap_or(white)),
                    tex(shore_view(&self.shores, &self.no_shore, water)),
                    tex(match &self.wakes {
                        Some(w) if face.wake[3] > 0.5 => &w.view,
                        _ => &self.no_wake,
                    }),
                ];
                let entries: Vec<wgpu::BindGroupEntry> = entries
                    .into_iter()
                    .enumerate()
                    .map(|(i, resource)| wgpu::BindGroupEntry {
                        binding: i as u32,
                        resource,
                    })
                    .collect();
                let half = Vec3::new(water.size.x, 0.0, water.size.y) * 0.5;
                WaterDraw {
                    group: device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("water (probe face)"),
                        layout: &self.layout,
                        entries: &entries,
                    }),
                    // A surface over a mesh is left out of the faces (the
                    // faces' draws carry no mesh buffers).
                    vertices: if water.mesh.is_some() {
                        0
                    } else if sea {
                        sea_vertices(slot.face.clip[1] as u32)
                    } else {
                        grid_segments(water) * grid_segments(water) * 6
                    },
                    mesh: None,
                    // The spray is the camera's alone (its droplets are about the view), as the waterline is.
                    spray: 0,
                    waterline: false,
                    bounds: (
                        water.transform.transform_point3(Vec3::ZERO),
                        water.transform.transform_vector3(half).length(),
                    ),
                }
            })
            .collect()
    }

    /// Draws the surfaces' view depth for the screen AO (`fs_ao_depth`; group 0: the frame). Returns the draw calls.
    pub(crate) fn draw_ao_depth(&self, pass: &mut wgpu::RenderPass, draws: &[WaterDraw]) -> u32 {
        let [grid, mesh] = self.ao_depth_pipelines.get();
        for d in draws {
            pass.set_bind_group(1, &d.group, &[]);
            match &d.mesh {
                Some((vertices, indices, count)) => {
                    pass.set_pipeline(mesh);
                    pass.set_vertex_buffer(0, vertices.slice(..));
                    pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..*count, 0, 0..1);
                }
                None => {
                    pass.set_pipeline(grid);
                    pass.draw(0..d.vertices, 0..1);
                }
            }
        }
        draws.len() as u32
    }

    /// Draws the surfaces (group 0: the frame, or each surface's reflection
    /// probe's: probe_select.rs). Returns the draw calls.
    pub(crate) fn draw(
        &self,
        pass: &mut wgpu::RenderPass,
        draws: &[WaterDraw],
        probes: &mut crate::probe_select::Binder,
        // The frame's features (pipeline_features.rs).
        features: Features,
    ) -> u32 {
        for d in draws {
            probes.at(pass, d.bounds.0, d.bounds.1);
            pass.set_bind_group(1, &d.group, &[]);
            match &d.mesh {
                Some((vertices, indices, count)) => {
                    pass.set_pipeline(self.mesh_pipeline.get(features));
                    pass.set_vertex_buffer(0, vertices.slice(..));
                    pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..*count, 0, 0..1);
                }
                None => {
                    pass.set_pipeline(self.pipeline.get(features));
                    pass.draw(0..d.vertices, 0..1);
                }
            }
        }
        probes.reset(pass);
        // The spray over every surface (it lies in front of the water it leaves).
        let mut calls = draws.len() as u32;
        for d in draws.iter().filter(|d| d.spray > 0) {
            pass.set_pipeline(&self.spray_pipeline);
            pass.set_bind_group(1, &d.group, &[]);
            pass.draw(0..6, 0..d.spray);
            calls += 1;
        }
        // The waterline over all of it.
        for d in draws.iter().filter(|d| d.waterline) {
            pass.set_pipeline(&self.waterline_pipeline);
            pass.set_bind_group(1, &d.group, &[]);
            pass.draw(0..3, 0..1);
            calls += 1;
        }
        calls
    }
}

/// The water module after the frame group, the light tables, the lighting,
/// the light volumes' hook, the area lights and the shared shading (the
/// surface gathers the light around it and takes the local lights as every
/// lit surface; its reflection, glitter and foam are its own). It reads no
/// screen occlusion through the shared shading: its own binding holds it.
pub(crate) fn water_source(frame_wgsl: &str, lighting_wgsl: &str) -> String {
    crate::assemble(
        &[
            frame_wgsl,
            &crate::clusters::wgsl(),
            lighting_wgsl,
            &og_render_data::lightmap::probe_wgsl(),
            crate::AREA_WGSL,
            crate::SURFACE_SHADE_WGSL,
            &water_parts(),
        ],
        &[crate::Capability::ScreenAo],
    )
}

/// The water's own parts.
fn water_parts() -> String {
    [
        crate::ocean_fft::SHORE_WGSL,
        wgsl3d!("water.wgsl"),
        wgsl3d!("water_sea.wgsl"),
        wgsl3d!("water_detail.wgsl"),
        wgsl3d!("water_below.wgsl"),
        wgsl3d!("water_spray.wgsl"),
        wgsl3d!("water_waterline.wgsl"),
    ]
    .concat()
}
