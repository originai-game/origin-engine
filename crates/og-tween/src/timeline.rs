//! GSAP-shaped timelines (`og.tween.timeline`): tweens, calls, labels and
//! nested timelines placed at times of the timeline's own clock.
//!
//! A timeline is rendered from its playhead, never accumulated: every run
//! of `og.tween` moves the playhead (by scaled time, its `timeScale`, and
//! backwards while reversed) and writes each child's value at that time, so
//! a seek, a reverse or a restore reaches the values a straight play does.
//! Children render in order (in reverse order while the playhead moves
//! back), a child is written only when its own time changed, and a child
//! the playhead has never reached is left alone, as GSAP's are.
//!
//! Callbacks are names: crossing a call, or a tween or timeline starting,
//! updating, completing or repeating, sends `og.TweenEvent` with the name
//! (no closure runs in the tick). A seek jumps without sending them.

use crate::{Ease, Leaf};

/// GSAP's `onStart`, `onUpdate`, `onComplete` and `onRepeat`, as the names
/// of the `og.TweenEvent`s they send.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Callbacks {
    pub start: Option<String>,
    pub update: Option<String>,
    pub complete: Option<String>,
    pub repeat: Option<String>,
}

impl Callbacks {
    pub(crate) fn names(&self) -> [&Option<String>; 4] {
        [&self.start, &self.update, &self.complete, &self.repeat]
    }
}

/// What a child is.
#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    /// A tween of one target (its delay folded into the child's start).
    Tween(Leaf),
    /// `.call(name)`: sends `og.TweenEvent` `name` when the playhead
    /// crosses it, either way.
    Call(String),
    /// A nested timeline (keyframes, or `.add` of a timeline or a group).
    Timeline(Box<Timeline>),
}

/// A child placed at `start` seconds of its timeline.
#[derive(Clone, Debug, PartialEq)]
pub struct Child {
    pub start: f64,
    pub item: Item,
    /// A tween's callbacks (a nested timeline carries its own).
    pub on: Callbacks,
    /// `from` and `fromTo`: the start value is written on the timeline's
    /// first render, before the child's time (GSAP's immediateRender).
    pub immediate: bool,
    /// The child's own time last rendered: `None` never, below 0 before
    /// its start.
    pub(crate) last: Option<f64>,
}

impl Child {
    pub fn new(start: f64, item: Item) -> Child {
        Child {
            start,
            item,
            on: Callbacks::default(),
            immediate: false,
            last: None,
        }
    }

    /// Seconds it lasts on its timeline (infinite for a tween or timeline
    /// that repeats forever).
    pub fn length(&self) -> f64 {
        match &self.item {
            Item::Tween(l) => f64::from(l.length()),
            Item::Call(_) => 0.0,
            Item::Timeline(t) => t.total_duration(),
        }
    }

    pub(crate) fn leaves(&self) -> Vec<&Leaf> {
        match &self.item {
            Item::Tween(l) => vec![l],
            Item::Call(_) => Vec::new(),
            Item::Timeline(t) => t.leaves(),
        }
    }

    pub(crate) fn leaves_mut(&mut self) -> Vec<&mut Leaf> {
        match &mut self.item {
            Item::Tween(l) => vec![l],
            Item::Call(_) => Vec::new(),
            Item::Timeline(t) => t.leaves_mut(),
        }
    }
}

/// Where a child goes: seconds, or GSAP's position text (`"<"`, `">"`,
/// `"+=1"`, `"-=0.5"`, `"intro"`, `"intro+=0.2"`, `"<50%"`).
#[derive(Clone, Debug, PartialEq)]
pub enum Position {
    At(f64),
    Text(String),
}

/// What a render reads, writes and sends.
pub(crate) trait Io {
    /// Captures a tween's start and end on its first render; `false` when
    /// its target is gone.
    fn begin(&mut self, leaf: &mut Leaf) -> bool;
    fn write(&mut self, leaf: &Leaf, value: &[f32]);
    /// An `og.TweenEvent` named `name` (empty names send nothing).
    fn fire(&mut self, name: &Option<String>, entity: u64, progress: f32);
}

