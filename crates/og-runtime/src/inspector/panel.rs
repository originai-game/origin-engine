//! The inspector's panel, built with og-ui each frame it is open: a
//! toolbar, the Entities and Resources tabs, the hierarchy, and the
//! selected entity's components with an editor per field.

use super::model::{self, Editor};
use super::{Edit, Inspector, Request};
use crate::codec::Layout as HostLayout;
use og_core::reflect::{FieldSchema, Layout, Schema};
use og_ecs::{Entity, World};
use og_probe::Names;
use og_ui::{
    Align, ButtonDesc, ButtonVariant, ControlSize, Cross, Direction, Element, FieldDesc,
    InputFrame, Insets, LabelDesc, ListDesc, PanelDesc, ScrollDesc, SelectDesc, Size, SliderDesc,
    Srgba, TabsDesc, Ui, UiKey, Viewport,
};
use std::collections::BTreeMap;

/// What the panel reads of the engine this frame.
pub(super) struct Ctx<'a> {
    pub world: &'a World,
    pub names: &'a Names,
    pub layouts: &'a BTreeMap<String, HostLayout>,
    pub paused: bool,
    pub tick: u64,
    pub viewport: Viewport,
    /// What a handle field names: its sound's or asset's id, by handle.
    pub sounds: &'a BTreeMap<u64, String>,
    pub assets: &'a BTreeMap<u64, String>,
}

/// The panel's width in CSS pixels at the og-ui unit.
const WIDTH: f32 = 368.0;
/// A hierarchy row's height.
const ROW: f32 = 26.0;
/// A field name's column.
const NAME: f32 = 112.0;
/// The detail's right margin: its scrollbar draws over the content's edge.
const GUTTER: Insets = Insets {
    left: 0.0,
    top: 0.0,
    right: 10.0,
    bottom: 0.0,
};

/// The column kind a host layout gives a field, if the host declared it.
fn kind_of(
    layouts: &BTreeMap<String, HostLayout>,
    schema: &str,
    field: &str,
) -> Option<crate::codec::Kind> {
    layouts
        .get(schema)
        .and_then(|l| l.field(field))
        .map(|(_, f)| f.kind)
}

/// One field's bytes and schema: of an entity's component, or of a
/// resource when `entity` is none.
fn field_bytes(
    world: &World,
    entity: Option<Entity>,
    component: &str,
    field: &str,
) -> Option<(FieldSchema, Vec<u8>)> {
    let id = world.id_by_name(component).ok()?;
    let schema = world.schema(id).ok()?;
    let i = schema.field_index(field)?;
    let bytes = match entity {
        Some(e) => world.get_bytes(e, id).ok()?,
        None => world.resource_bytes(id).ok()?,
    };
    let range = Layout::of(schema).range(i);
    Some((schema.fields[i].clone(), bytes.get(range)?.to_vec()))
}

/// A field's shown parts now (what a watch compares).
pub(super) fn read_parts(
    world: &World,
    layouts: &BTreeMap<String, HostLayout>,
    entity: Option<Entity>,
    component: &str,
    field: &str,
) -> Option<Vec<String>> {
    let (f, bytes) = field_bytes(world, entity, component, field)?;
    let editor = model::editor(f.ty, kind_of(layouts, component, field));
    Some(model::parts(editor, &f, &bytes, &|e| Names.id_of(world, e)))
}

fn label(ui: &mut Ui, id: &str, text: &str, color: Srgba, small: bool, width: Size) {
    let t = *ui.theme();
    ui.label(
        id,
        text,
        LabelDesc {
            step: Some(if small {
                t.type_ramp.xs
            } else {
                t.type_ramp.sm
            }),
            color: Some(color),
            nowrap: true,
            ellipsis: true,
            width,
            ..LabelDesc::default()
        },
    );
}

fn small(variant: ButtonVariant) -> ButtonDesc {
    ButtonDesc {
        variant,
        size: ControlSize::Small,
        ..ButtonDesc::default()
    }
}

