//! Historical opt-in contrast diagnostics, not acceptance checks. The UI
//! re-plan (2026-11-04) removes contrast thresholds; these remain available
//! with `--ignored` for investigating old contrast reports only.
//!
//! button_contrast.rs for the style packs and the games' styles: every
//! button role's label, in every state, the three label inks (`ink`,
//! `inkMuted`, `inkFaint`), a text field's placeholder (`inkFaint` on the
//! field's frame) and typed text, at rest and focused, a stack's count on
//! a bag slot, a meter's label over each kind's fill and over its track,
//! and a badge in every tone, as og-ui draws them with a style, read against
//! what is really under them: the frame's draw commands composited in
//! linear light, a styled piece as the mean of its atlas texels (the
//! style's `atlas.png`) times its tint. Styles: the four atlas packs of
//! the former test sample, Ember Vale's copy of the fantasy pack
//! (the former test sample `ember`), the archetypes' sparkorb,
//! lampdive, lanebeat and pathguard (palette round two, row 77: sparkorb's primary label read
//! 3.42:1, lampdive's placeholder 3.57:1, pathguard's primary 3.30:1 and
//! its danger button 2.96:1, Hollowbell's danger badge 4.26:1, Lantern
//! Vale's success, warn and info badges 3.15, 2.72 and 4.07:1, before
//! their palettes moved); a generated style for every
//! shape language and finish, drawn with their procedural dressings and no
//! atlas (docs/ui.md §12.5: a generated style passes what a built-in one
//! does). Every style is drawn with its dressings set, as the runtime sets
//! them.
//!
//! A third test draws every style's primary and secondary buttons through
//! og-render with the style's own cooked fonts and reads each label, in
//! every state, from the frame as og ui check does: the colours above can be
//! right while the strokes never take them (the pixel pack's primary
//! labels, hollowbell FINDINGS #20, fixed).
//!
//! Two bars. A label on a surface (a screen, a panel, a dialog) fails
//! under 3:1 and is printed as a warning under WCAG's 4.5:1, as og ui
//! check reads the frame since the owner relaxed contrast
//! (docs/ui.md §15.4, 2026-11-02); text in a dim ink (a placeholder, a
//! stack's count) is held to the 3:1 floor alone. A button's label
//! reaches 7:1 (WCAG AAA) here: a 13 to 15 px label drawn by MSDF over a
//! textured, shaded fill reads about a third lower in the frame, where og
//! ui check takes the ground's median and the ink's farthest percent. Ember
//! Vale's Set out read 5.3:1 by these commands and 3.36:1 in its game
//! frame (owed row 48); with every primary and secondary label at 7:1 here
//! the lowest in its game frames is 4.65:1
//! (the former test sample The bars hold
//! for every pack (the fantasy family since the web host and tools round,
//! the other five since the UI shell round's palettes:
//! the former test sample every row is printed.
//!
//! Ways this could fail: a pack's accent or fill too dark for its ink once
//! the frame art shades it; a muted or subtle ink that reads on paper
//! only; a hovered or pressed state darker than its rest; a translucent
//! piece that reads over one ground only; a placeholder under 4.5:1 (the
//! fantasy pack's read 4.40:1 here and 2.93:1 in lantern-vale's game
//! frame, FINDINGS #29; pixel 4.22, cartoon 4.13, before their inkFaint
//! moved), or the focus ring showing through a focused field's frame
//! (scifi 3.52:1 and neon 3.00:1 on the ring's colour, before og-ui cut
//! the ring under every control); an accent badge under 4.5:1 in a panel
//! (pixel 3.90, cartoon 3.78, fantasy and ember 4.12, scifi 4.37 before
//! pixel's and scifi's primarySoft thinned and cartoon's and fantasy's
//! highlight darkened).

#[path = "../../../crates/og-ui/tests/support/mod.rs"]
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
/// Under it a label fails; between it and [`LABEL_BAR`] it is printed as a
/// warning (docs/ui.md §15.4, the owner, 2026-11-02: the look comes first,
/// and contrast is an error only for live text under 3:1). Text in a dim
/// ink (inkMuted, inkFaint: a placeholder, a stack's count) is held to
/// this floor alone.
const ERROR_BAR: f32 = 3.0;

