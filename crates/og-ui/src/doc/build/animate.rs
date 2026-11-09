//! Transitions and animations of computed values (docs/ui.md §3.2). The
//! properties the element's channels hold move in the motion step; every
//! other animatable property moves here, over the cascade's winning values
//! before they are routed, so a width, a shadow, a filter or a gradient
//! animates through the same parsing as a declaration. An element with one
//! running builds again each frame; an idle one is left alone. Transition
//! runs are counted in ticks and kept in the motion state, so a restore
//! lands on the same value.
use super::*;
use crate::motion::Transition;
use crate::motion::css::{CssAnimation, CssTransition, StyleRun, StyleState};
use crate::motion::interpolate;

/// What a resolve leaves the motion state, applied as the element builds.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct StyleCommit {
    pub key: u64,
    /// `None`: it keeps no transition state.
    pub state: Option<StyleState>,
    /// A transition or animation still moves a value: build it next frame.
    pub running: bool,
}

/// A transition's progress (eased, may overshoot) `ms` after its delay,
/// and whether it has finished.
fn progress(
    t: &Transition,
    ms: f32,
    run: &StyleRun,
    tps: u32,
    tokens: &crate::motion::MotionTokens,
) -> (f32, bool) {
    match *t {
        Transition::Token(token) => progress(&tokens.get(token), ms, run, tps, tokens),
        Transition::Spring(spec) => {
            // From 0 to 1 by the tick, as a channel's spring moves.
            let spring = spec.to_tween();
            let dt = 1.0 / tps.max(1) as f32;
            let mut s = og_tween::SpringState::default();
            let ticks = (ms * tps as f32 / 1000.0).round().max(0.0) as u32;
            for _ in 0..ticks {
                s = spring.step(s, 1.0, dt);
                if og_tween::Spring::settled(s, 1.0) {
                    return (1.0, true);
                }
            }
            (s.value, false)
        }
        Transition::Tween { ease, .. } => {
            let d = run.duration_ms;
            if d <= 0.0 || ms >= d {
                (1.0, true)
            } else {
                (ease.apply((ms / d).max(0.0)), false)
            }
        }
        Transition::Instant => (1.0, true),
    }
}

