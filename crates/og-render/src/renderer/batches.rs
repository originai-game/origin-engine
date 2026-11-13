//! Batch construction, culling and pipeline selection.

use super::*;
use crate::pipeline_features::{Features, Variants};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

mod crowds;
mod groups;
mod pipeline_jobs;
mod pipeline_select;
mod skinned;

pub(crate) use crowds::CrowdPlan;
pub(crate) use pipeline_jobs::{Made, variant_job, variants};
use pipeline_jobs::{Slot, Work, deferred};

impl Renderer {
    /// Sorts the instances into (mesh, material) batches
    /// ([`instances::SortedInstances`]: from the last frame's sort while the
    /// submission's keys hold); returns the batches and the sorted
    /// instances (transforms, world bounding spheres, submission indices),
    /// lent to the frame until [`Renderer::restore_sorted`].
    pub(crate) fn build_batches(
        &mut self,
        scene: &RenderScene,
    ) -> Result<(Vec<Batch>, instances::Sorted), RenderError> {
        let (meshes, materials) = (&self.cache.meshes, &self.cache.materials);
        let sorted = self.instances.sorted.update(
            scene,
            meshes.len(),
            |m| (meshes[m].bounds, meshes[m].index_count == 0),
            materials.len(),
            |mesh, material| Self::dequant(meshes, materials, mesh, material),
            |material| materials[material].array.is_some(),
            |material| materials[material].wire_only,
        )?;
        let batches = sorted
            .runs
            .iter()
            .map(|r| {
                let m = &self.cache.materials[r.material];
                Batch {
                    mesh: r.mesh,
                    material: r.material,
                    first: r.first,
                    count: r.count,
                    faces: Faces {
                        side: m.side,
                        mirrored: r.mirrored,
                    },
                    transparent: m.transparent,
                    blend: m.blend,
                    physical: m.physical.is_some(),
                    transmissive: m.transmissive,
                    dissolve: m.dissolve,
                    effects: m.effects,
                    shader: m.shader,
                    lean: m.lean,
                    derivatives: m.flat,
                    compact: r.dequant.is_some(),
                    array: r.array,
                    foliage: scene.mesh_foliage_all()[r.mesh].is_some(),
                    simulated: scene.mesh_surfaces[r.mesh].is_some(),
                    cast: r.cast,
                    layers: r.layers,
                    probe: r.probe,
                    crowd: None,
                }
            })
            .collect();
        Ok((batches, sorted))
    }

    /// How a (mesh, material) run undoes its vertices' quantisation
    /// ([`instances::Run::dequant`]): a plain material drawing a compact
    /// mesh draws its compact vertices; anything else the f32 ones.
    pub(crate) fn dequant(
        meshes: &[GpuMesh],
        materials: &[GpuMaterial],
        mesh: usize,
        material: usize,
    ) -> Option<Mat4> {
        (materials[material].lean && meshes[mesh].foliage.is_none())
            .then(|| meshes[mesh].compact.as_ref().map(|(_, dq)| dq.matrix()))
            .flatten()
    }

