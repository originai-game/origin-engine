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
mod tests;
