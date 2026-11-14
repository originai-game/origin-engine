//! What the panel shows this frame, as its document's values: the
//! inspector's state and what it reads of the world.

use super::super::{gizmo, ui_debug};
use super::*;
use og_core::reflect::Schema;
use serde_json::{Map, Value, json};
use std::collections::BTreeSet;

/// The views, as the panel's tabs name them (`Inspector::tab` indexes it).
pub(super) const TABS: [&str; 3] = ["entities", "resources", "ui"];

/// What the values show where, for the events they answer.
#[derive(Default)]
pub(in super::super) struct Shown {
    /// The hierarchy's entities, by row key.
    pub entities: BTreeMap<String, Entity>,
    /// The fields shown, by `set` target.
    pub fields: BTreeMap<String, Field>,
    /// The selected prefab instance and the fields it overrides.
    pub prefab: Option<(Entity, Vec<String>)>,
}

/// A field shown: how it is edited, its parts as shown, and its value as
/// `set` writes it.
pub(in super::super) struct Field {
    pub editor: Editor,
    pub shown: Vec<String>,
    pub current: String,
    pub variants: Vec<String>,
}

/// The key of part `part` of the field `target`: `<target>#<part>`.
pub(super) fn part_key(target: &str, part: usize) -> String {
    format!("{target}#{part}")
}

/// The panel's values this frame; `unit` is the panel's UI unit (the
/// UI view's boxes are in device px).
pub(super) fn of(ins: &Inspector, cx: &Ctx, unit: f32) -> (Value, Shown) {
    let mut out = Out::default();
    let hint = match ins.tab {
        0 => {
            entities(ins, cx, &mut out);
            "Filter entities by name"
        }
        ui_debug::TAB => {
            let filter = ins.filter.trim().to_lowercase();
            match ui_debug::lines(&ins.ui_view, &filter) {
                Some(lines) => out.lines = lines,
                None => out.note = "Pick an element of the game's UI".into(),
            }
            out.boxes = ui_debug::boxes(&ins.ui_view, unit);
            "Filter styles and bindings by name"
        }
        _ => {
            resources(ins, cx, &mut out);
            "Filter resources by name"
        }
    };
    let key = ins.key_label();
    let key_tip = if ins.needs_ctrl {
        format!(
            "{key} shows and hides the inspector (the game binds {}, so it takes Ctrl with it)",
            ins.key
        )
    } else {
        format!("{key} shows and hides the inspector")
    };
    let clock = if cx.paused {
        format!("Tick {} · paused", cx.tick)
    } else {
        format!("Tick {}", cx.tick)
    };
    let space = match ins.tool.space {
        gizmo::Space::World => "World",
        gizmo::Space::Local => "Local",
    };
    let values = json!({
        "key": key,
        "keyTip": key_tip,
        "dirty": ins.dirty.len(),
        "paused": cx.paused,
        "picking": ins.picking,
        "canUndo": ins.history.can_undo(),
        "canRedo": ins.history.can_redo(),
        "tab": TABS[ins.tab.min(TABS.len() - 1)],
        "filter": ins.filter,
        "hint": hint,
        "tree": out.tree,
        "note": out.note,
        "head": out.head,
        "prefab": out.prefab,
        "overrides": out.overrides,
        "sections": out.sections,
        "bounds": ins.ui_bounds,
        "lines": out.lines,
        "boxes": out.boxes,
        "mode": ins.tool.mode.name(),
        "space": space,
        "clock": clock,
        "status": ins.status,
    });
    (values, out.shown)
}

/// The values a view fills.
struct Out {
    tree: Vec<Value>,
    note: String,
    head: Value,
    prefab: Value,
    overrides: Vec<Value>,
    sections: Vec<Value>,
    lines: Vec<Value>,
    boxes: Vec<Value>,
    shown: Shown,
}

impl Default for Out {
    fn default() -> Out {
        Out {
            tree: Vec::new(),
            note: String::new(),
            head: json!({ "id": "", "raw": "" }),
            prefab: json!({ "what": "", "none": true }),
            overrides: Vec::new(),
            sections: Vec::new(),
            lines: Vec::new(),
            boxes: Vec::new(),
            shown: Shown::default(),
        }
    }
}

