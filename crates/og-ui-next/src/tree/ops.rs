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
    /// A context menu was asked for on it: the secondary button went down
    /// over it (`x`, `y` the pointer), or Shift+F10 or the Menu key was
    /// pressed while the focus was in it (`value` `"key"`, `x`, `y` its
    /// box's bottom start) (`onContextMenu`).
    ContextMenu,
    /// Its scroll moved (a `Scroll`, or overflow that scrolls), once a
    /// frame: `x`, `y` the offset, `value` `"viewportWidth viewportHeight
    /// contentWidth contentHeight"` (`onScroll`).
    Scroll,
}

impl Event {
    const ALL: [Event; 22] = [
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
        Event::ContextMenu,
        Event::Scroll,
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
    ARIA_PRESSED, BUSY, CHECKED, DISABLED, EXIT, EXPANDED, FOCUS_VISIBLE, FOCUS_WITHIN, FOCUSED,
    HOVERED, INDETERMINATE, INVALID, PRESSED, SELECTED, STARTING,
};

/// The states input and the accessibility props set on op nodes, which
/// their classes may read: NSH's core sets `selected:`, `checked:`,
/// `indeterminate:`, `aria-expanded:`, `aria-pressed:`, `aria-busy:` and
/// `aria-invalid:` from a view's `accessibilityState`, as ours does from
/// the props of the same names.
pub(super) const INTERACTION: [u16; 13] = [
    PRESSED,
    HOVERED,
    FOCUSED,
    DISABLED,
    FOCUS_WITHIN,
    FOCUS_VISIBLE,
    SELECTED,
    CHECKED,
    INDETERMINATE,
    EXPANDED,
    ARIA_PRESSED,
    BUSY,
    INVALID,
];

/// The states a group's descendants read (`group-hover:`,
/// `group-checked:`): all of [`INTERACTION`].
pub(super) const GROUP_STATES: u16 = {
    let mut all = 0;
    let mut i = 0;
    while i < INTERACTION.len() {
        all |= INTERACTION[i];
        i += 1;
    }
    all
};

/// Cached interaction-state and presence metadata for a class string alone
/// (`style == ""`). Inline declarations carry no Tailwind variants, so every
/// `(class, style)` entry reuses its class string's metadata without
/// re-probing all interaction states or presence variants.
#[derive(Clone, Copy, Default)]
struct ClassMeta {
    tracked: u16,
    has_starting: bool,
    has_exit: bool,
}

/// How many unreferenced inline-styled entries may remain cached before
/// orphaned ones are evicted and their slot ids recycled.
const INLINE_ORPHAN_CAP: usize = 64;

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
    /// Live `El::class` references to each entry; inline-styled entries
    /// whose count drops to 0 are recycled once `orphan_styled` grows past
    /// [`INLINE_ORPHAN_CAP`].
    refs: Vec<u32>,
    free: Vec<u32>,
    orphan_styled: Vec<u32>,
    class_meta: FxHashMap<Box<str>, ClassMeta>,
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
/// differ from it, using `meta` to skip probing absent presence variants.
fn resolved_with(
    resolve: Option<&Resolve>,
    c: &str,
    s: &str,
    meta: ClassMeta,
) -> (Record, [Option<Box<Record>>; 2]) {
    let Some(f) = resolve else {
        return (Record::default(), [None, None]);
    };
    let base = f(c, s, 0, "");
    let starting = meta
        .has_starting
        .then(|| Box::new(f(c, s, STARTING, "")))
        .filter(|r| **r != base);
    let exit = meta
        .has_exit
        .then(|| Box::new(f(c, s, EXIT, "")))
        .filter(|r| **r != base);
    (base, [starting, exit])
}

impl Classes {
    /// Theme colour role `role` as `text-<role>` resolves it, in the
    /// page's scheme or the other one; transparent without a resolver.
    pub(super) fn tone(&self, role: &str, other: bool) -> Rgba {
        let Some(f) = &self.resolve else {
            return [0.0; 4];
        };
        let state = if other {
            crate::classes::OTHER_SCHEME
        } else {
            0
        };
        f(&format!("text-{role}"), "", state, "").skin.ink
    }

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
    pub(super) fn groups(&self, i: u32) -> impl Iterator<Item = &str> {
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

    pub(super) fn acquire(&mut self, i: u32) {
        if let Some(r) = self.refs.get_mut(i as usize) {
            *r = r.saturating_add(1);
        }
    }

    pub(super) fn release(&mut self, i: u32) {
        let Some(r) = self.refs.get_mut(i as usize) else {
            return;
        };
        *r = r.saturating_sub(1);
        if *r == 0
            && !self.keys[i as usize].is_empty()
            && !Self::split(&self.keys[i as usize]).1.is_empty()
        {
            self.orphan_styled.push(i);
            if self.orphan_styled.len() > INLINE_ORPHAN_CAP {
                self.evict_orphans();
            }
        }
    }

    fn evict_orphans(&mut self) {
        let keep = INLINE_ORPHAN_CAP / 2;
        let drain_end = self.orphan_styled.len().saturating_sub(keep);
        let mut evicted: Vec<u32> = Vec::new();
        for idx in self.orphan_styled.drain(..drain_end) {
            let slot = idx as usize;
            if self.refs[slot] == 0
                && !self.keys[slot].is_empty()
                && !Self::split(&self.keys[slot]).1.is_empty()
            {
                let key = std::mem::take(&mut self.keys[slot]);
                self.index.remove(&key);
                self.presence[slot] = [None, None];
                self.tracked[slot] = 0;
                self.free.push(idx);
                evicted.push(idx);
            }
        }
        if !evicted.is_empty() {
            self.stated.retain(|&(i, _, _), _| !evicted.contains(&i));
        }
    }

    fn class_meta(&mut self, c: &str) -> ClassMeta {
        if let Some(&m) = self.class_meta.get(c) {
            return m;
        }
        let Some(f) = self.resolve.as_ref() else {
            return ClassMeta::default();
        };
        let base = f(c, "", 0, "");
        let meta = ClassMeta {
            tracked: tracks(Some(f), c, "", &base),
            has_starting: f(c, "", STARTING, "") != base,
            has_exit: f(c, "", EXIT, "") != base,
        };
        self.class_meta.insert(c.into(), meta);
        meta
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
        let key: Box<str> = self.buf.as_str().into();
        let (c, s) = Self::split(&key);
        let (record, presence, tracked) = if s.is_empty() {
            if let Some(f) = self.resolve.as_ref() {
                let base = f(c, "", 0, "");
                let starting = Some(Box::new(f(c, "", STARTING, ""))).filter(|r| **r != base);
                let exit = Some(Box::new(f(c, "", EXIT, ""))).filter(|r| **r != base);
                let tr = match self.class_meta.get(c) {
                    Some(&m) => m.tracked,
                    None => {
                        let tr = tracks(Some(f), c, "", &base);
                        self.class_meta.insert(
                            c.into(),
                            ClassMeta {
                                tracked: tr,
                                has_starting: starting.is_some(),
                                has_exit: exit.is_some(),
                            },
                        );
                        tr
                    }
                };
                (base, [starting, exit], tr)
            } else {
                (Record::default(), [None, None], 0)
            }
        } else {
            let meta = self.class_meta(c);
            let (record, presence) = resolved_with(self.resolve.as_ref(), c, s, meta);
            (record, presence, meta.tracked)
        };
        if let Some(i) = self.free.pop() {
            let slot = i as usize;
            self.records[slot] = record;
            self.presence[slot] = presence;
            self.tracked[slot] = tracked;
            self.refs[slot] = 0;
            self.keys[slot] = key.clone();
            self.index.insert(key, i);
            i
        } else {
            let i = self.keys.len() as u32;
            self.records.push(record);
            self.presence.push(presence);
            self.tracked.push(tracked);
            self.refs.push(0);
            self.keys.push(key.clone());
            self.index.insert(key, i);
            i
        }
    }

    fn reresolve(&mut self) {
        self.stated.clear();
        self.class_meta.clear();
        for i in 0..self.keys.len() {
            if self.keys[i].is_empty() {
                continue;
            }
            let key = self.keys[i].clone();
            let (c, s) = Self::split(&key);
            let meta = self.class_meta(c);
            (self.records[i], self.presence[i]) = resolved_with(self.resolve.as_ref(), c, s, meta);
            self.tracked[i] = meta.tracked;
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
        self.tone_colors.borrow_mut().clear();
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
        if let Some(w) = self.els[e as usize].image.as_ref().and_then(|im| im.wipe) {
            record.skin.image.wipe = w;
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

    /// Replaces `e`'s class entry with `new`, maintaining reference counts.
    fn swap_class(&mut self, e: u32, new: u32) {
        let old = self.els[e as usize].class;
        if new != old {
            self.classes.acquire(new);
            self.classes.release(old);
            self.els[e as usize].class = new;
        }
    }

    /// Whether op element `e` is a control a finger presses: it answers a
    /// press, or it is an action's touch button (`touchButton`).
    /// An `Input`'s class: the game's own, after `@Field.frame` (or
    /// `@Field.area` when it is multi-line) unless the game's names a pack
    /// slot of its own. Its slot draws its box and its states.
    pub(super) fn field_class(&mut self, e: u32) {
        let p = &self.input.per[e as usize];
        let own = p.own_class.split_whitespace().any(|c| c.starts_with('@'));
        let class = match (own, p.multiline) {
            (true, _) => p.own_class.to_string(),
            (false, false) => format!("@Field.frame {}", p.own_class),
            (false, true) => format!("@Field.area {}", p.own_class),
        };
        let old = self.els[e as usize].class;
        let new = self.classes.with(old, true, &class);
        self.swap_class(e, new);
        let slot = class.split_whitespace().find_map(|c| c.strip_prefix('@'));
        self.input.per[e as usize].slot = slot.unwrap_or("").into();
        self.data_dirty = true;
        self.restyle(e);
    }

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
    fn enter_subtree(&mut self, top: u32, wait: f32) {
        for i in 0..self.entering.len() {
            let d = self.entering[i];
            if d == top || self.els[d as usize].entering != Some(false) {
                continue;
            }
            if !self.is_under(d, top) {
                continue;
            }
            self.els[d as usize].entering = Some(true);
            let wait = wait + self.cascade(d);
            let el = &self.els[d as usize];
            let from = self.classes.presence(el.class, 0);
            // A child its list staggers enters late even with no
            // `starting:` of its own: its `animate-*` starts then.
            if from.is_some() || wait > 0.0 {
                self.motion.enter_after(el.node, &el.record, from, wait);
            }
        }
    }

    /// `e`'s ancestors, nearest first, four at most (`NONE` past the root).
    fn ancestors(&self, e: u32) -> [u32; 4] {
        let mut out = [NONE; 4];
        let mut a = self.els[e as usize].parent;
        for o in &mut out {
            if a == NONE {
                break;
            }
            *o = a;
            a = self.els[a as usize].parent;
        }
        out
    }

    /// The box `e` was last drawn in, in surface space.
    fn drawn_rect(&self, e: u32) -> Option<[f32; 4]> {
        let a = self.abs.get(e as usize).filter(|a| a.2)?;
        let n = self.els[e as usize].node;
        let b = self
            .motion
            .box_of(n)
            .unwrap_or_else(|| self.arena.box_of(n));
        Some([a.0, a.1, b.w, b.h])
    }

    /// A `glide-<name>` element `e` is taken off the screen: where it was
    /// is kept a moment for the one that replaces it.
    fn left_glide(&mut self, e: u32) {
        let t = self.time;
        self.glides.retain(|g| t - g.1 < 0.5);
        if let Some(rect) = self.drawn_rect(e) {
            let name = self.els[e as usize].record.glide.clone().expect("a glide");
            let anc = self.ancestors(e);
            self.glides.push((name, t, rect, anc));
        }
    }

    /// A `glide-<name>` element `child` went on screen: it slides from
    /// the nearest element of its name, by their common ancestor, that
    /// shows (the selection it replaces, removed after it) or left it in
    /// the last half second (removed before it), as Framer Motion's
    /// `layoutId` does. Nothing near: it enters as it is.
    fn glide_from(&mut self, child: u32) {
        let name = self.els[child as usize]
            .record
            .glide
            .clone()
            .expect("a glide");
        let mine = self.ancestors(child);
        let near = |anc: &[u32; 4]| mine.iter().position(|&a| a != NONE && anc.contains(&a));
        let mut best: Option<(usize, [f32; 4])> = None;
        let mut take = |level: Option<usize>, rect: [f32; 4]| {
            if let Some(l) = level
                && best.is_none_or(|b| l < b.0)
            {
                best = Some((l, rect));
            }
        };
        for d in 0..self.els.len() as u32 {
            let el = &self.els[d as usize];
            if d == child || el.dead || el.record.glide.as_deref() != Some(&*name) {
                continue;
            }
            if let Some(rect) = self.drawn_rect(d) {
                take(near(&self.ancestors(d)), rect);
            }
        }
        let t = self.time;
        for g in self
            .glides
            .iter()
            .filter(|g| *g.0 == *name && t - g.1 < 0.5)
        {
            take(near(&g.3), g.2);
        }
        let Some((_, [x, y, w, h])) = best else {
            return;
        };
        let p = self.els[child as usize].arena_parent;
        let (px, py) = match self.abs.get(p as usize) {
            Some(a) => {
                let [sx, sy] = self.scroll_of(p);
                (a.0 - sx, a.1 - sy)
            }
            None => (0.0, 0.0),
        };
        let node = self.els[child as usize].node;
        let mut from = self.arena.box_of(node);
        (from.x, from.y, from.w, from.h) = (x - px, y - py, w, h);
        self.motion.glide(node, from);
    }

    /// How long `child`, put on screen now, waits for its turn in its
    /// parent's cascade (`stagger-<step>/<cap>`): the n-th child entering
    /// at this time waits n steps, never past the cap. 0 under a parent
    /// with none.
    fn cascade(&mut self, child: u32) -> f32 {
        let p = self.els[child as usize].parent;
        let Some([step, cap]) = (p != NONE)
            .then(|| self.els[p as usize].record.stagger)
            .flatten()
        else {
            return 0.0;
        };
        if self.cascade.0 != self.time {
            self.cascade = (self.time, Vec::new());
        }
        let ranks = &mut self.cascade.1;
        let n = match ranks.iter_mut().find(|r| r.0 == p) {
            Some(r) => {
                r.1 += 1;
                r.1 - 1
            }
            None => {
                ranks.push((p, 1));
                0
            }
        };
        (n as f32 * step).min(cap)
    }

    /// Longest remaining exit time of `e` and all descendants under it.
    fn max_leaves_in(&self, e: u32) -> f32 {
        let el = &self.els[e as usize];
        if el.dead {
            return 0.0;
        }
        let mut wait = self.motion.leaves_in(el.node);
        for &c in &el.children {
            wait = wait.max(self.max_leaves_in(c));
        }
        wait
    }

    /// How long `child`, put on screen beside an absolutely placed sibling
    /// on its way out, holds its entrance: until what leaves there has
    /// left, so a screen that replaces one (a `<Show>` branch, a mount)
    /// enters after it instead of over it. 0 for anything else: a list's
    /// rows, which flow, come and go together.
    fn handoff(&self, child: u32) -> f32 {
        let abs =
            |e: u32| self.els[e as usize].record.layout.position() == taffy::Position::Absolute;
        if !abs(child) {
            return 0.0;
        }
        let parent = self.els[child as usize].parent;
        let mut wait = 0.0f32;
        if parent != NONE {
            for &s in &self.els[parent as usize].children {
                let el = &self.els[s as usize];
                if s == child || el.dead || !abs(s) {
                    continue;
                }
                if !self.motion.exiting(el.node) && !self.leaving.iter().any(|l| l.0 == s) {
                    continue;
                }
                wait = wait.max(self.max_leaves_in(s));
            }
        } else {
            for s in 0..self.els.len() as u32 {
                let el = &self.els[s as usize];
                if s == child || el.dead || el.parent != NONE || !abs(s) {
                    continue;
                }
                if !self.motion.exiting(el.node) && !self.leaving.iter().any(|l| l.0 == s) {
                    continue;
                }
                wait = wait.max(self.max_leaves_in(s));
            }
        }
        wait
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
        self.classes.acquire(class);
        let record = self.classes.records[class as usize].clone();
        let node = self.arena.element(NONE, record.layout.clone());
        let leaf = text.then(|| self.arena.text(node, "", 16.0));
        let fresh = El {
            node,
            disabled: false,
            clickable: false,
            squash: false,
            js: id,
            listens: 0,
            class,
            arena_parent: NONE,
            parent: NONE,
            children: Vec::new(),
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
            inline: NONE,
            para: None,
            queued: false,
            ruby: Box::default(),
        };
        let e = if let Some(slot) = self.free_els.pop() {
            self.els[slot as usize] = fresh;
            if let Some(a) = self.abs.get_mut(slot as usize) {
                a.2 = false;
            }
            if let Some(x) = self.xfs.get_mut(slot as usize) {
                *x = 0;
            }
            slot
        } else {
            let slot = self.els.len() as u32;
            self.els.push(fresh);
            slot
        };
        if self.by_js.len() <= id as usize {
            self.by_js.resize(id as usize + 1, NONE);
        }
        self.by_js[id as usize] = e;
        if self.input.per.len() <= e as usize {
            self.input.per.resize(e as usize + 1, input::Per::default());
        } else {
            self.input.per[e as usize] = input::Per::default();
        }
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
        let mut stack: Vec<u32> = self.els[top as usize].children.clone();
        while let Some(d) = stack.pop() {
            let el = &self.els[d as usize];
            if el.dead {
                continue;
            }
            stack.extend_from_slice(&el.children);
            if el.js == NONE {
                continue;
            }
            let to = self.classes.presence(el.class, 1);
            any |= to.is_some() && self.motion.exit(el.node, &el.record, to);
        }
        any
    }

    /// Whether an element under `top` is on its way out.
    fn exiting_under(&self, top: u32) -> bool {
        fn check(tree: &Tree, e: u32) -> bool {
            for &c in &tree.els[e as usize].children {
                let el = &tree.els[c as usize];
                if el.dead {
                    continue;
                }
                if tree.motion.exiting(el.node) || check(tree, c) {
                    return true;
                }
            }
            false
        }
        check(self, top)
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
        for k in 0..self.els[e as usize].children.len() {
            let c = self.els[e as usize].children[k];
            if !self.els[c as usize].dead {
                self.unbind(c);
            }
        }
    }

    /// Frees `e` and what is under it now, recycling both element and arena slots.
    fn free(&mut self, e: u32) {
        if self.els[e as usize].dead {
            return;
        }
        let kids = std::mem::take(&mut self.els[e as usize].children);
        for c in kids {
            if self.els[c as usize].parent == e {
                self.els[c as usize].parent = NONE;
            }
            if !self.els[c as usize].dead {
                self.free(c);
            }
        }
        self.forget_el(e, self.els[e as usize].node);
        self.set_parent(e, NONE);
        let el = &mut self.els[e as usize];
        el.dead = true;
        let (node, leaf, js, class) = (el.node, el.leaf.take(), el.js, el.class);
        el.js = NONE;
        el.arena_parent = NONE;
        el.text.clear();
        el.attrs.clear();
        el.line = None;
        el.particles = None;
        el.anchor = None;
        el.image = None;
        el.para = None;
        self.classes.release(class);
        if let Some(leaf) = leaf {
            self.texts.release(leaf);
            self.arena.release(leaf);
        }
        self.arena.release(node);
        self.motion.forget(node);
        self.sem.release(node);
        self.anchored.retain(|&a| a != e);
        self.spotlights.retain(|&s| s != e);
        self.leaving.retain(|l| l.0 != e);
        self.entering.retain(|&x| x != e);
        self.counts.retain(|c| c.e != e);
        self.paras.retain(|&p| p != e);
        self.warned_layers.retain(|&n| n != node);
        self.warned_paras.retain(|&x| x != e);
        self.glides.retain(|g| !g.3.contains(&e));
        self.cascade.1.retain(|r| r.0 != e);
        if self.select.as_ref().is_some_and(|s| s.e == e) {
            self.select = None;
        }
        if self.select_press.as_ref().is_some_and(|p| p.e == e) {
            self.select_press = None;
        }
        if let Some(a) = self.abs.get_mut(e as usize) {
            a.2 = false;
        }
        if let Some(x) = self.xfs.get_mut(e as usize) {
            *x = 0;
        }
        if let Some(slot) = self.by_js.get_mut(js as usize).filter(|s| **s == e) {
            *slot = NONE;
        }
        self.ordered = false;
        self.free_els.push(e);
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
                self.set_parent(e, NONE);
                self.els[e as usize].arena_parent = NONE;
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
                self.set_parent(e, NONE);
                self.els[e as usize].arena_parent = NONE;
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
                        1 => Kind::Text,
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
                    if tag == 4 {
                        self.field_class(e);
                    }
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
                    // An `Input`'s class keeps its default field slot.
                    if code == 4 && self.input.per[e as usize].input {
                        self.input.per[e as usize].own_class = text.into();
                        self.field_class(e);
                        continue;
                    }
                    let old = self.els[e as usize].class;
                    let new = self.classes.with(old, code == 4, text);
                    self.swap_class(e, new);
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
                    let node = self.els[e as usize].node;
                    let was = self.sem.get(node).range[2];
                    self.sem.prop(node, name, value);
                    // A slider moved to a new value sounds its `step`.
                    let s = self.sem.get(node);
                    if name == "valueNow"
                        && s.role == Some(Some(accesskit::Role::Slider))
                        && was.is_some()
                        && s.range[2] != was
                    {
                        self.sound(e, "step");
                    }
                    match name {
                        "disabled" => {
                            let on = !matches!(value, "" | "false" | "0");
                            let was = self.els[e as usize].disabled;
                            self.els[e as usize].disabled = on;
                            self.set_state(e, DISABLED, on);
                            // A control drawn disabled that becomes ready
                            // calls for attention once (`Attention.cue`).
                            if was && !on && self.els[e as usize].entering.is_none() {
                                let cue = self.classes.get("@Attention.cue", "");
                                let lp = self.classes.record(cue, 0, 0).motion.animate;
                                if let Some(lp) = lp {
                                    let el = &self.els[e as usize];
                                    self.motion.cue(el.node, lp, &el.record.skin);
                                    self.paint = true;
                                }
                            }
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
                                p.composing.clear();
                                p.caret = p.value.len();
                                p.anchor = p.caret;
                                // Another text: what undo knew is gone.
                                p.undo.clear();
                                p.redo.clear();
                                p.last_edit = Default::default();
                            }
                            self.show_value(e);
                        }
                        "type" if self.input.per[e as usize].input => {
                            let v = value.trim();
                            let p = &mut self.input.per[e as usize];
                            p.input_type = v.into();
                            if v == "password" {
                                p.secret = true;
                            }
                            self.show_value(e);
                        }
                        "inputMode" | "inputmode" if self.input.per[e as usize].input => {
                            self.input.per[e as usize].input_mode = value.trim().into();
                        }
                        "autoComplete" | "autocomplete" if self.input.per[e as usize].input => {
                            self.input.per[e as usize].autocomplete = value.trim().into();
                        }
                        "secret" if self.input.per[e as usize].input => {
                            self.input.per[e as usize].secret =
                                !matches!(value, "" | "false" | "0" | "null");
                            self.show_value(e);
                        }
                        "multiline" if self.input.per[e as usize].input => {
                            self.input.per[e as usize].multiline =
                                !matches!(value, "" | "false" | "0" | "null");
                            self.field_class(e);
                            self.inherit(e);
                        }
                        // `rows`: a multi-line field that many lines tall.
                        "rows" if self.input.per[e as usize].input => {
                            let rows = value.trim().parse::<u16>().unwrap_or(0);
                            let p = &mut self.input.per[e as usize];
                            p.rows = rows;
                            p.multiline |= rows > 1;
                            self.field_class(e);
                            self.inherit(e);
                        }
                        "maxLength" | "maxlength" if self.input.per[e as usize].input => {
                            self.input.per[e as usize].max_length =
                                value.trim().parse::<usize>().unwrap_or(0);
                        }
                        "readOnly" | "readonly" if self.input.per[e as usize].input => {
                            let on = !matches!(value, "" | "false" | "0" | "null");
                            self.input.per[e as usize].read_only = on;
                            // `data-readonly:` classes draw it.
                            self.mark(e, "readonly", on);
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
                        "focusRoving" => {
                            self.input.per[e as usize].roving =
                                !matches!(value, "" | "false" | "0" | "null");
                        }
                        "draggable" => self.input.per[e as usize].drag = value.trim().into(),
                        "dropTarget" => self.input.per[e as usize].drop = value.trim().into(),
                        "pinned" => {
                            let on = !matches!(value, "" | "false" | "0" | "null");
                            let node = self.els[e as usize].node;
                            self.input.per[e as usize].pinned = on;
                            self.input.pinned.retain(|&n| n != node);
                            if on {
                                self.input.pinned.push(node);
                            }
                            self.paint = true;
                        }
                        "thumb" => {
                            self.input.per[e as usize].thumb = value.trim().into();
                            if !self.input.thumbs.contains(&e) {
                                self.input.thumbs.push(e);
                            }
                            self.inherit(e);
                        }
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
                        // `haptic`: preset override played with this node's
                        // non-hover interactions and cues (`"none"` silences).
                        "haptic" => {
                            self.input.per[e as usize].haptic = value.into();
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
                                "wipe" => {
                                    let next = number.map(|w| w.clamp(0.0, 1.0));
                                    props.wipe = next;
                                    let target = next.unwrap_or(1.0);
                                    if self.els[e as usize].record.skin.image.wipe != target {
                                        let mut next_rec = self.els[e as usize].record.clone();
                                        next_rec.skin.image.wipe = target;
                                        let old = std::mem::replace(
                                            &mut self.els[e as usize].record,
                                            next_rec,
                                        );
                                        let el = &self.els[e as usize];
                                        if el.entering.is_none() {
                                            self.motion.restyle(el.node, &old, &el.record);
                                        }
                                    }
                                }
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
                        // `ruby`: a span's annotation (tree/para.rs).
                        "ruby" if self.els[e as usize].kind == Kind::Text => {
                            if *self.els[e as usize].ruby != *value {
                                self.els[e as usize].ruby = value.into();
                                self.para_touch(e);
                            }
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
                        // An accessibility state is also the class state
                        // of its name, as NSH's core sets it from a view's
                        // `accessibilityState`: `checked` true is
                        // `checked:`, "mixed" `indeterminate:`.
                        "checked" | "pressed" | "selected" | "expanded" | "busy" | "invalid" => {
                            let v = value.trim().trim_matches('"');
                            let on = matches!(v, "true" | "1");
                            match name {
                                "checked" => {
                                    self.set_state(e, CHECKED, on);
                                    self.set_state(e, INDETERMINATE, v == "mixed");
                                }
                                "pressed" => self.set_state(e, ARIA_PRESSED, on),
                                "selected" => self.set_state(e, SELECTED, on),
                                "expanded" => self.set_state(e, EXPANDED, on),
                                "busy" => self.set_state(e, BUSY, on),
                                _ => self.set_state(e, INVALID, on),
                            }
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
                    // The paragraph it leaves and the one it joins lay out again.
                    self.para_touch(self.els[child as usize].parent);
                    self.arena
                        .attach(self.els[child as usize].node, pnode, before);
                    self.set_parent(child, p);
                    self.els[child as usize].arena_parent = p;
                    self.para_touch(child);
                    self.data_dirty = true;
                    let held = self.leaving.iter().any(|l| l.0 == child)
                        && !self.motion.exiting(self.els[child as usize].node);
                    self.leaving.retain(|l| l.0 != child);
                    let el = &self.els[child as usize];
                    self.sem.exiting(el.node, false);
                    if held {
                        // Put back while what is under it exits: they turn back.
                        let mut stack: Vec<u32> = self.els[child as usize].children.clone();
                        while let Some(d) = stack.pop() {
                            let el = &self.els[d as usize];
                            if el.dead {
                                continue;
                            }
                            stack.extend_from_slice(&el.children);
                            if self.motion.exiting(el.node) {
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
                    let wait = if shown {
                        self.handoff(child) + self.cascade(child)
                    } else {
                        0.0
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
                    self.motion.enter_after(el.node, &el.record, from, wait);
                    if shown {
                        self.enter_subtree(child, wait);
                        if self.els[child as usize].record.glide.is_some() {
                            self.glide_from(child);
                        }
                    }
                }
                // remove
                9 => {
                    let _parent = r.u32()?;
                    let child = self.op(r.u32()?)?;
                    self.para_touch(self.els[child as usize].parent);
                    if self.els[child as usize].record.glide.is_some() {
                        self.left_glide(child);
                    }
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
                    self.set_parent(child, NONE);
                    self.els[child as usize].arena_parent = NONE;
                    self.ordered = false;
                    self.paint = true;
                }
                // release
                10 => {
                    let e = self.op(r.u32()?)?;
                    self.para_touch(self.els[e as usize].parent);
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
                    // Its states, which `group-<state>:` reads.
                    let st = self.input.per(p).map_or(0, |pp| pp.states) & GROUP_STATES;
                    if st != 0 {
                        use std::fmt::Write;
                        let _ = write!(data, "{g}/@{st} ");
                    }
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
        self.fire_scrolled();
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
    /// A press squashes it: an enabled control a finger presses that its
    /// own classes or the pack's `Press.feedback` move under a press.
    pub press: bool,
    /// Its classes give it a `starting:` record it enters from (0) and an
    /// `exit:` record it leaves to (1).
    pub presence: [bool; 2],
    /// A `modal` view: a dialog the rest waits behind.
    pub modal: bool,
    /// It draws a nine-sliced frame its fill colours (a tint mask): that
    /// fill (linear RGB, alpha), at rest. `None`: no such frame.
    pub frame: Option<[f32; 4]>,
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
                let mut b = self.arena.box_of(el.node);
                // A span in a paragraph is where a press on it lands.
                let (x, y) = match self.span_piece(e as u32) {
                    Some([sx, sy, w, h]) => {
                        (b.w, b.h) = (w, h);
                        (sx, sy)
                    }
                    None => (x, y),
                };
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
                    press: el.squash
                        && self.presses(e as u32)
                        && self
                            .input
                            .per(e as u32)
                            .is_none_or(|p| p.states & DISABLED == 0)
                        && !self.catcher(e as u32),
                    presence: [0, 1].map(|k| self.classes.presence(el.class, k).is_some()),
                    modal: self.input.per(e as u32).is_some_and(|p| p.modal),
                    frame: {
                        let im = &el.record.skin.image;
                        (im.texture != 0
                            && im.slice != [0.0; 4]
                            && im.tint == crate::record::Tint::Fill)
                            .then_some(el.record.skin.fill)
                    },
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
