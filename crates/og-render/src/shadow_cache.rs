//! Static shadow caching (`docs/design.md` §2.6): a shadow layer whose
//! view stands still keeps what its static casters drew, and a frame draws
//! only what moves over it.
//!
//! The split follows Unreal's virtual shadow maps' separate static cache
//! and HDRP's mixed cached shadows: casters that stood still for
//! [`QUIET_FRAMES`] frames are *static* and drawn into a layer once; the
//! frame restores that and draws the *dynamic* casters over it (moving
//! instances, swaying foliage, skinned casters, terrain, scattered copies,
//! grass, the clustered casters). A static caster that appears, goes,
//! starts or stops moving invalidates only the layers its sphere reaches
//! (Unreal's per-primitive invalidation), and a layer whose view changed
//! (its light, its cascade's fit) draws everything at once as before.
//!
//! Instances are tracked by submission index and batches are the unit of
//! the split: an instance that moves makes its whole batch (mesh and
//! material) dynamic while it moves. A layer's content lives in the live
//! shadow array, which the lit shaders read; the static content a layer
//! needs back under an overlay is kept in a texture of its own and put
//! back by the overlay's pass (`shadow_keep.rs`). That costs a fullscreen
//! draw a frame, so a layer under an overlay is kept only when its static
//! casters cost more than that to draw ([`KEEP_TEXELS_PER_TRIANGLE`]):
//! a terrain, a crowd or a meadow over a few props draws in one pass.
//! Rendering never feeds back into the simulation: the cache only decides
//! what a frame redraws.
//!
//! The rule: whatever a vertex shader moves without its instance's
//! transform changing is dynamic. The skinned casters (joints and morph
//! targets: morphed meshes draw only as skinned instances), scattered
//! copies in [`crate::Wind`], og.Grass's shadow (the first cascade) and the
//! clustered casters (the meshlet path's foliage sway among them) draw over
//! every layer they reach, every frame; an instance batch is dynamic while
//! [`deforms`] says so (foliage in a wind, a material shader). A new kind
//! of vertex deformation joins one of the two.

use glam::Vec3;

use crate::frame::Frustum;

/// Frames an instance must stand still before its shadow joins a cached
/// layer: a caster moving on alternate frames stays dynamic instead of
/// invalidating its layers twice a cycle.
pub(crate) const QUIET_FRAMES: u8 = 8;

/// How many texels of a layer one static triangle is worth: a layer under
/// an overlay is kept only when drawing its static casters again costs
/// more than putting the kept layer back, one fullscreen draw
/// (`shadow_keep.rs`). A terrain or a crowd over a few static props is
/// then one pass, as if nothing were cached.
pub(crate) const KEEP_TEXELS_PER_TRIANGLE: u64 = 64;

/// The most dynamic casters whose spheres decide per layer whether it
/// needs an overlay; past it every layer takes one.
pub(crate) const MAX_TESTED_DYNAMIC: usize = 256;

/// One submitted instance as last seen.
#[derive(Clone, Copy, Debug, Default)]
struct Tracked {
    /// What its shadow is drawn from (transform as drawn, mesh, material
    /// and their revisions, sway), hashed.
    hash: u64,
    /// Frames it has been the same, up to [`QUIET_FRAMES`].
    quiet: u8,
    /// Its sphere while it is among the cached (static) casters.
    cached: Option<(Vec3, f32)>,
    /// Its mesh's triangles: what drawing its shadow again costs.
    triangles: u32,
    /// Drawn at least once (`hash` holds something).
    seen: bool,
}

/// What one shadow layer (a cascade or a local face, by its shadow slot)
/// holds.
#[derive(Clone, Copy, Debug, Default)]
struct Layer {
    /// The view it was last drawn with ([`view_key`]), and the frame.
    key: Option<u64>,
    frame: u64,
    /// It was drawn with that view the frame before too.
    still: bool,
    /// The live layer holds the static casters alone.
    live_pure: bool,
    /// The layer's cache texture holds them.
    cached: bool,
    /// Whether its static casters are worth keeping under an overlay
    /// ([`KEEP_TEXELS_PER_TRIANGLE`]), while its view and they hold;
    /// `None`: not counted since.
    worth: Option<bool>,
}

/// What a frame does for one shadow layer, in order: draw its static
/// casters (clearing the layer), copy the layer into its kept texture,
/// then one pass that puts the kept texture back and draws the dynamic
/// casters over it (`restore` alone: the kept texture back, nothing over
/// it). `draw_static` with `overlay` and without `save` is one pass of
/// every caster (the layer is not kept); nothing set is a layer held as
/// it is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Plan {
    pub draw_static: bool,
    pub save: bool,
    pub restore: bool,
    pub overlay: bool,
}

