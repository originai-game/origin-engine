//! The instance records a frame's draws read: on the CPU path the batches
//! drawn whole and the copies appended after the sorted instances (under
//! GPU culling the transparent draws), their previous transforms, and the
//! debris layer's step over them.

use super::*;

pub(super) struct InstanceUploadInputs<'a> {
    pub scene: &'a RenderScene,
    pub batches: &'a [Batch],
    pub transforms: &'a [instances::GpuTransform],
    pub origins: &'a [u32],
    pub debris_instances: &'a HashMap<u32, (usize, Mat4)>,
    pub sorted: u32,
    pub gpu_cull: bool,
    pub auxiliary_instances: bool,
    pub previous_valid: bool,
}

impl Renderer {
    /// Uploads the records, and answers the bytes written and the debris
    /// layer's command buffer, submitted before any that reads them.
    pub(super) fn upload_instance_records(
        &mut self,
        InstanceUploadInputs {
            scene,
            batches,
            transforms,
            origins,
            debris_instances,
            sorted,
            gpu_cull,
            auxiliary_instances,
            previous_valid,
        }: InstanceUploadInputs<'_>,
    ) -> (u64, Option<wgpu::CommandBuffer>) {
        let previous_sway = if self.instances.previous_sway.len() == scene.objects.instances.len() {
            self.instances.previous_sway.as_slice()
        } else {
            &[]
        };
        let cpu_instances = if gpu_cull {
            &transforms[sorted as usize..]
        } else {
            transforms
        };
        // The records a draw reads: under GPU culling every one here (the
        // lists' own copies); on the CPU path the batches a list draws
        // whole and the copies appended after the sorted instances.
        let mut drawn: Vec<std::ops::Range<usize>> = Vec::new();
        let mut draw = |r: std::ops::Range<usize>| match drawn.last_mut() {
            Some(last) if last.end == r.start => last.end = r.end,
            _ if r.is_empty() => {}
            _ => drawn.push(r),
        };
        if !gpu_cull {
            for (b, _) in batches
                .iter()
                .zip(&self.visibility.cull_inside.whole)
                .filter(|(_, w)| **w)
            {
                draw(b.first as usize..(b.first + b.count) as usize);
            }
            draw(sorted as usize..cpu_instances.len());
        } else {
            draw(0..cpu_instances.len());
        }
        let stride = INSTANCE_STRIDE as usize;
        let drawn_bytes: Vec<std::ops::Range<usize>> = drawn
            .iter()
            .map(|r| r.start * stride..r.end * stride)
            .collect();
        // Of those, only the blocks that changed since the last frame are
        // written (a buffer just replaced is written whole).
        let instance_bytes: &[u8] = bytemuck::cast_slice(cpu_instances);
        if grow(
            &self.device,
            &mut self.instances.instance_buffer,
            "instances",
            instance_bytes.len() as u64,
        ) {
            self.instances.instance_upload.invalidate();
        }
        let mut instance_upload_bytes = self.instances.instance_upload.write(
            &self.queue,
            &self.instances.instance_buffer,
            instance_bytes,
            &drawn_bytes,
        );
        if gpu_cull && let Some(gc) = &self.visibility.gpu_cull {
            instance_upload_bytes += gc.uploaded;
        }
        if auxiliary_instances {
            // As the instances' own: with a compact run's dequantisation,
            // for the records a draw reads.
            let (meshes, materials) = (&self.cache.meshes, &self.cache.materials);
            // Under GPU culling these records start after the sorted ones.
            let offset = if gpu_cull { sorted as usize } else { 0 };
            let mut prev = vec![instances::GpuTransform::default(); cpu_instances.len()];
            for k in drawn.iter().flat_map(|r| r.clone()) {
                let i = origins[offset + k] as usize;
                let inst = &scene.objects.instances[i];
                let p = if previous_valid {
                    scene.objects.previous_transforms[i]
                } else {
                    inst.transform
                };
                let d = Self::dequant(meshes, materials, inst.mesh.index(), inst.material.index());
                prev[k] = instances::gpu_transform(&d.map_or(p, |d| p * d));
                let sway = previous_sway.get(i).copied().unwrap_or(inst.sway);
                prev[k].layer = instances::layer_bits(sway.max(0.0), inst.receive_shadows);
                prev[k].fx[0] = inst.sway.max(0.0);
            }
            let bytes: &[u8] = bytemuck::cast_slice(&prev);
            if grow(
                &self.device,
                &mut self.instances.prev_instance_buffer,
                "previous instances",
                bytes.len() as u64,
            ) {
                self.instances.prev_instance_upload.invalidate();
            }
            instance_upload_bytes += self.instances.prev_instance_upload.write(
                &self.queue,
                &self.instances.prev_instance_buffer,
                bytes,
                &drawn_bytes,
            );
        } else {
            // Not written this frame: what it holds is not known next time.
            self.instances.prev_instance_upload.invalidate();
        }
        // The debris layer: its pieces stepped and their transforms
        // written into the records just uploaded (and last frame's beside
        // them), in a command buffer submitted before any that reads them
        // (debris.rs).
        let debris_commands = {
            let (meshes, materials) = (&self.cache.meshes, &self.cache.materials);
            let record = |k: usize| -> Option<(usize, glam::Mat4)> {
                let i = origins[k];
                let &(p, local) = debris_instances.get(&i)?;
                let inst = &scene.objects.instances[i as usize];
                let d = Self::dequant(meshes, materials, inst.mesh.index(), inst.material.index());
                Some((p, d.map_or(local, |d| local * d)))
            };
            let records =
                |range: std::ops::Range<usize>, base: usize| -> Vec<(u32, usize, glam::Mat4)> {
                    if debris_instances.is_empty() {
                        return Vec::new();
                    }
                    range
                        .filter_map(|k| record(k).map(|(p, l)| ((k - base) as u32, p, l)))
                        .collect()
                };
            let mut targets = Vec::new();
            if gpu_cull && let Some(gc) = &self.visibility.gpu_cull {
                targets.push(crate::debris::Target {
                    instances: &gc.instances,
                    previous: auxiliary_instances.then_some(&gc.previous),
                    records: records(0..sorted as usize, 0),
                });
            }
            let base = if gpu_cull { sorted as usize } else { 0 };
            targets.push(crate::debris::Target {
                instances: &self.instances.instance_buffer,
                previous: auxiliary_instances.then_some(&self.instances.prev_instance_buffer),
                records: records(base..base + cpu_instances.len(), base),
            });
            self.instances
                .debris
                .encode(&self.device, &self.queue, scene, &targets)
        };
        (instance_upload_bytes, debris_commands)
    }
}