/// A label read at `r`:1: a failure under [`ERROR_BAR`], a warning under
/// [`LABEL_BAR`].
fn gate(r: f32, what: String, failing: &mut Vec<String>, warnings: &mut Vec<String>) {
    if r < ERROR_BAR {
        failing.push(format!("{what}: {r:.2}:1 (under {ERROR_BAR}:1)"));
    } else if r < LABEL_BAR {
        warnings.push(format!("{what}: {r:.2}:1 (under {LABEL_BAR}:1, a warning)"));
    }
}

fn styles() -> Vec<(String, PathBuf)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates");
    let out: Vec<(String, PathBuf)> = ["pixel", "cartoon", "fantasy", "scifi"]
        .iter()
        .map(|n| (n.to_string(), root.join("styles").join(n)))
        .collect();
    out
}

/// The seed styles of templates/styles that `og style new` generated
/// (og-ui's `style_gen::seeds`): drawn with no atlas, every part from
/// their tokens and procedural dressings.
fn seeds() -> Vec<(String, PathBuf)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates/styles");
    og_ui::style_gen::seeds()
        .into_iter()
        .map(|c| (c.name.clone(), root.join(&c.name)))
        .collect()
}

/// Each style's name, text and atlas (a white texel for a style with none),
/// with `matrix`, a generated style for every shape language and finish
/// besides, each with another motion, ornament level, scheme and seed: a
/// generated style is held to the bars a built-in one is (docs/ui.md
/// §12.5).
fn sources(
    dirs: impl Iterator<Item = (String, PathBuf)>,
    matrix: bool,
) -> Vec<(String, String, image::RgbaImage)> {
    use og_ui::style_gen::{Finish, Motion, Ornament, ShapeLanguage, StyleChoices, generate};
    let white = || image::RgbaImage::from_pixel(1, 1, image::Rgba([255; 4]));
    let mut out: Vec<(String, String, image::RgbaImage)> = dirs
        .map(|(name, dir)| {
            let text = std::fs::read_to_string(dir.join("style.ogt"))
                .unwrap_or_else(|e| panic!("{name}: style.ogt: {e}"));
            let atlas =
                image::open(dir.join("atlas.png")).map_or_else(|_| white(), |i| i.to_rgba8());
            (name, text, atlas)
        })
        .collect();
    if matrix {
        let seeds = [
            0x1f6feb, 0xffb000, 0x0f9d58, 0xd81b60, 0x7c4dff, 0x00bcd4, 0x8d6e63, 0xfafafa,
        ];
        for (i, &shape) in ShapeLanguage::ALL.iter().enumerate() {
            for (j, &finish) in Finish::ALL.iter().enumerate() {
                let mut c = StyleChoices::for_look("matrix", "minimal modern", "a test");
                c.name = format!("gen-{}-{}", shape.name(), finish.name());
                let v = seeds[(i + j) % seeds.len()];
                c.seed = Srgba::rgb((v >> 16) as u8, (v >> 8) as u8, v as u8);
                c.ground = None;
                c.accent = None;
                c.shape = shape;
                c.finish = finish;
                c.motion = Motion::ALL[(i + j) % Motion::ALL.len()];
                c.ornament = Ornament::ALL[j % 3];
                c.scheme = if (i + j) % 2 == 0 {
                    Scheme::Dark
                } else {
                    Scheme::Light
                };
                let g = generate(&c).unwrap_or_else(|e| panic!("{}: {e}", c.name));
                out.push((c.name, g.text, white()));
            }
        }
    }
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
            DrawCmd::Image {
                rect,
                texture: Some(t),
                uv,
                color,
                ..
            } if *t == tex => {
                let m = texels(atlas, *uv);
                Some((
                    *rect,
                    [
                        m[0] * color[0],
                        m[1] * color[1],
                        m[2] * color[2],
                        m[3] * color[3],
                    ],
                ))
            }
            cmd => cmd.filled(),
        };
        if let Some((rect, c)) = piece
            && contains(&rect, centre)
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
#[ignore = "historical contrast diagnostic; UI packs have no contrast threshold"]
fn every_pack_keeps_button_labels_at_7_to_1_and_labels_at_4_5_to_1_as_drawn() {
    let mut rows = Vec::new();
    let mut failing = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    for (name, text, atlas) in sources(styles().into_iter().chain(seeds()), true) {
        let file = parse_style(&text).unwrap_or_else(|e| panic!("{name}: {e}"));
        let tex = atlas_handle();
        let theme = file.theme(Some(tex), [0; 6]);
        let p = theme.palette;
        let bg = rgb(p.backdrop);
        // A button straight on the screen and inside a panel (a menu card or
        // a dialog), every role in every state.
        for (ground_name, panel) in [("screen", None), ("panel", Some(p.panel))] {
            for (label, _) in ROLES {
                for state in ["rest", "hovered", "pressed"] {
                    let mut scene = RenderScene::new();
                    let mut ui = ui_with_font(add_test_font(&mut scene));
                    ui.set_theme(theme);
                    ui.set_dressings(file.dressings.clone());
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
                        "{name:10} {label:9} {state:7} on the {ground_name:6} {r:6.2}:1  {} on {}",
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
        for (ground_name, panel) in [("screen", None), ("panel", Some(p.panel))] {
            for (ink_name, ink) in [
                ("fg", p.ink),
                ("fgMuted", p.ink_muted),
                ("fgSubtle", p.ink_faint),
            ] {
                let mut scene = RenderScene::new();
                let mut ui = ui_with_font(add_test_font(&mut scene));
                ui.set_theme(theme);
                ui.set_dressings(file.dressings.clone());
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
                    "{name:10} {ink_name:9} label   on the {ground_name:6} {r:6.2}:1  {} on {}",
                    hex(ink_l),
                    hex(ground)
                ));
                // On the bare screen a pack's light ink is for a dark bg it
                // never sits on in the kit; the screen's own text is fg.
                let bar_applies = panel.is_some();
                if bar_applies {
                    gate(
                        r,
                        format!("{name}: {ink_name} on the {ground_name}"),
                        &mut failing,
                        &mut warnings,
                    );
                }
            }
        }
        // A text field's placeholder (inkFaint on the field's fill and
        // frame), and its typed text (fg), unfocused and focused, on the
        // screen and in a panel: the pack's own settings screens show both.
        for (ground_name, panel) in [("screen", None), ("panel", Some(p.panel))] {
            for (what, value) in [("placeholder", ""), ("text", "Ada")] {
                for focused in [false, true] {
                    let mut scene = RenderScene::new();
                    let mut ui = ui_with_font(add_test_font(&mut scene));
                    ui.set_theme(theme);
                    ui.set_dressings(file.dressings.clone());
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
                        "{name:10} {what:11} {state:7} on the {ground_name:6} {r:6.2}:1  {} on {}",
                        hex(ink),
                        hex(fill)
                    ));
                    gate(
                        r,
                        format!("{name}: a field's {what}, {state}, on the {ground_name}"),
                        &mut failing,
                        &mut warnings,
                    );
                }
            }
        }
        // A stack's count on a bag slot (the inventory screen's `inkMuted`
        // label, xs), in a slot at rest and in the selected one, in a panel.
        {
            let mut scene = RenderScene::new();
            let mut ui = ui_with_font(add_test_font(&mut scene));
            ui.set_theme(theme);
            ui.set_dressings(file.dressings.clone());
            let vp = Viewport::new(800.0, 600.0);
            for _ in 0..40 {
                ui.begin(InputFrame::default(), vp);
                let xs = ui.theme().type_ramp.xs;
                ui.panel(
                    "G",
                    PanelDesc {
                        background: Some(p.panel),
                        ..PanelDesc::default()
                    },
                    |ui| {
                        ui.grid(
                            "Slots",
                            GridDesc {
                                count: 2,
                                columns: 2,
                                ..GridDesc::default()
                            },
                            Some(1),
                            |ui, k| {
                                ui.label(
                                    "Count",
                                    if k == 0 { "12" } else { "34" },
                                    LabelDesc {
                                        step: Some(xs),
                                        color: Some(p.ink_muted),
                                        ..LabelDesc::default()
                                    },
                                );
                            },
                        );
                    },
                );
                ui.end();
            }
            for (text, state) in [("12", "rest"), ("34", "selected")] {
                let (ink, ground) = measured(&ui, text, bg, tex, &atlas);
                let r = ratio(ink, ground);
                rows.push(format!(
                    "{name:10} count     {state:8} on a slot    {r:6.2}:1  {} on {}",
                    hex(ink),
                    hex(ground)
                ));
                gate(
                    r,
                    format!("{name}: a stack's count on a {state} slot"),
                    &mut failing,
                    &mut warnings,
                );
            }
        }
        // A meter's label (the HUD's "HP 80/100"), in a panel, over its
        // fill (full: each kind's colour, warn and danger as a health
        // meter shows them under 30 % and 15 %) and over its track (empty).
        for (kind_name, kind, value) in [
            ("health", MeterKind::Health, 1.0),
            ("warn", MeterKind::Custom(p.warn), 1.0),
            ("danger", MeterKind::Custom(p.danger), 1.0),
            ("mana", MeterKind::Mana, 1.0),
            ("xp", MeterKind::Xp, 1.0),
            ("accent", MeterKind::Primary, 1.0),
            ("track", MeterKind::Health, 0.0),
        ] {
            let mut scene = RenderScene::new();
            let mut ui = ui_with_font(add_test_font(&mut scene));
            ui.set_theme(theme);
            ui.set_dressings(file.dressings.clone());
            let vp = Viewport::new(800.0, 600.0);
            for _ in 0..40 {
                ui.begin(InputFrame::default(), vp);
                ui.panel(
                    "G",
                    PanelDesc {
                        background: Some(p.panel),
                        ..PanelDesc::default()
                    },
                    |ui| {
                        ui.meter(
                            "M",
                            value,
                            1.0,
                            &MeterDesc {
                                kind,
                                label: "HP 80".into(),
                                width: Some(Size::Fixed(160.0)),
                                ..MeterDesc::default()
                            },
                        )
                    },
                );
                ui.end();
            }
            let (ink, fill) = measured(&ui, "HP 80", bg, tex, &atlas);
            let r = ratio(ink, fill);
            rows.push(format!(
                "{name:10} {kind_name:9} meter   in a panel     {r:6.2}:1  {} on {}",
                hex(ink),
                hex(fill)
            ));
            gate(
                r,
                format!("{name}: a {kind_name} meter's label"),
                &mut failing,
                &mut warnings,
            );
        }
    }
    println!("{}", rows.join("\n"));
    if !warnings.is_empty() {
        println!("warnings:\n{}", warnings.join("\n"));
    }
    assert!(failing.is_empty(), "{}", failing.join("\n"));
}

