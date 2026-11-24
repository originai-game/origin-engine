//! Paragraphs: a `<Text>` whose content is more than one run is laid out
//! as one text, as CSS lays out an inline formatting context. Its text
//! runs, the nested `<Text>` spans around them and the `<Icon>` and
//! `<Image>` in it become one og-text layout on a leaf of the paragraph's
//! own: each run's span in its own resolved style ([`RichSpan`]), each
//! icon or image an inline box ([`RichBox`]). They wrap, align and break
//! together, and a span's glyphs draw in its ink.
//!
//! The members keep their elements, so their classes, events and states
//! work as they do elsewhere: a run takes no box of its own, a span stands
//! at the paragraph's corner without size, and an icon or image is drawn
//! where its inline box was laid out. A span that answers a press is hit
//! on its glyphs ([`Tree::on_span`]): a link in a sentence.
//!
//! A `select-text` text can be selected with the pointer, read only: a
//! press places nothing, a drag selects from there, a second press in a
//! row selects the word and a third the whole text; Control or Command +
//! C copies it ([`Tree::take_copy`]) and + A selects it whole.

use super::{El, Kind, NONE, Tree};
use crate::arena::Id;
use crate::text::{Rich, RichBox, RichSpan};

/// A paragraph's members, in their order, and where each lies in its
/// text: a run's and a span's bytes, an icon's or an image's place.
#[derive(Clone, Debug, Default)]
pub(super) struct Para {
    pub members: Vec<u32>,
    pub ranges: Vec<[u32; 2]>,
}

/// A selection in a `select-text` text: the element whose leaf holds it,
/// where it started and where it ends, bytes of the leaf's laid-out text.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Selected {
    pub e: u32,
    pub anchor: usize,
    pub head: usize,
}

/// A press in a selectable text: its element, time, place and how many
/// presses in a row; whether the pointer is still down selecting, and
/// whether it is down at all (a finger waiting to long-press).
#[derive(Clone, Copy, Debug)]
pub(super) struct SelectPress {
    pub(super) e: u32,
    at: f64,
    place: (f32, f32),
    count: u8,
    pub(super) held: bool,
    down: bool,
}

/// Whether `kind` can be in a paragraph's line.
fn inline(kind: Kind) -> bool {
    matches!(kind, Kind::Label | Kind::Text | Kind::Icon | Kind::Image)
}

impl Tree {
    /// The paragraph `e` is in or is: its outermost `<Text>` through a
    /// chain of texts, runs, icons and images; `NONE` outside one.
    pub(super) fn para_root(&self, e: u32) -> u32 {
        let (mut root, mut c) = (NONE, e);
        while c != NONE {
            let el = &self.els[c as usize];
            if !inline(el.kind) {
                break;
            }
            if el.kind == Kind::Text {
                root = c;
            }
            c = el.parent;
        }
        root
    }

    /// `e` or what is in it changed: its paragraph lays out again.
    pub(super) fn para_touch(&mut self, e: u32) {
        if self.composing || e == NONE {
            return;
        }
        let root = self.para_root(e);
        if root != NONE && !self.els[root as usize].queued {
            self.els[root as usize].queued = true;
            self.paras.push(root);
        }
    }

    /// Lays out again each paragraph something changed in.
    pub(super) fn compose_paras(&mut self) {
        if self.paras.is_empty() {
            return;
        }
        self.order();
        self.composing = true;
        let roots = std::mem::take(&mut self.paras);
        for &root in &roots {
            self.els[root as usize].queued = false;
            if !self.els[root as usize].dead {
                self.make_para(root);
            }
        }
        self.paras = roots;
        self.paras.clear();
        self.composing = false;
    }

