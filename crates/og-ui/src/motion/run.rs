//! Motion's part of [`crate::Ui::end`], between layout and drawing: each
//! element's visual states, its animated channels (springs that keep their
//! velocity, tweens), its entrance, the plays (timelines and their events),
//! layout animation (FLIP), shared elements and leaving elements (ghosts).

use super::spring::{Channel, Reduced, Transition, pack};
use super::state::*;
use super::timeline::{
    Choreo, EventKind, PlayDesc, Repeat, StaggerFrom, Timeline, UiEvent, sample,
};
use crate::element::{CHANNELS, Prop};
use crate::id::child_key;
use crate::layout::{Rect, read_values, write_values};
use crate::ui::Ui;
use crate::ui::visit::feat;

/// A play asked for this tick, flattened at `end` (when every rectangle is known).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PendingPlay {
    pub id: u32,
    /// The element it plays on (0: the element built just before, by index).
    pub root: u64,
    pub choreo: Choreo,
    pub desc: PlayDesc,
}

/// An element in a visual state or with motion in flight, for `dump_ui`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MotionDump {
    pub path: String,
    /// Each state group and its current state.
    pub states: Vec<(String, String)>,
    /// The props moving now (`x`, `opacity`, `layout`, `drag`…), and
    /// `presence` while an entrance, a stagger or a transition is in flight
    /// (with `glitch` while a glitch tears it).
    pub moving: Vec<String>,
    /// Plays on it (their ids).
    pub plays: Vec<u32>,
    /// It left the tree and plays its exit.
    pub leaving: bool,
}

/// The channel indices colour and opacity use (they fade under reduced motion).
fn fades(c: usize) -> bool {
    (7..=16).contains(&c) && c != 8
}

