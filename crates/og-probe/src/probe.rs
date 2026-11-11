//! [`Probe`]: RuntimeProbe protocol 5 plus the engine's wire members, and the
//! command channel's execution, over a world the host owns and passes in.

mod actions;
#[cfg(not(feature = "publish"))]
mod marks;
mod protocol;
mod state;
mod timeline;

#[cfg(not(feature = "publish"))]
pub use marks::MarksDesc;

use crate::assertions::Assertions;
use crate::base64;
use crate::command::{
    self, CORE_ACTIONS, CommandDesc, CommandHandler, CommandInfo, CommandStep, CommandsReport,
    DumpHandler, Inbox, LogLine, Output, ParseError, Payload,
};
use crate::expect::{ExpectResult, Watch};
use crate::hooks::{Capture, Reloader, SceneView, SceneWriter, UiTree, find_by_id, find_by_text};
use crate::input::{
    InjectedInput, InputSink, KeyAction, Modifiers, PointerAction, PointerButton, WorldInput,
    key_for_char,
};
use crate::names::Names;
use crate::value;
use og_core::glam::{Mat3, Quat, Vec3};
use og_core::{Error, ErrorKind, FieldType, Layout, Rng, Schema};
use og_ecs::{Entity, QueryDesc, SchemaId, World};
use serde_json::{Map, Value, json};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::rc::Rc;

mod record;

/// The RuntimeProbe contract revision (`origin/packages/contracts/src/probe.ts`).
pub const PROBE_VERSION: u32 = 5;
/// The wire revision `hello` reports (`engine/docs/probe-wire.md`).
pub const PROBE_WIRE_VERSION: u32 = 1;
/// The longest playtest one request may advance, in fixed steps.
pub const PLAYTEST_MAX_TICKS: u64 = 3600;
/// The most operations one probe program may carry.
pub const PROBE_PROGRAM_MAX_OPERATIONS: usize = 256;
/// The most values one sweep may try.
pub const MAX_SWEEP_VALUES: usize = 64;
/// The longest capture edge, in pixels.
pub const MAX_CAPTURE_EDGE: u32 = 4096;
/// Snapshots kept by `snapshot`; the oldest is dropped beyond this.
pub const MAX_SNAPSHOTS: usize = 16;
/// The most entities `observe` reads of one component.
pub const OBSERVE_ROWS: usize = 256;
/// The most calls a `screenshot` or `sweep` value waits for a frame that
/// lands on a later call (the web's readback) before it gives up.
const MAX_CAPTURE_POLLS: u32 = 120;
/// What a `screenshot` step logs (not a warning) in a host without a
/// renderer, followed by ` (<file>)`: the step writes no PNG, and a
/// playtest reports the frame as unchecked and passes (docs/design.md §5,
/// the `engine` and `probe` sandboxes carry no renderer).
pub const FRAME_UNCHECKED: &str = "screenshot: no renderer, frame unchecked";
/// `hello.engine` unless the host says otherwise.
pub const ENGINE_VERSION: &str = concat!("origin-engine ", env!("CARGO_PKG_VERSION"));

/// Every member this crate implements, in protocol order.
pub const MEMBERS: &[&str] = &[
    "hello",
    "tick",
    "state",
    "advance",
    "pause",
    "resume",
    "input",
    "accepted",
    "pick",
    "entity",
    "bounds",
    "patch",
    "run",
    "capture",
    "sweep",
    "reload",
    "snapshot",
    "restore",
    "dump",
    "commands",
    "listen",
    "memory",
    "describe",
    "expect",
    "expectations",
    "save_scene",
    "assertions",
    "assertion_events",
    "observe",
    "pick_ui",
    "marks",
    "timeline",
];

/// The player-facing members a `publish` build keeps.
pub const PUBLISH_MEMBERS: &[&str] = &[
    "hello", "tick", "state", "pick", "entity", "input", "capture",
];

/// The members this build serves: [`PUBLISH_MEMBERS`] with the `publish`
/// feature, else [`MEMBERS`].
pub fn served_members() -> &'static [&'static str] {
    if cfg!(feature = "publish") {
        PUBLISH_MEMBERS
    } else {
        MEMBERS
    }
}

/// The build configuration `hello` reports (IDL `BuildProfile`).
pub const BUILD: &str = if cfg!(feature = "publish") {
    "publish"
} else if cfg!(debug_assertions) {
    "dev"
} else {
    "release"
};

/// What `dump` reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DumpKind {
    World,
    Entity,
    Ui,
    /// The host's `dump_desync` (og.net's lockstep), when it registered one.
    Desync,
}

/// A parameter sweep (IDL `SweepDesc`): for each value, restore the snapshot
/// taken at the call, patch, advance and capture.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SweepDesc {
    pub id: String,
    pub parameter: String,
    pub values: Vec<Value>,
    pub steps: u64,
    /// 0 is the render target's.
    pub width: u32,
    pub height: u32,
}

/// One probe snapshot: the world (its names included), and the renderer's
/// state that follows the world's time ([`Capture::save_state`]).
#[derive(Clone)]
struct Saved {
    world: Vec<u8>,
    tick: u64,
    hash: u64,
    render: u64,
}

/// A `screenshot` step whose frame is in flight: written when it lands,
/// at the tick it was asked at.
struct Shot {
    tick: u64,
    /// The step's time, for the warning when it never lands.
    at: f64,
    path: Option<String>,
    width: u32,
    height: u32,
    polls: u32,
}

