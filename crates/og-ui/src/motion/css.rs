//! CSS transitions and animations on presentation ticks (docs/ui.md §3.2).
//! Transitions pick a timing per animated channel; animations sample their
//! `@keyframes` from the number of ticks they have been running, so a
//! restore lands on the same value and nothing reads a clock. Runs live in
//! the motion state, outside `world_hash`.

use super::interpolate::{self, Composite};
use super::spring::{Ease, Transition};
use super::timeline::EventKind;
use crate::element::{CHANNELS, Prop};

/// One `transition` entry: the property it times (`all` for every one),
/// how, and after what delay. A property the element's channels hold
/// moves its channels; any other moves its computed value
/// ([`StyleRun`]).
#[derive(Clone, Debug, PartialEq)]
pub struct CssTransition {
    pub property: String,
    /// Bit `c`: channel `c` ([`Prop::channels`]).
    pub channels: u32,
    pub transition: Transition,
    pub delay_ms: f32,
    /// `transition-behavior: allow-discrete`: discrete values transition,
    /// flipping halfway.
    pub discrete: bool,
}

impl CssTransition {
    /// Whether it times `property`'s computed value.
    pub fn covers(&self, property: &str) -> bool {
        self.property == property
            || self.property == "all"
            || crate::css::boxes::longhands(&self.property)
                .iter()
                .any(|p| crate::css::values::css_name(p) == Some(property))
    }
}

/// The channels of every transitionable property (`all`).
pub const ALL_CHANNELS: u32 = (1 << CHANNELS) - 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Direction {
    #[default]
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}

impl Direction {
    /// Whether iteration `k` (0-based) runs from 100% to 0%.
    fn reversed(self, k: u64) -> bool {
        match self {
            Direction::Normal => false,
            Direction::Reverse => true,
            Direction::Alternate => k % 2 == 1,
            Direction::AlternateReverse => k % 2 == 0,
        }
    }
}

/// One `animation` entry, with `name` qualified by its document
/// (`<document>/<keyframes>`).
#[derive(Clone, Debug, PartialEq)]
pub struct CssAnimation {
    pub name: String,
    pub duration_ms: f32,
    pub ease: Ease,
    pub delay_ms: f32,
    /// `f32::INFINITY` for `infinite`.
    pub iterations: f32,
    pub direction: Direction,
    pub fill_backwards: bool,
    pub fill_forwards: bool,
    pub paused: bool,
    /// `animation-composition`.
    pub composite: Composite,
    /// `animation-timeline`.
    pub timeline: super::scroll::AnimationTimeline,
    /// `animation-range-start` and `-end` (`None`: `normal`).
    pub range_start: Option<super::scroll::RangeEdge>,
    pub range_end: Option<super::scroll::RangeEdge>,
}

impl Default for CssAnimation {
    fn default() -> Self {
        Self {
            name: String::new(),
            duration_ms: 0.0,
            ease: Ease::from_css("ease").expect("CSS ease"),
            delay_ms: 0.0,
            iterations: 1.0,
            direction: Direction::Normal,
            fill_backwards: false,
            fill_forwards: false,
            paused: false,
            composite: Composite::Replace,
            timeline: super::scroll::AnimationTimeline::Document,
            range_start: None,
            range_end: None,
        }
    }
}

/// A keyframe of one property: its offset (0–1), value, and the timing
/// function from it to the next keyframe (`None`: the animation's).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Keyframe {
    pub offset: f32,
    pub value: f32,
    pub ease: Option<Ease>,
    /// Its `animation-composition` (`None`: the animation's).
    pub composite: Option<Composite>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct KeyframeTrack {
    pub prop: Prop,
    /// By offset. A missing 0% or 100% keyframe takes the element's value.
    pub keys: Vec<Keyframe>,
}

/// A keyframe of a property animated by its computed value: the
/// declaration's text, substituted and computed for each element.
#[derive(Clone, Debug, PartialEq)]
pub struct StyleKey {
    pub offset: f32,
    pub value: String,
    pub ease: Option<Ease>,
    pub composite: Option<Composite>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StyleTrack {
    pub property: String,
    /// By offset; a missing 0% or 100% keyframe takes the underlying value.
    pub keys: Vec<StyleKey>,
}

/// A parsed `@keyframes` rule.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Keyframes {
    /// Properties the element's channels hold.
    pub tracks: Vec<KeyframeTrack>,
    /// Every other animated property.
    pub styles: Vec<StyleTrack>,
    /// Events at offsets (`-og-sound`), in each iteration.
    pub events: Vec<(f32, EventKind)>,
}

