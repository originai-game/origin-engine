//! World-anchored elements in documents (docs/ui.md §3.3): `<anchor
//! entity={u.entity} offset="0 2.1 0" occlusion="fade" declutter="stack"
//! clamp="edge">…</anchor>`, the native anchor ([`Ui::open_world_anchor`])
//! with its children laid out as any box. The anchor is placed every tick
//! from the camera and the entity's pose; its subtree is built again only
//! when a binding it reads changes ([`Ui::keep_world_anchors`] places the
//! ones not built this tick).
use super::*;
use crate::world::{
    AnchorScale, Declutter, EdgeClamp, WorldAnchorDesc, WorldOcclusion, WorldPanelDesc, WorldTarget,
};
use og_core::glam::Vec3;

fn vec3(text: &str) -> Option<Vec3> {
    let n: Vec<f32> = text
        .replace(',', " ")
        .split_whitespace()
        .filter_map(|w| w.parse().ok())
        .collect();
    match n.as_slice() {
        [x, y, z] => Some(Vec3::new(*x, *y, *z)),
        _ => None,
    }
}

/// The anchor `r` describes; `Err` names a bad attribute.
pub(super) fn anchor_desc(r: &Resolved, el: Element) -> Result<WorldAnchorDesc, String> {
    let target = match (r.num("entity"), r.str("worldAt")) {
        (Some(e), _) => WorldTarget::Entity(e as u64),
        (None, Some(p)) => WorldTarget::Point(
            vec3(p).ok_or_else(|| format!("at=\"{p}\" is three numbers, x y z"))?,
        ),
        (None, None) => return Err("an <anchor> takes entity={…} or at=\"x y z\"".into()),
    };
    let offset = match r.str("worldOffset") {
        Some(v) => vec3(v).ok_or_else(|| format!("offset=\"{v}\" is three numbers"))?,
        None => Vec3::ZERO,
    };
    let occlusion = match r.str("occlusion") {
        None | Some("none") => WorldOcclusion::None,
        Some("fade") => WorldOcclusion::Fade { alpha: 0.35 },
        Some("hide") => WorldOcclusion::Hide,
        Some(o) => return Err(format!("occlusion=\"{o}\" is none, fade or hide")),
    };
    let declutter = match r.str("declutter") {
        None | Some("none") => Declutter::None,
        Some("shift") => Declutter::Shift,
        Some("stack") => Declutter::Stack,
        Some("hide") => Declutter::Hide,
        Some(o) => return Err(format!("declutter=\"{o}\" is none, shift, stack or hide")),
    };
    let clamp = match r.str("worldClamp") {
        None | Some("none") => None,
        Some("edge") => Some(EdgeClamp::default()),
        Some(o) => return Err(format!("clamp=\"{o}\" is none or edge")),
    };
    let scale = match r.num("worldScaleAt") {
        Some(d) if d > 0.0 => AnchorScale::Distance {
            reference: d,
            min: 0.5,
            max: 2.0,
        },
        _ => AnchorScale::Constant,
    };
    Ok(WorldAnchorDesc {
        target,
        offset,
        element: el,
        occlusion,
        declutter,
        clamp,
        scale,
        priority: r.num("priority").unwrap_or(0.0) as i32,
        max_distance: r.num("maxDistance").unwrap_or(0.0).max(0.0),
        ..WorldAnchorDesc::default()
    })
}

/// Three numbers or `None`; `what` names the attribute in the error.
fn three(r: &Resolved, prop: &str, what: &str) -> Result<Option<Vec3>, String> {
    r.str(prop)
        .map(|v| vec3(v).ok_or_else(|| format!("{what}=\"{v}\" is three numbers")))
        .transpose()
}

/// The world panel `r` describes (`<panel world entity size rotation
/// src>`): the box `el` laid out at its CSS size, shown on a quad `size`
/// world units across turned by `rotation` (degrees about x, y, z).
pub(super) fn panel_desc(
    r: &Resolved,
    el: Element,
    texture: Option<og_render::TextureHandle>,
) -> Result<WorldPanelDesc, String> {
    let px = |s: Size, d: f32| match s {
        Size::Fixed(v) => v,
        _ => d,
    };
    let target = match (r.num("entity"), r.str("worldAt")) {
        (Some(e), _) => Some(WorldTarget::Entity(e as u64)),
        (None, Some(p)) => {
            Some(WorldTarget::Point(vec3(p).ok_or_else(|| {
                format!("at=\"{p}\" is three numbers, x y z")
            })?))
        }
        (None, None) => None,
    };
    let size = match r.str("worldSize") {
        Some(v) => {
            let n: Vec<f32> = v
                .split_whitespace()
                .filter_map(|w| w.parse().ok())
                .collect();
            match n.as_slice() {
                [w, h] if *w > 0.0 && *h > 0.0 => og_core::glam::Vec2::new(*w, *h),
                _ => {
                    return Err(format!(
                        "size=\"{v}\" is two positive numbers, width height"
                    ));
                }
            }
        }
        None => og_core::glam::Vec2::ONE,
    };
    let turn = three(r, "worldRotation", "rotation")?.unwrap_or(Vec3::ZERO);
    let rotation = og_core::glam::Quat::from_euler(
        og_core::glam::EulerRot::YXZ,
        turn.y.to_radians(),
        turn.x.to_radians(),
        turn.z.to_radians(),
    );
    Ok(WorldPanelDesc {
        target: texture,
        width: px(el.width, 480.0).max(1.0),
        height: px(el.height, 320.0).max(1.0),
        quad: target.map(|t| (t, rotation, size)),
        element: el,
    })
}

