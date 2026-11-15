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
    // A press on nothing takes the focus away: a focused control is
    // built every tick, wherever it stands.
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
    // Laid out: the layer's root alone, as every layer each tick.
    assert_eq!(
        (idle.bindings, idle.reconciled, idle.laid_out),
        (0, 0, 1),
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
        (0, 0, 1),
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
