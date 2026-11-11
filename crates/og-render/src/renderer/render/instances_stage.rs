//! The instances' stage: under GPU culling the culling's encode (its sorted
//! instances to its own buffer) and the CPU-culled lists moved past them;
//! then the records a draw reads and the debris step, and the sorted
//! instances' scratch given back.

use super::*;

pub(super) struct UploadInstancesInputs<'a> {
    pub scene: &'a RenderScene,
    pub cam: &'a frame::CameraFrame,
    pub batches: &'a [Batch],
    pub transforms: Vec<instances::GpuTransform>,
    pub spheres: Vec<(Vec3, f32)>,
    pub origins: &'a [u32],
    pub debris_instances: &'a HashMap<u32, (usize, Mat4)>,
    pub sorted: u32,
    pub vis: &'a FrameVisibility,
    pub gpu_cull: bool,
    pub max_passes: usize,
    pub auxiliary_instances: bool,
    pub previous_valid: bool,
    pub object_motion: bool,
    pub prepass: bool,
    pub skinned_gpu: bool,
    pub skinned_batches: &'a [Batch],
    pub skinned_plain: usize,
    pub skinned_data: &'a [crate::skin::SkinnedInstanceGpu],
    pub skinned: &'a SkinnedFrame,
    pub reflection_list: &'a mut [DrawList],
    pub reflection_blended: &'a mut [DrawList],
    pub blended_list: &'a mut DrawList,
    pub probe_lists: &'a mut [DrawList],
    pub probe_blended: &'a mut [DrawList],
    pub view_lists: &'a mut [DrawList],
    pub view_blended: &'a mut [DrawList],
}

pub(super) struct UploadInstancesOutputs {
    pub cull_commands: Vec<wgpu::CommandBuffer>,
    pub gpu_counts: Option<(Option<cull_gpu::GpuCounts>, u32)>,
    pub instance_upload_bytes: u64,
    pub debris_commands: Option<wgpu::CommandBuffer>,
}

impl Renderer {
    /// Encodes the GPU culling and uploads the frame's instance records.
    #[allow(clippy::too_many_lines)]
    pub(super) fn upload_instances(
        &mut self,
        UploadInstancesInputs {
            scene,
            cam,
            batches,
            transforms,
            spheres,
            origins,
            debris_instances,
            sorted,
            vis,
            gpu_cull,
            max_passes,
            auxiliary_instances,
            previous_valid,
            object_motion,
            prepass,
            skinned_gpu,
            skinned_batches,
            skinned_plain,
            skinned_data,
            skinned,
            reflection_list,
            reflection_blended,
            blended_list,
            probe_lists,
            probe_blended,
            view_lists,
            view_blended,
        }: UploadInstancesInputs<'_>,
    ) -> UploadInstancesOutputs {
        // Upload instances, previous transforms, skinned data, strips and
        // sprites. With GPU culling the sorted instances go to its buffer and
        // this one holds only the transparent draws.
        let copy_counts = std::mem::take(&mut self.visibility.culling_counts);
        let (cull_commands, gpu_counts) = if gpu_cull {
            let (commands, counts) = self.encode_gpu_cull(GpuCullInputs {
                scene,
                batches,
                transforms: &transforms,
                origins,
                sorted,
                vis,
                max_passes,
                auxiliary_instances,
                previous_valid,
                object_motion,
                prepass,
                skinned: skinned_gpu.then_some(cull_gpu::SkinnedCull {
                    batches: &skinned_batches[..skinned_plain],
                    records: skinned_data,
                    boxes: &skinned.boxes,
                }),
                cam,
                copy_counts,
            });
            // The CPU-culled lists sit after the sorted instances, in this buffer.
            for list in reflection_list
                .iter_mut()
                .filter(|l| l.indirect.is_none())
                .chain(reflection_blended.iter_mut())
                .chain(std::iter::once(&mut *blended_list))
            {
                for r in &mut list.ranges {
                    r.1 -= sorted;
                }
            }
            for r in probe_lists
                .iter_mut()
                .chain(probe_blended.iter_mut())
                .chain(view_lists.iter_mut().filter(|l| l.indirect.is_none()))
                .chain(view_blended.iter_mut())
                .flat_map(|l| &mut l.ranges)
            {
                r.1 -= sorted;
            }
            (commands, Some(counts))
        } else {
            if let Some(m) = self.visibility.meshlet.as_mut() {
                m.frame.clear();
            }
            (Vec::new(), None)
        };
        // The records a draw reads and the debris step (instance_uploads.rs).
        let (instance_upload_bytes, debris_commands) =
            self.upload_instance_records(InstanceUploadInputs {
                scene,
                batches,
                transforms: &transforms,
                origins,
                debris_instances,
                sorted,
                gpu_cull,
                auxiliary_instances,
                previous_valid,
            });
        self.instances.sorted.restore(transforms, spheres);
        UploadInstancesOutputs {
            cull_commands,
            gpu_counts,
            instance_upload_bytes,
            debris_commands,
        }
    }
}
