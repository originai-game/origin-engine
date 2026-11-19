//! The engine's component documents (`packages/engine/src/ui/components`):
//! each parses, takes its strings and builds from its `fixture:default`
//! props at a desktop and a phone width with no author or frame warning.
//! A document that stops cooking, names a missing string, or builds a
//! tree the frame refuses fails here before a game ejects it.

use crate::support;
use og_ui::*;
use support::*;

#[test]
fn world_previews_aim_at_the_subject_without_recentering_edge_fixtures() {
    use og_core::glam::Vec3;
    let (docs, messages) = engine_documents(&["src/ui/components"]);
    for (w, h) in [(1280.0, 720.0), (390.0, 844.0)] {
        for scale in [1.0, 1.5] {
            let mut ui = ui_with_font(test_font_metrics());
            ui.set_theme(Theme {
                text_scale: scale,
                ..*ui.theme()
            });
            let doc = docs
                .iter()
                .find(|d| d.path.ends_with("world/nameplate.svelte"))
                .unwrap();
            preview(&mut ui, &docs, doc, &messages, w, h);
            let view = ui.world_view().unwrap();
            let anchor = ui.world_anchor_points()[0];
            let WorldTarget::Point(at) = anchor.target else {
                panic!("point fixture")
            };
            let pivot = at + anchor.offset;
            assert_eq!(view.forward, Vec3::NEG_Z, "level label preview");
            assert!((view.eye - pivot - Vec3::Z * 6.0).length() < 0.001);
            let (x, y, _) = view.project(pivot, w, h).unwrap();
            assert!((x - w / 2.0).abs() < 0.01 && (y - h / 2.0).abs() < 0.01);

            let mut ui = ui_with_font(test_font_metrics());
            let doc = docs
                .iter()
                .find(|d| d.path.ends_with("world/telegraph.svelte"))
                .unwrap();
            preview(&mut ui, &docs, doc, &messages, w, h);
            assert!(
                (ui.world_view().unwrap().forward.y + 35.0_f32.to_radians().sin()).abs() < 0.001
            );
            assert!(ui.frame().grounds.iter().all(|g| g.corners.is_some()));

            let doc = docs
                .iter()
                .find(|d| d.path.ends_with("world/path-line.svelte"))
                .unwrap();
            let mut ui = ui_with_font(test_font_metrics());
            ui.set_theme(Theme {
                text_scale: scale,
                ..*ui.theme()
            });
            preview(&mut ui, &docs, doc, &messages, w, h);
            let pad = 24.0 * scale;
            assert!(!ui.frame().grounds.is_empty());
            assert!(
                ui.frame().grounds.iter().all(|g| {
                    g.bounds.x >= pad
                        && g.bounds.y >= pad
                        && g.bounds.right() <= w - pad
                        && g.bounds.bottom() <= h - pad
                }),
                "endpoint badges must fit beyond the ground square"
            );

            let doc = docs
                .iter()
                .find(|d| d.path.ends_with("world/waypoint.svelte"))
                .unwrap();
            let mut props = doc.props.clone();
            props["at"] = serde_json::json!("200 0 0");
            let edge = EngineDocument {
                path: doc.path.clone(),
                doc: doc.doc.clone(),
                props,
            };
            let mut ui = ui_with_font(test_font_metrics());
            preview(&mut ui, &docs, &edge, &messages, w, h);
            assert_eq!(ui.world_view(), Some(support::view(w, h)));
            assert!(ui.world_anchor_points().iter().any(|a| a.clamped));
        }
    }
}

#[test]
fn document_preview_camera_keeps_the_ground_in_frame_on_a_phone() {
    for (w, h) in [(1280.0, 720.0), (390.0, 844.0)] {
        for x in [-6.0, 6.0] {
            for z in [-6.0, 6.0] {
                let (sx, sy, _) = view(w, h)
                    .project(og_core::glam::Vec3::new(x, 0.0, z), w, h)
                    .unwrap();
                assert!(
                    (24.0..=w - 24.0).contains(&sx) && (24.0..=h - 24.0).contains(&sy),
                    "{w}x{h}: world ground cut at {sx},{sy}"
                );
            }
        }
    }
}

#[test]
fn loot_preview_stays_visible_with_its_authored_distance_limit_on_a_phone() {
    let (docs, messages) = engine_documents(&["src/ui/components"]);
    let doc = docs
        .iter()
        .find(|d| d.path.ends_with("world/loot-label.svelte"))
        .unwrap();
    for (w, h) in [(1280.0, 720.0), (390.0, 844.0)] {
        let mut ui = ui_with_font(test_font());
        preview(&mut ui, &docs, doc, &messages, w, h);
        assert!(ui.frame().items.iter().any(|item| {
            let cmd = match &item.cmd {
                DrawCmd::StyledText { text, .. } => &**text,
                cmd => cmd,
            };
            matches!(cmd, DrawCmd::Text { text, color, .. } if text == "Ember Blade" && color[3] > 0.5)
        }), "{w}: camera must not fade the loot label out of the preview");
    }
}

#[test]
fn world_document_previews_do_not_cover_the_projected_scene() {
    let (docs, messages) = engine_documents(&["src/ui/components"]);
    for name in [
        "loot-label",
        "nameplate",
        "path-line",
        "placement-ghost",
        "selection-ring",
        "speech-bubble",
        "telegraph",
        "waypoint",
    ] {
        let doc = docs
            .iter()
            .find(|d| d.path.ends_with(&format!("world/{name}.svelte")))
            .unwrap();
        let mut ui = ui_with_font(test_font_metrics());
        preview(&mut ui, &docs, doc, &messages, 1280.0, 720.0);
        assert!(
            !ui.frame().grounds.is_empty()
                || ui
                    .frame()
                    .items
                    .iter()
                    .any(|item| matches!(item.cmd, DrawCmd::World(_))),
            "{name}: no projected paint"
        );
        assert!(
            fills_in(&ui, Rect::new(0.0, 0.0, 1280.0, 720.0)).is_empty(),
            "{name}: an opaque screen-sized preview panel covers the world layer"
        );
    }
}