impl Ui {
    /// A `<panel world>`: its subtree drawn into the render target `src`
    /// names (shown by the game on a plane), and rays from the pointer
    /// that meet its quad reach its controls.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn build_world_panel(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        r: &Resolved,
        props: Props,
        path: String,
        key: u64,
        built: Built,
        build_children: impl FnOnce(&mut Ui, &mut Build<'_>, Option<Props>),
    ) -> Option<String> {
        let el = Self::box_of(
            r,
            Element {
                direction: Direction::Column,
                ..Element::default()
            },
            self.em_of(r),
        );
        let texture = r
            .num("viewportSrc")
            .and_then(|h| b.env.texture(f64::from(h)));
        let desc = match panel_desc(r, el, texture) {
            Ok(d) => d,
            Err(e) => {
                self.warnings
                    .push(format!("{}:{} ({path}): {e}", b.doc.source, n.line));
                return None;
            }
        };
        self.open_world_panel_keyed(path.as_str().into(), Some(key), desc);
        let i = self.parent();
        let mut p = Self::resolved_props(n, r);
        p.states = props.states;
        p.state = props.state;
        self.nodes[i].props = p;
        if let Some(name) = r.str("name").filter(|n| !n.is_empty()) {
            self.nodes[i].a11y.name = Some(name.to_string());
        }
        self.keep_world_panel(i, key);
        self.note_built(built);
        build_children(self, b, None);
        let resp = self.close();
        self.fire(b, n, key, &path, resp);
        Some(path)
    }

    /// An `<anchor>`: opened as the native anchor, kept for placement while
    /// its subtree is not built again.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn build_anchor(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        r: &Resolved,
        props: Props,
        path: String,
        key: u64,
        built: Built,
        build_children: impl FnOnce(&mut Ui, &mut Build<'_>, Option<Props>),
    ) -> Option<String> {
        let el = Self::box_of(
            r,
            Element {
                direction: Direction::Column,
                ..Element::default()
            },
            self.em_of(r),
        );
        let desc = match anchor_desc(r, el) {
            Ok(d) => d,
            Err(e) => {
                self.warnings
                    .push(format!("{}:{} ({path}): {e}", b.doc.source, n.line));
                return None;
            }
        };
        self.open_world_anchor_keyed(path.as_str().into(), Some(key), desc.clone());
        let i = self.parent();
        let mut p = Self::resolved_props(n, r);
        p.states = props.states;
        p.state = props.state;
        self.nodes[i].props = p;
        if let Some(name) = r.str("name").filter(|n| !n.is_empty()) {
            self.nodes[i].a11y.name = Some(name.to_string());
        }
        self.world.kept.insert(self.nodes[i].key, (desc, key));
        self.note_built(built);
        build_children(self, b, None);
        let resp = self.close();
        self.fire(b, n, key, &path, resp);
        Some(path)
    }
}

#[cfg(test)]
mod tests {
    use crate::doc::UiDocument;
    use crate::reactive::SourceId;
    use crate::reactive::session::Command;
    use crate::world::WorldView;
    use crate::{InputFrame, Ui, UiDesc, Viewport};
    use og_core::glam::{Quat, Vec3};
    use serde_json::json;
    use std::collections::BTreeMap;

