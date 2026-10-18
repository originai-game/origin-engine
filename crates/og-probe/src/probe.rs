//! [`Probe`]: RuntimeProbe protocol 5 plus the engine's wire members, and the
//! command channel's execution, over a world the host owns and passes in.

use crate::base64;
use crate::command::{
    self, CORE_ACTIONS, CommandDesc, CommandHandler, CommandInfo, CommandStep, CommandsReport,
    DumpHandler, Inbox, LogLine, Output, ParseError, Payload,
};
use crate::hooks::{Capture, Reloader, UiTree, find_by_id, find_by_text};
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
use std::collections::{BTreeMap, VecDeque};
use std::rc::Rc;

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
/// `hello.engine` unless the host says otherwise.
pub const ENGINE_VERSION: &str = concat!("origin-engine ", env!("CARGO_PKG_VERSION"));

/// Every member this crate implements, in protocol order.
pub const MEMBERS: &[&str] = &[
    "hello", "tick", "state", "advance", "pause", "resume", "input", "accepted", "pick", "entity",
    "bounds", "patch", "run", "capture", "sweep", "reload", "snapshot", "restore", "dump",
    "commands", "listen",
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

/// One probe snapshot: the world (its names included).
#[derive(Clone)]
struct Saved {
    world: Vec<u8>,
    tick: u64,
    hash: u64,
}

enum Queued {
    Step(CommandStep),
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
    engine: String,
    names: Names,
    input: Box<dyn InputSink>,
    capture: Option<Box<dyn Capture>>,
    ui: Option<Box<dyn UiTree>>,
    reloader: Option<Box<dyn Reloader>>,
    facts: Option<FactsFn>,
    commands: BTreeMap<String, Registered>,
    queue: BTreeMap<(u64, u64), Queued>,
    seq: u64,
    snapshots: VecDeque<(String, Saved)>,
    next_snapshot: u64,
    paused: bool,
    quit: bool,
    step_ms: f64,
    render_ms: f64,
    outputs: Vec<Output>,
    log: Vec<LogLine>,
    pointer: (f32, f32),
    target_missed: bool,
    inbox: Inbox,
    listen: Option<ListenFn>,
}

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

fn fault(context: &str, err: Error) -> Error {
    err.context(context)
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
            engine: ENGINE_VERSION.to_owned(),
            names: Names,
            input,
            capture: None,
            ui: None,
            reloader: None,
            facts: None,
            listen: None,
            commands: BTreeMap::new(),
            queue: BTreeMap::new(),
            seq: 0,
            snapshots: VecDeque::new(),
            next_snapshot: 1,
            paused: false,
            quit: false,
            step_ms: 0.0,
            render_ms: 0.0,
            outputs: Vec::new(),
            log: Vec::new(),
            pointer: (0.0, 0.0),
            target_missed: false,
            inbox: Inbox::default(),
        })
    }

    // ---- host setup -------------------------------------------------------

    /// `hello.engine`, such as `origin-engine 0.1.0`.
    pub fn set_engine_version(&mut self, engine: impl Into<String>) {
        self.engine = engine.into();
    }

    /// The renderer: frames for `capture`, `sweep`, `screenshot`, and `pick`.
    pub fn set_capture(&mut self, capture: Box<dyn Capture>) {
        self.capture = Some(capture);
    }

    /// The UI tree for `dump_ui`, `touch_move_to` and `move_to_text`.
    pub fn set_ui(&mut self, ui: Box<dyn UiTree>) {
        self.ui = Some(ui);
    }

    /// The bundle swap for `reload`.
    pub fn set_reloader(&mut self, reloader: Box<dyn Reloader>) {
        self.reloader = Some(reloader);
    }

    /// The game's own facts for `state().facts` (`GameDesc.facts`).
    pub fn set_facts(&mut self, facts: impl Fn(&World) -> Value + 'static) {
        self.facts = Some(Box::new(facts));
    }

    /// Gives `listen` the host's audio: the last seconds its mixer rendered.
    pub fn set_listen(&mut self, listen: impl Fn(f32) -> Option<Heard> + 'static) {
        self.listen = Some(Box::new(listen));
    }

    /// `listen`: the last `seconds` (0 to 2) of audio the host's mixer
    /// rendered, a WAV in base64 with its rate, frames, peak and RMS; `None`
    /// without audio or before anything was rendered.
    pub fn listen(&mut self, world: &World, seconds: f32) -> Option<Value> {
        let h = (self.listen.as_ref()?)(seconds.clamp(0.0, 2.0))?;
        Some(json!({
            "tick": world.time().tick,
            "rate": h.rate,
            "frames": h.frames,
            "peak": h.peak,
            "rms": h.rms,
            "wav": base64::encode(&h.wav),
        }))
    }

    /// The last tick's timings for `state().timings`, measured by the host.
    pub fn set_timings(&mut self, step_ms: f64, render_ms: f64) {
        self.step_ms = step_ms;
        self.render_ms = render_ms;
    }

    /// Probe ids over the world's names (`World::set_name` names entities).
    pub fn names(&self) -> &Names {
        &self.names
    }

    /// Whether the clock is held (the host's loop then takes no ticks).
    pub fn paused(&self) -> bool {
        self.paused
    }

    /// Whether a `quit` ran: the host ends the run.
    pub fn quit_requested(&self) -> bool {
        self.quit
    }

    /// Steps still scheduled.
    pub fn pending(&self) -> usize {
        self.queue.len()
    }

    /// Files commands produced since the last call, for the host to write.
    pub fn take_outputs(&mut self) -> Vec<Output> {
        std::mem::take(&mut self.outputs)
    }

    /// Log lines since the last call.
    pub fn take_log(&mut self) -> Vec<LogLine> {
        std::mem::take(&mut self.log)
    }

    fn note(&mut self, tick: u64, warning: bool, text: String) {
        self.log.push(LogLine {
            tick,
            warning,
            text,
        });
    }

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
        if self.commands.contains_key(name) {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("'{name}' is already registered"),
            ));
        }
        self.commands.insert(
            name.to_owned(),
            Registered {
                desc,
                handler: Handler::Act(Rc::new(handler)),
            },
        );
        Ok(())
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
        self.register_command(name, desc, |_, _| Ok(()))?;
        if let Some(r) = self.commands.get_mut(name) {
            r.handler = Handler::Dump(Rc::new(handler), path_arg);
        }
        Ok(())
    }

    /// The queue engine modules push actions into from inside a tick; they
    /// run at the start of the next tick (before its scheduled steps).
    pub fn inbox(&self) -> Inbox {
        self.inbox.clone()
    }

    /// Every action the channel knows: core actions, then registered ones by name.
    pub fn help(&self) -> Vec<CommandInfo> {
        let core = CORE_ACTIONS.iter().map(|(name, usage, doc)| CommandInfo {
            name: (*name).to_owned(),
            usage: (*usage).to_owned(),
            doc: (*doc).to_owned(),
            core: true,
        });
        let registered = self.commands.iter().map(|(name, r)| CommandInfo {
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

    fn knows(&self, action: &str) -> bool {
        CORE_ACTIONS.iter().any(|(name, _, _)| *name == action)
            || self.commands.contains_key(action)
    }

    /// Schedule a script on simulation time from the current tick: a step at
    /// `t` seconds runs at the start of tick `now + round(t · rate)`.
    pub fn schedule(&mut self, world: &World, script: &str) -> Result<CommandsReport, ParseError> {
        let steps = command::parse(script)?;
        let now = world.time().tick;
        let rate = f64::from(tick_rate(world));
        let mut report = CommandsReport {
            scheduled: 0,
            until_tick: now,
            warnings: Vec::new(),
        };
        for step in steps {
            let tick = now + (step.at * rate).round() as u64;
            if !self.knows(&step.action) {
                report.warnings.push(format!(
                    "{}s: unknown action '{}'; it is skipped unless registered before tick {tick} (`help` lists the actions)",
                    step.at, step.action
                ));
            }
            self.seq += 1;
            self.queue.insert((tick, self.seq), Queued::Step(step));
            report.scheduled += 1;
            report.until_tick = tick;
        }
        Ok(report)
    }

    /// Run the steps due at the current tick (and earlier), in order.
    pub fn pump(&mut self, world: &mut World) {
        for (action, args) in self.inbox.take() {
            if self.quit {
                break;
            }
            self.execute(
                world,
                CommandStep {
                    at: 0.0,
                    action,
                    args,
                },
            );
        }
        while !self.quit {
            let now = world.time().tick;
            let Some((&key, _)) = self.queue.first_key_value() else {
                break;
            };
            if key.0 > now {
                break;
            }
            match self.queue.remove(&key).expect("present") {
                Queued::Step(step) => self.execute(world, step),
                Queued::Inject(event) => {
                    if !self.deliver(world, &event) {
                        self.note(now, true, format!("type: {} was refused", event.to_json()));
                    }
                }
            }
        }
    }

    /// One tick through the probe: due steps, then the world's step, then
    /// input edges expire. After `quit` it takes no tick.
    pub fn step(&mut self, world: &mut World) -> og_core::Result<()> {
        self.pump(world);
        if self.quit {
            return Ok(());
        }
        self.tick_once(world)
    }

    /// One frame of the host's loop: due steps, then `due` ticks unless the
    /// clock is held or the run has quit. Returns the ticks taken.
    pub fn frame(&mut self, world: &mut World, due: u32) -> og_core::Result<u32> {
        self.pump(world);
        let mut taken = 0;
        while taken < due && !self.paused && !self.quit {
            self.step(world)?;
            taken += 1;
        }
        Ok(taken)
    }

    fn tick_once(&mut self, world: &mut World) -> og_core::Result<()> {
        world.step()?;
        self.input.end_tick(world);
        Ok(())
    }

    fn deliver(&mut self, world: &mut World, event: &InjectedInput) -> bool {
        let accepted = self.input.inject(world, event);
        if accepted {
            match event {
                InjectedInput::Pointer { x, y, id: 0, .. } | InjectedInput::Wheel { x, y, .. } => {
                    self.pointer = (*x, *y);
                }
                InjectedInput::Pointer { .. }
                | InjectedInput::Key { .. }
                | InjectedInput::Gamepad(_) => {}
            }
        }
        accepted
    }

    fn pointer_event(
        &mut self,
        world: &mut World,
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
        };
        if self.deliver(world, &event) {
            Ok(())
        } else {
            Err("the input layer refused the pointer event".into())
        }
    }

    fn execute(&mut self, world: &mut World, step: CommandStep) {
        let tick = world.time().tick;
        let result = self.action(world, &step);
        if let Err(reason) = result {
            self.note(
                tick,
                true,
                format!("{}s {}: {reason}", step.at, step.action),
            );
        }
    }

    fn action(&mut self, world: &mut World, step: &CommandStep) -> Result<(), String> {
        let tick = world.time().tick;
        let args = &step.args;
        let arg = |i: usize, usage: &str| -> Result<&str, String> {
            args.get(i)
                .map(String::as_str)
                .filter(|a| !a.is_empty())
                .ok_or_else(|| format!("missing argument; write `{usage}`"))
        };
        let joined = args.join(":");
        match step.action.as_str() {
            "screenshot" => {
                let mut path = None;
                let mut edge = None;
                for a in args {
                    if a == "full" {
                        edge = None;
                    } else if let Ok(n) = a.parse::<u32>() {
                        edge = Some(n.clamp(1, MAX_CAPTURE_EDGE));
                    } else if path.is_none() && !a.is_empty() {
                        path = Some(a.clone());
                    } else {
                        return Err(format!(
                            "unexpected argument '{a}'; write `screenshot[:path][:full|:N]`"
                        ));
                    }
                }
                let capture = self
                    .capture
                    .as_mut()
                    .ok_or("no renderer in this host: nothing to capture")?;
                let (width, height) = match edge {
                    None => (0, 0),
                    Some(n) => {
                        let (tw, th) = capture.target_size();
                        let (tw, th) = (f64::from(tw.max(1)), f64::from(th.max(1)));
                        if tw >= th {
                            (n, ((f64::from(n) * th / tw).round() as u32).max(1))
                        } else {
                            (((f64::from(n) * tw / th).round() as u32).max(1), n)
                        }
                    }
                };
                let frame = capture
                    .capture(world, width, height)
                    .ok_or("the renderer produced no frame")?;
                self.outputs.push(Output {
                    tick,
                    action: step.action.clone(),
                    path,
                    payload: Payload::Png {
                        width: frame.width,
                        height: frame.height,
                        bytes: frame.png,
                    },
                });
                Ok(())
            }
            "listen" => {
                let mut path = None;
                let mut seconds = 1.0f32;
                for a in args {
                    if let Ok(s) = a.parse::<f32>() {
                        seconds = s.clamp(0.0, 2.0);
                    } else if path.is_none() && !a.is_empty() {
                        path = Some(a.clone());
                    }
                }
                let listen = self
                    .listen
                    .as_ref()
                    .ok_or("no audio in this host: nothing to listen to")?;
                let heard = listen(seconds).ok_or("the mixer has rendered nothing yet")?;
                self.outputs.push(Output {
                    tick,
                    action: step.action.clone(),
                    path,
                    payload: Payload::Wav(heard.wav),
                });
                Ok(())
            }
            "touch_move_to" | "move_to_text" => {
                let target = if step.action == "touch_move_to" {
                    arg(0, "touch_move_to:<ui id>")?.to_owned()
                } else {
                    if joined.is_empty() {
                        return Err("missing argument; write `move_to_text:[=]text[@N]`".into());
                    }
                    joined.clone()
                };
                let elements = match &self.ui {
                    Some(ui) => ui.elements(world),
                    None => Vec::new(),
                };
                let found = if step.action == "touch_move_to" {
                    find_by_id(&elements, &target)
                } else {
                    find_by_text(&elements, &target)
                };
                let Some(element) = found else {
                    self.target_missed = true;
                    return Err(format!(
                        "target not found: '{target}' ({} UI elements; `dump_ui` lists them); the next touch_start/touch_end are skipped",
                        elements.len()
                    ));
                };
                self.target_missed = false;
                let (x, y) = element.center();
                self.pointer = (x, y);
                self.pointer_event(world, PointerAction::Move, PointerButton::None)
            }
            "touch_move" => {
                let xy = numbers(arg(0, "touch_move:x,y")?)?;
                let [x, y] = xy[..] else {
                    return Err("write `touch_move:x,y` with two numbers".into());
                };
                self.target_missed = false;
                self.pointer = (x, y);
                self.pointer_event(world, PointerAction::Move, PointerButton::None)
            }
            "touch_start" | "touch_end" => {
                if self.target_missed {
                    if step.action == "touch_end" {
                        self.target_missed = false;
                    }
                    return Err("skipped: the last touch target was not found".into());
                }
                let action = if step.action == "touch_start" {
                    PointerAction::Down
                } else {
                    PointerAction::Up
                };
                self.pointer_event(world, action, PointerButton::Left)
            }
            "scroll" => {
                let d = numbers(arg(0, "scroll:dx,dy")?)?;
                let [dx, dy] = d[..] else {
                    return Err("write `scroll:dx,dy` with two numbers".into());
                };
                let (x, y) = self.pointer;
                let event = InjectedInput::Wheel {
                    x,
                    y,
                    delta_x: dx,
                    delta_y: dy,
                };
                if self.deliver(world, &event) {
                    Ok(())
                } else {
                    Err("the input layer refused the wheel event".into())
                }
            }
            "touch" => {
                let usage = "touch:<id>:<down|move|up>:<x,y | ui id>";
                let id: u32 = arg(0, usage)?
                    .parse()
                    .ok()
                    .filter(|id| (1..og_input::MAX_POINTERS as u32).contains(id))
                    .ok_or("the touch id is 1 to 4 (0 is the pointer touch_start drives)")?;
                let action = match arg(1, usage)? {
                    "down" => PointerAction::Down,
                    "move" => PointerAction::Move,
                    "up" => PointerAction::Up,
                    other => {
                        return Err(format!(
                            "'{other}' is not down, move or up; write `{usage}`"
                        ));
                    }
                };
                let target = step.args.get(2..).map(|a| a.join(":")).unwrap_or_default();
                let (x, y) = match numbers(&target).ok().as_deref() {
                    Some(&[x, y]) => (x, y),
                    _ => {
                        let elements = match &self.ui {
                            Some(ui) => ui.elements(world),
                            None => Vec::new(),
                        };
                        let element = find_by_id(&elements, &target).ok_or_else(|| {
                            format!("touch target not found: '{target}' (x,y or a UI id)")
                        })?;
                        element.center()
                    }
                };
                let event = InjectedInput::Pointer {
                    action,
                    x,
                    y,
                    button: if action == PointerAction::Move {
                        PointerButton::None
                    } else {
                        PointerButton::Left
                    },
                    id,
                };
                if self.deliver(world, &event) {
                    Ok(())
                } else {
                    Err(format!("the input layer refused touch {id}"))
                }
            }
            "gamepad" => {
                let usage = "gamepad:<a+b+...|->[:lx,ly[,rx,ry]][:lt,rt]";
                let first = arg(0, usage)?;
                let mut state = og_input::GamepadState {
                    connected: first != "off",
                    ..Default::default()
                };
                if state.connected && first != "-" {
                    for name in first.split('+') {
                        let b = og_input::gamepad_button(name).ok_or_else(|| {
                            format!(
                                "'{name}' is not a standard gamepad button ({})",
                                og_input::GAMEPAD_BUTTONS.join(", ")
                            )
                        })?;
                        state.buttons |= 1 << b;
                    }
                }
                if let Some(axes) = step.args.get(1) {
                    let v = numbers(axes)?;
                    if v.len() != 2 && v.len() != 4 {
                        return Err(format!(
                            "the axes are lx,ly or lx,ly,rx,ry; write `{usage}`"
                        ));
                    }
                    state.axes[..v.len()].copy_from_slice(&v);
                }
                if let Some(t) = step.args.get(2) {
                    let v = numbers(t)?;
                    let [lt, rt] = v[..] else {
                        return Err(format!("the triggers are lt,rt; write `{usage}`"));
                    };
                    state.triggers = [lt, rt];
                }
                if self.deliver(world, &InjectedInput::Gamepad(state)) {
                    Ok(())
                } else {
                    Err("the input layer refused the gamepad state".into())
                }
            }
            "key_down" | "key_up" => {
                let key = arg(0, "key_down:<Key>")?.to_owned();
                let action = if step.action == "key_down" {
                    KeyAction::Down
                } else {
                    KeyAction::Up
                };
                let event = InjectedInput::Key {
                    key: key.clone(),
                    action,
                    modifiers: Modifiers::default(),
                };
                if self.deliver(world, &event) {
                    Ok(())
                } else {
                    Err(format!(
                        "the input layer refused key '{key}'; keys are KeyboardEvent.code names such as KeyW, Space, ArrowLeft"
                    ))
                }
            }
            "type" => {
                if joined.is_empty() {
                    return Err("missing argument; write `type:<text>`".into());
                }
                let mut skipped = Vec::new();
                let mut at = tick;
                for c in joined.chars() {
                    let Some((key, shift)) = key_for_char(c) else {
                        skipped.push(c);
                        continue;
                    };
                    let event = InjectedInput::Key {
                        key,
                        action: KeyAction::Press,
                        modifiers: Modifiers {
                            shift,
                            ..Modifiers::default()
                        },
                    };
                    // One press per two ticks: down for one tick, up for the next.
                    self.seq += 1;
                    self.queue.insert((at, self.seq), Queued::Inject(event));
                    at += 2;
                }
                if skipped.is_empty() {
                    Ok(())
                } else {
                    Err(format!("no key types {skipped:?}; the rest is typed"))
                }
            }
            "teleport" => {
                let id = arg(0, "teleport:<entity>:x,y,z")?;
                let e = self.entity_of(world, id)?;
                let mut xyz = numbers(arg(1, "teleport:<entity>:x,y,z")?)?;
                let tid = engine_type(world, "Transform")?;
                let current = world.get_bytes(e, tid).map_err(|e| e.reason().to_owned())?;
                let schema = world
                    .schema(tid)
                    .map_err(|e| e.reason().to_owned())?
                    .clone();
                let fi = schema
                    .field_index("position")
                    .ok_or("Transform has no position field")?;
                let lanes = schema.fields[fi].ty.lanes();
                if xyz.len() == lanes - 1 {
                    let layout = Layout::of(&schema);
                    let old = value::field_json(schema.fields[fi].ty, &current[layout.range(fi)]);
                    xyz.push(old[lanes - 1].as_f64().unwrap_or(0.0) as f32);
                }
                if xyz.len() != lanes {
                    return Err(format!("Transform.position takes {lanes} numbers"));
                }
                let parameter = format!("{}.position", schema.name);
                self.write_field(world, e, &parameter, &json!(xyz), false)
            }
            "frame_entity" => {
                let id = arg(0, "frame_entity:<entity>")?;
                let e = self.entity_of(world, id)?;
                let target = self.position_of(world, e)?;
                let cid = engine_type(world, "Camera")?;
                let schema = world
                    .schema(cid)
                    .map_err(|e| e.reason().to_owned())?
                    .clone();
                let mut bytes = world
                    .resource_bytes(cid)
                    .map_err(|_| "no Camera resource in the world".to_owned())?;
                let layout = Layout::of(&schema);
                let (Some(pi), Some(qi)) = (
                    schema.field_index("position"),
                    schema.field_index("quaternion"),
                ) else {
                    return Err("Camera needs position and quaternion fields".into());
                };
                if schema.fields[pi].ty != FieldType::Vec3
                    || schema.fields[qi].ty != FieldType::Quat
                {
                    return Err(
                        "Camera.position must be a vec3 and Camera.quaternion a quat".into(),
                    );
                }
                let eye: Vec3 =
                    og_core::bytemuck::pod_read_unaligned::<[f32; 3]>(&bytes[layout.range(pi)])
                        .into();
                let rotation = look_at(eye, target)
                    .ok_or("the camera is at the entity; nothing to look at")?;
                bytes[layout.range(qi)]
                    .copy_from_slice(og_core::bytemuck::bytes_of(&rotation.to_array()));
                world
                    .insert_resource_bytes(cid, &bytes)
                    .map_err(|e| e.reason().to_owned())
            }
            "dump_world" | "dump_ui" => {
                let dump = if step.action == "dump_world" {
                    self.dump_world(world)
                } else {
                    self.dump_ui(world)
                };
                self.outputs.push(Output {
                    tick,
                    action: step.action.clone(),
                    path: args.first().filter(|p| !p.is_empty()).cloned(),
                    payload: Payload::Json(dump),
                });
                Ok(())
            }
            "dump_entity" => {
                let id = arg(0, "dump_entity:<entity>[:path]")?;
                let e = self.entity_of(world, id)?;
                let dump = self.dump_entity(world, e);
                self.outputs.push(Output {
                    tick,
                    action: step.action.clone(),
                    path: args.get(1).filter(|p| !p.is_empty()).cloned(),
                    payload: Payload::Json(dump),
                });
                Ok(())
            }
            "set" => {
                let usage = "set:<entity>.<component>.<field>=<value> or set:<Resource>.<field>=<value>";
                let (target, text) = joined
                    .split_once('=')
                    .ok_or_else(|| format!("write `{usage}`"))?;
                let value = value::text_json(text)?;
                if let Some((resource, field)) = Self::resource_target(world, target) {
                    return Self::write_resource_field(world, resource, &field, &value);
                }
                let (entity, parameter) = match target.strip_prefix('#') {
                    Some(rest) => {
                        let mut parts = rest.splitn(3, '.');
                        match (parts.next(), parts.next(), parts.next()) {
                            (Some(i), Some(g), Some(p)) => (format!("#{i}.{g}"), p.to_owned()),
                            _ => return Err(format!("write `{usage}`")),
                        }
                    }
                    None => {
                        let (entity, parameter) = target
                            .split_once('.')
                            .ok_or_else(|| format!("write `{usage}`"))?;
                        (entity.to_owned(), parameter.to_owned())
                    }
                };
                let e = self.entity_of(world, &entity)?;
                self.write_field(world, e, &parameter, &value, false)
            }
            "get" => {
                let usage = "get:<Resource>[.<field>] or get:<entity>[.<component>[.<field>]]";
                let target = arg(0, usage)?.trim();
                let value = self.read_target(world, target)?;
                let mut shown = value.to_string();
                if shown.len() > 160 {
                    let mut end = 157;
                    while !shown.is_char_boundary(end) {
                        end -= 1;
                    }
                    shown.truncate(end);
                    shown.push_str("...");
                }
                self.note(tick, false, format!("get {target} = {shown}"));
                self.outputs.push(Output {
                    tick,
                    action: "get".into(),
                    path: args.get(1).filter(|p| !p.is_empty()).cloned(),
                    payload: Payload::Json(json!({
                        "kind": "get",
                        "target": target,
                        "tick": tick,
                        "value": value,
                    })),
                });
                Ok(())
            }
            "seed" => {
                let seed: u32 = arg(0, "seed:<n>")?
                    .trim()
                    .parse()
                    .map_err(|_| "the seed is a whole number in [0, 4294967295]".to_owned())?;
                *world
                    .resource_mut::<Rng>()
                    .map_err(|e| e.reason().to_owned())? = Rng::new(seed);
                Ok(())
            }
            "pause" => {
                self.paused = true;
                Ok(())
            }
            "resume" => {
                self.paused = false;
                Ok(())
            }
            "advance" => {
                let ticks: u64 = arg(0, "advance:<ticks>")?
                    .trim()
                    .parse()
                    .ok()
                    .filter(|t| *t <= PLAYTEST_MAX_TICKS)
                    .ok_or_else(|| {
                        format!("ticks is a whole number in [0, {PLAYTEST_MAX_TICKS}]")
                    })?;
                self.advance(world, ticks)
                    .map(|_| ())
                    .map_err(|e| e.reason().to_owned())
            }
            "quit" => {
                self.quit = true;
                Ok(())
            }
            "help" => {
                let mut text = String::new();
                for info in self.help() {
                    let origin = if info.core { "" } else { " (registered)" };
                    text.push_str(&format!("{}{origin}\n    {}\n", info.usage, info.doc));
                }
                self.note(tick, false, format!("help: {} actions", self.help().len()));
                self.outputs.push(Output {
                    tick,
                    action: "help".into(),
                    path: args.first().filter(|p| !p.is_empty()).cloned(),
                    payload: Payload::Text(text),
                });
                Ok(())
            }
            name => {
                let Some(registered) = self.commands.get(name) else {
                    return Err(format!(
                        "unknown action '{name}', skipped; `help` lists the actions"
                    ));
                };
                match registered.handler.clone() {
                    Handler::Act(handler) => handler(world, args),
                    Handler::Dump(handler, path_arg) => {
                        let (head, path) = if args.len() > path_arg {
                            (
                                &args[..path_arg],
                                args.get(path_arg).filter(|p| !p.is_empty()).cloned(),
                            )
                        } else {
                            (&args[..], None)
                        };
                        let value = handler(world, head)?;
                        self.outputs.push(Output {
                            tick,
                            action: name.to_owned(),
                            path,
                            payload: Payload::Json(value),
                        });
                        Ok(())
                    }
                }
            }
        }
    }

    fn entity_of(&self, world: &World, id: &str) -> Result<Entity, String> {
        self.names.resolve(world, id).ok_or_else(|| {
            format!("no live entity '{id}' (an entity name, or #<index>.<generation>)")
        })
    }

    fn position_of(&self, world: &World, e: Entity) -> Result<Vec3, String> {
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

    // ---- schema-driven reads and writes -----------------------------------

    /// The (component, schema, field index) a `Component.field` names.
    fn field_ref(world: &World, parameter: &str) -> Result<(SchemaId, Schema, usize), String> {
        let (component, field) = parameter
            .rsplit_once('.')
            .ok_or_else(|| format!("'{parameter}' is not `Component.field`"))?;
        let id = world
            .id_by_name(component)
            .map_err(|_| format!("no component '{component}' is registered"))?;
        let schema = world.schema(id).map_err(|e| e.reason().to_owned())?.clone();
        let index = schema
            .field_index(field)
            .ok_or_else(|| format!("'{component}' has no field '{field}'"))?;
        Ok((id, schema, index))
    }

    /// Write one field of an entity's component through the schema. With
    /// `tunable_only`, a field without the `tunable` flag is refused.
    fn write_field(
        &self,
        world: &mut World,
        e: Entity,
        parameter: &str,
        value: &Value,
        tunable_only: bool,
    ) -> Result<(), String> {
        let (id, schema, index) = Self::field_ref(world, parameter)?;
        let field = &schema.fields[index];
        if tunable_only && !field.tunable {
            return Err(format!("'{parameter}' is not tunable"));
        }
        let mut bytes = world.get_bytes(e, id).map_err(|e| e.reason().to_owned())?;
        let packed = value::json_field(field, value).map_err(|r| format!("{parameter}: {r}"))?;
        bytes[Layout::of(&schema).range(index)].copy_from_slice(&packed);
        world
            .insert_bytes(e, id, &bytes)
            .map_err(|e| e.reason().to_owned())
    }

    /// `Resource.field` when the part before the last dot names a resource
    /// the world holds (resource names may themselves hold dots: `og.Camera`).
    fn resource_target(world: &World, target: &str) -> Option<(SchemaId, String)> {
        let (name, field) = target.rsplit_once('.')?;
        let id = world.id_by_name(name).ok()?;
        world
            .has_resource_id(id)
            .then(|| (id, field.to_owned()))
    }

    /// Write one field of a resource through its schema.
    fn write_resource_field(
        world: &mut World,
        id: SchemaId,
        field: &str,
        value: &Value,
    ) -> Result<(), String> {
        let schema = world.schema(id).map_err(|e| e.reason().to_owned())?.clone();
        let index = schema
            .field_index(field)
            .ok_or_else(|| format!("'{}' has no field '{field}'", schema.name))?;
        let mut bytes = world
            .resource_bytes(id)
            .map_err(|e| e.reason().to_owned())?;
        let packed = value::json_field(&schema.fields[index], value)
            .map_err(|r| format!("{}.{field}: {r}", schema.name))?;
        bytes[Layout::of(&schema).range(index)].copy_from_slice(&packed);
        world
            .insert_resource_bytes(id, &bytes)
            .map_err(|e| e.reason().to_owned())
    }

    /// What `get` reads: a resource or one of its fields, an entity (its
    /// dump), one of its components or one field.
    fn read_target(&self, world: &World, target: &str) -> Result<Value, String> {
        let resource = |name: &str| {
            world
                .id_by_name(name)
                .ok()
                .filter(|&id| world.has_resource_id(id))
        };
        if let Some(id) = resource(target) {
            let schema = world.schema(id).map_err(|e| e.reason().to_owned())?;
            let bytes = world
                .resource_bytes(id)
                .map_err(|e| e.reason().to_owned())?;
            return Ok(Value::Object(value::component_json(schema, &bytes)));
        }
        if let Some((id, field)) = Self::resource_target(world, target) {
            let schema = world.schema(id).map_err(|e| e.reason().to_owned())?;
            let bytes = world
                .resource_bytes(id)
                .map_err(|e| e.reason().to_owned())?;
            return value::component_json(schema, &bytes)
                .remove(&field)
                .ok_or_else(|| format!("'{}' has no field '{field}'", schema.name));
        }
        // An entity: `#i.g` or a name, then `.Component[.field]`, where a
        // component's name may hold dots (`og.Transform`).
        let (entity, rest) = match target.strip_prefix('#') {
            Some(r) => {
                let mut parts = r.splitn(3, '.');
                match (parts.next(), parts.next(), parts.next()) {
                    (Some(i), Some(g), rest) => (format!("#{i}.{g}"), rest.unwrap_or("")),
                    _ => return Err(format!("no resource or live entity '{target}'")),
                }
            }
            None => match target.split_once('.') {
                Some((e, rest)) => (e.to_owned(), rest),
                None => (target.to_owned(), ""),
            },
        };
        let e = self.names.resolve(world, &entity).ok_or_else(|| {
            format!("no resource or live entity '{entity}' (a resource name, an entity name, or #<index>.<generation>)")
        })?;
        if rest.is_empty() {
            return Ok(self.dump_entity(world, e));
        }
        let read = |name: &str| -> Option<Map<String, Value>> {
            let id = world.id_by_name(name).ok()?;
            let schema = world.schema(id).ok()?;
            let bytes = world.get_bytes(e, id).ok()?;
            Some(value::component_json(schema, &bytes))
        };
        if let Some(fields) = read(rest) {
            return Ok(Value::Object(fields));
        }
        let (component, field) = rest
            .rsplit_once('.')
            .ok_or_else(|| format!("'{entity}' has no component '{rest}'"))?;
        let mut fields =
            read(component).ok_or_else(|| format!("'{entity}' has no component '{component}'"))?;
        fields
            .remove(field)
            .ok_or_else(|| format!("'{component}' has no field '{field}'"))
    }

    /// Each component of an entity with its schema and packed value.
    fn components(world: &World, e: Entity) -> Vec<(Schema, Vec<u8>)> {
        world
            .components_of(e)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|id| {
                let schema = world.schema(id).ok()?.clone();
                let bytes = world.get_bytes(e, id).ok()?;
                Some((schema, bytes))
            })
            .collect()
    }

    // ---- RuntimeProbe protocol 5 ------------------------------------------

    /// `hello`: who is speaking.
    pub fn hello(&self, world: &World) -> Value {
        json!({
            "protocol": PROBE_VERSION,
            "wire": PROBE_WIRE_VERSION,
            "substrate": "origin-engine",
            "engine": self.engine,
            "ticksPerSecond": tick_rate(world),
            "tier": self.capture.as_ref().map_or("none", |c| c.tier()),
            "build": BUILD,
            "members": served_members(),
        })
    }

    /// `tick`: fixed steps since load.
    pub fn tick(&self, world: &World) -> u64 {
        world.time().tick
    }

    /// `state`: the fixed head, then the game's facts.
    pub fn state(&self, world: &World) -> Value {
        let seed = world.resource::<Rng>().map_or(0, |r| r.seed());
        let entities: Vec<Value> = self
            .names
            .live(world)
            .into_iter()
            .map(|(id, _)| json!({ "id": id }))
            .collect();
        json!({
            "tick": world.time().tick,
            "paused": self.paused,
            "seed": seed,
            "timings": {"step": self.step_ms, "render": self.render_ms, "frameMs": self.step_ms + self.render_ms},
            "entities": entities,
            "facts": self.facts.as_ref().map_or(Value::Null, |f| f(world)),
        })
    }

    /// `advance`: take whole fixed steps regardless of the wall clock,
    /// running the command steps that come due on the way. `Err` is a system
    /// that failed; the world stops at that tick.
    pub fn advance(&mut self, world: &mut World, steps: u64) -> og_core::Result<u64> {
        for _ in 0..steps {
            self.pump(world);
            if self.quit {
                break;
            }
            self.tick_once(world)?;
        }
        Ok(world.time().tick)
    }

    /// `pause`: hold the clock.
    pub fn pause(&mut self) -> bool {
        self.paused = true;
        true
    }

    /// `resume`: release the clock.
    pub fn resume(&mut self) -> bool {
        self.paused = false;
        true
    }

    /// `input`: one event through the input layer's own path.
    pub fn input(&mut self, world: &mut World, event: &InjectedInput) -> bool {
        self.deliver(world, event)
    }

    /// `accepted`: events the input layer accepted since load.
    pub fn accepted(&self) -> u32 {
        self.input.accepted()
    }

    /// `pick`: what is under a point; `None` without a renderer that picks.
    pub fn pick(&mut self, world: &World, x: f32, y: f32) -> Option<Value> {
        let hit = self.capture.as_mut()?.pick(world, x, y)?;
        Some(json!({
            "entity": self.names.id_of(world, hit.entity),
            "world": hit.world,
            "source": hit.source,
        }))
    }

    /// `entity`: the entity's tunable fields as `Component.field` keys.
    pub fn entity(&self, world: &World, id: &str) -> Option<Map<String, Value>> {
        let e = self.names.resolve(world, id)?;
        let mut out = Map::new();
        for (schema, bytes) in Self::components(world, e) {
            let layout = Layout::of(&schema);
            for (i, f) in schema.fields.iter().enumerate().filter(|(_, f)| f.tunable) {
                out.insert(
                    format!("{}.{}", schema.name, f.name),
                    value::field_json(f.ty, &bytes[layout.range(i)]),
                );
            }
        }
        Some(out)
    }

    /// `bounds`: the ranges of the entity's tunable fields, from the schema.
    pub fn bounds(&self, world: &World, id: &str) -> Option<Map<String, Value>> {
        let e = self.names.resolve(world, id)?;
        let mut out = Map::new();
        for (schema, _) in Self::components(world, e) {
            for f in schema.fields.iter().filter(|f| f.tunable) {
                let path = format!("{}.{}", schema.name, f.name);
                let mut tunable = Map::new();
                tunable.insert("path".into(), Value::String(path.clone()));
                if let Some(label) = &f.label {
                    tunable.insert("label".into(), Value::String(label.clone()));
                }
                for (key, bound) in [("min", f.min), ("max", f.max)] {
                    if let Some(n) = bound.and_then(serde_json::Number::from_f64) {
                        tunable.insert(key.into(), Value::Number(n));
                    }
                }
                out.insert(path, Value::Object(tunable));
            }
        }
        Some(out)
    }

    /// `patch`: move one tunable field for this run only. `false` for an
    /// unknown entity or field, a field that is not tunable, or a value of
    /// another kind or outside the schema's range.
    pub fn patch(&mut self, world: &mut World, id: &str, parameter: &str, value: &Value) -> bool {
        match self.names.resolve(world, id) {
            Some(e) => self.write_field(world, e, parameter, value, true).is_ok(),
            None => false,
        }
    }

    /// `run`: a program of members, stopping at the first refusal.
    pub fn run(&mut self, world: &mut World, program: &[Value]) -> og_core::Result<Value> {
        let mut results = Vec::new();
        if program.len() > PROBE_PROGRAM_MAX_OPERATIONS {
            return Ok(json!({"results": results, "refused": 0, "tick": self.tick(world)}));
        }
        for (index, operation) in program.iter().enumerate() {
            let op = operation.get("op").and_then(Value::as_str).unwrap_or("");
            let answer: Option<Value> = match op {
                "advance" => match operation
                    .get("steps")
                    .and_then(Value::as_u64)
                    .filter(|s| (1..=PLAYTEST_MAX_TICKS).contains(s))
                {
                    Some(steps) => Some(json!(self.advance(world, steps)?)),
                    None => None,
                },
                "input" => Some(json!(
                    operation
                        .get("event")
                        .and_then(InjectedInput::from_json)
                        .is_some_and(|e| self.input(world, &e))
                )),
                "pause" => Some(json!(self.pause())),
                "resume" => Some(json!(self.resume())),
                "read" => Some(json!({"tick": self.tick(world), "state": self.state(world)})),
                "pick" => {
                    let x = operation
                        .get("x")
                        .and_then(Value::as_f64)
                        .filter(|v| v.is_finite());
                    let y = operation
                        .get("y")
                        .and_then(Value::as_f64)
                        .filter(|v| v.is_finite());
                    match (x, y) {
                        (Some(x), Some(y)) => {
                            Some(self.pick(world, x as f32, y as f32).unwrap_or(Value::Null))
                        }
                        _ => None,
                    }
                }
                "entity" => operation
                    .get("id")
                    .and_then(Value::as_str)
                    .and_then(|id| self.entity(world, id))
                    .map(Value::Object),
                _ => None,
            };
            match answer {
                None | Some(Value::Bool(false)) => {
                    return Ok(
                        json!({"results": results, "refused": index, "tick": self.tick(world)}),
                    );
                }
                Some(value) => results.push(value),
            }
        }
        Ok(json!({"results": results, "refused": Value::Null, "tick": self.tick(world)}))
    }

    // ---- the engine's wire members ----------------------------------------

    /// `capture`: the frame at the current tick; `None` without a renderer.
    pub fn capture(&mut self, world: &World, width: u32, height: u32) -> Option<Value> {
        let frame = self.capture.as_mut()?.capture(world, width, height)?;
        Some(json!({
            "tick": world.time().tick,
            "width": frame.width,
            "height": frame.height,
            "png": base64::encode(&frame.png),
        }))
    }

    fn save(&self, world: &World) -> Saved {
        let bytes = world.snapshot();
        Saved {
            hash: world.world_hash(),
            world: bytes,
            tick: world.time().tick,
        }
    }

    fn load(&mut self, world: &mut World, saved: &Saved) -> og_core::Result<()> {
        world.restore(&saved.world)?;
        Ok(())
    }

    /// `sweep`: per value, restore the snapshot taken at the call, patch,
    /// advance (no command steps run) and capture; the world is restored
    /// afterwards. `None` for an unknown entity or parameter, a refused
    /// value, or no renderer. `Err` is a system that failed.
    pub fn sweep(&mut self, world: &mut World, desc: &SweepDesc) -> og_core::Result<Option<Value>> {
        if self.capture.is_none() {
            return Ok(None);
        }
        let Some(e) = self.names.resolve(world, &desc.id) else {
            return Ok(None);
        };
        let tunable = Self::field_ref(world, &desc.parameter).is_ok_and(|(id, schema, i)| {
            schema.fields[i].tunable && world.get_bytes(e, id).is_ok()
        });
        if !tunable {
            return Ok(None);
        }
        let saved = self.save(world);
        let result = self.sweep_values(world, e, desc, &saved);
        self.load(world, &saved)
            .map_err(|err| fault("sweep could not restore the world", err))?;
        result
    }

    fn sweep_values(
        &mut self,
        world: &mut World,
        e: Entity,
        desc: &SweepDesc,
        saved: &Saved,
    ) -> og_core::Result<Option<Value>> {
        let mut frames = Vec::with_capacity(desc.values.len());
        for value in &desc.values {
            self.load(world, saved)?;
            if self
                .write_field(world, e, &desc.parameter, value, true)
                .is_err()
            {
                return Ok(None);
            }
            for _ in 0..desc.steps {
                self.tick_once(world)?;
            }
            let capture = self.capture.as_mut().expect("checked");
            let Some(frame) = capture.capture(world, desc.width, desc.height) else {
                return Ok(None);
            };
            frames.push(json!({
                "value": value,
                "tick": world.time().tick,
                "png": base64::encode(&frame.png),
            }));
        }
        Ok(Some(json!({ "frames": frames })))
    }

    /// `reload`: snapshot, swap the bundle through the [`Reloader`],
    /// re-register the old world's types by name in the new one, restore.
    /// `None` (the old bundle and world keep running, the reason in the log)
    /// without a reloader, when the bundle fails to load, or when a type
    /// changed layout (field-by-field migration needs og-ecs support).
    pub fn reload(&mut self, world: &mut World) -> Option<Value> {
        let tick = world.time().tick;
        let Some(reloader) = self.reloader.as_mut() else {
            self.note(tick, true, "reload: this host has no bundle loader".into());
            return None;
        };
        let saved = world.snapshot();
        let rebuilt = match reloader.rebuild(world) {
            Ok(rebuilt) => rebuilt,
            Err(why) => {
                self.note(
                    tick,
                    true,
                    format!(
                        "reload: the new bundle failed to load ({why}); the old one keeps running"
                    ),
                );
                return None;
            }
        };
        let mut next = rebuilt.world;
        if let Err(err) = self.input.attach(&mut next) {
            self.note(tick, true, format!("reload: input: {}", err.reason()));
            return None;
        }
        // Types by name, fields by name (og-ecs `restore_migrating`): added
        // fields start at their defaults, removed fields and types are dropped.
        let report = match next.restore_migrating(&saved) {
            Ok(r) => r,
            Err(err) => {
                self.note(
                    tick,
                    true,
                    format!(
                        "reload: restore into the new bundle failed: {}; the old one keeps running",
                        err.reason()
                    ),
                );
                return None;
            }
        };
        let warnings = report.warnings.clone();
        let migrated: Vec<String> = report
            .migrated
            .iter()
            .chain(report.added.iter())
            .cloned()
            .collect();
        let dropped: Vec<String> = report
            .dropped
            .iter()
            .chain(report.removed.iter())
            .cloned()
            .collect();
        *world = next;
        Some(json!({
            "tick": world.time().tick,
            "ms": rebuilt.ms,
            "migrated": migrated,
            "dropped": dropped,
            "warnings": warnings,
        }))
    }

    /// `snapshot`: the whole world, kept in this process.
    pub fn snapshot(&mut self, world: &World) -> Value {
        let saved = self.save(world);
        let id = format!("s{}", self.next_snapshot);
        self.next_snapshot += 1;
        let info = json!({
            "id": id,
            "tick": saved.tick,
            "hash": format!("{:016x}", saved.hash),
            "bytes": saved.world.len(),
        });
        self.snapshots.push_back((id, saved));
        while self.snapshots.len() > MAX_SNAPSHOTS {
            self.snapshots.pop_front();
        }
        info
    }

    /// `restore`: a snapshot this process took; `false` for an unknown id
    /// (or one dropped beyond [`MAX_SNAPSHOTS`]) or a world it no longer fits.
    pub fn restore(&mut self, world: &mut World, id: &str) -> bool {
        let Some(saved) = self
            .snapshots
            .iter()
            .find(|(sid, _)| sid == id)
            .map(|(_, s)| s.clone())
        else {
            return false;
        };
        self.load(world, &saved).is_ok()
    }

    /// `dump`: a world, entity or UI dump; `None` for an unknown entity.
    pub fn dump(&self, world: &World, kind: DumpKind, id: &str) -> Option<Value> {
        match kind {
            DumpKind::World => Some(self.dump_world(world)),
            DumpKind::Ui => Some(self.dump_ui(world)),
            DumpKind::Entity => {
                let e = self.names.resolve(world, id)?;
                Some(self.dump_entity(world, e))
            }
        }
    }

    /// `commands`: schedule a script; `None` for a malformed one (the parse
    /// error is in the log).
    pub fn commands(&mut self, world: &World, script: &str) -> Option<Value> {
        match self.schedule(world, script) {
            Ok(report) => Some(report.to_json()),
            Err(err) => {
                self.note(world.time().tick, true, format!("commands: {err}"));
                None
            }
        }
    }

    // ---- dumps ------------------------------------------------------------

    /// The world dump: archetype tables (non-empty, in creation order) with
    /// their component sets and entity counts, systems in run order, resources.
    pub fn dump_world(&self, world: &World) -> Value {
        let archetypes: Vec<Value> = world
            .query(&QueryDesc::default())
            .map(|q| {
                q.map(|view| {
                    let components: Vec<String> = view
                        .types()
                        .iter()
                        .map(|id| schema_name(world, *id))
                        .collect();
                    json!({"components": components, "count": view.len()})
                })
                .collect()
            })
            .unwrap_or_default();
        // og-ecs reports every system: its declared access and what its
        // queries matched when it last ran. The probe reads no clock: ms is null.
        let systems: Vec<Value> = world
            .system_info()
            .into_iter()
            .map(|s| json!({"name": s.name, "matched": s.matched, "reads": s.reads, "writes": s.writes, "ms": Value::Null}))
            .collect();
        let resources: Vec<String> = world
            .schemas()
            .into_iter()
            .filter(|s| {
                world
                    .id_by_name(&s.name)
                    .is_ok_and(|id| world.resource_bytes(id).is_ok())
            })
            .map(|s| s.name)
            .collect();
        json!({
            "kind": "world",
            "tick": world.time().tick,
            "hash": format!("{:016x}", world.world_hash()),
            "entities": world.entity_count(),
            "archetypes": archetypes,
            "systems": systems,
            "resources": resources,
        })
    }

    /// One entity: every component with every field, by component name
    /// (its transform is `components["og.Transform"]`).
    pub fn dump_entity(&self, world: &World, e: Entity) -> Value {
        let components: Map<String, Value> = Self::components(world, e)
            .into_iter()
            .map(|(schema, bytes)| {
                let fields = value::component_json(&schema, &bytes);
                (schema.name, Value::Object(fields))
            })
            .collect();
        json!({
            "kind": "entity",
            "id": self.names.id_of(world, e),
            "name": self.names.name_of(world, e),
            "components": components,
        })
    }

    /// The UI tree in draw order; empty without a UI module.
    pub fn dump_ui(&self, world: &World) -> Value {
        let elements: Vec<Value> = self
            .ui
            .as_ref()
            .map(|ui| ui.elements(world).iter().map(|e| e.to_json()).collect())
            .unwrap_or_default();
        json!({"kind": "ui", "tick": world.time().tick, "elements": elements})
    }
}