/// A badge in every tone (its tint, its tone's ink), on the screen and in
/// a panel, for every pack and the games' styles, every row printed. Every
/// tone holds 4.5:1 on both grounds. In a panel (where a pack's badges
/// sit) the status tones and the accent hold it with the palette's own
/// ink: the tones are the colours meters, toasts and status dots share, so
/// the light packs' are deep (cartoon's warn read 1.63:1 on its panel at
/// #f2a516, now #7b5102) and the dark packs' light; the tint mixes in sRGB as a
/// CSS wash would (`Srgba::tint_over`: a linear 16 % lifted a dark
/// ground so far that no ink, white included, reached 4.5:1 on the pixel
/// pack's panel). On the other side of the scale, a light pack's badge on
/// its dark stage (cartoon's success read 1.98:1 there, fantasy's danger
/// 1.54:1), the badge moves its ink (`Srgba::readable_on`); so does the
/// neutral badge, `inkMuted` on `control`, whose ground scifi's and neon's mid
/// fills made 2.15:1 and 2.81:1 (scifi's moves its fill: no ink reaches
/// 4.5:1 on it from the light side).
/// A game's own primary colour (`og.ui.configure({ primaryColor })`) over
/// each pack: the primary button's label reads at 7:1 on the fill as drawn
/// (the token times the slice's art, over a panel), whatever the colour.
/// Ways this could fail: the label's ink picked against the flat token, so
/// a light primary on a dark neon or scifi body takes dark ink on a dark
/// fill (a run-3 game's gold primary read 1.5:1 on neon); the accent
/// badge's ink (`highlight`, the style's) left on the game's wash.
#[test]
#[ignore = "historical contrast diagnostic; UI packs have no contrast threshold"]
fn a_games_primary_colour_keeps_its_button_label_readable_on_every_pack() {
    let primaries = [
        ("gold", Srgba::rgb(255, 199, 94)),
        ("navy", Srgba::rgb(26, 51, 153)),
        ("mint", Srgba::rgb(110, 220, 170)),
        ("crimson", Srgba::rgb(190, 30, 50)),
    ];
    let mut failing = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    for (name, text, atlas) in sources(styles().into_iter().chain(seeds()), false) {
        let file = parse_style(&text).expect("parses");
        let tex = atlas_handle();
        for (cname, c) in primaries {
            let theme = file.theme(Some(tex), [0; 6]).with_primary(c);
            let mut scene = RenderScene::new();
            let mut ui = ui_with_font(add_test_font(&mut scene));
            ui.set_theme(theme);
            ui.set_dressings(file.dressings.clone());
            let panel = theme.palette.panel;
            for _ in 0..40 {
                ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
                ui.panel(
                    "G",
                    PanelDesc {
                        background: Some(panel),
                        ..PanelDesc::default()
                    },
                    |ui| {
                        ui.badge("New", "New best", Tone::Primary);
                        ui.button(
                            "Play",
                            "Play",
                            // Large, as a title's play button: its label's
                            // centre sits on the slice's middle, not a seam.
                            ButtonDesc {
                                variant: ButtonVariant::Primary,
                                size: ControlSize::Large,
                                ..ButtonDesc::default()
                            },
                        );
                    },
                );
                ui.end();
            }
            let (ink, ground) = measured(&ui, "Play", rgb(theme.palette.backdrop), tex, &atlas);
            let r = ratio(ink, ground);
            if r < 7.0 {
                failing.push(format!("{name} with {cname}: {r:.2}:1"));
            }
            let (ink, ground) = measured(&ui, "New best", rgb(theme.palette.backdrop), tex, &atlas);
            let r = ratio(ink, ground);
            gate(
                r,
                format!("{name} with {cname}, its accent badge"),
                &mut failing,
                &mut warnings,
            );
        }
    }
    if !warnings.is_empty() {
        println!("warnings:\n{}", warnings.join("\n"));
    }
    assert!(
        failing.is_empty(),
        "primary labels under 7:1 as drawn: {failing:?}"
    );
}

