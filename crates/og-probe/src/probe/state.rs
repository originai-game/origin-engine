//! [`Probe`]'s states: its fields grouped by the subject that owns them.

use super::*;

/// What the host gave the probe: its input sink, renderer, UI and files,
/// and the hooks its members answer through.
pub(crate) struct Host {
    pub(super) engine: String,
    pub(super) input: Box<dyn InputSink>,
    pub(crate) capture: Option<Box<dyn Capture>>,
    pub(crate) ui: Option<Box<dyn UiTree>>,
    pub(super) reloader: Option<Box<dyn Reloader>>,
    pub(crate) facts: Option<FactsFn>,
    /// The frame as data and picking ([`Probe::set_scene_view`]).
    pub(crate) scene_view: Option<Box<dyn SceneView>>,
    /// The boot scene's write-back ([`Probe::set_scene_writer`]).
    pub(crate) scene_writer: Option<Box<dyn SceneWriter>>,
    /// Reference files for `expect:frame` ([`Probe::set_reader`]).
    pub(crate) reader: Option<ReaderFn>,
    pub(super) listen: Option<ListenFn>,
    pub(super) memory: Option<MemoryFn>,
    pub(super) derive: Option<DeriveFn>,
    pub(super) register: Option<RegisterFn>,
}

/// Whether the next tick runs: the holds and the host's hooks around a tick.
pub(crate) struct Clock {
    pub(super) paused: bool,
    /// The clock was held by a script's `pause`: the script's later steps
    /// run in order on the held world instead of waiting for ticks that
    /// will not come.
    pub(super) script_paused: bool,
    pub(super) quit: bool,
    /// The host's end-of-tick work ([`Probe::set_after_tick`]).
    pub(super) after_tick: Option<AfterTickFn>,
    /// The host's start-of-tick work ([`Probe::set_before_tick`]).
    pub(super) before_tick: Option<AfterTickFn>,
    /// The host's tick gate ([`Probe::set_gate`]).
    pub(super) gate: Option<GateFn>,
    /// After a rollback's rewind (the gate moved `Time` back): the tick the
    /// world had reached. The ticks below it run again: they take no share
    /// of a frame's ticks, and the input's edges, which belong to the
    /// present tick, stay for it.
    pub(super) replay_until: u64,
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

/// The host's last measurements, for `state().timings`.
pub(crate) struct Stats {
    /// The last tick's milliseconds per system and per render pass.
    pub(super) systems_ms: Vec<(String, f64)>,
    pub(super) passes_ms: Vec<(String, f64)>,
    pub(super) gpu_passes_ms: Vec<(String, f64)>,
    pub(super) step_ms: f64,
    pub(super) render_ms: f64,
    pub(super) triangles: u64,
}

/// Worlds kept to go back to: the snapshots and a sweep in flight.
pub(crate) struct Saves {
    pub(super) snapshots: VecDeque<(String, Saved)>,
    pub(super) next_snapshot: u64,
    pub(super) sweeping: Option<SweepJob>,
}

impl Clock {
    /// Whether the gate holds the next tick. A gate that moved `Time`
    /// restored a peer's snapshot (lockstep's start or a late join): the
    /// world now runs on the room's shared clock, and the scheduled steps
    /// stay on their ticks of it, so every peer of a room runs the same
    /// script step (a `quit`, an `expect`) on the same tick. A step whose
    /// tick the jump passed runs at the next tick, in order.
    pub(super) fn held(&mut self, world: &mut World) -> bool {
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
    pub(super) fn replaying(&self, world: &World) -> bool {
        world.time().tick < self.replay_until
    }
}

impl Schedule {
    /// Polls a screenshot whose frame is in flight and writes it when it
    /// lands; true while it still waits.
    pub(super) fn shot_waits(
        &mut self,
        world: &World,
        host: &mut state::Host,
        out: &mut state::Out,
    ) -> bool {
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

impl Commands {
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

    pub(super) fn knows(&self, action: &str) -> bool {
        CORE_ACTIONS.iter().any(|(name, _, _)| *name == action)
            || self.registered.contains_key(action)
    }
}

impl Out {
    /// Hands an output a step made to the host. A file an earlier step of
    /// this script already wrote is not overwritten: the second is written
    /// beside it as `<stem>-2.<ext>` (then `-3`, ...), with a warning naming
    /// both steps, so a check reading the first finds what it expects and
    /// the clash is in the run's warnings instead of far from its cause.
    pub(super) fn emit(&mut self, at: f64, mut output: Output) {
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

    pub(crate) fn note(&mut self, tick: u64, warning: bool, text: String) {
        self.log.push(LogLine {
            tick,
            warning,
            text,
        });
    }
}

impl Inject {
    pub(super) fn deliver(
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

    pub(super) fn pointer_event(
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
