//! Fragment links (HTML's "scroll to the fragment", URL Fragment Text
//! Directives): a document link's `#id` scrolls the element with that id to
//! the start of its scroll containers; `#id:~:text=[prefix-,]start[,end]
//! [,-suffix]` finds its passage in the laid-out text, marks every match
//! for `::target-text`, scrolls the first to the middle of its scroll
//! containers and makes the element holding it `:target` (the id's
//! element when nothing matches).
//!
//! The search reads the text as drawn: each text element's laid-out lines,
//! a wrapped line's break as its source had it. A term (prefix, start, end
//! or suffix) matches inside one element and starts and ends on word
//! boundaries (UAX #29) as the spec bounds it; a context term may sit in
//! the element before or after, with only white space between. Matching
//! ignores case and accents ([`og_i18n::search_fold`]) and runs of white
//! space. A match is a selection's pair of ends ([`SelectEnd`]): the
//! frame marks each element's range of it as it marks the selection.

use super::select::SelectEnd;
use super::*;

/// One `text=` directive, its terms percent-decoded.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TextDirective {
    pub prefix: Option<String>,
    pub start: String,
    pub end: Option<String>,
    pub suffix: Option<String>,
}

/// A link's fragment (after `#`): the id it names (empty for none) and
/// the text directives after its `:~:` delimiter, in order; a malformed
/// directive is skipped, as is any directive but `text=`.
pub(crate) fn parse_fragment(fragment: &str) -> (String, Vec<TextDirective>) {
    let Some((id, directives)) = fragment.split_once(":~:") else {
        return (fragment.to_string(), Vec::new());
    };
    let parsed = directives
        .split('&')
        .filter_map(|d| d.strip_prefix("text="))
        .filter_map(parse_directive)
        .collect();
    (id.to_string(), parsed)
}

/// `[prefix-,]start[,end][,-suffix]`, or `None` when malformed (an empty
/// term, a stray `-`, more than two match terms).
fn parse_directive(value: &str) -> Option<TextDirective> {
    let mut tokens: Vec<&str> = value.split(',').collect();
    let prefix = match tokens.first() {
        Some(t) if t.ends_with('-') => {
            let p = &t[..t.len() - 1];
            tokens.remove(0);
            Some(p)
        }
        _ => None,
    };
    let suffix = match tokens.last() {
        Some(t) if t.starts_with('-') && tokens.len() > 1 => {
            let s = &t[1..];
            tokens.pop();
            Some(s)
        }
        _ => None,
    };
    if tokens.is_empty() || tokens.len() > 2 {
        return None;
    }
    let term = |t: &str| -> Option<String> {
        if t.is_empty() || t.contains('-') {
            return None;
        }
        decode(t)
    };
    let optional = |t: Option<&str>| match t {
        Some(t) => term(t).map(Some),
        None => Some(None),
    };
    Some(TextDirective {
        prefix: optional(prefix)?,
        start: term(tokens[0])?,
        end: optional(tokens.get(1).copied())?,
        suffix: optional(suffix)?,
    })
}

/// Percent-decoding of a term as UTF-8; `None` when it is not UTF-8.
fn decode(t: &str) -> Option<String> {
    let b = t.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut k = 0;
    while k < b.len() {
        let hex = |c: u8| (c as char).to_digit(16);
        match (b[k], b.get(k + 1).copied(), b.get(k + 2).copied()) {
            (b'%', Some(h), Some(l)) if hex(h).is_some() && hex(l).is_some() => {
                out.push((hex(h)? * 16 + hex(l)?) as u8);
                k += 3;
            }
            (c, ..) => {
                out.push(c);
                k += 1;
            }
        }
    }
    String::from_utf8(out).ok()
}

/// One character of the searched text, folded: the element (tree index;
/// `NONE` between elements) and the character of its marks it came from,
/// and whether a word boundary is before it.
#[derive(Clone, Copy)]
struct Unit {
    c: char,
    node: usize,
    at: u32,
    word: bool,
}

