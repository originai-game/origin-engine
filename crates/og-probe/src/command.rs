//! The command channel's grammar, registry entries, reports and outputs
//! (design §3.3). Execution lives on [`crate::Probe`].
//!
//! ```text
//! script := step (';' step)*            whitespace around steps is ignored, empty steps too
//! step   := time ':' action (':' arg)*
//! time   := seconds of simulation time, a finite decimal ≥ 0, never below the previous step's
//! action := [a-z_][a-z0-9_]*
//! arg    := any text; '\' escapes the next character, so '\:' and '\;' are literal
//! ```

use og_ecs::World;
use serde_json::{Value, json};
use std::fmt;
use std::rc::Rc;

/// One parsed step of a command script.
#[derive(Clone, Debug, PartialEq)]
pub struct CommandStep {
    /// When it runs, in simulation seconds from the moment it is scheduled.
    pub at: f64,
    /// The action name.
    pub action: String,
    /// The `:`-separated arguments after the action name.
    pub args: Vec<String>,
}

impl CommandStep {
    pub fn to_json(&self) -> Value {
        json!({"at": self.at, "action": self.action, "args": self.args})
    }
}

/// The first malformed step of a script.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    /// The step's position, from 1.
    pub step: usize,
    /// The step as written.
    pub text: String,
    /// What is wrong and how to write it instead.
    pub reason: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "step {} ('{}'): {}", self.step, self.text, self.reason)
    }
}

impl std::error::Error for ParseError {}

fn is_action(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_lowercase() || c == '_')
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Parse a script without scheduling it.
pub fn parse(script: &str) -> Result<Vec<CommandStep>, ParseError> {
    // Split into steps and fields in one pass, honouring escapes.
    let mut raw: Vec<(String, Vec<String>)> = Vec::new();
    let mut text = String::new();
    let mut fields = vec![String::new()];
    let mut chars = script.chars();
    let flush =
        |text: &mut String, fields: &mut Vec<String>, raw: &mut Vec<(String, Vec<String>)>| {
            if !text.trim().is_empty() {
                raw.push((
                    text.trim().to_owned(),
                    std::mem::replace(fields, vec![String::new()]),
                ));
            } else {
                *fields = vec![String::new()];
            }
            text.clear();
        };
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                text.push(c);
                let literal = chars.next().unwrap_or('\\');
                text.push(literal);
                fields.last_mut().expect("a field").push(literal);
            }
            ';' => flush(&mut text, &mut fields, &mut raw),
            ':' => {
                text.push(c);
                fields.push(String::new());
            }
            _ => {
                text.push(c);
                fields.last_mut().expect("a field").push(c);
            }
        }
    }
    flush(&mut text, &mut fields, &mut raw);

    let mut steps = Vec::with_capacity(raw.len());
    let mut previous = 0.0f64;
    for (index, (text, fields)) in raw.into_iter().enumerate() {
        let fail = |reason: String| ParseError {
            step: index + 1,
            text: text.clone(),
            reason,
        };
        let mut fields = fields.into_iter();
        let time = fields.next().unwrap_or_default();
        let time = time.trim();
        let at: f64 = time.parse().map_err(|_| {
            fail(format!(
                "'{time}' is not a time; start each step with seconds, as `1.5:screenshot`"
            ))
        })?;
        if !at.is_finite() || at < 0.0 {
            return Err(fail(format!(
                "time {time} must be a finite number of seconds ≥ 0"
            )));
        }
        if at < previous {
            return Err(fail(format!(
                "time {at} is before the previous step's {previous}; times ascend"
            )));
        }
        previous = at;
        let action = fields
            .next()
            .ok_or_else(|| fail("no action after the time; write `t:action[:args]`".into()))?;
        let action = action.trim().to_owned();
        if !is_action(&action) {
            return Err(fail(format!(
                "'{action}' is not an action name (snake_case, such as `dump_world`)"
            )));
        }
        let mut args: Vec<String> = fields.collect();
        if let Some(last) = args.last_mut() {
            let trimmed = last.trim_end().len();
            last.truncate(trimmed);
        }
        steps.push(CommandStep { at, action, args });
    }
    Ok(steps)
}

/// How a registered action describes itself in `help` and `og run --help`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommandDesc {
    /// The argument form, such as `<count>`.
    pub usage: String,
    /// One sentence on what it does.
    pub doc: String,
}

/// A registered action: it runs at the start of the tick it was scheduled
/// for with the world and its arguments. `Err` is a warning in the log; the
/// script continues.
pub type CommandHandler = Rc<dyn Fn(&mut World, &[String]) -> Result<(), String>>;

