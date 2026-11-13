//! The panel's lists: entities, a prefab instance, one entity's components, and resources.

use super::*;

pub(super) fn entities(ui: &mut Ui, s: &mut State, cx: &Ctx) {
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

/// A prefab instance's field as `apply_prefab` names it: `Component.field`
/// with the engine's `og.` dropped (`og.Transform.position` is
/// `Transform.position`, as the scene's text writes it).
pub(crate) fn prefab_field(component_field: &str) -> String {
    component_field
        .strip_prefix("og.")
        .unwrap_or(component_field)
        .to_string()
}

/// The prefab an entity was placed from, and what its instance overrides:
/// the scene's override lines and this session's unsaved edits, each with
/// Apply (into the prefab) and Revert (back to it), and both for all.
pub(super) fn prefab(ui: &mut Ui, s: &mut State, e: Entity, id: &str, info: &serde_json::Value) {
    let t = *ui.theme();
    let p = t.palette;
    let name = info["prefab"].as_str().unwrap_or("");
    let node = info["node"].as_str().unwrap_or("");
    // The fields it overrides: saved lines, then unsaved edits.
    let mut fields: Vec<(String, bool)> = info["overrides"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str())
        .map(|f| (prefab_field(f), true))
        .collect();
    let prefix = format!("{id}.");
    for target in &s.ins.dirty {
        if let Some(rest) = target.strip_prefix(&prefix) {
            let f = prefab_field(rest);
            if !fields.iter().any(|(g, _)| *g == f) {
                fields.push((f, false));
            }
        }
    }
    // The `set` targets they show under (og.Transform's with its og.).
    for (f, _) in &fields {
        let (c, field) = f.split_once('.').unwrap_or((f, ""));
        for comp in [c.to_string(), format!("og.{c}")] {
            s.overridden.insert(format!("{id}.{comp}.{field}"));
        }
    }
    let row = Element {
        direction: Direction::Row,
        width: Size::grow(),
        cross: Cross::Center,
        gap: t.space.xs,
        ..Element::default()
    };
    ui.row("Prefab", row, |ui| {
        let what = if node.is_empty() {
            format!("Prefab {name}")
        } else {
            format!("Prefab {name} · {node}")
        };
        label(ui, "Of", &what, p.info, true, Size::grow());
        let none = fields.is_empty();
        let apply = ui.button(
            "ApplyAll",
            "Apply all",
            ButtonDesc {
                disabled: none,
                ..small(ButtonVariant::Secondary)
            },
        );
        if apply.clicked {
            s.ins.requests.push(Request::Prefab {
                entity: e,
                target: None,
                fields: fields.iter().map(|(f, _)| f.clone()).collect(),
                apply: true,
            });
        }
        ui.tooltip("Write every override into the prefab's file: every instance takes it");
        let revert = ui.button(
            "RevertAll",
            "Revert all",
            ButtonDesc {
                disabled: none,
                ..small(ButtonVariant::Quiet)
            },
        );
        if revert.clicked {
            s.ins.requests.push(Request::Prefab {
                entity: e,
                target: None,
                fields: fields.iter().map(|(f, _)| f.clone()).collect(),
                apply: false,
            });
        }
        ui.tooltip("Take every override back to the prefab's value");
    });
    for (f, saved) in &fields {
        ui.row(format!("O:{f}"), row, |ui| {
            ui.spacer("Indent", Some(Size::Fixed(6.0 + t.space.xs)));
            let text = if *saved {
                f.clone()
            } else {
                format!("{f} (unsaved)")
            };
            label(ui, "Field", &text, p.ink_muted, true, Size::grow());
            if ui
                .button("Apply", "Apply", small(ButtonVariant::Quiet))
                .clicked
            {
                s.ins.requests.push(Request::Prefab {
                    entity: e,
                    target: Some(f.clone()),
                    fields: vec![f.clone()],
                    apply: true,
                });
            }
            ui.tooltip("Write this value into the prefab");
            if ui
                .button("Revert", "Revert", small(ButtonVariant::Quiet))
                .clicked
            {
                s.ins.requests.push(Request::Prefab {
                    entity: e,
                    target: Some(f.clone()),
                    fields: vec![f.clone()],
                    apply: false,
                });
            }
            ui.tooltip("Back to the prefab's value");
        });
    }
}

/// The components of an entity, og.Transform first, then by name.
pub(super) fn components(world: &World, e: Entity) -> Vec<Schema> {
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

pub(super) fn entity(ui: &mut Ui, s: &mut State, cx: &Ctx, e: Entity) {
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
    s.overridden.clear();
    if let Some(info) = cx.prefab {
        prefab(ui, s, e, &id, info);
    }
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

pub(super) fn resources(ui: &mut Ui, s: &mut State, cx: &Ctx) {
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
