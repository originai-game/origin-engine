//! CSS Transitions Level 1 and CSS Animations Level 1 for document styles.
//! `transition` and `animation` expand into their longhands at cook, so the
//! cascade picks each longhand per element like any other property and
//! selectors, state and media choose an element's animations. The winning
//! lists combine when the element resolves: the property or name list sets
//! the count and the other lists repeat.
use super::StyleDeclaration;
use super::variables::Value as Tokens;
use crate::motion::css::{
    ALL_CHANNELS, CssAnimation, CssTransition, Direction, Keyframe, KeyframeTrack, Keyframes,
    StyleKey, StyleTrack,
};
use crate::motion::interpolate::{self, Composite};
use crate::motion::scroll::{AnimationTimeline, Axis, Inset, NamedTimeline, RangeEdge};
use crate::motion::{Ease, MotionToken, SpringSpec, Transition};

pub(crate) const TRANSITION: [&str; 5] = [
    "transition-property",
    "transition-duration",
    "transition-timing-function",
    "transition-delay",
    "transition-behavior",
];
pub(crate) const ANIMATION: [&str; 8] = [
    "animation-name",
    "animation-duration",
    "animation-timing-function",
    "animation-delay",
    "animation-iteration-count",
    "animation-direction",
    "animation-fill-mode",
    "animation-play-state",
];

/// Animation longhands the `animation` shorthand does not set; it resets
/// the timeline and range ones (CSS Scroll-driven Animations 1 §4.1).
const ANIMATION_EXTRA: [&str; 4] = [
    "animation-composition",
    "animation-timeline",
    "animation-range-start",
    "animation-range-end",
];

/// The declarations of named scroll and view timelines (CSS Scroll-driven
/// Animations 1 §2.3, §3.3, §4.2).
pub(crate) const TIMELINES: [&str; 6] = [
    "scroll-timeline-name",
    "scroll-timeline-axis",
    "view-timeline-name",
    "view-timeline-axis",
    "view-timeline-inset",
    "timeline-scope",
];

/// Whether a declaration is a transition, animation or timeline longhand.
pub(crate) fn longhand(name: &str) -> bool {
    TRANSITION.contains(&name)
        || ANIMATION.contains(&name)
        || ANIMATION_EXTRA.contains(&name)
        || TIMELINES.contains(&name)
}

/// The channels of a property the element's channels hold; any other
/// property animates by its computed value.
pub(crate) fn channels(property: &str) -> Option<u32> {
    let range = |a: u32, b: u32| (a..=b).fold(0, |m, c| m | 1 << c);
    Some(match property {
        "all" => ALL_CHANNELS,
        "translate" => range(0, 1),
        "scale" => range(2, 3),
        "rotate" => 1 << 4,
        "-og-skew" => range(5, 6),
        "opacity" => 1 << 7,
        "border-radius" => 1 << 8,
        "background" => range(9, 12),
        "color" => range(13, 16),
        "z-index" => 1 << 17,
        _ => return None,
    })
}

/// Top-level comma items, each split into space-separated words, keeping
/// function arguments together.
fn items(text: &str) -> Vec<Vec<String>> {
    let mut out = vec![Vec::new()];
    let mut word = String::new();
    let mut depth = 0;
    for c in text.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
        if depth == 0 && (c == ',' || c.is_whitespace()) {
            if !word.is_empty() {
                out.last_mut().unwrap().push(std::mem::take(&mut word));
            }
            if c == ',' {
                out.push(Vec::new());
            }
        } else {
            word.push(c);
        }
    }
    if !word.is_empty() {
        out.last_mut().unwrap().push(word);
    }
    out
}

fn time(word: &str) -> Option<f32> {
    let ms = if let Some(n) = word.strip_suffix("ms") {
        n.parse::<f32>().ok()?
    } else {
        word.strip_suffix('s')?.parse::<f32>().ok()? * 1000.0
    };
    ms.is_finite().then_some(ms)
}

