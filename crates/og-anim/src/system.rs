//! The `og.anim` system: one evaluation per tick of every skeleton, its
//! player, graph, root motion, blendshapes and IK chains; the joint palettes
//! it leaves for the renderer; and the state block that saves it all.

mod constraints;
mod playback;
mod state;

use constraints::{Limits, aim, foot_lock, joint, solve, spring};
use playback::{eval_graph, eval_player};
use state::AnimState;

use crate::asset::{ClipAsset, NONE, SkeletonAsset, bind};
use crate::codec::{Reader, Writer};
use crate::graph::{GraphAsset, Interrupt, LayerMode, Motion};
use crate::ik;
use crate::pose::{Blend, Fired, Leaf, Pose, RootMotion, stepped};
use crate::retarget::{FootPlants, HumanoidMap, TwistBone, share_twist, twist_bones};
use crate::schema::*;
use crate::xform::{Trs, nlerp, rotation_arc, unit};
use og_core::IdMap;
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

/// Foot plants by (skeleton, clip, mirrored, foot node), derived from the assets.
type Plants = BTreeMap<(u64, u64, bool, u32), Arc<FootPlants>>;

/// A clip mirrored for a skeleton (a graph's `mirror true`) and its binding.
type Mirrored = Arc<(ClipAsset, Vec<u32>)>;

/// Mirrored clips by (skeleton, clip).
type Mirrors = BTreeMap<(u64, u64), Mirrored>;

/// A skeleton's twist bones (on the limb segments of the humanoid map its
/// names give) and its rest pose in model space.
struct TwistRig {
    bones: Vec<TwistBone>,
    rest: Vec<Trs>,
}

/// Twist rigs by skeleton.
type TwistRigs = IdMap<u64, Arc<TwistRig>>;

/// What a clip does to each twist bone of a skeleton, by (skeleton, clip,
/// mirrored): [`TWIST_STILL`], [`TWIST_SEGMENT`] or [`TWIST_ANIMATED`].
type TwistRoles = BTreeMap<(u64, u64, bool), Arc<Vec<u8>>>;

/// The clip turns neither the twist bone nor its segment.
const TWIST_STILL: u8 = 0;
/// The clip turns the segment (its bone or the next) and not the twist bone.
const TWIST_SEGMENT: u8 = 1;
/// The clip has the twist bone's rotation.
const TWIST_ANIMATED: u8 = 2;

/// What a clip that poses the legs played this tick: ((clip, mirrored),
/// the time it was sampled at, its share of the pose). The base layer's
/// and the player's clips, for foot locks.
type Stance = Vec<((u64, bool), f32, f32)>;

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

impl Lock {
    fn shift(&mut self, d: og_core::glam::DVec3) {
        let m = |p: &mut Vec3| *p = (p.as_dvec3() + d).as_vec3();
        match self {
            Lock::Free => {}
            Lock::Held(p) | Lock::Releasing(p, _) => m(p),
        }
    }
}

/// One `og.SpringBone` entity's tails between ticks, keyed by that
/// entity; positions and velocities in the spring's `space`.
#[derive(Clone, Debug, Default, PartialEq)]
struct SpringState {
    /// The skeleton entity it swung (a spring moved to another, or to
    /// another space, or given another length, restarts at rest).
    skeleton: u64,
    space: u32,
    /// Per chain joint, top first: the tail's position and velocity.
    tails: Vec<(Vec3, Vec3)>,
}

/// What the state block holds.
#[derive(Clone, Debug, Default, PartialEq)]
struct Saved {
    states: IdMap<u64, EntityState>,
    locks: IdMap<u64, LockState>,
    springs: IdMap<u64, SpringState>,
}