    /// Makes `root`'s paragraph from its members, or takes it apart when
    /// its content is one run (that run lays itself out) or holds a box
    /// that is not inline (it lays out as boxes, said once).
    fn make_para(&mut self, root: u32) {
        // A text of one run, as most are, stays as it is.
        let el = &self.els[root as usize];
        if el.para.is_none()
            && let [only] = self.arena.children(el.node)
            && let Some(&c) = self.el_of.get(*only as usize)
            && c != NONE
            && self.els[c as usize].kind == Kind::Label
            && self.els[c as usize].inline == NONE
        {
            return;
        }
        let mut para = Para::default();
        let mut text = String::new();
        let mut rich = Rich::default();
        let mut blocked = NONE;
        self.gather(
            self.els[root as usize].node,
            &mut para,
            &mut text,
            &mut rich,
            &mut blocked,
            false,
        );
        let one_run =
            para.members.len() == 1 && self.els[para.members[0] as usize].kind == Kind::Label;
        let is = blocked == NONE && !para.members.is_empty() && !one_run;
        if blocked != NONE && !self.warned_paras.contains(&root) {
            self.warned_paras.push(root);
            let name = &self.input.per[root as usize].name;
            self.warnings.push(format!(
                "node {}{}{} is a <Text> holding a box that is not text, an icon or an image: its content lays out as boxes, not as one paragraph",
                self.els[root as usize].node,
                if name.is_empty() { "" } else { " " },
                name,
            ));
        }
        let old = self.els[root as usize]
            .para
            .take()
            .map(|p| p.members)
            .unwrap_or_default();
        let node = self.els[root as usize].node;
        if is {
            let leaf = match self.els[root as usize].leaf {
                Some(leaf) => {
                    self.arena.attach(leaf, node, None);
                    leaf
                }
                None => {
                    let leaf = self.arena.text(node, "", 16.0);
                    self.els[root as usize].leaf = Some(leaf);
                    leaf
                }
            };
            if self.texts.set_rich(leaf, &rich) {
                self.arena.remeasure_one(leaf);
                self.paint = true;
            }
            self.arena.set_text(leaf, &text);
            let el = &mut self.els[root as usize];
            el.text.clear();
            el.text.push_str(&text);
            self.inherit(root);
        } else if let Some(leaf) = self.els[root as usize].leaf {
            self.arena.detach(leaf);
            self.texts.set_rich(leaf, &Rich::default());
            self.els[root as usize].text.clear();
        }
        // What joined or left lays out as it now is.
        for &m in &old {
            if !para.members.contains(&m) && !self.els[m as usize].dead {
                self.els[m as usize].inline = NONE;
                self.inherit(m);
            }
        }
        if is {
            for &m in &para.members {
                if self.els[m as usize].inline != root {
                    self.els[m as usize].inline = root;
                    self.inherit(m);
                }
                // A span's ruby is the text of a leaf of its own.
                let el = &self.els[m as usize];
                if el.kind == Kind::Text && (!el.ruby.is_empty() || el.leaf.is_some()) {
                    let (node, ruby) = (el.node, el.ruby.clone());
                    let leaf = match el.leaf {
                        Some(leaf) => leaf,
                        None => {
                            let leaf = self.arena.text(node, "", 16.0);
                            self.els[m as usize].leaf = Some(leaf);
                            self.inherit(m);
                            leaf
                        }
                    };
                    self.arena.set_text(leaf, &ruby);
                    let el = &mut self.els[m as usize];
                    el.text.clear();
                    el.text.push_str(&ruby);
                }
            }
            self.els[root as usize].para = Some(Box::new(para));
        } else {
            for &m in &para.members {
                if self.els[m as usize].inline != NONE {
                    self.els[m as usize].inline = NONE;
                    self.inherit(m);
                }
            }
        }
        if self
            .select
            .is_some_and(|s| s.e == root || old.contains(&s.e))
        {
            self.select = None;
        }
    }

