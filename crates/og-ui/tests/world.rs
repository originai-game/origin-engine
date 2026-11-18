//! World-space UI (`docs/ui.md` §1, `src/world.rs`), without a renderer:
//! anchored elements land on their projected points, scale with distance,
//! declutter by priority and clamp to the edge with an arrow; occlusion
//! asks the renderer for a depth test; mass elements pool, age out and
//! replay; a world panel takes a press through a ray; particles (and the
//! `burst` preset) emit seeded mass elements; meta layers cover the
//! screen; a viewport hands its drag and ray to the game.
//!
//! Ways it could fail: the anchor drawn at its layout's top left (the
//! projection not applied), its scale ignored, crowded tags left on top of
//! each other, a waypoint behind the camera mirrored onto the wrong edge
//! (the w < 0 flip), a pool that never empties or does not survive a
//! restore, a panel whose press never reaches its button (the ray not
//! mapped to its layout), particles that differ between two runs.

use crate::support::*;
use og_core::glam::{Vec2, Vec3};
use og_draw::{Overlay, UiMassInstance, UiMassKind, UiMassSpace, UiMassTemplate};
use og_ui::*;

const W: f32 = 1280.0;
const H: f32 = 720.0;

fn view() -> WorldView {
    WorldView::look_at(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::ZERO,
        Vec3::Y,
        50.0,
        W / H,
        0.1,
        100.0,
    )
}

fn ui() -> Ui {
    ui_with_font(test_font_metrics())
}

fn vp() -> Viewport {
    Viewport::new(W, H)
}

fn input(p: Vec2, down: bool, pressed: bool, released: bool) -> InputFrame {
    InputFrame {
        pointer: p,
        pointer_inside: true,
        down,
        pressed,
        released,
        ..InputFrame::default()
    }
}

/// `ticks` frames of `build`, the view set before each end.
fn run(ui: &mut Ui, ticks: u32, mut build: impl FnMut(&mut Ui)) {
    for _ in 0..ticks {
        ui.begin(InputFrame::default(), vp());
        ui.set_world_view(Some(view()));
        build(ui);
        ui.end();
    }
}

fn rect(ui: &Ui, path: &str) -> (f32, f32, f32, f32) {
    let e = ui
        .element(path)
        .unwrap_or_else(|| panic!("no element {path}"));
    (e.x, e.y, e.width, e.height)
}

fn numbers() -> UiMassTemplate {
    UiMassTemplate {
        kind: UiMassKind::Number {
            decimals: 0,
            plus: false,
            compact: false,
        },
        lifetime: 48,
        ..UiMassTemplate::default()
    }
}