impl Ui {
    /// The interaction state of element `i` (`normal`, `hover`, `pressed`,
    /// `focused`, `disabled`).
    fn interaction_of(&self, i: usize) -> &'static str {
        let n = &self.tree.nodes[self.tree.interaction_owner(i)];
        if n.disabled {
            return "disabled";
        }
        let r = self.respond(n.key);
        if r.pressed {
            "pressed"
        } else if r.hovered {
            "hover"
        } else if r.focused && self.interaction.state.focus_visible {
            "focused"
        } else {
            "normal"
        }
    }

    /// The state group `g` of node `i` is in now.
    fn state_of(&self, i: usize, g: &super::StateGroup, m: &MotionState) -> String {
        let n = &self.tree.nodes[i];
        let owner = if g.name == "interaction" {
            self.tree.interaction_owner(i)
        } else {
            i
        };
        let gk = group_key(self.tree.nodes[owner].key, &g.name);
        let has = |s: &str| g.states.iter().any(|v| v.name == s);
        if let Some(s) = m.forced.get(&gk).filter(|s| has(s)) {
            return s.clone();
        }
        if g.name == "interaction" {
            let want = self.interaction_of(i);
            let chain: &[&str] = match want {
                "pressed" => &["pressed", "hover", "normal"],
                "hover" => &["hover", "normal"],
                "focused" => &["focused", "hover", "normal"],
                "disabled" => &["disabled", "normal"],
                _ => &["normal"],
            };
            if let Some(s) = chain.iter().find(|s| has(s)) {
                return (*s).to_owned();
            }
        }
        if let Some(s) = m.set_states.get(&gk).filter(|s| has(s)) {
            return s.clone();
        }
        if let Some((_, s)) = n
            .props
            .state
            .iter()
            .rev()
            .find(|(name, s)| *name == g.name && has(s))
        {
            return s.clone();
        }
        g.states.first().map(|s| s.name.clone()).unwrap_or_default()
    }

    /// Resolves the element a path below `root` names.
    pub(crate) fn below(root: u64, path: &str) -> u64 {
        path.split('/')
            .filter(|p| !p.is_empty())
            .fold(root, child_key)
    }

    /// The tick's motion: on the nodes that animate, drag or match
    /// (`visit::feat`), and those a play this tick drives. On a bounded
    /// frame only the animated nodes it may change (`seeds`: written, laid
    /// out, entered, or still moving last tick) are visited, and what is
    /// forgotten is read from them, the nodes that left the trees and the
    /// widgets' channels stepped lately; on a whole frame every animated
    /// node, and everything. Answers the animated nodes it visited.
    pub(crate) fn run_motion(&mut self, seeds: Option<&[usize]>) -> Vec<usize> {
        let whole = seeds.is_none();
        let now = self.interaction.state.tick;
        let tps = self.env.ticks_per_second;
        let tokens = self.env.theme.motion;
        let reduced = self.env.theme.reduced_motion;
        let mut m = std::mem::take(&mut self.interaction.state.motion);
        // Last step's asks were taken by this frame's build.
        self.style.motion.timeline_restyle.clear();
        let mut touched =
            crate::id::KeySet::with_capacity_and_hasher(m.channels.len(), Default::default());

        let (overrides, mut events) = self.step_plays(&mut m, now, tps, reduced);
        self.style.step_gestures(
            &mut m,
            &mut touched,
            now,
            &self.env,
            &mut self.out,
            &mut self.interaction,
            &mut self.tree,
        );

        // Each element: states, channels, entrance, FLIP.
        // The nodes that animate their layout, by node (FLIP is relative to
        // the nearest of them above).
        let mut flip_rel = std::collections::HashSet::new();
        let mut css_events: Vec<UiEvent> = Vec::new();
        let mut css_bursts: Vec<BurstRun> = Vec::new();
        let mut starts: Vec<(u64, String)> = Vec::new();
        // The nodes that animate, and those a play drives, in preorder.
        let mut animating: Vec<usize> = match seeds.filter(|_| !whole) {
            Some(seeds) => seeds
                .iter()
                .copied()
                .filter(|&i| {
                    self.tree.index.has(i)
                        && self
                            .tree
                            .orders
                            .feat
                            .get(i)
                            .is_some_and(|f| f & feat::ANIMATED != 0)
                })
                .collect(),
            None => self.tree.orders.with(feat::ANIMATED).to_vec(),
        };
        animating.extend(
            overrides
                .iter()
                .filter_map(|o| self.tree.index.by_key.get(&o.0).copied()),
        );
        animating.sort_unstable_by_key(|&i| self.tree.orders.pre_pos[i]);
        animating.dedup();
        let visited = animating.clone();
        // Visited nodes still moving: visited next tick too.
        let mut moving = Vec::new();
        if whole {
            self.style.motion.motion_groups.clear();
        }
        for i in animating {
            let key = self.tree.nodes[i].key;
            let props = &self.tree.nodes[i].props;
            let has_override = overrides.iter().any(|o| o.0 == key);
            let animated = props.animated() || has_override;
            if !animated && props.layout_motion.is_none() {
                continue;
            }
            // A play, a CSS animation and a layout motion move it on their
            // own; a channel off its target (or waiting out a delay) too.
            // A scroll-driven animation is visited when its timeline moves
            // (`Ui::frame_seeds`), not every tick.
            let busy = has_override
                || props.animations.iter().any(|a| a.timeline.ticks())
                || props.layout_motion.is_some();
            let mut node_groups = Vec::new();
            let fallback = props.reduced.unwrap_or(tokens.reduced);
            let base = read_values(&self.tree.nodes[i]);
            let mut target = base;
            // Per channel: the transition a state change gives it this tick.
            let mut moved_by: [Option<Transition>; CHANNELS] = [None; CHANNELS];
            let groups = self.step_states(
                i,
                key,
                &mut m,
                &mut touched,
                &mut node_groups,
                &mut starts,
                &mut moved_by,
                &mut target,
                now,
            );
            let enter = self.tree.nodes[i].props.enter.clone();
            let entering = enter.is_some() && !m.seen.contains_key(&key);
            if enter.is_some() {
                m.seen.entry(key).or_insert(now);
            }
            let (mut row, busy) = self.tree.step_channels(
                i,
                key,
                &groups,
                &moved_by,
                target,
                &overrides,
                animated,
                &mut m,
                &mut touched,
                busy,
                fallback,
                entering,
                &enter,
                now,
                tps,
                tokens,
                reduced,
            );
            self.style.step_animations(
                i,
                key,
                &mut row,
                &mut m,
                &mut css_events,
                &mut css_bursts,
                now,
                tps,
                reduced,
                &self.docs,
                &self.interaction,
                &self.tree,
            );
            if row != base {
                write_values(&mut self.tree.nodes[i], &row);
            }

            self.tree.step_flip(
                i,
                key,
                &mut m,
                &mut touched,
                &mut flip_rel,
                fallback,
                now,
                tps,
                tokens,
                reduced,
            );
            if busy {
                moving.push(i);
            }
            // The groups it no longer has are forgotten (on a whole tick,
            // with everything untouched, below).
            if !whole && let Some(old) = self.style.motion.motion_groups.get(&key) {
                for gk in old.iter().filter(|gk| !node_groups.contains(gk)) {
                    m.groups.remove(gk);
                    m.set_states.remove(gk);
                }
            }
            if node_groups.is_empty() {
                self.style.motion.motion_groups.remove(&key);
            } else {
                self.style.motion.motion_groups.insert(key, node_groups);
            }
        }

        self.tree
            .step_matches(&mut m, &mut touched, now, tps, tokens, reduced);
        self.style
            .step_exits(&mut m, now, tps, tokens, reduced, &self.env, &self.tree);

        let visited = self
            .style
            .forget_motion(&mut m, whole, seeds, &touched, visited, now, &self.tree);
        // A CSS animation no element ran this tick ended with its element
        // or its `animation-name`.
        m.css.retain(|r| r.stepped == now);
        let tree = &self.tree.index;
        self.style
            .motion
            .timelines
            .retain(|k, _| tree.by_key.contains_key(k));
        // Transitions of computed values outlive no element.
        m.styles
            .retain(|k, _| self.tree.index.by_key.contains_key(k));
        m.part_styles
            .retain(|k, _| self.tree.index.by_key.contains_key(k));
        m.bursts.extend(css_bursts);
        for ev in &css_events {
            if let Some(s) = crate::style_file::UiSound::from_name(&ev.name) {
                self.sound(s);
            }
        }
        events.extend(css_events);
        self.interaction.state.motion = m;
        self.style.motion.motion_moving = moving;
        self.out.events.extend(events);
        for (key, name) in starts {
            self.style.start_play(
                key,
                Choreo::Play {
                    timeline: name,
                    target: String::new(),
                    delay_ms: 0.0,
                },
                PlayDesc::default(),
                &mut self.interaction,
            );
        }
        visited
    }

    /// A node's state groups: their states now, the plays they start, the
    /// transition each moved channel takes and the values they set; the
    /// groups are handed back.
    #[allow(clippy::too_many_arguments)]
    fn step_states(
        &mut self,
        i: usize,
        key: u64,
        m: &mut MotionState,
        touched: &mut crate::id::KeySet,
        node_groups: &mut Vec<u64>,
        starts: &mut Vec<(u64, String)>,
        moved_by: &mut [Option<Transition>; CHANNELS],
        target: &mut [f32; CHANNELS],
        now: u64,
    ) -> Vec<super::StateGroup> {
        let groups = self.tree.nodes[i].props.states.clone();
        for g in &groups {
            let cur = self.state_of(i, g, m);
            let gk = group_key(key, &g.name);
            touched.insert(gk);
            node_groups.push(gk);
            let prev = m.groups.get(&gk).map(|s| s.state.clone());
            let changed = prev.as_ref().is_some_and(|p| *p != cur);
            if prev.is_none() || changed {
                m.groups.insert(
                    gk,
                    GroupState {
                        state: cur.clone(),
                        since: now,
                    },
                );
            }
            if changed {
                let prev = prev.clone().unwrap_or_default();
                if let Some(s) = g.states.iter().find(|s| s.name == prev)
                    && !s.on_exit.is_empty()
                {
                    starts.push((key, s.on_exit.clone()));
                }
                if let Some(s) = g.states.iter().find(|s| s.name == cur)
                    && !s.on_enter.is_empty()
                {
                    starts.push((key, s.on_enter.clone()));
                }
            }
            if changed {
                let t = g
                    .transition(prev.as_deref().unwrap_or(""), &cur)
                    .or(self.tree.nodes[i].props.motion)
                    .unwrap_or_default();
                // Every value the group's states touch moves with it, to
                // the new state's or back to the element's own.
                for s in &g.states {
                    for &(prop, _) in &s.values {
                        for &c in prop.channels() {
                            moved_by[c] = Some(t);
                        }
                    }
                }
            }
            if let Some(st) = g.states.iter().find(|s| s.name == cur) {
                for &(prop, v) in &st.values {
                    for &c in prop.channels() {
                        target[c] = v;
                    }
                }
            }
        }
        groups
    }

    /// Plays: those asked for this tick flattened, then each one's values
    /// this tick and the events it passed; the values by node and channel
    /// and the events.
    fn step_plays(
        &mut self,
        m: &mut MotionState,
        now: u64,
        tps: u32,
        reduced: bool,
    ) -> (Vec<(u64, usize, f32)>, Vec<UiEvent>) {
        // Plays asked for this tick: flattened now that rectangles are known.
        for p in std::mem::take(&mut self.style.pending_plays) {
            if reduced && p.desc.decorative {
                continue;
            }
            let mut clips = Vec::new();
            let length = self.style.flatten(
                &p.choreo,
                p.root,
                &self.tree.index.by_key,
                0.0,
                0,
                &mut clips,
                &self.tree,
            );
            let first = clips
                .first()
                .and_then(|c| self.style.timelines.get(&c.timeline))
                .map(|(_, t)| t.clone())
                .unwrap_or_default();
            let at = first.label(&p.desc.from_label).unwrap_or(p.desc.from_ms);
            m.plays.retain(|q| q.id != p.id);
            m.plays.push(Play {
                id: p.id,
                root: p.root,
                clips,
                start: now,
                at_ms: at,
                last_ms: at - 1e-3,
                speed: if p.desc.speed > 0.0 {
                    p.desc.speed
                } else {
                    1.0
                },
                reverse: p.desc.reverse,
                repeat: match p.desc.repeat {
                    Repeat::Once => 0,
                    Repeat::Loop => 1,
                    Repeat::PingPong => 2,
                },
                hold: p.desc.hold,
                paused: false,
                length_ms: length,
                ended: false,
            });
        }

        // Plays: values this tick and the events passed.
        let mut overrides: Vec<(u64, usize, f32)> = Vec::new();
        let mut events: Vec<UiEvent> = Vec::new();
        let mut state_events: Vec<(u64, String, String)> = Vec::new();
        let mut bursts: Vec<BurstRun> = Vec::new();
        m.plays
            .retain(|p| self.tree.index.by_key.contains_key(&p.root) && !(p.ended && !p.hold));
        if reduced {
            // Decorative loops stop under reduced motion.
            m.plays.retain(|p| p.repeat == 0);
        }
        for p in &mut m.plays {
            let len = p.length_ms.max(1e-3);
            let raw = if p.paused {
                p.at_ms
            } else {
                p.at_ms + (now - p.start) as f32 * 1000.0 / tps as f32 * p.speed
            };
            let map = |raw: f32| -> f32 {
                let t = match p.repeat {
                    0 => raw.clamp(0.0, len),
                    1 => raw.rem_euclid(len),
                    _ => {
                        let k = raw.rem_euclid(2.0 * len);
                        if k > len { 2.0 * len - k } else { k }
                    }
                };
                if p.reverse { len - t } else { t }
            };
            let (t_prev, t_now) = (map(p.last_ms), map(raw));
            let wrapped = p.repeat == 1 && (raw / len).floor() != (p.last_ms / len).floor();
            // A play's first tick includes its starting instant: an event at
            // 0 ms (a reversed play's last) fires as it starts.
            let fresh = p.last_ms < p.at_ms;
            let crossed = |e: f32| -> bool {
                if wrapped {
                    if p.reverse {
                        (e < t_prev && e >= 0.0) || (e >= t_now && e <= len)
                    } else {
                        (e > t_prev && e <= len) || (e >= 0.0 && e <= t_now)
                    }
                } else if t_now > t_prev || (t_now == t_prev && !p.reverse) {
                    (e > t_prev || (fresh && e == t_prev)) && e <= t_now
                } else {
                    e >= t_now && (e < t_prev || (fresh && e == t_prev))
                }
            };
            if p.repeat == 0 && raw >= len && !p.ended {
                p.ended = true;
                events.push(UiEvent {
                    kind: "end".into(),
                    play: p.id,
                    pitch: 1.0,
                    ..UiEvent::default()
                });
            }
            for clip in &p.clips {
                let Some((_, tl)) = self.style.timelines.get(&clip.timeline) else {
                    continue;
                };
                let local = (t_now - clip.offset_ms).clamp(0.0, tl.length_ms());
                for tr in &tl.tracks {
                    let target = Ui::below(clip.target, &tr.target);
                    if let Some(v) = sample(tr, local, tps) {
                        for &c in tr.prop.channels() {
                            overrides.push((target, c, v));
                        }
                    }
                }
                if t_prev == t_now && p.last_ms >= p.at_ms {
                    continue;
                }
                for e in &tl.events {
                    if !crossed(clip.offset_ms + e.at_ms) {
                        continue;
                    }
                    let target = Ui::below(clip.target, &e.target);
                    let path = self
                        .tree
                        .index
                        .by_key
                        .get(&target)
                        .map_or(String::new(), |&i| self.tree.path_of(i));
                    let pitch_of = |pitch: f32| {
                        (if pitch > 0.0 { pitch } else { 1.0 }) * 1.06f32.powi(clip.index as i32)
                    };
                    let mut ev = UiEvent {
                        target: path,
                        play: p.id,
                        index: clip.index,
                        pitch: 1.0,
                        ..UiEvent::default()
                    };
                    match &e.kind {
                        EventKind::Sound { name, pitch } => {
                            ev.kind = "sound".into();
                            ev.name = name.clone();
                            ev.pitch = pitch_of(*pitch);
                        }
                        EventKind::Haptic(name) => {
                            ev.kind = "haptic".into();
                            ev.name = name.clone();
                        }
                        EventKind::Burst {
                            count,
                            radius,
                            color,
                            duration_ms,
                        } => {
                            ev.kind = "burst".into();
                            bursts.push(BurstRun {
                                key: target,
                                since: now,
                                count: *count,
                                radius: *radius,
                                color: *color,
                                duration_ms: *duration_ms,
                            });
                        }
                        EventKind::State { group, state } => {
                            ev.kind = "state".into();
                            ev.name = format!("{group}={state}");
                            state_events.push((target, group.clone(), state.clone()));
                        }
                        EventKind::Callback(name) => {
                            ev.kind = "callback".into();
                            ev.name = name.clone();
                        }
                    }
                    events.push(ev);
                }
            }
            p.last_ms = raw;
        }
        for (key, group, s) in state_events {
            m.set_states.insert(group_key(key, &group), s);
        }
        for ev in &events {
            if ev.kind == "sound"
                && let Some(s) = crate::style_file::UiSound::from_name(&ev.name)
            {
                self.sound(s);
            }
        }
        if !reduced {
            m.bursts.extend(bursts);
        }

        (overrides, events)
    }
}

