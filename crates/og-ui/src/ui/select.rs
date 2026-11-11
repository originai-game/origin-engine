//! Selectable read-only text (CSS `user-select`, docs/ui.md §2): a press
//! and drag over text elements that allow it selects their characters, a
//! double press a word and a triple press a line (a long press on a touch
//! screen a word), across elements in the tree's order as a page selects;
//! Ctrl or Cmd+C copies it for the host's clipboard.
//!
//! A selection's ends are (text element, character) pairs; a character is
//! counted over the element's laid-out lines joined by one break each. The
//! press reads last frame's hits and lines (what the player sees), as every
//! other press does; the frame marks only the elements whose selected range
//! changed, so the paint follows the selection, not the tree.

use super::*;
use crate::element::UserSelect;

/// One end of a selection: a text element's key and a character in it.
pub(crate) type SelectEnd = (u64, u32);

/// The player's selection of read-only text ([`State::selection`]).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Selection {
    /// Where it began and where it ends now (equal: a caret, nothing selected).
    pub anchor: SelectEnd,
    pub focus: SelectEnd,
    /// The word or line a double or triple press selected first: a drag
    /// extends by words or lines from it.
    pub origin: (SelectEnd, SelectEnd),
    /// 1 characters, 2 words, 3 lines.
    pub unit: u8,
}

/// The presses on selectable text, counted into double and triple presses
/// and timed into a touch screen's long press ([`State::select_press`]).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct SelectPress {
    /// The tick of the last press, and where it landed.
    pub tick: u64,
    pub at: Vec2,
    /// Presses in a row on selectable text (0: none).
    pub count: u8,
    /// The press moved past the slop: not a long press, and the next
    /// press starts again.
    pub moved: bool,
    /// A touch on selectable text, held: a long press selects a word.
    pub long: bool,
}

/// A selectable text's lines where they are drawn, in its layout box.
pub(crate) struct TextLines {
    /// Each line: its first character's index, its characters, its pen's
    /// start, its box's top, and its glyphs' boxes.
    pub lines: Vec<TextLine>,
    /// Vertical lines' column width: each line's `top` is its column's
    /// left edge and its boxes run down from `x` (0), along y.
    pub column: Option<f32>,
}

pub(crate) struct TextLine {
    pub start: u32,
    pub chars: Vec<char>,
    pub x: f32,
    pub top: f32,
    pub boxes: Vec<crate::edit::GlyphBox>,
}

impl TextLines {
    /// Its characters, lines joined by `\n`: what words and lines are found in.
    fn flat(&self) -> Vec<char> {
        let mut out = Vec::new();
        for (k, l) in self.lines.iter().enumerate() {
            if k > 0 {
                out.push('\n');
            }
            out.extend_from_slice(&l.chars);
        }
        out
    }

    fn len(&self) -> u32 {
        self.lines
            .last()
            .map_or(0, |l| l.start + l.chars.len() as u32)
    }

    /// The character boundary nearest `(x, y)` in its layout box: on the
    /// line at `y` (the first above its lines, the last below them).
    fn at(&self, x: f32, y: f32) -> u32 {
        let Some(first) = self.lines.first() else {
            return 0;
        };
        if let Some(w) = self.column {
            // The column nearest across, then down it.
            let d = |l: &TextLine| (x - (l.top + w / 2.0)).abs();
            let l = self
                .lines
                .iter()
                .min_by(|a, b| d(a).total_cmp(&d(b)))
                .unwrap_or(first);
            return l.start + crate::edit::nearest(&l.boxes, &l.chars, y - l.x) as u32;
        }
        let li = self.lines.iter().rposition(|l| y >= l.top).unwrap_or(0);
        let l = if y < first.top {
            first
        } else {
            &self.lines[li]
        };
        l.start + crate::edit::nearest(&l.boxes, &l.chars, x - l.x) as u32
    }
}

