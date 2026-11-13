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
                    side: m.side,
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
