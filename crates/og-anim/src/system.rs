//! The `og.anim` system: one evaluation per tick of every skeleton, its
//! player, graph, root motion, blendshapes and IK chains; the joint palettes
//! it leaves for the renderer; and the state block that saves it all.

use crate::asset::{ClipAsset, NONE, SkeletonAsset, bind};
use crate::codec::{Reader, Writer};
use crate::graph::{GraphAsset, Interrupt, LayerMode, Motion};
use crate::ik;
use crate::pose::{Blend, Fired, Leaf, Pose, RootMotion, stepped};
use crate::retarget::FootPlants;
use crate::schema::*;
use crate::xform::{Trs, unit};
use og_core::glam::{Mat4, Quat, Vec3};
use og_core::{Error, ErrorKind, Result, Str64};
use og_ecs::{Entity, QueryDesc, StateBlock, SystemContext, SystemDesc, World};
use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, MutexGuard};

/// The scheduled system's and the state block's name.
pub const SYSTEM_NAME: &str = "og.anim";

/// The renderer side of the palette contract (`README.md` §Palette). The
/// host extracts every animated entity's output into its render scene
/// through this; og-render's skinning input implements it.
pub trait PaletteSink {
    /// The joint palette of `entity`'s skin `skin`: per joint, model matrix
    /// times inverse bind matrix, in the skeleton's model space. A skinned
    /// vertex is drawn at `matrixWorld(entity) · Σ wᵢ · joints[jᵢ] · v`; the
    /// skinned mesh node's own transform is not applied (glTF).
    fn joint_palette(&mut self, entity: Entity, skin: u32, joints: &[Mat4]);
    /// The blendshape weights of `entity`'s mesh `mesh` (glTF mesh index),
    /// in target order; entities without `og.Blendshapes` report none.
    fn morph_weights(&mut self, entity: Entity, mesh: u32, weights: &[f32; BLENDSHAPES]) {
        let _ = (entity, mesh, weights);
    }
}

/// A palette as a plain list of matrices (the stand-in until og-render's
/// `JointPalette` lands).
pub type JointPalette = Vec<Mat4>;

/// What the system produced for one entity in the last tick.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimOutput {
    pub skin: u32,
    pub palette: JointPalette,
    /// The mesh the weights are for ([`NONE`] without `og.Blendshapes`).
    pub mesh: u32,
    pub weights: [f32; BLENDSHAPES],
    /// Per skeleton node: whether it is drawn (it and every ancestor
    /// visible, [`SkeletonAsset::drawn`]). The renderer skips a node's
    /// primitives where this is false, so a hidden node costs no draw.
    pub drawn: Vec<bool>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct PlayerState {
    clip: u64,
    time: f32,
    /// The `time` the system last wrote (a different value is a seek).
    written: f32,
    speed: f32,
    looping: bool,
    prev_clip: u64,
    prev_time: f32,
    prev_speed: f32,
    prev_looping: bool,
    fade: f32,
    fade_duration: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Phase {
    cycles: u32,
    frac: f32,
}

impl Phase {
    fn total(self) -> f64 {
        self.cycles as f64 + self.frac as f64
    }
    /// Advances by `d` cycles; returns the wraps.
    fn advance(&mut self, d: f32) -> i32 {
        let f = self.frac + d;
        let k = f.floor();
        self.frac = f - k;
        if self.frac >= 1.0 {
            self.frac = 0.0;
        }
        let k = k as i32;
        self.cycles = self.cycles.saturating_add_signed(k);
        k
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct LayerState {
    state: u32,
    phase: Phase,
    time: f32,
    prev: u32,
    prev_phase: Phase,
    fade: f32,
    fade_duration: f32,
    /// What may interrupt the fade in progress (the transition's
    /// `interrupt`); `Any` when none is.
    guard: Interrupt,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct GraphState {
    asset: u64,
    layers: Vec<LayerState>,
}

#[derive(Clone, Debug, PartialEq)]
struct EntityState {
    skeleton: u64,
    skin: u32,
    mesh: u32,
    /// The final pose of the last tick (after IK).
    pose: Vec<Trs>,
    /// The final blendshape weights of the last tick.
    weights: [f32; BLENDSHAPES],
    /// The final per-node visible flags of the last tick.
    visible: Vec<bool>,
    player: PlayerState,
    graph: GraphState,
}

/// Channel-to-node bindings by (skeleton, clip), derived from the assets.
type Bindings = BTreeMap<(u64, u64), Arc<Vec<u32>>>;

/// Foot plants by (skeleton, clip, foot node), derived from the assets.
type Plants = BTreeMap<(u64, u64, u32), Arc<FootPlants>>;

/// What a clip that poses the legs played this tick: (clip, the time it
/// was sampled at, its share of the pose). The base layer's and the
/// player's clips, for foot locks.
type Stance = Vec<(u64, f32, f32)>;

/// A foot lock between ticks (`og.FootLock`); positions are in the frame
/// the skeleton entity's Transform is written in (model space without one).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Lock {
    #[default]
    Free,
    /// Held at this point.
    Held(Vec3),
    /// Rejoining the animation from this point, this many seconds in.
    Releasing(Vec3, f32),
}

/// One `og.FootLock` entity's state, keyed by that entity.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct LockState {
    /// The skeleton entity it held (a lock moved to another restarts).
    skeleton: u64,
    lock: Lock,
}

/// What the state block holds.
#[derive(Clone, Debug, Default, PartialEq)]
struct Saved {
    states: BTreeMap<u64, EntityState>,
    locks: BTreeMap<u64, LockState>,
}

/// The registered assets and the world state.
#[derive(Default)]
pub(crate) struct Shared {
    skeletons: BTreeMap<u64, Arc<SkeletonAsset>>,
    clips: BTreeMap<u64, Arc<ClipAsset>>,
    graphs: BTreeMap<u64, Arc<GraphAsset>>,
    /// A cache filled by the lanes as they meet (skeleton, clip) pairs.
    bindings: Mutex<Bindings>,
    /// A cache filled as foot locks meet (skeleton, clip, foot) triples.
    plants: Mutex<Plants>,
    states: BTreeMap<u64, EntityState>,
    locks: BTreeMap<u64, LockState>,
    outputs: BTreeMap<u64, AnimOutput>,
}

fn lock(shared: &Mutex<Shared>) -> MutexGuard<'_, Shared> {
    shared.lock().unwrap_or_else(|p| p.into_inner())
}

fn lock_bindings(b: &Mutex<Bindings>) -> MutexGuard<'_, Bindings> {
    b.lock().unwrap_or_else(|p| p.into_inner())
}

fn lock_plants(p: &Mutex<Plants>) -> MutexGuard<'_, Plants> {
    p.lock().unwrap_or_else(|p| p.into_inner())
}

impl Shared {
    fn binding(&self, skeleton: u64, clip: u64) -> Option<Arc<Vec<u32>>> {
        if let Some(b) = lock_bindings(&self.bindings).get(&(skeleton, clip)) {
            return Some(b.clone());
        }
        // Computed outside the lock: two lanes may both bind a pair, to the
        // same value.
        let b = Arc::new(bind(self.skeletons.get(&skeleton)?, self.clips.get(&clip)?));
        Some(
            lock_bindings(&self.bindings)
                .entry((skeleton, clip))
                .or_insert(b)
                .clone(),
        )
    }

