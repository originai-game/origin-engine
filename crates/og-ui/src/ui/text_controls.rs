use super::*;

impl Ui {
    /// A one-line text field: a press focuses it (caret at the press, a
    /// drag selects and scrolls the text past the field's edge, a double
    /// click selects a word and a triple click everything), typed text
    /// replaces the selection at the caret, Backspace/Delete/←/→/Home/End
    /// edit (Shift extends the selection; Alt or Ctrl moves and deletes by
    /// words, Ctrl+Home/End goes to the ends; Ctrl or Cmd with A selects
    /// all, with C copies and with X cuts, for the host's clipboard, with Z
    /// undoes and with Shift+Z or Y redoes, from a history of the last 100
    /// steps with typing grouped by words), an input method's composition
    /// shows underlined at the caret until it commits, Enter submits and
    /// Escape leaves. The text scrolls to keep the caret in view; the caret
    /// blinks by the tick (steady under reduced motion).
    pub fn text_field(
        &mut self,
        id: impl Into<Id>,
        value: &mut String,
        desc: &FieldDesc,
    ) -> Response {
        let t = self.theme;
        let sizes = self.sizes();
        // A bare field sits in a component's own frame, beside its icon.
        let pad = if desc.bare { t.space.xs } else { t.space.md };
        let i = self.leaf(
            "field",
            id.into(),
            Element {
                width: desc.width.unwrap_or(Size::grow()),
                height: Size::Fixed(sizes.control_md),
                padding: Insets::xy(pad, 0.0),
                interactive: true,
                radius: t.radius.control,
                ..Element::default()
            },
        );
        self.nodes[i].focusable = true;
        self.nodes[i].nav = true;
        let key = self.nodes[i].key;
        let mut r = self.respond(key);
        let step = t.type_ramp.body;
        if r.focused {
            let inner = self.hit(key).map(|h| h.full.inset(&Insets::xy(pad, 0.0)));
            let e = self.edit(
                key,
                value,
                &crate::edit::EditDesc {
                    multiline: false,
                    max_chars: desc.max_chars,
                    role: FontRole::Body,
                    size: step.size,
                    tracking: crate::layout::step_tracking(&step, value),
                    line_height: step.line_height,
                    inner,
                },
            );
            r.changed = e.changed;
            r.submitted = e.submitted;
            r.focused = self.state.focus == key;
        }
        let focused = r.focused;
        let (shown, composition) = self.shown_text(value, focused);
        let placeholder = shown.is_empty();
        let shown = if placeholder {
            desc.placeholder.clone()
        } else {
            shown
        };
        let color = if placeholder {
            t.palette.ink_faint
        } else {
            t.palette.ink
        };
        let mut content = self.text_content(&shown, step, color);
        content.wrap = false;
        // A placeholder longer than the field ends in an ellipsis; a value scrolls.
        content.ellipsis = placeholder;
        let edit = self.edit_view(key, focused, value, composition, false);
        self.field_part(i, focused, desc.invalid, desc.bare);
        let n = &mut self.nodes[i];
        n.content = Content::Text(content);
        n.widget = Widget::Field(edit);
        n.probe_text = if value.is_empty() {
            desc.placeholder.clone()
        } else {
            value.clone()
        };
        // A field is named by its label (the label before it in its row),
        // else its placeholder, else its id; the placeholder describes it,
        // and its value is its text.
        n.a11y.description = Some(desc.placeholder.clone());
        n.a11y.value = Some(value.clone());
        r
    }

    /// A field's frame as a part (`Field.frame[focused][invalid]`): its ground moves
    /// from `control` to `panel` on the `ui` spring as it takes the focus.
    /// A bare field draws no frame and no ring: its component frames it
    /// and shows its focus.
    fn field_part(&mut self, i: usize, focused: bool, invalid: bool, bare: bool) {
        if bare {
            let n = &mut self.nodes[i];
            n.el.background = None;
            n.el.part = None;
            return;
        }
        let p = self.theme.palette;
        let key = self.nodes[i].key;
        let f = self.spring_to(child_key(key, "focus"), if focused { 1.0 } else { 0.0 });
        let n = &mut self.nodes[i];
        n.el.background = Some(mix_srgba(p.control, p.panel, f.clamp(0.0, 1.0)));
        let part = crate::dress::PartRef::new("Field", "frame", crate::tokens::ElementClass::Field)
            .flag_if(focused, "focused")
            .flag_if(invalid, "invalid")
            .flag_if(n.disabled, "disabled");
        n.el.part = Some(part);
    }

