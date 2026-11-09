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
//! needs back after an overlay is kept in a texture of its own (made on
//! first need, in the renderer). Rendering never feeds back into the
//! simulation: the cache only decides what a frame redraws.
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
    /// The live layer holds the static casters alone.
    live_pure: bool,
    /// The layer's cache texture holds them.
    cached: bool,
}

/// What a frame does for one shadow layer, in order: draw its static
/// casters (clearing the layer), copy the layer into its cache texture,
/// copy the cache texture back into the layer, draw its dynamic casters
/// over it. `draw_static` with `overlay` and without `save` is one pass of
/// every caster (the layer is not cached); nothing set is a layer kept as
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
    /// `frustum`, with dynamic casters to draw over it (`overlay`) or not.
    /// Call [`ShadowCache::split`] first; `any_static` is its split's.
    pub(crate) fn plan(
        &mut self,
        slot: usize,
        key: u64,
        frustum: &Frustum,
        overlay: bool,
        any_static: bool,
    ) -> Plan {
        if self.layers.len() <= slot {
            self.layers.resize(slot + 1, Layer::default());
        }
        let frame = self.frame;
        let dirty = &self.dirty;
        let s = &mut self.layers[slot];
        let same_key = s.frame + 1 == frame && s.key == Some(key);
        let kept = same_key
            && (s.live_pure || s.cached)
            && !dirty.iter().any(|&(c, r)| frustum.intersects_sphere(c, r));
        s.key = Some(key);
        s.frame = frame;
        if !any_static {
            // Nothing to keep: the layer draws as it always did.
            s.live_pure = !overlay;
            s.cached = false;
            return Plan {
                draw_static: true,
                save: false,
                restore: false,
                overlay,
            };
        }
        if !kept {
            s.cached = false;
            if !overlay {
                s.live_pure = true;
                Plan {
                    draw_static: true,
                    ..Plan::default()
                }
            } else if same_key {
                // A still view whose static casters changed: kept from now.
                s.cached = true;
                s.live_pure = false;
                Plan {
                    draw_static: true,
                    save: true,
                    restore: false,
                    overlay: true,
                }
            } else {
                // A view that moved: nothing to keep yet.
                s.live_pure = false;
                Plan::ALL
            }
        } else if !overlay {
            if s.live_pure {
                Plan::default()
            } else {
                s.live_pure = true;
                Plan {
                    restore: true,
                    ..Plan::default()
                }
            }
        } else {
            let plan = Plan {
                draw_static: false,
                save: !s.cached,
                restore: s.cached,
                overlay: true,
            };
            s.cached = true;
            s.live_pure = false;
            plan
        }
    }
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
            let plan = w.cache.plan(0, 7, &near, false, true);
            assert_eq!(plan, if frame == 0 { draw } else { Plan::default() });
        }
        w.frame();
        assert_eq!(w.cache.plan(0, 8, &near, false, true), draw, "a new view");
        // A layer skipped for a frame missed its invalidations.
        w.frame();
        w.frame();
        assert_eq!(w.cache.plan(0, 8, &near, false, true), draw);
        // A frame planned and never submitted drew nothing.
        w.frame();
        assert_eq!(w.cache.plan(0, 8, &near, false, true), Plan::default());
        w.unsubmitted();
        assert_eq!(w.cache.plan(0, 8, &near, false, true), draw);
    }

    /// A caster that moves turns dynamic at once and back to static after
    /// [`QUIET_FRAMES`] still frames; only the layers its spheres reach
    /// draw again, and while it moves the layer it is in restores its cache
    /// and draws it over.
    #[test]
    fn a_moving_caster_invalidates_only_the_layers_it_reaches() {
        let mut w = World::new();
        let near = box_frustum(0.0);
        let far = box_frustum(10.0);
        w.frame();
        w.cache.plan(0, 1, &near, false, true);
        w.cache.plan(1, 2, &far, false, true);
        // It moves: dynamic, its old sphere dirty.
        w.nudge(0.1);
        let split = w.frame();
        assert_eq!(split.dynamic, vec![false, true]);
        assert_eq!(split.dirty.len(), 1);
        assert!(!ShadowCache::moving_in(&split, &near));
        assert!(ShadowCache::moving_in(&split, &far));
        assert_eq!(w.cache.plan(0, 1, &near, false, true), Plan::default());
        // The far layer redraws its (now no) static casters, saves them and
        // draws the mover over them.
        let save = Plan {
            draw_static: true,
            save: true,
            restore: false,
            overlay: true,
        };
        assert_eq!(w.cache.plan(1, 2, &far, true, true), save);
        // Still moving: the far layer restores and overlays.
        let restore = Plan {
            draw_static: false,
            save: false,
            restore: true,
            overlay: true,
        };
        for _ in 0..3 {
            w.nudge(0.1);
            let split = w.frame();
            assert!(split.dirty.is_empty());
            assert_eq!(w.cache.plan(0, 1, &near, false, true), Plan::default());
            assert_eq!(w.cache.plan(1, 2, &far, true, true), restore);
        }
        // It stops: dynamic for QUIET_FRAMES - 1 more frames, then static
        // and dirty where it stands.
        for f in 1..=QUIET_FRAMES {
            let split = w.frame();
            let still = f == QUIET_FRAMES;
            assert_eq!(split.dynamic[1], !still, "frame {f}");
            assert_eq!(split.dirty.len(), still as usize);
            assert_eq!(w.cache.plan(0, 1, &near, false, true), Plan::default());
            let plan = w.cache.plan(1, 2, &far, !still, true);
            if still {
                assert_eq!(
                    plan,
                    Plan {
                        draw_static: true,
                        ..Plan::default()
                    }
                );
            } else {
                assert_eq!(plan, restore);
            }
        }
        // Without the mover the far layer's cache comes back once.
        let split = w.frame();
        assert!(split.dirty.is_empty());
        assert_eq!(w.cache.plan(1, 2, &far, false, true), Plan::default());
    }

    /// A view that moves every frame never pays for the cache: one pass of
    /// every caster, nothing saved; a layer without static casters is not
    /// cached either.
    #[test]
    fn a_moving_view_draws_in_one_pass() {
        let mut w = World::new();
        let near = box_frustum(0.0);
        for key in 0..4 {
            w.frame();
            assert_eq!(w.cache.plan(0, key, &near, true, true), Plan::ALL);
        }
        w.frame();
        assert!(w.cache.plan(0, 3, &near, true, true).save, "still: kept");
        w.frame();
        let plan = w.cache.plan(0, 3, &near, true, false);
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
}
