//! Every og-ui button role's label against the fill it is drawn on, in the
//! light and dark themes, at rest, hovered and pressed, on each ground a
//! button sits on (the screen's `backdrop`, a `well` well, a `panel` panel,
//! a `panelRaised` dialog): live text holds 3:1, an error under it (4.5:1,
//! WCAG 1.4.3's, is what the art director judges toward, docs/ui.md §6;
//! each ratio is printed). Read from the frame og-ui draws: the label's text
//! command over the button's fill quad over the ground's quad, composited
//! in linear light as the renderer blends.
//!
//! Ways this could fail: a role's ink and fill tokens too close (the danger
//! button was red ink on a grey fill: 4.05:1 hovered on a dark dialog); a
//! translucent fill that reads on one ground only; a hovered or pressed
//! state darker or lighter than its rest state by enough to fall under.

use crate::support;

use og_core::glam::Vec2;
use og_ui::*;
use support::*;

/// Each role's button id (its label) and its `variant`.
const ROLES: [(&str, &str); 4] = [
    ("Primary", "primary"),
    ("Secondary", "secondary"),
    ("Quiet", "quiet"),
    ("Danger", "danger"),
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

/// `c` as a CSS colour.
fn css(c: Srgba) -> String {
    let [r, g, b] = c.rgb;
    format!("rgb({r} {g} {b} / {})", c.alpha)
}

/// The four buttons, in a panel of `ground` (or on the screen when `None`).
fn buttons(ground: Option<Srgba>) -> String {
    let buttons: String = ROLES
        .iter()
        .map(|(name, variant)| {
            format!(r#"<button id="{name}" variant="{variant}">{name}</button>"#)
        })
        .collect();
    let body = match ground {
        Some(bg) => format!(
            r#"<panel id="G" style="background: {}">{buttons}</panel>"#,
            css(bg)
        ),
        None => buttons,
    };
    format!("<meta name=\"component\" content=\"t\">\n<column>{body}</column>")
}

/// A UI in `theme` with the drawing test font.
fn themed(theme: Theme) -> Ui {
    let mut ui = ui_with_font(test_font());
    ui.set_theme(theme);
    ui
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
        if let Some((rect, c)) = it.cmd.filled()
            && contains(&rect, centre)
        {
            under = over(c, under);
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
            ("bg", None, p.backdrop),
            ("bgSubtle", Some(p.well), p.well),
            ("surface", Some(p.panel), p.panel),
            ("surfaceRaised", Some(p.panel_raised), p.panel_raised),
        ];
        for (ground_name, panel, ground) in grounds {
            for (label, _) in ROLES {
                for state in ["rest", "hovered", "pressed"] {
                    let mut d = Doc::with_ui(themed(theme), &buttons(panel));
                    d.viewport = Viewport::new(800.0, 600.0);
                    let step = |d: &mut Doc, input: InputFrame| d.step(input);
                    step(&mut d, InputFrame::default());
                    let (x, y) = d.get(label).center();
                    let at = Vec2::new(x, y);
                    let hover = InputFrame {
                        pointer: at,
                        pointer_inside: true,
                        ..InputFrame::default()
                    };
                    match state {
                        "hovered" => {
                            step(&mut d, hover.clone());
                            step(&mut d, hover);
                        }
                        "pressed" => {
                            step(
                                &mut d,
                                InputFrame {
                                    down: true,
                                    pressed: true,
                                    ..hover.clone()
                                },
                            );
                            step(
                                &mut d,
                                InputFrame {
                                    down: true,
                                    ..hover
                                },
                            );
                        }
                        _ => step(&mut d, InputFrame::default()),
                    }
                    let (ink, fill) = measured(&d.ui, label, rgb(ground));
                    let r = ratio(ink, fill);
                    rows.push(format!(
                        "{theme_name:5} {label:9} {state:7} on {ground_name:13} {r:5.2}:1"
                    ));
                    if r < 3.0 {
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
    assert!(failing.is_empty(), "under 3:1:\n{}", failing.join("\n"));
}

/// Every tab's label against what it sits on, in both themes, once the
/// selected tab's ground has slid in: the selected one on `knob`, the
/// others on the track. Ways this could fail: the selected label drawn in
/// `ink` on a `knob` as light as it (the dark theme's: 1:1, a blank white
/// pill on the kit's settings tabs); the others in `inkMuted` on a dark
/// track that the renderer's linear blend makes lighter than it looks
/// (3.95:1).
#[test]
fn every_tab_label_reads_at_3_to_1_selected_or_not_in_both_themes() {
    let mut failing = Vec::new();
    for (theme_name, theme) in [("light", Theme::light()), ("dark", Theme::dark())] {
        let p = theme.palette;
        let mut d = Doc::with_ui(
            themed(theme),
            &format!(
                r#"<meta name="component" content="t">
<column><panel id="G" style="background: {}"><tabs id="Tabs" items="Audio Graphics Controls" selected="1"></tabs></panel></column>"#,
                css(p.panel_raised)
            ),
        );
        d.viewport = Viewport::new(800.0, 600.0);
        for _ in 0..120 {
            d.step(InputFrame::default());
        }
        for label in ["Audio", "Graphics", "Controls"] {
            let (ink, fill) = measured(&d.ui, label, rgb(p.panel_raised));
            let r = ratio(ink, fill);
            if r < 3.0 {
                failing.push(format!("{theme_name} {label}: {r:.2}:1"));
            } else if r < 4.5 {
                println!("warning: {theme_name} {label} under 4.5:1: {r:.2}:1");
            }
        }
    }
    assert!(failing.is_empty(), "under 3:1:\n{}", failing.join("\n"));
}