    fn plants(&self, skeleton: u64, clip: u64, foot: u32) -> Option<Arc<FootPlants>> {
        let key = (skeleton, clip, foot);
        if let Some(p) = lock_plants(&self.plants).get(&key) {
            return Some(p.clone());
        }
        let p = Arc::new(FootPlants::new(
            self.skeletons.get(&skeleton)?,
            self.clips.get(&clip)?,
            foot,
        ));
        Some(lock_plants(&self.plants).entry(key).or_insert(p).clone())
    }

    /// Forgets what was derived from skeleton `skeleton` or clip `clip`.
    fn forget(&self, skeleton: Option<u64>, clip: Option<u64>) {
        lock_bindings(&self.bindings).retain(|k, _| Some(k.0) != skeleton && Some(k.1) != clip);
        lock_plants(&self.plants).retain(|k, _| Some(k.0) != skeleton && Some(k.1) != clip);
    }

    fn recompute_outputs(&mut self) {
        self.outputs.clear();
        for (&e, st) in &self.states {
            if let Some(skel) = self.skeletons.get(&st.skeleton)
                && skel.nodes.len() == st.pose.len()
            {
                let mut palette = Vec::new();
                skel.palette(st.skin, &st.pose, &mut palette);
                self.outputs.insert(
                    e,
                    AnimOutput {
                        skin: st.skin,
                        palette,
                        mesh: st.mesh,
                        weights: st.weights,
                        drawn: skel.drawn(&st.visible),
                    },
                );
            }
        }
    }
}

/// The handle to an installed animation system: asset registration and the
/// per-tick output. Cloning shares the same system.
#[derive(Clone)]
pub struct AnimationSystem {
    shared: Arc<Mutex<Shared>>,
}

impl std::fmt::Debug for AnimationSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnimationSystem").finish_non_exhaustive()
    }
}

fn invalid(reason: String) -> Error {
    Error::new(ErrorKind::Invalid, reason)
}

/// Registers the animation types and schedules the `og.anim` system (with
/// its state block) at this point of the schedule: after the systems that
/// set players and parameters, before the ones that read the moved
/// transforms (and before physics, which then sees root motion as a move).
pub fn install(world: &mut World) -> Result<AnimationSystem> {
    world.register::<Transform>()?;
    world.register::<Skeleton>()?;
    world.register::<AnimationPlayer>()?;
    world.register::<AnimationGraph>()?;
    world.register::<Blendshapes>()?;
    world.register::<IkChain>()?;
    world.register::<LookAt>()?;
    world.register::<JointLimit>()?;
    world.register::<FootLock>()?;
    world.register::<AnimationEvent>()?;
    let shared = Arc::new(Mutex::new(Shared::default()));
    let desc = SystemDesc::new(SYSTEM_NAME)
        .write::<Skeleton>()
        .write::<AnimationPlayer>()
        .write::<AnimationGraph>()
        .write::<Blendshapes>()
        .write::<IkChain>()
        .write::<LookAt>()
        .read::<JointLimit>()
        .write::<FootLock>()
        .write::<Transform>()
        .write::<AnimationEvent>()
        .query(QueryDesc::new().read::<Skeleton>())
        .query(
            QueryDesc::new()
                .read::<Skeleton>()
                .read::<AnimationPlayer>(),
        )
        .query(QueryDesc::new().read::<Skeleton>().read::<AnimationGraph>())
        .query(QueryDesc::new().read::<Skeleton>().read::<Blendshapes>())
        .query(QueryDesc::new().read::<Skeleton>().read::<Transform>())
        .query(QueryDesc::new().read::<IkChain>())
        .query(QueryDesc::new().read::<LookAt>())
        .query(QueryDesc::new().read::<JointLimit>())
        .query(QueryDesc::new().read::<FootLock>());
    let s = shared.clone();
    world.add_system(desc, move |ctx| run(ctx, &s))?;
    world.add_state_block(SYSTEM_NAME, Box::new(AnimState(shared.clone())))?;
    Ok(AnimationSystem { shared })
}

impl AnimationSystem {
    /// Registers (or replaces) skeleton asset `handle` (not 0).
    pub fn add_skeleton(&self, handle: u64, asset: SkeletonAsset) -> Result<()> {
        if handle == 0 {
            return Err(invalid("asset handle 0 means none".into()));
        }
        asset.validate()?;
        let mut sh = lock(&self.shared);
        sh.skeletons.insert(handle, Arc::new(asset));
        sh.forget(Some(handle), None);
        Ok(())
    }

    /// Registers (or replaces) clip asset `handle` (not 0).
    pub fn add_clip(&self, handle: u64, asset: ClipAsset) -> Result<()> {
        if handle == 0 {
            return Err(invalid("asset handle 0 means none".into()));
        }
        asset.validate()?;
        let mut sh = lock(&self.shared);
        sh.clips.insert(handle, Arc::new(asset));
        sh.forget(None, Some(handle));
        Ok(())
    }

    /// Registers (or replaces) graph asset `handle` (not 0); every clip it
    /// plays must be registered.
    pub fn add_graph(&self, handle: u64, asset: GraphAsset) -> Result<()> {
        if handle == 0 {
            return Err(invalid("asset handle 0 means none".into()));
        }
        let mut sh = lock(&self.shared);
        if let Some(c) = asset
            .clips()
            .into_iter()
            .find(|c| !sh.clips.contains_key(c))
        {
            return Err(invalid(format!(
                "the graph plays clip {c}, which is not registered"
            )));
        }
        sh.graphs.insert(handle, Arc::new(asset));
        Ok(())
    }

    pub fn remove_skeleton(&self, handle: u64) {
        let mut sh = lock(&self.shared);
        sh.skeletons.remove(&handle);
        sh.forget(Some(handle), None);
    }

    pub fn remove_clip(&self, handle: u64) {
        let mut sh = lock(&self.shared);
        sh.clips.remove(&handle);
        sh.forget(None, Some(handle));
    }

    pub fn remove_graph(&self, handle: u64) {
        lock(&self.shared).graphs.remove(&handle);
    }

    pub fn skeleton(&self, handle: u64) -> Option<Arc<SkeletonAsset>> {
        lock(&self.shared).skeletons.get(&handle).cloned()
    }

    pub fn clip(&self, handle: u64) -> Option<Arc<ClipAsset>> {
        lock(&self.shared).clips.get(&handle).cloned()
    }

    pub fn graph(&self, handle: u64) -> Option<Arc<GraphAsset>> {
        lock(&self.shared).graphs.get(&handle).cloned()
    }

    /// The final local pose of `entity` in the last tick (every node, in
    /// the skeleton's node order).
    pub fn pose(&self, entity: Entity) -> Option<Vec<Trs>> {
        lock(&self.shared)
            .states
            .get(&entity.to_bits())
            .map(|s| s.pose.clone())
    }

    /// The final per-node visible flags of `entity` in the last tick (the
    /// pose's own; [`AnimOutput::drawn`] folds in the ancestors').
    pub fn visible(&self, entity: Entity) -> Option<Vec<bool>> {
        lock(&self.shared)
            .states
            .get(&entity.to_bits())
            .map(|s| s.visible.clone())
    }

    /// What the last tick produced for `entity`.
    pub fn output(&self, entity: Entity) -> Option<AnimOutput> {
        lock(&self.shared).outputs.get(&entity.to_bits()).cloned()
    }