/// The document's text as a search reads it, in the trees' order.
struct Corpus {
    units: Vec<Unit>,
}

impl Corpus {
    fn of(tree: &crate::ui::Tree) -> Corpus {
        let mut units: Vec<Unit> = Vec::new();
        for &i in &tree.orders.pre {
            let Some((source, lines)) = tree.text_source(i) else {
                continue;
            };
            let seps = super::select::line_breaks(&source, &lines);
            // Its characters as read, each with the mark it stands for:
            // a wrapped line's break, when its source had one, is the
            // break's character.
            let mut read: Vec<(char, u32)> = Vec::new();
            let mut at = 0u32;
            for (k, line) in lines.iter().enumerate() {
                for c in line.text.chars() {
                    read.push((c, at));
                    at += 1;
                }
                if k + 1 < lines.len() {
                    read.extend(seps[k].chars().map(|c| (c, at)));
                    at += 1;
                }
            }
            if read.iter().all(|(c, _)| c.is_whitespace()) {
                continue;
            }
            let chars: Vec<char> = read.iter().map(|r| r.0).collect();
            let mut bounds = vec![false; chars.len() + 1];
            for s in og_i18n::word_segments(&chars) {
                bounds[s.start] = true;
                bounds[s.end] = true;
            }
            if units.last().is_some_and(|u| !u.c.is_whitespace()) {
                units.push(Unit {
                    c: ' ',
                    node: NONE,
                    at: 0,
                    word: true,
                });
            }
            let first = units.len();
            for (k, &(c, at)) in read.iter().enumerate() {
                if c.is_whitespace() {
                    if units.last().is_some_and(|u| u.c == ' ') {
                        continue;
                    }
                    units.push(Unit {
                        c: ' ',
                        node: i,
                        at,
                        word: bounds[k],
                    });
                    continue;
                }
                for (j, f) in og_i18n::search_fold(c).chars().enumerate() {
                    units.push(Unit {
                        c: f,
                        node: i,
                        at,
                        word: j == 0 && bounds[k],
                    });
                }
            }
            // Its end is a boundary: the next element starts afresh.
            if units.len() > first {
                units.push(Unit {
                    c: ' ',
                    node: NONE,
                    at: 0,
                    word: true,
                });
            }
        }
        Corpus { units }
    }

    /// Whether a word boundary is before position `p` (an element's edge is one).
    fn bounded(&self, p: usize) -> bool {
        let u = &self.units;
        p == 0 || p >= u.len() || u[p].word || u[p].node != u[p - 1].node
    }

    /// Whether `q` matches at `p`, inside one element and on the word
    /// boundaries asked for; its end.
    fn at(&self, q: &[char], p: usize, start: bool, end: bool) -> Option<usize> {
        let u = &self.units;
        let e = p + q.len();
        if q.is_empty() || e > u.len() {
            return None;
        }
        let node = u[p].node;
        let same = node != NONE
            && u[p..e]
                .iter()
                .zip(q)
                .all(|(x, c)| x.node == node && x.c == *c);
        (same && (!start || self.bounded(p)) && (!end || self.bounded(e))).then_some(e)
    }

    /// The first match of `q` at or after `from`: its start and end.
    fn find(&self, q: &[char], from: usize, start: bool, end: bool) -> Option<(usize, usize)> {
        (from..self.units.len()).find_map(|p| self.at(q, p, start, end).map(|e| (p, e)))
    }

    /// The first position at or after `p` that is not white space.
    fn skip_space(&self, mut p: usize) -> usize {
        while p < self.units.len() && self.units[p].c == ' ' {
            p += 1;
        }
        p
    }