#[test]
fn spectator_rows_and_party_seals_keep_their_content_inside_the_frame() {
    let (docs, messages) = engine_documents(&["src/ui/components"]);
    for (w, h) in [(1280.0, 720.0), (390.0, 844.0)] {
        let mut ui = ui_with_font(test_font_metrics());
        let doc = docs
            .iter()
            .find(|d| d.path.ends_with("arena/spectator.svelte"))
            .unwrap();
        preview(&mut ui, &docs, doc, &messages, w, h);
        let watching = ui
            .probe_elements()
            .into_iter()
            .find(|e| e.text == "Watching")
            .unwrap();
        let row = ui.elements().iter().find(|e| e.id == "Follow#kai").unwrap();
        assert!(
            watching.y + watching.height <= row.y + row.height - ui.theme().space.xs,
            "{w}: Watching needs block padding"
        );
        let mut ui = ui_with_font(test_font_metrics());
        let doc = docs
            .iter()
            .find(|d| d.path.ends_with("rpg/party.svelte"))
            .unwrap();
        preview(&mut ui, &docs, doc, &messages, w, h);
        let elements = ui.probe_elements();
        let seal = elements.iter().find(|e| e.name == "Level 8").unwrap();
        let level = elements
            .iter()
            .find(|e| e.text == "8" && e.path.starts_with(&format!("{}/", seal.path)))
            .unwrap();
        let label = elements
            .iter()
            .find(|e| e.text == "Lv" && e.path.starts_with(&format!("{}/", seal.path)))
            .unwrap();
        assert!(
            !level.clipped && level.y + level.height <= seal.y + seal.height,
            "{w}: level must fit without a scrollbar: {level:?}"
        );
        assert!(
            seal.height <= label.height + level.height + ui.theme().space.sm,
            "{w}: the seal must fit its text, not stretch over the portrait"
        );
    }
}