impl crate::ui::Style {
    /// Drags and pinches.
    #[allow(clippy::too_many_arguments)]
    fn step_gestures(
        &mut self,
        m: &mut MotionState,
        touched: &mut crate::id::KeySet,
        now: u64,
        env: &crate::ui::Env,
        ui_out: &mut crate::ui::Out,
        interaction: &mut crate::ui::Interaction,
        tree: &mut crate::ui::Tree,
    ) {
        // Drags and pinches.
        for k in 0..tree.orders.with(feat::GESTURE).len() {
            let i = tree.orders.with(feat::GESTURE)[k];
            let n = &tree.nodes[i];
            if n.props.drag.is_none() && n.props.pinch.is_none() {
                continue;
            }
            let key = n.key;
            let out = interaction.run_gesture(i, m, env, ui_out, tree);
            tree.nodes[i].gesture = out;
            touched.insert(channel_key(key, DRAG_X));
            touched.insert(channel_key(key, DRAG_Y));
        }
        m.gestures.retain(|_, g| g.tick == now);
    }
}

impl crate::ui::Tree {
    fn interaction_owner(&self, i: usize) -> usize {
        self.nodes[i]
            .part_box
            .as_ref()
            .filter(|part| self.nodes[part.owner].content_slot == i)
            .map_or(i, |part| part.owner)
    }

