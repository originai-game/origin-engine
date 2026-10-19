//! Every og-ui button role's label against the fill it is drawn on, in the
//! light and dark themes, at rest, hovered and pressed, on each ground a
//! button sits on (the screen's `bg`, a `bgSubtle` well, a `surface` panel,
//! a `surfaceRaised` dialog): WCAG 1.4.3's 4.5:1 for normal text, as the
//! platform's tokens test holds its own buttons (packages/tokens'
//! button-contrast test). Read from the frame og-ui draws: the label's text
//! command over the button's fill quad over the ground's quad, composited
//! in linear light as the renderer blends.
//!
//! Ways this could fail: a role's ink and fill tokens too close (the danger
//! button was red ink on a grey fill: 4.05:1 hovered on a dark dialog); a
//! translucent fill that reads on one ground only; a hovered or pressed
//! state darker or lighter than its rest state by enough to fall under;
//! the tokens' copy out of date with the platform's.

mod support;

use og_core::glam::Vec2;
use og_ui::*;
use support::*;

const ROLES: [(&str, ButtonVariant); 4] = [
    ("Primary", ButtonVariant::Primary),
    ("Secondary", ButtonVariant::Secondary),
    ("Quiet", ButtonVariant::Quiet),
    ("Danger", ButtonVariant::Danger),
];

fn luminance(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

fn ratio(a: [f32; 3], b: [f32; 3]) -> f32 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

/// `top` (linear, straight alpha) over the opaque linear `below`.
fn over(top: [f32; 4], below: [f32; 3]) -> [f32; 3] {
    [0, 1, 2].map(|i| top[i] * top[3] + below[i] * (1.0 - top[3]))
}

fn rgb(c: Srgba) -> [f32; 3] {
    let l = c.linear();
    [l[0], l[1], l[2]]
}

fn contains(r: &Rect, (x, y): (f32, f32)) -> bool {
    x >= r.x && x <= r.x + r.width && y >= r.y && y <= r.y + r.height
}

/// The four buttons, in a panel of `ground` (or on the screen when `None`).
fn build(ui: &mut Ui, ground: Option<Srgba>) {
    let buttons = |ui: &mut Ui| {
        for (name, variant) in ROLES {
            ui.button(
                name,
                name,
                ButtonDesc {
                    variant,
                    ..ButtonDesc::default()
                },
            );
        }
    };
    match ground {
        Some(bg) => ui.panel(
            "G",
            PanelDesc {
                background: Some(bg),
                ..PanelDesc::default()
            },
            buttons,
        ),
        None => buttons(ui),
    }
}

/// The label's colour over its button's fill over the ground, from the
/// frame: (label, what it sits on), linear.
fn measured(ui: &Ui, label: &str, ground: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let items = &ui.frame().items;
    let (k, color, centre) = items
        .iter()
        .enumerate()
        .find_map(|(k, it)| match &it.cmd {
            DrawCmd::Text {
                text,
                color,
                bounds,
                ..
            } if text == label => Some((
                k,
                *color,
                (
                    bounds.x + bounds.width / 2.0,
                    bounds.y + bounds.height / 2.0,
                ),
            )),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no text {label} drawn"));
    // The last solid rectangle under the label before it: the button's fill
    // (or, for a quiet button at rest, the ground's own quad, or nothing).
    let mut under = ground;
    for it in &items[..k] {
        let quad = match &it.cmd {
            DrawCmd::Quad { rect, color, .. } => Some((rect, color)),
            DrawCmd::Image {
                rect,
                texture: None,
                color,
                ..
            } => Some((rect, color)),
            _ => None,
        };
        if let Some((rect, c)) = quad
            && contains(rect, centre)
        {
            under = over(*c, under);
        }
    }
    (over(color, under), under)
}

#[test]
fn every_button_role_reads_at_4_5_to_1_on_its_fill_in_both_themes_and_every_state() {
    let mut rows = Vec::new();
    let mut failing = Vec::new();
    for (theme_name, theme) in [("light", Theme::light()), ("dark", Theme::dark())] {
        let p = theme.palette;
        let grounds = [
            ("bg", None, p.bg),
            ("bgSubtle", Some(p.bg_subtle), p.bg_subtle),
            ("surface", Some(p.surface), p.surface),
            ("surfaceRaised", Some(p.surface_raised), p.surface_raised),
        ];
        for (ground_name, panel, ground) in grounds {
            for (label, _) in ROLES {
                for state in ["rest", "hovered", "pressed"] {
                    let mut scene = og_render::RenderScene::new();
                    let mut ui = ui_with_font(add_test_font(&mut scene));
                    ui.set_theme(theme);
                    let vp = Viewport::new(800.0, 600.0);
                    let step = |ui: &mut Ui, input: InputFrame| {
                        ui.begin(input, vp);
                        build(ui, panel);
                        ui.end();
                    };
                    step(&mut ui, InputFrame::default());
                    let path = if panel.is_some() {
                        format!("G/{label}")
                    } else {
                        label.to_string()
                    };
                    let (x, y) = ui
                        .frame()
                        .element(&path)
                        .unwrap_or_else(|| panic!("no button {path}"))
                        .center();
                    let at = Vec2::new(x, y);
                    let hover = InputFrame {
                        pointer: at,
                        pointer_inside: true,
                        ..InputFrame::default()
                    };
                    match state {
                        "hovered" => {
                            step(&mut ui, hover.clone());
                            step(&mut ui, hover);
                        }
                        "pressed" => {
                            step(
                                &mut ui,
                                InputFrame {
                                    down: true,
                                    pressed: true,
                                    ..hover.clone()
                                },
                            );
                            step(
                                &mut ui,
                                InputFrame {
                                    down: true,
                                    ..hover
                                },
                            );
                        }
                        _ => step(&mut ui, InputFrame::default()),
                    }
                    let (ink, fill) = measured(&ui, label, rgb(ground));
                    let r = ratio(ink, fill);
                    rows.push(format!(
                        "{theme_name:5} {label:9} {state:7} on {ground_name:13} {r:5.2}:1"
                    ));
                    if r < 4.5 {
                        failing.push(format!(
                            "{theme_name} {label} {state} on {ground_name}: {r:.2}:1"
                        ));
                    }
                }
            }
        }
    }
    println!("{}", rows.join("\n"));
    // 2 themes × 4 grounds × 4 roles × 3 states.
    assert_eq!(rows.len(), 96);
    assert!(failing.is_empty(), "under 4.5:1:\n{}", failing.join("\n"));
}
