//! button_contrast.rs for the style packs: every button role's label, in
//! every state, the three label inks (`fg`, `fgMuted`, `fgSubtle`) and a
//! text field's placeholder (`fgSubtle` on the field's frame) and typed
//! text, at rest and focused, as
//! og-ui draws them with a pack's skin, read against what is really under
//! them: the frame's draw commands composited in linear light, a skinned
//! piece as the mean of its atlas texels (the pack's `atlas.png`) times its
//! tint. Packs: the six of templates/ui-kit and Ember Vale's copy of the
//! fantasy pack (templates/archetypes/third-person, `ember`).
//!
//! Two bars. A label on a surface (a screen, a panel, a dialog) reaches
//! WCAG's 4.5:1 here, as og polish wants it in the frame. A button's label
//! reaches 7:1 (WCAG AAA) here: a 13 to 15 px label drawn by MSDF over a
//! textured, shaded fill reads about a third lower in the frame, where og
//! polish takes the ground's median and the ink's farthest percent. Ember
//! Vale's Set out read 5.3:1 by these commands and 3.36:1 in its polish
//! frame (owed row 48); with every primary and secondary label at 7:1 here
//! the lowest in its polish frames is 4.65:1
//! (tests/evidence/P4/web-host-tools/fantasy-contrast.md). The bars hold
//! for every pack (the fantasy family since the web host and tools round,
//! the other five since the UI shell round's palettes:
//! tests/evidence/P4/ui-shell/pack-contrast.md); every row is printed.
//!
//! Ways this could fail: a pack's accent or fill too dark for its ink once
//! the frame art shades it; a muted or subtle ink that reads on paper
//! only; a hovered or pressed state darker than its rest; a translucent
//! piece that reads over one ground only; a placeholder under 4.5:1 (the
//! fantasy pack's read 4.40:1 here and 2.93:1 in lantern-vale's polish
//! frame, FINDINGS #29; pixel 4.22, cartoon 4.13, before their fgSubtle
//! moved), or the focus ring showing through a focused field's frame
//! (scifi 3.52:1 and neon 3.00:1 on the ring's colour, before og-ui cut
//! the ring under every control).

mod support;

use og_core::glam::Vec2;
use og_render::{ColorSpace, Mips, RenderScene, TextureData, TextureHandle};
use og_ui::*;
use std::path::PathBuf;
use support::*;

const ROLES: [(&str, ButtonVariant); 4] = [
    ("Primary", ButtonVariant::Primary),
    ("Secondary", ButtonVariant::Secondary),
    ("Quiet", ButtonVariant::Quiet),
    ("Danger", ButtonVariant::Danger),
];

/// The bar for a primary or secondary button's label at rest and hovered
/// (what a still frame shows), and for every other label: pressed states,
/// quiet and danger buttons, the label inks on a panel.
const BUTTON_BAR: f32 = 7.0;
const LABEL_BAR: f32 = 4.5;

fn packs() -> Vec<(String, PathBuf)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates");
    let mut out: Vec<(String, PathBuf)> = ["pixel", "cartoon", "fantasy", "scifi", "paper", "neon"]
        .iter()
        .map(|n| (n.to_string(), root.join("ui-kit/assets/skins").join(n)))
        .collect();
    out.push((
        "ember".into(),
        root.join("archetypes/third-person/assets/skins/ember"),
    ));
    out
}