/// A timeline: its children and labels, its repeat, and its playhead.
#[derive(Clone, Debug, PartialEq)]
pub struct Timeline {
    pub children: Vec<Child>,
    pub labels: Vec<(String, f64)>,
    /// Extra cycles: 0 once, -1 forever.
    pub repeat: i32,
    pub yoyo: bool,
    pub repeat_delay: f32,
    /// Playback rate of a top-level timeline (a nested one plays at its
    /// parent's pace).
    pub time_scale: f32,
    pub paused: bool,
    pub reversed: bool,
    /// Removed when it completes, instead of staying to be reversed,
    /// restarted or extended.
    pub auto_kill: bool,
    pub on: Callbacks,
    /// The playhead over every cycle.
    pub(crate) total: f64,
    /// The playhead last rendered (`None`: never).
    pub(crate) last: Option<f64>,
    /// The local time last rendered.
    pub(crate) last_local: Option<f64>,
    /// The next render jumps without sending callbacks (a seek).
    pub(crate) quiet: bool,
    /// Something was added since the last render.
    pub(crate) dirty: bool,
    /// The span of what was added last (`"<"` and `">"`).
    pub(crate) recent: Option<(f64, f64)>,
}

impl Default for Timeline {
    fn default() -> Self {
        Timeline {
            children: Vec::new(),
            labels: Vec::new(),
            repeat: 0,
            yoyo: false,
            repeat_delay: 0.0,
            time_scale: 1.0,
            paused: false,
            reversed: false,
            auto_kill: false,
            on: Callbacks::default(),
            total: 0.0,
            last: None,
            last_local: None,
            quiet: false,
            dirty: false,
            recent: None,
        }
    }
}

/// Within a microsecond: a playhead that reaches an end by a sum of tick
/// lengths still counts as there.
const EPS: f64 = 1e-6;

impl Timeline {
    /// One cycle: where the last child ends (0 when empty).
    pub fn duration(&self) -> f64 {
        self.children
            .iter()
            .map(|c| c.start + c.length())
            .fold(0.0, f64::max)
    }

    /// Every cycle and the holds between them; infinite when it repeats
    /// forever or holds a child that does.
    pub fn total_duration(&self) -> f64 {
        let d = self.duration();
        if self.repeat < 0 && d > 0.0 {
            return f64::INFINITY;
        }
        let r = f64::from(self.repeat.max(0));
        d * (r + 1.0) + f64::from(self.repeat_delay) * r
    }

    /// The playhead over every cycle, in seconds.
    pub fn total_time(&self) -> f64 {
        self.total
    }

    /// The cycle and the local time (yoyo applied) of playhead `total`.
    fn cycle_at(&self, total: f64) -> (i64, f64) {
        let d = self.duration();
        if d <= 0.0 || !d.is_finite() {
            return (0, total.min(d.max(0.0)));
        }
        let cycle = d + f64::from(self.repeat_delay);
        let td = self.total_duration();
        let (k, t) = if td.is_finite() && total + EPS >= td {
            (i64::from(self.repeat.max(0)), d)
        } else {
            let k = (total / cycle).floor().max(0.0);
            (k as i64, (total - k * cycle).min(d))
        };
        if self.yoyo && k % 2 == 1 {
            (k, d - t)
        } else {
            (k, t)
        }
    }

    /// The local time of the playhead (within its cycle).
    pub fn time(&self) -> f64 {
        self.cycle_at(self.total).1
    }

    /// How far through its cycle the playhead is, 0 to 1.
    pub fn progress(&self) -> f64 {
        let d = self.duration();
        if d <= 0.0 || !d.is_finite() {
            return if self.total > 0.0 { 1.0 } else { 0.0 };
        }
        self.time() / d
    }

