//! Concrete exposed-part identities and shared paint values. Part styling
//! uses the document CSS cascade, with no separate resolution path.

use crate::paint::{Layer, Paint};
use crate::tokens::{ElementClass, Level, ShapeToken};
use std::fmt;

/// A dressing's material: a built-in's layer, or a `type ui-material`
/// asset named by path (`asset`, bound to its handle when the style's
/// assets load; `layer.material` is plain until then), with the `params`
/// numbers it set (the rest are the material's defaults).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DressMaterial {
    pub layer: Layer,
    pub asset: Option<String>,
    pub params: usize,
}

/// The style's ornament art a dressing puts on its part.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrnamentKind {
    Corner,
    Divider,
    Flourish,
}

/// A track drawn as `count` segments `gap` CSS px apart, its fill a whole
/// number of segments (the nearest to the value); with `ramp`, filled
/// segment k in `from` mixed towards `to` by k / (count − 1) (cyber's
/// signal bars, a pixel meter's blocks).
#[derive(Clone, Debug, PartialEq)]
pub struct Segments {
    pub count: u32,
    pub gap: f32,
    pub ramp: Option<[ColorRef; 2]>,
}

impl Segments {
    /// The segments' rects across `track`, start to end.
    pub fn rects(&self, track: crate::Rect) -> Vec<crate::Rect> {
        let n = self.count.max(1) as f32;
        let gap = self.gap.min(track.width / n / 2.0).max(0.0);
        let w = ((track.width - gap * (n - 1.0)) / n).max(0.0);
        (0..self.count)
            .map(|k| crate::Rect::new(track.x + k as f32 * (w + gap), track.y, w, track.height))
            .collect()
    }

    /// How many segments a value 0–1 fills: the nearest whole number.
    pub fn filled(&self, value: f32) -> u32 {
        ((value.clamp(0.0, 1.0) * self.count as f32).round() as u32).min(self.count)
    }

    /// Filled segment `k`'s colour on its ramp, or `fill`.
    pub fn color(&self, k: u32, fill: Srgba, theme: &Theme) -> Srgba {
        let Some([a, b]) = &self.ramp else {
            return fill;
        };
        let of = |c: &ColorRef| c.srgb(theme).unwrap_or(fill);
        let t = k as f32 / (self.count.max(2) - 1) as f32;
        of(a).mix(of(b), t)
    }
}

// ───────────────────────────────────────────── parts and their resolution

/// Qualifiers a [`PartRef`] carries (variants and states).
pub const MAX_FLAGS: usize = 8;
/// Attributes a [`PartRef`] carries (`rarity=epic`, `tone=success`).
pub const MAX_ATTRS: usize = 3;

/// A name made `'static` once, so a part a game names at run time
/// (`og.ui` element descs, documents) is as cheap to carry as a widget's
/// own: the set of component, part, flag and attribute names is small.
pub fn intern(s: &str) -> &'static str {
    use std::collections::HashSet;
    use std::sync::{Mutex, OnceLock};
    static NAMES: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
    let mut set = NAMES
        .get_or_init(|| Mutex::new(HashSet::new()))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if let Some(n) = set.get(s) {
        return n;
    }
    let n: &'static str = Box::leak(s.to_string().into_boxed_str());
    set.insert(n);
    n
}

/// Host and part identity with the host's attributes and interaction state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PartRef {
    pub component: &'static str,
    pub part: &'static str,
    /// The control's own part when a document names another on its host
    /// (`<button part="Bag.cell">`), empty otherwise. Rules for either name
    /// match, as a `part` attribute exposes an element without taking its
    /// own styling away: `button::part(base)` still draws the plate and the
    /// named part's rules dress it further.
    pub own: &'static str,
    /// Widget layout classification. It supplies no styling fallback.
    pub class: Option<ElementClass>,
    pub level: Option<Level>,
    /// Empty entries are unused.
    pub flags: [&'static str; MAX_FLAGS],
    pub attrs: [(&'static str, &'static str); MAX_ATTRS],
    pub hover: f32,
    pub press: f32,
}

impl Default for PartRef {
    fn default() -> PartRef {
        PartRef::new("Element", "base", ElementClass::Panel)
    }
}

impl PartRef {
    pub const fn new(component: &'static str, part: &'static str, class: ElementClass) -> PartRef {
        PartRef::of(component, part, Some(class))
    }

    /// A part of `class`, or of none (see [`PartRef::class`]).
    pub const fn of(
        component: &'static str,
        part: &'static str,
        class: Option<ElementClass>,
    ) -> PartRef {
        PartRef {
            component,
            part,
            own: "",
            class,
            level: None,
            flags: [""; MAX_FLAGS],
            attrs: [("", ""); MAX_ATTRS],
            hover: 0.0,
            press: 0.0,
        }
    }

