//! The engine's built-in UI styles (templates/styles/<name>/style.ogt
//! and atlas.png) parse, and every pack keeps body text legible: each ink a
//! template puts on a styled surface reaches WCAG's 4.5:1 against what is
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

const PACKS: [&str; 6] = ["pixel", "cartoon", "fantasy", "scifi", "paper", "neon"];

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
            (
                "tooltip",
                p.panel_raised,
                StylePart::Tooltip,
                vec![("fg", p.ink)],
            ),
            ("field", p.control, StylePart::Field, vec![("fg", p.ink)]),
            ("slot", p.control, StylePart::Slot, vec![("fg", p.ink)]),
        ];
        for (what, fill, part, inks) in checks {
            let ground = under(&atlas, &slice(part), fill, bg);
            for (ink, c) in inks {
                let r = ratio(over(linear(c), c.alpha, ground), ground);
                report.push(format!("{name:8} {ink:9} on {what:16} {r:5.2}:1"));
                if r < 4.5 {
                    failures.push(format!(
                        "{name}: {ink} on the {what} is {r:.2}:1 (under 4.5:1)"
                    ));
                }
            }
        }
        // A selected tab: fg on thumb, over the dialog.
        let dialog = under(&atlas, &slice(StylePart::Panel), p.panel_raised, bg);
        let tab = over(linear(p.knob), p.knob.alpha, dialog);
        let r = ratio(linear(p.ink), tab);
        report.push(format!("{name:8} fg        on selected tab     {r:5.2}:1"));
        if r < 4.5 {
            failures.push(format!(
                "{name}: fg on a selected tab (thumb) is {r:.2}:1 (under 4.5:1)"
            ));
        }
    }
    println!("{}", report.join("\n"));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