/// A timing: a CSS easing function, or the engine's `-og-spring(<ms>,
/// <bounce>)` and `var(--motion-<role>)` timings.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Timing {
    Ease(Ease),
    Spring(SpringSpec),
    Token(MotionToken),
}

fn timing(word: &str) -> Option<Timing> {
    if let Some(role) = word
        .strip_prefix("var(--motion-")
        .and_then(|r| r.strip_suffix(')'))
    {
        return MotionToken::from_name(role).map(Timing::Token);
    }
    if let Some(args) = word
        .strip_prefix("-og-spring(")
        .and_then(|r| r.strip_suffix(')'))
    {
        let mut args = args.split(',').map(str::trim);
        let ms = time(args.next()?)?;
        let bounce = args.next().map_or(Some(0.0), |b| b.parse().ok())?;
        return args
            .next()
            .is_none()
            .then(|| Timing::Spring(SpringSpec::new(ms, bounce)));
    }
    Ease::from_css(word).map(Timing::Ease)
}

fn is_timing(word: &str) -> bool {
    timing(word).is_some()
        || word.starts_with("cubic-bezier(")
        || word.starts_with("steps(")
        || word.starts_with("linear(")
        || word.starts_with("-og-spring(")
}

fn declaration(name: &str, value: String, important: bool) -> Result<StyleDeclaration, String> {
    Ok(StyleDeclaration {
        name: name.into(),
        value: Tokens::parse(&value)?,
        source: value,
        important,
    })
}

/// Expands a transition or animation shorthand into its longhands; validates
/// longhands without variables. Other declarations pass through.
pub(crate) fn expand(
    d: StyleDeclaration,
    keyframes: &dyn Fn(&str) -> bool,
) -> Result<Vec<StyleDeclaration>, String> {
    let shorthand = match d.name.as_str() {
        "transition" => &TRANSITION[..],
        "animation" => &ANIMATION[..],
        "animation-range" | "scroll-timeline" | "view-timeline" => {
            return timeline_shorthand(&d);
        }
        name if longhand(name) => {
            if !d.value.has_variables() {
                validate(name, &d.source, keyframes)?;
            }
            return Ok(vec![d]);
        }
        _ => return Ok(vec![d]),
    };
    if d.source.replace("var(--motion-", "").contains("var(") {
        return Err(format!(
            "{}: write var() in a longhand ({}), not in the shorthand",
            d.name,
            shorthand.join(", ")
        ));
    }
    let mut lists: Vec<Vec<String>> = vec![Vec::new(); shorthand.len()];
    for item in items(&d.source) {
        let parts = if d.name == "transition" {
            transition_item(&item)?
        } else {
            animation_item(&item)?
        };
        for (list, part) in lists.iter_mut().zip(parts) {
            list.push(part);
        }
    }
    let mut out = Vec::new();
    for (name, list) in shorthand.iter().zip(lists) {
        let value = list.join(", ");
        validate(name, &value, keyframes)?;
        out.push(declaration(name, value, d.important)?);
    }
    if d.name == "animation" {
        // Reset-only longhands.
        for (name, value) in [
            ("animation-timeline", "auto"),
            ("animation-range-start", "normal"),
            ("animation-range-end", "normal"),
        ] {
            out.push(declaration(name, value.into(), d.important)?);
        }
    }
    Ok(out)
}

