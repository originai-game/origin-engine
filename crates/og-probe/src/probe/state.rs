//! [`Probe`]'s states: its fields grouped by the subject that owns them.

use super::*;

/// What the host gave the probe: its input sink, renderer, UI and files,
/// and the hooks its members answer through.
pub(crate) struct Host {
    pub(crate) engine: String,
    pub(crate) input: Box<dyn InputSink>,
    pub(crate) capture: Option<Box<dyn Capture>>,
    pub(crate) ui: Option<Box<dyn UiTree>>,
    pub(crate) reloader: Option<Box<dyn Reloader>>,
    pub(crate) facts: Option<FactsFn>,
    /// The frame as data and picking ([`Probe::set_scene_view`]).
    pub(crate) scene_view: Option<Box<dyn SceneView>>,
    /// The boot scene's write-back ([`Probe::set_scene_writer`]).
    pub(crate) scene_writer: Option<Box<dyn SceneWriter>>,
    /// Reference files for `expect:frame` ([`Probe::set_reader`]).
    pub(crate) reader: Option<ReaderFn>,
    pub(crate) listen: Option<ListenFn>,
    pub(crate) memory: Option<MemoryFn>,
    pub(crate) derive: Option<DeriveFn>,
    pub(crate) register: Option<RegisterFn>,
}

/// Whether the next tick runs: the holds and the host's hooks around a tick.
pub(crate) struct Clock {
    pub(crate) paused: bool,
    /// The clock was held by a script's `pause`: the script's later steps
    /// run in order on the held world instead of waiting for ticks that
    /// will not come.
    pub(crate) script_paused: bool,
    pub(crate) quit: bool,
    /// The host's end-of-tick work ([`Probe::set_after_tick`]).
    pub(crate) after_tick: Option<AfterTickFn>,
    /// The host's start-of-tick work ([`Probe::set_before_tick`]).
    pub(crate) before_tick: Option<AfterTickFn>,
    /// The host's tick gate ([`Probe::set_gate`]).
    pub(crate) gate: Option<GateFn>,
    /// After a rollback's rewind (the gate moved `Time` back): the tick the
    /// world had reached. The ticks below it run again: they take no share
    /// of a frame's ticks, and the input's edges, which belong to the
    /// present tick, stay for it.
    pub(crate) replay_until: u64,
}

/// The script's steps waiting to run, and the screenshot that holds them.
pub(crate) struct Schedule {
    pub(super) queue: BTreeMap<(u64, u64), Queued>,
    pub(super) seq: u64,
    /// A screenshot waiting on its frame: it holds the clock and later steps.
    pub(super) shot: Option<Shot>,
}

/// The command channel: the registered actions and the ones modules push.
pub(crate) struct Commands {
    pub(super) registered: BTreeMap<String, Registered>,
    pub(super) inbox: Inbox,
}

/// What a run hands back to the host: output files and log lines.
pub(crate) struct Out {
    pub(super) outputs: Vec<Output>,
    pub(super) log: Vec<LogLine>,
    /// The tick of the last unnamed `get` and how many it has had, so a
    /// second one on the same tick writes `get-<tick>-2.json`.
    pub(super) gets: (u64, u32),
    /// The files this script's steps wrote, each with the step that wrote
    /// it (its time and action): a second step naming one is written beside
    /// it, suffixed, with a warning naming both ([`Probe::emit`]).
    pub(super) written: BTreeMap<String, (f64, String)>,
}

/// The input being injected: where the pointer is and when in the tick.
pub(crate) struct Inject {
    pub(super) pointer: (f32, f32),
    pub(super) target_missed: bool,
    /// The sub-tick stamp of the input being delivered (InputSink::inject_at).
    pub(super) stamp: f32,
    /// The host's input filter ([`Probe::set_input_filter`]).
    pub(super) filter: Option<InputFilterFn>,
}

/// What a run asserts: its expectations, assertions, invariants and counted events.
pub(crate) struct Checks {
    /// Expectations evaluated since the host last took them.
    pub(crate) expectations: Vec<ExpectResult>,
    /// The run's assertions: the game's, the default invariants' and the
    /// script's `invariant:` steps' (shared with the host, which records
    /// the game's calls while a system runs).
    pub(crate) assertions: Rc<RefCell<Assertions>>,
    /// The script's `invariant:` steps, each an expectation (its words
    /// after `invariant:`) evaluated after every tick.
    pub(crate) invariants: Vec<Vec<String>>,
    /// An invariant is being evaluated: its expectation is not kept or logged.
    pub(crate) quiet: bool,
    /// Event types an expectation counts, with their counts, and each
    /// one's count when the script that asks began.
    pub(crate) events: BTreeMap<String, Watch>,
    pub(crate) event_base: BTreeMap<String, u64>,
}
