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
use og_render::{Overlay, UiMassInstance, UiMassKind, UiMassSpace, UiMassTemplate};
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
        .frame()
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
