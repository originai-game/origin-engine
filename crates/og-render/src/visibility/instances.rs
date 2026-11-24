//! The frame's instance list, kept across frames: the submitted instances
//! stably sorted into (mesh, material, caster) runs, each sorted instance's
//! world bounding sphere and its transform as the GPU reads it.
//!
//! A scene usually submits the same instances in the same order frame after
//! frame, most of them where they were. So the sort is redone only when the
//! submission's (mesh, material, caster) sequence or a drawn mesh's bounds changed,
//! and a sphere and GPU transform are recomputed only for the instances
//! whose transform changed (compared bit for bit). An instance whose
//! transform mirrors its mesh ([`mirrors`]) is in a run of its own, drawn
//! with the other winding as the front; one that starts or stops mirroring
//! sorts the instances again. Everything kept is what
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
    // The outline group rides in the flash word's even part, 2 a group
    // (pbr.wgsl `look_flash_amount`, `look_outline`): an exact integer plus
    // the flash's fraction, so an instance without a group carries what it
    // did before groups existed.
    let group = match fx.outline {
        0 => 0.0,
        g => (1 + g % 255) as f32,
    };
    // How much of a model fading in is still hidden rides above the group,
    // 512 a step, with whether it fades out against another in the step's
    // bit 8 (instance.wgsl `look_hidden_steps`): a whole instance carries
    // what it did before, and the word stays below 2^18, so the flash
    // keeps six bits of its fraction while the instance fades.
    let hidden = fx.hidden_bits() as f32;
    [
        flash + 2.0 * group + 512.0 * hidden,
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
    /// Its instances are fading in ([`crate::InstanceFx::reveal`] below
    /// 1): the run's fragments are cut by the dither in every pass, so it
    /// draws with the pipelines that keep the discard and off the cluster
    /// path (`batches.rs`). Whole instances never share a run with them.
    pub reveal: bool,
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
    /// Its instances mirror the mesh ([`mirrors`]): its triangles wind the
    /// other way as seen ([`crate::Faces::mirrored`]).
    pub mirrored: bool,
    /// The box of its instances' world spheres ([`fold_box`]). A pass under
    /// GPU culling draws a run whole, whatever the GPU keeps of it, so it
    /// leaves out the runs whose box misses it ([`crate::Batch::reaches`]).
    pub reach: Option<(Vec3, Vec3)>,
}

/// Whether `m` mirrors what it places: a negative determinant, which turns
/// every triangle's winding as seen (glTF: such a node's triangles are
/// wound the other way).
pub fn mirrors(m: &Mat4) -> bool {
    let (x, y, z) = (
        m.x_axis.truncate(),
        m.y_axis.truncate(),
        m.z_axis.truncate(),
    );
    x.dot(y.cross(z)) < 0.0
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
    // A run without array maps carries the instance's sway there instead
    // (meshlet_sway.wgsl reads it for foliage meshes alone).
    let layer = if run.array {
        crate::material::instance_layer_bits(inst.material_layer)
    } else {
        inst.sway.max(0.0)
    };
    layer_bits(layer, inst.receive_shadows)
}

/// The sorted instances of the last frame and what was derived from them.
#[derive(Default)]
pub(crate) struct SortedInstances {
    /// The scene they came from (a cache is per scene).
    scene: u64,
    /// Per submitted instance, its (mesh, material) indices, its caster
    /// and fading flags ([`run_flags`]) and layer mask.
    keys: Vec<(u32, u32, u8, u32)>,
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
    /// Per sorted instance, its own material's index and its sway (as
    /// bits) as last seen: with its GPU transform, what the static shadow
    /// cache hashes of it ([`Sorted::changed`]).
    looks: Vec<(u32, u32)>,
    /// The sorted instances whose GPU transform or look the last refresh
    /// found changed.
    changed: Vec<u32>,
    /// The submitted instances as the last sort or refresh read them, and
    /// each one's sorted slot (`u32::MAX`: not drawn): a refresh compares
    /// the submission with them in its own order and visits through the
    /// sort only the instances that differ.
    seen: Vec<Instance>,
    slots: Vec<u32>,
    /// What else the keys were read from: each material's batch material
    /// ([`drawn`]), the instances' layer masks, and the mesh and material
    /// counts the handles were checked against. While these and an
    /// instance hold, so does its key.
    keyed: (Vec<u32>, Vec<u32>, usize, usize),
    /// Moves on whenever a sorted instance's GPU transform may have
    /// changed ([`Sorted::content`]).
    content: u64,
    /// The producer's number of the list last read
    /// ([`crate::KeptInstances::list`]; 0: none).
    list: u64,
    /// Counts the updates ([`Sorted::serial`]).
    serial: u64,
    /// The materials whose shader parameters moved this frame, ascending
    /// ([`SortedInstances::set_params_moved`]).
    params_moved: Vec<u32>,
    /// What the still hidden instances the last rebuild left out will
    /// draw with, once each ([`Sorted::ahead`]).
    ahead: Vec<Ahead>,
}