/// Builds the panel and lays it out; what it asks for is left in
/// `ins.requests`.
pub(super) fn build(ins: &mut Inspector, cx: &Ctx) {
    let input = InputFrame::read(&ins.input).with_text(&ins.typed);
    ins.typed.clear();
    let shortcut = input.ctrl || input.meta;
    let escape = input.keys.contains(&UiKey::Escape);
    if shortcut && !ins.editing && ins.over_panel(ins.input.pointer.x, ins.input.pointer.y) {
        if input.keys.contains(&UiKey::KeyZ) {
            ins.requests.push(if input.shift {
                Request::Redo
            } else {
                Request::Undo
            });
        } else if input.keys.contains(&UiKey::KeyY) {
            ins.requests.push(Request::Redo);
        }
    }
    let Some(mut ui) = ins.ui.take() else {
        return;
    };
    ui.begin_with(
        input,
        cx.viewport,
        Element {
            direction: Direction::Row,
            align: Align::End,
            padding: Insets::all(12.0),
            ..Element::default()
        },
    );
    let t = *ui.theme();
    let mut s = State {
        ins,
        editing: false,
        escape,
    };
    ui.open_panel(
        "Inspector",
        PanelDesc {
            width: Size::Fixed(WIDTH),
            height: Size::grow(),
            padding: Some(Insets::all(t.space.md)),
            gap: Some(t.space.sm),
            ..PanelDesc::default()
        },
    );
    toolbar(&mut ui, &mut s, cx);
    let mut tab = s.ins.tab;
    ui.tabs(
        "View",
        &["Entities", "Resources"],
        &mut tab,
        TabsDesc {
            fill: true,
            label: "Inspector view".into(),
            ..TabsDesc::default()
        },
    );
    s.ins.tab = tab;
    let mut filter = s.ins.filter.clone();
    let r = ui.text_field(
        "Filter",
        &mut filter,
        &FieldDesc {
            placeholder: if tab == 0 {
                "Filter entities by name".into()
            } else {
                "Filter resources by name".into()
            },
            ..FieldDesc::default()
        },
    );
    s.editing |= r.focused;
    s.ins.filter = filter;
    if tab == 0 {
        entities(&mut ui, &mut s, cx);
    } else {
        resources(&mut ui, &mut s, cx);
    }
    footer(&mut ui, &mut s, cx);
    ui.close();
    ui.end();
    let frame = ui.frame();
    s.ins.panel = frame.element("Inspector").map(|e| {
        let u = frame.unit;
        [e.x * u, e.y * u, e.width * u, e.height * u]
    });
    let editing = s.editing;
    s.ins.editing = editing;
    if !editing {
        s.ins.draft = None;
    }
    s.ins.input.end_tick();
    s.ins.ui = Some(ui);
}

/// The panel's state while it is built.
struct State<'a> {
    ins: &'a mut Inspector,
    /// A field has the keyboard this frame.
    editing: bool,
    /// Escape this frame: a field being typed in is left unchanged.
    escape: bool,
}

