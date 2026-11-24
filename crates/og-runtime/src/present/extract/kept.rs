//! The last frame's models, kept (`frame/models.rs`): an og.Model row
//! whose inputs are the last frame's, under the same view, draws what it
//! drew then, copied, instead of being built again (its level picked, its
//! parts walked, its meshes asked whether the cluster path draws them).
//! So a frame where little moves builds only what moved.
//!
//! A row is copied when, bit for bit, its entity, model, look (so not
//! while it fades in), bias,
//! sway, the parts it draws from and its drawn world matrix are the last
//! frame's, it is replaced by a proxy or not as then, its level was
//! settled (the hysteresis kept the level it found) and it was neither
//! posed nor a crowd. The view is everything else the rows are built
//! from: the camera, the view's height, the settings' bias, the level
//! view, baked lighting and the cluster path, with the counts of the
//! changes to the meshes, materials, lightmaps and HLOD places; a model
//! loaded or reloaded forgets the whole list ([`KeptModels::forget`]).
//! The HLOD proxies are copied when every row and the HLOD places are
//! as they were and its last walk changed no choice.
//!
//! Each frame's list is numbered, and a frame that copied every model
//! promises the renderer that its models' instances are the last list's
//! (`og_render::KeptInstances`, [`KeptModels::promise`]), so the renderer
//! does not compare them again.

use super::*;
use crate::present::hlod::PlaceIn;

/// What the rows of a frame's models are built from besides their own
/// inputs.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct ModelsView {
    /// The camera, without its exposure (it picks no level).
    pub camera: og_render::Camera,
    pub view_height: u32,
    pub bias: u32,
    pub debug: bool,
    pub baked: bool,
    pub meshlets: bool,
    pub meshlet_serial: u64,
    pub lightmaps: u64,
    pub hlod_changes: u64,
}

impl ModelsView {
    pub(crate) fn of(
        rs: &RenderState,
        view_height: f32,
        bias: f32,
        debug: bool,
        baked: bool,
    ) -> Self {
        let camera = og_render::Camera {
            exposure: Default::default(),
            ..rs.scene.frame.camera
        };
        ModelsView {
            camera,
            view_height: view_height.to_bits(),
            bias: bias.to_bits(),
            debug,
            baked,
            meshlets: rs.scene.objects.meshlets,
            meshlet_serial: rs.scene.meshlet_serial(),
            lightmaps: rs.baked.serial,
            hlod_changes: rs.hlod.changes,
        }
    }
}

/// One og.Model row's inputs, compared bit for bit.
#[derive(Clone, Copy)]
pub(crate) struct RowIn {
    pub entity: u64,
    pub asset: u64,
    /// The parts it draws from (its lightmap's or its model's), by
    /// address: the same while the view's counts hold.
    pub source: usize,
    pub variant: u64,
    pub look: crate::present::fx::ModelLook,
    pub bias: f32,
    pub sway: f32,
    pub matrix: Mat4,
    pub posed: bool,
    pub crowd: bool,
}

impl RowIn {
    fn same(&self, o: &RowIn) -> bool {
        let look = |r: &RowIn| {
            let f = &r.look.fx;
            let c = |v: Vec3| v.to_array().map(f32::to_bits);
            (
                [f.flash.to_bits(), r.bias.to_bits(), r.sway.to_bits()],
                c(f.flash_color),
                c(f.tint),
                (f.outline, r.look.cast, r.look.receive, f.reveal.to_bits()),
            )
        };
        let m = |r: &RowIn| r.matrix.to_cols_array().map(f32::to_bits);
        (self.entity, self.asset, self.source, self.variant)
            == (o.entity, o.asset, o.source, o.variant)
            && (self.posed, self.crowd) == (o.posed, o.crowd)
            && look(self) == look(o)
            && m(self) == m(o)
    }
}

/// Where a row's output starts in a frame's lists.
#[derive(Clone, Copy, Default)]
struct At {
    instances: usize,
    keys: usize,
    impostors: usize,
}

/// How a row was drawn.
#[derive(Clone, Copy)]
struct Drew {
    replaced: bool,
    /// The level it drew (the next frame's hysteresis).
    level: u8,
    /// Whether the next frame may copy it: settled, not posed, no crowd.
    copyable: bool,
}

