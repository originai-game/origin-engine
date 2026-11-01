//! Gameplay actions (IDL `input.ActionDesc`): named buttons and one- and
//! two-axis values bound to controls ([`crate::Control`]: keys, the mouse,
//! the wheel, pad buttons, sticks and triggers, touch), in contexts, with
//! dead zones, sensitivity, holds and toggles; the player's rebinding with
//! conflict detection, a key-capture flow for a controls screen, and
//! bindings saved and loaded as text; the glyph of an action's binding on
//! the device used last; rumble requests for the host.
//!
//! An [`ActionMap`] is sampled once per tick, before the game's systems,
//! from the tick's `og.Input`, `og.Pointers` and `og.Gamepad`
//! ([`ActionMap::sample`]). Those are world state, and the map with its
//! per-action state is a state block ([`ActionsBlock`]), so snapshots carry
//! it, the world hash covers it, and a replay reads the same actions. Like
//! og.Input it is each peer's own: lockstep leaves it out of the hash it
//! compares, and a game hands its actions to og.net.submit.
//!
//! Edges follow og.Input's: `pressed` and `released` last one tick, and a
//! tap shorter than a tick sets both. An analog control presses a button
//! action at its threshold and releases it under three quarters of it.

use crate::control::{
    Axis, Axis2, Control, Half, PadDeadzone, ReadOptions, Reader, Reading, Shape, Surface, Zone,
    deadzone, deadzone2,
};
use crate::event::{GAMEPAD_BUTTONS, MAX_GAMEPADS};
use crate::glyph::{Device, Glyph};
use crate::resource::{Gamepad, GamepadAxis, Input, KeySet, Pointers};
use og_core::glam::Vec2;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

/// The state block an [`ActionMap`] is saved in.
pub const ACTIONS_BLOCK: &str = "og.input.actions";
/// The engine system that samples it, first in the tick.
pub const ACTIONS_SYSTEM: &str = "og.input.actions";
/// The context an action is in when its desc names none.
pub const DEFAULT_CONTEXT: &str = "gameplay";
/// The version of the text [`ActionMap::save_bindings`] writes.
pub const BINDINGS_VERSION: u64 = 1;

const NO_CONTROL: &str = "write a key code (KeyW, Space), mouse.left, mouse.xy, wheel.y, pad.a, pad.leftX, pad.leftXY, pad.dpad, touch or touch.xy; a one-axis control takes + or - for its half (pad.leftX+)";

/// What an action gives.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ActionKind {
    /// On or off.
    #[default]
    Button,
    /// One value, -1..1.
    Axis,
    /// Two values, x right and y up.
    Axis2,
}

impl ActionKind {
    pub const NAMES: [&'static str; 3] = ["button", "axis", "axis2"];
    const ALL: [ActionKind; 3] = [ActionKind::Button, ActionKind::Axis, ActionKind::Axis2];

    pub fn name(self) -> &'static str {
        Self::NAMES[self as usize]
    }

    pub fn from_name(n: &str) -> Option<ActionKind> {
        Self::NAMES
            .iter()
            .position(|x| *x == n)
            .map(|i| Self::ALL[i])
    }

    /// The composite parts a binding of this kind may have.
    pub fn parts(self) -> &'static [&'static str] {
        match self {
            ActionKind::Button => &[],
            ActionKind::Axis => &["negative", "positive"],
            ActionKind::Axis2 => &["up", "down", "left", "right"],
        }
    }
}

/// One binding of an action: a control, or a composite of parts (A and D
/// as an axis, WASD as two), with its own sensitivity and dead zone.
#[derive(Clone, Debug, PartialEq)]
pub struct Binding {
    /// The control ([`Control::parse`]); empty for a composite or an
    /// unbound slot.
    pub control: String,
    /// A one-axis composite's parts.
    pub negative: String,
    pub positive: String,
    /// A two-axis composite's parts.
    pub up: String,
    pub down: String,
    pub left: String,
    pub right: String,
    /// Multiplies the value (negative inverts): the binding's sensitivity.
    pub scale: f32,
    /// A two-axis value's y is turned over.
    pub invert_y: bool,
    /// A pad axis's dead zone, over the pad's when above 0.
    pub deadzone: f32,
    /// Where touches must start (fractions of the surface; empty: all).
    pub zone: Zone,
    /// A touch stick's full deflection, CSS px (0: 64).
    pub radius: f32,
}

impl Default for Binding {
    fn default() -> Self {
        Binding {
            control: String::new(),
            negative: String::new(),
            positive: String::new(),
            up: String::new(),
            down: String::new(),
            left: String::new(),
            right: String::new(),
            scale: 1.0,
            invert_y: false,
            deadzone: 0.0,
            zone: Zone::default(),
            radius: 0.0,
        }
    }
}

fn f32_of(v: &Value, key: &str, default: f32) -> Result<f32, String> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(x) => x
            .as_f64()
            .map(|f| f as f32)
            .filter(|f| f.is_finite())
            .ok_or_else(|| format!("{key} is a finite number")),
    }
}

fn str_of(v: &Value, key: &str) -> Result<String, String> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(String::new()),
        Some(x) => x
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| format!("{key} is a string")),
    }
}

fn bool_of(v: &Value, key: &str, default: bool) -> Result<bool, String> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(x) => x.as_bool().ok_or_else(|| format!("{key} is true or false")),
    }
}

impl Binding {
    /// A binding of one control.
    pub fn control(name: &str) -> Binding {
        Binding {
            control: name.to_string(),
            ..Binding::default()
        }
    }

    /// A one-axis composite.
    pub fn axis(negative: &str, positive: &str) -> Binding {
        Binding {
            negative: negative.into(),
            positive: positive.into(),
            ..Binding::default()
        }
    }

    /// A two-axis composite (WASD: `plane("KeyW", "KeyS", "KeyA", "KeyD")`).
    pub fn plane(up: &str, down: &str, left: &str, right: &str) -> Binding {
        Binding {
            up: up.into(),
            down: down.into(),
            left: left.into(),
            right: right.into(),
            ..Binding::default()
        }
    }

    /// A composite part's control (`None` for a name that is no part).
    pub fn part(&self, part: &str) -> Option<&str> {
        Some(match part {
            "negative" => &self.negative,
            "positive" => &self.positive,
            "up" => &self.up,
            "down" => &self.down,
            "left" => &self.left,
            "right" => &self.right,
            _ => return None,
        })
    }

    fn part_mut(&mut self, part: &str) -> Option<&mut String> {
        Some(match part {
            "negative" => &mut self.negative,
            "positive" => &mut self.positive,
            "up" => &mut self.up,
            "down" => &mut self.down,
            "left" => &mut self.left,
            "right" => &mut self.right,
            _ => return None,
        })
    }

