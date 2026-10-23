//! `describe`: the frame as data (docs/design.md §3.4). The visible
//! entities with their screen boxes, depth order and occlusion (from the
//! host's [`crate::SceneView`], the renderer's entity-id pass), their names,
//! archetypes and labels; the UI elements with their rects and text (the
//! same tree `dump_ui` writes); and the camera. Deterministic: the same
//! world gives the same JSON on every host and backend.

use crate::probe::Probe;
use og_ecs::World;
use serde_json::{Value, json};

fn round(v: f32, digits: i32) -> f64 {
    let k = 10f64.powi(digits);
    (f64::from(v) * k).round() / k
}

fn rect(r: [f32; 4]) -> Value {
    json!({"x": round(r[0], 2), "y": round(r[1], 2), "width": round(r[2], 2), "height": round(r[3], 2)})
}

impl Probe {
    /// `describe`: what the frame at the current tick shows, as data
    /// (IDL `probe.Description`). Entities are nearest first (`order` 0),
    /// then by id; without a scene view the entity list is empty and
    /// `pass` and `camera` are null.
    pub fn describe(&mut self, world: &World) -> Value {
        let seen = self.scene_view.as_mut().and_then(|v| v.seen(world));
        let mut entities = Vec::new();
        let mut views = Vec::new();
        if let Some(seen) = &seen {
            // Each secondary camera's entities, and per entity the views
            // that show it.
            let mut shows: Vec<(og_ecs::Entity, String)> = Vec::new();
            for v in &seen.views {
                let id = self.names.id_of(world, v.entity);
                for s in &v.entities {
                    shows.push((s.entity, id.clone()));
                }
                views.push(json!({
                    "id": id,
                    "name": self.names.name_of(world, v.entity),
                    "kind": if v.mirror { "mirror" } else { "camera" },
                    "size": {"width": v.size.0, "height": v.size.1},
                    "entities": self.shown_entities(world, &v.entities, &[]),
                }));
            }
            entities = self.shown_entities(world, &seen.entities, &shows);
        }
        let ui: Vec<Value> = self
            .ui
            .as_ref()
            .map(|ui| {
                ui.elements(world)
                    .iter()
                    .map(|e| {
                        json!({
                            "id": e.id, "parent": e.parent, "kind": e.kind, "role": e.role,
                            "text": e.text, "value": e.value, "states": e.states,
                            "box": rect([e.x, e.y, e.width, e.height]),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        let surface = match self.ui.as_ref().and_then(|ui| ui.surface(world)) {
            Some((w, h, dpr)) => json!({"width": w, "height": h, "dpr": dpr}),
            None => match self.capture.as_ref().map(|c| c.target_size()) {
                Some((w, h)) if w > 0 => json!({"width": w, "height": h, "dpr": 1}),
                _ => Value::Null,
            },
        };
        json!({
            "kind": "describe",
            "tick": world.time().tick,
            "surface": surface,
            "pass": seen.as_ref().map(|s| json!({"width": s.pass.0, "height": s.pass.1})),
            "camera": seen.as_ref().map_or(Value::Null, |s| s.camera.clone()),
            "entities": entities,
            "views": views,
            "ui": ui,
        })
    }

    /// `shown` as described entities, nearest first (`order` 0), then by
    /// id; each names the views (`shows`: entity, view id) it shows in.
    fn shown_entities(
        &mut self,
        world: &World,
        shown: &[crate::Shown],
        shows: &[(og_ecs::Entity, String)],
    ) -> Vec<Value> {
        let mut entities: Vec<(f32, String, Value)> = Vec::new();
        for s in shown {
            let id = self.names.id_of(world, s.entity);
            let archetype: Vec<String> = world
                .components_of(s.entity)
                .unwrap_or_default()
                .into_iter()
                .filter_map(|c| world.schema(c).ok().map(|schema| schema.name.clone()))
                .collect();
            let mut in_views: Vec<&str> = shows
                .iter()
                .filter(|(e, _)| *e == s.entity)
                .map(|(_, v)| v.as_str())
                .collect();
            in_views.dedup();
            let value = json!({
                "id": id,
                "name": self.names.name_of(world, s.entity),
                "order": 0,
                "archetype": archetype,
                "labels": s.labels,
                "box": rect(s.rect),
                "drawn": rect(s.drawn),
                "point": {"x": round(s.point[0], 2), "y": round(s.point[1], 2)},
                "depth": round(s.depth, 4),
                "coverage": round(s.coverage, 5),
                "occlusion": round(s.occlusion, 4),
                "views": in_views,
            });
            entities.push((s.depth, id, value));
        }
        entities.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        entities
            .into_iter()
            .enumerate()
            .map(|(order, (_, _, mut v))| {
                v["order"] = json!(order);
                v
            })
            .collect()
    }
}
