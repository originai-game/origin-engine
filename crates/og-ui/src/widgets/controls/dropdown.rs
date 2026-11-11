//! The dropdown (a select). Its list is a floating layer (a focus scope
//! that Back and a press outside close, the focus returning to its
//! trigger) opening at the chosen option, with groups, icons,
//! descriptions, a tick on the chosen and a filter past 10 options.

use super::*;

/// One option of a dropdown.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DropdownOption {
    pub label: String,
    /// A line under it in the list; empty for none.
    pub description: String,
    pub icon: String,
    pub disabled: bool,
    /// The heading it sits under; options of one group follow each other.
    pub group: String,
}

/// A dropdown: its options, label and states.
#[derive(Clone, Debug, PartialEq)]
pub struct DropdownDesc {
    pub options: Vec<DropdownOption>,
    /// The chosen one; -1 shows the placeholder.
    pub selected: i32,
    pub label: String,
    /// Shown while nothing is chosen; empty is `Select…`.
    pub placeholder: String,
    pub helper: String,
    /// An error under it (and the danger frame); empty for none.
    pub error: String,
    pub disabled: bool,
    /// Its width; `None` grows.
    pub width: Option<Size>,
    /// A filter field at the list's top; `None` shows it past 10 options.
    pub searchable: Option<bool>,
    /// The filter's placeholder; empty is `Filter`.
    pub search_placeholder: String,
    /// The list's most height before it scrolls, CSS px.
    pub max_height: f32,
}

impl Default for DropdownDesc {
    fn default() -> Self {
        DropdownDesc {
            options: Vec::new(),
            selected: -1,
            label: String::new(),
            placeholder: String::new(),
            helper: String::new(),
            error: String::new(),
            disabled: false,
            width: None,
            searchable: None,
            search_placeholder: String::new(),
            max_height: 320.0,
        }
    }
}

/// A dropdown's state after this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DropdownResult {
    pub selected: i32,
    pub changed: bool,
    /// Its list is open.
    pub open: bool,
}

/// `s` lower-cased with the accents of Latin letters dropped, for a
/// filter's match (`é` finds `e`, `Ö` finds `o`).
fn fold(s: &str) -> String {
    const FROM: &str =
        "àáâãäåāăąçćĉċčďđèéêëēĕėęěĝğġģĥħìíîïĩīĭįıĵķĺļľŀłñńņňòóôõöøōŏőŕŗřśŝşšţťŧùúûüũūŭůűųŵýÿŷźżž";
    const TO: &str =
        "aaaaaaaaacccccddeeeeeeeeegggghhiiiiiiiiijklllllnnnnooooooooorrrsssstttuuuuuuuuuuwyyyzzz";
    s.chars()
        .flat_map(char::to_lowercase)
        .map(|c| {
            FROM.chars()
                .position(|f| f == c)
                .and_then(|k| TO.chars().nth(k))
                .unwrap_or(c)
        })
        .collect()
}

/// The options a filter keeps, by index: those whose label contains it
/// (case and accents ignored).
pub fn kept_options(options: &[DropdownOption], filter: &str) -> Vec<usize> {
    let q = fold(filter.trim());
    (0..options.len())
        .filter(|&i| q.is_empty() || fold(&options[i].label).contains(&q))
        .collect()
}

/// A text input's suggestions for `query`: those containing it (case and
/// accents ignored), those starting with it first, then the shorter; with
/// no query, the first ones as written. At most `max`.
fn kept_suggestions<'a>(
    all: &'a [String],
    labels: &'a [String],
    query: &str,
    max: usize,
) -> Vec<(usize, &'a str)> {
    let q = fold(query.trim());
    let mut hits: Vec<(bool, usize, &str)> = all
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            let label = labels.get(index).filter(|s| !s.is_empty()).unwrap_or(value);
            let (label_folded, value_folded) = (fold(label), fold(value));
            (q.is_empty() || label_folded.contains(&q) || value_folded.contains(&q)).then(|| {
                (
                    !label_folded.starts_with(&q) && !value_folded.starts_with(&q),
                    index,
                    label.as_str(),
                )
            })
        })
        .collect();
    if !q.is_empty() {
        hits.sort_by_key(|&(later, _, s)| (later, s.chars().count()));
    }
    hits.into_iter()
        .take(max)
        .map(|(_, index, s)| (index, s))
        .collect()
}