    /// A node's channels: each one's transition, delay and value this tick;
    /// the values and whether any is still moving.
    #[allow(clippy::too_many_arguments)]
    fn step_channels(
        &mut self,
        i: usize,
        key: u64,
        groups: &[super::StateGroup],
        moved_by: &[Option<Transition>; CHANNELS],
        target: [f32; CHANNELS],
        overrides: &[(u64, usize, f32)],
        animated: bool,
        m: &mut MotionState,
        touched: &mut crate::id::KeySet,
        mut busy: bool,
        fallback: Reduced,
        entering: bool,
        enter: &Option<super::PresenceSpec>,
        now: u64,
        tps: u32,
        tokens: crate::motion::MotionTokens,
        reduced: bool,
    ) -> ([f32; CHANNELS], bool) {
        let motion = self.nodes[i].props.motion;
        let css_transitions = self.nodes[i].props.transitions.clone();
        let mut row = target;
        for c in 0..CHANNELS {
            let ck = channel_key(key, c);
            let stepped = Prop::stepped(c);
            // A state group's own transition, then CSS `transition`.
            let css =
                super::css::transition_for(&css_transitions, c).filter(|_| moved_by[c].is_none());
            let mut tr = moved_by[c].or(css.map(|t| t.transition)).or(motion);
            // Whole ticks, rounded up; no delay is no tick.
            let delay = css.filter(|t| t.delay_ms != 0.0).map_or(0, |t| {
                crate::style::Theme::ticks(t.delay_ms.abs(), tps) as i64
                    * if t.delay_ms < 0.0 { -1 } else { 1 }
            });
            if reduced {
                tr = tr.map(|t| match fallback {
                    Reduced::Keep => t,
                    Reduced::None => Transition::Instant,
                    Reduced::Fade if fades(c) => Transition::Tween {
                        duration_ms: tokens.reduced_ms,
                        ease: super::Ease::Linear,
                    },
                    Reduced::Fade => Transition::Instant,
                });
            }
            let ov = overrides
                .iter()
                .rev()
                .find(|o| o.0 == key && o.1 == c)
                .map(|o| o.2);
            match m.channels.get_mut(&ck) {
                Some(ch) => {
                    touched.insert(ck);
                    if let Some(v) = ov {
                        *ch = Channel::at(v, now);
                        row[c] = v;
                        continue;
                    }
                    if ch.target != target[c] {
                        let (kind, mode) = pack(&tr.unwrap_or(Transition::Instant), &tokens);
                        // A negative delay starts partway through.
                        let since = (now as i64 + delay).max(0);
                        ch.retarget(target[c], kind, mode, since as u64);
                    }
                    row[c] = ch.step(now, tps, stepped);
                    busy |= !ch.settled() || now < ch.since;
                }
                None => {
                    if let Some(v) = ov {
                        if animated {
                            m.channels.insert(ck, Channel::at(v, now));
                            touched.insert(ck);
                        }
                        row[c] = v;
                        continue;
                    }
                    let from = enter.as_ref().filter(|_| entering).and_then(|e| {
                        e.values
                            .iter()
                            .rev()
                            .find(|(p, _)| p.channels().contains(&c))
                            .map(|&(_, v)| (v, e))
                    });
                    if let Some((v, e)) = from {
                        let t = if reduced {
                            match fallback {
                                Reduced::Keep => e.transition,
                                Reduced::Fade if fades(c) => Transition::Tween {
                                    duration_ms: tokens.reduced_ms,
                                    ease: super::Ease::Linear,
                                },
                                _ => Transition::Instant,
                            }
                        } else {
                            e.transition
                        };
                        let mut ch = Channel::at(v, now);
                        let (kind, mode) = pack(&t, &tokens);
                        let delay = crate::style::Theme::ticks(e.delay_ms, tps) as u64;
                        let delay = if e.delay_ms > 0.0 { delay } else { 0 };
                        ch.retarget(target[c], kind, mode, now + delay);
                        ch.stepped = now.saturating_sub(1);
                        row[c] = ch.step(now, tps, stepped);
                        busy |= !ch.settled() || now < ch.since;
                        m.channels.insert(ck, ch);
                        touched.insert(ck);
                    } else if let Some(&(_, v)) = css.and(
                        self.nodes[i]
                            .props
                            .starting
                            .iter()
                            .rev()
                            .find(|(p, _)| p.channels().contains(&c)),
                    ) {
                        // `@starting-style`: the first style's
                        // before-change value, moved by its transition.
                        let mut ch = Channel::at(v, now);
                        let (kind, mode) = pack(&tr.unwrap_or(Transition::Instant), &tokens);
                        ch.retarget(target[c], kind, mode, (now as i64 + delay).max(0) as u64);
                        ch.stepped = now.saturating_sub(1);
                        row[c] = ch.step(now, tps, stepped);
                        busy |= !ch.settled() || now < ch.since;
                        m.channels.insert(ck, ch);
                        touched.insert(ck);
                    } else if tr.is_some() || moved_by[c].is_some() || !groups.is_empty() {
                        m.channels.insert(ck, Channel::at(target[c], now));
                        touched.insert(ck);
                    }
                }
            }
        }
        (row, busy)
    }

    /// FLIP: from a node's last laid-out rectangle to this one, relative
    /// to the nearest ancestor that animates its layout too.
    #[allow(clippy::too_many_arguments)]
    fn step_flip(
        &mut self,
        i: usize,
        key: u64,
        m: &mut MotionState,
        touched: &mut crate::id::KeySet,
        flip_rel: &mut std::collections::HashSet<usize>,
        fallback: Reduced,
        now: u64,
        tps: u32,
        tokens: crate::motion::MotionTokens,
        reduced: bool,
    ) {
        // FLIP: from its last laid-out rectangle to this one (relative to
        // the nearest ancestor that animates its layout too).
        if let Some(lm) = self.nodes[i].props.layout_motion {
            let rect = self.nodes[i].rect;
            let mut up = self.nodes[i].parent;
            let mut origin = (0.0, 0.0);
            while up != crate::layout::NONE {
                if flip_rel.contains(&up) {
                    let pr = self.nodes[up].rect;
                    origin = (pr.x, pr.y);
                    break;
                }
                up = self.nodes[up].parent;
            }
            let rel = Rect::new(
                rect.x - origin.0,
                rect.y - origin.1,
                rect.width,
                rect.height,
            );
            flip_rel.insert(i);
            let t = if reduced && fallback != Reduced::Keep {
                Transition::Instant
            } else {
                lm
            };
            let (kind, mode) = pack(&t, &tokens);
            let keys = [FLIP_X, FLIP_Y, FLIP_SX, FLIP_SY].map(|c| channel_key(key, c));
            if let Some(prev) = m.flip.get(&key).copied()
                && prev != rel
            {
                let get = |m: &MotionState, k: u64, d: f32| {
                    m.channels.get(&k).copied().unwrap_or(Channel::at(d, now))
                };
                let (mut cx, mut cy) = (get(m, keys[0], 0.0), get(m, keys[1], 0.0));
                let (mut sx, mut sy) = (get(m, keys[2], 1.0), get(m, keys[3], 1.0));
                // Where it shows now, kept: the offset takes up the move.
                cx.value += prev.x - rel.x;
                cy.value += prev.y - rel.y;
                if rel.width > 0.0 {
                    sx.value *= prev.width / rel.width;
                }
                if rel.height > 0.0 {
                    sy.value *= prev.height / rel.height;
                }
                cx.retarget(0.0, kind, mode, now);
                cy.retarget(0.0, kind, mode, now);
                sx.retarget(1.0, kind, mode, now);
                sy.retarget(1.0, kind, mode, now);
                for (k, ch) in keys.iter().zip([cx, cy, sx, sy]) {
                    m.channels.insert(*k, ch);
                }
            }
            m.flip.insert(key, rel);
            touched.insert(key);
            let mut out = [0.0, 0.0, 1.0, 1.0];
            for (o, k) in out.iter_mut().zip(keys) {
                if let Some(ch) = m.channels.get_mut(&k) {
                    *o = ch.step(now, tps, false);
                    touched.insert(k);
                }
            }
            self.nodes[i].flip = out;
        }
    }