    /// Hands every animated entity's palette (and blendshape weights) to
    /// `sink`, in entity order.
    pub fn write_outputs(&self, sink: &mut dyn PaletteSink) {
        let sh = lock(&self.shared);
        for (&e, out) in &sh.outputs {
            let e = Entity::from_bits(e);
            sink.joint_palette(e, out.skin, &out.palette);
            if out.mesh != NONE {
                sink.morph_weights(e, out.mesh, &out.weights);
            }
        }
    }
}

// ---- the state block -------------------------------------------------------

struct AnimState(Arc<Mutex<Shared>>);

fn write_phase(w: &mut Writer, p: Phase) {
    w.u32(p.cycles);
    w.f32(p.frac);
}

fn read_phase(r: &mut Reader) -> Result<Phase> {
    Ok(Phase {
        cycles: r.u32()?,
        frac: r.f32()?,
    })
}

fn write_vec3(w: &mut Writer, v: Vec3) {
    for x in v.to_array() {
        w.f32(x);
    }
}

fn read_vec3(r: &mut Reader) -> Result<Vec3> {
    Ok(Vec3::new(r.f32()?, r.f32()?, r.f32()?))
}

/// Sections after the entities, each a tag and its contents, written only
/// when they hold something (so a world without them saves, and hashes, as
/// before they existed).
const SECTION_LOCKS: u32 = 1;
/// Graph layers whose fade a transition's `interrupt` guards.
const SECTION_GUARDS: u32 = 2;
/// Skeleton entities with a hidden node: the hidden nodes of each.
const SECTION_HIDDEN: u32 = 3;

fn guard_code(g: Interrupt) -> u8 {
    match g {
        Interrupt::Any => 0,
        Interrupt::None => 1,
        Interrupt::Source => 2,
        Interrupt::Next => 3,
    }
}

fn encode(
    states: &BTreeMap<u64, EntityState>,
    locks: &BTreeMap<u64, LockState>,
    out: &mut Vec<u8>,
) {
    let mut w = Writer::new(out);
    w.u32(1); // format
    w.u32(states.len() as u32);
    for (&e, s) in states {
        w.u64(e);
        w.u64(s.skeleton);
        w.u32(s.skin);
        w.u32(s.mesh);
        w.u32(s.pose.len() as u32);
        for t in &s.pose {
            for v in t.translation.to_array() {
                w.f32(v);
            }
            for v in t.rotation.to_array() {
                w.f32(v);
            }
            for v in t.scale.to_array() {
                w.f32(v);
            }
        }
        for v in s.weights {
            w.f32(v);
        }
        let p = &s.player;
        w.u64(p.clip);
        w.f32(p.time);
        w.f32(p.written);
        w.f32(p.speed);
        w.u8(p.looping as u8);
        w.u64(p.prev_clip);
        w.f32(p.prev_time);
        w.f32(p.prev_speed);
        w.u8(p.prev_looping as u8);
        w.f32(p.fade);
        w.f32(p.fade_duration);
        w.u64(s.graph.asset);
        w.u32(s.graph.layers.len() as u32);
        for l in &s.graph.layers {
            w.u32(l.state);
            write_phase(&mut w, l.phase);
            w.f32(l.time);
            w.u32(l.prev);
            write_phase(&mut w, l.prev_phase);
            w.f32(l.fade);
            w.f32(l.fade_duration);
        }
    }
    if !locks.is_empty() {
        w.u32(SECTION_LOCKS);
        w.u32(locks.len() as u32);
        for (&e, l) in locks {
            w.u64(e);
            w.u64(l.skeleton);
            let (mode, at, t) = match l.lock {
                Lock::Free => (0, Vec3::ZERO, 0.0),
                Lock::Held(at) => (1, at, 0.0),
                Lock::Releasing(from, t) => (2, from, t),
            };
            w.u8(mode);
            write_vec3(&mut w, at);
            w.f32(t);
        }
    }
    let guards: Vec<(u64, u32, u8)> = states
        .iter()
        .flat_map(|(&e, s)| {
            s.graph
                .layers
                .iter()
                .enumerate()
                .filter(|(_, l)| l.guard != Interrupt::Any)
                .map(move |(i, l)| (e, i as u32, guard_code(l.guard)))
        })
        .collect();
    if !guards.is_empty() {
        w.u32(SECTION_GUARDS);
        w.u32(guards.len() as u32);
        for (e, layer, g) in guards {
            w.u64(e);
            w.u32(layer);
            w.u8(g);
        }
    }
    let hidden: Vec<(u64, Vec<u32>)> = states
        .iter()
        .filter_map(|(&e, s)| {
            let h: Vec<u32> = (0..s.visible.len() as u32)
                .filter(|&i| !s.visible[i as usize])
                .collect();
            (!h.is_empty()).then_some((e, h))
        })
        .collect();
    if !hidden.is_empty() {
        w.u32(SECTION_HIDDEN);
        w.u32(hidden.len() as u32);
        for (e, nodes) in hidden {
            w.u64(e);
            w.u32(nodes.len() as u32);
            for n in nodes {
                w.u32(n);
            }
        }
    }
}

