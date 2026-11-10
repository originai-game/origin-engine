//! Svelte's element directives (docs/ui.md §2), each mapped onto what
//! og-ui already does: `transition:`, `in:` and `out:` onto the element's
//! presence (`enter`, `exit`) and `crossfade` onto its shared-element
//! key (`match`); `animate:flip` onto its layout motion; `style:` and the
//! `style` attribute onto inline declarations that win the cascade;
//! `bind:this` onto its id; `use:` onto lifecycle events the game's
//! handler hears. All of it is presentation: nothing here reaches the
//! simulation.

use super::*;
use crate::element::Prop;
use crate::motion::{Ease, PresenceSpec, SpringSpec, Transition};

/// Svelte's built-in transitions (`svelte/transition`) and the
/// parameters each takes.
const TRANSITIONS: &[(&str, &[&str])] = &[
    ("fade", &["delay", "duration", "easing"]),
    ("fly", &["delay", "duration", "easing", "x", "y", "opacity"]),
    ("slide", &["delay", "duration", "easing", "axis"]),
    (
        "scale",
        &["delay", "duration", "easing", "start", "opacity"],
    ),
    (
        "blur",
        &["delay", "duration", "easing", "amount", "opacity"],
    ),
    ("draw", &["delay", "duration", "easing"]),
];

/// `svelte/easing`'s curves as CSS cubic Béziers; the elastic and bounce
/// ones are springs.
fn easing(name: &str, duration: f32) -> Result<Transition, String> {
    let bezier = |b: [f32; 4]| {
        Ok(Transition::Tween {
            duration_ms: duration,
            ease: Ease::CubicBezier(b),
        })
    };
    match name {
        "linear" => Ok(Transition::Tween {
            duration_ms: duration,
            ease: Ease::Linear,
        }),
        "sineIn" => bezier([0.12, 0.0, 0.39, 0.0]),
        "sineOut" => bezier([0.61, 1.0, 0.88, 1.0]),
        "sineInOut" => bezier([0.37, 0.0, 0.63, 1.0]),
        "quadIn" => bezier([0.11, 0.0, 0.5, 0.0]),
        "quadOut" => bezier([0.5, 1.0, 0.89, 1.0]),
        "quadInOut" => bezier([0.45, 0.0, 0.55, 1.0]),
        "cubicIn" => bezier([0.32, 0.0, 0.67, 0.0]),
        "cubicOut" => bezier([0.33, 1.0, 0.68, 1.0]),
        "cubicInOut" => bezier([0.65, 0.0, 0.35, 1.0]),
        "quartIn" => bezier([0.5, 0.0, 0.75, 0.0]),
        "quartOut" => bezier([0.25, 1.0, 0.5, 1.0]),
        "quartInOut" => bezier([0.76, 0.0, 0.24, 1.0]),
        "quintIn" => bezier([0.64, 0.0, 0.78, 0.0]),
        "quintOut" => bezier([0.22, 1.0, 0.36, 1.0]),
        "quintInOut" => bezier([0.83, 0.0, 0.17, 1.0]),
        "expoIn" => bezier([0.7, 0.0, 0.84, 0.0]),
        "expoOut" => bezier([0.16, 1.0, 0.3, 1.0]),
        "expoInOut" => bezier([0.87, 0.0, 0.13, 1.0]),
        "circIn" => bezier([0.55, 0.0, 1.0, 0.45]),
        "circOut" => bezier([0.0, 0.55, 0.45, 1.0]),
        "circInOut" => bezier([0.85, 0.0, 0.15, 1.0]),
        "backIn" => bezier([0.36, 0.0, 0.66, -0.56]),
        "backOut" => bezier([0.34, 1.56, 0.64, 1.0]),
        "backInOut" => bezier([0.68, -0.6, 0.32, 1.6]),
        "elasticIn" | "elasticOut" | "elasticInOut" => {
            Ok(Transition::Spring(SpringSpec::new(duration, 0.6)))
        }
        "bounceIn" | "bounceOut" | "bounceInOut" => {
            Ok(Transition::Spring(SpringSpec::new(duration, 0.35)))
        }
        other => Err(format!(
            "easing {other} is not one of svelte/easing's (linear, cubicOut, quintInOut, backOut, elasticOut…)"
        )),
    }
}