/// A `sweep` whose frames land on later calls: asked again with the same
/// description on the same world, it goes on from the frames it has.
struct SweepJob {
    desc: SweepDesc,
    /// The tick and hash of the world it started from.
    from: (u64, u64),
    frames: Vec<Value>,
    /// The next value's world, patched and advanced, while its frame is in flight.
    waiting: Option<Saved>,
    polls: u32,
}

enum Queued {
    /// A step and where in its tick it falls (its time less the tick's, in
    /// ticks: -0.5 to 0.5), the stamp its input events carry.
    Step(CommandStep, f32),
    /// One character of `type`, spread one press per two ticks.
    Inject(InjectedInput),
}

struct Registered {
    desc: CommandDesc,
    handler: Handler,
}

#[derive(Clone)]
enum Handler {
    Act(CommandHandler),
    /// A dump and the index of its optional path argument.
    Dump(DumpHandler, usize),
}

type FactsFn = Box<dyn Fn(&World) -> Value>;

/// The host's file reader for `expect:frame` references: a project-relative
/// path's bytes, `None` when it cannot be read.
pub type ReaderFn = Box<dyn Fn(&str) -> Option<Vec<u8>>>;

/// The engine's probe over a world the host owns.
///
/// The host creates it once per world ([`Probe::new`]), hands it the hooks it
/// has ([`Probe::set_capture`], [`Probe::set_ui`], [`Probe::set_reloader`],
/// [`Probe::set_facts`]), and drives every tick through it
/// ([`Probe::frame`] or [`Probe::step`]) so scheduled commands run at the
/// start of their tick and input edges expire. Calls arrive as JSON
/// ([`Probe::call`]) and are answered on the world's thread between ticks.
///
/// The probe is not `Send`: hooks and command handlers may hold host
/// objects (JavaScript values on the web). Create it on the engine thread.
pub struct Probe {
    /// What the host gave ([`Probe::set_capture`] and the other `set_*`).
    pub(crate) host: state::Host,
    pub(crate) names: Names,
    /// What a run asserts: its expectations, assertions, invariants and counted events.
    pub(crate) checks: state::Checks,
    /// The run's events since `timeline` last read them ([`timeline::Recorder`]).
    pub(crate) timeline: timeline::Recorder,
    /// Fields `set` and `patch` wrote (`save_scene` writes them back).
    tuned: BTreeSet<String>,
    /// The host's last measurements, for `state().timings`.
    pub(crate) stats: state::Stats,
    /// The command channel: the registered actions and the ones modules push.
    pub(crate) commands: state::Commands,
    /// The script's steps waiting to run, and the screenshot that holds them.
    pub(crate) schedule: state::Schedule,
    /// Worlds kept to go back to: the snapshots and a sweep in flight.
    pub(crate) saves: state::Saves,
    /// Whether the next tick runs: the holds and the host's hooks around a tick.
    pub(crate) clock: state::Clock,
    /// What a run hands back to the host: output files and log lines.
    pub(crate) out: state::Out,
    /// The input being injected: where the pointer is and when in the tick.
    pub(crate) inject: state::Inject,
}

/// The host's tick gate: asked before every tick the clock takes (a frame's
/// ticks, `advance`), with the world between ticks; false holds the tick
/// (og.net's lockstep waiting for a player's input). It may restore the
/// world (a lockstep start or late join hands a peer's snapshot over): the
/// clock it moves to is the one every peer shares, so the scheduled steps
/// stay on their ticks ([`Probe::step`]), unlike a game's own restore.
pub type GateFn = Box<dyn FnMut(&mut World) -> bool>;

/// The host's input filter: asked for every input event before the world
/// sees it (the host's own and a script's alike); true takes the event,
/// so the world never receives it (a development tool drawn over the game,
/// the inspector, taking the pointer over its panel). A taken event counts
/// as accepted.
pub type InputFilterFn = Box<dyn FnMut(&World, &InjectedInput) -> bool>;

/// The host's end-of-tick hook: called after every tick the probe takes,
/// with the world between ticks (a game's deferred restore lands here).
pub type AfterTickFn = Box<dyn FnMut(&mut World)>;

/// The host's `derive` hook: what `get` answers for a record whose value the
/// engine derives each frame from other state (a light og.DayNight drives).
/// It is called with the entity (`None` for a resource), the record's name
/// and the stored fields, and may replace or add fields. It reads the world
/// only, as a pure function of it.
pub type DeriveFn = Box<dyn Fn(&World, Option<Entity>, &str, &mut Map<String, Value>)>;

/// Registers an engine record the world has not registered yet because no
/// game has named it (the runtime's lazy records): `set` names the record
/// its target starts with, as a game's write would, so a script can set a
/// lazy resource's field. Unknown names are left alone.
pub type RegisterFn = Box<dyn Fn(&mut World, &str)>;

/// What the host's mixer rendered last, for `listen`: the last `seconds`
/// as a 16-bit stereo WAV and its rate, frames, peak and RMS.
pub struct Heard {
    pub wav: Vec<u8>,
    pub rate: u32,
    pub frames: u64,
    pub peak: f32,
    pub rms: f32,
}

/// The host's `listen` hook.
pub type ListenFn = Box<dyn Fn(f32) -> Option<Heard>>;

/// The host's `memory` hook: what the engine holds, as the IDL's
/// `ProbeMemory` (the host, the wasm linear memory or the process's resident
/// memory, the counted heap, the world, the streaming budget, the audio).
pub type MemoryFn = Box<dyn Fn(&World) -> Value>;

