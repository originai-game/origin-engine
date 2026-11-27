//! The inspector's panel: the engine's TSX component
//! (`packages/engine/src/ui/inspector.tsx`) that the runtime mounts on the
//! game's UI while the inspector is open. Each frame gives it the
//! inspector's state and the world as its values ([`values`]); what its
//! controls did comes back as [`Event`]s, the inspector's requests and
//! state ([`events`]).

use super::model::{self, Editor};
use super::{Edit, Inspector, Request};
use crate::host::codec::Layout as HostLayout;
use og_core::reflect::{FieldSchema, Layout};
use og_ecs::{Entity, World};
use og_probe::Names;
use std::collections::BTreeMap;

mod events;
mod values;

pub(super) use events::Event;
pub(super) use values::Shown;

/// What the panel reads of the engine this frame.
pub(super) struct Ctx<'a> {
    pub world: &'a World,
    pub names: &'a Names,
    pub layouts: &'a BTreeMap<String, HostLayout>,
    pub paused: bool,
    pub tick: u64,
    /// What a handle field names: its sound's or asset's id, by handle.
    pub sounds: &'a BTreeMap<u64, String>,
    pub assets: &'a BTreeMap<u64, String>,
    /// The selected entity's prefab, when it was placed from one
    /// (`Probe::entity_prefab`).
    pub prefab: Option<&'a serde_json::Value>,
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

/// A prefab instance's field as `revert_prefab` names it: `Component.field`
/// with the engine's `og.` dropped (`og.Transform.position` is
/// `Transform.position`).
pub(crate) fn prefab_field(component_field: &str) -> String {
    component_field
        .strip_prefix("og.")
        .unwrap_or(component_field)
        .to_string()
}

/// Does what the panel's controls did since the last frame, against the
/// rows it showed, then makes its values for this frame; what it asks for
/// is left in `ins.requests`.
pub(super) fn build(ins: &mut Inspector, cx: &Ctx) {
    let shown = std::mem::take(&mut ins.shown);
    for e in std::mem::take(&mut ins.events) {
        events::apply(ins, &shown, cx.paused, &e);
    }
    let (values, shown) = values::of(ins, cx);
    ins.shown = shown;
    let text = values.to_string();
    if text != ins.last {
        ins.values = Some(text);
    }
}
