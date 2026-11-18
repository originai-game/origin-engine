//! Public document fixtures must exercise their content, and size-driven
//! components must keep that content inside the host supplied by the game.
//! These are CPU layout/probe checks; no renderer or reference images.

use crate::support::*;

#[test]
fn settings_default_fixture_exercises_all_entry_controls() {
    let (docs, messages) = engine_documents(&["src/ui/screens"]);
    let doc = docs
        .iter()
        .find(|d| d.path.ends_with("/settings.svelte"))
        .unwrap();
    let page = doc.props["page"].as_str().unwrap_or("");
    assert!(
        !page.is_empty(),
        "settings fixture needs an active page, not only its header"
    );
    let entries = doc.props["entries"]
        .as_array()
        .expect("settings fixture entries");
    for kind in ["toggle", "range", "select", "binding"] {
        assert!(
            entries
                .iter()
                .any(|e| e["page"] == page && e["kind"] == kind),
            "missing {kind} on active page"
        );
    }
    for (w, h) in [(1280.0, 720.0), (390.0, 844.0)] {
        let mut ui = ui_with_font(test_font_metrics());
        assert!(preview(&mut ui, &docs, doc, &messages, w, h).is_empty());
        let elements = ui.probe_elements();
        let choice = elements
            .iter()
            .find(|e| e.path.ends_with("/Choice/Value"))
            .unwrap();
        assert_eq!(
            choice.text, "High",
            "the fixture must show a selected label"
        );
        for entry in entries.iter().filter(|e| e["page"] == page) {
            let name = entry["name"].as_str().unwrap();
            assert!(
                elements.iter().any(|e| e.text == name && !e.clipped),
                "{w}: missing visible {name}"
            );
        }
        for id in [
            "Toggle", "Range", "Choice", "Binding", "Apply", "Revert", "Defaults",
        ] {
            assert!(
                elements
                    .iter()
                    .any(|e| (e.id == id || e.id.starts_with(&format!("{id}#"))) && !e.clipped),
                "{w}: missing visible {id}"
            );
        }
    }
}