impl Ui {
    /// Node `i`'s selectable text: its source and its laid-out lines (a
    /// plain text's, or a vertical rich text's, each line its pieces'
    /// text); `None` for any other node.
    pub(crate) fn select_source(&self, i: usize) -> Option<(String, Vec<crate::text::Line>)> {
        let n = &self.nodes[i];
        match &n.content {
            Content::Text(tc) => Some((tc.text.clone(), n.lines.clone())),
            Content::Rich(rc) if n.rich_vertical().is_some() => {
                let source = rc.spans.iter().filter(|s| s.atom.is_none());
                let lines = n
                    .rich_lines
                    .iter()
                    .map(|l| crate::text::Line {
                        text: l.pieces.iter().map(|p| p.text.as_str()).collect(),
                        width: l.width,
                    })
                    .collect();
                Some((source.map(|s| s.text.as_str()).collect(), lines))
            }
            _ => None,
        }
    }

    /// Node `i`'s text lines as [`Ui::node_commands`] draws them, `None`
    /// for a node that is not a plain text or a vertical rich text.
    pub(crate) fn text_lines(&self, i: usize) -> Option<TextLines> {
        let n = &self.nodes[i];
        if matches!(n.content, Content::Rich(_)) {
            let (_, lines) = self.select_source(i)?;
            let v = n.rich_vertical()?;
            let up =
                self.text_flow(i).0 != (v.mode == crate::layout::flow::WritingMode::SidewaysLr);
            let mut start = 0u32;
            let drawn = self.vertical_rich_lines(i);
            let column = drawn.first().map_or(0.0, |l| l.width);
            let lines = drawn
                .into_iter()
                .zip(&lines)
                .map(|(l, line)| {
                    let l = vertical_line(l, line, start, up);
                    start += l.chars.len() as u32 + 1;
                    l
                })
                .collect();
            return Some(TextLines {
                lines,
                column: Some(column),
            });
        }
        let Content::Text(tc) = &n.content else {
            return None;
        };
        if let Some(v) = tc.vertical {
            // Down vertical lines: each cluster's stretch along y, a
            // bottom-up one with its caret at its bottom.
            let lines = self.vertical_lines(i, tc, v, n.rect, n.clip, None);
            let up =
                self.text_flow(i).0 != (v.mode == crate::layout::flow::WritingMode::SidewaysLr);
            let mut start = 0u32;
            let lines = lines
                .into_iter()
                .zip(&n.lines)
                .map(|(l, line)| {
                    let l = vertical_line(l, line, start, up);
                    start += l.chars.len() as u32 + 1;
                    l
                })
                .collect();
            return Some(TextLines {
                lines,
                column: Some(tc.line_height() * n.scale),
            });
        }
        let font = self
            .env
            .fonts
            .for_styled(tc.font, tc.step.weight, false, &tc.text)
            .0;
        let m = &font.metrics;
        let sk = n.scale;
        let size = tc.step.size * sk;
        let lh = tc.line_height() * sk;
        let pad = n.el.padding;
        let rect = n.rect;
        let inner = Rect::new(
            rect.x + pad.left * sk,
            rect.y + pad.top * sk,
            rect.width - pad.horizontal() * sk,
            rect.height - pad.vertical() * sk,
        );
        let block = lh * n.lines.len() as f32;
        let top = inner.y + ((inner.height - block) / 2.0).max(0.0);
        let rtl = self.env.direction == og_i18n::Direction::Rtl;
        let tracking = tc.tracking();
        let mut start = 0u32;
        let lines = n
            .lines
            .iter()
            .enumerate()
            .map(|(k, line)| {
                let free = inner.width - line.width * sk;
                let x =
                    inner.x
                        + match (tc.align, rtl) {
                            (TextAlign::Start | TextAlign::Justify, false)
                            | (TextAlign::End, true) => 0.0,
                            (TextAlign::Center, _) => free / 2.0,
                            (TextAlign::End, false)
                            | (TextAlign::Start | TextAlign::Justify, true) => free,
                        };
                let chars: Vec<char> = line.text.chars().collect();
                let l = TextLine {
                    start,
                    x,
                    top: top + k as f32 * lh,
                    boxes: crate::edit::boxes(m, &line.text, self.env.direction, size, tracking),
                    chars,
                };
                start += l.chars.len() as u32 + 1;
                l
            })
            .collect();
        Some(TextLines {
            lines,
            column: None,
        })
    }

