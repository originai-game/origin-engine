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
use crate::record::{Record, Rgba, SHADOWS, Shadow, Skin};

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

/// A built-in keyframe animation (`animate-*`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keyframes {
    /// Tailwind's: one turn clockwise.
    Spin,
    /// Tailwind's: scales to 2× and fades out.
    Ping,
    /// Tailwind's: fades to half opacity and back.
    Pulse,
    /// Tailwind's: jumps up a quarter of its height and lands.
    Bounce,
    /// Ours: in from opacity 0 at 0.9× on a spring.
    Pop,
    /// Ours: in from opacity 0 at 1.4×.
    Stamp,
    /// Ours: from 1.15× back to its size.
    Beat,
    /// Ours: settles from 1.4×, then rises 48 px and fades out, and stays
    /// out (a floating number).
    Float,
    /// Ours: sweeps across from -100% to 420% of its width.
    Shimmer,
}

/// `animate-*` and how it is timed (the class resolver's presets, or
/// `animation-*`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Loop {
    pub keyframes: Keyframes,
    /// Seconds per iteration.
    pub duration: f32,
    /// Seconds before it starts; it holds its first pose meanwhile.
    pub delay: f32,
    /// Cubic Bézier over each keyframe segment; `Bounce` keeps its own.
    pub ease: [f32; 4],
    /// `None` repeats forever.
    pub iterations: Option<u32>,
}

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
        }
    }
}

impl Spec {
    /// The animation a change runs on, or none when it jumps.
    fn anim(&self) -> Option<Anim> {
        let curve = match self.ease {
            Ease::Spring {
                stiffness,
                damping,
                mass,
            } => Curve::Spring(Spring::from_stiffness_damping(stiffness, damping, mass)),
            Ease::Bezier([a, b, c, d]) if self.duration > 0.0 => Curve::Tween {
                duration: self.duration,
                easing: Easing::cubic(a, b, c, d),
            },
            Ease::Bezier(_) => return None,
        };
        Some(Anim::new(curve).delay(self.delay.max(0.0)))
    }
}

/// Eight lanes: every channel's track has the same shape.
#[derive(Clone, Copy, Debug, PartialEq)]
struct L([f32; 8]);

impl Animatable for L {
    const LANES: usize = 8;
    fn write_lanes(&self, out: &mut [f32]) {
        out[..8].copy_from_slice(&self.0);
    }
    fn read_lanes(l: &[f32]) -> Self {
        let mut v = [0.0; 8];
        v.copy_from_slice(&l[..8]);
        L(v)
    }
}

/// A skin's animated values, one track each.
const CHANNELS: usize = 13;
const OPACITY_CH: usize = 0;
const RADIUS_CH: usize = 5;
const TRANSFORM_CH: usize = 6;
/// The first box shadow's channel; the other three follow.
const SHADOW_CH: usize = 7;
const TEXT_SHADOW_CH: usize = SHADOW_CH + SHADOWS;
const BLUR_CH: usize = TEXT_SHADOW_CH + 1;
const _: () = assert!(BLUR_CH + 1 == CHANNELS);

/// Each channel's transition group and settle class.
const GROUPS: [(u8, PropClass); CHANNELS] = [
    (OPACITY, PropClass::Opacity),
    (COLORS, PropClass::Color), // fill
    (COLORS, PropClass::Color), // ink
    (COLORS, PropClass::Color), // border colour
    (COLORS, PropClass::Color), // outline colour
    (SHAPE, PropClass::Shape),
    (TRANSFORM, PropClass::Scale),
    (SHADOW, PropClass::Color),
    (SHADOW, PropClass::Color),
    (SHADOW, PropClass::Color),
    (SHADOW, PropClass::Color),
    (SHADOW, PropClass::Color), // text shadow
    (SHADOW, PropClass::Blur),  // backdrop blur
];

const PROPS: [Prop<L>; CHANNELS] = {
    let mut p = [Prop::custom(64, PropClass::Other); CHANNELS];
    let mut k = 0;
    while k < CHANNELS {
        p[k] = Prop::custom(64 + k as u16, GROUPS[k].1);
        k += 1;
    }
    p
};