    /// The spec's "find a range from a text directive": the first passage
    /// that matches, as positions `[start, end)`.
    fn search(&self, d: &TextDirective) -> Option<(usize, usize)> {
        let fold = |t: &str| -> Vec<char> {
            let mut out: Vec<char> = Vec::new();
            for c in t.chars() {
                if c.is_whitespace() {
                    if out.last() != Some(&' ') {
                        out.push(' ');
                    }
                } else {
                    out.extend(og_i18n::search_fold(c).chars());
                }
            }
            out
        };
        let (prefix, start) = (d.prefix.as_deref().map(fold), fold(&d.start));
        let (end, suffix) = (d.end.as_deref().map(fold), d.suffix.as_deref().map(fold));
        let start_ends_word = end.is_some() || suffix.is_none();
        let mut from = 0;
        while from < self.units.len() {
            let found = match &prefix {
                Some(prefix) => {
                    let (p, e) = self.find(prefix, from, true, false)?;
                    from = p + 1;
                    let s = self.skip_space(e);
                    self.at(&start, s, false, start_ends_word).map(|e| (s, e))
                }
                None => {
                    let m = self.find(&start, from, true, start_ends_word)?;
                    from = m.0 + 1;
                    Some(m)
                }
            };
            let Some(mut m) = found else {
                continue;
            };
            let mut end_from = m.1;
            loop {
                if let Some(end) = &end {
                    let (p, e) = self.find(end, end_from, true, suffix.is_none())?;
                    m.1 = e;
                    end_from = p + 1;
                }
                let Some(suffix) = &suffix else {
                    return Some(m);
                };
                if self.at(suffix, self.skip_space(m.1), false, true).is_some() {
                    return Some(m);
                }
                if end.is_none() {
                    break;
                }
            }
        }
        None
    }

    /// Positions `[a, b)` as a selection's ends.
    fn ends(&self, tree: &crate::ui::Tree, (a, b): (usize, usize)) -> (SelectEnd, SelectEnd) {
        let (first, last) = (&self.units[a], &self.units[b - 1]);
        (
            (tree.nodes[first.node].key, first.at),
            (tree.nodes[last.node].key, last.at + 1),
        )
    }
}

impl crate::ui::Tree {
    /// Each directive's first passage, in the directives' order: the
    /// ranges a link's text fragment marks (none for a directive that
    /// matches nothing).
    pub(crate) fn find_text(&self, directives: &[TextDirective]) -> Vec<(SelectEnd, SelectEnd)> {
        let corpus = Corpus::of(self);
        directives
            .iter()
            .filter_map(|d| corpus.search(d))
            .map(|m| corpus.ends(self, m))
            .collect()
    }

    /// The elements and their ranges a list of passages covers, in the
    /// trees' order: every text element from each passage's first end to
    /// its last.
    pub(crate) fn passage_ranges(
        &self,
        passages: &[(SelectEnd, SelectEnd)],
    ) -> Vec<(usize, Vec<(u32, u32)>)> {
        let o = &self.orders;
        let mut out: Vec<(usize, Vec<(u32, u32)>)> = Vec::new();
        for &((ka, a), (kb, b)) in passages {
            let (Some(ia), Some(ib)) = (self.index.find(ka), self.index.find(kb)) else {
                continue;
            };
            let (pa, pb) = (o.pre_pos[ia], o.pre_pos[ib]);
            if pa == u32::MAX || pb == u32::MAX || pa > pb {
                continue;
            }
            for pos in pa..=pb {
                let i = o.pre[pos as usize];
                let Some((_, lines)) = self.text_source(i) else {
                    continue;
                };
                let n = lines
                    .iter()
                    .map(|l| l.text.chars().count() as u32 + 1)
                    .sum::<u32>()
                    .saturating_sub(1);
                let from = if pos == pa { a.min(n) } else { 0 };
                let to = if pos == pb { b.min(n) } else { n };
                if from >= to {
                    continue;
                }
                match out.iter_mut().find(|(j, _)| *j == i) {
                    Some((_, r)) => r.push((from, to)),
                    None => out.push((i, vec![(from, to)])),
                }
            }
        }
        out
    }
}