    /// Every part's name and control, in the order prompts read them
    /// (up, left, down, right: W A S D; negative, positive).
    fn parts(&self) -> [(&'static str, &str); 6] {
        [
            ("up", &self.up),
            ("left", &self.left),
            ("down", &self.down),
            ("right", &self.right),
            ("negative", &self.negative),
            ("positive", &self.positive),
        ]
    }

    /// Whether it binds nothing (an unbound slot).
    pub fn is_empty(&self) -> bool {
        self.control.is_empty() && !self.is_composite()
    }

    /// Whether it is a composite.
    pub fn is_composite(&self) -> bool {
        self.parts().iter().any(|(_, c)| !c.is_empty())
    }

    /// Its controls with the part each is (`None`: the binding's own),
    /// skipping names that name none.
    pub fn controls(&self) -> Vec<(Option<&'static str>, Control)> {
        let mut out = Vec::new();
        if let Some(c) = Control::parse(&self.control) {
            out.push((None, c));
        }
        for (part, name) in self.parts() {
            if let Some(c) = Control::parse(name) {
                out.push((Some(part), c));
            }
        }
        out
    }

    fn options(&self) -> ReadOptions {
        ReadOptions {
            deadzone: self.deadzone,
            zone: self.zone,
            radius: self.radius,
        }
    }

    /// Whether it fits an action of `kind`: the reason when it does not.
    pub fn check(&self, kind: ActionKind) -> Result<(), String> {
        if !self.scale.is_finite() {
            return Err("scale is a finite number".into());
        }
        if !(0.0..=0.95).contains(&self.deadzone) {
            return Err("deadzone is 0 to 0.95".into());
        }
        if !(self.radius >= 0.0 && self.radius.is_finite()) {
            return Err("radius is 0 or more".into());
        }
        let z = self.zone;
        if ![z.x, z.y, z.width, z.height]
            .iter()
            .all(|v| (0.0..=1.0).contains(v))
        {
            return Err(
                "a zone's x, y, width and height are fractions of the surface, 0 to 1".into(),
            );
        }
        let parse = |name: &str| -> Result<Option<Control>, String> {
            if name.is_empty() {
                return Ok(None);
            }
            Control::parse(name)
                .map(Some)
                .ok_or_else(|| format!("'{name}' names no control: {NO_CONTROL}"))
        };
        let control = parse(&self.control)?;
        let composite = self.is_composite();
        if control.is_some() && composite {
            return Err(format!(
                "a binding is a control or a composite, not both: '{}' with parts",
                self.control
            ));
        }
        for (part, name) in self.parts() {
            let Some(c) = parse(name)? else { continue };
            if !kind.parts().contains(&part) {
                return Err(match kind {
                    ActionKind::Button => format!(
                        "a button action binds single controls: write {{ control: \"{name}\" }}, not {part}"
                    ),
                    ActionKind::Axis => {
                        format!("an axis's composite has negative and positive, not {part}")
                    }
                    ActionKind::Axis2 => {
                        format!("a two-axis composite has up, down, left and right, not {part}")
                    }
                });
            }
            if c.shape() != Shape::Digital {
                return Err(format!(
                    "a composite's {part} is 0 to 1 (a key, a button, a trigger or a half axis such as {name}+), not '{name}'"
                ));
            }
        }
        if let Some(c) = control {
            match (kind, c.shape()) {
                (ActionKind::Axis2, Shape::Plane) => {}
                (ActionKind::Axis2, _) => {
                    return Err(format!(
                        "'{}' is one axis: a two-axis action binds pad.leftXY, pad.rightXY, pad.dpad, mouse.xy, touch.xy, or a composite (up, down, left, right)",
                        self.control
                    ));
                }
                (_, Shape::Plane) => {
                    return Err(format!(
                        "'{}' is two axes: bind it to an axis2 action, or take one of its axes (pad.leftX)",
                        self.control
                    ));
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn from_json(v: &Value) -> Result<Binding, String> {
        if !v.is_object() {
            return Err("a binding is an object such as { control: \"KeyW\" }".into());
        }
        let zone = match v.get("zone") {
            None | Some(Value::Null) => Zone::default(),
            Some(z) => Zone {
                x: f32_of(z, "x", 0.0)?,
                y: f32_of(z, "y", 0.0)?,
                width: f32_of(z, "width", 0.0)?,
                height: f32_of(z, "height", 0.0)?,
            },
        };
        Ok(Binding {
            control: str_of(v, "control")?,
            negative: str_of(v, "negative")?,
            positive: str_of(v, "positive")?,
            up: str_of(v, "up")?,
            down: str_of(v, "down")?,
            left: str_of(v, "left")?,
            right: str_of(v, "right")?,
            scale: f32_of(v, "scale", 1.0)?,
            invert_y: bool_of(v, "invertY", false)?,
            deadzone: f32_of(v, "deadzone", 0.0)?,
            zone,
            radius: f32_of(v, "radius", 0.0)?,
        })
    }

    /// As JSON, its defaults left out.
    pub fn to_json(&self) -> Value {
        let mut m = Map::new();
        for (key, val) in [
            ("control", &self.control),
            ("negative", &self.negative),
            ("positive", &self.positive),
            ("up", &self.up),
            ("down", &self.down),
            ("left", &self.left),
            ("right", &self.right),
        ] {
            if !val.is_empty() {
                m.insert(key.into(), json!(val));
            }
        }
        if self.scale != 1.0 {
            m.insert("scale".into(), json!(self.scale));
        }
        if self.invert_y {
            m.insert("invertY".into(), json!(true));
        }
        if self.deadzone != 0.0 {
            m.insert("deadzone".into(), json!(self.deadzone));
        }
        if self.zone != Zone::default() {
            let z = self.zone;
            m.insert(
                "zone".into(),
                json!({ "x": z.x, "y": z.y, "width": z.width, "height": z.height }),
            );
        }
        if self.radius != 0.0 {
            m.insert("radius".into(), json!(self.radius));
        }
        Value::Object(m)
    }

    /// Its reading this tick for an action of `kind` (x for one value),
    /// with `muted` controls reading nothing.
    fn read(&self, kind: ActionKind, r: &Reader, muted: &dyn Fn(Control) -> bool) -> Reading {
        let o = self.options();
        let one = |name: &str| -> Reading {
            match Control::parse(name) {
                Some(c) if !muted(c) => r.read(c, &o),
                _ => Reading::default(),
            }
        };
        if !self.control.is_empty() {
            let mut out = one(&self.control);
            match kind {
                ActionKind::Button => out.value = Vec2::new(out.value.x.abs().min(1.0), 0.0),
                ActionKind::Axis => out.value.x *= self.scale,
                ActionKind::Axis2 => {
                    if self.invert_y {
                        out.value.y = -out.value.y;
                    }
                    out.value *= self.scale;
                }
            }
            return out;
        }
        let parts: Vec<(&str, Reading)> = self
            .parts()
            .iter()
            .filter(|(_, c)| !c.is_empty())
            .map(|(p, c)| (*p, one(c)))
            .collect();
        let get = |p: &str| {
            parts
                .iter()
                .find(|(q, _)| *q == p)
                .map_or(0.0, |(_, r)| r.value.x.clamp(0.0, 1.0))
        };
        let mut out = Reading {
            pressed: parts.iter().any(|(_, r)| r.pressed),
            released: parts.iter().any(|(_, r)| r.released),
            device: parts.iter().find_map(|(_, r)| r.device),
            ..Reading::default()
        };
        match kind {
            ActionKind::Button => {}
            ActionKind::Axis => out.value.x = (get("positive") - get("negative")) * self.scale,
            ActionKind::Axis2 => {
                let mut v = Vec2::new(get("right") - get("left"), get("up") - get("down"));
                if v.length() > 1.0 {
                    v = v.normalize();
                }
                if self.invert_y {
                    v.y = -v.y;
                }
                out.value = v * self.scale;
            }
        }
        out
    }
}

fn set_field(b: &mut Binding, name: &str, to: &str) {
    let field = if name.is_empty() {
        &mut b.control
    } else {
        b.part_mut(name).expect("a part")
    };
    *field = to.to_string();
}

/// Whether two controls read the same input (a stick and one of its axes,
/// a half axis and the whole, the d-pad and its buttons).
pub fn overlaps(a: Control, b: Control) -> bool {
    let axes_of = |s: Axis2| -> &'static [Axis] {
        match s {
            Axis2::Mouse => &[Axis::MouseX, Axis::MouseY],
            Axis2::PadLeft => &[Axis::Pad(GamepadAxis::LeftX), Axis::Pad(GamepadAxis::LeftY)],
            Axis2::PadRight => &[
                Axis::Pad(GamepadAxis::RightX),
                Axis::Pad(GamepadAxis::RightY),
            ],
            Axis2::Dpad | Axis2::Touch => &[],
        }
    };
    match (a, b) {
        // Touch zones may lie apart: touches never conflict.
        (Control::Touch | Control::Axis2(Axis2::Touch), _)
        | (_, Control::Touch | Control::Axis2(Axis2::Touch)) => false,
        (Control::Axis(x, hx), Control::Axis(y, hy)) => {
            x == y && (hx == hy || hx == Half::Full || hy == Half::Full)
        }
        (Control::Axis2(s), Control::Axis(x, _)) | (Control::Axis(x, _), Control::Axis2(s)) => {
            axes_of(s).contains(&x)
        }
        (Control::Axis2(Axis2::Dpad), Control::PadButton(i))
        | (Control::PadButton(i), Control::Axis2(Axis2::Dpad)) => (12..=15).contains(&i),
        _ => a == b,
    }
}

/// An action as a game defines it (IDL `input.ActionDesc`).
#[derive(Clone, Debug, PartialEq)]
pub struct ActionDesc {
    pub name: String,
    pub kind: ActionKind,
    /// Its context ([`ActionMap::set_contexts`]); empty is
    /// [`DEFAULT_CONTEXT`].
    pub context: String,
    /// What a settings screen and prompts call it.
    pub label: String,
    /// The default bindings, in slot order.
    pub bindings: Vec<Binding>,
    /// Where an analog value presses it (0..1].
    pub threshold: f32,
    /// A dead zone on its combined value (radial for two axes).
    pub deadzone: f32,
    /// How long a press is held before `held` (0: never).
    pub hold_ms: f32,
    /// The player cannot rebind it.
    pub locked: bool,
    /// A press turns it on and the next press off (a hold's alternative,
    /// XAG 107); the player may change it ([`ActionMap::set_toggle`]).
    pub toggle: bool,
}

impl Default for ActionDesc {
    fn default() -> Self {
        ActionDesc {
            name: String::new(),
            kind: ActionKind::Button,
            context: DEFAULT_CONTEXT.into(),
            label: String::new(),
            bindings: Vec::new(),
            threshold: 0.5,
            deadzone: 0.0,
            hold_ms: 0.0,
            locked: false,
            toggle: false,
        }
    }
}

impl ActionDesc {
    pub fn from_json(v: &Value) -> Result<ActionDesc, String> {
        if !v.is_object() {
            return Err("an action is an object such as { name: \"jump\", bindings: [{ control: \"Space\" }] }".into());
        }
        let kind = match v.get("kind") {
            None | Some(Value::Null) => ActionKind::Button,
            Some(k) => k
                .as_str()
                .and_then(ActionKind::from_name)
                .ok_or_else(|| format!("kind is button, axis or axis2, not {k}"))?,
        };
        let bindings = match v.get("bindings") {
            None | Some(Value::Null) => Vec::new(),
            Some(Value::Array(a)) => a
                .iter()
                .enumerate()
                .map(|(i, b)| Binding::from_json(b).map_err(|e| format!("binding {i}: {e}")))
                .collect::<Result<_, _>>()?,
            Some(_) => return Err("bindings is a list".into()),
        };
        let context = str_of(v, "context")?;
        Ok(ActionDesc {
            name: str_of(v, "name")?,
            kind,
            context: if context.is_empty() {
                DEFAULT_CONTEXT.into()
            } else {
                context
            },
            label: str_of(v, "label")?,
            bindings,
            threshold: f32_of(v, "threshold", 0.5)?,
            deadzone: f32_of(v, "deadzone", 0.0)?,
            hold_ms: f32_of(v, "holdMs", 0.0)?,
            locked: bool_of(v, "locked", false)?,
            toggle: bool_of(v, "toggle", false)?,
        })
    }

    pub fn to_json(&self) -> Value {
        json!({
            "name": self.name, "kind": self.kind.name(), "context": self.context,
            "label": self.label, "bindings": self.bindings.iter().map(Binding::to_json).collect::<Vec<_>>(),
            "threshold": self.threshold, "deadzone": self.deadzone, "holdMs": self.hold_ms,
            "locked": self.locked, "toggle": self.toggle,
        })
    }

    /// Whether it is well formed: the reason when it is not.
    pub fn check(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("an action has a name".into());
        }
        if !(self.threshold > 0.0 && self.threshold <= 1.0) {
            return Err(format!("{}: threshold is above 0 and at most 1", self.name));
        }
        if !(0.0..=0.95).contains(&self.deadzone) {
            return Err(format!("{}: deadzone is 0 to 0.95", self.name));
        }
        if !(self.hold_ms >= 0.0 && self.hold_ms.is_finite()) {
            return Err(format!("{}: holdMs is 0 or more", self.name));
        }
        if self.toggle && self.kind != ActionKind::Button {
            return Err(format!("{}: only a button action toggles", self.name));
        }
        for (i, b) in self.bindings.iter().enumerate() {
            b.check(self.kind)
                .map_err(|e| format!("{} binding {i}: {e}", self.name))?;
        }
        Ok(())
    }
}

/// An action's state this tick (IDL `input.ActionState`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ActionState {
    /// Held: a binding past the threshold (or its toggle on).
    pub down: bool,
    /// It went down this tick.
    pub pressed: bool,
    /// It went up this tick.
    pub released: bool,
    /// Down for its hold (`holdMs`).
    pub held: bool,
    /// A button's strength (0..1: a trigger's travel), an axis's value, a
    /// two-axis value's length.
    pub value: f32,
    /// A two-axis action's value (x right, y up); (value, 0) for the others.
    pub vector: Vec2,
    /// The device whose binding drove it last.
    pub device: Device,
}

impl ActionState {
    pub fn to_json(&self) -> Value {
        json!({
            "down": self.down, "pressed": self.pressed, "released": self.released, "held": self.held,
            "value": self.value, "vector": { "x": self.vector.x, "y": self.vector.y },
            "device": self.device.name(),
        })
    }
}

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
        json!({ "action": self.action, "slot": self.slot, "control": self.control, "context": self.context })
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

    fn to_json(&self) -> Value {
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

    fn from_json(v: &Value) -> Result<CaptureState, String> {
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
                        context: str_of(c, "context")?,
                    })
                })
                .collect::<Result<_, String>>()?,
        })
    }
}