/// One frame's models: their inputs, and what they drew when `stored`.
#[derive(Default)]
struct Frame {
    /// None: nothing to compare with (forgotten, or never built).
    view: Option<ModelsView>,
    /// Whether the lists below hold the frame's output.
    stored: bool,
    rows: Vec<RowIn>,
    drew: Vec<Drew>,
    /// Per row where its output starts, then where the last ends.
    at: Vec<At>,
    /// The HLOD's inputs ([`crate::present::hlod::inputs`]) and where its
    /// proxies start and end.
    hlod: Option<(Vec<PlaceIn>, At, At)>,
    instances: Vec<og_render::Instance>,
    keys: Vec<(u64, u32)>,
    /// Per instance its level for the cluster path, `u8::MAX` for none.
    levels: Vec<u8>,
    /// Where the models' instances start in the frame's list (after a
    /// fractured model's chunks).
    start: usize,
    impostors: Vec<og_render::ImpostorDraw>,
}

/// The last frame's models, and the lists the next frame is built in.
#[derive(Default)]
pub(crate) struct KeptModels {
    last: Frame,
    next: Frame,
    /// Rows the last frame copied, and whether it copied its proxies.
    pub(crate) copied: usize,
    pub(crate) copied_hlod: bool,
    /// The number of the last frame's list (one more each frame).
    pub(crate) list: u64,
    /// The last frame's promise: the list before it, and how many of its
    /// first instances this one repeats (every model copied, none before).
    repeats: Option<(u64, usize)>,
    /// The render origin the last list was moved to, as bits.
    origin: Option<[u32; 3]>,
}

/// A frame's models being built against the last frame's.
pub(crate) struct Build {
    kept: KeptModels,
    view: ModelsView,
    /// Whether the last frame is there to copy from.
    reusable: bool,
    /// Per row of this frame, whether its inputs are the last frame's
    /// (copied only when `reusable`).
    same: Vec<bool>,
    copied: usize,
    fresh: usize,
    hlod_copied: bool,
}

impl KeptModels {
    /// Nothing kept is copied again: a model was loaded or reloaded.
    pub(crate) fn forget(&mut self) {
        self.last.view = None;
        self.last.stored = false;
    }

    /// The renderer's promise for this frame's list, moved to render origin
    /// `origin`, whose previous transforms are list `previous`'s
    /// transforms (`crate::present::fx::motion`; 0: none): its models'
    /// instances are the last list's when every model was copied and the
    /// origin held.
    pub(crate) fn promise(&mut self, origin: Vec3, previous: u64) -> og_render::KeptInstances {
        let origin = origin.to_array().map(f32::to_bits);
        let moved = self.origin.replace(origin) != Some(origin);
        let list = self.list;
        match self.repeats.filter(|_| !moved) {
            Some((from, count)) => og_render::KeptInstances {
                list,
                from,
                count,
                still: previous == from,
            },
            None => og_render::KeptInstances {
                list,
                ..Default::default()
            },
        }
    }

    /// Starts this frame's models under `view` with `rows`.
    pub(crate) fn begin(mut self, view: ModelsView, rows: Vec<RowIn>) -> Build {
        let reusable = self.last.stored && self.last.view == Some(view);
        let same = rows
            .iter()
            .enumerate()
            .map(|(i, r)| self.last.rows.get(i).is_some_and(|l| l.same(r)))
            .collect();
        let next = &mut self.next;
        next.rows = rows;
        next.drew.clear();
        next.at.clear();
        next.hlod = None;
        next.instances.clear();
        next.keys.clear();
        next.levels.clear();
        next.impostors.clear();
        Build {
            kept: self,
            view,
            reusable,
            same,
            copied: 0,
            fresh: 0,
            hlod_copied: false,
        }
    }
}

impl Build {
    pub(crate) fn rows(&self) -> &[RowIn] {
        &self.kept.next.rows
    }

    /// Copies the last frame's proxies when every row and the HLOD's
    /// `inputs` are as they were and its last walk changed no choice
    /// (`settled`); true when it did.
    pub(crate) fn copy_hlod(
        &mut self,
        scene: &mut og_render::RenderScene,
        keys: &mut Vec<(u64, u32)>,
        inputs: &[PlaceIn],
        settled: bool,
    ) -> bool {
        let last = &self.kept.last;
        let copy = self.reusable
            && settled
            && self.same.len() == last.rows.len()
            && self.same.iter().all(|&s| s)
            && last.hlod.as_ref().is_some_and(|h| h.0 == inputs);
        if let (true, Some((_, from, to))) = (copy, &last.hlod) {
            append(last, scene, keys, *from, *to);
        }
        self.hlod_copied = copy;
        copy
    }

    /// The HLOD's proxies of this frame were drawn between `from` and now.
    pub(crate) fn hlod_drawn(
        &mut self,
        inputs: Vec<PlaceIn>,
        from: (usize, usize),
        scene: &og_render::RenderScene,
        keys: &[(u64, u32)],
    ) {
        let at = |instances, keys| At {
            instances,
            keys,
            impostors: 0,
        };
        let to = at(scene.objects.instances.len(), keys.len());
        self.kept.next.hlod = Some((inputs, at(from.0, from.1), to));
        self.kept.next.start = from.0;
    }