/// A running CSS animation of an element. Its time is the ticks it has run
/// (paused ticks do not count).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CssRun {
    pub element: u64,
    /// Its place in the element's `animation` list.
    pub index: u32,
    pub name: String,
    pub ticks: u64,
    /// The last tick it was stepped.
    pub stepped: u64,
    /// The active time events were fired up to, ms (negative: in its delay).
    pub last_ms: f32,
}

/// Where an animation is at `ticks`: per channel, its value; and the
/// iteration-local times it passed since `last_ms`, for events.
pub(crate) struct Sample {
    pub values: Vec<(usize, f32)>,
    pub active_ms: f32,
}

impl CssAnimation {
    fn active_total(&self) -> f32 {
        if self.iterations.is_infinite() {
            f32::INFINITY
        } else {
            self.duration_ms * self.iterations
        }
    }

    /// Whether it has stopped changing at `ticks` of running time.
    pub(crate) fn settled(&self, ticks: u64, tps: u32, reduced: bool) -> bool {
        let elapsed = ticks as f32 * 1000.0 / tps.max(1) as f32;
        self.paused || reduced || elapsed - self.delay_ms >= self.active_total()
    }

    /// Its active time after `ticks` of running time.
    pub(crate) fn active_ms(&self, ticks: u64, tps: u32) -> f32 {
        ticks as f32 * 1000.0 / tps.max(1) as f32 - self.delay_ms
    }

    /// The computed values of `keyframes`' style tracks at active time
    /// `active_ms`, each over `underlying` (the property's value below
    /// this animation, `None` when it has none), with `compute` turning a
    /// keyframe's declaration into the element's computed text.
    pub(crate) fn sample_styles(
        &self,
        keyframes: &Keyframes,
        active_ms: f32,
        reduced: bool,
        underlying: &dyn Fn(&str) -> Option<String>,
        compute: &dyn Fn(&str, &str) -> Option<String>,
    ) -> Vec<(String, String)> {
        let Some(t) = self.local_ms(active_ms, reduced) else {
            return Vec::new();
        };
        let d = self.duration_ms;
        let mut out = Vec::new();
        for track in &keyframes.styles {
            let property = track.property.as_str();
            let under = underlying(property);
            let mut keys: Vec<(f32, String, Option<Ease>)> = Vec::new();
            if let Some(u) = &under
                && track.keys.first().is_none_or(|k| k.offset > 0.0)
            {
                keys.push((0.0, u.clone(), None));
            }
            for k in &track.keys {
                let Some(v) = compute(property, &k.value) else {
                    continue;
                };
                let v = match (&under, k.composite.unwrap_or(self.composite)) {
                    (Some(u), op) => interpolate::compose(property, u, &v, op),
                    (None, _) => v,
                };
                keys.push((k.offset * d, v, k.ease));
            }
            if let Some(u) = &under
                && track.keys.last().is_none_or(|k| k.offset < 1.0)
            {
                keys.push((d, u.clone(), None));
            }
            let Some(first) = keys.first() else {
                continue;
            };
            let mut value = first.1.clone();
            if t > first.0 {
                value = keys[keys.len() - 1].1.clone();
                for w in keys.windows(2) {
                    let ((a_ms, a, ease), (b_ms, b, _)) = (&w[0], &w[1]);
                    if t <= *b_ms {
                        let span = (b_ms - a_ms).max(0.0);
                        let k = if span > 0.0 {
                            ((t - a_ms) / span).clamp(0.0, 1.0)
                        } else {
                            1.0
                        };
                        value =
                            interpolate::mix(property, a, b, ease.unwrap_or(self.ease).apply(k));
                        break;
                    }
                }
            }
            out.push((track.property.clone(), value));
        }
        out
    }

    /// The keyframe progress in ms (0..=duration) at active time `t`, or
    /// `None` when the animation has no effect then.
    fn local_ms(&self, active: f32, reduced: bool) -> Option<f32> {
        let d = self.duration_ms;
        let total = self.active_total();
        let at_end = |s: &Self| {
            // The end of the last iteration, as its direction leaves it.
            let iterations = s.iterations;
            let whole = iterations.floor();
            let (k, frac) = if iterations > 0.0 && iterations == whole {
                (whole as u64 - 1, 1.0)
            } else {
                (whole as u64, iterations - whole)
            };
            let t = frac * d;
            Some(if s.direction.reversed(k) { d - t } else { t })
        };
        if reduced {
            // Reduced motion: a finite animation is at its end at once; an
            // endless one (decorative) does not play.
            return (total.is_finite() && self.fill_forwards)
                .then(|| at_end(self))
                .flatten();
        }
        if active < 0.0 {
            return self
                .fill_backwards
                .then(|| if self.direction.reversed(0) { d } else { 0.0 });
        }
        if active >= total || d <= 0.0 {
            return if self.fill_forwards {
                at_end(self)
            } else {
                None
            };
        }
        let k = (active / d).floor();
        let t = active - k * d;
        Some(if self.direction.reversed(k as u64) {
            d - t
        } else {
            t
        })
    }

