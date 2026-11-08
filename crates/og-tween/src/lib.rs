//! Origin Engine tweens: eased, delayed, looping, yoyo and grouped animation
//! of any reflected numeric component field, advanced by one scheduled
//! system (`og.tween`) with a state block, so snapshots restore running
//! tweens and a replay animates the same bits.
//!
//! ```text
//! let tweens = og_tween::install(&mut world, &component_names)?;   // after the game's systems
//! let target = Target::resolve(world.schema(id)?, id.index(), entity.to_bits(), "position.y")?;
//! let h = tweens.start(Node::Leaf(Leaf::new(target, vec![2.0]).duration(0.3).ease(Ease::BackOut)), false, 0)?;
//! world.step()?;                       // writes the field; og.TweenCompleted when it ends
//! ```
//!
//! Time: a tween advances by the world's scaled tick (`TimeScale`: slow
//! motion, hit stop) unless it is `unscaled`. A tween of a dead entity, or
//! of a component the entity lost, ends without an event.

mod codec;
pub mod presets;
mod spring;
mod timeline;

pub use og_core::ease::{EASE_NAMES, EASES, Ease, StepPosition};
pub use spring::{Spring, SpringState};
pub use timeline::{Callbacks, Child, Item, Position, Stagger, StaggerFrom, Timeline};

use og_core::IdMap;
use og_core::reflect::FieldType;
use og_core::{Error, ErrorKind, Reflect, Result, Schema, Str64};
use og_ecs::{Entity, SchemaId, StateBlock, SystemContext, SystemDesc, World};
use std::any::Any;
use std::sync::{Arc, Mutex, MutexGuard};

/// The system's and the state block's name.
pub const SYSTEM_NAME: &str = "og.tween";

/// Sent when a tween (or a group) started with [`TweenSystem::start`] ends
/// on its own (not when cancelled, nor when its target goes away). Read in
/// the next tick, like every event.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.TweenCompleted")]
pub struct TweenCompleted {
    /// The handle `start` returned.
    pub tween: u64,
    /// The entity of its (first) target.
    pub entity: u64,
    /// The tag it was started with.
    pub tag: u32,
}

/// A timeline's callback (`onStart`, `onUpdate`, `onComplete`,
/// `onRepeat`, `.call`), by the name it was given: a system reads them as
/// any event, in the tick after.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.TweenEvent")]
pub struct TweenEvent {
    /// The timeline's handle.
    pub tween: u64,
    /// The tween's target entity (0 for the timeline's own and a call's).
    pub entity: u64,
    /// The name the callback was given.
    pub name: Str64,
    /// How far through it is (its cycle), 0 to 1.
    pub progress: f32,
    /// The timeline's tag.
    pub tag: u32,
}

/// How a field's lanes are stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lane {
    F32,
    F64,
    I32,
    U32,
    /// Four f32 interpolated as a rotation (normalised lerp, shortest arc).
    Quat,
}

impl Lane {
    fn width(self) -> usize {
        match self {
            Lane::F64 => 8,
            _ => 4,
        }
    }
}

/// What a tween writes: `lanes` consecutive values of one component's
/// packed bytes, from byte `offset`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Target {
    pub entity: u64,
    /// The component's schema id index.
    pub component: u32,
    pub offset: u32,
    pub lane: Lane,
    pub lanes: u32,
}

