//! The `og.anim` system: one evaluation per tick of every skeleton, its
//! player, graph, root motion, blendshapes and IK chains; the joint palettes
//! it leaves for the renderer; and the state block that saves it all.

use crate::asset::{bind, ClipAsset, SkeletonAsset, NONE};
use crate::codec::{Reader, Writer};
use crate::graph::{GraphAsset, LayerMode, Motion};
use crate::ik;
use crate::pose::{Blend, Fired, Leaf, Pose, RootMotion};
use crate::schema::*;
use crate::xform::{unit, Trs};
use og_core::glam::{Mat4, Quat, Vec3};
use og_core::{Error, ErrorKind, Result, Str64};
use og_ecs::{Entity, QueryDesc, StateBlock, SystemContext, SystemDesc, World};
use std::any::Any;
use std::cell::RefCell;
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
    player: PlayerState,
    graph: GraphState,
}

/// Channel-to-node bindings by (skeleton, clip), derived from the assets.
type Bindings = BTreeMap<(u64, u64), Arc<Vec<u32>>>;

/// The registered assets and the world state.
#[derive(Default)]
pub(crate) struct Shared {
    skeletons: BTreeMap<u64, Arc<SkeletonAsset>>,
    clips: BTreeMap<u64, Arc<ClipAsset>>,
    graphs: BTreeMap<u64, Arc<GraphAsset>>,
    bindings: RefCell<Bindings>,
    states: BTreeMap<u64, EntityState>,
    outputs: BTreeMap<u64, AnimOutput>,
}

fn lock(shared: &Mutex<Shared>) -> MutexGuard<'_, Shared> {
    shared.lock().unwrap_or_else(|p| p.into_inner())
}

impl Shared {
    fn binding(&self, skeleton: u64, clip: u64) -> Option<Arc<Vec<u32>>> {
        if let Some(b) = self.bindings.borrow().get(&(skeleton, clip)) {
            return Some(b.clone());
        }
        let b = Arc::new(bind(self.skeletons.get(&skeleton)?, self.clips.get(&clip)?));
        self.bindings
            .borrow_mut()
            .insert((skeleton, clip), b.clone());
        Some(b)
    }

    fn recompute_outputs(&mut self) {
        self.outputs.clear();
        for (&e, st) in &self.states {
            if let Some(skel) = self.skeletons.get(&st.skeleton) {
                if skel.nodes.len() == st.pose.len() {
                    let mut palette = Vec::new();
                    skel.palette(st.skin, &st.pose, &mut palette);
                    self.outputs.insert(
                        e,
                        AnimOutput {
                            skin: st.skin,
                            palette,
                            mesh: st.mesh,
                            weights: st.weights,
                        },
                    );
                }
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
    world.register::<AnimationEvent>()?;
    let shared = Arc::new(Mutex::new(Shared::default()));
    let desc = SystemDesc::new(SYSTEM_NAME)
        .write::<Skeleton>()
        .write::<AnimationPlayer>()
        .write::<AnimationGraph>()
        .write::<Blendshapes>()
        .write::<IkChain>()
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
        .query(QueryDesc::new().read::<IkChain>());
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
        sh.bindings.borrow_mut().retain(|k, _| k.0 != handle);
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
        sh.bindings.borrow_mut().retain(|k, _| k.1 != handle);
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
        sh.bindings.borrow_mut().retain(|k, _| k.0 != handle);
    }

    pub fn remove_clip(&self, handle: u64) {
        let mut sh = lock(&self.shared);
        sh.clips.remove(&handle);
        sh.bindings.borrow_mut().retain(|k, _| k.1 != handle);
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

fn encode(states: &BTreeMap<u64, EntityState>, out: &mut Vec<u8>) {
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
}

fn decode(bytes: &[u8]) -> Result<BTreeMap<u64, EntityState>> {
    let mut r = Reader::new(bytes);
    let mut states = BTreeMap::new();
    if bytes.is_empty() {
        return Ok(states);
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
            });
        }
        states.insert(
            e,
            EntityState {
                skeleton,
                skin,
                mesh,
                pose,
                weights,
                player,
                graph: GraphState { asset, layers },
            },
        );
    }
    r.finish()?;
    Ok(states)
}

impl StateBlock for AnimState {
    fn save(&self, out: &mut Vec<u8>) {
        encode(&lock(&self.0).states, out);
    }

