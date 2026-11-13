//! A record's fields: sections, rows and their editors.

use super::*;

/// The fold's mark: a chevron drawn as a path (pointing down when open,
/// toward the name when closed), so it needs nothing of the game's icons.
pub(super) fn glyph(ui: &mut Ui, open: bool) {
    let p = ui.theme().palette;
    let s = 14.0;
    let pts: [[f32; 2]; 3] = if open {
        [[0.24, 0.38], [0.5, 0.64], [0.76, 0.38]]
    } else {
        [[0.38, 0.24], [0.64, 0.5], [0.38, 0.76]]
    };
    ui.open(
        "Chevron",
        Element {
            width: Size::Fixed(s),
            height: Size::Fixed(s),
            ..Element::default()
        },
    );
    ui.set_draw(og_ui::paint::ElementDraw {
        shape: og_ui::paint::ShapeKind::Path {
            points: pts.iter().map(|[x, y]| [x * s, y * s]).collect(),
            closed: false,
            filled: false,
            curves: false,
            fill_rule: og_ui::paint::FillRule::EvenOdd,
            contours: Vec::new(),
            width: 1.75,
            cap: og_ui::paint::Cap::Round,
            join: og_ui::paint::Join::Round,
        },
        fills: vec![og_ui::paint::Paint::Solid(og_ui::paint::ColorRef::Literal(
            p.ink_muted,
        ))],
        ..og_ui::paint::ElementDraw::default()
    });
    ui.close();
}

/// A square fold control: true when it was clicked.
pub(super) fn chevron(ui: &mut Ui, id: &str, open: bool, size: f32) -> bool {
    let t = *ui.theme();
    ui.container(
        id,
        Element {
            width: Size::Fixed(size),
            height: Size::Fixed(size),
            align: Align::Center,
            cross: Cross::Center,
            radius: t.radius.small,
            interactive: true,
            ..Element::default()
        },
        |ui| glyph(ui, open),
    )
    .clicked
}

/// A component's or resource's header: true when it was clicked (fold).
pub(super) fn section(ui: &mut Ui, name: &str, open: bool) -> bool {
    let t = *ui.theme();
    let p = t.palette;
    let r = ui.container(
        format!("S:{name}"),
        Element {
            direction: Direction::Row,
            width: Size::grow(),
            height: Size::Fixed(28.0),
            cross: Cross::Center,
            gap: t.space.xs,
            padding: Insets::xy(t.space.xs, 0.0),
            radius: t.radius.control,
            interactive: true,
            background: Some(p.well),
            ..Element::default()
        },
        |ui| {
            glyph(ui, open);
            ui.label(
                "Name",
                name,
                LabelDesc {
                    step: Some(t.type_ramp.sm),
                    color: Some(p.ink),
                    nowrap: true,
                    ellipsis: true,
                    width: Size::grow(),
                    ..LabelDesc::default()
                },
            );
        },
    );
    r.clicked
}

/// The field rows of one record: `prefix` is its `set` target before the
/// field (`<entity>.<Component>` or `<Resource>`).
pub(super) fn fields(
    ui: &mut Ui,
    s: &mut State,
    cx: &Ctx,
    prefix: &str,
    entity: Option<Entity>,
    schema: &Schema,
    bytes: &[u8],
) {
    let layout = Layout::of(schema);
    for (i, f) in schema.fields.iter().enumerate() {
        let Some(slice) = bytes.get(layout.range(i)) else {
            continue;
        };
        let target = format!("{prefix}.{}", f.name);
        let kind = kind_of(cx.layouts, &schema.name, &f.name);
        let editor = model::editor(f.ty, kind);
        let world = cx.world;
        let mut shown = model::parts(editor, f, slice, &|e| cx.names.id_of(world, e));
        if editor == Editor::ReadOnly
            && let Some(k) = kind.filter(|k| k.is_handle())
            && let Some(bits) = slice.get(..8).and_then(|b| b.try_into().ok())
        {
            shown = vec![handle_name(cx, k, u64::from_le_bytes(bits))];
        }
        let current = model::current(editor, f, slice);
        field_row(ui, s, cx, &target, f, editor, &shown, &current, entity);
    }
}

