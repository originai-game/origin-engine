//! Reactivity: change detection, add and remove hooks, and state machines
//! with the systems that run in, on entering and on leaving their values.
//!
//! **Change ticks.** The world counts system runs: as a tick begins its
//! change tick moves on once (the tick's start), and again before each
//! system that runs. A tracked component's row records the change tick it
//! was added at and the change tick its value last changed at; a system
//! remembers the change tick of its last run, so a filter asks "after
//! that". Nothing reads a clock: the counter is world state, saved and
//! restored with the world.
//!
//! **What counts as a change.** A component is compared with the value it
//! had when last compared, after each system for the components it declared
//! writes of, after each hook likewise, and as each tick begins for every
//! tracked component (what was written between ticks, through a world
//! query's views or otherwise). A write that leaves the value as it was is
//! not a change. Spawning with a component, or inserting one, adds it (and
//! changes it) at the change tick of the system whose commands applied it,
//! or, between ticks, at the next tick's start. A removal (a remove or a
//! despawn) is logged with its change tick; the log keeps a tick's removals
//! through the next tick, as events are kept.
//!
//! **Tracking** is per component and turns on when a system's query filters
//! on it ([`crate::QueryDesc::changed`], `added`, `removed`) or with
//! [`World::track`]. A world that tracks nothing and has no state machine
//! saves and hashes exactly as before this existed: the reactive state is
//! the `og.ecs` entry of the snapshot's state blocks, written only when it
//! has something to hold.
//!
//! **Hooks** ([`World::add_hook`]) run when a component is added to an
//! entity (after it is in place) or removed from one (before it goes, so
//! the hook reads it), wherever the change comes from: a system's commands
//! as they apply, or a call between ticks. They run in registration order;
//! a hook's own structural changes apply when it returns, and may run
//! hooks in turn, at most 16 deep. Restores run none.
//!
//! **State machines** ([`World::add_state`]) hold one of their values.
//! [`World::set_state`] (or a system's) asks for a transition, which happens
//! as the next tick begins: the systems that run on exiting the old value,
//! in schedule order, then the state takes the new value, then the systems
//! that run on entering it, then the tick's systems, of which those bound
//! to a state run only while it holds their value. The first tick enters
//! every state's initial value.

use crate::entity::Entity;
use crate::registry::SchemaId;
use crate::schedule::{System, SystemContext};
use crate::snapshot::{Reader, Writer, bad};
use crate::world::World;
use og_core::handle::RawHandle;
use og_core::{Error, ErrorKind, Result};

/// The name of the reactive state among the snapshot's state blocks.
pub(crate) const BLOCK: &str = "og.ecs";
/// The reactive block's byte version.
const VERSION: u32 = 1;
/// How deep hooks may run hooks.
const MAX_HOOK_DEPTH: u32 = 16;

/// When a system bound to a state machine runs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StateWhen {
    /// Every tick while the state holds the value.
    #[default]
    During,
    /// Once, as a tick begins, when the state enters the value (the first
    /// tick, for its initial value).
    Enter,
    /// Once, as a tick begins, when the state leaves the value.
    Exit,
}

/// A system's binding to a state machine's value ([`crate::SystemDesc::state`]).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StateRun {
    /// The state machine, by name ([`World::add_state`]).
    pub state: String,
    /// One of its values.
    pub value: String,
    pub when: StateWhen,
}

/// A state machine: a name, its values, and the one it starts in.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StateDesc {
    pub name: String,
    /// At least one, each unique and non-empty.
    pub values: Vec<String>,
    /// The value it starts in; empty: the first.
    pub initial: String,
}

/// What a hook reacts to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HookEvent {
    /// The component was added: spawned with, or inserted into an entity
    /// without it. Runs after it is in place.
    #[default]
    Add,
    /// The component is being removed: removed, or its entity despawned.
    /// Runs before it goes, so the hook reads it.
    Remove,
}

/// A hook's declaration: a name, the component and event it reacts to, and
/// what it reads and writes (checked as a system's declarations are).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HookDesc {
    pub name: String,
    pub component: String,
    pub on: HookEvent,
    pub reads: Vec<String>,
    pub writes: Vec<String>,
}