    /// Moves the playhead to `p` of its current cycle, without callbacks.
    pub fn set_progress(&mut self, p: f64) {
        let d = self.duration();
        if !d.is_finite() {
            return;
        }
        let p = p.clamp(0.0, 1.0);
        let (k, _) = self.cycle_at(self.total);
        let t = if self.yoyo && k % 2 == 1 { 1.0 - p } else { p } * d;
        self.seek_total(k as f64 * (d + f64::from(self.repeat_delay)) + t);
    }

    /// Moves the playhead to `total` (clamped to the timeline), without
    /// callbacks; the next run of `og.tween` renders it.
    pub fn seek_total(&mut self, total: f64) {
        self.total = total.clamp(0.0, self.total_duration());
        self.quiet = true;
        self.dirty = true;
    }

    /// Whether it plays: not paused, and not held at the end it moves
    /// towards.
    pub fn is_active(&self) -> bool {
        !self.paused
            && if self.reversed {
                self.total > 0.0
            } else {
                self.total + EPS < self.total_duration()
            }
    }

    /// Back to its start as if never played (a timeline nested by `.add`
    /// plays from its start on its new parent's clock).
    pub fn rewind(&mut self) {
        self.total = 0.0;
        self.last = None;
        self.last_local = None;
        self.paused = false;
        self.reversed = false;
        self.quiet = false;
        self.dirty = true;
        for c in &mut self.children {
            c.last = None;
            if let Item::Timeline(t) = &mut c.item {
                t.rewind();
            }
        }
    }

    /// The time `position` names, for something lasting `inserting`
    /// seconds. A label it names that does not exist is added at the end,
    /// as GSAP does.
    pub fn resolve(&mut self, position: Option<&Position>, inserting: f64) -> Result<f64, String> {
        let end = self.duration();
        let text = match position {
            None => return Ok(end),
            Some(Position::At(t)) => {
                if !t.is_finite() {
                    return Err(format!("position {t} is not a finite time"));
                }
                return Ok(t.max(0.0));
            }
            Some(Position::Text(s)) => s.trim().to_string(),
        };
        let amount = |s: &str, whole: f64| -> Result<f64, String> {
            let s = s.trim();
            let (n, pct) = match s.strip_suffix('%') {
                Some(n) => (n, true),
                None => (s, false),
            };
            let v: f64 = n
                .trim()
                .parse()
                .ok()
                .filter(|v: &f64| v.is_finite())
                .ok_or_else(|| {
                    format!("position \"{text}\": '{s}' is not a number of seconds or a percentage")
                })?;
            Ok(if pct { v / 100.0 * whole } else { v })
        };
        // An offset after a base: `+=n`, `-=n` (percentages of what is
        // inserted), or after `<` and `>` a signed number (percentages of
        // the recent child).
        let offset = |rest: &str, signed_whole: f64| -> Result<f64, String> {
            let rest = rest.trim();
            if rest.is_empty() {
                Ok(0.0)
            } else if let Some(r) = rest.strip_prefix("+=") {
                amount(r, inserting)
            } else if let Some(r) = rest.strip_prefix("-=") {
                Ok(-amount(r, inserting)?)
            } else {
                amount(rest, signed_whole)
            }
        };
        let at = if let Some(rest) = text.strip_prefix('<') {
            let (start, stop) = self.recent.unwrap_or((0.0, 0.0));
            start + offset(rest, stop - start)?
        } else if let Some(rest) = text.strip_prefix('>') {
            let (start, stop) = self.recent.unwrap_or((end, end));
            stop + offset(rest, stop - start)?
        } else if text.starts_with("+=") || text.starts_with("-=") {
            end + offset(&text, 0.0)?
        } else if let Ok(t) = text.parse::<f64>() {
            if !t.is_finite() {
                return Err(format!("position \"{text}\" is not a finite time"));
            }
            t
        } else {
            let (name, rest) = match text.find("+=").or_else(|| text.find("-=")) {
                Some(i) => (text[..i].trim(), &text[i..]),
                None => (text.as_str(), ""),
            };
            if name.is_empty() {
                return Err(format!(
                    "position \"{text}\" is seconds, \"<\", \">\", \"+=n\", \"-=n\", a label or \"label+=n\""
                ));
            }
            let base = match self.labels.iter().find(|(n, _)| n == name) {
                Some((_, t)) => *t,
                None => {
                    self.labels.push((name.to_string(), end));
                    end
                }
            };
            base + offset(rest, 0.0)?
        };
        Ok(at.max(0.0))
    }