    /// Adds the members under arena node `node` to `para`, their text to
    /// `text` and their spans and boxes to `rich`; `blocked` takes the
    /// first one that cannot be in a line.
    fn gather(
        &self,
        node: Id,
        para: &mut Para,
        text: &mut String,
        rich: &mut Rich,
        blocked: &mut u32,
        ruby: bool,
    ) {
        for &n in self.arena.children(node) {
            let e = self.el_of.get(n as usize).copied().unwrap_or(NONE);
            if e == NONE {
                continue;
            }
            let el = &self.els[e as usize];
            if el.dead || el.hidden > 0 || el.record.hidden {
                continue;
            }
            if !inline(el.kind) {
                if *blocked == NONE {
                    *blocked = e;
                }
                continue;
            }
            let ink = para.members.len() as u16 + 1;
            let start = text.len() as u32;
            para.members.push(e);
            para.ranges.push([start, start]);
            let k = para.ranges.len() - 1;
            match el.kind {
                Kind::Label => {
                    let Some((drawn, run)) = el.leaf.and_then(|l| self.arena.text_run(l).as_ref())
                    else {
                        continue;
                    };
                    text.push_str(drawn);
                    let mut run = *run;
                    // Under a ruby its line is taller by the annotation
                    // above it and as much below, so it sits centred.
                    if ruby {
                        let line = if run.line < 0.0 {
                            run.size * 1.2
                        } else {
                            run.line
                        };
                        run.line = line + 2.0 * super::RUBY * run.size;
                    }
                    rich.spans.push(RichSpan {
                        start,
                        end: text.len() as u32,
                        run,
                        ink,
                    });
                }
                Kind::Text => self.gather(
                    el.node,
                    para,
                    text,
                    rich,
                    blocked,
                    ruby || !el.ruby.is_empty(),
                ),
                _ => {
                    let [w, h] = self.inline_size(el);
                    rich.boxes.push(RichBox {
                        at: start,
                        w,
                        h,
                        id: e,
                    });
                }
            }
            para.ranges[k][1] = text.len() as u32;
        }
    }

    /// An icon's or an image's size in a line: its own width and height
    /// when its classes give them in px, else an em square.
    fn inline_size(&self, el: &El) -> [f32; 2] {
        let s = taffy::Style::from(&el.record.layout);
        let em = el.eff.size * self.text_scale;
        let w = s.size.width.into_option();
        let h = s.size.height.into_option();
        [w.or(h).unwrap_or(em), h.or(w).unwrap_or(em)]
    }

    /// A member's layout in its paragraph, over `s` (its record's): a run
    /// takes no box (its text is the paragraph's), a span stands at the
    /// paragraph's corner without size, and an icon or image keeps its
    /// size out of the flow, drawn where its paragraph put it.
    pub(super) fn member_style(kind: Kind, s: &mut taffy::Style) {
        use taffy::{LengthPercentage as P, LengthPercentageAuto as A};
        match kind {
            Kind::Label => s.display = taffy::Display::None,
            _ => {
                s.position = taffy::Position::Absolute;
                s.inset = taffy::Rect {
                    left: A::length(0.0),
                    top: A::length(0.0),
                    right: A::auto(),
                    bottom: A::auto(),
                };
                s.margin = taffy::Rect::length(0.0);
                if kind == Kind::Text {
                    s.size = taffy::Size::from_lengths(0.0, 0.0);
                    s.min_size = taffy::Size::auto();
                    s.padding = taffy::Rect::length(0.0);
                    s.border = taffy::Rect {
                        left: P::length(0.0),
                        right: P::length(0.0),
                        top: P::length(0.0),
                        bottom: P::length(0.0),
                    };
                }
            }
        }
    }

    /// The surface point paragraph `root`'s text starts at, from its
    /// painted place, and its leaf.
    fn para_origin(&self, root: u32) -> Option<(Id, f32, f32)> {
        let leaf = self.els[root as usize].leaf?;
        let &(x, y, ..) = self.abs.get(root as usize)?;
        let lb = self.arena.box_of(leaf);
        Some((leaf, x + lb.x, y + lb.y))
    }

    /// Where inline member `e` (an icon or an image) is drawn: its inline
    /// box on the surface, as its paragraph laid it out; `None` when a
    /// cut line dropped it.
    pub(super) fn inline_box(&self, e: u32) -> Option<[f32; 2]> {
        let (leaf, ox, oy) = self.para_origin(self.els[e as usize].inline)?;
        let g = self.texts.leaf(leaf)?;
        let &(_, [x, y, ..]) = g.boxes.iter().find(|b| b.0 == e)?;
        Some([ox + x, oy + y])
    }

    /// Where span member `e`'s ruby is drawn: its box's place, so that
    /// its leaf stands centred over the span's first piece, its foot just
    /// over the em above the piece's middle (CJK glyphs fill the em); `None` when the span is not laid
    /// out.
    pub(super) fn ruby_at(&self, e: u32) -> Option<[f32; 2]> {
        let el = &self.els[e as usize];
        let [x, y, w, h] = self.span_piece(e)?;
        let leaf = self.arena.box_of(el.leaf?);
        let base = el.eff.size * self.text_scale;
        let middle = y + h / 2.0;
        Some([
            x + (w - leaf.w) / 2.0 - leaf.x,
            middle - base * 0.56 - leaf.h - leaf.y,
        ])
    }