impl Target {
    /// The target of `path` (`position`, `position.y`, `color.r`, `opacity`)
    /// in a component with `schema`. Vector lanes are `x y z w` (or `r g b
    /// a`); a quaternion tweens whole. Refused for a field that is not
    /// numeric (bool, enum, handle, matrix, string) or does not exist.
    pub fn resolve(schema: &Schema, component: u32, entity: u64, path: &str) -> Result<Target> {
        let (field, lane) = match path.split_once('.') {
            Some((f, l)) => (f, Some(l)),
            None => (path, None),
        };
        let index = schema
            .fields
            .iter()
            .position(|f| f.name == field)
            .ok_or_else(|| {
                let names: Vec<_> = schema.fields.iter().map(|f| f.name.as_str()).collect();
                invalid(format!(
                    "{} has no field '{field}' (its fields: {})",
                    schema.name,
                    names.join(", ")
                ))
            })?;
        let offset: usize = schema.fields[..index].iter().map(|f| f.ty.size()).sum();
        let ty = schema.fields[index].ty;
        let (kind, count) = match ty {
            FieldType::F32 => (Lane::F32, 1),
            FieldType::F64 => (Lane::F64, 1),
            FieldType::I32 => (Lane::I32, 1),
            FieldType::U32 => (Lane::U32, 1),
            FieldType::Vec2 => (Lane::F32, 2),
            FieldType::Vec3 => (Lane::F32, 3),
            FieldType::Vec4 => (Lane::F32, 4),
            FieldType::Quat => (Lane::Quat, 4),
            _ => {
                return Err(invalid(format!(
                    "{}.{field} is a {ty:?} field; tweens animate numbers (f32, f64, i32, u32, vectors, colours, quaternions)",
                    schema.name
                )));
            }
        };
        let mut t = Target {
            entity,
            component,
            offset: offset as u32,
            lane: kind,
            lanes: count,
        };
        if let Some(l) = lane {
            let i = match l {
                "x" | "r" => 0,
                "y" | "g" => 1,
                "z" | "b" => 2,
                "w" | "a" => 3,
                _ => u32::MAX,
            };
            if count == 1 || kind == Lane::Quat || i >= count {
                return Err(invalid(format!(
                    "{}.{field} has no lane '{l}': a vector or colour field has x y z w (r g b a) up to its size",
                    schema.name
                )));
            }
            t.offset += 4 * i;
            t.lanes = 1;
        }
        Ok(t)
    }

    /// Read the target lanes from a packed component, for call-time capture.
    pub fn read(&self, bytes: &[u8]) -> Option<Vec<f32>> {
        let w = self.lane.width();
        (0..self.lanes as usize)
            .map(|i| {
                let at = self.offset as usize + i * w;
                let b = bytes.get(at..at + w)?;
                Some(match self.lane {
                    Lane::F32 | Lane::Quat => f32::from_le_bytes(b.try_into().ok()?),
                    Lane::F64 => f64::from_le_bytes(b.try_into().ok()?) as f32,
                    Lane::I32 => i32::from_le_bytes(b.try_into().ok()?) as f32,
                    Lane::U32 => u32::from_le_bytes(b.try_into().ok()?) as f32,
                })
            })
            .collect()
    }

    fn write(&self, bytes: &mut [u8], values: &[f32]) {
        let w = self.lane.width();
        for (i, v) in values.iter().enumerate().take(self.lanes as usize) {
            let at = self.offset as usize + i * w;
            let Some(slot) = bytes.get_mut(at..at + w) else {
                return;
            };
            match self.lane {
                Lane::F32 | Lane::Quat => slot.copy_from_slice(&v.to_le_bytes()),
                Lane::F64 => slot.copy_from_slice(&f64::from(*v).to_le_bytes()),
                Lane::I32 => slot.copy_from_slice(&(v.round() as i32).to_le_bytes()),
                Lane::U32 => slot.copy_from_slice(&(v.round().max(0.0) as u32).to_le_bytes()),
            }
        }
    }
}

fn invalid(reason: String) -> Error {
    Error::new(ErrorKind::Invalid, reason)
}

/// One tween of one target.
#[derive(Clone, Debug, PartialEq)]
pub struct Leaf {
    pub target: Target,
    /// The end value per lane (one value fills every lane).
    pub to: Vec<f32>,
    /// The start value, written from the tick the tween is started and held
    /// through its delay; `None` takes the field's value when the tween
    /// starts (after its delay).
    pub from: Option<Vec<f32>>,
    /// `to` is an offset from the start value.
    pub relative: bool,
    pub duration: f32,
    pub delay: f32,
    pub ease: Ease,
    /// Extra cycles: 0 plays once, n repeats n more times, -1 forever.
    pub loops: i32,
    /// Odd cycles play backwards.
    pub yoyo: bool,
    /// Seconds held between cycles (GSAP's `repeatDelay`).
    pub repeat_delay: f32,
    // Running state (in the state block).
    pub(crate) elapsed: f64,
    pub(crate) started: bool,
    pub(crate) start: Vec<f32>,
    pub(crate) end: Vec<f32>,
}