    /// The topmost selectable text under `p` drawn over `hot` (what a
    /// press there would otherwise land on): a scroll container or panel
    /// behind it, or nothing.
    pub(crate) fn select_hit_at(&self, p: Vec2, hot: u64) -> Option<u64> {
        let hits = &self.state.hits;
        let above = hits.iter().rposition(|h| h.key == hot).map_or(0, |k| k + 1);
        hits[above..]
            .iter()
            .rev()
            .find(|h| h.has(HIT_SELECT) && !h.has(HIT_OFFSCREEN) && h.contains(p.x, p.y))
            .map(|h| h.key)
    }

    /// The character under screen point `p` in text `key` (last frame's
    /// lines, through its transform), and its whole length.
    fn select_point(&self, key: u64, p: Vec2) -> Option<(u32, TextLines)> {
        let i = self.tree.find(key)?;
        let t = self.text_lines(i)?;
        let (x, y) = self.nodes[i]
            .world
            .invert()
            .unwrap_or(Affine::IDENTITY)
            .apply(p.x, p.y);
        Some((t.at(x, y), t))
    }

    /// A drag's text: the one under the pointer, else the selectable text
    /// level with it nearest across (a drag past a line's end), else the
    /// nearest above or below (a drag between paragraphs or past the last
    /// selects up to its edge), as a page picks.
    fn select_drag_target(&self, p: Vec2) -> Option<u64> {
        let off = |a: f32, lo: f32, hi: f32| (lo - a).max(a - hi).max(0.0);
        self.select_hit_at(p, 0).or_else(|| {
            self.state
                .hits
                .iter()
                .filter(|h| h.has(HIT_SELECT) && !h.has(HIT_OFFSCREEN))
                .min_by(|a, b| {
                    let d = |h: &Hit| {
                        let r = &h.full;
                        (off(p.y, r.y, r.bottom()), off(p.x, r.x, r.right()))
                    };
                    let (da, db) = (d(a), d(b));
                    da.0.total_cmp(&db.0).then(da.1.total_cmp(&db.1))
                })
                .map(|h| h.key)
        })
    }

    /// Where `end` falls in the trees' order (its element's place, then
    /// its character).
    fn select_order(&self, end: SelectEnd) -> (u32, u32) {
        let pos = self
            .tree
            .find(end.0)
            .and_then(|i| self.orders.pre_pos.get(i).copied())
            .unwrap_or(u32::MAX);
        (pos, end.1)
    }

    /// The unit a press of `count` selects around character `k` of `key`.
    fn select_unit(&self, key: u64, t: &TextLines, count: u8, k: u32) -> (SelectEnd, SelectEnd) {
        let all = self
            .tree
            .find(key)
            .is_some_and(|i| self.orders.select_of.get(i) == Some(&UserSelect::All));
        let (a, b) = if all {
            (0, t.len() as usize)
        } else {
            let flat = t.flat();
            match count {
                2 => crate::edit::word_at(&flat, k as usize),
                3 => crate::edit::line_at(&flat, k as usize),
                _ => (k as usize, k as usize),
            }
        };
        ((key, a as u32), (key, b as u32))
    }