/// The material `m`'s instances batch under: the one it is an alias of
/// ([`RenderScene::alias_material`]), unless either's shader parameters
/// moved this frame. Equal parameters now do not make equal previous ones
/// (`previous_shader_params()`, the prepass's motion), so each of the two
/// then draws with its own uniform, and they batch again from the next frame.
fn drawn(scene: &RenderScene, params_moved: &[u32], m: usize) -> usize {
    let o = scene.drawn_material(m);
    let moved = |i: usize| params_moved.binary_search(&(i as u32)).is_ok();
    if o != m && (moved(m) || moved(o)) {
        m
    } else {
        o
    }
}

/// What [`SortedInstances::update`] lends a frame.
pub(crate) struct Sorted {
    pub runs: Vec<Run>,
    /// What the instances still hidden (fading in, nothing of them drawn
    /// yet) will draw with: in no run, their pipelines made ahead.
    pub ahead: Vec<Ahead>,
    /// Per sorted instance (only those of non-empty meshes).
    pub transforms: Vec<GpuTransform>,
    pub spheres: Vec<(Vec3, f32)>,
    /// Per sorted instance, its index in the submission.
    pub submission: Vec<u32>,
    /// The box of `spheres` as [`fold_box`] folds it.
    pub sphere_box: Option<(Vec3, Vec3)>,
    /// The update's number: one more than the update before.
    pub serial: u64,
    /// The sorted instances (indices into `transforms`) whose GPU
    /// transform, own material or sway changed since the update before;
    /// `None` when they were sorted afresh (any of them may have).
    pub changed: Option<Vec<u32>>,
    /// Every submitted instance is the update before's, bit for bit.
    pub still: bool,
    /// An instance's previous transform ([`crate::SceneObjects::previous_transforms`],
    /// when there is one per instance) is not its transform: read in the
    /// same pass as the comparison, for the frame's motion.
    pub objects_moved: bool,
    /// The sorted GPU transforms are those of every update with the same
    /// number here, bit for bit (a buffer they were written to holds them).
    pub content: u64,
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

/// Whether a run reaching `reach` ([`Run::reach`]; none: not known) may
/// touch the volume inside `planes` (`n · p + d >= 0` inside). False only
/// when its box lies outside one of them, where a GPU cull of that volume
/// keeps none of its instances: the box is widened past the rounding of
/// the GPU's own spheres (cull.wgsl), so a run is never found out of a
/// volume the GPU keeps one of its instances in.
pub(crate) fn reaches(reach: Option<(Vec3, Vec3)>, planes: &[glam::Vec4]) -> bool {
    let Some((lo, hi)) = reach else {
        return true;
    };
    let pad = 1e-4 * (lo.abs().max(hi.abs()).max_element() + (hi - lo).max_element());
    let (lo, hi) = (lo - Vec3::splat(pad), hi + Vec3::splat(pad));
    planes.iter().all(|p| {
        let n = p.truncate();
        n.dot(Vec3::select(n.cmpge(Vec3::ZERO), hi, lo)) + p.w >= 0.0
    })
}

/// A run's [`Run::dequant`]: none for one fading in, which draws its
/// mesh's f32 vertices (the compact shadow pipeline has no fragment to
/// cut).
fn run_dequant(
    dequant: &impl Fn(usize, usize) -> Option<Mat4>,
    mesh: usize,
    material: usize,
    reveal: bool,
) -> Option<Mat4> {
    if reveal {
        None
    } else {
        dequant(mesh, material)
    }
}

/// Whether an instance is fading in ([`Run::reveal`]).
fn revealing(inst: &Instance) -> bool {
    inst.fx.hidden_steps() != 0
}

/// Whether an instance fading in is still wholly hidden
/// ([`crate::InstanceFx::reveal`] 0): every pass would cut all of it, so
/// it is in no run, and the frame makes its pipelines ahead ([`Ahead`]).
fn hidden(inst: &Instance) -> bool {
    inst.fx.hidden_steps() >= 255
}

/// What an instance's key holds of its flags: bit 0 that it casts
/// shadows, bit 1 that it is fading in, bit 2 that it is still hidden.
fn run_flags(inst: &Instance) -> u8 {
    u8::from(inst.cast_shadows) | (u8::from(revealing(inst)) << 1) | (u8::from(hidden(inst)) << 2)
}

/// What a still hidden instance will draw with once its fade starts: the
/// frame starts the pipelines of its run, fading in and whole, without
/// waiting for them or drawing it (`Renderer::ahead_batches`), so the
/// frame that first shows it compiles nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Ahead {
    pub mesh: usize,
    pub material: usize,
    pub cast: bool,
    pub mirrored: bool,
}