impl Leaf {
    /// Moves the leaf's absolute lanes on a place in the world by the
    /// shift (og.WorldOrigin, R12): `to` unless relative, `from`, and the
    /// running start and end, each lane along its field's axis
    /// ([`og_ecs::Shift::lane`]).
    fn shift(&mut self, s: &og_ecs::Shift) {
        let t = self.target;
        let width = t.lane.width() as u32;
        for i in 0..t.lanes {
            let d = s.lane(t.entity, t.component, t.offset + i * width);
            if d == 0.0 {
                continue;
            }
            let i = i as usize;
            let by = |v: &mut Vec<f32>, lanes: usize| {
                if v.len() == 1 && lanes > 1 {
                    *v = vec![v[0]; lanes];
                }
                if let Some(x) = v.get_mut(i) {
                    *x = (f64::from(*x) + d) as f32;
                }
            };
            let lanes = t.lanes as usize;
            if !self.relative {
                by(&mut self.to, lanes);
            }
            if let Some(f) = &mut self.from {
                by(f, lanes);
            }
            if self.started {
                by(&mut self.start, lanes);
                by(&mut self.end, lanes);
            }
        }
    }

    /// A 0.3 s power1.out tween of `target` to `to`.
    pub fn new(target: Target, to: Vec<f32>) -> Leaf {
        Leaf {
            target,
            to,
            from: None,
            relative: false,
            duration: 0.3,
            delay: 0.0,
            ease: Ease::QuadOut,
            loops: 0,
            yoyo: false,
            repeat_delay: 0.0,
            elapsed: 0.0,
            started: false,
            start: Vec::new(),
            end: Vec::new(),
        }
    }
    pub fn duration(mut self, s: f32) -> Leaf {
        self.duration = s;
        self
    }
    pub fn delay(mut self, s: f32) -> Leaf {
        self.delay = s;
        self
    }
    pub fn ease(mut self, e: Ease) -> Leaf {
        self.ease = e;
        self
    }
    pub fn from(mut self, v: Vec<f32>) -> Leaf {
        self.from = Some(v);
        self
    }
    pub fn relative(mut self, r: bool) -> Leaf {
        self.relative = r;
        self
    }
    pub fn loops(mut self, n: i32, yoyo: bool) -> Leaf {
        self.loops = n;
        self.yoyo = yoyo;
        self
    }

    /// Seconds held between cycles.
    pub fn repeat_delay(mut self, s: f32) -> Leaf {
        self.repeat_delay = s;
        self
    }

    /// Seconds from start to end, delay included; infinite for `loops -1`.
    pub fn length(&self) -> f32 {
        if self.loops < 0 {
            return f32::INFINITY;
        }
        self.delay
            + self.duration * (self.loops as f32 + 1.0)
            + self.repeat_delay * self.loops as f32
    }

    /// Whether `elapsed` reached the end (within a microsecond, so a tween
    /// of n ticks ends on its n-th tick whatever the rounding of dt).
    fn at_end(&self) -> bool {
        self.elapsed + 1e-6 >= f64::from(self.length())
    }

    /// Whether it has played to its end.
    pub fn ended(&self) -> bool {
        self.started && self.at_end()
    }

    /// Refuses an invalid ease, values of the wrong length or non-finite
    /// times.
    pub fn check(&self) -> Result<()> {
        if !self.ease.is_valid() {
            return Err(invalid("ease needs finite bezier controls with x in 0..=1, or positive steps (jump-none needs at least 2)".into()));
        }
        let n = self.target.lanes as usize;
        let ok = |v: &[f32]| (v.len() == 1 || v.len() == n) && v.iter().all(|x| x.is_finite());
        if !ok(&self.to) || self.from.as_deref().is_some_and(|f| !ok(f)) {
            return Err(invalid(format!(
                "a tween of {n} lane(s) takes 1 or {n} finite values for `to` and `from`"
            )));
        }
        if !(self.duration.is_finite() && self.duration >= 0.0)
            || !(self.delay.is_finite() && self.delay >= 0.0)
            || !(self.repeat_delay.is_finite() && self.repeat_delay >= 0.0)
        {
            return Err(invalid(
                "duration, delay and repeatDelay are finite seconds, 0 or more".into(),
            ));
        }
        if self.loops < -1 {
            return Err(invalid("loops is -1 (forever), 0 (once) or more".into()));
        }
        Ok(())
    }