/// `animation-range` (`<start> <end>?`: an omitted end is the start's
/// range name at 100%, or `normal`), `scroll-timeline` and `view-timeline`
/// (`<name> <axis>?`) into their longhands.
fn timeline_shorthand(d: &StyleDeclaration) -> Result<Vec<StyleDeclaration>, String> {
    if d.source.contains("var(") {
        return Err(format!(
            "{}: write var() in a longhand, not in the shorthand",
            d.name
        ));
    }
    let named = |w: &str| crate::motion::scroll::RangeName::from_css(w).is_some();
    let mut lists: [Vec<String>; 2] = Default::default();
    for item in items(&d.source) {
        let words: Vec<&str> = item.iter().map(String::as_str).collect();
        let parts = if d.name == "animation-range" {
            // The start is a name and a length-percentage, or one word.
            let split = if words.len() >= 2 && named(words[0]) && !named(words[1]) {
                2
            } else {
                1
            };
            let (start, end) = words.split_at(split.min(words.len()));
            let end = match (end.is_empty(), start.first()) {
                (true, Some(w)) if named(w) => format!("{w} 100%"),
                (true, _) => "normal".into(),
                (false, _) => end.join(" "),
            };
            [start.join(" "), end]
        } else {
            match words[..] {
                [name] => [name.to_string(), "block".into()],
                [name, axis] => [name.to_string(), axis.to_string()],
                _ => {
                    return Err(format!(
                        "{}: write <name> <axis>?, a comma-separated list",
                        d.name
                    ));
                }
            }
        };
        for (list, part) in lists.iter_mut().zip(parts) {
            list.push(part);
        }
    }
    let names: [&str; 2] = match d.name.as_str() {
        "animation-range" => ["animation-range-start", "animation-range-end"],
        "scroll-timeline" => ["scroll-timeline-name", "scroll-timeline-axis"],
        _ => ["view-timeline-name", "view-timeline-axis"],
    };
    let mut out = Vec::new();
    for (name, list) in names.iter().zip(lists) {
        let value = list.join(", ");
        validate(name, &value, &|_| true)?;
        out.push(declaration(name, value, d.important)?);
    }
    Ok(out)
}

fn transition_item(words: &[String]) -> Result<Vec<String>, String> {
    let (mut property, mut timing_word, mut times, mut behavior) = (None, None, Vec::new(), None);
    for w in words {
        if matches!(w.as_str(), "normal" | "allow-discrete") && behavior.is_none() {
            behavior = Some(w.clone());
        } else if time(w).is_some() && times.len() < 2 {
            times.push(w.clone());
        } else if is_timing(w) && timing_word.is_none() {
            timing_word = Some(w.clone());
        } else if property.is_none() && !w.contains('(') {
            property = Some(w.clone());
        } else {
            return Err(format!(
                "transition: unexpected `{w}`; write <property> <duration> <timing-function> <delay> <behavior>"
            ));
        }
    }
    Ok(vec![
        property.unwrap_or_else(|| "all".into()),
        times.first().cloned().unwrap_or_else(|| "0s".into()),
        timing_word.unwrap_or_else(|| "ease".into()),
        times.get(1).cloned().unwrap_or_else(|| "0s".into()),
        behavior.unwrap_or_else(|| "normal".into()),
    ])
}

fn animation_item(words: &[String]) -> Result<Vec<String>, String> {
    let mut slots: [Option<String>; 8] = Default::default();
    let mut times = Vec::new();
    for w in words {
        let slot = if time(w).is_some() && times.len() < 2 {
            times.push(w.clone());
            continue;
        } else if is_timing(w) {
            2
        } else if w == "infinite" || w.parse::<f32>().is_ok() {
            4
        } else if matches!(
            w.as_str(),
            "normal" | "reverse" | "alternate" | "alternate-reverse"
        ) {
            5
        } else if matches!(w.as_str(), "forwards" | "backwards" | "both")
            || (w == "none" && slots[0].is_some())
        {
            6
        } else if matches!(w.as_str(), "running" | "paused") {
            7
        } else {
            0
        };
        if slots[slot].replace(w.clone()).is_some() {
            return Err(format!(
                "animation: `{w}` repeats a part; write <name> <duration> <timing-function> <delay> <iteration-count> <direction> <fill-mode> <play-state>"
            ));
        }
    }
    let defaults = ["none", "0s", "ease", "0s", "1", "normal", "none", "running"];
    slots[1] = times.first().cloned();
    slots[3] = times.get(1).cloned();
    Ok(slots
        .into_iter()
        .zip(defaults)
        .map(|(s, d)| s.unwrap_or_else(|| d.into()))
        .collect())
}