fn decode(bytes: &[u8]) -> Result<Saved> {
    let mut r = Reader::new(bytes);
    let mut saved = Saved::default();
    if bytes.is_empty() {
        return Ok(saved);
    }
    let format = r.u32()?;
    if format != 1 {
        return Err(Error::new(
            ErrorKind::Parse,
            format!("og.anim state format {format}; this engine reads 1"),
        ));
    }
    let n = r.count(8)?;
    let mut last = None;
    for _ in 0..n {
        let e = r.u64()?;
        if last.is_some_and(|l| l >= e) {
            return Err(Error::new(
                ErrorKind::Parse,
                "og.anim state entities are not in order",
            ));
        }
        last = Some(e);
        let skeleton = r.u64()?;
        let skin = r.u32()?;
        let mesh = r.u32()?;
        let nodes = r.count(40)?;
        let mut pose = Vec::with_capacity(nodes);
        for _ in 0..nodes {
            let mut f = [0f32; 10];
            for v in f.iter_mut() {
                *v = r.f32()?;
            }
            pose.push(Trs::new(
                Vec3::new(f[0], f[1], f[2]),
                Quat::from_xyzw(f[3], f[4], f[5], f[6]),
                Vec3::new(f[7], f[8], f[9]),
            ));
        }
        let mut weights = [0f32; BLENDSHAPES];
        for v in weights.iter_mut() {
            *v = r.f32()?;
        }
        let player = PlayerState {
            clip: r.u64()?,
            time: r.f32()?,
            written: r.f32()?,
            speed: r.f32()?,
            looping: r.u8()? != 0,
            prev_clip: r.u64()?,
            prev_time: r.f32()?,
            prev_speed: r.f32()?,
            prev_looping: r.u8()? != 0,
            fade: r.f32()?,
            fade_duration: r.f32()?,
        };
        let asset = r.u64()?;
        let layers_n = r.count(32)?;
        let mut layers = Vec::with_capacity(layers_n);
        for _ in 0..layers_n {
            layers.push(LayerState {
                state: r.u32()?,
                phase: read_phase(&mut r)?,
                time: r.f32()?,
                prev: r.u32()?,
                prev_phase: read_phase(&mut r)?,
                fade: r.f32()?,
                fade_duration: r.f32()?,
                guard: Interrupt::Any,
            });
        }
        let visible = vec![true; pose.len()];
        saved.states.insert(
            e,
            EntityState {
                skeleton,
                skin,
                mesh,
                pose,
                weights,
                visible,
                player,
                graph: GraphState { asset, layers },
            },
        );
    }
    let mut last_tag = 0;
    while !r.done() {
        let tag = r.u32()?;
        if tag <= last_tag {
            return Err(Error::new(
                ErrorKind::Parse,
                format!("og.anim state section {tag} after {last_tag}"),
            ));
        }
        last_tag = tag;
        match tag {
            SECTION_LOCKS => {
                let n = r.count(33)?;
                for _ in 0..n {
                    let e = r.u64()?;
                    let skeleton = r.u64()?;
                    let mode = r.u8()?;
                    let at = read_vec3(&mut r)?;
                    let t = r.f32()?;
                    let lock = match mode {
                        0 => Lock::Free,
                        1 => Lock::Held(at),
                        2 => Lock::Releasing(at, t),
                        m => {
                            return Err(Error::new(
                                ErrorKind::Parse,
                                format!("og.anim foot lock mode {m}"),
                            ));
                        }
                    };
                    saved.locks.insert(e, LockState { skeleton, lock });
                }
            }
            SECTION_GUARDS => {
                let n = r.count(13)?;
                for _ in 0..n {
                    let (e, layer, g) = (r.u64()?, r.u32()?, r.u8()?);
                    let guard = match g {
                        1 => Interrupt::None,
                        2 => Interrupt::Source,
                        3 => Interrupt::Next,
                        g => {
                            return Err(Error::new(
                                ErrorKind::Parse,
                                format!("og.anim interrupt guard {g}"),
                            ));
                        }
                    };
                    let l = saved
                        .states
                        .get_mut(&e)
                        .and_then(|s| s.graph.layers.get_mut(layer as usize))
                        .ok_or_else(|| {
                            Error::new(
                                ErrorKind::Parse,
                                format!(
                                    "og.anim guard for entity {e} layer {layer}, which it lacks"
                                ),
                            )
                        })?;
                    l.guard = guard;
                }
            }
            SECTION_HIDDEN => {
                let n = r.count(12)?;
                for _ in 0..n {
                    let e = r.u64()?;
                    let count = r.count(4)?;
                    let s = saved.states.get_mut(&e).ok_or_else(|| {
                        Error::new(
                            ErrorKind::Parse,
                            format!("og.anim hidden nodes for entity {e}, which it lacks"),
                        )
                    })?;
                    for _ in 0..count {
                        let node = r.u32()?;
                        let v = s.visible.get_mut(node as usize).ok_or_else(|| {
                            Error::new(
                                ErrorKind::Parse,
                                format!("og.anim entity {e} hides node {node}, past its pose"),
                            )
                        })?;
                        *v = false;
                    }
                }
            }
            t => {
                return Err(Error::new(
                    ErrorKind::Parse,
                    format!("og.anim state section {t}; this engine reads up to {SECTION_HIDDEN}"),
                ));
            }
        }
    }
    r.finish()?;
    Ok(saved)
}

impl StateBlock for AnimState {
    fn save(&self, out: &mut Vec<u8>) {
        let sh = lock(&self.0);
        encode(&sh.states, &sh.locks, out);
    }

    fn decode(&self, bytes: &[u8]) -> Result<Box<dyn Any + Send>> {
        Ok(Box::new(decode(bytes)?))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let mut sh = lock(&self.0);
        let saved = decoded
            .map(|d| *d.downcast::<Saved>().expect("decoded by AnimState::decode"))
            .unwrap_or_default();
        sh.states = saved.states;
        sh.locks = saved.locks;
        sh.recompute_outputs();
    }
}

// ---- the tick ----------------------------------------------------------------

fn entities(ctx: &SystemContext<'_>, q: usize) -> Result<Vec<Entity>> {
    let mut out = Vec::new();
    for a in ctx.query(q)? {
        out.extend_from_slice(a.entities());
    }
    Ok(out)
}

/// Everything a tick reads about one skeleton entity, and what it writes back.
struct Work {
    entity: Entity,
    skeleton: Skeleton,
    asset: Arc<SkeletonAsset>,
    player: Option<AnimationPlayer>,
    graph: Option<AnimationGraph>,
    blend: Option<Blendshapes>,
    transform: Option<Transform>,
    state: EntityState,
    pose: Pose,
    /// The clips posing the legs this tick, for foot locks.
    stance: Stance,
}

/// One skeleton entity on its way through the tick: what was read (serially,
/// from the world), then what its lane computed from that alone.
struct Eval {
    work: Work,
    graph_asset: Option<Arc<GraphAsset>>,
    root: Option<RootMotion>,
    mesh: u32,
    /// The lane's result: the root motion delta and the events fired.
    out: Result<(Trs, Vec<Fired>)>,
}

fn missing(entity: Entity, what: &str, handle: u64) -> Error {
    Error::new(
        ErrorKind::DeadHandle,
        format!(
            "entity {}: {what} {handle} is not registered with the animation system",
            entity.to_bits()
        ),
    )
}

/// Reads one skeleton entity's components and state (serial: the world is
/// one thread's).
#[allow(clippy::too_many_arguments)]
fn gather(
    ctx: &SystemContext<'_>,
    sh: &Shared,
    e: Entity,
    players: &BTreeSet<u64>,
    graphs: &BTreeSet<u64>,
    blends: &BTreeSet<u64>,
    transforms: &BTreeSet<u64>,
) -> Result<Eval> {
    let key = e.to_bits();
    let skeleton: Skeleton = ctx.get(e)?;
    let asset = sh
        .skeletons
        .get(&skeleton.asset)
        .cloned()
        .ok_or_else(|| missing(e, "og.Skeleton.asset", skeleton.asset))?;
    let state = sh
        .states
        .get(&key)
        .filter(|s| s.skeleton == skeleton.asset && s.pose.len() == asset.nodes.len())
        .cloned()
        .unwrap_or_else(|| EntityState {
            skeleton: skeleton.asset,
            skin: skeleton.skin,
            mesh: NONE,
            pose: asset.rest_pose(),
            weights: [0.0; BLENDSHAPES],
            visible: asset.rest_visibility(),
            player: PlayerState::default(),
            graph: GraphState::default(),
        });
    let player = players
        .contains(&key)
        .then(|| ctx.get::<AnimationPlayer>(e))
        .transpose()?;
    let graph = graphs
        .contains(&key)
        .then(|| ctx.get::<AnimationGraph>(e))
        .transpose()?;
    let blend = blends
        .contains(&key)
        .then(|| ctx.get::<Blendshapes>(e))
        .transpose()?;
    let transform = transforms
        .contains(&key)
        .then(|| ctx.get::<Transform>(e))
        .transpose()?;
    let mesh = blend.map_or(NONE, |b| b.mesh);
    let root = (skeleton.root_motion != ROOT_MOTION_OFF)
        .then(|| asset.joint_node(skeleton.skin, skeleton.root_joint))
        .flatten()
        .map(|n| RootMotion::new(&asset, n, skeleton.root_motion));
    let graph_asset = match &graph {
        Some(g) => Some(
            sh.graphs
                .get(&g.asset)
                .cloned()
                .ok_or_else(|| missing(e, "og.AnimationGraph.asset", g.asset))?,
        ),
        None => None,
    };
    let pose = Pose::rest(&asset);
    Ok(Eval {
        work: Work {
            entity: e,
            skeleton,
            asset,
            player,
            graph,
            blend,
            transform,
            state,
            pose,
            stance: Vec::new(),
        },
        graph_asset,
        root,
        mesh,
        out: Ok((Trs::IDENTITY, Vec::new())),
    })
}

