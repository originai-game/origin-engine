//! UI motion on rune-motion: transitions of a record's skin, presence
//! (enter from a `starting:` record, exit to an `exit:` one while the
//! element stays drawn) and layout animation of the arena's boxes.
//!
//! Time is an input: [`Motion::tick_at`] takes the UI tick's time, and
//! nothing here reads a clock. The frame calls `tick_at` first, then
//! applies the frame's changes ([`Motion::enter`], [`Motion::restyle`],
//! [`Motion::exit`]), which start at that time, then lays out and calls
//! [`Motion::relayout`], and paint reads [`Motion::skin`] and
//! [`Motion::box_of`]. Values are sampled once per frame into one skin per
//! moving element; an element with nothing moving costs paint one index.
//! Paint never writes back.
//!
//! Transitions animate what Tailwind's `transition-*` groups name, on the
//! element's `duration-*`, `delay-*` and `ease-*` (a cubic Bézier, or
//! NSH's `ease-spring-[…]`). A change that arrives mid-flight retargets
//! from where the value is, keeping its velocity. Colours move in
//! premultiplied OKLab. `animate-*` keyframes are closed forms of time,
//! applied over the transitioned skin. Under reduced motion every
//! movement (transform, layout) jumps; opacity and colour still fade.

use rune_motion::{
    Anim, AnimExt, Animatable, AnimationStore, Curve, Easing, NodeId, Prop, PropClass, Rect,
    Spring, linear_to_oklab, oklab_to_linear, prop,
};
use rune_taffy::BoxLayout;

use crate::arena::Id;
use crate::record::{Record, Rgba, SHADOWS, Shadow, Skin, TEXT_SHADOWS};

/// `transition-colors`: fill, ink, border and outline colours.
pub const COLORS: u8 = 1;
/// `transition-opacity`.
pub const OPACITY: u8 = 2;
/// `transition-transform`: translate, scale, rotate.
pub const TRANSFORM: u8 = 4;
/// `transition-shadow`: box and text shadows, and the backdrop blur.
pub const SHADOW: u8 = 8;
/// Corner radii (in `transition` and `transition-all`).
pub const SHAPE: u8 = 16;
/// `transition-layout` (ours): a box that moves or resizes slides there.
/// Not in `transition-all`.
pub const LAYOUT: u8 = 32;
/// `transition-all` (and CSS's initial `transition-property`).
pub const ALL: u8 = COLORS | OPACITY | TRANSFORM | SHADOW | SHAPE;

/// The curve a transition runs on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ease {
    /// `cubic-bezier(x1, y1, x2, y2)`, over `duration`.
    Bezier([f32; 4]),
    /// NSH's `ease-spring-[stiffness,damping,mass]`: it times itself and
    /// `duration` is not read.
    Spring {
        stiffness: f32,
        damping: f32,
        mass: f32,
    },
}

impl Ease {
    /// CSS's `ease`.
    pub const EASE: Ease = Ease::Bezier([0.25, 0.1, 0.25, 1.0]);
}

mod channels;
mod keyframes;
use channels::*;
pub use keyframes::{Keyframes, Loop};
use keyframes::{bezier, pose};

/// What a class list asks of motion: `Record::motion`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spec {
    /// The groups that transition: [`COLORS`], [`OPACITY`], … [`LAYOUT`].
    pub props: u8,
    /// Seconds.
    pub duration: f32,
    /// Seconds.
    pub delay: f32,
    pub ease: Ease,
    pub animate: Option<Loop>,
    /// The transform's own curve and duration (seconds), over `props`,
    /// `ease` and `duration`: the pack's press feedback (`Press.feedback`),
    /// which the tree gives a control that answers a press, so its squash
    /// releases on the pack's spring while its colours keep their own.
    pub transform: Option<(Ease, f32)>,
}

impl Default for Spec {
    /// Nothing moves.
    fn default() -> Self {
        Spec {
            props: 0,
            duration: 0.0,
            delay: 0.0,
            ease: Ease::EASE,
            animate: None,
            transform: None,
        }
    }
}

impl Spec {
    /// The animation a change runs on, or none when it jumps.
    fn anim(&self) -> Option<Anim> {
        Self::curve(self.ease, self.duration, self.delay)
    }

    /// The animation channel group `group` runs on, or none when it jumps.
    fn anim_of(&self, group: u8) -> Option<Anim> {
        match self.transform {
            Some((ease, duration)) if group == TRANSFORM => Self::curve(ease, duration, 0.0),
            _ if self.props & group != 0 => self.anim(),
            _ => None,
        }
    }

