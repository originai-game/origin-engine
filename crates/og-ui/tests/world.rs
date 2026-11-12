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
use og_core::glam::{Quat, Vec2, Vec3};
use og_draw::{Overlay, UiMassInstance, UiMassKind, UiMassSpace, UiMassTemplate};
use og_ui::prelude::*;
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

fn tag(ui: &mut Ui, id: &str, desc: WorldAnchorDesc) -> AnchorInfo {
    ui.world_anchor(id, desc, |ui, _| {
        ui.label("Name", "Brenna", LabelDesc::default())
    })
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

#[test]
fn a_world_panel_takes_a_press_through_the_pointer_ray() {
    let mut ui = ui();
    let desc = WorldPanelDesc {
        target: None,
        width: 480.0,
        height: 320.0,
        quad: Some((
            WorldTarget::Point(Vec3::ZERO),
            Quat::IDENTITY,
            Vec2::new(3.0, 2.0),
        )),
        ..WorldPanelDesc::default()
    };
    let tick = |ui: &mut Ui, i: InputFrame| {
        ui.begin(i, vp());
        ui.set_world_view(Some(view()));
        let mut clicked = false;
        let info = ui.world_panel("Terminal", desc.clone(), |ui, _| {
            ui.label("Title", "Terminal", LabelDesc::default());
            clicked = ui.button("Run", "Run", ButtonDesc::default()).clicked;
        });
        ui.end();
        (info, clicked)
    };
    tick(&mut ui, InputFrame::default());
    // Where the button is on the panel, as a world point, on the screen.
    let (bx, by, bw, bh) = rect(&ui, "Terminal/Run");
    let (px, py, ..) = rect(&ui, "Terminal");
    let u = (bx + bw / 2.0 - px) / 480.0;
    let v = (by + bh / 2.0 - py) / 320.0;
    let p = Vec3::new((u - 0.5) * 3.0, (0.5 - v) * 2.0, 0.0);
    let (sx, sy, _) = view().project(p, W, H).unwrap();
    let at = Vec2::new(sx, sy);
    let (info, _) = tick(&mut ui, input(at, true, true, false));
    assert!(info.pointed, "{info:?}");
    assert!((info.x - u * 480.0).abs() < 1.0 && (info.y - v * 320.0).abs() < 1.0);
    let (_, clicked) = tick(&mut ui, input(at, false, false, true));
    assert!(clicked, "the panel's button took no press");
    // Off the quad: nothing.
    let off = Vec2::new(40.0, 40.0);
    tick(&mut ui, input(off, true, true, false));
    let (info, clicked) = tick(&mut ui, input(off, false, false, true));
    assert!(!clicked && !info.pointed);
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

/// §15.1: a HUD plus 2,000 mass elements and 100 anchored elements in
/// ≤ 1 ms of CPU a tick (build, lay out, place and record). A measurement
/// for the reference machines, not the orb: `cargo test -p og-ui --release
/// --test tests world::perf -- --ignored --nocapture`.
#[test]
#[ignore = "perf: run on the reference machines in release"]
fn perf_hud_2000_mass_and_100_anchors_within_a_millisecond() {
    let mut ui = ui();
    let h = ui.define_mass("damage", numbers(), UiMassSpace::World);
    let mut times = Vec::new();
    for tick in 0..240u32 {
        let start = std::time::Instant::now();
        ui.begin(InputFrame::default(), vp());
        ui.set_world_view(Some(view()));
        ui.panel("Hud", PanelDesc::default(), |ui| {
            for k in 0..12 {
                ui.label(format!("Stat{k}"), "Health 100", LabelDesc::default());
            }
            ui.button("Menu", "Menu", ButtonDesc::default());
        });
        // About 2,000 alive: 42 a tick over a 48-tick life.
        for i in 0..42 {
            ui.spawn_mass(
                h,
                UiMassInstance {
                    position: [(i as f32 - 21.0) * 0.2, 1.0, -((tick % 10) as f32)],
                    value: (tick * 42 + i) as f32,
                    color: [1.0; 4],
                    scale: 1.0,
                    ..Default::default()
                },
            );
        }
        for k in 0..100 {
            let a = k as f32 * 0.37;
            tag(
                &mut ui,
                &format!("Plate{k}"),
                WorldAnchorDesc {
                    target: WorldTarget::Point(Vec3::new(
                        og_core::math::cos(a) * 4.0,
                        1.0,
                        og_core::math::sin(a) * 4.0 - 4.0,
                    )),
                    declutter: Declutter::Stack,
                    occlusion: WorldOcclusion::Fade { alpha: 0.3 },
                    scale: AnchorScale::Distance {
                        reference: 5.0,
                        min: 0.5,
                        max: 1.5,
                    },
                    priority: k % 3,
                    ..WorldAnchorDesc::default()
                },
            );
        }
        ui.end();
        let mut o = Overlay::new(W, H, 1.0);
        ui.frame().emit(
            &EmitDesc {
                layer: 1000,
                time: 0.0,
            },
            &mut o,
        );
        ui.world_frame().emit(1000, &mut o);
        if tick >= 48 {
            times.push(start.elapsed().as_secs_f64() * 1000.0);
        }
    }
    assert!(ui.mass_count(h) >= 1900, "{}", ui.mass_count(h));
    times.sort_by(f64::total_cmp);
    let median = times[times.len() / 2];
    let p95 = times[times.len() * 95 / 100];
    println!("world perf: median {median:.3} ms, p95 {p95:.3} ms a tick");
    assert!(median <= 1.0, "median {median:.3} ms a tick");
}