/// Documents configure the existing native placement and mass templates,
/// including on retained ticks; omitted settings keep their old defaults.
#[test]
fn document_world_distance_bounds_edge_controls_and_mass_settings() {
    let mut ui = ui();
    let doc = r##"<meta name="component" content="bounds">
<div>
<anchor id="Near" at="0 0 4" scale-at="5" scale-min="0.72" scale-max="1"><div class="tag">Near</div></anchor>
<anchor id="Far" at="0 0 -15" scale-at="5" scale-min="0.72" scale-max="1"><div class="tag">Far</div></anchor>
<anchor id="Edge" at="30 0 0" clamp="edge" clamp-margin="40" arrow-size="9" arrow-color="#00ff00"><div class="tag">Edge</div></anchor>
<anchor id="Contained" at="4 0 0" clamp="contain" clamp-margin="12"><div class="tag">Contained</div></anchor>
<anchor at="0 0 0" mass="hp" value="60" max="100" scale-at="8" scale-min="0.72" scale-max="1" bar-ghost="#ff0000" />
</div>
<style>.tag { width: 100px; height: 30px; background: #202020; }</style>"##;
    ui.define_document(UiDocument::parse("bounds.svelte", doc).unwrap());
    ui.mount_document(
        reactive::SourceId::new("test", "bounds"),
        "bounds",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    run(&mut ui, 40, |_| {});
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let near = ui.element("test:bounds/Near").unwrap();
    let far = ui.element("test:bounds/Far").unwrap();
    assert!((near.width - 100.0).abs() < 0.01, "{near:?}");
    assert!((far.width - 72.0).abs() < 0.01, "{far:?}");
    let edge = ui.element("test:bounds/Edge").unwrap();
    assert!((edge.x + edge.width - (W - 49.0)).abs() < 0.1, "{edge:?}");
    assert!(ui.frame().items.iter().any(|item| matches!(
        &item.cmd,
        DrawCmd::Shape { shape, .. }
            if matches!(shape.geometry, og_draw::UiGeometry::Polygon { sides: 3, .. })
            && shape.half == Vec2::splat(4.5)
            && shape.paint == og_draw::UiPaint::Solid([0.0, 1.0, 0.0, 1.0])
    )));
    let contained = ui.element("test:bounds/Contained").unwrap();
    assert!((contained.x + contained.width - (W - 12.0)).abs() < 0.1);
    let batch = &ui.world_frame().batches[0];
    assert_eq!(batch.template.distance, Vec3::new(8.0, 0.72, 1.0));
    let UiMassKind::Bar { ghost, .. } = batch.template.kind else {
        panic!("expected a bar");
    };
    assert_eq!(ghost, [1.0, 0.0, 0.0, 1.0]);
    ui.set_theme(Theme {
        text_scale: 1.5,
        ..*ui.theme()
    });
    run(&mut ui, 40, |_| {});
    let edge = ui.element("test:bounds/Edge").unwrap();
    assert!((edge.x + edge.width - (W - 73.5)).abs() < 0.1, "{edge:?}");
    let contained = ui.element("test:bounds/Contained").unwrap();
    assert!((contained.x + contained.width - (W - 18.0)).abs() < 0.1);
}

/// Last-tick placement feedback reaches the ordinary retained CSS cascade.
#[test]
fn document_anchor_feedback_styles_clamped_edges_angle_and_locked_lod() {
    let mut ui = ui();
    ui.define_document(
        UiDocument::parse(
            "feedback.svelte",
            r#"
<meta name="component" content="feedback">
<div>
<anchor id="Plate" at="0 0 0" lod="8 15" lod-lock={$locked}><div id="Body" /></anchor>
<anchor id="Bubble" at={$at} clamp="edge" arrow-size="0"><div id="Tail" /></anchor>
</div>
<style>
#Body { width: 100px; height: 20px; }
#Plate[data-lod="1"] #Body { width: 70px; }
#Plate[data-lod="2"] #Body { width: 40px; }
#Tail { width: 10px; height: 4px; background: white; }
#Bubble[data-clamped] #Tail { rotate: var(--anchor-angle); }
#Bubble[data-edge="left"] #Tail { width: 30px; }
</style>"#,
        )
        .unwrap(),
    );
    mount_stores(
        &mut ui,
        source("feedback"),
        "feedback",
        &[
            ("locked", serde_json::json!(false)),
            ("at", serde_json::json!("0 0 0")),
        ],
    );
    run(&mut ui, 10, |_| {});
    assert_eq!(rect(&ui, "test:feedback/Plate/Body").2, 100.0);
    for _ in 0..10 {
        ui.begin(InputFrame::default(), vp());
        ui.set_world_view(Some(WorldView::look_at(
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::ZERO,
            Vec3::Y,
            50.0,
            W / H,
            0.1,
            100.0,
        )));
        ui.end();
    }
    assert_eq!(rect(&ui, "test:feedback/Plate/Body").2, 70.0);
    let far_view = WorldView::look_at(
        Vec3::new(0.0, 0.0, 25.0),
        Vec3::ZERO,
        Vec3::Y,
        50.0,
        W / H,
        0.1,
        100.0,
    );
    let far = |ui: &mut Ui| {
        for _ in 0..10 {
            ui.begin(InputFrame::default(), vp());
            ui.set_world_view(Some(far_view));
            ui.end();
        }
    };
    far(&mut ui);
    assert_eq!(rect(&ui, "test:feedback/Plate/Body").2, 40.0);
    set_store(&mut ui, "locked", serde_json::json!(true));
    far(&mut ui);
    assert_eq!(rect(&ui, "test:feedback/Plate/Body").2, 100.0);
    set_store(&mut ui, "locked", serde_json::json!(false));
    far(&mut ui);
    assert_eq!(rect(&ui, "test:feedback/Plate/Body").2, 40.0);
    set_store(&mut ui, "at", serde_json::json!("-30 0 0"));
    run(&mut ui, 10, |_| {});
    assert!((rect(&ui, "test:feedback/Bubble/Tail").2 - 30.0).abs() < 0.1);
    set_store(&mut ui, "at", serde_json::json!("0 30 0"));
    run(&mut ui, 10, |_| {});
    let (_, _, w, h) = rect(&ui, "test:feedback/Bubble/Tail");
    assert!((w - 4.0).abs() < 0.1 && (h - 10.0).abs() < 0.1, "{w} × {h}");
    set_store(&mut ui, "at", serde_json::json!("0 0 0"));
    run(&mut ui, 10, |_| {});
    assert_eq!(rect(&ui, "test:feedback/Bubble/Tail").2, 10.0);
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
}

/// Each run keeps its own designed-size cap, including an em child; box
/// lengths in px keep shrinking while ch measures follow the floored font.
#[test]
fn world_text_floor_preserves_small_runs_relative_measures_and_scaled_bars() {
    let mut ui = ui_with_font(test_font());
    ui.define_document(UiDocument::parse("floor.svelte", r#"
<meta name="component" content="floor">
<div style="--type-name-size:15px;--type-title-size:10.5px;--type-unit-size:13px;--type-boss-size:17px;--type-cast-size:11.5px">
<anchor id="Plate" at="0 0 0" scale-at="12" scale-min="0.72" scale-max="1" text-floor="11">
  <div id="Name" style="font-size:var(--type-name-size)">Ashbound Thrall</div>
  <div style="font-size:var(--type-boss-size)">ASHEN COLOSSUS</div>
  <div style="font-size:var(--type-cast-size);letter-spacing:0.08em">SHADOW VOLLEY</div>
  <div style="font-size:var(--type-title-size)">WARDEN</div>
  <p style="margin:0;font-size:var(--type-unit-size)">412<span style="font-size:0.78em">m</span></p>
  <div id="Speech" style="font-size:var(--type-name-size);width:27ch">Break its shield then strike the runes before it rises.</div>
  <div id="Bar" style="width:116px;height:5px;background:red" />
  <p id="Scroll" style="margin:0;font-size:var(--type-name-size);width:150px;white-space:nowrap;text-overflow:-og-scroll 2s 400ms">Crown of the Ninefold King</p>
</anchor>
<anchor id="Unfloored" at="3 0 0" scale-at="12" scale-min="0.72" scale-max="1"><div style="font-size:var(--type-title-size)">CONTROL</div></anchor>
</div>"#).unwrap());
    ui.mount_document(
        source("floor"),
        "floor",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    for text_scale in [1.0, 1.5] {
        ui.set_theme(Theme {
            text_scale,
            ..*ui.theme()
        });
        let mut near_speech: Option<(f32, f32)> = None;
        for distance in [6.0, 40.0, 6.0, 40.0] {
            for _ in 0..4 {
                ui.begin(InputFrame::default(), vp());
                ui.set_world_view(Some(WorldView::look_at(
                    Vec3::new(0.0, 0.0, distance),
                    Vec3::ZERO,
                    Vec3::Y,
                    50.0,
                    W / H,
                    0.1,
                    100.0,
                )));
                ui.end();
            }
            assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
            let mut overlay = Overlay::new(W, H, 1.0);
            ui.frame().emit(&EmitDesc::default(), &mut overlay);
            for (text, near, far) in [
                ("Ashbound Thrall", 15.0, 11.0),
                ("WARDEN", 10.5, 10.5),
                ("412", 13.0, 11.0),
                ("m", 10.14, 10.14),
                ("ASHEN COLOSSUS", 17.0, 12.24),
                ("SHADOW VOLLEY", 11.5, 11.0),
            ] {
                let run = overlay
                    .texts
                    .iter()
                    .find(|t| t.text == text)
                    .unwrap_or_else(|| panic!("missing {text}"));
                let expected = if distance < 12.0 { near } else { far } * text_scale;
                assert!(
                    (run.size - expected).abs() < 0.02,
                    "{text}: {} expected {expected}",
                    run.size
                );
            }
            let scale = if distance < 12.0 { 1.0 } else { 0.72 };
            assert!((rect(&ui, "test:floor/Plate/Bar").2 - 116.0 * scale).abs() < 0.1);
            assert!((rect(&ui, "test:floor/Plate/Scroll").2 - 150.0 * scale).abs() < 0.1);
            let control = overlay.texts.iter().find(|t| t.text == "CONTROL").unwrap();
            assert!((control.size - 10.5 * text_scale * scale).abs() < 0.02);
            let cast = overlay
                .texts
                .iter()
                .find(|t| t.text == "SHADOW VOLLEY")
                .unwrap();
            assert!((cast.tracking - 0.08).abs() < 0.001);
            let speech = rect(&ui, "test:floor/Plate/Speech");
            let name = overlay
                .texts
                .iter()
                .find(|t| t.text == "Ashbound Thrall")
                .unwrap();
            // Both the ch measure and line height follow the compensated font,
            // so distance cannot squeeze this paragraph onto extra lines.
            let normalized = (speech.2 / name.size, speech.3 / name.size);
            if let Some((width, height)) = near_speech {
                assert!((normalized.0 - width).abs() < 0.05);
                // Layout snaps line boxes to pixels; one extra line is much
                // larger than this subpixel rounding allowance.
                assert!((speech.3 - height * name.size).abs() < 1.0);
            } else {
                near_speech = Some(normalized);
            }
        }
        let scroll_x = |ui: &Ui| {
            let mut overlay = Overlay::new(W, H, 1.0);
            ui.frame().emit(&EmitDesc::default(), &mut overlay);
            overlay
                .texts
                .iter()
                .find(|t| t.text == "Crown of the Ninefold King")
                .unwrap()
                .position
                .x
        };
        let initial = scroll_x(&ui);
        let bounds = rect(&ui, "test:floor/Plate/Scroll");
        for _ in 0..60 {
            ui.begin(InputFrame::default(), vp());
            ui.set_world_view(Some(WorldView::look_at(
                Vec3::new(0.0, 0.0, 40.0),
                Vec3::ZERO,
                Vec3::Y,
                50.0,
                W / H,
                0.1,
                100.0,
            )));
            ui.end();
        }
        assert!(
            (scroll_x(&ui) - initial).abs() > 1.0,
            "floored overflow keeps scrolling"
        );
        assert_eq!(
            rect(&ui, "test:floor/Plate/Scroll"),
            bounds,
            "only text moves"
        );
        assert!(
            ui.frame().items.iter().any(|it| matches!(
                it.cmd,
                DrawCmd::BeginLayer {
                    mask: Some(og_draw::UiMask::Gradient(_)),
                    ..
                }
            )),
            "scrolling text retains its edge fade"
        );
    }
}

#[test]
fn document_containment_can_keep_an_offscreen_boss_readable() {
    let mut ui = ui();
    ui.define_document(UiDocument::parse("boss.svelte", r##"
<meta name="component" content="boss">
<div>
<anchor id="Boss" at="0 30 0" clamp="contain" clamp-margin="12" contain-fade="none"><div style="width:280px;height:30px;background:#ff0000" /></anchor>
<anchor id="Default" at="0 30 0" clamp="contain"><div style="width:280px;height:30px;background:#00ff00" /></anchor>
</div>"##).unwrap());
    ui.mount_document(
        source("boss"),
        "boss",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    run(&mut ui, 40, |_| {});
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let boss = ui.element("test:boss/Boss").unwrap();
    assert!((boss.y - 12.0).abs() < 0.1, "{boss:?}");
    let mut overlay = Overlay::new(W, H, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    assert!(
        overlay
            .ui
            .shapes
            .iter()
            .any(|s| s.paint == og_draw::UiPaint::Solid([1.0, 0.0, 0.0, 1.0]))
    );
    assert!(
        !overlay
            .ui
            .shapes
            .iter()
            .any(|s| matches!(s.paint, og_draw::UiPaint::Solid([0.0, 1.0, 0.0, a]) if a > 0.01))
    );
}

#[test]
fn mass_elements_pool_age_out_and_replay() {
    let mut ui = ui();
    let h = ui.define_mass("damage", numbers(), UiMassSpace::World);
    run(&mut ui, 1, |ui| {
        for i in 0..2000 {
            ui.spawn_mass(
                h,
                UiMassInstance {
                    position: [i as f32 * 0.01, 1.0, 0.0],
                    value: i as f32,
                    color: [1.0; 4],
                    scale: 1.0,
                    ..Default::default()
                },
            );
        }
    });
    assert_eq!(ui.mass_count(h), 2000);
    let mut o = Overlay::new(W, H, 1.0);
    ui.world_frame().emit(1000, &mut o);
    assert_eq!(o.ui.mass.len(), 1);
    assert_eq!(o.ui.mass[0].instances.len(), 2000);
    assert_eq!(o.ui.tick, o.ui.mass[0].instances[0].start);
    // Saved and restored into another UI: the same pool.
    let bytes = ui.save_world_state();
    let mut other = crate::world::ui();
    let h2 = other.define_mass("damage", numbers(), UiMassSpace::World);
    other.restore_world_state(&bytes).unwrap();
    assert_eq!(other.mass_count(h2), 2000);
    assert_eq!(other.save_world_state(), bytes);
    assert!(
        other
            .restore_world_state(&bytes[..bytes.len() - 3])
            .is_err()
    );
    // Past its 48 ticks the pool is empty.
    run(&mut ui, 47, |_| {});
    assert_eq!(ui.mass_count(h), 2000, "still alive at tick 47");
    run(&mut ui, 1, |_| {});
    assert_eq!(ui.mass_count(h), 0);
    // push_mass shows for one tick only.
    run(&mut ui, 1, |ui| ui.push_mass(h, UiMassInstance::default()));
    assert_eq!(ui.world_frame().batches[0].instances.len(), 1);
    run(&mut ui, 1, |_| {});
    assert!(ui.world_frame().batches.is_empty());
}

/// A world panel takes a press through the pointer ray: the ray meets its
/// quad where its button is drawn, so the button is pressed, and a
/// pointer handler on the panel reads the point in its own box; a press
/// off the quad reaches nothing.
#[test]
fn a_world_panel_takes_a_press_through_the_pointer_ray() {
    let mut ui = ui();
    let doc = r#"<meta name="component" content="term">
<div id="Root">
<panel id="Terminal" world at="0 0 0" size="3 2" onpointerdown={down}>
  <text id="Title">Terminal</text>
  <button id="Run" onclick={run}>Run</button>
</panel>
</div>
<style>#Terminal { width: 480px; height: 320px; }</style>"#;
    ui.define_document(UiDocument::parse("term.svelte", doc).unwrap());
    ui.mount_document(
        reactive::SourceId::new("test", "term"),
        "term",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    let tick = |ui: &mut Ui, i: InputFrame| {
        ui.begin(i, vp());
        ui.set_world_view(Some(view()));
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
        ui.take_doc_events()
    };
    for _ in 0..2 {
        tick(&mut ui, InputFrame::default());
    }
    // Where the button is on the panel, as a world point, on the screen.
    let (bx, by, bw, bh) = rect(&ui, "test:term/Terminal/Run");
    let (px, py, ..) = rect(&ui, "test:term/Terminal");
    let u = (bx + bw / 2.0 - px) / 480.0;
    let v = (by + bh / 2.0 - py) / 320.0;
    let p = Vec3::new((u - 0.5) * 3.0, (0.5 - v) * 2.0, 0.0);
    let (sx, sy, _) = view().project(p, W, H).unwrap();
    let at = Vec2::new(sx, sy);
    let events = tick(&mut ui, input(at, true, true, false));
    let down = events
        .iter()
        .find(|e| e.name == "down")
        .unwrap_or_else(|| panic!("the ray met no panel: {events:?}"));
    let (x, y) = (
        down.value["x"].as_f64().unwrap() as f32,
        down.value["y"].as_f64().unwrap() as f32,
    );
    assert!(
        (x - u * 480.0).abs() < 1.0 && (y - v * 320.0).abs() < 1.0,
        "the point in the panel's box: {x}, {y}"
    );
    let events = tick(&mut ui, input(at, false, false, true));
    assert!(
        events.iter().any(|e| e.name == "run"),
        "the panel's button took no press: {events:?}"
    );
    // Off the quad: nothing.
    let off = Vec2::new(40.0, 40.0);
    let mut events = tick(&mut ui, input(off, true, true, false));
    events.extend(tick(&mut ui, input(off, false, false, true)));
    assert!(events.is_empty(), "{events:?}");
}

/// A document's `<panel world>` takes presses through the pointer ray on
/// ticks it is not built again, and an idle panel builds nothing.
///
/// Fails when a retained panel is placed or hit only on the tick it was
/// built (the press goes nowhere), or when it rebuilds every tick.
#[test]
fn a_document_world_panel_takes_a_press_while_it_is_not_rebuilt() {
    let mut ui = ui();
    let doc = r#"<meta name="component" content="term">
<div id="Root">
<button id="Out" onclick={out}>Out</button>
<panel id="Terminal" world at="0 0 0" size="3 2">
  <div id="Row"><button id="Run" onclick={run}>{$label}</button></div>
</panel>
<anchor id="Tag" at="0 1 0"><button id="Ping" onclick={ping}>{$label}</button></anchor>
</div>
<style>#Terminal { width: 480px; height: 320px; align-items: center; justify-content: center; }</style>"#;
    let send = |ui: &mut Ui, command: serde_json::Value| {
        let reply = reactive::wire::decode(
            &ui.apply_reactive(&reactive::wire::encode(&serde_json::json!([command])).unwrap()),
        )
        .unwrap();
        assert_eq!(reply[0]["ok"], true, "{reply}");
    };
    let label = reactive::SourceId::new("test", "label");
    send(
        &mut ui,
        serde_json::json!({"op": "signal", "source": label, "initial": "Run"}),
    );
    ui.define_document(UiDocument::parse("term.svelte", doc).unwrap());
    ui.mount_document(
        reactive::SourceId::new("test", "term"),
        "term",
        [("label".to_string(), label.clone())].into_iter().collect(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    let tick = |ui: &mut Ui, i: InputFrame| {
        ui.begin(i, vp());
        ui.set_world_view(Some(view()));
        ui.end().stats
    };
    for _ in 0..3 {
        tick(&mut ui, InputFrame::default());
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let idle = tick(&mut ui, InputFrame::default());
    assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{idle:?}");
    // The panel's subtree is named as if in place.
    let run = "test:term/Terminal/Row/Run";
    let ping = "test:term/Tag/Ping";
    let (bx, by, bw, bh) = rect(&ui, run);
    let (px, py, ..) = rect(&ui, "test:term/Terminal");
    let u = (bx + bw / 2.0 - px) / 480.0;
    let v = (by + bh / 2.0 - py) / 320.0;
    let p = Vec3::new((u - 0.5) * 3.0, (0.5 - v) * 2.0, 0.0);
    let (sx, sy, _) = view().project(p, W, H).unwrap();
    let at = Vec2::new(sx, sy);
    let click = |ui: &mut Ui, p: Vec2| {
        let mut events = Vec::new();
        for i in [input(p, true, true, false), input(p, false, false, true)] {
            tick(ui, i);
            events.extend(ui.take_doc_events());
        }
        events
    };
    let center = |ui: &Ui, path: &str| {
        let (x, y, w, h) = rect(ui, path);
        Vec2::new(x + w / 2.0, y + h / 2.0)
    };
    ui.take_doc_events();
    let o = center(&ui, "test:term/Out");
    let events = click(&mut ui, o);
    assert!(events.iter().any(|e| e.name == "out"), "{events:?}");
    // A ray from the pointer meets the quad and presses the button.
    let events = click(&mut ui, at);
    assert!(
        events.iter().any(|e| e.name == "run"),
        "the panel's button took no press: {events:?}"
    );
    // Pressed, it holds focus: Enter activates it.
    tick(
        &mut ui,
        InputFrame {
            keys: vec![UiKey::Enter],
            keys_pressed: vec!["Enter"],
            ..Default::default()
        },
    );
    let events = ui.take_doc_events();
    assert!(events.iter().any(|e| e.name == "run"), "{events:?}");
    // The anchored button takes a press where it is drawn.
    let pa = center(&ui, ping);
    let events = click(&mut ui, pa);
    assert!(events.iter().any(|e| e.name == "ping"), "{events:?}");
    // A press on nothing takes the focus away; idle, nothing is built.
    click(&mut ui, Vec2::new(4.0, H - 4.0));
    for _ in 0..30 {
        tick(&mut ui, InputFrame::default());
    }
    let idle = tick(&mut ui, InputFrame::default());
    assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{idle:?}");
    // A label both buttons read: each written in place, its layer kept.
    send(
        &mut ui,
        serde_json::json!({"op": "set", "source": label, "value": "Go"}),
    );
    let patched = tick(&mut ui, InputFrame::default());
    assert_eq!(patched.reconciled, 2, "{patched:?}");
    for path in [run, ping] {
        assert_eq!(
            ui.element(path).map(|e| e.text.clone()),
            Some("Go".into()),
            "{path}"
        );
    }
    // Narrower, each button and what holds it read their new size once.
    let settled = tick(&mut ui, InputFrame::default());
    assert!(settled.reconciled <= 4, "{settled:?}");
    let idle = tick(&mut ui, InputFrame::default());
    assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{idle:?}");
    let events = click(&mut ui, at);
    assert!(
        events.iter().any(|e| e.name == "run"),
        "after a patch: {events:?}"
    );
    // Saved and restored: the panel and the anchor stand and take presses.
    let saved = ui.save_state();
    ui.restore_state(&saved).unwrap();
    for _ in 0..2 {
        tick(&mut ui, InputFrame::default());
    }
    assert_eq!(ui.element(run).map(|e| e.text.clone()), Some("Go".into()));
    let events = click(&mut ui, at);
    assert!(
        events.iter().any(|e| e.name == "run"),
        "after a restore: {events:?}"
    );
    let pa = center(&ui, ping);
    let events = click(&mut ui, pa);
    assert!(
        events.iter().any(|e| e.name == "ping"),
        "after a restore: {events:?}"
    );
}

/// A `<ground>` lies flat at its point, drawn by the UI as a decal where
/// its plane is seen: the camera moving moves its quad and drape
/// (`UiFrame::grounds`) and nothing else. It is inert; with the camera
/// among it (a corner behind the eye) it covers the whole screen, and
/// wholly behind the camera it is not drawn.
///
/// Ways it could fail: the quad written into the commands (every tick
/// relevels and repaints the screen), the ground rebuilt or laid out per
/// tick, a press reaching its button, a quad wrapped from behind the eye,
/// or a ground skipped while the camera stands among it.
#[test]
fn a_document_ground_follows_the_camera_and_builds_nothing() {
    let mut ui = ui();
    let doc = r#"<meta name="component" content="sel">
<div id="Root">
<button id="Out" onclick={out}>Out</button>
<ground id="Ring" at="0 0 0" size="3 3">
  <div id="Mark"><button id="In" onclick={inside}>In</button></div>
</ground>
</div>
<style>#Ring { width: 200px; height: 200px; align-items: center; justify-content: center; }</style>"#;
    ui.define_document(UiDocument::parse("sel.svelte", doc).unwrap());
    ui.mount_document(
        reactive::SourceId::new("test", "sel"),
        "sel",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    let look = |eye: Vec3, at: Vec3| WorldView::look_at(eye, at, Vec3::Y, 50.0, W / H, 0.1, 100.0);
    let tick = |ui: &mut Ui, v: WorldView, i: InputFrame| {
        ui.begin(i, vp());
        ui.set_world_view(Some(v));
        ui.end().stats
    };
    let above = look(Vec3::new(0.0, 5.0, 6.0), Vec3::ZERO);
    for _ in 0..3 {
        tick(&mut ui, above, InputFrame::default());
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let quad = |ui: &Ui| ui.frame().grounds.first().and_then(|g| g.corners);
    let q = quad(&ui).expect("a ground seen from above");
    // Its far edge (top) is above and narrower than its near one.
    assert!(q[0].y < q[3].y && q[1].y < q[2].y, "{q:?}");
    assert!((q[1].x - q[0].x) < (q[2].x - q[3].x), "{q:?}");
    let centre = (q[0] + q[1] + q[2] + q[3]) / 4.0;
    assert!((centre.x - W / 2.0).abs() < 2.0, "{q:?}");
    let idle = tick(&mut ui, above, InputFrame::default());
    let items = ui.frame().items.clone();
    // Projection does not change the layer's CSS layout.
    assert_eq!(
        (idle.bindings, idle.reconciled, idle.laid_out),
        (0, 0, 0),
        "{idle:?}"
    );
    // The camera moves: the quad moves, the commands stay as they were.
    let moved = tick(
        &mut ui,
        look(Vec3::new(2.0, 4.0, 6.0), Vec3::ZERO),
        InputFrame::default(),
    );
    assert_eq!(
        (moved.bindings, moved.reconciled, moved.laid_out),
        (0, 0, 0),
        "{moved:?}"
    );
    assert_eq!(moved.painted, idle.painted, "{moved:?} vs {idle:?}");
    assert_ne!(quad(&ui), Some(q));
    assert!(
        ui.frame().items == items,
        "the frame's commands moved with the camera"
    );
    // Inert: a press where it is seen reaches what is behind it, not its button.
    ui.take_doc_events();
    for i in [
        input(centre, true, true, false),
        input(centre, false, false, true),
    ] {
        tick(&mut ui, above, i);
    }
    let events = ui.take_doc_events();
    assert!(!events.iter().any(|e| e.name == "inside"), "{events:?}");
    // Among it, a corner behind the eye: no quad (it would wrap), the
    // whole screen covered, the commands as they were.
    tick(
        &mut ui,
        look(Vec3::new(0.0, 1.0, -1.0), Vec3::new(0.0, 1.0, -10.0)),
        InputFrame::default(),
    );
    let g = ui.frame().grounds.first().copied().expect("drawn among it");
    assert_eq!(g.corners, None);
    assert_eq!(g.bounds, Rect::new(0.0, 0.0, W, H));
    assert!(ui.frame().items == items);
    // Wholly behind the camera: not drawn.
    tick(
        &mut ui,
        look(Vec3::new(0.0, 1.0, -5.0), Vec3::new(0.0, 1.0, -10.0)),
        InputFrame::default(),
    );
    assert!(ui.frame().grounds.is_empty());
}

/// A `<ground>` is a layer holding all it draws: a child past its box (a
/// path's badge at its end) keeps its own width, and the decal's screen
/// bounds reach it, as a CSS box's overflow shows.
///
/// Ways it could fail: the ground laid out clipped to its box (the badge
/// cut to the room its inset leaves), or the decal drawn only over its
/// box's quad (the badge cut where the box ends on the screen).
#[test]
fn a_ground_draws_what_overflows_its_box() {
    let mut ui = ui();
    let doc = r#"<meta name="component" content="sel">
<div id="Root">
<ground id="Path" at="0 0 0" size="3 3"><span id="Badge">Out of reach · 6 AP</span></ground>
</div>
<style>
#Path { width: 200px; height: 200px; }
#Badge { position: absolute; left: 150px; top: 20px; white-space: nowrap; }
</style>"#;
    ui.define_document(UiDocument::parse("sel.svelte", doc).unwrap());
    ui.mount_document(
        reactive::SourceId::new("test", "sel"),
        "sel",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    let above = WorldView::look_at(
        Vec3::new(0.0, 5.0, 6.0),
        Vec3::ZERO,
        Vec3::Y,
        50.0,
        W / H,
        0.1,
        100.0,
    );
    for _ in 0..3 {
        ui.begin(InputFrame::default(), vp());
        ui.set_world_view(Some(above));
        ui.end();
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let path = ui
        .probe_elements()
        .into_iter()
        .find(|e| e.path.ends_with("/Badge"))
        .expect("the badge");
    // Its text is well over the 50px its inset leaves inside the box.
    assert!(path.width > 100.0, "the badge is cut: {}", path.width);
    let g = ui.frame().grounds.first().copied().expect("a ground seen");
    let box_right = g
        .corners
        .unwrap()
        .iter()
        .map(|c| c.x)
        .fold(f32::MIN, f32::max);
    // The badge reaches about twice the box's half width past its centre.
    assert!(
        g.bounds.right() > box_right + 100.0,
        "the decal stops at its box: {:?} vs {box_right}",
        g.bounds
    );
}

/// A ground's text reads over the other grounds' drawing: a path's badge
/// stays over a ring laid after it, where it overlaps that ring.
///
/// Ways it could fail: the badge composited with its ground's marks, in
/// the one band every ground shares, so a ground later in the tree (the
/// ring) draws over it.
#[test]
fn a_grounds_label_draws_over_the_other_grounds() {
    // A font that draws, so the badge's text is in the overlay.
    let mut ui = ui_with_font(test_font());
    let doc = r#"<meta name="component" content="sel">
<div id="Root">
<ground id="Path" at="0 0 0" size="3 3"><div id="Line"></div><span id="Badge">12 m · 2 AP</span></ground>
<ground id="Ring" at="0 0 0" size="3 3"><div id="Circle"></div></ground>
</div>
<style>
#Path, #Ring { width: 200px; height: 200px; }
#Line { width: 200px; height: 4px; background: #fff; }
#Badge { position: absolute; left: 80px; top: 90px; white-space: nowrap; }
#Circle { width: 200px; height: 200px; border: 4px solid #3c3; border-radius: 50%; }
</style>"#;
    ui.define_document(UiDocument::parse("sel.svelte", doc).unwrap());
    ui.mount_document(
        reactive::SourceId::new("test", "sel"),
        "sel",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    let above = WorldView::look_at(
        Vec3::new(0.0, 5.0, 6.0),
        Vec3::ZERO,
        Vec3::Y,
        50.0,
        W / H,
        0.1,
        100.0,
    );
    for _ in 0..3 {
        ui.begin(InputFrame::default(), vp());
        ui.set_world_view(Some(above));
        ui.end();
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let mut o = Overlay::new(W, H, 1.0);
    ui.frame().emit(
        &EmitDesc {
            layer: 1000,
            time: 0.0,
        },
        &mut o,
    );
    let text = o
        .texts
        .iter()
        .find(|t| t.text.contains("AP"))
        .expect("the badge's text");
    let layers = &o.ui.layers;
    let holding = |level: i32| {
        layers
            .iter()
            .enumerate()
            .rfind(|(_, l)| l.capture[0] <= level && level <= l.capture[1])
            .map(|(k, _)| k as u32)
            .expect("the text is drawn into a layer")
    };
    // Where each ground layer is composited on the screen.
    let composite = |k: u32| {
        o.ui.shapes
            .iter()
            .find(|s| s.paint == og_draw::UiPaint::Layer(k))
            .map(|s| s.layer)
            .expect("a ground layer composited")
    };
    let label = holding(text.layer);
    let others: Vec<i32> = (0..layers.len() as u32)
        .filter(|&k| k != label && layers[k as usize].ground.is_some())
        .map(composite)
        .collect();
    assert_eq!(others.len(), 2, "two grounds' marks: {others:?}");
    assert!(
        others.iter().all(|&l| composite(label) > l),
        "the badge at {} under a ground's marks at {others:?}",
        composite(label)
    );
}

/// Decluttered tags stay on the screen and clear of each other: a crowd
/// of nameplates whose point is near the top stacks down under the first
/// once stacking up would leave the screen, and a moved tag never leaves
/// it (as map labels are placed: a candidate off the screen is no spot).
///
/// Ways it could fail: a stack climbing off the top of the screen (moves
/// bounded only for `contain` tags), tags left on top of each other, or
/// the crowd hidden when there is room below.
#[test]
fn decluttered_tags_stay_on_the_screen_and_clear() {
    let mut ui = ui();
    let tag = |name: &str, priority: u32, mode: &str| {
        format!(
            r#"<anchor id="{name}" at="0 1.8 0" declutter="{mode}" priority="{priority}"><div class="tag">{name}</div></anchor>"#
        )
    };
    let tags = [
        tag("A", 50, "stack"),
        tag("B", 40, "stack"),
        tag("C", 30, "shift"),
        tag("D", 20, "stack"),
        tag("E", 10, "stack"),
    ]
    .join("\n");
    let doc = format!(
        r#"<meta name="component" content="sel">
<div id="Root">
{tags}
</div>
<style>.tag {{ width: 160px; height: 40px; background: #202020; }}</style>"#
    );
    ui.define_document(UiDocument::parse("sel.svelte", &doc).unwrap());
    ui.mount_document(
        reactive::SourceId::new("test", "sel"),
        "sel",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    // Gliding and fading settle.
    run(&mut ui, 40, |_| {});
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let shown: Vec<_> = ui
        .probe_elements()
        .into_iter()
        .filter(|e| e.kind != "world" && e.opaque && e.width == 160.0)
        .collect();
    // Its point is 82px from the top: one tag fits above it, so the rest
    // stack down beneath, all five shown.
    assert_eq!(shown.len(), 5, "{shown:#?}");
    for e in &shown {
        assert!(
            !e.clipped && e.x >= 0.0 && e.y >= 0.0 && e.x + e.width <= W && e.y + e.height <= H,
            "off the screen: {e:#?}"
        );
    }
    for (k, a) in shown.iter().enumerate() {
        for b in &shown[k + 1..] {
            let apart = a.x + a.width <= b.x
                || b.x + b.width <= a.x
                || a.y + a.height <= b.y
                || b.y + b.height <= a.y;
            assert!(apart, "overlapping: {a:#?} {b:#?}");
        }
    }
}

/// Mounts the document `body` (with `css`) as the only one.
fn mount(ui: &mut Ui, body: &str, css: &str) {
    let doc = format!(
        "<meta name=\"component\" content=\"sel\">\n<div id=\"Root\">\n{body}\n</div>\n<style>{css}</style>"
    );
    ui.define_document(UiDocument::parse("sel.svelte", &doc).unwrap());
    ui.mount_document(
        reactive::SourceId::new("test", "sel"),
        "sel",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
}

/// Boxes (x, y, width, height) at least `gap` apart.
fn apart(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32), gap: f32) -> bool {
    a.0 + a.2 + gap <= b.0
        || b.0 + b.2 + gap <= a.0
        || a.1 + a.3 + gap <= b.1
        || b.1 + b.3 + gap <= a.1
}

/// A decluttered tag keeps a gap from all a tag over it draws: a
/// targeted nameplate's frame scaled up past its box, a name wider than
/// its plate.
///
/// Ways it could fail: tags kept apart by their layout boxes only (the
/// scaled frame drawn over the tag stacked on it), or tags that touch
/// counted as clear.
#[test]
fn decluttered_tags_keep_clear_of_what_each_draws() {
    let mut ui = ui();
    mount(
        &mut ui,
        r#"<anchor id="A" at="0 0 0" declutter="stack" priority="50"><div id="Big" class="tag big">A</div></anchor>
<anchor id="B" at="0 0 0" declutter="stack" priority="10"><div id="Small" class="tag">B</div></anchor>"#,
        ".tag { width: 100px; height: 40px; background: #202020; } .big { scale: 1.5; }",
    );
    run(&mut ui, 40, |_| {});
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let at = |id: &str| {
        let e = ui
            .probe_elements()
            .into_iter()
            .find(|e| e.id == id)
            .unwrap_or_else(|| panic!("no {id}"));
        (e.x, e.y, e.width, e.height)
    };
    let (big, small) = (at("Big"), at("Small"));
    // Drawn half again its 100 × 40.
    assert!((big.2 - 150.0).abs() < 0.5, "{big:?}");
    assert!(apart(big, small, 3.9), "B on A's frame: {big:?} {small:?}");
}

/// A tag keeps clear of the text a ground draws on the scene (a path's
/// badge at its end), which is pinned and never moves for it.
///
/// Ways it could fail: grounds left out of declutter, so a prompt over a
/// path's end hides its badge ("Out of reach" read "ut of reach").
#[test]
fn decluttered_tags_keep_clear_of_a_grounds_text() {
    let mut ui = ui();
    mount(
        &mut ui,
        r#"<ground id="Path" at="0 0 0" size="3 3"><span id="Badge">Out of reach</span></ground>
<anchor id="Prompt" at="0 0 0" declutter="shift"><div id="Tag" class="tag">E</div></anchor>"#,
        "#Path { width: 200px; height: 200px; } #Badge { position: absolute; left: 50px; top: 90px; width: 100px; height: 20px; } .tag { width: 60px; height: 40px; background: #202020; }",
    );
    let above = WorldView::look_at(
        Vec3::new(0.0, 5.0, 6.0),
        Vec3::ZERO,
        Vec3::Y,
        50.0,
        W / H,
        0.1,
        100.0,
    );
    for _ in 0..40 {
        ui.begin(InputFrame::default(), vp());
        ui.set_world_view(Some(above));
        ui.end();
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    // The badge is the middle half of the 3 m box across and a tenth of
    // it deep: ±0.75 m by ±0.15 m about the point, seen from above.
    let seen: Vec<Vec2> = [(-0.75, -0.15), (0.75, -0.15), (0.75, 0.15), (-0.75, 0.15)]
        .iter()
        .map(|&(x, z)| {
            let (sx, sy, _) = above.project(Vec3::new(x, 0.0, z), W, H).unwrap();
            Vec2::new(sx, sy)
        })
        .collect();
    let (lo, hi) = seen
        .iter()
        .fold((seen[0], seen[0]), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
    let badge = (lo.x, lo.y, hi.x - lo.x, hi.y - lo.y);
    let tag = ui
        .probe_elements()
        .into_iter()
        .find(|e| e.id == "Tag")
        .expect("the tag");
    let tag = (tag.x, tag.y, tag.width, tag.height);
    assert!(
        apart(badge, tag, 0.0),
        "the tag covers the badge: {badge:?} {tag:?}"
    );
}

/// Numbers spawned on one point on one tick (a target's hits) stand
/// apart, each whole on the screen and clear of the tag over the target.
///
/// Ways it could fail: mass numbers left out of declutter (a target's
/// hits drawn one over another and over its nameplate, "44277009640"),
/// or a number moved off the screen or dropped while there is room.
#[test]
fn numbers_spawned_together_stand_apart() {
    let mut ui = ui();
    mount(
        &mut ui,
        r#"<anchor id="Plate" at="0 1 0"><div id="Tag" class="tag">Ashen Pyre</div></anchor>"#,
        ".tag { width: 120px; height: 30px; background: #202020; }",
    );
    let h = ui.define_mass("damage", numbers(), UiMassSpace::World);
    run(&mut ui, 3, |_| {});
    run(&mut ui, 1, |ui| {
        for _ in 0..8 {
            ui.spawn_mass(
                h,
                UiMassInstance {
                    position: [0.0, 1.0, 0.0],
                    value: 88.0,
                    color: [1.0; 4],
                    scale: 1.0,
                    ..Default::default()
                },
            );
        }
    });
    let tag = ui
        .probe_elements()
        .into_iter()
        .find(|e| e.id == "Tag")
        .expect("the tag");
    let tag = (tag.x, tag.y, tag.width, tag.height);
    let (x, y, _) = view().project(Vec3::new(0.0, 1.0, 0.0), W, H).unwrap();
    // The template's 18 px numbers, centred on their points.
    let w = test_font_metrics().metrics.measure("88", 18.0);
    let boxes: Vec<_> = ui.world_frame().batches[0]
        .instances
        .iter()
        .map(|i| {
            assert!(i.color[3] > 0.0, "a number dropped: {i:?}");
            (x + i.aux[0] - w / 2.0, y + i.aux[1] - 9.0, w, 18.0)
        })
        .collect();
    assert_eq!(boxes.len(), 8);
    for (k, a) in boxes.iter().enumerate() {
        assert!(
            a.0 >= 0.0 && a.1 >= 0.0 && a.0 + a.2 <= W && a.1 + a.3 <= H,
            "off the screen: {a:?}"
        );
        assert!(apart(*a, tag, 3.9), "on the tag: {a:?} {tag:?}");
        for b in &boxes[k + 1..] {
            assert!(apart(*a, *b, 3.9), "one over another: {a:?} {b:?}");
        }
    }
}

/// A number keeps clear of a tag over the whole path it rises along
/// while it shows, not only where it is spawned.
///
/// Ways it could fail: a number placed clear of the tag over its target
/// only where it starts, so it rises straight into the nameplate (the
/// crowd's hits drawn over Ashen Pyre's name).
#[test]
fn a_rising_number_keeps_clear_of_the_tag_over_it() {
    let mut ui = ui();
    mount(
        &mut ui,
        r#"<anchor id="Plate" at="0 0.4 0"><div id="Tag" class="tag">Ashen Pyre</div></anchor>"#,
        ".tag { width: 120px; height: 30px; background: #202020; }",
    );
    let rising = UiMassTemplate {
        keys: vec![
            og_draw::UiMassKey::default(),
            og_draw::UiMassKey {
                t: 1.0,
                offset: Vec2::new(0.0, -120.0),
                alpha: 0.0,
                ..Default::default()
            },
        ],
        ..numbers()
    };
    let h = ui.define_mass("damage", rising.clone(), UiMassSpace::World);
    run(&mut ui, 3, |_| {});
    run(&mut ui, 1, |ui| {
        ui.spawn_mass(
            h,
            UiMassInstance {
                position: [0.0, 0.0, 0.0],
                value: 88.0,
                color: [1.0; 4],
                scale: 1.0,
                ..Default::default()
            },
        )
    });
    let tag = ui
        .probe_elements()
        .into_iter()
        .find(|e| e.id == "Tag")
        .expect("the tag");
    let tag = (tag.x, tag.y, tag.width, tag.height);
    let i = ui.world_frame().batches[0].instances[0];
    assert!(i.color[3] > 0.0, "the number dropped");
    let (x, y, _) = view().project(Vec3::ZERO, W, H).unwrap();
    let w = test_font_metrics().metrics.measure("88", 18.0);
    // Its first half life, while it is half visible or more.
    for k in 0..=24 {
        let (off, _, _) = rising.sample(k as f32 / 48.0);
        let b = (
            x + i.aux[0] + off.x - w / 2.0,
            y + i.aux[1] + off.y - 9.0,
            w,
            18.0,
        );
        assert!(
            apart(b, tag, 0.0),
            "it rises onto the tag at {k}/48: {b:?} {tag:?}"
        );
    }
}

/// A `<ground>`'s child centred by percent insets (an overlay's
/// `data-anchor=center`: a placement ghost's footprint) is laid in the
/// ground's box, as one placed in px is.
///
/// Ways it could fail: the placement laid out again after its percent is
/// resolved, cut to the screen while the ground's layer lies off it, so
/// the child and everything in it are gone from the ground.
#[test]
fn a_grounds_child_placed_by_percent_insets_stays_in_it() {
    let mut ui = ui();
    let doc = r#"<meta name="component" content="sel">
<div id="Root">
<ground id="Ghost" at="0 0 0" size="3 3"><div id="Room"><div id="Footprint"><span id="Cost">5</span></div></div></ground>
</div>
<style>
#Room { position: relative; width: 200px; height: 200px; }
#Footprint { position: absolute; top: 50%; left: 50%; translate: -50% -50%; width: 60px; height: 40px; background: red; }
</style>"#;
    ui.define_document(UiDocument::parse("sel.svelte", doc).unwrap());
    ui.mount_document(
        reactive::SourceId::new("test", "sel"),
        "sel",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    let above = WorldView::look_at(
        Vec3::new(0.0, 5.0, 6.0),
        Vec3::ZERO,
        Vec3::Y,
        50.0,
        W / H,
        0.1,
        100.0,
    );
    for _ in 0..3 {
        ui.begin(InputFrame::default(), vp());
        ui.set_world_view(Some(above));
        ui.end();
    }
    let all = ui.probe_elements();
    let at = |name: &str| {
        all.iter()
            .find(|e| e.path.ends_with(name))
            .unwrap_or_else(|| panic!("{name} is laid out"))
    };
    let (room, footprint) = (at("/Room"), at("/Footprint"));
    at("/Cost");
    // Its percent insets resolve against the room (100 px in, 100 down),
    // and its translate centres it there.
    assert_eq!(
        (footprint.x - room.x, footprint.y - room.y),
        (70.0, 80.0),
        "{footprint:?}"
    );
}

/// Reserves, opt-in danger, priority and visible fallbacks share one solve;
/// changing a retained HUD box must invalidate its placement certificate.
#[test]
fn document_world_obstacles_respect_opt_in_priority_edges_and_visible_fallbacks() {
    let scenarios = [
        (
            "hud",
            r#"<div id="Hud" reserve="hud" style:left={$left} style="position:absolute;top:320px;width:200px;height:80px;background:black" />
<anchor id="Boss" at="0 0 0" priority="400" declutter="stack" class="tag"><text>Boss</text></anchor>
<anchor id="Loot" at="0 0 0" priority="10" declutter="stack" class="tag"><text>Loot</text></anchor>"#,
        ),
        (
            "player",
            r#"<anchor id="Player" reserve="player" at="0 0 0" reserve-radius="0.6" reserve-height="1.9" />
<anchor id="Boss" at="0 0.4 0" priority="400" declutter="stack" keep-visible class="tag"><text>Boss</text></anchor>
<anchor id="Bubble" at="0 0.4 0" priority="200" declutter="shift" avoid="player" clamp="edge" arrow-size="0" class="tag"><text>Bubble</text></anchor>"#,
        ),
        (
            "player-edge",
            r#"<anchor id="Player" reserve="player" at="0 -1.2 0" reserve-radius="1.7" reserve-height="2.4" />
<anchor id="Bubble" at="0 0 0" declutter="shift" avoid="player" clamp="edge" clamp-margin="12" arrow-size="0" class="tag"><text>Bubble</text></anchor>"#,
        ),
        (
            "danger",
            r#"<ground id="Danger" at="0 0 0" size="4 4" danger style="width:200px;height:200px;background:red" />
<anchor id="Boss" at="0 0 0" priority="400" declutter="stack" class="tag"><text>Boss</text></anchor>
<anchor id="Prompt" at="0 0 0" priority="300" declutter="shift" avoid="danger" class="tag"><text>Prompt</text></anchor>"#,
        ),
        (
            "blocked",
            r#"<ground id="Danger" at="0 0 0" size="100 100" danger style="width:200px;height:200px;background:red" />
<anchor id="Prompt" at="0 0 0" declutter="shift" avoid="danger" class="tag"><div id="Badge" /></anchor>"#,
        ),
        (
            "covered",
            r#"<div reserve="hud" style="position:absolute;left:0;top:0;width:1280px;height:720px;background:black" />
<anchor at="0 0 0" declutter="stack" keep-visible class="tag"><text>Locked</text></anchor>
<anchor at="2 0 0" declutter="stack" class="tag"><text>Unlocked</text></anchor>"#,
        ),
        (
            "edge",
            r#"<div id="Hud" reserve="hud" style="position:absolute;left:0;top:300px;width:220px;height:220px;background:black" />
<anchor id="Waypoint" at="-100 0 0" priority="500" declutter="shift" clamp="edge" arrow-size="0" clamp-margin="24" class="tag"><text>Waypoint</text></anchor>"#,
        ),
    ];
    let overlaps = |a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)| {
        a.0 < b.0 + b.2 && b.0 < a.0 + a.2 && a.1 < b.1 + b.3 && b.1 < a.1 + a.3
    };
    for (scenario, body) in scenarios {
        let mut ui = ui_with_font(test_font());
        let doc = format!(
            r#"<meta name="component" content="obstacles"><div>{body}</div>
<style>.tag {{ width:80px;height:24px;color:white;font-size:14px; }} #Badge {{ width:16px;height:4px;background:white; }} #Prompt[data-blocked] #Badge {{ width:32px; }}</style>"#
        );
        ui.define_document(UiDocument::parse("obstacles.svelte", &doc).unwrap());
        mount_stores(
            &mut ui,
            source("obstacles"),
            "obstacles",
            &[("left", serde_json::json!("540px"))],
        );
        let camera = if matches!(scenario, "danger" | "blocked") {
            WorldView::look_at(
                Vec3::new(0.0, 4.0, 8.0),
                Vec3::ZERO,
                Vec3::Y,
                50.0,
                W / H,
                0.1,
                100.0,
            )
        } else {
            view()
        };
        let step = |ui: &mut Ui| {
            for _ in 0..30 {
                ui.begin(InputFrame::default(), vp());
                ui.set_world_view(Some(camera));
                ui.end();
            }
        };
        step(&mut ui);
        assert!(
            ui.frame().warnings.is_empty(),
            "{scenario}: {:?}",
            ui.frame().warnings
        );
        let get = |ui: &Ui, id: &str| rect(ui, &format!("test:obstacles/{id}"));
        match scenario {
            "hud" => {
                let hud = get(&ui, "Hud");
                for id in ["Boss", "Loot"] {
                    let tag = get(&ui, id);
                    assert!(!overlaps(tag, hud), "{id} clears HUD");
                    assert!(tag.1 + tag.3 <= hud.1, "{id} stacks upward");
                }
                assert!(!overlaps(get(&ui, "Boss"), get(&ui, "Loot")));
                set_store(&mut ui, "left", serde_json::json!("0px"));
                step(&mut ui);
                assert!(
                    (get(&ui, "Boss").1 - 336.0).abs() < 0.1,
                    "HUD layout changes release the original point"
                );
            }
            "player" => {
                let player = get(&ui, "Player");
                assert!(
                    overlaps(get(&ui, "Boss"), player),
                    "non-opted plate stays on the unit"
                );
                assert!(
                    !overlaps(get(&ui, "Bubble"), player),
                    "bubble clears player"
                );
            }
            "player-edge" => {
                let bubble = get(&ui, "Bubble");
                assert!(!overlaps(bubble, get(&ui, "Player")));
                assert!((bubble.1 - 12.0).abs() < 0.1, "nearest clear safe edge");
                assert!(
                    336.0 - bubble.1 > 240.0,
                    "only edge fallback can reach here"
                );
            }
            "danger" => {
                let ground = ui.frame().grounds[0].bounds;
                let danger = (ground.x, ground.y, ground.width, ground.height);
                assert!(
                    overlaps(get(&ui, "Boss"), danger),
                    "non-opted plate stays on the unit"
                );
                assert!(
                    !overlaps(get(&ui, "Prompt"), danger),
                    "prompt {:?} clears whole telegraph {:?} (corners {:?})",
                    get(&ui, "Prompt"),
                    danger,
                    ui.frame().grounds[0].corners
                );
                assert!(
                    !overlaps(get(&ui, "Prompt"), get(&ui, "Boss")),
                    "prompt yields to boss too"
                );
            }
            "blocked" => {
                assert_eq!(
                    get(&ui, "Prompt/Badge").2,
                    32.0,
                    "forced visible fallback reports blocked"
                );
                assert!(
                    ui.frame()
                        .items
                        .iter()
                        .any(|i| i.cmd.filled().is_some_and(|(_, color)| color[3] > 0.99
                            && color[0] > 0.9
                            && color[1] > 0.9)),
                    "blocked fallback remains opaque"
                );
            }
            "covered" => {
                let mut overlay = Overlay::new(W, H, 1.0);
                ui.frame().emit(&EmitDesc::default(), &mut overlay);
                assert!(
                    overlay
                        .texts
                        .iter()
                        .any(|t| t.text == "Locked" && t.color[3] > 0.99)
                );
                assert!(
                    !overlay
                        .texts
                        .iter()
                        .any(|t| t.text == "Unlocked" && t.color[3] > 0.01)
                );
            }
            "edge" => {
                let tag = get(&ui, "Waypoint");
                assert!((tag.0 - 24.0).abs() < 0.1, "waypoint stays on its edge");
                assert!(
                    !overlaps(tag, get(&ui, "Hud")),
                    "waypoint slides past minimap"
                );
            }
            _ => unreachable!(),
        }
    }
}

/// Bounded columns rank keyed items, fold overflow back into the document,
/// and replace/remove rows without interleaving neighbouring piles.
#[test]
fn keyed_loot_columns_bound_rank_reflow_and_write_hidden_keys() {
    use serde_json::json;
    let mut ui = ui_with_font(test_font());
    ui.define_document(UiDocument::parse("loot.svelte", r#"
<meta name="component" content="loot">
<div>
{#each $items as it (it.id)}
<anchor id="Item" at={it.at} stack-group={it.group} stack-max={$max} stack-gap="3" priority={it.rank} scale-at="12" scale-min="0.72" scale-max="1" text-floor="11" max-distance="30" style:width={it.width} class="row" class:grown={$hover && it.id == 'i0'}><text>{it.id}</text></anchor>
{/each}
<anchor id="More" stack-group="pile" stack-more bind:hidden={$hidden} class="row"><text>+{$hidden.length}</text></anchor>
</div>
<style>.row { height:24px; background:#202020; color:white; font-size:15px; } .grown { scale:1.06; transform-origin:center; }</style>
"#).unwrap());
    let items: Vec<_> = [50, 40, 30, 30, 20, 20, 10, 10, 10]
        .into_iter()
        .enumerate()
        .map(|(i, rank)| {
            json!({
                "id":format!("i{i}"), "rank":rank, "group":"pile",
                "at": if i==0 {"0 0 0"} else {"1.5 0 -15"},
                "width":format!("{}px",if i==1 {160} else {100+i*3})
            })
        })
        .collect();
    mount_stores(
        &mut ui,
        source("loot"),
        "loot",
        &[
            ("items", json!(items)),
            ("max", json!(5)),
            ("hover", json!(false)),
            // A string-list signal, not the test helper's empty keyed collection.
            ("hidden", json!(["stale"])),
        ],
    );
    let row = |ui: &Ui, id: &str| {
        ui.probe_elements()
            .into_iter()
            .find(|e| e.kind == "world" && e.path.ends_with(&format!("Item#{id}")))
            .unwrap_or_else(|| panic!("row {id}: {:?}", ui.probe_elements()))
    };
    let drawn = |ui: &Ui| {
        let mut overlay = Overlay::new(W, H, 1.0);
        ui.frame().emit(&EmitDesc::default(), &mut overlay);
        overlay
            .texts
            .into_iter()
            .filter(|t| t.color[3] > 0.99)
            .map(|t| t.text)
            .collect::<Vec<_>>()
    };
    run(&mut ui, 40, |_| {});
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    assert_eq!(
        read_store(&mut ui, "hidden"),
        json!(["i5", "i6", "i7", "i8"]),
        "{:?}",
        ui.probe_elements()
    );
    assert!(drawn(&ui).contains(&"+4".into()));
    for i in 0..5 {
        let r = row(&ui, &format!("i{i}"));
        assert!((r.width - 160.0).abs() < 0.1, "equal widest width: {r:?}");
        assert!(
            (r.x - 560.0).abs() < 0.1,
            "representative point, not scattered member: {r:?}"
        );
        assert!(
            (r.y - (336.0 - 27.0 * i as f32)).abs() < 0.1,
            "best at bottom: {r:?}"
        );
    }
    assert!(!drawn(&ui).contains(&"i5".into()));
    let unhovered = row(&ui, "i0");
    let neighbour = row(&ui, "i1");
    set_store(&mut ui, "hover", json!(true));
    run(&mut ui, 20, |_| {});
    let hovered = row(&ui, "i0");
    assert!((hovered.width / unhovered.width - 1.06).abs() < 0.001);
    assert!((hovered.x + hovered.width / 2.0 - unhovered.x - unhovered.width / 2.0).abs() < 0.1);
    assert!(
        (hovered.y + hovered.height / 2.0 - unhovered.y - unhovered.height / 2.0).abs() < 0.1,
        "hover centre: before {unhovered:?}, after {hovered:?}"
    );
    assert_eq!(
        (row(&ui, "i1").x, row(&ui, "i1").y, row(&ui, "i1").width),
        (neighbour.x, neighbour.y, neighbour.width),
        "hover never reflows the column"
    );
    set_store(&mut ui, "hover", json!(false));
    run(&mut ui, 20, |_| {});
    set_store(&mut ui, "max", json!(1));
    run(&mut ui, 40, |_| {});
    assert_eq!(
        read_store(&mut ui, "hidden"),
        json!(["i1", "i2", "i3", "i4", "i5", "i6", "i7", "i8"])
    );
    assert!(drawn(&ui).contains(&"+8".into()));
    assert!(
        (row(&ui, "i0").width - 100.0).abs() < 0.1,
        "column shrinks when widest row leaves"
    );
    set_store(&mut ui, "max", json!(5));
    run(&mut ui, 40, |_| {});
    let before = row(&ui, "i2").y;
    edit_store(&mut ui, "items", json!({"op":"remove","key":"i1"}));
    run(&mut ui, 1, |_| {});
    let gliding = row(&ui, "i2").y;
    assert!(
        gliding > before && gliding < before + 27.0,
        "keyed row glides, not remounts"
    );
    for _ in 0..12 {
        let entering = row(&ui, "i5");
        let previous_top = row(&ui, "i4");
        let mut overlay = Overlay::new(W, H, 1.0);
        ui.frame().emit(&EmitDesc::default(), &mut overlay);
        let visible = overlay
            .texts
            .iter()
            .any(|t| t.text == "i5" && t.color[3] > 0.0);
        assert!(
            !visible || entering.y + entering.height <= previous_top.y,
            "incoming row must not cross the retained top row"
        );
        run(&mut ui, 1, |_| {});
    }
    run(&mut ui, 40, |_| {});
    assert_eq!(read_store(&mut ui, "hidden"), json!(["i6", "i7", "i8"]));
    assert!(drawn(&ui).contains(&"i5".into()));
    assert!(
        (row(&ui, "i2").width - 115.0).abs() < 0.1,
        "widest removed item releases width: {:?}",
        row(&ui, "i2")
    );
    edit_store(
        &mut ui,
        "items",
        json!({"op":"patch","key":"i8","fields":{"rank":45}}),
    );
    run(&mut ui, 40, |_| {});
    assert_eq!(read_store(&mut ui, "hidden"), json!(["i5", "i6", "i7"]));
    assert!(
        row(&ui, "i8").y > row(&ui, "i2").y,
        "priority change reorders same keys"
    );
    set_store(&mut ui, "max", json!(0));
    run(&mut ui, 40, |_| {});
    assert_eq!(read_store(&mut ui, "hidden"), json!([]));
    assert!(!drawn(&ui).iter().any(|t| t.starts_with('+')));
    assert_eq!(drawn(&ui).iter().filter(|t| t.starts_with('i')).count(), 8);

    set_store(&mut ui, "max", json!(5));
    edit_store(
        &mut ui,
        "items",
        json!({"op":"insert", "row":{
            "id":"new", "rank":35, "group":"pile", "at":"2 0 -15", "width":"110px"
        }}),
    );
    run(&mut ui, 40, |_| {});
    assert_eq!(
        read_store(&mut ui, "hidden"),
        json!(["i4", "i5", "i6", "i7"])
    );
    assert!(row(&ui, "new").y > row(&ui, "i2").y);

    // Departure fades at the old column pivot before returning to its
    // own scattered point. Replaying mid-fade must preserve that pivot.
    let before = row(&ui, "new");
    edit_store(
        &mut ui,
        "items",
        json!({"op":"patch","key":"new","fields":{"group":""}}),
    );
    run(&mut ui, 1, |_| {});
    let leaving = row(&ui, "new");
    assert!((leaving.x + leaving.width / 2.0 - before.x - before.width / 2.0).abs() < 0.1);
    assert!((leaving.y - before.y).abs() < 0.1);
    let saved = ui.save_world_state();
    run(&mut ui, 1, |_| {});
    let replay = ui.save_world_state();
    ui.restore_world_state(&saved).unwrap();
    run(&mut ui, 1, |_| {});
    assert_eq!(ui.save_world_state(), replay);
    run(&mut ui, 40, |_| {});
    assert!(
        row(&ui, "new").x > before.x + 40.0,
        "returns to its own world point"
    );

    edit_store(&mut ui, "items", json!({"op":"remove","key":"new"}));
    for i in 0..3 {
        edit_store(
            &mut ui,
            "items",
            json!({"op":"insert", "row":{
                "id":format!("b{i}"), "rank":50-i, "group":"other",
                "at":"0.26 0 0", "width":"100px"
            }}),
        );
    }
    run(&mut ui, 40, |_| {});
    let first = row(&ui, "i0");
    let second = row(&ui, "b0");
    for i in 0..3 {
        let b = row(&ui, &format!("b{i}"));
        assert!(
            (b.x - second.x).abs() < 0.1,
            "other column never interleaves"
        );
        assert!((b.y - second.y + i as f32 * 27.0).abs() < 0.1);
        for id in ["i0", "i8", "i2", "i3", "i4"] {
            let a = row(&ui, id);
            assert!(
                a.x + a.width <= b.x
                    || b.x + b.width <= a.x
                    || a.y + a.height <= b.y
                    || b.y + b.height <= a.y,
                "neighbouring columns clear: {a:?} {b:?}"
            );
        }
    }
    assert!((first.x - 578.0).abs() < 1.0, "representative stayed put");
    edit_store(
        &mut ui,
        "items",
        json!({"op":"patch","key":"i0","fields":{"at":"0 0 -40"}}),
    );
    run(&mut ui, 40, |_| {});
    assert!(
        !drawn(&ui)
            .iter()
            .any(|t| t.starts_with('i') || t.starts_with('+')),
        "whole group follows representative max-distance, not members' own points"
    );
    assert!(
        drawn(&ui).contains(&"b0".into()),
        "other group is independent"
    );
}

/// Standing document bars retain hit history without rebuilding every tick.
#[test]
fn document_mass_bars_expose_tracks_ink_occlusion_and_replay_hit_ghosts() {
    use serde_json::json;
    let mut ui = ui();
    ui.define_document(UiDocument::parse("tracks.svelte", r##"
<meta name="component" content="tracks">
<div>
<anchor mass="hp" at="0 0 -35" value={$hp} max="100" shield="30" segments="5" resource="70" resource-max="100" scale-at="12" scale-min="0.7" scale-max="1" max-distance="80" bar-ghost occlusion={$mode} class="bar" />
<anchor mass="noGhost" at="1 0 0" value={$hp} max="100" bar-ghost={false} />
<anchor at="0 0 -35"><text>Above bars</text></anchor>
</div>
<style>.bar { width:46px; height:5px; color:#00ff00; --shield:#7fe0ff;
--bar-back:rgb(8 12 20 / 0.72); --bar-rim:rgb(255 255 255 / 0.28);
--bar-shield:var(--shield); --bar-resource:#6ea8ff; --bar-ghost:#ffecc8;
--bar-low:#ff4d5e; --bar-low-at:0.25; border-radius:1px; }</style>
"##).unwrap());
    mount_stores(
        &mut ui,
        source("tracks"),
        "tracks",
        &[("hp", json!(100)), ("mode", json!("fade"))],
    );
    let batch = |ui: &Ui| {
        ui.world_frame()
            .batches
            .iter()
            .find(|b| b.template.size.x == 46.0)
            .unwrap()
            .clone()
    };
    let ghost = |ui: &Ui| batch(ui).instances[0].aux[2];
    run(&mut ui, 2, |_| {});
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let b = batch(&ui);
    assert_eq!(b.template.size, Vec2::new(46.0, 5.0));
    assert_eq!(b.template.distance, Vec3::new(12.0, 0.7, 1.0));
    assert_eq!(b.template.max_distance, 80.0);
    assert_eq!(
        b.template.occlusion,
        og_draw::UiOcclusion::Fade { alpha: 0.35 }
    );
    assert_eq!(b.template.quads(), 6);
    assert_eq!(b.depth_sort, Some(view().forward));
    let style = b.template.bar.as_ref().unwrap();
    assert_eq!(style.min_size, Vec2::new(30.0, 3.0));
    assert_eq!(style.shield, Srgba::rgb(127, 224, 255).linear());
    assert_eq!(style.resource, Srgba::rgb(110, 168, 255).linear());
    assert_eq!(style.low, Srgba::rgb(255, 77, 94).linear());
    assert_eq!(style.low_at, 0.25);
    assert!((style.rim[3] - 0.28).abs() < 0.005);
    assert_eq!(b.instances[0].aux[3], 0.3);
    assert_eq!(b.instances[0].velocity[2..], [5.0, 0.7]);
    ui.set_theme(Theme {
        text_scale: 1.5,
        ..*ui.theme()
    });
    run(&mut ui, 1, |_| {});
    assert_eq!(
        batch(&ui).template.size,
        b.template.size,
        "bars ignore text scale"
    );
    assert_eq!(batch(&ui).instances[0].scale, b.instances[0].scale);

    set_store(&mut ui, "hp", json!(60));
    run(&mut ui, 1, |_| {});
    assert_eq!(batch(&ui).instances[0].value, 0.6);
    assert_eq!(ghost(&ui), 1.0);
    run(&mut ui, 24, |_| {});
    assert_eq!(ghost(&ui), 1.0, "400ms hold at 60Hz");
    run(&mut ui, 9, |_| {});
    assert!(
        (ghost(&ui) - 0.8).abs() < 0.001,
        "halfway through 300ms drain"
    );
    let saved = (ui.save_state(), ui.save_world_state());
    run(&mut ui, 9, |_| {});
    let replay = ui.save_world_state();
    assert_eq!(ghost(&ui), 0.6);
    ui.restore_state(&saved.0).unwrap();
    ui.restore_world_state(&saved.1).unwrap();
    run(&mut ui, 9, |_| {});
    assert_eq!(ui.save_world_state(), replay);

    set_store(&mut ui, "mode", json!("hide"));
    set_store(&mut ui, "hp", json!(25));
    run(&mut ui, 1, |_| {});
    assert_eq!(batch(&ui).template.occlusion, og_draw::UiOcclusion::Hide);
    assert_eq!(ghost(&ui), 0.6, "a second hit starts from previous health");
    let other = ui
        .world_frame()
        .batches
        .iter()
        .find(|b| b.template.size.x != 46.0)
        .unwrap();
    assert_eq!(
        other.instances[0].aux[2], 0.25,
        "disabled ghost follows health directly"
    );
    let mut overlay = Overlay::new(W, H, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    assert!(
        overlay
            .ui
            .mass
            .iter()
            .all(|b| b.layer < overlay.texts[0].layer),
        "text tags paint above mass bars"
    );
}
