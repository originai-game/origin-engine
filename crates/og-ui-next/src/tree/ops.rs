//! The TS renderer's ops applied to the arena (CONVENTIONS.md §Ops): a
//! binary batch read in place, each op patching the element it names. An
//! op node is an element of the tree like a document's, so layout, text,
//! icons and paint serve it unchanged; its record comes from its class and
//! style through the resolver, once per distinct pair.

use super::*;
use crate::a11y;

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
}

impl Event {
    const ALL: [Event; 17] = [
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
        let st = states & self.tracked(i);
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
        self.classes.resolve = Some(resolve);
        self.restyle_all();
    }

    /// Takes another look (a scheme or pack change): what the root passes
    /// down and every op node's record follow it.
    pub fn set_look(&mut self, look: Rc<Look>) {
        let mut root = look.root.clone();
        root.text.set = 0;
        self.root = Inherit {
            size: root.font_size,
            ink: root.skin.ink,
            text: root.text,
            opacity: 1.0,
        };
        self.look = look;
        self.restyle_all();
        for e in 0..self.els.len() as u32 {
            let el = &self.els[e as usize];
            if el.js != NONE && !el.dead && el.parent == NONE {
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

    pub(super) fn restyle(&mut self, e: u32) {
        let class = self.els[e as usize].class;
        let states = self.input.per(e).map_or(0, |p| p.states);
        let data = self.els[e as usize].data;
        let record = self.classes.record(class, states, data);
        if self.els[e as usize].record != *record {
            // The new record's transition runs the change.
            let old = std::mem::replace(&mut self.els[e as usize].record, record.clone());
            let el = &self.els[e as usize];
            self.motion.restyle(el.node, &old, &el.record);
            self.reapply(e);
            self.paint = true;
        }
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
            at: 0,
            row: self.row.clone(),
            disabled: NONE,
            clickable: false,
            js: id,
            listens: 0,
            class,
            arena_parent: NONE,
            parent: NONE,
            inst: NONE,
            style: NONE,
            kind,
            leaf,
            text: String::new(),
            hidden: 0,
            dead: false,
            queued: false,
            record,
            holds: Box::default(),
            line: None,
            anchor: None,
            attrs: Vec::new(),
            data: 0,
            image: None,
            eff: self.root,
        });
        if self.by_js.len() <= id as usize {
            self.by_js.resize(id as usize + 1, NONE);
        }
        self.by_js[id as usize] = e;
        self.input.per.resize(e as usize + 1, input::Per::default());
        self.input.per[e as usize].tab = input::NO_TAB;
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

    /// Whether `e` or an element it is in is on its way out.
    fn leaving_in(&self, mut e: u32) -> bool {
        while e != NONE {
            if self.motion.exiting(self.els[e as usize].node) {
                return true;
            }
            e = self.els[e as usize].parent;
        }
        false
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
        // What the renderer released inside an exit that ended.
        let mut k = 0;
        while k < self.leaving.len() {
            let (e, release) = self.leaving[k];
            if release && !self.motion.exiting(self.els[e as usize].node) && !self.leaving_in(e) {
                self.free(e);
            } else {
                k += 1;
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
                        0..=6 => Kind::Box,
                        t => return Err(format!("og.ui ops: unknown tag {t}")),
                    };
                    // An `Input` shows its text in a text leaf of its own.
                    let e = self.create(id, kind, tag == 4)?;
                    self.input.per[e as usize].scroller = tag == 3;
                    if tag == 6 {
                        self.els[e as usize].line = Some(Box::default());
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
                            match self.els[e as usize].disabled {
                                NONE => self.els[e as usize].disabled = queue::signal(on.into()),
                                s => queue::set(s, on.into()),
                            }
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
                            self.input.zone_els.retain(|&z| z != e);
                            if per.zone.is_some() {
                                self.input.zone_els.push(e);
                            }
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
                        "deadZone" => {
                            self.input.per[e as usize].dead_zone =
                                value.trim().parse::<f32>().unwrap_or(0.0).clamp(0.0, 0.95);
                        }
                        "glyph" => {
                            self.input.per[e as usize].glyph = value.into();
                            self.input.glyphs_wanted = true;
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
                        // `anchor` an entity's number or a world point
                        // `x,y,z` (empty: back in flow), `lift` a world
                        // offset: the host places the node (§World).
                        "anchor" => {
                            let n: Vec<f32> = value
                                .split([',', ' '])
                                .filter(|t| !t.is_empty())
                                .map(|t| t.parse().unwrap_or(0.0))
                                .collect();
                            let target = match n[..] {
                                [] => None,
                                [x, y, z] => Some(super::AnchorTarget::Point([x, y, z])),
                                _ => value
                                    .trim()
                                    .parse::<f64>()
                                    .ok()
                                    .map(|b| super::AnchorTarget::Entity(b as u64)),
                            };
                            let node = self.els[e as usize].node;
                            let was = self.els[e as usize].anchor.is_some();
                            match target {
                                Some(t) if was => {
                                    self.els[e as usize]
                                        .anchor
                                        .as_mut()
                                        .expect("anchored")
                                        .target = t
                                }
                                // Off the screen until the host places it.
                                Some(target) => {
                                    self.els[e as usize].anchor = Some(Box::new(super::Anchor {
                                        target,
                                        lift: [0.0; 3],
                                        at: [0.0; 2],
                                        shown: false,
                                    }));
                                    self.anchored.push(e);
                                    self.sem.place(node, None);
                                    self.reapply(e);
                                }
                                None if was => {
                                    self.els[e as usize].anchor = None;
                                    self.anchored.retain(|&a| a != e);
                                    self.sem.unplace(node);
                                    self.reapply(e);
                                }
                                None => {}
                            }
                            self.paint = true;
                        }
                        "lift" => {
                            let mut lift = [0.0f32; 3];
                            for (l, t) in lift
                                .iter_mut()
                                .zip(value.split([',', ' ']).filter(|t| !t.is_empty()))
                            {
                                *l = t.parse().unwrap_or(0.0);
                            }
                            if let Some(a) = &mut self.els[e as usize].anchor {
                                a.lift = lift;
                            }
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
                    el.clickable = el.listens & (Event::Press.bit() | Event::Tap.bit()) != 0;
                    let (node, pressable) = (el.node, el.clickable);
                    self.sem.pressable(node, pressable);
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
                    self.leaving.retain(|l| l.0 != child);
                    self.sem.exiting(el.node, false);
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
                    self.motion.enter(el.node, &el.record, from);
                }
                // remove
                9 => {
                    let _parent = r.u32()?;
                    let child = self.op(r.u32()?)?;
                    let el = &self.els[child as usize];
                    // With an `exit:` record it stays drawn, not hit,
                    // until its exit settles.
                    let to = self.classes.presence(el.class, 1);
                    if self.motion.exit(el.node, &el.record, to) {
                        self.sem.exiting(el.node, true);
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
    fn refresh_data(&mut self) {
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
            if id != self.els[e as usize].data {
                self.els[e as usize].data = id;
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
    pub shown: bool,
    pub text: &'a str,
    /// Where its laid-out lines cut a word with no UAX #14 break there,
    /// each as `end of the line|start of the next`: a text measured
    /// narrower than it draws. Empty: none.
    pub broken: Vec<String>,
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
        let (width, height) = self.env.size.get();
        let viewport = a11y::Viewport {
            width,
            height,
            scale,
        };
        client.update(&self.arena, &self.sem, focus, viewport)
    }

    /// Every live op node, for the probe: its op id, its op parent's (0 at
    /// a mount's top), its arena node, its box on screen (x, y, width,
    /// height) as last painted, whether it shows, and its text.
    pub fn dump_ops(&self) -> Vec<OpNode<'_>> {
        self.els
            .iter()
            .enumerate()
            .filter(|(_, el)| !el.dead && el.js != NONE)
            .filter_map(|(e, el)| {
                let &(x, y, shown, _) = self.abs.get(e)?;
                let b = self.arena.box_of(el.node);
                let parent = match el.parent {
                    NONE => 0,
                    p => self.els[p as usize].js,
                };
                Some(OpNode {
                    id: el.js,
                    parent,
                    node: el.node,
                    rect: [x, y, b.w, b.h],
                    shown,
                    text: &el.text,
                    broken: self.glyphs.get(e).map_or(Vec::new(), |g| {
                        crate::text::broken_words(&g.text, &g.line_text)
                    }),
                })
            })
            .collect()
    }

    /// The accessibility tree now, built whole ([`a11y::snapshot`]), with
    /// the focus where input put it: each arena node's facts and the focus
    /// order. For readers that ask once (the probe).
    pub fn a11y_snapshot(&self) -> (rustc_hash::FxHashMap<Id, a11y::Facts>, Vec<Id>) {
        let (width, height) = self.env.size.get();
        let viewport = a11y::Viewport {
            width,
            height,
            scale: 1.0,
        };
        a11y::snapshot(&self.arena, &self.sem, self.focused(), viewport)
    }

    /// The viewport the tree lays out in, CSS px.
    pub fn size(&self) -> (f32, f32) {
        self.env.size.get()
    }

    /// Op node `id`'s box on screen (x, y, width, height) as last painted,
    /// when it shows.
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
    use crate::look::Layer;

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
        let look = Rc::new(Look::load(
            &[Layer {
                tokens: crate::look::default_tokens(),
                surfaces: crate::look::DEFAULT_SURFACES,
            }],
            "dark",
        ));
        let mut t = Tree::new(look, []);
        t.resize(400.0, 300.0);
        // `row` lays its children in a row; `b` is 50 by 20.
        t.set_resolver(Box::new(|class, _, _, _| {
            let mut s = taffy::Style::default();
            for c in class.split_whitespace() {
                match c {
                    "row" => s.flex_direction = taffy::FlexDirection::Row,
                    "b" => s.size = taffy::Size::from_lengths(50.0, 20.0),
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
            r
        }));
        t
    }

    fn frame(t: &mut Tree) {
        t.frame(&og_ui::Fonts::default(), &IconAtlas::default());
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
        let mut out = Vec::new();
        // 2 does not listen: a press on it fires nothing.
        t.pointer(75.0, 10.0, true, false, &mut out);
        t.pointer(75.0, 10.0, false, true, &mut out);
        assert!(t.events.is_empty());
        t.pointer(25.0, 10.0, true, false, &mut out);
        t.pointer(25.0, 10.0, false, true, &mut out);
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
        t.pointer(25.0, 10.0, true, false, &mut out);
        t.pointer(25.0, 10.0, false, true, &mut out);
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
            t.place_anchors(|target, lift| {
                asked.borrow_mut().push((target, lift));
                at
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
        let mut out = Vec::new();
        t.pointer(130.0, 65.0, true, false, &mut out);
        t.pointer(130.0, 65.0, false, true, &mut out);
        assert_eq!(t.events.len(), 1, "a press where it is placed reaches it");
        t.take_events(&mut Vec::new());
        // Behind the camera, and in front but off the surface: hidden.
        for at in [None, Some([-10.0, 60.0])] {
            place(&mut t, at);
            assert!(!node(&t, 2).1, "{at:?}");
            assert!(!t.a11y_snapshot().0.contains_key(&two), "{at:?}");
            t.pointer(130.0, 65.0, true, false, &mut out);
            t.pointer(130.0, 65.0, false, true, &mut out);
            assert!(t.events.is_empty(), "{at:?}");
        }
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
        t.set_textures(Box::new(|h| if h == 4294967298.0 { 8 } else { 0 }));
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
    /// 0.2 s, linearly, in from and out to transparent.
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
        let mut out = Vec::new();
        t.pointer(75.0, 10.0, true, false, &mut out);
        t.pointer(75.0, 10.0, false, true, &mut out);
        assert!(t.events.is_empty(), "an exiting node answers nothing");
        t.set_time(1.7, false);
        frame(&mut t);
        assert_eq!(alpha_at(&t, 50.0), None, "out once its exit settled");
        assert_eq!(t.op_box(3), None);
        assert!(t.leaving.is_empty() && t.els.iter().filter(|e| e.js == 4).all(|e| e.dead));
        // Nothing moves: the next frame paints nothing.
        t.set_time(1.8, false);
        let (_, painted) = t.present(&og_ui::Fonts::default(), &IconAtlas::default());
        assert!(!painted);
    }

    /// The ops' tags, a11y props and listeners reach the accessibility
    /// tree: a pressed view is a button named by its text, at its box; a
    /// A node under a transform (a dialog scaling in) is where it is
    /// drawn in the accessibility tree, not at its layout box.
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
        t.pointer(x, y, pressed, released, &mut Vec::new());
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
    /// again when one comes.
    #[test]
    fn a_glyph_shows_its_actions_binding() {
        let mut t = stated();
        let mut b = Batch::default();
        b.op(2).u32(1).str("");
        b.op(6).u32(1).str("glyph").str("use");
        b.op(8).u32(0).u32(1).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        assert!(t.glyphs_wanted());
        t.set_glyphs(&|a| (a == "use").then(|| "E".to_string()));
        assert!(!t.glyphs_wanted());
        let e = t.op(1).unwrap();
        assert_eq!(t.els[e as usize].text, "E");
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
        assert!(t.nav(og_ui::Nav::Down), "the d-pad still moves it");
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
                    _ => {}
                }
            }
            Record {
                layout: rune_taffy::CompactStyle::from(&s),
                ..Record::default()
            }
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
        use og_text::text::{FontMetrics, UiFont};
        let fonts = og_ui::Fonts {
            body: UiFont {
                handle: Some(og_draw::FontHandle::from_index(0)),
                metrics: FontMetrics::fixed(0.5),
            },
            ..Default::default()
        };
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
        t.frame(&fonts, &IconAtlas::default());
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
        // The caret is after "a": half an em (the class's 14 px) in, at the
        // fixed face's advance.
        let caret = |t: &mut Tree| {
            t.frame(&fonts, &IconAtlas::default());
            let x0 = t.op_box(1).unwrap()[0];
            t.drawn
                .quads
                .iter()
                .find(|q| q.kind == og_draw::paint::PaintKind::Solid as u32 && q.rect[2] <= 2.0)
                .map(|q| q.rect[0] - x0)
        };
        assert_eq!(caret(&mut t), Some(7.0));
        t.key("Home", false, false, false, false);
        assert_eq!(caret(&mut t), Some(0.0));
        let mut b = Batch::default();
        b.op(6).u32(1).str("value").str("Zed");
        t.apply(&b.bytes()).unwrap();
        assert_eq!(
            caret(&mut t),
            Some(21.0),
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

    /// The arrows move among a focus group's members first: at its edge
    /// one that stops leaves for the control beyond, one that wraps goes
    /// round inside itself, and focus coming into one that remembers lands
    /// on the member it last had.
    #[test]
    fn focus_groups_stop_wrap_and_remember() {
        use og_ui::Nav;
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

    /// A character the body face lacks draws from the face its mixed font
    /// falls back to (a CJK pack), in that face's own paint slot, which
    /// the paint lists with the face's handle; the body's stay in slot 0.
    #[test]
    fn a_character_the_body_lacks_draws_from_the_fallback_face_linked_for_it() {
        use og_text::text::{FontMetrics, UiFont};
        let face = |h: usize, m: FontMetrics| UiFont {
            handle: Some(og_draw::FontHandle::from_index(h)),
            metrics: m,
        };
        let body = face(0, FontMetrics::fixed(0.5).with_range(&[(0, 0x7f)]));
        let cjk = face(5, FontMetrics::fixed(1.0));
        let mut mixed = body.clone();
        mixed.metrics.runs = Some(vec![cjk].into());
        let mut fonts = og_ui::Fonts {
            body,
            ..Default::default()
        };
        let mut t = tree();
        let mut b = Batch::default();
        b.op(2).u32(1).str("A中");
        b.op(8).u32(0).u32(1).u32(u32::MAX);
        t.apply(&b.bytes()).unwrap();
        let glyphs = |t: &Tree| -> Vec<_> {
            t.drawn
                .quads
                .iter()
                .filter(|q| q.kind == og_draw::paint::PaintKind::Glyph as u32)
                .map(|q| (char::from_u32(q.a).unwrap(), q.b))
                .collect()
        };
        t.frame(&fonts, &IconAtlas::default());
        assert_eq!(glyphs(&t), [('A', 0)]);
        // No face has it yet: reported once, for the host to load a pack.
        assert_eq!(t.take_missing(), ['中']);
        // A locale links the face later, with nothing in the tree changed:
        // the text is measured and drawn again in it.
        fonts.body.metrics.mixed = Some(std::sync::Arc::new(mixed));
        t.frame(&fonts, &IconAtlas::default());
        assert_eq!(glyphs(&t), [('A', 0), ('中', 2)]);
        assert!(t.take_missing().is_empty());
        let five = og_draw::FontHandle::from_index(5);
        assert_eq!(t.drawn.fonts[2].first(), Some(&five));
    }
}
