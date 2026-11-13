//! The inspector's panel, built with og-ui each frame it is open: a
//! toolbar, the Entities and Resources tabs, the hierarchy, and the
//! selected entity's components with an editor per field.

use super::model::{self, Editor};
use super::{Edit, Inspector, Request};
use crate::host::codec::Layout as HostLayout;
use og_core::reflect::{FieldSchema, Layout, Schema};
use og_ecs::{Entity, World};
use og_probe::Names;
use og_ui::{
    Align, ButtonDesc, ButtonVariant, CheckFieldDesc, ControlSize, Cross, Direction, DropdownDesc,
    DropdownOption, Element, FieldDesc, InputFrame, Insets, LabelDesc, ListDesc, PanelDesc,
    ScrollDesc, SegmentItem, SegmentedDesc, Size, Srgba, Ui, UiKey, ValueSliderDesc, Viewport,
};
use std::collections::BTreeMap;

mod bars;
mod fields;
mod lists;

use bars::*;
use fields::*;
pub(crate) use lists::*;

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
    /// The selected entity's prefab, when it was placed from one
    /// (`Probe::entity_prefab`).
    pub prefab: Option<&'a serde_json::Value>,
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
) -> Option<crate::host::codec::Kind> {
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
        overridden: Default::default(),
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
    let tab = ui
        .segmented(
            "View",
            &SegmentedDesc {
                items: ["Entities", "Resources", "UI"]
                    .map(|l| SegmentItem {
                        label: l.into(),
                        ..SegmentItem::default()
                    })
                    .to_vec(),
                selected: s.ins.tab as i32,
                size: ControlSize::Small,
                width: Some(Size::grow()),
                label: "Inspector view".into(),
                ..SegmentedDesc::default()
            },
        )
        .selected
        .max(0) as usize;
    s.ins.tab = tab;
    let mut filter = s.ins.filter.clone();
    let r = ui.text_field(
        "Filter",
        &mut filter,
        &FieldDesc {
            placeholder: match tab {
                0 => "Filter entities by name".into(),
                super::ui_debug::TAB => "Filter styles and bindings by name".into(),
                _ => "Filter resources by name".into(),
            },
            ..FieldDesc::default()
        },
    );
    s.editing |= r.focused;
    s.ins.filter = filter;
    let view = s.ins.ui_view.clone();
    match tab {
        0 => entities(&mut ui, &mut s, cx),
        super::ui_debug::TAB => {
            let filter = s.ins.filter.trim().to_lowercase();
            super::ui_debug::detail(&mut ui, &view, &filter, &mut s.ins.ui_bounds)
        }
        _ => resources(&mut ui, &mut s, cx),
    }
    footer(&mut ui, &mut s, cx);
    ui.close();
    if tab == super::ui_debug::TAB {
        let unit = ui.frame().unit;
        og_ui::debug_overlay(
            &mut ui,
            view.element.as_ref(),
            view.hover.as_ref(),
            &view.bounds,
            unit,
        );
    }
    ui.end();
    let u = ui.frame().unit;
    s.ins.panel = ui
        .element_bounds("Inspector")
        .map(|e| [e.x * u, e.y * u, e.width * u, e.height * u]);
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
    /// The selected prefab instance's overridden fields, by `set` target.
    overridden: std::collections::BTreeSet<String>,
}