/// A script's output path, relative to wherever the host writes outputs:
/// an absolute path or a `..` component is refused, so a script (or a
/// caller of the probe) cannot write outside the output directory, and so
/// is a name starting with a dot, which the directory keeps for the harness.
fn output_path(arg: Option<&String>) -> Result<Option<String>, String> {
    let Some(path) = arg.filter(|p| !p.is_empty()) else {
        return Ok(None);
    };
    let absolute = path.starts_with('/') || path.starts_with('\\');
    if absolute || path.split(['/', '\\']).any(|part| part == "..") {
        return Err(format!(
            "output path '{path}' leaves the output directory; write a relative path without '..'"
        ));
    }
    // Dot names in the output directory are the harness's (a playtest's
    // kept store, its bundle, its later runs): no step may overwrite them.
    if path
        .split(['/', '\\'])
        .any(|part| part.starts_with('.') && part != ".")
    {
        return Err(format!(
            "output path '{path}' has a name starting with '.', which the output directory keeps for the harness (a playtest's kept og.platform store, its bundle); name the file without the leading dot"
        ));
    }
    Ok(Some(path.clone()))
}

/// `path` with `-<n>` before its extension (`x.json` -> `x-2.json`).
fn suffixed(path: &str, n: u32) -> String {
    let base = path.rfind(['/', '\\']).map_or(0, |i| i + 1);
    match path[base..].rfind('.') {
        Some(dot) if dot > 0 => {
            let dot = base + dot;
            format!("{}-{n}{}", &path[..dot], &path[dot..])
        }
        _ => format!("{path}-{n}"),
    }
}

fn fault(context: &str, err: Error) -> Error {
    err.context(context)
}

/// An injected event's `at` (KeyInput.at, PointerInput.at): finite, -1 to
/// 1 ticks; 0 when absent.
pub fn stamp_of(event: &Value) -> f32 {
    event
        .get("at")
        .and_then(Value::as_f64)
        .filter(|a| a.is_finite())
        .map_or(0.0, |a| a.clamp(-1.0, 1.0) as f32)
}

/// og.Time as game code reads it (`world.resource(og.Time)`, a system's
/// `time`): the clock, its rate, and the time scale the world keeps in
/// og.TimeScale (scale, hitStop, scaledDt, scaledElapsed).
/// A resource's fields as JSON by name (a UI viewmodel's reflected root,
/// docs/ui.md §2.1): `None` for a name that is no resource of the world.
pub fn resource_json(world: &World, name: &str) -> Option<Value> {
    let id = world
        .id_by_name(name)
        .ok()
        .filter(|&id| world.has_resource_id(id))?;
    if name == <og_core::Time as og_core::Reflect>::NAME {
        return Some(Value::Object(time_json(world)));
    }
    let schema = world.schema(id).ok()?;
    let bytes = world.resource_bytes(id).ok()?;
    Some(Value::Object(value::component_json(schema, &bytes)))
}

pub fn time_json(world: &World) -> Map<String, Value> {
    let t = world.time();
    let s = world.time_scale();
    let Value::Object(fields) = json!({
        "tick": t.tick,
        "dt": f64::from(t.dt),
        "elapsed": t.elapsed,
        "rate": tick_rate(world),
        "scale": s.scale,
        "hitStop": s.hit_stop,
        "scaledDt": s.dt(&t),
        "scaledElapsed": s.elapsed,
    }) else {
        unreachable!("an object")
    };
    fields
}

fn tick_rate(world: &World) -> u32 {
    (1.0 / f64::from(world.time().dt)).round().max(1.0) as u32
}

/// An engine-owned type by its dotted name (`og.Transform`), or by its bare
/// name (`Transform`) in a world that registered it so.
fn engine_type(world: &World, name: &str) -> Result<SchemaId, String> {
    world
        .id_by_name(&format!("og.{name}"))
        .or_else(|_| world.id_by_name(name))
        .map_err(|_| format!("no og.{name} is registered"))
}

fn schema_name(world: &World, id: SchemaId) -> String {
    world
        .schema(id)
        .map_or_else(|_| format!("#{}", id.index()), |s| s.name.clone())
}

/// `x,y[,z...]` as numbers.
fn numbers(text: &str) -> Result<Vec<f32>, String> {
    text.split(',')
        .map(|t| {
            t.trim()
                .parse::<f32>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| format!("'{}' is not a number", t.trim()))
        })
        .collect()
}

/// The rotation that points a camera's -Z from `eye` towards `target`, +Y up.
fn look_at(eye: Vec3, target: Vec3) -> Option<Quat> {
    let back = (eye - target).try_normalize()?;
    let up = if back.cross(Vec3::Y).length_squared() < 1e-12 {
        Vec3::Z
    } else {
        Vec3::Y
    };
    let x = up.cross(back).normalize();
    let y = back.cross(x);
    Some(Quat::from_mat3(&Mat3::from_cols(x, y, back)).normalize())
}

impl Probe {
    /// A probe for `world`, with the default input sink ([`WorldInput`]),
    /// which registers the `Input` resource. Create the probe before the
    /// first snapshot so every snapshot has the same types.
    pub fn new(world: &mut World) -> og_core::Result<Probe> {
        Probe::with_input(world, Box::new(WorldInput::default()))
    }

