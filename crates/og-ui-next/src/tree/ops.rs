//! The TS renderer's ops applied to the arena (CONVENTIONS.md §Ops): a
//! binary batch read in place, each op patching the element it names. An
//! op node is an element of the tree like a document's, so layout, text,
//! icons and paint serve it unchanged; its record comes from its class and
//! style through the resolver, once per distinct pair.

use super::*;
use crate::a11y;
use rustc_hash::FxHashMap;

/// The first bytes of an op batch, which no store command batch starts
/// with (the wire codec's first byte is a value tag, 0 to 6).
pub const MAGIC: &[u8; 4] = b"OGOP";
/// The first bytes of the reply: the events since the last batch.
pub const EVENTS: &[u8; 4] = b"OGEV";
/// The first bytes of the reply to a batch an op refused, then the reason.
pub const REFUSED: &[u8; 4] = b"OGER";

/// What an op node may listen to, by its code in `listen`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Press = 0,
    Tap,
    PointerDown,
    PointerMove,
    PointerUp,
    Wheel,
    Focus,
    Blur,
    Key,
    Change,
    /// An input action its `actions` prop lists (`onAction`), by name in
    /// the event's `key`.
    Action,
    /// A finger held still on it for 0.5 s (`onLongPress`).
    LongPress,
    /// A held press on a node with a `hold` duration: its progress, 0 to
    /// 1, in `value` each frame; "0" when it lets go early (`onHold`).
    Hold,
    /// A `draggable` node was picked up: its data in `key`, its box in
    /// `value` (`onDragStart`).
    DragStart,
    /// A carried node moved: its data in `key`, the `dropTarget` under it
    /// (or focused) in `value`, "" over none (`onDragMove`).
    DragMove,
    /// A carried node was put down: its data in `key`, the `dropTarget` it
    /// went to in `value`, "" when cancelled (`onDragEnd`).
    DragEnd,
    /// A carried node was dropped on this `dropTarget`: the node's data in
    /// `key`, this target's box in `value` (`onDrop`).
    Drop,
    /// Two fingers that went down on it pinch: `key` is `start`, `move`
    /// or `end`, `value` the scale from where they began, x and y their
    /// midpoint (`onPinch`).
    Pinch,
    /// Two fingers that went down on it turn: as `Pinch`, `value` the
    /// turn in radians, clockwise on screen (`onRotate`).
    Rotate,
    /// A pointer that went down on it was let go moving fast: `key` its
    /// direction (`left`, `right`, `up`, `down`), `value` its velocity
    /// `"vx vy"` in px/s (`onSwipe`).
    Swipe,
}

impl Event {
    const ALL: [Event; 20] = [
        Event::Press,
        Event::Tap,
        Event::PointerDown,
        Event::PointerMove,
        Event::PointerUp,
        Event::Wheel,
        Event::Focus,
        Event::Blur,
        Event::Key,
        Event::Change,
        Event::Action,
        Event::LongPress,
        Event::Hold,
        Event::DragStart,
        Event::DragMove,
        Event::DragEnd,
        Event::Drop,
        Event::Pinch,
        Event::Rotate,
        Event::Swipe,
    ];

    pub fn bit(self) -> u32 {
        1 << self as u32
    }
}

/// An event an op node fired, for the renderer to dispatch by id.
#[derive(Clone, Debug, PartialEq)]
pub struct Fired {
    pub id: u32,
    pub event: Event,
    /// The pointer on the UI's surface.
    pub x: f32,
    pub y: f32,
    /// A key's name (`key`) and a control's new value (`change`).
    pub key: Box<str>,
    pub value: Box<str>,
}

impl Fired {
    pub(super) fn at(id: u32, event: Event, x: f32, y: f32) -> Fired {
        Fired {
            id,
            event,
            x,
            y,
            key: Box::default(),
            value: Box::default(),
        }
    }
}

/// A class string and inline style to their record under the given
/// [`crate::classes::state`] bits: 0, or `STARTING` and `EXIT` for the
/// records motion enters from and leaves to.
/// `data` is the node's data context ([`Classes::data_id`]): its own
/// `data-*` names and its nearest groups' as `group/name`, by spaces.
pub type Resolve = Box<dyn Fn(&str, &str, u16, &str) -> Record>;

use crate::classes::state::{
    DISABLED, EXIT, FOCUS_VISIBLE, FOCUS_WITHIN, FOCUSED, HOVERED, PRESSED, STARTING,
};

/// The states input sets on op nodes, which their classes may read.
const INTERACTION: [u16; 6] = [
    PRESSED,
    HOVERED,
    FOCUSED,
    DISABLED,
    FOCUS_WITHIN,
    FOCUS_VISIBLE,
];

/// The records of the class and style pairs the op nodes use, each
/// resolved once; an entry's key is `class\0style`. `presence` holds an
/// entry's `starting:` and `exit:` records where they differ from its own.
#[derive(Default)]
pub struct Classes {
    index: FxHashMap<Box<str>, u32>,
    keys: Vec<Box<str>>,
    records: Vec<Record>,
    presence: Vec<[Option<Box<Record>>; 2]>,
    /// Each entry's interaction states its record reads, and its records
    /// under the ones it reads, made the first time they are met.
    tracked: Vec<u16>,
    stated: FxHashMap<(u32, u16, u32), Record>,
    /// The data contexts met, by id less one (0: none).
    datas: Vec<Box<str>>,
    data_index: FxHashMap<Box<str>, u32>,
    buf: String,
    resolve: Option<Resolve>,
}

/// The interaction states an entry's record reads: each one that changes
/// it alone, or all of them when only a combination (`hover:focus:`)
/// does.
fn tracks(resolve: Option<&Resolve>, c: &str, s: &str, base: &Record) -> u16 {
    let Some(f) = resolve else {
        return 0;
    };
    let mut t = 0;
    for bit in INTERACTION {
        if f(c, s, bit, "") != *base {
            t |= bit;
        }
    }
    let all = INTERACTION.iter().fold(0, |a, b| a | b);
    if t != all && f(c, s, all, "") != f(c, s, t, "") {
        t = all;
    }
    t
}

/// An entry's record, and its `starting:` and `exit:` ones where they
/// differ from it.
fn resolved(resolve: Option<&Resolve>, c: &str, s: &str) -> (Record, [Option<Box<Record>>; 2]) {
    let Some(f) = resolve else {
        return (Record::default(), [None, None]);
    };
    let base = f(c, s, 0, "");
    let other = |state| Some(Box::new(f(c, s, state, ""))).filter(|r| **r != base);
    let presence = [other(STARTING), other(EXIT)];
    (base, presence)
}

impl Classes {
    /// The interaction states entry `i` reads.
    pub(super) fn tracked(&self, i: u32) -> u16 {
        self.tracked.get(i as usize).copied().unwrap_or(0)
    }

    /// Entry `i`'s record under interaction states `states` and data
    /// context `data`.
    pub(super) fn record(&mut self, i: u32, states: u16, data: u32) -> &Record {
        let st = states & (self.tracked(i) | crate::classes::OTHER_SCHEME);
        if st == 0 && data == 0 {
            return &self.records[i as usize];
        }
        let resolve = self.resolve.as_ref();
        let key = &self.keys[i as usize];
        let d = data.checked_sub(1).map_or("", |d| &*self.datas[d as usize]);
        self.stated.entry((i, st, data)).or_insert_with(|| {
            let (c, s) = Self::split(key);
            resolve.map_or_else(Record::default, |f| f(c, s, st, d))
        })
    }

    /// The id of data context `data` (0 for none), interned.
    pub(super) fn data_id(&mut self, data: &str) -> u32 {
        if data.is_empty() {
            return 0;
        }
        if let Some(&i) = self.data_index.get(data) {
            return i;
        }
        self.datas.push(data.into());
        let i = self.datas.len() as u32;
        self.data_index.insert(data.into(), i);
        i
    }

    /// The groups entry `i`'s classes make it (`group`: `""`,
    /// `group/name`: `name`).
    fn groups(&self, i: u32) -> impl Iterator<Item = &str> {
        let class = self.keys.get(i as usize).map_or("", |k| Self::split(k).0);
        class.split_ascii_whitespace().filter_map(|c| match c {
            "group" => Some(""),
            c => c.strip_prefix("group/"),
        })
    }

    /// Entry `i`'s `starting:` (0) or `exit:` (1) record.
    pub(super) fn presence(&self, i: u32, k: usize) -> Option<&Record> {
        self.presence[i as usize][k].as_deref()
    }

    fn split(key: &str) -> (&str, &str) {
        key.split_once('\0').unwrap_or((key, ""))
    }

    fn get(&mut self, class: &str, style: &str) -> u32 {
        self.buf.clear();
        self.buf.push_str(class);
        self.buf.push('\0');
        self.buf.push_str(style);
        self.lookup()
    }

    /// Entry `old` with its class (true) or its style replaced.
    fn with(&mut self, old: u32, class: bool, text: &str) -> u32 {
        let (c, s) = self
            .keys
            .get(old as usize)
            .map_or(("", ""), |k| Self::split(k));
        self.buf.clear();
        self.buf.push_str(if class { text } else { c });
        self.buf.push('\0');
        self.buf.push_str(if class { s } else { text });
        self.lookup()
    }

    /// The entry of the key in `buf`, resolved the first time it is seen.
    fn lookup(&mut self) -> u32 {
        if let Some(&i) = self.index.get(self.buf.as_str()) {
            return i;
        }
        let (c, s) = Self::split(&self.buf);
        let (record, presence) = resolved(self.resolve.as_ref(), c, s);
        self.tracked
            .push(tracks(self.resolve.as_ref(), c, s, &record));
        let i = self.keys.len() as u32;
        self.records.push(record);
        self.presence.push(presence);
        self.keys.push(self.buf.as_str().into());
        self.index.insert(self.buf.as_str().into(), i);
        i
    }

    fn reresolve(&mut self) {
        self.stated.clear();
        for (i, k) in self.keys.iter().enumerate() {
            let (c, s) = Self::split(k);
            (self.records[i], self.presence[i]) = resolved(self.resolve.as_ref(), c, s);
            self.tracked[i] = tracks(self.resolve.as_ref(), c, s, &self.records[i]);
        }
    }
}

/// A batch read in place.
struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self.at.checked_add(n).filter(|&e| e <= self.b.len());
        let end = end.ok_or("og.ui ops: the batch ends inside an op")?;
        let s = &self.b[self.at..end];
        self.at = end;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("4 bytes"),
        ))
    }
    fn str(&mut self) -> Result<&'a str, String> {
        let n = self.u32()? as usize;
        std::str::from_utf8(self.take(n)?).map_err(|_| "og.ui ops: a string is not UTF-8".into())
    }
}

impl Tree {
    /// Gives op nodes' classes and styles their records (head 1's
    /// Tailwind), and resolves the ones in use again: a scheme or pack
    /// change calls it too.
    pub fn set_resolver(&mut self, resolve: Resolve) {
        let mut shadow = resolve("bg-bg", "", 0, "").skin.fill;
        shadow[3] *= 0.45;
        let fg = resolve("text-fg bg-focus", "", 0, "");
        self.cursor_colors = [shadow, fg.skin.ink, fg.skin.fill];
        self.page_dark = fg.dark;
        // The root passes down the theme's body text in its ink, as the
        // page's body does.
        let mut root = resolve("text-fg text-body font-body", "", 0, "");
        root.text.set = 0;
        self.root = Inherit {
            size: root.font_size,
            ink: root.skin.ink,
            text: root.text,
            opacity: 1.0,
            ..self.root
        };
        self.classes.resolve = Some(resolve);
        // Which nodes are in the other scheme is relative to the page's.
        for e in 0..self.els.len() as u32 {
            self.els[e as usize].other = self.other_scheme(e);
        }
        self.restyle_all();
        for e in 0..self.els.len() as u32 {
            let el = &self.els[e as usize];
            if !el.dead && el.parent == NONE {
                self.reapply(e);
            }
        }
        self.paint = true;
    }

    /// Whether it holds any op node.
    pub fn has_ops(&self) -> bool {
        self.by_js.iter().any(|&e| e != NONE)
    }

    /// Every op node takes its class entry's record again.
    pub fn restyle_all(&mut self) {
        self.classes.reresolve();
        for e in 0..self.els.len() as u32 {
            let el = &self.els[e as usize];
            if el.js != NONE && !el.dead {
                self.restyle(e);
            }
        }
    }

    /// Whether element `e` is drawn in the other scheme than the page's:
    /// the nearest `scheme-*` of it and its ancestors names that one.
    fn other_scheme(&self, e: u32) -> bool {
        let mut at = e;
        while at != NONE {
            let el = &self.els[at as usize];
            if let Some(dark) = el.record.scheme {
                return dark != self.page_dark;
            }
            at = el.parent;
        }
        false
    }

    pub(super) fn restyle(&mut self, e: u32) {
        let class = self.els[e as usize].class;
        let mut states = self.input.per(e).map_or(0, |p| p.states);
        if self.els[e as usize].other {
            states |= crate::classes::OTHER_SCHEME;
        }
        let data = self.els[e as usize].data;
        let mut record = self.classes.record(class, states, data).clone();
        // The focus ring every control shows while the keyboard or a pad
        // has it, as a browser's `:focus-visible { outline: auto }` does:
        // the pack's `Focus.ring`, unless the element's own classes style
        // its focus (they read `focus-visible:`) or draw an outline.
        // (An entry that reads a combination of states tracks them all, so
        // whether focus-visible changes its record is asked directly.)
        if states & FOCUS_VISIBLE != 0
            && record.skin.outline.width == 0.0
            && *self.classes.record(class, states & !FOCUS_VISIBLE, data) == record
        {
            let ring = self.classes.get("@Focus.ring", "");
            record.skin.outline = self.classes.record(ring, 0, 0).skin.outline;
        }
        // The press every control that answers one shows, as a game's
        // buttons squash under a finger: the pack's `Press.feedback`, its
        // pose under `active:` laid over the element's own transform and
        // its transform's curve kept up and down (in on its `active:`
        // curve, back on its spring), unless the element's own classes
        // move it under a press. A disabled control and a press catcher
        // (a screen's "tap to start") stay still.
        let mut squash = false;
        if self.presses(e) {
            let own_press = |c: &mut Classes, s: u16| c.record(class, s, data).skin.transform;
            let own = own_press(&mut self.classes, states | PRESSED)
                != own_press(&mut self.classes, states & !PRESSED);
            let press = self.classes.get("@Press.feedback", "");
            let p = self
                .classes
                .record(press, states | PRESSED, 0)
                .skin
                .transform;
            squash = own || !p.is_identity();
            if !own && states & DISABLED == 0 && !self.catcher(e) {
                let p = self.classes.record(press, states & PRESSED, 0).clone();
                if p.motion.props & crate::motion::TRANSFORM != 0 {
                    record.motion.transform = Some((p.motion.ease, p.motion.duration));
                }
                let t = &mut record.skin.transform;
                let s = p.skin.transform.scale;
                t.scale = [t.scale[0] * s[0], t.scale[1] * s[1]];
                t.translate = [
                    t.translate[0] + p.skin.transform.translate[0],
                    t.translate[1] + p.skin.transform.translate[1],
                ];
            }
        }
        self.els[e as usize].squash = squash;
        let record = &record;
        if self.els[e as usize].record != *record {
            // The new record's transition runs the change; a node not
            // drawn since it was made or inserted enters again with it.
            let old = std::mem::replace(&mut self.els[e as usize].record, record.clone());
            let el = &self.els[e as usize];
            match el.entering {
                Some(shown) => {
                    let from = self.classes.presence(el.class, 0).filter(|_| shown);
                    self.motion.enter(el.node, &el.record, from);
                }
                None => self.motion.restyle(el.node, &old, &el.record),
            }
            self.reapply(e);
            self.paint = true;
        }
    }

    /// Whether op element `e` is a control a finger presses: it answers a
    /// press, or it is an action's touch button (`touchButton`).
    pub(super) fn presses(&self, e: u32) -> bool {
        self.els[e as usize].clickable
            || self
                .input
                .per(e)
                .is_some_and(|p| matches!(p.zone, Some((og_input::ZoneKind::Button, _))))
    }

    /// `top` went on screen: the elements under it not drawn yet, built
    /// before it was put there, enter with it from their `starting:`
    /// records, as CSS's `@starting-style` applies to every element first
    /// drawn (a screen inside the `<Show>` branch or the mount that brings
    /// it).
    fn enter_subtree(&mut self, top: u32) {
        for i in 0..self.entering.len() {
            let d = self.entering[i];
            if d == top || self.els[d as usize].entering != Some(false) {
                continue;
            }
            if !self.is_under(d, top) {
                continue;
            }
            self.els[d as usize].entering = Some(true);
            let el = &self.els[d as usize];
            if let Some(from) = self.classes.presence(el.class, 0) {
                self.motion.enter(el.node, &el.record, Some(from));
            }
        }
    }

    /// Whether op element `e` covers over a third of the surface: a press
    /// catcher (a screen's "tap to start", a backdrop), not a control.
    pub(super) fn catcher(&self, e: u32) -> bool {
        let b = self.arena.box_of(self.els[e as usize].node);
        b.w * b.h * 3.0 > self.size.0 * self.size.1
    }

    /// The element op node `id` is.
    fn op(&self, id: u32) -> Result<u32, String> {
        match self.by_js.get(id as usize) {
            Some(&e) if e != NONE => Ok(e),
            _ => Err(format!("og.ui ops: no node {id}")),
        }
    }

    fn create(&mut self, id: u32, kind: Kind, text: bool) -> Result<u32, String> {
        if id == 0 || self.by_js.get(id as usize).is_some_and(|&e| e != NONE) {
            return Err(format!("og.ui ops: node {id} exists"));
        }
        let class = self.classes.get("", "");
        let record = self.classes.records[class as usize].clone();
        let node = self.arena.element(NONE, record.layout.clone());
        let leaf = text.then(|| self.arena.text(node, "", 16.0));
        let e = self.els.len() as u32;
        self.els.push(El {
            node,
            disabled: false,
            clickable: false,
            squash: false,
            js: id,
            listens: 0,
            class,
            arena_parent: NONE,
            parent: NONE,
            kind,
            leaf,
            text: String::new(),
            hidden: 0,
            dead: false,
            record,
            line: None,
            particles: None,
            anchor: None,
            attrs: Vec::new(),
            data: 0,
            other: false,
            image: None,
            entering: Some(false),
            eff: self.root,
        });
        if self.by_js.len() <= id as usize {
            self.by_js.resize(id as usize + 1, NONE);
        }
        self.by_js[id as usize] = e;
        self.input.per.resize(e as usize + 1, input::Per::default());
        self.input.per[e as usize].tab = input::NO_TAB;
        self.entering.push(e);
        self.inherit(e);
        Ok(e)
    }

    /// Frees element `e` and every element under it. Inside an element on
    /// its way out, it is freed when that exit settles; its id is free now.
    fn release(&mut self, e: u32) {
        if self.leaving_in(e) {
            self.unbind(e);
            match self.leaving.iter_mut().find(|l| l.0 == e) {
                Some(l) => l.1 = true,
                None => self.leaving.push((e, true)),
            }
            return;
        }
        self.free(e);
    }

    /// Whether `e` or an element it is in is on its way out: exiting, or
    /// removed and held while what is under it exits.
    fn leaving_in(&self, mut e: u32) -> bool {
        while e != NONE {
            if self.motion.exiting(self.els[e as usize].node)
                || self.leaving.iter().any(|l| l.0 == e)
            {
                return true;
            }
            e = self.els[e as usize].parent;
        }
        false
    }

    /// `top` was removed with no `exit:` record of its own: the elements
    /// under it with one leave to it (a screen inside the `<Show>` branch
    /// or the mount that goes), as a removed element with one does. True
    /// when any animates out: hold `top` drawn, not hit, until they settle.
    fn exit_under(&mut self, top: u32) -> bool {
        let mut any = false;
        for d in 0..self.els.len() as u32 {
            let el = &self.els[d as usize];
            if el.dead || el.js == NONE || !self.is_under(d, top) {
                continue;
            }
            let to = self.classes.presence(el.class, 1);
            any |= to.is_some() && self.motion.exit(el.node, &el.record, to);
        }
        any
    }

    /// Whether an element under `top` is on its way out.
    fn exiting_under(&self, top: u32) -> bool {
        (0..self.els.len() as u32).any(|d| {
            let el = &self.els[d as usize];
            !el.dead && self.motion.exiting(el.node) && self.is_under(d, top)
        })
    }

    /// Whether element `d` is below `top` (not `top` itself).
    fn is_under(&self, d: u32, top: u32) -> bool {
        let mut a = self.els[d as usize].parent;
        while a != NONE && a != top {
            a = self.els[a as usize].parent;
        }
        a == top && d != top
    }

    /// The ids of `e` and what is under it go back to the renderer.
    fn unbind(&mut self, e: u32) {
        let js = self.els[e as usize].js;
        // What is on its way out is no longer in the accessibility tree.
        self.sem.exiting(self.els[e as usize].node, true);
        if let Some(slot) = self.by_js.get_mut(js as usize).filter(|s| **s == e) {
            *slot = NONE;
        }
        for c in 0..self.els.len() as u32 {
            let el = &self.els[c as usize];
            if el.parent == e && !el.dead {
                self.unbind(c);
            }
        }
    }

    /// Frees `e` and what is under it now.
    fn free(&mut self, e: u32) {
        let el = &mut self.els[e as usize];
        el.dead = true;
        let (node, js) = (el.node, el.js);
        if let Some(leaf) = el.leaf {
            self.texts.release(leaf);
        }
        self.arena.detach(node);
        self.motion.forget(node);
        self.sem.release(node);
        self.input.modal_els.retain(|&m| m != e);
        self.input.zone_els.retain(|&z| z != e);
        self.forget_drag(e);
        self.anchored.retain(|&a| a != e);
        self.leaving.retain(|l| l.0 != e);
        if let Some(slot) = self.by_js.get_mut(js as usize).filter(|s| **s == e) {
            *slot = NONE;
        }
        for c in 0..self.els.len() as u32 {
            let el = &self.els[c as usize];
            if el.parent == e && !el.dead {
                self.free(c);
            }
        }
    }

    /// The exits that settled in this frame's motion tick: a removed
    /// element leaves its parent, a released one is freed, and what was
    /// released inside it goes with it.
    pub(super) fn settle_exits(&mut self) {
        let mut gone = std::mem::take(&mut self.gone);
        gone.extend(self.motion.take_exited());
        for &node in &gone {
            let Some(k) = self
                .leaving
                .iter()
                .position(|l| self.els[l.0 as usize].node == node)
            else {
                continue;
            };
            let (e, release) = self.leaving.swap_remove(k);
            if release {
                self.free(e);
            } else {
                self.arena.detach(node);
                let el = &mut self.els[e as usize];
                (el.parent, el.arena_parent) = (NONE, NONE);
            }
            self.ordered = false;
            self.paint = true;
        }
        gone.clear();
        self.gone = gone;
        // What the renderer released inside an exit that ended, and a
        // removed element held while what is under it exited.
        let mut k = 0;
        while k < self.leaving.len() {
            let (e, release) = self.leaving[k];
            let node = self.els[e as usize].node;
            let held = self.motion.exiting(node)
                || self.exiting_under(e)
                || self.leaving_in(self.els[e as usize].parent);
            if held {
                k += 1;
            } else if release {
                self.free(e);
            } else {
                self.leaving.swap_remove(k);
                self.arena.detach(node);
                let el = &mut self.els[e as usize];
                (el.parent, el.arena_parent) = (NONE, NONE);
                self.ordered = false;
                self.paint = true;
            }
        }
    }