/// A rumble the game asked for (IDL `input.RumbleDesc`): output for the
/// host, never world state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rumble {
    /// The pad, 0 to 7; -1 every connected pad.
    pub pad: i32,
    /// The heavy motor, 0..1.
    pub low: f32,
    /// The light motor, 0..1.
    pub high: f32,
    /// How long, ms; 0 stops the pad's rumble.
    pub ms: f32,
}

/// What a capture's tick needs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Need {
    /// A button action or a composite's part: 0..1.
    Digital,
    /// An axis action's whole binding.
    Signed,
    /// A two-axis action's whole binding.
    Plane,
}

#[derive(Clone, Debug, PartialEq)]
struct Capture {
    desc: CaptureDesc,
    start: u64,
    /// Axes deflected when it started, which count once they return:
    /// bit `pad * 6 + axis`.
    disarmed: u64,
}

/// One action's state between ticks.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Runtime {
    state: ActionState,
    /// Its bindings were past the threshold last tick.
    raw_down: bool,
    /// The tick its press started.
    since: u64,
    toggled: bool,
    /// After a capture: reads nothing until its bindings read nothing.
    suppressed: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct Action {
    desc: ActionDesc,
    /// The player's bindings, when they differ from the desc's.
    current: Option<Vec<Binding>>,
    /// The player's toggle, when set.
    toggle: Option<bool>,
    rt: Runtime,
}

impl Action {
    fn bindings(&self) -> &[Binding] {
        self.current.as_deref().unwrap_or(&self.desc.bindings)
    }

    fn toggles(&self) -> bool {
        self.toggle.unwrap_or(self.desc.toggle)
    }
}

/// One action as a settings screen lists it (IDL `input.ActionInfo`).
#[derive(Clone, Debug, PartialEq)]
pub struct ActionInfo {
    pub desc: ActionDesc,
    /// Its bindings now (the player's, else the defaults).
    pub bindings: Vec<Binding>,
    pub toggle: bool,
    /// The player changed its bindings.
    pub rebound: bool,
}

impl ActionInfo {
    pub fn to_json(&self) -> Value {
        json!({
            "name": self.desc.name, "kind": self.desc.kind.name(), "context": self.desc.context,
            "label": if self.desc.label.is_empty() { &self.desc.name } else { &self.desc.label },
            "bindings": self.bindings.iter().map(Binding::to_json).collect::<Vec<_>>(),
            "defaults": self.desc.bindings.iter().map(Binding::to_json).collect::<Vec<_>>(),
            "locked": self.desc.locked, "toggle": self.toggle, "rebound": self.rebound,
        })
    }
}

/// An action's state and its bindings, as og-ui's UI actions follow it.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionFeed {
    pub name: String,
    pub state: ActionState,
    pub bindings: Vec<Binding>,
}

/// Gameplay actions, their bindings and state, and the player's
/// settings: see the module docs.
#[derive(Clone, Debug, Default)]
pub struct ActionMap {
    actions: Vec<Action>,
    /// Loaded bindings of actions not defined yet, applied when they are.
    pending: BTreeMap<String, Vec<Binding>>,
    pending_toggles: BTreeMap<String, bool>,
    contexts: Vec<String>,
    deadzones: [PadDeadzone; MAX_GAMEPADS],
    device: Device,
    capture: Option<Capture>,
    captured: CaptureState,
    /// The last sampled tick, and the pad axes deflected then (bit
    /// `pad * 6 + axis`): where a capture starts.
    tick: u64,
    deflected: u64,
    /// Session output and settings: never saved.
    rumble: Vec<Rumble>,
    surface: Surface,
}