    /// A probe delivering input through another sink (the input module's).
    pub fn with_input(world: &mut World, mut input: Box<dyn InputSink>) -> og_core::Result<Probe> {
        input.attach(world)?;
        Ok(Probe {
            host: state::Host {
                engine: ENGINE_VERSION.to_owned(),
                input,
                capture: None,
                ui: None,
                reloader: None,
                facts: None,
                scene_view: None,
                scene_writer: None,
                reader: None,
                listen: None,
                memory: None,
                derive: None,
                register: None,
            },
            names: Names,
            checks: state::Checks {
                expectations: Vec::new(),
                assertions: Rc::default(),
                invariants: Vec::new(),
                quiet: false,
                events: BTreeMap::new(),
                event_base: BTreeMap::new(),
            },
            timeline: timeline::Recorder::default(),
            tuned: BTreeSet::new(),
            stats: state::Stats {
                systems_ms: Vec::new(),
                passes_ms: Vec::new(),
                gpu_passes_ms: Vec::new(),
                step_ms: 0.0,
                render_ms: 0.0,
                triangles: 0,
            },
            inject: state::Inject {
                stamp: 0.0,
                pointer: (0.0, 0.0),
                target_missed: false,
                filter: None,
            },
            commands: state::Commands {
                registered: BTreeMap::new(),
                inbox: Inbox::default(),
            },
            schedule: state::Schedule {
                queue: BTreeMap::new(),
                seq: 0,
                shot: None,
            },
            saves: state::Saves {
                snapshots: VecDeque::new(),
                next_snapshot: 1,
                sweeping: None,
            },
            clock: state::Clock {
                paused: false,
                script_paused: false,
                quit: false,
                after_tick: None,
                before_tick: None,
                gate: None,
                replay_until: 0,
            },
            out: state::Out {
                outputs: Vec::new(),
                log: Vec::new(),
                gets: (u64::MAX, 0),
                written: BTreeMap::new(),
            },
        })
    }

    // ---- host setup -------------------------------------------------------

    /// `hello.engine`, such as `origin-engine 0.1.0`.
    pub fn set_engine_version(&mut self, engine: impl Into<String>) {
        self.host.engine = engine.into();
    }

    /// The renderer: frames for `capture`, `sweep`, `screenshot`, and `pick`.
    pub fn set_capture(&mut self, capture: Box<dyn Capture>) {
        self.host.capture = Some(capture);
    }

    /// The UI tree for `dump_ui`, `touch_move_to` and `move_to_text`.
    pub fn set_ui(&mut self, ui: Box<dyn UiTree>) {
        self.host.ui = Some(ui);
    }

    /// The frame as data (`describe`) and picking where the capture hook
    /// does not pick.
    pub fn set_scene_view(&mut self, view: Box<dyn SceneView>) {
        self.host.scene_view = Some(view);
    }

    /// The boot scene's write-back for `save_scene`.
    pub fn set_scene_writer(&mut self, writer: Box<dyn SceneWriter>) {
        self.host.scene_writer = Some(writer);
    }

    /// Reads `expect:frame` references (project-relative paths).
    pub fn set_reader(&mut self, read: impl Fn(&str) -> Option<Vec<u8>> + 'static) {
        // Only a member a publish build does not serve reads it.
        #[cfg(feature = "publish")]
        let _ = read;
        #[cfg(not(feature = "publish"))]
        {
            self.host.reader = Some(Box::new(read));
        }
    }

    /// The bundle swap for `reload`.
    pub fn set_reloader(&mut self, reloader: Box<dyn Reloader>) {
        self.host.reloader = Some(reloader);
    }

    /// What `get` answers for records the engine derives from other state
    /// ([`DeriveFn`]).
    pub fn set_derive(&mut self, derive: DeriveFn) {
        self.host.derive = Some(derive);
    }

    /// How `set` registers a lazy record it names ([`RegisterFn`]).
    pub fn set_register(&mut self, register: RegisterFn) {
        self.host.register = Some(register);
    }

    /// Work the host does after every tick, between ticks ([`AfterTickFn`]).
    /// When it moves `Time` (a game restoring a save), the scheduled steps
    /// keep their remaining delays ([`Probe::shift_schedule`]).
    pub fn set_after_tick(&mut self, after: AfterTickFn) {
        self.clock.after_tick = Some(after);
    }

    /// Work the host does as every tick starts, just before the step (after
    /// the command steps due at it): events it sends there are delivered in
    /// that tick, and the world between ticks is untouched by them (a
    /// restore's og.WorldRestored).
    pub fn set_before_tick(&mut self, before: AfterTickFn) {
        self.clock.before_tick = Some(before);
    }

    /// The host's tick gate ([`GateFn`]): a frame takes no further tick, and
    /// `advance` stops, while it holds.
    pub fn set_gate(&mut self, gate: GateFn) {
        self.clock.gate = Some(gate);
    }

    /// The host's input filter ([`InputFilterFn`]): an event it takes never
    /// reaches the world.
    pub fn set_input_filter(&mut self, filter: InputFilterFn) {
        self.inject.filter = Some(filter);
    }
}

impl state::Clock {
    /// Whether the gate holds the next tick. A gate that moved `Time`
    /// restored a peer's snapshot (lockstep's start or a late join): the
    /// world now runs on the room's shared clock, and the scheduled steps
    /// stay on their ticks of it, so every peer of a room runs the same
    /// script step (a `quit`, an `expect`) on the same tick. A step whose
    /// tick the jump passed runs at the next tick, in order.
    fn held(&mut self, world: &mut World) -> bool {
        let Some(gate) = self.gate.as_mut() else {
            return false;
        };
        let before = world.time().tick;
        let go = gate(world);
        let after = world.time().tick;
        if after < before {
            self.replay_until = self.replay_until.max(before);
        }
        !go
    }

    /// Whether the next tick runs again after a rollback's rewind.
    fn replaying(&self, world: &World) -> bool {
        world.time().tick < self.replay_until
    }
}