#[test]
#[ignore = "historical contrast diagnostic; UI packs have no contrast threshold"]
fn every_style_keeps_its_badges_at_4_5_to_1_on_the_screen_and_in_a_panel() {
    let mut rows = Vec::new();
    let mut failing = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    for (name, text, atlas) in sources(styles().into_iter().chain(seeds()), true) {
        let file = parse_style(&text).unwrap_or_else(|e| panic!("{name}: {e}"));
        let tex = atlas_handle();
        let theme = file.theme(Some(tex), [0; 6]);
        let p = theme.palette;
        let bg = rgb(p.backdrop);
        // The accent badge is the results screen's "New best" (it read
        // 4.12:1 here in the fantasy pack and 4.03:1 in Lantern Vale's
        // game frame before highlight moved).
        for (ground_name, panel) in [("screen", None), ("panel", Some(p.panel))] {
            for (tone_name, tone) in [
                ("neutral", Tone::Neutral),
                ("success", Tone::Success),
                ("warn", Tone::Warn),
                ("danger", Tone::Danger),
                ("info", Tone::Info),
                ("accent", Tone::Primary),
                ("inverse", Tone::Inverse),
            ] {
                let mut scene = RenderScene::new();
                let mut ui = ui_with_font(add_test_font(&mut scene));
                ui.set_theme(theme);
                ui.set_dressings(file.dressings.clone());
                let vp = Viewport::new(800.0, 600.0);
                for _ in 0..40 {
                    ui.begin(InputFrame::default(), vp);
                    let badge = |ui: &mut Ui| ui.badge("B", "Badge", tone);
                    match panel {
                        Some(bg) => ui.panel(
                            "G",
                            PanelDesc {
                                background: Some(bg),
                                ..PanelDesc::default()
                            },
                            badge,
                        ),
                        None => badge(&mut ui),
                    }
                    ui.end();
                }
                let (ink, ground) = measured(&ui, "Badge", bg, tex, &atlas);
                let r = ratio(ink, ground);
                rows.push(format!(
                    "{name:10} {tone_name:9} badge   on the {ground_name:6} {r:6.2}:1  {} on {}",
                    hex(ink),
                    hex(ground)
                ));
                gate(
                    r,
                    format!("{name}: a {tone_name} badge on the {ground_name}"),
                    &mut failing,
                    &mut warnings,
                );
                let own = match tone {
                    Tone::Success => Some(p.success),
                    Tone::Warn => Some(p.warn),
                    Tone::Danger => Some(p.danger),
                    Tone::Info => Some(p.info),
                    Tone::Primary => Some(p.highlight),
                    _ => None,
                };
                if let Some(own) = own
                    && panel.is_some()
                    && hex(ink) != hex(rgb(own))
                {
                    failing.push(format!(
                        "{name}: a {tone_name} badge in a panel needs its ink moved ({} to {}): the palette's tone reads under {LABEL_BAR}:1 there",
                        hex(rgb(own)),
                        hex(ink)
                    ));
                }
            }
        }
    }
    println!("{}", rows.join("\n"));
    if !warnings.is_empty() {
        println!("warnings:\n{}", warnings.join("\n"));
    }
    assert!(failing.is_empty(), "{}", failing.join("\n"));
}