/// An [`ActionMap`] shared between its system, its state block and the API.
pub type SharedActions = Arc<Mutex<ActionMap>>;

/// Locks a shared map (a panic elsewhere does not poison it for good).
pub fn lock(shared: &SharedActions) -> MutexGuard<'_, ActionMap> {
    shared.lock().unwrap_or_else(|e| e.into_inner())
}

impl ActionMap {
    pub fn new() -> ActionMap {
        ActionMap::default()
    }

    fn find(&self, name: &str) -> Option<usize> {
        self.actions.iter().position(|a| a.desc.name == name)
    }

    fn known(&self, name: &str) -> Result<usize, String> {
        self.find(name).ok_or_else(|| {
            format!("no action '{name}': define it with og.input.defineAction first")
        })
    }

    /// Defines an action, or redefines one (its state stays when its kind
    /// does, and the player's bindings stay while they fit it).
    pub fn define(&mut self, mut desc: ActionDesc) -> Result<(), String> {
        if desc.context.is_empty() {
            desc.context = DEFAULT_CONTEXT.into();
        }
        desc.check()?;
        let kind = desc.kind;
        let fits = |list: &[Binding]| list.iter().all(|b| b.check(kind).is_ok());
        match self.find(&desc.name) {
            Some(i) => {
                let a = &mut self.actions[i];
                if a.desc.kind != kind {
                    a.rt = Runtime::default();
                }
                if a.current.as_deref().is_some_and(|c| !fits(c)) {
                    a.current = None;
                }
                a.desc = desc;
                if a.current.as_deref() == Some(a.desc.bindings.as_slice()) {
                    a.current = None;
                }
            }
            None => {
                let current = self
                    .pending
                    .remove(&desc.name)
                    .filter(|c| fits(c) && *c != desc.bindings);
                let toggle = self
                    .pending_toggles
                    .remove(&desc.name)
                    .filter(|_| kind == ActionKind::Button);
                self.actions.push(Action {
                    desc,
                    current,
                    toggle,
                    rt: Runtime::default(),
                });
            }
        }
        Ok(())
    }

    /// Every action, in definition order.
    pub fn infos(&self) -> Vec<ActionInfo> {
        self.actions
            .iter()
            .map(|a| ActionInfo {
                desc: a.desc.clone(),
                bindings: a.bindings().to_vec(),
                toggle: a.toggles(),
                rebound: a.current.is_some(),
            })
            .collect()
    }

    /// An action's state this tick (`None`: no such action).
    pub fn state(&self, name: &str) -> Option<ActionState> {
        self.find(name).map(|i| self.actions[i].rt.state)
    }

    /// An action's bindings now.
    pub fn bindings(&self, name: &str) -> Option<&[Binding]> {
        self.find(name).map(|i| self.actions[i].bindings())
    }

    /// Every action's state and bindings (what og-ui's actions follow).
    pub fn feed(&self) -> Vec<ActionFeed> {
        self.actions
            .iter()
            .map(|a| ActionFeed {
                name: a.desc.name.clone(),
                state: a.rt.state,
                bindings: a.bindings().to_vec(),
            })
            .collect()
    }

    /// The contexts whose actions are live, first the one whose bindings
    /// win a control another holds; empty is every context, none winning.
    pub fn set_contexts(&mut self, contexts: Vec<String>) {
        self.contexts = contexts;
    }

    pub fn contexts(&self) -> &[String] {
        &self.contexts
    }

    /// A pad's dead zones (`None`: every pad).
    pub fn set_deadzone(&mut self, pad: Option<usize>, dz: PadDeadzone) -> Result<(), String> {
        if !(0.0..=0.95).contains(&dz.stick) || !(0.0..=0.95).contains(&dz.trigger) {
            return Err("a dead zone is 0 to 0.95".into());
        }
        match pad {
            Some(p) if p >= MAX_GAMEPADS => Err(format!("pad {p}: pads are 0 to 7")),
            Some(p) => {
                self.deadzones[p] = dz;
                Ok(())
            }
            None => {
                self.deadzones = [dz; MAX_GAMEPADS];
                Ok(())
            }
        }
    }

    pub fn deadzone(&self, pad: usize) -> PadDeadzone {
        self.deadzones.get(pad).copied().unwrap_or_default()
    }

    /// The device the player used last.
    pub fn device(&self) -> Device {
        self.device
    }

    /// The play surface touch zones are fractions of (the host's viewport).
    pub fn set_surface(&mut self, surface: Surface) {
        self.surface = surface;
    }

    /// The player's toggle for a button action (`None`: its desc's).
    pub fn set_toggle(&mut self, name: &str, on: Option<bool>) -> Result<(), String> {
        let i = self.known(name)?;
        if self.actions[i].desc.kind != ActionKind::Button {
            return Err(format!("{name}: only a button action toggles"));
        }
        self.actions[i].toggle = on;
        Ok(())
    }

    /// Takes the player's settings from `from` (each action's bindings and
    /// toggle, by name; bindings kept for actions not defined yet; the
    /// pads' dead zones), keeping everything else: what a loaded save keeps
    /// of the session.
    pub fn keep_player(&mut self, from: &ActionMap) {
        for a in &mut self.actions {
            match from.actions.iter().find(|b| b.desc.name == a.desc.name) {
                Some(b) => {
                    let kind = a.desc.kind;
                    a.current = b
                        .current
                        .clone()
                        .filter(|c| c.iter().all(|x| x.check(kind).is_ok()));
                    a.toggle = b.toggle.filter(|_| kind == ActionKind::Button);
                }
                None => {
                    a.current = from.pending.get(&a.desc.name).cloned();
                    a.toggle = from.pending_toggles.get(&a.desc.name).copied();
                }
            }
        }
        self.pending = from.pending.clone();
        self.pending_toggles = from.pending_toggles.clone();
        for b in &from.actions {
            if self.find(&b.desc.name).is_none() {
                if let Some(c) = &b.current {
                    self.pending.insert(b.desc.name.clone(), c.clone());
                }
                if let Some(t) = b.toggle {
                    self.pending_toggles.insert(b.desc.name.clone(), t);
                }
            }
        }
        self.deadzones = from.deadzones;
    }

    /// Queues a rumble for the host.
    pub fn rumble(&mut self, r: Rumble) -> Result<(), String> {
        if !(-1..MAX_GAMEPADS as i32).contains(&r.pad) {
            return Err(format!(
                "pad {}: pads are 0 to 7, or -1 for every pad",
                r.pad
            ));
        }
        if ![r.low, r.high, r.ms].iter().all(|v| v.is_finite()) || r.ms < 0.0 {
            return Err("low, high and ms are finite, ms 0 or more".into());
        }
        self.rumble.push(Rumble {
            low: r.low.clamp(0.0, 1.0),
            high: r.high.clamp(0.0, 1.0),
            ..r
        });
        Ok(())
    }

    /// The rumbles asked for since the last call.
    pub fn take_rumble(&mut self) -> Vec<Rumble> {
        std::mem::take(&mut self.rumble)
    }

    // ── sampling ────────────────────────────────────────────────────────

