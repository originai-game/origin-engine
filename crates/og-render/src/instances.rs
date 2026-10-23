//! The frame's instance list, kept across frames: the submitted instances
//! stably sorted into (mesh, material, caster) runs, each sorted instance's
//! world bounding sphere and its transform as the GPU reads it.
//!
//! A scene usually submits the same instances in the same order frame after
//! frame, most of them where they were. So the sort is redone only when the
//! submission's (mesh, material, caster) sequence or a drawn mesh's bounds changed,
//! and a sphere and GPU transform are recomputed only for the instances
//! whose transform changed (compared bit for bit). Everything kept is what
//! a fresh build computes, by the same operations, so a frame is the same
//! whether it was built fresh or from the last one.

use crate::{Instance, RenderError, RenderScene};
use glam::{Mat4, Vec3};

/// An instance as the GPU reads it, 64 bytes: the x, y and z of its
/// transform's four columns (`instance.wgsl` `instance_matrix`; their w is
/// taken as (0, 0, 0, 1): an instance's transform is affine), the
/// texture-array layer of an array-map material's instance plus one and
/// whether it receives shadows ([`layer_bits`]) and its look ([`fx_bits`];
/// with the layer, `InstanceFxIn` in pbr.wgsl).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GpuTransform {
    pub columns: [[f32; 3]; 4],
    pub layer: f32,
    pub fx: [f32; 3],
}

/// An instance's look as the lit shaders read it (pbr.wgsl `InstanceFxIn`
/// y, z and w): the flash amount, the flash colour and the tint's distance
/// from white, each colour 8:8:8 in an integer below 2^24 (exact in an f32,
/// so a copy through GPU culling keeps it). [`crate::InstanceFx::NONE`] is
/// all zeros, as an instance record that never had a look.
pub(crate) fn fx_bits(fx: &crate::InstanceFx) -> [f32; 3] {
    let pack = |c: Vec3| {
        let q = |x: f32| (x.clamp(0.0, 1.0) * 255.0 + 0.5) as u32;
        ((q(c.x) << 16) | (q(c.y) << 8) | q(c.z)) as f32
    };
    let flash = if fx.flash.is_finite() {
        fx.flash.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let color = if flash > 0.0 {
        pack(fx.flash_color)
    } else {
        0.0
    };
    [
        flash,
        color,
        pack(Vec3::ONE - fx.tint.clamp(Vec3::ZERO, Vec3::ONE)),
    ]
}

/// [`GpuTransform::layer`] (pbr.wgsl `InstanceFxIn` x): an array-map run's
/// instance layer plus one (`material::instance_layer_bits`; 0, the
/// material's own, in every other run), as is when the instance receives
/// shadows, else negated less one (pbr.wgsl `look_layer` and
/// `look_receives`). An instance that receives carries what it did before
/// the flag existed, and the value stays an exact integer in an f32 for
/// every layer below 2^23 (a texture array holds at most 2048).
pub(crate) fn layer_bits(layer: f32, receive_shadows: bool) -> f32 {
    if receive_shadows { layer } else { -1.0 - layer }
}

/// The columns of `m` as the GPU reads them ([`GpuTransform::columns`]).
pub(crate) fn gpu_columns(m: &Mat4) -> [[f32; 3]; 4] {
    // Written out: an array map is a loop of closure calls in the dev profile.
    let (x, y, z, w) = (m.x_axis, m.y_axis, m.z_axis, m.w_axis);
    [
        [x.x, x.y, x.z],
        [y.x, y.y, y.z],
        [z.x, z.y, z.z],
        [w.x, w.y, w.z],
    ]
}

/// `m` as the GPU reads it, layer 0, no look ([`GpuTransform`]).
pub(crate) fn gpu_transform(m: &Mat4) -> GpuTransform {
    GpuTransform {
        columns: gpu_columns(m),
        layer: 0.0,
        fx: [0.0; 3],
    }
}

/// One (mesh, material, caster) run of the sorted instances.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Run {
    pub mesh: usize,
    pub material: usize,
    /// Its instances cast shadows ([`Instance::cast_shadows`]): a run that
    /// does not is left out of the shadow passes.
    pub cast: bool,
    /// Its instances' three.js layer mask ([`RenderScene::instance_layers`]):
    /// a camera draws the run when the masks share a bit (`views.rs`).
    pub layers: u32,
    pub first: u32,
    pub count: u32,
    /// The run draws its mesh's compact vertices (`compact.rs`): this
    /// undoes their quantisation, as the right factor of each instance's
    /// GPU transform.
    pub dequant: Option<Mat4>,
    /// Its material samples texture arrays: its instances carry their
    /// layer ([`GpuTransform::layer`]).
    pub array: bool,
    /// The reflection probe its instances sample (`probe_select.rs`; an
    /// index into the frame's probes, [`crate::probe_select::NONE`]: none).
    pub probe: u8,
}

