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
            let e = ui
                .elements()
                .iter()
                .find(|e| e.path.ends_with("/Quit"))
                .expect("Quit");
            (e.y, e.height, e.clipped)
        };
        assert!(
            quit(&ui).2,
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
        let (y, height, clipped) = quit(&ui);
        assert!(
            !clipped && y + height <= h,
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
/// fills the local player's row (`aria-current`) with the row role's
/// selected fill, whatever corner shape a style gives rows. Ways it could
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
    for shape in ["round", "bevel", "scoop", "notch", "squircle", "square"] {
        let mut ui = ui_with_font(test_font_metrics());
        ui.set_part_css(&format!(
            "{}\n:root {{ --row-corner-shape: {shape}; --row-radius: 6px; }}",
            og_ui::DEFAULT_CSS
        ))
        .unwrap();
        preview(&mut ui, &docs, doc, &messages, 1280.0, 720.0);
        let rows = ui
            .frame()
            .items
            .iter()
            .filter_map(|i| i.cmd.filled())
            .filter(|(r, c)| r.width > 1000.0 && (10.0..60.0).contains(&r.height) && c[3] > 0.0)
            .count();
        assert_eq!(rows, 1, "{shape}: one marked row");
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
/// text a status grid, a nameplate and a waypoint draw over the scene,
/// `--ink-on-warn` the spell over a cast's fill. Fails when a component
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
        ("Nameplate", "--tag-halo", "none", "0 0 4px #ff0000"),
        ("Nameplate", "--tag-fill", "none", "#ff0000"),
        ("Nameplate", "--ink-on-warn", "#000000", "#00ff00"),
        ("Waypoint", "--tag-halo", "none", "0 0 4px #ff0000"),
        ("Waypoint", "--tag-fill", "none", "#ff0000"),
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