    /// Reads this tick's actions from the tick's input. `tick` and `dt`
    /// are og.Time's (holds and a capture's timeout count ticks).
    pub fn sample(
        &mut self,
        input: &Input,
        pointers: &Pointers,
        pads: &Gamepad,
        tick: u64,
        dt: f32,
    ) {
        let deadzones = self.deadzones;
        let reader = Reader {
            input,
            pointers,
            pads,
            deadzones: &deadzones,
            surface: self.surface,
        };
        self.track_device(&reader);
        self.tick = tick;
        self.deflected = 0;
        for pad in 0..MAX_GAMEPADS as u32 {
            for (k, ax) in GamepadAxis::ALL.iter().enumerate() {
                if pads.axis(*ax, pad).abs() >= 0.25 {
                    self.deflected |= 1 << (pad as usize * 6 + k);
                }
            }
        }
        let capturing = self.capture.is_some();
        if capturing {
            self.capture_step(&reader, tick, dt);
        }
        let claims = self.claims();
        let contexts = self.contexts.clone();
        for a in &mut self.actions {
            let rank = if contexts.is_empty() {
                Some(usize::MAX)
            } else {
                contexts.iter().position(|c| *c == a.desc.context)
            };
            let muted = |c: Control| {
                rank.is_some_and(|r| claims.iter().any(|(cc, ci)| *ci < r && overlaps(*cc, c)))
            };
            // The binding that reads most this tick.
            let mut best = Reading::default();
            let mut best_len = 0.0f32;
            let mut any_pressed = false;
            if rank.is_some() && !capturing {
                for b in a.bindings() {
                    if b.is_empty() {
                        continue;
                    }
                    let r = b.read(a.desc.kind, &reader, &muted);
                    any_pressed |= r.pressed;
                    let len = match a.desc.kind {
                        ActionKind::Axis2 => r.value.length(),
                        _ => r.value.x.abs(),
                    };
                    if len > best_len
                        || best.device.is_none() && r.device.is_some() && len >= best_len
                    {
                        best_len = len;
                        best = r;
                    }
                }
            }
            let rt = &mut a.rt;
            if rt.suppressed {
                if best_len == 0.0 && !any_pressed {
                    rt.suppressed = false;
                }
                best = Reading::default();
                best_len = 0.0;
                any_pressed = false;
            }
            let (value, vector) = match a.desc.kind {
                ActionKind::Button => (best_len, Vec2::new(best_len, 0.0)),
                ActionKind::Axis => {
                    let v = deadzone(best.value.x, a.desc.deadzone);
                    (v, Vec2::new(v, 0.0))
                }
                ActionKind::Axis2 => {
                    let v = if a.desc.deadzone > 0.0 {
                        deadzone2(best.value, a.desc.deadzone)
                    } else {
                        best.value
                    };
                    (v.length(), v)
                }
            };
            let magnitude = value.abs();
            let threshold = a.desc.threshold;
            let on = if rt.raw_down {
                magnitude >= threshold * 0.75 && magnitude > 0.0
            } else {
                magnitude >= threshold
            };
            // A press and a release between two ticks: both edges.
            let tap = any_pressed && !on;
            let pressed = (!rt.raw_down && on) || tap;
            let released = (rt.raw_down && !on) || tap;
            rt.raw_down = on;
            let mut s = ActionState {
                value,
                vector,
                device: best.device.unwrap_or(rt.state.device),
                ..ActionState::default()
            };
            if a.toggle.unwrap_or(a.desc.toggle) {
                if pressed {
                    rt.toggled = !rt.toggled;
                    s.pressed = rt.toggled;
                    s.released = !rt.toggled;
                }
                s.down = rt.toggled;
                s.value = f32::from(u8::from(rt.toggled));
                s.vector = Vec2::new(s.value, 0.0);
            } else {
                s.down = on;
                s.pressed = pressed;
                s.released = released;
            }
            if s.pressed {
                rt.since = tick;
            }
            if a.desc.hold_ms > 0.0 && s.down && dt > 0.0 {
                let need = ((a.desc.hold_ms / 1000.0) / dt).ceil() as u64;
                s.held = tick.saturating_sub(rt.since) >= need;
            }
            rt.state = s;
        }
        if capturing && self.capture.is_none() {
            // The capture ended: what the player pressed for it moves no
            // action until it is let go.
            for a in &mut self.actions {
                a.rt.suppressed = true;
            }
        }
    }

    /// Each control the live contexts bind, with the rank of the first
    /// context that binds it (empty when every context is live).
    fn claims(&self) -> Vec<(Control, usize)> {
        let mut out: Vec<(Control, usize)> = Vec::new();
        for (rank, ctx) in self.contexts.iter().enumerate() {
            for a in self.actions.iter().filter(|a| a.desc.context == *ctx) {
                for b in a.bindings() {
                    for (_, c) in b.controls() {
                        if !out.iter().any(|(x, _)| *x == c) {
                            out.push((c, rank));
                        }
                    }
                }
            }
        }
        out
    }

    fn track_device(&mut self, r: &Reader) {
        for pad in 0..MAX_GAMEPADS as u32 {
            if !r.pads.is_connected(pad) {
                continue;
            }
            let moved = GamepadAxis::ALL
                .iter()
                .any(|a| r.pads.axis(*a, pad).abs() > 0.5);
            if r.pads.masks(pad).1 != 0 || moved {
                self.device = r.pads.device(pad).unwrap_or(Device::Xbox);
                return;
            }
        }
        let p = r.pointers;
        let touch_mask = if r.surface.coarse { !0 } else { !1 };
        if p.pressed & touch_mask != 0 {
            self.device = Device::Touch;
            return;
        }
        let keys = (0..4).any(|w| r.input.word(KeySet::Pressed, w) != 0);
        let mouse = !r.surface.coarse
            && (r.input.buttons_pressed != 0
                || r.input.wheel != Vec2::ZERO
                || p.movement != Vec2::ZERO);
        if keys || mouse {
            self.device = Device::KeyboardMouse;
        }
    }

    // ── rebinding ───────────────────────────────────────────────────────

    /// The other bindings (of actions in the same context, or other slots
    /// of this one) that share a control with `binding` in `slot` of
    /// `action`.
    pub fn conflicts(&self, action: &str, slot: u32, binding: &Binding) -> Vec<Conflict> {
        let Some(i) = self.find(action) else {
            return Vec::new();
        };
        let context = &self.actions[i].desc.context;
        let mine = binding.controls();
        let mut out = Vec::new();
        for (j, other) in self.actions.iter().enumerate() {
            if other.desc.context != *context {
                continue;
            }
            for (s, b) in other.bindings().iter().enumerate() {
                if j == i && s as u32 == slot {
                    continue;
                }
                let theirs = b.controls();
                if let Some((_, c)) = mine
                    .iter()
                    .find(|(_, c)| theirs.iter().any(|(_, t)| overlaps(*c, *t)))
                {
                    out.push(Conflict {
                        action: other.desc.name.clone(),
                        slot: s as u32,
                        control: c.name(),
                        context: context.clone(),
                    });
                }
            }
        }
        out
    }

    /// The binding `slot` of `action` becomes with `binding` (or, for a
    /// part, with the part's control), checked against the action's kind.
    fn rebound(
        &self,
        i: usize,
        slot: u32,
        part: &str,
        binding: &Binding,
    ) -> Result<Binding, String> {
        let a = &self.actions[i];
        let list = a.bindings();
        if slot as usize > list.len() {
            return Err(format!(
                "{}: slot {slot} is past its {} bindings (one past the last adds one)",
                a.desc.name,
                list.len()
            ));
        }
        let mut out = if part.is_empty() {
            binding.clone()
        } else {
            if !a.desc.kind.parts().contains(&part) {
                return Err(format!(
                    "{}: a {} action has no part '{part}'{}",
                    a.desc.name,
                    a.desc.kind.name(),
                    match a.desc.kind {
                        ActionKind::Button => String::new(),
                        k => format!(" (its parts: {})", k.parts().join(", ")),
                    }
                ));
            }
            let mut b = list.get(slot as usize).cloned().unwrap_or_default();
            b.control.clear();
            *b.part_mut(part).expect("a part") = binding.control.clone();
            b
        };
        if out.zone == Zone::default()
            && let Some(old) = list.get(slot as usize)
            && part.is_empty()
            && out
                .controls()
                .iter()
                .all(|(_, c)| c.class() == Device::Touch)
        {
            out.zone = old.zone;
        }
        out.check(a.desc.kind)
            .map_err(|e| format!("{}: {e}", a.desc.name))?;
        Ok(out)
    }