#[test]
fn document_preview_stage_preserves_fixtures_and_contains_fixed_roots() {
    use serde_json::json;
    let messages = Messages(Default::default());
    let mount = reactive::SourceId::new("test", "preview");
    for kind in ["component", "screen"] {
        let text = format!(
            r##"<meta name="{kind}" content="table" data-focus="Choose">
<meta name="prop:rows" content="list">
<meta name="prop:count" content="number" data-default="7">
<div id="FixtureRoot">
  <p>Fixture {{count}}</p>
  {{#each rows as row (row.action)}}<p>{{row.label}}</p>{{/each}}
  <button id="Other">Other</button>
  <button id="Choose">Choose</button>
</div>
<style>:host {{ position: fixed; inset: 0; background: #123456; }}</style>"##
        );
        let doc = EngineDocument {
            path: "preview-fixture.svelte".into(),
            doc: UiDocument::parse("preview-fixture.svelte", &text).unwrap(),
            props: json!({"rows": [{"action": "jump", "label": "Jump"}]}),
        };
        for (width, height) in [(1280.0, 720.0), (390.0, 844.0)] {
            let mut ui = ui_with_font(test_font_metrics());
            let warnings = preview(
                &mut ui,
                std::slice::from_ref(&doc),
                &doc,
                &messages,
                width,
                height,
            );
            assert!(warnings.is_empty(), "{kind}: {warnings:?}");
            let root_path = ui
                .doc_built()
                .into_iter()
                .find(|e| e.doc == doc.doc.name && e.node == doc.doc.root)
                .unwrap()
                .path;
            let root = ui.elements().iter().find(|e| e.path == root_path).unwrap();
            for (got, expected) in [
                (root.x, 24.0),
                (root.y, 24.0),
                (root.width, width - 48.0),
                (root.height, height - 48.0),
            ] {
                assert!((got - expected).abs() < 0.1, "{kind}: {root:?}");
            }
            assert_eq!(
                fills_in(&ui, Rect::new(0.0, 0.0, width, height)).is_empty(),
                kind == "component",
                "{kind}: only a screen's own scrim covers the viewport"
            );
            assert!(
                !fills_in(&ui, Rect::new(24.0, 24.0, width - 48.0, height - 48.0)).is_empty(),
                "the original :host still paints its root"
            );
            assert_eq!(ui.layer_stack().len(), usize::from(kind == "screen"));
            if kind == "screen" {
                assert!(
                    ui.focus_path(0).ends_with("/Choose"),
                    "{}",
                    ui.focus_path(0)
                );
            }
            assert_eq!(ui.mount_values(&mount).unwrap()["count"], 7);
            assert!(ui.probe_elements().iter().any(|item| item.text == "Jump"));
            ui.set_mount_values(
                &mount,
                &json!({"count": 11, "rows": [{"action": "hold", "label": "Hold"}]}),
            )
            .unwrap();
            ui.begin(InputFrame::default(), Viewport::new(width, height));
            ui.build_retained_documents(&messages).unwrap();
            ui.end();
            assert_eq!(ui.mount_values(&mount).unwrap()["count"], 11);
            assert!(ui.probe_elements().iter().any(|item| item.text == "Hold"));
            assert!(!ui.probe_elements().iter().any(|item| item.text == "Jump"));
        }
    }
}

#[test]
fn component_documents_build_from_their_fixtures() {
    let (docs, messages) = engine_documents(&["src/ui/components"]);
    assert!(!docs.is_empty());
    let mut failures = Vec::new();
    // A genre kit's documents (rpg/, shooter/) mount from their own
    // fixtures, whose art is their sheet's, in
    // packages/engine/tests/<kit>.test.ts; the by-function folders here.
    let top = |d: &&EngineDocument| {
        let path = &d.path["src/ui/components/".len()..];
        ["base/", "hud/", "world/", "panels/", "meta/"].iter().any(|f| path.starts_with(f))
            // Their fixtures draw the rpg sheet's item art (rpg.test.ts).
            && !["panels/backpack.svelte", "panels/item-card.svelte"].contains(&path)
    };
    for doc in docs.iter().filter(top) {
        for (width, height) in [(1280.0, 720.0), (390.0, 844.0)] {
            let mut ui = ui_with_font(test_font_metrics());
            ui.set_icons(icons(og_draw::TextureHandle::from_slot(1)));
            let warnings = preview(&mut ui, &docs, doc, &messages, width, height);
            if !warnings.is_empty() {
                failures.push(format!("{} at {width}: {warnings:?}", doc.path));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Every built-in document, the screens and the groups' among them,
/// mounts and builds from its `fixture:default` alone: the values a
/// fixture leaves out are read as unset, not refused.
///
/// Ways it could fail: a document reading a store or prop its fixture
/// leaves out (the mount has no such store), or iterating a fixture's
/// list by a key other than `id` (the list is held as one value).
#[test]
fn every_built_in_document_mounts_from_its_fixture() {
    let (docs, messages) = engine_documents(&["src/ui/components", "src/ui/screens"]);
    assert!(docs.len() > 100, "{} documents", docs.len());
    for doc in &docs {
        let mut ui = ui_with_font(test_font_metrics());
        ui.set_icons(icons(og_draw::TextureHandle::from_slot(1)));
        preview(&mut ui, &docs, doc, &messages, 1280.0, 720.0);
        assert!(
            ui.doc_built().iter().any(|built| {
                built.doc == doc.doc.name && ui.elements().iter().any(|e| e.path == built.path)
            }),
            "{}: draws no element",
            doc.path
        );
    }
}

/// Every built-in component is usable at its zero value: a game's page
/// that writes `<Name />` with no props mounts it and builds with no
/// warning, each prop read at its default.
///
/// Ways it could fail: a prop that must be passed (mount refused), a
/// default its markup cannot take (a word attribute given "", a divisor
/// of 0 reaching a length), or a group's component warning on art its
/// sheet would give.
#[test]
fn every_built_in_component_builds_with_no_props() {
    let (docs, messages) = engine_documents(&["src/ui/components"]);
    let mut failures = Vec::new();
    for doc in &docs {
        let name = &doc.doc.name;
        let page = format!(
            "<meta name=\"component\" content=\"page\">\n<div id=\"Page\"><{name} /></div>"
        );
        let mut ui = ui_with_font(test_font_metrics());
        ui.set_icons(icons(og_draw::TextureHandle::from_slot(1)));
        let page = EngineDocument {
            path: format!("page of {}", doc.path),
            doc: UiDocument::parse("page.svelte", &page).unwrap(),
            props: serde_json::json!({}),
        };
        ui.define_document(page.doc.clone());
        let mounted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            preview(&mut ui, &docs, &page, &messages, 1280.0, 720.0)
        }));
        match mounted {
            Ok(warnings) if warnings.is_empty() => {}
            Ok(warnings) => failures.push(format!("{}: {warnings:?}", doc.path)),
            Err(_) => failures.push(format!("{}: does not mount with no props", doc.path)),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// A friend's avatar is round in the default style, as the base Avatar
/// is (`--avatar-radius`): its well draws as a circle.
///
/// Ways it could fail: the avatar's radius left to the role it wears
/// last (a well's control radius), so it draws as a rounded square.
#[test]
fn friends_avatars_are_round() {
    let (docs, messages) = engine_documents(&["src/ui/components"]);
    let doc = docs.iter().find(|d| d.doc.name == "Friends").unwrap();
    let mut ui = ui_with_font(test_font_metrics());
    ui.set_icons(icons(og_draw::TextureHandle::from_slot(1)));
    preview(&mut ui, &docs, doc, &messages, 1280.0, 720.0);
    let avatars: Vec<_> = ui
        .probe_elements()
        .into_iter()
        .filter(|e| {
            e.detail
                .get("part")
                .is_some_and(|p| p.to_string().contains("Friends.avatar"))
        })
        .collect();
    assert!(!avatars.is_empty(), "the fixture shows avatars");
    for a in &avatars {
        let centre = og_core::glam::Vec2::new(a.x + a.width / 2.0, a.y + a.height / 2.0);
        let radii = ui
            .frame()
            .items
            .iter()
            .find_map(|i| match &i.cmd {
                DrawCmd::Shape { shape, .. } if (shape.center - centre).length() < 1.0 => {
                    match shape.geometry {
                        og_draw::UiGeometry::Rect { radii, .. } => Some(radii),
                        _ => None,
                    }
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("{} draws its well", a.path));
        assert!(
            radii
                .iter()
                .all(|&r| r >= a.width.min(a.height) / 2.0 - 0.5),
            "{}: {radii:?} on {}x{}",
            a.path,
            a.width,
            a.height
        );
    }
}

/// The ground marks drape over the scene (c8e4b7a8c): each document's
/// `<ground>` reaches the renderer as layers carrying its one drape (its
/// marks, and each of its labels on the same quad). Fails when a mark is
/// rebuilt as a flat panel or a screen-space box.
#[test]
fn ground_mark_documents_reach_the_renderer_as_draped_grounds() {
    let (docs, messages) = engine_documents(&["src/ui/components"]);
    for name in ["SelectionRing", "Telegraph", "PathLine", "PlacementGhost"] {
        let doc = docs.iter().find(|d| d.doc.name == name).unwrap();
        let (width, height) = (1280.0, 720.0);
        let mut ui = ui_with_font(test_font_metrics());
        preview(
            &mut ui,
            std::slice::from_ref(doc),
            doc,
            &messages,
            width,
            height,
        );
        let mut overlay = og_draw::Overlay::new(width, height, 1.0);
        ui.frame().emit(&EmitDesc::default(), &mut overlay);
        let mut drapes: Vec<og_draw::UiDrape> = Vec::new();
        for g in overlay.ui.layers.iter().filter_map(|l| l.ground) {
            if !drapes.contains(&g.drape) {
                drapes.push(g.drape);
            }
        }
        assert_eq!(drapes.len(), 1, "{name}: {} draped ground(s)", drapes.len());
    }
}

/// The touch controls are sources of the actions they name where the
/// primary pointer is a finger, and of nothing under a mouse or while
/// disabled: each control shown is one zone of the action map, the stick at
/// the start side and the buttons at the end with the first nearest the
/// corner. Fails when a hidden or disabled control still feeds its action,
/// when `always` does not show them under a mouse, when the d-pad or the
/// second stick names the wrong action, or when the buttons leave the
/// corner.
#[test]
fn touch_controls_feed_their_actions_under_a_finger_only() {
    use og_input::ZoneKind;
    let (docs, messages) = engine_documents(&["src/ui/components"]);
    let doc = docs.iter().find(|d| d.doc.name == "TouchControls").unwrap();
    let (width, height) = (844.0, 390.0);
    // A game's HUD page uses the component by name, its props from stores,
    // and the engine mounts the page.
    let page = r#"<meta name="screen" content="pad" data-layer="hud">
<div id="Page"><TouchControls move={$move} pad={$pad} look={$look} deadZone={$deadZone} actions={$actions} always={$always} disabled={$disabled} /></div>
<style>
#Page { display: flex; flex-direction: column; justify-content: end; width: stretch; height: stretch; }
</style>"#;
    let zones = |coarse: bool, change: serde_json::Value| -> Vec<og_input::ControlZone> {
        use og_ui::reactive::{SourceId, wire};
        use serde_json::json;
        let mut ui = ui_with_font(test_font_metrics());
        ui.set_icons(icons(og_draw::TextureHandle::from_slot(1)));
        for d in &docs {
            ui.define_document(d.doc.clone());
        }
        ui.define_document(UiDocument::parse("pad.svelte", page).unwrap());
        let mut props = doc.props.clone();
        props["always"] = json!(false);
        for (name, value) in change.as_object().unwrap() {
            props[name] = value.clone();
        }
        let mut inputs = std::collections::BTreeMap::new();
        for (name, value) in props.as_object().unwrap() {
            let source = SourceId::new("test", name);
            let command = match value {
                serde_json::Value::Array(items) => {
                    json!({"op": "collection", "source": source, "key": "id", "items": items})
                }
                _ => json!({"op": "signal", "source": source, "initial": value}),
            };
            let reply = wire::decode(&ui.apply_reactive(&wire::encode(&json!([command])).unwrap()))
                .unwrap();
            assert_eq!(reply[0]["ok"], true, "{reply}");
            inputs.insert(name.clone(), source);
        }
        ui.mount_document(
            SourceId::new("test", "pad"),
            "pad",
            inputs,
            Default::default(),
            &messages,
        )
        .unwrap();
        let mut zones = Vec::new();
        for _ in 0..4 {
            let mut viewport = Viewport::new(width, height);
            viewport.coarse_pointer = coarse;
            ui.begin(InputFrame::default(), viewport);
            ui.build_retained_documents(&messages).unwrap();
            ui.end();
            assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
            if let Some(now) = ui.take_control_zones() {
                zones = now;
            }
        }
        zones.sort_by(|a, b| a.x.total_cmp(&b.x));
        zones
    };
    let named = |zones: &[og_input::ControlZone]| -> Vec<(String, ZoneKind)> {
        zones.iter().map(|z| (z.action.clone(), z.kind)).collect()
    };
    let none = serde_json::json!({});

    let shown = zones(true, none.clone());
    assert_eq!(
        named(&shown),
        [
            ("move".to_string(), ZoneKind::Stick),
            ("use".to_string(), ZoneKind::Button),
            ("dash".to_string(), ZoneKind::Button),
            ("jump".to_string(), ZoneKind::Button),
        ],
        "the stick, then the buttons toward the end corner"
    );
    let (stick, jump) = (&shown[0], &shown[3]);
    assert!(
        stick.x + stick.width < width / 2.0,
        "the stick is at the start"
    );
    assert!(shown[1].x > width / 2.0, "the buttons are at the end");
    assert!(
        jump.x + jump.width > width - 48.0 && jump.y + jump.height > height - 48.0,
        "the first button is in the corner: {jump:?}"
    );
    assert!(shown.iter().all(|z| z.x >= 0.0 && z.x + z.width <= width));

    assert!(
        zones(false, none.clone()).is_empty(),
        "nothing under a mouse"
    );
    assert_eq!(
        zones(false, serde_json::json!({"always": true})).len(),
        4,
        "always shows them under a mouse"
    );
    assert!(
        zones(true, serde_json::json!({"disabled": true})).is_empty(),
        "disabled controls feed nothing"
    );
    assert_eq!(
        named(&zones(
            true,
            serde_json::json!({"pad": true, "actions": []})
        )),
        [("move".to_string(), ZoneKind::Pad)]
    );
    let twin = zones(
        true,
        serde_json::json!({"look": "look", "actions": [], "deadZone": 0.3}),
    );
    assert_eq!(
        named(&twin),
        [
            ("move".to_string(), ZoneKind::Stick),
            ("look".to_string(), ZoneKind::Stick)
        ]
    );
    assert!(twin.iter().all(|z| z.dead_zone == 0.3), "{twin:?}");
    assert!(twin[1].x > width / 2.0, "the second stick is at the end");
    // Five buttons: three in the corner's row, the rest in a row above.
    let many = docs.iter().find(|d| d.doc.name == "TouchControls").unwrap();
    let many = many.doc.fixtures.iter().find(|(n, _)| n == "many").unwrap();
    let rows = zones(true, serde_json::json!({"actions": many.1["actions"]}));
    let attack = rows.iter().find(|z| z.action == "attack").unwrap();
    let above = rows
        .iter()
        .filter(|z| z.kind == ZoneKind::Button && z.y + z.height <= attack.y)
        .count();
    assert_eq!((rows.len(), above), (6, 2), "{rows:?}");
}

/// The main menu on a screen too short for it (a short landscape phone,
/// a portrait one with the logo shown) keeps its first button where it
/// was and scrolls the rest into reach.
///
/// Ways it could fail: a column that only centres (`justify-content:
/// center`, no scroll), whose last buttons stay cut off at the bottom
/// edge whatever the player does.
#[test]
fn main_menu_scrolls_when_it_is_taller_than_the_screen() {
    let (docs, messages) = engine_documents(&["src/ui/components", "src/ui/screens"]);
    let doc = docs
        .iter()
        .find(|d| d.path == "src/ui/screens/main-menu.svelte")
        .expect("main-menu.svelte");
    for (w, h) in [(844.0, 240.0), (360.0, 480.0)] {
        let mut ui = ui_with_font(test_font_metrics());
        ui.set_icons(icons(og_draw::TextureHandle::from_slot(1)));
        preview(&mut ui, &docs, doc, &messages, w, h);
        let quit = |ui: &Ui| {
            ui.elements()
                .iter()
                .find(|e| e.path.ends_with("/Quit"))
                .map(|e| (e.y, e.height, e.clipped))
        };
        assert!(
            quit(&ui).is_none_or(|(_, _, clipped)| clipped),
            "{w}×{h}: the menu fits, so this checks nothing: {:?}",
            quit(&ui)
        );
        for _ in 0..30 {
            let wheel = InputFrame {
                pointer: og_core::glam::Vec2::new(w / 4.0, h / 2.0),
                pointer_inside: true,
                wheel: og_core::glam::Vec2::new(0.0, 120.0),
                ..InputFrame::default()
            };
            ui.begin(wheel, Viewport::new(w, h));
            ui.build_retained_documents(&messages).unwrap();
            ui.end();
        }
        let (y, height, clipped) = quit(&ui).expect("Quit reachable after scrolling");
        let body = ui
            .elements()
            .iter()
            .find(|e| e.id == "PreviewBody")
            .unwrap();
        assert!(
            !clipped && y >= body.y && y + height <= body.y + body.height,
            "{w}×{h}: Quit out of reach after scrolling: y {y}, height {height}"
        );
    }
}

/// An ability slot grows with the player's text scale, and its readouts
/// (the cost, the charges, an ultimate's percent) stay apart at normal and
/// largest text, so none hides another.
///
/// Ways it could fail: a slot kept at its 100 % size while its badges grow
/// (at 150 % a square slot's cost and charges met in the middle), or the
/// cost kept at the bottom corner, where an ultimate's percent also sits.
#[test]
fn an_ability_slot_grows_with_the_text_and_keeps_its_readouts_apart() {
    let (docs, messages) = engine_documents(&["src/ui/components"]);
    let doc = docs
        .iter()
        .find(|d| d.path == "src/ui/components/hud/ability-slot.svelte")
        .expect("ability-slot.svelte");
    let square = [
        ("cooldown", serde_json::json!(0)),
        ("charges", serde_json::json!(2)),
        ("maxCharges", serde_json::json!(3)),
        ("cost", serde_json::json!("40")),
    ];
    let ultimate = [
        ("cooldown", serde_json::json!(0)),
        ("maxCharges", serde_json::json!(0)),
        ("cost", serde_json::json!("40")),
        ("ultimate", serde_json::json!(0.72)),
    ];
    for (name, set, size, readouts) in [
        ("square", &square, 48.0, &["Cost", "Charges"][..]),
        ("ultimate", &ultimate, 64.0, &["Cost", "Percent"][..]),
    ] {
        let mut props = doc.props.clone();
        let p = props.as_object_mut().unwrap();
        for (k, v) in set.iter() {
            p.insert((*k).into(), v.clone());
        }
        let slot = EngineDocument {
            path: doc.path.clone(),
            doc: doc.doc.clone(),
            props,
        };
        for scale in [1.0, 1.5] {
            let mut ui = ui_with_font(test_font_metrics());
            ui.set_icons(icons(og_draw::TextureHandle::from_slot(1)));
            let mut theme = *ui.theme();
            theme.text_scale = scale;
            ui.set_theme(theme);
            preview(&mut ui, &docs, &slot, &messages, 1280.0, 720.0);
            let rect = |id: &str| {
                let e = ui
                    .elements()
                    .iter()
                    .find(|e| e.path.ends_with(&format!("/{id}")))
                    .unwrap_or_else(|| panic!("{name} {scale}: no {id}"));
                (e.x, e.y, e.width, e.height)
            };
            let s = rect("Slot");
            assert!(
                (s.2 - size * scale).abs() < 0.5 && (s.3 - size * scale).abs() < 0.5,
                "{name} {scale}: slot {s:?}, not {}",
                size * scale
            );
            let (a, b) = (rect(readouts[0]), rect(readouts[1]));
            let apart =
                a.0 + a.2 <= b.0 || b.0 + b.2 <= a.0 || a.1 + a.3 <= b.1 || b.1 + b.3 <= a.1;
            assert!(
                apart,
                "{name} {scale}: {} {a:?} overlaps {} {b:?}",
                readouts[0], readouts[1]
            );
        }
    }
}

/// The default style marks a table's current row: the arena scoreboard
/// fills the local player's row (`aria-current`) with the chosen wash
/// (`--chosen-fill`), whatever corner shape a style gives rows. The teams
/// may sit side by side and nonlocal rows have their own fills. Ways it could
/// fail: a role's `corner-shape: round` beside its radius taken for a
/// CSS-corner box, or a table's rows (which the table draws, not its row
/// groups) left out of the tree's orders, so a shaped row's fill, drawn
/// with its corners, is never drawn.
#[test]
fn the_scoreboard_marks_the_local_players_row() {
    let (docs, messages) = engine_documents(&["src/ui/components"]);
    let doc = docs
        .iter()
        .find(|d| d.path.ends_with("arena/scoreboard.svelte"))
        .expect("the arena scoreboard");
    // Mark a player on either team, and no player. A generic row wash
    // must not count as the local highlight in any of those cases.
    for local in ["you", "rook", ""] {
        let mut props = doc.props.clone();
        for row in props["rows"].as_array_mut().unwrap() {
            row["local"] = serde_json::json!(row["id"] == local);
        }
        let doc = EngineDocument {
            path: doc.path.clone(),
            doc: doc.doc.clone(),
            props,
        };
        for shape in ["round", "bevel", "scoop", "notch", "squircle", "square"] {
            let mut ui = ui_with_font(test_font_metrics());
            ui.set_part_css(&format!(
                "{}\n:root {{ --row-corner-shape: {shape}; --row-radius: 6px; }}",
                og_ui::DEFAULT_CSS
            ))
            .unwrap();
            preview(&mut ui, &docs, &doc, &messages, 1280.0, 720.0);
            let rows = ui
                .frame()
                .items
                .iter()
                .filter(|i| {
                    let DrawCmd::Shape { shape, .. } = &i.cmd else {
                        return false;
                    };
                    let stops = match &shape.paint {
                        og_draw::UiPaint::Gradient(g) => &g.stops,
                        og_draw::UiPaint::GradientImage(g) => &g.gradient.stops,
                        _ => return false,
                    };
                    // The default local row wears the chosen wash: it starts
                    // at the families' cyan light at 24 %; a team's row wash
                    // starts at its team colour at 16 %. Check its drawn
                    // fill, not a style flag.
                    shape.mode == og_draw::UiMode::Fill
                        && shape.opacity > 0.0
                        && shape.half.x > 0.0
                        && shape.half.y > 0.0
                        && stops.first().is_some_and(|s| {
                            let [r, _, b, a] = s.color;
                            (a - 0.24).abs() < 0.005 && b - r > 0.3
                        })
                })
                .count();
            assert_eq!(
                rows,
                usize::from(!local.is_empty()),
                "{shape}, local={local}: local highlight count"
            );
        }
    }
}

/// A status effect refreshed past its duration (more seconds left than it
/// first had) sweeps nothing. Fails when the sweep's elapsed share is read
/// unclamped, a negative dash the frame refuses.
#[test]
fn status_grid_takes_an_effect_extended_past_its_duration() {
    let (docs, messages) = engine_documents(&["src/ui/components"]);
    let grid = docs.iter().find(|d| d.doc.name == "StatusGrid").unwrap();
    let mut props = grid.props.clone();
    props["debuffs"][0]["remaining"] = serde_json::json!(3599);
    let extended = EngineDocument {
        path: grid.path.clone(),
        doc: grid.doc.clone(),
        props,
    };
    let mut ui = ui_with_font(test_font_metrics());
    ui.set_icons(icons(og_draw::TextureHandle::from_slot(1)));
    let warnings = preview(&mut ui, &docs, &extended, &messages, 1280.0, 720.0);
    assert!(warnings.is_empty(), "{warnings:?}");
}

/// Two interaction prompts over one spot (a chest beside a locked vault)
/// stand apart, the nearer in its place. Fails when the prompt's anchor
/// does not declutter and one prompt is drawn over the other.
#[test]
fn interaction_prompts_over_one_spot_stand_apart() {
    let (docs, messages) = engine_documents(&["src/ui/components"]);
    let page = r#"<meta name="component" content="page">
<div id="Page">
  <InteractionPrompt at="0 0 0" height={1.2} verb="Open" name="Vault" disabled={true} reason="Locked" maxDistance={20} />
  <InteractionPrompt at="0.3 0 0.5" height={1.2} verb="Pick up and equip" name="Ember Blade of the Ninefold Gate" maxDistance={20} />
</div>"#;
    let page = EngineDocument {
        path: "page of prompts".into(),
        doc: UiDocument::parse("page.svelte", page).unwrap(),
        props: serde_json::json!({}),
    };
    let mut ui = ui_with_font(test_font_metrics());
    ui.set_icons(icons(og_draw::TextureHandle::from_slot(1)));
    ui.define_document(page.doc.clone());
    let warnings = preview(&mut ui, &docs, &page, &messages, 1280.0, 720.0);
    assert!(warnings.is_empty(), "{warnings:?}");
    // Shifts glide in over a few ticks.
    for _ in 0..30 {
        ui.begin(InputFrame::default(), Viewport::new(1280.0, 720.0));
        ui.set_world_view(Some(view(1280.0, 720.0)));
        ui.build_retained_documents(&messages).unwrap();
        ui.end();
    }
    let prompts: Vec<_> = ui
        .probe_elements()
        .into_iter()
        .filter(|e| e.id == "Prompt")
        .collect();
    assert_eq!(prompts.len(), 2, "{prompts:#?}");
    let (a, b) = (&prompts[0], &prompts[1]);
    let apart = a.x + a.width <= b.x
        || b.x + b.width <= a.x
        || a.y + a.height <= b.y
        || b.y + b.height <= a.y;
    assert!(apart, "one prompt over the other: {a:#?} {b:#?}");
}

/// A pack's tokens reach the parts that read them: the tag tokens the
/// text a status grid draws over the scene, the HUD's stroke the text a
/// nameplate and a waypoint draw over it, the elite chip's fill, the
/// waypoint's distance tag, `--world-cast-ink` a nameplate's spell. Fails when a component
/// writes its own value over the token (the status grid's figures kept
/// their own halo whatever a pack set).
#[test]
fn packs_dress_text_over_the_scene_through_its_tokens() {
    let (docs, messages) = engine_documents(&["src/ui/components"]);
    let drawn = |name: &str, root: String| {
        let mut ui = ui_with_font(test_font());
        ui.set_icons(icons(og_draw::TextureHandle::from_slot(1)));
        let css = format!(":root {{ {root} }}");
        let pack = og_ui::style_pack::StylePack::parse(
            "styles/test",
            vec![("styles/test/main.css".into(), css)],
        );
        ui.set_style_pack(&pack.unwrap()).unwrap();
        let doc = docs.iter().find(|d| d.doc.name == name).unwrap();
        let warnings = preview(&mut ui, &docs, doc, &messages, 1280.0, 720.0);
        assert!(warnings.is_empty(), "{name}: {warnings:?}");
        format!("{:?}", ui.frame().items)
    };
    for (name, token, a, b) in [
        ("StatusGrid", "--tag-halo", "none", "0 0 4px #ff0000"),
        ("StatusGrid", "--tag-fill", "none", "#ff0000"),
        ("Nameplate", "--hud-stroke-color", "#000000", "#ff0000"),
        ("Nameplate", "--world-elite-chip-fill", "none", "#ff0000"),
        ("Nameplate", "--world-cast-ink", "#000000", "#00ff00"),
        ("Waypoint", "--hud-stroke-color", "#000000", "#ff0000"),
        ("Waypoint", "--hud-tag-fill", "none", "#ff0000"),
    ] {
        assert_ne!(
            drawn(name, format!("{token}: {a};")),
            drawn(name, format!("{token}: {b};")),
            "{name} draws the same whatever {token} is"
        );
    }
}

/// The minimap's north mark stands on the map inside the bezel, never on
/// the bezel's rim. Fails when it is lifted over the bezel's top edge.
#[test]
fn the_minimap_north_mark_stands_clear_of_the_bezel() {
    let (docs, messages) = engine_documents(&["src/ui/components"]);
    let doc = docs.iter().find(|d| d.doc.name == "MinimapView").unwrap();
    let mut ui = ui_with_font(test_font_metrics());
    ui.set_icons(icons(og_draw::TextureHandle::from_slot(1)));
    preview(&mut ui, &docs, doc, &messages, 1280.0, 720.0);
    let at = |id: &str| {
        ui.probe_elements()
            .into_iter()
            .find(|e| e.id == id)
            .unwrap_or_else(|| panic!("no {id}"))
    };
    let (north, glass) = (at("North"), at("Glass"));
    assert!(
        north.y >= glass.y,
        "north {} over the bezel above the glass {}",
        north.y,
        glass.y
    );
}

/// A node graph's well stretches over its container, and the graph stands
/// in its middle, scaled to fit, as an SVG viewBox's xMidYMid meet does. Ways
/// it could fail: the canvas pinned at the well's top-left (the map in a
/// corner of an empty panel), the well kept at a fixed size, or the graph
/// kept at its view's size in a larger well.
#[test]
fn a_node_graph_centres_its_graph_in_its_well() {
    let (docs, messages) = engine_documents(&["src/ui/components"]);
    let doc = docs
        .iter()
        .find(|d| d.path.ends_with("/node-graph.svelte"))
        .expect("the node graph");
    let mut ui = ui_with_font(test_font_metrics());
    preview(&mut ui, &docs, doc, &messages, 1280.0, 720.0);
    let elements = ui.probe_elements();
    let well = elements
        .iter()
        .find(|e| e.name == "Graph")
        .expect("the well");
    assert!(well.width > 1000.0, "the well stretches: {}", well.width);
    let nodes: Vec<_> = elements
        .iter()
        .filter(|e| {
            ["Strength", "Cleave", "Guard", "Rage"].contains(&e.name.as_str()) && e.kind == "button"
        })
        .collect();
    assert_eq!(nodes.len(), 4);
    let span = |lo: f32, hi: f32| (lo + hi) / 2.0;
    let x = span(
        nodes.iter().map(|e| e.x).fold(f32::MAX, f32::min),
        nodes.iter().map(|e| e.x + e.width).fold(f32::MIN, f32::max),
    );
    let y = span(
        nodes.iter().map(|e| e.y).fold(f32::MAX, f32::min),
        nodes
            .iter()
            .map(|e| e.y + e.height)
            .fold(f32::MIN, f32::max),
    );
    let (cx, cy) = (well.x + well.width / 2.0, well.y + well.height / 2.0);
    assert!((x - cx).abs() < 40.0, "across: graph {x}, well {cx}");
    assert!((y - cy).abs() < 40.0, "down: graph {y}, well {cy}");
    // It fits as xMidYMid meet: the fixture's 360 by 240 view scaled by the
    // most that holds it, with a node's width (48) to spare at each edge.
    // Strength stands at x 40 and Rage at x 320.
    let fit = ((well.width - 96.0) / 360.0).min((well.height - 96.0) / 240.0);
    let at = |name: &str| {
        let e = nodes.iter().find(|e| e.name == name).unwrap();
        e.x + e.width / 2.0
    };
    let span = at("Rage") - at("Strength");
    assert!(
        (span - 280.0 * fit).abs() < 2.0,
        "Strength to Rage {span}, the view fitted {}",
        280.0 * fit
    );
}

/// Calibration's controller step sets its dead-zone slider beside its
/// label, and the slider ends inside the row on a phone and a desktop.
///
/// Ways it could fail: the slider sized `width: stretch`, CSS's
/// stretch-fit size, which is the whole row, so it runs past the row's end
/// by the label and the gap (the controller step is in no fixture, so the
/// reference frames never lay it out).
#[test]
fn the_dead_zone_slider_shares_its_row_with_its_label() {
    let (docs, messages) = engine_documents(&["src/ui/components", "src/ui/screens"]);
    let doc = docs
        .iter()
        .find(|d| d.path == "src/ui/screens/calibration.svelte")
        .expect("calibration.svelte");
    let mut props = doc.props.clone();
    props["step"] = serde_json::json!("controller");
    let doc = EngineDocument {
        path: doc.path.clone(),
        doc: doc.doc.clone(),
        props,
    };
    for (w, h) in [(390.0, 844.0), (1280.0, 720.0)] {
        let mut ui = ui_with_font(test_font_metrics());
        ui.set_icons(icons(og_draw::TextureHandle::from_slot(1)));
        preview(&mut ui, &docs, &doc, &messages, w, h);
        let at = |id: &str| {
            ui.probe_elements()
                .into_iter()
                .find(|e| e.id == id)
                .unwrap_or_else(|| panic!("no {id}"))
        };
        let (row, slider) = (at("DeadZone"), at("DeadZoneValue"));
        assert!(
            slider.width > 0.0 && slider.x + slider.width <= row.x + row.width + 0.5,
            "{w}×{h}: the slider [{}, {}] runs past its row [{}, {}]",
            slider.x,
            slider.x + slider.width,
            row.x,
            row.x + row.width
        );
    }
}

/// A component's own `<style>` lays it out; its look comes from the roles
/// (docs/ui.md §3.1 Role parts). A look property (a fill, a rim, a radius,
/// a shadow, an ink, type, a filter) in a component's rule may only read a
/// role's token (`var(--card-fill)`) or reset (`none`, `0`, `inherit`),
/// so a style pack's tokens reach every component. Documents written
/// before the rule carry a count in `component_look_backlog.tsv`, which
/// only shrinks: a converted document lowers or drops its line.
///
/// Ways it could fail: a document that paints a box itself
/// (`border-radius: var(--radius-card)`, `color: var(--ink-muted)`)
/// instead of naming its part for the role that does.
#[test]
fn component_styles_lay_out_and_roles_dress() {
    let roles: std::collections::BTreeSet<String> = {
        let css = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../packages/design/tokens/role.css"),
        )
        .unwrap();
        css.split("--role-")
            .skip(1)
            .filter_map(|s| s.split(':').next())
            .filter(|n| n.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'))
            .map(str::to_string)
            .collect()
    };
    let backlog_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/component_look_backlog.tsv");
    let backlog: std::collections::BTreeMap<String, usize> = std::fs::read_to_string(&backlog_path)
        .unwrap()
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            let (path, n) = l.split_once('\t').expect("path<TAB>count");
            (path.to_string(), n.parse().unwrap())
        })
        .collect();
    let (docs, _) = engine_documents(&["src/ui/components"]);
    let mut counts = std::collections::BTreeMap::new();
    let mut examples = std::collections::BTreeMap::new();
    for doc in &docs {
        let path = doc.path["src/ui/components/".len()..].to_string();
        let found = look_declarations(&doc.doc.text, &roles);
        if !found.is_empty() {
            examples.insert(path.clone(), found[0].clone());
            counts.insert(path, found.len());
        }
    }
    let mut failures = Vec::new();
    for (path, n) in &counts {
        let allowed = backlog.get(path).copied().unwrap_or(0);
        if *n > allowed {
            failures.push(format!(
                "{path}: {n} look declarations outside the roles (backlog {allowed}), as `{}`",
                examples[path]
            ));
        }
    }
    for (path, allowed) in &backlog {
        let n = counts.get(path).copied().unwrap_or(0);
        if n < *allowed {
            failures.push(format!(
                "{path}: now {n}; lower its backlog line from {allowed}"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The look and motion declarations of a document's `<style>` that read
/// no role token, as `property: value`, and each `@keyframes` it defines
/// (the shared motions are the style's). Custom properties are tokens and
/// do not count.
fn look_declarations(source: &str, roles: &std::collections::BTreeSet<String>) -> Vec<String> {
    const LOOK: &[&str] = &[
        "background",
        "border",
        "outline",
        "box-shadow",
        "color",
        "font",
        "letter-spacing",
        "text-transform",
        "text-shadow",
        "text-decoration",
        "filter",
        "backdrop-filter",
        "corner-shape",
        "animation",
    ];
    const RESETS: &[&str] = &[
        "none",
        "0",
        "0px",
        "inherit",
        "unset",
        "initial",
        "revert-layer",
        "currentColor",
        "transparent",
        "normal",
    ];
    let Some(start) = source.find("<style") else {
        return Vec::new();
    };
    let css = &source[start..];
    let css = &css[css.find('>').unwrap() + 1..css.find("</style>").unwrap_or(css.len())];
    // Comments out, then each `@keyframes` block.
    let mut text = String::new();
    let mut rest = css;
    while let Some(i) = rest.find("/*") {
        text.push_str(&rest[..i]);
        rest = rest[i..].find("*/").map_or("", |j| &rest[i + j + 2..]);
    }
    text.push_str(rest);
    let mut found = Vec::new();
    while let Some(i) = text.find("@keyframes") {
        let open = i + text[i..].find('{').unwrap();
        let (mut depth, mut end) = (0, open);
        for (k, c) in text[open..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = open + k + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        found.push(text[i..open].trim().to_string());
        text.replace_range(i..end, "");
    }
    let role_var = |v: &str| {
        let v = v.trim();
        v.strip_prefix("var(--")
            .and_then(|r| r.strip_suffix(')'))
            .is_some_and(|name| roles.contains(name))
    };
    for body in text.split('{').skip(1).filter_map(|b| b.split('}').next()) {
        for decl in body.split(';') {
            let Some((property, value)) = decl.split_once(':') else {
                continue;
            };
            let (property, value) = (property.trim(), value.trim());
            let look = LOOK
                .iter()
                .any(|l| property == *l || property.starts_with(&format!("{l}-")));
            if !look || matches!(property, "border-collapse" | "border-spacing") {
                continue;
            }
            if RESETS.contains(&value) || role_var(value) {
                continue;
            }
            found.push(format!("{property}: {value}"));
        }
    }
    found
}

/// Only the final name in a part list wears its role. Keep existing
/// ambiguities in a shrinking backlog while the documents are redesigned;
/// new ones fail, and removed ones must lose their exception.
#[test]
fn component_parts_wear_at_most_one_role() {
    let rule = regex_lite::Regex::new(r#"part\$="\.([a-z-]+)""#).unwrap();
    let roles: std::collections::BTreeSet<String> = rule
        .captures_iter(og_ui::DEFAULT_CSS)
        .map(|c| c[1].to_string())
        .collect();
    let backlog = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/component_role_backlog.tsv"),
    )
    .unwrap();
    let mut allowed = std::collections::BTreeMap::new();
    for line in backlog
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let columns: Vec<_> = line.split('\t').collect();
        assert_eq!(columns.len(), 3, "path<TAB>part list<TAB>count");
        let key = (columns[0].to_string(), columns[1].to_string());
        assert!(
            allowed
                .insert(key, columns[2].parse::<usize>().unwrap())
                .is_none()
        );
    }
    let (docs, _) = engine_documents(&["src/ui/components"]);
    let mut found = std::collections::BTreeMap::new();
    for doc in &docs {
        for node in &doc.doc.nodes {
            let Some(part) = node.part_name() else {
                continue;
            };
            let count = part
                .split_ascii_whitespace()
                .filter(|name| {
                    name.rsplit_once('.')
                        .is_some_and(|(_, role)| roles.contains(role))
                })
                .count();
            if count > 1 {
                let path = doc.path.strip_prefix("src/ui/components/").unwrap();
                *found
                    .entry((path.to_string(), part.to_string()))
                    .or_insert(0usize) += 1;
            }
        }
    }
    let mut problems = Vec::new();
    for ((path, part), count) in &found {
        let limit = allowed
            .get(&(path.clone(), part.clone()))
            .copied()
            .unwrap_or(0);
        if *count > limit {
            problems.push(format!("{path}: `{part}` lists multiple roles ({count} occurrences, backlog {limit}); keep one role"));
        }
    }
    for ((path, part), limit) in &allowed {
        let count = found
            .get(&(path.clone(), part.clone()))
            .copied()
            .unwrap_or(0);
        if count < *limit {
            problems.push(format!("{path}: `{part}` now has {count} occurrences; lower or remove its role backlog line ({limit})"));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// A role dresses every built-in part whose last name ends in it, so a new
/// role silently restyles parts named before it existed (a chart's text
/// `tick` would have taken a box's fill from a meter's `tick` role). Each
/// role in the default style is listed in `role_names.tsv` once the parts it
/// dresses were checked to be meant for it; an unlisted role fails with the
/// parts it would dress, and a listed role the style no longer has fails as
/// stale. The default style's `data-` attributes must not be ones og-ui
/// reads with a meaning of its own. Wrong: a role added to the default
/// style that no review saw, or a style attribute that collides.
#[test]
fn component_parts_meant_for_a_role() {
    let rule = regex_lite::Regex::new(r#"part\$="\.([a-z-]+)""#).unwrap();
    let roles: std::collections::BTreeSet<String> = rule
        .captures_iter(og_ui::DEFAULT_CSS)
        .map(|c| c[1].to_string())
        .collect();
    let listed: std::collections::BTreeSet<String> = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/role_names.tsv"),
    )
    .unwrap()
    .lines()
    .filter(|l| !l.is_empty() && !l.starts_with('#'))
    .map(|l| l.split('\t').next().unwrap().to_string())
    .collect();
    let parts_line = regex_lite::Regex::new(r#"<meta name="parts" content="([^"]*)""#).unwrap();
    let (docs, _) = engine_documents(&["src/ui/components"]);
    let mut wearers = std::collections::BTreeMap::<String, Vec<String>>::new();
    for doc in &docs {
        for line in parts_line.captures_iter(&doc.doc.text) {
            for part in line[1].split_whitespace() {
                if let Some((_, suffix)) = part.rsplit_once('.') {
                    wearers
                        .entry(suffix.to_string())
                        .or_default()
                        .push(part.to_string());
                }
            }
        }
    }
    let mut problems = Vec::new();
    for role in roles.difference(&listed) {
        problems.push(format!(
            "role `{role}` is not in tests/role_names.tsv; it dresses {:?}: list it once each is meant to wear it, or rename the role",
            wearers.get(role).cloned().unwrap_or_default()
        ));
    }
    for role in listed.difference(&roles) {
        problems.push(format!(
            "tests/role_names.tsv lists `{role}`, which the default style no longer has"
        ));
    }
    // The same for the attributes the default style keys on: one og-ui
    // already reads with a meaning of its own (a sheet's `data-edge`) is
    // refused there with another's values. These three are og-ui's own
    // states, which the style dresses on purpose.
    let own: std::collections::BTreeSet<String> = {
        let literal = regex_lite::Regex::new(r#""(data-[a-z-]+)"#).unwrap();
        let mut found = std::collections::BTreeSet::new();
        let mut dirs = vec![std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src")];
        while let Some(dir) = dirs.pop() {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    dirs.push(path);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    let text = std::fs::read_to_string(&path).unwrap();
                    found.extend(literal.captures_iter(&text).map(|c| c[1].to_string()));
                }
            }
        }
        found
    };
    let keyed = regex_lite::Regex::new(r"\[(data-[a-z-]+)").unwrap();
    for attribute in keyed
        .captures_iter(og_ui::DEFAULT_CSS)
        .map(|c| c[1].to_string())
        .collect::<std::collections::BTreeSet<_>>()
    {
        if own.contains(&attribute)
            && !["data-outline", "data-player", "data-scheme"].contains(&attribute.as_str())
        {
            problems.push(format!(
                "the default style keys on `{attribute}`, which og-ui already reads with its own meaning: rename the style's"
            ));
        }
    }
    assert!(problems.is_empty(), "{problems:#?}");
}
