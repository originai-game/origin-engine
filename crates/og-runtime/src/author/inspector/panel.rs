//! The inspector's panel: the built-in document `panel.svelte`, mounted
//! in the inspector's own UI and given the inspector's state and the
//! world as its values each frame it is open ([`values`]). Its events are
//! the inspector's requests ([`events`]).

use super::model::{self, Editor};
use super::{Edit, Inspector, Request};
use crate::host::codec::Layout as HostLayout;
use og_core::reflect::{FieldSchema, Layout};
use og_ecs::{Entity, World};
use og_probe::Names;
use og_ui::reactive::SourceId;
use og_ui::{InputFrame, UiDocument, UiKey, Viewport};
use std::collections::BTreeMap;

mod events;
mod values;

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

/// The panel's mount in the inspector's UI.
fn mount() -> SourceId {
    SourceId::new("og.inspector", "panel")
}

/// The inspector's UI, its panel mounted: in the default style at the
/// mouse density.
pub(super) fn ui(ticks_per_second: u32) -> og_ui::Ui {
    let theme = og_ui::Theme {
        density: og_ui::DensityMode::Mouse,
        ..og_ui::Theme::default()
    };
    let mut ui = og_ui::Ui::new(og_ui::UiDesc {
        theme,
        ticks_per_second,
        fonts: og_ui::Fonts::default(),
    });
    let doc = UiDocument::parse("inspector.svelte", include_str!("panel.svelte"))
        .unwrap_or_else(|e| panic!("the inspector's panel: {}", e.join("; ")));
    ui.define_document(doc);
    if let Err(e) = ui.mount_with_values(mount(), "inspector", &serde_json::Value::Null) {
        panic!("the inspector's panel: {e}");
    }
    ui
}

/// The panel element's path: what the pointer is over when it is on it.
const PANEL: &str = "og.inspector:panel/Inspector";

/// An element's path in the panel, as `dump_inspector` gives it (the
/// mount's left out: `Inspector/Tools/Undo`); none for the mount itself.
pub(super) fn local_path(path: &str) -> Option<&str> {
    path.strip_prefix("og.inspector:panel/")
}

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

/// A prefab instance's field as `apply_prefab` names it: `Component.field`
/// with the engine's `og.` dropped (`og.Transform.position` is
/// `Transform.position`, as the scene's text writes it).
pub(crate) fn prefab_field(component_field: &str) -> String {
    component_field
        .strip_prefix("og.")
        .unwrap_or(component_field)
        .to_string()
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
    // Escape leaves the field being typed in as it was.
    if escape {
        ins.draft = None;
    }
    let Some(mut ui) = ins.ui.take() else {
        return;
    };
    let (values, shown) = values::of(ins, cx, ui.frame().unit.max(1.0));
    if let Err(e) = ui.set_mount_values(&mount(), &values) {
        ins.status = format!("inspector: {e}");
    }
    ui.begin(input, cx.viewport);
    if let Err(e) = ui.build_retained_documents(&og_ui::bind::NoEnv) {
        ins.status = format!("inspector: {e}");
    }
    ui.end();
    events::apply(ins, &shown, cx.paused, &ui.take_doc_events());
    let u = ui.frame().unit;
    ins.panel = ui
        .element_bounds(PANEL)
        .map(|e| [e.x * u, e.y * u, e.width * u, e.height * u]);
    // A text box has the keyboard: the filter or a field's part.
    let focus = ui.focus_path(0);
    ins.editing = focus.ends_with("/Filter") || focus.contains("/Part#");
    events::commit_draft(ins, &shown, focus);
    ins.input.end_tick();
    ins.ui = Some(ui);
}