    /// Rebinds a slot of an action: the player's binding, kept until reset
    /// and written by [`ActionMap::save_bindings`].
    pub fn rebind(&mut self, desc: &RebindDesc) -> Result<RebindAnswer, String> {
        let i = self.known(&desc.action)?;
        if self.actions[i].desc.locked {
            return Err(format!(
                "{} is locked: the player cannot rebind it",
                desc.action
            ));
        }
        let new = self.rebound(i, desc.slot, &desc.part, &desc.binding)?;
        let conflicts = self.conflicts(&desc.action, desc.slot, &new);
        if !conflicts.is_empty() && desc.policy == ConflictPolicy::Refuse {
            return Ok(RebindAnswer {
                applied: false,
                conflicts,
            });
        }
        let old = self.actions[i]
            .bindings()
            .get(desc.slot as usize)
            .cloned()
            .unwrap_or_default();
        // What the other actions get in a swap: what this slot (or part) had.
        let given = if desc.part.is_empty() {
            old.control.clone()
        } else {
            old.part(&desc.part).unwrap_or_default().to_string()
        };
        let mut changes: Vec<(usize, usize, Binding)> = Vec::new();
        if matches!(desc.policy, ConflictPolicy::Swap | ConflictPolicy::Replace) {
            for c in &conflicts {
                let j = self.find(&c.action).expect("a conflict names an action");
                if j != i && self.actions[j].desc.locked {
                    return Err(format!(
                        "{} is locked: its {} cannot move (refuse or allow instead)",
                        c.action, c.control
                    ));
                }
                let shared = Control::parse(&c.control).expect("a conflict names a control");
                let mut b = changes
                    .iter()
                    .find(|(x, s, _)| *x == j && *s == c.slot as usize)
                    .map(|(_, _, b)| b.clone())
                    .unwrap_or_else(|| self.actions[j].bindings()[c.slot as usize].clone());
                let to = if desc.policy == ConflictPolicy::Swap {
                    given.clone()
                } else {
                    String::new()
                };
                let kind = self.actions[j].desc.kind;
                for name in ["", "negative", "positive", "up", "down", "left", "right"] {
                    let now = if name.is_empty() {
                        b.control.as_str()
                    } else {
                        b.part(name).expect("a part")
                    };
                    if !Control::parse(now).is_some_and(|f| overlaps(f, shared)) {
                        continue;
                    }
                    let mut trial = b.clone();
                    set_field(&mut trial, name, &to);
                    if trial.check(kind).is_err() {
                        // What this slot had does not fit there: unbind it instead.
                        set_field(&mut trial, name, "");
                    }
                    b = trial;
                }
                match changes
                    .iter_mut()
                    .find(|(x, s, _)| *x == j && *s == c.slot as usize)
                {
                    Some(entry) => entry.2 = b,
                    None => changes.push((j, c.slot as usize, b)),
                }
            }
        }
        changes.push((i, desc.slot as usize, new));
        for (j, slot, b) in changes {
            let a = &mut self.actions[j];
            let mut list = a.bindings().to_vec();
            if slot == list.len() {
                list.push(b);
            } else {
                list[slot] = b;
            }
            a.current = (list != a.desc.bindings).then_some(list);
        }
        Ok(RebindAnswer {
            applied: true,
            conflicts,
        })
    }

    /// Back to the defaults: one action's bindings and toggle, or every
    /// action's.
    pub fn reset(&mut self, action: Option<&str>) -> Result<(), String> {
        match action {
            Some(name) => {
                let i = self.known(name)?;
                self.actions[i].current = None;
                self.actions[i].toggle = None;
            }
            None => {
                for a in &mut self.actions {
                    a.current = None;
                    a.toggle = None;
                }
                self.pending.clear();
                self.pending_toggles.clear();
                self.deadzones = [PadDeadzone::DEFAULT; MAX_GAMEPADS];
            }
        }
        Ok(())
    }

    /// The player's settings as text to keep (og.platform's settings, a
    /// save): every rebound action's bindings, the toggles they set and
    /// the dead zones that differ from the defaults.
    pub fn save_bindings(&self) -> String {
        let mut bindings = Map::new();
        let mut toggles = Map::new();
        for a in &self.actions {
            if let Some(c) = &a.current {
                bindings.insert(
                    a.desc.name.clone(),
                    Value::Array(c.iter().map(Binding::to_json).collect()),
                );
            }
            if let Some(t) = a.toggle {
                toggles.insert(a.desc.name.clone(), json!(t));
            }
        }
        // Loaded for actions not defined (yet) in this run: kept as they were.
        for (name, c) in &self.pending {
            bindings
                .entry(name.clone())
                .or_insert_with(|| Value::Array(c.iter().map(Binding::to_json).collect()));
        }
        for (name, t) in &self.pending_toggles {
            toggles.entry(name.clone()).or_insert(json!(t));
        }
        let deadzones: Vec<Value> = self
            .deadzones
            .iter()
            .enumerate()
            .filter(|(_, d)| **d != PadDeadzone::DEFAULT)
            .map(|(p, d)| json!({ "pad": p, "stick": d.stick, "trigger": d.trigger }))
            .collect();
        json!({ "version": BINDINGS_VERSION, "bindings": bindings, "toggles": toggles, "deadzones": deadzones })
            .to_string()
    }

    /// Takes what [`ActionMap::save_bindings`] wrote, in place of the
    /// player's settings now. Answers a warning for each entry it could
    /// not take (and leaves that action at its defaults); bindings of an
    /// action not defined yet are kept until it is.
    pub fn load_bindings(&mut self, text: &str) -> Result<Vec<String>, String> {
        let v: Value =
            serde_json::from_str(text).map_err(|e| format!("the bindings are not JSON: {e}"))?;
        let version = v["version"].as_u64().unwrap_or(0);
        if version != BINDINGS_VERSION {
            return Err(format!(
                "bindings version {version}: this engine reads version {BINDINGS_VERSION}"
            ));
        }
        let mut warnings = Vec::new();
        for a in &mut self.actions {
            a.current = None;
            a.toggle = None;
        }
        self.pending.clear();
        self.pending_toggles.clear();
        self.deadzones = [PadDeadzone::DEFAULT; MAX_GAMEPADS];
        for (name, list) in v["bindings"].as_object().into_iter().flatten() {
            let parsed: Result<Vec<Binding>, String> = list
                .as_array()
                .ok_or_else(|| "its bindings are a list".to_string())
                .and_then(|l| l.iter().map(Binding::from_json).collect());
            let parsed = match parsed {
                Ok(p) => p,
                Err(e) => {
                    warnings.push(format!("{name}: {e}"));
                    continue;
                }
            };
            match self.find(name) {
                Some(i) => {
                    let a = &mut self.actions[i];
                    match parsed.iter().try_for_each(|b| b.check(a.desc.kind)) {
                        Ok(()) if a.desc.locked => warnings
                            .push(format!("{name} is locked: its saved bindings are not used")),
                        Ok(()) => a.current = (parsed != a.desc.bindings).then_some(parsed),
                        Err(e) => warnings.push(format!("{name}: {e}")),
                    }
                }
                None => {
                    self.pending.insert(name.clone(), parsed);
                }
            }
        }
        for (name, t) in v["toggles"].as_object().into_iter().flatten() {
            let Some(t) = t.as_bool() else {
                warnings.push(format!("{name}: its toggle is true or false"));
                continue;
            };
            match self.find(name) {
                Some(i) if self.actions[i].desc.kind == ActionKind::Button => {
                    self.actions[i].toggle = Some(t)
                }
                Some(_) => warnings.push(format!("{name}: only a button action toggles")),
                None => {
                    self.pending_toggles.insert(name.clone(), t);
                }
            }
        }
        for d in v["deadzones"].as_array().into_iter().flatten() {
            let pad = d["pad"].as_u64().unwrap_or(u64::MAX) as usize;
            let dz = PadDeadzone {
                stick: d["stick"].as_f64().unwrap_or(-1.0) as f32,
                trigger: d["trigger"].as_f64().unwrap_or(-1.0) as f32,
            };
            if let Err(e) = self.set_deadzone(Some(pad), dz) {
                warnings.push(format!("dead zone: {e}"));
            }
        }
        Ok(warnings)
    }

    // ── capture ─────────────────────────────────────────────────────────

    /// Starts capturing the player's next input for a slot of an action
    /// (a controls screen's "press a key"): from the next tick every
    /// action reads nothing until it ends ([`ActionMap::capture`]).
    pub fn start_capture(&mut self, desc: CaptureDesc) -> Result<(), String> {
        let i = self.known(&desc.action)?;
        let a = &self.actions[i];
        if a.desc.locked {
            return Err(format!(
                "{} is locked: the player cannot rebind it",
                desc.action
            ));
        }
        // The same checks a rebinding of this slot and part makes.
        self.rebound(i, desc.slot, &desc.part, &Binding::default())?;
        for c in &desc.cancel {
            Control::parse(c)
                .ok_or_else(|| format!("cancel: '{c}' names no control: {NO_CONTROL}"))?;
        }
        let disarmed = self.deflected;
        let tick = self.tick;
        self.captured = CaptureState {
            active: true,
            action: desc.action.clone(),
            slot: desc.slot,
            part: desc.part.clone(),
            ..CaptureState::default()
        };
        self.capture = Some(Capture {
            desc,
            start: tick,
            disarmed,
        });
        Ok(())
    }