impl Plan {
    /// Every caster in one pass, nothing cached (the uncached path).
    pub(crate) const ALL: Plan = Plan {
        draw_static: true,
        save: false,
        restore: false,
        overlay: true,
    };

    /// Whether the layer's static casters come from the cache.
    pub(crate) fn cached(&self) -> bool {
        !self.draw_static
    }

    /// Whether the layer is drawn in one pass of every caster.
    pub(crate) fn single(&self) -> bool {
        self.draw_static && self.overlay && !self.save
    }

    /// Whether the layer's dynamic casters are drawn over its kept static
    /// ones.
    pub(crate) fn over_kept(&self) -> bool {
        self.overlay && !self.single()
    }

    /// The plan for a layer whose depth is not its value (a local
    /// light's: `shadow_keep.rs`). Its static casters are saved in the
    /// frame that draws them, while the depth buffer the layers share
    /// still holds theirs: a layer that held them alone and now takes an
    /// overlay draws them again.
    pub(crate) fn with_own_depth(self) -> Plan {
        Plan {
            draw_static: self.draw_static || self.save,
            ..self
        }
    }

    /// Whether the frame reads or writes the layer's kept texture.
    pub(crate) fn keeps(&self) -> bool {
        self.save || self.restore || self.over_kept()
    }
}

/// The frame's split: per batch whether it is drawn over the cached
/// layers, and the spheres that invalidate them.
#[derive(Debug, Default)]
pub(crate) struct Split {
    pub dynamic: Vec<bool>,
    /// The static casters' spheres that changed (where they were and where
    /// they are): a layer they reach draws its static casters again.
    pub dirty: Vec<(Vec3, f32)>,
    /// The dynamic casters' spheres (all of them while at most
    /// [`MAX_TESTED_DYNAMIC`], else `None`: every layer takes an overlay).
    pub moving: Option<Vec<(Vec3, f32)>>,
    /// Whether any static caster is drawn this frame (without one nothing
    /// is cached).
    pub any_static: bool,
}

/// A batch as the split reads it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct BatchInfo {
    pub first: u32,
    pub count: u32,
    pub cast: bool,
    /// One instance's triangles.
    pub triangles: u32,
    /// Dynamic whatever its instances do (swaying foliage, a material
    /// shader's clock).
    pub always_dynamic: bool,
}

/// The batches a [`Split`] draws over the cached layers, asked as a layer's
/// lists ask, by a batch's first instance and count: a table over the first
/// instances, so each ask is one load. A binary search of the dynamic
/// batches' sorted firsts, per batch, layer and list, was most of the
/// cascades' list building (engine s108). A batch without instances is never
/// dynamic, nor is a first past the last dynamic batch's.
#[derive(Debug)]
pub(crate) struct DynamicFirsts(Vec<bool>);

impl DynamicFirsts {
    /// From each batch's `(first, count, dynamic)` ([`Split::dynamic`]).
    pub(crate) fn new(
        batches: impl IntoIterator<Item = (u32, u32, bool)> + Clone,
    ) -> DynamicFirsts {
        let dynamic = |(_, count, dynamic): &(u32, u32, bool)| *dynamic && *count > 0;
        let len = batches
            .clone()
            .into_iter()
            .filter(dynamic)
            .map(|(first, _, _)| first as usize + 1)
            .max()
            .unwrap_or(0);
        let mut at = vec![false; len];
        for (first, _, _) in batches.into_iter().filter(dynamic) {
            at[first as usize] = true;
        }
        DynamicFirsts(at)
    }

    /// Whether the batch of these instances is a dynamic one.
    pub(crate) fn contains(&self, first: u32, count: u32) -> bool {
        count > 0 && self.0.get(first as usize).copied().unwrap_or(false)
    }
}

/// Whether an instance batch's vertices move without its transforms
/// changing, so its shadow is never cached: a foliage mesh (its batch's
/// [`crate::RenderScene::set_mesh_foliage`]; og.Model's sway) while the
/// wind drawn this frame or the last blows (`swaying`: the sway animates
/// with the clock at a steady strength, and settles the frame after it
/// stops), and a batch with a material shader, whose vertex hook may
/// displace it with the clock.
pub(crate) fn deforms(foliage: bool, swaying: bool, shader: bool) -> bool {
    (foliage && swaying) || shader
}

