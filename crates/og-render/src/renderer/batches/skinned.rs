//! The skinned instances' batches, palettes and culling.

use super::*;
use crate::instances::mirrors;

impl Renderer {
    /// The skinned instances as (mesh, material) batches over the skinned
    /// instance buffer, their instance data, and the frame's joint matrices.
    #[allow(clippy::type_complexity)]
    pub(crate) fn build_skinned(
        &mut self,
        scene: &RenderScene,
        probes: &[(Vec3, Vec3)],
    ) -> Result<SkinnedFrame, RenderError> {
        let mut offsets = Vec::with_capacity(scene.objects.joint_palettes.len());
        let mut matrices = Vec::new();
        let mut garments = Vec::new();
        for p in &scene.objects.joint_palettes {
            offsets.push((matrices.len() as u32, p.matrices.len() as u32));
            matrices.extend(p.matrices.iter().map(|m| m.to_cols_array_2d()));
        }
        if matrices.len() > MAX_JOINTS {
            return Err(RenderError::InvalidData(format!(
                "{} joints in the frame's palettes (at most {MAX_JOINTS})",
                matrices.len()
            )));
        }
        for (i, s) in scene.objects.skinned_instances.iter().enumerate() {
            let mesh = s.mesh.index();
            if mesh >= self.cache.meshes.len() || self.cache.skins[mesh].is_none() {
                return Err(RenderError::InvalidHandle(format!(
                    "skinned instance {i}: mesh {mesh} has neither a skin nor morph targets"
                )));
            }
            if s.material.index() >= self.cache.materials.len() {
                return Err(RenderError::InvalidHandle(format!(
                    "skinned instance {i}: material {}",
                    s.material.index()
                )));
            }
            match &s.baked {
                Some(b) => {
                    for c in [b.clip, b.previous_clip] {
                        let ok = scene.objects.baked_clips.get(c as usize).is_some_and(|c| {
                            c.joints > 0
                                && c.frames.len() % c.joints as usize == 0
                                && c.frame_count() >= 2
                        });
                        if !ok {
                            return Err(RenderError::InvalidData(format!(
                                "skinned instance {i}: baked clip {c} of {} (a clip holds whole frames, at least two)",
                                scene.objects.baked_clips.len()
                            )));
                        }
                    }
                    if s.deformed.is_some() {
                        return Err(RenderError::InvalidData(format!(
                            "skinned instance {i}: a baked clip and deformed vertices"
                        )));
                    }
                }
                None if s.palette as usize >= offsets.len() => {
                    return Err(RenderError::InvalidHandle(format!(
                        "skinned instance {i}: palette {} of {}",
                        s.palette,
                        offsets.len()
                    )));
                }
                None => {}
            }
        }
        // The clips the skinned instances and the crowds play, now or the
        // frame before (a crowd's members mostly share one: marked, not
        // inserted one by one).
        let mut marked = vec![false; scene.objects.baked_clips.len()];
        let mut mark = |b: &skin::BakedPose| {
            for c in [b.clip, b.previous_clip] {
                if let Some(m) = marked.get_mut(c as usize) {
                    *m = true;
                }
            }
        };
        scene
            .objects
            .skinned_instances
            .iter()
            .filter_map(|s| s.baked.as_ref())
            .for_each(&mut mark);
        scene
            .objects
            .crowds
            .iter()
            .flat_map(|c| &c.members)
            .for_each(|m| mark(&m.pose));
        // And the clips crowds' impostors are baked from.
        for i in scene
            .objects
            .crowds
            .iter()
            .filter_map(|c| c.impostor.as_ref())
        {
            if let Some(m) = marked.get_mut(i.clip as usize) {
                *m = true;
            }
        }
        let used: std::collections::BTreeSet<u32> = marked
            .iter()
            .enumerate()
            .filter(|(_, m)| **m)
            .map(|(i, _)| i as u32)
            .collect();
        let baked_joints: usize = used
            .iter()
            .map(|&i| scene.objects.baked_clips[i as usize].frames.len())
            .sum();
        if baked_joints + matrices.len() > MAX_JOINTS {
            return Err(RenderError::InvalidData(format!(
                "{} joints in the frame's palettes and bakes (at most {MAX_JOINTS})",
                baked_joints + matrices.len()
            )));
        }
        self.instances.palette.prepare_bakes(
            &self.device,
            &self.queue,
            &scene.objects.baked_clips,
            &used,
        );
        let mut order: Vec<usize> = (0..scene.objects.skinned_instances.len()).collect();
        // Each instance's reflection probe: the one holding the centre of
        // its posed bounds, its joints' boxes moved by its palette
        // (skin.rs; a morphed mesh without a skin: its bind pose's), so a
        // figure walked out of a room by its root joint takes the room it
        // is in (probe_select.rs).
        let probe_of: Vec<u8> = scene
            .objects
            .skinned_instances
            .iter()
            .map(|s| {
                let mesh = s.mesh.index();
                // A baked instance: its bind pose's centre.
                let posed = match s.baked {
                    Some(_) => None,
                    None => self.cache.joint_boxes.get(mesh).and_then(|b| {
                        let palette = &scene.objects.joint_palettes[s.palette as usize].matrices;
                        skin::posed_centre(b, palette, &s.transform)
                    }),
                };
                let centre = posed.unwrap_or_else(|| {
                    self.cache
                        .meshes
                        .get(mesh)
                        .and_then(|m| m.bounds)
                        .map_or(s.transform.w_axis.truncate(), |(c, _)| {
                            s.transform.transform_point3(c)
                        })
                });
                probe_select::pick(probes, centre)
            })
            .collect();
        // A (mesh, material)'s casters first, its non-casters a batch of
        // their own (left out of the shadow passes), then by probe.
        og_core::sort::sort_by_key(&mut order, |&i| {
            let s = &scene.objects.skinned_instances[i];
            (
                s.mesh,
                s.material,
                !s.cast_shadows,
                views::layer_of(&scene.objects.skinned_layers, i),
                probe_of[i],
                mirrors(&s.transform),
            )
        });
        let mut batches: Vec<Batch> = Vec::new();
        let mut data = Vec::with_capacity(order.len());
        let mut boxes = Vec::with_capacity(order.len());
        // Each (mesh, baked clip)'s box over every frame, once a frame.
        let mut clip_boxes: std::collections::HashMap<(usize, u32), Option<(Vec3, Vec3)>> =
            Default::default();
        let mut previous_slots = Vec::new();
        let mut baked_moved = false;
        let taa = self.info.antialias.temporal() || scene.look.post.motion_blur.is_some();
        for &i in &order {
            let s = &scene.objects.skinned_instances[i];
            let (mesh, material, cast) = (s.mesh.index(), s.material.index(), s.cast_shadows);
            let layers = views::layer_of(&scene.objects.skinned_layers, i);
            let probe = probe_of[i];
            let mirrored = mirrors(&s.transform);
            if self.cache.meshes[mesh].index_count == 0 {
                continue;
            }
            match batches.last_mut() {
                Some(b)
                    if b.mesh == mesh
                        && b.material == material
                        && b.cast == cast
                        && b.layers == layers
                        && b.probe == probe
                        && b.faces.mirrored == mirrored =>
                {
                    b.count += 1;
                    b.simulated |= s.deformed.is_some();
                }
                _ => {
                    let m = &self.cache.materials[material];
                    batches.push(Batch {
                        mesh,
                        material,
                        first: data.len() as u32,
                        count: 1,
                        faces: Faces {
                            side: m.side,
                            mirrored,
                        },
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
                        simulated: s.deformed.is_some(),
                        cast,
                        layers,
                        probe,
                        crowd: None,
                    });
                }
            }
            let (morph_first, targets) = self.cache.morph_ranges[mesh];
            let joints = self
                .cache
                .joint_boxes
                .get(mesh)
                .map_or(&[][..], Vec::as_slice);
            let clip = s.baked.and_then(|b| {
                *clip_boxes.entry((mesh, b.clip)).or_insert_with(|| {
                    skin::clip_box(joints, &scene.objects.baked_clips[b.clip as usize])
                })
            });
            let morphing = targets > 0
                && s.morph_weights[..targets as usize]
                    .iter()
                    .any(|&w| w != 0.0);
            boxes.push(skin::skinned_box(
                s,
                self.cache.meshes[mesh].bounds,
                joints,
                match s.baked {
                    Some(_) => &[],
                    None => &scene.objects.joint_palettes[s.palette as usize].matrices,
                },
                clip,
                morphing,
            ));
            if let Some(b) = s.baked {
                // Bakes stay in their own buffer; only the previous
                // transform belongs to the frame's dynamic palette.
                let clip = &scene.objects.baked_clips[b.clip as usize];
                let (k, t) = clip.at(b.frame);
                let now = self.instances.palette.baked_offset(b.clip) + k * clip.joints as usize;
                let before = &scene.objects.baked_clips[b.previous_clip as usize];
                let (pk, pt) = before.at(b.previous_frame);
                let then = self.instances.palette.baked_offset(b.previous_clip)
                    + pk * before.joints as usize;
                // Motion effects read it; otherwise a crowd costs its clips alone.
                let mut transform_at = 0;
                if taa {
                    transform_at = matrices.len() + 1;
                    matrices.push(b.previous_transform.to_cols_array_2d());
                    previous_slots.push(b.previous_transform.to_cols_array_2d());
                }
                baked_moved |= b.clip != b.previous_clip
                    || b.frame != b.previous_frame
                    || b.previous_transform != s.transform;
                data.push(skin::SkinnedInstanceGpu {
                    transform: instances::gpu_columns(&s.transform),
                    palette: [now as f32, clip.joints as f32, morph_first as f32, -1.0 - t],
                    morph0: [0.0; 4],
                    morph1: [transform_at as f32, then as f32, pt, 0.0],
                    look: {
                        let [flash, color, tint] = instances::fx_bits(&s.fx);
                        [
                            instances::layer_bits(0.0, s.receive_shadows),
                            flash,
                            color,
                            tint,
                        ]
                    },
                });
                continue;
            }
            let (first, count) = offsets[s.palette as usize];
            // Its weights' slots after the joints (`skin.rs`, the layout);
            // none while every weight is zero.
            let targets_n = targets as usize;
            let mut weights_at = 0.0;
            if targets_n > 0 && s.morph_weights[..targets_n].iter().any(|&w| w != 0.0) {
                weights_at = (matrices.len() + 1) as f32;
                matrices.extend(skin::weight_slots(&s.morph_weights, targets_n));
                let before = scene
                    .objects
                    .previous_morph_weights
                    .get(i)
                    .filter(|_| {
                        scene.objects.previous_morph_weights.len()
                            == scene.objects.skinned_instances.len()
                    })
                    .unwrap_or(&s.morph_weights);
                previous_slots.extend(skin::weight_slots(before, targets_n));
            }
            // Its own vertices (a garment), after its weights.
            let mut deformed_at = 0.0;
            if let Some(d) = s.deformed {
                let Some(now) = scene.objects.deformed_vertices.get(d as usize) else {
                    return Err(RenderError::InvalidData(format!(
                        "skinned instance {i}: deformed vertices {d} of {}",
                        scene.objects.deformed_vertices.len()
                    )));
                };
                let n = scene.mesh(s.mesh).map_or(0, |m| m.vertices.len());
                if now.vertex_count() != n || (now.garment.is_none() && now.normals.len() != n) {
                    return Err(RenderError::InvalidData(format!(
                        "skinned instance {i}: {} deformed positions and {} normals for a mesh of {n} vertices",
                        now.positions.len(),
                        now.normals.len()
                    )));
                }
                deformed_at = (matrices.len() + 1) as f32;
                let before = scene
                    .objects
                    .previous_deformed_vertices
                    .get(d as usize)
                    .filter(|b| {
                        scene.objects.previous_deformed_vertices.len()
                            == scene.objects.deformed_vertices.len()
                            && b.vertex_count() == n
                            && (b.garment.is_some() || b.normals.len() == n)
                    })
                    .unwrap_or(now);
                if self.world.garments.is_some() && now.garment.is_some() {
                    garments.push((d, matrices.len() as u32));
                    matrices.resize(matrices.len() + n.div_ceil(2), [[0.; 4]; 4]);
                    previous_slots.resize(previous_slots.len() + n.div_ceil(2), [[0.; 4]; 4]);
                } else {
                    matrices.extend(skin::deformed_slots(now));
                    previous_slots.extend(skin::deformed_slots(before));
                }
            }
            data.push(skin::SkinnedInstanceGpu {
                transform: instances::gpu_columns(&s.transform),
                palette: [
                    first as f32,
                    count as f32,
                    morph_first as f32,
                    targets as f32,
                ],
                morph0: [weights_at, deformed_at, 0.0, 0.0],
                morph1: [0.0; 4],
                look: {
                    let [flash, color, tint] = instances::fx_bits(&s.fx);
                    [
                        instances::layer_bits(0.0, s.receive_shadows),
                        flash,
                        color,
                        tint,
                    ]
                },
            });
        }
        if matrices.len() + baked_joints > MAX_JOINTS {
            return Err(RenderError::InvalidData(format!(
                "{} joints, blendshape weight and deformed vertex slots in the frame's palette (at most {MAX_JOINTS})",
                matrices.len() + baked_joints
            )));
        }
        Ok(SkinnedFrame {
            batches,
            garments,
            data,
            boxes,
            matrices,
            previous_slots,
            baked_moved,
        })
    }

    /// The skinned batches the camera sees: instances whose posed box
    /// touches `frustum`, appended to `frame.data` as new runs.
    /// Only the batches on a layer of `layers` (`views.rs`).
    pub(crate) fn cull_skinned(
        frame: &mut SkinnedFrame,
        frustum: &Frustum,
        layers: u32,
    ) -> (Vec<Batch>, u32) {
        let mut visible = Vec::new();
        let mut culled = 0;
        for b in frame.batches.iter().filter(|b| b.layers & layers != 0) {
            let first = frame.data.len() as u32;
            for k in b.first as usize..(b.first + b.count) as usize {
                let (lo, hi) = frame.boxes[k];
                if frustum.intersects_box(lo, hi) {
                    let d = frame.data[k];
                    frame.data.push(d);
                } else {
                    culled += 1;
                }
            }
            let count = frame.data.len() as u32 - first;
            if count > 0 {
                visible.push(Batch { first, count, ..*b });
            }
        }
        (visible, culled)
    }
}