/// An instance's GPU transform in `run`.
fn run_transform(m: &Mat4, inst: &Instance, run: &Run) -> GpuTransform {
    let mut t = match run.dequant {
        Some(d) => gpu_transform(&(*m * d)),
        None => gpu_transform(m),
    };
    t.layer = run_layer(inst, run);
    t.fx = fx_bits(&inst.fx);
    t
}

/// An instance's [`GpuTransform::layer`] in `run`.
fn run_layer(inst: &Instance, run: &Run) -> f32 {
    let layer = if run.array {
        crate::material::instance_layer_bits(inst.material_layer)
    } else {
        0.0
    };
    layer_bits(layer, inst.receive_shadows)
}

/// The sorted instances of the last frame and what was derived from them.
#[derive(Default)]
pub(crate) struct SortedInstances {
    /// The scene they came from (a cache is per scene).
    scene: u64,
    /// Per submitted instance, its (mesh, material) indices, caster flag
    /// and layer mask.
    keys: Vec<(u32, u32, bool, u32)>,
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
    /// The materials whose instances the last rebuild left out (wireframe
    /// materials, drawn by the lines pass).
    skipped: Vec<u32>,
    /// The frame's reflection probe boxes ([`SortedInstances::set_probes`]):
    /// an instance's run is also keyed by the probe holding its centre.
    probes: Vec<(Vec3, Vec3)>,
    /// The boxes changed since the last sort.
    probes_moved: bool,
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
    bytemuck::cast_ref::<Mat4, [u32; 16]>(a) == bytemuck::cast_ref::<Mat4, [u32; 16]>(b)
}

impl SortedInstances {
    /// The frame's reflection probe boxes, before [`SortedInstances::update`]:
    /// other boxes than the last frame's sort the instances again.
    pub fn set_probes(&mut self, boxes: Vec<(Vec3, Vec3)>) {
        if boxes != self.probes {
            self.probes = boxes;
            self.probes_moved = true;
        }
    }