    fn widen(&self, v: &[f32]) -> Vec<f32> {
        if v.len() == 1 {
            vec![v[0]; self.target.lanes as usize]
        } else {
            v.to_vec()
        }
    }

    /// The value at `elapsed` (once the start and end are known).
    fn value(&self) -> Vec<f32> {
        let t = (self.elapsed - f64::from(self.delay)).max(0.0) as f32;
        let d = self.duration;
        let cycles = if self.loops < 0 {
            f32::INFINITY
        } else {
            self.loops as f32 + 1.0
        };
        let (k, p) = if d <= 0.0 || self.at_end() {
            (
                if cycles.is_finite() {
                    cycles - 1.0
                } else {
                    0.0
                },
                1.0,
            )
        } else if self.repeat_delay > 0.0 {
            // Each cycle is its duration, then the hold at its end.
            let cycle = d + self.repeat_delay;
            let k = (t / cycle).floor();
            (k, ((t - k * cycle) / d).min(1.0))
        } else {
            let k = (t / d).floor();
            (k, (t - k * d) / d)
        };
        let backwards = self.yoyo && (k as i64) % 2 == 1;
        let f = self.ease.apply(if backwards { 1.0 - p } else { p });
        let (a, b) = (&self.start, &self.end);
        if self.target.lane == Lane::Quat && a.len() == 4 && b.len() == 4 {
            return nlerp(a, b, f);
        }
        a.iter().zip(b).map(|(x, y)| x + (y - x) * f).collect()
    }
}

fn nlerp(a: &[f32], b: &[f32], f: f32) -> Vec<f32> {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let sign = if dot < 0.0 { -1.0 } else { 1.0 };
    let v: Vec<f32> = a
        .iter()
        .zip(b)
        .map(|(x, y)| x + (sign * y - x) * f)
        .collect();
    let len = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if len > 0.0 {
        v.iter().map(|x| x / len).collect()
    } else {
        a.to_vec()
    }
}

/// How a group runs its children.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupKind {
    /// One after another; the time one leaves over goes to the next.
    Sequence,
    /// All at once; done when the last one is.
    Parallel,
}

/// A tween or a group of tweens.
#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Leaf(Leaf),
    Group(Group),
    /// A GSAP-shaped timeline (`og.tween.timeline`).
    Timeline(Box<Timeline>),
}

/// Tweens run one after another or together, the whole repeated `loops` times.
#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    pub kind: GroupKind,
    pub children: Vec<Leaf>,
    /// Extra runs of the whole group: 0 once, -1 forever.
    pub loops: i32,
    /// Running: the sequence's current child and the runs completed.
    pub(crate) current: u32,
    pub(crate) runs: u32,
}

impl Node {
    /// A group of `children` (each with its own delay, duration and loops).
    pub fn group(kind: GroupKind, children: Vec<Leaf>, loops: i32) -> Node {
        Node::Group(Group {
            kind,
            children,
            loops,
            current: 0,
            runs: 0,
        })
    }

    fn entity(&self) -> u64 {
        match self {
            Node::Leaf(l) => l.target.entity,
            Node::Group(g) => g.children.first().map_or(0, |c| c.target.entity),
            Node::Timeline(t) => t.entity(),
        }
    }

    fn leaves(&self) -> Vec<&Leaf> {
        match self {
            Node::Leaf(l) => vec![l],
            Node::Group(g) => g.children.iter().collect(),
            Node::Timeline(t) => t.leaves(),
        }
    }

    fn leaves_mut(&mut self) -> Vec<&mut Leaf> {
        match self {
            Node::Leaf(l) => vec![l],
            Node::Group(g) => g.children.iter_mut().collect(),
            Node::Timeline(t) => t.leaves_mut(),
        }
    }
}