    /// The draw ranges of one pass: per batch (`include`d), every instance
    /// whose sphere touches `frustum`. With `whole`, a batch that loses
    /// less than a tenth of its instances is drawn whole from the sorted
    /// instances (cheaper than copying its transforms); otherwise, and
    /// always without `whole`, its visible instances are appended to
    /// `transforms`. A frame under GPU culling passes `whole` false: its
    /// sorted instances live in the culling's buffer, and the CPU lists
    /// (views, probe faces) draw only what they appended, from the
    /// instance buffer. `scratch` is the caller's, reused across the
    /// frame's passes; a batch drawn whole is marked in its `whole`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn cull(
        &self,
        batches: &[Batch],
        spheres: &[(Vec3, f32)],
        transforms: &mut Vec<instances::GpuTransform>,
        origins: &mut Vec<u32>,
        scratch: &mut CullScratch,
        frustum: &Frustum,
        whole: bool,
        include: impl Fn(&Batch) -> bool,
    ) -> (DrawList, u32) {
        let mut list = DrawList::default();
        let mut culled = 0;
        let mut visible = Vec::new();
        let inside = &mut scratch.inside;
        scratch.whole.resize(batches.len(), false);
        // Only the included batches' spheres are tested: they are marked
        // first, and the test leaves an unmarked one alone (no read of it
        // follows). On the lanes only from a six-figure count of bounds.
        inside.clear();
        inside.resize(spheres.len(), false);
        for b in batches.iter().filter(|b| include(b)) {
            inside[b.first as usize..(b.first + b.count) as usize].fill(true);
        }
        frustum.test_spheres(inside, spheres);
        for (b_index, b) in batches.iter().enumerate() {
            if !include(b) {
                continue;
            }
            let range = b.first as usize..(b.first + b.count) as usize;
            visible.clear();
            visible.extend(range.clone().filter(|&k| inside[k]));
            culled += b.count - visible.len() as u32;
            let index_count = self.cache.meshes[b.mesh].index_count;
            list.in_view_triangles += (index_count / 3) as u64 * visible.len() as u64;
            let (first, count) = if visible.is_empty() {
                continue;
            } else if whole && visible.len() as u32 * 10 >= b.count * 9 {
                scratch.whole[b_index] = true;
                (b.first, b.count)
            } else {
                let first = transforms.len() as u32;
                for &k in &visible {
                    transforms.push(transforms[k]);
                    origins.push(origins[k]);
                }
                (first, visible.len() as u32)
            };
            list.ranges.push((b_index as u32, first, count));
            list.instances += count;
            list.triangles += (index_count / 3) as u64 * count as u64;
        }
        (list, culled)
    }

    /// Whether the effect group carries the planar reflection material
    /// shaders read ([`planar_slot`] for this renderer's planar
    /// reflections).
    pub(crate) fn planar_slot(&self) -> bool {
        planar_slot(self.info.planar_reflections)
    }

    /// An effect group without the planar reflection, for the lightmapped
    /// pipelines (`fx_lean_layout`); `None` where the effect group has no
    /// planar reflection and serves them itself.
    pub(crate) fn fx_lean_group(
        &self,
        buffer: &wgpu::Buffer,
        ao: &wgpu::TextureView,
        atlas: &wgpu::TextureView,
    ) -> Option<wgpu::BindGroup> {
        Some(crate::fx::fx_group(
            &self.device,
            "fx (lightmapped)",
            self.fx_lean_layout.as_ref()?,
            &[
                buffer.as_entire_binding(),
                wgpu::BindingResource::TextureView(ao),
                wgpu::BindingResource::TextureView(atlas),
                wgpu::BindingResource::Sampler(&self.trilinear_sampler),
            ],
            &self.lights.decal_buffers,
        ))
    }

    /// The transparent instances in view, sorted back to front by the view
    /// depth of their bounds' centre (ties keep submission order), as runs of
    /// one (mesh, material) appended to `transforms`. `billboards`: the
    /// depths, far first, of the billboards drawn among them (the pass's
    /// [`sprites::SpriteFrame::billboards`]): a run is cut where one falls
    /// between two of its instances, and [`DrawList::before`] says how many
    /// draw before each run (a billboard as far as a mesh draws after it).
    /// Returns the list and how many were outside the view.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn sort_transparent(
        &self,
        batches: &[Batch],
        spheres: &[(Vec3, f32)],
        submission: &[u32],
        transforms: &mut Vec<instances::GpuTransform>,
        origins: &mut Vec<u32>,
        frustum: &Frustum,
        eye: Vec3,
        forward: Vec3,
        layers: u32,
        billboards: &[f32],
    ) -> (DrawList, u32) {
        let mut keyed: Vec<(f32, u32, usize, u32)> = Vec::new();
        let mut culled = 0;
        let drawn = |b: &Batch| b.transparent && b.layers & layers != 0;
        for (b_index, b) in batches.iter().enumerate().filter(|(_, b)| drawn(b)) {
            for k in b.first as usize..(b.first + b.count) as usize {
                let (c, r) = spheres[k];
                if frustum.intersects_sphere(c, r) {
                    keyed.push(((c - eye).dot(forward), submission[k], k, b_index as u32));
                } else {
                    culled += 1;
                }
            }
        }
        og_core::sort::sort_by(&mut keyed, |a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        let mut list = DrawList::default();
        let mut farther = 0;
        for (depth, _, k, b) in keyed {
            let at = transforms.len() as u32;
            transforms.push(transforms[k]);
            origins.push(origins[k]);
            let before = farther;
            while farther < billboards.len() && billboards[farther] > depth {
                farther += 1;
            }
            match list.ranges.last_mut() {
                Some(r) if r.0 == b && r.1 + r.2 == at && farther == before => r.2 += 1,
                _ => {
                    list.ranges.push((b, at, 1));
                    list.before.push(farther as u32);
                }
            }
            list.instances += 1;
            list.triangles += (self.cache.meshes[batches[b as usize].mesh].index_count / 3) as u64;
        }
        (list, culled)
    }

    /// World bounds of every instance as (center, radius), from their box
    /// ([`instances::fold_box`]; `None`: no instance).
    pub(crate) fn instance_bounds(bounds: Option<(Vec3, Vec3)>) -> (Vec3, f32) {
        let Some((lo, hi)) = bounds else {
            return (Vec3::ZERO, 1.0);
        };
        ((lo + hi) * 0.5, ((hi - lo).length() * 0.5).max(1e-3))
    }
}