    /// Adds children (one call's: a tween's staggered targets, its
    /// keyframes); they become the recent span.
    pub fn insert(&mut self, children: Vec<Child>) {
        if children.is_empty() {
            return;
        }
        let start = children
            .iter()
            .map(|c| c.start)
            .fold(f64::INFINITY, f64::min);
        let stop = children
            .iter()
            .map(|c| c.start + c.length())
            .fold(f64::NEG_INFINITY, f64::max);
        self.recent = Some((start, stop));
        self.children.extend(children);
        self.dirty = true;
    }

    /// Adds or moves label `name` to `at`.
    pub fn add_label(&mut self, name: &str, at: f64) {
        match self.labels.iter_mut().find(|(n, _)| n == name) {
            Some(l) => l.1 = at,
            None => self.labels.push((name.to_string(), at)),
        }
    }

    pub(crate) fn leaves(&self) -> Vec<&Leaf> {
        self.children.iter().flat_map(|c| c.leaves()).collect()
    }

    pub(crate) fn leaves_mut(&mut self) -> Vec<&mut Leaf> {
        self.children
            .iter_mut()
            .flat_map(|c| c.leaves_mut())
            .collect()
    }

    /// Drops every tween of `entity` (nested ones too); how many.
    pub(crate) fn drop_entity(&mut self, entity: u64) -> u32 {
        let mut n = 0;
        self.children.retain_mut(|c| match &mut c.item {
            Item::Tween(l) if l.target.entity == entity => {
                n += 1;
                false
            }
            Item::Timeline(t) => {
                n += t.drop_entity(entity);
                true
            }
            _ => true,
        });
        n
    }

    /// The entity of the first tween (the time group a top-level timeline
    /// runs in).
    pub(crate) fn entity(&self) -> u64 {
        self.leaves().first().map_or(0, |l| l.target.entity)
    }

    /// One run of `og.tween` for a top-level timeline: the playhead moves
    /// by `dt` and what changed is rendered. `true` when it completed in
    /// this run.
    pub(crate) fn step(&mut self, io: &mut dyn Io, dt: f32) -> bool {
        let td = self.total_duration();
        if !self.paused && dt > 0.0 {
            let dir = if self.reversed { -1.0 } else { 1.0 };
            self.total =
                (self.total + f64::from(dt) * f64::from(self.time_scale) * dir).clamp(0.0, td);
        }
        let mut completed = false;
        if self.last != Some(self.total) || self.dirty {
            // The first render of a timeline still at 0 (paused, or not
            // yet moved) only writes its from() values.
            let events = !self.quiet && !(self.last.is_none() && self.total == 0.0);
            completed = self.render(io, self.total, events);
        }
        self.quiet = false;
        self.dirty = false;
        completed
    }

