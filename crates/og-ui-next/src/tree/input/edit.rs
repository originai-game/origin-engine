//! `<Input>` text editing, caret placement, undo/redo, clipboard, and IME
//! composition.

use super::*;

/// The grapheme cluster edge before (`back`) or after byte `at` in `text`
/// (og-i18n's UAX #29 segmenter): one step of the caret, one Backspace.
fn cluster_edge(text: &str, at: usize, back: bool) -> usize {
    // The segmenter counts in characters; the caret is in bytes.
    let bytes: Vec<usize> = text
        .char_indices()
        .map(|(b, _)| b)
        .chain([text.len()])
        .collect();
    let edges = og_i18n::graphemes(text)
        .into_iter()
        .map(|(_, end)| bytes[end]);
    let edges: Vec<usize> = std::iter::once(0).chain(edges).collect();
    if back {
        edges.iter().rev().copied().find(|&b| b < at).unwrap_or(0)
    } else {
        edges
            .iter()
            .copied()
            .find(|&b| b > at)
            .unwrap_or(text.len())
    }
}

/// Word boundary before (`back`) or after byte `at` in `text`, for Ctrl/Alt+Arrow
/// and Ctrl/Alt+Backspace/Delete.
fn word_edge(text: &str, at: usize, back: bool) -> usize {
    let at = at.min(text.len());
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    if back {
        let mut iter = text[..at].char_indices().rev().peekable();
        while let Some(&(_, c)) = iter.peek() {
            if c.is_whitespace() {
                iter.next();
            } else {
                break;
            }
        }
        let Some((first_i, first_c)) = iter.next() else {
            return 0;
        };
        let target_word = is_word(first_c);
        let mut pos = first_i;
        for (i, c) in iter {
            if is_word(c) == target_word && !c.is_whitespace() {
                pos = i;
            } else {
                break;
            }
        }
        pos
    } else {
        let mut iter = text[at..].char_indices().peekable();
        while let Some(&(_, c)) = iter.peek() {
            if c.is_whitespace() {
                iter.next();
            } else {
                break;
            }
        }
        let Some((_, first_c)) = iter.next() else {
            return text.len();
        };
        let target_word = is_word(first_c);
        for (i, c) in iter {
            if is_word(c) != target_word || c.is_whitespace() {
                return at + i;
            }
        }
        text.len()
    }
}

/// An `Input`'s caret bar, from its box's top left: x, y, its width (a
/// whole device pixel or more at `scale`) and its line's height. At the
/// composition's end while an input method composes, before the
/// placeholder while the text is empty. `None` before its text is laid out.
pub(in crate::tree) fn caret_bar(
    el: &El,
    f: &Per,
    texts: &crate::text::Texts,
    arena: &Arena,
    scale: f32,
    text_scale: f32,
) -> Option<[f32; 4]> {
    let leaf = el.leaf?;
    let size = el.eff.size * text_scale;
    let line = el.eff.text.line_height(el.eff.size) * text_scale;
    let at = if let Some((_, ce)) = f.composing_display_range() {
        ce
    } else if f.value.is_empty() {
        0
    } else {
        f.display_offset(f.caret)
    };
    let [cx, top, height] = match el.text.is_empty() {
        true => None,
        false => texts.caret(leaf, at),
    }
    .unwrap_or([0.0, 0.0, line]);
    let lb = arena.box_of(leaf);
    let bar = (size / 14.0 * scale).round().max(1.0) / scale;
    let scroll = if f.multiline { 0.0 } else { f.scroll_x };
    Some([lb.x + cx - scroll, lb.y + top, bar, height])
}