    /// Shared elements: one that arrives where another with its match key
    /// was last tick morphs from that one's place.
    fn step_matches(
        &mut self,
        m: &mut MotionState,
        touched: &mut crate::id::KeySet,
        now: u64,
        tps: u32,
        tokens: crate::motion::MotionTokens,
        reduced: bool,
    ) {
        // Shared elements: one that arrives where another with its match
        // key was last tick morphs from that one's place, and the old one's
        // drawing follows it while it fades.
        for k in 0..self.orders.with(feat::MATCH).len() {
            let i = self.orders.with(feat::MATCH)[k];
            let key = self.nodes[i].key;
            let mk = name_hash(&self.nodes[i].props.match_key);
            let Some(prev) = m.matches.get(&mk).copied() else {
                continue;
            };
            if prev.key == key || prev.tick + 1 < now || self.index.by_key.contains_key(&prev.key) {
                continue;
            }
            let rect = self.nodes[i].rect;
            let t = self.nodes[i]
                .props
                .layout_motion
                .unwrap_or(Transition::Token(super::MotionToken::Panel));
            let t = if reduced { Transition::Instant } else { t };
            let (kind, mode) = pack(&t, &tokens);
            let mut set = |c: usize, from: f32, to: f32| {
                let mut ch = Channel::at(from, now);
                ch.retarget(to, kind, mode, now);
                let k = channel_key(key, c);
                m.channels.insert(k, ch);
                touched.insert(k);
            };
            set(FLIP_X, prev.rect.x - rect.x, 0.0);
            set(FLIP_Y, prev.rect.y - rect.y, 0.0);
            set(
                FLIP_SX,
                if rect.width > 0.0 {
                    prev.rect.width / rect.width
                } else {
                    1.0
                },
                1.0,
            );
            set(
                FLIP_SY,
                if rect.height > 0.0 {
                    prev.rect.height / rect.height
                } else {
                    1.0
                },
                1.0,
            );
            let r = self.nodes[i].sampled.radius;
            set(8, prev.radius, r);
            // The flip pivots on its top left: fold the scale into the offset
            // so the first frame sits exactly on the old rectangle.
            self.nodes[i].flip = [
                prev.rect.x - rect.x,
                prev.rect.y - rect.y,
                if rect.width > 0.0 {
                    prev.rect.width / rect.width
                } else {
                    1.0
                },
                if rect.height > 0.0 {
                    prev.rect.height / rect.height
                } else {
                    1.0
                },
            ];
            self.nodes[i].sampled.radius = prev.radius;
            if let Some(f) = m.frozen.get(&prev.key).cloned() {
                let ticks = crate::style::Theme::ticks(t.duration_ms(&tokens), tps);
                let mut fade = Channel::at(1.0, now);
                let (k2, m2) = pack(
                    &Transition::Tween {
                        duration_ms: t.duration_ms(&tokens) * 0.6,
                        ease: super::Ease::Linear,
                    },
                    &tokens,
                );
                fade.retarget(0.0, k2, m2, now);
                m.ghosts.retain(|g| g.key != prev.key);
                m.ghosts.push(Ghost {
                    key: prev.key,
                    frozen: f,
                    channels: vec![(7, fade)],
                    since: now,
                    morph_to: key,
                    ticks,
                });
            }
        }
    }
}

impl crate::ui::Style {
    /// Flattens a choreography into clips from `offset`, answering its end.
    #[allow(clippy::too_many_arguments)]
    fn flatten(
        &self,
        c: &Choreo,
        root: u64,
        present: &crate::id::KeyMap<usize>,
        offset: f32,
        index: u32,
        out: &mut Vec<Clip>,
        tree: &crate::ui::Tree,
    ) -> f32 {
        let length = |name: &str| {
            self.timelines
                .get(&name_hash(name))
                .map_or(0.0, |(_, t)| t.length_ms())
        };
        match c {
            Choreo::Play {
                timeline,
                target,
                delay_ms,
            } => {
                let at = offset + delay_ms.max(0.0);
                out.push(Clip {
                    timeline: name_hash(timeline),
                    target: Ui::below(root, target),
                    offset_ms: at,
                    index,
                });
                at + length(timeline)
            }
            Choreo::Sequence(steps) => {
                let mut t = offset;
                for s in steps {
                    t = self.flatten(s, root, present, t, index, out, tree);
                }
                t
            }
            Choreo::Parallel(steps) => steps
                .iter()
                .map(|s| self.flatten(s, root, present, offset, index, out, tree))
                .fold(offset, f32::max),
            Choreo::Stagger {
                timeline,
                targets,
                step_ms,
                cap_ms,
                from,
            } => {
                let keys: Vec<u64> = if targets.is_empty() {
                    present.get(&root).map_or(Vec::new(), |&i| {
                        crate::layout::all_children(&tree.nodes[i])
                            .into_iter()
                            .map(|c| tree.nodes[c].key)
                            .collect()
                    })
                } else {
                    targets.iter().map(|t| Ui::below(root, t)).collect()
                };
                let n = keys.len();
                if n == 0 {
                    return offset;
                }
                let centre = |k: u64| {
                    present
                        .get(&k)
                        .map_or((0.0, 0.0), |&i| tree.nodes[i].rect.center())
                };
                let pts: Vec<(f32, f32)> = keys.iter().map(|&k| centre(k)).collect();
                let metric: Vec<f32> = match *from {
                    StaggerFrom::Start => (0..n).map(|k| k as f32).collect(),
                    StaggerFrom::End => (0..n).map(|k| (n - 1 - k) as f32).collect(),
                    StaggerFrom::Center | StaggerFrom::Point(..) => {
                        let (cx, cy) = match *from {
                            StaggerFrom::Point(x, y) => (x, y),
                            _ => {
                                let (sx, sy) = pts
                                    .iter()
                                    .fold((0.0, 0.0), |(a, b), &(x, y)| (a + x, b + y));
                                (sx / n as f32, sy / n as f32)
                            }
                        };
                        pts.iter()
                            .map(|&(x, y)| ((x - cx) * (x - cx) + (y - cy) * (y - cy)).sqrt())
                            .collect()
                    }
                };
                let max = metric.iter().copied().fold(0.0, f32::max);
                let mut total = step_ms.max(0.0) * (n - 1) as f32;
                if *cap_ms > 0.0 {
                    total = total.min(*cap_ms);
                }
                // Ranks for the clip index: by the metric, ties by order.
                let mut rank: Vec<usize> = (0..n).collect();
                og_core::sort::sort_by(&mut rank, |&a, &b| metric[a].total_cmp(&metric[b]));
                let mut end = offset;
                for (r, &k) in rank.iter().enumerate() {
                    let delay = if max > 0.0 {
                        metric[k] / max * total
                    } else {
                        0.0
                    };
                    let at = offset + delay;
                    out.push(Clip {
                        timeline: name_hash(timeline),
                        target: keys[k],
                        offset_ms: at,
                        index: r as u32,
                    });
                    end = end.max(at + length(timeline));
                }
                end
            }
            Choreo::Cascade { steps, on } => {
                let mut t = offset;
                for (k, s) in steps.iter().enumerate() {
                    let first = out.len();
                    let end = self.flatten(s, root, present, t, k as u32, out, tree);
                    // The next step starts at this one's first `on` event.
                    let cue = out[first..]
                        .iter()
                        .filter_map(|clip| {
                            let (_, tl) = self.timelines.get(&clip.timeline)?;
                            tl.events
                                .iter()
                                .filter(|e| matches!(&e.kind, EventKind::Callback(n) if n == on))
                                .map(|e| clip.offset_ms + e.at_ms)
                                .reduce(f32::min)
                        })
                        .reduce(f32::min);
                    t = cue.unwrap_or(end);
                }
                t
            }
        }
    }