/// The renderer's static shadow cache state (the textures stay with the
/// renderer).
#[derive(Debug, Default)]
pub(crate) struct ShadowCache {
    instances: Vec<Tracked>,
    layers: Vec<Layer>,
    /// The frame counter: a layer not drawn the frame before missed that
    /// frame's invalidations and is drawn afresh.
    frame: u64,
    dirty: Vec<(Vec3, f32)>,
    /// Whether the frame's split draws any static caster.
    any_static: bool,
    /// A frame was planned and not submitted ([`ShadowCache::submitted`]):
    /// its layers were never drawn, so the next frame forgets them.
    pending: bool,
}

/// The hash the cache keys on (FNV-1a over 32-bit words).
pub(crate) fn hash_words(words: impl IntoIterator<Item = u32>) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for w in words {
        h ^= u64::from(w);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// What a sorted instance's shadow is drawn from, as [`ShadowCache::split`]
/// reads it: its GPU transform, sway, mesh and material, and the mesh's and
/// material's `revisions`, hashed.
pub(crate) fn instance_hash(
    t: &crate::instances::GpuTransform,
    inst: &crate::Instance,
    revisions: [u64; 2],
) -> u64 {
    let (mesh, material) = (inst.mesh.index(), inst.material.index());
    hash_words(
        bytemuck::cast_slice::<_, u32>(std::slice::from_ref(t))
            .iter()
            .copied()
            .chain([inst.sway.to_bits(), mesh as u32, material as u32])
            .chain(bytemuck::cast_slice::<_, u32>(&revisions).iter().copied()),
    )
}

/// Each sorted instance's [`instance_hash`], kept across frames: a frame
/// hashes again the instances [`crate::instances::Sorted::changed`] lists
/// and those whose mesh's or material's revision moved, and every one after
/// a fresh sort or an update it missed. Each hash is the one hashing them
/// all would give.
#[derive(Debug, Default)]
pub(crate) struct InstanceHashes {
    hashes: Vec<u64>,
    /// Per sorted instance, the mesh and material its hash read.
    read: Vec<(u32, u32)>,
    /// Every mesh's and material's revision, as the hashes read them.
    meshes: Vec<u64>,
    materials: Vec<u64>,
    /// The update of the sorted instances the hashes are of.
    serial: Option<u64>,
}

/// Brings `kept` to `now`; per index whether it differed (a new index
/// did), `None` when none did.
fn moved(kept: &mut Vec<u64>, now: impl ExactSizeIterator<Item = u64>) -> Option<Vec<bool>> {
    let len = now.len();
    kept.truncate(len);
    let mut out: Option<Vec<bool>> = None;
    for (i, r) in now.enumerate() {
        match kept.get_mut(i) {
            Some(k) if *k == r => continue,
            Some(k) => *k = r,
            None => kept.push(r),
        }
        out.get_or_insert_with(|| vec![false; len])[i] = true;
    }
    out
}

impl InstanceHashes {
    /// The hashes of `n` sorted instances, from `sorted`'s update (its
    /// `serial` and `changed`): `hash(k)` is the k-th one's with the mesh
    /// and material it read; `meshes` and `materials` give every mesh's
    /// and material's revision.
    pub(crate) fn update(
        &mut self,
        n: usize,
        (serial, changed): (u64, Option<&[u32]>),
        meshes: impl ExactSizeIterator<Item = u64>,
        materials: impl ExactSizeIterator<Item = u64>,
        mut hash: impl FnMut(usize) -> (u64, (u32, u32)),
    ) -> &[u64] {
        let mesh_moved = moved(&mut self.meshes, meshes);
        let material_moved = moved(&mut self.materials, materials);
        let follows = self.serial == Some(serial.wrapping_sub(1)) && self.hashes.len() == n;
        self.serial = Some(serial);
        match changed.filter(|_| follows) {
            None => {
                self.hashes.clear();
                self.read.clear();
                for k in 0..n {
                    let (h, r) = hash(k);
                    self.hashes.push(h);
                    self.read.push(r);
                }
            }
            Some(changed) => {
                let mut put = |s: &mut Self, k: usize| (s.hashes[k], s.read[k]) = hash(k);
                for &k in changed {
                    put(self, k as usize);
                }
                if mesh_moved.is_some() || material_moved.is_some() {
                    let stale = |(m, t): (u32, u32)| {
                        mesh_moved.as_ref().is_some_and(|v| v[m as usize])
                            || material_moved.as_ref().is_some_and(|v| v[t as usize])
                    };
                    for k in 0..n {
                        if stale(self.read[k]) {
                            put(self, k);
                        }
                    }
                }
            }
        }
        &self.hashes
    }
}

/// A layer's view: its shadow pass uniform's view, light and projection
/// (not its clock: a material shader's batch is dynamic instead), the
/// scene's origin and the map's side.
pub(crate) fn view_key(
    view_proj: &[[f32; 4]; 4],
    light: [f32; 4],
    paraboloid: [f32; 4],
    origin: Vec3,
    side: u32,
) -> u64 {
    hash_words(
        view_proj
            .iter()
            .flatten()
            .chain(&light)
            .chain(&paraboloid)
            .chain(&origin.to_array())
            .map(|v| v.to_bits())
            .chain([side]),
    )
}

fn same_sphere(a: Option<(Vec3, f32)>, b: Option<(Vec3, f32)>) -> bool {
    let bits =
        |s: Option<(Vec3, f32)>| s.map(|(c, r)| (c.to_array().map(f32::to_bits), r.to_bits()));
    bits(a) == bits(b)
}

impl ShadowCache {
    /// Forgets every layer (the shadow arrays were made anew or resized);
    /// the instances keep their history.
    pub(crate) fn forget_layers(&mut self) {
        self.layers.clear();
    }

    /// Starts a frame: tracks each sorted instance (`submission[k]` its
    /// index in the scene's `submitted` instances, `hashes[k]` what its
    /// shadow is drawn from, `spheres[k]` its world sphere) and splits the
    /// batches into static and dynamic.
    pub(crate) fn split(
        &mut self,
        submitted: usize,
        batches: &[BatchInfo],
        submission: &[u32],
        hashes: &[u64],
        spheres: &[(Vec3, f32)],
    ) -> Split {
        self.frame += 1;
        self.dirty.clear();
        if std::mem::replace(&mut self.pending, true) {
            self.layers.clear();
        }
        // Instances gone: their cached shadows go with them.
        for t in self.instances.iter().skip(submitted) {
            if let Some(s) = t.cached {
                self.dirty.push(s);
            }
        }
        self.instances.truncate(submitted);
        self.instances.resize(submitted, Tracked::default());
        let mut drawn = vec![false; submitted];
        for (k, &i) in submission.iter().enumerate() {
            let i = i as usize;
            let t = &mut self.instances[i];
            drawn[i] = true;
            // A new instance stands still until it moves.
            t.quiet = if !t.seen {
                QUIET_FRAMES
            } else if t.hash != hashes[k] {
                0
            } else {
                t.quiet.saturating_add(1).min(QUIET_FRAMES)
            };
            t.hash = hashes[k];
            t.seen = true;
        }
        let mut split = Split {
            dynamic: Vec::with_capacity(batches.len()),
            moving: Some(Vec::new()),
            ..Default::default()
        };
        for b in batches {
            let slots = b.first as usize..(b.first + b.count) as usize;
            let dynamic = b.always_dynamic
                || submission[slots.clone()]
                    .iter()
                    .any(|&i| self.instances[i as usize].quiet < QUIET_FRAMES);
            split.dynamic.push(dynamic);
            split.any_static |= b.cast && !dynamic && b.count > 0;
            for k in slots {
                let t = &mut self.instances[submission[k] as usize];
                t.triangles = b.triangles;
                let now = (b.cast && !dynamic).then_some(spheres[k]);
                if !same_sphere(now, t.cached) {
                    self.dirty.extend(t.cached);
                    self.dirty.extend(now);
                    t.cached = now;
                }
                if b.cast && dynamic {
                    match &mut split.moving {
                        Some(m) if m.len() < MAX_TESTED_DYNAMIC => m.push(spheres[k]),
                        _ => split.moving = None,
                    }
                }
            }
        }
        // Instances not drawn this frame (an emptied mesh, a skipped
        // material) leave the cache too.
        for (t, _) in self.instances.iter_mut().zip(&drawn).filter(|(_, d)| !**d) {
            if let Some(s) = t.cached.take() {
                self.dirty.push(s);
            }
        }
        split.dirty = self.dirty.clone();
        self.any_static = split.any_static;
        split
    }

    /// The frame's plans were encoded and submitted: its layers hold what
    /// they say.
    pub(crate) fn submitted(&mut self) {
        self.pending = false;
    }

    /// Whether a layer of `frustum` needs its dynamic casters drawn over
    /// it, as far as the split's instances go (the renderer adds the
    /// casters it draws unculled).
    pub(crate) fn moving_in(split: &Split, frustum: &Frustum) -> bool {
        match &split.moving {
            Some(m) => m.iter().any(|&(c, r)| frustum.intersects_sphere(c, r)),
            None => true,
        }
    }

    /// What shadow `slot` does this frame: drawn from view `key` over
    /// `frustum` into `texels` texels, with dynamic casters to draw over
    /// it (`overlay`) or not. Call [`ShadowCache::split`] first.
    pub(crate) fn plan(
        &mut self,
        slot: usize,
        key: u64,
        frustum: &Frustum,
        overlay: bool,
        texels: u64,
    ) -> Plan {
        if self.layers.len() <= slot {
            self.layers.resize(slot + 1, Layer::default());
        }
        let frame = self.frame;
        let s = &mut self.layers[slot];
        let same_key = s.frame + 1 == frame && s.key == Some(key);
        // A view that held until now is likely to hold again: a cascade's
        // moves in steps (`shadow_fit.rs`).
        let holds = same_key || s.still;
        s.still = same_key;
        let reached = |&(c, r): &(Vec3, f32)| frustum.intersects_sphere(c, r);
        let clean = same_key && !self.dirty.iter().any(reached);
        let kept = clean && (s.live_pure || s.cached);
        s.key = Some(key);
        s.frame = frame;
        if !clean {
            s.worth = None;
        }
        if !kept {
            s.cached = false;
        }
        if !overlay {
            // Static casters alone: the live layer holds them.
            let plan = Plan {
                draw_static: !kept,
                restore: kept && !s.live_pure,
                ..Plan::default()
            };
            s.live_pure = true;
            return plan;
        }
        // A view that moves on every frame has nothing to keep yet; one
        // that holds keeps what costs more to draw than to put back.
        let instances = &self.instances;
        let worth = self.any_static
            && holds
            && *s
                .worth
                .get_or_insert_with(|| worth_keeping(instances, frustum, texels));
        if !worth {
            s.cached = false;
            s.live_pure = false;
            return Plan::ALL;
        }
        // Kept from now (a still view whose static casters changed, one
        // that took a step, a layer that held them alone), or put back.
        let plan = Plan {
            draw_static: !kept,
            save: !s.cached,
            restore: s.cached,
            overlay: true,
        };
        s.cached = true;
        s.live_pure = false;
        plan
    }
}

/// Whether the static casters that reach `frustum` cost more to draw than
/// a layer of `texels` costs to put back.
fn worth_keeping(instances: &[Tracked], frustum: &Frustum, texels: u64) -> bool {
    let need = (texels / KEEP_TEXELS_PER_TRIANGLE).max(1);
    let mut triangles = 0;
    instances.iter().any(|t| {
        if let Some((c, r)) = t.cached
            && frustum.intersects_sphere(c, r)
        {
            triangles += u64::from(t.triangles);
        }
        triangles >= need
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Mat4;

    fn box_frustum(x: f32) -> Frustum {
        // An orthographic volume 2 m wide around (x, 0, 0), looking down -z.
        let proj = glam::camera::rh::proj::directx::orthographic(-1.0, 1.0, -1.0, 1.0, 0.0, 20.0);
        let view = Mat4::from_translation(Vec3::new(-x, 0.0, -10.0));
        Frustum::from_view_proj(proj * view)
    }

    /// Two batches of one instance each: one at x = 0, one at x = 10.
    struct World {
        cache: ShadowCache,
        hashes: Vec<u64>,
        spheres: Vec<(Vec3, f32)>,
    }

    impl World {
        fn new() -> World {
            World {
                cache: ShadowCache::default(),
                hashes: vec![1, 2],
                spheres: vec![(Vec3::ZERO, 0.5), (Vec3::new(10.0, 0.0, 0.0), 0.5)],
            }
        }

        /// The next frame, the last one submitted.
        fn frame(&mut self) -> Split {
            self.cache.submitted();
            self.unsubmitted()
        }

        /// The next frame, the last one never submitted.
        fn unsubmitted(&mut self) -> Split {
            let batches = [0, 1].map(|first| BatchInfo {
                first,
                count: 1,
                cast: true,
                triangles: 12,
                always_dynamic: false,
            });
            self.cache
                .split(2, &batches, &[0, 1], &self.hashes, &self.spheres)
        }

        /// Moves the second instance by `dx`.
        fn nudge(&mut self, dx: f32) {
            self.hashes[1] += 1;
            self.spheres[1].0.x += dx;
        }
    }

    /// A still scene draws each layer once and then keeps it; a layer whose
    /// view changes draws again.
    #[test]
    fn a_still_layer_is_drawn_once() {
        let mut w = World::new();
        let near = box_frustum(0.0);
        let draw = Plan {
            draw_static: true,
            ..Plan::default()
        };
        for frame in 0..4 {
            let split = w.frame();
            assert!(split.any_static && split.dirty.len() == if frame == 0 { 2 } else { 0 });
            let plan = w.cache.plan(0, 7, &near, false, 1);
            assert_eq!(plan, if frame == 0 { draw } else { Plan::default() });
        }
        w.frame();
        assert_eq!(w.cache.plan(0, 8, &near, false, 1), draw, "a new view");
        // A layer skipped for a frame missed its invalidations.
        w.frame();
        w.frame();
        assert_eq!(w.cache.plan(0, 8, &near, false, 1), draw);
        // A frame planned and never submitted drew nothing.
        w.frame();
        assert_eq!(w.cache.plan(0, 8, &near, false, 1), Plan::default());
        w.unsubmitted();
        assert_eq!(w.cache.plan(0, 8, &near, false, 1), draw);
    }

    /// Under an overlay a layer is kept only when its static casters cost
    /// more to draw than the kept layer costs to put back.
    #[test]
    fn a_layer_is_kept_only_when_its_static_casters_cost_more() {
        let near = box_frustum(0.0);
        // Twelve triangles in a layer worth thirteen: one pass a frame.
        let mut w = World::new();
        for _ in 0..3 {
            w.frame();
            let plan = w
                .cache
                .plan(0, 1, &near, true, 13 * KEEP_TEXELS_PER_TRIANGLE);
            assert_eq!(plan, Plan::ALL);
        }
        // In a layer worth twelve: drawn and saved once the view holds,
        // then put back under the overlay.
        let mut w = World::new();
        let texels = 12 * KEEP_TEXELS_PER_TRIANGLE;
        let save = Plan {
            draw_static: true,
            save: true,
            overlay: true,
            ..Plan::default()
        };
        let back = Plan {
            restore: true,
            overlay: true,
            ..Plan::default()
        };
        for expected in [Plan::ALL, save, back, back] {
            w.frame();
            assert_eq!(w.cache.plan(0, 1, &near, true, texels), expected);
        }
    }

    /// A caster that moves turns dynamic at once and back to static after
    /// [`QUIET_FRAMES`] still frames; only the layers its spheres reach
    /// draw again, and while it moves the layer it is in, which holds no
    /// other caster, draws in one pass.
    #[test]
    fn a_moving_caster_invalidates_only_the_layers_it_reaches() {
        let mut w = World::new();
        let near = box_frustum(0.0);
        let far = box_frustum(10.0);
        w.frame();
        w.cache.plan(0, 1, &near, false, 1);
        w.cache.plan(1, 2, &far, false, 1);
        // It moves: dynamic, its old sphere dirty.
        w.nudge(0.1);
        let split = w.frame();
        assert_eq!(split.dynamic, vec![false, true]);
        assert_eq!(split.dirty.len(), 1);
        assert!(!ShadowCache::moving_in(&split, &near));
        assert!(ShadowCache::moving_in(&split, &far));
        assert_eq!(w.cache.plan(0, 1, &near, false, 1), Plan::default());
        // The far layer has no static caster left to keep: one pass.
        assert_eq!(w.cache.plan(1, 2, &far, true, 1), Plan::ALL);
        // Still moving: the same.
        for _ in 0..3 {
            w.nudge(0.1);
            let split = w.frame();
            assert!(split.dirty.is_empty());
            assert_eq!(w.cache.plan(0, 1, &near, false, 1), Plan::default());
            assert_eq!(w.cache.plan(1, 2, &far, true, 1), Plan::ALL);
        }
        // It stops: dynamic for QUIET_FRAMES - 1 more frames, then static
        // and dirty where it stands.
        for f in 1..=QUIET_FRAMES {
            let split = w.frame();
            let still = f == QUIET_FRAMES;
            assert_eq!(split.dynamic[1], !still, "frame {f}");
            assert_eq!(split.dirty.len(), still as usize);
            assert_eq!(w.cache.plan(0, 1, &near, false, 1), Plan::default());
            let plan = w.cache.plan(1, 2, &far, !still, 1);
            if still {
                assert_eq!(
                    plan,
                    Plan {
                        draw_static: true,
                        ..Plan::default()
                    }
                );
            } else {
                assert_eq!(plan, Plan::ALL);
            }
        }
        // Without the mover the far layer's cache comes back once.
        let split = w.frame();
        assert!(split.dirty.is_empty());
        assert_eq!(w.cache.plan(1, 2, &far, false, 1), Plan::default());
    }

    /// A view that moves every frame never pays for the cache: one pass of
    /// every caster, nothing saved; one that takes a step after holding
    /// still is kept from the step on; a layer without static casters is
    /// not cached.
    #[test]
    fn a_moving_view_draws_in_one_pass() {
        let mut w = World::new();
        let near = box_frustum(0.0);
        for key in 0..4 {
            w.frame();
            assert_eq!(w.cache.plan(0, key, &near, true, 1), Plan::ALL);
        }
        w.frame();
        assert!(w.cache.plan(0, 3, &near, true, 1).save, "still: kept");
        w.frame();
        let held = w.cache.plan(0, 3, &near, true, 1);
        assert!(held.cached() && held.restore && !held.save);
        w.frame();
        let step = w.cache.plan(0, 4, &near, true, 1);
        assert!(step.draw_static && step.save && !step.single(), "{step:?}");
        w.frame();
        assert!(w.cache.plan(0, 4, &near, true, 1).cached());
        w.frame();
        w.cache.plan(0, 5, &near, true, 1);
        w.frame();
        assert_eq!(w.cache.plan(0, 6, &near, true, 1), Plan::ALL, "moving");
        w.frame();
        assert!(w.cache.plan(0, 6, &near, true, 1).save, "still: kept");
        w.frame();
        let plan = w.cache.plan(0, 3, &near, true, u64::MAX);
        assert_eq!(plan, Plan::ALL, "nothing static");
        assert!(plan.single() && !plan.cached());
    }

    /// Foliage is dynamic only in a wind, a material shader always; a plain
    /// batch never is (its instances' transforms say when it moves).
    #[test]
    fn what_a_vertex_shader_moves_is_dynamic() {
        assert!(deforms(true, true, false));
        assert!(
            !deforms(true, false, false),
            "foliage without wind is still"
        );
        assert!(!deforms(false, true, false), "wind moves only foliage");
        assert!(deforms(false, false, true));
        assert!(!deforms(false, false, false));
    }

    /// A layer's lists ask the table what a binary search of the dynamic
    /// batches' sorted firsts answered, for every batch and for firsts that
    /// are no batch's (an empty batch at a dynamic one's first, past the
    /// table's end).
    #[test]
    fn dynamic_firsts_answer_as_the_sorted_firsts_did() {
        // (first, count, dynamic): runs of instances, empty batches among
        // them, the last batch dynamic.
        let batches: Vec<(u32, u32, bool)> = vec![
            (0, 3, false),
            (3, 2, true),
            (5, 0, true),
            (5, 4, false),
            (9, 1, true),
            (10, 0, false),
            (10, 6, false),
            (16, 2, true),
            (18, 5, false),
            (23, 1, true),
        ];
        let mut firsts: Vec<u32> = batches
            .iter()
            .filter(|(_, count, dynamic)| *dynamic && *count > 0)
            .map(|(first, _, _)| *first)
            .collect();
        firsts.sort_unstable();
        let before = |first: u32, count: u32| count > 0 && firsts.binary_search(&first).is_ok();
        let table = DynamicFirsts::new(batches.iter().copied());
        for &(first, count, _) in &batches {
            assert_eq!(
                table.contains(first, count),
                before(first, count),
                "batch at {first}, {count}"
            );
        }
        for first in 0..40 {
            for count in [0, 1, 7] {
                assert_eq!(
                    table.contains(first, count),
                    before(first, count),
                    "{first}, {count}"
                );
            }
        }
        assert!(table.contains(23, 1), "the last dynamic batch");
        assert!(
            !table.contains(3, 0),
            "an empty batch at a dynamic one's first"
        );
        assert!(
            !DynamicFirsts::new(std::iter::empty()).contains(0, 1),
            "no dynamic batch"
        );
    }

    /// An instance gone or not drawn takes its cached shadow with it; a
    /// new one starts still and only dirties where it appears.
    #[test]
    fn instances_that_come_and_go_dirty_their_spheres() {
        let mut w = World::new();
        w.frame();
        let one = [BatchInfo {
            first: 0,
            count: 1,
            cast: true,
            triangles: 12,
            always_dynamic: false,
        }];
        let split = w.cache.split(1, &one, &[0], &[1], &w.spheres[..1]);
        assert_eq!(split.dirty.len(), 1);
        assert_eq!(split.dirty[0].0.x, 10.0);
        let split = w.cache.split(2, &one, &[0], &[1], &w.spheres[..1]);
        assert!(split.dirty.is_empty(), "the second is not drawn yet");
        let split = w.frame();
        assert_eq!(split.dynamic, vec![false, false], "new instances are still");
        assert_eq!(split.dirty.len(), 1);
        // A batch that is always dynamic is never cached.
        let swaying = [BatchInfo {
            always_dynamic: true,
            ..one[0]
        }];
        let split = w.cache.split(1, &swaying, &[0], &[1], &w.spheres[..1]);
        assert!(!split.any_static && split.dynamic == vec![true]);
        assert_eq!(split.dirty.len(), 2, "both leave the cache");
    }

    /// Kept across frames, the instance hashes are the ones hashing every
    /// instance gives, after each change they read: a transform, a look, a
    /// sway the GPU transform does not show, an own material the sort does
    /// not see (an alias), a mesh's and a material's revision, a new order,
    /// and an update the hashes missed.
    #[test]
    fn kept_instance_hashes_are_a_full_passes() {
        use crate::instances::SortedInstances;
        let mut scene = crate::RenderScene::new();
        let meshes = [0, 1].map(|_| scene.add_mesh(crate::MeshData::default()));
        let materials = [0, 1, 2].map(|_| scene.add_material(crate::Material::default()));
        for i in 0..60u32 {
            let m = Mat4::from_translation(Vec3::new(i as f32, 0.0, 0.0));
            scene.add_instance(meshes[(i % 2) as usize], materials[(i % 2) as usize], m);
        }
        assert!(scene.alias_material(materials[2], materials[1]));
        let info = |_: usize| (Some((Vec3::ZERO, Vec3::splat(0.5))), false);
        let mut revisions = (vec![1u64; 2], vec![1u64; 3]);
        let mut sorted = SortedInstances::default();
        let mut kept = InstanceHashes::default();
        // One frame: the kept hashes, a full pass's, and how many changed.
        let mut frame =
            |scene: &crate::RenderScene, revisions: &(Vec<u64>, Vec<u64>), hashed: bool| {
                let s = sorted
                    .update(scene, 2, info, 3, |_, _| None, |_| false, |_| false)
                    .unwrap();
                let hash = |k: usize| {
                    let inst = &scene.objects.instances[s.submission[k] as usize];
                    let (m, t) = (inst.mesh.index(), inst.material.index());
                    let h = instance_hash(&s.transforms[k], inst, [revisions.0[m], revisions.1[t]]);
                    (h, (m as u32, t as u32))
                };
                let n = s.transforms.len();
                let full: Vec<u64> = (0..n).map(|k| hash(k).0).collect();
                let got = if hashed {
                    let (m, t) = (revisions.0.iter().copied(), revisions.1.iter().copied());
                    kept.update(n, (s.serial, s.changed.as_deref()), m, t, hash)
                        .to_vec()
                } else {
                    full.clone()
                };
                let changed = s.changed.as_ref().map(Vec::len);
                sorted.restore(s.transforms, s.spheres);
                (got == full, changed)
            };
        assert_eq!(frame(&scene, &revisions, true), (true, None), "the first");
        assert_eq!(
            frame(&scene, &revisions, true),
            (true, Some(0)),
            "unchanged"
        );
        scene.objects.instances[3].transform.w_axis.y = 2.0;
        assert_eq!(frame(&scene, &revisions, true), (true, Some(1)), "moved");
        scene.objects.instances[4].fx.flash = 0.5;
        assert_eq!(frame(&scene, &revisions, true), (true, Some(1)), "flashed");
        // Its GPU transform carries max(sway, 0): both 0.
        scene.objects.instances[5].sway = -1.0;
        assert_eq!(frame(&scene, &revisions, true), (true, Some(1)), "sway");
        // Drawn in the same batches: the sort keeps its place.
        scene.objects.instances[7].material = materials[2];
        assert_eq!(
            frame(&scene, &revisions, true),
            (true, Some(1)),
            "aliased material"
        );
        revisions.0[1] += 1;
        assert_eq!(
            frame(&scene, &revisions, true),
            (true, Some(0)),
            "a mesh's revision"
        );
        revisions.1[2] += 1;
        assert_eq!(
            frame(&scene, &revisions, true),
            (true, Some(0)),
            "a material's revision"
        );
        scene.objects.instances.swap(0, 1);
        assert_eq!(
            frame(&scene, &revisions, true),
            (true, None),
            "another order"
        );
        // A frame that sorted and failed before hashing: the next hashes all.
        scene.objects.instances[9].transform.w_axis.z = 1.0;
        assert_eq!(frame(&scene, &revisions, false), (true, Some(1)));
        assert_eq!(
            frame(&scene, &revisions, true),
            (true, Some(0)),
            "after a missed update"
        );
    }
}