fn validate(name: &str, value: &str, keyframes: &dyn Fn(&str) -> bool) -> Result<(), String> {
    let list: Vec<String> = items(value).into_iter().map(|w| w.join(" ")).collect();
    for word in &list {
        let ok = match name {
            // Any property name: one that does not animate is skipped
            // (CSS Transitions 1 §2.1).
            "transition-property" => {
                (word == "none" && list.len() == 1)
                    || (word != "none"
                        && !matches!(word.as_str(), "initial" | "inherit" | "unset" | "default")
                        && super::syntax::identifier(word.trim_start_matches('-')))
            }
            "transition-behavior" => matches!(word.as_str(), "normal" | "allow-discrete"),
            "animation-composition" => Composite::from_css(word).is_some(),
            "transition-duration" | "animation-duration" => time(word).is_some_and(|t| t >= 0.0),
            "transition-delay" | "animation-delay" => time(word).is_some(),
            "transition-timing-function" | "animation-timing-function" => timing(word).is_some(),
            "animation-name" => {
                if word != "none" && !keyframes(word) {
                    return Err(format!(
                        "animation-name: declare @keyframes {word} {{ … }} in this document"
                    ));
                }
                true
            }
            "animation-iteration-count" => {
                word == "infinite" || word.parse::<f32>().is_ok_and(|n| n >= 0.0)
            }
            "animation-direction" => matches!(
                word.as_str(),
                "normal" | "reverse" | "alternate" | "alternate-reverse"
            ),
            "animation-fill-mode" => {
                matches!(word.as_str(), "none" | "forwards" | "backwards" | "both")
            }
            "animation-play-state" => matches!(word.as_str(), "running" | "paused"),
            "animation-timeline" => crate::motion::scroll::timeline_valid(word),
            "animation-range-start" | "animation-range-end" => {
                crate::motion::scroll::RangeEdge::valid(word)
            }
            "scroll-timeline-name" | "view-timeline-name" => {
                word == "none" || crate::motion::scroll::dashed(word)
            }
            "scroll-timeline-axis" | "view-timeline-axis" => {
                crate::motion::scroll::Axis::from_css(word).is_some()
            }
            "view-timeline-inset" => crate::motion::scroll::Inset::pair_valid(word),
            "timeline-scope" => {
                (matches!(word.as_str(), "none" | "all") && list.len() == 1)
                    || crate::motion::scroll::dashed(word)
            }
            _ => true,
        };
        if !ok {
            return Err(format!(
                "{name}: `{word}` is not valid here; {}",
                hint(name)
            ));
        }
    }
    if list.is_empty() || list.iter().any(String::is_empty) {
        return Err(format!("{name}: an empty list item; {}", hint(name)));
    }
    Ok(())
}

fn hint(name: &str) -> &'static str {
    match name {
        "transition-duration" | "animation-duration" => "write a non-negative time: 200ms or 0.2s",
        "transition-delay" | "animation-delay" => "write a time: 50ms or -0.1s",
        "transition-timing-function" | "animation-timing-function" => {
            "write a CSS easing (ease-out, cubic-bezier(…), steps(…), linear(…)), -og-spring(<ms>, <bounce>) or var(--motion-<role>)"
        }
        "animation-iteration-count" => "write a non-negative number or infinite",
        "animation-direction" => "write normal, reverse, alternate or alternate-reverse",
        "animation-fill-mode" => "write none, forwards, backwards or both",
        "animation-play-state" => "write running or paused",
        "transition-property" => "write none, all or property names",
        "transition-behavior" => "write normal or allow-discrete",
        "animation-composition" => "write replace, add or accumulate",
        "animation-timeline" => {
            "write auto, none, a --timeline name, scroll(<nearest|root|self> <block|inline|x|y>) or view(<axis> <inset>)"
        }
        "animation-range-start" | "animation-range-end" => {
            "write normal, a length-percentage, or cover, contain, entry, exit, entry-crossing or exit-crossing with an optional length-percentage"
        }
        "scroll-timeline-name" | "view-timeline-name" => "write none or a --name",
        "scroll-timeline-axis" | "view-timeline-axis" => "write block, inline, x or y",
        "view-timeline-inset" => "write one or two of auto and length-percentages",
        "timeline-scope" => "write none, all or --names",
        _ => "see the document CSS reference",
    }
}