    /// The channel values of `keyframes` at `ticks` of running time, over
    /// the element's own values `base`.
    pub(crate) fn sample(
        &self,
        keyframes: &Keyframes,
        ticks: u64,
        tps: u32,
        base: &[f32; CHANNELS],
        reduced: bool,
    ) -> Sample {
        self.sample_active(keyframes, self.active_ms(ticks, tps), base, reduced)
    }

    /// The channel values of `keyframes` at active time `active_ms`.
    pub(crate) fn sample_active(
        &self,
        keyframes: &Keyframes,
        active_ms: f32,
        base: &[f32; CHANNELS],
        reduced: bool,
    ) -> Sample {
        let mut values = Vec::new();
        if let Some(t) = self.local_ms(active_ms, reduced) {
            for track in &keyframes.tracks {
                for &c in track.prop.channels() {
                    values.push((c, self.track_at(track, t, base[c])));
                }
            }
        }
        Sample { values, active_ms }
    }

    fn track_at(&self, track: &KeyframeTrack, t: f32, base: f32) -> f32 {
        let scales = matches!(track.prop, Prop::Scale | Prop::ScaleX | Prop::ScaleY);
        let d = self.duration_ms;
        // Keys in ms, with implicit 0% and 100% keyframes at the element's value.
        let mut keys: Vec<(f32, f32, Option<Ease>)> = Vec::with_capacity(track.keys.len() + 2);
        if track.keys.first().is_none_or(|k| k.offset > 0.0) {
            keys.push((0.0, base, None));
        }
        let composite = |k: &Keyframe| match k.composite.unwrap_or(self.composite) {
            Composite::Replace => k.value,
            // Scales multiply when added and accumulate one-based.
            Composite::Add if scales => base * k.value,
            Composite::Accumulate if scales => base + k.value - 1.0,
            _ => base + k.value,
        };
        keys.extend(
            track
                .keys
                .iter()
                .map(|k| (k.offset * d, composite(k), k.ease)),
        );
        if track.keys.last().is_none_or(|k| k.offset < 1.0) {
            keys.push((d, base, None));
        }
        if t <= keys[0].0 {
            return keys[0].1;
        }
        for w in keys.windows(2) {
            let ((a_ms, a, ease), (b_ms, b, _)) = (w[0], w[1]);
            if t <= b_ms {
                let span = (b_ms - a_ms).max(0.0);
                let k = if span > 0.0 {
                    ((t - a_ms) / span).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                return a + (b - a) * ease.unwrap_or(self.ease).apply(k);
            }
        }
        keys[keys.len() - 1].1
    }

    /// The events of `keyframes` passed in (`from`, `to`] of active time.
    pub(crate) fn passed<'a>(
        &self,
        keyframes: &'a Keyframes,
        from: f32,
        to: f32,
    ) -> Vec<&'a EventKind> {
        let d = self.duration_ms;
        let total = self.active_total();
        let mut out = Vec::new();
        if keyframes.events.is_empty() || to <= from || d <= 0.0 {
            return out;
        }
        let first = (from.max(0.0) / d).floor() as u64;
        let last = (to.min(total) / d).floor() as u64;
        // A long jump (a restore far ahead) fires at most a few iterations.
        for k in first.max(last.saturating_sub(8))..=last {
            for (offset, kind) in &keyframes.events {
                let local = offset * d;
                let at = k as f32 * d
                    + if self.direction.reversed(k) {
                        d - local
                    } else {
                        local
                    };
                if at > from && at <= to && at <= total {
                    out.push(kind);
                }
            }
        }
        out
    }
}

/// The transition timing for channel `c`: the last entry naming it.
pub(crate) fn transition_for(list: &[CssTransition], c: usize) -> Option<&CssTransition> {
    list.iter().rev().find(|t| t.channels >> c & 1 == 1)
}

/// A running transition of a computed value (CSS Transitions 1 §3), in
/// ticks so a restore lands on the same value.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StyleRun {
    pub property: String,
    pub from: String,
    pub to: String,
    /// The tick it started (its delay counts from here).
    pub start: u64,
    pub delay_ms: f32,
    pub duration_ms: f32,
    /// Its reversing-adjusted start value and shortening factor.
    pub reversing_from: String,
    pub factor: f32,
}