/// The registered assets and the world state.
#[derive(Default)]
pub(crate) struct Shared {
    skeletons: IdMap<u64, Arc<SkeletonAsset>>,
    clips: IdMap<u64, Arc<ClipAsset>>,
    graphs: IdMap<u64, Arc<GraphAsset>>,
    /// A cache filled by the lanes as they meet (skeleton, clip) pairs.
    bindings: Mutex<Bindings>,
    /// A cache filled as foot locks meet (skeleton, clip, foot) triples.
    plants: Mutex<Plants>,
    /// A cache filled as graphs play mirrored clips on skeletons.
    mirrors: Mutex<Mirrors>,
    /// Caches filled as skeletons with clips meet twist distribution.
    twist_rigs: Mutex<TwistRigs>,
    twist_roles: Mutex<TwistRoles>,
    states: IdMap<u64, EntityState>,
    locks: IdMap<u64, LockState>,
    springs: IdMap<u64, SpringState>,
    outputs: IdMap<u64, AnimOutput>,
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

fn lock_mirrors(m: &Mutex<Mirrors>) -> MutexGuard<'_, Mirrors> {
    m.lock().unwrap_or_else(|p| p.into_inner())
}

fn lock_cache<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|p| p.into_inner())
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

    fn plants(
        &self,
        skeleton: u64,
        (clip, mirrored): (u64, bool),
        foot: u32,
    ) -> Option<Arc<FootPlants>> {
        let key = (skeleton, clip, mirrored, foot);
        if let Some(p) = lock_plants(&self.plants).get(&key) {
            return Some(p.clone());
        }
        let skel = self.skeletons.get(&skeleton)?;
        let p = Arc::new(if mirrored {
            FootPlants::new(skel, &self.mirrored(skeleton, clip).ok()??.0, foot)
        } else {
            FootPlants::new(skel, self.clips.get(&clip)?, foot)
        });
        Some(lock_plants(&self.plants).entry(key).or_insert(p).clone())
    }

    /// Clip `clip` mirrored for skeleton `skeleton` and its binding, made
    /// on first use (`None` when either is not registered).
    fn mirrored(&self, skeleton: u64, clip: u64) -> Result<Option<Mirrored>> {
        if let Some(m) = lock_mirrors(&self.mirrors).get(&(skeleton, clip)) {
            return Ok(Some(m.clone()));
        }
        let (Some(skel), Some(c)) = (self.skeletons.get(&skeleton), self.clips.get(&clip)) else {
            return Ok(None);
        };
        // Computed outside the lock: two lanes may both mirror a pair, to
        // the same value.
        let m = crate::retarget::mirror(skel, c)?;
        let b = bind(skel, &m);
        let m = Arc::new((m, b));
        Ok(Some(
            lock_mirrors(&self.mirrors)
                .entry((skeleton, clip))
                .or_insert(m)
                .clone(),
        ))
    }

    /// Skeleton `skeleton`'s twist rig, made on first use.
    fn twist_rig(&self, skeleton: u64, asset: &SkeletonAsset) -> Arc<TwistRig> {
        if let Some(r) = lock_cache(&self.twist_rigs).get(&skeleton) {
            return r.clone();
        }
        let rest = asset.model_space(&asset.rest_pose());
        let bones = twist_bones(asset, &HumanoidMap::detect(asset), &rest);
        let r = Arc::new(TwistRig { bones, rest });
        lock_cache(&self.twist_rigs)
            .entry(skeleton)
            .or_insert(r)
            .clone()
    }

    /// What clip `clip` (mirrored or not) does to each of `rig`'s twist
    /// bones on skeleton `skeleton` (`None` when it is not registered).
    fn twist_roles(
        &self,
        skeleton: u64,
        rig: &TwistRig,
        (clip, mirrored): (u64, bool),
    ) -> Result<Option<Arc<Vec<u8>>>> {
        let key = (skeleton, clip, mirrored);
        if let Some(r) = lock_cache(&self.twist_roles).get(&key) {
            return Ok(Some(r.clone()));
        }
        let turned: Vec<u32> = if mirrored {
            let Some(m) = self.mirrored(skeleton, clip)? else {
                return Ok(None);
            };
            rotated(&m.0, &m.1)
        } else {
            let (Some(c), Some(b)) = (self.clips.get(&clip), self.binding(skeleton, clip)) else {
                return Ok(None);
            };
            rotated(c, &b)
        };
        let has = |n: usize| turned.contains(&(n as u32));
        let roles: Vec<u8> = rig
            .bones
            .iter()
            .map(|t| {
                if has(t.node) {
                    TWIST_ANIMATED
                } else if has(t.a) || has(t.c) {
                    TWIST_SEGMENT
                } else {
                    TWIST_STILL
                }
            })
            .collect();
        let r = Arc::new(roles);
        Ok(Some(
            lock_cache(&self.twist_roles)
                .entry(key)
                .or_insert(r)
                .clone(),
        ))
    }

    /// Forgets what was derived from skeleton `skeleton` or clip `clip`.
    fn forget(&self, skeleton: Option<u64>, clip: Option<u64>) {
        lock_cache(&self.twist_rigs).retain(|&k, _| Some(k) != skeleton);
        lock_cache(&self.twist_roles).retain(|k, _| Some(k.0) != skeleton && Some(k.1) != clip);
        lock_bindings(&self.bindings).retain(|k, _| Some(k.0) != skeleton && Some(k.1) != clip);
        lock_plants(&self.plants).retain(|k, _| Some(k.0) != skeleton && Some(k.1) != clip);
        lock_mirrors(&self.mirrors).retain(|k, _| Some(k.0) != skeleton && Some(k.1) != clip);
    }

    /// og.WorldOrigin moved the world's zero ([`og_ecs::StateBlock::shift`]).
    fn shift(&mut self, s: &og_ecs::Shift) {
        for l in self.locks.values_mut() {
            l.lock.shift(s.of(l.skeleton));
        }
        for sp in self.springs.values_mut() {
            if sp.space == 0 {
                let d = s.of(sp.skeleton);
                for (p, _) in &mut sp.tails {
                    *p = (p.as_dvec3() + d).as_vec3();
                }
            }
        }
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

/// The nodes a clip's rotation channels turn, by `binding`.
fn rotated(clip: &ClipAsset, binding: &[u32]) -> Vec<u32> {
    clip.channels
        .iter()
        .zip(binding)
        .filter(|(c, b)| c.path == crate::asset::ChannelPath::Rotation && **b != NONE)
        .map(|(_, &b)| b)
        .collect()
}

/// Twist distribution, per tick, for clips that are not retargeted: each
/// twist bone of `w`'s skeleton that a clip posing it this tick turns the
/// segment of, while none of them animates the twist bone itself, takes
/// its share of the segment's twist in the pose as it now is; the nodes
/// below it keep their model rotations. A skeleton without twist bones, or
/// whose clips animate them (retargeting writes them), is left bit for bit.
fn distribute_twist(sh: &Shared, w: &mut Work) -> Result<()> {
    if w.stance.iter().all(|s| s.2 <= 0.0) {
        return Ok(());
    }
    let rig = sh.twist_rig(w.skeleton.asset, &w.asset);
    if rig.bones.is_empty() {
        return Ok(());
    }
    let (mut segment, mut animated) = (vec![false; rig.bones.len()], vec![false; rig.bones.len()]);
    for &(clip, _, weight) in &w.stance {
        if weight <= 0.0 {
            continue;
        }
        if let Some(roles) = sh.twist_roles(w.skeleton.asset, &rig, clip)? {
            for (j, &r) in roles.iter().enumerate() {
                segment[j] |= r == TWIST_SEGMENT;
                animated[j] |= r == TWIST_ANIMATED;
            }
        }
    }
    let driven: Vec<&TwistBone> = rig
        .bones
        .iter()
        .enumerate()
        .filter(|&(j, _)| segment[j] && !animated[j])
        .map(|(_, t)| t)
        .collect();
    if driven.is_empty() {
        return Ok(());
    }
    // The nodes below a driven twist bone (nodes are in topological order).
    let nodes = &w.asset.nodes;
    let mut below = vec![false; nodes.len()];
    for n in 0..nodes.len() {
        let p = nodes[n].parent;
        below[n] = p != NONE && (below[p as usize] || driven.iter().any(|t| t.node == p as usize));
    }
    let fk = w.asset.model_space(&w.pose.nodes);
    share_twist(&w.asset, &driven, &rig.rest, &fk, &mut w.pose.nodes, |n| {
        below[n]
    });
    Ok(())
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
    install_with(world, None)
}

/// Where a player whose own `stepFps` is 0 takes its stepped playback
/// from: an `f32` field of a resource, read every tick inside the system
/// (og-runtime's `ArtMotion.stepFps` under a project's art contract,
/// docs/art.md §2.4), so it follows the world as a look changes it and a
/// replay or restore sees what the tick saw. The component is never
/// written with it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepDefault {
    /// The resource's registered name.
    pub resource: String,
    /// The field's byte offset in the resource's packed value.
    pub offset: usize,
}