    /// Renders playhead `to`, sending callbacks when `events`; `true` when
    /// it reached its end going forward.
    pub(crate) fn render(&mut self, io: &mut dyn Io, to: f64, events: bool) -> bool {
        let td = self.total_duration();
        let d = self.duration();
        let prev = self.last;
        let forward = prev.is_none_or(|p| to >= p);
        let entity = self.entity();
        let progress = |t: &Timeline| t.progress() as f32;
        if events && forward && prev.is_none_or(|p| p <= 0.0) && to > 0.0 {
            io.fire(&self.on.start, entity, 0.0);
        }
        // Across a cycle boundary: finish the cycle left, then jump to the
        // new one's start without callbacks, as GSAP does on a repeat.
        if let Some(p) = prev
            && d > 0.0
            && d.is_finite()
        {
            let (k0, _) = self.cycle_at(p);
            let (k1, _) = self.cycle_at(to);
            if k1 != k0 {
                let end_of = |k: i64| if self.yoyo && k % 2 == 1 { 0.0 } else { d };
                let start_of = |k: i64| if self.yoyo && k % 2 == 1 { d } else { 0.0 };
                let (edge, enter) = if forward {
                    (end_of(k0), start_of(k1))
                } else {
                    (start_of(k0), end_of(k1))
                };
                self.render_local(io, edge, events);
                if events && forward {
                    io.fire(&self.on.repeat, entity, progress(self));
                }
                self.render_local(io, enter, false);
            }
        }
        let (_, t) = self.cycle_at(to);
        self.render_local(io, t, events);
        self.last = Some(to);
        self.total = to;
        if events {
            io.fire(&self.on.update, entity, progress(self));
        }
        let done = forward
            && td.is_finite()
            && td > 0.0
            && to + EPS >= td
            && prev.is_none_or(|p| p + EPS < td);
        if done && events {
            io.fire(&self.on.complete, entity, 1.0);
        }
        done
    }

    /// Renders every child at local time `t`.
    fn render_local(&mut self, io: &mut dyn Io, t: f64, events: bool) {
        let back = self.last_local.is_some_and(|p| t < p);
        let n = self.children.len();
        for j in 0..n {
            let i = if back { n - 1 - j } else { j };
            let child = &mut self.children[i];
            let lt = t - child.start;
            match &mut child.item {
                Item::Call(name) => {
                    let crossed = match child.last {
                        None => lt >= 0.0,
                        Some(p) => (p < 0.0 && lt >= 0.0) || (lt < 0.0 && p >= 0.0),
                    };
                    if events && crossed {
                        io.fire(&Some(name.clone()), 0, 1.0);
                    }
                    // A jump that lands exactly on a call leaves it to fire
                    // when the playhead next moves forward over it.
                    child.last = Some(if !events && lt == 0.0 { -EPS } else { lt });
                }
                Item::Tween(leaf) => {
                    if child.last.is_none() && lt < 0.0 && !child.immediate {
                        continue;
                    }
                    let len = f64::from(leaf.length());
                    // Below 0: before its start, its start value.
                    let c = if lt < 0.0 { -1.0 } else { lt.min(len) };
                    if child.last == Some(c) {
                        continue;
                    }
                    if !leaf.started && !io.begin(leaf) {
                        child.last = Some(c);
                        continue;
                    }
                    let value = if c < 0.0 {
                        leaf.start.clone()
                    } else {
                        leaf.elapsed = c;
                        leaf.value()
                    };
                    io.write(leaf, &value);
                    if events {
                        let entity = leaf.target.entity;
                        let p = child.last;
                        let progress = if len > 0.0 && len.is_finite() {
                            (c.max(0.0) / len) as f32
                        } else {
                            1.0
                        };
                        let from_start = p.is_none_or(|p| p <= 0.0);
                        if from_start
                            && (c > 0.0 || (len == 0.0 && c >= 0.0 && p.is_none_or(|p| p < 0.0)))
                        {
                            io.fire(&child.on.start, entity, progress);
                        }
                        io.fire(&child.on.update, entity, progress);
                        let cycle = |x: f64| {
                            let len = f64::from(leaf.duration + leaf.repeat_delay);
                            if len > 0.0 {
                                (x.max(0.0) / len).floor() as i64
                            } else {
                                0
                            }
                        };
                        if let Some(p) = p
                            && c < len
                            && c > p
                            && cycle(c) > cycle(p)
                        {
                            io.fire(&child.on.repeat, entity, progress);
                        }
                        if c >= len && p.is_none_or(|p| p < len) {
                            io.fire(&child.on.complete, entity, 1.0);
                        }
                    }
                    child.last = Some(c);
                }
                Item::Timeline(tl) => {
                    if child.last.is_none() && lt < 0.0 {
                        continue;
                    }
                    let c = lt.clamp(0.0, tl.total_duration());
                    if child.last == Some(c) && !tl.dirty {
                        continue;
                    }
                    tl.render(io, c, events);
                    tl.dirty = false;
                    child.last = Some(c);
                }
            }
        }
        self.last_local = Some(t);
    }
}