    /// Whether (x, y) is on span member `e`'s glyphs (or an icon or image
    /// in it); `None` when `e` is no span of a paragraph.
    pub(super) fn on_span(&self, e: u32, x: f32, y: f32) -> Option<bool> {
        let el = &self.els[e as usize];
        if el.inline == NONE || el.kind != Kind::Text {
            return None;
        }
        let root = el.inline;
        let (leaf, ox, oy) = self.para_origin(root)?;
        let para = self.els[root as usize].para.as_ref()?;
        let k = para.members.iter().position(|&m| m == e)?;
        let [start, end] = para.ranges[k];
        let g = self.texts.leaf(leaf)?;
        let Some((ux, uy)) = self.untransform(root, x, y) else {
            return Some(false);
        };
        let (px, py) = (ux - ox, uy - oy);
        let mut on = false;
        g.selection(start as usize..end as usize, |[sx, sy, w, h]| {
            on |= px >= sx && px < sx + w && py >= sy && py < sy + h;
        });
        // Its icons and images are in it too.
        on |= para.members[k + 1..]
            .iter()
            .zip(&para.ranges[k + 1..])
            .take_while(|(_, r)| r[0] < end || (r[0] == end && r[1] == end))
            .filter(|(m, _)| matches!(self.els[**m as usize].kind, Kind::Icon | Kind::Image))
            .any(|(&m, _)| {
                g.boxes
                    .iter()
                    .find(|b| b.0 == m)
                    .is_some_and(|&(_, [bx, by, w, h])| {
                        px >= bx && px < bx + w && py >= by && py < by + h
                    })
            });
        Some(on)
    }

    /// Span member `e`'s first piece on screen (its glyphs on the first
    /// line it is on), where a press on it lands: its box for the probe.
    pub(super) fn span_piece(&self, e: u32) -> Option<[f32; 4]> {
        let el = &self.els[e as usize];
        if el.inline == NONE || el.kind != Kind::Text {
            return None;
        }
        let (leaf, ox, oy) = self.para_origin(el.inline)?;
        let para = self.els[el.inline as usize].para.as_ref()?;
        let k = para.members.iter().position(|&m| m == e)?;
        let [start, end] = para.ranges[k];
        let mut first = None;
        self.texts
            .leaf(leaf)?
            .selection(start as usize..end as usize, |[x, y, w, h]| {
                first.get_or_insert([ox + x, oy + y, w, h]);
            });
        first
    }

    /// The element whose text a press at (x, y) selects in: the topmost
    /// `select-text` text with a leaf of its own (a run, or a paragraph)
    /// under it.
    fn selectable_at(&self, x: f32, y: f32) -> u32 {
        self.topmost(
            x,
            y,
            |el| {
                matches!(el.kind, Kind::Label | Kind::Text)
                    && el.leaf.is_some()
                    && el.inline == NONE
                    && el.eff.text.select
                    && !el.text.is_empty()
            },
            false,
        )
    }

    /// Leaf of selectable `e`, its text's surface origin and the byte of
    /// its laid-out text nearest (x, y).
    fn select_hit(&self, e: u32, x: f32, y: f32) -> Option<(usize, std::ops::Range<usize>)> {
        let leaf = self.els[e as usize].leaf?;
        let &(ex, ey, ..) = self.abs.get(e as usize)?;
        let lb = self.arena.box_of(leaf);
        let g = self.texts.leaf(leaf)?;
        let (ux, uy) = self.untransform(e, x, y)?;
        let (px, py) = (ux - ex - lb.x, uy - ey - lb.y);
        Some((g.hit(px, py), g.word_at(px, py)))
    }