    /// Where the capture is: waiting, done with the control it took and
    /// the conflicts applying it would meet, or cancelled.
    pub fn capture(&self) -> &CaptureState {
        &self.captured
    }

    /// Stops a capture without taking anything.
    pub fn cancel_capture(&mut self) {
        if self.capture.take().is_some() {
            self.captured.active = false;
            self.captured.cancelled = true;
            for a in &mut self.actions {
                a.rt.suppressed = true;
            }
        }
    }

    /// Applies what a capture took ([`ActionMap::rebind`] with its slot,
    /// part and binding). Refused when no capture is done.
    pub fn apply_capture(&mut self, policy: ConflictPolicy) -> Result<RebindAnswer, String> {
        if !self.captured.done {
            return Err("no capture took a control: start one with og.input.startCapture and wait for capture().done".into());
        }
        let c = self.captured.clone();
        let answer = self.rebind(&RebindDesc {
            action: c.action,
            slot: c.slot,
            part: String::new(),
            binding: c.binding,
            policy,
        })?;
        if answer.applied {
            self.captured.done = false;
        } else {
            self.captured.conflicts = answer.conflicts.clone();
        }
        Ok(answer)
    }

    fn capture_step(&mut self, r: &Reader, tick: u64, dt: f32) {
        let Some(cap) = self.capture.as_mut() else {
            return;
        };
        // Axes that were deflected at the start count once they return.
        for pad in 0..MAX_GAMEPADS as u32 {
            for (k, ax) in GamepadAxis::ALL.iter().enumerate() {
                if r.pads.axis(*ax, pad).abs() < 0.25 {
                    cap.disarmed &= !(1 << (pad as usize * 6 + k));
                }
            }
        }
        let desc = cap.desc.clone();
        let disarmed = cap.disarmed;
        let cancel: Vec<Control> = if desc.cancel.is_empty() {
            vec![
                Control::parse("Escape").expect("a key"),
                Control::parse("pad.back").expect("a button"),
            ]
        } else {
            desc.cancel
                .iter()
                .filter_map(|c| Control::parse(c))
                .collect()
        };
        let o = ReadOptions::default();
        let timed_out = desc.timeout_ms > 0.0
            && dt > 0.0
            && tick.saturating_sub(cap.start) >= ((desc.timeout_ms / 1000.0) / dt).ceil() as u64;
        if timed_out || cancel.iter().any(|c| r.read(*c, &o).pressed) {
            self.capture = None;
            self.captured.active = false;
            self.captured.cancelled = true;
            return;
        }
        let Some(i) = self.find(&desc.action) else {
            self.capture = None;
            self.captured.active = false;
            self.captured.cancelled = true;
            return;
        };
        let kind = self.actions[i].desc.kind;
        let need = match (kind, desc.part.is_empty()) {
            (_, false) | (ActionKind::Button, true) => Need::Digital,
            (ActionKind::Axis, true) => Need::Signed,
            (ActionKind::Axis2, true) => Need::Plane,
        };
        let keys = desc.accept != CaptureAccept::Gamepad;
        let pads = desc.accept != CaptureAccept::KeyboardMouse;
        let mut found: Option<Control> = None;
        if keys && need != Need::Plane {
            found = (0..crate::KEY_CODES.len())
                .find(|k| r.input.contains(KeySet::Pressed, *k))
                .map(|k| Control::Key(k as u16))
                .or_else(|| {
                    [
                        crate::PointerButton::Left,
                        crate::PointerButton::Middle,
                        crate::PointerButton::Right,
                    ]
                    .into_iter()
                    .find(|b| r.input.buttons_pressed & b.bit() != 0)
                    .map(Control::Mouse)
                })
                .or_else(|| {
                    let w = r.input.wheel;
                    let half = |v: f32| {
                        if v < 0.0 {
                            Half::Positive
                        } else {
                            Half::Negative
                        }
                    };
                    if w.y != 0.0 {
                        Some(Control::Axis(
                            Axis::WheelY,
                            if need == Need::Signed {
                                Half::Full
                            } else {
                                half(w.y)
                            },
                        ))
                    } else if w.x != 0.0 {
                        Some(Control::Axis(
                            Axis::WheelX,
                            if need == Need::Signed {
                                Half::Full
                            } else {
                                half(-w.x)
                            },
                        ))
                    } else {
                        None
                    }
                });
        }
        if found.is_none() && pads {
            'pads: for pad in 0..MAX_GAMEPADS as u32 {
                if !r.pads.is_connected(pad) {
                    continue;
                }
                let pressed = r.pads.masks(pad).1;
                for (b, _) in GAMEPAD_BUTTONS.iter().enumerate() {
                    if pressed & (1 << b) == 0 || b == 6 || b == 7 {
                        continue;
                    }
                    found = match need {
                        Need::Plane if (12..=15).contains(&b) => Some(Control::Axis2(Axis2::Dpad)),
                        Need::Plane => None,
                        _ => Some(Control::PadButton(b as u8)),
                    };
                    if found.is_some() {
                        break 'pads;
                    }
                }
                for (k, ax) in GamepadAxis::ALL.iter().enumerate() {
                    let v = r.pads.axis(*ax, pad);
                    if v.abs() < 0.5 || disarmed & (1 << (pad as usize * 6 + k)) != 0 {
                        continue;
                    }
                    // Up-positive, as actions read it.
                    let v = if matches!(ax, GamepadAxis::LeftY | GamepadAxis::RightY) {
                        -v
                    } else {
                        v
                    };
                    found = Some(match (need, ax.is_trigger()) {
                        (Need::Plane, _) => Control::Axis2(match ax {
                            GamepadAxis::LeftX | GamepadAxis::LeftY => Axis2::PadLeft,
                            GamepadAxis::RightX | GamepadAxis::RightY => Axis2::PadRight,
                            _ => continue,
                        }),
                        (_, true) | (Need::Signed, _) => Control::Axis(Axis::Pad(*ax), Half::Full),
                        (Need::Digital, false) => Control::Axis(
                            Axis::Pad(*ax),
                            if v > 0.0 {
                                Half::Positive
                            } else {
                                Half::Negative
                            },
                        ),
                    });
                    break 'pads;
                }
            }
        }
        let Some(control) = found else {
            return;
        };
        let name = control.name();
        let template = Binding::control(&name);
        // A control that does not fit here (a stick for a key's part)
        // leaves it waiting for another.
        if let Ok(binding) = self.rebound(i, desc.slot, &desc.part, &template) {
            {
                let conflicts = self.conflicts(&desc.action, desc.slot, &binding);
                self.captured = CaptureState {
                    active: false,
                    done: true,
                    cancelled: false,
                    action: desc.action,
                    slot: desc.slot,
                    part: desc.part,
                    control: name,
                    binding,
                    conflicts,
                };
                self.capture = None;
            }
        }
    }

    // ── glyphs ──────────────────────────────────────────────────────────

    /// The glyphs of an action's first binding on `device` (the device
    /// used last when `None`), one per control: a composite's parts in
    /// prompt order (up, left, down, right; negative, positive). `None`
    /// for an unknown action; empty when it has no binding there.
    pub fn glyphs(&self, action: &str, device: Option<Device>) -> Option<Vec<(Glyph, String)>> {
        let i = self.find(action)?;
        let device = device.unwrap_or(self.device);
        Some(binding_glyphs(self.actions[i].bindings(), device))
    }
}

/// The glyphs of the first of `bindings` with a control on `device`'s
/// class, each with its control's name (a composite's parts in prompt
/// order). og-ui's UI actions draw theirs with it too.
pub fn binding_glyphs(bindings: &[Binding], device: Device) -> Vec<(Glyph, String)> {
    let class = device.class();
    for b in bindings {
        let controls = b.controls();
        if controls.is_empty() || controls.iter().any(|(_, c)| c.class() != class) {
            continue;
        }
        let mut ordered = controls;
        let rank = |p: Option<&str>| {
            ["up", "left", "down", "right", "negative", "positive"]
                .iter()
                .position(|x| Some(*x) == p)
                .unwrap_or(0)
        };
        ordered.sort_by_key(|(p, _)| rank(*p));
        return ordered
            .into_iter()
            .map(|(_, c)| (c.glyph(device), c.name()))
            .collect();
    }
    Vec::new()
}