impl Probe {
    /// The world's clock jumped from `from` to `to` outside a tick (a game
    /// restored a save, which carries its own `Time`): every scheduled step
    /// moves with it, so each keeps the delay it had left.
    pub fn shift_schedule(&mut self, from: u64, to: u64) {
        if from == to || self.schedule.queue.is_empty() {
            return;
        }
        let queue = std::mem::take(&mut self.schedule.queue);
        self.schedule.queue = og_core::sort::map(queue.into_iter().map(|((tick, seq), step)| {
            let left = tick.saturating_sub(from);
            ((to + left, seq), step)
        }));
    }

    /// The game's own facts for `state().facts` (`GameDesc.facts`).
    pub fn set_facts(&mut self, facts: impl Fn(&World) -> Value + 'static) {
        self.host.facts = Some(Box::new(facts));
    }

    /// Gives `listen` the host's audio: the last seconds its mixer rendered.
    pub fn set_listen(&mut self, listen: impl Fn(f32) -> Option<Heard> + 'static) {
        // Only a member a publish build does not serve reads it.
        #[cfg(feature = "publish")]
        let _ = listen;
        #[cfg(not(feature = "publish"))]
        {
            self.host.listen = Some(Box::new(listen));
        }
    }

    /// `listen`: the last `seconds` (0 to 2) of audio the host's mixer
    /// rendered, a WAV in base64 with its rate, frames, peak and RMS; `None`
    /// without audio or before anything was rendered.
    pub fn listen(&mut self, world: &World, seconds: f32) -> Option<Value> {
        let h = (self.host.listen.as_ref()?)(seconds.clamp(0.0, 2.0))?;
        Some(json!({
            "tick": world.time().tick,
            "rate": h.rate,
            "frames": h.frames,
            "peak": h.peak,
            "rms": h.rms,
            "wav": base64::encode(&h.wav),
        }))
    }

    /// Gives `memory` the host's figures ([`MemoryFn`]).
    pub fn set_memory(&mut self, memory: impl Fn(&World) -> Value + 'static) {
        // Only a member a publish build does not serve reads it.
        #[cfg(feature = "publish")]
        let _ = memory;
        #[cfg(not(feature = "publish"))]
        {
            self.host.memory = Some(Box::new(memory));
        }
    }

    /// `memory`: what the engine holds (`ProbeMemory`); `None` from a host
    /// that gave no figures.
    pub fn memory(&self, world: &World) -> Option<Value> {
        self.host.memory.as_ref().map(|f| f(world))
    }

    /// The last tick's timings for `state().timings`, measured by the host.
    /// The last frame's main-pass triangles (`ProbeTimings.triangles`).
    pub fn set_triangles(&mut self, triangles: u64) {
        self.stats.triangles = triangles;
    }

    pub fn set_timings(&mut self, step_ms: f64, render_ms: f64) {
        self.stats.step_ms = step_ms;
        self.stats.render_ms = render_ms;
    }

    /// The last tick's milliseconds per system (in run order) and the last
    /// frame's per render pass (in the graph's order), for
    /// `state().timings.systems` and `.passes`.
    /// `gpu`: the GPU's milliseconds per pass of the newest frame its
    /// timestamps were read back for (`.gpuPasses`; empty without them).
    pub fn set_timing_detail(
        &mut self,
        systems: Vec<(String, f64)>,
        passes: Vec<(String, f64)>,
        gpu: Vec<(String, f64)>,
    ) {
        self.stats.systems_ms = systems;
        self.stats.passes_ms = passes;
        self.stats.gpu_passes_ms = gpu;
    }

    /// Probe ids over the world's names (`World::set_name` names entities).
    pub fn names(&self) -> &Names {
        &self.names
    }

    /// Whether the clock is held (the host's loop then takes no ticks).
    pub fn paused(&self) -> bool {
        self.clock.paused
    }

    /// Whether a `quit` ran: the host ends the run.
    pub fn quit_requested(&self) -> bool {
        self.clock.quit
    }

    /// Steps still scheduled.
    pub fn pending(&self) -> usize {
        self.schedule.queue.len()
    }

    /// Steps still to run: those scheduled and a screenshot waiting for
    /// its frame. A host running a script ends it when this is 0 (or on
    /// `quit`), not at the tick the script's last step was scheduled for,
    /// which a restore moving og.Time ([`Probe::shift_schedule`]) moves.
    pub fn steps_left(&self) -> usize {
        self.schedule.queue.len() + usize::from(self.schedule.shot.is_some())
    }

    /// Files commands produced since the last call, for the host to write.
    pub fn take_outputs(&mut self) -> Vec<Output> {
        std::mem::take(&mut self.out.outputs)
    }
}

impl state::Out {
    /// Hands an output a step made to the host. A file an earlier step of
    /// this script already wrote is not overwritten: the second is written
    /// beside it as `<stem>-2.<ext>` (then `-3`, ...), with a warning naming
    /// both steps, so a check reading the first finds what it expects and
    /// the clash is in the run's warnings instead of far from its cause.
    fn emit(&mut self, at: f64, mut output: Output) {
        let named = output.path.is_some();
        let wanted = output.file_name();
        let mut name = wanted.clone();
        let mut n = 1;
        while self.written.contains_key(&name) {
            n += 1;
            name = suffixed(&wanted, n);
        }
        if name != wanted {
            output.path = Some(name.clone());
            // Unnamed outputs are the probe's own names: suffixed quietly.
            if named {
                let (first_at, first_action) = &self.written[&wanted];
                self.note(
                    output.tick,
                    true,
                    format!(
                        "{at}s {}: '{wanted}' was already written by the {first_at}s {first_action} step, so this one is '{name}'; give each step its own file name",
                        output.action
                    ),
                );
            }
        }
        self.written.insert(name, (at, output.action.clone()));
        self.outputs.push(output);
    }
}