/// A started tween: its node, how it advances, and its tag.
#[derive(Clone, Debug, PartialEq)]
pub struct Running {
    pub node: Node,
    /// Advance by `Time::dt`, ignoring slow motion and hit stop.
    pub unscaled: bool,
    pub tag: u32,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub(crate) struct State {
    pub next: u64,
    pub running: IdMap<u64, Running>,
}

/// Starts, cancels and inspects the tweens of one world.
#[derive(Clone)]
pub struct TweenSystem {
    shared: Arc<Mutex<State>>,
}

fn lock(m: &Mutex<State>) -> MutexGuard<'_, State> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// Registers `og.TweenCompleted` and schedules `og.tween` at this point of
/// the schedule (after the systems that start tweens, before animation and
/// physics), declaring a write to every component in `components` (the
/// possible targets), with its state block.
pub fn install(world: &mut World, components: &[String]) -> Result<TweenSystem> {
    world.register::<TweenCompleted>()?;
    world.register::<TweenEvent>()?;
    let shared = Arc::new(Mutex::new(State::default()));
    let mut desc = SystemDesc::new(SYSTEM_NAME);
    desc.writes = components.to_vec();
    desc.writes.push(TweenCompleted::NAME.to_string());
    desc.writes.push(TweenEvent::NAME.to_string());
    let s = shared.clone();
    world.add_system(desc, move |ctx| run(ctx, &s))?;
    world.add_state_block(SYSTEM_NAME, Box::new(TweenState(shared.clone())))?;
    Ok(TweenSystem { shared })
}

impl TweenSystem {
    /// Starts a tween or a group; the handle (never 0) names it in
    /// [`TweenCompleted`], [`Self::cancel`] and [`Self::is_active`]. It
    /// first moves in the next run of `og.tween` (this tick, when started by
    /// a system before it).
    pub fn start(&self, node: Node, unscaled: bool, tag: u32) -> Result<u64> {
        for l in node.leaves() {
            l.check()?;
        }
        if let Node::Group(g) = &node {
            if g.children.is_empty() {
                return Err(invalid("a group needs at least one tween".into()));
            }
            if g.loops < -1 {
                return Err(invalid("loops is -1 (forever), 0 (once) or more".into()));
            }
        }
        let mut st = lock(&self.shared);
        st.next += 1;
        let handle = st.next | (1u64 << 32);
        st.running.insert(
            handle,
            Running {
                node,
                unscaled,
                tag,
            },
        );
        Ok(handle)
    }

    /// Stops a tween where it is (no event). `false` when it is not running.
    pub fn cancel(&self, handle: u64) -> bool {
        lock(&self.shared).running.remove(&handle).is_some()
    }

    /// Stops every tween with a target on `entity` (no events); how many.
    /// A timeline loses its tweens of the entity and plays on.
    pub fn cancel_entity(&self, entity: u64) -> u32 {
        let mut st = lock(&self.shared);
        let mut n = 0;
        for r in st.running.values_mut() {
            if let Node::Timeline(t) = &mut r.node {
                n += t.drop_entity(entity);
            }
        }
        let before = st.running.len();
        st.running.retain(|_, r| {
            matches!(r.node, Node::Timeline(_))
                || !r.node.leaves().iter().any(|l| l.target.entity == entity)
        });
        n + (before - st.running.len()) as u32
    }

    /// Runs `f` on running timeline `handle`; `None` when it is not one.
    /// What `f` changes is rendered in the next run of `og.tween`.
    pub fn with_timeline<R>(&self, handle: u64, f: impl FnOnce(&mut Timeline) -> R) -> Option<R> {
        match lock(&self.shared)
            .running
            .get_mut(&handle)
            .map(|r| &mut r.node)
        {
            Some(Node::Timeline(t)) => Some(f(t)),
            _ => None,
        }
    }

    /// Takes a running tween, group or timeline out to nest it in a
    /// timeline (`.add`): it stops running on its own, without an event.
    pub fn take(&self, handle: u64) -> Option<Node> {
        lock(&self.shared).running.remove(&handle).map(|r| r.node)
    }