// ── the labels as the frame shows them ──────────────────────────────────────
//
// The test above reads the colours og-ui asks for, which a label's thin
// strokes may never reach on screen: the pixel pack's DotGothic16 draws
// one-pixel strokes that fall between device pixels, so no pixel took more
// than about half the ink, and its gold primary labels (Resume, New game,
// Buy) read 2.86:1 to 3.04:1 in the games' frames while the colours above
// read 9:1 (hollowbell FINDINGS #20; `textWeight body 0.5` with the font at
// `px_range 8` fixed it). So every pack's primary and secondary buttons are
// also drawn headless with the pack's own atlas and fonts as the cook makes
// them, at 1 device pixel per CSS pixel (the worst case: a phone's 3 gives
// a stroke three pixels), and measured as og ui check measures a frame
// (packages/engine/src/node/ui-check.ts `textBoxes`): inside the button (its
// sides left out: `frame_contrast`), the ground is the median luminance and the ink the 1 % of pixels farthest
// from it. Every label, in every state, reaches 4.5:1.

use og_cook::runtime::{Bundle, CookedFont};
use og_cook::{CookOptions, cook_project};
use og_core::glam::Vec3;
use og_render::{Background, Glyph, Kern, MsdfFont, RenderDesc, RenderTarget, Renderer, Tonemap};
use og_ui::shape::FontFile;
use std::sync::Arc;

