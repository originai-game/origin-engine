//! The frame's bind groups (the bind phase): the environment's frame and
//! sky groups, the probes' groups (per active probe, per due face, and with
//! OIT's arena), the effect groups, the physical pipelines' group, and the
//! particles' and metaballs' groups. Built from the renderer's state after
//! every upload; nothing here writes the queue. The planar reflections',
//! the views' and the due probe faces' own groups are their passes' bind
//! steps.

use super::*;

pub(super) struct BindFrameInputs<'a> {
    pub scene: &'a RenderScene,
    pub batches: &'a [Batch],
    pub oit_mode: Option<oit::Oit>,
    pub probe_frame: &'a probes::ProbeFrame,
    pub skinned_visible: &'a [Batch],
    pub ao_target: bool,
    pub contact: bool,
    pub transmission_view: Option<(&'a wgpu::TextureView, &'a mips::MipChain)>,
    pub particle_frame: &'a particles_gpu::ParticleFrame,
}

pub(super) struct BindFrameOutputs<'a> {
    pub frame_bind_group: &'a wgpu::BindGroup,
    pub sky_group: &'a wgpu::BindGroup,
    pub probe_groups: Vec<wgpu::BindGroup>,
    pub oit_probe_groups: Vec<wgpu::BindGroup>,
    pub face_groups: Vec<wgpu::BindGroup>,
    pub oit_frame_group: Option<wgpu::BindGroup>,
    pub fx_group: &'a wgpu::BindGroup,
    pub sprite_fx_group: &'a wgpu::BindGroup,
    pub glow_fx_group: &'a wgpu::BindGroup,
    pub physical_fx_group: Option<wgpu::BindGroup>,
    pub scene_depth: &'a wgpu::TextureView,
    pub particle_group: Option<wgpu::BindGroup>,
    pub metaball_groups: Vec<wgpu::BindGroup>,
    pub fx_lean: Option<&'a wgpu::BindGroup>,
}