impl crate::ui::Interaction {
    /// The frame's text-fragment marks, as [`Self::mark_selection`] marks
    /// the selection: each matched element's ranges on its node, and the
    /// nodes whose ranges changed.
    pub(crate) fn mark_target_text(&mut self, tree: &mut crate::ui::Tree) -> Vec<usize> {
        let now = tree.passage_ranges(&self.target_text);
        let mut changed = Vec::new();
        for key in std::mem::take(&mut self.target_marked) {
            if let Some(i) = tree.index.find(key)
                && !now.iter().any(|(j, _)| *j == i)
                && !std::mem::take(&mut tree.nodes[i].marks.target_text).is_empty()
            {
                changed.push(i);
            }
        }
        for (i, r) in now {
            if tree.nodes[i].marks.target_text != r {
                tree.nodes[i].marks.target_text = r;
                changed.push(i);
            }
            self.target_marked.push(tree.nodes[i].key);
        }
        changed
    }
}

impl Ui {
    /// A link's text fragment, after the layout it was followed in: its
    /// passages found and marked, the element holding the first made
    /// `:target`, and that passage's line scrolled to the middle of each
    /// scroll container around it. Whether an offset moved (the layout
    /// runs again).
    pub(crate) fn follow_text_fragment(&mut self) -> bool {
        let Some(directives) = self.docs.state_mut().text_directives.take() else {
            return false;
        };
        let found = self.tree.find_text(&directives);
        self.interaction.target_text = found.clone();
        let Some(&((key, a), (end_key, _))) = found.first() else {
            return self.scroll_to_target_id();
        };
        let (Some(i), Some(j)) = (self.tree.index.find(key), self.tree.index.find(end_key)) else {
            return false;
        };
        // `:target`: the nearest document element holding both ends.
        let up = |mut k: usize| {
            let mut out = vec![k];
            while self.tree.nodes[k].parent != NONE {
                k = self.tree.nodes[k].parent;
                out.push(k);
            }
            out
        };
        let (from_start, from_end) = (up(i), up(j));
        let elements = &self.docs.state().memory.retained.by_key;
        if let Some(&holder) = from_start
            .iter()
            .find(|x| from_end.contains(x) && elements.contains_key(&self.tree.nodes[**x].key))
        {
            self.docs.state_mut().target = Some(crate::doc::build::state::FragmentTarget::Element(
                self.tree.nodes[holder].key,
            ));
        }
        let Some(line) = self.passage_line(i, a) else {
            return false;
        };
        self.interaction
            .scroll_to_fragment(&self.tree, i, line, true)
    }

    /// HTML's "scroll to the fragment" for a link naming an element by id
    /// (and a text fragment that matched nothing): the first document
    /// element with the id, in tree order, scrolled so its scroll-margin
    /// box starts at each scroller's view (`block: start`, `inline:
    /// nearest`). Whether an offset moved.
    fn scroll_to_target_id(&mut self) -> bool {
        let Some(crate::doc::build::state::FragmentTarget::Id(id)) = &self.docs.state().target
        else {
            return false;
        };
        let elements = &self.docs.state().memory.retained.by_key;
        let Some(&i) = self.tree.orders.pre.iter().find(|&&i| {
            self.tree.nodes[i].id == *id && elements.contains_key(&self.tree.nodes[i].key)
        }) else {
            return false;
        };
        let n = &self.tree.nodes[i];
        let (r, m) = (n.placed.rect, n.el.scroll_css.margin);
        let r = Rect::new(
            r.x - m[3],
            r.y - m[0],
            r.width + m[1] + m[3],
            r.height + m[0] + m[2],
        );
        self.interaction.scroll_to_fragment(&self.tree, i, r, false)
    }

