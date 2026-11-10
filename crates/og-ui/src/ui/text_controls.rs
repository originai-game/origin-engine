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
    ///
    /// Its [`crate::InputType`] (HTML `type`): a password is drawn masked
    /// (a bullet per character), is never copied, cut or killed into the
    /// kill buffer, moves and selects by words as one word, and has a
    /// reveal toggle at its end (`Reveal`, `<component>.reveal[revealed]`,
    /// a button named Show password or Hide password; a press or tap keeps
    /// the field's focus, so it is no focus stop) whose state is a widget
    /// memory, saved with the UI state; a number takes only a number's
    /// characters; a number, an email or a url whose text is not one is
    /// invalid (HTML's type mismatch), as `desc.invalid` makes it.
    pub fn text_field(
        &mut self,
        id: impl Into<Id>,
        value: &mut String,
        desc: &FieldDesc,
    ) -> Response {
        let t = self.theme;
        let sizes = self.sizes();
        // A bare field sits in a component's own frame, beside its icon; a
        // tight one has no inset at all.
        let pad = if desc.tight {
            0.0
        } else if desc.bare {
            t.space.xs
        } else {
            t.space.md
        };
        let base = self.field_component(desc.component.as_deref().unwrap_or("input"));
        // Its frame's states are known once it is laid; its text's step is
        // the same in every state (its height must not move as it focuses).
        let ink = self.field_ink(base, false, desc.invalid, false);
        let step = ink.step;
        // Tight is the line's height, but never under a finger's reach on a
        // touch screen, where the line is centred in the taller box.
        let height = if desc.tight {
            ink.line_height(value).max(self.touch_min())
        } else {
            sizes.control_md
        };
        let secret = desc.input_type == crate::InputType::Password;
        // A password's reveal toggle takes a square at the field's end:
        // its text stops before it (the start side in right to left).
        // A search box's clear button (HTML's search cancel button) takes
        // the same square, kept while it is empty so its text never moves.
        let search = desc.input_type == crate::InputType::Search;
        let number = desc.input_type == crate::InputType::Number;
        let spinner = if self.viewport.breakpoint().is_phone() {
            44.0
        } else {
            24.0_f32.max(self.touch_min())
        };
        let reveal_w = if number {
            2.0 * spinner + pad
        } else if secret || search {
            height.min(sizes.control_md)
        } else {
            0.0
        };
        let mut padding = Insets::xy(pad, 0.0);
        if self.direction == og_i18n::Direction::Rtl {
            padding.left += reveal_w;
        } else {
            padding.right += reveal_w;
        }
        let i = self.push(
            "field",
            id.into(),
            Element {
                width: desc.width.unwrap_or(Size::grow()),
                height: Size::Fixed(height),
                padding,
                interactive: true,
                radius: t.radius.control,
                ..Element::default()
            },
        );
        self.nodes[i].focusable = true;
        self.nodes[i].nav = true;
        if number {
            self.nodes[i].claim = ARROW_UP | ARROW_DOWN;
        }
        let key = self.nodes[i].key;
        let revealed = secret && self.reveal_toggle(i, base, reveal_w, height);
        let cleared = search
            && !value.is_empty()
            && !desc.readonly
            && self.clear_button(i, base, reveal_w, height);
        if cleared {
            value.clear();
        }
        let stepped = number && self.number_buttons(i, base, [spinner, height, pad], value, desc);
        self.last_closed = i;
        let mut r = self.respond(key);
        r.changed |= cleared || stepped;
        // Only the real focus edits; a forced one (the probe's) only shows.
        if self.state.focus == key {
            let inner = self.hit(key).map(|h| h.full.inset(&padding));
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
                    input_type: desc.input_type,
                    masked: secret && !revealed,
                    readonly: desc.readonly,
                },
            );
            r.changed = e.changed || cleared || stepped;
            r.submitted = e.submitted;
            r.focused = self.state.focus == key;
        }
        let focused = r.focused;
        let invalid = desc.invalid
            || desc.input_type.mismatch(value)
            || desc.constraints.fails(desc.input_type, value);
        let (shown, composition) = self.shown_text(value, focused);
        let shown = if number && !focused {
            desc.number
                .as_ref()
                .and_then(|options| {
                    value
                        .parse::<f32>()
                        .ok()
                        .filter(|n| n.is_finite())
                        .map(|n| {
                            options.format.show(
                                n,
                                desc.constraints.min.unwrap_or(0.0) as f32,
                                desc.constraints.step.unwrap_or(1.0) as f32,
                            )
                        })
                })
                .unwrap_or(shown)
        } else {
            shown
        };
        let mask = (secret && !revealed).then(|| {
            crate::edit::mask_char(&self.fonts.for_text(FontRole::Body, "\u{2022}").metrics)
        });
        let shown = match mask {
            Some(m) => shown.chars().map(|_| m).collect(),
            None => shown,
        };
        let placeholder = shown.is_empty();
        let shown = if placeholder {
            desc.placeholder.clone()
        } else {
            shown
        };
        let disabled = self.nodes[i].disabled;
        let ink = self.field_ink(base, focused, invalid, disabled);
        let (step, color) = if placeholder {
            (ink.placeholder_step, ink.placeholder)
        } else {
            (ink.step, ink.ink)
        };
        let mut content = self.text_content(&shown, step, color);
        content.wrap = false;
        // A placeholder longer than the field ends in an ellipsis; a value scrolls.
        content.ellipsis = placeholder;
        let mut edit = self.edit_view(key, focused, value, composition, false);
        edit.placeholder = placeholder;
        if let Some(target) = self
            .text_target
            .as_mut()
            .filter(|t| focused && self.state.focus == key && t.rect.width > 0.0)
        {
            target.input_type = desc.input_type;
            target.inputmode = desc.inputmode.unwrap_or(desc.input_type.default_mode());
            target.autocomplete = desc.autocomplete.clone();
            target.secret = secret;
            target.readonly = desc.readonly;
            if secret {
                target.selection.clear();
            }
        }
        self.field_part(i, base, focused, invalid, desc.bare);
        // What the probe and a screen reader read: a password masked
        // unless revealed (the mask, never the text).
        let said = match mask {
            Some(m) => value.chars().map(|_| m).collect(),
            None => value.clone(),
        };
        let n = &mut self.nodes[i];
        n.content = Content::Text(content);
        n.widget = Widget::Field(edit);
        n.probe_text = if value.is_empty() {
            desc.placeholder.clone()
        } else {
            said.clone()
        };
        // A field is named by its label (the label before it in its row),
        // else its placeholder, else its id; the placeholder describes it,
        // and its value is its text.
        n.a11y.description = Some(desc.placeholder.clone());
        n.a11y.value = Some(said);
        r
    }

    fn number_buttons(
        &mut self,
        owner: usize,
        base: crate::dress::PartRef,
        geometry: [f32; 3],
        value: &mut String,
        desc: &FieldDesc,
    ) -> bool {
        let [width, height, inset] = geometry;
        let now = self.seconds();
        let mut accelerated = self.input.shift;
        let mut direction = if self.state.focus == self.nodes[owner].key {
            -f64::from(self.nav.adjust_y)
        } else {
            0.0
        };
        self.stack.push(owner);
        for (id, class, sign, end) in [
            ("Increment", "inc", 1.0, inset),
            ("Decrement", "dec", -1.0, inset + width),
        ] {
            let i = self.leaf(
                "button",
                id.into(),
                Element {
                    width: Size::Fixed(width),
                    height: Size::Fixed(height),
                    interactive: !desc.readonly,
                    ..Element::default()
                },
            );
            let response = self.respond(self.nodes[i].key);
            self.nodes[i].props.position = crate::element::Position::Absolute;
            self.nodes[i].props.insets = crate::element::Edges {
                end: Some(end),
                top: Some(0.0),
                ..Default::default()
            };
            self.nodes[i].el.part = Some(
                crate::dress::PartRef {
                    part: "button",
                    ..base
                }
                .attr("type", "number")
                .flag(class)
                .flag_if(response.hovered, "hover")
                .flag_if(response.pressed, "pressed")
                .flag_if(desc.readonly, "disabled"),
            );
            let part = self.nodes[i].el.part.unwrap();
            self.nodes[i].shapes.push(Shape::Rect {
                frame: crate::layout::Frame::centered(
                    -width / 2.0,
                    0.0,
                    1.0,
                    (height - 2.0 * inset).max(1.0),
                ),
                radius: 0.0,
                color: self.theme.palette.ink_muted,
            });
            for side in [-1.0, 1.0] {
                self.nodes[i].shapes.push(Shape::Bar {
                    part,
                    frame: crate::layout::Frame::centered(side * 2.5, 0.0, 7.0, 2.0),
                    angle: side * sign as f32 * std::f32::consts::FRAC_PI_4,
                    color: self.theme.palette.ink,
                });
            }
            self.nodes[i].a11y.name = Some(if sign > 0.0 { "Increase" } else { "Decrease" }.into());
            self.nodes[i].disabled = desc.readonly;
            let since_id = format!("{id}~since");
            let repeat_id = format!("{id}~repeat");
            let since = self.memory(&since_id, -1.0);
            let last = self.memory(&repeat_id, -1.0);
            if response.pressed && !desc.readonly && !self.nodes[owner].disabled {
                if since < 0.0 {
                    direction = sign;
                    self.remember(&since_id, now);
                    self.remember(&repeat_id, now);
                } else {
                    let age = now - since;
                    let every = if age > 1.6 { 0.05 } else { 0.12 };
                    if age > 0.4 && now - last >= every {
                        direction = sign;
                        accelerated |= age > 1.0;
                        self.remember(&repeat_id, now);
                    }
                }
            } else {
                if response.clicked && since < 0.0 && !desc.readonly {
                    direction = sign;
                }
                self.remember(&since_id, -1.0);
                self.remember(&repeat_id, -1.0);
            }
        }
        self.stack.pop();
        if direction == 0.0 || desc.readonly {
            return false;
        }
        let step = desc.constraints.step.filter(|s| *s > 0.0).unwrap_or(1.0);
        let options = desc.number.as_ref();
        let amount = if accelerated {
            options
                .map(|o| f64::from(o.large_step))
                .filter(|s| *s > 0.0)
                .unwrap_or(step * 10.0)
        } else {
            step
        };
        let (min, max) = (desc.constraints.min, desc.constraints.max);
        let base = min.unwrap_or(0.0);
        let current = value
            .parse::<f64>()
            .ok()
            .filter(|n| n.is_finite())
            .unwrap_or(0.0);
        let grid = (current - base) / step;
        let count = (amount / step).round().max(1.0);
        let index = if direction > 0.0 {
            (grid + 1e-9).floor() + count
        } else {
            (grid - 1e-9).ceil() - count
        };
        let max = max.map(|max| base + ((max - base) / step + 1e-9).floor() * step);
        let mut next = base + index * step;
        if options.is_some_and(|o| o.wrap)
            && let (Some(min), Some(max)) = (min, max)
        {
            if next > max {
                next = min;
            } else if next < min {
                next = max;
            }
        }
        if let Some(min) = min {
            next = next.max(min);
        }
        if let Some(max) = max {
            next = next.min(max);
        }
        if !next.is_finite() {
            return false;
        }
        let decimals = |n: f64| {
            n.to_string()
                .split_once('.')
                .map_or(0, |(_, tail)| tail.len())
        };
        let precision = decimals(step).max(decimals(base));
        let next = if precision == 0 {
            next.to_string()
        } else {
            format!("{next:.precision$}")
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        };
        let changed = *value != next;
        *value = next;
        changed
    }

    /// A password field's reveal toggle (`<id>/Reveal`): a square button
    /// at field `i`'s end, dressed `<component>.reveal[revealed]`, drawing
    /// an eye (struck through while revealed: a press masks it again),
    /// named Show password or Hide password. Answers whether the text
    /// shows, after this tick's press.
    fn reveal_toggle(&mut self, i: usize, base: crate::dress::PartRef, w: f32, h: f32) -> bool {
        use std::f32::consts::FRAC_PI_2;
        let field = self.nodes[i].key;
        let memory = child_key(field, "reveal");
        let mut revealed = self.has_anim(memory) && self.anim(memory).h == 1;
        self.stack.push(i);
        let j = self.leaf(
            "button",
            Id::from("Reveal"),
            Element {
                width: Size::Fixed(w),
                height: Size::Fixed(h),
                interactive: true,
                radius: self.theme.radius.control,
                ..Element::default()
            },
        );
        self.stack.pop();
        self.nodes[j].props.position = crate::element::Position::Absolute;
        self.nodes[j].props.insets = crate::element::Edges {
            end: Some(0.0),
            top: Some(0.0),
            ..Default::default()
        };
        // No focus stop of its own: a press gives the focus to the field
        // around it (the caret stays, a phone's keyboard stays up), and a
        // screen reader presses it by its button.
        let r = self.respond(self.nodes[j].key);
        if r.clicked && !self.nodes[i].disabled {
            revealed = !revealed;
        }
        self.set_anim(
            memory,
            Anim {
                h: u64::from(revealed),
                ..Anim::default()
            },
        );
        let p = self.theme.palette;
        let part = crate::dress::PartRef {
            part: "reveal",
            ..base
        }
        .flag_if(revealed, "revealed")
        .flag_if(r.hovered, "hover")
        .flag_if(r.pressed, "pressed");
        let icon = crate::dress::PartRef {
            part: "reveal-icon",
            class: None,
            ..base
        }
        .flag_if(revealed, "revealed");
        let color = if r.hovered { p.ink } else { p.ink_muted };
        // An almond of two arcs, a pupil, and a stroke across while revealed.
        let s = (w.min(h) * 0.5).max(8.0);
        let (ew, eh) = (s * 0.9, s * 0.55);
        let radius = (ew * ew / 4.0 + eh * eh / 4.0) / eh;
        let half = ((ew / 2.0) / radius).clamp(-1.0, 1.0).asin();
        let thick = (s * 0.09).max(1.0);
        let n = &mut self.nodes[j];
        n.el.part = Some(part);
        n.shapes.clear();
        for (dy, mid) in [
            (radius - eh / 2.0, -FRAC_PI_2),
            (eh / 2.0 - radius, FRAC_PI_2),
        ] {
            n.shapes.push(Shape::Arc {
                part: icon,
                frame: crate::layout::Frame::centered(0.0, dy, 2.0 * radius, 2.0 * radius),
                start: mid - half,
                sweep: 2.0 * half,
                thickness: thick,
                color,
            });
        }
        let pupil = s * 0.26;
        n.shapes.push(Shape::Rect {
            frame: crate::layout::Frame::centered(0.0, 0.0, pupil, pupil),
            radius: pupil,
            color,
        });
        if revealed {
            n.shapes.push(Shape::Bar {
                part: icon,
                frame: crate::layout::Frame::centered(0.0, 0.0, s * 1.05, thick),
                angle: std::f32::consts::FRAC_PI_4,
                color,
            });
        }
        let label = if revealed {
            "Hide password"
        } else {
            "Show password"
        };
        n.probe_text = label.into();
        n.a11y.name = Some(label.into());
        n.a11y.checked = Some(revealed);
        revealed
    }

    /// A search box's clear button (`<id>/Clear`, HTML's search cancel
    /// button): a square button at field `i`'s end while it has text,
    /// dressed `<component>.clear[hover|pressed]`, drawing a cross, named
    /// Clear. Like the reveal toggle it is no focus stop (a press keeps
    /// the field's focus). Answers whether it was pressed this tick.
    fn clear_button(&mut self, i: usize, base: crate::dress::PartRef, w: f32, h: f32) -> bool {
        self.stack.push(i);
        let j = self.leaf(
            "button",
            Id::from("Clear"),
            Element {
                width: Size::Fixed(w),
                height: Size::Fixed(h),
                interactive: true,
                radius: self.theme.radius.control,
                ..Element::default()
            },
        );
        self.stack.pop();
        self.nodes[j].props.position = crate::element::Position::Absolute;
        self.nodes[j].props.insets = crate::element::Edges {
            end: Some(0.0),
            top: Some(0.0),
            ..Default::default()
        };
        let r = self.respond(self.nodes[j].key);
        let p = self.theme.palette;
        let part = crate::dress::PartRef {
            part: "clear",
            ..base
        }
        .flag_if(r.hovered, "hover")
        .flag_if(r.pressed, "pressed");
        let icon = crate::dress::PartRef {
            part: "clear-icon",
            class: None,
            ..base
        };
        let color = if r.hovered { p.ink } else { p.ink_muted };
        let s = (w.min(h) * 0.32).max(6.0);
        let thick = (s * 0.16).max(1.0);
        let n = &mut self.nodes[j];
        n.el.part = Some(part);
        n.shapes.clear();
        for angle in [std::f32::consts::FRAC_PI_4, -std::f32::consts::FRAC_PI_4] {
            n.shapes.push(Shape::Bar {
                part: icon,
                frame: crate::layout::Frame::centered(0.0, 0.0, s * 1.4, thick),
                angle,
                color,
            });
        }
        n.probe_text = "Clear".into();
        n.a11y.name = Some("Clear".into());
        r.clicked && !self.nodes[i].disabled
    }

    /// A field's frame as a part (`Field.frame[focused][invalid]`): its ground moves
    /// from `control` to `panel` on the `ui` spring as it takes the focus.
    /// A bare field draws no frame and no ring: its component frames it
    /// and shows its focus.
    fn field_part(
        &mut self,
        i: usize,
        base: crate::dress::PartRef,
        focused: bool,
        invalid: bool,
        bare: bool,
    ) {
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
        let part = base
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
        let base = self.field_component(desc.component.as_deref().unwrap_or("textarea"));
        let ink = self.field_ink(base, false, desc.invalid, false);
        let step = ink.step;
        let rows = if desc.rows == 0 { 4 } else { desc.rows };
        let lh = ink.line_height(value);
        let pad = if desc.tight {
            Insets::default()
        } else {
            Insets::xy(t.space.md, t.space.sm)
        };
        let i = self.leaf(
            "textarea",
            id.into(),
            Element {
                width: desc.width.unwrap_or(Size::grow()),
                height: Size::Fixed((rows as f32 * lh + pad.vertical()).max(self.touch_min())),
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
        // Only the real focus edits; a forced one (the probe's) only shows.
        if self.state.focus == key {
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
                    input_type: crate::InputType::Text,
                    masked: false,
                    readonly: desc.readonly,
                },
            );
            r.changed = e.changed;
            r.submitted = e.submitted;
            r.focused = self.state.focus == key;
        }
        let focused = r.focused;
        let (shown, composition) = self.shown_text(value, focused);
        let mut edit = self.edit_view(key, focused, value, composition, true);
        edit.placeholder = shown.is_empty();
        let invalid = desc.invalid || desc.constraints.fails(crate::InputType::Text, value);
        if let Some(target) = self
            .text_target
            .as_mut()
            .filter(|_| focused && self.state.focus == key)
        {
            target.readonly = desc.readonly;
        }
        let disabled = self.nodes[i].disabled;
        let ink = self.field_ink(base, focused, invalid, disabled);
        self.field_part(i, base, focused, invalid, desc.bare);
        let n = &mut self.nodes[i];
        n.widget = Widget::TextArea {
            edit,
            text: shown,
            placeholder: desc.placeholder.clone(),
            line_height: lh,
            ink,
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

    /// Supply textarea text to layout only when CSS requests content sizing.
    /// Editing and scrolling still paint through `text_area_commands`.
    pub(crate) fn field_measurement(&mut self, i: usize, content: bool) -> Option<f32> {
        if let Widget::TextArea {
            text,
            placeholder,
            ink,
            line_height,
            ..
        } = &self.nodes[i].widget
        {
            let height = *line_height;
            let measured = if content {
                let (shown, step, color) = if text.is_empty() {
                    (placeholder, ink.placeholder_step, ink.placeholder)
                } else {
                    (text, ink.step, ink.ink)
                };
                let mut measured = self.text_content(shown, step, color);
                measured.step.line_height = height;
                measured.step.cjk_line_height = height;
                Content::Text(measured)
            } else {
                Content::None
            };
            self.nodes[i].content = measured;
            return content.then_some(height);
        }
        if content
            && matches!(self.nodes[i].widget, Widget::Field(_))
            && let Content::Text(text) = &self.nodes[i].content
        {
            return Some(text.line_height());
        }
        None
    }

    /// The text control's markup host, including classes and attributes.
    fn field_component(&mut self, component: &str) -> crate::dress::PartRef {
        let class = crate::tokens::ElementClass::Field;
        match crate::dress::PartRef::parse(&format!("{component}::part(frame)"), class) {
            Ok(p) => p,
            Err(error) => {
                self.warnings
                    .push(format!("text control host '{component}': {error}"));
                crate::dress::PartRef::of("", "frame", None)
            }
        }
    }

    /// A text control's text step and inks: its component's `text` and
    /// `placeholder` parts in its states (`ink`, `step`), else the body
    /// step in ink and the placeholder in inkFaint at the text's step.
    fn field_ink(
        &self,
        base: crate::dress::PartRef,
        focused: bool,
        invalid: bool,
        disabled: bool,
    ) -> FieldInk {
        let t = &self.theme;
        let of = |part: &'static str| {
            let mut p = base
                .flag_if(focused, "focused")
                .flag_if(invalid, "invalid")
                .flag_if(disabled, "disabled");
            p.part = part;
            self.part_values(&p, None)
        };
        let (text, hint) = (of("text"), of("placeholder"));
        let step = text.part_type(t, t.type_ramp.body);
        FieldInk {
            step,
            placeholder_step: hint.part_type(t, step),
            ink: text.color("color").unwrap_or(t.palette.ink),
            placeholder: self
                .docs
                .placeholder_ink
                .unwrap_or_else(|| hint.color("color").unwrap_or(t.palette.ink_faint)),
        }
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
                ..TextTarget::default()
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
            placeholder: false,
        }
    }
}