/// A registered dump: it answers JSON for the world as of its tick, which
/// the probe writes as an output like `dump_entity`'s (see
/// [`crate::Probe::register_dump`]).
pub type DumpHandler = Rc<dyn Fn(&World, &[String]) -> Result<serde_json::Value, String>>;

/// Actions an engine module asks for from inside a tick (an AI's game
/// command), run at the start of the next tick before its scheduled steps.
/// Cloning shares it.
#[derive(Clone, Debug, Default)]
pub struct Inbox(std::sync::Arc<std::sync::Mutex<Vec<Queued>>>);

/// An action and its arguments, waiting in an [`Inbox`].
type Queued = (String, Vec<String>);

impl Inbox {
    /// Queues `action` with `args` for the next tick's start.
    pub fn push(&self, action: impl Into<String>, args: Vec<String>) {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push((action.into(), args));
    }

    pub(crate) fn take(&self) -> Vec<Queued> {
        std::mem::take(&mut *self.0.lock().unwrap_or_else(|e| e.into_inner()))
    }

    /// Puts back, ahead of anything queued since, actions taken and not run.
    pub(crate) fn requeue(&self, actions: Vec<Queued>) {
        if !actions.is_empty() {
            let mut queued = self.0.lock().unwrap_or_else(|e| e.into_inner());
            queued.splice(0..0, actions);
        }
    }
}

/// One action as `help` lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandInfo {
    pub name: String,
    /// `screenshot[:path][:full|:N]`.
    pub usage: String,
    pub doc: String,
    /// Shipped with the engine (true) or registered by a module or game.
    pub core: bool,
}

impl CommandInfo {
    pub fn to_json(&self) -> Value {
        json!({"name": self.name, "usage": self.usage, "doc": self.doc, "core": self.core})
    }
}

/// The actions the engine ships: name, usage, doc (design §3.3, IDL
/// `commands.CoreAction`, plus `help`).
pub const CORE_ACTIONS: &[(&str, &str, &str)] = &[
    (
        "screenshot",
        "screenshot[:path][:full|:N]",
        "Capture the frame at this tick: the render target's size, `full` the same, `N` a long edge of N pixels.",
    ),
    (
        "touch_move_to",
        "touch_move_to:<ui id>",
        "Move the pointer to the centre of a UI element by its id, or a `Parent/Child` path of ids.",
    ),
    (
        "touch_move",
        "touch_move:x,y",
        "Move the pointer to CSS pixel coordinates.",
    ),
    (
        "touch_start",
        "touch_start",
        "Press the pointer (left button) where it is.",
    ),
    ("touch_end", "touch_end", "Release the pointer where it is."),
    (
        "scroll",
        "scroll:dx,dy",
        "Scroll at the pointer by CSS pixels.",
    ),
    (
        "touch",
        "touch:<id>:<down|move|up>:<x,y | ui id>",
        "A further touch (pointer 1 to 4, og.Pointers) while the pointer is held elsewhere: press, move or lift it at CSS pixel coordinates or a UI element's centre.",
    ),
    (
        "gamepad",
        "gamepad:<a+b+...|->[@pad][:lx,ly[,rx,ry]][:lt,rt]",
        "Set a pad's whole state (og.Gamepad; pad 0 unless `@1` to `@3` follows the buttons): the standard buttons held, joined by `+` (`-` none), the stick axes (y down) and the triggers; `gamepad:off` disconnects it (`gamepad:off@2` pad 2).",
    ),
    (
        "key_down",
        "key_down:[Mod+]<Key>",
        "Hold a key by its KeyboardEvent.code, such as `KeyW` or `Space`, with the modifiers named before it held too (`Shift+ArrowLeft`, `Meta+ArrowLeft` for Command, `Ctrl+Shift+KeyZ`; Shift, Ctrl, Alt or Option, Meta or Cmd).",
    ),
    (
        "key_up",
        "key_up:[Mod+]<Key>",
        "Release a key, then the modifiers named before it.",
    ),
    (
        "type",
        "type:<text>",
        "Type text, two ticks per character: its key press (Shift for capitals and symbols) and the character as typed text (og.TextInput); a character no key types (é, 中) is typed text alone.",
    ),
    (
        "compose",
        "compose:<text>",
        "An input method's composition so far (og.TextInput with composing): a focused text field shows it underlined at the caret; `compose:` with nothing ends it without a commit.",
    ),
    (
        "commit",
        "commit:<text>",
        "An input method commits text (og.TextInput): a focused text field inserts it at the caret and the composition ends.",
    ),
    (
        "move_to_text",
        "move_to_text:[=]text[@N]",
        "Move the pointer to the UI element showing the text: a substring, exactly with `=`, the Nth match (0-based, negative from the end) with `@N`; topmost first.",
    ),
    (
        "teleport",
        "teleport:<entity>:x,y[,z]",
        "Set an entity's Transform.position.",
    ),
    (
        "frame_entity",
        "frame_entity:<entity>",
        "Turn the Camera resource to look at an entity's Transform.position.",
    ),
    (
        "dump_world",
        "dump_world[:path]",
        "Write the world dump: archetype tables, systems and their matched counts, resources.",
    ),
    (
        "dump_entity",
        "dump_entity:<entity>[:path]",
        "Write one entity's dump: its components with every field, and its transform.",
    ),
    (
        "dump_ui",
        "dump_ui[:path]",
        "Write the UI tree with bounds.",
    ),
    (
        "set",
        "set:<entity>.<component>.<field>=<value> | set:<Resource>.<field>=<value>",
        "Write one field of a component or a resource through the schema; vectors as `x,y,z`; range-checked, never clamped.",
    ),
    (
        "get",
        "get:<Resource>[.<field>] | get:<entity>[.<component>[.<field>]][:path]",
        "Read a resource, an entity, a component or one field: logged and written as JSON (`get-<tick>.json`).",
    ),
    ("seed", "seed:<n>", "Reseed the world's Rng resource."),
    ("pause", "pause", "Hold the clock; advance still steps."),
    ("resume", "resume", "Release the clock."),
    (
        "advance",
        "advance:<ticks>",
        "Take fixed steps now (at most 3600), running the steps scheduled on the way.",
    ),
    (
        "listen",
        "listen[:path][:seconds]",
        "Write the last seconds (default 1, at most 2) of audio the host's mixer rendered as a 16-bit stereo WAV.",
    ),
    ("quit", "quit", "End the run."),
    (
        "help",
        "help",
        "List every action the channel knows, core and registered.",
    ),
];