pub(crate) type HookFn = Box<dyn Fn(&mut SystemContext<'_>, Entity) -> Result<()> + Send>;
pub(crate) type ConditionFn = Box<dyn Fn(&mut SystemContext<'_>) -> Result<bool> + Send>;

pub(crate) struct Hook {
    pub component: SchemaId,
    pub on: HookEvent,
    /// Its access, as a system without queries.
    pub access: System,
    pub run: HookFn,
}

pub(crate) struct StateMachine {
    pub name: String,
    pub values: Vec<String>,
    pub initial: u32,
    pub current: u32,
    /// A transition asked for, applied as the next tick begins.
    pub pending: Option<u32>,
    /// Whether the first tick has entered the initial value.
    pub entered: bool,
}

/// A transition the tick in progress makes: the state, the value it
/// leaves (none for the first tick's entry) and the one it enters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Transition {
    pub state: usize,
    pub from: Option<u32>,
    pub to: u32,
}

/// One step of a tick: a system, or a state taking its new value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Slot {
    Run(usize),
    Set(usize),
}

/// The world's reactive state.
#[derive(Default)]
pub(crate) struct Reactive {
    /// Per type id: whether its changes are tracked.
    pub tracked: Vec<bool>,
    /// Per type id: removals since the start of the previous tick, in
    /// order (entity, change tick).
    pub removed: Vec<Vec<(Entity, u64)>>,
    /// The change tick of the system running (or the tick's start), and of
    /// the last one before a step ends.
    pub change_tick: u64,
    /// The change tick the last tick began at.
    pub step_start: u64,
    /// Whether a step is running: changes then take the running change
    /// tick, else the next tick's start.
    pub in_step: bool,
    pub states: Vec<StateMachine>,
    /// The transitions of the tick in progress.
    pub transitions: Vec<Transition>,
    pub hooks: Vec<Hook>,
    pub hook_depth: u32,
}

impl Reactive {
    /// Whether the world holds reactive state a snapshot must carry.
    pub fn enabled(&self) -> bool {
        self.tracked.iter().any(|&t| t) || !self.states.is_empty()
    }

    pub fn is_tracked(&self, id: SchemaId) -> bool {
        self.tracked.get(id.0 as usize).copied().unwrap_or(false)
    }
}

impl World {
    // ---- change detection ---------------------------------------------------

    /// The change tick changes are stamped with now: the running system's
    /// during a step, else the next tick's start.
    pub(crate) fn stamp(&self) -> u64 {
        if self.reactive.in_step {
            self.reactive.change_tick
        } else {
            self.reactive.change_tick + 1
        }
    }

    /// The world's change tick: it moves on once as each tick begins and
    /// once before each system runs.
    pub fn change_tick(&self) -> u64 {
        self.reactive.change_tick
    }

    /// Tracks the changes of component `id` from now on: rows present now
    /// count as added and changed before the first tick. Systems whose
    /// queries filter on a component track it when they are added.
    pub fn track(&mut self, id: SchemaId) -> Result<()> {
        self.registry.check(id)?;
        let i = id.0 as usize;
        if self.reactive.tracked.len() <= i {
            self.reactive.tracked.resize(i + 1, false);
            self.reactive.removed.resize_with(i + 1, Vec::new);
        }
        if self.reactive.tracked[i] {
            return Ok(());
        }
        self.reactive.tracked[i] = true;
        for arch in &mut self.archetypes {
            if let Some(position) = arch.position(id) {
                arch.track(position);
            }
        }
        Ok(())
    }

    /// Whether component `id`'s changes are tracked.
    pub fn is_tracked(&self, id: SchemaId) -> bool {
        self.reactive.is_tracked(id)
    }

    /// Ticks for the tracked types of archetype `a` (made, or given a type,
    /// after tracking began).
    pub(crate) fn track_archetype(&mut self, a: usize) {
        let arch = &mut self.archetypes[a];
        for position in 0..arch.types.len() {
            if self.reactive.is_tracked(arch.types[position]) {
                arch.track(position);
            }
        }
    }

    /// Compares the tracked components among `types` (every tracked one
    /// when `None`) with their last compared values, stamping a row that
    /// differs with `stamp`.
    pub(crate) fn diff(&mut self, types: Option<&[SchemaId]>, stamp: u64) {
        if !self.reactive.tracked.iter().any(|&t| t) {
            return;
        }
        for arch in &mut self.archetypes {
            for position in 0..arch.types.len() {
                let id = arch.types[position];
                if arch.ticks[position].is_some()
                    && types.is_none_or(|t| t.binary_search(&id).is_ok())
                {
                    arch.diff(position, stamp);
                }
            }
        }
    }