    fn curve(ease: Ease, duration: f32, delay: f32) -> Option<Anim> {
        let curve = match ease {
            Ease::Spring {
                stiffness,
                damping,
                mass,
            } => Curve::Spring(Spring::from_stiffness_damping(stiffness, damping, mass)),
            Ease::Bezier([a, b, c, d]) if duration > 0.0 => Curve::Tween {
                duration,
                easing: Easing::cubic(a, b, c, d),
            },
            Ease::Bezier(_) => return None,
        };
        Some(Anim::new(curve).delay(delay.max(0.0)))
    }
}

/// An element motion holds state for: one with a track in flight, a
/// keyframe animation, a layout transition, or an exit under way.
#[derive(Clone, Debug)]
struct Live {
    id: Id,
    spec: Spec,
    /// The skin it rests on: its record's.
    base: Skin,
    /// What paint draws this frame.
    shown: Skin,
    /// Channels with a track, and [`FRAME_BIT`].
    tracks: u32,
    /// Its last laid-out box, parent space.
    laid: Option<BoxLayout>,
    /// The box paint draws while its layout moves.
    frame: BoxLayout,
    /// Its keyframes' start, NaN for none playing, or infinity for the
    /// next sample's time: a loop set between ticks starts at the tick
    /// that first shows it, not at the store's last sample, which may be
    /// long past (an idle HUD's damage number then started finished).
    since: f64,
    /// Its keyframes hold a pose away from `base` (a running loop, or a
    /// finished one that stays where it ended).
    posed: bool,
    /// Its keyframes played at the last sample.
    playing: bool,
    exiting: bool,
    /// When its exit logically ends (the store's time): what replaces it
    /// enters then.
    ends: f64,
    /// When its entrance starts after a wait ([`Motion::enter_after`]), so
    /// entering again before it is drawn (a new record) keeps the wait.
    starts: f64,
    /// A one-shot pose over whatever its record plays ([`Motion::cue`]),
    /// and when it started (infinity: at the next sample).
    cue: Option<(Loop, f64)>,
}

/// The UI's motion: what moves, sampled once per frame.
pub struct Motion {
    store: AnimationStore,
    /// Arena id → index in `live`, or [`NONE`].
    slot: Vec<u32>,
    live: Vec<Live>,
    exited: Vec<Id>,
    reduced: bool,
}

impl Default for Motion {
    fn default() -> Self {
        Self::new()
    }
}

impl Motion {
    pub fn new() -> Motion {
        Motion {
            store: AnimationStore::new(),
            slot: Vec::new(),
            live: Vec::new(),
            exited: Vec::new(),
            reduced: false,
        }
    }

    /// The player's reduced-motion setting: movement jumps, fades stay.
    pub fn set_reduced_motion(&mut self, on: bool) {
        self.reduced = on;
        self.store.set_reduced_motion(on);
    }

    /// Whether anything is still on its way to rest: a transition or layout
    /// move in flight, an exit under way, or keyframes that end playing
    /// (an entrance's pop) or not yet started. A loop that never ends (a
    /// spinner, a shimmer) does not count: a frame of it is as settled as
    /// any other.
    pub fn settling(&self) -> bool {
        self.live.iter().any(|l| {
            l.tracks != 0
                || l.exiting
                || l.cue.is_some()
                || l.spec.animate.is_some_and(|a| {
                    a.iterations.is_some() && (l.playing || l.since == f64::INFINITY)
                })
        })
    }