    /// The line of text element `i` holding its character `a`, on screen.
    fn passage_line(&self, i: usize, a: u32) -> Option<Rect> {
        let n = &self.tree.nodes[i];
        let rect = n.placed.rect;
        if let Some(t) = self.text_lines(i) {
            let l = t
                .lines
                .iter()
                .rev()
                .find(|l| l.start <= a)
                .or(t.lines.first())?;
            return Some(match t.column {
                Some(w) => Rect::new(l.top, rect.y, w, rect.height),
                None => {
                    let lh = n
                        .text()
                        .map_or(rect.height, |tc| tc.line_height() * n.placed.scale);
                    Rect::new(rect.x, l.top, rect.width, lh)
                }
            });
        }
        let pieces = self.rich_pieces(i);
        let p = pieces
            .iter()
            .rev()
            .find(|p| p.at <= a as usize)
            .or(pieces.first())?;
        Some(Rect::new(rect.x, p.rect.y, rect.width, p.rect.height))
    }
}

impl crate::ui::Interaction {
    /// CSSOM View's scroll into view for a fragment: each scroll container
    /// around node `i`, innermost first, moves so `r` stands in the middle
    /// of its view (`center`, a text fragment's line) or starts at it (an
    /// element) along the block axis, and is shown by the least move across
    /// (`inline: nearest`); the view is inside the container's padding and
    /// scroll-padding, the offset clamped to its range. Whether an offset
    /// moved.
    fn scroll_to_fragment(
        &mut self,
        tree: &crate::ui::Tree,
        i: usize,
        mut r: Rect,
        center: bool,
    ) -> bool {
        let mut moved = false;
        let mut c = tree.nodes[i].parent;
        while c != NONE {
            let cn = &tree.nodes[c];
            if cn.el.scroll || cn.el.scroll_x {
                let pad = cn.layout.used.padding;
                let p = cn.placed.rect;
                let (w, h) = (
                    (p.width - pad.horizontal()).max(0.0),
                    (p.height - pad.vertical()).max(0.0),
                );
                let sp = cn.el.scroll_css.padding_px(w, h);
                let view = Rect::new(
                    p.x + pad.left + sp[3],
                    p.y + pad.top + sp[0],
                    (w - sp[1] - sp[3]).max(0.0),
                    (h - sp[0] - sp[2]).max(0.0),
                );
                let near = |lo: f32, hi: f32, vlo: f32, vhi: f32| {
                    if hi - lo > vhi - vlo || lo < vlo {
                        lo - vlo
                    } else if hi > vhi {
                        hi - vhi
                    } else {
                        0.0
                    }
                };
                let d = Vec2::new(
                    near(r.x, r.right(), view.x, view.right()),
                    if center {
                        (r.y + r.height / 2.0) - (view.y + view.height / 2.0)
                    } else {
                        r.y - view.y
                    },
                );
                let at = self.offset(cn.key);
                let range = Vec2::new(cn.max_scroll_x(), cn.max_scroll()).max(Vec2::ZERO);
                let to = (at + d).clamp(Vec2::ZERO, range);
                let s = &mut self.state;
                if cn.el.scroll_x && to.x != at.x {
                    s.scroll_x.insert(cn.key, to.x);
                    moved = true;
                    r.x -= to.x - at.x;
                }
                if cn.el.scroll && to.y != at.y {
                    s.scroll.insert(cn.key, to.y);
                    moved = true;
                    r.y -= to.y - at.y;
                }
            }
            c = cn.parent;
        }
        moved
    }
}