/// Every pack's fonts cooked into one scratch project with the pack's own
/// import settings: `(pack, [body, display])`.
fn cooked_fonts() -> Vec<(String, [CookedFont; 2])> {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-ui-pack-fonts");
    let _ = std::fs::remove_dir_all(&root);
    let mut wanted = Vec::new();
    for (name, dir) in styles() {
        let file = parse_style(&std::fs::read_to_string(dir.join("style.ogt")).expect("style.ogt"))
            .expect("parses");
        // A shared face (`@styles/cjk/…`, the pixel pack's Fusion Pixel)
        // is read from templates/styles, its import settings with it.
        let styles = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates/styles");
        let source = |i: usize| -> (String, PathBuf) {
            let f = file.fonts[i]
                .clone()
                .unwrap_or_else(|| panic!("{name}: font {i}"));
            let base = f.rsplit('/').next().expect("a file name").to_string();
            let from = match f.strip_prefix("@styles/") {
                Some(rel) => styles.join(rel),
                None => dir.join(&base),
            };
            (base, from)
        };
        let sources = [source(0), source(1)];
        let files = [sources[0].0.clone(), sources[1].0.clone()];
        for (f, from) in &sources {
            let dst = root.join("assets/styles").join(&name).join(f);
            std::fs::create_dir_all(dst.parent().expect("dir")).expect("dir");
            std::fs::copy(from, &dst).unwrap_or_else(|e| panic!("{name}/{f}: {e}"));
            let settings = from.with_file_name(format!("{f}.ogt"));
            if settings.exists() {
                std::fs::copy(&settings, dst.with_file_name(format!("{f}.ogt"))).expect("settings");
            }
        }
        wanted.push((name, files));
    }
    let report = cook_project(&root, &CookOptions::default()).expect("cook");
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).expect("bundle");
    wanted
        .into_iter()
        .map(|(name, [body, display])| {
            let load = |f: &str| {
                bundle
                    .load_font(&format!("styles/{name}/{f}"))
                    .unwrap_or_else(|e| panic!("{name}/{f}: {e:?}"))
            };
            let fonts = [load(&body), load(&display)];
            (name, fonts)
        })
        .collect()
}