/// The element's transitions and animations from its winning longhands
/// (name, substituted value). Animation names are qualified by `document`.
pub(crate) fn resolve(
    winners: &[(String, String)],
    document: &str,
    diagnostics: &mut Vec<String>,
) -> (Vec<CssTransition>, Vec<CssAnimation>) {
    let get = |name: &str, default: &str| -> Vec<String> {
        let value = winners
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map_or(default, |(_, v)| v.as_str());
        items(value).into_iter().map(|w| w.join(" ")).collect()
    };
    let mut valid = true;
    for (name, value) in winners {
        if let Err(error) = validate(name, value, &|_| true) {
            diagnostics.push(format!("{error} (invalid at computed-value time)"));
            valid = false;
        }
    }
    if !valid {
        return (Vec::new(), Vec::new());
    }
    let pick = |list: &[String], i: usize| list[i % list.len()].clone();
    let mut transitions = Vec::new();
    // Without a transition declaration the initial `all 0s` changes nothing;
    // it must not override motion the style or the element gives.
    let declared = winners.iter().any(|(n, _)| n.starts_with("transition-"));
    let properties = if declared {
        get("transition-property", "all")
    } else {
        Vec::new()
    };
    let (durations, timings, delays, behaviors) = (
        get("transition-duration", "0s"),
        get("transition-timing-function", "ease"),
        get("transition-delay", "0s"),
        get("transition-behavior", "normal"),
    );
    for (i, property) in properties.iter().enumerate() {
        if property == "none" {
            continue;
        }
        let duration_ms = time(&pick(&durations, i)).unwrap_or(0.0);
        let transition = match timing(&pick(&timings, i)).expect("validated timing") {
            Timing::Token(t) => Transition::Token(t),
            Timing::Spring(s) => Transition::Spring(s),
            Timing::Ease(_) if duration_ms <= 0.0 => Transition::Instant,
            Timing::Ease(ease) => Transition::Tween { duration_ms, ease },
        };
        transitions.push(CssTransition {
            property: property.clone(),
            channels: channels(property).unwrap_or(0),
            transition,
            delay_ms: time(&pick(&delays, i)).unwrap_or(0.0),
            discrete: pick(&behaviors, i) == "allow-discrete",
        });
    }
    let mut animations = Vec::new();
    let names = get("animation-name", "none");
    let lists = [
        get("animation-duration", "0s"),
        get("animation-timing-function", "ease"),
        get("animation-delay", "0s"),
        get("animation-iteration-count", "1"),
        get("animation-direction", "normal"),
        get("animation-fill-mode", "none"),
        get("animation-play-state", "running"),
        get("animation-composition", "replace"),
        get("animation-timeline", "auto"),
        get("animation-range-start", "normal"),
        get("animation-range-end", "normal"),
    ];
    for (i, name) in names.iter().enumerate() {
        if name == "none" {
            continue;
        }
        let at = |k: usize| pick(&lists[k], i);
        let ease = match timing(&at(1)).expect("validated timing") {
            Timing::Ease(e) => e,
            _ => {
                diagnostics.push(format!(
                    "animation-timing-function of {name}: springs and motion tokens time transitions; give an animation a CSS easing"
                ));
                continue;
            }
        };
        let fill = at(5);
        animations.push(CssAnimation {
            name: format!("{document}/{name}"),
            duration_ms: time(&at(0)).unwrap_or(0.0),
            ease,
            delay_ms: time(&at(2)).unwrap_or(0.0),
            iterations: if at(3) == "infinite" {
                f32::INFINITY
            } else {
                at(3).parse().unwrap_or(1.0)
            },
            direction: match at(4).as_str() {
                "reverse" => Direction::Reverse,
                "alternate" => Direction::Alternate,
                "alternate-reverse" => Direction::AlternateReverse,
                _ => Direction::Normal,
            },
            fill_backwards: matches!(fill.as_str(), "backwards" | "both"),
            fill_forwards: matches!(fill.as_str(), "forwards" | "both"),
            paused: at(6) == "paused",
            composite: Composite::from_css(&at(7)).unwrap_or_default(),
            timeline: AnimationTimeline::parse(&at(8)).unwrap_or_else(|| {
                diagnostics.push(format!(
                    "animation-timeline of {name}: `{}` takes px or % insets after the element's units",
                    at(8)
                ));
                AnimationTimeline::None
            }),
            range_start: RangeEdge::parse(&at(9), false).flatten(),
            range_end: RangeEdge::parse(&at(10), true).flatten(),
        });
    }
    (transitions, animations)
}