#[test]
fn card_hand_fixtures_keep_transformed_cards_inside_the_stage() {
    let (docs, messages) = engine_documents(&["src/ui/components"]);
    let doc = docs
        .iter()
        .find(|d| d.path.ends_with("card/hand.svelte"))
        .unwrap();
    let mut failures = Vec::new();
    for (fixture, props) in &doc.doc.fixtures {
        for (w, h) in [(1280.0, 720.0), (390.0, 844.0), (800.0, 360.0)] {
            let candidate = EngineDocument {
                path: doc.path.clone(),
                doc: doc.doc.clone(),
                props: props.clone(),
            };
            let mut ui = ui_with_font(test_font_metrics());
            assert!(preview(&mut ui, &docs, &candidate, &messages, w, h).is_empty());
            for e in ui.probe_elements().iter().filter(|e| e.role == "option") {
                if e.clipped
                    || e.x < 23.9
                    || e.y < 23.9
                    || e.x + e.width > w - 23.9
                    || e.y + e.height > h - 23.9
                {
                    failures.push(format!(
                        "{fixture} {w}x{h}: {} bounds [{}, {}, {}, {}] escape the 24px stage",
                        e.path, e.x, e.y, e.width, e.height
                    ));
                }
            }
            assert_eq!(
                ui.probe_elements()
                    .iter()
                    .filter(|e| e.role == "option")
                    .count(),
                props["cards"].as_array().unwrap().len()
            );
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn enemy_without_a_portrait_does_not_reserve_a_picture_and_keeps_health_visible() {
    let (docs, messages) = engine_documents(&["src/ui/components"]);
    let doc = docs
        .iter()
        .find(|d| d.path.ends_with("rpg/enemy-info.svelte"))
        .unwrap();
    assert_eq!(doc.props["portrait"], 0);
    for (w, h) in [(1280.0, 720.0), (390.0, 844.0), (800.0, 600.0)] {
        let mut ui = ui_with_font(test_font_metrics());
        assert!(preview(&mut ui, &docs, doc, &messages, w, h).is_empty());
        let elements = ui.probe_elements();
        let health = elements
            .iter()
            .find(|e| e.role == "meter")
            .expect("health meter");
        assert!(
            !health.clipped && health.height >= 24.0 && health.y + health.height <= h - 24.0,
            "{w}: health must fit the host: {health:?}"
        );
        let name = elements
            .iter()
            .find(|e| e.text == doc.props["name"].as_str().unwrap())
            .unwrap();
        assert!(
            health.y - (name.y + name.height) <= ui.theme().space.xxl * 3.0,
            "{w}: portrait=0 must not reserve an empty portrait body: name bottom {}, health top {}",
            name.y + name.height,
            health.y
        );
        for effect in doc.props["effects"].as_array().unwrap() {
            let label = effect["label"].as_str().unwrap();
            assert!(
                elements
                    .iter()
                    .any(|e| e.text == label && !e.clipped && e.y + e.height <= h - 24.0),
                "{w}: missing visible {label}"
            );
        }
    }
}

#[test]
fn supplied_enemy_portrait_is_bounded_and_does_not_push_health_offstage() {
    use og_ui::*;

    struct PortraitEnv<'a> {
        messages: &'a Messages,
        texture: og_draw::TextureHandle,
    }
    impl bind::Env for PortraitEnv<'_> {
        fn translate(&self, key: &str) -> Option<String> {
            self.messages.translate(key)
        }
        fn texture(&self, handle: f64) -> Option<og_draw::TextureHandle> {
            (handle == 1.0).then_some(self.texture)
        }
    }

    let (docs, messages) = engine_documents(&["src/ui/components"]);
    let doc = docs
        .iter()
        .find(|d| d.path.ends_with("rpg/enemy-info.svelte"))
        .unwrap();
    let mut candidate = EngineDocument {
        path: doc.path.clone(),
        doc: doc.doc.clone(),
        props: doc.props.clone(),
    };
    candidate.props["portrait"] = serde_json::json!(1);
    let env = PortraitEnv {
        messages: &messages,
        texture: Handles::default().texture(),
    };
    for (w, h) in [
        (1280.0, 720.0),
        (390.0, 844.0),
        (800.0, 600.0),
        (800.0, 500.0),
    ] {
        let mut ui = ui_with_font(test_font_metrics());
        assert!(preview(&mut ui, &docs, &candidate, &env, w, h).is_empty());
        let elements = ui.probe_elements();
        let health = elements.iter().find(|e| e.role == "meter").unwrap();
        assert!(
            !health.clipped && health.height >= 24.0 && health.y + health.height <= h - 24.0,
            "{w}: the portrait must leave health inside the host: {health:?}"
        );
        for effect in candidate.props["effects"].as_array().unwrap() {
            let label = effect["label"].as_str().unwrap();
            assert!(
                elements
                    .iter()
                    .any(|e| e.text == label && !e.clipped && e.y + e.height <= h - 24.0),
                "{w}: missing visible {label}"
            );
        }
        let portrait = elements.iter().find(|e| e.kind == "image");
        if w >= 600.0 && h >= 500.0 {
            let portrait = portrait.expect("a supplied portrait must remain visible on desktop");
            assert!(
                portrait.width > 0.0
                    && portrait.height > 0.0
                    && portrait.width <= 192.1
                    && portrait.height <= 192.1,
                "{w}: portrait must keep its art-size cap: {portrait:?}"
            );
            assert!(ui.frame().items.iter().any(|item| matches!(&item.cmd, DrawCmd::Shape { shape, .. } if matches!(&shape.paint, og_draw::UiPaint::Image(image) if image.texture == env.texture))), "supplied art must reach paint without a renderer");
        }
    }
}
