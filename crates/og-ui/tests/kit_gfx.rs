//! The game widget set rendered headless and compared with
//! `references/<tier>/widget-set.png` (og-tests' tolerance, as `tests/gfx.rs`;
//! `OG_UI_CAPTURE=1` records). Covers what only drawing shows: cooldown pies
//! made of rotated half discs at 0, a quarter, past half and full; checkbox
//! ticks and chevrons as rotated bars; meter trails, notches and labels;
//! focus rings with their gap; the d-pad, stick and action buttons; a dialog
//! over a scrim with an open select menu over it; a toast.

mod support;

use og_core::glam::{Vec2, Vec3};
use og_render::{Background, RenderDesc, RenderScene, RenderTarget, Renderer, Tonemap};
use og_ui::*;
use std::path::PathBuf;
use support::*;

#[derive(Default)]
struct Kit {
    hp: f32,
    tab: usize,
    pick: usize,
    amount: f32,
    mode: usize,
    checks: [bool; 2],
}

fn kit_screen(ui: &mut Ui, s: &mut Kit) {
    let t = *ui.theme();
    ui.row(
        "Screen",
        Element {
            width: Size::grow(),
            height: Size::grow(),
            padding: Insets::all(t.space.lg),
            gap: t.space.lg,
            background: Some(t.palette.bg),
            ..Element::default()
        },
        |ui| {
            ui.panel(
                "Left",
                PanelDesc {
                    width: Size::Fixed(380.0),
                    height: Size::grow(),
                    ..PanelDesc::default()
                },
                |ui| {
                    ui.tabs(
                        "Tabs",
                        &["Overview", "Stats", "Gear"],
                        &mut s.tab,
                        TabsDesc {
                            fill: true,
                            ..TabsDesc::default()
                        },
                    );
                    ui.row(
                        "Pickers",
                        Element {
                            gap: t.space.md,
                            cross: Cross::Center,
                            ..Element::default()
                        },
                        |ui| {
                            ui.select(
                                "Difficulty",
                                &["Easy", "Normal", "Hard"],
                                &mut s.pick,
                                &SelectDesc::default(),
                            );
                            ui.stepper(
                                "Amount",
                                &mut s.amount,
                                StepperDesc {
                                    min: 0.0,
                                    max: 9.0,
                                    ..StepperDesc::default()
                                },
                            );
                        },
                    );
                    ui.radio(
                        "Mode",
                        &["Solo", "Co-op", "Versus"],
                        &mut s.mode,
                        RadioDesc { horizontal: true },
                    );
                    ui.checkbox_list("Options", &["Show hints", "Hard mode"], &mut s.checks);
                    ui.row(
                        "Chips",
                        Element {
                            gap: t.space.sm,
                            cross: Cross::Center,
                            ..Element::default()
                        },
                        |ui| {
                            ui.chip(
                                "All",
                                "All",
                                ChipDesc {
                                    selected: true,
                                    ..ChipDesc::default()
                                },
                            );
                            ui.chip("Weapons", "Weapons", ChipDesc::default());
                            ui.badge("New", "New", Tone::Accent);
                            ui.badge("Sale", "-20%", Tone::Success);
                        },
                    );
                    ui.divider("Rule", false);
                    ui.meter(
                        "Health",
                        s.hp,
                        100.0,
                        &MeterDesc {
                            label: format!("{} / 100", s.hp),
                            ..MeterDesc::default()
                        },
                    );
                    ui.meter(
                        "Xp",
                        620.0,
                        1000.0,
                        &MeterDesc {
                            kind: MeterKind::Xp,
                            segments: 5,
                            ..MeterDesc::default()
                        },
                    );
                    ui.skeleton(
                        "Skeleton",
                        SkeletonDesc {
                            lines: 2,
                            ..SkeletonDesc::default()
                        },
                    );
                },
            );
            ui.column(
                "Right",
                Element {
                    gap: t.space.lg,
                    ..Element::default()
                },
                |ui| {
                    ui.row(
                        "Rings",
                        Element {
                            gap: t.space.md,
                            ..Element::default()
                        },
                        |ui| {
                            for (k, rem) in [1.0, 0.75, 0.4, 0.0].into_iter().enumerate() {
                                ui.cooldown(
                                    ui_idi("Ring", k as u32),
                                    rem,
                                    &CooldownDesc {
                                        label: format!("{}", k + 1),
                                        ..CooldownDesc::default()
                                    },
                                );
                            }
                        },
                    );
                    ui.row(
                        "Touch",
                        Element {
                            gap: t.space.lg,
                            cross: Cross::End,
                            ..Element::default()
                        },
                        |ui| {
                            ui.joystick("Stick", JoystickDesc::default());
                            ui.dpad("Dpad", DpadDesc::default());
                            ui.action_button(
                                "Jump",
                                "Jump",
                                &ActionButtonDesc {
                                    primary: true,
                                    ..ActionButtonDesc::default()
                                },
                            );
                        },
                    );
                    ui.spinner("Spinner", 0.0);
                },
            );
        },
    );
}

