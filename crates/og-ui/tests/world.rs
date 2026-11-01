//! World-space UI (`docs/ui.md` §14, `src/world.rs`), without a renderer:
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
use og_render::{
    Overlay, UiGeometry, UiMassInstance, UiMassKind, UiMassSpace, UiMassTemplate, UiPaint,
};
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

#[test]
fn an_anchor_sits_on_its_projected_point_and_scales_with_distance() {
    let mut ui = ui();
    let at = |p: Vec3, scale: AnchorScale| WorldAnchorDesc {
        target: WorldTarget::Point(p),
        scale,
        pivot: Anchor::Bottom,
        ..WorldAnchorDesc::default()
    };
    let distance = AnchorScale::Distance {
        reference: 5.0,
        min: 0.1,
        max: 4.0,
    };
    run(&mut ui, 8, |ui| {
        tag(ui, "Near", at(Vec3::ZERO, AnchorScale::Constant));
        tag(ui, "Far", at(Vec3::new(2.0, 0.0, -5.0), distance));
        tag(
            ui,
            "Same",
            at(Vec3::new(2.0, 0.0, -5.0), AnchorScale::Constant),
        );
    });
    // The origin projects to the screen's centre: the tag's bottom centre.
    let (x, y, w, h) = rect(&ui, "Near");
    assert!((x + w / 2.0 - W / 2.0).abs() < 1.0, "x {x} w {w}");
    assert!((y + h - H / 2.0).abs() < 1.0, "y {y} h {h}");
    // At 10 m with 1 at 5 m: half the constant one's size.
    let (_, _, fw, fh) = rect(&ui, "Far");
    let (sx, _, sw, sh) = rect(&ui, "Same");
    assert!((fw - sw / 2.0).abs() < 0.5 && (fh - sh / 2.0).abs() < 0.5);
    // Right of the centre, nearer it than the near tag's offset would be.
    let p = view().project(Vec3::new(2.0, 0.0, -5.0), W, H).unwrap();
    assert!((sx + sw / 2.0 - p.0).abs() < 1.0, "{sx} {sw} {p:?}");
}

#[test]
fn a_contained_tag_slides_in_from_the_edge_and_fades_as_its_point_leaves() {
    let mut ui = ui();
    let at = |x: f32, contain: bool| WorldAnchorDesc {
        target: WorldTarget::Point(Vec3::new(x, 0.0, 0.0)),
        contain,
        ..WorldAnchorDesc::default()
    };
    let mut gone = AnchorInfo::default();
    run(&mut ui, 30, |ui| {
        // Its point on the screen, a few px in from the right edge.
        tag(ui, "Cut", at(4.12, false));
        tag(ui, "In", at(4.12, true));
        // Its point well off the screen to the right.
        gone = tag(ui, "Gone", at(9.0, true));
    });
    let p = view().project(Vec3::new(4.12, 0.0, 0.0), W, H).unwrap();
    assert!(p.0 < W, "the point is on the screen: {p:?}");
    let (cx, _, cw, _) = rect(&ui, "Cut");
    let (ix, _, iw, _) = rect(&ui, "In");
    // The frame reports an element's box cut to the screen.
    assert!(cw < iw - 10.0, "uncontained it is cut: {cx} + {cw}");
    assert!(
        (ix + iw - W).abs() < 1.0,
        "contained it ends at the edge, whole: {ix} + {iw}"
    );
    assert!(
        gone.alpha < 0.01,
        "its point off the screen, it has faded: {gone:?}"
    );
}