/// One entity's lane: its graph and player evaluated into `work.pose`, from
/// what [`gather`] read and the shared assets only.
fn evaluate(sh: &Shared, ev: &mut Eval, dt: f32) -> Result<(Trs, Vec<Fired>)> {
    let w = &mut ev.work;
    let (e, skeleton) = (w.entity, w.skeleton.asset);
    let mut events: Vec<Fired> = Vec::new();
    let mut delta = Trs::IDENTITY;
    match (w.graph.as_mut(), ev.graph_asset.as_ref()) {
        (Some(g), Some(ga)) => {
            let (gp, gd) = eval_graph(
                sh,
                e,
                skeleton,
                &w.asset,
                ga,
                g,
                &mut w.state.graph,
                ev.root.as_ref(),
                ev.mesh,
                dt,
                &mut events,
                &mut w.stance,
            )?;
            for s in &mut w.stance {
                s.2 *= g.weight.clamp(0.0, 1.0);
            }
            w.pose.lerp_to(&gp, g.weight.clamp(0.0, 1.0));
            delta = Trs::IDENTITY.lerp(&gd, g.weight.clamp(0.0, 1.0));
        }
        _ => w.state.graph = GraphState::default(),
    }
    match w.player.as_mut() {
        Some(p) => {
            if let Some((pp, pd)) = eval_player(
                sh,
                e,
                skeleton,
                &w.asset,
                p,
                &mut w.state.player,
                ev.root.as_ref(),
                ev.mesh,
                dt,
                &mut events,
                &mut w.stance,
            )? {
                let wt = p.weight.clamp(0.0, 1.0);
                w.pose.lerp_to(&pp, wt);
                delta = delta.lerp(&pd, wt);
            }
        }
        None => w.state.player = PlayerState::default(),
    }
    Ok((delta, events))
}

/// Skeletons per lane task. An entity's graph or clip evaluation is about a
/// microsecond and its palette less (20 characters: 21 and 12 us on an M4
/// Pro), while waking the pool costs tens of microseconds idle and
/// milliseconds on a loaded machine, so only crowds of 128 or more split.
const GRAIN: usize = 64;

fn run(ctx: &mut SystemContext<'_>, shared: &Mutex<Shared>) -> Result<()> {
    // Scaled time: slow motion plays slower, a hit stop holds the pose.
    let dt = ctx.scaled_dt();
    let skeletons = entities(ctx, 0)?;
    let with = |q: usize| -> Result<BTreeSet<u64>> {
        Ok(entities(ctx, q)?.iter().map(|e| e.to_bits()).collect())
    };
    let (players, graphs, blends, transforms) = (with(1)?, with(2)?, with(3)?, with(4)?);
    let chains = entities(ctx, 5)?;
    let looks = entities(ctx, 6)?;
    let joint_limits = entities(ctx, 7)?;
    let foot_locks = entities(ctx, 8)?;

    let mut sh = lock(shared);
    // Read every skeleton entity in entity order, up to the first one that
    // fails; evaluate them in lanes; then take the results in entity order,
    // so the first error and every write are the ones one thread doing
    // entity after entity would have produced.
    let mut evals: Vec<Eval> = Vec::with_capacity(skeletons.len());
    let mut unread = None;
    for &e in &skeletons {
        match gather(ctx, &sh, e, &players, &graphs, &blends, &transforms) {
            Ok(ev) => evals.push(ev),
            Err(err) => {
                unread = Some(err);
                break;
            }
        }
    }
    {
        let sh: &Shared = &sh;
        og_lanes::for_each_mut(og_lanes::Site::Anim, &mut evals, GRAIN, |ev| {
            ev.out = evaluate(sh, ev, dt)
        });
    }
    let mut work: Vec<Work> = Vec::with_capacity(evals.len());
    let mut fired: Vec<(Entity, Fired)> = Vec::new();
    for ev in evals {
        let (delta, events) = ev.out?;
        let mut w = ev.work;
        if ev.root.is_some()
            && let Some(t) = w.transform.as_mut()
        {
            t.position += t.quaternion * (t.scale * delta.translation);
            t.quaternion = unit(t.quaternion * delta.rotation);
        }
        if let Some(b) = w.blend.as_mut() {
            if b.source == BLENDSHAPES_FROM_CLIPS {
                b.set_weights(w.pose.weights);
            }
            w.pose.weights = b.weights();
        }
        w.state.skin = w.skeleton.skin;
        w.state.mesh = ev.mesh;
        for f in events {
            fired.push((w.entity, f));
        }
        work.push(w);
    }
    if let Some(err) = unread {
        return Err(err);
    }

    // Joint limits, by (skeleton entity, node); a later one for the same
    // joint replaces an earlier one.
    let mut limits: Limits = BTreeMap::new();
    for &l in &joint_limits {
        let limit: JointLimit = ctx.get(l)?;
        if let Some(w) = work.iter().find(|w| w.entity.to_bits() == limit.skeleton) {
            let node = joint(w, limit.joint, "og.JointLimit.joint")?;
            limits.insert((limit.skeleton, node), limit);
        }
    }

    // Foot locks, after root motion moved the Transforms and before the IK
    // chains, in query order. Only locks holding or releasing a foot are
    // kept.
    let mut locks: BTreeMap<u64, LockState> = BTreeMap::new();
    let mut held: Vec<(Entity, FootLock)> = Vec::with_capacity(foot_locks.len());
    for &f in &foot_locks {
        let mut fl: FootLock = ctx.get(f)?;
        fl.planted = false;
        fl.locked = false;
        if let Some(w) = work.iter_mut().find(|w| w.entity.to_bits() == fl.skeleton) {
            let prev = sh
                .locks
                .get(&f.to_bits())
                .filter(|l| l.skeleton == fl.skeleton)
                .map_or(Lock::Free, |l| l.lock);
            let lock = foot_lock(&sh, w, &mut fl, prev, dt, &limits)?;
            if lock != Lock::Free {
                locks.insert(
                    f.to_bits(),
                    LockState {
                        skeleton: fl.skeleton,
                        lock,
                    },
                );
            }
        }
        held.push((f, fl));
    }

    // IK, after every skeleton's clips and graph, in chain query order.
    let mut solved: Vec<(Entity, IkChain)> = Vec::with_capacity(chains.len());
    for &c in &chains {
        let mut chain: IkChain = ctx.get(c)?;
        chain.reached = false;
        chain.error = 0.0;
        if let Some(w) = work
            .iter_mut()
            .find(|w| w.entity.to_bits() == chain.skeleton)
        {
            solve(w, &mut chain, &limits)?;
        }
        solved.push((c, chain));
    }
    // Look-at constraints, after IK, in query order.
    let mut aimed: Vec<(Entity, LookAt)> = Vec::with_capacity(looks.len());
    for &l in &looks {
        let mut look: LookAt = ctx.get(l)?;
        look.clamped = false;
        if let Some(w) = work
            .iter_mut()
            .find(|w| w.entity.to_bits() == look.skeleton)
        {
            aim(w, &mut look, &limits)?;
        }
        aimed.push((l, look));
    }

    // The joint palettes, in lanes; then everything is written back in
    // entity order.
    let palettes = og_lanes::map(og_lanes::Site::Anim, &work, GRAIN, |w| {
        let mut palette = Vec::new();
        w.asset
            .palette(w.skeleton.skin, &w.pose.nodes, &mut palette);
        palette
    });
    let mut states = BTreeMap::new();
    let mut outputs = BTreeMap::new();
    for (mut w, palette) in work.into_iter().zip(palettes) {
        w.skeleton.joint_count = palette.len() as u32;
        ctx.set(w.entity, w.skeleton)?;
        if let Some(p) = w.player {
            ctx.set(w.entity, p)?;
        }
        if let Some(g) = w.graph {
            ctx.set(w.entity, g)?;
        }
        if let Some(b) = w.blend {
            ctx.set(w.entity, b)?;
        }
        if let Some(t) = w.transform {
            ctx.set(w.entity, t)?;
        }
        w.state.pose = w.pose.nodes;
        w.state.weights = w.pose.weights;
        w.state.visible = w.pose.visible;
        outputs.insert(
            w.entity.to_bits(),
            AnimOutput {
                skin: w.state.skin,
                palette,
                mesh: w.state.mesh,
                weights: w.state.weights,
                drawn: w.asset.drawn(&w.state.visible),
            },
        );
        states.insert(w.entity.to_bits(), w.state);
    }
    for (c, chain) in solved {
        ctx.set(c, chain)?;
    }
    for (l, look) in aimed {
        ctx.set(l, look)?;
    }
    for (f, fl) in held {
        ctx.set(f, fl)?;
    }
    for (e, f) in fired {
        ctx.send(AnimationEvent {
            entity: e.to_bits(),
            clip: f.clip,
            name: Str64::new(&f.name)?,
            time: f.time,
            weight: f.weight,
        })?;
    }
    sh.states = states;
    sh.locks = locks;
    sh.outputs = outputs;
    Ok(())
}