    /// The pointer on the surface: a press in a `select-text` text starts
    /// a selection there (a word on a second press in a row, the whole
    /// text on a third), a held press drags it, and a press anywhere else
    /// clears it.
    pub(super) fn select_pointer(
        &mut self,
        x: f32,
        y: f32,
        moved: bool,
        pressed: bool,
        released: bool,
    ) {
        if pressed {
            let e = self.selectable_at(x, y);
            let Some((at, word)) = (e != NONE).then(|| self.select_hit(e, x, y)).flatten() else {
                if self.select.take().is_some() {
                    self.paint = true;
                }
                self.select_press = None;
                return;
            };
            let count = match self.select_press {
                Some(p)
                    if p.e == e
                        && self.time - p.at <= super::input::DOUBLE_PRESS
                        && (p.place.0 - x).abs() <= 8.0
                        && (p.place.1 - y).abs() <= 8.0 =>
                {
                    p.count % 3 + 1
                }
                _ => 1,
            };
            // A finger's first press selects nothing: a drag pans, and a
            // still finger selects the word when it long-presses.
            let touch = self.input.touch && count == 1;
            self.select_press = Some(SelectPress {
                e,
                at: self.time,
                place: (x, y),
                count,
                held: count == 1 && !touch,
                down: true,
            });
            if touch {
                if self.select.take().is_some() {
                    self.paint = true;
                }
                return;
            }
            let all = self
                .texts
                .leaf(self.els[e as usize].leaf.unwrap_or(0))
                .map_or(0, |g| g.laid().len());
            let (anchor, head) = match count {
                2 => (word.start, word.end),
                3 => (0, all),
                _ => (at, at),
            };
            self.select = Some(Selected { e, anchor, head });
            self.paint = true;
        } else if moved
            && let Some(p) = self.select_press.filter(|p| p.held)
            && let Some(s) = self.select.filter(|s| s.e == p.e)
            && let Some((at, _)) = self.select_hit(s.e, x, y)
            && at != s.head
        {
            self.select = Some(Selected { head: at, ..s });
            self.paint = true;
        }
        if released && let Some(p) = &mut self.select_press {
            p.held = false;
            p.down = false;
        }
    }

    /// A finger held still on a selectable text for a long press selects
    /// the word under it, and from then on its moves extend the selection
    /// (as `select_pointer`'s drag) until it lifts.
    pub(super) fn select_due(&mut self, x: f32, y: f32) {
        let Some(p) = self.select_press else { return };
        if !self.input.touch
            || !p.down
            || p.held
            || p.count != 1
            || self.time - p.at < super::input::LONG_PRESS
        {
            return;
        }
        let (dx, dy) = (x - p.place.0, y - p.place.1);
        let still = dx * dx + dy * dy <= super::input::TOUCH_SLOP * super::input::TOUCH_SLOP;
        let word = still
            .then(|| self.select_hit(p.e, p.place.0, p.place.1))
            .flatten();
        self.select_press = Some(SelectPress {
            held: word.is_some(),
            down: word.is_some(),
            ..p
        });
        if let Some((_, word)) = word {
            self.select = Some(Selected {
                e: p.e,
                anchor: word.start,
                head: word.end,
            });
            self.paint = true;
        }
    }

    /// Control or Command + C copies the selected text, + A selects all of
    /// it: whether the key was taken.
    pub(super) fn select_key(&mut self, code: &str, cmd: bool) -> bool {
        let Some(s) = self.select else {
            return false;
        };
        let Some(g) = self.els[s.e as usize].leaf.and_then(|l| self.texts.leaf(l)) else {
            return false;
        };
        let laid = g.laid();
        match code {
            "KeyC" if cmd => {
                let (a, b) = (s.anchor.min(s.head), s.anchor.max(s.head));
                if let Some(t) = laid.get(a..b).filter(|t| !t.is_empty()) {
                    self.input.copied = Some(t.to_owned());
                }
                true
            }
            "KeyA" if cmd => {
                let all = laid.len();
                self.select = Some(Selected {
                    anchor: 0,
                    head: all,
                    ..s
                });
                self.paint = true;
                true
            }
            _ => false,
        }
    }

    /// The selection in `e`'s text, as bytes of its laid-out text.
    pub(super) fn selected_in(&self, e: u32) -> Option<std::ops::Range<usize>> {
        let s = self.select.filter(|s| s.e == e && s.anchor != s.head)?;
        Some(s.anchor.min(s.head)..s.anchor.max(s.head))
    }
}