fn toolbar(ui: &mut Ui, s: &mut State, cx: &Ctx) {
    let t = *ui.theme();
    let p = t.palette;
    let dirty = s.ins.dirty.len();
    let row = |gap| Element {
        direction: Direction::Row,
        width: Size::grow(),
        cross: Cross::Center,
        gap,
        ..Element::default()
    };
    ui.row("Bar", row(t.space.xs), |ui| {
        ui.label(
            "Title",
            "Inspector",
            LabelDesc {
                step: Some(t.type_ramp.base),
                color: Some(p.ink),
                heading: true,
                nowrap: true,
                ..LabelDesc::default()
            },
        );
        let key = s.ins.key_label();
        if ui.button("Key", &key, small(ButtonVariant::Quiet)).clicked {
            s.ins.requests.push(Request::Close);
        }
        ui.tooltip(&if s.ins.needs_ctrl {
            format!(
                "{key} shows and hides the inspector (the game binds {}, so it takes Ctrl with it)",
                s.ins.key
            )
        } else {
            format!("{key} shows and hides the inspector")
        });
        ui.spacer("Gap", None);
        let save = if dirty > 0 {
            format!("Save ({dirty})")
        } else {
            "Save".to_string()
        };
        let r = ui.button(
            "Save",
            &save,
            ButtonDesc {
                disabled: dirty == 0,
                ..small(if dirty > 0 {
                    ButtonVariant::Primary
                } else {
                    ButtonVariant::Secondary
                })
            },
        );
        if r.clicked {
            s.ins.requests.push(Request::Save);
        }
        ui.tooltip("Write the edited fields into the scenes' text (save_scene)");
        if ui
            .button("Close", "Close", small(ButtonVariant::Quiet))
            .clicked
        {
            s.ins.requests.push(Request::Close);
        }
    });
    ui.row("Tools", row(t.space.xs), |ui| {
        let (text, req) = if cx.paused {
            ("Play", Request::Resume)
        } else {
            ("Pause", Request::Pause)
        };
        if ui
            .button("Clock", text, small(ButtonVariant::Secondary))
            .clicked
        {
            s.ins.requests.push(req);
        }
        let step = ui.button(
            "Step",
            "Step",
            ButtonDesc {
                disabled: !cx.paused,
                ..small(ButtonVariant::Secondary)
            },
        );
        if step.clicked {
            s.ins.requests.push(Request::Step);
        }
        ui.tooltip("Advance one tick while paused");
        let pick = ui.button(
            "Pick",
            if s.ins.picking { "Picking…" } else { "Pick" },
            small(if s.ins.picking {
                ButtonVariant::Primary
            } else {
                ButtonVariant::Secondary
            }),
        );
        if pick.clicked {
            s.ins.picking = !s.ins.picking;
        }
        ui.tooltip("Select what is under the next click in the game");
        ui.spacer("Gap", None);
        let undo = ui.button(
            "Undo",
            "Undo",
            ButtonDesc {
                disabled: !s.ins.history.can_undo(),
                ..small(ButtonVariant::Secondary)
            },
        );
        if undo.clicked {
            s.ins.requests.push(Request::Undo);
        }
        ui.tooltip("Undo the last edit (Ctrl+Z)");
        let redo = ui.button(
            "Redo",
            "Redo",
            ButtonDesc {
                disabled: !s.ins.history.can_redo(),
                ..small(ButtonVariant::Secondary)
            },
        );
        if redo.clicked {
            s.ins.requests.push(Request::Redo);
        }
        ui.tooltip("Redo (Ctrl+Shift+Z)");
    });
}

fn footer(ui: &mut Ui, s: &mut State, cx: &Ctx) {
    let t = *ui.theme();
    let p = t.palette;
    if s.ins.tab == 0 {
        gizmo_bar(ui, s);
    }
    ui.row(
        "Status",
        Element {
            direction: Direction::Row,
            width: Size::grow(),
            cross: Cross::Center,
            gap: t.space.sm,
            ..Element::default()
        },
        |ui| {
            let clock = if cx.paused {
                format!("Tick {} · paused", cx.tick)
            } else {
                format!("Tick {}", cx.tick)
            };
            label(ui, "Clock", &clock, p.ink_faint, true, Size::fit());
            let text = if s.ins.status.is_empty() {
                String::new()
            } else {
                s.ins.status.clone()
            };
            label(ui, "Note", &text, p.ink_muted, true, Size::grow());
        },
    );
}

/// The gizmo's mode (W, E, R) and space.
fn gizmo_bar(ui: &mut Ui, s: &mut State) {
    use super::gizmo::{Mode, Space};
    let t = *ui.theme();
    ui.row(
        "Gizmo",
        Element {
            direction: Direction::Row,
            width: Size::grow(),
            cross: Cross::Center,
            gap: t.space.xs,
            ..Element::default()
        },
        |ui| {
            for (id, text, mode, tip) in [
                ("Move", "Move", Mode::Translate, "Move along an axis or a plane (W); Ctrl snaps to 0.5"),
                ("Rotate", "Rotate", Mode::Rotate, "Turn about an axis or the view (E); Ctrl snaps to 15°"),
                ("Scale", "Scale", Mode::Scale, "Scale along the entity's own axes, or all from the centre (R); Ctrl snaps to 0.1"),
            ] {
                let on = s.ins.tool.mode == mode;
                let r = ui.button(
                    id,
                    text,
                    small(if on {
                        ButtonVariant::Primary
                    } else {
                        ButtonVariant::Secondary
                    }),
                );
                if r.clicked {
                    s.ins.tool.mode = mode;
                }
                ui.tooltip(tip);
            }
            ui.spacer("Gap", None);
            let (text, next) = match s.ins.tool.space {
                Space::World => ("World", Space::Local),
                Space::Local => ("Local", Space::World),
            };
            if ui
                .button("Space", text, small(ButtonVariant::Quiet))
                .clicked
            {
                s.ins.tool.space = next;
            }
            ui.tooltip("Move and rotate along the world's axes or the entity's own");
        },
    );
}