/// [`install`], with players defaulting their step to `step` (none: 0,
/// smooth, as [`install`]).
pub fn install_with(world: &mut World, step: Option<StepDefault>) -> Result<AnimationSystem> {
    world.register::<Transform>()?;
    world.register::<Skeleton>()?;
    world.register::<AnimationPlayer>()?;
    world.register::<AnimationGraph>()?;
    world.register::<Blendshapes>()?;
    world.register::<IkChain>()?;
    world.register::<LookAt>()?;
    world.register::<JointLimit>()?;
    world.register::<FootLock>()?;
    world.register::<SpringBone>()?;
    world.register::<SpringCollider>()?;
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
        .write::<SpringBone>()
        .read::<SpringCollider>()
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
        .query(QueryDesc::new().read::<FootLock>())
        .query(QueryDesc::new().read::<SpringBone>())
        .query(QueryDesc::new().read::<SpringCollider>());
    let mut desc = desc;
    let step = match step {
        Some(st) => {
            let id = world.id_by_name(&st.resource)?;
            desc.reads.push(st.resource);
            Some((id, st.offset))
        }
        None => None,
    };
    let s = shared.clone();
    world.add_system(desc, move |ctx| run(ctx, &s, step))?;
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
    /// This tick's scaled length for the entity: the global scaled dt
    /// times its time group's scale (og.TimeGroup).
    dt: f32,
}

