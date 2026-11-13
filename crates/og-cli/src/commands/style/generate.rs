use std::path::PathBuf;

const FINISHES: &[&str] = &[
    "flat", "gradient", "glass", "hologram", "neon", "paper", "bevel", "brush",
];

fn css(look: &str, scheme: &str, finish: Option<&str>) -> Result<String, String> {
    if look != "default"
        && !og_cook::cook::look::PRESETS
            .iter()
            .any(|(name, _)| *name == look)
    {
        return Err(format!("unknown look `{look}`; use `og look list`"));
    }
    let (accent, radius, treatment) = if look.contains("pixel") {
        ("#e9b949", "0px", "bevel")
    } else if look.contains("neon") {
        ("#40f0ff", "2px", "neon")
    } else if look.contains("paper") || look.contains("manuscript") || look.contains("pencil") {
        ("#b56b36", "3px", "paper")
    } else if look.contains("painting") || look.contains("watercolor") {
        ("#bb74ba", "12px", "brush")
    } else if look.contains("comic") || look.contains("rubber") {
        ("#ed9361", "14px", "gradient")
    } else {
        ("#698cff", "8px", "flat")
    };
    let finish = finish.unwrap_or(treatment);
    if !FINISHES.contains(&finish) {
        return Err(format!(
            "unknown finish `{finish}`; choose {}",
            FINISHES.join(", ")
        ));
    }
    if !matches!(scheme, "dark" | "light") {
        return Err("scheme must be dark or light".into());
    }
    // Copy the complete public control vocabulary as an editable starting point,
    // without retaining the engine's lower-priority layer or generated-file notice.
    let defaults = og_ui::DEFAULT_CSS
        .split_once("@layer og-default {\n")
        .unwrap()
        .1
        .rsplit_once('}')
        .unwrap()
        .0;
    let mut out = format!(
        "/* Editable CSS pack. Look: {look}; finish: {finish}.\n   Loaded in og-pack. Set data-scheme on the root to select a palette. */\n{defaults}\n"
    );
    if scheme == "light" {
        let dark = defaults
            .split_once(":root {")
            .unwrap()
            .1
            .split_once('}')
            .unwrap()
            .0;
        let light = defaults
            .split_once(":root[data-scheme=light] {")
            .unwrap()
            .1
            .split_once('}')
            .unwrap()
            .0;
        out.push_str(&format!(
            ":root {{{light}}}\n:root[data-scheme=dark] {{{dark}}}\n"
        ));
    }
    out.push_str(&format!(":root, :root[data-scheme=dark], :root[data-scheme=light] {{\n  --pack-accent: {accent};\n  --radius-button: {radius}; --radius-panel: {radius}; --radius-field: {radius};\n  --primary: var(--pack-accent);\n  --primary-hover: color-mix(in srgb, var(--pack-accent), white 20%);\n  --primary-active: color-mix(in srgb, var(--pack-accent), black 20%);\n  --on-primary: #101522;\n}}\n"));
    // The finish's values are root tokens; its rule names them with var(),
    // as `og style check` requires (docs/ui.md §3.1).
    let (tokens, treatment) = match finish {
        "flat" => ("", "box-shadow: none;"),
        "gradient" => (
            "--pack-finish-image: linear-gradient(180deg, #ffffff24, #00000024); --pack-finish-shadow: inset 0 1px 0 #ffffff60, 0 3px 0 #00000070;",
            "background-image: var(--pack-finish-image); box-shadow: var(--pack-finish-shadow);",
        ),
        "glass" => (
            "--pack-finish-fill: #182434b0; --pack-finish-blur: blur(16px) saturate(130%);",
            "background: var(--pack-finish-fill); backdrop-filter: var(--pack-finish-blur); -og-material: glass regular lift;",
        ),
        "hologram" => (
            "--pack-finish-fill: #102c3eaa; --pack-finish-image: repeating-linear-gradient(0deg, transparent 0px, transparent 3px, #40f0ff12 4px);",
            "background: var(--pack-finish-fill); border: var(--stroke-hairline) solid var(--pack-accent); background-image: var(--pack-finish-image);",
        ),
        "neon" => (
            "--pack-finish-shadow: inset 0 0 8px #40f0ff40, 0 0 12px #40f0ff50;",
            "border: var(--stroke-regular) solid var(--pack-accent); box-shadow: var(--pack-finish-shadow);",
        ),
        "paper" => (
            "--pack-finish-image: linear-gradient(110deg, #e1b37718, #fff5db20); --pack-finish-edge: #86623c; --pack-finish-shadow: 2px 3px 0 #33201050;",
            "background-image: var(--pack-finish-image); border: var(--stroke-hairline) solid var(--pack-finish-edge); box-shadow: var(--pack-finish-shadow);",
        ),
        "bevel" => (
            "--pack-finish-shadow: inset 2px 2px 0 #ffffff30, inset -2px -2px 0 #00000050;",
            "border: var(--stroke-thick) outset var(--pack-accent); border-radius: 0; box-shadow: var(--pack-finish-shadow);",
        ),
        "brush" => (
            "--pack-finish-image: linear-gradient(125deg, #ffffff24, transparent 40%, #8b427c24); --pack-finish-radius: 12px 5px 14px 7px;",
            "background-image: var(--pack-finish-image); border: var(--stroke-regular) solid var(--pack-accent); border-radius: var(--pack-finish-radius);",
        ),
        _ => unreachable!(),
    };
    out.push_str(&format!(":root {{\n  {tokens}\n  --pack-sfx-tap: url(/@styles/shared/sfx-tap.ogt); --pack-sfx-tap2: url(/@styles/shared/sfx-tap2.ogt); --pack-sfx-confirm: url(/@styles/shared/sfx-confirm.ogt);\n  --pack-sfx-back: url(/@styles/shared/sfx-back.ogt); --pack-sfx-open: url(/@styles/shared/sfx-open.ogt); --pack-sfx-error: url(/@styles/shared/sfx-error.ogt); --pack-sfx-reward: url(/@styles/shared/sfx-reward.ogt);\n}}\n"));
    out.push_str(&format!("button::part(base), panel::part(frame), dialog::part(panel), input::part(frame), textarea::part(frame), select::part(trigger), slot::part(frame) {{ {treatment} }}\n"));
    out.push_str(":root { -og-sound: press var(--pack-sfx-tap) var(--pack-sfx-tap2) volume(0.24 0.3) pitch(0.98 1.02), select var(--pack-sfx-confirm) volume(0.3), confirm var(--pack-sfx-confirm) volume(0.3), cancel var(--pack-sfx-back) volume(0.25), back var(--pack-sfx-back) volume(0.25), open var(--pack-sfx-open) volume(0.2), close var(--pack-sfx-back) volume(0.2), error var(--pack-sfx-error) volume(0.3), reward var(--pack-sfx-reward) volume(0.3); }\n");
    out.push_str("button::part(base) { -og-sound: press var(--pack-sfx-tap) volume(0.24 0.3) pitch(0.98 1.02), hover var(--pack-sfx-tap2) volume(0.12); }\n@media (prefers-reduced-motion: reduce) { button::part(base) { transition: none; animation: none; } }\n");
    og_ui::style_pack::StylePack::parse("pack", vec![("pack/main.css".into(), out.clone())])?;
    Ok(out)
}