    /// This tick's pointer and keys on selectable text, after the press
    /// decided what is hot: a press on text over what it would land on
    /// selects (and holds the pointer, so nothing behind it scrolls), a
    /// drag extends, a press anywhere else clears; Ctrl or Cmd+C copies.
    pub(crate) fn select_input(&mut self, input: &crate::input::InputFrame, hot: u64) {
        let p = input.pointer;
        let tick = self.state.tick;
        let coarse = self.env.viewport.coarse_pointer;
        let hold = crate::edit::click_ticks(self.env.ticks_per_second);
        let mut press = self.state.select_press;
        let held_key = self
            .hit(self.state.active)
            .filter(|h| h.has(HIT_SELECT))
            .map(|h| h.key);
        let mut start = None;
        if input.pressed && input.pointer_inside {
            let at = self.select_hit_at(p, hot);
            let near = (p - press.at).length() <= SELECT_SLOP;
            let soon = tick.saturating_sub(press.tick) <= hold;
            press.count = if at.is_some() && press.count > 0 && !press.moved && near && soon {
                (press.count + 1).min(3)
            } else {
                1
            };
            press.tick = tick;
            press.at = p;
            press.moved = false;
            // A touch selects with a long press, a pointer with the press.
            press.long = at.is_some() && coarse;
            match at.filter(|_| !coarse) {
                Some(key) => start = Some((key, press.count)),
                None => {
                    if self.state.selection.is_some() {
                        self.state.selection = None;
                    }
                }
            }
        } else if input.down {
            if (p - press.at).length() > SELECT_SLOP {
                press.moved = true;
            }
            // A long press, still: the word under it.
            if press.long && !press.moved && tick.saturating_sub(press.tick) >= hold {
                press.long = false;
                if let Some(key) = self.select_hit_at(press.at, 0) {
                    start = Some((key, 2));
                }
            } else if press.moved {
                press.long = false;
            }
        } else {
            press.long = false;
        }
        self.state.select_press = press;
        if let Some((key, count)) = start {
            if let Some((k, t)) = self.select_point(key, press.at) {
                let (a, b) = self.select_unit(key, &t, count, k);
                let extend = input.shift && count == 1 && !coarse;
                self.state.selection = Some(match self.state.selection {
                    Some(s) if extend => Selection {
                        focus: (key, k),
                        unit: 1,
                        ..s
                    },
                    _ => Selection {
                        anchor: a,
                        focus: b,
                        origin: (a, b),
                        unit: count,
                    },
                });
                // The press is the text's: nothing behind it scrolls or drags.
                self.state.active = key;
            }
        } else if input.down
            && !input.pressed
            && held_key.is_some()
            && let Some(mut s) = self.state.selection
            && let Some(key) = self.select_drag_target(p)
            && let Some((k, t)) = self.select_point(key, p)
        {
            if s.unit >= 2 {
                let (a, b) = self.select_unit(key, &t, s.unit, k);
                let (o0, o1) = s.origin;
                if self.select_order(a) < self.select_order(o0) {
                    s.anchor = o1;
                    s.focus = a;
                } else {
                    s.anchor = o0;
                    s.focus = if self.select_order(b) > self.select_order(o1) {
                        b
                    } else {
                        o1
                    };
                }
            } else {
                s.focus = (key, k);
            }
            self.state.selection = Some(s);
        }
        let mac = self.env.viewport.mac_keys;
        let shortcut = if mac {
            input.meta
        } else {
            input.ctrl || input.meta
        };
        // A focused text control copies its own selection.
        let editing = self.hit(self.state.focus).is_some_and(|h| h.has(HIT_TEXT));
        if shortcut
            && !editing
            && input.keys.contains(&crate::input::UiKey::KeyC)
            && let Some(text) = self.selected_text().filter(|t| !t.is_empty())
        {
            self.out.clipboard = Some(text);
        }
    }

