//! The crowds' members as the frame's batches.

use super::*;

impl Renderer {
    /// The scene's crowds as the crowd cull's tables (crowd.rs `build`),
    /// their previous transforms with them when `motion`.
    pub(crate) fn build_crowds(
        &self,
        scene: &RenderScene,
        motion: bool,
    ) -> Result<crate::crowd::CrowdFrame, RenderError> {
        let cache = &self.cache;
        crate::crowd::build(
            &scene.objects.crowds,
            &scene.objects.baked_clips,
            &|m| {
                Some(crate::crowd::MeshInfo {
                    skinned: cache.skins.get(m)?.is_some(),
                    index_count: cache.meshes.get(m)?.index_count,
                    joints: cache.joint_boxes.get(m).map_or(&[][..], Vec::as_slice),
                })
            },
            cache.materials.len(),
            &|m| cache.materials[m].shader.is_some(),
            &|c| self.instances.palette.baked_offset(c),
            motion,
            scene.objects.lod_bias,
        )
    }

    /// One skinned batch per crowd draw (a part at a level): no instances
    /// of its own; on the GPU path it draws the crowd cull's records
    /// ([`Batch::crowd`]), on the CPU path [`Self::append_crowds`] gives it
    /// runs of records.
    pub(crate) fn crowd_batches(
        &self,
        scene: &RenderScene,
        frame: &crate::crowd::CrowdFrame,
    ) -> Vec<Batch> {
        frame
            .draws
            .iter()
            .enumerate()
            .map(|(k, d)| {
                let m = &self.cache.materials[d.material];
                let c = &scene.objects.crowds[d.crowd as usize];
                Batch {
                    mesh: d.mesh,
                    material: d.material,
                    first: 0,
                    count: 0,
                    faces: m.side.into(),
                    transparent: m.transparent,
                    blend: m.blend,
                    physical: m.physical.is_some(),
                    transmissive: m.transmissive,
                    dissolve: m.dissolve,
                    effects: m.effects,
                    shader: m.shader,
                    lean: false,
                    derivatives: m.flat,
                    compact: false,
                    array: m.array.is_some(),
                    foliage: false,
                    simulated: false,
                    cast: c.cast_shadows,
                    layers: c.layers,
                    probe: crate::probe_select::NONE,
                    crowd: Some(k as u32),
                }
            })
            .collect()
    }

    /// The crowds drawn as skinned instances (the CPU path, crowd.rs):
    /// each member at the camera's level, a record per part, appended to
    /// `skinned` as batches of `crowd_batches`' kinds with their boxes.
    /// `previous_at`: the palette slot of the first member's previous
    /// transform (TAA). Answers the card-level members' records, by card
    /// (`(card, first, count)` runs over them).
    #[allow(clippy::type_complexity)]
    pub(crate) fn append_crowds(
        &self,
        scene: &RenderScene,
        frame: &crate::crowd::CrowdFrame,
        crowd_batches: &[Batch],
        skinned: &mut SkinnedFrame,
        previous_at: Option<u32>,
    ) -> (Vec<(u32, u32, u32)>, Vec<crate::skin::SkinnedInstanceGpu>) {
        let crate::crowd::CpuCrowds {
            runs,
            data,
            boxes,
            card_runs,
            cards,
        } = crate::crowd::cpu_records(
            &scene.objects.crowds,
            frame,
            &scene.frame.camera,
            self.height as f32,
            scene.objects.lod_bias,
            previous_at,
        );
        let base = skinned.data.len() as u32;
        for (draw, first, count) in runs {
            skinned.batches.push(Batch {
                first: base + first,
                count,
                crowd: None,
                ..crowd_batches[draw as usize]
            });
        }
        skinned.data.extend(data);
        skinned.boxes.extend(boxes);
        skinned.baked_moved |= frame.moved;
        (card_runs, cards)
    }
}

