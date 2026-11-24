//! Resolved class/style record caching, interaction state tracking, and
//! `data-*` context refreshes.

use super::*;

/// The states input and the accessibility props set on op nodes, which
/// their classes may read: NSH's core sets `selected:`, `checked:`,
/// `indeterminate:`, `aria-expanded:`, `aria-pressed:`, `aria-busy:` and
/// `aria-invalid:` from a view's `accessibilityState`, as ours does from
/// the props of the same names.
pub(in crate::tree) const INTERACTION: [u16; 13] = [
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
pub(in crate::tree) const GROUP_STATES: u16 = {
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
    pub(super) records: Vec<Record>,
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
    pub(in crate::tree) fn tone(&self, role: &str, other: bool) -> Rgba {
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
    pub(in crate::tree) fn tracked(&self, i: u32) -> u16 {
        self.tracked.get(i as usize).copied().unwrap_or(0)
    }

    /// Entry `i`'s record under interaction states `states` and data
    /// context `data`.
    pub(in crate::tree) fn record(&mut self, i: u32, states: u16, data: u32) -> &Record {
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
    pub(in crate::tree) fn data_id(&mut self, data: &str) -> u32 {
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
    pub(in crate::tree) fn groups(&self, i: u32) -> impl Iterator<Item = &str> {
        let class = self.keys.get(i as usize).map_or("", |k| Self::split(k).0);
        class.split_ascii_whitespace().filter_map(|c| match c {
            "group" => Some(""),
            c => c.strip_prefix("group/"),
        })
    }

    /// Entry `i`'s `starting:` (0) or `exit:` (1) record.
    pub(in crate::tree) fn presence(&self, i: u32, k: usize) -> Option<&Record> {
        self.presence[i as usize][k].as_deref()
    }

    fn split(key: &str) -> (&str, &str) {
        key.split_once('\0').unwrap_or((key, ""))
    }

    pub(in crate::tree) fn acquire(&mut self, i: u32) {
        if let Some(r) = self.refs.get_mut(i as usize) {
            *r = r.saturating_add(1);
        }
    }

    pub(in crate::tree) fn release(&mut self, i: u32) {
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

    pub(super) fn get(&mut self, class: &str, style: &str) -> u32 {
        self.buf.clear();
        self.buf.push_str(class);
        self.buf.push('\0');
        self.buf.push_str(style);
        self.lookup()
    }

    /// Entry `old` with its class (true) or its style replaced.
    pub(super) fn with(&mut self, old: u32, class: bool, text: &str) -> u32 {
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

    pub(super) fn reresolve(&mut self) {
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
    pub(super) fn other_scheme(&self, e: u32) -> bool {
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

    pub(in crate::tree) fn restyle(&mut self, e: u32) {
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
    pub(super) fn swap_class(&mut self, e: u32, new: u32) {
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
    pub(in crate::tree) fn field_class(&mut self, e: u32) {
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

    pub(in crate::tree) fn presses(&self, e: u32) -> bool {
        self.els[e as usize].clickable
            || self
                .input
                .per(e)
                .is_some_and(|p| matches!(p.zone, Some((og_input::ZoneKind::Button, _))))
    }

    /// Works out each op node's data context again when a batch changed a
    /// `data-*` prop, a class or the tree's shape: its own `data-*` names,
    /// then each of its nearest ancestor groups' as `group/name`, and
    /// restyles the nodes whose context changed.
    pub(in crate::tree) fn refresh_data(&mut self) {
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
}