/// One foot lock this tick: whether the clips plant the foot, the lock's
/// next state from `prev`, and the leg bent to hold the foot there.
fn foot_lock(
    sh: &Shared,
    w: &mut Work,
    fl: &mut FootLock,
    prev: Lock,
    dt: f32,
    limits: &Limits,
) -> Result<Lock> {
    let foot = joint(w, fl.foot, "og.FootLock.foot")?;
    let Some(leg) = ik::chain(&w.asset, foot, 2) else {
        return Err(invalid(format!(
            "og.FootLock: joint {} has fewer than 2 bones above it",
            fl.foot
        )));
    };
    // Planted when the clips planting it hold at least half the stance.
    let (mut on, mut total) = (0.0f32, 0.0f32);
    for &(clip, t, weight) in &w.stance {
        if weight <= 0.0 {
            continue;
        }
        let (Some(c), Some(p)) = (sh.clips.get(&clip), sh.plants(w.skeleton.asset, clip, foot))
        else {
            continue;
        };
        total += weight;
        if p.planted_at(c, t) {
            on += weight;
        }
    }
    let planted = total > 0.0 && on >= 0.5 * total;
    let frame = w.transform.map_or(Trs::IDENTITY, |t| {
        Trs::new(t.position, t.quaternion, t.scale)
    });
    let model = w.asset.model_space(&w.pose.nodes);
    let animated = frame.transform_point(model[foot as usize].translation);
    let release = fl.release.max(0.0);
    let lifted = |at: Vec3| {
        if release > 0.0 {
            Lock::Releasing(at, 0.0)
        } else {
            Lock::Free
        }
    };
    let lock = match prev {
        Lock::Free if planted => Lock::Held(animated),
        Lock::Free => Lock::Free,
        Lock::Held(at) if !planted || (at - animated).length() > fl.max_distance => lifted(at),
        Lock::Held(at) => Lock::Held(at),
        Lock::Releasing(from, t) => {
            let t = t + dt;
            if t >= release {
                Lock::Free
            } else {
                Lock::Releasing(from, t)
            }
        }
    };
    let target = match lock {
        Lock::Free => None,
        Lock::Held(at) => Some(at),
        Lock::Releasing(from, t) => Some(from + (animated - from) * (t / release)),
    };
    if let Some(target) = target {
        let to = frame.inverse().transform_point(target);
        let turned = model[foot as usize].rotation;
        let lim = chain_limits(w, &leg, limits);
        ik::two_bone(
            &w.asset,
            &mut w.pose.nodes,
            &leg,
            to,
            None,
            fl.weight.clamp(0.0, 1.0),
            &lim,
        );
        // The foot keeps its animated orientation.
        let parent = w.asset.nodes[foot as usize].parent as usize;
        let m = w.asset.model_space(&w.pose.nodes);
        w.pose.nodes[foot as usize].rotation = unit(m[parent].rotation.conjugate() * turned);
    }
    fl.planted = planted;
    fl.locked = matches!(lock, Lock::Held(_));
    Ok(lock)
}

/// Joint limits by (skeleton entity, node).
type Limits = BTreeMap<(u64, u32), JointLimit>;

/// The node of joint `joint` of `w`'s skin, or an error naming `field`.
fn joint(w: &Work, joint: u32, field: &str) -> Result<u32> {
    let skin = w.skeleton.skin;
    w.asset
        .joint_node(skin, joint)
        .ok_or_else(|| invalid(format!("{field} {joint} is not a joint of skin {skin}")))
}

/// The limits of `nodes` on `w`'s skeleton, aligned with them (empty when
/// none is limited, so the solvers take their unlimited path).
fn chain_limits(w: &Work, nodes: &[u32], limits: &Limits) -> Vec<Option<JointLimit>> {
    let key = w.entity.to_bits();
    let out: Vec<Option<JointLimit>> = nodes
        .iter()
        .map(|&n| limits.get(&(key, n)).copied())
        .collect();
    if out.iter().any(Option::is_some) {
        out
    } else {
        Vec::new()
    }
}

/// A point of `space` in the skeleton's model space (world: the inverse
/// of the entity's Transform).
fn model_point(space: u32, t: Option<Transform>, p: Vec3) -> Vec3 {
    match (space, t) {
        (IK_WORLD, Some(t)) => Trs::new(t.position, t.quaternion, t.scale)
            .inverse()
            .transform_point(p),
        _ => p,
    }
}

/// A direction of `space` in the skeleton's model space.
fn model_direction(space: u32, t: Option<Transform>, v: Vec3) -> Vec3 {
    match (space, t) {
        (IK_WORLD, Some(t)) => t.quaternion.conjugate() * v,
        _ => v,
    }
}

fn aim(w: &mut Work, look: &mut LookAt, limits: &Limits) -> Result<()> {
    let node = joint(w, look.joint, "og.LookAt.joint")?;
    let bones = look.bones.clamp(1, 8);
    let Some(nodes) = ik::chain(&w.asset, node, bones - 1) else {
        return Err(invalid(format!(
            "og.LookAt: joint {} has fewer than {} joints above it",
            look.joint,
            bones - 1
        )));
    };
    let target = model_point(look.space, w.transform, look.target);
    let up = (look.up != Vec3::ZERO).then(|| model_direction(look.space, w.transform, look.up));
    let lim = chain_limits(w, &nodes, limits);
    look.clamped = ik::look_at(&w.asset, &mut w.pose.nodes, &nodes, target, up, look, &lim);
    Ok(())
}