impl Ui {
    /// The screen rectangles of node `i`'s characters in `marks` (each
    /// `[start, end)` as [`crate::layout::node::NodeMarks::selected`]
    /// counts them), adjacent glyphs merged, and whether its lines run
    /// down (each rectangle a stretch of a column) rather than across.
    fn mark_bands(&self, i: usize, marks: &[(u32, u32)]) -> (Vec<Rect>, bool) {
        let n = &self.tree.nodes[i];
        let hit = |k: usize| {
            marks
                .iter()
                .any(|&(a, b)| (a as usize..b as usize).contains(&k))
        };
        let mut out: Vec<Rect> = Vec::new();
        let mut push = |r: Rect, down: bool| match out.last_mut() {
            Some(l) if !down && (l.y - r.y).abs() < 0.01 && (l.right() - r.x).abs() < 0.01 => {
                l.width = r.right() - l.x;
            }
            Some(l) if down && (l.x - r.x).abs() < 0.01 && (l.bottom() - r.y).abs() < 0.01 => {
                l.height = r.bottom() - l.y;
            }
            _ => out.push(r),
        };
        if let Some(t) = self.text_lines(i) {
            let lh = n.text().map_or(0.0, |tc| tc.line_height() * n.placed.scale);
            for l in &t.lines {
                for b in l.boxes.iter().filter(|b| hit(l.start as usize + b.index)) {
                    let (lo, hi) = (b.x0.min(b.x1), b.x0.max(b.x1));
                    match t.column {
                        Some(w) => push(Rect::new(l.top, l.x + lo, w, hi - lo), true),
                        None => push(Rect::new(l.x + lo, l.top, hi - lo, lh), false),
                    }
                }
            }
            return (out, t.column.is_some());
        }
        let Content::Rich(rc) = &*n.content else {
            return (out, false);
        };
        for p in self.rich_pieces(i) {
            let s = &rc.spans[p.span];
            let r = p.rect;
            if s.atom.is_some() {
                if hit(p.at) {
                    push(r, false);
                }
                continue;
            }
            let size = s.step.size * n.placed.scale;
            match &p.glyphs {
                Some(glyphs) => {
                    let mut x = r.x;
                    for g in glyphs {
                        let w = g.advance * size;
                        if hit(p.at + g.index) {
                            push(Rect::new(x, r.y, w, r.height), false);
                        }
                        x += w;
                    }
                }
                None => {
                    // Drawn as written: each character from where the
                    // text before it ends, kerned as the renderer kerns.
                    let m = &self
                        .env
                        .fonts
                        .for_styled(s.font, s.query(), &p.text)
                        .0
                        .metrics;
                    let tracking = crate::rich::span_tracking(s, &p.text);
                    let mut from = 0.0;
                    for (k, (b, c)) in p.text.char_indices().enumerate() {
                        let to = m.measure_tracked(&p.text[..b + c.len_utf8()], size, tracking);
                        if hit(p.at + k) {
                            push(Rect::new(r.x + from, r.y, to - from, r.height), false);
                        }
                        from = to;
                    }
                }
            }
        }
        (out, false)
    }

    /// `::target-text` on node `i`, its overlay at `under`
    /// ([`Self::overlay_commands`]): the `Mark` system colour under each
    /// marked stretch by default, as a browser's.
    pub(super) fn target_text_commands(&self, i: usize, under: usize, cmds: &mut Vec<DrawCmd>) {
        let owner = self.tree.nodes[self.text_owner(i)].key;
        let style = self.docs.state.target_text.get(&owner);
        let fill = style
            .and_then(|s| s.background)
            .or(Some(self.paint_theme().palette.primary_soft));
        let marks = &self.tree.nodes[i].marks.target_text;
        self.overlay_commands(i, under, marks, style.map(|s| &**s), fill, cmds);
    }