/// The scroll and view timelines an element declares and its
/// `timeline-scope`, from its winning longhands (lengths already px).
pub(crate) fn timelines(winners: &[(String, String)]) -> (Vec<NamedTimeline>, Vec<String>) {
    let get = |name: &str, default: &str| -> Vec<String> {
        let value = winners
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map_or(default, |(_, v)| v.as_str());
        items(value).into_iter().map(|w| w.join(" ")).collect()
    };
    let pick = |list: &[String], i: usize| list[i % list.len().max(1)].clone();
    let mut out = Vec::new();
    for view in [false, true] {
        let kind = if view { "view" } else { "scroll" };
        let axes = get(&format!("{kind}-timeline-axis"), "block");
        let insets = get("view-timeline-inset", "auto");
        for (i, name) in get(&format!("{kind}-timeline-name"), "none")
            .iter()
            .enumerate()
        {
            if !crate::motion::scroll::dashed(name) {
                continue;
            }
            out.push(NamedTimeline {
                name: name.clone(),
                view,
                axis: Axis::from_css(&pick(&axes, i)).unwrap_or_default(),
                inset: if view {
                    Inset::pair(&pick(&insets, i)).unwrap_or_default()
                } else {
                    Default::default()
                },
            });
        }
    }
    let scope = get("timeline-scope", "none")
        .into_iter()
        .filter(|w| w != "none")
        .collect();
    (out, scope)
}