fn solve(w: &mut Work, chain: &mut IkChain, limits: &Limits) -> Result<()> {
    let tip = joint(w, chain.tip, "og.IkChain.tip")?;
    let bones = if chain.solver == IK_TWO_BONE {
        2
    } else {
        chain.bones.max(1)
    };
    let Some(nodes) = ik::chain(&w.asset, tip, bones) else {
        return Err(invalid(format!(
            "og.IkChain: joint {} has fewer than {bones} bones above it",
            chain.tip
        )));
    };
    let to_model = |p: Vec3| model_point(chain.space, w.transform, p);
    let target = to_model(chain.target);
    let weight = chain.weight.clamp(0.0, 1.0);
    let lim = chain_limits(w, &nodes, limits);
    let result = if chain.solver == IK_TWO_BONE {
        let pole = (chain.pole != Vec3::ZERO).then(|| to_model(chain.pole));
        ik::two_bone(
            &w.asset,
            &mut w.pose.nodes,
            &nodes,
            target,
            pole,
            weight,
            &lim,
        )
    } else {
        ik::fabrik(
            &w.asset,
            &mut w.pose.nodes,
            &nodes,
            target,
            chain.iterations,
            chain.tolerance,
            weight,
            &lim,
        )
    };
    chain.error = result.error;
    chain.reached = result.reachable && result.error <= chain.tolerance;
    Ok(())
}

/// A clip leaf's playback over this tick, from `phase0` to `phase1`.
#[allow(clippy::too_many_arguments)]
fn phase_leaf<'a>(
    handle: u64,
    clip: &'a ClipAsset,
    binding: &'a [u32],
    looping: bool,
    before: Phase,
    after: Phase,
    wraps: i32,
    weight: f32,
) -> Leaf<'a> {
    let len = clip.duration;
    let at = |p: Phase| {
        if looping {
            p.frac * len
        } else if p.cycles >= 1 {
            len
        } else {
            p.frac * len
        }
    };
    Leaf {
        handle,
        clip,
        binding,
        t0: at(before),
        t1: at(after),
        wraps: if looping { wraps } else { 0 },
        weight,
    }
}

#[allow(clippy::too_many_arguments)]
fn eval_graph(
    sh: &Shared,
    e: Entity,
    skeleton_handle: u64,
    skel: &SkeletonAsset,
    graph: &GraphAsset,
    comp: &mut AnimationGraph,
    gs: &mut GraphState,
    root: Option<&RootMotion>,
    mesh: u32,
    dt: f32,
    events: &mut Vec<Fired>,
    stance: &mut Stance,
) -> Result<(Pose, Trs)> {
    if gs.asset != comp.asset || gs.layers.len() != graph.layers.len() {
        gs.asset = comp.asset;
        gs.layers = graph
            .layers
            .iter()
            .map(|l| LayerState {
                state: l.entry as u32,
                prev: NONE,
                ..LayerState::default()
            })
            .collect();
    }
    let params = comp.params();
    let speed = comp.speed.max(0.0);
    let graph_weight = comp.weight.clamp(0.0, 1.0);
    let mut pose = Pose::rest(skel);
    let mut delta = Trs::IDENTITY;
    // Each sync group's phase this tick (before, after, wraps), set by the
    // first layer whose current state is in it.
    let mut leaders: Vec<Option<(Phase, Phase, i32)>> = vec![None; graph.sync_groups.len()];
    for (li, layer) in graph.layers.iter().enumerate() {
        let ls = &mut gs.layers[li];
        if ls.state as usize >= layer.states.len() {
            ls.state = layer.entry as u32;
        }
        // During a fade, the transition's `interrupt` says which may fire.
        let guard = if ls.prev != NONE {
            ls.guard
        } else {
            Interrupt::Any
        };
        for t in &layer.transitions {
            let (cur, prev) = (ls.state as usize, ls.prev as usize);
            // A source interrupt leaves from the state being faded out.
            let from_source = guard == Interrupt::Source;
            let open = match (guard, t.from) {
                (Interrupt::None, _) => false,
                (Interrupt::Source, Some(f)) => f == prev && t.to != cur,
                (Interrupt::Source, None) => false,
                (Interrupt::Next, Some(f)) => f == cur,
                (Interrupt::Next, None) => false,
                (Interrupt::Any, Some(f)) => f == cur,
                (Interrupt::Any, None) => t.to != cur,
            };
            if !open || !t.conditions.iter().all(|c| c.holds(&params)) {
                continue;
            }
            let (left, left_phase) = if from_source {
                (prev, ls.prev_phase)
            } else {
                (cur, ls.phase)
            };
            if t.exit.is_some_and(|x| left_phase.total() < x as f64) {
                continue;
            }
            let carried = match (layer.states[left].sync, layer.states[t.to].sync) {
                // Between two states of one group the fraction carries over.
                (Some(a), Some(b)) if a == b => left_phase.frac,
                _ => 0.0,
            };
            // The fade continues from the state left (a source interrupt
            // drops the target it abandons).
            ls.prev = left as u32;
            ls.prev_phase = left_phase;
            ls.guard = t.interrupt;
            ls.state = t.to as u32;
            ls.phase = Phase {
                cycles: 0,
                frac: carried,
            };
            ls.time = 0.0;
            ls.fade = 0.0;
            ls.fade_duration = t.fade;
            break;
        }
        let layer_weight = layer.weight(&params);
        // The cross-fade's progress after this tick.
        let alpha = if ls.prev != NONE {
            ls.fade += dt;
            if ls.fade_duration > 0.0 && ls.fade < ls.fade_duration {
                ls.fade / ls.fade_duration
            } else {
                1.0
            }
        } else {
            1.0
        };
        let base = li == 0;
        let leaf_root = if base { root } else { None };
        let mut layer_blend = Blend::new();
        let mut reference = Blend::new();
        let mut run_state = |state: usize,
                             phase: &mut Phase,
                             w: f32,
                             follow: Option<(Phase, Phase, i32)>|
         -> Result<(Phase, i32)> {
            let st = &layer.states[state];
            let mut leaves = Vec::new();
            graph.leaves(st.motion, &params, 1.0, &mut leaves);
            let mut duration = 0.0f32;
            for &(n, lw) in &leaves {
                if let Motion::Clip {
                    clip, speed: cs, ..
                } = graph.nodes[n].motion
                {
                    let c = sh
                        .clips
                        .get(&clip)
                        .ok_or_else(|| missing(e, "clip", clip))?;
                    duration += lw * c.duration / cs;
                }
            }
            let (before, wraps) = match follow {
                // A follower plays at its group leader's phase.
                Some((b, a, k)) => {
                    *phase = a;
                    (b, k)
                }
                None => {
                    let before = *phase;
                    let wraps = if duration > 0.0 {
                        phase.advance(dt * speed * st.speed / duration)
                    } else {
                        0
                    };
                    (before, wraps)
                }
            };
            for &(n, lw) in &leaves {
                let Motion::Clip { clip, looping, .. } = graph.nodes[n].motion else {
                    continue;
                };
                let c = sh
                    .clips
                    .get(&clip)
                    .ok_or_else(|| missing(e, "clip", clip))?;
                let b = sh
                    .binding(skeleton_handle, clip)
                    .ok_or_else(|| missing(e, "clip", clip))?;
                let leaf = phase_leaf(
                    clip,
                    c,
                    &b,
                    looping,
                    before,
                    *phase,
                    wraps,
                    lw * w * layer_weight * graph_weight,
                );
                leaf.events(events);
                let d = if base {
                    stance.push((clip, leaf.t1, lw * w * layer_weight));
                    leaf.delta(skel, leaf_root)
                } else {
                    Trs::IDENTITY
                };
                layer_blend.push(leaf.pose(skel, leaf_root, mesh), d, lw * w);
                if layer.mode == LayerMode::Additive {
                    reference.push(leaf.reference(skel, mesh), Trs::IDENTITY, lw * w);
                }
            }
            Ok((before, wraps))
        };
        let (cur, mut cur_phase) = (ls.state as usize, ls.phase);
        if ls.prev != NONE && alpha < 1.0 {
            let (prev, mut prev_phase) = (ls.prev as usize, ls.prev_phase);
            run_state(prev, &mut prev_phase, 1.0 - alpha, None)?;
            ls.prev_phase = prev_phase;
        }
        let group = layer.states[cur].sync;
        let follow = group.and_then(|g| leaders[g]);
        let (before, wraps) = run_state(cur, &mut cur_phase, alpha, follow)?;
        if let Some(g) = group
            && leaders[g].is_none()
        {
            leaders[g] = Some((before, cur_phase, wraps));
        }
        ls.phase = cur_phase;
        ls.time += dt * speed;
        if alpha >= 1.0 {
            ls.prev = NONE;
            ls.guard = Interrupt::Any;
        }
        let layer_pose = layer_blend.pose.unwrap_or_else(|| Pose::rest(skel));
        let mask = layer.node_weights(skel);
        match (layer.mode, &mask) {
            (LayerMode::Override, None) => pose.lerp_to(&layer_pose, layer_weight),
            (LayerMode::Override, Some(m)) => pose.lerp_to_masked(&layer_pose, layer_weight, m),
            (LayerMode::Additive, _) => {
                let r = reference.pose.unwrap_or_else(|| Pose::rest(skel));
                match &mask {
                    None => pose.add(&layer_pose, &r, layer_weight),
                    Some(m) => pose.add_masked(&layer_pose, &r, layer_weight, m),
                }
            }
        }
        if base {
            delta = Trs::IDENTITY.lerp(&layer_blend.delta, layer_weight);
        }
    }
    if let Some(l) = gs.layers.first() {
        comp.state = l.state;
        comp.state_time = l.time;
    }
    Ok((pose, delta))
}