/// The instance records every pass that draws meshes reads: this
/// frame's and the previous frame's, the skinned instances', the joint
/// palette, the sway and the debris moved on the GPU.
pub(crate) struct InstanceRecords {
    /// The CPU cull's per-instance scratch ([`Renderer::cull`]), kept
    /// between frames so no pass allocates it.
    /// The sorted instances, kept across frames (`instances.rs`).
    pub(crate) sorted: instances::SortedInstances,
    pub(crate) instance_buffer: wgpu::Buffer,
    /// What `instance_buffer` holds, so a frame writes only what changed.
    pub(crate) instance_upload: upload::DirtyUpload,
    pub(crate) prev_instance_buffer: wgpu::Buffer,
    /// What `prev_instance_buffer` holds ([`upload::DirtyUpload`]).
    pub(crate) prev_instance_upload: upload::DirtyUpload,
    pub(crate) skinned_buffer: wgpu::Buffer,
    pub(crate) palette: skin::GpuPalette,
    /// Each instance's sway at the last drawn frame, by instance index
    /// ([`Instance::sway`]): a swaying vertex's motion vector starts there.
    pub(crate) previous_sway: Vec<f32>,
    /// The debris layer's bodies (`debris.rs`).
    pub(crate) debris: debris::DebrisLayer,
}

/// Visibility: where instances are culled, and the GPU culling, indirect
/// draws and meshlets every view registers into.
pub(crate) struct Visibility {
    pub(crate) cull_inside: CullScratch,
    /// Where instances are culled ([`RenderDesc::culling`]), whether the
    /// adapter can on the GPU (compute and indirect draws), and the GPU
    /// path's state once a frame used it.
    pub(crate) desc_culling: Culling,
    pub(crate) indirect: bool,
    pub(crate) gpu_cull: Option<cull_gpu::GpuCulling>,
    /// The meshlet path (meshlet_gpu.rs), once a scene has a clustered mesh.
    pub(crate) meshlet: Option<meshlet_gpu::MeshletGpu>,
    /// Whether the next frame's GPU culling counts are read back
    /// ([`Renderer::request_culling_counts`]).
    pub(crate) culling_counts: bool,
}