/// An instance's own material and its sway (as bits): what the static
/// shadow cache hashes of it besides its GPU transform and mesh.
fn look(inst: &Instance) -> (u32, u32) {
    (inst.material.index() as u32, inst.sway.to_bits())
}

fn same_bits(a: &Mat4, b: &Mat4) -> bool {
    bytemuck::cast_ref::<Mat4, [u32; 16]>(a) == bytemuck::cast_ref::<Mat4, [u32; 16]>(b)
}

/// Numeric equality for motion, including signed zero and NaN as Mat4's
/// equality does. Reduce without an early-exit branch per matrix lane:
/// the still scene compares all sixteen lanes for every instance.
fn same_transform(a: &Mat4, b: &Mat4) -> bool {
    a.to_cols_array()
        .into_iter()
        .zip(b.to_cols_array())
        .fold(true, |same, (a, b)| same & (a == b))
}

/// Whether two instance looks are the same, bit for bit.
fn same_fx(a: &crate::InstanceFx, b: &crate::InstanceFx) -> bool {
    let fx = |f: &crate::InstanceFx| {
        [
            f.flash.to_bits(),
            f.flash_color.x.to_bits(),
            f.flash_color.y.to_bits(),
            f.flash_color.z.to_bits(),
            f.tint.x.to_bits(),
            f.tint.y.to_bits(),
            f.tint.z.to_bits(),
            f.outline,
            f.reveal.to_bits(),
        ]
    };
    fx(a) == fx(b)
}

