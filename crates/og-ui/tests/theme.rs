//! The default style is the design language's: `src/theme.rs` is exactly
//! what the generator writes from `packages/design/tokens.json` (a hand
//! edit, a stale file or a token changed without regenerating fail here),
//! its glass tints are og-render's presets, and its dressings parse and put
//! glass on floating chrome only.

#[path = "../scripts/theme_codegen.rs"]
mod theme_codegen;

use og_ui::{Selector, Srgba};
use std::path::PathBuf;

fn tokens() -> serde_json::Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(
        &std::fs::read_to_string(root.join("../../packages/design/tokens.json"))
            .expect("tokens.json"),
    )
    .expect("tokens.json parses")
}

#[test]
fn theme_rs_is_generated_from_the_design_tokens() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tokens = std::fs::read_to_string(root.join("../../packages/design/tokens.json"))
        .expect("tokens.json");
    let committed = std::fs::read_to_string(root.join("src/theme.rs")).expect("theme.rs");
    let generated = theme_codegen::generate(&tokens).expect("the tokens generate");
    if committed != generated {
        let line = committed
            .lines()
            .zip(generated.lines())
            .position(|(a, b)| a != b)
            .unwrap_or(committed.lines().count().min(generated.lines().count()));
        panic!(
            "src/theme.rs differs from the generator's output at line {}:\n  committed: {:?}\n  generated: {:?}\nrun crates/og-ui/scripts/gen-theme.sh (never edit theme.rs by hand)",
            line + 1,
            committed.lines().nth(line),
            generated.lines().nth(line)
        );
    }
    // The theme the toolkit starts from is the generated one.
    assert_eq!(og_ui::Theme::dark().palette, og_ui::theme::DARK);
    assert_eq!(og_ui::Theme::light().palette, og_ui::theme::LIGHT);
    assert_eq!(og_ui::Theme::dark().radius, og_ui::theme::RADIUS);
}

/// The tokens' material tints are the ones og-render's glass draws: a
/// tint changed on one side only would hold the language's contrast pairs
/// in the test and not on the screen.
#[test]
fn the_tokens_glass_tints_are_og_renders_presets() {
    use og_render::{UiGlassStyle, UiGlassWeight};
    let t = tokens();
    for (dark, scheme) in [(false, &t["color"]), (true, &t["dark"]["color"])] {
        for (weight, key) in [
            (UiGlassWeight::Thin, "material-thin"),
            (UiGlassWeight::Regular, "material-regular"),
            (UiGlassWeight::Thick, "material-thick"),
        ] {
            let s = scheme[key]
                .as_str()
                .or(t["color"][key].as_str())
                .expect(key);
            let inner = s.trim_start_matches("rgb(").trim_end_matches(')');
            let (rgb, a) = inner.split_once('/').expect("an alpha");
            let c: Vec<u8> = rgb
                .split_whitespace()
                .map(|p| p.parse().expect("a channel"))
                .collect();
            let want = Srgba {
                rgb: [c[0], c[1], c[2]],
                alpha: a.trim().parse().expect("an alpha"),
            }
            .linear();
            let (_, got) = UiGlassStyle {
                weight,
                dark,
                ..UiGlassStyle::default()
            }
            .backdrop();
            for k in 0..4 {
                assert!(
                    (want[k] - got[k]).abs() < 1e-4,
                    "{key} ({}): tokens {want:?}, og-render {got:?}",
                    if dark { "dark" } else { "light" }
                );
            }
        }
    }
}

/// Floating chrome is glass and nothing else is: dialogs, sheets, menus,
/// popovers and tooltips thick, HUD capsules regular; cards and controls
/// carry no backdrop, and controls no border.
#[test]
fn the_default_dressings_put_glass_on_floating_chrome_only() {
    use og_ui::paint::Effect;
    let d = og_ui::default_dressings();
    let glass = |sel: &str| -> Option<og_render::UiGlassWeight> {
        let s = Selector::parse(sel).expect("a selector");
        d.iter().find(|x| x.selector == s).and_then(|x| {
            x.draw.effects.iter().find_map(|e| match e {
                Effect::Backdrop { glass, .. } => glass.weight,
                _ => None,
            })
        })
    };
    use og_render::UiGlassWeight::{Regular, Thick};
    for s in [
        "Panel.frame",
        "Sheet.frame",
        "Menu.frame",
        "Popover.frame",
        "Tooltip.frame",
    ] {
        assert_eq!(glass(s), Some(Thick), "{s}");
    }
    assert_eq!(glass("Hud.frame"), Some(Regular));
    for s in ["Card.frame", "Button.base", "Field.frame", "Slot.frame"] {
        assert_eq!(glass(s), None, "{s}");
    }
    for s in ["Button.base", "Field.frame", "Chip.base", "Slot.frame"] {
        let x = d
            .iter()
            .find(|x| x.selector == Selector::parse(s).expect("a selector"))
            .expect(s);
        assert!(
            x.draw.strokes.is_empty() && x.draw.cleared & og_ui::dress::CLEAR_STROKES != 0,
            "{s} is borderless"
        );
    }
}