    /// A node's CSS animations, over its channels' values: later entries
    /// win.
    #[allow(clippy::too_many_arguments)]
    fn step_animations(
        &mut self,
        i: usize,
        key: u64,
        row: &mut [f32; CHANNELS],
        m: &mut MotionState,
        css_events: &mut Vec<UiEvent>,
        css_bursts: &mut Vec<BurstRun>,
        now: u64,
        tps: u32,
        reduced: bool,
        docs: &crate::ui::Docs,
        interaction: &crate::ui::Interaction,
        tree: &crate::ui::Tree,
    ) {
        // CSS animations: over the channels' values, later entries win.
        let animations = tree.nodes[i].props.animations.clone();
        let scrolled = animations.iter().any(|a| !a.timeline.ticks());
        let mut progress: Vec<Option<f32>> = vec![None; animations.len()];
        let mut styled = false;
        for (index, a) in animations.iter().enumerate() {
            let Some(frames) = docs.host.keyframes.get(&a.name).cloned() else {
                continue;
            };
            if !a.timeline.ticks() {
                // Its time is its timeline's progress (no run, no
                // events: scrolling back and forth is not playback).
                let p = self.timeline_progress(i, a, interaction, tree);
                progress[index] = p;
                styled |= !frames.styles.is_empty();
                if let Some(p) = p {
                    let (b, active) = a.on_progress(p);
                    for (c, v) in b.sample_active(&frames, active, row, false).values {
                        row[c] = v;
                    }
                }
                continue;
            }
            let nth = animations[..index]
                .iter()
                .filter(|b| b.name == a.name)
                .count() as u32;
            let at = match m
                .css
                .iter()
                .position(|r| r.element == key && r.name == a.name && r.index == nth)
            {
                Some(at) => at,
                None => {
                    m.css.push(super::css::CssRun {
                        element: key,
                        index: nth,
                        name: a.name.clone(),
                        ticks: 0,
                        stepped: now,
                        last_ms: -a.delay_ms - 1e-3,
                    });
                    m.css.len() - 1
                }
            };
            let run = &mut m.css[at];
            if run.stepped != now {
                run.stepped = now;
                if !a.paused {
                    run.ticks += 1;
                }
            }
            let sample = a.sample(&frames, run.ticks, tps, row, reduced);
            for (c, v) in sample.values {
                row[c] = v;
            }
            if !(reduced && a.iterations.is_infinite()) {
                for kind in a.passed(&frames, run.last_ms, sample.active_ms) {
                    match kind {
                        EventKind::Sound { name, pitch } => css_events.push(UiEvent {
                            kind: "sound".into(),
                            name: name.clone(),
                            target: tree.path_of(i),
                            pitch: *pitch,
                            ..UiEvent::default()
                        }),
                        // `-og-burst` in a keyframe: silenced under
                        // reduced motion, presentation only.
                        EventKind::Burst {
                            count,
                            radius,
                            color,
                            duration_ms,
                        } if !reduced => css_bursts.push(BurstRun {
                            key: tree.nodes[i].key,
                            since: now,
                            count: *count,
                            radius: *radius,
                            color: *color,
                            duration_ms: *duration_ms,
                        }),
                        _ => {}
                    }
                }
            }
            run.last_ms = sample.active_ms;
        }
        if scrolled {
            let old = self.motion.timelines.insert(key, progress.clone());
            if styled && old.as_ref() != Some(&progress) {
                self.motion.timeline_restyle.push(key);
            }
        } else {
            self.motion.timelines.remove(&key);
        }
    }

    /// Leaving elements, the ghosts they leave and the bursts an event
    /// started.
    #[allow(clippy::too_many_arguments)]
    fn step_exits(
        &mut self,
        m: &mut MotionState,
        now: u64,
        tps: u32,
        tokens: crate::motion::MotionTokens,
        reduced: bool,
        env: &crate::ui::Env,
        tree: &crate::ui::Tree,
    ) {
        // Leaving elements: what had an exit and is gone plays it from its
        // last drawing; what came back stops leaving.
        let gone: Vec<u64> = m
            .frozen
            .iter()
            .filter(|(k, f)| !tree.index.by_key.contains_key(k) && f.exit.is_some())
            .map(|(k, _)| *k)
            .collect();
        m.ghosts.retain(|g| !tree.index.by_key.contains_key(&g.key));
        for k in gone {
            if m.ghosts.iter().any(|g| g.key == k) {
                continue;
            }
            let Some(f) = m.frozen.get(&k).cloned() else {
                continue;
            };
            let exit = f.exit.clone().unwrap_or_default();
            let t = if reduced {
                match tokens.reduced {
                    Reduced::Keep => exit.transition,
                    Reduced::None => Transition::Instant,
                    Reduced::Fade => Transition::Tween {
                        duration_ms: tokens.reduced_ms,
                        ease: super::Ease::Linear,
                    },
                }
            } else {
                exit.transition
            };
            let (kind, mode) = pack(&t, &tokens);
            let delay = if exit.delay_ms > 0.0 {
                crate::style::Theme::ticks(exit.delay_ms, tps) as u64
            } else {
                0
            };
            let neutral = [0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0];
            let mut channels = Vec::new();
            for &(prop, v) in &exit.values {
                for &c in prop.channels() {
                    if c >= neutral.len() || (reduced && !fades(c) && t != exit.transition) {
                        continue;
                    }
                    let mut ch = Channel::at(neutral[c], now);
                    ch.retarget(v, kind, mode, now + delay);
                    channels.push((c as u8, ch));
                }
            }
            if channels.is_empty() {
                let mut ch = Channel::at(1.0, now);
                ch.retarget(1.0, kind, mode, now + delay);
                channels.push((7, ch));
            }
            let ticks = crate::style::Theme::ticks(t.duration_ms(&tokens), tps) + delay as u32;
            m.ghosts.push(Ghost {
                key: k,
                frozen: f,
                channels,
                since: now,
                morph_to: 0,
                ticks,
            });
        }
        for g in &mut m.ghosts {
            for (_, ch) in &mut g.channels {
                ch.step(now, tps, false);
            }
        }
        m.ghosts.retain(|g| {
            let left = now.saturating_sub(g.since) <= u64::from(g.ticks)
                || g.channels.iter().any(|(_, c)| !c.done(now, tps));
            left && (g.morph_to == 0 || tree.index.by_key.contains_key(&g.morph_to))
        });

        // Bursts an event started.
        m.bursts.retain(|b| {
            let ms = if b.duration_ms > 0.0 {
                b.duration_ms
            } else {
                env.theme.durations_ms.slow + env.theme.durations_ms.base
            };
            tree.index.by_key.contains_key(&b.key)
                && now.saturating_sub(b.since) < u64::from(crate::style::Theme::ticks(ms, tps))
        });
    }