impl Tree {
    /// Shows an `Input`'s text (masked when `secret`, with active IME
    /// `composing` text spliced at the selection), or its placeholder while
    /// both are empty, and paints its caret again.
    pub(in crate::tree) fn show_value(&mut self, e: u32) {
        let p = &self.input.per[e as usize];
        let shown = if p.value.is_empty() && p.composing.is_empty() {
            p.placeholder.to_string()
        } else if p.composing.is_empty() {
            if p.secret {
                "•".repeat(p.value.chars().count())
            } else {
                p.value.clone()
            }
        } else {
            let (start, end) = p.selection_range();
            let mut s = String::with_capacity(p.value.len() + p.composing.len());
            if p.secret {
                s.push_str(&"•".repeat(p.value[..start].chars().count()));
                s.push_str(&"•".repeat(p.composing.chars().count()));
                s.push_str(&"•".repeat(p.value[end..].chars().count()));
            } else {
                s.push_str(&p.value[..start]);
                s.push_str(&p.composing);
                s.push_str(&p.value[end..]);
            }
            s
        };
        self.set_text(e, &shown);
        self.paint = true;
    }

    /// The focused `Input` that takes edits: shown and enabled.
    pub(super) fn editing(&self) -> Option<u32> {
        let f = self.input.focus;
        (f != NONE && self.input.per(f).is_some_and(|p| p.input) && !self.disabled(f)).then_some(f)
    }

    /// The focused `Input` for the host shell's text-entry overlay
    /// (`packages/engine/src/web/text-input.ts`), or `None` when no `Input` is focused.
    pub fn text_target(&self) -> Option<TextTarget> {
        let e = self.editing()?;
        let p = &self.input.per[e as usize];
        let &(x, y, ..) = self.abs.get(e as usize)?;
        let b = self.arena.box_of(self.els[e as usize].node);
        let selection = if p.secret {
            String::new()
        } else {
            let (start, end) = p.selection_range();
            p.value[start..end].to_owned()
        };
        let caret = self
            .caret_in(e)
            .map_or([x, y, 0.0, b.h], |[cx, cy, w, h]| [x + cx, y + cy, w, h]);
        Some(TextTarget {
            path: self.els[e as usize].node.to_string(),
            rect: [x, y, b.w, b.h],
            caret,
            multiline: p.multiline,
            selection,
            composing: !p.composing.is_empty(),
            input_type: p.input_type.to_string(),
            input_mode: p.input_mode.to_string(),
            autocomplete: p.autocomplete.to_string(),
            secret: p.secret,
        })
    }

    /// The `Input` `e`'s caret bar ([`caret_bar`]).
    pub(in crate::tree) fn caret_in(&self, e: u32) -> Option<[f32; 4]> {
        let f = self.input.per(e).filter(|f| f.input)?;
        let el = &self.els[e as usize];
        caret_bar(el, f, &self.texts, &self.arena, self.scale, self.text_scale)
    }

    /// Text copied or cut from a focused `Input` since the last call.
    pub fn take_copy(&mut self) -> Option<String> {
        self.input.copied.take()
    }

    /// Updates active IME preedit composition on the focused `Input` without
    /// firing `change` until committed via [`Tree::type_text`]. Empty string
    /// clears composition.
    pub fn compose(&mut self, text: &str) -> bool {
        let Some(e) = self.editing() else {
            return false;
        };
        let p = &mut self.input.per[e as usize];
        if p.read_only {
            return false;
        }
        if p.composing == text {
            return true;
        }
        p.composing.clear();
        p.composing.push_str(text);
        self.show_value(e);
        true
    }

    /// Before an edit of `kind` to `e`: its state goes on the undo stack,
    /// unless the edit joins the one before it (typing after typing,
    /// deleting after deleting, within [`UNDO_PAUSE`]); redo is cleared.
    pub(super) fn remember(&mut self, e: u32, kind: Edit) {
        let time = self.time;
        let p = &mut self.input.per[e as usize];
        let (last, at) = p.last_edit;
        if kind == Edit::Other || kind != last || time - at > UNDO_PAUSE {
            if p.undo.len() == UNDO_STEPS {
                p.undo.remove(0);
            }
            p.undo.push(Snap {
                value: p.value.clone(),
                caret: p.caret,
                anchor: p.anchor,
            });
        }
        p.redo.clear();
        p.last_edit = (kind, time);
    }

