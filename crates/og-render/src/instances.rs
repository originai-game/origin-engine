//! The frame's instance list, kept across frames: the submitted instances
//! stably sorted into (mesh, material) runs, each sorted instance's world
//! bounding sphere and its transform as the GPU reads it.
//!
//! A scene usually submits the same instances in the same order frame after
//! frame, most of them where they were. So the sort is redone only when the
//! submission's (mesh, material) sequence or a drawn mesh's bounds changed,
//! and a sphere and GPU transform are recomputed only for the instances
//! whose transform changed (compared bit for bit). Everything kept is what
//! a fresh build computes, by the same operations, so a frame is the same
//! whether it was built fresh or from the last one.

use crate::{RenderError, RenderScene};
use glam::{Mat4, Vec3};

/// An instance's transform as the GPU reads it: the x, y and z of its four
/// columns (`instance.wgsl` `instance_matrix`), 48 bytes. Their w is taken
/// as (0, 0, 0, 1): an instance's transform is affine.
pub(crate) type GpuTransform = [[f32; 3]; 4];

/// `m` as the GPU reads it ([`GpuTransform`]).
pub(crate) fn gpu_transform(m: &Mat4) -> GpuTransform {
    [m.x_axis, m.y_axis, m.z_axis, m.w_axis].map(|c| c.truncate().to_array())
}

/// One (mesh, material) run of the sorted instances.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Run {
    pub mesh: usize,
    pub material: usize,
    pub first: u32,
    pub count: u32,
}

/// The sorted instances of the last frame and what was derived from them.
#[derive(Default)]
pub(crate) struct SortedInstances {
    /// The scene they came from (a cache is per scene).
    scene: u64,
    /// Per submitted instance, its (mesh, material) indices.
    keys: Vec<(u32, u32)>,
    /// Per drawn mesh the runs were built with: its index, local bounds
    /// and whether it was empty (an empty mesh's instances are dropped).
    meshes: Vec<(usize, MeshInfo)>,
    runs: Vec<Run>,
    /// Per sorted instance: its submission index, its transform as last
    /// seen, its world sphere and its GPU transform. `spheres` and
    /// `transforms` are lent to the frame ([`SortedInstances::update`]) and
    /// given back by [`SortedInstances::restore`]; a frame that never gives
    /// them back leaves the next one to rebuild.
    order: Vec<u32>,
    matrices: Vec<Mat4>,
    spheres: Vec<(Vec3, f32)>,
    transforms: Vec<GpuTransform>,
    /// The spheres' box (min and max), `None` when there are none.
    sphere_box: Option<(Vec3, Vec3)>,
}

/// What [`SortedInstances::update`] lends a frame.
pub(crate) struct Sorted {
    pub runs: Vec<Run>,
    /// Per sorted instance (only those of non-empty meshes).
    pub transforms: Vec<GpuTransform>,
    pub spheres: Vec<(Vec3, f32)>,
    /// Per sorted instance, its index in the submission.
    pub submission: Vec<u32>,
    /// The box of `spheres` as [`fold_box`] folds it.
    pub sphere_box: Option<(Vec3, Vec3)>,
}

/// What the sort needs of a mesh: its local bounds as (centre, half
/// extent) and whether it is empty (draws nothing).
pub(crate) type MeshInfo = (Option<(Vec3, Vec3)>, bool);

/// An instance's world bounding sphere: its mesh's local box through the
/// transform, the radius scaled by the transform's largest axis.
fn world_sphere(m: &Mat4, bounds: Option<(Vec3, Vec3)>) -> (Vec3, f32) {
    match bounds {
        Some((c, e)) => {
            let scale = m
                .x_axis
                .truncate()
                .length()
                .max(m.y_axis.truncate().length())
                .max(m.z_axis.truncate().length());
            (m.transform_point3(c), e.length() * scale)
        }
        None => (m.w_axis.truncate(), 0.0),
    }
}

/// Folds spheres into a box (min and max corner), starting from `start`.
pub(crate) fn fold_box(
    start: Option<(Vec3, Vec3)>,
    spheres: impl IntoIterator<Item = (Vec3, f32)>,
) -> Option<(Vec3, Vec3)> {
    let (mut lo, mut hi) = start.unwrap_or((Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)));
    for (c, r) in spheres {
        lo = lo.min(c - Vec3::splat(r));
        hi = hi.max(c + Vec3::splat(r));
    }
    (lo.x <= hi.x).then_some((lo, hi))
}

fn same_bits(a: &Mat4, b: &Mat4) -> bool {
    bytemuck::bytes_of(a) == bytemuck::bytes_of(b)
}