fn lin(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

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

/// A linear colour as `#rrggbb` (sRGB), for the printed rows.
fn hex(c: [f32; 3]) -> String {
    let e = |v: f32| {
        let s = if v <= 0.003_130_8 {
            v * 12.92
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        };
        (s.clamp(0.0, 1.0) * 255.0).round() as u8
    };
    format!("#{:02x}{:02x}{:02x}", e(c[0]), e(c[1]), e(c[2]))
}

fn rgb(c: Srgba) -> [f32; 3] {
    let l = c.linear();
    [l[0], l[1], l[2]]
}

fn contains(r: &Rect, (x, y): (f32, f32)) -> bool {
    x >= r.x && x <= r.x + r.width && y >= r.y && y <= r.y + r.height
}

/// The mean of the atlas texels under `uv`, linear with straight alpha.
fn texels(atlas: &image::RgbaImage, uv: [f32; 4]) -> [f32; 4] {
    let (w, h) = (atlas.width() as f32, atlas.height() as f32);
    let (x0, y0) = ((uv[0] * w).floor() as u32, (uv[1] * h).floor() as u32);
    let (x1, y1) = ((uv[2] * w).ceil() as u32, (uv[3] * h).ceil() as u32);
    let (mut sum, mut n) = ([0.0f32; 4], 0.0f32);
    for y in y0..y1.max(y0 + 1).min(atlas.height()) {
        for x in x0..x1.max(x0 + 1).min(atlas.width()) {
            let p = atlas.get_pixel(x, y).0;
            for k in 0..3 {
                sum[k] += lin(f32::from(p[k]) / 255.0);
            }
            sum[3] += f32::from(p[3]) / 255.0;
            n += 1.0;
        }
    }
    sum.map(|s| s / n.max(1.0))
}

/// The text `label` drawn and what is under its centre, from the frame:
/// (ink, ground), linear.
fn measured(
    ui: &Ui,
    label: &str,
    ground: [f32; 3],
    tex: TextureHandle,
    atlas: &image::RgbaImage,
) -> ([f32; 3], [f32; 3]) {
    let items = &ui.frame().items;
    // The last text drawn with the label: a pack's text shadow or glow is
    // drawn first, under it.
    let (k, color, centre) = items
        .iter()
        .enumerate()
        .rev()
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
    let mut under = ground;
    for it in &items[..k] {
        let piece = match &it.cmd {
            DrawCmd::Quad { rect, color, .. } => Some((rect, *color)),
            DrawCmd::Image {
                rect,
                texture: None,
                color,
                ..
            } => Some((rect, *color)),
            DrawCmd::Image {
                rect,
                texture: Some(t),
                uv,
                color,
                ..
            } if *t == tex => {
                let m = texels(atlas, *uv);
                Some((
                    rect,
                    [
                        m[0] * color[0],
                        m[1] * color[1],
                        m[2] * color[2],
                        m[3] * color[3],
                    ],
                ))
            }
            _ => None,
        };
        if let Some((rect, c)) = piece
            && contains(rect, centre)
        {
            under = over(c, under);
        }
    }
    (over(color, under), under)
}

fn atlas_handle() -> TextureHandle {
    RenderScene::new().add_texture(TextureData {
        width: 2,
        height: 2,
        rgba: vec![255; 16],
        color_space: ColorSpace::Srgb,
        mips: Mips::None,
    })
}

#[test]
fn every_pack_keeps_button_labels_at_7_to_1_and_labels_at_4_5_to_1_as_drawn() {
    let mut rows = Vec::new();
    let mut failing = Vec::new();
    for (name, dir) in packs() {
        let text = std::fs::read_to_string(dir.join("skin.ogt"))
            .unwrap_or_else(|e| panic!("{name}: skin.ogt: {e}"));
        let file = parse_skin(&text).unwrap_or_else(|e| panic!("{name}: {e}"));
        let atlas = image::open(dir.join("atlas.png"))
            .expect("atlas.png")
            .to_rgba8();
        let tex = atlas_handle();
        let theme = file.theme(Some(tex), [0; 6]);
        let p = theme.palette;
        let bg = rgb(p.bg);
        // A button straight on the screen and inside a panel (a menu card or
        // a dialog), every role in every state.
        for (ground_name, panel) in [("screen", None), ("panel", Some(p.surface))] {
            for (label, _) in ROLES {
                for state in ["rest", "hovered", "pressed"] {
                    let mut scene = RenderScene::new();
                    let mut ui = ui_with_font(add_test_font(&mut scene));
                    ui.set_theme(theme);
                    let vp = Viewport::new(800.0, 600.0);
                    let build = |ui: &mut Ui| {
                        let buttons = |ui: &mut Ui| {
                            for (n, variant) in ROLES {
                                ui.button(
                                    n,
                                    n,
                                    ButtonDesc {
                                        variant,
                                        ..ButtonDesc::default()
                                    },
                                );
                            }
                        };
                        match panel {
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
                    };
                    let step = |ui: &mut Ui, input: InputFrame| {
                        ui.begin(input, vp);
                        build(ui);
                        ui.end();
                    };
                    // Past the entrance motion.
                    for _ in 0..40 {
                        step(&mut ui, InputFrame::default());
                    }
                    let path = if panel.is_some() {
                        format!("G/{label}")
                    } else {
                        label.to_string()
                    };
                    let (x, y) = ui
                        .frame()
                        .element(&path)
                        .unwrap_or_else(|| panic!("{name}: no button {path}"))
                        .center();
                    let hover = InputFrame {
                        pointer: Vec2::new(x, y),
                        pointer_inside: true,
                        ..InputFrame::default()
                    };
                    match state {
                        "hovered" => {
                            for _ in 0..20 {
                                step(&mut ui, hover.clone());
                            }
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
                            for _ in 0..20 {
                                step(
                                    &mut ui,
                                    InputFrame {
                                        down: true,
                                        ..hover.clone()
                                    },
                                );
                            }
                        }
                        _ => step(&mut ui, InputFrame::default()),
                    }
                    let (ink, fill) = measured(&ui, label, bg, tex, &atlas);
                    let r = ratio(ink, fill);
                    rows.push(format!(
                        "{name:8} {label:9} {state:7} on the {ground_name:6} {r:6.2}:1  {} on {}",
                        hex(ink),
                        hex(fill)
                    ));
                    let bar = if matches!(state, "rest" | "hovered")
                        && matches!(label, "Primary" | "Secondary")
                    {
                        BUTTON_BAR
                    } else {
                        LABEL_BAR
                    };
                    // Quiet and danger buttons have no fill of their own (a
                    // tint over what is behind): their ink is for the pack's
                    // surfaces, not the bare screen.
                    let applies = panel.is_some() || matches!(label, "Primary" | "Secondary");
                    if applies && r < bar {
                        failing.push(format!(
                            "{name}: the {label} button's label, {state}, on the {ground_name}: {r:.2}:1 (under {bar}:1)"
                        ));
                    }
                }
            }
        }
        // The three label inks, on the screen and in a panel.
        for (ground_name, panel) in [("screen", None), ("panel", Some(p.surface))] {
            for (ink_name, ink) in [
                ("fg", p.fg),
                ("fgMuted", p.fg_muted),
                ("fgSubtle", p.fg_subtle),
            ] {
                let mut scene = RenderScene::new();
                let mut ui = ui_with_font(add_test_font(&mut scene));
                ui.set_theme(theme);
                let vp = Viewport::new(800.0, 600.0);
                for _ in 0..40 {
                    ui.begin(InputFrame::default(), vp);
                    let label = |ui: &mut Ui| {
                        ui.label(
                            "L",
                            "Ink",
                            LabelDesc {
                                color: Some(ink),
                                ..LabelDesc::default()
                            },
                        )
                    };
                    match panel {
                        Some(bg) => ui.panel(
                            "G",
                            PanelDesc {
                                background: Some(bg),
                                ..PanelDesc::default()
                            },
                            label,
                        ),
                        None => label(&mut ui),
                    }
                    ui.end();
                }
                let (ink_l, ground) = measured(&ui, "Ink", bg, tex, &atlas);
                let r = ratio(ink_l, ground);
                rows.push(format!(
                    "{name:8} {ink_name:9} label   on the {ground_name:6} {r:6.2}:1  {} on {}",
                    hex(ink_l),
                    hex(ground)
                ));
                // On the bare screen a pack's light ink is for a dark bg it
                // never sits on in the kit; the screen's own text is fg.
                let bar_applies = panel.is_some();
                if bar_applies && r < LABEL_BAR {
                    failing.push(format!(
                        "{name}: {ink_name} on the {ground_name}: {r:.2}:1 (under {LABEL_BAR}:1)"
                    ));
                }
            }
        }
        // A text field's placeholder (fgSubtle on the field's fill and
        // frame), and its typed text (fg), unfocused and focused, on the
        // screen and in a panel: the pack's own settings screens show both.
        for (ground_name, panel) in [("screen", None), ("panel", Some(p.surface))] {
            for (what, value) in [("placeholder", ""), ("text", "Ada")] {
                for focused in [false, true] {
                    let mut scene = RenderScene::new();
                    let mut ui = ui_with_font(add_test_font(&mut scene));
                    ui.set_theme(theme);
                    let vp = Viewport::new(800.0, 600.0);
                    let mut v = value.to_string();
                    let step = |ui: &mut Ui, v: &mut String, input: InputFrame| {
                        ui.begin(input, vp);
                        let mut field = |ui: &mut Ui| {
                            ui.text_field(
                                "F",
                                v,
                                &FieldDesc {
                                    placeholder: "Your name".into(),
                                    ..FieldDesc::default()
                                },
                            );
                        };
                        match panel {
                            Some(bg) => ui.panel(
                                "G",
                                PanelDesc {
                                    background: Some(bg),
                                    ..PanelDesc::default()
                                },
                                field,
                            ),
                            None => field(ui),
                        }
                        ui.end();
                    };
                    for _ in 0..40 {
                        step(&mut ui, &mut v, InputFrame::default());
                    }
                    if focused {
                        let path = if panel.is_some() { "G/F" } else { "F" };
                        let (x, y) = ui.frame().element(path).expect("the field").center();
                        let at = InputFrame {
                            pointer: Vec2::new(x, y),
                            pointer_inside: true,
                            ..InputFrame::default()
                        };
                        step(
                            &mut ui,
                            &mut v,
                            InputFrame {
                                down: true,
                                pressed: true,
                                ..at.clone()
                            },
                        );
                        step(
                            &mut ui,
                            &mut v,
                            InputFrame {
                                released: true,
                                ..at.clone()
                            },
                        );
                        for _ in 0..20 {
                            step(&mut ui, &mut v, at.clone());
                        }
                    }
                    let shown = if value.is_empty() { "Your name" } else { value };
                    let (ink, fill) = measured(&ui, shown, bg, tex, &atlas);
                    let r = ratio(ink, fill);
                    let state = if focused { "focused" } else { "rest" };
                    rows.push(format!(
                        "{name:8} {what:11} {state:7} on the {ground_name:6} {r:6.2}:1  {} on {}",
                        hex(ink),
                        hex(fill)
                    ));
                    if r < LABEL_BAR {
                        failing.push(format!(
                            "{name}: a field's {what}, {state}, on the {ground_name}: {r:.2}:1 (under {LABEL_BAR}:1)"
                        ));
                    }
                }
            }
        }
    }
    println!("{}", rows.join("\n"));
    assert!(failing.is_empty(), "{}", failing.join("\n"));
}