    /// Undo (or `redo`) on the focused field `e`: its text and selection
    /// go back to the last step (forward to the next), which it hears as
    /// a `change`. Whether there was one.
    pub(super) fn undo(&mut self, e: u32, redo: bool) -> bool {
        let p = &mut self.input.per[e as usize];
        if p.read_only {
            return false;
        }
        let Some(s) = (if redo { p.redo.pop() } else { p.undo.pop() }) else {
            return false;
        };
        let now = Snap {
            value: std::mem::replace(&mut p.value, s.value),
            caret: p.caret,
            anchor: p.anchor,
        };
        (p.caret, p.anchor) = (s.caret, s.anchor);
        p.composing.clear();
        p.last_edit = (Edit::None, 0.0);
        if redo { &mut p.undo } else { &mut p.redo }.push(now);
        self.edited(e);
        true
    }

    /// The caret offset in field `e`'s value nearest the surface point
    /// (x, y), from its laid-out text.
    fn field_hit(&self, e: u32, x: f32, y: f32) -> Option<usize> {
        let (leaf, ox, oy) = self.field_origin(e)?;
        let p = &self.input.per[e as usize];
        if p.value.is_empty() {
            return Some(0);
        }
        let (ux, uy) = self.untransform(e, x, y)?;
        let shown = self.texts.leaf(leaf)?.hit(ux - ox, uy - oy);
        Some(p.value_offset(shown))
    }

    /// Field `e`'s text leaf and the surface point its text starts at
    /// (scrolled with a one-line field's text).
    fn field_origin(&self, e: u32) -> Option<(Id, f32, f32)> {
        let leaf = self.els[e as usize].leaf?;
        let &(x, y, ..) = self.abs.get(e as usize)?;
        let lb = self.arena.box_of(leaf);
        let p = &self.input.per[e as usize];
        let scroll = if p.multiline { 0.0 } else { p.scroll_x };
        Some((leaf, x + lb.x - scroll, y + lb.y))
    }

    /// A press at (x, y) after it focused: in the focused field's text it
    /// places the caret there (and starts a drag selection), a second
    /// press in a row selects the word there, a third the whole text.
    pub(in crate::tree) fn field_press(&mut self, x: f32, y: f32) {
        let Some(e) = self.editing() else {
            return;
        };
        if !self.inside(e, x, y) || !self.input.per[e as usize].composing.is_empty() {
            return;
        }
        let n = match self.input.field_press {
            Some((f, at, p, n)) if f == e && self.time - at <= DOUBLE_PRESS && near(p, (x, y)) => {
                n % 3 + 1
            }
            _ => 1,
        };
        self.input.field_press = Some((e, self.time, (x, y), n));
        let word = match n {
            2 => self.untransform(e, x, y).and_then(|(ux, uy)| {
                self.field_origin(e).and_then(|(leaf, ox, oy)| {
                    Some(self.texts.leaf(leaf)?.word_at(ux - ox, uy - oy))
                })
            }),
            _ => None,
        };
        let Some(at) = self.field_hit(e, x, y) else {
            return;
        };
        let p = &mut self.input.per[e as usize];
        (p.anchor, p.caret) = match (n, word) {
            (2, Some(w)) if !p.value.is_empty() => (p.value_offset(w.start), p.value_offset(w.end)),
            (3, _) => (0, p.value.len()),
            _ => (at, at),
        };
        p.last_edit = (Edit::None, 0.0);
        self.input.selecting = (n == 1).then_some(e);
        self.paint = true;
    }

    /// The pointer moved: a press held in a field's text selects from where
    /// it went down to here.
    pub(in crate::tree) fn field_drag(&mut self, x: f32, y: f32) {
        let Some(e) = self.input.selecting else {
            return;
        };
        if self.editing() != Some(e) {
            self.input.selecting = None;
            return;
        }
        if let Some(at) = self.field_hit(e, x, y)
            && at != self.input.per[e as usize].caret
        {
            self.input.per[e as usize].caret = at;
            self.paint = true;
        }
    }