/// Whether two instances are the same, bit for bit: everything a sort or
/// a refresh reads of one (its transform, layer, look, flags and sway).
fn same_instance(a: &Instance, b: &Instance) -> bool {
    a.mesh == b.mesh
        && a.material == b.material
        && same_bits(&a.transform, &b.transform)
        && a.material_layer == b.material_layer
        && same_fx(&a.fx, &b.fx)
        && a.cast_shadows == b.cast_shadows
        && a.receive_shadows == b.receive_shadows
        && a.sway.to_bits() == b.sway.to_bits()
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

    /// The materials whose shader parameters moved this frame (ascending),
    /// before [`SortedInstances::update`]: none of them draws in another's
    /// batch, nor any of their aliases in theirs.
    pub fn set_params_moved(&mut self, materials: Vec<u32>) {
        self.params_moved = materials;
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
        let instances = &scene.objects.instances;
        let n = instances.len();
        self.serial = self.serial.wrapping_add(1);
        let mut same = self.scene == scene.id() && self.keys.len() == n;
        self.scene = scene.id();
        // What the keys are read from besides the instances.
        let table: Vec<u32> = (0..materials)
            .map(|m| drawn(scene, &self.params_moved, m) as u32)
            .collect();
        let layers = &scene.objects.instance_layers;
        let keyed = same
            && self.seen.len() == n
            && self.slots.len() == n
            && self.keyed.0 == table
            && self.keyed.1 == *layers
            && (self.keyed.2, self.keyed.3) == (meshes, materials);
        // One pass in submission order: the instances that differ from
        // the update before's (only their keys and what derives from them
        // are read again), and whether one moved since its previous
        // transform. A still town of 94,865 instances walked them four
        // times a frame for these (the keys, the refresh, the motion
        // test, the sway kept for the next frame).
        // The instances the producer promises are the list last read
        // (`SceneObjects::kept`) are not compared, nor, when it promises
        // they did not move, tested for motion.
        let kept = scene.objects.kept;
        let vouched = match keyed && kept.from != 0 && kept.from == self.list {
            true => kept.count.min(n),
            false => 0,
        };
        self.list = kept.list;
        let previous = &scene.objects.previous_transforms;
        let previous = (previous.len() == n).then_some(previous.as_slice());
        let mut objects_moved = false;
        let mut differ: Vec<u32> = Vec::new();
        if keyed {
            for (i, (a, b)) in self.seen.iter().zip(instances).enumerate().skip(vouched) {
                if !same_instance(a, b) {
                    differ.push(i as u32);
                }
                if !objects_moved && let Some(p) = previous {
                    objects_moved = !same_transform(&p[i], &b.transform);
                }
            }
            // The promised instances' motion, unless promised too.
            let unpromised = if kept.still { 0 } else { vouched };
            if !objects_moved && let Some(p) = previous {
                let (p, i) = (&p[..unpromised], &instances[..unpromised]);
                objects_moved = p
                    .iter()
                    .zip(i)
                    .any(|(p, i)| !same_transform(p, &i.transform));
            }
        } else if let Some(p) = previous {
            objects_moved = p
                .iter()
                .zip(instances)
                .any(|(p, i)| !same_transform(p, &i.transform));
        }
        let all: Vec<u32>;
        let check: &[u32] = if keyed {
            &differ
        } else {
            all = (0..n as u32).collect();
            &all
        };
        // The submission's keys against the last frame's (and kept for the
        // next), each checked against the scene's resources on the way.
        self.keys.resize(n, (u32::MAX, u32::MAX, 1, 1));
        for &i in check {
            let i = i as usize;
            let (key, inst) = (&mut self.keys[i], &instances[i]);
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
            // A material batches under the one it is an alias of
            // (RenderScene::alias_material): its instances join that run.
            let k = (
                k.0 as u32,
                table[k.1],
                run_flags(inst),
                crate::views::layer_of(layers, i),
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
                same_dequant(
                    r.dequant,
                    run_dequant(&dequant, r.mesh, r.material, r.reveal),
                ) && r.array == array(r.material)
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
        same = same && self.seen.len() == n && self.slots.len() == n;
        let still = same && keyed && differ.is_empty();
        // An instance that moved into another probe's box sorts again.
        let rebuilt = !same
            || self
                .refresh(scene, &mesh, keyed.then_some(&differ))
                .is_none();
        if rebuilt {
            self.rebuild(scene, &mesh, &dequant, &array);
        }
        if rebuilt || !self.changed.is_empty() {
            self.content = self.content.wrapping_add(1);
        }
        self.keyed = (table, layers.clone(), meshes, materials);
        Ok(Sorted {
            runs: self.runs.clone(),
            ahead: self.ahead.clone(),
            transforms: std::mem::take(&mut self.transforms),
            spheres: std::mem::take(&mut self.spheres),
            submission: self.order.clone(),
            sphere_box: self.sphere_box,
            serial: self.serial,
            changed: (!rebuilt).then(|| std::mem::take(&mut self.changed)),
            still,
            objects_moved,
            content: self.content,
        })
    }

    /// Gives back what [`SortedInstances::update`] lent: `transforms` cut to
    /// the sorted instances (a frame appends culled copies after them).
    pub fn restore(&mut self, mut transforms: Vec<GpuTransform>, spheres: Vec<(Vec3, f32)>) {
        transforms.truncate(self.order.len());
        self.transforms = transforms;
        self.spheres = spheres;
    }

    /// What the last update made of each submitted instance it left out
    /// (`seen` per submission; the sorted ones are their culls'): a
    /// wireframe material's (the lines pass draws it), one fading in that
    /// shows nothing yet, else one with nothing to draw.
    pub(crate) fn record_left_out(&self, scene: &RenderScene, seen: &mut [crate::Culled]) {
        for ((&slot, inst), seen) in self.slots.iter().zip(&scene.objects.instances).zip(seen) {
            if slot != u32::MAX {
                continue;
            }
            let material = drawn(scene, &self.params_moved, inst.material.index()) as u32;
            *seen = if self.skipped.binary_search(&material).is_ok() {
                crate::Culled::Lines
            } else if hidden(inst) {
                crate::Culled::FadingIn
            } else {
                crate::Culled::Empty
            };
        }
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
        let instances = &scene.objects.instances;
        let mut order: Vec<usize> = (0..instances.len()).collect();
        // Stable: instances of one batch keep their submission order. A
        // (mesh, material) pair's casters come first, then each layer mask
        // a run of its own (a camera draws or skips a run whole).
        let layers = |i: usize| crate::views::layer_of(&scene.objects.instance_layers, i);
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
        og_core::sort::sort_by_key(&mut order, |&i| {
            let inst = &instances[i];
            (
                inst.mesh,
                drawn(scene, &self.params_moved, inst.material.index()),
                !inst.cast_shadows,
                revealing(inst),
                layers(i),
                probe_of(i),
                mirrors(&inst.transform),
            )
        });
        self.order.clear();
        self.matrices.clear();
        self.spheres.clear();
        self.transforms.clear();
        self.looks.clear();
        self.runs.clear();
        self.ahead.clear();
        self.meshes.clear();
        self.seen.clear();
        self.seen.extend_from_slice(instances);
        self.slots.clear();
        self.slots.resize(instances.len(), u32::MAX);
        for &i in &order {
            let inst = &instances[i];
            let (mesh, material, cast, reveal, mask, probe, mirrored) = (
                inst.mesh.index(),
                drawn(scene, &self.params_moved, inst.material.index()),
                inst.cast_shadows,
                revealing(inst),
                layers(i),
                probe_of(i),
                mirrors(&inst.transform),
            );
            let (bounds, empty) = mesh_info(mesh);
            // Sorted by mesh: a mesh's instances are together.
            if self.meshes.last().is_none_or(|&(m, _)| m != mesh) {
                self.meshes.push((mesh, (bounds, empty)));
            }
            if empty || self.skipped.binary_search(&(material as u32)).is_ok() {
                continue;
            }
            if hidden(inst) {
                let ahead = Ahead {
                    mesh,
                    material,
                    cast,
                    mirrored,
                };
                if !self.ahead.contains(&ahead) {
                    self.ahead.push(ahead);
                }
                continue;
            }
            match self.runs.last_mut() {
                Some(r)
                    if r.mesh == mesh
                        && r.material == material
                        && r.cast == cast
                        && r.reveal == reveal
                        && r.layers == mask
                        && r.probe == probe
                        && r.mirrored == mirrored =>
                {
                    r.count += 1
                }
                _ => self.runs.push(Run {
                    mesh,
                    material,
                    cast,
                    reveal,
                    layers: mask,
                    first: self.order.len() as u32,
                    count: 1,
                    dequant: run_dequant(dequant, mesh, material, reveal),
                    array: array(material),
                    probe,
                    mirrored,
                    reach: None,
                }),
            }
            let m = inst.transform;
            let sphere = world_sphere(&m, bounds);
            self.spheres.push(sphere);
            let run = self.runs.last_mut().expect("pushed above");
            run.reach = fold_box(run.reach, [sphere]);
            self.transforms.push(run_transform(&m, inst, run));
            self.matrices.push(m);
            self.looks.push(look(inst));
            self.slots[i] = self.order.len() as u32;
            self.order.push(i as u32);
        }
        self.sphere_box = fold_box(None, self.spheres.iter().copied());
        self.order.len()
    }

    /// The same sort: recomputes what changed with the transforms; returns
    /// how many instances that was, `None` when one moved into another
    /// reflection probe's box or started or stopped mirroring its mesh
    /// (the sort is then redone).
    /// `differ`: the submitted instances that differ from those last
    /// read, when known; otherwise every one is compared.
    fn refresh(
        &mut self,
        scene: &RenderScene,
        mesh_info: &impl Fn(usize) -> MeshInfo,
        differ: Option<&[u32]>,
    ) -> Option<usize> {
        let instances = &scene.objects.instances;
        let mut changed = 0;
        self.changed.clear();
        // In submission order against the instances as last read: one
        // that is the same, bit for bit, keeps everything derived from it,
        // so a still scene's frame reads its instances once, in order (a
        // walk through the sort's order was 13 ms of a 94,784-instance
        // town's frame).
        let all: Vec<u32>;
        let candidates = match differ {
            Some(d) => d,
            None => {
                all = (0..instances.len() as u32)
                    .filter(|&i| !same_instance(&self.seen[i as usize], &instances[i as usize]))
                    .collect();
                &all
            }
        };
        if candidates.is_empty() {
            return Some(0);
        }
        let mut dirty = vec![0u64; self.order.len().div_ceil(64)];
        for &i in candidates {
            let i = i as usize;
            let k = self.slots[i];
            if k != u32::MAX {
                dirty[k as usize / 64] |= 1 << (k % 64);
            } else {
                self.seen[i] = instances[i];
            }
        }
        // A bit per sorted slot keeps sparse updates sparse without
        // sorting a moving field or searching for its run per instance.
        for run in &mut self.runs {
            let bounds = mesh_info(run.mesh).0;
            let first = run.first as usize;
            let end = (run.first + run.count) as usize;
            let mut moved = 0;
            for (offset, &bits) in dirty[first / 64..end.div_ceil(64)].iter().enumerate() {
                let word = first / 64 + offset;
                let mut bits = bits;
                if word == first / 64 {
                    bits &= u64::MAX << (first % 64);
                }
                if word == end / 64 {
                    bits &= (1 << (end % 64)) - 1;
                }
                while bits != 0 {
                    let k = word * 64 + bits.trailing_zeros() as usize;
                    bits &= bits - 1;
                    let i = self.order[k] as usize;
                    let inst = &instances[i];
                    let fx = if same_fx(&self.seen[i].fx, &inst.fx) {
                        self.transforms[k].fx
                    } else {
                        fx_bits(&inst.fx)
                    };
                    self.seen[i] = *inst;
                    let m = &inst.transform;
                    if same_bits(m, &self.matrices[k]) {
                        // An array run's layer, whether an instance receives
                        // shadows and its look may change on their own.
                        let (layer, seen) = (run_layer(inst, run), look(inst));
                        let t = &mut self.transforms[k];
                        if layer.to_bits() != t.layer.to_bits()
                            || fx.map(f32::to_bits) != t.fx.map(f32::to_bits)
                            || seen != self.looks[k]
                        {
                            self.looks[k] = seen;
                            self.changed.push(k as u32);
                        }
                        t.layer = layer;
                        t.fx = fx;
                        continue;
                    }
                    if mirrors(m) != run.mirrored {
                        return None;
                    }
                    self.spheres[k] = world_sphere(m, bounds);
                    if !self.probes.is_empty()
                        && crate::probe_select::pick(&self.probes, self.spheres[k].0) != run.probe
                    {
                        return None;
                    }
                    self.transforms[k] = GpuTransform {
                        columns: match run.dequant {
                            Some(d) => gpu_columns(&(*m * d)),
                            None => gpu_columns(m),
                        },
                        layer: run_layer(inst, run),
                        fx,
                    };
                    self.matrices[k] = *m;
                    self.looks[k] = look(inst);
                    self.changed.push(k as u32);
                    changed += 1;
                    moved += 1;
                }
            }
            if moved > 0 {
                // From matrices, not spheres: a frame may have returned
                // the spheres widened for debris.
                let members = first..end;
                run.reach = if moved == run.count {
                    // Every sphere was freshly computed above; none can
                    // still contain the prior frame's debris expansion.
                    fold_box(None, self.spheres[members].iter().copied())
                } else {
                    fold_box(
                        None,
                        self.matrices[members]
                            .iter()
                            .map(|m| world_sphere(m, bounds)),
                    )
                };
            }
        }
        if changed > 0 {
            self.sphere_box = fold_box(None, self.spheres.iter().copied());
        }
        Some(changed)
    }
}
