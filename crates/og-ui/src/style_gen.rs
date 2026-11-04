//! The style generator behind `og style new` (docs/ui.md §12.5): from a
//! look, a premise and a few choices (a palette seed, a shape language, an
//! ornament level, a motion personality, a finish and a sound kit) it
//! writes a complete version 2 style: its palette (every ink held to its
//! contrast bar by construction), the tokens, procedural dressings in the
//! UI pass's paints, strokes, effects and materials, its sound sets (the
//! `type sfx` presets it writes beside it) and haptics; then the art-slot
//! requests for its frames, buttons, ornaments and icons, which the art
//! brief turns into prompts (`packages/knowledge/scripts/brief.ts`,
//! `styleArtBriefs`).
//!
//! A generated style is held to the same tests as a built-in one
//! (og-ui's tests/packs.rs and og-tests-cooked's pack_contrast.rs generate a matrix of
//! them); the seed styles of `templates/styles` are generated here from
//! their recorded choices, then read and kept.

use crate::Srgba;
use crate::slot::{SlotKind, SlotSpec};
use crate::style::Scheme;
use crate::tokens::{SoundEvent, hex};

/// How corners are drawn across the element classes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeLanguage {
    /// Gentle rounding: modern, cosy.
    Soft,
    /// Chunky rounding: casual, cartoon.
    Round,
    /// Capsules everywhere: kawaii, gacha.
    Pill,
    /// Continuous corners: minimal modern.
    Squircle,
    /// Hard rectangles: tactical, broadcast.
    Sharp,
    /// 45° cuts: sci-fi, tactical.
    Cut,
    /// Square notches: cyberpunk, ornate frames.
    Notch,
    /// Pixel squares: retro.
    Pixel,
}

/// How much a style frames and decorates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ornament {
    None,
    Light,
    Rich,
}

/// How a style moves (docs/ui.md §7.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    /// Unhurried, no bounce.
    Calm,
    /// Quick and crisp.
    Snappy,
    /// Springs with overshoot, squash on press.
    Bouncy,
    /// Weighty, slow to start, firm to stop.
    Heavy,
    /// Instant, mechanical, no easing flourish.
    Mechanical,
    /// Drifting, soft settles.
    Floaty,
}

/// The surface treatment the dressings paint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Finish {
    /// Flat fills, hairlines.
    Flat,
    /// Lit gradients, outlines, hard shadows.
    Gradient,
    /// Frosted translucent panels over the scene.
    Glass,
    /// Glowing lines and holographic frames.
    Hologram,
    /// Glowing tubes on dark.
    Neon,
    /// Paper sheets, ink lines, soft shadows.
    Paper,
    /// Bevels and hard steps.
    Bevel,
    /// Ink washes and brush strokes.
    Brush,
}