    /// Keeps what was used and drops the memory of elements that are gone;
    /// the nodes visited are handed back.
    #[allow(clippy::too_many_arguments)]
    fn forget_motion(
        &mut self,
        m: &mut MotionState,
        whole: bool,
        seeds: Option<&[usize]>,
        touched: &crate::id::KeySet,
        visited: Vec<usize>,
        now: u64,
        tree: &crate::ui::Tree,
    ) -> Vec<usize> {
        // Keep what was used; drop the memory of elements that are gone.
        // A channel nobody stepped this tick belongs to something gone.
        if whole {
            // What lives only by being stepped (a widget's own channels,
            // `animate_key`) is read again next tick.
            let mut stepped = Vec::new();
            m.channels.retain(|k, c| {
                let touched = touched.contains(k);
                if !touched && c.stepped >= now {
                    stepped.push(*k);
                }
                touched || c.stepped >= now
            });
            self.motion.widget_channels = stepped;
            m.groups.retain(|k, _| touched.contains(k));
            m.set_states.retain(|k, _| touched.contains(k));
            m.flip.retain(|k, _| tree.index.by_key.contains_key(k));
            m.seen.retain(|k, _| tree.index.by_key.contains_key(k));
        } else {
            // A widget's channel nothing stepped this tick is forgotten,
            // as the whole frame forgets it.
            let mut lately = std::mem::take(&mut self.motion.widget_channels);
            lately.sort_unstable();
            lately.dedup();
            lately.retain(|k| match m.channels.get(k) {
                Some(_) if touched.contains(k) => false,
                Some(c) if c.stepped < now => {
                    m.channels.remove(k);
                    false
                }
                Some(_) => true,
                None => false,
            });
            self.motion.widget_channels = lately;
            // What left the trees is forgotten whole.
            for &key in &tree.index.moved_keys {
                if tree.index.by_key.contains_key(&key) {
                    continue;
                }
                for c in 0..CHANNEL_NAMES.len() {
                    let ck = channel_key(key, c);
                    if !touched.contains(&ck)
                        && m.channels.get(&ck).is_some_and(|ch| ch.stepped < now)
                    {
                        m.channels.remove(&ck);
                    }
                }
                for gk in self.motion.motion_groups.remove(&key).unwrap_or_default() {
                    m.groups.remove(&gk);
                    m.set_states.remove(&gk);
                }
                m.flip.remove(&key);
                m.seen.remove(&key);
            }
            // Of what stayed, only a node the frame wrote (one that stopped
            // animating among them) or visited can have stopped using a
            // channel or group.
            let written = seeds.unwrap_or_default().iter().chain(&visited);
            for &i in written.filter(|&&i| i < tree.nodes.len()) {
                let key = tree.nodes[i].key;
                for c in 0..CHANNEL_NAMES.len() {
                    let ck = channel_key(key, c);
                    if !touched.contains(&ck)
                        && m.channels.get(&ck).is_some_and(|ch| ch.stepped < now)
                    {
                        m.channels.remove(&ck);
                    }
                }
                if !visited.contains(&i)
                    && let Some(old) = self.motion.motion_groups.remove(&key)
                {
                    for gk in old {
                        m.groups.remove(&gk);
                        m.set_states.remove(&gk);
                    }
                }
            }
        }
        visited
    }

    /// Queues a play on element `key` (flattened at the next `end`).
    pub(crate) fn start_play(
        &mut self,
        key: u64,
        choreo: Choreo,
        desc: PlayDesc,
        interaction: &mut crate::ui::Interaction,
    ) -> u32 {
        interaction.state.motion.next_play += 1;
        let id = interaction.state.motion.next_play;
        self.pending_plays.push(PendingPlay {
            id,
            root: key,
            choreo,
            desc,
        });
        id
    }
}

/// Where a play is ([`Ui::play_info`]).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlayInfo {
    /// Its time, ms (before a reverse's turn).
    pub at_ms: f32,
    pub length_ms: f32,
    pub paused: bool,
    pub reverse: bool,
    pub ended: bool,
}

impl Ui {
    /// Defines a timeline by name (configuration, like the theme: define
    /// it once, at start). Plays and state groups name it.
    pub fn define_timeline(&mut self, name: &str, timeline: Timeline) {
        self.style
            .timelines
            .insert(name_hash(name), (name.to_owned(), timeline));
    }

    /// A timeline by name, as defined.
    pub fn timeline(&self, name: &str) -> Option<&Timeline> {
        self.style.timelines.get(&name_hash(name)).map(|(_, t)| t)
    }

    /// The element a play targets: a full path, or empty for the element
    /// built just before.
    fn play_root(&mut self, path: &str) -> Option<u64> {
        if path.is_empty() {
            let i = self.tree.last_closed;
            if i == crate::layout::NONE {
                self.out.warnings.push(
                    "play with no element before it: build the element first or pass its path"
                        .into(),
                );
                return None;
            }
            return Some(self.tree.nodes[i].key);
        }
        Some(crate::id::key_of_path(path))
    }

    /// Plays a choreography on an element (a full path; empty: the element
    /// built just before) from the end of this tick: its timelines' keys
    /// drive the elements' props and its events fire as it passes them
    /// ([`crate::UiFrame::events`]). Answers the play's id (0: no element).
    pub fn play(&mut self, path: &str, choreo: Choreo, desc: PlayDesc) -> u32 {
        match self.play_root(path) {
            Some(root) => self
                .style
                .start_play(root, choreo, desc, &mut self.interaction),
            None => 0,
        }
    }

    /// Plays one timeline (by name) on an element.
    pub fn play_timeline(&mut self, path: &str, timeline: &str, desc: PlayDesc) -> u32 {
        self.play(
            path,
            Choreo::Play {
                timeline: timeline.to_owned(),
                target: String::new(),
                delay_ms: 0.0,
            },
            desc,
        )
    }

    /// The play's time now, ms.
    fn play_now(&self, p: &Play) -> f32 {
        if p.paused {
            p.at_ms
        } else {
            p.at_ms
                + self.interaction.state.tick.saturating_sub(p.start) as f32 * 1000.0
                    / self.env.ticks_per_second as f32
                    * p.speed
        }
    }

    fn with_play(&mut self, id: u32, f: impl FnOnce(&mut Play, f32, u64)) -> bool {
        if let Some(p) = self.style.pending_plays.iter_mut().find(|p| p.id == id) {
            // Not started yet: it starts from what is asked.
            let mut tmp = Play {
                at_ms: p.desc.from_ms,
                speed: 1.0,
                reverse: p.desc.reverse,
                ..Play::default()
            };
            f(&mut tmp, p.desc.from_ms, 0);
            p.desc.from_ms = tmp.at_ms;
            p.desc.reverse = tmp.reverse;
            return true;
        }
        let now = self.interaction.state.tick;
        let Some(k) = self
            .interaction
            .state
            .motion
            .plays
            .iter()
            .position(|p| p.id == id)
        else {
            return false;
        };
        let at = self.play_now(&self.interaction.state.motion.plays[k]);
        f(&mut self.interaction.state.motion.plays[k], at, now);
        true
    }

    /// Moves a play to `ms` (it fires no events on the way).
    pub fn seek(&mut self, play: u32, ms: f32) -> bool {
        self.with_play(play, |p, _, now| {
            p.at_ms = ms;
            p.last_ms = ms;
            p.start = now;
            p.ended = false;
        })
    }

    /// Holds a play where it is.
    pub fn pause(&mut self, play: u32) -> bool {
        self.with_play(play, |p, at, now| {
            p.at_ms = at;
            p.start = now;
            p.paused = true;
        })
    }

    pub fn resume(&mut self, play: u32) -> bool {
        self.with_play(play, |p, at, now| {
            p.at_ms = at;
            p.start = now;
            p.paused = false;
        })
    }

    /// Turns a play round where it is (an interrupted opening closes from
    /// where it got to).
    pub fn reverse(&mut self, play: u32) -> bool {
        self.with_play(play, |p, at, now| {
            let len = p.length_ms;
            let shown = if p.reverse {
                len - at.clamp(0.0, len)
            } else {
                at.clamp(0.0, len)
            };
            p.reverse = !p.reverse;
            p.at_ms = if p.reverse { len - shown } else { shown };
            p.last_ms = p.at_ms;
            p.start = now;
            p.ended = false;
        })
    }