    fn apply_ops(&mut self, bytes: &[u8]) -> Result<(), String> {
        let mut r = Reader { b: bytes, at: 0 };
        let count = r.u32()?;
        for _ in 0..count {
            let code = r.u8()?;
            match code {
                // create
                1 => {
                    let id = r.u32()?;
                    let tag = r.u8()?;
                    let kind = match tag {
                        2 => Kind::Image,
                        5 => Kind::Icon,
                        0..=7 => Kind::Box,
                        t => return Err(format!("og.ui ops: unknown tag {t}")),
                    };
                    // An `Input` shows its text in a text leaf of its own.
                    let e = self.create(id, kind, tag == 4)?;
                    self.input.per[e as usize].scroller = tag == 3;
                    if tag == 6 {
                        self.els[e as usize].line = Some(Box::default());
                    }
                    if tag == 7 {
                        self.els[e as usize].particles = Some(Box::default());
                    }
                    self.input.per[e as usize].input = tag == 4;
                    self.sem
                        .create(self.els[e as usize].node, a11y::Kind::of(tag));
                }
                // createText
                2 => {
                    let id = r.u32()?;
                    let text = r.str()?;
                    let e = self.create(id, Kind::Label, true)?;
                    self.set_text(e, text);
                    self.sem.create(self.els[e as usize].node, a11y::Kind::Text);
                }
                // setText
                3 => {
                    let e = self.op(r.u32()?)?;
                    let text = r.str()?;
                    self.set_text(e, text);
                }
                // setClass, setStyle
                4 | 5 => {
                    let e = self.op(r.u32()?)?;
                    let text = r.str()?;
                    let old = self.els[e as usize].class;
                    self.els[e as usize].class = self.classes.with(old, code == 4, text);
                    if code == 4 {
                        let slot = text.split_whitespace().find_map(|c| c.strip_prefix('@'));
                        let per = &mut self.input.per[e as usize];
                        if *per.slot != *slot.unwrap_or("") {
                            per.slot = slot.unwrap_or("").into();
                        }
                    }
                    self.data_dirty = true;
                    self.restyle(e);
                }
                // setProp
                6 => {
                    let e = self.op(r.u32()?)?;
                    let (name, value) = (r.str()?, r.str()?);
                    self.sem.prop(self.els[e as usize].node, name, value);
                    match name {
                        "disabled" => {
                            let on = !matches!(value, "" | "false" | "0");
                            self.els[e as usize].disabled = on;
                            self.set_state(e, DISABLED, on);
                        }
                        "tabIndex" => {
                            self.input.per[e as usize].tab =
                                value.trim().parse().unwrap_or(input::NO_TAB);
                        }
                        "modal" => {
                            let on = !matches!(value, "" | "false" | "0" | "null");
                            self.input.per[e as usize].modal = on;
                            self.input.modal_els.retain(|&m| m != e);
                            if on {
                                self.input.modal_els.push(e);
                            }
                        }
                        "actions" => self.input.per[e as usize].actions = value.into(),
                        "autofocus" => {
                            let on = !matches!(value, "" | "false" | "0" | "null");
                            self.input.per[e as usize].autofocus = on;
                            self.input.autofocus.retain(|&a| a != e);
                            if on {
                                self.input.autofocus.push(e);
                            }
                        }
                        "hold" => {
                            self.input.per[e as usize].hold = value.trim().parse().unwrap_or(0.0);
                        }
                        "touchStick" | "touchPad" | "touchButton" => {
                            let kind = match name {
                                "touchStick" => og_input::ZoneKind::Stick,
                                "touchPad" => og_input::ZoneKind::Pad,
                                _ => og_input::ZoneKind::Button,
                            };
                            let action = value.trim();
                            let per = &mut self.input.per[e as usize];
                            if !action.is_empty() {
                                per.zone = Some((kind, action.into()));
                            } else if per.zone.as_ref().is_some_and(|z| z.0 == kind) {
                                per.zone = None;
                            }
                            let zone = per.zone.is_some();
                            self.input.zone_els.retain(|&z| z != e);
                            if zone {
                                self.input.zone_els.push(e);
                            }
                            // A touch button squashes by the pack's press, or no longer.
                            self.restyle(e);
                        }
                        "value" | "placeholder" if self.input.per[e as usize].input => {
                            let p = &mut self.input.per[e as usize];
                            if name == "placeholder" {
                                p.placeholder = value.into();
                            } else if p.value != value {
                                // Set from outside: the caret goes to the end.
                                p.value.clear();
                                p.value.push_str(value);
                                p.caret = p.value.len();
                                p.anchor = p.caret;
                            }
                            self.show_value(e);
                        }
                        "focusGroup" => {
                            self.input.per[e as usize].group = match value.trim() {
                                "wrap" => Some(true),
                                "stop" | "true" => Some(false),
                                _ => None,
                            };
                        }
                        "focusMemory" => {
                            self.input.per[e as usize].memory =
                                !matches!(value, "" | "false" | "0" | "null");
                        }
                        "draggable" => self.input.per[e as usize].drag = value.trim().into(),
                        "dropTarget" => self.input.per[e as usize].drop = value.trim().into(),
                        "knob" => {
                            self.input.per[e as usize].rides_knob =
                                !matches!(value, "" | "false" | "0" | "null");
                            self.paint = true;
                        }
                        "deadZone" => {
                            self.input.per[e as usize].dead_zone =
                                value.trim().parse::<f32>().unwrap_or(0.0).clamp(0.0, 0.95);
                        }
                        "glyph" => {
                            self.input.per[e as usize].glyph = value.into();
                            self.input.glyphs_wanted = true;
                            // The glyph is the node's text: it lays out in
                            // a text leaf of its own, as an `Input`'s does.
                            if self.els[e as usize].leaf.is_none() {
                                let node = self.els[e as usize].node;
                                self.els[e as usize].leaf = Some(self.arena.text(node, "", 16.0));
                                self.inherit(e);
                            }
                        }
                        // A `<Line>`'s points, numbers split by commas or
                        // spaces, read into its own buffer; `viewBox` the
                        // points' space (`w h`, or SVG's `x y w h`).
                        "points" | "closed" | "viewBox" | "fit"
                            if self.els[e as usize].line.is_some() =>
                        {
                            let line = self.els[e as usize].line.as_mut().expect("a line");
                            let numbers = |v: &str| -> Vec<f32> {
                                v.split([',', ' '])
                                    .filter(|t| !t.is_empty())
                                    .map(|t| t.parse::<f32>().unwrap_or(0.0))
                                    .collect()
                            };
                            match name {
                                "closed" => line.closed = !matches!(value, "" | "false" | "0"),
                                "fit" => line.contain = value == "contain",
                                "viewBox" => {
                                    line.view = match numbers(value)[..] {
                                        [w, h] => Some([0.0, 0.0, w, h]),
                                        [x, y, w, h] => Some([x, y, w, h]),
                                        _ => None,
                                    }
                                }
                                _ => {
                                    line.points = numbers(value);
                                    line.points.truncate(line.points.len() & !1);
                                }
                            }
                            self.paint = true;
                        }
                        // A `<Particles>`' emitter, seed, delay and restart
                        // key (`particles`).
                        "emitter" | "seed" | "delay" | "restart"
                            if self.els[e as usize].particles.is_some() =>
                        {
                            self.set_particles(e, name, value);
                        }
                        // `sound`: the pack event the node sounds when it
                        // first paints (its slot's, else the pack's).
                        "sound" => {
                            self.input.per[e as usize].cue = value.into();
                            self.paint = true;
                        }
                        // `anchor` an entity's number or a world point
                        // `x,y,z` (empty: back in flow): the host places
                        // the node (§World); any other text is a UI
                        // node's `name`, which the tree places it on.
                        "anchor" => {
                            let v = value.trim();
                            let node_name = !v.is_empty()
                                && v.split([',', ' '])
                                    .any(|t| !t.is_empty() && t.parse::<f64>().is_err());
                            let n: Vec<f32> = value
                                .split([',', ' '])
                                .filter(|t| !t.is_empty())
                                .map(|t| t.parse().unwrap_or(0.0))
                                .collect();
                            let target = match n[..] {
                                _ if node_name => {
                                    Some(super::AnchorTarget::Node(self.name_index(v)))
                                }
                                [] => None,
                                [x, y, z] => Some(super::AnchorTarget::Point([x, y, z])),
                                _ => value
                                    .trim()
                                    .parse::<f64>()
                                    .ok()
                                    .map(|b| super::AnchorTarget::Entity(b as u64)),
                            };
                            let node = self.els[e as usize].node;
                            let a = self.els[e as usize]
                                .anchor
                                .get_or_insert_with(|| Box::new(super::Anchor::new()));
                            let was = a.target.is_some();
                            a.target = target;
                            match target {
                                Some(_) if was => {}
                                // Off the screen until the host places it.
                                Some(_) => {
                                    (a.shown, a.ticked, a.seen) = (false, false, None);
                                    self.anchored.push(e);
                                    self.sem.place(node, None);
                                    self.reapply(e);
                                }
                                None if was => {
                                    self.anchored.retain(|&a| a != e);
                                    self.sem.unplace(node);
                                    for name in [
                                        "clamped",
                                        "edge-top",
                                        "edge-right",
                                        "edge-bottom",
                                        "edge-left",
                                        "occluded",
                                    ] {
                                        self.mark(e, name, false);
                                    }
                                    self.reapply(e);
                                }
                                None => {}
                            }
                            self.paint = true;
                        }
                        // How the host places it (§World): `lift` a world
                        // offset, `occlusion` none, fade or hide behind
                        // geometry, `declutter` none, stack or hide over
                        // another, `priority` which keeps its place,
                        // `clamp` edge pins it inside the screen,
                        // `clampMargin` px from its edge.
                        "lift" | "occlusion" | "declutter" | "priority" | "clamp"
                        | "clampMargin" => {
                            let a = self.els[e as usize]
                                .anchor
                                .get_or_insert_with(|| Box::new(super::Anchor::new()));
                            let v = value.trim();
                            match name {
                                "lift" => {
                                    a.lift = [0.0; 3];
                                    for (l, t) in a
                                        .lift
                                        .iter_mut()
                                        .zip(v.split([',', ' ']).filter(|t| !t.is_empty()))
                                    {
                                        *l = t.parse().unwrap_or(0.0);
                                    }
                                }
                                "occlusion" => {
                                    a.occlusion = match v {
                                        "fade" => super::Occlusion::Fade,
                                        "hide" => super::Occlusion::Hide,
                                        _ => super::Occlusion::None,
                                    }
                                }
                                "declutter" => {
                                    a.declutter = match v {
                                        "stack" => super::Declutter::Stack,
                                        "hide" => super::Declutter::Hide,
                                        _ => super::Declutter::None,
                                    }
                                }
                                "priority" => a.priority = v.parse().unwrap_or(0.0),
                                "clamp" => a.clamp = v == "edge",
                                _ => a.margin = v.parse().unwrap_or(super::CLAMP_MARGIN),
                            }
                            self.paint = true;
                        }
                        // The box image's own props over its classes':
                        // `src` a texture handle, the wipe a number per tick.
                        "src" => self.set_src(e, value),
                        "wipe" | "wipeStart" | "wipeCcw" => {
                            let props = self.els[e as usize].image.get_or_insert_with(|| {
                                Box::new(super::ImageProps {
                                    handle: f64::NAN,
                                    ..Default::default()
                                })
                            });
                            let number = value.parse::<f32>().ok().filter(|n| n.is_finite());
                            match name {
                                "wipe" => props.wipe = number.map(|w| w.clamp(0.0, 1.0)),
                                "wipeStart" => props.wipe_start = number,
                                _ => {
                                    props.wipe_ccw = (!value.is_empty())
                                        .then_some(!matches!(value, "false" | "0"))
                                }
                            }
                            self.paint = true;
                        }
                        "name" if self.els[e as usize].kind == Kind::Icon => {
                            self.set_text(e, value);
                        }
                        // `name`: what `anchor` and `spotlight` find it by.
                        "name" => {
                            self.input.per[e as usize].name = value.trim().into();
                            self.paint = true;
                        }
                        // `spotlight`: the named node its ground lights.
                        "spotlight" => {
                            self.input.per[e as usize].spotlight = value.trim().into();
                            if !self.spotlights.contains(&e) {
                                self.spotlights.push(e);
                            }
                            // Off until the next frame finds its node.
                            if value.trim().is_empty() {
                                self.mark(e, "lit", false);
                            }
                            self.paint = true;
                        }
                        // A `data-*` attribute, on unless empty or false:
                        // the classes' `data-*:` variants read it here,
                        // their `group-data-*:` ones in its group's nodes.
                        n if n.starts_with("data-") => {
                            let n = &n["data-".len()..];
                            let on = !matches!(value, "" | "false");
                            let attrs = &mut self.els[e as usize].attrs;
                            match (attrs.binary_search_by(|a| (**a).cmp(n)), on) {
                                (Err(at), true) => attrs.insert(at, n.into()),
                                (Ok(at), false) => drop(attrs.remove(at)),
                                _ => {}
                            }
                            self.data_dirty = true;
                        }
                        // The rest are read by their own heads: roles and
                        // labels by a11y.
                        _ => {}
                    }
                }
                // listen
                7 => {
                    let e = self.op(r.u32()?)?;
                    let event = r.u8()?;
                    let on = r.u8()? != 0;
                    let Some(&ev) = Event::ALL.get(event as usize) else {
                        return Err(format!("og.ui ops: unknown event {event}"));
                    };
                    let el = &mut self.els[e as usize];
                    el.listens = if on {
                        el.listens | ev.bit()
                    } else {
                        el.listens & !ev.bit()
                    };
                    let was = el.clickable;
                    el.clickable = el.listens & (Event::Press.bit() | Event::Tap.bit()) != 0;
                    let (node, pressable) = (el.node, el.clickable);
                    self.sem.pressable(node, pressable);
                    if was != pressable {
                        // Its squash under a press is the pack's now, or no longer.
                        self.restyle(e);
                    }
                }
                // insert
                8 => {
                    let parent = r.u32()?;
                    let child = self.op(r.u32()?)?;
                    let before = r.u32()?;
                    let (p, pnode) = match parent {
                        0 => (NONE, 0),
                        id => {
                            let p = self.op(id)?;
                            (p, self.els[p as usize].node)
                        }
                    };
                    let before = match before {
                        u32::MAX => None,
                        id => Some(self.els[self.op(id)? as usize].node),
                    };
                    self.arena
                        .attach(self.els[child as usize].node, pnode, before);
                    let el = &mut self.els[child as usize];
                    (el.parent, el.arena_parent) = (p, p);
                    self.data_dirty = true;
                    let held = self.leaving.iter().any(|l| l.0 == child)
                        && !self.motion.exiting(self.els[child as usize].node);
                    self.leaving.retain(|l| l.0 != child);
                    let el = &self.els[child as usize];
                    self.sem.exiting(el.node, false);
                    if held {
                        // Put back while what is under it exits: they turn back.
                        for d in 0..self.els.len() as u32 {
                            let el = &self.els[d as usize];
                            if !el.dead && self.motion.exiting(el.node) && self.is_under(d, child) {
                                self.motion.enter(el.node, &el.record, None);
                            }
                        }
                    }
                    self.ordered = false;
                    self.paint = true;
                    self.reapply(child);
                    // Into a parent already on screen it enters from its
                    // `starting:` record; the first build appears as it is.
                    let shown = match p {
                        NONE => !self.abs.is_empty(),
                        p => self.abs.get(p as usize).is_some_and(|a| a.2),
                    };
                    let el = &self.els[child as usize];
                    let from = self.classes.presence(el.class, 0).filter(|_| shown);
                    // One put back while it exits turns back from where it
                    // is, and restyles as a drawn node does.
                    if !self.motion.exiting(el.node) {
                        let el = &mut self.els[child as usize];
                        if el.entering.replace(shown).is_none() {
                            self.entering.push(child);
                        }
                    }
                    let el = &self.els[child as usize];
                    self.motion.enter(el.node, &el.record, from);
                    if shown {
                        self.enter_subtree(child);
                    }
                }
                // remove
                9 => {
                    let _parent = r.u32()?;
                    let child = self.op(r.u32()?)?;
                    let el = &self.els[child as usize];
                    // With an `exit:` record it stays drawn, not hit,
                    // until its exit settles.
                    let to = self.classes.presence(el.class, 1);
                    if self.motion.exit(el.node, &el.record, to) || self.exit_under(child) {
                        self.sem.exiting(self.els[child as usize].node, true);
                        self.leaving.push((child, false));
                        self.paint = true;
                        continue;
                    }
                    self.arena.detach(self.els[child as usize].node);
                    let el = &mut self.els[child as usize];
                    (el.parent, el.arena_parent) = (NONE, NONE);
                    self.ordered = false;
                    self.paint = true;
                }
                // release
                10 => {
                    let e = self.op(r.u32()?)?;
                    self.release(e);
                    self.ordered = false;
                    self.paint = true;
                }
                // placeholder: an anchor that takes no room
                11 => {
                    let e = self.create(r.u32()?, Kind::Box, false)?;
                    self.sem
                        .create(self.els[e as usize].node, a11y::Kind::Placeholder);
                    self.els[e as usize].hidden = 1;
                    self.inherit(e);
                }
                // strings: a string table the bundle's components import,
                // for the host to register as base strings
                12 => {
                    let id = r.str()?;
                    self.strings.push(id.to_owned());
                }
                c => return Err(format!("og.ui ops: unknown op {c}")),
            }
        }
        if r.at != bytes.len() {
            return Err("og.ui ops: bytes after the last op".into());
        }
        Ok(())
    }

    /// Applies an op batch (after [`MAGIC`]) in order. A refused op stops
    /// the batch; the ones before it stay applied, and the data contexts
    /// follow them.
    pub fn apply(&mut self, bytes: &[u8]) -> Result<(), String> {
        let applied = self.apply_ops(bytes);
        self.refresh_data();
        applied
    }

    /// Works out each op node's data context again when a batch changed a
    /// `data-*` prop, a class or the tree's shape: its own `data-*` names,
    /// then each of its nearest ancestor groups' as `group/name`, and
    /// restyles the nodes whose context changed.
    pub(super) fn refresh_data(&mut self) {
        if !std::mem::take(&mut self.data_dirty) {
            return;
        }
        let mut data = String::new();
        // The group names met, nearest first: a farther group of one is hidden.
        let mut seen: Vec<String> = Vec::new();
        for e in 0..self.els.len() as u32 {
            let el = &self.els[e as usize];
            if el.js == NONE || el.dead {
                continue;
            }
            data.clear();
            seen.clear();
            for a in &el.attrs {
                data.push_str(a);
                data.push(' ');
            }
            let mut p = el.parent;
            while p != NONE {
                let pe = &self.els[p as usize];
                for g in self.classes.groups(pe.class) {
                    if seen.iter().any(|s| s == g) {
                        continue;
                    }
                    seen.push(g.to_owned());
                    for a in &pe.attrs {
                        data.push_str(g);
                        data.push('/');
                        data.push_str(a);
                        data.push(' ');
                    }
                }
                p = pe.parent;
            }
            let id = self.classes.data_id(data.trim_end());
            let other = self.other_scheme(e);
            let el = &mut self.els[e as usize];
            if id != el.data || other != el.other {
                (el.data, el.other) = (id, other);
                self.restyle(e);
            }
        }
    }

    /// Writes the events since the last call, after [`EVENTS`]: a u32
    /// count, then each one's id (u32), event (u8), x and y (f32), key and
    /// value (u32 length and UTF-8).
    /// The string tables the batches since the last call named (op 12), by
    /// bundle id: the host registers each as base strings.
    pub fn take_strings(&mut self) -> Vec<String> {
        std::mem::take(&mut self.strings)
    }

    pub fn take_events(&mut self, out: &mut Vec<u8>) {
        out.extend_from_slice(EVENTS);
        out.extend_from_slice(&(self.events.len() as u32).to_le_bytes());
        for f in self.events.drain(..) {
            out.extend_from_slice(&f.id.to_le_bytes());
            out.push(f.event as u8);
            out.extend_from_slice(&f.x.to_le_bytes());
            out.extend_from_slice(&f.y.to_le_bytes());
            for s in [&f.key, &f.value] {
                out.extend_from_slice(&(s.len() as u32).to_le_bytes());
                out.extend_from_slice(s.as_bytes());
            }
        }
    }
}

/// An op node as the probe lists it ([`Tree::dump_ops`]).
#[derive(Clone, Debug, PartialEq)]
pub struct OpNode<'a> {
    pub id: u32,
    pub parent: u32,
    pub node: Id,
    pub rect: [f32; 4],
    /// The part of it a scroller or a box cutting its overflow lets show
    /// on screen (x0, y0, x1, y1): `rect`, or under a transform the box
    /// bounding `rect` drawn through it, cut to `clip`.
    pub visible: [f32; 4],
    /// The rectangle on screen it is drawn cut to (x0, y0, x1, y1), by
    /// its ancestors' scrollers and overflow; `None` when nothing cuts it.
    pub clip: Option<[f32; 4]>,
    /// Whether `clip` cuts any of it.
    pub clipped: bool,
    pub shown: bool,
    pub text: &'a str,
    /// Where its laid-out lines cut a word with no UAX #14 break there,
    /// each as `end of the line|start of the next`: a text measured
    /// narrower than it draws. Empty: none.
    pub broken: Vec<String>,
    /// Its text as drawn: the ink (linear RGB, alpha with its opacity),
    /// the font size in CSS px, and each line's box (x, y, width, height:
    /// the em at its baseline) before `transform`. Empty lines: it draws
    /// no text.
    pub ink: [f32; 4],
    pub size: f32,
    pub lines: Vec<[f32; 4]>,
    /// The transform its lines are drawn through, `[a, b, c, d, tx, ty]`
    /// (a point `(x, y)` lands at `(a x + c y + tx, b x + d y + ty)`), when
    /// it or an ancestor has one: a line's quad on screen is its box
    /// through it.
    pub transform: Option<[f32; 6]>,
}

impl Tree {
    /// The accessibility update to send `client` after this frame, when
    /// what it shows changed: the arena's nodes with their semantics, at
    /// the viewport's size and `scale` (physical px per CSS px). `focus`:
    /// the arena node the keyboard's focus is on.
    pub fn a11y(
        &self,
        client: &mut a11y::Tree,
        focus: Option<Id>,
        scale: f32,
    ) -> Option<accesskit::TreeUpdate> {
        let (width, height) = self.size;
        let viewport = a11y::Viewport {
            width,
            height,
            scale,
        };
        client.update(&self.arena, &self.sem, focus, viewport)
    }