    /// Where the output of row `i` starts.
    pub(crate) fn start(&mut self, scene: &og_render::RenderScene, keys: &[(u64, u32)]) {
        self.kept.next.at.push(At {
            instances: scene.objects.instances.len(),
            keys: keys.len(),
            impostors: scene.objects.impostors.len(),
        });
    }

    /// Copies row `i`'s output from the last frame when its inputs and its
    /// proxy's are the last frame's and it may be copied: the level it
    /// drew (none for a replaced row).
    pub(crate) fn copy_row(
        &mut self,
        i: usize,
        replaced: bool,
        scene: &mut og_render::RenderScene,
        keys: &mut Vec<(u64, u32)>,
    ) -> Option<Option<u8>> {
        let last = &self.kept.last;
        if !self.reusable || !self.same[i] {
            return None;
        }
        let d = last.drew[i];
        if d.replaced != replaced || !d.copyable {
            return None;
        }
        append(last, scene, keys, last.at[i], last.at[i + 1]);
        self.kept.next.drew.push(d);
        self.copied += 1;
        Some((!replaced).then_some(d.level))
    }

    /// Row `i` was built: replaced by a proxy or drawn at `level`;
    /// `copyable` when the next frame may copy it.
    pub(crate) fn built(&mut self, replaced: bool, level: u8, copyable: bool) {
        self.kept.next.drew.push(Drew {
            replaced,
            level,
            copyable,
        });
        self.fresh += 1;
    }

    /// Ends the frame: keeps what it drew for the next. A frame that
    /// copied everything keeps the last frame's lists as they are; one
    /// whose view moved since the last frame, or where most rows changed,
    /// keeps only their inputs (the next frame would copy little of it).
    pub(crate) fn finish(
        self,
        scene: &og_render::RenderScene,
        keys: &[(u64, u32)],
        fractured: bool,
    ) -> KeptModels {
        let Build {
            mut kept,
            view,
            reusable,
            same,
            copied,
            fresh,
            hlod_copied,
        } = self;
        kept.copied = copied;
        kept.copied_hlod = hlod_copied;
        kept.list += 1;
        let next = &mut kept.next;
        next.at.push(At {
            instances: scene.objects.instances.len(),
            keys: keys.len(),
            impostors: scene.objects.impostors.len(),
        });
        let hlod_same = hlod_copied || (kept.last.hlod.is_none() && next.hlod.is_none());
        let whole =
            reusable && fresh == 0 && !fractured && hlod_same && same.len() == kept.last.rows.len();
        // Every model copied, from the start of both lists: the models'
        // instances are the last list's.
        kept.repeats = (whole && next.start == 0 && kept.last.start == 0)
            .then(|| (kept.list - 1, scene.objects.instances.len()));
        if whole {
            return kept;
        }
        let matched = same.iter().filter(|&&s| s).count();
        let still = kept.last.view == Some(view);
        next.view = Some(view);
        next.stored = still && 2 * matched >= same.len();
        if next.stored {
            let objects = &scene.objects;
            next.instances.extend_from_slice(&objects.instances);
            next.keys.extend_from_slice(keys);
            next.levels.extend_from_slice(&objects.instance_levels);
            next.levels.resize(objects.instances.len(), u8::MAX);
            next.impostors.extend_from_slice(&objects.impostors);
        }
        std::mem::swap(&mut kept.last, &mut kept.next);
        kept
    }
}

/// Appends `last`'s output between `from` and `to` to this frame's.
fn append(
    last: &Frame,
    scene: &mut og_render::RenderScene,
    keys: &mut Vec<(u64, u32)>,
    from: At,
    to: At,
) {
    let objects = &mut scene.objects;
    let at = objects.instances.len();
    let range = from.instances..to.instances;
    objects
        .instances
        .extend_from_slice(&last.instances[range.clone()]);
    // The cluster path's levels, as the builder leaves them: up to the
    // last instance that has one, `u8::MAX` before it for those without.
    let levels = &last.levels[range];
    if let Some(j) = levels.iter().rposition(|&l| l != u8::MAX) {
        objects.instance_levels.resize(at, u8::MAX);
        objects.instance_levels.extend_from_slice(&levels[..=j]);
    }
    keys.extend_from_slice(&last.keys[from.keys..to.keys]);
    objects
        .impostors
        .extend_from_slice(&last.impostors[from.impostors..to.impostors]);
}