/// One skeleton entity on its way through the tick: what was read (serially,
/// from the world), then what its lane computed from that alone.
struct Eval {
    work: Work,
    /// The stepped playback of a player whose own `stepFps` is 0 (the art
    /// contract's ArtMotion.stepFps, [`StepDefault`]); 0 none.
    step_default: f32,
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
            dt: 0.0,
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
        step_default: 0.0,
        graph_asset,
        root,
        mesh,
        out: Ok((Trs::IDENTITY, Vec::new())),
    })
}

/// One entity's lane: its graph and player evaluated into `work.pose`, from
/// what [`gather`] read and the shared assets only.
fn evaluate(sh: &Shared, ev: &mut Eval, dt: f32) -> Result<(Trs, Vec<Fired>)> {
    let step_default = ev.step_default;
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
                step_default,
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

fn run(
    ctx: &mut SystemContext<'_>,
    shared: &Mutex<Shared>,
    step: Option<(og_ecs::SchemaId, usize)>,
) -> Result<()> {
    // Scaled time: slow motion plays slower, a hit stop holds the pose;
    // an entity in a time group (og.TimeGroup) by its group's scale too.
    let dt = ctx.scaled_dt();
    let groups = ctx.time_groups();
    // The stepped playback of players without their own (StepDefault).
    let step_default = match step {
        Some((id, at)) => {
            let bytes = ctx.resource_bytes(id)?;
            bytes
                .get(at..at + 4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .filter(|v| v.is_finite() && *v > 0.0)
                .unwrap_or(0.0)
        }
        None => 0.0,
    };
    let skeletons = entities(ctx, 0)?;
    let with = |q: usize| -> Result<BTreeSet<u64>> {
        Ok(entities(ctx, q)?.iter().map(|e| e.to_bits()).collect())
    };
    let (players, graphs, blends, transforms) = (with(1)?, with(2)?, with(3)?, with(4)?);
    let chains = entities(ctx, 5)?;
    let looks = entities(ctx, 6)?;
    let joint_limits = entities(ctx, 7)?;
    let foot_locks = entities(ctx, 8)?;
    let spring_bones = entities(ctx, 9)?;
    let spring_colliders = entities(ctx, 10)?;

    let mut sh = lock(shared);
    // Read every skeleton entity in entity order, up to the first one that
    // fails; evaluate them in lanes; then take the results in entity order,
    // so the first error and every write are the ones one thread doing
    // entity after entity would have produced.
    let mut evals: Vec<Eval> = Vec::with_capacity(skeletons.len());
    let mut unread = None;
    for &e in &skeletons {
        match gather(ctx, &sh, e, &players, &graphs, &blends, &transforms) {
            Ok(mut ev) => {
                ev.step_default = step_default;
                ev.work.dt = ctx.group_dt(groups.as_ref(), e, dt);
                evals.push(ev)
            }
            Err(err) => {
                unread = Some(err);
                break;
            }
        }
    }
    {
        let sh: &Shared = &sh;
        og_lanes::for_each_mut(og_lanes::Site::Anim, &mut evals, GRAIN, |ev| {
            let dt = ev.work.dt;
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
    let mut locks: IdMap<u64, LockState> = IdMap::new();
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
            let lock = foot_lock(&sh, w, &mut fl, prev, w.dt, &limits)?;
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
    // Twist bones, after the solvers that turn their segments.
    for w in &mut work {
        distribute_twist(&sh, w)?;
    }
    // Spring colliders, in query order, with the node each rides on.
    let mut colliders: Vec<(u32, SpringCollider)> = Vec::with_capacity(spring_colliders.len());
    for &c in &spring_colliders {
        let col: SpringCollider = ctx.get(c)?;
        if let Some(w) = work.iter().find(|w| w.entity.to_bits() == col.skeleton) {
            colliders.push((joint(w, col.joint, "og.SpringCollider.joint")?, col));
        }
    }
    // Spring bones, last (they follow the final pose of their parents), in
    // query order.
    let mut springs: IdMap<u64, SpringState> = IdMap::new();
    let mut swung: Vec<(Entity, SpringBone)> = Vec::with_capacity(spring_bones.len());
    for &s in &spring_bones {
        let mut sb: SpringBone = ctx.get(s)?;
        sb.lag = 0.0;
        if let Some(w) = work.iter_mut().find(|w| w.entity.to_bits() == sb.skeleton) {
            let prev = sh
                .springs
                .get(&s.to_bits())
                .filter(|p| p.skeleton == sb.skeleton && p.space == sb.space);
            let dt = w.dt;
            springs.insert(s.to_bits(), spring(w, &mut sb, prev, &colliders, dt)?);
        }
        swung.push((s, sb));
    }

    // The joint palettes, in lanes; then everything is written back in
    // entity order.
    let palettes = og_lanes::map(og_lanes::Site::Anim, &work, GRAIN, |w| {
        let mut palette = Vec::new();
        w.asset
            .palette(w.skeleton.skin, &w.pose.nodes, &mut palette);
        palette
    });
    let mut states = IdMap::new();
    let mut outputs = IdMap::new();
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
    for (s, sb) in swung {
        ctx.set(s, sb)?;
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
    sh.springs = springs;
    sh.outputs = outputs;
    Ok(())
}