#[test]
fn the_widget_set_matches_its_reference_frame() {
    let vp = Viewport {
        width: 900.0,
        height: 520.0,
        dpr: 1.5,
        ..Viewport::default()
    };
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    let atlas = scene.add_texture(atlas_texture());
    let mut ui = ui_with_font(font);
    let mut s = Kit {
        hp: 100.0,
        amount: 3.0,
        checks: [true, false],
        pick: 1,
        ..Kit::default()
    };
    let step = |ui: &mut Ui, s: &mut Kit, input: InputFrame| {
        ui.begin(input, vp);
        kit_screen(ui, s);
        ui.end();
    };
    step(&mut ui, &mut s, InputFrame::default());
    // A hit: the meter's trail shows the old value for a beat.
    s.hp = 55.0;
    for _ in 0..4 {
        step(&mut ui, &mut s, InputFrame::default());
    }
    // Open the select with a press: its menu floats over the panel and
    // takes focus; Tab moves it to Hard with the ring and its gap.
    let sel = ui.frame().element("Difficulty").expect("select").clone();
    let (x, y) = sel.center();
    let p = Vec2::new(x, y);
    step(
        &mut ui,
        &mut s,
        InputFrame {
            pointer: p,
            pointer_inside: true,
            down: true,
            pressed: true,
            ..InputFrame::default()
        },
    );
    step(
        &mut ui,
        &mut s,
        InputFrame {
            pointer: p,
            pointer_inside: true,
            released: true,
            ..InputFrame::default()
        },
    );
    step(
        &mut ui,
        &mut s,
        InputFrame {
            keys: vec![UiKey::Tab],
            ..InputFrame::hover(p.x, p.y)
        },
    );
    ui.toast(
        "Achievement unlocked",
        ToastDesc {
            tone: Tone::Success,
            ..ToastDesc::default()
        },
    );
    for _ in 0..20 {
        step(&mut ui, &mut s, InputFrame::hover(p.x, p.y));
    }
    let frame = ui.frame();
    assert!(frame.warnings.is_empty(), "{:?}", frame.warnings);
    assert!(
        frame.element("Difficulty.menu/Option#2").is_some(),
        "the menu is open"
    );
    assert!(frame.element("Toasts/Toast#1").is_some(), "the toast is up");

    // The pies: done 0.25 is the right half and a track half turned a
    // quarter; done 0.6 is the right half and a fill half turned 0.6 turn
    // less a half (the rings of 1 and 0 remaining draw no halves).
    let mut angles: Vec<f32> = frame
        .items
        .iter()
        .filter_map(|i| match i.cmd {
            DrawCmd::Half { angle, .. } => Some(angle),
            _ => None,
        })
        .collect();
    angles.sort_by(f32::total_cmp);
    let pi = std::f32::consts::PI;
    let expected = [0.0, 0.0, 0.2 * pi, 0.5 * pi];
    assert!(
        angles.len() == 4
            && angles
                .iter()
                .zip(expected)
                .all(|(a, e)| (a - e).abs() < 1e-4),
        "half-disc angles {angles:?}, expected {expected:?}"
    );

    let mut overlay = og_render::Overlay::new(vp.width, vp.height, vp.dpr);
    frame.emit(&EmitDesc { atlas, layer: 0 }, &mut overlay);
    scene.overlay = Some(overlay);
    let bg = ui.theme().palette.bg.linear();
    scene.environment.background = Background::Color(Vec3::new(bg[0], bg[1], bg[2]));
    scene.post.tonemap = Tonemap::None;
    let desc = RenderDesc {
        width: (vp.width * vp.dpr) as u32,
        height: (vp.height * vp.dpr) as u32,
        ..RenderDesc::default()
    };
    let mut renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
    let stats = renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    let image = renderer.read_offscreen().expect("read back");
    let tier = renderer.info().backend_tier();
    let actual = out_dir().join(format!("widget-set-{tier}.png"));
    image::save_buffer(
        &actual,
        &image.pixels,
        image.width,
        image.height,
        image::ColorType::Rgba8,
    )
    .expect("write frame");
    println!(
        "sprites {} glyphs {} sprite draws {} levels {} missing glyphs {}",
        stats.sprites,
        stats.glyphs,
        stats.sprite_draws,
        frame.levels(),
        stats.missing_glyphs
    );
    assert_eq!(stats.missing_glyphs, 0);

    let dir = |tier: &str| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("references")
            .join(tier)
    };
    let own = dir(&tier).join("widget-set.png");
    if std::env::var_os("OG_UI_CAPTURE").is_some() {
        std::fs::create_dir_all(own.parent().expect("dir")).expect("reference dir");
        std::fs::copy(&actual, &own).expect("record reference");
        println!("recorded {}", own.display());
    }
    let reference = if own.exists() {
        own
    } else {
        println!(
            "no {tier} reference; comparing with the metal one. Record it with OG_UI_CAPTURE=1 on that tier"
        );
        dir("metal").join("widget-set.png")
    };
    let expected = image::open(&reference)
        .unwrap_or_else(|e| panic!("{}: {e} (OG_UI_CAPTURE=1 records it)", reference.display()))
        .to_rgba8();
    assert_eq!(
        (expected.width(), expected.height()),
        (image.width, image.height)
    );
    match compare(&image.pixels, expected.as_raw()) {
        Ok((rmse, outliers)) => println!(
            "{tier}: RMSE {rmse:.5}, outliers {:.3}% against {}",
            outliers * 100.0,
            reference.display()
        ),
        Err(e) => panic!(
            "{} differs from {}: {e}",
            actual.display(),
            reference.display()
        ),
    }
}