    /// Logs the removal of the tracked components among `types` from `e`.
    pub(crate) fn log_removed(&mut self, e: Entity, types: &[SchemaId]) {
        let stamp = self.stamp();
        for id in types {
            if self.reactive.is_tracked(*id) {
                self.reactive.removed[id.0 as usize].push((e, stamp));
            }
        }
    }

    /// Entities whose `id` was removed after change tick `since`.
    pub(crate) fn removed_since(&self, id: SchemaId, since: u64) -> Vec<Entity> {
        let mut out: Vec<Entity> = self
            .reactive
            .removed
            .get(id.0 as usize)
            .map(|log| {
                log.iter()
                    .filter(|(_, t)| *t > since)
                    .map(|(e, _)| *e)
                    .collect()
            })
            .unwrap_or_default();
        og_core::sort::sort(&mut out);
        out.dedup();
        out
    }

    /// Entities that lost component `T`'s type `id` since the start of the
    /// last tick (between ticks), in entity order, despawned ones included.
    pub fn removed(&self, id: SchemaId) -> Result<Vec<Entity>> {
        self.registry.check(id)?;
        Ok(self.removed_since(id, self.reactive.step_start.saturating_sub(1)))
    }

    /// The change tick filters of a query made between ticks read after:
    /// what changed since the last tick began.
    pub(crate) fn world_since(&self) -> u64 {
        self.reactive.step_start.saturating_sub(1)
    }

    // ---- hooks --------------------------------------------------------------