impl Probe {
    /// Log lines since the last call.
    pub fn take_log(&mut self) -> Vec<LogLine> {
        std::mem::take(&mut self.out.log)
    }
}

impl state::Out {
    pub(crate) fn note(&mut self, tick: u64, warning: bool, text: String) {
        self.log.push(LogLine {
            tick,
            warning,
            text,
        });
    }
}

impl Probe {
    // ---- the command channel ---------------------------------------------

    /// Register an action (a module's or a game's `commands:`). Refused for
    /// a name that is not snake_case or that a core action or another
    /// registration uses.
    pub fn register_command(
        &mut self,
        name: &str,
        desc: CommandDesc,
        handler: impl Fn(&mut World, &[String]) -> Result<(), String> + 'static,
    ) -> og_core::Result<()> {
        self.commands.register_command(name, desc, handler)
    }

    /// Register a dump action: `handler` answers JSON for the world as of
    /// the tick it runs (with the arguments before `path_arg`), written as
    /// an output at `args[path_arg]` or `<name>-<tick>.json`. Refused as
    /// [`Self::register_command`] refuses.
    pub fn register_dump(
        &mut self,
        name: &str,
        desc: CommandDesc,
        path_arg: usize,
        handler: impl Fn(&World, &[String]) -> Result<Value, String> + 'static,
    ) -> og_core::Result<()> {
        self.commands.register_dump(name, desc, path_arg, handler)
    }

    /// The queue engine modules push actions into from inside a tick; they
    /// run at the start of the next tick (before its scheduled steps).
    pub fn inbox(&self) -> Inbox {
        self.commands.inbox.clone()
    }

    /// Every action the channel knows: core actions, then registered ones by name.
    pub fn help(&self) -> Vec<CommandInfo> {
        self.commands.help()
    }
}

impl state::Commands {
    /// [`Probe::register_command`].
    pub(super) fn register_command(
        &mut self,
        name: &str,
        desc: CommandDesc,
        handler: impl Fn(&mut World, &[String]) -> Result<(), String> + 'static,
    ) -> og_core::Result<()> {
        let valid = command::parse(&format!("0:{name}"))
            .is_ok_and(|s| s.len() == 1 && s[0].action == name && s[0].args.is_empty());
        if !valid {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("'{name}' is not an action name (snake_case, such as `spawn_wave`)"),
            ));
        }
        if CORE_ACTIONS.iter().any(|(core, _, _)| *core == name) {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("'{name}' is a core action; register under another name"),
            ));
        }
        if self.registered.contains_key(name) {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("'{name}' is already registered"),
            ));
        }
        self.registered.insert(
            name.to_owned(),
            Registered {
                desc,
                handler: Handler::Act(Rc::new(handler)),
            },
        );
        Ok(())
    }

    /// [`Probe::register_dump`].
    pub(super) fn register_dump(
        &mut self,
        name: &str,
        desc: CommandDesc,
        path_arg: usize,
        handler: impl Fn(&World, &[String]) -> Result<Value, String> + 'static,
    ) -> og_core::Result<()> {
        // A dump only observes the world for a tool: a publish build keeps
        // none, so the handlers the engine's modules register are not built.
        #[cfg(feature = "publish")]
        let _ = (name, desc, path_arg, handler);
        #[cfg(not(feature = "publish"))]
        {
            self.register_command(name, desc, |_, _| Ok(()))?;
            if let Some(r) = self.registered.get_mut(name) {
                r.handler = Handler::Dump(Rc::new(handler), path_arg);
            }
        }
        Ok(())
    }

    /// [`Probe::help`].
    pub(super) fn help(&self) -> Vec<CommandInfo> {
        let core = CORE_ACTIONS.iter().map(|(name, usage, doc)| CommandInfo {
            name: (*name).to_owned(),
            usage: (*usage).to_owned(),
            doc: (*doc).to_owned(),
            core: true,
        });
        let registered = self.registered.iter().map(|(name, r)| CommandInfo {
            name: name.clone(),
            usage: if r.desc.usage.is_empty() {
                name.clone()
            } else {
                format!("{name}:{}", r.desc.usage)
            },
            doc: r.desc.doc.clone(),
            core: false,
        });
        core.chain(registered).collect()
    }

    /// [`Probe::act`].
    pub(super) fn act(
        &mut self,
        world: &mut World,
        action: &str,
        args: &[String],
    ) -> Option<Result<(), String>> {
        match self.registered.get(action).map(|r| r.handler.clone())? {
            Handler::Act(handler) => Some(handler(world, args)),
            Handler::Dump(..) => Some(Err(format!(
                "`{action}` is a dump; run it through a commands script"
            ))),
        }
    }

    /// [`Probe::registered`].
    pub(super) fn registered(&self) -> Vec<String> {
        self.registered.keys().cloned().collect()
    }

    fn knows(&self, action: &str) -> bool {
        CORE_ACTIONS.iter().any(|(name, _, _)| *name == action)
            || self.registered.contains_key(action)
    }
}

impl Probe {
    /// Runs a registered action now, outside the schedule (a host's
    /// `engine.command`, `og run --setup`). `None` for a name nothing
    /// registered; a dump refuses, since its output belongs to a script.
    pub fn act(
        &mut self,
        world: &mut World,
        action: &str,
        args: &[String],
    ) -> Option<Result<(), String>> {
        self.commands.act(world, action, args)
    }

    /// The registered actions' names, for a refusal that lists them.
    pub fn registered(&self) -> Vec<String> {
        self.commands.registered()
    }