/// One glyph for a list of a binding's glyphs: a single control's own, a
/// composite's texts together with a set's icon where one names them
/// (`key.wasd`, `key.arrows`, `xbox.dpad`), else its first part's.
pub fn combined_glyph(parts: &[(Glyph, String)], device: Device) -> Option<Glyph> {
    match parts {
        [] => None,
        [(g, _)] => Some(g.clone()),
        _ => {
            let names: Vec<&str> = parts.iter().map(|(_, n)| n.as_str()).collect();
            let icon = match names.as_slice() {
                ["KeyW", "KeyA", "KeyS", "KeyD"] => "key.wasd".to_string(),
                ["ArrowUp", "ArrowLeft", "ArrowDown", "ArrowRight"] => "key.arrows".to_string(),
                ["pad.up", "pad.left", "pad.down", "pad.right"] => {
                    Control::Axis2(Axis2::Dpad).glyph(device).icon
                }
                _ => parts[0].0.icon.clone(),
            };
            Some(Glyph {
                icon,
                text: parts
                    .iter()
                    .map(|(g, _)| g.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" "),
            })
        }
    }
}

// ── the state block ─────────────────────────────────────────────────────

fn rt_json(rt: &Runtime) -> Value {
    let s = rt.state;
    json!({
        "down": s.down, "pressed": s.pressed, "released": s.released, "held": s.held,
        "value": s.value, "vx": s.vector.x, "vy": s.vector.y, "device": s.device.index(),
        "rawDown": rt.raw_down, "since": rt.since, "toggled": rt.toggled, "suppressed": rt.suppressed,
    })
}

fn rt_of(v: &Value) -> Result<Runtime, String> {
    Ok(Runtime {
        state: ActionState {
            down: bool_of(v, "down", false)?,
            pressed: bool_of(v, "pressed", false)?,
            released: bool_of(v, "released", false)?,
            held: bool_of(v, "held", false)?,
            value: f32_of(v, "value", 0.0)?,
            vector: Vec2::new(f32_of(v, "vx", 0.0)?, f32_of(v, "vy", 0.0)?),
            device: Device::from_index(v["device"].as_u64().unwrap_or(0) as u32)
                .unwrap_or_default(),
        },
        raw_down: bool_of(v, "rawDown", false)?,
        since: v["since"].as_u64().unwrap_or(0),
        toggled: bool_of(v, "toggled", false)?,
        suppressed: bool_of(v, "suppressed", false)?,
    })
}

fn list_json(list: &[Binding]) -> Value {
    Value::Array(list.iter().map(Binding::to_json).collect())
}

fn list_of(v: &Value) -> Result<Vec<Binding>, String> {
    v.as_array()
        .ok_or("a list of bindings")?
        .iter()
        .map(Binding::from_json)
        .collect()
}

impl ActionMap {
    /// The map as the state block saves it: the actions with the player's
    /// bindings and each one's state, the contexts, dead zones, the device
    /// used last and a capture. Equal maps, equal bytes.
    pub fn save_state(&self) -> Vec<u8> {
        let v = json!({
            "actions": self.actions.iter().map(|a| json!({
                "desc": a.desc.to_json(),
                "current": a.current.as_deref().map(list_json),
                "toggle": a.toggle,
                "rt": rt_json(&a.rt),
            })).collect::<Vec<_>>(),
            "pending": self.pending.iter().map(|(k, v)| (k.clone(), list_json(v))).collect::<Map<_, _>>(),
            "pendingToggles": self.pending_toggles.iter().map(|(k, v)| (k.clone(), json!(v))).collect::<Map<_, _>>(),
            "contexts": self.contexts,
            "deadzones": self.deadzones.iter().map(|d| json!([d.stick, d.trigger])).collect::<Vec<_>>(),
            "device": self.device.index(),
            "tick": self.tick,
            "deflected": self.deflected,
            "capture": self.capture.as_ref().map(|c| json!({ "desc": c.desc.to_json(), "start": c.start, "disarmed": c.disarmed })),
            "captured": self.captured.to_json(),
        });
        v.to_string().into_bytes()
    }

    /// A map from [`ActionMap::save_state`]'s bytes (the session's
    /// surface and queued rumbles are not saved: `self`'s stay).
    pub fn restore_state(&self, bytes: &[u8]) -> Result<ActionMap, String> {
        let v: Value =
            serde_json::from_slice(bytes).map_err(|e| format!("og.input.actions: {e}"))?;
        let mut out = ActionMap {
            rumble: self.rumble.clone(),
            surface: self.surface,
            ..ActionMap::default()
        };
        for a in v["actions"].as_array().into_iter().flatten() {
            out.actions.push(Action {
                desc: ActionDesc::from_json(&a["desc"])?,
                current: match &a["current"] {
                    Value::Null => None,
                    c => Some(list_of(c)?),
                },
                toggle: a["toggle"].as_bool(),
                rt: rt_of(&a["rt"])?,
            });
        }
        for (k, l) in v["pending"].as_object().into_iter().flatten() {
            out.pending.insert(k.clone(), list_of(l)?);
        }
        for (k, t) in v["pendingToggles"].as_object().into_iter().flatten() {
            out.pending_toggles
                .insert(k.clone(), t.as_bool().unwrap_or(false));
        }
        out.contexts = v["contexts"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|c| c.as_str().map(str::to_string))
            .collect();
        for (p, d) in v["deadzones"]
            .as_array()
            .into_iter()
            .flatten()
            .enumerate()
            .take(MAX_GAMEPADS)
        {
            out.deadzones[p] = PadDeadzone {
                stick: d[0].as_f64().unwrap_or(0.15) as f32,
                trigger: d[1].as_f64().unwrap_or(0.05) as f32,
            };
        }
        out.device =
            Device::from_index(v["device"].as_u64().unwrap_or(0) as u32).unwrap_or_default();
        out.tick = v["tick"].as_u64().unwrap_or(0);
        out.deflected = v["deflected"].as_u64().unwrap_or(0);
        if let Some(c) = v["capture"].as_object() {
            out.capture = Some(Capture {
                desc: CaptureDesc::from_json(&c["desc"])?,
                start: c["start"].as_u64().unwrap_or(0),
                disarmed: c["disarmed"].as_u64().unwrap_or(0),
            });
        }
        out.captured = CaptureState::from_json(&v["captured"])?;
        Ok(out)
    }
}

/// The map as world state: a state block over a [`SharedActions`].
pub struct ActionsBlock(pub SharedActions);

impl og_ecs::StateBlock for ActionsBlock {
    fn save(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&lock(&self.0).save_state());
    }

    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn std::any::Any + Send>> {
        let map = lock(&self.0)
            .restore_state(bytes)
            .map_err(|e| og_core::Error::new(og_core::ErrorKind::Parse, e))?;
        Ok(Box::new(map))
    }

    fn apply(&mut self, decoded: Option<Box<dyn std::any::Any + Send>>) {
        let mut map = lock(&self.0);
        let next = match decoded.and_then(|d| d.downcast::<ActionMap>().ok()) {
            Some(m) => *m,
            None => ActionMap {
                rumble: map.rumble.clone(),
                surface: map.surface,
                ..ActionMap::default()
            },
        };
        *map = next;
    }
}

/// Adds the map's sampling system and its state block to `world`, at the
/// end of the schedule so far: add it before the game's systems. The
/// system reads og.Input, og.Pointers and og.Gamepad (registered by
/// [`crate::register`]); `surface` gives the play surface each tick
/// (`None` keeps the map's).
pub fn install(
    world: &mut og_ecs::World,
    shared: SharedActions,
    surface: impl Fn() -> Option<Surface> + Send + 'static,
) -> og_core::Result<()> {
    let sampler = shared.clone();
    world.add_system(
        og_ecs::SystemDesc::new(ACTIONS_SYSTEM)
            .read::<Input>()
            .read::<Pointers>()
            .read::<Gamepad>(),
        move |ctx| {
            let input = ctx.resource::<Input>().map(|r| *r).unwrap_or_default();
            let pointers = ctx.resource::<Pointers>().map(|r| *r).unwrap_or_default();
            let pads = ctx.resource::<Gamepad>().map(|r| *r).unwrap_or_default();
            let time = ctx.time();
            let mut map = lock(&sampler);
            if let Some(s) = surface() {
                map.set_surface(s);
            }
            map.sample(&input, &pointers, &pads, time.tick, time.dt);
            Ok(())
        },
    )?;
    world.add_state_block(ACTIONS_BLOCK, Box::new(ActionsBlock(shared)))
}