    /// With a flag (a variant or a state); an empty one, a repeat or one
    /// past [`MAX_FLAGS`] changes nothing.
    pub fn flag(mut self, f: &'static str) -> PartRef {
        if f.is_empty() || self.has(f) {
            return self;
        }
        if let Some(slot) = self.flags.iter_mut().find(|s| s.is_empty()) {
            *slot = f;
        }
        self
    }

    /// With `flag` when `on`.
    pub fn flag_if(self, on: bool, f: &'static str) -> PartRef {
        if on { self.flag(f) } else { self }
    }

    /// With an attribute (a later one of the same key replaces it).
    pub fn attr(mut self, k: &'static str, v: &'static str) -> PartRef {
        if k.is_empty() || v.is_empty() {
            return self;
        }
        if let Some(slot) = self.attrs.iter_mut().find(|(a, _)| *a == k) {
            slot.1 = v;
        } else if let Some(slot) = self.attrs.iter_mut().find(|(a, _)| a.is_empty()) {
            *slot = (k, v);
        }
        self
    }

    pub fn at_level(mut self, l: Level) -> PartRef {
        self.level = Some(l);
        self
    }

    pub fn has(&self, f: &str) -> bool {
        self.flags.contains(&f) && !f.is_empty()
    }

    pub fn attr_value(&self, k: &str) -> Option<&'static str> {
        self.attrs.iter().find(|(a, _)| *a == k).map(|(_, v)| *v)
    }

    pub fn flags(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.flags.iter().copied().filter(|f| !f.is_empty())
    }

    /// The same part in another interaction state: `hover` and `pressed`
    /// as given (the rest of its flags kept).
    pub fn in_state(&self, hover: bool, pressed: bool) -> PartRef {
        let mut p = *self;
        for f in p.flags.iter_mut() {
            if *f == "hover" || *f == "pressed" {
                *f = "";
            }
        }
        p.flag_if(hover, "hover").flag_if(pressed, "pressed")
    }

    /// A concrete part named with standard CSS: host facts, then
    /// `::part(name)`, then state pseudo-classes.
    pub fn parse(text: &str, class: impl Into<Option<ElementClass>>) -> Result<PartRef, String> {
        use crate::doc::selectors::{Selector, Simple};
        let selectors = Selector::parse_list(text)?;
        let [selector] = selectors.as_slice() else {
            return Err("name one part".into());
        };
        let [compound] = selector.compounds.as_slice() else {
            return Err("a part identity has one host".into());
        };
        let Some(Simple::Type(host)) = compound.0.first() else {
            return Err("a part starts with its host tag".into());
        };
        if host != &host.to_ascii_lowercase() {
            return Err("a part's host tag is lowercase".into());
        }
        let mut p = PartRef::of(intern(host), "", class.into());
        for simple in &compound.0[1..] {
            match simple {
                Simple::Part(name) if p.part.is_empty() => p.part = intern(name),
                Simple::Class(name) if p.part.is_empty() => p = p.flag(intern(name)),
                Simple::Attribute { name, test } if p.part.is_empty() => {
                    let value = match test {
                        None => "true",
                        Some(test) if test.op == crate::bind::AttrOp::Equals => &test.value,
                        _ => return Err("a concrete part attribute uses =".into()),
                    };
                    p = p.attr(intern(name), intern(value));
                }
                Simple::State(state) if !p.part.is_empty() && *state != crate::bind::InteractionState::Selected => {
                    let name = crate::doc::css::kebab(&format!("{state:?}"));
                    p = p.flag(intern(name.trim_start_matches('-')));
                }
                _ => return Err("write host classes and attributes before ::part(name), and standard state pseudo-classes after it".into()),
            }
        }
        if p.part.is_empty() {
            return Err("a part needs ::part(name)".into());
        }
        Ok(p)
    }
}

impl fmt::Display for PartRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.with_facts(|facts| {
            write!(f, "{}", facts.tag)?;
            if let Some(classes) = facts
                .attributes
                .get("class")
                .and_then(serde_json::Value::as_str)
            {
                for class in classes.split_whitespace() {
                    write!(f, ".{class}")?;
                }
            }
            for (name, value) in facts
                .attributes
                .iter()
                .filter(|(n, _)| n.as_str() != "class")
            {
                write!(f, "[{name}={}]", crate::bind::text(value))?;
            }
            write!(f, "::part({})", self.part)?;
            for state in facts.states {
                if *state == crate::bind::InteractionState::Enabled {
                    continue;
                }
                write!(
                    f,
                    ":{}",
                    crate::doc::css::kebab(&format!("{state:?}")).trim_start_matches('-')
                )?;
            }
            Ok(())
        })
    }
}