pub(super) fn run(args: &[&str]) -> Result<(), String> {
    let Some((&name, rest)) = args.split_first() else {
        return Err(super::USAGE.into());
    };
    if name.is_empty()
        || !name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err("name must contain only lowercase letters, digits and hyphens".into());
    }
    let mut look = "default";
    let mut scheme = "dark";
    let mut finish = None;
    let mut out = PathBuf::from("assets/styles").join(name);
    for pair in rest.chunks(2) {
        match pair {
            ["--look", value] => look = value,
            ["--scheme", value] => scheme = value,
            ["--finish", value] => finish = Some(*value),
            ["--out", value] => out = PathBuf::from(value),
            _ => return Err(super::USAGE.into()),
        }
    }
    let text = css(look, scheme, finish)?;
    if out.exists() {
        return Err(format!(
            "{} already exists; edit its CSS directly",
            out.display()
        ));
    }
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    std::fs::write(out.join("main.css"), text).map_err(|e| e.to_string())?;
    println!("{}", out.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use og_ui::prelude::*;

    #[test]
    fn every_finish_and_look_is_a_loadable_css_pack() {
        for (look, _) in og_cook::cook::look::PRESETS {
            for scheme in ["dark", "light"] {
                css(look, scheme, None).unwrap();
            }
        }
        for finish in FINISHES {
            css("default", "dark", Some(finish)).unwrap();
        }
        assert!(css("unknown", "dark", None).is_err());
    }

    #[test]
    fn look_accent_survives_both_scheme_selectors() {
        let pack = og_ui::style_pack::StylePack::parse(
            "pack",
            vec![(
                "pack/main.css".into(),
                css("neon-vector", "light", None).unwrap(),
            )],
        )
        .unwrap();
        for theme in [og_ui::Theme::dark(), og_ui::Theme::light()] {
            let mut ui = og_ui::Ui::new(og_ui::UiDesc {
                theme,
                ..Default::default()
            });
            ui.set_style_pack(&pack).unwrap();
            assert_eq!(
                ui.theme().palette.primary,
                og_ui::Srgba::rgb(0x40, 0xf0, 0xff)
            );
        }
    }
}