    /// Every live op node, for the probe: its op id, its op parent's (0 at
    /// a mount's top), its arena node, its box on screen (x, y, width,
    /// height) as last painted, whether it shows, and its text. What is on
    /// its way out (an exit playing) is gone already, as for a reader.
    pub fn dump_ops(&self) -> Vec<OpNode<'_>> {
        self.els
            .iter()
            .enumerate()
            .filter(|&(e, el)| !el.dead && el.js != NONE && !self.leaving_in(e as u32))
            .filter_map(|(e, el)| {
                let &(x, y, shown, _) = self.abs.get(e)?;
                let b = self.arena.box_of(el.node);
                let transform = self
                    .xfs
                    .get(e)
                    .copied()
                    .filter(|&xf| xf != 0)
                    .and_then(|xf| self.painted_transforms.get(xf as usize))
                    .map(|&[a, b, c, d, tx, ty, _, _]| [a, b, c, d, tx, ty]);
                // Its box on screen: through its transform, the box that
                // bounds it there, where the cuts (screen space) apply.
                let own = match transform {
                    None => [x, y, x + b.w, y + b.h],
                    Some([ma, mb, mc, md, me, mf]) => {
                        let mut o = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
                        for [cx, cy] in [[x, y], [x + b.w, y], [x, y + b.h], [x + b.w, y + b.h]] {
                            let (sx, sy) = (ma * cx + mc * cy + me, mb * cx + md * cy + mf);
                            o = [o[0].min(sx), o[1].min(sy), o[2].max(sx), o[3].max(sy)];
                        }
                        o
                    }
                };
                // An anchored node escapes its ancestors' clips.
                let clip = match el.arena_parent {
                    _ if el.anchored().is_some() => None,
                    NONE => None,
                    p => self.cuts.get(p as usize).copied().filter(|c| {
                        c[0] > f32::MIN || c[1] > f32::MIN || c[2] < f32::MAX || c[3] < f32::MAX
                    }),
                };
                let cut = clip.unwrap_or(own);
                let visible = [
                    own[0].max(cut[0]),
                    own[1].max(cut[1]),
                    own[2].min(cut[2]).max(own[0].max(cut[0])),
                    own[3].min(cut[3]).max(own[1].max(cut[1])),
                ];
                let clipped = visible != own;
                // Its text as the last paint laid it out.
                let glyphs = el
                    .leaf
                    .filter(|_| !el.text.is_empty())
                    .and_then(|leaf| self.texts.leaf(leaf))
                    .filter(|g| !g.fill.is_empty() && *g.text == *el.text);
                let parent = match el.parent {
                    NONE => 0,
                    p => self.els[p as usize].js,
                };
                Some(OpNode {
                    id: el.js,
                    parent,
                    node: el.node,
                    rect: [x, y, b.w, b.h],
                    visible,
                    clip,
                    clipped,
                    shown,
                    text: &el.text,
                    broken: glyphs.map_or(Vec::new(), crate::text::Leaf::broken_words),
                    ink: self.shown.get(e).map_or([0.0; 4], |s| {
                        [s.ink[0], s.ink[1], s.ink[2], s.ink[3] * s.alpha]
                    }),
                    size: glyphs.map_or(0.0, |g| g.size()),
                    lines: glyphs.zip(el.leaf).map_or(Vec::new(), |(g, leaf)| {
                        let lb = self.arena.box_of(leaf);
                        let size = g.size();
                        g.lines
                            .iter()
                            .map(|&[dx, baseline, w]| {
                                [x + lb.x + dx, y + lb.y + baseline - 0.8 * size, w, size]
                            })
                            .collect()
                    }),
                    transform,
                })
            })
            .collect()
    }

    /// The accessibility tree now, built whole ([`a11y::snapshot`]), with
    /// the focus where input put it: each arena node's facts and the focus
    /// order. For readers that ask once (the probe).
    pub fn a11y_snapshot(&self) -> (rustc_hash::FxHashMap<Id, a11y::Facts>, Vec<Id>) {
        let (width, height) = self.size;
        let viewport = a11y::Viewport {
            width,
            height,
            scale: 1.0,
        };
        a11y::snapshot(&self.arena, &self.sem, self.focused(), viewport)
    }

    /// The viewport the tree lays out in, CSS px.
    pub fn size(&self) -> (f32, f32) {
        self.size
    }

    /// Op node `id`'s box (x, y, width, height) as last painted, in the
    /// tree's units, when it shows.
    pub fn op_box(&self, id: u32) -> Option<[f32; 4]> {
        let e = self.op(id).ok()? as usize;
        let &(x, y, shown, _) = self.abs.get(e)?;
        let b = self.arena.box_of(self.els[e].node);
        shown.then_some([x, y, b.w, b.h])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An op batch as the TS renderer writes it.
    #[derive(Default)]
    struct Batch(Vec<u8>, u32);

    impl Batch {
        fn op(&mut self, code: u8) -> &mut Self {
            self.1 += 1;
            self.0.push(code);
            self
        }
        fn u32(&mut self, v: u32) -> &mut Self {
            self.0.extend_from_slice(&v.to_le_bytes());
            self
        }
        fn u8(&mut self, v: u8) -> &mut Self {
            self.0.push(v);
            self
        }
        fn str(&mut self, s: &str) -> &mut Self {
            self.u32(s.len() as u32);
            self.0.extend_from_slice(s.as_bytes());
            self
        }
        fn bytes(&self) -> Vec<u8> {
            let mut out = self.1.to_le_bytes().to_vec();
            out.extend_from_slice(&self.0);
            out
        }
    }

    fn tree() -> Tree {
        let mut t = Tree::new();
        t.resize(400.0, 300.0);
        // `row` lays its children in a row; `b` is 50 by 20.
        t.set_resolver(Box::new(|class, _, _, _| {
            let mut s = taffy::Style::default();
            for c in class.split_whitespace() {
                match c {
                    "row" => s.flex_direction = taffy::FlexDirection::Row,
                    "b" => s.size = taffy::Size::from_lengths(50.0, 20.0),
                    "big" => s.size = taffy::Size::from_lengths(400.0, 300.0),
                    "col" => s.flex_direction = taffy::FlexDirection::Column,
                    "fixed" => s.flex_shrink = 0.0,
                    "wide" => s.size = taffy::Size::from_lengths(600.0, 20.0),
                    _ => {}
                }
            }
            let mut r = Record {
                layout: rune_taffy::CompactStyle::from(&s),
                ..Record::default()
            };
            // `half`: drawn at half its size about its centre.
            if class.split_whitespace().any(|c| c == "half") {
                r.skin.transform.scale = [0.5, 0.5];
            }
            // `up`: drawn 40 px higher; `left`: 30 px to the left; `clip`:
            // cuts what overflows it;
            // `ink`: a white fill.
            for c in class.split_whitespace() {
                match c {
                    "up" => r.skin.transform.translate = [0.0, -40.0],
                    "left" => r.skin.transform.translate = [-30.0, 0.0],
                    "clip" => {
                        let mut l = taffy::Style::from(&r.layout);
                        l.overflow = taffy::Point {
                            x: taffy::Overflow::Hidden,
                            y: taffy::Overflow::Hidden,
                        };
                        r.layout = rune_taffy::CompactStyle::from(&l);
                    }
                    "ink" => r.skin.fill = [1.0; 4],
                    _ => {}
                }
            }
            r
        }));
        t
    }

    thread_local! {
        /// The default Latin face as the sans, and a glyph atlas: what a
        /// test's frames lay text out from and raster into.
        static TEXT: std::cell::RefCell<(og_text::fonts::Collection, og_draw::glyphs::Atlas)> =
            std::cell::RefCell::new((crate::text::tests::latin(), Default::default()));
    }

    fn present(t: &mut Tree) -> (bool, bool) {
        TEXT.with_borrow_mut(|(fonts, atlas)| t.present(fonts, atlas, &IconAtlas::default()))
    }

    fn frame(t: &mut Tree) {
        present(t);
    }

    /// Nodes a batch makes lay out where their inserts put them (`before`
    /// moves one ahead of its sibling); a press and release over a node
    /// that listens fires its press with its id; a released subtree no
    /// longer shows or answers.
    #[test]
    fn a_batch_builds_lays_out_presses_and_releases() {
        let mut t = tree();
        let mut b = Batch::default();
        b.op(1).u32(1).u8(0); // view 1
        b.op(4).u32(1).str("row");
        b.op(1).u32(2).u8(0); // view 2
        b.op(4).u32(2).str("b");
        b.op(1).u32(3).u8(0); // view 3
        b.op(4).u32(3).str("b");
        b.op(7).u32(3).u8(Event::Press as u8).u8(1);
        b.op(8).u32(0).u32(1).u32(u32::MAX);
        b.op(8).u32(1).u32(2).u32(u32::MAX);
        // 3 goes before 2.
        b.op(8).u32(1).u32(3).u32(2);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let (two, three) = (t.op_box(2).unwrap(), t.op_box(3).unwrap());
        assert_eq!(three, [0.0, 0.0, 50.0, 20.0]);
        assert_eq!(two, [50.0, 0.0, 50.0, 20.0]);
        // 2 does not listen: a press on it fires nothing.
        t.pointer(75.0, 10.0, true, false);
        t.pointer(75.0, 10.0, false, true);
        assert!(t.events.is_empty());
        t.pointer(25.0, 10.0, true, false);
        t.pointer(25.0, 10.0, false, true);
        assert_eq!(t.events, [Fired::at(3, Event::Press, 25.0, 10.0)]);
        let mut reply = Vec::new();
        t.take_events(&mut reply);
        assert_eq!(&reply[..4], EVENTS);
        assert_eq!(u32::from_le_bytes(reply[4..8].try_into().unwrap()), 1);
        assert!(t.events.is_empty());
        let mut b = Batch::default();
        b.op(10).u32(1);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        assert_eq!((t.op_box(1), t.op_box(3)), (None, None));
        t.pointer(25.0, 10.0, true, false);
        t.pointer(25.0, 10.0, false, true);
        assert!(t.events.is_empty(), "a released node answers nothing");
    }

    /// A text and a class change on a laid-out tree patch only what they
    /// name: the second same class string resolves nothing new.
    #[test]
    fn a_class_string_resolves_once() {
        let mut t = tree();
        let mut b = Batch::default();
        b.op(1).u32(1).u8(0);
        b.op(4).u32(1).str("b");
        b.op(1).u32(2).u8(0);
        b.op(4).u32(2).str("b");
        b.op(8).u32(0).u32(1).u32(u32::MAX);
        b.op(8).u32(0).u32(2).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        // "" and "b".
        assert_eq!(t.classes.keys.len(), 2);
        let mut b = Batch::default();
        b.op(4).u32(1).str("row b");
        b.op(4).u32(1).str("b");
        t.apply(&b.bytes()).unwrap();
        assert_eq!(t.classes.keys.len(), 3);
    }

    /// A `<Line>` (tag 6) paints one Line quad: its points in its box, its
    /// stroke's width, colour and closing; a line under 1 px or with one
    /// point paints none.
    #[test]
    fn a_line_paints_its_points_as_one_stroke() {
        let mut t = tree();
        t.set_resolver(Box::new(|class, _, _, _| {
            let mut r = Record::default();
            if class.contains("w2") {
                r.stroke_width = 2.0;
                r.stroke = Some([1.0, 0.0, 0.0, 1.0]);
            }
            if class.contains("thin") {
                r.stroke_width = 0.5;
            }
            r
        }));
        let mut b = Batch::default();
        b.op(1).u32(1).u8(6);
        b.op(4).u32(1).str("w2");
        b.op(6).u32(1).str("points").str("0,10 5 0, 9.5,10 7");
        b.op(6).u32(1).str("closed").str("true");
        b.op(1).u32(2).u8(6);
        b.op(4).u32(2).str("thin");
        b.op(6).u32(2).str("points").str("0 0 4 4");
        b.op(8).u32(0).u32(1).u32(u32::MAX);
        b.op(8).u32(0).u32(2).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let lines: Vec<_> = t
            .drawn
            .quads
            .iter()
            .filter(|q| q.kind == PaintKind::Line as u32)
            .collect();
        assert_eq!(lines.len(), 1, "the thin line paints nothing");
        let q = lines[0];
        // An odd number is dropped: three points.
        assert_eq!(
            (q.b, q.p[0], q.p[1], q.color),
            (3, 2.0, 1.0, [1.0, 0.0, 0.0, 1.0])
        );
        let pts = &t.drawn.points[q.a as usize..(q.a + q.b) as usize];
        assert_eq!(pts, [[0.0, 10.0], [5.0, 0.0], [9.5, 10.0]]);
    }

    /// An anchored node leaves the flow and sits where the host places it:
    /// it paints, takes presses and is in the accessibility tree there, and
    /// a target off the screen hides it from all three.
    #[test]
    fn an_anchored_node_sits_where_the_host_places_it() {
        let mut t = tree();
        let mut b = Batch::default();
        b.op(1).u32(1).u8(0);
        b.op(4).u32(1).str("row");
        for id in [2, 3] {
            b.op(1).u32(id).u8(0);
            b.op(4).u32(id).str("b");
            b.op(8).u32(1).u32(id).u32(u32::MAX);
        }
        b.op(6).u32(2).str("anchor").str("3,4,5");
        b.op(6).u32(2).str("lift").str("0 2 0");
        b.op(6).u32(2).str("role").str("button");
        b.op(6).u32(2).str("label").str("\"Talk\"");
        b.op(7).u32(2).u8(Event::Press as u8).u8(1);
        b.op(8).u32(0).u32(1).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        let asked = std::cell::RefCell::new(Vec::new());
        let place = |t: &mut Tree, at: Option<[f32; 2]>| {
            t.place_anchors(|target, lift, _| {
                asked.borrow_mut().push((target, lift));
                at.map(|at| super::super::Sighting {
                    at,
                    ..Default::default()
                })
            });
            frame(t);
        };
        place(&mut t, Some([120.0, 60.0]));
        assert_eq!(
            *asked.borrow(),
            [(
                super::super::AnchorTarget::Point([3.0, 4.0, 5.0]),
                [0.0, 2.0, 0.0]
            )]
        );
        let node = |t: &Tree, id: u32| {
            let n = t.dump_ops().into_iter().find(|n| n.id == id).unwrap();
            (n.rect, n.shown)
        };
        assert_eq!(node(&t, 2), ([120.0, 60.0, 50.0, 20.0], true));
        assert_eq!(
            node(&t, 3).0[0],
            0.0,
            "the anchored node takes no room in the row"
        );
        let two = t.els[t.op(2).unwrap() as usize].node;
        let (facts, _) = t.a11y_snapshot();
        assert_eq!(facts[&two].role, "button");
        t.pointer(130.0, 65.0, true, false);
        t.pointer(130.0, 65.0, false, true);
        assert_eq!(t.events.len(), 1, "a press where it is placed reaches it");
        t.take_events(&mut Vec::new());
        // Behind the camera, and in front but off the surface: hidden.
        for at in [None, Some([-10.0, 60.0])] {
            place(&mut t, at);
            assert!(!node(&t, 2).1, "{at:?}");
            assert!(!t.a11y_snapshot().0.contains_key(&two), "{at:?}");
            t.pointer(130.0, 65.0, true, false);
            t.pointer(130.0, 65.0, false, true);
            assert!(t.events.is_empty(), "{at:?}");
        }
    }

    /// Anchored 50×20 boxes `(id, props)`, each sighted where `at` puts
    /// its id; `class` adds classes to each.
    fn anchored(t: &mut Tree, class: &str, nodes: &[(u32, &[(&str, &str)])]) {
        let mut b = Batch::default();
        for &(id, props) in nodes {
            b.op(1).u32(id).u8(0);
            b.op(4).u32(id).str(&format!("b {class}"));
            // Placement props before `anchor` hold too.
            for &(k, v) in props {
                b.op(6).u32(id).str(k).str(v);
            }
            b.op(6).u32(id).str("anchor").str(&id.to_string());
            b.op(8).u32(0).u32(id).u32(u32::MAX);
        }
        t.apply(&b.bytes()).unwrap();
    }

    /// `animate-float` on an anchored node mounted into a tree whose last
    /// sample is long past (an idle HUD's damage number) starts when the
    /// tick first shows it: at 200 ms it is drawn, still mostly opaque.
    /// Way this could fail: the loop timed from the motion store's last
    /// sample, so it starts finished and draws nothing.
    #[test]
    fn a_keyframe_loop_starts_at_the_tick_that_shows_it() {
        let mut t = real();
        t.set_time(0.1, false);
        frame(&mut t);
        t.set_time(5.0, false);
        anchored(
            &mut t,
            "w-[40px] h-[20px] bg-white animate-float",
            &[(1, &[])],
        );
        let alpha = |t: &mut Tree, at: f64| -> Vec<f32> {
            t.set_time(at, false);
            t.place_anchors(|_, _, _| sighted([100.0, 100.0], 1.0));
            frame(t);
            t.drawn.quads.iter().map(|q| q.color[3]).collect()
        };
        assert_eq!(alpha(&mut t, 5.0), [1.0], "its first frame");
        let a = alpha(&mut t, 5.2);
        assert!(a.len() == 1 && a[0] > 0.5, "200 ms in: {a:?}");
        assert_eq!(placed(&t, 1), Some([100.0, 100.0]));
    }

    fn placed(t: &Tree, id: u32) -> Option<[f32; 2]> {
        let n = t.dump_ops().into_iter().find(|n| n.id == id).unwrap();
        n.shown.then_some([n.rect[0], n.rect[1]])
    }

    fn sighted(at: [f32; 2], depth: f32) -> Option<super::super::Sighting> {
        Some(super::super::Sighting {
            at,
            depth,
            ..Default::default()
        })
    }

    /// `declutter="stack"` takes the anchored nodes by priority, then the
    /// nearer first, each moved straight up clear of those before it (a
    /// gap of 2 px), and hidden when that leaves the top; `hide` hides one
    /// that overlaps; a node without `declutter` neither moves nor blocks.
    #[test]
    fn decluttered_anchors_stack_by_priority_then_depth() {
        let mut t = tree();
        let stack: &[(&str, &str)] = &[("declutter", "stack")];
        anchored(
            &mut t,
            "",
            &[
                (1, &[("declutter", "stack"), ("priority", "1")]),
                (2, stack),
                (3, stack),
                (4, &[]),
                (5, &[("declutter", "hide")]),
                (6, stack),
                (7, stack),
                (8, stack),
            ],
        );
        // id: (at, depth). 1 is the farthest of 1 to 5 but ranks first.
        let at = |id: u64| match id {
            1 => ([100.0, 100.0], 9.0),
            2 => ([100.0, 100.0], 3.0),
            3 => ([100.0, 100.0], 6.0),
            4 => ([100.0, 100.0], 1.0),
            5 => ([100.0, 100.0], 7.0),
            6 => ([100.0, 30.0], 8.0),
            7 => ([100.0, 10.0], 8.5),
            _ => ([100.0, 5.0], 8.7),
        };
        t.place_anchors(|target, _, _| {
            let super::super::AnchorTarget::Entity(id) = target else {
                unreachable!()
            };
            let (p, d) = at(id);
            sighted(p, d)
        });
        frame(&mut t);
        assert_eq!(
            placed(&t, 1),
            Some([100.0, 100.0]),
            "the first keeps its place"
        );
        assert_eq!(
            placed(&t, 2),
            Some([100.0, 78.0]),
            "the nearer of the rest over it"
        );
        assert_eq!(placed(&t, 3), Some([100.0, 56.0]), "then over both");
        assert_eq!(
            placed(&t, 4),
            Some([100.0, 100.0]),
            "no declutter: it stays"
        );
        assert_eq!(placed(&t, 5), None, "hide: it overlaps the first");
        assert_eq!(placed(&t, 6), Some([100.0, 30.0]), "clear of 3 by 6 px");
        assert_eq!(placed(&t, 7), Some([100.0, 8.0]), "over 6, inside the top");
        assert_eq!(placed(&t, 8), None, "over 7 would leave the top");
    }

    /// `clamp="edge"` pins an anchored node whose drawn box would leave
    /// the margin (24 px by default) where the line from the screen's
    /// centre toward its target meets it, behind the camera too, and marks
    /// it `data-clamped` and `data-edge-<side>`; one inside stays put and
    /// unmarked. Its drawn box (after its transform) is what stays inside.
    #[test]
    fn an_edge_clamped_anchor_pins_inside_the_margin_toward_its_target() {
        let mut t = tree();
        t.set_resolver(Box::new(|class, _, _, data| {
            let mut r = Record::default();
            let mut s = taffy::Style::default();
            if class.contains('b') {
                s.size = taffy::Size::from_lengths(50.0, 20.0);
            }
            r.layout = rune_taffy::CompactStyle::from(&s);
            if class.contains("up") {
                r.skin.transform.translate = [0.0, -40.0];
            }
            r.font_size = match data {
                "clamped edge-right" => 30.0,
                "clamped edge-bottom" => 31.0,
                "" => 16.0,
                _ => 0.0,
            };
            r
        }));
        anchored(
            &mut t,
            "",
            &[(1, &[("clamp", "edge")]), (3, &[("clamp", "edge")])],
        );
        anchored(
            &mut t,
            "up",
            &[(2, &[("clampMargin", "24"), ("clamp", "edge")])],
        );
        let place = |t: &mut Tree, out: bool| {
            t.place_anchors(|target, _, _| {
                let super::super::AnchorTarget::Entity(id) = target else {
                    unreachable!()
                };
                Some(match (id, out) {
                    (1, true) => super::super::Sighting {
                        at: [1000.0, 150.0],
                        ..Default::default()
                    },
                    // Behind the camera, the way it would turn: down.
                    (2, true) => super::super::Sighting {
                        at: [200.0, 600.0],
                        behind: true,
                        ..Default::default()
                    },
                    _ => super::super::Sighting {
                        at: [100.0, 100.0],
                        ..Default::default()
                    },
                })
            });
            frame(t);
        };
        let size = |t: &Tree, id: usize| t.els[t.by_js[id] as usize].record.font_size;
        // Two placings: the second has the boxes laid out.
        place(&mut t, true);
        place(&mut t, true);
        // 1's box centre (1025, 160) seen from (200, 150): x has 151 px of
        // room over an 825 px run, so it is 0.183 of the way.
        let [x, y] = placed(&t, 1).unwrap();
        assert!(
            (x - 326.0).abs() < 0.01 && (y - 141.83).abs() < 0.01,
            "{x}, {y}"
        );
        assert_eq!(size(&t, 1), 30.0, "clamped, on the right edge");
        // 2 is drawn 40 px above its top left: its drawn box's bottom
        // (top left + 20 - 40) sits on the margin, 276.
        let [x, y] = placed(&t, 2).unwrap();
        assert!(
            (x - 181.905).abs() < 0.01 && (y - 296.0).abs() < 0.01,
            "{x}, {y}"
        );
        assert_eq!(size(&t, 2), 31.0, "clamped, on the bottom edge");
        assert_eq!(placed(&t, 3), Some([100.0, 100.0]), "inside: put");
        assert_eq!(size(&t, 3), 16.0, "and unmarked");
        place(&mut t, false);
        assert_eq!(placed(&t, 1), Some([100.0, 100.0]));
        assert_eq!(size(&t, 1), 16.0, "back inside, unmarked");
    }

    /// `occlusion` asks the host, at the tick only, whether geometry hides
    /// the target: `fade` draws it at 0.35 (`data-occluded`), `hide` hides
    /// it, none never asks; an edge-clamped one is never occluded, and a
    /// frame between ticks keeps the tick's answer.
    #[test]
    fn an_occluded_anchor_fades_or_hides_as_the_tick_saw_it() {
        let mut t = tree();
        anchored(
            &mut t,
            "ink",
            &[
                (1, &[("occlusion", "fade")]),
                (2, &[("occlusion", "hide")]),
                (3, &[]),
                (4, &[("occlusion", "fade"), ("clamp", "edge")]),
            ],
        );
        let asked = std::cell::RefCell::new(Vec::new());
        let place = |t: &mut Tree, tick: bool, behind_wall: bool| {
            let project = |target, _, ask| {
                let super::super::AnchorTarget::Entity(id) = target else {
                    unreachable!()
                };
                asked.borrow_mut().push((id, ask));
                let x = if id == 4 { 1000.0 } else { 60.0 * id as f32 };
                Some(super::super::Sighting {
                    at: [x, 100.0],
                    occluded: ask && behind_wall,
                    ..Default::default()
                })
            };
            if tick {
                t.place_anchors(project);
            } else {
                t.follow_anchors(project);
            }
            frame(t);
        };
        let fill = |t: &Tree, id: u32| {
            let [x, y] = placed(t, id)?;
            t.drawn
                .quads
                .iter()
                .find(|q| q.rect[0] == x && q.rect[1] == y)
                .map(|q| q.color[3])
        };
        place(&mut t, true, true);
        assert_eq!(
            *asked.borrow(),
            [(1, true), (2, true), (3, false), (4, true)],
            "only the nodes with occlusion are asked"
        );
        assert_eq!(fill(&t, 1), Some(super::super::OCCLUDED_ALPHA));
        assert_eq!(placed(&t, 2), None, "hide");
        assert_eq!(fill(&t, 3), Some(1.0), "no occlusion");
        assert_eq!(fill(&t, 4), Some(1.0), "clamped: never occluded");
        assert!(
            t.els[t.by_js[1] as usize]
                .attrs
                .iter()
                .any(|a| &**a == "occluded")
        );
        asked.borrow_mut().clear();
        place(&mut t, false, false);
        assert!(
            asked.borrow().iter().all(|&(_, ask)| !ask),
            "never between ticks"
        );
        assert_eq!(
            fill(&t, 1),
            Some(super::super::OCCLUDED_ALPHA),
            "the tick's answer holds"
        );
        place(&mut t, true, false);
        assert_eq!(fill(&t, 1), Some(1.0));
        assert_eq!(fill(&t, 2), Some(1.0), "in sight again");
    }

    /// A `<Line>`'s `viewBox` maps its points onto the laid-out box as it
    /// paints: stretched by default, and with `fit="contain"` scaled
    /// uniformly and centred, an SVG view's min-x shifting them; the
    /// stroke's width stays in px.
    #[test]
    fn a_line_maps_its_view_box_onto_its_box() {
        let mut t = tree();
        t.set_resolver(Box::new(|_, _, _, _| {
            let s = taffy::Style {
                size: taffy::Size::from_lengths(200.0, 100.0),
                flex_shrink: 0.0,
                ..taffy::Style::default()
            };
            Record {
                layout: rune_taffy::CompactStyle::from(&s),
                stroke_width: 2.0,
                stroke: Some([1.0; 4]),
                ..Record::default()
            }
        }));
        let mut b = Batch::default();
        for (id, view, fit) in [(1, "100 100", ""), (2, "10 0 100 100", "contain")] {
            b.op(1).u32(id).u8(6);
            b.op(4).u32(id).str("line");
            b.op(6).u32(id).str("points").str("10 0 110 50");
            b.op(6).u32(id).str("viewBox").str(view);
            b.op(6).u32(id).str("fit").str(fit);
            b.op(8).u32(0).u32(id).u32(u32::MAX);
        }
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let lines: Vec<_> = t
            .drawn
            .quads
            .iter()
            .filter(|q| q.kind == PaintKind::Line as u32)
            .map(|q| (q.p[0], &t.drawn.points[q.a as usize..(q.a + q.b) as usize]))
            .collect();
        assert_eq!(
            lines,
            [
                (2.0, &[[20.0, 0.0], [220.0, 50.0]][..]),
                (2.0, &[[50.0, 0.0], [150.0, 50.0]][..]),
            ]
        );
    }

    /// An `Image`'s `src` draws its texture over its box once the host
    /// resolves the handle, and its wipe props lie over the classes' image
    /// (a pack's nine-slice keeps its slices); a node with neither draws
    /// no image.
    #[test]
    fn image_props_lie_over_the_classes_image() {
        let mut t = tree();
        t.set_resolver(Box::new(|class, _, _, _| {
            let mut r = Record::default();
            if class.contains("nine") {
                r.skin.image.texture = 4;
                r.skin.image.slice = [3.0; 4];
            }
            r
        }));
        let mut b = Batch::default();
        b.op(1).u32(1).u8(2);
        b.op(4).u32(1).str("w-4 h-4");
        b.op(6).u32(1).str("src").str("4294967298");
        b.op(1).u32(2).u8(0);
        b.op(4).u32(2).str("nine");
        b.op(6).u32(2).str("wipe").str("0.25");
        b.op(6).u32(2).str("wipeStart").str("1.5");
        b.op(6).u32(2).str("wipeCcw").str("true");
        b.op(1).u32(3).u8(0);
        b.op(6).u32(3).str("wipe").str("0.5");
        for id in 1..=3 {
            b.op(8).u32(0).u32(id).u32(u32::MAX);
        }
        t.apply(&b.bytes()).unwrap();
        let images = |t: &Tree| -> Vec<(u32, [f32; 4], [f32; 4])> {
            t.drawn
                .quads
                .iter()
                .filter(|q| q.kind == PaintKind::Image as u32)
                .map(|q| (q.a, q.wipe, q.p))
                .collect()
        };
        frame(&mut t);
        assert_eq!(images(&t).len(), 1, "an unresolved src draws nothing yet");
        t.set_textures(Box::new(|h| {
            (if h == 4294967298.0 { 8 } else { 0 }, [0.0; 2])
        }));
        frame(&mut t);
        let tex = |slot: u32| og_draw::TextureHandle::from_slot(slot).unwrap().to_bits();
        let got = images(&t);
        assert_eq!(got.len(), 2, "{got:?}");
        assert_eq!(
            (got[0].0, got[0].1),
            (tex(8), [0.0, 0.0, 0.0, 0.0]),
            "src, whole"
        );
        assert_eq!(
            (got[1].0, got[1].1),
            (tex(4), [0.75, 1.5, 1.0, 0.0]),
            "the wipe over nine"
        );
        assert_eq!(got[1].2, [3.0; 4], "its slices are the classes'");
    }

    /// Colour filters compose into one matrix per element: its own
    /// functions in order, then its ancestors' over them, as CSS filters
    /// nested groups; a sibling outside stays unfiltered, and `blur-*`
    /// adds no matrix. Ways this could fail: the parent's matrix applied
    /// first (the wrong order for anything but scales), the child's
    /// replacing it, or the matrix not reaching the child's quads.
    #[test]
    fn colour_filters_compose_down_the_subtree() {
        let mut t = real();
        nodes(
            &mut t,
            &[
                (1, 0, "w-[10px] h-[10px] bg-white brightness-50", 0),
                (2, 0, "w-[10px] h-[10px] bg-white invert", 1),
                (3, 0, "w-[10px] h-[10px] bg-white", 0),
                (4, 0, "w-[10px] h-[10px] bg-white blur-sm", 0),
            ],
        );
        frame(&mut t);
        let apply = |m: &og_draw::paint::ColorMatrix, c: [f32; 4]| -> [f32; 4] {
            std::array::from_fn(|i| (0..4).map(|k| m[i][k] * c[k]).sum::<f32>() + m[4][i])
        };
        let filters: Vec<u32> = t
            .drawn
            .quads
            .iter()
            .filter(|q| q.kind == PaintKind::Solid as u32)
            .map(|q| q.filter)
            .collect();
        assert_eq!(filters.len(), 4);
        assert!(filters[0] != 0 && filters[1] != 0 && filters[0] != filters[1]);
        assert_eq!(&filters[2..], [0, 0], "unfiltered, and blur is no matrix");
        let child = &t.drawn.filters[filters[1] as usize - 1];
        // invert first, then halved: 0.2 → 0.8 → 0.4 (the other order
        // gives 0.9).
        let got = apply(child, [0.2, 0.2, 0.2, 1.0]);
        assert!((got[0] - 0.4).abs() < 1e-5 && got[3] == 1.0, "{got:?}");
        let parent = &t.drawn.filters[filters[0] as usize - 1];
        assert!((apply(parent, [0.8, 0.0, 0.0, 1.0])[0] - 0.4).abs() < 1e-5);
    }

    /// `object-*` fits an image's source of known size into its box as
    /// CSS does: `contain` letterboxes it, `cover` crops it about the
    /// centre (`object-top` from the top), `none` shows it at its size,
    /// and no `object-*` stretches it. Ways this could fail: the crop's
    /// texture region off its drawn box, or the position ignored.
    #[test]
    fn object_fit_places_and_crops_an_images_source() {
        let mut t = real();
        let classes = [
            "w-[100px] h-[50px]",
            "w-[100px] h-[50px] object-contain",
            "w-[100px] h-[50px] object-cover",
            "w-[100px] h-[50px] object-cover object-top",
            "w-[100px] h-[50px] object-none object-left-top",
        ];
        let mut b = Batch::default();
        for (k, c) in classes.iter().enumerate() {
            let id = k as u32 + 1;
            b.op(1).u32(id).u8(2);
            b.op(4).u32(id).str(&format!("{c} absolute top-0 left-0"));
            b.op(6).u32(id).str("src").str("7");
            b.op(8).u32(0).u32(id).u32(u32::MAX);
        }
        t.apply(&b.bytes()).unwrap();
        t.set_textures(Box::new(|_| (3, [200.0, 200.0])));
        frame(&mut t);
        let got: Vec<([f32; 4], [f32; 4])> = t
            .drawn
            .quads
            .iter()
            .filter(|q| q.kind == PaintKind::Image as u32)
            .map(|q| (q.rect, q.uv))
            .collect();
        let whole = [0.0, 0.0, 1.0, 1.0];
        assert_eq!(
            got,
            [
                ([0.0, 0.0, 100.0, 50.0], whole),
                ([25.0, 0.0, 50.0, 50.0], whole),
                ([0.0, 0.0, 100.0, 50.0], [0.0, 0.25, 1.0, 0.75]),
                ([0.0, 0.0, 100.0, 50.0], [0.0, 0.0, 1.0, 0.5]),
                ([0.0, 0.0, 100.0, 50.0], [0.0, 0.0, 0.5, 0.25]),
            ]
        );
    }

    /// An `aspect-[7/10]` flex item in an `items-center` row takes its
    /// height from its used width through the ratio, as CSS does, while
    /// content taller than that still grows it: a reward screen's card
    /// offers, which were their text's height. A column's own width is its
    /// items' contributions, not what their flexed heights give through
    /// their ratios: it stays 0 while its square items are 80 wide.
    #[test]
    fn a_flex_item_takes_its_cross_size_from_its_aspect_ratio() {
        use taffy::prelude::*;
        let mut t = tree();
        t.set_resolver(Box::new(|class, _, _, _| {
            let mut s = taffy::Style::default();
            match class {
                "row" => {
                    s.flex_direction = FlexDirection::Row;
                    s.align_items = AlignItems::CENTER;
                    s.size.width = length(600.0);
                }
                "card" => {
                    s.flex_direction = FlexDirection::Row;
                    s.aspect_ratio = Some(0.7);
                }
                "wide" => s.size = Size::from_lengths(210.0, 80.0),
                "tall" => s.size = Size::from_lengths(50.0, 100.0),
                "box" => {
                    s.flex_direction = FlexDirection::Row;
                    s.size = Size::from_lengths(400.0, 168.0);
                }
                "col" => {
                    s.flex_direction = FlexDirection::Column;
                    s.align_items = AlignItems::CENTER;
                    s.size.height = percent(1.0);
                    s.gap.height = length(8.0);
                }
                "sq" => {
                    s.flex_grow = 1.0;
                    s.flex_basis = length(0.0);
                    s.min_size.height = length(0.0);
                    s.max_size.width = length(100.0);
                    s.aspect_ratio = Some(1.0);
                }
                _ => {}
            }
            Record {
                layout: rune_taffy::CompactStyle::from(&s),
                ..Record::default()
            }
        }));
        let mut b = Batch::default();
        for (id, class, parent) in [
            (1, "row", 0),
            (2, "card", 1),
            (3, "wide", 2),
            (4, "card", 1),
            (5, "tall", 4),
        ] {
            b.op(1).u32(id).u8(0);
            b.op(4).u32(id).str(class);
            b.op(8).u32(parent).u32(id).u32(u32::MAX);
        }
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let wide = t.op_box(2).unwrap();
        assert_eq!([wide[2], wide[3]], [210.0, 300.0], "210 wide at 7:10");
        let tall = t.op_box(4).unwrap();
        assert_eq!([tall[2], tall[3]], [50.0, 100.0], "its content, past 7:10");

        let mut b = Batch::default();
        for (id, class, parent) in [(6, "box", 0), (7, "col", 6), (8, "sq", 7), (9, "sq", 7)] {
            b.op(1).u32(id).u8(0);
            b.op(4).u32(id).str(class);
            b.op(8).u32(parent).u32(id).u32(u32::MAX);
        }
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let col = t.op_box(7).unwrap();
        assert_eq!(col[2], 0.0, "the column's width ignores its items' ratios");
        let sq = t.op_box(8).unwrap();
        assert_eq!(
            [sq[2], sq[3]],
            [80.0, 80.0],
            "a square at its flexed height"
        );
    }

    /// A row sized to its content (a `justify-items-center` grid item)
    /// counts a `grow basis-0` item at its max-content width, as CSS does,
    /// not at its zero basis: the title screen's prompt between two rules.
    #[test]
    fn a_content_sized_row_gives_a_zero_basis_item_its_content() {
        use taffy::prelude::*;
        let mut t = tree();
        t.set_resolver(Box::new(|class, _, _, _| {
            let mut s = taffy::Style::default();
            match class {
                "grid" => {
                    s.display = Display::Grid;
                    s.justify_items = AlignItems::CENTER;
                    s.size.width = length(400.0);
                }
                "row" => s.flex_direction = FlexDirection::Row,
                "rule" => {
                    (s.flex_grow, s.flex_shrink, s.flex_basis) = (0.0, 0.0, length(20.0));
                    s.size.height = length(2.0);
                }
                "prompt" => (s.flex_grow, s.flex_shrink, s.flex_basis) = (1.0, 1.0, length(0.0)),
                "text" => s.size = Size::from_lengths(50.0, 10.0),
                _ => {}
            }
            Record {
                layout: rune_taffy::CompactStyle::from(&s),
                ..Record::default()
            }
        }));
        let mut b = Batch::default();
        for (id, class, parent) in [
            (1, "grid", 0),
            (2, "row", 1),
            (3, "rule", 2),
            (4, "prompt", 2),
            (5, "text", 4),
            (6, "rule", 2),
        ] {
            b.op(1).u32(id).u8(0);
            b.op(4).u32(id).str(class);
            b.op(8).u32(parent).u32(id).u32(u32::MAX);
        }
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        assert_eq!(
            t.op_box(2).unwrap()[2],
            90.0,
            "two rules and the prompt's content"
        );
        assert_eq!(
            t.op_box(4).unwrap()[2],
            50.0,
            "the prompt is as wide as its content"
        );
        assert_eq!(t.op_box(2).unwrap()[0], 155.0, "the row is centred");
    }

    /// A node's `data-*` props and its nearest groups' reach its classes'
    /// resolver as its data context: its own by name, a group's as
    /// `group/name` from the nearest group of that name only, and a prop
    /// turned off or a group's changing restyles the nodes that read it.
    #[test]
    fn data_props_reach_the_resolver_with_the_nearest_groups() {
        let mut t = tree();
        t.set_resolver(Box::new(|_, _, _, data| {
            let mut r = Record::default();
            r.skin.opacity = 1.0 - data.split(' ').filter(|d| !d.is_empty()).count() as f32 / 10.0;
            r.font_size = if data.contains("b/armed") { 30.0 } else { 16.0 };
            r
        }));
        let mut b = Batch::default();
        // 1 (group/b) > 2 > 3 (group/b) > 4
        for (id, class) in [(1, "group/b"), (2, "x"), (3, "group/b"), (4, "y")] {
            b.op(1).u32(id).u8(0);
            b.op(4).u32(id).str(class);
        }
        b.op(8).u32(0).u32(1).u32(u32::MAX);
        b.op(8).u32(1).u32(2).u32(u32::MAX);
        b.op(8).u32(2).u32(3).u32(u32::MAX);
        b.op(8).u32(3).u32(4).u32(u32::MAX);
        b.op(6).u32(1).str("data-armed").str("true");
        b.op(6).u32(2).str("data-on").str("true");
        b.op(6).u32(2).str("data-off").str("false");
        t.apply(&b.bytes()).unwrap();
        let rec = |t: &Tree, id: usize| {
            let r = &t.els[t.by_js[id] as usize].record;
            (r.skin.opacity, r.font_size)
        };
        assert_eq!(rec(&t, 2), (0.8, 30.0), "on, and b/armed from 1");
        assert_eq!(rec(&t, 4), (1.0, 16.0), "3 is its nearest b, unarmed");
        assert_eq!(rec(&t, 1), (0.9, 16.0), "its own armed, no group above it");
        let mut b = Batch::default();
        b.op(6).u32(1).str("data-armed").str("false");
        t.apply(&b.bytes()).unwrap();
        assert_eq!(rec(&t, 2), (0.9, 16.0), "the group's change restyles it");
        assert_eq!(rec(&t, 1), (1.0, 16.0));
    }

    #[test]
    fn a_broken_batch_is_refused() {
        let mut t = tree();
        let mut b = Batch::default();
        b.op(3).u32(9).str("x");
        assert!(t.apply(&b.bytes()).unwrap_err().contains("no node 9"));
        let mut b = Batch::default();
        b.op(1).u32(0).u8(0);
        assert!(t.apply(&b.bytes()).is_err(), "0 is the root's");
        assert!(t.apply(&[1, 0, 0, 0, 2, 7]).is_err(), "cut short");
    }

    /// A tree whose class `fade` is a red 50 by 20 box that fades over
    /// 0.2 s, linearly, in from and out to transparent; `slow` fades
    /// over 0.2 s with no `starting:` or `exit:` record; `dim` is half
    /// opaque.
    fn fading() -> Tree {
        let mut t = tree();
        t.set_resolver(Box::new(|class, _, state, _| {
            let mut s = taffy::Style::default();
            if class != "row" {
                s.size = taffy::Size::from_lengths(50.0, 20.0);
            }
            let mut r = Record {
                layout: rune_taffy::CompactStyle::from(&s),
                ..Record::default()
            };
            r.skin.fill = [1.0, 0.0, 0.0, 1.0];
            if class.contains("fade") {
                r.motion = crate::motion::Spec {
                    props: crate::motion::OPACITY,
                    duration: 0.2,
                    ease: crate::motion::Ease::Bezier([0.0, 0.0, 1.0, 1.0]),
                    ..Default::default()
                };
                if state & (STARTING | EXIT) != 0 {
                    r.skin.opacity = 0.0;
                }
            }
            if class.contains("slow") {
                r.motion = crate::motion::Spec {
                    props: crate::motion::OPACITY,
                    duration: 0.2,
                    ease: crate::motion::Ease::Bezier([0.0, 0.0, 1.0, 1.0]),
                    ..Default::default()
                };
            }
            if class.contains("dim") && !(class.contains("fade") && state & (STARTING | EXIT) != 0)
            {
                r.skin.opacity = 0.5;
            }
            r
        }));
        t
    }

    /// The alpha of the solid quad at `x`, if one is drawn there.
    fn alpha_at(t: &Tree, x: f32) -> Option<f32> {
        t.drawn
            .quads
            .iter()
            .find(|q| q.kind == PaintKind::Solid as u32 && q.rect[0] == x)
            .map(|q| q.color[3])
    }

    /// The first build appears as it is; a node inserted later fades in
    /// from its `starting:` record on the tick's time; a removed one stays
    /// drawn and unhit while it fades to its `exit:` record, then leaves;
    /// what was released inside it goes when it does.
    #[test]
    fn an_op_node_enters_from_starting_and_stays_drawn_until_its_exit_settles() {
        let mut t = fading();
        let mut b = Batch::default();
        b.op(1).u32(1).u8(0);
        b.op(4).u32(1).str("row");
        b.op(1).u32(2).u8(0);
        b.op(4).u32(2).str("fade");
        b.op(8).u32(0).u32(1).u32(u32::MAX);
        b.op(8).u32(1).u32(2).u32(u32::MAX);
        t.set_time(1.0, false);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        assert_eq!(
            alpha_at(&t, 0.0),
            Some(1.0),
            "the first build does not fade in"
        );
        // A second node, inserted into a row on screen, fades in.
        let mut b = Batch::default();
        b.op(1).u32(3).u8(0);
        b.op(4).u32(3).str("fade");
        b.op(7).u32(3).u8(Event::Press as u8).u8(1);
        b.op(1).u32(4).u8(0); // a child of 3
        b.op(8).u32(3).u32(4).u32(u32::MAX);
        b.op(8).u32(1).u32(3).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        t.set_time(1.05, false);
        frame(&mut t);
        let a = alpha_at(&t, 50.0).unwrap();
        assert!((a - 0.25).abs() < 0.01, "a quarter in after 0.05 s: {a}");
        t.set_time(1.3, false);
        frame(&mut t);
        assert_eq!(alpha_at(&t, 50.0), Some(1.0));
        let three = t.els[t.op(3).unwrap() as usize].node as u64;
        let mut client = a11y::Tree::new();
        let kids = |u: &accesskit::TreeUpdate| {
            let root = &u.nodes.iter().find(|n| n.0.0 == 0).unwrap().1;
            root.children().iter().map(|c| c.0).collect::<Vec<_>>()
        };
        assert_eq!(kids(&t.a11y(&mut client, None, 1.0).unwrap()), [three]);
        // Removed: it fades out where it was and takes no press.
        let mut b = Batch::default();
        b.op(9).u32(1).u32(3);
        b.op(10).u32(4); // released inside it: kept until it goes
        t.apply(&b.bytes()).unwrap();
        t.set_time(1.4, false);
        frame(&mut t);
        let u = t.a11y(&mut client, None, 1.0).unwrap();
        assert!(
            kids(&u).is_empty(),
            "a reader no longer finds what is on its way out"
        );
        let a = alpha_at(&t, 50.0).unwrap();
        assert!((a - 0.5).abs() < 0.01, "half out after 0.1 s: {a}");
        assert!(t.op(4).is_err(), "a released id is free at once");
        t.pointer(75.0, 10.0, true, false);
        t.pointer(75.0, 10.0, false, true);
        assert!(t.events.is_empty(), "an exiting node answers nothing");
        t.set_time(1.7, false);
        frame(&mut t);
        assert_eq!(alpha_at(&t, 50.0), None, "out once its exit settled");
        assert_eq!(t.op_box(3), None);
        assert!(t.leaving.is_empty() && t.els.iter().filter(|e| e.js == 4).all(|e| e.dead));
        // Nothing moves: the next frame paints nothing.
        t.set_time(1.8, false);
        let (_, painted) = present(&mut t);
        assert!(!painted);
    }

    /// A subtree built off screen and then put on it (a screen in the
    /// `<Show>` branch or the mount that brings it) enters whole: a node
    /// under a new wrapper fades in from its `starting:` record too, as
    /// CSS's `@starting-style` applies to every element first drawn. Taken
    /// off again by its wrapper, which has no `exit:`, it fades out to its
    /// own `exit:` record before the wrapper goes.
    #[test]
    fn a_node_under_a_wrapper_enters_and_leaves_with_it() {
        let mut t = fading();
        let mut b = Batch::default();
        b.op(1).u32(1).u8(0);
        b.op(4).u32(1).str("row");
        b.op(8).u32(0).u32(1).u32(u32::MAX);
        t.set_time(1.0, false);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        // The wrapper 2 gets its child 3 before it goes on screen.
        let mut b = Batch::default();
        b.op(1).u32(2).u8(0);
        b.op(4).u32(2).str("row");
        b.op(1).u32(3).u8(0);
        b.op(4).u32(3).str("fade");
        b.op(8).u32(2).u32(3).u32(u32::MAX);
        b.op(8).u32(1).u32(2).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        t.set_time(1.05, false);
        frame(&mut t);
        // The child paints over its wrapper: the last solid quad at x 0.
        let a = t
            .drawn
            .quads
            .iter()
            .rev()
            .find(|q| q.kind == PaintKind::Solid as u32 && q.rect[0] == 0.0)
            .map(|q| q.color[3])
            .unwrap();
        assert!((a - 0.25).abs() < 0.01, "a quarter in after 0.05 s: {a}");
        t.set_time(1.5, false);
        frame(&mut t);
        // The wrapper, with no `exit:` of its own, is removed: the node
        // under it fades out to its `exit:` record and both stay drawn
        // until it settles, then go.
        let mut b = Batch::default();
        b.op(9).u32(0).u32(1);
        b.op(10).u32(1); // and released, as an unmount does
        t.apply(&b.bytes()).unwrap();
        t.set_time(1.6, false);
        frame(&mut t);
        let child = |t: &Tree| {
            t.drawn
                .quads
                .iter()
                .rev()
                .find(|q| {
                    q.kind == PaintKind::Solid as u32 && q.rect[0] == 0.0 && q.rect[2] == 50.0
                })
                .map(|q| q.color[3])
        };
        let a = child(&t).unwrap();
        assert!((a - 0.5).abs() < 0.01, "half out after 0.1 s: {a}");
        t.set_time(1.8, false);
        frame(&mut t);
        assert_eq!(child(&t), None, "out once its exit settled");
        assert!(t.leaving.is_empty());
    }

    /// A node classed before it is first drawn, while detached or after
    /// its insert, takes that class as its first style, as CSS does: it shows at once where nothing enters it, and enters from
    /// that class's `starting:` record into a parent on screen, never
    /// transitioning from the bare record. Once drawn, a class change
    /// transitions.
    #[test]
    fn a_class_set_before_the_first_draw_is_the_first_style() {
        let mut t = fading();
        // The 50 px box at `x` (the row's own quad starts at 0 too).
        let alpha_at = |t: &Tree, x: f32| {
            t.drawn
                .quads
                .iter()
                .find(|q| q.kind == PaintKind::Solid as u32 && q.rect[0] == x && q.rect[2] == 50.0)
                .map(|q| q.color[3])
        };
        let mut b = Batch::default();
        b.op(1).u32(1).u8(0);
        b.op(4).u32(1).str("row");
        b.op(8).u32(0).u32(1).u32(u32::MAX);
        t.set_time(1.0, false);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        // Into the row on screen, classed after the insert.
        let mut b = Batch::default();
        b.op(1).u32(2).u8(0);
        b.op(8).u32(1).u32(2).u32(u32::MAX);
        b.op(4).u32(2).str("fade dim");
        b.op(1).u32(3).u8(0);
        b.op(8).u32(1).u32(3).u32(u32::MAX);
        b.op(4).u32(3).str("fade");
        t.apply(&b.bytes()).unwrap();
        t.set_time(1.05, false);
        frame(&mut t);
        let dim = alpha_at(&t, 0.0).unwrap();
        assert!(
            (dim - 0.125).abs() < 0.01,
            "enters from starting: to its class's 0.5: {dim}"
        );
        let a = alpha_at(&t, 50.0).unwrap();
        assert!(
            (a - 0.25).abs() < 0.01,
            "enters from starting:, a quarter in: {a}"
        );
        t.set_time(1.3, false);
        frame(&mut t);
        assert_eq!(alpha_at(&t, 0.0), Some(0.5));
        // Drawn now: dimming the second transitions.
        let mut b = Batch::default();
        b.op(4).u32(3).str("fade dim");
        t.apply(&b.bytes()).unwrap();
        t.set_time(1.4, false);
        frame(&mut t);
        let a = alpha_at(&t, 50.0).unwrap();
        assert!((a - 0.75).abs() < 0.01, "half way from 1 to 0.5: {a}");
        // A node that transitions but has no `starting:` record, classed
        // after its insert, shows its class at once.
        let mut b = Batch::default();
        b.op(1).u32(4).u8(0);
        b.op(8).u32(1).u32(4).u32(u32::MAX);
        b.op(4).u32(4).str("slow dim");
        t.apply(&b.bytes()).unwrap();
        t.set_time(1.45, false);
        frame(&mut t);
        assert_eq!(alpha_at(&t, 100.0), Some(0.5));
        // The TS renderer's usual order: classed while detached, then
        // inserted (a second page mounted into the same tree).
        let mut b = Batch::default();
        b.op(1).u32(5).u8(0);
        b.op(4).u32(5).str("slow dim");
        b.op(8).u32(1).u32(5).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        t.set_time(1.5, false);
        frame(&mut t);
        assert_eq!(alpha_at(&t, 150.0), Some(0.5), "classed before its insert");
    }

    /// The ops' tags, a11y props and listeners reach the accessibility
    /// tree: a pressed view is a button named by its text, at its box; a
    /// A node under a transform (a dialog scaling in) is where it is
    /// drawn in the accessibility tree and the probe, not at its layout
    /// box, after the host took the paint.
    #[test]
    fn a_transformed_node_is_where_it_is_drawn() {
        let mut t = tree();
        let mut b = Batch::default();
        b.op(1).u32(1).u8(0);
        b.op(4).u32(1).str("b half");
        b.op(6).u32(1).str("role").str("button");
        b.op(8).u32(0).u32(1).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        // A host takes the paint it presents; where things were drawn stays.
        drop(std::mem::take(&mut t.drawn));
        let ops = t.dump_ops();
        let n = ops.iter().find(|n| n.id == 1).unwrap();
        assert_eq!(n.transform, Some([0.5, 0.0, 0.0, 0.5, 12.5, 5.0]));
        let node = t.els[t.op(1).unwrap() as usize].node as u64;
        let mut client = a11y::Tree::new();
        let u = t.a11y(&mut client, None, 1.0).unwrap();
        let r = u
            .nodes
            .iter()
            .find(|n| n.0.0 == node)
            .unwrap()
            .1
            .bounds()
            .unwrap();
        assert_eq!((r.x0, r.y0, r.x1, r.y1), (12.5, 5.0, 37.5, 15.0));
    }

    /// The probe lists the part of a node its clipping ancestors let show:
    /// a row below a 20 px box that cuts its overflow shows none of it.
    #[test]
    fn the_probe_lists_what_a_cut_box_lets_show() {
        let mut t = tree();
        let mut b = Batch::default();
        b.op(1).u32(1).u8(0);
        b.op(4).u32(1).str("b clip col");
        for k in 2..4 {
            b.op(1).u32(k).u8(0);
            b.op(4).u32(k).str("b fixed");
            b.op(8).u32(1).u32(k).u32(u32::MAX);
        }
        b.op(8).u32(0).u32(1).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let ops = t.dump_ops();
        let at = |id: u32| ops.iter().find(|n| n.id == id).unwrap().visible;
        assert_eq!(at(2), [0.0, 0.0, 50.0, 20.0]);
        let [_, y0, _, y1] = at(3);
        assert_eq!(y0, y1, "the second row is cut away whole");
        // The surface (400 px wide) cuts nothing: what runs past the
        // screen is listed whole, so the safe-area check sees it.
        let mut b = Batch::default();
        b.op(1).u32(8).u8(0);
        b.op(4).u32(8).str("row");
        b.op(1).u32(9).u8(0);
        b.op(4).u32(9).str("wide fixed");
        b.op(8).u32(0).u32(8).u32(u32::MAX);
        b.op(8).u32(8).u32(9).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let n = t.dump_ops().into_iter().find(|n| n.id == 9).unwrap();
        let [x, y, w, h] = n.rect;
        assert_eq!((w, n.visible), (600.0, [x, y, x + w, y + h]));
    }

    /// A moved node is cut where it is drawn, as the probe lists it: one
    /// moved out of its clipping box shows none of it (the compass's
    /// letters off its strip), one moved partly out shows the rest, cut.
    #[test]
    fn the_probe_cuts_a_moved_node_where_it_is_drawn() {
        let mut t = tree();
        let mut b = Batch::default();
        b.op(1).u32(1).u8(0);
        b.op(4).u32(1).str("big col");
        for (clip, moved, class) in [(2, 3, "b up"), (4, 5, "b left")] {
            b.op(1).u32(clip).u8(0);
            b.op(4).u32(clip).str("b clip fixed");
            b.op(1).u32(moved).u8(0);
            b.op(4).u32(moved).str(class);
            b.op(8).u32(1).u32(clip).u32(u32::MAX);
            b.op(8).u32(clip).u32(moved).u32(u32::MAX);
        }
        b.op(8).u32(0).u32(1).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let ops = t.dump_ops();
        let at = |id: u32| ops.iter().find(|n| n.id == id).unwrap();
        let [_, y0, _, y1] = at(3).visible;
        assert_eq!(y0, y1, "drawn above its box, cut away whole");
        let n = at(5);
        assert_eq!(n.visible, [0.0, 20.0, 20.0, 40.0]);
        assert_eq!((n.clip, n.clipped), (Some([0.0, 20.0, 50.0, 40.0]), true));
        assert_eq!(
            (at(2).clip, at(2).clipped),
            (None, false),
            "nothing cuts the boxes"
        );
    }

    /// What a box under a transform clips is cut where that box is drawn,
    /// not where it is laid out: a card lifted 40 px keeps its face.
    #[test]
    fn a_transformed_clip_moves_with_its_box() {
        let mut t = tree();
        let mut b = Batch::default();
        b.op(1).u32(1).u8(0);
        b.op(4).u32(1).str("big col");
        // Three 20 px rows above it: laid out at y 60, drawn at 20.
        for k in 5..8 {
            b.op(1).u32(k).u8(0);
            b.op(4).u32(k).str("b");
            b.op(8).u32(1).u32(k).u32(u32::MAX);
        }
        b.op(1).u32(2).u8(0);
        b.op(4).u32(2).str("b up");
        b.op(1).u32(3).u8(0);
        b.op(4).u32(3).str("b clip");
        b.op(1).u32(4).u8(0);
        b.op(4).u32(4).str("b ink");
        b.op(8).u32(0).u32(1).u32(u32::MAX);
        b.op(8).u32(1).u32(2).u32(u32::MAX);
        b.op(8).u32(2).u32(3).u32(u32::MAX);
        b.op(8).u32(3).u32(4).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let q = t.drawn.quads.iter().find(|q| q.color == [1.0; 4]).unwrap();
        assert_eq!(q.clip, [0.0, 20.0, 50.0, 40.0]);
    }

    /// prop's text sets a role; a release takes the node out.
    #[test]
    fn ops_reach_the_accessibility_tree() {
        let mut t = tree();
        let mut b = Batch::default();
        b.op(1).u32(1).u8(0); // view 1, a row
        b.op(4).u32(1).str("row");
        b.op(1).u32(2).u8(0); // view 2, pressed
        b.op(4).u32(2).str("b");
        b.op(7).u32(2).u8(Event::Press as u8).u8(1);
        b.op(2).u32(3).str("Play");
        b.op(1).u32(4).u8(0); // view 4, a status
        b.op(6).u32(4).str("role").str("status");
        b.op(6).u32(4).str("label").str("\"Wave 3\"");
        b.op(11).u32(5);
        b.op(8).u32(0).u32(1).u32(u32::MAX);
        b.op(8).u32(1).u32(2).u32(u32::MAX);
        b.op(8).u32(2).u32(3).u32(u32::MAX);
        b.op(8).u32(1).u32(4).u32(u32::MAX);
        b.op(8).u32(1).u32(5).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        let mut client = a11y::Tree::new();
        // Before the layout gives the nodes their boxes (a page's first
        // frame answer), nothing is sent; the frame after it sends it all.
        assert!(t.a11y(&mut client, None, 1.0).is_none());
        frame(&mut t);
        let node = |e: u32| t.els[t.op(e).unwrap() as usize].node as u64;
        let (two, four) = (node(2), node(4));
        let u = t.a11y(&mut client, None, 1.0).unwrap();
        let get = |u: &accesskit::TreeUpdate, id: u64| {
            u.nodes.iter().find(|n| n.0.0 == id).map(|n| n.1.clone())
        };
        let root = get(&u, 0).unwrap();
        assert_eq!(
            root.children().iter().map(|c| c.0).collect::<Vec<_>>(),
            [two, four],
            "the row and the placeholder are no nodes, the text is the button's"
        );
        let button = get(&u, two).unwrap();
        assert_eq!(
            (button.role(), button.label()),
            (accesskit::Role::Button, Some("Play"))
        );
        let r = button.bounds().unwrap();
        assert_eq!((r.x0, r.y0, r.x1, r.y1), (0.0, 0.0, 50.0, 20.0));
        let status = get(&u, four).unwrap();
        assert_eq!(
            (status.role(), status.label()),
            (accesskit::Role::Status, Some("Wave 3"))
        );
        // As the web mirror takes it.
        let web = a11y::web(&u);
        assert_eq!(web["full"], true);
        let path = two.to_string();
        let b = web["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["path"] == path.as_str())
            .unwrap();
        assert_eq!(
            (&b["role"], &b["name"], &b["states"], &b["width"]),
            (
                &"button".into(),
                &"Play".into(),
                &serde_json::json!(["focusable"]),
                &50.0.into()
            )
        );
        let s = web["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["path"] == four.to_string().as_str())
            .unwrap();
        assert_eq!(
            (&s["role"], &s["live"], &s["atomic"]),
            (&"status".into(), &"polite".into(), &true.into())
        );
        assert!(t.a11y(&mut client, None, 1.0).is_none(), "nothing changed");
        // The probe's view: every op node that shows, by op id, with its
        // op parent, box and text.
        let ops = t.dump_ops();
        let at = |id: u32| ops.iter().find(|n| n.id == id).unwrap();
        assert_eq!((at(2).parent, at(3).parent, at(1).parent), (1, 2, 0));
        assert_eq!(
            (at(2).node as u64, at(2).rect, at(3).text),
            (two, [0.0, 0.0, 50.0, 20.0], "Play")
        );
        assert!(!at(5).shown, "a placeholder takes no room");
        let (facts, order) = t.a11y_snapshot();
        let f = &facts[&(two as Id)];
        assert_eq!((f.role, f.name.as_str()), ("button", "Play"));
        assert!(f.states.contains(&"focusable") && !f.states.contains(&"focused"));
        assert_eq!(order, [two as Id], "the status is not focusable");
        assert_eq!(facts[&(four as Id)].role, "status");
        let mut b = Batch::default();
        b.op(10).u32(2);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let u = t.a11y(&mut client, None, 1.0).unwrap();
        let root = get(&u, 0).unwrap();
        assert_eq!(
            root.children().iter().map(|c| c.0).collect::<Vec<_>>(),
            [four]
        );
    }

    /// A tree whose classes read input states: `b` is 50 by 20, and
    /// `hover:` makes `hw` 80 wide, `active:` makes `aw` 60, and
    /// `focus-visible:` makes `fw` 90.
    fn stated() -> Tree {
        let mut t = tree();
        t.set_resolver(Box::new(|class, _, state, _| {
            let mut s = taffy::Style::default();
            for c in class.split_whitespace() {
                let w = match c {
                    "row" => {
                        s.flex_direction = taffy::FlexDirection::Row;
                        continue;
                    }
                    "col" => {
                        s.flex_direction = taffy::FlexDirection::Column;
                        continue;
                    }
                    "b" => 50.0,
                    "hw" if state & HOVERED != 0 => 80.0,
                    "aw" if state & PRESSED != 0 => 60.0,
                    "fw" if state & FOCUS_VISIBLE != 0 => 90.0,
                    _ => continue,
                };
                s.size = taffy::Size::from_lengths(w, 20.0);
            }
            Record {
                layout: rune_taffy::CompactStyle::from(&s),
                ..Record::default()
            }
        }));
        t
    }

    /// A node under `scheme-dark` or `scheme-light` naming the other scheme
    /// than the page's resolves with [`crate::classes::OTHER_SCHEME`], and
    /// so does its subtree down to a nearer `scheme-*`; a sibling does not.
    /// A class change or a page scheme change moves it. Ways this could
    /// fail: only the node naming it flips (its labels keep the page's
    /// ink), the nearest names are not the ones read, or a child keeps its
    /// old scheme after its parent's class changes.
    #[test]
    fn a_scheme_class_resolves_its_subtree_in_that_scheme() {
        let resolver = |page_dark: bool| -> Resolve {
            Box::new(move |class, _, state, _| {
                let mut r = Record {
                    dark: page_dark != (state & crate::classes::OTHER_SCHEME != 0),
                    ..Record::default()
                };
                for c in class.split_whitespace() {
                    match c {
                        "scheme-dark" => r.scheme = Some(true),
                        "scheme-light" => r.scheme = Some(false),
                        _ => {}
                    }
                }
                r
            })
        };
        let mut t = tree();
        t.set_resolver(resolver(false));
        let mut b = Batch::default();
        for (id, class, parent) in [
            (1, "scheme-dark", 0),
            (2, "", 1),
            (3, "scheme-light", 2),
            (4, "", 3),
            (5, "", 0),
        ] {
            b.op(1).u32(id).u8(0);
            b.op(4).u32(id).str(class);
            b.op(8).u32(parent).u32(id).u32(u32::MAX);
        }
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let dark = |t: &Tree| -> Vec<bool> {
            (1..=5)
                .map(|id| t.els[t.by_js[id] as usize].record.dark)
                .collect()
        };
        assert_eq!(dark(&t), [true, true, false, false, false], "light page");
        let mut b = Batch::default();
        b.op(4).u32(1).str("");
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        assert_eq!(dark(&t), [false; 5], "its parent's class went");
        t.set_resolver(resolver(true));
        frame(&mut t);
        assert_eq!(dark(&t), [true, true, false, false, true], "dark page");
    }

    /// `pointer-events-none` lets a press through it and the subtree that
    /// sets nothing to what is under it, and `pointer-events-auto` inside
    /// takes it back; `invisible` hides its subtree from presses too.
    /// Ways this could fail: none not inherited (its child still takes the
    /// press), auto not overriding it, or an invisible node still hit.
    #[test]
    fn pointer_events_and_invisible_decide_what_a_press_reaches() {
        let mut t = tree();
        t.set_resolver(Box::new(|class, _, _, _| {
            let s = taffy::Style {
                size: taffy::Size::from_lengths(50.0, 20.0),
                ..Default::default()
            };
            let mut r = Record {
                layout: rune_taffy::CompactStyle::from(&s),
                ..Record::default()
            };
            for c in class.split_whitespace() {
                match c {
                    "pn" => r.touch = Some(false),
                    "pa" => r.touch = Some(true),
                    "inv" => r.hidden = true,
                    _ => {}
                }
            }
            r
        }));
        let mut b = Batch::default();
        for (id, parent) in [(1, 0), (2, 1), (3, 2)] {
            b.op(1).u32(id).u8(0);
            b.op(4).u32(id).str("");
            b.op(7).u32(id).u8(Event::Press as u8).u8(1);
            b.op(8).u32(parent).u32(id).u32(u32::MAX);
        }
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let press = |t: &mut Tree, classes: [&str; 2]| -> Vec<u32> {
            let mut b = Batch::default();
            for (id, c) in [2, 3].into_iter().zip(classes) {
                b.op(4).u32(id).str(c);
            }
            t.apply(&b.bytes()).unwrap();
            frame(t);
            t.events.clear();
            for (pressed, released) in [(false, false), (true, false), (false, true)] {
                t.pointer(10.0, 10.0, pressed, released);
                frame(t);
            }
            let ids = t.events.iter().filter(|f| f.event == Event::Press);
            ids.map(|f| f.id).collect()
        };
        assert_eq!(press(&mut t, ["", ""]), [3], "the topmost");
        assert_eq!(press(&mut t, ["pn", ""]), [1], "through 2 and 3");
        assert_eq!(press(&mut t, ["pn", "pa"]), [3], "3 takes it back");
        assert_eq!(press(&mut t, ["inv", "pa"]), [1], "2 and 3 hidden");
    }

    /// Ids 1..=n as `class` views under a row at the root, each listening
    /// to `press`, `focus` and `blur`.
    fn buttons(t: &mut Tree, classes: &[&str]) {
        let mut b = Batch::default();
        b.op(1).u32(100).u8(0);
        b.op(4).u32(100).str("row");
        b.op(8).u32(0).u32(100).u32(u32::MAX);
        for (i, c) in classes.iter().enumerate() {
            let id = i as u32 + 1;
            b.op(1).u32(id).u8(0);
            b.op(4).u32(id).str(c);
            for ev in [Event::Press, Event::Focus, Event::Blur] {
                b.op(7).u32(id).u8(ev as u8).u8(1);
            }
            b.op(8).u32(100).u32(id).u32(u32::MAX);
        }
        t.apply(&b.bytes()).unwrap();
        frame(t);
    }

    fn at(t: &mut Tree, x: f32, y: f32, pressed: bool, released: bool) {
        t.pointer(x, y, pressed, released);
        frame(t);
    }

    /// `hover:` follows the pointer on and off; `active:` holds while the
    /// press is on its node or within NSH's press retention (8 px) around
    /// it, where a release still clicks; further off, a release does not.
    #[test]
    fn hover_and_active_follow_the_pointer_and_its_retention() {
        let mut t = stated();
        buttons(&mut t, &["b hw", "b aw"]);
        at(&mut t, 10.0, 10.0, false, false);
        assert_eq!(t.op_box(1).unwrap()[2], 80.0, "hovered");
        // A finger hovers nothing: what the mouse lit goes out, and a tap
        // leaves nothing lit.
        t.set_device(true, false);
        frame(&mut t);
        assert_eq!(t.op_box(1).unwrap()[2], 50.0, "a finger hovers nothing");
        at(&mut t, 10.0, 12.0, true, false);
        at(&mut t, 10.0, 12.0, false, true);
        assert_eq!(t.op_box(1).unwrap()[2], 50.0, "no hover left after a tap");
        t.events.clear();
        t.set_device(false, false);
        at(&mut t, 10.0, 200.0, false, false);
        assert_eq!(t.op_box(1).unwrap()[2], 50.0, "left");
        // 2 sits at x 50..100 now.
        at(&mut t, 60.0, 10.0, true, false);
        assert_eq!(t.op_box(2).unwrap()[2], 60.0, "pressed");
        // 25 px below its 20 px box is past the retention: not active.
        at(&mut t, 60.0, 45.0, false, false);
        assert_eq!(t.op_box(2).unwrap()[2], 50.0, "dragged off");
        at(&mut t, 60.0, 45.0, false, true);
        let pressed = t.events.iter().any(|f| f.event == Event::Press);
        assert!(!pressed, "a release off it clicks nothing");
        at(&mut t, 60.0, 10.0, true, false);
        t.events.clear();
        // 5 px below the box: inside the retention, a click.
        at(&mut t, 60.0, 25.0, false, true);
        let presses: Vec<_> = t
            .events
            .iter()
            .filter(|f| f.event == Event::Press)
            .collect();
        assert_eq!(presses, [&Fired::at(2, Event::Press, 60.0, 25.0)]);
        assert_eq!(t.op_box(2).unwrap()[2], 50.0, "released");
    }

    /// A pad's virtual cursor: the stick moves it from the real pointer's
    /// place onto a button and takes the stick from the focus; A presses
    /// the button through the pointer's path; the real pointer moving
    /// hides it again.
    #[test]
    fn the_virtual_cursor_moves_with_the_stick_and_presses_with_a() {
        use crate::tree::{CursorPad, CursorPointer, VirtualCursorDesc};
        let mut t = stated();
        buttons(&mut t, &["b", "b"]);
        let desc = Some(VirtualCursorDesc::default());
        let tick = |t: &mut Tree, stick: [f32; 2], a: (bool, bool), real: [f32; 2]| {
            let mut pad = CursorPad {
                stick,
                a_pressed: a.0,
                a_held: a.1,
                ..Default::default()
            };
            let mut ptr = CursorPointer {
                x: real[0],
                y: real[1],
                inside: true,
                ..Default::default()
            };
            t.virtual_cursor(desc, &mut pad, &mut ptr, 60);
            t.pointer(ptr.x, ptr.y, ptr.pressed, ptr.released);
            frame(t);
            pad
        };
        let real = [75.0, 200.0];
        tick(&mut t, [0.0; 2], (false, false), real);
        assert_eq!(
            t.virtual_cursor_at(),
            Some(([75.0, 200.0], false)),
            "at the pointer, hidden"
        );
        for _ in 0..60 {
            let pad = tick(&mut t, [0.0, -1.0], (false, false), real);
            assert_eq!(pad.stick, [0.0; 2], "the stick is the cursor's");
            if t.virtual_cursor_at().unwrap().0[1] < 15.0 {
                break;
            }
        }
        let ([x, y], shown) = t.virtual_cursor_at().unwrap();
        assert!(
            shown && (50.0..100.0).contains(&x) && y < 20.0,
            "on button 2: {x} {y}"
        );
        t.events.clear();
        let pad = tick(&mut t, [0.0; 2], (true, true), real);
        assert!(!pad.a_pressed, "A is the cursor's");
        tick(&mut t, [0.0; 2], (false, false), real);
        let presses: Vec<u32> = t
            .events
            .iter()
            .filter(|f| f.event == Event::Press)
            .map(|f| f.id)
            .collect();
        assert_eq!(presses, [2], "A clicks where it points");
        tick(&mut t, [0.0; 2], (false, false), [10.0, 10.0]);
        assert_eq!(
            t.virtual_cursor_at().map(|c| c.1),
            Some(false),
            "the mouse hides it"
        );
    }

    /// Tab goes to positive `tabIndex` first, then tree order, skipping a
    /// negative one, and wraps; Shift+Tab goes back. Focus moved by the
    /// keys shows `focus-visible:`, a press's does not; each move fires
    /// `blur` then `focus`.
    #[test]
    fn tab_order_focus_events_and_focus_visible() {
        let mut t = stated();
        buttons(&mut t, &["b fw", "b fw", "b fw", "b fw"]);
        let mut b = Batch::default();
        b.op(6).u32(3).str("tabIndex").str("2");
        b.op(6).u32(4).str("tabIndex").str("-1");
        t.apply(&b.bytes()).unwrap();
        let mut order = Vec::new();
        for _ in 0..4 {
            t.key("Tab", false, false, false, false);
            order.push(t.focused_id().unwrap());
        }
        assert_eq!(order, [3, 1, 2, 3]);
        t.key("Tab", true, false, false, false);
        assert_eq!(t.focused_id(), Some(2));
        frame(&mut t);
        assert_eq!(t.op_box(2).unwrap()[2], 90.0, "keys show focus");
        t.events.clear();
        // 1 sits at 0..50: a press there focuses it without the ring.
        at(&mut t, 10.0, 10.0, true, false);
        assert_eq!(t.focused_id(), Some(1));
        let fired: Vec<_> = t.events.iter().map(|f| (f.id, f.event)).collect();
        assert_eq!(fired, [(2, Event::Blur), (1, Event::Focus)]);
        assert_eq!(t.op_box(1).unwrap()[2], 50.0, "a press shows no ring");
        assert_eq!(t.op_box(2).unwrap()[2], 50.0, "blurred");
        // 4 is focused by a press though Tab passes it.
        at(&mut t, 10.0, 10.0, false, true);
        let x4 = t.op_box(4).unwrap()[0] + 5.0;
        at(&mut t, x4, 10.0, true, false);
        assert_eq!(t.focused_id(), Some(4));
    }

    /// On a touch screen a press just past a small control's box takes
    /// it, the nearest one where two grown boxes meet; a mouse takes only
    /// the box itself.
    #[test]
    fn a_finger_takes_the_nearest_small_control_within_the_touch_target() {
        let mut t = stated();
        // Two 50x20 buttons side by side at y 0..20.
        buttons(&mut t, &["b", "b"]);
        let pressed = |t: &mut Tree, x: f32, y: f32| {
            t.events.clear();
            at(t, x, y, true, false);
            at(t, x, y, false, true);
            t.events
                .iter()
                .filter(|f| f.event == Event::Press)
                .map(|f| f.id)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            pressed(&mut t, 10.0, 28.0),
            [] as [u32; 0],
            "a mouse: only the box"
        );
        t.set_device(true, false);
        assert_eq!(pressed(&mut t, 60.0, 10.0), [2], "the second, first");
        assert_eq!(pressed(&mut t, 10.0, 28.0), [1], "8 px under the first");
        assert_eq!(pressed(&mut t, 60.0, 30.0), [2], "under the second");
        assert_eq!(pressed(&mut t, 60.0, 10.0), [2], "the second itself");
        assert_eq!(pressed(&mut t, 10.0, 33.0), [] as [u32; 0], "past 44 px");
        assert_eq!(pressed(&mut t, 10.0, 10.0), [1], "the box itself");
    }

    /// A control whose classes say nothing of focus shows the pack's
    /// `Focus.ring` while keys focus it, none after a press, and none once
    /// it loses focus; one that styles `focus-visible:` itself keeps only
    /// its own look.
    #[test]
    fn keys_focus_shows_the_packs_ring_unless_the_control_styles_focus() {
        let mut t = tree();
        let mut classes = crate::classes::Classes::new(None).unwrap().0;
        classes.set_variants(
            [(
                "Focus.ring".to_string(),
                "outline-2 outline-[#ff0000] outline-offset-2".to_string(),
            )]
            .into(),
        );
        let classes = std::cell::RefCell::new(classes);
        t.set_resolver(Box::new(move |class, _, states, _| {
            let env = crate::classes::Env {
                states,
                ..Default::default()
            };
            classes.borrow_mut().record(class, &env)
        }));
        buttons(
            &mut t,
            &[
                "w-12 h-12",
                "w-12 h-12 focus-visible:bg-[#00ff00]",
                // A combination of states (Button's `data-pressed:hover:`)
                // makes the entry read them all, focus-visible among them.
                "w-12 h-12 hover:active:bg-[#0000ff]",
            ],
        );
        let outline = |t: &Tree, id: u32| t.els[t.op(id).unwrap() as usize].record.skin.outline;
        t.key("Tab", false, false, false, false);
        assert_eq!(t.focused_id(), Some(1));
        let ring = outline(&t, 1);
        assert_eq!(
            (ring.width, ring.offset),
            (2.0, 2.0),
            "keys show the pack's ring"
        );
        assert!(
            ring.color[0] > 0.9 && ring.color[1] < 0.1,
            "in the pack's colour: {ring:?}"
        );
        t.key("Tab", false, false, false, false);
        assert_eq!(t.focused_id(), Some(2));
        assert_eq!(outline(&t, 1).width, 0.0, "blurred");
        assert_eq!(outline(&t, 2).width, 0.0, "its own focus look, no ring");
        t.key("Tab", false, false, false, false);
        assert_eq!(t.focused_id(), Some(3));
        assert_eq!(
            outline(&t, 3).width,
            2.0,
            "states combined elsewhere still ring"
        );
        t.key("Tab", true, false, false, false);
        t.key("Tab", true, false, false, false);
        // A press focuses without the ring.
        at(&mut t, 5.0, 5.0, true, false);
        at(&mut t, 5.0, 5.0, false, true);
        assert_eq!(t.focused_id(), Some(1));
        assert_eq!(outline(&t, 1).width, 0.0, "a press shows no ring");
    }

    /// A control that answers a press squashes to the pack's
    /// `Press.feedback` under it, in on the slot's `active:` curve and
    /// back on its spring, over its own transform; one whose classes move
    /// it under a press keeps only its own, and a disabled control or a
    /// view that answers no press stays still.
    #[test]
    fn a_press_squashes_a_control_by_the_packs_press_feedback() {
        let mut t = tree();
        let mut classes = crate::classes::Classes::new(None).unwrap().0;
        classes.set_variants(
            [(
                "Press.feedback".to_string(),
                "transition-transform ease-spring-[900,30] active:scale-90 active:duration-50 active:ease-out"
                    .to_string(),
            )]
            .into(),
        );
        let classes = std::cell::RefCell::new(classes);
        t.set_resolver(Box::new(move |class, _, states, _| {
            let env = crate::classes::Env {
                states,
                ..Default::default()
            };
            classes.borrow_mut().record(class, &env)
        }));
        buttons(
            &mut t,
            &[
                "w-12 h-12 translate-x-1 transition-colors duration-300",
                "w-12 h-12 active:scale-[0.5]",
            ],
        );
        let rec = |t: &Tree, id: u32| t.els[t.op(id).unwrap() as usize].record.clone();
        at(&mut t, 5.0, 5.0, true, false);
        let down = rec(&t, 1);
        assert_eq!(down.skin.transform.scale, [0.9, 0.9], "squashed");
        assert_eq!(
            down.skin.transform.translate[0], 4.0,
            "over its own translate"
        );
        assert!(
            matches!(down.motion.transform, Some((crate::motion::Ease::Bezier(_), d)) if (d - 0.05).abs() < 1e-6),
            "in on the slot's active curve: {:?}",
            down.motion.transform
        );
        assert_eq!(
            down.motion.duration, 0.3,
            "its colours keep their own timing"
        );
        at(&mut t, 5.0, 5.0, false, true);
        let up = rec(&t, 1);
        assert_eq!(up.skin.transform.scale, [1.0, 1.0], "let go");
        assert!(
            matches!(
                up.motion.transform,
                Some((crate::motion::Ease::Spring { .. }, _))
            ),
            "back on the spring: {:?}",
            up.motion.transform
        );
        at(&mut t, 5.0, 53.0, true, false);
        assert_eq!(
            rec(&t, 2).skin.transform.scale,
            [0.5, 0.5],
            "its own press only"
        );
        assert_eq!(rec(&t, 2).motion.transform, None);
        at(&mut t, 5.0, 53.0, false, true);
        // Disabled: the press does not reach it, and a held state shows no squash.
        let mut b = Batch::default();
        b.op(4).u32(1).str("w-12 h-12 disabled:opacity-50");
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let e = t.op(1).unwrap();
        t.set_state(e, DISABLED, true);
        t.set_state(e, PRESSED, true);
        assert_eq!(
            rec(&t, 1).skin.transform.scale,
            [1.0, 1.0],
            "disabled stays still"
        );
    }

    /// While a `modal` view shows, focus goes into it, Tab stays in
    /// it and a press outside reaches nothing; when it goes, focus
    /// returns to what had it before.
    #[test]
    fn a_modal_traps_focus_and_returns_it() {
        let mut t = stated();
        buttons(&mut t, &["b", "b"]);
        t.key("Tab", false, false, false, false);
        t.key("Tab", false, false, false, false);
        assert_eq!(t.focused_id(), Some(2));
        let mut b = Batch::default();
        b.op(1).u32(50).u8(0);
        b.op(6).u32(50).str("modal").str("true");
        for id in [51, 52] {
            b.op(1).u32(id).u8(0);
            b.op(4).u32(id).str("b");
            b.op(7).u32(id).u8(Event::Press as u8).u8(1);
            b.op(8).u32(50).u32(id).u32(u32::MAX);
        }
        b.op(8).u32(0).u32(50).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        t.key("Tab", false, false, false, false);
        assert_eq!(t.focused_id(), Some(52), "opened on 51, Tab moved on");
        t.key("Tab", false, false, false, false);
        assert_eq!(t.focused_id(), Some(51), "Tab wraps inside the modal");
        t.events.clear();
        at(&mut t, 10.0, 10.0, true, false);
        at(&mut t, 10.0, 10.0, false, true);
        assert!(
            !t.events.iter().any(|f| f.id == 1),
            "the page behind answers no press"
        );
        let mut b = Batch::default();
        b.op(9).u32(0).u32(50);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        t.action("noop");
        assert_eq!(t.focused_id(), Some(2), "focus returned");
    }

    /// An input action goes to the focused node or the nearest around it
    /// that lists it; `confirm` presses a focused control no one lists it
    /// on, once, though Enter is bound to it too.
    #[test]
    fn actions_go_to_the_focus_and_its_ancestors() {
        let mut t = stated();
        buttons(&mut t, &["b", "b"]);
        let mut b = Batch::default();
        b.op(6).u32(100).str("actions").str("details back");
        t.apply(&b.bytes()).unwrap();
        assert!(!t.action("details"), "nothing focused, no modal");
        t.key("Tab", false, false, false, false);
        t.events.clear();
        assert!(t.action("details"));
        assert!(!t.action("rotate"));
        let ev = &t.events[0];
        assert_eq!((ev.id, ev.event, &*ev.key), (100, Event::Action, "details"));
        t.events.clear();
        // Enter presses the focused control, and its `confirm` does not
        // again.
        assert!(t.key("Enter", false, false, false, false));
        t.action("confirm");
        t.input_done(false);
        let presses: Vec<_> = t.events.iter().map(|f| (f.id, f.event)).collect();
        assert_eq!(presses, [(1, Event::Press)]);
        t.events.clear();
        t.action("confirm");
        assert_eq!(t.events.len(), 1, "a pad's confirm presses it");
    }

    /// A `<Glyph action>` shows the host's binding glyph, and asks for it
    /// again when one comes; the pack's art for the binding draws in its
    /// text's place, and follows the device: a pad's art replaces the
    /// key's, and a binding without art shows its text again.
    #[test]
    fn a_glyph_shows_its_actions_binding() {
        let mut t = stated();
        let mut b = Batch::default();
        // A `<Text glyph>` host element, with no text child of its own.
        b.op(1).u32(1).u8(0);
        b.op(6).u32(1).str("glyph").str("use");
        b.op(8).u32(0).u32(1).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        assert!(t.glyphs_wanted());
        t.set_glyphs(&|a| (a == "use").then(|| ("E".to_string(), 0)));
        assert!(!t.glyphs_wanted());
        let e = t.op(1).unwrap();
        assert_eq!(t.els[e as usize].text, "E");
        frame(&mut t);
        let leaf = t.els[e as usize].leaf.expect("the glyph lays out as text");
        assert!(t.arena.box_of(leaf).w > 0.0, "the binding's text has width");
        let art = |t: &mut Tree| -> Vec<u32> {
            frame(t);
            t.drawn
                .quads
                .iter()
                .filter(|q| q.kind == PaintKind::Image as u32)
                .map(|q| {
                    assert_eq!(q.uv, [0.0, 0.0, 1.0, 1.0], "the whole texture");
                    q.a
                })
                .collect()
        };
        let bits = |slot: u32| og_draw::TextureHandle::from_slot(slot).unwrap().to_bits();
        assert!(art(&mut t).is_empty(), "no art: the text");
        // The keyboard's art, then a pad's in its place.
        t.set_glyphs(&|_| Some(("E".to_string(), 7)));
        assert_eq!(art(&mut t), [bits(7)]);
        assert_eq!(
            t.els[e as usize].text, "\u{2003}",
            "an em space under the art"
        );
        t.set_glyphs(&|_| Some(("A".to_string(), 5)));
        assert_eq!(art(&mut t), [bits(5)], "the pad's art, not the key's");
        // A binding without art shows its text again.
        t.set_glyphs(&|_| Some(("X".to_string(), 0)));
        assert!(art(&mut t).is_empty());
        assert_eq!(t.els[e as usize].text, "X");
    }

    /// Arrows and the d-pad move the focus to the nearest stop that way,
    /// in its row or column first, and wrap to the far end of the row;
    /// the first one lands on the first stop. A focused node that hears
    /// keys gets the arrow instead.
    #[test]
    fn arrows_move_the_focus_through_a_grid() {
        let mut t = stated();
        // Two rows of two in a column: 1 2 / 3 4.
        let mut b = Batch::default();
        b.op(1).u32(99).u8(0);
        b.op(4).u32(99).str("col");
        b.op(8).u32(0).u32(99).u32(u32::MAX);
        for (row, ids) in [(100, [1, 2]), (101, [3, 4])] {
            b.op(1).u32(row).u8(0);
            b.op(4).u32(row).str("row");
            b.op(8).u32(99).u32(row).u32(u32::MAX);
            for id in ids {
                b.op(1).u32(id).u8(0);
                b.op(4).u32(id).str("b");
                b.op(7).u32(id).u8(Event::Press as u8).u8(1);
                b.op(8).u32(row).u32(id).u32(u32::MAX);
            }
        }
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        assert_eq!(t.op_box(4).unwrap(), [50.0, 20.0, 50.0, 20.0]);
        let mut path = Vec::new();
        for key in [
            "ArrowDown",
            "ArrowRight",
            "ArrowDown",
            "ArrowLeft",
            "ArrowUp",
            "ArrowLeft",
        ] {
            assert!(t.key(key, false, false, false, false));
            path.push(t.focused_id().unwrap());
        }
        assert_eq!(
            path,
            [1, 2, 4, 3, 1, 2],
            "the last Left wraps along the row"
        );
        let mut b = Batch::default();
        b.op(7).u32(2).u8(Event::Key as u8).u8(1);
        t.apply(&b.bytes()).unwrap();
        t.events.clear();
        t.key("ArrowDown", false, false, false, false);
        assert_eq!(t.focused_id(), Some(2), "it took the arrow");
        assert_eq!((t.events[0].id, &*t.events[0].key), (2, "ArrowDown"));
        assert!(
            t.nav(og_input::frame::Nav::Down),
            "the d-pad still moves it"
        );
        assert_eq!(t.focused_id(), Some(4));
    }

    /// A pointerdown's listener hears the moves and the release until the
    /// release, though the pointer left it (a drag); its neighbour under
    /// the pointer hears none of them, and after the release moves go to
    /// what is under the pointer again.
    #[test]
    fn a_pointerdown_captures_the_moves_until_the_release() {
        let mut t = stated();
        let mut b = Batch::default();
        b.op(1).u32(100).u8(0);
        b.op(4).u32(100).str("row");
        b.op(8).u32(0).u32(100).u32(u32::MAX);
        for id in [1, 2] {
            b.op(1).u32(id).u8(0);
            b.op(4).u32(id).str("b");
            for ev in [Event::PointerDown, Event::PointerMove, Event::PointerUp] {
                b.op(7).u32(id).u8(ev as u8).u8(1);
            }
            b.op(8).u32(100).u32(id).u32(u32::MAX);
        }
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        at(&mut t, 10.0, 10.0, true, false);
        // Over 2, then off everything.
        at(&mut t, 70.0, 10.0, false, false);
        at(&mut t, 300.0, 200.0, false, true);
        let got: Vec<_> = t.events.iter().map(|f| (f.id, f.event)).collect();
        assert_eq!(
            got,
            [
                // It came over 1 as it went down.
                (1, Event::PointerMove),
                (1, Event::PointerDown),
                (1, Event::PointerMove),
                (1, Event::PointerMove),
                (1, Event::PointerUp)
            ]
        );
        t.events.clear();
        at(&mut t, 70.0, 10.0, false, false);
        assert_eq!(
            t.events[0].id, 2,
            "released: the moves go under the pointer"
        );
    }

    /// A 50 by 40 `Scroll` (id 10) of four 50 by 20 rows (ids 1 to 4,
    /// pressable) in a column: 40 px run past its end.
    fn list(t: &mut Tree) {
        let mut b = Batch::default();
        b.op(1).u32(99).u8(0);
        b.op(4).u32(99).str("col");
        b.op(8).u32(0).u32(99).u32(u32::MAX);
        b.op(1).u32(10).u8(3);
        b.op(4).u32(10).str("col s");
        b.op(8).u32(99).u32(10).u32(u32::MAX);
        for id in 1..=4 {
            b.op(1).u32(id).u8(0);
            b.op(4).u32(id).str("b");
            b.op(7).u32(id).u8(Event::Press as u8).u8(1);
            b.op(8).u32(10).u32(id).u32(u32::MAX);
        }
        t.apply(&b.bytes()).unwrap();
        frame(t);
    }

    fn scrolled(t: &Tree) -> f32 {
        t.scroll_at(t.op(10).unwrap(), false)
    }

    /// A scroll view whose class sets a fixed 50 by 40 box (`s`).
    fn scrolling() -> Tree {
        let mut t = tree();
        t.set_resolver(Box::new(|class, _, _, _| {
            let mut s = taffy::Style::default();
            for c in class.split_whitespace() {
                match c {
                    "col" => s.flex_direction = taffy::FlexDirection::Column,
                    "s" => s.size = taffy::Size::from_lengths(50.0, 40.0),
                    "b" => {
                        s.size = taffy::Size::from_lengths(50.0, 20.0);
                        s.flex_shrink = 0.0;
                    }
                    "wb" => {
                        s.size = taffy::Size::from_lengths(80.0, 20.0);
                        s.flex_shrink = 0.0;
                    }
                    "oy" => s.overflow.y = taffy::Overflow::Scroll,
                    "bd" => s.border = taffy::Rect::length(2.0),
                    _ => {}
                }
            }
            let mut r = Record {
                layout: rune_taffy::CompactStyle::from(&s),
                ..Record::default()
            };
            // `bd`: a 2 px border, laid out and drawn.
            if class.split_whitespace().any(|c| c == "bd") {
                r.skin.border = [2.0; 4];
            }
            r
        }));
        t
    }

    /// The wheel scrolls the scroller under it, within what runs past its
    /// end; its rows move with it, a press lands on the row now there,
    /// and the clipped row under it answers nothing.
    #[test]
    fn the_wheel_scrolls_a_scroll_and_its_rows_move() {
        let mut t = scrolling();
        list(&mut t);
        assert_eq!(t.op_box(2).unwrap()[1], 20.0);
        t.wheel(10.0, 10.0, 0.0, 30.0);
        frame(&mut t);
        assert_eq!(scrolled(&t), 30.0);
        assert_eq!(t.op_box(2).unwrap()[1], -10.0, "row 2 moved up by 30");
        t.wheel(10.0, 10.0, 0.0, 100.0);
        frame(&mut t);
        assert_eq!(scrolled(&t), 40.0, "no further than its end");
        // Row 4 (60..80) now shows at 20..40.
        at(&mut t, 10.0, 30.0, true, false);
        at(&mut t, 10.0, 30.0, false, true);
        assert_eq!(t.events.iter().map(|f| f.id).collect::<Vec<_>>(), [4]);
        t.events.clear();
        // Below the scroller (y 50) is row 4's old place: clipped, no hit.
        at(&mut t, 10.0, 50.0, true, false);
        at(&mut t, 10.0, 50.0, false, true);
        assert!(t.events.is_empty());
    }

    /// A `View` whose overflow is `auto` along an axis scrolls along it
    /// as a `Scroll` does, and along that axis only; one whose overflow
    /// is visible does not scroll. What it clips is cut at its padding
    /// box, inside its border.
    #[test]
    fn an_overflow_auto_view_scrolls_along_its_axis() {
        for (class, down) in [("col s", 0.0), ("col s oy bd", 30.0)] {
            let mut t = scrolling();
            let mut b = Batch::default();
            b.op(1).u32(10).u8(0);
            b.op(4).u32(10).str(class);
            b.op(8).u32(0).u32(10).u32(u32::MAX);
            for id in 1..=4 {
                b.op(1).u32(id).u8(0);
                // 80 wide in 50: they run past its right side too.
                b.op(4).u32(id).str("wb");
                b.op(8).u32(10).u32(id).u32(u32::MAX);
            }
            t.apply(&b.bytes()).unwrap();
            frame(&mut t);
            t.wheel(10.0, 10.0, 0.0, 30.0);
            t.wheel(10.0, 10.0, 30.0, 0.0);
            frame(&mut t);
            let e = t.op(10).unwrap();
            assert_eq!(t.scroll_of(e), [0.0, down], "{class}");
            // Inside a 2 px border when it scrolls.
            let border = if down > 0.0 { 2.0 } else { 0.0 };
            assert_eq!(
                t.op_box(2).unwrap()[1],
                border + 20.0 - down,
                "{class}: its rows move"
            );
            if down > 0.0 {
                assert_eq!(
                    t.abs[e as usize].3,
                    [2.0, 2.0, 48.0, 38.0],
                    "inside the border"
                );
            }
        }
    }

    /// `-translate-x-1/2 -translate-y-1/2` moves a box by half its own size
    /// at paint, centring it on its `left-1/2 top-1/2` point. Real classes,
    /// end to end.
    #[test]
    fn a_fraction_translate_is_of_the_box_itself() {
        let mut t = tree();
        let classes = std::cell::RefCell::new(crate::classes::Classes::new(None).unwrap().0);
        t.set_resolver(Box::new(move |class, _, _, _| {
            classes
                .borrow_mut()
                .record(class, &crate::classes::Env::default())
        }));
        let mut b = Batch::default();
        for (id, class, parent) in [
            (1, "relative w-[100px] h-[100px]", 0),
            (
                2,
                "absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-[40px] h-[20px] bg-white",
                1,
            ),
        ] {
            b.op(1).u32(id).u8(0);
            b.op(4).u32(id).str(class);
            b.op(8).u32(parent).u32(id).u32(u32::MAX);
        }
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let [x, y, w, h] = t.op_box(2).unwrap();
        assert_eq!([x, y, w, h], [50.0, 50.0, 40.0, 20.0], "layout stays");
        let q = t.drawn.quads.last().unwrap();
        let [a, b, c, d, e, f, _, _] = t.drawn.transforms[q.xf as usize];
        assert_eq!(
            [a * x + c * y + e, b * x + d * y + f],
            [30.0, 40.0],
            "drawn half its size back"
        );
    }

    /// An inline `calc()`, `min()`, `max()` or `clamp()` resolves against
    /// the containing block at layout, as the browser's does: the
    /// crosshair's `left: calc(50% - 0.0715rem)` centres its arm instead of
    /// dropping to 0.
    #[test]
    fn an_inline_calc_resolves_against_the_containing_block() {
        let mut t = tree();
        let classes = std::cell::RefCell::new(crate::classes::Classes::new(None).unwrap().0);
        t.set_resolver(Box::new(move |class, style, _, _| {
            classes
                .borrow_mut()
                .record_styled(class, style, &crate::classes::Env::default())
        }));
        let mut b = Batch::default();
        for (id, class, style, parent) in [
            (1, "relative w-[200px] h-[100px]", "", 0),
            (
                2,
                "absolute h-[10px]",
                "left: calc(50% - 10px); top: calc(50% - 0.3125rem); width: calc(3 * (25% - 2px))",
                1,
            ),
            (
                3,
                "absolute h-[10px]",
                "right: clamp(8px, 25%, 40px); bottom: max(5px, 10%); width: min(30px, 50%)",
                1,
            ),
        ] {
            b.op(1).u32(id).u8(0);
            b.op(4).u32(id).str(class);
            b.op(5).u32(id).str(style);
            b.op(8).u32(parent).u32(id).u32(u32::MAX);
        }
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        assert_eq!(t.op_box(2).unwrap(), [90.0, 45.0, 144.0, 10.0]);
        // right 40 (25% is 50, clamped), bottom 10 (10% of 100).
        assert_eq!(t.op_box(3).unwrap(), [130.0, 80.0, 30.0, 10.0]);
    }

    /// A `w-fit` wrapping row (strategy/resources) is as wide as its items
    /// on one line and keeps them on it, whatever layout's sums lose; one
    /// its box caps wraps, and is as tall as its lines.
    #[test]
    fn a_fit_wrapping_row_keeps_its_line_and_grows_by_its_lines() {
        let row = "flex-row flex-wrap items-center w-fit max-w-full min-w-0 gap-x-5.5 gap-y-1.5 py-1.5 px-4";
        // Widths whose sum, taken back in the line break's order, lost an
        // ulp: the last one wrapped and the row stayed one line tall.
        let widths = [43.014706f32, 39.824173, 67.97606, 40.306263, 67.63821];
        for (width, lines) in [(600.0, 1), (200.0, 3)] {
            let mut t = tree();
            let classes = std::cell::RefCell::new(crate::classes::Classes::new(None).unwrap().0);
            t.set_resolver(Box::new(move |class, style, _, _| {
                classes
                    .borrow_mut()
                    .record_styled(class, style, &crate::classes::Env::default())
            }));
            let mut b = Batch::default();
            let mut nodes = vec![
                (1, format!("flex-col items-start w-[{width}px]"), 0),
                (2, row.to_string(), 1),
            ];
            for (k, w) in widths.iter().enumerate() {
                nodes.push((10 + k as u32, format!("h-[20px] w-[{w}px] shrink-0"), 2));
            }
            for (id, class, parent) in &nodes {
                b.op(1).u32(*id).u8(0);
                b.op(4).u32(*id).str(class);
                b.op(8).u32(*parent).u32(*id).u32(u32::MAX);
            }
            t.apply(&b.bytes()).unwrap();
            frame(&mut t);
            let tops: std::collections::BTreeSet<u32> = (10..15)
                .map(|id| t.op_box(id).unwrap()[1].to_bits())
                .collect();
            let [_, _, w, h] = t.op_box(2).unwrap();
            assert_eq!(tops.len(), lines, "{width}: lines {tops:?}, row {w}×{h}");
            let want = lines as f32 * 20.0 + (lines - 1) as f32 * 6.0 + 12.0;
            assert_eq!(h, want, "{width}: the row holds its lines");
        }
    }

    /// A tutorial step: `spotlight="goal"` draws the shade's `bg-*` over
    /// the surface outside the node named `goal`, rounded as the goal is
    /// (one quad cut by an inverted round of its border box and radii),
    /// and takes presses only there, a corner outside the goal's arc
    /// included, so the goal takes its own; a card anchored to `goal` with
    /// `lift` 0,1 stands under it, and follows it when it moves.
    #[test]
    fn a_spotlight_lights_its_named_node_and_a_card_anchors_to_it() {
        let mut t = tree();
        let classes = std::cell::RefCell::new(crate::classes::Classes::new(None).unwrap().0);
        t.set_resolver(Box::new(move |class, style, _, _| {
            classes
                .borrow_mut()
                .record_styled(class, style, &crate::classes::Env::default())
        }));
        let mut b = Batch::default();
        for (id, class, parent) in [
            (1, "relative w-[400px] h-[300px]", 0),
            (
                2,
                "absolute left-[100px] top-[50px] w-[60px] h-[40px] bg-white rounded-[16px]",
                1,
            ),
            (3, "absolute left-0 top-0 w-[10px] h-[10px] bg-black/50", 1),
            (4, "w-[50px] h-[20px]", 3),
        ] {
            b.op(1).u32(id).u8(0);
            b.op(4).u32(id).str(class);
            b.op(8).u32(parent).u32(id).u32(u32::MAX);
        }
        b.op(6).u32(2).str("name").str("goal");
        b.op(7).u32(2).u8(Event::Press as u8).u8(1);
        b.op(6).u32(3).str("spotlight").str("goal");
        b.op(7).u32(3).u8(Event::Press as u8).u8(1);
        b.op(6).u32(4).str("anchor").str("goal");
        b.op(6).u32(4).str("lift").str("0,1,0");
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        // The shade: one quad over the surface, cut by the goal's rounded
        // box, inverted.
        let shade: Vec<&PaintQuad> = t
            .drawn
            .quads
            .iter()
            .filter(|q| q.color[..3] == [0.0; 3] && q.color[3] > 0.0)
            .collect();
        assert_eq!(shade.len(), 1, "{shade:?}");
        let (w, h) = t.size;
        assert_eq!(shade[0].rect, [0.0, 0.0, w, h]);
        let hole = t.drawn.rounds[shade[0].round as usize - 1];
        assert_eq!(hole.rect, [100.0, 50.0, 160.0, 90.0]);
        assert_eq!(hole.radius, [16.0; 4]);
        assert_eq!(hole.invert, 1);
        assert_eq!(t.op_box(4).unwrap(), [100.0, 90.0, 50.0, 20.0]);
        let press = |t: &mut Tree, x, y| {
            t.events.clear();
            t.pointer(x, y, true, false);
            t.pointer(x, y, false, true);
            t.events.iter().map(|f| f.id).collect::<Vec<_>>()
        };
        assert_eq!(press(&mut t, 130.0, 70.0), [2], "the goal takes its own");
        assert_eq!(press(&mut t, 300.0, 200.0), [3], "the shade takes the rest");
        // Its top left corner: inside the box, outside the 16 px arc (whose
        // centre is at 116, 66), and a point just inside the arc.
        assert_eq!(press(&mut t, 102.0, 52.0), [3], "outside the arc");
        assert_eq!(press(&mut t, 106.0, 56.0), [2], "inside the arc");
        // The goal moves: the hole and the card follow.
        let mut b = Batch::default();
        b.op(4)
            .u32(2)
            .str("absolute left-[200px] top-[150px] w-[60px] h-[40px] bg-white");
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        assert_eq!(t.op_box(4).unwrap(), [200.0, 190.0, 50.0, 20.0]);
        assert_eq!(press(&mut t, 230.0, 170.0), [2]);
        assert_eq!(press(&mut t, 130.0, 70.0), [3]);
    }

    /// A spotlight carries `data-lit` while the node it names shows: on,
    /// off once the target hides, on again when it shows; its classes and
    /// its group's (`group-data-lit/spot:`) read it in the same frame.
    #[test]
    fn a_spotlight_is_lit_while_its_node_shows() {
        let mut t = tree();
        let classes = std::cell::RefCell::new(crate::classes::Classes::new(None).unwrap().0);
        t.set_resolver(Box::new(move |class, style, states, data| {
            classes.borrow_mut().record_data(
                class,
                style,
                &crate::classes::Env {
                    states,
                    ..crate::classes::Env::default()
                },
                data,
            )
        }));
        let goal = |hidden: bool| {
            format!(
                "absolute left-[100px] top-[50px] w-[60px] h-[40px] {}",
                if hidden { "hidden" } else { "" }
            )
        };
        let mut b = Batch::default();
        for (id, class, parent) in [
            (1, "relative w-[400px] h-[300px]".to_string(), 0),
            (2, goal(false), 1),
            (
                3,
                "group/spot absolute inset-0 data-lit:bg-black/50".to_string(),
                1,
            ),
            (
                4,
                "w-[50px] h-[20px] group-data-lit/spot:hidden".to_string(),
                3,
            ),
        ] {
            b.op(1).u32(id).u8(0);
            b.op(4).u32(id).str(&class);
            b.op(8).u32(parent).u32(id).u32(u32::MAX);
        }
        b.op(6).u32(2).str("name").str("goal");
        b.op(6).u32(3).str("spotlight").str("goal");
        t.apply(&b.bytes()).unwrap();
        let shaded = |t: &Tree| {
            t.drawn
                .quads
                .iter()
                .any(|q| q.color[..3] == [0.0; 3] && q.color[3] > 0.0)
        };
        for (hidden, lit) in [(false, true), (true, false), (false, true)] {
            let mut b = Batch::default();
            b.op(4).u32(2).str(&goal(hidden));
            t.apply(&b.bytes()).unwrap();
            frame(&mut t);
            let el = &t.els[t.op(3).unwrap() as usize];
            assert_eq!(
                el.attrs.iter().any(|a| &**a == "lit"),
                lit,
                "hidden {hidden}"
            );
            assert_eq!(shaded(&t), lit, "the shade's bg, hidden {hidden}");
            assert_eq!(
                t.op_box(4).is_none(),
                lit,
                "the fallback card, hidden {hidden}"
            );
        }
    }

    /// Classes lay out with CSS's initial flex values: a `w-full min-w-0`
    /// bar beside a `shrink-0` count gives way to it (meta/daily-quests),
    /// and a `flex-1 min-h-0` `Scroll` in a column sized to its content
    /// keeps its content's height (Tailwind's `0%` basis; racing/standings).
    /// Real classes, end to end.
    #[test]
    fn classes_lay_out_with_css_flex_defaults() {
        let mut t = tree();
        let classes = std::cell::RefCell::new(crate::classes::Classes::new(None).unwrap().0);
        t.set_resolver(Box::new(move |class, _, _, _| {
            classes
                .borrow_mut()
                .record(class, &crate::classes::Env::default())
        }));
        let mut b = Batch::default();
        for (id, tag, class, parent) in [
            (1, 0, "flex flex-row w-[300px]", 0),
            (2, 0, "w-full min-w-0 h-[10px]", 1),
            (3, 0, "shrink-0 w-[50px] h-[10px]", 1),
            (7, 0, "flex flex-row items-start h-[300px]", 0),
            (4, 0, "flex flex-col w-[100px]", 7),
            (5, 3, "flex-1 min-h-0", 4),
            (6, 0, "h-[40px]", 5),
        ] {
            b.op(1).u32(id).u8(tag);
            b.op(4).u32(id).str(class);
            b.op(8).u32(parent).u32(id).u32(u32::MAX);
        }
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        assert_eq!(t.op_box(2).unwrap()[2], 250.0, "the bar gives way");
        assert_eq!(t.op_box(3).unwrap()[0], 250.0, "the count sits beside it");
        assert_eq!(
            t.op_box(5).unwrap()[3],
            40.0,
            "the scroll keeps its rows' height"
        );
    }

    /// Real z classes affect both quad compositing and which overlapping
    /// button hears a press. Auto descendants may escape their parent;
    /// explicit zero isolates them. Negative children stay above their
    /// context's background, and equal levels follow document order.
    #[test]
    fn z_index_paints_and_hits_in_isolated_contexts() {
        let mut t = tree();
        let classes = std::cell::RefCell::new(crate::classes::Classes::new(None).unwrap().0);
        t.set_resolver(Box::new(move |class, _, _, _| {
            classes
                .borrow_mut()
                .record(class, &crate::classes::Env::default())
        }));
        let box_class = "absolute left-0 top-0 w-[100px] h-[80px]";
        let mut b = Batch::default();
        for (id, parent, class) in [
            (
                1,
                0,
                "relative w-[200px] h-[160px] z-0 bg-[#000000]".to_string(),
            ),
            (2, 1, format!("{box_class} bg-[#00ff00]")),
            (3, 2, format!("{box_class} z-20 bg-[#ff0000]")),
            (4, 1, format!("{box_class} z-10 bg-[#0000ff]")),
        ] {
            b.op(1).u32(id).u8(0);
            b.op(4).u32(id).str(&class);
            b.op(8).u32(parent).u32(id).u32(u32::MAX);
            b.op(7).u32(id).u8(Event::Press as u8).u8(1);
        }
        t.apply(&b.bytes()).unwrap();
        let check = |t: &mut Tree, colors: &[[f32; 4]], button: u32| {
            frame(t);
            assert_eq!(
                t.drawn
                    .quads
                    .iter()
                    .filter(|q| q.kind == PaintKind::Solid as u32)
                    .map(|q| q.color)
                    .collect::<Vec<_>>(),
                colors
            );
            t.pointer(25.0, 25.0, true, false);
            t.pointer(25.0, 25.0, false, true);
            let fired = std::mem::take(&mut t.events);
            assert_eq!(
                fired
                    .iter()
                    .filter(|f| f.event == Event::Press)
                    .map(|f| f.id)
                    .collect::<Vec<_>>(),
                [button]
            );
            assert_eq!(
                t.op_box(3).unwrap(),
                [0.0, 0.0, 100.0, 80.0],
                "stacking never changes layout"
            );
        };
        let (black, green, red, blue) = (
            [0., 0., 0., 1.],
            [0., 1., 0., 1.],
            [1., 0., 0., 1.],
            [0., 0., 1., 1.],
        );
        check(&mut t, &[black, green, blue, red], 3);
        let mut b = Batch::default();
        b.op(4).u32(2).str(&format!("{box_class} z-0 bg-[#00ff00]"));
        t.apply(&b.bytes()).unwrap();
        check(&mut t, &[black, green, red, blue], 4);
        let mut b = Batch::default();
        b.op(4)
            .u32(3)
            .str(&format!("{box_class} -z-10 bg-[#ff0000]"));
        t.apply(&b.bytes()).unwrap();
        check(&mut t, &[black, green, red, blue], 4);
        let mut b = Batch::default();
        b.op(4)
            .u32(2)
            .str(&format!("{box_class} z-auto bg-[#00ff00]"));
        b.op(4).u32(4).str(&format!("{box_class} z-0 bg-[#0000ff]"));
        t.apply(&b.bytes()).unwrap();
        check(&mut t, &[black, red, green, blue], 4);
        // Move the auto container after the equal-level sibling. Its
        // negative child still precedes it, but the green box now wins.
        let mut b = Batch::default();
        b.op(8).u32(1).u32(2).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        check(&mut t, &[black, red, blue, green], 2);

        // A negative child paints before its auto parent's own box, but
        // still receives that parent's current geometry and clip.
        let mut b = Batch::default();
        b.op(4)
            .u32(2)
            .str("absolute left-[10px] top-[15px] w-[60px] h-[40px] overflow-hidden bg-[#00ff00]");
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let q = t.drawn.quads.iter().find(|q| q.color == red).unwrap();
        assert_eq!(q.rect, [10., 15., 100., 80.]);
        assert_eq!(q.clip, [10., 15., 70., 55.]);
        t.pointer(90., 30., true, false);
        t.pointer(90., 30., false, true);
        assert_eq!(
            t.events
                .iter()
                .filter(|f| f.event == Event::Press)
                .map(|f| f.id)
                .collect::<Vec<_>>(),
            [4]
        );
    }

    /// `order-*` places a flex or grid item among its siblings for layout,
    /// as CSS's `order`: the flex row and the grid's auto-placement take
    /// order-modified document order. Real classes, end to end.
    #[test]
    fn order_classes_reorder_flex_and_grid_items() {
        let mut t = tree();
        let classes = std::cell::RefCell::new(crate::classes::Classes::new(None).unwrap().0);
        t.set_resolver(Box::new(move |class, _, _, _| {
            classes
                .borrow_mut()
                .record(class, &crate::classes::Env::default())
        }));
        let cell = "w-[10px] h-[10px]";
        let mut b = Batch::default();
        for (parent, class, kids) in [
            (1, "flex flex-row w-[100px]", [2, 3, 4]),
            (5, "grid grid-cols-[10px_10px_10px] w-[100px]", [6, 7, 8]),
        ] {
            b.op(1).u32(parent).u8(0);
            b.op(4).u32(parent).str(class);
            b.op(8).u32(0).u32(parent).u32(u32::MAX);
            for k in kids {
                b.op(1).u32(k).u8(0);
                b.op(8).u32(parent).u32(k).u32(u32::MAX);
            }
        }
        // The row: 2, then 3 first, then 4.
        b.op(4).u32(2).str(cell);
        b.op(4).u32(3).str(&format!("{cell} -order-1"));
        b.op(4).u32(4).str(cell);
        // The grid: 8 placed first.
        b.op(4).u32(6).str(cell);
        b.op(4).u32(7).str(cell);
        b.op(4).u32(8).str(&format!("{cell} order-first"));
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        // Each from its container's corner.
        let at = |t: &Tree, id: u32, of: u32| {
            let (b, c) = (t.op_box(id).unwrap(), t.op_box(of).unwrap());
            [b[0] - c[0], b[1] - c[1], b[2], b[3]]
        };
        let x = |t: &Tree, id: u32, of: u32| at(t, id, of)[0];
        assert_eq!(
            [x(&t, 3, 1), x(&t, 2, 1), x(&t, 4, 1)],
            [0.0, 10.0, 20.0],
            "the row"
        );
        assert_eq!(
            [x(&t, 8, 5), x(&t, 6, 5), x(&t, 7, 5)],
            [0.0, 10.0, 20.0],
            "the grid"
        );
    }

    /// A finger past the 8 px slop pans the scroller and lets the press
    /// go; let go while moving, it flings on, slowing to a stop. Within
    /// the slop it is a tap.
    #[test]
    fn a_finger_pans_and_flings_a_scroll() {
        let mut t = scrolling();
        list(&mut t);
        t.set_device(true, false);
        t.set_time(1.0, false);
        at(&mut t, 10.0, 30.0, true, false);
        t.set_time(1.016, false);
        at(&mut t, 10.0, 25.0, false, false);
        assert_eq!(scrolled(&t), 0.0, "within the slop");
        t.set_time(1.032, false);
        at(&mut t, 10.0, 18.0, false, false);
        assert_eq!(scrolled(&t), 12.0);
        t.set_time(1.048, false);
        at(&mut t, 10.0, 15.0, false, true);
        assert!(t.events.is_empty(), "a pan presses nothing");
        assert_eq!(scrolled(&t), 15.0);
        // 3 px in 16 ms: it coasts on, never back.
        let mut last = 15.0;
        for k in 1..60 {
            t.set_time(1.048 + k as f64 / 60.0, false);
            t.input_done(false);
            frame(&mut t);
            assert!(scrolled(&t) >= last);
            last = scrolled(&t);
        }
        // About 190 px/s coasts about 93 px (v / 2.002): it meets the end.
        assert_eq!(last, 40.0, "it coasts on to the end and stops there");
        assert!(!t.input_busy());
        t.set_time(3.0, false);
        at(&mut t, 10.0, 10.0, true, false);
        at(&mut t, 12.0, 12.0, false, true);
        assert_eq!(t.events.len(), 1, "a tap within the slop presses");
    }

    /// A finger held still 0.5 s on a node that listens long-presses it,
    /// and its release presses nothing.
    #[test]
    fn a_still_finger_long_presses() {
        let mut t = scrolling();
        list(&mut t);
        let mut b = Batch::default();
        b.op(7).u32(2).u8(Event::LongPress as u8).u8(1);
        t.apply(&b.bytes()).unwrap();
        t.set_device(true, false);
        t.set_time(1.0, false);
        at(&mut t, 10.0, 30.0, true, false);
        t.set_time(1.4, false);
        t.input_done(false);
        assert!(t.events.is_empty());
        t.set_time(1.5, false);
        t.input_done(false);
        at(&mut t, 10.0, 30.0, false, true);
        let got: Vec<_> = t.events.iter().map(|f| (f.id, f.event)).collect();
        assert_eq!(got, [(2, Event::LongPress)]);
    }

    /// A press held on a `hold` node reports its progress and presses it
    /// when it completes, not on release; let go early, it reports 0 and
    /// presses nothing. `confirm` held on its focus does the same.
    #[test]
    fn a_hold_reports_progress_and_presses_when_it_completes() {
        let mut t = stated();
        buttons(&mut t, &["b"]);
        let mut b = Batch::default();
        b.op(6).u32(1).str("hold").str("0.5");
        b.op(7).u32(1).u8(Event::Hold as u8).u8(1);
        t.apply(&b.bytes()).unwrap();
        let got = |t: &mut Tree| {
            let v: Vec<_> = t
                .events
                .drain(..)
                .filter(|f| matches!(f.event, Event::Hold | Event::Press))
                .map(|f| (f.event, f.value.to_string()))
                .collect();
            v
        };
        t.set_time(1.0, false);
        at(&mut t, 10.0, 10.0, true, false);
        t.set_time(1.25, false);
        t.input_done(false);
        assert_eq!(got(&mut t), [(Event::Hold, "0.50".into())]);
        t.set_time(1.5, false);
        t.input_done(false);
        assert_eq!(
            got(&mut t),
            [(Event::Hold, "1".into()), (Event::Press, "".into())]
        );
        at(&mut t, 10.0, 10.0, false, true);
        t.input_done(false);
        assert!(got(&mut t).is_empty(), "the release presses nothing more");
        // Let go early.
        t.set_time(2.0, false);
        at(&mut t, 10.0, 10.0, true, false);
        t.set_time(2.1, false);
        at(&mut t, 10.0, 10.0, false, true);
        t.input_done(false);
        assert_eq!(got(&mut t), [(Event::Hold, "0".into())]);
        // Confirm held on the focus.
        t.key("Tab", false, false, false, false);
        t.key("Tab", false, false, false, false);
        assert_eq!(t.focused_id(), Some(1));
        t.set_time(3.0, false);
        t.action("confirm");
        t.input_done(true);
        t.set_time(3.6, false);
        t.input_done(true);
        let v = got(&mut t);
        assert_eq!(v.last(), Some(&(Event::Press, "".into())));
    }

    /// Focus the keys move to a row below a scroller's end scrolls it
    /// into view.
    #[test]
    fn keyboard_focus_scrolls_into_view() {
        let mut t = scrolling();
        list(&mut t);
        for _ in 0..4 {
            t.key("Tab", false, false, false, false);
            frame(&mut t);
        }
        assert_eq!(t.focused_id(), Some(4));
        assert_eq!(scrolled(&t), 40.0, "row 4 (60..80) shows at the end");
        t.key("Tab", true, false, false, false);
        t.key("Tab", true, false, false, false);
        t.key("Tab", true, false, false, false);
        frame(&mut t);
        assert_eq!(scrolled(&t), 0.0, "row 1 at the top");
    }

    /// An `autofocus` node takes the focus once it shows, with the ring
    /// when the player used a pad last; a pointer event's `value` is its
    /// node's box, which a slider reads its track from.
    #[test]
    fn autofocus_and_the_box_on_pointer_events() {
        let mut t = stated();
        t.set_device(false, true);
        let mut b = Batch::default();
        b.op(1).u32(100).u8(0);
        b.op(4).u32(100).str("row");
        b.op(8).u32(0).u32(100).u32(u32::MAX);
        for id in [1, 2] {
            b.op(1).u32(id).u8(0);
            b.op(4).u32(id).str("b fw");
            b.op(7).u32(id).u8(Event::Press as u8).u8(1);
            b.op(7).u32(id).u8(Event::PointerDown as u8).u8(1);
            b.op(8).u32(100).u32(id).u32(u32::MAX);
        }
        b.op(6).u32(2).str("autofocus").str("true");
        t.apply(&b.bytes()).unwrap();
        t.input_done(false);
        assert_eq!(t.focused_id(), None, "not shown yet");
        frame(&mut t);
        t.input_done(false);
        assert_eq!(t.focused_id(), Some(2));
        frame(&mut t);
        assert_eq!(t.op_box(2).unwrap()[2], 90.0, "a pad player sees the ring");
        t.events.clear();
        // 2 is 90 wide at x 50 with its ring; press 3/4 along it.
        at(&mut t, 117.5, 5.0, true, false);
        let down = t
            .events
            .iter()
            .find(|f| f.event == Event::PointerDown)
            .unwrap();
        assert_eq!((down.id, &*down.value), (2, "50 0 90 20"));
    }

    /// A focused `Input` takes typed text at its caret and edits with its
    /// keys, a combining mark going with its base, and hears `change`
    /// with its text each time; its caret is drawn where it stands; a
    /// `value` set from outside replaces the text.
    #[test]
    fn an_input_edits_at_its_caret_and_hears_change() {
        let mut t = stated();
        let mut b = Batch::default();
        b.op(1).u32(1).u8(4);
        b.op(7).u32(1).u8(Event::Change as u8).u8(1);
        b.op(6).u32(1).str("placeholder").str("Name");
        b.op(8).u32(0).u32(1).u32(u32::MAX);
        b.op(1).u32(2).u8(0);
        b.op(7).u32(2).u8(Event::Press as u8).u8(1);
        b.op(8).u32(0).u32(2).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        assert!(!t.type_text("x"), "no focus, no edit");
        t.key("Tab", false, false, false, false);
        assert_eq!(t.focused_id(), Some(1));
        let changes = |t: &mut Tree| -> Vec<String> {
            let out = t
                .events
                .iter()
                .filter(|f| f.event == Event::Change)
                .map(|f| f.value.to_string())
                .collect();
            t.events.clear();
            out
        };
        t.events.clear();
        t.type_text("ab");
        t.type_text("e\u{301}");
        assert_eq!(changes(&mut t), ["ab", "abe\u{301}"]);
        // ← steps over the e and its accent; the arrows stay in the field.
        assert!(t.key("ArrowLeft", false, false, false, false));
        assert_eq!(t.focused_id(), Some(1));
        t.type_text("X");
        assert!(t.key("Backspace", false, false, false, false));
        assert!(t.key("Backspace", false, false, false, false));
        assert!(t.key("Delete", false, false, false, false));
        assert_eq!(
            changes(&mut t),
            ["abXe\u{301}", "abe\u{301}", "ae\u{301}", "a"]
        );
        // The caret is after "a": its advance in (at the root's 16 px: the
        // test's resolver gives no text size), on a whole pixel.
        let advance = |text: &str| {
            let run = crate::arena::Run::sized(16.0);
            TEXT.with_borrow_mut(|(fonts, _)| {
                crate::text::Texts::default()
                    .measure(fonts, 0, text, &run, crate::arena::Avail::Max)
                    .0
                    .round()
            })
        };
        let caret = |t: &mut Tree| {
            frame(t);
            let x0 = t.op_box(1).unwrap()[0];
            t.drawn
                .quads
                .iter()
                .find(|q| q.kind == og_draw::paint::PaintKind::Solid as u32 && q.rect[2] <= 2.0)
                .map(|q| q.rect[0] - x0)
        };
        assert_eq!(caret(&mut t), Some(advance("a")));
        t.key("Home", false, false, false, false);
        assert_eq!(caret(&mut t), Some(0.0));
        let mut b = Batch::default();
        b.op(6).u32(1).str("value").str("Zed");
        t.apply(&b.bytes()).unwrap();
        assert_eq!(
            caret(&mut t),
            Some(advance("Zed")),
            "a new value puts the caret at its end"
        );
        assert!(
            changes(&mut t).is_empty(),
            "a value set from outside is no change"
        );
        t.key("Tab", false, false, false, false);
        assert_eq!(t.focused_id(), Some(2));
        assert_eq!(caret(&mut t), None, "no caret without the focus");
        assert!(!t.type_text("x"));
    }

    /// A control sounds the pack's sound for its slot (`@Component.slot`
    /// in its class; a variant falls back to its bare slot), else the
    /// pack's generic one: hover as the pointer comes over it, press on a
    /// click, `confirm` when that action presses it.
    #[test]
    fn controls_sound_their_slots_sounds_else_the_packs() {
        let cue = |h: u64| Cue {
            clips: vec![h],
            volume: [0.5, 0.5],
            pitch: [1.0, 1.0],
        };
        let mut table = SoundTable::new();
        table.insert(
            String::new(),
            [("press", 10), ("hover", 30), ("confirm", 40)]
                .map(|(e, h)| (e.to_string(), cue(h)))
                .into(),
        );
        table.insert(
            "Button.plate".into(),
            [("press".to_string(), cue(20))].into(),
        );
        let mut t = stated();
        t.set_sounds(std::rc::Rc::new(table));
        buttons(&mut t, &["b @Button.plate.gold", "b @Chip.frame", "b"]);
        let heard = |t: &mut Tree| -> Vec<u64> {
            let p = t.take_sounds();
            assert!(p.iter().all(|p| p.volume == 0.5 && p.pitch == 1.0));
            p.iter().map(|p| p.sound).collect()
        };
        at(&mut t, 25.0, 5.0, false, false);
        assert_eq!(heard(&mut t), [30], "hover: the pack's");
        at(&mut t, 25.0, 5.0, true, false);
        at(&mut t, 25.0, 5.0, false, true);
        assert_eq!(heard(&mut t), [20], "the variant's slot has a press");
        at(&mut t, 75.0, 5.0, false, false);
        at(&mut t, 75.0, 5.0, true, false);
        at(&mut t, 75.0, 5.0, false, true);
        assert_eq!(
            heard(&mut t),
            [30, 10],
            "a slot without sounds plays the pack's"
        );
        t.input_done(false);
        t.key("Tab", false, false, false, false);
        let f = t.focused_id().unwrap();
        t.input_done(false);
        heard(&mut t);
        assert!(t.action("confirm"));
        assert_eq!(heard(&mut t), [40], "node {f} pressed by confirm");
    }

    /// `sound` names the pack event a node sounds when it first paints:
    /// its slot's, else the pack's; once, and not again while it stays.
    /// A node the pack has no such sound for is silent.
    #[test]
    fn a_node_sounds_its_sound_when_it_first_paints() {
        let cue = |h: u64| Cue {
            clips: vec![h],
            volume: [0.5, 0.5],
            pitch: [1.0, 1.0],
        };
        let mut table = SoundTable::new();
        table.insert(String::new(), [("reward".to_string(), cue(50))].into());
        table.insert(
            "Banner.frame".into(),
            [("reward".to_string(), cue(60))].into(),
        );
        let mut t = stated();
        t.set_sounds(std::rc::Rc::new(table));
        buttons(&mut t, &["b @Banner.frame", "b", "b"]);
        let heard =
            |t: &mut Tree| -> Vec<u64> { t.take_sounds().iter().map(|p| p.sound).collect() };
        assert_eq!(heard(&mut t), [0u64; 0], "no node asks for a sound");
        let mut b = Batch::default();
        b.op(6).u32(1).str("sound").str("reward");
        b.op(6).u32(2).str("sound").str("reward");
        b.op(6).u32(3).str("sound").str("error");
        t.apply(&b.bytes()).unwrap();
        assert_eq!(heard(&mut t), [0u64; 0], "nothing sounds before it paints");
        frame(&mut t);
        assert_eq!(heard(&mut t), [60, 50], "the slot's, then the pack's");
        frame(&mut t);
        at(&mut t, 25.0, 5.0, false, false);
        assert_eq!(heard(&mut t), [0u64; 0], "once");
    }

    /// The arrows move among a focus group's members first: at its edge
    /// one that stops leaves for the control beyond, one that wraps goes
    /// round inside itself, and focus coming into one that remembers lands
    /// on the member it last had.
    #[test]
    fn focus_groups_stop_wrap_and_remember() {
        use og_input::frame::Nav;
        // `dir`: the root's direction; 10 is a row group of 2 and 3, then 4.
        let build = |dir: &str, group: &str, memory: bool| {
            let mut t = stated();
            let mut b = Batch::default();
            b.op(1).u32(100).u8(0);
            b.op(4).u32(100).str(dir);
            b.op(8).u32(0).u32(100).u32(u32::MAX);
            b.op(1).u32(10).u8(0);
            b.op(4).u32(10).str("row");
            b.op(6).u32(10).str("focusGroup").str(group);
            if memory {
                b.op(6).u32(10).str("focusMemory").str("true");
            }
            b.op(8).u32(100).u32(10).u32(u32::MAX);
            for (id, parent) in [(2, 10), (3, 10), (4, 100)] {
                b.op(1).u32(id).u8(0);
                b.op(4).u32(id).str("b");
                b.op(7).u32(id).u8(Event::Press as u8).u8(1);
                b.op(8).u32(parent).u32(id).u32(u32::MAX);
            }
            t.apply(&b.bytes()).unwrap();
            frame(&mut t);
            t.nav(Nav::Right);
            assert_eq!(
                t.focused_id(),
                Some(2),
                "the first move lands on the first stop"
            );
            t
        };
        let mut t = build("row", "stop", false);
        t.nav(Nav::Right);
        assert_eq!(t.focused_id(), Some(3));
        t.nav(Nav::Right);
        assert_eq!(
            t.focused_id(),
            Some(4),
            "a stopping group is left at its edge"
        );
        t.nav(Nav::Right);
        assert_eq!(t.focused_id(), Some(2), "outside any group the move wraps");
        t.nav(Nav::Left);
        assert_eq!(
            t.focused_id(),
            Some(2),
            "a group's edge with nothing beyond stays"
        );
        let mut t = build("row", "wrap", false);
        t.nav(Nav::Right);
        t.nav(Nav::Right);
        assert_eq!(
            t.focused_id(),
            Some(2),
            "a wrapping group goes round inside"
        );
        // 4 is under 2; from 4 Up lands on 2, or on 3 when the group
        // remembers it had 3.
        for (memory, lands) in [(false, 2), (true, 3)] {
            let mut t = build("col", "stop", memory);
            t.nav(Nav::Right);
            t.nav(Nav::Down);
            assert_eq!(t.focused_id(), Some(4));
            t.nav(Nav::Up);
            assert_eq!(t.focused_id(), Some(lands), "memory {memory}");
        }
    }

    /// A `draggable` pressed and moved past the slop is picked up (its
    /// press then clicks nothing), reports the `dropTarget` under it and,
    /// let go there, drops on it; the keys do the same: Enter picks the
    /// focused one up, Tab reaches the targets, Enter puts it down, Escape
    /// puts it back.
    #[test]
    fn a_draggable_drops_on_a_target_by_pointer_and_by_keys() {
        let mut t = stated();
        let mut b = Batch::default();
        b.op(1).u32(100).u8(0);
        b.op(4).u32(100).str("row");
        b.op(8).u32(0).u32(100).u32(u32::MAX);
        for id in [1, 2, 3] {
            b.op(1).u32(id).u8(0);
            b.op(4).u32(id).str("b");
            // Nothing takes presses yet: a `draggable` drags without
            // one, and the targets are focus stops only while it is carried.
            for ev in [
                Event::DragStart,
                Event::DragMove,
                Event::DragEnd,
                Event::Drop,
            ] {
                b.op(7).u32(id).u8(ev as u8).u8(1);
            }
            b.op(8).u32(100).u32(id).u32(u32::MAX);
        }
        b.op(6).u32(1).str("draggable").str("sword");
        b.op(6).u32(2).str("dropTarget").str("slot-a");
        b.op(6).u32(3).str("dropTarget").str("slot-b");
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        // Ids 1, 2, 3 are 50 wide from x 0.
        let seen = |t: &mut Tree| -> Vec<(u32, Event, String, String)> {
            let out = t
                .events
                .iter()
                .filter(|f| f.event != Event::Focus)
                .map(|f| (f.id, f.event, f.key.to_string(), f.value.to_string()))
                .collect();
            t.events.clear();
            out
        };
        let ev = |id, e, k: &str, v: &str| (id, e, k.to_string(), v.to_string());
        at(&mut t, 10.0, 5.0, true, false);
        at(&mut t, 12.0, 5.0, false, false);
        assert!(seen(&mut t).is_empty(), "within the slop nothing drags");
        at(&mut t, 125.0, 5.0, false, false);
        at(&mut t, 125.0, 5.0, false, true);
        assert_eq!(
            seen(&mut t),
            [
                ev(1, Event::DragStart, "sword", "0 0 50 20"),
                ev(1, Event::DragMove, "sword", "slot-b"),
                ev(3, Event::Drop, "sword", "100 0 50 20"),
                ev(1, Event::DragEnd, "sword", "slot-b"),
            ],
            "no press on release"
        );
        // On an item that answers a press, a click without a move still
        // presses.
        let mut b = Batch::default();
        b.op(7).u32(1).u8(Event::Press as u8).u8(1);
        t.apply(&b.bytes()).unwrap();
        at(&mut t, 10.0, 5.0, true, false);
        at(&mut t, 10.0, 5.0, false, true);
        assert_eq!(seen(&mut t), [ev(1, Event::Press, "", "")]);
        // The keys: 1 has the focus from the click.
        t.input_done(false);
        assert!(t.key("Enter", false, false, false, false));
        t.key("Tab", false, false, false, false);
        assert_eq!(t.focused_id(), Some(2), "a target is a stop while carrying");
        assert!(t.key("Enter", false, false, false, false));
        assert_eq!(
            seen(&mut t),
            [
                ev(1, Event::DragStart, "sword", "0 0 50 20"),
                ev(1, Event::DragMove, "sword", "slot-a"),
                ev(2, Event::Drop, "sword", "50 0 50 20"),
                ev(1, Event::DragEnd, "sword", "slot-a"),
            ]
        );
        assert_eq!(t.focused_id(), Some(1), "focus goes back to the node");
        t.input_done(false);
        assert!(t.key("Enter", false, false, false, false));
        assert!(t.key("Escape", false, false, false, false));
        let s = seen(&mut t);
        assert_eq!(s.last(), Some(&ev(1, Event::DragEnd, "sword", "")), "{s:?}");
        t.input_done(false);
        assert!(t.action("confirm"), "confirm picks up too");
        assert!(t.action("back"));
        assert_eq!(seen(&mut t).len(), 2);
    }

    /// An on-screen control's box is a zone of the action it names, in
    /// surface px; one disabled or no longer a control leaves, and a frame
    /// where nothing changed hands out nothing.
    #[test]
    fn on_screen_controls_are_zones_of_their_actions() {
        use og_input::ZoneKind;
        let mut t = stated();
        let mut b = Batch::default();
        b.op(1).u32(100).u8(0);
        b.op(4).u32(100).str("row");
        b.op(8).u32(0).u32(100).u32(u32::MAX);
        for id in [1, 2] {
            b.op(1).u32(id).u8(0);
            b.op(4).u32(id).str("b fw");
            b.op(8).u32(100).u32(id).u32(u32::MAX);
        }
        b.op(6).u32(1).str("touchStick").str("move");
        b.op(6).u32(1).str("deadZone").str("0.2");
        b.op(6).u32(2).str("touchButton").str("jump");
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let zones = |t: &mut Tree| {
            t.take_zones().map(|z| {
                z.iter()
                    .map(|z| (z.action.clone(), z.kind, [z.x, z.width], z.dead_zone))
                    .collect::<Vec<_>>()
            })
        };
        let (x1, w1) = (t.op_box(1).unwrap()[0], t.op_box(1).unwrap()[2]);
        let (x2, w2) = (t.op_box(2).unwrap()[0], t.op_box(2).unwrap()[2]);
        assert!(x2 > x1, "the button comes after the stick");
        assert_eq!(
            zones(&mut t),
            Some(vec![
                ("move".into(), ZoneKind::Stick, [x1, w1], 0.2),
                ("jump".into(), ZoneKind::Button, [x2, w2], 0.0),
            ])
        );
        frame(&mut t);
        assert_eq!(zones(&mut t), None, "nothing changed");
        let mut b = Batch::default();
        b.op(6).u32(1).str("disabled").str("true");
        b.op(6).u32(2).str("touchButton").str("");
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        assert_eq!(zones(&mut t), Some(vec![]));
    }

    /// What og-input read from a control comes back: the control is
    /// `active:` while a pointer holds it, and a `knob` node inside moves
    /// by its knob, in tree units (the zones went out in surface px).
    #[test]
    fn a_control_shows_the_knob_and_press_og_input_read() {
        let mut t = tree();
        let mut b = Batch::default();
        for (id, class, parent) in [(1, "big", 0), (2, "b", 1)] {
            b.op(1).u32(id).u8(0);
            b.op(4).u32(id).str(class);
            b.op(8).u32(parent).u32(id).u32(u32::MAX);
        }
        b.op(6).u32(1).str("touchStick").str("move");
        b.op(6).u32(2).str("knob").str("true");
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        assert!(t.take_zones().is_some());
        let held = og_input::ZoneState {
            pointer: Some(2),
            knob: og_core::glam::Vec2::new(12.0, -4.0),
            ..Default::default()
        };
        t.zone_states([&held], 2.0);
        frame(&mut t);
        let knob = t.op(2).unwrap() as usize;
        let xf = t.xfs[knob];
        assert!(xf != 0, "the knob draws moved");
        let [_, _, _, _, e, f, _, _] = t.drawn.transforms[xf as usize];
        assert_eq!([e, f], [6.0, -2.0], "the knob, in tree units");
        let one = t.op(1).unwrap();
        assert!(t.input.per(one).unwrap().states & PRESSED != 0, "held");
        // Pushed to the ring's edge (150 px of tree), the knob (20 px
        // tall: 10 of radius) stops 140 from the centre, inside the ring.
        let edge = og_input::ZoneState {
            pointer: Some(2),
            knob: og_core::glam::Vec2::new(0.0, 300.0),
            ..Default::default()
        };
        t.zone_states([&edge], 2.0);
        frame(&mut t);
        let [_, _, _, _, e, f, _, _] = t.drawn.transforms[t.xfs[knob] as usize];
        assert_eq!([e, f], [0.0, 140.0], "the knob inside the ring");
        t.zone_states([&og_input::ZoneState::default()], 2.0);
        frame(&mut t);
        assert!(t.input.per(one).unwrap().states & PRESSED == 0, "let go");
        assert_eq!(t.xfs[knob], 0, "the knob back home");
    }

    /// A held d-pad carries the way og-input reads from it as
    /// `data-dir-<way>` (y up: two on a diagonal, none in its dead
    /// middle), a stick carries none, and a pad that stops being a zone
    /// while held reads as let go.
    #[test]
    fn a_held_pad_carries_the_way_it_reads() {
        let mut t = tree();
        let mut b = Batch::default();
        for id in [1, 2] {
            b.op(1).u32(id).u8(0);
            b.op(4).u32(id).str("b");
            b.op(8).u32(0).u32(id).u32(u32::MAX);
        }
        b.op(6).u32(1).str("touchPad").str("move");
        b.op(6).u32(2).str("touchStick").str("look");
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        assert!(t.take_zones().is_some());
        let held = |x: f32, y: f32| og_input::ZoneState {
            pointer: Some(0),
            value: og_core::glam::Vec2::new(x, y),
            ..Default::default()
        };
        let dirs = |t: &Tree, id: u32| -> Vec<String> {
            let e = t.op(id).unwrap() as usize;
            t.els[e].attrs.iter().map(|a| a.to_string()).collect()
        };
        t.zone_states([&held(-1.0, 1.0), &held(1.0, -1.0)], 1.0);
        assert_eq!(dirs(&t, 1), ["dir-left", "dir-up"], "up and to the left");
        assert_eq!(dirs(&t, 2), [""; 0], "a stick has no way");
        t.zone_states([&held(0.0, -1.0), &held(0.0, 0.0)], 1.0);
        assert_eq!(dirs(&t, 1), ["dir-down"]);
        t.zone_states([&held(0.0, 0.0), &held(0.0, 0.0)], 1.0);
        assert_eq!(dirs(&t, 1), [""; 0], "held in its dead middle");
        t.zone_states([&held(1.0, 0.0), &held(0.0, 0.0)], 1.0);
        assert_eq!(dirs(&t, 1), ["dir-right"]);
        // Disabled while held: og-input drops the zone and says no more.
        let mut b = Batch::default();
        b.op(6).u32(1).str("disabled").str("true");
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        assert_eq!(t.take_zones().map(|z| z.len()), Some(1));
        assert_eq!(dirs(&t, 1), [""; 0], "let go with its zone");
        let one = t.op(1).unwrap();
        assert!(t.input.per(one).unwrap().states & PRESSED == 0);
    }

    /// Two fingers down on a node that hears `onPinch` and `onRotate`
    /// pinch and turn it (NSH's recogniser): start, move with the scale
    /// and the turn from where they began, end. A finger let go fast on
    /// one that hears `onSwipe` swipes it its way; a slow one does not.
    #[test]
    fn fingers_pinch_turn_and_swipe_the_node_they_went_down_on() {
        let mut t = tree();
        let mut b = Batch::default();
        b.op(1).u32(1).u8(0);
        b.op(4).u32(1).str("big");
        b.op(8).u32(0).u32(1).u32(u32::MAX);
        b.op(7).u32(1).u8(Event::Pinch as u8).u8(1);
        b.op(7).u32(1).u8(Event::Rotate as u8).u8(1);
        b.op(7).u32(1).u8(Event::Swipe as u8).u8(1);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let mut p = og_input::Pointers::default();
        let step = |t: &mut Tree, p: &mut og_input::Pointers, at: f64| {
            t.set_time(at, false);
            t.touches(p);
            p.pressed = 0;
            p.released = 0;
            std::mem::take(&mut t.events)
                .into_iter()
                .map(|f| (f.event, f.key.to_string(), f.value.to_string()))
                .collect::<Vec<_>>()
        };
        // Fingers 1 and 2, 100 px apart, then 200 px apart and turned.
        p.held = 0b110;
        p.pressed = 0b110;
        p.position1 = og_core::glam::Vec2::new(100.0, 100.0);
        p.position2 = og_core::glam::Vec2::new(200.0, 100.0);
        let got = step(&mut t, &mut p, 0.0);
        assert_eq!(got[0], (Event::Pinch, "start".into(), "1".into()));
        p.position2 = og_core::glam::Vec2::new(100.0, 300.0);
        let got = step(&mut t, &mut p, 0.1);
        assert_eq!(got[0], (Event::Pinch, "move".into(), "2".into()));
        let turn: f32 = got[1].2.parse().unwrap();
        assert_eq!(got[1].1, "move");
        assert!((turn - std::f32::consts::FRAC_PI_2).abs() < 1e-5, "{turn}");
        p.held = 0b100;
        p.released = 0b010;
        let got = step(&mut t, &mut p, 0.2);
        assert_eq!(
            got.iter().map(|g| g.1.as_str()).collect::<Vec<_>>(),
            ["end", "end"]
        );
        p.held = 0;
        p.released = 0b100;
        assert!(
            step(&mut t, &mut p, 0.3).is_empty(),
            "a pinch's finger swipes nothing"
        );
        // Finger 3: 120 px left in 0.1 s swipes left; 20 px in 1 s does not.
        for (dx, dt, want) in [(-120.0, 0.1, Some("left")), (20.0, 1.0, None)] {
            p.held = 0b1000;
            p.pressed = 0b1000;
            p.position3 = og_core::glam::Vec2::new(200.0, 150.0);
            step(&mut t, &mut p, 1.0);
            p.position3.x += dx / 2.0;
            step(&mut t, &mut p, 1.0 + dt / 2.0);
            p.position3.x += dx / 2.0;
            step(&mut t, &mut p, 1.0 + dt);
            p.held = 0;
            p.released = 0b1000;
            let got = step(&mut t, &mut p, 1.0 + dt);
            assert_eq!(
                got.first().map(|g| (g.0, g.1.as_str())),
                want.map(|w| (Event::Swipe, w))
            );
        }
    }

    /// A character no face of the text's family draws is reported once,
    /// for the host to load a font pack; once a face that has it is among
    /// the collection's fallbacks, with nothing in the tree changed, the
    /// text is measured and drawn again with it.
    #[test]
    fn a_character_the_family_lacks_draws_from_a_fallback_face_once_it_loads() {
        let mut fonts = crate::text::tests::latin();
        let mut atlas = og_draw::glyphs::Atlas::default();
        let mut t = tree();
        let mut b = Batch::default();
        b.op(2).u32(1).str("A中");
        b.op(8).u32(0).u32(1).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        let glyphs = |t: &Tree| -> usize {
            let raster = og_draw::paint::PaintKind::Raster as u32;
            t.drawn.quads.iter().filter(|q| q.kind == raster).count()
        };
        t.frame(&mut fonts, &mut atlas, &IconAtlas::default());
        assert_eq!(glyphs(&t), 1);
        let narrow = t.op_box(1).unwrap()[2];
        assert_eq!(t.take_missing(), ['中']);
        let frame = t.frame(&mut fonts, &mut atlas, &IconAtlas::default());
        assert!(!frame.painted && t.take_missing().is_empty(), "once");
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../templates/styles/cjk/NotoSansSC-Regular.ttf"
        );
        let cjk = fonts
            .add(std::sync::Arc::new(std::fs::read(path).unwrap()))
            .unwrap();
        fonts.set_fallback(None, &[og_text::fonts::FaceId(0), cjk]);
        t.frame(&mut fonts, &mut atlas, &IconAtlas::default());
        assert_eq!(glyphs(&t), 2);
        assert!(t.op_box(1).unwrap()[2] > narrow, "measured again");
        assert!(t.take_missing().is_empty());
    }

    /// A tree whose classes resolve with NSH's default theme, as a game's do.
    fn real() -> Tree {
        let mut t = tree();
        let classes = std::cell::RefCell::new(crate::classes::Classes::new(None).unwrap().0);
        t.set_resolver(Box::new(move |class, _, _, _| {
            classes
                .borrow_mut()
                .record(class, &crate::classes::Env::default())
        }));
        t
    }

    /// Op nodes `(id, tag, class, parent)` in order (`tag` as op 1's).
    fn nodes(t: &mut Tree, list: &[(u32, u8, &str, u32)]) {
        let mut b = Batch::default();
        for &(id, tag, class, parent) in list {
            b.op(1).u32(id).u8(tag);
            b.op(4).u32(id).str(class);
            b.op(8).u32(parent).u32(id).u32(u32::MAX);
        }
        t.apply(&b.bytes()).unwrap();
    }

    #[test]
    fn em_sizes_use_the_parent_once_and_remeasure_when_it_changes() {
        let mut t = real();
        nodes(
            &mut t,
            &[
                (1, 0, "text-[32px] items-start", 0),
                (2, 0, "text-[0.5em] items-start", 1),
                (3, 0, "items-start", 2),
            ],
        );
        let mut b = Batch::default();
        b.op(2).u32(4).str("MMMM");
        b.op(8).u32(3).u32(4).u32(u32::MAX);
        b.op(2).u32(5).str("MMMM");
        b.op(4).u32(5).str("text-[16px]");
        b.op(8).u32(0).u32(5).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        assert_eq!(t.op_box(4).unwrap()[2], t.op_box(5).unwrap()[2]);
        let before = t.op_box(4).unwrap()[2];
        let mut b = Batch::default();
        b.op(4).u32(1).str("text-[48px] items-start");
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        assert!((t.op_box(4).unwrap()[2] / before - 1.5).abs() < 0.01);
    }

    #[test]
    fn decoration_styles_and_offsets_reach_the_painted_text() {
        let mut t = real();
        nodes(
            &mut t,
            &[(
                1,
                0,
                "text-white underline decoration-2 underline-offset-4",
                0,
            )],
        );
        let mut b = Batch::default();
        b.op(2).u32(2).str("Read");
        b.op(8).u32(1).u32(2).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let lines = |t: &Tree| {
            t.drawn
                .quads
                .iter()
                .filter(|q| q.kind == PaintKind::Solid as u32)
                .map(|q| q.rect)
                .collect::<Vec<_>>()
        };
        let solid = lines(&t);
        assert_eq!(solid.len(), 1);
        assert!(
            t.drawn.quads.iter().all(|q| q.color == [1.; 4]),
            "decorations use inherited currentColor"
        );
        for (style, count) in [("double", 2), ("dotted", 0), ("dashed", 0), ("wavy", 0)] {
            let mut b = Batch::default();
            b.op(4).u32(1).str(&format!(
                "text-white underline decoration-2 underline-offset-8 decoration-{style}"
            ));
            t.apply(&b.bytes()).unwrap();
            frame(&mut t);
            if style == "wavy" {
                assert_eq!(
                    t.drawn
                        .quads
                        .iter()
                        .filter(|q| q.kind == PaintKind::Line as u32)
                        .count(),
                    1
                );
                assert!(t.drawn.points.iter().any(|a| a[1] != t.drawn.points[0][1]));
            } else {
                let current = lines(&t);
                if count != 0 {
                    assert_eq!(current.len(), count);
                } else {
                    assert!(current.len() > 2);
                }
                assert!(current.iter().all(|r| r[1] > solid[0][1]));
            }
        }
    }

    #[test]
    fn input_colors_and_grapheme_selections_are_drawn_and_edited() {
        let mut t = real();
        nodes(
            &mut t,
            &[(
                1,
                4,
                "text-white placeholder:text-[#ff0000] caret-[#00ff00] selection:bg-[#0000ff]",
                0,
            )],
        );
        let mut b = Batch::default();
        b.op(6).u32(1).str("placeholder").str("Name");
        b.op(7).u32(1).u8(Event::Change as u8).u8(1);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let raster = PaintKind::Raster as u32;
        assert!(t.drawn.quads.iter().any(|q| q.kind == raster));
        assert!(
            t.drawn
                .quads
                .iter()
                .filter(|q| q.kind == raster)
                .all(|q| q.color == [1., 0., 0., 1.])
        );
        t.key("Tab", false, false, false, false);
        frame(&mut t);
        assert!(t.drawn.quads.iter().any(|q| q.color == [0., 1., 0., 1.]));
        t.type_text("ae\u{301}中");
        t.key("ArrowLeft", true, false, false, false);
        t.key("ArrowLeft", true, false, false, false);
        frame(&mut t);
        assert!(
            t.drawn
                .quads
                .iter()
                .any(|q| q.kind == PaintKind::Solid as u32
                    && q.color == [0., 0., 1., 1.]
                    && q.rect[2] > 0.)
        );
        t.type_text("X");
        assert_eq!(
            &*t.events.last().unwrap().value,
            "aX",
            "replace the entire accented cluster and Han"
        );
        t.key("KeyA", false, false, false, true);
        t.key("Backspace", false, false, false, false);
        assert_eq!(&*t.events.last().unwrap().value, "");
        t.type_text("ab");
        t.key("Home", false, false, false, false);
        t.key("ArrowRight", true, false, false, false);
        t.key("ArrowRight", false, false, false, false);
        t.type_text("Z");
        assert_eq!(
            &*t.events.last().unwrap().value,
            "aZb",
            "an unshifted arrow collapses selection without another step"
        );
    }

    /// `space-x-4` gives each child but the last a 16 px right margin, under
    /// a child's own margin on that side; a child added after the last
    /// gives the old last one its margin; `space-y-reverse` puts it on top.
    /// Ways this could fail: the last child spaced too, a child's own
    /// margin overwritten, or the old last one left unspaced.
    #[test]
    fn space_classes_margin_every_child_but_the_last() {
        let mut t = real();
        nodes(
            &mut t,
            &[
                (1, 0, "flex flex-row space-x-4", 0),
                (2, 0, "w-[10px] h-[10px] mr-[2px]", 1),
                (3, 0, "w-[10px] h-[10px]", 1),
                (4, 0, "w-[10px] h-[10px]", 1),
            ],
        );
        frame(&mut t);
        let xs = |t: &Tree, ids: &[u32]| -> Vec<f32> {
            ids.iter().map(|&i| t.op_box(i).unwrap()[0]).collect()
        };
        assert_eq!(xs(&t, &[2, 3, 4]), [0.0, 12.0, 38.0]);
        assert_eq!(t.op_box(1).unwrap()[2], 48.0, "the last unspaced");
        nodes(&mut t, &[(5, 0, "w-[10px] h-[10px]", 1)]);
        frame(&mut t);
        assert_eq!(xs(&t, &[2, 3, 4, 5]), [0.0, 12.0, 38.0, 64.0]);
        nodes(
            &mut t,
            &[
                (6, 0, "flex flex-col space-y-2 space-y-reverse", 0),
                (7, 0, "w-[10px] h-[10px]", 6),
                (8, 0, "w-[10px] h-[10px]", 6),
            ],
        );
        frame(&mut t);
        let top = t.op_box(6).unwrap()[1];
        let ys: Vec<f32> = [7, 8]
            .iter()
            .map(|&i| t.op_box(i).unwrap()[1] - top)
            .collect();
        assert_eq!(ys, [8.0, 18.0], "the top margin, on 7 only");
    }

    /// `line-clamp-2` on an element shows its text's first two lines, the
    /// second ending in "…" inside the box, and the text is two lines
    /// tall. Ways this could fail: the clamp on the element never reaching
    /// its text child, the measure still sizing every line, or the
    /// ellipsis pushed past the box.
    #[test]
    fn line_clamp_cuts_the_text_to_its_lines_with_an_ellipsis() {
        let mut t = real();
        nodes(
            &mut t,
            &[(1, 0, "w-[60px] text-[10px] leading-[10px] line-clamp-2", 0)],
        );
        let mut b = Batch::default();
        b.op(2).u32(2).str("aaaa bbbb cccc dddd eeee ffff gggg");
        b.op(8).u32(1).u32(2).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let e = t.by_js[2] as usize;
        let leaf = t.texts.leaf(t.els[e].leaf.unwrap()).unwrap();
        let lines: Vec<&str> = leaf.line_text.lines().collect();
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert!(lines[0].starts_with("aaaa"), "{lines:?}");
        assert!(lines[1].ends_with('\u{2026}'), "{lines:?}");
        assert_eq!(t.op_box(2).unwrap()[3], 20.0, "two lines tall");
        for l in &leaf.lines {
            assert!(l[0] + l[2] <= 60.5, "{l:?}");
        }
    }

    /// A text shadow and outline inherit, as CSS's do: a `<Text>`'s classes
    /// sit on the element and its glyphs on its text child. A child's own
    /// outline (here none) wins over its parent's.
    #[test]
    fn text_shadow_and_outline_reach_the_text_inside_the_element() {
        let mut t = real();
        nodes(
            &mut t,
            &[(
                1,
                0,
                "text-white text-shadow-md text-stroke-2 text-stroke-black",
                0,
            )],
        );
        let mut b = Batch::default();
        b.op(2).u32(2).str("ab");
        b.op(8).u32(1).u32(2).u32(u32::MAX);
        b.op(2).u32(3).str("c");
        b.op(4).u32(3).str("text-stroke-0");
        b.op(8).u32(1).u32(3).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let black = [0.0, 0.0, 0.0, 1.0];
        let glyphs: Vec<&PaintQuad> = t
            .drawn
            .quads
            .iter()
            .filter(|q| q.kind == PaintKind::Raster as u32)
            .collect();
        // "ab": its glyphs' shadows, their outlines (the band the stroke's
        // 2 px wide, a pixel past the fill each side), then their fills;
        // "c" has the shadow and no outline.
        let [sa, sb, oa, ob, a, b, sc, c] = glyphs[..] else {
            panic!("{} glyph quads", glyphs.len());
        };
        for (shadow, outline, fill) in [(sa, Some(oa), a), (sb, Some(ob), b), (sc, None, c)] {
            assert!(shadow.rect[2] > fill.rect[2] + 2.0, "a blurred shadow");
            assert!(shadow.rect[1] > fill.rect[1] - 0.5 * (shadow.rect[3] - fill.rect[3]));
            assert_ne!(fill.color, black);
            if let Some(o) = outline {
                assert_eq!(o.color, black, "the outline is black");
                assert_eq!(o.rect[2], fill.rect[2] + 2.0);
                assert_eq!(o.rect[0], fill.rect[0] - 1.0);
            }
        }
    }

    /// `truncate` on an element ends its text in "…" at the element's
    /// content box and lets the element shrink below its text; `break-words`
    /// breaks a word wider than its box at the box.
    #[test]
    fn truncate_ends_in_an_ellipsis_and_break_words_breaks_a_long_word() {
        let mut t = real();
        nodes(
            &mut t,
            &[
                (1, 0, "flex flex-row w-[120px]", 0),
                (2, 0, "truncate px-[4px]", 1),
                (8, 0, "flex flex-col", 0),
                (4, 0, "w-[60px] break-words", 8),
                (6, 0, "w-[60px]", 8),
            ],
        );
        let mut b = Batch::default();
        for (id, parent, text) in [
            (3, 2, "The Warden of the Far Northern Observatory"),
            (5, 4, "Supercalifragilistic"),
            (7, 6, "Supercalifragilistic"),
        ] {
            b.op(2).u32(id).str(text);
            b.op(8).u32(parent).u32(id).u32(u32::MAX);
        }
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        fn leaf(t: &Tree, id: u32) -> &crate::text::Leaf {
            let e = t.by_js[id as usize] as usize;
            t.texts.leaf(t.els[e].leaf.unwrap()).unwrap()
        }
        assert_eq!(t.op_box(2).unwrap()[2], 120.0, "it shrinks to its row");
        let name = leaf(&t, 3);
        assert!(
            name.line_text.ends_with("\u{2026}\n"),
            "{:?}",
            name.line_text
        );
        assert!(
            name.lines[0][2] <= 112.0,
            "inside the padding: {:?}",
            name.lines
        );
        assert!(leaf(&t, 5).lines.len() > 1, "broken at the box");
        assert!(leaf(&t, 5).lines.iter().all(|l| l[2] <= 60.0));
        assert_eq!(leaf(&t, 7).lines.len(), 1, "whole, past its box");
        assert!(t.op_box(4).unwrap()[3] > t.op_box(6).unwrap()[3]);
    }

    #[test]
    fn hyphens_inherit_and_reflow_after_an_override() {
        let mut t = real();
        nodes(
            &mut t,
            &[(1, 0, "w-[40px] hyphens-manual", 0), (2, 0, "", 1)],
        );
        let mut b = Batch::default();
        b.op(2).u32(3).str("hy\u{ad}phenation");
        b.op(8).u32(2).u32(3).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let lines = |t: &Tree| {
            let e = t.by_js[3] as usize;
            t.texts.leaf(t.els[e].leaf.unwrap()).unwrap().lines.len()
        };
        assert!(lines(&t) > 1, "the text inherits manual breaks");
        let mut b = Batch::default();
        b.op(4).u32(2).str("hyphens-none");
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        assert_eq!(lines(&t), 1, "a child override reshapes and remeasures");
        let mut b = Batch::default();
        b.op(4).u32(2).str("");
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        assert!(lines(&t) > 1, "removing the override restores inheritance");
    }

    /// A rounded `overflow-hidden` box clips everything its subtree paints
    /// to its rounded padding box (its inner radii, its radius less its
    /// border), as CSS clips: every quad of a child and a grandchild names
    /// that round, whatever its kind, while the box's own quads and a
    /// sibling's name none. Ways this could fail: the outer radius used
    /// inside a border, the round given to the clipping box itself, a
    /// grandchild left square, or a child's own rect and radii changed
    /// instead (the old corner rounding).
    #[test]
    fn a_rounded_clip_names_its_round_on_everything_under_it() {
        let mut t = real();
        nodes(
            &mut t,
            &[
                (
                    1,
                    0,
                    "relative w-[40px] h-[40px] shrink-0 bg-black rounded-[20px] border-2 overflow-hidden",
                    0,
                ),
                (2, 0, "absolute inset-0 bg-white", 1),
                (3, 0, "absolute top-[10px] left-[10px] w-[10px] h-[10px]", 1),
                (4, 0, "w-[4px] h-[4px] bg-black", 3),
                (5, 0, "w-[10px] h-[10px] shrink-0 bg-white", 0),
            ],
        );
        frame(&mut t);
        let d = &t.drawn;
        let solid = |id: u32| {
            let r = t.op_box(id).unwrap();
            *d.quads
                .iter()
                .find(|q| q.kind == PaintKind::Solid as u32 && q.rect == r)
                .unwrap()
        };
        assert_eq!(d.rounds.len(), 1, "{:?}", d.rounds);
        let round = d.rounds[0];
        assert_eq!(round.rect, [2.0, 2.0, 38.0, 38.0], "the padding box");
        assert_eq!(round.radius, [18.0; 4], "the inner radii");
        assert_eq!(round.invert, 0);
        assert_eq!(solid(1).round, 0, "the clipping box itself");
        let child = solid(2);
        assert_eq!((child.round, child.radius), (1, [0.0; 4]));
        assert_eq!(child.rect, [2.0, 2.0, 36.0, 36.0], "its own box");
        assert_eq!(solid(4).round, 1, "a grandchild");
        assert_eq!(solid(5).round, 0, "a sibling");
    }

    /// `drop-shadow-*` and `blur-*` draw an element and its subtree
    /// through a group: its composite quad first, then what it holds, the
    /// group listed when its subtree ends, so a nested one comes before
    /// its parent and its range lies inside its parent's. The group's
    /// opacity and mask go to its composite, and what it holds draws at
    /// full alpha and unmasked. A backdrop is a group of no quads under
    /// its element's own. Ways this could fail: a group's range ending at
    /// its own quads (its children outside the layer), groups listed
    /// parent first, the opacity applied twice (inside and on the
    /// composite), or a sibling drawn into the layer.
    #[test]
    fn a_filter_draws_its_subtree_through_a_group_and_a_backdrop_under_it() {
        let mut t = real();
        nodes(
            &mut t,
            &[
                (
                    1,
                    0,
                    "w-[100px] h-[100px] shrink-0 bg-white opacity-50 \
                     mask-[linear-gradient(black,transparent)] drop-shadow-[0_4px_6px_#000]",
                    0,
                ),
                (2, 0, "w-[40px] h-[40px] bg-black blur-[3px]", 1),
                (3, 0, "w-[10px] h-[10px] bg-white", 2),
                (4, 0, "w-[20px] h-[20px] shrink-0 bg-white", 0),
                (
                    5,
                    0,
                    "w-[30px] h-[30px] shrink-0 bg-white/20 rounded-[8px] backdrop-blur-[5px] backdrop-brightness-50",
                    0,
                ),
            ],
        );
        frame(&mut t);
        let d = &t.drawn;
        assert_eq!(d.groups.len(), 3, "{:?}", d.groups);
        let (inner, outer, backdrop) = (d.groups[0], d.groups[1], d.groups[2]);
        assert!(!inner.backdrop && !outer.backdrop && backdrop.backdrop);
        // The outer group holds the inner one, which holds its child.
        assert!(outer.quad < inner.quad && inner.end <= outer.end);
        let at = |id: u32| {
            let r = t.op_box(id).unwrap();
            d.quads
                .iter()
                .position(|q| q.kind == PaintKind::Solid as u32 && q.rect == r)
                .unwrap() as u32
        };
        assert!(
            (inner.quad + 1..inner.end).contains(&at(3)),
            "the grandchild"
        );
        assert!(
            (outer.quad + 1..outer.end).contains(&at(1)),
            "its own ground"
        );
        assert!(!(outer.quad..outer.end).contains(&at(4)), "a sibling");
        let composite = d.quads[outer.quad as usize];
        assert_eq!(composite.kind, PaintKind::Group as u32);
        assert_eq!(composite.a, 1, "the group it composites");
        assert_eq!(composite.color[3], 0.5, "its opacity");
        assert_ne!(composite.mask, 0, "its mask");
        let own = d.quads[at(1) as usize];
        assert_eq!((own.color[3], own.mask), (1.0, 0), "inside: as it is");
        // Its layer covers its box and the shadow's reach (3 deviations of
        // 3 px and the 4 px offset).
        let [x, y, w, h] = t.op_box(1).unwrap();
        let r = composite.rect;
        assert!(r[0] <= x - 9.0 && r[1] <= y - 9.0, "{r:?}");
        assert!(
            r[0] + r[2] >= x + w + 9.0 && r[1] + r[3] >= y + h + 13.0,
            "{r:?}"
        );
        assert_eq!(
            &d.effects[outer.first as usize..(outer.first + outer.count) as usize],
            &[PaintEffect::DropShadow {
                offset: [0.0, 4.0],
                sigma: 3.0,
                color: [0.0, 0.0, 0.0, 1.0]
            }]
        );
        assert_eq!(d.effects[inner.first as usize], PaintEffect::Blur(3.0));
        // The backdrop: its composite under its element's ground, its
        // blur then its colour matrix.
        let b = d.quads[backdrop.quad as usize];
        assert_eq!(
            (b.kind, b.rect),
            (PaintKind::Group as u32, t.op_box(5).unwrap())
        );
        assert_eq!(b.radius, [8.0; 4]);
        assert_eq!(backdrop.end, backdrop.quad + 1);
        assert!(at(5) > backdrop.quad);
        let fx = &d.effects[backdrop.first as usize..(backdrop.first + backdrop.count) as usize];
        assert_eq!(fx[0], PaintEffect::Blur(5.0));
        assert!(matches!(fx[1], PaintEffect::Matrix(_)));
    }

    /// A filter is a stacking context, as in CSS: a z-indexed child stays
    /// in its group, and a group and a backdrop still name their own
    /// quads after z moves a sibling painted before them past them. Ways
    /// this could fail: the child lifted out of the layer to the root's
    /// context, or the groups left naming the places their quads had
    /// before the reorder.
    #[test]
    fn a_filter_keeps_its_z_indexed_children_and_its_groups_follow_the_reorder() {
        let mut t = real();
        nodes(
            &mut t,
            &[
                (1, 0, "w-[20px] h-[20px] shrink-0 bg-white z-[1]", 0),
                (2, 0, "w-[60px] h-[60px] shrink-0 bg-black blur-[2px]", 0),
                (3, 0, "w-[10px] h-[10px] bg-white z-10", 2),
                (4, 0, "w-[10px] h-[10px] bg-white", 2),
                (
                    5,
                    0,
                    "w-[30px] h-[30px] shrink-0 bg-white/20 backdrop-blur-[4px]",
                    0,
                ),
            ],
        );
        frame(&mut t);
        let d = &t.drawn;
        let at = |id: u32| {
            let r = t.op_box(id).unwrap();
            d.quads
                .iter()
                .position(|q| q.kind == PaintKind::Solid as u32 && q.rect == r)
                .unwrap() as u32
        };
        let group = *d.groups.iter().find(|g| !g.backdrop).unwrap();
        let backdrop = *d.groups.iter().find(|g| g.backdrop).unwrap();
        assert_eq!(d.quads[group.quad as usize].kind, PaintKind::Group as u32);
        for id in [2, 3, 4] {
            assert!((group.quad + 1..group.end).contains(&at(id)), "{id}");
        }
        assert!(at(1) > at(5), "z-1 paints over the auto siblings");
        assert!(!(group.quad..group.end).contains(&at(1)));
        let b = d.quads[backdrop.quad as usize];
        assert_eq!(
            (b.kind, b.rect, backdrop.end),
            (
                PaintKind::Group as u32,
                t.op_box(5).unwrap(),
                backdrop.quad + 1
            )
        );
        assert_eq!(at(5), backdrop.quad + 1, "the backdrop right under its box");
    }

    /// Siblings with the same filter whose reach does not meet share one
    /// group (one layer, one filter run), its composite over them all;
    /// ones that meet, or that filter differently, keep their own; blurs
    /// alone meet only closer than 3.5 deviations. Ways this could fail:
    /// overlapping shadows merged (one would shadow the other), a
    /// different blur merged into a neighbour's, close blurs merged, or a
    /// merged sibling's quads left outside the group.
    #[test]
    fn apart_siblings_with_one_filter_share_a_group() {
        let mut t = real();
        let icon = "w-[20px] h-[20px] shrink-0 bg-white drop-shadow-[0_2px_2px_#000]";
        let card = "w-[40px] h-[40px] shrink-0 bg-white blur-[8px]";
        nodes(
            &mut t,
            &[
                (20, 0, "flex flex-col", 0),
                (1, 0, "flex flex-row gap-[40px]", 20),
                (2, 0, icon, 1),
                (3, 0, icon, 1),
                (4, 0, icon, 1),
                (5, 0, "w-[20px] h-[20px] shrink-0 bg-white blur-[2px]", 1),
                (6, 0, "flex flex-row gap-[2px]", 20),
                (7, 0, icon, 6),
                (8, 0, icon, 6),
                (9, 0, "flex flex-row gap-[30px]", 20),
                (10, 0, card, 9),
                (11, 0, card, 9),
                (12, 0, "flex flex-row gap-[20px]", 20),
                (13, 0, card, 12),
                (14, 0, card, 12),
                (15, 0, "flex flex-row flex-wrap w-[110px] gap-[30px]", 20),
                (16, 0, card, 15),
                (17, 0, card, 15),
                (18, 0, card, 15),
                (19, 0, card, 15),
            ],
        );
        frame(&mut t);
        let d = &t.drawn;
        let at = |id: u32| {
            let r = t.op_box(id).unwrap();
            d.quads
                .iter()
                .position(|q| q.kind == PaintKind::Solid as u32 && q.rect == r)
                .unwrap() as u32
        };
        let group_of = |id: u32| {
            let k = at(id);
            d.groups
                .iter()
                .position(|g| (g.quad + 1..g.end).contains(&k))
                .unwrap()
        };
        assert_eq!(d.groups.len(), 8, "{:?}", d.groups);
        assert_eq!(group_of(2), group_of(3));
        assert_eq!(group_of(2), group_of(4));
        assert_ne!(group_of(4), group_of(5), "another filter");
        assert_ne!(group_of(7), group_of(8), "shadows that meet");
        // A blur alone: 3.5 deviations apart (and the edge's pixel) is far
        // enough, closer is not.
        assert_eq!(group_of(10), group_of(11), "blurs 30 px apart");
        assert_ne!(group_of(13), group_of(14), "blurs 20 px apart");
        // A grid: the last is apart from each of the others, though in
        // the box around them.
        for id in [17, 18, 19] {
            assert_eq!(group_of(16), group_of(id), "{id} of the grid");
        }
        // The shared composite covers all three and their shadows' reach
        // (3 deviations of 1 px and the 2 px offset).
        let c = d.quads[d.groups[group_of(2)].quad as usize];
        let (first, last) = (t.op_box(2).unwrap(), t.op_box(4).unwrap());
        assert!(c.rect[0] <= first[0] - 5.0, "{:?}", c.rect);
        assert!(
            c.rect[0] + c.rect[2] >= last[0] + last[2] + 5.0,
            "{:?}",
            c.rect
        );
    }

    /// A box paints its outer shadows under it, then its fill and
    /// gradient, its inset shadows, its border and its outline, with CSS's
    /// radii (scaled down together when they overlap) and a corner
    /// gradient's angle. Real classes, end to end.
    #[test]
    fn a_box_paints_shadows_fill_border_and_outline_in_order() {
        let mut t = real();
        nodes(
            &mut t,
            &[(
                1,
                0,
                "w-[200px] h-[100px] bg-[#102030] bg-linear-to-tr from-black to-white \
            border border-[#ff0000] border-t-1 border-r-2 border-b-3 border-l-4 rounded-[80px] rounded-bl-none \
            shadow-[0_4px_8px_#000] inset-shadow-[0_0_2px_#fff] outline-2 outline-[#00ff00] outline-offset-3",
                0,
            )],
        );
        frame(&mut t);
        let q = &t.drawn.quads;
        use PaintKind as K;
        let kinds: Vec<u32> = q.iter().map(|q| q.kind).collect();
        assert_eq!(
            kinds,
            [
                K::Shadow,
                K::Solid,
                K::Gradient,
                K::InsetShadow,
                K::Border,
                K::Border
            ]
            .map(|k| k as u32)
        );
        // 80 + 80 on the 100px sides: all four scale by 100 / 160.
        assert_eq!(q[1].radius, [50.0, 50.0, 50.0, 0.0]);
        assert_eq!(q[0].p, [0.0, 4.0, 8.0, 0.0], "offset, blur, spread");
        assert_eq!(q[4].p, [1.0, 2.0, 3.0, 4.0], "the border's widths");
        let [x, y, w, h] = t.op_box(1).unwrap();
        assert_eq!(
            q[5].rect,
            [x - 5.0, y - 5.0, w + 10.0, h + 10.0],
            "outline 3px out, 2px wide"
        );
        assert_eq!(q[5].p, [2.0; 4]);
        assert_eq!(
            q[5].radius,
            [55.0, 55.0, 55.0, 0.0],
            "a square corner stays square"
        );
        // To the top right of a 2:1 box: perpendicular to the other diagonal.
        let angle = t.drawn.gradients[0].angle;
        assert!(
            (angle - og_core::math::atan2(1.0, 2.0)).abs() < 1e-3,
            "{angle}"
        );
    }

    /// A `mask-[<gradient>]` masks every quad of its element and its
    /// subtree, measured over its own box; a nested mask replaces it under
    /// it, and a sibling is not masked.
    #[test]
    fn a_mask_covers_its_subtree_over_its_own_box() {
        let mut t = real();
        let outer = "w-[200px] h-[100px] shrink-0 bg-white p-2 \
            mask-[linear-gradient(to_right,black,transparent)]";
        nodes(
            &mut t,
            &[
                (1, 0, outer, 0),
                (2, 0, "w-[50px] h-[20px] bg-black border border-white", 1),
                (
                    3,
                    0,
                    "w-[50px] h-[20px] bg-black mask-[radial-gradient(black,transparent)]",
                    1,
                ),
                (4, 0, "w-[30px] h-[30px] shrink-0 bg-black", 0),
            ],
        );
        frame(&mut t);
        let d = &t.drawn;
        let at = |id: u32| t.op_box(id).unwrap();
        let mask_of = |id: u32| -> Vec<u32> {
            let [x, y, w, h] = at(id);
            d.quads
                .iter()
                .filter(|q| q.rect == [x, y, w, h] && q.kind == PaintKind::Solid as u32)
                .map(|q| q.mask)
                .collect()
        };
        let outer = mask_of(1);
        assert_eq!(outer.len(), 1, "{:?}", d.quads);
        assert_ne!(outer[0], 0, "the masked box");
        assert_eq!(mask_of(2), outer, "its child takes its mask");
        // The child's border too.
        assert!(
            d.quads
                .iter()
                .filter(|q| q.kind == PaintKind::Border as u32)
                .all(|q| q.mask == outer[0])
        );
        let inner = mask_of(3);
        assert!(inner[0] != 0 && inner != outer, "a nested mask replaces it");
        assert_eq!(mask_of(4), [0], "a sibling is not masked");
        let g = &d.gradients[outer[0] as usize - 1];
        assert_eq!(g.area, at(1), "measured over the masking box");
        assert_eq!(
            d.gradients[inner[0] as usize - 1].shape,
            og_draw::paint::GRADIENT_RADIAL
        );
    }

    /// Radial and conic backgrounds paint as their own shapes: a radial
    /// ellipse through the farthest corner (the CSS default), a circle off
    /// the centre to its farthest corner, and a conic sweep from its start
    /// angle with its stops in turns.
    #[test]
    fn radial_and_conic_backgrounds_paint_their_shapes() {
        let mut t = real();
        nodes(
            &mut t,
            &[
                (
                    1,
                    0,
                    "w-[200px] h-[100px] shrink-0 bg-radial from-black to-white",
                    0,
                ),
                (
                    2,
                    0,
                    "w-[200px] h-[100px] shrink-0 bg-[radial-gradient(circle_at_25%_75%,black,white)]",
                    0,
                ),
                (
                    3,
                    0,
                    "w-[200px] h-[100px] shrink-0 -bg-conic-30 from-black to-white to-17%",
                    0,
                ),
                (
                    4,
                    0,
                    "w-[200px] h-[100px] shrink-0 bg-linear-to-r from-black via-red-500 via-50% to-white",
                    0,
                ),
            ],
        );
        frame(&mut t);
        let g = &t.drawn.gradients;
        assert_eq!(g.len(), 4, "{g:?}");
        use og_draw::paint::{GRADIENT_CONIC, GRADIENT_RADIAL};
        let near = |a: f32, b: f32| (a - b).abs() < 1e-2;
        assert_eq!(g[0].shape, GRADIENT_RADIAL);
        assert_eq!(g[0].center, [0.0, 0.0]);
        // The ellipse of the half sides' 2:1, through the corner.
        assert!(near(g[0].radii[0], 100.0 * 2f32.sqrt()), "{:?}", g[0].radii);
        assert!(near(g[0].radii[1], 50.0 * 2f32.sqrt()), "{:?}", g[0].radii);
        // At (50, 75) of 200 × 100: the farthest corner is the top right,
        // 150 across and 75 up.
        assert_eq!(g[1].shape, GRADIENT_RADIAL);
        assert_eq!(g[1].center, [-50.0, 25.0]);
        assert!(
            near(g[1].radii[0], (150f32 * 150.0 + 75.0 * 75.0).sqrt()),
            "{:?}",
            g[1].radii
        );
        assert_eq!(g[1].radii[0], g[1].radii[1], "a circle");
        assert_eq!(g[2].shape, GRADIENT_CONIC);
        assert!(near(g[2].angle, (-30f32).to_radians()), "{}", g[2].angle);
        let stops: Vec<f32> = t.drawn.stops[g[2].first as usize..][..g[2].count as usize]
            .iter()
            .map(|s| s.offset)
            .collect();
        assert!(near(*stops.last().unwrap(), 0.17), "{stops:?}");
        // More Oklab stops than a record keeps: the last still ends it.
        let last = g.len() - 1;
        let stops = &t.drawn.stops[g[last].first as usize..][..g[last].count as usize];
        assert!(stops.len() <= crate::record::GRADIENT_STOPS);
        assert!(near(stops.last().unwrap().offset, 1.0), "{stops:?}");
        assert!(stops.last().unwrap().color[0] > 0.9, "it ends white");
    }

    /// A transform moves a box and everything under it at paint, about its
    /// origin, composed down the tree; its layout stays. A text's shadow
    /// paints under its glyphs and its underline after them, and the text
    /// paints over the panel it sits on.
    #[test]
    fn a_transform_composes_down_the_tree_and_text_paints_its_shadow_and_lines() {
        let mut t = real();
        nodes(
            &mut t,
            &[
                (1, 0, "w-[100px] h-[40px] bg-[#102030] rotate-90", 0),
                (2, 0, "w-[20px] h-[10px] bg-white translate-x-[30px]", 1),
            ],
        );
        let mut b = Batch::default();
        b.op(2).u32(3).str("ab");
        b.op(4).u32(3).str("text-shadow-md underline");
        b.op(8).u32(2).u32(3).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let [px, py, pw, ph] = t.op_box(1).unwrap();
        let [cx, cy, _, _] = t.op_box(2).unwrap();
        assert_eq!((cx, cy), (px, py), "layout does not move");
        let d = &t.drawn;
        let at = |q: &PaintQuad, x: f32, y: f32| {
            let [a, b, c, dd, e, f, _, _] = d.transforms[q.xf as usize];
            [a * x + c * y + e, b * x + dd * y + f]
        };
        let near =
            |u: [f32; 2], v: [f32; 2]| (u[0] - v[0]).abs() < 1e-3 && (u[1] - v[1]).abs() < 1e-3;
        // The parent turns a quarter clockwise about its centre: its top
        // left goes to the top right of the turned box.
        let (mx, my) = (px + pw / 2.0, py + ph / 2.0);
        let pq = &d.quads[0];
        assert!(pq.xf != 0);
        assert!(
            near(at(pq, px, py), [mx + ph / 2.0, my - pw / 2.0]),
            "{:?}",
            at(pq, px, py)
        );
        // The child moves 30px along the turned x axis: down the screen.
        let cq = &d.quads[1];
        assert!(cq.xf != pq.xf);
        assert!(
            near(at(cq, cx, cy), [mx + ph / 2.0, my - pw / 2.0 + 30.0]),
            "{:?}",
            at(cq, cx, cy)
        );
        let kinds: Vec<u32> = d.quads[2..].iter().map(|q| q.kind).collect();
        let g = PaintKind::Raster as u32;
        assert_eq!(
            kinds,
            [g, g, g, g, PaintKind::Solid as u32],
            "shadow, glyphs, underline"
        );
        // `text-shadow-md` is offset down, not across.
        assert_eq!(
            d.quads[2].rect[0], d.quads[4].rect[0],
            "the shadow under its glyph"
        );
        assert!(
            d.quads[2].rect[1] > d.quads[4].rect[1],
            "the shadow's offset"
        );
        assert!(
            d.quads[2..].iter().all(|q| q.xf == cq.xf),
            "the text rides its parents"
        );
    }

    /// A hidden node and everything under it paint nothing and take no
    /// room; showing it again paints it.
    #[test]
    fn a_hidden_node_paints_nothing() {
        let mut t = real();
        nodes(
            &mut t,
            &[
                (1, 0, "hidden w-[40px] h-[40px] bg-white", 0),
                (2, 0, "w-[10px] h-[10px] bg-black", 1),
            ],
        );
        frame(&mut t);
        assert!(t.drawn.quads.is_empty(), "{:?}", t.drawn.quads);
        let mut b = Batch::default();
        b.op(4).u32(1).str("w-[40px] h-[40px] bg-white");
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        assert_eq!(t.drawn.quads.len(), 2);
    }

    /// An `Icon` draws its built-in icon's segments in its box, in its
    /// ink, or in the `fill-*` it or an ancestor sets (`fill-current`: the
    /// ink again). Ways this could fail: fill read only on the icon itself,
    /// or `fill-current` taken for black.
    #[test]
    fn a_built_in_icon_draws() {
        let mut t = real();
        nodes(
            &mut t,
            &[
                (1, 5, "w-[24px] h-[24px] text-white", 0),
                (2, 0, "fill-[#ff0000] text-white", 0),
                (3, 5, "w-[24px] h-[24px]", 2),
                (4, 5, "w-[24px] h-[24px] fill-current", 2),
            ],
        );
        let mut b = Batch::default();
        for id in [1, 3, 4] {
            b.op(6).u32(id).str("name").str("clock");
        }
        t.apply(&b.bytes()).unwrap();
        frame(&mut t);
        let colors: Vec<Rgba> = t
            .drawn
            .quads
            .iter()
            .filter(|q| q.kind == PaintKind::Icon as u32)
            .map(|q| q.color)
            .collect();
        let white = [1.0; 4];
        assert_eq!(colors, [white, [1.0, 0.0, 0.0, 1.0], white]);
        let (first, count) = crate::icons::icons().find("clock").unwrap();
        let q = t
            .drawn
            .quads
            .iter()
            .find(|q| q.kind == PaintKind::Icon as u32)
            .expect("an icon quad");
        assert_eq!((q.a, q.b), (first, count));
        assert_eq!(q.rect, [0.0, 0.0, 24.0, 24.0]);
    }
}