/// An element's transitions of computed values: the after-change values of
/// the properties they cover, and those running.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StyleState {
    pub after: std::collections::BTreeMap<String, String>,
    pub runs: Vec<StyleRun>,
    #[serde(default)]
    pub animations: Vec<CssRun>,
    /// Finished animation runs remain for fill and replay, but need no repaint.
    #[serde(default)]
    pub running: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opacity(keys: &[(f32, f32)]) -> Keyframes {
        Keyframes {
            tracks: vec![KeyframeTrack {
                prop: Prop::Opacity,
                keys: keys
                    .iter()
                    .map(|&(offset, value)| Keyframe {
                        offset,
                        value,
                        ease: None,
                        composite: None,
                    })
                    .collect(),
            }],
            styles: Vec::new(),
            events: Vec::new(),
        }
    }

    fn at(a: &CssAnimation, k: &Keyframes, ms: u64, reduced: bool) -> Option<f32> {
        let mut base = [0.0; CHANNELS];
        base[7] = 0.9;
        // 1000 ticks a second: one tick per ms.
        a.sample(k, ms, 1000, &base, reduced)
            .values
            .first()
            .map(|&(_, v)| v)
    }

    #[test]
    fn iterations_direction_delay_and_fill_follow_css() {
        let linear = Ease::from_css("linear").unwrap();
        let keys = opacity(&[(0.0, 0.0), (1.0, 1.0)]);
        let a = CssAnimation {
            duration_ms: 100.0,
            ease: linear,
            delay_ms: 50.0,
            iterations: 2.5,
            direction: Direction::Alternate,
            ..Default::default()
        };
        assert_eq!(
            at(&a, &keys, 10, false),
            None,
            "delay without backwards fill"
        );
        assert_eq!(at(&a, &keys, 75, false), Some(0.25));
        assert_eq!(
            at(&a, &keys, 175, false),
            Some(0.75),
            "second iteration runs back"
        );
        assert_eq!(at(&a, &keys, 275, false), Some(0.25));
        assert_eq!(
            at(&a, &keys, 400, false),
            None,
            "ended without forwards fill"
        );
        let both = CssAnimation {
            fill_backwards: true,
            fill_forwards: true,
            direction: Direction::AlternateReverse,
            ..a.clone()
        };
        assert_eq!(
            at(&both, &keys, 10, false),
            Some(1.0),
            "backwards fill of a reversed start"
        );
        // 2.5 iterations alternate-reverse end halfway through a reversed third.
        assert_eq!(at(&both, &keys, 400, false), Some(0.5));
        assert_eq!(
            at(&both, &keys, 0, true),
            Some(0.5),
            "reduced: at its end at once"
        );
        let endless = CssAnimation {
            iterations: f32::INFINITY,
            fill_forwards: true,
            ..a
        };
        assert_eq!(at(&endless, &keys, 100_050, false), Some(0.0));
        assert_eq!(at(&endless, &keys, 100, true), None, "reduced: loops stop");
    }

    #[test]
    fn missing_ends_take_the_element_value_and_keyframes_carry_their_timing() {
        let linear = Ease::from_css("linear").unwrap();
        let mut keys = opacity(&[(0.5, 0.1)]);
        keys.tracks[0].keys[0].ease = Some(Ease::from_css("step-end").unwrap());
        let a = CssAnimation {
            duration_ms: 200.0,
            ease: linear,
            ..Default::default()
        };
        assert!(
            (at(&a, &keys, 50, false).unwrap() - 0.5).abs() < 1e-6,
            "0.9 → 0.1 linearly"
        );
        // From 50% the keyframe's own step-end holds until 100%.
        assert_eq!(at(&a, &keys, 150, false), Some(0.1));
        assert_eq!(at(&a, &keys, 199, false), Some(0.1));
    }

    #[test]
    fn events_fire_once_per_iteration_in_the_direction_run() {
        let mut keys = opacity(&[(0.0, 0.0), (1.0, 1.0)]);
        keys.events.push((0.25, EventKind::Callback("tick".into())));
        let a = CssAnimation {
            duration_ms: 100.0,
            iterations: 3.0,
            direction: Direction::Alternate,
            ..Default::default()
        };
        let mut fired = Vec::new();
        let mut last = -1e-3;
        for ms in 0..400 {
            let now = ms as f32;
            fired.extend(a.passed(&keys, last, now).into_iter().map(|_| ms));
            last = now;
        }
        // 25 ms, then 75 ms into the reversed second, then 25 ms into the third.
        assert_eq!(fired, [25, 175, 225]);
    }
}
