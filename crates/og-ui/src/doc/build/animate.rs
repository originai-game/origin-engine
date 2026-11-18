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

fn scalar_track(frames: &crate::motion::css::Keyframes, name: &str) -> bool {
    crate::doc::motion::channels(name).is_some_and(|channels| {
        frames.tracks.iter().any(|track| {
            track
                .prop
                .channels()
                .iter()
                .any(|c| channels & (1 << c) != 0)
        })
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleTarget {
    /// Element channels animate background, colour and opacity separately.
    Element(u64),
    /// Parts have no channels: every property uses computed-value motion.
    Part(u64, &'static str),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Effects {
    /// Old declarations and existing runs, sampled without creating motion.
    Before,
    /// New keyframes, excluding transitions from after-change inheritance.
    After,
    /// Presented values; only this pass commits transition/animation state.
    Current,
}

/// What a resolve leaves the motion state, applied as the element builds.
#[derive(Clone, Debug, PartialEq)]
pub struct StyleCommit {
    pub target: StyleTarget,
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
    #[allow(clippy::too_many_arguments)]
    pub(super) fn animate_values(
        &self,
        target: StyleTarget,
        cascade: &mut cascade::Cascade,
        starting: &[(String, String, usize)],
        transitions: &[CssTransition],
        animations: &[CssAnimation],
        basis: crate::doc::calc::Basis,
        sheet: Option<&crate::doc::at_rules::Sheet>,
        attr: &dyn Fn(&str) -> Option<String>,
        attributes: &BTreeMap<String, String>,
        scope: &str,
    ) -> Option<Arc<StyleCommit>> {
        let recipe = cascade.recipe_with_attributes(attributes);
        let inherited = recipe.inherited();
        // CSS Transitions §3: advance the OLD style's declarative effects
        // to now, rather than starting from the last frame's strings.
        let before = self.previous_style(target).and_then(|previous| {
            let old = previous.recipe.as_ref()?;
            let mut values = old.cascade(inherited.before.as_deref().unwrap_or(inherited), sheet);
            let motion = values
                .values
                .iter()
                .filter(|(name, ..)| super::super::motion::longhand(name))
                .map(|(name, value, _)| (name.clone(), value.clone()))
                .collect::<Vec<_>>();
            let (old_transitions, mut old_animations) =
                super::super::motion::resolve(&motion, &previous.scope, &mut values.diagnostics);
            for (animation, name) in old_animations.iter_mut().zip(&previous.names) {
                animation.name.clone_from(name);
            }
            self.settle_effects(
                target,
                &mut values,
                &[],
                &old_transitions,
                &old_animations,
                old.basis().unwrap_or(basis),
                sheet,
                &|name| old.attribute(name),
                Effects::Before,
                None,
                None,
            );
            Some(values)
        });
        // After-change inheritance includes animations, but never a parent's
        // transitions. Retain this independently from the presented sample.
        let mut after = recipe.cascade(inherited.after.as_deref().unwrap_or(inherited), sheet);
        self.settle_effects(
            target,
            &mut after,
            &[],
            transitions,
            animations,
            basis,
            sheet,
            attr,
            Effects::After,
            None,
            None,
        );
        let mut commit = self.settle_effects(
            target,
            cascade,
            starting,
            transitions,
            animations,
            basis,
            sheet,
            attr,
            Effects::Current,
            before.as_ref(),
            Some(&after),
        );
        let state = commit.state.get_or_insert_with(Default::default);
        state.recipe = Some(recipe);
        state.scope = scope.into();
        state.names = animations.iter().map(|a| a.name.clone()).collect();
        let computed = Arc::make_mut(&mut cascade.computed);
        computed.before = before.map(|c| Arc::new(c.computed.snapshot_values()));
        computed.after = Some(Arc::new(after.computed.snapshot_values()));
        Some(Arc::new(commit))
    }

    fn previous_style(&self, target: StyleTarget) -> Option<&StyleState> {
        match target {
            StyleTarget::Part(key, name) => self
                .interaction
                .state
                .motion
                .part_styles
                .get(&key)
                .and_then(|s| s.get(name)),
            StyleTarget::Element(key) => self.interaction.state.motion.styles.get(&key),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn settle_effects(
        &self,
        target: StyleTarget,
        cascade: &mut cascade::Cascade,
        starting: &[(String, String, usize)],
        transitions: &[CssTransition],
        animations: &[CssAnimation],
        basis: crate::doc::calc::Basis,
        sheet: Option<&crate::doc::at_rules::Sheet>,
        attr: &dyn Fn(&str) -> Option<String>,
        effects: Effects,
        before: Option<&cascade::Cascade>,
        after: Option<&cascade::Cascade>,
    ) -> StyleCommit {
        let mut out = StyleCommit {
            target,
            state: None,
            running: false,
        };
        for custom in [true, false] {
            let commit = self.animate_phase(
                target,
                cascade,
                starting,
                transitions,
                animations,
                basis,
                sheet,
                custom,
                effects,
                before,
                after,
            );
            if custom {
                let samples = cascade
                    .values
                    .iter()
                    .filter(|(name, ..)| name.starts_with("--"))
                    .map(|(name, value, _)| (name.clone(), value.clone()))
                    .collect::<BTreeMap<_, _>>();
                if !samples.is_empty() {
                    *cascade = cascade.with_custom_effects(&samples, attr, sheet);
                }
            }
            if let Some(commit) = commit {
                if let Some(state) = &commit.state {
                    let merged = out.state.get_or_insert_with(Default::default);
                    merged.after.extend(state.after.clone());
                    merged.runs.extend(state.runs.clone());
                    // Both phases sample the same part clock; keep it once.
                    merged.animations.clone_from(&state.animations);
                    merged.running |= state.running;
                }
                out.running |= commit.running;
            }
        }
        out
    }

    #[allow(clippy::too_many_arguments)]
    fn animate_phase(
        &self,
        target: StyleTarget,
        cascade: &mut cascade::Cascade,
        starting: &[(String, String, usize)],
        transitions: &[CssTransition],
        animations: &[CssAnimation],
        basis: crate::doc::calc::Basis,
        sheet: Option<&crate::doc::at_rules::Sheet>,
        custom: bool,
        effects: Effects,
        before_change: Option<&cascade::Cascade>,
        after_change: Option<&cascade::Cascade>,
    ) -> Option<Arc<StyleCommit>> {
        let (key, part) = match target {
            StyleTarget::Element(key) => (key, false),
            StyleTarget::Part(key, _) => (key, true),
        };
        let previous = self.previous_style(target);
        let styled = |name: &str| {
            (part || crate::doc::motion::channels(name).is_none())
                && interpolate::animatable(name)
                && name.starts_with("--") == custom
        };
        let covered = |name: &str| transitions.iter().rev().find(|t| t.covers(name));
        let properties = sheet.map_or(&[][..], |s| s.properties.as_slice());
        let registration = |name: &str| {
            properties
                .iter()
                .rev()
                .find(|(n, _)| n == name)
                .map(|(_, p)| p)
        };
        let keyframes = |name: &str| match sheet.filter(|_| part) {
            Some(sheet) => sheet
                .keyframes
                .iter()
                .find(|(n, _)| n == name.trim_start_matches('/'))
                .map(|(_, k)| k),
            None => self.docs.library.keyframes.get(name).map(Arc::as_ref),
        };
        let has_styles = animations.iter().any(|a| {
            keyframes(&a.name).is_some_and(|k| {
                k.styles.iter().any(|t| {
                    interpolate::animatable(&t.property)
                        && t.property.starts_with("--") == custom
                        && (part || !scalar_track(k, &t.property))
                })
            })
        });
        let transitioned = transitions
            .iter()
            .any(|t| part || t.channels == 0 || t.property == "all");
        // The before-change style: last build's after-change values, or on
        // the first, `@starting-style`'s.
        let first =
            (previous.is_none() && starting.iter().any(|(n, ..)| styled(n))).then(|| StyleState {
                after: starting
                    .iter()
                    .filter(|(n, ..)| styled(n))
                    .map(|(n, v, _)| (n.clone(), v.clone()))
                    .collect(),
                ..Default::default()
            });
        let previous = previous.or(first.as_ref());
        if previous.is_none() && !has_styles && !transitioned {
            return None;
        }
        let now = self.interaction.state.tick;
        let tps = self.env.ticks_per_second;
        let reduced = self.env.theme.reduced_motion;
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
        // A border or outline colour the cascade set through a shorthand
        // (`border: … var(--rim)`) rather than a matching `*-color`
        // longhand: its computed value for a transition, which reads the
        // longhand's name. The last shorthand that sets this side wins, as
        // it does when both lower (`cascade.values` is in cascade order).
        let from_shorthand = |values: &[(String, String, usize)], name: &str| {
            (name.starts_with("border") && name.ends_with("-color") || name == "outline-color")
                .then(|| {
                    values.iter().rev().find_map(|(n, v, _)| {
                        matches!(n.as_str(), "border" | "outline")
                            .then(|| crate::css::boxes::shorthand_colour(n, v, name))
                            .or_else(|| {
                                (n.starts_with("border-")
                                    && !n.ends_with("-width")
                                    && !n.ends_with("-style")
                                    && !n.ends_with("-color"))
                                .then(|| crate::css::boxes::shorthand_colour(n, v, name))
                            })
                            .flatten()
                    })
                })
                .flatten()
        };
        let value_of = |values: &[(String, String, usize)], name: &str| {
            get(values, name)
                .or_else(|| from_shorthand(values, name))
                .or_else(|| interpolate::initial(name).map(str::to_string))
        };
        // CSS Values 5 §8.2: under `interpolate-size: allow-keywords` (an
        // inherited value) a size moves to or from an intrinsic keyword
        // through calc-size().
        let mut phase_values = cascade.values.clone();
        if custom {
            phase_values.extend(
                cascade
                    .computed
                    .iter()
                    .filter_map(|(n, v)| v.as_ref().map(|v| (n.clone(), v.clone(), 0))),
            );
        }
        let keywords = value_of(&cascade.values, "interpolate-size")
            .is_some_and(|v| v.trim() == "allow-keywords");
        let mix = |name: &str, a: &str, b: &str, p: f32| {
            if custom {
                return registration(name)
                    .and_then(|r| r.interpolate(a, b, p))
                    .unwrap_or_else(|| {
                        if p < 0.5 {
                            a.to_string()
                        } else {
                            b.to_string()
                        }
                    });
            }
            keywords
                .then(|| interpolate::mix_sizes(name, a, b, p))
                .flatten()
                .unwrap_or_else(|| interpolate::mix(name, a, b, p))
        };
        let mut running = false;
        let mut animated_values = Vec::new();
        let mut animation_cascade = phase_values.clone();

        // CSS animations, in list order over the cascade (later win).
        let theme = self.theme();
        let computed = cascade.computed.clone();
        let compute = |name: &str, text: &str| -> Option<String> {
            let v = crate::doc::variables::Value::parse(text)
                .ok()?
                .substitute(&mut |n| match computed.get(n) {
                    Some(v) => v.clone(),
                    None => None,
                })
                .ok()?;
            let v = if crate::doc::calc::per_element(name, &v) {
                crate::doc::calc::absolute(&v, basis, name).0
            } else {
                v
            };
            let v = if crate::doc::calc::has_math(&v) {
                crate::doc::calc::reduce(&v).ok()
            } else {
                Some(v)
            }?;
            if custom
                && registration(name)
                    .is_some_and(|r| !crate::css::at_rules::matches_syntax(&r.syntax, &v))
            {
                return None;
            }
            Some(v)
        };
        let mut animation_runs = Vec::new();
        for (index, a) in animations.iter().enumerate() {
            let Some(frames) = keyframes(&a.name) else {
                continue;
            };
            if frames.styles.is_empty() {
                continue;
            }
            let values = &animation_cascade;
            let sampled = if a.timeline.ticks() {
                // The run the motion step counts, as it will count it this tick.
                let nth = animations[..index]
                    .iter()
                    .filter(|b| b.name == a.name)
                    .count() as u32;
                let runs = if part {
                    previous.map_or(&[][..], |p| p.animations.as_slice())
                } else {
                    &self.interaction.state.motion.css
                };
                let run = runs
                    .iter()
                    .find(|r| r.element == key && r.name == a.name && r.index == nth);
                let ticks = run.map_or(0, |r| {
                    if r.stepped == now {
                        r.ticks
                    } else {
                        r.next_ticks(a, tps, reduced)
                    }
                });
                if part {
                    animation_runs.push(crate::motion::css::CssRun {
                        element: key,
                        index: nth,
                        name: a.name.clone(),
                        ticks,
                        stepped: now,
                        last_ms: a.active_ms(ticks, tps),
                        playback: None,
                    });
                }
                // A triggered one moves only while its playback plays.
                running |= match run.and_then(|r| r.playback.as_ref()) {
                    Some(p) => p.running(ticks, crate::motion::trigger::end_ticks(a, tps)),
                    None if !a.trigger.is_empty() && !part => false,
                    None => !a.settled(ticks, tps, reduced),
                };
                a.sample_styles(
                    frames,
                    a.active_ms(ticks, tps),
                    reduced,
                    &|name| value_of(values, name),
                    &compute,
                    custom,
                    properties,
                )
            } else {
                // Its timeline's progress as the motion step last read it
                // (the motion step asks for a build when it moves).
                let Some(p) = self
                    .style
                    .motion
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
                    custom,
                    properties,
                )
            };
            for (name, value) in sampled {
                if (part || !scalar_track(frames, &name))
                    && (!custom || !cascade.important_custom.contains(&name))
                {
                    set(&mut animation_cascade, &name, value.clone());
                    animated_values.push((name, value));
                }
            }
        }

        // Transitions: from the before-change value to the after-change one.
        let mut state = StyleState {
            animations: animation_runs,
            ..Default::default()
        };
        let mut names: BTreeSet<String> = phase_values
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
        let written = cascade.reduced_motion_authored;
        for name in names {
            if effects == Effects::After {
                continue;
            }
            let Some(t) = covered(&name) else { continue };
            // Display must keep the box alive for the opacity channels'
            // reduced-motion fade, rather than removing it immediately.
            // One written for reduced motion (under its media query) plays
            // as written.
            let reduced_display = reduced && !written && name == "display" && t.discrete;
            let transition = if reduced_display {
                match theme.motion.reduced {
                    crate::motion::Reduced::Keep => t.transition,
                    crate::motion::Reduced::None => Transition::Instant,
                    crate::motion::Reduced::Fade => Transition::Tween {
                        duration_ms: theme.motion.reduced_ms,
                        ease: crate::motion::Ease::Linear,
                    },
                }
            } else {
                t.transition
            };
            let computed_value = |c: &cascade::Cascade| {
                if custom {
                    c.computed.get(&name).cloned().flatten()
                } else {
                    value_of(&c.values, &name)
                }
            };
            let after = after_change
                .and_then(computed_value)
                .or_else(|| value_of(&animation_cascade, &name));
            let Some(after) = after else { continue };
            state.after.insert(name.clone(), after.clone());
            let Some(previous) = previous else { continue };
            let before = before_change
                .and_then(computed_value)
                .or_else(|| previous.after.get(&name).cloned())
                .or_else(|| interpolate::initial(&name).map(str::to_string));
            let old = previous.runs.iter().find(|r| r.property == name);
            let sample = |r: &StyleRun| {
                let ms =
                    (now.saturating_sub(r.start)) as f32 * 1000.0 / tps.max(1) as f32 - r.delay_ms;
                if ms < 0.0 {
                    return (r.from.clone(), false);
                }
                let (p, done) = progress(&r.timing, ms, r, tps, &theme.motion);
                (mix(&name, &r.from, &r.to, p), done)
            };
            let run = match old {
                Some(r) if effects == Effects::Before => Some(r.clone()),
                None if effects == Effects::Before => None,
                // Still heading where it was.
                Some(r) if r.to == after => Some(r.clone()),
                _ => {
                    let current = before.or_else(|| old.map(|r| sample(r).0));
                    let duration_ms = match transition {
                        Transition::Tween { duration_ms, .. } => duration_ms.max(0.0),
                        Transition::Instant => 0.0,
                        _ => 1.0,
                    };
                    match current {
                        Some(from)
                            if from != after
                                && (!reduced || written || reduced_display)
                                && duration_ms + t.delay_ms > 0.0
                                && (t.discrete
                                    || (if custom {
                                        registration(&name)
                                            .and_then(|r| r.interpolate(&from, &after, 0.5))
                                    } else {
                                        interpolate::interpolate(&name, &from, &after, 0.5)
                                    })
                                    .is_some()
                                    || (keywords
                                        && interpolate::mix_sizes(&name, &from, &after, 0.5)
                                            .is_some())) =>
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
                                        progress(&r.timing, ms, r, tps, &theme.motion).0
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
                                timing: transition.resolve(&theme.motion),
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
        for (name, value) in animated_values {
            if !state.runs.iter().any(|run| run.property == name) {
                set(&mut cascade.values, &name, value);
            }
        }
        running |= !state.runs.is_empty();
        state.running = running;
        let keep =
            !state.after.is_empty() || !state.runs.is_empty() || !state.animations.is_empty();
        Some(Arc::new(StyleCommit {
            target,
            state: keep.then_some(state),
            running,
        }))
    }
}

impl crate::ui::Style {
    /// Keeps what [`Ui::animate_values`] found; a running element builds
    /// again next frame.
    pub fn commit_styles(
        &mut self,
        commit: &StyleCommit,
        interaction: &mut crate::ui::Interaction,
        tree: &mut crate::ui::Tree,
    ) {
        match (commit.target, &commit.state) {
            (StyleTarget::Part(key, name), Some(state)) => {
                interaction
                    .state
                    .motion
                    .part_styles
                    .entry(key)
                    .or_default()
                    .insert(name.into(), state.clone());
            }
            (StyleTarget::Part(key, name), None) => {
                if let Some(styles) = interaction.state.motion.part_styles.get_mut(&key) {
                    styles.remove(name);
                }
            }
            (StyleTarget::Element(key), Some(state)) => {
                if interaction.state.motion.styles.get(&key) != Some(state) {
                    interaction.state.motion.styles.insert(key, state.clone());
                }
            }
            (StyleTarget::Element(key), None) => {
                interaction.state.motion.styles.remove(&key);
            }
        }
        if commit.running
            && let Some(owned) = tree.owning.last_mut()
        {
            owned.restyle = true;
        }
    }
}
