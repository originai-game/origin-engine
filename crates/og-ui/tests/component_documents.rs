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