/// Advances clip time `t` by `d` over a clip of `len`: the new time and the wraps.
fn advance_time(t: f32, d: f32, len: f32, looping: bool) -> (f32, i32) {
    let raw = t + d;
    if looping && len > 0.0 {
        let k = (raw / len).floor();
        let mut t1 = raw - k * len;
        if t1 >= len {
            t1 = 0.0;
        }
        if t1 < 0.0 {
            t1 = 0.0;
        }
        (t1, k as i32)
    } else {
        (raw.clamp(0.0, len.max(0.0)), 0)
    }
}

#[allow(clippy::too_many_arguments)]
fn eval_player(
    sh: &Shared,
    e: Entity,
    skeleton_handle: u64,
    skel: &SkeletonAsset,
    comp: &mut AnimationPlayer,
    ps: &mut PlayerState,
    root: Option<&RootMotion>,
    mesh: u32,
    dt: f32,
    events: &mut Vec<Fired>,
    stance: &mut Stance,
) -> Result<Option<(Pose, Trs)>> {
    let seek = comp.time != ps.written;
    if comp.clip != ps.clip {
        if ps.clip != 0 && comp.fade > 0.0 && sh.clips.contains_key(&ps.clip) {
            ps.prev_clip = ps.clip;
            ps.prev_time = ps.time;
            ps.prev_speed = ps.speed;
            ps.prev_looping = ps.looping;
            ps.fade = 0.0;
            ps.fade_duration = comp.fade;
        } else {
            ps.prev_clip = 0;
        }
        ps.clip = comp.clip;
        ps.time = if seek { comp.time } else { 0.0 };
    } else if seek {
        ps.time = comp.time;
    }
    ps.speed = comp.speed;
    ps.looping = comp.looping;
    if ps.clip == 0 {
        ps.prev_clip = 0;
        ps.written = comp.time;
        return Ok(None);
    }
    let clip = sh
        .clips
        .get(&ps.clip)
        .ok_or_else(|| missing(e, "og.AnimationPlayer.clip", ps.clip))?;
    let alpha = if ps.prev_clip != 0 {
        ps.fade += dt;
        if ps.fade_duration > 0.0 && ps.fade < ps.fade_duration {
            ps.fade / ps.fade_duration
        } else {
            1.0
        }
    } else {
        1.0
    };
    // The player decides `weight` of the pose over what the graph left.
    let wt = comp.weight.clamp(0.0, 1.0);
    for s in stance.iter_mut() {
        s.2 *= 1.0 - wt;
    }
    let mut blend = Blend::new();
    if ps.prev_clip != 0 && alpha < 1.0 {
        let pc = sh
            .clips
            .get(&ps.prev_clip)
            .ok_or_else(|| missing(e, "clip", ps.prev_clip))?;
        let b = sh
            .binding(skeleton_handle, ps.prev_clip)
            .ok_or_else(|| missing(e, "clip", ps.prev_clip))?;
        let (t1, wraps) = advance_time(
            ps.prev_time,
            dt * ps.prev_speed,
            pc.duration,
            ps.prev_looping,
        );
        let leaf = Leaf {
            handle: ps.prev_clip,
            clip: pc,
            binding: &b,
            t0: ps.prev_time,
            t1,
            wraps,
            weight: (1.0 - alpha) * comp.weight,
        };
        leaf.events(events);
        let held = stepped(t1, comp.step_fps, pc.duration, ps.prev_looping);
        stance.push((ps.prev_clip, held, (1.0 - alpha) * wt));
        blend.push(
            leaf.pose_at(skel, root, mesh, held),
            leaf.delta(skel, root),
            1.0 - alpha,
        );
        ps.prev_time = t1;
    }
    if alpha >= 1.0 {
        ps.prev_clip = 0;
    }
    let b = sh
        .binding(skeleton_handle, ps.clip)
        .ok_or_else(|| missing(e, "clip", ps.clip))?;
    let (t1, wraps) = advance_time(ps.time, dt * comp.speed, clip.duration, comp.looping);
    let leaf = Leaf {
        handle: ps.clip,
        clip,
        binding: &b,
        t0: ps.time,
        t1,
        wraps,
        weight: alpha * comp.weight,
    };
    leaf.events(events);
    let held = stepped(t1, comp.step_fps, clip.duration, comp.looping);
    stance.push((ps.clip, held, alpha * wt));
    blend.push(
        leaf.pose_at(skel, root, mesh, held),
        leaf.delta(skel, root),
        alpha,
    );
    ps.time = t1;
    ps.written = t1;
    comp.time = t1;
    comp.finished = !comp.looping
        && ((comp.speed >= 0.0 && t1 >= clip.duration) || (comp.speed < 0.0 && t1 <= 0.0));
    Ok(blend.pose.map(|p| (p, blend.delta)))
}