/// What scheduling a script did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandsReport {
    /// Steps scheduled.
    pub scheduled: u32,
    /// The tick of the last step.
    pub until_tick: u64,
    /// Unknown actions and other notes, each naming the fix.
    pub warnings: Vec<String>,
}

impl CommandsReport {
    pub fn to_json(&self) -> Value {
        json!({"scheduled": self.scheduled, "untilTick": self.until_tick, "warnings": self.warnings})
    }
}

/// What an output carries.
#[derive(Clone, Debug, PartialEq)]
pub enum Payload {
    /// A captured frame.
    Png {
        width: u32,
        height: u32,
        bytes: Vec<u8>,
    },
    /// A dump.
    Json(Value),
    /// A listing, such as `help`.
    Text(String),
    /// Audio: a WAV file (`listen`).
    Wav(Vec<u8>),
}

/// A file a command produced (`screenshot`, `dump_*`, `help`). The probe
/// never writes files: the host takes outputs and writes or posts them.
#[derive(Clone, Debug, PartialEq)]
pub struct Output {
    /// The tick it shows.
    pub tick: u64,
    /// The action that produced it.
    pub action: String,
    /// The path the script asked for, if any.
    pub path: Option<String>,
    pub payload: Payload,
}

impl Output {
    /// The path to write: the script's, or `<action>-<tick>.<ext>` with the
    /// tick zero-padded to six digits.
    pub fn file_name(&self) -> String {
        if let Some(path) = &self.path {
            return path.clone();
        }
        let ext = match self.payload {
            Payload::Png { .. } => "png",
            Payload::Json(_) => "json",
            Payload::Text(_) => "txt",
            Payload::Wav(_) => "wav",
        };
        format!("{}-{:06}.{ext}", self.action, self.tick)
    }

    /// The bytes to write: the PNG, pretty JSON with a final newline, or the text.
    pub fn bytes(&self) -> Vec<u8> {
        match &self.payload {
            Payload::Png { bytes, .. } => bytes.clone(),
            Payload::Json(value) => {
                let mut text = serde_json::to_string_pretty(value).expect("JSON serialises");
                text.push('\n');
                text.into_bytes()
            }
            Payload::Text(text) => text.clone().into_bytes(),
            Payload::Wav(bytes) => bytes.clone(),
        }
    }
}

/// One log line: warnings from the channel and the probe, each naming the fix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogLine {
    pub tick: u64,
    pub warning: bool,
    pub text: String,
}

impl fmt::Display for LogLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level = if self.warning { "WARN" } else { "INFO" };
        write!(f, "{level} tick {}: {}", self.tick, self.text)
    }
}