    const TAGS: &str = r#"
<meta name="component" content="tags">
<div id="Root">
  {#each $units as u (u.id)}
    <anchor id="Tag" entity={u.entity} offset="0 2 0"><p id="Name">{u.name}</p></anchor>
  {/each}
</div>
"#;

    /// Anchors follow their entities every tick without being built again;
    /// a changed name builds that tag alone; a removed row's anchor leaves.
    ///
    /// Ways it could fail: an anchor placed only on the tick it was built
    /// (frozen when the camera moves), every anchor rebuilt per tick, or a
    /// removed anchor still placed.
    #[test]
    fn anchors_follow_entities_and_build_only_what_changed() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(UiDocument::parse("tags.svelte", TAGS).unwrap());
        let units = SourceId::new("test", "units");
        let rows = |names: &[&str]| -> Vec<serde_json::Map<String, serde_json::Value>> {
            names
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    json!({"id": format!("u{i}"), "entity": i + 1, "name": n})
                        .as_object()
                        .unwrap()
                        .clone()
                })
                .collect()
        };
        ui.reactive
            .execute(Command::Collection {
                source: units.clone(),
                key: "id".into(),
                items: rows(&["Ash", "Birch"]),
            })
            .unwrap();
        ui.mount_document(
            SourceId::new("test", "tags"),
            "tags",
            BTreeMap::from([("units".to_string(), units.clone())]),
            BTreeMap::new(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let tick = |ui: &mut Ui, eye_x: f32| {
            ui.set_world_view(Some(WorldView::look_at(
                Vec3::new(eye_x, 3.0, 10.0),
                Vec3::new(eye_x, 1.0, 0.0),
                Vec3::Y,
                50.0,
                800.0 / 600.0,
                0.1,
                100.0,
            )));
            ui.set_world_poses(
                &[1u64, 2]
                    .iter()
                    .map(|&e| {
                        (
                            e,
                            (
                                Vec3::new(e as f32 * 2.0 - 3.0, 0.0, 0.0),
                                Quat::IDENTITY,
                                Vec3::ONE,
                            ),
                        )
                    })
                    .collect::<Vec<_>>(),
            );
            ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end().stats
        };
        for _ in 0..3 {
            tick(&mut ui, 0.0);
        }
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        let xs = |ui: &Ui| {
            let mut v: Vec<f32> = ui
                .probe_elements()
                .into_iter()
                .filter(|e| e.kind == "world")
                .map(|e| e.x)
                .collect();
            v.sort_by(f32::total_cmp);
            v
        };
        let before = xs(&ui);
        assert_eq!(before.len(), 2, "{before:?}");
        // The camera moves: both tags move, nothing is built again.
        let moved = tick(&mut ui, 1.0);
        assert_eq!((moved.bindings, moved.reconciled), (0, 0), "{moved:?}");
        let after = xs(&ui);
        assert!(
            after.iter().zip(&before).all(|(a, b)| a < b),
            "{before:?} → {after:?}"
        );
        // One name: that tag alone.
        ui.reactive
            .execute(Command::Patch {
                source: units.clone(),
                key: "u1".into(),
                fields: json!({"name": "Cedar"}).as_object().unwrap().clone(),
            })
            .unwrap();
        let named = tick(&mut ui, 1.0);
        assert!(
            named.bindings >= 1 && (1..=2).contains(&named.reconciled),
            "{named:?}"
        );
        assert_eq!(xs(&ui).len(), 2);
        let names: Vec<String> = ui
            .probe_elements()
            .into_iter()
            .filter(|e| e.path.ends_with("/Name"))
            .map(|e| e.text)
            .collect();
        assert!(names.contains(&"Cedar".to_string()), "{names:?}");
        let idle = tick(&mut ui, 1.0);
        assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{idle:?}");
        // A row leaves: its anchor leaves.
        ui.reactive
            .execute(Command::Remove {
                source: units,
                key: "u1".into(),
            })
            .unwrap();
        for _ in 0..2 {
            tick(&mut ui, 1.0);
        }
        assert_eq!(xs(&ui).len(), 1);
    }

    /// A world element's layer is keyed by its own identity, so what it
    /// holds has the keys the retained document gives it, two levels down
    /// as at the top, and one can be its document's root.
    ///
    /// Ways it could fail: a layer keyed from its id under the root (its
    /// children's keys then match no retained element: presses, focus and
    /// patches miss them), or a root anchor refused or colliding.
    #[test]
    fn a_world_layer_keys_its_subtree_as_the_document_does() {
        let mut ui = Ui::new(UiDesc::default());
        let doc = r#"<meta name="component" content="sign">
<anchor id="Sign" at="0 1 0"><div id="Box"><div id="Line"><p id="Word">Hi</p></div></div></anchor>"#;
        ui.define_document(UiDocument::parse("sign.svelte", doc).unwrap());
        ui.mount_document(
            SourceId::new("test", "sign"),
            "sign",
            BTreeMap::new(),
            BTreeMap::new(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        for _ in 0..2 {
            ui.set_world_view(Some(WorldView::look_at(
                Vec3::new(0.0, 3.0, 10.0),
                Vec3::ZERO,
                Vec3::Y,
                50.0,
                800.0 / 600.0,
                0.1,
                100.0,
            )));
            ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
        }
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        let path = "test:sign/Box/Line/Word";
        let word = ui
            .nodes
            .iter()
            .position(|n| n.id == "Word")
            .expect("the word is built");
        assert_eq!(ui.nodes[word].key, crate::id::key_of_path(path));
        assert!(ui.element(path).is_some(), "{path}");
        let layer = ui.nodes[ui.nodes[ui.nodes[word].parent].parent].parent;
        assert_eq!(ui.nodes[layer].parent, crate::layout::NONE);
        assert_eq!(ui.nodes[layer].key, crate::id::key_of_path("test:sign"));
    }
}