    /// The frame's sorted instances, from the last frame's where they still
    /// hold. Fails on an instance whose mesh or material index is not below
    /// `meshes` / `materials` (the renderer's copies of the scene's);
    /// `mesh` describes the mesh at an index below `meshes`, `dequant` a
    /// (mesh, material) run's [`Run::dequant`], `array` whether a material
    /// samples texture arrays ([`Run::array`]), `skip` whether a material's
    /// instances are left out (a wireframe material, `lines.rs`).
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        scene: &RenderScene,
        meshes: usize,
        mesh: impl Fn(usize) -> MeshInfo,
        materials: usize,
        dequant: impl Fn(usize, usize) -> Option<Mat4>,
        array: impl Fn(usize) -> bool,
        skip: impl Fn(usize) -> bool,
    ) -> Result<Sorted, RenderError> {
        let instances = &scene.instances;
        // The submission's keys against the last frame's (and kept for the
        // next), each checked against the scene's resources on the way.
        let mut same = self.scene == scene.id() && self.keys.len() == instances.len();
        self.scene = scene.id();
        self.keys
            .resize(instances.len(), (u32::MAX, u32::MAX, true, 1));
        for (i, (key, inst)) in self.keys.iter_mut().zip(instances).enumerate() {
            let k = (inst.mesh.index(), inst.material.index());
            if k.0 >= meshes || k.1 >= materials {
                // Never kept: the next frame compares afresh.
                self.keys.clear();
                return Err(RenderError::InvalidHandle(if k.0 >= meshes {
                    format!("instance {i}: mesh {}", k.0)
                } else {
                    format!("instance {i}: material {}", k.1)
                }));
            }
            let k = (
                k.0 as u32,
                k.1 as u32,
                inst.cast_shadows,
                crate::views::layer_of(&scene.instance_layers, i),
            );
            if *key != k {
                *key = k;
                same = false;
            }
        }
        let same_dequant = |a: Option<Mat4>, b: Option<Mat4>| match (a, b) {
            (Some(a), Some(b)) => same_bits(&a, &b),
            (a, b) => a.is_none() && b.is_none(),
        };
        same = same
            && self.meshes.iter().all(|&(m, info)| mesh(m) == info)
            && self.runs.iter().all(|r| {
                same_dequant(r.dequant, dequant(r.mesh, r.material)) && r.array == array(r.material)
            })
            && self.transforms.len() == self.order.len()
            && self.spheres.len() == self.order.len();
        // The materials left out are the same ones.
        let skipped: Vec<u32> = (0..materials as u32)
            .filter(|&m| skip(m as usize))
            .collect();
        same = same && skipped == self.skipped && !self.probes_moved;
        self.skipped = skipped;
        self.probes_moved = false;
        // An instance that moved into another probe's box sorts again.
        if !same || self.refresh(scene, &mesh).is_none() {
            self.rebuild(scene, &mesh, &dequant, &array);
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
    fn rebuild(
        &mut self,
        scene: &RenderScene,
        mesh_info: &impl Fn(usize) -> MeshInfo,
        dequant: &impl Fn(usize, usize) -> Option<Mat4>,
        array: &impl Fn(usize) -> bool,
    ) -> usize {
        let instances = &scene.instances;
        let mut order: Vec<usize> = (0..instances.len()).collect();
        // Stable: instances of one batch keep their submission order. A
        // (mesh, material) pair's casters come first, then each layer mask
        // a run of its own (a camera draws or skips a run whole).
        let layers = |i: usize| crate::views::layer_of(&scene.instance_layers, i);
        // Each instance's reflection probe (last: a run splits by probe
        // only where its instances sit in different boxes).
        let probe: Vec<u8> = if self.probes.is_empty() {
            Vec::new()
        } else {
            instances
                .iter()
                .map(|inst| {
                    let bounds = mesh_info(inst.mesh.index()).0;
                    crate::probe_select::pick(&self.probes, world_sphere(&inst.transform, bounds).0)
                })
                .collect()
        };
        let probe_of = |i: usize| probe.get(i).copied().unwrap_or(crate::probe_select::NONE);
        order.sort_by_key(|&i| {
            let inst = &instances[i];
            (
                inst.mesh,
                inst.material,
                !inst.cast_shadows,
                layers(i),
                probe_of(i),
            )
        });
        self.order.clear();
        self.matrices.clear();
        self.spheres.clear();
        self.transforms.clear();
        self.runs.clear();
        self.meshes.clear();
        for &i in &order {
            let inst = &instances[i];
            let (mesh, material, cast, mask, probe) = (
                inst.mesh.index(),
                inst.material.index(),
                inst.cast_shadows,
                layers(i),
                probe_of(i),
            );
            let (bounds, empty) = mesh_info(mesh);
            // Sorted by mesh: a mesh's instances are together.
            if self.meshes.last().is_none_or(|&(m, _)| m != mesh) {
                self.meshes.push((mesh, (bounds, empty)));
            }
            if empty || self.skipped.binary_search(&(material as u32)).is_ok() {
                continue;
            }
            match self.runs.last_mut() {
                Some(r)
                    if r.mesh == mesh
                        && r.material == material
                        && r.cast == cast
                        && r.layers == mask
                        && r.probe == probe =>
                {
                    r.count += 1
                }
                _ => self.runs.push(Run {
                    mesh,
                    material,
                    cast,
                    layers: mask,
                    first: self.order.len() as u32,
                    count: 1,
                    dequant: dequant(mesh, material),
                    array: array(material),
                    probe,
                }),
            }
            let m = inst.transform;
            self.spheres.push(world_sphere(&m, bounds));
            let run = self.runs.last().expect("pushed above");
            self.transforms.push(run_transform(&m, inst, run));
            self.matrices.push(m);
            self.order.push(i as u32);
        }
        self.sphere_box = fold_box(None, self.spheres.iter().copied());
        self.order.len()
    }

    /// The same sort: recomputes what changed with the transforms; returns
    /// how many instances that was, `None` when one moved into another
    /// reflection probe's box (the sort is then redone).
    fn refresh(
        &mut self,
        scene: &RenderScene,
        mesh_info: &impl Fn(usize) -> MeshInfo,
    ) -> Option<usize> {
        let instances = &scene.instances;
        let mut changed = 0;
        for run in &self.runs {
            let bounds = mesh_info(run.mesh).0;
            for k in run.first as usize..(run.first + run.count) as usize {
                let inst = &instances[self.order[k] as usize];
                let m = &inst.transform;
                if same_bits(m, &self.matrices[k]) {
                    // An array run's layer, whether an instance receives
                    // shadows and its look may change on their own.
                    self.transforms[k].layer = run_layer(inst, run);
                    self.transforms[k].fx = fx_bits(&inst.fx);
                    continue;
                }
                self.spheres[k] = world_sphere(m, bounds);
                if !self.probes.is_empty()
                    && crate::probe_select::pick(&self.probes, self.spheres[k].0) != run.probe
                {
                    return None;
                }
                self.transforms[k] = run_transform(m, inst, run);
                self.matrices[k] = *m;
                changed += 1;
            }
        }
        if changed > 0 {
            self.sphere_box = fold_box(None, self.spheres.iter().copied());
        }
        Some(changed)
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
            .update(scene, 3, info, 2, |_, _| None, |_| false, |_| false)
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

    /// One mesh and material in two reflection probes' boxes is two runs,
    /// each with its box's probe (an aggregate centre between the rooms
    /// would give both the same one, or none); an instance that walks into
    /// the other room sorts again and changes run.
    #[test]
    fn instances_in_two_probes_boxes_are_two_runs() {
        let mut scene = RenderScene::new();
        let mesh = scene.add_mesh(MeshData::default());
        let material = scene.add_material(Material::default());
        for x in [-5.0, -4.0, 5.0] {
            scene.add_instance(
                mesh,
                material,
                Mat4::from_translation(Vec3::new(x, 0.0, 0.0)),
            );
        }
        let mut kept = SortedInstances::default();
        kept.set_probes(vec![
            (Vec3::new(-8.0, -2.0, -2.0), Vec3::new(-2.0, 2.0, 2.0)),
            (Vec3::new(2.0, -2.0, -2.0), Vec3::new(8.0, 2.0, 2.0)),
        ]);
        let update = |k: &mut SortedInstances, s: &RenderScene| {
            k.update(s, 3, info, 2, |_, _| None, |_| false, |_| false)
                .unwrap()
        };
        let runs = |s: &Sorted| {
            s.runs
                .iter()
                .map(|r| (r.probe, r.count))
                .collect::<Vec<_>>()
        };
        let first = update(&mut kept, &scene);
        assert_eq!(runs(&first), vec![(0, 2), (1, 1)]);
        kept.restore(first.transforms, first.spheres);
        scene.instances[1].transform = Mat4::from_translation(Vec3::new(4.0, 0.0, 0.0));
        let walked = update(&mut kept, &scene);
        assert_eq!(runs(&walked), vec![(0, 1), (1, 2)]);
        kept.restore(walked.transforms, walked.spheres);
        // No probes: one run, sampling none.
        kept.set_probes(Vec::new());
        let none = update(&mut kept, &scene);
        assert_eq!(runs(&none), vec![(crate::probe_select::NONE, 3)]);
    }

    /// Kept, refreshed or rebuilt, a frame's sorted instances are a fresh
    /// build's, bit for bit, and only what changed is computed again.
    #[test]
    fn kept_instances_are_a_fresh_builds() {
        let mut scene = scene();
        let mut kept = SortedInstances::default();
        let first = kept
            .update(&scene, 3, info, 2, |_, _| None, |_| false, |_| false)
            .unwrap();
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
        assert_eq!(kept.refresh(&scene, &info), Some(0));
        let still = kept
            .update(&scene, 3, info, 2, |_, _| None, |_| false, |_| false)
            .unwrap();
        assert_same(&still, &fresh(&scene));
        let mut culled = still.transforms;
        culled.push(GpuTransform {
            columns: [[9.0; 3]; 4],
            layer: 0.0,
            fx: [0.0; 3],
        });
        kept.restore(culled, still.spheres);
        // Three moved (one of them by a sign of zero only).
        scene.instances[3].transform.w_axis.x += 2.0;
        scene.instances[10].transform = Mat4::from_translation(Vec3::new(-3.0, 1.0, 0.0));
        scene.instances[7].transform.w_axis.y = -0.0;
        assert_eq!(kept.refresh(&scene, &info), Some(3));
        kept.refresh(&scene, &info);
        let moved = kept
            .update(&scene, 3, info, 2, |_, _| None, |_| false, |_| false)
            .unwrap();
        assert_same(&moved, &fresh(&scene));
        kept.restore(moved.transforms, moved.spheres);
        // Another order: sorted again.
        scene.instances.swap(0, 1);
        scene.instances[5].material = scene.instances[6].material;
        let resorted = kept
            .update(&scene, 3, info, 2, |_, _| None, |_| false, |_| false)
            .unwrap();
        assert_same(&resorted, &fresh(&scene));
        // Not given back: rebuilt.
        let rebuilt = kept
            .update(&scene, 3, info, 2, |_, _| None, |_| false, |_| false)
            .unwrap();
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
        let after = kept
            .update(&scene, 3, grown, 2, |_, _| None, |_| false, |_| false)
            .unwrap();
        assert_same(
            &after,
            &SortedInstances::default()
                .update(&scene, 3, grown, 2, |_, _| None, |_| false, |_| false)
                .unwrap(),
        );
        assert!(matches!(
            kept.update(&scene, 2, info, 2, |_, _| None, |_| false, |_| false),
            Err(RenderError::InvalidHandle(_))
        ));
    }

    /// Instances that cast no shadow are runs of their own (after the
    /// casters of their mesh and material, each keeping its submission
    /// order), and flipping one instance's flag sorts again.
    #[test]
    fn non_casters_are_runs_of_their_own() {
        let mut scene = scene();
        for (i, inst) in scene.instances.iter_mut().enumerate() {
            inst.cast_shadows = i % 3 != 0;
        }
        let mut kept = SortedInstances::default();
        let sorted = kept
            .update(&scene, 3, info, 2, |_, _| None, |_| false, |_| false)
            .unwrap();
        assert_same(&sorted, &fresh(&scene));
        let mut seen = std::collections::HashSet::new();
        for r in &sorted.runs {
            assert!(seen.insert((r.mesh, r.material, r.cast)), "one run per key");
            let members = &sorted.submission[r.first as usize..(r.first + r.count) as usize];
            assert!(members.windows(2).all(|w| w[0] < w[1]), "submission order");
            for &i in members {
                let inst = &scene.instances[i as usize];
                assert_eq!(
                    (inst.mesh.index(), inst.material.index(), inst.cast_shadows),
                    (r.mesh, r.material, r.cast)
                );
            }
        }
        assert!(sorted.runs.iter().any(|r| !r.cast) && sorted.runs.iter().any(|r| r.cast));
        kept.restore(sorted.transforms, sorted.spheres);
        scene.instances[1].cast_shadows = false;
        let again = kept
            .update(&scene, 3, info, 2, |_, _| None, |_| false, |_| false)
            .unwrap();
        assert_same(&again, &fresh(&scene));
    }

    /// Whether an instance receives shadows rides in its record's layer
    /// lane (`layer_bits`): as before for one that receives, negated less
    /// one for one that does not, with an array run's layer kept beside it
    /// (pbr.wgsl `look_layer` reads it back). Flipping the flag sorts
    /// nothing and is seen the next frame, and the runs never split on it.
    #[test]
    fn the_receive_flag_rides_in_the_layer_lane() {
        let decode = |x: f32| (x >= 0.0, if x >= 0.0 { x } else { -x - 1.0 });
        for layer in [0.0, 1.0, 7.0, 2049.0, (1u32 << 23) as f32] {
            assert_eq!(layer_bits(layer, true), layer);
            assert_eq!(decode(layer_bits(layer, true)), (true, layer));
            assert_eq!(decode(layer_bits(layer, false)), (false, layer));
        }
        let mut scene = scene();
        for (i, inst) in scene.instances.iter_mut().enumerate() {
            inst.material_layer = Some(i as u32 % 5);
        }
        // Material 1 samples arrays.
        let array = |m: usize| m == 1;
        let update = |kept: &mut SortedInstances, scene: &RenderScene| {
            kept.update(scene, 3, info, 2, |_, _| None, array, |_| false)
                .unwrap()
        };
        let mut kept = SortedInstances::default();
        let before = update(&mut kept, &scene);
        let runs = before.runs.clone();
        kept.restore(before.transforms.clone(), before.spheres.clone());
        for inst in scene.instances.iter_mut().step_by(3) {
            inst.receive_shadows = false;
        }
        let after = update(&mut kept, &scene);
        assert_eq!(after.runs, runs, "the receive flag splits no run");
        assert_same(&after, &update(&mut SortedInstances::default(), &scene));
        for r in &after.runs {
            for k in r.first as usize..(r.first + r.count) as usize {
                let inst = &scene.instances[after.submission[k] as usize];
                let layer = if r.array {
                    inst.material_layer.unwrap() as f32 + 1.0
                } else {
                    0.0
                };
                let (was, now) = (before.transforms[k].layer, after.transforms[k].layer);
                assert_eq!(was, layer, "receiving: the layer as before");
                assert_eq!(decode(now), (inst.receive_shadows, layer));
                assert_eq!(before.transforms[k].columns, after.transforms[k].columns);
            }
        }
    }

    /// A run drawing compact vertices carries their dequantisation in its
    /// instances' GPU transforms; turning it on or off sorts again.
    #[test]
    fn compact_runs_fold_the_dequantisation_into_the_transforms() {
        let scene = scene();
        let d =
            Mat4::from_translation(Vec3::splat(-0.5)) * Mat4::from_scale(Vec3::new(1.0, 2.0, 0.5));
        let compact = |mesh: usize, material: usize| (mesh == 1 && material == 0).then_some(d);
        let mut kept = SortedInstances::default();
        let plain = kept
            .update(&scene, 3, info, 2, |_, _| None, |_| false, |_| false)
            .unwrap();
        kept.restore(plain.transforms.clone(), plain.spheres.clone());
        let with = kept
            .update(&scene, 3, info, 2, compact, |_| false, |_| false)
            .unwrap();
        for r in &with.runs {
            assert_eq!(r.dequant.is_some(), r.mesh == 1 && r.material == 0);
            for k in r.first as usize..(r.first + r.count) as usize {
                let m = scene.instances[with.submission[k] as usize].transform;
                let want = gpu_transform(&r.dequant.map_or(m, |d| m * d));
                assert_eq!(with.transforms[k], want);
            }
        }
        // The spheres are the mesh's, whatever the vertices.
        assert_eq!(with.spheres, plain.spheres);
        kept.restore(with.transforms, with.spheres);
        let back = kept
            .update(&scene, 3, info, 2, |_, _| None, |_| false, |_| false)
            .unwrap();
        assert_same(&back, &plain);
    }
}