fn entities(ins: &Inspector, cx: &Ctx, out: &mut Out) {
    let world = cx.world;
    let rows = model::rows(
        &world.entities(),
        &|e| model::parent_of(world, e),
        &|e| (cx.names.id_of(world, e), world.name(e).is_some()),
        &ins.collapsed,
        &ins.filter,
    );
    for row in &rows {
        let bits = row.entity.to_bits();
        let fold = if row.children == 0 {
            "leaf"
        } else if ins.collapsed.contains(&bits) {
            "closed"
        } else {
            "open"
        };
        out.tree.push(json!({
            "id": bits.to_string(),
            "label": row.label,
            "depth": row.depth,
            "fold": fold,
            "selected": ins.selected == Some(row.entity),
            "named": row.named,
            "children": row.children,
        }));
        out.shown.entities.insert(bits.to_string(), row.entity);
    }
    let Some(e) = ins.selected else {
        out.note = if !rows.is_empty() {
            "Select an entity in the list, or Pick one in the game"
        } else if ins.filter.trim().is_empty() {
            "No entities"
        } else {
            "No entity matches the filter"
        }
        .into();
        return;
    };
    let id = cx.names.id_of(world, e);
    let raw = match world.name(e) {
        Some(_) => format!("#{}.{}", e.index(), e.generation()),
        None => String::new(),
    };
    out.head = json!({ "id": id, "raw": raw });
    let overridden = match cx.prefab {
        Some(info) => prefab(ins, out, e, &id, info),
        None => BTreeSet::new(),
    };
    for schema in components(world, e) {
        let key = format!("C:{}", schema.name);
        let open = !ins.folded.contains(&key);
        let fields = if open {
            let bytes = world
                .id_by_name(&schema.name)
                .ok()
                .and_then(|c| world.get_bytes(e, c).ok())
                .unwrap_or_default();
            let prefix = format!("{id}.{}", schema.name);
            fields(
                ins,
                cx,
                &mut out.shown,
                &prefix,
                &schema,
                &bytes,
                &overridden,
            )
        } else {
            Vec::new()
        };
        out.sections.push(json!({
            "id": key, "name": schema.name, "open": open, "fields": fields,
        }));
    }
}