impl Ui {
    pub(crate) fn edit_marks(&self, i: usize, mk: &EditMarks<'_>, cmds: &mut Vec<DrawCmd>) {
        let p = &self.theme.palette;
        let dpr = self.viewport.dpr();
        let boxes = crate::edit::boxes(mk.m, mk.text, self.direction, mk.size, mk.tracking);
        let n = mk.text.chars().count();
        let mark = |name, rect: Rect, color: Srgba, cmds: &mut Vec<DrawCmd>| {
            let part = crate::dress::PartRef::of("TextEdit", name, None);
            let values = self.part_values(&part, Some(self.nodes[i].key));
            self.part_commands_at(
                Some(self.nodes[i].key),
                &part,
                values.part_rect(rect, self.nodes[i].rect).snapped(dpr),
                Some(values.color("color").unwrap_or(color)),
                0.0,
                mk.alpha,
                mk.clip,
                cmds,
            );
        };
        let span =
            |name, range: (usize, usize), cmds: &mut Vec<DrawCmd>, y: f32, h: f32, color: Srgba| {
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
                    mark(name, Rect::new(mk.x + x0, y, x1 - x0, h), color, cmds);
                }
            };
        let e = mk.edit;
        let ch = mk.size + self.theme.space.xxs * 2.0;
        let cy = mk.top + (mk.line_h - ch) / 2.0;
        if mk.under {
            let selection = self
                .docs
                .selection
                .get(&self.nodes[i].key)
                .and_then(|s| s.background)
                .unwrap_or(p.control_strong);
            span("selection", e.selection, cmds, cy, ch, selection);
            return;
        }
        if let Some(c) = e.composition {
            let th = (mk.size * 0.07).max(1.0 / dpr);
            span("composition", c, cmds, cy + ch - th, th, p.ink);
        }
        if e.caret_on && mk.caret_here && e.caret >= mk.first && e.caret <= mk.first + n {
            let cx = mk.x + crate::edit::caret_x_at(&boxes, e.caret - mk.first, e.upstream);
            let caret_w = 1.0f32.max(1.0 / dpr);
            let caret = self.nearest(i, |n| n.caret).unwrap_or(p.primary);
            mark("caret", Rect::new(cx, cy, caret_w, ch), caret, cmds);
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
            ink,
        } = &n.widget
        else {
            return;
        };
        let t = &self.theme;
        let (shown, color, step) = if text.is_empty() {
            (placeholder.as_str(), ink.placeholder, ink.placeholder_step)
        } else {
            (text.as_str(), ink.ink, ink.step)
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
        let snap = |v: f32| crate::layout::snap(v, dpr);
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
                        i,
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

/// A text control's text, as its style dresses it ([`Ui::field_ink`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FieldInk {
    pub step: TypeStep,
    pub placeholder_step: TypeStep,
    pub ink: Srgba,
    pub placeholder: Srgba,
}

impl FieldInk {
    /// One line of `value` (or its placeholder): the taller of its value's
    /// and its placeholder's lines, CJK's where the value has CJK.
    pub fn line_height(&self, value: &str) -> f32 {
        let cjk = value.chars().any(crate::text::is_cjk);
        let lh = |s: &TypeStep| {
            if cjk {
                s.cjk_line_height
            } else {
                s.line_height
            }
        };
        lh(&self.step).max(lh(&self.placeholder_step))
    }
}