/// A directive's `{{ key: value, … }}` as written: each key with its
/// value's text.
fn params(a: &Attr) -> Result<Vec<(String, String)>, Error> {
    if !a.bound {
        if a.value == "true" {
            return Ok(Vec::new());
        }
        return Err((
            a.at,
            format!("{} takes {{{{ params }}}} or nothing", a.name),
        ));
    }
    let text = a.value.trim();
    let Some(inner) = text.strip_prefix('{').and_then(|t| t.strip_suffix('}')) else {
        return Err((
            a.at,
            format!(
                "{} takes its params as an object: {}={{{{ duration: 300 }}}}",
                a.name, a.name
            ),
        ));
    };
    let mut out = Vec::new();
    for entry in inner.split(',').map(str::trim).filter(|e| !e.is_empty()) {
        let Some((k, v)) = entry.split_once(':') else {
            return Err((
                a.at,
                format!("write each param as name: value, not {entry}"),
            ));
        };
        out.push((k.trim().to_string(), v.trim().to_string()));
    }
    Ok(out)
}

fn number(a: &Attr, key: &str, text: &str) -> Result<f32, Error> {
    text.trim_end_matches("px")
        .trim()
        .parse()
        .map_err(|_| (a.at, format!("{key} takes a literal number, not {text}")))
}

impl Parser {
    /// Handles `a` on element `i` (of kind `internal`) when it is a Svelte
    /// directive or the `style` attribute; `Ok(false)` otherwise.
    pub(super) fn directive(&mut self, i: usize, internal: &str, a: &Attr) -> Result<bool, Error> {
        let (head, mods) = match a.name.split_once('|') {
            Some((h, m)) => (h, m),
            None => (a.name.as_str(), ""),
        };
        if let Some(name) = head.strip_prefix("transition:") {
            self.transition(i, internal, a, name, mods, true, true)?;
        } else if let Some(name) = head.strip_prefix("in:") {
            self.transition(i, internal, a, name, mods, true, false)?;
        } else if let Some(name) = head.strip_prefix("out:") {
            self.transition(i, internal, a, name, mods, false, true)?;
        } else if let Some(name) = head.strip_prefix("animate:") {
            self.animate(i, internal, a, name)?;
        } else if let Some(name) = head.strip_prefix("use:") {
            if !identifier(name) || !mods.is_empty() {
                return Err((a.at, "write use:name or use:name={params}".into()));
            }
            if self.doc.nodes[i].uses.iter().any(|(n, _)| n == name) {
                return Err((a.at, format!("use:{name} is set twice")));
            }
            let params = if a.bound {
                let text = a.value.trim();
                Some(match crate::doc::script::literal(text) {
                    Some(v) => Expr::Lit(v),
                    None => expression(text, a.at)?,
                })
            } else if a.value == "true" {
                None
            } else {
                return Err((a.at, format!("use:{name} takes {{params}} or nothing")));
            };
            self.doc.nodes[i].uses.push((name.into(), params));
        } else if head == "bind:this" {
            if !a.bound || !identifier(&a.value) {
                return Err((
                    a.at,
                    "bind:this={name} names the element; the name reads as its id (popovertarget={name}, events' paths)".into(),
                ));
            }
            let name = a.value.clone();
            if !self.doc.nodes[i].id.is_empty() && self.doc.nodes[i].id != name {
                return Err((
                    a.at,
                    "the element has an id already; bind:this names it by its id".into(),
                ));
            }
            if !self.ids.insert(name.clone()) {
                return Err((
                    a.at,
                    format!("duplicate id '{name}'; bind:this names one element"),
                ));
            }
            if self.doc.props.iter().any(|p| p.name == name)
                || self.doc.derived.iter().any(|(n, _)| *n == name)
            {
                return Err((
                    a.at,
                    format!("{name} names a prop or derived value already"),
                ));
            }
            self.doc.nodes[i].id = name.clone();
            // `#name` selects it as an id attribute would.
            self.doc.nodes[i]
                .selector_attrs
                .push(("id".into(), Expr::Lit(name.clone().into())));
            if let Some((_, _, attrs)) = self.selectors.iter_mut().find(|(n, ..)| *n == i) {
                attrs.push(Attr {
                    name: "id".into(),
                    value: name.clone(),
                    bound: false,
                    at: a.at,
                    end: a.end,
                });
            }
            self.doc
                .derived
                .push((name.clone(), Expr::Lit(name.into())));
        } else if head == "style" {
            if a.bound || !mods.is_empty() {
                return Err((
                    a.at,
                    "the style attribute is literal CSS; bind a value with style:prop={value}"
                        .into(),
                ));
            }
            let declarations = super::super::stylesheet::inline(&self.doc, i, &a.value, a.at)?;
            self.inline_styles.push((i, declarations));
        } else if let Some(prop) = head.strip_prefix("style:") {
            if prop.starts_with("--") {
                return Ok(false);
            }
            let important = match mods {
                "" => false,
                "important" => true,
                other => {
                    return Err((
                        a.at,
                        format!("style:{prop}|{other}: the modifier is |important"),
                    ));
                }
            };
            let value = if a.bound {
                // The value is data: a custom property only this element
                // declares, which its own declaration reads.
                let var = format!("--og-style-{prop}");
                let expr = expression(&a.value, a.at)?;
                self.doc.nodes[i].custom.push((var.clone(), expr));
                format!("var({var})")
            } else {
                a.value.clone()
            };
            let body = format!(
                "{prop}: {value}{};",
                if important { " !important" } else { "" }
            );
            let declarations = super::super::stylesheet::inline(&self.doc, i, &body, a.at)?;
            self.inline_styles.push((i, declarations));
        } else {
            return Ok(false);
        }
        Ok(true)
    }

