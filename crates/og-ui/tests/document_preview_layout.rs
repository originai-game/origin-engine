//! Public document fixtures must exercise their content, and size-driven
//! components must keep that content inside the host supplied by the game.
//! These are CPU layout/probe checks; no renderer or reference images.

use crate::support::*;

#[test]
fn world_components_clear_player_and_telegraph_without_duplicate_arrows() {
    use og_ui::*;
    let (docs, messages) = engine_documents(&["src/ui/components/world"]);
    let host = UiDocument::parse("placement.svelte", r#"<meta name="component" content="Placement" />
<div>
  <anchor id="Player" at="0 -1 0" reserve="player" reserve-radius="0.8" reserve-height="2" />
  <Waypoint id="Waypoint" at="-8 0 0" height={0} label="Brannoch" distance="86" units="m" edge="left" />
  <Telegraph id="Telegraph" at="0.6 -1 -3" radius={1.5} progress={0.4} angle={90} width={0.5} inner={0} direction={0} />
  <InteractionPrompt id="InteractionPrompt" at="0.6 -1 -3" height={0} key="E" verb="Open" name="Chest" hold={-1} maxDistance={80} />
</div>"#).unwrap();
    let host = EngineDocument {
        path: "placement.svelte".into(),
        doc: host,
        props: serde_json::json!({}),
    };
    let mut all = docs;
    all.push(host);
    let mut ui = ui_with_font(test_font());
    for doc in &all {
        ui.define_document(doc.doc.clone());
    }
    ui.mount_with_values(
        reactive::SourceId::new("test", "placement"),
        "Placement",
        &serde_json::json!({}),
    )
    .unwrap();
    for _ in 0..60 {
        ui.set_world_view(Some(WorldView::look_at(
            og_core::glam::Vec3::new(0.0, 0.0, 6.0),
            og_core::glam::Vec3::ZERO,
            og_core::glam::Vec3::Y,
            50.0,
            390.0 / 844.0,
            0.1,
            100.0,
        )));
        ui.begin(InputFrame::default(), Viewport::new(390.0, 844.0));
        ui.build_retained_documents(&messages).unwrap();
        ui.end();
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let elements = ui.probe_elements();
    let get = |id: &str| {
        elements
            .iter()
            .find(|e| e.id == id || e.path.ends_with(&format!("/{id}")))
            .unwrap_or_else(|| panic!("missing {id}: {elements:?}"))
    };
    let intersects = |a: &UiElement, b: &UiElement| {
        a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y
    };
    assert!(
        !intersects(get("Waypoint"), get("Player")),
        "waypoint covers player"
    );
    // At this level camera the ground's near rim is 6 + 3 - 1.5 m away;
    // the far rim is 10.5 m away. Include a pixel beyond the projected rim.
    let focal = 844.0 / (2.0 * 25.0_f32.to_radians().tan());
    let top = 422.0 + focal / 10.5 - 1.0;
    let bottom = 422.0 + focal / 7.5 + 1.0;
    let prompt = get("InteractionPrompt");
    assert!(
        prompt.y + prompt.height <= top || prompt.y >= bottom,
        "prompt covers danger rim: {prompt:?}"
    );
    assert!(
        elements.iter().any(|e| e
            .detail
            .get("part")
            .and_then(|p| p.as_str())
            .is_some_and(|p| p.contains("::part(Waypoint.chevron)"))),
        "component chevron is retained"
    );
    assert!(
        !ui.frame()
            .items
            .iter()
            .any(|i| matches!(&i.cmd, DrawCmd::Shape { shape, .. }
        if matches!(shape.geometry, og_draw::UiGeometry::Polygon { sides: 3, .. })
        && shape.paint == og_draw::UiPaint::Solid([1.0; 4]))),
        "native white arrow duplicates component chevron"
    );
}

#[test]
fn world_nameplate_scroll_reserves_the_badge_inside_the_phone_plate() {
    let (docs, messages) = engine_documents(&["src/ui/components/world"]);
    let source = docs.iter().find(|d| d.doc.name == "Nameplate").unwrap();
    for (w, h) in [(1280.0, 720.0), (390.0, 844.0)] {
        for scale in [1.0, 1.5] {
            let inset = 24.0 * scale - 1.0;
            let mut props = source.props.clone();
            props.as_object_mut().unwrap().extend(
                source
                    .doc
                    .fixtures
                    .iter()
                    .find(|(n, _)| n == "overflow")
                    .unwrap()
                    .1
                    .as_object()
                    .unwrap()
                    .clone(),
            );
            let doc = EngineDocument {
                path: source.path.clone(),
                doc: source.doc.clone(),
                props,
            };
            let mut ui = ui_with_font(test_font());
            ui.set_theme(og_ui::Theme {
                text_scale: scale,
                ..*ui.theme()
            });
            assert!(preview(&mut ui, &docs, &doc, &messages, w, h).is_empty());
            let elements = ui.probe_elements();
            let part = |name: &str| {
                elements
                    .iter()
                    .find(|e| {
                        e.detail
                            .get("part")
                            .and_then(|p| p.as_str())
                            .is_some_and(|p| p.contains(&format!("::part({name})")))
                    })
                    .unwrap()
            };
            let name = part("Nameplate.name");
            let badge = part("Nameplate.level");
            let plate = part("Nameplate.frame");
            assert_eq!(
                name.detail.get("textScrollActive"),
                Some(&serde_json::json!(true))
            );
            assert!(
                name.x >= badge.x + badge.width + 1.0,
                "badge overlaps name: {badge:?}, {name:?}"
            );
            assert!(name.x >= plate.x - 0.1 && name.x + name.width <= plate.x + plate.width + 0.1);
            assert!(
                !name.clipped && plate.x >= inset && plate.x + plate.width <= w - inset,
                "{w} scale {scale}: {plate:?}, {name:?}"
            );
            assert!(!part("Nameplate.time").clipped);
            assert!(
                ui.frame().items.iter().any(|item| matches!(
                    item.cmd,
                    og_ui::DrawCmd::BeginLayer {
                        mask: Some(og_draw::UiMask::Gradient(_)),
                        ..
                    }
                )),
                "long name keeps its edge fade"
            );
            let view = ui.world_view().unwrap();
            for dx in [-0.6, 0.6] {
                let eye = view.eye + og_core::glam::Vec3::X * dx;
                for _ in 0..40 {
                    ui.set_world_view(Some(og_ui::WorldView::look_at(
                        eye,
                        eye + view.forward,
                        og_core::glam::Vec3::Y,
                        50.0,
                        w / h,
                        0.1,
                        100.0,
                    )));
                    ui.begin(og_ui::InputFrame::default(), og_ui::Viewport::new(w, h));
                    ui.build_retained_documents(&messages).unwrap();
                    ui.end();
                }
                let placed = ui.element(&plate.path).unwrap();
                assert!(
                    !placed.clipped && placed.x >= inset && placed.x + placed.width <= w - inset,
                    "near-edge plate must be contained: {placed:?}"
                );
            }
        }
    }
}

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