#[test]
fn crowded_anchors_resolve_by_priority() {
    let overlap = |a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)| {
        a.0 < b.0 + b.2 && b.0 < a.0 + a.2 && a.1 < b.1 + b.3 && b.1 < a.1 + a.3
    };
    for mode in [Declutter::Shift, Declutter::Stack] {
        let mut ui = ui();
        run(&mut ui, 40, |ui| {
            for (k, id) in ["A", "B", "C"].iter().enumerate() {
                tag(
                    ui,
                    id,
                    WorldAnchorDesc {
                        target: WorldTarget::Point(Vec3::ZERO),
                        declutter: mode,
                        priority: 3 - k as i32,
                        ..WorldAnchorDesc::default()
                    },
                );
            }
        });
        let (a, b, c) = (rect(&ui, "A"), rect(&ui, "B"), rect(&ui, "C"));
        assert!(
            !overlap(a, b) && !overlap(a, c) && !overlap(b, c),
            "{mode:?}: {a:?} {b:?} {c:?}"
        );
        // The first keeps its place on the point.
        assert!((a.1 + a.3 - H / 2.0).abs() < 1.0, "{mode:?}: {a:?}");
    }
    // Hide: the lower one fades out, the higher stays.
    let mut ui = ui();
    let mut infos = (AnchorInfo::default(), AnchorInfo::default());
    run(&mut ui, 12, |ui| {
        let d = |p| WorldAnchorDesc {
            target: WorldTarget::Point(Vec3::ZERO),
            declutter: Declutter::Hide,
            priority: p,
            ..WorldAnchorDesc::default()
        };
        infos = (tag(ui, "Low", d(1)), tag(ui, "High", d(2)));
    });
    assert_eq!(infos.0.alpha, 0.0, "{:?}", infos.0);
    assert_eq!(infos.1.alpha, 1.0, "{:?}", infos.1);
}

#[test]
fn a_waypoint_behind_the_camera_clamps_to_the_edge_it_must_turn_to() {
    let mut ui = ui();
    let mut info = AnchorInfo::default();
    run(&mut ui, 3, |ui| {
        info = tag(
            ui,
            "Waypoint",
            WorldAnchorDesc {
                // Behind the eye, to the right.
                target: WorldTarget::Point(Vec3::new(3.0, 0.0, 12.0)),
                clamp: Some(EdgeClamp::default()),
                ..WorldAnchorDesc::default()
            },
        );
    });
    assert!(info.clamped, "{info:?}");
    let (x, y, w, h) = rect(&ui, "Waypoint");
    // On the right edge, inside the margin, its arrow's room beside it.
    let right = x + w;
    assert!(
        right <= W - 24.0 + 0.5 && right > W - 24.0 - 30.0,
        "{x} {w}"
    );
    assert!(y >= 24.0 && y + h <= H - 24.0);
    // The arrow: a triangle right of the box, pointing right.
    let arrow = ui.frame().items.iter().find_map(|i| match &i.cmd {
        DrawCmd::Shape { shape, .. } => match shape.geometry {
            UiGeometry::Polygon { sides: 3, .. } => Some(shape.center),
            _ => None,
        },
        _ => None,
    });
    let a = arrow.expect("an arrow");
    assert!(a.x > right && a.x < W, "{a:?}");
    // In front and on the screen: not clamped.
    let mut ui = crate::world::ui();
    run(&mut ui, 2, |ui| {
        info = tag(
            ui,
            "Waypoint",
            WorldAnchorDesc {
                target: WorldTarget::Point(Vec3::ZERO),
                clamp: Some(EdgeClamp::default()),
                ..WorldAnchorDesc::default()
            },
        );
    });
    assert!(!info.clamped && info.on_screen, "{info:?}");
}