/// A frame's crowds, planned ([`Renderer::plan_crowds`]).
pub(crate) struct CrowdPlan {
    /// The crowd cull's tables.
    pub frame: crate::crowd::CrowdFrame,
    /// One skinned batch per crowd draw.
    pub batches: Vec<Batch>,
    /// The palette slot of the first member's previous transform, where
    /// the frame keeps motion.
    pub previous: Option<u32>,
    /// Whether the GPU-driven path draws them (crowd_gpu.rs).
    pub gpu: bool,
    /// The CPU path's card records (`crowd_impostor.rs` draws them whole).
    pub cpu_cards: (Vec<(u32, u32, u32)>, Vec<crate::skin::SkinnedInstanceGpu>),
}

impl Renderer {
    /// The frame's crowds: their tables and batches, where their previous
    /// transforms go, and which path draws them. On the CPU path their
    /// members are appended to `skinned` here. `no_probe_due`: no probe
    /// face is drawn this frame.
    pub(crate) fn plan_crowds(
        &mut self,
        scene: &RenderScene,
        temporal: bool,
        no_probe_due: bool,
        skinned: &mut SkinnedFrame,
    ) -> Result<CrowdPlan, RenderError> {
        // Crowds (crowd.rs): their members as the crowd cull's tables, and
        // where each member's previous transform goes under TAA (after the
        // palette's previous section, skin.rs).
        let crowd_motion = temporal || scene.look.post.motion_blur.is_some();
        let crowd_frame = self.build_crowds(scene, crowd_motion)?;
        let crowd_batches = self.crowd_batches(scene, &crowd_frame);
        let crowd_previous = crowd_motion.then(|| {
            (skinned.matrices.len()
                + skin::previous_section_len(
                    &scene.objects.joint_palettes,
                    &scene.objects.previous_joint_palettes,
                    skinned.matrices.len(),
                    skinned.previous_slots.len(),
                )) as u32
        });
        if crowd_previous.is_some_and(|at| at as usize + crowd_frame.previous.len() > MAX_JOINTS) {
            return Err(RenderError::InvalidData(format!(
                "{} crowd members' previous transforms past the palette's other slots (at most {MAX_JOINTS} in all)",
                crowd_frame.previous.len()
            )));
        }
        // On the GPU-driven path (crowd_gpu.rs) where the device draws
        // indirect with a first instance and every view that draws them is
        // a GPU culling pass (secondary views and probe faces cull skinned
        // instances on the CPU); else as skinned instances.
        let crowd_gpu = crowd_frame.len() > 0
            && self.world.crowd_indirect
            && self.visibility.desc_culling != Culling::Cpu
            && scene.frame.views.is_empty()
            && no_probe_due
            && crowd_batches.iter().all(|b| !b.transparent)
            && {
                let limits = self.device.limits();
                let max = limits
                    .max_storage_buffer_binding_size
                    .min(limits.max_buffer_size);
                let rows = 4
                    + MAX_CASCADES as usize
                    + scene
                        .lighting
                        .lights
                        .iter()
                        .filter(|l| l.cast_shadows)
                        .count()
                        * 6;
                // GPU culling must take the frame too (its pyramid, its
                // passes): checked for every instance the frame may sort.
                let culls = self
                    .visibility
                    .gpu_cull
                    .get_or_insert_with(|| {
                        cull_gpu::GpuCulling::new(&self.device, max, self.info.indirect_per_draw)
                    })
                    .fits(
                        scene.objects.instances.len() as u32,
                        scene.objects.skinned_instances.len() as u32,
                        rows,
                    );
                culls
                    && self
                        .world
                        .crowd_gpu
                        .get_or_insert_with(|| {
                            crowd_gpu::CrowdCulling::new(
                                &self.device,
                                max,
                                self.info.indirect_per_draw,
                            )
                        })
                        .fits(crowd_frame.len(), rows)
            };
        // The CPU path's card records (`crowd_impostor.rs` draws them whole).
        let cpu_cards = if !crowd_gpu && crowd_frame.len() > 0 {
            self.append_crowds(scene, &crowd_frame, &crowd_batches, skinned, crowd_previous)
        } else {
            Default::default()
        };
        Ok(CrowdPlan {
            frame: crowd_frame,
            batches: crowd_batches,
            previous: crowd_previous,
            gpu: crowd_gpu,
            cpu_cards,
        })
    }
}