    /// Adds a hook: `run` is called with the entity whenever component
    /// `desc.component` is added to one (`HookEvent::Add`, after it is in
    /// place) or removed from one (`HookEvent::Remove`, before it goes;
    /// despawning removes every component). Hooks run in the order they were
    /// added, with what `desc` declares (as a system: reads, writes, and
    /// structural changes as commands applied when the hook returns, which
    /// may run hooks in turn, at most 16 deep). Refused for an unknown
    /// component or declared type, or a taken name.
    pub fn add_hook<F>(&mut self, desc: HookDesc, run: F) -> Result<()>
    where
        F: Fn(&mut SystemContext<'_>, Entity) -> Result<()> + Send + 'static,
    {
        let component = self.registry.id_by_name(&desc.component).map_err(|_| {
            Error::new(
                ErrorKind::NotFound,
                format!(
                    "hook '{}' reacts to unknown component '{}'",
                    desc.name, desc.component
                ),
            )
        })?;
        if self
            .reactive
            .hooks
            .iter()
            .any(|h| h.access.name == desc.name)
        {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("a hook named '{}' is already added", desc.name),
            ));
        }
        let access = System::resolve(
            self,
            crate::SystemDesc {
                name: desc.name.clone(),
                reads: desc.reads,
                writes: desc.writes,
                ..Default::default()
            },
            Box::new(|_| Ok(())),
        )?;
        self.reactive.hooks.push(Hook {
            component,
            on: desc.on,
            access,
            run: Box::new(run),
        });
        Ok(())
    }

    /// Hook names in run order, with the component and event each reacts to.
    pub fn hooks(&self) -> Vec<(String, String, HookEvent)> {
        self.reactive
            .hooks
            .iter()
            .map(|h| {
                (
                    h.access.name.clone(),
                    self.registry.name(h.component).to_owned(),
                    h.on,
                )
            })
            .collect()
    }

    /// Runs the hooks of `on` for the components `types` of `e`, each after
    /// the last's commands applied.
    pub(crate) fn fire(&mut self, on: HookEvent, e: Entity, types: &[SchemaId]) -> Result<()> {
        if self.reactive.hooks.is_empty() {
            return Ok(());
        }
        for i in 0..self.reactive.hooks.len() {
            let hook = &self.reactive.hooks[i];
            if hook.on != on || !types.contains(&hook.component) {
                continue;
            }
            // An earlier hook's commands may have taken it away.
            let component = hook.component;
            if !self.is_alive(e)
                || self
                    .location(e)
                    .map(|l| !self.archetypes[l.archetype as usize].has(component))
                    .unwrap_or(true)
            {
                continue;
            }
            if self.reactive.hook_depth >= MAX_HOOK_DEPTH {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    format!(
                        "hook '{}': hooks ran hooks {MAX_HOOK_DEPTH} deep (a hook's change keeps setting off hooks)",
                        hook.access.name
                    ),
                ));
            }
            let mut commands = Vec::new();
            let result = {
                let hook = &self.reactive.hooks[i];
                let mut ctx = SystemContext::new(self, &hook.access, &mut commands);
                (hook.run)(&mut ctx, e)
            };
            let name = self.reactive.hooks[i].access.name.clone();
            let writes = self.reactive.hooks[i].access.writes.clone();
            self.diff(Some(&writes), self.stamp());
            if let Err(err) = result {
                self.discard(&mut commands);
                return Err(err.context(format!("hook '{name}'")));
            }
            self.reactive.hook_depth += 1;
            let applied = self.apply(&mut commands);
            self.reactive.hook_depth -= 1;
            if let Err(err) = applied {
                self.discard(&mut commands);
                return Err(err.context(format!("hook '{name}'")));
            }
        }
        Ok(())
    }

    // ---- state machines -----------------------------------------------------

    /// Adds a state machine. Refused for an empty or taken name, no values,
    /// an empty or repeated value, or an initial value it does not have.
    pub fn add_state(&mut self, desc: StateDesc) -> Result<()> {
        let invalid = |m: String| Err(Error::new(ErrorKind::Invalid, m));
        if desc.name.is_empty() {
            return invalid("a state machine needs a name".into());
        }
        if self.reactive.states.iter().any(|s| s.name == desc.name) {
            return invalid(format!("a state machine named '{}' exists", desc.name));
        }
        if desc.values.is_empty() {
            return invalid(format!("state machine '{}' has no values", desc.name));
        }
        for (i, v) in desc.values.iter().enumerate() {
            if v.is_empty() || desc.values[..i].contains(v) {
                return invalid(format!(
                    "state machine '{}': the value '{v}' is empty or repeated",
                    desc.name
                ));
            }
        }
        let current = if desc.initial.is_empty() {
            0
        } else {
            match desc.values.iter().position(|v| *v == desc.initial) {
                Some(i) => i as u32,
                None => {
                    return invalid(format!(
                        "state machine '{}' has no value '{}' to start in",
                        desc.name, desc.initial
                    ));
                }
            }
        };
        self.reactive.states.push(StateMachine {
            name: desc.name,
            values: desc.values,
            initial: current,
            current,
            pending: None,
            entered: false,
        });
        Ok(())
    }

    pub(crate) fn state_index(&self, name: &str) -> Result<usize> {
        self.reactive
            .states
            .iter()
            .position(|s| s.name == name)
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::NotFound,
                    format!("no state machine named '{name}'"),
                )
            })
    }

    pub(crate) fn state_value(&self, state: usize, value: &str) -> Result<u32> {
        let s = &self.reactive.states[state];
        s.values
            .iter()
            .position(|v| v == value)
            .map(|i| i as u32)
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::NotFound,
                    format!(
                        "state machine '{}' has no value '{value}' (its values: {})",
                        s.name,
                        s.values.join(", ")
                    ),
                )
            })
    }

    /// A state machine's current value.
    pub fn state(&self, name: &str) -> Result<&str> {
        let s = &self.reactive.states[self.state_index(name)?];
        Ok(&s.values[s.current as usize])
    }

    /// The value a state machine moves to as the next tick begins, if a
    /// transition is asked for.
    pub fn next_state(&self, name: &str) -> Result<Option<&str>> {
        let s = &self.reactive.states[self.state_index(name)?];
        Ok(s.pending.map(|p| s.values[p as usize].as_str()))
    }

    /// Asks a state machine to move to `value` as the next tick begins (the
    /// last ask before then wins; asking for the value it holds cancels a
    /// transition asked for). Refused for an unknown state or value.
    pub fn set_state(&mut self, name: &str, value: &str) -> Result<()> {
        let state = self.state_index(name)?;
        let v = self.state_value(state, value)?;
        let s = &mut self.reactive.states[state];
        s.pending = (v != s.current).then_some(v);
        Ok(())
    }

    /// The state machines' names, values and current values, in added order.
    pub fn states(&self) -> Vec<(String, Vec<String>, String)> {
        self.reactive
            .states
            .iter()
            .map(|s| {
                (
                    s.name.clone(),
                    s.values.clone(),
                    s.values[s.current as usize].clone(),
                )
            })
            .collect()
    }

    /// The transitions a tick beginning now makes: each state's first
    /// entry, and each transition asked for.
    pub(crate) fn begin_transitions(&mut self) {
        self.reactive.transitions.clear();
        for (i, s) in self.reactive.states.iter_mut().enumerate() {
            if !s.entered {
                s.entered = true;
                s.pending = None;
                self.reactive.transitions.push(Transition {
                    state: i,
                    from: None,
                    to: s.current,
                });
            } else if let Some(to) = s.pending.take()
                && to != s.current
            {
                self.reactive.transitions.push(Transition {
                    state: i,
                    from: Some(s.current),
                    to,
                });
            }
        }
    }

    /// The tick's steps: per transition, the systems exiting its old value,
    /// the state taking the new one, the systems entering it; then every
    /// system not bound to an exit or an entry, in schedule order.
    pub(crate) fn slots(&self) -> Vec<Slot> {
        let mut out = Vec::new();
        for (k, t) in self.reactive.transitions.iter().enumerate() {
            let bound = |when: StateWhen, value: Option<u32>| {
                self.systems.iter().enumerate().filter_map(move |(i, s)| {
                    (s.state == Some((t.state, value?, when))).then_some(Slot::Run(i))
                })
            };
            out.extend(bound(StateWhen::Exit, t.from));
            out.push(Slot::Set(k));
            out.extend(bound(StateWhen::Enter, Some(t.to)));
        }
        out.extend(
            self.systems
                .iter()
                .enumerate()
                .filter_map(|(i, s)| match s.state {
                    Some((_, _, StateWhen::Enter | StateWhen::Exit)) => None,
                    _ => Some(Slot::Run(i)),
                }),
        );
        out
    }

    /// Whether system `i` runs this tick by its state binding (exit and
    /// entry systems are only ever placed by their transitions).
    pub(crate) fn in_its_state(&self, i: usize) -> bool {
        match self.systems[i].state {
            Some((state, value, StateWhen::During)) => self.reactive.states[state].current == value,
            _ => true,
        }
    }

    /// Runs system `i`'s run condition, if it has one: whether it runs.
    pub(crate) fn condition_holds(&self, i: usize) -> Result<bool> {
        let Some(condition) = &self.systems[i].condition else {
            return Ok(true);
        };
        let (access, cond) = &**condition;
        let mut commands = Vec::new();
        let holds = {
            let mut ctx = SystemContext::new(self, access, &mut commands);
            cond(&mut ctx)?
        };
        if !commands.is_empty() {
            return Err(Error::new(
                ErrorKind::Access,
                "a run condition reads the world; it may not change it",
            ));
        }
        Ok(holds)
    }

    /// Drops removals older than the start of the tick that just ended, and
    /// starts a tick: moves the change tick on and compares every tracked
    /// component (what was written between ticks).
    pub(crate) fn begin_tick_changes(&mut self) {
        let keep_from = self.reactive.step_start;
        for log in &mut self.reactive.removed {
            log.retain(|(_, t)| *t >= keep_from);
        }
        self.reactive.change_tick += 1;
        self.reactive.step_start = self.reactive.change_tick;
        self.diff(None, self.reactive.change_tick);
    }

    // ---- snapshot -----------------------------------------------------------

    /// The reactive block's bytes (`unguarded`: taken by a running system,
    /// whose columns are read in place).
    pub(crate) fn save_reactive(&self, unguarded: bool) -> Vec<u8> {
        let mut w = Writer(Vec::new());
        let r = &self.reactive;
        w.u32(VERSION as usize);
        w.u64(r.change_tick);
        w.u64(r.step_start);
        w.u32(self.systems.len());
        for s in &self.systems {
            w.sized(s.name.as_bytes());
            w.u64(s.last_run);
        }
        let tracked: Vec<usize> = (0..r.tracked.len()).filter(|&i| r.tracked[i]).collect();
        w.u32(tracked.len());
        for &i in &tracked {
            let id = SchemaId(i as u32);
            w.sized(self.registry.name(id).as_bytes());
            let rows: usize = self
                .archetypes
                .iter()
                .filter(|a| a.has(id))
                .map(|a| a.len())
                .sum();
            w.u32(rows);
            for arch in &self.archetypes {
                let Some(position) = arch.position(id) else {
                    continue;
                };
                for (row, e) in arch.entities.iter().enumerate() {
                    let (added, changed) = arch.row_ticks(position, row).unwrap_or((0, 0));
                    w.u64(e.to_bits());
                    w.u64(added);
                    w.u64(changed);
                    if arch.dirty(position, row, unguarded) {
                        w.u8(1);
                        w.sized(&arch.shadow_row(position, row));
                    } else {
                        w.u8(0);
                    }
                }
            }
            let log = &r.removed[i];
            w.u32(log.len());
            for (e, t) in log {
                w.u64(e.to_bits());
                w.u64(*t);
            }
        }
        w.u32(r.states.len());
        for s in &r.states {
            w.sized(s.name.as_bytes());
            w.sized(s.values[s.current as usize].as_bytes());
            let pending = s.pending.map_or("", |p| s.values[p as usize].as_str());
            w.sized(pending.as_bytes());
            w.u8(u8::from(s.entered));
        }
        w.u32(r.transitions.len());
        for t in &r.transitions {
            let s = &r.states[t.state];
            w.sized(s.name.as_bytes());
            let from = t.from.map_or("", |f| s.values[f as usize].as_str());
            w.sized(from.as_bytes());
            w.sized(s.values[t.to as usize].as_bytes());
        }
        w.0
    }

    /// Reads a reactive block against this world's systems, tracked types
    /// and state machines; nothing changes. What this world lacks (a system,
    /// a type, a state or value) is refused, or, with `report`, dropped with
    /// a warning.
    pub(crate) fn decode_reactive(
        &self,
        bytes: &[u8],
        mut report: Option<&mut crate::MigrationReport>,
    ) -> Result<SavedReactive> {
        let mut r = Reader { buf: bytes, pos: 0 };
        let version = r.u32()? as u32;
        if version != VERSION {
            return Err(bad(format!(
                "the og.ecs block is version {version}, this engine reads {VERSION}"
            )));
        }
        let mut out = SavedReactive {
            change_tick: r.u64()?,
            step_start: r.u64()?,
            ..Default::default()
        };
        let mut lacks = |what: String| -> Result<()> {
            match report.as_deref_mut() {
                Some(rep) => {
                    rep.warnings.push(format!("{what}; it is dropped"));
                    Ok(())
                }
                None => Err(Error::new(ErrorKind::NotFound, format!("snapshot: {what}"))),
            }
        };
        let n = r.count(12)?;
        for _ in 0..n {
            let name = r.str("system name")?;
            let last = r.u64()?;
            match self.systems.iter().position(|s| s.name == name) {
                Some(i) => out.last_run.push((i, last)),
                None => lacks(format!(
                    "its change ticks name system '{name}', which this world does not schedule"
                ))?,
            }
        }
        let n = r.count(12)?;
        for _ in 0..n {
            let name = r.str("tracked type")?;
            // The type table governs which types a restore takes; change
            // ticks of one it does not (renamed or dropped by a migration)
            // are dropped, and its rows start as tracked afresh.
            let id = self.registry.id_by_name(name).ok();
            let rows = r.count(25)?;
            let mut list = Vec::with_capacity(rows);
            for _ in 0..rows {
                let e = Entity(RawHandle::from_bits(r.u64()?));
                let added = r.u64()?;
                let changed = r.u64()?;
                let shadow = match r.u8()? {
                    0 => None,
                    1 => Some(r.sized()?.to_vec()),
                    v => return Err(bad(format!("change tick dirty byte {v}"))),
                };
                list.push(SavedRow {
                    entity: e,
                    added,
                    changed,
                    shadow,
                });
            }
            let k = r.count(16)?;
            let mut log = Vec::with_capacity(k);
            for _ in 0..k {
                log.push((Entity(RawHandle::from_bits(r.u64()?)), r.u64()?));
            }
            if let Some(id) = id {
                out.types.push((id, list, log));
            }
        }
        let n = r.count(13)?;
        for _ in 0..n {
            let name = r.str("state name")?;
            let current = r.str("state value")?;
            let pending = r.str("state value")?;
            let entered = r.u8()? != 0;
            let Ok(state) = self.state_index(name) else {
                lacks(format!(
                    "it holds state machine '{name}', which this world does not have"
                ))?;
                continue;
            };
            let value = |v: &str| self.state_value(state, v);
            let current = match value(current) {
                Ok(v) => v,
                Err(e) => {
                    lacks(e.reason().to_string())?;
                    continue;
                }
            };
            let pending = if pending.is_empty() {
                None
            } else {
                match value(pending) {
                    Ok(v) => Some(v),
                    Err(e) => {
                        lacks(e.reason().to_string())?;
                        None
                    }
                }
            };
            out.states.push((state, current, pending, entered));
        }
        let n = r.count(12)?;
        for _ in 0..n {
            let name = r.str("state name")?;
            let from = r.str("state value")?;
            let to = r.str("state value")?;
            let Ok(state) = self.state_index(name) else {
                lacks(format!(
                    "it holds a transition of state machine '{name}', which this world does not have"
                ))?;
                continue;
            };
            let from = if from.is_empty() {
                Ok(None)
            } else {
                self.state_value(state, from).map(Some)
            };
            match (from, self.state_value(state, to)) {
                (Ok(from), Ok(to)) => out.transitions.push(Transition { state, from, to }),
                (Err(e), _) | (_, Err(e)) => lacks(e.reason().to_string())?,
            }
        }
        if r.pos != bytes.len() {
            return Err(bad(format!(
                "{} trailing bytes in the og.ecs block",
                bytes.len() - r.pos
            )));
        }
        Ok(out)
    }

    /// Puts restored reactive state in place, after the archetypes are: the
    /// saved ticks by entity, the rest tracked from now as if before the
    /// first tick; `None` (a snapshot without the block) starts it afresh.
    pub(crate) fn apply_reactive(&mut self, saved: Option<SavedReactive>) {
        let saved = saved.unwrap_or_default();
        self.reactive.change_tick = saved.change_tick;
        self.reactive.step_start = saved.step_start;
        for s in &mut self.systems {
            s.last_run = 0;
        }
        for (i, last) in saved.last_run {
            self.systems[i].last_run = last;
        }
        for log in &mut self.reactive.removed {
            log.clear();
        }
        for a in 0..self.archetypes.len() {
            self.archetypes[a].ticks.fill(None);
            self.track_archetype(a);
        }
        for (id, rows, log) in saved.types {
            if !self.reactive.is_tracked(id) {
                continue;
            }
            for row in rows {
                let Ok(loc) = self.location(row.entity) else {
                    continue;
                };
                let arch = &mut self.archetypes[loc.archetype as usize];
                let Some(position) = arch.position(id) else {
                    continue;
                };
                let r = loc.row as usize;
                let sizes: Vec<usize> = arch
                    .columns_of(position)
                    .map(|c| arch.columns[c].get_mut().row(0).len())
                    .collect();
                let Some(t) = arch.ticks[position].as_mut() else {
                    continue;
                };
                t.added[r] = row.added;
                t.changed[r] = row.changed;
                if let Some(shadow) = row.shadow
                    && shadow.len() == sizes.iter().sum::<usize>()
                {
                    let mut at = 0;
                    for (k, size) in sizes.iter().enumerate() {
                        t.shadow[k][r * size..(r + 1) * size]
                            .copy_from_slice(&shadow[at..at + size]);
                        at += size;
                    }
                }
            }
            self.reactive.removed[id.0 as usize] = log;
        }
        // A state the snapshot did not hold starts afresh.
        for s in &mut self.reactive.states {
            s.current = s.initial;
            s.pending = None;
            s.entered = false;
        }
        for (state, current, pending, entered) in saved.states {
            let s = &mut self.reactive.states[state];
            s.current = current;
            s.pending = pending;
            s.entered = entered;
        }
        self.reactive.transitions = saved.transitions;
        self.reactive.in_step = false;
        self.reactive.hook_depth = 0;
    }
}

/// One tracked row as saved.
pub(crate) struct SavedRow {
    pub entity: Entity,
    pub added: u64,
    pub changed: u64,
    /// The value as last compared, when it differs from the saved value.
    pub shadow: Option<Vec<u8>>,
}

/// One tracked type as saved: its rows and its removals.
pub(crate) type SavedType = (SchemaId, Vec<SavedRow>, Vec<(Entity, u64)>);

/// A decoded reactive block.
#[derive(Default)]
pub(crate) struct SavedReactive {
    pub change_tick: u64,
    pub step_start: u64,
    /// (system index, last run).
    pub last_run: Vec<(usize, u64)>,
    pub types: Vec<SavedType>,
    /// (state, current, pending, entered).
    pub states: Vec<(usize, u32, Option<u32>, bool)>,
    pub transitions: Vec<Transition>,
}
