//! The TS renderer's ops applied to the arena (CONVENTIONS.md §Ops): a
//! binary batch read in place, each op patching the element it names. An
//! op node is an element of the tree like a document's, so layout, text,
//! icons and paint serve it unchanged; its record comes from its class and
//! style through the resolver, once per distinct pair.

use super::*;

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
}

impl Event {
    const ALL: [Event; 10] = [
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
    ];

    pub fn bit(self) -> u16 {
        1 << self as u16
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

/// A class string and inline style to their record.
pub type Resolve = Box<dyn Fn(&str, &str) -> Record>;

/// The records of the class and style pairs the op nodes use, each
/// resolved once; an entry's key is `class\0style`.
#[derive(Default)]
pub struct Classes {
    index: FxHashMap<Box<str>, u32>,
    keys: Vec<Box<str>>,
    records: Vec<Record>,
    buf: String,
    resolve: Option<Resolve>,
}

impl Classes {
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
        let record = self
            .resolve
            .as_ref()
            .map_or_else(Record::default, |f| f(c, s));
        let i = self.keys.len() as u32;
        self.records.push(record);
        self.keys.push(self.buf.as_str().into());
        self.index.insert(self.buf.as_str().into(), i);
        i
    }

    fn reresolve(&mut self) {
        for (k, r) in self.keys.iter().zip(&mut self.records) {
            let (c, s) = Self::split(k);
            *r = self
                .resolve
                .as_ref()
                .map_or_else(Record::default, |f| f(c, s));
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

    fn restyle(&mut self, e: u32) {
        let class = self.els[e as usize].class;
        let record = &self.classes.records[class as usize];
        if self.els[e as usize].record != *record {
            self.els[e as usize].record = record.clone();
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
            eff: self.root,
        });
        if self.by_js.len() <= id as usize {
            self.by_js.resize(id as usize + 1, NONE);
        }
        self.by_js[id as usize] = e;
        self.inherit(e);
        Ok(e)
    }

    /// Frees element `e` and every element under it.
    fn release(&mut self, e: u32) {
        let el = &mut self.els[e as usize];
        el.dead = true;
        let (node, js) = (el.node, el.js);
        self.arena.detach(node);
        if let Some(slot) = self.by_js.get_mut(js as usize) {
            *slot = NONE;
        }
        for c in 0..self.els.len() as u32 {
            let el = &self.els[c as usize];
            if el.parent == e && !el.dead {
                self.release(c);
            }
        }
    }

    /// Applies an op batch (after [`MAGIC`]) in order. A refused op stops
    /// the batch; the ones before it stay applied.
    pub fn apply(&mut self, bytes: &[u8]) -> Result<(), String> {
        let mut r = Reader { b: bytes, at: 0 };
        let count = r.u32()?;
        for _ in 0..count {
            let code = r.u8()?;
            match code {
                // create
                1 => {
                    let id = r.u32()?;
                    let kind = match r.u8()? {
                        5 => Kind::Icon,
                        0..=6 => Kind::Box,
                        t => return Err(format!("og.ui ops: unknown tag {t}")),
                    };
                    self.create(id, kind, false)?;
                }
                // createText
                2 => {
                    let id = r.u32()?;
                    let text = r.str()?;
                    let e = self.create(id, Kind::Label, true)?;
                    self.set_text(e, text);
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
                    self.restyle(e);
                }
                // setProp
                6 => {
                    let e = self.op(r.u32()?)?;
                    let (name, value) = (r.str()?, r.str()?);
                    match name {
                        "disabled" => {
                            let on = !matches!(value, "" | "false" | "0");
                            match self.els[e as usize].disabled {
                                NONE => self.els[e as usize].disabled = queue::signal(on.into()),
                                s => queue::set(s, on.into()),
                            }
                        }
                        "name" if self.els[e as usize].kind == Kind::Icon => {
                            self.set_text(e, value);
                        }
                        // The rest are read by their own heads: `data-*`
                        // by class variants, roles and labels by a11y.
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
                    self.ordered = false;
                    self.paint = true;
                    self.reapply(child);
                }
                // remove
                9 => {
                    let _parent = r.u32()?;
                    let child = self.op(r.u32()?)?;
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
                    self.els[e as usize].hidden = 1;
                    self.inherit(e);
                }
                c => return Err(format!("og.ui ops: unknown op {c}")),
            }
        }
        if r.at != bytes.len() {
            return Err("og.ui ops: bytes after the last op".into());
        }
        Ok(())
    }

    /// Writes the events since the last call, after [`EVENTS`]: a u32
    /// count, then each one's id (u32), event (u8), x and y (f32), key and
    /// value (u32 length and UTF-8).
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

impl Tree {
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
        t.set_resolver(Box::new(|class, _| {
            let mut s = taffy::Style::default();
            for c in class.split_whitespace() {
                match c {
                    "row" => s.flex_direction = taffy::FlexDirection::Row,
                    "b" => s.size = taffy::Size::from_lengths(50.0, 20.0),
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
}