    /// A multi-line text area: [`Ui::text_field`]'s editing, with Enter
    /// starting a new line, ↑ and ↓ moving between lines (and Home and End
    /// to a line's ends), a triple click selecting the line between its
    /// breaks, the text wrapped between words at the area's width and
    /// scrolled to keep the caret's line in view (a drag above or below it
    /// scrolls it, faster the further out). `rows` lines show at once.
    pub fn text_area(
        &mut self,
        id: impl Into<Id>,
        value: &mut String,
        desc: &TextAreaDesc,
    ) -> Response {
        let t = self.theme;
        let step = t.type_ramp.body;
        let rows = if desc.rows == 0 { 4 } else { desc.rows };
        let lh = if value.chars().any(crate::text::is_cjk) {
            step.cjk_line_height
        } else {
            step.line_height
        };
        let pad = Insets::xy(t.space.md, t.space.sm);
        let i = self.leaf(
            "textarea",
            id.into(),
            Element {
                width: desc.width.unwrap_or(Size::grow()),
                height: Size::Fixed(rows as f32 * lh + pad.vertical()),
                padding: pad,
                interactive: true,
                radius: t.radius.control,
                ..Element::default()
            },
        );
        self.nodes[i].focusable = true;
        self.nodes[i].nav = true;
        self.nodes[i].multiline = true;
        let key = self.nodes[i].key;
        let mut r = self.respond(key);
        if r.focused {
            let inner = self.hit(key).map(|h| h.full.inset(&pad));
            let e = self.edit(
                key,
                value,
                &crate::edit::EditDesc {
                    multiline: true,
                    max_chars: desc.max_chars,
                    role: FontRole::Body,
                    size: step.size,
                    tracking: crate::layout::step_tracking(&step, value),
                    line_height: lh,
                    inner,
                },
            );
            r.changed = e.changed;
            r.submitted = e.submitted;
            r.focused = self.state.focus == key;
        }
        let focused = r.focused;
        let (shown, composition) = self.shown_text(value, focused);
        let edit = self.edit_view(key, focused, value, composition, true);
        self.field_part(i, focused, desc.invalid, desc.bare);
        let n = &mut self.nodes[i];
        n.widget = Widget::TextArea {
            edit,
            text: shown,
            placeholder: desc.placeholder.clone(),
            line_height: lh,
        };
        n.probe_text = if value.is_empty() {
            desc.placeholder.clone()
        } else {
            value.clone()
        };
        // Named and described as a field is.
        n.a11y.description = Some(desc.placeholder.clone());
        n.a11y.value = Some(value.clone());
        r
    }

    /// A text control's text as shown: its value with the composition (if
    /// focused and composing) at the caret, and the composition's range.
    fn shown_text(&self, value: &str, focused: bool) -> (String, Option<(usize, usize)>) {
        if !focused || self.state.composition.is_empty() {
            return (value.to_string(), None);
        }
        let at = self.state.caret.min(value.chars().count());
        let mut out: String = value.chars().take(at).collect();
        out.push_str(&self.state.composition);
        out.extend(value.chars().skip(at));
        let n = self.state.composition.chars().count();
        (out, Some((at, at + n)))
    }

    /// The caret, selection and composition of text control `key` for
    /// drawing (in the shown text's characters), and the host's view of it.
    fn edit_view(
        &mut self,
        key: u64,
        focused: bool,
        value: &str,
        composition: Option<(usize, usize)>,
        multiline: bool,
    ) -> crate::layout::EditView {
        if !focused {
            return crate::layout::EditView::default();
        }
        let len = value.chars().count();
        let caret = self.state.caret.min(len);
        let anchor = self.state.anchor.min(len);
        let (a, b) = (caret.min(anchor), caret.max(anchor));
        let n = composition.map_or(0, |(s, e)| e - s);
        if let Some(h) = self.hit(key) {
            let path = self
                .nodes
                .iter()
                .position(|n| n.key == key)
                .map(|i| self.path_of(i))
                .unwrap_or_default();
            self.text_target = Some(TextTarget {
                path,
                rect: h.full,
                multiline,
                selection: value.chars().skip(a).take(b - a).collect(),
                composing: n > 0,
            });
        }
        crate::layout::EditView {
            focused: true,
            caret: caret + n,
            selection: (a, b),
            composition,
            scroll: self.state.edit_scroll,
            caret_on: self.caret_shown(),
            upstream: self.state.caret_upstream && n == 0,
        }
    }
}