    /// Schedule a script on simulation time from the current tick: a step at
    /// `t` seconds runs at the start of tick `now + round(t · rate)`.
    pub fn schedule(&mut self, world: &World, script: &str) -> Result<CommandsReport, ParseError> {
        let steps = command::parse(script)?;
        let now = world.time().tick;
        // The events its expectations count, from now.
        let watched: Vec<String> = steps
            .iter()
            .filter(|s| s.action == "expect" && s.args.first().map(String::as_str) == Some("event"))
            .filter_map(|s| s.args.get(1).cloned())
            .collect();
        self.watch_events(watched.iter().map(String::as_str));
        // A script's outputs are its own: one names each file once.
        self.out.written.clear();
        let rate = f64::from(tick_rate(world));
        let mut report = CommandsReport {
            scheduled: 0,
            until_tick: now,
            warnings: Vec::new(),
        };
        for step in steps {
            let exact = step.at * rate;
            let tick = now + exact.round() as u64;
            // Where in its tick the step falls: an input action's events carry it.
            // A step written at a tick's time (`t/rate` in decimal) is at its
            // start: the float error of the division is not a time within it.
            let off = exact - exact.round();
            let within = if off.abs() < 1e-6 { 0.0 } else { off as f32 };
            if !self.commands.knows(&step.action) {
                report.warnings.push(format!(
                    "{}s: unknown action '{}'; it is skipped unless registered before tick {tick} (`help` lists the actions)",
                    step.at, step.action
                ));
            }
            self.schedule.seq += 1;
            self.schedule
                .queue
                .insert((tick, self.schedule.seq), Queued::Step(step, within));
            report.scheduled += 1;
            report.until_tick = tick;
        }
        Ok(report)
    }

    /// Run the steps due at the current tick (and earlier), in order. A
    /// `screenshot` whose frame lands on a later call (the web) holds the
    /// steps after it until it lands.
    pub fn pump(&mut self, world: &mut World) {
        if self
            .schedule
            .shot_waits(world, &mut self.host, &mut self.out)
        {
            return;
        }
        let mut inbox = self.commands.inbox.take().into_iter();
        while !self.clock.quit && self.schedule.shot.is_none() {
            let Some((action, args)) = inbox.next() else {
                break;
            };
            self.execute(
                world,
                CommandStep {
                    at: 0.0,
                    action,
                    args,
                },
            );
        }
        if !self.clock.quit {
            self.commands.inbox.requeue(inbox.collect());
        }
        while !self.clock.quit && self.schedule.shot.is_none() {
            let now = world.time().tick;
            let Some((&key, _)) = self.schedule.queue.first_key_value() else {
                break;
            };
            if key.0 > now {
                if !(self.clock.paused && self.clock.script_paused) {
                    break;
                }
                // Held by the script: the rest of it moves up to now and
                // keeps its spacing, so the steps after a `resume` keep
                // their distance from it.
                let shift = key.0 - now;
                self.schedule.queue = std::mem::take(&mut self.schedule.queue)
                    .into_iter()
                    .map(|((tick, seq), q)| ((tick - shift, seq), q))
                    .collect();
                continue;
            }
            match self.schedule.queue.remove(&key).expect("present") {
                Queued::Step(step, within) => {
                    // A step run late happened that many ticks earlier.
                    self.inject.stamp = (within - (now - key.0) as f32).max(-1.0);
                    self.execute(world, step);
                    self.inject.stamp = 0.0;
                }
                Queued::Inject(event) => {
                    if !self.inject.deliver(world, &mut *self.host.input, &event) {
                        self.out
                            .note(now, true, format!("type: {} was refused", event.to_json()));
                    }
                }
            }
        }
    }

    /// One tick through the probe: due steps, then the world's step, then
    /// input edges expire. After `quit`, and while a screenshot waits for
    /// its frame, it takes no tick.
    pub fn step(&mut self, world: &mut World) -> og_core::Result<()> {
        self.pump(world);
        if self.clock.quit || self.schedule.shot.is_some() || self.clock.held(world) {
            return Ok(());
        }
        self.tick_once(world)
    }

    /// One frame of the host's loop: due steps, then `due` ticks unless the
    /// clock is held, a screenshot waits for its frame, or the run has
    /// quit. Returns the ticks taken.
    pub fn frame(&mut self, world: &mut World, due: u32) -> og_core::Result<u32> {
        self.pump(world);
        let mut taken = 0;
        while taken < due && !self.clock.paused && !self.clock.quit && self.schedule.shot.is_none()
        {
            self.pump(world);
            if self.clock.quit || self.schedule.shot.is_some() || self.clock.held(world) {
                break;
            }
            // A re-simulated tick is the past again, not one of the frame's.
            let again = self.clock.replaying(world);
            self.tick_once(world)?;
            if !again {
                taken += 1;
            }
        }
        Ok(taken)
    }
}

impl state::Schedule {
    /// Polls a screenshot whose frame is in flight and writes it when it
    /// lands; true while it still waits.
    fn shot_waits(&mut self, world: &World, host: &mut state::Host, out: &mut state::Out) -> bool {
        let Some(shot) = self.shot.as_mut() else {
            return false;
        };
        let frame = match host.capture.as_mut() {
            Some(capture) => capture.capture(world, shot.width, shot.height),
            None => None,
        };
        shot.polls += 1;
        if frame.is_none() && shot.polls < MAX_CAPTURE_POLLS {
            return true;
        }
        let shot = self.shot.take().expect("waiting");
        match frame {
            Some(frame) => out.emit(
                shot.at,
                Output {
                    tick: shot.tick,
                    action: "screenshot".into(),
                    path: shot.path,
                    payload: Payload::Png {
                        width: frame.width,
                        height: frame.height,
                        bytes: frame.png,
                    },
                },
            ),
            None => out.note(
                shot.tick,
                true,
                format!(
                    "{}s screenshot: the renderer produced no frame in {MAX_CAPTURE_POLLS} frames",
                    shot.at
                ),
            ),
        }
        false
    }
}