    pub fn is_active(&self, handle: u64) -> bool {
        lock(&self.shared).running.contains_key(&handle)
    }

    /// Running tweens and groups.
    pub fn count(&self) -> usize {
        lock(&self.shared).running.len()
    }
}

/// Captures a leaf's start and end from its target's current value (its
/// first write); `false` when the target is gone.
fn begin_leaf(ctx: &SystemContext<'_>, leaf: &mut Leaf) -> bool {
    let Some(bytes) = target_bytes(ctx, leaf) else {
        return false;
    };
    let Some(current) = leaf.target.read(&bytes) else {
        return false;
    };
    leaf.start = match &leaf.from {
        Some(f) => leaf.widen(f),
        None => current,
    };
    let to = leaf.widen(&leaf.to);
    leaf.end = if leaf.relative {
        leaf.start.iter().zip(&to).map(|(a, b)| a + b).collect()
    } else {
        to
    };
    leaf.started = true;
    true
}

fn target_bytes(ctx: &SystemContext<'_>, leaf: &Leaf) -> Option<Vec<u8>> {
    let e = Entity::from_bits(leaf.target.entity);
    let id = SchemaId::from_index(leaf.target.component);
    if ctx.is_alive(e) {
        ctx.get_bytes(e, id).ok()
    } else {
        None
    }
}

/// Writes `value` into a leaf's target; `false` when it is gone.
fn write_leaf(ctx: &SystemContext<'_>, leaf: &Leaf, value: &[f32]) -> Result<bool> {
    let Some(mut bytes) = target_bytes(ctx, leaf) else {
        return Ok(false);
    };
    leaf.target.write(&mut bytes, value);
    let e = Entity::from_bits(leaf.target.entity);
    ctx.set_bytes(e, SchemaId::from_index(leaf.target.component), &bytes)?;
    Ok(true)
}

/// Advances a leaf by `dt` and writes its value; `Ok(true)` when it has
/// ended. Clears `alive` when its target is gone.
fn step_leaf(ctx: &SystemContext<'_>, leaf: &mut Leaf, dt: f32, alive: &mut bool) -> Result<bool> {
    leaf.elapsed += f64::from(dt);
    // A tween with an explicit `from` holds it through its delay (a preset's
    // impact shows on the tick it starts, even inside a hit stop); one
    // without waits untouched, taking the field's value when it starts.
    let waiting = leaf.elapsed + 1e-6 < f64::from(leaf.delay);
    if waiting && leaf.from.is_none() {
        return Ok(false);
    }
    if !leaf.started && !begin_leaf(ctx, leaf) {
        *alive = false;
        return Ok(false);
    }
    let value = if waiting {
        leaf.start.clone()
    } else {
        leaf.value()
    };
    if !write_leaf(ctx, leaf, &value)? {
        *alive = false;
        return Ok(false);
    }
    Ok(!waiting && leaf.at_end())
}

/// A timeline's reads, writes and callbacks in one run of `og.tween`.
struct WorldIo<'a, 'b> {
    ctx: &'a SystemContext<'b>,
    tween: u64,
    tag: u32,
    events: &'a mut Vec<TweenEvent>,
    error: Option<Error>,
}

impl timeline::Io for WorldIo<'_, '_> {
    fn begin(&mut self, leaf: &mut Leaf) -> bool {
        begin_leaf(self.ctx, leaf)
    }

    fn write(&mut self, leaf: &Leaf, value: &[f32]) {
        if let Err(e) = write_leaf(self.ctx, leaf, value) {
            self.error.get_or_insert(e);
        }
    }