    /// The selected elements and their ranges, in the trees' order: the
    /// selectable texts from the selection's first end to its last.
    pub(crate) fn selected_ranges(&self) -> Vec<(usize, (u32, u32))> {
        let Some(s) = self.state.selection else {
            return Vec::new();
        };
        let (Some(ia), Some(ib)) = (self.tree.find(s.anchor.0), self.tree.find(s.focus.0)) else {
            return Vec::new();
        };
        let o = &self.orders;
        let (pa, pb) = (o.pre_pos[ia], o.pre_pos[ib]);
        if pa == u32::MAX || pb == u32::MAX {
            return Vec::new();
        }
        let ((first, a), (last, b)) = if (pa, s.anchor.1) <= (pb, s.focus.1) {
            ((pa, s.anchor.1), (pb, s.focus.1))
        } else {
            ((pb, s.focus.1), (pa, s.anchor.1))
        };
        let mut out = Vec::new();
        for pos in first..=last {
            let i = o.pre[pos as usize];
            if !matches!(o.select_of[i], UserSelect::Text | UserSelect::All) {
                continue;
            }
            let Some((_, lines)) = self.select_source(i) else {
                continue;
            };
            let n = lines
                .iter()
                .map(|l| l.text.chars().count() as u32 + 1)
                .sum::<u32>()
                - u32::from(!lines.is_empty());
            let from = if pos == first { a.min(n) } else { 0 };
            let to = if pos == last { b.min(n) } else { n };
            if from < to {
                out.push((i, (from, to)));
            }
        }
        out
    }

    /// The selected text as the clipboard takes it: each element's
    /// characters, a wrapped line's break as its source had it (a space,
    /// nothing between CJK, a line break), elements on lines of their own.
    pub(crate) fn selected_text(&self) -> Option<String> {
        let ranges = self.selected_ranges();
        if ranges.is_empty() {
            return None;
        }
        let mut out = String::new();
        for (k, &(i, (a, b))) in ranges.iter().enumerate() {
            if k > 0 {
                out.push('\n');
            }
            let Some((source, lines)) = self.select_source(i) else {
                continue;
            };
            let seps = line_breaks(&source, &lines);
            let mut at = 0u32;
            for (li, line) in lines.iter().enumerate() {
                for c in line.text.chars() {
                    if at >= a && at < b {
                        out.push(c);
                    }
                    at += 1;
                }
                if li + 1 < lines.len() {
                    if at >= a && at < b {
                        out.push_str(&seps[li]);
                    }
                    at += 1;
                }
            }
        }
        Some(out)
    }

    /// Before the paint: each selected element's range on its node, and
    /// the nodes whose range changed (to be painted again); a selection
    /// whose element left the trees is dropped.
    pub(crate) fn mark_selection(&mut self) -> Vec<usize> {
        if let Some(s) = self.state.selection
            && (self.tree.find(s.anchor.0).is_none() || self.tree.find(s.focus.0).is_none())
        {
            self.state.selection = None;
        }
        let now = self.selected_ranges();
        let mut changed = Vec::new();
        let last = std::mem::take(&mut self.selected);
        for &key in &last {
            if let Some(i) = self.tree.find(key)
                && !now.iter().any(|&(j, _)| j == i)
                && self.nodes[i].selected.take().is_some()
            {
                changed.push(i);
            }
        }
        for &(i, r) in &now {
            if self.nodes[i].selected != Some(r) {
                self.nodes[i].selected = Some(r);
                changed.push(i);
            }
            self.selected.push(self.nodes[i].key);
        }
        changed
    }

    /// The selection's marks on one line of node `i`, under its text: the
    /// text control's `selection` part (`TextEdit.selection`).
    pub(crate) fn select_marks(
        &self,
        i: usize,
        line: &str,
        first: u32,
        (x, top, lh): (f32, f32, f32),
        clip: Rect,
        cmds: &mut Vec<DrawCmd>,
    ) {
        let n = &self.nodes[i];
        let (Some((a, b)), Content::Text(tc)) = (n.selected, &n.content) else {
            return;
        };
        let count = line.chars().count() as u32;
        if b <= first || a >= first + count {
            return;
        }
        let font = self
            .env
            .fonts
            .for_styled(tc.font, tc.step.weight, false, &tc.text)
            .0;
        let edit = crate::layout::EditView {
            focused: true,
            selection: (a as usize, b as usize),
            ..Default::default()
        };
        self.edit_marks(
            i,
            &EditMarks {
                m: &font.metrics,
                text: line,
                x,
                top,
                line_h: lh,
                size: tc.step.size * n.scale,
                tracking: tc.tracking(),
                first: first as usize,
                edit: &edit,
                clip,
                alpha: n.alpha,
                under: true,
                caret_here: false,
            },
            cmds,
        );
    }