/// A cooked font in the scene as og-runtime adds it (one page, kerned).
fn add_cooked(scene: &mut RenderScene, f: &CookedFont) -> UiFont {
    assert_eq!(f.pages.len(), 1, "{}: one page", f.id);
    let atlas = scene.add_texture(TextureData {
        mips: Mips::None,
        color_space: ColorSpace::Linear,
        ..f.pages[0].clone()
    });
    let m = &f.meta;
    let font = MsdfFont {
        atlas,
        distance_range: m.px_range,
        line_height: m.ascender - m.descender + m.line_gap,
        glyphs: f
            .glyphs
            .iter()
            .filter_map(|g| {
                Some(Glyph {
                    codepoint: char::from_u32(g.codepoint)?,
                    advance: g.advance,
                    plane_bounds: [g.plane_left, g.plane_bottom, g.plane_right, g.plane_top],
                    atlas_bounds: [
                        g.atlas_x as f32,
                        g.atlas_y as f32,
                        (g.atlas_x + g.atlas_w) as f32,
                        (g.atlas_y + g.atlas_h) as f32,
                    ],
                })
            })
            .collect(),
        kerning: f
            .kerning
            .iter()
            .filter_map(|k| {
                Some(Kern {
                    left: char::from_u32(k.left)?,
                    right: char::from_u32(k.right)?,
                    adjust: k.adjust,
                })
            })
            .collect(),
    };
    let metrics = FontMetrics::from_msdf(&font)
        .with_file(
            f.ttf
                .clone()
                .and_then(|t| FontFile::new(Arc::new(t)))
                .or_else(|| {
                    f.shaping
                        .clone()
                        .and_then(|t| FontFile::for_shaping(Arc::new(t)))
                }),
        )
        .with_pixel(f.pixel);
    UiFont {
        handle: Some(scene.add_font(font)),
        metrics,
    }
}

/// og ui check's luminance of an sRGB byte triple.
fn byte_luminance(p: &[u8]) -> f32 {
    luminance([0, 1, 2].map(|k| lin(f32::from(p[k]) / 255.0)))
}

/// og ui check's reading of the text in `r` (CSS px at dpr 1): the ground's median
/// luminance against the mean of the 1 % of pixels farthest from it, on the
/// side most of them are. `None`: no ink.
fn frame_contrast(pixels: &[u8], width: u32, r: &Rect) -> Option<f32> {
    // og ui check insets a wide box's rounded ends; this also leaves out a
    // button's sides, where its frame's border and a press's squash (the
    // panel showing inside the layout rect) sit, which og ui check,
    // reading still frames, would count as the label's ink.
    let ix = if r.width > 3.0 * r.height {
        (r.height / 2.0).min(12.0)
    } else {
        0.0
    }
    .max(0.15 * r.width);
    let iy = r.height * 0.12;
    let (x0, y0) = ((r.x + ix).floor() as u32, (r.y + iy).floor() as u32);
    let (x1, y1) = (
        (r.x + r.width - ix).ceil() as u32,
        (r.y + r.height - iy).ceil() as u32,
    );
    let mut lum = Vec::new();
    for y in y0..y1 {
        for x in x0..x1 {
            let i = ((y * width + x) * 4) as usize;
            lum.push(byte_luminance(&pixels[i..i + 3]));
        }
    }
    let mut sorted = lum.clone();
    sorted.sort_by(f32::total_cmp);
    let ground = sorted[sorted.len() / 2];
    let mut dev: Vec<f32> = lum.iter().map(|v| (v - ground).abs()).collect();
    let far = {
        let mut d = dev.clone();
        d.sort_by(f32::total_cmp);
        d[((d.len() - 1) as f32 * 0.99).round() as usize]
    };
    if far < 0.01 {
        return None;
    }
    let (mut up, mut down, mut up_sum, mut down_sum) = (0, 0, 0.0f32, 0.0f32);
    for (k, d) in dev.iter_mut().enumerate() {
        if *d < far {
            continue;
        }
        if lum[k] > ground {
            up += 1;
            up_sum += lum[k];
        } else {
            down += 1;
            down_sum += lum[k];
        }
    }
    let ink = if up >= down {
        up_sum / up.max(1) as f32
    } else {
        down_sum / down.max(1) as f32
    };
    Some((ink.max(ground) + 0.05) / (ink.min(ground) + 0.05))
}