/// A handle as the panel shows it: the sound's or asset's id when the
/// engine loaded it, else its kind and index.
pub(super) fn handle_name(cx: &Ctx, kind: crate::host::codec::Kind, bits: u64) -> String {
    use crate::host::codec::Kind;
    if bits == 0 {
        return "none".into();
    }
    let named = match kind {
        Kind::Sound => cx.sounds.get(&bits),
        Kind::Asset => cx.assets.get(&bits),
        _ => None,
    };
    if let Some(id) = named {
        return id.clone();
    }
    let what = match kind {
        Kind::Sound => "sound",
        Kind::Geometry => "geometry",
        Kind::Material => "material",
        _ => "asset",
    };
    format!("{what} {}", bits & 0xffff_ffff)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn field_row(
    ui: &mut Ui,
    s: &mut State,
    cx: &Ctx,
    target: &str,
    f: &FieldSchema,
    editor: Editor,
    shown: &[String],
    current: &str,
    _entity: Option<Entity>,
) {
    let t = *ui.theme();
    let p = t.palette;
    let dirty = s.ins.dirty.contains(target);
    let flashing = s.ins.flash.contains_key(target);
    let name = f.label.clone().unwrap_or_else(|| f.name.clone());
    let name = match &f.unit {
        Some(u) if !u.is_empty() => format!("{name} ({u})"),
        _ if editor == Editor::Euler => format!("{name} (°)"),
        _ => name,
    };
    ui.row(
        format!("F:{target}"),
        Element {
            direction: Direction::Row,
            width: Size::grow(),
            cross: Cross::Center,
            gap: t.space.xs,
            ..Element::default()
        },
        |ui| {
            // Unsaved: a dot before the name.
            ui.open(
                "Mark",
                Element {
                    width: Size::Fixed(6.0),
                    height: Size::Fixed(6.0),
                    radius: 3.0,
                    background: dirty.then_some(p.ink),
                    ..Element::default()
                },
            );
            ui.close();
            let ink = if flashing {
                p.warn
            } else if s.overridden.contains(target) {
                p.info
            } else {
                p.ink_muted
            };
            label(ui, "Name", &name, ink, true, Size::Fixed(NAME));
            editors(ui, s, target, f, editor, shown, current);
        },
    );
    if flashing {
        ui.row(
            format!("W:{target}"),
            Element {
                direction: Direction::Row,
                width: Size::grow(),
                padding: Insets {
                    left: 6.0 + NAME + 2.0 * t.space.xs,
                    ..Insets::default()
                },
                ..Element::default()
            },
            |ui| {
                label(
                    ui,
                    "Changed",
                    "changed by the game: a system writes it every tick",
                    p.warn,
                    true,
                    Size::grow(),
                );
            },
        );
    }
    let _ = cx;
}

pub(super) fn tune(s: &mut State, target: &str, before: &str, after: String, merge: bool) {
    s.ins.requests.push(Request::Tune {
        edit: Edit {
            target: target.to_string(),
            before: before.to_string(),
            after,
        },
        merge,
    });
}

/// A text box for part `part` of a field: typed text is kept while the
/// box has the keyboard and written on Enter or when it is left; Escape
/// drops it.
#[allow(clippy::too_many_arguments)]
pub(super) fn text_part(
    ui: &mut Ui,
    s: &mut State,
    target: &str,
    editor: Editor,
    shown: &[String],
    current: &str,
    part: usize,
    width: Size,
    placeholder: &str,
) {
    let key = format!("{target}#{part}");
    let drafting = s.ins.draft.as_ref().is_some_and(|(k, _)| *k == key);
    let was = shown.get(part).cloned().unwrap_or_default();
    let mut text = match (&s.ins.draft, drafting) {
        (Some((_, d)), true) => d.clone(),
        _ => was.clone(),
    };
    let r = ui.text_field(
        format!("P{part}"),
        &mut text,
        &FieldDesc {
            width: Some(width),
            placeholder: placeholder.to_string(),
            ..FieldDesc::default()
        },
    );
    if r.focused && !r.submitted && !s.escape {
        s.editing = true;
        s.ins.draft = Some((key, text.clone()));
        return;
    }
    let commit = (r.submitted || drafting) && !s.escape;
    if drafting {
        s.ins.draft = None;
    }
    if commit && text.trim() != was {
        match model::command(editor, shown, part, &text) {
            Ok(after) => tune(s, target, current, after, false),
            Err(e) => s.ins.status = format!("{target}: {e}"),
        }
    }
}

pub(super) fn editors(
    ui: &mut Ui,
    s: &mut State,
    target: &str,
    f: &FieldSchema,
    editor: Editor,
    shown: &[String],
    current: &str,
) {
    let t = *ui.theme();
    let p = t.palette;
    let doc = (!f.doc.is_empty()).then(|| f.doc.clone());
    match editor {
        Editor::Number { int } => {
            if let (Some(min), Some(max)) = (f.min, f.max)
                && max > min
                && (max - min) <= 1.0e6
            {
                let r = ui.value_slider(
                    "Slide",
                    shown[0].parse::<f32>().unwrap_or(min as f32),
                    &ValueSliderDesc {
                        min: min as f32,
                        max: max as f32,
                        step: if int { 1.0 } else { 0.0 },
                        label: f.name.clone(),
                        // The value is the text part beside it.
                        show_value: false,
                        bubble: false,
                        width: Some(Size::grow()),
                        ..ValueSliderDesc::default()
                    },
                );
                let v = r.value;
                if let Some(d) = &doc {
                    ui.tooltip(d);
                }
                if r.changed {
                    let after = model::number_text(if int {
                        f64::from(v).round()
                    } else {
                        f64::from(v)
                    });
                    tune(s, target, current, after, true);
                }
                text_part(
                    ui,
                    s,
                    target,
                    editor,
                    shown,
                    current,
                    0,
                    Size::Fixed(72.0),
                    "",
                );
            } else {
                text_part(ui, s, target, editor, shown, current, 0, Size::grow(), "");
                if let Some(d) = &doc {
                    ui.tooltip(d);
                }
            }
        }
        Editor::Vector(n) => {
            for i in 0..n.min(shown.len()) {
                text_part(
                    ui,
                    s,
                    target,
                    editor,
                    shown,
                    current,
                    i,
                    Size::grow(),
                    ["x", "y", "z", "w"][i],
                );
            }
            if let Some(d) = &doc {
                ui.tooltip(d);
            }
        }
        Editor::Euler => {
            for i in 0..3 {
                text_part(
                    ui,
                    s,
                    target,
                    editor,
                    shown,
                    current,
                    i,
                    Size::grow(),
                    ["x", "y", "z"][i],
                );
            }
            if let Some(d) = &doc {
                ui.tooltip(d);
            }
        }
        Editor::Bool => {
            let r = ui.check_field(
                "Check",
                shown[0] == "true",
                &CheckFieldDesc {
                    label: f.name.clone(),
                    ..CheckFieldDesc::default()
                },
            );
            let on = r.on;
            if let Some(d) = &doc {
                ui.tooltip(d);
            }
            if r.changed {
                tune(s, target, current, on.to_string(), false);
            }
        }
        Editor::Enum => {
            let at = f.variants.iter().position(|n| *n == shown[0]).unwrap_or(0);
            let r = ui.dropdown(
                "Pick",
                &DropdownDesc {
                    options: f
                        .variants
                        .iter()
                        .map(|n| DropdownOption {
                            label: n.clone(),
                            ..DropdownOption::default()
                        })
                        .collect(),
                    selected: at as i32,
                    width: Some(Size::grow()),
                    placeholder: f.name.clone(),
                    ..DropdownDesc::default()
                },
            );
            if r.changed
                && let Some(n) = usize::try_from(r.selected)
                    .ok()
                    .and_then(|i| f.variants.get(i))
            {
                tune(s, target, current, n.to_string(), false);
            }
        }
        Editor::Color => {
            let hex = shown[0].trim_start_matches('#');
            let byte = |i: usize| {
                hex.get(i..i + 2)
                    .and_then(|h| u8::from_str_radix(h, 16).ok())
                    .unwrap_or(0)
            };
            ui.open(
                "Swatch",
                Element {
                    width: Size::Fixed(20.0),
                    height: Size::Fixed(20.0),
                    radius: t.radius.small,
                    background: Some(Srgba::rgb(byte(0), byte(2), byte(4))),
                    ..Element::default()
                },
            );
            ui.close();
            text_part(
                ui,
                s,
                target,
                editor,
                shown,
                current,
                0,
                Size::grow(),
                "#rrggbb",
            );
        }
        Editor::Text => {
            text_part(ui, s, target, editor, shown, current, 0, Size::grow(), "");
            if let Some(d) = &doc {
                ui.tooltip(d);
            }
        }
        Editor::Entity => {
            if shown[0] == "none" {
                label(ui, "None", "none", p.ink_faint, true, Size::grow());
            } else {
                let r = ui.button(
                    "Link",
                    &shown[0],
                    ButtonDesc {
                        width: Some(Size::grow()),
                        ..small(ButtonVariant::Quiet)
                    },
                );
                ui.tooltip("Select this entity");
                if r.clicked {
                    s.ins.pending_select = Some(shown[0].clone());
                }
            }
        }
        Editor::ReadOnly => {
            label(ui, "Value", &shown[0], p.ink_faint, true, Size::grow());
        }
    }
}