impl PartRef {
    pub(crate) fn with_facts<T>(&self, f: impl FnOnce(crate::doc::matching::Facts<'_>) -> T) -> T {
        use crate::bind::InteractionState as S;
        use serde_json::Value;
        use std::collections::{BTreeMap, BTreeSet};
        let mut attributes: BTreeMap<String, Value> = self
            .attrs
            .iter()
            .filter(|(k, _)| !k.is_empty())
            .map(|(k, v)| ((*k).into(), (*v).into()))
            .collect();
        let tag = match self.component {
            "Slider" | "Checkbox" | "Toggle" | "Radio" => {
                let ty = match self.component {
                    "Slider" => "range",
                    "Radio" => "radio",
                    _ => "checkbox",
                };
                attributes.insert("type".into(), ty.into());
                if self.component == "Toggle" {
                    attributes.insert("switch".into(), true.into());
                }
                "input".to_string()
            }
            "Dropdown" => "select".into(),
            "Field" => "input".into(),
            "TextEdit" => "textarea".into(),
            tag => tag.to_ascii_lowercase(),
        };
        let checkable = tag == "input"
            && attributes
                .get("type")
                .and_then(Value::as_str)
                .is_some_and(|s| matches!(s, "checkbox" | "radio"));
        let mut states = BTreeSet::new();
        let mut classes = attributes
            .get("class")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        for flag in self.flags() {
            let name = match flag {
                "pressed" => "active",
                "focused" => "focus-visible",
                _ => flag,
            };
            match name {
                "on" | "selected" if checkable || (tag == "tabs" && self.part == "tab") => {
                    states.insert(S::Checked);
                }
                "on" => {
                    attributes.insert("aria-pressed".into(), "true".into());
                }
                "selected" => {
                    attributes.insert("aria-selected".into(), "true".into());
                }
                "expanded" => {
                    attributes.insert("aria-expanded".into(), "true".into());
                }
                name => {
                    if let Some(state) = crate::doc::selectors::state(name) {
                        states.insert(state);
                    } else {
                        if !classes.is_empty() {
                            classes.push(' ');
                        }
                        classes.push_str(name);
                    }
                }
            }
        }
        if tag == "select" && states.contains(&S::Open) {
            attributes.insert("aria-expanded".into(), "true".into());
        }
        if !states.contains(&S::Disabled) {
            states.insert(S::Enabled);
        }
        attributes.insert("class".into(), classes.into());
        f(crate::doc::matching::Facts {
            scope: "",
            tag: &tag,
            part: Some(self.part),
            own_part: Some(self.own).filter(|p| !p.is_empty()),
            root: false,
            attributes: &attributes,
            states: &states,
            text: "",
        })
    }
}

// ──────────────────────────────────────────── a part drawn in the UI pass

use crate::paint::{ColorRef, Corner, ElementDraw};
use crate::style::{Srgba, Theme};
use crate::tokens::CornerShape;

/// The UI pass's corner for a shape token, and its radius.
pub fn corner_of(s: ShapeToken) -> (Corner, f32) {
    match s.corner {
        CornerShape::Round => (Corner::Round, s.radius),
        CornerShape::Squircle => (Corner::Squircle(4.0), s.radius),
        CornerShape::Notch => (Corner::Notch, s.radius),
        CornerShape::Bevel => (Corner::Bevel, s.radius),
        CornerShape::Square => (Corner::Round, 0.0),
    }
}

/// The colour a draw shows under its text: its fills' mean over `below`
/// (a gradient's stops averaged), in linear light as the renderer blends.
pub fn ground_of(d: &ElementDraw, theme: &Theme, below: Srgba) -> Srgba {
    let mut c = below;
    for effect in &d.effects {
        if let crate::paint::Effect::Backdrop {
            blur,
            saturation,
            brightness,
            tint,
            glass,
        } = effect
            && glass.weight.is_some()
        {
            // Contrast calculations use the material's nominal surface ink;
            // the pixels behind a live backdrop are not a stable UI input.
            let (_, tint) = glass.resolve(
                *blur,
                *saturation,
                *brightness,
                tint.resolve(theme),
                glass.own_tint,
                theme,
            );
            c = Srgba::from_linear(tint);
            c.alpha = 1.0;
        }
    }
    for f in &d.fills {
        let top = match f {
            Paint::Solid(r) => Srgba::from_linear(r.resolve(theme)),
            Paint::Gradient(g) if !g.stops.is_empty() => {
                let n = g.stops.len() as f32;
                let mut sum = [0.0f32; 4];
                for s in &g.stops {
                    let l = s.color.resolve(theme);
                    for k in 0..4 {
                        sum[k] += l[k] / n;
                    }
                }
                Srgba::from_linear(sum)
            }
            _ => continue,
        };
        c = top.over_linear(c);
    }
    c
}