/// GSAP's `stagger`: the offset of each of `n` targets.
#[derive(Clone, Debug, PartialEq)]
pub struct Stagger {
    /// Seconds between neighbours (by distance from `from`).
    pub each: Option<f64>,
    /// Seconds over all of them (wins over `each`).
    pub amount: Option<f64>,
    pub from: StaggerFrom,
    /// `[rows, columns]`: distances are measured on the grid.
    pub grid: Option<[u32; 2]>,
    /// Distributes the offsets through a curve.
    pub ease: Ease,
}

/// Where a stagger starts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StaggerFrom {
    Start,
    Center,
    End,
    /// Both ends first, the middle last.
    Edges,
    /// A seeded shuffle of the targets.
    Random,
    Index(u32),
}

impl Stagger {
    pub fn each(s: f64) -> Stagger {
        Stagger {
            each: Some(s),
            amount: None,
            from: StaggerFrom::Start,
            grid: None,
            ease: Ease::Linear,
        }
    }

    /// The offset of each of `n` targets, in seconds. `shuffle` answers a
    /// permutation of `0..n` (the engine's seeded randomness) for `random`.
    pub fn offsets(&self, n: usize, shuffle: &mut dyn FnMut(usize) -> Vec<usize>) -> Vec<f64> {
        if n == 0 {
            return Vec::new();
        }
        let distances: Vec<f64> = match self.from {
            StaggerFrom::Random => {
                let mut d = vec![0.0; n];
                for (rank, i) in shuffle(n).into_iter().enumerate() {
                    if let Some(x) = d.get_mut(i) {
                        *x = rank as f64;
                    }
                }
                d
            }
            from => {
                let cols = self.grid.map_or(n, |g| g[1].max(1) as usize) as f64;
                let rows = self.grid.map_or(1.0, |g| f64::from(g[0].max(1)));
                let cell = |i: usize| ((i as f64) % cols, (i as f64 / cols).floor());
                let (ox, oy) = match from {
                    StaggerFrom::Start => (0.0, 0.0),
                    StaggerFrom::End => (cols - 1.0, rows - 1.0),
                    StaggerFrom::Center | StaggerFrom::Edges => {
                        ((cols - 1.0) / 2.0, (rows - 1.0) / 2.0)
                    }
                    StaggerFrom::Index(i) => cell(i as usize),
                    StaggerFrom::Random => unreachable!(),
                };
                let d: Vec<f64> = (0..n)
                    .map(|i| {
                        let (x, y) = cell(i);
                        ((x - ox).powi(2) + (y - oy).powi(2)).sqrt()
                    })
                    .collect();
                if from == StaggerFrom::Edges {
                    let max = d.iter().copied().fold(0.0, f64::max);
                    d.iter().map(|x| max - x).collect()
                } else {
                    d
                }
            }
        };
        let max = distances.iter().copied().fold(0.0, f64::max);
        let whole = match (self.amount, self.each) {
            (Some(a), _) => a,
            (None, Some(e)) => e * max,
            (None, None) => 0.0,
        };
        distances
            .iter()
            .map(|d| {
                if max > 0.0 {
                    f64::from(self.ease.apply((d / max) as f32)) * whole
                } else {
                    0.0
                }
            })
            .collect()
    }
}