    fn fire(&mut self, name: &Option<String>, entity: u64, progress: f32) {
        let Some(name) = name.as_deref().filter(|n| !n.is_empty()) else {
            return;
        };
        // Names are checked when they are given (at most 63 bytes).
        let name = Str64::new(name).unwrap_or_default();
        self.events.push(TweenEvent {
            tween: self.tween,
            entity,
            name,
            progress,
            tag: self.tag,
        });
    }
}

/// Advances a group; `Ok(true)` when its last run ended.
fn step_group(ctx: &SystemContext<'_>, g: &mut Group, dt: f32, alive: &mut bool) -> Result<bool> {
    let done = match g.kind {
        GroupKind::Parallel => {
            let mut all = true;
            for c in g.children.iter_mut() {
                if c.ended() {
                    continue;
                }
                all &= step_leaf(ctx, c, dt, alive)?;
                if !*alive {
                    return Ok(false);
                }
            }
            all
        }
        GroupKind::Sequence => {
            let mut left = dt;
            while let Some(c) = g.children.get_mut(g.current as usize) {
                if !step_leaf(ctx, c, left, alive)? || !*alive {
                    break;
                }
                left = (c.elapsed - f64::from(c.length())).max(0.0) as f32;
                g.current += 1;
                if left <= 0.0 {
                    break;
                }
            }
            g.current as usize >= g.children.len()
        }
    };
    if done && (g.loops < 0 || (g.runs as i32) < g.loops) {
        // Again over the same spans: each child keeps the start and end it
        // resolved in the first run.
        g.runs += 1;
        for c in &mut g.children {
            c.elapsed = 0.0;
        }
        g.current = 0;
        return Ok(false);
    }
    Ok(done)
}

fn run(ctx: &mut SystemContext<'_>, shared: &Mutex<State>) -> Result<()> {
    let scaled = ctx.scaled_dt();
    let raw = ctx.time().dt;
    let groups = ctx.time_groups();
    let mut st = lock(shared);
    let mut ended: Vec<u64> = Vec::new();
    let mut completed: Vec<TweenCompleted> = Vec::new();
    let mut events: Vec<TweenEvent> = Vec::new();
    for (&handle, r) in st.running.iter_mut() {
        let entity = r.node.entity();
        // A scaled tween also runs at its target's time group's pace.
        let dt = if r.unscaled {
            raw
        } else {
            ctx.group_dt(groups.as_ref(), Entity::from_bits(entity), scaled)
        };
        let mut alive = true;
        let done = match &mut r.node {
            Node::Leaf(l) => step_leaf(ctx, l, dt, &mut alive)?,
            Node::Group(g) => step_group(ctx, g, dt, &mut alive)?,
            Node::Timeline(t) => {
                let mut io = WorldIo {
                    ctx,
                    tween: handle,
                    tag: r.tag,
                    events: &mut events,
                    error: None,
                };
                let done = t.step(&mut io, dt);
                if let Some(e) = io.error {
                    return Err(e);
                }
                // A timeline stays when it completes (to be reversed,
                // restarted or extended) unless it auto-kills.
                if done && !t.auto_kill {
                    completed.push(TweenCompleted {
                        tween: handle,
                        entity,
                        tag: r.tag,
                    });
                    continue;
                }
                done
            }
        };
        if !alive {
            ended.push(handle);
        } else if done {
            ended.push(handle);
            completed.push(TweenCompleted {
                tween: handle,
                entity,
                tag: r.tag,
            });
        }
    }
    for h in ended {
        st.running.remove(&h);
    }
    drop(st);
    for e in events {
        ctx.send(e)?;
    }
    for c in completed {
        ctx.send(c)?;
    }
    Ok(())
}

struct TweenState(Arc<Mutex<State>>);

impl StateBlock for TweenState {
    fn save(&self, out: &mut Vec<u8>) {
        codec::encode(&lock(&self.0), out);
    }

    fn decode(&self, bytes: &[u8]) -> Result<Box<dyn Any + Send>> {
        Ok(Box::new(codec::decode(bytes)?))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        *lock(&self.0) = decoded
            .map(|d| {
                *d.downcast::<State>()
                    .expect("decoded by TweenState::decode")
            })
            .unwrap_or_default();
    }

    /// Every leaf that tweens a place in the world (a root's
    /// og.Transform.position, og.Camera.position) moves with it.
    fn shift(&mut self, s: &og_ecs::Shift) {
        for r in lock(&self.0).running.values_mut() {
            r.node.leaves_mut().into_iter().for_each(|l| l.shift(s));
        }
    }
}
