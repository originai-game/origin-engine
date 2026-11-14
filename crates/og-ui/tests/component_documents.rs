//! The engine's component documents (`packages/engine/src/ui/components`):
//! each parses, takes its strings and builds from its `fixture:default`
//! props at a desktop and a phone width with no author or frame warning.
//! A document that stops cooking, names a missing string, or builds a
//! tree the frame refuses fails here before a game ejects it.

use crate::support;
use og_ui::*;
use support::*;

#[test]
fn component_documents_build_from_their_fixtures() {
    let (docs, messages) = engine_documents(&["src/ui/components"]);
    assert!(!docs.is_empty());
    let mut failures = Vec::new();
    // A group's documents (rpg/, shooter/) mount from their own fixtures,
    // whose art is their sheet's, in packages/engine/tests/<group>.test.ts.
    let top = |d: &&EngineDocument| !d.path["src/ui/components/".len()..].contains('/');
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
            !ui.probe_elements().is_empty(),
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

/// The ground marks drape over the scene (c8e4b7a8c): each document's
/// `<ground>` reaches the renderer as one layer carrying its drape. Fails
/// when a mark is rebuilt as a flat panel or a screen-space box.
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
        let grounds = overlay
            .ui
            .layers
            .iter()
            .filter(|l| l.ground.is_some())
            .count();
        assert_eq!(grounds, 1, "{name}: {grounds} draped ground layer(s)");
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