fn entities(ui: &mut Ui, s: &mut State, cx: &Ctx) {
    let t = *ui.theme();
    let p = t.palette;
    let world = cx.world;
    let rows = model::rows(
        &world.entities(),
        &|e| model::parent_of(world, e),
        &|e| (cx.names.id_of(world, e), world.name(e).is_some()),
        &s.ins.collapsed,
        &s.ins.filter,
    );
    let range = ui.open_list(
        "Tree",
        ListDesc {
            count: rows.len() as u32,
            row_height: ROW,
            width: Size::grow(),
            height: Some(Size::Fixed(232.0)),
            ..ListDesc::default()
        },
    );
    for i in range.first..range.last {
        let row = &rows[i as usize];
        let selected = s.ins.selected == Some(row.entity);
        let r = ui.list_item(i, selected);
        ui.spacer("Indent", Some(Size::Fixed(row.depth as f32 * 14.0)));
        if row.children > 0 {
            let open = !s.ins.collapsed.contains(&row.entity.to_bits());
            if chevron(ui, "Fold", open, 20.0) {
                let bits = row.entity.to_bits();
                if !s.ins.collapsed.remove(&bits) {
                    s.ins.collapsed.insert(bits);
                }
            }
        } else {
            ui.spacer("Leaf", Some(Size::Fixed(20.0)));
        }
        let ink = if row.named { p.ink } else { p.ink_muted };
        label(ui, "Name", &row.label, ink, false, Size::grow());
        if row.children > 0 {
            label(
                ui,
                "Count",
                &row.children.to_string(),
                p.ink_faint,
                true,
                Size::fit(),
            );
        }
        ui.close();
        if r.clicked {
            s.ins.selected = Some(row.entity);
        }
    }
    ui.close_list();
    if rows.is_empty() {
        let none = if s.ins.filter.trim().is_empty() {
            "No entities"
        } else {
            "No entity matches the filter"
        };
        label(ui, "Empty", none, p.ink_muted, true, Size::grow());
    }
    ui.divider("Split", false);
    ui.scroll(
        "Detail",
        ScrollDesc {
            width: Size::grow(),
            height: Size::grow(),
            gap: t.space.xs,
            padding: GUTTER,
            ..ScrollDesc::default()
        },
        |ui| match s.ins.selected {
            Some(e) => entity(ui, s, cx, e),
            None => label(
                ui,
                "Hint",
                "Select an entity in the list, or Pick one in the game",
                p.ink_muted,
                false,
                Size::grow(),
            ),
        },
    );
}

/// The components of an entity, og.Transform first, then by name.
fn components(world: &World, e: Entity) -> Vec<Schema> {
    let mut out: Vec<Schema> = world
        .components_of(e)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|id| world.schema(id).ok().cloned())
        .collect();
    out.sort_by(|a, b| {
        (a.name != "og.Transform", a.name.as_str())
            .cmp(&(b.name != "og.Transform", b.name.as_str()))
    });
    out
}

fn entity(ui: &mut Ui, s: &mut State, cx: &Ctx, e: Entity) {
    let t = *ui.theme();
    let p = t.palette;
    let world = cx.world;
    let id = cx.names.id_of(world, e);
    ui.row(
        "Head",
        Element {
            direction: Direction::Row,
            width: Size::grow(),
            cross: Cross::Center,
            gap: t.space.sm,
            padding: Insets::xy(0.0, t.space.xxs),
            ..Element::default()
        },
        |ui| {
            label(ui, "Id", &id, p.ink, false, Size::grow());
            if world.name(e).is_some() {
                let raw = format!("#{}.{}", e.index(), e.generation());
                label(ui, "Raw", &raw, p.ink_faint, true, Size::fit());
            }
        },
    );
    for schema in components(world, e) {
        let key = format!("C:{}", schema.name);
        let open = !s.ins.folded.contains(&key);
        if section(ui, &schema.name, open) && !s.ins.folded.remove(&key) {
            s.ins.folded.insert(key);
        }
        if open {
            let bytes = world
                .id_by_name(&schema.name)
                .ok()
                .and_then(|id| world.get_bytes(e, id).ok())
                .unwrap_or_default();
            fields(
                ui,
                s,
                cx,
                &format!("{id}.{}", schema.name),
                Some(e),
                &schema,
                &bytes,
            );
        }
    }
}