    /// The selection as the probe reads it (`dump_ui`'s `selection`):
    /// its text and its ends' paths and characters, `null` with none.
    pub(crate) fn selection_json(&self) -> serde_json::Value {
        let Some(s) = self.state.selection else {
            return serde_json::Value::Null;
        };
        let path = |k: u64| self.tree.find(k).map(|i| self.path_of(i));
        serde_json::json!({
            "text": self.selected_text().unwrap_or_default(),
            "anchor": { "path": path(s.anchor.0), "at": s.anchor.1 },
            "focus": { "path": path(s.focus.0), "at": s.focus.1 },
        })
    }
}

/// A vertical line's selection line: its clusters' stretches down it as
/// glyph boxes along y (`up`: a bottom-up line, its caret at its bottom),
/// its column's left edge as its `top`.
fn vertical_line(
    l: crate::ui::VerticalLine,
    line: &crate::text::Line,
    start: u32,
    up: bool,
) -> TextLine {
    let boxes = l
        .glyphs
        .iter()
        .flat_map(|(c, y0, y1)| {
            c.clone().map(move |k| crate::edit::GlyphBox {
                index: k,
                x0: *y0,
                x1: *y1,
                rtl: up,
            })
        })
        .collect();
    TextLine {
        start,
        chars: line.text.chars().collect(),
        x: 0.0,
        top: l.col,
        boxes,
    }
}

/// How far a press may land from the last and still count as its next
/// (and move before it is a drag), CSS px: the text controls' slop.
const SELECT_SLOP: f32 = 4.0;

/// What stood between each two laid-out lines in the source: a line
/// break, the spaces a wrap dropped (one space), or nothing (a break
/// between CJK, inside a word too long for the line).
fn line_breaks(source: &str, lines: &[crate::text::Line]) -> Vec<String> {
    let mut out = Vec::with_capacity(lines.len().saturating_sub(1));
    let mut at = 0usize;
    let mut found = Vec::with_capacity(lines.len());
    for l in lines {
        match source.get(at..).and_then(|rest| rest.find(&l.text)) {
            Some(off) if !l.text.is_empty() || at < source.len() => {
                found.push(Some((at + off, at + off + l.text.len())));
                at += off + l.text.len();
            }
            _ => found.push(None),
        }
    }
    for w in found.windows(2) {
        out.push(match (w[0], w[1]) {
            (Some((_, end)), Some((start, _))) if start >= end => {
                let gap = &source[end..start];
                if gap.contains('\n') {
                    "\n".into()
                } else if gap.chars().any(char::is_whitespace) {
                    " ".into()
                } else {
                    String::new()
                }
            }
            _ => "\n".into(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::line_breaks;
    use crate::text::Line;

    fn lines(t: &[&str]) -> Vec<Line> {
        t.iter()
            .map(|s| Line {
                text: (*s).into(),
                width: 0.0,
            })
            .collect()
    }

    #[test]
    fn a_wrap_copies_as_its_source_had_it() {
        // A wrap's dropped spaces are one space, a hard break stays a
        // break, a break between CJK is nothing.
        assert_eq!(
            line_breaks(
                "the quick  brown\nfox",
                &lines(&["the quick", "brown", "fox"])
            ),
            vec![" ".to_string(), "\n".to_string()]
        );
        assert_eq!(
            line_breaks("漢字かな", &lines(&["漢字", "かな"])),
            vec![String::new()]
        );
    }
}