impl Ui {
    /// Moves the computed values `cascade` won by the element's
    /// transitions and animations, returning what the motion state keeps.
    pub(super) fn animate_values(
        &self,
        key: u64,
        cascade: &mut cascade::Cascade,
        starting: &[(String, String, usize)],
        transitions: &[CssTransition],
        animations: &[CssAnimation],
        basis: crate::doc::calc::Basis,
    ) -> Option<Arc<StyleCommit>> {
        let previous = self.state.motion.styles.get(&key);
        let styled = |name: &str| {
            crate::doc::motion::channels(name).is_none()
                && interpolate::animatable(name)
                && !name.starts_with("--")
        };
        let covered = |name: &str| transitions.iter().rev().find(|t| t.covers(name));
        let has_styles = animations.iter().any(|a| {
            self.docs
                .keyframes
                .get(&a.name)
                .is_some_and(|k| !k.styles.is_empty())
        });
        let transitioned = transitions
            .iter()
            .any(|t| t.channels == 0 || t.property == "all");
        // The before-change style: last build's after-change values, or on
        // the first, `@starting-style`'s.
        let first =
            (previous.is_none() && starting.iter().any(|(n, ..)| styled(n))).then(|| StyleState {
                after: starting
                    .iter()
                    .filter(|(n, ..)| styled(n))
                    .map(|(n, v, _)| (n.clone(), v.clone()))
                    .collect(),
                runs: Vec::new(),
            });
        let previous = previous.or(first.as_ref());
        if previous.is_none() && !has_styles && !transitioned {
            return None;
        }
        let now = self.state.tick;
        let tps = self.ticks_per_second;
        let reduced = self.theme.reduced_motion;
        let get = |values: &[(String, String, usize)], name: &str| {
            values
                .iter()
                .rev()
                .find(|(n, ..)| n == name)
                .map(|(_, v, _)| v.clone())
        };
        let set =
            |values: &mut Vec<(String, String, usize)>, name: &str, value: String| match values
                .iter_mut()
                .rev()
                .find(|(n, ..)| n == name)
            {
                Some(entry) => entry.1 = value,
                None => values.push((name.to_string(), value, 0)),
            };
        let value_of = |values: &[(String, String, usize)], name: &str| {
            get(values, name).or_else(|| interpolate::initial(name).map(str::to_string))
        };
        let mut running = false;

        // CSS animations, in list order over the cascade (later win).
        let theme = self.theme();
        let computed = cascade.computed.clone();
        let compute = |name: &str, text: &str| -> Option<String> {
            let v = crate::doc::variables::Value::parse(text)
                .ok()?
                .substitute(&mut |n| match computed.get(n) {
                    Some(v) => v.clone(),
                    None => cascade::token(theme, n),
                })
                .ok()?;
            let v = if crate::doc::calc::per_element(name, &v) {
                crate::doc::calc::absolute(&v, basis, name).0
            } else {
                v
            };
            if crate::doc::calc::has_math(&v) {
                crate::doc::calc::reduce(&v).ok()
            } else {
                Some(v)
            }
        };
        for (index, a) in animations.iter().enumerate() {
            let Some(frames) = self.docs.keyframes.get(&a.name) else {
                continue;
            };
            if frames.styles.is_empty() {
                continue;
            }
            let values = &cascade.values;
            let sampled = if a.timeline.ticks() {
                // The run the motion step counts, as it will count it this tick.
                let nth = animations[..index]
                    .iter()
                    .filter(|b| b.name == a.name)
                    .count() as u32;
                let ticks = self
                    .state
                    .motion
                    .css
                    .iter()
                    .find(|r| r.element == key && r.name == a.name && r.index == nth)
                    .map_or(0, |r| r.ticks + u64::from(r.stepped != now && !a.paused));
                running |= !a.settled(ticks, tps, reduced);
                a.sample_styles(
                    frames,
                    a.active_ms(ticks, tps),
                    reduced,
                    &|name| value_of(values, name),
                    &compute,
                )
            } else {
                // Its timeline's progress as the motion step last read it
                // (the motion step asks for a build when it moves).
                let Some(p) = self
                    .kept
                    .timelines
                    .get(&key)
                    .and_then(|v| v.get(index).copied().flatten())
                else {
                    continue;
                };
                let (a, active) = a.on_progress(p);
                a.sample_styles(
                    frames,
                    active,
                    false,
                    &|name| value_of(values, name),
                    &compute,
                )
            };
            for (name, value) in sampled {
                set(&mut cascade.values, &name, value);
            }
        }

        // Transitions: from the before-change value to the after-change one.
        let mut state = StyleState::default();
        let mut names: BTreeSet<String> = cascade
            .values
            .iter()
            .map(|(n, ..)| n.clone())
            .filter(|n| styled(n) && covered(n).is_some())
            .collect();
        if let Some(p) = previous {
            names.extend(
                p.after
                    .keys()
                    .filter(|n| styled(n) && covered(n).is_some())
                    .cloned(),
            );
        }
        for name in names {
            let Some(t) = covered(&name) else { continue };
            let after = value_of(&cascade.values, &name);
            let Some(after) = after else { continue };
            state.after.insert(name.clone(), after.clone());
            let Some(previous) = previous else { continue };
            let before = previous
                .after
                .get(&name)
                .cloned()
                .or_else(|| interpolate::initial(&name).map(str::to_string));
            let old = previous.runs.iter().find(|r| r.property == name);
            let sample = |r: &StyleRun| {
                let ms =
                    (now.saturating_sub(r.start)) as f32 * 1000.0 / tps.max(1) as f32 - r.delay_ms;
                if ms < 0.0 {
                    return (r.from.clone(), false);
                }
                let (p, done) = progress(&t.transition, ms, r, tps, &theme.motion);
                (interpolate::mix(&name, &r.from, &r.to, p), done)
            };
            let run = match old {
                // Still heading where it was.
                Some(r) if r.to == after => Some(r.clone()),
                _ => {
                    let current = old.map(|r| sample(r).0).or(before);
                    let duration_ms = match t.transition {
                        Transition::Tween { duration_ms, .. } => duration_ms.max(0.0),
                        Transition::Instant => 0.0,
                        _ => 1.0,
                    };
                    match current {
                        Some(from)
                            if from != after
                                && !reduced
                                && duration_ms + t.delay_ms > 0.0
                                && (t.discrete
                                    || interpolate::interpolate(&name, &from, &after, 0.5)
                                        .is_some()) =>
                        {
                            // CSS Transitions 1 §3, step 4: back towards
                            // where a running one started, shortened by how
                            // far it had come.
                            let (reversing_from, factor) = match old {
                                Some(r) if r.reversing_from == after => {
                                    let ms = (now.saturating_sub(r.start)) as f32 * 1000.0
                                        / tps.max(1) as f32
                                        - r.delay_ms;
                                    let p = if ms <= 0.0 {
                                        0.0
                                    } else {
                                        progress(&t.transition, ms, r, tps, &theme.motion).0
                                    };
                                    (
                                        r.to.clone(),
                                        (p * r.factor + 1.0 - r.factor).abs().clamp(0.0, 1.0),
                                    )
                                }
                                _ => (from.clone(), 1.0),
                            };
                            Some(StyleRun {
                                property: name.clone(),
                                from,
                                to: after.clone(),
                                start: now,
                                delay_ms: if t.delay_ms < 0.0 {
                                    t.delay_ms * factor
                                } else {
                                    t.delay_ms
                                },
                                duration_ms: duration_ms * factor,
                                reversing_from,
                                factor,
                            })
                        }
                        _ => None,
                    }
                }
            };
            if let Some(r) = run {
                let (value, done) = sample(&r);
                if !done {
                    set(&mut cascade.values, &name, value);
                    state.runs.push(r);
                }
            }
        }
        running |= !state.runs.is_empty();
        let keep = !state.after.is_empty() || !state.runs.is_empty();
        Some(Arc::new(StyleCommit {
            key,
            state: keep.then_some(state),
            running,
        }))
    }

    /// Keeps what [`Ui::animate_values`] found; a running element builds
    /// again next frame.
    pub(super) fn commit_styles(&mut self, commit: &StyleCommit) {
        match &commit.state {
            Some(state) => {
                if self.state.motion.styles.get(&commit.key) != Some(state) {
                    self.state.motion.styles.insert(commit.key, state.clone());
                }
            }
            None => {
                self.state.motion.styles.remove(&commit.key);
            }
        }
        if commit.running
            && let Some(owned) = self.owning.last_mut()
        {
            owned.restyle = true;
        }
    }
}