fn resources(ui: &mut Ui, s: &mut State, cx: &Ctx) {
    let t = *ui.theme();
    let p = t.palette;
    let world = cx.world;
    let filter = s.ins.filter.trim().to_lowercase();
    let mut list: Vec<Schema> = world
        .schemas()
        .into_iter()
        .filter(|sc| {
            world
                .id_by_name(&sc.name)
                .is_ok_and(|id| world.has_resource_id(id))
        })
        .filter(|sc| filter.is_empty() || sc.name.to_lowercase().contains(&filter))
        .collect();
    list.sort_by(|a, b| a.name.cmp(&b.name));
    ui.scroll(
        "Detail",
        ScrollDesc {
            width: Size::grow(),
            height: Size::grow(),
            gap: t.space.xs,
            padding: GUTTER,
            ..ScrollDesc::default()
        },
        |ui| {
            if list.is_empty() {
                label(
                    ui,
                    "Empty",
                    "No resource matches the filter",
                    p.ink_muted,
                    true,
                    Size::grow(),
                );
            }
            for schema in &list {
                // Resources start folded: there are many, and a game's own
                // are what a person looks for.
                let key = format!("R:{}", schema.name);
                let open = s.ins.folded.contains(&key);
                if section(ui, &schema.name, open) && !s.ins.folded.remove(&key) {
                    s.ins.folded.insert(key);
                }
                if open {
                    let bytes = world
                        .id_by_name(&schema.name)
                        .ok()
                        .and_then(|id| world.resource_bytes(id).ok())
                        .unwrap_or_default();
                    fields(ui, s, cx, &schema.name, None, schema, &bytes);
                }
            }
        },
    );
}

/// The fold's mark: a chevron drawn as a path (pointing down when open,
/// toward the name when closed), so it needs nothing of the game's icons.
fn glyph(ui: &mut Ui, open: bool) {
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
fn chevron(ui: &mut Ui, id: &str, open: bool, size: f32) -> bool {
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
fn section(ui: &mut Ui, name: &str, open: bool) -> bool {
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
fn fields(
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
fn handle_name(cx: &Ctx, kind: crate::codec::Kind, bits: u64) -> String {
    use crate::codec::Kind;
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
fn field_row(
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
            let ink = if flashing { p.warn } else { p.ink_muted };
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

fn tune(s: &mut State, target: &str, before: &str, after: String, merge: bool) {
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
fn text_part(
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

fn editors(
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
                let mut v = shown[0].parse::<f32>().unwrap_or(min as f32);
                let r = ui.slider(
                    "Slide",
                    &mut v,
                    SliderDesc {
                        min: min as f32,
                        max: max as f32,
                        step: if int { 1.0 } else { 0.0 },
                        width: Some(Size::grow()),
                    },
                );
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
            let mut on = shown[0] == "true";
            let r = ui.checkbox("Check", &mut on, "");
            if let Some(d) = &doc {
                ui.tooltip(d);
            }
            if r.changed {
                tune(s, target, current, on.to_string(), false);
            }
        }
        Editor::Enum => {
            let names: Vec<&str> = f.variants.iter().map(String::as_str).collect();
            let mut i = names.iter().position(|n| *n == shown[0]).unwrap_or(0);
            let r = ui.select(
                "Pick",
                &names,
                &mut i,
                &SelectDesc {
                    width: Some(Size::grow()),
                    label: f.name.clone(),
                    ..SelectDesc::default()
                },
            );
            if r.changed
                && let Some(n) = names.get(i)
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
