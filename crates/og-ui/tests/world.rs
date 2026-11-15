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