/// The `type sfx` presets a style's UI sounds come from.
#[derive(Clone, Debug, PartialEq)]
pub struct SoundKit {
    /// A press: (preset, seed, pitch); a second seed makes the set.
    pub tap: (&'static str, u32, f32),
    pub confirm: (&'static str, u32, f32),
    pub back: (&'static str, u32, f32),
    pub open: (&'static str, u32, f32),
    pub error: (&'static str, u32, f32),
    pub reward: (&'static str, u32, f32),
    /// The press set's volume (0–1); hovers sound at a third of it when
    /// `hover` (a tactile, clicky style).
    pub volume: f32,
    pub hover: bool,
    /// How far the pitch of a set wanders (0.05: ±5 %).
    pub spread: f32,
}

macro_rules! choice {
    ($ty:ident { $($v:ident $n:literal,)* }) => {
        impl $ty {
            pub const ALL: &'static [$ty] = &[$($ty::$v,)*];
            pub fn name(self) -> &'static str {
                match self { $($ty::$v => $n,)* }
            }
            pub fn from_name(s: &str) -> Option<$ty> {
                match s { $($n => Some($ty::$v),)* _ => None }
            }
            pub fn names() -> String {
                Self::ALL.iter().map(|v| v.name()).collect::<Vec<_>>().join(", ")
            }
        }
    };
}

choice!(ShapeLanguage {
    Soft "soft",
    Round "round",
    Pill "pill",
    Squircle "squircle",
    Sharp "sharp",
    Cut "cut",
    Notch "notch",
    Pixel "pixel",
});
choice!(Ornament {
    None "none",
    Light "light",
    Rich "rich",
});
choice!(Motion {
    Calm "calm",
    Snappy "snappy",
    Bouncy "bouncy",
    Heavy "heavy",
    Mechanical "mechanical",
    Floaty "floaty",
});
choice!(Finish {
    Flat "flat",
    Gradient "gradient",
    Glass "glass",
    Hologram "hologram",
    Neon "neon",
    Paper "paper",
    Bevel "bevel",
    Brush "brush",
});

/// What `og style new` is given.
#[derive(Clone, Debug, PartialEq)]
pub struct StyleChoices {
    /// The style's slug.
    pub name: String,
    /// The look family it is for (og-taste §12: `ornate fantasy`).
    pub look: String,
    /// One line of what the game is.
    pub premise: String,
    pub scheme: Scheme,
    /// The palette seed: the accent the palette is built round.
    pub seed: Srgba,
    /// The screen's ground (`None`: from the seed and the scheme).
    pub ground: Option<Srgba>,
    /// The main ink (`None`: from the scheme, tinted by the seed).
    pub ink: Option<Srgba>,
    /// A second accent for focus and highlights (`None`: from the seed).
    pub accent: Option<Srgba>,
    pub shape: ShapeLanguage,
    pub ornament: Ornament,
    pub motion: Motion,
    pub finish: Finish,
    /// Font asset ids: body, display, numeric, then the Chinese
    /// companions (body and display).
    pub fonts: [String; 5],
    /// The icon sheet's id (8×4, the standard 32 names).
    pub icons: Option<String>,
    /// Body text size in CSS px (the ramp scales from it).
    pub body: f32,
    /// Labels set in capitals with this tracking (em): tactical, broadcast.
    pub label_tracking: f32,
    pub sound: SoundKit,
    /// Dialogs, panels and toasts enter with a glitch of this strength
    /// (0: the motion personality's own entrances): cyber.
    pub glitch: f32,
    /// The hue danger and health take (`None`: the scheme's red and
    /// green), held to the status bar like them: cyber's signal red.
    pub danger: Option<Srgba>,
    /// The hue info takes (`None`: the scheme's blue): cyber's cyan.
    pub info: Option<Srgba>,
}

/// A generated style: its `style.ogt` and the sound files it names.
#[derive(Clone, Debug, PartialEq)]
pub struct Generated {
    pub text: String,
    /// `(file name, text)` of each `type sfx` preset beside the style.
    pub sounds: Vec<(String, String)>,
}

/// The standard names of an 8×4 icon sheet (every built-in style's).
pub const ICON_NAMES: [&str; 32] = [
    "heart", "star", "coin", "lock", "pause", "gear", "sword", "shield", "potion", "gem", "key",
    "map", "bolt", "clock", "trophy", "home", "plus", "close", "check", "play", "back", "bag",
    "bomb", "crown", "flag", "apple", "leaf", "fire", "drop", "skull", "music", "person",
];

// ─────────────────────────────────────────────────────────────── colour

fn hsl(c: Srgba) -> (f32, f32, f32) {
    let [r, g, b] = c.rgb.map(|v| f32::from(v) / 255.0);
    let (mx, mn) = (r.max(g).max(b), r.min(g).min(b));
    let l = (mx + mn) / 2.0;
    if mx == mn {
        return (0.0, 0.0, l);
    }
    let d = mx - mn;
    let s = if l > 0.5 {
        d / (2.0 - mx - mn)
    } else {
        d / (mx + mn)
    };
    let h = if mx == r {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if mx == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    (h * 60.0, s, l)
}

fn from_hsl(h: f32, s: f32, l: f32) -> Srgba {
    let h = h.rem_euclid(360.0) / 360.0;
    let (s, l) = (s.clamp(0.0, 1.0), l.clamp(0.0, 1.0));
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let ch = |t: f32| {
        let t = t.rem_euclid(1.0);
        let v = if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        };
        (v * 255.0).round().clamp(0.0, 255.0) as u8
    };
    Srgba::rgb(ch(h + 1.0 / 3.0), ch(h), ch(h - 1.0 / 3.0))
}

/// `ink`, moved as little as it takes to read `bar`:1 on every ground.
fn readable_on_all(ink: Srgba, grounds: &[Srgba], bar: f32) -> Srgba {
    let mut c = ink;
    for _ in 0..4 {
        for g in grounds {
            if let Some(m) = c.readable_on(*g, bar) {
                c = m;
            }
        }
        if grounds.iter().all(|g| c.contrast(*g) >= bar) {
            break;
        }
    }
    c
}

/// A palette with every role, its inks at their bars: body inks 7:1 on the
/// panels, muted and faint 4.6:1 on the panels and controls, the primary's
/// label 7.2:1 in every state, the status inks 5.5:1 on the screen and the
/// panels.
pub fn palette(c: &StyleChoices) -> crate::theme::Palette {
    let dark = c.scheme == Scheme::Dark;
    let (h, s, _) = hsl(c.seed);
    let w = Srgba::WHITE;
    let k = Srgba::BLACK;
    let ground = c.ground.unwrap_or(if dark {
        from_hsl(h, (s * 0.35).min(0.3), 0.075)
    } else {
        from_hsl(h, (s * 0.3).min(0.25), 0.90)
    });
    let (panel, well, raised, hover) = if dark {
        let panel = ground.mix(w, 0.07).mix(c.seed, 0.03);
        (
            panel,
            ground.mix(k, 0.3),
            panel.mix(w, 0.05),
            panel.mix(w, 0.08),
        )
    } else {
        let panel = ground.mix(w, 0.75).mix(c.seed, 0.02);
        (
            panel,
            ground.mix(k, 0.06),
            panel.mix(w, 0.4),
            panel.mix(k, 0.035),
        )
    };
    let ink0 = c.ink.unwrap_or(if dark {
        w.mix(c.seed, 0.06)
    } else {
        k.mix(c.seed, 0.18)
    });
    let control = panel.mix(ink0, 0.07);
    let control_strong = panel.mix(ink0, 0.14);
    let grounds = [panel, raised, hover, control, control_strong, ground];
    let ink = readable_on_all(ink0, &grounds, 7.2);
    let muted = readable_on_all(ink.mix(panel, 0.25), &grounds, 6.0);
    let faint = readable_on_all(ink.mix(panel, 0.45), &grounds, 4.6);
    let line = panel.mix(ink, 0.18);
    let line_strong = panel.mix(ink, 0.5);
    // The selected tab's ground: a step off the track that ink reads on.
    let mut knob = if dark {
        panel.mix(ink, 0.2)
    } else {
        raised.mix(w, 0.6)
    };
    if ink.contrast(knob) < 4.8 {
        knob = knob.readable_on(ink, 4.8).unwrap_or(panel);
    }
    // The primary and its label at 7.2:1 in every state.
    let light_label = w.mix(c.seed, 0.03);
    let dark_label = k.mix(c.seed, 0.22);
    let label = if c.seed.contrast(light_label) >= c.seed.contrast(dark_label) {
        light_label
    } else {
        dark_label
    };
    let away = if label.luminance() > 0.5 { k } else { w };
    let primary = c.seed.readable_on(label, 7.4).map_or(c.seed, |_| {
        let mut p = c.seed;
        let mut t = 0.0;
        while p.contrast(label) < 7.4 && t < 1.0 {
            t += 0.02;
            p = c.seed.mix(away, t);
        }
        p
    });
    let primary_hover = primary.mix(away, 0.08);
    let primary_active = primary.mix(away, 0.16);
    let accent = c.accent.unwrap_or(if dark {
        from_hsl(h, s.max(0.6), 0.68)
    } else {
        from_hsl(h, s.max(0.6), 0.35)
    });
    // The accent badge's ink, on its primarySoft wash too: a light
    // primary on a dark ground (gold, olive) lightens its wash towards the
    // ink, so the wash thins until the ink reads.
    let (soft, highlight) = {
        let mut best = None;
        for a in [0.22, 0.18, 0.14, 0.11, 0.08, 0.05] {
            let soft = primary.fade(a);
            let grounds = [
                panel,
                raised,
                ground,
                // Composited in linear light, as the UI pass draws it.
                soft.over_linear(ground),
                soft.over_linear(panel),
                soft.over_linear(raised),
            ];
            let h = readable_on_all(accent, &grounds, 4.9);
            best = Some((soft, h));
            if grounds.iter().all(|g| h.contrast(*g) >= 4.9) {
                break;
            }
        }
        best.expect("a wash")
    };
    let focus = readable_on_all(accent, &[ground, panel], 3.2);
    let status_at = |base: Srgba, bar: f32| {
        readable_on_all(
            base,
            &[
                ground,
                panel,
                raised,
                base.fade(0.16).over(ground),
                base.fade(0.16).over(panel),
            ],
            bar,
        )
    };
    let status =
        |hue: f32, sat: f32| status_at(from_hsl(hue, sat, if dark { 0.62 } else { 0.32 }), 5.6);
    // A chosen hue keeps as much of itself as a label on its tint allows
    // (4.5:1, pack_contrast's bar for a danger button).
    let status_of = |base: Srgba| status_at(base, 4.6);
    let danger = c.danger.map_or_else(|| status(2.0, 0.7), status_of);
    let shadow = if dark {
        k.fade(0.55)
    } else {
        k.mix(c.seed, 0.25).fade(0.24)
    };
    crate::theme::Palette {
        backdrop: ground,
        well,
        panel,
        panel_hover: hover,
        panel_raised: raised,
        line,
        line_strong,
        ink,
        ink_muted: muted,
        ink_faint: faint,
        ink_inverse: if dark { panel } else { raised.mix(w, 0.5) },
        control,
        control_strong,
        primary,
        primary_hover,
        primary_active,
        on_primary: label,
        primary_soft: soft,
        highlight,
        focus,
        knob,
        hud: panel.fade(0.94),
        scrim: ground.mix(k, 0.5).fade(0.7),
        shadow,
        sheen: w.fade(if dark { 0.22 } else { 0.5 }),
        success: status(142.0, 0.55),
        warn: status(36.0, 0.85),
        danger,
        info: c.info.map_or_else(|| status(212.0, 0.7), status_of),
        health: c.danger.map_or_else(|| status(130.0, 0.6), |_| danger),
        mana: status(214.0, 0.85),
        xp: status(276.0, 0.6),
        gold: status(44.0, 0.85),
    }
}

// ────────────────────────────────────────────────────────── the writing

fn lit(c: Srgba) -> String {
    format!("\"{}\"", hex(c))
}

fn num(v: f32) -> String {
    let r = (v * 1000.0).round() / 1000.0;
    if r == r.trunc() {
        format!("{}", r as i64)
    } else {
        format!("{r}")
    }
}

/// The radii (small, control, bar, panel) of a shape language.
fn radii(s: ShapeLanguage) -> [f32; 4] {
    match s {
        ShapeLanguage::Soft => [6.0, 8.0, 6.0, 12.0],
        ShapeLanguage::Round => [10.0, 14.0, 10.0, 20.0],
        ShapeLanguage::Pill => [12.0, 22.0, 12.0, 24.0],
        ShapeLanguage::Squircle => [8.0, 12.0, 8.0, 18.0],
        ShapeLanguage::Sharp => [0.0, 2.0, 0.0, 2.0],
        ShapeLanguage::Cut => [4.0, 6.0, 2.0, 10.0],
        ShapeLanguage::Notch => [4.0, 6.0, 2.0, 10.0],
        ShapeLanguage::Pixel => [0.0, 0.0, 0.0, 0.0],
    }
}

/// The `shape` block: every element class's corner and radius.
fn shape_block(s: ShapeLanguage) -> String {
    use ShapeLanguage as S;
    let [small, control, bar, panel] = radii(s);
    let corner = |r: f32, kind: &str| {
        if r == 0.0 {
            "square 0".to_string()
        } else {
            format!("{kind} {}", num(r))
        }
    };
    let (frame, ctl, chip, slot) = match s {
        S::Soft | S::Round => ("round", "round", "round pill".to_string(), "round"),
        S::Pill => ("round", "round", "round pill".to_string(), "round"),
        S::Squircle => ("squircle", "squircle", "round pill".to_string(), "squircle"),
        S::Sharp => ("round", "round", corner(control, "round"), "round"),
        S::Cut => ("bevel", "bevel", corner(control, "bevel"), "bevel"),
        S::Notch => ("notch", "bevel", corner(small, "bevel"), "notch"),
        S::Pixel => ("square", "square", "square 0".to_string(), "square"),
    };
    let button = if s == S::Pill {
        "round pill".to_string()
    } else {
        corner(control, ctl)
    };
    let lines = [
        ("panel", corner(panel, frame)),
        ("dialog", corner(panel * 1.2, frame)),
        ("card", corner(panel * 0.8, frame)),
        ("tooltip", corner(small, ctl)),
        ("button", button.clone()),
        (
            "field",
            if s == S::Pill {
                button.clone()
            } else {
                corner(control * 0.8, ctl)
            },
        ),
        ("chip", chip.clone()),
        ("badge", chip),
        ("tab", corner(control * 0.8, ctl)),
        ("slot", corner(small * 1.5, slot)),
        (
            "bar",
            if matches!(s, S::Pill | S::Round | S::Soft | S::Squircle) {
                "round pill".to_string()
            } else {
                corner(bar, ctl)
            },
        ),
        ("hud", corner(control, ctl)),
        (
            "avatar",
            if s == S::Pixel {
                "square 0".into()
            } else {
                "round pill".into()
            },
        ),
    ];
    let mut out = String::from("shape {\n");
    for (k, v) in lines {
        out.push_str(&format!("  {k} {v}\n"));
    }
    out.push_str("}\n");
    out.replace("bevel 0", "square 0")
        .replace("notch 0", "square 0")
}

/// The springs (duration ms, bounce) of ui, panel, emphasis and playful,
/// the durations, enter and toast transitions, pop, shake, the press
/// squash (sx sy dy) and the hover lift (dy scale).
struct MotionSet {
    springs: [(f32, f32); 4],
    durations: [f32; 6],
    enter: &'static str,
    toast: &'static str,
    pop: f32,
    shake: f32,
    press: (f32, f32, f32),
    hover: (f32, f32),
    stagger: f32,
    overshoot: f32,
}

fn motion_set(m: Motion) -> MotionSet {
    match m {
        Motion::Calm => MotionSet {
            springs: [(220.0, 0.0), (380.0, 0.0), (480.0, 0.06), (560.0, 0.1)],
            durations: [120.0, 200.0, 300.0, 460.0, 560.0, 680.0],
            enter: "fade",
            toast: "slideUp",
            pop: 0.04,
            shake: 4.0,
            press: (0.98, 0.98, 1.0),
            hover: (-1.0, 1.0),
            stagger: 50.0,
            overshoot: 0.02,
        },
        Motion::Snappy => MotionSet {
            springs: [(150.0, 0.0), (240.0, 0.04), (320.0, 0.12), (400.0, 0.18)],
            durations: [90.0, 140.0, 220.0, 340.0, 420.0, 520.0],
            enter: "slideUp",
            toast: "slideDown",
            pop: 0.06,
            shake: 6.0,
            press: (0.97, 0.97, 1.0),
            hover: (-1.0, 1.01),
            stagger: 30.0,
            overshoot: 0.04,
        },
        Motion::Bouncy => MotionSet {
            springs: [(260.0, 0.3), (420.0, 0.32), (520.0, 0.42), (620.0, 0.5)],
            durations: [110.0, 170.0, 280.0, 420.0, 560.0, 700.0],
            enter: "pop",
            toast: "pop",
            pop: 0.2,
            shake: 8.0,
            press: (1.06, 0.9, 3.0),
            hover: (-3.0, 1.04),
            stagger: 60.0,
            overshoot: 0.18,
        },
        Motion::Heavy => MotionSet {
            springs: [(240.0, 0.0), (500.0, 0.03), (640.0, 0.06), (700.0, 0.1)],
            durations: [130.0, 220.0, 360.0, 520.0, 640.0, 760.0],
            enter: "slideUp",
            toast: "slideUp",
            pop: 0.03,
            shake: 10.0,
            press: (1.0, 0.96, 2.0),
            hover: (0.0, 1.0),
            stagger: 70.0,
            overshoot: 0.02,
        },
        Motion::Mechanical => MotionSet {
            springs: [(100.0, 0.0), (180.0, 0.0), (240.0, 0.0), (300.0, 0.04)],
            durations: [60.0, 100.0, 160.0, 240.0, 320.0, 400.0],
            enter: "slideLeft",
            toast: "slideLeft",
            pop: 0.0,
            shake: 6.0,
            press: (1.0, 1.0, 1.0),
            hover: (0.0, 1.0),
            stagger: 25.0,
            overshoot: 0.0,
        },
        Motion::Floaty => MotionSet {
            springs: [(320.0, 0.1), (560.0, 0.14), (680.0, 0.22), (760.0, 0.28)],
            durations: [140.0, 240.0, 380.0, 560.0, 680.0, 820.0],
            enter: "fade",
            toast: "fade",
            pop: 0.05,
            shake: 4.0,
            press: (0.99, 0.99, 0.0),
            hover: (-2.0, 1.01),
            stagger: 80.0,
            overshoot: 0.06,
        },
    }
}

/// The version 1 text the generator starts from: palette, radii, ramp,
/// durations, fonts, icons, text treatments and the motion basics, which
/// `upgrade_style` turns into version 2 with derived tokens.
fn v1_text(c: &StyleChoices, p: &crate::theme::Palette, m: &MotionSet) -> String {
    let mut s = String::from("type style\nversion 1\n---\n");
    s.push_str(&format!("name \"{}\"\n", c.name));
    s.push_str(&format!(
        "scheme {}\n",
        if c.scheme == Scheme::Dark {
            "dark"
        } else {
            "light"
        }
    ));
    s.push_str("palette {\n");
    for n in crate::theme::Palette::NAMES {
        s.push_str(&format!("  {n} {}\n", lit(p.get(n).expect("a role"))));
    }
    s.push_str("}\n");
    let [small, control, bar, panel] = radii(c.shape);
    s.push_str(&format!(
        "radius {{\n  small {}\n  control {}\n  bar {}\n  panel {}\n}}\n",
        num(small),
        num(control),
        num(bar),
        num(panel)
    ));
    let b = c.body;
    let step = |k: f32| (b * k).round();
    let lh = |sz: f32, k: f32| (sz * k).round();
    s.push_str("ramp {\n");
    for (name, k, l) in [
        ("body", 1.0, 1.42),
        ("meta", 0.87, 1.38),
        ("caption", 0.87, 1.38),
        ("reading", 1.07, 1.55),
        ("lg", 1.27, 1.36),
        ("xl", 1.47, 1.32),
        ("xl2", 1.73, 1.22),
        ("xl3", 2.0, 1.18),
        ("displaySm", 2.13, 1.14),
        ("displayMd", 2.67, 1.12),
        ("displayLg", 3.33, 1.1),
    ] {
        let sz = step(k);
        s.push_str(&format!("  {name} {} {}\n", num(sz), num(lh(sz, l))));
    }
    s.push_str("}\n");
    let d = m.durations;
    s.push_str(&format!(
        "durations {{\n  instant {}\n  fast {}\n  base {}\n  slow {}\n  spring {}\n  bounce {}\n}}\n",
        num(d[0]),
        num(d[1]),
        num(d[2]),
        num(d[3]),
        num(d[4]),
        num(d[5])
    ));
    s.push_str(&format!(
        "fonts {{\n  body \"{}\"\n  display \"{}\"\n  numeric \"{}\"\n  cjk \"{}\"\n  cjkDisplay \"{}\"\n}}\n",
        c.fonts[0], c.fonts[1], c.fonts[2], c.fonts[3], c.fonts[4]
    ));
    if let Some(i) = &c.icons {
        s.push_str(&format!("icons \"{i}\" 8 4 {{\n  names"));
        for n in ICON_NAMES {
            s.push_str(&format!(" \"{n}\""));
        }
        s.push_str("\n}\n");
    }
    match c.finish {
        Finish::Neon | Finish::Hologram => {
            s.push_str("textGlow display 1.4 primary 0.45\n");
        }
        Finish::Gradient => {
            s.push_str(&format!(
                "textShadow display 0 3 0 {}\n",
                lit(p.shadow.fade(1.0).mix(Srgba::BLACK, 0.4))
            ));
        }
        _ => {}
    }
    s.push_str(&format!(
        "press {} {} {}\nhover {} {}\n",
        num(m.press.0),
        num(m.press.1),
        num(m.press.2),
        num(m.hover.0),
        num(m.hover.1)
    ));
    let (enter, toast) = if c.glitch > 0.0 {
        let g = format!("glitch {}", num(c.glitch));
        (g.clone(), g)
    } else {
        (m.enter.to_string(), m.toast.to_string())
    };
    s.push_str(&format!(
        "motion {{\n  enter {}\n  stagger {}\n  toast {}\n  progress {}\n  shake {}\n  pop {}\n}}\n",
        enter,
        num(m.stagger),
        toast,
        num(d[3]),
        num(m.shake),
        num(m.pop)
    ));
    s
}

fn typography(c: &StyleChoices) -> String {
    let b = c.body;
    let display_w = 700;
    let label_upper = c.label_tracking > 0.0;
    let rows = [
        (
            "display",
            3.2,
            1.12,
            -0.01,
            display_w,
            "display",
            Some((0.8, 0.9)),
        ),
        (
            "title",
            2.0,
            1.2,
            -0.005,
            display_w,
            "display",
            Some((0.85, 0.92)),
        ),
        ("heading", 1.47, 1.32, 0.0, 600, "display", None),
        ("body", 1.0, 1.45, 0.0, 400, "body", None),
        (
            "label",
            if label_upper { 0.8 } else { 0.87 },
            1.38,
            if label_upper { c.label_tracking } else { 0.01 },
            600,
            "body",
            None,
        ),
        ("caption", 0.87, 1.38, 0.0, 400, "body", None),
        ("numeric", 1.27, 1.36, 0.0, 500, "numeric", None),
    ];
    let mut out = String::from("typography {\n");
    for (role, k, l, tr, wgt, font, phone) in rows {
        let sz = (b * k).round();
        out.push_str(&format!(
            "  {role} {} {} {} {wgt} {font}",
            num(sz),
            num((sz * l).round()),
            num(tr)
        ));
        if let Some((pp, pl)) = phone {
            let a = (sz * pp).round();
            let bl = (sz * pl).round();
            out.push_str(&format!(
                " {{\n    phonePortrait {} {}\n    phoneLandscape {} {}\n  }}\n",
                num(a),
                num((a * l).round()),
                num(bl),
                num((bl * l).round())
            ));
        } else {
            out.push('\n');
        }
    }
    out.push_str("}\n");
    out
}

fn stroke_block(c: &StyleChoices) -> String {
    let (h, t, r, k, f) = match (c.shape, c.ornament) {
        (ShapeLanguage::Pixel, _) => (2.0, 2.0, 2.0, 4.0, 2.0),
        (_, Ornament::Rich) => (1.0, 1.5, 2.0, 3.0, 2.0),
        (ShapeLanguage::Round | ShapeLanguage::Pill, _) => (1.5, 2.0, 2.5, 3.5, 2.5),
        _ => (1.0, 1.25, 1.75, 2.5, 2.0),
    };
    format!(
        "stroke {{\n  hairline {}\n  thin {}\n  regular {}\n  thick {}\n  focus {}\n}}\n",
        num(h),
        num(t),
        num(r),
        num(k),
        num(f)
    )
}

fn elevation_block(c: &StyleChoices, p: &crate::theme::Palette) -> String {
    let sh = p.shadow;
    let a = |k: f32| lit(sh.fade((sh.alpha * k).min(1.0)));
    let hard = matches!(c.shape, ShapeLanguage::Pixel)
        || matches!(c.finish, Finish::Gradient | Finish::Bevel);
    let mut out = String::from("elevation {\n");
    for (level, dy, blur, k) in [
        ("raised", 2.0_f32, 6.0_f32, 0.7_f32),
        ("floating", 4.0, 12.0, 0.8),
        ("overlay", 8.0, 24.0, 0.9),
        ("top", 14.0, 36.0, 1.0),
    ] {
        out.push_str(&format!("  {level} {{\n"));
        if hard {
            out.push_str(&format!("    shadow 0 {} 0 0 {}\n", num(dy.min(6.0)), a(k)));
        } else {
            out.push_str(&format!(
                "    shadow 0 {} {} 0 {}\n    shadow 0 1 2 0 {}\n",
                num(dy),
                num(blur),
                a(k * 0.8),
                a(0.4)
            ));
        }
        out.push_str("  }\n");
    }
    out.push_str("}\n");
    out
}

/// The motion block's token lines, replacing the derived ones.
fn motion_tokens(m: &MotionSet) -> Vec<String> {
    let names = ["ui", "panel", "emphasis", "playful"];
    let mut out: Vec<String> = names
        .iter()
        .zip(m.springs)
        .map(|(n, (d, b))| format!("spring {n} {} {}", num(d), num(b)))
        .collect();
    out.push(format!("overshoot {}", num(m.overshoot)));
    out
}

/// The `sounds` block: every event's set, its ranges and its haptic.
fn sounds_block(c: &StyleChoices) -> String {
    let k = &c.sound;
    let sp = k.spread;
    let v = k.volume;
    let range =
        |lo: f32, hi: f32| format!("{} {}", num(lo.clamp(0.0, 1.0)), num(hi.clamp(0.0, 1.0)));
    let pitch = |p: f32, s: f32| format!("{} {}", num((p - s).max(0.25)), num((p + s).min(4.0)));
    let tap = "\"sfx-tap.ogt\" \"sfx-tap2.ogt\"";
    let mut lines: Vec<String> = Vec::new();
    for e in SoundEvent::ALL {
        use SoundEvent as E;
        let line = match e {
            E::Hover if k.hover => format!(
                "{tap} volume {} pitch {} haptic none",
                range(v * 0.25, v * 0.35),
                pitch(1.25, sp)
            ),
            E::Hover | E::Focus | E::ScrollTick => "none haptic none".to_string(),
            E::Press | E::Select => format!(
                "{tap} volume {} pitch {} haptic tick",
                range(v * 0.8, v),
                pitch(1.0, sp)
            ),
            E::Tab => format!(
                "{tap} volume {} pitch {} haptic tick",
                range(v * 0.7, v * 0.9),
                pitch(1.15, sp)
            ),
            E::SliderTick => format!(
                "{tap} volume {} pitch {} haptic tick",
                range(v * 0.35, v * 0.5),
                pitch(1.4, sp * 2.0)
            ),
            E::Typewriter => format!(
                "{tap} volume {} pitch {} haptic none",
                range(v * 0.2, v * 0.3),
                pitch(1.6, sp * 3.0)
            ),
            E::CounterTick => format!(
                "{tap} volume {} pitch 0.9 1.5 haptic none",
                range(v * 0.35, v * 0.45)
            ),
            E::Confirm => format!(
                "\"sfx-confirm.ogt\" volume {} pitch {} haptic tap",
                range(v * 0.85, v),
                pitch(1.0, sp * 0.5)
            ),
            E::ToggleOn => format!(
                "\"sfx-confirm.ogt\" volume {} pitch {} haptic tick",
                range(v * 0.6, v * 0.7),
                pitch(1.2, sp)
            ),
            E::ToggleOff => format!(
                "\"sfx-back.ogt\" volume {} pitch {} haptic tick",
                range(v * 0.6, v * 0.7),
                pitch(1.1, sp)
            ),
            E::Cancel | E::Back => format!(
                "\"sfx-back.ogt\" volume {} pitch {} haptic tick",
                range(v * 0.8, v),
                pitch(1.0, sp)
            ),
            E::Open => format!(
                "\"sfx-open.ogt\" volume {} pitch {} haptic none",
                range(v * 0.6, v * 0.7),
                pitch(1.05, sp)
            ),
            E::Close => format!(
                "\"sfx-open.ogt\" volume {} pitch {} haptic none",
                range(v * 0.5, v * 0.6),
                pitch(0.85, sp)
            ),
            E::Notification => format!(
                "\"sfx-confirm.ogt\" volume {} pitch {} haptic tap",
                range(v * 0.7, v * 0.8),
                pitch(1.3, sp)
            ),
            E::Warning => format!(
                "\"sfx-error.ogt\" volume {} pitch {} haptic warning",
                range(v * 0.6, v * 0.7),
                pitch(1.25, sp)
            ),
            E::Error => format!(
                "\"sfx-error.ogt\" volume {} pitch {} haptic error",
                range(v * 0.8, v),
                pitch(1.0, sp * 0.5)
            ),
            E::Success => format!(
                "\"sfx-reward.ogt\" volume {} pitch {} haptic success",
                range(v * 0.8, v),
                pitch(1.1, sp * 0.5)
            ),
            E::Reward => format!(
                "\"sfx-reward.ogt\" volume {} pitch {} haptic double",
                range(v * 0.9, v),
                pitch(1.0, sp * 0.5)
            ),
            E::RevealCommon => format!(
                "{tap} volume {} pitch {} haptic none",
                range(v * 0.6, v * 0.7),
                pitch(0.9, sp)
            ),
            E::RevealUncommon => format!(
                "\"sfx-confirm.ogt\" volume {} pitch {} haptic tick",
                range(v * 0.7, v * 0.8),
                pitch(0.95, sp)
            ),
            E::RevealRare => format!(
                "\"sfx-confirm.ogt\" volume {} pitch {} haptic tap",
                range(v * 0.8, v * 0.9),
                pitch(1.15, sp)
            ),
            E::RevealEpic => format!(
                "\"sfx-reward.ogt\" volume {} pitch {} haptic double",
                range(v * 0.85, v),
                pitch(0.9, sp * 0.5)
            ),
            E::RevealLegendary => format!(
                "\"sfx-reward.ogt\" volume {} pitch {} haptic thud",
                range(v * 0.9, v),
                pitch(0.8, sp * 0.5)
            ),
            E::RevealMythic => format!(
                "\"sfx-reward.ogt\" volume {} pitch {} haptic thud",
                range(v, v),
                pitch(0.7, sp * 0.5)
            ),
        };
        lines.push(format!("  {} {line}", e.name()));
    }
    format!("sounds {{\n{}\n}}\n", lines.join("\n"))
}

fn sfx(preset: &str, seed: u32, pitch: f32, volume: f32, what: &str) -> String {
    format!(
        "type sfx\nversion 1\n---\n# {what}, generated by og style new.\npreset \"{preset}\"\nseed {seed}\npitch {}\nlength 1\nvolume {}\n",
        num(pitch),
        num(volume)
    )
}

/// The dressings: the style's surfaces in the UI pass's paints, strokes,
/// effects and materials, by finish and ornament level. Every fill stays
/// within a step of the role the widget gives the part (a label's inks are
/// held against the role), and the primary button keeps its label's bar
/// (the button reads the mean of the paints it is drawn with).
fn dressings(c: &StyleChoices, p: &crate::theme::Palette) -> String {
    let w = Srgba::WHITE;
    let k = Srgba::BLACK;
    let rich = c.ornament == Ornament::Rich;
    let light = c.ornament != Ornament::None;
    // Selector → lines, in first-seen order; a selector dressed twice
    // takes both sets of lines in one block.
    let mut blocks: Vec<(String, Vec<String>)> = Vec::new();
    let mut dress = |sel: &str, body: Vec<String>| match blocks.iter_mut().find(|(s, _)| s == sel) {
        Some((_, lines)) => lines.extend(body),
        None => blocks.push((sel.to_string(), body)),
    };
    // The motion personality's press squash and hover lift.
    let m = motion_set(c.motion);
    dress(
        "Button.base",
        vec![
            format!(
                "press {} {} {}",
                num(m.press.0),
                num(m.press.1),
                num(m.press.2)
            ),
            format!("hover {} {}", num(m.hover.0), num(m.hover.1)),
        ],
    );
    let alpha = |c: Srgba, a: f32| lit(c.fade(a));
    let ornaments = |v: &mut Vec<String>| {
        if rich {
            v.push("ornament corner".into());
        }
    };
    match c.finish {
        Finish::Flat => {
            if light {
                dress("Panel.frame", vec!["stroke 1 line inside".into()]);
            }
            dress("Tabs.tab[selected]", vec!["stroke 2 primary inside".into()]);
        }
        Finish::Gradient if rich => {
            // Gilt: gold-leaf edges lit from above, a warm glow in the
            // panels, corner ornaments on every framed surface.
            let gold = lit(p.focus);
            let gilt = |w: f32| {
                format!(
                    "stroke {} linear 180 {gold} primary primaryActive inside",
                    num(w)
                )
            };
            let mut panel = vec![
                "fill linear 180 panelRaised panel well".into(),
                format!(
                    "fill radial {} {}",
                    alpha(p.focus, 0.1),
                    alpha(p.focus, 0.0)
                ),
                gilt(3.0),
                format!("stroke 1 {} outside", alpha(p.focus, 0.35)),
                format!(
                    "effect dropShadow 0 6 0 0 {}",
                    alpha(p.shadow, (p.shadow.alpha * 1.4).min(1.0))
                ),
                format!("effect innerGlow 16 {}", alpha(p.focus, 0.14)),
            ];
            ornaments(&mut panel);
            dress("Panel.frame", panel);
            let mut card = vec![
                "fill linear 180 panelRaised panel".into(),
                gilt(2.0),
                format!(
                    "effect dropShadow 0 4 0 0 {}",
                    alpha(p.shadow, p.shadow.alpha)
                ),
                format!("effect innerGlow 10 {}", alpha(p.focus, 0.1)),
            ];
            ornaments(&mut card);
            dress("Card.frame", card);
            dress(
                "Button.base",
                vec![
                    "fill linear 180 controlStrong control".into(),
                    gilt(2.5),
                    format!(
                        "effect dropShadow 0 4 0 0 {}",
                        alpha(p.shadow, (p.shadow.alpha * 1.4).min(1.0))
                    ),
                    format!("effect innerShadow 0 -3 0 0 {}", alpha(k, 0.16)),
                ],
            );
            dress(
                "Button.base[hover]",
                vec![format!("effect outerGlow 8 {}", alpha(p.focus, 0.3))],
            );
            dress(
                "Button.base[pressed]",
                vec![format!("effect innerShadow 0 3 0 0 {}", alpha(k, 0.22))],
            );
            dress("Field.frame", vec![gilt(1.5)]);
            dress(
                "Slot.frame",
                vec![
                    gilt(2.0),
                    format!("effect innerShadow 0 3 0 0 {}", alpha(k, 0.18)),
                ],
            );
            dress("Tooltip.frame", vec![gilt(1.5)]);
            dress(
                "Meter.track",
                vec![format!("stroke 1.5 {} inside", alpha(p.focus, 0.55))],
            );
            dress(
                "Meter.fill",
                vec![
                    format!("effect innerShadow 0 -3 0 0 {}", alpha(k, 0.18)),
                    format!("effect innerShadow 0 2 0 0 {}", alpha(w, 0.35)),
                ],
            );
            dress("Chip.base", vec![gilt(1.5)]);
            dress("Tabs.tab[selected]", vec![gilt(2.0)]);
        }
        Finish::Gradient if c.scheme == Scheme::Light => {
            // Toy: thick ink outlines, hard ink shadows under every
            // surface, a glossy top on the buttons and the bars.
            let line = |w: f32| format!("stroke {} {} inside", num(w), alpha(p.ink, 0.88));
            let hard =
                |y: f32, a: f32| format!("effect dropShadow 0 {} 0 0 {}", num(y), alpha(p.ink, a));
            let gloss =
                |y: f32, a: f32| format!("effect innerShadow 0 {} 0 0 {}", num(y), alpha(w, a));
            dress(
                "Panel.frame",
                vec![
                    "fill linear 180 panelRaised panel".into(),
                    line(3.5),
                    hard(6.0, 0.3),
                    gloss(4.0, 0.7),
                ],
            );
            dress(
                "Card.frame",
                vec![
                    "fill linear 180 panelRaised panel".into(),
                    line(3.0),
                    hard(4.0, 0.3),
                    gloss(3.0, 0.7),
                ],
            );
            dress(
                "Button.base",
                vec![
                    "fill linear 180 panelRaised control".into(),
                    line(3.0),
                    hard(4.0, 0.88),
                    gloss(3.0, 0.55),
                    format!("effect innerShadow 0 -4 0 0 {}", alpha(k, 0.12)),
                ],
            );
            dress(
                "Button.base[pressed]",
                vec![
                    hard(1.0, 0.88),
                    format!("effect innerShadow 0 3 0 0 {}", alpha(k, 0.18)),
                ],
            );
            dress("Field.frame", vec![line(2.5)]);
            dress("Slot.frame", vec![line(2.5), hard(3.0, 0.25)]);
            dress("Tooltip.frame", vec![line(2.5), hard(3.0, 0.3)]);
            dress("Meter.track", vec![line(2.5)]);
            dress(
                "Meter.fill",
                vec![
                    gloss(3.0, 0.5),
                    format!("effect innerShadow 0 -3 0 0 {}", alpha(k, 0.16)),
                ],
            );
            dress("Chip.base", vec![line(2.5), hard(2.0, 0.6)]);
            dress("Tabs.tab[selected]", vec![line(2.5)]);
        }
        Finish::Gradient => {
            let mut panel = vec![
                "fill linear 180 panelRaised panel".into(),
                "stroke 3 lineStrong inside".into(),
                format!(
                    "effect dropShadow 0 5 0 0 {}",
                    alpha(p.shadow, (p.shadow.alpha * 1.4).min(1.0))
                ),
            ];
            ornaments(&mut panel);
            dress("Panel.frame", panel);
            dress(
                "Card.frame",
                vec![
                    "fill linear 180 panelRaised panel".into(),
                    "stroke 2.5 lineStrong inside".into(),
                    format!(
                        "effect dropShadow 0 4 0 0 {}",
                        alpha(p.shadow, p.shadow.alpha)
                    ),
                ],
            );
            dress(
                "Button.base",
                vec![
                    "fill linear 180 controlStrong control".into(),
                    "stroke 2.5 lineStrong inside".into(),
                    format!(
                        "effect dropShadow 0 4 0 0 {}",
                        alpha(p.shadow, (p.shadow.alpha * 1.4).min(1.0))
                    ),
                    format!("effect innerShadow 0 -3 0 0 {}", alpha(k, 0.12)),
                ],
            );
            dress(
                "Button.base[pressed]",
                vec![format!("effect innerShadow 0 3 0 0 {}", alpha(k, 0.18))],
            );
            dress("Field.frame", vec!["stroke 2 lineStrong inside".into()]);
            dress(
                "Slot.frame",
                vec![
                    "stroke 2 lineStrong inside".into(),
                    format!("effect innerShadow 0 3 0 0 {}", alpha(k, 0.12)),
                ],
            );
            dress(
                "Meter.fill",
                vec![
                    format!("effect innerShadow 0 -3 0 0 {}", alpha(k, 0.18)),
                    format!("effect innerShadow 0 2 0 0 {}", alpha(w, 0.35)),
                ],
            );
            dress("Chip.base", vec!["stroke 2 lineStrong inside".into()]);
        }
        Finish::Glass => {
            let mut panel = vec![
                format!("fill {}", alpha(p.panel, 0.82)),
                format!(
                    "effect backdrop 18 saturation 1.3 tint {}",
                    alpha(p.panel, 0.4)
                ),
                format!("stroke 1 {} inside", alpha(w, 0.18)),
            ];
            ornaments(&mut panel);
            dress("Panel.frame", panel);
            dress(
                "Card.frame",
                vec![
                    format!(
                        "fill linear 160 {} {}",
                        alpha(p.panel_raised, 0.92),
                        alpha(p.panel, 0.88)
                    ),
                    format!("stroke 1 {} inside", alpha(w, 0.14)),
                ],
            );
            dress(
                "Button.base",
                vec![format!("stroke 1 {} inside", alpha(w, 0.2))],
            );
            dress(
                "Hud.frame",
                vec![format!(
                    "effect backdrop 12 saturation 1.2 tint {}",
                    alpha(p.panel, 0.5)
                )],
            );
        }
        Finish::Hologram | Finish::Neon => {
            let glow = if c.finish == Finish::Neon { 0.55 } else { 0.35 };
            let mut panel = vec![
                "fill linear 180 panel well".into(),
                format!("stroke 1.5 {} inside", alpha(p.focus, 0.8)),
                format!("effect outerGlow 10 {}", alpha(p.focus, glow * 0.5)),
            ];
            if c.finish == Finish::Hologram {
                panel.push(format!("effect innerGlow 14 {}", alpha(p.focus, 0.12)));
            }
            ornaments(&mut panel);
            dress("Panel.frame", panel);
            dress(
                "Card.frame",
                vec![
                    format!("stroke 1 {} inside", alpha(p.focus, 0.55)),
                    format!("effect innerGlow 10 {}", alpha(p.focus, 0.1)),
                ],
            );
            dress(
                "Button.base",
                vec![
                    format!("stroke 1.5 {} inside", alpha(p.focus, 0.75)),
                    format!("effect outerGlow 6 {}", alpha(p.focus, glow * 0.4)),
                ],
            );
            dress(
                "Button.base[primary]",
                vec![format!("effect outerGlow 12 {}", alpha(p.primary, glow))],
            );
            dress(
                "Button.base[hover]",
                vec![format!(
                    "effect outerGlow 12 {}",
                    alpha(p.focus, glow * 0.7)
                )],
            );
            dress(
                "Field.frame[focused]",
                vec![format!("effect outerGlow 8 {}", alpha(p.focus, glow * 0.6))],
            );
            dress(
                "Meter.fill",
                vec![format!(
                    "effect outerGlow 8 {}",
                    alpha(p.primary, glow * 0.6)
                )],
            );
            dress(
                "Tabs.tab[selected]",
                vec![format!("stroke 1.5 {} inside", alpha(p.focus, 0.9))],
            );
            if c.finish == Finish::Hologram {
                dress(
                    "Hud.frame",
                    vec![format!(
                        "effect backdrop 10 saturation 1.2 tint {}",
                        alpha(p.panel, 0.45)
                    )],
                );
            }
        }
        Finish::Paper if !rich => {
            // Cut paper: sheets with a fibre grain, stitched edges and hard
            // offset shadows; tape across the selected tab.
            let hard = |x: f32, y: f32, a: f32| {
                format!(
                    "effect dropShadow {} {} 0 0 {}",
                    num(x),
                    num(y),
                    alpha(p.ink, a)
                )
            };
            let grain = "fill linear 174 panelRaised panel 0.2 panelRaised 0.38 panel 0.55 panelRaised 0.8 panel";
            dress(
                "Panel.frame",
                vec![
                    grain.into(),
                    "stroke 2 lineStrong inside dash 7 4 round".into(),
                    hard(4.0, 5.0, 0.2),
                ],
            );
            dress(
                "Card.frame",
                vec![
                    grain.into(),
                    "stroke 1.5 lineStrong inside dash 6 4 round".into(),
                    hard(3.0, 4.0, 0.18),
                ],
            );
            dress(
                "Button.base",
                vec!["stroke 2 lineStrong inside".into(), hard(2.0, 3.0, 0.3)],
            );
            dress(
                "Button.base[pressed]",
                vec![
                    hard(0.0, 1.0, 0.3),
                    format!(
                        "effect innerShadow 1 2 0 0 {}",
                        alpha(p.shadow, p.shadow.alpha)
                    ),
                ],
            );
            dress(
                "Field.frame",
                vec!["stroke 1.5 lineStrong inside dash 5 3".into()],
            );
            dress(
                "Slot.frame",
                vec![
                    "stroke 1.5 lineStrong inside dash 4 3 round".into(),
                    hard(2.0, 2.0, 0.14),
                ],
            );
            dress("Tooltip.frame", vec![hard(2.0, 3.0, 0.22)]);
            dress(
                "Chip.base",
                vec!["stroke 1.5 lineStrong inside dash 4 3".into()],
            );
            dress(
                "Tabs.tab[selected]",
                vec![
                    format!("fill {}", alpha(p.focus, 0.22)),
                    "stroke 1.5 lineStrong inside".into(),
                ],
            );
        }
        Finish::Paper => {
            let mut panel = vec![
                "stroke 1.5 lineStrong inside".into(),
                format!(
                    "effect dropShadow 1 3 6 0 {}",
                    alpha(p.shadow, p.shadow.alpha)
                ),
            ];
            ornaments(&mut panel);
            dress("Panel.frame", panel);
            dress(
                "Card.frame",
                vec![
                    "stroke 1.5 lineStrong inside".into(),
                    format!(
                        "effect dropShadow 2 3 0 0 {}",
                        alpha(p.shadow, p.shadow.alpha)
                    ),
                ],
            );
            dress("Button.base", vec!["stroke 1.5 lineStrong inside".into()]);
            dress(
                "Button.base[pressed]",
                vec![format!(
                    "effect innerShadow 1 2 0 0 {}",
                    alpha(p.shadow, p.shadow.alpha)
                )],
            );
            dress(
                "Field.frame",
                vec!["stroke 1.5 lineStrong inside dash 5 3".into()],
            );
            dress(
                "Tabs.tab[selected]",
                vec!["stroke 1.5 lineStrong inside".into()],
            );
        }
        Finish::Bevel => {
            let mut panel = vec![
                format!("stroke 2 {} inside", lit(p.line_strong)),
                format!("effect innerShadow 2 2 0 0 {}", alpha(w, 0.22)),
                format!("effect innerShadow -2 -2 0 0 {}", alpha(k, 0.35)),
            ];
            ornaments(&mut panel);
            dress("Panel.frame", panel);
            // A bevelled button is a solid slab, its quiet and ghost
            // variants too: unfilled, they showed the art behind them
            // (midcore's leaderboard tabs read 2.1:1 over a sunset sky).
            dress(
                "Button.base",
                vec![
                    "fill linear 180 controlStrong control".into(),
                    "stroke 2 lineStrong inside".into(),
                    format!("effect innerShadow 2 2 0 0 {}", alpha(w, 0.3)),
                    format!("effect innerShadow -2 -2 0 0 {}", alpha(k, 0.3)),
                ],
            );
            dress(
                "Button.base[pressed]",
                vec![format!("effect innerShadow 2 2 0 0 {}", alpha(k, 0.35))],
            );
            dress(
                "Slot.frame",
                vec![
                    "stroke 2 lineStrong inside".into(),
                    format!("effect innerShadow 2 2 0 0 {}", alpha(k, 0.3)),
                ],
            );
            dress(
                "Field.frame",
                vec![format!("effect innerShadow 2 2 0 0 {}", alpha(k, 0.3))],
            );
            dress("Meter.track", vec!["stroke 2 lineStrong inside".into()]);
        }
        Finish::Brush => {
            // Ink on paper: a wash that pools towards the edges, the
            // brush's fibre streaks across it, dry-brush edges broken where
            // the brush lifted.
            let i = |a: f32| alpha(p.ink, a);
            // The streaks darken the sheet on either scheme: a light ink's
            // streaks on a dark sheet would lift it under the status inks.
            let fibre = if c.scheme == Scheme::Light { p.ink } else { k };
            let f = |a: f32| alpha(fibre, a);
            let wash = "fill radial panelRaised panelRaised panel well".to_string();
            let streaks = format!(
                "fill linear 176 {} {} 0.14 {} 0.27 {} 0.41 {} 0.58 {} 0.72 {} 0.86 {}",
                f(0.0),
                f(0.05),
                f(0.0),
                f(0.035),
                f(0.0),
                f(0.05),
                f(0.0),
                f(0.03)
            );
            let dry = |w: f32, a: f32, dash: f32, gap: f32, off: f32| {
                format!(
                    "stroke {} {} inside dash {} {} offset {} round",
                    num(w),
                    i(a),
                    num(dash),
                    num(gap),
                    num(off)
                )
            };
            let mut panel = vec![
                wash.clone(),
                streaks.clone(),
                dry(2.5, 0.8, 46.0, 5.0, 9.0),
                format!("stroke 1 {} outside dash 13 7 offset 3 round", i(0.4)),
                format!("effect innerShadow 0 0 18 0 {}", i(0.1)),
            ];
            ornaments(&mut panel);
            dress("Panel.frame", panel);
            dress(
                "Card.frame",
                vec![
                    wash,
                    streaks,
                    dry(1.5, 0.65, 30.0, 4.0, 5.0),
                    format!("effect innerShadow 0 0 12 0 {}", i(0.08)),
                ],
            );
            dress("Button.base", vec![dry(1.75, 0.65, 28.0, 4.0, 6.0)]);
            dress(
                "Button.base[primary]",
                vec![format!("effect innerShadow 0 0 10 0 {}", alpha(k, 0.18))],
            );
            dress("Field.frame", vec![dry(1.5, 0.55, 22.0, 3.0, 4.0)]);
            dress("Slot.frame", vec![dry(1.5, 0.5, 18.0, 4.0, 2.0)]);
            dress("Tabs.tab[selected]", vec!["stroke 2 primary inside".into()]);
            dress(
                "Meter.fill",
                vec![format!("effect innerShadow 0 0 6 0 {}", alpha(k, 0.2))],
            );
        }
    }
    dress("Button.base[primary]", vec![PRIMARY_FILL.into()]);
    // Every style: a selected slot rings in its accent; the rarities glow
    // in their colours and the top tiers shine.
    dress(
        "Slot.frame[selected]",
        vec![format!("stroke 2 {} inside", lit(p.focus))],
    );
    dress(
        "Slot.frame[rarity=rare]",
        vec!["stroke 2 \"#3d8bfd\" inside".into()],
    );
    dress(
        "Slot.frame[rarity=epic]",
        vec![
            "stroke 2 \"#a855f7\" inside".into(),
            "effect outerGlow 8 \"#a855f766\"".into(),
        ],
    );
    dress(
        "Slot.frame[rarity=legendary]",
        vec![
            "stroke 2 \"#f59e0b\" inside".into(),
            "effect outerGlow 10 \"#f59e0b80\"".into(),
            "material shine 0.6".into(),
        ],
    );
    dress(
        "Slot.frame[rarity=mythic]",
        vec![
            "stroke 2 \"#ef4444\" inside".into(),
            "effect outerGlow 12 \"#ef444480\"".into(),
            "material hologram 0.5 2".into(),
        ],
    );
    if rich {
        dress("Dialog.frame", vec!["ornament corner".into()]);
    }
    let mut out = String::new();
    for (sel, lines) in blocks {
        out.push_str(&format!("dress \"{sel}\" {{\n"));
        for l in lines {
            out.push_str(&format!("  {l}\n"));
        }
        out.push_str("}\n");
    }
    out
}

/// The primary button's lit fill: from the game's primary down to its
/// pressed tone (`primaryActive`, always the one further from its label),
/// so any primary a game sets keeps its label at 7:1 on an atlas-free
/// style (a flat crimson reads 6.3:1 under white at best).
const PRIMARY_FILL: &str = "fill linear 180 primary primaryActive primaryActive primaryActive";

/// Replaces block `key` of a version 2 style text with `block` (its full
/// text), keeping everything else.
fn replace_block(text: &str, key: &str, block: &str) -> Result<String, String> {
    let mut doc = og_core::ogt::Document::parse(text).map_err(|e| e.to_string())?;
    let new = og_core::ogt::Document::parse(&format!("type style\nversion 2\n---\n{block}"))
        .map_err(|e| format!("{key}: {e}"))?;
    let entry = new
        .body
        .entries
        .into_iter()
        .next()
        .ok_or_else(|| format!("{key}: empty"))?;
    match doc.body.entries.iter().position(|e| e.key == key) {
        Some(i) => doc.body.entries[i] = entry,
        None => doc.body.entries.push(entry),
    }
    doc.to_text().map_err(|e| e.to_string())
}

/// A complete version 2 style from `c` (see the module docs), with the
/// `type sfx` presets it names. The text parses and states what it was
/// generated from.
pub fn generate(c: &StyleChoices) -> Result<Generated, String> {
    let p = palette(c);
    let m = motion_set(c.motion);
    let v1 = v1_text(c, &p, &m);
    let mut text = crate::style_file::upgrade_style(&v1)?;
    {
        // The upgrade's dressings (the press and hover) are rewritten
        // with the procedural ones; a new style draws today's screens,
        // not a shipped version 1 game's (`screens 1`).
        let mut doc = og_core::ogt::Document::parse(&text).map_err(|e| e.to_string())?;
        doc.body
            .entries
            .retain(|e| e.key != "dress" && e.key != "screens");
        text = doc.to_text().map_err(|e| e.to_string())?;
    }
    text = replace_block(&text, "typography", &typography(c))?;
    text = replace_block(&text, "stroke", &stroke_block(c))?;
    text = replace_block(&text, "elevation", &elevation_block(c, &p))?;
    text = replace_block(&text, "shape", &shape_block(c.shape))?;
    text = replace_block(&text, "sounds", &sounds_block(c))?;
    // The motion personality's springs over the derived ones.
    {
        let mut doc = og_core::ogt::Document::parse(&text).map_err(|e| e.to_string())?;
        if let Some(e) = doc.body.entries.iter_mut().find(|e| e.key == "motion")
            && let Some(b) = e.block.as_mut()
        {
            let tokens = motion_tokens(&m);
            let parsed = og_core::ogt::Document::parse(&format!(
                "type style\nversion 2\n---\n{}\n",
                tokens.join("\n")
            ))
            .map_err(|e| e.to_string())?;
            for t in parsed.body.entries {
                let same = |x: &og_core::ogt::Entry| {
                    x.key == t.key && (t.key != "spring" || x.values.first() == t.values.first())
                };
                match b.entries.iter().position(same) {
                    Some(i) => b.entries[i] = t,
                    None => b.entries.push(t),
                }
            }
        }
        text = doc.to_text().map_err(|e| e.to_string())?;
    }
    let header = format!(
        "# {}: the {} look, generated by og style new (docs/ui.md §12.5) for \"{}\".\n# Choices: scheme {}, seed {}, shape {}, ornament {}, motion {}, finish {}.\n",
        c.name,
        c.look,
        c.premise.replace('"', "'"),
        if c.scheme == Scheme::Dark {
            "dark"
        } else {
            "light"
        },
        hex(c.seed),
        c.shape.name(),
        c.ornament.name(),
        c.motion.name(),
        c.finish.name()
    );
    let text = match text.split_once("---\n") {
        Some((head, body)) => format!("{head}---\n{header}{body}{}", dressings(c, &p)),
        None => return Err("the upgraded style has no header".into()),
    };
    crate::style_file::parse_style(&text).map_err(|e| format!("the generated style: {e}"))?;
    let k = &c.sound;
    let sounds = vec![
        (
            "sfx-tap.ogt".to_string(),
            sfx(k.tap.0, k.tap.1, k.tap.2, 0.8, "A press"),
        ),
        (
            "sfx-tap2.ogt".to_string(),
            sfx(
                k.tap.0,
                k.tap.1 + 1,
                k.tap.2 * 1.03,
                0.8,
                "A press, its second voice",
            ),
        ),
        (
            "sfx-confirm.ogt".to_string(),
            sfx(k.confirm.0, k.confirm.1, k.confirm.2, 0.85, "A confirm"),
        ),
        (
            "sfx-back.ogt".to_string(),
            sfx(k.back.0, k.back.1, k.back.2, 0.8, "A back or cancel"),
        ),
        (
            "sfx-open.ogt".to_string(),
            sfx(k.open.0, k.open.1, k.open.2, 0.6, "A dialog opening"),
        ),
        (
            "sfx-error.ogt".to_string(),
            sfx(k.error.0, k.error.1, k.error.2, 0.8, "An error"),
        ),
        (
            "sfx-reward.ogt".to_string(),
            sfx(k.reward.0, k.reward.1, k.reward.2, 0.85, "A reward"),
        ),
    ];
    Ok(Generated { text, sounds })
}

/// The art a generated style asks for (docs/ui.md §13.3): its frames,
/// buttons (each state), ornaments and icon set as slot specs, which the
/// art brief turns into prompts with the look's conform rules. Until the
/// art lands the style draws with its procedural dressings.
pub fn art_requests(c: &StyleChoices) -> Vec<SlotSpec> {
    let mut out = Vec::new();
    for part in ["panel", "card", "field", "slot", "tooltip", "hud"] {
        out.push(SlotSpec::new(
            &format!("{}/{part}", c.name),
            SlotKind::StyleFrame,
        ));
    }
    out.push(SlotSpec::new(
        &format!("{}/button", c.name),
        SlotKind::StyleButton,
    ));
    out.push(SlotSpec::new(
        &format!("{}/button-primary", c.name),
        SlotKind::StyleButton,
    ));
    if c.ornament != Ornament::None {
        for o in ["corner", "divider", "flourish"] {
            out.push(SlotSpec::new(
                &format!("{}/{o}", c.name),
                SlotKind::Ornament,
            ));
        }
    }
    out.push(SlotSpec::new(
        &format!("{}/icons", c.name),
        SlotKind::IconSet,
    ));
    out
}

/// The requests as the art brief reads them (`styleArtBriefs` in
/// `packages/knowledge/scripts/brief.ts`): the style, its look, premise
/// and choices, its palette, and each slot's spec.
pub fn art_requests_json(c: &StyleChoices) -> serde_json::Value {
    let p = palette(c);
    let palette: serde_json::Map<String, serde_json::Value> =
        ["backdrop", "panel", "ink", "primary", "line", "highlight"]
            .iter()
            .map(|n| {
                (
                    n.to_string(),
                    serde_json::json!(hex(p.get(n).expect("a role"))),
                )
            })
            .collect();
    serde_json::json!({
        "style": c.name,
        "look": c.look,
        "premise": c.premise,
        "choices": {
            "scheme": if c.scheme == Scheme::Dark { "dark" } else { "light" },
            "seed": hex(c.seed),
            "shape": c.shape.name(),
            "ornament": c.ornament.name(),
            "motion": c.motion.name(),
            "finish": c.finish.name(),
        },
        "palette": palette,
        "slots": art_requests(c)
            .iter()
            .map(crate::doc::build::slot_json)
            .collect::<Vec<_>>(),
    })
}

// ─────────────────────────────────────────────────────────── the seeds

#[allow(clippy::too_many_arguments)]
fn kit(
    tap: (&'static str, u32, f32),
    confirm: (&'static str, u32, f32),
    back: (&'static str, u32, f32),
    open: (&'static str, u32, f32),
    error: (&'static str, u32, f32),
    reward: (&'static str, u32, f32),
    volume: f32,
    hover: bool,
    spread: f32,
) -> SoundKit {
    SoundKit {
        tap,
        confirm,
        back,
        open,
        error,
        reward,
        volume,
        hover,
        spread,
    }
}

fn rgb(v: u32) -> Srgba {
    Srgba::rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

fn fonts(body: &str, display: &str, numeric: &str, cjk: &str, cjk_display: &str) -> [String; 5] {
    let shared = |f: &str| {
        if f.contains('/') {
            format!("@styles/{f}")
        } else {
            format!("@styles/cjk/{f}")
        }
    };
    [
        shared(body),
        shared(display),
        shared(numeric),
        shared(cjk),
        shared(cjk_display),
    ]
}

#[allow(clippy::too_many_arguments)]
fn seed(
    name: &str,
    look: &str,
    premise: &str,
    scheme: Scheme,
    colours: (u32, u32, Option<u32>),
    shape: ShapeLanguage,
    ornament: Ornament,
    motion: Motion,
    finish: Finish,
    fonts: [String; 5],
    icons: &str,
    body: f32,
    label_tracking: f32,
    sound: SoundKit,
) -> StyleChoices {
    StyleChoices {
        name: name.into(),
        look: look.into(),
        premise: premise.into(),
        scheme,
        seed: rgb(colours.0),
        ground: Some(rgb(colours.1)),
        ink: None,
        accent: colours.2.map(rgb),
        shape,
        ornament,
        motion,
        finish,
        fonts,
        icons: Some(format!("@styles/{icons}")),
        body,
        label_tracking,
        sound,
        glitch: 0.0,
        danger: None,
        info: None,
    }
}

/// The choices `og style new` starts from when a look shares no word with
/// any seed's family: a quiet light style of squircles and hairlines, with
/// no ornament and calm motion. It is not a built-in pack (the owner dropped
/// the `minimal` pack, 2026-11-02).
fn fallback() -> StyleChoices {
    use Finish as F;
    use Motion as M;
    use Ornament as O;
    use Scheme::Light;
    use ShapeLanguage as S;
    seed(
        "minimal",
        "minimal modern",
        "a clean, quiet strategy game where the numbers speak",
        Light,
        (0x3452ff, 0xf3f4f6, Some(0x0f9d8a)),
        S::Squircle,
        O::None,
        M::Calm,
        F::Flat,
        fonts(
            "shared/InstrumentSans-Regular.ttf",
            "shared/InstrumentSans-Regular.ttf",
            "shared/PTMono-Regular.ttf",
            "NotoSansSC-Regular.ttf",
            "NotoSansSC-Bold.ttf",
        ),
        "shared/ink-icons.png",
        15.0,
        0.0,
        kit(
            ("click", 16, 1.1),
            ("select", 17, 1.0),
            ("back", 18, 1.0),
            ("whoosh", 16, 1.1),
            ("error", 13, 1.0),
            ("select", 19, 1.2),
            0.4,
            false,
            0.03,
        ),
    )
}

/// The seed styles of `templates/styles` (docs/ui.md §12.5): one per look
/// family of og-taste §12, each generated from these choices. Their
/// directories are written by `og style new --seeds` and held to these
/// choices by og-ui's tests.
pub fn seeds() -> Vec<StyleChoices> {
    use Finish as F;
    use Motion as M;
    use Ornament as O;
    use Scheme::{Dark, Light};
    use ShapeLanguage as S;
    vec![
        seed(
            "gilded",
            "ornate fantasy",
            "a high-fantasy RPG of guilds, relics and gilded halls",
            Dark,
            (0xc9973a, 0x15100b, Some(0xe8c46a)),
            S::Notch,
            O::Rich,
            M::Heavy,
            F::Gradient,
            fonts(
                "fantasy/Newsreader-Medium.ttf",
                "fantasy/Cinzel-Bold.ttf",
                "fantasy/Cinzel-Bold.ttf",
                "NotoSerifSC-SemiBold.ttf",
                "NotoSerifSC-SemiBold.ttf",
            ),
            "fantasy/icons.png",
            15.0,
            0.0,
            kit(
                ("click", 4, 0.8),
                ("select", 2, 0.85),
                ("back", 1, 0.8),
                ("whoosh", 3, 0.7),
                ("error", 1, 0.85),
                ("levelup", 2, 0.9),
                0.7,
                false,
                0.04,
            ),
        ),
        seed(
            "storybook",
            "storybook",
            "a gentle illustrated adventure told like a picture book",
            Light,
            (0xb5523b, 0xefe3c8, Some(0x4f7d5b)),
            S::Soft,
            O::Rich,
            M::Floaty,
            F::Paper,
            fonts(
                "shared/InstrumentSans-Regular.ttf",
                "shared/Newsreader-SemiBold.ttf",
                "shared/Newsreader-SemiBold.ttf",
                "NotoSansSC-Regular.ttf",
                "NotoSerifSC-SemiBold.ttf",
            ),
            "shared/ink-icons.png",
            16.0,
            0.0,
            kit(
                ("pickup", 2, 1.2),
                ("select", 5, 1.1),
                ("back", 3, 1.05),
                ("whoosh", 6, 1.2),
                ("error", 2, 1.1),
                ("coin", 3, 1.0),
                0.55,
                false,
                0.08,
            ),
        ),
        seed(
            "tactical",
            "tactical military",
            "a squad tactics game of briefings, loadouts and grid maps",
            Dark,
            (0x9aae5c, 0x101210, Some(0xe0b13a)),
            S::Cut,
            O::Light,
            M::Mechanical,
            F::Flat,
            fonts(
                "shared/Rajdhani-Bold.ttf",
                "shared/ChakraPetch-SemiBold.ttf",
                "scifi/JetBrainsMono-Regular.ttf",
                "NotoSansSC-Regular.ttf",
                "NotoSansSC-Bold.ttf",
            ),
            "scifi/icons.png",
            15.0,
            0.12,
            kit(
                ("click", 7, 0.7),
                ("select", 8, 0.75),
                ("back", 4, 0.7),
                ("click", 9, 0.55),
                ("error", 3, 0.75),
                ("powerup", 4, 0.8),
                0.6,
                true,
                0.02,
            ),
        ),
        // The owner's approved sheet (ui-art-2 59079f25d): acid yellow,
        // signal red and cyan on near-black, 45° cuts, bold condensed type
        // and dense data text; no magenta, no glow; a glitch on
        // transitions only.
        StyleChoices {
            glitch: 0.8,
            danger: Some(rgb(0xff3b3b)),
            info: Some(rgb(0x2de2e6)),
            ..seed(
                "cyber",
                "cyberpunk signal",
                "a street-level heist game of black-market chrome and data runs in a megacity",
                Dark,
                (0xf5e60c, 0x0a0a0c, Some(0x2bf0ff)),
                S::Cut,
                O::Light,
                M::Snappy,
                F::Flat,
                fonts(
                    "shared/ChakraPetch-SemiBold.ttf",
                    "shared/Rajdhani-Bold.ttf",
                    "scifi/JetBrainsMono-Regular.ttf",
                    "NotoSansSC-Bold.ttf",
                    "NotoSansSC-Bold.ttf",
                ),
                "shared/glow-icons.png",
                15.0,
                0.08,
                kit(
                    ("laser", 11, 1.4),
                    ("powerup", 9, 1.3),
                    ("back", 8, 1.3),
                    ("whoosh", 10, 1.6),
                    ("error", 6, 1.3),
                    ("levelup", 7, 1.2),
                    0.6,
                    true,
                    0.06,
                ),
            )
        },
        // Concept C: navy ink on ivory, not ink of the gold's hue.
        StyleChoices {
            ink: Some(rgb(0x1d2a4e)),
            ..seed(
                "kawaii",
                "kawaii anime gacha",
                "an anime adventure gacha of bright skies, ivory halls and star pulls",
                Light,
                (0xd6a443, 0xf7f0e3, Some(0x283a6e)),
                S::Pill,
                O::Light,
                M::Bouncy,
                F::Glass,
                fonts(
                    "cartoon/Fredoka-SemiBold.ttf",
                    "cartoon/LilitaOne-Regular.ttf",
                    "cartoon/Fredoka-SemiBold.ttf",
                    "ZCOOLKuaiLe-Regular.ttf",
                    "ZCOOLKuaiLe-Regular.ttf",
                ),
                "cartoon/icons.png",
                15.0,
                0.0,
                kit(
                    ("pickup", 14, 1.6),
                    ("coin", 8, 1.35),
                    ("back", 12, 1.4),
                    ("powerup", 11, 1.5),
                    ("error", 9, 1.4),
                    ("levelup", 12, 1.3),
                    0.6,
                    false,
                    0.12,
                ),
            )
        },
        seed(
            "craft",
            "paper and craft",
            "a cosy crafting sim of cut paper, stickers and tape",
            Light,
            (0x2f7d6d, 0xd9bf94, Some(0xd9623b)),
            S::Round,
            O::Light,
            M::Snappy,
            F::Paper,
            fonts(
                "shared/InstrumentSans-Regular.ttf",
                "cartoon/LilitaOne-Regular.ttf",
                "shared/PTMono-Regular.ttf",
                "NotoSansSC-Regular.ttf",
                "ZCOOLKuaiLe-Regular.ttf",
            ),
            "shared/ink-icons.png",
            15.0,
            0.0,
            kit(
                ("footstep", 4, 1.4),
                ("select", 14, 1.0),
                ("back", 16, 0.95),
                ("whoosh", 14, 1.0),
                ("error", 11, 1.0),
                ("coin", 15, 0.95),
                0.55,
                false,
                0.09,
            ),
        ),
        seed(
            "dread",
            "diegetic horror",
            "a survival horror game read through notes, tapes and a dying torch",
            Dark,
            (0xa3241f, 0x0c0a09, Some(0xc9b28a)),
            S::Sharp,
            O::Light,
            M::Heavy,
            F::Brush,
            fonts(
                "fantasy/Newsreader-Medium.ttf",
                "shared/Newsreader-SemiBold.ttf",
                "shared/PTMono-Regular.ttf",
                "NotoSerifSC-SemiBold.ttf",
                "NotoSerifSC-SemiBold.ttf",
            ),
            "fantasy/icons.png",
            15.0,
            0.0,
            kit(
                ("footstep", 9, 0.6),
                ("door", 3, 0.8),
                ("back", 20, 0.6),
                ("door", 5, 0.6),
                ("hurt", 6, 0.7),
                ("death", 2, 1.2),
                0.5,
                false,
                0.06,
            ),
        ),
        seed(
            "broadcast",
            "sports broadcast",
            "an arcade football game framed like a live TV broadcast",
            Dark,
            (0xe8322b, 0x0a1431, Some(0xffd21a)),
            S::Sharp,
            O::None,
            M::Snappy,
            F::Gradient,
            fonts(
                "shared/Rajdhani-Bold.ttf",
                "shared/ChakraPetch-BoldItalic.ttf",
                "shared/ChakraPetch-SemiBold.ttf",
                "NotoSansSC-Bold.ttf",
                "NotoSansSC-Bold.ttf",
            ),
            "scifi/icons.png",
            15.0,
            0.08,
            kit(
                ("whoosh", 20, 1.8),
                ("select", 21, 1.2),
                ("back", 22, 1.15),
                ("whoosh", 22, 1.0),
                ("error", 15, 1.1),
                ("levelup", 18, 1.1),
                0.65,
                false,
                0.04,
            ),
        ),
        // The owner's approved sheets (jade-cultivation, jade-menu,
        // jade-settings): carved jade-green panels edged in gold leaf,
        // cloud filigree in their corners, gold primaries with dark ink,
        // brush calligraphy for display (Kaushan Script, Ma Shan Zheng)
        // over an old-style serif; unhurried motion like drifting silk.
        seed(
            "jade",
            "eastern jade xianxia",
            "a xianxia cultivation saga of sects, flying swords and jade pavilions above the clouds",
            Dark,
            (0xdcbc72, 0x0b1a16, Some(0xf2d68c)),
            S::Soft,
            O::Rich,
            M::Calm,
            F::Gradient,
            fonts(
                "shared/EBGaramond-Medium.ttf",
                "shared/KaushanScript-Regular.ttf",
                "shared/EBGaramond-Medium.ttf",
                "NotoSerifSC-SemiBold.ttf",
                "MaShanZheng-Regular.ttf",
            ),
            "fantasy/icons.png",
            16.0,
            0.0,
            kit(
                ("select", 25, 1.35),
                ("pickup", 21, 1.1),
                ("back", 26, 0.9),
                ("whoosh", 28, 0.7),
                ("error", 20, 0.85),
                ("levelup", 24, 0.95),
                0.5,
                false,
                0.05,
            ),
        ),
        // The owner's approved sheets (midcore-battle, midcore-menu,
        // midcore-settings): chunky slate-stone panels with thick dark
        // outlines and bevels, a glossy green primary, blue energy bars,
        // gold selection, heavy rounded type (Titan One); springy,
        // punchy presses.
        StyleChoices {
            info: Some(rgb(0x3b9bff)),
            ..seed(
                "midcore",
                "mobile midcore strategy",
                "a competitive card battler of chunky heroes, stone forts and lane pushes on floating islands",
                Dark,
                (0x52c234, 0x1c2128, Some(0xffc83a)),
                S::Round,
                O::Light,
                M::Bouncy,
                F::Bevel,
                fonts(
                    "shared/Rubik-SemiBold.ttf",
                    "shared/TitanOne-Regular.ttf",
                    "shared/TitanOne-Regular.ttf",
                    "NotoSansSC-Bold.ttf",
                    "ZCOOLQingKeHuangYou-Regular.ttf",
                ),
                "cartoon/icons.png",
                16.0,
                0.0,
                kit(
                    ("click", 21, 0.9),
                    ("coin", 22, 1.05),
                    ("back", 25, 0.95),
                    ("whoosh", 27, 0.9),
                    ("error", 19, 0.95),
                    ("levelup", 22, 1.05),
                    0.7,
                    false,
                    0.08,
                ),
            )
        },
    ]
}

/// The look families of the seeds folded into other packs (the owner,
/// 2026-11-02: holo into scifi, bubbly into cartoon, arcade into pixel,
/// inkwash into jade): no built-in pack ships them, but `og style new`
/// still starts from their choices for a look of their family, since the
/// packs that absorbed them are painted atlas packs, not choices it can
/// start from.
fn retired() -> Vec<StyleChoices> {
    use Finish as F;
    use Motion as M;
    use Ornament as O;
    use Scheme::{Dark, Light};
    use ShapeLanguage as S;
    vec![
        seed(
            "holo",
            "sci-fi holographic",
            "a starship command game on holographic consoles",
            Dark,
            (0x3cc8ff, 0x050d16, Some(0x7ef0ff)),
            S::Cut,
            O::Light,
            M::Snappy,
            F::Hologram,
            fonts(
                "scifi/ChakraPetch-Medium.ttf",
                "scifi/Orbitron-Bold.ttf",
                "scifi/JetBrainsMono-Regular.ttf",
                "NotoSansSC-Regular.ttf",
                "NotoSansSC-Bold.ttf",
            ),
            "scifi/icons.png",
            15.0,
            0.06,
            kit(
                ("laser", 5, 1.8),
                ("select", 9, 1.3),
                ("back", 6, 1.2),
                ("whoosh", 8, 1.4),
                ("error", 4, 1.2),
                ("powerup", 6, 1.15),
                0.5,
                true,
                0.05,
            ),
        ),
        seed(
            "bubbly",
            "casual cartoon",
            "a bright match-and-collect puzzle game for everyone",
            Light,
            (0x2f7bff, 0xffd36e, Some(0xff7a1a)),
            S::Round,
            O::Light,
            M::Bouncy,
            F::Gradient,
            fonts(
                "cartoon/Fredoka-SemiBold.ttf",
                "cartoon/LilitaOne-Regular.ttf",
                "cartoon/LilitaOne-Regular.ttf",
                "ZCOOLKuaiLe-Regular.ttf",
                "ZCOOLKuaiLe-Regular.ttf",
            ),
            "cartoon/icons.png",
            16.0,
            0.0,
            kit(
                ("pickup", 12, 1.3),
                ("coin", 5, 1.1),
                ("back", 10, 1.2),
                ("jump", 4, 1.3),
                ("error", 7, 1.2),
                ("levelup", 9, 1.05),
                0.7,
                false,
                0.1,
            ),
        ),
        seed(
            "arcade",
            "pixel retro",
            "an 8-bit arcade platformer with a high-score table",
            Dark,
            (0xffcc1a, 0x0f0f26, Some(0x40e0ff)),
            S::Pixel,
            O::None,
            M::Mechanical,
            F::Bevel,
            fonts(
                // The pixel pack's one face, Latin and Chinese on one
                // 12-pixel grid (its import's `pixel 12`).
                "FusionPixel12-zhHans.ttf",
                "FusionPixel12-zhHans.ttf",
                "FusionPixel12-zhHans.ttf",
                "FusionPixel12-zhHans.ttf",
                "FusionPixel12-zhHans.ttf",
            ),
            "pixel/icons.png",
            16.0,
            0.0,
            kit(
                ("jump", 15, 1.6),
                ("coin", 13, 1.0),
                ("back", 14, 1.0),
                ("powerup", 13, 1.1),
                ("hurt", 3, 1.0),
                ("levelup", 14, 1.0),
                0.6,
                false,
                0.0,
            ),
        ),
        seed(
            "inkwash",
            "ink and eastern brush",
            "a wuxia journey painted in ink on rice paper",
            Light,
            (0xb22f26, 0xece3cf, Some(0x2d4a43)),
            S::Soft,
            O::Light,
            M::Calm,
            F::Brush,
            fonts(
                "fantasy/Newsreader-Medium.ttf",
                "shared/Newsreader-SemiBold.ttf",
                "shared/Newsreader-SemiBold.ttf",
                "NotoSerifSC-SemiBold.ttf",
                "NotoSerifSC-SemiBold.ttf",
            ),
            "fantasy/icons.png",
            16.0,
            0.0,
            kit(
                ("whoosh", 24, 0.9),
                ("select", 23, 0.8),
                ("back", 24, 0.8),
                ("whoosh", 26, 0.75),
                ("error", 17, 0.85),
                ("levelup", 20, 0.85),
                0.5,
                false,
                0.05,
            ),
        ),
    ]
}

impl StyleChoices {
    /// The choices of the seed style whose look family is nearest `look`
    /// (the most words in common; [`fallback`]'s quiet light choices when
    /// none), renamed:
    /// what `og style new` starts from before its flags.
    pub fn for_look(name: &str, look: &str, premise: &str) -> StyleChoices {
        let words = |s: &str| -> Vec<String> {
            s.to_lowercase()
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| w.len() > 2)
                .map(str::to_string)
                .collect()
        };
        let want = words(look);
        let mut all = seeds();
        all.extend(retired());
        let best = all
            .iter()
            .map(|s| {
                let have = words(&s.look);
                let hits = want
                    .iter()
                    .filter(|w| have.contains(w) || **w == s.name)
                    .count();
                (hits, s)
            })
            .filter(|(h, _)| *h > 0)
            .max_by_key(|(h, _)| *h)
            .map(|(_, s)| s.clone());
        let mut c = best.unwrap_or_else(fallback);
        c.name = name.into();
        c.look = look.into();
        c.premise = premise.into();
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_seed_and_every_choice_generates_a_style_that_parses() {
        let mut all = seeds();
        let base = StyleChoices::for_look("matrix", "minimal modern", "a test");
        for (i, &shape) in ShapeLanguage::ALL.iter().enumerate() {
            for &finish in Finish::ALL {
                let mut c = base.clone();
                c.name = format!("m-{}-{}", shape.name(), finish.name());
                c.shape = shape;
                c.finish = finish;
                c.motion = Motion::ALL[i % Motion::ALL.len()];
                c.ornament = Ornament::ALL[i % 3];
                c.scheme = if i % 2 == 0 {
                    Scheme::Dark
                } else {
                    Scheme::Light
                };
                c.ground = None;
                all.push(c);
            }
        }
        for c in &all {
            let g = generate(c).unwrap_or_else(|e| panic!("{}: {e}", c.name));
            let f = crate::style_file::parse_style(&g.text).unwrap();
            assert_eq!(f.version, 2, "{}", c.name);
            assert_eq!(g.sounds.len(), 7);
            assert!(!f.dressings.is_empty(), "{}", c.name);
        }
    }
}