impl Renderer {
    /// Builds the frame's bind groups.
    #[allow(clippy::too_many_lines)]
    pub(super) fn bind_frame<'a>(
        &'a self,
        BindFrameInputs {
            scene,
            batches,
            oit_mode,
            probe_frame,
            skinned_visible,
            ao_target,
            contact,
            transmission_view,
            particle_frame,
        }: BindFrameInputs<'a>,
    ) -> BindFrameOutputs<'a> {
        let (_, frame_bind_group, sky_group, _) = self
            .lights
            .environment
            .as_ref()
            .expect("environment generated above");
        // The probes' frame groups (`probes.rs`): per active probe, the
        // environment's prefiltered cube and the probe's in place of the
        // irradiance and prefiltered cubes; per due capture face, the
        // frame's own cubes under the face camera's uniform.
        // With the linked lists (oit.rs) the OIT pass binds the frame's and
        // its probes' groups again with the arena at binding 14.
        let oit_arena = self
            .oit
            .frame_list_layout
            .as_ref()
            .zip(self.oit.arena.as_ref())
            .filter(|_| oit_mode.is_some_and(oit::Oit::arena));
        let (probe_groups, face_groups, oit_frame_group, oit_probe_groups) = {
            let ibl = self.lights.ibl.views();
            // The reflections' read (frame.wgsl ssr_tex), whose rows the
            // uniforms switch: on in the main camera's and its probes'.
            let ssr_view = self.ssr.frame_view();
            let group_in = |layout: &wgpu::BindGroupLayout,
                            arena: Option<&wgpu::Buffer>,
                            label: &str,
                            uniform: wgpu::BindingResource,
                            a: &wgpu::TextureView,
                            b: &wgpu::TextureView| {
                let mut entries = vec![
                    uniform,
                    wgpu::BindingResource::TextureView(&self.shadows.cascade_map.array_view),
                    wgpu::BindingResource::TextureView(&self.shadows.local_map.array_view),
                    wgpu::BindingResource::TextureView(a),
                    wgpu::BindingResource::TextureView(b),
                    wgpu::BindingResource::TextureView(&ibl.brdf_lut),
                    wgpu::BindingResource::Sampler(&ibl.sampler),
                    self.lights.light_buffers[0].as_entire_binding(),
                    self.lights.light_buffers[1].as_entire_binding(),
                    self.lights.light_buffers[2].as_entire_binding(),
                    self.lights.lightmaps.probe_buffer.as_entire_binding(),
                    self.lights.ltc.as_entire_binding(),
                    self.lights.caustic_grid.as_entire_binding(),
                    wgpu::BindingResource::TextureView(ssr_view),
                ];
                entries.extend(arena.map(|a| a.as_entire_binding()));
                bind_group(&self.device, label, layout, &entries)
            };
            let oit_frame_group = oit_arena.map(|(layout, arena)| {
                group_in(
                    layout,
                    Some(&arena.buffer),
                    "frame (oit lists)",
                    self.lights.frame_buffer.as_entire_binding(),
                    &ibl.irradiance,
                    &ibl.prefiltered,
                )
            });
            let mut oit_probe_groups: Vec<wgpu::BindGroup> = Vec::new();
            let group = |k: usize, a: &wgpu::TextureView, b: &wgpu::TextureView| {
                bind_group(
                    &self.device,
                    "frame (reflection probe)",
                    &self.frame_layout,
                    &[
                        self.probes.binding(k),
                        wgpu::BindingResource::TextureView(&self.shadows.cascade_map.array_view),
                        wgpu::BindingResource::TextureView(&self.shadows.local_map.array_view),
                        wgpu::BindingResource::TextureView(a),
                        wgpu::BindingResource::TextureView(b),
                        wgpu::BindingResource::TextureView(&ibl.brdf_lut),
                        wgpu::BindingResource::Sampler(&ibl.sampler),
                        self.lights.light_buffers[0].as_entire_binding(),
                        self.lights.light_buffers[1].as_entire_binding(),
                        self.lights.light_buffers[2].as_entire_binding(),
                        self.lights.lightmaps.probe_buffer.as_entire_binding(),
                        self.lights.ltc.as_entire_binding(),
                        self.lights.caustic_grid.as_entire_binding(),
                        wgpu::BindingResource::TextureView(ssr_view),
                    ],
                )
            };
            let probe_groups: Vec<wgpu::BindGroup> = probe_frame.active[..probe_frame.sampled]
                .iter()
                .enumerate()
                .map(|(k, (p, slot))| {
                    let view = match (slot, p.baked) {
                        (Some(s), _) => &self.probes.slots[*s].view,
                        (None, Some(h)) => {
                            &self.cache.environment_maps[h.index()]
                                .as_ref()
                                .expect("a loaded map (Probes::prepare)")
                                .view
                        }
                        (None, None) => unreachable!("a probe without a slot is baked"),
                    };
                    if let Some((layout, arena)) = oit_arena {
                        oit_probe_groups.push(group_in(
                            layout,
                            Some(&arena.buffer),
                            "frame (reflection probe, oit lists)",
                            self.probes.binding(k),
                            &ibl.prefiltered,
                            view,
                        ));
                    }
                    group(k, &ibl.prefiltered, view)
                })
                .collect();
            let first = probe_frame.active.len();
            let face_groups: Vec<wgpu::BindGroup> = (0..6 * probe_frame.due.len())
                .map(|j| group(first + j, &ibl.irradiance, &ibl.prefiltered))
                .collect();
            (probe_groups, face_groups, oit_frame_group, oit_probe_groups)
        };
        let groups = self.lights.fx_groups.as_ref().expect("made above");
        let (fx_group, sprite_fx_group, glow_fx_group) = (&groups.fx, &groups.sprite, &groups.glow);
        // The physical pipelines' effect group: the same plus the scene copy
        // (a placeholder in a frame without transmission).
        let physical_fx_group = self.lit.physical.as_ref().and_then(|p| {
            batches
                .iter()
                .chain(skinned_visible)
                .any(|b| b.physical)
                .then(|| {
                    let atlas_view = scene.effects.decal_atlas.map_or(&self.default_white, |t| {
                        &self.cache.textures[t.index()].view
                    });
                    let ao_view = if ao_target || contact {
                        self.graph.view("ao")
                    } else {
                        &self.post.white
                    };
                    fx::fx_group(
                        &self.device,
                        "physical fx",
                        &p.fx_layout,
                        &[
                            self.lights.fx_buffer.as_entire_binding(),
                            wgpu::BindingResource::TextureView(ao_view),
                            wgpu::BindingResource::TextureView(atlas_view),
                            wgpu::BindingResource::Sampler(&self.trilinear_sampler),
                            wgpu::BindingResource::TextureView(
                                transmission_view.map_or(&self.post.white, |(v, _)| v),
                            ),
                        ],
                        &self.lights.decal_buffers,
                    )
                })
        });
        let scene_depth = self
            .graph
            .try_view("lindepth")
            .unwrap_or(&self.post.far_depth);
        let particle_group = (!particle_frame.systems.is_empty()).then(|| {
            let copy = self
                .graph
                .try_view("scene-copy")
                .unwrap_or(&self.post.white);
            self.particles
                .gpu
                .draw_group(&self.device, scene_depth, copy)
        });
        let metaball_groups: Vec<wgpu::BindGroup> = post::FIELD_NAMES
            .iter()
            .map_while(|name| self.graph.try_view(name))
            .map(|v| self.world.metaballs.group(&self.device, v))
            .collect();
        let fx_lean = groups.fx_lean.as_ref();
        BindFrameOutputs {
            frame_bind_group,
            sky_group,
            probe_groups,
            oit_probe_groups,
            face_groups,
            oit_frame_group,
            fx_group,
            sprite_fx_group,
            glow_fx_group,
            physical_fx_group,
            scene_depth,
            particle_group,
            metaball_groups,
            fx_lean,
        }
    }
}