impl SortedInstances {
    /// The frame's sorted instances, from the last frame's where they still
    /// hold. Fails on an instance whose mesh or material index is not below
    /// `meshes` / `materials` (the renderer's copies of the scene's);
    /// `mesh` describes the mesh at an index below `meshes`.
    pub fn update(
        &mut self,
        scene: &RenderScene,
        meshes: usize,
        mesh: impl Fn(usize) -> MeshInfo,
        materials: usize,
    ) -> Result<Sorted, RenderError> {
        let instances = &scene.instances;
        for (i, inst) in instances.iter().enumerate() {
            if inst.mesh.index() >= meshes {
                return Err(RenderError::InvalidHandle(format!(
                    "instance {i}: mesh {}",
                    inst.mesh.index()
                )));
            }
            if inst.material.index() >= materials {
                return Err(RenderError::InvalidHandle(format!(
                    "instance {i}: material {}",
                    inst.material.index()
                )));
            }
        }
        // The submission's keys against the last frame's (and kept for the next).
        let mut same = self.scene == scene.id() && self.keys.len() == instances.len();
        self.scene = scene.id();
        self.keys.resize(instances.len(), (u32::MAX, u32::MAX));
        for (key, inst) in self.keys.iter_mut().zip(instances) {
            let k = (inst.mesh.index() as u32, inst.material.index() as u32);
            if *key != k {
                *key = k;
                same = false;
            }
        }
        same = same
            && self.meshes.iter().all(|&(m, info)| mesh(m) == info)
            && self.transforms.len() == self.order.len()
            && self.spheres.len() == self.order.len();
        if same {
            self.refresh(scene, &mesh);
        } else {
            self.rebuild(scene, &mesh);
        }
        Ok(Sorted {
            runs: self.runs.clone(),
            transforms: std::mem::take(&mut self.transforms),
            spheres: std::mem::take(&mut self.spheres),
            submission: self.order.clone(),
            sphere_box: self.sphere_box,
        })
    }

    /// Gives back what [`SortedInstances::update`] lent: `transforms` cut to
    /// the sorted instances (a frame appends culled copies after them).
    pub fn restore(&mut self, mut transforms: Vec<GpuTransform>, spheres: Vec<(Vec3, f32)>) {
        transforms.truncate(self.order.len());
        self.transforms = transforms;
        self.spheres = spheres;
    }

    /// Sorts and derives everything again; returns how many instances it
    /// computed (all those drawn).
    fn rebuild(&mut self, scene: &RenderScene, mesh_info: &impl Fn(usize) -> MeshInfo) -> usize {
        let instances = &scene.instances;
        let mut order: Vec<usize> = (0..instances.len()).collect();
        // Stable: instances of one batch keep their submission order.
        order.sort_by_key(|&i| (instances[i].mesh, instances[i].material));
        self.order.clear();
        self.matrices.clear();
        self.spheres.clear();
        self.transforms.clear();
        self.runs.clear();
        self.meshes.clear();
        for &i in &order {
            let inst = &instances[i];
            let (mesh, material) = (inst.mesh.index(), inst.material.index());
            let (bounds, empty) = mesh_info(mesh);
            // Sorted by mesh: a mesh's instances are together.
            if self.meshes.last().is_none_or(|&(m, _)| m != mesh) {
                self.meshes.push((mesh, (bounds, empty)));
            }
            if empty {
                continue;
            }
            match self.runs.last_mut() {
                Some(r) if r.mesh == mesh && r.material == material => r.count += 1,
                _ => self.runs.push(Run {
                    mesh,
                    material,
                    first: self.order.len() as u32,
                    count: 1,
                }),
            }
            let m = inst.transform;
            self.spheres.push(world_sphere(&m, bounds));
            self.transforms.push(gpu_transform(&m));
            self.matrices.push(m);
            self.order.push(i as u32);
        }
        self.sphere_box = fold_box(None, self.spheres.iter().copied());
        self.order.len()
    }