impl Ui {
    fn select_list_part(&mut self) {
        let owner = self.parent();
        let element = self.tree.nodes[owner].el;
        let reference = element.part.expect("select list part");
        self.tree.nodes[owner].el.padding = Insets::default();
        self.tree.nodes[owner].el.background = None;
        self.tree.nodes[owner].el.radius = 0.0;
        // The old list inset included the painted one-pixel rim. The
        // persistent CSS box now accounts for that rim in its padding.
        let padding = element.padding;
        let slot = self.write_part(
            owner,
            reference,
            ("list", 0),
            Element {
                width: Size::grow(),
                height: Size::grow(),
                padding: Insets {
                    top: padding.top - 1.0,
                    right: padding.right - 1.0,
                    bottom: padding.bottom - 1.0,
                    left: padding.left - 1.0,
                },
                part: None,
                ..element
            },
            crate::layout::Content::None,
        );
        self.tree.nodes[owner].content_slot = slot;
    }

    /// A text input's suggestion list (HTML's `list` naming a
    /// `<datalist>`): open under `field` (the field's node) while the field
    /// or one of its options has the focus, it lists the suggestions that
    /// keep the query (§ [`kept_suggestions`]). Down reaches it by spatial
    /// navigation; a press or Enter on one chooses it, and the focus goes
    /// back to the field. Back, Escape or a press outside closes it until
    /// the player types again. Dressed as a select's list: `Dropdown.list`,
    /// `Dropdown.option`. Role
    /// a listbox of options. Probe: `<id>~list/Option#i`. Answers the
    /// chosen suggestion.
    pub(crate) fn suggestions(
        &mut self,
        id: &str,
        field: usize,
        (focused, typed): (bool, bool),
        query: &str,
        all: &[String],
        labels: &[String],
    ) -> Option<String> {
        let t = self.env.theme;
        let kept = kept_suggestions(all, labels, query, 8);
        let open_key = format!("{id}~suggest");
        let shut_key = format!("{id}~shut");
        let field_key = format!("{id}~field");
        if focused {
            self.remember(&field_key, f64::from_bits(self.interaction.state.focus));
        }
        let was_open = self.memory(&open_key, 0.0) > 0.0;
        let shut = self.memory(&shut_key, 0.0) > 0.0 && !typed;
        let open = (focused || was_open) && !shut && !kept.is_empty();
        let row_h = self.by_text(if self.touch_frame() { TOUCH } else { 36.0 });
        let content = Element {
            direction: Direction::Column,
            width: Size::px(
                self.last_bounds(self.tree.nodes[field].key)
                    .map_or(240.0, |b| b.width),
            ),
            padding: Insets::xy(0.0, t.space.xxs),
            radius: t.radius.control,
            background: Some(self.role("panelRaised").srgba()),
            part: crate::dress::PartRef::parse("select::part(list)", ElementClass::Tooltip).ok(),
            ..Element::default()
        };
        let mut chosen = None;
        let mut on_option = false;
        // Back while it is open closes it and leaves the field focused.
        let dismissed = open && self.interaction.nav.back;
        if dismissed {
            self.interaction.nav.back = false;
        }
        // An anchored layer that is no focus scope: the focus stays in the
        // field while the player types (a combobox), and Down reaches it.
        if open && !dismissed {
            self.open_layer(
                "menu",
                Id::from(format!("{id}~list").as_str()),
                content,
                crate::layout::Float::Popover,
                field,
            );
            self.select_list_part();
            for (i, (index, s)) in kept.iter().enumerate() {
                let b = self.ctl(
                    crate::ui_idi("Option", i as u32),
                    Ctl {
                        part: "select::part(option)",
                        part_node: true,
                        width: Some(Size::grow()),
                        height: Some(Size::px(row_h)),
                        padding: Insets::xy(t.space.md, 0.0),
                        align: Align::Start,
                        cross: Cross::Center,
                        fill: None,
                        wash_by: 0.08,
                        lift: 0.0,
                        press: 0.99,
                        radius: Some(t.radius.small),
                        semantics: Some(Semantics {
                            role: Some("option".into()),
                            name: Some((*s).to_string()),
                            ..Semantics::default()
                        }),
                        ..Ctl::default()
                    },
                    |ui, _| {
                        ui.text(
                            "Text",
                            s,
                            Text {
                                nowrap: true,
                                ellipsis: true,
                                width: Some(Size::grow()),
                                ..Text::default()
                            },
                        );
                    },
                );
                on_option |= b.r.focused;
                if b.r.clicked {
                    chosen = Some(all[*index].clone());
                }
            }
            self.close();
            self.semantics(&Semantics {
                role: Some("listbox".into()),
                name: Some("Suggestions".into()),
                ..Semantics::default()
            });
        }
        if chosen.is_some() || dismissed {
            self.feedback(if dismissed {
                SoundEvent::Close
            } else {
                SoundEvent::Select
            });
            let back = self.memory(&field_key, 0.0).to_bits();
            if back != 0 {
                self.interaction.request_focus = back;
            }
        }
        self.remember(
            &open_key,
            if (focused || on_option) && chosen.is_none() {
                1.0
            } else {
                0.0
            },
        );
        let reopen = !(focused || on_option) || typed;
        self.remember(
            &shut_key,
            if dismissed || chosen.is_some() || (shut && !reopen) {
                1.0
            } else {
                0.0
            },
        );
        chosen
    }

