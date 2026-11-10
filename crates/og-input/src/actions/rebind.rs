use super::*;

/// What the player's rebinding does with a control another action holds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ConflictPolicy {
    /// Change nothing and answer the conflicts.
    #[default]
    Refuse,
    /// The other action takes what this slot had.
    Swap,
    /// The other action loses the control.
    Replace,
    /// Bind it anyway: both read it.
    Allow,
}

impl ConflictPolicy {
    pub const NAMES: [&'static str; 4] = ["refuse", "swap", "replace", "allow"];
    const ALL: [ConflictPolicy; 4] = [
        ConflictPolicy::Refuse,
        ConflictPolicy::Swap,
        ConflictPolicy::Replace,
        ConflictPolicy::Allow,
    ];

    pub fn from_name(n: &str) -> Option<ConflictPolicy> {
        Self::NAMES
            .iter()
            .position(|x| *x == n)
            .map(|i| Self::ALL[i])
    }
}

/// Another binding a rebinding would share a control with (IDL
/// `input.BindingConflict`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conflict {
    pub action: String,
    pub slot: u32,
    /// The shared control, as the new binding names it.
    pub control: String,
    pub context: String,
}

impl Conflict {
    pub fn to_json(&self) -> Value {
        json!({ "action": self.action, "slot": self.slot, "control": self.control, "actionMap": self.context })
    }
}

/// A rebinding (IDL `input.RebindDesc`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RebindDesc {
    pub action: String,
    /// Which binding: 0 the first; one past the last adds one.
    pub slot: u32,
    /// A composite's part to change; empty sets the whole binding.
    pub part: String,
    /// The new binding; for a part, its `control` is the part's.
    pub binding: Binding,
    pub policy: ConflictPolicy,
}

impl RebindDesc {
    pub fn from_json(v: &Value) -> Result<RebindDesc, String> {
        Ok(RebindDesc {
            action: str_of(v, "action")?,
            slot: f32_of(v, "slot", 0.0)?.max(0.0) as u32,
            part: str_of(v, "part")?,
            binding: match v.get("binding") {
                None | Some(Value::Null) => Binding::default(),
                Some(b) => Binding::from_json(b)?,
            },
            policy: match v.get("policy") {
                None | Some(Value::Null) => ConflictPolicy::Refuse,
                Some(p) => p
                    .as_str()
                    .and_then(ConflictPolicy::from_name)
                    .ok_or_else(|| format!("policy is refuse, swap, replace or allow, not {p}"))?,
            },
        })
    }
}

/// What a rebinding did (IDL `input.RebindAnswer`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RebindAnswer {
    pub applied: bool,
    pub conflicts: Vec<Conflict>,
}

impl RebindAnswer {
    pub fn to_json(&self) -> Value {
        json!({ "applied": self.applied, "conflicts": self.conflicts.iter().map(Conflict::to_json).collect::<Vec<_>>() })
    }
}

/// Which devices a capture takes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CaptureAccept {
    /// Keyboard and mouse, and pads.
    #[default]
    Any,
    KeyboardMouse,
    Gamepad,
}

impl CaptureAccept {
    pub const NAMES: [&'static str; 3] = ["any", "keyboardMouse", "gamepad"];
    const ALL: [CaptureAccept; 3] = [
        CaptureAccept::Any,
        CaptureAccept::KeyboardMouse,
        CaptureAccept::Gamepad,
    ];

    fn from_name(n: &str) -> Option<CaptureAccept> {
        Self::NAMES
            .iter()
            .position(|x| *x == n)
            .map(|i| Self::ALL[i])
    }

    fn name(self) -> &'static str {
        Self::NAMES[self as usize]
    }
}

/// A capture of the player's next input for a binding (IDL
/// `input.CaptureDesc`): a controls screen's "press a key".
#[derive(Clone, Debug, PartialEq)]
pub struct CaptureDesc {
    pub action: String,
    pub slot: u32,
    /// A composite's part to capture (`up` of WASD); empty: the whole binding.
    pub part: String,
    pub accept: CaptureAccept,
    /// Controls that cancel it (empty: Escape and the pad's back).
    pub cancel: Vec<String>,
    /// It cancels itself after this long (0: never), in simulated time.
    pub timeout_ms: f32,
}

impl CaptureDesc {
    pub fn from_json(v: &Value) -> Result<CaptureDesc, String> {
        let cancel = match v.get("cancel") {
            None | Some(Value::Null) => Vec::new(),
            Some(Value::Array(a)) => a
                .iter()
                .map(|c| {
                    c.as_str()
                        .map(str::to_string)
                        .ok_or("cancel lists control names")
                })
                .collect::<Result<_, _>>()?,
            Some(_) => return Err("cancel is a list of control names".into()),
        };
        Ok(CaptureDesc {
            action: str_of(v, "action")?,
            slot: f32_of(v, "slot", 0.0)?.max(0.0) as u32,
            part: str_of(v, "part")?,
            accept: match v.get("accept") {
                None | Some(Value::Null) => CaptureAccept::Any,
                Some(a) => a
                    .as_str()
                    .and_then(CaptureAccept::from_name)
                    .ok_or_else(|| format!("accept is any, keyboardMouse or gamepad, not {a}"))?,
            },
            cancel,
            timeout_ms: f32_of(v, "timeoutMs", 0.0)?.max(0.0),
        })
    }

    pub(super) fn to_json(&self) -> Value {
        json!({ "action": self.action, "slot": self.slot, "part": self.part, "accept": self.accept.name(),
            "cancel": self.cancel, "timeoutMs": self.timeout_ms })
    }
}

/// Where a capture is (IDL `input.CaptureState`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CaptureState {
    /// It waits for input.
    pub active: bool,
    /// It took a control: `binding` holds the slot with it, until applied
    /// ([`ActionMap::apply_capture`]) or another capture starts.
    pub done: bool,
    /// The player cancelled it, or it timed out.
    pub cancelled: bool,
    pub action: String,
    pub slot: u32,
    pub part: String,
    /// The control it took.
    pub control: String,
    /// The slot's binding with that control in it.
    pub binding: Binding,
    /// What applying it would conflict with.
    pub conflicts: Vec<Conflict>,
}

impl CaptureState {
    pub fn to_json(&self) -> Value {
        json!({
            "active": self.active, "done": self.done, "cancelled": self.cancelled,
            "action": self.action, "slot": self.slot, "part": self.part, "control": self.control,
            "binding": self.binding.to_json(),
            "conflicts": self.conflicts.iter().map(Conflict::to_json).collect::<Vec<_>>(),
        })
    }

    pub(super) fn from_json(v: &Value) -> Result<CaptureState, String> {
        Ok(CaptureState {
            active: bool_of(v, "active", false)?,
            done: bool_of(v, "done", false)?,
            cancelled: bool_of(v, "cancelled", false)?,
            action: str_of(v, "action")?,
            slot: f32_of(v, "slot", 0.0)? as u32,
            part: str_of(v, "part")?,
            control: str_of(v, "control")?,
            binding: Binding::from_json(v.get("binding").unwrap_or(&json!({})))?,
            conflicts: v["conflicts"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|c| {
                    Ok(Conflict {
                        action: str_of(c, "action")?,
                        slot: f32_of(c, "slot", 0.0)? as u32,
                        control: str_of(c, "control")?,
                        context: str_of(c, "actionMap")?,
                    })
                })
                .collect::<Result<_, String>>()?,
        })
    }
}