/// Premultiplied OKLab lanes of a linear colour.
fn lab(c: Rgba) -> [f32; 4] {
    let [l, a, b] = linear_to_oklab(c[0], c[1], c[2]);
    let al = c[3];
    [l * al, a * al, b * al, al]
}

fn unlab(v: &[f32]) -> Rgba {
    let al = v[3].clamp(0.0, 1.0);
    if al <= 1e-6 {
        return [0.0; 4];
    }
    let [r, g, b] = oklab_to_linear([v[0] / al, v[1] / al, v[2] / al]);
    [r.max(0.0), g.max(0.0), b.max(0.0), al]
}

fn shadow_lanes(s: &Shadow) -> [f32; 8] {
    let c = lab(s.color);
    [
        s.offset[0],
        s.offset[1],
        s.blur,
        s.spread,
        c[0],
        c[1],
        c[2],
        c[3],
    ]
}

fn put_shadow(s: &mut Shadow, v: &[f32; 8]) {
    s.offset = [v[0], v[1]];
    s.blur = v[2].max(0.0);
    s.spread = v[3];
    s.color = unlab(&v[4..]);
}

fn read(ch: usize, s: &Skin) -> L {
    let mut v = [0.0; 8];
    match ch {
        OPACITY_CH => v[0] = s.opacity,
        1..=4 => {
            let c = [s.fill, s.ink, s.border_color, s.outline.color][ch - 1];
            v[..4].copy_from_slice(&lab(c));
        }
        RADIUS_CH => v[..4].copy_from_slice(&s.radius),
        TRANSFORM_CH => {
            let t = &s.transform;
            v[..7].copy_from_slice(&[
                t.translate[0],
                t.translate[1],
                t.scale[0],
                t.scale[1],
                t.rotate,
                t.percent[0],
                t.percent[1],
            ]);
        }
        TEXT_SHADOW_CH => v = shadow_lanes(&s.text_shadow),
        BLUR_CH => v[0] = s.backdrop_blur,
        _ => v = shadow_lanes(&s.shadows[ch - SHADOW_CH]),
    }
    L(v)
}

fn write(ch: usize, s: &mut Skin, L(v): &L) {
    match ch {
        OPACITY_CH => s.opacity = v[0].clamp(0.0, 1.0),
        1 => s.fill = unlab(v),
        2 => s.ink = unlab(v),
        3 => s.border_color = unlab(v),
        4 => s.outline.color = unlab(v),
        RADIUS_CH => s.radius = [v[0], v[1], v[2], v[3]].map(|r| r.max(0.0)),
        TRANSFORM_CH => {
            let t = &mut s.transform;
            (t.translate, t.scale, t.rotate) = ([v[0], v[1]], [v[2], v[3]], v[4]);
            t.percent = [v[5], v[6]];
        }
        TEXT_SHADOW_CH => put_shadow(&mut s.text_shadow, v),
        BLUR_CH => s.backdrop_blur = v[0].max(0.0),
        _ => {
            let k = ch - SHADOW_CH;
            put_shadow(&mut s.shadows[k], v);
            // A layer only one side has draws while it moves (from or to
            // transparent, as CSS pads the shorter list).
            s.shadow_count = s.shadow_count.max(k as u8 + 1);
        }
    }
}

fn rect(b: &BoxLayout) -> Rect {
    Rect::from_edges(b.x, b.y, b.x + b.w, b.y + b.h)
}

fn node(id: Id) -> NodeId {
    NodeId(id as u64)
}