    #[allow(clippy::too_many_arguments)]
    fn transition(
        &mut self,
        i: usize,
        internal: &str,
        a: &Attr,
        name: &str,
        mods: &str,
        enter: bool,
        exit: bool,
    ) -> Result<(), Error> {
        for m in mods.split('|').filter(|m| !m.is_empty()) {
            if !matches!(m, "local" | "global") {
                return Err((
                    a.at,
                    format!("|{m} is not a transition modifier; use |local or |global"),
                ));
            }
        }
        if name == "crossfade" || name == "send" || name == "receive" {
            // A shared element: one that leaves and one that arrives with
            // the same key move from the first's place to the second's.
            let ps = params(a)?;
            let Some((_, key)) = ps.iter().find(|(k, _)| k == "key") else {
                return Err((
                    a.at,
                    format!(
                        "{} needs {{{{ key: … }}}}, the element's shared key",
                        a.name
                    ),
                ));
            };
            if let Some((k, _)) = ps.iter().find(|(k, _)| k != "key") {
                return Err((
                    a.at,
                    format!(
                        "crossfade takes only key here, not {k}; time the move with CSS -og-layout-motion"
                    ),
                ));
            }
            if self.doc.nodes[i].props.iter().any(|p| p.name() == "match") {
                return Ok(()); // `in:receive` and `out:send` name one key.
            }
            let value = PropValue::Bind(expression(key, a.at)?, None);
            return self.prop(i, internal, "match", value, a.at, key);
        }
        if name == "draw" && internal != "svgShape" {
            return Err((
                a.at,
                "draw strokes an SVG shape on: put it on a <path>, <line>, <polyline>, <polygon>, <rect>, <circle> or <ellipse>".into(),
            ));
        }
        let Some((_, allowed)) = TRANSITIONS.iter().find(|(n, _)| *n == name) else {
            return Err((
                a.at,
                format!(
                    "{name} is not a built-in transition; use fade, fly, slide, scale, blur, draw, crossfade, or CSS @starting-style"
                ),
            ));
        };
        let ps = params(a)?;
        let mut duration = 400.0;
        let mut delay = 0.0;
        let mut ease = match name {
            "fade" => "linear",
            "draw" => "cubicInOut",
            _ => "cubicOut",
        }
        .to_string();
        let mut amount = 5.0;
        let mut values: Vec<(Prop, f32)> = Vec::new();
        let mut opacity = 0.0;
        for (k, v) in &ps {
            if name == "draw" && k == "speed" {
                return Err((
                    a.at,
                    "draw's speed reads the path's length, which a document does not know; give duration".into(),
                ));
            }
            if !allowed.contains(&k.as_str()) {
                return Err((
                    a.at,
                    format!("{name} takes {}, not {k}", allowed.join(", ")),
                ));
            }
            match k.as_str() {
                "duration" => duration = number(a, k, v)?,
                "delay" => delay = number(a, k, v)?,
                "easing" => ease = v.clone(),
                "opacity" => opacity = number(a, k, v)?,
                "x" => values.push((Prop::X, number(a, k, v)?)),
                "y" => values.push((Prop::Y, number(a, k, v)?)),
                "start" => values.push((Prop::Scale, number(a, k, v)?)),
                "amount" => amount = number(a, k, v)?,
                "axis" => {
                    let axis = v.trim_matches(['\'', '"']);
                    values.retain(|(p, _)| !matches!(p, Prop::ScaleX | Prop::ScaleY));
                    values.push(match axis {
                        "x" => (Prop::ScaleX, 0.0),
                        "y" => (Prop::ScaleY, 0.0),
                        _ => return Err((a.at, "axis is 'x' or 'y'".into())),
                    });
                }
                _ => unreachable!(),
            }
        }
        match name {
            // Svelte's slide changes its size without fading; og-ui's
            // presence scales it along the axis from its start edge.
            "slide" => {
                if values.is_empty() {
                    values.push((Prop::ScaleY, 0.0));
                }
            }
            "scale" if !values.iter().any(|(p, _)| *p == Prop::Scale) => {
                values.push((Prop::Scale, 0.0));
                values.push((Prop::Opacity, opacity));
            }
            // CSS `filter: blur()` from `amount` px, fading to `opacity`.
            "blur" => {
                values.push((Prop::Blur, amount));
                values.push((Prop::Opacity, opacity));
            }
            // The stroke drawn on from its start, the fill as it is.
            "draw" => values.push((Prop::Draw, 1.0)),
            _ => values.push((Prop::Opacity, opacity)),
        }
        let transition = easing(&ease, duration).map_err(|e| (a.at, e))?;
        let spec = PresenceSpec {
            values,
            transition,
            delay_ms: delay,
        };
        for (on, prop) in [(enter, "enter"), (exit, "exit")] {
            if !on {
                continue;
            }
            if self.doc.nodes[i].props.iter().any(|p| p.name() == prop) {
                return Err((
                    a.at,
                    format!(
                        "the element's {prop} is set twice; write one of transition:, in: and out: for it"
                    ),
                ));
            }
            let value = PropValue::Lit(Lit::Presence(spec.clone()));
            self.prop(i, internal, prop, value, a.at, &a.value)?;
        }
        Ok(())
    }

    fn animate(&mut self, i: usize, internal: &str, a: &Attr, name: &str) -> Result<(), Error> {
        if name != "flip" {
            return Err((
                a.at,
                format!("animate:{name} is not built in; use animate:flip"),
            ));
        }
        let parent = self.doc.nodes[i].parent;
        if parent == NONE || !matches!(self.doc.nodes[parent].kind, NodeKind::For { .. }) {
            return Err((
                a.at,
                "animate: sits on an element directly inside a keyed {#each}".into(),
            ));
        }
        let mut duration = 300.0;
        let mut ease = "cubicOut".to_string();
        for (k, v) in params(a)? {
            match k.as_str() {
                "duration" => duration = number(a, &k, &v)?,
                "easing" => ease = v,
                "delay" => {
                    return Err((
                        a.at,
                        "flip starts with the change; it takes no delay".into(),
                    ));
                }
                _ => return Err((a.at, format!("flip takes duration and easing, not {k}"))),
            }
        }
        let transition = easing(&ease, duration).map_err(|e| (a.at, e))?;
        self.prop(
            i,
            internal,
            "layoutMotion",
            PropValue::Lit(Lit::Transition(transition)),
            a.at,
            &a.value,
        )
    }
}
