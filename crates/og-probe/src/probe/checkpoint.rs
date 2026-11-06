//! Run-local probe state, separate from the player's save/restore contract.
//! A host must also checkpoint its own state and establish that game callbacks
//! have no mutable state outside the world before it can fork a whole run.

use super::*;

/// An in-process checkpoint of one probe at a settled tick boundary.
/// It is not portable to another probe, build, or JavaScript heap.
#[derive(Clone)]
pub struct RunCheckpoint {
    owner: Rc<RefCell<Assertions>>,
    world: Vec<u8>,
    input: Value,
    render: u64,
    commands: BTreeMap<String, Registered>,
    queue: BTreeMap<(u64, u64), Queued>,
    seq: u64,
    inbox: Vec<(String, Vec<String>)>,
    snapshots: VecDeque<(String, Saved)>,
    next_snapshot: u64,
    paused: bool,
    script_paused: bool,
    quit: bool,
    pointer: (f32, f32),
    target_missed: bool,
    replay_until: u64,
    tuned: BTreeSet<String>,
    gets: (u64, u32),
    written: BTreeMap<String, (f64, String)>,
    outputs: Vec<Output>,
    log: Vec<LogLine>,
    expectations: Vec<ExpectResult>,
    assertions: Assertions,
    invariants: Vec<Vec<String>>,
    events: BTreeMap<String, Watch>,
    event_base: BTreeMap<String, u64>,
}

impl Probe {
    /// Checkpoint a settled command boundary. In-flight captures and sweeps
    /// must finish first; a custom input sink must support checkpoints.
    pub fn checkpoint(&mut self, world: &World) -> Option<RunCheckpoint> {
        if self.shot.is_some() || self.sweeping.is_some() || self.quiet {
            return None;
        }
        Some(RunCheckpoint {
            owner: self.assertions.clone(),
            world: world.snapshot(),
            input: self.input.checkpoint()?,
            render: self.capture.as_mut().map_or(0, |c| c.save_state()),
            commands: self.commands.clone(),
            queue: self.queue.clone(),
            seq: self.seq,
            inbox: self.inbox.checkpoint(),
            snapshots: self.snapshots.clone(),
            next_snapshot: self.next_snapshot,
            paused: self.paused,
            script_paused: self.script_paused,
            quit: self.quit,
            pointer: self.pointer,
            target_missed: self.target_missed,
            replay_until: self.replay_until,
            tuned: self.tuned.clone(),
            gets: self.gets,
            written: self.written.clone(),
            outputs: self.outputs.clone(),
            log: self.log.clone(),
            expectations: self.expectations.clone(),
            assertions: self.assertions.borrow().clone(),
            invariants: self.invariants.clone(),
            events: self.events.clone(),
            event_base: self.event_base.clone(),
        })
    }

    /// Restore this probe's command timeline as well as its world. Unlike
    /// `restore`, this rewinds quit, pending commands, inputs and reports.
    /// Prefix assertions return with the prefix; sibling coverage does not.
    pub fn restore_checkpoint(
        &mut self,
        world: &mut World,
        saved: &RunCheckpoint,
    ) -> og_core::Result<()> {
        if !Rc::ptr_eq(&self.assertions, &saved.owner) {
            return Err(Error::new(
                ErrorKind::Invalid,
                "checkpoint belongs to another probe",
            ));
        }
        let input = self.input.checkpoint().ok_or_else(|| {
            Error::new(ErrorKind::Invalid, "input sink cannot restore checkpoints")
        })?;
        if !self.input.restore_checkpoint(&saved.input) {
            return Err(Error::new(
                ErrorKind::Invalid,
                "input sink refused checkpoint",
            ));
        }
        if let Err(error) = world.restore(&saved.world) {
            self.input.restore_checkpoint(&input);
            return Err(error);
        }
        if let Some(capture) = self.capture.as_mut() {
            capture.restore_state(saved.render);
        }
        self.commands.clone_from(&saved.commands);
        self.queue.clone_from(&saved.queue);
        self.seq = saved.seq;
        self.inbox.restore_checkpoint(saved.inbox.clone());
        self.snapshots.clone_from(&saved.snapshots);
        self.next_snapshot = saved.next_snapshot;
        self.paused = saved.paused;
        self.script_paused = saved.script_paused;
        self.quit = saved.quit;
        self.pointer = saved.pointer;
        self.target_missed = saved.target_missed;
        self.replay_until = saved.replay_until;
        self.tuned.clone_from(&saved.tuned);
        self.gets = saved.gets;
        self.written.clone_from(&saved.written);
        self.outputs.clone_from(&saved.outputs);
        self.log.clone_from(&saved.log);
        self.expectations.clone_from(&saved.expectations);
        self.assertions.borrow_mut().clone_from(&saved.assertions);
        self.invariants.clone_from(&saved.invariants);
        self.events.clone_from(&saved.events);
        self.event_base.clone_from(&saved.event_base);
        self.shot = None;
        self.sweeping = None;
        self.stamp = 0.0;
        self.systems_ms.clear();
        self.passes_ms.clear();
        self.gpu_passes_ms.clear();
        self.step_ms = 0.0;
        self.render_ms = 0.0;
        self.triangles = 0;
        Ok(())
    }
}