    /// Advances to the UI tick's time `t` (seconds) and samples what moves.
    /// Call it first in the frame. True when anything moved since the last
    /// tick, the tick it lands included: paint again even with nothing
    /// dirty.
    pub fn tick_at(&mut self, t: f64) -> bool {
        self.store.tick_at(t);
        let mut moving = false;
        let mut i = 0;
        while i < self.live.len() {
            let l = &mut self.live[i];
            if l.since == f64::INFINITY {
                l.since = t;
            }
            if let Some((_, since)) = l.cue.as_mut().filter(|c| c.1 == f64::INFINITY) {
                *since = t;
            }
            // What moved since the last tick moves this one, landing.
            let was = l.tracks != 0 || l.playing;
            let n = node(l.id);
            let mut bits = l.tracks & !FRAME_BIT;
            while bits != 0 {
                let ch = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                if !self.store.is_animating(n, PROPS[ch]) {
                    self.store.remove(n, PROPS[ch]);
                    l.tracks &= !(1 << ch);
                }
            }
            if l.tracks & FRAME_BIT != 0 {
                if let Some(r) = self.store.get(n, prop::FRAME) {
                    (l.frame.x, l.frame.y, l.frame.w, l.frame.h) = (r.x, r.y, r.w, r.h);
                }
                if !self.store.is_animating(n, prop::FRAME) {
                    self.store.remove(n, prop::FRAME);
                    l.tracks &= !FRAME_BIT;
                }
            }
            let playing = self.present(i);
            let l = &self.live[i];
            moving |= was || l.tracks != 0 || playing;
            if l.exiting && l.tracks == 0 {
                self.exited.push(l.id);
                self.drop_live(i);
            } else if l.tracks == 0 && !l.posed && l.spec.props & LAYOUT == 0 && !playing {
                self.drop_live(i);
            } else {
                i += 1;
            }
        }
        moving
    }

    /// `id` was inserted, resting on `to`. Pass the record its `starting:`
    /// classes give as `from` to have it enter from there; pass none for an
    /// element of the first build, which appears as it is. An element
    /// inserted again while it exits turns back from where it is.
    pub fn enter(&mut self, id: Id, to: &Record, from: Option<&Record>) {
        self.enter_after(id, to, from, 0.0);
    }

    /// [`Motion::enter`], holding its `starting:` record `wait` seconds
    /// first: a screen that replaces one leaving enters once that one has
    /// left ([`Motion::leaves_in`]).
    pub fn enter_after(&mut self, id: Id, to: &Record, from: Option<&Record>, wait: f32) {
        if let Some(i) = self.find(id).filter(|&i| self.live[i].exiting) {
            self.live[i].exiting = false;
            let base = self.live[i].base;
            self.go(id, &to.motion, &base, &to.skin, false);
            return;
        }
        let now = self.store.now();
        let held = self.find(id).map_or(0.0, |i| self.live[i].starts - now);
        let wait = wait.max(held as f32).max(0.0);
        let from = from.map_or(&to.skin, |f| &f.skin);
        let mut spec = to.motion;
        spec.delay += wait;
        if let Some(a) = spec.animate.as_mut() {
            a.delay += wait;
        }
        self.go(id, &spec, from, &to.skin, true);
        if let Some(i) = self.find(id).filter(|_| wait > 0.0) {
            self.live[i].starts = now + wait as f64;
        }
    }

    /// `id`'s record changed from `old` to `new` (a class, a state, the
    /// theme). The new record's transition runs the change, as in CSS.
    pub fn restyle(&mut self, id: Id, old: &Record, new: &Record) {
        let replay =
            new.motion.animate.map(|a| a.keyframes) != old.motion.animate.map(|a| a.keyframes);
        self.go(id, &new.motion, &old.skin, &new.skin, replay);
    }

    /// `id` was removed while resting on `from`, and `to` is the record
    /// its `exit:` classes give, if they give one. True when it animates
    /// out: keep it in place and drawn (not hit) until
    /// [`Motion::take_exited`] lists it. False: free it now.
    pub fn exit(&mut self, id: Id, from: &Record, to: Option<&Record>) -> bool {
        let Some(to) = to else {
            self.forget(id);
            return false;
        };
        self.go(id, &to.motion, &from.skin, &to.skin, false);
        match self.find(id) {
            Some(i) if self.live[i].tracks & !FRAME_BIT != 0 => {
                let m = &to.motion;
                let end = [m.anim(), m.anim_of(TRANSFORM)]
                    .into_iter()
                    .flatten()
                    .map(|a| a.delay + a.curve.logical_duration())
                    .fold(0.0f32, f32::max);
                let l = &mut self.live[i];
                l.exiting = true;
                l.ends = self.store.now() + end as f64;
                true
            }
            _ => {
                self.forget(id);
                false
            }
        }
    }

    /// `id` was freed: what moved it stops.
    pub fn forget(&mut self, id: Id) {
        if let Some(i) = self.find(id) {
            self.drop_live(i);
        }
    }

    /// Seconds until `id`'s exit logically ends (a spring's perceptual
    /// duration, a tween's own); 0 when it is not exiting.
    pub fn leaves_in(&self, id: Id) -> f32 {
        self.find(id)
            .filter(|&i| self.live[i].exiting)
            .map_or(0.0, |i| {
                (self.live[i].ends - self.store.now()).max(0.0) as f32
            })
    }