    /// The same sort: recomputes what changed with the transforms; returns
    /// how many instances that was.
    fn refresh(&mut self, scene: &RenderScene, mesh_info: &impl Fn(usize) -> MeshInfo) -> usize {
        let instances = &scene.instances;
        let mut changed = 0;
        for run in &self.runs {
            let bounds = mesh_info(run.mesh).0;
            for k in run.first as usize..(run.first + run.count) as usize {
                let m = &instances[self.order[k] as usize].transform;
                if same_bits(m, &self.matrices[k]) {
                    continue;
                }
                self.spheres[k] = world_sphere(m, bounds);
                self.transforms[k] = gpu_transform(m);
                self.matrices[k] = *m;
                changed += 1;
            }
        }
        if changed > 0 {
            self.sphere_box = fold_box(None, self.spheres.iter().copied());
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Material, MeshData};
    use glam::Quat;

    fn info(i: usize) -> MeshInfo {
        match i {
            0 => (Some((Vec3::ZERO, Vec3::splat(0.5))), false),
            1 => (Some((Vec3::Y, Vec3::new(1.0, 2.0, 0.5))), false),
            _ => (None, true),
        }
    }

    fn scene() -> RenderScene {
        let mut scene = RenderScene::new();
        let meshes = [0, 1, 2].map(|_| scene.add_mesh(MeshData::default()));
        let materials = [0, 1].map(|_| scene.add_material(Material::default()));
        for i in 0..300u32 {
            let m = Mat4::from_scale_rotation_translation(
                Vec3::splat(1.0 + (i % 5) as f32 * 0.25),
                Quat::from_rotation_y(i as f32 * 0.3),
                Vec3::new(i as f32, (i % 7) as f32, -(i as f32) * 0.5),
            );
            scene.add_instance(meshes[(i * 7 % 3) as usize], materials[(i % 2) as usize], m);
        }
        scene
    }

    fn fresh(scene: &RenderScene) -> Sorted {
        SortedInstances::default()
            .update(scene, 3, info, 2)
            .unwrap()
    }

    fn assert_same(a: &Sorted, b: &Sorted) {
        assert_eq!(a.runs, b.runs);
        assert_eq!(a.submission, b.submission);
        assert_eq!(
            bytemuck::cast_slice::<_, u8>(&a.transforms),
            bytemuck::cast_slice::<_, u8>(&b.transforms)
        );
        let bits = |s: &[(Vec3, f32)]| {
            s.iter()
                .map(|(c, r)| (c.to_array().map(f32::to_bits), r.to_bits()))
                .collect::<Vec<_>>()
        };
        assert_eq!(bits(&a.spheres), bits(&b.spheres));
        assert_eq!(a.sphere_box, b.sphere_box);
    }

    /// Kept, refreshed or rebuilt, a frame's sorted instances are a fresh
    /// build's, bit for bit, and only what changed is computed again.
    #[test]
    fn kept_instances_are_a_fresh_builds() {
        let mut scene = scene();
        let mut kept = SortedInstances::default();
        let first = kept.update(&scene, 3, info, 2).unwrap();
        assert_same(&first, &fresh(&scene));
        // The empty mesh's instances are dropped.
        assert_eq!(first.submission.len(), 200);
        assert!(
            first
                .runs
                .windows(2)
                .all(|w| (w[0].mesh, w[0].material) < (w[1].mesh, w[1].material))
        );
        kept.restore(first.transforms, first.spheres);
        // Unchanged: nothing computed.
        assert_eq!(kept.refresh(&scene, &info), 0);
        let still = kept.update(&scene, 3, info, 2).unwrap();
        assert_same(&still, &fresh(&scene));
        let mut culled = still.transforms;
        culled.push([[9.0; 3]; 4]);
        kept.restore(culled, still.spheres);
        // Three moved (one of them by a sign of zero only).
        scene.instances[3].transform.w_axis.x += 2.0;
        scene.instances[10].transform = Mat4::from_translation(Vec3::new(-3.0, 1.0, 0.0));
        scene.instances[7].transform.w_axis.y = -0.0;
        assert_eq!(kept.refresh(&scene, &info), 3);
        kept.refresh(&scene, &info);
        let moved = kept.update(&scene, 3, info, 2).unwrap();
        assert_same(&moved, &fresh(&scene));
        kept.restore(moved.transforms, moved.spheres);
        // Another order: sorted again.
        scene.instances.swap(0, 1);
        scene.instances[5].material = scene.instances[6].material;
        let resorted = kept.update(&scene, 3, info, 2).unwrap();
        assert_same(&resorted, &fresh(&scene));
        // Not given back: rebuilt.
        let rebuilt = kept.update(&scene, 3, info, 2).unwrap();
        assert_same(&rebuilt, &fresh(&scene));
        // A mesh's bounds changed: rebuilt with the new ones.
        kept.restore(rebuilt.transforms, rebuilt.spheres);
        let grown = |i: usize| {
            if i == 0 {
                (Some((Vec3::ZERO, Vec3::ONE)), false)
            } else {
                info(i)
            }
        };
        let after = kept.update(&scene, 3, grown, 2).unwrap();
        assert_same(
            &after,
            &SortedInstances::default()
                .update(&scene, 3, grown, 2)
                .unwrap(),
        );
        assert!(matches!(
            kept.update(&scene, 2, info, 2),
            Err(RenderError::InvalidHandle(_))
        ));
    }
}
