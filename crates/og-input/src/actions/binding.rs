use super::*;

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
        }
    }
}

pub(super) fn f32_of(v: &Value, key: &str, default: f32) -> Result<f32, String> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(x) => x
            .as_f64()
            .map(|f| f as f32)
            .filter(|f| f.is_finite())
            .ok_or_else(|| format!("{key} is a finite number")),
    }
}

pub(super) fn str_of(v: &Value, key: &str) -> Result<String, String> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(String::new()),
        Some(x) => x
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| format!("{key} is a string")),
    }
}

pub(super) fn bool_of(v: &Value, key: &str, default: bool) -> Result<bool, String> {
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

    pub(super) fn part_mut(&mut self, part: &str) -> Option<&mut String> {
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

    /// A single analog control (the mouse's movement, the wheel, a stick,
    /// a trigger): what a player's sensitivity scales. Keys,
    /// buttons, the d-pad and composites read whole.
    pub fn is_analog(&self) -> bool {
        !self.is_composite()
            && matches!(
                Control::parse(&self.control),
                Some(Control::Axis(..))
                    | Some(Control::Axis2(
                        Axis2::Mouse | Axis2::PadLeft | Axis2::PadRight
                    ))
            )
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
                        "'{}' is one axis: a two-axis action binds pad.leftXY, pad.rightXY, pad.dpad, mouse.xy, or a composite (up, down, left, right)",
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
        Value::Object(m)
    }

    /// Its reading this tick for an action of `kind` (x for one value),
    /// with `muted` controls reading nothing.
    pub(super) fn read(
        &self,
        kind: ActionKind,
        r: &Reader,
        muted: &dyn Fn(Control) -> bool,
    ) -> Reading {
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

pub(super) fn set_field(b: &mut Binding, name: &str, to: &str) {
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
            Axis2::Dpad => &[],
        }
    };
    match (a, b) {
        // Touch zones may lie apart: touches never conflict.
        (Control::Touch, _) | (_, Control::Touch) => false,
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