    fn decode(&self, bytes: &[u8]) -> Result<Box<dyn Any + Send>> {
        Ok(Box::new(decode(bytes)?))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let mut sh = lock(&self.0);
        sh.states = decoded
            .map(|d| {
                *d.downcast::<BTreeMap<u64, EntityState>>()
                    .expect("decoded by AnimState::decode")
            })
            .unwrap_or_default();
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

fn run(ctx: &mut SystemContext<'_>, shared: &Mutex<Shared>) -> Result<()> {
    let dt = ctx.time().dt;
    let skeletons = entities(ctx, 0)?;
    let with = |q: usize| -> Result<BTreeSet<u64>> {
        Ok(entities(ctx, q)?.iter().map(|e| e.to_bits()).collect())
    };
    let (players, graphs, blends, transforms) = (with(1)?, with(2)?, with(3)?, with(4)?);
    let chains = entities(ctx, 5)?;

    let mut sh = lock(shared);
    let mut work: Vec<Work> = Vec::with_capacity(skeletons.len());
    let mut fired: Vec<(Entity, Fired)> = Vec::new();
    for &e in &skeletons {
        let key = e.to_bits();
        let skeleton: Skeleton = ctx.get(e)?;
        let asset = sh
            .skeletons
            .get(&skeleton.asset)
            .cloned()
            .ok_or_else(|| missing(e, "og.Skeleton.asset", skeleton.asset))?;
        let mut state = sh
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
                player: PlayerState::default(),
                graph: GraphState::default(),
            });
        let mut player = players
            .contains(&key)
            .then(|| ctx.get::<AnimationPlayer>(e))
            .transpose()?;
        let mut graph = graphs
            .contains(&key)
            .then(|| ctx.get::<AnimationGraph>(e))
            .transpose()?;
        let blend = blends
            .contains(&key)
            .then(|| ctx.get::<Blendshapes>(e))
            .transpose()?;
        let mut transform = transforms
            .contains(&key)
            .then(|| ctx.get::<Transform>(e))
            .transpose()?;
        let mesh = blend.map_or(NONE, |b| b.mesh);
        let root = (skeleton.root_motion != ROOT_MOTION_OFF)
            .then(|| asset.joint_node(skeleton.skin, skeleton.root_joint))
            .flatten()
            .map(|n| RootMotion::new(&asset, n, skeleton.root_motion));

        let mut events: Vec<Fired> = Vec::new();
        let rest = Pose::rest(&asset);
        let (mut pose, mut delta) = (rest.clone(), Trs::IDENTITY);
        match graph.as_mut() {
            Some(g) => {
                let ga = sh
                    .graphs
                    .get(&g.asset)
                    .cloned()
                    .ok_or_else(|| missing(e, "og.AnimationGraph.asset", g.asset))?;
                let (gp, gd) = eval_graph(
                    &sh,
                    e,
                    skeleton.asset,
                    &asset,
                    &ga,
                    g,
                    &mut state.graph,
                    root.as_ref(),
                    mesh,
                    dt,
                    &mut events,
                )?;
                pose.lerp_to(&gp, g.weight.clamp(0.0, 1.0));
                delta = Trs::IDENTITY.lerp(&gd, g.weight.clamp(0.0, 1.0));
            }
            None => state.graph = GraphState::default(),
        }
        match player.as_mut() {
            Some(p) => {
                if let Some((pp, pd)) = eval_player(
                    &sh,
                    e,
                    skeleton.asset,
                    &asset,
                    p,
                    &mut state.player,
                    root.as_ref(),
                    mesh,
                    dt,
                    &mut events,
                )? {
                    let w = p.weight.clamp(0.0, 1.0);
                    pose.lerp_to(&pp, w);
                    delta = delta.lerp(&pd, w);
                }
            }
            None => state.player = PlayerState::default(),
        }
        if root.is_some() {
            if let Some(t) = transform.as_mut() {
                t.position += t.quaternion * (t.scale * delta.translation);
                t.quaternion = unit(t.quaternion * delta.rotation);
            }
        }
        let mut blend = blend;
        if let Some(b) = blend.as_mut() {
            if b.source == BLENDSHAPES_FROM_CLIPS {
                b.set_weights(pose.weights);
            }
            pose.weights = b.weights();
        }
        state.skin = skeleton.skin;
        state.mesh = mesh;
        for f in events {
            fired.push((e, f));
        }
        work.push(Work {
            entity: e,
            skeleton,
            asset,
            player,
            graph,
            blend,
            transform,
            state,
            pose,
        });
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
            solve(w, &mut chain)?;
        }
        solved.push((c, chain));
    }