/// The prefab an entity was placed from, and what its instance overrides:
/// the scene's override lines and this session's unsaved edits. Answers
/// the `set` targets they show under.
fn prefab(ins: &Inspector, out: &mut Out, e: Entity, id: &str, info: &Value) -> BTreeSet<String> {
    let name = info["prefab"].as_str().unwrap_or("");
    let node = info["node"].as_str().unwrap_or("");
    // The fields it overrides: saved lines, then unsaved edits.
    let mut fields: Vec<(String, bool)> = info["overrides"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|f| (prefab_field(f), true))
        .collect();
    let prefix = format!("{id}.");
    for target in &ins.dirty {
        if let Some(rest) = target.strip_prefix(&prefix) {
            let f = prefab_field(rest);
            if !fields.iter().any(|(g, _)| *g == f) {
                fields.push((f, false));
            }
        }
    }
    let what = if node.is_empty() {
        format!("Prefab {name}")
    } else {
        format!("Prefab {name} · {node}")
    };
    out.prefab = json!({ "what": what, "none": fields.is_empty() });
    let mut overridden = BTreeSet::new();
    for (f, saved) in &fields {
        let text = if *saved {
            f.clone()
        } else {
            format!("{f} (unsaved)")
        };
        out.overrides.push(json!({ "id": f, "text": text }));
        // og.Transform's shows with its og.
        let (c, field) = f.split_once('.').unwrap_or((f, ""));
        for comp in [c.to_string(), format!("og.{c}")] {
            overridden.insert(format!("{id}.{comp}.{field}"));
        }
    }
    out.shown.prefab = Some((e, fields.into_iter().map(|(f, _)| f).collect()));
    overridden
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

fn resources(ins: &Inspector, cx: &Ctx, out: &mut Out) {
    let world = cx.world;
    let filter = ins.filter.trim().to_lowercase();
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
    if list.is_empty() {
        out.note = "No resource matches the filter".into();
    }
    for schema in &list {
        // Resources start folded: there are many, and a game's own are
        // what a person looks for.
        let key = format!("R:{}", schema.name);
        let open = ins.folded.contains(&key);
        let fields = if open {
            let bytes = world
                .id_by_name(&schema.name)
                .ok()
                .and_then(|id| world.resource_bytes(id).ok())
                .unwrap_or_default();
            let none = BTreeSet::new();
            fields(ins, cx, &mut out.shown, &schema.name, schema, &bytes, &none)
        } else {
            Vec::new()
        };
        out.sections.push(json!({
            "id": key, "name": schema.name, "open": open, "fields": fields,
        }));
    }
}

/// The field rows of one record: `prefix` is its `set` target before the
/// field (`<entity>.<Component>` or `<Resource>`).
fn fields(
    ins: &Inspector,
    cx: &Ctx,
    shown_fields: &mut Shown,
    prefix: &str,
    schema: &Schema,
    bytes: &[u8],
    overridden: &BTreeSet<String>,
) -> Vec<Value> {
    let layout = Layout::of(schema);
    let world = cx.world;
    let mut out = Vec::new();
    for (i, f) in schema.fields.iter().enumerate() {
        let Some(slice) = bytes.get(layout.range(i)) else {
            continue;
        };
        let target = format!("{prefix}.{}", f.name);
        let kind = kind_of(cx.layouts, &schema.name, &f.name);
        let editor = model::editor(f.ty, kind);
        let mut shown = model::parts(editor, f, slice, &|e| cx.names.id_of(world, e));
        if editor == Editor::ReadOnly
            && let Some(k) = kind.filter(|k| k.is_handle())
            && let Some(bits) = slice.get(..8).and_then(|b| b.try_into().ok())
        {
            shown = vec![handle_name(cx, k, u64::from_le_bytes(bits))];
        }
        let tone = if ins.flash.contains_key(&target) {
            "flash"
        } else if overridden.contains(&target) {
            "override"
        } else {
            ""
        };
        let mut row = field_row(ins, &target, f, editor, &shown);
        row.insert("tone".into(), json!(tone));
        out.push(Value::Object(row));
        shown_fields.fields.insert(
            target,
            Field {
                editor,
                current: model::current(editor, f, slice),
                shown,
                variants: f.variants.clone(),
            },
        );
    }
    out
}

/// One field's row: its name, its mark and the editor's values.
fn field_row(
    ins: &Inspector,
    target: &str,
    f: &FieldSchema,
    editor: Editor,
    shown: &[String],
) -> Map<String, Value> {
    let name = f.label.clone().unwrap_or_else(|| f.name.clone());
    let name = match &f.unit {
        Some(u) if !u.is_empty() => format!("{name} ({u})"),
        _ if editor == Editor::Euler => format!("{name} (°)"),
        _ => name,
    };
    let first = shown.first().cloned().unwrap_or_default();
    // The text boxes: one per part typed in, each with its draft while it
    // has the keyboard.
    let texts = |n: usize, hints: &[&str]| -> Vec<Value> {
        (0..n.min(shown.len()))
            .map(|k| {
                let key = part_key(target, k);
                let text = match &ins.draft {
                    Some((d, text)) if *d == key => text.clone(),
                    _ => shown[k].clone(),
                };
                json!({ "id": key, "hint": hints.get(k).copied().unwrap_or(""), "text": text })
            })
            .collect()
    };
    let mut row = Map::new();
    let mut set = |k: &str, v: Value| {
        row.insert(k.into(), v);
    };
    set("id", json!(target));
    set("name", json!(name));
    set("mark", json!(ins.dirty.contains(target)));
    set("flash", json!(ins.flash.contains_key(target)));
    set("doc", json!(f.doc));
    for (k, v) in [
        ("editor", json!("text")),
        ("parts", json!([])),
        ("min", json!(0)),
        ("max", json!(1)),
        ("step", json!(0)),
        ("value", json!(0)),
        ("on", json!(false)),
        ("options", json!([])),
        ("choice", json!("")),
        ("text", json!("")),
        ("swatch", json!("")),
    ] {
        set(k, v);
    }
    match editor {
        Editor::Number { int } => {
            if let (Some(min), Some(max)) = (f.min, f.max)
                && max > min
                && (max - min) <= 1.0e6
            {
                set("editor", json!("slider"));
                set("min", json!(min));
                set("max", json!(max));
                set("step", json!(if int { 1.0 } else { 0.0 }));
                set("value", json!(first.parse::<f64>().unwrap_or(min)));
            }
            set("parts", json!(texts(1, &[])));
        }
        Editor::Vector(n) => set("parts", json!(texts(n, &["x", "y", "z", "w"]))),
        Editor::Euler => set("parts", json!(texts(3, &["x", "y", "z"]))),
        Editor::Bool => {
            set("editor", json!("check"));
            set("on", json!(first == "true"));
        }
        Editor::Enum => {
            set("editor", json!("pick"));
            set("options", json!(f.variants));
            set("choice", json!(first));
        }
        Editor::Color => {
            set("swatch", json!(first));
            set("parts", json!(texts(1, &["#rrggbb"])));
        }
        Editor::Text => set("parts", json!(texts(1, &[]))),
        Editor::Entity if first == "none" => {
            set("editor", json!("value"));
            set("text", json!("none"));
        }
        Editor::Entity => {
            set("editor", json!("link"));
            set("text", json!(first));
        }
        Editor::ReadOnly => {
            set("editor", json!("value"));
            set("text", json!(first));
        }
    }
    row
}

/// A handle as the panel shows it: the sound's or asset's id when the
/// engine loaded it, else its kind and index.
fn handle_name(cx: &Ctx, kind: crate::host::codec::Kind, bits: u64) -> String {
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