    /// ↑ or ↓ in a multi-line field: the caret goes a line up or down,
    /// keeping its x (extending the selection with `shift`).
    pub(super) fn line_key(&mut self, e: u32, down: bool, shift: bool) {
        let Some(leaf) = self.els[e as usize].leaf else {
            return;
        };
        let p = &self.input.per[e as usize];
        let shown = p.display_offset(p.caret);
        let Some(to) = self
            .texts
            .leaf(leaf)
            .map(|l| l.line_step(shown, if down { 1 } else { -1 }))
        else {
            return;
        };
        let p = &mut self.input.per[e as usize];
        p.caret = p.value_offset(to);
        if !shift {
            p.anchor = p.caret;
        }
        p.last_edit = (Edit::None, 0.0);
        self.paint = true;
    }

    /// Before paint: the focused one-line field scrolls its text so the
    /// caret shows inside its content box, no further than its text runs;
    /// a field that lost the focus shows its text's start again.
    pub(in crate::tree) fn follow_caret(&mut self) {
        let focused = self
            .editing()
            .filter(|&e| !self.input.per[e as usize].multiline);
        if let Some(was) = self.input.caret_scrolled
            && Some(was) != focused
            && let Some(p) = self.input.per.get_mut(was as usize)
            && p.scroll_x != 0.0
        {
            p.scroll_x = 0.0;
            self.paint = true;
        }
        self.input.caret_scrolled = focused;
        let Some(e) = focused else {
            return;
        };
        let el = &self.els[e as usize];
        let Some(leaf) = el.leaf else {
            return;
        };
        let p = &self.input.per[e as usize];
        let Some(caret) = caret_bar(el, p, &self.texts, &self.arena, self.scale, self.text_scale)
        else {
            return;
        };
        let b = self.arena.box_of(el.node);
        let lb = self.arena.box_of(leaf);
        let view = (b.w - b.inset[0] - b.inset[2]).max(0.0);
        let width = self.texts.leaf(leaf).map_or(0.0, |l| l.width());
        // The caret from the text's start, unscrolled.
        // (Its bar may stand past the content box's right edge, into the
        // room paint leaves it there.)
        let x = caret[0] - lb.x + p.scroll_x;
        let mut s = p.scroll_x;
        if x - s > view {
            s = x - view;
        }
        if x < s {
            s = x;
        }
        let s = s.clamp(0.0, (width - view).max(0.0));
        if s != p.scroll_x {
            self.input.per[e as usize].scroll_x = s;
            self.paint = true;
        }
    }

    /// A frame's text from the host (og-input's `InputFrame`): `text` it
    /// committed, then the input method's `composition` as it stands after
    /// that commit (`Some("")` ends it). A commit and the next word's
    /// preedit in one tick keep both.
    pub fn text_frame(&mut self, text: &str, composition: Option<&str>) {
        if !text.is_empty() {
            self.type_text(text);
        }
        if let Some(c) = composition {
            self.compose(c);
        }
    }