#[test]
#[ignore = "historical contrast diagnostic; UI packs have no contrast threshold"]
fn every_pack_draws_its_primary_and_secondary_labels_at_4_5_to_1_in_the_frame() {
    let fonts = cooked_fonts();
    let mut rows = Vec::new();
    let mut failing = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    for ((name, dir), (fname, [body, display])) in styles().into_iter().zip(&fonts) {
        assert_eq!(&name, fname);
        let file = parse_style(&std::fs::read_to_string(dir.join("style.ogt")).expect("style.ogt"))
            .expect("parses");
        let img = image::open(dir.join("atlas.png"))
            .expect("atlas.png")
            .to_rgba8();
        let labels = [("Primary", "New game"), ("Secondary", "Settings")];
        for state in ["rest", "hovered", "pressed"] {
            for (id, _) in labels {
                let mut scene = RenderScene::new();
                let style_atlas = scene.add_texture(TextureData {
                    width: img.width(),
                    height: img.height(),
                    rgba: img.as_raw().clone(),
                    color_space: ColorSpace::Srgb,
                    mips: Mips::None,
                });
                let mut ui = Ui::new(UiDesc {
                    fonts: Fonts {
                        body: add_cooked(&mut scene, body),
                        display: Some(add_cooked(&mut scene, display)),
                        ..Fonts::default()
                    },
                    ..UiDesc::default()
                });
                ui.set_theme(file.theme(Some(style_atlas), [0; 6]));
                let vp = Viewport::new(640.0, 360.0);
                let step = |ui: &mut Ui, input: InputFrame| {
                    ui.begin(input, vp);
                    let surface = ui.theme().palette.panel;
                    ui.panel(
                        "G",
                        PanelDesc {
                            background: Some(surface),
                            ..PanelDesc::default()
                        },
                        |ui| {
                            for ((id, text), (_, variant)) in labels.iter().zip(&ROLES[..2]) {
                                ui.button(
                                    *id,
                                    text,
                                    ButtonDesc {
                                        variant: *variant,
                                        ..ButtonDesc::default()
                                    },
                                );
                            }
                        },
                    );
                    ui.end();
                };
                for _ in 0..40 {
                    step(&mut ui, InputFrame::default());
                }
                let path = format!("G/{id}");
                let (x, y) = ui.frame().element(&path).expect("the button").center();
                let at = InputFrame {
                    pointer: Vec2::new(x, y),
                    pointer_inside: true,
                    ..InputFrame::default()
                };
                match state {
                    "hovered" => (0..20).for_each(|_| step(&mut ui, at.clone())),
                    "pressed" => {
                        step(
                            &mut ui,
                            InputFrame {
                                down: true,
                                pressed: true,
                                ..at.clone()
                            },
                        );
                        for _ in 0..20 {
                            step(
                                &mut ui,
                                InputFrame {
                                    down: true,
                                    ..at.clone()
                                },
                            );
                        }
                    }
                    _ => step(&mut ui, InputFrame::default()),
                }
                let e = ui.frame().element(&path).expect("the button");
                let rect = Rect::new(e.x, e.y, e.width, e.height);
                let mut overlay = og_render::Overlay::new(vp.width, vp.height, vp.dpr);
                ui.frame().emit(
                    &EmitDesc {
                        layer: 0,
                        time: 0.0,
                    },
                    &mut overlay,
                );
                scene.overlay = Some(overlay);
                let bg = ui.theme().palette.backdrop.linear();
                scene.environment.background = Background::Color(Vec3::new(bg[0], bg[1], bg[2]));
                scene.post.tonemap = Tonemap::None;
                let desc = RenderDesc {
                    width: vp.width as u32,
                    height: vp.height as u32,
                    ..RenderDesc::default()
                };
                let mut renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
                renderer
                    .render(&scene, &RenderTarget::Offscreen)
                    .expect("render");
                let image = renderer.read_offscreen().expect("read back");
                if std::env::var_os("OG_UI_PACK_FRAMES").is_some() {
                    let f = out_dir().join(format!("pack-label-{name}-{id}-{state}.png"));
                    image::save_buffer(
                        &f,
                        &image.pixels,
                        image.width,
                        image.height,
                        image::ColorType::Rgba8,
                    )
                    .expect("write frame");
                }
                let r = frame_contrast(&image.pixels, image.width, &rect)
                    .unwrap_or_else(|| panic!("{name}: no ink in the {id} button, {state}"));
                rows.push(format!("{name:10} {id:9} {state:7} in the frame {r:6.2}:1"));
                gate(
                    r,
                    format!("{name}: the {id} button's label, {state}, in the frame"),
                    &mut failing,
                    &mut warnings,
                );
            }
        }
    }
    println!("{}", rows.join("\n"));
    if !warnings.is_empty() {
        println!("warnings:\n{}", warnings.join("\n"));
    }
    assert!(failing.is_empty(), "{}", failing.join("\n"));
}