/// One `@keyframes` body: `from`, `to` and percentage selector lists whose
/// declarations animate any animatable property (those the element's
/// channels hold by channel, the rest by computed value);
/// `animation-timing-function` times the segment that starts at its
/// keyframe and `animation-composition` combines it with the underlying
/// value; `-og-sound` plays a style sound when the iteration reaches it.
pub(crate) fn keyframes(frames: &[(String, String)]) -> Result<Keyframes, String> {
    use crate::element::Prop;
    let mut out = Keyframes::default();
    for (selector, body) in frames {
        let mut offsets = Vec::new();
        for s in selector.split(',').map(str::trim) {
            let offset = match s {
                "from" => Some(0.0),
                "to" => Some(1.0),
                other => other
                    .strip_suffix('%')
                    .and_then(|n| n.trim().parse::<f32>().ok())
                    .map(|n| n / 100.0),
            }
            .filter(|n| n.is_finite() && (0.0..=1.0).contains(n))
            .ok_or_else(|| {
                format!("keyframe selector `{s}`: write from, to or a percentage from 0% to 100%")
            })?;
            offsets.push(offset);
        }
        let mut ease = None;
        let mut composite = None;
        let mut values = Vec::new();
        for (name, value) in super::css::split_decls(body)? {
            match name.as_str() {
                "-og-burst" => {
                    // `-og-burst: <count> [<radius>px] [<colour>]`: rays and
                    // sparks from the element when the iteration reaches it.
                    let mut count = None;
                    let mut radius = 0.0;
                    let mut color = None;
                    for w in value.split_whitespace() {
                        if let Ok(n) = w.parse::<u32>()
                            && count.is_none()
                        {
                            count = Some(n);
                        } else if let Some(px) =
                            w.strip_suffix("px").and_then(|n| n.parse::<f32>().ok())
                        {
                            radius = px;
                        } else {
                            let c = crate::svg::color(w).map_err(|_| {
                                format!("-og-burst: `{w}` is a count, a radius in px or a colour")
                            })?;
                            let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                            color = Some(crate::style::Srgba {
                                rgb: [b(c[0]), b(c[1]), b(c[2])],
                                alpha: c[3],
                            });
                        }
                    }
                    let count =
                        count.ok_or("-og-burst takes a count: `-og-burst: 24 48px #ffd166`")?;
                    for &offset in &offsets {
                        out.events.push((
                            offset,
                            crate::motion::EventKind::Burst {
                                count,
                                radius,
                                color,
                            },
                        ));
                    }
                }
                "-og-sound" => {
                    if !super::syntax::identifier(&value) {
                        return Err("-og-sound names a style sound, for example error".into());
                    }
                    for &offset in &offsets {
                        out.events.push((
                            offset,
                            crate::motion::EventKind::Sound {
                                name: value.clone(),
                                pitch: 1.0,
                            },
                        ));
                    }
                }
                "animation-composition" => {
                    composite = Some(Composite::from_css(&value).ok_or_else(|| {
                        format!(
                            "animation-composition: `{value}`; {}",
                            hint("animation-composition")
                        )
                    })?);
                }
                "animation-timing-function" => {
                    ease = Some(Ease::from_css(&value).ok_or_else(|| {
                        format!(
                            "animation-timing-function: `{value}` in a keyframe takes a CSS easing"
                        )
                    })?);
                }
                _ => values.push((name, value)),
            }
        }
        for (name, text) in values {
            // A property the channels hold, with numbers: channel tracks.
            let scalar: Option<Vec<(Prop, f32)>> = channels(&name)
                .filter(|_| !text.contains("var("))
                .and_then(|_| super::css::declarations(&format!("{name}: {text};"), 0).ok())
                .and_then(|entries| {
                    entries
                        .into_iter()
                        .map(|entry| {
                            let prop = Prop::from_name(&entry.key)?;
                            let value = entry
                                .values
                                .first()
                                .and_then(og_core::ogt::Value::as_number)
                                .map(og_core::ogt::Number::as_f32)
                                .filter(|v| v.is_finite())?;
                            Some((prop, value))
                        })
                        .collect()
                });
            let Some(scalar) = scalar else {
                if !interpolate::animatable(&name) || name.starts_with("--") {
                    return Err(format!("{name} cannot be animated in @keyframes"));
                }
                if !text.contains("var(") && !super::build::cascade::known(&name, &text) {
                    return Err(format!(
                        "@keyframes: `{name}: {text}` is not a declaration document CSS takes"
                    ));
                }
                let index = out
                    .styles
                    .iter()
                    .position(|t| t.property == name)
                    .unwrap_or_else(|| {
                        out.styles.push(StyleTrack {
                            property: name.clone(),
                            keys: Vec::new(),
                        });
                        out.styles.len() - 1
                    });
                for &offset in &offsets {
                    let keys = &mut out.styles[index].keys;
                    keys.retain(|k| k.offset != offset);
                    keys.push(StyleKey {
                        offset,
                        value: text.clone(),
                        ease,
                        composite,
                    });
                }
                continue;
            };
            for (prop, value) in scalar {
                let index = out
                    .tracks
                    .iter()
                    .position(|t| t.prop == prop)
                    .unwrap_or_else(|| {
                        out.tracks.push(KeyframeTrack {
                            prop,
                            keys: Vec::new(),
                        });
                        out.tracks.len() - 1
                    });
                for &offset in &offsets {
                    let keys = &mut out.tracks[index].keys;
                    keys.retain(|k| k.offset != offset);
                    keys.push(Keyframe {
                        offset,
                        value,
                        ease,
                        composite,
                    });
                }
            }
        }
    }
    for track in &mut out.tracks {
        track.keys.sort_by(|a, b| a.offset.total_cmp(&b.offset));
    }
    for track in &mut out.styles {
        track.keys.sort_by(|a, b| a.offset.total_cmp(&b.offset));
    }
    if out.tracks.is_empty() && out.styles.is_empty() && out.events.is_empty() {
        return Err("@keyframes must animate a property or play a -og-sound".into());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decl(name: &str, value: &str) -> StyleDeclaration {
        super::declaration(name, value.into(), false).unwrap()
    }

    fn resolved(decls: &[(&str, &str)]) -> (Vec<CssTransition>, Vec<CssAnimation>, Vec<String>) {
        let known = |n: &str| matches!(n, "spin" | "pulse");
        let mut winners = Vec::new();
        for (name, value) in decls {
            for d in expand(decl(name, value), &known).unwrap() {
                winners.retain(|(n, _): &(String, String)| *n != d.name);
                winners.push((d.name, d.source));
            }
        }
        let mut diagnostics = Vec::new();
        let (t, a) = resolve(&winners, "doc", &mut diagnostics);
        (t, a, diagnostics)
    }

    #[test]
    fn shorthand_lists_expand_and_longhands_override_by_position() {
        let (t, _, d) = resolved(&[
            (
                "transition",
                "opacity 200ms ease-in 50ms, scale 0.3s steps(3), translate 1s",
            ),
            ("transition-duration", "120ms, 0s"),
        ]);
        assert!(d.is_empty(), "{d:?}");
        // The duration list repeats: opacity 120ms, scale 0s (instant), translate 120ms.
        assert_eq!(t.len(), 3);
        assert_eq!(t[1].transition, Transition::Instant);
        assert_eq!(t[0].channels, 1 << 7);
        assert_eq!(t[0].delay_ms, 50.0);
        assert!(
            matches!(t[0].transition, Transition::Tween { duration_ms, .. } if duration_ms == 120.0)
        );
        assert_eq!(t[2].channels, 0b11);
        assert_eq!(t[2].delay_ms, 0.0);
        assert!(
            resolved(&[("animation", "spin 1s")]).0.is_empty(),
            "no transition declared"
        );
        let (all, _, _) = resolved(&[("transition", "var(--motion-ui)")]);
        assert_eq!(all[0].channels, ALL_CHANNELS);
        assert!(matches!(
            all[0].transition,
            Transition::Token(MotionToken::Ui)
        ));
    }

    #[test]
    fn animation_shorthand_assigns_parts_by_kind() {
        let (_, a, d) = resolved(&[(
            "animation",
            "spin 2s linear infinite alternate-reverse both paused 100ms, pulse 300ms 2.5",
        )]);
        assert!(d.is_empty(), "{d:?}");
        assert_eq!(a.len(), 2);
        assert_eq!(a[0].name, "doc/spin");
        assert_eq!((a[0].duration_ms, a[0].delay_ms), (2000.0, 100.0));
        assert!(a[0].iterations.is_infinite() && a[0].paused);
        assert_eq!(a[0].direction, Direction::AlternateReverse);
        assert!(a[0].fill_backwards && a[0].fill_forwards);
        assert_eq!((a[1].iterations, a[1].fill_forwards), (2.5, false));
        assert_eq!(a[1].ease, Ease::from_css("ease").unwrap());
    }

    #[test]
    fn out_of_scope_motion_fails_with_the_property_named() {
        let known = |n: &str| n == "spin";
        for (name, value, needle) in [
            (
                "transition-property",
                "none, opacity",
                "transition-property",
            ),
            ("transition", "opacity var(--d)", "longhand"),
            ("animation", "fade 1s", "@keyframes fade"),
            ("animation-direction", "sideways", "animation-direction"),
            (
                "animation-iteration-count",
                "-1",
                "animation-iteration-count",
            ),
            (
                "transition-timing-function",
                "bounce",
                "transition-timing-function",
            ),
        ] {
            let error = expand(decl(name, value), &known).unwrap_err();
            assert!(error.contains(needle), "{name}: {value} → {error}");
        }
    }
}