    /// A dropdown (a select): a field-like trigger with the chosen option
    /// and a chevron that turns as it opens; its list opens below it (above
    /// without room, kept on screen) as a floating layer that takes the
    /// focus at the chosen option. A press, Enter, Space or A opens it and
    /// chooses; Back, Escape or a press outside closes it and the focus
    /// returns to the trigger. Plays open, select and close. Parts:
    /// `Dropdown.trigger[open][invalid]`, `Dropdown.list`,
    /// `Dropdown.option[selected]`, `Dropdown.group`. Role combobox and a
    /// listbox of options. Probe: `<id>/Trigger`,
    /// `<id>.list/List/Option#i`.
    pub fn dropdown(&mut self, id: impl Into<Id>, desc: &DropdownDesc) -> DropdownResult {
        self.dropdown_marked(id, desc, !desc.error.is_empty(), &mut |_, _| {})
    }

    /// [`Ui::dropdown`], its trigger marked invalid without an error line
    /// (a document's `aria-invalid`, its message written beside it), and
    /// `list` styling its open list's node (a document's `::picker(select)`).
    pub(crate) fn dropdown_marked(
        &mut self,
        id: impl Into<Id>,
        desc: &DropdownDesc,
        invalid: bool,
        list: &mut dyn FnMut(&mut Ui, usize),
    ) -> DropdownResult {
        let id = id.into();
        let key = id.as_str().to_owned();
        let t = self.env.theme;
        let options = &desc.options;
        let before = desc.selected;
        let mut selected = before;
        let disabled = desc.disabled;
        let mut open = self.memory(&format!("{key}~open"), 0.0) > 0.0 && !disabled;
        let was_open = open;
        let touch = self.touch_frame();
        let h = self.sizes().control_md.max(if touch { TOUCH } else { 0.0 });
        let chosen = usize::try_from(selected).ok().and_then(|i| options.get(i));
        let searchable = desc.searchable.unwrap_or(options.len() > 10);
        let (panel, well, ink, faint, muted, raised, primary) = (
            self.role("panel"),
            self.role("well"),
            self.role("ink"),
            self.role("inkFaint"),
            self.role("inkMuted"),
            self.role("panelRaised"),
            self.role("primary"),
        );
        let mut trig_path = String::new();
        let mut trig_node = 0;
        let placeholder = if desc.placeholder.is_empty() {
            "Select…"
        } else {
            &desc.placeholder
        };
        let name = [desc.label.as_str(), desc.placeholder.as_str(), key.as_str()]
            .into_iter()
            .find(|s| !s.is_empty())
            .unwrap_or_default()
            .to_owned();
        let mut toggled = false;
        self.field_chrome(
            id,
            &desc.label,
            &desc.helper,
            &desc.error,
            desc.width.unwrap_or(Size::grow()),
            |ui| {
                let icon = ui.by_text(18.0);
                let trig = ui.ctl(
                    "Trigger",
                    Ctl {
                        part: "select::part(trigger)",
                        part_node: true,
                        cls: ElementClass::Field,
                        flags: vec![open.then_some("open"), invalid.then_some("invalid")],
                        disabled,
                        width: Some(Size::grow()),
                        height: Some(Size::px(h)),
                        padding: Insets::xy(t.space.md, 0.0),
                        gap: t.space.sm,
                        fill: Some(if open { panel } else { well }),
                        wash_by: 0.05,
                        lift: 0.0,
                        press: 0.99,
                        // Closed, Up and Down open it (a popup button's keys).
                        arrows: if open {
                            0
                        } else {
                            crate::ARROW_UP | crate::ARROW_DOWN
                        },
                        semantics: Some(Semantics {
                            role: Some("combobox".into()),
                            name: Some(name.clone()),
                            value: Some(chosen.map(|c| c.label.clone()).unwrap_or_default()),
                            expanded: Some(open),
                            ..Semantics::default()
                        }),
                        ..Ctl::default()
                    },
                    |ui, _| {
                        if let Some(c) = chosen.filter(|c| !c.icon.is_empty()) {
                            ui.glyph_icon("Icon", &c.icon, icon, ink);
                        }
                        ui.text(
                            "Value",
                            chosen.map_or(placeholder, |c| c.label.as_str()),
                            Text {
                                color: Ink::Role(if chosen.is_some() && !disabled {
                                    "ink"
                                } else {
                                    "inkFaint"
                                }),
                                nowrap: true,
                                ellipsis: true,
                                width: Some(Size::grow()),
                                ..Text::default()
                            },
                        );
                        ui.chevron_down(
                            "Chevron",
                            icon,
                            if disabled { faint } else { muted },
                            open,
                            2.2,
                        );
                    },
                );
                trig_path = trig.path;
                trig_node = trig.node;
                toggled = (trig.r.clicked || (!open && trig.r.adjust_y != 0)) && !disabled;
            },
        );
        if toggled {
            open = !open;
            self.feedback(if open {
                SoundEvent::Open
            } else {
                SoundEvent::Close
            });
        }
        let anchor = self.bounds_of(&trig_path);
        let list_id = format!("{key}.list");
        let row_h = self.by_text(40f32.max(if touch { TOUCH } else { 0.0 }));
        let mut dismissed = false;
        if let Some(anchor) = anchor {
            let filter_key = format!("{key}~f");
            let mut filter = String::new();
            let mut picked: Option<usize> = None;
            let max_h = desc.max_height;
            let content = Element {
                direction: Direction::Column,
                width: Size::px(anchor.width),
                // Rows inset a step beyond the focus ring's reach: a focused
                // end row's ring stays inside the list with its ground round it,
                // so the ring never stands for the list's edge.
                padding: Insets::all(
                    t.tokens.stroke(crate::tokens::StrokeWidth::Focus) + 2.0 * t.space.xxs,
                ),
                radius: t.radius.control,
                background: Some(raised.srgba()),
                part: crate::dress::PartRef::parse("select::part(list)", ElementClass::Tooltip)
                    .ok(),
                ..Element::default()
            };
            // Its rim and shadow are the part's (a floating surface's) dressing.
            let m = self.floating(&list_id, open, trig_node, content, |ui| {
                let layer = ui.parent();
                ui.tree.nodes[layer].part_source = trig_node;
                list(ui, layer);
                ui.select_list_part();
                if searchable {
                    // The filter's text lives in the UI's text memory while the list is open.
                    let mut typed = ui.take_text(&filter_key);
                    ui.open(
                        "Filter",
                        Element {
                            direction: Direction::Column,
                            width: Size::grow(),
                            padding: Insets::xy(t.space.sm, t.space.xxs),
                            ..Element::default()
                        },
                    );
                    let ph = if desc.search_placeholder.is_empty() {
                        "Filter"
                    } else {
                        &desc.search_placeholder
                    };
                    ui.text_field(
                        "Search",
                        &mut typed,
                        &crate::ui::FieldDesc {
                            placeholder: ph.to_owned(),
                            width: Some(Size::grow()),
                            ..crate::ui::FieldDesc::default()
                        },
                    );
                    ui.close();
                    filter = typed.clone();
                    ui.keep_text(&filter_key, typed);
                }
                let kept = kept_options(options, &filter);
                let tall = kept.len() as f32 * row_h > max_h;
                let tick = ui.by_text(18.0);
                let mut body = |ui: &mut Ui| {
                    let mut group = "";
                    for &i in &kept {
                        let o = &options[i];
                        if o.group != group {
                            group = &o.group;
                            if !group.is_empty() {
                                ui.open_part(
                                    crate::ui_idi("Group", i as u32),
                                    Element {
                                        direction: Direction::Row,
                                        width: Size::grow(),
                                        padding: Insets {
                                            left: t.space.md,
                                            right: t.space.md,
                                            top: t.space.sm,
                                            bottom: t.space.xxs,
                                        },
                                        part: crate::dress::PartRef::parse(
                                            "select::part(group)",
                                            ElementClass::Panel,
                                        )
                                        .ok(),
                                        ..Element::default()
                                    },
                                );
                                ui.text(
                                    "Text",
                                    group,
                                    Text {
                                        step: Some(t.type_ramp.caption),
                                        color: Ink::Role("inkMuted"),
                                        nowrap: true,
                                        ..Text::default()
                                    },
                                );
                                ui.close();
                            }
                        }
                        let on = i as i32 == selected;
                        let off = o.disabled;
                        let b = ui.ctl(
                            crate::ui_idi("Option", i as u32),
                            Ctl {
                                part: "select::part(option)",
                                part_node: true,
                                flags: vec![on.then_some("selected")],
                                disabled: off,
                                width: Some(Size::grow()),
                                height: o.description.is_empty().then_some(Size::px(row_h)),
                                padding: Insets::xy(
                                    t.space.md,
                                    if o.description.is_empty() {
                                        0.0
                                    } else {
                                        t.space.xs
                                    },
                                ),
                                gap: t.space.sm,
                                align: Align::Start,
                                fill: on.then(|| mix(raised, primary, 0.1)),
                                wash_by: 0.08,
                                lift: 0.0,
                                press: 0.99,
                                radius: Some(t.radius.small),
                                semantics: Some(Semantics {
                                    role: Some("option".into()),
                                    name: Some(o.label.clone()),
                                    selected: Some(on),
                                    ..Semantics::default()
                                }),
                                ..Ctl::default()
                            },
                            |ui, _| {
                                if !o.icon.is_empty() {
                                    ui.glyph_icon(
                                        "Icon",
                                        &o.icon,
                                        tick,
                                        if off { faint } else { ink },
                                    );
                                }
                                ui.open(
                                    "Text",
                                    Element {
                                        direction: Direction::Column,
                                        width: Size::grow(),
                                        gap: 2.0,
                                        ..Element::default()
                                    },
                                );
                                ui.text(
                                    "Label",
                                    &o.label,
                                    Text {
                                        color: Ink::Role(if off { "inkFaint" } else { "ink" }),
                                        nowrap: true,
                                        ellipsis: true,
                                        width: Some(Size::grow()),
                                        ..Text::default()
                                    },
                                );
                                if !o.description.is_empty() {
                                    ui.text(
                                        "Description",
                                        &o.description,
                                        Text {
                                            step: Some(t.type_ramp.sm),
                                            color: Ink::Role("inkMuted"),
                                            width: Some(Size::grow()),
                                            max_lines: 2,
                                            ellipsis: true,
                                            ..Text::default()
                                        },
                                    );
                                }
                                ui.close();
                                // The chosen one's tick: never colour alone.
                                if on {
                                    ui.drawn("Tick", "tick", tick, primary, None);
                                } else {
                                    ui.spacer("Tick", Some(Size::px(tick)));
                                }
                            },
                        );
                        if b.r.clicked && !off {
                            picked = Some(i);
                        }
                    }
                    if kept.is_empty() {
                        ui.open(
                            "Empty",
                            Element {
                                direction: Direction::Row,
                                width: Size::grow(),
                                height: Size::px(row_h),
                                padding: Insets::xy(t.space.md, 0.0),
                                cross: Cross::Center,
                                ..Element::default()
                            },
                        );
                        ui.text(
                            "Text",
                            "—",
                            Text {
                                color: Ink::Role("inkMuted"),
                                ..Text::default()
                            },
                        );
                        ui.close();
                    }
                };
                if tall {
                    ui.scroll(
                        "List",
                        crate::ui::ScrollDesc {
                            width: Size::grow(),
                            height: Size::px(max_h),
                            ..crate::ui::ScrollDesc::default()
                        },
                        body,
                    );
                } else {
                    ui.open(
                        "List",
                        Element {
                            direction: Direction::Column,
                            width: Size::grow(),
                            ..Element::default()
                        },
                    );
                    body(ui);
                    ui.close();
                }
                ui.semantics(&Semantics {
                    role: Some("listbox".into()),
                    name: Some(name.clone()),
                    ..Semantics::default()
                });
                // Its rows are one group: Home, End, the page keys and
                // typeahead move among them, and the arrows wrap.
                if let Some(p) = ui.props_mut() {
                    p.focus_group = Some(crate::element::FocusGroup {
                        wrap: true,
                        memory: false,
                    });
                }
            });
            if let Some(i) = picked {
                selected = i as i32;
                open = false;
            }
            if m.dismissed {
                dismissed = true;
                open = false;
            }
            // Opening, the list takes the focus at the chosen option (the filter when it has one).
            if open && !was_open {
                let at = if searchable {
                    format!("{list_id}/Filter/Search")
                } else {
                    format!("{list_id}/List/Option#{}", selected.max(0))
                };
                self.set_focus(&at);
            }
            if !open {
                self.take_text(&filter_key);
            }
            // A pick or Back gives the focus back to the trigger (the list
            // took it by request, so no scope return was kept); a press
            // elsewhere leaves it where it lands.
            if was_open && !open && (picked.is_some() || !self.interaction.input.pressed) {
                self.set_focus(&trig_path);
            }
        }
        if selected != before {
            self.feedback(SoundEvent::Select);
        } else if dismissed && was_open {
            self.feedback(SoundEvent::Close);
        }
        self.remember(&format!("{key}~open"), if open { 1.0 } else { 0.0 });
        DropdownResult {
            selected,
            changed: selected != before,
            open,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_filter_ignores_case_and_accents() {
        let o = |l: &str| DropdownOption {
            label: l.into(),
            ..DropdownOption::default()
        };
        let opts = [o("Français"), o("English"), o("Österreich"), o("日本語")];
        assert_eq!(kept_options(&opts, "FRANC"), vec![0]);
        assert_eq!(kept_options(&opts, "oster"), vec![2]);
        assert_eq!(kept_options(&opts, "本"), vec![3]);
        assert_eq!(kept_options(&opts, " "), vec![0, 1, 2, 3]);
    }
}