    // Everything is computed: write it back.
    let mut states = BTreeMap::new();
    let mut outputs = BTreeMap::new();
    for mut w in work {
        let mut palette = Vec::new();
        w.asset
            .palette(w.skeleton.skin, &w.pose.nodes, &mut palette);
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
        outputs.insert(
            w.entity.to_bits(),
            AnimOutput {
                skin: w.state.skin,
                palette,
                mesh: w.state.mesh,
                weights: w.state.weights,
            },
        );
        states.insert(w.entity.to_bits(), w.state);
    }
    for (c, chain) in solved {
        ctx.set(c, chain)?;
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
    sh.outputs = outputs;
    Ok(())
}

fn solve(w: &mut Work, chain: &mut IkChain) -> Result<()> {
    let skin = w.skeleton.skin;
    let Some(tip) = w.asset.joint_node(skin, chain.tip) else {
        return Err(invalid(format!(
            "og.IkChain.tip {} is not a joint of skin {skin}",
            chain.tip
        )));
    };
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
    let to_model = |p: Vec3| -> Vec3 {
        match (chain.space, w.transform) {
            (IK_WORLD, Some(t)) => Trs::new(t.position, t.quaternion, t.scale)
                .inverse()
                .transform_point(p),
            _ => p,
        }
    };
    let target = to_model(chain.target);
    let weight = chain.weight.clamp(0.0, 1.0);
    let result = if chain.solver == IK_TWO_BONE {
        let pole = (chain.pole != Vec3::ZERO).then(|| to_model(chain.pole));
        ik::two_bone(&w.asset, &mut w.pose.nodes, &nodes, target, pole, weight)
    } else {
        ik::fabrik(
            &w.asset,
            &mut w.pose.nodes,
            &nodes,
            target,
            chain.iterations,
            chain.tolerance,
            weight,
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
    for (li, layer) in graph.layers.iter().enumerate() {
        let ls = &mut gs.layers[li];
        if ls.state as usize >= layer.states.len() {
            ls.state = layer.entry as u32;
        }
        for t in &layer.transitions {
            match t.from {
                Some(f) if f != ls.state as usize => continue,
                None if t.to == ls.state as usize => continue,
                _ => {}
            }
            if !t.conditions.iter().all(|c| c.holds(&params)) {
                continue;
            }
            if t.exit.is_some_and(|x| ls.phase.total() < x as f64) {
                continue;
            }
            ls.prev = ls.state;
            ls.prev_phase = ls.phase;
            ls.state = t.to as u32;
            ls.phase = Phase::default();
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
        let mut run_state = |state: usize, phase: &mut Phase, w: f32| -> Result<()> {
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
            let before = *phase;
            let wraps = if duration > 0.0 {
                phase.advance(dt * speed * st.speed / duration)
            } else {
                0
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
                    leaf.delta(skel, leaf_root)
                } else {
                    Trs::IDENTITY
                };
                layer_blend.push(leaf.pose(skel, leaf_root, mesh), d, lw * w);
                if layer.mode == LayerMode::Additive {
                    reference.push(leaf.reference(skel, mesh), Trs::IDENTITY, lw * w);
                }
            }
            Ok(())
        };
        let (cur, mut cur_phase) = (ls.state as usize, ls.phase);
        if ls.prev != NONE && alpha < 1.0 {
            let (prev, mut prev_phase) = (ls.prev as usize, ls.prev_phase);
            run_state(prev, &mut prev_phase, 1.0 - alpha)?;
            ls.prev_phase = prev_phase;
        }
        run_state(cur, &mut cur_phase, alpha)?;
        ls.phase = cur_phase;
        ls.time += dt * speed;
        if alpha >= 1.0 {
            ls.prev = NONE;
        }
        let layer_pose = layer_blend.pose.unwrap_or_else(|| Pose::rest(skel));
        match layer.mode {
            LayerMode::Override => pose.lerp_to(&layer_pose, layer_weight),
            LayerMode::Additive => {
                let r = reference.pose.unwrap_or_else(|| Pose::rest(skel));
                pose.add(&layer_pose, &r, layer_weight);
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
        blend.push(
            leaf.pose(skel, root, mesh),
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
    blend.push(leaf.pose(skel, root, mesh), leaf.delta(skel, root), alpha);
    ps.time = t1;
    ps.written = t1;
    comp.time = t1;
    comp.finished = !comp.looping
        && ((comp.speed >= 0.0 && t1 >= clip.duration) || (comp.speed < 0.0 && t1 <= 0.0));
    Ok(blend.pose.map(|p| (p, blend.delta)))
}