    /// Stops a play: its elements go back to their own values (on their
    /// motion, if they have one).
    pub fn stop(&mut self, play: u32) -> bool {
        let had = self.style.pending_plays.len() + self.interaction.state.motion.plays.len();
        self.style.pending_plays.retain(|p| p.id != play);
        self.interaction.state.motion.plays.retain(|p| p.id != play);
        had != self.style.pending_plays.len() + self.interaction.state.motion.plays.len()
    }

    pub fn play_info(&self, play: u32) -> Option<PlayInfo> {
        let p = self
            .interaction
            .state
            .motion
            .plays
            .iter()
            .find(|p| p.id == play)?;
        Some(PlayInfo {
            at_ms: self.play_now(p),
            length_ms: p.length_ms,
            paused: p.paused,
            reverse: p.reverse,
            ended: p.ended,
        })
    }

    /// Forces a state group of the element at `path` (the probe and agents
    /// render and check each state this way); `None` lets it go.
    pub fn force_state(&mut self, path: &str, group: &str, state: Option<&str>) {
        let gk = group_key(crate::id::key_of_path(path), group);
        match state {
            Some(s) => {
                self.interaction
                    .state
                    .motion
                    .forced
                    .insert(gk, s.to_owned());
            }
            None => {
                self.interaction.state.motion.forced.remove(&gk);
            }
        }
    }

    /// The state a group of the element at `path` is in (last frame).
    pub fn visual_state(&self, path: &str, group: &str) -> Option<String> {
        let gk = group_key(crate::id::key_of_path(path), group);
        self.interaction
            .state
            .motion
            .groups
            .get(&gk)
            .map(|g| g.state.clone())
    }

    /// Any number on a spring or an ease (docs/ui.md §3.1: every value is
    /// animatable): `id` names it under the open element; call it every
    /// tick with the target, it answers the value this tick. A retarget
    /// keeps its value and velocity. Under reduced motion it jumps.
    pub fn animate(&mut self, id: &str, target: f32, transition: Transition) -> f32 {
        let key = child_key(self.key_for(id), "~animate");
        self.animate_key(key, target, transition, None)
    }

    /// A channel by key moved towards `target` once this tick; `from` is
    /// where a new one starts (`None`: at its target).
    pub(crate) fn animate_key(
        &mut self,
        key: u64,
        target: f32,
        transition: Transition,
        from: Option<f32>,
    ) -> f32 {
        let now = self.interaction.state.tick;
        let tps = self.env.ticks_per_second;
        let tokens = self.env.theme.motion;
        let t = if self.env.theme.reduced_motion {
            Transition::Instant
        } else {
            transition
        };
        let (kind, mode) = pack(&t, &tokens);
        self.style.motion.widget_channels.push(key);
        let ch = self
            .interaction
            .state
            .motion
            .channels
            .entry(key)
            .or_insert_with(|| {
                let mut c = Channel::at(from.unwrap_or(target), now.saturating_sub(1));
                c.stepped = now.saturating_sub(1);
                c
            });
        if ch.target != target || ch.kind != kind {
            ch.retarget(target, kind, mode, now);
        }
        ch.step(now, tps, false)
    }

    /// This frame's motion events (last tick's, while building).
    pub fn events(&self) -> &[UiEvent] {
        &self.out.frame.events
    }
}

impl Ui {
    /// Shared elements' places this tick (where the next screen's element
    /// with the same key morphs from).
    pub(crate) fn record_matches(&mut self, order: &[usize]) {
        let now = self.interaction.state.tick;
        let m = &mut self.interaction.state.motion;
        for &i in order {
            let n = &self.tree.nodes[i];
            if n.props.match_key.is_empty() {
                continue;
            }
            m.matches.insert(
                name_hash(&n.props.match_key),
                Matched {
                    key: n.key,
                    rect: n.screen,
                    radius: n.sampled.radius,
                    tick: now,
                },
            );
        }
        m.matches.retain(|_, v| v.tick + 1 >= now);
    }

    /// Drop targets and dragged elements of this frame, by key.
    pub(crate) fn note_drag_and_drop(&mut self, order: &[usize]) {
        self.interaction.drop_paths.clear();
        self.interaction.drag_paths.clear();
        self.interaction.drop_accepts.clear();
        for &i in order {
            let n = &self.tree.nodes[i];
            if let Some(d) = &n.props.drop {
                let key = n.key;
                let accept = d.accept.clone();
                let path = self.tree.path_of(i);
                self.interaction.drop_paths.insert(key, path);
                self.interaction.drop_accepts.insert(key, accept);
            }
            if self.tree.nodes[i].props.drag.is_some() {
                let key = self.tree.nodes[i].key;
                let path = self.tree.path_of(i);
                self.interaction.drag_paths.insert(key, path);
            }
        }
    }

    /// Elements in states or moving, and leaving ones, for `dump_ui`.
    pub(crate) fn motion_dump_of(&self, order: &[usize]) -> Vec<MotionDump> {
        let m = &self.interaction.state.motion;
        let now = self.interaction.state.tick;
        let tps = self.env.ticks_per_second;
        let names = |c: usize| -> &'static str {
            match c {
                c if c < CHANNELS => Prop::ALL
                    .iter()
                    .find(|p| p.channels() == [c])
                    .map_or("", |p| p.name()),
                c if c == FLIP_X || c == FLIP_Y || c == FLIP_SX || c == FLIP_SY => "layout",
                _ => "drag",
            }
        };
        let mut out = Vec::new();
        for &i in order {
            let n = &self.tree.nodes[i];
            let p = &n.props;
            // A presence in flight (a layer's or a transition's entrance,
            // a stagger, a glitch): `presence`, and `glitch` while it tears.
            let presence = n.fx.filter(|f| {
                f.alpha < 1.0 || f.glitch > 0.0 || f.dx != 0.0 || f.dy != 0.0 || f.scale != 1.0
            });
            if !p.animated() && p.layout_motion.is_none() && p.drag.is_none() && presence.is_none()
            {
                continue;
            }
            let states: Vec<(String, String)> = p
                .states
                .iter()
                .map(|g| {
                    let s = m
                        .groups
                        .get(&group_key(n.key, &g.name))
                        .map_or(String::new(), |s| s.state.clone());
                    (g.name.clone(), s)
                })
                .collect();
            let mut moving: Vec<String> = Vec::new();
            if let Some(f) = presence {
                moving.push("presence".to_owned());
                if f.glitch > 0.0 {
                    moving.push("glitch".to_owned());
                }
            }
            for c in 0..CHANNELS + 6 {
                if let Some(ch) = m.channels.get(&channel_key(n.key, c))
                    && !ch.done(now, tps)
                {
                    let name = names(c).to_owned();
                    if !name.is_empty() && !moving.contains(&name) {
                        moving.push(name);
                    }
                }
            }
            let plays: Vec<u32> = m
                .plays
                .iter()
                .filter(|pl| pl.root == n.key || pl.clips.iter().any(|c| c.target == n.key))
                .map(|pl| pl.id)
                .collect();
            if states.is_empty() && moving.is_empty() && plays.is_empty() {
                continue;
            }
            out.push(MotionDump {
                path: self.tree.path_of(i),
                states,
                moving,
                plays,
                leaving: false,
            });
        }
        for g in &m.ghosts {
            out.push(MotionDump {
                path: g.frozen.path.clone(),
                leaving: true,
                moving: g
                    .channels
                    .iter()
                    .map(|(c, _)| names(*c as usize).to_owned())
                    .collect(),
                ..MotionDump::default()
            });
        }
        out
    }
}