    /// A highlight overlay (CSS Pseudo-Elements 4 §3.6) of node `i`'s
    /// characters in `marks`, over the text commands from `under` on (the
    /// text and the overlays below this one): `fill` under each marked
    /// stretch, above the fills already there, and with `style`'s color,
    /// shadow or decoration the marked glyphs drawn again in it, the
    /// commands below cut around them. Where the next overlay's fills go.
    pub(super) fn overlay_commands(
        &self,
        i: usize,
        under: usize,
        marks: &[(u32, u32)],
        style: Option<&crate::doc::generated::Selection>,
        fill: Option<crate::Srgba>,
        cmds: &mut Vec<DrawCmd>,
    ) -> usize {
        let (bands, down) = self.mark_bands(i, marks);
        if bands.is_empty() {
            return under;
        }
        let n = &self.tree.nodes[i];
        let (alpha, sk, dpr) = (n.placed.alpha, n.placed.scale, self.env.viewport.dpr());
        let clip = self.paint_clip(n);
        let fills: Vec<DrawCmd> = fill
            .map(|fill| {
                bands
                    .iter()
                    .map(|r| DrawCmd::Quad {
                        rect: r.snapped(dpr),
                        radius: 0.0,
                        color: fill.fade(alpha).linear(),
                        clip,
                    })
                    .collect()
            })
            .unwrap_or_default();
        let at = under + fills.len();
        cmds.splice(under..under, fills);
        let Some(sel) =
            style.filter(|s| s.color.is_some() || s.decoration.is_some() || s.shadows.is_some())
        else {
            return at;
        };
        // Along the line (x across, y down a column) and across it.
        let along = |r: &Rect| {
            if down {
                (r.y, r.bottom())
            } else {
                (r.x, r.right())
            }
        };
        let meets = |r: &Rect, b: &Rect| {
            if down {
                r.x < b.right() && r.right() > b.x
            } else {
                r.y < b.bottom() && r.bottom() > b.y
            }
        };
        let cut = |cl: Rect, (lo, hi): (f32, f32)| {
            let (a, b) = along(&cl);
            let (lo, hi) = (lo.max(a), hi.min(b));
            (hi > lo).then(|| {
                if down {
                    Rect::new(cl.x, lo, cl.width, hi - lo)
                } else {
                    Rect::new(lo, cl.y, hi - lo, cl.height)
                }
            })
        };
        let text = |c: &DrawCmd| match c {
            DrawCmd::StyledText { text, .. } => match text.as_ref() {
                DrawCmd::Text {
                    bounds,
                    size,
                    baseline,
                    ..
                } => Some((*bounds, *size, *baseline)),
                _ => None,
            },
            DrawCmd::Text {
                bounds,
                size,
                baseline,
                ..
            } => Some((*bounds, *size, *baseline)),
            _ => None,
        };
        let drawn: Vec<DrawCmd> = cmds.drain(at..).collect();
        // The text's own commands, cut where marked stretches cross them.
        for c in &drawn {
            let Some((bounds, ..)) = text(c) else {
                cmds.push(c.clone());
                continue;
            };
            let mut on: Vec<(f32, f32)> = bands
                .iter()
                .filter(|b| meets(b, &bounds))
                .map(along)
                .collect();
            on.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut from = f32::NEG_INFINITY;
            for gap in on.iter().copied().chain([(f32::INFINITY, f32::INFINITY)]) {
                let mut piece = c.clone();
                if let Some(cl) = crate::rich::text_clip(&mut piece)
                    && let Some(r) = cut(*cl, (from, gap.0))
                {
                    *cl = r;
                    cmds.push(piece);
                }
                from = gap.1;
            }
        }
        // The marked stretches again, in its ink and shadows.
        let shadows = |a: f32| -> Option<Vec<og_draw::UiTextShadow>> {
            sel.shadows.as_ref().map(|list| {
                list.iter()
                    .map(|(o, blur, c)| og_draw::UiTextShadow {
                        offset: Vec2::new(o[0] * sk, o[1] * sk),
                        blur: blur * sk,
                        color: c.fade(a).linear(),
                    })
                    .collect()
            })
        };
        for band in &bands {
            let mut line: Option<(f32, f32)> = None;
            for c in &drawn {
                let Some((_, size, baseline)) = text(c).filter(|t| meets(band, &t.0)) else {
                    continue;
                };
                line.get_or_insert((size, baseline));
                let mut c = c.clone();
                let Some(cl) = crate::rich::text_clip(&mut c) else {
                    continue;
                };
                let Some(r) = cut(*cl, along(band)) else {
                    continue;
                };
                *cl = r;
                let (mut inner, effects) = match c {
                    DrawCmd::StyledText { text, effects } => (*text, Some(effects)),
                    c => (c, None),
                };
                let DrawCmd::Text { color, .. } = &mut inner else {
                    continue;
                };
                let a = color[3];
                if let Some(ink) = sel.color {
                    *color = ink.fade(a).linear();
                }
                let effects = match (effects, shadows(a)) {
                    (Some(mut e), Some(sh)) => {
                        e.shadows = sh;
                        Some(e)
                    }
                    (Some(e), None) => Some(e),
                    (None, Some(sh)) => Some(Box::new(og_draw::UiTextEffects {
                        shadows: sh,
                        transform: [1.0, 0.0, 0.0, 1.0],
                        ..Default::default()
                    })),
                    (None, None) => None,
                };
                cmds.push(match effects {
                    Some(effects) => DrawCmd::StyledText {
                        text: Box::new(inner),
                        effects,
                    },
                    None => inner,
                });
            }
            // Its lines along the stretch, where the text's own go.
            let (Some(d), Some((size, baseline))) = (&sel.decoration, line) else {
                continue;
            };
            let m = match &*n.content {
                Content::Text(tc) => {
                    let tc = n.used_text(tc);
                    &self
                        .env
                        .fonts
                        .for_styled(tc.font, tc.query(), &tc.text)
                        .0
                        .metrics
                }
                Content::Rich(rc) => match rc.spans.iter().find(|s| s.atom.is_none()) {
                    Some(s) => {
                        &self
                            .env
                            .fonts
                            .for_styled(s.font, s.query(), &s.text)
                            .0
                            .metrics
                    }
                    None => continue,
                },
                _ => continue,
            };
            let ink = d
                .color
                .or(sel.color)
                .unwrap_or(n.style.sampled.ink)
                .fade(alpha)
                .linear();
            if down {
                let (th, off) = crate::css::text_css::Decoration {
                    under: false,
                    ..d.clone()
                }
                .placed(m, size, sk, dpr);
                let mid = band.x + band.width / 2.0;
                let mut rule = |x: f32| {
                    crate::rich::vertical_decoration_line(
                        x - th / 2.0,
                        band.y,
                        band.height,
                        th,
                        d.style,
                        ink,
                        clip,
                        dpr,
                        cmds,
                    )
                };
                if d.underline {
                    rule(mid - size * 0.5 - off);
                }
                if d.overline {
                    rule(mid + size * 0.6);
                }
                if d.through {
                    rule(mid);
                }
            } else {
                let (th, off) = d.placed(m, size, sk, dpr);
                let mut rule = |y: f32| {
                    crate::rich::decoration_line(
                        band.x, band.width, y, th, d.style, ink, clip, dpr, cmds,
                    )
                };
                if d.underline {
                    rule(baseline + off);
                }
                if d.overline {
                    rule(baseline - size * 0.95);
                }
                if d.through {
                    rule(baseline - size * 0.3);
                }
            }
        }
        at
    }
}

#[cfg(test)]
mod tests {
    use super::{TextDirective, parse_fragment};

    fn d(
        prefix: Option<&str>,
        start: &str,
        end: Option<&str>,
        suffix: Option<&str>,
    ) -> TextDirective {
        TextDirective {
            prefix: prefix.map(Into::into),
            start: start.into(),
            end: end.map(Into::into),
            suffix: suffix.map(Into::into),
        }
    }

    #[test]
    fn a_fragment_splits_into_its_id_and_text_directives() {
        assert_eq!(parse_fragment("Rules"), ("Rules".into(), vec![]));
        assert_eq!(
            parse_fragment("Rules:~:text=the%20gate,-is%2Dshut&unknown&text=a-,b,c"),
            (
                "Rules".into(),
                vec![
                    d(None, "the gate", None, Some("is-shut")),
                    d(Some("a"), "b", Some("c"), None),
                ]
            )
        );
        // An empty term, a stray dash, three match terms: no directive.
        for bad in [
            "text=",
            "text=a,,b",
            "text=a-b",
            "text=a,b,c",
            "text=-,a",
            "text=%FF",
        ] {
            assert_eq!(parse_fragment(&format!(":~:{bad}")).1, vec![], "{bad}");
        }
    }
}