impl Ui {
    pub(crate) fn edit_marks(&self, mk: &EditMarks<'_>, cmds: &mut Vec<DrawCmd>) {
        let p = &self.theme.palette;
        let dpr = self.viewport.dpr();
        let boxes = crate::edit::boxes(mk.m, mk.text, self.direction, mk.size, mk.tracking);
        let n = mk.text.chars().count();
        let span =
            |range: (usize, usize), cmds: &mut Vec<DrawCmd>, y: f32, h: f32, color: Srgba| {
                let (a, b) = (range.0.max(mk.first), range.1.min(mk.first + n));
                if a >= b {
                    return;
                }
                let mut runs: Vec<(f32, f32)> = Vec::new();
                for g in &boxes {
                    let k = g.index + mk.first;
                    if k < a || k >= b {
                        continue;
                    }
                    match runs.last_mut() {
                        Some(r) if (r.1 - g.x0).abs() < 0.01 => r.1 = g.x1,
                        _ => runs.push((g.x0, g.x1)),
                    }
                }
                for (x0, x1) in runs {
                    cmds.push(DrawCmd::Quad {
                        rect: Rect::new(mk.x + x0, y, x1 - x0, h).snapped(dpr),
                        radius: 0.0,
                        color: color.fade(mk.alpha).linear(),
                        clip: mk.clip,
                    });
                }
            };
        let e = mk.edit;
        let ch = mk.size + self.theme.space.xxs * 2.0;
        let cy = mk.top + (mk.line_h - ch) / 2.0;
        if mk.under {
            span(e.selection, cmds, cy, ch, p.control_strong);
            return;
        }
        if let Some(c) = e.composition {
            let th = (mk.size * 0.07).max(1.0 / dpr);
            span(c, cmds, cy + ch - th, th, p.ink);
        }
        if e.caret_on && mk.caret_here && e.caret >= mk.first && e.caret <= mk.first + n {
            let cx = mk.x + crate::edit::caret_x_at(&boxes, e.caret - mk.first, e.upstream);
            let caret_w = 1.0f32.max(1.0 / dpr);
            cmds.push(DrawCmd::Quad {
                rect: Rect::new(cx, cy, caret_w, ch).snapped(dpr),
                radius: 0.0,
                color: p.primary.fade(mk.alpha).linear(),
                clip: mk.clip,
            });
        }
    }

    /// A text area's lines: wrapped at its inner width, scrolled, clipped
    /// to it, each with its marks.
    pub(super) fn text_area_commands(
        &self,
        i: usize,
        rect: Rect,
        clip: Rect,
        cmds: &mut Vec<DrawCmd>,
    ) {
        let n = &self.nodes[i];
        let Widget::TextArea {
            edit,
            text,
            placeholder,
            line_height,
        } = &n.widget
        else {
            return;
        };
        let t = &self.theme;
        let p = &t.palette;
        let step = t.type_ramp.body;
        let (shown, color) = if text.is_empty() {
            (placeholder.as_str(), p.ink_faint)
        } else {
            (text.as_str(), p.ink)
        };
        let font = self.fonts.for_text(FontRole::Body, shown);
        if font.handle.is_none() {
            return;
        }
        let m = &font.metrics;
        let sk = n.scale;
        let size = step.size * sk;
        let lh = line_height * sk;
        let tracking = crate::layout::step_tracking(&step, shown);
        let pad = n.el.padding;
        let inner = Rect::new(
            rect.x + pad.left * sk,
            rect.y + pad.top * sk,
            rect.width - pad.horizontal() * sk,
            rect.height - pad.vertical() * sk,
        );
        let clip = clip.intersect(&rect);
        let dpr = self.viewport.dpr();
        let snap = |v: f32| (v * dpr).round() / dpr;
        let chars: Vec<char> = shown.chars().collect();
        let lines = crate::edit::wrap(&chars, m, size, tracking, inner.width.max(1.0));
        let caret_line = lines
            .iter()
            .rposition(|l| l.0 <= edit.caret && edit.caret <= l.1)
            .unwrap_or(0);
        let rtl = self.direction == og_i18n::Direction::Rtl;
        let alpha = n.alpha;
        for (li, &(a, b)) in lines.iter().enumerate() {
            let top = inner.y + li as f32 * lh - edit.scroll * sk;
            if top + lh < inner.y - lh || top > inner.bottom() + lh {
                continue;
            }
            let line: String = chars[a..b].iter().collect();
            let w = m.measure_tracked(&line, size, tracking);
            let x = if rtl { inner.right() - w } else { inner.x };
            let marks = |under: bool, cmds: &mut Vec<DrawCmd>| {
                if edit.focused && !text.is_empty() || (edit.focused && !under) {
                    self.edit_marks(
                        &EditMarks {
                            m,
                            text: if text.is_empty() { "" } else { &line },
                            x,
                            top,
                            line_h: lh,
                            size,
                            tracking,
                            first: a,
                            edit,
                            clip,
                            alpha,
                            under,
                            caret_here: li == caret_line,
                        },
                        cmds,
                    );
                }
            };
            marks(true, cmds);
            let baseline = m.baseline(&line, top, lh, size);
            let pieces = crate::text::pieces(font, &line, self.direction, tracking);
            let one = pieces.len() == 1;
            for pc in pieces {
                let Some(handle) = pc.font.filter(|_| !pc.text.is_empty()) else {
                    continue;
                };
                let ox = pc.dx * size;
                cmds.push(DrawCmd::Text {
                    x: snap(x) + ox,
                    baseline: snap(baseline),
                    size,
                    text: pc.text,
                    color: color.fade(alpha).linear(),
                    font: handle,
                    bounds: Rect::new(x + ox, top, if one { w } else { pc.width * size }, lh),
                    clip,
                    tracking,
                    weight: t.art.text_weights[0] * sk / dpr,
                    glow: None,
                    placements: pc.placements,
                });
            }
            marks(false, cmds);
        }
    }
}