impl Probe {
    fn tick_once(&mut self, world: &mut World) -> og_core::Result<()> {
        let replaying = self.clock.replaying(world);
        if let Some(before) = self.clock.before_tick.as_mut() {
            before(world);
        }
        let counting = !self.checks.events.is_empty();
        if counting {
            self.count_events(world, true);
        }
        // The tick these systems run as (og.Time.tick inside them).
        let ran = world.time().tick;
        world.step()?;
        if counting {
            self.count_events(world, false);
        }
        if !replaying {
            self.host.input.end_tick(world);
        }
        if let Some(after) = self.clock.after_tick.as_mut() {
            let next = world.time().tick;
            after(world);
            let now = world.time().tick;
            self.shift_schedule(next, now);
        }
        #[cfg(not(feature = "publish"))]
        self.check_invariants(world, ran);
        #[cfg(not(feature = "publish"))]
        if let Some(ui) = self.host.ui.as_ref() {
            let screens = ui.screens(world);
            self.timeline.screens(world.time().tick, screens);
        }
        Ok(())
    }
}

impl state::Inject {
    fn deliver(
        &mut self,
        world: &mut World,
        input: &mut dyn InputSink,
        event: &InjectedInput,
    ) -> bool {
        let taken = self.filter.as_mut().is_some_and(|f| f(world, event));
        let accepted = taken || input.inject_at(world, event, self.stamp);
        if accepted {
            match event {
                InjectedInput::Pointer { x, y, id: 0, .. } | InjectedInput::Wheel { x, y, .. } => {
                    self.pointer = (*x, *y);
                }
                InjectedInput::Pointer { .. }
                | InjectedInput::Key { .. }
                | InjectedInput::Gamepad(_)
                | InjectedInput::Text { .. }
                | InjectedInput::A11y { .. }
                | InjectedInput::Intent { .. }
                | InjectedInput::PointerLock { .. } => {}
            }
        }
        accepted
    }

    fn pointer_event(
        &mut self,
        world: &mut World,
        input: &mut dyn InputSink,
        action: PointerAction,
        button: PointerButton,
    ) -> Result<(), String> {
        let (x, y) = self.pointer;
        let event = InjectedInput::Pointer {
            action,
            x,
            y,
            button,
            id: 0,
            movement_x: 0.0,
            movement_y: 0.0,
        };
        if self.deliver(world, input, &event) {
            Ok(())
        } else {
            Err("the input layer refused the pointer event".into())
        }
    }
}

impl Probe {
    fn execute(&mut self, world: &mut World, step: CommandStep) {
        let tick = world.time().tick;
        self.timeline.step(tick, &step);
        let result = self.action(world, &step);
        if let Err(reason) = result {
            self.out.note(
                tick,
                true,
                format!("{}s {}: {reason}", step.at, step.action),
            );
        }
    }

    fn entity_of(&self, world: &World, id: &str) -> Result<Entity, String> {
        self.names.resolve(world, id).ok_or_else(|| {
            format!("no live entity '{id}' (an entity name, or #<index>.<generation>)")
        })
    }

    pub(crate) fn position_of(&self, world: &World, e: Entity) -> Result<Vec3, String> {
        let tid = engine_type(world, "Transform")?;
        let schema = world.schema(tid).map_err(|e| e.reason().to_owned())?;
        let fi = schema
            .field_index("position")
            .filter(|i| schema.fields[*i].ty == FieldType::Vec3)
            .ok_or("Transform.position must be a vec3")?;
        let bytes = world.get_bytes(e, tid).map_err(|e| e.reason().to_owned())?;
        let range = Layout::of(schema).range(fi);
        Ok(og_core::bytemuck::pod_read_unaligned::<[f32; 3]>(&bytes[range]).into())
    }
}

/// A key with the modifiers held for it, as `key_down` and `key_up` take
/// it: `ArrowLeft`, `Shift+ArrowLeft`, `Meta+Shift+KeyZ` (`Shift`, `Ctrl`
/// or `Control`, `Alt` or `Option`, `Meta`, `Cmd` or `Command`; any case).
/// Command is `MetaLeft`, apart from Control's `ControlLeft`.
fn key_chord(text: &str) -> Result<(Modifiers, Vec<&'static str>, String), String> {
    let mut parts: Vec<&str> = text.split('+').collect();
    let key = parts.pop().unwrap_or_default().trim().to_owned();
    let mut m = Modifiers::default();
    let mut held = Vec::new();
    for p in parts {
        let (flag, code) = match p.trim().to_ascii_lowercase().as_str() {
            "shift" => (&mut m.shift, "ShiftLeft"),
            "ctrl" | "control" => (&mut m.control, "ControlLeft"),
            "alt" | "option" => (&mut m.alt, "AltLeft"),
            "meta" | "cmd" | "command" => (&mut m.platform, "MetaLeft"),
            other => {
                return Err(format!(
                    "'{other}' is not a modifier; write `key_down:Shift+ArrowLeft` with Shift, Ctrl, Alt or Meta (Cmd)"
                ));
            }
        };
        if !*flag {
            *flag = true;
            held.push(code);
        }
    }
    Ok((m, held, key))
}