    /// The player's reduced-motion setting.
    pub fn reduced(&self) -> bool {
        self.reduced
    }

    /// Where a change on `spec`'s curve stands `t` seconds in, from 0 to 1
    /// (a spring passes 1 and comes back), and whether it has come to rest:
    /// a number that follows its text (`transition-count`).
    pub fn progress(spec: &Spec, t: f32) -> (f32, bool) {
        let t = (t - spec.delay).max(0.0);
        match spec.ease {
            Ease::Spring {
                stiffness,
                damping,
                mass,
            } => {
                let s = Spring::from_stiffness_damping(stiffness, damping, mass);
                let rest = t >= s.settling_duration(1e-3);
                (
                    if rest {
                        1.0
                    } else {
                        s.sample(0.0, 0.0, 1.0, t).0
                    },
                    rest,
                )
            }
            Ease::Bezier(e) if spec.duration > 0.0 => {
                let x = t / spec.duration;
                (bezier(e, x), x >= 1.0)
            }
            Ease::Bezier(_) => (1.0, true),
        }
    }

    /// Whether `id` is on its way out (drawn, not hit).
    pub fn exiting(&self, id: Id) -> bool {
        self.find(id).is_some_and(|i| self.live[i].exiting)
    }

    /// The elements whose exit settled since the last call: detach and free
    /// them now.
    pub fn take_exited(&mut self) -> std::vec::Drain<'_, Id> {
        self.exited.drain(..)
    }

    /// After layout, each frame: `box_of` answers an element's laid-out
    /// box. A `transition-layout` element whose box moved slides from where
    /// it was drawn; the first box an element gets it takes as it is.
    pub fn relayout(&mut self, box_of: impl Fn(Id) -> BoxLayout) {
        for i in 0..self.live.len() {
            let l = &mut self.live[i];
            let b = box_of(l.id);
            let Some(was) = l.laid.replace(b) else {
                l.frame = b;
                continue;
            };
            l.frame.inset = b.inset;
            if (was.x, was.y, was.w, was.h) == (b.x, b.y, b.w, b.h) {
                continue;
            }
            let anim = l.spec.anim().filter(|_| l.spec.props & LAYOUT != 0);
            let n = node(l.id);
            match anim {
                Some(anim) => {
                    if l.tracks & FRAME_BIT == 0 {
                        self.store.set(n, prop::FRAME, rect(&was));
                        l.frame = was;
                        l.frame.inset = b.inset;
                    }
                    self.store.animate(n, prop::FRAME, rect(&b), anim);
                    l.tracks |= FRAME_BIT;
                }
                None => {
                    if l.tracks & FRAME_BIT != 0 {
                        self.store.remove(n, prop::FRAME);
                        l.tracks &= !FRAME_BIT;
                    }
                    l.frame = b;
                }
            }
        }
    }

    /// The skin paint draws for `id`: its record's `base`, or what moves it.
    pub fn skin<'a>(&'a self, id: Id, base: &'a Skin) -> &'a Skin {
        match self.find(id).map(|i| &self.live[i]) {
            Some(l) if l.tracks & !FRAME_BIT != 0 || l.posed => &l.shown,
            _ => base,
        }
    }

    /// The box paint draws for `id` while its layout moves (parent space).
    pub fn box_of(&self, id: Id) -> Option<BoxLayout> {
        let l = &self.live[self.find(id)?];
        (l.tracks & FRAME_BIT != 0).then_some(l.frame)
    }

    fn find(&self, id: Id) -> Option<usize> {
        match self.slot.get(id as usize) {
            Some(&s) if s != NONE => Some(s as usize),
            _ => None,
        }
    }

    fn ensure(&mut self, id: Id, base: &Skin) -> usize {
        if let Some(i) = self.find(id) {
            return i;
        }
        if self.slot.len() <= id as usize {
            self.slot.resize(id as usize + 1, NONE);
        }
        self.slot[id as usize] = self.live.len() as u32;
        self.live.push(Live {
            id,
            spec: Spec::default(),
            base: *base,
            shown: *base,
            tracks: 0,
            laid: None,
            frame: BoxLayout::ZERO,
            since: f64::NAN,
            posed: false,
            playing: false,
            exiting: false,
            ends: f64::NEG_INFINITY,
            starts: f64::NEG_INFINITY,
            cue: None,
        });
        self.live.len() - 1
    }

    fn drop_live(&mut self, i: usize) {
        let l = self.live.swap_remove(i);
        self.store.remove_node(node(l.id));
        self.slot[l.id as usize] = NONE;
        if let Some(moved) = self.live.get(i) {
            self.slot[moved.id as usize] = i as u32;
        }
    }

    /// Runs `id` from `from` to `to` on `spec`: each channel that differs
    /// (or is already moving) animates if `spec` transitions its group,
    /// else jumps. `replay` restarts its keyframes.
    fn go(&mut self, id: Id, spec: &Spec, from: &Skin, to: &Skin, replay: bool) {
        let n = node(id);
        let mut at = self.find(id);
        for ch in 0..CHANNELS {
            let bit = 1u32 << ch;
            let tracked = at.is_some_and(|i| self.live[i].tracks & bit != 0);
            let (a, b) = (read(ch, from), read(ch, to));
            if !tracked && a == b {
                continue;
            }
            match spec.anim_of(GROUPS[ch].0) {
                Some(anim) => {
                    if !tracked {
                        self.store.set(n, PROPS[ch], a);
                    }
                    self.store.animate(n, PROPS[ch], b, anim);
                    let i = *at.get_or_insert_with(|| self.ensure(id, to));
                    self.live[i].tracks |= bit;
                }
                None if tracked => {
                    self.store.remove(n, PROPS[ch]);
                    self.live[at.unwrap()].tracks &= !bit;
                }
                None => {}
            }
        }
        let keeps = spec.animate.is_some() || spec.props & LAYOUT != 0;
        if keeps && at.is_none() {
            at = Some(self.ensure(id, to));
        }
        let Some(i) = at else { return };
        let l = &mut self.live[i];
        if spec.props & LAYOUT == 0 && l.tracks & FRAME_BIT != 0 {
            self.store.remove(n, prop::FRAME);
            l.tracks &= !FRAME_BIT;
        }
        if spec.animate.is_none() {
            l.since = f64::NAN;
            l.posed = false;
        } else if replay {
            l.since = f64::INFINITY;
        }
        l.spec = *spec;
        l.base = *to;
        self.present(i);
    }

    /// Writes live `i`'s shown skin: its base, its tracks' values, then its
    /// keyframes' pose. True while its keyframes play.
    fn present(&mut self, i: usize) -> bool {
        let now = self.store.now();
        let l = &mut self.live[i];
        let mut s = l.base;
        let n = node(l.id);
        let mut bits = l.tracks & !FRAME_BIT;
        while bits != 0 {
            let ch = bits.trailing_zeros() as usize;
            bits &= bits - 1;
            if let Some(v) = self.store.get(n, PROPS[ch]) {
                write(ch, &mut s, &v);
            }
        }
        let mut playing = false;
        l.posed = false;
        if let Some(lp) = l.spec.animate.filter(|_| !l.since.is_nan()) {
            let size = l.laid.map_or([0.0; 2], |b| [b.w, b.h]);
            let (run, posed) = pose(&lp, now - l.since, &mut s, size, self.reduced);
            playing = run;
            l.posed = posed;
        }
        if let Some((lp, since)) = l.cue {
            let size = l.laid.map_or([0.0; 2], |b| [b.w, b.h]);
            let (run, posed) = pose(&lp, (now - since).max(0.0), &mut s, size, self.reduced);
            if run {
                playing = true;
                l.posed |= posed;
            } else {
                l.cue = None;
            }
        }
        l.shown = s;
        l.playing = playing;
        playing
    }

    /// Plays `lp` once on `id`, resting on `base`, over what its record
    /// plays and through its restyles: an attention cue.
    pub fn cue(&mut self, id: Id, lp: Loop, base: &Skin) {
        let i = self.ensure(id, base);
        self.live[i].cue = Some((lp, f64::INFINITY));
        self.present(i);
    }

    /// `id`, which transitions its layout, was put on screen where the
    /// element it replaces was last drawn (`from`, its parent's space):
    /// it slides from there to its own box on its next layout.
    pub fn glide(&mut self, id: Id, from: BoxLayout) {
        if let Some(i) = self
            .find(id)
            .filter(|&i| self.live[i].spec.props & LAYOUT != 0)
        {
            self.live[i].laid = Some(from);
        }
    }
}