    /// Text typed this frame (og-input's `og.TextInput` stream, or the
    /// printable keys) goes into the focused `Input` at its caret, which
    /// hears `change` with its new text. Whether one took it.
    pub fn type_text(&mut self, text: &str) -> bool {
        let Some(e) = self.editing() else {
            return false;
        };
        let p = &mut self.input.per[e as usize];
        if p.read_only {
            return false;
        }
        let had_composing = !p.composing.is_empty();
        // A pasted line break is a newline in a multi-line field and a
        // space in a one-line one; other control characters are keys.
        let brk = if p.multiline { '\n' } else { ' ' };
        let filtered: String = text
            .replace("\r\n", "\n")
            .chars()
            .filter_map(|c| match c {
                '\n' | '\r' => Some(brk),
                c if c.is_control() => None,
                c => Some(c),
            })
            .collect();
        if filtered.is_empty() {
            if had_composing {
                p.composing.clear();
                self.show_value(e);
            }
            return false;
        }
        p.composing.clear();
        let (start, end) = p.selection_range();
        // `maxLength` counts characters, and cuts what goes in at the
        // last whole grapheme cluster that fits.
        let mut insert = filtered.as_str();
        if p.max_length > 0 {
            let kept = p.value.chars().count() - p.value[start..end].chars().count();
            let room = p.max_length.saturating_sub(kept);
            if let Some((cut, _)) = insert.char_indices().nth(room) {
                insert = &insert[..cluster_edge(insert, cut + 1, true)];
            }
        }
        if insert.is_empty() && start == end {
            if had_composing {
                self.show_value(e);
            }
            return false;
        }
        // One grapheme typed at a caret joins the typing before it; a
        // paste, a commit of several, or typing over a selection is a step.
        let one = cluster_edge(insert, 0, false) == insert.len();
        let insert = insert.to_owned();
        self.remember(
            e,
            if one && start == end {
                Edit::Type
            } else {
                Edit::Other
            },
        );
        let p = &mut self.input.per[e as usize];
        p.value.replace_range(start..end, &insert);
        p.caret = start + insert.len();
        p.anchor = p.caret;
        self.edited(e);
        true
    }

    /// An editing key on the focused `Input`: Backspace and Delete remove
    /// the grapheme cluster before and after the caret, ←/→ step over one
    /// (or a word with `word`), Home and End go to the ends. Whether it was one.
    pub(super) fn edit_key(&mut self, e: u32, code: &str, shift: bool, word: bool) -> bool {
        let p = &mut self.input.per[e as usize];
        let at = p.caret.min(p.value.len());
        match code {
            "Backspace" | "Delete" => {
                if p.read_only {
                    return true;
                }
                p.composing.clear();
                let (a, b) = if p.anchor != at {
                    (p.anchor.min(at), p.anchor.max(at))
                } else if code == "Backspace" {
                    let prev = if word {
                        word_edge(&p.value, at, true)
                    } else {
                        cluster_edge(&p.value, at, true)
                    };
                    (prev, at)
                } else {
                    let next = if word {
                        word_edge(&p.value, at, false)
                    } else {
                        cluster_edge(&p.value, at, false)
                    };
                    (at, next)
                };
                p.caret = at;
                if a < b {
                    let kind = if p.anchor == at {
                        Edit::Delete
                    } else {
                        Edit::Other
                    };
                    self.remember(e, kind);
                    let p = &mut self.input.per[e as usize];
                    p.value.replace_range(a..b, "");
                    (p.caret, p.anchor) = (a, a);
                    self.edited(e);
                } else {
                    p.anchor = a;
                    p.caret = a;
                }
                self.paint = true;
                return true;
            }
            "ArrowLeft" => {
                p.caret = if !shift && p.anchor != at {
                    p.anchor.min(at)
                } else if word {
                    word_edge(&p.value, at, true)
                } else {
                    cluster_edge(&p.value, at, true)
                }
            }
            "ArrowRight" => {
                p.caret = if !shift && p.anchor != at {
                    p.anchor.max(at)
                } else if word {
                    word_edge(&p.value, at, false)
                } else {
                    cluster_edge(&p.value, at, false)
                }
            }
            "Home" => p.caret = 0,
            "End" => p.caret = p.value.len(),
            _ => return false,
        }
        // The caret moved: the next typing is a new undo step.
        p.last_edit = (Edit::None, 0.0);
        if !shift {
            self.input.per[e as usize].anchor = self.input.per[e as usize].caret;
        }
        self.paint = true;
        true
    }

    pub(super) fn edited(&mut self, e: u32) {
        self.show_value(e);
        let value = self.input.per[e as usize].value.clone();
        self.fire_value(e, OpEvent::Change, &value);
    }
}