#[test]
fn an_occluded_anchor_asks_for_a_depth_test_at_its_point() {
    let mut ui = ui();
    run(&mut ui, 6, |ui| {
        tag(
            ui,
            "Tag",
            WorldAnchorDesc {
                target: WorldTarget::Point(Vec3::new(0.0, 0.0, -3.0)),
                occlusion: WorldOcclusion::Fade { alpha: 0.25 },
                element: Element {
                    background: Some(Srgba::rgb(20, 20, 30)),
                    padding: Insets::all(6.0),
                    ..Element::column()
                },
                ..WorldAnchorDesc::default()
            },
        );
    });
    let mut o = Overlay::new(W, H, 1.0);
    ui.frame().emit(
        &EmitDesc {
            layer: 1000,
            time: 0.0,
        },
        &mut o,
    );
    assert!(!o.ui.occluders.is_empty(), "no occluder");
    let occ = o.ui.occluders[0];
    assert!((occ.depth - 8.0).abs() < 1e-3, "{occ:?}");
    assert!((occ.at - Vec2::new(W / 2.0, H / 2.0)).length() < 1.0);
    assert_eq!(occ.alpha, 0.25);
    // The tag's background uses the occluded clip rectangle.
    assert!(
        o.ui.shapes.iter().any(|t| t.clip == Some(occ.rect)),
        "the tag is not under the depth test"
    );
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

/// The particles' mass batches in a frame's items.
fn particle_batches(ui: &Ui) -> Vec<og_render::UiMassBatch> {
    ui.frame()
        .items
        .iter()
        .filter_map(|i| match &i.cmd {
            DrawCmd::World(w) => match &**w {
                WorldCmd::Mass { batch, .. } => Some((**batch).clone()),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

#[test]
fn particles_and_the_burst_preset_emit_seeded_mass_elements() {
    let build = |reduced: bool| {
        let mut ui = ui();
        for k in 0..3 {
            let v = Viewport {
                reduced_motion: reduced,
                ..vp()
            };
            ui.begin(InputFrame::default(), v);
            ui.button("Claim", "Claim", ButtonDesc::default());
            ui.particles(ParticlesDesc {
                fire: k == 0,
                ..ParticlesDesc::confetti(50)
            });
            ui.button("Buy", "Buy", ButtonDesc::default());
            ui.burst(BurstDesc {
                count: 6,
                ..BurstDesc::default()
            });
            ui.end();
        }
        ui
    };
    let a = build(false);
    let batches = particle_batches(&a);
    let counts: Vec<usize> = batches.iter().map(|b| b.instances.len()).collect();
    assert_eq!(
        counts,
        [50, 12],
        "confetti, then the burst's rays and sparks"
    );
    assert!(
        batches
            .iter()
            .all(|b| matches!(b.template.kind, UiMassKind::Particle { .. }))
    );
    // Flung from the button: started on the first tick, from its centre.
    let (bx, by, bw, bh) = rect(&a, "Buy");
    let p = batches[1].instances[0].position;
    assert!((p[0] - (bx + bw / 2.0)).abs() < 0.5 && (p[1] - (by + bh / 2.0)).abs() < 0.5);
    // The same on another run, none under reduced motion.
    assert_eq!(particle_batches(&build(false)), batches);
    assert!(particle_batches(&build(true)).is_empty());
}

#[test]
fn a_meta_vignette_covers_the_screen_under_the_hud() {
    let mut ui = ui();
    run(&mut ui, 1, |ui| {
        ui.button("Hud", "Hud", ButtonDesc::default());
        ui.meta("Hurt", MetaDesc::default());
    });
    let items = &ui.frame().items;
    let vignette = items.iter().position(|i| match &i.cmd {
        DrawCmd::Shape { shape, .. } => {
            matches!(&shape.paint, UiPaint::Gradient(g) if g.kind == og_render::UiGradientKind::Radial)
                && shape.half == Vec2::new(W / 2.0, H / 2.0)
        }
        _ => false,
    });
    let v = vignette.expect("a full-screen radial vignette");
    // Under the HUD's text, though the HUD was built first.
    // The first command: under the HUD's button, though built after it.
    assert_eq!(v, 0, "the vignette draws over the HUD");
    assert!(items.len() > 1);
}

#[test]
fn a_viewport_hands_its_drag_and_ray_to_the_game() {
    let mut ui = ui();
    let desc = ViewportDesc {
        width: Size::Fixed(400.0),
        height: Size::Fixed(300.0),
        view: Some(view()),
        label: "Hero".into(),
        ..ViewportDesc::default()
    };
    let tick = |ui: &mut Ui, i: InputFrame| {
        ui.begin(i, vp());
        let r = ui.scene_viewport("Hero", desc.clone());
        ui.end();
        r
    };
    tick(&mut ui, InputFrame::default());
    let r = tick(&mut ui, InputFrame::default());
    let (x, y, w, h) = (r.rect.x, r.rect.y, r.rect.width, r.rect.height);
    assert_eq!((w, h), (400.0, 300.0));
    assert_eq!(r.pixels, (400, 300));
    let c = Vec2::new(x + w / 2.0, y + h / 2.0);
    let r = tick(&mut ui, input(c, true, true, false));
    let (_, d) = r.ray.expect("a ray");
    assert!((d - Vec3::NEG_Z).length() < 1e-3, "{d:?}");
    let r = tick(
        &mut ui,
        input(c + Vec2::new(12.0, -5.0), true, false, false),
    );
    assert!(r.pressed);
    assert_eq!(r.drag, Vec2::new(12.0, -5.0));
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