const NONE: u32 = u32::MAX;
const FRAME_BIT: u16 = 1 << CHANNELS;

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
    tracks: u16,
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
        if let Some(i) = self.find(id).filter(|&i| self.live[i].exiting) {
            self.live[i].exiting = false;
            let base = self.live[i].base;
            self.go(id, &to.motion, &base, &to.skin, false);
            return;
        }
        let from = from.map_or(&to.skin, |f| &f.skin);
        self.go(id, &to.motion, from, &to.skin, true);
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
                self.live[i].exiting = true;
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
        let anim = spec.anim();
        let n = node(id);
        let mut at = self.find(id);
        for ch in 0..CHANNELS {
            let bit = 1u16 << ch;
            let tracked = at.is_some_and(|i| self.live[i].tracks & bit != 0);
            let (a, b) = (read(ch, from), read(ch, to));
            if !tracked && a == b {
                continue;
            }
            match anim.filter(|_| spec.props & GROUPS[ch].0 != 0) {
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
        l.shown = s;
        l.playing = playing;
        playing
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn bezier(e: [f32; 4], x: f32) -> f32 {
    Easing::cubic(e[0], e[1], e[2], e[3]).eval(x.clamp(0.0, 1.0))
}

/// The keyframes' pose `elapsed` seconds after they started, over `s`:
/// whether they still play, and whether they hold a pose at all. Reduced
/// motion keeps the opacity and drops the movement.
fn pose(lp: &Loop, elapsed: f64, s: &mut Skin, size: [f32; 2], reduced: bool) -> (bool, bool) {
    use Keyframes as K;
    let e = (elapsed - lp.delay as f64).max(0.0);
    let n = e / lp.duration.max(1e-3) as f64;
    let (p, playing) = match lp.iterations {
        Some(k) if n >= k as f64 => (1.0, false),
        _ => (n.fract() as f32, true),
    };
    // Pops, stamps and beats end where they rest; only a float stays away.
    if !playing && lp.keyframes != K::Float {
        return (false, false);
    }
    let ez = |x: f32| bezier(lp.ease, x);
    let mut scale = 1.0;
    let mut shift = [0.0f32; 2];
    let mut turn = 0.0;
    let o = &mut s.opacity;
    match lp.keyframes {
        K::Spin => turn = std::f32::consts::TAU * ez(p),
        K::Ping => {
            let q = ez((p / 0.75).min(1.0));
            scale = 1.0 + q;
            *o *= 1.0 - q;
        }
        K::Pulse => {
            let q = ez(if p < 0.5 { p * 2.0 } else { 2.0 - p * 2.0 });
            *o = lerp(*o, 0.5, q);
        }
        K::Bounce => {
            let q = if p < 0.5 {
                1.0 - bezier([0.8, 0.0, 1.0, 1.0], p * 2.0)
            } else {
                bezier([0.0, 0.0, 0.2, 1.0], p * 2.0 - 1.0)
            };
            shift[1] = -0.25 * size[1] * q;
        }
        K::Pop => {
            let q = ez(p);
            *o *= q.clamp(0.0, 1.0);
            scale = lerp(0.9, 1.0, q);
        }
        K::Stamp => {
            let q = ez(p);
            *o *= q;
            scale = lerp(1.4, 1.0, q);
        }
        K::Beat => scale = lerp(1.15, 1.0, ez(p)),
        K::Float => {
            if p < 0.2 {
                scale = lerp(1.4, 1.0, ez(p / 0.2));
            } else {
                let q = ez((p - 0.2) / 0.8);
                shift[1] = -48.0 * q;
                *o *= 1.0 - q;
            }
        }
        K::Shimmer => shift[0] = size[0] * lerp(-1.0, 4.2, ez(p)),
    }
    if !reduced {
        let t = &mut s.transform;
        t.scale = [t.scale[0] * scale, t.scale[1] * scale];
        t.translate = [t.translate[0] + shift[0], t.translate[1] + shift[1]];
        t.rotate += turn;
    }
    (playing, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    type R = Record;

    fn timed(props: u8, duration: f32, ease: [f32; 4]) -> Spec {
        Spec {
            props,
            duration,
            ease: Ease::Bezier(ease),
            ..Spec::default()
        }
    }

    fn with(spec: Spec, f: impl FnOnce(&mut Skin)) -> R {
        let mut r = R {
            motion: spec,
            ..R::default()
        };
        f(&mut r.skin);
        r
    }

    const LINEAR: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

    fn looping(keyframes: Keyframes, duration: f32, iterations: Option<u32>) -> Spec {
        Spec {
            animate: Some(Loop {
                keyframes,
                duration,
                delay: 0.0,
                ease: [0.23, 1.0, 0.32, 1.0],
                iterations,
            }),
            ..Spec::default()
        }
    }

    #[test]
    fn a_transition_runs_on_its_duration_and_ease_and_retargets_from_where_it_is() {
        let mut m = Motion::new();
        let spec = timed(OPACITY, 0.2, LINEAR);
        let a = with(spec, |s| s.opacity = 0.0);
        let b = with(spec, |s| s.opacity = 1.0);
        m.tick_at(1.0);
        m.restyle(7, &a, &b);
        assert_eq!(
            m.skin(7, &b.skin).opacity,
            0.0,
            "it starts where it was, this frame"
        );
        assert!(m.tick_at(1.05));
        let o = m.skin(7, &b.skin).opacity;
        assert!(
            (o - 0.25).abs() < 1e-3,
            "a quarter of the way, linearly: {o}"
        );
        // Turned back halfway: it leaves from 0.25, not from 1.
        assert!(m.tick_at(1.1));
        let half = m.skin(7, &b.skin).opacity;
        m.restyle(7, &b, &a);
        assert!((m.skin(7, &a.skin).opacity - half).abs() < 1e-3);
        // Linear tweens add: the rise and the fall cancel until the rise
        // would have ended, then it falls.
        m.tick_at(1.25);
        assert!(m.skin(7, &a.skin).opacity < half, "now on its way down");
        // Long after, it rests on the record and motion lets it go.
        m.tick_at(2.0);
        assert!(!m.tick_at(2.1));
        assert!(
            std::ptr::eq(m.skin(7, &a.skin), &a.skin),
            "settled: paint reads the record"
        );
    }

    #[test]
    fn only_the_named_groups_move_and_the_rest_jumps() {
        let mut m = Motion::new();
        let spec = timed(COLORS, 0.3, LINEAR);
        let red = with(spec, |s| {
            s.fill = [1.0, 0.0, 0.0, 1.0];
            s.opacity = 1.0;
        });
        let blue = with(spec, |s| {
            s.fill = [0.0, 0.0, 1.0, 1.0];
            s.opacity = 0.5;
        });
        m.tick_at(0.0);
        m.restyle(1, &red, &blue);
        m.tick_at(0.15);
        let s = m.skin(1, &blue.skin);
        assert_eq!(s.opacity, 0.5, "opacity is not in transition-colors");
        assert!(
            s.fill[0] > 0.05 && s.fill[2] > 0.05,
            "between red and blue: {:?}",
            s.fill
        );
        // OKLab, not sRGB: the midpoint keeps its lightness instead of
        // passing through a dark purple.
        let grey = (s.fill[0] + s.fill[1] + s.fill[2]) / 3.0;
        assert!(grey > 0.1, "{:?}", s.fill);
        // No duration: nothing moves at all.
        let still = timed(ALL, 0.0, LINEAR);
        m.restyle(
            2,
            &with(still, |s| s.opacity = 0.0),
            &with(still, |s| s.opacity = 1.0),
        );
        assert!(m.find(2).is_none(), "an untimed change holds no state");
    }

    #[test]
    fn a_delay_holds_the_old_value_and_a_spring_times_itself() {
        let mut m = Motion::new();
        let mut spec = timed(TRANSFORM, 0.1, LINEAR);
        spec.delay = 0.2;
        let a = with(spec, |_| {});
        let b = with(spec, |s| s.transform.translate = [100.0, 0.0]);
        m.tick_at(0.0);
        m.restyle(3, &a, &b);
        assert!(m.tick_at(0.15), "waiting counts as moving");
        assert_eq!(
            m.skin(3, &b.skin).transform.translate[0],
            0.0,
            "still in its delay"
        );
        m.tick_at(0.25);
        let x = m.skin(3, &b.skin).transform.translate[0];
        assert!(
            (x - 50.0).abs() < 1.0,
            "half of the 0.1 s after the delay: {x}"
        );
        spec = Spec {
            props: TRANSFORM,
            ease: Ease::Spring {
                stiffness: 170.0,
                damping: 26.0,
                mass: 1.0,
            },
            ..Spec::default()
        };
        let (a, b) = (
            with(spec, |_| {}),
            with(spec, |s| s.transform.scale = [2.0, 2.0]),
        );
        m.restyle(4, &a, &b);
        m.tick_at(0.35);
        let sc = m.skin(4, &b.skin).transform.scale[0];
        assert!(sc > 1.2 && sc < 2.0, "{sc}");
    }

    #[test]
    fn an_exit_keeps_the_element_until_it_settles_and_an_enter_starts_from_starting() {
        let mut m = Motion::new();
        let spec = timed(OPACITY | TRANSFORM, 0.2, LINEAR);
        let shown = with(spec, |_| {});
        let gone = with(spec, |s| {
            s.opacity = 0.0;
            s.transform.translate = [0.0, 8.0];
        });
        m.tick_at(0.0);
        m.enter(5, &shown, Some(&gone));
        assert_eq!(
            m.skin(5, &shown.skin).opacity,
            0.0,
            "enters from its starting: record"
        );
        m.tick_at(0.3);
        assert!(!m.tick_at(0.31));
        assert_eq!(m.skin(5, &shown.skin).opacity, 1.0);
        // The first build appears as it is.
        m.enter(6, &shown, None::<&R>);
        assert!(m.find(6).is_none());
        assert!(m.exit(5, &shown, Some(&gone)));
        assert!(m.exiting(5));
        m.tick_at(0.4);
        assert_eq!(m.take_exited().count(), 0, "still on its way out");
        assert!(m.skin(5, &shown.skin).opacity < 1.0);
        m.tick_at(0.6);
        assert_eq!(
            m.take_exited().collect::<Vec<_>>(),
            [5],
            "out once it settles"
        );
        assert!(!m.exiting(5));
        // No exit: record, or nothing that moves: freed at once.
        assert!(!m.exit(6, &shown, None::<&R>));
        assert!(!m.exit(6, &shown, Some(&shown)));
    }

    #[test]
    fn an_element_inserted_again_while_it_exits_turns_back() {
        let mut m = Motion::new();
        let spec = timed(OPACITY, 0.2, LINEAR);
        let shown = with(spec, |_| {});
        let gone = with(spec, |s| s.opacity = 0.0);
        m.tick_at(0.0);
        assert!(m.exit(9, &shown, Some(&gone)));
        m.tick_at(0.1);
        let half = m.skin(9, &shown.skin).opacity;
        m.enter(9, &shown, Some(&gone));
        assert!(!m.exiting(9));
        assert!(
            (m.skin(9, &shown.skin).opacity - half).abs() < 1e-3,
            "from where it is"
        );
        m.tick_at(0.5);
        assert_eq!(m.take_exited().count(), 0, "it never left");
        assert_eq!(m.skin(9, &shown.skin).opacity, 1.0);
    }

    fn boxed(x: f32, y: f32, w: f32) -> BoxLayout {
        BoxLayout {
            x,
            y,
            w,
            h: 20.0,
            inset: [0.0; 4],
        }
    }

    #[test]
    fn a_layout_transition_slides_a_moved_box_and_others_jump() {
        let mut m = Motion::new();
        let slide = with(timed(LAYOUT, 0.2, LINEAR), |_| {});
        let plain = with(timed(ALL, 0.2, LINEAR), |_| {});
        m.tick_at(0.0);
        m.enter(1, &slide, None::<&R>);
        m.enter(2, &plain, None::<&R>);
        let mut at = [boxed(0.0, 0.0, 50.0); 3];
        m.relayout(|id| at[id as usize]);
        assert_eq!(m.box_of(1), None, "its first box it takes as it is");
        at[1] = boxed(100.0, 0.0, 70.0);
        at[2] = boxed(100.0, 0.0, 70.0);
        m.relayout(|id| at[id as usize]);
        assert_eq!(m.box_of(1).unwrap().x, 0.0, "it leaves from where it was");
        assert_eq!(m.box_of(2), None, "transition-all does not move layout");
        m.tick_at(0.1);
        let b = m.box_of(1).unwrap();
        assert!(
            (b.x - 50.0).abs() < 0.5 && (b.w - 60.0).abs() < 0.5,
            "{b:?}"
        );
        // Moved again mid-flight: from where it is drawn.
        at[1] = boxed(0.0, 0.0, 50.0);
        m.relayout(|id| at[id as usize]);
        assert!((m.box_of(1).unwrap().x - b.x).abs() < 0.5);
        m.tick_at(0.5);
        assert!(!m.tick_at(0.6));
        assert_eq!(m.box_of(1), None, "settled: paint reads the arena");
    }

    #[test]
    fn keyframes_pose_over_the_skin_and_a_one_shot_replays_when_it_newly_applies() {
        let mut m = Motion::new();
        let mut spin = looping(Keyframes::Spin, 1.0, None);
        spin.animate.as_mut().unwrap().ease = LINEAR;
        let r = with(spin, |_| {});
        // A loop starts at the sample that first shows it.
        m.enter(1, &r, None::<&R>);
        m.tick_at(0.0);
        assert!(m.tick_at(0.25));
        let turn = m.skin(1, &r.skin).transform.rotate;
        assert!((turn - std::f32::consts::FRAC_PI_2).abs() < 1e-3, "{turn}");
        assert!(m.tick_at(10.0), "forever");

        let mut m = Motion::new();
        let beat = looping(Keyframes::Beat, 0.4, Some(1));
        let (calm, hit) = (with(Spec::default(), |_| {}), with(beat, |_| {}));
        m.restyle(2, &calm, &hit);
        assert!(
            m.skin(2, &hit.skin).transform.scale[0] > 1.1,
            "it starts swollen"
        );
        m.tick_at(10.0);
        m.tick_at(10.5);
        assert!(!m.tick_at(10.6), "once");
        assert!(std::ptr::eq(m.skin(2, &hit.skin), &hit.skin));
        // Another change that keeps the beat does not replay it …
        let mut hit2 = hit.clone();
        hit2.skin.fill = [1.0; 4];
        m.restyle(2, &hit, &hit2);
        assert!(!m.tick_at(10.7));
        // … one that applies it again does.
        m.restyle(2, &calm, &hit);
        assert!(m.tick_at(10.75));

        // A float rises, fades and stays out.
        let mut m = Motion::new();
        let float = looping(Keyframes::Float, 0.8, Some(1));
        let f = with(float, |_| {});
        m.enter(3, &f, None::<&R>);
        m.tick_at(10.0);
        m.tick_at(12.0);
        let s = m.skin(3, &f.skin);
        assert_eq!((s.opacity, s.transform.translate[1]), (0.0, -48.0));
        assert!(!m.tick_at(12.1));
    }

    #[test]
    fn reduced_motion_keeps_fades_and_drops_movement() {
        let mut m = Motion::new();
        m.set_reduced_motion(true);
        let spec = timed(OPACITY | TRANSFORM, 0.2, LINEAR);
        let a = with(spec, |s| s.opacity = 0.0);
        let b = with(spec, |s| {
            s.transform.translate = [40.0, 0.0];
            s.opacity = 1.0;
        });
        m.tick_at(0.0);
        m.restyle(1, &a, &b);
        m.tick_at(0.02);
        let s = m.skin(1, &b.skin);
        assert_eq!(s.transform.translate[0], 40.0, "movement jumps");
        assert!(
            s.opacity > 0.0 && s.opacity < 1.0,
            "the fade stays: {}",
            s.opacity
        );
        let ping = with(looping(Keyframes::Ping, 1.0, None), |_| {});
        m.enter(2, &ping, None::<&R>);
        m.tick_at(0.02);
        m.tick_at(0.4);
        let s = m.skin(2, &ping.skin);
        assert_eq!(s.transform.scale, [1.0; 2]);
        assert!(s.opacity < 1.0);
    }
}
