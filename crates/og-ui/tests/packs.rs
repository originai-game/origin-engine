//! The engine's built-in UI styles (templates/styles/<name>/style.ogt
//! and atlas.png) parse, and every pack keeps body text legible: each ink a
//! template puts on a styled surface holds 3:1 (under 4.5:1 a printed
//! warning, docs/ui.md §16.5) against what is
//! really drawn there (the frame's art multiplied by its tint, composited
//! over the pack's `backdrop`, measured over the frame's middle).
//!
//! Ways this could fail: a pack whose palette reads on paper but whose
//! frame art darkens or lightens the fill under the text; a translucent
//! panel whose ink only reads over one backdrop; a selected tab (`knob`)
//! in the same tone as its text; a pack missing, or a typo in its file.

use og_ui::style_file::{Slice, StylePart};
use og_ui::*;
use std::path::PathBuf;

/// Where, the fill a template gives it, the frame, the inks on it.
type Check = (&'static str, Srgba, StylePart, Vec<(&'static str, Srgba)>);

const PACKS: [&str; 4] = ["pixel", "cartoon", "fantasy", "scifi"];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates/ui-kit/assets")
}

fn lin(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Linear RGB of an sRGB colour.
fn linear(c: Srgba) -> [f32; 3] {
    let l = c.linear();
    [l[0], l[1], l[2]]
}

fn luminance(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

fn ratio(a: [f32; 3], b: [f32; 3]) -> f32 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

fn over(top: [f32; 3], a: f32, below: [f32; 3]) -> [f32; 3] {
    [0, 1, 2].map(|i| top[i] * a + below[i] * (1.0 - a))
}

/// What a frame draws under its text: the mean of its middle (inside the
/// corner insets), tinted, over `backdrop`.
fn under(atlas: &image::RgbaImage, s: &Slice, tint: Srgba, bg: [f32; 3]) -> [f32; 3] {
    let (w, h) = (atlas.width() as f32, atlas.height() as f32);
    let x0 = ((s.uv[0] + s.border_uv[0]) * w).ceil() as u32;
    let y0 = ((s.uv[1] + s.border_uv[1]) * h).ceil() as u32;
    let x1 = ((s.uv[2] - s.border_uv[2]) * w).floor() as u32;
    let y1 = ((s.uv[3] - s.border_uv[3]) * h).floor() as u32;
    let (mut sum, mut n) = ([0.0f32; 4], 0.0f32);
    for y in y0..y1.max(y0 + 1) {
        for x in x0..x1.max(x0 + 1) {
            let p = atlas.get_pixel(x, y).0;
            for k in 0..3 {
                sum[k] += lin(f32::from(p[k]) / 255.0);
            }
            sum[3] += f32::from(p[3]) / 255.0;
            n += 1.0;
        }
    }
    let tex = [sum[0] / n, sum[1] / n, sum[2] / n];
    let tex_a = sum[3] / n;
    let (rgb, a) = if s.tint {
        let t = linear(tint);
        (
            [tex[0] * t[0], tex[1] * t[1], tex[2] * t[2]],
            tex_a * tint.alpha,
        )
    } else {
        (tex, tex_a * tint.alpha)
    };
    over(rgb, a, bg)
}

#[test]
fn every_style_pack_parses_and_keeps_body_text_at_4_5_to_1() {
    let mut report = Vec::new();
    let mut failures = Vec::new();
    for name in PACKS {
        let dir = root().join("../../styles").join(name);
        let text = std::fs::read_to_string(dir.join("style.ogt"))
            .unwrap_or_else(|e| panic!("{name}: style.ogt: {e}"));
        let mut file = parse_style(&text).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(file.name, name);
        // A built-in style names only its own files and the shared built-in
        // Chinese companions: it is cooked into any game as `@styles/<name>/`
        // (with `@styles/cjk/`), where nothing else of the game is.
        let styles = root().join("../../styles");
        file.resolve(&format!("{name}/style.ogt"), |id| styles.join(id).exists());
        for r in file.references() {
            let shared = r.strip_prefix("@styles/").filter(|r| r.starts_with("cjk/"));
            let path = shared.map_or_else(|| r.clone(), str::to_string);
            assert!(
                (shared.is_some() || r.starts_with(&format!("{name}/")))
                    && styles.join(&path).exists(),
                "{name} names {r}, which is neither beside its style file nor a shared companion"
            );
        }
        assert!(
            file.fonts[4].is_some() && file.fonts[5].is_some(),
            "{name} names its Chinese companions"
        );
        let atlas = image::open(dir.join("atlas.png"))
            .expect("atlas.png")
            .to_rgba8();
        let t = file.theme;
        let p = t.palette;
        let bg = linear(p.backdrop);
        let style = t.art;
        let slice = |part: StylePart| {
            style.slices[part.index()]
                .or(style.slices[StylePart::Button.index()]
                    .filter(|_| part.class() == StylePart::Button))
                .unwrap_or_else(|| panic!("{name} has no {} slice", part.name()))
        };
        // A tooltip's label takes the one of fg and inkInverse that reads on
        // its frame (og-ui's `Ui::tooltip`): a painted bubble may be dark.
        let tip = under(&atlas, &slice(StylePart::Tooltip), p.panel_raised, bg);
        let tip_ink = if ratio(linear(p.ink_inverse), tip) > ratio(linear(p.ink), tip) {
            ("inkInverse", p.ink_inverse)
        } else {
            ("fg", p.ink)
        };
        // (where, the fill a template gives it, the frame, the inks on it)
        let checks: Vec<Check> = vec![
            (
                "dialog",
                p.panel_raised,
                StylePart::Panel,
                vec![("fg", p.ink), ("fgMuted", p.ink_muted)],
            ),
            (
                "screen panel",
                p.hud,
                StylePart::Panel,
                vec![("fg", p.ink), ("fgMuted", p.ink_muted)],
            ),
            (
                "gallery panel",
                p.panel,
                StylePart::Panel,
                vec![("fg", p.ink), ("fgMuted", p.ink_muted)],
            ),
            (
                "card",
                p.panel,
                StylePart::Card,
                vec![("fg", p.ink), ("fgMuted", p.ink_muted)],
            ),
            ("button", p.control, StylePart::Button, vec![("fg", p.ink)]),
            (
                "hovered button",
                p.control_strong,
                StylePart::ButtonHover,
                vec![("fg", p.ink)],
            ),
            (
                "primary button",
                p.primary,
                StylePart::Button,
                vec![("accentFg", p.on_primary)],
            ),
            (
                "hovered primary",
                p.primary_hover,
                StylePart::ButtonHover,
                vec![("accentFg", p.on_primary)],
            ),
            ("HUD capsule", p.hud, StylePart::Hud, vec![("fg", p.ink)]),
            ("tooltip", p.panel_raised, StylePart::Tooltip, vec![tip_ink]),
            ("field", p.control, StylePart::Field, vec![("fg", p.ink)]),
            ("slot", p.control, StylePart::Slot, vec![("fg", p.ink)]),
        ];
        for (what, fill, part, inks) in checks {
            let ground = under(&atlas, &slice(part), fill, bg);
            for (ink, c) in inks {
                let r = ratio(over(linear(c), c.alpha, ground), ground);
                report.push(format!("{name:8} {ink:9} on {what:16} {r:5.2}:1"));
                // Live text: an error under 3:1; under 4.5:1 a warning
                // the art director judges (docs/ui.md §15.4, §16.5).
                if r < 3.0 {
                    failures.push(format!(
                        "{name}: {ink} on the {what} is {r:.2}:1 (under 3:1)"
                    ));
                } else if r < 4.5 {
                    report.push(format!(
                        "  warning: {name}: {ink} on the {what} under 4.5:1"
                    ));
                }
            }
        }
        // A selected tab: its label on the thumb, over the dialog, in the
        // first of fg, inkInverse and onPrimary that reads 4.5:1 there, as
        // og-ui's tabs pick it.
        let dialog = under(&atlas, &slice(StylePart::Panel), p.panel_raised, bg);
        let tab = over(linear(p.knob), p.knob.alpha, dialog);
        let inks = [
            ("fg", p.ink),
            ("inkInverse", p.ink_inverse),
            ("onPrimary", p.on_primary),
        ];
        let (ink, r) = inks
            .iter()
            .map(|&(n, c)| (n, ratio(linear(c), tab)))
            .find(|&(_, r)| r >= 4.5)
            .unwrap_or(("fg", ratio(linear(p.ink), tab)));
        report.push(format!("{name:8} {ink:9} on selected tab     {r:5.2}:1"));
        if r < 3.0 {
            failures.push(format!(
                "{name}: {ink} on a selected tab (thumb) is {r:.2}:1 (under 3:1)"
            ));
        } else if r < 4.5 {
            report.push(format!(
                "  warning: {name}: {ink} on a selected tab under 4.5:1"
            ));
        }
    }
    println!("{}", report.join("\n"));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Every built-in style is version 2 with tokens of its own (not the ones
/// `og style upgrade` derives), and the colours that carry meaning without
/// text (rarity, teams, damage types, chart series) read 3:1 on its raised
/// panel over its backdrop (WCAG 1.4.11), each status and rarity with its
/// own cue (the parser refuses a shared one).
#[test]
fn every_built_in_style_is_version_2_with_colour_tokens_that_read_on_its_panels() {
    let mut failures = Vec::new();
    for name in PACKS {
        let dir = root().join("../../styles").join(name);
        let file = parse_style(&std::fs::read_to_string(dir.join("style.ogt")).expect("style.ogt"))
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(file.version, 2, "{name} is version 2");
        let t = file.theme;
        let icons: Vec<String> = file.icons.map(|i| i.3).unwrap_or_default();
        assert_ne!(
            t.tokens,
            og_ui::StyleTokens::derive(&t, &icons),
            "{name} sets its own tokens"
        );
        let ground = t.palette.panel_raised.over(t.palette.backdrop);
        let c = &t.tokens.colors;
        let named = c
            .rarity
            .iter()
            .chain(&c.team)
            .chain(&c.damage)
            .map(|s| s.color)
            .chain(c.series);
        for (i, color) in named.enumerate() {
            let r = color.contrast(ground);
            if r < 3.0 {
                failures.push(format!(
                    "{name}: colour token {i} reads {r:.2}:1 on its panel"
                ));
            }
        }
        // The resource inks are text too ("+1,240 XP" at body size on a
        // results panel): 4.5:1 on the panel, as body text.
        let pal = &t.palette;
        for (what, ink) in [
            ("health", pal.health),
            ("mana", pal.mana),
            ("xp", pal.xp),
            ("gold", pal.gold),
        ] {
            let r = ink.contrast(pal.panel);
            if r < 4.5 {
                failures.push(format!("{name}: {what} reads {r:.2}:1 on its panel"));
            }
        }
        // A button's words are never a speck on its plate: at least the
        // label step, and in a pixel face (label is a HUD's 12) the body.
        use og_ui::tokens::TypeRole as R;
        let size = |r: R| t.tokens.typography.get(r, Breakpoint::Desktop).size;
        assert!(size(R::Button) >= size(R::Label), "{name}");
        if name == "pixel" {
            assert_eq!((size(R::Label), size(R::Button)), (12.0, size(R::Body)));
        }
        // The widgets still draw from the version 1 values: the corners of
        // the four classes version 1 had are its radii.
        use og_ui::tokens::ElementClass as E;
        assert_eq!(
            [E::Slot, E::Button, E::Bar, E::Panel].map(|k| t.tokens.shape(k).radius),
            [
                t.radius.small,
                t.radius.control,
                t.radius.bar,
                t.radius.panel
            ],
            "{name}"
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// A style's text without what og style art packs into it: the atlas
/// line, authored ornament lines and each dressing's slice and tone (a
/// dressing left empty goes too).
fn unpainted(text: &str) -> String {
    use og_core::ogt::{Document, Value};
    let mut doc = Document::parse(text).expect("a style parses");
    let authored = |vs: &[Value]| vs.iter().any(|v| v.as_word() == Some("authored"));
    doc.body.entries.retain(|e| e.key != "atlas");
    doc.body
        .entries
        .retain(|e| e.key == "dress" || !authored(&e.values));
    for e in doc.body.entries.iter_mut().filter(|e| e.key == "dress") {
        if let Some(b) = e.block.as_mut() {
            b.entries.retain(|c| c.key != "slice" && c.key != "tone");
        }
    }
    doc.body
        .entries
        .retain(|e| e.key != "dress" || e.block.as_ref().is_some_and(|b| !b.entries.is_empty()));
    doc.to_text().expect("a style writes back")
}

/// The seed styles of templates/styles (docs/ui.md §12.5) are what
/// `og style new --seeds` writes from their recorded choices
/// (`style_gen::seeds`): a seed changed by hand, or a generator change
/// whose seeds were not rewritten, fails here. Each is a complete
/// version 2 style with no atlas until its art is painted (then it is
/// that style with og style art's packing), every part dressed procedurally, a
/// sound for every event that names one, and its own shape language,
/// motion personality and sounds: no two seeds share all three.
#[test]
fn every_seed_style_is_what_og_style_new_writes_from_its_choices() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates/styles");
    let seeds = og_ui::style_gen::seeds();
    assert!(seeds.len() >= 8, "{} seeds", seeds.len());
    let mut signatures = Vec::new();
    for c in &seeds {
        let dir = root.join(&c.name);
        let g = og_ui::style_gen::generate(c).unwrap_or_else(|e| panic!("{}: {e}", c.name));
        let on_disk = std::fs::read_to_string(dir.join("style.ogt"))
            .unwrap_or_else(|e| panic!("{}: {e} (run og style new --seeds)", c.name));
        // A painted seed is its generated style with its art packed in by
        // og style art (docs/ui.md §12.6): the same without what it adds.
        let painted = dir.join("art/report.json").is_file();
        let ours = if painted {
            unpainted(&on_disk)
        } else {
            on_disk.clone()
        };
        assert!(
            ours == if painted {
                unpainted(&g.text)
            } else {
                g.text.clone()
            },
            "{}: style.ogt is not what og style new --seeds writes",
            c.name
        );
        for (f, text) in &g.sounds {
            let disk = std::fs::read_to_string(dir.join(f)).unwrap_or_default();
            assert!(
                &disk == text,
                "{}: {f} is not what og style new writes",
                c.name
            );
        }
        let file = parse_style(&on_disk).unwrap_or_else(|e| panic!("{}: {e}", c.name));
        assert_eq!(file.version, 2, "{}", c.name);
        assert!(
            file.atlas.is_none() || painted,
            "{}: a seed draws without an atlas until its art is painted",
            c.name
        );
        for e in og_ui::tokens::SoundEvent::ALL {
            let f = &file.feedback_sounds[e.index()];
            for s in f.iter() {
                assert!(
                    dir.join(s).is_file(),
                    "{}: {} names {s}, which is missing",
                    c.name,
                    e.name()
                );
            }
        }
        assert!(
            file.feedback_sounds.iter().any(|s| !s.is_empty()),
            "{}: no sounds",
            c.name
        );
        let sig = (c.shape, c.motion, c.sound.tap);
        assert!(
            !signatures.contains(&sig),
            "{}: the same shape, motion and press sound as another seed",
            c.name
        );
        signatures.push(sig);
    }
}

#[test]
fn a_styles_fonts_name_weights_italics_and_each_locales_companion() {
    let text = std::fs::read_to_string(root().join("../../styles/scifi/style.ogt")).unwrap();
    let fonts = r#"fonts {
  body 300 "Light.ttf"
  body 600 "SemiBold.ttf"
  body italic "Italic.ttf"
  display 800 italic "BlackItalic.ttf"
  numeric "Mono.ttf"
  cjk "SC.ttf"
  cjk ja "JP.ttf"
  cjkDisplay ko 700 "KR-Bold.ttf"
}"#;
    let a = text.find("\nfonts {").expect("a fonts block") + 1;
    let b = a + text[a..].find("\n}").unwrap() + 2;
    let file = parse_style(&format!("{}{fonts}{}", &text[..a], &text[b..]))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(file.fonts[0].as_deref(), Some("Light.ttf"));
    assert_eq!(file.font_weights[0], 300);
    assert_eq!(file.fonts[1], None, "display names only an italic");
    assert_eq!(file.fonts[4].as_deref(), Some("SC.ttf"));
    let faces: Vec<_> = file
        .font_faces
        .iter()
        .map(|f| (f.role, f.cjk, f.locale, f.weight, f.italic, f.path.as_str()))
        .collect();
    assert_eq!(
        faces,
        [
            (
                FontRole::Body,
                false,
                CjkLocale::Hans,
                600,
                false,
                "SemiBold.ttf"
            ),
            (
                FontRole::Body,
                false,
                CjkLocale::Hans,
                400,
                true,
                "Italic.ttf"
            ),
            (
                FontRole::Display,
                false,
                CjkLocale::Hans,
                800,
                true,
                "BlackItalic.ttf"
            ),
            (FontRole::Body, true, CjkLocale::Ja, 400, false, "JP.ttf"),
            (
                FontRole::Display,
                true,
                CjkLocale::Ko,
                700,
                false,
                "KR-Bold.ttf"
            ),
        ]
    );
    // Each companion with its forms, so the cook ships only the shown ones.
    assert_eq!(
        file.companions(),
        [
            (CjkLocale::Hans, "SC.ttf".to_string()),
            (CjkLocale::Ja, "JP.ttf".to_string()),
            (CjkLocale::Ko, "KR-Bold.ttf".to_string()),
        ]
    );
    assert!(file.references().contains(&"BlackItalic.ttf".to_string()));
    for bad in [
        r#"body 650 "x.ttf""#,
        r#"cjk fr "x.ttf""#,
        r#"body ja "x.ttf""#,
    ] {
        let f = format!("fonts {{\n  {bad}\n}}");
        let e = parse_style(&format!("{}{f}{}", &text[..a], &text[b..])).unwrap_err();
        assert!(e.contains("line"), "{bad}: {e}");
    }
}
